//! PLAN-828 T-08 scaffold controls — the formal worker protocol over
//! REAL cross-process IPC (the persistent `session-serve` worker + the
//! parent-side judge), on a small fixture. The FORMAL measurement
//! campaign (frozen 273 575 B load, ≥30 revisions, the P95<1000 ms
//! verdict, dual-line memory budget) runs on the reviewed execution core
//! after the batch-2 re-review — these controls prove the protocol, the
//! judged window and every V-09 gate axis, not the campaign numbers.

use auto_av_probe::formal_worker::{run_formal_session_judge, JudgeConfig, JudgeEdit};

const SRC: &str = "fn helper(x int) int { let r = x * 3; return r + 7 }\nfn main() int { let a = helper(5); return a + 1 }\nfn tail() int { return 9 }\nfn extra() int { return 6 }\n";

fn worker_exe() -> std::path::PathBuf {
    // The real worker binary built with this test target.
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_av_probe_worker"))
}

fn small_config(gate_ms: f64) -> JudgeConfig {
    let x3 = SRC.find("x * 3").unwrap() as u64 + 4;
    let t9 = SRC.find("9").unwrap() as u64;
    JudgeConfig {
        src: SRC.to_string(),
        entry: "d_main".to_string(),
        caps: vec![],
        expected_initial: 23,
        edits: vec![
            JudgeEdit {
                start: x3,
                end: x3 + 1,
                insert: "4".into(),
                expected: 28,
                expected_install: "d_helper".into(),
            },
            JudgeEdit {
                start: x3,
                end: x3 + 1,
                insert: "5".into(),
                expected: 33,
                expected_install: "d_helper".into(),
            },
            JudgeEdit {
                start: x3,
                end: x3 + 1,
                insert: "3".into(),
                expected: 23,
                expected_install: "d_helper".into(),
            },
            // An edit of a DIFFERENT function: main's own result is
            // unchanged; the install set stays exactly that function.
            JudgeEdit {
                start: t9,
                end: t9 + 1,
                insert: "8".into(),
                expected: 23,
                expected_install: "d_tail".into(),
            },
        ],
        gate_ms,
    }
}

/// The judged window over real IPC: ready handshake (fresh session,
/// constant worker identity, independent initial expectation), every
/// edit published with the verified current value, structural gates
/// green (unchanged functions do zero stage work; one install per
/// revision), both memory lines sampled, raw stream complete.
#[test]
fn t08_judge_small_fixture_passes_over_real_ipc() {
    let cfg = small_config(5000.0); // scaffold: a loose bound, NOT the campaign gate
    let report =
        run_formal_session_judge(&worker_exe(), &cfg).expect("the judged series completes");
    assert!(report.failures.is_empty(), "{:?}", report.failures);
    assert!(report.all_correct);
    assert!(report.structural_ok);
    assert_eq!(report.samples.len(), 4, "one sample per scripted edit");
    assert!(report.gate_pass);
    assert!(
        report.session_id > 0,
        "the ready handshake carried the worker identity"
    );
    assert!(
        report.worker_peak_bytes > 0,
        "the worker memory line is sampled"
    );
    assert!(
        report.parent_peak_bytes > 0,
        "the parent memory line is sampled"
    );
    // The judged window includes IPC: strictly more than the worker's
    // own internal edit→result time.
    for s in &report.samples {
        assert!(s.edit_to_verified_ms >= 0.0);
    }
    // Raw observations: one JSONL line per sample, failures included.
    let raw = report.raw_jsonl();
    assert_eq!(raw.lines().count(), 4);
    assert!(raw.contains("\"req_id\":4"));
}

/// V-09 gate negatives — each axis reddens the gate on its own:
/// (a) `gate_ms == 0` must fail (a zero gate can never pass);
/// (b) a wrong independent expectation fails correctness and the gate;
/// (c) a lost worker stream mid-campaign leaves MISSING samples and the
///     completeness gate reddens instead of shrinking the campaign.
#[test]
fn t08_judge_gate_negatives_each_axis_red() {
    // (a) zero gate
    let report = run_formal_session_judge(&worker_exe(), &small_config(0.0))
        .expect("the series itself is healthy");
    assert!(!report.gate_pass, "gate-ms 0 must fail");
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.contains("not a positive bound")),
        "{:?}",
        report.failures
    );

    // (b) wrong expectation: the very first publication mismatches.
    let mut cfg = small_config(5000.0);
    cfg.edits[1].expected = 999;
    let report = run_formal_session_judge(&worker_exe(), &cfg).expect("series completes");
    assert!(!report.all_correct);
    assert!(!report.gate_pass);
    assert!(report
        .failures
        .iter()
        .any(|f| f.contains("independent expected")));

    // (c) missing samples: an edit whose coordinates are out of the
    // source kills the worker mid-stream; the judge records the loss and
    // the completeness gate fails — the campaign never silently shrinks.
    let mut cfg = small_config(5000.0);
    cfg.edits[2] = JudgeEdit {
        start: 1 << 40,
        end: (1 << 40) + 4,
        insert: "x".into(),
        expected: 23,
        expected_install: "d_helper".into(),
    };
    let report = run_formal_session_judge(&worker_exe(), &cfg).expect("judge survives the loss");
    assert!(!report.gate_pass);
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.contains("worker stream lost")),
        "{:?}",
        report.failures
    );
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.contains("missing samples")),
        "{:?}",
        report.failures
    );
}

/// The ready handshake itself: a wrong INITIAL expectation is caught
/// before any edit is judged (the judge never starts a campaign whose
/// first verified fact is already wrong).
#[test]
fn t08_ready_with_wrong_initial_expectation_fails_before_edits() {
    let mut cfg = small_config(5000.0);
    cfg.expected_initial = 999;
    let report = run_formal_session_judge(&worker_exe(), &cfg).expect("series completes");
    assert!(!report.gate_pass);
    assert!(
        report.failures.iter().any(|f| f.contains("ready value")),
        "{:?}",
        report.failures
    );
}

