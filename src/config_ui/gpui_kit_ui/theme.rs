//! Theme registration and semantic colors for the configuration window.
use gpui_kit::App;
use gpui_kit::Hsla;
use gpui_kit::SharedString;
use gpui_kit::base::ScrollbarMode;
use gpui_kit::component::Theme;
use gpui_kit::component::ThemeRegistry;
use gpui_kit::rgb;

pub const DEFAULT_THEME: &str = "Default";
include!(concat!(env!("OUT_DIR"), "/bundled_themes.rs"));

pub fn init(cx: &mut App) {
  for source in BUNDLED_THEMES {
    ThemeRegistry::global_mut(cx)
      .load_themes_from_str(source)
      .expect("bundled gpui-kit theme must be valid");
  }
}

pub fn theme_names(cx: &App) -> Vec<SharedString> {
  let mut names: Vec<_> = ThemeRegistry::global(cx).themes().keys().cloned().collect();
  names.sort_by_key(|name| name.to_lowercase());
  names.insert(0, DEFAULT_THEME.into());
  names
}

/// Unknown names fall back to Default so hand-edited settings still open.
pub fn apply(name: &str, cx: &mut App) -> SharedString {
  let config = if name == DEFAULT_THEME {
    None
  } else {
    ThemeRegistry::global(cx).themes().get(name).cloned()
  };
  let selected = config
    .as_ref()
    .map(|config| config.name.clone())
    .unwrap_or_else(|| DEFAULT_THEME.into());
  let config = config.unwrap_or_else(|| ThemeRegistry::global(cx).default_light_theme().clone());
  Theme::update(cx, |theme| {
    // Reset optional font/radius fields as well when switching between themes.
    *theme = Theme::default();
    theme.apply_config(&config);
    theme.scrollbar_mode = ScrollbarMode::Always;
    if selected.as_ref() == DEFAULT_THEME {
      theme.colors.scrollbar_thumb = rgb(0x94a3b8).into();
      theme.colors.scrollbar_thumb_hover = rgb(0x64748b).into();
    }
  });
  cx.set_global(SelectedTheme(selected.clone()));
  cx.refresh_windows();
  selected
}
struct SelectedTheme(SharedString);
impl gpui_kit::Global for SelectedTheme {}

/// Original colors in Default; semantic GPUI Kit colors for named themes.
#[derive(Clone, Copy)]
pub enum Palette {
  Warning,
  WarningHover,
  Card,
  Help,
  CardBorder,
  ControlBorder,
  Secondary,
  SecondaryHover,
  Canvas,
  ErrorBackground,
  ErrorBorder,
  ErrorText,
  Success,
  SuccessHover,
  Primary,
  PrimaryHover,
  Record,
  RecordHover,
  Danger,
  DangerHover,
  MutedText,
  Text,
  SecondaryText,
  PrimaryText,
  SecondaryButtonText,
  SuccessText,
  DangerText,
  RecordText,
  WarningText,
}
impl Palette {
  pub fn color(self, cx: &App) -> Hsla {
    let is_default =
      !cx.has_global::<SelectedTheme>() || cx.global::<SelectedTheme>().0.as_ref() == DEFAULT_THEME;
    if is_default {
      return rgb(match self {
        Self::Warning => 0xd97706,
        Self::WarningHover => 0xb45309,
        Self::Card => 0xffffff,
        Self::Help => 0xf1f5f9,
        Self::CardBorder => 0xe2e8f0,
        Self::ControlBorder => 0xcbd5e1,
        Self::Secondary => 0xe2e8f0,
        Self::SecondaryHover => 0xcbd5e1,
        Self::Canvas => 0xf8fafc,
        Self::ErrorBackground => 0xfee2e2,
        Self::ErrorBorder => 0xfca5a5,
        Self::ErrorText => 0xb91c1c,
        Self::Success => 0x16a34a,
        Self::SuccessHover => 0x15803d,
        Self::Primary => 0x2563eb,
        Self::PrimaryHover => 0x1d4ed8,
        Self::Record => 0x475569,
        Self::RecordHover => 0x334155,
        Self::Danger => 0xef4444,
        Self::DangerHover => 0xdc2626,
        Self::MutedText => 0x64748b,
        Self::Text => 0x0f172a,
        Self::SecondaryText => 0x334155,
        Self::PrimaryText => 0xffffff,
        Self::SecondaryButtonText => 0x0f172a,
        Self::SuccessText => 0xffffff,
        Self::DangerText => 0xffffff,
        Self::RecordText => 0xffffff,
        Self::WarningText => 0xffffff,
      })
      .into();
    }
    let colors = &Theme::global(cx).colors;
    match self {
      Self::Warning => colors.warning,
      Self::WarningHover => colors.warning_hover,
      Self::Card => colors.group_box,
      Self::Help => colors.muted,
      Self::CardBorder => colors.border,
      Self::ControlBorder => colors.input,
      Self::Secondary => colors.button_secondary,
      Self::SecondaryHover => colors.button_secondary_hover,
      Self::Canvas => colors.background,
      Self::ErrorBackground => colors.danger.opacity(0.12),
      Self::ErrorBorder => colors.danger.opacity(0.4),
      Self::ErrorText => colors.danger,
      Self::Success => colors.button_success,
      Self::SuccessHover => colors.button_success_hover,
      Self::Primary => colors.button_primary,
      Self::PrimaryHover => colors.button_primary_hover,
      Self::Record => colors.secondary,
      Self::RecordHover => colors.secondary_hover,
      Self::Danger => colors.button_danger,
      Self::DangerHover => colors.button_danger_hover,
      Self::MutedText => colors.muted_foreground,
      Self::Text => colors.foreground,
      Self::SecondaryText => colors.foreground,
      Self::PrimaryText => colors.button_primary_foreground,
      Self::SecondaryButtonText => colors.button_secondary_foreground,
      Self::SuccessText => colors.button_success_foreground,
      Self::DangerText => colors.button_danger_foreground,
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
        assert_ne!(config.name.as_ref(), DEFAULT_THEME);
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
}
