//! Plan 462 T5 / 463 T7 验收宿主：虚拟桌面 —— 一个 OS 窗口承载多 App 虚拟
//! 窗口（拖拽/缩放/聚焦/× 关闭）+ 全屏桌面 shell（任务栏/热键/注册表启动）。
//!
//! 运行：`cargo run -p auto-lang --features ui-iced --example ui_desktop`
//! 全屏：`cargo run -p auto-lang --features ui-iced --example ui_desktop -- --fullscreen`
//! 指定注册表：`... -- --fullscreen --apps-dir examples/ui`
//! panic 隔离演示沿用 459：`AUTOUI_PANIC_PROBE=1`（事件名 `__panic_probe`）。

use auto_lang::ui::iced::{run_dynamic_desktop_fullscreen, run_dynamic_desktop_with_options, DesktopOptions};
use std::path::PathBuf;

const APP_A: &str = include_str!("../../../examples/ui/459-dual-app/app.at");
const APP_B: &str = include_str!("../../../examples/ui/011-calculator/src/front/app.at");

/// 默认注册表目录：仓库 examples/ui（相对 crate 编译期定位，CWD 无关）。
fn default_apps_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("ui")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    // Plan 463 T3：--fullscreen = borderless 全屏桌面（Esc 调试退出）。
    let fullscreen = args.iter().any(|a| a == "--fullscreen");
    // Plan 463 T7：--apps-dir <path>（默认仓库 examples/ui；vm 兼容过滤）。
    let apps_dir = args
        .iter()
        .position(|a| a == "--apps-dir")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(default_apps_dir);

    let comp_a = auto_lang::build_dynamic_component(APP_A, None)?;
    let comp_b = auto_lang::build_dynamic_component(APP_B, None)?;
    // Plan 472 T5：窗口模式同样装配注册表（dock pinned/launch 依赖；
    // 463 只给了全屏路径）。
    let opts = DesktopOptions {
        fullscreen,
        apps_dir: Some(apps_dir),
        // Plan 494：真洞模式（测试宿主透传 env 开关；生产桌面走
        // `shell.native.hole` storage 键）。
        hole_mode: std::env::var("AUTO_DESKTOP_HOLE").as_deref() == Ok("1"),
    };
    if fullscreen {
        run_dynamic_desktop_fullscreen(vec![comp_a, comp_b], opts)?;
    } else {
        run_dynamic_desktop_with_options(vec![comp_a, comp_b], opts)?;
    }
    Ok(())
}
