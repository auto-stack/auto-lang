//! auto-ui-compiler — UI 编译/生成域（PLAN-814）。
//!
//! 从 auto-lang 迁入的 UI 分析（aura 提取/schema）、设计 token 与
//! vue/jet/ark/rust 四生成器及 widget registry（T-03 迁移面）。
//!
//! 边界合同：依赖纯语言 SDK（auto-lang-core），不被 SDK 反向依赖；
//! 生成的 runtime import 路径策略单源在本 crate；跨层集成测试归上层。

// ---- PLAN-814 T-03 Batch 1：style DSL 分析/模型、action 配置解析、lucide 表 ----
// （iced 后端适配器留 auto-lang ui/style_iced_adapter.rs——runtime 件。）
pub mod action_config;
// PLAN-815 T-04: 画廊 app 元信息扫描 + i18n 查询（自 auto-lang ui 迁入；
// 旧包经 ui/mod.rs 再导出保持 auto_lang::ui::{app_registry,i18n_lookup} 路径）。
pub mod app_registry;
pub mod i18n_lookup;
pub mod lucide_generated;
pub mod store_deps;
pub mod style;
// ---- PLAN-814 T-03 Batch 2：AURA 模型/校验、a2ui、四生成器、route 分析 ----
pub mod a2ui;
pub mod aura;
pub mod route;
pub mod ui_gen;

/// 生成器源指纹（本 crate build.rs 对 src/ui_gen 扫描；Plan 692 W-1
/// 失效语义单源）。SDK database::ui_cache 经注入消费；None=无 build.rs 环境。
pub fn generator_fingerprint() -> Option<String> {
    option_env!("AUTO_UI_GEN_FINGERPRINT").map(|s| s.to_string())
}

// PLAN-814 R3-03b/R4-01：UI 文法函数表装配。显式优先：公开生成入口
// （ui_gen::api 的 transpile_aura/transpile_vue_aura/
// generate_component_from_file）经 [`assemble_ui_grammar`] 首选显式组合；
// 下方 inventory 自注册（SDK 收集点 UiGrammarSource）仅为本 crate 被链接
// 即覆盖不经入口的直达 `Parser::from(..).with_session(UI)` 构造兜底——
// 语义边界与 auto-lang 侧 UiCompatInstaller 同（SDK lib.rs 对账注记）。
inventory::submit!(auto_lang_core::UiGrammarSource(auto_ui_dialect::ui_grammar));

/// 显式装配 UI 文法函数表（PLAN-814 R4-01；幂等，首装优先）。
pub fn assemble_ui_grammar() {
    let _ = auto_lang_core::set_ui_grammar(auto_ui_dialect::ui_grammar());
}
