// Plan 075 Phase 3: Unified Compilation API Tests
// Tests for run_with_mode, detect_mode_from_extension, run_file_with_auto_mode

use crate::{CompileMode, run_with_mode, detect_mode_from_extension};
use std::path::Path;

#[test]
fn test_run_with_mode_script() {
    let source = "1 + 2";
    let result = run_with_mode(source, CompileMode::Script).unwrap();
    assert!(result.contains("script"), "Result should contain 'script': {}", result);
    assert!(result.contains("bytecode="), "Result should contain bytecode info: {}", result);
}

#[test]
fn test_run_with_mode_config() {
    let source = r#"
server.host = "localhost"
server.port = 8080
"#;
    let result = run_with_mode(source, CompileMode::Config).unwrap();
    assert!(result.contains("config"), "Result should contain 'config': {}", result);
    assert!(result.contains("bytecode="), "Result should contain bytecode info: {}", result);
    assert!(result.contains("strings="), "Result should contain strings info: {}", result);
}

#[test]
fn test_run_with_mode_template() {
    let source = r#""Hello, "
"World!""#;
    let result = run_with_mode(source, CompileMode::Template).unwrap();
    assert!(result.contains("template"), "Result should contain 'template': {}", result);
    assert!(result.contains("bytecode="), "Result should contain bytecode info: {}", result);
}

#[test]
fn test_detect_mode_from_extension_config() {
    let path = Path::new("database.config.at");
    let mode = detect_mode_from_extension(path).unwrap();
    assert_eq!(mode, CompileMode::Config);
}

#[test]
fn test_detect_mode_from_extension_template() {
    let path = Path::new("email.template.at");
    let mode = detect_mode_from_extension(path).unwrap();
    assert_eq!(mode, CompileMode::Template);
}

#[test]
fn test_detect_mode_from_extension_script() {
    let path = Path::new("script.at");
    let mode = detect_mode_from_extension(path).unwrap();
    assert_eq!(mode, CompileMode::Script);
}

#[test]
fn test_detect_mode_from_extension_default() {
    let path = Path::new("unknown.txt");
    let mode = detect_mode_from_extension(path).unwrap();
    assert_eq!(mode, CompileMode::Script);
}

#[test]
fn test_detect_mode_from_extension_nested_path() {
    let path = Path::new("configs/production/database.config.at");
    let mode = detect_mode_from_extension(path).unwrap();
    assert_eq!(mode, CompileMode::Config);
}

#[test]
fn test_config_mode_with_nested_fields() {
    let source = r#"
server.host = "localhost"
server.port = 5432
database.name = "mydb"
debug = true
"#;
    let result = run_with_mode(source, CompileMode::Config).unwrap();
    assert!(result.contains("config"), "Result should contain 'config': {}", result);
}

#[test]
fn test_template_mode_with_multiple_strings() {
    let source = r#""Hello"
" "
"World!""#;
    let result = run_with_mode(source, CompileMode::Template).unwrap();
    assert!(result.contains("template"), "Result should contain 'template': {}", result);
}

#[test]
fn test_script_mode_with_function() {
    let source = r#"
fn add(a int, b int) int {
    a + b
}

add(1, 2)
"#;
    let result = run_with_mode(source, CompileMode::Script).unwrap();
    assert!(result.contains("script"), "Result should contain 'script': {}", result);
}

#[test]
fn test_empty_config() {
    let source = "";
    let result = run_with_mode(source, CompileMode::Config).unwrap();
    assert!(result.contains("config"), "Empty config should still compile");
}

#[test]
fn test_empty_template() {
    let source = "";
    let result = run_with_mode(source, CompileMode::Template).unwrap();
    assert!(result.contains("template"), "Empty template should still compile");
}

#[test]
fn test_config_mode_with_expressions() {
    let source = r#"
port = 8080
max_connections = 100
timeout = port * 2
"#;
    let result = run_with_mode(source, CompileMode::Config).unwrap();
    assert!(result.contains("config"), "Config with expressions should compile");
}
