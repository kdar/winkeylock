use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCombo {
  pub key: u16,
  pub shift: bool,
  pub ctrl: bool,
  pub alt: bool,
  pub win: bool,
}

impl KeyCombo {
  pub fn new(key: VIRTUAL_KEY, shift: bool, ctrl: bool, alt: bool, win: bool) -> Self {
    Self {
      key: key.0,
      shift,
      ctrl,
      alt,
      win,
    }
  }

  pub fn matches(&self, key: u16, shift: bool, ctrl: bool, alt: bool, win: bool) -> bool {
    self.key == key
      && self.shift == shift
      && self.ctrl == ctrl
      && self.alt == alt
      && self.win == win
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
  /// Key combinations that should be blocked
  pub blacklist: Vec<KeyCombo>,
  /// Key combinations that should be explicitly allowed (overrides blacklist)
  pub whitelist: Vec<KeyCombo>,
}

impl Default for KeyConfig {
  fn default() -> Self {
    Self {
      // Default: block Windows key by itself
      blacklist: vec![
        KeyCombo::new(
          windows::Win32::UI::Input::KeyboardAndMouse::VK_LWIN,
          false,
          false,
          false,
          false,
        ),
        KeyCombo::new(
          windows::Win32::UI::Input::KeyboardAndMouse::VK_RWIN,
          false,
          false,
          false,
          false,
        ),
      ],
      // Default: allow common Windows shortcuts
      whitelist: vec![
        // Win + Shift + S (screenshot)
        KeyCombo::new(
          windows::Win32::UI::Input::KeyboardAndMouse::VK_S,
          true,
          false,
          false,
          true,
        ),
        // Win + D (show desktop)
        KeyCombo::new(
          windows::Win32::UI::Input::KeyboardAndMouse::VK_D,
          false,
          false,
          false,
          true,
        ),
        // Win + L (lock screen)
        KeyCombo::new(
          windows::Win32::UI::Input::KeyboardAndMouse::VK_L,
          false,
          false,
          false,
          true,
        ),
        // Win + E (file explorer)
        KeyCombo::new(
          windows::Win32::UI::Input::KeyboardAndMouse::VK_E,
          false,
          false,
          false,
          true,
        ),
        // Win + R (run dialog)
        KeyCombo::new(
          windows::Win32::UI::Input::KeyboardAndMouse::VK_R,
          false,
          false,
          false,
          true,
        ),
      ],
    }
  }
}

impl KeyConfig {
  pub fn should_block(&self, key: u16, shift: bool, ctrl: bool, alt: bool, win: bool) -> bool {
    // First check whitelist - if explicitly allowed, don't block
    for combo in &self.whitelist {
      if combo.matches(key, shift, ctrl, alt, win) {
        return false;
      }
    }

    // Then check blacklist - if explicitly blocked, block it
    for combo in &self.blacklist {
      println!(
        "combo: {}, {}, {}, {}, {}\nkey  : {}, {}, {}, {}, {}",
        combo.key, combo.shift, combo.ctrl, combo.alt, combo.win, key, shift, ctrl, alt, win,
      );
      if combo.matches(key, shift, ctrl, alt, win) {
        return true;
      }
    }

    // Default: don't block
    false
  }

  pub fn load() -> Self {
    let config_path = Self::config_path();

    if config_path.exists() {
      match fs::read_to_string(&config_path) {
        Ok(content) => {
          match serde_json::from_str(&content) {
            Ok(config) => return config,
            Err(e) => {
              eprintln!("Failed to parse config file: {}", e);
            },
          }
        },
        Err(e) => {
          eprintln!("Failed to read config file: {}", e);
        },
      }
    }

    // Return default config and save it
    let default_config = Self::default();
    default_config.save();
    default_config
  }

  pub fn save(&self) {
    let config_path = Self::config_path();

    if let Some(parent) = config_path.parent() {
      let _ = fs::create_dir_all(parent);
    }

    match serde_json::to_string_pretty(self) {
      Ok(content) => {
        if let Err(e) = fs::write(&config_path, content) {
          eprintln!("Failed to write config file: {}", e);
        }
      },
      Err(e) => {
        eprintln!("Failed to serialize config: {}", e);
      },
    }
  }

  fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("winkeylock");
    path.push("config.json");
    path
  }
}
