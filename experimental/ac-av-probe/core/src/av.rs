//! The AV executor for `av.core-i32/1` (PLAN-798 §5.2).
//!
//! Explicit frame/slot stack and a flat dispatch loop: guest recursion
//! never rides the host Rust stack. Budgets: instruction fuel, frame count,
//! live slot bytes, wall clock, test-trace bytes; cancellation is polled at
//! most every [`CANCEL_CHECK_INTERVAL`] instructions. Language traps are
//! distinct from resource exhaustion and cancellation; a trap on an
//! unexecuted branch stays latent because traps are only raised by actually
//! executed instructions.

use crate::abc_verify::LoadedModule;
use crate::contract::{
    ExecOutcome, ResourceKind, SpanInfo, TrapKind, Value, CANCEL_CHECK_INTERVAL, MAX_AV_FRAMES,
    MAX_AV_INSTR_STEPS, MAX_AV_SLOT_BYTES, MAX_AV_WALL, MAX_TRACE_BYTES,
};
use crate::mir::{
    BinOp, CalleeRef, MirFunction, MirInstr, MirModule, MirType, Terminator, TraceEvent, UnOp,
};
use std::time::Instant;

/// Per-execution limits (defaults = the frozen probe budgets).
#[derive(Clone, Copy, Debug)]
pub struct ExecLimits {
    pub max_steps: u64,
    pub max_frames: usize,
    pub max_slot_bytes: usize,
    pub wall: std::time::Duration,
    pub max_trace_bytes: usize,
    /// Test-only fault-injection seam (V-07): fail the Nth value
    /// allocation (any slot write with a positive data delta) with an
    /// Infra outcome BEFORE the write lands — prior frames/values/trace
    /// stay intact, proving allocation failure does not corrupt state.
    /// `None` in every production path.
    pub fault_after_allocs: Option<u64>,
}

impl Default for ExecLimits {
    fn default() -> Self {
        ExecLimits {
            max_steps: MAX_AV_INSTR_STEPS,
            max_frames: MAX_AV_FRAMES,
            max_slot_bytes: MAX_AV_SLOT_BYTES,
            wall: MAX_AV_WALL,
            max_trace_bytes: MAX_TRACE_BYTES,
            fault_after_allocs: None,
        }
    }
}

/// Cancellation handle: an atomic flag owned by the host (probe driver /
/// worker).
pub type CancelFlag = std::sync::atomic::AtomicBool;

/// The frozen cancellation contract: a single relaxed load per poll.
pub struct CancelCheck;

impl CancelCheck {
    pub fn poll(flag: &CancelFlag) -> bool {
        flag.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// B3-C03: the process-wide AV execution sequence — every entry-point
/// execution takes the next number. A publication whose dispatched-
/// payload record carries seq N while the CURRENT seq is > N describes
/// an execution that is not the latest one (e.g. a redundant deep-copy
/// execution ran after the recorded run) — the formal judge refuses it.
static EXECUTION_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The current process-wide AV execution sequence (see [`EXECUTION_SEQ`]).
pub fn execution_seq() -> u64 {
    EXECUTION_SEQ.load(std::sync::atomic::Ordering::Relaxed)
}

#[derive(Clone, Debug, Default)]
pub struct ExecTrace {
    pub events: Vec<TraceEvent>,
    /// PLAN-821: the execution's owned-text arena (allocated even for
    /// non-owned profiles — an empty ledger; `None` keeps the struct
    /// cheap on the historic paths).
    pub arena: Option<crate::owned_text::StrArena>,
    /// This execution's process-wide sequence number (B3-C03).
    pub execution_seq: u64,
    /// High-water live slot data across all frames during execution (the
    /// execution partition of the managed 32 MiB budget, §3).
    pub peak_slot_bytes: u64,
    /// B3-C03 (F-02 axis AT THE EXECUTOR): the payload identities the
    /// executor dispatched for THE program passed to this execution —
    /// object address, mint generation, and a content digest over the
    /// same Debug view `ProgramSnapshot::dispatched_payload_same_content`
    /// compares. Collected inside the execution entries, so this evidence
    /// always describes the program that produced the trace — never a
    /// wrapper's view of some other snapshot it chose not to execute.
    pub dispatched: Vec<DispatchedPayload>,
    pub loaded_index_builds: u64,
    pub loaded_index_reuses: u64,
    pub loaded_index_bytes: u64,
    pub loaded_pc_lookups: u64,
    pub reference_pc_lookups: u64,
    pub loaded_param_map_queries: u64,
    pub reference_param_map_copies: u64,
    pub root_frame_commits: u64,
    pub call_frame_commits: u64,
    pub frames_remaining: u64,
}

/// One dispatched payload's identity evidence (see [`ExecTrace::dispatched`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DispatchedPayload {
    /// The function's stable def-id name (`d_main`).
    pub name: String,
    /// The `FunctionCode` Arc object address — the identity axis
    /// (`Arc::ptr_eq` semantics).
    pub ptr: u64,
    /// The code object's mint generation.
    pub generation: u64,
    /// FNV-1a over the unit payload's Debug view (the content axis —
    /// equal digests ⇔ `dispatched_payload_same_content` equality).
    pub content: u64,
}

/// Collect the dispatch identity evidence for the program THIS execution
/// is about to run (all four public entries route through here).
fn observe_dispatched(prog: &crate::program_snapshot::ProgramSnapshot) -> Vec<DispatchedPayload> {
    use std::sync::Arc;
    prog.code()
        .iter()
        .map(|c| DispatchedPayload {
            name: c.name().to_string(),
            ptr: Arc::as_ptr(c) as u64,
            generation: c.generation(),
            content: c
                .unit()
                .module()
                .funcs
                .first()
                .map(|f| fnv64(format!("{f:?}").as_bytes()))
                .unwrap_or(0),
        })
        .collect()
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

struct Frame {
    func: u32,
    block: u32,
    ip: usize,
    slots: Vec<Value>,
    /// Per-slot WRITE generation (PLAN-821 V-05): bumped by every
    /// [`commit_slot_write`]; a frame-rooted view/mut handle whose recorded
    /// generation no longer matches is stale and rejected at use.
    gens: Vec<u64>,
    /// This frame ACTIVATION's identity (monotonic; the MutRef token's
    /// runtime rooting key — PLAN-821 T-03).
    seq: u64,
    _allocation: Option<crate::frame_allocation::AllocationLease>,
    /// Caller bookkeeping: destination slot written on return (None = root).
    ret_dst: Option<u32>,
    /// PLAN-823: the closure-env-store handle this frame ACTIVATION is
    /// bound to (set by CALL_CLOSURE; the env itself is owned by the
    /// store/closure value, never by the frame).
    closure_env: Option<u64>,
}

/// Evaluation error: language traps vs. admission-violation internals stay
/// separated even inside the arithmetic helpers.
enum EvalErr {
    Trap(TrapKind),
    Infra(String),
}

/// Execute the loaded module's entry function under the given limits.
/// The entry contract is zero-params/i32 (enforced at admission).
pub fn execute(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
) -> (ExecOutcome, ExecTrace) {
    execute_entry(module, limits, cancel, &[])
}

pub fn execute_reference(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
) -> (ExecOutcome, ExecTrace) {
    execute_entry_reference(module, limits, cancel, &[])
}

pub fn execute_with_arena(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    arena: crate::owned_text::StrArena,
) -> (ExecOutcome, ExecTrace) {
    execute_module_impl(module, limits, cancel, &[], arena, true)
}

pub fn execute_entry(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    args: &[Value],
) -> (ExecOutcome, ExecTrace) {
    execute_module_impl(
        module,
        limits,
        cancel,
        args,
        crate::owned_text::StrArena::new(),
        true,
    )
}

pub fn execute_entry_reference(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    args: &[Value],
) -> (ExecOutcome, ExecTrace) {
    execute_module_impl(
        module,
        limits,
        cancel,
        args,
        crate::owned_text::StrArena::new(),
        false,
    )
}

fn execute_module_impl(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    args: &[Value],
    arena: crate::owned_text::StrArena,
    optimize: bool,
) -> (ExecOutcome, ExecTrace) {
    execute_module_impl_with_host(module, limits, cancel, args, arena, optimize, None)
}

/// PLAN-863: structural credentials carry no authority; current grants are checked per run.
pub fn execute_console(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    grants: &[String],
    provider: &mut dyn crate::host::ConsoleHost,
) -> Result<(ExecOutcome, ExecTrace), crate::host::AdmissionError> {
    crate::host::authorize(module.module(), grants)?;
    Ok(execute_module_impl_with_host(
        module,
        limits,
        cancel,
        &[],
        crate::owned_text::StrArena::new(),
        false,
        Some(provider),
    ))
}

fn execute_module_impl_with_host(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    args: &[Value],
    arena: crate::owned_text::StrArena,
    optimize: bool,
    host: Option<&mut dyn crate::host::ConsoleHost>,
) -> (ExecOutcome, ExecTrace) {
    let m = module.module();
    if !m.host_imports.is_empty() && host.is_none() {
        return (
            ExecOutcome::Infra(
                "host.unauthorized: explicit console execution/provider required".into(),
            ),
            ExecTrace::default(),
        );
    }
    if !module.executable() {
        let unit_kind = m.is_unit;
        return (
            ExecOutcome::Infra(format!(
                "admission violation: credential is a re-verifiable unit ({}), not an executable closure — close it into a whole module first",
                if unit_kind { "unit kind" } else { "unresolved import call" }
            )),
            ExecTrace::default(),
        );
    }
    let loaded = if optimize && m.profile == crate::contract::AV_PROFILE {
        match crate::loaded_program::LoadedProgram::build(module, limits.max_slot_bytes) {
            Ok(loaded) => Some(loaded),
            Err(crate::loaded_program::IndexError::Budget) => {
                return (
                    ExecOutcome::Resource {
                        kind: ResourceKind::SlotBytes,
                        message: format!(
                            "loaded-program indexes exceed execution budget {}",
                            limits.max_slot_bytes
                        ),
                    },
                    ExecTrace::default(),
                )
            }
            Err(e) => {
                return (
                    ExecOutcome::Infra(format!("loaded-program preparation: {e}")),
                    ExecTrace::default(),
                )
            }
        }
    } else {
        None
    };
    let retained = host.as_ref().map_or(0, |h| h.retained_bytes());
    if retained > crate::host::MAX_RUN_BYTES || retained > limits.max_slot_bytes {
        return (
            ExecOutcome::Resource {
                kind: ResourceKind::ConsoleOutput,
                message: "existing provider capture exceeds budget".into(),
            },
            ExecTrace::default(),
        );
    }
    let mut st = LoopState {
        slot_bytes: retained,
        output_bytes: retained,
        arena,
        ..Default::default()
    };
    st.trace.arena = Some(st.arena.clone());
    let index_bytes = loaded.as_ref().map_or(0, |loaded| loaded.usage().0);
    let limits = match limits_with_loaded_index(limits, index_bytes) {
        Ok(limits) => limits,
        Err(out) => return (out, st.trace),
    };
    let out = if let Some(loaded) = loaded.as_ref() {
        st.trace.loaded_index_builds = loaded.builds();
        st.trace.loaded_index_reuses = loaded.reuses();
        st.trace.loaded_index_bytes = loaded.usage().1 as u64;
        dispatch_with_host(
            &LoadedView(m, loaded),
            &limits,
            cancel,
            args,
            &mut st,
            None,
            false,
            host,
        )
    } else {
        dispatch_with_host(
            &MergedView(m),
            &limits,
            cancel,
            args,
            &mut st,
            None,
            false,
            host,
        )
    }
    .expect("no budget: the loop always terminates with an outcome");
    if !st.frames.is_empty() {
        cleanup_owned_frames(&mut st);
    }
    st.trace.frames_remaining = st.frames.len() as u64;
    (out, st.trace)
}

fn limits_with_loaded_index(
    limits: &ExecLimits,
    index_bytes: usize,
) -> Result<ExecLimits, ExecOutcome> {
    let mut remaining = *limits;
    remaining.max_slot_bytes = limits
        .max_slot_bytes
        .checked_sub(index_bytes)
        .ok_or_else(|| ExecOutcome::Resource {
            kind: ResourceKind::SlotBytes,
            message: format!(
                "resident indexes {index_bytes} bytes exceed execution budget {}",
                limits.max_slot_bytes
            ),
        })?;
    Ok(remaining)
}

/// Execute a linked [`ProgramSnapshot`]'s entry (PLAN-828 T-01): the
/// snapshot's merged view is dispatchable BY CONSTRUCTION — it was linked
/// from ABC-admitted single-function units with signature-checked call
/// bindings, so no whole-closure credential exists or is needed here. A new
/// top-level call captures the snapshot it wants to see; the immutable
/// snapshot keeps in-flight chains on their own revision.
/// PLAN-828 D-02 first case: execute the entry through the UNIT view —
/// the original per-function payloads with link-time call resolution
/// (no merged re-materialization). This prototype and the three merged
/// entries share the ONE `dispatch`; T-05/06 switch the snapshot link
/// to binding tables and make this the default path.
pub fn execute_program_units(
    prog: &crate::program_snapshot::ProgramSnapshot,
    limits: &ExecLimits,
    cancel: &CancelFlag,
) -> (ExecOutcome, ExecTrace) {
    execute_program(prog, limits, cancel)
}

pub fn execute_program(
    prog: &crate::program_snapshot::ProgramSnapshot,
    limits: &ExecLimits,
    cancel: &CancelFlag,
) -> (ExecOutcome, ExecTrace) {
    execute_program_impl(prog, limits, cancel, true)
}

pub fn execute_program_reference(
    prog: &crate::program_snapshot::ProgramSnapshot,
    limits: &ExecLimits,
    cancel: &CancelFlag,
) -> (ExecOutcome, ExecTrace) {
    execute_program_impl(prog, limits, cancel, false)
}

fn execute_program_impl(
    prog: &crate::program_snapshot::ProgramSnapshot,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    optimize: bool,
) -> (ExecOutcome, ExecTrace) {
    let mut st = LoopState {
        arena: crate::owned_text::StrArena::new(),
        ..Default::default()
    };
    st.trace.arena = Some(st.arena.clone());
    st.trace.execution_seq = EXECUTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    st.trace.dispatched = observe_dispatched(prog);
    if optimize {
        st.trace.loaded_index_builds = prog.loaded_program().builds();
        st.trace.loaded_index_reuses = prog.loaded_program().reuses();
        st.trace.loaded_index_bytes = prog.loaded_program().usage().1 as u64;
    }
    let index_bytes = optimize
        .then(|| prog.loaded_program().usage().0)
        .unwrap_or(0);
    let limits = match limits_with_loaded_index(limits, index_bytes) {
        Ok(limits) => limits,
        Err(out) => return (out, st.trace),
    };
    let view = if optimize {
        prog.unit_view()
    } else {
        prog.reference_unit_view()
    };
    let out = dispatch(&view, &limits, cancel, &[], &mut st, None, false)
        .expect("no budget: the loop always terminates with an outcome");
    if !st.frames.is_empty() {
        cleanup_owned_frames(&mut st);
    }
    st.trace.frames_remaining = st.frames.len() as u64;
    (out, st.trace)
}

/// A paused in-flight execution (PLAN-828 T-06): the live call chain plus
/// the PINNED snapshot it runs against. Resuming continues the same chain
/// over the same revision — an install that lands while this run is parked
/// cannot mix callee versions into it.
pub struct PausedRun {
    prog: std::sync::Arc<crate::program_snapshot::ProgramSnapshot>,
    limits: ExecLimits,
    state: LoopState,
    /// PLAN-866: this pause parked the chain exactly at a guest call
    /// boundary (right before the next `Call`/`CallClosure` dispatch) —
    /// the online-reload safety point, as opposed to a plain fuel
    /// boundary.
    at_call_boundary: bool,
}

impl PausedRun {
    /// Cumulative guest instruction steps executed so far across all
    /// segments of this chain (B2-C04 observability: a zero-budget
    /// resume must not move it).
    pub fn step_count(&self) -> u64 {
        self.state.steps
    }

    /// Whether this pause is a PLAN-866 call-boundary safety point (true
    /// only for pauses produced by `resume_program_at_call_boundary`).
    pub fn at_call_boundary(&self) -> bool {
        self.at_call_boundary
    }

    /// The pinned snapshot this chain currently executes against (its
    /// resolution table). PLAN-866 release oracle: downgrade this before
    /// a swapped resume to observe the retired generation die with its
    /// last in-flight frame.
    pub fn prog(&self) -> &std::sync::Arc<crate::program_snapshot::ProgramSnapshot> {
        &self.prog
    }

    /// The function-slot indices currently live on this chain's frame
    /// stack (innermost last). PLAN-866 deferral guard: the host refuses
    /// to swap a revision whose changed function is still executing —
    /// its old body runs to return, the new one waits for the next run
    /// (frozen contract §2.3).
    pub fn frame_funcs(&self) -> Vec<u32> {
        self.state.frames.iter().map(|f| f.func).collect()
    }

    /// The current closure frame and its published environment, if live.
    pub fn active_closure_frame(&self) -> Option<(&str, u64)> {
        let frame = self.state.frames.last()?;
        let handle = frame.closure_env?;
        if !self.state.closure_envs.contains_key(&handle) {
            return None;
        }
        Some((self.prog.code().get(frame.func as usize)?.name(), handle))
    }

    /// The paused chain's owned-text ledger — the read-only seam for
    /// mid-run state proofs (R1-F04: proving the committed env state
    /// before triggering a cancellation).
    pub fn arena(&self) -> &crate::owned_text::StrArena {
        &self.state.arena
    }
}

/// Result of a possibly-paused program execution.
pub enum SteppedOutcome {
    Finished(ExecOutcome, ExecTrace),
    Paused(PausedRun),
}

/// Execute up to `max_steps` guest instructions of the snapshot's entry,
/// then park the chain deterministically (no sleeps, no races — the
/// boundary is a step count). Fuel/frames/slot budgets and the trace carry
/// into the [`PausedRun`]; the wall budget applies per execution segment.
pub fn execute_program_steps(
    prog: &std::sync::Arc<crate::program_snapshot::ProgramSnapshot>,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    max_steps: u64,
) -> SteppedOutcome {
    let mut st = LoopState {
        arena: crate::owned_text::StrArena::new(),
        ..Default::default()
    };
    st.trace.arena = Some(st.arena.clone());
    st.trace.execution_seq = EXECUTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    st.trace.dispatched = observe_dispatched(prog);
    st.trace.loaded_index_builds = prog.loaded_program().builds();
    st.trace.loaded_index_reuses = prog.loaded_program().reuses();
    st.trace.loaded_index_bytes = prog.loaded_program().usage().1 as u64;
    let dispatch_limits = match limits_with_loaded_index(limits, prog.loaded_program().usage().0) {
        Ok(limits) => limits,
        Err(out) => return SteppedOutcome::Finished(out, st.trace),
    };
    match dispatch(
        &prog.unit_view(),
        &dispatch_limits,
        cancel,
        &[],
        &mut st,
        Some(max_steps),
        false,
    ) {
        Some(out) => {
            if !st.frames.is_empty() {
                cleanup_owned_frames(&mut st);
            }
            st.trace.frames_remaining = st.frames.len() as u64;
            SteppedOutcome::Finished(out, st.trace)
        }
        None => SteppedOutcome::Paused(PausedRun {
            prog: prog.clone(),
            limits: *limits,
            state: st,
            at_call_boundary: false,
        }),
    }
}

/// Continue a paused run (optionally up to another step budget; `None` =
/// run to completion). The chain keeps its original snapshot.
pub fn resume_program(
    paused: PausedRun,
    cancel: &CancelFlag,
    max_steps: Option<u64>,
) -> SteppedOutcome {
    resume_program_mode(paused, cancel, max_steps, ResumeMode::Plain)
}

/// Continue a paused run and park it right BEFORE the next guest call
/// dispatch (`Call`/`CallClosure`) — the PLAN-866 safety point: one
/// completed call boundary, independent of host fuel slicing. The
/// returned pause reports [`PausedRun::at_call_boundary`]; a plain fuel
/// pause inside this same segment does not. A chain that completes
/// without meeting another call returns `Finished` (the caller commits
/// at chain end — trivially "after all calls").
pub fn resume_program_at_call_boundary(
    paused: PausedRun,
    cancel: &CancelFlag,
    max_steps: Option<u64>,
) -> SteppedOutcome {
    resume_program_mode(paused, cancel, max_steps, ResumeMode::AtCallBoundary)
}

/// Continue a paused run with callee RESOLUTION redirected to
/// `resolution` (PLAN-866 online swap): already-dispatched frames keep
/// executing their pinned code objects — the old generation — to return,
/// while every call dispatched from here on resolves through the NEW
/// snapshot. The caller must guarantee the 828 outline-stable body-edit
/// contract (unchanged declaration set and order), which keeps function
/// slot indices comparable across the two snapshots; this segment's
/// dispatch evidence is re-observed from the resolution snapshot it
/// actually dispatches against.
pub fn resume_program_with_resolution(
    paused: PausedRun,
    cancel: &CancelFlag,
    max_steps: Option<u64>,
    resolution: &std::sync::Arc<crate::program_snapshot::ProgramSnapshot>,
) -> SteppedOutcome {
    resume_program_mode(
        paused,
        cancel,
        max_steps,
        ResumeMode::Swapped(resolution.clone()),
    )
}

enum ResumeMode {
    Plain,
    AtCallBoundary,
    Swapped(std::sync::Arc<crate::program_snapshot::ProgramSnapshot>),
}

fn resume_program_mode(
    paused: PausedRun,
    cancel: &CancelFlag,
    max_steps: Option<u64>,
    mode: ResumeMode,
) -> SteppedOutcome {
    let PausedRun {
        mut prog,
        limits,
        mut state,
        ..
    } = paused;
    if let ResumeMode::Swapped(resolution) = &mode {
        prog = resolution.clone();
    }
    // The resumed segment keeps the chain's dispatch evidence — the
    // pinned snapshot it actually continues to execute — and takes its
    // own sequence number (each entry-point execution is one). A swapped
    // segment re-observes from the RESOLUTION snapshot: it is the
    // program this segment's callee dispatches resolve against.
    state.trace.execution_seq =
        EXECUTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    if state.trace.dispatched.is_empty() || matches!(mode, ResumeMode::Swapped(_)) {
        state.trace.dispatched = observe_dispatched(&prog);
    }
    let dispatch_limits = match limits_with_loaded_index(&limits, prog.loaded_program().usage().0) {
        Ok(limits) => limits,
        Err(out) => {
            if !state.frames.is_empty() {
                cleanup_owned_frames(&mut state);
            }
            state.trace.frames_remaining = state.frames.len() as u64;
            return SteppedOutcome::Finished(out, state.trace);
        }
    };
    let stop_before_call = matches!(mode, ResumeMode::AtCallBoundary);
    match dispatch(
        &prog.unit_view(),
        &dispatch_limits,
        cancel,
        &[],
        &mut state,
        max_steps,
        stop_before_call,
    ) {
        Some(out) => {
            if !state.frames.is_empty() {
                cleanup_owned_frames(&mut state);
            }
            state.trace.frames_remaining = state.frames.len() as u64;
            SteppedOutcome::Finished(out, state.trace)
        }
        None => SteppedOutcome::Paused(PausedRun {
            prog,
            limits,
            state,
            at_call_boundary: stop_before_call,
        }),
    }
}

/// Resumable interpreter state: the live call chain of one top-level run.
#[derive(Default)]
struct LoopState {
    output_bytes: usize,
    frames: Vec<Frame>,
    slot_bytes: usize,
    steps: u64,
    trace: ExecTrace,
    /// Value-allocation events (V-07 observability: budget-rejections
    /// before the attempt report 0; injected faults report the count).
    alloc_events: u64,
    /// Step counter at the last cancellation poll.
    last_poll: u64,
    /// PLAN-821 owned-text ledger: every owned Str allocation/free event
    /// (ids in creation order; the LIFO/release oracles read the traces).
    arena: crate::owned_text::StrArena,
    /// Monotonic frame ACTIVATION sequence (V-05 handle domain).
    frame_seq: u64,
    /// PLAN-823 closure env store: handle → the owned env aggregate. The
    /// store outlives frames (a closure escapes its creator's frame); env
    /// bytes count toward `slot_bytes` (the one value-data line).
    closure_envs: std::collections::HashMap<u64, ClosureEntry>,
}

struct ClosureEntry {
    env: crate::closures::ClosureEnv,
    func: u32,
    sig: u32,
    borrows: Vec<ClosureBorrow>,
}

#[derive(Clone, Copy)]
struct ClosureBorrow {
    frame: u64,
    slot: u32,
    generation: u64,
}

fn closure_borrows_valid(st: &LoopState, cref: crate::contract::ClosureRefV) -> bool {
    st.closure_envs
        .get(&cref.env)
        .map(|entry| {
            entry.borrows.iter().all(|b| {
                st.frames
                    .iter()
                    .find(|frame| frame.seq == b.frame)
                    .and_then(|frame| frame.gens.get(b.slot as usize))
                    == Some(&b.generation)
            })
        })
        .unwrap_or(false)
}

fn closure_binding_bytes(fields: usize, borrows: usize) -> usize {
    // The entry includes the environment and vector headers; its backing
    // field values are charged by deep_size, while moved flags and borrow
    // metadata are charged here.
    std::mem::size_of::<ClosureEntry>()
        .saturating_add(fields.saturating_mul(std::mem::size_of::<bool>()))
        .saturating_add(borrows.saturating_mul(std::mem::size_of::<ClosureBorrow>()))
}
static NEXT_CLOSURE_HANDLE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn reserve_closure_handle() -> Option<u64> {
    use std::sync::atomic::Ordering;
    NEXT_CLOSURE_HANDLE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .ok()
}

fn closure_ref_valid(st: &LoopState, cref: crate::contract::ClosureRefV) -> bool {
    cref.gen == cref.env
        && st
            .closure_envs
            .get(&cref.env)
            .map(|entry| entry.func == cref.func && entry.sig == cref.sig)
            .unwrap_or(false)
}

/// The shared flat dispatch loop: explicit frames, budgets and traps.
/// Fresh state seeds the root frame (entry lookup, argument admission,
/// root budgets); non-empty state resumes the chain untouched. Returns
/// `None` when the step budget pauses the chain; the wall budget applies
/// per call (per execution segment).
/// PLAN-828 D-02: the ONE dispatch consumption seam over a program's
/// functions. `MergedView` reads the snapshot's merged module (all three
/// public entries today); the unit view (program_snapshot.rs) reads each
/// function's ORIGINAL unit payload — blocks/slots/PC/consts/types stay
/// UNIT-LOCAL — and resolves cross-function calls through the link-time
/// binding table instead of a re-materialized merged copy.
pub(crate) trait ModuleView {
    fn host_imports(&self) -> &[crate::mir::MirHostImport] {
        &[]
    }
    /// The function payload execution reads (blocks/slots/spans).
    fn fn_at(&self, idx: u32) -> Option<&MirFunction>;
    /// A constant by the index space function `func`'s instructions use
    /// (merged: global; unit: fn-local).
    fn const_at(&self, func: u32, idx: u32) -> Option<crate::mir::MirConst>;
    /// The typed zero for a slot type index in `func`'s type space. Record
    /// slots receive the documented empty-fields placeholder (never read:
    /// definite initialization gates every read).
    fn zero_of(&self, func: u32, ty: u32) -> Value;
    /// The scalar shape of a type index in `func`'s type space (`None` =
    /// not a scalar — Fn or Record; entry-arg admission fails on None).
    fn scalar_shape_of(&self, func: u32, ty: u32) -> Option<crate::mir::ScalarShape>;
    /// Resolve a call site's callee to a function slot (None = an
    /// admission violation: unresolved import or foreign index space).
    fn resolve_callee(&self, func: u32, callee: &CalleeRef) -> Option<u32>;
    /// Whether a type index denotes `StrView` (PLAN-821; drives the
    /// field-get borrow-view materialization).
    fn is_strview_type(&self, func: u32, ty: u32) -> bool;
    /// The raw MIR type of an index (PLAN-821 entry admission for char).
    fn mir_type_of(&self, func: u32, ty: u32) -> Option<&crate::mir::MirType>;
    /// Whether a type index OWNS resources (Str / resource record) —
    /// move-in CALL semantics and slot kills (PLAN-821).
    fn type_is_owned(&self, func: u32, ty: u32) -> bool;
    fn logical_pc(&self, func: u32, block: u32, ip: usize) -> Option<(bool, u32)> {
        let f = self.fn_at(func)?;
        Some((false, f.block_base(block) + ip as u32))
    }
    fn param_slots(&self, _func: u32) -> Option<&[u32]> {
        None
    }

    /// Project a TRAP span of the EXECUTING function (frame slot `func`)
    /// into the outcome's published coordinates. The default is the
    /// identity — module-level views publish their own payload
    /// coordinates; the snapshot UNIT view maps through ITS OWN
    /// per-code-slot source map (B2-C03).
    fn map_span(&self, _func: u32, span: SpanInfo) -> SpanInfo {
        span
    }
    /// The entry function slot.
    fn entry(&self) -> u32;
}

/// The whole-module view over the merged MirModule (today's behavior,
/// unchanged).
pub(crate) struct MergedView<'a>(pub &'a MirModule);

impl ModuleView for MergedView<'_> {
    fn host_imports(&self) -> &[crate::mir::MirHostImport] {
        &self.0.host_imports
    }
    fn fn_at(&self, idx: u32) -> Option<&MirFunction> {
        self.0.funcs.get(idx as usize)
    }
    fn const_at(&self, _func: u32, idx: u32) -> Option<crate::mir::MirConst> {
        self.0.consts.get(idx as usize).cloned()
    }
    fn zero_of(&self, _func: u32, ty: u32) -> Value {
        typed_zero(self.0.types.get(ty as usize))
    }
    fn scalar_shape_of(&self, _func: u32, ty: u32) -> Option<crate::mir::ScalarShape> {
        self.0.type_shape(ty)
    }
    fn resolve_callee(&self, _func: u32, callee: &CalleeRef) -> Option<u32> {
        match callee {
            CalleeRef::Func(idx) => self.0.funcs.get(*idx as usize).map(|_| *idx),
            CalleeRef::Import(_) => None,
        }
    }
    fn is_strview_type(&self, _func: u32, ty: u32) -> bool {
        matches!(
            self.0.types.get(ty as usize),
            Some(crate::mir::MirType::StrView)
        )
    }
    fn mir_type_of(&self, _func: u32, ty: u32) -> Option<&crate::mir::MirType> {
        self.0.types.get(ty as usize)
    }
    fn type_is_owned(&self, _func: u32, ty: u32) -> bool {
        crate::ownership::type_is_owned(self.0, ty)
    }
    fn entry(&self) -> u32 {
        self.0.entry
    }
}

