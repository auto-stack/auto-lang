//! Function-body lowering: the frozen MIR→CLIF instruction table
//! (contract-decision §2; PLAN-870 extends it over the av.data/1 record
//! subset and the av.owned/1 text/text-record subset — freeze §2/§3).
//! MIR blocks map to pre-created, pre-sealed CLIF blocks (the CFG is
//! static and complete); typed mutable slots map to Cranelift variables
//! (I32 physical for i32/bool/char — bool keeps logical 0/1), (I64,I32)
//! pairs for Str/StrView, aggregate stack slots for records, one I64
//! address variable for record views; the FunctionBuilder bridges to
//! SSA. Arithmetic mirrors the landed checked algorithms exactly:
//! explicit pre-target overflow / divide guards branching to the
//! ExitProcess(70) trap block — a hardware divide never sees a zero or
//! −1 divisor, and a trap on an unexecuted branch stays latent (no
//! constant folding into unconditional failures).
//!
//! PLAN-870 ownership discipline over flat native memory (freeze §2):
//! a dead slot is a zeroed pointer. Entry zeroes every owned-capable
//! own-storage slot; every kill (Move/FieldMove out, Aggregate/FieldSet
//! transfer, Str move-out of a param) zeroes the source; `own_free`
//! treats null as an uncounted no-op. Every exit (Return minus the
//! result slot, and every trap branch) sweeps owned slots by
//! LAST-OWNING-WRITE position descending (record fields in declaration
//! order) — the static image of AV's runtime `min_buffer_id` frame
//! sweep. Scalar-only units emit none of this (their objects stay
//! byte-identical to the pre-870 surface).

use crate::emit::event_symbol;
use crate::layout::{self, RecordLayout};
use auto_av_core::contract::{AV_DATA_PROFILE, AV_OWNED_PROFILE};
use auto_av_core::mir::*;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{
    types, AbiParam, FuncRef, InstBuilder, MemFlags, Signature, StackSlot, StackSlotData, TrapCode,
    Value,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{Linkage, Module};
use std::collections::BTreeMap;

/// Emit the single function of `unit` into `ctx.func`, declaring callees
/// in `om`. The caller owns `ctx` and performs `define_function` after
/// this returns. Errors are backend-lowering messages (wrapped by the
/// caller as `mn.backend.lowering`).
#[allow(clippy::too_many_arguments)]
pub fn emit_function(
    unit: &MirModule,
    om: &mut cranelift_object::ObjectModule,
    _fid: cranelift_module::FuncId,
    import_ids: &[cranelift_module::FuncId],
    trace_ids: &[(String, cranelift_module::FuncId)],
    exit_id: cranelift_module::FuncId,
    ctx: &mut cranelift_codegen::Context,
) -> Result<(), String> {
    let f = &unit.funcs[0];

    let mut fb_ctx = FunctionBuilderContext::new();
    let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);

    // Block creation order: the MIR entry block FIRST (it becomes the
    // cranelift function entry and receives the signature params), the
    // rest in MIR order.
    let n_blocks = f.blocks.len();
    let mut order: Vec<usize> = Vec::with_capacity(n_blocks);
    order.push(f.entry as usize);
    for i in 0..n_blocks {
        if i != f.entry as usize {
            order.push(i);
        }
    }
    let mut blocks: Vec<Option<cranelift_codegen::ir::Block>> =
        (0..n_blocks).map(|_| None).collect();
    for i in &order {
        blocks[*i] = Some(fb.create_block());
    }
    let blocks: Vec<cranelift_codegen::ir::Block> =
        blocks.into_iter().map(|b| b.expect("created")).collect();

    // Predecessor counts from the MIR CFG (a Branch with both arms on the
    // same block counts once — emit_term normalizes it to a Jump). Blocks
    // are sealed LAZILY: sealing asserts that no further predecessor will
    // be declared, so a target is sealed exactly when its last predecessor
    // terminator has been emitted (back edges stay unsealed until the
    // latch, which is the deferred-use case the SSA builder supports).
    let mut remaining: Vec<u32> = vec![0; n_blocks];
    for b in &f.blocks {
        match &b.term {
            Terminator::Jump { target } => remaining[*target as usize] += 1,
            Terminator::Branch { then, else_, .. } => {
                remaining[*then as usize] += 1;
                if else_ != then {
                    remaining[*else_ as usize] += 1;
                }
            }
            Terminator::Return { .. } => {}
        }
    }
    let mut sealed = vec![false; n_blocks];

    // ---- slot representation plan (PLAN-870 freeze §2) ------------------
    let owned_domain = unit.profile == AV_OWNED_PROFILE;
    let mut layouts: BTreeMap<u32, RecordLayout> = BTreeMap::new();
    let mut str_paths: BTreeMap<u32, Vec<Vec<u16>>> = BTreeMap::new();
    let mut reps: Vec<SlotRep> = Vec::with_capacity(f.slots.len());
    for (si, s) in f.slots.iter().enumerate() {
        let rep = match unit.types.get(s.ty as usize) {
            Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Char) => {
                SlotRep::Scalar(fb.declare_var(types::I32))
            }
            Some(MirType::Str) | Some(MirType::StrView) => SlotRep::Pair {
                ptr: fb.declare_var(types::I64),
                len: fb.declare_var(types::I32),
            },
            Some(MirType::Record(r)) => {
                let l = layout::record_layout(unit, s.ty, r, &mut layouts)
                    .map_err(|e| format!("slot {si}: {e:?}"))?;
                let paths =
                    layout::str_field_paths(unit, s.ty).map_err(|e| format!("slot {si}: {e:?}"))?;
                str_paths.insert(s.ty, paths);
                if s.kind == SlotKind::Param && owned_domain {
                    // Owned-domain record param: the CALLER's memory,
                    // addressed through the incoming pointer (write-through
                    // + zero-through discipline, freeze §3).
                    SlotRep::Agg {
                        size: l.size,
                        slot: None,
                        incoming: Some(fb.declare_var(types::I64)),
                    }
                } else {
                    // Own storage: locals, temps, and data-domain params
                    // (the latter entry-copy into the private slot).
                    let ss = fb.create_sized_stack_slot(StackSlotData::new(
                        cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                        l.size.max(1) as u32,
                        l.align_log2 as u8,
                    ));
                    SlotRep::Agg {
                        size: l.size,
                        slot: Some(ss),
                        incoming: None,
                    }
                }
            }
            Some(MirType::RecordView(_)) => SlotRep::View(fb.declare_var(types::I64)),
            Some(MirType::MutRef) => SlotRep::Token,
            Some(
                MirType::I64
                | MirType::U64
                | MirType::F32
                | MirType::F64
                | MirType::Fn(_)
                | MirType::List(_)
                | MirType::Variant(_)
                | MirType::Place(_)
                | MirType::Closure(_)
                | MirType::Dyn,
            ) => {
                return Err(format!(
                    "slot {si} type {} is outside the mn data/owned surface",
                    s.ty
                ))
            }
            None => return Err(format!("slot {si} type {} is missing", s.ty)),
        };
        reps.push(rep);
    }

    // The sweep set: slots that may own buffers at an exit, ordered by
    // last-owning-write position DESC (ties by slot id DESC) — the static
    // image of AV's descending `min_buffer_id` frame sweep.
    let sweep = sweep_order(unit, f, &str_paths, owned_domain);

    let mut fx = FnLowerer {
        fb: &mut fb,
        om,
        unit,
        exit_id,
        reps,
        layouts,
        str_paths,
        sweep,
        sweep_live: false,
        out_agg: None,
        import_refs: BTreeMap::new(),
        trace_refs: BTreeMap::new(),
        rt_refs: BTreeMap::new(),
        lit_ids: BTreeMap::new(),
        outlen_slot: None,
        trap_block: None,
        blocks,
    };
    fx.sweep_live = !fx.sweep.is_empty();
    let _ = AV_DATA_PROFILE; // profile intent documented above; admission lives in emit.rs

    // Function refs for imports.
    for (i, id) in import_ids.iter().enumerate() {
        let r = fx.om.declare_func_in_func(*id, fx.fb.func);
        fx.import_refs.insert(i as u32, r);
    }
    for (sym, id) in trace_ids {
        let r = fx.om.declare_func_in_func(*id, fx.fb.func);
        fx.trace_refs.insert(sym.clone(), r);
    }

    // Entry block: signature params in order define the param slots
    // (verify guarantees param slots come first, in signature order). A
    // block with no remaining predecessors (the common entry shape) seals
    // right away; a looped entry stays open until its latch emits.
    let entry_block = fx.blocks[f.entry as usize];
    fx.fb.switch_to_block(entry_block);
    fx.fb.append_block_params_for_function_params(entry_block);
    let n_params = fx.fb.func.signature.params.len();
    let params: Vec<_> = (0..n_params)
        .map(|i| fx.fb.block_params(entry_block)[i])
        .collect();
    {
        // Entry zeroing FIRST (freeze §2): every own-storage owned-capable
        // slot starts as a zeroed pointer; machine params then overwrite
        // the param slots. Scalar-only units emit nothing here.
        fx.emit_entry_zeroing()?;
        let mut pi = 0usize;
        for (si, s) in f.slots.iter().enumerate() {
            if s.kind != SlotKind::Param {
                continue;
            }
            match fx.reps[si] {
                SlotRep::Scalar(v) => {
                    let pv = params[pi];
                    pi += 1;
                    fx.fb.def_var(v, pv);
                }
                SlotRep::Pair { ptr, len } => {
                    let p0 = params[pi];
                    let p1 = params[pi + 1];
                    pi += 2;
                    fx.fb.def_var(ptr, p0);
                    fx.fb.def_var(len, p1);
                }
                SlotRep::Agg {
                    incoming: Some(v), ..
                } => {
                    let pv = params[pi];
                    pi += 1;
                    fx.fb.def_var(v, pv);
                }
                SlotRep::Agg { size, slot, .. } => {
                    // Data domain: the param gets a PRIVATE copy of the
                    // caller's aggregate (AV shares params into callee
                    // frame slots — callee writes stay private).
                    let pv = params[pi];
                    pi += 1;
                    let ss = slot.expect("own-storage aggregate has a stack slot");
                    let dst = fx.fb.ins().stack_addr(types::I64, ss, 0);
                    let ty = s.ty;
                    fx.copy_agg_bytes(dst, pv, size, ty);
                }
                SlotRep::View(v) => {
                    let pv = params[pi];
                    pi += 1;
                    fx.fb.def_var(v, pv);
                }
                SlotRep::Token => {}
            }
        }
        // The trailing hidden out-aggregate parameter (record result).
        if pi < params.len() {
            fx.out_agg = Some(params[pi]);
        }
    }

    // Body: blocks in MIR order; seal a block only when ALL its
    // predecessors have declared their edges — a block still expecting a
    // back edge (a loop header whose latch emits later) stays UNSEALED
    // while being filled; the SSA builder resolves its uses at seal time.
    // The prologue left us ON the entry block (params bound, zeroing
    // emitted) — re-switching to a block that is already current but
    // unfilled trips the frontend's fill discipline once the entry block
    // carries instructions (owned zeroing). Blocks are emitted in MIR
    // order with the entry first.
    debug_assert_eq!(
        f.entry, 0,
        "the delivered adapter emits entry-first functions"
    );
    let mut on_entry = true;
    for (bi, b) in f.blocks.iter().enumerate() {
        if !sealed[bi] && remaining[bi] == 0 {
            fx.fb.seal_block(fx.blocks[bi]);
            sealed[bi] = true;
        }
        if on_entry && bi == f.entry as usize {
            on_entry = false; // keep filling the entry block
        } else {
            fx.fb.switch_to_block(fx.blocks[bi]);
        }
        for instr in &b.instrs {
            fx.emit_instr(instr)
                .map_err(|e| format!("`{}`: {e}", f.name))?;
        }
        fx.emit_term(&b.term)
            .map_err(|e| format!("`{}`: {e}", f.name))?;
        // The emitted terminator declared predecessors on its targets.
        let touch = |fx: &mut FnLowerer, t: u32, sealed: &mut Vec<bool>, remaining: &mut [u32]| {
            let ti = t as usize;
            remaining[ti] -= 1;
            if remaining[ti] == 0 && !sealed[ti] {
                fx.fb.seal_block(fx.blocks[ti]);
                sealed[ti] = true;
            }
        };
        match &b.term {
            Terminator::Jump { target } => {
                touch(&mut fx, *target, &mut sealed, &mut remaining);
            }
            Terminator::Branch { then, else_, .. } => {
                touch(&mut fx, *then, &mut sealed, &mut remaining);
                if else_ != then {
                    touch(&mut fx, *else_, &mut sealed, &mut remaining);
                }
            }
            Terminator::Return { .. } => {}
        }
    }
    // Defensive: every block's predecessors are declared by now (all
    // blocks were emitted) — seal anything left (a block no terminator
    // targets and that was skipped, e.g. the entry when it has no preds).
    for bi in 0..n_blocks {
        if !sealed[bi] {
            fx.fb.seal_block(fx.blocks[bi]);
        }
    }

    // Trap block (only when some guard referenced it): the all-exit sweep
    // already ran at each trap BRANCH site (freeze §2); the block itself
    // is ExitProcess(70) followed by an unreachable trap.
    if let Some(trap) = fx.trap_block.take() {
        fx.fb.switch_to_block(trap);
        fx.fb.seal_block(trap);
        let code = fx
            .fb
            .ins()
            .iconst(types::I32, crate::config::TRAP_EXIT_CODE as i64);
        let exit_ref = fx.om.declare_func_in_func(fx.exit_id, fx.fb.func);
        fx.fb.ins().call(exit_ref, &[code]);
        fx.fb.ins().trap(TrapCode::INTEGER_OVERFLOW); // never reached
    }
    drop(fx);
    fb.finalize();
    Ok(())
}

