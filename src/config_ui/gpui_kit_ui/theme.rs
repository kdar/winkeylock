//! WinKeyLock colors and the same window materials used by Relay2FA.
use gpui_kit::App;
use gpui_kit::Global;
use gpui_kit::Hsla;
use gpui_kit::SharedString;
use gpui_kit::WindowBackgroundAppearance;
use gpui_kit::base::ScrollbarMode;
use gpui_kit::component::Theme;
use gpui_kit::component::ThemeColor;
use gpui_kit::component::ThemeRegistry;
pub const DEFAULT_THEME: &str = "WinKeyLock";
include!(concat!(env!("OUT_DIR"), "/bundled_themes.rs"));
#[derive(Clone, Copy)]
pub(super) struct WindowSurface {
  pub enabled: bool,
  pub supported: bool,
}
impl Global for WindowSurface {}

/// Always derive materials from the selected theme, never a previous preview.
struct SolidPalette(ThemeColor);
impl Global for SolidPalette {}

fn material_colors(mut colors: ThemeColor, glass: bool) -> ThemeColor {
  if !glass {
    return colors;
  }
  let mut wash = colors.foreground;
  wash.s = 0.08;
  wash.l = if colors.background.l < 0.5 {
    0.94
  } else {
    0.06
  };
  let surface = |alpha| {
    Hsla {
      a: alpha,
      ..wash
    }
  };
  colors.background = surface(0.06);
  colors.group_box = surface(0.045);
  colors.muted = surface(0.05);
  colors.sidebar = surface(0.05);
  colors.title_bar = surface(0.025);
  colors.button = surface(0.08);
  colors.button_hover = surface(0.12);
  colors.button_active = surface(0.16);
  colors.button_secondary = surface(0.08);
  colors.button_secondary_hover = surface(0.12);
  colors.button_secondary_active = surface(0.16);
  colors.secondary = surface(0.08);
  colors.secondary_hover = surface(0.12);
  colors.secondary_active = surface(0.16);
  colors.switch = surface(0.18);
  colors.list = surface(0.025);
  colors.list_hover = surface(0.08);
  colors.list_active = surface(0.13);
  colors.list_head = surface(0.05);
  colors.border = surface(0.10);
  colors.input = surface(0.14);
  colors.sidebar_border = colors.border;
  colors.title_bar_border = colors.border;
  // Floating menus overlap other text; retain enough tint to separate it.
  colors.popover.a = 0.96;
  // Accent fills remain strong enough for their foreground text.
  colors.button_primary.a = 0.88;
  colors.button_primary_hover.a = 0.94;
  colors.button_primary_active.a = 0.98;
  colors
}

fn apply_material(cx: &mut App) {
  if let Some(solid) = cx.try_global::<SolidPalette>() {
    let colors = material_colors(solid.0, glass_active(cx));
    Theme::update(cx, |theme| theme.colors = colors);
  }
}

pub fn init_glass(enabled: bool, cx: &mut App) {
  cx.set_global(WindowSurface {
    enabled,
    supported: glass_available(),
  });
}

pub fn glass_active(cx: &App) -> bool {
  cx.try_global::<WindowSurface>()
    .is_some_and(|surface| surface.enabled && surface.supported)
}

pub fn glass_supported(cx: &App) -> bool {
  cx.try_global::<WindowSurface>()
    .is_some_and(|surface| surface.supported)
}

pub fn background_appearance(cx: &App) -> WindowBackgroundAppearance {
  if glass_active(cx) {
    WindowBackgroundAppearance::Blurred
  } else {
    WindowBackgroundAppearance::Opaque
  }
}

pub fn set_glass(enabled: bool, cx: &mut App) {
  let mut surface = *cx.global::<WindowSurface>();
  if surface.enabled == enabled {
    return;
  }
  surface.enabled = enabled;
  cx.set_global(surface);
  apply_material(cx);
  let appearance = background_appearance(cx);
  // Do not re-enter a native window while a switch callback is using it.
  cx.defer(move |cx| {
    for handle in cx.windows() {
      let _ = handle.update(cx, |_, window, _| {
        window.set_background_appearance(appearance)
      });
    }
    cx.refresh_windows();
  });
  cx.refresh_windows();
}

/// Paint exactly once per window, above the native Windows blur.
pub fn window_surface(cx: &App) -> Hsla {
  let mut tint = cx.global::<SolidPalette>().0.background;
  tint.a = if glass_active(cx) { 0.78 } else { 1.0 };
  tint
}

pub fn init(cx: &mut App) {
  for source in BUNDLED_THEMES {
    ThemeRegistry::global_mut(cx)
      .load_themes_from_str(source)
      .expect("bundled GPUI Kit theme must be valid");
  }
}

pub fn theme_names(cx: &App) -> Vec<SharedString> {
  let mut names: Vec<_> = ThemeRegistry::global(cx).themes().keys().cloned().collect();
  names.sort_by_key(|name| name.to_lowercase());
  names.retain(|name| name.as_ref() != DEFAULT_THEME);
  names.insert(0, DEFAULT_THEME.into());
  names
}