struct LoadedView<'a>(&'a MirModule, &'a crate::loaded_program::LoadedProgram<'a>);

impl ModuleView for LoadedView<'_> {
    fn fn_at(&self, idx: u32) -> Option<&MirFunction> {
        self.0.funcs.get(idx as usize)
    }
    fn const_at(&self, _func: u32, idx: u32) -> Option<crate::mir::MirConst> {
        self.0.consts.get(idx as usize).cloned()
    }
    fn zero_of(&self, _func: u32, ty: u32) -> Value {
        typed_zero(self.0.types.get(ty as usize))
    }
    fn scalar_shape_of(&self, _func: u32, ty: u32) -> Option<crate::mir::ScalarShape> {
        self.0.type_shape(ty)
    }
    fn resolve_callee(&self, _func: u32, callee: &CalleeRef) -> Option<u32> {
        match callee {
            CalleeRef::Func(idx) => self.0.funcs.get(*idx as usize).map(|_| *idx),
            CalleeRef::Import(_) => None,
        }
    }
    fn is_strview_type(&self, _func: u32, ty: u32) -> bool {
        matches!(self.0.types.get(ty as usize), Some(MirType::StrView))
    }
    fn mir_type_of(&self, _func: u32, ty: u32) -> Option<&MirType> {
        self.0.types.get(ty as usize)
    }
    fn type_is_owned(&self, _func: u32, ty: u32) -> bool {
        crate::ownership::type_is_owned(self.0, ty)
    }
    fn logical_pc(&self, func: u32, block: u32, ip: usize) -> Option<(bool, u32)> {
        self.1.logical_pc(func, block, ip).map(|pc| (true, pc))
    }
    fn param_slots(&self, func: u32) -> Option<&[u32]> {
        self.1.param_slots(func)
    }
    fn entry(&self) -> u32 {
        self.0.entry
    }
}

/// The typed zero / never-read placeholder for a slot type (see
/// [`ModuleView::zero_of`]). The record placeholder carries `ty: u32::MAX`
/// and no fields — distinct from any admitted record value (verification
/// guarantees record slots are written by AGGREGATE/FIELD_SET/Copy/param
/// before any read).
pub(crate) fn typed_zero(t: Option<&MirType>) -> Value {
    match t {
        // PLAN-862: the unit value — a real value of the zero-width result
        // type (NOT a placeholder for `I32(0)`). The verifier admits a Unit
        // slot only as the Unit entry's return operand, so this is the value
        // that entry's `Return` hands back.
        Some(MirType::Unit) => Value::Unit,
        Some(MirType::Bool) => Value::Bool(false),
        Some(MirType::I64) => Value::I64(0),
        Some(MirType::U64) => Value::U64(0),
        Some(MirType::F32) => Value::F32(f32::from_bits(0)),
        Some(MirType::F64) => Value::F64(f64::from_bits(0)),
        Some(MirType::Record(_)) => Value::Record(std::rc::Rc::new(crate::contract::RecordValue {
            ty: u32::MAX,
            fields: Vec::new(),
        })),
        // PLAN-821 placeholders: definite initialization gates every read,
        // so these are never observed (the detached Str carries NO arena
        // hook — no free event can fire for a placeholder).
        Some(MirType::Char) => Value::Char('\0'),
        Some(MirType::Str) => Value::Str(crate::contract::StrHandle::detached(Box::from([]))),
        Some(MirType::RecordView(_)) => {
            Value::Record(std::rc::Rc::new(crate::contract::RecordValue {
                ty: u32::MAX,
                fields: Vec::new(),
            }))
        }
        Some(MirType::StrView) => Value::StrView(crate::contract::StrView {
            // Never observed (definite initialization gates every read):
            // an empty immortal buffer (F821-R1-03 shape).
            src: crate::contract::StrViewSrc::Static {
                buf: std::rc::Rc::new(Box::from([])),
            },
            start: 0,
            end: 0,
        }),
        Some(MirType::MutRef) => Value::MutRef(crate::contract::MutRefV {
            frame: u32::MAX,
            slot: u32::MAX,
            gen: 0,
        }),
        // PLAN-822 placeholders: never observed (definite initialization
        // gates every read); the detached list carries NO arena hook.
        Some(MirType::List(_)) => Value::List(crate::contract::ListHandle::detached(Vec::new())),
        Some(MirType::Variant(_)) => Value::Variant(crate::contract::VariantValue {
            ty: u32::MAX,
            disc: 0,
            payload: None,
        }),
        // PLAN-822 R2: a never-read place placeholder (definite
        // initialization gates every use; never observed).
        Some(MirType::Place(_)) => Value::Place(crate::contract::ListPlaceV {
            frame: u32::MAX,
            slot: u32::MAX,
            buf_id: u64::MAX,
            epoch: u64::MAX,
            index: u32::MAX,
            elem_gen: u32::MAX,
        }),
        // I32 and (unusable) Fn slots keep the historic scalar zero.
        _ => Value::I32(0),
    }
}

// -- value/budget helpers (PLAN-820 deep slot-data accounting) ----------------

/// Σ deep data size of one frame's slot values (the live-data accounting
/// base; `visited` accumulates field visits so callers charge fuel).
fn frame_slots_bytes(slots: &[Value], visited: &mut u64) -> usize {
    slots.iter().map(|s| s.deep_size(visited)).sum()
}

/// PLAN-824: resolve a module type index to the self-contained
/// [`crate::dynamic::DynType`] tree (the MIR→DynType adapter the Dyn
/// instructions consume). PATH-based cycle detection (a type table is a
/// DAG — shared children are fine, only a back-reference to the current
/// path is a cycle); the 256 depth bound matches the verifier's
/// `dyn_whitelisted`.
///
/// Every materialized node, name and vector allocation is checked against
/// the execution budget before construction; cancellation is polled during
/// the traversal rather than after the complete tree has been allocated.
enum DynTypeResolve {
    Ready(crate::dynamic::DynType),
    OutsideClosure,
    Stopped(ExecOutcome),
}

fn dyn_exec_work(
    st: &mut LoopState,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    started: Instant,
) -> Result<(), ExecOutcome> {
    st.steps = st.steps.saturating_add(1);
    if st.steps > limits.max_steps {
        return Err(ExecOutcome::Resource {
            kind: ResourceKind::InstrSteps,
            message: format!("execution exceeded {} instruction steps", limits.max_steps),
        });
    }
    if st.steps - st.last_poll >= CANCEL_CHECK_INTERVAL {
        st.last_poll = st.steps;
        if CancelCheck::poll(cancel) {
            return Err(ExecOutcome::Cancelled);
        }
        if started.elapsed() > limits.wall {
            return Err(ExecOutcome::Resource {
                kind: ResourceKind::ExecutionWall,
                message: "execution exceeded wall budget".to_string(),
            });
        }
    }
    Ok(())
}

/// Compare only after charging every visited node and every field/arm,
/// including payload-less arms, to the executor's fuel and poll cadence.
fn dyn_type_equal(
    a: &crate::dynamic::DynType,
    b: &crate::dynamic::DynType,
    st: &mut LoopState,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    started: Instant,
) -> Result<bool, ExecOutcome> {
    use crate::dynamic::DynType;
    let mut stack = vec![(a, b)];
    while let Some((a, b)) = stack.pop() {
        dyn_exec_work(st, limits, cancel, started)?;
        match (a, b) {
            (DynType::List(a), DynType::List(b)) => stack.push((a, b)),
            (
                DynType::Record {
                    owner: ao,
                    name: an,
                    fields: af,
                },
                DynType::Record {
                    owner: bo,
                    name: bn,
                    fields: bf,
                },
            ) => {
                if ao != bo || an != bn || af.len() != bf.len() {
                    return Ok(false);
                }
                for (x, y) in af.iter().zip(bf) {
                    dyn_exec_work(st, limits, cancel, started)?;
                    if x.name != y.name {
                        return Ok(false);
                    }
                    stack.push((&x.ty, &y.ty));
                }
            }
            (
                DynType::Variant {
                    owner: ao,
                    name: an,
                    arms: aa,
                },
                DynType::Variant {
                    owner: bo,
                    name: bn,
                    arms: ba,
                },
            ) => {
                if ao != bo || an != bn || aa.len() != ba.len() {
                    return Ok(false);
                }
                for (x, y) in aa.iter().zip(ba) {
                    dyn_exec_work(st, limits, cancel, started)?;
                    if x.name != y.name {
                        return Ok(false);
                    }
                    match (&x.payload, &y.payload) {
                        (Some(x), Some(y)) => stack.push((x, y)),
                        (None, None) => {}
                        _ => return Ok(false),
                    }
                }
            }
            (DynType::Bool, DynType::Bool)
            | (DynType::I32, DynType::I32)
            | (DynType::I64, DynType::I64)
            | (DynType::U64, DynType::U64)
            | (DynType::F32, DynType::F32)
            | (DynType::F64, DynType::F64)
            | (DynType::Char, DynType::Char)
            | (DynType::Str, DynType::Str) => {}
            _ => return Ok(false),
        }
    }
    Ok(true)
}

/// Account each allocation and unit of work before constructing the owned
/// descriptor. The temporary tree shares the execution data partition with
/// live slots, and its high water is observable even for is/take operands.
fn resolve_dyn_type<V: ModuleView>(
    v: &V,
    func: u32,
    idx: u32,
    st: &mut LoopState,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    started: Instant,
) -> DynTypeResolve {
    use crate::dynamic::{DynArm, DynField, DynType, MAX_DYN_NODES};
    use crate::mir::MirType;
    struct Budget {
        nodes: u64,
        bytes: usize,
    }
    fn reserve(
        b: &mut Budget,
        n: usize,
        st: &mut LoopState,
        limits: &ExecLimits,
    ) -> Result<(), ExecOutcome> {
        let next = b.bytes.saturating_add(n);
        let total = st.slot_bytes.saturating_add(next);
        if next > crate::dynamic::MAX_DYN_BYTES || total > limits.max_slot_bytes {
            return Err(ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                message: format!("dyn type temporary data exceeds execution byte budget ({total})"),
            });
        }
        b.bytes = next;
        st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(total as u64);
        Ok(())
    }
    fn go<V: ModuleView>(
        v: &V,
        func: u32,
        idx: u32,
        depth: u32,
        path: &mut Vec<u32>,
        b: &mut Budget,
        embedded: bool,
        st: &mut LoopState,
        limits: &ExecLimits,
        cancel: &CancelFlag,
        started: Instant,
    ) -> Result<Option<DynType>, ExecOutcome> {
        if depth > crate::dynamic::MAX_DYN_DEPTH || path.contains(&idx) {
            return Ok(None);
        }
        b.nodes += 1;
        if b.nodes > MAX_DYN_NODES as u64 {
            return Err(ExecOutcome::Resource {
                kind: ResourceKind::DynTypeNodes,
                message: format!("dyn type operand expands beyond {MAX_DYN_NODES} nodes"),
            });
        }
        dyn_exec_work(st, limits, cancel, started)?;
        if !embedded {
            reserve(b, std::mem::size_of::<DynType>(), st, limits)?;
        }
        let Some(ty) = v.mir_type_of(func, idx) else {
            return Ok(None);
        };
        path.push(idx);
        let out = match ty {
            MirType::Bool => Some(DynType::Bool),
            MirType::I32 => Some(DynType::I32),
            MirType::I64 => Some(DynType::I64),
            MirType::U64 => Some(DynType::U64),
            MirType::F32 => Some(DynType::F32),
            MirType::F64 => Some(DynType::F64),
            MirType::Char => Some(DynType::Char),
            MirType::Str => Some(DynType::Str),
            MirType::List(l) => match go(
                v,
                func,
                l.elem,
                depth + 1,
                path,
                b,
                false,
                st,
                limits,
                cancel,
                started,
            )? {
                Some(t) => Some(DynType::List(Box::new(t))),
                None => None,
            },
            MirType::Record(r) => {
                let extra = r
                    .owner_module
                    .len()
                    .saturating_add(r.name.len())
                    .saturating_add(
                        r.fields
                            .len()
                            .saturating_mul(std::mem::size_of::<DynField>()),
                    );
                reserve(b, extra, st, limits)?;
                let mut fields = Vec::with_capacity(r.fields.len());
                for f in &r.fields {
                    dyn_exec_work(st, limits, cancel, started)?;
                    reserve(b, f.name.len(), st, limits)?;
                    let Some(ty) = go(
                        v,
                        func,
                        f.ty,
                        depth + 1,
                        path,
                        b,
                        true,
                        st,
                        limits,
                        cancel,
                        started,
                    )?
                    else {
                        path.pop();
                        return Ok(None);
                    };
                    fields.push(DynField {
                        name: f.name.clone(),
                        ty,
                    });
                }
                Some(DynType::Record {
                    owner: r.owner_module.clone(),
                    name: r.name.clone(),
                    fields,
                })
            }
            MirType::Variant(vt) => {
                // Empty arms still occupy their inline Option<DynType>.
                let extra = vt
                    .owner_module
                    .len()
                    .saturating_add(vt.name.len())
                    .saturating_add(vt.arms.len().saturating_mul(std::mem::size_of::<DynArm>()));
                reserve(b, extra, st, limits)?;
                let mut arms = Vec::with_capacity(vt.arms.len());
                for a in &vt.arms {
                    dyn_exec_work(st, limits, cancel, started)?;
                    reserve(b, a.name.len(), st, limits)?;
                    let payload = if let Some(p) = a.payload {
                        match go(
                            v,
                            func,
                            p,
                            depth + 1,
                            path,
                            b,
                            true,
                            st,
                            limits,
                            cancel,
                            started,
                        )? {
                            Some(t) => Some(t),
                            None => {
                                path.pop();
                                return Ok(None);
                            }
                        }
                    } else {
                        None
                    };
                    arms.push(DynArm {
                        name: a.name.clone(),
                        payload,
                    });
                }
                Some(DynType::Variant {
                    owner: vt.owner_module.clone(),
                    name: vt.name.clone(),
                    arms,
                })
            }
            _ => None,
        };
        path.pop();
        Ok(out)
    }
    let mut b = Budget { nodes: 0, bytes: 0 };
    match go(
        v,
        func,
        idx,
        1,
        &mut Vec::new(),
        &mut b,
        false,
        st,
        limits,
        cancel,
        started,
    ) {
        Ok(Some(t)) => DynTypeResolve::Ready(t),
        Ok(None) => DynTypeResolve::OutsideClosure,
        Err(out) => DynTypeResolve::Stopped(out),
    }
}

/// Commit one slot write with deep delta accounting: a positive delta is
/// checked against the live-data budget BEFORE the write lands (a resource
/// outcome — the previous value stays in place). Fuel for the traversal is
/// charged by the caller from `visited`.
fn commit_slot_write(
    st: &mut LoopState,
    slot: usize,
    val: Value,
    limits: &ExecLimits,
) -> Result<(), ExecOutcome> {
    let mut visited = 0u64;
    let fr = st.frames.last_mut().expect("frame present");
    let old = fr.slots[slot].deep_size(&mut visited);
    let new = val.deep_size(&mut visited);
    st.steps += visited;
    fr.gens[slot] = fr.gens[slot].wrapping_add(1);
    if new > old {
        // V-07 fault seam: the Nth allocation fails WITHOUT applying the
        // write — the slot keeps its prior value and the ledger its prior
        // accounting (nothing half-committed).
        if let Some(n) = limits.fault_after_allocs {
            st.alloc_events += 1;
            if st.alloc_events == n {
                return Err(ExecOutcome::Infra(format!(
                    "fault-injected at value allocation #{}",
                    n
                )));
            }
        }
        let delta = new - old;
        if st.slot_bytes.saturating_add(delta) > limits.max_slot_bytes {
            return Err(ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                message: format!(
                    "live slot data would reach {} bytes, above budget {}",
                    st.slot_bytes + delta,
                    limits.max_slot_bytes
                ),
            });
        }
        st.slot_bytes += delta;
        st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
    }
    let dying = std::mem::replace(&mut fr.slots[slot], val);
    // (the frame borrow ends at the replace above — NLL)
    // F822-R12-01: the replaced value releases through the EXPLICIT
    // reverse cleanup — VariantSwitch's old live-arm payload, FieldSet's
    // old field chain, re-assignments over owned slots — never the host
    // drop glue, which recurses through arbitrarily deep chains.
    drop_owned_reverse(st, dying);
    Ok(())
}

impl Value {
    fn as_strview(&self) -> Option<&crate::contract::StrView> {
        match self {
            Value::StrView(v) => Some(v),
            _ => None,
        }
    }
}

/// Deep-clone one owned value (782 `clone()`): every reachable owned Str
/// field gets a FRESH independent buffer; scalars/copyable fields copy by
/// value. Iterative over the record closure (no host recursion).
/// PLAN-821 V-07 precheck: reject a prospective owned allocation BEFORE
/// any attempt is recorded (预算前拒 attempt=0) — the exact deep delta is
/// new_size - old_size of the destination slot.
fn owned_alloc_precheck(
    st: &LoopState,
    new_size: usize,
    dst: usize,
    limits: &ExecLimits,
) -> Result<(), ExecOutcome> {
    let mut visited = 0u64;
    let fr = st.frames.last().expect("frame present");
    let old = fr.slots[dst].deep_size(&mut visited);
    let delta = new_size.saturating_sub(old);
    if st.slot_bytes.saturating_add(delta) > limits.max_slot_bytes {
        return Err(ExecOutcome::Resource {
            kind: ResourceKind::SlotBytes,
            message: format!(
                "live slot data would reach {} bytes, above budget {}",
                st.slot_bytes + delta,
                limits.max_slot_bytes
            ),
        });
    }
    Ok(())
}

