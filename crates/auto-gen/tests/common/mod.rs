//! Shared helpers for the PLAN-791 frozen-behavior suites
//! (`v02_compat_golden.rs` / `v03_failure_semantics.rs`).
//!
//! Goldens under `tests/fixtures/goldens/<case>/` are byte-frozen from the
//! pre-extraction implementation (worktree base e4ac72a62). They are inputs,
//! not outputs of the current code: regenerate ONLY deliberately via
//! `UPDATE_GOLDENS=1` after proving the behavior change is intended.

use std::fs;
use std::path::{Path, PathBuf};

pub fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn templates_dir() -> PathBuf {
    fixture_dir().join("templates")
}

pub fn data_dir() -> PathBuf {
    fixture_dir().join("data")
}

pub fn goldens_dir() -> PathBuf {
    fixture_dir().join("goldens")
}

pub fn update_goldens() -> bool {
    std::env::var("UPDATE_GOLDENS").is_ok_and(|v| v == "1")
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let target = dst.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn rel_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                out.push(path.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    walk(root, root, &mut out);
    out.sort();
    out
}

/// Assert every file under `produced_dir` equals the byte-frozen golden tree
/// `goldens/<case>` (same relative file set, same bytes). With
/// `UPDATE_GOLDENS=1` the golden tree is rewritten from `produced_dir`
/// instead (bless mode; never set this in CI/plain runs).
pub fn assert_tree_matches_golden(case: &str, produced_dir: &Path) {
    let golden_root = goldens_dir().join(case);
    if update_goldens() {
        if golden_root.exists() {
            fs::remove_dir_all(&golden_root).unwrap();
        }
        copy_tree(produced_dir, &golden_root);
        eprintln!("[goldens] updated case '{case}' from {produced_dir:?}");
        return;
    }
    assert!(
        golden_root.is_dir(),
        "golden tree missing for case '{case}': {}",
        golden_root.display()
    );
    let produced: Vec<PathBuf> = rel_files(produced_dir);
    let expected: Vec<PathBuf> = rel_files(&golden_root);
    assert_eq!(
        produced, expected,
        "file set differs for golden case '{case}'"
    );
    for rel in &expected {
        let got = fs::read(produced_dir.join(rel)).unwrap();
        let want = fs::read(golden_root.join(rel)).unwrap();
        assert_eq!(
            got, want,
            "byte mismatch for golden case '{case}' / {}",
            rel.display()
        );
    }
}

/// Standard in-memory data atom used by the frozen cases. Mirrors the shape
/// auto-man passes to the legacy API (a to_atom() node) and exercises both
/// scalar interpolation and `$ for` code lines over an array of nodes.
pub fn sample_atom() -> auto_lang::atom::Atom {
    use auto_lang::atom::Atom;
    use auto_val::{Node, Value};

    let mut alpha = Node::new("item");
    alpha.set_prop("name", "alpha");
    alpha.set_prop("id", 1);
    let mut beta = Node::new("item");
    beta.set_prop("name", "beta");
    beta.set_prop("id", 2);
    let items = auto_val::Array::from_vec(vec![Value::node(alpha), Value::node(beta)]);

    Atom::assemble(vec![
        Value::pair("name", "AuthService"),
        Value::pair("version", 2),
        Value::pair("items", Value::array(items)),
    ])
    .unwrap()
}