/// How one MIR slot lives in the emitted function.
#[derive(Clone, Copy, Debug)]
enum SlotRep {
    /// I32/Bool/Char — one I32 variable.
    Scalar(Variable),
    /// Str/StrView — (ptr, len_bytes) pair.
    Pair { ptr: Variable, len: Variable },
    /// Record — aggregate storage. `incoming` set: an owned-domain param
    /// addressed THROUGH the caller's memory; `slot` set: a private stack
    /// slot (locals, temps, data-domain params).
    Agg {
        size: u64,
        slot: Option<StackSlot>,
        incoming: Option<Variable>,
    },
    /// RecordView — one I64 address variable.
    View(Variable),
    /// MutRef — no runtime representation (verify-time exclusivity).
    Token,
}

/// The precomputed exit-sweep order (freeze §2): owned-capable slots by
/// last-owning-write position descending, ties by slot id descending.
/// Owned-domain record PARAMS are excluded (their memory is the caller's
/// move temp — the caller sweep owns it).
fn sweep_order(
    unit: &MirModule,
    f: &MirFunction,
    str_paths: &BTreeMap<u32, Vec<Vec<u16>>>,
    owned_domain: bool,
) -> Vec<u32> {
    // Owned-capable: a Str slot, or a record slot with ≥1 Str field path.
    let capable = |si: usize| -> bool {
        let ty = f.slots[si].ty;
        match unit.types.get(ty as usize) {
            Some(MirType::Str) => true,
            Some(MirType::Record(_)) => str_paths.get(&ty).map(|p| !p.is_empty()).unwrap_or(false),
            _ => false,
        }
    };
    let mut last_write: Vec<Option<(usize, usize)>> = vec![None; f.slots.len()];
    for (bi, b) in f.blocks.iter().enumerate() {
        for (ii, ins) in b.instrs.iter().enumerate() {
            let dsts: Vec<u32> = match ins {
                MirInstr::Const { dst, .. }
                | MirInstr::Clone { dst, .. }
                | MirInstr::Move { dst, .. }
                | MirInstr::FieldMove { dst, .. }
                | MirInstr::StrConcat { dst, .. }
                | MirInstr::StrToOwned { dst, .. }
                | MirInstr::Aggregate { dst, .. }
                | MirInstr::Call { dst, .. } => vec![*dst],
                MirInstr::FieldSet { root, .. } => vec![*root],
                _ => vec![],
            };
            for d in dsts {
                if capable(d as usize) {
                    last_write[d as usize] = Some((bi, ii));
                }
            }
        }
    }
    let mut ids: Vec<u32> = (0..f.slots.len() as u32)
        .filter(|&si| {
            capable(si as usize)
                && !(owned_domain
                    && f.slots[si as usize].kind == SlotKind::Param
                    && matches!(
                        unit.types.get(f.slots[si as usize].ty as usize),
                        Some(MirType::Record(_))
                    ))
        })
        .collect();
    ids.sort_by(|a, b| {
        let ka = (last_write[*a as usize], *a);
        let kb = (last_write[*b as usize], *b);
        kb.cmp(&ka)
    });
    ids
}

