//! PLAN-827 T-09 (V-13): hot edits over this plan's source entry reuse the
//! D828 session (read-only consumption of the delivered public API) —
//! never a new Driver / whole-source recompile per edit. Four sub-cases
//! per the ACE contract:
//! (1.1) the D828 frozen baseline (273 575 B / 115 functions) x >=30
//!       distinct body revisions through THIS consumer, dual gate:
//!       structure (unchanged functions' stages all-zero, exactly one
//!       install per edit) AND edit->verified-result P90 <= 2000 ms;
//! (1.2) two-function local body updates on the AP-01-derived entry —
//!       only the edited unit re-keys/installs, the other function's
//!       stages stay zero, results/semantics propagate correctly;
//! (1.3) precise invalidation/refusal: a signature/interface edit and an
//!       unauthorized capability edit are REFUSED with the state
//!       untouched (revision preserved, prior snapshot still correct);
//! (1.4) degradation is CAUGHT: a fast-path edit records no fallback,
//!       and a non-fast-path edit is REFUSED WITH a recorded reason —
//!       never a silent whole-module recompute (store-side hash-preserving
//!       object swaps are Conflict-guarded by the T-05 cache tests).

use ace_av::session::{warmup, FunctionIncrementalSession};
use ace_av_core::av::{CancelFlag, ExecLimits};
use ace_av_core::contract::ExecOutcome;

const AP01_ORIGINAL: &str = "../core/fixtures/scalar-source/valid/p-gcd-euclid.at";
const AP01_SHA256: &str = "6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3";

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn ap01() -> String {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(AP01_ORIGINAL))
        .expect("read AP-01");
    assert_eq!(sha256_hex(&bytes), AP01_SHA256, "AP-01 drifted");
    String::from_utf8(bytes).unwrap()
}

fn edit_once(src: &str, from: &str, to: &str) -> (u64, u64, Vec<u8>) {
    let hits = src.matches(from).count();
    assert_eq!(hits, 1, "edit literal `{from}` matched {hits} times");
    let start = src.find(from).expect("literal") as u64;
    let end = start + from.len() as u64;
    (start, end, to.as_bytes().to_vec())
}

fn run_value(s: &FunctionIncrementalSession) -> i32 {
    let cancel = CancelFlag::new(false);
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    match out {
        ExecOutcome::Success { value, .. } => value.as_i32().expect("entry returns i32"),
        other => panic!("session run not a success: {other:?}"),
    }
}

fn percentile(vals: &[f64], p: f64) -> f64 {
    let mut s = vals.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((p / 100.0) * (s.len() - 1) as f64).round() as usize % s.len()]
}

/// R1-F02: the STRUCTURE GATE as an explicit predicate — the same rules
/// the assertions below enforce inline, so the forced-degradation control
/// can prove the DETECTOR fires (a bad consumer cannot pass it silently).
/// Red iff more than one function gained a `mir_lower` stage, or any
/// module-level non-syntax counter grew (whole frontend/reload shapes).
fn whole_module_findings(
    before: &std::collections::BTreeMap<String, u64>,
    after: &std::collections::BTreeMap<String, u64>,
) -> Vec<String> {
    let mut lowered: Vec<String> = Vec::new();
    let mut findings: Vec<String> = Vec::new();
    for (k, v) in after {
        let bv = before.get(k).copied().unwrap_or(0);
        if *v <= bv {
            continue;
        }
        if let Some(fn_name) = k.strip_prefix("mir_lower:") {
            lowered.push(fn_name.to_string());
        } else if !k.contains(':') && !k.starts_with("syntax_") {
            findings.push(format!("module-level counter `{k}` grew"));
        }
    }
    if lowered.len() > 1 {
        findings.push(format!(
            "{} functions re-lowered in one step (whole-module shape)",
            lowered.len()
        ));
    }
    findings
}

/// R1-F02: resident object identity of one function — (Arc pointer,
/// generation) from the CURRENT snapshot: the identity axes the formal
/// gate binds (a swapped object with equal content hash still moves the
/// pointer; a re-published generation still moves the number).
fn resident_identity(s: &FunctionIncrementalSession, def_id: &str) -> Option<(usize, u64)> {
    let snap = s.current();
    let code = snap.find_code(def_id)?;
    let ptr = std::sync::Arc::as_ptr(code) as usize;
    let gen = snap.generation_of(def_id)?;
    Some((ptr, gen))
}

