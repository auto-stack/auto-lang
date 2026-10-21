// auto-lang/crates/auto-lang/src/ui/mod.rs
//! AutoUI Core - Backend-agnostic UI abstraction
//!
//! This module provides the core UI abstraction layer that can be adapted
//! to multiple backends (Iced desktop, Vue.js web, etc.) through a unified
//! Component trait and View system.

// Re-export `auto_val` so generated rust-mode code (which only depends on
// auto-lang) can name `auto_val::Value` in its `Component::state_snapshot`
// override (Plan 371 Task 21). Accessed as `auto_lang::ui::auto_val::Value`.
pub use auto_val;

pub mod component;

// PLAN-716 组B: 帧时间戳观测通道（供②——AUTO_FRAME_BENCH 门控零开销；
// VM 内建 9918/9919 + a2r 臂同源，SD-B 契约）。
pub mod frame_bench;

// PLAN-779: UI host frame provider adapter——读 frame_bench 通道，经
// vm::host_capabilities 显式装配（core shim 9918/9919 不再直引 ui）。
pub mod frame_provider;

// PLAN-725 T-00: 键入帧五段成本链分段探针（S1..S4+孤儿帧——AUTO_FRAME_BENCH
// 同门复用，[P725-FRAME] 行随帧吐出，阶梯谱脚本消费）。
pub mod frame_segments;

// PLAN-721 T-1: 桌面轨更新路径家族诊断通道（AUTO_SCHED_DIAG 门控——泵/
// 恢复链路订阅装配/消息到达/泵臂消费/置脏传播的统一 trace 面）。
pub mod sched_diag;

// Plan 547: backend-neutral media runtime.  Concrete registries and worker
// machinery are feature-gated so default language builds stay image-free.
#[cfg(feature = "image-pipeline")]
pub mod image_pipeline;

// Plan 617 T-05: local media file service (recursive index + HTTP byte-range
// semantics).  Needs no optional dependency (blake3 is unconditional), so it is
// ungated like `component` — the generated backend reaches it as
// `auto_lang::ui::media_service`.
pub mod media_service;

