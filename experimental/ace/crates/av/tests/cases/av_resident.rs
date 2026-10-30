//! PLAN-866 resident-run phase-1 controls (V-01 face): one guest chain
//! advanced by fuel slices, never restarted, cancellable/stoppable, with
//! the candidate-queue discipline face of V-04.
//!
//! - slice invariance: final value AND cumulative step ledger are
//!   identical under any slice granularity; the chain-start ledger moves
//!   exactly once per honest run.
//! - restart is RED, not merely unpreferred: a deliberately restarting
//!   driver (fresh entry per slice, paused chain discarded) cannot
//!   produce the honest value within the same fuel and mints one chain
//!   start per slice — both observable, both wrong.
//! - cancel surfaces as the guest outcome at the executor poll; stop is
//!   the host's terminal decision between slices and refuses further
//!   slices.
//! - queued candidates never disturb the running chain and never commit
//!   early; rejected edits are observable reasons; a cancelled candidate
//!   releases its never-installed code object (Weak oracle).

use ace_av::resident::{ResidentCandidate, ResidentConfig, ResidentRunner, ResidentState};
use ace_av::session::warmup;
use ace_av_core::av::{
    execute_program_steps, resume_program, resume_program_at_call_boundary, CancelFlag, ExecLimits,
    SteppedOutcome,
};
use ace_av_core::contract::{ExecOutcome, Value};

/// Straight-line accumulation loop: deterministic value (500) and step
/// count, long enough to span many slices at small granularities.
const SPIN: &str = "fn helper() int { return 7 }\nfn main() int {\n    var i = 0\n    for i < 500 {\n        i = i + 1\n    }\n    return i\n}\n";

/// The V-02-shaped program: main calls helper every round, so a future
/// mid-run helper swap changes exactly the rounds after the safety point.
/// Phase 1 asserts the DISCIPLINE: a queued (never-committed) candidate
/// leaves the running chain on the old version.
const CALLS: &str = "fn helper() int { return 7 }\nfn main() int {\n    var i = 0\n    var total = 0\n    for i < 300 {\n        total = total + helper()\n        i = i + 1\n    }\n    return total\n}\n";

fn drive_to_completion(r: &mut ResidentRunner, cap: u64) -> (ExecOutcome, u64, u64) {
    let mut slices = 0u64;
    loop {
        let report = r.slice();
        assert!(slices < cap, "run did not finish within {cap} slices");
        slices += 1;
        if report.state != ResidentState::Running {
            return (
                report.outcome.expect("terminal slice carries the outcome"),
                report.total_steps,
                slices,
            );
        }
    }
}

fn success_value(out: &ExecOutcome) -> i32 {
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => *v,
        other => panic!("expected i32 success, got {other:?}"),
    }
}

/// V-01: the honest resident driver is slice-invariant — same value, same
/// cumulative steps, one chain entry per run — at every granularity from
/// single-instruction slices to unbounded.
#[test]
fn av_resident_slice_invariance_and_chain_continuity() {
    let mut runs: Vec<(i32, u64, u64)> = Vec::new();
    for slice_steps in [1u64, 7, 64, u64::MAX] {
        let session = warmup(SPIN, "d_main", &[]).expect("warmup");
        let mut r = ResidentRunner::start(
            session,
            ResidentConfig {
                slice_steps,
                ..ResidentConfig::default()
            },
        );
        let (out, steps, slices) = drive_to_completion(&mut r, 60_000);
        assert_eq!(success_value(&out), 500, "value must not depend on slicing");
        assert_eq!(r.state(), ResidentState::Finished);
        assert_eq!(r.total_steps(), steps, "ledger equals the report");
        assert!(r.counters().get("resident:slices").copied().unwrap_or(0) == slices);
        assert_eq!(r.chain_entries(), 1, "one chain entry per honest run");
        runs.push((success_value(&out), steps, slices));
    }
    // Same value and same total work at every granularity.
    let (v0, s0, _) = runs[0];
    for (v, s, _) in &runs[1..] {
        assert_eq!(*v, v0, "value invariance");
        assert_eq!(*s, s0, "cumulative step invariance (no re-execution)");
    }
}