/// R7 F-827-R7-01: a PER-RUN durable raw recorder. The run file is created
/// and written BEFORE any assertion can discard evidence — each revision is
/// persisted as soon as it is observed (that revision's assertions run
/// AFTER its row is on disk), and the final structure/P95 verdict is
/// persisted BEFORE the P95 assertion. A failing revision or time gate
/// therefore always leaves its raw on disk, and per-run filenames mean a
/// later run never overwrites an earlier run's evidence. The fixed-name
/// `incremental-raw.json` remains only as a pointer copy of the LAST
/// COMPLETED run (its `run_id` names the immutable evidence file).
struct IncrementalRawRun {
    path: std::path::PathBuf,
    doc: serde_json::Value,
    /// false for the controlled-negative location: a negative never updates
    /// the "last completed run" pointer.
    update_latest: bool,
}

fn report_file(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../docs/reports/plan827")
        .join(rel)
}

impl IncrementalRawRun {
    fn start(samples: usize, run_id: &str) -> Self {
        let mut run = Self::start_at(samples, run_id, &format!("incremental-raw-{run_id}.json"));
        run.update_latest = true;
        run
    }

    fn start_at(samples: usize, run_id: &str, rel: &str) -> Self {
        let path = report_file(rel);
        std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create report dir");
        let doc = serde_json::json!({
            "scope": "incremental-raw",
            "run_id": run_id,
            "samples": samples,
            "verdict": { "status": "in-progress" },
            "revisions": [],
        });
        let run = Self {
            path,
            doc,
            update_latest: false,
        };
        run.write();
        run
    }

    fn push_revision(&mut self, rev: serde_json::Value) {
        self.doc["revisions"]
            .as_array_mut()
            .expect("revisions array")
            .push(rev);
        self.write();
    }

    /// Persist the final verdict BEFORE the caller asserts on it, so a red
    /// gate is recorded on disk exactly as observed.
    fn finish(&mut self, verdict: serde_json::Value) {
        self.doc["verdict"] = verdict;
        self.write();
        if self.update_latest {
            let mut latest = self.doc.clone();
            latest["latest_completed"] = serde_json::Value::Bool(true);
            std::fs::write(
                report_file("incremental-raw.json"),
                serde_json::to_string_pretty(&latest).expect("serialize latest"),
            )
            .unwrap_or_else(|e| panic!("write latest raw: {e}"));
        }
    }

    fn write(&self) {
        std::fs::write(
            &self.path,
            serde_json::to_string_pretty(&self.doc).expect("serialize raw"),
        )
        .unwrap_or_else(|e| panic!("write raw {}: {e}", self.path.display()));
    }
}

/// R7 F-827-R7-01: the persisted-raw SHAPE contract as a predicate — every
/// revision must carry the complete observation, and the unchanged sample's
/// identity baseline and current value must be CONCRETE (ptr, generation)
/// pairs. A null/missing `before` (the shipped defect: 30 rows looked up
/// `d_main` against a bare-name-keyed map) or a missing/short `after` is
/// REJECTED, never silently persisted.
fn validate_raw_shape(doc: &serde_json::Value) -> Result<(), String> {
    let revisions = doc["revisions"]
        .as_array()
        .ok_or_else(|| "raw has no revisions array".to_string())?;
    if revisions.is_empty() {
        return Err("raw has zero revisions".into());
    }
    for rev in revisions {
        let n = rev["revision"].as_u64().unwrap_or(0);
        for field in ["changed_fn", "install_set", "value", "wall_ms"] {
            if rev[field].is_null() {
                return Err(format!("revision {n}: `{field}` is null"));
            }
        }
        let hold = &rev["identity_hold"]["unchanged_sample"];
        let name = hold["name"].as_str().unwrap_or("");
        for axis in ["before", "after"] {
            let pair = hold[axis].as_array().cloned().unwrap_or_default();
            if pair.len() != 2 || !pair.iter().all(|v| v.is_u64()) {
                return Err(format!(
                    "revision {n}: unchanged sample `{name}` axis `{axis}` is not a concrete (ptr, generation) pair"
                ));
            }
        }
    }
    Ok(())
}

