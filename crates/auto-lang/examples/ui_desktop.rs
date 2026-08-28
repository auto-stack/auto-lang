//! Plan 462 T5 验收 demo：虚拟桌面 —— 一个 OS 窗口承载两个不同 App 的
//! 虚拟窗口（拖拽标题栏移动、边缘/角落缩放、点击聚焦、× 关闭）。
//!
//! 运行：`cargo run -p auto-lang --features ui-iced --example ui_desktop`
//! 全屏：`cargo run -p auto-lang --features ui-iced --example ui_desktop -- --fullscreen`
//! panic 隔离演示沿用 459：`AUTOUI_PANIC_PROBE=1`（事件名 `__panic_probe`）。

use auto_lang::ui::iced::{run_dynamic_desktop, run_dynamic_desktop_fullscreen};

const APP_A: &str = include_str!("../../../examples/ui/459-dual-app/app.at");
const APP_B: &str = include_str!("../../../examples/ui/011-calculator/src/front/app.at");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Plan 463 T3：--fullscreen = borderless 全屏桌面（Esc 调试退出）。
    let fullscreen = std::env::args().any(|a| a == "--fullscreen");
    let comp_a = auto_lang::build_dynamic_component(APP_A, None)?;
    let comp_b = auto_lang::build_dynamic_component(APP_B, None)?;
    if fullscreen {
        run_dynamic_desktop_fullscreen(vec![comp_a, comp_b])?;
    } else {
        run_dynamic_desktop(vec![comp_a, comp_b])?;
    }
    Ok(())
}