/// V-01 negative control: a restarting driver (fresh entry every slice,
/// paused chain discarded) is distinguishable — it never reaches the
/// honest value within comparable fuel, while the honest driver over the
/// same program and per-slice fuel finishes with the honest value.
#[test]
fn av_resident_restart_variant_is_red() {
    let session = warmup(SPIN, "d_main", &[]).expect("warmup");
    let snap = session.current();
    let cancel = CancelFlag::new(false);
    let limits = ExecLimits::default();

    let mut finished: Option<ExecOutcome> = None;
    for _ in 0..200 {
        match execute_program_steps(&snap, &limits, &cancel, 64) {
            SteppedOutcome::Paused(_) => {
                // The dishonest variant throws the chain away here: every
                // slice re-enters from the top, so `i` never passes the
                // first ~8 rounds of its 500.
            }
            SteppedOutcome::Finished(out, _) => {
                finished = Some(out);
                break;
            }
        }
    }
    match finished {
        Some(out) => assert_ne!(
            success_value(&out),
            500,
            "a restarting driver must not produce the honest value"
        ),
        None => { /* never finished in 200 slices: red by non-arrival */ }
    }

    // The honest driver over the SAME program and per-slice fuel finishes
    // with the honest value — the two implementations are separable.
    let session = warmup(SPIN, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 64,
            ..ResidentConfig::default()
        },
    );
    let (out, _, slices) = drive_to_completion(&mut r, 60_000);
    assert_eq!(success_value(&out), 500);
    assert_eq!(
        r.chain_entries(),
        1,
        "honest driver resumes, never restarts"
    );
    assert!(
        slices < 200,
        "honest driver needs fewer slices than the mutation's budget"
    );
}

/// V-01: host cancellation surfaces as the guest outcome (a host decision
/// through the executor's own poll seam), before and between slices.
#[test]
fn av_resident_cancel_surfaces_as_outcome() {
    let session = warmup(SPIN, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 4_096,
            ..ResidentConfig::default()
        },
    );
    r.cancel();
    let report = r.slice();
    assert_eq!(report.state, ResidentState::Finished);
    assert!(
        matches!(report.outcome, Some(ExecOutcome::Cancelled)),
        "pre-set cancel must surface as Cancelled, got {:?}",
        report.outcome
    );
    // Terminal: further slices are controlled no-ops.
    let again = r.slice();
    assert_eq!(again.state, ResidentState::Finished);
    assert_eq!(again.slice_steps, 0);
    assert_eq!(again.total_steps, report.total_steps);
}

/// V-01: stop is the host's terminal decision BETWEEN slices — the chain
/// is released, further slices refuse, the session stays inspectable.
#[test]
fn av_resident_stop_between_slices() {
    let session = warmup(SPIN, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 8,
            ..ResidentConfig::default()
        },
    );
    for _ in 0..3 {
        let report = r.slice();
        assert_eq!(report.state, ResidentState::Running);
        assert!(report.total_steps > 0);
    }
    let steps_at_stop = r.total_steps();
    r.stop();
    assert_eq!(r.state(), ResidentState::Stopped);
    for _ in 0..2 {
        let report = r.slice();
        assert_eq!(report.state, ResidentState::Stopped);
        assert_eq!(report.slice_steps, 0);
        assert_eq!(report.total_steps, steps_at_stop, "no progress after stop");
        assert!(report.outcome.is_none(), "stopped is not a guest outcome");
    }
    // The session survives the stop (host-side inspection/editing remains).
    assert!(!r.session().closed());
    assert_eq!(r.session().revision(), 0);
}

