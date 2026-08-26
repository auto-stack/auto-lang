use std::time::Instant;

pub struct RunResult {
    pub stdout: String,
    pub result: String,
    pub time_ms: u64,
    pub bytecode: Vec<serde_json::Value>,
    /// Symbol tables (strings/functions/natives) for disassembly tooltips
    pub meta: Option<serde_json::Value>,
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

fn meta_to_json(meta: auto_lang::vm::disasm::BytecodeMeta) -> Option<serde_json::Value> {
    serde_json::to_value(meta).ok()
}

pub fn run_source(source: &str) -> RunResult {
    let start = Instant::now();

    let (result, stdout, bytecode, meta) = match auto_lang::run_with_capture_and_bytecode_with_meta(source) {
        Ok((res, out, bc, meta)) => (res, out, disasm_to_json(bc), meta_to_json(meta)),
        Err(e) => (
            String::new(),
            format!("Error: {}", e),
            Vec::new(),
            None,
        ),
    };

    let time_ms = start.elapsed().as_millis() as u64;

    RunResult {
        stdout,
        result,
        time_ms,
        bytecode,
        meta,
    }
}

/// Run source that belongs to a project example. `project_dir` is relative to
/// `examples/playground-demo`; the entry file is `main.at` inside that directory.
/// When `files` is provided, the project is first materialized in a temp
/// directory with those edited contents overlaid, so edits to non-entry files
/// take effect too.
pub fn run_project_source(
    source: &str,
    project_dir: &str,
    files: Option<Vec<crate::project::ProjectFile>>,
) -> RunResult {
    let start = Instant::now();

    let examples_dir = crate::project::project_examples_dir();

    // With edited files, run from a temp copy; otherwise run in place.
    let temp_dir = match &files {
        Some(f) => match crate::project::prepare_project_temp_dir(source, project_dir, Some(f)) {
            Ok(dir) => Some(dir),
            Err(e) => {
                return RunResult {
                    stdout: format!("Error: {}", e),
                    result: String::new(),
                    time_ms: start.elapsed().as_millis() as u64,
                    bytecode: Vec::new(),
                    meta: None,
                }
            }
        },
        None => None,
    };
    let entry_path = match &temp_dir {
        Some(dir) => dir.join("main.at"),
        None => examples_dir.join(project_dir).join("main.at"),
    };

    let (result, stdout, bytecode, meta) = match auto_lang::run_with_capture_and_path_and_bytecode_with_meta(
        source,
        &entry_path.to_string_lossy(),
    ) {
        Ok((res, out, bc, meta)) => (res, out, disasm_to_json(bc), meta_to_json(meta)),
        Err(e) => (
            String::new(),
            format!("Error: {}", e),
            Vec::new(),
            None,
        ),
    };

    if let Some(dir) = temp_dir {
        let _ = std::fs::remove_dir_all(&dir);
    }

    let time_ms = start.elapsed().as_millis() as u64;

    RunResult {
        stdout,
        result,
        time_ms,
        bytecode,
        meta,
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
        meta: None,
    }
}
