//! PLAN-736: 工程级 HTTP 服务独立启动（`auto serve`）。
//!
//! 两条服务轨，同一份解析后的 [HttpServiceConfig]：
//! - **vm**：进程内 VM 网络轨（`auto_lang::run_file` + 服务配置 seam），
//!   不启动任何 UI/Vite/桌面。
//! - **rust**：真实生成链（`api_gen::generate_api`，strict 门）→ 子进程
//!   `cargo run`，配置经 `AUTO_HTTP_SERVICE_JSON` 注入，子进程自行
//!   re-resolve（两端 effective hash 必须一致）。
//!
//! 与 legacy `auto run` 的边界：serve 面**不 kill 占端口进程**（端口冲突 =
//! bind 失败 = 非零退出诊断）；无文件 legacy 端口链不受影响。

use auto_lang::http_service_config::{
    resolve_service_config, set_active_service_config, HttpServiceConfig,
};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 显式 `--http-config` 的两种载体（文件路径 / 内联 JSON）。
#[derive(Debug, Clone)]
pub enum HttpConfigSource {
    File(PathBuf),
    Inline(String),
}

impl HttpConfigSource {
    fn read_json(&self) -> Result<String, String> {
        match self {
            HttpConfigSource::Inline(s) => Ok(s.clone()),
            HttpConfigSource::File(p) => {
                let mut s = String::new();
                std::fs::File::open(p)
                    .map_err(|e| format!("failed to open http-config {}: {e}", p.display()))?
                    .read_to_string(&mut s)
                    .map_err(|e| format!("failed to read http-config {}: {e}", p.display()))?;
                Ok(s)
            }
        }
    }
}

