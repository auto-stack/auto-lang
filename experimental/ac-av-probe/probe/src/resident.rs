//! PLAN-866 (RF-H): AV resident online hot reload — the resident run
//! driver.
//!
//! A resident run is ONE guest execution chain (one top-level entry call)
//! that the host advances by fuel slices and NEVER restarts: each slice
//! resumes the paused chain, so guest-visible state (loop counters, step
//! accounting, pinned code) carries across slices. The host does not pause
//! the guest *because an edit arrived* — edits travel as prepared
//! candidates and take effect exactly at the frozen safety points (guest
//! call boundaries, via the core `resume_program_at_call_boundary` /
//! `resume_program_with_resolution` pair): the armed segment keeps
//! executing guest work until a real call boundary parks it, the 828
//! commit publishes there, and the swap-resolution resume redirects every
//! LATER call dispatch to the new revision while in-flight frames keep
//! their pinned old code to return (no mixed chains).
//!
//! Negative-control contract (SD-01 禁止替代): a restart, a re-run, a
//! pause-that-secretly-restarts, a label-only bump, or a swap that never
//! lands mid-chain cannot pass the V-01/V-02 oracles — the honest driver's
//! final value and cumulative step ledger are identical under any slice
//! granularity, while the effective-iteration math pins exactly which
//! calls ran old and which ran new.

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use auto_av_core::av::{
    execute_program_steps, resume_program, resume_program_at_call_boundary,
    resume_program_with_resolution, CancelFlag, ExecLimits, PausedRun, SteppedOutcome,
};
use auto_av_core::contract::ExecOutcome;
use auto_av_core::program_snapshot::ProgramSnapshot;

use crate::session::{EditOutcome, FunctionIncrementalSession, PreparedEdit};

/// Default fuel slice: guest instructions per host slice. The slice size
/// is a HOST scheduling knob only — it must never be observable in guest
/// semantics (V-01 slice-invariance oracle).
pub const RESIDENT_SLICE_STEPS: u64 = 4_096;

/// Simultaneous live old generations kept by one resident run after
/// swaps. Frozen at the HISTORY_CAP shape (PLAN-828): the resident
/// current entry is never evicted, history beyond the cap is dropped
/// eagerly, and the Weak release oracle proves the drop.
pub const MAX_RESIDENT_OLD_GENERATIONS: usize = 32;

/// Host-side configuration of a resident run.
#[derive(Clone, Debug)]
pub struct ResidentConfig {
    /// Fuel per slice (guest instructions). `0` pauses without progress,
    /// like the 828 per-segment budget contract.
    pub slice_steps: u64,
    /// Cumulative hard budgets for the WHOLE chain: `max_steps` is the
    /// total fuel across all slices (825 ledger semantics); `wall` applies
    /// per execution segment (per slice), matching `resume_program`.
    pub limits: ExecLimits,
}

impl Default for ResidentConfig {
    fn default() -> Self {
        ResidentConfig {
            slice_steps: RESIDENT_SLICE_STEPS,
            limits: ExecLimits::default(),
        }
    }
}

/// Terminal-vs-live state of the resident run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidentState {
    /// The chain is live; further slices advance it.
    Running,
    /// The guest produced a terminal `ExecOutcome` (value, trap, resource
    /// exhaustion, or a surfaced host cancellation). Terminal.
    Finished,
    /// The host stopped the run between slices; the paused chain was
    /// dropped and its pins released. Terminal. Distinct from `Finished`:
    /// the chain was never allowed to complete.
    Stopped,
}

/// A prepared candidate waiting for its safety point. The queue entry
/// owns the candidate (828: held OUT of the session until commit) plus
/// the reload-visibility measurement origin.
pub struct ResidentCandidate {
    prepared: PreparedEdit,
    /// When the edit entered the queue — the `t0` of the reload-visibility
    /// measurement (edit arrival → first slice that dispatches the new
    /// generation), TestCase v2 calibre.
    pub submitted_at: Instant,
}

impl ResidentCandidate {
    /// Wrap an externally prepared candidate (same-session prepare via
    /// [`FunctionIncrementalSession::prepare_edit`]) for the resident
    /// queue. A candidate minted by ANOTHER session wraps fine too — the
    /// commit at the safety point refuses it (observable), which is
    /// exactly the V-04 foreign control.
    pub fn wrap_prepared(prepared: PreparedEdit) -> Self {
        ResidentCandidate {
            prepared,
            submitted_at: Instant::now(),
        }
    }

