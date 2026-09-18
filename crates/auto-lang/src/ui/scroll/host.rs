//! PLAN-656: ScrollContent hosting contract（plan r2 §4，设计文档 §7/§13）。
//!
//! ScrollPane 与直接内容节点之间的能力协议——backend-independent，iced / Vue
//! 只做 adapter。三条语义通道：
//!
//! ```text
//! content → pane : scroll_state()            （State down）
//! pane → content : apply_scroll_intent()     （Intent up）
//! pane → content : viewport_changed()        （Viewport down）
//! ```
//!
//! 状态所有权（plan r2 §4.2）：managed content 的 semantic offset 与 logical
//! extent 归 host 单源；ScrollPane/backend 内部缓存只能是投影。public
//! `on-scroll` 不参与本协议（观察面与控制面分离，plan r2 §4.4）。

use crate::ui::scroll::geometry::{clamp_offset, scroll_range};
use crate::ui::scroll::intent::ScrollIntent;
use crate::ui::scroll::state::{Axis, ScrollAxes, ScrollAxisState, ScrollState, ScrollViewportState};

/// hosting contract 的规范形态。runtime 可用 trait 对象、callback bundle
/// （[`ScrollContentHostRecord`]）或内部 message enum 承载，但三条通道语义
/// 不得缩水成某一 backend 的专属 helper。
pub trait ScrollContentHost {
    /// host 单源的逻辑滚动状态（offset + logical content extent）。
    fn scroll_state(&self) -> ScrollState;

    /// 应用一条滚动意图（wheel/scrollbar/keyboard/controller 统一入口）。
    fn apply_scroll_intent(&mut self, intent: ScrollIntent);

    /// 接收最新 viewport 测量（初次挂载 / resize / layout 变化时 pane 下发）。
    fn viewport_changed(&mut self, viewport: ScrollViewportState);
}

/// callback-bundle 形态的 seam：适合跨消息边界 / runtime 句柄场景，语义与
/// [`ScrollContentHost`] 三通道一一对应。
pub struct ScrollContentHostRecord {
    pub state: Box<dyn Fn() -> ScrollState + Send + Sync>,
    pub apply_intent: Box<dyn Fn(ScrollIntent) + Send + Sync>,
    pub viewport: Box<dyn Fn(ScrollViewportState) + Send + Sync>,
}

impl ScrollContentHostRecord {
    pub fn new(
        state: Box<dyn Fn() -> ScrollState + Send + Sync>,
        apply_intent: Box<dyn Fn(ScrollIntent) + Send + Sync>,
        viewport: Box<dyn Fn(ScrollViewportState) + Send + Sync>,
    ) -> Self {
        Self { state, apply_intent, viewport }
    }
}

/// synthetic managed-content 默认逻辑尺寸（plan r2 §11.2）：
/// 逻辑高度 10,000,000px / 宽度 2,000,000px（both case）。
pub const SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_H: f64 = 10_000_000.0;
pub const SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_W: f64 = 2_000_000.0;

/// intent 日志容量上限（能力验证用，防长期运行无界增长）。
const SYNTHETIC_INTENT_LOG_CAP: usize = 64;

/// synthetic managed content（plan r2 §11）：PLAN-656 的架构验收工具，**不是**
/// 正式 public widget。证明 ScrollPane 能滚动一个没有对应真实巨大内容树的
/// logical content：
///
/// - 逻辑 extent 巨大（默认 10M × 2M px），实际物化节点由各 backend bridge
///   控制在 ~20 行/列；
/// - 维护 host offset / logical extent / latest viewport / received intent log；
/// - 接收 `ScrollViewportState` + `ScrollIntent`，发布 `ScrollState`。
#[derive(Debug, Clone)]
pub struct SyntheticManagedContent {
    axes: ScrollAxes,
    logical_w: f64,
    logical_h: f64,
    offset_x: f64,
    offset_y: f64,
    viewport: ScrollViewportState,
    intent_log: Vec<ScrollIntent>,
}

impl SyntheticManagedContent {
    pub fn new(axes: ScrollAxes) -> Self {
        Self {
            axes,
            logical_w: SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_W,
            logical_h: SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_H,
            offset_x: 0.0,
            offset_y: 0.0,
            viewport: ScrollViewportState::default(),
            intent_log: Vec::new(),
        }
        .clamped()
    }

