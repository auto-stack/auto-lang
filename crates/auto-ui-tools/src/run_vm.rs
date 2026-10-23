//! PLAN-815 T-05: VM 臂运行面自 auto-man `rust_ui_vm_compat` 迁入（M814-B
//! 门解除后的 T-04 残留）。依赖全部经新公开包解析：解释器执行
//! `auto_lang_core::execute_autovm_with_path`、动态 UI `auto_ui_runtime::
//! run_file_dynamic_ui`、外部后端 ABI `auto_lang_core::vm::{backend_abi,
//! host_bridge}`、renderer 事件注入 `auto_ui_runtime::ui::iced::renderer`、
//! back-proxy/画廊注册面 `crate::run_vue`。零 legacy `auto_lang` 依赖。
//!
//! 托管模式（工具宿主安装 [`crate::proc_host::RunIo`]）：iced 执行经
//! `proc_host::vm_main_hook` 交回宿主的 OS 主线程（iced 约束）；步骤
//! 边界取消收口。未安装时行为与迁移前一致（CLI 主线程直接执行）。

use std::path::Path;

use colored::Colorize;

use auto_val::{AutoError, AutoResult};

use crate::proc_host::{self, FlowEnd};
use crate::rust_ui::{probe_service_ready, start_api_server, stop_api_server, PacSource};

/// 旧 `auto_lang::run_file` 的同语义复刻（读文件 → UI 关键字分流）：
/// UI 入口走动态 UI 管线（iced），否则走通用解释器执行（#[api] serve
/// 检测在执行器内）。32MB 栈线程承载（扁平化 api.at+db.at+types.at 超
/// 1MB 默认栈）。
/// T-05 R2（R2-01）：内部 VM 会话执行臂的入口（bin --internal-vm-exec
/// 消费）；legacy CLI 与宿主钩子同走本口。
pub fn run_entry_file(path: &str) -> auto_lang_core::AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    if auto_ui_runtime::has_ui_keywords(&code) {
        return execute_vm_entry(&code, Some(path));
    }
    let path = path.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(auto_lang_core::vm_thread_stack_size())
        .name("auto-vm-exec".into())
        .spawn(move || {
            let (result, stdout) = auto_lang_core::run_with_capture_and_path(&code, &path)?;
            if !stdout.is_empty() {
                println!("{stdout}");
            }
            Ok(result)
        })
        .expect("spawn VM exec thread");
    match handle.join() {
        Ok(r) => r,
        Err(_) => Err("execution thread panicked".into()),
    }
}

/// 执行动态 UI 入口：托管时交宿主主线程，未托管在当前线程（= CLI 的
/// OS 主线程，迁移前行为）。
fn execute_vm_entry(code: &str, path: Option<&str>) -> auto_lang_core::AutoResult<String> {
    match proc_host::vm_main_hook() {
        Some(hook) => hook(code, path),
        None => auto_ui_runtime::run_file_dynamic_ui(code, path),
    }
}