/// 828 per-segment contract face: a zero-fuel slice parks the chain
/// without progress; the cumulative ledger does not move.
#[test]
fn av_resident_zero_fuel_slice_parks_without_progress() {
    let session = warmup(SPIN, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 0,
            ..ResidentConfig::default()
        },
    );
    for _ in 0..3 {
        let report = r.slice();
        assert_eq!(report.state, ResidentState::Running);
        assert_eq!(report.slice_steps, 0);
        assert_eq!(report.total_steps, 0);
    }
    assert_eq!(r.slice_count(), 3, "slices are counted even at zero fuel");
}

/// V-04 discipline face: candidates queue without touching the running
/// chain or the session revision; a non-fast-path edit is an observable
/// rejection; cancelling the queue consumes the candidate and releases
/// its never-installed code object (Weak oracle).
#[test]
fn av_resident_candidate_queue_never_disturbs_the_run() {
    let session = warmup(CALLS, "d_main", &[]).expect("warmup");
    let revision0 = session.revision();
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 128,
            ..ResidentConfig::default()
        },
    );
    // A few slices of live chain before queueing.
    for _ in 0..3 {
        assert_eq!(r.slice().state, ResidentState::Running);
    }
    let steps_before = r.total_steps();

    // Good candidate: single body edit of helper (7 -> 9), old-source
    // byte coordinates.
    let src = CALLS.to_string();
    let at = src.find("return 7").expect("helper body") + "return ".len();
    let queued = r.submit_edit(at as u64, (at + 1) as u64, b"9".to_vec());
    assert!(matches!(queued, Ok(Ok(()))), "body edit must prepare");
    assert_eq!(r.pending().len(), 1);
    assert_eq!(r.pending()[0].changed_name(), "helper");
    assert_eq!(
        r.session().revision(),
        revision0,
        "a queued candidate must not advance the revision"
    );

    // Non-fast-path edit (spans two declarations): an observable
    // rejection that changes nothing.
    let mid = src.find("\nfn main").expect("decl boundary");
    let rejected = r.submit_edit(mid as u64, (mid + 2) as u64, b"  ".to_vec());
    match rejected {
        Ok(Err(out)) => {
            assert!(!out.ok);
            assert!(out.non_fast_path_reason.is_some());
        }
        other => panic!("two-decl edit must be an observable rejection, got {other:?}"),
    }
    assert_eq!(r.pending().len(), 1, "rejection must not queue");

    // The chain keeps running the OLD version untouched.
    for _ in 0..3 {
        assert_eq!(r.slice().state, ResidentState::Running);
    }
    assert!(r.total_steps() > steps_before);

    // Capture the never-installed code object, then cancel the queue:
    // consumed, revision unchanged, object released.
    let weak = r.pending()[0].code_weak();
    assert!(weak.upgrade().is_some(), "candidate owns its code object");
    let cancels = r.cancel_pending();
    assert_eq!(cancels.len(), 1);
    assert!(!cancels[0].ok, "cancelled candidate never installs");
    assert!(r.pending().is_empty());
    assert_eq!(r.session().revision(), revision0);
    assert!(
        weak.upgrade().is_none(),
        "cancelled candidate's code object must be released"
    );

    // Finish on the old version: 300 rounds x 7.
    let (out, _, _) = drive_to_completion(&mut r, 60_000);
    assert_eq!(success_value(&out), 300 * 7, "old version end to end");
}

// ---------------------------------------------------------------------------
// Online reload (V-02..V-05): the safety-point machinery.
// ---------------------------------------------------------------------------

/// Helper body byte offset of the `7` in `return 7` (old-source coords).
fn seven_at(src: &str) -> (u64, u64) {
    let at = src.find("return 7").expect("helper body") + "return ".len();
    (at as u64, (at + 1) as u64)
}

