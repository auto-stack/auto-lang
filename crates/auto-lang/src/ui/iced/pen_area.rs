// Plan 563: pen 事件层 widget —— canvas 元素 onpenstart/onpenmove/onpenend
// 的 iced 承载(PointerArea〔499 M2〕同型扩展)。
//
// 为什么不用 iced 原生 `mouse_area` 组合(T1d 源码实证,iced_widget 0.14
// mouse_area.rs):三硬缺口——
//   1. on_press/on_release 是 `Option<Message>` 固定消息,**不带坐标**
//      (penstart/penend 需要 (x, y));
//   2. on_move 仅 hover 期派发(`if state.is_hovered`),无按下态门控
//      (penmove 仅 pen-down 期间派发);
//   3. `if !cursor.is_over(bounds) { return; }` 早退在事件 match 之前——
//      按住移出组件后 move/release 均丢失(窗外事件不可见)。
// 本 widget 在事件现场持 layout bounds,自己维护按下态,语义自洽:
//   - ButtonPressed(界内)→ onpenstart(逻辑坐标) + is_down + capture;
//   - CursorMoved(is_down 且界内)→ ≤30Hz 限频 + 量化去重 → onpenmove;
//   - CursorMoved(is_down 且**出界**)→ onpenend(最后已知界内逻辑坐标)
//     + is_down=false —— **离开画布即收笔**(双端统一语义,/vue 端
//     document-mousemove bounds 检查对齐);
//   - ButtonReleased(is_down)→ onpenend(界内当前坐标;出界则最后已知)。
// 坐标换算与限频复用 pointer_area::{to_logical, should_publish}(场景
// 数据契约的同一映射规约)。

use crate::ui::iced::pointer_area::{should_publish, to_logical};
use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Point, Rectangle, Size, Vector};
use std::sync::Arc;
use std::time::Instant;

type PenFn<Message> = Arc<dyn Fn(f32, f32) -> Message + Send + Sync>;

/// PenArea 的本地状态(tree::Tag 标识)。throttle 复用 PointerArea 的
/// 限频核心(33ms 时间闸 + 0.5px 量化去重);last_logical 同时承担
/// "出界收笔"的最后已知界内坐标。
#[derive(Debug, Default)]
pub struct State {
    is_down: bool,
    throttle: super::pointer_area::State,
}

pub struct PenArea<'a, Message>
where
    Message: Clone + 'static,
{
    content: Element<'a, Message>,
    extent: Option<(f32, f32)>,
    on_pen_start: Option<PenFn<Message>>,
    on_pen_move: Option<PenFn<Message>>,
    on_pen_end: Option<PenFn<Message>>,
}

impl<'a, Message> PenArea<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            extent: None,
            on_pen_start: None,
            on_pen_move: None,
            on_pen_end: None,
        }
    }

    /// 逻辑幅面(coords prop "WxH" 解析结果;None = raw px)。
    pub fn extent(mut self, w: f32, h: f32) -> Self {
        self.extent = Some((w, h));
        self
    }

    pub fn on_pen_start(mut self, f: PenFn<Message>) -> Self {
        self.on_pen_start = Some(f);
        self
    }

    pub fn on_pen_move(mut self, f: PenFn<Message>) -> Self {
        self.on_pen_move = Some(f);
        self
    }

    pub fn on_pen_end(mut self, f: PenFn<Message>) -> Self {
        self.on_pen_end = Some(f);
        self
    }

    /// 出界/窗外收笔:最后已知界内逻辑坐标(无记录 → bounds 左上角)。
    fn last_known(state: &State, bounds: &Rectangle, extent: &Option<(f32, f32)>) -> (f32, f32) {
        state
            .throttle
            .last_logical
            .unwrap_or_else(|| to_logical(Point::ORIGIN, bounds, *extent))
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for PenArea<'_, Message>
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
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                if let (Some(f), true) = (&self.on_pen_start, cursor.is_over(bounds)) {
                    if let Some(local) = cursor.position_in(bounds) {
                        let logical = to_logical(local, &bounds, self.extent);
                        state.is_down = true;
                        // 起笔即重置限频窗(新笔画首 move 不受上笔尾闸压制)。
                        state.throttle = super::pointer_area::State::default();
                        state.throttle.last_logical = Some(logical);
                        state.throttle.last_pub = Some(Instant::now());
                        shell.publish(f(logical.0, logical.1));
                        // 按下即绘画:吞掉事件,防外层 scrollable 抢拖动。
                        shell.capture_event();
                    }
                }
            }
            Event::Mouse(iced::mouse::Event::CursorMoved { .. }) => {
                if state.is_down {
                    match cursor.position_in(bounds) {
                        Some(local) => {
                            let logical = to_logical(local, &bounds, self.extent);
                            if let Some(f) = &self.on_pen_move {
                                if should_publish(&mut state.throttle, logical, Instant::now()) {
                                    shell.publish(f(logical.0, logical.1));
                                }
                            } else {
                                // 无 move 消费者仍要记账(收笔坐标用)。
                                state.throttle.last_logical = Some(logical);
                            }
                        }
                        None => {
                            // 出界即收笔(T1d 双端统一语义)。
                            if let Some(f) = &self.on_pen_end {
                                let last = Self::last_known(state, &bounds, &self.extent);
                                shell.publish(f(last.0, last.1));
                            }
                            state.is_down = false;
                        }
                    }
                }
            }
            Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                if state.is_down {
                    if let Some(f) = &self.on_pen_end {
                        let logical = cursor
                            .position_in(bounds)
                            .map(|local| to_logical(local, &bounds, self.extent))
                            .unwrap_or_else(|| Self::last_known(state, &bounds, &self.extent));
                        shell.publish(f(logical.0, logical.1));
                    }
                    state.is_down = false;
                }
            }
            _ => {}
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let child = tree.children.iter_mut().next();
        let child_layout = layout;
        self.content.as_widget_mut().overlay(
            child.unwrap(),
            child_layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<PenArea<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(area: PenArea<'a, Message>) -> Self {
        Element::new(area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_known_falls_back_to_origin() {
        // 无任何 move 记录时的收笔坐标兜底(bounds 左上角逻辑值)。
        let s = State::default();
        let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(400.0, 200.0));
        let last = PenArea::<()>::last_known(&s, &bounds, &Some((200.0, 100.0)));
        assert_eq!(last, (0.0, 0.0));
    }
}