/// R7 F-827-R7-01 negative: the persisted-raw SHAPE contract rejects a
/// missing/null identity baseline (the exact defect all 30 shipped rows
/// carried) and an absent/degenerate current pair; a complete row passes.
#[test]
fn av_source_incremental_raw_shape_rejects_missing_baseline() {
    let good = serde_json::json!({
        "revisions": [{
            "revision": 1,
            "changed_fn": "gcd",
            "install_set": ["d_gcd"],
            "value": 1,
            "wall_ms": 12.5,
            "identity_hold": {
                "changed_moved": true,
                "unchanged_sample": {
                    "name": "d_main",
                    "before": [7011024, 3],
                    "after": [7011024, 3],
                },
            },
        }]
    });
    assert!(
        validate_raw_shape(&good).is_ok(),
        "a complete row is accepted"
    );

    // The shipped defect shape: `before` null (keyed `d_main` against a
    // bare-name-keyed map) must be REJECTED, with the axis named.
    let mut doc = good.clone();
    doc["revisions"][0]["identity_hold"]["unchanged_sample"]["before"] = serde_json::Value::Null;
    let err = validate_raw_shape(&doc).expect_err("a null baseline must be rejected");
    assert!(err.contains("before"), "error names the axis: {err}");

    // Missing current pair / short pair / zero revisions: all rejected.
    let mut doc = good.clone();
    doc["revisions"][0]["identity_hold"]["unchanged_sample"]
        .as_object_mut()
        .expect("sample object")
        .remove("after");
    validate_raw_shape(&doc).expect_err("a missing current pair must be rejected");

    let mut doc = good.clone();
    doc["revisions"][0]["identity_hold"]["unchanged_sample"]["after"] = serde_json::json!([42]);
    validate_raw_shape(&doc).expect_err("a 1-element pair must be rejected");

    let empty = serde_json::json!({ "revisions": [] });
    validate_raw_shape(&empty).expect_err("zero revisions must be rejected");
}

/// R7 F-827-R7-01 negative: a CONTROLLED FAILING VERDICT still leaves its
/// per-run raw on disk — the recorder's write-before-assert contract, the
/// exact path a red revision/P95 takes in the campaign test. The negative
/// writes to its own negatives/ location and never touches a real campaign
/// run file or the latest-completed pointer.
#[test]
fn av_source_incremental_failing_verdict_still_writes_raw() {
    let revision = |n: u64, wall_ms: f64| {
        serde_json::json!({
            "revision": n,
            "changed_fn": "gcd",
            "install_set": ["d_gcd"],
            "value": 1,
            "wall_ms": wall_ms,
            "identity_hold": {
                "changed_moved": true,
                "unchanged_sample": {
                    "name": "d_main",
                    "before": [7011024, 3],
                    "after": [7011024, 3],
                },
            },
        })
    };
    let mut run = IncrementalRawRun::start_at(
        2,
        "negative-failing-verdict",
        "negatives/incremental-failing-verdict.json",
    );
    run.push_revision(revision(1, 900.0));
    run.push_revision(revision(2, 2100.0));
    run.finish(serde_json::json!({
        "status": "fail",
        "reason": "controlled negative: P90 exceeded the 2000ms gate",
        "structure": "pass",
        "p90_ms": 2100.0,
        "p95_ms": 2100.0,
        "time_gate_ms": 2000.0,
        "time_gate": false,
    }));
    // The failing run's raw EXISTS on disk, parses, and carries the failing
    // verdict plus both observed revisions.
    let doc: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(report_file("negatives/incremental-failing-verdict.json"))
            .expect("the failing run left its raw on disk"),
    )
    .expect("parse the retained raw");
    assert_eq!(
        doc["verdict"]["status"], "fail",
        "the failing verdict is persisted"
    );
    assert_eq!(doc["verdict"]["time_gate"], false);
    assert_eq!(
        doc["revisions"].as_array().map(Vec::len),
        Some(2),
        "both observed revisions are retained"
    );
    assert!(validate_raw_shape(&doc).is_ok());
}

