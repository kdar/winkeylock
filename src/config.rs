use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use notify::Event;
use notify::EventKind;
use notify::RecursiveMode;
use notify::Watcher;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_0;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_1;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_2;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_3;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_4;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_5;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_6;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_7;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_8;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_9;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_A;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_B;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_BACK;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_C;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CAPITAL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_D;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_DELETE;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_DOWN;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_E;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_END;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F1;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F2;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F3;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F4;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F5;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F6;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F7;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F8;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F9;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F10;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F11;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F12;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F13;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F14;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F15;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F16;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F17;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F18;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F19;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F20;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F21;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F22;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F23;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F24;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_G;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_H;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_HOME;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_I;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_INSERT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_J;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_K;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_L;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LEFT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_M;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_N;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_NEXT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMLOCK;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_O;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_1;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_2;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_3;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_4;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_5;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_6;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_7;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_COMMA;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_MINUS;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_PERIOD;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_PLUS;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_P;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_PAUSE;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_PRIOR;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_Q;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_R;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RIGHT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_S;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SCROLL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SNAPSHOT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SPACE;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_T;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_TAB;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_U;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_UP;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_V;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_W;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_X;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_Y;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_Z;

#[derive(Debug, Clone)]
pub struct KeyCombo {
  pub key: Option<u16>,
  pub shift: bool,
  pub ctrl: bool,
  pub alt: bool,
  pub lwin: bool,
  pub rwin: bool,
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

  map
}

pub fn vk_to_key_name(vk: u16) -> Option<&'static str> {
  match windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk) {
    VK_A => Some("a"),
    VK_B => Some("b"),
    VK_C => Some("c"),
    VK_D => Some("d"),
    VK_E => Some("e"),
    VK_F => Some("f"),
    VK_G => Some("g"),
    VK_H => Some("h"),
    VK_I => Some("i"),
    VK_J => Some("j"),
    VK_K => Some("k"),
    VK_L => Some("l"),
    VK_M => Some("m"),
    VK_N => Some("n"),
    VK_O => Some("o"),
    VK_P => Some("p"),
    VK_Q => Some("q"),
    VK_R => Some("r"),
    VK_S => Some("s"),
    VK_T => Some("t"),
    VK_U => Some("u"),
    VK_V => Some("v"),
    VK_W => Some("w"),
    VK_X => Some("x"),
    VK_Y => Some("y"),
    VK_Z => Some("z"),
    VK_0 => Some("0"),
    VK_1 => Some("1"),
    VK_2 => Some("2"),
    VK_3 => Some("3"),
    VK_4 => Some("4"),
    VK_5 => Some("5"),
    VK_6 => Some("6"),
    VK_7 => Some("7"),
    VK_8 => Some("8"),
    VK_9 => Some("9"),
    VK_F1 => Some("f1"),
    VK_F2 => Some("f2"),
    VK_F3 => Some("f3"),
    VK_F4 => Some("f4"),
    VK_F5 => Some("f5"),
    VK_F6 => Some("f6"),
    VK_F7 => Some("f7"),
    VK_F8 => Some("f8"),
    VK_F9 => Some("f9"),
    VK_F10 => Some("f10"),
    VK_F11 => Some("f11"),
    VK_F12 => Some("f12"),
    VK_F13 => Some("f13"),
    VK_F14 => Some("f14"),
    VK_F15 => Some("f15"),
    VK_F16 => Some("f16"),
    VK_F17 => Some("f17"),
    VK_F18 => Some("f18"),
    VK_F19 => Some("f19"),
    VK_F20 => Some("f20"),
    VK_F21 => Some("f21"),
    VK_F22 => Some("f22"),
    VK_F23 => Some("f23"),
    VK_F24 => Some("f24"),
    VK_SPACE => Some("space"),
    VK_RETURN => Some("enter"),
    VK_TAB => Some("tab"),
    VK_ESCAPE => Some("esc"),
    VK_BACK => Some("backspace"),
    VK_DELETE => Some("delete"),
    VK_INSERT => Some("insert"),
    VK_HOME => Some("home"),
    VK_END => Some("end"),
    VK_PRIOR => Some("pageup"),
    VK_NEXT => Some("pagedown"),
    VK_UP => Some("up"),
    VK_DOWN => Some("down"),
    VK_LEFT => Some("left"),
    VK_RIGHT => Some("right"),
    VK_SNAPSHOT => Some("printscreen"),
    VK_PAUSE => Some("pause"),
    VK_CAPITAL => Some("capslock"),
    VK_NUMLOCK => Some("numlock"),
    VK_SCROLL => Some("scrolllock"),
    VK_OEM_1 => Some("semicolon"),
    VK_OEM_PLUS => Some("equals"),
    VK_OEM_COMMA => Some("comma"),
    VK_OEM_MINUS => Some("minus"),
    VK_OEM_PERIOD => Some("period"),
    VK_OEM_2 => Some("slash"),
    VK_OEM_3 => Some("grave"),
    VK_OEM_4 => Some("leftbracket"),
    VK_OEM_5 => Some("backslash"),
    VK_OEM_6 => Some("rightbracket"),
    VK_OEM_7 => Some("quote"),
    _ => None,
  }
}

