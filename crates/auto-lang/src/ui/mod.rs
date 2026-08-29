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
#[cfg(feature = "ui-iced")]
pub mod session;

// Plan 463 T2：桌面布局引擎（free/grid/master-stack 纯函数 + snap 几何）。
#[cfg(feature = "ui-iced")]
pub mod layout;

// Plan 463 T5：桌面 shell（特权 .at App 装配）。
pub mod shell;

// Plan 463 T7：应用注册表（apps 目录扫描 → LaunchApp 目标清单）。
pub mod app_registry;

// Plan 386 Stage 1：桌面协议（进程外 App 五通道）——loopback 同进程走通，
// 施工图 Design 25 §7；Stage 2 换真 transport 时只替换其 loopback 层。
#[cfg(feature = "ui-iced")]
pub mod desktop_protocol;

// Plan 442 A3: web-ecosystem ext imports on the VM render target
// (adapter-chain loading + platform stubs).
pub mod ext_stubs;

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
// Plan 019 批次七: autodown-core crate 消费（VM markdown/autodown 真渲染）。
#[cfg(feature = "autodown")]
pub mod autodown_render;

// Plan 019 Phase 3: autodown 文档编辑壳（cosmic-text 块缓冲 + 焦点导航）。
// 双 feature 门控：块模型单源（autodown）× cosmic-text 栈（code-editor）。
#[cfg(all(feature = "autodown", feature = "code-editor"))]
pub mod autodown_editor;

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

#[cfg(feature = "ui-interpreter")]
pub mod i18n_lookup;

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
