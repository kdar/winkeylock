use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, fs, path::PathBuf};
use windows::Win32::UI::Input::KeyboardAndMouse::{
  VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9, VK_A, VK_B, VK_BACK, VK_C,
  VK_CAPITAL, VK_D, VK_DELETE, VK_DOWN, VK_E, VK_END, VK_ESCAPE, VK_F, VK_F1, VK_F2, VK_F3, VK_F4,
  VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16,
  VK_F17, VK_F18, VK_F19, VK_F20, VK_F21, VK_F22, VK_F23, VK_F24, VK_G, VK_H, VK_HOME, VK_I,
  VK_INSERT, VK_J, VK_K, VK_L, VK_LEFT, VK_LWIN, VK_M, VK_N, VK_NEXT, VK_NUMLOCK, VK_O, VK_OEM_1,
  VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS,
  VK_OEM_PERIOD, VK_OEM_PLUS, VK_P, VK_PAUSE, VK_PRIOR, VK_Q, VK_R, VK_RETURN, VK_RIGHT, VK_RWIN,
  VK_S, VK_SCROLL, VK_SNAPSHOT, VK_SPACE, VK_T, VK_TAB, VK_U, VK_UP, VK_V, VK_W, VK_X, VK_Y, VK_Z,
};

#[derive(Debug, Clone)]
pub struct KeyCombo {
  pub key: u16,
  pub shift: bool,
  pub ctrl: bool,
  pub alt: bool,
  pub win: bool,
  pub string_repr: String,
}

fn create_key_map() -> HashMap<String, u16> {
  let mut map = HashMap::new();

  // Letters
  map.insert("a".to_string(), VK_A.0);
  map.insert("b".to_string(), VK_B.0);
  map.insert("c".to_string(), VK_C.0);
  map.insert("d".to_string(), VK_D.0);
  map.insert("e".to_string(), VK_E.0);
  map.insert("f".to_string(), VK_F.0);
  map.insert("g".to_string(), VK_G.0);
  map.insert("h".to_string(), VK_H.0);
  map.insert("i".to_string(), VK_I.0);
  map.insert("j".to_string(), VK_J.0);
  map.insert("k".to_string(), VK_K.0);
  map.insert("l".to_string(), VK_L.0);
  map.insert("m".to_string(), VK_M.0);
  map.insert("n".to_string(), VK_N.0);
  map.insert("o".to_string(), VK_O.0);
  map.insert("p".to_string(), VK_P.0);
  map.insert("q".to_string(), VK_Q.0);
  map.insert("r".to_string(), VK_R.0);
  map.insert("s".to_string(), VK_S.0);
  map.insert("t".to_string(), VK_T.0);
  map.insert("u".to_string(), VK_U.0);
  map.insert("v".to_string(), VK_V.0);
  map.insert("w".to_string(), VK_W.0);
  map.insert("x".to_string(), VK_X.0);
  map.insert("y".to_string(), VK_Y.0);
  map.insert("z".to_string(), VK_Z.0);

  // Numbers
  map.insert("0".to_string(), VK_0.0);
  map.insert("1".to_string(), VK_1.0);
  map.insert("2".to_string(), VK_2.0);
  map.insert("3".to_string(), VK_3.0);
  map.insert("4".to_string(), VK_4.0);
  map.insert("5".to_string(), VK_5.0);
  map.insert("6".to_string(), VK_6.0);
  map.insert("7".to_string(), VK_7.0);
  map.insert("8".to_string(), VK_8.0);
  map.insert("9".to_string(), VK_9.0);

  // Function keys
  map.insert("f1".to_string(), VK_F1.0);
  map.insert("f2".to_string(), VK_F2.0);
  map.insert("f3".to_string(), VK_F3.0);
  map.insert("f4".to_string(), VK_F4.0);
  map.insert("f5".to_string(), VK_F5.0);
  map.insert("f6".to_string(), VK_F6.0);
  map.insert("f7".to_string(), VK_F7.0);
  map.insert("f8".to_string(), VK_F8.0);
  map.insert("f9".to_string(), VK_F9.0);
  map.insert("f10".to_string(), VK_F10.0);
  map.insert("f11".to_string(), VK_F11.0);
  map.insert("f12".to_string(), VK_F12.0);
  map.insert("f13".to_string(), VK_F13.0);
  map.insert("f14".to_string(), VK_F14.0);
  map.insert("f15".to_string(), VK_F15.0);
  map.insert("f16".to_string(), VK_F16.0);
  map.insert("f17".to_string(), VK_F17.0);
  map.insert("f18".to_string(), VK_F18.0);
  map.insert("f19".to_string(), VK_F19.0);
  map.insert("f20".to_string(), VK_F20.0);
  map.insert("f21".to_string(), VK_F21.0);
  map.insert("f22".to_string(), VK_F22.0);
  map.insert("f23".to_string(), VK_F23.0);
  map.insert("f24".to_string(), VK_F24.0);

  // Special keys
  map.insert("space".to_string(), VK_SPACE.0);
  map.insert("enter".to_string(), VK_RETURN.0);
  map.insert("return".to_string(), VK_RETURN.0);
  map.insert("tab".to_string(), VK_TAB.0);
  map.insert("escape".to_string(), VK_ESCAPE.0);
  map.insert("esc".to_string(), VK_ESCAPE.0);
  map.insert("backspace".to_string(), VK_BACK.0);
  map.insert("delete".to_string(), VK_DELETE.0);
  map.insert("del".to_string(), VK_DELETE.0);
  map.insert("insert".to_string(), VK_INSERT.0);
  map.insert("ins".to_string(), VK_INSERT.0);
  map.insert("home".to_string(), VK_HOME.0);
  map.insert("end".to_string(), VK_END.0);
  map.insert("pageup".to_string(), VK_PRIOR.0);
  map.insert("pagedown".to_string(), VK_NEXT.0);
  map.insert("up".to_string(), VK_UP.0);
  map.insert("down".to_string(), VK_DOWN.0);
  map.insert("left".to_string(), VK_LEFT.0);
  map.insert("right".to_string(), VK_RIGHT.0);
  map.insert("printscreen".to_string(), VK_SNAPSHOT.0);
  map.insert("prtsc".to_string(), VK_SNAPSHOT.0);
  map.insert("pause".to_string(), VK_PAUSE.0);
  map.insert("capslock".to_string(), VK_CAPITAL.0);
  map.insert("numlock".to_string(), VK_NUMLOCK.0);
  map.insert("scrolllock".to_string(), VK_SCROLL.0);

  // Punctuation
  map.insert("semicolon".to_string(), VK_OEM_1.0);
  map.insert("equals".to_string(), VK_OEM_PLUS.0);
  map.insert("comma".to_string(), VK_OEM_COMMA.0);
  map.insert("minus".to_string(), VK_OEM_MINUS.0);
  map.insert("period".to_string(), VK_OEM_PERIOD.0);
  map.insert("slash".to_string(), VK_OEM_2.0);
  map.insert("grave".to_string(), VK_OEM_3.0);
  map.insert("leftbracket".to_string(), VK_OEM_4.0);
  map.insert("backslash".to_string(), VK_OEM_5.0);
  map.insert("rightbracket".to_string(), VK_OEM_6.0);
  map.insert("quote".to_string(), VK_OEM_7.0);

  // Windows keys
  map.insert("super".to_string(), VK_LWIN.0);
  map.insert("lwin".to_string(), VK_LWIN.0);
  map.insert("rwin".to_string(), VK_RWIN.0);

  map
}

