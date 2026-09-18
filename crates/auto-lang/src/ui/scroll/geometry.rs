//! PLAN-656: 滚动几何纯函数（plan r2 §3.6，设计文档 §12.3）。
//!
//! 关键陷阱：`min_thumb_extent` clamp 之后映射必须继续用
//! `thumb_travel = rail_extent - thumb_extent`，不能按 content 比例直映射；
//! 正反映射共用同一套 thumb 几何，保证往返恒等。

use crate::ui::scroll::state::ScrollAxisState;

/// rail 坐标系下的 thumb 几何。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThumbGeometry {
    /// thumb 在 rail 内的起点。
    pub pos: f64,
    /// thumb 长度（已 clamp）。
    pub extent: f64,
}

/// `scroll_range = max(content_extent - viewport_extent, 0)`。
pub fn scroll_range(s: &ScrollAxisState) -> f64 {
    (s.content_extent - s.viewport_extent).max(0.0)
}

/// clamp 到 `[0, scroll_range]`；NaN 折为 0（total 函数，plan r2 §14.1
/// pathological input 稳定行为）。
pub fn clamp_offset(s: &ScrollAxisState, offset: f64) -> f64 {
    if offset.is_nan() {
        return 0.0;
    }
    offset.clamp(0.0, scroll_range(s))
}

/// 滚动进度 `offset / scroll_range`；range ≤ 0 时恒 0。
pub fn progress(s: &ScrollAxisState) -> f64 {
    let range = scroll_range(s);
    if range <= 0.0 {
        return 0.0;
    }
    (clamp_offset(s, s.offset) / range).clamp(0.0, 1.0)
}

fn thumb_extent(viewport: f64, content: f64, rail_extent: f64, min_thumb_extent: f64) -> f64 {
    if rail_extent <= 0.0 || content <= 0.0 {
        return 0.0;
    }
    let ratio = (viewport / content).clamp(0.0, 1.0);
    (rail_extent * ratio).clamp(min_thumb_extent.max(0.0), rail_extent)
}

/// offset → thumb 几何（设计文档 §12.3 逐式）。
pub fn thumb_from_state(
    s: &ScrollAxisState,
    rail_extent: f64,
    min_thumb_extent: f64,
) -> ThumbGeometry {
    let extent = thumb_extent(
        s.viewport_extent,
        s.content_extent,
        rail_extent,
        min_thumb_extent,
    );
    let range = scroll_range(s);
    let travel = rail_extent - extent;
    let pos = if range <= 0.0 || travel <= 0.0 {
        0.0
    } else {
        (clamp_offset(s, s.offset) / range) * travel
    };
    ThumbGeometry { pos, extent }
}

