use std::sync::atomic::{AtomicBool, Ordering};

use once_cell::sync::OnceCell;
use windows::{
  Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::{
      Input::KeyboardAndMouse::{
        VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL,
        VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
      },
      Shell::{QUNS_BUSY, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState},
      WindowsAndMessaging::{
        CallNextHookEx, HHOOK, KBDLLHOOKSTRUCT, SetWindowsHookExW, UnhookWindowsHookEx,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
      },
    },
  },
  core::Error as WinError,
};

use crate::config::ConfigManager;

static SHIFT_DOWN: AtomicBool = AtomicBool::new(false);
static CTRL_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);
static WIN_DOWN: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
struct UnsafePtr {
  ptr: *mut core::ffi::c_void,
}

unsafe impl Send for UnsafePtr {}
unsafe impl Sync for UnsafePtr {}

static KEYBOARD_HOOK: OnceCell<UnsafePtr> = OnceCell::new();
static CONFIG_MANAGER: OnceCell<ConfigManager> = OnceCell::new();

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code < 0 {
    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
  }

  let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
  let vk = ev.vkCode as u16;
  let msg = wparam.0 as u32;

  let is_keydown = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
  // let is_keyup = msg == WM_KEYUP || msg == WM_SYSKEYUP;

  // Update modifier key states
  match VIRTUAL_KEY(vk) {
    VK_LSHIFT | VK_RSHIFT | VK_SHIFT => SHIFT_DOWN.store(is_keydown, Ordering::Relaxed),
    VK_LCONTROL | VK_RCONTROL | VK_CONTROL => CTRL_DOWN.store(is_keydown, Ordering::Relaxed),
    VK_LMENU | VK_RMENU | VK_MENU => ALT_DOWN.store(is_keydown, Ordering::Relaxed),
    VK_LWIN | VK_RWIN => WIN_DOWN.store(is_keydown, Ordering::Relaxed),
    _ => {},
  };

  let shift = SHIFT_DOWN.load(Ordering::Relaxed);
  let ctrl = CTRL_DOWN.load(Ordering::Relaxed);
  let alt = ALT_DOWN.load(Ordering::Relaxed);
  let win = WIN_DOWN.load(Ordering::Relaxed);

  // Only check configuration for key down events
  if is_keydown {
    if let Some(config_manager) = CONFIG_MANAGER.get() {
      if config_manager.should_block(vk, shift, ctrl, alt, win) {
        // Check notification state for backwards compatibility
        let state = unsafe { SHQueryUserNotificationState().unwrap_or(QUNS_BUSY) };
        if state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN {
          return LRESULT(1);
        }
      }
    } else {
      // Fallback to old behavior if config is not available
      let is_win_key = vk == VK_LWIN.0 || vk == VK_RWIN.0;
      if is_win_key && !(shift || ctrl || alt) {
        let state = unsafe { SHQueryUserNotificationState().unwrap_or(QUNS_BUSY) };
        if state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN {
          return LRESULT(1);
        }
      }
    }
  }

  unsafe {
    CallNextHookEx(
      Some(HHOOK(KEYBOARD_HOOK.get().unwrap().ptr)),
      code,
      wparam,
      lparam,
    )
  }
}

pub(crate) fn attach() {
  // Initialize configuration manager with file watching
  match ConfigManager::new() {
    Ok(config_manager) => {
      CONFIG_MANAGER.set(config_manager).unwrap();
      println!("Configuration manager initialized with file watching");
    },
    Err(e) => {
      eprintln!("Failed to initialize config manager: {}", e);
      eprintln!("Continuing with fallback behavior");
    },
  }

  let hhk = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), None, 0) };
  KEYBOARD_HOOK
    .set(UnsafePtr {
      ptr: hhk.unwrap().0,
    })
    .unwrap();
}

pub(crate) fn detach() -> Result<(), WinError> {
  unsafe {
    UnhookWindowsHookEx(HHOOK(KEYBOARD_HOOK.get().unwrap().ptr))?;
  }

  Ok(())
}

// Note: Configuration changes require application restart for now
// pub(crate) fn reload_config() {
//   if let Some(_config) = CONFIG.get() {
//     let _new_config = KeyConfig::load();
//     // We can't easily replace the static config, so we'll need to restart the hook
//     // For now, this is a limitation - config changes require app restart
//     eprintln!("Configuration reloaded. Note: Restart the application for changes to take effect.");
//   }
// }

pub(crate) fn get_config_path() -> std::path::PathBuf {
  if let Some(config_manager) = CONFIG_MANAGER.get() {
    config_manager.get_config_path()
  } else {
    // Fallback if config manager not initialized
    let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("winkeylock");
    path.push("config.json");
    path
  }
}
