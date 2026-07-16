use std::time::Instant;
use std::path::PathBuf;

pub struct RunResult {
    pub stdout: String,
    pub result: String,
    pub time_ms: u64,
    pub bytecode: Vec<serde_json::Value>,
}

fn disasm_to_json(lines: Vec<auto_lang::vm::disasm::DisasmLine>) -> Vec<serde_json::Value> {
    lines
        .into_iter()
        .map(|l| {
            serde_json::json!({
                "offset": l.offset,
                "mnemonic": l.mnemonic,
                "operands": l.operands,
                "line": l.line,
            })
        })
        .collect()
}

pub fn run_source(source: &str) -> RunResult {
    let start = Instant::now();

    let (result, stdout, bytecode) = match auto_lang::run_with_capture_and_bytecode(source) {
        Ok((res, out, bc)) => (res, out, disasm_to_json(bc)),
        Err(e) => (
            String::new(),
            format!("Error: {}", e),
            Vec::new(),
        ),
    };

    let time_ms = start.elapsed().as_millis() as u64;

    RunResult {
        stdout,
        result,
        time_ms,
        bytecode,
    }
}

/// Run source that belongs to a project example. `project_dir` is relative to
/// `examples/playground-demo`; the entry file is `main.at` inside that directory.
pub fn run_project_source(source: &str, project_dir: &str) -> RunResult {
    let start = Instant::now();

    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("examples/playground-demo"))
        .unwrap_or_else(|| PathBuf::from("examples/playground-demo"));
    let entry_path = examples_dir.join(project_dir).join("main.at");

    let (result, stdout, bytecode) = match auto_lang::run_with_capture_and_path_and_bytecode(
        source,
        &entry_path.to_string_lossy(),
    ) {
        Ok((res, out, bc)) => (res, out, disasm_to_json(bc)),
        Err(e) => (
            String::new(),
            format!("Error: {}", e),
            Vec::new(),
        ),
    };

    let time_ms = start.elapsed().as_millis() as u64;

    RunResult {
        stdout,
        result,
        time_ms,
        bytecode,
    }
}

pub fn run_abt(abt: &str) -> RunResult {
    let start = Instant::now();

    let runtime = auto_lang::get_global_runtime();
    let stdout = match runtime.block_on(auto_lang::run_abt(abt)) {
        Ok(out) => out,
        Err(e) => format!("Error: {}", e),
    };

    let time_ms = start.elapsed().as_millis() as u64;

    RunResult {
        stdout,
        result: String::new(),
        time_ms,
        bytecode: Vec::new(),
    }
}
