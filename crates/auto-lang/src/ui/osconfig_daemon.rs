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
}
