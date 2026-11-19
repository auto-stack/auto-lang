//! PLAN-1107 V-02 (AV leg): the shared protocol case — nested calls, a bound
//! method call, a keyword argument, loop/branch value flow and explicit
//! conversions — runs through the AV `PySession` and matches the independent
//! native-Python reference byte-for-value.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

use ace_abi::py_session::PySession;
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
        .arg(corpus_dir().join("protocol_1107_ref.py"))
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
fn p1107_v02_protocol_case_av_matches_the_native_oracle() {
    let src = std::fs::read_to_string(corpus_dir().join("protocol_1107.at"))
        .expect("the shared V-02 protocol corpus");
    let mut session = PySession::open().expect("session opens");
    let run = session
        .run_with(
            &src,
            &ExecLimits {
                max_steps: 50_000_000,
                wall: std::time::Duration::from_secs(120),
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("AV protocol lane: {e}"));
    assert!(run.success_i32().is_some(), "{:?}", run.outcome);
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
    eprintln!("V-02 AV: {got:?} == oracle {want:?}");
}
/// PLAN-1107 V-02 regression (AV): the frame operand chunk must evaluate its
/// operand EXACTLY ONCE. `l.count(22).to(int)` as a print argument used to be
/// written into the frame twice (dlen + value), minting two tokens: with
/// single-digit tokens the bug was invisible, with two-digit tokens the
/// declared payload length desynced and the frame was rejected.
#[test]
fn p1107_v02_av_frame_operand_is_evaluated_once() {
    let src = "use.py builtins: list, round, len, repr
fn main() int {
var l = list()
var a1 = l.append(11)
var a2 = l.append(22)
var a3 = l.append(33)
print(l.count(22).to(int))
return 7
}";
    let mut session = PySession::open().expect("session opens");
    let run = session
        .run_with(
            src,
            &ExecLimits {
                max_steps: 5_000_000,
                wall: std::time::Duration::from_secs(60),
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("the frame must be well formed: {e}"));
    assert_eq!(run.success_i32(), Some(7), "{:?}", run.outcome);
    assert_eq!(
        run.stdout_text(),
        "1\n",
        "one evaluation, one frame, one print"
    );
}

/// PLAN-1107 V-02 regression (AV): an inline side-effecting receiver runs
/// once. `l.pop()` as a conversion receiver must remove exactly one element.
#[test]
fn p1107_v02_av_inline_side_effect_is_evaluated_once() {
    let src = "use.py builtins: list, len
fn main() int {
var l = list()
var a1 = l.append(7)
var a2 = l.append(8)
print(l.pop().to(int))
print(len(l).to(int))
return 0
}";
    let mut session = PySession::open().expect("session opens");
    let run = session
        .run_with(
            src,
            &ExecLimits {
                max_steps: 5_000_000,
                wall: std::time::Duration::from_secs(60),
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("lane: {e}"));
    assert_eq!(run.success_i32(), Some(0), "{:?}", run.outcome);
    assert_eq!(
        run.stdout_text(),
        "8\n1\n",
        "the popped value once, and exactly one element removed (a double evaluation would leave 0)"
    );
}
