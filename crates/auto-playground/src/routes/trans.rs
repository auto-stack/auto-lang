use axum::Json;
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use auto_lang::trans::SourceMapEntry;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct TransRequest {
    pub source: String,
    pub target: String, // "rust" | "c" | "python" | "javascript" | "typescript"
    pub project_dir: Option<String>,
}

#[derive(Serialize)]
pub struct TransFile {
    pub path: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct TransResponse {
    pub target: String,
    pub files: Vec<TransFile>,
    pub source_map: Vec<SourceMapEntry>,
}

pub async fn trans_handler(
    Json(req): Json<TransRequest>,
) -> Result<Json<TransResponse>, AppError> {
    let target = req.target.clone();
    let source = req.source.clone();
    let project_dir = req.project_dir.clone();

    let (files, source_map) = tokio::task::spawn_blocking(move || match project_dir {
        Some(dir) => match target.as_str() {
            "rust" => transpile_rust_project(&source, &dir),
            "c" => transpile_project_merged(&source, &dir, "c", transpile_c),
            "python" => transpile_project_merged(&source, &dir, "py", transpile_python),
            "javascript" => transpile_project_merged(&source, &dir, "js", transpile_javascript),
            "typescript" => transpile_project_merged(&source, &dir, "ts", transpile_typescript),
            _ => Err(AppError::Internal(format!(
                "Project examples do not support target: {target}"
            ))),
        },
        None => {
            let (code, source_map) = match target.as_str() {
                "rust" => transpile_rust(&source),
                "c" => transpile_c(&source),
                "python" => transpile_python(&source),
                "javascript" => transpile_javascript(&source),
                "typescript" => transpile_typescript(&source),
                "abt" | "bytecode" => transpile_abt(&source),
                _ => return Err(AppError::Internal(format!("Unknown target: {target}"))),
            }?;
            let path = format!("playground.{}", target_to_extension(&target));
            Ok((vec![TransFile { path, code }], source_map))
        }
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(TransResponse {
        target: req.target,
        files,
        source_map,
    }))
}

fn target_to_extension(target: &str) -> &str {
    match target {
        "rust" => "rs",
        "c" => "c",
        "python" => "py",
        "javascript" => "js",
        "typescript" => "ts",
        _ => "txt",
    }
}

fn project_examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("examples/playground-demo"))
        .unwrap_or_else(|| PathBuf::from("examples/playground-demo"))
}

/// Build a temporary copy of the project directory, overwriting `main.at` with
/// the edited source. Returns the temp directory path.
fn prepare_project_temp_dir(source: &str, project_dir: &str) -> Result<PathBuf, AppError> {
    let original_dir = project_examples_dir().join(project_dir);
    if !original_dir.is_dir() {
        return Err(AppError::Internal(format!(
            "Project directory not found: {}",
            original_dir.display()
        )));
    }

    let temp_id = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let temp_dir = std::env::temp_dir().join(format!("auto-playground-project-{}", temp_id));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| AppError::Internal(format!("Failed to create temp dir: {e}")))?;

    for entry in std::fs::read_dir(&original_dir)
        .map_err(|e| AppError::Internal(format!("Failed to read project dir: {e}")))?
    {
        let entry = entry.map_err(|e| AppError::Internal(format!("Failed to read dir entry: {e}")))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "at") {
            let dest = temp_dir.join(path.file_name().unwrap_or_default());
            std::fs::copy(&path, &dest)
                .map_err(|e| AppError::Internal(format!("Failed to copy {}: {e}", path.display())))?;
        }
    }

    let main_path = temp_dir.join("main.at");
    std::fs::write(&main_path, source)
        .map_err(|e| AppError::Internal(format!("Failed to write main.at: {e}")))?;

    Ok(temp_dir)
}

