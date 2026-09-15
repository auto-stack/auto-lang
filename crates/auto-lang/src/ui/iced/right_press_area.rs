// PLAN-631 F-7: 指针按下记忆根 wrapper —— popover `placement: "pointer"` 的锚源。
//
// 为什么需要它:iced 0.14 的 `mouse_area.on_right_press(message)` 与 button
// `on_press` 都只发布消息、不携带坐标,`.at` 事件同样无坐标(PLAN-631 §4
// D-1 实证)——Win11 式"菜单在指针处打开"无从落点。又因 iced `mouse::Event::
// ButtonPressed` 本身不带 position(仅 CursorMoved 带),逐触发件包装也拿不到
// 按下现场坐标。
//
// 落点:窗口根单包装。iced 事件遍历把每个事件送达树内**每个** widget 的
// update(命中与否由各 widget 自判),且 `Cursor` 携带窗口绝对位置——根级
// 纯委托 wrapper 在 ButtonPressed 事件现场读 cursor 记账,即得全窗口的
// "最近一次指针按下位置"(左键右键都记:行右键菜单与 "···" 左键快捷菜单
// 同源;按下事件先于其发布的 onclick/oncontextmenu 消息,故 open 翻真时
// 记录必已就位)。坐标不进 VM 状态、不经消息回路(plan §5.2 契约)。
//
// 单槽语义(plan §10 Q3 v1 裁定:先单槽实测定):多触发件并存时后写覆盖,
// 语义 = "最近一次按下位置";跨 App 窗口进程内共享同槽(桌面多窗竞用同一
// popover 的场景不存在,最近写入即正确锚)。

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Point, Rectangle, Size, Vector};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// 会话级单槽:HAS = 有效位,POS 低 32 位 = x f32 位型、高 32 位 = y 位型
/// （f32 位型原样存储——负坐标多显示器左/上扩展不丢符号）。
static HAS_POINTER_PRESS: AtomicBool = AtomicBool::new(false);
static LAST_POINTER_PRESS: AtomicU64 = AtomicU64::new(0);

/// 指针按下时记录位置(窗口逻辑坐标)。
pub fn note_pointer_press(x: f32, y: f32) {
    let bits = ((x.to_bits() as u64) << 32) | y.to_bits() as u64;
    LAST_POINTER_PRESS.store(bits, Ordering::Relaxed);
    HAS_POINTER_PRESS.store(true, Ordering::Relaxed);
}

/// 最近一次指针按下位置(popover 面板 placement "pointer" 读取;None = 回退
/// 锚件语义)。
pub fn last_pointer_press() -> Option<Point> {
    if !HAS_POINTER_PRESS.load(Ordering::Relaxed) {
        return None;
    }
    let bits = LAST_POINTER_PRESS.load(Ordering::Relaxed);
    let x = f32::from_bits((bits >> 32) as u32);
    let y = f32::from_bits(bits as u32);
    Some(Point::new(x, y))
}

/// 清空记账(测试/诊断用:验证"未记录回退"路径)。
pub fn clear_pointer_press() {
    HAS_POINTER_PRESS.store(false, Ordering::Relaxed);
}

/// 测试支援:槽为进程级单例,触碰它的测试(裸 cargo test 并行态)须持锁
/// 串行化(nextest 每测试独立进程,无此需求,锁兼容两态)。
#[cfg(test)]
pub mod test_support {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn lock() -> &'static Mutex<()> {
        static L: OnceLock<Mutex<()>> = OnceLock::new();
        L.get_or_init(|| Mutex::new(()))
    }

    pub fn slot_lock() -> MutexGuard<'static, ()> {
        lock().lock().expect("slot lock poisoned")
    }
}

/// 指针按下记忆 wrapper(窗口根单包装)。纯委托:不捕获事件、不改内层
/// 语义、不参与命中,只在 ButtonPressed 事件现场叠加记账。
pub struct PointerPressArea<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message> PointerPressArea<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for PointerPressArea<'_, Message>
where
    Message: Clone + 'static,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<()>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(())
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
        // 按下记账叠加在事件转发之上(不捕获)。cursor.position() 是窗口
        // 绝对位置(无论按下落在树内哪个触发件上),与 popover overlay 的
        // 面板定位同坐标系。
        if let Event::Mouse(mouse::Event::ButtonPressed(_)) = event {
            if let Some(pos) = cursor.position() {
                note_pointer_press(pos.x, pos.y);
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

impl<'a, Message> From<PointerPressArea<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(area: PointerPressArea<'a, Message>) -> Self {
        Self::new(area)
    }
}

#[cfg(test)]
mod tests {
    use super::{last_pointer_press, note_pointer_press};

    #[test]
    fn roundtrip() {
        // 单槽为进程级(测试并发下初始态不可假定),先写后读。
        note_pointer_press(120.5, 88.25);
        let p = last_pointer_press().expect("recorded");
        assert!((p.x - 120.5).abs() < f32::EPSILON, "x = {}", p.x);
        assert!((p.y - 88.25).abs() < f32::EPSILON, "y = {}", p.y);
    }

    #[test]
    fn negative_coords_roundtrip() {
        // 负坐标(多显示器左/上扩展)位型往返不丢符号。
        note_pointer_press(-15.0, -200.5);
        let p = last_pointer_press().expect("recorded");
        assert!((p.x - (-15.0)).abs() < f32::EPSILON, "x = {}", p.x);
        assert!((p.y - (-200.5)).abs() < f32::EPSILON, "y = {}", p.y);
    }
}
