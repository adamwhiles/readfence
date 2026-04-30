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
                for (path, content, mtime) in files {
                    if self.files.iter().any(|f| f.path == path) {
                        continue;
                    }
                    let items = markdown::parse(&content).collect();
                    let editor_content = text_editor::Content::with_text(&content);
                    self.files.push(OpenFile { path, content, items, editor_content, last_modified: mtime });
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

            Message::WatchTick => {
                let tasks: Vec<Task<Message>> = self.files
                    .iter()
                    .enumerate()
                    .map(|(i, file)| {
                        let path = file.path.clone();
                        let last = file.last_modified;
                        Task::perform(
                            async move {
                                let meta = tokio::fs::metadata(&path).await.ok()?;
                                let mtime = meta.modified().ok()?;
                                Some((i, mtime))
                            },
                            move |result| match result {
                                Some((i, mtime)) if last.map_or(false, |l| mtime > l) => {
                                    Message::FileChanged(i, mtime)
                                }
                                _ => Message::NoOp,
                            },
                        )
                    })
                    .collect();
                Task::batch(tasks)
            }

            Message::FileChanged(i, mtime) => {
                let path = match self.files.get(i) {
                    Some(f) => f.path.clone(),
                    None => return Task::none(),
                };
                Task::perform(
                    async move {
                        let content = tokio::fs::read_to_string(&path).await.ok()?;
                        Some((i, content, mtime))
                    },
                    |result| match result {
                        Some((i, content, mtime)) => Message::FileReloaded(i, content, mtime),
                        None => Message::NoOp,
                    },
                )
            }

            Message::FileReloaded(i, content, mtime) => {
                if let Some(file) = self.files.get_mut(i) {
                    file.items = markdown::parse(&content).collect();
                    file.editor_content = text_editor::Content::with_text(&content);
                    file.last_modified = Some(mtime);
                    file.content = content;
                }
                Task::none()
            }

            Message::NoOp => Task::none(),
        }
    }
}