//! PLAN-805：四命令（编译/汇编/链接/归档）内联字符串模板与占位符渲染的
//! 单一实现已迁至 `auto-toolchain-core`
//! （`crates/auto-toolchain-core/src/templates.rs`，自本文件逐字抽取——
//! 内联命令字符串，与 PLAN-803 迁出的 assets/templates 资源无关）。
//!
//! 本模块保留为**薄 re-export 门面**：旧公开路径
//! `auto_man::builder::ninja::templates::CommandTemplates` 不变。

pub use auto_toolchain_core::templates::*;