/// V-08 FORMAL CAMPAIGN over the persistent worker: the frozen
/// 273 575 B / 115-function load, 30 REAL body edits at fn_037's frozen
/// multiplier site, the judged window = edit ENQUEUE (IPC included) →
/// parent-verified publication. The dual gate: P95 < 1000 ms AND all
/// correct AND structural green. Initialization is excluded (single
/// listed in the report), the ready value is judged independently, and
/// the raw stream covers every sample. Dual-line memory peaks are
/// reported (counts into the F-09 budget evidence, not a gate here).
#[test]
fn t08_formal_campaign_v08_dual_gate_30_revisions() {
    use auto_av_probe::formal_campaign::{
        run_formal_campaign, CampaignConfig, FORMAL_GATE_MS, FORMAL_MIN_REVISIONS,
    };
    let cfg = CampaignConfig {
        revisions: FORMAL_MIN_REVISIONS,
        gate_ms: FORMAL_GATE_MS,
    };
    let report = run_formal_campaign(&worker_exe(), &cfg).expect("the formal campaign completes");
    let raw = report.raw_jsonl();
    assert!(
        raw.lines().count() >= FORMAL_MIN_REVISIONS as usize,
        "every sample preserved: {}",
        raw.lines().count()
    );
    assert!(report.failures.is_empty(), "{:?}", report.failures);
    assert!(
        report.all_correct,
        "every revision matched its independent expectation"
    );
    assert!(report.structural_ok);
    // B3-C02/C03: every publication bound to its expected revision and
    // install target, and every unchanged function dispatched the SAME
    // payload object across the whole campaign.
    assert!(report
        .samples
        .iter()
        .all(|s| s.revision_ok && s.install_ok && s.identity_ok));
    assert!(report
        .samples
        .iter()
        .all(|s| s.expected_install == "d_fn_037"));
    // B3-C04: the initialization window is single-listed and excluded.
    assert!(report.init_ms > 0.0);
    assert!(
        report.p95_ms < FORMAL_GATE_MS,
        "P95 {:.1} ms must be < {:.0} ms",
        report.p95_ms,
        FORMAL_GATE_MS
    );
    assert!(report.gate_pass);
    assert!(report.worker_peak_bytes > 0 && report.parent_peak_bytes > 0);
    // p50 <= p95 <= p99 sanity over the same samples.
    assert!(report.p50_ms <= report.p95_ms && report.p95_ms <= report.p99_ms);

    // B3-C04: the formal entry never generates-and-drops its raw — the
    // full report and every sample line (success and failure alike) are
    // written to a durable evidence directory and asserted.
    let evidence_dir = std::env::var_os("PLAN828_FORMAL_EVIDENCE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("828-formal-evidence")
        });
    let report_path = report
        .write_evidence(&evidence_dir)
        .expect("evidence written");
    let samples_path = evidence_dir.join("formal-samples.jsonl");
    assert!(report_path.is_file() && samples_path.is_file());
    let persisted = std::fs::read_to_string(&samples_path).expect("samples readable");
    assert_eq!(
        persisted.lines().count(),
        report.samples.len(),
        "every raw sample line persisted"
    );
    println!("formal evidence: {}", evidence_dir.display());

    // PLAN-828 T-07 dual-line budget verdict (798 r5 §5.9 口径, now
    // HARD-gated on the campaign's own measurements): Line A = the
    // frontend compile transient, measured in ISOLATION on the frozen
    // load in a fresh worker process (≤ 224 MiB, ac-core owned); Line B
    // = the probe-owned residual, overall worker sampled peak MINUS the
    // Line-A attribution (≤ 128 MiB — a missing attribution would
    // conservatively charge the whole peak to this line).
    let attribution = std::process::Command::new(worker_exe())
        .arg("frontend-attribution")
        .output()
        .expect("spawn frontend-attribution");
    assert!(attribution.status.success(), "attribution compile failed");
    let attr: serde_json::Value =
        serde_json::from_slice(&attribution.stdout).expect("attribution JSON line");
    let line_a = attr["frontend_peak_bytes"].as_u64().expect("peak bytes");
    assert_eq!(attr["ok"].as_bool(), Some(true));
    let line_a_budget = auto_av_probe::bench::BUDGET_FRONTEND_TRANSIENT_PEAK_BYTES;
    let line_b_budget = auto_av_probe::bench::BUDGET_WORKER_PRIVATE_PEAK_BYTES;
    assert!(
        line_a <= line_a_budget,
        "Line A (frontend transient) {line_a} > {line_a_budget}"
    );
    let line_b = report.worker_peak_bytes.saturating_sub(line_a);
    assert!(
        line_b <= line_b_budget,
        "Line B (probe-owned residual) {line_b} > {line_b_budget}"
    );
    println!(
        "formal: n={}, p50={:.1}ms p95={:.1}ms p99={:.1}ms, worker_peak={}MB parent_peak={}MB",
        report.samples.len(),
        report.p50_ms,
        report.p95_ms,
        report.p99_ms,
        report.worker_peak_bytes / (1024 * 1024),
        report.parent_peak_bytes / (1024 * 1024),
    );
    println!(
        "dual-line verdict: Line A (frontend transient, isolated)={}MB <= {}MB, Line B (probe residual)={}MB <= {}MB",
        line_a / (1024 * 1024),
        line_a_budget / (1024 * 1024),
        line_b / (1024 * 1024),
        line_b_budget / (1024 * 1024),
    );
}
