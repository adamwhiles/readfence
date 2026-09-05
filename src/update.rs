use crate::app::{
    App, DEFAULT_FONT_SIZE, MAX_FONT_SIZE, MIN_FONT_SIZE, OpenFile, RemoteImage, ViewMode,
    looks_like_svg, svg_dimensions,
};
use crate::files::{load_files, load_paths};
use crate::find::{self, FindMatch, line_column};
use crate::markdown_text::{
    ImageSource, RenderedBlockKind, copy_gap, rendered_blocks, selectable_text,
};
use crate::messages::Message;
use crate::selection_group::{DOCUMENT_SCROLL_ID, LocateBlock};
use crate::updates::{InstallState, UpdateCheckOutcome, UpdateStatus, install_update};
use crate::view::{FIND_INPUT_ID, SOURCE_EDITOR_ID};
use iced::advanced::text::editor::{Cursor, Position};
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{Id, Operation};
use iced::widget::{image, operation, operation::AbsoluteOffset, svg, text_editor};
use iced::{Point, Rectangle, Task, clipboard, window};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// How long the "Copied" note stays in the status bar.
const COPY_NOTICE_DURATION: Duration = Duration::from_millis(1800);

/// Space left above a block that was jumped to, so it does not sit hard
/// against the top edge.
const JUMP_MARGIN: f32 = 24.0;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenDialog => Task::perform(load_files(), Message::FilesLoaded),

            Message::FileDropped(path) => {
                Task::perform(load_paths(vec![path]), Message::FilesLoaded)
            }

            Message::FilesLoaded(files) => {
                let before = self.files.len();
                self.add_files(files);
                if self.files.len() > before {
                    self.active = self.files.len() - 1;
                }
                self.document_changed()
            }

            Message::SessionRestored(files, active) => {
                self.add_files(files);
                self.active = active
                    .and_then(|path| self.files.iter().position(|file| file.path == path))
                    .unwrap_or(0);
                self.document_changed()
            }

            Message::SelectFile(i) => {
                if i < self.files.len() {
                    self.active = i;
                }
                self.document_changed()
            }

            Message::CloseFile(i) => {
                if i < self.files.len() {
                    self.files.remove(i);
                }
                if !self.files.is_empty() {
                    self.active = self.active.min(self.files.len() - 1);
                }
                self.document_changed()
            }

            Message::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
                self.persist_session();
                Task::none()
            }

            Message::ToggleViewMode => {
                self.view_mode = match self.view_mode {
                    ViewMode::Rendered => ViewMode::Source,
                    ViewMode::Source => ViewMode::Rendered,
                };
                self.refresh_find();
                self.persist_session();
                self.jump_to_current_match()
            }

            Message::IncreaseFontSize => self.set_font_size(self.font_size + 2.0),

            Message::DecreaseFontSize => self.set_font_size(self.font_size - 2.0),

            Message::Zoom(steps) => self.set_font_size(self.font_size + steps as f32),

            Message::ResetFontSize => self.set_font_size(DEFAULT_FONT_SIZE),

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

            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
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

            Message::RenderedBlockPressed(index) => {
                self.clear_rendered_selections(Some(index));
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

            Message::OpenLink(url) => {
                let _ = open::that_detached(url);
                Task::none()
            }

            Message::CopyRenderedSelection => {
                let text = self.rendered_selection_text();
                self.copy_text(text)
            }

            Message::CopyShortcut => {
                let text = match self.view_mode {
                    ViewMode::Rendered => self.rendered_selection_text(),
                    ViewMode::Source => self
                        .files
                        .get(self.active)
                        .and_then(|file| file.editor_content.selection())
                        .unwrap_or_default(),
                };
                self.copy_text(text)
            }

            Message::SelectAllRendered => self.select_all_rendered(),

            Message::SelectAllShortcut => match self.view_mode {
                ViewMode::Rendered => self.select_all_rendered(),
                ViewMode::Source => {
                    if let Some(file) = self.files.get_mut(self.active) {
                        file.editor_content.perform(text_editor::Action::SelectAll);
                    }
                    // Editors only paint their selection while focused.
                    operation::focus(SOURCE_EDITOR_ID)
                }
            },

            Message::CopyCode(code) => self.copy_text(code),

            Message::CopyRenderedText(text) => self.copy_text(text),

            Message::ClearCopyNotice(serial) => {
                if serial == self.copy_notice_serial {
                    self.copy_notice = None;
                }
                Task::none()
            }

            Message::JumpToBlock(index) => scroll_to_block(index),

            Message::OpenFind => {
                self.find.open = true;
                self.refresh_find();
                Task::batch([
                    operation::focus(FIND_INPUT_ID),
                    operation::select_all(FIND_INPUT_ID),
                ])
            }

            Message::CloseFind => {
                self.close_find();
                Task::none()
            }

            Message::Escape => {
                if self.find.open {
                    self.close_find();
                } else if self.update_menu_open {
                    self.update_menu_open = false;
                }
                Task::none()
            }

            Message::FindQueryChanged(query) => {
                self.find.query = query;
                self.find.current = 0;
                self.refresh_find();
                self.jump_to_current_match()
            }

            Message::FindNext => self.step_find(1),

            Message::FindPrevious => self.step_find(-1),

            Message::FindSubmit => {
                if self.modifiers.shift() {
                    self.step_find(-1)
                } else {
                    self.step_find(1)
                }
            }

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
                if i == self.active {
                    self.refresh_find();
                }
                self.fetch_remote_images()
            }

            Message::RemoteImageLoaded(url, bytes) => {
                let loaded = match bytes {
                    Some(bytes) if looks_like_svg(&bytes) => match svg_dimensions(&bytes) {
                        Some((width, height)) => RemoteImage::Vector {
                            handle: svg::Handle::from_memory(bytes),
                            width,
                            height,
                        },
                        None => RemoteImage::Failed,
                    },
                    Some(bytes) => RemoteImage::Raster(image::Handle::from_bytes(bytes)),
                    None => RemoteImage::Failed,
                };
                self.remote_images.insert(url, loaded);
                Task::none()
            }

            Message::WindowResized(width, height) => {
                self.window_width = width;
                self.window_height = height;
                self.window_size = Some((width, height));
                Task::none()
            }

            Message::CloseRequested => {
                // The window size is only worth writing once, on the way out.
                self.persist_session();
                window::latest().and_then(window::close)
            }

            Message::UpdateCheckTick | Message::CheckForUpdates => {
                self.update_status = UpdateStatus::Checking;
                Task::perform(
                    crate::updates::check_for_updates(),
                    Message::UpdateCheckCompleted,
                )
            }

            Message::UpdateCheckCompleted(outcome) => {
                match outcome {
                    UpdateCheckOutcome::Available(info) => {
                        self.update_status = UpdateStatus::Available;
                        self.update_notice = Some(info);
                    }
                    UpdateCheckOutcome::UpToDate => {
                        self.update_status = UpdateStatus::UpToDate;
                        self.update_notice = None;
                    }
                    // A failed check keeps any update already known instead
                    // of downgrading it to an error.
                    UpdateCheckOutcome::Failed => {
                        self.update_status = if self.update_notice.is_some() {
                            UpdateStatus::Available
                        } else {
                            UpdateStatus::Failed
                        };
                    }
                    UpdateCheckOutcome::StoreManaged => {
                        self.update_status = UpdateStatus::StoreManaged;
                    }
                }
                Task::none()
            }

            Message::ToggleUpdateMenu => {
                self.update_menu_open = !self.update_menu_open;
                Task::none()
            }

            Message::InstallUpdate => match &self.update_notice {
                Some(notice) if self.install_state != InstallState::Running => {
                    self.install_state = InstallState::Running;
                    Task::perform(install_update(notice.clone()), Message::InstallCompleted)
                }
                _ => Task::none(),
            },

            Message::InstallCompleted(result) => {
                self.install_state = match result {
                    Ok(exe) => InstallState::Done(exe),
                    Err(error) => InstallState::Failed(error),
                };
                Task::none()
            }

            Message::RestartApp => {
                // The executable on disk is already the new release; hand the
                // open documents to a fresh process and bow out.
                let InstallState::Done(exe) = &self.install_state else {
                    return Task::none();
                };
                self.persist_session();
                let mut command = std::process::Command::new(exe);
                for file in &self.files {
                    command.arg(&file.path);
                }
                match command.spawn() {
                    Ok(_) => window::latest().and_then(window::close),
                    Err(error) => {
                        self.install_state =
                            InstallState::Failed(format!("couldn't relaunch: {error}"));
                        Task::none()
                    }
                }
            }

            Message::OpenUpdatePage => {
                if let Some(notice) = &self.update_notice {
                    let _ = open::that_detached(&notice.url);
                }
                self.update_menu_open = false;
                Task::none()
            }

            Message::OpenRepoPage => {
                let _ = open::that_detached(crate::updates::REPO_URL);
                self.update_menu_open = false;
                Task::none()
            }

            Message::DismissUpdate => {
                // Keep the notice so the updates menu can still offer it;
                // only the banner goes quiet for this version.
                self.dismissed_update = self
                    .update_notice
                    .as_ref()
                    .map(|notice| notice.version.clone());
                Task::none()
            }

            Message::NoOp => Task::none(),
        }
    }

    /// Adds documents that are not open yet, keeping their order.
    fn add_files(&mut self, files: Vec<(PathBuf, String, Option<SystemTime>)>) {
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
    }

    /// Everything that follows a change of the open or active documents.
    fn document_changed(&mut self) -> Task<Message> {
        self.refresh_find();
        self.persist_session();
        self.fetch_remote_images()
    }

    fn set_font_size(&mut self, size: f32) -> Task<Message> {
        self.font_size = size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
        self.persist_session();
        Task::none()
    }

    /// Recomputes the find hits for the active document in the current
    /// view, keeping the selected hit in range.
    fn refresh_find(&mut self) {
        let Some(file) = self.files.get(self.active) else {
            self.find.matches.clear();
            self.find.current = 0;
            return;
        };
        if !self.find.open || self.find.query.is_empty() {
            self.find.matches.clear();
            self.find.current = 0;
            return;
        }
        self.find.matches = match self.view_mode {
            ViewMode::Rendered => find::rendered_matches(&file.rendered_blocks, &self.find.query),
            ViewMode::Source => find::source_matches(&file.content, &self.find.query),
        };
        self.find.current = if self.find.matches.is_empty() {
            0
        } else {
            self.find.current.min(self.find.matches.len() - 1)
        };
    }

    fn close_find(&mut self) {
        self.find.open = false;
        self.find.matches.clear();
        self.find.current = 0;
    }

    /// Moves to the next (`1`) or previous (`-1`) hit, wrapping around.
    fn step_find(&mut self, direction: i32) -> Task<Message> {
        let count = self.find.matches.len();
        if count == 0 {
            return Task::none();
        }
        self.find.current = if direction < 0 {
            (self.find.current + count - 1) % count
        } else {
            (self.find.current + 1) % count
        };
        self.jump_to_current_match()
    }

    /// Brings the selected find hit into view: in the rendered view by
    /// scrolling to its block, in the source view by moving the editor's
    /// cursor there, which the editor keeps visible.
    fn jump_to_current_match(&mut self) -> Task<Message> {
        let Some(hit) = self.find.current_match().cloned() else {
            return Task::none();
        };
        let Some(file) = self.files.get_mut(self.active) else {
            return Task::none();
        };

        match self.view_mode {
            ViewMode::Rendered => {
                // Park the cursor on the hit so a following Ctrl+C copies it.
                if hit.cell.is_none()
                    && let Some(block) = file.rendered_blocks.get_mut(hit.block)
                {
                    block.content.move_to(cursor_for(&block.text, &hit));
                }
                self.clear_rendered_selections(Some(hit.block));
                scroll_to_block(hit.block)
            }
            ViewMode::Source => {
                file.editor_content.move_to(cursor_for(&file.content, &hit));
                Task::none()
            }
        }
    }

    /// Drops the selection of every rendered block except `keep`.
    fn clear_rendered_selections(&mut self, keep: Option<usize>) {
        if let Some(file) = self.files.get_mut(self.active) {
            for (index, block) in file.rendered_blocks.iter_mut().enumerate() {
                if Some(index) == keep || block.content.selection().is_none() {
                    continue;
                }
                let position = block.content.cursor().position;
                block.content.move_to(Cursor {
                    position,
                    selection: None,
                });
            }
        }
    }

    /// The selected text across the rendered blocks, joined the way the
    /// document reads: list items on consecutive lines, blank lines
    /// between other blocks.
    fn rendered_selection_text(&self) -> String {
        let Some(file) = self.files.get(self.active) else {
            return String::new();
        };
        let mut text = String::new();
        let mut previous: Option<&RenderedBlockKind> = None;
        for block in &file.rendered_blocks {
            let Some(selected) = block.content.selection() else {
                continue;
            };
            if selected.is_empty() {
                continue;
            }
            if let Some(previous) = previous {
                text.push_str(copy_gap(previous, &block.kind));
            }
            text.push_str(&selected);
            previous = Some(&block.kind);
        }
        text
    }

    fn select_all_rendered(&mut self) -> Task<Message> {
        if let Some(file) = self.files.get_mut(self.active) {
            for block in &mut file.rendered_blocks {
                block.content.perform(text_editor::Action::SelectAll);
            }
        }
        // Editors only paint their selection while focused, so focus every
        // block editor; the find field keeps its own focus state.
        iced::advanced::widget::operate(FocusBlockEditors).discard()
    }

    /// Writes `text` to the clipboard and shows a short note in the status
    /// bar; empty text is ignored.
    fn copy_text(&mut self, text: String) -> Task<Message> {
        if text.is_empty() {
            return Task::none();
        }
        let characters = text.chars().count();
        self.copy_notice = Some(if characters == 1 {
            "Copied 1 character".to_string()
        } else {
            format!("Copied {characters} characters")
        });
        self.copy_notice_serial += 1;
        let serial = self.copy_notice_serial;

        Task::batch([
            clipboard::write(text),
            Task::perform(tokio::time::sleep(COPY_NOTICE_DURATION), move |_| {
                Message::ClearCopyNotice(serial)
            }),
        ])
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

/// An editor cursor spanning a find hit in `text`.
fn cursor_for(text: &str, hit: &FindMatch) -> Cursor {
    let (line, column) = line_column(text, hit.range.start);
    let (end_line, end_column) = line_column(text, hit.range.end);
    Cursor {
        position: Position { line, column },
        selection: Some(Position {
            line: end_line,
            column: end_column,
        }),
    }
}

/// Scrolls the rendered document so the block sits just below the top of
/// the view. The block's position is only known to the widget tree, so it
/// is looked up with an operation and the scroll follows from the result.
fn scroll_to_block(index: usize) -> Task<Message> {
    iced::advanced::widget::operate(LocateBlock::new(index)).then(|top| {
        operation::scroll_to(
            DOCUMENT_SCROLL_ID,
            AbsoluteOffset::<Option<f32>> {
                x: None,
                y: Some((top - JUMP_MARGIN).max(0.0)),
            },
        )
    })
}

/// Focuses every text widget in the rendered view except the find field,
/// so a select-all shows up in every block.
struct FocusBlockEditors;

impl Operation for FocusBlockEditors {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if id != Some(&FIND_INPUT_ID) {
            state.focus();
        }
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
