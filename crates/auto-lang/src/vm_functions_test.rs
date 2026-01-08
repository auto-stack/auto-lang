use crate::run;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

fn create_test_file(filename: &str, content: &str) -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push(filename);
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

fn cleanup_test_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_vm_function_open() {
    let test_file = create_test_file("test_vm_open.at", "Hello from VM!");

    let code = format!(r#"
use auto.io: open

let file = open("test_vm_open.at")
file
"#);
    let result = run(&code);
    cleanup_test_file(&test_file);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("File"), "Expected File instance, got: {}", output);
}

#[test]
fn test_vm_method_read_text() {
    let test_content = "Hello from VM read_text!";
    let test_file = create_test_file("test_vm_read.at", test_content);

    let code = format!(r#"
use auto.io: open

let file = open("test_vm_read.at")
file.read_text()
"#);
    let result = run(&code);
    cleanup_test_file(&test_file);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains(test_content) || output.contains("Hello"),
            "Expected file content, got: {}", output);
}

#[test]
fn test_vm_method_close() {
    let test_file = create_test_file("test_vm_close.at", "Hello from VM close!");

    let code = format!(r#"
use auto.io: open

let file = open("test_vm_close.at")
file.close()
"#);
    let result = run(&code);
    cleanup_test_file(&test_file);

    assert!(result.is_ok(), "VM method close failed: {:?}", result);
}

#[test]
fn test_vm_function_error() {
    let code = r#"
use auto.io: open

let file = open("nonexistent_vm_test_file.txt")
file
"#;
    let result = run(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Error") || output.contains("not found"),
            "Expected error for nonexistent file, got: {}", output);
}
