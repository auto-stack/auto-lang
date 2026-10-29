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
        33,
        "tr-native suite check count (16 two-leg + 1 mn-only row; R07+R08 flipped to value by 881); got: {:?}",
        report
            .results
            .iter()
            .map(|r| format!("{}:{}", r.case_id, r.check_id))
            .collect::<Vec<_>>()
    );
    assert!(report.coverage_gaps.is_empty(), "no coverage gaps");
}

/// PLAN-881 T-05: the record/text-record METHOD suite — every method row
/// runs the real mn + av worker legs against the hand-audited oracles
/// (42/0/57/75/135/42/52/84 + R08 42), diagnostics carry the mn leg.
#[test]
fn ace_m1_method_881_suite_green() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("method-881");
    let out = root.join("out");
    let report = run_suite(
        &suite_dir.join("method-881.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("ace-m1 method-881 suite runs");
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
        "ace-m1 method-881 suite has non-PASS checks:
{}",
        failing.join("
")
    );
    assert_eq!(
        report.results.len(),
        22,
        "method-881 check count (9 two-leg value rows + 4 diagnostic rows)"
    );
    assert!(report.coverage_gaps.is_empty(), "no coverage gaps");
}

/// PLAN-865 T-05: the six original K sources bind to the closures source
/// recipe. K01/K02/K03/K05 execute; K04/K06 must fail at the declared source
/// diagnostics, with the source parent judge checking release accounting.
#[test]
fn ace_m1_closures_865_source_suite_green() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("closures-865-source");
    let corpus = root.join("corpus");
    copy_tree(&suite_dir, &corpus);
    let session_id = format!(
        "av_closures_865_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let case_path = corpus.join("closures-source-865.case.ad");
    let case_text = std::fs::read_to_string(&case_path).unwrap();
    let case_text = case_text.replace("av_closures_865_k01", &session_id);
    assert!(case_text.contains(&session_id), "session id substitution applies");
    std::fs::write(&case_path, case_text).unwrap();
    let report = run_suite(
        &corpus.join("closures-source-865.suite.ad"),
        &root.join("out"),
        &Limits::defaults(),
    )
    .expect("ace-m1 closures source suite runs");
    let failing: Vec<String> = report
        .results
        .iter()
        .filter(|result| result.status != Status::Pass)
        .map(|result| {
            format!(
                "{}:{} = {:?} | {:?} | mismatches: {:?}",
                result.case_id, result.check_id, result.status, result.detail, result.mismatches
            )
        })
        .collect();
    assert!(
        failing.is_empty(),
        "ace-m1 closures source suite has non-PASS checks:\n{}",
        failing.join("\n")
    );
    assert_eq!(report.results.len(), 8, "K01-K06 plus restart/edit rows all ran");
    assert!(report.coverage_gaps.is_empty(), "all six source cases are bound");

    let receipt_for = |check: &auto_test_case::report::CheckOutcome,
                       out_root: &Path|
     -> serde_json::Value {
          let path = std::fs::read_dir(out_root)
              .unwrap_or_else(|error| panic!("read {}: {error}", out_root.display()))
              .filter_map(Result::ok)
              .map(|run| {
                  run.path()
                      .join(&check.case_id)
                      .join(&check.check_id)
                      .join("probe-receipt.json")
              })
              .find(|path| path.is_file())
              .unwrap_or_else(|| {
                  panic!("{}:{} has no probe receipt under {}", check.case_id, check.check_id, out_root.display())
              });
          serde_json::from_slice(&std::fs::read(&path).unwrap_or_else(|error| {
              panic!("read {}: {error}", path.display())
          }))
          .unwrap_or_else(|error| panic!("decode {}: {error}", path.display()))
    };
    let result_for = |case_id: &str, check_id: &str| {
        report
            .results
            .iter()
            .find(|result| result.case_id == case_id && result.check_id == check_id)
            .unwrap_or_else(|| panic!("missing result {case_id}:{check_id}"))
    };

    let fresh = receipt_for(
        result_for("ace.m1.k01-av865", "av-k01-closures-source-fresh"),
        &root.join("out"),
    );
    assert_eq!(fresh["step"]["closures_source"]["cache_hits"], 0);
    assert_eq!(fresh["step"]["closures_source"]["cache_misses"], 2);
    assert_eq!(fresh["step"]["closures_source"]["store_writes"], 2);

    let restarted = receipt_for(
        result_for("ace.m1.k01-av865", "av-k01-closures-source-restart"),
        &root.join("out"),
    );
    assert_eq!(restarted["step"]["closures_source"]["cache_hits"], 2);
    assert_eq!(restarted["step"]["closures_source"]["cache_misses"], 0);
    assert_eq!(restarted["step"]["closures_source"]["store_writes"], 0);

    let body_edit = receipt_for(
        result_for(
            "ace.m1.k01-body-edit-av865",
            "av-k01-closures-source-body-edit",
        ),
        &root.join("out"),
    );
    assert_eq!(body_edit["step"]["closures_source"]["submitted_units"], 1);
    assert_eq!(body_edit["step"]["closures_source"]["verified"], 1);
    assert_eq!(body_edit["step"]["closures_source"]["encoded"], 1);
    assert_eq!(body_edit["step"]["closures_source"]["admitted"], 1);

    // Re-run the real TestCase source through fresh worker processes sharing
    // the same file-backed session. This supplies a bounded P90 sample of the
    // source+cache path, not an in-process microbenchmark.
    let sample_case = corpus.join("closures-source-cache-sample.case.ad");
    let sample_suite = corpus.join("closures-source-cache-sample.suite.ad");
    std::fs::write(
        &sample_case,
        format!(
            r#"```test-case
case {{ schema: "autolang.case/2", id: "ace.m1.k01-cache-sample",
  tags: ["ace-m1", "closures", "plan-865"], links: ["PLAN-865"],
  source: {{ file: "src/K01.at", syntax: "auto", path: "K01.at" }}, entry: {{ mode: "program" }},
  checks: [{{ id: "av-k01-cache-restart", adapter: "probe", action: "run",
    options: {{ recipe: "ac-av/closures-source-run-v1", av: {{ entry: "source:K01:restart", session: "{session_id}" }}, steps: [{{ op: "run" }}] }},
    expect: {{ completion: "success", exit: 42 }} }}]
}}
```"#
        ),
    )
    .unwrap();
    std::fs::write(
        &sample_suite,
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"ace-m1-closures-865-cache-sample\", include: [\"closures-source-cache-sample.case.ad\"] }\n```\n",
    )
    .unwrap();
    let mut latencies_ms = Vec::new();
    let mut peak_bytes = 0u64;
    let mut hot_bytes = 0u64;
    for sample in 0..20 {
        let out = root.join(format!("cache-sample-{sample}"));
        let sample_report = run_suite(&sample_suite, &out, &Limits::defaults())
            .expect("cache restart sample runs");
        assert_eq!(sample_report.results.len(), 1);
        assert_eq!(sample_report.results[0].status, Status::Pass);
        let receipt = receipt_for(&sample_report.results[0], &out);
        let face = &receipt["step"]["closures_source"];
        assert_eq!(face["cache_hits"], 2, "restart sample {sample}");
        assert_eq!(face["cache_misses"], 0, "restart sample {sample}");
        latencies_ms.push(
            receipt["request_wall_ms"]
                .as_f64()
                .expect("request_wall_ms recorded"),
        );
        peak_bytes = peak_bytes.max(
            face["process_peak_bytes"]
                .as_u64()
                .expect("process peak recorded"),
        );
        hot_bytes = hot_bytes.max(
            face["current_hot_bytes"]
                .as_u64()
                .expect("current hot bytes recorded")
                + face["history_hot_bytes"]
                    .as_u64()
                    .expect("history hot bytes recorded"),
        );
    }
    latencies_ms.sort_by(f64::total_cmp);
    let p90_index = ((latencies_ms.len() as f64 * 0.90).ceil() as usize).saturating_sub(1);
    let p90_ms = latencies_ms[p90_index];
    assert!(p90_ms <= 2000.0, "source cache P90 {p90_ms:.2}ms exceeds 2000ms");
    let evidence_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plans/attachments/865/work");
    std::fs::create_dir_all(&evidence_root).unwrap();
    std::fs::write(
        evidence_root.join("source-cache-metrics.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "samples": latencies_ms.len(),
            "p90_request_wall_ms": p90_ms,
            "max_process_peak_bytes": peak_bytes,
            "max_current_plus_history_hot_bytes": hot_bytes,
            "request_wall_ms": latencies_ms,
        }))
        .unwrap(),
    )
    .unwrap();
}

