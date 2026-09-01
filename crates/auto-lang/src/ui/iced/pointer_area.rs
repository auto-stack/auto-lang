// Plan 499 M2: 指针移动限频 widget —— mouse-area onmousemove 臂的 iced 承载。
//
// 为什么不用 iced 原生 `mouse_area.on_move`:其闭包签名 `Fn(Point) -> Message`
// 只给 bounds 局部 px,闭包拿不到 bounds 尺寸——px→逻辑坐标换算无从进行
// (docs/design/autoui/canvas-pointer-events.md §1.3)。本 widget 在事件现场
// 持有 layout bounds,完成两件事:
//   1. **坐标换算**:logical = local_px / bounds_size × extent(extent 缺省
//      恒等 = raw px 模式)——"屏幕→逻辑换算在引擎层完成"的落点;
//   2. **限频**(VM 臂专属,plan 目标 ≤30Hz):
//      a) 时间闸——距上次发布 < MIN_INTERVAL_MS 丢弃;
//      b) 量化去重——逻辑坐标量化 0.5px 后与上次发布相同则丢弃
//         (静止悬停零事件流)。
//
// 结构为纯委托包装(precedent:popover.rs):enter/exit/dblclick 等其余事件
// 语义由内层元素(通常就是 iced mouse_area)承担,本 widget 只在 CursorMoved
// 上叠加发布,不捕获事件。

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Point, Rectangle, Size, Vector};
use std::sync::Arc;
use std::time::Instant;

/// 时间闸:两次发布最小间隔(≈30Hz,Plan 499 目标采样率)。
const MIN_INTERVAL_MS: u64 = 33;
/// 量化步长:逻辑坐标按 0.5px 量化后比较,静止/亚像素移动不发布。
const QUANTIZE_STEP: f32 = 0.5;

/// PointerArea 的本地状态(tree::Tag 标识)。
#[derive(Debug, Default)]
pub struct State {
    last_pub: Option<Instant>,
    last_logical: Option<(f32, f32)>,
}

/// 限频决策核心(纯函数,便于单测):给定量化后的逻辑坐标与上次发布记录,
/// 返回是否发布。规则:量化坐标不变 → 不发布;距上次发布 < 时间闸 → 不发布
/// (且**不**回写 last_pub——闸内移动被丢弃,闸过期后下一次移动即发布,
/// 不积压陈旧坐标)。
pub fn should_publish(
    state: &mut State,
    logical: (f32, f32),
    now: Instant,
) -> bool {
    let qx = (logical.0 / QUANTIZE_STEP).round() * QUANTIZE_STEP;
    let qy = (logical.1 / QUANTIZE_STEP).round() * QUANTIZE_STEP;
    if let Some((px, py)) = state.last_logical {
        if (px - qx).abs() < f32::EPSILON && (py - qy).abs() < f32::EPSILON {
            return false;
        }
    }
    if let Some(t) = state.last_pub {
        if (now.duration_since(t).as_millis() as u64) < MIN_INTERVAL_MS {
            return false;
        }
    }
    state.last_pub = Some(now);
    state.last_logical = Some((qx, qy));
    true
}

/// bounds 局部 px → 组件局部逻辑坐标(extent 缺省 = 恒等/raw px 模式)。
pub fn to_logical(
    local: Point,
    bounds: &Rectangle,
    extent: Option<(f32, f32)>,
) -> (f32, f32) {
    match extent {
        Some((w, h)) if bounds.width > 0.0 && bounds.height > 0.0 => {
            (local.x / bounds.width * w, local.y / bounds.height * h)
        }
        _ => (local.x, local.y),
    }
}

pub struct PointerArea<'a, Message>
where
    Message: Clone + 'static,
{
    content: Element<'a, Message>,
    extent: Option<(f32, f32)>,
    on_move: Option<Arc<dyn Fn(f32, f32) -> Message + Send + Sync>>,
}

