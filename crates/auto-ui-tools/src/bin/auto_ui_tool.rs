//! `auto-ui-tool` — the real tool entry (PLAN-815 T-12/T-05). Serves the
//! light I-TOOL protocol on framed stdio until EOF; logs to stderr only.
//!
//! T-05: before serving, the process stdout handle is routed onto stderr
//! and the control writer keeps the original handle — every `println!` in
//! the migrated run paths and every inherited-stdio child lands on the log
//! stream, so the control stream stays framed JSON by construction.
//!
//! T-05 R2（R2-01）：`--internal-vm-exec <entry>` 是**宿主内部**执行臂
//! （不出现在 --help，不是客户端面）：宿主把每个 VM 会话隔离到本二进制
//! 的一个子进程里跑（iced/winit 每进程单事件循环约束的进程级隔离），
//! 子进程在自己的 OS 主线程运行动态 UI，stdin 由宿主的 `Input` 通道
//! 供给——行 `q` 经 runtime 的 `request_host_exit` 干净收口会话。

use auto_tool_protocol::Budgets;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => {}
        [single] if single == "--version" => {
            println!("{} {}", auto_ui_tools::TOOL_NAME, env!("CARGO_PKG_VERSION"));
            return;
        }
        [single] if single == "--help" || single == "-h" => {
            println!(
                "{} {} — UI engineering tool host (M815-A: describe/validate_project; T-05: generate_ui/run_ui)\n\
                 Usage: auto-ui-tool            serve framed I-TOOL protocol on stdio\n\
                        auto-ui-tool --version  print tool version",
                auto_ui_tools::TOOL_NAME,
                env!("CARGO_PKG_VERSION")
            );
            return;
        }
        // PLAN-815 T-05 R2（R2-01）：内部 VM 会话执行臂——仅由宿主自身
        // 经 `proc_host` spawn 消费，不进 --help/客户端文档。
        [flag, entry] if flag == "--internal-vm-exec" => {
            std::process::exit(run_internal_vm_session(entry));
        }
        _ => {
            eprintln!("unknown arguments: {args:?} (see --help)");
            std::process::exit(2);
        }
    }
    // Control/log handle split BEFORE the first stdout write (unsafe by
    // contract: exactly once, at bin start).
    let control = unsafe { auto_ui_tools::stdio_split::split_control_from_stdout() };
    let code = auto_ui_tools::run_tool(
        std::io::stdin(),
        std::io::BufWriter::new(control),
        std::io::stderr(),
        Budgets::contract().max_frame_bytes,
    );
    if code != 0 {
        eprintln!("auto-ui-tool exiting with code {code}");
    }
    std::process::exit(code);
}