/// Deep clone of one owned value (782 `clone()`): iterative over the full
/// supported graph (R9 — no host recursion for any kind). Fuel is charged
/// INCREMENTALLY on the shared cumulative counter as the work happens: one
/// structural unit per visited/assembled node plus the OUTPUT byte volume
/// of every fresh Str buffer, charged BEFORE its copy (R11 — a clone's
/// cost scales with its data volume; a memory precheck alone is not
/// execution fuel). Cancellation and the wall budget poll at the same
/// shared-counter cadence as dispatch (`CANCEL_CHECK_INTERVAL`), so the
/// traversal is interruptible before and during data copies with the
/// partial assembly fully unwound.
/// Wall coordinate (R11): `wall_left` is the REMAINING allowance measured
/// from THIS helper entry — the dispatch caller derives it from the
/// segment budget, and helper entry + `wall_left` reproduces the segment's
/// absolute deadline exactly. No hidden thread-local origin participates:
/// the argument is the whole contract (a ZERO allowance rejects at the
/// entry poll).
/// Internal observation seam (F822-R10-01): exposed for the bounded-
/// traversal oracle tests — NOT a public execution API.
#[doc(hidden)]
pub fn clone_owned_value(
    val: &Value,
    arena: &crate::owned_text::StrArena,
    cancel: &CancelFlag,
    wall_left: std::time::Duration,
    fuel: &mut u64,
    max_fuel: u64,
) -> Result<Value, ExecOutcome> {
    // F821-R2/F06 + F822-R9-01: EXPLICIT-STACK deep clone — no host
    // recursion for ANY supported kind and NO language depth cutoff.
    // Phase 1 flattens the value graph with an explicit DFS worklist over
    // OWNED SHALLOW SNAPSHOTS (a Value clone is an Rc/handle bump, never
    // a content copy), so RefCell borrows never ride the plan; Record
    // fields, List elements and Variant payloads (nested Variants
    // included) are all children. Phase 2 assembles leaves-first: a
    // child's parent id is always SMALLER, so REVERSE id order has every
    // child ready when its parent reads it.
    struct Node {
        src: Value,
        parent: usize, // usize::MAX for the root
    }
    // F822-R11-01/02: ONE charge/poll discipline on the shared cumulative
    // counter — a structural unit per node in BOTH phases plus the OUTPUT
    // byte volume of each fresh Str buffer charged BEFORE its copy, with
    // cancel/wall polls at the dispatch cadence (a last_poll cursor, not
    // a modulo — batch byte charges cannot skip the boundary). The wall
    // compares the helper's OWN origin against the passed REMAINING
    // allowance; helper entry + wall_left reproduces the caller segment's
    // absolute deadline exactly (no double subtraction, no thread-local
    // leftovers), and a ZERO allowance rejects at the entry poll.
    let wall_origin = std::time::Instant::now();
    let mut last_poll = *fuel;
    macro_rules! poll_cancel_wall {
        () => {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(ExecOutcome::Cancelled);
            }
            if wall_origin.elapsed() >= wall_left {
                return Err(ExecOutcome::Resource {
                    kind: ResourceKind::ExecutionWall,
                    message: "deep clone exceeded the execution wall budget".to_string(),
                });
            }
        };
    }
    macro_rules! charge {
        ($units:expr) => {{
            *fuel = fuel.saturating_add($units as u64);
            if *fuel > max_fuel {
                return Err(ExecOutcome::Resource {
                    kind: ResourceKind::InstrSteps,
                    message: "deep clone exceeded the execution step budget".to_string(),
                });
            }
            if *fuel - last_poll >= crate::contract::CANCEL_CHECK_INTERVAL {
                last_poll = *fuel;
                poll_cancel_wall!();
            }
        }};
    }
    // F822-R13-01: EVERY aborting exit (fuel/cancel/wall/allocator fault)
    // funnels through ONE epilogue: the partially assembled output — the
    // `out` entries plus any in-arm temporaries parked in `drain_extras`
    // — releases through the SAME explicit last-owner worklist as the
    // dispatcher's cleanup (release_owned_worklist), never the host drop
    // glue: an abort mid-assembly of a deep chain would otherwise recurse
    // the glue and overflow the stack before the helper returns. The
    // drain is CHARGE-FREE (a scratch counter): the historical glue-drop
    // rollback accounted no cleanup fuel, so abort-path fuel totals stay
    // exactly what the work charged.
    let mut out: Vec<Option<Value>> = Vec::new();
    let mut drain_extras: Vec<Value> = Vec::new();
    let mut build = || -> Result<(), ExecOutcome> {
        // Entry poll: a pre-exhausted allowance (or a raised cancel flag)
        // rejects before ANY planning work — deterministic at any graph
        // size and independent of prior calls on this thread.
        poll_cancel_wall!();
        let mut nodes: Vec<Node> = Vec::new();
        let mut stack: Vec<(usize, Value)> = vec![(usize::MAX, val.clone())];
        while let Some((parent, v)) = stack.pop() {
            charge!(1);
            let id = nodes.len();
            match &v {
                // Children are pushed REVERSED so the LIFO worklist pops them
                // in DECLARATION order — the child_list built below therefore
                // preserves field/element order.
                Value::Record(r) => {
                    for f in r.fields.iter().rev() {
                        stack.push((id, f.clone()));
                    }
                }
                Value::List(h) => {
                    let buf = h.rc.borrow();
                    for e in buf.elems.iter().rev() {
                        stack.push((id, e.clone()));
                    }
                }
                Value::Variant(var) => {
                    if let Some(pl) = var.payload.as_ref() {
                        stack.push((id, pl.as_ref().clone()));
                    }
                }
                _ => {}
            }
            nodes.push(Node { src: v, parent });
        }
        // Phase 2: children have LARGER ids than their parent (the DFS
        // pushes children with the just-allocated id). Assemble in reverse
        // id order; a parent consumes its children by scanning the parent
        // map. `out` lives OUTSIDE the closure so the abort epilogue owns
        // every partial entry.
        out = vec![None; nodes.len()];
        let mut child_list: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
        for (k, n) in nodes.iter().enumerate() {
            if n.parent != usize::MAX {
                child_list[n.parent].push(k);
            }
        }
        for id in (0..nodes.len()).rev() {
            charge!(1);
            let v = nodes[id].src.clone();
            let assembled = match &v {
                Value::Str(h) => {
                    // F822-R11-01: output-byte-scaled fuel charged BEFORE the
                    // copy — the step-budget rejection precedes the allocation
                    // attempt (nothing minted for this node on rejection).
                    charge!(h.bytes().len());
                    match arena.try_alloc(h.bytes()) {
                        Ok(nh) => Value::Str(nh),
                        Err(n) => {
                            return Err(ExecOutcome::Infra(format!(
                                "fault-injected at owned allocation attempt #{n}"
                            )))
                        }
                    }
                }
                Value::Record(r) => {
                    let fields: Vec<Value> = child_list[id]
                        .iter()
                        .map(|k| out[*k].take().expect("child cloned first"))
                        .collect();
                    Value::Record(std::rc::Rc::new(crate::contract::RecordValue {
                        ty: r.ty,
                        fields,
                    }))
                }
                Value::List(_) => {
                    let elems: Vec<Value> = child_list[id]
                        .iter()
                        .map(|k| out[*k].take().expect("child cloned first"))
                        .collect();
                    let cap = elems.len();
                    // F822-R13-01: reserve the fault ATTEMPT before the
                    // elements change hands — on the injected fault the
                    // collected children stay HERE for the explicit drain
                    // (handing them to the arena by value would drop them
                    // through the recursive glue inside its frame).
                    match arena.list_alloc_attempt() {
                        Err(n) => {
                            drain_extras.extend(elems);
                            return Err(ExecOutcome::Infra(format!(
                                "fault-injected at owned allocation attempt #{n}"
                            )));
                        }
                        Ok(()) => Value::List(arena.alloc_list_after_attempt(elems, cap)),
                    }
                }
                Value::Variant(var) => {
                    let payload = child_list[id]
                        .first()
                        .map(|k| std::rc::Rc::new(out[*k].take().expect("child cloned first")));
                    Value::Variant(crate::contract::VariantValue {
                        ty: var.ty,
                        disc: var.disc,
                        payload,
                    })
                }
                other => other.clone(),
            };
            out[id] = Some(assembled);
        }
        Ok(())
    };
    match build() {
        Ok(()) => Ok(out[0].take().expect("root cloned")),
        Err(e) => {
            // The ONE explicit rollback: partial owners drain through the
            // worklist (leaf frees fire through the handle hooks; shared
            // handles are plain transfers — the source stays untouched).
            let mut partials: Vec<Value> = out.into_iter().flatten().collect();
            partials.extend(drain_extras);
            let mut rollback_fuel = 0u64; // charge-free (see above)
            release_owned_worklist(&mut rollback_fuel, partials);
            Err(e)
        }
    }
}

/// Project a view's scalar range to its byte slice over the source bytes.
fn project_bytes<'a>(bytes: &'a [u8], view: &crate::contract::StrView) -> &'a [u8] {
    match crate::owned_text::scalar_range_to_bytes(bytes, view.start as i32, view.end as i32) {
        Ok((s, e)) => &bytes[s..e],
        // Verified programs never carry an out-of-domain stored range; the
        // empty projection keeps this total for defense in depth.
        Err(_) => &[],
    }
}

/// Resolve one view VALUE to (source bytes, the view) for the str ops:
/// static views read the module's const bytes; frame-rooted views
/// re-validate the handle (frame alive + activations + generation) and
/// project the path to the owning Str (V-05 stale-handle rejection).
fn resolve_view_bytes<V: ModuleView>(
    _v: &V,
    _func_idx: u32,
    val: &Value,
) -> Result<(Vec<u8>, crate::contract::StrView), ExecOutcome> {
    let _unused_frames: Option<&LoopState> = None;
    let view = val.as_strview().ok_or_else(|| {
        ExecOutcome::Infra(format!(
            "str operand holds {} (admission violation)",
            val.type_name()
        ))
    })?;
    match &view.src {
        crate::contract::StrViewSrc::Static { buf } => {
            // F821-R1-03: the bytes were resolved in the OWNING unit at
            // mint time — a cross-unit view stays correct (per-unit const
            // index domains never mix).
            Ok((buf.to_vec(), view.clone()))
        }
        crate::contract::StrViewSrc::Share(_) => {
            let resolved = crate::place::resolve_share_view(view).ok_or_else(|| {
                ExecOutcome::Infra("share view resolution failed (admission violation)".to_string())
            })?;
            Ok((resolved.handle.bytes().to_vec(), view.clone()))
        }
    }
}

/// PLAN-821 all-exit cleanup: drain remaining frames top-down, releasing
/// each frame's owned values in DESCENDING creation order (the 782 LIFO
/// table's cross-type rule; intra-record field order refines with T-03).
/// Fuel is the reserved cleanup accounting — never cancelled by the
/// exhaustion that triggered the unwind (contract-decision §5).
/// Release one frame's owned values in descending creation order
/// (shared by the RETURN path and the unwind path).
fn release_frame_owned_lifo(st: &mut LoopState, frame: &mut Frame) {
    let mut owned: Vec<(u64, Value)> = Vec::new();
    for slot_val in frame.slots.iter_mut() {
        match slot_val {
            Value::Str(h) => {
                owned.push((h.rc.id, Value::Str(h.clone())));
                *slot_val = Value::I32(0);
            }
            Value::Record(_) => {
                let min_id = min_buffer_id(slot_val);
                owned.push((min_id, slot_val.clone()));
                *slot_val = Value::I32(0);
            }
            // F822-R2-03: lists join the LIFO sweep by their creation-
            // order identity; their elements release REVERSE inside
            // drop_owned_reverse (contract §4 / V-06).
            Value::List(_) => {
                let min_id = min_buffer_id(slot_val);
                owned.push((min_id, slot_val.clone()));
                *slot_val = Value::I32(0);
            }
            Value::Variant(_) => {
                let min_id = min_buffer_id(slot_val);
                owned.push((min_id, slot_val.clone()));
                *slot_val = Value::I32(0);
            }
            // PLAN-823: a closure slot's env payload lives in the STORE —
            // the snapshot drop cannot reach it, so the env releases HERE
            // (reverse capture order through the explicit worklist). The
            // escaped-through-return case never reaches this sweep: the
            // Return arm zeroes the returning slot first (responsibility
            // transferred OUT in `rv`).
            Value::Closure(cref) => {
                // A stale reference must not release another live owner.
                if closure_ref_valid(st, *cref) {
                    release_closure_env(st, cref.env);
                }
                *slot_val = Value::I32(0);
            }
            _ => {}
        }
    }
    owned.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, val) in owned {
        st.steps += 1; // reserved cleanup fuel
        drop_owned_reverse(st, val);
    }
}

/// PLAN-823: release one closure env from the store — take it out, subtract
/// its byte share, and feed the fields (REVERSE capture order) through the
/// shared explicit cleanup worklist (F822-R12 discipline: bounded work
/// items, no host drop-glue recursion on deep chains).
fn release_closure_env(st: &mut LoopState, handle: u64) {
    if let Some(entry) = st.closure_envs.remove(&handle) {
        let binding_bytes = closure_binding_bytes(entry.env.field_count(), entry.borrows.len());
        let env = entry.env;
        let mut visited = 0u64;
        let env_bytes = env.deep_size(&mut visited).saturating_add(binding_bytes);
        st.slot_bytes = st.slot_bytes.saturating_sub(env_bytes);
        // The worklist pops from the end, so capture-order roots release
        // in reverse capture order without reversing them twice.
        release_owned_worklist(&mut st.steps, env.into_fields());
    }
}

fn cleanup_owned_frames(st: &mut LoopState) {
    while let Some(mut frame) = st.frames.pop() {
        release_frame_owned_lifo(st, &mut frame);
        drop(frame);
    }
    // A rejected stale reference cannot release a different environment.
    // Once every frame has unwound, reclaim any orphaned entries using the
    // same bounded cleanup path, in reverse creation order.
    let mut remaining: Vec<u64> = st.closure_envs.keys().copied().collect();
    remaining.sort_unstable_by(|a, b| b.cmp(a));
    for handle in remaining {
        release_closure_env(st, handle);
    }
}

/// F822-R12-01: the minimum ledger buffer id reachable from a value —
/// the LIFO release ORDER KEY for frame sweeps. EXPLICIT worklist over
/// OWNED shallow snapshots (Value clones are Rc/handle bumps for every
/// supported kind, so the plan never rides the host stack): the frozen
/// domain admits arbitrarily deep acyclic Variant/Record nesting with NO
/// language depth cap, so order discovery must not recurse (§5.1.6's
/// explicit-traversal rule reaches every value walk over the supported
/// graph, not only the clone visitor). The dropped snapshots never reach
/// the last reference (the source value still holds each one), so no
/// free event can fire here.
fn min_buffer_id(val: &Value) -> u64 {
    let mut min = u64::MAX;
    let mut stack: Vec<Value> = vec![val.clone()];
    while let Some(x) = stack.pop() {
        match &x {
            Value::Str(h) => min = min.min(h.rc.id),
            Value::List(h) => {
                let buf = h.rc.borrow();
                min = min.min(buf.id);
                for e in buf.elems.iter() {
                    stack.push(e.clone());
                }
            }
            Value::Variant(var) => {
                if let Some(p) = var.payload.as_deref() {
                    stack.push(p.clone());
                }
            }
            Value::Record(r) => {
                for f in r.fields.iter() {
                    stack.push(f.clone());
                }
            }
            _ => {}
        }
    }
    min
}

/// F822-R2-03: REVERSE-order responsibility cleanup of an owned value
/// (contract §4 / V-06): list elements release last-pushed-first (the
/// reverse of insertion order — `pop()` from the tail), each element
/// cleaning its own payload (variant payloads included); the container's
/// own buffer frees only after its elements. Scalars and non-container
/// values drop directly. Cleanup fuel is reserved per element/payload
/// (the cancel-point rule).
/// F822-R12-01: the release is a FULLY EXPLICIT worklist — every
/// supported kind (arbitrarily deep acyclic Variant chains, nested
/// records, lists) releases through bounded work items. Containers are
/// UNWRAPPED rather than dropped whole: a last-reference payload/field
/// set is taken out of its Rc and queued, so the final drops are leaves
/// and the host drop glue never recurses through a nested chain. A
/// shared handle (Rc::strong_count > 1) is a responsibility transfer
/// (F822-R3-02 MOVE/COPY/kill-after-move-in semantics): nothing drains,
/// the handle clone just releases.
/// F822-R13-01: the worklist core is SHARED — the dispatcher cleanup
/// charges reserved fuel through `st.steps`, and the clone visitor's
/// partial-output rollback drains through the same walk with a
/// charge-free scratch counter (the historical glue-drop rollback
/// accounted no cleanup fuel; abort-path fuel totals stay exactly what
/// the work charged).
fn release_owned_worklist(fuel: &mut u64, roots: Vec<Value>) {
    let mut stack: Vec<Value> = roots;
    while let Some(v) = stack.pop() {
        match v {
            Value::List(h) => {
                if std::rc::Rc::strong_count(&h.rc) == 1 {
                    let elems = std::mem::take(&mut h.rc.borrow_mut().elems);
                    if elems.is_empty() {
                        // The drained (or born-empty) buffer: its free
                        // event fires HERE — after its elements, because
                        // the marker pushed below pops last.
                        drop(h);
                    } else {
                        // Buffer marker FIRST (pops LAST, event after its
                        // elements); elements pushed in declaration order
                        // pop in REVERSE — the contracted order.
                        stack.push(Value::List(h));
                        for e in elems.into_iter() {
                            *fuel += 1; // reserved cleanup fuel (cancel point)
                            stack.push(e);
                        }
                    }
                }
                // else: shared buffer — the handle drop is a transfer.
            }
            Value::Variant(mut var) => {
                if let Some(p) = var.payload.take() {
                    *fuel += 1; // reserved cleanup fuel (cancel point)
                    if std::rc::Rc::strong_count(&p) == 1 {
                        // Last reference: take the inner value OUT so the
                        // empty variant shell drops without glue recursion
                        // and the payload processes as its own work item.
                        if let Ok(inner) = std::rc::Rc::try_unwrap(p) {
                            stack.push(inner);
                        }
                    }
                    // else: shared payload — the clone release transfers.
                }
                drop(var); // payload-less shell: plain drop, no chain
            }
            Value::Record(r) => {
                if std::rc::Rc::strong_count(&r) == 1 {
                    if let Ok(rv) = std::rc::Rc::try_unwrap(r) {
                        let mut fields = rv.fields;
                        // Fields pushed REVERSED pop in DECLARATION order
                        // — the same free-event order the previous plain
                        // Vec drop glue produced.
                        while let Some(f) = fields.pop() {
                            stack.push(f);
                        }
                    }
                }
                // else: shared record — handle drop is a transfer.
            }
            // Str/scalar/leaf: the handle drop reports the free event at
            // the last reference; nothing nests below.
            other => drop(other),
        }
    }
}

fn drop_owned_reverse(st: &mut LoopState, val: Value) {
    release_owned_worklist(&mut st.steps, vec![val]);
}

/// Kill one slot of the CURRENT frame (move-out / move-in at a call):
/// replace with the typed placeholder, bump the generation and adjust the
/// live-data ledger by the deep delta (a kill never allocates — no fault
/// seam here, V-03's attempt accounting lives in the allocating op).
fn owned_kill_slot(st: &mut LoopState, slot: usize, zero: Value) {
    let mut visited = 0u64;
    {
        let fr = st.frames.last_mut().expect("frame present");
        let old = fr.slots[slot].deep_size(&mut visited);
        let new = zero.deep_size(&mut visited);
        st.steps += visited;
        st.slot_bytes = st.slot_bytes.saturating_sub(old.saturating_sub(new));
        // Take the dying value OUT before the placeholder lands so its
        // cleanup runs the contracted REVERSE element order (lists), not
        // whatever drop order the assignment would choose.
        let dying = std::mem::replace(&mut fr.slots[slot], zero);
        fr.gens[slot] = fr.gens[slot].wrapping_add(1);
        drop_owned_reverse(st, dying);
    }
}

// -- PLAN-822 collections helpers -------------------------------------------

/// Checked list-capacity growth (0 -> 4 -> doubling), bounded by the frozen
/// element guard.
fn grow_cap(cap: u32) -> u32 {
    if cap == 0 {
        4
    } else {
        cap.saturating_mul(2)
    }
}

/// Read the i32 index operand; a negative index is the same runtime trap
/// as an out-of-range one (checked BEFORE any buffer access).
fn list_index_of(iv: &Value) -> Option<usize> {
    match iv {
        Value::I32(i) if *i >= 0 => Some(*i as usize),
        _ => None,
    }
}

fn list_index_trap(
    v: &dyn ModuleView,
    func_idx: u32,
    func: &MirFunction,
    flat: u32,
) -> ExecOutcome {
    ExecOutcome::Trap {
        kind: crate::contract::TrapKind::ListIndex,
        span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
    }
}

/// The variant arm's payload TYPE INDEX in `func`'s type space (u32::MAX
/// when unknown — typing caught those earlier; `type_is_owned` is false
/// for it).
fn variant_payload_ty(v: &dyn ModuleView, func_idx: u32, variant_slot_ty: u32, disc: u32) -> u32 {
    match v.mir_type_of(func_idx, variant_slot_ty) {
        Some(crate::mir::MirType::Variant(vr)) => vr
            .arms
            .get(disc as usize)
            .and_then(|a| a.payload)
            .unwrap_or(u32::MAX),
        _ => u32::MAX,
    }
}

/// Resolve a place to its CURRENT element reference: find the root frame
/// by activation sequence, read the root slot's list buffer, and re-
/// validate buffer identity + structural epoch + element generation + the
/// index bound. Any mismatch is the contracted `av.trap.stale-place`
/// (F822-R2-02) — checked BEFORE any buffer access.
fn resolve_place_elem(
    st: &LoopState,
    place: &crate::contract::ListPlaceV,
) -> Result<Value, crate::contract::TrapKind> {
    let fr = st
        .frames
        .iter()
        .find(|f| f.seq as u32 == place.frame)
        .ok_or(crate::contract::TrapKind::InvalidHandle)?;
    let list = fr
        .slots
        .get(place.slot as usize)
        .ok_or(crate::contract::TrapKind::InvalidHandle)?;
    let h = match list {
        Value::List(h) => h,
        _ => return Err(crate::contract::TrapKind::StalePlace),
    };
    let buf = h.rc.borrow();
    if buf.id != place.buf_id || buf.epoch != place.epoch {
        // A structural change (realloc / element shift) happened after
        // the mint — the OLD place may not touch the new buffer.
        return Err(crate::contract::TrapKind::StalePlace);
    }
    let idx = place.index as usize;
    if idx >= buf.elems.len() || buf.elem_gens[idx] != place.elem_gen {
        // In-place replacement retired this element's places.
        return Err(crate::contract::TrapKind::StalePlace);
    }
    Ok(buf.elems[idx].clone())
}

/// Same validation, returning the current element's deep size for the
/// PLACE_SET precheck (the write path's old-value half).
fn validate_place_for_write(
    st: &LoopState,
    place: &crate::contract::ListPlaceV,
) -> Result<usize, crate::contract::TrapKind> {
    let fr = st
        .frames
        .iter()
        .find(|f| f.seq as u32 == place.frame)
        .ok_or(crate::contract::TrapKind::InvalidHandle)?;
    let list = fr
        .slots
        .get(place.slot as usize)
        .ok_or(crate::contract::TrapKind::InvalidHandle)?;
    let h = match list {
        Value::List(h) => h,
        _ => return Err(crate::contract::TrapKind::StalePlace),
    };
    let buf = h.rc.borrow();
    if buf.id != place.buf_id || buf.epoch != place.epoch {
        return Err(crate::contract::TrapKind::StalePlace);
    }
    let idx = place.index as usize;
    if idx >= buf.elems.len() || buf.elem_gens[idx] != place.elem_gen {
        return Err(crate::contract::TrapKind::StalePlace);
    }
    let mut visited = 0u64;
    Ok(buf.elems[idx].deep_size(&mut visited))
}

/// Kill a source slot after its value transferred into a container/variant:
/// OWNED sources drop to their typed zero (deep-delta accounted); Copy
/// sources keep their value (copy-in).
fn kill_value_slot(
    v: &dyn ModuleView,
    func_idx: u32,
    func: &MirFunction,
    st: &mut LoopState,
    slot: usize,
) {
    let ty = func.slots[slot].ty;
    if v.type_is_owned(func_idx, ty) {
        let zero = v.zero_of(func_idx, ty);
        owned_kill_slot(st, slot, zero);
    }
}

/// Field-path read: walks a record value by declaration indices, cloning
/// the addressed field out (rvalue copy). `None` = shape mismatch (an
/// admission violation — verification guarantees this never fires for an
/// admitted module).
fn field_get_val(base: &Value, path: &[u16]) -> Option<Value> {
    let mut cur = base;
    for (i, seg) in path.iter().enumerate() {
        let r = match cur {
            Value::Record(r) => r,
            _ => return None,
        };
        let f = r.fields.get(*seg as usize)?;
        cur = f;
        if i + 1 == path.len() {
            return Some(f.clone());
        }
    }
    None
}

/// Field-path write (structural rebuild, PLAN-820 §6): pure values are
/// immutable, so the addressed field is replaced by rebuilding every record
/// level along the path into fresh values. Recursion depth is bounded by
/// the verified path-length cap (64). `None` = shape mismatch (admission
/// violation for an admitted program).
fn record_set_val(root: &Value, path: &[u16], val: Value) -> Option<Value> {
    let r = match root {
        Value::Record(r) => r,
        _ => return None,
    };
    let seg = *path.first()? as usize;
    let mut fields = r.fields.clone();
    if path.len() == 1 {
        if seg >= fields.len() {
            return None;
        }
        fields[seg] = val;
    } else {
        let inner = fields.get(seg)?.clone();
        fields[seg] = record_set_val(&inner, &path[1..], val)?;
    }
    Some(Value::Record(std::rc::Rc::new(
        crate::contract::RecordValue { ty: r.ty, fields },
    )))
}

