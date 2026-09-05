#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod files;
mod find;
mod markdown_text;
mod messages;
mod selection_group;
mod session;
mod styles;
mod update;
mod updates;
mod view;
mod zoom_area;

use app::App;
use iced::window;

fn main() -> iced::Result {
    let mut builder = iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .settings(iced::Settings {
            // Match the `.desktop` file basename so the window associates
            // with the installed desktop entry (correct icon/name under
            // Wayland app_id and X11 WM_CLASS, e.g. inside a Flatpak).
            id: Some("com.readfence.Readfence".into()),
            ..iced::Settings::default()
        });

    // Reopen at the size the window had when it was last closed.
    let size = session::load()
        .and_then(|session| session.window)
        .map(|(width, height)| iced::Size::new(width, height))
        .unwrap_or(window::Settings::default().size);

    builder = builder.window(window::Settings {
        icon: load_icon(),
        size,
        position: window::Position::Centered,
        min_size: Some(iced::Size::new(520.0, 400.0)),
        // Closing goes through the app so the session can be saved first.
        exit_on_close_request: false,
        ..Default::default()
    });

    builder.run()
}

fn load_icon() -> Option<window::Icon> {
    const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(ICON_BYTES).ok()?;
    let img = img
        .resize(32, 32, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let (w, h) = (img.width(), img.height());
    window::icon::from_rgba(img.into_raw(), w, h).ok()
}
