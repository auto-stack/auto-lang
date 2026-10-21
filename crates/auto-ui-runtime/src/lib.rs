//! auto-ui-runtime — UI 运行/renderer/宿主域（PLAN-814）。
//!
//! 从 auto-lang 迁入的 VM 渲染轨与桌面宿主栈：iced renderer、desktop
//! session/protocol、编辑器/终端/媒体 widget、host adapter 与宿主能力
//! provider 实现（T-04 迁移面）。
//!
//! 边界合同：依赖 SDK（auto-lang-core）与 UI compiler（auto-ui-compiler）；
//! 不被任一反向依赖。宿主能力装配（`install_ui_host_capabilities` 族）
//! 收敛到本 crate，SDK 零隐式 UI 装配。
//!
//! r3 §5.3 边界：本 crate **不**经 inventory 登记 UI 文法表或宿主安装器
//! （登记权归旧 auto-lang 兼容包与 auto-ui-compiler）；纯 SDK 缺席/显式
//! 拒绝语义不变。

// ── 模块树（PLAN-814 T-04 自 auto-lang 整体随迁；cfg 门原名保持） ──

// Plan 015: AutoUI Core (feature-gated)
#[cfg(feature = "ui")]
pub mod ui;

// PLAN-804: 宿主 OS 能力适配层（vm core 与 ui renderer 门之外的第三
// 落点——首员 = ui-dialog 单门控的 OS 文件对话框）。
pub mod host_adapters;

// PLAN-658: 单进程多后端宿主（画廊内嵌 demo 后端 session）；PLAN-814
// T-04 随 UI 运行域迁入（media/photo 服务面，photo_service 残边清偿=T-06）。
pub mod back_proxy;

// PLAN-814 T-04：UI 动态构建管线（原 auto-lang lib.rs Plan 015 UI glue 区
// 整体随迁）——collect_module_imports 模块装载链 → build_dynamic_component
// → run_file_dynamic_ui。公开面经 crate 根再导出，旧 crate 根路径不变。
mod dynamic_build;

// Plan 370 shared test support: builds the real 015-notes DynamicComponent.
// Must precede the plan370_* test modules that depend on it（auto-lang 侧
// 测试树经再导出消费）。
#[cfg(all(test, feature = "ui-iced"))]
pub(crate) mod plan370_test_support;

#[cfg(test)]
mod ui_tests;

// ── 根再导出：旧 crate 根路径在 runtime 内保持（迁移引用零改写） ──

pub use auto_lang_core::{AutoResult, Parser, UseModuleResolution};

// Re-export UI types when feature is enabled（与旧 auto-lang 根面同集）
#[cfg(feature = "ui")]
pub use ui::{
    view_to_vtree, App, AppResult, Component, Style, VNode, VNodeId, VNodeKind, VNodeProps, VTree,
    View, ViewBuilder,
};

#[cfg(feature = "ui-interpreter")]
pub use ui::{
    event_router::{EventContext, EventRouter, EventType},
    hot_reload::{HotReloadComponent, UIWatcher},
    interpreter::{DynamicMessage, InterpreterBridge},
};

#[cfg(feature = "ui-iced")]
pub use ui::iced::{ComponentIced, IntoIcedElement};

// dynamic_build 管线公开面（原 auto-lang lib.rs 根项，cfg 门原样）。
// PLAN-814 R8-03：load_ext_imports_for_vm 唯一 crate 内消费方是
// test-gated 的 plan370_test_support——再导出门与消费方同门（非测试
// lib 构建不再产生未用再导出告警）；stmt_symbol_name 仅模块内消费，
// 根再导出删除（R5 cfg(test) 条件声明同款纪律）。
pub(crate) use dynamic_build::collect_module_imports;
// stmt_symbol_name 消费方（vm_bridge 测试 mod/plan339）均 test-gated。
#[cfg(all(test, feature = "ui-iced"))]
pub(crate) use dynamic_build::load_ext_imports_for_vm;
#[cfg(feature = "ui-iced")]
pub use dynamic_build::run_file_dynamic_ui_with_scenario;
#[cfg(test)]
pub(crate) use dynamic_build::stmt_symbol_name;
pub use dynamic_build::ui_module_parse_failures;
#[cfg(feature = "ui-iced")]
pub use dynamic_build::{build_dynamic_component, has_ui_keywords, run_file_dynamic_ui};