/// Run the CALLS scenario once: drive `arm_slices` slices, submit the
/// helper 7->9 edit, apply it at the safety point, finish, and report
/// (final value, boundary steps, retired snapshot alive at end).
fn calls_scenario(slice_steps: u64, arm_slices: u64) -> (i32, u64, bool) {
    let session = warmup(CALLS, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps,
            ..ResidentConfig::default()
        },
    );
    for _ in 0..arm_slices {
        let report = r.slice();
        assert_eq!(report.state, ResidentState::Running, "arm phase stays live");
    }
    let (s, e) = seven_at(CALLS);
    assert!(matches!(r.submit_edit(s, e, b"9".to_vec()), Ok(Ok(()))));
    let sp = r.apply_pending_at_safety_point();
    assert!(sp.armed, "a live chain and a queued candidate must arm");
    assert!(sp.reached_boundary, "armed segment must meet a helper call");
    assert!(sp.edit.ok, "body edit must commit at the safety point");
    assert!(
        sp.edit.install_set.iter().any(|d| d.contains("helper")),
        "install set targets helper: {:?}",
        sp.edit.install_set
    );
    assert!(sp.visibility.is_some(), "visibility sample recorded");
    // The retired snapshot Weak is handed out on the successful swap.
    let retired = sp.retired_snapshot_weak.expect("swap retires a snapshot");
    let (out, _, _) = drive_to_completion(&mut r, 60_000);
    assert_eq!(r.chain_entries(), 1, "the swap never restarts the chain");
    (
        success_value(&out),
        sp.steps_at_boundary.expect("boundary steps recorded"),
        retired.upgrade().is_some(),
    )
}

/// V-02: the online swap takes effect at an EXACT call boundary — the
/// rounds after the safety point run the new body, the rounds before it
/// ran the old one — and the effective point is INDEPENDENT of the host's
/// fuel granularity (the same arm position lands the same boundary under
/// any slicing, which a slice-boundary switcher cannot satisfy).
#[test]
fn av_resident_online_swap_effective_at_call_boundary() {
    // Granularity A: 10 slices x 64 = 640 fuel steps to the arm.
    let (va, boundary_a, retired_alive_a) = calls_scenario(64, 10);
    // Granularity B: 20 slices x 32 = the SAME 640 arm position.
    let (vb, boundary_b, retired_alive_b) = calls_scenario(32, 20);

    // Both landed the same guest-semantic effective point.
    assert_eq!(
        boundary_a, boundary_b,
        "effective point is guest-semantic, not host-fuel"
    );
    assert_eq!(va, vb, "final value is slice-granularity independent");

    // Exact old/new split: va = 7*(m-1) + 9*(301-m) for a mid-run m —
    // neither all-old (2100) nor all-new (2700), and the delta from
    // all-old is an exact number of new calls.
    assert_ne!(
        va,
        300 * 7,
        "an all-old run means the swap never landed mid-chain"
    );
    assert_ne!(
        va,
        300 * 9,
        "an all-new run means the chain restarted on the new body"
    );
    let delta = va - 300 * 7;
    assert_eq!(delta % 2, 0, "each new call adds exactly +2");
    let new_calls = (delta / 2) as i32;
    assert!(
        (1..300).contains(&new_calls),
        "mid-run swap: {new_calls} new calls out of 300"
    );

    // The retired SNAPSHOT dies at/after the swap even while the chain
    // keeps running: frames pin code objects, not whole snapshots.
    assert!(
        !retired_alive_a && !retired_alive_b,
        "retired snapshot released after swap"
    );
}

