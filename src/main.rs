#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tao;
use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR},
};
use wry::application::{
  event::{Event, StartCause},
  event_loop::{ControlFlow, EventLoop},
  menu::{ContextMenu, MenuItemAttributes, MenuType},
  system_tray::SystemTrayBuilder,
};

use wide_string::ToWide;

mod autostart;
mod disable_winkey;
mod wide_string;

const APP_NAME: &str = "autowinkey";

fn main() -> wry::Result<()> {
  let mut autostart_enabled = autostart::check(APP_NAME);
  let event_loop = EventLoop::new();

  let mut tray_menu = ContextMenu::new();
  let mut autostart_item = tray_menu
    .add_item(MenuItemAttributes::new("Run when windows starts").with_selected(autostart_enabled));
  let quit_item = tray_menu.add_item(MenuItemAttributes::new("Quit"));

  let icon = include_bytes!("icon.ico").to_vec();

  let _system_tray = SystemTrayBuilder::new(icon.clone(), Some(tray_menu))
    .build(&event_loop)
    .unwrap();

  disable_winkey::attach();

  event_loop.run(move |event, _event_loop, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::NewEvents(StartCause::Init) => {},
      Event::TrayEvent {
        event: tao::event::TrayEvent::LeftClick,
        ..
      } => {},
      Event::LoopDestroyed => {
        disable_winkey::detach().ok();
      },
      Event::MenuEvent {
        menu_id,
        origin: MenuType::ContextMenu,
        ..
      } => {
        if menu_id == autostart_item.clone().id() {
          if autostart_enabled {
            match autostart::remove(APP_NAME) {
              Ok(_) => {
                autostart_item.set_selected(false);
                autostart_enabled = autostart::check(APP_NAME);
              },
              Err(e) => unsafe {
                MessageBoxW(
                  HWND::default(),
                  "Error removing autostart".to_wide().as_pwstr(),
                  format!("{:?}", e.message()).to_wide().as_pwstr(),
                  MB_ICONERROR,
                );
              },
            };
          } else {
            match autostart::add(APP_NAME) {
              Ok(_) => {
                autostart_item.set_selected(true);
                autostart_enabled = autostart::check(APP_NAME);
              },
              Err(e) => unsafe {
                MessageBoxW(
                  HWND::default(),
                  "Error adding autostart".to_wide().as_pwstr(),
                  format!("{:?}", e.message()).to_wide().as_pwstr(),
                  MB_ICONERROR,
                );
              },
            }
          }
        } else if menu_id == quit_item.clone().id() {
          *control_flow = ControlFlow::Exit;
        }
      },
      _ => {
        // println!("{:?}", v);
      },
    }
  });
}
