//! shim-metadata 库面(plan-430)。
//!
//! bin(CLI)负责离线人工使用;lib 面（rustdoc 解析 / 分类器 / std 与 cdylib 两路
//! 代码生成 / shim 包清单结构）供 dep 管线（auto-cache → auto-lang）进程内调用，
//! 实现"导入三方 crate 时自动提取元信息并编译 shim 包"(Phase C2)。

pub mod classify;
pub mod emit;
pub mod emit_cdylib;
pub mod rustdoc;
pub mod std_catalog;
pub mod types;
