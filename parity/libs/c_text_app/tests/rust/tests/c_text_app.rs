// Rust oracle for c_text_app consumer parity.
//
// Mirrors parity/libs/c_text_app/auto/c_text_app.at: reads a file, applies a
// text transform (replace / trim / lowercase), writes it back, reads it back.
// Uses std::fs directly (the same backend the Auto VM uses for fs.*) and the
// matching str methods. Test names EXACTLY mirror tests/auto/basic.at.
//
// Determinism: each #[test] uses a UNIQUE file path under c_text_app_tmp/ so
// the parallel test threads do not collide (Auto/a2r run one sequential main).
// ASCII-only inputs so Auto lower() and Rust to_lowercase agree.

use std::fs;

fn transform_replace(path: &str, old: &str, new: &str) -> String {
    let content = fs::read_to_string(path).unwrap_or_default();
    let changed = content.replace(old, new);
    let _ = fs::write(path, &changed);
    fs::read_to_string(path).unwrap_or_default()
}

fn transform_trim(path: &str) -> String {
    let content = fs::read_to_string(path).unwrap_or_default();
    let changed = content.trim();
    let _ = fs::write(path, changed);
    fs::read_to_string(path).unwrap_or_default()
}

fn transform_lower(path: &str) -> String {
    let content = fs::read_to_string(path).unwrap_or_default();
    let changed = content.to_lowercase();
    let _ = fs::write(path, &changed);
    fs::read_to_string(path).unwrap_or_default()
}

fn tap_ok(n: u32, name: &str) {
    println!("ok {} - {}", n, name);
}

#[test]
fn test_replace_basic() {
    let dir = "c_text_app_tmp";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/r1.txt", dir);
    let _ = fs::write(&p, "hello world world");
    assert_eq!(transform_replace(&p, "world", "auto"), "hello auto auto");
    tap_ok(1, "test_replace_basic");
}

#[test]
fn test_replace_no_match() {
    let dir = "c_text_app_tmp";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/r2.txt", dir);
    let _ = fs::write(&p, "no match here");
    assert_eq!(transform_replace(&p, "xyz", "abc"), "no match here");
    tap_ok(2, "test_replace_no_match");
}

#[test]
fn test_trim_both() {
    let dir = "c_text_app_tmp";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/t1.txt", dir);
    let _ = fs::write(&p, "  hello  ");
    assert_eq!(transform_trim(&p), "hello");
    tap_ok(3, "test_trim_both");
}

#[test]
fn test_trim_none() {
    let dir = "c_text_app_tmp";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/t2.txt", dir);
    let _ = fs::write(&p, "already");
    assert_eq!(transform_trim(&p), "already");
    tap_ok(4, "test_trim_none");
}

#[test]
fn test_lower_mixed() {
    let dir = "c_text_app_tmp";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/l1.txt", dir);
    let _ = fs::write(&p, "Hello WORLD");
    assert_eq!(transform_lower(&p), "hello world");
    tap_ok(5, "test_lower_mixed");
}

#[test]
fn test_lower_already() {
    let dir = "c_text_app_tmp";
    let _ = fs::create_dir_all(dir);
    let p = format!("{}/l2.txt", dir);
    let _ = fs::write(&p, "already_lower");
    assert_eq!(transform_lower(&p), "already_lower");
    tap_ok(6, "test_lower_already");
}
