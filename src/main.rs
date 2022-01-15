use once_cell::sync::OnceCell;
use windows::Win32::{
  Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
  UI::{
    Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_LWIN},
    Shell::{SHQueryUserNotificationState, QUNS_BUSY, QUNS_RUNNING_D3D_FULL_SCREEN},
    WindowsAndMessaging::{
      CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
      UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
      WM_SYSKEYDOWN, WM_SYSKEYUP,
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

fn main() {
  let hhk =
    unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), HINSTANCE::default(), 0) };
  KEYBOARD_HOOK.set(hhk).unwrap();

  let mut message = MSG::default();
  unsafe {
    while GetMessageW(&mut message, HWND(0), 0, 0).into() {
      TranslateMessage(&mut message);
      DispatchMessageW(&mut message);
    }
  }

  if unsafe { UnhookWindowsHookEx(hhk) }.0 == 0 {
    panic!("{:?}", unsafe { GetLastError() });
  }
}
