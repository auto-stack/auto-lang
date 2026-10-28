//! PLAN-866 file-backed resident-online corpus: one continuously-running
//! guest chain (fuel slices, never restarted) with edits installed at
//! guest call-boundary safety points — END-TO-END through the unified
//! runner (a real av worker process, parent-side oracle judgement; the
//! worker never sees an expectation).

use auto_lang_core::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "av866-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("scratch root");
    root
}

fn out_of(root: &Path, sub: &str) -> PathBuf {
    let out = root.join(sub);
    std::fs::create_dir_all(&out).expect("scratch out");
    out
}

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1")
}

/// AC-01/AC-02/AC-06: the file-backed resident corpus runs green — three
/// checks (R01 continuity 2100; R02 one online swap with the audited
/// old/new split; R03 two-swap churn), all PASS, no load errors, no
/// duplicated ids — with the structural faces (chain entries == 1,
/// unaffected stages == 0, whole-module == 0) judged parent-side.
#[test]
fn av_resident_866_corpus_green() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("corpus");
    let out = out_of(&root, "out");
    let report = run_suite(
        &suite_dir().join("resident-online-866.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("resident-866 suite runs");
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
        "resident-866 corpus must be all green: {failing:#?}"
    );
    assert_eq!(
        report.results.len(),
        3,
        "three bound rows (R01/R02/R03), no more, no fewer"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Discrimination control: a sabotaged exit oracle (off by the exact
/// per-round delta of one new call) must REDDEN the online-swap row —
/// proving the judge actually compares the value, and that the split is
/// exact to the round, not "eventually changed".
#[test]
fn av_resident_866_wrong_exit_oracle_reddens() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("control");
    let case_dir = out_of(&root, "case");
    let src_dir = out_of(&case_dir, "src");
    std::fs::copy(
        suite_dir().join("src/RES866A.at"),
        src_dir.join("RES866A.at"),
    )
    .expect("copy source");
    let case_text = std::fs::read_to_string(suite_dir().join("resident-online-866.case.ad"))
        .expect("read case");
    // R02's frozen oracle minus 2 == one round ran the OLD body where the
    // case says it ran the new one: an implementation that swaps one call
    // later than the frozen boundary would produce exactly this value.
    // The frozen value is read from the committed case (single source of
    // truth): parse the R02 case block's expect line.
    // Inner case text only (up to the closing fence) — the control
    // document re-fences it itself.
    let r02_block: String = case_text
        .split("```test-case")
        .find(|b| b.contains("av-res-r02-online-swap"))
        .expect("R02 block")
        .split("```")
        .next()
        .unwrap_or_default()
        .to_string();
    assert!(r02_block.contains("exit:"), "R02 block text extracted");
    let frozen: i32 = r02_block
        .lines()
        .find_map(|l| {
            l.trim()
                .strip_prefix("expect: { completion: \"success\", exit: ")
        })
        .and_then(|rest| {
            rest.split_whitespace()
                .next()
                .and_then(|tok| tok.parse().ok())
        })
        .expect("R02 frozen exit");
    assert!(
        frozen > 0,
        "the frozen oracle must be committed, got {frozen}"
    );
    let sabotaged = frozen - 2;
    let control_text = r02_block.replace(
        &format!("exit: {frozen} }}"),
        &format!("exit: {sabotaged} }}"),
    );
    assert_ne!(
        control_text, r02_block,
        "the sabotage must actually change the oracle"
    );
    std::fs::write(
        case_dir.join("resident-control.case.ad"),
        format!("```test-case{control_text}```"),
    )
    .expect("write control case");
    std::fs::write(
        case_dir.join("resident-control.suite.ad"),
        "# control suite\n\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"ace-m1-resident-866-ctl\", include: [\"resident-control.case.ad\"] }\n```\n",
    )
    .expect("write control suite");
    let out = out_of(&root, "out");
    let report = run_suite(
        &case_dir.join("resident-control.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("control suite runs");
    let red = report
        .results
        .iter()
        .any(|r| r.status == auto_test_case::report::Status::Fail);
    assert!(
        red,
        "a wrong exit oracle must redden the online-swap row: {:#?}",
        report
            .results
            .iter()
            .map(|r| (r.check_id.clone(), r.status))
            .collect::<Vec<_>>()
    );
    let _ = std::fs::remove_dir_all(&root);
}
