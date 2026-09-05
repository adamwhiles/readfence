//! A transparent wrapper that turns Ctrl+wheel into zoom steps before the
//! scrollable underneath can scroll on them.

use crate::messages::Message;
use iced::advanced::layout;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Layout, Shell};
use iced::{Element, Event, Length, Rectangle, Size, Vector, keyboard, mouse};

/// Touchpads report pixel deltas; this many pixels count as one notch.
const PIXELS_PER_STEP: f32 = 40.0;

pub fn zoom_area<'a>(
    content: impl Into<Element<'a, Message>>,
    on_zoom: fn(i32) -> Message,
) -> ZoomArea<'a> {
    ZoomArea {
        content: content.into(),
        on_zoom,
    }
}

pub struct ZoomArea<'a> {
    content: Element<'a, Message>,
    on_zoom: fn(i32) -> Message,
}

#[derive(Default)]
struct State {
    modifiers: keyboard::Modifiers,
    partial: f32,
}

impl Widget<Message, iced::Theme, iced::Renderer> for ZoomArea<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        {
            let state = tree.state.downcast_mut::<State>();
            match event {
                Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                    state.modifiers = *modifiers;
                }
                Event::Mouse(mouse::Event::WheelScrolled { delta })
                    if (state.modifiers.control() || state.modifiers.command())
                        && cursor.is_over(layout.bounds()) =>
                {
                    let steps = match *delta {
                        mouse::ScrollDelta::Lines { y, .. } => y,
                        mouse::ScrollDelta::Pixels { y, .. } => y / PIXELS_PER_STEP,
                    };
                    state.partial += steps;
                    let whole = state.partial.trunc();
                    state.partial -= whole;
                    if whole != 0.0 {
                        shell.publish((self.on_zoom)(whole as i32));
                    }
                    // Swallow the wheel so the document does not scroll
                    // while it is being zoomed.
                    shell.capture_event();
                    return;
                }
                _ => {}
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a> From<ZoomArea<'a>> for Element<'a, Message> {
    fn from(area: ZoomArea<'a>) -> Self {
        Element::new(area)
    }
}