pub fn format_key_combo(
  key: Option<u16>,
  shift: bool,
  ctrl: bool,
  alt: bool,
  lwin: bool,
  rwin: bool,
) -> Option<String> {
  let mut parts: Vec<&'static str> = Vec::new();
  if lwin {
    parts.push("lwin");
  }
  if rwin {
    parts.push("rwin");
  }
  if ctrl {
    parts.push("ctrl");
  }
  if alt {
    parts.push("alt");
  }
  if shift {
    parts.push("shift");
  }
  if let Some(vk) = key {
    let key_name = vk_to_key_name(vk)?;
    parts.push(key_name);
  }
  if parts.is_empty() {
    None
  } else {
    Some(parts.join("+"))
  }
}

impl KeyCombo {
  pub fn from_string(s: &str) -> Result<Self, String> {
    let key_map = create_key_map();
    let parts: Vec<String> = s
      .split('+')
      .map(|p| p.trim().to_lowercase())
      .filter(|p| !p.is_empty())
      .collect();

    if parts.is_empty() {
      return Err("Empty key combination".to_string());
    }

    let mut shift = false;
    let mut ctrl = false;
    let mut alt = false;
    let mut lwin = false;
    let mut rwin = false;
    let mut key_name = None;

    for part in &parts {
      match part.as_str() {
        "shift" | "lshift" | "rshift" => shift = true,
        "ctrl" | "control" | "lctrl" | "rctrl" => ctrl = true,
        "alt" | "lalt" | "ralt" => alt = true,
        "lwin" | "super" => lwin = true,
        "rwin" => rwin = true,
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

    let key = match key_name {
      Some(name) => {
        Some(
          *key_map
            .get(name)
            .ok_or_else(|| format!("Unknown key: {}", name))?,
        )
      },
      None => {
        if !(shift || ctrl || alt || lwin || rwin) {
          return Err("No key specified".to_string());
        }
        None
      },
    };

    Ok(Self {
      key,
      shift,
      ctrl,
      alt,
      lwin,
      rwin,
      string_repr: s.to_string(),
    })
  }

  pub fn matches(
    &self,
    key: Option<u16>,
    shift: bool,
    ctrl: bool,
    alt: bool,
    lwin: bool,
    rwin: bool,
  ) -> bool {
    match self.key {
      // Exact key + exact modifiers (e.g. "lwin+shift+s", "alt+f4", "capslock")
      Some(k) => {
        key == Some(k)
          && self.shift == shift
          && self.ctrl == ctrl
          && self.alt == alt
          && self.lwin == lwin
          && self.rwin == rwin
      },
      // Modifier-only rule (e.g. "lwin", "ctrl", "alt"): matches whenever those modifiers are active
      None => {
        (!self.shift || shift)
          && (!self.ctrl || ctrl)
          && (!self.alt || alt)
          && (!self.lwin || lwin)
          && (!self.rwin || rwin)
      },
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DetectMethod {
  #[default]
  NotificationState,
  Fullscreen,
  WindowStyle,
}

impl DetectMethod {
  pub const ALL: [DetectMethod; 3] = [
    DetectMethod::NotificationState,
    DetectMethod::Fullscreen,
    DetectMethod::WindowStyle,
  ];

  pub fn display_name(&self) -> &'static str {
    match self {
      DetectMethod::NotificationState => "Notification State",
      DetectMethod::Fullscreen => "Fullscreen",
      DetectMethod::WindowStyle => "Window Style",
    }
  }
}

impl fmt::Display for DetectMethod {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.display_name())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
  /// Key combinations that should be blocked
  pub blacklist: Vec<KeyCombo>,
  /// Key combinations that should be explicitly allowed (overrides blacklist)
  pub whitelist: Vec<KeyCombo>,
  /// Method used to detect if a window is a game
  #[serde(default)]
  pub detect_method: DetectMethod,
  /// Whether the app should run as administrator
  #[serde(default)]
  pub run_as_admin: bool,
}

impl Default for KeyConfig {
  fn default() -> Self {
    // Helper function to create KeyCombo from string, panicking on error (safe for defaults)
    let parse = |s: &str| KeyCombo::from_string(s).expect("Invalid default key combination");

    Self {
      // Default: block Windows key by itself
      blacklist: vec![parse("lwin")],
      whitelist: vec![],
      detect_method: DetectMethod::default(),
      run_as_admin: false,
    }
  }
}

impl KeyConfig {
  pub fn should_block(
    &self,
    key: Option<u16>,
    shift: bool,
    ctrl: bool,
    alt: bool,
    lwin: bool,
    rwin: bool,
  ) -> bool {
    if self
      .whitelist
      .iter()
      .any(|c| c.matches(key, shift, ctrl, alt, lwin, rwin))
    {
      return false;
    }

    self
      .blacklist
      .iter()
      .any(|c| c.matches(key, shift, ctrl, alt, lwin, rwin))
  }

  /// Returns true if any whitelisted combination uses all of the currently held modifiers
  pub fn has_potential_whitelist_match(
    &self,
    shift: bool,
    ctrl: bool,
    alt: bool,
    lwin: bool,
    rwin: bool,
  ) -> bool {
    self.whitelist.iter().any(|c| {
      (!shift || c.shift)
        && (!ctrl || c.ctrl)
        && (!alt || c.alt)
        && (!lwin || c.lwin)
        && (!rwin || c.rwin)
    })
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

/// Dynamic config manager that supports live reloading
#[derive(Debug)]
pub struct ConfigManager {
  config: Arc<RwLock<KeyConfig>>,
  // We don't store the watcher here to avoid Send/Sync issues
  // The watcher runs in its own thread
}

impl ConfigManager {
  pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    let config = Arc::new(RwLock::new(KeyConfig::load()));
    let config_clone = Arc::clone(&config);

    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
      match res {
        Ok(event) => {
          if let Err(e) = tx.send(event) {
            eprintln!("Failed to send file watch event: {}", e);
          }
        },
        Err(e) => eprintln!("File watch error: {}", e),
      }
    })?;

    let config_path = KeyConfig::config_path();
    if let Some(config_dir) = config_path.parent() {
      // Watch the directory containing the config file
      watcher.watch(config_dir, RecursiveMode::NonRecursive)?;
    }

    // Spawn a thread to handle file change events
    // We move the watcher into this thread to avoid Send/Sync issues
    thread::spawn(move || {
      let mut last_reload = std::time::Instant::now();

      // Keep the watcher alive in this thread
      let _watcher = watcher;

      for event in rx {
        match event.kind {
          EventKind::Create(_) | EventKind::Modify(_) => {
            // Check if the event is for our config file
            if event.paths.iter().any(|p| p.ends_with("config.json")) {
              // Debounce rapid file changes (editors often create multiple events)
              let now = std::time::Instant::now();
              if now.duration_since(last_reload) > Duration::from_millis(500) {
                last_reload = now;

                // Small delay to ensure file write is complete
                thread::sleep(Duration::from_millis(100));

                match KeyConfig::load() {
                  config => {
                    if let Ok(mut current_config) = config_clone.write() {
                      *current_config = config;
                      println!("Configuration reloaded successfully");
                    } else {
                      eprintln!("Failed to acquire write lock for config reload");
                    }
                  },
                }
              }
            }
          },
          _ => {},
        }
      }
    });

    Ok(Self {
      config,
    })
  }

  pub fn should_block(
    &self,
    key: Option<u16>,
    shift: bool,
    ctrl: bool,
    alt: bool,
    lwin: bool,
    rwin: bool,
  ) -> bool {
    if let Ok(config) = self.config.read() {
      config.should_block(key, shift, ctrl, alt, lwin, rwin)
    } else {
      eprintln!("Failed to acquire read lock for config");
      false // Default to not blocking if we can't read config
    }
  }

  pub fn has_potential_whitelist_match(
    &self,
    shift: bool,
    ctrl: bool,
    alt: bool,
    lwin: bool,
    rwin: bool,
  ) -> bool {
    if let Ok(config) = self.config.read() {
      config.has_potential_whitelist_match(shift, ctrl, alt, lwin, rwin)
    } else {
      eprintln!("Failed to acquire read lock for config");
      false
    }
  }

  pub fn detect_method(&self) -> DetectMethod {
    if let Ok(config) = self.config.read() {
      config.detect_method
    } else {
      eprintln!("Failed to acquire read lock for config");
      DetectMethod::default()
    }
  }

  pub fn get_config_path(&self) -> PathBuf {
    KeyConfig::config_path()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_lwin_blacklisted_no_whitelist_blocks_all_lwin_activations() {
    let config = KeyConfig {
      blacklist: vec![KeyCombo::from_string("lwin").unwrap()],
      whitelist: vec![],
      detect_method: DetectMethod::default(),
      run_as_admin: false,
    };

    assert!(!config.has_potential_whitelist_match(false, false, false, true, false));
    // Standalone lwin
    assert!(config.should_block(None, false, false, false, true, false));
    // lwin with other modifiers held
    assert!(config.should_block(None, true, false, false, true, false));
    // lwin+s combo
    assert!(config.should_block(Some(VK_S.0), false, false, false, true, false));
    // lwin+shift+s combo
    assert!(config.should_block(Some(VK_S.0), true, false, false, true, false));
    // Plain 's' without lwin is not blocked
    assert!(!config.should_block(Some(VK_S.0), false, false, false, false, false));
    // rwin is not blocked when only lwin is blacklisted
    assert!(!config.should_block(None, false, false, false, false, true));
    assert!(!config.should_block(Some(VK_S.0), false, false, false, false, true));
  }

  #[test]
  fn test_lwin_blacklisted_with_whitelist_allows_only_whitelisted_combo() {
    let config = KeyConfig {
      blacklist: vec![KeyCombo::from_string("lwin").unwrap()],
      whitelist: vec![KeyCombo::from_string("lwin+shift+s").unwrap()],
      detect_method: DetectMethod::default(),
      run_as_admin: false,
    };

    assert!(config.has_potential_whitelist_match(false, false, false, true, false));
    assert!(config.has_potential_whitelist_match(true, false, false, true, false));
    // Whitelisted combo is allowed
    assert!(!config.should_block(Some(VK_S.0), true, false, false, true, false));
    // Non-whitelisted lwin+s is still blocked
    assert!(config.should_block(Some(VK_S.0), false, false, false, true, false));
    // Standalone lwin is still blocked
    assert!(config.should_block(None, false, false, false, true, false));
  }

  #[test]
  fn test_ctrl_blacklisted_and_whitelisted() {
    let config = KeyConfig {
      blacklist: vec![KeyCombo::from_string("ctrl").unwrap()],
      whitelist: vec![KeyCombo::from_string("ctrl+c").unwrap()],
      detect_method: DetectMethod::default(),
      run_as_admin: false,
    };

    assert!(config.has_potential_whitelist_match(false, true, false, false, false));
    // Standalone ctrl is blocked
    assert!(config.should_block(None, false, true, false, false, false));
    // ctrl+s is blocked
    assert!(config.should_block(Some(VK_S.0), false, true, false, false, false));
    // ctrl+c is allowed
    assert!(!config.should_block(Some(VK_C.0), false, true, false, false, false));
  }
}
