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
pub mod debug;
pub mod vm_bridge;

#[cfg(feature = "ui-interpreter")]
pub mod mcp_types;

#[cfg(feature = "ui-interpreter")]
pub mod snapshot_builder;

#[cfg(feature = "ui-interpreter")]
pub mod action_mapper;

#[cfg(feature = "ui-interpreter")]
pub mod mcp_server;

#[cfg(feature = "ui-interpreter")]
pub mod vtree_atom;

#[cfg(feature = "ui-interpreter")]
pub mod aura_snapshot_builder;

#[cfg(feature = "ui-interpreter")]
pub mod render_support;

#[cfg(feature = "ui-interpreter")]
pub mod interpreter;

#[cfg(feature = "ui-interpreter")]
pub mod aura_view_builder;

#[cfg(feature = "ui-interpreter")]
pub mod debug_id_map;

#[cfg(feature = "ui-interpreter")]
pub mod event_router;

#[cfg(feature = "ui-interpreter")]
pub mod hot_reload;

#[cfg(feature = "ui-interpreter")]
pub mod dynamic;

#[cfg(feature = "ui-interpreter")]
pub mod widget_registry;

#[cfg(feature = "ui-interpreter")]
pub mod state_migration;

#[cfg(feature = "ui-headless")]
pub mod headless;

#[cfg(feature = "ui-iced")]
pub mod iced;

#[cfg(feature = "ui-gpui")]
pub mod gpui;

// Re-exports
pub use component::Component;
pub use view::{View, ViewBuilder};
pub use vnode::{VNodeId, VNodeKind, VNode, VNodeProps, VTree};
pub use vnode_converter::view_to_vtree;
pub use app::{App, AppResult};
pub use style::Style;
pub use debug::{DebugLayer, DebugState, Rect, LayoutReporter};
