//! auto-ui-compiler — UI 编译/生成域（PLAN-814）。
//!
//! 从 auto-lang 迁入的 UI 分析（aura 提取/schema）、设计 token 与
//! vue/jet/ark/rust 四生成器及 widget registry（T-03 迁移面）。
//!
//! 边界合同：依赖纯语言 SDK（auto-lang-core），不被 SDK 反向依赖；
//! 生成的 runtime import 路径策略单源在本 crate；跨层集成测试归上层。

// ---- PLAN-814 T-03 Batch 1：style DSL 分析/模型、action 配置解析、lucide 表 ----
// （iced 后端适配器留 auto-lang ui/style_iced_adapter.rs——runtime 件。）
pub mod style;
pub mod action_config;
pub mod lucide_generated;
