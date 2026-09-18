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

pub mod controller;
pub mod geometry;
pub mod host;
pub mod intent;
pub mod managed;
pub mod state;

pub use controller::{
    CONTROLLER_HANDLE_PREFIX, bind_controller, controller_snapshot, drain_resolved_intents,
    enqueue_intent, is_controller_handle, next_controller_handle, note_controller_state,
};
pub use geometry::{
    ThumbGeometry, clamp_offset, offset_from_thumb_pos, progress, scroll_range, thumb_from_state,
};
pub use host::{
    SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_H, SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_W,
    ScrollContentHost, ScrollContentHostRecord, SyntheticManagedContent,
};
pub use intent::{ResolvedScrollIntent, ScrollIntent, ScrollSource};
pub use state::{
    Axis, ScrollAxes, ScrollAxisState, ScrollState, ScrollViewportState, ScrollbarPolicy,
};
