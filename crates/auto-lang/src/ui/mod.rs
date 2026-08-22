// auto-lang/crates/auto-lang/src/ui/mod.rs
//! AutoUI Core - Backend-agnostic UI abstraction
//!
//! This module provides the core UI abstraction layer that can be adapted
//! to multiple backends (GPUI, Iced, Vue.js, etc.) through a unified
//! Component trait and View system.

// Re-export `auto_val` so generated rust-mode code (which only depends on
// auto-lang) can name `auto_val::Value` in its `Component::state_snapshot`
// override (Plan 371 Task 21). Accessed as `auto_lang::ui::auto_val::Value`.
pub use auto_val;

pub mod component;

// Plan 413: cross-platform code editor widget (feature `code-editor`,
// enabled by default under `ui-iced`).
#[cfg(feature = "code-editor")]
pub mod code_editor;
// Plan 418: OS clipboard bridge (arboard) behind `ui-clipboard`.
#[cfg(feature = "ui-clipboard")]
pub mod clipboard;
// Plan 418 Phase 2: declarative action/binding config (auto-atom).
pub mod action_config;
pub mod view;
pub mod vnode;
pub mod vnode_converter;
pub mod node_converter;
pub mod app;
pub mod widget;
pub mod style;
pub mod debug;
pub mod vm_bridge;
pub mod handler_codegen;

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

// Plan 365 W1: Unified host backend interface (seam for W2/W3).
// Available whenever the base `ui` feature is on; each variant is individually
// cfg-gated inside the module.
#[cfg(feature = "ui")]
pub mod host;

// Re-exports
pub use component::Component;
pub use view::{View, ViewBuilder};
pub use vnode::{VNodeId, VNodeKind, VNode, VNodeProps, VTree};
pub use vnode_converter::view_to_vtree;
pub use app::{App, AppResult};
pub use host::HostBackend;
pub use style::Style;
pub use debug::{DebugLayer, DebugState, Rect, LayoutReporter};
