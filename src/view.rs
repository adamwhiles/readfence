use crate::app::{App, OpenFile, ViewMode};
use crate::markdown_text::{LinkHighlighter, RenderedBlock, RenderedBlockKind};
use crate::messages::Message;
use crate::selection_group::{SelectionRegion, selection_group};
use crate::styles::{
    style_app_background, style_badge, style_btn_ghost, style_btn_ghost_dim, style_btn_primary,
    style_btn_seg, style_btn_seg_active, style_file_active, style_file_inactive, style_panel,
    style_selectable_code, style_selectable_prose, style_sidebar, style_status_bar,
    style_subtle_panel, style_toolbar,
};
use iced::{
    Border, Center, Color, Element, Fill, Font, Theme,
    advanced::text::highlighter,
    keyboard::Key,
    widget::{
        Space, button, column, container, pick_list, row, rule, scrollable, text, text_editor,
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
        let mut elements = Vec::with_capacity(file.rendered_blocks.len());
        let mut selection_regions = Vec::with_capacity(file.rendered_blocks.len());
        for (index, block) in file.rendered_blocks.iter().enumerate() {
            selection_regions.push(match block.kind {
                RenderedBlockKind::Code { .. } => SelectionRegion::code(index),
                RenderedBlockKind::Rule => SelectionRegion::rule(),
                RenderedBlockKind::Quote => SelectionRegion::quote(index),
                _ => SelectionRegion::block(index),
            });
            elements.push(self.view_rendered_block(index, block));
        }
        let rendered_blocks = selection_group(elements, selection_regions);

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
                button(text("Copy text").size(11))
                    .on_press(Message::CopyRenderedText(file.rendered_text.clone()))
                    .style(style_btn_ghost)
                    .padding([5, 10]),
                badge("Markdown".to_string()),
            ]
            .align_y(Center),
            row![
                badge(format!("{} words", word_count(&file.content))),
                badge(format!("{} min read", reading_time_minutes(&file.content))),
                badge(format!("{} lines", line_count(&file.content))),
                badge(format!("{}px text", self.font_size as u32)),
            ]
            .spacing(8),
        ]
        .spacing(14);

        let document =
            container(column![document_header, rule::horizontal(1), rendered_blocks,].spacing(26))
                .width(Fill)
                .padding([34, 44])
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

    fn view_rendered_block<'a>(
        &'a self,
        index: usize,
        block: &'a RenderedBlock,
    ) -> Element<'a, Message> {
        match &block.kind {
            RenderedBlockKind::Code { language } => {
                self.view_selectable_code_block(index, language.as_deref(), block)
            }
            RenderedBlockKind::Rule => container(rule::horizontal(1))
                .width(Fill)
                .padding([8, 0])
                .into(),
            kind => {
                let size = match kind {
                    RenderedBlockKind::Heading(1) => self.font_size.max(15.0) * 1.85,
                    RenderedBlockKind::Heading(2) => self.font_size.max(15.0) * 1.55,
                    RenderedBlockKind::Heading(3) => self.font_size.max(15.0) * 1.32,
                    RenderedBlockKind::Heading(4) => self.font_size.max(15.0) * 1.16,
                    RenderedBlockKind::Heading(_) => self.font_size.max(15.0),
                    RenderedBlockKind::ListItem => self.font_size.max(15.0),
                    RenderedBlockKind::Quote => self.font_size.max(15.0),
                    RenderedBlockKind::Table => (self.font_size.max(15.0) * 0.92).max(13.0),
                    RenderedBlockKind::Paragraph
                    | RenderedBlockKind::Code { .. }
                    | RenderedBlockKind::Rule => self.font_size.max(15.0),
                };

                let editor = text_editor(&block.content)
                    .on_action(move |action| Message::RenderedBlockAction(index, action))
                    .key_binding(rendered_key_binding)
                    .size(size)
                    .line_height(text::LineHeight::Relative(1.35))
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .padding([2, 0])
                    .style(style_selectable_prose)
                    .highlight_with::<LinkHighlighter>(
                        block.link_highlights(),
                        link_highlight_format,
                    );

                match kind {
                    RenderedBlockKind::Quote => container(editor)
                        .width(Fill)
                        .padding([2, 14])
                        .style(|theme: &Theme| {
                            let p = theme.extended_palette();
                            container::Style {
                                border: Border {
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                    radius: 0.0.into(),
                                },
                                background: Some(
                                    Color {
                                        a: 0.08,
                                        ..p.primary.base.color
                                    }
                                    .into(),
                                ),
                                ..Default::default()
                            }
                        })
                        .into(),
                    _ => editor.into(),
                }
            }
        }
    }

    fn view_selectable_code_block<'a>(
        &self,
        index: usize,
        language: Option<&'a str>,
        block: &'a RenderedBlock,
    ) -> Element<'a, Message> {
        let copy_btn = button(text("Copy").size(11))
            .on_press(Message::CopyCode(block.text.clone()))
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
                background: Some(
                    Color {
                        a: 0.74,
                        ..p.background.strong.color
                    }
                    .into(),
                ),
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

        let code_view = text_editor(&block.content)
            .on_action(move |action| Message::RenderedBlockAction(index, action))
            .key_binding(rendered_key_binding)
            .font(Font::MONOSPACE)
            .size((self.font_size.max(15.0) * 0.86).max(13.0))
            .line_height(text::LineHeight::Relative(1.35))
            .wrapping(text::Wrapping::None)
            .padding([14, 16])
            .style(style_selectable_code);

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

fn reading_time_minutes(content: &str) -> usize {
    word_count(content).div_ceil(225).max(1)
}

fn line_count(content: &str) -> usize {
    content.lines().count().max(1)
}

fn rendered_key_binding(key_press: text_editor::KeyPress) -> Option<text_editor::Binding<Message>> {
    if key_press.modifiers.command() {
        match key_press.key.as_ref() {
            Key::Character("c") => {
                return Some(text_editor::Binding::Custom(Message::CopyRenderedSelection));
            }
            Key::Character("a") => {
                return Some(text_editor::Binding::Custom(Message::SelectAllRendered));
            }
            _ => {}
        }
    }

    text_editor::Binding::from_key_press(key_press)
}

fn link_highlight_format(_highlight: &(), theme: &Theme) -> highlighter::Format<Font> {
    highlighter::Format {
        color: Some(theme.extended_palette().primary.strong.color),
        font: Some(Font {
            weight: iced::font::Weight::Semibold,
            ..Font::DEFAULT
        }),
    }
}
