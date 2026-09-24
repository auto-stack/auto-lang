//! Plan 462 T5 / 463 T7 验收宿主：虚拟桌面 —— 一个 OS 窗口承载多 App 虚拟
//! 窗口（拖拽/缩放/聚焦/× 关闭）+ 全屏桌面 shell（任务栏/热键/注册表启动）。
//!
//! 运行：`cargo run -p auto-lang --features ui-iced --example ui_desktop`
//! 全屏：`cargo run -p auto-lang --features ui-iced --example ui_desktop -- --fullscreen`
//! 指定注册表：`... -- --fullscreen --apps-dir examples/ui`
//! panic 隔离演示沿用 459：`AUTOUI_PANIC_PROBE=1`（事件名 `__panic_probe`；
//! PLAN-552 起 459-dual-app 源位于 examples/capability-tests/）。

use auto_lang::ui::iced::{run_dynamic_desktop_fullscreen, run_dynamic_desktop_with_options, DesktopOptions};
use std::path::PathBuf;

// PLAN-552：459-dual-app 探针迁 examples/capability-tests/（深度 ../../../ 不变）。
// 2026-09-15 用户裁定：459 双窗口 demo 的 boot 直挂窗退役——验收遗留，普通
// 桌面每次启动都带 demo 窗（任务栏还得靠 app-window 兜底图标）；双窗隔离
// 验收跑专用 example `ui_dual_app`（同一源，进程内双 OS 窗）。
// PLAN-043 执行期补刀（2026-09-24 用户报告"每次启动都开计算器"）：011 的
// boot 直挂同属验收遗留，本入口每次启动把计算器当直挂组件装配（无开关、
// 不走 launch 链，故无 autostart/会话恢复痕迹）——一并退役；boot 组件表
// 恒空，桌面窗口全部经注册表 launch 链。

/// 默认注册表目录：仓库 examples/ui（相对 crate 编译期定位，CWD 无关）。
fn default_apps_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("ui")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // PLAN-042 T-02：桌面轨宿主 log crate trap——ui_desktop 入口本无 logger
    // （`auto` CLI 的 simplelog 只在 CLI 子命令进程，与本进程无涉），装环即
    // 全量收 error/warn/info；已装（未来入口变更）= 降级 `syslog!` 宏面，
    // 不强拆既有链（PLAN-042 §5.2 定案）。
    auto_lang::ui::syslog::install_host_logger();
    // PLAN-016 R7：桌面在场标记——App 可用 Env.get 探测自身是否运行在
    // 虚拟桌面内（如 file-manager 的 open_with / 系统默认程序分流）。
    // 单 app 窗口（auto run -r vm）不设此变量。
    std::env::set_var("AUTO_UI_IN_DESKTOP", "1");
    // PLAN-024 R7：storage 路径对齐 desktop.sh（PLAN-018 rev2 确定性
    // per-user 文件）——直接 exec exe（不经脚本）时缺此 env 会落到按
    // CWD 哈希的临时库：桌面快捷方式/shell 配置全部"消失"（实机走查
    // R7 根因）。已设 env（脚本/测试隔离）不覆盖。
    if std::env::var_os("AUTO_VM_STORAGE_FILE").is_none() {
        if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
            let path = std::path::PathBuf::from(home)
                .join(".config")
                .join("autoos")
                .join("desktop-storage.json");
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            std::env::set_var("AUTO_VM_STORAGE_FILE", &path);
        }
    }
    let args: Vec<String> = std::env::args().collect();
    // Plan 463 T3：--fullscreen = borderless 全屏桌面（PLAN-526 T13 起
    // Esc 调试退出退役——退出走 dock 电源键确认面板）。
    let fullscreen = args.iter().any(|a| a == "--fullscreen");
    // Plan 463 T7：--apps-dir <path>（默认仓库 examples/ui；vm 兼容过滤）。
    let apps_dir = args
        .iter()
        .position(|a| a == "--apps-dir")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(default_apps_dir);

    // PLAN-697 T-01 观测加固（外因勘定收口，镜像 rust_ui.rs X9 足迹桩形制）：
    // P694-D1「宿主静默退出」勘定结论 = 机器级外因（09-23 10:59 内核
    // bugcheck 实锤，见 docs/plans/reports/p697-stability/），但宿主进程
    // 此前零足迹——死亡无法与机器灾难对时。三件：①panic 钩子带 backtrace
    // （此前 ui_desktop 无钩子，内部 panic 直接 abort 零痕迹）；②30s 存活
    // 心跳（墙钟 epoch，与 System 事件日志 6008/1001 直接对时——死亡时刻
    // 贴近 bugcheck 即机器外因，反之查 WER）；③start/exit 留痕行（正常
    // 关窗 = exit 行在场；被杀 = 心跳截断无 exit 行；崩溃 = WER + 心跳缺口）。
    {
        let boot = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        eprintln!(
            "[ui-desktop] start pid={} fullscreen={fullscreen} apps_dir={} t={boot}",
            std::process::id(),
            apps_dir.display()
        );
        std::panic::set_hook(Box::new(|info| {
            eprintln!("[ui-desktop] panic {info}
backtrace:
{}", std::backtrace::Backtrace::force_capture());
        }));
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(30));
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            eprintln!("[ui-desktop] alive pid={} t={now}", std::process::id());
        });
    }

    // PLAN-043 执行期补刀：boot 直挂计算器退役（APP_B 装配块拆除）——
    // boot 组件表恒空，窗口全部经注册表 launch 链。此前的 PLAN-013 容错
    // 与 PLAN-526 T6 源路径对齐都只服务于直挂组件，随直挂退役一并失效。
    let comps = Vec::new();
    // Plan 472 T5：窗口模式同样装配注册表（dock pinned/launch 依赖；
    // 463 只给了全屏路径）。
    let opts = DesktopOptions {
        fullscreen,
        apps_dir: Some(apps_dir),
        // Plan 494：真洞模式（测试宿主透传 env 开关；生产桌面走
        // `shell.native.hole` storage 键）。
        hole_mode: std::env::var("AUTO_DESKTOP_HOLE").as_deref() == Ok("1"),
        ..Default::default()
    };
    let result = if fullscreen {
        run_dynamic_desktop_fullscreen(comps, opts)
    } else {
        run_dynamic_desktop_with_options(comps, opts)
    };
    // PLAN-697 T-01：退出留痕行——正常关窗/Err 返回均留痕（0/1 退出码
    // 即本 Result 的映射）；被杀/崩溃则此行缺席（心跳缺口+WER 判位面）。
    match &result {
        Ok(_) => eprintln!("[ui-desktop] exit ok (code 0)"),
        Err(e) => eprintln!("[ui-desktop] exit err (code 1): {e}"),
    }
    result.map(|_| ())
}
