//! Shared helpers of the PLAN-978 permanent cases (`ace.m2.p978.*`).
//!
//! Every case builds in its own scratch directory under the system temp
//! root (never inside a repo checkout) and measures its own wall time;
//! the plan-artifacts evidence policy keeps raw runs out of git.
#![allow(dead_code)] // per-test-file copies use different subsets

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use ace_m2_build::driver::{M2Build, ProjectInputs};

pub const B01_SRC: &str = "fn main() int { return 2 + 3 * 4 }";
pub const B01_EDITED: &str = "fn main() int { return 2 + 3 * 5 }";
/// crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at (SHA256 prefix
/// 4af83cfa91c6ba9c — the plan's source02 mapping): an explicit C-FFI
/// consumer whose support domain is the frozen F0/C lane.
pub const STRLEN_AT: &str = r#"use c <string.h>
fn main() int { return strlen(c"hello") }"#;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A unique scratch dir (system temp — never inside a repo checkout).
pub fn scratch(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("p978-{}-{}-{}", name, std::process::id(), n));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

pub fn b01_inputs() -> ProjectInputs {
    ProjectInputs::new("b01", "main", "main.at", B01_SRC)
}

pub fn b01_edited_inputs() -> ProjectInputs {
    ProjectInputs::new("b01", "main", "main.at", B01_EDITED)
}

pub fn driver(root: &std::path::Path) -> M2Build {
    M2Build::new(root)
}

/// Run `f` timed (no process probing — use `owned_tools_guard` around a
/// whole sequence for the toolchain oracle).
pub fn timed<F, T>(f: F) -> (T, f64)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let out = f();
    (out, start.elapsed().as_secs_f64() * 1000.0)
}

/// The INDEPENDENT toolchain oracle around a whole operation sequence:
/// toolchain processes DESCENDED FROM THIS TEST PROCESS, before vs
/// after. External concurrent builds (other agents' worktrees) do not
/// false-positive; a hidden build inside this process tree cannot
/// escape.
pub fn owned_tools_guard<F, T>(f: F) -> (T, BTreeMap<String, u32>, bool)
where
    F: FnOnce() -> T,
{
    let before = ace_m2_build::probe::owned_snapshot(std::process::id());
    let out = f();
    let after = ace_m2_build::probe::owned_snapshot(std::process::id());
    let mut diff = BTreeMap::new();
    for (img, count) in after.counts {
        let prev = before.counts.get(&img).copied().unwrap_or(0);
        if count > prev {
            diff.insert(img, count - prev);
        }
    }
    (out, diff, before.available && after.available)
}

use std::collections::BTreeMap;

/// The prepared `ac-build` CLI (built by the test build phase, never by a
/// test body).
pub fn ac_build_exe() -> &'static str {
    env!("CARGO_BIN_EXE_ac-build")
}