impl<'a, Message> PointerArea<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            extent: None,
            on_move: None,
        }
    }

    /// 逻辑幅面(coords prop "WxH" 解析结果;None = raw px)。
    pub fn extent(mut self, w: f32, h: f32) -> Self {
        self.extent = Some((w, h));
        self
    }

    pub fn on_move(
        mut self,
        f: Arc<dyn Fn(f32, f32) -> Message + Send + Sync>,
    ) -> Self {
        self.on_move = Some(f);
        self
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for PointerArea<'_, Message>
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
        // 限频发布叠加在事件转发之上(不捕获,不改变内层语义)。
        if let (Some(on_move), Event::Mouse(iced::mouse::Event::CursorMoved { .. })) =
            (&self.on_move, event)
        {
            let bounds = layout.bounds();
            if cursor.is_over(bounds) {
                if let Some(local) = cursor.position_in(bounds) {
                    let logical = to_logical(local, &bounds, self.extent);
                    let state = tree.state.downcast_mut::<State>();
                    if should_publish(state, logical, Instant::now()) {
                        shell.publish((on_move)(logical.0, logical.1));
                    }
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

impl<'a, Message> From<PointerArea<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(area: PointerArea<'a, Message>) -> Self {
        Element::new(area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(ms: u64) -> Instant {
        let base = Instant::now();
        base + std::time::Duration::from_millis(ms)
    }

    #[test]
    fn throttle_time_gate_and_quantized_dedupe() {
        // Plan 499 M2 压测断言(单元级):125Hz 合成流 → 发布率 ≤ 30Hz;
        // 量化同坐标零发布。
        let mut s = State::default();
        let mut published = 0usize;
        // 模拟 125Hz × 1s = 125 个事件,坐标每次 +3px(持续移动)。
        for i in 0..125 {
            let now = t((i as u64) * 8); // 8ms 间隔 = 125Hz
            let x = i as f32 * 3.0;
            if should_publish(&mut s, (x, 0.0), now) {
                published += 1;
            }
        }
        // 时间闸 33ms:8ms 步进流对齐到 40ms 发布节拍 → 实测 25 次(25Hz)。
        // 断言 ≤ 32(≤30Hz 预算)且 ≥ 20(持续移动不过度丢弃)。
        assert!(
            published <= 32,
            "125Hz 流下发布 {} 次,超出 30Hz 限频预算",
            published
        );
        assert!(published >= 20, "持续移动下不应过度丢弃(实测 {} 次)", published);
    }

    #[test]
    fn stationary_hover_publishes_once() {
        // 静止悬停:量化后坐标不变 → 仅首 publish。
        let mut s = State::default();
        let mut published = 0;
        for i in 0..50 {
            if should_publish(&mut s, (100.0, 50.0), t(i * 100)) {
                published += 1;
            }
        }
        assert_eq!(published, 1, "静止悬停只应发布一次,实测 {}", published);
    }

    #[test]
    fn gate_expiry_publishes_fresh_position() {
        // 闸内移动被丢弃,闸过期后下一次移动立即发布(新坐标,无积压)。
        let mut s = State::default();
        assert!(should_publish(&mut s, (10.0, 0.0), t(0)));
        assert!(!should_publish(&mut s, (40.0, 0.0), t(10)), "闸内丢弃");
        assert!(
            should_publish(&mut s, (70.0, 0.0), t(40)),
            "闸过期后发布最新坐标"
        );
    }

    #[test]
    fn logical_conversion_with_extent() {
        // Plan 499 M2 坐标断言:bounds 400×214、extent 560×300、
        // px (200,107) → 逻辑 (280,150) ± 0.5。
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(400.0, 214.0));
        let (x, y) = to_logical(Point::new(200.0, 107.0), &bounds, Some((560.0, 300.0)));
        assert!((x - 280.0).abs() < 0.5, "logical x = {}", x);
        assert!((y - 150.0).abs() < 0.5, "logical y = {}", y);
    }

    #[test]
    fn raw_px_mode_without_extent() {
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(400.0, 214.0));
        let (x, y) = to_logical(Point::new(200.0, 107.0), &bounds, None);
        assert_eq!((x, y), (200.0, 107.0), "无 extent = raw px 恒等");
    }

    #[test]
    fn zero_bounds_falls_back_to_raw() {
        // 布局未就绪(bounds 0)不产生 NaN/inf。
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(0.0, 0.0));
        let (x, y) = to_logical(Point::new(5.0, 7.0), &bounds, Some((560.0, 300.0)));
        assert_eq!((x, y), (5.0, 7.0));
    }
}
