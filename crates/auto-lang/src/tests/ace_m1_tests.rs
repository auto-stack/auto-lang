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
        failing.join(
            "
"
        )
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
        failing.join(
            "
"
        )
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
    assert!(
        case_text.contains(&session_id),
        "session id substitution applies"
    );
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
    assert_eq!(
        report.results.len(),
        8,
        "K01-K06 plus restart/edit rows all ran"
    );
    assert!(
        report.coverage_gaps.is_empty(),
        "all six source cases are bound"
    );

    let receipt_for =
        |check: &auto_test_case::report::CheckOutcome, out_root: &Path| -> serde_json::Value {
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
                    panic!(
                        "{}:{} has no probe receipt under {}",
                        check.case_id,
                        check.check_id,
                        out_root.display()
                    )
                });
            serde_json::from_slice(
                &std::fs::read(&path)
                    .unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
            )
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
        let sample_report =
            run_suite(&sample_suite, &out, &Limits::defaults()).expect("cache restart sample runs");
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
    assert!(
        p90_ms <= 2000.0,
        "source cache P90 {p90_ms:.2}ms exceeds 2000ms"
    );
    let evidence_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/plans/attachments/865/work");
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
    let report = run_suite(
        &corpus.join("method-881.suite.ad"),
        &out,
        &Limits::defaults(),
    )
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

