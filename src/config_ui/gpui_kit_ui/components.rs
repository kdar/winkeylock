//! Reusable `RenderOnce` UI components for the GPUI Kit configuration window.

extern crate gpui_kit as gpui;

use gpui_kit::App;
use gpui_kit::ElementId;
use gpui_kit::FontWeight;
use gpui_kit::InteractiveElement;
use gpui_kit::IntoElement;
use gpui_kit::MouseButton;
use gpui_kit::MouseUpEvent;
use gpui_kit::ParentElement;
use gpui_kit::RenderOnce;
use gpui_kit::SharedString;
use gpui_kit::Styled;
use gpui_kit::Window;
use gpui_kit::component::h_flex;
use gpui_kit::component::v_flex;
use gpui_kit::div;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::px;
use gpui_kit::rgb;

use super::theme::Palette;

/// Interactive pill button with hand cursor and soft hover feedback.
#[derive(IntoElement)]
pub struct PillButton {
  id: ElementId,
  label: SharedString,
  bg: u32,
  hover_bg: u32,
  text_color: u32,
  font_weight: FontWeight,
  padding_x: gpui_kit::Pixels,
  padding_y: gpui_kit::Pixels,
  text_size_sm: bool,
  on_click: Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App)>,
}

impl PillButton {
  pub fn primary(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
  ) -> Self {
    Self {
      id: id.into(),
      label: label.into(),
      bg: Palette::PRIMARY,
      hover_bg: Palette::PRIMARY_HOVER,
      text_color: Palette::WHITE,
      font_weight: FontWeight::SEMIBOLD,
      padding_x: px(16.0),
      padding_y: px(6.0),
      text_size_sm: true,
      on_click: Box::new(on_click),
    }
  }

  pub fn secondary(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
  ) -> Self {
    Self {
      id: id.into(),
      label: label.into(),
      bg: Palette::BTN_SECONDARY,
      hover_bg: Palette::BTN_SECONDARY_HOVER,
      text_color: Palette::TEXT_PRIMARY,
      font_weight: FontWeight::NORMAL,
      padding_x: px(12.0),
      padding_y: px(6.0),
      text_size_sm: true,
      on_click: Box::new(on_click),
    }
  }

  pub fn success(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
  ) -> Self {
    Self {
      id: id.into(),
      label: label.into(),
      bg: Palette::GREEN,
      hover_bg: Palette::GREEN_HOVER,
      text_color: Palette::WHITE,
      font_weight: FontWeight::NORMAL,
      padding_x: px(12.0),
      padding_y: px(6.0),
      text_size_sm: true,
      on_click: Box::new(on_click),
    }
  }

  pub fn record(
    id: impl Into<ElementId>,
    recording: bool,
    on_click: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
  ) -> Self {
    let (bg, hover_bg) = if recording {
      (Palette::AMBER, Palette::AMBER_HOVER)
    } else {
      (Palette::RECORD_IDLE, Palette::RECORD_IDLE_HOVER)
    };
    Self {
      id: id.into(),
      label: if recording {
        "Stop".into()
      } else {
        "Record".into()
      },
      bg,
      hover_bg,
      text_color: Palette::WHITE,
      font_weight: FontWeight::NORMAL,
      padding_x: px(12.0),
      padding_y: px(6.0),
      text_size_sm: true,
      on_click: Box::new(on_click),
    }
  }

  pub fn danger_badge(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
  ) -> Self {
    Self {
      id: id.into(),
      label: label.into(),
      bg: Palette::RED,
      hover_bg: Palette::RED_HOVER,
      text_color: Palette::WHITE,
      font_weight: FontWeight::NORMAL,
      padding_x: px(8.0),
      padding_y: px(2.0),
      text_size_sm: false,
      on_click: Box::new(on_click),
    }
  }
}

impl RenderOnce for PillButton {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    div()
      .id(self.id)
      .flex_shrink_0()
      .px(self.padding_x)
      .py(self.padding_y)
      .rounded_md()
      .bg(rgb(self.bg))
      .font_weight(self.font_weight)
      .text_color(rgb(self.text_color))
      .cursor_pointer()
      .hover(move |s| s.bg(rgb(self.hover_bg)))
      .when(self.text_size_sm, |s| s.text_sm())
      .when(!self.text_size_sm, |s| s.text_xs())
      .on_mouse_up(MouseButton::Left, self.on_click)
      .child(self.label)
  }
}

/// Elevated card representing a single key combo row with a delete badge.
#[derive(IntoElement)]
pub struct KeyRow {
  pub label: SharedString,
  pub remove_id: ElementId,
  pub on_remove: Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App)>,
}

impl RenderOnce for KeyRow {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    h_flex()
      .items_center()
      .justify_between()
      .px_3()
      .py_1p5()
      .rounded_md()
      .bg(rgb(Palette::BG_CARD))
      .border_1()
      .border_color(rgb(Palette::BORDER_CARD))
      .child(
        div()
          .text_sm()
          .font_weight(FontWeight::MEDIUM)
          .text_color(rgb(Palette::TEXT_PRIMARY))
          .child(self.label),
      )
      .child(PillButton::danger_badge(
        self.remove_id,
        "Remove",
        self.on_remove,
      ))
  }
}

/// Interactive selection card for detection methods.
#[derive(IntoElement)]
pub struct DetectMethodCard {
  pub id: ElementId,
  pub name: &'static str,
  pub selected: bool,
  pub on_select: Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App)>,
}

impl RenderOnce for DetectMethodCard {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    div()
      .id(self.id)
      .flex_1()
      .px_3()
      .py_1p5()
      .rounded_md()
      .border_1()
      .border_color(if self.selected {
        rgb(Palette::PRIMARY)
      } else {
        rgb(Palette::BORDER_CONTROL)
      })
      .bg(if self.selected {
        rgb(Palette::PRIMARY)
      } else {
        rgb(Palette::BG_CARD)
      })
      .text_xs()
      .text_center()
      .text_color(if self.selected {
        rgb(Palette::WHITE)
      } else {
        rgb(Palette::TEXT_PRIMARY)
      })
      .cursor_pointer()
      .hover(move |s| {
        if !self.selected {
          s.bg(rgb(Palette::BG_HELP))
        } else {
          s.bg(rgb(Palette::PRIMARY_HOVER))
        }
      })
      .on_mouse_up(MouseButton::Left, self.on_select)
      .child(self.name)
  }
}

/// Collapsible instructions card.
#[derive(IntoElement)]
pub struct HelpCard;

impl RenderOnce for HelpCard {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    v_flex()
      .w_full()
      .whitespace_normal()
      .gap_1()
      .p_3()
      .rounded_md()
      .bg(rgb(Palette::BG_HELP))
      .border_1()
      .border_color(rgb(Palette::BORDER_CONTROL))
      .text_xs()
      .text_color(rgb(Palette::TEXT_SECONDARY))
      .child(
        div()
          .text_sm()
          .font_weight(FontWeight::SEMIBOLD)
          .text_color(rgb(Palette::TEXT_PRIMARY))
          .child("How to enter key combinations"),
      )
      .child("• Use '+' between each modifier and key, e.g. 'ctrl+shift+esc'.")
      .child("• Supported modifiers: 'ctrl', 'alt', 'shift', 'lwin', 'rwin'.")
      .child("• Finish with a key name like 'c', 'f12', 'space', 'delete', or just a modifier.")
      .child("• Or click Record and press the combination on your keyboard.")
      .child("• Names are case-insensitive; spaces around '+' are optional.")
      .child("• At most one non-modifier key is allowed per combination.")
  }
}
