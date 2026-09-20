// PLAN-077 (auto-musk): `max-w-[N%]` 百分比上限 widget —— iced 的
// `Container::max_width` 只收像素值，CSS 的百分比 max-width（相对父级
// 内容盒）没有原生承载。本 widget 纯委托包装：只在 layout 期用父级
// offered limits（limits.max().width 即父级分配宽度）按 fraction 收窄
// limits.max_width，再交回内层——Shrink 内容不受影响，Fill 内容被压到
// 父宽 × fraction。px 形态 max-width 仍走 Container::max_width 快路径，
// 本 widget 仅百分比形态构造（renderer 的 col/row/container 臂尾部）。

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Rectangle, Size, Vector};

/// 百分比 max-width 委托包装。`fraction` = 百分数/100（0.7 = 70%）。
pub struct MaxWidthPct<'a, Message> {
    content: Element<'a, Message>,
    fraction: f32,
}

impl<'a, Message> MaxWidthPct<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>, fraction: f32) -> Self {
        Self {
            content: content.into(),
            fraction: fraction.clamp(0.0, 1.0),
        }
    }
}

/// layout 期收窄核心（纯函数，便于单测）：父级 offered 宽度有限时返回
/// 收窄后的 max_width，否则原样返回（无法得知父宽，保持不约束）。
pub fn capped_max_width(offered: f32, fraction: f32, min_width: f32) -> Option<f32> {
    if offered.is_finite() && offered > 0.0 {
        Some((offered * fraction.clamp(0.0, 1.0)).max(min_width))
    } else {
        None
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for MaxWidthPct<'_, Message>
where
    Message: Clone + 'static,
{
    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<iced::Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = match capped_max_width(limits.max().width, self.fraction, limits.min().width) {
            Some(cap) => &limits.max_width(cap),
            None => limits,
        };
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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

    fn draw(
        &self,
        tree: &widget::Tree,
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

    // PLAN-080 T-01: operate 委托——iced Operation（LayoutCollector bounds
    // 收集 / iced_test 文本选择器）默认 no-op 不进子件，MCP 实机几何与
    // headless 断言此前都量不到被本 widget 包住的节点（080 定罪实证）。
    // 转发模式同 popover Panel::operate（container 上报 + traverse 递归）。
    // UAT F-UAT-1 勘误：裸递归不带 Operation::traverse 作用域——子树里的
    // text 节点对 iced_selector/MCP 快照操作恒不上报（文本选择器只见
    // container 见不到 text，bisect 实证）。traverse 递归与 popover 同款。
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        let content_layout = layout.children().next().expect("max-width pct child");
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                content_layout,
                renderer,
                operation,
            );
        });
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

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'a, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<MaxWidthPct<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(widget: MaxWidthPct<'a, Message>) -> Self {
        Self::new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_relative_to_offered_width() {
        assert_eq!(capped_max_width(1000.0, 0.7, 0.0), Some(700.0));
        assert_eq!(capped_max_width(2560.0, 0.7, 0.0), Some(1792.0));
    }

    #[test]
    fn never_below_parent_min_width() {
        assert_eq!(capped_max_width(100.0, 0.1, 50.0), Some(50.0));
    }

    #[test]
    fn unbounded_parent_leaves_no_cap() {
        assert_eq!(capped_max_width(f32::INFINITY, 0.7, 0.0), None);
        assert_eq!(capped_max_width(0.0, 0.7, 0.0), None);
    }

    #[test]
    fn fraction_is_clamped() {
        assert_eq!(capped_max_width(1000.0, 1.5, 0.0), Some(1000.0));
        assert_eq!(capped_max_width(1000.0, -0.5, 0.0), Some(0.0));
    }
}
