use once_cell::sync::OnceCell;
use windows::{
  core::Error as WinError,
  Win32::{
    Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
    UI::{
      Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_LWIN},
      Shell::{SHQueryUserNotificationState, QUNS_BUSY, QUNS_RUNNING_D3D_FULL_SCREEN},
      WindowsAndMessaging::{
        CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
      },
    },
  },
};

static KEYBOARD_HOOK: OnceCell<HHOOK> = OnceCell::new();

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

  match (ev.vkCode as VIRTUAL_KEY, wparam.0 as u32) {
    (VK_LWIN, WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP) => {
      let state = unsafe { SHQueryUserNotificationState().unwrap() };
      if state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN {
        return LRESULT(1);
      }
    },
    _ => {},
  };

  unsafe { CallNextHookEx(KEYBOARD_HOOK.get().unwrap(), code, wparam, lparam) }
}

pub(crate) fn attach() {
  let hhk =
    unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), HINSTANCE::default(), 0) };
  KEYBOARD_HOOK.set(hhk).unwrap();
}

pub(crate) fn detach() -> Result<(), WinError> {
  if unsafe { UnhookWindowsHookEx(KEYBOARD_HOOK.get().unwrap()) }.0 == 0 {
    return Err(WinError::from_win32());
  }

  Ok(())
}
