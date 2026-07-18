use axum::Json;
use serde::Serialize;
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

fn load_examples() -> Vec<Example> {
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