fn dispatch<V: ModuleView>(
    v: &V,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    args: &[Value],
    st: &mut LoopState,
    budget: Option<u64>,
    stop_before_call: bool,
) -> Option<ExecOutcome> {
    dispatch_with_host(v, limits, cancel, args, st, budget, stop_before_call, None)
}

fn dispatch_with_host<V: ModuleView>(
    v: &V,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    args: &[Value],
    st: &mut LoopState,
    budget: Option<u64>,
    stop_before_call: bool,
    mut host: Option<&mut dyn crate::host::ConsoleHost>,
) -> Option<ExecOutcome> {
    if crate::host::inside_provider() {
        return Some(ExecOutcome::Infra(
            "host.reentry: guest execution from a host call is forbidden".into(),
        ));
    }
    if !v.host_imports().is_empty() && host.is_none() {
        return Some(ExecOutcome::Infra(
            "host.unauthorized: no console provider".into(),
        ));
    }
    let started = Instant::now();
    // B2-C04: the step budget is PER SEGMENT (this dispatch call's own
    // budget — `resume(.., Some(n))` grants n MORE steps); the cumulative
    // hard fuel (`limits.max_steps`) below still spans the whole chain.
    let seg_start = st.steps;
    let entry_slot = v.entry();
    if st.frames.is_empty() {
        let entry_fn = match v.fn_at(entry_slot) {
            Some(f) => f,
            None => {
                return Some(ExecOutcome::Infra(format!(
                    "entry function #{} missing",
                    entry_slot
                )))
            }
        };
        let reference_params;
        let params = if let Some(params) = v.param_slots(entry_slot) {
            st.trace.loaded_param_map_queries += 1;
            params
        } else {
            reference_params = entry_fn.param_slots();
            st.trace.reference_param_map_copies += 1;
            &reference_params
        };
        if params.len() != args.len() {
            return Some(ExecOutcome::Infra(format!(
                "entry expects {} args, driver passed {}",
                params.len(),
                args.len()
            )));
        }
        // Entry-argument admission against the VERIFIED signature
        // (F798-R6-02): each Value must match its param slot's scalar shape
        // — a Bool handed to an i32 slot (or any cross-kind pair, an
        // Fn/Record-typed slot, or a record value at the entry boundary) is
        // an admission violation, never a silently executing coercion.
        for (i, arg) in args.iter().enumerate() {
            let slot_ty = entry_fn.slots[params[i] as usize].ty;
            use crate::mir::ScalarShape;
            let ok = match v.scalar_shape_of(entry_slot, slot_ty) {
                Some(ScalarShape::I32) => matches!(arg, Value::I32(_)),
                Some(ScalarShape::Bool) => matches!(arg, Value::Bool(_)),
                Some(ScalarShape::I64) => matches!(arg, Value::I64(_)),
                Some(ScalarShape::U64) => matches!(arg, Value::U64(_)),
                Some(ScalarShape::F32) => matches!(arg, Value::F32(_)),
                Some(ScalarShape::F64) => matches!(arg, Value::F64(_)),
                // PLAN-821: char entry params admit char values (the
                // owned entry contract, contract-decision §6).
                None => matches!(
                    (v.mir_type_of(entry_slot, slot_ty), arg),
                    (Some(crate::mir::MirType::Char), Value::Char(_))
                ),
            };
            if !ok {
                return Some(ExecOutcome::Infra(format!(
                    "entry argument #{} has kind {}, param slot type {} — admission violation",
                    i,
                    arg.type_name(),
                    slot_ty
                )));
            }
        }
        // Root-frame budgets BEFORE allocation (F798-R3-04): a zero frame or
        // slot budget must reject a program that genuinely uses slots,
        // instead of executing it and reporting peak 0.
        if limits.max_frames < 1 {
            return Some(ExecOutcome::Resource {
                kind: ResourceKind::Frames,
                message: format!(
                    "guest frames budget {} cannot hold the root frame",
                    limits.max_frames
                ),
            });
        }
        // Root frame. PLAN-827 R6: a shallow pre-gate so a budget refusal
        // PRECEDES the reserve attempt (fault-probe observable), then the
        // slot storage goes through the fallible fault-injectable reserve
        // (attempt accounting + ownership lease); PLAN-820 §6 keeps the
        // authoritative deep filled-frame budget below.
        if entry_fn.slots.len() * crate::contract::SLOT_CELL_BYTES > limits.max_slot_bytes {
            return Some(ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                message: format!(
                    "root frame slot data would reach {} bytes, above budget {}",
                    entry_fn.slots.len() * crate::contract::SLOT_CELL_BYTES,
                    limits.max_slot_bytes
                ),
            });
        }
        let (slots, allocation) = match crate::frame_allocation::reserve_slots(entry_fn.slots.len())
        {
            Ok(v) => v,
            Err(()) => {
                return Some(ExecOutcome::Resource {
                    kind: ResourceKind::SlotBytes,
                    message: "root frame slot allocation failed".into(),
                })
            }
        };
        st.frame_seq += 1;
        let mut frame = Frame {
            func: entry_slot,
            block: entry_fn.entry,
            ip: 0,
            slots,
            gens: vec![0; entry_fn.slots.len()],
            seq: st.frame_seq,
            _allocation: allocation,
            ret_dst: None,
            closure_env: None,
        };
        let mut arg_iter = args.iter().cloned();
        for s in &entry_fn.slots {
            frame.slots.push(if s.kind == crate::mir::SlotKind::Param {
                arg_iter.next().expect("count checked above")
            } else {
                v.zero_of(entry_slot, s.ty)
            });
        }
        // Deep live-data budget on the FILLED frame before it commits
        // (PLAN-820 §6: slots + aggregates share the one 8 MiB line).
        let mut visited = 0u64;
        let root_slot_bytes = frame_slots_bytes(&frame.slots, &mut visited);
        st.steps += visited;
        if st.slot_bytes.saturating_add(root_slot_bytes) > limits.max_slot_bytes {
            return Some(ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                message: format!(
                    "root frame live slot data {} bytes exceeds budget {}",
                    root_slot_bytes, limits.max_slot_bytes
                ),
            });
        }
        st.slot_bytes += root_slot_bytes;
        // The peak lives ON the trace: every exit (success, trap, resource,
        // cancel) carries the real high-water (F798-R3-04).
        st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
        st.frames.push(frame);
        st.trace.root_frame_commits += 1;
    }

    loop {
        if let Some(budget) = budget {
            if st.steps - seg_start >= budget {
                return None;
            }
        }
        st.steps += 1;
        if st.steps > limits.max_steps {
            return Some(ExecOutcome::Resource {
                kind: ResourceKind::InstrSteps,
                message: format!("execution exceeded {} instruction steps", limits.max_steps),
            });
        }
        // Poll at ≥INTERVAL steps since the LAST poll (data-profile writes
        // charge fuel in batches — a plain modulo can skip the boundary;
        // unit-increment programs poll at exactly the same steps as the
        // historic modulo cadence).
        if st.steps - st.last_poll >= CANCEL_CHECK_INTERVAL {
            st.last_poll = st.steps;
            if CancelCheck::poll(cancel) {
                return Some(ExecOutcome::Cancelled);
            }
            if started.elapsed() > limits.wall {
                return Some(ExecOutcome::Resource {
                    kind: ResourceKind::ExecutionWall,
                    message: "execution exceeded wall budget".to_string(),
                });
            }
        }
        // Copy the frame cursor, then re-borrow only for writes: the call
        // arm needs to push/pop `st.frames` while the current frame is read.
        let (func_idx, block_idx, ip) = {
            let fr = match st.frames.last() {
                Some(f) => f,
                None => return Some(ExecOutcome::Infra("frame stack empty".to_string())),
            };
            (fr.func, fr.block, fr.ip)
        };
        let func = match v.fn_at(func_idx) {
            Some(f) => f,
            None => {
                return Some(ExecOutcome::Infra(format!(
                    "frame func #{} missing",
                    func_idx
                )))
            }
        };
        let block = match func.blocks.get(block_idx as usize) {
            Some(b) => b,
            None => {
                return Some(ExecOutcome::Infra(format!(
                    "frame block #{} missing",
                    block_idx
                )))
            }
        };
        if ip < block.instrs.len() {
            let instr = &block.instrs[ip];
            // PLAN-866 safety point: park the chain right BEFORE the next
            // guest call dispatch (direct or closure) — the frozen "one
            // completed call" boundary. Step-neutral by construction:
            // this iteration's step is rolled back and the re-entry
            // (plain or swapped resolution) re-fetches the same call.
            if stop_before_call
                && (matches!(instr, MirInstr::Call { .. })
                    || matches!(instr, MirInstr::CallClosure { .. }))
            {
                st.steps -= 1;
                return None;
            }
            {
                let fr = st.frames.last_mut().expect("frame present");
                fr.ip += 1;
            }
            let (loaded, flat) = match v.logical_pc(func_idx, block_idx, ip) {
                Some(parts) => parts,
                None => {
                    return Some(ExecOutcome::Infra(format!(
                        "logical PC missing for func #{func_idx} block #{block_idx} ip {ip}"
                    )))
                }
            };
            if loaded {
                st.trace.loaded_pc_lookups += 1;
            } else {
                st.trace.reference_pc_lookups += 1;
            }
            match instr {
                MirInstr::Const { dst, value } => {
                    let cv = match v.const_at(func_idx, *value) {
                        Some(cv) => cv,
                        None => {
                            return Some(ExecOutcome::Infra(format!(
                                "const #{} of func #{} missing",
                                value, func_idx
                            )))
                        }
                    };
                    let dst_ty = func.slots[*dst as usize].ty;
                    let val = match cv {
                        crate::mir::MirConst::I32(x) => Value::I32(x),
                        crate::mir::MirConst::Bool(b) => Value::Bool(b),
                        crate::mir::MirConst::I64(x) => Value::I64(x),
                        crate::mir::MirConst::U64(x) => Value::U64(x),
                        // Bit-exact re-materialization: NaN payloads and
                        // signed zeros survive the const round trip.
                        crate::mir::MirConst::F32Bits(bits) => Value::F32(f32::from_bits(bits)),
                        crate::mir::MirConst::F64Bits(bits) => Value::F64(f64::from_bits(bits)),
                        // PLAN-821: a char const is its scalar; a Str const
                        // materializes by DESTINATION kind — an owned Str
                        // slot gets a FRESH arena allocation, a StrView
                        // slot the immortal static view (770 literal rule).
                        crate::mir::MirConst::Char(c) => Value::Char(c),
                        crate::mir::MirConst::Str(bytes) => {
                            if v.is_strview_type(func_idx, dst_ty) {
                                let n = crate::owned_text::scalars(&bytes).len() as u32;
                                // F821-R1-03: bind the OWNING unit's
                                // resolved literal bytes — the view may
                                // cross unit boundaries (its const index
                                // must not).
                                Value::StrView(crate::contract::StrView {
                                    src: crate::contract::StrViewSrc::Static {
                                        buf: std::rc::Rc::new(bytes.clone()),
                                    },
                                    start: 0,
                                    end: n,
                                })
                            } else {
                                // The prospective value's DEEP size (the
                                // slot overhead rides every write).
                                let deep = std::mem::size_of::<Value>() + bytes.len();
                                if let Err(out) =
                                    owned_alloc_precheck(st, deep, *dst as usize, limits)
                                {
                                    return Some(out);
                                }
                                // F821-R1-06: materialization copies the
                                // literal — byte-scaled fuel.
                                st.steps += bytes.len() as u64;
                                match st.arena.try_alloc(&bytes) {
                                    Ok(h) => Value::Str(h),
                                    Err(n) => {
                                        return Some(ExecOutcome::Infra(format!(
                                            "fault-injected at owned allocation attempt #{n}"
                                        )))
                                    }
                                }
                            }
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, val, limits) {
                        return Some(out);
                    }
                }
                MirInstr::Copy { dst, src } => {
                    let val = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*src as usize].clone()
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, val, limits) {
                        return Some(out);
                    }
                }
                MirInstr::Bin { op, dst, lhs, rhs } => {
                    let (l, r) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*lhs as usize].clone(),
                            fr.slots[*rhs as usize].clone(),
                        )
                    };
                    match eval_bin(*op, l, r) {
                        Ok(res) => {
                            if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                                return Some(out);
                            }
                        }
                        Err(EvalErr::Trap(kind)) => {
                            // B2-C03: the span maps through the view of
                            // the EXECUTING frame's code slot — payload
                            // func numbers are per-producer coordinates
                            // and may collide across real same-owner
                            // units; the frame's slot never does.
                            return Some(ExecOutcome::Trap {
                                kind,
                                span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                            });
                        }
                        Err(EvalErr::Infra(msg)) => return Some(ExecOutcome::Infra(msg)),
                    }
                }
                MirInstr::Un { op, dst, operand } => {
                    // (do not shadow `v` — the ModuleView — with the
                    // operand value; the trap arm needs the view for the
                    // per-frame span projection.)
                    let operand_val = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*operand as usize].clone()
                    };
                    match eval_un(*op, operand_val) {
                        Ok(res) => {
                            if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                                return Some(out);
                            }
                        }
                        Err(EvalErr::Trap(kind)) => {
                            // B2-C03: the span maps through the view of
                            // the EXECUTING frame's code slot — payload
                            // func numbers are per-producer coordinates
                            // and may collide across real same-owner
                            // units; the frame's slot never does.
                            return Some(ExecOutcome::Trap {
                                kind,
                                span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                            });
                        }
                        Err(EvalErr::Infra(msg)) => return Some(ExecOutcome::Infra(msg)),
                    }
                }
                MirInstr::Call { callee, dst, args } => {
                    // v1 executable closures lower test-mark imports to
                    // TRACE instructions at closure assembly; an
                    // unresolved callee (import or foreign index space) is
                    // an admission violation in every view.
                    let target = match v.resolve_callee(func_idx, callee) {
                        Some(t) => t,
                        None => {
                            return Some(ExecOutcome::Infra(format!(
                                "callee resolution failed in func #{}: {:?}",
                                func_idx, callee
                            )))
                        }
                    };
                    if st.frames.len() >= limits.max_frames {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::Frames,
                            message: format!("guest frames exceeded {}", limits.max_frames),
                        });
                    }
                    let callee_fn = v
                        .fn_at(target)
                        .expect("resolve_callee only returns slots with a payload");
                    // Slot budget BEFORE the callee frame is allocated
                    // (F798-R3-04): the check used to run after the push.
                    // PLAN-827 R6 keeps a shallow pre-gate so a budget
                    // refusal still PRECEDES the allocation attempt; the
                    // authoritative PLAN-820 §6 check is the deep size of
                    // the FILLED callee frame (params + typed placeholders)
                    // after the reserve below.
                    let prospective =
                        st.slot_bytes + callee_fn.slots.len() * crate::contract::SLOT_CELL_BYTES;
                    if prospective > limits.max_slot_bytes {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: format!(
                                "live slot data would reach {} bytes, above budget {}",
                                prospective, limits.max_slot_bytes
                            ),
                        });
                    }
                    let (slots, allocation) =
                        match crate::frame_allocation::reserve_slots(callee_fn.slots.len()) {
                            Ok(v) => v,
                            Err(()) => {
                                return Some(ExecOutcome::Resource {
                                    kind: ResourceKind::SlotBytes,
                                    message: "callee frame slot allocation failed".into(),
                                })
                            }
                        };
                    st.frame_seq += 1;
                    let seq = st.frame_seq;
                    let mut nf = Frame {
                        func: target,
                        block: callee_fn.entry,
                        ip: 0,
                        slots,
                        gens: vec![0; callee_fn.slots.len()],
                        seq,
                        _allocation: allocation,
                        ret_dst: Some(*dst),
                        closure_env: None,
                    };
                    let caller = st.frames.last().expect("frame present");
                    let mut ai = args.iter().map(|a| caller.slots[*a as usize].clone());
                    for s in &callee_fn.slots {
                        nf.slots.push(if s.kind == crate::mir::SlotKind::Param {
                            ai.next().expect("arity verified at admission")
                        } else {
                            v.zero_of(target, s.ty)
                        });
                    }
                    // PLAN-821 move-in: owned-typed arguments transfer to
                    // the callee — the CALLER slot dies at the call (killed
                    // in the ownership dataflow; here the reference and the
                    // generation both move on). R1-F08: the kill is
                    // SIG-DRIVEN — a borrowed (view-kind) callee param
                    // keeps the caller's ownership (782 ABI-D `*const`).
                    {
                        let callee_sig_ty = v.mir_type_of(target, callee_fn.sig);
                        for (ai, a) in args.iter().enumerate() {
                            let ty = func.slots[*a as usize].ty;
                            let param_borrows = callee_sig_ty
                                .and_then(|t| match t {
                                    crate::mir::MirType::Fn(sig) => sig.params.get(ai).copied(),
                                    _ => None,
                                })
                                .map(|pt| !v.type_is_owned(target, pt))
                                .unwrap_or(false);
                            if v.type_is_owned(func_idx, ty) && !param_borrows {
                                let zero = v.zero_of(func_idx, ty);
                                owned_kill_slot(st, *a as usize, zero);
                            }
                        }
                    }
                    let mut visited = 0u64;
                    let callee_bytes = frame_slots_bytes(&nf.slots, &mut visited);
                    st.steps += visited;
                    let prospective = st.slot_bytes + callee_bytes;
                    if prospective > limits.max_slot_bytes {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: format!(
                                "live slot data would reach {} bytes, above budget {}",
                                prospective, limits.max_slot_bytes
                            ),
                        });
                    }
                    st.slot_bytes = prospective;
                    st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                    st.frames.push(nf);
                    st.trace.call_frame_commits += 1;
                }
                // -- PLAN-823 closure surface (av.closures/1) ----------------
                MirInstr::CreateClosure {
                    dst,
                    callee,
                    captures,
                } => {
                    let target = match v.resolve_callee(func_idx, callee) {
                        Some(t) => t,
                        None => {
                            return Some(ExecOutcome::Infra(format!(
                                "closure callee resolution failed in func #{}: {:?}",
                                func_idx, callee
                            )))
                        }
                    };
                    // Reserve a process-unique handle before capture commit.
                    // Live environments never cross process boundaries.
                    let Some(handle) = reserve_closure_handle() else {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::TableEntries,
                            message: "closure handle space exhausted".into(),
                        });
                    };
                    let borrow_count = captures
                        .iter()
                        .filter(|c| c.kind == crate::mir::CaptureKind::Borrow)
                        .count();
                    // Phase 1 (prepare) over BORROWED slots: every fallible
                    // clone allocation happens BEFORE any source transfer
                    // (§5.5 prepare/commit law). The clone walk rides the
                    // shared visitor's cancel/wall/fuel discipline. BORROW
                    // captures stage caller-minted share views (no
                    // allocation, no drop right).
                    let wall_left = limits.wall.saturating_sub(started.elapsed());
                    let (plan, _prep) = {
                        let fr = st.frames.last().expect("frame present");
                        let env_handle = fr.closure_env;
                        let mut specs: Vec<crate::closures::CaptureSpec> =
                            Vec::with_capacity(captures.len());
                        for c in captures.iter() {
                            let mode = match c.kind {
                                crate::mir::CaptureKind::Move => crate::closures::CaptureMode::Move,
                                crate::mir::CaptureKind::Clone => {
                                    crate::closures::CaptureMode::Clone
                                }
                                crate::mir::CaptureKind::Borrow => {
                                    crate::closures::CaptureMode::Borrow
                                }
                                // R1-F01: the DEDICATED env-field transfer —
                                // prepare borrows the PARENT env's live
                                // field value (the take happens at commit).
                                crate::mir::CaptureKind::MoveFromEnv => {
                                    crate::closures::CaptureMode::Move
                                }
                            };
                            let src: &Value = if c.kind == crate::mir::CaptureKind::MoveFromEnv {
                                let h = env_handle.expect(
                                    "MoveFromEnv in a frame without an env binding (admission violation)",
                                );
                                match st
                                    .closure_envs
                                    .get(&h)
                                    .and_then(|e| e.env.field(c.src as usize))
                                {
                                    Some(v) => v,
                                    None => {
                                        return Some(ExecOutcome::Trap {
                                            kind: TrapKind::InvalidHandle,
                                            span: span_of(func, flat)
                                                .map(|sp| v.map_span(func_idx, sp)),
                                        })
                                    }
                                }
                            } else {
                                &fr.slots[c.src as usize]
                            };
                            specs.push(crate::closures::CaptureSpec { mode, src });
                        }
                        // Reserve the combined live-data budget before any
                        // clone allocation or ownership transfer. Move slots
                        // are replaced with their typed zero; parent env
                        // fields become scalar placeholders on transfer.
                        let mut visited = 0u64;
                        let mut after_moves = st.slot_bytes;
                        let mut env_bytes = closure_binding_bytes(captures.len(), borrow_count);
                        let mut dst_after_moves = fr.slots[*dst as usize].deep_size(&mut visited);
                        for (capture, spec) in captures.iter().zip(&specs) {
                            let bytes = if capture.kind == crate::mir::CaptureKind::Borrow {
                                std::mem::size_of::<Value>()
                            } else {
                                spec.src.deep_size(&mut visited)
                            };
                            env_bytes = env_bytes.saturating_add(bytes);
                            if capture.kind == crate::mir::CaptureKind::Move {
                                let old = fr.slots[capture.src as usize].deep_size(&mut visited);
                                let zero = v
                                    .zero_of(func_idx, func.slots[capture.src as usize].ty)
                                    .deep_size(&mut visited);
                                after_moves = after_moves.saturating_sub(old.saturating_sub(zero));
                                if capture.src == *dst {
                                    dst_after_moves = zero;
                                }
                            } else if capture.kind == crate::mir::CaptureKind::MoveFromEnv {
                                // The parent field becomes an i32 placeholder
                                // when its ownership moves to the child.
                                after_moves = after_moves.saturating_sub(
                                    bytes.saturating_sub(std::mem::size_of::<Value>()),
                                );
                            }
                        }
                        st.steps = st.steps.saturating_add(visited);
                        let after_write = after_moves.saturating_add(
                            std::mem::size_of::<Value>().saturating_sub(dst_after_moves),
                        );
                        let peak = after_moves.max(after_write).saturating_add(env_bytes);
                        if peak > limits.max_slot_bytes {
                            return Some(ExecOutcome::Resource {
                                kind: ResourceKind::SlotBytes,
                                message: format!(
                                    "closure env reservation would reach {} bytes, above budget {}",
                                    peak, limits.max_slot_bytes
                                ),
                            });
                        }
                        // Mint the borrow views over the creator's slots
                        // (the SHARE keeps the bytes readable — snapshot
                        // semantics; the static pin analysis guarantees the
                        // source outlives the synchronous use).
                        let staged_borrows: Vec<Value> = captures
                            .iter()
                            .filter(|c| c.kind == crate::mir::CaptureKind::Borrow)
                            .map(|c| {
                                let fr = st.frames.last().expect("frame present");
                                match &fr.slots[c.src as usize] {
                                    Value::Str(h) => {
                                        let n = crate::owned_text::scalars(h.bytes()).len() as u32;
                                        Value::StrView(crate::contract::StrView {
                                            src: crate::contract::StrViewSrc::Share(h.clone()),
                                            start: 0,
                                            end: n,
                                        })
                                    }
                                    // Unreachable for admitted programs (the
                                    // typing pass restricts Borrow sources to
                                    // Str slots; prepare re-rejects the kind)
                                    // — the empty static view keeps this
                                    // total instead of panicking mid-dispatch.
                                    _ => Value::StrView(crate::contract::StrView {
                                        src: crate::contract::StrViewSrc::Static {
                                            buf: std::rc::Rc::new(Box::from(&b""[..])),
                                        },
                                        start: 0,
                                        end: 0,
                                    }),
                                }
                            })
                            .collect();
                        let mut ctx = crate::closures::CloneContext {
                            cancel,
                            wall_left,
                            fuel: &mut st.steps,
                            max_fuel: limits.max_steps,
                        };
                        match crate::closures::EnvPlan::prepare(
                            &st.arena,
                            &specs,
                            &crate::closures::EnvLimits::default(),
                            &mut ctx,
                            staged_borrows,
                        ) {
                            Ok(x) => x,
                            Err(e) => {
                                return Some(match e {
                                    crate::closures::EnvPrepareError::FaultedClone {
                                        message,
                                        ..
                                    } => ExecOutcome::Infra(message),
                                    crate::closures::EnvPrepareError::Allocation { what } => {
                                        ExecOutcome::Resource {
                                            kind: ResourceKind::SlotBytes,
                                            message: format!("could not reserve {what}"),
                                        }
                                    }
                                    crate::closures::EnvPrepareError::Cancelled => {
                                        ExecOutcome::Cancelled
                                    }
                                    crate::closures::EnvPrepareError::CloneBudget {
                                        code,
                                        message,
                                    } => {
                                        if code == ResourceKind::InstrSteps.code() {
                                            ExecOutcome::Resource {
                                                kind: ResourceKind::InstrSteps,
                                                message,
                                            }
                                        } else if code == ResourceKind::ExecutionWall.code() {
                                            ExecOutcome::Resource {
                                                kind: ResourceKind::ExecutionWall,
                                                message,
                                            }
                                        } else {
                                            ExecOutcome::Infra(format!(
                                                "capture clone budget {}: {}",
                                                code, message
                                            ))
                                        }
                                    }
                                    crate::closures::EnvPrepareError::EnvBytesBudget {
                                        would,
                                        max,
                                    } => ExecOutcome::Resource {
                                        kind: ResourceKind::SlotBytes,
                                        message: format!(
                                            "closure env data {} bytes would exceed budget {}",
                                            would, max
                                        ),
                                    },
                                    crate::closures::EnvPrepareError::TooManyFields {
                                        got,
                                        max,
                                    } => ExecOutcome::Resource {
                                        kind: ResourceKind::TableEntries,
                                        message: format!(
                                            "closure env has {} captures, above the frozen {}",
                                            got, max
                                        ),
                                    },
                                    crate::closures::EnvPrepareError::UnsupportedKind { kind } => {
                                        ExecOutcome::Infra(format!(
                                            "capture of a {} (admission violation)",
                                            kind
                                        ))
                                    }
                                    crate::closures::EnvPrepareError::BorrowStaging(msg) => {
                                        ExecOutcome::Infra(msg)
                                    }
                                })
                            }
                        }
                    };
                    // Reserve the store entry and move-source list before
                    // touching ownership. A failed reservation unwinds the
                    // staged plan and leaves all capture sources intact.
                    if st.closure_envs.try_reserve(1).is_err() {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: "could not reserve closure store entry".into(),
                        });
                    }
                    let borrows: Vec<ClosureBorrow> = {
                        let fr = st.frames.last().expect("frame present");
                        captures
                            .iter()
                            .filter(|c| c.kind == crate::mir::CaptureKind::Borrow)
                            .map(|c| ClosureBorrow {
                                frame: fr.seq,
                                slot: c.src,
                                generation: fr.gens[c.src as usize],
                            })
                            .collect()
                    };
                    // Phase 2 (commit): take the MOVE sources out of their
                    // slots — whole-root transfer, typed zero lands, the
                    // generation bump retires stale rooted views (mirror of
                    // MOVE's transfer bookkeeping; clone sources keep their
                    // slots untouched). The env's materialization is one
                    // VALUE allocation on the fault seam (V-07: a fault
                    // here reports the attempt and consumes NO source —
                    // the kill loop below never runs).
                    if let Some(n) = limits.fault_after_allocs {
                        st.alloc_events += 1;
                        if st.alloc_events == n {
                            return Some(ExecOutcome::Infra(format!(
                                "fault-injected at value allocation #{} (closure env)",
                                n
                            )));
                        }
                    }
                    let parent_env_handle = st.frames.last().expect("frame present").closure_env;
                    let mut move_sources: Vec<Value> = Vec::new();
                    if move_sources.try_reserve_exact(captures.len()).is_err() {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: "could not reserve capture sources".into(),
                        });
                    }
                    // R1-F01: move sources are collected in CAPTURE order —
                    // commit binds the k-th source to the k-th Move spec.
                    // MoveFromEnv takes the PARENT env's field out
                    // (moved-out state + placeholder): the child env alone
                    // owns the value from commit on; plain Move takes the
                    // whole slot (typed zero + generation bump).
                    for c in captures.iter() {
                        match c.kind {
                            crate::mir::CaptureKind::MoveFromEnv => {
                                let h =
                                    parent_env_handle.expect("MoveFromEnv requires an env binding");
                                let taken = st
                                    .closure_envs
                                    .get_mut(&h)
                                    .and_then(|e| e.env.take_field(c.src as usize));
                                match taken {
                                    Some(v) => {
                                        let mut visited = 0u64;
                                        let old = v.deep_size(&mut visited);
                                        st.steps = st.steps.saturating_add(visited);
                                        st.slot_bytes = st.slot_bytes.saturating_sub(
                                            old.saturating_sub(std::mem::size_of::<Value>()),
                                        );
                                        move_sources.push(v);
                                    }
                                    None => {
                                        return Some(ExecOutcome::Trap {
                                            kind: TrapKind::InvalidHandle,
                                            span: span_of(func, flat)
                                                .map(|sp| v.map_span(func_idx, sp)),
                                        })
                                    }
                                }
                            }
                            crate::mir::CaptureKind::Move => {
                                let taken = {
                                    let mut visited = 0u64;
                                    let fr = st.frames.last_mut().expect("frame present");
                                    let old = fr.slots[c.src as usize].deep_size(&mut visited);
                                    let zero = v.zero_of(func_idx, func.slots[c.src as usize].ty);
                                    let new = zero.deep_size(&mut visited);
                                    st.steps += visited;
                                    st.slot_bytes =
                                        st.slot_bytes.saturating_sub(old.saturating_sub(new));
                                    fr.gens[c.src as usize] =
                                        fr.gens[c.src as usize].wrapping_add(1);
                                    std::mem::replace(&mut fr.slots[c.src as usize], zero)
                                };
                                move_sources.push(taken);
                            }
                            _ => {}
                        }
                    }
                    let (env, commit_acc) = match plan.commit(move_sources) {
                        Ok(x) => x,
                        // Unreachable for executor-handed sources (the very
                        // values prepare admitted); stays Infra, never a
                        // silent pass.
                        Err((e, _sources)) => {
                            return Some(ExecOutcome::Infra(format!(
                                "closure commit rejected: {}",
                                e.code()
                            )))
                        }
                    };
                    st.steps += commit_acc.fields;
                    // The env joins the ONE value-data line (§5.2:
                    // 槽+聚合+buffer+环境).
                    let mut visited = 0u64;
                    let env_bytes = env
                        .deep_size(&mut visited)
                        .saturating_add(closure_binding_bytes(env.field_count(), borrows.len()));
                    st.steps += visited;
                    if st.slot_bytes.saturating_add(env_bytes) > limits.max_slot_bytes {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: format!(
                                "live data incl. closure env would reach {} bytes, above budget {}",
                                st.slot_bytes + env_bytes,
                                limits.max_slot_bytes
                            ),
                        });
                    }
                    let sig_ty = func.slots[*dst as usize].ty;
                    st.closure_envs.insert(
                        handle,
                        ClosureEntry {
                            env,
                            func: target,
                            sig: sig_ty,
                            borrows,
                        },
                    );
                    let cref = Value::Closure(crate::contract::ClosureRefV {
                        func: target,
                        sig: sig_ty,
                        env: handle,
                        gen: handle,
                    });
                    if let Err(out) = commit_slot_write(st, *dst as usize, cref, limits) {
                        return Some(out);
                    }
                    st.slot_bytes += env_bytes;
                    st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                }
                MirInstr::CallClosure { dst, closure, args } => {
                    let cv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*closure as usize].clone()
                    };
                    let cref = match cv {
                        Value::Closure(c) => c,
                        other => {
                            return Some(ExecOutcome::Infra(format!(
                                "call-closure operand holds {} (admission violation)",
                                other.type_name()
                            )))
                        }
                    };
                    // V-04 runtime defense in depth: the store re-check (a
                    // stale handle traps before any callee side effect).
                    if !closure_ref_valid(st, cref) || !closure_borrows_valid(st, cref) {
                        return Some(ExecOutcome::Trap {
                            kind: TrapKind::InvalidHandle,
                            span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                        });
                    }
                    if st.frames.len() >= limits.max_frames {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::Frames,
                            message: format!("guest frames exceeded {}", limits.max_frames),
                        });
                    }
                    let callee_fn = v.fn_at(cref.func).expect("verify bound the closure callee");
                    // Shallow pre-gate, then the authoritative deep check on
                    // the FILLED frame (mirror of CALL's discipline).
                    let prospective =
                        st.slot_bytes + callee_fn.slots.len() * crate::contract::SLOT_CELL_BYTES;
                    if prospective > limits.max_slot_bytes {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: format!(
                                "live slot data would reach {} bytes, above budget {}",
                                prospective, limits.max_slot_bytes
                            ),
                        });
                    }
                    let (slots, allocation) =
                        match crate::frame_allocation::reserve_slots(callee_fn.slots.len()) {
                            Ok(v) => v,
                            Err(()) => {
                                return Some(ExecOutcome::Resource {
                                    kind: ResourceKind::SlotBytes,
                                    message: "callee frame slot allocation failed".into(),
                                })
                            }
                        };
                    st.frame_seq += 1;
                    let mut nf = Frame {
                        func: cref.func,
                        block: callee_fn.entry,
                        ip: 0,
                        slots,
                        gens: vec![0; callee_fn.slots.len()],
                        seq: st.frame_seq,
                        _allocation: allocation,
                        ret_dst: Some(*dst),
                        closure_env: Some(cref.env),
                    };
                    let caller = st.frames.last().expect("frame present");
                    let mut ai = args.iter().map(|a| caller.slots[*a as usize].clone());
                    for s in &callee_fn.slots {
                        nf.slots.push(if s.kind == crate::mir::SlotKind::Param {
                            ai.next().expect("arity verified at admission")
                        } else {
                            v.zero_of(cref.func, s.ty)
                        });
                    }
                    // Sig-driven move-in kills (mirror CALL; the signature
                    // comes from the Closure type, never an Fn slot).
                    {
                        let callee_sig_ty = v.mir_type_of(cref.func, callee_fn.sig);
                        for (ai, a) in args.iter().enumerate() {
                            let ty = func.slots[*a as usize].ty;
                            let param_borrows = callee_sig_ty
                                .and_then(|t| match t {
                                    crate::mir::MirType::Closure(sig) => {
                                        sig.params.get(ai).copied()
                                    }
                                    _ => None,
                                })
                                .map(|pt| !v.type_is_owned(cref.func, pt))
                                .unwrap_or(false);
                            if v.type_is_owned(func_idx, ty) && !param_borrows {
                                let zero = v.zero_of(func_idx, ty);
                                owned_kill_slot(st, *a as usize, zero);
                            }
                        }
                    }
                    let mut visited = 0u64;
                    let callee_bytes = frame_slots_bytes(&nf.slots, &mut visited);
                    st.steps += visited;
                    let prospective = st.slot_bytes + callee_bytes;
                    if prospective > limits.max_slot_bytes {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: format!(
                                "live slot data would reach {} bytes, above budget {}",
                                prospective, limits.max_slot_bytes
                            ),
                        });
                    }
                    st.slot_bytes = prospective;
                    st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                    st.frames.push(nf);
                }
                MirInstr::EnvGet { dst, field } => {
                    let handle = {
                        let fr = st.frames.last().expect("frame present");
                        match fr.closure_env {
                            Some(h) => h,
                            None => {
                                return Some(ExecOutcome::Infra(
                                    "env-get in a frame with no closure binding \
                                     (admission violation)"
                                        .to_string(),
                                ))
                            }
                        }
                    };
                    // Handle-share read: the env retains ownership; the
                    // budget is commit_slot_write's deep delta (the frozen
                    // conservative rule charges the shared deep size).
                    let share = match st
                        .closure_envs
                        .get(&handle)
                        .and_then(|e| e.env.field(*field as usize))
                    {
                        Some(v) => v.clone(),
                        None => {
                            return Some(ExecOutcome::Trap {
                                kind: TrapKind::InvalidHandle,
                                span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                            });
                        }
                    };
                    let mut visited = 0u64;
                    let deep = share.deep_size(&mut visited);
                    st.steps += visited;
                    if st.slot_bytes.saturating_add(deep) > limits.max_slot_bytes {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: format!(
                                "env field share {} bytes would reach {} bytes, above budget {}",
                                deep,
                                st.slot_bytes + deep,
                                limits.max_slot_bytes
                            ),
                        });
                    }
                    if let Err(out) = commit_slot_write(st, *dst as usize, share, limits) {
                        return Some(out);
                    }
                }
                MirInstr::Clone { dst, src } => {
                    // Explicit deep clone (782): per-field independent
                    // buffers, scalars by value, empty Str zero-alloc.
                    let sv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*src as usize].clone()
                    };
                    // PLAN-823 (§5.1.2): cloning a CLOSURE deep-clones its
                    // ENV — the copy owns independent resources under a
                    // fresh store handle (V-06: 副本资源独立). A stale
                    // source handle traps before any copy (defense in
                    // depth, mirroring CALL_CLOSURE).
                    if let Value::Closure(cref) = sv {
                        if !closure_ref_valid(st, cref)
                            || st
                                .closure_envs
                                .get(&cref.env)
                                .map(|e| !e.borrows.is_empty())
                                .unwrap_or(true)
                        {
                            return Some(ExecOutcome::Trap {
                                kind: TrapKind::InvalidHandle,
                                span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                            });
                        }
                        if st.closure_envs.try_reserve(1).is_err() {
                            return Some(ExecOutcome::Resource {
                                kind: ResourceKind::SlotBytes,
                                message: "could not reserve cloned closure store entry".into(),
                            });
                        }
                        let env = match st.closure_envs.get(&cref.env) {
                            Some(e) => &e.env,
                            None => {
                                return Some(ExecOutcome::Trap {
                                    kind: TrapKind::InvalidHandle,
                                    span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                                })
                            }
                        };
                        let Some(handle) = reserve_closure_handle() else {
                            return Some(ExecOutcome::Resource {
                                kind: ResourceKind::TableEntries,
                                message: "closure handle space exhausted".into(),
                            });
                        };
                        let mut visited = 0u64;
                        let env_bytes = env
                            .deep_size(&mut visited)
                            .saturating_add(closure_binding_bytes(env.field_count(), 0));
                        st.steps += visited;
                        if st.slot_bytes.saturating_add(env_bytes) > limits.max_slot_bytes {
                            return Some(ExecOutcome::Resource {
                                kind: ResourceKind::SlotBytes,
                                message: format!(
                                    "closure env clone {} bytes would reach {} bytes, above budget {}",
                                    env_bytes,
                                    st.slot_bytes + env_bytes,
                                    limits.max_slot_bytes
                                ),
                            });
                        }
                        let wall_left = limits.wall.saturating_sub(started.elapsed());
                        let mut new_fields = Vec::new();
                        if new_fields.try_reserve_exact(env.field_count()).is_err() {
                            return Some(ExecOutcome::Resource {
                                kind: ResourceKind::SlotBytes,
                                message: "could not reserve cloned environment fields".into(),
                            });
                        }
                        for i in 0..env.field_count() {
                            let f = env.field(i).expect("field_count bounds i");
                            let cloned_field = match clone_owned_value(
                                f,
                                &st.arena,
                                &cancel,
                                wall_left,
                                &mut st.steps,
                                limits.max_steps,
                            ) {
                                Ok(v) => v,
                                Err(out) => {
                                    // Partial output unwinds through the
                                    // fields' own handle drops (the visitor
                                    // fully unwound each single clone).
                                    return Some(out);
                                }
                            };
                            new_fields.push(cloned_field);
                        }
                        let new_env = crate::closures::ClosureEnv::from_fields(new_fields);
                        st.closure_envs.insert(
                            handle,
                            ClosureEntry {
                                env: new_env,
                                func: cref.func,
                                sig: cref.sig,
                                borrows: Vec::new(),
                            },
                        );
                        st.slot_bytes += env_bytes;
                        st.trace.peak_slot_bytes =
                            st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                        let new_ref = Value::Closure(crate::contract::ClosureRefV {
                            func: cref.func,
                            sig: cref.sig,
                            env: handle,
                            gen: handle,
                        });
                        if let Err(out) = commit_slot_write(st, *dst as usize, new_ref, limits) {
                            return Some(out);
                        }
                        continue;
                    }
                    let mut visited = 0u64;
                    let new_size = sv.deep_size(&mut visited);
                    if let Err(out) = owned_alloc_precheck(st, new_size, *dst as usize, limits) {
                        return Some(out);
                    }
                    // F822-R11-01: no precharge and no memory-check
                    // substitute — the visitor charges the shared
                    // cumulative fuel INCREMENTALLY: one structural unit
                    // per node plus the OUTPUT bytes of every fresh Str
                    // buffer, so the clone's cost scales with its data
                    // volume and is exhaustible mid-clone (InstrSteps).
                    // F822-R11-02: wall_left is the segment's remaining
                    // allowance AT THIS POINT; the helper measures elapsed
                    // against ITS OWN entry origin, so helper entry +
                    // wall_left reproduces the segment's absolute deadline
                    // exactly (one coordinate system, explicitly passed —
                    // no thread-local segment state participates).
                    // Cancelled, wall and fuel outcomes surface mid-clone
                    // with the partial assembly fully unwound.
                    let wall_left = limits.wall.saturating_sub(started.elapsed());
                    let cloned = match clone_owned_value(
                        &sv,
                        &st.arena,
                        &cancel,
                        wall_left,
                        &mut st.steps,
                        limits.max_steps,
                    ) {
                        Ok(v) => v,
                        Err(out) => return Some(out),
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, cloned, limits) {
                        return Some(out);
                    }
                }
                MirInstr::Move { dst, src } => {
                    // Whole-root transfer: the source slot dies HERE (V-05
                    // generation bump makes stale rooted views unusable).
                    let (sv, zero) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*src as usize].clone(),
                            v.zero_of(func_idx, func.slots[*src as usize].ty),
                        )
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, sv, limits) {
                        return Some(out);
                    }
                    owned_kill_slot(st, *src as usize, zero);
                }
                MirInstr::View { dst, src } => {
                    let (share, n) = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*src as usize] {
                            Value::Str(h) => (
                                h.clone(),
                                crate::owned_text::scalars(h.bytes()).len() as u32,
                            ),
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "VIEW source holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    let view = Value::StrView(crate::contract::StrView {
                        src: crate::contract::StrViewSrc::Share(share),
                        start: 0,
                        end: n,
                    });
                    if let Err(out) = commit_slot_write(st, *dst as usize, view, limits) {
                        return Some(out);
                    }
                }
                MirInstr::MutRef { dst, src } => {
                    // The exclusive write token: a (frame, slot,
                    // generation) handle over the ROOT slot. Writes never
                    // go through the token (the static analysis gates the
                    // root while any token is live) — the handle IS the
                    // exclusivity lock (V-05 generation domain).
                    let tok = {
                        let fr = st.frames.last().expect("frame present");
                        Value::MutRef(crate::contract::MutRefV {
                            frame: fr.seq as u32,
                            slot: *src as u32,
                            gen: fr.gens[*src as usize],
                        })
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, tok, limits) {
                        return Some(out);
                    }
                }
                MirInstr::FieldMove { dst, root, path } => {
                    // R3-F08 per-field consumption: `dst` receives the
                    // field VALUE (an Rc share for records, the handle for
                    // Str — the share IS the transfer); the leaf's slot in
                    // the root becomes its typed ZERO and the root slot's
                    // generation bumps (stale views over the moved field
                    // retire). The root stays live for its remaining
                    // fields — the write-back wrapper is consumed field by
                    // field (f0 result + one wb per mut param).
                    let field_zero = v.zero_of(func_idx, func.slots[*dst as usize].ty);
                    let taken = {
                        let fr = st.frames.last_mut().expect("frame present");
                        let root_val = &mut fr.slots[*root as usize];
                        let mut visited = 0u64;
                        let before = root_val.deep_size(&mut visited);
                        let taken = match crate::place::field_take(root_val, path, field_zero) {
                            Some(f) => f,
                            None => {
                                return Some(ExecOutcome::Infra(format!(
                                    "field-move path {:?} does not resolve on the root (admission violation)",
                                    path
                                )))
                            }
                        };
                        let after = root_val.deep_size(&mut visited);
                        st.steps += visited;
                        st.slot_bytes = st.slot_bytes.saturating_sub(before.saturating_sub(after));
                        fr.gens[*root as usize] = fr.gens[*root as usize].wrapping_add(1);
                        taken
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, taken, limits) {
                        return Some(out);
                    }
                }
                MirInstr::Drop { slot } => {
                    // PLAN-823: dropping a CLOSURE releases its env from
                    // the store FIRST (reverse capture order); the metadata
                    // cell then dies through the ordinary kill path.
                    {
                        let fr = st.frames.last().expect("frame present");
                        if let Value::Closure(cref) = fr.slots[*slot as usize] {
                            if !closure_ref_valid(st, cref) {
                                return Some(ExecOutcome::Trap {
                                    kind: TrapKind::InvalidHandle,
                                    span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                                });
                            }
                            release_closure_env(st, cref.env);
                        }
                    }
                    // Explicit drop: take the owned value out; its
                    // StrHandle drops report the frees to the arena
                    // (exactly-once, reverse-creation order inside the
                    // value), the generation bump retires stale rooted
                    // views, and the slot returns to the uninit zero.
                    let zero = v.zero_of(func_idx, func.slots[*slot as usize].ty);
                    owned_kill_slot(st, *slot as usize, zero);
                }
                MirInstr::BorrowEnd { .. } => {
                    // The token is a plain value: release is a no-op at
                    // runtime (the static analysis gates conflicts).
                }
                MirInstr::StrLen { dst, view } | MirInstr::StrLenBytes { dst, view } => {
                    let vv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*view as usize].clone()
                    };
                    let (bytes, sv) = match resolve_view_bytes(v, func_idx, &vv) {
                        Ok(x) => x,
                        Err(out) => return Some(out),
                    };
                    let res = match instr {
                        MirInstr::StrLen { .. } => (sv.end - sv.start) as i32,
                        _ => {
                            let (bs, be) = match crate::owned_text::scalar_range_to_bytes(
                                &bytes,
                                sv.start as i32,
                                sv.end as i32,
                            ) {
                                Ok(r) => r,
                                Err(kind) => {
                                    return Some(ExecOutcome::Trap {
                                        kind,
                                        span: span_of(func, flat)
                                            .map(|sp| v.map_span(func_idx, sp)),
                                    })
                                }
                            };
                            (be - bs) as i32
                        }
                    };
                    let res = Value::I32(res);
                    if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                        return Some(out);
                    }
                }
                MirInstr::StrCharAt { dst, view, index } => {
                    let (vv, iv) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*view as usize].clone(),
                            fr.slots[*index as usize].clone(),
                        )
                    };
                    let (bytes, sv) = match resolve_view_bytes(v, func_idx, &vv) {
                        Ok(x) => x,
                        Err(out) => return Some(out),
                    };
                    let idx = match iv {
                        Value::I32(i) => i,
                        other => {
                            return Some(ExecOutcome::Infra(format!(
                                "char_at index holds {} (admission violation)",
                                other.type_name()
                            )))
                        }
                    };
                    match crate::owned_text::char_at(&bytes, sv.start as i32 + idx) {
                        Ok(c) => {
                            let res = Value::Char(c);
                            if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                                return Some(out);
                            }
                        }
                        Err(kind) => {
                            return Some(ExecOutcome::Trap {
                                kind,
                                span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                            })
                        }
                    }
                }
                MirInstr::StrSlice {
                    dst,
                    view,
                    start,
                    end,
                } => {
                    let (vv, s_v, e_v) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*view as usize].clone(),
                            fr.slots[*start as usize].clone(),
                            fr.slots[*end as usize].clone(),
                        )
                    };
                    let (bytes, sv) = match resolve_view_bytes(v, func_idx, &vv) {
                        Ok(x) => x,
                        Err(out) => return Some(out),
                    };
                    let (s_i, e_i) = match (s_v, e_v) {
                        (Value::I32(a), Value::I32(b)) => (a, b),
                        _ => {
                            return Some(ExecOutcome::Infra(
                                "slice bounds hold a non-i32 (admission violation)".to_string(),
                            ))
                        }
                    };
                    // Validate against THIS view's range, then rebases.
                    if s_i < 0 || e_i < 0 || s_i > e_i || sv.start as i32 + e_i > sv.end as i32 {
                        return Some(ExecOutcome::Trap {
                            kind: crate::contract::TrapKind::StrIndex,
                            span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                        });
                    }
                    let sliced = Value::StrView(crate::contract::StrView {
                        src: vv.as_strview().unwrap().src.clone(),
                        start: sv.start + s_i as u32,
                        end: sv.start + e_i as u32,
                    });
                    let _ = bytes;
                    if let Err(out) = commit_slot_write(st, *dst as usize, sliced, limits) {
                        return Some(out);
                    }
                }
                MirInstr::StrConcat { dst, lhs, rhs } => {
                    let (lv, rv_) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*lhs as usize].clone(),
                            fr.slots[*rhs as usize].clone(),
                        )
                    };
                    let (lb, ls) = match resolve_view_bytes(v, func_idx, &lv) {
                        Ok(x) => x,
                        Err(out) => return Some(out),
                    };
                    let (rb, rs) = match resolve_view_bytes(v, func_idx, &rv_) {
                        Ok(x) => x,
                        Err(out) => return Some(out),
                    };
                    let l = project_bytes(&lb, &ls);
                    let r = project_bytes(&rb, &rs);
                    let mut joined = Vec::with_capacity(l.len() + r.len());
                    joined.extend_from_slice(l);
                    joined.extend_from_slice(r);
                    let deep = std::mem::size_of::<Value>() + joined.len();
                    if let Err(out) = owned_alloc_precheck(st, deep, *dst as usize, limits) {
                        return Some(out);
                    }
                    // F821-R1-06: concat copies both operands — byte fuel.
                    st.steps += joined.len() as u64;
                    let res = match st.arena.try_alloc(&joined) {
                        Ok(h) => Value::Str(h),
                        Err(n) => {
                            return Some(ExecOutcome::Infra(format!(
                                "fault-injected at owned allocation attempt #{n}"
                            )))
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                        return Some(out);
                    }
                }
                MirInstr::StrToOwned { dst, view } => {
                    let vv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*view as usize].clone()
                    };
                    let (bytes, sv) = match resolve_view_bytes(v, func_idx, &vv) {
                        Ok(x) => x,
                        Err(out) => return Some(out),
                    };
                    let projected = project_bytes(&bytes, &sv).to_vec();
                    let deep = std::mem::size_of::<Value>() + projected.len();
                    if let Err(out) = owned_alloc_precheck(st, deep, *dst as usize, limits) {
                        return Some(out);
                    }
                    // F821-R1-06: byte-scaled fuel for the range copy.
                    st.steps += projected.len() as u64;
                    let owned = match st.arena.try_alloc(&projected) {
                        Ok(h) => Value::Str(h),
                        Err(n) => {
                            return Some(ExecOutcome::Infra(format!(
                                "fault-injected at owned allocation attempt #{n}"
                            )))
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, owned, limits) {
                        return Some(out);
                    }
                }
                MirInstr::StrCmp { eq, dst, lhs, rhs } => {
                    let (lv, rv_) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*lhs as usize].clone(),
                            fr.slots[*rhs as usize].clone(),
                        )
                    };
                    let (lb, ls) = match resolve_view_bytes(v, func_idx, &lv) {
                        Ok(x) => x,
                        Err(out) => return Some(out),
                    };
                    let (rb, rs) = match resolve_view_bytes(v, func_idx, &rv_) {
                        Ok(x) => x,
                        Err(out) => return Some(out),
                    };
                    let same = project_bytes(&lb, &ls) == project_bytes(&rb, &rs);
                    let res = Value::Bool(if *eq { same } else { !same });
                    if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                        return Some(out);
                    }
                }
                MirInstr::CharToCodepoint { dst, ch } => {
                    let cv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*ch as usize].clone()
                    };
                    let res = match cv {
                        Value::Char(c) => Value::I32(c as i32),
                        other => {
                            return Some(ExecOutcome::Infra(format!(
                                "to_codepoint holds {} (admission violation)",
                                other.type_name()
                            )))
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                        return Some(out);
                    }
                }
                MirInstr::CharFromCodepoint { dst, code } => {
                    let cv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*code as usize].clone()
                    };
                    let res = match cv {
                        Value::I32(n) => match char::from_u32(n as u32) {
                            Some(c) => Value::Char(c),
                            None => {
                                return Some(ExecOutcome::Trap {
                                    kind: crate::contract::TrapKind::CharInvalid,
                                    span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                                })
                            }
                        },
                        other => {
                            return Some(ExecOutcome::Infra(format!(
                                "from_codepoint holds {} (admission violation)",
                                other.type_name()
                            )))
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                        return Some(out);
                    }
                }
                // -- PLAN-822 collections surface (execution) -------------
                MirInstr::ListNew { dst, list_ty, cap } => {
                    let _ = list_ty;
                    let deep = crate::contract::SLOT_CELL_BYTES
                        .saturating_mul((*cap as usize).min(crate::contract::MAX_LIST_ELEMS));
                    if let Err(out) = owned_alloc_precheck(st, deep, *dst as usize, limits) {
                        return Some(out);
                    }
                    let handle = match st.arena.try_alloc_list(Vec::new(), *cap as usize) {
                        Ok(h) => h,
                        Err(n) => {
                            return Some(ExecOutcome::Infra(format!(
                                "fault-injected at owned allocation attempt #{n}"
                            )))
                        }
                    };
                    st.steps += 1;
                    if let Err(out) =
                        commit_slot_write(st, *dst as usize, Value::List(handle), limits)
                    {
                        return Some(out);
                    }
                }
                MirInstr::ListLen { dst, src } | MirInstr::ListCap { dst, src } => {
                    let n = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*src as usize] {
                            Value::List(h) => match instr {
                                MirInstr::ListLen { .. } => h.rc.borrow().elems.len() as i32,
                                _ => h.rc.borrow().cap as i32,
                            },
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, Value::I32(n), limits) {
                        return Some(out);
                    }
                }
                MirInstr::ListGet { dst, base, index } => {
                    let iv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*index as usize].clone()
                    };
                    let idx = match list_index_of(&iv) {
                        Some(i) => i,
                        None => return Some(list_index_trap(v, func_idx, func, flat)),
                    };
                    let elem = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*base as usize] {
                            Value::List(h) => {
                                let buf = h.rc.borrow();
                                if idx >= buf.elems.len() {
                                    drop(buf);
                                    return Some(list_index_trap(v, func_idx, func, flat));
                                }
                                buf.elems[idx].clone()
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, elem, limits) {
                        return Some(out);
                    }
                }
                MirInstr::ListPush { root, value } => {
                    let val = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*value as usize].clone()
                    };
                    // Per-element fuel + budget BEFORE any mutation (a
                    // failure leaves the list and source untouched).
                    let mut visited = 0u64;
                    let elem_deep = val.deep_size(&mut visited);
                    st.steps += visited;
                    if st.slot_bytes.saturating_add(elem_deep) > limits.max_slot_bytes {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: format!(
                                "live slot data would reach {} bytes, above budget {}",
                                st.slot_bytes + elem_deep,
                                limits.max_slot_bytes
                            ),
                        });
                    }
                    let push = {
                        let fr = st.frames.last_mut().expect("frame present");
                        match fr.slots[*root as usize].clone() {
                            Value::List(h) => {
                                let mut buf = h.rc.borrow_mut();
                                if buf.elems.len() >= crate::contract::MAX_LIST_ELEMS {
                                    drop(buf);
                                    return Some(ExecOutcome::Resource {
                                        kind: ResourceKind::ListElems,
                                        message: format!(
                                            "list length would exceed {}",
                                            crate::contract::MAX_LIST_ELEMS
                                        ),
                                    });
                                }
                                if buf.elems.len() as u32 == buf.cap {
                                    let new_cap = grow_cap(buf.cap);
                                    if new_cap as usize > crate::contract::MAX_LIST_ELEMS {
                                        drop(buf);
                                        return Some(ExecOutcome::Resource {
                                            kind: ResourceKind::ListElems,
                                            message: format!(
                                                "list capacity would exceed {}",
                                                crate::contract::MAX_LIST_ELEMS
                                            ),
                                        });
                                    }
                                    buf.cap = new_cap;
                                }
                                buf.elems.push(val.clone());
                                buf.elem_gens.push(0);
                                buf.epoch = buf.epoch.wrapping_add(1);
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                        true
                    };
                    let _ = push;
                    st.slot_bytes += elem_deep;
                    st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                    // The source transfers: kill an owned value slot.
                    kill_value_slot(v, func_idx, func, st, *value as usize);
                }
                MirInstr::ListInsert { root, index, value } => {
                    let (iv, val) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*index as usize].clone(),
                            fr.slots[*value as usize].clone(),
                        )
                    };
                    let idx = match list_index_of(&iv) {
                        Some(i) => i,
                        None => return Some(list_index_trap(v, func_idx, func, flat)),
                    };
                    let mut visited = 0u64;
                    let elem_deep = val.deep_size(&mut visited);
                    st.steps += visited;
                    if st.slot_bytes.saturating_add(elem_deep) > limits.max_slot_bytes {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::SlotBytes,
                            message: format!(
                                "live slot data would reach {} bytes, above budget {}",
                                st.slot_bytes + elem_deep,
                                limits.max_slot_bytes
                            ),
                        });
                    }
                    let shifted = {
                        let fr = st.frames.last_mut().expect("frame present");
                        match fr.slots[*root as usize].clone() {
                            Value::List(h) => {
                                let mut buf = h.rc.borrow_mut();
                                if idx > buf.elems.len()
                                    || buf.elems.len() >= crate::contract::MAX_LIST_ELEMS
                                {
                                    drop(buf);
                                    return Some(list_index_trap(v, func_idx, func, flat));
                                }
                                if buf.elems.len() as u32 == buf.cap {
                                    let new_cap = grow_cap(buf.cap);
                                    if new_cap as usize > crate::contract::MAX_LIST_ELEMS {
                                        drop(buf);
                                        return Some(ExecOutcome::Resource {
                                            kind: ResourceKind::ListElems,
                                            message: format!(
                                                "list capacity would exceed {}",
                                                crate::contract::MAX_LIST_ELEMS
                                            ),
                                        });
                                    }
                                    buf.cap = new_cap;
                                }
                                // The tail shift is billed per moved element.
                                st.steps += (buf.elems.len() - idx) as u64;
                                buf.elems.insert(idx, val.clone());
                                buf.elem_gens.insert(idx, 0);
                                buf.epoch = buf.epoch.wrapping_add(1);
                                buf.elems.len() - idx - 1
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    let _ = shifted;
                    st.slot_bytes += elem_deep;
                    st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                    kill_value_slot(v, func_idx, func, st, *value as usize);
                }
                MirInstr::ListSet { root, index, value } => {
                    let (iv, val) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*index as usize].clone(),
                            fr.slots[*value as usize].clone(),
                        )
                    };
                    let idx = match list_index_of(&iv) {
                        Some(i) => i,
                        None => return Some(list_index_trap(v, func_idx, func, flat)),
                    };
                    // F822-R1-01: PRECHECK before any mutation — a budget
                    // rejection leaves the element, the source slot and
                    // the ledger untouched (§5.1.2 失败旧列表/源不变).
                    let mut visited = 0u64;
                    let new_deep = val.deep_size(&mut visited);
                    st.steps += visited;
                    let old_deep = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*root as usize] {
                            Value::List(h) => {
                                let buf = h.rc.borrow();
                                match buf.elems.get(idx) {
                                    Some(old) => {
                                        let mut v2 = 0u64;
                                        let d = old.deep_size(&mut v2);
                                        st.steps += v2;
                                        d
                                    }
                                    None => {
                                        drop(buf);
                                        return Some(list_index_trap(v, func_idx, func, flat));
                                    }
                                }
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    if new_deep > old_deep {
                        let delta = new_deep - old_deep;
                        if st.slot_bytes.saturating_add(delta) > limits.max_slot_bytes {
                            return Some(ExecOutcome::Resource {
                                kind: ResourceKind::SlotBytes,
                                message: format!(
                                    "live slot data would reach {} bytes, above budget {}",
                                    st.slot_bytes + delta,
                                    limits.max_slot_bytes
                                ),
                            });
                        }
                    }
                    // COMMIT: the swap-in and the old element's single
                    // release happen only after the budget admitted them.
                    let old_elem = {
                        let fr = st.frames.last_mut().expect("frame present");
                        match fr.slots[*root as usize].clone() {
                            Value::List(h) => {
                                let mut buf = h.rc.borrow_mut();
                                let old = std::mem::replace(&mut buf.elems[idx], val.clone());
                                buf.elem_gens[idx] = buf.elem_gens[idx].wrapping_add(1);
                                old
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    // F822-R12-01: the replaced element (any supported
                    // nested shape) releases through the explicit reverse
                    // cleanup, not the host drop glue.
                    drop_owned_reverse(st, old_elem);
                    if new_deep > old_deep {
                        st.slot_bytes += new_deep - old_deep;
                    } else {
                        st.slot_bytes = st.slot_bytes.saturating_sub(old_deep - new_deep);
                    }
                    st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                    kill_value_slot(v, func_idx, func, st, *value as usize);
                }
                MirInstr::ListRemove { dst, root, index } => {
                    let iv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*index as usize].clone()
                    };
                    let idx = match list_index_of(&iv) {
                        Some(i) => i,
                        None => return Some(list_index_trap(v, func_idx, func, flat)),
                    };
                    // F822-R1-01 (same-cause sibling): precheck the
                    // destination's budget BEFORE the element moves out —
                    // a rejection leaves the list (contents AND epoch)
                    // untouched; the tail shift is billed per element.
                    let removed_deep = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*root as usize] {
                            Value::List(h) => {
                                let buf = h.rc.borrow();
                                match buf.elems.get(idx) {
                                    Some(e) => {
                                        st.steps += (buf.elems.len() - 1 - idx) as u64;
                                        let mut v2 = 0u64;
                                        let d = e.deep_size(&mut v2);
                                        st.steps += v2;
                                        d
                                    }
                                    None => {
                                        drop(buf);
                                        return Some(list_index_trap(v, func_idx, func, flat));
                                    }
                                }
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    if let Err(out) = owned_alloc_precheck(st, removed_deep, *dst as usize, limits)
                    {
                        return Some(out);
                    }
                    // COMMIT: the transfer-out and the epoch bump happen
                    // only after the budget admitted the destination.
                    let removed = {
                        let fr = st.frames.last_mut().expect("frame present");
                        match fr.slots[*root as usize].clone() {
                            Value::List(h) => {
                                let mut buf = h.rc.borrow_mut();
                                let out = buf.elems.remove(idx);
                                buf.elem_gens.remove(idx);
                                buf.epoch = buf.epoch.wrapping_add(1);
                                out
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    // The root shed the element; the destination gains it.
                    st.slot_bytes = st.slot_bytes.saturating_sub(removed_deep);
                    if let Err(out) = commit_slot_write(st, *dst as usize, removed, limits) {
                        return Some(out);
                    }
                }
                MirInstr::ListFieldGet {
                    dst,
                    base,
                    index,
                    path,
                } => {
                    let iv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*index as usize].clone()
                    };
                    let idx = match list_index_of(&iv) {
                        Some(i) => i,
                        None => return Some(list_index_trap(v, func_idx, func, flat)),
                    };
                    let got = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*base as usize] {
                            Value::List(h) => {
                                let buf = h.rc.borrow();
                                match buf.elems.get(idx) {
                                    Some(e) => field_get_val(e, path),
                                    None => {
                                        drop(buf);
                                        return Some(list_index_trap(v, func_idx, func, flat));
                                    }
                                }
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    let res = match got {
                        Some(x) => x,
                        None => {
                            return Some(ExecOutcome::Infra(format!(
                                "list-field-get path {:?} does not resolve (admission violation)",
                                path
                            )))
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                        return Some(out);
                    }
                }
                MirInstr::ListFieldSet {
                    root,
                    index,
                    path,
                    value,
                } => {
                    let (iv, val) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*index as usize].clone(),
                            fr.slots[*value as usize].clone(),
                        )
                    };
                    let idx = match list_index_of(&iv) {
                        Some(i) => i,
                        None => return Some(list_index_trap(v, func_idx, func, flat)),
                    };
                    // F822-R1-01 (same-cause sibling): PRECHECK the rebuilt
                    // element's deep delta BEFORE committing — a budget
                    // rejection leaves the field, the list and the source
                    // untouched; the old field chain drops once at commit.
                    let mut visited = 0u64;
                    let _val_deep = val.deep_size(&mut visited);
                    st.steps += visited;
                    let new_elem = {
                        let fr = st.frames.last().expect("frame present");
                        let handle = match fr.slots[*root as usize].clone() {
                            Value::List(h) => h,
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        };
                        let buf = handle.rc.borrow();
                        match buf.elems.get(idx) {
                            None => {
                                drop(buf);
                                return Some(list_index_trap(v, func_idx, func, flat));
                            }
                            Some(elem) => {
                                match crate::place::record_set_path(elem, path, val.clone()) {
                                    Some(ne) => {
                                        let mut v2 = 0u64;
                                        let nd = ne.deep_size(&mut v2);
                                        st.steps += v2;
                                        let mut v3 = 0u64;
                                        let od = elem.deep_size(&mut v3);
                                        st.steps += v3;
                                        if nd > od {
                                            let delta = nd - od;
                                            if st.slot_bytes.saturating_add(delta)
                                                > limits.max_slot_bytes
                                            {
                                                return Some(ExecOutcome::Resource {
                                                    kind: ResourceKind::SlotBytes,
                                                    message: format!(
                                                        "live slot data would reach {} bytes, above budget {}",
                                                        st.slot_bytes + delta,
                                                        limits.max_slot_bytes
                                                    ),
                                                });
                                            }
                                        }
                                        (ne, nd, od)
                                    }
                                    None => {
                                        return Some(ExecOutcome::Infra(format!(
                                            "list-field-set path {:?} does not resolve (admission violation)",
                                            path
                                        )))
                                    }
                                }
                            }
                        }
                    };
                    // COMMIT: swap the rebuilt element in; the old
                    // element's replaced field chain releases here, once.
                    let old_elem = {
                        let fr = st.frames.last_mut().expect("frame present");
                        match fr.slots[*root as usize].clone() {
                            Value::List(h) => {
                                let mut buf = h.rc.borrow_mut();
                                let old = std::mem::replace(&mut buf.elems[idx], new_elem.0);
                                buf.elem_gens[idx] = buf.elem_gens[idx].wrapping_add(1);
                                old
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    // F822-R12-01: explicit reverse cleanup for the
                    // replaced chain (no host drop-glue recursion).
                    drop_owned_reverse(st, old_elem);
                    if new_elem.1 > new_elem.2 {
                        st.slot_bytes += new_elem.1 - new_elem.2;
                    } else {
                        st.slot_bytes = st.slot_bytes.saturating_sub(new_elem.2 - new_elem.1);
                    }
                    st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                    kill_value_slot(v, func_idx, func, st, *value as usize);
                }
                MirInstr::VariantPack {
                    dst,
                    variant_ty,
                    disc,
                    payload,
                } => {
                    let payload_val = match payload {
                        Some(p) => {
                            let fr = st.frames.last().expect("frame present");
                            Some(fr.slots[*p as usize].clone())
                        }
                        None => None,
                    };
                    let res = Value::Variant(crate::contract::VariantValue {
                        ty: *variant_ty,
                        disc: *disc,
                        payload: payload_val.map(std::rc::Rc::new),
                    });
                    if let Err(out) = commit_slot_write(st, *dst as usize, res, limits) {
                        return Some(out);
                    }
                    if let Some(p) = payload {
                        kill_value_slot(v, func_idx, func, st, *p as usize);
                    }
                }
                MirInstr::VariantDisc { dst, src } => {
                    let d = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*src as usize] {
                            Value::Variant(var) => var.disc as i32,
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "variant operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, Value::I32(d), limits) {
                        return Some(out);
                    }
                }
                MirInstr::VariantGuard { dst, src, disc } => {
                    let b = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*src as usize] {
                            Value::Variant(var) => var.disc == *disc,
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "variant operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, Value::Bool(b), limits) {
                        return Some(out);
                    }
                }
                MirInstr::VariantProject { dst, root, disc } => {
                    // Runtime defense in depth: the wrong arm or an
                    // already-moved payload traps BEFORE any read.
                    let current = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*root as usize] {
                            Value::Variant(var) => (var.disc, var.payload.is_some()),
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "variant operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    if current.0 != *disc || !current.1 {
                        return Some(ExecOutcome::Trap {
                            kind: crate::contract::TrapKind::VariantBranch,
                            span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                        });
                    }
                    let root_slot_ty = func.slots[*root as usize].ty;
                    let payload_owned = v.type_is_owned(
                        func_idx,
                        variant_payload_ty(v, func_idx, root_slot_ty, *disc),
                    );
                    if payload_owned {
                        // F822-R1-01 (same-cause sibling): precheck the
                        // destination BEFORE the payload moves out — a
                        // budget rejection leaves the variant's arm intact.
                        let mut visited = 0u64;
                        let deep = {
                            let fr = st.frames.last().expect("frame present");
                            match &fr.slots[*root as usize] {
                                Value::Variant(var) => var
                                    .payload
                                    .as_ref()
                                    .expect("checked above")
                                    .deep_size(&mut visited),
                                _ => unreachable!("checked above"),
                            }
                        };
                        st.steps += visited;
                        if let Err(out) = owned_alloc_precheck(st, deep, *dst as usize, limits) {
                            return Some(out);
                        }
                        // COMMIT: take the payload, shrink the root's
                        // live-data accounting, grow the destination.
                        let taken = {
                            let fr = st.frames.last_mut().expect("frame present");
                            match &mut fr.slots[*root as usize] {
                                Value::Variant(var) => var
                                    .payload
                                    .take()
                                    .map(|rc| (*rc).clone())
                                    .expect("checked above"),
                                _ => unreachable!("checked above"),
                            }
                        };
                        st.slot_bytes = st.slot_bytes.saturating_sub(deep);
                        if let Err(out) = commit_slot_write(st, *dst as usize, taken, limits) {
                            return Some(out);
                        }
                    } else {
                        let taken = {
                            let fr = st.frames.last_mut().expect("frame present");
                            match &mut fr.slots[*root as usize] {
                                Value::Variant(var) => var.payload.take().expect("checked above"),
                                _ => unreachable!("checked above"),
                            }
                        };
                        // Copy payload: the arm keeps its value.
                        let copy = taken.as_ref().clone();
                        {
                            let fr = st.frames.last_mut().expect("frame present");
                            match &mut fr.slots[*root as usize] {
                                Value::Variant(var) => {
                                    var.payload = Some(taken);
                                }
                                _ => unreachable!("checked above"),
                            }
                        }
                        if let Err(out) = commit_slot_write(st, *dst as usize, copy, limits) {
                            return Some(out);
                        }
                    }
                }
                MirInstr::VariantSwitch {
                    root,
                    disc,
                    payload,
                } => {
                    let payload_val = match payload {
                        Some(p) => {
                            let fr = st.frames.last().expect("frame present");
                            Some(fr.slots[*p as usize].clone())
                        }
                        None => None,
                    };
                    // The old live arm's payload drops exactly once inside
                    // commit_slot_write's old-value replacement.
                    let res = Value::Variant(crate::contract::VariantValue {
                        ty: {
                            let fr = st.frames.last().expect("frame present");
                            match &fr.slots[*root as usize] {
                                Value::Variant(var) => var.ty,
                                other => {
                                    return Some(ExecOutcome::Infra(format!(
                                        "variant operand holds {} (admission violation)",
                                        other.type_name()
                                    )))
                                }
                            }
                        },
                        disc: *disc,
                        payload: payload_val.map(std::rc::Rc::new),
                    });
                    if let Err(out) = commit_slot_write(st, *root as usize, res, limits) {
                        return Some(out);
                    }
                    if let Some(p) = payload {
                        kill_value_slot(v, func_idx, func, st, *p as usize);
                    }
                }
                // -- PLAN-822 R2 index-place surface (execution) ---------
                MirInstr::ListPlace { dst, base, index } => {
                    let iv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*index as usize].clone()
                    };
                    let idx = match list_index_of(&iv) {
                        Some(i) => i,
                        None => return Some(list_index_trap(v, func_idx, func, flat)),
                    };
                    let place = {
                        let fr = st.frames.last().expect("frame present");
                        match &fr.slots[*base as usize] {
                            Value::List(h) => {
                                let buf = h.rc.borrow();
                                if idx >= buf.elems.len() {
                                    drop(buf);
                                    return Some(list_index_trap(v, func_idx, func, flat));
                                }
                                crate::contract::ListPlaceV {
                                    frame: fr.seq as u32,
                                    slot: *base as u32,
                                    buf_id: buf.id,
                                    epoch: buf.epoch,
                                    index: idx as u32,
                                    elem_gen: buf.elem_gens[idx],
                                }
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    };
                    if let Err(out) =
                        commit_slot_write(st, *dst as usize, Value::Place(place), limits)
                    {
                        return Some(out);
                    }
                }
                MirInstr::PlaceGet { dst, src } => {
                    let pv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*src as usize].clone()
                    };
                    let place = match pv {
                        Value::Place(p) => p,
                        other => {
                            return Some(ExecOutcome::Infra(format!(
                                "place operand holds {} (admission violation)",
                                other.type_name()
                            )))
                        }
                    };
                    // Root resolution + stale validation BEFORE any access.
                    let elem = match resolve_place_elem(st, &place) {
                        Ok(e) => e,
                        Err(kind) => {
                            return Some(ExecOutcome::Trap {
                                kind,
                                span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                            })
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, elem, limits) {
                        return Some(out);
                    }
                }
                MirInstr::PlaceSet { place, value } => {
                    let (pv, val) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*place as usize].clone(),
                            fr.slots[*value as usize].clone(),
                        )
                    };
                    let place = match pv {
                        Value::Place(p) => p,
                        other => {
                            return Some(ExecOutcome::Infra(format!(
                                "place operand holds {} (admission violation)",
                                other.type_name()
                            )))
                        }
                    };
                    // PRECHECK (F822-R1-01 discipline): stale validation and
                    // budget before any mutation.
                    let mut visited = 0u64;
                    let new_deep = val.deep_size(&mut visited);
                    st.steps += visited;
                    let old_deep = match validate_place_for_write(st, &place) {
                        Ok(d) => d,
                        Err(kind) => {
                            return Some(ExecOutcome::Trap {
                                kind,
                                span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                            })
                        }
                    };
                    if new_deep > old_deep {
                        let delta = new_deep - old_deep;
                        if st.slot_bytes.saturating_add(delta) > limits.max_slot_bytes {
                            return Some(ExecOutcome::Resource {
                                kind: ResourceKind::SlotBytes,
                                message: format!(
                                    "live slot data would reach {} bytes, above budget {}",
                                    st.slot_bytes + delta,
                                    limits.max_slot_bytes
                                ),
                            });
                        }
                    }
                    // COMMIT: replace the element, bump its generation
                    // (retiring THIS place and every sibling minted before
                    // the replacement), release the old element once.
                    // F822-R5-01: the OWNER frame (by activation sequence)
                    // carries the root — the same frame the precheck just
                    // validated — never the CURRENT frame's same-numbered
                    // slot.
                    let old_elem = {
                        let fr = st
                            .frames
                            .iter_mut()
                            .find(|f| f.seq as u32 == place.frame)
                            .expect("precheck located the owner frame");
                        match &fr.slots[place.slot as usize] {
                            Value::List(h) => {
                                let mut buf = h.rc.borrow_mut();
                                let idx = place.index as usize;
                                let old = std::mem::replace(&mut buf.elems[idx], val.clone());
                                buf.elem_gens[idx] = buf.elem_gens[idx].wrapping_add(1);
                                old
                            }
                            _ => unreachable!("precheck validated the owner slot holds the list"),
                        }
                    };
                    // F822-R12-01: explicit reverse cleanup for the
                    // replaced element (no host drop-glue recursion).
                    drop_owned_reverse(st, old_elem);
                    if new_deep > old_deep {
                        st.slot_bytes += new_deep - old_deep;
                    } else {
                        st.slot_bytes = st.slot_bytes.saturating_sub(old_deep - new_deep);
                    }
                    st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                    kill_value_slot(v, func_idx, func, st, *value as usize);
                }
                // -- PLAN-824 dynamic surface (execution) ---------------
                MirInstr::DynPack { dst, src } => {
                    let val = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*src as usize].clone()
                    };
                    let src_ty = func.slots[*src as usize].ty;
                    let dyn_ty =
                        match resolve_dyn_type(v, func_idx, src_ty, st, limits, cancel, started) {
                            DynTypeResolve::Ready(t) => t,
                            DynTypeResolve::Stopped(out) => return Some(out),
                            DynTypeResolve::OutsideClosure => {
                                return Some(ExecOutcome::Infra(
                                    "dyn-pack source type is outside the erasable closure \
                                 (admission violation)"
                                        .to_string(),
                                ))
                            }
                        };
                    // The executor re-checks the admission the verifier
                    // gated (defense in depth): a mismatch here is an
                    // admission violation, never a silent degrade.
                    let mut admission_stop = None;
                    let packed = crate::dynamic::DynValue::pack_with_work(val, dyn_ty, &mut || {
                        match dyn_exec_work(st, limits, cancel, started) {
                            Ok(()) => true,
                            Err(out) => {
                                admission_stop = Some(out);
                                false
                            }
                        }
                    });
                    let packed = match packed {
                        Ok(p) => std::rc::Rc::new(p),
                        Err(e) => {
                            if let Some(out) = admission_stop {
                                return Some(out);
                            }
                            return Some(ExecOutcome::Infra(format!(
                                "dyn-pack admission rejected the value ({})",
                                e.code()
                            )));
                        }
                    };
                    st.steps += 1;
                    if let Err(out) =
                        commit_slot_write(st, *dst as usize, Value::Dyn(packed), limits)
                    {
                        return Some(out);
                    }
                }
                MirInstr::DynIs { dst, src, ty } => {
                    let sv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*src as usize].clone()
                    };
                    let want = match resolve_dyn_type(v, func_idx, *ty, st, limits, cancel, started)
                    {
                        DynTypeResolve::Ready(t) => t,
                        DynTypeResolve::Stopped(out) => return Some(out),
                        DynTypeResolve::OutsideClosure => {
                            return Some(ExecOutcome::Infra(
                                "dyn-is type operand is outside the erasable closure \
                                 (admission violation)"
                                    .to_string(),
                            ))
                        }
                    };
                    let hits = match &sv {
                        Value::Dyn(d) => {
                            match dyn_type_equal(d.ty(), &want, st, limits, cancel, started) {
                                Ok(same) => same,
                                Err(out) => return Some(out),
                            }
                        }
                        other => {
                            return Some(ExecOutcome::Infra(format!(
                                "dyn-is operand holds {} (admission violation)",
                                other.type_name()
                            )))
                        }
                    };
                    st.steps += 1;
                    if let Err(out) =
                        commit_slot_write(st, *dst as usize, Value::Bool(hits), limits)
                    {
                        return Some(out);
                    }
                }
                MirInstr::DynTake { dst, src, ty } => {
                    let sv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*src as usize].clone()
                    };
                    let want = match resolve_dyn_type(v, func_idx, *ty, st, limits, cancel, started)
                    {
                        DynTypeResolve::Ready(t) => t,
                        DynTypeResolve::Stopped(out) => return Some(out),
                        DynTypeResolve::OutsideClosure => {
                            return Some(ExecOutcome::Infra(
                                "dyn-take type operand is outside the erasable closure \
                                 (admission violation)"
                                    .to_string(),
                            ))
                        }
                    };
                    st.steps += 1;
                    match &sv {
                        Value::Dyn(d) => {
                            let same =
                                match dyn_type_equal(d.ty(), &want, st, limits, cancel, started) {
                                    Ok(same) => same,
                                    Err(out) => return Some(out),
                                };
                            if same {
                                // v1 take materializes a SHARE of the inner
                                // value (Value::clone — the Value model's
                                // own semantics): the Dyn keeps its share,
                                // Rc ownership releases at the last owner.
                                let out = d.as_value().clone();
                                if let Err(out) = commit_slot_write(st, *dst as usize, out, limits)
                                {
                                    return Some(out);
                                }
                            } else {
                                // A RUNTIME CHECKED failure: the source
                                // stays intact in its slot, nothing dropped
                                // or half-moved.
                                return Some(ExecOutcome::Trap {
                                    kind: crate::contract::TrapKind::DynTypeMismatch,
                                    span: span_of(func, flat).map(|sp| v.map_span(func_idx, sp)),
                                });
                            }
                        }
                        other => {
                            return Some(ExecOutcome::Infra(format!(
                                "dyn-take operand holds {} (admission violation)",
                                other.type_name()
                            )))
                        }
                    }
                }
                MirInstr::HostCall { host: index, arg } => {
                    let Some(h) = v.host_imports().get(*index as usize) else {
                        return Some(ExecOutcome::Infra(
                            "host.reference: missing declaration".into(),
                        ));
                    };
                    let Some((_, stream)) = crate::host::signature(&h.symbol) else {
                        return Some(ExecOutcome::Infra("host.unknown-symbol".into()));
                    };
                    if st.frames.len() >= limits.max_frames {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::Frames,
                            message: "host call exceeds call depth".into(),
                        });
                    }
                    if CancelCheck::poll(cancel) {
                        return Some(ExecOutcome::Cancelled);
                    }
                    let input = st.frames.last().expect("frame").slots[*arg as usize].clone();
                    let result = with_console_input(&input, |bytes| {
                        let len = bytes
                            .len()
                            .checked_add(1)
                            .ok_or_else(|| console_output_limit("call length overflow"))?;
                        let total = st
                            .output_bytes
                            .checked_add(len)
                            .ok_or_else(|| console_output_limit("run length overflow"))?;
                        if len > crate::host::MAX_CALL_BYTES || total > crate::host::MAX_RUN_BYTES {
                            return Err(console_output_limit(
                                "console output byte budget exceeded",
                            ));
                        }
                        let peak = st
                            .slot_bytes
                            .checked_add(len.saturating_mul(2))
                            .ok_or_else(|| console_output_limit("output memory overflow"))?;
                        if peak > limits.max_slot_bytes {
                            return Err(ExecOutcome::Resource {
                                kind: ResourceKind::SlotBytes,
                                message: "console staging + retained output exceeds memory budget"
                                    .into(),
                            });
                        }
                        let steps = st
                            .steps
                            .checked_add(len as u64)
                            .ok_or_else(|| console_output_limit("fuel overflow"))?;
                        if steps > limits.max_steps {
                            return Err(ExecOutcome::Resource {
                                kind: ResourceKind::InstrSteps,
                                message: "console byte work exceeds fuel".into(),
                            });
                        }
                        st.steps = steps;
                        let mut line = Vec::new();
                        line.try_reserve_exact(len)
                            .map_err(|_| ExecOutcome::Resource {
                                kind: ResourceKind::SlotBytes,
                                message: "cannot reserve console staging".into(),
                            })?;
                        line.extend_from_slice(bytes);
                        line.push(b'\n');
                        if CancelCheck::poll(cancel) {
                            return Err(ExecOutcome::Cancelled);
                        }
                        if started.elapsed() > limits.wall {
                            return Err(ExecOutcome::Resource {
                                kind: ResourceKind::ExecutionWall,
                                message: "console preparation exceeded wall budget".into(),
                            });
                        }
                        st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(peak as u64);
                        let provider = host.as_deref_mut().ok_or_else(|| {
                            ExecOutcome::Infra("host.unauthorized: no provider".into())
                        })?;
                        let provider_result = {
                            let _scope = crate::host::ProviderScope::enter()
                                .ok_or_else(|| ExecOutcome::Infra("host.reentry".into()))?;
                            provider.write(stream, &line, cancel)
                        };
                        provider_result.map_err(|e| match e {
                            crate::host::HostError::Resource => {
                                console_output_limit("provider output resource failure")
                            }
                            crate::host::HostError::Cancelled => ExecOutcome::Cancelled,
                            crate::host::HostError::Io(message) => {
                                ExecOutcome::Infra(format!("host.io: {message}"))
                            }
                        })?;
                        st.output_bytes = total;
                        st.slot_bytes += len;
                        if CancelCheck::poll(cancel) {
                            return Err(ExecOutcome::Cancelled);
                        }
                        if started.elapsed() > limits.wall {
                            return Err(ExecOutcome::Resource {
                                kind: ResourceKind::ExecutionWall,
                                message: "host call exceeded wall budget".into(),
                            });
                        }
                        Ok(())
                    });
                    if let Err(out) = result {
                        return Some(out);
                    }
                }
                MirInstr::Trace { event } => {
                    st.trace.events.push(*event);
                    if st.trace.events.len() > limits.max_trace_bytes {
                        return Some(ExecOutcome::Resource {
                            kind: ResourceKind::TraceBytes,
                            message: format!(
                                "test trace exceeded {} bytes",
                                limits.max_trace_bytes
                            ),
                        });
                    }
                }
                MirInstr::FieldGet { dst, base, path } => {
                    let bv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*base as usize].clone()
                    };
                    let dst_ty = func.slots[*dst as usize].ty;
                    if v.is_strview_type(func_idx, dst_ty) {
                        // PLAN-821: a Str-field read is a BORROW VIEW rooted
                        // at the base slot's place (782: 字段读取=视图) —
                        // the owned value never leaves its root.
                        let field = match crate::place::field_project(&bv, path) {
                            Some(f) => f.clone(),
                            None => {
                                return Some(ExecOutcome::Infra(format!(
                                    "field-get path {:?} does not resolve on the base value (admission violation)",
                                    path
                                )))
                            }
                        };
                        let share = match &field {
                            Value::Str(h) => h.clone(),
                            _ => {
                                return Some(ExecOutcome::Infra(
                                    "field-get view of a non-Str field (admission violation)"
                                        .to_string(),
                                ))
                            }
                        };
                        let n = crate::owned_text::scalars(share.bytes()).len() as u32;
                        // 774 snapshot: the view shares the FIELD's buffer —
                        // a later replace leaves it reading the old bytes.
                        let view = Value::StrView(crate::contract::StrView {
                            src: crate::contract::StrViewSrc::Share(share),
                            start: 0,
                            end: n,
                        });
                        if let Err(out) = commit_slot_write(st, *dst as usize, view, limits) {
                            return Some(out);
                        }
                        continue;
                    }
                    let picked = match field_get_val(&bv, path) {
                        Some(x) => x,
                        None => {
                            return Some(ExecOutcome::Infra(format!(
                                "field-get path {:?} does not resolve on the base value (admission violation)",
                                path
                            )))
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *dst as usize, picked, limits) {
                        return Some(out);
                    }
                }
                MirInstr::FieldSet { root, path, value } => {
                    // RHS-first materialization: both operands are read
                    // from already-materialized slots, then the root is
                    // rebuilt — the overlap invariant holds by shape.
                    let (root_val, new_val) = {
                        let fr = st.frames.last().expect("frame present");
                        (
                            fr.slots[*root as usize].clone(),
                            fr.slots[*value as usize].clone(),
                        )
                    };
                    let rebuilt = match record_set_val(&root_val, path, new_val) {
                        Some(x) => x,
                        None => {
                            return Some(ExecOutcome::Infra(format!(
                                "field-set path {:?} does not resolve on the root value (admission violation)",
                                path
                            )))
                        }
                    };
                    if let Err(out) = commit_slot_write(st, *root as usize, rebuilt, limits) {
                        return Some(out);
                    }
                    // F821-R1-04: the analysis marks an owned VALUE slot
                    // moved at a field replace — the runtime must match
                    // (the source's Rc must not ghost-own the buffer past
                    // the commit; failure paths above return BEFORE this).
                    if v.type_is_owned(func_idx, func.slots[*value as usize].ty) {
                        let zero = v.zero_of(func_idx, func.slots[*value as usize].ty);
                        owned_kill_slot(st, *value as usize, zero);
                    }
                }
                MirInstr::Aggregate {
                    dst,
                    record_ty,
                    operands,
                } => {
                    let fields: Vec<Value> = {
                        let fr = st.frames.last().expect("frame present");
                        operands
                            .iter()
                            .map(|s| fr.slots[*s as usize].clone())
                            .collect()
                    };
                    let rv = Value::Record(std::rc::Rc::new(crate::contract::RecordValue {
                        ty: *record_ty,
                        fields,
                    }));
                    if let Err(out) = commit_slot_write(st, *dst as usize, rv, limits) {
                        return Some(out);
                    }
                    // F821-R1-04: owned operands TRANSFER into the record —
                    // the source slots die at the commit (matching the
                    // analysis's kill), so the buffers release with the
                    // record, not with ghost owners at frame end.
                    for s in operands {
                        let ty = func.slots[*s as usize].ty;
                        if v.type_is_owned(func_idx, ty) {
                            let zero = v.zero_of(func_idx, ty);
                            owned_kill_slot(st, *s as usize, zero);
                        }
                    }
                }
            }
            continue;
        }
        // Terminator.
        match &block.term {
            Terminator::Jump { target } => {
                let fr = st.frames.last_mut().expect("frame present");
                fr.block = *target;
                fr.ip = 0;
            }
            Terminator::Branch { cond, then, else_ } => {
                let go = {
                    let fr = st.frames.last().expect("frame present");
                    match &fr.slots[*cond as usize] {
                        Value::Bool(b) => *b,
                        other => {
                            return Some(ExecOutcome::Infra(format!(
                                "branch condition holds {} (admission violation)",
                                other.type_name()
                            )))
                        }
                    }
                };
                let fr = st.frames.last_mut().expect("frame present");
                fr.block = if go { *then } else { *else_ };
                fr.ip = 0;
            }
            Terminator::Return { value } => {
                let (rv, ret_dst) = {
                    let fr = st.frames.last().expect("frame present");
                    (fr.slots[*value as usize].clone(), fr.ret_dst)
                };
                // PLAN-823: a returned Closure is Copy METADATA — the env
                // responsibility leaves with `rv`; zero the slot BEFORE the
                // frame sweep so the sweep cannot release the escaped env
                // (shared-kind values need no such step: their handle
                // clones keep the buffers alive through the sweep).
                {
                    let fr = st.frames.last_mut().expect("frame present");
                    if matches!(fr.slots[*value as usize], Value::Closure(_)) {
                        fr.slots[*value as usize] = Value::I32(0);
                    }
                }
                // Deep live-data of the popped frame leaves the ledger.
                let mut visited = 0u64;
                let frame_bytes = {
                    let fr = st.frames.last().expect("frame present");
                    frame_slots_bytes(&fr.slots, &mut visited)
                };
                st.steps += visited;
                st.slot_bytes = st.slot_bytes.saturating_sub(frame_bytes);
                // PLAN-821: the popped frame's remaining owned values
                // release in reverse creation order (result responsibility
                // already transferred OUT in `rv`).
                let mut frame = st.frames.pop().expect("frame present");
                release_frame_owned_lifo(st, &mut frame);
                drop(frame);
                match (st.frames.last_mut(), ret_dst) {
                    (Some(_), Some(dst)) => {
                        if let Err(out) = commit_slot_write(st, dst as usize, rv, limits) {
                            return Some(out);
                        }
                    }
                    (Some(_), None) => {}
                    (None, _) => {
                        return Some(ExecOutcome::Success {
                            value: rv,
                            steps: st.steps,
                        })
                    }
                }
            }
        }
    }
}