    pub fn with_logical_extent(mut self, w: f64, h: f64) -> Self {
        self.logical_w = w.max(0.0);
        self.logical_h = h.max(0.0);
        self.clamped()
    }

    pub fn axes(&self) -> ScrollAxes {
        self.axes
    }

    pub fn logical_extent(&self) -> (f64, f64) {
        (self.logical_w, self.logical_h)
    }

    pub fn host_offset(&self) -> (f64, f64) {
        (self.offset_x, self.offset_y)
    }

    pub fn latest_viewport(&self) -> ScrollViewportState {
        self.viewport
    }

    pub fn intent_log(&self) -> &[ScrollIntent] {
        &self.intent_log
    }

    fn axis_state(&self, axis: Axis) -> ScrollAxisState {
        match axis {
            Axis::X => ScrollAxisState {
                offset: self.offset_x,
                viewport_extent: self.viewport.width,
                content_extent: self.logical_w,
            },
            Axis::Y => ScrollAxisState {
                offset: self.offset_y,
                viewport_extent: self.viewport.height,
                content_extent: self.logical_h,
            },
        }
    }

    fn set_axis_offset(&mut self, axis: Axis, offset: f64) {
        match axis {
            Axis::X => self.offset_x = offset,
            Axis::Y => self.offset_y = offset,
        }
    }

    /// 把双轴 offset 重新 clamp 到各自 range（extent/viewport 变化后调用）。
    fn clamped(mut self) -> Self {
        for axis in [Axis::X, Axis::Y] {
            let state = self.axis_state(axis);
            let clamped = clamp_offset(&state, state.offset);
            self.set_axis_offset(axis, clamped);
        }
        self.intent_log = Vec::new();
        self
    }
}

impl ScrollContentHost for SyntheticManagedContent {
    fn scroll_state(&self) -> ScrollState {
        let mut state = ScrollState::default();
        if self.axes.x {
            state.x = Some(self.axis_state(Axis::X));
        }
        if self.axes.y {
            state.y = Some(self.axis_state(Axis::Y));
        }
        state
    }

    fn apply_scroll_intent(&mut self, intent: ScrollIntent) {
        let resolved = intent.resolve(&self.scroll_state());
        self.set_axis_offset(resolved.axis, resolved.offset);
        self.intent_log.push(intent);
        if self.intent_log.len() > SYNTHETIC_INTENT_LOG_CAP {
            self.intent_log.remove(0);
        }
    }

