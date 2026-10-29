//! Shared helpers for the PLAN-783 probe integration tests.

// Shared helpers compiled into EVERY test binary; each binary uses a
// subset, so unused-item warnings here are noise.
#![allow(dead_code)]

use std::path::{Path, PathBuf};


/// (fixture, expected exit) — the four core oracle fixtures (§5.3).
pub const ORACLE: &[(&str, i32)] = &[
    ("p01-precedence.at", 14),
    ("p04-while-retest.at", 3),
    ("p12-unexecuted-trap-not-lifted-x1.at", 0),
    ("p13-add-overflow-trap.at", 70),
];

pub fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ac-core/fixtures/source/valid")
        .join(name)
}

/// A unique, cleaned temp root per test family.
pub fn scratch(family: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ac783-{}", family));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

pub struct NodeOut {
    pub code: i32,
    pub lines: Vec<(String, serde_json::Value)>,
    pub raw: String,
}

pub fn first<'a>(out: &'a NodeOut, key: &str) -> &'a serde_json::Value {
    out.lines
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("missing {} line in node output:\n{}", key, out.raw))
}

/// The scalar compatibility control fixture (§4: core line + one scalar
/// control): while-retest compiled under the scalar profile.
pub fn fixture_src(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name)).unwrap()
}