pub fn apply(name: &str, cx: &mut App) -> SharedString {
  // Preserve the previous built-in label when loading older saved preferences.
  let name = if name == "Default" {
    DEFAULT_THEME
  } else {
    name
  };
  let config = ThemeRegistry::global(cx).themes().get(name).cloned();
  let selected = config
    .as_ref()
    .map(|config| config.name.clone())
    .unwrap_or_else(|| DEFAULT_THEME.into());
  let config = config
    .or_else(|| {
      ThemeRegistry::global(cx)
        .themes()
        .get(DEFAULT_THEME)
        .cloned()
    })
    .unwrap_or_else(|| ThemeRegistry::global(cx).default_dark_theme().clone());
  Theme::update(cx, |theme| {
    *theme = Theme::default();
    theme.apply_config(&config);
    theme.scrollbar_mode = ScrollbarMode::Always;
  });
  cx.set_global(SolidPalette(Theme::global(cx).colors));
  apply_material(cx);
  cx.refresh_windows();
  selected
}

fn glass_available() -> bool {
  use windows::Win32::Foundation::ERROR_SUCCESS;
  use windows::Win32::System::Registry::*;
  use windows::core::w;
  let mut build = [0u16; 32];
  let mut bytes = std::mem::size_of_val(&build) as u32;
  // SAFETY: Registry queries use static names and correctly sized output buffers.
  let status = unsafe {
    RegGetValueW(
      HKEY_LOCAL_MACHINE,
      w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
      w!("CurrentBuildNumber"),
      RRF_RT_REG_SZ,
      None,
      Some(build.as_mut_ptr().cast()),
      Some(&mut bytes),
    )
  };
  if status != ERROR_SUCCESS {
    return false;
  }
  let end = build.iter().position(|c| *c == 0).unwrap_or(build.len());
  if String::from_utf16_lossy(&build[..end])
    .parse::<u32>()
    .unwrap_or(0)
    < 17763
  {
    return false;
  }
  let mut enabled = 1u32;
  let mut bytes = std::mem::size_of_val(&enabled) as u32;
  // SAFETY: Read-only DWORD query; missing values use Windows' enabled default.
  let status = unsafe {
    RegGetValueW(
      HKEY_CURRENT_USER,
      w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
      w!("EnableTransparency"),
      RRF_RT_REG_DWORD,
      None,
      Some((&mut enabled as *mut u32).cast()),
      Some(&mut bytes),
    )
  };
  status != ERROR_SUCCESS || enabled != 0
}
#[derive(Clone, Copy)]
pub enum Palette {
  Panel,
  TitleBar,
  Selection,
  Warning,
  WarningHover,
  Card,
  Help,
  Secondary,
  SecondaryHover,
  ErrorBackground,
  ErrorBorder,
  ErrorText,
  Success,
  SuccessHover,
  Primary,
  PrimaryHover,
  Record,
  RecordHover,
  MutedText,
  Text,
  SecondaryText,
  PrimaryText,
  SecondaryButtonText,
  SuccessText,
  RecordText,
  WarningText,
}
impl Palette {
  pub fn color(self, cx: &App) -> Hsla {
    let colors = &Theme::global(cx).colors;
    match self {
      Self::Panel => colors.muted,
      Self::TitleBar => colors.title_bar,
      Self::Selection => colors.button_secondary_hover,
      Self::Warning => colors.warning,
      Self::WarningHover => colors.warning_hover,
      Self::Card => colors.group_box,
      Self::Help => colors.muted,
      Self::Secondary => colors.button_secondary,
      Self::SecondaryHover => colors.button_secondary_hover,
      Self::ErrorBackground => colors.danger.opacity(0.12),
      Self::ErrorBorder => colors.danger.opacity(0.4),
      Self::ErrorText => colors.danger,
      Self::Success => colors.button_success,
      Self::SuccessHover => colors.button_success_hover,
      Self::Primary => colors.button_primary,
      Self::PrimaryHover => colors.button_primary_hover,
      Self::Record => colors.secondary,
      Self::RecordHover => colors.secondary_hover,
      Self::MutedText => colors.muted_foreground,
      Self::Text => colors.foreground,
      Self::SecondaryText => colors.foreground,
      Self::PrimaryText => colors.button_primary_foreground,
      Self::SecondaryButtonText => colors.button_secondary_foreground,
      Self::SuccessText => colors.button_success_foreground,
      Self::RecordText => colors.secondary_foreground,
      Self::WarningText => colors.warning_foreground,
    }
  }
}
#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use gpui_kit::component::ThemeSet;

  use super::*;
  #[test]
  fn bundled_themes_have_unique_names_and_light_and_dark_choices() {
    let mut names = HashSet::new();
    let mut light = false;
    let mut dark = false;
    for source in BUNDLED_THEMES {
      let set: ThemeSet = serde_json::from_str(source).unwrap();
      assert!(!set.themes.is_empty());
      for config in set.themes {
        assert!(
          names.insert(config.name.to_string()),
          "duplicate theme name"
        );
        dark |= config.mode.is_dark();
        light |= !config.mode.is_dark();
      }
    }
    assert!(light && dark);
  }
  #[test]
  fn glass_covers_surfaces_without_fading_content() {
    for solid in [*ThemeColor::dark(), *ThemeColor::light()] {
      let glass = material_colors(solid, true);
      for surface in [
        glass.background,
        glass.group_box,
        glass.muted,
        glass.title_bar,
        glass.button,
        glass.button_secondary,
        glass.switch,
      ] {
        assert!(surface.a > 0.0 && surface.a < 1.0);
      }
      assert_eq!(glass.foreground, solid.foreground);
      assert_eq!(glass.muted_foreground, solid.muted_foreground);
      assert_eq!(
        glass.button_primary_foreground,
        solid.button_primary_foreground
      );
      assert_eq!(glass.switch_thumb, solid.switch_thumb);
      assert!(glass.button_hover.a > glass.button.a);
      assert!(glass.button_active.a > glass.button_hover.a);
      assert_eq!(material_colors(solid, false), solid);
    }
  }
}
