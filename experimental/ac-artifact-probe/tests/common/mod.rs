//! Shared helpers for the PLAN-783 probe integration tests.

// Shared helpers compiled into EVERY test binary; each binary uses a
// subset, so unused-item warnings here are noise.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub const NODE_EXE: &str = env!("CARGO_BIN_EXE_node");

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

/// Run the node helper under the probe deadline, parse its JSON lines.
pub fn run_node(args: &[&str]) -> NodeOut {
    run_node_deadlined(args, 120)
}

/// `run_node` with an explicit deadline (long loads: the 512-iteration
/// churn legitimately needs minutes; the plan's 60 s per-subprocess rule
/// applies to BUILD subprocesses, not the orchestrating helper).
pub fn run_node_deadlined(args: &[&str], secs: u64) -> NodeOut {
    let mut cmd = Command::new(NODE_EXE);
    cmd.args(args);
    let r = ac_artifact_probe::subproc::run_deadlined(cmd, Duration::from_secs(secs))
        .expect("spawn node");
    assert!(!r.timed_out, "node timed out: {:?}", args);
    let mut lines = Vec::new();
    for l in r.stdout.lines() {
        if let Some((k, rest)) = l.split_once(' ') {
            if let Ok(v) = serde_json::from_str(rest) {
                lines.push((k.to_string(), v));
            }
        }
    }
    NodeOut { code: r.code.unwrap_or(-1), lines, raw: format!("{}{}", r.stdout, r.stderr) }
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
