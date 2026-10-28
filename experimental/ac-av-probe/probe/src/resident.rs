//! PLAN-866 (RF-H): AV resident online hot reload — the resident run
//! driver.
//!
//! A resident run is ONE guest execution chain (one top-level entry call)
//! that the host advances by fuel slices and NEVER restarts: each slice
//! resumes the paused chain, so guest-visible state (loop counters, step
//! accounting, pinned code) carries across slices. The host does not pause
//! the guest *because an edit arrived* — edits travel as prepared
//! candidates and take effect only at the frozen safety points (guest call
//! boundaries), which land with the core dispatch hook after the 864
//! write-surface release (wave-handoff §5.0 write ordering; this module
//! deliberately does not fork a private runtime to bypass that ordering).
//!
//! Phase 1 (parallel with PLAN-864, private module + private tests):
//! continuous slice advance, cancel/stop, observation counters, and the
//! candidate queue face — `prepare_edit` is host-side and never touches
//! the running chain, so queueing/rejecting candidates is already
//! observable without the safety-point hook.
//!
//! Negative-control contract (SD-01 禁止替代): a restart, a re-run, or a
//! pause-that-secretly-restarts cannot pass the V-01 oracle — the honest
//! driver's final value AND cumulative step count are identical under any
//! slice granularity, while any restart-per-slice variant either exhausts
//! the cumulative fuel or returns a partial-run value.

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use auto_av_core::av::{
    execute_program_steps, resume_program, CancelFlag, ExecLimits, PausedRun, SteppedOutcome,
};
use auto_av_core::contract::ExecOutcome;

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