/// V-02 negative control at the core seam: committing at the safety point
/// but resuming WITHOUT the swapped resolution is the pause-resume-only
/// implementation — it must keep running the old body to the end.
#[test]
fn av_resident_no_swap_resume_stays_old() {
    let mut session = warmup(CALLS, "d_main", &[]).expect("warmup");
    let cancel = CancelFlag::new(false);
    let limits = ExecLimits::default();
    let snap = session.current();
    let mut paused = match execute_program_steps(&snap, &limits, &cancel, 640) {
        SteppedOutcome::Paused(p) => p,
        _ => panic!("640 fuel must park the 300-round loop"),
    };
    let (s, e) = seven_at(CALLS);
    let prepared = match session.prepare_edit(s, e, b"9".to_vec()).expect("prepare") {
        Ok(p) => p,
        Err(out) => panic!("body edit must prepare: {:?}", out.non_fast_path_reason),
    };
    // Park at the very same call boundary the honest driver would.
    paused = match resume_program_at_call_boundary(paused, &cancel, Some(4_096)) {
        SteppedOutcome::Paused(p) => p,
        _ => panic!("armed segment must meet a helper call"),
    };
    let committed = session.commit(prepared);
    assert!(committed.ok, "commit at the boundary succeeds");
    // The dishonest variant: plain resume, no swapped resolution.
    let out = match resume_program(paused, &cancel, None) {
        SteppedOutcome::Finished(o, _) => o,
        _ => panic!("plain resume must finish"),
    };
    assert_eq!(
        success_value(&out),
        300 * 7,
        "no-swap resume keeps the old body"
    );
}

/// V-03: an in-flight caller frame keeps its pinned old code across the
/// swap — outer's loop logic stays the old generation while every helper
/// dispatch after the safety point resolves new.
const NESTED: &str = "fn helper() int { return 7 }\nfn outer() int {\n    var t = 0\n    var i = 0\n    for i < 300 {\n        t = t + helper()\n        i = i + 1\n    }\n    return t\n}\nfn main() int { return outer() }\n";

#[test]
fn av_resident_inflight_frame_stays_old_callee_resolves_new() {
    let session = warmup(NESTED, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 64,
            ..ResidentConfig::default()
        },
    );
    // Drive into outer's loop: main -> outer (in flight) -> helper calls.
    for _ in 0..10 {
        assert_eq!(r.slice().state, ResidentState::Running);
    }
    let (s, e) = seven_at(NESTED);
    assert!(matches!(r.submit_edit(s, e, b"9".to_vec()), Ok(Ok(()))));
    let sp = r.apply_pending_at_safety_point();
    assert!(sp.armed && sp.reached_boundary && sp.edit.ok);
    // The caller chain (outer's frame) was NOT rebuilt: the swap only
    // redirects resolution; one chain entry for the whole run.
    assert_eq!(r.chain_entries(), 1);
    let (out, _, _) = drive_to_completion(&mut r, 60_000);
    let v = success_value(&out);
    // outer (old code, in flight across the swap) accumulated rounds of
    // helper: rounds before the boundary at 7, after at 9. An
    // implementation that rebuilt the in-flight frame onto the new
    // revision, or restarted it, cannot produce this exact split.
    let delta = v - 300 * 7;
    assert_eq!(delta % 2, 0);
    let new_calls = (delta / 2) as i32;
    assert!(
        (1..300).contains(&new_calls),
        "in-flight old caller + new callee split: got {new_calls}/300 new calls"
    );
}

