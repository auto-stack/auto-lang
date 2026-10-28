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

use auto_av_core::av::{execute_program_steps, CancelFlag, ExecLimits, SteppedOutcome};
use auto_av_core::contract::{ExecOutcome, Value};
use auto_av_probe::resident::{ResidentConfig, ResidentRunner, ResidentState};
use auto_av_probe::session::warmup;

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
fn av_resident_866_slice_invariance_and_chain_continuity() {
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
fn av_resident_866_restart_variant_is_red() {
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
fn av_resident_866_cancel_surfaces_as_outcome() {
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
fn av_resident_866_stop_between_slices() {
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
fn av_resident_866_zero_fuel_slice_parks_without_progress() {
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
fn av_resident_866_candidate_queue_never_disturbs_the_run() {
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
