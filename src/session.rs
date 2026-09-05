//! Remembers the reading session between launches: which documents were
//! open, which one was active, the reading preferences, and the window size.
//!
//! The file is a plain `key=value` list so it stays inspectable and needs no
//! extra dependencies.

use crate::app::ViewMode;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Session {
    pub files: Vec<PathBuf>,
    pub active: Option<PathBuf>,
    pub font_size: Option<f32>,
    pub sidebar_visible: Option<bool>,
    pub view_mode: Option<ViewMode>,
    /// Logical window size, width by height.
    pub window: Option<(f32, f32)>,
}

fn session_file() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("readfence").join("session"))
}

pub fn load() -> Option<Session> {
    let text = std::fs::read_to_string(session_file()?).ok()?;
    Some(parse(&text))
}

pub fn save(session: &Session) {
    let Some(path) = session_file() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, serialize(session));
}

fn parse(text: &str) -> Session {
    let mut session = Session::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "file" if !value.is_empty() => session.files.push(PathBuf::from(value)),
            "active" if !value.is_empty() => session.active = Some(PathBuf::from(value)),
            "font_size" => session.font_size = value.trim().parse().ok(),
            "sidebar" => {
                session.sidebar_visible = match value.trim() {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                }
            }
            "view" => {
                session.view_mode = match value.trim() {
                    "rendered" => Some(ViewMode::Rendered),
                    "source" => Some(ViewMode::Source),
                    _ => None,
                }
            }
            "window" => {
                session.window = value.trim().split_once('x').and_then(|(width, height)| {
                    let width: f32 = width.parse().ok()?;
                    let height: f32 = height.parse().ok()?;
                    (width >= 200.0 && height >= 150.0).then_some((width, height))
                })
            }
            _ => {}
        }
    }
    session
}

fn serialize(session: &Session) -> String {
    let mut out = String::new();
    for file in &session.files {
        out.push_str(&format!("file={}\n", file.to_string_lossy()));
    }
    if let Some(active) = &session.active {
        out.push_str(&format!("active={}\n", active.to_string_lossy()));
    }
    if let Some(font_size) = session.font_size {
        out.push_str(&format!("font_size={}\n", font_size.round() as u32));
    }
    if let Some(sidebar) = session.sidebar_visible {
        out.push_str(&format!("sidebar={sidebar}\n"));
    }
    if let Some(view_mode) = session.view_mode {
        out.push_str(match view_mode {
            ViewMode::Rendered => "view=rendered\n",
            ViewMode::Source => "view=source\n",
        });
    }
    if let Some((width, height)) = session.window {
        out.push_str(&format!(
            "window={}x{}\n",
            width.round() as u32,
            height.round() as u32
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Session, parse, serialize};
    use crate::app::ViewMode;
    use std::path::PathBuf;

    #[test]
    fn round_trips_a_session() {
        let session = Session {
            files: vec![
                PathBuf::from("/docs/README.md"),
                PathBuf::from("/docs/notes with spaces.md"),
            ],
            active: Some(PathBuf::from("/docs/notes with spaces.md")),
            font_size: Some(18.0),
            sidebar_visible: Some(false),
            view_mode: Some(ViewMode::Source),
            window: Some((1400.0, 900.0)),
        };

        assert_eq!(parse(&serialize(&session)), session);
    }

    #[test]
    fn tolerates_missing_and_malformed_lines() {
        let session = parse("garbage\nfont_size=big\nwindow=10x10\nfile=\nfile=/a.md\n");

        assert_eq!(session.files, vec![PathBuf::from("/a.md")]);
        assert_eq!(session.active, None);
        assert_eq!(session.font_size, None);
        assert_eq!(session.window, None);
        assert_eq!(session.view_mode, None);
    }
}
