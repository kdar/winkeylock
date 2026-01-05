use std::sync::atomic::{AtomicBool, Ordering};

use once_cell::sync::OnceCell;
use windows::{
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow},
    UI::{
      Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        SendInput, VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU,
        VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
      },
      Shell::{QUNS_BUSY, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState},
      WindowsAndMessaging::{
        CallNextHookEx, EnumChildWindows, GWL_STYLE, GetForegroundWindow, GetWindowLongPtrW,
        GetWindowRect, HHOOK, KBDLLHOOKSTRUCT, SetWindowsHookExW, UnhookWindowsHookEx,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WS_CAPTION, WS_SYSMENU,
      },
    },
  },
  core::{BOOL, Error as WinError},
};

use crate::config::{ConfigManager, DetectMethod};

static SHIFT_DOWN: AtomicBool = AtomicBool::new(false);
static CTRL_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);
static WIN_DOWN: AtomicBool = AtomicBool::new(false);

// Track if another key was pressed while a modifier was held
// If true, the modifier was used in a combo and shouldn't be blocked on keyup
static WIN_USED_IN_COMBO: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
struct UnsafePtr {
  ptr: *mut core::ffi::c_void,
}

unsafe impl Send for UnsafePtr {}
unsafe impl Sync for UnsafePtr {}

static KEYBOARD_HOOK: OnceCell<UnsafePtr> = OnceCell::new();
static CONFIG_MANAGER: OnceCell<ConfigManager> = OnceCell::new();

extern "system" fn enum_child_cb(_hwnd: HWND, lparam: LPARAM) -> BOOL {
  unsafe {
    // lparam points to our counter
    let counter = &mut *(lparam.0 as *mut u32);
    *counter += 1;
  }
  // TRUE = continue enumeration
  BOOL(1)
}

pub fn count_child_windows(hwnd: HWND) -> u32 {
  let mut count: u32 = 0;

  unsafe {
    _ = EnumChildWindows(
      Some(hwnd),
      Some(enum_child_cb),
      LPARAM(&mut count as *mut _ as isize),
    );
  }

  count
}

fn is_foreground_game_windowstyle() -> bool {
  unsafe {
    let hwnd = GetForegroundWindow();
    if hwnd.is_invalid() {
      return false;
    }

    let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
    let looks_like_game = (style & WS_SYSMENU.0) == 0 && (style & WS_CAPTION.0) == 0;
    if !looks_like_game {
      return false;
    }

    if count_child_windows(hwnd) > 0 {
      return false;
    }
  }

  true
}

