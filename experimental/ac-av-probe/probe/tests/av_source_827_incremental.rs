//! PLAN-827 T-09 (V-13): hot edits over this plan's source entry reuse the
//! D828 session (read-only consumption of the delivered public API) —
//! never a new Driver / whole-source recompile per edit. Four sub-cases
//! per the ACE contract:
//! (1.1) the D828 frozen baseline (273 575 B / 115 functions) x >=30
//!       distinct body revisions through THIS consumer, dual gate:
//!       structure (unchanged functions' stages all-zero, exactly one
//!       install per edit) AND edit->verified-result P95 < 1000 ms;
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

use auto_av_core::av::{CancelFlag, ExecLimits};
use auto_av_core::contract::ExecOutcome;
use auto_av_probe::session::{warmup, FunctionIncrementalSession};

const AP01_ORIGINAL: &str = "../../ac-core/fixtures/scalar-source/valid/p-gcd-euclid.at";
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

/// V-13 (1.1): the D828 frozen baseline through this plan's consumer —
/// 30 distinct body revisions, structure AND time gates.
#[test]
fn av_source_827_incremental_d828_baseline_30_structure_and_p95() {
    let base = auto_av_probe::load256::generate_revision(0);
    assert_eq!(base.source.len(), 273_575, "the D828 frozen baseline size");
    let mut s = warmup(&base.source, "d_main", &[]).expect("warmup");
    assert_eq!(run_value(&s), base.expected_main, "warm-up value");
    assert_eq!(s.function_names().len(), 115, "115 functions");

    let samples = 30usize;
    let mut walls = Vec::with_capacity(samples);
    let mut prev_src = base.source.clone();
    for n in 1..=samples {
        let rev = auto_av_probe::load256::generate_revision(n as u32);
        let (start, end, insert) =
            auto_av_probe::formal_828::diff_edit(&prev_src, &rev.source).expect("diff");
        let before = s.counters().snapshot();
        let t0 = std::time::Instant::now();
        let obs = s.edit(start, end, insert).expect("edit infra");
        let v = run_value(&s);
        walls.push(t0.elapsed().as_secs_f64() * 1000.0);
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
        prev_src = rev.source;
    }
    let p95 = percentile(&walls, 95.0);
    assert!(
        p95 < 1000.0,
        "edit->verified-result P95 {p95:.1}ms over {samples} revisions must be < 1000ms"
    );
    eprintln!(
        "av827 incremental baseline: P50 {:.1}ms P95 {:.1}ms",
        percentile(&walls, 50.0),
        p95
    );
}

/// V-13 (1.2): two-function local body updates on the REAL AP-01 entry —
/// only the edited function re-keys/installs; the other's stages stay
/// zero; semantics propagate through the shared pipeline.
/// (Shape note: main's trailing `return 0` replacement is REFUSED by the
/// session's incremental syntax layer with a recorded `NoConvergence`
/// reason — recorded refusal, state preserved; the whole-source overlay
/// path handles it — so the two-fn lane uses the working body anchors.)
#[test]
fn av_source_827_incremental_two_fn_local_updates() {
    let src = ap01();
    let mut s = warmup(&src, "d_main", &[]).expect("warmup");
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
    // main is untouched: every stage count for main is zero.
    for (stage, count) in s.counters().stages_of("main") {
        assert_eq!(count, 0, "main stage `{stage}` must stay zero");
    }

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
    for stage in auto_av_probe::session::STAGES {
        let b = before.get(&format!("{stage}:gcd")).copied().unwrap_or(0);
        let a = after.get(&format!("{stage}:gcd")).copied().unwrap_or(0);
        assert_eq!(a, b, "gcd stage `{stage}` unchanged by main's edit");
    }
}

/// V-13 (1.3): precise refusal — signature/interface and unauthorized
/// capability edits are refused with the state untouched.
#[test]
fn av_source_827_incremental_precise_refusals() {
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
}

/// V-13 (1.4): degradation is CAUGHT, never silent — fast-path body edits
/// record no fallback; a non-fast-path edit (outside a function body) is
/// REFUSED with a recorded reason instead of a hidden whole-module
/// recompute; the resident session keeps serving the prior revision.
#[test]
fn av_source_827_incremental_degradation_caught() {
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
