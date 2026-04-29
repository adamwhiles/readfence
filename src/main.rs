#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    Border, Center, Color, Element, Event, Fill, Font, Shadow, Subscription, Task, Theme, Vector,
    clipboard, event,
    keyboard::{self, key::Named, Key},
    widget::{
        button, column, container, markdown, pick_list,
        row, rule, scrollable, text, text_editor, Space,
    },
    window,
};
use std::path::PathBuf;

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
    // Pre-scale to 32×32 — Windows title bar icons are 16–32px, and scaling a large
    // gradient icon down live produces a garbled result.
    let img = img.resize(32, 32, image::imageops::FilterType::Lanczos3).to_rgba8();
    let (w, h) = (img.width(), img.height());
    window::icon::from_rgba(img.into_raw(), w, h).ok()
}

// ── State ────────────────────────────────────────────────────────────────────

struct App {
    files: Vec<OpenFile>,
    active: usize,
    sidebar_visible: bool,
    view_mode: ViewMode,
    font_size: f32,
    theme: Theme,
    fullscreen: bool,
}

struct OpenFile {
    path: PathBuf,
    #[allow(dead_code)]
    content: String,
    items: Vec<markdown::Item>,
    editor_content: text_editor::Content,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum ViewMode {
    #[default]
    Rendered,
    Source,
}

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Message {
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

// ── Init ─────────────────────────────────────────────────────────────────────

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
        }
    }
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::Nightfly,
                ..Self::default()
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
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

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|ev, _status, _window| match ev {
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
            _ => None,
        })
    }
}

// ── Update ───────────────────────────────────────────────────────────────────

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenDialog => Task::perform(load_files(), Message::FilesLoaded),

            Message::FilesLoaded(files) => {
                let before = self.files.len();
                for (path, content) in files {
                    if self.files.iter().any(|f| f.path == path) {
                        continue;
                    }
                    let items = markdown::parse(&content).collect();
                    let editor_content = text_editor::Content::with_text(&content);
                    self.files.push(OpenFile { path, content, items, editor_content });
                }
                if self.files.len() > before {
                    self.active = self.files.len() - 1;
                }
                Task::none()
            }

            Message::SelectFile(i) => {
                self.active = i;
                Task::none()
            }

            Message::CloseFile(i) => {
                self.files.remove(i);
                if !self.files.is_empty() {
                    self.active = self.active.min(self.files.len() - 1);
                }
                Task::none()
            }

            Message::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
                Task::none()
            }

            Message::ToggleViewMode => {
                self.view_mode = match self.view_mode {
                    ViewMode::Rendered => ViewMode::Source,
                    ViewMode::Source => ViewMode::Rendered,
                };
                Task::none()
            }

            Message::IncreaseFontSize => {
                self.font_size = (self.font_size + 2.0).min(36.0);
                Task::none()
            }

            Message::DecreaseFontSize => {
                self.font_size = (self.font_size - 2.0).max(10.0);
                Task::none()
            }

            Message::ThemeChanged(theme) => {
                self.theme = theme;
                Task::none()
            }

            Message::ToggleFullscreen => {
                self.fullscreen = !self.fullscreen;
                let mode = if self.fullscreen {
                    window::Mode::Fullscreen
                } else {
                    window::Mode::Windowed
                };
                window::latest().and_then(move |id| window::set_mode(id, mode))
            }

            Message::LinkClicked(url) => {
                let _ = open::that_detached(&url);
                Task::none()
            }

            Message::EditorAction(action) => {
                // Allow cursor movement and selection; silently drop text edits (read-only).
                if !matches!(action, text_editor::Action::Edit(_)) {
                    if let Some(file) = self.files.get_mut(self.active) {
                        file.editor_content.perform(action);
                    }
                }
                Task::none()
            }

            Message::CopyCode(code) => clipboard::write(code),
        }
    }
}

// ── View ─────────────────────────────────────────────────────────────────────

