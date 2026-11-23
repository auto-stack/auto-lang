//! PLAN-1107 V-01 (AV leg): the SAME shared corpus that the A2PY lane and the
//! 978 engineering lane consume runs through the AV `PySession` and matches the
//! independent native-Python oracle. The corpus is one file for every lane —
//! byte identity is pinned here so a second hand-written copy cannot appear.
#![cfg(windows)]

use super::py_prepared::py_session;
use ace_av_core::av::ExecLimits;

const CORPUS: &str = "../../../../crates/auto-lang/test/cases/ace-m2/python-1107/pytorch_train.as";
/// sha256 of `pytorch_train.as` (the frozen §5.1 source, r2 spelling).
const CORPUS_SHA256: &str = "0d5dc8504252fbb1caff55a0e2adf8b0cb8b2b4ab83d0583409d97015a0e9d38";

fn corpus() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS);
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "the shared PLAN-1107 corpus is missing at {}: {e}",
            path.display()
        )
    })
}

#[test]
fn p1107_v01_corpus_identity_is_pinned() {
    let text = corpus();
    assert_eq!(
        ace_core::runtime_contract::sha256_hex(text.as_bytes()),
        CORPUS_SHA256,
        "the shared §5.1 corpus changed — every lane and the oracle note must move together"
    );
    assert!(
        text.contains("SGD(model.parameters(), lr: 0.1)"),
        "the frozen nested-call/kwarg form stays verbatim"
    );
    assert!(
        text.contains("for step < 60"),
        "the language's conditional-loop spelling (r2 correction)"
    );
    assert!(
        !text.contains("while "),
        "`while` is not AutoLang; the r2 corpus must not contain it"
    );
}

#[test]
fn p1107_v01_section51_training_via_av_session() {
    let mut session = py_session();
    let limits = ExecLimits {
        max_steps: 200_000_000,
        wall: std::time::Duration::from_secs(180),
        ..Default::default()
    };
    let started = std::time::Instant::now();
    let run = session
        .run_with(&corpus(), &limits)
        .unwrap_or_else(|e| panic!("AV lane: {e}"));
    assert!(run.success_i32().is_some(), "{:?}", run.outcome);
    let text = run.stdout_text();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 62, "60 losses + shape + pred0: {lines:?}");
    assert_eq!(lines[0], "0.2370201", "first loss (pinned)");
    let first: f32 = lines[0].parse().unwrap();
    let last: f32 = lines[59].parse().unwrap();
    assert!((first - 0.2370201051235199f32).abs() < 1e-6);
    assert!((last - 0.1013089045882225f32).abs() < 1e-6);
    assert!(last < first && last < 1.0, "training converges");
    assert_eq!(lines[60], "torch.Size([4, 1])", "shape display");
    let pred0: f32 = lines[61].parse().unwrap();
    assert!((pred0 - 0.4728480279445648f32).abs() < 1e-5);
    eprintln!(
        "V-01 AV lane: {} lines in {} ms",
        lines.len(),
        started.elapsed().as_millis()
    );
}
