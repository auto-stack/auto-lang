//! PLAN-1107 V-03 (AV leg): object identity and aliasing across the boundary —
//! the same Python object keeps its identity when passed repeatedly, in-place
//! mutation is visible through every binding, a derived tensor is a NEW object,
//! and the same tensor passed twice to one call is the same object twice.
//! The independent native-Python reference must agree value-for-value.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

use super::py_prepared::py_session;
use ace_av_core::av::ExecLimits;

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/ace-m2/python-1107")
}

fn find_python() -> String {
    for cand in [
        std::env::var("AUTO_PYTHON_HOME")
            .map(|h| format!("{h}\\python.exe"))
            .unwrap_or_default(),
        r"C:\Python314\python.exe".to_string(),
        "python".to_string(),
    ] {
        if cand.is_empty() {
            continue;
        }
        if let Ok(out) = Command::new(&cand).arg("--version").output() {
            if out.status.success() {
                return cand;
            }
        }
    }
    panic!("no usable Python interpreter (set AUTO_PYTHON_HOME)");
}

fn oracle_lines() -> Vec<f64> {
    let out = Command::new(find_python())
        .arg(corpus_dir().join("identity_1107_ref.py"))
        .output()
        .expect("run the independent python oracle");
    assert!(out.status.success(), "the oracle must run");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| {
            l.parse::<f64>()
                .unwrap_or_else(|_| panic!("oracle line `{l}`"))
        })
        .collect()
}

#[test]
fn p1107_v03_identity_and_aliasing_via_av_session() {
    let src = std::fs::read_to_string(corpus_dir().join("identity_1107.at"))
        .expect("the shared V-03 identity corpus");
    let mut session = py_session();
    let run = session
        .run_with(
            &src,
            &ExecLimits {
                max_steps: 20_000_000,
                wall: std::time::Duration::from_secs(120),
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("AV identity lane: {e}"));
    assert_eq!(run.success_i32(), Some(0), "{:?}", run.outcome);
    let got: Vec<f64> = run
        .stdout_text()
        .lines()
        .map(|l| l.parse::<f64>().unwrap_or_else(|_| panic!("AV line `{l}`")))
        .collect();
    let want = oracle_lines();
    assert_eq!(got.len(), want.len(), "line count: {got:?} vs {want:?}");
    for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
        assert!((g - w).abs() < 1e-6, "line {i}: {g} vs oracle {w}");
    }
    assert_eq!(want[1], 1.0, "the same binding keeps its Python identity");
    assert_eq!(
        want[2], 1.0,
        "a derived tensor is a DIFFERENT Python object (id difference non-zero)"
    );
    eprintln!("V-03 AV: {got:?} == oracle {want:?}");
}
