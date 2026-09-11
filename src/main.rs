#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::{env, error::Error, process::Command};

use elevated_command::Command as ECommand;
use tao::{
  self,
  event::Event,
  event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::{
  MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
  menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use windows::Win32::{
  Foundation::{ERROR_ALREADY_EXISTS, GetLastError},
  System::Threading::CreateMutexW,
  UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxW},
};
use windows_strings::w;

use wide_string::ToWide;

mod autostart;
mod config;
mod config_ui;
mod disable_key;
mod wide_string;

const APP_NAME: &str = "winkeylock";

#[derive(Debug)]
enum UserEvent {
  TrayIconEvent(tray_icon::TrayIconEvent),
  MenuEvent(tray_icon::menu::MenuEvent),
}

fn elevate() -> Result<(), Box<dyn Error>> {
  let mut cmd = Command::new(env::current_exe()?.to_str().unwrap());
  cmd.args(env::args());
  let elevated_cmd = ECommand::new(cmd);
  elevated_cmd.output()?;
  Ok(())
}

fn already_running() -> Result<Option<windows::Win32::Foundation::HANDLE>, Box<dyn Error>> {
  let mutex_name = env!("CARGO_CRATE_NAME").to_wide();

  unsafe {
    let handle = CreateMutexW(
      None,                  // Default security attributes
      false.into(),          // Do not acquire ownership immediately
      mutex_name.as_pwstr(), // Name of the mutex
    )?;

    if handle.is_invalid() {
      return Err("failed to create mutex".into());
    }

    if GetLastError() == ERROR_ALREADY_EXISTS {
      windows::Win32::Foundation::CloseHandle(handle).ok();
      return Ok(None);
    }
    
    Ok(Some(handle))
  }
}

use tracing::{info, error, warn, debug};
use tracing_subscriber::EnvFilter;

fn init_logging() -> Result<tracing_appender::non_blocking::WorkerGuard, Box<dyn Error>> {
  let mut log_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
  log_dir.push("winkeylock");
  log_dir.push("logs");
  std::fs::create_dir_all(&log_dir)?;

  let file_appender = tracing_appender::rolling::daily(log_dir, "winkeylock.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .with_writer(non_blocking)
    .with_ansi(false)
    .init();

  Ok(guard)
}

fn main() -> Result<(), Box<dyn Error>> {
  let _log_guard = match init_logging() {
    Ok(g) => Some(g),
    Err(e) => {
      eprintln!("Failed to init logging: {}", e);
      None
    }
  };

  info!("Winkeylock starting up...");

  // Check if we should run the config UI
  let args: Vec<String> = env::args().collect();
  if args.iter().any(|arg| arg == "--config-ui") {
    if let Err(e) = config_ui::run_config_ui_main() {
      eprintln!("Failed to run config UI: {}", e);
      std::process::exit(1);
    }
    return Ok(());
  }

  let mutex_handle = match already_running() {
    Ok(None) => {
      info!("Another instance is already running. Exiting.");
      return Ok(())
    },
    Ok(Some(handle)) => handle,
    Err(e) => unsafe {
      MessageBoxW(
        None,
        format!("{:?}", e).to_wide().as_pwstr(),
        w!("Error with creating global mutex"),
        MB_ICONERROR,
      );
      return Err(e);
    },
  };

  let config = config::KeyConfig::load();
  info!("Loaded config: run_as_admin={}", config.run_as_admin);
  let is_elevated = ECommand::is_elevated();
  info!("Process elevated: {}", is_elevated);

  // If run_as_admin is enabled but we aren't elevated, elevate and exit this instance.
  if config.run_as_admin && !is_elevated {
    info!("Auto-elevating process due to config...");
    
    // Explicitly release the mutex so the newly elevated instance can start
    unsafe {
      windows::Win32::Foundation::CloseHandle(mutex_handle).ok();
    }
    
    match elevate() {
      Ok(_) => {
        info!("Successfully handed off to elevated process, exiting.");
        return Ok(())
      },
      Err(e) => {
        error!("Failed to auto-elevate: {}", e);
      }
    }
  }

  // Migration logic: if run_as_admin is enabled, they are elevated, and the old registry key exists
  if config.run_as_admin && is_elevated && autostart::check_registry(APP_NAME) {
    info!("Performing startup migration from registry to task scheduler...");
    if let Err(e) = autostart::remove_registry(APP_NAME) {
      error!("Failed to remove registry key during migration: {:?}", e);
    }
    match autostart::add_task(APP_NAME) {
      Ok(_) => info!("Successfully created task scheduler entry during migration."),
      Err(e) => error!("Failed to create task scheduler entry during migration: {:?}", e),
    }
  }

  let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

  // set a tray event handler that forwards the event and wakes up the event loop
  let proxy = event_loop.create_proxy();
  TrayIconEvent::set_event_handler(Some(move |event| {
    // println!("tray: {:?}", event);
    proxy.send_event(UserEvent::TrayIconEvent(event)).unwrap();
  }));

  // set a menu event handler that forwards the event and wakes up the event loop
  let proxy = event_loop.create_proxy();
  MenuEvent::set_event_handler(Some(move |event| {
    proxy.send_event(UserEvent::MenuEvent(event)).unwrap();
  }));

  let tray_menu = Menu::new();

  let elevate_i = CheckMenuItem::new("Run as administrator", true, config.run_as_admin, None);
  let autorun_i = CheckMenuItem::new(
    "Run when windows starts",
    true,
    autostart::check(APP_NAME),
    None,
  );
  let config_i = MenuItem::new("Open configuration", true, None);

  let quit_i = MenuItem::new("Quit", true, None);
  tray_menu.append_items(&[&autorun_i])?;
  tray_menu.append(&elevate_i)?;
  tray_menu.append_items(&[&config_i, &PredefinedMenuItem::separator(), &quit_i])?;

  let mut tray_icon = None;

  disable_key::attach();

  event_loop.run(move |event, _event_loop, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::NewEvents(tao::event::StartCause::Init) => {
        tray_icon = Some(
          TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu.clone()))
            .with_tooltip(APP_NAME)
            .with_icon(tray_icon::Icon::from_resource(1, None).unwrap())
            .build()
            .unwrap(),
        );
      },
      Event::LoopDestroyed => {
        disable_key::detach().ok();
      },
      Event::UserEvent(UserEvent::TrayIconEvent(event)) => {
        match event {
          TrayIconEvent::Click {
            button: MouseButton::Right,
            button_state: MouseButtonState::Down,
            ..
          } => {
            autorun_i.set_checked(autostart::check(APP_NAME));
            let current_config = config::KeyConfig::load();
            elevate_i.set_checked(current_config.run_as_admin);
          },
          _ => (),
        };
      },
      Event::UserEvent(UserEvent::MenuEvent(event)) => {
        if event.id == autorun_i.id() {
          let run_as_admin = config::KeyConfig::load().run_as_admin;
          info!("Menu: toggling autostart (run_as_admin={})", run_as_admin);
          if autostart::check(APP_NAME) {
            match autostart::remove(APP_NAME) {
              Ok(_) => {
                info!("Menu: successfully removed autostart");
                autorun_i.set_checked(false);
              },
              Err(e) => unsafe {
                error!("Menu: failed to remove autostart: {:?}", e);
                MessageBoxW(
                  None,
                  format!("{:?}", e.message()).to_wide().as_pwstr(),
                  w!("Error removing autostart"),
                  MB_ICONERROR,
                );
              },
            };
          } else {
            match autostart::add(APP_NAME, run_as_admin) {
              Ok(_) => {
                info!("Menu: successfully added autostart");
                autorun_i.set_checked(true);
              },
              Err(e) => unsafe {
                error!("Menu: failed to add autostart: {:?}", e);
                MessageBoxW(
                  None,
                  format!("{:?}", e.message()).to_wide().as_pwstr(),
                  "Error adding autostart".to_wide().as_pwstr(),
                  MB_ICONERROR,
                );
              },
            }
          }
        } else if event.id == elevate_i.id() {
          let mut current_config = config::KeyConfig::load();
          current_config.run_as_admin = !current_config.run_as_admin;
          current_config.save();
          elevate_i.set_checked(current_config.run_as_admin);
          info!("Menu: toggled run_as_admin to {}", current_config.run_as_admin);

          // If autostart is already checked, migrate it seamlessly to the new setting
          if autostart::check(APP_NAME) {
            info!("Menu: autostart is enabled, migrating to match new run_as_admin setting");
            // To avoid double UAC prompts (one for schtasks, one for app elevation),
            // we skip migration here if we are about to elevate the app.
            // The startup logic of the newly elevated instance will handle the migration.
            if !current_config.run_as_admin || ECommand::is_elevated() {
              info!("Menu: performing immediate autostart migration");
              if let Err(e) = autostart::add(APP_NAME, current_config.run_as_admin) {
                error!("Menu: failed to migrate autostart: {:?}", e);
              }
            } else {
              info!("Menu: deferring autostart migration to newly elevated instance");
            }
          }

          // If they turned it on and we aren't elevated, elevate now
          if current_config.run_as_admin && !ECommand::is_elevated() {
            info!("Menu: run_as_admin turned on, initiating elevation...");
            
            // Explicitly release the mutex so the newly elevated instance can start
            unsafe {
              windows::Win32::Foundation::CloseHandle(mutex_handle).ok();
            }
            
            match elevate() {
              Ok(_) => {
                info!("Menu: handed off to elevated process, exiting");
                *control_flow = ControlFlow::Exit;
                tray_icon.take();
              },
              Err(e) => unsafe {
                error!("Menu: failed to elevate permissions: {:?}", e);
                MessageBoxW(
                  None,
                  format!("{:?}", e).to_wide().as_pwstr(),
                  w!("Error elevating permissions"),
                  MB_ICONERROR,
                );
              },
            };
          }
        } else if event.id == config_i.id() {
          // Launch the config UI in a separate thread
          std::thread::spawn(|| {
            match config_ui::launch_config_ui() {
              Ok(_) => {},
              Err(e) => {
                eprintln!("Failed to launch config UI: {}", e);
                // Fallback: open the config file with the default editor
                let config_path = disable_key::get_config_path();

                // Ensure config file exists
                if !config_path.exists() {
                  let default_config = config::KeyConfig::default();
                  default_config.save();
                }

                if let Err(open_err) = opener::open(&config_path) {
                  eprintln!("Failed to open config file as fallback: {}", open_err);
                }
              },
            }
          });
        } else if event.id == quit_i.id() {
          *control_flow = ControlFlow::Exit;
          tray_icon.take();
        }
      },
      _ => {
        // println!("{:?}", v);
      },
    }
  });
}
