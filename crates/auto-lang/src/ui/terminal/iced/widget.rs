// PLAN-009 P1 ② iced adapter — the `Terminal` widget.
//
// The only iced dependency point of the terminal component (hard layering:
// mod.rs never imports iced). T2 renders the placeholder rect (dark
// background + border + geometry label); T3 replaces this with grid glyph
// rendering + damage gating, T4 adds interaction.

use iced::advanced::widget::{tree, Tree};
use iced::advanced::{layout::Node, mouse, renderer, widget::Widget, Clipboard, Layout, Shell};
use iced::advanced::Renderer as _;
use iced::{Background, Border, Color, Element, Length, Rectangle, Size, Theme};

use crate::ui::terminal::TerminalCore;

/// Placeholder cell metrics (monospace 8×16 — real metrics arrive with the
/// T3 glyph pipeline / cosmic-text integration).
const CELL_W: f32 = 8.0;
const CELL_H: f32 = 16.0;
const BORDER: f32 = 1.0;

/// Terminal widget — draws the engine grid carried by `core`.
pub struct Terminal {
    core: &'static TerminalCore,
    width: Length,
    height: Length,
}

impl Terminal {
    /// Get-or-create the terminal state for `key` (geometry diffed in).
    pub fn new(key: &str, cols: u16, rows: u16) -> Self {
        Self {
            core: crate::ui::terminal::terminal(key, cols, rows),
            width: Length::Fixed(cols as f32 * CELL_W + 2.0 * BORDER),
            height: Length::Fixed(rows as f32 * CELL_H + 2.0 * BORDER),
        }
    }

    /// Override the suggested size (defaults to the grid's cell metrics).
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Terminal
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TerminalState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TerminalState {
            generation: self.core.generation(),
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        let limits = limits.width(self.width).height(self.height);
        Node::new(limits.resolve(self.width, self.height, Size::default()))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        // T2 placeholder: dark canvas + border (真网格渲染 T3 落地)。
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: Color::from_rgb(0.25, 0.28, 0.32),
                    width: BORDER.into(),
                    radius: 0.0.into(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(Color::from_rgb(0.06, 0.07, 0.09)),
        );
        // Suppress the unused-state warning until T3 consumes generations.
        let _ = tree.state.downcast_ref::<TerminalState>().generation;
    }
}

#[derive(Debug, Default, Clone)]
pub struct TerminalState {
    /// Last drawn content generation (damage gating lands in T3).
    pub generation: u64,
}

impl<'a, Message> From<Terminal> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(widget: Terminal) -> Self {
        Self::new(widget)
    }
}
