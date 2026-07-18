  #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

  mod app;
  mod files;
  mod markdown_text;
  mod messages;
  mod selection_group;
  mod styles;
  mod update;
  mod updates;
  mod view;

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

      builder = builder.window(window::Settings {
          icon: load_icon(),
          position: window::Position::Centered,
          min_size: Some(iced::Size::new(520.0, 400.0)),
          ..Default::default()
      });

      builder.run()
  }

  fn load_icon() -> Option<window::Icon> {
      const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
      let img = image::load_from_memory(ICON_BYTES).ok()?;
      let img = img.resize(32, 32, image::imageops::FilterType::Lanczos3).to_rgba8();
      let (w, h) = (img.width(), img.height());
      window::icon::from_rgba(img.into_raw(), w, h).ok()
  }