pub fn is_foreground_fullscreen() -> bool {
  unsafe {
    let hwnd = GetForegroundWindow();
    if hwnd.is_invalid() {
      return false;
    }

    let mut win_rect = RECT::default();
    if !GetWindowRect(hwnd, &mut win_rect).is_ok() {
      return false;
    }

    let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
    let mut mi = MONITORINFO {
      cbSize: std::mem::size_of::<MONITORINFO>() as u32,
      ..Default::default()
    };

    if !GetMonitorInfoW(hmon, &mut mi).as_bool() {
      return false;
    }

    win_rect.left <= mi.rcMonitor.left
      && win_rect.top <= mi.rcMonitor.top
      && win_rect.right >= mi.rcMonitor.right
      && win_rect.bottom >= mi.rcMonitor.bottom
  }
}

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code < 0 {
    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
  }

  let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
  let vk = ev.vkCode as u16;
  let msg = wparam.0 as u32;

  let is_keydown = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
  let is_keyup = msg == WM_KEYUP || msg == WM_SYSKEYUP;

  let is_win_key = matches!(VIRTUAL_KEY(vk), VK_LWIN | VK_RWIN);

  // Update modifier key states
  match VIRTUAL_KEY(vk) {
    VK_LSHIFT | VK_RSHIFT | VK_SHIFT => SHIFT_DOWN.store(is_keydown, Ordering::Relaxed),
    VK_LCONTROL | VK_RCONTROL | VK_CONTROL => CTRL_DOWN.store(is_keydown, Ordering::Relaxed),
    VK_LMENU | VK_RMENU | VK_MENU => ALT_DOWN.store(is_keydown, Ordering::Relaxed),
    VK_LWIN | VK_RWIN => {
      if is_keydown {
        WIN_DOWN.store(true, Ordering::Relaxed);
        // Reset combo tracking when win key is pressed
        WIN_USED_IN_COMBO.store(false, Ordering::Relaxed);
      } else {
        WIN_DOWN.store(false, Ordering::Relaxed);
      }
    },
    _ => {},
  };

  let shift = SHIFT_DOWN.load(Ordering::Relaxed);
  let ctrl = CTRL_DOWN.load(Ordering::Relaxed);
  let alt = ALT_DOWN.load(Ordering::Relaxed);
  let win = WIN_DOWN.load(Ordering::Relaxed);

  // If a non-modifier key is pressed while win is held, mark win as used in combo
  if is_keydown && !is_win_key && win {
    WIN_USED_IN_COMBO.store(true, Ordering::Relaxed);
  }

  if let Some(config_manager) = CONFIG_MANAGER.get() {
    // For win key: check on keyup whether to cancel Start menu
    // This allows win+X combos to work while blocking win alone
    if is_win_key && is_keyup {
      let win_used = WIN_USED_IN_COMBO.load(Ordering::Relaxed);

      // If win was NOT used in a combo and should be blocked, inject Ctrl to cancel Start menu
      if !win_used
        && config_manager.should_block(vk, shift, ctrl, alt, false)
        && should_block_in_current_context(config_manager)
      {
        // Inject a Ctrl key press+release to cancel the Start menu trigger
        // This is a common technique - pressing any other key while Win is held cancels Start menu
        inject_ctrl_tap();
      }
      // Always let the keyup through so the key doesn't get stuck
    }

    // For non-modifier keys on keydown: check whitelist/blacklist as before
    if is_keydown && !is_win_key {
      if config_manager.should_block(vk, shift, ctrl, alt, win) {
        if should_block_in_current_context(config_manager) {
          return LRESULT(1);
        }
      }
    }
  } else {
    // Fallback to old behavior if config is not available
    if is_win_key && is_keyup && !(shift || ctrl || alt) {
      let win_used = WIN_USED_IN_COMBO.load(Ordering::Relaxed);
      if !win_used {
        let state = unsafe { SHQueryUserNotificationState().unwrap_or(QUNS_BUSY) };
        if state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN {
          inject_ctrl_tap();
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

fn should_block_in_current_context(config_manager: &ConfigManager) -> bool {
  match config_manager.detect_method() {
    DetectMethod::NotificationState => {
      let state = unsafe { SHQueryUserNotificationState().unwrap_or(QUNS_BUSY) };
      state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN
    },
    DetectMethod::Fullscreen => is_foreground_fullscreen(),
    DetectMethod::WindowStyle => is_foreground_game_windowstyle(),
  }
}

fn inject_ctrl_tap() {
  let inputs = [
    INPUT {
      r#type: INPUT_KEYBOARD,
      Anonymous: INPUT_0 {
        ki: KEYBDINPUT {
          wVk: VK_CONTROL,
          wScan: 0,
          dwFlags: KEYBD_EVENT_FLAGS(0),
          time: 0,
          dwExtraInfo: 0,
        },
      },
    },
    INPUT {
      r#type: INPUT_KEYBOARD,
      Anonymous: INPUT_0 {
        ki: KEYBDINPUT {
          wVk: VK_CONTROL,
          wScan: 0,
          dwFlags: KEYEVENTF_KEYUP,
          time: 0,
          dwExtraInfo: 0,
        },
      },
    },
  ];

  unsafe {
    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
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
