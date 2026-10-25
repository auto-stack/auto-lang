//! Function-body lowering: the frozen MIR→CLIF instruction table
//! (contract-decision §2). MIR blocks map to pre-created, pre-sealed CLIF
//! blocks (the CFG is static and complete); typed mutable slots map to
//! Cranelift variables (I32 physical for both i32 and bool — bool keeps
//! logical 0/1); the FunctionBuilder bridges to SSA. Arithmetic mirrors
//! the landed checked algorithms exactly: explicit pre-target overflow /
//! divide guards branching to the ExitProcess(70) trap block — a hardware
//! divide never sees a zero or −1 divisor, and a trap on an unexecuted
//! branch stays latent (no constant folding into unconditional failures).

use crate::emit::event_symbol;
use auto_av_core::mir::*;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, FuncRef, InstBuilder, TrapCode};
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

    let mut fx = FnLowerer {
        fb: &mut fb,
        om,
        unit,
        exit_id,
        vars: Vec::with_capacity(f.slots.len()),
        import_refs: BTreeMap::new(),
        trace_refs: BTreeMap::new(),
        trap_block: None,
        blocks,
    };

    // Slots → variables.
    for _ in &f.slots {
        let v = fx.fb.declare_var(types::I32);
        fx.vars.push(v);
    }
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
    {
        let params: Vec<_> = (0..n_params)
            .map(|i| fx.fb.block_params(entry_block)[i])
            .collect();
        for (i, pv) in params.into_iter().enumerate() {
            fx.fb.def_var(fx.vars[i], pv);
        }
    }

    // Body: blocks in MIR order; seal a block only when ALL its
    // predecessors have declared their edges — a block still expecting a
    // back edge (a loop header whose latch emits later) stays UNSEALED
    // while being filled; the SSA builder resolves its uses at seal time.
    for (bi, b) in f.blocks.iter().enumerate() {
        if !sealed[bi] && remaining[bi] == 0 {
            fx.fb.seal_block(fx.blocks[bi]);
            sealed[bi] = true;
        }
        fx.fb.switch_to_block(fx.blocks[bi]);
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

    // Trap block (only when some guard referenced it): ExitProcess(70),
    // then an unreachable trap so the block terminates.
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

struct FnLowerer<'a, 'b> {
    fb: &'a mut FunctionBuilder<'b>,
    om: &'a mut cranelift_object::ObjectModule,
    unit: &'a MirModule,
    exit_id: cranelift_module::FuncId,
    vars: Vec<Variable>,
    import_refs: BTreeMap<u32, FuncRef>,
    trace_refs: BTreeMap<String, FuncRef>,
    trap_block: Option<cranelift_codegen::ir::Block>,
    blocks: Vec<cranelift_codegen::ir::Block>,
}

impl<'a, 'b> FnLowerer<'a, 'b> {
    fn v(&self, slot: u32) -> Variable {
        self.vars[slot as usize]
    }

    fn use_slot(&mut self, slot: u32) -> cranelift_codegen::ir::Value {
        self.fb.use_var(self.v(slot))
    }

    fn def_slot(&mut self, slot: u32, val: cranelift_codegen::ir::Value) {
        self.fb.def_var(self.v(slot), val);
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

    /// `brif(ovf → trap70, else cont)`; the continuation is sealed with
    /// its single predecessor and becomes the current block.
    fn branch_to_trap_on(&mut self, ovf: cranelift_codegen::ir::Value) {
        let trap = self.ensure_trap_block();
        let cont = self.fb.create_block();
        self.fb.ins().brif(ovf, trap, &[], cont, &[]);
        self.fb.seal_block(cont);
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

    fn emit_instr(&mut self, i: &MirInstr) -> Result<(), String> {
        match i {
            MirInstr::Const { dst, value } => {
                let val = match self.unit.consts.get(*value as usize) {
                    Some(MirConst::I32(v)) => self.fb.ins().iconst(types::I32, *v as i64),
                    Some(MirConst::Bool(b)) => {
                        self.fb.ins().iconst(types::I32, if *b { 1 } else { 0 })
                    }
                    None => return Err(format!("out-of-range constant {value}")),
                    // PLAN-820 shared-core widening: wide constants are a
                    // controlled lower rejection (mn keeps its scalar ABI).
                    Some(_) => {
                        return Err(format!(
                            "constant {value} is an av.data/1 wide value outside the mn scalar surface"
                        ))
                    }
                };
                self.def_slot(*dst, val);
            }
            MirInstr::Copy { dst, src } => {
                let v = self.use_slot(*src);
                self.def_slot(*dst, v);
            }
            MirInstr::Bin { op, dst, lhs, rhs } => {
                let a = self.use_slot(*lhs);
                let b = self.use_slot(*rhs);
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
                self.def_slot(*dst, r);
            }
            MirInstr::Un { op, dst, operand } => {
                let x = self.use_slot(*operand);
                match op {
                    UnOp::NegI32 => {
                        let r = self.checked_neg(x);
                        self.def_slot(*dst, r);
                    }
                    UnOp::NotBool => {
                        // Does not rely on a 0/1 precondition: compare to 0
                        // and widen — the result is strictly 0/1.
                        let zero = self.fb.ins().iconst(types::I32, 0);
                        let r = self.icmp_bool(IntCC::Equal, x, zero);
                        self.def_slot(*dst, r);
                    }
                }
            }
            MirInstr::Call { callee, dst, args } => {
                let CalleeRef::Import(ii) = callee else {
                    return Err(
                        "direct Func callee in a unit — verify must reject this".to_string()
                    );
                };
                let r = *self
                    .import_refs
                    .get(ii)
                    .ok_or_else(|| format!("out-of-range import {ii}"))?;
                let argv: Vec<_> = args.iter().map(|a| self.use_slot(*a)).collect();
                let call = self.fb.ins().call(r, &argv);
                let result = self.fb.inst_results(call)[0];
                self.def_slot(*dst, result);
            }
            MirInstr::Trace { event } => {
                let r = self.trace_ref(event_symbol(*event))?;
                self.fb.ins().call(r, &[]); // result dropped — observation only
            }
            // PLAN-820 shared-core widening: the data-profile instructions
            // are outside mn's frozen lower surface — controlled rejection
            // (mn owner reviews at its own merge).
            MirInstr::FieldGet { .. } | MirInstr::FieldSet { .. } | MirInstr::Aggregate { .. } => {
                return Err(
                    "av.data/1 record instructions are outside the mn scalar surface".to_string()
                )
            }
        }
        Ok(())
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
                let c = self.use_slot(*cond);
                let tb = self.blocks[*then as usize];
                let eb = self.blocks[*else_ as usize];
                self.fb.ins().brif(c, tb, &[], eb, &[]);
            }
            Terminator::Return { value } => {
                let v = self.use_slot(*value);
                self.fb.ins().return_(&[v]);
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
