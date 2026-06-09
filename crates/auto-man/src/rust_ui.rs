//! Rust UI (ICED/GPUI) project generation utilities
//!
//! This module generates Rust code from AURA widget definitions,
//! targeting ICED or GPUI backends via the auto_lang::ui runtime.
//!
//! Workflow:
//! 1. Read .at files from a `front/` directory
//! 2. Parse with AURA pipeline (CompilerSession::ui with "rust" backend)
//! 3. Extract WidgetDecl AST nodes -> AuraWidget
//! 4. Generate Rust code via RustGenerator
//! 5. Wrap in main() with backend selection (ICED/GPUI)
//! 6. Write to `rust/<name>.rs`

use std::fs;
use std::path::{Path, PathBuf};

use auto_lang::ui_gen::rust::RustGenerator;
use auto_lang::ui_gen::BackendGenerator;
use auto_lang::Parser;
use auto_lang::session::CompilerSession;
use colored::Colorize;

use crate::AutoResult;

/// Generate Rust UI code from .at files in a project directory.
///
/// Resolve the front/ source directory for a project.
fn find_front_dir(project_dir: &Path) -> PathBuf {
    if project_dir.join("src").join("front").exists() {
        project_dir.join("src").join("front")
    } else if project_dir.join("source").join("front").exists() {
        project_dir.join("source").join("front")
    } else if project_dir.join("front").exists() {
        project_dir.join("front")
    } else {
        project_dir.join("src").join("front")
    }
}

