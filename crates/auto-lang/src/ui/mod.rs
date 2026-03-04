// auto-lang/crates/auto-lang/src/ui/mod.rs
//! AutoUI Core - Backend-agnostic UI abstraction
//!
//! This module provides the core UI abstraction layer that can be adapted
//! to multiple backends (GPUI, Iced, Vue.js, etc.) through a unified
//! Component trait and View system.

pub mod component;
pub mod view;
pub mod vnode;
pub mod vnode_converter;
pub mod node_converter;
pub mod app;
pub mod widget;
pub mod style;

#[cfg(feature = "ui-interpreter")]
pub mod interpreter;

#[cfg(feature = "ui-interpreter")]
pub mod event_router;

#[cfg(feature = "ui-interpreter")]
pub mod hot_reload;

// Re-exports
pub use component::Component;
pub use view::{View, ViewBuilder};
pub use vnode::{VNodeId, VNodeKind, VNode, VNodeProps, VTree};
pub use vnode_converter::view_to_vtree;
pub use app::{App, AppResult};
pub use style::Style;
