// Rust oracle for c_fs_app consumer parity.
//
// Mirrors parity/libs/c_fs_app/auto/c_fs_app.at: uses std::fs directly
// (the same backend the Auto VM uses for fs.*) to write/read/exists/mkdir.
// Test names EXACTLY mirror tests/auto/basic.at so the parity framework can
// compare three-way.
//
// cargo test runs with cwd = tests/rust/; the Auto side runs with cwd = lib
// root. Determinism (design doc §3.2): each #[test] is self-contained — it
// writes to a UNIQUE subdir under c_fs_app_tmp/, so the tests do not collide
// when cargo runs them on parallel threads. (Auto/a2r run all assertions in a
// single sequential main(), so they are already race-free.)

use std::fs;

fn write_and_read(path: &str, content: &str) -> String {
    let _ = fs::write(path, content);
    fs::read_to_string(path).unwrap_or_default()
}

fn check_exists(path: &str) -> i32 {
    if fs::metadata(path).is_ok() { 1 } else { 0 }
}

fn mkdir_write_read(dir: &str, filename: &str, content: &str) -> String {
    let _ = fs::create_dir_all(dir);
    let fullpath = format!("{}/{}", dir, filename);
    let _ = fs::write(&fullpath, content);
    fs::read_to_string(&fullpath).unwrap_or_default()
}

fn tap_ok(n: u32, name: &str) {
    println!("ok {} - {}", n, name);
}

#[test]
fn test_write_read_basic() {
    let dir = "c_fs_app_tmp/basic";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/a.txt", dir);
    assert_eq!(write_and_read(&p, "hello world"), "hello world");
    tap_ok(1, "test_write_read_basic");
}

#[test]
fn test_write_read_empty() {
    let dir = "c_fs_app_tmp/empty";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/b.txt", dir);
    assert_eq!(write_and_read(&p, ""), "");
    tap_ok(2, "test_write_read_empty");
}

#[test]
fn test_write_overwrite() {
    let dir = "c_fs_app_tmp/overwrite";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/a.txt", dir);
    assert_eq!(write_and_read(&p, "first"), "first");
    // Overwrite the same path with new content, then read back.
    assert_eq!(write_and_read(&p, "second"), "second");
    tap_ok(3, "test_write_overwrite");
}

#[test]
fn test_exists_yes() {
    let dir = "c_fs_app_tmp/exists_yes";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/a.txt", dir);
    let _ = fs::write(&p, "x");
    assert_eq!(check_exists(&p), 1);
    tap_ok(4, "test_exists_yes");
}

#[test]
fn test_exists_no() {
    assert_eq!(check_exists("c_fs_app_tmp/nope/no_such.txt"), 0);
    tap_ok(5, "test_exists_no");
}

#[test]
fn test_mkdir_write_read() {
    let dir = "c_fs_app_tmp/mkdir/sub";
    assert_eq!(mkdir_write_read(dir, "nested.txt", "nested content"), "nested content");
    tap_ok(6, "test_mkdir_write_read");
}

#[test]
fn test_nested_exists() {
    // Self-contained: create the nested file in this test's own subdir so it
    // does not depend on test_mkdir_write_read having run first.
    let dir = "c_fs_app_tmp/nested/sub";
    let _ = mkdir_write_read(dir, "nested.txt", "nested content");
    let p = format!("{}/nested.txt", dir);
    assert_eq!(check_exists(&p), 1);
    tap_ok(7, "test_nested_exists");
}
