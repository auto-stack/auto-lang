// PLAN-002 B: 布局件 hover 态承载 widget —— 布局件（row/col/div）`hover:`
// 变体类的 iced 消费面。
//
// 为什么需要它:iced 的 container/row/column 没有 hover 状态回调(button/svg
// 有 Status,布局件没有),`hover:` 类此前只有 Button/SVG 臂消费——row/col 上
// 的 `hover:bg-*` 被静默丢弃(526 Q9① 实测;KNOWN-DEBT 526 行)。两条既有
// 路线都不合用:
//   - mouse_area on_enter/on_exit 发消息 → 每次 hover 翻转重建整棵 view 树
//     (大列表悬停代价不可接受);
//   - iced container 的样式闭包在 draw 期求值但拿不到 hover 态。
// 本 widget 取 PointerArea/table_resize 先例:自持 hover 态(tree::State),
// 与样式闭包**共享一个 Arc<AtomicBool> 标志**——闭包读标志在 base/hover 两套
// 已构建样式间二选一,状态翻转只发 request_redraw(重绘,不重建、不发消息)。
// 这与 iced 原生 button 的 Status 悬停开销同档。
//
// 纯委托包装:不捕获事件、不改变内层语义,只观察游标位置维护 hover 态
// (enter/exit/dblclick 等仍由内层 mouse_area 承担)。inspect 捕获态由调用面
// (renderer::wrap_layout_events)自守卫——捕获态不构造本 widget,标志恒 false。

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Rectangle, Size, Vector};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 布局件 hover 标志:样式闭包与 HoverArea 共享(见模块头注)。构造于
/// renderer 的 row/col/container 臂(仅当样式声明了 `hover:` 变体类)。
pub type HoverFlag = Arc<AtomicBool>;

/// HoverArea 的本地状态(tree::Tag 标识)。hovered 跨帧存活(重建后由 draw
/// 回写当前帧标志——新闭包捕获的是新 Arc,须重新同步);cursor_position/
/// bounds 用于跳过无变化事件的重复求值(mouse_area 同款闸)。
#[derive(Debug, Default)]
pub struct State {
    hovered: bool,
    cursor_position: Option<iced::Point>,
    bounds: Rectangle,
}

/// hover 态包装 widget。`flag` 与承载样式闭包的那个 Arc 是同一实例。
pub struct HoverArea<'a, Message> {
    content: Element<'a, Message>,
    flag: HoverFlag,
}

impl<'a, Message> HoverArea<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>, flag: HoverFlag) -> Self {
        Self {
            content: content.into(),
            flag,
        }
    }
}

/// hover 态转移核心(纯函数,便于单测):给定上一次 hover 态与本次命中结果,
/// 返回 (新状态, 是否变化)。无变化不触发重绘。
pub fn hover_transition(was_hovered: bool, is_over: bool) -> (bool, bool) {
    (is_over, was_hovered != is_over)
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for HoverArea<'_, Message>
where
    Message: Clone + 'static,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
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
        // hover 态维护叠加在事件转发之上(不捕获,不改变内层语义)。
        let bounds = layout.bounds();
        let cursor_position = cursor.position();
        {
            let state = tree.state.downcast_mut::<State>();
            if state.cursor_position != cursor_position || state.bounds != bounds {
                state.cursor_position = cursor_position;
                state.bounds = bounds;
                let (hovered, changed) = hover_transition(state.hovered, cursor.is_over(bounds));
                if changed {
                    state.hovered = hovered;
                    self.flag.store(hovered, Ordering::Relaxed);
                    // 只请求重绘(样式闭包在 draw 期读标志)——不重建 view、
                    // 不发消息(hover 重建整树是大列表悬停的性能陷阱)。
                    shell.request_redraw();
                }
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
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        // 重建后新闭包捕获的是新 Arc,标志默认 false——每帧从状态回写,
        // 保证 hover 视觉不因重建而闪断。
        let state = tree.state.downcast_ref::<State>();
        self.flag.store(state.hovered, Ordering::Relaxed);

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<HoverArea<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(widget: HoverArea<'a, Message>) -> Self {
        Self::new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::hover_transition;

    #[test]
    fn transition_fires_only_on_change() {
        // 进入:翻转 + 变化。
        assert_eq!(hover_transition(false, true), (true, true));
        // 停留在内:无变化(不重绘)。
        assert_eq!(hover_transition(true, true), (true, false));
        // 离开:翻转 + 变化。
        assert_eq!(hover_transition(true, false), (false, true));
        // 停留在外:无变化。
        assert_eq!(hover_transition(false, false), (false, false));
    }
}