// ── 宿主能力装配单点（原 auto-lang 实装随迁；文法装配留旧兼容包） ──
//
// PLAN-814 T-04：三处装配扇入（旧 lib.rs 安装器/run_file_dynamic_ui_inner/
// ui::vm_bridge）收敛本 crate 单点。r3 §5.3：显式入口优先——本安装器不
// 登记 UI 文法（auto-ui-dialect 依赖与 UiGrammarSource 提交权归旧
// auto-lang 兼容包），缺文法时 SDK 面显式拒绝语义保持。

/// UI 宿主能力 provider 装配单点（frame/clipboard/scroll/terminal/
/// code-editor/dnd/dialog + autodown 引擎 + i18n lookup 钩子）。
///
/// PLAN-804: 两槽并入（2927/2928 + 2938）——本入口的门同时扩为
/// `any(ui, ui-dialog)`：dialog adapter 走 ui-dialog 单门控，**ui-dialog
/// only（无 ui）构建恢复迁移前高层对话框路径**（不把 OS 实现藏在 ui
/// renderer 门后）；dnd adapter 走 windows+native-dnd 双门控（模块本身
/// 在 ui 门内=三分），缺席槽=false/VMError 同旧。
#[cfg(any(feature = "ui", feature = "ui-dialog"))]
pub fn install_ui_host_capabilities() {
    // PLAN-814: i18n lookup 钩子注册（原 shim_i18n_t 的 ui 直连边收口；
    // i18n_lookup 模块在 ui-interpreter 门——钩子注册随同门，R9-03 补齐）。
    #[cfg(feature = "ui-interpreter")]
    auto_lang_core::set_i18n_lookup_hook(crate::ui::i18n_lookup::lookup);
    #[cfg(feature = "ui")]
    {
        crate::ui::frame_provider::install_process_default();
        // PLAN-814 R3-04：autodown 文档引擎（feature 同 owner；SDK 侧
        // natives dispatch 槽未设时经宿主钩子回调至此注册）。
        #[cfg(feature = "autodown")]
        crate::ui::autodown_engine::install();
        #[cfg(feature = "ui-clipboard")]
        crate::ui::clipboard_provider::install_process_default();
        #[cfg(all(windows, feature = "native-clipboard"))]
        crate::ui::clipboard_media_provider::install_process_default();
        crate::ui::scroll_host_provider::install_process_default();
        // PLAN-808: terminal 域 adapter（ui 单门控——引擎服务 + UI 输入/
        // 投影/menu/scroll/window/palette 接线恢复；34 native 全族）。
        crate::ui::terminal_provider::install_process_default();
        // PLAN-802: code editor host adapter（code-editor 单门控——editor
        // 注册表接线恢复，22 native 全族）。
        #[cfg(feature = "code-editor")]
        crate::ui::code_editor_provider::install_process_default();
        // PLAN-804: dnd adapter（委托 ui::native_dnd 既有
        // parse_payload_json + win32::start_drag；模块在 ui 门内，此处
        // 双门控即有效三分 windows ∧ native-dnd ∧ ui）。
        #[cfg(all(windows, feature = "native-dnd"))]
        crate::ui::dnd_host_provider::install_process_default();
    }
    // PLAN-804: dialog adapter（host_adapters 落点，ui-dialog 单门控——
    // 与 ui renderer 门无关，ui-dialog-only 形态同装）。
    #[cfg(feature = "ui-dialog")]
    crate::host_adapters::dialog::install_process_default();
}

/// 无 renderer 构建形态：纯 no-op 安装器（旧 auto-lang 同形——文法装配
/// 与钩子注册在旧兼容包侧完成，本 crate 无 UI 实现可装）。
#[cfg(not(any(feature = "ui", feature = "ui-dialog")))]
pub fn install_ui_host_capabilities() {}
