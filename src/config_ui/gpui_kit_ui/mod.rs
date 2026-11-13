//! Config UI built on `gpui-kit` with modular architecture:
//! - `theme`: Palette tokens and color constants.
//! - `components`: Reusable `RenderOnce` UI primitives (`PillButton`, `KeyRow`, `DetectMethodCard`, `HelpCard`).
//! - `mod`: Application view, state management, event orchestration, and window setup.

pub mod components;
pub mod theme;

use std::time::Duration;

use components::DetectMethodCard;
use components::HelpCard;
use components::KeyRow;
use components::PillButton;
use gpui_kit::App;
use gpui_kit::AppContext;
use gpui_kit::Bounds;
use gpui_kit::Context;
use gpui_kit::ElementId;
use gpui_kit::Entity;
use gpui_kit::FontWeight;
use gpui_kit::IntoElement;
use gpui_kit::MouseUpEvent;
use gpui_kit::ParentElement;
use gpui_kit::Render;
use gpui_kit::SharedString;
use gpui_kit::Styled;
use gpui_kit::TitlebarOptions;
use gpui_kit::Window;
use gpui_kit::WindowBounds;
use gpui_kit::WindowOptions;
use gpui_kit::base::ScrollbarMode;
use gpui_kit::component::Theme;
use gpui_kit::component::h_flex;
use gpui_kit::component::input::Input;
use gpui_kit::component::input::InputEvent;
use gpui_kit::component::input::InputState;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::v_flex;
use gpui_kit::div;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::px;
use gpui_kit::rgb;
use gpui_kit::size;
use theme::Palette;

use crate::config::DetectMethod;
use crate::config::KeyCombo;
use crate::config::KeyConfig;
use crate::config_ui::CONFIG_WINDOW_TITLE;
use crate::config_ui::is_recording_mode;
use crate::config_ui::set_recording_mode;
use crate::config_ui::take_recorded_combo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListKind {
  Blacklist,
  Whitelist,
}

pub struct GpuiKitConfigUI {
  config: KeyConfig,
  blacklist_input: Entity<InputState>,
  whitelist_input: Entity<InputState>,
  recording_target: ListKind,
  error_message: Option<SharedString>,
  show_help: bool,
}

impl GpuiKitConfigUI {
  fn new(config: KeyConfig, window: &mut Window, cx: &mut Context<Self>) -> Self {
    let blacklist_input = cx.new(|cx| {
      InputState::new(window, cx)
        .placeholder("Type a combo (e.g. 'lwin', 'lwin+s') or click Record")
    });
    let whitelist_input = cx.new(|cx| {
      InputState::new(window, cx).placeholder("Type a combo (e.g. 'lwin+shift+s') or click Record")
    });

    for (input, kind) in [
      (&blacklist_input, ListKind::Blacklist),
      (&whitelist_input, ListKind::Whitelist),
    ] {
      cx.subscribe_in(
        input,
        window,
        move |this, _, event: &InputEvent, window, cx| {
          if let InputEvent::PressEnter {
            ..
          } = event
          {
            this.add_combo(kind, window, cx);
          }
        },
      )
      .detach();
    }

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

    let rows = if items.is_empty() {
      v_flex().child(
        div()
          .text_sm()
          .text_color(rgb(Palette::TEXT_MUTED))
          .py_1()
          .child(empty_text),
      )
    } else {
      v_flex()
        .gap_1()
        .children(items.iter().enumerate().map(|(i, combo)| {
          KeyRow {
            label: SharedString::from(combo.string_repr.clone()),
            remove_id: ElementId::from((SharedString::from(format!("{prefix}-remove")), i)),
            on_remove: Box::new(cx.listener(move |this, _: &MouseUpEvent, _window, cx| {
              let list = this.list_mut(kind);
              if i < list.len() {
                list.remove(i);
                cx.notify();
              }
            })),
          }
        }))
    };

    v_flex()
      .gap_2()
      .child(
        div()
          .text_base()
          .font_weight(FontWeight::SEMIBOLD)
          .text_color(rgb(Palette::TEXT_PRIMARY))
          .child(title),
      )
      .child(
        div()
          .text_xs()
          .text_color(rgb(Palette::TEXT_MUTED))
          .child(description),
      )
      .child(rows)
      .child(
        h_flex()
          .items_center()
          .gap_2()
          .child(
            div()
              .flex_1()
              .min_w(px(0.0))
              .child(Input::new(self.input_for(kind))),
          )
          .child(PillButton::record(
            SharedString::from(format!("{prefix}-record")),
            recording,
            cx.listener(move |this, _: &MouseUpEvent, _window, cx| {
              this.toggle_record(kind, cx);
            }),
          ))
          .child(PillButton::success(
            SharedString::from(format!("{prefix}-add")),
            "Add",
            cx.listener(move |this, _: &MouseUpEvent, window, cx| {
              this.add_combo(kind, window, cx);
            }),
          ).tooltip(match kind {
            ListKind::Blacklist => "Add the entered combination to the blocked keys. Click Save to apply the change.",
            ListKind::Whitelist => "Add the entered combination to the allowed keys, overriding matching blocked rules. Click Save to apply the change.",
          })),
      )
  }
}

