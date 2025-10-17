use std::env;
use windows::{
  Win32::{
    Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS},
    System::Registry::{
      HKEY, HKEY_CURRENT_USER, KEY_CREATE_SUB_KEY, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ,
      RRF_RT_REG_SZ, RegCloseKey, RegCreateKeyExW, RegGetValueW, RegSetValueExW,
    },
  },
  core::Error as WinError,
};

use windows::Win32::System::Registry::{RegDeleteValueW, RegOpenKeyExW};

use crate::wide_string::ToWide;

const AUTOSTART_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

pub fn add(app_name: &str) -> Result<(), WinError> {
  unsafe {
    let mut hkey = HKEY::default();

    let res = RegCreateKeyExW(
      HKEY_CURRENT_USER,
      AUTOSTART_SUBKEY.to_wide().as_pwstr(),
      None,
      None,
      REG_OPTION_NON_VOLATILE,
      KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
      None,
      &mut hkey,
      None,
    );
    if res != ERROR_SUCCESS {
      return Err(WinError::from_thread());
    }

    let lpdata: Vec<u8> = env::current_exe()
      .unwrap()
      .to_str()
      .unwrap()
      .to_wide_u8_vec();
    match RegSetValueExW(
      hkey,
      app_name.to_wide().as_pwstr(),
      None,
      REG_SZ,
      Some(&lpdata),
    ) {
      ERROR_SUCCESS => Ok(()),
      _ => Err(WinError::from_thread()),
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
        None,
        None,
        None
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
      None,
      KEY_SET_VALUE,
      &mut hkey,
    ) != ERROR_SUCCESS
    {
      return Err(WinError::from_thread());
    }

    if RegDeleteValueW(hkey, app_name.to_wide().as_pwstr()) != ERROR_SUCCESS {
      return Err(WinError::from_thread());
    }

    if RegCloseKey(hkey) != ERROR_SUCCESS {
      return Err(WinError::from_thread());
    }

    Ok(())
  }
}
