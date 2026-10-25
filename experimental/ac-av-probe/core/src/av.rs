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
    let out = dispatch(&MergedView(m), limits, cancel, args, &mut st, None)
        .expect("no budget: the loop always terminates with an outcome");
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
    st.trace.execution_seq = EXECUTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    st.trace.dispatched = observe_dispatched(prog);
    let out = dispatch(&prog.unit_view(), limits, cancel, &[], &mut st, None)
        .expect("no budget: the loop always terminates with an outcome");
    (out, st.trace)
}

pub fn execute_program(
    prog: &crate::program_snapshot::ProgramSnapshot,
    limits: &ExecLimits,
    cancel: &CancelFlag,
) -> (ExecOutcome, ExecTrace) {
    let mut st = LoopState::default();
    st.trace.execution_seq = EXECUTION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    st.trace.dispatched = observe_dispatched(prog);
    let out = dispatch(&prog.unit_view(), limits, cancel, &[], &mut st, None)
        .expect("no budget: the loop always terminates with an outcome");
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
        self.0.consts.get(idx as usize).copied()
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
                None => false,
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
        // Root frame. PLAN-827 R6: the slot storage goes through the
        // fallible fault-injectable reserve (attempt accounting + ownership
        // lease); PLAN-820 §6 keeps the deep filled-frame budget below.
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
        let mut frame = Frame {
            func: entry_slot,
            block: entry_fn.entry,
            ip: 0,
            slots,
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
                    let val = match cv {
                        crate::mir::MirConst::I32(x) => Value::I32(x),
                        crate::mir::MirConst::Bool(b) => Value::Bool(b),
                        crate::mir::MirConst::I64(x) => Value::I64(x),
                        crate::mir::MirConst::U64(x) => Value::U64(x),
                        // Bit-exact re-materialization: NaN payloads and
                        // signed zeros survive the const round trip.
                        crate::mir::MirConst::F32Bits(bits) => Value::F32(f32::from_bits(bits)),
                        crate::mir::MirConst::F64Bits(bits) => Value::F64(f64::from_bits(bits)),
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
                    let prospective = st.slot_bytes + callee_fn.slots.len() * crate::contract::SLOT_CELL_BYTES;
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
                    let mut nf = Frame {
                        func: target,
                        block: callee_fn.entry,
                        ip: 0,
                        slots,
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
                st.frames.pop();
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
