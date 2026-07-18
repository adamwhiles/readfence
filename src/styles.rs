use iced::widget::{button, container, pick_list, scrollable, text_editor};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// The color documents sit on: dark themes read on a raised lighter panel,
/// light themes read on a white page over a grey canvas.
pub fn surface_color(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    if p.is_dark {
        p.background.weak.color
    } else {
        p.background.base.color
    }
}

pub fn style_app_background(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    let canvas = if p.is_dark {
        p.background.base.color
    } else {
        p.background.weak.color
    };
    container::Style {
        background: Some(canvas.into()),
        text_color: Some(p.background.base.text),
        ..Default::default()
    }
}

pub fn style_toolbar(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(p.background.weak.color.into()),
        text_color: Some(p.background.base.text),
        ..Default::default()
    }
}

pub fn style_sidebar(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(p.background.weak.color.into()),
        text_color: Some(p.background.base.text),
        ..Default::default()
    }
}

pub fn style_panel(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    let (shadow_alpha, blur) = if p.is_dark { (0.28, 24.0) } else { (0.10, 16.0) };
    container::Style {
        background: Some(surface_color(theme).into()),
        border: Border {
            radius: 10.0.into(),
            color: Color {
                a: if p.is_dark { 0.34 } else { 0.55 },
                ..p.background.strong.color
            },
            width: 1.0,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, shadow_alpha),
            offset: Vector::new(0.0, 6.0),
            blur_radius: blur,
        },
        ..Default::default()
    }
}

pub fn style_subtle_panel(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(
            Color {
                a: 0.42,
                ..p.background.weak.color
            }
            .into(),
        ),
        border: Border {
            radius: 8.0.into(),
            color: Color {
                a: 0.28,
                ..p.background.strong.color
            },
            width: 1.0,
        },
        ..Default::default()
    }
}

pub fn style_selectable_prose(theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    let p = theme.extended_palette();
    text_editor::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            width: 0.0,
            ..Default::default()
        },
        placeholder: Color {
            a: 0.48,
            ..p.background.base.text
        },
        value: p.background.base.text,
        selection: Color {
            a: 0.42,
            ..p.primary.base.color
        },
    }
}

pub fn style_selectable_code(theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    let p = theme.extended_palette();
    text_editor::Style {
        background: Background::Color(p.background.strong.color),
        border: Border {
            width: 0.0,
            ..Default::default()
        },
        placeholder: Color {
            a: 0.48,
            ..p.background.base.text
        },
        value: p.background.base.text,
        selection: Color {
            a: 0.46,
            ..p.primary.base.color
        },
    }
}

pub fn style_badge(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(
            Color {
                a: 0.16,
                ..p.primary.base.color
            }
            .into(),
        ),
        text_color: Some(p.primary.base.color),
        border: Border {
            radius: 8.0.into(),
            color: Color {
                a: 0.22,
                ..p.primary.base.color
            },
            width: 1.0,
        },
        ..Default::default()
    }
}

pub fn style_picker(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let p = theme.extended_palette();
    let hovered = !matches!(status, pick_list::Status::Active);
    pick_list::Style {
        text_color: p.background.base.text,
        placeholder_color: Color {
            a: 0.5,
            ..p.background.base.text
        },
        handle_color: Color {
            a: 0.55,
            ..p.background.base.text
        },
        background: Background::Color(if hovered {
            p.background.strong.color
        } else {
            Color {
                a: 0.42,
                ..p.background.weak.color
            }
        }),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color {
                a: 0.28,
                ..p.background.strong.color
            },
        },
    }
}

pub fn style_scrollable(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let p = theme.extended_palette();
    let hovered = !matches!(status, scrollable::Status::Active { .. });
    let scroller_alpha = if hovered { 0.35 } else { 0.16 };
    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(Color {
                a: scroller_alpha,
                ..p.background.base.text
            }),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
        },
    };

    let mut style = scrollable::default(theme, status);
    style.vertical_rail = rail;
    style.horizontal_rail = rail;
    style
}

pub fn style_update_banner(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    let base = if p.is_dark {
        p.background.base.color
    } else {
        p.background.weak.color
    };
    container::Style {
        background: Some(
            Color {
                r: base.r + (p.primary.base.color.r - base.r) * 0.14,
                g: base.g + (p.primary.base.color.g - base.g) * 0.14,
                b: base.b + (p.primary.base.color.b - base.b) * 0.14,
                a: 1.0,
            }
            .into(),
        ),
        text_color: Some(p.background.base.text),
        ..Default::default()
    }
}

pub fn style_status_bar(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(p.background.weak.color.into()),
        text_color: Some(Color {
            a: 0.68,
            ..p.background.base.text
        }),
        ..Default::default()
    }
}

pub fn style_btn_primary(theme: &Theme, status: button::Status) -> button::Style {
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
            radius: 8.0.into(),
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        snap: false,
    }
}

pub fn style_btn_ghost(theme: &Theme, status: button::Status) -> button::Style {
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
            radius: 8.0.into(),
            color: match status {
                button::Status::Hovered => Color {
                    a: 0.26,
                    ..p.background.base.text
                },
                _ => Color::TRANSPARENT,
            },
            width: 1.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn style_btn_ghost_dim(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let (bg, fg) = match status {
        button::Status::Hovered => (p.background.strong.color, p.background.base.text),
        button::Status::Pressed => (p.background.base.color, p.background.base.text),
        _ => (
            Color::TRANSPARENT,
            Color {
                a: 0.45,
                ..p.background.base.text
            },
        ),
    };
    button::Style {
        background: Some(bg.into()),
        text_color: fg,
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn style_btn_seg(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered => p.background.base.color,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(bg.into()),
        text_color: Color {
            a: 0.55,
            ..p.background.base.text
        },
        border: Border {
            radius: 7.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn style_btn_seg_active(theme: &Theme, _status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    button::Style {
        background: Some(p.background.base.color.into()),
        text_color: p.background.base.text,
        border: Border {
            radius: 7.0.into(),
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

pub fn style_file_active(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color {
            a: 0.24,
            ..p.primary.base.color
        },
        _ => Color {
            a: 0.16,
            ..p.primary.base.color
        },
    };
    button::Style {
        background: Some(bg.into()),
        text_color: p.primary.base.color,
        border: Border {
            radius: 8.0.into(),
            color: Color {
                a: 0.22,
                ..p.primary.base.color
            },
            width: 1.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn style_file_inactive(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let (bg, fg) = match status {
        button::Status::Hovered => (p.background.base.color, p.background.base.text),
        button::Status::Pressed => (p.background.strong.color, p.background.base.text),
        _ => (
            Color::TRANSPARENT,
            Color {
                a: (p.background.base.text.a * 0.85).max(0.7),
                ..p.background.base.text
            },
        ),
    };
    button::Style {
        background: Some(bg.into()),
        text_color: fg,
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
