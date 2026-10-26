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
    #[allow(dead_code)]
    seq: u64,
    _allocation: Option<crate::frame_allocation::AllocationLease>,
    /// Caller bookkeeping: destination slot written on return (None = root).
    ret_dst: Option<u32>,
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

/// PLAN-821 T-04 test seam: execute with a PRE-CONFIGURED arena (fault
/// injection / attempt accounting observers). The arena is Rc-shared —
/// the returned trace's arena reflects the injected instance.
pub fn execute_with_arena(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    arena: crate::owned_text::StrArena,
) -> (ExecOutcome, ExecTrace) {
    let m = module.module();
    if !module.executable() {
        return (
            ExecOutcome::Infra(format!(
                "admission violation: credential is a re-verifiable unit, not an executable closure"
            )),
            ExecTrace::default(),
        );
    }
    let mut st = LoopState::default();
    st.arena = arena;
    st.trace.arena = Some(st.arena.clone());
    let out = dispatch(&MergedView(m), limits, cancel, &[], &mut st, None)
        .expect("no budget: the loop always terminates with an outcome");
    if !st.frames.is_empty() {
        cleanup_owned_frames(&mut st);
    }
    (out, st.trace)
}

/// Execute with explicit entry arguments (driver seam for unit closures —
/// the root frame receives them as the entry's param slots).
pub fn execute_entry(
    module: &LoadedModule,
    limits: &ExecLimits,
    cancel: &CancelFlag,
    args: &[Value],
) -> (ExecOutcome, ExecTrace) {
    let m = module.module();

    // Executable-closure gate (F798-R6-02 + F798-R12-02): a re-verifiable
    // UNIT — by kind or by an unresolved import call — is never run;
    // closure assembly resolves those callees and re-verifies the whole
    // module first. Zero dispatch steps: an admission refusal, not a run.
    if !module.executable() {
        let unit_kind = m.is_unit;
        return (
            ExecOutcome::Infra(format!(
                "admission violation: credential is a re-verifiable unit ({}), not an executable closure — close it into a whole module first",
                if unit_kind {
                    "unit kind"
                } else {
                    "unresolved import call"
                }
            )),
            ExecTrace::default(),
        );
    }
    let mut st = LoopState::default();
    st.arena = crate::owned_text::StrArena::new();
    st.trace.arena = Some(st.arena.clone());
    let out = dispatch(&MergedView(m), limits, cancel, args, &mut st, None)
        .expect("no budget: the loop always terminates with an outcome");
    if !st.frames.is_empty() {
        // All-exit cleanup: trap/resource/cancel leaves frames behind —
        // their owned values release before the outcome surfaces.
        cleanup_owned_frames(&mut st);
    }
    (out, st.trace)
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
    let mut st = LoopState::default();
    st.arena = crate::owned_text::StrArena::new();
    st.trace.arena = Some(st.arena.clone());
    st.trace.execution_seq = EXECUTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    st.trace.dispatched = observe_dispatched(prog);
    let out = dispatch(&prog.unit_view(), limits, cancel, &[], &mut st, None)
        .expect("no budget: the loop always terminates with an outcome");
    if !st.frames.is_empty() {
        cleanup_owned_frames(&mut st);
    }
    (out, st.trace)
}

pub fn execute_program(
    prog: &crate::program_snapshot::ProgramSnapshot,
    limits: &ExecLimits,
    cancel: &CancelFlag,
) -> (ExecOutcome, ExecTrace) {
    let mut st = LoopState::default();
    st.arena = crate::owned_text::StrArena::new();
    st.trace.arena = Some(st.arena.clone());
    st.trace.execution_seq = EXECUTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    st.trace.dispatched = observe_dispatched(prog);
    let out = dispatch(&prog.unit_view(), limits, cancel, &[], &mut st, None)
        .expect("no budget: the loop always terminates with an outcome");
    if !st.frames.is_empty() {
        cleanup_owned_frames(&mut st);
    }
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
}

