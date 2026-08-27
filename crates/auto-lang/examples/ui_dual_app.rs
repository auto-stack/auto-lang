//! Plan 459 T4：双 AppSession 双 OS 窗口 demo。
//!
//! 一个进程、一个 `DesktopSession`、两个 OS 窗口，各自渲染同一 `.at`
//! 的独立实例（方案① 同源双实例，计划 §2.4）。验证：
//! 1. `iced::daemon` 多窗口：每窗口 view 按窗口归属的 AppSession 渲染；
//! 2. 隔离性：计数/输入/print 日志互不串扰（F12 Console 按 App 打标）；
//! 3. panic 隔离（T5）：`AUTOUI_PANIC_PROBE=1` 后点 Crash 按钮，仅该
//!    窗口落崩溃页，另一窗口持续可交互。
//!
//! 运行：
//!   cargo run -p auto-lang --features ui-iced --example ui_dual_app
//! panic 隔离验证：
//!   AUTOUI_PANIC_PROBE=1 cargo run -p auto-lang --features ui-iced --example ui_dual_app
//!   （在一个窗口点 "Crash"，切另一窗口继续点 "+"——应照常工作）

use auto_lang::ui::iced::run_dynamic_iced_multi;

const APP_SOURCE: &str = include_str!("../../../examples/ui/459-dual-app/app.at");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 方案①：同一 .at 双实例 —— 两个独立 DynamicComponent（各自 VM/store），
    // 证明会话层状态隔离；组件注册表/路由互不污染属方案②（可选加深，454）。
    let comp_a = auto_lang::build_dynamic_component(APP_SOURCE, None)?;
    let comp_b = auto_lang::build_dynamic_component(APP_SOURCE, None)?;

    // 阻塞至全部窗口关闭（全窗口关闭 → iced::exit）。
    run_dynamic_iced_multi(vec![comp_a, comp_b])?;
    Ok(())
}