pub fn transpile_rust_project(
    source: &str,
    project_dir: &str,
) -> Result<(Vec<TransFile>, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::rust::transpile_rust_project as auto_transpile_rust_project;

    let temp_dir = prepare_project_temp_dir(source, project_dir)?;
    let main_path = temp_dir.join("main.at");

    let result = auto_transpile_rust_project(&main_path.to_string_lossy())
        .map_err(|e| AppError::CompileError(e.to_string()))?;

    // Best-effort cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Order: Cargo.toml first, then entry file (main.rs), then remaining .rs files.
    let mut files: Vec<TransFile> = Vec::new();
    if let Some(code) = result.get("Cargo.toml") {
        files.push(TransFile {
            path: "Cargo.toml".into(),
            code: String::from_utf8_lossy(code).to_string(),
        });
    }
    let mut remaining: Vec<(String, Vec<u8>)> = result
        .into_iter()
        .filter(|(k, _)| k != "Cargo.toml")
        .collect();
    remaining.sort_by(|a, b| {
        // main.rs first, then alphabetical
        if a.0 == "main.rs" {
            std::cmp::Ordering::Less
        } else if b.0 == "main.rs" {
            std::cmp::Ordering::Greater
        } else {
            a.0.cmp(&b.0)
        }
    });
    for (path, code) in remaining {
        files.push(TransFile {
            path,
            code: String::from_utf8_lossy(&code).to_string(),
        });
    }

    Ok((files, Vec::new()))
}

/// For non-Rust targets we merge all project `.at` files into a single source
/// string (topological order, stripping project-local `use` lines) and then
/// run the ordinary single-file transpiler. This keeps the change small while
/// making C / Python / TS / JS work for project examples.
pub fn transpile_project_merged(
    source: &str,
    project_dir: &str,
    extension: &str,
    transpile: fn(&str) -> Result<(String, Vec<SourceMapEntry>), AppError>,
) -> Result<(Vec<TransFile>, Vec<SourceMapEntry>), AppError> {
    let merged = merge_project_source(source, project_dir)?;
    let (code, source_map) = transpile(&merged)?;
    Ok((vec![TransFile {
        path: format!("playground.{extension}"),
        code,
    }], source_map))
}

/// Merge project `.at` files into one source string.
///
/// Project-local `use <stem>` lines are stripped; files are concatenated in
/// topological order so dependencies are defined before their dependents.
fn merge_project_source(source: &str, project_dir: &str) -> Result<String, AppError> {
    let original_dir = project_examples_dir().join(project_dir);
    if !original_dir.is_dir() {
        return Err(AppError::Internal(format!(
            "Project directory not found: {}",
            original_dir.display()
        )));
    }

    // Read all .at files into a map keyed by stem; main.at uses the edited source.
    let mut sources: HashMap<String, String> = HashMap::new();
    for entry in std::fs::read_dir(&original_dir)
        .map_err(|e| AppError::Internal(format!("Failed to read project dir: {e}")))?
    {
        let entry = entry.map_err(|e| AppError::Internal(format!("Failed to read dir entry: {e}")))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "at") {
            let stem = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let content = if stem == "main" {
                source.to_string()
            } else {
                std::fs::read_to_string(&path)
                    .map_err(|e| AppError::Internal(format!("Failed to read {}: {e}", path.display())))?
            };
            sources.insert(stem, content);
        }
    }

    // Detect project-local dependencies via `use <stem>` lines.
    let re = regex::Regex::new(r"(?m)^\s*use\s+(\w+)(\s*::.*)?\s*$").unwrap();
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    for (stem, content) in &sources {
        let mut d = Vec::new();
        for cap in re.captures_iter(content) {
            let used = cap[1].to_string();
            if used != *stem && sources.contains_key(&used) {
                d.push(used);
            }
        }
        deps.insert(stem.clone(), d);
    }

    // Topological sort (Kahn).
    let mut in_degree: HashMap<String, usize> = sources.keys().map(|s| (s.clone(), 0)).collect();
    for (dependent, deps_list) in &deps {
        for _dep in deps_list {
            *in_degree.get_mut(dependent).unwrap() += 1;
        }
    }
    let mut queue: Vec<String> = in_degree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(s, _)| s.clone())
        .collect();
    queue.sort();
    let mut order: Vec<String> = Vec::new();
    while let Some(stem) = queue.pop() {
        order.push(stem.clone());
        // Find all modules that depend on `stem`
        for (dependent, deps_list) in &deps {
            if deps_list.contains(&stem) {
                let deg = in_degree.get_mut(dependent).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(dependent.clone());
                    queue.sort();
                }
            }
        }
    }
    if order.len() != sources.len() {
        return Err(AppError::Internal(
            "Circular dependency detected in project modules".into(),
        ));
    }

    // Ensure main.at is always last (entry point), regardless of dependencies.
    order.retain(|s| s != "main");
    order.push("main".into());

    // Build merged source, stripping project-local `use` lines.
    let mut merged = String::new();
    for stem in order {
        let content = sources.get(&stem).unwrap();
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(cap) = re.captures(trimmed) {
                let used = cap[1].to_string();
                if used != stem && sources.contains_key(&used) {
                    continue;
                }
            }
            merged.push_str(line);
            merged.push('\n');
        }
        merged.push('\n');
    }

    Ok(merged)
}

