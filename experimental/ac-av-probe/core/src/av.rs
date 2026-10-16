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
    MAX_AV_INSTR_STEPS, MAX_AV_SLOT_BYTES, MAX_AV_WALL, MAX_TRACE_BYTES, SLOT_CELL_BYTES,
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
}

impl Default for ExecLimits {
    fn default() -> Self {
        ExecLimits {
            max_steps: MAX_AV_INSTR_STEPS,
            max_frames: MAX_AV_FRAMES,
            max_slot_bytes: MAX_AV_SLOT_BYTES,
            wall: MAX_AV_WALL,
            max_trace_bytes: MAX_TRACE_BYTES,
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

#[derive(Clone, Debug, Default)]
pub struct ExecTrace {
    pub events: Vec<TraceEvent>,
}

struct Frame {
    func: u32,
    block: u32,
    ip: usize,
    slots: Vec<Value>,
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
    let mut trace = ExecTrace::default();
    let started = Instant::now();
    let mut steps: u64 = 0;
    let mut slot_bytes: usize = 0;

    let entry_fn = match m.funcs.get(m.entry as usize) {
        Some(f) => f,
        None => {
            return (
                ExecOutcome::Infra(format!("entry function #{} missing", m.entry)),
                trace,
            );
        }
    };
    let n_params = entry_fn.param_slots().len();
    if n_params != args.len() {
        return (
            ExecOutcome::Infra(format!(
                "entry expects {} args, driver passed {}",
                n_params,
                args.len()
            )),
            trace,
        );
    }
    let zero = typed_zero(m);
    // Root frame.
    let mut frame = Frame {
        func: m.entry,
        block: entry_fn.entry,
        ip: 0,
        slots: Vec::with_capacity(entry_fn.slots.len()),
        ret_dst: None,
    };
    let mut arg_iter = args.iter().copied();
    for s in &entry_fn.slots {
        frame.slots.push(if s.kind == crate::mir::SlotKind::Param {
            arg_iter.next().expect("count checked above")
        } else {
            zero(s.ty)
        });
    }
    slot_bytes += frame.slots.len() * SLOT_CELL_BYTES;
    let mut frames: Vec<Frame> = Vec::with_capacity(16);
    frames.push(frame);

    loop {
        steps += 1;
        if steps > limits.max_steps {
            return (
                ExecOutcome::Resource {
                    kind: ResourceKind::InstrSteps,
                    message: format!("execution exceeded {} instruction steps", limits.max_steps),
                },
                trace,
            );
        }
        if steps % CANCEL_CHECK_INTERVAL == 0 {
            if CancelCheck::poll(cancel) {
                return (ExecOutcome::Cancelled, trace);
            }
            if started.elapsed() > limits.wall {
                return (
                    ExecOutcome::Resource {
                        kind: ResourceKind::ExecutionWall,
                        message: "execution exceeded wall budget".to_string(),
                    },
                    trace,
                );
            }
        }
        // Copy the frame cursor, then re-borrow only for writes: the call
        // arm needs to push/pop `frames` while the current frame is read.
        let (func_idx, block_idx, ip) = {
            let fr = match frames.last() {
                Some(f) => f,
                None => return (ExecOutcome::Infra("frame stack empty".to_string()), trace),
            };
            (fr.func, fr.block, fr.ip)
        };
        let func = match m.funcs.get(func_idx as usize) {
            Some(f) => f,
            None => {
                return (
                    ExecOutcome::Infra(format!("frame func #{} missing", func_idx)),
                    trace,
                )
            }
        };
        let block = match func.blocks.get(block_idx as usize) {
            Some(b) => b,
            None => {
                return (
                    ExecOutcome::Infra(format!("frame block #{} missing", block_idx)),
                    trace,
                )
            }
        };
        if ip < block.instrs.len() {
            let instr = &block.instrs[ip];
            {
                let fr = frames.last_mut().expect("frame present");
                fr.ip += 1;
            }
            let flat = func.block_base(block_idx) + ip as u32;
            match instr {
                MirInstr::Const { dst, value } => {
                    let v = match m.consts[*value as usize] {
                        crate::mir::MirConst::I32(v) => Value::I32(v),
                        crate::mir::MirConst::Bool(b) => Value::Bool(b),
                    };
                    let fr = frames.last_mut().expect("frame present");
                    fr.slots[*dst as usize] = v;
                }
                MirInstr::Copy { dst, src } => {
                    let v = {
                        let fr = frames.last().expect("frame present");
                        fr.slots[*src as usize]
                    };
                    let fr = frames.last_mut().expect("frame present");
                    fr.slots[*dst as usize] = v;
                }
                MirInstr::Bin { op, dst, lhs, rhs } => {
                    let (l, r) = {
                        let fr = frames.last().expect("frame present");
                        (fr.slots[*lhs as usize], fr.slots[*rhs as usize])
                    };
                    match eval_bin(*op, l, r) {
                        Ok(v) => {
                            let fr = frames.last_mut().expect("frame present");
                            fr.slots[*dst as usize] = v;
                        }
                        Err(EvalErr::Trap(kind)) => {
                            return (
                                ExecOutcome::Trap {
                                    kind,
                                    span: span_of(func, flat),
                                },
                                trace,
                            )
                        }
                        Err(EvalErr::Infra(msg)) => return (ExecOutcome::Infra(msg), trace),
                    }
                }
                MirInstr::Un { op, dst, operand } => {
                    let v = {
                        let fr = frames.last().expect("frame present");
                        fr.slots[*operand as usize]
                    };
                    match eval_un(*op, v) {
                        Ok(v) => {
                            let fr = frames.last_mut().expect("frame present");
                            fr.slots[*dst as usize] = v;
                        }
                        Err(EvalErr::Trap(kind)) => {
                            return (
                                ExecOutcome::Trap {
                                    kind,
                                    span: span_of(func, flat),
                                },
                                trace,
                            )
                        }
                        Err(EvalErr::Infra(msg)) => return (ExecOutcome::Infra(msg), trace),
                    }
                }
                MirInstr::Call { callee, dst, args } => {
                    let target = match callee {
                        CalleeRef::Func(idx) => match m.funcs.get(*idx as usize) {
                            Some(_) => *idx,
                            None => {
                                return (
                                    ExecOutcome::Infra(format!("callee #{} missing", idx)),
                                    trace,
                                )
                            }
                        },
                        CalleeRef::Import(idx) => {
                            // v1 executable closures lower test-mark imports
                            // to TRACE instructions at closure assembly; a
                            // CALL on an import is an admission violation.
                            return (
                                ExecOutcome::Infra(format!("call to unresolved import #{}", idx)),
                                trace,
                            );
                        }
                    };
                    if frames.len() >= limits.max_frames {
                        return (
                            ExecOutcome::Resource {
                                kind: ResourceKind::Frames,
                                message: format!("guest frames exceeded {}", limits.max_frames),
                            },
                            trace,
                        );
                    }
                    let callee_fn = &m.funcs[target as usize];
                    let arg_vals: Vec<Value> = {
                        let fr = frames.last().expect("frame present");
                        args.iter().map(|a| fr.slots[*a as usize]).collect()
                    };
                    let mut nf = Frame {
                        func: target,
                        block: callee_fn.entry,
                        ip: 0,
                        slots: Vec::with_capacity(callee_fn.slots.len()),
                        ret_dst: Some(*dst),
                    };
                    let mut ai = arg_vals.into_iter();
                    for s in &callee_fn.slots {
                        nf.slots.push(if s.kind == crate::mir::SlotKind::Param {
                            ai.next().expect("arity verified at admission")
                        } else {
                            zero(s.ty)
                        });
                    }
                    slot_bytes += nf.slots.len() * SLOT_CELL_BYTES;
                    if slot_bytes > limits.max_slot_bytes {
                        return (
                            ExecOutcome::Resource {
                                kind: ResourceKind::SlotBytes,
                                message: format!(
                                    "live slot data exceeded {} bytes",
                                    limits.max_slot_bytes
                                ),
                            },
                            trace,
                        );
                    }
                    frames.push(nf);
                }
                MirInstr::Trace { event } => {
                    trace.events.push(*event);
                    if trace.events.len() > limits.max_trace_bytes {
                        return (
                            ExecOutcome::Resource {
                                kind: ResourceKind::TraceBytes,
                                message: format!(
                                    "test trace exceeded {} bytes",
                                    limits.max_trace_bytes
                                ),
                            },
                            trace,
                        );
                    }
                }
            }
            continue;
        }
        // Terminator.
        match &block.term {
            Terminator::Jump { target } => {
                let fr = frames.last_mut().expect("frame present");
                fr.block = *target;
                fr.ip = 0;
            }
            Terminator::Branch { cond, then, else_ } => {
                let go = {
                    let fr = frames.last().expect("frame present");
                    match fr.slots[*cond as usize] {
                        Value::Bool(b) => b,
                        Value::I32(_) => {
                            return (
                                ExecOutcome::Infra(
                                    "branch condition holds i32 (admission violation)".to_string(),
                                ),
                                trace,
                            )
                        }
                    }
                };
                let fr = frames.last_mut().expect("frame present");
                fr.block = if go { *then } else { *else_ };
                fr.ip = 0;
            }
            Terminator::Return { value } => {
                let v = {
                    let fr = frames.last().expect("frame present");
                    (fr.slots[*value as usize], fr.ret_dst, fr.slots.len())
                };
                let (v, ret_dst, len) = v;
                slot_bytes -= len * SLOT_CELL_BYTES;
                frames.pop();
                match (frames.last_mut(), ret_dst) {
                    (Some(caller), Some(dst)) => {
                        caller.slots[dst as usize] = v;
                    }
                    (Some(_), None) => {}
                    (None, _) => {
                        return (ExecOutcome::Success { value: v, steps }, trace);
                    }
                }
            }
        }
    }
}

fn typed_zero(m: &MirModule) -> impl Fn(u32) -> Value + '_ {
    move |ty| match m.types.get(ty as usize) {
        Some(MirType::Bool) => Value::Bool(false),
        _ => Value::I32(0),
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
