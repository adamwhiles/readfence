use crate::messages::Message;
use iced::advanced::layout;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Layout, Shell};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse};

const DEFAULT_GAP: f32 = 16.0;

#[derive(Debug, Clone, Copy)]
pub struct SelectionRegion {
    pub block: Option<usize>,
    pub top_inset: f32,
    pub left_inset: f32,
}

impl SelectionRegion {
    pub const fn block(index: usize) -> Self {
        Self {
            block: Some(index),
            top_inset: 2.0,
            left_inset: 0.0,
        }
    }

    pub const fn quote(index: usize) -> Self {
        // Accent bar (3) + inner padding (14, 8).
        Self {
            block: Some(index),
            top_inset: 11.0,
            left_inset: 17.0,
        }
    }

    pub const fn code(index: usize) -> Self {
        Self {
            block: Some(index),
            top_inset: 52.0,
            left_inset: 16.0,
        }
    }

    pub const fn table(index: usize) -> Self {
        Self {
            block: Some(index),
            top_inset: 14.0,
            left_inset: 14.0,
        }
    }

    pub const fn rule() -> Self {
        Self {
            block: None,
            top_inset: 0.0,
            left_inset: 0.0,
        }
    }
}

pub fn selection_group<'a>(
    children: Vec<Element<'a, Message>>,
    regions: Vec<SelectionRegion>,
    gaps: Vec<f32>,
) -> SelectionGroup<'a> {
    SelectionGroup {
        children,
        regions,
        gaps,
    }
}

pub struct SelectionGroup<'a> {
    children: Vec<Element<'a, Message>>,
    regions: Vec<SelectionRegion>,
    /// Vertical gap above each child after the first (`gaps[i]` sits between
    /// child `i` and child `i + 1`).
    gaps: Vec<f32>,
}

#[derive(Default)]
struct State {
    anchor: Option<usize>,
    press_position: Option<Point>,
    moved: bool,
    focused_range: Option<(usize, usize)>,
}

impl Widget<Message, iced::Theme, iced::Renderer> for SelectionGroup<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let child_limits = layout::Limits::new(Size::ZERO, max);

        let mut nodes = Vec::with_capacity(self.children.len());
        let mut y = 0.0;
        for (index, (child, state)) in
            self.children.iter_mut().zip(&mut tree.children).enumerate()
        {
            if index > 0 {
                y += self.gaps.get(index - 1).copied().unwrap_or(DEFAULT_GAP);
            }
            let node = child
                .as_widget_mut()
                .layout(state, renderer, &child_limits)
                .move_to(Point::new(0.0, y));
            y += node.size().height;
            nodes.push(node);
        }

        let size = limits.resolve(Length::Fill, Length::Shrink, Size::new(max.width, y));
        layout::Node::with_children(size, nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
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
        self.update_selection(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );

        for ((child, state), child_layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                state,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
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
        for ((child, state), child_layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().intersects(viewport))
        {
            child.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl SelectionGroup<'_> {
    #[allow(clippy::too_many_arguments)]
    fn update_selection(
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
        let Some(position) = cursor.position() else {
            return;
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some((block, _, _)) = self.block_at(layout, position) {
                    let state = tree.state.downcast_mut::<State>();
                    state.anchor = Some(block);
                    state.press_position = Some(position);
                    state.moved = false;
                    state.focused_range = Some((block, block));
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let (anchor, focused_range) = {
                    let state = tree.state.downcast_mut::<State>();
                    let Some(anchor) = state.anchor else {
                        return;
                    };

                    if let Some(start) = state.press_position
                        && start.distance(position) > 3.0
                    {
                        state.moved = true;
                    }

                    (anchor, state.focused_range)
                };

                let Some((target, point, _)) = self.block_at(layout, position) else {
                    return;
                };
                if target == anchor {
                    return;
                }

                let range = (anchor.min(target), anchor.max(target));
                if focused_range != Some(range) {
                    self.focus_range(tree, layout, range, renderer, clipboard, shell, viewport);
                    tree.state.downcast_mut::<State>().focused_range = Some(range);
                }

                shell.publish(Message::RenderedCrossBlockSelection {
                    anchor,
                    target,
                    point,
                });
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let state = tree.state.downcast_mut::<State>();
                if let Some(anchor) = state.anchor.take()
                    && !state.moved
                {
                    shell.publish(Message::RenderedBlockClicked(anchor));
                }
                state.press_position = None;
                state.moved = false;
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn focus_range(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        range: (usize, usize),
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let layouts = layout.children().collect::<Vec<_>>();

        for (child_index, region) in self.regions.iter().enumerate() {
            let Some(block) = region.block else {
                continue;
            };
            if block < range.0 || block > range.1 {
                if let Some(child_layout) = layouts.get(child_index).copied() {
                    self.children[child_index].as_widget_mut().update(
                        &mut tree.children[child_index],
                        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                        child_layout,
                        mouse::Cursor::Unavailable,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
                continue;
            }

            let Some(child_layout) = layouts.get(child_index).copied() else {
                continue;
            };
            let bounds = child_layout.bounds();
            let position = Point::new(
                bounds.x + region.left_inset + 1.0,
                bounds.y + region.top_inset + 1.0,
            );
            let synthetic_cursor = mouse::Cursor::Available(position);

            for synthetic_event in [
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            ] {
                self.children[child_index].as_widget_mut().update(
                    &mut tree.children[child_index],
                    &synthetic_event,
                    child_layout,
                    synthetic_cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        }
    }

    fn block_at(&self, layout: Layout<'_>, position: Point) -> Option<(usize, Point, usize)> {
        let children = layout.children().collect::<Vec<_>>();
        let selectable = children
            .iter()
            .zip(&self.regions)
            .enumerate()
            .filter_map(|(index, (child, region))| {
                region
                    .block
                    .map(|block| (child.bounds(), *region, block, index))
            })
            .collect::<Vec<_>>();

        let (bounds, region, block, child_index) = selectable
            .iter()
            .find(|(bounds, _, _, _)| bounds.contains(position))
            .or_else(|| {
                selectable.iter().min_by(|(a, _, _, _), (b, _, _, _)| {
                    vertical_distance(*a, position.y).total_cmp(&vertical_distance(*b, position.y))
                })
            })?;

        Some((
            *block,
            Point::new(
                (position.x - bounds.x - region.left_inset).max(0.0),
                (position.y - bounds.y - region.top_inset).max(0.0),
            ),
            *child_index,
        ))
    }
}

fn vertical_distance(bounds: Rectangle, y: f32) -> f32 {
    if y < bounds.y {
        bounds.y - y
    } else if y > bounds.y + bounds.height {
        y - (bounds.y + bounds.height)
    } else {
        0.0
    }
}

impl<'a> From<SelectionGroup<'a>> for Element<'a, Message> {
    fn from(group: SelectionGroup<'a>) -> Self {
        Element::new(group)
    }
}
