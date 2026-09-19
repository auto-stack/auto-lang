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

// Plan 547: backend-neutral media runtime.  Concrete registries and worker
// machinery are feature-gated so default language builds stay image-free.
#[cfg(feature = "image-pipeline")]
pub mod image_pipeline;

// Plan 617 T-05: local media file service (recursive index + HTTP byte-range
// semantics).  Needs no optional dependency (blake3 is unconditional), so it is
// ungated like `component` — the generated backend reaches it as
// `auto_lang::ui::media_service`.
pub mod media_service;

// PLAN-617 T-16: VM/iced 端原生播放引擎（libmpv DLL 运行时加载）。可选能力，
// gated behind `mpv-native`：它是**运行时**依赖——没装 mpv 的机器照常编译、照常
// 过 CI，只是运行期走降级（AC-20）。为什么是 libmpv 而非 ffmpeg FFI、为什么是 SW
// 后端：见 docs/design/autoui/030-video-player.md §4（T-15 的门控实测与四选一裁定）。
#[cfg(feature = "mpv-native")]
pub mod mpv;
// （帧上屏的那一半 `mpv::channel`/`mpv::present` 由 `mpv-gpu` 单独门控。）
// 014 内存哨兵:提交内存自检 + 超限冻结(渲染层 update 入口挂接;
// AUTO_MEM_LIMIT_MB 阈值,0 = 关闭)。零依赖(直接声明 K32GetProcessMemoryInfo)。
pub mod mem_guard;

// Plan 413: cross-platform code editor widget (feature `code-editor`,
// enabled by default under `ui-iced`).
#[cfg(feature = "code-editor")]
pub mod code_editor;
// PLAN-009 P1: native `terminal` component (auto-term engine grid; core is
// iced-free, `iced/` is the only iced point — code_editor layering).
pub mod terminal;
// PLAN-656: AutoUI 通用滚动架构核心语义层（backend-independent——
// State/Intent/Viewport 三通道 + geometry 纯函数 + hosting contract；
// iced/Vue adapter 在各自 backend 内，不在此处）。
pub mod scroll;
// Plan 418: OS clipboard bridge (arboard) behind `ui-clipboard`.
#[cfg(feature = "ui-clipboard")]
pub mod clipboard;
// Plan 485: native clipboard bridge (CF_HDROP files / DIBV5+PNG images).
// Pure codec helpers compile on every tier (`cargo t clipboard_native`);
// Win32 calls are double-gated inside (windows × `native-clipboard`).
pub mod clipboard_native;
// Plan 418 Phase 2: declarative action/binding config (auto-atom).
pub mod action_config;
pub mod view;
// PLAN-063 T-04d-2: 右栏块锚定坐标槽（iced 布局期记录 + 同步目标消费）。
#[cfg(feature = "ui-iced")]
pub mod anchor_slot;
pub mod vnode;
pub mod vnode_converter;
pub mod node_converter;
pub mod app;
pub mod widget;
pub mod style;
pub mod debug;
// PLAN-646 Select Anything——框选语义纯函数 + 结果信封。依赖
// mcp_server/vtree_atom（StyledNodeSnapshot/VTreeAtomBuilder），同门控。
#[cfg(feature = "ui-interpreter")]
pub mod selection;
pub mod vm_bridge;
pub mod handler_codegen;
// PLAN-051 C2：子→父 msg 参数回调通用路由表（handler_codegen 无 feature 门，
// 故本模块同样不门控）。
pub mod child_emit;
#[cfg(feature = "ui-iced")]
pub mod session;
pub mod shell_projection;
// PLAN-615 T-06: OS 系统主题读取（深/浅色个人化跟随地基；纯 std 全平台可编译）。
pub mod system_theme;

// Plan 463 T2：桌面布局引擎（free/grid/master-stack 纯函数 + snap 几何）。
#[cfg(feature = "ui-iced")]
pub mod layout;

// Plan 463 T5：桌面 shell（特权 .at App 装配）。
pub mod shell;

// Plan 463 T7：应用注册表（apps 目录扫描 → LaunchApp 目标清单）。
pub mod app_registry;

// Plan 501：os-config daemon 生命周期（检活/发现序 spawn/AUTOOS_DAEMON env
// 注入——外部仓 settings app 的宿主侧底座；纯逻辑全平台单测）。
pub mod osconfig_daemon;

// Plan 504 S7：os-config 应用配置读取（~/.config/autoos/apps/<app>/config.at
// 只读；run 臂优先级链 + desktop launch 播种两个消费点）。
pub mod osconfig_apps;

// Plan 540 M1：桌面单源配置（~/.config/autoos/apps/desktop/config.at——
// boot 读 + 设置窗经宿主臂写 + os-config 通用编辑器同文件）。
pub mod desktop_config;

// Plan 386 Stage 1：桌面协议（进程外 App 五通道）——loopback 同进程走通，
// 施工图 Design 25 §7；Stage 2 换真 transport 时只替换其 loopback 层。
#[cfg(feature = "ui-iced")]
pub mod desktop_protocol;

// Plan 473：原生窗口 dock（NativeSlot，假洞 Phase 1）。纯逻辑层零依赖全平台
// 单测；Win32 适配 #[cfg(windows)] 门控在 native_dock/win32.rs，非 Windows
// 以同名 no-op 模块顶替。
pub mod native_dock;

// Plan 488：原生互操作 Phase 3——OLE 拖放双向（payload 模型全平台可编译；
// Win32/COM 调用 #[cfg(all(windows, feature = "native-dnd"))] 门控在模块内
// 分节，未开 feature 时 natives 走降级 shim）。
pub mod native_dnd;

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

// PLAN-041 T1: 块家族注册表——chrome/样式单源，只读臂与编辑壳共同消费。
#[cfg(feature = "autodown")]
pub mod autodown_blocks;

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

// PLAN-066: 原生组件外部注册 SPI（NativeWidgetRegistry + View::Custom 派发
// 通道）。跟随 ui-interpreter 门控——消费方（aura_view_builder 派发 /
// iced renderer lowering）都在其内。
#[cfg(feature = "ui-interpreter")]
pub mod native_widget;

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