impl KeyCombo {
  pub fn from_string(s: &str) -> Result<Self, String> {
    let key_map = create_key_map();
    let parts: Vec<String> = s.split('+').map(|p| p.trim().to_lowercase()).collect();

    if parts.is_empty() {
      return Err("Empty key combination".to_string());
    }

    let mut shift = false;
    let mut ctrl = false;
    let mut alt = false;
    let mut win = false;
    let mut key_name = None;

    for part in &parts {
      match part.as_str() {
        "shift" => shift = true,
        "ctrl" | "control" => ctrl = true,
        "alt" => alt = true,
        "lwin" | "super" => {
          // Special case: if "lwin" is the only part, treat it as the key itself
          if parts.len() == 1 {
            key_name = Some("lwin");
          }
          win = true;
        },
        "rwin" => {
          // Special case: if "rwin" is the only part, treat it as the key itself
          if parts.len() == 1 {
            key_name = Some("rwin");
          }
          win = true;
        },
        _ => {
          if key_name.is_some() {
            return Err(format!(
              "Multiple keys specified: {} and {}",
              key_name.as_ref().unwrap(),
              part
            ));
          }
          key_name = Some(part.as_str());
        },
      }
    }

    let key_name = key_name.ok_or("No key specified".to_string())?;
    let key_code = key_map
      .get(key_name)
      .ok_or_else(|| format!("Unknown key: {}", key_name))?;

    Ok(Self {
      key: *key_code,
      shift,
      ctrl,
      alt,
      win,
      string_repr: s.to_string(),
    })
  }

  pub fn matches(&self, key: u16, shift: bool, ctrl: bool, alt: bool, win: bool) -> bool {
    self.key == key
      && self.shift == shift
      && self.ctrl == ctrl
      && self.alt == alt
      && self.win == win
  }
}

impl Serialize for KeyCombo {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.string_repr)
  }
}

impl<'de> Deserialize<'de> for KeyCombo {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    KeyCombo::from_string(&s).map_err(serde::de::Error::custom)
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
    // Helper function to create KeyCombo from string, panicking on error (safe for defaults)
    let parse = |s: &str| KeyCombo::from_string(s).expect("Invalid default key combination");

    Self {
      // Default: block Windows key by itself
      blacklist: vec![parse("lwin"), parse("rwin")],
      whitelist: vec![],
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
      //   println!(
      //     r#"
      //       combo: {}, {}, {}, {}, {}
      //       key  : {}, {}, {}, {}, {}
      //       "#,
      //     combo.key, combo.shift, combo.ctrl, combo.alt, combo.win, key, shift, ctrl, alt, win
      //   );
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
    path.push("lwinkeylock");
    path.push("config.json");
    path
  }
}
