//! PLAN-656: AutoUI 通用滚动架构——后端无关核心语义层（设计文档 Phase A，
//! [docs/design/autoui/universal-scroll-architecture.md]）。
//!
//! 本模块是 Scroll Architecture 的语义单源：逻辑滚动空间统一 `f64`，进入具体
//! backend（iced f32 / DOM number）时才由 adapter 转换。核心类型**禁止**依赖
//! iced / DOM / widget id——`ScrollController` 的 runtime binding、echo suppression
//! 等实现细节属于 backend adapter 私有层（plan r2 §5.2）。
//!
//! 三条 hosting 语义通道（见 `host.rs`）：content→pane `ScrollState`；
//! pane→content `ScrollIntent` 与 `ScrollViewportState`。
//!
//! 刻意不落地的类型：`ScrollAnchor`（无真实 consumer 前不固定 `ItemKey` 抽象，
//! Phase D/E 再设计）与 core 级 `ScrollControllerId`（controller 是 logical
//! handle，binding id 属 runtime 私有类型）。
//!
//! PLAN-781 T-01 更新（2026-10-14）：子内容锚点已按 [781 接口决策]
//! (../../plans/reports/781-interface-decision.md) 落地——锚点 key 为业务
//! 字符串（DSL `scroll-anchor`），不引入 `ItemKey` 泛型抽象；请求队列与
//! nearest 解析在 `controller.rs`（runtime 私有层），纯几何在 `geometry.rs`
//! `nearest_child_offset`。core 语义层仍不引入 anchor 类型。

pub mod controller;
pub mod geometry;
pub mod host;
pub mod intent;
pub mod managed;
pub mod state;

pub use controller::{
    bind_controller, controller_snapshot, drain_resolved_intents, enqueue_intent,
    is_controller_handle, next_controller_handle, note_controller_state, CONTROLLER_HANDLE_PREFIX,
};
pub use geometry::{
    clamp_offset, offset_from_thumb_pos, progress, scroll_range, thumb_from_state, ThumbGeometry,
};
pub use host::{
    ScrollContentHost, ScrollContentHostRecord, SyntheticManagedContent,
    SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_H, SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_W,
};
pub use intent::{ResolvedScrollIntent, ScrollIntent, ScrollSource};
pub use state::{
    Axis, ScrollAxes, ScrollAxisState, ScrollState, ScrollViewportState, ScrollbarPolicy,
};

/// PLAN-783 T-19：controller pane 的 iced widget id 派生单源（当前 = 裸
/// 内插 `scroll_ctl_{handle}`——F-781-D01 调查中：@scrollctl 前缀句柄使
/// iced scroll 操作全局失效的机制未明（D:/p781-t19/live*.log 最小复现），
/// 哈希派生 containment 实测引入自身回归（静态句柄也失效）已回滚；待 D01
/// 根因清晰后再定安全派生形态。
pub fn controller_widget_id(handle: &str) -> String {
    format!("scroll_ctl_{handle}")
}
