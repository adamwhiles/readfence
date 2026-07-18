use crate::app::{App, OpenFile, RemoteImage, ViewMode, svg_dimensions};
use crate::markdown_text::{
    ImageSource, RenderedBlock, RenderedBlockKind, SpanHighlighter, SpanStyle,
};
use crate::messages::Message;
use crate::selection_group::{SelectionRegion, selection_group};
use crate::styles::{
    style_app_background, style_badge, style_btn_ghost, style_btn_ghost_dim, style_btn_primary,
    style_btn_seg, style_btn_seg_active, style_file_active, style_file_inactive, style_panel,
    style_picker, style_scrollable, style_selectable_code, style_selectable_prose, style_sidebar,
    style_status_bar, style_subtle_panel, style_toolbar, style_update_banner, surface_color,
};
use iced::{
    Border, Center, Color, Element, Fill, Font, Theme,
    advanced::text::highlighter,
    keyboard::Key,
    widget::{
        Space, button, column, container, image, pick_list, row, rule, scrollable, svg, text,
        text_editor,
    },
};
use std::path::Path;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let mut layout = column![self.view_toolbar()];
        if let Some(notice) = &self.update_notice {
            layout = layout.push(self.view_update_banner(notice));
        }
        layout = layout.push(if self.files.is_empty() {
            self.view_welcome()
        } else {
            self.view_body()
        });

        container(layout.height(Fill))
            .width(Fill)
            .height(Fill)
            .style(style_app_background)
            .into()
    }

    fn view_update_banner<'a>(
        &self,
        notice: &'a crate::updates::UpdateInfo,
    ) -> Element<'a, Message> {
        let message = text(format!("Readfence {} is available", notice.version)).size(13);

        let download = button(text("Download").size(12).wrapping(text::Wrapping::None))
            .on_press(Message::OpenUpdatePage)
            .style(style_btn_primary)
            .padding([5, 14]);

        let dismiss = button(text("✕").size(11))
            .on_press(Message::DismissUpdate)
            .style(style_btn_ghost_dim)
            .padding([5, 9]);

        column![
            container(
                row![message, Space::new().width(Fill), download, dismiss]
                    .spacing(10)
                    .align_y(Center),
            )
            .width(Fill)
            .padding([7, 18])
            .style(style_update_banner),
            rule::horizontal(1),
        ]
        .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        // Breakpoints: the document meta needs the most room, then the long
        // button labels; below that the controls move to a second row.
        let width = self.window_width;
        let show_meta = width >= 1280.0;
        let full_labels = width >= 1040.0;
        let two_rows = width < 900.0;

        let title_chars = if full_labels {
            40
        } else if two_rows {
            24
        } else {
            20
        };
        let active_title = self
            .files
            .get(self.active)
            .map(|file| file_name(&file.path))
            .unwrap_or_else(|| "No document open".to_string());
        let active_title = truncate_label(&active_title, title_chars);

        let active_meta = self
            .files
            .get(self.active)
            .map(|file| {
                format!(
                    "{} words · {}",
                    word_count(&file.content),
                    truncate_path_tail(&parent_label(&file.path), 36)
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

        let sidebar_label = match (self.sidebar_visible, full_labels) {
            (true, true) => "Hide files",
            (false, true) => "Show files",
            (_, false) => "Files",
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

        let theme_picker = pick_list(
            App::available_themes(),
            Some(self.theme.clone()),
            Message::ThemeChanged,
        )
        .text_size(13)
        .padding([8, 10])
        .style(style_picker);
        let theme_picker = if full_labels {
            theme_picker
        } else {
            theme_picker.width(150)
        };

        let fs_label = match (self.fullscreen, full_labels) {
            (true, true) => "Exit full",
            (false, true) => "Fullscreen",
            (true, false) => "Exit",
            (false, false) => "Full",
        };
        let fs_btn = button(text(fs_label).size(13))
            .on_press(Message::ToggleFullscreen)
            .style(style_btn_ghost)
            .padding([8, 14]);

        let toolbar: Element<'_, Message> = if two_rows {
            column![
                row![brand, Space::new().width(Fill), open, sidebar_btn]
                    .spacing(8)
                    .align_y(Center),
                row![
                    seg,
                    font_row,
                    Space::new().width(Fill),
                    theme_picker,
                    fs_btn
                ]
                .spacing(8)
                .align_y(Center),
            ]
            .spacing(10)
            .padding([10, 14])
            .into()
        } else {
            let mut toolbar_row = row![brand];
            if show_meta {
                toolbar_row = toolbar_row.push(
                    container(
                        text(active_meta)
                            .size(12)
                            .style(|theme: &Theme| text::Style {
                                color: Some(Color {
                                    a: 0.58,
                                    ..theme.extended_palette().background.base.text
                                }),
                            }),
                    )
                    .padding([0, 10]),
                );
            }
            toolbar_row
                .push(Space::new().width(Fill))
                .push(open)
                .push(sidebar_btn)
                .push(seg)
                .push(font_row)
                .push(theme_picker)
                .push(fs_btn)
                .spacing(8)
                .padding([12, 18])
                .align_y(Center)
                .into()
        };

        column![
            container(toolbar).width(Fill).style(style_toolbar),
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

                let close = button(text("✕").size(11))
                    .on_press(Message::CloseFile(i))
                    .padding([8, 9])
                    .style(style_btn_ghost_dim);

                row![label, close].spacing(4).align_y(Center).into()
            })
            .collect();

        container(
            column![
                container(header).padding([14, 14]),
                scrollable(column(items).spacing(4).padding([8, 8]))
                    .style(style_scrollable)
                    .height(Fill),
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
                RenderedBlockKind::Table => SelectionRegion::table(index),
                RenderedBlockKind::Rule | RenderedBlockKind::Image { .. } => {
                    SelectionRegion::rule()
                }
                RenderedBlockKind::Quote => SelectionRegion::quote(index),
                _ => SelectionRegion::block(index),
            });
            elements.push(self.view_rendered_block(index, block));
        }
        let gaps = file
            .rendered_blocks
            .windows(2)
            .map(|pair| block_gap(&pair[0].kind, &pair[1].kind))
            .collect();
        let rendered_blocks = selection_group(elements, selection_regions, gaps);

        let document_header = column![
            row![
                column![
                    text(file_name(&file.path)).size(28).font(Font {
                        weight: iced::font::Weight::Bold,
                        ..Font::DEFAULT
                    }),
                    text(truncate_path_tail(&parent_label(&file.path), 46))
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
                button(text("Copy text").size(11).wrapping(text::Wrapping::None))
                    .on_press(Message::CopyRenderedText(file.rendered_text.clone()))
                    .style(style_btn_ghost)
                    .padding([5, 10]),
                badge("Markdown".to_string()),
            ]
            .spacing(8)
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

        // Cap the measure so long lines stay comfortable to read on wide
        // windows.
        let document =
            container(column![document_header, rule::horizontal(1), rendered_blocks,].spacing(26))
                .width(Fill)
                .max_width(900)
                .padding([34, 44])
                .style(style_panel);

        scrollable(
            container(document)
                .width(Fill)
                .center_x(Fill)
                .padding([28, 34]),
        )
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(8)
                .margin(2)
                .scroller_width(6),
        ))
        .style(style_scrollable)
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
            RenderedBlockKind::Table => self.view_selectable_table_block(index, block),
            RenderedBlockKind::Image { source, alt } => self.view_image_block(source, alt),
            RenderedBlockKind::Rule => container(rule::horizontal(1))
                .width(Fill)
                .padding([8, 0])
                .into(),
            kind => {
                let base = self.font_size.max(15.0);
                let size = match kind {
                    RenderedBlockKind::Heading(1) => base * 2.0,
                    RenderedBlockKind::Heading(2) => base * 1.5,
                    RenderedBlockKind::Heading(3) => base * 1.3,
                    RenderedBlockKind::Heading(4) => base * 1.15,
                    _ => base,
                };
                let font = match kind {
                    RenderedBlockKind::Heading(_) => Font {
                        weight: iced::font::Weight::Bold,
                        ..Font::DEFAULT
                    },
                    _ => Font::DEFAULT,
                };
                let line_height = match kind {
                    RenderedBlockKind::Heading(_) => 1.25,
                    RenderedBlockKind::ListItem => 1.45,
                    _ => 1.55,
                };

                let editor = text_editor(&block.content)
                    .on_action(move |action| Message::RenderedBlockAction(index, action))
                    .key_binding(rendered_key_binding)
                    .font(font)
                    .size(size)
                    .line_height(text::LineHeight::Relative(line_height))
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .padding([2, 0])
                    .style(style_selectable_prose)
                    .highlight_with::<SpanHighlighter>(
                        block.span_highlights(),
                        span_highlight_format,
                    );

                match kind {
                    // Section headings get a hairline underline, like a
                    // well-set document's running heads.
                    RenderedBlockKind::Heading(1) | RenderedBlockKind::Heading(2) => column![
                        editor,
                        container(Space::new().width(Fill).height(1))
                            .style(|theme: &Theme| container::Style {
                                background: Some(
                                    Color {
                                        a: 0.14,
                                        ..theme.extended_palette().background.base.text
                                    }
                                    .into(),
                                ),
                                ..Default::default()
                            })
                    ]
                    .spacing(8)
                    .into(),
                    // Quotes carry the accent as a left bar over a soft tint.
                    // An opaque inner panel over an accent outer one exposes a
                    // 3px strip as the bar; a Fill-height bar widget would
                    // blow up inside the scrollable's unbounded layout.
                    RenderedBlockKind::Quote => container(
                        container(editor)
                            .width(Fill)
                            .padding([8, 14])
                            .style(|theme: &Theme| {
                                let p = theme.extended_palette();
                                container::Style {
                                    background: Some(
                                        mix(surface_color(theme), p.primary.base.color, 0.08)
                                            .into(),
                                    ),
                                    border: Border {
                                        radius: iced::border::Radius {
                                            top_left: 0.0,
                                            top_right: 6.0,
                                            bottom_right: 6.0,
                                            bottom_left: 0.0,
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            }),
                    )
                    .width(Fill)
                    .padding(iced::Padding {
                        left: 3.0,
                        ..iced::Padding::ZERO
                    })
                    .style(|theme: &Theme| container::Style {
                        background: Some(theme.extended_palette().primary.base.color.into()),
                        border: Border {
                            radius: iced::border::Radius {
                                top_left: 3.0,
                                top_right: 6.0,
                                bottom_right: 6.0,
                                bottom_left: 3.0,
                            },
                            ..Default::default()
                        },
                        ..Default::default()
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
            .line_height(text::LineHeight::Relative(1.45))
            .wrapping(text::Wrapping::None)
            .padding([14, 16])
            .style(style_selectable_code);

        container(column![header, code_view])
            .width(Fill)
            .style(style_subtle_panel)
            .into()
    }

    fn view_image_block<'a>(&'a self, source: &'a ImageSource, alt: &'a str) -> Element<'a, Message> {
        // The document column tops out at 900 with 44px side padding.
        let max_width = (self.window_width - 200.0).clamp(320.0, 812.0);

        match source {
            ImageSource::Local(path) => {
                if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("svg")) {
                    match std::fs::read(path) {
                        Ok(bytes) => {
                            let size = svg_dimensions(&bytes);
                            sized_svg(svg::Handle::from_memory(bytes), size, max_width)
                        }
                        Err(_) => image_placeholder(alt, false),
                    }
                } else if path.exists() {
                    container(
                        image(image::Handle::from_path(path))
                            .content_fit(iced::ContentFit::ScaleDown),
                    )
                    .width(Fill)
                    .into()
                } else {
                    image_placeholder(alt, false)
                }
            }
            ImageSource::Remote(url) => match self.remote_images.get(url) {
                Some(RemoteImage::Raster(handle)) => container(
                    image(handle.clone()).content_fit(iced::ContentFit::ScaleDown),
                )
                .width(Fill)
                .into(),
                Some(RemoteImage::Vector {
                    handle,
                    width,
                    height,
                }) => sized_svg(handle.clone(), Some((*width, *height)), max_width),
                Some(RemoteImage::Loading) => image_placeholder(alt, true),
                Some(RemoteImage::Failed) | None => image_placeholder(alt, false),
            },
        }
    }

    fn view_selectable_table_block<'a>(
        &self,
        index: usize,
        block: &'a RenderedBlock,
    ) -> Element<'a, Message> {
        let editor = text_editor(&block.content)
            .on_action(move |action| Message::RenderedBlockAction(index, action))
            .key_binding(rendered_key_binding)
            .font(Font::MONOSPACE)
            .size((self.font_size.max(15.0) * 0.86).max(13.0))
            .line_height(text::LineHeight::Relative(1.5))
            .wrapping(text::Wrapping::None)
            .padding([12, 14])
            .style(style_selectable_prose)
            .highlight_with::<SpanHighlighter>(block.span_highlights(), span_highlight_format);

        container(editor)
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

/// Renders an SVG at its natural size, scaled down when it exceeds the
/// available measure. Without an intrinsic size the alt placeholder shows
/// instead, since an unsized SVG would stretch to fill the column.
fn sized_svg<'a>(
    handle: svg::Handle,
    size: Option<(f32, f32)>,
    max_width: f32,
) -> Element<'a, Message> {
    match size {
        Some((width, height)) if width > 0.0 && height > 0.0 => {
            let scale = (max_width / width).min(1.0);
            container(svg(handle).width(width * scale).height(height * scale))
                .width(Fill)
                .into()
        }
        _ => image_placeholder("SVG image", false),
    }
}

fn image_placeholder<'a>(alt: &str, loading: bool) -> Element<'a, Message> {
    let label = if loading {
        "Loading image…".to_string()
    } else if alt.is_empty() {
        "[image]".to_string()
    } else {
        format!("[{alt}]")
    };
    text(label)
        .size(14)
        .font(Font {
            style: iced::font::Style::Italic,
            ..Font::DEFAULT
        })
        .style(|theme: &Theme| text::Style {
            color: Some(Color {
                a: 0.5,
                ..theme.extended_palette().background.base.text
            }),
        })
        .into()
}

// Opaque blend of two colors, for tinted surfaces that must cover what is
// painted beneath them.
fn mix(base: Color, tint: Color, amount: f32) -> Color {
    Color {
        r: base.r + (tint.r - base.r) * amount,
        g: base.g + (tint.g - base.g) * amount,
        b: base.b + (tint.b - base.b) * amount,
        a: 1.0,
    }
}

// Keeps the end of a path, which carries the informative components.
fn truncate_path_tail(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        value.to_string()
    } else {
        let tail: String = value
            .chars()
            .skip(count - max_chars.saturating_sub(1))
            .collect();
        format!("…{tail}")
    }
}

