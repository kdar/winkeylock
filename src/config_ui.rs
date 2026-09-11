use crate::config::{DetectMethod, KeyCombo, KeyConfig};
use iced::{
  Alignment, Element, Length, Task, Theme,
  widget::{Space, button, column, container, pick_list, row, scrollable, text, text_input},
};
use std::process;

#[derive(Debug, Clone)]
pub enum Message {
  AddBlacklistKey,
  AddWhitelistKey,
  RemoveBlacklistKey(usize),
  RemoveWhitelistKey(usize),
  BlacklistKeyInput(String),
  WhitelistKeyInput(String),
  DetectMethodChanged(DetectMethod),
  ToggleHelp,
  Save,
  Cancel,
  Close,
}

#[derive(Debug)]
pub struct ConfigUI {
  config: KeyConfig,
  blacklist_input: String,
  whitelist_input: String,
  error_message: Option<String>,
  show_help: bool,
}

impl ConfigUI {
  pub fn new(config: KeyConfig) -> (Self, Task<Message>) {
    (
      ConfigUI {
        config,
        blacklist_input: String::new(),
        whitelist_input: String::new(),
        error_message: None,
        show_help: false,
      },
      Task::none(),
    )
  }

  pub fn title(&self) -> String {
    "WinKeyLock Configuration".to_string()
  }

  pub fn theme(&self) -> Theme {
    Theme::Light
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    self.error_message = None; // Clear error on each update

    match message {
      Message::AddBlacklistKey => {
        if !self.blacklist_input.trim().is_empty() {
          match KeyCombo::from_string(&self.blacklist_input.trim()) {
            Ok(key_combo) => {
              self.config.blacklist.push(key_combo);
              self.blacklist_input.clear();
            },
            Err(e) => {
              self.error_message = Some(format!("Invalid key combination: {}", e));
            },
          }
        }
      },
      Message::AddWhitelistKey => {
        if !self.whitelist_input.trim().is_empty() {
          match KeyCombo::from_string(&self.whitelist_input.trim()) {
            Ok(key_combo) => {
              self.config.whitelist.push(key_combo);
              self.whitelist_input.clear();
            },
            Err(e) => {
              self.error_message = Some(format!("Invalid key combination: {}", e));
            },
          }
        }
      },
      Message::RemoveBlacklistKey(index) => {
        if index < self.config.blacklist.len() {
          self.config.blacklist.remove(index);
        }
      },
      Message::RemoveWhitelistKey(index) => {
        if index < self.config.whitelist.len() {
          self.config.whitelist.remove(index);
        }
      },
      Message::BlacklistKeyInput(input) => {
        self.blacklist_input = input;
      },
      Message::WhitelistKeyInput(input) => {
        self.whitelist_input = input;
      },
      Message::DetectMethodChanged(method) => {
        self.config.detect_method = method;
      },
      Message::ToggleHelp => {
        self.show_help = !self.show_help;
      },
      Message::Save => {
        self.config.save();
        return Task::perform(async {}, |_| Message::Close);
      },
      Message::Cancel => {
        return Task::perform(async {}, |_| Message::Close);
      },
      Message::Close => {
        process::exit(0);
      },
    }

    Task::none()
  }

  pub fn view(&self) -> Element<'_, Message> {
    let description = text(
      "Configure which keys should be blocked or allowed. \
            Examples: 'lwin', 'ctrl+c', 'shift+alt+tab'",
    )
    .size(14)
    .width(Length::Fill);

    let help_button_label = if self.show_help {
      "Hide Help"
    } else {
      "Show Help"
    };

    let help_button = button(help_button_label).on_press(Message::ToggleHelp);

    let header = row![description, help_button]
      .spacing(10)
      .align_y(Alignment::Center);

    let help_content: Element<Message> = if self.show_help {
      container(
        column![
          text("How to enter key combinations").size(16),
          text("Use '+' between each modifier and key, e.g. 'ctrl+shift+esc'.").size(13),
          text("Supported modifiers: 'ctrl', 'alt', 'shift', 'lwin', 'rwin'.").size(13),
          text("Finish with exactly one key name like 'c', 'f12', 'space', 'delete', etc.")
            .size(13),
          text("Names are case-insensitive; spaces around '+' are optional.").size(13),
          text("Only one non-modifier key is allowed per combination.").size(13),
        ]
        .spacing(6),
      )
      .width(Length::Fill)
      .padding(12)
      .into()
    } else {
      Space::new().into()
    };

    // Blacklist section
    let blacklist_title = text("Blocked Keys (Blacklist)").size(18);
    let blacklist_description = text("Keys that will be blocked when active").size(12);