/// Check if the generated Rust project needs to be regenerated.
/// Returns (needs_full_regen, needs_code_regen).
fn needs_regeneration(project_dir: &Path, rust_dir: &Path) -> (bool, bool) {
    let cargo_toml = rust_dir.join("Cargo.toml");
    let main_rs = rust_dir.join("src").join("main.rs");

    if !cargo_toml.exists() || !main_rs.exists() {
        return (true, true);
    }

    // Check if any .at source file is newer than main.rs
    let front_dir = find_front_dir(project_dir);
    if let Ok(at_files) = collect_at_files(&front_dir) {
        if let Ok(main_meta) = fs::metadata(&main_rs) {
            if let Ok(main_time) = main_meta.modified() {
                for at_file in &at_files {
                    if let Ok(at_meta) = fs::metadata(at_file) {
                        if let Ok(at_time) = at_meta.modified() {
                            if at_time > main_time {
                                return (false, true);
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if default feature in Cargo.toml matches expected
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        if !content.contains("default = [\"ui-iced\"]") {
            return (true, true);
        }
    }

    (false, false)
}

/// Regenerate only main.rs (skip Cargo.toml to preserve cargo cache).
fn regenerate_code_only(project_dir: &Path, rust_dir: &Path) -> AutoResult<()> {
    let front_dir = find_front_dir(project_dir);
    let at_files = collect_at_files(&front_dir)?;
    if at_files.is_empty() {
        return Ok(());
    }

    let pac_path = project_dir.join("pac.at");
    let project_name = if pac_path.exists() {
        parse_pac_name(&pac_path).unwrap_or_else(|| "MyApp".to_string())
    } else {
        "MyApp".to_string()
    };

    let mut all_components = String::new();
    for at_path in &at_files {
        match compile_at_file(at_path) {
            Ok(code) => {
                all_components.push_str(&code);
                all_components.push('\n');
            }
            Err(e) => {
                let file_name = at_path.file_name().unwrap_or_default().to_string_lossy();
                println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), file_name, e);
            }
        }
    }

    if all_components.trim().is_empty() {
        return Ok(());
    }

    let full_code = wrap_example(&project_name, &all_components);
    let main_rs = rust_dir.join("src").join("main.rs");
    fs::write(&main_rs, &full_code)
        .map_err(|e| format!("Failed to write {}: {}", main_rs.display(), e))?;

    Ok(())
}

/// `project_dir` is the workspace root (where pac.at lives).
/// `output_dir` overrides the default `rust/` output directory.
/// `_project` is reserved for future full-project scaffolding.
pub fn generate_rust_ui(
    project_dir: &Path,
    output_dir: Option<&Path>,
    _project: bool,
) -> AutoResult<()> {
    println!("{}", "Generating Rust UI code".bright_cyan());

    let front_dir = find_front_dir(project_dir);

    if !front_dir.exists() {
        return Err(format!(
            "Front directory not found: {}",
            front_dir.display()
        )
        .into());
    }

    // Collect .at files
    let at_files = collect_at_files(&front_dir)?;
    if at_files.is_empty() {
        println!("{}", "  No .at files found in front directory".bright_yellow());
        return Ok(());
    }

    println!(
        "{} {} files found",
        "  Found".bright_green(),
        at_files.len()
    );

    // Determine output directory
    let default_output = project_dir.join("gen").join("front").join("rust");
    let output = output_dir
        .map(|p| p.to_path_buf())
        .unwrap_or(default_output);

    fs::create_dir_all(&output)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Get project name from pac.at
    let pac_path = project_dir.join("pac.at");
    let project_name = if pac_path.exists() {
        parse_pac_name(&pac_path).unwrap_or_else(|| "MyApp".to_string())
    } else {
        "MyApp".to_string()
    };

    // Compile each .at file and collect generated components
    let mut all_components = String::new();
    for at_path in &at_files {
        let file_name = at_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        println!("  {} {}", "Parsing".bright_cyan(), file_name);

        match compile_at_file(at_path) {
            Ok(code) => {
                all_components.push_str(&code);
                all_components.push('\n');
            }
            Err(e) => {
                println!(
                    "{} Failed to compile {}: {}",
                    "Warning:".bright_yellow(),
                    file_name,
                    e
                );
            }
        }
    }

    if all_components.trim().is_empty() {
        println!(
            "{}",
            "  No components generated (no WidgetDecl nodes found)".bright_yellow()
        );
        return Ok(());
    }

    // Wrap in main() boilerplate
    let main_widget = extract_main_widget(&all_components);
    let full_code = wrap_example(&project_name, &all_components);

    // Write output as a Cargo project
    let src_dir = output.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create src directory: {}", e))?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, &full_code)
        .map_err(|e| format!("Failed to write {}: {}", main_rs.display(), e))?;

    // Generate Cargo.toml with auto-lang dependency + UI features
    let cargo_toml = generate_cargo_toml(&project_name, project_dir);
    let cargo_path = output.join("Cargo.toml");
    fs::write(&cargo_path, &cargo_toml)
        .map_err(|e| format!("Failed to write {}: {}", cargo_path.display(), e))?;

    println!();
    println!(
        "{} {}",
        "  Generated".bright_green(),
        output.display()
    );
    println!(
        "{} {} (main widget)",
        "  Entry".bright_green(),
        main_widget
    );
    println!();
    println!(
        "{}",
        "  Rust UI project generated successfully!".bright_green().bold()
    );

    Ok(())
}

/// Compile a single .at file to Rust UI code.
fn compile_at_file(at_path: &Path) -> AutoResult<String> {
    let code = fs::read_to_string(at_path)
        .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

    // Parse with UI scenario targeting rust backend
    let session = CompilerSession::ui().with_backend("rust");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);
    let ast = parser
        .parse()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let mut output = String::new();
    let mut generator = RustGenerator::new();

    // Extract AURA widgets from AST
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = auto_lang::aura::extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;

            let rust_code = generator
                .generate(&aura_widget)
                .map_err(|e| e.to_string())?;

            output.push_str(&rust_code);
            output.push('\n');
        }
    }

    Ok(output)
}

/// Wrap generated components in a main() function with ICED/GPUI backend selection.
fn wrap_example(project_name: &str, components: &str) -> String {
    let main_widget = extract_main_widget(components);

    // Strip duplicate imports — RustGenerator already emits them
    let cleaned = components.trim()
        .replace("use auto_lang::ui::{Component, View};\n", "")
        .replace("use auto_lang::ui::{Component, View};", "");

    format!(
        r#"// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{{Component, View}};

{cleaned}

fn main() -> auto_lang::ui::AppResult<()> {{
    #[cfg(feature = "ui-iced")]
    {{
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app::<{main_widget}>();
    }}
    #[cfg(feature = "ui-gpui")]
    {{
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<{main_widget}>("{project_name}");
    }}
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {{
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }}
}}
"#,
        cleaned = cleaned.trim(),
        main_widget = main_widget,
        project_name = to_snake_case(project_name),
    )
}

/// Extract the main widget name from generated components.
/// Looks for "App" struct first, then falls back to the first `pub struct` found.
fn extract_main_widget(components: &str) -> String {
    // Look for "pub struct App"
    for line in components.lines() {
        let trimmed = line.trim();
        if trimmed == "pub struct App {" {
            return "App".to_string();
        }
    }

    // Fallback: find first pub struct
    for line in components.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub struct ") {
            if let Some(name) = rest.split_whitespace().next() {
                // Remove trailing brace if present
                let name = name.trim_end_matches('{').trim();
                return name.to_string();
            }
        }
    }

    // Last resort
    "App".to_string()
}

