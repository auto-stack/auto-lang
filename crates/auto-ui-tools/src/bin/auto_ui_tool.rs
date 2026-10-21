//! `auto-ui-tool` — the real tool entry (PLAN-815 T-12/T-05). Serves the
//! light I-TOOL protocol on framed stdio until EOF; logs to stderr only.
//!
//! T-05: before serving, the process stdout handle is routed onto stderr
//! and the control writer keeps the original handle — every `println!` in
//! the migrated run paths and every inherited-stdio child lands on the log
//! stream, so the control stream stays framed JSON by construction.

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
