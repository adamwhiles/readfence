use iced::{Theme, widget::text_editor};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    OpenDialog,
    FilesLoaded(Vec<(PathBuf, String)>),
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
}