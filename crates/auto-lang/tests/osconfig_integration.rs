//! Plan 501 T3：os-config 集成档——真 daemon 起停 + 全链用例
//!（注册表条目 → launch → App Init 经 daemon 拉数据 → 模块列表非空 →
//! PUT 改写 → 配置根落盘断言）。
//!
//! 门控（按 daemon 可用性，计划 §测试设计 3）：真机材料三件——相邻仓
//! `../auto-os-config` 前端根、`auto-os-config-back/target/release/`
//! daemon 二进制——任一缺席即跳过（eprintln 提示，不算失败；开发机
//! `cargo build --release` 于 auto-os-config-back/ 后自动纳入）。
//! daemon 配置根经 spawn 期 `USERPROFILE`/`HOME` 重定向到临时目录
//!（daemon config_root() 读 env 解析 `~/.config/autoos`——测试零污染
//! 真实家目录；待澄清③的跨仓 config root env 因此非必需）。
//!
//! Run with:
//!   cargo nextest run -p auto-lang --features ui-iced --test osconfig_integration

#![cfg(feature = "ui-iced")]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

/// 相邻仓根候选（多形态：主检出 `crates/auto-lang` 上 2 级即 repo 根 →
/// `../auto-os-config`；worktree 检出深 `.worktrees/plan-501-dev/` → 需上
/// 5 级到 autostack。首个带材料的候选胜——生产 host_extra_roots 的
/// CWD 相对探测不受影响，测试自带定位）。
fn sibling_repo_candidates() -> Vec<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    [2usize, 5, 3]
        .into_iter()
        .map(|up| {
            let mut p = manifest.to_path_buf();
            for _ in 0..up {
                p.pop();
            }
            p.join("auto-os-config")
        })
        .collect()
}

/// 首个材料齐全（前端 + daemon 二进制）的相邻仓候选；None = 跳过档。
fn sibling_with_materials() -> Option<PathBuf> {
    sibling_repo_candidates()
        .into_iter()
        .find(|s| {
            s.join("auto")
                .join("src")
                .join("front")
                .join("app.at")
                .is_file()
                && daemon_exe(s).is_file()
        })
}

/// daemon 二进制（相邻仓 Cargo [[bin]]，release 产物）。
fn daemon_exe(sibling: &Path) -> PathBuf {
    sibling
        .join("auto-os-config-back")
        .join("target")
        .join("release")
        .join(if cfg!(windows) {
            "auto-os-config-back-server.exe"
        } else {
            "auto-os-config-back-server"
        })
}

/// 裸 HTTP 工具（std 单依赖；与 osconfig_daemon::tcp_ping 同型，测试自用
/// 完整 GET/PUT）。
fn http_request(
    url: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<(u16, String), String> {
    use std::net::ToSocketAddrs;
    let base = url.strip_prefix("http://").ok_or("仅 http")?;
    let addr = base
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("无 addr")?;
    let mut stream = TcpStream::connect(addr).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    let body_bytes = body.map(|b| b.as_bytes().to_vec());
    let len = body_bytes.as_ref().map(|b| b.len()).unwrap_or(0);
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {base}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {len}\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    if let Some(b) = &body_bytes {
        stream.write_all(b).map_err(|e| e.to_string())?;
    }
    let mut out = Vec::new();
    stream.read_to_end(&mut out).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&out).to_string();
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or("响应行解析失败")?;
    let body = text
        .split("\r\n\r\n")
        .nth(1)
        .ok_or("无响应体")?
        .to_string();
    Ok((status, body))
}

