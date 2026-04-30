use iced::{Theme, widget::text_editor};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum Message {
    OpenDialog,
    FilesLoaded(Vec<(PathBuf, String, Option<SystemTime>)>),
    SelectFile(usize),
    CloseFile(usize),
    ToggleSidebar,
    ToggleViewMode,
    IncreaseFontSize,
    DecreaseFontSize,
    ThemeChanged(Theme),
    ToggleFullscreen,
    LinkClicked(String),
    EditorAction(text_editor::Action),
    CopyCode(String),
    FileChanged(usize, SystemTime),
    FileReloaded(usize, String, SystemTime),
    NoOp,
    WatchTick,
}