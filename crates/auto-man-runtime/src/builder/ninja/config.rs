//! PLAN-805：工具链配置模型的单一实现已迁至 `auto-toolchain-core`
//! （`crates/auto-toolchain-core/src/config.rs`，自本文件逐字抽取）。
//!
//! 本模块保留为**薄 re-export 门面**：旧公开路径
//! `auto_man::builder::ninja::config::*` 与类型身份不变（re-export 即同一
//! 类型，可与新 crate 直接互传），CompilerStore/Resolver/NinjaBuilder/
//! Port/Pac 等消费者经此路径消费新实现。行为与旧臂逐字节冻结
//! （oracle：`crates/auto-man/tests/fixtures/plan805/`）。

pub use auto_toolchain_core::config::*;