    /// Source name of the function this candidate would install.
    pub fn changed_name(&self) -> &str {
        self.prepared.changed_name()
    }

    /// Weak view of the never-installed code object (diagnostics-only
    /// release oracle, 828 C05-03).
    pub fn code_weak(&self) -> Weak<auto_av_core::function_code::FunctionCode> {
        self.prepared.code_weak()
    }
}

/// Report returned by one [`ResidentRunner::slice`] call.
#[derive(Clone, Debug)]
pub struct SliceReport {
    pub state: ResidentState,
    /// Guest instructions executed during THIS slice (0 for refused
    /// no-op slices after a terminal state).
    pub slice_steps: u64,
    /// Cumulative guest steps across all slices of this chain — the
    /// no-restart ledger: a restart resets it, an honest resume only ever
    /// grows it.
    pub total_steps: u64,
    /// Terminal outcome once the chain finished; `None` while live or
    /// stopped.
    pub outcome: Option<ExecOutcome>,
    /// Wall time of this slice's execution segment.
    pub wall: Duration,
}

/// Report of one candidate application at a call-boundary safety point
/// (PLAN-866 online reload): the guest-semantic effective point, the 828
/// commit outcome, the reload-visibility measurement, and the bounded
/// old-generation face.
#[derive(Clone, Debug)]
pub struct SafetyPointReport {
    /// Whether the chain was armed to seek a safety point at all (false:
    /// no live chain, no pending candidate, or the old-generation cap).
    pub armed: bool,
    /// Whether the armed segment parked at a real guest call boundary.
    pub reached_boundary: bool,
    /// The 828 commit outcome at the safety point (install set on
    /// success; the observable refusal reason on stale/foreign/etc.).
    pub edit: EditOutcome,
    /// Chain steps at the safety point — the effective point in guest
    /// terms, independent of host fuel slicing.
    pub steps_at_boundary: Option<u64>,
    /// Reload visibility: edit submission → effect at the safety point
    /// (TestCase v2 measurement, one sample).
    pub visibility: Option<Duration>,
    /// The chain finished during this application (commit at chain end:
    /// effective for future runs only).
    pub chain_finished: bool,
    /// Live retired generations still pinned after this application.
    pub live_old_generations: usize,
    /// Weak view of the retired snapshot on a successful swap — dies with
    /// its last in-flight frame (V-05 release oracle).
    pub retired_snapshot_weak: Option<Weak<ProgramSnapshot>>,
}

/// One continuously-running guest, driven by fuel slices.
///
/// Owns the [`FunctionIncrementalSession`] for its whole lifetime: between
/// slices the host may queue candidates (`submit_edit`), and later — once
/// the safety-point hook lands — commit them exactly at call boundaries.
/// The live chain holds its own `Arc<ProgramSnapshot>` pin, so host-side
/// prepares/commits can never yank code out from under it (828 pin
/// semantics).
pub struct ResidentRunner {
    session: FunctionIncrementalSession,
    paused: Option<PausedRun>,
    cancel: Arc<CancelFlag>,
    config: ResidentConfig,
    state: ResidentState,
    outcome: Option<ExecOutcome>,
    total_steps: u64,
    slices: u64,
    /// Chain entries this runner minted: exactly one per honest run (the
    /// driver re-enters the entry ONLY when no live chain exists; every
    /// other slice resumes). The countable no-restart face of V-01: a
    /// driver that restarts the entry per slice would carry one per slice.
    chain_entries: u64,
    wall_total: Duration,
    queue: Vec<ResidentCandidate>,
    /// Weak views of snapshots retired by swaps (bounded-recycle ledger:
    /// live entries are those still pinned by in-flight frames).
    retired: Vec<Weak<ProgramSnapshot>>,
    counters: BTreeMap<String, u64>,
}

impl ResidentRunner {
    /// Start a resident run of the session's CURRENT snapshot entry. Does
    /// not execute yet — the first [`Self::slice`] starts the chain, so
    /// `t0`-adjacent host bookkeeping (queueing an edit before the run
    /// even begins) stays expressible.
    pub fn start(session: FunctionIncrementalSession, config: ResidentConfig) -> Self {
        ResidentRunner {
            session,
            paused: None,
            cancel: Arc::new(CancelFlag::new(false)),
            config,
            state: ResidentState::Running,
            outcome: None,
            total_steps: 0,
            slices: 0,
            chain_entries: 0,
            wall_total: Duration::ZERO,
            queue: Vec::new(),
            retired: Vec::new(),
            counters: BTreeMap::new(),
        }
    }

