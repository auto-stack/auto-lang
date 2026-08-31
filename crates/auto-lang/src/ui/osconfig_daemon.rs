//! Plan 501：os-config daemon 生命周期管理（Design 25 S7——系统 settings =
//! auto-os-config 的 UI 面）。桌面侧职责三件：检活（ping `:17701`）、按发现
//! 序 spawn、`AUTOOS_DAEMON` env 注入 App 会话。
//!
//! 分层：本文件纯逻辑（发现序/决策/env 构造，全注入式可单测）+ 进程管理
//! （spawn + ping 就绪轮询，T2）。daemon 本体在 `../auto-os-config` 仓
//! （axum，二进制 `auto-os-config-back-server`；端口经 `AUTOOS_BACK_PORT`
//! 覆盖，缺省 17901——生产约定 17701 由 [`DAEMON_PORT`] 兑现，front api.at
//! 的缺省 base 同为 17701）。桌面退出**不**杀 daemon（共享服务语义：vite/
//! 其他消费方可能复用；待澄清② v1 裁定）。
//!
//! 现场核验修正（2026-08-31）：计划原文写 `../auto-os-config/target/release/
//! auto-os-config-daemon(.exe)`，实际仓库布局为 `auto-os-config-back/target/
//! release/auto-os-config-back-server(.exe)`（Cargo.toml [[bin]] 名）——发现序
//! 按实际路径兑现，序本身不变（storage 键 > 相邻仓 target > PATH）。

use std::path::{Path, PathBuf};

/// 生产约定端口（front `back.api` 的缺省 base 同源；README :17701）。
pub const DAEMON_PORT: u16 = 17701;

/// 注入 App 会话的 env 键（os-config 仓 vm track 既有约定，
/// `auto/src/back/api.at` 的 `Env.get("AUTOOS_DAEMON")` 消费）。
pub const ENV_DAEMON: &str = "AUTOOS_DAEMON";

/// spawn 时覆盖 daemon 缺省端口（17901）用的 env 键（daemon main.rs 既有）。
pub const ENV_BACK_PORT: &str = "AUTOOS_BACK_PORT";

/// daemon 二进制名（相邻仓 `auto-os-config-back` 的 [[bin]]）。
pub const DAEMON_BIN_NAME: &str = "auto-os-config-back-server";

/// daemon 生命周期状态（面板徽标/launch 门控消费）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonStatus {
    /// 已就绪（复用既有或本会话拉起），携带 base url。
    Running(String),
    /// 拉起中（spawn 已发、就绪 ping 未过）。
    Spawning,
    /// 不可用（路径解析失败/spawn 失败/就绪超时），携带原因。
    Offline(String),
}

/// 缺省 daemon base url（`http://127.0.0.1:<DAEMON_PORT>`）。
pub fn default_daemon_url() -> String {
    format!("http://127.0.0.1:{DAEMON_PORT}")
}

/// 从 base url 提取端口（spawn 期 `AUTOOS_BACK_PORT` 用）；
/// 非法 url 回退 [`DAEMON_PORT`]。
pub fn port_of(url: &str) -> u16 {
    url.rsplit(':').next().and_then(|p| p.trim_matches('/').parse().ok()).unwrap_or(DAEMON_PORT)
}

/// 发现序解析 daemon 可执行文件（G4）：
/// 1. `override_path`（storage 键 `shell.osconfig.daemon`）——用户显式配置，
///    原样采用（spawn 失败时原因携带该路径，不做存在性预判）；
/// 2. 相邻仓 target：`<sibling_root>/auto-os-config-back/target/release/
///    auto-os-config-back-server(.exe)`（现场核验的实际布局，存在才采用）；
/// 3. PATH 查找（`lookup_path` 注入，宿主侧接 `which` 语义）。
///
/// 全部未命中 → None（Offline 的原因之一）。
pub fn resolve_daemon_path(
    override_path: Option<&str>,
    sibling_root: &Path,
    lookup_path: impl Fn(&str) -> Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(explicit) = override_path.map(str::trim).filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(explicit));
    }
    let bin = if cfg!(windows) {
        format!("{DAEMON_BIN_NAME}.exe")
    } else {
        DAEMON_BIN_NAME.to_string()
    };
    let sibling = sibling_root
        .join("auto-os-config-back")
        .join("target")
        .join("release")
        .join(&bin);
    if sibling.is_file() {
        return Some(sibling);
    }
    lookup_path(&bin)
}

