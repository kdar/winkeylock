use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use once_cell::sync::OnceCell;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::RECT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::Graphics::Gdi::GetMonitorInfoW;
use windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST;
use windows::Win32::Graphics::Gdi::MONITORINFO;
use windows::Win32::Graphics::Gdi::MonitorFromWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_KEYBOARD;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_KEYUP;
use windows::Win32::UI::Input::KeyboardAndMouse::SendInput;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LCONTROL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LMENU;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LSHIFT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LWIN;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RCONTROL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RMENU;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RSHIFT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RWIN;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT;
use windows::Win32::UI::Shell::QUNS_BUSY;
use windows::Win32::UI::Shell::QUNS_RUNNING_D3D_FULL_SCREEN;
use windows::Win32::UI::Shell::SHQueryUserNotificationState;
use windows::Win32::UI::WindowsAndMessaging::CallNextHookEx;
use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;
use windows::Win32::UI::WindowsAndMessaging::GWL_STYLE;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
use windows::Win32::UI::WindowsAndMessaging::HHOOK;
use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;
use windows::Win32::UI::WindowsAndMessaging::LLKHF_INJECTED;
use windows::Win32::UI::WindowsAndMessaging::SetWindowsHookExW;
use windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx;
use windows::Win32::UI::WindowsAndMessaging::WH_KEYBOARD_LL;
use windows::Win32::UI::WindowsAndMessaging::WM_KEYDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_KEYUP;
use windows::Win32::UI::WindowsAndMessaging::WM_SYSKEYDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_SYSKEYUP;
use windows::Win32::UI::WindowsAndMessaging::WS_CAPTION;
use windows::Win32::UI::WindowsAndMessaging::WS_SYSMENU;
use windows::core::BOOL;
use windows::core::Error as WinError;

use crate::config::ConfigManager;
use crate::config::DetectMethod;

static SHIFT_DOWN: AtomicBool = AtomicBool::new(false);
static CTRL_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);
static LWIN_DOWN: AtomicBool = AtomicBool::new(false);
static RWIN_DOWN: AtomicBool = AtomicBool::new(false);

// Track if an allowed non-modifier key was pressed while Win was held
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
  if ev.flags.contains(LLKHF_INJECTED) {
    return unsafe {
      CallNextHookEx(
        Some(HHOOK(KEYBOARD_HOOK.get().unwrap().ptr)),
        code,
        wparam,
        lparam,
      )
    };
  }

  let vk = ev.vkCode as u16;
  let msg = wparam.0 as u32;

  let is_keydown = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
  let is_keyup = msg == WM_KEYUP || msg == WM_SYSKEYUP;

  let is_win_key = matches!(VIRTUAL_KEY(vk), VK_LWIN | VK_RWIN);
  let is_modifier_key = matches!(
    VIRTUAL_KEY(vk),
    VK_LWIN
      | VK_RWIN
      | VK_LSHIFT
      | VK_RSHIFT
      | VK_SHIFT
      | VK_LCONTROL
      | VK_RCONTROL
      | VK_CONTROL
      | VK_LMENU
      | VK_RMENU
      | VK_MENU
  );

  // On keydown, set modifier state before reading; on keyup, clear it after reading
  // so the modifier being released is still reflected in the state for this event.
  if is_keydown {
    match VIRTUAL_KEY(vk) {
      VK_LSHIFT | VK_RSHIFT | VK_SHIFT => SHIFT_DOWN.store(true, Ordering::Relaxed),
      VK_LCONTROL | VK_RCONTROL | VK_CONTROL => CTRL_DOWN.store(true, Ordering::Relaxed),
      VK_LMENU | VK_RMENU | VK_MENU => ALT_DOWN.store(true, Ordering::Relaxed),
      VK_LWIN => {
        if !LWIN_DOWN.swap(true, Ordering::Relaxed) {
          WIN_USED_IN_COMBO.store(false, Ordering::Relaxed);
        }
      },
      VK_RWIN => {
        if !RWIN_DOWN.swap(true, Ordering::Relaxed) {
          WIN_USED_IN_COMBO.store(false, Ordering::Relaxed);
        }
      },
      _ => {},
    }
  }

  let shift = SHIFT_DOWN.load(Ordering::Relaxed);
  let ctrl = CTRL_DOWN.load(Ordering::Relaxed);
  let alt = ALT_DOWN.load(Ordering::Relaxed);
  let lwin = LWIN_DOWN.load(Ordering::Relaxed);
  let rwin = RWIN_DOWN.load(Ordering::Relaxed);

  if is_keyup {
    match VIRTUAL_KEY(vk) {
      VK_LSHIFT | VK_RSHIFT | VK_SHIFT => SHIFT_DOWN.store(false, Ordering::Relaxed),
      VK_LCONTROL | VK_RCONTROL | VK_CONTROL => CTRL_DOWN.store(false, Ordering::Relaxed),
      VK_LMENU | VK_RMENU | VK_MENU => ALT_DOWN.store(false, Ordering::Relaxed),
      VK_LWIN => LWIN_DOWN.store(false, Ordering::Relaxed),
      VK_RWIN => RWIN_DOWN.store(false, Ordering::Relaxed),
      _ => {},
    }
  }

  if let Some(config_manager) = CONFIG_MANAGER.get() {
    if is_modifier_key {
      let blocked = config_manager.should_block(None, shift, ctrl, alt, lwin, rwin)
        && should_block_in_current_context(config_manager);

      if !config_manager.has_potential_whitelist_match(shift, ctrl, alt, lwin, rwin) {
        if blocked {
          return LRESULT(1);
        }
      } else if is_keyup && is_win_key && blocked && !WIN_USED_IN_COMBO.load(Ordering::Relaxed) {
        // Win was allowed down for a potential whitelisted combo that never happened;
        // inject Ctrl to cancel the Windows Start menu on keyup.
        inject_ctrl_tap();
      }
    } else if is_keydown {
      if config_manager.should_block(Some(vk), shift, ctrl, alt, lwin, rwin)
        && should_block_in_current_context(config_manager)
      {
        return LRESULT(1);
      }

      if lwin || rwin {
        WIN_USED_IN_COMBO.store(true, Ordering::Relaxed);
      }
    }
  } else if is_win_key {
    let state = unsafe { SHQueryUserNotificationState().unwrap_or(QUNS_BUSY) };
    if state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN {
      return LRESULT(1);
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
