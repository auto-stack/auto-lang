//! PLAN-063 T-04d-2: 右栏块锚定坐标槽——AnchorSlot wrapper 在 iced layout
//! 期记录每块**高度**（布局对屏内外一视同仁；layout 期 bounds 位置尚未被
//! 父级定位，故记高度不记 y），消费侧按「前块高累计 + 列间距」精确算出
//! 任意块的内容 y，写进 sync_anchor_target 状态字段（右栏 offset 绑定
//! 写臂既有通路滚动到目标；iced 原生钳制内容边界）。
//!
//! 块 0 = 内容原点。锚块索引来自编辑壳 ade 存储（sync_anchor 高亮链同源）。

use iced::{Element, Length, Point, Rectangle, Size, Theme};
use std::sync::Mutex;

/// index → 块高（layout 期写入）。未布局槽 = f32::NAN。
static SLOT_HEIGHTS: Mutex<Vec<f32>> = Mutex::new(Vec::new());
/// 文档列子件间距（autodown_render 文档 Column spacing）。
static COL_SPACING: Mutex<f32> = Mutex::new(8.0);
static PENDING_ANCHOR: Mutex<Option<usize>> = Mutex::new(None);
static TARGET_SINK: Mutex<Option<String>> = Mutex::new(None);

/// 布局期记录（AnchorSlot wrapper 每帧调用；NaN 占位 = 本帧未布局）。
pub fn record_slot(index: usize, h: f32) {
    let mut slots = SLOT_HEIGHTS.lock().unwrap();
    if slots.len() <= index {
        slots.resize(index + 1, f32::NAN);
    }
    slots[index] = h;
}

/// 文档列子件间距（render 面每帧登记，与 View::Column spacing 同源）。
pub fn set_col_spacing(s: f32) {
    *COL_SPACING.lock().unwrap() = s;
}

/// 块 index 的内容 y = Σ_{j<index}(h_j + spacing) − spacing（首块 y=0，
/// 末段不含尾间距）。任一前序块未布局 → None（消费侧跳过本拍）。
pub fn content_y(index: usize) -> Option<f32> {
    let slots = SLOT_HEIGHTS.lock().unwrap();
    if index == 0 {
        return Some(0.0);
    }
    if slots.len() < index {
        return None;
    }
    let spacing = *COL_SPACING.lock().unwrap();
    let mut y = 0.0_f32;
    for slot in slots.iter().take(index) {
        if slot.is_nan() {
            return None;
        }
        y += slot + spacing;
    }
    Some(y)
}

/// scroll 回调：登记待消费锚块（渲染 update 尾部清算）。
pub fn set_pending_anchor(index: usize) {
    *PENDING_ANCHOR.lock().unwrap() = Some(index);
}

/// 渲染 update 尾部清算：取走 (target 状态字段, 锚块索引)。
pub fn drain_pending_anchor() -> Option<(Option<String>, usize)> {
    let idx = PENDING_ANCHOR.lock().unwrap().take()?;
    let sink = TARGET_SINK.lock().unwrap().clone();
    Some((sink, idx))
}

/// 视图构建面登记目标状态字段名（sync_anchor_target prop，剥前导点）。
pub fn set_target_sink(field: Option<String>) {
    *TARGET_SINK.lock().unwrap() = field;
}

/// PLAN-063 T-04d-2: 锚槽本地状态——**必须非 ()**：iced 0.14 对无状态
/// widget 不构建子树（children 被无视→子件拿到无状态树→downcast 崩溃）。
#[derive(Default)]
pub struct AnchorSlotState;

/// PLAN-063 T-04d-2: 块锚定坐标槽 wrapper——layout 委托子件并记录块高。
pub struct AnchorSlot<'a, M: Clone + std::fmt::Debug + 'static> {
    pub index: usize,
    pub child: Element<'a, M>,
}

impl<M: Clone + std::fmt::Debug + 'static> iced::advanced::Widget<M, Theme, iced::Renderer>
    for AnchorSlot<'_, M>
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<AnchorSlot<M>>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(AnchorSlotState)
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.child)]
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.child));
    }

    fn size(&self) -> Size<Length> {
        self.child.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let node = self.child.as_widget_mut().layout(&mut tree.children[0], renderer, limits);
        // PLAN-063 T-04d-2: 记录块高（高度在子件自身 layout 内即确定，
        // 不依赖父级定位；y 由消费侧按累计+间距计算）。
        record_slot(self.index, node.bounds().height);
        node
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        self.child.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.child.as_widget().draw(
            &tree.children[0], renderer, theme, style, layout, cursor, viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.child
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }
}