/// V-13 (1.2): two-function local body updates on the REAL AP-01 entry —
/// only the edited function re-keys/installs; the other's stages stay
/// zero; semantics propagate through the shared pipeline.
/// (Shape note: main's trailing `return 0` replacement is REFUSED by the
/// session's incremental syntax layer with a recorded `NoConvergence`
/// reason — recorded refusal, state preserved; the whole-source overlay
/// path handles it — so the two-fn lane uses the working body anchors.)
#[test]
fn av_source_incremental_two_fn_local_updates() {
    let src = ap01();
    let mut s = warmup(&src, "d_main", &[]).expect("warmup");
    // R5: helper-dependency axis at the IDENTITY level — main's resident
    // (ptr, generation) must be UNCHANGED by gcd's body edit (the caller
    // reads only the callee interface).
    let main_identity_before = resident_identity(&s, "d_main").expect("main identity");
    assert_eq!(run_value(&s), 0);

    // Edit gcd's body (semantics change: negate the gcd result) — every
    // in-source group now fails, main returns 1.
    let (start, end, insert) = edit_once(&src, "    return x\n}", "    return 0 - x\n}");
    let obs = s.edit(start, end, insert).expect("edit infra");
    assert!(
        obs.ok,
        "{:?} {:?}",
        obs.non_fast_path_reason, obs.diagnostics
    );
    assert_eq!(
        obs.install_set,
        vec!["d_gcd".to_string()],
        "gcd only installs"
    );
    assert_eq!(obs.changed_fn.as_deref(), Some("gcd"));
    assert_eq!(run_value(&s), 1, "negated gcd fails every group -> 1");
    // main is untouched: every stage count for main is zero AND its
    // resident identity holds.
    for (stage, count) in s.counters().stages_of("main") {
        assert_eq!(count, 0, "main stage `{stage}` must stay zero");
    }
    assert_eq!(
        resident_identity(&s, "d_main").expect("main identity after helper edit"),
        main_identity_before,
        "a helper body edit must not move the caller's resident object"
    );

    // Edit main's body (first group's comparison 6 -> 7): main only; gcd
    // untouched by THIS edit (deltas over `before`). Offsets are taken
    // against the CURRENT source (the gcd edit shifted the tail by +2).
    let src_after = src.replace("    return x\n}", "    return 0 - x\n}");
    let before = s.counters().snapshot();
    let (start, end, insert) = edit_once(&src_after, "gcd(48, 18) != 6", "gcd(48, 18) != 7");
    let obs = s.edit(start, end, insert).expect("edit infra");
    assert!(
        obs.ok,
        "{:?} {:?}",
        obs.non_fast_path_reason, obs.diagnostics
    );
    assert_eq!(
        obs.install_set,
        vec!["d_main".to_string()],
        "main only installs"
    );
    assert_eq!(run_value(&s), 1, "first group now fails -> 1 (gcd intact)");
    let after = s.counters().snapshot();
    for stage in ace_av::session::STAGES {
        let b = before.get(&format!("{stage}:gcd")).copied().unwrap_or(0);
        let a = after.get(&format!("{stage}:gcd")).copied().unwrap_or(0);
        assert_eq!(a, b, "gcd stage `{stage}` unchanged by main's edit");
    }
}

/// V-13 (1.3): precise refusal — signature/interface and unauthorized
/// capability edits are refused with the state untouched.
#[test]
fn av_source_incremental_precise_refusals() {
    let src = ap01();
    let mut s = warmup(&src, "d_main", &[]).expect("warmup");
    assert_eq!(run_value(&s), 0);
    let rev0 = s.revision();

    // Signature/interface edit on gcd: refused, revision preserved. The
    // refusal is RECORDED (reason or rendered diagnostics).
    let (start, end, insert) = edit_once(
        &src,
        "fn gcd(a int, b int) int {",
        "fn gcd(a int, b int, c int) int {",
    );
    let obs = s.edit(start, end, insert).expect("edit infra");
    assert!(!obs.ok, "a signature edit must be refused");
    assert!(
        obs.non_fast_path_reason.is_some() || !obs.diagnostics.is_empty(),
        "the refusal is recorded: reason={:?} diags={:?}",
        obs.non_fast_path_reason,
        obs.diagnostics
    );
    assert_eq!(s.revision(), rev0, "refusal keeps the revision");
    assert_eq!(run_value(&s), 0, "the prior snapshot still runs correctly");

    // Unauthorized capability edit: a mark_a() call without the trace
    // grant is refused by the frontend (source.undefined).
    let pos = src.find("fn main() int {").expect("main") as u64 + "fn main() int {".len() as u64;
    let obs = s
        .edit(pos, pos, b"\n    let m = mark_a()".to_vec())
        .expect("edit infra");
    assert!(!obs.ok, "an unauthorized capability edit must be refused");
    assert_eq!(s.revision(), rev0, "refusal keeps the revision");
    assert_eq!(run_value(&s), 0);

    // R5: revoking the CURRENT grant at session RE-ENTRY — a trace-using
    // source cannot even warm up without the live grant (this is a
    // different input from adding unauthorized NEW text above).
    let trace_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../crates/auto-lang/test/cases/ac-av/av/core/trace_order.at"),
    )
    .expect("trace seed");
    assert!(
        warmup(&trace_src, "d_main", &[]).is_err(),
        "re-entry with the grant revoked must refuse (marks undefined without the live grant)"
    );
}

