use std::time::Duration;

use crate::{
  config::{DetectMethod, KeyCombo, KeyConfig},
  config_ui::{
    CONFIG_WINDOW_TITLE_GPUI, is_recording_mode, set_recording_mode, take_recorded_combo,
  },
};
use gpui::{
  App, Application, Bounds, Context, CursorStyle, FocusHandle, Focusable, KeyDownEvent,
  MouseButton, MouseDownEvent, MouseUpEvent, SharedString, TitlebarOptions, Window, WindowBounds,
  WindowOptions, div, prelude::*, px, rgb, size,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveInput {
  Blacklist,
  Whitelist,
}

pub struct GpuiConfigUI {
  config: KeyConfig,
  blacklist_input: String,
  whitelist_input: String,
  active_input: ActiveInput,
  last_recording_state: bool,
  error_message: Option<String>,
  show_help: bool,
  focus_handle: FocusHandle,
}

impl Focusable for GpuiConfigUI {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}

impl GpuiConfigUI {
  fn new(config: KeyConfig, cx: &mut Context<Self>) -> Self {
    cx.spawn(async move |this, cx| {
      loop {
        cx.background_executor()
          .timer(Duration::from_millis(25))
          .await;
        let should_continue = this
          .update(cx, |view, cx| {
            view.poll_recorded_keys(cx);
          })
          .is_ok();
        if !should_continue {
          break;
        }
      }
    })
    .detach();

    Self {
      config,
      blacklist_input: String::new(),
      whitelist_input: String::new(),
      active_input: ActiveInput::Blacklist,
      last_recording_state: false,
      error_message: None,
      show_help: false,
      focus_handle: cx.focus_handle(),
    }
  }

  fn poll_recorded_keys(&mut self, cx: &mut Context<Self>) {
    let mut changed = false;
    if let Some(combo) = take_recorded_combo() {
      self.error_message = None;
      *self.current_input_mut() = combo;
      changed = true;
    }
    let rec = is_recording_mode();
    if rec != self.last_recording_state {
      self.last_recording_state = rec;
      changed = true;
    }
    if changed {
      cx.notify();
    }
  }

  fn current_input_mut(&mut self) -> &mut String {
    match self.active_input {
      ActiveInput::Blacklist => &mut self.blacklist_input,
      ActiveInput::Whitelist => &mut self.whitelist_input,
    }
  }

  fn add_blacklist_key(&mut self, cx: &mut Context<Self>) {
    self.error_message = None;
    let trimmed = self.blacklist_input.trim();
    if !trimmed.is_empty() {
      match KeyCombo::from_string(trimmed) {
        Ok(combo) => {
          self.config.blacklist.push(combo);
          self.blacklist_input.clear();
        },
        Err(e) => {
          self.error_message = Some(format!("Invalid key combination: {}", e));
        },
      }
      cx.notify();
    }
  }

  fn add_whitelist_key(&mut self, cx: &mut Context<Self>) {
    self.error_message = None;
    let trimmed = self.whitelist_input.trim();
    if !trimmed.is_empty() {
      match KeyCombo::from_string(trimmed) {
        Ok(combo) => {
          self.config.whitelist.push(combo);
          self.whitelist_input.clear();
        },
        Err(e) => {
          self.error_message = Some(format!("Invalid key combination: {}", e));
        },
      }
      cx.notify();
    }
  }

  fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
    self.error_message = None;
    let ks = &event.keystroke;

    if ks.modifiers.control && ks.key.eq_ignore_ascii_case("v") {
      if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
        self.current_input_mut().push_str(text.trim());
        cx.notify();
      }
      return;
    }

    match ks.key.as_str() {
      "backspace" => {
        self.current_input_mut().pop();
        cx.notify();
      },
      "enter" => match self.active_input {
        ActiveInput::Blacklist => self.add_blacklist_key(cx),
        ActiveInput::Whitelist => self.add_whitelist_key(cx),
      },
      "tab" => {
        self.active_input = match self.active_input {
          ActiveInput::Blacklist => ActiveInput::Whitelist,
          ActiveInput::Whitelist => ActiveInput::Blacklist,
        };
        cx.notify();
      },
      "escape" => {
        self.current_input_mut().clear();
        cx.notify();
      },
      _ => {
        if !ks.modifiers.control && !ks.modifiers.alt && !ks.modifiers.platform {
          if let Some(ref key_char) = ks.key_char {
            if key_char.chars().all(|c| !c.is_control()) {
              self.current_input_mut().push_str(key_char);
              cx.notify();
            }
          } else if ks.key.len() == 1 {
            self.current_input_mut().push_str(&ks.key);
            cx.notify();
          } else if ks.key == "space" {
            self.current_input_mut().push(' ');
            cx.notify();
          }
        }
      },
    }
  }
}