// PLAN-043 Part 1: local photo file service (recursive index + on-demand
// thumbnail rendering + original byte serving) for 029-photo-gallery.
// Gated on `image-pipeline` — thumbnails need the `image`/`kamadak-exif`
// decoders (media_service stays ungated because it never decodes pixels).
#[cfg(feature = "image-pipeline")]
pub mod photo_service;

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
// PLAN-808: UI terminal host-provider adapter — composes the auto-terminal-core
// engine service with the UI view/input registry; VM shims only codec.
pub mod terminal_provider;
// PLAN-656: AutoUI 通用滚动架构核心语义层（backend-independent——
// State/Intent/Viewport 三通道 + geometry 纯函数 + hosting contract；
// iced/Vue adapter 在各自 backend 内，不在此处）。
pub mod scroll;
// Plan 418: OS clipboard bridge (arboard) behind `ui-clipboard`.
#[cfg(feature = "ui-clipboard")]
pub mod clipboard;
// PLAN-785: clipboard text host provider adapter——委托 ui::clipboard 既有
// 实现，经 vm::host_capabilities 显式装配（core shim 2925/2926 不再直引
// ui；779 frame_provider 同构）。
#[cfg(feature = "ui-clipboard")]
pub mod clipboard_provider;
// PLAN-790: clipboard media host provider adapter——委托 ui::clipboard_native
// 既有实现（files/image 2934..2937；779/785 同构）。双门控 windows ×
// native-clipboard 与被委托的 Win32 桥函数一致——非 Windows 无 adapter，
// VM 缺席 provider → G3 值降级，不把 UI 默认伪造为可用 OS。
#[cfg(all(windows, feature = "native-clipboard"))]
pub mod clipboard_media_provider;
// PLAN-794: scroll host provider adapter——委托 ui::scroll::controller 既有
// 注册表/锚点状态机（9900..9905 + 9922/9923；779/785/790 同构）。单门控
// `ui`（与被委托的 scroll 模块一致——队列层纯 Rust 无 OS 面）；renderer
// binding/measured 状态仍为单一真源，adapter 只做 core DTO 转换。
pub mod scroll_host_provider;
// PLAN-802: code editor host provider adapter——委托 ui::code_editor 既有
// 注册表（22 native 全族；779/785/790/794 同构）。单门控 `code-editor`
//（与被委托的 code_editor 模块一致）；cut/copy/paste 实现级第二门控
// `ui-clipboard` 在方法体内（无此 feature 返 None = 组合特征 raise 臂）。
// editor 注册表仍是单一真源，adapter 只做 core 值域转换。
#[cfg(feature = "code-editor")]
pub mod code_editor_provider;
// PLAN-681: native open/save file dialogs — host of the `dialog_open`/
// `dialog_save` VM builtins (natives 2927/2928) for a2r generated apps,
// behind `ui-dialog` (enabled by `ui-iced`). PLAN-804 起为薄门面（单一
// OS 实现在 host_adapters::dialog，本模块保 a2r unparented 策略）。
#[cfg(feature = "ui-dialog")]
pub mod dialog;
// PLAN-804: dnd host provider adapter——委托 ui::native_dnd 既有载荷
// parser 与 Win32 会话实现（2938；779/785/790/794 同构）。双门控
// windows × native-dnd（模块在 ui 门内=有效三分，与迁移前 shim 门逐字
// 相同）；无 adapter 形态 VM 缺席 provider → false 降级。
#[cfg(all(windows, feature = "native-dnd"))]
pub mod dnd_host_provider;
// Plan 485: native clipboard bridge (CF_HDROP files / DIBV5+PNG images).
// Pure codec helpers compile on every tier (`cargo t clipboard_native`);
// Win32 calls are double-gated inside (windows × `native-clipboard`).
pub mod clipboard_native;
// Plan 418 Phase 2: declarative action/binding config (auto-atom).
// PLAN-814 T-03：action 配置解析迁 auto-ui-compiler；旧路径经再导出保持。
pub use auto_ui_compiler::action_config;
pub mod view;
// PLAN-063 T-04d-2: 右栏块锚定坐标槽（iced 布局期记录 + 同步目标消费）。
#[cfg(feature = "ui-iced")]
pub mod anchor_slot;
pub mod app;
pub mod debug;
pub mod node_converter;
// PLAN-814 T-03：style DSL 分析/模型单源迁 auto-ui-compiler。
// PLAN-814 R9-02/R9-03：style 从整体别名改为门面模块——①compiler DSL 面
// 经 glob 再导出（旧 style::* 路径保持）；②旧公开 iced 路径恢复
// （style::IcedStyle / style::iced_adapter::*——同一类型单源=
// style_iced_adapter，不复制实现）；③适配器与再导出同上 ui-iced 门
// （headless 构建零 iced 引用——203e2ce91 同款门）。
pub mod style {
    #[cfg(feature = "ui-iced")]
    pub use crate::ui::style_iced_adapter::IcedStyle;
    pub use auto_ui_compiler::style::*;
    #[cfg(feature = "ui-iced")]
    pub mod iced_adapter {
        pub use crate::ui::style_iced_adapter::*;
    }
}
#[cfg(feature = "ui-iced")]
pub mod style_iced_adapter;
#[cfg(feature = "ui-iced")]
pub use style_iced_adapter::IcedStyle;
pub mod vnode;
pub mod vnode_converter;
pub mod widget;
// PLAN-646 Select Anything——框选语义纯函数 + 结果信封。依赖
// mcp_server/vtree_atom（StyledNodeSnapshot/VTreeAtomBuilder），同门控。
pub mod handler_codegen;
#[cfg(feature = "ui-interpreter")]
pub mod selection;
// PLAN-814 R9-03：vm_bridge 类型面依赖 interpreter（dynamic/memo 字段族
// 与 ui::dynamic 消费）——随 ui-interpreter 门（旧巨树 bare-ui 同为预存
// 缺门，本批补齐声明面）。
#[cfg(feature = "ui-interpreter")]
pub mod vm_bridge;
// PLAN-045：VM 渲染组件级 memo（菜单族四件 + sidebar nav 块；opt-in，
// 缺省原始路径逐字节不变）。类型面依赖 interpreter 的 View/DynamicMessage，
// 同门控。
#[cfg(feature = "ui-interpreter")]
pub mod memo_deps;
// PLAN-051 C2：子→父 msg 参数回调通用路由表（handler_codegen 无 feature 门，
// 故本模块同样不门控）。
pub mod child_emit;
#[cfg(feature = "ui-iced")]
pub mod session;
// PLAN-042 T-01：宿主系统日志环（三层采集归一：log trap / syslog! 双写宏 /
// App log 动词）。纯 std + log，零 feature 门（headless 单测同轨可用）。
pub mod syslog;
// PLAN-037：桌面后端供给层（launch 决策树四臂 + 按需 proxy 装载 +
// 前端 URL 内存态前缀化；658 谓词/前缀化函数迁入 auto-man 复用）。
#[cfg(feature = "ui-iced")]
pub mod back_provision;
pub mod shell_projection;
// PLAN-615 T-06: OS 系统主题读取（深/浅色个人化跟随地基；纯 std 全平台可编译）。
pub mod system_theme;

// Plan 463 T2：桌面布局引擎（free/grid/master-stack 纯函数 + snap 几何）。
#[cfg(feature = "ui-iced")]
pub mod layout;

// Plan 463 T5：桌面 shell（特权 .at App 装配）。
pub mod shell;

// Plan 463 T7：应用注册表（apps 目录扫描 → LaunchApp 目标清单）。
// PLAN-815 T-04: moved to auto-ui-compiler; re-export keeps paths.
pub use auto_ui_compiler::app_registry;

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
// PLAN-814 R3-04：autodown 文档引擎实装（跨仓 autodown-core 依赖与
// `autodown` feature 同 owner 在本包）——经 SDK 端口注册。
#[cfg(feature = "autodown")]
pub mod autodown_blocks;
// PLAN-814 R3-04：autodown 文档引擎实装（跨仓 autodown-core 依赖与
// `autodown` feature 同 owner 在本包）——经 SDK 端口注册。
#[cfg(feature = "autodown")]
pub mod autodown_engine;

// PLAN-084 T-01b: VM 轨 LaTeX→Unicode 数学符号转换（真渲染 bounded 子集）。
#[cfg(feature = "autodown")]
pub mod autodown_math;

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
pub use auto_ui_compiler::i18n_lookup;

#[cfg(feature = "ui-headless")]
pub mod headless;

#[cfg(feature = "ui-iced")]
pub mod iced;

// Plan 365 W1: Unified host backend interface (seam for W2/W3).
// Available whenever the base `ui` feature is on; each variant is individually
// cfg-gated inside the module.
#[cfg(feature = "ui")]
pub mod host;

// Re-exports
pub use app::{App, AppResult};
pub use component::Component;
pub use debug::{DebugLayer, DebugState, LayoutReporter, Rect};
pub use host::HostBackend;
pub use style::Style;
pub use view::{View, ViewBuilder};
pub use vnode::{VNode, VNodeId, VNodeKind, VNodeProps, VTree};
pub use vnode_converter::view_to_vtree;
