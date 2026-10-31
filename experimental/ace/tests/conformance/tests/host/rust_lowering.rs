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
    PathBuf::from(env!("AUTO_LANG_HOST_DIR")).join("test/cases/a2r-hir-892")
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
fn a2r_hir_first_case_green() {
    let _guard = A2R_HIR_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // the maintenance build is an observable pre-consumer op, never on the
    // case timing path (plan §5.5): build the release worker now
    let repo = PathBuf::from(env!("AUTO_LANG_HOST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    if std::env::var_os("AC_A2X_HIR_WORKER").is_none() {
        maintenance_build_worker(&repo);
    }

    let root = temp_root("base");
    let report = run_suite(
        &suite_dir().join("a2r-hir-892.suite.ad"),
        &root.join("out"),
        &Limits::defaults(),
    )
    .expect("a2r-hir suite runs");
    assert_eq!(report.results.len(), 5, "the five registered daily cases");
    let bad = failing(&report);
    assert!(
        bad.is_empty(),
        "a2r-hir first case must be green:\n{}",
        bad.join("\n")
    );
    assert!(report.coverage_gaps.is_empty(), "no coverage gaps");
}

/// Negative oracle: one tampered golden byte must turn the check RED —
/// the channel is a real judge, never a green stamp.
#[test]
fn a2r_hir_tampered_golden_is_red() {
    let _guard = A2R_HIR_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = temp_root("neg");
    let suite_copy = root.join("cases");
    copy_dir(&suite_dir(), &suite_copy);
    // mutate one byte of the reviewed golden (comment head), keeping size
    let golden = suite_copy.join("golden-B01/generated.rs");
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
fn a2r_hir_missing_worker_is_infra_error() {
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

/// ---------------------------------------------------------------------------
/// The EXPLICIT certification leg (plan §5.5): for every registered case,
/// the same on-the-spot actual set is REALLY compiled and run, the
/// hand-written independent reference compiles and runs separately, and
/// both must equal the closed-form oracle (value or named trap). A strict
/// certificate draft binding the full file set + reference + observed
/// toolchain is written per case.
struct CertifyCase {
    name: &'static str,
    profile: &'static str,
    caps: &'static [&'static str],
    oracle_kind: &'static str,
    oracle_value: Option<i64>,
    oracle_trap: Option<&'static str>,
}

/// The observable maintenance build (plan §5.5: outside case timing). Under
/// the parallel module runner several shards of this suite may build the
/// worker CONCURRENTLY while yet another shard has the exe running: a
/// relink can hit Windows' locked-file remove. Serialize cross-process via
/// a temp lock file, pin a stable toolchain env, and retry the relink once.
fn maintenance_build_worker(repo: &Path) {
    let lock = std::env::temp_dir().join("a2x-892-worker-build.lock");
    let mut guard = None;
    for _ in 0..240 {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
        {
            Ok(f) => {
                guard = Some(f);
                break;
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(250)),
        }
    }
    let _guard = guard;
    let manifest = repo.join("experimental/ace/Cargo.toml");
    assert!(manifest.is_file(), "a2x workspace present");
    let mut last_err = String::new();
    for attempt in 0..2 {
        let mut owned_compiler = std::process::Command::new(env!("CARGO"));
        owned_compiler
            .args([
                "build",
                "--release",
                "-p",
                "ace-a2x-worker",
                "--bin",
                "a2x-worker",
                "--manifest-path",
            ])
            .arg(&manifest)
            .env("RUSTC_WRAPPER", "")
            .current_dir(repo)
            .env_remove("CARGO_TARGET_DIR")
            .arg("--target-dir")
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"));
        let build = auto_tool_execution::run_command_output(owned_compiler)
            .expect("maintenance worker build spawns");
        if build.status.success() {
            let _ = std::fs::remove_file(&lock);
            return;
        }
        last_err = String::from_utf8_lossy(&build.stderr).to_string();
        std::thread::sleep(std::time::Duration::from_millis(500 * (attempt + 1)));
    }
    panic!("maintenance build failed (attempt {}): {}", 2, last_err);
}

fn worker_exe(repo: &Path) -> std::path::PathBuf {
    let exe = std::env::var_os("AC_A2X_HIR_WORKER")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo.join("experimental/ace/target/release")
                .join(if cfg!(windows) {
                    "a2x-worker.exe"
                } else {
                    "a2x-worker"
                })
        });
    assert!(exe.is_file(), "prebuilt worker at {}", exe.display());
    exe
}