fn span_of(f: &MirFunction, flat: u32) -> Option<SpanInfo> {
    let mut lo = 0usize;
    let mut hi = f.spans.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        match f.spans[mid].0.cmp(&flat) {
            std::cmp::Ordering::Less => lo = mid + 1,
            _ => hi = mid,
        }
    }
    if lo < f.spans.len() && f.spans[lo].0 == flat {
        Some(f.spans[lo].1)
    } else {
        None
    }
}

impl MirFunction {
    /// Param slots in signature order (admission guarantees they lead).
    pub fn param_slots(&self) -> Vec<u32> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == crate::mir::SlotKind::Param)
            .map(|(i, _)| i as u32)
            .collect()
    }
}

fn eval_bin(op: BinOp, l: Value, r: Value) -> Result<Value, EvalErr> {
    use BinOp::*;
    match (l, r) {
        (Value::I32(a), Value::I32(b)) => Ok(match op {
            AddI32 => Value::I32(
                a.checked_add(b)
                    .ok_or(EvalErr::Trap(TrapKind::IntOverflow))?,
            ),
            SubI32 => Value::I32(
                a.checked_sub(b)
                    .ok_or(EvalErr::Trap(TrapKind::IntOverflow))?,
            ),
            MulI32 => Value::I32(
                a.checked_mul(b)
                    .ok_or(EvalErr::Trap(TrapKind::IntOverflow))?,
            ),
            DivI32 => {
                if b == 0 {
                    return Err(EvalErr::Trap(TrapKind::DivByZero));
                }
                if a == i32::MIN && b == -1 {
                    return Err(EvalErr::Trap(TrapKind::DivOverflow));
                }
                Value::I32(a / b)
            }
            RemI32 => {
                if b == 0 {
                    return Err(EvalErr::Trap(TrapKind::DivByZero));
                }
                if a == i32::MIN && b == -1 {
                    return Err(EvalErr::Trap(TrapKind::RemOverflow));
                }
                Value::I32(a % b)
            }
            LtI32 => Value::Bool(a < b),
            LeI32 => Value::Bool(a <= b),
            GtI32 => Value::Bool(a > b),
            GeI32 => Value::Bool(a >= b),
            EqI32 => Value::Bool(a == b),
            NeI32 => Value::Bool(a != b),
        }),
        (Value::Bool(a), Value::Bool(b)) => match op {
            EqI32 => Ok(Value::Bool(a == b)),
            NeI32 => Ok(Value::Bool(a != b)),
            other => Err(EvalErr::Infra(format!(
                "binary {:?} on bool operands (admission violation)",
                other
            ))),
        },
        _ => Err(EvalErr::Infra(format!(
            "binary {:?} on mixed-type operands (admission violation)",
            op
        ))),
    }
}