/// 起测试 daemon：随机端口 + 配置根重定向（USERPROFILE/HOME → temp home）。
/// 返回 (daemon url, Child, temp home)。就绪 ping ≤10s。
fn spawn_test_daemon(exe: &Path) -> Result<(String, std::process::Child, PathBuf), String> {
    for _ in 0..3 {
        let port = {
            let l = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
            l.local_addr().map_err(|e| e.to_string())?.port()
        };
        let home = std::env::temp_dir().join(format!(
            "autoui-501-t3-home-{}-{port}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).map_err(|e| e.to_string())?;
        let mut child = std::process::Command::new(exe)
            .env("AUTOOS_BACK_PORT", port.to_string())
            .env("USERPROFILE", &home)
            .env("HOME", &home)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .stdin(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn daemon 失败: {e}"))?;
        let url = format!("http://127.0.0.1:{port}");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while std::time::Instant::now() < deadline {
            if auto_lang::ui::osconfig_daemon::tcp_ping(&url, std::time::Duration::from_millis(500))
            {
                return Ok((url, child, home));
            }
            if let Ok(Some(_)) = child.try_wait() {
                break; // 端口竞态崩了——换端口重试
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        let _ = child.kill();
        let _ = std::fs::remove_dir_all(&home);
    }
    Err("三次随机端口均未就绪".to_string())
}

/// T3 全链（单测串行：材料检查 → daemon 起 → 注册表 → launch → 断言 → 落盘）。
#[test]
fn osconfig_full_chain_launch_modules_and_persist() {
    // 看门狗：全链 90s 未完成即自杀（headless VM/编译卡死可观测，防无限自旋）。
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(90));
        eprintln!("[501-T3] WATCHDOG fired — full chain hung");
        std::process::exit(0xF00D);
    });
    let Some(sibling) = sibling_with_materials() else {
        eprintln!(
            "[501-T3] skip: 相邻仓 auto-os-config 材料不全（前端 auto/src/front/app.at + \
             auto-os-config-back/target/release daemon 二进制）——cd auto-os-config-back && \
             cargo build --release 后纳入"
        );
        return;
    };
    let front = sibling.join("auto");
    eprintln!("[501-T3] phase A: spawning daemon");

    // ── ① daemon 起停：配置根重定向 + 随机端口 ──────────────────────────
    let (url, mut child, home) = spawn_test_daemon(&daemon_exe(&sibling)).expect("测试 daemon 就绪");
    let cfg_root = home.join(".config").join("autoos");
    eprintln!("[501-T3] phase B: daemon ready at {url}");

    // ── ② 注册表：真相邻仓前端自含根 → os-config 条目 ──────────────────
    let entry = auto_lang::ui::app_registry::scan_app_root(
        &front,
        "os-config",
        &auto_lang::ui::app_registry::ScanOptions::default(),
    )
    .expect("相邻仓前端可扫出 os-config 条目");
    assert!(entry.entry.is_file(), "入口存在: {}", entry.entry.display());
    // pac `back: { project }` 链接式契约解析（残缺本地副本 → 后端项目根）。
    let back_root = entry
        .back_root
        .clone()
        .expect("pac back 声明存在");
    assert!(
        back_root.join("api.at").is_file(),
        "后端契约 api.at 在位: {}",
        back_root.display()
    );

    eprintln!("[501-T3] phase C: launching app");
    // ── ③ launch：探活注入 Running(测试 url) → env 注入 → 真前端装载 ──
    //（daemon 声明取 pac.at 自然字段——T7 跨仓已落 `daemon: "autoos"` 行；
    //  检活注入位指向测试 daemon url，ensure_ready 生产路径 T4 实机验证。）
    std::env::remove_var(auto_lang::ui::osconfig_daemon::ENV_DAEMON);
    assert_eq!(
        entry.daemon.as_deref(),
        Some("autoos"),
        "pac daemon 声明（T7 跨仓行）被注册表自然消费"
    );
    let code = std::fs::read_to_string(&entry.entry).expect("读入口源");
    let source_path = entry.entry.to_string_lossy().to_string();
    let daemon_decl = entry.daemon.clone();
    let mut ds = auto_lang::ui::session::DesktopSession::__test_session();
    ds.open_desktop(iced::window::Id::unique());
    let win = ds.host.as_ref().expect("desktop").window;
    let primary = auto_lang::build_dynamic_component(
        "widget HostProbe {\n    model { var n int = 0 }\n    view { text \"${.n}\" }\n}\n",
        None,
    )
    .expect("主窗 probe");
    let primary = ds.allocate_app(primary);
    ds.register_window(win, primary, iced::Size::new(1280.0, 800.0));
    ds.desktop.app_resolver = Some(std::sync::Arc::new(move |name: &str| {
        (name == "os-config").then(|| auto_lang::ui::session::LaunchSpec {
            code: code.clone(),
            source_path: Some(source_path.clone()),
            title: Some("系统设置".to_string()),
            daemon: daemon_decl.clone(),
            back_root: Some(back_root.clone()),
        })
    }));
    let probe_url = url.clone();
    ds.desktop.osconfig_daemon_probe = Some(std::sync::Arc::new(move || {
        auto_lang::ui::osconfig_daemon::DaemonStatus::Running(probe_url.clone())
    }));
    ds.launch_app("os-config")
        .unwrap_or_else(|e| panic!("launch os-config 失败: {e}"));
    assert_eq!(
        std::env::var(auto_lang::ui::osconfig_daemon::ENV_DAEMON).as_deref(),
        Ok(url.as_str()),
        "AUTOOS_DAEMON 注入测试 daemon url"
    );

    eprintln!("[501-T3] phase D: launched; asserting app state");
    // ── ④ App Init 经 daemon 拉真数据（system_info → sys_host 非空）────
    let launched = ds
        .host
        .as_ref()
        .unwrap()
        .wm
        .wins
        .values()
        .find(|w| w.registry_id.as_deref() == Some("os-config"))
        .map(|w| w.app)
        .expect("os-config 虚拟窗");
    let app = ds.apps.get(&launched).expect("App 会话");
    let sys_host = match app.component.read_state("sys_host") {
        Ok(auto_val::Value::Str(ref s)) => s.to_string(),
        other => panic!("sys_host 读回异常: {other:?}"),
    };
    assert!(
        !sys_host.is_empty(),
        "App Init 经 daemon system_info 填充 sys_host（得空 = env 注入链断）"
    );

    eprintln!("[501-T3] phase E: modules via daemon API");
    // ── ⑤ 模块列表非空（经 daemon API；内置注册表 ≥7）──────────────────
    let (status, body) = http_request(&url, "GET", "/api/modules", None).expect("GET modules");
    assert_eq!(status, 200, "modules 状态码");
    let modules: serde_json::Value = serde_json::from_str(&body).expect("modules JSON");
    let arr = modules.as_array().expect("modules 数组");
    assert!(arr.len() >= 7, "内置注册表 ≥7 模块，实际 {}", arr.len());
    assert!(
        arr.iter().any(|m| m.get("id").and_then(|v| v.as_str()) == Some("ai-daemon")),
        "内置 ai-daemon 模块在列"
    );

    eprintln!("[501-T3] phase F: PUT + persist");
    // ── ⑥ PUT 改写 → 配置根落盘（重定向家目录下的 ai-daemon.at）────────
    // daemon 对缺席配置文件 404（不自动建桩）——预置最小 atom 文件后改写。
    std::fs::create_dir_all(&cfg_root).expect("建配置根");
    std::fs::write(
        cfg_root.join("ai-daemon.at"),
        "daemon {\n    tier : \"baseline\"\n}\n",
    )
    .expect("预置基线配置");
    let put_body = format!(
        "{{\"value\": {{\"probe_501\": \"t3-written\"}}}}"
    );
    let (status, resp) = http_request(
        &url,
        "PUT",
        "/api/config/ai-daemon",
        Some(&put_body),
    )
    .expect("PUT config");
    assert_eq!(status, 200, "PUT 状态码: {resp}");
    let put: serde_json::Value = serde_json::from_str(&resp).expect("PUT JSON");
    assert_eq!(put.get("ok").and_then(|v| v.as_bool()), Some(true), "PUT ok: {resp}");
    let file_on_disk = cfg_root.join("ai-daemon.at");
    assert!(
        file_on_disk.is_file(),
        "配置落盘于重定向根: {}（resp file = {:?}）",
        file_on_disk.display(),
        put.get("file").and_then(|v| v.as_str())
    );
    let written = std::fs::read_to_string(&file_on_disk).expect("读落盘文件");
    assert!(
        written.contains("t3-written"),
        "改写内容在盘（实际: {written}）"
    );

    // ── 清理：测试自有 daemon 即起即杀（生产 detached 语义不受影响）──
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&home);
    std::env::remove_var(auto_lang::ui::osconfig_daemon::ENV_DAEMON);
}