impl App {
    fn view(&self) -> Element<'_, Message> {
        column![
            self.view_toolbar(),
            if self.files.is_empty() {
                self.view_welcome()
            } else {
                self.view_body()
            }
        ]
        .height(Fill)
        .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        // Left group: navigation actions
        let open = button(text("Open").size(13))
            .on_press(Message::OpenDialog)
            .style(style_btn_primary)
            .padding([7, 16]);

        let sidebar_label = if self.sidebar_visible { "Hide Sidebar" } else { "Sidebar" };
        let sidebar_btn = button(text(sidebar_label).size(13))
            .on_press(Message::ToggleSidebar)
            .style(style_btn_ghost)
            .padding([7, 14]);

        // View mode toggle — visually indicates current mode
        let (rendered_style, source_style) = match self.view_mode {
            ViewMode::Rendered => (style_btn_seg_active as fn(&Theme, button::Status) -> button::Style, style_btn_seg as fn(&Theme, button::Status) -> button::Style),
            ViewMode::Source   => (style_btn_seg         as fn(&Theme, button::Status) -> button::Style, style_btn_seg_active as fn(&Theme, button::Status) -> button::Style),
        };
        let seg = container(
            row![
                button(text("Rendered").size(12))
                    .on_press(Message::ToggleViewMode)
                    .style(rendered_style)
                    .padding([5, 12]),
                button(text("Source").size(12))
                    .on_press(Message::ToggleViewMode)
                    .style(source_style)
                    .padding([5, 12]),
            ]
            .spacing(1),
        )
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.strong.color.into()),
                border: Border {
                    radius: 7.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .padding(2);

        // Right group: font size + theme + fullscreen
        let font_row = row![
            button(text("−").size(15))
                .on_press(Message::DecreaseFontSize)
                .style(style_btn_ghost)
                .padding([5, 10]),
            container(
                text(format!("{}px", self.font_size as u32)).size(12),
            )
            .padding([5, 10])
            .style(|theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(p.background.base.color.into()),
                    border: Border {
                        radius: 5.0.into(),
                        color: p.background.strong.color,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            }),
            button(text("+").size(15))
                .on_press(Message::IncreaseFontSize)
                .style(style_btn_ghost)
                .padding([5, 10]),
        ]
        .spacing(2)
        .align_y(Center);

        let theme_picker = pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged)
            .text_size(13)
            .padding([7, 10]);

        let fs_label = if self.fullscreen { "Exit Full" } else { "Fullscreen" };
        let fs_btn = button(text(fs_label).size(13))
            .on_press(Message::ToggleFullscreen)
            .style(style_btn_ghost)
            .padding([7, 14]);

        let toolbar_row = row![
            open,
            sidebar_btn,
            container(rule::vertical(1)).height(22),
            seg,
            Space::new().width(Fill),
            font_row,
            container(rule::vertical(1)).height(22),
            theme_picker,
            container(rule::vertical(1)).height(22),
            fs_btn,
        ]
        .spacing(6)
        .padding([9, 16])
        .align_y(Center);

        column![
            container(toolbar_row).width(Fill).style(|theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(p.background.weak.color.into()),
                    ..Default::default()
                }
            }),
            rule::horizontal(1),
        ]
        .into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let header = container(
            text("OPEN FILES")
                .size(11)
                .font(Font { weight: iced::font::Weight::Bold, ..Font::DEFAULT })
                .style(|theme: &Theme| text::Style {
                    color: Some(Color {
                        a: 0.55,
                        ..theme.extended_palette().primary.base.color
                    }),
                }),
        )
        .padding([10, 14]);

        let items: Vec<Element<'_, Message>> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let name = file
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Untitled");

                let is_active = i == self.active;

                let label = button(text(name).size(14))
                    .on_press(Message::SelectFile(i))
                    .width(Fill)
                    .padding([8, 12])
                    .style(if is_active {
                        style_file_active
                    } else {
                        style_file_inactive
                    });

                let close = button(text("×").size(12))
                    .on_press(Message::CloseFile(i))
                    .padding([7, 8])
                    .style(style_btn_ghost_dim);

                row![label, close]
                    .spacing(2)
                    .align_y(Center)
                    .into()
            })
            .collect();

        column![
            container(
                scrollable(
                    column![header, column(items).spacing(2).padding([0, 6])]
                )
                .height(Fill),
            )
            .width(210)
            .height(Fill)
            .style(|theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(p.background.weak.color.into()),
                    ..Default::default()
                }
            }),
        ]
        .into()
    }

    fn view_body(&self) -> Element<'_, Message> {
        let main = match self.files.get(self.active) {
            Some(file) => match self.view_mode {
                ViewMode::Rendered => self.view_rendered(file),
                ViewMode::Source => self.view_source(file),
            },
            None => container(text("No file selected"))
                .center(Fill)
                .width(Fill)
                .height(Fill)
                .into(),
        };

        if self.sidebar_visible {
            row![self.view_sidebar(), rule::vertical(1), main]
                .height(Fill)
                .into()
        } else {
            main
        }
    }

    fn view_rendered<'a>(&'a self, file: &'a OpenFile) -> Element<'a, Message> {
        let settings = markdown::Settings::with_text_size(
            self.font_size,
            markdown::Style::from_palette(self.theme.palette()),
        );

        let items = &file.items;
        let mut elements: Vec<Element<'a, Message>> = Vec::new();
        let mut i = 0;

        while i < items.len() {
            if let markdown::Item::CodeBlock { language, code, .. } = &items[i] {
                elements.push(self.view_code_block(
                    &items[i],
                    language.as_deref(),
                    code,
                    settings.clone(),
                ));
                i += 1;
            } else {
                // Batch consecutive non-code items so markdown renders them together
                // (preserves spacing between headings, paragraphs, lists, etc.)
                let start = i;
                while i < items.len()
                    && !matches!(&items[i], markdown::Item::CodeBlock { .. })
                {
                    i += 1;
                }
                let batch = markdown::view(&items[start..i], settings.clone())
                    .map(|url| Message::LinkClicked(url.to_string()));
                elements.push(batch.into());
            }
        }

        scrollable(
            container(column(elements).spacing(8))
                .width(Fill)
                .padding([32, 48]),
        )
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn view_code_block<'a>(
        &self,
        item: &'a markdown::Item,
        language: Option<&'a str>,
        code: &str,
        settings: markdown::Settings,
    ) -> Element<'a, Message> {
        let copy_btn = button(text("Copy").size(11))
            .on_press(Message::CopyCode(code.to_string()))
            .style(style_btn_ghost)
            .padding([4, 10]);

        let header = container(
            row![
                text(language.unwrap_or(""))
                    .size(11)
                    .font(Font::MONOSPACE)
                    .style(|theme: &Theme| text::Style {
                        color: Some(Color {
                            a: 0.5,
                            ..theme.extended_palette().background.base.text
                        }),
                    }),
                Space::new().width(Fill),
                copy_btn,
            ]
            .align_y(Center),
        )
        .width(Fill)
        .padding([6, 10])
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.strong.color.into()),
                border: Border {
                    radius: iced::border::Radius {
                        top_left: 6.0,
                        top_right: 6.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        // Render the code block through markdown::view so syntax highlighting is preserved.
        let code_view = markdown::view(std::slice::from_ref(item), settings)
            .map(|url| Message::LinkClicked(url.to_string()));

        container(column![header, code_view])
            .width(Fill)
            .style(|theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    border: Border {
                        radius: 6.0.into(),
                        color: p.background.strong.color,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            })
            .into()
    }

    fn view_source<'a>(&'a self, file: &'a OpenFile) -> Element<'a, Message> {
        // text_editor gives full text selection and Ctrl+C copy support.
        // EditorAction::Edit variants are silently dropped in update() to keep it read-only.
        text_editor(&file.editor_content)
            .on_action(Message::EditorAction)
            .font(Font::MONOSPACE)
            .size(self.font_size * 0.9)
            .height(Fill)
            .padding([32, 48])
            .into()
    }

    fn view_welcome(&self) -> Element<'_, Message> {
        container(
            column![
                text("Readfence")
                    .size(56)
                    .font(Font { weight: iced::font::Weight::Bold, ..Font::DEFAULT })
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.extended_palette().primary.base.color),
                    }),
                text("A clean markdown viewer for developers")
                    .size(16)
                    .style(|theme: &Theme| text::Style {
                        color: Some(Color {
                            a: 0.6,
                            ..theme.extended_palette().background.base.text
                        }),
                    }),
                Space::new().height(40),
                button(text("Open Markdown File").size(14))
                    .on_press(Message::OpenDialog)
                    .style(style_btn_primary)
                    .padding([10, 28]),
                Space::new().height(32),
                column![
                    shortcut_hint("Ctrl+O", "Open files"),
                    shortcut_hint("Ctrl+B", "Toggle sidebar"),
                    shortcut_hint("F11", "Toggle fullscreen"),
                    shortcut_hint("Ctrl+= / -", "Adjust font size"),
                ]
                .spacing(10),
            ]
            .spacing(10)
            .align_x(Center),
        )
        .center(Fill)
        .width(Fill)
        .height(Fill)
        .into()
    }
}