/// V-04: stale and foreign candidates are refused AT THE SAFETY POINT —
/// the run keeps the current version and the refusal is observable.
#[test]
fn av_resident_stale_and_foreign_refused_at_safety_point() {
    let session = warmup(CALLS, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 64,
            ..ResidentConfig::default()
        },
    );
    for _ in 0..10 {
        assert_eq!(r.slice().state, ResidentState::Running);
    }
    // Two candidates prepared at the same base revision.
    let (s, e) = seven_at(CALLS);
    assert!(matches!(r.submit_edit(s, e, b"9".to_vec()), Ok(Ok(()))));
    assert!(matches!(r.submit_edit(s, e, b"9".to_vec()), Ok(Ok(()))));

    // First commits and swaps (revision advances).
    let first = r.apply_pending_at_safety_point();
    assert!(first.edit.ok, "first candidate installs");
    let revision_after_first = r.session().revision();

    // Second is stale now (same base revision as the first): refused at
    // its safety point, run continues on the installed version.
    let second = r.apply_pending_at_safety_point();
    assert!(second.armed && second.reached_boundary);
    assert!(!second.edit.ok, "stale candidate must be refused");
    assert!(
        second.edit.non_fast_path_reason.is_some() || !second.edit.diagnostics.is_empty(),
        "refusal reason observable: {:?}",
        second.edit.non_fast_path_reason
    );
    assert_eq!(r.session().revision(), revision_after_first);
    assert!(second.retired_snapshot_weak.is_none(), "no swap on refusal");

    // Foreign: a candidate minted by ANOTHER session, applied through
    // the primitive — refused by the session's own commit.
    let mut other = warmup(CALLS, "d_main", &[]).expect("other session");
    let foreign_prepared = match other.prepare_edit(s, e, b"11".to_vec()).expect("prepare") {
        Ok(p) => p,
        Err(out) => {
            panic!(
                "foreign body edit must prepare: {:?}",
                out.non_fast_path_reason
            )
        }
    };
    let foreign =
        r.apply_prepared_at_safety_point(ResidentCandidate::wrap_prepared(foreign_prepared));
    assert!(foreign.armed && foreign.reached_boundary);
    assert!(!foreign.edit.ok, "cross-session candidate must be refused");
    assert_eq!(
        r.session().revision(),
        revision_after_first,
        "foreign never installs"
    );

    // Finish on the FIRST swap's version only: 7/9 split, no 11 ever.
    let (out, _, _) = drive_to_completion(&mut r, 60_000);
    let delta = success_value(&out) - 300 * 7;
    assert_eq!(delta % 2, 0);
    assert!(
        (delta / 2) as i32 >= 1,
        "the one committed swap did land mid-run"
    );
    assert!(
        success_value(&out) <= 300 * 9,
        "no round ever saw the refused 11"
    );
}

/// V-05 (bounded recycle face): churn distinct helper bodies through the
/// safety point; every retired snapshot is released, the live-old ledger
/// stays empty, and the bounded history keeps the old-generation budget.
#[test]
fn av_resident_churn_retired_generations_release() {
    let session = warmup(CALLS, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 128,
            ..ResidentConfig::default()
        },
    );
    assert_eq!(r.slice().state, ResidentState::Running);
    let mut retired_weaks = Vec::new();
    let swaps = 8u32;
    for k in 0..swaps {
        // Each edit is a DISTINCT body content (7 -> 9 -> 11 -> ...), so
        // every commit mints a new generation (no content hits). The
        // coordinates are computed on the session's CURRENT source —
        // prior edits shifted the offsets (old-source byte semantics).
        let from = (7 + 2 * k).to_string();
        let to = (7 + 2 * (k + 1)).to_string();
        let cur = r.session().src().to_string();
        let needle = format!("return {from}");
        let at = cur.find(&needle).expect("current helper body") + "return ".len();
        let (s, e) = (at as u64, (at + from.len()) as u64);
        assert!(
            matches!(r.submit_edit(s, e, to.into_bytes()), Ok(Ok(()))),
            "churn edit {k} prepares"
        );
        let sp = r.apply_pending_at_safety_point();
        assert!(
            sp.armed && sp.reached_boundary && sp.edit.ok,
            "churn swap {k}"
        );
        if let Some(w) = sp.retired_snapshot_weak {
            retired_weaks.push(w);
        }
        // V-06 structural face: a helper-only body edit must do ZERO
        // production work for every other function, and no whole-module
        // stage may ever move on the edit path.
        assert!(
            r.session()
                .counters()
                .stages_of("main")
                .values()
                .all(|v| *v == 0),
            "unchanged main has zero production stages"
        );
        assert!(
            r.session()
                .counters()
                .whole_module()
                .values()
                .all(|v| *v == 0),
            "no whole-module recompute on the online path"
        );
        assert_eq!(
            r.live_old_generations(),
            0,
            "retired snapshots release at swap"
        );
        // Keep the chain running between swaps.
        if r.state() == ResidentState::Running {
            assert_eq!(r.slice().state, ResidentState::Running);
        }
    }
    assert_eq!(
        r.counters().get("resident:swaps").copied().unwrap_or(0),
        swaps as u64
    );
    assert!(retired_weaks.iter().all(|w| w.upgrade().is_none()));
    // The bounded history keeps per-function old generations capped.
    let fns = r.session().function_names().len() as u64;
    assert!(
        (r.session().history_len() as u64) <= 32 * fns,
        "history bounded: {} entries for {fns} functions",
        r.session().history_len()
    );
    let (out, _, _) = drive_to_completion(&mut r, 60_000);
    // Rounds strictly increase per swap; the exact split is deterministic
    // but granular — the invariant is the value lies strictly between the
    // all-old and all-new extremes.
    let v = success_value(&out);
    assert!(v > 300 * 7, "at least one swap landed: {v}");
    assert!(
        v < 300 * (7 + 2 * swaps as i32),
        "not all rounds ran the last body: {v}"
    );
}

