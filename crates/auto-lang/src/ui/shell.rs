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

/// Plan 478 T4：switcher overlay 源码（进程内嵌；T1 施工图 §1.4——shell
/// pack 同级特权组件，不进注册表/examples，无 launcher 式 entry 路径）。
pub const SWITCHER_AT: &str = include_str!("../../assets/switcher.at");

/// 进程内编译装载 switcher 组件（召唤期懒挂载调用；失败由调用方 toast 降级）。
#[cfg(feature = "ui-iced")]
pub fn build_switcher_component(
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    crate::build_dynamic_component(SWITCHER_AT, None)
}

/// Plan 479 T3：通知中心 overlay 源码（进程内嵌；shell pack 同级特权组件，
/// 不进注册表/examples——switcher 同型第三枚 overlay 槽）。
pub const NOTIFICATION_CENTER_AT: &str = include_str!("../../assets/notification_center.at");

/// 进程内编译装载通知中心组件（notes_toggle 召唤期懒挂载调用；失败由调用方
/// 通知降级）。
#[cfg(feature = "ui-iced")]
pub fn build_notification_center_component(
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    crate::build_dynamic_component(NOTIFICATION_CENTER_AT, None)
}

/// Plan 487 M4：设置面板 overlay 源码（进程内嵌；shell pack 同级特权组件，
/// 不进注册表/examples——switcher/通知中心同型第四枚 overlay 槽）。
pub const SETTINGS_AT: &str = include_str!("../../assets/settings.at");

/// 进程内编译装载设置面板组件（open_settings 召唤期懒挂载调用；失败由调用
/// 方通知降级）。
#[cfg(feature = "ui-iced")]
pub fn build_settings_component(
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    crate::build_dynamic_component(SETTINGS_AT, None)
}

/// Plan 496 M5：桌面本体面源码（进程内嵌；shell pack 同级特权组件，不进
/// 注册表/examples——第五面。与 overlay 槽不同：常驻不召唤，boot 期装载，
/// 挂桌面层 z 槽（壁纸之上、App 虚拟窗口之下））。
pub const DESKTOP_AT: &str = include_str!("../../assets/desktop.at");

/// 进程内编译装载桌面本体面组件（boot 期常驻装载调用；失败由调用方降级为
/// 无图标桌面——不阻断既有桌面）。
#[cfg(feature = "ui-iced")]
pub fn build_desktop_surface_component(
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    crate::build_dynamic_component(DESKTOP_AT, None)
}

/// Plan 503：shell pack 特权 .at 全量编译冒烟。include_str 内嵌源不进
/// cargo check,语法回归此前只能实机 boot 才暴露(降级为无任务栏桌面)。
/// 本测试走 build_dynamic_component 真管线(编译 + Init),守卫 pack 级
/// 纯 .at 改动(shell/desktop/overlay 槽)。
#[cfg(all(test, feature = "ui-iced"))]
mod pack_tests {
    #[test]
    fn shell_packs_compile() {
        for (name, src) in [
            ("shell", crate::ui::shell::SHELL_AT),
            ("switcher", crate::ui::shell::SWITCHER_AT),
            ("notification_center", crate::ui::shell::NOTIFICATION_CENTER_AT),
            ("settings", crate::ui::shell::SETTINGS_AT),
            ("desktop", crate::ui::shell::DESKTOP_AT),
        ] {
            crate::build_dynamic_component(src, None)
                .unwrap_or_else(|e| panic!("{name}.at 编译失败: {e}"));
        }
    }
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
