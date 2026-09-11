use elevated_command::Command as ECommand;
use std::env;
use std::process::Command;
use std::os::windows::process::CommandExt;
use tracing::{debug, error, info};
use windows::{
  Win32::{
    Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS, E_FAIL},
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

pub fn add_registry(app_name: &str) -> Result<(), WinError> {
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

pub fn check_registry(app_name: &str) -> bool {
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

pub fn remove_registry(app_name: &str) -> Result<(), WinError> {
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

pub fn add_task(app_name: &str) -> Result<(), WinError> {
  let exe_path = env::current_exe().unwrap();
  let tr_arg = format!("\"{}\"", exe_path.display());

  let mut cmd = Command::new("schtasks");
  cmd.args([
    "/create", "/f", "/tn", app_name, "/tr", &tr_arg, "/sc", "onlogon", "/rl", "highest",
  ]);
  cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

  let success = if ECommand::is_elevated() {
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
  } else {
    ECommand::new(cmd).output().map(|o| o.status.success()).unwrap_or(false)
  };

  if success {
    Ok(())
  } else {
    Err(WinError::from(E_FAIL))
  }
}

pub fn check_task(app_name: &str) -> bool {
  let mut cmd = Command::new("schtasks");
  cmd.args(["/query", "/tn", app_name]);
  cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
  
  if let Ok(output) = cmd.output() {
    output.status.success()
  } else {
    false
  }
}

pub fn remove_task(app_name: &str) -> Result<(), WinError> {
  let mut cmd = Command::new("schtasks");
  cmd.args(["/delete", "/f", "/tn", app_name]);
  cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

  let success = if ECommand::is_elevated() {
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
  } else {
    ECommand::new(cmd).output().map(|o| o.status.success()).unwrap_or(false)
  };

  if success {
    Ok(())
  } else {
    Err(WinError::from(E_FAIL))
  }
}

pub fn add(app_name: &str, run_as_admin: bool) -> Result<(), WinError> {
  info!("autostart::add called with run_as_admin={}", run_as_admin);
  if run_as_admin {
    debug!("Cleaning up registry autostart before adding task...");
    let _ = remove_registry(app_name); // Cleanup old
    debug!("Adding task scheduler autostart...");
    add_task(app_name)
  } else {
    debug!("Cleaning up task autostart before adding registry...");
    let _ = remove_task(app_name); // Cleanup old
    debug!("Adding registry autostart...");
    add_registry(app_name)
  }
}

pub fn check(app_name: &str) -> bool {
  let reg = check_registry(app_name);
  let task = check_task(app_name);
  debug!("autostart::check results - registry: {}, task: {}", reg, task);
  reg || task
}

pub fn remove(app_name: &str) -> Result<(), WinError> {
  info!("autostart::remove called");
  let mut ret = Ok(());
  if check_registry(app_name) {
    debug!("Removing registry autostart...");
    if let Err(e) = remove_registry(app_name) {
      error!("Failed to remove registry autostart: {:?}", e);
      ret = Err(e);
    }
  }
  if check_task(app_name) {
    debug!("Removing task scheduler autostart...");
    if let Err(e) = remove_task(app_name) {
      error!("Failed to remove task scheduler autostart: {:?}", e);
      ret = Err(e);
    }
  }
  ret
}