/// PLAN-881 V-08 counter-control: mutating one frozen oracle (M03 75 ->
/// 76) must turn the parent gate RED — the file suite is a real judge,
/// never a green stamp.
#[test]
fn ace_m1_method_881_negative_oracle_fails() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("method-881-neg");
    let corpus = root.join("corpus");
    copy_tree(&suite_dir, &corpus);
    let cases = corpus.join("method-881.case.ad");
    let text = std::fs::read_to_string(&cases).unwrap();
    // Flip BOTH legs' expected exits for the m03 case (75 -> 76), keeping
    // the fence delimiters intact.
    let mut rebuilt = String::new();
    for (i, sec) in text.split("```test-case").enumerate() {
        if i > 0 {
            rebuilt.push_str("```test-case");
        }
        if sec.contains("ace.m1.m03") {
            rebuilt.push_str(&sec.replace("exit: 75", "exit: 76"));
        } else {
            rebuilt.push_str(sec);
        }
    }
    std::fs::write(&cases, rebuilt).unwrap();
    let out = root.join("out");
    let report = run_suite(&corpus.join("method-881.suite.ad"), &out, &Limits::defaults())
        .expect("mutated suite still runs");
    let red: Vec<&str> = report
        .results
        .iter()
        .filter(|r| r.status != Status::Pass)
        .map(|r| r.case_id.as_str())
        .collect();
    assert!(
        red.contains(&"ace.m1.m03"),
        "the mutated oracle must redden m03 exactly: {red:?}"
    );
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