    fn viewport_changed(&mut self, viewport: ScrollViewportState) {
        self.viewport = viewport;
        // viewport 变化可能缩小 range：立即重 clamp，避免悬挂 offset。
        for axis in [Axis::X, Axis::Y] {
            let state = self.axis_state(axis);
            let clamped = clamp_offset(&state, state.offset);
            self.set_axis_offset(axis, clamped);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::scroll::intent::ScrollIntent;
    use crate::ui::scroll::state::ScrollbarPolicy;

    fn viewport(w: f64, h: f64) -> ScrollViewportState {
        ScrollViewportState { width: w, height: h }
    }

    #[test]
    fn state_channel_publishes_logical_extent_and_offset() {
        let mut host = SyntheticManagedContent::new(ScrollAxes::Y);
        host.viewport_changed(viewport(400.0, 600.0));
        host.apply_scroll_intent(ScrollIntent::ScrollBy { axis: Axis::Y, delta: 1_000.0, source: crate::ui::scroll::ScrollSource::Wheel });
        let state = host.scroll_state();
        let y = state.y.expect("y axis enabled");
        assert_eq!(y.content_extent, SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_H);
        assert_eq!(y.viewport_extent, 600.0);
        assert_eq!(y.offset, 1_000.0);
        assert!(state.x.is_none(), "x axis not enabled");
    }

    #[test]
    fn intent_channel_resolves_and_clamps_against_logical_extent() {
        let mut host = SyntheticManagedContent::new(ScrollAxes::Y);
        host.viewport_changed(viewport(400.0, 600.0));
        host.apply_scroll_intent(ScrollIntent::ToEnd { axis: Axis::Y, source: crate::ui::scroll::ScrollSource::Programmatic });
        let (_, offset_y) = host.host_offset();
        assert_eq!(offset_y, SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_H - 600.0);
        assert_eq!(host.intent_log().len(), 1);
    }

    #[test]
    fn viewport_channel_drives_range_and_reclamps() {
        let mut host = SyntheticManagedContent::new(ScrollAxes::Y).with_logical_extent(400.0, 1_000.0);
        host.viewport_changed(viewport(400.0, 200.0));
        host.apply_scroll_intent(ScrollIntent::ToEnd { axis: Axis::Y, source: crate::ui::scroll::ScrollSource::Programmatic });
        assert_eq!(host.host_offset().1, 800.0);
        // resize：viewport 变大 → range 缩小 → offset 重 clamp（无悬挂）。
        host.viewport_changed(viewport(400.0, 900.0));
        assert_eq!(host.host_offset().1, 100.0);
        // 逻辑 extent 缩小同理（host 程序化更新路径由 bridge 走 apply/set 后重 clamp）。
        let state = host.scroll_state();
        assert_eq!(crate::ui::scroll::geometry::scroll_range(&state.y.unwrap()), 100.0);
    }

    #[test]
    fn both_axes_independent() {
        let mut host = SyntheticManagedContent::new(ScrollAxes::BOTH);
        host.viewport_changed(viewport(300.0, 500.0));
        host.apply_scroll_intent(ScrollIntent::ScrollTo { axis: Axis::X, offset: 500_000.0, source: crate::ui::scroll::ScrollSource::Programmatic });
        host.apply_scroll_intent(ScrollIntent::ScrollTo { axis: Axis::Y, offset: 7_000_000.0, source: crate::ui::scroll::ScrollSource::Programmatic });
        let (x, y) = host.host_offset();
        assert_eq!(x, 500_000.0);
        assert_eq!(y, 7_000_000.0);
        // 各自 clamp 到各自 range，互不影响。
        host.apply_scroll_intent(ScrollIntent::ToEnd { axis: Axis::X, source: crate::ui::scroll::ScrollSource::Programmatic });
        assert_eq!(host.host_offset().0, SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_W - 300.0);
        assert_eq!(host.host_offset().1, 7_000_000.0);
    }

    #[test]
    fn intent_log_capped() {
        let mut host = SyntheticManagedContent::new(ScrollAxes::Y);
        host.viewport_changed(viewport(400.0, 600.0));
        for _ in 0..(SYNTHETIC_INTENT_LOG_CAP + 10) {
            host.apply_scroll_intent(ScrollIntent::ScrollBy { axis: Axis::Y, delta: 0.0, source: crate::ui::scroll::ScrollSource::Wheel });
        }
        assert_eq!(host.intent_log().len(), SYNTHETIC_INTENT_LOG_CAP);
    }

    #[test]
    fn host_record_carries_three_channels() {
        // callback-bundle seam 与 trait 三通道一一对应（backend adapter 可用形态）。
        let mut host = SyntheticManagedContent::new(ScrollAxes::Y);
        host.viewport_changed(viewport(400.0, 600.0));
        let mut inner = host;
        let record = ScrollContentHostRecord::new(
            Box::new(move || inner.scroll_state()),
            Box::new(|_intent| {}),
            Box::new(|_vp| {}),
        );
        let _ = record;
        // 仅证明可构造；通道语义已在上方用例覆盖。
    }

    #[test]
    fn policy_is_orthogonal_to_host() {
        // scrollbar 策略是 pane 配置，与 hosting 通道正交（类型层可同存）。
        let policy = ScrollbarPolicy::Hidden;
        let mut host = SyntheticManagedContent::new(ScrollAxes::Y);
        host.viewport_changed(viewport(400.0, 600.0));
        host.apply_scroll_intent(ScrollIntent::ScrollBy { axis: Axis::Y, delta: 50.0, source: crate::ui::scroll::ScrollSource::Keyboard });
        assert_eq!(host.host_offset().1, 50.0);
        assert_eq!(policy, ScrollbarPolicy::Hidden);
    }
}