impl Render for GpuiConfigUI {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let show_help = self.show_help;
    let active_input = self.active_input;
    let current_method = self.config.detect_method;

    let header = div()
      .w_full()
      .flex()
      .flex_row()
      .items_center()
      .justify_between()
      .gap_3()
      .child(
        div()
          .flex_1()
          .min_w(px(0.0))
          .whitespace_normal()
          .text_sm()
          .text_color(rgb(0x334155))
          .child("Configure which keys should be blocked or allowed. Examples: 'lwin', 'ctrl+c', 'shift+alt+tab'"),
      )
      .child(
        div()
          .id("toggle-help")
          .flex_shrink_0()
          .px_3()
          .py_1()
          .rounded_md()
          .bg(rgb(0xe2e8f0))
          .text_sm()
          .text_color(rgb(0x1e293b))
          .cursor_pointer()
          .hover(|s| s.bg(rgb(0xcbd5e1)))
          .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _: &MouseUpEvent, _window, cx| {
              this.show_help = !this.show_help;
              cx.notify();
            }),
          )
          .child(if show_help { "Hide Help" } else { "Show Help" }),
      );

    let help_section = if show_help {
      div()
        .w_full()
        .whitespace_normal()
        .flex()
        .flex_col()
        .gap_1()
        .p_3()
        .rounded_md()
        .bg(rgb(0xf1f5f9))
        .border_1()
        .border_color(rgb(0xcbd5e1))
        .text_xs()
        .text_color(rgb(0x334155))
        .child(
          div()
            .text_sm()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(0x0f172a))
            .child("How to enter key combinations"),
        )
        .child("• Use '+' between each modifier and key, e.g. 'ctrl+shift+esc'.")
        .child("• Supported modifiers: 'ctrl', 'alt', 'shift', 'lwin', 'rwin'.")
        .child("• Finish with a key name like 'c', 'f12', 'space', 'delete', or just a modifier.")
        .child("• Names are case-insensitive; spaces around '+' are optional.")
        .child("• At most one non-modifier key is allowed per combination.")
    } else {
      div()
    };

    let blacklist_rows = if self.config.blacklist.is_empty() {
      div()
        .text_sm()
        .text_color(rgb(0x64748b))
        .py_1()
        .child("No blocked keys configured")
    } else {
      div()
        .flex()
        .flex_col()
        .gap_1()
        .children(
          self
            .config
            .blacklist
            .iter()
            .enumerate()
            .map(|(i, combo)| {
              let label = SharedString::from(combo.string_repr.clone());
              div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_3()
                .py_1()
                .rounded_md()
                .bg(rgb(0xffffff))
                .border_1()
                .border_color(rgb(0xe2e8f0))
                .child(div().text_sm().text_color(rgb(0x0f172a)).child(label))
                .child(
                  div()
                    .id(("remove-blacklist", i))
                    .px_2()
                    .py_0p5()
                    .rounded_sm()
                    .bg(rgb(0xef4444))
                    .text_xs()
                    .text_color(rgb(0xffffff))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0xdc2626)))
                    .on_mouse_up(
                      MouseButton::Left,
                      cx.listener(move |this, _: &MouseUpEvent, _window, cx| {
                        if i < this.config.blacklist.len() {
                          this.config.blacklist.remove(i);
                          cx.notify();
                        }
                      }),
                    )
                    .child("Remove"),
                )
            })
            .collect::<Vec<_>>(),
        )
    };

    let recording = is_recording_mode();
    let blacklist_focused = active_input == ActiveInput::Blacklist;
    let blacklist_recording = recording && blacklist_focused;
    let blacklist_display = if self.blacklist_input.is_empty() {
      if blacklist_recording {
        "Press any key or combination...".to_string()
      } else if blacklist_focused {
        "|".to_string()
      } else {
        "Enter key combination (or click Record)".to_string()
      }
    } else if blacklist_focused {
      format!("{}|", self.blacklist_input)
    } else {
      self.blacklist_input.clone()
    };

    let blacklist_input_row = div()
      .w_full()
      .flex()
      .flex_row()
      .items_center()
      .gap_2()
      .child(
        div()
          .id("blacklist-input")
          .flex_1()
          .min_w(px(0.0))
          .overflow_hidden()
          .px_3()
          .py_1p5()
          .rounded_md()
          .bg(rgb(0xffffff))
          .border_1()
          .border_color(if blacklist_recording {
            rgb(0xd97706)
          } else if blacklist_focused {
            rgb(0x2563eb)
          } else {
            rgb(0xcbd5e1)
          })
          .cursor(CursorStyle::IBeam)
          .text_sm()
          .text_color(if self.blacklist_input.is_empty() && !blacklist_focused {
            rgb(0x94a3b8)
          } else {
            rgb(0x0f172a)
          })
          .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _: &MouseDownEvent, window, cx| {
              set_recording_mode(false);
              this.active_input = ActiveInput::Blacklist;
              window.focus(&this.focus_handle);
              cx.notify();
            }),
          )
          .child(blacklist_display),
      )
      .child(
        div()
          .id("record-blacklist")
          .flex_shrink_0()
          .px_3()
          .py_1p5()
          .rounded_md()
          .bg(if blacklist_recording {
            rgb(0xd97706)
          } else {
            rgb(0x475569)
          })
          .text_sm()
          .text_color(rgb(0xffffff))
          .cursor_pointer()
          .hover(|s| {
            s.bg(if blacklist_recording {
              rgb(0xb45309)
            } else {
              rgb(0x334155)
            })
          })
          .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, _: &MouseUpEvent, window, cx| {
              this.active_input = ActiveInput::Blacklist;
              set_recording_mode(!blacklist_recording);
              window.focus(&this.focus_handle);
              cx.notify();
            }),
          )
          .child(if blacklist_recording { "Stop" } else { "Record" }),
      )
      .child(
        div()
          .id("add-blacklist")
          .flex_shrink_0()
          .px_3()
          .py_1p5()
          .rounded_md()
          .bg(rgb(0x16a34a))
          .text_sm()
          .text_color(rgb(0xffffff))
          .cursor_pointer()
          .hover(|s| s.bg(rgb(0x15803d)))
          .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _: &MouseUpEvent, _window, cx| {
              set_recording_mode(false);
              this.add_blacklist_key(cx);
            }),
          )
          .child("Add"),
      );

    let whitelist_rows = if self.config.whitelist.is_empty() {
      div()
        .text_sm()
        .text_color(rgb(0x64748b))
        .py_1()
        .child("No allowed keys configured")
    } else {
      div()
        .flex()
        .flex_col()
        .gap_1()
        .children(
          self
            .config
            .whitelist
            .iter()
            .enumerate()
            .map(|(i, combo)| {
              let label = SharedString::from(combo.string_repr.clone());
              div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_3()
                .py_1()
                .rounded_md()
                .bg(rgb(0xffffff))
                .border_1()
                .border_color(rgb(0xe2e8f0))
                .child(div().text_sm().text_color(rgb(0x0f172a)).child(label))
                .child(
                  div()
                    .id(("remove-whitelist", i))
                    .px_2()
                    .py_0p5()
                    .rounded_sm()
                    .bg(rgb(0xef4444))
                    .text_xs()
                    .text_color(rgb(0xffffff))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0xdc2626)))
                    .on_mouse_up(
                      MouseButton::Left,
                      cx.listener(move |this, _: &MouseUpEvent, _window, cx| {
                        if i < this.config.whitelist.len() {
                          this.config.whitelist.remove(i);
                          cx.notify();
                        }
                      }),
                    )
                    .child("Remove"),
                )
            })
            .collect::<Vec<_>>(),
        )
    };

    let whitelist_focused = active_input == ActiveInput::Whitelist;
    let whitelist_recording = recording && whitelist_focused;
    let whitelist_display = if self.whitelist_input.is_empty() {
      if whitelist_recording {
        "Press any key or combination...".to_string()
      } else if whitelist_focused {
        "|".to_string()
      } else {
        "Enter key combination (or click Record)".to_string()
      }
    } else if whitelist_focused {
      format!("{}|", self.whitelist_input)
    } else {
      self.whitelist_input.clone()
    };

    let whitelist_input_row = div()
      .w_full()
      .flex()
      .flex_row()
      .items_center()
      .gap_2()
      .child(
        div()
          .id("whitelist-input")
          .flex_1()
          .min_w(px(0.0))
          .overflow_hidden()
          .px_3()
          .py_1p5()
          .rounded_md()
          .bg(rgb(0xffffff))
          .border_1()
          .border_color(if whitelist_recording {
            rgb(0xd97706)
          } else if whitelist_focused {
            rgb(0x2563eb)
          } else {
            rgb(0xcbd5e1)
          })
          .cursor(CursorStyle::IBeam)
          .text_sm()
          .text_color(if self.whitelist_input.is_empty() && !whitelist_focused {
            rgb(0x94a3b8)
          } else {
            rgb(0x0f172a)
          })
          .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _: &MouseDownEvent, window, cx| {
              set_recording_mode(false);
              this.active_input = ActiveInput::Whitelist;
              window.focus(&this.focus_handle);
              cx.notify();
            }),
          )
          .child(whitelist_display),
      )
      .child(
        div()
          .id("record-whitelist")
          .flex_shrink_0()
          .px_3()
          .py_1p5()
          .rounded_md()
          .bg(if whitelist_recording {
            rgb(0xd97706)
          } else {
            rgb(0x475569)
          })
          .text_sm()
          .text_color(rgb(0xffffff))
          .cursor_pointer()
          .hover(|s| {
            s.bg(if whitelist_recording {
              rgb(0xb45309)
            } else {
              rgb(0x334155)
            })
          })
          .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, _: &MouseUpEvent, window, cx| {
              this.active_input = ActiveInput::Whitelist;
              set_recording_mode(!whitelist_recording);
              window.focus(&this.focus_handle);
              cx.notify();
            }),
          )
          .child(if whitelist_recording { "Stop" } else { "Record" }),
      )
      .child(
        div()
          .id("add-whitelist")
          .flex_shrink_0()
          .px_3()
          .py_1p5()
          .rounded_md()
          .bg(rgb(0x16a34a))
          .text_sm()
          .text_color(rgb(0xffffff))
          .cursor_pointer()
          .hover(|s| s.bg(rgb(0x15803d)))
          .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _: &MouseUpEvent, _window, cx| {
              set_recording_mode(false);
              this.add_whitelist_key(cx);
            }),
          )
          .child("Add"),
      );

    let detect_method_row = div()
      .w_full()
      .flex()
      .flex_row()
      .flex_wrap()
      .gap_2()
      .children(DetectMethod::ALL.iter().copied().enumerate().map(|(i, method)| {
        let selected = method == current_method;
        div()
          .id(("detect-method", i))
          .flex_1()
          .px_3()
          .py_1p5()
          .rounded_md()
          .border_1()
          .border_color(if selected {
            rgb(0x2563eb)
          } else {
            rgb(0xcbd5e1)
          })
          .bg(if selected {
            rgb(0x2563eb)
          } else {
            rgb(0xffffff)
          })
          .text_xs()
          .text_center()
          .text_color(if selected {
            rgb(0xffffff)
          } else {
            rgb(0x1e293b)
          })
          .cursor_pointer()
          .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, _: &MouseUpEvent, _window, cx| {
              this.config.detect_method = method;
              cx.notify();
            }),
          )
          .child(method.display_name())
      }));

    let error_banner = if let Some(ref err) = self.error_message {
      div()
        .w_full()
        .whitespace_normal()
        .p_2()
        .rounded_md()
        .bg(rgb(0xfee2e2))
        .border_1()
        .border_color(rgb(0xfca5a5))
        .text_xs()
        .text_color(rgb(0xb91c1c))
        .child(err.clone())
    } else {
      div()
    };

    let action_buttons = div()
      .flex()
      .flex_row()
      .gap_3()
      .pt_2()
      .child(
        div()
          .id("save-button")
          .px_4()
          .py_1p5()
          .rounded_md()
          .bg(rgb(0x2563eb))
          .text_sm()
          .text_color(rgb(0xffffff))
          .cursor_pointer()
          .hover(|s| s.bg(rgb(0x1d4ed8)))
          .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _: &MouseUpEvent, _window, cx| {
              this.config.save();
              cx.quit();
            }),
          )
          .child("Save"),
      )
      .child(
        div()
          .id("cancel-button")
          .px_4()
          .py_1p5()
          .rounded_md()
          .bg(rgb(0xe2e8f0))
          .text_sm()
          .text_color(rgb(0x1e293b))
          .cursor_pointer()
          .hover(|s| s.bg(rgb(0xcbd5e1)))
          .on_mouse_up(
            MouseButton::Left,
            cx.listener(|_this, _: &MouseUpEvent, _window, cx| {
              cx.quit();
            }),
          )
          .child("Cancel"),
      );

    div()
      .id("gpui-config-root")
      .track_focus(&self.focus_handle)
      .on_key_down(cx.listener(Self::on_key_down))
      .size_full()
      .overflow_y_scroll()
      .bg(rgb(0xf8fafc))
      .p_5()
      .flex()
      .flex_col()
      .items_center()
      .child(
        div()
          .w_full()
          .flex()
          .flex_col()
          .gap_4()
          .child(header)
          .child(help_section)
          .child(
            div()
              .flex()
              .flex_col()
              .gap_2()
              .child(
                div()
                  .text_base()
                  .font_weight(gpui::FontWeight::SEMIBOLD)
                  .text_color(rgb(0x0f172a))
                  .child("Blocked Keys (Blacklist)"),
              )
              .child(
                div()
                  .text_xs()
                  .text_color(rgb(0x64748b))
                  .child("Keys that will be blocked when active"),
              )
              .child(blacklist_rows)
              .child(blacklist_input_row),
          )
          .child(
            div()
              .flex()
              .flex_col()
              .gap_2()
              .child(
                div()
                  .text_base()
                  .font_weight(gpui::FontWeight::SEMIBOLD)
                  .text_color(rgb(0x0f172a))
                  .child("Allowed Keys (Whitelist)"),
              )
              .child(
                div()
                  .text_xs()
                  .text_color(rgb(0x64748b))
                  .child("Keys that will be allowed even if they match blacklist rules"),
              )
              .child(whitelist_rows)
              .child(whitelist_input_row),
          )
          .child(
            div()
              .flex()
              .flex_col()
              .gap_2()
              .child(
                div()
                  .text_base()
                  .font_weight(gpui::FontWeight::SEMIBOLD)
                  .text_color(rgb(0x0f172a))
                  .child("Detection Method"),
              )
              .child(
                div()
                  .text_xs()
                  .text_color(rgb(0x64748b))
                  .child("How to detect if the focused window is a game"),
              )
              .child(detect_method_row),
          )
          .child(error_banner)
          .child(action_buttons),
      )
  }
}

pub fn run(config: KeyConfig) -> Result<(), Box<dyn std::error::Error>> {
  Application::new().run(move |cx: &mut App| {
    cx.on_window_closed(|cx| {
      if cx.windows().is_empty() {
        cx.quit();
      }
    })
    .detach();

    let bounds = Bounds::centered(None, size(px(480.0), px(600.0)), cx);
    let config_clone = config.clone();

    if let Ok(window) = cx.open_window(
      WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(400.0), px(500.0))),
        titlebar: Some(TitlebarOptions {
          title: Some(CONFIG_WINDOW_TITLE_GPUI.into()),
          ..Default::default()
        }),
        ..Default::default()
      },
      move |_, cx| cx.new(|cx| GpuiConfigUI::new(config_clone, cx)),
    ) {
      let _ = window.update(cx, |view, window, cx| {
        window.focus(&view.focus_handle);
        cx.activate(true);
      });
    }
  });

  Ok(())
}