impl Render for GpuiKitConfigUI {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let show_help = self.show_help;
    let current_method = self.config.detect_method;

    let header = h_flex()
      .items_center()
      .justify_between()
      .gap_3()
      .child(
        div()
          .flex_1()
          .min_w(px(0.0))
          .whitespace_normal()
          .text_sm()
          .text_color(rgb(Palette::TEXT_SECONDARY))
          .child("Configure which keys should be blocked or allowed. Examples: 'lwin', 'ctrl+c', 'shift+alt+tab'"),
      )
      .child(PillButton::secondary(
        "toggle-help",
        if show_help { "Hide Help" } else { "Show Help" },
        cx.listener(|this, _: &MouseUpEvent, _window, cx| {
          this.show_help = !this.show_help;
          cx.notify();
        }),
      ).tooltip(if show_help {
        "Hide the instructions for entering key combinations."
      } else {
        "Show supported key names, modifiers, and examples of key combinations."
      }));

    let detect_method_section = v_flex()
      .gap_2()
      .child(
        div()
          .text_base()
          .font_weight(FontWeight::SEMIBOLD)
          .text_color(rgb(Palette::TEXT_PRIMARY))
          .child("Detection Method"),
      )
      .child(
        div()
          .text_xs()
          .text_color(rgb(Palette::TEXT_MUTED))
          .child("How to detect if the focused window is a game"),
      )
      .child(
        h_flex()
          .flex_wrap()
          .gap_2()
          .children(
            DetectMethod::ALL
              .iter()
              .copied()
              .enumerate()
              .map(|(i, method)| {
                DetectMethodCard {
                  id: ElementId::from(("detect-method", i)),
                  name: method.display_name(),
                  description: match method {
                    DetectMethod::NotificationState => "Block configured keys when Windows reports that you are busy or running a fullscreen Direct3D app. This can also activate for non-game apps.",
                    DetectMethod::Fullscreen => "Block configured keys when the focused window covers its entire monitor, including borderless fullscreen apps. This can also activate for non-game apps.",
                    DetectMethod::WindowStyle => "Block configured keys when the focused window has no title bar, no system menu, and no child windows. This is a game-detection heuristic and may also match other apps.",
                  },
                  selected: method == current_method,
                  on_select: Box::new(cx.listener(move |this, _: &MouseUpEvent, _window, cx| {
                    this.config.detect_method = method;
                    cx.notify();
                  })),
                }
              }),
          ),
      );

    let action_buttons = h_flex()
      .w_full()
      .flex_shrink_0()
      .items_center()
      .justify_end()
      .gap_3()
      .px_5()
      .py_3()
      .bg(rgb(Palette::CANVAS))
      .border_t_1()
      .border_color(rgb(Palette::BORDER_CARD))
      .child(PillButton::primary(
        "save-button",
        "Save",
        cx.listener(|this, _: &MouseUpEvent, _window, cx| {
          this.config.save();
          cx.quit();
        }),
      ))
      .child(PillButton::secondary(
        "cancel-button",
        "Cancel",
        cx.listener(|_this, _: &MouseUpEvent, _window, cx| {
          cx.quit();
        }),
      ));

    v_flex()
      .size_full()
      .bg(rgb(Palette::CANVAS))
      .child(
        v_flex()
          .flex_1()
          .min_h(px(0.0))
          .p_5()
          .gap_4()
          .overflow_y_scrollbar()
          .child(header)
          .when(show_help, |this| this.child(HelpCard))
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
          .child(detect_method_section)
          .when_some(self.error_message.as_ref(), |this, err| {
            this.child(
              div()
                .w_full()
                .whitespace_normal()
                .p_2()
                .rounded_md()
                .bg(rgb(Palette::ERROR_BG))
                .border_1()
                .border_color(rgb(Palette::ERROR_BORDER))
                .text_xs()
                .text_color(rgb(Palette::ERROR_TEXT))
                .child(err.clone()),
            )
          }),
      )
      .child(action_buttons)
  }
}

pub fn run(config: KeyConfig) -> Result<(), Box<dyn std::error::Error>> {
  gpui_kit::application().run(move |cx: &mut App| {
    gpui_kit::init(cx);

    // Ensure the vertical scrollbar is permanently visible whenever content overflows
    Theme::update(cx, |theme| {
      theme.scrollbar_mode = ScrollbarMode::Always;
      theme.colors.scrollbar_thumb = rgb(0x94a3b8).into(); // slate-400 thumb
      theme.colors.scrollbar_thumb_hover = rgb(0x64748b).into(); // slate-500 thumb on hover
    });

    cx.on_window_closed(|cx, _window_id| {
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
        title: Some(CONFIG_WINDOW_TITLE.into()),
        ..Default::default()
      }),
      ..Default::default()
    };

    let _ = gpui_kit::open_window(options, cx, |window, cx| {
      cx.new(|cx| GpuiKitConfigUI::new(config.clone(), window, cx))
    });
    cx.activate(true);
  });

  Ok(())
}
