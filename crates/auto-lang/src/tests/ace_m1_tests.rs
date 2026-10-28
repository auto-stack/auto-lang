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

/// The committed base corpus runs END TO END green: 36 cases / 97 checks —
/// 10 scalar value cases on all FOUR paths (ac-native + av + autovm +
/// a2r-snapshot against reviewed goldens), 4 traps, 4 diagnostics, 14
/// legacy-only composite values (autovm + a2r snapshot), 2 legacy parse
/// rejections.
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
                r.case_id, r.check_id, r.status, r.detail, r.mismatches
            )
        })
        .collect();
    assert!(
        failing.is_empty(),
        "ace-m1 base suite has non-PASS checks:\n{}",
        failing.join("\n")
    );
    assert_eq!(
        report.results.len(),
        97,
        "base suite check count (incl. T-05 lifecycle)"
    );
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
    let report = run_suite(
        &corpus.join("ace-m1-base.suite.ad"),
        &root.join("out"),
        &Limits::defaults(),
    )
    .expect("mutated suite still runs");
    let b01_ac = report
        .results
        .iter()
        .find(|r| r.case_id == "ace.m1.b01" && r.check_id == "ac-native")
        .expect("B01 ac-native check present");
    assert_eq!(b01_ac.status, Status::Fail, "wrong exit must be RED");
    assert!(
        b01_ac.mismatches.iter().any(|m| m.observation == "exit"),
        "the failure must name the exit axis: {:?}",
        b01_ac.mismatches
    );
    // Every other scalar ac check stays green — the mutation is localized.
    let other_reds: Vec<&str> = report
        .results
        .iter()
        .filter(|r| {
            r.status != Status::Pass && !(r.case_id == "ace.m1.b01" && r.check_id == "ac-native")
        })
        .map(|r| r.case_id.as_str())
        .collect();
    assert!(
        other_reds.is_empty(),
        "unexpected extra reds: {:?}",
        other_reds
    );
}

/// Golden-drift control (PLAN-852 G04): when the emitted bytes stop matching
/// the frozen golden, the a2r-snapshot check must be RED — a drift is never
/// silently absorbed, and goldens are only updated through explicit review.
#[test]
fn ace_m1_golden_drift_is_red() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("drift");
    let corpus = root.join("corpus");
    copy_tree(&suite_dir, &corpus);
    let golden = corpus.join("oracle/m1_b01.rs");
    let bytes = std::fs::read(&golden).unwrap();
    // A visible, semantics-bearing mutation (not whitespace).
    let mutated: Vec<u8> = bytes
        .iter()
        .map(|&b| if b == b'+' { b'*' } else { b })
        .collect();
    assert_ne!(mutated, bytes, "mutation must change the golden");
    std::fs::write(&golden, mutated).unwrap();
    let report = run_suite(
        &corpus.join("ace-m1-base.suite.ad"),
        &root.join("out"),
        &Limits::defaults(),
    )
    .expect("mutated suite still runs");
    let snap = report
        .results
        .iter()
        .find(|r| r.case_id == "ace.m1.b01" && r.check_id == "a2r-snapshot")
        .expect("B01 a2r-snapshot check present");
    assert_eq!(snap.status, Status::Fail, "golden drift must be RED");
    assert!(
        snap.mismatches
            .iter()
            .any(|m| m.observation.contains("m1_b01.rs")),
        "the failure must name the drifted artifact: {:?}",
        snap.mismatches
    );
}

