  #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

  mod app;
  mod files;
  mod messages;
  mod styles;
  mod update;
  mod view;

  use app::App;
  use iced::window;

  fn main() -> iced::Result {
      let mut builder = iced::application(App::new, App::update, App::view)
          .title(App::title)
          .theme(App::theme)
          .subscription(App::subscription);

      if let Some(icon) = load_icon() {
          builder = builder.window(window::Settings {
              icon: Some(icon),
              position: window::Position::Centered,
              ..Default::default()
          });
      } else {
          builder = builder.centered();
      }

      builder.run()
  }

  fn load_icon() -> Option<window::Icon> {
      const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
      let img = image::load_from_memory(ICON_BYTES).ok()?;
      let img = img.resize(32, 32, image::imageops::FilterType::Lanczos3).to_rgba8();
      let (w, h) = (img.width(), img.height());
      window::icon::from_rgba(img.into_raw(), w, h).ok()
  }