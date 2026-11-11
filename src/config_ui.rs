pub mod gpui_kit_ui;
pub mod iced_ui;

use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use once_cell::sync::Lazy;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::ERROR_ALREADY_EXISTS;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::System::Threading::CreateMutexW;
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
use windows::Win32::UI::WindowsAndMessaging::BringWindowToTop;
use windows::Win32::UI::WindowsAndMessaging::CallNextHookEx;
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::UI::WindowsAndMessaging::HHOOK;
use windows::Win32::UI::WindowsAndMessaging::IsIconic;
use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;
use windows::Win32::UI::WindowsAndMessaging::LLKHF_INJECTED;
use windows::Win32::UI::WindowsAndMessaging::SW_RESTORE;
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::SetWindowsHookExW;
use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
use windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx;
use windows::Win32::UI::WindowsAndMessaging::WH_KEYBOARD_LL;
use windows::Win32::UI::WindowsAndMessaging::WM_KEYDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_KEYUP;
use windows::Win32::UI::WindowsAndMessaging::WM_SYSKEYDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_SYSKEYUP;

use crate::config::format_key_combo;
use crate::wide_string::ToWide;

pub const CONFIG_WINDOW_TITLE: &str = "WinKeyLock Configuration";

const CONFIG_UI_MUTEX_NAME: &str = "winkeylock_config_ui";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiBackend {
  /// GPUI Kit with bundled component system (`gpui_kit_ui.rs`).
  GpuiKit,
  Iced,
}

impl UiBackend {
  pub fn as_str(&self) -> &'static str {
    match self {
      UiBackend::GpuiKit => "gpui-kit",
      UiBackend::Iced => "iced",
    }
  }

  pub fn from_arg(arg: &str) -> Option<Self> {
    match arg.trim().to_ascii_lowercase().as_str() {
      "gpui-kit" | "gpui_kit" | "gpui" | "gpui-component" => Some(UiBackend::GpuiKit),
      "iced" => Some(UiBackend::Iced),
      _ => None,
    }
  }
}

/// Switch this constant between `UiBackend::GpuiKit` and `UiBackend::Iced`
/// to change the default configuration UI backend.
/// You can also pass `--config-ui=gpui-kit` or `--config-ui=iced` on the command line.
pub const DEFAULT_BACKEND: UiBackend = UiBackend::GpuiKit;

/// Find an existing configuration window and bring it to the foreground if present.
pub fn focus_existing_config_window() -> bool {
  let wide_title = CONFIG_WINDOW_TITLE.to_wide();
  unsafe {
    if let Ok(hwnd) = FindWindowW(None, wide_title.as_pwstr())
      && !hwnd.is_invalid()
      && !hwnd.0.is_null()
    {
      if IsIconic(hwnd).as_bool() {
        let _ = ShowWindow(hwnd, SW_RESTORE);
      }
      let _ = BringWindowToTop(hwnd);
      let _ = SetForegroundWindow(hwnd);
      return true;
    }
  }
  false
}

fn acquire_config_ui_mutex() -> Result<Option<HANDLE>, Box<dyn std::error::Error>> {
  let mutex_name = CONFIG_UI_MUTEX_NAME.to_wide();
  unsafe {
    let handle = CreateMutexW(None, false.into(), mutex_name.as_pwstr())?;
    if handle.is_invalid() {
      return Err("failed to create config UI mutex".into());
    }
    if GetLastError() == ERROR_ALREADY_EXISTS {
      CloseHandle(handle).ok();
      return Ok(None);
    }
    Ok(Some(handle))
  }
}

// --- Low-level keyboard hook for capturing key combinations in the Config UI ---

static REC_SHIFT: AtomicBool = AtomicBool::new(false);
static REC_CTRL: AtomicBool = AtomicBool::new(false);
static REC_ALT: AtomicBool = AtomicBool::new(false);
static REC_LWIN: AtomicBool = AtomicBool::new(false);
static REC_RWIN: AtomicBool = AtomicBool::new(false);
static NON_MOD_PRESSED_IN_CHORD: AtomicBool = AtomicBool::new(false);
static INTERCEPTING_CHORD: AtomicBool = AtomicBool::new(false);
static RECORDING_MODE: AtomicBool = AtomicBool::new(false);