/// 检活结果 → 是否需要 spawn（G1 决策纯函数：ping 通即复用，零打扰）。
pub fn should_spawn(ping_ok: bool) -> bool {
    !ping_ok
}

/// App 会话 env 注入表（`AUTOOS_DAEMON=<url>`；os-config api.at 的
/// `daemon_base()` 消费完整 url——含 scheme）。
pub fn env_for(url: &str) -> Vec<(String, String)> {
    vec![(ENV_DAEMON.to_string(), url.to_string())]
}

// ─────────────────────────────────────────────────────────────────────────
// T2：进程管理 —— ping（std 裸 HTTP，零三方依赖；reqwest::blocking 在
// tokio 上下文线程会 panic，桌面宿主持全局 runtime，故不用）+ spawn +
// 就绪轮询状态机。执行环境经 [`DaemonIo`] 注入——单测假实现覆盖状态机
// 全分支，真实路径由 T3/T4 集成消费。
// ─────────────────────────────────────────────────────────────────────────

/// 检活/spawn 执行环境（生产 [`RealDaemonIo`]；单测假实现）。
pub trait DaemonIo {
    /// 一次检活（GET `<url>/api/health`，通 = true）。
    fn ping(&mut self, url: &str) -> bool;
    /// 拉起 daemon（detached；env 为 AUTOOS_BACK_PORT 端口覆盖）。
    fn spawn(&mut self, path: &Path, env: &[(String, String)]) -> Result<(), String>;
}

/// url（`http://127.0.0.1:17701`）→ `127.0.0.1:17701` SocketAddr；
/// 非 loopback http 形态返回 None。
pub fn socket_addr_of(url: &str) -> Option<std::net::SocketAddr> {
    let host_port = url
        .strip_prefix("http://")?
        .split('/')
        .next()?;
    use std::net::ToSocketAddrs;
    host_port.to_socket_addrs().ok()?.next()
}

