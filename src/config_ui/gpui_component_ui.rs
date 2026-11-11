//! Config UI built on `gpui-component` (shadcn-style widgets on top of GPUI).
//!
//! Same behavior as `gpui_ui.rs`, but uses ready-made `Input`, `Button`, and
//! `ButtonGroup` widgets instead of hand-rolled text fields and styled divs.

use std::time::Duration;

use gpui::{
  App, AppContext, Application, Bounds, Context, Entity, IntoElement, ParentElement, Render,
  SharedString, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_component::{
  ActiveTheme, Root, Selectable, Sizable, StyledExt,
  button::{Button, ButtonGroup, ButtonVariants},
  h_flex,
  input::{Input, InputEvent, InputState},
  scroll::ScrollableElement,
  v_flex,
};

use crate::{
  config::{DetectMethod, KeyCombo, KeyConfig},
  config_ui::{
    CONFIG_WINDOW_TITLE_GPUI, is_recording_mode, set_recording_mode, take_recorded_combo,
  },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListKind {
  Blacklist,
  Whitelist,
}

pub struct GpuiComponentConfigUI {
  config: KeyConfig,
  blacklist_input: Entity<InputState>,
  whitelist_input: Entity<InputState>,
  /// Which list the Record button is currently capturing into.
  recording_target: ListKind,
  error_message: Option<SharedString>,
  show_help: bool,
}

impl GpuiComponentConfigUI {
  fn new(config: KeyConfig, window: &mut Window, cx: &mut Context<Self>) -> Self {
    let blacklist_input = cx.new(|cx| {
      InputState::new(window, cx).placeholder("Type a combo (e.g. 'lwin', 'lwin+s') or click Record")
    });
    let whitelist_input = cx.new(|cx| {
      InputState::new(window, cx).placeholder("Type a combo (e.g. 'lwin+shift+s') or click Record")
    });

    // Enter in an input adds its combo to the matching list.
    for (input, kind) in [
      (&blacklist_input, ListKind::Blacklist),
      (&whitelist_input, ListKind::Whitelist),
    ] {
      cx.subscribe_in(input, window, move |this, _, event: &InputEvent, window, cx| {
        if let InputEvent::PressEnter { .. } = event {
          this.add_combo(kind, window, cx);
        }
      })
      .detach();
    }

    // Poll the low-level keyboard hook for recorded combos.
    cx.spawn_in(window, async move |this, cx| {
      loop {
        cx.background_executor()
          .timer(Duration::from_millis(25))
          .await;
        let alive = this
          .update_in(cx, |view, window, cx| view.poll_recorded_combo(window, cx))
          .is_ok();
        if !alive {
          break;
        }
      }
    })
    .detach();

    Self {
      config,
      blacklist_input,
      whitelist_input,
      recording_target: ListKind::Blacklist,
      error_message: None,
      show_help: false,
    }
  }

  fn input_for(&self, kind: ListKind) -> &Entity<InputState> {
    match kind {
      ListKind::Blacklist => &self.blacklist_input,
      ListKind::Whitelist => &self.whitelist_input,
    }
  }

  fn list_mut(&mut self, kind: ListKind) -> &mut Vec<KeyCombo> {
    match kind {
      ListKind::Blacklist => &mut self.config.blacklist,
      ListKind::Whitelist => &mut self.config.whitelist,
    }
  }

  fn poll_recorded_combo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
    if let Some(combo) = take_recorded_combo() {
      let input = self.input_for(self.recording_target).clone();
      input.update(cx, |state, cx| state.set_value(combo, window, cx));
      self.error_message = None;
      cx.notify();
    } else if is_recording_mode() {
      // Keep the Record/Stop button label in sync while recording.
      cx.notify();
    }
  }

  fn add_combo(&mut self, kind: ListKind, window: &mut Window, cx: &mut Context<Self>) {
    set_recording_mode(false);
    let input = self.input_for(kind).clone();
    let text = input.read(cx).value().trim().to_string();
    if text.is_empty() {
      return;
    }
    match KeyCombo::from_string(&text) {
      Ok(combo) => {
        self.list_mut(kind).push(combo);
        input.update(cx, |state, cx| state.set_value("", window, cx));
        self.error_message = None;
      },
      Err(e) => self.error_message = Some(format!("Invalid key combination: {e}").into()),
    }
    cx.notify();
  }

  fn toggle_record(&mut self, kind: ListKind, cx: &mut Context<Self>) {
    let was_recording_this = is_recording_mode() && self.recording_target == kind;
    self.recording_target = kind;
    set_recording_mode(!was_recording_this);
    cx.notify();
  }

  fn render_section(
    &self,
    kind: ListKind,
    title: &'static str,
    description: &'static str,
    empty_text: &'static str,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let items = match kind {
      ListKind::Blacklist => &self.config.blacklist,
      ListKind::Whitelist => &self.config.whitelist,
    };
    let prefix = match kind {
      ListKind::Blacklist => "black",
      ListKind::Whitelist => "white",
    };
    let recording = is_recording_mode() && self.recording_target == kind;
    let theme = cx.theme();

    let rows = if items.is_empty() {
      v_flex().child(
        div()
          .text_sm()
          .text_color(theme.muted_foreground)
          .child(empty_text),
      )
    } else {
      v_flex()
        .gap_1()
        .children(items.iter().enumerate().map(|(i, combo)| {
          h_flex()
            .justify_between()
            .px_3()
            .py_1()
            .rounded(theme.radius)
            .border_1()
            .border_color(theme.border)
            .child(div().text_sm().child(combo.string_repr.clone()))
            .child(
              Button::new((SharedString::from(format!("{prefix}-remove")), i))
                .label("Remove")
                .danger()
                .xsmall()
                .on_click(cx.listener(move |this, _, _, cx| {
                  let list = this.list_mut(kind);
                  if i < list.len() {
                    list.remove(i);
                    cx.notify();
                  }
                })),
            )
        }))
    };

    v_flex()
      .gap_2()
      .child(div().text_base().font_semibold().child(title))
      .child(
        div()
          .text_xs()
          .text_color(theme.muted_foreground)
          .child(description),
      )
      .child(rows)
      .child(
        h_flex()
          .gap_2()
          .child(div().flex_1().min_w(px(0.)).child(Input::new(self.input_for(kind))))
          .child(
            Button::new(SharedString::from(format!("{prefix}-record")))
              .label(if recording { "Stop" } else { "Record" })
              .when_recording(recording)
              .on_click(cx.listener(move |this, _, _, cx| this.toggle_record(kind, cx))),
          )
          .child(
            Button::new(SharedString::from(format!("{prefix}-add")))
              .label("Add")
              .success()
              .on_click(cx.listener(move |this, _, window, cx| this.add_combo(kind, window, cx))),
          ),
      )
  }
}

