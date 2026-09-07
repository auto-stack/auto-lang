use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::project::ProjectFile;

#[derive(Serialize)]
pub struct Example {
    pub name: String,
    pub source: String,
    pub example_type: String,
    pub project_dir: Option<String>,
    pub files: Option<Vec<ProjectFile>>,
}

#[derive(Serialize)]
pub struct ExamplesResponse {
    pub examples: Vec<Example>,
}

pub async fn examples_handler() -> Json<ExamplesResponse> {
    Json(ExamplesResponse {
        examples: load_examples(),
    })
}

// ── Notes manifest 单一事实源（Plan 582 T13；schema v1 见 scripts/build-playground-notes.mjs）──
//
// 启动时探测 CARGO_MANIFEST_DIR/notes.json（构建期产物，gitignore）：存在则解析
// 映射为现有 Example 响应（schema 不变，兼容 ExampleSelector/Full）；缺失或解析
// 失败回退现有目录扫描。选择文件探测而非 include_str!：生成时序灵活、二进制不背书。

#[derive(Deserialize)]
struct NotesManifest {
    #[allow(dead_code)] // version 字段仅校验存在性，不参与映射
    version: serde_json::Value,
    groups: Vec<NoteGroup>,
}

#[derive(Deserialize)]
struct NoteGroup {
    notes: Vec<NoteMetaEntry>,
}

#[derive(Deserialize)]
struct NoteMetaEntry {
    title: String,
    #[serde(rename = "sourceType")]
    _source_type: String,
    #[serde(rename = "sourcePath")]
    source_path: String,
    kind: String,
    code: Option<String>,
    files: Option<Vec<ManifestFile>>,
}

#[derive(Deserialize)]
struct ManifestFile {
    path: String,
    content: String,
}

fn load_examples() -> Vec<Example> {
    match load_from_manifest() {
        Some(examples) => examples,
        None => load_from_dir_scan(),
    }
}

fn load_from_manifest() -> Option<Vec<Example>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("notes.json");
    let raw = std::fs::read_to_string(&path).ok()?;
    let manifest: NotesManifest = match serde_json::from_str(&raw) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("notes.json 解析失败（{}），回退目录扫描", e);
            return None;
        }
    };
    let mut out = Vec::new();
    for group in &manifest.groups {
        for note in &group.notes {
            let files: Vec<ProjectFile> = note
                .files
                .iter()
                .flatten()
                .map(|f| ProjectFile { path: f.path.clone(), source: f.content.clone() })
                .collect();
            // entry 恒为 main.at 内容（kind=project 时 code 为 null）；parity 等无 main.at
            // 的 project 笔记回退首文件（与前端 cardCode/loadNote 回退规则一致）。
            let source = note
                .code
                .clone()
                .or_else(|| files.iter().find(|f| f.path == "main.at").map(|f| f.source.clone()))
                .or_else(|| files.first().map(|f| f.source.clone()))
                .unwrap_or_default();
            if source.is_empty() {
                continue;
            }
            let is_project = note.kind == "project";
            // 项目目录相对 examples/playground-demo（服务端物化基座）；其余来源无服务端目录。
            let project_dir = if is_project {
                note.source_path
                    .strip_prefix("examples/playground-demo/")
                    .and_then(|rest| rest.strip_suffix("/main.at"))
                    .map(str::to_string)
            } else {
                None
            };
            out.push(Example {
                name: note.title.clone(),
                source,
                example_type: if is_project { "project".into() } else { "single".into() },
                project_dir,
                files: if is_project { Some(files) } else { None },
            });
        }
    }
    if out.is_empty() {
        tracing::warn!("notes.json 存在但 0 条笔记，回退目录扫描");
        return None;
    }
    tracing::info!("/api/examples 读 notes.json（{} 条，单一事实源）", out.len());
    Some(out)
}

fn load_from_dir_scan() -> Vec<Example> {
    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("examples/playground-demo"));

    if let Some(dir) = examples_dir {
        if dir.is_dir() {
            return load_from_dir(&dir);
        }
    }

    fallback_examples()
}

fn display_name_from_stem(stem: &str) -> String {
    stem.split_once('-')
        .map(|(_, rest)| {
            let mut title = String::new();
            let mut prev = '-';
            for c in rest.chars() {
                if c == '_' {
                    title.push(' ');
                } else if prev == '-' || prev == '_' || prev == ' ' {
                    for uc in c.to_uppercase() {
                        title.push(uc);
                    }
                } else {
                    title.push(c);
                }
                prev = c;
            }
            title
        })
        .unwrap_or_else(|| stem.to_string())
}

fn load_from_dir(dir: &std::path::Path) -> Vec<Example> {
    let mut examples = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read examples dir: {e}"))
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    // Single-file examples
    for entry in entries.iter() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "at") {
            continue;
        }
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        examples.push(Example {
            name: display_name_from_stem(&stem),
            source,
            example_type: "single".into(),
            project_dir: None,
            files: None,
        });
    }

    // Project examples: directories containing main.at
    for entry in entries.iter() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let main_path = path.join("main.at");
        if !main_path.is_file() {
            continue;
        }
        let source = match std::fs::read_to_string(&main_path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let stem = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let project_dir = path
            .strip_prefix(dir)
            .ok()
            .map(|p| p.to_string_lossy().to_string().replace('\\', "/"));
        // Collect all project .at files: main.at first, then alphabetical.
        let mut files: Vec<ProjectFile> = Vec::new();
        let mut others: Vec<ProjectFile> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let file_path = entry.path();
                if !file_path.is_file()
                    || file_path.extension().is_none_or(|ext| ext != "at")
                {
                    continue;
                }
                let file_name = file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    if file_name == "main.at" {
                        files.push(ProjectFile { path: file_name, source: content });
                    } else {
                        others.push(ProjectFile { path: file_name, source: content });
                    }
                }
            }
        }
        others.sort_by(|a, b| a.path.cmp(&b.path));
        files.extend(others);
        examples.push(Example {
            name: display_name_from_stem(&stem),
            source,
            example_type: "project".into(),
            project_dir,
            files: Some(files),
        });
    }

    examples
}

fn fallback_examples() -> Vec<Example> {
    vec![
        Example {
            name: "Hello World".into(),
            source: r#"print("Hello, World!")"#.into(),
            example_type: "single".into(),
            project_dir: None,
            files: None,
        },
        Example {
            name: "Variables".into(),
            source: r#"let x = 42
let name = "Auto"
print(f"Hello, $name! The answer is $x")"#.into(),
            example_type: "single".into(),
            project_dir: None,
            files: None,
        },
        Example {
            name: "Functions".into(),
            source: r#"fn add(a int, b int) int {
    a + b
}

let result = add(3, 4)
print(result)"#.into(),
            example_type: "single".into(),
            project_dir: None,
            files: None,
        },
    ]
}
