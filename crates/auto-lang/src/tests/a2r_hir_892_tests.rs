//! PLAN-892 T-02 first registered case: B01 through the a2r-hir channel
//! (`ac-a2r/hir-scalar-emit-v1`, prebuilt a2x-worker, daily spawn-zero
//! emit, parent-side golden byte-compare) — plus the negative oracle (a
//! tampered golden must turn the check RED) and the missing-worker
//! control (infra ERROR carrying the maintenance hint, never a language
//! verdict).

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use auto_test_case::report::Status;
use std::path::{Path, PathBuf};

/// Serialize worker/env-dependent checks (process-global env + worker
/// caches; mirrors the ace-m1 discipline).
static A2R_HIR_CASE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/a2r-hir-892")
}

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-a2r-hir-892-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn failing(report: &auto_test_case::report::RunReport) -> Vec<String> {
    report
        .results
        .iter()
        .filter(|r| r.status != Status::Pass)
        .map(|r| {
            format!(
                "{}:{} = {:?} | {:?} | mismatches: {:?}",
                r.case_id, r.check_id, r.status, r.detail, r.mismatches
            )
        })
        .collect()
}

/// The FIRST registered a2r-hir case runs green end to end: the prebuilt
/// worker emits the four-file actual set on the spot, judged byte-wise
/// against the reviewed golden (1 case / 1 check; the suite never absorbs
/// a gap).
#[test]
fn a2r_hir_892_first_case_green() {
    let _guard = A2R_HIR_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // the maintenance build is an observable pre-consumer op, never on the
    // case timing path (plan §5.5): build the release worker now
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let manifest = repo.join("experimental/auto-a2x/Cargo.toml");
    assert!(manifest.is_file(), "a2x workspace present");
    let build = std::process::Command::new(env!("CARGO"))
        .args([
            "build",
            "--release",
            "-p",
            "a2x-probe",
            "--bin",
            "a2x-worker",
            "--manifest-path",
        ])
        .arg(&manifest)
        .current_dir(&repo)
        .output()
        .expect("maintenance worker build spawns");
    assert!(
        build.status.success(),
        "maintenance build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let root = temp_root("base");
    let report = run_suite(
        &suite_dir().join("a2r-hir-892.suite.ad"),
        &root.join("out"),
        &Limits::defaults(),
    )
    .expect("a2r-hir suite runs");
    assert_eq!(report.results.len(), 1, "exactly the first case's check");
    let bad = failing(&report);
    assert!(bad.is_empty(), "a2r-hir first case must be green:\n{}", bad.join("\n"));
    assert!(report.coverage_gaps.is_empty(), "no coverage gaps");
}

/// Negative oracle: one tampered golden byte must turn the check RED —
/// the channel is a real judge, never a green stamp.
#[test]
fn a2r_hir_892_tampered_golden_is_red() {
    let _guard = A2R_HIR_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = temp_root("neg");
    let suite_copy = root.join("cases");
    copy_dir(&suite_dir(), &suite_copy);
    // mutate one byte of the reviewed golden (comment head), keeping size
    let golden = suite_copy.join("golden/generated.rs");
    let mut bytes = std::fs::read(&golden).unwrap();
    assert!(bytes.len() > 16);
    bytes[10] = bytes[10].wrapping_add(1);
    std::fs::write(&golden, bytes).unwrap();
    let report = run_suite(
        &suite_copy.join("a2r-hir-892.suite.ad"),
        &root.join("out"),
        &Limits::defaults(),
    )
    .expect("tampered suite runs");
    assert!(
        !failing(&report).is_empty(),
        "a tampered golden must be RED (got all-pass!)"
    );
    assert_eq!(
        report.results[0].status,
        Status::Fail,
        "tampered golden is a language-axis FAIL (not infra)"
    );
}

/// Missing worker control: a broken AC_A2X_HIR_WORKER override is a
/// controlled infra ERROR with the maintenance hint — never a language
/// FAIL, and never an auto-build on the case path.
#[test]
fn a2r_hir_892_missing_worker_is_infra_error() {
    let _guard = A2R_HIR_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let bogus = temp_root("bogus-worker").join("no-such-worker.exe");
    std::env::set_var("AC_A2X_HIR_WORKER", &bogus);
    let report = run_suite(
        &suite_dir().join("a2r-hir-892.suite.ad"),
        &temp_root("infra").join("out"),
        &Limits::defaults(),
    );
    std::env::remove_var("AC_A2X_HIR_WORKER");
    match report {
        Ok(r) => {
            assert_eq!(
                r.results[0].status,
                Status::Error,
                "missing worker must be infra ERROR: {:?}",
                r.results[0].detail
            );
            assert!(
                r.results[0]
                    .detail
                    .as_deref()
                    .unwrap_or("")
                    .contains("missing file"),
                "error carries the override diagnosis"
            );
        }
        Err(e) => panic!("suite load must not fail for an infra refusal: {e}"),
    }
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), &to).unwrap();
        }
    }
}