fn run_certify(
    c: &CertifyCase,
    oracle_value_override: Option<i64>,
    reference_override: Option<&Path>,
) -> serde_json::Value {
    let repo = PathBuf::from(env!("AUTO_LANG_HOST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let dir = suite_dir();
    let root = temp_root(&format!("cert-{}", c.name));
    let req = serde_json::json!({
        "op": "certify",
        "source_path": dir.join(format!("src/{}.at", c.name)),
        "profile": c.profile,
        "entry": "main",
        "caps": c.caps,
        "out_dir": root.join("out"),
        "case_id": format!("a2r-hir-892/{}", c.name),
        "reference_path": reference_override.map(|p| p.to_path_buf())
            .unwrap_or_else(|| dir.join(format!("reference-{}.rs", c.name))),
        "oracle_kind": c.oracle_kind,
        "oracle_value": oracle_value_override.or(c.oracle_value).unwrap_or(0),
        "oracle_trap": c.oracle_trap.unwrap_or(""),
    });
    let out = std::process::Command::new(worker_exe(&repo))
        .arg(req.to_string())
        .output()
        .expect("spawn worker certify");
    assert!(
        out.status.success(),
        "worker certify failed: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    serde_json::from_slice(&out.stdout).expect("certify response json")
}

fn certify_family() -> Vec<CertifyCase> {
    vec![
        CertifyCase {
            name: "B01",
            profile: "core",
            caps: &[],
            oracle_kind: "value",
            oracle_value: Some(14),
            oracle_trap: None,
        },
        CertifyCase {
            name: "B04",
            profile: "scalar",
            caps: &[],
            oracle_kind: "value",
            oracle_value: Some(42),
            oracle_trap: None,
        },
        CertifyCase {
            name: "B13",
            profile: "core",
            caps: &[],
            oracle_kind: "trap",
            oracle_value: None,
            oracle_trap: Some("Overflow"),
        },
        CertifyCase {
            name: "B14",
            profile: "scalar",
            caps: &[],
            oracle_kind: "trap",
            oracle_value: None,
            oracle_trap: Some("DivByZero"),
        },
        CertifyCase {
            name: "NAMED-01",
            profile: "scalar",
            caps: &["hir.test.trace.v1"],
            oracle_kind: "value",
            oracle_value: Some(23),
            oracle_trap: None,
        },
    ]
}

#[test]
fn a2r_hir_certify_leg_all_cases_pass() {
    let _guard = A2R_HIR_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    for c in certify_family() {
        let resp = run_certify(&c, None, None);
        assert_eq!(
            resp["verdict"],
            "PASS",
            "{} certify: actual={:?} reference={:?} marks={}",
            c.name,
            resp["actual_observed"],
            resp["reference_observed"],
            resp["marks_match_reference"]
        );
        assert!(
            resp["cert_sha256"].is_string(),
            "{} binds a strict certificate",
            c.name
        );
        // 852 budget lines (observed walls, asserted here): each
        // compile+run leg under 120s; a leg contains both programs' work
        let wall: u64 = resp["actual_wall_ms"].as_u64().unwrap_or(u64::MAX);
        let rwall: u64 = resp["reference_wall_ms"].as_u64().unwrap_or(u64::MAX);
        assert!(
            wall < 120_000,
            "{} actual leg {}ms over budget",
            c.name,
            wall
        );
        assert!(
            rwall < 120_000,
            "{} reference leg {}ms over budget",
            c.name,
            rwall
        );
    }
}

/// V-09 shapes: a wrong oracle and a wrong independent reference are
/// REJECTED certifications — never a PASS by force.
#[test]
fn a2r_hir_certify_negative_controls_reject() {
    let _guard = A2R_HIR_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let family = certify_family();
    let b01 = &family[0];

    // wrong closed-form oracle (14 -> 15)
    let resp = run_certify(b01, Some(15), None);
    assert_eq!(resp["verdict"], "REJECTED", "wrong oracle must reject");

    // wrong independent reference (computes 15): the actual is right but
    // the reference disagrees -> the certification is still REJECTED
    let wrong_ref = temp_root("wrong-ref").join("reference.rs");
    let wrong = "fn main() { println!(\"value:{}\", 15); }
";
    std::fs::write(&wrong_ref, wrong).unwrap();
    let resp = run_certify(b01, None, Some(&wrong_ref));
    assert_eq!(resp["verdict"], "REJECTED", "wrong reference must reject");
}

/// F-2 repair control (review): a PINNED worker content credential that
/// disagrees with the observed binary is an infra ERROR — never a language
/// verdict; a matching pin passes.
#[test]
fn a2r_hir_worker_credential_pin() {
    let _guard = A2R_HIR_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let repo = PathBuf::from(env!("AUTO_LANG_HOST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let exe = worker_exe(&repo);
    let bytes = std::fs::read(&exe).unwrap();
    let real = auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(&bytes));
    let run = |pin: &str| {
        std::env::set_var("AC_A2X_HIR_WORKER_SHA256", pin);
        let r = run_suite(
            &suite_dir().join("a2r-hir-892.suite.ad"),
            &temp_root(&format!("pin-{}", pin.len())).join("out"),
            &Limits::defaults(),
        );
        std::env::remove_var("AC_A2X_HIR_WORKER_SHA256");
        r.expect("suite loads")
    };
    // wrong pin -> every check is infra ERROR with the credential diagnosis
    let wrong = run("deadbeef");
    assert_eq!(wrong.results[0].status, Status::Error, "wrong pin is infra");
    assert!(
        wrong.results[0]
            .detail
            .as_deref()
            .unwrap_or("")
            .contains("credential mismatch"),
        "diagnosis present: {:?}",
        wrong.results[0].detail
    );
    // matching pin -> green (the observed hash is accepted)
    let right = run(&real);
    assert!(
        right.results.iter().all(|r| r.status == Status::Pass),
        "matching pin passes"
    );
}