/// PLAN-889 formal SDK: original 17 sources on both engines plus three owned
/// List<Variant> combinations on both, each judged against its own oracle.
#[test]
fn ace_m1_native_collections_889_source_sdk() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/cases/ace-m1/native-collections-889.suite.ad");
    let root = temp_root("native-collections-889");
    let report =
        run_suite(&suite, &root.join("out"), &Limits::defaults()).expect("889 formal source SDK");
    let failures: Vec<_> = report
        .results
        .iter()
        .filter(|r| r.status != Status::Pass)
        .map(|r| {
            format!(
                "{}:{} {:?} {:?} {:?}",
                r.case_id, r.check_id, r.status, r.detail, r.mismatches
            )
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(
        report.results.len(),
        40,
        "17 original native +17 original AV+3 combinations x2"
    );
    assert!(report.coverage_gaps.is_empty());
}

#[test]
fn ace_m1_native_collections_889_wrong_oracle_and_wrong_type_turn_red() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
    let root = temp_root("native-collections-889-negative");
    let corpus = root.join("corpus");
    copy_tree(&suite, &corpus);
    let file = corpus.join("native-collections-889.case.ad");
    let original = std::fs::read_to_string(&file).unwrap();
    let wrong = original.replacen(
        "expect: { completion: \"success\", exit: 42 }",
        "expect: { completion: \"success\", exit: 43 }",
        1,
    );
    assert_ne!(wrong, original);
    std::fs::write(&file, wrong).unwrap();
    let report = crate::test_cases::run_suite_selected(
        &corpus.join("native-collections-889.suite.ad"),
        &root.join("oracle-out"),
        &Limits::defaults(),
        &crate::test_cases::Selection {
            case: Some("ace.m1.l01-mn889".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        report
            .results
            .iter()
            .filter(|row| row.status != Status::NotRun)
            .count(),
        1
    );
    let wrongs: Vec<_> = report
        .results
        .iter()
        .filter(|r| r.status != Status::Pass && r.status != Status::NotRun)
        .collect();
    assert_eq!(wrongs.len(), 1, "{:?}", wrongs);
    assert_eq!(wrongs[0].case_id, "ace.m1.l01-mn889");
    assert_eq!(wrongs[0].status, Status::Fail);
    // Reuse the entire accepted suite but mutate a real original source in this
    // private fixture; both the native and AV judges must reject the wrong type.
    std::fs::write(&file, original).unwrap();
    let source = corpus.join("src/L01.at");
    let old = std::fs::read_to_string(&source).unwrap();
    let wrong = old.replacen("List<int>", "List<Str>", 1);
    assert_ne!(wrong, old);
    std::fs::write(source, wrong).unwrap();
    for id in ["ace.m1.l01-mn889", "ace.m1.l01-av864"] {
        let report = crate::test_cases::run_suite_selected(
            &corpus.join("native-collections-889.suite.ad"),
            &root.join(format!("type-{id}")),
            &Limits::defaults(),
            &crate::test_cases::Selection {
                case: Some(id.into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            report
                .results
                .iter()
                .filter(|row| row.status != Status::NotRun)
                .count(),
            1
        );
        assert_ne!(
            report
                .results
                .iter()
                .find(|row| row.status != Status::NotRun)
                .unwrap()
                .status,
            Status::Pass
        );
    }
    // A normal return70 has no trap marker and cannot satisfy original L06.
    std::fs::write(corpus.join("src/L06.at"), "fn main() int { return 70 }").unwrap();
    let report = crate::test_cases::run_suite_selected(
        &corpus.join("native-collections-889.suite.ad"),
        &root.join("normal70"),
        &Limits::defaults(),
        &crate::test_cases::Selection {
            case: Some("ace.m1.l06-mn889".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        report
            .results
            .iter()
            .filter(|row| row.status != Status::NotRun)
            .count(),
        1
    );
    assert_eq!(
        report
            .results
            .iter()
            .find(|row| row.status != Status::NotRun)
            .unwrap()
            .status,
        Status::Fail
    );
}

#[test]
fn ace_m1_native_collections_889_incremental_rounds_and_restore_sdk() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = temp_root("mn889-rounds");
    let corpus = root.join("corpus");
    std::fs::create_dir_all(&corpus).unwrap();
    std::fs::write(corpus.join("main.at"),"fn helper() int { var xs=List<Str>.new()\n xs.push(\"hello\")\n return xs.get(0).len() + 37 }\nfn main() int { return helper() }").unwrap();
    std::fs::write(corpus.join("rounds.case.ad"),r#"```test-case
case { schema:"autolang.case/2",id:"mn889-rounds",source:{file:"main.at",syntax:"auto",path:"main.at"},entry:{mode:"program"},checks:[
 {id:"mn",adapter:"probe",action:"run",options:{recipe:"ac/mir-native-collections-v1",mn:{entry:"d_main"},steps:[{op:"edit-source",from:"+ 37",to:"+ 38"},{op:"edit-source",from:"+ 38",to:"+ 37"},{op:"run"}]},expect:{completion:"success",exit:42}}
] }
```"#).unwrap();
    std::fs::write(
        corpus.join("rounds.suite.ad"),
        r#"```test-suite
suite {schema:"autolang.suite/2",id:"mn889-rounds",include:["rounds.case.ad"]}
```"#,
    )
    .unwrap();
    std::env::set_var("AC_MN_INCREMENTAL_EXPECT_EXITS", "43,42");
    let result = run_suite(
        &corpus.join("rounds.suite.ad"),
        &root.join("rounds-out"),
        &Limits::defaults(),
    );
    std::env::remove_var("AC_MN_INCREMENTAL_EXPECT_EXITS");
    let report = result.unwrap();
    assert_eq!(
        report
            .results
            .iter()
            .filter(|row| row.status != Status::NotRun)
            .count(),
        1
    );
    assert_eq!(
        report.results[0].status,
        Status::Pass,
        "{:?}",
        report.results[0]
    );
    // Restart is a real second worker and consumes the exact persisted closure.
    let text = std::fs::read_to_string(corpus.join("rounds.case.ad")).unwrap();
    let start = text.find("steps:[").unwrap();
    let end = text[start..].find("]},expect").unwrap() + start;
    let text = format!(
        "{}steps:[{{op:\"run\"}},{{op:\"restart\"}},{{op:\"run\"}}]{}",
        &text[..start],
        &text[end + 1..]
    );
    std::fs::write(corpus.join("rounds.case.ad"), text).unwrap();
    let report = run_suite(
        &corpus.join("rounds.suite.ad"),
        &root.join("restart-out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(
        report
            .results
            .iter()
            .filter(|row| row.status != Status::NotRun)
            .count(),
        1
    );
    assert_eq!(
        report.results[0].status,
        Status::Pass,
        "{:?}",
        report.results[0]
    );
    fn receipts(root: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(root).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                receipts(&entry.path(), out);
            } else if entry.file_name() == "mn-probe-receipt.json" {
                out.push(entry.path());
            }
        }
    }
    let mut files = vec![];
    receipts(&root.join("restart-out"), &mut files);
    assert_eq!(files.len(), 1);
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&files[0]).unwrap()).unwrap();
    let sessions = receipt["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 2);
    let counters = sessions[1]["counters"].as_array().unwrap();
    assert!(counters
        .iter()
        .any(|pair| pair[0] == "restored_mir_units_readmitted" && pair[1] == 2));
    assert!(!counters.iter().any(|pair| pair[0] == "codegen_emit_units"));
}

#[test]
fn ace_m1_native_collections_889_actual_missing_free_turns_parent_red() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/cases/ace-m1/native-collections-889.suite.ad");
    let root = temp_root("mn889-missing-free");
    std::env::set_var("AC_MN_COLLECTIONS_FAULT", "omit-frame-free");
    let result = crate::test_cases::run_suite_selected(
        &suite,
        &root.join("out"),
        &Limits::defaults(),
        &crate::test_cases::Selection {
            case: Some("ace.m1.l04-mn889".into()),
            ..Default::default()
        },
    );
    std::env::remove_var("AC_MN_COLLECTIONS_FAULT");
    let report = result.unwrap();
    assert_eq!(
        report
            .results
            .iter()
            .filter(|row| row.status != Status::NotRun)
            .count(),
        1
    );
    let row = report
        .results
        .iter()
        .find(|row| row.status != Status::NotRun)
        .unwrap();
    assert_eq!(row.status, Status::Error, "{row:?}");
    assert!(
        row.mismatches.iter().any(|m| m
            .actual
            .contains("native owned resource leak/invalid release")),
        "{row:?}"
    );
    fn evidence(root: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(root).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                evidence(&entry.path(), out);
            } else if entry.file_name() == ".mn889-omitted-free.json" {
                out.push(entry.path());
            }
        }
    }
    let mut files = vec![];
    evidence(&root, &mut files);
    assert_eq!(files.len(), 1);
    let fact: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&files[0]).unwrap()).unwrap();
    assert!(fact["omitted_native_returns"].as_u64().unwrap() > 0);
    assert_eq!(fact["codegen_units"], 1);
    assert_eq!(fact["exit_code"], 42);
    let raw =
        std::fs::read_to_string(files[0].with_file_name(".mn889-omitted-free.stderr")).unwrap();
    assert!(raw.lines().any(|line| line
        .strip_prefix("MN-COLL registered_bytes=")
        .and_then(|n| n.parse::<u64>().ok())
        .is_some_and(|n| n > 0)));
    println!(
        "real omitted native free evidence: {} {fact}",
        files[0].display()
    );
}

/// Dedicated serial measurement window. Never substitute this for source SDK
/// conformance; it writes every attempt before evaluating the five path gates.
#[test]
#[ignore = "PLAN-889 formal release-worker timing and512-revision retained campaign; run explicitly in a serial measurement window"]
fn ace_m1_native_collections_889_formal_performance_and_churn() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let source = include_str!("../../test/cases/ace-m1/src/N889Load.at");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plans/attachments/889/baseline")
        .join(format!(
            "formal-campaign-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    println!("889 formal campaign evidence {}", root.display());
    let result = crate::test_cases::ac_mn::run_collections_campaign(source, &root)
        .unwrap_or_else(|error| panic!("{error}; retained attempts: {}", root.display()));
    println!("889 formal result {result}");
    assert_eq!(result["complete"], true);
    assert_eq!(result["passed"], true, "{result}");
}

/// Dedicated serial measurement window. Never substitute this for source SDK
/// conformance; repair scope preserves full matrix evidence separately.
#[test]
#[ignore = "PLAN-889 affected fresh/restart timing repair; needs full-matrix reuse receipt"]
fn ace_m1_native_collections_889_formal_cold_verifier_repair() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let source = include_str!("../../test/cases/ace-m1/src/N889Load.at");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plans/attachments/889/baseline")
        .join(format!(
            "formal-cold-repair-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    println!("889 formal campaign evidence {}", root.display());
    let result = crate::test_cases::ac_mn::run_collections_cold_repair_campaign(source, &root)
        .unwrap_or_else(|error| panic!("{error}; retained attempts: {}", root.display()));
    println!("889 formal result {result}");
    assert_eq!(result["scope"], "cold-verifier-repair");
    assert_eq!(result["churn"], 0);
    assert_eq!(result["complete"], true);
    assert_eq!(result["passed"], true, "{result}");
}

#[test]
fn ace_m1_native_collections_889_report_runtime_content_identity_is_current() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/cases/ace-m1/native-collections-889.suite.ad");
    let root = temp_root("mn889-stale");
    // Each nextest case is a separate process. The in-process case mutex
    // cannot protect a shared product library against sibling processes.
    // Mirror only the runtime inputs/configuration and its attested output;
    // the real worker still prepares/hashes/adopts this private library.
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let runtime_rel = Path::new("experimental/ac-mir-native/runtime/collections");
    let text_rel = Path::new("experimental/ac-core/runtime/text/shim");
    let lib_rel =
        runtime_rel.join("target/x86_64-pc-windows-msvc/release/ac_collections_runtime.lib");
    let shared_library = repo.join(&lib_rel);
    // This case must prepare its own attested input: nextest can run it
    // before every other collections test on a clean worktree.
    let prepared = crate::test_cases::ac_mn::current_collections_toolchain_identity();
    assert!(
        prepared.contains_key("collections-runtime-content"),
        "collections runtime preparation failed: {prepared:?}"
    );
    let shared_bytes = std::fs::read(&shared_library).unwrap();
    let mirror = root.join("runtime-mirror");
    for relative in [runtime_rel, text_rel] {
        std::fs::create_dir_all(mirror.join(relative)).unwrap();
        copy_tree(
            &repo.join(relative).join("src"),
            &mirror.join(relative).join("src"),
        );
        for name in ["Cargo.toml", "Cargo.lock"] {
            let from = repo.join(relative).join(name);
            if from.is_file() {
                std::fs::copy(from, mirror.join(relative).join(name)).unwrap();
            }
        }
    }
    std::fs::create_dir_all(mirror.join(".cargo")).unwrap();
    std::fs::copy(
        repo.join(".cargo/config.toml"),
        mirror.join(".cargo/config.toml"),
    )
    .unwrap();
    let library = mirror.join(&lib_rel);
    std::fs::create_dir_all(library.parent().unwrap()).unwrap();
    std::fs::copy(&shared_library, &library).unwrap();
    std::fs::copy(
        shared_library.with_extension("identity.json"),
        library.with_extension("identity.json"),
    )
    .unwrap();
    struct RestoreRoot(Option<std::ffi::OsString>);
    impl Drop for RestoreRoot {
        fn drop(&mut self) {
            if let Some(old) = self.0.take() {
                std::env::set_var("AUTO_LANG_ROOT", old);
            } else {
                std::env::remove_var("AUTO_LANG_ROOT");
            }
        }
    }
    let _restore_root = RestoreRoot(std::env::var_os("AUTO_LANG_ROOT"));
    std::env::set_var("AUTO_LANG_ROOT", &mirror);
    let report = crate::test_cases::run_suite_selected(
        &suite,
        &root.join("out"),
        &Limits::defaults(),
        &crate::test_cases::Selection {
            case: Some("ace.m1.l04-mn889".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let row = report
        .results
        .iter()
        .find(|row| row.case_id == "ace.m1.l04-mn889")
        .unwrap();
    assert_eq!(row.status, Status::Pass, "{row:?}");
    assert_eq!(row.native.as_ref().unwrap().toolchain.len(), 9);
    let value: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
    let check = || {
        crate::test_cases::stale::check(&suite, &value)
            .unwrap()
            .into_iter()
            .find(|row| match row {
                crate::test_cases::stale::RowVerdict::Fresh { case, .. }
                | crate::test_cases::stale::RowVerdict::Stale { case, .. } => {
                    case == "ace.m1.l04-mn889"
                }
            })
            .unwrap()
    };
    assert!(matches!(
        check(),
        crate::test_cases::stale::RowVerdict::Fresh { .. }
    ));

    struct RestoreFile {
        path: PathBuf,
        bytes: Vec<u8>,
    }
    impl Drop for RestoreFile {
        fn drop(&mut self) {
            std::fs::write(&self.path, &self.bytes).expect("restore owned runtime content");
        }
    }
    let saved = RestoreFile {
        bytes: std::fs::read(&library).unwrap(),
        path: library.clone(),
    };
    let mut wrong = saved.bytes.clone();
    let middle = wrong.len() / 2;
    wrong[middle] ^= 1;
    std::fs::write(&library, &wrong).unwrap();
    assert_eq!(
        std::fs::metadata(&library).unwrap().len(),
        saved.bytes.len() as u64
    );
    let stale = check();
    assert!(
        matches!(&stale,crate::test_cases::stale::RowVerdict::Stale{drifted_axes,..} if drifted_axes.iter().any(|axis|axis.contains("collections-runtime-content"))),
        "{stale:?}"
    );
    drop(saved);
    assert!(matches!(
        check(),
        crate::test_cases::stale::RowVerdict::Fresh { .. }
    ));
    assert!(
        std::fs::read(&shared_library).unwrap() == shared_bytes,
        "identity control modified the shared product library"
    );
}

#[test]
fn ace_m1_native_collections_889_real_whole_work_and_code_copy_turn_sdk_red() {
    let _guard = ACE_M1_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = temp_root("mn889-real-degeneration");
    let corpus = root.join("corpus");
    std::fs::create_dir_all(&corpus).unwrap();
    std::fs::write(corpus.join("main.at"),"fn helper() int { var xs=List<Str>.new()\n xs.push(\"hello\")\n return xs.get(0).len() + 37 }\nfn main() int { return helper() }").unwrap();
    std::fs::write(corpus.join("case.ad"),r#"```test-case
case { schema:"autolang.case/2",id:"mn889-degeneration",source:{file:"main.at",syntax:"auto",path:"main.at"},entry:{mode:"program"},checks:[
 {id:"mn",adapter:"probe",action:"run",options:{recipe:"ac/mir-native-collections-v1",mn:{entry:"d_main"},steps:[{op:"edit-source",from:"+ 37",to:"+ 38"},{op:"run"}]},expect:{completion:"success",exit:43}}
] }
```"#).unwrap();
    std::fs::write(
        corpus.join("suite.ad"),
        r#"```test-suite
suite {schema:"autolang.suite/2",id:"mn889-degeneration",include:["case.ad"]}
```"#,
    )
    .unwrap();
    for (fault, actual) in [
        ("whole-front", "actual parsed"),
        ("whole-lower", "negative_control_lower:main"),
        ("copy-unchanged", "negative_control_same_content_copy"),
    ] {
        std::env::set_var("AC_MN_COLLECTIONS_FAULT", fault);
        let result = run_suite(
            &corpus.join("suite.ad"),
            &root.join(fault),
            &Limits::defaults(),
        );
        std::env::remove_var("AC_MN_COLLECTIONS_FAULT");
        let report = result.unwrap();
        assert_eq!(report.results.len(), 1);
        let row = &report.results[0];
        assert_eq!(row.status, Status::Error, "{row:?}");
        assert!(
            row.mismatches.iter().any(|m| m.actual.contains(actual)),
            "actual degeneration was not observed for {fault}: {row:?}"
        );
    }
}