impl PausedRun {
    /// Cumulative guest instruction steps executed so far across all
    /// segments of this chain (B2-C04 observability: a zero-budget
    /// resume must not move it).
    pub fn step_count(&self) -> u64 {
        self.state.steps
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
    let mut st = LoopState::default();
    st.arena = crate::owned_text::StrArena::new();
    st.trace.arena = Some(st.arena.clone());
    st.trace.execution_seq = EXECUTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    st.trace.dispatched = observe_dispatched(prog);
    match dispatch(
        &prog.unit_view(),
        limits,
        cancel,
        &[],
        &mut st,
        Some(max_steps),
    ) {
        Some(out) => SteppedOutcome::Finished(out, st.trace),
        None => SteppedOutcome::Paused(PausedRun {
            prog: prog.clone(),
            limits: *limits,
            state: st,
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
    let PausedRun {
        prog,
        limits,
        mut state,
    } = paused;
    // The resumed segment keeps the ORIGINAL chain's dispatch evidence —
    // the pinned snapshot it actually continues to execute — and takes
    // its own sequence number (each entry-point execution is one).
    state.trace.execution_seq =
        EXECUTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    if state.trace.dispatched.is_empty() {
        state.trace.dispatched = observe_dispatched(&prog);
    }
    match dispatch(
        &prog.unit_view(),
        &limits,
        cancel,
        &[],
        &mut state,
        max_steps,
    ) {
        Some(out) => SteppedOutcome::Finished(out, state.trace),
        None => SteppedOutcome::Paused(PausedRun {
            prog,
            limits,
            state,
        }),
    }
}

/// Resumable interpreter state: the live call chain of one top-level run.
#[derive(Default)]
struct LoopState {
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

/// The typed zero / never-read placeholder for a slot type (see
/// [`ModuleView::zero_of`]). The record placeholder carries `ty: u32::MAX`
/// and no fields — distinct from any admitted record value (verification
/// guarantees record slots are written by AGGREGATE/FIELD_SET/Copy/param
/// before any read).
pub(crate) fn typed_zero(t: Option<&MirType>) -> Value {
    match t {
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
    fr.slots[slot] = val;
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
    // Entry poll: a pre-exhausted allowance (or a raised cancel flag)
    // rejects before ANY planning work — deterministic at any graph size
    // and independent of prior calls on this thread.
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
    // pushes children with the just-allocated id). Assemble in reverse id
    // order; a parent consumes its children by scanning the parent map.
    let mut out: Vec<Option<Value>> = vec![None; nodes.len()];
    let mut next_child: Vec<usize> = vec![usize::MAX; nodes.len()];
    // next_child[id] = the SMALLEST node index whose parent == id (they
    // are NOT contiguous — sibling subtrees interleave — so each parent
    // keeps a cursor into a per-parent child list).
    let mut child_list: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for (k, n) in nodes.iter().enumerate() {
        if n.parent != usize::MAX {
            child_list[n.parent].push(k);
        }
    }
    for id in 0..nodes.len() {
        next_child[id] = 0; // cursor into child_list[id]
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
                match arena.try_alloc_list(elems, cap) {
                    Ok(h) => Value::List(h),
                    Err(n) => {
                        return Err(ExecOutcome::Infra(format!(
                            "fault-injected at owned allocation attempt #{n}"
                        )))
                    }
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
    let _ = &mut next_child;
    Ok(out[0].take().expect("root cloned"))
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
            _ => {}
        }
    }
    owned.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, val) in owned {
        st.steps += 1; // reserved cleanup fuel
        drop_owned_reverse(st, val);
    }
}

fn cleanup_owned_frames(st: &mut LoopState) {
    while let Some(mut frame) = st.frames.pop() {
        release_frame_owned_lifo(st, &mut frame);
        drop(frame);
    }
}

fn min_buffer_id(val: &Value) -> u64 {
    // Direct recursion: record nesting is verified acyclic and containers
    // are one layer deep (the frozen domain), so the depth is table-
    // bounded — and the RefCell borrows of nested list buffers cannot
    // outlive their guard in an explicit-stack shape.
    match val {
        Value::Str(h) => h.rc.id,
        Value::List(h) => {
            let buf = h.rc.borrow();
            let mut min = buf.id;
            for e in buf.elems.iter() {
                min = min.min(min_buffer_id(e));
            }
            min
        }
        Value::Variant(var) => match var.payload.as_deref() {
            Some(p) => min_buffer_id(p),
            None => u64::MAX,
        },
        Value::Record(r) => {
            let mut min = u64::MAX;
            let mut stack: Vec<&Value> = r.fields.iter().collect();
            while let Some(x) = stack.pop() {
                min = min.min(min_buffer_id(x));
            }
            min
        }
        _ => u64::MAX,
    }
}

/// F822-R2-03: REVERSE-order responsibility cleanup of an owned value
/// (contract §4 / V-06): list elements release last-pushed-first (the
/// reverse of insertion order — `pop()` from the tail), each element
/// recursively cleaning its own payload (variant payloads included);
/// the container's own buffer frees only after its elements. Scalars and
/// non-container values drop directly. Cleanup fuel is reserved per
/// element (the cancel-point rule).
fn drop_owned_reverse(st: &mut LoopState, val: Value) {
    match val {
        Value::List(h) => {
            // F822-R3-02: a handle clone releasing a SHARED buffer is a
            // responsibility transfer (MOVE/COPY/kill-after-push-in), not
            // the final destruction — draining it here would empty the
            // destination's list. Only the LAST handle reverse-cleans the
            // elements, then the buffer frees.
            if std::rc::Rc::strong_count(&h.rc) == 1 {
                let mut elems = std::mem::take(&mut h.rc.borrow_mut().elems);
                while let Some(e) = elems.pop() {
                    st.steps += 1; // reserved cleanup fuel (cancel point)
                    drop_owned_reverse(st, e);
                }
            }
            drop(h); // the buffer itself frees after its elements
        }
        Value::Variant(var) => {
            if let Some(p) = var.payload.as_ref() {
                st.steps += 1;
                drop_owned_reverse(st, p.as_ref().clone());
            }
        }
        other => drop(other),
    }
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
) -> Option<ExecOutcome> {
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
        let n_params = entry_fn.param_slots().len();
        if n_params != args.len() {
            return Some(ExecOutcome::Infra(format!(
                "entry expects {} args, driver passed {}",
                n_params,
                args.len()
            )));
        }
        // Entry-argument admission against the VERIFIED signature
        // (F798-R6-02): each Value must match its param slot's scalar shape
        // — a Bool handed to an i32 slot (or any cross-kind pair, an
        // Fn/Record-typed slot, or a record value at the entry boundary) is
        // an admission violation, never a silently executing coercion.
        for (i, arg) in args.iter().enumerate() {
            let slot_ty = entry_fn.slots[entry_fn.param_slots()[i] as usize].ty;
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
        if root_slot_bytes > limits.max_slot_bytes {
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
            {
                let fr = st.frames.last_mut().expect("frame present");
                fr.ip += 1;
            }
            let flat = func.block_base(block_idx) + ip as u32;
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
                }
                MirInstr::Clone { dst, src } => {
                    // Explicit deep clone (782): per-field independent
                    // buffers, scalars by value, empty Str zero-alloc.
                    let sv = {
                        let fr = st.frames.last().expect("frame present");
                        fr.slots[*src as usize].clone()
                    };
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
                    // drop happen only after the budget admitted them.
                    {
                        let fr = st.frames.last_mut().expect("frame present");
                        match fr.slots[*root as usize].clone() {
                            Value::List(h) => {
                                let mut buf = h.rc.borrow_mut();
                                let old = std::mem::replace(&mut buf.elems[idx], val.clone());
                                buf.elem_gens[idx] = buf.elem_gens[idx].wrapping_add(1);
                                drop(old);
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    }
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
                    // element's replaced field chain drops here, once.
                    {
                        let fr = st.frames.last_mut().expect("frame present");
                        match fr.slots[*root as usize].clone() {
                            Value::List(h) => {
                                let mut buf = h.rc.borrow_mut();
                                let old = std::mem::replace(&mut buf.elems[idx], new_elem.0);
                                buf.elem_gens[idx] = buf.elem_gens[idx].wrapping_add(1);
                                drop(old);
                            }
                            other => {
                                return Some(ExecOutcome::Infra(format!(
                                    "list operand holds {} (admission violation)",
                                    other.type_name()
                                )))
                            }
                        }
                    }
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
                    // the replacement), drop the old element once.
                    // F822-R5-01: the OWNER frame (by activation sequence)
                    // carries the root — the same frame the precheck just
                    // validated — never the CURRENT frame's same-numbered
                    // slot.
                    {
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
                                drop(old);
                            }
                            _ => unreachable!("precheck validated the owner slot holds the list"),
                        }
                    }
                    if new_deep > old_deep {
                        st.slot_bytes += new_deep - old_deep;
                    } else {
                        st.slot_bytes = st.slot_bytes.saturating_sub(old_deep - new_deep);
                    }
                    st.trace.peak_slot_bytes = st.trace.peak_slot_bytes.max(st.slot_bytes as u64);
                    kill_value_slot(v, func_idx, func, st, *value as usize);
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
}
