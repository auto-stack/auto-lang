//! PLAN-805：标志映射器的单一实现已迁至 `auto-toolchain-core`
//! （`crates/auto-toolchain-core/src/mapper.rs`，自本文件逐字抽取）。
//!
//! 本模块保留为**薄 re-export 门面**：旧公开路径
//! `auto_man::builder::ninja::mapper::FlagMapper` 不变，NinjaBuilder 生产
//! 路径经此消费新实现。

pub use auto_toolchain_core::mapper::*;