fn shortcut_hint<'a>(key: &'a str, desc: &'a str) -> Element<'a, Message> {
    row![
        container(text(key).size(11).font(Font::MONOSPACE))
            .padding([3, 8])
            .style(|theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(p.background.strong.color.into()),
                    border: Border {
                        radius: 4.0.into(),
                        color: p.background.strong.color,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            }),
        text(desc).size(13),
    ]
    .spacing(12)
    .align_y(Center)
    .into()
}

// ── Button styles ─────────────────────────────────────────────────────────────

fn style_btn_primary(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let (bg, fg) = match status {
        button::Status::Hovered => (p.primary.strong.color, p.primary.strong.text),
        button::Status::Pressed => (p.primary.weak.color, p.primary.weak.text),
        _ => (p.primary.base.color, p.primary.base.text),
    };
    button::Style {
        background: Some(bg.into()),
        text_color: fg,
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.18),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 4.0,
        },
        snap: false,
    }
}

fn style_btn_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered => p.background.strong.color,
        button::Status::Pressed => p.background.base.color,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(bg.into()),
        text_color: p.background.base.text,
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn style_btn_ghost_dim(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let (bg, fg) = match status {
        button::Status::Hovered => (p.background.strong.color, p.background.base.text),
        button::Status::Pressed => (p.background.base.color, p.background.base.text),
        _ => (Color::TRANSPARENT, Color { a: 0.45, ..p.background.base.text }),
    };
    button::Style {
        background: Some(bg.into()),
        text_color: fg,
        border: Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

// Segmented control — inactive segment
fn style_btn_seg(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered => p.background.base.color,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(bg.into()),
        text_color: Color { a: 0.55, ..p.background.base.text },
        border: Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

// Segmented control — active segment
fn style_btn_seg_active(theme: &Theme, _status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    button::Style {
        background: Some(p.background.base.color.into()),
        text_color: p.background.base.text,
        border: Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        snap: false,
    }
}

fn style_file_active(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => p.primary.weak.color,
        _ => Color { a: 0.15, ..p.primary.base.color },
    };
    button::Style {
        background: Some(bg.into()),
        text_color: p.primary.base.color,
        border: Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn style_file_inactive(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let (bg, fg) = match status {
        button::Status::Hovered => (p.background.base.color, p.background.base.text),
        button::Status::Pressed => (p.background.strong.color, p.background.base.text),
        _ => (Color::TRANSPARENT, Color { a: (p.background.base.text.a * 0.85).max(0.7), ..p.background.base.text }),
    };
    button::Style {
        background: Some(bg.into()),
        text_color: fg,
        border: Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

// ── Async helpers ─────────────────────────────────────────────────────────────

async fn load_files() -> Vec<(PathBuf, String)> {
    let handles = rfd::AsyncFileDialog::new()
        .add_filter("Markdown", &["md", "markdown", "txt"])
        .set_title("Open Markdown Files")
        .pick_files()
        .await
        .unwrap_or_default();

    let mut result = Vec::new();
    for handle in handles {
        let path = handle.path().to_path_buf();
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            result.push((path, content));
        }
    }
    result
}