/// 解析最终服务配置。
///
/// - `source`：None = 隐式 development 缺省（决策 §3.1：loopback:8080）。
/// - `cli_port`：`-B` 显式旗标（覆盖 listen.port）；与配置 listen 的并存
///   语义在 resolve 内实现。
pub fn resolve_config(
    source: Option<&HttpConfigSource>,
    cli_port: Option<u16>,
) -> Result<HttpServiceConfig, String> {
    match source {
        Some(src) => {
            let json = src.read_json()?;
            resolve_service_config(&json, cli_port).map_err(|e| e.to_string())
        }
        None => resolve_service_config(r#"{"profile":"development"}"#, cli_port)
            .map_err(|e| e.to_string()),
    }
}

/// 服务入口（`auto serve` 实现）。返回进程退出码。
///
/// 运行失败（生成失败/bind 冲突/VM 编译失败）返回非零码；调用方据此退出。
pub fn serve_project(
    project_dir: &Path,
    server: &str,
    config: HttpServiceConfig,
) -> Result<i32, String> {
    match server {
        "vm" => serve_vm(project_dir, config),
        "rust" => serve_rust(project_dir, config),
        other => Err(format!(
            "unknown --server `{other}` (expected `vm` or `rust`)"
        )),
    }
}


/// 以项目目录为锚解析 rust workspace 落点（resolve 序的 CWD 无关镜像）：
/// 项目在框架仓内（向上找到含 crates/auto-lang 的仓根）→ 共享
/// `<repo>/examples/rust-workspace`；仓外项目 → `<project>/rust-workspace`。
fn resolve_workspace_anchored(project_dir: &Path) -> PathBuf {
    let mut dir = project_dir.to_path_buf();
    for _ in 0..10 {
        if dir.join("crates").join("auto-lang").exists() {
            return dir.join("examples").join("rust-workspace");
        }
        if !dir.pop() {
            break;
        }
    }
    project_dir.join("rust-workspace")
}

// ============================================================================
// VM 轨：进程内 VM 网络服务
// ============================================================================

fn serve_vm(project_dir: &Path, config: HttpServiceConfig) -> Result<i32, String> {
    let api_path = auto_lang::config::resolve_back_api(project_dir).ok_or_else(|| {
        format!(
            "no back API found under {} (expected src/back/api.at)",
            project_dir.display()
        )
    })?;
    let api_str = api_path.to_string_lossy().to_string();

    // 模块搜索目录注入（与 rust_ui::start_vm_server 同一语义：split 模式
    // api.at 的 `use types` 需要解析 src/front）。
    let src_front = project_dir.join("src").join("front");
    if src_front.exists() {
        let joined = src_front.to_string_lossy().to_string();
        let existing = std::env::var("AUTO_SOURCE_DIRS").unwrap_or_default();
        let combined = if existing.is_empty() {
            joined
        } else {
            format!("{existing};{joined}")
        };
        std::env::set_var("AUTO_SOURCE_DIRS", &combined);
    }

    auto_lang::vm::ffi::http_server::reset_serve_state();
    set_active_service_config(Arc::new(config.clone()));
    let display_addr = format!("{}:{}", config.listen_addr, config.listen_port);
    println!(
        "▶ Starting VM HTTP service on {display_addr} (profile: {})",
        config.profile.as_str()
    );

    // 与 start_vm_server 同款大栈线程（flattened api.at 解析超 1MB 默认栈）。
    let handle = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .name("auto-vm-http-service".into())
        .spawn(move || auto_lang::run_file(&api_str))
        .map_err(|e| format!("failed to spawn VM service thread: {e}"))?;

    // bind 判定：等待 serve hook 报告真实 bound 地址（port=0 → 内核分配）。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    let mut bound = None;
    while std::time::Instant::now() < deadline {
        if let Some(addr) = auto_lang::vm::ffi::http_server::bound_addr_snapshot() {
            bound = Some(addr);
            break;
        }
        if handle.is_finished() {
            break; // 编译失败/启动失败：join 后按 fatal 报告
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    match bound {
        Some(addr) => {
            println!(
                "  ✓ VM HTTP service ready (bound {})",
                addr
            );
            println!(
                "  AUTO_SERVICE_READY {{\"bound\":\"{addr}\",\"profile\":\"{}\",\"config_hash\":\"{:016x}\"}}",
                config.profile.as_str(),
                config.effective_config_hash()
            );
        }
        None => {
            // 未在期限内 bind：join 收尸并按 fatal 非零退出（消息三源取一：
            // serve fatal > run_file 的 Err > 兜底超时文案）。
            let join_result = handle.join();
            let fatal = auto_lang::vm::ffi::http_server::take_serve_fatal_error();
            if let Some(f) = fatal {
                return Err(f);
            }
            if let Ok(Err(run_err)) = join_result {
                return Err(format!("VM service run failed: {run_err}"));
            }
            return Err("VM HTTP service did not become ready within 120s".to_string());
        }
    }

    // Ctrl+C 由 serve_async 内注册的处理器驱动优雅关闭；此处 join 至线程结束。
    let run_result = handle.join();
    let fatal = auto_lang::vm::ffi::http_server::take_serve_fatal_error();
    match (run_result, fatal) {
        (Ok(_), None) => {
            println!("  ✓ VM HTTP service stopped cleanly");
            Ok(0)
        }
        (Ok(_), Some(f)) | (Err(_), Some(f)) => Err(format!("http service failed: {f}")),
        (Err(_), None) => Err("VM service thread failed".to_string()),
    }
}

// ============================================================================
// Rust 轨：生成 + 子进程承载
// ============================================================================

fn serve_rust(project_dir: &Path, config: HttpServiceConfig) -> Result<i32, String> {
    // workspace 落点以**项目目录**为锚（CWD 无关）：get_rust_workspace_dir
    // 从进程 CWD 走锚点是 legacy 语义，`auto service` 可从任意目录启动。
    // 走 env 档（resolve 序第一档，设置即权威）让 ensure/generate/cargo run
    // 三方一致。
    if std::env::var_os("AUTO_RUST_WORKSPACE").is_none() {
        let anchored = resolve_workspace_anchored(project_dir);
        std::env::set_var("AUTO_RUST_WORKSPACE", &anchored);
    }
    // 共享 workspace 根重写（成员表+相对路径锚点按实际落点重算；serve 面无
    // generate_rust_ui 前置，必须自行 ensure——陈旧根的坏 path dep 会毒化
    // 整个 workspace 的 cargo 解析）。
    let ws_dir = crate::rust_ui::ensure_shared_workspace(project_dir);
    // 真实生成链（734 strict 门；失败=Err 硬传播，不落旧产物）。
    crate::api_gen::generate_api(project_dir, "rust")
        .map_err(|e| format!("Failed to generate Rust backend: {e}"))?;
    let back_name = crate::rust_ui::back_member_name(project_dir);
    let backend_dir = ws_dir.join(&back_name);
    let cargo_toml = backend_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(format!(
            "backend Cargo.toml missing under {} (generate_api reported success?)",
            backend_dir.display()
        ));
    }

    let display_addr = format!("{}:{}", config.listen_addr, config.listen_port);
    println!(
        "▶ Starting generated Rust HTTP service on {display_addr} (profile: {})",
        config.profile.as_str()
    );
    println!("  cargo run --manifest-path {}", cargo_toml.display());

    // 配置注入：子进程 re-resolve 规范形（决策 §3.1：env 注入，非文件散落）。
    let service_json = config.to_json();
    let mut cmd = std::process::Command::new("cargo");
    // cwd = workspace 落点：cargo 从 CWD（非 manifest 目录）发现
    // .cargo/config.toml（shared target-dir + MSVC 栈 rustflags）——从项目
    // 目录 spawn 会拿不到配置 → 不同指纹 → 全量冷编（onig_sys C1083 链）。
    cmd.args(["run", "--manifest-path", cargo_toml.to_str().unwrap_or(".")])
        .env("AUTO_HTTP_SERVICE_JSON", &service_json)
        .current_dir(&ws_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn backend cargo run: {e}"))?;
    println!("  ✓ backend spawning (PID: {}); Ctrl+C stops the service", child.id());

    // 子进程 ready/身份判定（health 轮询）在 T-05 接管；T-02 以子进程退出码
    // 为准：bind 冲突/配置错误 = 子进程非零 → 本进程同样非零。
    let status = child.wait().map_err(|e| format!("backend wait failed: {e}"))?;
    if status.success() {
        println!("  ✓ Rust HTTP service stopped cleanly");
        Ok(0)
    } else {
        let code = status.code().unwrap_or(1);
        Err(format!(
            "backend exited with {code} — bind conflict or startup failure (see output above)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_config_none_is_development_loopback() {
        let c = resolve_config(None, None).unwrap();
        assert_eq!(c.profile, auto_lang::http_service_config::ServiceProfile::Development);
        assert_eq!(c.listen_port, 8080);
    }

    #[test]
    fn resolve_config_cli_port_overrides() {
        let c = resolve_config(None, Some(9123)).unwrap();
        assert_eq!(c.listen_port, 9123);
    }

    #[test]
    fn resolve_config_bad_file_errors() {
        let e = resolve_config(
            Some(&HttpConfigSource::File(PathBuf::from("Z:/definitely/missing/service.json"))),
            None,
        )
        .unwrap_err();
        assert!(e.contains("failed to open"), "{e}");
    }

    #[test]
    fn resolve_config_inline_bad_json_errors_named() {
        let e = resolve_config(
            Some(&HttpConfigSource::Inline(r#"{"profile":"development","limits":{"max_connections":0}}"#.into())),
            None,
        )
        .unwrap_err();
        assert!(e.contains("max_connections"), "{e}");
    }

    #[test]
    fn serve_project_rejects_unknown_server() {
        let c = resolve_config(None, None).unwrap();
        let err = serve_project(Path::new("."), "vue", c).unwrap_err();
        assert!(err.contains("unknown --server"), "{err}");
    }
}
