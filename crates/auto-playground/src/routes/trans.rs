use axum::Json;
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use crate::project::{prepare_project_temp_dir, project_examples_dir, ProjectFile};
use auto_lang::trans::SourceMapEntry;
use std::collections::HashMap;
use std::io::Write;

#[derive(Deserialize)]
pub struct TransRequest {
    pub source: String,
    pub target: String, // "rust" | "c" | "python" | "javascript" | "typescript"
    pub project_dir: Option<String>,
    pub files: Option<Vec<ProjectFile>>,
}

#[derive(Serialize)]
pub struct TransFile {
    pub path: String,
    pub code: String,
    pub source_map: Option<Vec<SourceMapEntry>>,
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
    let req_files = req.files.clone();

    let (files, source_map) = tokio::task::spawn_blocking(move || match project_dir {
        Some(dir) => match target.as_str() {
            "rust" => transpile_rust_project(&source, &dir, req_files.as_deref()),
            "c" => transpile_project_merged(&source, &dir, "c", transpile_c, req_files.as_deref()),
            "python" => transpile_project_merged(&source, &dir, "py", transpile_python, req_files.as_deref()),
            "javascript" => transpile_project_merged(&source, &dir, "js", transpile_javascript, req_files.as_deref()),
            "typescript" => transpile_project_merged(&source, &dir, "ts", transpile_typescript, req_files.as_deref()),
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
            let file_source_map = source_map.clone();
            Ok((vec![TransFile { path, code, source_map: Some(file_source_map) }], source_map))
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

pub fn transpile_rust_project(
    source: &str,
    project_dir: &str,
    files: Option<&[ProjectFile]>,
) -> Result<(Vec<TransFile>, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::rust::transpile_rust_project as auto_transpile_rust_project;

    let temp_dir = prepare_project_temp_dir(source, project_dir, files)?;
    let main_path = temp_dir.join("main.at");

    let result = auto_transpile_rust_project(&main_path.to_string_lossy())
        .map_err(|e| AppError::CompileError(e.to_string()))?;

    // Best-effort cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Order: Cargo.toml first, then entry file (main.rs), then remaining .rs files.
    let mut files: Vec<TransFile> = Vec::new();
    if let Some((code, source_map)) = result.get("Cargo.toml") {
        files.push(TransFile {
            path: "Cargo.toml".into(),
            code: String::from_utf8_lossy(code).to_string(),
            source_map: Some(source_map.clone()),
        });
    }
    let mut remaining: Vec<(String, Vec<u8>, Vec<SourceMapEntry>)> = result
        .into_iter()
        .filter(|(k, _)| k != "Cargo.toml")
        .map(|(k, (code, source_map))| (k, code, source_map))
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
    for (path, code, source_map) in remaining {
        files.push(TransFile {
            path,
            code: String::from_utf8_lossy(&code).to_string(),
            source_map: Some(source_map),
        });
    }

    // Aggregate a best-effort project-level source map for backwards compatibility.
    let source_map: Vec<SourceMapEntry> = files
        .iter()
        .filter_map(|f| f.source_map.clone())
        .flat_map(|sm| sm.into_iter())
        .collect();
    Ok((files, source_map))
}

/// For non-Rust targets we merge all project `.at` files into a single source
/// string (topological order, stripping project-local `use` lines) and then
/// run the ordinary single-file transpiler. This keeps the change small while
/// making C / Python / TS / JS work for project examples.
/// Maps a line in the merged source back to the original project file and line.
#[derive(Debug, Clone)]
struct LineOrigin {
    file: String,
    line: usize,
}

pub fn transpile_project_merged(
    source: &str,
    project_dir: &str,
    extension: &str,
    transpile: fn(&str) -> Result<(String, Vec<SourceMapEntry>), AppError>,
    files: Option<&[ProjectFile]>,
) -> Result<(Vec<TransFile>, Vec<SourceMapEntry>), AppError> {
    let (merged, line_origins) = merge_project_source(source, project_dir, files)?;
    let (code, source_map) = transpile(&merged)?;

    // Remap source_map entries from merged-line coordinates back to original
    // project-file coordinates so multi-file C/Python/JS/TS projects can use
    // the same per-file bidirectional highlighting as Rust projects.
    let remapped: Vec<SourceMapEntry> = source_map
        .into_iter()
        .map(|mut entry| {
            if let Some(origin) = line_origins.get(entry.source_line.saturating_sub(1)) {
                entry.source_file = Some(origin.file.clone());
                entry.source_line = origin.line;
            }
            entry
        })
        .collect();

    Ok((vec![TransFile {
        path: format!("playground.{extension}"),
        code,
        source_map: Some(remapped.clone()),
    }], remapped))
}

/// Merge project `.at` files into one source string and record the origin of
/// every merged line so that transpiler source maps can be remapped back to
/// the original files.
///
/// Project-local `use <stem>` lines are stripped; files are concatenated in
/// topological order so dependencies are defined before their dependents.
/// When `files` is provided, those edited contents replace the disk copies.
fn merge_project_source(
    source: &str,
    project_dir: &str,
    files: Option<&[ProjectFile]>,
) -> Result<(String, Vec<LineOrigin>), AppError> {
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

    // Overlay edited file contents (keyed by file stem, main excluded — the
    // entry source is already applied above).
    if let Some(files) = files {
        for file in files {
            let stem = std::path::Path::new(&file.path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if stem != "main" && sources.contains_key(&stem) {
                sources.insert(stem, file.source.clone());
            }
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

    // Build merged source, stripping project-local `use` lines, and record
    // the original file/line for every emitted line.
    let mut merged = String::new();
    let mut line_origins: Vec<LineOrigin> = Vec::new();
    for stem in order {
        let content = sources.get(&stem).unwrap();
        let file_name = format!("{stem}.at");
        let mut file_line: usize = 0;
        for line in content.lines() {
            file_line += 1;
            let trimmed = line.trim();
            if let Some(cap) = re.captures(trimmed) {
                let used = cap[1].to_string();
                if used != stem && sources.contains_key(&used) {
                    continue;
                }
            }
            merged.push_str(line);
            merged.push('\n');
            line_origins.push(LineOrigin {
                file: file_name.clone(),
                line: file_line,
            });
        }
        // Separator blank line. Map it to the last real line of this file so
        // that any spurious mapping landing here still points into this module.
        merged.push('\n');
        let last_line = file_line.max(1);
        line_origins.push(LineOrigin {
            file: file_name,
            line: last_line,
        });
    }

    Ok((merged, line_origins))
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
    let (text, source_map) = abt.to_string_with_source_map();
    Ok((text, source_map))
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