fn truncate_label(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        let prefix: String = value.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}…", prefix.trim_end())
    }
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

// Vertical rhythm: headings hug the section they introduce, list items and
// quote lines read as one unit, panels and rules get room to breathe.
fn block_gap(previous: &RenderedBlockKind, next: &RenderedBlockKind) -> f32 {
    use RenderedBlockKind::{Code, Heading, Image, ListItem, Quote, Rule, Table};
    match (previous, next) {
        (ListItem, ListItem) => 7.0,
        (Quote, Quote) => 6.0,
        (Image { .. }, Image { .. }) => 8.0,
        (Heading(_), Heading(_)) => 18.0,
        (Heading(_), _) => 14.0,
        (_, Heading(1) | Heading(2)) => 34.0,
        (_, Heading(_)) => 28.0,
        (Code { .. } | Table, _) | (_, Code { .. } | Table) => 20.0,
        (Rule, _) | (_, Rule) => 22.0,
        _ => 18.0,
    }
}

fn span_highlight_format(style: &SpanStyle, theme: &Theme) -> highlighter::Format<Font> {
    let palette = theme.extended_palette();

    // One accent, spent on structure: links and markers take the theme
    // accent, frame glyphs and struck text recede, content stays in ink.
    let color = if style.link {
        Some(palette.primary.strong.color)
    } else if style.marker {
        Some(palette.primary.base.color)
    } else if style.strike || style.dim {
        Some(Color {
            a: 0.5,
            ..palette.background.base.text
        })
    } else {
        None
    };

    // A `Some` font replaces the editor's base font entirely, so only emit
    // one when the span actually changes family, weight, or slant.
    let font = (style.strong || style.emphasis || style.code).then_some(Font {
        family: if style.code {
            iced::font::Family::Monospace
        } else {
            Font::DEFAULT.family
        },
        weight: if style.strong {
            iced::font::Weight::Bold
        } else {
            iced::font::Weight::Normal
        },
        style: if style.emphasis {
            iced::font::Style::Italic
        } else {
            iced::font::Style::Normal
        },
        ..Font::DEFAULT
    });

    highlighter::Format { color, font }
}