fn eval_un(op: UnOp, v: Value) -> Result<Value, EvalErr> {
    match (op, v) {
        (UnOp::NegI32, Value::I32(a)) => Ok(Value::I32(
            a.checked_neg()
                .ok_or(EvalErr::Trap(TrapKind::IntOverflow))?,
        )),
        (UnOp::NotBool, Value::Bool(b)) => Ok(Value::Bool(!b)),
        _ => Err(EvalErr::Infra(format!(
            "unary {:?} on wrong-shaped operand (admission violation)",
            op
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{MIR_SCHEMA, SLOT_CELL_BYTES};
    use crate::mir::*;

    fn owner() -> OwnerScope {
        OwnerScope {
            schema: MIR_SCHEMA.to_string(),
            module: "m".to_string(),
        }
    }

    /// `fn main() -> i32 { let r = 7; return r }` — one local slot.
    fn loaded() -> LoadedModule {
        let mut m = MirModule::new(owner());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        let c7 = m.intern_const(MirConst::I32(7));
        m.funcs.push(MirFunction {
            owner: owner(),
            name: "main".into(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: Some("r".into()),
            }],
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::Const { dst: 0, value: c7 }],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.entry = 0;
        let bytes = crate::abc::encode_abc(&m).expect("encode");
        crate::abc_verify::verify_abc_bytes(&bytes.to_flat(), &owner()).expect("admit")
    }

    #[test]
    fn av_zero_frame_budget_rejects_root_frame() {
        let m = loaded();
        let cancel = CancelFlag::new(false);
        let limits = ExecLimits {
            max_frames: 0,
            ..ExecLimits::default()
        };
        let (out, trace) = execute(&m, &limits, &cancel);
        assert!(
            matches!(
                out,
                ExecOutcome::Resource {
                    kind: ResourceKind::Frames,
                    ..
                }
            ),
            "got {out:?}"
        );
        assert_eq!(trace.peak_slot_bytes, 0);
    }

    #[test]
    fn av_zero_slot_budget_rejects_root_frame() {
        let m = loaded();
        let cancel = CancelFlag::new(false);
        let limits = ExecLimits {
            max_slot_bytes: 0,
            ..ExecLimits::default()
        };
        let (out, trace) = execute(&m, &limits, &cancel);
        assert!(
            matches!(
                out,
                ExecOutcome::Resource {
                    kind: ResourceKind::SlotBytes,
                    ..
                }
            ),
            "got {out:?}"
        );
        assert_eq!(trace.peak_slot_bytes, 0);
    }

    /// F798-R12-02 (replaces the R6 oracle "typed-param unit executes"):
    /// a UNIT-kind credential — even one that would satisfy every other
    /// closure rule (no imports, trivial body) — is never executable
    /// through the entry seam. The unit→closure conversion is driver
    /// closure assembly; a param entry is only VALID on units, so the
    /// entry-args kind admission below is defense-in-depth for credentials
    /// that should no longer exist, and this test pins the front gate.
    #[test]
    fn av_unit_kind_param_credential_not_executable() {
        let mut m = MirModule::unit(owner());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![ti],
            result: ti,
        }));
        m.funcs.push(MirFunction {
            owner: owner(),
            name: "main".into(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Param,
                ty: ti,
                name: None,
            }],
            blocks: vec![MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.entry = 0;
        let bytes = crate::abc::encode_abc(&m).expect("encode");
        let l = crate::abc_verify::verify_abc_bytes(&bytes.to_flat(), &owner()).expect("admit");
        assert!(!l.executable(), "unit kind is a re-verifiable credential");
        let cancel = CancelFlag::new(false);
        // Correct-kind args still never dispatch: the KIND gate fires first.
        let (out, _) = execute_entry(&l, &ExecLimits::default(), &cancel, &[Value::I32(5)]);
        assert!(
            matches!(&out, ExecOutcome::Infra(msg) if msg.contains("unit kind")),
            "got {out:?}"
        );
    }

    /// F798-R12-02: a zero-param i32 unit — a payload that would satisfy
    /// every COMPLETE-module entry rule — is still not executable; the
    /// deciding axis is the KIND declaration, proven by the identical
    /// body declared as a complete module executing successfully.
    #[test]
    fn av_unit_kind_zero_param_credential_not_executable() {
        let build = |is_unit: bool| {
            let mut m = if is_unit {
                MirModule::unit(owner())
            } else {
                MirModule::new(owner())
            };
            let ti = m.intern_type(MirType::I32);
            let sig = m.intern_type(MirType::Fn(FnSig {
                params: vec![],
                result: ti,
            }));
            let c = m.intern_const(MirConst::I32(7));
            m.funcs.push(MirFunction {
                owner: owner(),
                name: "main".into(),
                sig,
                slots: vec![MirSlot {
                    kind: SlotKind::Local,
                    ty: ti,
                    name: Some("r".into()),
                }],
                blocks: vec![MirBlock {
                    instrs: vec![MirInstr::Const { dst: 0, value: c }],
                    term: Terminator::Return { value: 0 },
                }],
                entry: 0,
                source_snapshot: 0,
                spans: vec![],
                env: None,
            });
            m.entry = 0;
            let bytes = crate::abc::encode_abc(&m).expect("encode");
            crate::abc_verify::verify_abc_bytes(&bytes.to_flat(), &owner()).expect("admit")
        };
        let cancel = CancelFlag::new(false);
        let unit = build(true);
        assert!(!unit.executable());
        let (out, _) = execute(&unit, &ExecLimits::default(), &cancel);
        assert!(
            matches!(&out, ExecOutcome::Infra(msg) if msg.contains("unit kind")),
            "unit kind refused before dispatch: {out:?}"
        );
        // Positive control: the IDENTICAL body declared a complete module
        // is an executable closure (K14-04: a legal kind declaration is
        // judged by the complete-module rules — not rejected for the flip).
        let whole = build(false);
        assert!(whole.executable());
        let (out, _) = execute(&whole, &ExecLimits::default(), &cancel);
        assert!(
            matches!(
                out,
                ExecOutcome::Success {
                    value: crate::contract::Value::I32(7),
                    ..
                }
            ),
            "complete-module control executes: {out:?}"
        );
    }

    /// F798-R6-02: a Call on ANY import yields a RE-VERIFIABLE UNIT
    /// credential — the executor refuses it up front instead of running
    /// into "unresolved import" infra mid-dispatch.
    #[test]
    fn av_unit_with_import_call_is_not_executable() {
        let mut m = MirModule::new(owner());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        let fi_t = m.intern_type(MirType::I32);
        let _ = fi_t;
        let imp_sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        m.imports.push(MirImport {
            symbol: "hir.test.mark_a".to_string(),
            effect: Effect::Observable,
            sig: imp_sig,
            env: None,
        });
        m.capabilities = vec![crate::contract::TRACE_CAPABILITY.to_string()];
        m.funcs.push(MirFunction {
            owner: owner(),
            name: "main".into(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: Some("r".into()),
            }],
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::Call {
                    callee: CalleeRef::Import(0),
                    dst: 0,
                    args: vec![],
                }],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.entry = 0;
        let bytes = crate::abc::encode_abc(&m).expect("encode");
        let l = crate::abc_verify::verify_abc_bytes(&bytes.to_flat(), &owner()).expect("admit");
        assert!(
            !l.executable(),
            "an import-calling module admits only as a unit"
        );
        let (out, _) = execute(&l, &ExecLimits::default(), &CancelFlag::new(false));
        assert!(
            matches!(&out, ExecOutcome::Infra(msg) if msg.contains("re-verifiable unit")),
            "credential gate fires before any dispatch: {out:?}"
        );
    }

    #[test]
    fn av_success_reports_real_peak_slot_bytes() {
        let m = loaded();
        let cancel = CancelFlag::new(false);
        let (out, trace) = execute(&m, &ExecLimits::default(), &cancel);
        assert!(matches!(out, ExecOutcome::Success { .. }), "got {out:?}");
        // One live local slot — the reported high-water must be its real
        // byte size, not the historic always-zero (F798-R3-04).
        assert_eq!(
            trace.peak_slot_bytes,
            1 * SLOT_CELL_BYTES as u64,
            "peak must cover the root frame's live slot"
        );
    }

    /// Fault injection into a live frame: stale generation and absent store
    /// handles must trap at the real call instruction before callee entry.
    #[test]
    fn av_823_stale_closure_identity_traps_before_callee() {
        let closure_owner = OwnerScope {
            schema: crate::contract::MIR_CLOSURES_SCHEMA.into(),
            module: "m".into(),
        };
        let mut m = MirModule::new_closures(closure_owner.clone());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        let cl = m.intern_type(MirType::Closure(FnSig {
            params: vec![],
            result: ti,
        }));
        let n = m.intern_const(MirConst::I32(29));
        let slot = |ty| MirSlot {
            kind: SlotKind::Local,
            ty,
            name: None,
        };
        m.funcs.push(MirFunction {
            owner: closure_owner.clone(),
            name: "main".into(),
            sig,
            slots: vec![slot(cl), slot(ti)],
            blocks: vec![MirBlock {
                instrs: vec![
                    MirInstr::CreateClosure {
                        dst: 0,
                        callee: CalleeRef::Func(1),
                        captures: vec![],
                    },
                    MirInstr::CallClosure {
                        dst: 1,
                        closure: 0,
                        args: vec![],
                    },
                    MirInstr::Drop { slot: 0 },
                ],
                term: Terminator::Return { value: 1 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.funcs.push(MirFunction {
            owner: closure_owner.clone(),
            name: "body".into(),
            sig,
            slots: vec![slot(ti)],
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::Const { dst: 0, value: n }],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: Some(vec![]),
        });
        m.entry = 0;
        let limits = ExecLimits::default();
        let cancel = CancelFlag::new(false);
        for operation in ["call", "clone", "drop"] {
            let mut variant = m.clone();
            match operation {
                "clone" => {
                    variant.funcs[0].slots.push(slot(cl));
                    variant.funcs[0].blocks[0]
                        .instrs
                        .insert(1, MirInstr::Clone { dst: 2, src: 0 });
                    variant.funcs[0].blocks[0]
                        .instrs
                        .insert(2, MirInstr::Drop { slot: 2 });
                }
                "drop" => {
                    variant.funcs[0].blocks[0].instrs = vec![
                        MirInstr::CreateClosure {
                            dst: 0,
                            callee: CalleeRef::Func(1),
                            captures: vec![],
                        },
                        MirInstr::Drop { slot: 0 },
                        MirInstr::Const { dst: 1, value: n },
                    ];
                }
                _ => {}
            }
            let wire = crate::abc::encode_abc(&variant).unwrap();
            let loaded =
                crate::abc_verify::verify_abc_bytes(&wire.to_flat(), &closure_owner).unwrap();
            let (control, _) = execute(&loaded, &limits, &cancel);
            assert!(
                matches!(
                    control,
                    ExecOutcome::Success {
                        value: Value::I32(29),
                        ..
                    }
                ),
                "{operation}: {control:?}"
            );
            for fault in [
                "generation",
                "handle",
                "function",
                "signature",
                "foreign-store",
            ] {
                let mut st = LoopState::default();
                let view = MergedView(loaded.module());
                loop {
                    let out = dispatch(&view, &limits, &cancel, &[], &mut st, Some(1), false);
                    assert!(out.is_none(), "must pause before {operation}");
                    if matches!(
                        st.frames.last().and_then(|f| f.slots.first()),
                        Some(Value::Closure(_))
                    ) {
                        break;
                    }
                    assert!(st.steps < 100, "closure creation was not reached");
                }
                let mut foreign = LoopState::default();
                let foreign_reference = if fault == "foreign-store" {
                    loop {
                        let out =
                            dispatch(&view, &limits, &cancel, &[], &mut foreign, Some(1), false);
                        assert!(out.is_none(), "foreign execution must pause after creation");
                        if let Some(Value::Closure(reference)) =
                            foreign.frames.last().and_then(|f| f.slots.first())
                        {
                            break Some(*reference);
                        }
                        assert!(foreign.steps < 100);
                    }
                } else {
                    None
                };
                let Value::Closure(reference) = &mut st.frames.last_mut().unwrap().slots[0] else {
                    unreachable!()
                };
                let protected_handle = reference.env;
                match fault {
                    "generation" => reference.gen = reference.gen.wrapping_add(1),
                    "handle" => reference.env = reference.env.wrapping_add(1),
                    "function" => reference.func = 0,
                    "signature" => reference.sig = reference.sig.wrapping_add(1),
                    "foreign-store" => {
                        let other = foreign_reference.unwrap();
                        assert_ne!(
                            other.env, protected_handle,
                            "live stores need distinct handles"
                        );
                        *reference = other;
                    }
                    _ => unreachable!(),
                }
                let out = dispatch(&view, &limits, &cancel, &[], &mut st, None, false).unwrap();
                assert!(
                    matches!(
                        out,
                        ExecOutcome::Trap {
                            kind: TrapKind::InvalidHandle,
                            ..
                        }
                    ),
                    "{operation}: {out:?}"
                );
                assert_eq!(st.frames.len(), 1, "target callee was never entered");
                assert!(
                    st.closure_envs.contains_key(&protected_handle),
                    "{operation} must not release the protected environment"
                );
                cleanup_owned_frames(&mut st);
                cleanup_owned_frames(&mut foreign);
                assert!(
                    st.closure_envs.is_empty(),
                    "all-exit cleanup releases orphaned environments after frames unwind"
                );
            }
        }
    }

    #[test]
    fn av_823_env_budget_refusal_preserves_move_source() {
        let own = OwnerScope {
            schema: crate::contract::MIR_CLOSURES_SCHEMA.into(),
            module: "budget".into(),
        };
        let mut m = MirModule::new_closures(own.clone());
        let ti = m.intern_type(MirType::I32);
        let ts = m.intern_type(MirType::Str);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        let cl = m.intern_type(MirType::Closure(FnSig {
            params: vec![],
            result: ti,
        }));
        let text = m.intern_const(MirConst::Str(Box::from(*b"owned")));
        let ret = m.intern_const(MirConst::I32(1));
        let slot = |ty| MirSlot {
            kind: SlotKind::Local,
            ty,
            name: None,
        };
        m.funcs.push(MirFunction {
            owner: own.clone(),
            name: "main".into(),
            sig,
            slots: vec![slot(ts), slot(cl), slot(ti), slot(ts)],
            blocks: vec![MirBlock {
                instrs: vec![
                    MirInstr::Const {
                        dst: 0,
                        value: text,
                    },
                    MirInstr::Const {
                        dst: 3,
                        value: text,
                    },
                    MirInstr::CreateClosure {
                        dst: 1,
                        callee: CalleeRef::Func(1),
                        captures: vec![
                            CaptureOperand {
                                kind: CaptureKind::Move,
                                src: 0,
                            },
                            CaptureOperand {
                                kind: CaptureKind::Clone,
                                src: 3,
                            },
                        ],
                    },
                    MirInstr::Drop { slot: 1 },
                    MirInstr::Const { dst: 2, value: ret },
                ],
                term: Terminator::Return { value: 2 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.funcs.push(MirFunction {
            owner: own.clone(),
            name: "body".into(),
            sig,
            slots: vec![slot(ti)],
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::Const { dst: 0, value: ret }],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: Some(vec![ts, ts]),
        });
        m.entry = 0;
        let wire = crate::abc::encode_abc(&m).unwrap();
        let loaded = crate::abc_verify::verify_abc_bytes(&wire.to_flat(), &own).unwrap();
        let view = MergedView(loaded.module());
        let mut st = LoopState::default();
        let cancel = CancelFlag::new(false);
        let limits = ExecLimits::default();
        loop {
            let out = dispatch(&view, &limits, &cancel, &[], &mut st, Some(1), false);
            assert!(out.is_none(), "must pause before capture");
            if st.frames.last().map(|f| f.ip == 2).unwrap_or(false) {
                break;
            }
            assert!(st.steps < 100);
        }
        let Value::Str(original) = &st.frames.last().unwrap().slots[0] else {
            panic!("source must be initialized")
        };
        let id = original.rc.id;
        let allocs_before = st.arena.alloc_events();
        let mut visited = 0;
        let old = st.frames.last().unwrap().slots[0].deep_size(&mut visited);
        let zero = view.zero_of(0, ts).deep_size(&mut visited);
        let clone_bytes = st.frames.last().unwrap().slots[3].deep_size(&mut visited);
        let projected = st.slot_bytes.saturating_sub(old.saturating_sub(zero))
            + old
            + clone_bytes
            + closure_binding_bytes(2, 0);
        assert!(projected > st.slot_bytes);
        let mut limited = limits;
        limited.max_slot_bytes = projected - 1;
        let outcome = dispatch(&view, &limited, &cancel, &[], &mut st, None, false).unwrap();
        assert!(
            matches!(
                outcome,
                ExecOutcome::Resource {
                    kind: ResourceKind::SlotBytes,
                    ..
                }
            ),
            "{outcome:?}"
        );
        assert!(
            matches!(&st.frames.last().unwrap().slots[0], Value::Str(h) if h.rc.id == id && h.bytes() == b"owned"),
            "budget refusal must preserve the uncommitted source"
        );
        assert_eq!(
            st.arena.alloc_events(),
            allocs_before,
            "budget failure must precede clone allocation"
        );
        assert!(
            matches!(&st.frames.last().unwrap().slots[3], Value::Str(h) if h.bytes() == b"owned")
        );
        assert!(
            st.closure_envs.is_empty(),
            "failure must not publish an environment"
        );
        cleanup_owned_frames(&mut st);
        limited.max_slot_bytes = projected;
        let (control, trace) = execute(&loaded, &limited, &cancel);
        assert!(
            matches!(
                control,
                ExecOutcome::Success {
                    value: Value::I32(1),
                    ..
                }
            ),
            "the exact boundary must remain admissible: {control:?}"
        );
        assert_eq!(trace.arena.unwrap().live(), 0);
    }

    #[test]
    fn av_823_borrow_rejects_expired_frame_and_slot_generation() {
        let own = OwnerScope {
            schema: crate::contract::MIR_CLOSURES_SCHEMA.into(),
            module: "borrow".into(),
        };
        let mut m = MirModule::new_closures(own.clone());
        let ti = m.intern_type(MirType::I32);
        let ts = m.intern_type(MirType::Str);
        let tv = m.intern_type(MirType::StrView);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        let cl = m.intern_type(MirType::Closure(FnSig {
            params: vec![],
            result: ti,
        }));
        let text = m.intern_const(MirConst::Str(Box::from(*b"owned")));
        let slot = |ty| MirSlot {
            kind: SlotKind::Local,
            ty,
            name: None,
        };
        m.funcs.push(MirFunction {
            owner: own.clone(),
            name: "main".into(),
            sig,
            slots: vec![slot(ts), slot(cl), slot(ti)],
            blocks: vec![MirBlock {
                instrs: vec![
                    MirInstr::Const {
                        dst: 0,
                        value: text,
                    },
                    MirInstr::CreateClosure {
                        dst: 1,
                        callee: CalleeRef::Func(1),
                        captures: vec![CaptureOperand {
                            kind: CaptureKind::Borrow,
                            src: 0,
                        }],
                    },
                    MirInstr::CallClosure {
                        dst: 2,
                        closure: 1,
                        args: vec![],
                    },
                    MirInstr::Drop { slot: 1 },
                ],
                term: Terminator::Return { value: 2 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.funcs.push(MirFunction {
            owner: own.clone(),
            name: "body".into(),
            sig,
            slots: vec![slot(tv), slot(ti)],
            blocks: vec![MirBlock {
                instrs: vec![
                    MirInstr::EnvGet { dst: 0, field: 0 },
                    MirInstr::StrLen { dst: 1, view: 0 },
                ],
                term: Terminator::Return { value: 1 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: Some(vec![tv]),
        });
        m.entry = 0;
        let wire = crate::abc::encode_abc(&m).unwrap();
        let loaded = crate::abc_verify::verify_abc_bytes(&wire.to_flat(), &own).unwrap();
        let limits = ExecLimits::default();
        let cancel = CancelFlag::new(false);
        let (control, trace) = execute(&loaded, &limits, &cancel);
        assert!(
            matches!(
                control,
                ExecOutcome::Success {
                    value: Value::I32(5),
                    ..
                }
            ),
            "{control:?}"
        );
        assert_eq!(trace.arena.unwrap().live(), 0);
        for stale_frame in [true, false] {
            let mut st = LoopState::default();
            let view = MergedView(loaded.module());
            loop {
                let out = dispatch(&view, &limits, &cancel, &[], &mut st, Some(1), false);
                assert!(out.is_none(), "must pause before the closure call");
                if matches!(
                    st.frames.last().and_then(|f| f.slots.get(1)),
                    Some(Value::Closure(_))
                ) {
                    break;
                }
                assert!(st.steps < 100);
            }
            let frame = st.frames.last_mut().unwrap();
            if stale_frame {
                frame.seq = frame.seq.wrapping_add(1);
            } else {
                frame.gens[0] = frame.gens[0].wrapping_add(1);
            }
            let out = dispatch(&view, &limits, &cancel, &[], &mut st, None, false).unwrap();
            assert!(
                matches!(
                    out,
                    ExecOutcome::Trap {
                        kind: TrapKind::InvalidHandle,
                        ..
                    }
                ),
                "{out:?}"
            );
            assert_eq!(
                st.frames.len(),
                1,
                "invalid borrow must not enter the target callee"
            );
            cleanup_owned_frames(&mut st);
            assert!(
                st.closure_envs.is_empty(),
                "owned closure environment is released"
            );
            assert_eq!(st.arena.live(), 0);
        }
    }

    #[test]
    fn av_823_nested_env_budget_refusal_preserves_parent_field() {
        let own = OwnerScope {
            schema: crate::contract::MIR_CLOSURES_SCHEMA.into(),
            module: "nested-budget".into(),
        };
        let mut m = MirModule::new_closures(own.clone());
        let ti = m.intern_type(MirType::I32);
        let ts = m.intern_type(MirType::Str);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        let cl = m.intern_type(MirType::Closure(FnSig {
            params: vec![],
            result: ti,
        }));
        let text = m.intern_const(MirConst::Str(Box::from(*b"owned")));
        let ret = m.intern_const(MirConst::I32(1));
        let slot = |ty| MirSlot {
            kind: SlotKind::Local,
            ty,
            name: None,
        };
        m.funcs.push(MirFunction {
            owner: own.clone(),
            name: "main".into(),
            sig,
            slots: vec![slot(ts), slot(cl), slot(ti)],
            blocks: vec![MirBlock {
                instrs: vec![
                    MirInstr::Const {
                        dst: 0,
                        value: text,
                    },
                    MirInstr::CreateClosure {
                        dst: 1,
                        callee: CalleeRef::Func(1),
                        captures: vec![CaptureOperand {
                            kind: CaptureKind::Move,
                            src: 0,
                        }],
                    },
                    MirInstr::CallClosure {
                        dst: 2,
                        closure: 1,
                        args: vec![],
                    },
                    MirInstr::Drop { slot: 1 },
                ],
                term: Terminator::Return { value: 2 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.funcs.push(MirFunction {
            owner: own.clone(),
            name: "parent".into(),
            sig,
            slots: vec![slot(cl), slot(ti)],
            blocks: vec![MirBlock {
                instrs: vec![
                    MirInstr::CreateClosure {
                        dst: 0,
                        callee: CalleeRef::Func(2),
                        captures: vec![CaptureOperand {
                            kind: CaptureKind::MoveFromEnv,
                            src: 0,
                        }],
                    },
                    MirInstr::Drop { slot: 0 },
                    MirInstr::Const { dst: 1, value: ret },
                ],
                term: Terminator::Return { value: 1 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: Some(vec![ts]),
        });
        m.funcs.push(MirFunction {
            owner: own.clone(),
            name: "child".into(),
            sig,
            slots: vec![slot(ti)],
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::Const { dst: 0, value: ret }],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: Some(vec![ts]),
        });
        m.entry = 0;
        let wire = crate::abc::encode_abc(&m).unwrap();
        let loaded = crate::abc_verify::verify_abc_bytes(&wire.to_flat(), &own).unwrap();
        let view = MergedView(loaded.module());
        let mut st = LoopState::default();
        let cancel = CancelFlag::new(false);
        let limits = ExecLimits::default();
        loop {
            let out = dispatch(&view, &limits, &cancel, &[], &mut st, Some(1), false);
            assert!(out.is_none(), "must pause in parent before nested capture");
            if st
                .frames
                .last()
                .map(|f| f.func == 1 && f.ip == 0)
                .unwrap_or(false)
            {
                break;
            }
            assert!(st.steps < 200);
        }
        let handle = st.frames.last().unwrap().closure_env.unwrap();
        let field = st.closure_envs.get(&handle).unwrap().env.field(0).unwrap();
        assert!(matches!(field, Value::Str(h) if h.bytes() == b"owned"));
        let mut visited = 0;
        let field_bytes = field.deep_size(&mut visited);
        let projected = st
            .slot_bytes
            .saturating_sub(field_bytes.saturating_sub(std::mem::size_of::<Value>()))
            + field_bytes
            + closure_binding_bytes(1, 0);
        let mut limited = limits;
        limited.max_slot_bytes = projected - 1;
        let outcome = dispatch(&view, &limited, &cancel, &[], &mut st, None, false).unwrap();
        assert!(
            matches!(
                outcome,
                ExecOutcome::Resource {
                    kind: ResourceKind::SlotBytes,
                    ..
                }
            ),
            "{outcome:?}"
        );
        assert!(
            matches!(st.closure_envs.get(&handle).unwrap().env.field(0), Some(Value::Str(h)) if h.bytes() == b"owned"),
            "parent field must remain owned on pre-commit refusal"
        );
        assert_eq!(
            st.closure_envs.len(),
            1,
            "no child environment is published"
        );
        cleanup_owned_frames(&mut st);
        assert_eq!(st.arena.live(), 0);
        limited.max_slot_bytes = projected;
        let (control, trace) = execute(&loaded, &limited, &cancel);
        assert!(
            matches!(
                control,
                ExecOutcome::Success {
                    value: Value::I32(1),
                    ..
                }
            ),
            "{control:?}"
        );
        assert_eq!(trace.arena.unwrap().live(), 0);
    }

    #[test]
    fn av_823_orphaned_env_cleanup_releases_in_reverse_order() {
        let mut st = LoopState::default();
        let arena = st.arena.clone();
        let first = reserve_closure_handle().unwrap();
        let second = reserve_closure_handle().unwrap();
        st.closure_envs.insert(
            first,
            ClosureEntry {
                env: crate::closures::ClosureEnv::from_fields(vec![
                    Value::Str(arena.alloc(b"a")),
                    Value::Str(arena.alloc(b"b")),
                ]),
                func: 1,
                sig: 0,
                borrows: Vec::new(),
            },
        );
        st.closure_envs.insert(
            second,
            ClosureEntry {
                env: crate::closures::ClosureEnv::from_fields(vec![Value::Str(arena.alloc(b"c"))]),
                func: 1,
                sig: 0,
                borrows: Vec::new(),
            },
        );
        assert_eq!(arena.live(), 3);
        cleanup_owned_frames(&mut st);
        assert!(st.closure_envs.is_empty());
        assert_eq!(arena.live(), 0);
        assert_eq!(
            arena.free_events().iter().map(|e| e.id).collect::<Vec<_>>(),
            vec![2, 1, 0]
        );
    }
}

fn console_output_limit(message: &str) -> ExecOutcome {
    ExecOutcome::Resource {
        kind: ResourceKind::ConsoleOutput,
        message: message.into(),
    }
}
/// Borrow/projection happens before formatting; never clone an unbounded text root.
fn with_console_input<T>(
    input: &Value,
    f: impl FnOnce(&[u8]) -> Result<T, ExecOutcome>,
) -> Result<T, ExecOutcome> {
    match input {
        Value::I32(n) => {
            let mut buf = [0u8; 11];
            let mut cursor = buf.len();
            let mut v = n.unsigned_abs();
            loop {
                cursor -= 1;
                buf[cursor] = b'0' + (v % 10) as u8;
                v /= 10;
                if v == 0 {
                    break;
                }
            }
            if *n < 0 {
                cursor -= 1;
                buf[cursor] = b'-';
            }
            f(&buf[cursor..])
        }
        Value::Bool(v) => f(if *v { b"true" } else { b"false" }),
        Value::Char(c) => {
            let mut buf = [0u8; 4];
            f(c.encode_utf8(&mut buf).as_bytes())
        }
        Value::StrView(view) => {
            let project = |bytes: &[u8]| -> Result<T, ExecOutcome> {
                let (s, e) = crate::owned_text::scalar_range_to_bytes(
                    bytes,
                    view.start as i32,
                    view.end as i32,
                )
                .map_err(|_| ExecOutcome::Infra("host.signature: invalid text range".into()))?;
                f(&bytes[s..e])
            };
            match &view.src {
                crate::contract::StrViewSrc::Static { buf } => project(buf),
                crate::contract::StrViewSrc::Share(_) => {
                    let resolved = crate::place::resolve_share_view(view).ok_or_else(|| {
                        ExecOutcome::Infra("host.signature: stale text view".into())
                    })?;
                    project(resolved.handle.bytes())
                }
            }
        }
        _ => Err(ExecOutcome::Infra(
            "host.signature: wrong runtime argument".into(),
        )),
    }
}
