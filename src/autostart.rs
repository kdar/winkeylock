use std::{env, iter, os::windows::ffi::OsStrExt, ptr};
use windows::{
  core::Error as WinError,
  Win32::{
    Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS},
    System::Registry::{
      RegCloseKey, RegCreateKeyExW, RegGetValueW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
      KEY_CREATE_SUB_KEY, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_SZ,
    },
  },
};

pub use windows::Win32::Foundation::ERROR_ACCESS_DENIED;
use windows::Win32::System::Registry::{RegDeleteValueW, RegOpenKeyExW};

use crate::wide_string::ToWide;

const AUTOSTART_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

pub fn add(app_name: &str) -> Result<(), WinError> {
  unsafe {
    let mut hkey = HKEY::default();

    let res = RegCreateKeyExW(
      HKEY_CURRENT_USER,
      AUTOSTART_SUBKEY.to_wide().as_pwstr(),
      0,
      None,
      0,
      KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
      std::ptr::null(),
      &mut hkey,
      std::ptr::null_mut(),
    );
    if res != ERROR_SUCCESS {
      return Err(WinError::from_win32());
    }

    let path = env::current_exe()
      .unwrap()
      .as_os_str()
      .encode_wide()
      .chain(iter::once(0))
      .collect::<Vec<u16>>();

    match RegSetValueExW(
      hkey,
      app_name.to_wide().as_pwstr(),
      0,
      REG_SZ,
      path.as_ptr() as *const _,
      path.len() as u32 * 2,
    ) {
      ERROR_SUCCESS => Ok(()),
      _ => Err(WinError::from_win32()),
    }
  }
}

pub fn check(app_name: &str) -> bool {
  unsafe {
    matches!(
      RegGetValueW(
        HKEY_CURRENT_USER,
        AUTOSTART_SUBKEY.to_wide().as_pwstr(),
        app_name.to_wide().as_pwstr(),
        RRF_RT_REG_SZ,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut()
      ),
      ERROR_SUCCESS | ERROR_MORE_DATA
    )
  }
}

pub fn remove(app_name: &str) -> Result<(), WinError> {
  unsafe {
    let mut hkey = HKEY::default();

    if RegOpenKeyExW(
      HKEY_CURRENT_USER,
      AUTOSTART_SUBKEY.to_wide().as_pwstr(),
      0,
      KEY_SET_VALUE,
      &mut hkey,
    ) != ERROR_SUCCESS
    {
      return Err(WinError::from_win32());
    }

    if RegDeleteValueW(hkey, app_name.to_wide().as_pwstr()) != ERROR_SUCCESS {
      return Err(WinError::from_win32());
    }

    if RegCloseKey(hkey) != ERROR_SUCCESS {
      return Err(WinError::from_win32());
    }

    Ok(())
  }
}
