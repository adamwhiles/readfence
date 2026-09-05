use crate::files::{file_watcher, load_paths};
use crate::find::FindState;
use crate::markdown_text::RenderedBlock;
use crate::messages::Message;
use crate::session::{self, Session};
use crate::updates::{InstallState, UpdateInfo, UpdateStatus, check_for_updates};
use iced::{
    Event, Subscription, Task, Theme, event,
    keyboard::{self, Key, key::Named},
    widget::{image, svg, text_editor},
    window,
};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

pub const DEFAULT_FONT_SIZE: f32 = 16.0;
pub const MIN_FONT_SIZE: f32 = 10.0;
pub const MAX_FONT_SIZE: f32 = 36.0;

pub struct App {
    pub files: Vec<OpenFile>,
    pub active: usize,
    pub sidebar_visible: bool,
    pub view_mode: ViewMode,
    pub font_size: f32,
    pub theme: Theme,
    pub fullscreen: bool,
    pub window_width: f32,
    pub window_height: f32,
    /// The logical window size to remember: the last size the window
    /// reported, or, until it reports one, the size restored from the
    /// previous session.
    pub window_size: Option<(f32, f32)>,
    pub remote_images: HashMap<String, RemoteImage>,
    /// Newest release known to be ahead of this build; feeds the banner and
    /// the updates menu.
    pub update_notice: Option<UpdateInfo>,
    /// Version whose banner the user dismissed; that release stays quiet in
    /// the banner (the menu still shows it), a newer one notifies again.
    pub dismissed_update: Option<String>,
    pub update_status: UpdateStatus,
    pub update_menu_open: bool,
    pub install_state: InstallState,
    /// Keyboard modifiers as last reported, for shortcuts that arrive
    /// through widgets without modifier information (Enter in the find
    /// field).
    pub modifiers: keyboard::Modifiers,
    pub find: FindState,
    /// A short-lived status bar note after something was copied.
    pub copy_notice: Option<String>,
    /// Bumped per copy so a stale clear timer cannot erase a newer notice.
    pub copy_notice_serial: u64,
}

pub enum RemoteImage {
    Loading,
    Raster(image::Handle),
    Vector {
        handle: svg::Handle,
        width: f32,
        height: f32,
    },
    Failed,
}

pub fn looks_like_svg(bytes: &[u8]) -> bool {
    String::from_utf8_lossy(&bytes[..bytes.len().min(512)]).contains("<svg")
}

/// Reads the intrinsic size from an SVG's root tag (`width`/`height`
/// attributes, falling back to the `viewBox`), so vector images can render
/// at their natural size.
pub fn svg_dimensions(bytes: &[u8]) -> Option<(f32, f32)> {
    let content = std::str::from_utf8(bytes).ok()?;
    let start = content.find("<svg")?;
    let end = content[start..].find('>')? + start;
    let tag = &content[start..end];

    if let (Some(width), Some(height)) = (svg_attr(tag, " width"), svg_attr(tag, " height")) {
        return Some((width, height));
    }

    let view_box = svg_attr_value(tag, " viewBox")?;
    let mut parts = view_box
        .split_whitespace()
        .filter_map(|value| value.parse::<f32>().ok())
        .skip(2);
    Some((parts.next()?, parts.next()?))
}

fn svg_attr_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

fn svg_attr(tag: &str, name: &str) -> Option<f32> {
    svg_attr_value(tag, name)?
        .trim()
        .trim_end_matches("px")
        .parse()
        .ok()
}

pub struct OpenFile {
    pub path: PathBuf,
    pub content: String,
    pub editor_content: text_editor::Content,
    pub rendered_text: String,
    pub rendered_blocks: Vec<RenderedBlock>,
    pub last_modified: Option<std::time::SystemTime>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ViewMode {
    #[default]
    Rendered,
    Source,
}

impl Default for App {
    fn default() -> Self {
        let window = window::Settings::default().size;
        Self {
            files: Vec::new(),
            active: 0,
            sidebar_visible: true,
            view_mode: ViewMode::default(),
            font_size: DEFAULT_FONT_SIZE,
            theme: Theme::Dark,
            fullscreen: false,
            window_width: window.width,
            window_height: window.height,
            window_size: None,
            remote_images: HashMap::new(),
            update_notice: None,
            dismissed_update: None,
            update_status: UpdateStatus::Unknown,
            update_menu_open: false,
            install_state: InstallState::Idle,
            modifiers: keyboard::Modifiers::default(),
            find: FindState::default(),
            copy_notice: None,
            copy_notice_serial: 0,
        }
    }
}

impl App {
    /// Themes curated to ones whose palettes hold up in the app: readable
    /// contrast between canvas, panel, and accent in both dark and light.
    pub fn available_themes() -> Vec<Theme> {
        vec![
            Theme::Light,
            Theme::Dark,
            Theme::Nightfly,
            Theme::Moonfly,
            Theme::Dracula,
            Theme::Nord,
            Theme::TokyoNight,
            Theme::TokyoNightStorm,
            Theme::CatppuccinMocha,
            Theme::CatppuccinLatte,
            Theme::GruvboxDark,
            Theme::SolarizedDark,
            Theme::KanagawaWave,
            Theme::Oxocarbon,
            Theme::Ferra,
        ]
    }

    fn theme_config_file() -> Option<PathBuf> {
        Some(dirs::config_dir()?.join("readfence").join("theme"))
    }