/// PLAN-870 T-05: the T*/R* record/text rows REALLY bound on the ac-native
/// (mn) path — every check green against the frozen oracle (the ac column
/// of binding.tsv flips from `gap:mn-scalar-only` to graded results). T05
/// is absent by the named upstream exclusion `gap:upstream-s-eq`.
#[test]
fn ace_m1_tr_native_suite_green() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("tr-native");
    let out = root.join("out");
    let report = run_suite(
        &suite_dir.join("ace-m1-tr-native.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("ace-m1 tr-native suite runs");
    let failing: Vec<String> = report
        .results
        .iter()
        .filter(|r| r.status != Status::Pass)
        .map(|r| {
            format!(
                "{}:{} = {:?} | {:?} | mismatches: {:?}",
                r.case_id, r.check_id, r.status, r.detail, r.mismatches
            )
        })
        .collect();
    assert!(
        failing.is_empty(),
        "ace-m1 tr-native suite has non-PASS checks:
{}",
        failing.join("
")
    );
    assert_eq!(
        report.results.len(),
        31,
        "tr-native suite check count (14 two-leg + 3 mn-only rows); got: {:?}",
        report
            .results
            .iter()
            .map(|r| format!("{}:{}", r.case_id, r.check_id))
            .collect::<Vec<_>>()
    );
    assert!(report.coverage_gaps.is_empty(), "no coverage gaps");
}

/// PLAN-870 V-06 counter-control: mutating one frozen exit expectation
/// (T01 38 -> 39) must turn the gate RED — the wiring is a real judge.
#[test]
fn ace_m1_tr_native_negative_oracle_fails() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("tr-native-neg");
    let corpus = root.join("corpus");
    copy_tree(&suite_dir, &corpus);
    let case = corpus.join("t-r-native.case.ad");
    let text = std::fs::read_to_string(&case).unwrap();
    let mutated = text.replace(
        "expect: { completion: \"success\", exit: 38 }",
        "expect: { completion: \"success\", exit: 39 }",
    );
    assert_ne!(&mutated, &text, "mutation must change the expectation");
    std::fs::write(&case, mutated).unwrap();
    let report = run_suite(
        &corpus.join("ace-m1-tr-native.suite.ad"),
        &root.join("out"),
        &Limits::defaults(),
    )
    .expect("mutated suite still runs");
    let t01 = report
        .results
        .iter()
        .find(|r| r.case_id == "ace.m1.t01" && r.check_id == "ac-native")
        .expect("T01 ac-native check present");
    assert_eq!(t01.status, Status::Fail, "mutated oracle must be RED");
}

/// PLAN-852 T-04: the explicit certification entry, driven directly. A fresh
/// run into a temp certs dir certifies every ok-emit row (actual==golden,
/// wrapper AND hand-written reference each print the fixed oracle through a
/// REAL rustc compile+run under containment); a tampered golden must FAIL
/// certification (drift is never re-certified silently — G04/G07), and a
/// structurally INVALID receipt is REJECTED — never reused, never silently
/// overwritten (F-852-R1-01, the reviewer's tamper variants replayed).
///
/// EXPLICIT-ONLY (F-852-R1-02): certification compiles and runs real Rust;
/// normal daily test selection must not trigger it. Invoke with
/// `cargo test ... ace_m1_certify -- --ignored`.
#[test]
#[ignore = "PLAN-852 F-852-R1-02: compiles+runs real Rust — certification is an explicit operation"]
fn ace_m1_certify_fresh_and_tamper() {
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("cert");
    let corpus = root.join("corpus");
    copy_tree(&suite_dir, &corpus);
    // binding.tsv drives the certifiable set (the 10 scalar ok-emit rows).
    let certs = root.join("certs");
    let args = crate::test_cases::ace_certify::CertifyArgs {
        corpus: corpus.clone(),
        certs_dir: certs.clone(),
        only: vec![],
        force: true,
        rustc: None,
        scratch: None,
    };
    let summaries =
        crate::test_cases::ace_certify::certify(&args).expect("certification pass runs");
    assert_eq!(summaries.len(), 10, "certifiable set size");
    for s in &summaries {
        assert_eq!(s.outcome, "certified", "{}: {}", s.case, s.detail);
    }
    assert_eq!(std::fs::read_dir(&certs).unwrap().count(), 10);

    // Tamper one golden byte: that case must FAIL and never rewrite the cert.
    let golden = corpus.join("oracle/m1_b01.rs");
    let bytes = std::fs::read(&golden).unwrap();
    let tampered: Vec<u8> = bytes
        .iter()
        .map(|&b| if b == b'+' { b'*' } else { b })
        .collect();
    std::fs::write(&golden, tampered).unwrap();
    let before = std::fs::read(&certs.join("m1_b01.cert.json")).unwrap();
    let summaries =
        crate::test_cases::ace_certify::certify(&args).expect("tampered pass still runs");
    let b01 = summaries
        .iter()
        .find(|s| s.case == "B01")
        .expect("B01 present");
    assert_eq!(b01.outcome, "fail", "golden drift must fail certification");
    assert!(b01.detail.contains("actual != golden"), "{}", b01.detail);
    let after = std::fs::read(&certs.join("m1_b01.cert.json")).unwrap();
    assert_eq!(
        before, after,
        "a failed certification never touches the cert"
    );

    // F-852-R1-01 replay: restore the golden, then TAMPER THE RECEIPT
    // (schema/flags/deps/observed — the reviewer's exact variants). Each
    // variant must be REJECTED as structurally invalid: never reused, never
    // silently overwritten — even --force refuses, demanding manual repair.
    std::fs::write(&golden, bytes).unwrap();
    let good_cert: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&before)).unwrap();
    for (path_in_cert, value) in [
        (vec!["schema"], serde_json::json!("invalid/999")),
        (
            vec!["toolchain", "flags"],
            serde_json::json!(["--wrong-flag"]),
        ),
        (
            vec!["deps", "closure"],
            serde_json::json!("changed-dependency"),
        ),
        (
            vec!["observed", "actual_run", "stdout"],
            serde_json::json!("i32:999"),
        ),
    ] {
        let mut tampered_cert = good_cert.clone();
        let mut cur = &mut tampered_cert;
        for key in &path_in_cert[..path_in_cert.len() - 1] {
            cur = cur.get_mut(key).expect("cert path exists");
        }
        cur[path_in_cert.last().unwrap()] = value;
        std::fs::write(
            &certs.join("m1_b01.cert.json"),
            serde_json::to_string_pretty(&tampered_cert).unwrap(),
        )
        .unwrap();
        for force in [false, true] {
            let args = crate::test_cases::ace_certify::CertifyArgs {
                corpus: corpus.clone(),
                certs_dir: certs.clone(),
                only: vec!["b01".to_string()],
                force,
                rustc: None,
                scratch: None,
            };
            let summaries = crate::test_cases::ace_certify::certify(&args).unwrap_or_else(|e| {
                panic!(
                    "certify errored for {:?} (force={force}): {e}",
                    path_in_cert
                )
            });
            let b01 = summaries.iter().find(|s| s.case == "B01").expect("B01");
            assert_eq!(
                b01.outcome, "fail",
                "tampered receipt ({:?}, force={force}) must be REJECTED: {}",
                path_in_cert, b01.detail
            );
            assert!(
                b01.detail.contains("REJECTED receipt"),
                "rejection detail: {}",
                b01.detail
            );
            let still_there = std::fs::read(&certs.join("m1_b01.cert.json")).unwrap();
            assert_ne!(
                before, still_there,
                "tampered receipt must NOT be silently overwritten"
            );
        }
    }
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