/// 内部 VM 会话执行臂：stdin 上的行 `q` 请求会话退出（真实交互通道，
/// 与 dev-server 的 `q` 语义同族）；入口在本（主）线程运行动态 UI。
///
/// T-05 R3（R3-01）：执行进程自行装配会话上下文——父进程把**单源请求
/// 衍生**的外部后端根/主题名经 env 传入（本进程不重读主 pac，无第二真
/// 源）；`AUTO_SOURCE_DIRS`/cwd 等随进程继承。keepalive 持有到会话
/// 结束（cdylib 注册表的宿主桥身份）。
///
/// T-05 R4（F-815-T05-R4-02）：请求声明的外部后端是**会话的前置依赖**
/// ——装载失败（缺 cdylib/ABI 不匹配）不再是告警后照跑，而是写
/// `VM_SESSION_FATAL: <reason>` 到 stderr（宿主转发捕获）并以专用退出码
/// 3 终止；宿主把它映射为具名 failed，不启动依赖它的会话。
fn run_internal_vm_session(entry: &str) -> i32 {
    use std::io::BufRead;
    /// Dedicated exit code: required external backend failed to load
    /// (distinct from 1 = entry execution failure).
    const EXIT_VM_BACKEND_LOAD: i32 = 3;
    // R3-02：退出请求窗口在本进程的最早边界开启——此后任何时刻的 q
    //（含组件构建/初始化期间）都不会被快照吸收。
    auto_ui_runtime::ui::iced::renderer::host_exit_window_begin();
    // 标记内部子形态（X9 足迹桩只属 CLI 直跑）。
    // SAFETY: edition 2021; set before any reader.
    unsafe {
        std::env::set_var("AUTO_UI_TOOL_VM_CHILD", "1");
    }
    // 上下文装配（请求衍生 env）。
    let mut backend_keepalive: Option<auto_lang_core::vm::backend_abi::LoadedBackend> = None;
    if let Ok(back_root) = std::env::var("AUTO_UI_TOOL_VM_BACK_ROOT") {
        match auto_ui_tools::run_vm::prepare_external_backend_from_root(std::path::Path::new(
            &back_root,
        )) {
            Ok(loaded) => {
                eprintln!(
                    "  {} external backend loaded (vm child): {}",
                    "✓", back_root
                );
                backend_keepalive = Some(loaded);
            }
            Err(e) => {
                eprintln!(
                    "{}external backend load failed (vm child): {e}",
                    auto_ui_tools::proc_host::VM_SESSION_FATAL_PREFIX
                );
                return EXIT_VM_BACKEND_LOAD;
            }
        }
    }
    if let Ok(json) = std::env::var("AUTO_UI_TOOL_VM_THEME_DECL") {
        // 请求衍生的 theme{} lite 声明（单源数据）：与父进程同链
        // compose 激活（PacSource::apply_theme_decl 请求臂的同构拷贝）。
        match serde_json::from_str::<serde_json::Value>(&json)
            .map(|v| parse_theme_lite(&v))
            .unwrap_or(None)
        {
            Some(lite) => {
                let decl = auto_lang_core::design_tokens::decl::ThemeDecl {
                    name: lite.name,
                    extends: lite.extends,
                    mode: lite.mode,
                    colors: lite.colors,
                };
                match auto_lang_core::design_tokens::decl::compose(
                    &decl,
                    &std::collections::BTreeMap::new(),
                ) {
                    Ok(theme) => {
                        auto_ui_compiler::style::theme::set_theme_composed(std::sync::Arc::new(
                            theme,
                        ));
                    }
                    Err(e) => {
                        eprintln!("  ⚠ vm child theme compose failed: {e} — keeping default");
                    }
                }
            }
            None => eprintln!("  ⚠ vm child theme decl unparsable — keeping default"),
        }
    }
    std::thread::Builder::new()
        .name("vm-session-stdin".into())
        .spawn(move || {
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                let Ok(line) = line else { break };
                let trimmed = line.trim();
                if trimmed == "q" || trimmed == "quit" || trimmed == "exit" {
                    auto_ui_runtime::ui::iced::renderer::request_host_exit();
                }
            }
        })
        .expect("spawn vm stdin watcher");
    let result = auto_ui_tools::run_vm::run_entry_file(entry);
    drop(backend_keepalive);
    match result {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("vm session failed: {e}");
            1
        }
    }
}

/// 手写解析 theme{} lite 声明 JSON（父进程 serde_json::json! 组装的镜像；
/// project-model 无 serde 派生）。
fn parse_theme_lite(
    value: &serde_json::Value,
) -> Option<auto_project_model::generation_request::ThemeDeclLite> {
    let colors = value
        .get("colors")?
        .as_array()?
        .iter()
        .map(|pair| {
            let arr = pair.as_array()?;
            Some((
                arr.first()?.as_str()?.to_string(),
                arr.get(1)?.as_str()?.to_string(),
            ))
        })
        .collect::<Option<Vec<(String, String)>>>()?;
    Some(auto_project_model::generation_request::ThemeDeclLite {
        name: value.get("name").and_then(|v| v.as_str()).map(String::from),
        extends: value
            .get("extends")
            .and_then(|v| v.as_str())
            .map(String::from),
        mode: value.get("mode").and_then(|v| v.as_str()).map(String::from),
        colors,
    })
}
