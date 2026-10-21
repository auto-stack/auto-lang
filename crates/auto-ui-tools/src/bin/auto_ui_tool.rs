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
fn run_internal_vm_session(entry: &str) -> i32 {
    use std::io::BufRead;
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
    match auto_ui_tools::run_vm::run_entry_file(entry) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("vm session failed: {e}");
            1
        }
    }
}
