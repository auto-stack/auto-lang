//! Plan 486 T5：实机冒烟驱动（`#[ignore]` 手动档——不进日常测试）。
//!
//! 用途：真人手势的自动化近似——经 `drag_sim`（SendInput 真实输入管线）
//! 把指定标题的真实窗口拖到指定屏幕点。配合全屏 `ui_desktop` 实例执行
//! 473 债务清偿冒烟（B1 拖入 / B5 模态 / B8 退出恢复 / D1 常驻）。
//!
//! 运行（先起桌面：`cargo run -p auto-lang --features ui-iced --example
//! ui_desktop -- --fullscreen`）：
//! ```text
//! T5_DRAG_TITLE="主页" T5_TARGET="1920,1100" \
//!   cargo test -p auto-lang --features test-native-dock --test t5_smoke -- --ignored --nocapture
//! ```

#![cfg(all(windows, feature = "test-native-dock"))]

use std::time::Duration;

use auto_lang::ui::native_dock::{win32 as ndw, NativeHwnd};

/// 按标题前缀找可见顶层窗口。
fn find_by_title(prefix: &str) -> Option<NativeHwnd> {
    ndw::list_top_level_windows()
        .into_iter()
        .find(|(h, _, title)| {
            let _ = h;
            title.starts_with(prefix)
        })
        .map(|(h, _, _)| h)
}

#[test]
#[ignore = "实机手动冒烟驱动（需全屏 ui_desktop 在跑；Plan 486 T5）"]
fn manual_drag_window_to_point() {
    // 单击模式：T5_CLICK="x,y"（物理坐标）——不拖拽，纯点击（任务栏
    // native 条目/× 等桌面控件核对）。
    if let Ok(point) = std::env::var("T5_CLICK") {
        let (x, y) = point
            .split_once(',')
            .and_then(|(x, y)| x.trim().parse::<i32>().ok().zip(y.trim().parse::<i32>().ok()))
            .expect("T5_CLICK 形如 1920,2430");
        let _ = ndw::set_process_dpi_aware_per_monitor_v2();
        println!("[t5] click ({x},{y}) = {}", ndw::drag_sim::click_at(x, y));
        return;
    }
    // 坐标域对齐：per-monitor v2（否则 GetWindowRect/目标点全在逻辑域，
    // 与物理 SendInput 绝对坐标错位——实测教训）。
    let _ = ndw::set_process_dpi_aware_per_monitor_v2();
    let title = std::env::var("T5_DRAG_TITLE").unwrap_or_else(|_| "主页".into());
    let target = std::env::var("T5_TARGET").unwrap_or_else(|_| "1920,1100".into());
    let (tx, ty) = target
        .split_once(',')
        .and_then(|(x, y)| x.trim().parse::<i32>().ok().zip(y.trim().parse::<i32>().ok()))
        .expect("T5_TARGET 形如 1920,1100");
    let hwnd = find_by_title(&title).unwrap_or_else(|| {
        for (h, pid, t) in ndw::list_top_level_windows() {
            if !t.is_empty() {
                println!("[t5] 候选 0x{:x} pid={pid} title={t:?}", h.0);
            }
        }
        panic!("未找到标题前缀 {title:?} 的窗口")
    });
    // 最大化/最小化窗先 restore（对齐 dock 执行臂 C5 语义；Win11 最大化
    // 态的 caption 拖拽行为依赖系统版本，显式还原最稳；最小化窗无 caption
    // 可抓）。
    // 归一到常规窗态：restore 重试（跨进程 ShowWindow 异步落地；Explorer
    // 这类忙线程可能吞首次请求）+ 几何强写兜底（set_bounds 脱离最大化形）。
    for attempt in 0..5 {
        if ndw::is_minimized(hwnd) || ndw::is_maximized(hwnd) {
            let _ = ndw::show_window(hwnd, ndw::ShowMode::Restore);
            std::thread::sleep(Duration::from_millis(250));
            if ndw::is_maximized(hwnd) && attempt == 3 {
                let _ = ndw::set_bounds(hwnd, auto_lang::ui::native_dock::Rect::new(
                    150, 120, 2400, 1500,
                ));
                std::thread::sleep(Duration::from_millis(250));
            }
            continue;
        }
        break;
    }
    // 窄窗放大（<1000 物理宽）：高 DPI 下小窗 caption 按钮占宽过半，
    // 任何抓点都会命中按钮（4K@200% 教训）。
    if let Some(b) = ndw::get_bounds_window(hwnd) {
        if b.w < 1000 || b.h < 700 {
            let _ = ndw::set_bounds(hwnd, auto_lang::ui::native_dock::Rect::new(
                b.x.max(60),
                b.y.max(60),
                1400,
                900,
            ));
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    println!(
        "[t5] 归一后窗态 iconic={} zoomed={} bounds={:?}",
        ndw::is_minimized(hwnd),
        ndw::is_maximized(hwnd),
        ndw::get_bounds_window(hwnd)
    );
    println!("[t5] 拖 {title:?} ({hwnd:?}) → ({tx},{ty})");
    let ok = ndw::drag_sim::caption_drag_to(hwnd, (tx, ty), 24);
    println!("[t5] SendInput caption 拖拽链 = {ok}");
    std::thread::sleep(Duration::from_millis(300));
    let bounds = ndw::get_bounds_window(hwnd);
    println!("[t5] 拖后窗口 bounds = {bounds:?}");
    // 交由人工/截图核对：落点高亮 → 松手收编 → 任务栏条目（B1 验收链）。
}
