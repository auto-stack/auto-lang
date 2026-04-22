use axum::Json;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct Example {
    pub name: String,
    pub source: String,
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

fn load_from_dir(dir: &std::path::Path) -> Vec<Example> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read examples dir: {e}"))
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "at")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    entries
        .into_iter()
        .filter_map(|e| {
            let path = e.path();
            let source = std::fs::read_to_string(&path).ok()?;
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let display_name = name
                .split_once('-')
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
                .unwrap_or_else(|| name.clone());
            Some(Example {
                name: display_name,
                source,
            })
        })
        .collect()
}

fn fallback_examples() -> Vec<Example> {
    vec![
        Example {
            name: "Hello World".into(),
            source: r#"print("Hello, World!")"#.into(),
        },
        Example {
            name: "Variables".into(),
            source: r#"let x = 42
let name = "Auto"
print(f"Hello, $name! The answer is $x")"#.into(),
        },
        Example {
            name: "Functions".into(),
            source: r#"fn add(a int, b int) int {
    a + b
}

let result = add(3, 4)
print(result)"#.into(),
        },
    ]
}