/// Plan 340 fix: Start the AutoVM HTTP server (not the a2r Rust backend) for
/// VM+VM split mode. Spawns a background thread running the api.at entry
/// (serve detection inside), polls the port until ready (or timeout). The
/// thread runs for the process lifetime (cleaned up when the process exits).
///
/// Returns true if the server became ready, false on timeout / missing api.at.
pub fn start_vm_server(project_dir: &Path, front_dir: Option<&Path>) -> bool {
    let api_path = project_dir.join("src").join("back").join("api.at");
    if !api_path.exists() {
        eprintln!(
            "  {} VM split mode requires src/back/api.at (not found)",
            "⚠".bright_yellow()
        );
        return false;
    }

    let port = crate::util::http_port();
    println!();
    println!(
        "  {} Starting AutoVM HTTP server (VM backend, port {})...",
        "▶".bright_cyan(),
        port
    );
    println!("    api.at: {}", api_path.display());

    let api_str = api_path.to_string_lossy().to_string();
    // Seed extra module search dirs for the standalone api.at run. A split-mode
    // api.at imports `types` from <front>/types.at, but the entry only seeds
    // the source file's own directory (src/back) — without this, `use types`
    // fails with "Module not found". Inject the (request-declared) front dir
    // via AUTO_SOURCE_DIRS (PLAN-815 T-06 R1: front_decl is honored).
    let extra_dirs: Vec<std::path::PathBuf> = match front_dir {
        Some(dir) if dir.exists() => vec![dir.to_path_buf()],
        _ => Vec::new(),
    };
    if !extra_dirs.is_empty() {
        let joined = extra_dirs
            .iter()
            .map(|d| d.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(";");
        let existing = std::env::var("AUTO_SOURCE_DIRS").unwrap_or_default();
        let combined = if existing.is_empty() {
            joined
        } else {
            format!("{};{}", existing, joined)
        };
        std::env::set_var("AUTO_SOURCE_DIRS", &combined);
        println!("    module search dirs: {}", combined);
    }
    // Spawn the VM server on a large-stack background thread (the flattened
    // api.at + db.at + types.at exceeds the 1MB default during parse/codegen).
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .name("auto-vm-http-server".into())
        .spawn(move || {
            if let Err(e) = run_entry_file(&api_str) {
                eprintln!("[AutoVM server] error: {}", e);
            }
        })
        .expect("spawn VM server thread");

    // Wait for the server to become ready by polling the port.
    // PLAN-026 T-05: 同 start_api_server 有界重试门(60s→120s)。
    println!("    Waiting for AutoVM server to be ready...");
    let max_wait = std::time::Duration::from_secs(120);
    let start = std::time::Instant::now();
    let probe_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    while start.elapsed() < max_wait {
        if proc_host::cancelled() {
            eprintln!("ERROR: cancelled while waiting for the AutoVM server");
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        // PLAN-736 T-05：health 优先（VM serve 面有控制面；legacy auto run
        // 的 VM 轨无 /__auto 路由 → 404 视为可继续），TCP connect 兜底。
        // PLAN-781 R1 勘定：legacy VM 轨的 __auto 控制面实际已注册（共享
        // transport），run 模式下 /ready 恒 503 "starting"（ready 态翻转只在
        // service 面发生）→ Some(false) 不能再跳过 TCP 兜底，否则 -r vm
        // --no-merge 原生窗口 split 形态 120s 必死。TCP 语义恢复为最终裁决。
        if probe_service_ready(port) == Some(true) {
            println!(
                "  {} AutoVM server ready on http://127.0.0.1:{}",
                "✓".bright_green(),
                port
            );
            return true;
        }
        if std::net::TcpStream::connect_timeout(&probe_addr, std::time::Duration::from_secs(1))
            .is_ok()
        {
            println!(
                "  {} AutoVM server ready on http://127.0.0.1:{}",
                "✓".bright_green(),
                port
            );
            return true;
        }
    }
    // PLAN-026 T-05: 超限显式错误(vm+vm split 独立卡死症状另档
    // §10.2;本门语义 = 不再吞错续跑)。
    eprintln!(
        "ERROR: AutoVM server not ready within {}s (port {}); aborting before UI init",
        max_wait.as_secs(),
        port
    );
    false
}

/// merged 模式宿主注册表:后端 cdylib 经它把端点实现注册进既有宿主桥,
/// 事件回流复用 renderer 的 SSE 注入通道(与 ash-runner 手写宿主同构)。
struct VmHostRegistry;

impl auto_lang_core::vm::backend_abi::BackendRegistry for VmHostRegistry {
    fn host_call(&self, name: &str, f: auto_lang_core::vm::backend_abi::BackendHostCallFn) {
        auto_lang_core::vm::host_bridge::register_host_call(name, f);
    }
    fn inject_event(&self, tag: &str, json: &str) -> bool {
        auto_ui_runtime::ui::iced::renderer::inject_shell_event(tag, json)
    }
    fn log(&self, msg: &str) {
        eprintln!("[backend] {msg}");
    }
}

/// T-05 R3（R3-01）：执行进程的外部后端装配——链接 `back.*` 模块解析根
/// 并装载 cdylib（宿主桥注册）。参数为**请求衍生**的绝对后端根（父进程
/// 从单源请求求值后经 env 传递；子进程不重读主 pac，不制造第二真源）。
/// 返回 keepalive——调用方必须持有到会话结束。
pub fn prepare_external_backend_from_root(
    backend_dir: &Path,
) -> Result<auto_lang_core::vm::backend_abi::LoadedBackend, String> {
    if backend_dir.join("api.at").is_file() {
        auto_lang_core::set_external_back_root(backend_dir.to_path_buf());
        println!(
            "  {} external backend linked: back.* -> {}",
            "✓".bright_green(),
            backend_dir.display()
        );
    } else {
        eprintln!(
            "  {} external backend: no api.at in {}",
            "⚠".bright_yellow(),
            backend_dir.display()
        );
    }
    load_external_backend(backend_dir)
}

/// 在后端项目的 target/{debug,release} 下定位其 cdylib 并装载注册。
/// 库名 = 后端 pac.at `name` 规整('-'→'_',Windows `x.dll` / macOS
/// `libx.dylib` / Linux `libx.so`);缺失时给出"先构建后端"的明确指引。
fn load_external_backend(
    backend_dir: &Path,
) -> Result<auto_lang_core::vm::backend_abi::LoadedBackend, String> {
    use auto_val::{AutoStr, Obj, Value};
    let pac_path = backend_dir.join("pac.at");
    let lib_stem = if pac_path.is_file() {
        let mut probe = Obj::new();
        probe.set("port", Value::Str(AutoStr::from("win32")));
        auto_lang_core::config::AutoConfig::from_file(&pac_path, &probe)
            .ok()
            .map(|c| c.name().to_string())
            .filter(|n| !n.is_empty())
            .map(|n| n.replace('-', "_"))
            .unwrap_or_else(|| "backend".to_string())
    } else {
        "backend".to_string()
    };
    let lib_file = if cfg!(windows) {
        format!("{lib_stem}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{lib_stem}.dylib")
    } else {
        format!("lib{lib_stem}.so")
    };
    let candidates = [
        backend_dir.join("target").join("debug").join(&lib_file),
        backend_dir.join("target").join("release").join(&lib_file),
    ];
    let found = candidates.iter().find(|p| p.is_file());
    let Some(lib_path) = found else {
        return Err(format!(
            "backend cdylib `{}` not found under {}/target/{{debug,release}} — build the backend project first (`cargo build` in {})",
            lib_file,
            backend_dir.display(),
            backend_dir.display()
        ));
    };
    auto_lang_core::vm::backend_abi::load_backend_cdylib(
        lib_path,
        std::sync::Arc::new(VmHostRegistry),
    )
}

/// Run the UI via the AutoLang interpreter (--render=vm mode).
/// Legacy signature (behavior frozen; hosted hooks engage only when the
/// tool host installed a RunIo).
pub fn run_vm_ui(project_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    let source = PacSource::legacy();
    run_vm_ui_flow(project_dir, args, &source).map(|_| ())
}

/// 请求路径（PLAN-809 T-06c 生产入口）。
pub fn run_vm_ui_with_request(
    project_dir: &Path,
    args: Vec<String>,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    let source = PacSource::from_request(req);
    run_vm_ui_flow(project_dir, args, &source).map(|_| ())
}

pub fn run_vm_ui_flow(
    project_dir: &Path,
    _args: Vec<String>,
    source: &PacSource,
) -> Result<FlowEnd, AutoError> {
    let total_steps = 4u64;
    let step = |n: u64, what: &str| proc_host::progress(n, Some(total_steps), what);

    // Plan 334: vm+vm 同进程合并。前端 widget VM（经 Plan 333）直接链接后端
    // 函数访问数据（list_notes → db.all_notes → notes 全局），不需要独立的 Axum
    // HTTP 后端进程。跳过 start_api_server 可消除冗余 cargo 编译、端口占用、
    // 启动等待。
    //
    // Plan 340: --no-merge（AUTO_VM_MERGE=0）切换到分离模式：启动独立后端 HTTP
    // 进程，前端 VM 通过 HTTP 调用后端 API（codegen 把 #[api] 调用改写成 HTTP）。
    // AUTO_VM_WITH_HTTP=1 是旧开关，等价于分离模式（向后兼容）。
    //
    // Plan 340 fix: 分离模式启动独立后端 HTTP 进程。后端可以是 AutoVM server
    // （VM+VM split）或 a2r 转译的 Rust axum server（VM+Rust）。
    // AUTO_BACKEND_IMPL=rust → Rust 后端（start_api_server）
    step(1, "resolve backend");
    // PLAN-815 T-06 R1：split 模式 api.at 的模块搜索种子同样消费请求
    // 声明的前端目录（在 split 分支前求值一次）。
    let front_seed = source.declared_front_dir(project_dir);
    let pac_path = project_dir.join("pac.at");
    let has_rust_api = source.api_is_rust(&pac_path);
    let backend_impl = std::env::var("AUTO_BACKEND_IMPL").unwrap_or_else(|_| {
        if has_rust_api {
            "rust".to_string()
        } else {
            "vm".to_string()
        }
    });
    let split_mode = std::env::var("AUTO_VM_MERGE").as_deref() == Ok("0")
        || std::env::var("AUTO_VM_WITH_HTTP").as_deref() == Ok("1")
        || backend_impl == "rust";
    let mut _api_child = if split_mode {
        if backend_impl == "rust" {
            // VM+Rust split: ensure the Rust axum server is generated, then
            // start it. generate_api("rust") writes Cargo.toml + main.rs +
            // api.rs + types.rs from #[api] annotations (idempotent).
            // PLAN-734 T-06：Err 传播（split 生成失败不再 warn 后仍启动——
            // start_api_server 内的二次生成同错误也会硬传，此处提前终结）。
            auto_api_gen::api_gen::generate_api(project_dir, "rust")
                .map_err(|e| format!("Failed to generate Rust backend: {}", e))?;
            let child = match start_api_server(project_dir) {
                Ok(child) => child,
                // PLAN-026 T-05: ready 失败显式中止(不进组件构建/Init)。
                Err(e) => return Err(e.into()),
            };
            if child.is_some() && std::env::var_os("AUTO_HTTP_BASE").is_none() {
                std::env::set_var(
                    "AUTO_HTTP_BASE",
                    format!("http://127.0.0.1:{}", crate::util::http_port()),
                );
            }
            child
        } else {
            // VM+VM split: AutoVM HTTP server as backend.
            // PLAN-026 T-05: ready 失败显式中止(不进组件构建/Init)。
            if !start_vm_server(project_dir, front_seed.as_deref()) {
                if proc_host::cancelled() {
                    return Ok(FlowEnd::Cancelled);
                }
                return Err("AutoVM backend not ready; aborting before UI init".into());
            }
            if std::env::var_os("AUTO_HTTP_BASE").is_none() {
                std::env::set_var(
                    "AUTO_HTTP_BASE",
                    format!("http://127.0.0.1:{}", crate::util::http_port()),
                );
            }
            None
        }
    } else {
        println!();
        println!(
            "  {} vm+vm merged mode: backend runs in-process (no HTTP server)",
            "✓".bright_green()
        );
        None
    };
    if proc_host::cancelled() {
        stop_api_server(&mut _api_child);
        return Ok(FlowEnd::Cancelled);
    }

    // Plan 061:外部后端(pac.at `back: { project: "..." }`)——merged 模式下
    // 由宿主链接并装载后端项目:① 链接式契约引用(`back.*` 解析映射到
    // 后端根,前端无需本地 back/,零复制);② libloading 装载 cdylib +
    // ABI 校验 + 注册进宿主桥(vm::host_bridge,与 ash-runner 同表)。
    // 无 back.project 配置时零行为变化(本地 back/ 现状)。
    let mut _backend_keepalive: Option<auto_lang_core::vm::backend_abi::LoadedBackend> = None;
    // T-05 R4（F-815-T05-R4-01）：托管形态下后端/主题上下文不再写宿主
    // 进程全局 env（None/无效根不清理 → 串行下一请求沿用上一后端的
    // 根因），改为**每请求精确条目**经 RunIo 传给会话子进程 spawn：
    // 请求未衍生该键时它被从继承环境移除——第二个请求只获得自己的
    // 上下文；装载本身仍在执行进程（internal-vm-exec 子进程）进行。
    if proc_host::current().is_some() {
        let mut session_env: Vec<(String, String)> = Vec::new();
        if !split_mode {
            match source.external_backend(project_dir) {
                Ok(Some(backend_rel)) => {
                    let backend_dir = project_dir
                        .join(backend_rel.as_str())
                        .canonicalize()
                        .unwrap_or_else(|_| project_dir.join(backend_rel.as_str()));
                    session_env.push((
                        "AUTO_UI_TOOL_VM_BACK_ROOT".to_string(),
                        backend_dir.to_string_lossy().to_string(),
                    ));
                }
                Ok(None) => {}
                Err(e) => eprintln!("  {} pac.at parse failed: {}", "⚠".bright_yellow(), e),
            }
        }
        if let Some(lite) = source.theme_decl_lite() {
            // 字段皆简单标量/字符串对偶——手写 JSON 组装（project-model 无
            // serde 依赖，wire 为手写编解码，同族做法）。
            let json = serde_json::json!({
                "name": lite.name,
                "extends": lite.extends,
                "mode": lite.mode,
                "colors": lite.colors,
            })
            .to_string();
            session_env.push(("AUTO_UI_TOOL_VM_THEME_DECL".to_string(), json));
        }
        proc_host::set_vm_session_env(session_env);
    } else if !split_mode {
        match source.external_backend(project_dir) {
            Ok(Some(backend_rel)) => {
                let backend_dir = project_dir
                    .join(backend_rel.as_str())
                    .canonicalize()
                    .unwrap_or_else(|_| project_dir.join(backend_rel.as_str()));
                // ① 链接式契约引用:`back.*` 模块解析映射到后端项目根
                // (lib.rs resolve_module_path 的 EXTERNAL_BACK_ROOT 钩子),
                // 前端无需本地 back/ 目录,零复制。
                if backend_dir.join("api.at").is_file() {
                    auto_lang_core::set_external_back_root(backend_dir.clone());
                    println!(
                        "  {} external backend linked: back.* -> {}",
                        "✓".bright_green(),
                        backend_dir.display()
                    );
                } else {
                    eprintln!(
                        "  {} external backend: no api.at in {}",
                        "⚠".bright_yellow(),
                        backend_dir.display()
                    );
                }
                // ② cdylib 装载(名字取自后端 pac.at 的 name;debug 优先)
                match load_external_backend(&backend_dir) {
                    Ok(lib) => {
                        println!(
                            "  {} external backend loaded: {}",
                            "✓".bright_green(),
                            backend_dir.display()
                        );
                        _backend_keepalive = Some(lib);
                    }
                    Err(e) => {
                        stop_api_server(&mut _api_child);
                        return Err(format!("external backend load failed: {e}").into());
                    }
                }
            }
            Ok(None) => {}
            Err(e) => eprintln!("  {} pac.at parse failed: {}", "⚠".bright_yellow(), e),
        }
    }

    // PLAN-625: ui-gallery VM 臂 registry 刷新——app.at 的 `use registry:`
    // 消费 src/front/registry.at(生成产物),须在编译前落盘。vue 臂每次
    // run/build 都刷 generate_gallery_host(run_vue_project 先例),VM 臂
    // 此前不产任何 registry。非 gallery 项目零行为变化。
    if project_dir.file_name().and_then(|n| n.to_str()) == Some("ui-gallery")
        || crate::vue::gallery_mode()
    {
        // PLAN-658 T-03: 多后端 proxy 先于发射启动（发射器 /api/ 字面量
        // 前缀化消费线程局部根，T-02）；失败降级不阻断画廊。
        crate::run_vue::start_gallery_back_proxy(project_dir);
        let n = crate::vue::refresh_gallery_registry(project_dir)?;
        println!(
            "  {} Gallery registry: src/front/registry.at ({} demos)",
            "✓".bright_green(),
            n
        );
    }

    // PLAN-815 T-06 R1（F-815-T06-R1-01）：前端源目录消费请求声明
    // （app("front") 节点形式优先、front: prop 次之——镜像
    // parse_workspace_path；无声明=默认 src/front 正控保持）。
    let front_dir = source
        .declared_front_dir(project_dir)
        .unwrap_or_else(|| project_dir.join("src").join("front"));
    let entry = front_dir.join("app.at");
    if !entry.exists() {
        stop_api_server(&mut _api_child);
        return Err(format!("Frontend entry not found: {}", entry.display()).into());
    }
    if proc_host::cancelled() {
        stop_api_server(&mut _api_child);
        return Ok(FlowEnd::Cancelled);
    }

    println!(
        "{}",
        if split_mode {
            format!(
                "Running VM interpreter UI (backend: {}, split over HTTP)",
                backend_impl
            )
            .bright_cyan()
        } else {
            "Running VM interpreter UI (backend: vm, merged)".bright_cyan()
        }
    );

    // Change CWD to the (request-declared) front dir so `use` imports
    // resolve correctly.
    let original_dir = std::env::current_dir().ok();
    let _ = std::env::set_current_dir(&front_dir);

    // Plan 458: apply the `auto run`-injected theme + accent presets
    // (AUTO_UI_THEME / AUTO_UI_ACCENT, set by the run handler from
    // --theme/--accent > pac.at) to the backend-neutral theme thread-locals
    // BEFORE the first frame. Values are re-validated here because this fn
    // is also reachable without the run handler (split-mode hosts). Apps
    // that declare `dark_mode`/`accent_color` state vars override these
    // per-frame via the renderer sync (renderer.rs read_state).
    if let Ok(t) = std::env::var("AUTO_UI_THEME") {
        if auto_ui_compiler::style::theme::THEME_PREFS.contains(&t.as_str()) {
            auto_ui_compiler::style::theme::set_dark_mode(t == "dark");
        }
    }
    if let Ok(a) = std::env::var("AUTO_UI_ACCENT") {
        if auto_ui_compiler::style::theme::ACCENT_PRESETS.contains(&a.as_str()) {
            auto_ui_compiler::style::theme::set_accent_name(&a);
        }
    }

    // PLAN-609 T-A1: pac.at `theme: {}` 声明色板激活（VM 腿 boot,与 vue 腿
    // T-06 双端同源）。在 env 种子**之后**执行：声明只换 ACTIVE_THEME 色板
    // 槽,不清写 DARK_MODE/ACCENT_NAME——优先级链
    // CLI env > os-config/宿主 > pac.at 声明 > 内置缺省 语义成文。
    // PLAN-609 T-A1: pac.at `theme: {}` 声明色板激活（VM 腿 boot,与 vue 腿
    // T-06 双端同源）。在 env 种子**之后**执行：声明只换 ACTIVE_THEME 色板
    // 槽,不清写 DARK_MODE/ACCENT_NAME——优先级链
    // CLI env > os-config/宿主 > pac.at 声明 > 内置缺省 语义成文。
    // T-05 R4（F-815-T05-R4-01）：托管形态的主题经会话子进程 env 激活
    // （见上方 session_env 装配）——父进程不再全局应用，避免跨请求残留。
    if proc_host::current().is_none() {
        if let Some(name) = source.apply_theme_decl(project_dir) {
            println!(
                "  {} theme declaration activated: {}",
                "✓".bright_green(),
                name
            );
        }
    }

    // PLAN-080 §10-9 足迹桩（CLI 调试遗留）：仅在未托管（CLI 直跑）形态
    // 安装——工具宿主进程不应被全局 panic 钩子/心跳线程改变生命周期语义。
    if proc_host::current().is_none() && std::env::var_os("AUTO_UI_TOOL_VM_CHILD").is_none() {
        std::panic::set_hook(Box::new(|info| {
            eprintln!(
                "[X9-PANIC] {info}
backtrace:
{}",
                std::backtrace::Backtrace::force_capture()
            );
        }));
        std::thread::spawn(|| loop {
            std::thread::sleep(std::time::Duration::from_secs(30));
            eprintln!(
                "[X9-ALIVE] {}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            );
        });
    }
    step(2, "vm interpreter");
    eprintln!("[X9] entering vm interpreter");
    let result = run_entry_file(entry.to_str().unwrap_or("src/front/app.at"));
    eprintln!("[X9] vm interpreter returned ok={}", result.is_ok());

    // Restore original CWD
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(dir);
    }

    stop_api_server(&mut _api_child);

    if proc_host::cancelled() {
        return Ok(FlowEnd::Cancelled);
    }
    match result {
        Ok(_) => Ok(FlowEnd::Completed),
        Err(e) => Err(format!("VM UI error: {}", e).into()),
    }
}
