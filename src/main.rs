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
mod disable_winkey;
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

fn already_running() -> Result<bool, Box<dyn Error>> {
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
      return Ok(true);
    }
  }

  Ok(false)
}

fn main() -> Result<(), Box<dyn Error>> {
  match already_running() {
    Ok(true) => return Ok(()),
    Ok(_) => (),
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

  let elevate_i = MenuItem::new("Run as administrator", true, None);
  let autorun_i = CheckMenuItem::new(
    "Run when windows starts",
    true,
    autostart::check(APP_NAME),
    None,
  );

  let quit_i = MenuItem::new("Quit", true, None);
  tray_menu.append_items(&[&autorun_i])?;
  if !ECommand::is_elevated() {
    tray_menu.append(&elevate_i)?;
  }
  tray_menu.append_items(&[&PredefinedMenuItem::separator(), &quit_i])?;

  let mut tray_icon = None;

  disable_winkey::attach();

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
        disable_winkey::detach().ok();
      },
      Event::UserEvent(UserEvent::TrayIconEvent(event)) => {
        match event {
          TrayIconEvent::Click {
            button: MouseButton::Right,
            button_state: MouseButtonState::Down,
            ..
          } => {
            autorun_i.set_checked(autostart::check(APP_NAME));
          },
          _ => (),
        };
      },
      Event::UserEvent(UserEvent::MenuEvent(event)) => {
        if event.id == autorun_i.id() {
          if autostart::check(APP_NAME) {
            match autostart::remove(APP_NAME) {
              Ok(_) => {
                autorun_i.set_checked(false);
              },
              Err(e) => unsafe {
                MessageBoxW(
                  None,
                  format!("{:?}", e.message()).to_wide().as_pwstr(),
                  w!("Error removing autostart"),
                  MB_ICONERROR,
                );
              },
            };
          } else {
            match autostart::add(APP_NAME) {
              Ok(_) => {
                autorun_i.set_checked(true);
              },
              Err(e) => unsafe {
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
          match elevate() {
            Ok(_) => {
              // elevate_i.set_checked(true);
              *control_flow = ControlFlow::Exit;
              tray_icon.take();
            },
            Err(e) => unsafe {
              MessageBoxW(
                None,
                format!("{:?}", e).to_wide().as_pwstr(),
                w!("Error elevating permissions"),
                MB_ICONERROR,
              );
            },
          };
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
