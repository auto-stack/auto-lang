//! PLAN-656: 滚动状态与轴模型（plan r2 §3.1–§3.3）。
//!
//! 区分「某一个轴」（[`Axis`]，`ScrollIntent` 使用）与「pane 开启哪些轴」
//! （[`ScrollAxes`]，pane 配置使用）——`axis: both` 在类型层闭合为
//! `ScrollAxes::BOTH`，而不是把双轴塞进单轴 `Axis`。

/// 单个滚动轴。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
}

/// pane 开启的轴集合（`axis: y|x|both`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScrollAxes {
    pub x: bool,
    pub y: bool,
}

impl ScrollAxes {
    pub const X: Self = Self { x: true, y: false };
    pub const Y: Self = Self { x: false, y: true };
    pub const BOTH: Self = Self { x: true, y: true };

    pub fn contains(self, axis: Axis) -> bool {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }

    /// AutoLang `axis:` prop 值（`y` / `x` / `both`）；未知值回落 `Y`（parser
    /// 层已 one_of 校验，这里是防御性 total 函数）。
    pub fn from_keyword(keyword: &str) -> Self {
        match keyword {
            "x" | "horizontal" => Self::X,
            "both" => Self::BOTH,
            _ => Self::Y,
        }
    }
}

/// 单轴滚动状态（逻辑空间，f64）。
///
/// 不变量（见 `geometry::scroll_range`）：
/// `scroll_range = max(content_extent - viewport_extent, 0)` 且
/// `offset ∈ [0, scroll_range]`——约束由 geometry 纯函数在消费点执行，结构体
/// 本身只承载数据。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollAxisState {
    pub offset: f64,
    pub viewport_extent: f64,
    pub content_extent: f64,
}

impl ScrollAxisState {
    /// 退化态：零 extent、零 offset（未启用轴的观测投影，plan r2 §7.1）。
    pub const fn degenerate() -> Self {
        Self { offset: 0.0, viewport_extent: 0.0, content_extent: 0.0 }
    }
}

/// 双轴滚动状态；未启用的轴为 `None`（读出侧投影为退化值，见
/// [`ScrollState::axis_or_degenerate`]）。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScrollState {
    pub x: Option<ScrollAxisState>,
    pub y: Option<ScrollAxisState>,
}

impl ScrollState {
    pub fn axis_state(&self, axis: Axis) -> Option<ScrollAxisState> {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }

    /// 未启用轴读取退化态（offset=0 / extent=0），保证 resolve/geometry 的
    /// total 行为；启用轴正常返回。
    pub fn axis_or_degenerate(&self, axis: Axis) -> ScrollAxisState {
        self.axis_state(axis).unwrap_or(ScrollAxisState::degenerate())
    }

    pub fn with_axis(mut self, axis: Axis, state: ScrollAxisState) -> Self {
        match axis {
            Axis::X => self.x = Some(state),
            Axis::Y => self.y = Some(state),
        }
        self
    }
}

/// pane → hosted content 的 viewport 下发通道（plan r2 §3.3）。
///
/// virtual-list 可见窗口、terminal rows/cols、editor visible lines 都由它驱动；
/// resize / 初次挂载 / layout 变化时 backend 必须重新下发。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScrollViewportState {
    pub width: f64,
    pub height: f64,
}

/// `scrollbar:` 可见策略（plan r2 §8）。宽度/颜色/hover 动画归 Theme。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollbarPolicy {
    /// backend 默认可见策略。
    Auto,
    /// 尽可能保持可见（iced 0.14 无 idle auto-hide 时与 Auto 视觉等价，spec
    /// 记录为实现限制，非语义降级）。
    Always,
    /// 无可见 rail/thumb 且**无 scrollbar hit target**；wheel/touch/keyboard/
    /// controller 滚动仍可用（语义红线：不得「透明但可拖」）。
    Hidden,
}

impl ScrollbarPolicy {
    pub fn from_keyword(keyword: &str) -> Self {
        match keyword {
            "always" => Self::Always,
            "hidden" => Self::Hidden,
            _ => Self::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_axes_consts_and_contains() {
        assert!(ScrollAxes::Y.contains(Axis::Y));
        assert!(!ScrollAxes::Y.contains(Axis::X));
        assert!(ScrollAxes::X.contains(Axis::X));
        assert!(ScrollAxes::BOTH.contains(Axis::X));
        assert!(ScrollAxes::BOTH.contains(Axis::Y));
    }

    #[test]
    fn scroll_axes_from_keyword() {
        assert_eq!(ScrollAxes::from_keyword("y"), ScrollAxes::Y);
        assert_eq!(ScrollAxes::from_keyword("x"), ScrollAxes::X);
        assert_eq!(ScrollAxes::from_keyword("both"), ScrollAxes::BOTH);
        // legacy direction 词表同表映射（vertical/horizontal/both）
        assert_eq!(ScrollAxes::from_keyword("horizontal"), ScrollAxes::X);
        // 防御性回落
        assert_eq!(ScrollAxes::from_keyword("nonsense"), ScrollAxes::Y);
    }

    #[test]
    fn degenerate_axis_projection() {
        let state = ScrollState { x: None, y: Some(ScrollAxisState { offset: 10.0, viewport_extent: 100.0, content_extent: 500.0 }) };
        assert_eq!(state.axis_or_degenerate(Axis::X), ScrollAxisState::degenerate());
        assert_eq!(state.axis_or_degenerate(Axis::Y).offset, 10.0);
    }

    #[test]
    fn scrollbar_policy_keywords() {
        assert_eq!(ScrollbarPolicy::from_keyword("auto"), ScrollbarPolicy::Auto);
        assert_eq!(ScrollbarPolicy::from_keyword("always"), ScrollbarPolicy::Always);
        assert_eq!(ScrollbarPolicy::from_keyword("hidden"), ScrollbarPolicy::Hidden);
        assert_eq!(ScrollbarPolicy::from_keyword("typo"), ScrollbarPolicy::Auto);
    }

    #[test]
    fn f64_large_extent_precision() {
        // 10,000,000px 逻辑高度下 offset 精度仍远高于亚像素阈值（f64 尾数 52bit）。
        let s = ScrollAxisState { offset: 9_999_000.5, viewport_extent: 600.0, content_extent: 10_000_000.0 };
        assert_eq!(s.offset, 9_999_000.5_f64);
        let next = s.offset + 0.5;
        assert_eq!(next - s.offset, 0.5);
    }
}
