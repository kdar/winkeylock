//! Reusable `RenderOnce` UI components for the GPUI Kit configuration window.

extern crate gpui_kit as gpui;

use gpui_kit::AnyView;
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
use gpui_kit::StatefulInteractiveElement;
use gpui_kit::Styled;
use gpui_kit::Window;
use gpui_kit::component::h_flex;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::v_flex;
use gpui_kit::div;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::px;

use super::theme::Palette;

/// Keep longer explanations readable in the narrow configuration window.
fn build_tooltip(text: SharedString, window: &mut Window, cx: &mut App) -> AnyView {
  Tooltip::element(move |_, _| div().w(px(280.0)).whitespace_normal().child(text.clone()))
    .build(window, cx)
}

/// Interactive pill button with hand cursor and soft hover feedback.
#[derive(IntoElement)]
pub struct PillButton {
  id: ElementId,
  label: SharedString,
  bg: Palette,
  hover_bg: Palette,
  text_color: Palette,
  font_weight: FontWeight,
  padding_x: gpui_kit::Pixels,
  padding_y: gpui_kit::Pixels,
  text_size_sm: bool,
  tooltip: Option<SharedString>,
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
      bg: Palette::Primary,
      hover_bg: Palette::PrimaryHover,
      text_color: Palette::PrimaryText,
      font_weight: FontWeight::SEMIBOLD,
      padding_x: px(16.0),
      padding_y: px(6.0),
      text_size_sm: true,
      tooltip: None,
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
      bg: Palette::Secondary,
      hover_bg: Palette::SecondaryHover,
      text_color: Palette::SecondaryButtonText,
      font_weight: FontWeight::NORMAL,
      padding_x: px(12.0),
      padding_y: px(6.0),
      text_size_sm: true,
      tooltip: None,
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
      bg: Palette::Success,
      hover_bg: Palette::SuccessHover,
      text_color: Palette::SuccessText,
      font_weight: FontWeight::NORMAL,
      padding_x: px(12.0),
      padding_y: px(6.0),
      text_size_sm: true,
      tooltip: None,
      on_click: Box::new(on_click),
    }
  }

  pub fn record(
    id: impl Into<ElementId>,
    recording: bool,
    on_click: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
  ) -> Self {
    let (bg, hover_bg) = if recording {
      (Palette::Warning, Palette::WarningHover)
    } else {
      (Palette::Record, Palette::RecordHover)
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
      text_color: if recording {
        Palette::WarningText
      } else {
        Palette::RecordText
      },
      font_weight: FontWeight::NORMAL,
      padding_x: px(12.0),
      padding_y: px(6.0),
      text_size_sm: true,
      tooltip: Some(if recording {
        "Stop recording a key combination.".into()
      } else {
        "Press a key combination to fill the input, then click Add to include it in this list."
          .into()
      }),
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
      bg: Palette::ErrorBackground,
      hover_bg: Palette::ErrorBorder,
      text_color: Palette::ErrorText,
      font_weight: FontWeight::NORMAL,
      padding_x: px(8.0),
      padding_y: px(2.0),
      text_size_sm: false,
      tooltip: None,
      on_click: Box::new(on_click),
    }
  }

  pub fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
    self.tooltip = Some(text.into());
    self
  }
}

impl RenderOnce for PillButton {
  fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
    let bg = self.bg.color(cx);
    let hover_bg = self.hover_bg.color(cx);
    let text_color = self.text_color.color(cx);
    div()
      .id(self.id)
      .flex_shrink_0()
      .px(self.padding_x)
      .py(self.padding_y)
      .rounded_md()
      .text_size(px(13.0))
      .bg(bg)
      .font_weight(self.font_weight)
      .text_color(text_color)
      .cursor_pointer()
      .hover(move |s| s.bg(hover_bg))
      .when(self.text_size_sm, |s| s.text_size(px(13.0)))
      .when(!self.text_size_sm, |s| s.text_xs())
      .when_some(self.tooltip, |s, text| {
        s.tooltip(move |window, cx| build_tooltip(text.clone(), window, cx))
      })
      .on_mouse_up(MouseButton::Left, self.on_click)
      .child(self.label)
  }
}

/// Soft key-combination row with an inline remove action.
#[derive(IntoElement)]
pub struct KeyRow {
  pub label: SharedString,
  pub remove_id: ElementId,
  pub on_remove: Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App)>,
}

impl RenderOnce for KeyRow {
  fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
    h_flex()
      .items_center()
      .justify_between()
      .px_3()
      .py_1p5()
      .rounded_md()
      .min_w(px(0.0))
      .gap_3()
      .bg(Palette::Help.color(cx))
      .child(
        div()
          .min_w(px(0.0))
          .flex_1()
          .whitespace_normal()
          .text_sm()
          .font_weight(FontWeight::MEDIUM)
          .text_color(Palette::Text.color(cx))
          .child(self.label),
      )
      .child(
        PillButton::danger_badge(self.remove_id, "Remove", self.on_remove)
          .tooltip("Remove this key combination from the list. Click Save to apply the change."),
      )
  }
}

/// Interactive selection card for detection methods.
#[derive(IntoElement)]
pub struct DetectMethodCard {
  pub id: ElementId,
  pub name: &'static str,
  pub description: &'static str,
  pub selected: bool,
  pub on_select: Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App)>,
}

impl RenderOnce for DetectMethodCard {
  fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
    let selected = self.selected;
    let hover = Palette::SecondaryHover.color(cx);
    h_flex()
      .id(self.id)
      .w_full()
      .min_w(px(0.0))
      .gap_4()
      .p_4()
      .rounded_lg()
      .bg(if selected {
        Palette::Selection.color(cx)
      } else {
        Palette::Card.color(cx)
      })
      .text_color(Palette::Text.color(cx))
      .cursor_pointer()
      .hover(move |s| s.bg(hover))
      .on_mouse_up(MouseButton::Left, self.on_select)
      .child(
        v_flex()
          .flex_1()
          .min_w(px(0.0))
          .gap_1()
          .child(
            div()
              .text_sm()
              .font_weight(FontWeight::MEDIUM)
              .child(self.name),
          )
          .child(
            div()
              .text_xs()
              .whitespace_normal()
              .text_color(Palette::MutedText.color(cx))
              .child(self.description),
          ),
      )
      .child(
        div()
          .flex_shrink_0()
          .text_color(Palette::Primary.color(cx))
          .child(if selected { "●" } else { "○" }),
      )
  }
}

/// Collapsible instructions card.
#[derive(IntoElement)]
pub struct HelpCard;

impl RenderOnce for HelpCard {
  fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
    v_flex()
      .w_full()
      .whitespace_normal()
      .gap_1()
      .p_3()
      .rounded_md()
      .bg(Palette::Help.color(cx))
      .text_xs()
      .text_color(Palette::SecondaryText.color(cx))
      .child(
        div()
          .text_sm()
          .font_weight(FontWeight::SEMIBOLD)
          .text_color(Palette::Text.color(cx))
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
