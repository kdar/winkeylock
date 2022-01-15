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
// static DISABLE_WIN: OnceCell<AtomicBool> = OnceCell::new();

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

  //   let scancode = ev.scanCode;

  match (ev.vkCode as VIRTUAL_KEY, wparam.0 as u32) {
    (VK_LWIN, WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP) => {
      // if DISABLE_WIN.get().unwrap().load(Ordering::Relaxed) {
      let state = unsafe { SHQueryUserNotificationState().unwrap() };
      if state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN {
        return LRESULT(1);
      }
    },
    _ => {
      //   println!("{:?}", ev);
    },
  };

  unsafe { CallNextHookEx(KEYBOARD_HOOK.get().unwrap(), code, wparam, lparam) }
}

// unsafe extern "system" fn foreground_hook(
//   _hwineventhook: HWINEVENTHOOK,
//   _event: u32,
//   hwnd: HWND,
//   _idobject: i32,
//   _idchild: i32,
//   _ideventthread: u32,
//   _dwmseventtime: u32,
// ) {
//   //   let mut text: [u16; 512] = [0; 512];
//   //   let len = GetClassNameW(hwnd, PWSTR(text.as_mut_ptr()), text.len() as i32);
//   //   let text = String::from_utf16_lossy(&text[..len as usize]);
//   //   println!("{:?}", text);

//   let state = SHQueryUserNotificationState().unwrap();
//   println!("{:?}", state);
//   if state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN {
//     DISABLE_WIN.get().unwrap().store(true, Ordering::Relaxed);
//   } else {
//     DISABLE_WIN.get().unwrap().store(false, Ordering::Relaxed);
//   }
// }

fn main() {
  // let fg_win = unsafe { GetForegroundWindow() };
  // println!("{:?}", fg_win);

  // DISABLE_WIN.set(AtomicBool::new(false)).unwrap();

  let hhk =
    unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), HINSTANCE::default(), 0) };
  KEYBOARD_HOOK.set(hhk).unwrap();

  // unsafe {
  //   SetWinEventHook(
  //     EVENT_SYSTEM_FOREGROUND,
  //     EVENT_SYSTEM_FOREGROUND,
  //     HINSTANCE::default(),
  //     Some(foreground_hook),
  //     0,
  //     0,
  //     WINEVENT_OUTOFCONTEXT,
  //   );
  // }

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
