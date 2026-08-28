//! Plan 463 T5：桌面 shell 装配（特权 .at App，R1/R8「shell 组件是
//! AutoUI App」首落）。
//!
//! `assets/shell.at` 进程内编译装载（`build_dynamic_component` = `auto run`
//! 同管线）；独立模式不装载（I3 配置位分叉，`auto run` 管线不变）。
//! 双向接缝形状见 shell.at 头注与 T1 报告 `docs/plans/reports/
//! 463-t1-bus-blueprint.md` §2/§3。

/// 桌面 shell 源码（编译期内嵌；T5 定案形态）。
pub const SHELL_AT: &str = include_str!("../../assets/shell.at");

/// 进程内编译装载 shell 组件（boot 期调用；失败由调用方降级为无任务栏桌面）。
#[cfg(feature = "ui-iced")]
pub fn build_shell_component(
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    crate::build_dynamic_component(SHELL_AT, None)
}