/// V-06: reload visibility by the TestCase v2 calibre — development-time
/// effective samples, P90 <= 2000ms hard gate, P95/P99 diagnostics only,
/// sample count never shrunk (30 planned, 30 recorded).
#[test]
fn av_resident_reload_visibility_v2_calibre() {
    const SAMPLES: usize = 30;
    let session = warmup(CALLS, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 128,
            ..ResidentConfig::default()
        },
    );
    assert_eq!(r.slice().state, ResidentState::Running);
    let mut samples: Vec<u128> = Vec::with_capacity(SAMPLES);
    for k in 0..SAMPLES as u32 {
        let from = (7 + 2 * k).to_string();
        let to = (7 + 2 * (k + 1)).to_string();
        let cur = r.session().src().to_string();
        let needle = format!("return {from}");
        let at = cur.find(&needle).expect("current helper body") + "return ".len();
        let (s, e) = (at as u64, (at + from.len()) as u64);
        assert!(matches!(r.submit_edit(s, e, to.into_bytes()), Ok(Ok(()))));
        let sp = r.apply_pending_at_safety_point();
        assert!(
            sp.armed && sp.reached_boundary && sp.edit.ok,
            "sample {k} swap"
        );
        samples.push(sp.visibility.expect("visibility sample").as_micros());
        if r.state() == ResidentState::Running {
            let _ = r.slice();
        }
    }
    assert_eq!(samples.len(), SAMPLES, "sample count never shrinks");
    samples.sort_unstable();
    let at = |q: f64| samples[(((SAMPLES as f64) * q).ceil() as usize).min(SAMPLES - 1)];
    let p90 = at(0.90);
    let p95 = at(0.95);
    let p99 = at(0.99);
    // Development-time hard gate (TestCase v2): P90 <= 2000ms.
    assert!(
        p90 <= 2_000_000,
        "P90 reload visibility {p90}us exceeds the 2000ms hard gate"
    );
    // Diagnostics only (not gates): P95/P99 recorded with the sample.
    eprintln!(
        "av_resident_866 reload visibility: n={} p90={}us p95={}us p99={}us",
        SAMPLES, p90, p95, p99
    );
}