pub fn transpile_rust(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::rust::transpile_rust as auto_transpile_rust;
    use auto_lang::trans::Sink;

    let mut sink: Sink = auto_transpile_rust("playground", source)
        .map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();
    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((String::from_utf8_lossy(output).to_string(), source_map))
}

pub fn transpile_abt(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    let (vm, _, _, _) = auto_lang::create_vm_from_source(source)
        .map_err(|e| AppError::CompileError(e.to_string()))?;

    let strings = vm.strings.read().map_err(|e| AppError::Internal(e.to_string()))?;
    let abt = auto_lang::vm::abt::disasm::disassemble_flash(&vm.flash, Some(&strings));
    Ok((abt.to_string(), Vec::new()))
}

pub fn transpile_c(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::c::transpile_c as auto_transpile_c;

    let mut sink = auto_transpile_c("playground", source)
        .map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();

    // For single-file playground output, inline header content directly
    // instead of generating a separate .h file with #include "playground.h"
    let mut output = Vec::new();
    if !sink.header.is_empty() {
        output.append(&mut sink.header);
        output.write(b"\n").unwrap();
    }
    output.append(&mut sink.body);

    Ok((String::from_utf8_lossy(&output).to_string(), source_map))
}

pub fn transpile_python(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::{Sink, Trans};
    use auto_lang::trans::python::PythonTrans;
    use auto_lang::Parser;

    let mut parser = Parser::from(source);
    let ast = parser.parse().map_err(|e| AppError::CompileError(e.to_string()))?;
    let mut sink = Sink::new("playground".into());
    let mut trans = PythonTrans::new("playground".into());
    trans.trans(ast, &mut sink).map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();
    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((String::from_utf8_lossy(output).to_string(), source_map))
}

pub fn transpile_javascript(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::{Sink, Trans};
    use auto_lang::trans::javascript::JavaScriptTrans;
    use auto_lang::Parser;

    let mut parser = Parser::from(source);
    let ast = parser.parse().map_err(|e| AppError::CompileError(e.to_string()))?;
    let mut sink = Sink::new("playground".into());
    let mut trans = JavaScriptTrans::new("playground".into());
    trans.trans(ast, &mut sink).map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();
    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((String::from_utf8_lossy(output).to_string(), source_map))
}

pub fn transpile_typescript(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::{Sink, Trans};
    use auto_lang::trans::typescript::TypeScriptTrans;
    use auto_lang::Parser;

    let mut parser = Parser::from(source);
    let ast = parser.parse().map_err(|e| AppError::CompileError(e.to_string()))?;
    let mut sink = Sink::new("playground".into());
    let mut trans = TypeScriptTrans::new("playground".into());
    trans.trans(ast, &mut sink).map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();
    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((String::from_utf8_lossy(output).to_string(), source_map))
}