/// V-13 (1.4): degradation is CAUGHT, never silent — fast-path body edits
/// record no fallback; a non-fast-path edit (outside a function body) is
/// REFUSED with a recorded reason instead of a hidden whole-module
/// recompute; the resident session keeps serving the prior revision.
#[test]
fn av_source_incremental_degradation_caught() {
    let src = ap01();
    let mut s = warmup(&src, "d_main", &[]).expect("warmup");
    // A normal body edit (working anchor): fast path, no fallback recorded.
    let (start, end, insert) = edit_once(&src, "gcd(48, 18) != 6", "gcd(48, 18) != 7");
    let obs = s.edit(start, end, insert).expect("edit infra");
    assert!(obs.ok);
    assert!(
        obs.non_fast_path_reason.is_none(),
        "body edits must not record a fallback"
    );
    assert_eq!(run_value(&s), 1, "the edit really installed (result moved)");
    let rev1 = s.revision();

    // An edit OUTSIDE any function body (comment between functions): the
    // fast path cannot apply it — it must be REFUSED WITH a recorded
    // reason, never a silent whole-module recompute.
    let sep = src.find("}\n\nfn main").expect("separator") as u64 + 2;
    let obs = s
        .edit(sep, sep, b"// inter-function comment\n".to_vec())
        .expect("edit infra");
    if obs.ok {
        panic!(
            "an outside-body edit must not silently install; reason={:?}",
            obs.non_fast_path_reason
        );
    }
    assert!(
        obs.non_fast_path_reason.is_some() || !obs.diagnostics.is_empty(),
        "the non-fast-path refusal is RECORDED (reason/diagnostics), not silent"
    );
    assert_eq!(s.revision(), rev1, "refusal keeps the live revision");
    assert_eq!(run_value(&s), 1, "the resident session still serves rev");
}

/// R1-F02: the IDENTITY STABILITY gate — the delivered degradation
/// detector (mirrors the 828 formal judge's `edit_identity_gate`): across
/// an edit step, every UNCHANGED function must hold its resident
/// (ptr, generation). A consumer that degrades to whole-frontend/reload
/// per edit (a fresh warm-up) reallocates EVERY function object — the
/// unchanged pointers move and the gate goes red. (The warm-up cold path
/// records no counters/phase-io — verified — so pointer stability is the
/// axis the production judge itself uses for this class.)
fn identity_findings(
    before: &std::collections::BTreeMap<String, (usize, u64)>,
    after: &std::collections::BTreeMap<String, (usize, u64)>,
    changed: &str,
) -> Vec<String> {
    let mut findings = Vec::new();
    for (name, was) in before {
        if name == changed {
            continue;
        }
        match after.get(name) {
            Some(now) if now == was => {}
            Some(now) => findings.push(format!(
                "{name}: identity moved {was:?} -> {now:?} without an edit"
            )),
            None => findings.push(format!("{name}: identity missing after the step")),
        }
    }
    findings
}

fn snapshot_identities(
    s: &FunctionIncrementalSession,
) -> std::collections::BTreeMap<String, (usize, u64)> {
    s.function_names()
        .iter()
        .map(|n| {
            let def = format!("d_{n}");
            (n.clone(), resident_identity(s, &def).expect("identity"))
        })
        .collect()
}