/// thumb 拖动位置 → offset（反向映射；travel ≤ 0 时恒 0）。
pub fn offset_from_thumb_pos(
    s: &ScrollAxisState,
    thumb_pos: f64,
    rail_extent: f64,
    min_thumb_extent: f64,
) -> f64 {
    let extent = thumb_extent(
        s.viewport_extent,
        s.content_extent,
        rail_extent,
        min_thumb_extent,
    );
    let range = scroll_range(s);
    let travel = rail_extent - extent;
    if range <= 0.0 || travel <= 0.0 {
        return 0.0;
    }
    if thumb_pos.is_nan() {
        return 0.0;
    }
    clamp_offset(s, (thumb_pos / travel) * range)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(offset: f64, viewport: f64, content: f64) -> ScrollAxisState {
        ScrollAxisState {
            offset,
            viewport_extent: viewport,
            content_extent: content,
        }
    }

    #[test]
    fn range_clamps_negative_to_zero() {
        assert_eq!(scroll_range(&state(0.0, 500.0, 300.0)), 0.0); // content < viewport
        assert_eq!(scroll_range(&state(0.0, 300.0, 300.0)), 0.0); // content == viewport
        assert_eq!(scroll_range(&state(0.0, 200.0, 1000.0)), 800.0);
    }

    #[test]
    fn clamp_bounds() {
        let s = state(0.0, 200.0, 1000.0);
        assert_eq!(clamp_offset(&s, -5.0), 0.0);
        assert_eq!(clamp_offset(&s, 400.0), 400.0);
        assert_eq!(clamp_offset(&s, 9999.0), 800.0);
        assert_eq!(clamp_offset(&s, f64::NAN), 0.0);
    }

    #[test]
    fn progress_edge_cases() {
        assert_eq!(progress(&state(0.0, 200.0, 1000.0)), 0.0);
        assert_eq!(progress(&state(800.0, 200.0, 1000.0)), 1.0);
        assert_eq!(progress(&state(400.0, 200.0, 1000.0)), 0.5);
        assert_eq!(progress(&state(0.0, 500.0, 300.0)), 0.0); // range 0
        assert_eq!(progress(&state(9_999_800.0, 200.0, 10_000_000.0)), 1.0); // 大逻辑高度
    }

    #[test]
    fn thumb_proportional_without_min_clamp() {
        // viewport/content = 0.25 → thumb = rail * 0.25；travel = 120；
        // offset/range = 400/750 → pos = 400/750 * 120 = 64。
        let s = state(400.0, 250.0, 1000.0);
        let t = thumb_from_state(&s, 160.0, 0.0);
        assert_eq!(t.extent, 40.0);
        assert!((t.pos - 400.0 / 750.0 * 120.0).abs() < 1e-9);
        assert_eq!(t.pos, 64.0);
    }

    #[test]
    fn thumb_min_clamp_uses_travel_not_content_ratio() {
        // 设计文档 §12.3 陷阱：min clamp 后必须按 travel 映射。
        let s = state(0.0, 10.0, 1_000_000.0); // raw thumb ≈ 0.0016px
        let rail = 160.0;
        let min = 24.0;
        let t = thumb_from_state(&s, rail, min);
        assert_eq!(t.extent, min);
        let travel = rail - min; // 136
        let at_end = thumb_from_state(&state(999_990.0, 10.0, 1_000_000.0), rail, min);
        assert!((at_end.pos - travel).abs() < 1e-9); // offset=range → pos=travel（非 rail！）
    }

    #[test]
    fn thumb_offset_round_trip_identity() {
        let rail = 300.0;
        let min = 30.0;
        for (offset, viewport, content) in [
            (0.0, 250.0, 1000.0),
            (375.0, 250.0, 1000.0),
            (750.0, 250.0, 1000.0),
            (0.0, 10.0, 10_000_000.0), // min-thumb clamp 生效域
            (4_999_995.0, 10.0, 10_000_000.0),
            (9_999_990.0, 10.0, 10_000_000.0),
            (123.0, 500.0, 300.0), // range 0
        ] {
            let s = state(offset, viewport, content);
            let t = thumb_from_state(&s, rail, min);
            let back = offset_from_thumb_pos(&s, t.pos, rail, min);
            assert!(
                (back - clamp_offset(&s, offset)).abs() < 1e-6,
                "round trip broke at offset={offset}"
            );
        }
    }

    #[test]
    fn thumb_drag_maps_to_offset_and_back() {
        // 反向：thumb_pos → offset → thumb_pos 恒等
        let s = state(0.0, 250.0, 1000.0);
        let rail = 200.0;
        let min = 20.0;
        let travel = rail - min; // 150
        let pos = 75.0; // 中点
        let off = offset_from_thumb_pos(&s, pos, rail, min);
        assert!((off - 375.0).abs() < 1e-9); // 0.5 * 750
        let t = thumb_from_state(&state(off, 250.0, 1000.0), rail, min);
        assert!((t.pos - pos).abs() < 1e-9);
    }

    #[test]
    fn pathological_zero_rail_and_extent() {
        let s = state(10.0, 100.0, 500.0);
        assert_eq!(
            thumb_from_state(&s, 0.0, 24.0),
            ThumbGeometry {
                pos: 0.0,
                extent: 0.0
            }
        );
        assert_eq!(offset_from_thumb_pos(&s, 50.0, 0.0, 24.0), 0.0);
        let zero_content = state(0.0, 0.0, 0.0);
        assert_eq!(thumb_from_state(&zero_content, 160.0, 24.0).extent, 0.0);
        assert_eq!(progress(&zero_content), 0.0);
    }

    #[test]
    fn thumb_extent_never_exceeds_rail() {
        let s = state(0.0, 900.0, 1000.0); // ratio 0.9
        assert_eq!(thumb_from_state(&s, 160.0, 0.0).extent, 144.0);
        let tiny = state(0.0, 1000.0, 1000.0); // ratio 1.0 → full rail
        assert_eq!(thumb_from_state(&tiny, 160.0, 0.0).extent, 160.0);
    }
}
