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

/// Plan 463 T7：启动失败占位页（Design 24 §6.5）—— LaunchApp 构建失败时
/// 的可见反馈窗：不白屏、不阻断桌面（toast 并行报错；关闭占位窗即走）。
pub const LAUNCH_FALLBACK_AT: &str = r#"widget LaunchFallback {
    msg {}
    model {
        var app str = ""
    }
    view {
        center {
            col {
                style: "gap-3 p-8 items-center"
                icon (name: "app-window", style: "h-10 w-10 text-muted-foreground") {}
                text "应用暂不可用" { style: "text-lg font-semibold" }
                text "无法启动：${.app}" { style: "text-sm text-muted-foreground" }
            }
        }
    }
}"#;

/// 进程内编译装载占位页组件（`app` 状态 = 目标 App 名，视图绑定展示）。
#[cfg(feature = "ui-iced")]
pub fn build_launch_fallback(
    app_name: &str,
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    let mut comp = crate::build_dynamic_component(LAUNCH_FALLBACK_AT, None)?;
    let _ = comp.write_state("app", auto_val::Value::str(app_name));
    Ok(comp)
}