    /// The host cancellation handle for this run — setting it makes the
    /// chain surface `ExecOutcome::Cancelled` at the executor's next poll
    /// (inside the current or the next slice).
    pub fn cancel_handle(&self) -> Arc<CancelFlag> {
        self.cancel.clone()
    }

    /// Request cooperative cancellation of the guest.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// Stop the resident run from the host side: terminal, idempotent.
    /// Drops the paused chain (releasing its snapshot pin and frames),
    /// keeps the session (the host may keep preparing edits or inspect
    /// counters), and refuses any further slice.
    pub fn stop(&mut self) {
        if self.state == ResidentState::Running {
            self.state = ResidentState::Stopped;
        }
        self.paused = None;
    }

    /// Advance the chain by one fuel slice.
    ///
    /// After a terminal state this is a controlled no-op (state echoed,
    /// zero progress, no execution). A `slice_steps` of `0` parks the
    /// chain without progress, mirroring the 828 per-segment contract.
    pub fn slice(&mut self) -> SliceReport {
        if self.state != ResidentState::Running {
            return SliceReport {
                state: self.state,
                slice_steps: 0,
                total_steps: self.total_steps,
                outcome: self.outcome.clone(),
                wall: Duration::ZERO,
            };
        }
        if self.session.closed() {
            self.state = ResidentState::Stopped;
            self.paused = None;
            return SliceReport {
                state: self.state,
                slice_steps: 0,
                total_steps: self.total_steps,
                outcome: None,
                wall: Duration::ZERO,
            };
        }
        let started = Instant::now();
        let stepped = match self.paused.take() {
            None => {
                // A fresh chain entry — at most once per honest resident
                // run (every later slice resumes). A driver that restarts
                // the entry per slice counts one here per slice: that is
                // the restart the V-01 oracle forbids, made countable.
                self.chain_entries += 1;
                let snap = self.session.current();
                execute_program_steps(
                    &snap,
                    &self.config.limits,
                    &self.cancel,
                    self.config.slice_steps,
                )
            }
            Some(paused) => resume_program(paused, &self.cancel, Some(self.config.slice_steps)),
        };
        let wall = started.elapsed();
        self.slices += 1;
        self.wall_total += wall;
        *self.counters.entry("resident:slices".into()).or_insert(0) += 1;
        match stepped {
            SteppedOutcome::Paused(p) => {
                let after = p.step_count();
                let slice_steps = after - self.total_steps;
                self.total_steps = after;
                self.paused = Some(p);
                *self
                    .counters
                    .entry("resident:guest_steps".into())
                    .or_insert(0) += slice_steps;
                SliceReport {
                    state: ResidentState::Running,
                    slice_steps,
                    total_steps: self.total_steps,
                    outcome: None,
                    wall,
                }
            }
            SteppedOutcome::Finished(out, _trace) => {
                // Success carries the authoritative cumulative step count;
                // other terminals report the last observed ledger.
                let mut slice_steps = 0;
                if let ExecOutcome::Success { steps, .. } = &out {
                    slice_steps = *steps - self.total_steps;
                    self.total_steps = *steps;
                }
                *self
                    .counters
                    .entry("resident:guest_steps".into())
                    .or_insert(0) += slice_steps;
                self.state = ResidentState::Finished;
                self.outcome = Some(out.clone());
                SliceReport {
                    state: self.state,
                    slice_steps,
                    total_steps: self.total_steps,
                    outcome: Some(out),
                    wall,
                }
            }
        }
    }

