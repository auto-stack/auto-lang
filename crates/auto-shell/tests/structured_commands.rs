//! Integration tests for structured command output

use auto_shell::shell::Shell;
use std::fs;

#[test]
fn test_ls_returns_structured_data() {
    let mut shell = Shell::new();

    // Execute ls command
    let result = shell.execute("ls .");

    // Should return structured data (array of objects)
    assert!(result.is_ok());
}

#[test]
fn test_ps_returns_structured_data() {
    let mut shell = Shell::new();

    // Execute ps command
    let result = shell.execute("ps");

    // Should return structured data
    assert!(result.is_ok());
}

#[test]
fn test_sys_disks_returns_structured_data() {
    let mut shell = Shell::new();

    // Execute sys disks command
    let result = shell.execute("sys disks");

    // Should return structured data
    assert!(result.is_ok());
}

#[test]
fn test_sys_mem_returns_structured_data() {
    let mut shell = Shell::new();

    // Execute sys mem command
    let result = shell.execute("sys mem");

    // Should return structured data
    assert!(result.is_ok());
}

// File operation tests (using current directory)
#[test]
fn test_mkdir_creates_directory() {
    let mut shell = Shell::new();
    let test_dir = "test_mkdir_ash_temp";

    // Clean up if exists
    let _ = fs::remove_dir_all(test_dir);

    // Create directory
    let result = shell.execute(&format!("mkdir {}", test_dir));

    // Should succeed
    assert!(result.is_ok(), "mkdir should succeed");

    // Verify directory exists
    assert!(std::path::Path::new(test_dir).exists(), "directory should exist");

    // Clean up
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_cp_copies_file() {
    let mut shell = Shell::new();
    let source_file = "test_source_ash.txt";
    let dest_file = "test_dest_ash.txt";

    // Clean up if exists
    let _ = fs::remove_file(source_file);
    let _ = fs::remove_file(dest_file);

    // Create source file
    fs::write(source_file, "test content").unwrap();

    // Copy file
    let result = shell.execute(&format!("cp {} {}", source_file, dest_file));

    // Should succeed
    assert!(result.is_ok(), "cp should succeed");

    // Verify destination exists
    assert!(std::path::Path::new(dest_file).exists(), "destination file should exist");

    // Clean up
    let _ = fs::remove_file(source_file);
    let _ = fs::remove_file(dest_file);
}

#[test]
fn test_mv_moves_file() {
    let mut shell = Shell::new();
    let source_file = "test_source_mv_ash.txt";
    let dest_file = "test_dest_mv_ash.txt";

    // Clean up if exists
    let _ = fs::remove_file(&source_file);
    let _ = fs::remove_file(&dest_file);

    // Create source file
    fs::write(&source_file, "test content").unwrap();

    // Move file
    let result = shell.execute(&format!("mv {} {}", source_file, dest_file));

    // Should succeed
    assert!(result.is_ok(), "mv should succeed");

    // Verify source is gone and destination exists
    assert!(!std::path::Path::new(&source_file).exists(), "source should not exist");
    assert!(std::path::Path::new(&dest_file).exists(), "destination should exist");

    // Clean up
    let _ = fs::remove_file(&dest_file);
}

#[test]
fn test_rm_removes_file() {
    let mut shell = Shell::new();
    let test_file = "test_rm_ash.txt";

    // Clean up if exists
    let _ = fs::remove_file(test_file);

    // Create test file
    fs::write(test_file, "test content").unwrap();

    // Remove file
    let result = shell.execute(&format!("rm {}", test_file));

    // Should succeed
    assert!(result.is_ok(), "rm should succeed");

    // Verify file is gone
    assert!(!std::path::Path::new(test_file).exists(), "file should not exist");
}

#[test]
fn test_rm_removes_directory_recursively() {
    let mut shell = Shell::new();
    let test_dir = "test_rm_recursive_ash";
    let nested_dir = format!("{}/nested", test_dir);

    // Clean up if exists
    let _ = fs::remove_dir_all(test_dir);

    // Create test directory structure
    fs::create_dir_all(&nested_dir).unwrap();

    // Remove directory recursively
    let result = shell.execute(&format!("rm -r {}", test_dir));

    // Should succeed
    assert!(result.is_ok(), "rm -r should succeed");

    // Verify directory is gone
    assert!(!std::path::Path::new(test_dir).exists(), "directory should not exist");
}

