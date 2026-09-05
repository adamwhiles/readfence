use iced::{Point, Theme, keyboard, widget::text_editor};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum Message {
    OpenDialog,
    FileDropped(PathBuf),
    FilesLoaded(Vec<(PathBuf, String, Option<SystemTime>)>),
    /// The documents from the last session, plus the one that was active.
    SessionRestored(Vec<(PathBuf, String, Option<SystemTime>)>, Option<PathBuf>),
    SelectFile(usize),
    CloseFile(usize),
    ToggleSidebar,
    ToggleViewMode,
    IncreaseFontSize,
    DecreaseFontSize,
    /// Wheel-driven zoom: positive steps enlarge, negative shrink.
    Zoom(i32),
    ResetFontSize,
    ThemeChanged(Theme),
    ToggleFullscreen,
    ModifiersChanged(keyboard::Modifiers),
    EditorAction(text_editor::Action),
    RenderedBlockAction(usize, text_editor::Action),
    RenderedCrossBlockSelection {
        anchor: usize,
        target: usize,
        point: Point,
    },
    /// The mouse went down on a rendered block; other blocks drop their
    /// selection.
    RenderedBlockPressed(usize),
    RenderedBlockClicked(usize),
    CopyRenderedSelection,
    SelectAllRendered,
    /// Ctrl+C pressed while no text widget had focus.
    CopyShortcut,
    /// Ctrl+A pressed while no text widget had focus.
    SelectAllShortcut,
    CopyCode(String),
    CopyRenderedText(String),
    ClearCopyNotice(u64),
    OpenLink(String),
    /// Scroll the rendered document so the block with this index is at the
    /// top of the view.
    JumpToBlock(usize),
    OpenFind,
    CloseFind,
    /// Escape: closes the find bar, or the About menu, whichever is open.
    Escape,
    FindQueryChanged(String),
    FindNext,
    FindPrevious,
    /// Enter in the find field: next match, or previous with Shift held.
    FindSubmit,
    FileChanged(usize, SystemTime),
    FileReloaded(usize, String, SystemTime),
    WindowResized(f32, f32),
    CloseRequested,
    RemoteImageLoaded(String, Option<Vec<u8>>),
    UpdateCheckTick,
    UpdateCheckCompleted(crate::updates::UpdateCheckOutcome),
    CheckForUpdates,
    ToggleUpdateMenu,
    InstallUpdate,
    InstallCompleted(Result<PathBuf, String>),
    RestartApp,
    OpenUpdatePage,
    OpenRepoPage,
    DismissUpdate,
    NoOp,
    WatchTick,
}
