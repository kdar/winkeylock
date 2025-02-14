use once_cell::sync::OnceCell;
use windows::{
  core::Error as WinError,
  Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
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

#[derive(Debug)]
struct UnsafePtr {
  ptr: *mut core::ffi::c_void,
}

unsafe impl Send for UnsafePtr {}
unsafe impl Sync for UnsafePtr {}

static KEYBOARD_HOOK: OnceCell<UnsafePtr> = OnceCell::new();

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

  match (VIRTUAL_KEY(ev.vkCode as u16), wparam.0 as u32) {
    (VK_LWIN, WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP) => {
      let state = unsafe { SHQueryUserNotificationState().unwrap() };
      // This is a bit hacky as it will probably disable the winkey in other applications in which
      // the notification state is busy. We could add exceptions by window title/class later.
      if state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN {
        return LRESULT(1);
      }
    },
    _ => {},
  };

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