/// F-866-R1/R3 negative control (frozen contract §2.3): an edit whose
/// changed function is ITSELF executing on the in-flight chain (here:
/// main, the resident ancestor) must NOT install at the safety point —
/// the old body runs to return and the new body takes effect only for
/// the NEXT top-level run. An implementation that swaps the in-flight
/// edited frame mid-invocation cannot produce the exact old-value
/// completion below.
#[test]
fn av_resident_inflight_ancestor_defers_and_next_run_sees_new() {
    let session = warmup(CALLS, "d_main", &[]).expect("warmup");
    let revision0 = session.revision();
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 64,
            ..ResidentConfig::default()
        },
    );
    for _ in 0..10 {
        assert_eq!(r.slice().state, ResidentState::Running);
    }
    // Edit the IN-FLIGHT ancestor itself: main's loop bound.
    let at = CALLS.find("i < 300").expect("main loop bound");
    let (s, e) = (at as u64, (at + "i < 300".len()) as u64);
    assert!(matches!(
        r.submit_edit(s, e, b"i < 200".to_vec()),
        Ok(Ok(()))
    ));

    let sp = r.apply_pending_at_safety_point();
    assert!(
        sp.armed && sp.reached_boundary,
        "the boundary is still reached"
    );
    assert!(
        sp.deferred_in_flight,
        "in-flight changed function must defer"
    );
    assert!(!sp.edit.ok, "nothing installs at the safety point");
    assert!(
        sp.edit
            .non_fast_path_reason
            .as_deref()
            .unwrap_or_default()
            .contains("deferred"),
        "deferral is observable: {:?}",
        sp.edit.non_fast_path_reason
    );
    assert!(sp.retired_snapshot_weak.is_none(), "no swap on deferral");
    assert_eq!(r.session().revision(), revision0, "revision unchanged");
    assert_eq!(r.pending().len(), 1, "the candidate stays queued");
    assert_eq!(r.chain_entries(), 1, "the chain is never restarted");

    // The in-flight old body completes the WHOLE run: 300 rounds x 7.
    let (out, _, _) = drive_to_completion(&mut r, 60_000);
    assert_eq!(
        success_value(&out),
        300 * 7,
        "the old body runs to return (a mid-invocation swap cannot give this)"
    );

    // After the chain ended, the held candidate commits for FUTURE runs.
    let landed = r.apply_pending_at_safety_point();
    assert!(
        landed.edit.ok,
        "the deferred candidate commits at chain end"
    );
    assert_eq!(r.session().revision(), revision0 + 1);
    // The NEXT top-level run sees the new body: 200 rounds x 7.
    let (next, _) = r.session().run(
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(false),
    );
    assert_eq!(
        success_value(&next),
        200 * 7,
        "the new body is visible to the next run only"
    );
}

/// F-866-R2 stopgap control: a slot-layout-changing body edit of the
/// in-flight ancestor (accepted by the 828 prepare/commit pipeline) must
/// defer — never panic — and the post-commit next run must execute the
/// new layout correctly.
#[test]
fn av_resident_inflight_layout_change_never_panics() {
    let session = warmup(CALLS, "d_main", &[]).expect("warmup");
    let mut r = ResidentRunner::start(
        session,
        ResidentConfig {
            slice_steps: 64,
            ..ResidentConfig::default()
        },
    );
    for _ in 0..10 {
        assert_eq!(r.slice().state, ResidentState::Running);
    }
    let at = CALLS.find("total = total + helper()").expect("main body");
    let from = "total = total + helper()";
    let to = "total = total + helper() + 1";
    let (s, e) = (at as u64, (at + from.len()) as u64);
    assert!(matches!(
        r.submit_edit(s, e, to.as_bytes().to_vec()),
        Ok(Ok(()))
    ));

    let sp = r.apply_pending_at_safety_point();
    assert!(
        sp.deferred_in_flight,
        "layout-changing ancestor edit defers"
    );
    // No panic through completion: the old layout runs to return.
    let (out, _, _) = drive_to_completion(&mut r, 60_000);
    assert_eq!(success_value(&out), 300 * 7);
    // Commits after the chain; the next run runs the NEW layout cleanly.
    let landed = r.apply_pending_at_safety_point();
    assert!(landed.edit.ok);
    let (next, _) = r.session().run(
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(false),
    );
    assert_eq!(success_value(&next), 300 * 7 + 300);
}
