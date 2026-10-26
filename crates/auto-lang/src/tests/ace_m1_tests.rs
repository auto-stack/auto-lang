//! PLAN-852 T-02: the ace-m1 BASE corpus — evidence-green subset of the M1
//! conformance denominator — executes end to end through the unified runner
//! (real mn/av worker processes, real legacy VM runs, typed trap/diagnostic
//! channels), plus the negative oracle: a wrong exit expectation must be RED.
//!
//! Difference rows (legacy engines that disagree with the frozen oracle, e.g.
//! B08/B11 shadowing/named-arg binding) are deliberately NOT green checks —
//! they live in `binding.tsv` until their owners resolve them; this suite
//! never silently absorbs a gap. a2r emit-snapshot checks for value cases
//! join in T-03 with their reviewed goldens (T-07/K04 parse rejections are
//! already here: error completions need no oracle sidecar).

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use auto_test_case::report::Status;
use std::path::{Path, PathBuf};

/// Serialize worker-spawning suites (worker identity caches are
/// process-global; mirrors the ac-mn/ac-av discipline).
static ACE_M1_CASE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-ace-m1-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The committed base corpus runs END TO END green: 34 cases / 67 checks —
/// 10 scalar value cases on ac-native+av+autovm, 4 traps, 4 diagnostics,
/// 14 legacy-only composite values, 2 legacy parse rejections.
#[test]
fn ace_m1_base_suite_green() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("base");
    let out = root.join("out");
    let report = run_suite(
        &suite_dir.join("ace-m1-base.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("ace-m1 base suite runs");
    let failing: Vec<String> = report
        .results
        .iter()
        .filter(|r| r.status != Status::Pass)
        .map(|r| {
            format!(
                "{}:{} = {:?} | {:?} | mismatches: {:?}",
                r.case_id,
                r.check_id,
                r.status,
                r.detail,
                r.mismatches
            )
        })
        .collect();
    assert!(
        failing.is_empty(),
        "ace-m1 base suite has non-PASS checks:\n{}",
        failing.join("\n")
    );
    assert_eq!(report.results.len(), 67, "base suite check count");
    assert!(report.coverage_gaps.is_empty(), "no coverage gaps");
}

/// Negative oracle: mutating one frozen exit expectation (B01 14 -> 15) must
/// turn the gate RED — the suite is a real judge, never a green stamp.
#[test]
fn ace_m1_base_negative_oracle_fails() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("neg");
    let corpus = root.join("corpus");
    copy_tree(&suite_dir, &corpus);
    let scalar = corpus.join("b-scalar.case.ad");
    let text = std::fs::read_to_string(&scalar).unwrap();
    let mutated = text.replacen(
        "expect: { completion: \"success\", exit: 14 }",
        "expect: { completion: \"success\", exit: 15 }",
        1, // ac-native is the first check of B01 — the mutation is localized to it
    );
    assert_ne!(mutated, text, "mutation must hit the B01 oracle");
    std::fs::write(&scalar, mutated).unwrap();
    let report = run_suite(&corpus.join("ace-m1-base.suite.ad"), &root.join("out"), &Limits::defaults())
        .expect("mutated suite still runs");
    let b01_ac = report
        .results
        .iter()
        .find(|r| r.case_id == "ace.m1.b01" && r.check_id == "ac-native")
        .expect("B01 ac-native check present");
    assert_eq!(b01_ac.status, Status::Fail, "wrong exit must be RED");
    assert!(
        b01_ac
            .mismatches
            .iter()
            .any(|m| m.observation == "exit"),
        "the failure must name the exit axis: {:?}",
        b01_ac.mismatches
    );
    // Every other scalar ac check stays green — the mutation is localized.
    let other_reds: Vec<&str> = report
        .results
        .iter()
        .filter(|r| r.status != Status::Pass && !(r.case_id == "ace.m1.b01" && r.check_id == "ac-native"))
        .map(|r| r.case_id.as_str())
        .collect();
    assert!(other_reds.is_empty(), "unexpected extra reds: {:?}", other_reds);
}

/// Minimal recursive copy for the mutation fixture (the corpus is small;
/// symlinks never appear — the tree is plain files).
fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), &to).unwrap();
        }
    }
}
