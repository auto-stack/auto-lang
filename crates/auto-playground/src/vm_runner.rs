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

pub fn run_source_with_input(
    source: &str,
    deadline: Option<std::time::Instant>,
    prepend_lib: bool,
    input: Option<String>,
) -> RunResult {
    let start = Instant::now();

    // PLAN-752: bootstrap notes are bare fragments that reference symbols
    // from the AAVM v1 lib — concat it like the golden is_bootstrap branch.
    let owned;
    let source = if prepend_lib {
        match auto_lang::bootstrap_lib_source() {
            Some(lib) => {
                owned = format!(
                    "{}
{}",
                    lib, source
                );
                &owned
            }
            None => source,
        }
    } else {
        source
    };

    let result = if input.is_some() {
        match crate::project::prepare_files_temp_dir_with_entry(source, &[], Some("main.at")) {
            Ok((dir, entry_path)) => {
                let run = auto_lang::run_with_capture_and_path_and_bytecode_with_input(
                    source,
                    &entry_path.to_string_lossy(),
                    deadline,
                    input,
                );
                let _ = std::fs::remove_dir_all(&dir);
                run.map_err(|error| error.to_string())
            }
            Err(error) => Err(error.to_string()),
        }
    } else {
        auto_lang::run_with_capture_and_bytecode_with_deadline(source, deadline)
            .map_err(|error| error.to_string())
    };
    let (result, stdout, bytecode, meta) = match result {
        Ok((res, out, bc, meta)) => (res, out, disasm_to_json(bc), meta_to_json(meta)),
        Err(error) => (String::new(), format!("Error: {error}"), Vec::new(), None),
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

/// Run a files-only project (no server-side `project_dir` — manifest notes,
/// Plan 582): `files` are materialized at their relative paths in a temp dir,
/// `source` is written at `entry`, and the entry path drives module resolution
/// (`use auto.<lib>` resolves via the entry's parent dirs).
pub fn run_files_project_with_input(
    source: &str,
    files: Vec<crate::project::ProjectFile>,
    entry: Option<&str>,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> RunResult {
    let start = Instant::now();

    let outcome = crate::project::prepare_files_temp_dir_with_entry(source, &files, entry)
        .and_then(|(dir, entry_path)| {
            let run = auto_lang::run_with_capture_and_path_and_bytecode_with_input(
                source,
                &entry_path.to_string_lossy(),
                deadline,
                input,
            );
            let _ = std::fs::remove_dir_all(&dir);
            Ok::<_, crate::error::AppError>(run)
        });

    let (result, stdout, bytecode, meta) = match outcome {
        Ok(Ok((res, out, bc, meta))) => (res, out, disasm_to_json(bc), meta_to_json(meta)),
        Ok(Err(e)) => (String::new(), format!("Error: {}", e), Vec::new(), None),
        Err(e) => (String::new(), format!("Error: {}", e), Vec::new(), None),
    };

    RunResult {
        stdout,
        result,
        time_ms: start.elapsed().as_millis() as u64,
        bytecode,
        meta,
    }
}

/// Run source that belongs to a project example. `project_dir` is relative to
/// `examples/playground-demo`; the entry file is `main.at` inside that directory.
/// When `files` is provided, the project is first materialized in a temp
/// directory with those edited contents overlaid, so edits to non-entry files
/// take effect too.
pub fn run_project_source_with_input(
    source: &str,
    project_dir: &str,
    files: Option<Vec<crate::project::ProjectFile>>,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> RunResult {
    let start = Instant::now();

    // Hosted runs use a fresh copy even when the editor has no overlays, so
    // relative file IO cannot read or mutate the server's example checkout.
    let needs_temp = input.is_some() || files.is_some();
    let temp_dir = match needs_temp {
        true => {
            match crate::project::prepare_project_temp_dir(source, project_dir, files.as_deref()) {
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
            }
        }
        false => None,
    };
    let examples_dir = crate::project::project_examples_dir();
    let entry_path = match &temp_dir {
        Some(dir) => dir.join("main.at"),
        None => examples_dir.join(project_dir).join("main.at"),
    };

    let (result, stdout, bytecode, meta) =
        match auto_lang::run_with_capture_and_path_and_bytecode_with_input(
            source,
            &entry_path.to_string_lossy(),
            deadline,
            input,
        ) {
            Ok((res, out, bc, meta)) => (res, out, disasm_to_json(bc), meta_to_json(meta)),
            Err(e) => (String::new(), format!("Error: {}", e), Vec::new(), None),
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
