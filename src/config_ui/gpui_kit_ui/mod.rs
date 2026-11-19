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
use gpui_kit::assets::IconName;
use gpui_kit::base::InteractiveElementExt;
use gpui_kit::component::Icon;
use gpui_kit::component::IndexPath;
use gpui_kit::component::TitleBar;
use gpui_kit::component::h_flex;
use gpui_kit::component::input::Input;
use gpui_kit::component::input::InputEvent;
use gpui_kit::component::input::InputState;
use gpui_kit::component::resizable::HandleEdge;
use gpui_kit::component::resizable::resize_handle;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::select::SearchableVec;
use gpui_kit::component::select::Select;
use gpui_kit::component::select::SelectEvent;
use gpui_kit::component::select::SelectState;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::v_flex;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
  Rules,
  Detection,
  Appearance,
  Help,
}
impl Page {
  const ALL: [Self; 4] = [Self::Rules, Self::Detection, Self::Appearance, Self::Help];

  fn title(self) -> &'static str {
    match self {
      Self::Rules => "Key rules",
      Self::Detection => "Game detection",
      Self::Appearance => "Appearance",
      Self::Help => "Help",
    }
  }

  fn description(self) -> &'static str {
    match self {
      Self::Rules => "Choose which shortcuts stay out of your way, and which ones always work.",
      Self::Detection => "Choose when WinKeyLock protects your keyboard shortcuts.",
      Self::Appearance => "Make WinKeyLock feel at home on your desktop.",
      Self::Help => "A quick guide to key names, modifiers, and recording shortcuts.",
    }
  }

  fn icon(self) -> IconName {
    match self {
      Self::Rules => IconName::SquareTerminal,
      Self::Detection => IconName::Frame,
      Self::Appearance => IconName::Palette,
      Self::Help => IconName::FileText,
    }
  }

  fn matches(self, query: &str) -> bool {
    let keywords = match self {
      Self::Rules => {
        "blocked allowed keys blacklist whitelist shortcut record add remove combination"
      },
      Self::Detection => "detection method notification state fullscreen window style game busy",
      Self::Appearance => {
        "theme colors colour dark light frosted glass blur transparency translucent window"
      },
      Self::Help => "help instructions modifiers supported names examples keyboard",
    };
    let text = format!("{} {} {keywords}", self.title(), self.description()).to_lowercase();
    query.split_whitespace().all(|word| text.contains(word))
  }
}

#[derive(Clone)]
struct SidebarResize;
impl Render for SidebarResize {
  fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
    div()
  }
}

pub struct GpuiKitConfigUI {
  config: KeyConfig,
  blacklist_input: Entity<InputState>,
  whitelist_input: Entity<InputState>,
  recording_target: ListKind,
  error_message: Option<SharedString>,
  page: Page,
  search_input: Entity<InputState>,
  sidebar_width: f32,
  theme_select: Entity<SelectState<SearchableVec<SharedString>>>,
}

