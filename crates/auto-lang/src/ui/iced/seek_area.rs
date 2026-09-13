//! 可拖拽进度条的指针承载（`progress` 的 `onseek` prop 在 iced 端的落点）。
//!
//! 为什么需要它：iced 原生 `mouse_area.on_press` **不带坐标**，而进度条的
//! 「点哪儿跳哪儿」必须知道指针在条内的横向位置。`PointerArea`（Plan 499）
//! 虽持有 bounds，但它的契约是「按 `coords` 声明把 px 换算成**逻辑坐标**并
//! 限频发布」——面向的是坐标流（画布/图表），不是「条内比例」。
//!
//! 本 widget 是一个**薄包装**（precedent: `pointer_area.rs` / `popover.rs`）：
//!
//! * `on_seek` 存在时：按下（bounds 内）与按住拖动都发布 **0..1 的横向比例**
//!   （`local_x / bounds.width`，出界钳到端点——拖出条外仍跟手）；
//! * 其余事件一律**原样转发**给内部元素，不捕获、不改变内层语义；
//! * 未启用时零开销（不发布、不拦截）。
//!
//! 「按下」与「拖动」的分界靠本 widget 自己记按下态（`State::pressed`）——
//! iced 的 `CursorMoved` 不携带按键状态，而这是「悬停不 scrub、按住才 scrub」
//! 的唯一判据。

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Rectangle, Size, Vector};
use std::sync::Arc;

#[derive(Default)]
struct State {
    /// 左键是否在条内按下且尚未抬起。
    pressed: bool,
}

pub struct SeekArea<'a, Message> {
    content: Element<'a, Message>,
    on_seek: Option<Arc<dyn Fn(f32) -> Message + Send + Sync>>,
}

impl<'a, Message> SeekArea<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            on_seek: None,
        }
    }

    /// 回调收 **0..1 的横向比例**（不是像素、也不是 `value`/`max` 刻度的值）。
    /// 两端同尺：Vue 侧同样交比例，作者写 `SeekTo(.duration * $0)` 即可。
    pub fn on_seek(mut self, f: Arc<dyn Fn(f32) -> Message + Send + Sync>) -> Self {
        self.on_seek = Some(f);
        self
    }

    /// bounds 局部 px → 0..1（出界钳到端点）。
    fn fraction(local_x: f32, width: f32) -> f32 {
        if width <= 0.0 {
            return 0.0;
        }
        (local_x / width).clamp(0.0, 1.0)
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for SeekArea<'_, Message>
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
        if let Some(on_seek) = self.on_seek.clone() {
            let bounds = layout.bounds();
            let state = tree.state.downcast_mut::<State>();
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if let Some(p) = cursor.position_in(bounds) {
                        state.pressed = true;
                        shell.publish((on_seek)(Self::fraction(p.x, bounds.width)));
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    state.pressed = false;
                }
                // 按住期间的移动 = 拖拽。**出界也继续跟手**（比例钳到端点），
                // 所以这里用绝对坐标而不是 position_in（后者出界即 None，
                // 会让拖到条外的操作忽然断流）。
                Event::Mouse(mouse::Event::CursorMoved { .. }) if state.pressed => {
                    if let Some(p) = cursor.position() {
                        shell.publish((on_seek)(Self::fraction(
                            p.x - bounds.x,
                            bounds.width,
                        )));
                    }
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
        // 可拖拽时给「手型」，与浏览器里进度条/滑条的可交互观感一致。
        if self.on_seek.is_some() && cursor.is_over(layout.bounds()) {
            return mouse::Interaction::Pointer;
        }
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<SeekArea<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(area: SeekArea<'a, Message>) -> Self {
        Element::new(area)
    }
}
