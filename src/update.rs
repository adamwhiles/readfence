use crate::app::{App, OpenFile, RemoteImage, ViewMode, looks_like_svg, svg_dimensions};
use crate::files::{load_files, load_paths};
use crate::markdown_text::{ImageSource, RenderedBlockKind, rendered_blocks, selectable_text};
use crate::messages::Message;
use iced::widget::{image, svg, text_editor};
use iced::{Point, Task, clipboard, window};
use std::path::Path;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenDialog => Task::perform(load_files(), Message::FilesLoaded),

            Message::FileDropped(path) => {
                Task::perform(load_paths(vec![path]), Message::FilesLoaded)
            }

            Message::FilesLoaded(files) => {
                let before = self.files.len();
                for (path, content, mtime) in files {
                    if self.files.iter().any(|f| f.path == path) {
                        continue;
                    }
                    let editor_content = text_editor::Content::with_text(&content);
                    let base_dir = path.parent().unwrap_or(Path::new("")).to_path_buf();
                    let rendered_blocks = rendered_blocks(&content, &base_dir);
                    let rendered_text = selectable_text(&rendered_blocks);
                    self.files.push(OpenFile {
                        path,
                        content,
                        editor_content,
                        rendered_text,
                        rendered_blocks,
                        last_modified: mtime,
                    });
                }
                if self.files.len() > before {
                    self.active = self.files.len() - 1;
                }
                self.fetch_remote_images()
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
                App::save_theme(&theme);
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

            Message::EditorAction(action) => {
                // Allow cursor movement and selection; silently drop text edits (read-only).
                if !matches!(action, text_editor::Action::Edit(_))
                    && let Some(file) = self.files.get_mut(self.active)
                {
                    file.editor_content.perform(action);
                }
                Task::none()
            }

            Message::RenderedBlockAction(index, action) => {
                // Allow cursor movement and selection; silently drop text edits (read-only).
                if !matches!(action, text_editor::Action::Edit(_))
                    && let Some(file) = self.files.get_mut(self.active)
                    && let Some(block) = file.rendered_blocks.get_mut(index)
                {
                    block.content.perform(action);
                }
                Task::none()
            }

            Message::RenderedCrossBlockSelection {
                anchor,
                target,
                point,
            } => {
                if let Some(file) = self.files.get_mut(self.active) {
                    let start = anchor.min(target);
                    let end = anchor.max(target);
                    let forward = target > anchor;
                    let far_edge = Point::new(f32::MAX / 4.0, f32::MAX / 4.0);

                    for (index, block) in file.rendered_blocks.iter_mut().enumerate() {
                        if index < start || index > end {
                            block
                                .content
                                .perform(text_editor::Action::Click(Point::ORIGIN));
                        } else if index == anchor {
                            block.content.perform(text_editor::Action::Drag(if forward {
                                far_edge
                            } else {
                                Point::ORIGIN
                            }));
                        } else if index == target {
                            block
                                .content
                                .perform(text_editor::Action::Click(if forward {
                                    Point::ORIGIN
                                } else {
                                    far_edge
                                }));
                            block.content.perform(text_editor::Action::Drag(point));
                        } else {
                            block.content.perform(text_editor::Action::SelectAll);
                        }
                    }
                }
                Task::none()
            }

            Message::RenderedBlockClicked(index) => {
                let url = self
                    .files
                    .get(self.active)
                    .and_then(|file| file.rendered_blocks.get(index))
                    .and_then(|block| block.link_at_cursor())
                    .map(ToOwned::to_owned);

                if let Some(url) = url {
                    let _ = open::that_detached(url);
                }
                Task::none()
            }

            Message::CopyRenderedSelection => {
                let selected = self.files.get(self.active).map(|file| {
                    file.rendered_blocks
                        .iter()
                        .filter_map(|block| block.content.selection())
                        .collect::<Vec<_>>()
                        .join("\n\n")
                });

                match selected {
                    Some(text) if !text.is_empty() => clipboard::write(text),
                    _ => Task::none(),
                }
            }

            Message::SelectAllRendered => {
                if let Some(file) = self.files.get_mut(self.active) {
                    for block in &mut file.rendered_blocks {
                        block.content.perform(text_editor::Action::SelectAll);
                    }
                }
                Task::none()
            }

            Message::CopyCode(code) => clipboard::write(code),

            Message::CopyRenderedText(text) => clipboard::write(text),

            Message::WatchTick => {
                let tasks: Vec<Task<Message>> = self
                    .files
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
                                Some((i, mtime)) if last.is_some_and(|l| mtime > l) => {
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
                    let base_dir = file.path.parent().unwrap_or(Path::new("")).to_path_buf();
                    file.editor_content = text_editor::Content::with_text(&content);
                    file.rendered_blocks = rendered_blocks(&content, &base_dir);
                    file.rendered_text = selectable_text(&file.rendered_blocks);
                    file.last_modified = Some(mtime);
                    file.content = content;
                }
                self.fetch_remote_images()
            }

            Message::RemoteImageLoaded(url, bytes) => {
                let loaded = match bytes {
                    Some(bytes) if looks_like_svg(&bytes) => {
                        match svg_dimensions(&bytes) {
                            Some((width, height)) => RemoteImage::Vector {
                                handle: svg::Handle::from_memory(bytes),
                                width,
                                height,
                            },
                            None => RemoteImage::Failed,
                        }
                    }
                    Some(bytes) => RemoteImage::Raster(image::Handle::from_bytes(bytes)),
                    None => RemoteImage::Failed,
                };
                self.remote_images.insert(url, loaded);
                Task::none()
            }

            Message::WindowResized(width) => {
                self.window_width = width;
                Task::none()
            }

            Message::UpdateCheckTick => Task::perform(
                crate::updates::check_for_update(),
                Message::UpdateCheckCompleted,
            ),

            Message::UpdateCheckCompleted(info) => {
                // A failed re-check keeps any notice already showing, and a
                // dismissed release stays dismissed.
                if let Some(info) = info
                    && self.dismissed_update.as_deref() != Some(info.version.as_str())
                {
                    self.update_notice = Some(info);
                }
                Task::none()
            }

            Message::OpenUpdatePage => {
                if let Some(notice) = &self.update_notice {
                    let _ = open::that_detached(&notice.url);
                }
                Task::none()
            }

            Message::DismissUpdate => {
                self.dismissed_update = self.update_notice.take().map(|notice| notice.version);
                Task::none()
            }

            Message::NoOp => Task::none(),
        }
    }

    /// Starts a download for every remote image referenced by an open file
    /// that has not been requested yet.
    fn fetch_remote_images(&mut self) -> Task<Message> {
        let urls: Vec<String> = self
            .files
            .iter()
            .flat_map(|file| file.rendered_blocks.iter())
            .filter_map(|block| match &block.kind {
                RenderedBlockKind::Image {
                    source: ImageSource::Remote(url),
                    ..
                } => Some(url.clone()),
                _ => None,
            })
            .collect();

        let mut tasks = Vec::new();
        for url in urls {
            if self.remote_images.contains_key(&url) {
                continue;
            }
            self.remote_images.insert(url.clone(), RemoteImage::Loading);
            let fetch_url = url.clone();
            tasks.push(Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || fetch_image_bytes(&fetch_url))
                        .await
                        .ok()
                        .flatten()
                },
                move |bytes| Message::RemoteImageLoaded(url.clone(), bytes),
            ));
        }
        Task::batch(tasks)
    }
}

fn fetch_image_bytes(url: &str) -> Option<Vec<u8>> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(20))
        .call()
        .ok()?;
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(
        &mut std::io::Read::take(response.into_reader(), 32 * 1024 * 1024),
        &mut bytes,
    )
    .ok()?;
    Some(bytes)
}