impl GpuiKitConfigUI {
  fn new(mut config: KeyConfig, window: &mut Window, cx: &mut Context<Self>) -> Self {
    theme::init_glass(config.frosted_glass, cx);
    let selected = theme::apply(config.theme.as_deref().unwrap_or(theme::DEFAULT_THEME), cx);
    config.theme = Some(selected.to_string());
    let names = theme::theme_names(cx);
    let index = names.iter().position(|name| name == &selected).unwrap_or(0);
    let theme_select = cx.new(|cx| {
      SelectState::new(
        SearchableVec::new(names),
        Some(IndexPath::new(index)),
        window,
        cx,
      )
      .searchable(true)
    });
    cx.subscribe_in(&theme_select, window, |this, _, event, _, cx| {
      if let SelectEvent::Confirm(Some(name)) = event {
        let selected = theme::apply(name, cx);
        this.config.theme = Some(selected.to_string());
        cx.notify();
      }
    })
    .detach();
    let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("Search settings"));
    cx.subscribe_in(&search_input, window, |_, _, event, _, cx| {
      if matches!(event, InputEvent::Change) {
        cx.notify();
      }
    })
    .detach();
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
      page: Page::Rules,
      search_input,
      sidebar_width: 196.0,
      theme_select,
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
          .text_color(Palette::MutedText.color(cx))
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
      .w_full()
      .min_w(px(0.0))
      .p_4()
      .rounded_xl()
      .bg(Palette::Card.color(cx))
      .gap_2()
      .child(
        div()
          .text_base()
          .font_weight(FontWeight::SEMIBOLD)
          .text_color(Palette::Text.color(cx))
          .child(title),
      )
      .child(
        div()
          .text_xs()
          .text_color(Palette::MutedText.color(cx))
          .child(description),
      )
      .child(rows)
      .child(
        h_flex()
          .w_full()
          .min_w(px(0.0))
          .flex_wrap()
          .items_center()
          .gap_2()
          .child(
            div()
              .flex_1()
              .min_w(px(160.0))
              .px_3()
              .rounded_md()
              .bg(Palette::Help.color(cx))
              .child(Input::new(self.input_for(kind)).appearance(false).px_0()),
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

impl GpuiKitConfigUI {
  fn select_page(&mut self, page: Page, window: &mut Window, cx: &mut Context<Self>) {
    set_recording_mode(false);
    self.page = page;
    self
      .search_input
      .update(cx, |input, cx| input.set_value("", window, cx));
    cx.notify();
  }

  fn nav_item(
    &self,
    page: Page,
    searching: bool,
    cx: &mut Context<Self>,
  ) -> impl IntoElement + use<> {
    let selected = self.page == page && !searching;
    h_flex()
      .id(page.title())
      .min_w(px(0.0))
      .gap_2()
      .px_3()
      .py_2()
      .rounded_md()
      .cursor_pointer()
      .bg(if selected {
        Palette::Selection.color(cx)
      } else {
        transparent_black()
      })
      .text_size(px(13.0))
      .text_color(if selected {
        Palette::Text.color(cx)
      } else {
        Palette::MutedText.color(cx)
      })
      .hover(|s| s.bg(Palette::Selection.color(cx)))
      .on_mouse_up(
        MouseButton::Left,
        cx.listener(move |this, _, window, cx| this.select_page(page, window, cx)),
      )
      .child(Icon::new(page.icon()).size_4())
      .child(div().min_w(px(0.0)).whitespace_normal().child(page.title()))
  }

  fn detection(&self, cx: &mut Context<Self>) -> impl IntoElement {
    v_flex().gap_2().children(DetectMethod::ALL.iter().copied().enumerate().map(|(i, method)| {
      DetectMethodCard {
        id: ElementId::from(("detect-method", i)), name: method.display_name(),
        description: match method {
          DetectMethod::NotificationState => "Use Windows’ busy or fullscreen Direct3D status. Some non-game apps can also activate this method.",
          DetectMethod::Fullscreen => "Detect windows covering their entire monitor, including borderless fullscreen apps.",
          DetectMethod::WindowStyle => "Detect windows without a title bar, system menu, or child windows. This heuristic can also match non-game apps.",
        },
        selected: self.config.detect_method == method,
        on_select: Box::new(cx.listener(move |this, _, _, cx| { this.config.detect_method = method; cx.notify(); })),
      }
    }))
  }

  fn appearance(&self, cx: &mut Context<Self>) -> impl IntoElement {
    v_flex().gap_4()
      .child(v_flex().gap_3().p_4().rounded_xl().bg(Palette::Card.color(cx))
        .child(div().font_weight(FontWeight::MEDIUM).child("Theme"))
        .child(div().text_xs().text_color(Palette::MutedText.color(cx)).child("Choose the app’s colors. Preview changes immediately; Save keeps your choice."))
        .child(div().w_full().max_w(px(300.0)).child(Select::new(&self.theme_select).search_placeholder("Search themes…").accessibility_label("Theme").w_full())))
      .child(h_flex().gap_4().p_4().rounded_xl().bg(Palette::Card.color(cx))
        .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
          .child(div().font_weight(FontWeight::MEDIUM).child("Frosted glass"))
          .child(div().text_xs().whitespace_normal().text_color(Palette::MutedText.color(cx)).child(
            if theme::glass_supported(cx) { "Use translucent panels and controls over a blurred desktop. Save keeps your choice." }
            else { "Windows transparency effects are off or unavailable. Solid surfaces are used." })))
        .child(Switch::new("frosted-glass").accessibility_label("Frosted glass")
          .checked(self.config.frosted_glass).color(Palette::Primary.color(cx))
          .on_change(cx.listener(|this, enabled, _, cx| {
            this.config.frosted_glass = *enabled;
            theme::set_glass(*enabled, cx);
            cx.notify();
          }))))
  }
}

impl Render for GpuiKitConfigUI {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let query = self.search_input.read(cx).value().trim().to_lowercase();
    let searching = !query.is_empty();
    let narrow = f32::from(window.viewport_size().width) < 640.0;
    let width = self.sidebar_width.clamp(
      176.0,
      (f32::from(window.viewport_size().width) - 360.0).clamp(176.0, 300.0),
    );
    let search = div()
      .min_w(px(0.0))
      .px_3()
      .rounded_md()
      .bg(Palette::Help.color(cx))
      .child(
        Input::new(&self.search_input)
          .appearance(false)
          .px_0()
          .aria_label("Search settings"),
      );
    let mut nav = v_flex().gap_1();
    for page in Page::ALL {
      if !searching || page.matches(&query) {
        nav = nav.child(self.nav_item(page, searching, cx));
      }
    }
    let mut sidebar = v_flex()
      .relative()
      .min_h(px(0.0))
      .flex_shrink_0()
      .rounded_md()
      .bg(Palette::Panel.color(cx))
      .overflow_hidden()
      .p_3()
      .gap_4()
      .when(!narrow, |s| s.w(px(width)))
      .when(narrow, |s| s.w_full())
      .child(
        div()
          .px_1()
          .font_weight(FontWeight::SEMIBOLD)
          .child("Settings"),
      )
      .child(search);
    if narrow {
      sidebar = sidebar.child(
        h_flex().flex_wrap().gap_1().children(
          Page::ALL
            .into_iter()
            .map(|page| self.nav_item(page, searching, cx)),
        ),
      );
    } else {
      sidebar = sidebar
        .child(nav)
        .child(div().flex_1())
        .child(
          div()
            .px_1()
            .text_xs()
            .text_color(Palette::MutedText.color(cx))
            .child(format!("WinKeyLock {}", env!("CARGO_PKG_VERSION"))),
        )
        .child(
          div()
            .absolute()
            .right_0()
            .top_0()
            .h_full()
            .w(px(8.0))
            .child(
              resize_handle("sidebar-resize", Axis::Horizontal)
                .with_appearance(std::rc::Rc::new(|_, _, _| None))
                .inside(HandleEdge::Trailing)
                .on_drag(SidebarResize, |drag, _, _, cx| {
                  cx.stop_propagation();
                  cx.new(|_| drag.clone())
                })
                .on_drag_move(cx.listener(
                  |this, event: &DragMoveEvent<SidebarResize>, window, cx| {
                    let max = (f32::from(window.viewport_size().width) - 360.0).clamp(176.0, 300.0);
                    this.sidebar_width =
                      (f32::from(event.event.position.x) - 4.0).clamp(176.0, max);
                    cx.notify();
                  },
                ))
                .on_double_click(cx.listener(|this, _, _, cx| {
                  this.sidebar_width = 196.0;
                  cx.notify();
                })),
            ),
        );
    }
    let mut content = v_flex()
      .gap_5()
      .min_w(px(0.0))
      .w_full()
      .max_w(px(840.0))
      .mx_auto();
    let mut count = 0;
    for page in Page::ALL {
      if (searching && !page.matches(&query)) || (!searching && page != self.page) {
        continue;
      }
      count += 1;
      content = content.child(
        v_flex()
          .gap_2()
          .px_4()
          .py_2()
          .child(
            h_flex()
              .gap_3()
              .child(
                Icon::new(page.icon())
                  .size_5()
                  .text_color(Palette::MutedText.color(cx)),
              )
              .child(
                div()
                  .min_w(px(0.0))
                  .text_xl()
                  .font_weight(FontWeight::SEMIBOLD)
                  .child(page.title()),
              ),
          )
          .child(
            div()
              .text_sm()
              .text_color(Palette::MutedText.color(cx))
              .child(page.description()),
          ),
      );
      content = match page {
        Page::Rules => {
          content
            .child(self.render_section(
              ListKind::Blacklist,
              "Blocked keys",
              "Blocked whenever the selected detection method is active.",
              "No blocked keys configured.",
              cx,
            ))
            .child(self.render_section(
              ListKind::Whitelist,
              "Allowed keys",
              "Always allowed, including shortcuts matching a blocked rule.",
              "No allowed keys configured.",
              cx,
            ))
        },
        Page::Detection => content.child(self.detection(cx)),
        Page::Appearance => content.child(self.appearance(cx)),
        Page::Help => content.child(HelpCard),
      };
    }
    if count == 0 {
      content = content.child(
        v_flex()
          .p_5()
          .gap_2()
          .rounded_xl()
          .bg(Palette::Card.color(cx))
          .child(
            div()
              .font_weight(FontWeight::MEDIUM)
              .child("No settings found"),
          )
          .child(
            div()
              .text_sm()
              .text_color(Palette::MutedText.color(cx))
              .child("Try key rules, detection, theme, glass, or help."),
          ),
      );
    }
    let body = h_flex()
      .items_stretch()
      .flex_1()
      .min_h(px(0.0))
      .min_w(px(0.0))
      .gap_1()
      .p_1()
      .when(narrow, |s| s.flex_col())
      .child(sidebar)
      .child(
        v_flex()
          .id("settings-content")
          .min_w(px(0.0))
          .min_h(px(0.0))
          .flex_1()
          .rounded_md()
          .bg(Palette::Panel.color(cx))
          .overflow_y_scrollbar()
          .p(if narrow { px(16.0) } else { px(28.0) })
          .child(content),
      );
    let footer = h_flex()
      .flex_wrap()
      .items_center()
      .gap_3()
      .mx_1()
      .mb_1()
      .px_4()
      .py_3()
      .rounded_md()
      .bg(Palette::Panel.color(cx))
      .child(
        div()
          .flex_1()
          .min_w(px(100.0))
          .text_xs()
          .text_color(Palette::MutedText.color(cx))
          .child("Save to apply your changes."),
      )
      .child(PillButton::secondary(
        "cancel-button",
        "Cancel",
        cx.listener(|_, _, _, cx| cx.quit()),
      ))
      .child(PillButton::primary(
        "save-button",
        "Save changes",
        cx.listener(|this, _, _, cx| {
          this.config.save();
          cx.quit();
        }),
      ));
    v_flex()
      .size_full()
      .bg(theme::window_surface(cx))
      .text_color(Palette::Text.color(cx))
      .text_size(px(13.0))
      .child(
        TitleBar::new()
          .border_0()
          .bg(Palette::TitleBar.color(cx))
          .text_color(Palette::Text.color(cx))
          .child(
            div()
              .h_full()
              .flex()
              .items_center()
              .text_size(px(12.0))
              .child("WinKeyLock"),
          ),
      )
      .child(body)
      .when_some(self.error_message.as_ref(), |s, err| {
        s.child(
          div()
            .mx_1()
            .mb_1()
            .p_3()
            .rounded_md()
            .bg(Palette::ErrorBackground.color(cx))
            .text_color(Palette::ErrorText.color(cx))
            .text_xs()
            .child(err.clone()),
        )
      })
      .child(footer)
  }
}

pub fn run(config: KeyConfig) -> Result<(), Box<dyn std::error::Error>> {
  gpui_kit::application().run(move |cx: &mut App| {
    gpui_kit::init(cx);
    theme::init(cx);
    theme::init_glass(config.frosted_glass, cx);
    theme::apply(config.theme.as_deref().unwrap_or(theme::DEFAULT_THEME), cx);
    cx.on_window_closed(|cx, _| {
      if cx.windows().is_empty() {
        cx.quit();
      }
    })
    .detach();
    let bounds = Bounds::centered(None, size(px(880.0), px(700.0)), cx);
    let mut options = TitleBar::window_options();
    options.titlebar.as_mut().unwrap().title = Some(CONFIG_WINDOW_TITLE.into());
    options.window_bounds = Some(WindowBounds::Windowed(bounds));
    options.window_min_size = Some(size(px(460.0), px(540.0)));
    options.window_background = theme::background_appearance(cx);
    if let Err(error) = cx.open_window(options, |window, cx| {
      let view = cx.new(|cx| GpuiKitConfigUI::new(config.clone(), window, cx));
      cx.new(|cx| gpui_kit::base::Root::new(view, window, cx).bg(transparent_black()))
    }) {
      tracing::error!(%error, "Could not open configuration window");
    }
    cx.activate(true);
  });
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::Page;

  #[test]
  fn search_finds_sections_by_settings_and_requires_all_words() {
    assert!(Page::Rules.matches("whitelist"));
    assert!(Page::Detection.matches("fullscreen"));
    assert!(Page::Appearance.matches("frosted glass"));
    assert!(Page::Help.matches("modifiers"));
    assert!(!Page::Appearance.matches("frosted blacklist"));
    assert!(!Page::Detection.matches("theme"));
  }
}
