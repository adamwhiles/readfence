use crate::app::{App, OpenFile, ViewMode};
use crate::messages::Message;
use crate::styles::{
    style_app_background, style_badge, style_btn_ghost, style_btn_ghost_dim, style_btn_primary,
    style_btn_seg, style_btn_seg_active, style_file_active, style_file_inactive, style_panel,
    style_sidebar, style_status_bar, style_subtle_panel, style_toolbar,
};
use iced::{
    Border, Center, Color, Element, Fill, Font, Theme,
    widget::{
        Space, button, column, container, markdown, pick_list, row, rule, scrollable, text,
        text_editor,
    },
};
use std::path::Path;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        container(
            column![
                self.view_toolbar(),
                if self.files.is_empty() {
                    self.view_welcome()
                } else {
                    self.view_body()
                }
            ]
            .height(Fill),
        )
        .width(Fill)
        .height(Fill)
        .style(style_app_background)
        .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        let active_title = self
            .files
            .get(self.active)
            .map(|file| file_name(&file.path))
            .unwrap_or_else(|| "No document open".to_string());

        let active_meta = self
            .files
            .get(self.active)
            .map(|file| {
                format!(
                    "{} words · {}",
                    word_count(&file.content),
                    parent_label(&file.path)
                )
            })
            .unwrap_or_else(|| "Open a Markdown file to start reading".to_string());

        let brand = row![
            container(text("R").size(14).font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            }))
            .center(30)
            .style(style_badge),
            column![
                text("Readfence").size(15).font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
                text(active_title)
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(Color {
                            a: 0.72,
                            ..theme.extended_palette().background.base.text
                        }),
                    }),
            ]
            .spacing(1)
        ]
        .spacing(10)
        .align_y(Center);

        let open = button(text("Open").size(13))
            .on_press(Message::OpenDialog)
            .style(style_btn_primary)
            .padding([8, 18]);

        let sidebar_label = if self.sidebar_visible {
            "Hide files"
        } else {
            "Show files"
        };
        let sidebar_btn = button(text(sidebar_label).size(13))
            .on_press(Message::ToggleSidebar)
            .style(style_btn_ghost)
            .padding([8, 14]);

        let (rendered_style, source_style) = match self.view_mode {
            ViewMode::Rendered => (
                style_btn_seg_active as fn(&Theme, button::Status) -> button::Style,
                style_btn_seg as fn(&Theme, button::Status) -> button::Style,
            ),
            ViewMode::Source => (
                style_btn_seg as fn(&Theme, button::Status) -> button::Style,
                style_btn_seg_active as fn(&Theme, button::Status) -> button::Style,
            ),
        };
        let seg = container(
            row![
                button(text("Rendered").size(12))
                    .on_press(Message::ToggleViewMode)
                    .style(rendered_style)
                    .padding([6, 13]),
                button(text("Source").size(12))
                    .on_press(Message::ToggleViewMode)
                    .style(source_style)
                    .padding([6, 13]),
            ]
            .spacing(2),
        )
        .style(style_subtle_panel)
        .padding(3);

        let font_row = row![
            button(text("-").size(15))
                .on_press(Message::DecreaseFontSize)
                .style(style_btn_ghost)
                .padding([6, 10]),
            container(text(format!("{}px", self.font_size as u32)).size(12))
                .padding([6, 12])
                .style(style_subtle_panel),
            button(text("+").size(15))
                .on_press(Message::IncreaseFontSize)
                .style(style_btn_ghost)
                .padding([6, 10]),
        ]
        .spacing(3)
        .align_y(Center);

        let theme_picker = pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged)
            .text_size(13)
            .padding([8, 10]);

        let fs_label = if self.fullscreen {
            "Exit full"
        } else {
            "Fullscreen"
        };
        let fs_btn = button(text(fs_label).size(13))
            .on_press(Message::ToggleFullscreen)
            .style(style_btn_ghost)
            .padding([8, 14]);

        let toolbar_row = row![
            brand,
            container(
                text(active_meta)
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(Color {
                            a: 0.58,
                            ..theme.extended_palette().background.base.text
                        }),
                    })
            )
            .padding([0, 10]),
            Space::new().width(Fill),
            open,
            sidebar_btn,
            seg,
            font_row,
            theme_picker,
            fs_btn,
        ]
        .spacing(8)
        .padding([12, 18])
        .align_y(Center);

        column![
            container(toolbar_row).width(Fill).style(style_toolbar),
            rule::horizontal(1),
        ]
        .into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let header = row![
            column![
                text("Library").size(16).font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
                text("Open Markdown files")
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(Color {
                            a: 0.56,
                            ..theme.extended_palette().background.base.text
                        }),
                    }),
            ]
            .spacing(2),
            Space::new().width(Fill),
            container(text(self.files.len().to_string()).size(11))
                .padding([4, 9])
                .style(style_badge),
        ]
        .spacing(10)
        .align_y(Center);

        let items: Vec<Element<'_, Message>> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let name = file_name(&file.path);
                let location = parent_label(&file.path);
                let is_active = i == self.active;

                let label = button(
                    column![
                        text(name).size(13).font(Font {
                            weight: if is_active {
                                iced::font::Weight::Bold
                            } else {
                                iced::font::Weight::Normal
                            },
                            ..Font::DEFAULT
                        }),
                        text(location).size(11).style(|theme: &Theme| text::Style {
                            color: Some(Color {
                                a: 0.54,
                                ..theme.extended_palette().background.base.text
                            }),
                        }),
                    ]
                    .spacing(2),
                )
                .on_press(Message::SelectFile(i))
                .width(Fill)
                .padding([9, 12])
                .style(if is_active {
                    style_file_active
                } else {
                    style_file_inactive
                });

                let close = button(text("x").size(12))
                    .on_press(Message::CloseFile(i))
                    .padding([8, 9])
                    .style(style_btn_ghost_dim);

                row![label, close].spacing(4).align_y(Center).into()
            })
            .collect();

        container(
            column![
                container(header).padding([14, 14]),
                scrollable(column(items).spacing(4).padding([8, 8])).height(Fill),
            ]
            .height(Fill),
        )
        .width(252)
        .height(Fill)
        .style(style_sidebar)
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

        let workspace: Element<'_, Message> = if self.sidebar_visible {
            row![self.view_sidebar(), rule::vertical(1), main]
                .height(Fill)
                .into()
        } else {
            main
        };

        column![workspace, self.view_status_bar()]
            .height(Fill)
            .into()
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        let mode = match self.view_mode {
            ViewMode::Rendered => "Rendered preview",
            ViewMode::Source => "Source view",
        };

        let file_meta = self
            .files
            .get(self.active)
            .map(|file| {
                format!(
                    "{} lines · {} words",
                    line_count(&file.content),
                    word_count(&file.content)
                )
            })
            .unwrap_or_else(|| "Ready".to_string());

        container(
            row![
                text(mode).size(12),
                container(rule::vertical(1)).height(16),
                text(file_meta).size(12),
                Space::new().width(Fill),
                text(format!("{} files", self.files.len())).size(12),
                container(rule::vertical(1)).height(16),
                text(format!("Theme: {}", self.theme)).size(12),
            ]
            .spacing(10)
            .padding([7, 16])
            .align_y(Center),
        )
        .width(Fill)
        .style(style_status_bar)
        .into()
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
                let start = i;
                while i < items.len() && !matches!(&items[i], markdown::Item::CodeBlock { .. }) {
                    i += 1;
                }
                let batch = markdown::view(&items[start..i], settings.clone())
                    .map(|url| Message::LinkClicked(url.to_string()));
                elements.push(batch.into());
            }
        }

        let document_header = column![
            row![
                column![
                    text(file_name(&file.path)).size(28).font(Font {
                        weight: iced::font::Weight::Bold,
                        ..Font::DEFAULT
                    }),
                    text(parent_label(&file.path))
                        .size(13)
                        .style(|theme: &Theme| text::Style {
                            color: Some(Color {
                                a: 0.56,
                                ..theme.extended_palette().background.base.text
                            }),
                        }),
                ]
                .spacing(4),
                Space::new().width(Fill),
                badge("Markdown".to_string()),
            ]
            .align_y(Center),
            row![
                badge(format!("{} words", word_count(&file.content))),
                badge(format!("{} lines", line_count(&file.content))),
                badge(format!("{}px text", self.font_size as u32)),
            ]
            .spacing(8),
        ]
        .spacing(14);

        let document = container(
            column![
                document_header,
                rule::horizontal(1),
                column(elements).spacing(10),
            ]
            .spacing(22),
        )
        .width(Fill)
        .max_width(980)
        .padding([30, 38])
        .style(style_panel);

        scrollable(
            container(document)
                .width(Fill)
                .center_x(Fill)
                .padding([28, 34]),
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
            .padding([5, 10]);

        let header = container(
            row![
                text(language.unwrap_or("code"))
                    .size(11)
                    .font(Font::MONOSPACE)
                    .style(|theme: &Theme| text::Style {
                        color: Some(Color {
                            a: 0.62,
                            ..theme.extended_palette().background.base.text
                        }),
                    }),
                Space::new().width(Fill),
                copy_btn,
            ]
            .align_y(Center),
        )
        .width(Fill)
        .padding([7, 12])
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.strong.color.into()),
                border: Border {
                    radius: iced::border::Radius {
                        top_left: 8.0,
                        top_right: 8.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        let code_view = markdown::view(std::slice::from_ref(item), settings)
            .map(|url| Message::LinkClicked(url.to_string()));

        container(column![header, code_view])
            .width(Fill)
            .style(style_subtle_panel)
            .into()
    }

    fn view_source<'a>(&'a self, file: &'a OpenFile) -> Element<'a, Message> {
        let header = row![
            column![
                text(file_name(&file.path)).size(18).font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
                text("Read-only Markdown source")
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(Color {
                            a: 0.58,
                            ..theme.extended_palette().background.base.text
                        }),
                    }),
            ]
            .spacing(2),
            Space::new().width(Fill),
            badge(format!("{} lines", line_count(&file.content))),
        ]
        .align_y(Center);

        let editor = text_editor(&file.editor_content)
            .on_action(Message::EditorAction)
            .font(Font::MONOSPACE)
            .size(self.font_size * 0.9)
            .height(Fill)
            .padding([24, 28]);

        let panel = container(
            column![header, rule::horizontal(1), editor]
                .spacing(16)
                .height(Fill),
        )
        .width(Fill)
        .height(Fill)
        .max_width(1040)
        .padding([22, 24])
        .style(style_panel);

        container(panel)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .padding([28, 34])
            .into()
    }

    fn view_welcome(&self) -> Element<'_, Message> {
        let shortcuts = column![
            shortcut_hint("Ctrl+O", "Open files"),
            shortcut_hint("Ctrl+B", "Toggle file list"),
            shortcut_hint("F11", "Toggle fullscreen"),
            shortcut_hint("Ctrl+= / -", "Adjust font size"),
        ]
        .spacing(10);

        let welcome = container(
            column![
                container(text("R").size(24).font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }))
                .center(52)
                .style(style_badge),
                text("Readfence").size(46).font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
                text("A polished Markdown reading desk with live reload, drag-and-drop opening, source view, and theme-aware presentation.")
                    .size(16)
                    .style(|theme: &Theme| text::Style {
                        color: Some(Color {
                            a: 0.64,
                            ..theme.extended_palette().background.base.text
                        }),
                    }),
                row![
                    button(text("Open Markdown File").size(14))
                        .on_press(Message::OpenDialog)
                        .style(style_btn_primary)
                        .padding([11, 24]),
                    container(text("Ctrl+O").size(12).font(Font::MONOSPACE))
                        .padding([8, 12])
                        .style(style_subtle_panel),
                ]
                .spacing(10)
                .align_y(Center),
                container(shortcuts)
                    .width(Fill)
                    .padding(16)
                    .style(style_subtle_panel),
            ]
            .spacing(18),
        )
        .max_width(620)
        .padding([36, 42])
        .style(style_panel);

        container(welcome)
            .center(Fill)
            .width(Fill)
            .height(Fill)
            .padding(28)
            .into()
    }
}

pub fn shortcut_hint<'a>(key: &'a str, desc: &'a str) -> Element<'a, Message> {
    row![
        container(text(key).size(11).font(Font::MONOSPACE))
            .padding([4, 9])
            .style(style_badge),
        text(desc).size(13),
    ]
    .spacing(12)
    .align_y(Center)
    .into()
}

fn badge<'a>(label: String) -> Element<'a, Message> {
    container(text(label).size(11))
        .padding([5, 10])
        .style(style_badge)
        .into()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

fn parent_label(path: &Path) -> String {
    path.parent()
        .and_then(|parent| parent.to_str())
        .unwrap_or("")
        .to_string()
}

fn word_count(content: &str) -> usize {
    content.split_whitespace().count()
}

fn line_count(content: &str) -> usize {
    content.lines().count().max(1)
}
