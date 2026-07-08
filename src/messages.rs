use iced::{Point, Theme, widget::text_editor};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum Message {
    OpenDialog,
    FileDropped(PathBuf),
    FilesLoaded(Vec<(PathBuf, String, Option<SystemTime>)>),
    SelectFile(usize),
    CloseFile(usize),
    ToggleSidebar,
    ToggleViewMode,
    IncreaseFontSize,
    DecreaseFontSize,
    ThemeChanged(Theme),
    ToggleFullscreen,
    EditorAction(text_editor::Action),
    RenderedBlockAction(usize, text_editor::Action),
    RenderedCrossBlockSelection {
        anchor: usize,
        target: usize,
        point: Point,
    },
    RenderedBlockClicked(usize),
    CopyRenderedSelection,
    SelectAllRendered,
    CopyCode(String),
    CopyRenderedText(String),
    FileChanged(usize, SystemTime),
    FileReloaded(usize, String, SystemTime),
    NoOp,
    WatchTick,
}