/// 裸 TCP 检活：connect → `GET /api/health` → 响应行 2xx 即通。
/// std 单依赖、无 tokio TLS 上下文风险；loopback 足够（daemon 恒本地）。
pub fn tcp_ping(url: &str, timeout: std::time::Duration) -> bool {
    use std::io::{Read, Write};
    let Some(addr) = socket_addr_of(url) else { return false };
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let req = format!("GET /api/health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = [0u8; 128];
    let Ok(n) = stream.read(&mut buf) else { return false };
    let head = String::from_utf8_lossy(&buf[..n]);
    head.starts_with("HTTP/1.1 2") || head.starts_with("HTTP/1.0 2")
}

/// 桌面本会话是否拉起过 daemon（复用判定/UX 文案；detached 语义下不持
/// Child——桌面退出不杀 daemon，句柄即刻放掉，只留"曾拉起"记录）。
static DID_SPAWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 本会话是否曾 spawn daemon（T4 复用断言用）。
pub fn did_spawn() -> bool {
    DID_SPAWN.load(std::sync::atomic::Ordering::Relaxed)
}

/// 真实执行环境：std TCP ping + detached spawn。
pub struct RealDaemonIo;

impl DaemonIo for RealDaemonIo {
    fn ping(&mut self, url: &str) -> bool {
        tcp_ping(url, std::time::Duration::from_secs(2))
    }

    fn spawn(&mut self, path: &Path, env: &[(String, String)]) -> Result<(), String> {
        let mut cmd = std::process::Command::new(path);
        for (k, v) in env {
            cmd.env(k, v);
        }
        // detached：与桌面生命周期解耦（共享服务语义，待澄清② v1 裁定）。
        // stdio 全 null——无控制台形态下防句柄悬挂/闪窗。
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        cmd.stdin(std::process::Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            // DETACHED_PROCESS：脱离桌面控制台；CREATE_NEW_PROCESS_GROUP：
            // 独立进程组，桌面组的 Ctrl+C 不传播。
            cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        }
        cmd.spawn()
            .map(|child| {
                DID_SPAWN.store(true, std::sync::atomic::Ordering::Relaxed);
                // detached 语义：立即放句柄（不 kill、不 wait——桌面退出
                // daemon 存活；退出码回收交 OS）。
                std::mem::drop(child);
            })
            .map_err(|e| e.to_string())
    }
}

impl RealDaemonIo {
    /// no-op 占位（struct 无字段；保留具名构造便于读点）。
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealDaemonIo {
    fn default() -> Self {
        Self::new()
    }
}

/// 相邻仓探测缺省根（`../auto-os-config`，CWD 相对——vm 桌面开发机形态
/// 即从本仓根起 `auto run`；storage `shell.apps.scan_siblings` 可关整段
/// 探测，见 app_registry 聚合）。
pub fn default_sibling_root() -> PathBuf {
    PathBuf::from("..").join("auto-os-config")
}

/// storage 键 `shell.osconfig.daemon`（G4 发现序第 1 级；缺席/空 = None）。
pub fn daemon_path_override() -> Option<String> {
    crate::vm::ffi::stdlib::storage_host_read("shell.osconfig.daemon")
        .filter(|s| !s.trim().is_empty())
}

/// G1 主流程：检活 → 未运行则按发现序 spawn → 就绪轮询 ≤`ready_timeout`
/// （200ms 间隔）→ `Running(url)` / `Offline(reason)`。已运行 daemon 零打扰
/// 复用（不 spawn）；spawn 只带 `AUTOOS_BACK_PORT=<port>`（兑现生产端口
/// 17701——daemon 缺省 17901）。PATH 查找固定 None（v1 不做 PATH 扫描，
/// 相邻仓/显式配置已覆盖开发机形态；发现序第 3 级留扩展位）。
pub fn ensure_ready_io(
    url: &str,
    override_path: Option<&str>,
    sibling_root: &Path,
    io: &mut dyn DaemonIo,
    ready_timeout: std::time::Duration,
) -> DaemonStatus {
    if io.ping(url) {
        return DaemonStatus::Running(url.to_string());
    }
    let Some(path) = resolve_daemon_path(override_path, sibling_root, |_| None) else {
        return DaemonStatus::Offline(
            "daemon 可执行未找到（发现序：shell.osconfig.daemon > ../auto-os-config 相邻仓 target）".to_string(),
        );
    };
    let env = vec![(ENV_BACK_PORT.to_string(), port_of(url).to_string())];
    if let Err(err) = io.spawn(&path, &env) {
        return DaemonStatus::Offline(format!("spawn {} 失败: {err}", path.display()));
    }
    let deadline = std::time::Instant::now() + ready_timeout;
    while std::time::Instant::now() < deadline {
        if io.ping(url) {
            return DaemonStatus::Running(url.to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    DaemonStatus::Offline(format!(
        "daemon 就绪超时（{}ms 内 ping 不通 {url}；{}）",
        ready_timeout.as_millis(),
        path.display()
    ))
}

/// 生产入口（缺省参数组：17701 + storage 覆盖 + 相邻仓根 + 5s 就绪）。
pub fn ensure_ready(url: &str) -> DaemonStatus {
    ensure_ready_io(
        url,
        daemon_path_override().as_deref(),
        &default_sibling_root(),
        &mut RealDaemonIo::new(),
        std::time::Duration::from_secs(5),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sibling_fixture() -> PathBuf {
        // 临时相邻仓根：释放 release/auto-os-config-back-server(.exe)。
        let root = std::env::temp_dir().join("autoui-501-daemon-fixture");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(
            root.join("auto-os-config-back").join("target").join("release"),
        )
        .unwrap();
        root
    }

    fn touch_server_exe(root: &Path) -> PathBuf {
        let bin = root
            .join("auto-os-config-back")
            .join("target")
            .join("release")
            .join(if cfg!(windows) {
                format!("{DAEMON_BIN_NAME}.exe")
            } else {
                DAEMON_BIN_NAME.to_string()
            });
        std::fs::write(&bin, b"stub").unwrap();
        bin
    }

    #[test]
    fn resolve_order_explicit_wins() {
        let root = sibling_fixture();
        let exe = touch_server_exe(&root);
        // 显式配置优先——即便相邻仓 target 也存在。
        let got = resolve_daemon_path(Some("D:/custom/daemon.exe"), &root, |_| {
            panic!("PATH 不应触达")
        });
        assert_eq!(got, Some(PathBuf::from("D:/custom/daemon.exe")));
        // 空白串视为缺席（storage 坏值容错），落相邻仓。
        let got = resolve_daemon_path(Some("  "), &root, |_| None);
        assert_eq!(got, Some(exe));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_order_sibling_target_then_path() {
        // 相邻仓无产物 → PATH 兜底。
        let root = sibling_fixture();
        let got = resolve_daemon_path(None, &root, |name| {
            assert_eq!(name, format!("{DAEMON_BIN_NAME}.exe").as_str());
            Some(PathBuf::from("/usr/bin/found"))
        });
        assert_eq!(got, Some(PathBuf::from("/usr/bin/found")));
        // 相邻仓有产物 → PATH 不触达。
        let exe = touch_server_exe(&root);
        let got = resolve_daemon_path(None, &root, |_| panic!("PATH 不应触达"));
        assert_eq!(got, Some(exe));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_all_miss_is_none() {
        let root = sibling_fixture();
        assert_eq!(resolve_daemon_path(None, &root, |_| None), None);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn should_spawn_decision() {
        assert!(!should_spawn(true), "ping 通 = 复用既有，零打扰");
        assert!(should_spawn(false), "ping 不通 = 走 spawn");
    }

    #[test]
    fn env_injection_shape() {
        let env = env_for("http://127.0.0.1:17708");
        assert_eq!(env, vec![("AUTOOS_DAEMON".to_string(), "http://127.0.0.1:17708".to_string())]);
    }

    #[test]
    fn url_port_helpers() {
        assert_eq!(default_daemon_url(), "http://127.0.0.1:17701");
        assert_eq!(port_of("http://127.0.0.1:17701"), 17701);
        assert_eq!(port_of("http://127.0.0.1:17708"), 17708);
        assert_eq!(port_of("bad"), DAEMON_PORT, "非法 url 回退缺省端口");
    }

    // ---- T2：检活/spawn 状态机（注入式假 IO）+ 真实 TCP ping ----

    /// 假执行环境：ping 按脚本表走（每调一次弹一个）；spawn 记录参数。
    struct FakeIo {
        ping_script: Vec<bool>,
        ping_calls: usize,
        spawn_result: Result<(), String>,
        spawn_path: Option<PathBuf>,
        spawn_env: Vec<(String, String)>,
    }

    impl FakeIo {
        fn new(ping_script: Vec<bool>) -> Self {
            Self {
                ping_script,
                ping_calls: 0,
                spawn_result: Ok(()),
                spawn_path: None,
                spawn_env: Vec::new(),
            }
        }
    }

    impl DaemonIo for FakeIo {
        fn ping(&mut self, _url: &str) -> bool {
            let i = self.ping_calls.min(self.ping_script.len() - 1);
            self.ping_calls += 1;
            self.ping_script[i]
        }
        fn spawn(&mut self, path: &Path, env: &[(String, String)]) -> Result<(), String> {
            self.spawn_path = Some(path.to_path_buf());
            self.spawn_env = env.to_vec();
            self.spawn_result.clone()
        }
    }

    #[test]
    fn ensure_ready_reuses_running_daemon() {
        let mut io = FakeIo::new(vec![true]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            Path::new("Z:/nowhere"),
            &mut io,
            std::time::Duration::from_secs(1),
        );
        assert_eq!(st, DaemonStatus::Running("http://127.0.0.1:17701".to_string()));
        assert_eq!(io.ping_calls, 1, "ping 通即复用");
        assert!(io.spawn_path.is_none(), "已运行 daemon 零打扰——不 spawn");
    }

    #[test]
    fn ensure_ready_offline_when_binary_not_found() {
        let mut io = FakeIo::new(vec![false; 10]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            Path::new("Z:/nowhere"),
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(reason.contains("未找到"), "原因应说明路径未找到: {reason}")
            }
            other => panic!("应 Offline，实际 {other:?}"),
        }
        assert!(io.spawn_path.is_none(), "无路径不 spawn");
    }

    #[test]
    fn ensure_ready_offline_when_spawn_fails() {
        let root = sibling_fixture();
        let exe = touch_server_exe(&root);
        let mut io = FakeIo::new(vec![false; 10]);
        io.spawn_result = Err("拒绝访问".to_string());
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            &root,
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(reason.contains("spawn") && reason.contains("拒绝访问"), "{reason}")
            }
            other => panic!("应 Offline，实际 {other:?}"),
        }
        assert_eq!(io.spawn_path, Some(exe), "发现序落相邻仓 target");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_ready_spawns_then_ready_with_port_env() {
        let root = sibling_fixture();
        touch_server_exe(&root);
        // ping 脚本：首次 false（触发 spawn）→ 之后 true（就绪）。
        let mut io = FakeIo::new(vec![false, true, true]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            &root,
            &mut io,
            std::time::Duration::from_secs(5),
        );
        assert_eq!(st, DaemonStatus::Running("http://127.0.0.1:17701".to_string()));
        // spawn env 只带端口覆盖（生产 17701——daemon 缺省 17901 需显式改）。
        assert_eq!(
            io.spawn_env,
            vec![(ENV_BACK_PORT.to_string(), "17701".to_string())]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_ready_offline_on_ready_timeout() {
        let root = sibling_fixture();
        touch_server_exe(&root);
        // spawn 成功但 ping 恒不通 → 就绪超时（ready_timeout 压到 1ms 免慢测）。
        let mut io = FakeIo::new(vec![false; 10]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            &root,
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => assert!(reason.contains("就绪超时"), "{reason}"),
            other => panic!("应 Offline，实际 {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_ready_override_path_used_for_spawn() {
        let root = sibling_fixture();
        touch_server_exe(&root);
        let mut io = FakeIo::new(vec![false, true, true]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17708",
            Some("D:/tools/custom-daemon.exe"),
            &root,
            &mut io,
            std::time::Duration::from_secs(5),
        );
        assert_eq!(st, DaemonStatus::Running("http://127.0.0.1:17708".to_string()));
        assert_eq!(io.spawn_path, Some(PathBuf::from("D:/tools/custom-daemon.exe")));
        assert_eq!(
            io.spawn_env,
            vec![(ENV_BACK_PORT.to_string(), "17708".to_string())],
            "端口取自目标 url"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 迷你 HTTP 服务（单线程 accept 循环，响应 200）——真实 tcp_ping 正路径。
    fn mini_health_server() -> String {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut s = stream;
                let mut buf = [0u8; 512];
                let _ = s.read(&mut buf);
                let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}");
            }
        });
        url
    }

    #[test]
    fn tcp_ping_real_server_and_dead_port() {
        let url = mini_health_server();
        assert!(tcp_ping(&url, std::time::Duration::from_secs(2)), "真服务 200 应通");
        // 死端口：bind 后立即 drop → connect 拒绝。
        let dead = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let dead_url = format!("http://127.0.0.1:{}", dead.local_addr().unwrap().port());
        drop(dead);
        assert!(!tcp_ping(&dead_url, std::time::Duration::from_secs(2)), "死端口应不通");
        // 非 http 形态 → false（不 panic）。
        assert!(!tcp_ping("not-a-url", std::time::Duration::from_secs(1)));
        assert!(socket_addr_of("http://127.0.0.1:17701").is_some());
        assert!(socket_addr_of("ftp://x").is_none());
    }

    #[test]
    fn real_io_ping_wires_tcp_ping() {
        // RealDaemonIo::ping 即 tcp_ping（2s 档）——经 trait 走一遍真路径。
        let url = mini_health_server();
        let mut io = RealDaemonIo::new();
        assert!(io.ping(&url));
    }
}