/// Small helper so the Record button turns into a warning-colored "Stop" while active.
trait RecordStyle {
  fn when_recording(self, recording: bool) -> Self;
}

impl RecordStyle for Button {
  fn when_recording(self, recording: bool) -> Self {
    if recording { self.warning() } else { self.outline() }
  }
}

impl Render for GpuiComponentConfigUI {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let current_method = self.config.detect_method;
    let muted = cx.theme().muted_foreground;

    let header = h_flex()
      .gap_3()
      .justify_between()
      .child(
        div()
          .flex_1()
          .min_w(px(0.))
          .text_sm()
          .text_color(muted)
          .child("Configure which keys should be blocked or allowed. Examples: 'lwin', 'ctrl+c', 'shift+alt+tab'"),
      )
      .child(
        Button::new("toggle-help")
          .label(if self.show_help { "Hide Help" } else { "Show Help" })
          .ghost()
          .small()
          .on_click(cx.listener(|this, _, _, cx| {
            this.show_help = !this.show_help;
            cx.notify();
          })),
      );

    let help = self.show_help.then(|| {
      v_flex()
        .gap_1()
        .p_3()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().muted)
        .text_xs()
        .child(div().text_sm().font_semibold().child("How to enter key combinations"))
        .child("• Use '+' between each modifier and key, e.g. 'ctrl+shift+esc'.")
        .child("• Supported modifiers: 'ctrl', 'alt', 'shift', 'lwin', 'rwin'.")
        .child("• Finish with a key name like 'c', 'f12', 'space', 'delete', or just a modifier.")
        .child("• Or click Record and press the combination on your keyboard.")
        .child("• At most one non-modifier key is allowed per combination.")
    });

    let detect_method = v_flex()
      .gap_2()
      .child(div().text_base().font_semibold().child("Detection Method"))
      .child(
        div()
          .text_xs()
          .text_color(muted)
          .child("How to detect if the focused window is a game"),
      )
      .child(
        ButtonGroup::new("detect-method")
          .outline()
          .children(DetectMethod::ALL.iter().map(|method| {
            Button::new(SharedString::from(method.display_name()))
              .label(method.display_name())
              .selected(*method == current_method)
          }))
          .on_click(cx.listener(|this, clicked: &Vec<usize>, _, cx| {
            if let Some(&ix) = clicked.first() {
              this.config.detect_method = DetectMethod::ALL[ix];
              cx.notify();
            }
          })),
      );

    let error = self.error_message.clone().map(|msg| {
      div()
        .p_2()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().danger)
        .text_xs()
        .text_color(cx.theme().danger)
        .child(msg)
    });

    let actions = h_flex()
      .gap_3()
      .child(
        Button::new("save")
          .label("Save")
          .primary()
          .on_click(cx.listener(|this, _, _, cx| {
            this.config.save();
            cx.quit();
          })),
      )
      .child(
        Button::new("cancel")
          .label("Cancel")
          .on_click(|_, _, cx| cx.quit()),
      );

    div().size_full().bg(cx.theme().background).child(
      v_flex()
        .size_full()
        .p_5()
        .gap_4()
        .overflow_y_scrollbar()
        .child(header)
        .children(help)
        .child(self.render_section(
          ListKind::Blacklist,
          "Blocked Keys (Blacklist)",
          "Keys that will be blocked when active",
          "No blocked keys configured",
          cx,
        ))
        .child(self.render_section(
          ListKind::Whitelist,
          "Allowed Keys (Whitelist)",
          "Keys that will be allowed even if they match blacklist rules",
          "No allowed keys configured",
          cx,
        ))
        .child(detect_method)
        .children(error)
        .child(actions),
    )
  }
}

pub fn run(config: KeyConfig) -> Result<(), Box<dyn std::error::Error>> {
  Application::new().run(move |cx: &mut App| {
    gpui_component::init(cx);

    cx.on_window_closed(|cx| {
      if cx.windows().is_empty() {
        cx.quit();
      }
    })
    .detach();

    let bounds = Bounds::centered(None, size(px(520.0), px(640.0)), cx);
    let options = WindowOptions {
      window_bounds: Some(WindowBounds::Windowed(bounds)),
      window_min_size: Some(size(px(420.0), px(500.0))),
      titlebar: Some(TitlebarOptions {
        title: Some(CONFIG_WINDOW_TITLE_GPUI.into()),
        ..Default::default()
      }),
      ..Default::default()
    };

    let _ = cx.open_window(options, move |window, cx| {
      let view = cx.new(|cx| GpuiComponentConfigUI::new(config.clone(), window, cx));
      cx.new(|cx| Root::new(view, window, cx))
    });
    cx.activate(true);
  });

  Ok(())
}
