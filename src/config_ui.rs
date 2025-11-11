use crate::config::{KeyCombo, KeyConfig};
use iced::{
  Alignment, Application, Command, Element, Length, Settings, Theme,
  widget::{Space, button, column, container, row, scrollable, text, text_input},
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

impl Application for ConfigUI {
  type Executor = iced::executor::Default;
  type Flags = KeyConfig;
  type Message = Message;
  type Theme = Theme;

  fn new(config: KeyConfig) -> (Self, Command<Message>) {
    (
      ConfigUI {
        config,
        blacklist_input: String::new(),
        whitelist_input: String::new(),
        error_message: None,
        show_help: false,
      },
      Command::none(),
    )
  }

  fn title(&self) -> String {
    "WinKeyLock Configuration".to_string()
  }

  fn update(&mut self, message: Message) -> Command<Message> {
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
      Message::ToggleHelp => {
        self.show_help = !self.show_help;
      },
      Message::Save => {
        self.config.save();
        return Command::perform(async {}, |_| Message::Close);
      },
      Message::Cancel => {
        return Command::perform(async {}, |_| Message::Close);
      },
      Message::Close => {
        process::exit(0);
      },
    }

    Command::none()
  }

  fn view(&self) -> Element<'_, Message> {
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
      .align_items(Alignment::Center);

    let help_content: Element<Message> = if self.show_help {
      container(
        column![
          text("How to enter key combinations").size(16),
          text("Use '+' between each modifier and key, e.g. 'ctrl+shift+esc'.").size(13),
          text("Supported modifiers: 'ctrl', 'alt', 'shift', 'lwin', 'rwin'.").size(13),
          text("Finish with exactly one key name like 'c', 'f12', 'space', 'delete', etc.").size(13),
          text("Names are case-insensitive; spaces around '+' are optional.").size(13),
          text("Only one non-modifier key is allowed per combination.").size(13),
        ]
        .spacing(6),
      )
      .width(Length::Fill)
      .padding(12)
      .into()
    } else {
      Space::with_height(Length::Fixed(0.0)).into()
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
                .style(iced::theme::Button::Destructive)
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .into()
          })
          .collect(),
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
        .style(iced::theme::Button::Positive)
    ]
    .spacing(10)
    .align_items(Alignment::Center);

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
                .style(iced::theme::Button::Destructive)
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .into()
          })
          .collect(),
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
        .style(iced::theme::Button::Positive)
    ]
    .spacing(10)
    .align_items(Alignment::Center);

    // Error message
    let error_section: Element<Message> = if let Some(ref error) = self.error_message {
      text(error)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(
          0.8, 0.2, 0.2,
        )))
        .into()
    } else {
      Space::with_height(Length::Fixed(0.0)).into()
    };

    // Action buttons
    let action_buttons = row![
      button("Save")
        .on_press(Message::Save)
        .style(iced::theme::Button::Primary),
      button("Cancel").on_press(Message::Cancel),
    ]
    .spacing(10)
    .align_items(Alignment::Center);

    let content = column![
      header,
      help_content,
      Space::with_height(Length::Fixed(20.0)),
      blacklist_title,
      blacklist_description,
      Space::with_height(Length::Fixed(10.0)),
      blacklist_items,
      blacklist_input_row,
      Space::with_height(Length::Fixed(20.0)),
      whitelist_title,
      whitelist_description,
      Space::with_height(Length::Fixed(10.0)),
      whitelist_items,
      whitelist_input_row,
      Space::with_height(Length::Fixed(20.0)),
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
      .center_x()
      .into()
  }

  fn theme(&self) -> Theme {
    Theme::Light
  }
}

pub fn run_config_ui(config: KeyConfig) -> Result<(), iced::Error> {
  ConfigUI::run(Settings {
    window: iced::window::Settings {
      size: (480, 600),           // Smaller width: 480px instead of default ~650px
      min_size: Some((400, 500)), // Minimum size to prevent it from being too small
      resizable: true,
      decorations: true,
      ..Default::default()
    },
    flags: config,
    ..Default::default()
  })
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