/// Collect all .at files in a directory (non-recursive).
fn collect_at_files(dir: &Path) -> AutoResult<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().map(|e| e == "at").unwrap_or(false) {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            // Skip pac.at (project config)
            if file_name == "pac.at" {
                continue;
            }
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

/// Parse project name from pac.at file.
fn parse_pac_name(pac_path: &Path) -> Option<String> {
    let content = fs::read_to_string(pac_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                let value = value.trim_end_matches(',');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Convert CamelCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Generate Cargo.toml content for the Rust UI project.
fn generate_cargo_toml(project_name: &str, project_dir: &Path) -> String {
    let snake_name = to_snake_case(project_name);

    // Compute relative path from rust/ back to the workspace root
    let auto_lang_path = if project_dir.join("crates").join("auto-lang").exists() {
        // We're running from the workspace root itself
        "../crates/auto-lang".to_string()
    } else {
        // Relative path from rust/ to the auto-lang crate
        // Find auto-lang crate by going up from the project directory
        find_auto_lang_path(project_dir)
    };

    format!(
        r#"[package]
name = "{snake_name}"
version = "0.1.0"
edition = "2021"

[features]
ui-gpui = ["auto-lang/ui-gpui"]
ui-iced = ["auto-lang/ui-iced"]
default = ["ui-iced"]

[workspace]

[dependencies]
auto-lang = {{ path = "{auto_lang_path}" }}
serde_json = "1"
"#
    )
}

/// Find the relative path from the generated rust/ project to auto-lang crate.
fn find_auto_lang_path(project_dir: &Path) -> String {
    // Try common relative paths
    let candidates = [
        "../../crates/auto-lang",  // examples/<project>/rust/ -> crates/auto-lang
        "../../../crates/auto-lang",
        "../crates/auto-lang",
    ];

    for candidate in &candidates {
        let full = project_dir.join("gen").join("front").join("rust").join(candidate);
        if full.exists() {
            return candidate.to_string();
        }
    }

    // Fallback: compute from project_dir structure
    // Count how many levels up to find crates/auto-lang
    let mut dir = project_dir.to_path_buf();
    let mut prefix = "..".to_string();
    for _ in 0..10 {
        if dir.join("crates").join("auto-lang").exists() {
            let rel = format!("{}/crates/auto-lang", prefix);
            // From rust/ subdirectory
            return format!("../{}", rel);
        }
        if !dir.pop() {
            break;
        }
        prefix = format!("{}/..", prefix);
    }

    // Absolute fallback
    "../../crates/auto-lang".to_string()
}

/// Run the generated Rust UI project.
pub fn run_rust_ui(project_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    let rust_dir = project_dir.join("gen").join("front").join("rust");
    let (full, code) = needs_regeneration(project_dir, &rust_dir);

    if full {
        println!("{}", "Generating Rust UI project...".bright_cyan());
        generate_rust_ui(project_dir, None, false)?;
    } else if code {
        println!("{}", "Regenerating Rust UI code (source changed)...".bright_cyan());
        regenerate_code_only(project_dir, &rust_dir)?;
    }

    println!("{}", "Running Rust UI app (backend: rust-ui)".bright_cyan());

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run");
    for arg in &args {
        cmd.arg(arg);
    }
    cmd.current_dir(&rust_dir);

    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("Cargo run failed with status: {}", status).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MyApp"), "my_app");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("App"), "app");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("lowercase"), "lowercase");
    }

    #[test]
    fn test_extract_main_widget_prefers_app() {
        let code = r#"
pub struct Counter {
    pub count: i32,
}

pub struct App {
    pub title: String,
}
"#;
        assert_eq!(extract_main_widget(code), "App");
    }

    #[test]
    fn test_extract_main_widget_fallback_first_struct() {
        let code = r#"
pub struct Counter {
    pub count: i32,
}

pub struct Timer {
    pub seconds: i32,
}
"#;
        assert_eq!(extract_main_widget(code), "Counter");
    }

    #[test]
    fn test_extract_main_widget_empty() {
        let code = "// no structs here";
        assert_eq!(extract_main_widget(code), "App");
    }

    #[test]
    fn test_parse_pac_name() {
        let dir = std::env::temp_dir().join("auto_test_pac");
        fs::create_dir_all(&dir).ok();
        let pac_path = dir.join("pac.at");
        fs::write(&pac_path, r#"name: "TestProject""#).ok();
        assert_eq!(parse_pac_name(&pac_path), Some("TestProject".to_string()));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_collect_at_files_skips_pac() {
        let dir = std::env::temp_dir().join("auto_test_collect");
        fs::create_dir_all(&dir).ok();
        fs::write(dir.join("app.at"), "").ok();
        fs::write(dir.join("pac.at"), "name: test").ok();
        fs::write(dir.join("other.at"), "").ok();

        let files = collect_at_files(&dir).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"app.at".to_string()));
        assert!(names.contains(&"other.at".to_string()));
        assert!(!names.contains(&"pac.at".to_string()));

        fs::remove_dir_all(&dir).ok();
    }
}
