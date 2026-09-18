//! PLAN-656: 统一滚动意图（plan r2 §3.4–§3.5，设计文档 §8.2/§8.3）。
//!
//! State down / Intent up：pane 把 wheel / scrollbar / keyboard / controller 等
//! 输入统一折叠成 [`ScrollIntent`] 交给内容宿主；宿主应用后重新发布
//! `ScrollState`。避免「双向 offset 绑定 + suppress echo」成为架构核心。

/// 滚动输入来源。`NativeScrollbar` / `Unknown` 为 Web / Compose / UIKit /
/// Harmony 等未来 native scrolling backend 保留出口——不强迫每个 backend 都能
/// 区分 thumb drag 与 rail click（plan r2 §3.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollSource {
    Wheel,
    Touchpad,
    Touch,
    ThumbDrag,
    RailClick,
    NativeScrollbar,
    Keyboard,
    Programmatic,
    Sync,
    FocusReveal,
    AutoScroll,
    Unknown,
}

/// 统一滚动意图（单轴 + 来源）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollIntent {
    ScrollBy { axis: Axis, delta: f64, source: ScrollSource },
    ScrollTo { axis: Axis, offset: f64, source: ScrollSource },
    PageBy { axis: Axis, pages: f64, source: ScrollSource },
    ToStart { axis: Axis, source: ScrollSource },
    ToEnd { axis: Axis, source: ScrollSource },
}

/// `ScrollIntent::resolve` 的归一化产物：clamp 后的绝对 offset。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedScrollIntent {
    pub axis: Axis,
    pub offset: f64,
    pub source: ScrollSource,
}

impl ScrollIntent {
    pub fn axis(&self) -> Axis {
        match self {
            Self::ScrollBy { axis, .. }
            | Self::ScrollTo { axis, .. }
            | Self::PageBy { axis, .. }
            | Self::ToStart { axis, .. }
            | Self::ToEnd { axis, .. } => *axis,
        }
    }

    pub fn source(&self) -> ScrollSource {
        match self {
            Self::ScrollBy { source, .. }
            | Self::ScrollTo { source, .. }
            | Self::PageBy { source, .. }
            | Self::ToStart { source, .. }
            | Self::ToEnd { source, .. } => *source,
        }
    }

    /// 归一化为 clamp 后的绝对 offset。对未启用轴（`ScrollState` 中为 `None`）
    /// 按退化态处理（range=0 → offset=0），保证 total 行为；`delta`/`offset` 的
    /// NaN 由 [`crate::ui::scroll::geometry::clamp_offset`] 折为 0。
    pub fn resolve(&self, current: &ScrollState) -> ResolvedScrollIntent {
        let axis = self.axis();
        let source = self.source();
        let state = current.axis_or_degenerate(axis);
        let range = geometry::scroll_range(&state);
        let resolved = match self {
            Self::ScrollBy { delta, .. } => state.offset + delta,
            Self::ScrollTo { offset, .. } => *offset,
            Self::PageBy { pages, .. } => state.offset + pages * state.viewport_extent,
            Self::ToStart { .. } => 0.0,
            Self::ToEnd { .. } => range,
        };
        ResolvedScrollIntent { axis, offset: geometry::clamp_offset(&state, resolved), source }
    }
}

use crate::ui::scroll::geometry;
use crate::ui::scroll::state::{Axis, ScrollAxisState, ScrollState};

#[cfg(test)]
mod tests {
    use super::*;

    fn y_state(offset: f64, viewport: f64, content: f64) -> ScrollState {
        ScrollState { x: None, y: Some(ScrollAxisState { offset, viewport_extent: viewport, content_extent: content }) }
    }

    #[test]
    fn resolve_scroll_by_clamps_at_edges() {
        let cur = y_state(100.0, 200.0, 1000.0);
        let r = ScrollIntent::ScrollBy { axis: Axis::Y, delta: 150.0, source: ScrollSource::Wheel }.resolve(&cur);
        assert_eq!(r.offset, 250.0);
        let over = ScrollIntent::ScrollBy { axis: Axis::Y, delta: 10_000.0, source: ScrollSource::Wheel }.resolve(&cur);
        assert_eq!(over.offset, 800.0); // range = 1000-200
        let under = ScrollIntent::ScrollBy { axis: Axis::Y, delta: -500.0, source: ScrollSource::Wheel }.resolve(&cur);
        assert_eq!(under.offset, 0.0);
    }

