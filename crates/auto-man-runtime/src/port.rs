//! Port 数据结构——PLAN-809 T-03 起单一实现迁至 `auto-man-core/src/port.rs`
//! （含 `Port::default()` 的编译期平台常量与 `port_from_node` 纯解析）；
//! 本模块保留为**薄 re-export 门面**：旧公开路径 `auto_man::port::*`
//! 与类型身份不变（`Port.compiler` 即 auto-toolchain-core 的 CompilerConfig）。

pub use auto_man_core::port::*;
