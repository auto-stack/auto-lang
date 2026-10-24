//! PLAN-827: the av-source-827 corpus — REAL Auto source files (byte-
//! identical copies of the AP seeds, identity receipts in the corpus
//! README) through the formal `ac-av/core-run-v1` recipe, judged
//! PARENT-SIDE by the unified runner. Mirrors the ac-av corpus gate; the
//! recipe/worker/protocol are reused unchanged (T-01 reuse ruling — no
//! new recipe, worker command, or workspace).

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

pub(crate) static AV_SOURCE_827_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-av-src-827-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn out_of(root: &Path, sub: &str) -> PathBuf {
    root.parent().unwrap_or(Path::new(".")).join(format!(
        "{}-{}",
        root.file_name().unwrap().to_string_lossy(),
        sub
    ))
}

/// V-01/V-10 first case (formal path): the av-source-827 corpus runs END
/// TO END through the unified runner — a real av worker process, real
/// store on disk, parent-side oracle judgement. The suite's checked
/// population is asserted (>= 1 check now; grows with T-04/T-05 cases —
/// an empty or shrunken suite can never pass this gate silently).
#[test]
fn av_source_827_corpus_green() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../auto-lang/test/cases/av-source-827");
    let root = temp_root("corpus");
    let out = out_of(&root, "out");
    let report = run_suite(
        &suite_dir.join("av-source-827.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("av-source-827 suite runs");
    let failing: Vec<String> = report
        .results
        .iter()
        .filter(|r| r.status != auto_test_case::report::Status::Pass)
        .map(|r| {
            format!(
                "{}: {:?} {} | mismatches: {:?}",
                r.check_id,
                r.status,
                r.detail.clone().unwrap_or_default(),
                r.mismatches
            )
        })
        .collect();
    assert!(
        failing.is_empty(),
        "av-source-827 corpus all green ({} results): {:#?}",
        report.results.len(),
        failing
    );
    assert!(
        !report.results.is_empty(),
        "an empty corpus is not a pass — the suite must carry its checks"
    );
    let ids: Vec<&str> = report
        .results
        .iter()
        .map(|r| r.check_id.as_str())
        .collect();
    assert!(
        ids.contains(&"run-first"),
        "the first-case check must be present: {ids:?}"
    );
}