static RECORDED_COMBO: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub fn set_recording_mode(active: bool) {
  reset_recorder_state();
  if let Ok(mut slot) = RECORDED_COMBO.lock() {
    *slot = None;
  }
  RECORDING_MODE.store(active, Ordering::Relaxed);
}

pub fn is_recording_mode() -> bool {
  RECORDING_MODE.load(Ordering::Relaxed)
}

pub fn take_recorded_combo() -> Option<String> {
  RECORDED_COMBO.lock().ok().and_then(|mut g| g.take())
}

fn is_config_window_focused() -> bool {
  unsafe {
    let hwnd = GetForegroundWindow();
    if hwnd.is_invalid() || hwnd.0.is_null() {
      return false;
    }
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    pid == std::process::id()
  }
}

fn reset_recorder_state() {
  REC_SHIFT.store(false, Ordering::Relaxed);
  REC_CTRL.store(false, Ordering::Relaxed);
  REC_ALT.store(false, Ordering::Relaxed);
  REC_LWIN.store(false, Ordering::Relaxed);
  REC_RWIN.store(false, Ordering::Relaxed);
  NON_MOD_PRESSED_IN_CHORD.store(false, Ordering::Relaxed);
  INTERCEPTING_CHORD.store(false, Ordering::Relaxed);
}

