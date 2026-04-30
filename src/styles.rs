use iced::{Border, Color, Shadow, Theme, Vector};
use iced::widget::button;

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
            radius: 6.0.into(),
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

pub fn style_btn_seg(theme: &Theme, status: button::Status) -> button::Style {
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

pub fn style_btn_seg_active(theme: &Theme, _status: button::Status) -> button::Style {
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

pub fn style_file_active(theme: &Theme, status: button::Status) -> button::Style {
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

pub fn style_file_inactive(theme: &Theme, status: button::Status) -> button::Style {
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