    /// Prepare an edit against the session and queue it for a future
    /// safety point. Never touches the running chain and never advances
    /// the session revision (that happens only at commit, on the safety
    /// point). A rejected edit is returned for observation — the resident
    /// run keeps running on the current version untouched.
    pub fn submit_edit(
        &mut self,
        start: u64,
        end: u64,
        insert: Vec<u8>,
    ) -> Result<Result<(), EditOutcome>, String> {
        if self.state == ResidentState::Running && self.session.closed() {
            self.state = ResidentState::Stopped;
            self.paused = None;
        }
        match self.session.prepare_edit(start, end, insert) {
            Ok(Ok(prepared)) => {
                self.queue.push(ResidentCandidate {
                    prepared,
                    submitted_at: Instant::now(),
                });
                *self
                    .counters
                    .entry("resident:edits_prepared".into())
                    .or_insert(0) += 1;
                Ok(Ok(()))
            }
            Ok(Err(rejected)) => {
                *self
                    .counters
                    .entry("resident:edits_rejected".into())
                    .or_insert(0) += 1;
                Ok(Err(rejected))
            }
            Err(hard) => Err(hard),
        }
    }

    /// Candidates currently waiting for a safety point.
    pub fn pending(&self) -> &[ResidentCandidate] {
        &self.queue
    }

    /// Apply the QUEUED head candidate at the next call-boundary safety
    /// point (PLAN-866 online reload). One candidate per call; loop for a
    /// queue drain. Never restarts the chain and never pauses it because
    /// an edit arrived — the armed segment keeps executing guest work
    /// until a real call boundary parks it.
    pub fn apply_pending_at_safety_point(&mut self) -> SafetyPointReport {
        let candidate = if self.queue.is_empty() {
            return SafetyPointReport {
                armed: false,
                reached_boundary: false,
                edit: EditOutcome::rejected(
                    self.session.revision(),
                    "no pending candidate".to_string(),
                    Vec::new(),
                ),
                steps_at_boundary: None,
                visibility: None,
                chain_finished: false,
                live_old_generations: self.live_old_generations(),
                retired_snapshot_weak: None,
            };
        } else {
            self.queue.remove(0)
        };
        self.apply_prepared_at_safety_point(candidate)
    }

    /// Apply ONE candidate at the next call-boundary safety point — the
    /// primitive behind [`Self::apply_pending_at_safety_point`]. A foreign
    /// session's candidate is refused by the session's own commit
    /// (observable in `report.edit`); the run is never disturbed.
    pub fn apply_prepared_at_safety_point(
        &mut self,
        candidate: ResidentCandidate,
    ) -> SafetyPointReport {
        let visibility_origin = candidate.submitted_at;
        if self.state != ResidentState::Running || self.paused.is_none() {
            // No live chain (never started or already terminal): the chain
            // end is trivially "after all calls" — commit for future runs.
            let edit = self.session.commit(candidate.prepared);
            return SafetyPointReport {
                armed: false,
                reached_boundary: false,
                edit,
                steps_at_boundary: None,
                visibility: Some(visibility_origin.elapsed()),
                chain_finished: self.state == ResidentState::Finished,
                live_old_generations: self.live_old_generations(),
                retired_snapshot_weak: None,
            };
        }
        if self.live_old_generations() >= MAX_RESIDENT_OLD_GENERATIONS {
            // Bounded recycle (frozen §2.6): refuse to arm while the cap
            // of simultaneously live retired generations is reached; the
            // candidate stays queued and the run continues untouched.
            self.queue.insert(0, candidate);
            return SafetyPointReport {
                armed: false,
                reached_boundary: false,
                edit: EditOutcome::rejected(
                    self.session.revision(),
                    format!(
                        "old-generation cap {MAX_RESIDENT_OLD_GENERATIONS} reached; candidate held"
                    ),
                    Vec::new(),
                ),
                steps_at_boundary: None,
                visibility: None,
                chain_finished: false,
                live_old_generations: self.live_old_generations(),
                retired_snapshot_weak: None,
            };
        }
        let paused = self.paused.take().expect("live chain checked above");
        let armed =
            resume_program_at_call_boundary(paused, &self.cancel, Some(self.config.slice_steps));
        match armed {
            SteppedOutcome::Paused(p) if p.at_call_boundary() => {
                // The frozen safety point: one completed call. Commit HERE
                // (828 single publication point), then swap resolution —
                // in-flight frames keep their pinned old code to return.
                self.total_steps = p.step_count();
                let steps_at_boundary = p.step_count();
                let retired_weak = Arc::downgrade(p.prog());
                let visibility = Some(visibility_origin.elapsed());
                let edit = self.session.commit(candidate.prepared);
                let resumed = if edit.ok {
                    let resolution = self.session.current();
                    resume_program_with_resolution(p, &self.cancel, Some(0), &resolution)
                } else {
                    resume_program(p, &self.cancel, Some(0))
                };
                let mut chain_finished = false;
                match resumed {
                    SteppedOutcome::Paused(p) => {
                        self.total_steps = self.total_steps.max(p.step_count());
                        self.paused = Some(p);
                    }
                    SteppedOutcome::Finished(out, _) => {
                        self.state = ResidentState::Finished;
                        self.outcome = Some(out);
                        chain_finished = true;
                    }
                }
                if edit.ok {
                    self.retired.push(retired_weak.clone());
                    *self.counters.entry("resident:swaps".into()).or_insert(0) += 1;
                }
                SafetyPointReport {
                    armed: true,
                    reached_boundary: true,
                    edit: edit.clone(),
                    steps_at_boundary: Some(steps_at_boundary),
                    visibility,
                    chain_finished,
                    live_old_generations: self.live_old_generations(),
                    retired_snapshot_weak: if edit.ok { Some(retired_weak) } else { None },
                }
            }
            SteppedOutcome::Paused(p) => {
                // Fuel exhausted before any call boundary: the candidate
                // is NOT consumed — still prepared, still valid; retry on
                // a later slice. The chain keeps running untouched.
                self.total_steps = self.total_steps.max(p.step_count());
                self.paused = Some(p);
                self.queue.insert(0, candidate);
                SafetyPointReport {
                    armed: true,
                    reached_boundary: false,
                    edit: EditOutcome::rejected(
                        self.session.revision(),
                        "no call boundary within the armed segment's fuel".to_string(),
                        Vec::new(),
                    ),
                    steps_at_boundary: None,
                    visibility: None,
                    chain_finished: false,
                    live_old_generations: self.live_old_generations(),
                    retired_snapshot_weak: None,
                }
            }
            SteppedOutcome::Finished(out, _) => {
                // The chain completed without meeting another call: the
                // chain end is the (trivial) safety point — commit there;
                // the revision advances for FUTURE runs only.
                self.state = ResidentState::Finished;
                self.outcome = Some(out.clone());
                let edit = self.session.commit(candidate.prepared);
                SafetyPointReport {
                    armed: true,
                    reached_boundary: false,
                    edit,
                    steps_at_boundary: None,
                    visibility: Some(visibility_origin.elapsed()),
                    chain_finished: true,
                    live_old_generations: self.live_old_generations(),
                    retired_snapshot_weak: None,
                }
            }
        }
    }