struct FnLowerer<'a, 'b> {
    fb: &'a mut FunctionBuilder<'b>,
    om: &'a mut cranelift_object::ObjectModule,
    unit: &'a MirModule,
    exit_id: cranelift_module::FuncId,
    reps: Vec<SlotRep>,
    layouts: BTreeMap<u32, RecordLayout>,
    str_paths: BTreeMap<u32, Vec<Vec<u16>>>,
    /// Ordered sweep list; empty for scalar-only functions.
    sweep: Vec<u32>,
    sweep_live: bool,
    /// Trailing hidden out-aggregate address (record-result functions).
    out_agg: Option<Value>,
    import_refs: BTreeMap<u32, FuncRef>,
    trace_refs: BTreeMap<String, FuncRef>,
    /// ac_rt_* runtime function refs (owned units only).
    rt_refs: BTreeMap<&'static str, FuncRef>,
    /// Static literal data objects per Str const index.
    lit_ids: BTreeMap<u32, cranelift_module::DataId>,
    /// One shared 4-byte ABI-C out-length slot per function.
    outlen_slot: Option<StackSlot>,
    trap_block: Option<cranelift_codegen::ir::Block>,
    blocks: Vec<cranelift_codegen::ir::Block>,
}

impl<'a, 'b> FnLowerer<'a, 'b> {
    fn slot_ty(&self, slot: u32) -> u32 {
        self.unit.funcs[0].slots[slot as usize].ty
    }

    fn mir_type(&self, ty: u32) -> Result<&MirType, String> {
        self.unit
            .types
            .get(ty as usize)
            .ok_or_else(|| format!("type {ty} missing"))
    }

    /// An instruction-embedded record type index that dangles in the
    /// unit-local table resolves through the destination slot's type —
    /// verify guarantees the two are the same nominal record.
    fn resolve_record_ty(&self, record_ty: u32, dst: u32) -> Result<u32, String> {
        match self.unit.types.get(record_ty as usize) {
            Some(MirType::Record(_)) => Ok(record_ty),
            _ => {
                let ty = self.slot_ty(dst);
                match self.unit.types.get(ty as usize) {
                    Some(MirType::Record(_)) => Ok(ty),
                    _ => Err(format!(
                        "AGGREGATE record type {record_ty} dangles and dst slot {dst} (type {ty}) is not a record"
                    )),
                }
            }
        }
    }

    fn v_scalar(&self, slot: u32) -> Result<Variable, String> {
        match self.reps[slot as usize] {
            SlotRep::Scalar(v) => Ok(v),
            _ => Err(format!("slot {slot} is not scalar")),
        }
    }

    fn v_pair(&self, slot: u32) -> Result<(Variable, Variable), String> {
        match self.reps[slot as usize] {
            SlotRep::Pair { ptr, len } => Ok((ptr, len)),
            _ => Err(format!("slot {slot} is not a text pair")),
        }
    }

    fn def_scalar(&mut self, slot: u32, val: Value) -> Result<(), String> {
        let v = self.v_scalar(slot)?;
        self.fb.def_var(v, val);
        Ok(())
    }

    fn use_ptr(&mut self, slot: u32) -> Result<Value, String> {
        let (p, _) = self.v_pair(slot)?;
        Ok(self.fb.use_var(p))
    }

    fn use_len(&mut self, slot: u32) -> Result<Value, String> {
        let (_, l) = self.v_pair(slot)?;
        Ok(self.fb.use_var(l))
    }

    fn def_pair(&mut self, slot: u32, ptr: Value, len: Value) -> Result<(), String> {
        let (p, l) = self.v_pair(slot)?;
        self.fb.def_var(p, ptr);
        self.fb.def_var(l, len);
        Ok(())
    }

    fn zero_pair(&mut self, slot: u32) -> Result<(), String> {
        let z = self.fb.ins().iconst(types::I64, 0);
        let zl = self.fb.ins().iconst(types::I32, 0);
        self.def_pair(slot, z, zl)
    }

    /// The memory address a record slot lives at (own stack slot or the
    /// incoming caller pointer).
    fn agg_addr(&mut self, slot: u32) -> Result<Value, String> {
        match self.reps[slot as usize] {
            SlotRep::Agg { slot: Some(ss), .. } => Ok(self.fb.ins().stack_addr(types::I64, ss, 0)),
            SlotRep::Agg {
                incoming: Some(v), ..
            } => Ok(self.fb.use_var(v)),
            _ => Err(format!("slot {slot} is not an aggregate")),
        }
    }

    /// The memory address a record or record-view slot is READ at.
    fn view_addr(&mut self, slot: u32) -> Result<Value, String> {
        match self.reps[slot as usize] {
            SlotRep::View(v) => Ok(self.fb.use_var(v)),
            _ => self.agg_addr(slot),
        }
    }

    fn paths_of(&self, ty: u32) -> Vec<Vec<u16>> {
        self.str_paths.get(&ty).cloned().unwrap_or_default()
    }

    fn path_offset_of(&mut self, base_ty: u32, path: &[u16]) -> Result<u64, String> {
        layout::path_offset(self.unit, base_ty, path, &mut self.layouts)
            .map_err(|e| format!("path {path:?}: {e:?}"))
    }

    fn path_ty_of(&self, base_ty: u32, path: &[u16]) -> Result<u32, String> {
        layout::path_ty(self.unit, base_ty, path).map_err(|e| format!("path {path:?}: {e:?}"))
    }

    fn record_size_of(&mut self, ty: u32) -> Result<u64, String> {
        let r = match self.mir_type(ty)? {
            MirType::Record(r) | MirType::RecordView(r) => r.clone(),
            _ => return Err(format!("type {ty} is not a record")),
        };
        let l = layout::record_layout(self.unit, ty, &r, &mut self.layouts)
            .map_err(|e| format!("layout of {ty}: {e:?}"))?;
        Ok(l.size)
    }

