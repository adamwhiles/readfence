  use crate::app::{App, OpenFile, ViewMode};
  use crate::messages::Message;
  use crate::styles::{
      style_btn_ghost, style_btn_ghost_dim, style_btn_primary,
      style_btn_seg, style_btn_seg_active,
      style_file_active, style_file_inactive,
  };
  use iced::{
      Border, Center, Color, Element, Fill, Font, Theme,
      widget::{
          button, column, container, markdown, pick_list,
          row, rule, scrollable, text, text_editor, Space,
      },
  };

  impl App {
      pub fn view(&self) -> Element<'_, Message> {
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
          let open = button(text("Open").size(13))
              .on_press(Message::OpenDialog)
              .style(style_btn_primary)
              .padding([7, 16]);

          let sidebar_label = if self.sidebar_visible { "Hide Sidebar" } else { "Sidebar" };
          let sidebar_btn = button(text(sidebar_label).size(13))
              .on_press(Message::ToggleSidebar)
              .style(style_btn_ghost)
              .padding([7, 14]);

          let (rendered_style, source_style) = match self.view_mode {
              ViewMode::Rendered => (style_btn_seg_active as fn(&Theme, button::Status) ->
  button::Style, style_btn_seg as fn(&Theme, button::Status) -> button::Style),
              ViewMode::Source   => (style_btn_seg         as fn(&Theme, button::Status) ->
  button::Style, style_btn_seg_active as fn(&Theme, button::Status) -> button::Style),
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

  pub fn shortcut_hint<'a>(key: &'a str, desc: &'a str) -> Element<'a, Message> {
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