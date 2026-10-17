//! 工具链配置与构建规则核心（PLAN-805 自 auto-man `builder/ninja` 抽出）。
//!
//! 单一实现归属：
//! - `config`：`CompilerKind`/`ExecutableType`/`FlagFormat`/`FlagMappings`/
//!   `CompilerLocation`/`CompilerConfig` 与八类默认配置（MSVC/GCC/Clang/
//!   GHS/IAR/Targeting/Hightec/TICLang）；
//! - `mapper`：`FlagMapper` 通用标志 → 编译器特定格式映射；
//! - `templates`：`CommandTemplates` 四命令（编译/汇编/链接/归档）内联
//!   字符串模板与占位符渲染。
//!
//! 边界（PLAN-805 合同）：动态 `am.at` 加载（CompilerStore）、PATH/磁盘/
//! 进程观测（CompilerResolver）、Ninja 执行与 AutoGen 编排（NinjaBuilder）
//! 留在 auto-man 上层；本 crate 依赖仅 auto-val + std，行为与旧臂逐字节
//! 冻结（oracle：`crates/auto-man/tests/fixtures/plan805/`）。

pub mod config;
pub mod mapper;
pub mod templates;

// 重新导出常用类型（与旧 builder::ninja 出口面一致）
pub use config::{
    CompilerConfig, CompilerKind, CompilerLocation, ExecutableType, FlagFormat, FlagMappings,
};
pub use mapper::FlagMapper;
pub use templates::CommandTemplates;