    fn emit_entry_zeroing(&mut self) -> Result<(), String> {
        if !self.sweep_live {
            return Ok(());
        }
        let n = self.unit.funcs[0].slots.len();
        for si in 0..n as u32 {
            let ty = self.slot_ty(si);
            match self.unit.types.get(ty as usize) {
                // StrView slots never free — only owned Str pairs zero.
                Some(MirType::Str) => {
                    self.zero_pair(si)?;
                }
                Some(MirType::Record(_)) => {
                    if matches!(
                        self.reps[si as usize],
                        SlotRep::Agg {
                            incoming: Some(_),
                            ..
                        }
                    ) {
                        continue; // caller memory — never zeroed here
                    }
                    let paths = self.paths_of(ty);
                    if paths.is_empty() {
                        continue;
                    }
                    let addr = self.agg_addr(si)?;
                    let zero = self.fb.ins().iconst(types::I64, 0);
                    let flags = MemFlags::trusted();
                    for p in &paths {
                        let off = self.path_offset_of(ty, p)? as i32;
                        self.fb.ins().store(flags, zero, addr, off);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Signed compare producing the profile's bool representation
    /// (i32 0/1): `icmp` yields i8 and MUST be widened before flowing
    /// into a variable, branch, call argument or return.
    fn icmp_bool(
        &mut self,
        cc: IntCC,
        a: cranelift_codegen::ir::Value,
        b: cranelift_codegen::ir::Value,
    ) -> cranelift_codegen::ir::Value {
        let c = self.fb.ins().icmp(cc, a, b);
        self.fb.ins().uextend(types::I32, c)
    }

    fn ensure_trap_block(&mut self) -> cranelift_codegen::ir::Block {
        *self
            .trap_block
            .get_or_insert_with(|| self.fb.create_block())
    }

    /// The all-exit sweep (freeze §2), emitted at every trap branch and
    /// at Return (minus the escaping result slot). Null pointers are
    /// no-op frees at the runtime boundary; scalar-only functions emit
    /// nothing.
    fn emit_exit_sweep(&mut self, exclude: Option<u32>) -> Result<(), String> {
        if !self.sweep_live {
            return Ok(());
        }
        for si in self.sweep.clone() {
            if Some(si) == exclude {
                continue;
            }
            let ty = self.slot_ty(si);
            match self.unit.types.get(ty as usize) {
                Some(MirType::Str) => {
                    let p = self.use_ptr(si)?;
                    self.rt_free(p);
                }
                Some(MirType::Record(_)) => {
                    let addr = self.agg_addr(si)?;
                    let flags = MemFlags::trusted();
                    for p in self.paths_of(ty) {
                        let off = self.path_offset_of(ty, &p)? as i32;
                        let bp = self.fb.ins().load(types::I64, flags, addr, off);
                        self.rt_free(bp);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// `brif(cond → [sweep] trap70, else cont)`; the continuation is
    /// sealed with its single predecessor and becomes the current block.
    /// The all-exit sweep runs ONLY on the trap arm (freeze §2): a guard
    /// that does not fire must leave every buffer live. Scalar-only
    /// functions (empty sweep) keep the pre-870 two-block shape — their
    /// objects stay byte-identical.
    fn branch_to_trap_on(&mut self, cond: cranelift_codegen::ir::Value) {
        let trap = self.ensure_trap_block();
        let cont = self.fb.create_block();
        if !self.sweep_live {
            self.fb.ins().brif(cond, trap, &[], cont, &[]);
            self.fb.seal_block(cont);
            self.fb.switch_to_block(cont);
            return;
        }
        let pre = self.fb.create_block();
        self.fb.ins().brif(cond, pre, &[], cont, &[]);
        self.fb.seal_block(cont);
        self.fb.seal_block(pre);
        self.fb.switch_to_block(pre);
        let _ = self.emit_exit_sweep(None);
        self.fb.ins().jump(trap, &[]);
        self.fb.switch_to_block(cont);
    }

    fn checked_add(
        &mut self,
        a: cranelift_codegen::ir::Value,
        b: cranelift_codegen::ir::Value,
    ) -> cranelift_codegen::ir::Value {
        let sum = self.fb.ins().iadd(a, b);
        let zero = self.fb.ins().iconst(types::I32, 0);
        let max = self.fb.ins().iconst(types::I32, i32::MAX as i64);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let room_pos = self.fb.ins().isub(max, b);
        let b_nonneg = self.icmp_bool(IntCC::SignedGreaterThanOrEqual, b, zero);
        let a_gt_room = self.icmp_bool(IntCC::SignedGreaterThan, a, room_pos);
        let ovf_pos = self.fb.ins().band(b_nonneg, a_gt_room);
        let room_neg = self.fb.ins().isub(min, b);
        let b_neg = self.icmp_bool(IntCC::SignedLessThan, b, zero);
        let a_lt_room = self.icmp_bool(IntCC::SignedLessThan, a, room_neg);
        let ovf_neg = self.fb.ins().band(b_neg, a_lt_room);
        let ovf = self.fb.ins().bor(ovf_pos, ovf_neg);
        self.branch_to_trap_on(ovf);
        sum
    }

    fn checked_sub(
        &mut self,
        a: cranelift_codegen::ir::Value,
        b: cranelift_codegen::ir::Value,
    ) -> cranelift_codegen::ir::Value {
        let diff = self.fb.ins().isub(a, b);
        let zero = self.fb.ins().iconst(types::I32, 0);
        let max = self.fb.ins().iconst(types::I32, i32::MAX as i64);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let room_pos = self.fb.ins().iadd(max, b);
        let b_nonpos = self.icmp_bool(IntCC::SignedLessThanOrEqual, b, zero);
        let a_gt_room = self.icmp_bool(IntCC::SignedGreaterThan, a, room_pos);
        let ovf_pos = self.fb.ins().band(b_nonpos, a_gt_room);
        let room_neg = self.fb.ins().iadd(min, b);
        let b_pos = self.icmp_bool(IntCC::SignedGreaterThan, b, zero);
        let a_lt_room = self.icmp_bool(IntCC::SignedLessThan, a, room_neg);
        let ovf_neg = self.fb.ins().band(b_pos, a_lt_room);
        let ovf = self.fb.ins().bor(ovf_pos, ovf_neg);
        self.branch_to_trap_on(ovf);
        diff
    }

    fn checked_mul(
        &mut self,
        a: cranelift_codegen::ir::Value,
        b: cranelift_codegen::ir::Value,
    ) -> cranelift_codegen::ir::Value {
        let a64 = self.fb.ins().sextend(types::I64, a);
        let b64 = self.fb.ins().sextend(types::I64, b);
        let r64 = self.fb.ins().imul(a64, b64);
        let r32 = self.fb.ins().ireduce(types::I32, r64);
        let back = self.fb.ins().sextend(types::I64, r32);
        let ovf = self.icmp_bool(IntCC::NotEqual, r64, back);
        self.branch_to_trap_on(ovf);
        r32
    }

    fn checked_neg(&mut self, a: cranelift_codegen::ir::Value) -> cranelift_codegen::ir::Value {
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let is_min = self.icmp_bool(IntCC::Equal, a, min);
        self.branch_to_trap_on(is_min);
        self.fb.ins().ineg(a)
    }

    /// `a / b` with pre-target guards: the divisor is sanitized so the
    /// hardware `sdiv` never sees 0 or −1; the exact quotient is selected
    /// back; the fixed trap rules (b == 0, MIN / −1) branch to trap70.
    fn guarded_sdiv(
        &mut self,
        a: cranelift_codegen::ir::Value,
        b: cranelift_codegen::ir::Value,
    ) -> cranelift_codegen::ir::Value {
        let zero = self.fb.ins().iconst(types::I32, 0);
        let one = self.fb.ins().iconst(types::I32, 1);
        let m1 = self.fb.ins().iconst(types::I32, -1);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let bz = self.icmp_bool(IntCC::Equal, b, zero);
        let safe_bz = self.fb.ins().select(bz, one, b);
        let bm1 = self.icmp_bool(IntCC::Equal, b, m1);
        let safe = self.fb.ins().select(bm1, one, safe_bz);
        let raw = self.fb.ins().sdiv(a, safe);
        let neg = self.fb.ins().ineg(a);
        let q = self.fb.ins().select(bm1, neg, raw);
        let a_min = self.icmp_bool(IntCC::Equal, a, min);
        let m1_min = self.fb.ins().band(bm1, a_min);
        let bad = self.fb.ins().bor(bz, m1_min);
        self.branch_to_trap_on(bad);
        q
    }

    /// `a % b` with the same pre-target guards; MIN % −1 traps per the
    /// scalar contract (unlike the host `%`).
    fn guarded_srem(
        &mut self,
        a: cranelift_codegen::ir::Value,
        b: cranelift_codegen::ir::Value,
    ) -> cranelift_codegen::ir::Value {
        let zero = self.fb.ins().iconst(types::I32, 0);
        let one = self.fb.ins().iconst(types::I32, 1);
        let m1 = self.fb.ins().iconst(types::I32, -1);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let bz = self.icmp_bool(IntCC::Equal, b, zero);
        let safe_bz = self.fb.ins().select(bz, one, b);
        let bm1 = self.icmp_bool(IntCC::Equal, b, m1);
        let safe = self.fb.ins().select(bm1, one, safe_bz);
        let raw = self.fb.ins().srem(a, safe);
        // x % -1 == 0 in exact arithmetic; only the non-MIN runs select it.
        let z = self.fb.ins().iconst(types::I32, 0);
        let r = self.fb.ins().select(bm1, z, raw);
        let a_min = self.icmp_bool(IntCC::Equal, a, min);
        let m1_min = self.fb.ins().band(bm1, a_min);
        let bad = self.fb.ins().bor(bz, m1_min);
        self.branch_to_trap_on(bad);
        r
    }

    fn trace_ref(&mut self, symbol: &str) -> Result<FuncRef, String> {
        if let Some(r) = self.trace_refs.get(symbol) {
            return Ok(*r);
        }
        if let Some(pos) = self.unit.imports.iter().position(|i| i.symbol == symbol) {
            if let Some(r) = self.import_refs.get(&(pos as u32)) {
                return Ok(*r);
            }
        }
        // Not in any table (bare Trace without a declared import): declare
        // the runtime symbol on demand — Linkage::Import, resolved by the
        // link inputs from the landed trace support library.
        let sig = trace_call_sig();
        let id = self
            .om
            .declare_function(symbol, Linkage::Import, &sig)
            .map_err(|e| format!("declare {symbol}: {e}"))?;
        let r = self.om.declare_func_in_func(id, self.fb.func);
        self.trace_refs.insert(symbol.to_string(), r);
        Ok(r)
    }

    // ---- PLAN-870 runtime (ac_text_runtime / ac.runtime.text.v1) ------

    /// The static address of one Str const's bytes (fresh `symbol_value`
    /// per use — a cached SSA value must not cross blocks).
    fn lit_addr(&mut self, const_idx: u32) -> Result<Value, String> {
        let id = match self.lit_ids.get(&const_idx) {
            Some(id) => *id,
            None => {
                let bytes = match self.unit.consts.get(const_idx as usize) {
                    Some(MirConst::Str(b)) => b.clone(),
                    _ => return Err(format!("const {const_idx} is not a Str literal")),
                };
                let name = format!("mn8_lit_c{const_idx}");
                let id = self
                    .om
                    .declare_data(&name, Linkage::Local, false, false)
                    .map_err(|e| format!("declare literal {name}: {e}"))?;
                let mut desc = cranelift_module::DataDescription::new();
                desc.define(bytes.to_vec().into_boxed_slice());
                desc.set_align(1);
                self.om
                    .define_data(id, &desc)
                    .map_err(|e| format!("define literal {name}: {e}"))?;
                self.lit_ids.insert(const_idx, id);
                id
            }
        };
        let gv = self.om.declare_data_in_func(id, self.fb.func);
        Ok(self.fb.ins().symbol_value(types::I64, gv))
    }

    fn ensure_outlen(&mut self) -> StackSlot {
        *self.outlen_slot.get_or_insert_with(|| {
            self.fb.create_sized_stack_slot(StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                4,
                2,
            ))
        })
    }

    /// Declare (once) and resolve one ac_rt_* runtime function.
    fn rt_ref(
        &mut self,
        sym: &'static str,
        params: &[types::Type],
        rets: &[types::Type],
    ) -> Result<FuncRef, String> {
        if let Some(r) = self.rt_refs.get(sym) {
            return Ok(*r);
        }
        let sig = Signature {
            call_conv: cranelift_codegen::isa::CallConv::WindowsFastcall,
            params: params.iter().map(|t| AbiParam::new(*t)).collect(),
            returns: rets.iter().map(|t| AbiParam::new(*t)).collect(),
        };
        let id = self
            .om
            .declare_function(sym, Linkage::Import, &sig)
            .map_err(|e| format!("declare {sym}: {e}"))?;
        let r = self.om.declare_func_in_func(id, self.fb.func);
        self.rt_refs.insert(sym, r);
        Ok(r)
    }

    fn rt_free(&mut self, ptr: Value) {
        let r = self
            .rt_ref("ac_rt_own_free", &[types::I64], &[])
            .expect("own_free declaration cannot fail");
        self.fb.ins().call(r, &[ptr]);
    }

    /// ABI-C owned-buffer-out call: trailing `*mut i32` length/status
    /// slot; returns (ptr, len). A negative len is the caller's trap
    /// (`negative_guard`).
    fn abic_call(
        &mut self,
        sym: &'static str,
        params: &[types::Type],
        args: &[Value],
    ) -> Result<(Value, Value), String> {
        // The frozen ABI-C shape carries the trailing `*mut i32` out
        // channel as a REAL parameter (the shim's declared signature).
        let mut sig_params = params.to_vec();
        sig_params.push(types::I64);
        let r = self.rt_ref(sym, &sig_params, &[types::I64])?;
        let slot = self.ensure_outlen();
        let out_addr = self.fb.ins().stack_addr(types::I64, slot, 0);
        let mut argv = args.to_vec();
        argv.push(out_addr);
        let call = self.fb.ins().call(r, &argv);
        let ptr = self.fb.inst_results(call)[0];
        let len = self
            .fb
            .ins()
            .load(types::I32, MemFlags::trusted(), out_addr, 0);
        Ok((ptr, len))
    }

    /// Negative i32 → full-exit trap (a runtime text failure is trap70
    /// per freeze §5; the sweep at the branch frees partial state).
    fn negative_guard(&mut self, v: Value) -> Result<(), String> {
        let zero = self.fb.ins().iconst(types::I32, 0);
        let neg = self.icmp_bool(IntCC::SignedLessThan, v, zero);
        self.branch_to_trap_on(neg);
        Ok(())
    }

    fn reject<T>(&self, code: &str, msg: &str) -> Result<T, String> {
        Err(format!("{code}: {msg}"))
    }

    fn emit_instr(&mut self, i: &MirInstr) -> Result<(), String> {
        match i {
            MirInstr::Const { dst, value } => {
                let ty = self.slot_ty(*dst);
                match self.unit.consts.get(*value as usize) {
                    Some(MirConst::I32(v)) => {
                        let val = self.fb.ins().iconst(types::I32, *v as i64);
                        self.def_scalar(*dst, val)?;
                    }
                    Some(MirConst::Bool(b)) => {
                        let val = self.fb.ins().iconst(types::I32, if *b { 1 } else { 0 });
                        self.def_scalar(*dst, val)?;
                    }
                    // PLAN-870: Char crosses as its Unicode scalar value.
                    Some(MirConst::Char(c)) => {
                        let val = self.fb.ins().iconst(types::I32, *c as u32 as i64);
                        self.def_scalar(*dst, val)?;
                    }
                    // PLAN-870: one Str const serves both slot kinds — an
                    // OWNED slot materializes a fresh buffer (ledgered,
                    // `LIT` trace); a VIEW slot aliases the static bytes.
                    Some(MirConst::Str(bytes)) => match self.mir_type(ty)? {
                        MirType::Str => {
                            let addr = self.lit_addr(*value)?;
                            let len = self.fb.ins().iconst(types::I32, bytes.len() as i64);
                            let (p, l) = self.abic_call(
                                "ac_rt_own_from_lit",
                                &[types::I64, types::I32],
                                &[addr, len],
                            )?;
                            self.negative_guard(l)?;
                            self.def_pair(*dst, p, l)?;
                        }
                        MirType::StrView => {
                            let addr = self.lit_addr(*value)?;
                            let len = self.fb.ins().iconst(types::I32, bytes.len() as i64);
                            self.def_pair(*dst, addr, len)?;
                        }
                        _ => {
                            return self.reject(
                                "mn.unsupported.const",
                                "Str constant into a non-text slot",
                            )
                        }
                    },
                    None => return Err(format!("out-of-range constant {value}")),
                    // Wide numeric constants stay a controlled rejection
                    // (R2-N).
                    Some(_) => {
                        return Err(format!(
                            "constant {value} is a wide numeric value outside the mn surface"
                        ))
                    }
                }
            }
            MirInstr::Copy { dst, src } => {
                let ty = self.slot_ty(*src);
                match self.mir_type(ty)? {
                    MirType::I32 | MirType::Bool | MirType::Char => {
                        let v = self.fb.use_var(self.v_scalar(*src)?);
                        self.def_scalar(*dst, v)?;
                    }
                    MirType::StrView => {
                        let p = self.use_ptr(*src)?;
                        let l = self.use_len(*src)?;
                        self.def_pair(*dst, p, l)?;
                    }
                    MirType::Record(_) | MirType::RecordView(_) => {
                        // av.data/1 pure-value record copy (a resource
                        // record is not copyable — verify rejects; mn
                        // rejects defensively).
                        let paths = self.paths_of(ty);
                        if !paths.is_empty() {
                            return self.reject(
                                "mn.unsupported.instruction",
                                "COPY of a resource record is not copy semantics",
                            );
                        }
                        let size = self.record_size_of(ty)?;
                        let d = self.agg_addr(*dst)?;
                        let s = self.agg_addr(*src)?;
                        self.copy_agg_bytes(d, s, size, ty);
                    }
                    _ => {
                        return self.reject(
                            "mn.unsupported.instruction",
                            "COPY of an owned/non-copyable slot",
                        )
                    }
                }
            }
            MirInstr::Bin { op, dst, lhs, rhs } => {
                let a = self.fb.use_var(self.v_scalar(*lhs)?);
                let b = self.fb.use_var(self.v_scalar(*rhs)?);
                let r = match op {
                    BinOp::AddI32 => self.checked_add(a, b),
                    BinOp::SubI32 => self.checked_sub(a, b),
                    BinOp::MulI32 => self.checked_mul(a, b),
                    BinOp::DivI32 => self.guarded_sdiv(a, b),
                    BinOp::RemI32 => self.guarded_srem(a, b),
                    BinOp::LtI32 => self.icmp_bool(IntCC::SignedLessThan, a, b),
                    BinOp::LeI32 => self.icmp_bool(IntCC::SignedLessThanOrEqual, a, b),
                    BinOp::GtI32 => self.icmp_bool(IntCC::SignedGreaterThan, a, b),
                    BinOp::GeI32 => self.icmp_bool(IntCC::SignedGreaterThanOrEqual, a, b),
                    BinOp::EqI32 => self.icmp_bool(IntCC::Equal, a, b),
                    BinOp::NeI32 => self.icmp_bool(IntCC::NotEqual, a, b),
                };
                self.def_scalar(*dst, r)?;
            }
            MirInstr::Un { op, dst, operand } => {
                let x = self.fb.use_var(self.v_scalar(*operand)?);
                match op {
                    UnOp::NegI32 => {
                        let r = self.checked_neg(x);
                        self.def_scalar(*dst, r)?;
                    }
                    UnOp::NotBool => {
                        // Does not rely on a 0/1 precondition: compare to 0
                        // and widen — the result is strictly 0/1.
                        let zero = self.fb.ins().iconst(types::I32, 0);
                        let r = self.icmp_bool(IntCC::Equal, x, zero);
                        self.def_scalar(*dst, r)?;
                    }
                }
            }
            MirInstr::Call { callee, dst, args } => {
                let CalleeRef::Import(ii) = callee else {
                    return Err(
                        "direct Func callee in a unit — verify must reject this".to_string()
                    );
                };
                let imp = self
                    .unit
                    .imports
                    .get(*ii as usize)
                    .ok_or_else(|| format!("out-of-range import {ii}"))?;
                let sig = self
                    .unit
                    .sig_of(imp.sig)
                    .ok_or_else(|| format!("import {ii} signature missing"))?
                    .clone();
                let r = *self
                    .import_refs
                    .get(ii)
                    .ok_or_else(|| format!("out-of-range import {ii}"))?;
                // Marshal arguments per the frozen ABI (freeze §3).
                let mut argv: Vec<Value> = Vec::with_capacity(args.len() + 1);
                let mut str_args: Vec<u32> = Vec::new();
                for (arg, pty) in args.iter().zip(sig.params.iter()) {
                    match self.mir_type(*pty)? {
                        MirType::I32 | MirType::Bool | MirType::Char => {
                            argv.push(self.fb.use_var(self.v_scalar(*arg)?));
                        }
                        MirType::Str | MirType::StrView => {
                            argv.push(self.use_ptr(*arg)?);
                            argv.push(self.use_len(*arg)?);
                            if matches!(self.mir_type(*pty)?, MirType::Str) {
                                str_args.push(*arg);
                            }
                        }
                        MirType::Record(_) | MirType::RecordView(_) => {
                            argv.push(self.agg_addr(*arg)?);
                        }
                        _ => {
                            return self.reject(
                                "mn.unsupported.param",
                                "call argument type outside the mn data/owned ABI",
                            )
                        }
                    }
                }
                // Result marshalling (the record case appends the hidden
                // out-aggregate address).
                let result = self.mir_type(sig.result)?.clone();
                match result {
                    MirType::I32 | MirType::Bool | MirType::Char => {
                        let call = self.fb.ins().call(r, &argv);
                        let v = self.fb.inst_results(call)[0];
                        self.def_scalar(*dst, v)?;
                    }
                    MirType::Str | MirType::StrView => {
                        let call = self.fb.ins().call(r, &argv);
                        let p = self.fb.inst_results(call)[0];
                        let l = self.fb.inst_results(call)[1];
                        self.def_pair(*dst, p, l)?;
                    }
                    MirType::Record(_) | MirType::RecordView(_) => {
                        let out = self.agg_addr(*dst)?;
                        argv.push(out);
                        self.fb.ins().call(r, &argv);
                    }
                    _ => {
                        return self.reject(
                            "mn.unsupported.result",
                            "call result type outside the mn data/owned ABI",
                        )
                    }
                }
                // Str move-args transfer to the callee (freeze §2): zero
                // the caller's temp so its own exit sweep cannot double
                // free. Record args are NOT zeroed here — the callee's
                // consuming instruction zeroes through the pointer, and an
                // unconsumed arg frees at the caller sweep (AV parity).
                for a in str_args {
                    self.zero_pair(a)?;
                }
            }
            MirInstr::Trace { event } => {
                let r = self.trace_ref(event_symbol(*event))?;
                self.fb.ins().call(r, &[]); // result dropped — observation only
            }
            // ---- av.data/1 record instructions (PLAN-870 T-03) --------
            MirInstr::Aggregate {
                dst,
                record_ty,
                operands,
            } => {
                // split_units does not remap instruction-embedded type
                // refs (registered S-side gap): resolve the record through
                // the verify-guaranteed identity dst-slot-type ==
                // record_ty whenever the unit-local index dangles.
                let record_ty = self.resolve_record_ty(*record_ty, *dst)?;
                let r = match self.mir_type(record_ty)? {
                    MirType::Record(r) => r.clone(),
                    _ => {
                        return self.reject(
                            "mn.unsupported.instruction",
                            "AGGREGATE of a non-record type",
                        )
                    }
                };
                let dst_addr = self.agg_addr(*dst)?;
                let layout = layout::record_layout(self.unit, record_ty, &r, &mut self.layouts)
                    .map_err(|e| format!("aggregate layout: {e:?}"))?;
                for (i, fd) in r.fields.iter().enumerate() {
                    let off = layout.field_off[i] as i32;
                    let op = operands[i];
                    match self.mir_type(fd.ty)?.clone() {
                        MirType::I32 | MirType::Bool | MirType::Char => {
                            let v = self.fb.use_var(self.v_scalar(op)?);
                            self.fb.ins().store(MemFlags::trusted(), v, dst_addr, off);
                        }
                        // Owned field: transfer — bytes in, source zeroed.
                        MirType::Str => {
                            let flags = MemFlags::trusted();
                            let p = self.use_ptr(op)?;
                            let l = self.use_len(op)?;
                            self.fb.ins().store(flags, p, dst_addr, off);
                            self.fb.ins().store(flags, l, dst_addr, off + 8);
                            self.zero_pair(op)?;
                        }
                        MirType::Record(_) => {
                            let src = self.agg_addr(op)?;
                            let tgt = self.addr_plus(dst_addr, off);
                            let size = self.record_size_of(fd.ty)?;
                            self.copy_agg_bytes(tgt, src, size, fd.ty);
                            // Resource-record field: transfer — zero the
                            // operand's Str paths.
                            let paths = self.paths_of(fd.ty);
                            if !paths.is_empty() {
                                let zero = self.fb.ins().iconst(types::I64, 0);
                                for p in paths {
                                    let poff = self.path_offset_of(fd.ty, &p)? as i32;
                                    self.fb.ins().store(MemFlags::trusted(), zero, src, poff);
                                }
                            }
                        }
                        _ => {
                            return self.reject(
                                "mn.unsupported.instruction",
                                "AGGREGATE field type outside the mn surface",
                            )
                        }
                    }
                }
            }
            MirInstr::FieldGet { dst, base, path } => {
                let base_ty = self.slot_ty(*base);
                let fty = self.path_ty_of(base_ty, path)?;
                let off = self.path_offset_of(base_ty, path)? as i32;
                let base_addr = self.view_addr(*base)?;
                match self.mir_type(fty)?.clone() {
                    MirType::I32 | MirType::Bool | MirType::Char => {
                        let v = self
                            .fb
                            .ins()
                            .load(types::I32, MemFlags::trusted(), base_addr, off);
                        self.def_scalar(*dst, v)?;
                    }
                    // Field read = view (782): the pair copies out.
                    MirType::Str => match self.mir_type(self.slot_ty(*dst))? {
                        MirType::StrView => {
                            let flags = MemFlags::trusted();
                            let p = self.fb.ins().load(types::I64, flags, base_addr, off);
                            let l = self.fb.ins().load(types::I32, flags, base_addr, off + 8);
                            self.def_pair(*dst, p, l)?;
                        }
                        _ => {
                            return self.reject(
                                "mn.unsupported.instruction",
                                "owned field extraction must use FieldMove",
                            )
                        }
                    },
                    MirType::Record(_) => {
                        let size = self.record_size_of(fty)?;
                        let d = self.agg_addr(*dst)?;
                        let s = self.addr_plus(base_addr, off);
                        self.copy_agg_bytes(d, s, size, fty);
                    }
                    _ => {
                        return self.reject(
                            "mn.unsupported.instruction",
                            "field type outside the mn surface",
                        )
                    }
                }
            }
            MirInstr::FieldSet { root, path, value } => {
                let root_ty = self.slot_ty(*root);
                let fty = self.path_ty_of(root_ty, path)?;
                let off = self.path_offset_of(root_ty, path)? as i32;
                let root_addr = self.agg_addr(*root)?;
                let flags = MemFlags::trusted();
                match self.mir_type(fty)?.clone() {
                    MirType::I32 | MirType::Bool | MirType::Char => {
                        let v = self.fb.use_var(self.v_scalar(*value)?);
                        self.fb.ins().store(flags, v, root_addr, off);
                    }
                    // Owned field replace: free the old buffer, store the
                    // new pair, zero the value slot (transfer in).
                    MirType::Str => {
                        let old = self.fb.ins().load(types::I64, flags, root_addr, off);
                        let p = self.use_ptr(*value)?;
                        let l = self.use_len(*value)?;
                        self.fb.ins().store(flags, p, root_addr, off);
                        self.fb.ins().store(flags, l, root_addr, off + 8);
                        self.rt_free(old);
                        self.zero_pair(*value)?;
                    }
                    // Record field replace: free the old field's buffers,
                    // copy the new bytes, zero the source's paths.
                    MirType::Record(_) => {
                        let tgt = self.addr_plus(root_addr, off);
                        let src = self.agg_addr(*value)?;
                        let paths = self.paths_of(fty);
                        for p in &paths {
                            let poff = self.path_offset_of(fty, p)? as i32;
                            let old = self.fb.ins().load(types::I64, flags, tgt, poff);
                            self.rt_free(old);
                        }
                        let size = self.record_size_of(fty)?;
                        self.copy_agg_bytes(tgt, src, size, fty);
                        if !paths.is_empty() {
                            let zero = self.fb.ins().iconst(types::I64, 0);
                            for p in &paths {
                                let poff = self.path_offset_of(fty, p)? as i32;
                                self.fb.ins().store(flags, zero, src, poff);
                            }
                        }
                    }
                    _ => {
                        return self.reject(
                            "mn.unsupported.instruction",
                            "field type outside the mn surface",
                        )
                    }
                }
            }
            // ---- av.owned/1 instructions (PLAN-870 T-04) --------------
            MirInstr::Clone { dst, src } => {
                let ty = self.slot_ty(*src);
                match self.mir_type(ty)?.clone() {
                    MirType::Str => {
                        let p = self.use_ptr(*src)?;
                        let l = self.use_len(*src)?;
                        let (np, nl) =
                            self.abic_call("ac_rt_own_clone", &[types::I64, types::I32], &[p, l])?;
                        self.negative_guard(nl)?;
                        self.def_pair(*dst, np, nl)?;
                    }
                    MirType::Record(_) => {
                        let size = self.record_size_of(ty)?;
                        let d = self.agg_addr(*dst)?;
                        let s = self.agg_addr(*src)?;
                        self.copy_agg_bytes(d, s, size, ty);
                        // Null the temp's fields FIRST: a mid-clone failure
                        // frees only the fields cloned so far (the trap
                        // sweep reads dst, never src).
                        let paths = self.paths_of(ty);
                        let zero = self.fb.ins().iconst(types::I64, 0);
                        for p in &paths {
                            let off = self.path_offset_of(ty, p)? as i32;
                            self.fb.ins().store(MemFlags::trusted(), zero, d, off);
                        }
                        // One clone per Str path in DECLARATION order (the
                        // AV worklist's field order).
                        for p in &paths {
                            let off = self.path_offset_of(ty, p)? as i32;
                            let flags = MemFlags::trusted();
                            let sp = self.fb.ins().load(types::I64, flags, s, off);
                            let sl = self.fb.ins().load(types::I32, flags, s, off + 8);
                            let (np, nl) = self.abic_call(
                                "ac_rt_own_clone",
                                &[types::I64, types::I32],
                                &[sp, sl],
                            )?;
                            self.negative_guard(nl)?;
                            self.fb.ins().store(flags, np, d, off);
                            self.fb.ins().store(flags, nl, d, off + 8);
                        }
                    }
                    _ => {
                        return self.reject(
                            "mn.unsupported.instruction",
                            "CLONE of a copyable slot is not clone semantics",
                        )
                    }
                }
            }
            MirInstr::Move { dst, src } => {
                if self.unit.funcs[0].slots[*dst as usize].kind == SlotKind::Param {
                    return self.reject(
                        "mn.unsupported.instruction",
                        "ownership write into a parameter slot is outside the delivered adapter surface",
                    );
                }
                let ty = self.slot_ty(*src);
                match self.mir_type(ty)?.clone() {
                    // Whole-root transfer: bytes to dst (its old buffers
                    // free here — null-safe), source zeroed.
                    MirType::Str => {
                        let old = self.use_ptr(*dst)?;
                        let p = self.use_ptr(*src)?;
                        let l = self.use_len(*src)?;
                        self.rt_free(old);
                        self.def_pair(*dst, p, l)?;
                        self.zero_pair(*src)?;
                    }
                    MirType::Record(_) => {
                        let paths = self.paths_of(ty);
                        if paths.is_empty() {
                            return self.reject(
                                "mn.unsupported.instruction",
                                "MOVE of a copyable (pure) record is ownership-required",
                            );
                        }
                        let d = self.agg_addr(*dst)?;
                        let s = self.agg_addr(*src)?;
                        let flags = MemFlags::trusted();
                        // Dst's old buffers free first (entry-zeroed for
                        // locals/temps — null no-ops).
                        for p in &paths {
                            let off = self.path_offset_of(ty, p)? as i32;
                            let old = self.fb.ins().load(types::I64, flags, d, off);
                            self.rt_free(old);
                        }
                        let size = self.record_size_of(ty)?;
                        self.copy_agg_bytes(d, s, size, ty);
                        let zero = self.fb.ins().iconst(types::I64, 0);
                        for p in &paths {
                            let off = self.path_offset_of(ty, p)? as i32;
                            self.fb.ins().store(flags, zero, s, off);
                        }
                    }
                    _ => {
                        return self.reject(
                            "mn.unsupported.instruction",
                            "MOVE of a copyable/outside slot is ownership-required",
                        )
                    }
                }
            }
            MirInstr::View { dst, src } => {
                // The view roots the same buffer: pair copy (the byte
                // projection is the AV scalar range's exact image).
                let p = self.use_ptr(*src)?;
                let l = self.use_len(*src)?;
                self.def_pair(*dst, p, l)?;
            }
            MirInstr::MutRef { .. } | MirInstr::BorrowEnd { .. } => {
                // Verify-time exclusivity; zero runtime representation.
            }
            MirInstr::Drop { slot } => {
                let ty = self.slot_ty(*slot);
                match self.mir_type(ty)?.clone() {
                    MirType::Str => {
                        let p = self.use_ptr(*slot)?;
                        self.rt_free(p);
                        self.zero_pair(*slot)?;
                    }
                    MirType::Record(_) => {
                        let paths = self.paths_of(ty);
                        let addr = self.agg_addr(*slot)?;
                        let flags = MemFlags::trusted();
                        let zero = self.fb.ins().iconst(types::I64, 0);
                        // Free in DECLARATION order (the AV worklist's
                        // field pop order), zero each freed field.
                        for p in &paths {
                            let off = self.path_offset_of(ty, p)? as i32;
                            let bp = self.fb.ins().load(types::I64, flags, addr, off);
                            self.rt_free(bp);
                            self.fb.ins().store(flags, zero, addr, off);
                        }
                    }
                    _ => {
                        return self
                            .reject("mn.unsupported.instruction", "DROP of a non-owned slot")
                    }
                }
            }
            MirInstr::FieldMove { dst, root, path } => {
                let root_ty = self.slot_ty(*root);
                let fty = self.path_ty_of(root_ty, path)?;
                let off = self.path_offset_of(root_ty, path)? as i32;
                // Resolve the base address along the parent segments.
                let addr = self.view_addr(*root)?;
                let flags = MemFlags::trusted();
                match self.mir_type(fty)?.clone() {
                    // Single extraction: bytes out, the leaf zeroed IN THE
                    // ROOT (through the pointer for param roots — the
                    // caller temp dies with the transfer).
                    MirType::Str => {
                        let p = self.fb.ins().load(types::I64, flags, addr, off);
                        let l = self.fb.ins().load(types::I32, flags, addr, off + 8);
                        self.def_pair(*dst, p, l)?;
                        let zero = self.fb.ins().iconst(types::I64, 0);
                        self.fb.ins().store(flags, zero, addr, off);
                    }
                    MirType::Record(_) => {
                        let size = self.record_size_of(fty)?;
                        let d = self.agg_addr(*dst)?;
                        let s = self.addr_plus(addr, off);
                        self.copy_agg_bytes(d, s, size, fty);
                        let zero = self.fb.ins().iconst(types::I64, 0);
                        for p in self.paths_of(fty) {
                            let poff = self.path_offset_of(fty, &p)? as i32;
                            self.fb.ins().store(flags, zero, s, poff);
                        }
                    }
                    _ => {
                        return self.reject(
                            "mn.unsupported.instruction",
                            "FIELDMOVE of a copyable field is ownership-required",
                        )
                    }
                }
            }
            MirInstr::StrLen { dst, view } => {
                let p = self.use_ptr(*view)?;
                let l = self.use_len(*view)?;
                let r =
                    self.rt_i32_call("ac_rt_text_scalars", &[types::I64, types::I32], &[p, l])?;
                self.negative_guard(r)?;
                self.def_scalar(*dst, r)?;
            }
            MirInstr::StrLenBytes { dst, view } => {
                // The pair's len IS the byte length (byte-projected views).
                let l = self.use_len(*view)?;
                self.def_scalar(*dst, l)?;
            }
            MirInstr::StrCharAt { dst, view, index } => {
                let p = self.use_ptr(*view)?;
                let l = self.use_len(*view)?;
                let ix = self.fb.use_var(self.v_scalar(*index)?);
                let r = self.rt_i32_call(
                    "ac_rt_char_at",
                    &[types::I64, types::I32, types::I32],
                    &[p, l, ix],
                )?;
                self.negative_guard(r)?;
                self.def_scalar(*dst, r)?;
            }
            MirInstr::StrSlice {
                dst,
                view,
                start,
                end,
            } => {
                let p = self.use_ptr(*view)?;
                let l = self.use_len(*view)?;
                let s = self.fb.use_var(self.v_scalar(*start)?);
                let e = self.fb.use_var(self.v_scalar(*end)?);
                let (np, nl) = self.abic_call(
                    "ac_rt_text_slice",
                    &[types::I64, types::I32, types::I32, types::I32],
                    &[p, l, s, e],
                )?;
                self.negative_guard(nl)?;
                self.def_pair(*dst, np, nl)?;
            }
            MirInstr::StrConcat { dst, lhs, rhs } => {
                let ap = self.use_ptr(*lhs)?;
                let al = self.use_len(*lhs)?;
                let bp = self.use_ptr(*rhs)?;
                let bl = self.use_len(*rhs)?;
                let (np, nl) = self.abic_call(
                    "ac_rt_own_concat",
                    &[types::I64, types::I32, types::I64, types::I32],
                    &[ap, al, bp, bl],
                )?;
                self.negative_guard(nl)?;
                self.def_pair(*dst, np, nl)?;
            }
            MirInstr::StrToOwned { dst, view } => {
                // The view's byte range is exact — a fresh copy of it is
                // own_from_lit over those bytes.
                let p = self.use_ptr(*view)?;
                let l = self.use_len(*view)?;
                let (np, nl) =
                    self.abic_call("ac_rt_own_from_lit", &[types::I64, types::I32], &[p, l])?;
                self.negative_guard(nl)?;
                self.def_pair(*dst, np, nl)?;
            }
            MirInstr::StrCmp { eq, dst, lhs, rhs } => {
                let ap = self.use_ptr(*lhs)?;
                let al = self.use_len(*lhs)?;
                let bp = self.use_ptr(*rhs)?;
                let bl = self.use_len(*rhs)?;
                let sym = if *eq {
                    "ac_rt_text_eq"
                } else {
                    "ac_rt_text_ne"
                };
                let r = self.rt_i32_call(
                    sym,
                    &[types::I64, types::I32, types::I64, types::I32],
                    &[ap, al, bp, bl],
                )?;
                self.negative_guard(r)?;
                self.def_scalar(*dst, r)?;
            }
            MirInstr::CharToCodepoint { dst, ch } => {
                // Char IS its codepoint in the mn representation.
                let v = self.fb.use_var(self.v_scalar(*ch)?);
                self.def_scalar(*dst, v)?;
            }
            MirInstr::CharFromCodepoint { dst, code } => {
                let c = self.fb.use_var(self.v_scalar(*code)?);
                let r = self.rt_i32_call("ac_rt_char_from_codepoint", &[types::I32], &[c])?;
                self.negative_guard(r)?;
                self.def_scalar(*dst, r)?;
            }
            // ---- controlled rejections (outside this Plan) -----------
            MirInstr::ListNew { .. }
            | MirInstr::ListLen { .. }
            | MirInstr::ListCap { .. }
            | MirInstr::ListGet { .. }
            | MirInstr::ListPush { .. }
            | MirInstr::ListInsert { .. }
            | MirInstr::ListSet { .. }
            | MirInstr::ListRemove { .. }
            | MirInstr::ListFieldGet { .. }
            | MirInstr::ListFieldSet { .. }
            | MirInstr::VariantPack { .. }
            | MirInstr::VariantDisc { .. }
            | MirInstr::VariantGuard { .. }
            | MirInstr::VariantProject { .. }
            | MirInstr::VariantSwitch { .. }
            | MirInstr::ListPlace { .. }
            | MirInstr::PlaceGet { .. }
            | MirInstr::PlaceSet { .. } => {
                return Err(
                    "av.collections/1 list/variant instructions are outside the mn surface"
                        .to_string(),
                )
            }
            MirInstr::CreateClosure { .. }
            | MirInstr::CallClosure { .. }
            | MirInstr::EnvGet { .. } => {
                return Err(
                    "av.closures/1 instructions are outside the mn scalar surface".to_string(),
                );
            }
            MirInstr::DynPack { .. } | MirInstr::DynIs { .. } | MirInstr::DynTake { .. } => {
                return Err(
                    "av.dynamic/1 dyn instructions are outside the mn scalar surface".to_string(),
                )
            }
        }
        Ok(())
    }

    fn addr_plus(&mut self, base: Value, off: i32) -> Value {
        if off == 0 {
            return base;
        }
        let d = self.fb.ins().iconst(types::I64, off as i64);
        self.fb.ins().iadd(base, d)
    }

    /// Aggregate copy with per-type alignment knowledge (8-byte chunks
    /// only for 8-aligned layouts — the landed reference's chunking).
    fn copy_agg_bytes(&mut self, dst: Value, src: Value, size: u64, ty: u32) {
        let align8 = self
            .layouts
            .get(&ty)
            .map(|l| l.align_log2 >= 3)
            .unwrap_or(false);
        self.copy_bytes_aligned(dst, src, size, align8);
    }

    fn copy_bytes_aligned(&mut self, dst: Value, src: Value, size: u64, align8: bool) {
        let flags = MemFlags::trusted();
        let mut off = 0u64;
        if align8 {
            while size - off >= 8 {
                let v = self.fb.ins().load(types::I64, flags, src, off as i32);
                self.fb.ins().store(flags, v, dst, off as i32);
                off += 8;
            }
        }
        while size - off >= 4 {
            let v = self.fb.ins().load(types::I32, flags, src, off as i32);
            self.fb.ins().store(flags, v, dst, off as i32);
            off += 4;
        }
        if size - off >= 2 {
            let v = self.fb.ins().load(types::I16, flags, src, off as i32);
            self.fb.ins().store(flags, v, dst, off as i32);
            off += 2;
        }
        if size - off >= 1 {
            let v = self.fb.ins().load(types::I8, flags, src, off as i32);
            self.fb.ins().store(flags, v, dst, off as i32);
        }
    }

    fn rt_i32_call(
        &mut self,
        sym: &'static str,
        params: &[types::Type],
        args: &[Value],
    ) -> Result<Value, String> {
        let r = self.rt_ref(sym, params, &[types::I32])?;
        let call = self.fb.ins().call(r, args);
        Ok(self.fb.inst_results(call)[0])
    }

    fn emit_term(&mut self, t: &Terminator) -> Result<(), String> {
        match t {
            Terminator::Jump { target } => {
                let b = self.blocks[*target as usize];
                self.fb.ins().jump(b, &[]);
            }
            Terminator::Branch { cond, then, else_ } if then == else_ => {
                // Both polarities reach the same block: a brif with
                // duplicated targets would declare the same predecessor
                // twice — an unconditional jump is semantically identical
                // (the cond read itself has no observable effect).
                let _ = cond;
                let b = self.blocks[*then as usize];
                self.fb.ins().jump(b, &[]);
            }
            Terminator::Branch { cond, then, else_ } => {
                let c = self.fb.use_var(self.v_scalar(*cond)?);
                let tb = self.blocks[*then as usize];
                let eb = self.blocks[*else_ as usize];
                self.fb.ins().brif(c, tb, &[], eb, &[]);
            }
            Terminator::Return { value } => {
                // All-exit cleanup BEFORE the result leaves (the result
                // slot is excluded — its responsibility transfers out).
                self.emit_exit_sweep(Some(*value))?;
                let ty = self.slot_ty(*value);
                match self.mir_type(ty)?.clone() {
                    MirType::I32 | MirType::Bool | MirType::Char => {
                        let v = self.fb.use_var(self.v_scalar(*value)?);
                        self.fb.ins().return_(&[v]);
                    }
                    MirType::Str | MirType::StrView => {
                        let p = self.use_ptr(*value)?;
                        let l = self.use_len(*value)?;
                        self.fb.ins().return_(&[p, l]);
                    }
                    MirType::Record(_) => {
                        let out = self
                            .out_agg
                            .ok_or_else(|| "record result without an out slot".to_string())?;
                        let size = self.record_size_of(ty)?;
                        let src = self.agg_addr(*value)?;
                        self.copy_agg_bytes(out, src, size, ty);
                        self.fb.ins().return_(&[]);
                    }
                    other => {
                        return Err(format!(
                            "return of {other:?} is outside the mn data/owned ABI"
                        ))
                    }
                }
            }
        }
        Ok(())
    }
}

fn trace_call_sig() -> cranelift_codegen::ir::Signature {
    cranelift_codegen::ir::Signature {
        call_conv: cranelift_codegen::isa::CallConv::WindowsFastcall,
        params: vec![],
        returns: vec![cranelift_codegen::ir::AbiParam::new(types::I32)],
    }
}