    #[test]
    fn resolve_to_end_and_start() {
        let cur = y_state(123.0, 200.0, 10_000_000.0);
        assert_eq!(ScrollIntent::ToEnd { axis: Axis::Y, source: ScrollSource::Programmatic }.resolve(&cur).offset, 9_999_800.0);
        assert_eq!(ScrollIntent::ToStart { axis: Axis::Y, source: ScrollSource::Programmatic }.resolve(&cur).offset, 0.0);
    }

    #[test]
    fn resolve_page_by_uses_viewport_extent() {
        let cur = y_state(0.0, 250.0, 2000.0);
        let r = ScrollIntent::PageBy { axis: Axis::Y, pages: 2.0, source: ScrollSource::Keyboard }.resolve(&cur);
        assert_eq!(r.offset, 500.0);
        let half_back = ScrollIntent::PageBy { axis: Axis::Y, pages: -1.0, source: ScrollSource::Keyboard }.resolve(&r_into(&r, 250.0, 2000.0));
        assert_eq!(half_back.offset, 250.0);
    }

    fn r_into(r: &ResolvedScrollIntent, viewport: f64, content: f64) -> ScrollState {
        y_state(r.offset, viewport, content)
    }

    #[test]
    fn resolve_x_axis_independent_of_y() {
        let mut cur = y_state(50.0, 100.0, 400.0);
        cur.x = Some(ScrollAxisState { offset: 10.0, viewport_extent: 100.0, content_extent: 2_000_000.0 });
        let r = ScrollIntent::ScrollTo { axis: Axis::X, offset: 1_500_000.0, source: ScrollSource::Programmatic }.resolve(&cur);
        assert_eq!(r.axis, Axis::X);
        assert_eq!(r.offset, 1_500_000.0);
        // y 轴不受影响（resolve 只读）
        assert_eq!(cur.y.unwrap().offset, 50.0);
    }

    #[test]
    fn resolve_disabled_axis_degenerates_to_zero() {
        let cur = y_state(10.0, 100.0, 500.0); // x 未启用
        let r = ScrollIntent::ScrollTo { axis: Axis::X, offset: 1234.0, source: ScrollSource::Programmatic }.resolve(&cur);
        assert_eq!(r.offset, 0.0);
    }

    #[test]
    fn resolve_nan_inputs_stay_total() {
        let cur = y_state(10.0, 100.0, 500.0);
        let r = ScrollIntent::ScrollTo { axis: Axis::Y, offset: f64::NAN, source: ScrollSource::Unknown }.resolve(&cur);
        assert_eq!(r.offset, 0.0);
        let by = ScrollIntent::ScrollBy { axis: Axis::Y, delta: f64::NAN, source: ScrollSource::Unknown }.resolve(&cur);
        assert_eq!(by.offset, 0.0);
    }

    #[test]
    fn resolve_content_smaller_than_viewport_is_sticky_zero() {
        let cur = y_state(0.0, 500.0, 300.0); // content < viewport
        assert_eq!(ScrollIntent::ToEnd { axis: Axis::Y, source: ScrollSource::Programmatic }.resolve(&cur).offset, 0.0);
        assert_eq!(ScrollIntent::ScrollBy { axis: Axis::Y, delta: 80.0, source: ScrollSource::Wheel }.resolve(&cur).offset, 0.0);
    }

    #[test]
    fn source_carried_through_resolve() {
        let cur = y_state(0.0, 100.0, 1000.0);
        for intent in [
            ScrollIntent::ScrollBy { axis: Axis::Y, delta: 1.0, source: ScrollSource::Touchpad },
            ScrollIntent::PageBy { axis: Axis::Y, pages: 1.0, source: ScrollSource::NativeScrollbar },
        ] {
            assert_eq!(intent.resolve(&cur).source, intent.source());
        }
    }
}