    fn load_saved_theme() -> Option<Theme> {
        let name = std::fs::read_to_string(Self::theme_config_file()?).ok()?;
        Self::available_themes()
            .into_iter()
            .find(|theme| theme.to_string() == name.trim())
    }

    pub fn save_theme(theme: &Theme) {
        let Some(path) = Self::theme_config_file() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, theme.to_string());
    }

    pub fn new() -> (Self, Task<Message>) {
        // READFENCE_THEME selects a start-up theme by its display name,
        // e.g. "Tokyo Night" (used by automated screenshots). Otherwise the
        // last chosen theme is restored; the very first launch gets Moonfly.
        let theme = std::env::var("READFENCE_THEME")
            .ok()
            .and_then(|name| {
                Self::available_themes()
                    .into_iter()
                    .find(|theme| theme.to_string().eq_ignore_ascii_case(name.trim()))
            })
            .or_else(Self::load_saved_theme)
            .unwrap_or(Theme::Moonfly);

        // Reading preferences carry over from the last session.
        let saved = session::load().unwrap_or_default();
        let app = Self {
            theme,
            // A check starts right below; the menu reports it truthfully.
            update_status: UpdateStatus::Checking,
            font_size: saved
                .font_size
                .map(|size| size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE))
                .unwrap_or(DEFAULT_FONT_SIZE),
            sidebar_visible: saved.sidebar_visible.unwrap_or(true),
            view_mode: saved.view_mode.unwrap_or_default(),
            window_size: saved.window,
            ..Self::default()
        };

        // Open any files passed on the command line, e.g. from a file
        // manager's "Open with" action or a `.desktop` MimeType association.
        // Unsupported paths are filtered out by `load_paths`. Without
        // arguments, the documents from the last session come back instead.
        let paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
        let load_task = if !paths.is_empty() {
            Task::perform(load_paths(paths), Message::FilesLoaded)
        } else if !saved.files.is_empty() {
            let active = saved.active.clone();
            Task::perform(load_paths(saved.files), move |files| {
                Message::SessionRestored(files, active)
            })
        } else {
            Task::none()
        };
        let update_task = Task::perform(check_for_updates(), Message::UpdateCheckCompleted);

        (app, Task::batch([load_task, update_task]))
    }

    /// Writes the current documents, preferences, and window size so the
    /// next launch can pick up where this one left off.
    pub fn persist_session(&self) {
        session::save(&Session {
            files: self.files.iter().map(|file| file.path.clone()).collect(),
            active: self.files.get(self.active).map(|file| file.path.clone()),
            font_size: Some(self.font_size),
            sidebar_visible: Some(self.sidebar_visible),
            view_mode: Some(self.view_mode),
            window: self.window_size,
        });
    }

    /// The update the banner should announce: a known newer release whose
    /// notice the user has not dismissed.
    pub fn visible_update_notice(&self) -> Option<&UpdateInfo> {
        self.update_notice
            .as_ref()
            .filter(|notice| self.dismissed_update.as_deref() != Some(notice.version.as_str()))
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
        let keyboard = event::listen_with(|ev, status, _window| match ev {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                // Ctrl everywhere, and Cmd as well on macOS.
                let command = modifiers.command() || modifiers.control();
                let shift = modifiers.shift();
                // Shortcuts a focused text widget already handled (Ctrl+C
                // in an editor, Enter in the find field) must not fire
                // twice; those only apply when the event went unclaimed.
                let unclaimed = matches!(status, event::Status::Ignored);
                match key.as_ref() {
                    Key::Character("o") if command => Some(Message::OpenDialog),
                    Key::Character("b") if command => Some(Message::ToggleSidebar),
                    Key::Character("f") if command => Some(Message::OpenFind),
                    Key::Character("0") if command => Some(Message::ResetFontSize),
                    Key::Character("=" | "+") if command => Some(Message::IncreaseFontSize),
                    Key::Character("-") if command => Some(Message::DecreaseFontSize),
                    Key::Character("c") if command && unclaimed => Some(Message::CopyShortcut),
                    Key::Character("a") if command && unclaimed => Some(Message::SelectAllShortcut),
                    Key::Named(Named::F11) => Some(Message::ToggleFullscreen),
                    Key::Named(Named::F3) => Some(if shift {
                        Message::FindPrevious
                    } else {
                        Message::FindNext
                    }),
                    Key::Named(Named::Escape) => Some(Message::Escape),
                    Key::Named(Named::Enter) if unclaimed => Some(if shift {
                        Message::FindPrevious
                    } else {
                        Message::FindNext
                    }),
                    _ => None,
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                Some(Message::ModifiersChanged(modifiers))
            }
            Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
            Event::Window(window::Event::Opened { size, .. } | window::Event::Resized(size)) => {
                Some(Message::WindowResized(size.width, size.height))
            }
            Event::Window(window::Event::CloseRequested) => Some(Message::CloseRequested),
            _ => None,
        });

        let mut hasher = DefaultHasher::new();
        for f in &self.files {
            f.path.hash(&mut hasher);
        }
        let path_hash = hasher.finish();

        // Re-check for updates periodically so long-running sessions still
        // hear about new releases; the launch check covers the common case.
        let update_timer = iced::time::every(std::time::Duration::from_secs(6 * 60 * 60))
            .map(|_| Message::UpdateCheckTick);

        Subscription::batch(vec![keyboard, file_watcher(path_hash), update_timer])
    }
}
