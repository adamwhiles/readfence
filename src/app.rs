use crate::files::{file_watcher, load_paths};
use crate::markdown_text::RenderedBlock;
use crate::messages::Message;
use crate::updates::{UpdateInfo, check_for_update};
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

pub struct App {
    pub files: Vec<OpenFile>,
    pub active: usize,
    pub sidebar_visible: bool,
    pub view_mode: ViewMode,
    pub font_size: f32,
    pub theme: Theme,
    pub fullscreen: bool,
    pub window_width: f32,
    pub remote_images: HashMap<String, RemoteImage>,
    pub update_notice: Option<UpdateInfo>,
    /// Version the user dismissed; that release stays quiet, a newer one
    /// notifies again.
    pub dismissed_update: Option<String>,
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
    #[allow(dead_code)]
    pub content: String,
    pub editor_content: text_editor::Content,
    pub rendered_text: String,
    pub rendered_blocks: Vec<RenderedBlock>,
    pub last_modified: Option<std::time::SystemTime>,
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
            window_width: 1280.0,
            remote_images: HashMap::new(),
            update_notice: None,
            dismissed_update: None,
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
        let app = Self {
            theme,
            ..Self::default()
        };

        // Open any files passed on the command line, e.g. from a file
        // manager's "Open with" action or a `.desktop` MimeType association.
        // Unsupported paths are filtered out by `load_paths`.
        let paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
        let load_task = if paths.is_empty() {
            Task::none()
        } else {
            Task::perform(load_paths(paths), Message::FilesLoaded)
        };
        let update_task = Task::perform(check_for_update(), Message::UpdateCheckCompleted);

        (app, Task::batch([load_task, update_task]))
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
        let keyboard = event::listen_with(|ev, _status, _window| match ev {
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
            Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
            Event::Window(
                window::Event::Opened { size, .. } | window::Event::Resized(size),
            ) => Some(Message::WindowResized(size.width)),
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
