use crate::messages::Message;
use iced::{
    Event, Subscription, Task, Theme,
    event,
    keyboard::{self, key::Named, Key},
    widget::{markdown, text_editor},
};
use std::path::PathBuf;

pub struct App {
    pub files: Vec<OpenFile>,
    pub active: usize,
    pub sidebar_visible: bool,
    pub view_mode: ViewMode,
    pub font_size: f32,
    pub theme: Theme,
    pub fullscreen: bool,
}

pub struct OpenFile {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub content: String,
    pub items: Vec<markdown::Item>,
    pub editor_content: text_editor::Content,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum ViewMode {
    #[default]
    Rendered,
    Source,
}

impl Default for App {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            active: 0,
            sidebar_visible: true,
            view_mode: ViewMode::default(),
            font_size: 16.0,
            theme: Theme::Dark,
            fullscreen: false,
        }
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::Nightfly,
                ..Self::default()
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        match self.files.get(self.active) {
            Some(file) => {
                let name = file
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Untitled");
                format!("{name} — Readfence")
            }
            None => "Readfence".into(),
        }
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|ev, _status, _window| match ev {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                let ctrl = modifiers.control();
                match key.as_ref() {
                    Key::Character("o") if ctrl => Some(Message::OpenDialog),
                    Key::Character("b") if ctrl => Some(Message::ToggleSidebar),
                    Key::Character("=") if ctrl => Some(Message::IncreaseFontSize),
                    Key::Character("+") if ctrl => Some(Message::IncreaseFontSize),
                    Key::Character("-") if ctrl => Some(Message::DecreaseFontSize),
                    Key::Named(Named::F11) => Some(Message::ToggleFullscreen),
                    _ => None,
                }
            }
            _ => None,
        })
    }
}