extern "system" fn config_keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code < 0 {
    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
  }

  let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
  if ev.flags.contains(LLKHF_INJECTED) {
    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
  }

  if !is_config_window_focused() {
    reset_recorder_state();
    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
  }

  let vk = ev.vkCode as u16;
  let msg = wparam.0 as u32;
  let is_keydown = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
  let is_keyup = msg == WM_KEYUP || msg == WM_SYSKEYUP;

  let is_modifier = matches!(
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

  let mut modifier_newly_pressed = false;
  if is_keydown {
    match VIRTUAL_KEY(vk) {
      VK_LSHIFT | VK_RSHIFT | VK_SHIFT => {
        modifier_newly_pressed = !REC_SHIFT.swap(true, Ordering::Relaxed);
      },
      VK_LCONTROL | VK_RCONTROL | VK_CONTROL => {
        modifier_newly_pressed = !REC_CTRL.swap(true, Ordering::Relaxed);
      },
      VK_LMENU | VK_RMENU | VK_MENU => {
        modifier_newly_pressed = !REC_ALT.swap(true, Ordering::Relaxed);
      },
      VK_LWIN => {
        modifier_newly_pressed = !REC_LWIN.swap(true, Ordering::Relaxed);
      },
      VK_RWIN => {
        modifier_newly_pressed = !REC_RWIN.swap(true, Ordering::Relaxed);
      },
      _ => {},
    }
  }

  let shift = REC_SHIFT.load(Ordering::Relaxed);
  let ctrl = REC_CTRL.load(Ordering::Relaxed);
  let alt = REC_ALT.load(Ordering::Relaxed);
  let lwin = REC_LWIN.load(Ordering::Relaxed);
  let rwin = REC_RWIN.load(Ordering::Relaxed);

  if is_keyup {
    match VIRTUAL_KEY(vk) {
      VK_LSHIFT | VK_RSHIFT | VK_SHIFT => REC_SHIFT.store(false, Ordering::Relaxed),
      VK_LCONTROL | VK_RCONTROL | VK_CONTROL => REC_CTRL.store(false, Ordering::Relaxed),
      VK_LMENU | VK_RMENU | VK_MENU => REC_ALT.store(false, Ordering::Relaxed),
      VK_LWIN => REC_LWIN.store(false, Ordering::Relaxed),
      VK_RWIN => REC_RWIN.store(false, Ordering::Relaxed),
      _ => {},
    }
  }

  let any_mod_after = REC_SHIFT.load(Ordering::Relaxed)
    || REC_CTRL.load(Ordering::Relaxed)
    || REC_ALT.load(Ordering::Relaxed)
    || REC_LWIN.load(Ordering::Relaxed)
    || REC_RWIN.load(Ordering::Relaxed);

  let recording_mode = RECORDING_MODE.load(Ordering::Relaxed);
  // Only intercept and record keys when explicit Record mode is active (or finishing an active recorded chord).
  let should_intercept = recording_mode || INTERCEPTING_CHORD.load(Ordering::Relaxed);

  if should_intercept {
    if is_keydown {
      INTERCEPTING_CHORD.store(true, Ordering::Relaxed);
      if is_modifier {
        if modifier_newly_pressed && !NON_MOD_PRESSED_IN_CHORD.load(Ordering::Relaxed) {
          if let Some(combo_str) = format_key_combo(None, shift, ctrl, alt, lwin, rwin)
            && let Ok(mut slot) = RECORDED_COMBO.lock()
          {
            *slot = Some(combo_str);
          }
        }
      } else if let Some(combo_str) = format_key_combo(Some(vk), shift, ctrl, alt, lwin, rwin) {
        NON_MOD_PRESSED_IN_CHORD.store(true, Ordering::Relaxed);
        if let Ok(mut slot) = RECORDED_COMBO.lock() {
          *slot = Some(combo_str);
        }
        if !any_mod_after {
          RECORDING_MODE.store(false, Ordering::Relaxed);
          INTERCEPTING_CHORD.store(false, Ordering::Relaxed);
          NON_MOD_PRESSED_IN_CHORD.store(false, Ordering::Relaxed);
        }
      }
    } else if is_keyup && !any_mod_after {
      RECORDING_MODE.store(false, Ordering::Relaxed);
      INTERCEPTING_CHORD.store(false, Ordering::Relaxed);
      NON_MOD_PRESSED_IN_CHORD.store(false, Ordering::Relaxed);
    }

    return LRESULT(1);
  }

  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

struct HookGuard(HHOOK);

impl Drop for HookGuard {
  fn drop(&mut self) {
    unsafe {
      let _ = UnhookWindowsHookEx(self.0);
    }
  }
}

pub fn run_config_ui_with_backend(
  config: crate::config::KeyConfig,
  backend: UiBackend,
) -> Result<(), Box<dyn std::error::Error>> {
  let _hook_guard = unsafe {
    SetWindowsHookExW(WH_KEYBOARD_LL, Some(config_keyboard_hook), None, 0)
      .ok()
      .map(HookGuard)
  };

  match backend {
    UiBackend::GpuiKit => gpui_kit_ui::run(config),
    UiBackend::Iced => iced_ui::run(config),
  }
}

/// Helper function to launch the config UI in a separate process
pub fn launch_config_ui() -> Result<(), Box<dyn std::error::Error>> {
  launch_config_ui_with_backend(DEFAULT_BACKEND)
}

pub fn launch_config_ui_with_backend(backend: UiBackend) -> Result<(), Box<dyn std::error::Error>> {
  // If the config window is already open, bring it to the foreground instead of spawning another
  if focus_existing_config_window() {
    return Ok(());
  }

  let exe_path = std::env::current_exe()?;

  std::process::Command::new(&exe_path)
    .arg(format!("--config-ui={}", backend.as_str()))
    .spawn()?;

  Ok(())
}

/// Function to actually run the config UI (called when launched with `--config-ui` or `--config-ui=<backend>`)
pub fn run_config_ui_main(backend: Option<UiBackend>) -> Result<(), Box<dyn std::error::Error>> {
  let _mutex_handle = match acquire_config_ui_mutex()? {
    Some(handle) => handle,
    None => {
      focus_existing_config_window();
      return Ok(());
    },
  };

  let config = crate::config::KeyConfig::load();
  run_config_ui_with_backend(config, backend.unwrap_or(DEFAULT_BACKEND))
}
