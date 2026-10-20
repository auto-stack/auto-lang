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
pub mod lucide_generated;
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

// PLAN-814 R3-03b：UI 文法函数表装配自注册（SDK 收集点 UiGrammarSource
// ——本 crate 被链接即生效；auto-ui-dialect 依赖为本 crate 直接清单边）。
// SDK 清单不再引用 auto-ui-dialect 后，UI 解析（`Parser::from(..)
// .with_session(UI)` 等路径）经该表派发文法单源实现。编译器消费方
// 闭包因此无需经旧 auto-lang 兼容包间接拉入文法。
inventory::submit!(auto_lang_core::UiGrammarSource(auto_ui_dialect::ui_grammar));
