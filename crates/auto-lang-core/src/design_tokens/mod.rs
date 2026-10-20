// design_tokens —— 语义 token 值单一事实源的家（PLAN-593，Design 29 Phase 1）。
//
// 为什么在这里而不在 `ui::style::theme` 下：registry 的消费方横跨三层——
// `ui_gen`（无 feature 门）与 `ui::style::theme`（feature="ui"）、
// `ui::code_editor`（feature="code-editor"→蕴含 ui）。放无门的基础层，
// 三方都能引用且不引入 feature 依赖边；`ui::style::theme` re-export
// `registry` 保持 `theme::registry` 路径稳定。

pub mod registry;

/// PLAN-601 T-03：theme{} 声明解析与 extends 合成（ComposedTheme）。
pub mod decl;

/// PLAN-607: style recipe 声明注册、语义校验、脱糖展开与 lint（Design 29 Phase 3）。
pub mod recipe;
pub use recipe::*;