/// V-13 (1.4) / R1-F02: the degradation DETECTOR's negative oracle.
/// (a) A REAL forced whole-frontend/reload (a fresh warm-up on the edited
///     source — exactly what a degrading consumer does per edit) must make
///     the identity gate RED; the legitimate fast-path edit over the same
///     source stays GREEN. The degraded path still computes the right
///     VALUE — the gate is structural, which is the point.
/// (b) A hash-preserving object swap — a second warm-up of the SAME
///     source: per-function content versions EQUAL, resident objects
///     (Arc pointers) DIFFERENT — so swapping objects behind an unchanged
///     hash cannot hide from the (ptr, generation) axes.
#[test]
fn av_source_incremental_detector_negative_controls() {
    let base = ace_av::load256::generate_revision(0);

    // (a) Positive: a real fast-path edit stays green under BOTH gates
    // (counter structure AND identity stability).
    let mut s = warmup(&base.source, "d_main", &[]).expect("warmup");
    let rev1 = ace_av::load256::generate_revision(1);
    let (start, end, insert) =
        ace_av::formal_828::diff_edit(&base.source, &rev1.source).expect("diff");
    let counter_before = s.counters().snapshot();
    let identity_before = snapshot_identities(&s);
    let obs = s.edit(start, end, insert).expect("edit infra");
    assert!(obs.ok, "the legitimate edit installs");
    let changed = obs.changed_fn.clone().unwrap_or_default();
    let after_counters = s.counters().snapshot();
    assert!(
        whole_module_findings(&counter_before, &after_counters).is_empty(),
        "a fast-path edit is green under the counter structure gate"
    );
    let after_identities = snapshot_identities(&s);
    assert!(
        identity_findings(&identity_before, &after_identities, &changed).is_empty(),
        "a fast-path edit is green under the identity gate: {:?}",
        identity_findings(&identity_before, &after_identities, &changed)
    );
    assert!(
        identity_before[&changed] != after_identities[&changed],
        "the edited function's identity moved"
    );

    // (a) Negative: FORCED degradation — the degrading consumer's actual
    // step (a fresh whole-frontend warm-up on the edited source, per
    // edit). Every unchanged function's resident object is REALLOCATED:
    // the identity gate over the same transition goes RED, while the
    // degraded path still computes the correct VALUE — the detector is
    // structural, which is the point.
    let s2 = warmup(&rev1.source, "d_main", &[]).expect("forced re-warmup");
    assert_eq!(
        run_value(&s2),
        rev1.expected_main,
        "the degraded path still computes the right value"
    );
    let degraded_identities = snapshot_identities(&s2);
    let findings = identity_findings(&after_identities, &degraded_identities, &changed);
    assert!(
        !findings.is_empty(),
        "the identity gate must FIRE on a forced whole-frontend/reload step"
    );
    assert!(
        findings.len() >= 100,
        "the re-warmup reallocates essentially every function object (saw {} findings)",
        findings.len()
    );

    // (b) Hash-preserving swap discrimination: two warm-ups of the SAME
    // source — executor-published per-function CONTENT versions are equal
    // (same hash), while the resident objects are distinct allocations
    // (different pointers): the ptr axis catches a swapped object that a
    // content hash alone would accept.
    let sa = warmup(&base.source, "d_main", &[]).expect("warmup a");
    let sb = warmup(&base.source, "d_main", &[]).expect("warmup b");
    let _ = run_value(&sa);
    let _ = run_value(&sb);
    let (pub_a, _, _) = sa.last_run_publication();
    let (pub_b, _, _) = sb.last_run_publication();
    let map_a: std::collections::BTreeMap<String, (u64, u64, u64)> = pub_a
        .iter()
        .map(|(n, ptr, gen, content, _)| (n.clone(), (*ptr, *gen, *content)))
        .collect();
    let map_b: std::collections::BTreeMap<String, (u64, u64, u64)> = pub_b
        .iter()
        .map(|(n, ptr, gen, content, _)| (n.clone(), (*ptr, *gen, *content)))
        .collect();
    assert!(
        !map_a.is_empty(),
        "the executor published dispatch identities"
    );
    for (name, (pa, ga, ca)) in &map_a {
        let (pb, gb, cb) = map_b
            .get(name)
            .unwrap_or_else(|| panic!("function {name} dispatched in both runs"));
        assert_eq!(ca, cb, "{name}: same source -> same content version (hash)");
        assert_eq!(ga, gb, "{name}: same warm-up sequence -> same generation");
        assert_ne!(
            pa, pb,
            "{name}: fresh warm-up = distinct resident object — the ptr axis discriminates hash-preserving swaps"
        );
    }
}