    /// Live retired generations: snapshots swapped out by this run that
    /// still have an in-flight frame pinning them (Weak oracle count).
    pub fn live_old_generations(&self) -> usize {
        self.retired
            .iter()
            .filter(|w| w.upgrade().is_some())
            .count()
    }

    /// Cancel every queued candidate through the session's own refuse
    /// path (consumed, never installed; 828 F-05b). The resident run is
    /// unaffected.
    pub fn cancel_pending(&mut self) -> Vec<EditOutcome> {
        let mut out = Vec::new();
        for cand in self.queue.drain(..) {
            out.push(self.session.cancel_prepared(cand.prepared));
        }
        out
    }

    pub fn state(&self) -> ResidentState {
        self.state
    }

    /// Cumulative guest steps across all slices (no-restart ledger).
    pub fn total_steps(&self) -> u64 {
        self.total_steps
    }

    /// Slices executed so far.
    pub fn slice_count(&self) -> u64 {
        self.slices
    }

    /// Chain entries minted by this runner (the no-restart ledger: one
    /// per honest run).
    pub fn chain_entries(&self) -> u64 {
        self.chain_entries
    }

    /// The terminal guest outcome, once the chain finished.
    pub fn outcome(&self) -> Option<&ExecOutcome> {
        self.outcome.as_ref()
    }

    /// Total wall time spent inside execution segments.
    pub fn wall_total(&self) -> Duration {
        self.wall_total
    }

    /// Observation counters (852 calibre: only real events bump).
    pub fn counters(&self) -> &BTreeMap<String, u64> {
        &self.counters
    }

    /// The session this resident run owns (host-side edits, counters,
    /// weak oracles between slices).
    pub fn session(&self) -> &FunctionIncrementalSession {
        &self.session
    }

    /// Mutable session access for host-side preparation between slices.
    pub fn session_mut(&mut self) -> &mut FunctionIncrementalSession {
        &mut self.session
    }
}