    let blacklist_items: Element<Message> = if self.config.blacklist.is_empty() {
      text("No blocked keys configured").into()
    } else {
      column(
        self
          .config
          .blacklist
          .iter()
          .enumerate()
          .map(|(i, key)| {
            row![
              text(&key.string_repr).width(Length::Fill),
              button("Remove")
                .on_press(Message::RemoveBlacklistKey(i))
                .style(button::danger)
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
          })
          .collect::<Vec<_>>(),
      )
      .spacing(5)
      .into()
    };

    let blacklist_input_row = row![
      text_input(
        "Enter key combination (e.g., 'lwin', 'ctrl+alt+del')",
        &self.blacklist_input
      )
      .on_input(Message::BlacklistKeyInput)
      .on_submit(Message::AddBlacklistKey)
      .width(Length::Fill),
      button("Add")
        .on_press(Message::AddBlacklistKey)
        .style(button::success)
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    // Whitelist section
    let whitelist_title = text("Allowed Keys (Whitelist)").size(18);
    let whitelist_description =
      text("Keys that will be allowed even if they match blacklist rules").size(12);

    let whitelist_items: Element<Message> = if self.config.whitelist.is_empty() {
      text("No allowed keys configured").into()
    } else {
      column(
        self
          .config
          .whitelist
          .iter()
          .enumerate()
          .map(|(i, key)| {
            row![
              text(&key.string_repr).width(Length::Fill),
              button("Remove")
                .on_press(Message::RemoveWhitelistKey(i))
                .style(button::danger)
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
          })
          .collect::<Vec<_>>(),
      )
      .spacing(5)
      .into()
    };

    let whitelist_input_row = row![
      text_input("Enter key combination", &self.whitelist_input)
        .on_input(Message::WhitelistKeyInput)
        .on_submit(Message::AddWhitelistKey)
        .width(Length::Fill),
      button("Add")
        .on_press(Message::AddWhitelistKey)
        .style(button::success)
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    // Detect method section
    let detect_method_title = text("Detection Method").size(18);
    let detect_method_description = text("How to detect if the focused window is a game").size(12);

    let detect_method_picker = pick_list(
      DetectMethod::ALL.as_slice(),
      Some(self.config.detect_method),
      Message::DetectMethodChanged,
    )
    .width(Length::Fill);

    // Error message
    let error_section: Element<Message> = if let Some(ref error) = self.error_message {
      text(error)
        .style(text::danger)
        .into()
    } else {
      Space::new().into()
    };

    // Action buttons
    let action_buttons = row![
      button("Save")
        .on_press(Message::Save)
        .style(button::primary),
      button("Cancel").on_press(Message::Cancel),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let content = column![
      header,
      help_content,
      Space::new().height(20.0),
      blacklist_title,
      blacklist_description,
      Space::new().height(10.0),
      blacklist_items,
      blacklist_input_row,
      Space::new().height(20.0),
      whitelist_title,
      whitelist_description,
      Space::new().height(10.0),
      whitelist_items,
      whitelist_input_row,
      Space::new().height(20.0),
      detect_method_title,
      detect_method_description,
      Space::new().height(10.0),
      detect_method_picker,
      Space::new().height(20.0),
      error_section,
      action_buttons,
    ]
    .spacing(10)
    .padding(20)
    .max_width(440);

    // Wrap the entire content in a scrollable container for when the window is too small
    let scrollable_content = scrollable(content).width(Length::Fill).height(Length::Fill);

    container(scrollable_content)
      .width(Length::Fill)
      .height(Length::Fill)
      .center_x(Length::Fill)
      .into()
  }
}

pub fn run_config_ui(config: KeyConfig) -> iced::Result {
  iced::application(
    move || ConfigUI::new(config.clone()),
    ConfigUI::update,
    ConfigUI::view
  )
    .title(ConfigUI::title)
    .window(iced::window::Settings {
      size: (480.0, 600.0).into(),
      min_size: Some((400.0, 500.0).into()),
      resizable: true,
      decorations: true,
      ..Default::default()
    })
    .theme(ConfigUI::theme)
    .run()
}

/// Helper function to launch the config UI in a separate process
pub fn launch_config_ui() -> Result<(), Box<dyn std::error::Error>> {
  // Launch the config UI in a separate process to avoid event loop conflicts
  let exe_path = std::env::current_exe()?;

  std::process::Command::new(&exe_path)
    .arg("--config-ui")
    .spawn()?;

  Ok(())
}

/// Function to actually run the config UI (called when launched with --config-ui)
pub fn run_config_ui_main() -> Result<(), iced::Error> {
  let config = crate::config::KeyConfig::load();
  run_config_ui(config)
}