// Formal campaigns run only through the explicit performance namespace.
mod performance {
    use super::*;

    /// V-13 (1.1): the D828 frozen baseline through this plan's consumer —
    /// 30 distinct body revisions, structure AND time gates.
    #[test]
    fn av_source_incremental_baseline_30_structure_and_p95() {
        let base = ace_av::load256::generate_revision(0);
        assert_eq!(base.source.len(), 273_575, "the D828 frozen baseline size");
        let mut s = warmup(&base.source, "d_main", &[]).expect("warmup");
        assert_eq!(run_value(&s), base.expected_main, "warm-up value");
        let fn_names = s.function_names();
        assert_eq!(fn_names.len(), 115, "115 functions");
        // R1-F02 identity binding: every function's resident (ptr, generation)
        // from the warm-up snapshot — unchanged functions must hold BOTH axes
        // across all 30 revisions; the edited function must move.
        let identities_before: std::collections::BTreeMap<String, (usize, u64)> = fn_names
            .iter()
            .map(|n| {
                let def = format!("d_{n}");
                (n.clone(), resident_identity(&s, &def).expect("identity"))
            })
            .collect();

        let samples = 30usize;
        let mut walls = Vec::with_capacity(samples);
        // R7 F-827-R7-01: per-run durable raw — each revision is written to disk
        // BEFORE that revision's assertions, and the final verdict is written
        // before the P95 assertion, so a failing revision/gate always leaves
        // its evidence. (The map above is keyed by BARE function names; def-ids
        // (`d_main`) are formed only at the `resident_identity` boundary.)
        let run_id = format!(
            "{}-pid{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_secs(),
            std::process::id()
        );
        let mut raw = IncrementalRawRun::start(samples, &run_id);
        let mut prev_src = base.source.clone();
        for n in 1..=samples {
            let rev = ace_av::load256::generate_revision(n as u32);
            let (start, end, insert) =
                ace_av::formal_828::diff_edit(&prev_src, &rev.source).expect("diff");
            let before = s.counters().snapshot();
            let t0 = std::time::Instant::now();
            let obs = s.edit(start, end, insert).expect("edit infra");
            let v = run_value(&s);
            let wall_ms = t0.elapsed().as_secs_f64() * 1000.0;
            walls.push(wall_ms);
            // R7: the unchanged sample's baseline and current identity pairs
            // are REQUIRED — an absent lookup is an error, never a persisted
            // null (the shipped defect persisted 30 null `before` fields).
            let before_pair = *identities_before.get("main").unwrap_or_else(|| {
                panic!("baseline identity for `main` absent from the warm-up map")
            });
            let after_pair = resident_identity(&s, "d_main")
                .unwrap_or_else(|| panic!("current identity for `d_main` absent after rev {n}"));
            let changed_name = obs.changed_fn.clone();
            let changed_moved: Option<bool> = changed_name.as_deref().and_then(|c| {
                identities_before.get(c).map(|was| {
                    let now = resident_identity(&s, &format!("d_{c}"))
                        .unwrap_or_else(|| panic!("current identity for changed `{c}` absent"));
                    now != *was
                })
            });
            raw.push_revision(serde_json::json!({
                "revision": n,
                "ok": obs.ok,
                "non_fast_path_reason": format!("{:?}", obs.non_fast_path_reason),
                "changed_fn": changed_name,
                "install_set": obs.install_set,
                "value": v,
                "wall_ms": wall_ms,
                "identity_hold": {
                    "changed_moved": changed_moved,
                    "unchanged_sample": {
                        // one unchanged function's before/after identity
                        "name": "d_main",
                        "before": [before_pair.0, before_pair.1],
                        "after": [after_pair.0, after_pair.1],
                    },
                },
            }));
            assert!(
                obs.ok,
                "rev {n} refused: {:?} {:?}",
                obs.non_fast_path_reason, obs.diagnostics
            );
            assert_eq!(v, rev.expected_main, "rev {n}: independent expectation");
            assert!(
                obs.install_set.len() == 1,
                "rev {n}: exactly one install, saw {:?}",
                obs.install_set
            );
            assert!(
                obs.non_fast_path_reason.is_none(),
                "rev {n}: a body edit is a fast-path edit"
            );
            // Structure: no whole-module stage counter grows; the ONLY
            // module-level counters that move are the incremental syntax
            // bookkeeping (`syntax_*`), and their reuse evidence is POSITIVE
            // (114 declarations reused on the frozen load).
            let after = s.counters().snapshot();
            let mut unexpected: Vec<String> = Vec::new();
            for (k, av) in &after {
                let bv = before.get(k).copied().unwrap_or(0);
                if *av > bv && !k.contains(':') && !k.starts_with("syntax_") {
                    unexpected.push(format!("{k} +{}", av - bv));
                }
            }
            assert!(
                unexpected.is_empty(),
                "rev {n}: whole-module counters grew: {unexpected:?}"
            );
            assert!(
                after.get("syntax_reused_decls").copied().unwrap_or(0)
                    > before.get("syntax_reused_decls").copied().unwrap_or(0),
                "rev {n}: the incremental syntax layer must show reuse"
            );
            // R1-F02 explicit structure gate over the same deltas — the
            // detector the forced-degradation control exercises.
            assert!(
                whole_module_findings(&before, &after).is_empty(),
                "rev {n}: structure gate red: {:?}",
                whole_module_findings(&before, &after)
            );
            // R1-F02 identity axes across this revision: unchanged functions
            // hold (ptr, generation) from the ORIGINAL warm-up; the edited
            // function moves at least one axis.
            let changed = obs.changed_fn.clone().unwrap_or_default();
            for n2 in &fn_names {
                let def = format!("d_{n2}");
                let now = resident_identity(&s, &def).expect("identity after edit");
                let was = identities_before[n2];
                if *n2 == changed {
                    assert_ne!(
                        now, was,
                        "rev {n}: the edited function `{changed}` must move identity"
                    );
                } else {
                    assert_eq!(
                        now, was,
                        "rev {n}: unchanged function `{n2}` must hold ptr+generation"
                    );
                }
            }
            // R1-F02 production/IO facts: the warm path rescans INCREMENTALLY
            // (bytes far below a whole-source rescan) and writes NOTHING to
            // the store (no store attached — zero disk IO).
            let io = s.phase_io();
            let rescan = io.get("syntax_rescan_bytes").copied().unwrap_or(0);
            assert!(
                rescan > 0 && (rescan as usize) < base.source.len() / 100,
                "rev {n}: incremental rescan {rescan}B vs whole {}B",
                base.source.len()
            );
            assert_eq!(
                io.get("store_write_bytes").copied().unwrap_or(0),
                0,
                "rev {n}: the warm edit path performs zero store IO"
            );
            prev_src = rev.source;
        }
        let p50 = percentile(&walls, 50.0);
        let p95 = percentile(&walls, 95.0);
        let p90 = ace_av::observations::latency_p90_ms(&walls).expect("valid latency windows");
        let time_gate = p90 <= 2000.0;
        // R7 F-827-R7-01: persist structure + time verdict BEFORE asserting it —
        // a red P95 leaves its per-run raw on disk with `verdict.status=fail`.
        validate_raw_shape(&raw.doc).expect("completed campaign raw shape");
        raw.finish(serde_json::json!({
            "status": if time_gate { "pass" } else { "fail" },
            "structure": "pass",
            "p50_ms": p50,
            "p90_ms": p90,
            "p95_ms": p95,
            "time_gate_ms": 2000.0,
            "time_gate": time_gate,
        }));
        assert!(
            time_gate,
            "edit->verified-result P90 {p90:.1}ms over {samples} revisions must be <= 2000ms"
        );
        eprintln!(
            "av827 incremental baseline: P50 {:.1}ms P95 {:.1}ms (raw run {run_id})",
            p50, p95
        );
    }
}
