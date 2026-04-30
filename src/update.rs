use crate::app::{App, OpenFile, ViewMode};
use crate::messages::Message;
use crate::files::load_files;
use iced::{Task, clipboard, window};
use iced::widget::{markdown, text_editor};

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenDialog => Task::perform(load_files(), Message::FilesLoaded),

            Message::FilesLoaded(files) => {
                let before = self.files.len();
                for (path, content) in files {
                    if self.files.iter().any(|f| f.path == path) {
                        continue;
                    }
                    let items = markdown::parse(&content).collect();
                    let editor_content = text_editor::Content::with_text(&content);
                    self.files.push(OpenFile { path, content, items, editor_content });
                }
                if self.files.len() > before {
                    self.active = self.files.len() - 1;
                }
                Task::none()
            }

            Message::SelectFile(i) => {
                self.active = i;
                Task::none()
            }

            Message::CloseFile(i) => {
                self.files.remove(i);
                if !self.files.is_empty() {
                    self.active = self.active.min(self.files.len() - 1);
                }
                Task::none()
            }

            Message::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
                Task::none()
            }

            Message::ToggleViewMode => {
                self.view_mode = match self.view_mode {
                    ViewMode::Rendered => ViewMode::Source,
                    ViewMode::Source => ViewMode::Rendered,
                };
                Task::none()
            }

            Message::IncreaseFontSize => {
                self.font_size = (self.font_size + 2.0).min(36.0);
                Task::none()
            }

            Message::DecreaseFontSize => {
                self.font_size = (self.font_size - 2.0).max(10.0);
                Task::none()
            }

            Message::ThemeChanged(theme) => {
                self.theme = theme;
                Task::none()
            }

            Message::ToggleFullscreen => {
                self.fullscreen = !self.fullscreen;
                let mode = if self.fullscreen {
                    window::Mode::Fullscreen
                } else {
                    window::Mode::Windowed
                };
                window::latest().and_then(move |id| window::set_mode(id, mode))
            }

            Message::LinkClicked(url) => {
                let _ = open::that_detached(&url);
                Task::none()
            }

            Message::EditorAction(action) => {
                // Allow cursor movement and selection; silently drop text edits (read-only).
                if !matches!(action, text_editor::Action::Edit(_)) {
                    if let Some(file) = self.files.get_mut(self.active) {
                        file.editor_content.perform(action);
                    }
                }
                Task::none()
            }

            Message::CopyCode(code) => clipboard::write(code),
        }
    }
}