//! MIR verifier — the sole constructor of [`CheckedMir`] (PLAN-798 §5.2).
//!
//! Stage order (frozen): identity/budget → shape/range/quota (ALL blocks,
//! reachable or not) → control flow → definite initialization over the
//! entry-reachable CFG → call/return signature admission → capability
//! admission. Unreachable code is not a hiding place for unknown shapes:
//! every table and instruction is structure-checked; only the init dataflow
//! restricts itself to the reachable subgraph.
//!
//! Initialization uses a conservative must-worklist: block entry state is
//! the INTERSECTION of predecessor exit states (params initialized at
//! entry). Loops converge from the optimistic bound, which keeps legal
//! zero-iteration loops sound — an initialization inside a loop body never
//! escapes to the loop exit — while `def-then-use` inside one iteration is
//! accepted. Legal CFG cycles are dataflow, not errors.

use crate::contract::{
    ResourceKind, SpanInfo, TRACE_CAPABILITY, MAX_TABLE_ENTRIES, MAX_VERIFY_WALL,
    MAX_VERIFY_WORK_STEPS,
};
use crate::mir::*;
use std::collections::VecDeque;
use std::time::Instant;

/// Verifier rejection taxonomy. `Resource` breaches are budget outcomes;
/// `Unsupported` is out-of-profile content; `Invalid` is every structural,
/// typing, control-flow or initialization violation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MirVerifyError {
    Invalid { code: String, message: String, span: Option<SpanInfo> },
    Unsupported { code: String, message: String, span: Option<SpanInfo> },
    Resource { kind: ResourceKind, message: String },
}

impl MirVerifyError {
    fn inv(code: &str, message: String) -> Self {
        MirVerifyError::Invalid { code: code.to_string(), message, span: None }
    }

    fn uns(code: &str, message: String) -> Self {
        MirVerifyError::Unsupported { code: code.to_string(), message, span: None }
    }

    pub fn code(&self) -> String {
        match self {
            MirVerifyError::Invalid { code, .. }
            | MirVerifyError::Unsupported { code, .. } => code.clone(),
            MirVerifyError::Resource { kind, .. } => kind.code().to_string(),
        }
    }
}

/// Statistics recorded on the credential for observability.
#[derive(Clone, Copy, Debug, Default)]
pub struct VerifyStats {
    pub work_steps: u64,
    pub reachable_blocks: usize,
    pub total_blocks: usize,
}

/// Verified MIR. Constructed only here; the inner module is private so a
/// stale credential cannot survive a rewrite.
#[derive(Clone, Debug)]
pub struct CheckedMir {
    inner: MirModule,
    stats: VerifyStats,
}

impl CheckedMir {
    pub fn module(&self) -> &MirModule {
        &self.inner
    }

    pub fn stats(&self) -> &VerifyStats {
        &self.stats
    }

    /// Provenance consumed by the codec (identity only, never trust).
    pub fn profile(&self) -> &str {
        &self.inner.profile
    }
}

/// Verify an unchecked MIR module under the frozen budgets.
pub fn verify_mir(module: &MirModule) -> Result<CheckedMir, MirVerifyError> {
    verify_mir_with_deadline(module, Instant::now() + MAX_VERIFY_WALL)
}

pub fn verify_mir_with_deadline(
    module: &MirModule,
    deadline: Instant,
) -> Result<CheckedMir, MirVerifyError> {
    let mut v = Verifier { module, steps: 0, deadline };
    v.run()?;
    let stats = v.stats_snapshot();
    Ok(CheckedMir { inner: module.clone(), stats })
}

struct Verifier<'a> {
    module: &'a MirModule,
    steps: u64,
    deadline: Instant,
}

impl<'a> Verifier<'a> {
    fn tick(&mut self, n: u64) -> Result<(), MirVerifyError> {
        self.steps += n;
        if self.steps > MAX_VERIFY_WORK_STEPS {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::VerifySteps,
                message: format!("verify exceeded {} work steps", MAX_VERIFY_WORK_STEPS),
            });
        }
        if self.steps & 0x3ff == 0 && Instant::now() > self.deadline {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::VerifyWall,
                message: "verify exceeded wall budget".to_string(),
            });
        }
        Ok(())
    }

    fn stats_snapshot(&self) -> VerifyStats {
        VerifyStats {
            work_steps: self.steps,
            reachable_blocks: 0,
            total_blocks: self.module.funcs.iter().map(|f| f.blocks.len()).sum(),
        }
    }

    fn run(&mut self) -> Result<(), MirVerifyError> {
        // ---- identity -------------------------------------------------
        if self.module.profile != crate::contract::AV_PROFILE {
            return Err(MirVerifyError::uns(
                "mir.profile",
                format!("profile `{}` is not the supported `{}`", self.module.profile, crate::contract::AV_PROFILE),
            ));
        }
        if self.module.is_unit && self.module.funcs.len() != 1 {
            return Err(MirVerifyError::inv(
                "mir.unit-shape",
                format!("a unit carries exactly one function, found {}", self.module.funcs.len()),
            ));
        }

        // ---- budget: table entries BEFORE any deep allocation ----------
        // Scope (contract-decision.md §6 r1.1): the 65_536 cap is a PER-TABLE
        // allocation guard — module tables (types/consts/imports/functions)
        // and EACH function's slot/block/instruction table independently.
        // Whole-module totals are governed by the request-level work-step
        // and wall budgets (and the 4 MiB input cap), which is what makes
        // the frozen 100-function reference load representable without
        // weakening any single-table guard.
        let module_tables = self.module.types.len()
            + self.module.consts.len()
            + self.module.imports.len()
            + self.module.funcs.len();
        if module_tables > MAX_TABLE_ENTRIES {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::TableEntries,
                message: format!(
                    "module table entries {} exceed the frozen budget {}",
                    module_tables, MAX_TABLE_ENTRIES
                ),
            });
        }
        for f in &self.module.funcs {
            let fn_entries = f.slots.len() + f.blocks.len() + f.flat_instr_count() as usize;
            if fn_entries > MAX_TABLE_ENTRIES {
                return Err(MirVerifyError::Resource {
                    kind: ResourceKind::TableEntries,
                    message: format!(
                        "function `{}` table entries {} exceed the frozen budget {}",
                        f.name, fn_entries, MAX_TABLE_ENTRIES
                    ),
                });
            }
        }

        // ---- shape: type table -----------------------------------------
        for (ti, t) in self.module.types.iter().enumerate() {
            if let MirType::Fn(sig) = t {
                for p in sig.params.iter().chain(std::iter::once(&sig.result)) {
                    if *p as usize > self.module.types.len() {
                        return Err(MirVerifyError::inv(
                            "mir.type-ref",
                            format!("type #{ti} references out-of-range type {}", p),
                        ));
                    }
                }
                if sig.params.len() > u8::MAX as usize {
                    return Err(MirVerifyError::inv(
                        "mir.sig-arity",
                        format!("type #{ti} has more than {} params", u8::MAX),
                    ));
                }
            }
        }
        // ---- shape: imports --------------------------------------------
        for (ii, imp) in self.module.imports.iter().enumerate() {
            let Some(sig) = self.module.sig_of(imp.sig) else {
                return Err(MirVerifyError::inv(
                    "mir.import-sig",
                    format!("import #{} `{}` does not reference a function type", ii, imp.symbol),
                ));
            };
            self.check_sig_indices(sig)?;
        }
        // ---- capability admission for trace intrinsics ------------------
        let uses_trace = self.module.imports.iter().any(|i| {
            i.symbol == "hir.test.mark_a" || i.symbol == "hir.test.mark_b"
        }) || self.module.funcs.iter().any(|f| {
            f.blocks.iter().any(|b| b.instrs.iter().any(|i| matches!(i, MirInstr::Trace { .. })))
        });
        if uses_trace && !self.module.capabilities.iter().any(|c| c == TRACE_CAPABILITY) {
            return Err(MirVerifyError::uns(
                "mir.capability",
                format!(
                    "module uses test trace intrinsics but does not require `{}`",
                    TRACE_CAPABILITY
                ),
            ));
        }

        // ---- entry ------------------------------------------------------
        if self.module.entry as usize >= self.module.funcs.len() {
            return Err(MirVerifyError::inv(
                "mir.entry",
                format!("entry function index {} out of range", self.module.entry),
            ));
        }
        let entry_sig_idx = self.module.funcs[self.module.entry as usize].sig;
        let Some(entry_sig) = self.module.sig_of(entry_sig_idx) else {
            return Err(MirVerifyError::inv(
                "mir.entry-sig",
                "entry function does not reference a function type".to_string(),
            ));
        };
        // The frozen entry contract (zero-param i32) applies to MODULE
        // closures; single-function units verify their one function as-is —
        // closure assembly selects and re-checks the real module entry.
        if !self.module.is_unit {
            if !entry_sig.params.is_empty() || self.module.type_shape(entry_sig.result) != Some(ScalarShape::I32) {
                return Err(MirVerifyError::inv(
                    "mir.entry-sig",
                    "module entry must be zero-param i32 (frozen entry contract)".to_string(),
                ));
            }
        }
        let _ = entry_sig;

        // ---- per-function deep checks -----------------------------------
        for (fi, f) in self.module.funcs.iter().enumerate() {
            self.check_function(fi as u32, f)?;
        }
        Ok(())
    }

    fn check_sig_indices(&mut self, sig: &FnSig) -> Result<(), MirVerifyError> {
        for p in sig.params.iter().chain(std::iter::once(&sig.result)) {
            if *p as usize >= self.module.types.len() {
                return Err(MirVerifyError::inv(
                    "mir.type-ref",
                    format!("signature references out-of-range type {}", p),
                ));
            }
        }
        Ok(())
    }

    fn span_of(&self, f: &MirFunction, flat: u32) -> Option<SpanInfo> {
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

    fn check_function(&mut self, fi: u32, f: &MirFunction) -> Result<(), MirVerifyError> {
        let _ = fi;
        // Precomputed block bases (flat instruction offsets): the naive
        // per-call prefix sum made chain-shaped functions quadratic.
        let bases: Vec<u32> = {
            let mut v = Vec::with_capacity(f.blocks.len() + 1);
            let mut acc = 0u32;
            v.push(0);
            for b in &f.blocks {
                acc += b.instrs.len() as u32;
                v.push(acc);
            }
            v
        };
        // shape: sig
        let Some(sig) = self.module.sig_of(f.sig).cloned() else {
            return Err(MirVerifyError::inv(
                "mir.fn-sig",
                format!("function `{}` does not reference a function type", f.name),
            ));
        };
        // shape: slots
        for (si, s) in f.slots.iter().enumerate() {
            if s.ty as usize >= self.module.types.len() {
                return Err(MirVerifyError::inv(
                    "mir.slot-type",
                    format!("slot #{} of `{}` references out-of-range type {}", si, f.name, s.ty),
                ));
            }
        }
        // params first and in signature order
        let mut param_iter = f.slots.iter().enumerate().filter(|(_, s)| s.kind == SlotKind::Param);
        for (want_ty, _p) in sig.params.iter().enumerate() {
            match param_iter.next() {
                Some((si, s)) => {
                    if s.ty != sig.params[want_ty] {
                        return Err(MirVerifyError::inv(
                            "mir.param-type",
                            format!(
                                "param slot #{} of `{}` has type {}, signature wants {}",
                                si, f.name, s.ty, sig.params[want_ty]
                            ),
                        ));
                    }
                }
                None => {
                    return Err(MirVerifyError::inv(
                        "mir.param-count",
                        format!(
                            "function `{}` has fewer param slots than its signature ({})",
                            f.name,
                            sig.params.len()
                        ),
                    ));
                }
            }
        }
        if param_iter.next().is_some() {
            return Err(MirVerifyError::inv(
                "mir.param-count",
                format!("function `{}` has more param slots than its signature", f.name),
            ));
        }
        // shape: blocks / entry
        if f.entry as usize >= f.blocks.len() {
            return Err(MirVerifyError::inv(
                "mir.fn-entry",
                format!("entry block {} of `{}` out of range", f.entry, f.name),
            ));
        }
        // shape: spans sorted & in range
        let flat_total = f.flat_instr_count();
        for w in f.spans.windows(2) {
            if w[0].0 >= w[1].0 {
                return Err(MirVerifyError::inv(
                    "mir.span-order",
                    format!("span map of `{}` is not strictly sorted", f.name),
                ));
            }
        }
        if let Some(last) = f.spans.last() {
            if last.0 >= flat_total {
                return Err(MirVerifyError::inv(
                    "mir.span-range",
                    format!("span map of `{}` references instruction {} beyond count {}", f.name, last.0, flat_total),
                ));
            }
        }

        // instruction/terminator typing over ALL blocks (reachable or not)
        for (bi, b) in f.blocks.iter().enumerate() {
            for (ii, instr) in b.instrs.iter().enumerate() {
                self.tick(1)?;
                let flat = bases[bi] + ii as u32;
                let sp = self.span_of(f, flat);
                let err = |code: &str, msg: String| MirVerifyError::Invalid {
                    code: code.to_string(),
                    message: msg,
                    span: sp,
                };
                match instr {
                    MirInstr::Const { dst, value } => {
                        let dt = slot_ty_of(f, *dst, sp, "const dst")?;
                        let want_shape = match self.module.consts.get(*value as usize) {
                            Some(MirConst::I32(_)) => Some(ScalarShape::I32),
                            Some(MirConst::Bool(_)) => Some(ScalarShape::Bool),
                            None => {
                                return Err(err(
                                    "mir.const-ref",
                                    format!("const instruction references out-of-range constant {}", value),
                                ))
                            }
                        };
                        // Shape comparison, not index equality: a wire table
                        // may carry duplicate scalar type entries.
                        if self.module.type_shape(dt) != want_shape {
                            return Err(err(
                                "mir.const-type",
                                format!("const into slot {} of type {}; constant has another shape", dst, dt),
                            ));
                        }
                    }
                    MirInstr::Copy { dst, src } => {
                        let dt = slot_ty_of(f, *dst, sp, "copy dst")?;
                        let st = slot_ty_of(f, *src, sp, "copy src")?;
                        if dt != st {
                            return Err(err(
                                "mir.copy-type",
                                format!("copy from type {} into type {}", st, dt),
                            ));
                        }
                    }
                    MirInstr::Bin { op, dst, lhs, rhs } => {
                        let dt = slot_ty_of(f, *dst, sp, "bin dst")?;
                        let lt = slot_ty_of(f, *lhs, sp, "bin lhs")?;
                        let rt = slot_ty_of(f, *rhs, sp, "bin rhs")?;
                        let shapes = (
                            self.module.type_shape(dt),
                            self.module.type_shape(lt),
                            self.module.type_shape(rt),
                        );
                        use ScalarShape::*;
                        let ok = if op.result_is_bool() {
                            match op {
                                BinOp::EqI32 | BinOp::NeI32 => {
                                    shapes.1.is_some() && shapes.1 == shapes.2 && shapes.0 == Some(Bool)
                                }
                                _ => {
                                    shapes == (Some(Bool), Some(I32), Some(I32))
                                }
                            }
                        } else {
                            shapes == (Some(I32), Some(I32), Some(I32))
                        };
                        if !ok {
                            return Err(err(
                                "mir.bin-type",
                                format!("binary {:?} does not type check for dst/lhs/rhs types ({},{},{})", op, dt, lt, rt),
                            ));
                        }
                    }
                    MirInstr::Un { op, dst, operand } => {
                        let dt = slot_ty_of(f, *dst, sp, "un dst")?;
                        let ot = slot_ty_of(f, *operand, sp, "un operand")?;
                        let ok = match op {
                            UnOp::NegI32 => {
                                self.module.type_shape(dt) == Some(ScalarShape::I32)
                                    && self.module.type_shape(ot) == Some(ScalarShape::I32)
                            }
                            UnOp::NotBool => {
                                self.module.type_shape(dt) == Some(ScalarShape::Bool)
                                    && self.module.type_shape(ot) == Some(ScalarShape::Bool)
                            }
                        };
                        if !ok {
                            return Err(err(
                                "mir.un-type",
                                format!("unary {:?} does not type check for types ({},{})", op, dt, ot),
                            ));
                        }
                    }
                    MirInstr::Call { callee, dst, args } => {
                        let dt = slot_ty_of(f, *dst, sp, "call dst")?;
                        let callee_sig: FnSig = match callee {
                            CalleeRef::Func(idx) => {
                                if self.module.is_unit {
                                    return Err(err(
                                        "mir.unit-callee",
                                        "a unit must address callees through imports".to_string(),
                                    ));
                                }
                                match self.module.funcs.get(*idx as usize) {
                                    Some(target) => match self.module.sig_of(target.sig) {
                                        Some(s) => s.clone(),
                                        None => {
                                            return Err(err(
                                                "mir.callee-sig",
                                                format!("callee #{} has no function type", idx),
                                            ))
                                        }
                                    },
                                    None => {
                                        return Err(err(
                                            "mir.callee-ref",
                                            format!("call references out-of-range function {}", idx),
                                        ))
                                    }
                                }
                            }
                            CalleeRef::Import(idx) => match self.module.imports.get(*idx as usize) {
                                Some(imp) => match self.module.sig_of(imp.sig) {
                                    Some(s) => s.clone(),
                                    None => {
                                        return Err(err(
                                            "mir.import-sig",
                                            format!("import #{} has no function type", idx),
                                        ))
                                    }
                                },
                                None => {
                                    return Err(err(
                                        "mir.import-ref",
                                        format!("call references out-of-range import {}", idx),
                                    ))
                                }
                            },
                        };
                        if callee_sig.params.len() != args.len() {
                            return Err(err(
                                "mir.call-arity",
                                format!(
                                    "call passes {} args, callee signature takes {}",
                                    args.len(),
                                    callee_sig.params.len()
                                ),
                            ));
                        }
                        for (ai, (a, want)) in args.iter().zip(callee_sig.params.iter()).enumerate() {
                            let at = slot_ty_of(f, *a, sp, &format!("call arg #{}", ai))?;
                            if at != *want {
                                return Err(err(
                                    "mir.call-arg-type",
                                    format!("call arg #{} has type {}, signature wants {}", ai, at, want),
                                ));
                            }
                        }
                        if dt != callee_sig.result {
                            return Err(err(
                                "mir.call-result-type",
                                format!("call dst has type {}, callee returns {}", dt, callee_sig.result),
                            ));
                        }
                    }
                    MirInstr::Trace { .. } => {}
                }
            }
            // terminator
            self.tick(1)?;
            let sp = self.span_of(f, bases[bi + 1]);
            let err = |code: &str, msg: String| MirVerifyError::Invalid {
                code: code.to_string(),
                message: msg,
                span: sp,
            };
            match &b.term {
                Terminator::Jump { target } => {
                    if *target as usize >= f.blocks.len() {
                        return Err(err(
                            "mir.jump-target",
                            format!("jump target {} out of range", target),
                        ));
                    }
                }
                Terminator::Branch { cond, then, else_ } => {
                    let ct = slot_ty_of(f, *cond, sp, "branch cond")?;
                    if self.module.type_shape(ct) != Some(ScalarShape::Bool) {
                        return Err(err(
                            "mir.branch-cond-type",
                            format!("branch condition slot type {} is not bool", ct),
                        ));
                    }
                    for (w, t) in [("then", then), ("else", else_)] {
                        if *t as usize >= f.blocks.len() {
                            return Err(err(
                                "mir.branch-target",
                                format!("branch {} target {} out of range", w, t),
                            ));
                        }
                    }
                }
                Terminator::Return { value } => {
                    let vt = slot_ty_of(f, *value, sp, "return value")?;
                    if vt != sig.result {
                        return Err(err(
                            "mir.return-type",
                            format!("return value type {} does not match signature result {}", vt, sig.result),
                        ));
                    }
                }
            }
        }
        // ---- definite initialization over the reachable CFG -------------
        self.check_init(f, &bases)
    }

    fn check_init(&mut self, f: &MirFunction, bases: &[u32]) -> Result<(), MirVerifyError> {
        let n = f.slots.len();
        let words = words_for(n);
        let blocks = f.blocks.len();
        // Def sets per block (fixed after the typing pass).
        let defs: Vec<Vec<u64>> = f
            .blocks
            .iter()
            .map(|b| {
                let mut m = vec![0u64; words];
                for instr in &b.instrs {
                    if let Some(d) = instr.def_slot() {
                        set_bit(&mut m, d as usize);
                    }
                }
                m
            })
            .collect();
        // Successors.
        let succs: Vec<Vec<u32>> = f
            .blocks
            .iter()
            .map(|b| match &b.term {
                Terminator::Jump { target } => vec![*target],
                Terminator::Branch { then, else_, .. } => {
                    if then == else_ {
                        vec![*then]
                    } else {
                        vec![*then, *else_]
                    }
                }
                Terminator::Return { .. } => vec![],
            })
            .collect();
        // Predecessors (reachable ones only, computed on the fly below).
        let mut preds: Vec<Vec<u32>> = vec![Vec::new(); blocks];
        // Reachability from entry.
        let mut reachable = vec![false; blocks];
        let mut stack = vec![f.entry];
        reachable[f.entry as usize] = true;
        while let Some(b) = stack.pop() {
            for s in &succs[b as usize] {
                if !reachable[*s as usize] {
                    reachable[*s as usize] = true;
                    stack.push(*s);
                }
            }
        }
        for b in 0..blocks {
            if !reachable[b] {
                continue;
            }
            for s in &succs[b] {
                if !preds[*s as usize].contains(&(b as u32)) {
                    preds[*s as usize].push(b as u32);
                }
            }
        }

        // In/out states; non-entry blocks start optimistic (all ones).
        let mut in_state: Vec<Vec<u64>> = (0..blocks)
            .map(|b| {
                if (b as u32) == f.entry {
                    let mut m = vec![0u64; words];
                    for (si, s) in f.slots.iter().enumerate() {
                        if s.kind == SlotKind::Param {
                            set_bit(&mut m, si);
                        }
                    }
                    m
                } else {
                    vec![u64::MAX; words]
                }
            })
            .collect();
        let mut out_state: Vec<Vec<u64>> = in_state
            .iter()
            .zip(defs.iter())
            .map(|(i, d)| union(i, d))
            .collect();
        // Terminator reads.
        let term_reads: Vec<Option<(u32, u32)>> = f
            .blocks
            .iter()
            .enumerate()
            .map(|(bi, b)| {
                let flat = bases[bi + 1];
                match &b.term {
                    Terminator::Branch { cond, .. } => Some((*cond, flat)),
                    Terminator::Return { value } => Some((*value, flat)),
                    Terminator::Jump { .. } => None,
                }
            })
            .collect();

        // Reverse postorder for a fast convergence seed.
        let mut rpo: Vec<u32> = Vec::with_capacity(blocks);
        {
            let mut seen = vec![false; blocks];
            let mut stack: Vec<(u32, usize)> = vec![(f.entry, 0)];
            seen[f.entry as usize] = true;
            while let Some((b, ci)) = stack.pop() {
                let ss = &succs[b as usize];
                if ci < ss.len() {
                    stack.push((b, ci + 1));
                    let s = ss[ci];
                    if !seen[s as usize] {
                        seen[s as usize] = true;
                        stack.push((s, 0));
                    }
                } else {
                    rpo.push(b);
                }
            }
            rpo.reverse();
        }

        let mut queue: VecDeque<u32> = rpo.iter().copied().collect();
        let mut queued = vec![false; blocks];
        for b in &queue {
            queued[*b as usize] = true;
        }
        let mut visits: u64 = 0;
        while let Some(b) = queue.pop_front() {
            queued[b as usize] = false;
            visits += 1;
            self.tick(1 + words as u64)?;
            // in = ∩ preds (reachable) out
            let mut new_in = if b == f.entry {
                in_state[b as usize].clone()
            } else {
                match preds[b as usize]
                    .iter()
                    .filter(|p| reachable[**p as usize])
                    .next()
                {
                    Some(first) => out_state[*first as usize].clone(),
                    None => vec![0u64; words], // unreachable-defensive
                }
            };
            if b != f.entry {
                for p in preds[b as usize].iter().filter(|p| reachable[**p as usize]).skip(1) {
                    intersect(&mut new_in, &out_state[*p as usize]);
                }
            }
            // Reads must be initialized in `in ∪ defs-so-far`: walk instructions
            // in order, checking uses then applying the def.
            let mut live = new_in.clone();
        let mut fail: Option<(u32, u32)> = None;
        'blk: for (ii, instr) in f.blocks[b as usize].instrs.iter().enumerate() {
            let flat = bases[b as usize] + ii as u32;
            for u in instr.use_slots() {
                if !bit_is_set(&live, u as usize) {
                    fail = Some((u, flat));
                    break 'blk;
                }
            }
            if let Some(d) = instr.def_slot() {
                set_bit(&mut live, d as usize);
            }
        }
        if fail.is_none() {
            if let Some((slot, flat)) = term_reads[b as usize] {
                if !bit_is_set(&live, slot as usize) {
                    fail = Some((slot, flat));
                }
            }
        }
            if let Some((slot, flat)) = fail {
                let sp = self.span_of(f, flat);
                let name = f
                    .slots
                    .get(slot as usize)
                    .and_then(|s| s.name.clone())
                    .unwrap_or_else(|| format!("slot#{}", slot));
                return Err(MirVerifyError::Invalid {
                    code: "mir.uninitialized-read".to_string(),
                    message: format!(
                        "read of uninitialized slot `{}` (kind {:?}) in `{}`",
                        name,
                        f.slots.get(slot as usize).map(|s| s.kind).unwrap_or(SlotKind::Temp),
                        f.name
                    ),
                    span: sp,
                });
            }
            // out = in ∪ defs
            let mut new_out = new_in.clone();
            union_into(&mut new_out, &defs[b as usize]);
            if new_in != in_state[b as usize] {
                in_state[b as usize] = new_in;
                out_state[b as usize] = new_out;
                for s in &succs[b as usize] {
                    if !queued[*s as usize] {
                        queued[*s as usize] = true;
                        queue.push_back(*s);
                    }
                }
            } else if new_out != out_state[b as usize] {
                out_state[b as usize] = new_out;
            }
            if visits > (blocks as u64) * 16 + 1024 {
                // Defensive: the lattice guarantees termination; this only
                // catches implementation bugs without looping forever.
                return Err(MirVerifyError::Resource {
                    kind: ResourceKind::VerifySteps,
                    message: "init worklist exceeded its defensive visit bound".to_string(),
                });
            }
        }
        Ok(())
    }
}

impl MirInstr {
    /// The slot this instruction initializes (its definition), if any.
    pub fn def_slot(&self) -> Option<SlotId> {
        match self {
            MirInstr::Const { dst, .. }
            | MirInstr::Copy { dst, .. }
            | MirInstr::Bin { dst, .. }
            | MirInstr::Un { dst, .. }
            | MirInstr::Call { dst, .. } => Some(*dst),
            MirInstr::Trace { .. } => None,
        }
    }

    /// Slots this instruction reads, in operand order.
    pub fn use_slots(&self) -> Vec<SlotId> {
        match self {
            MirInstr::Const { .. } | MirInstr::Trace { .. } => vec![],
            MirInstr::Copy { src, .. } => vec![*src],
            MirInstr::Bin { lhs, rhs, .. } => vec![*lhs, *rhs],
            MirInstr::Un { operand, .. } => vec![*operand],
            MirInstr::Call { args, .. } => args.clone(),
        }
    }
}

// -- bitset helpers -----------------------------------------------------------

fn words_for(bits: usize) -> usize {
    bits.div_ceil(64)
}

/// Slot type lookup with the shared out-of-range rejection.
fn slot_ty_of(
    f: &MirFunction,
    s: SlotId,
    sp: Option<SpanInfo>,
    what: &str,
) -> Result<TypeIdx, MirVerifyError> {
    f.slots
        .get(s as usize)
        .map(|x| x.ty)
        .ok_or_else(|| MirVerifyError::Invalid {
            code: "mir.slot-ref".to_string(),
            message: format!("{} references out-of-range slot {}", what, s),
            span: sp,
        })
}

fn set_bit(m: &mut [u64], i: usize) {
    m[i / 64] |= 1u64 << (i % 64);
}

fn bit_is_set(m: &[u64], i: usize) -> bool {
    m.get(i / 64).map(|w| *w >> (i % 64) & 1 == 1).unwrap_or(false)
}

fn union(a: &[u64], b: &[u64]) -> Vec<u64> {
    a.iter().zip(b.iter()).map(|(x, y)| x | y).collect()
}

fn union_into(a: &mut [u64], b: &[u64]) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x |= *y;
    }
}

fn intersect(a: &mut [u64], b: &[u64]) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x &= *y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::MIR_SCHEMA;

    fn owner() -> OwnerScope {
        OwnerScope { schema: MIR_SCHEMA.to_string(), module: "m".to_string() }
    }

    fn entry_fn(m: &mut MirModule, name: &str) -> u32 {
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig { params: vec![], result: ti }));
        let f = MirFunction {
            owner: owner(),
            name: name.to_string(),
            sig,
            slots: vec![],
            blocks: vec![MirBlock { instrs: vec![], term: Terminator::Return { value: 0 } }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
        };
        m.funcs.push(f);
        (m.funcs.len() - 1) as u32
    }

    #[test]
    fn mir_admission_minimal_entry_passes() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        // one slot holding the return value
        let t = m.intern_type(MirType::I32);
        m.funcs[fi as usize].slots = vec![MirSlot { kind: SlotKind::Local, ty: t, name: Some("r".into()) }];
        let c = m.intern_const(MirConst::I32(7));
        m.funcs[fi as usize].blocks[0].instrs = vec![MirInstr::Const { dst: 0, value: c }];
        m.entry = fi;
        let checked = verify_mir(&m).expect("verified");
        assert_eq!(checked.module().funcs.len(), 1);
    }

    #[test]
    fn mir_admission_uninitialized_read_rejected() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let t = m.intern_type(MirType::I32);
        m.funcs[fi as usize].slots = vec![
            MirSlot { kind: SlotKind::Local, ty: t, name: Some("r".into()) },
            MirSlot { kind: SlotKind::Local, ty: t, name: Some("dead".into()) },
        ];
        // return slot 1 without initializing it
        m.funcs[fi as usize].blocks[0].term = Terminator::Return { value: 1 };
        m.entry = fi;
        let err = verify_mir(&m).unwrap_err();
        assert!(matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.uninitialized-read"));
    }

    #[test]
    fn mir_admission_branch_merge_needs_both_arms() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let ti = m.intern_type(MirType::I32);
        let tb = m.intern_type(MirType::Bool);
        m.funcs[fi as usize].slots = vec![
            MirSlot { kind: SlotKind::Temp, ty: tb, name: None },
            MirSlot { kind: SlotKind::Local, ty: ti, name: Some("r".into()) },
        ];
        let c_true = m.intern_const(MirConst::Bool(true));
        let c1 = m.intern_const(MirConst::I32(1));
        let f = &mut m.funcs[fi as usize];
        // b0: r = 1 (slot1); branch on slot0 (uninit bool → also invalid, so init it first)
        f.blocks[0].instrs = vec![
            MirInstr::Const { dst: 0, value: c_true },
            MirInstr::Const { dst: 1, value: c1 },
        ];
        f.blocks[0].term = Terminator::Branch { cond: 0, then: 1, else_: 2 };
        // b1: jump b2 ; b2: return slot1 — all init: fine
        f.blocks.push(MirBlock { instrs: vec![], term: Terminator::Jump { target: 2 } });
        f.blocks.push(MirBlock { instrs: vec![], term: Terminator::Return { value: 1 } });
        m.entry = fi;
        verify_mir(&m).expect("both arms keep r initialized");

        // now make the then-arm reinit-only path: remove init from entry
        let f = &mut m.funcs[fi as usize];
        f.blocks[0].instrs.retain(|i| matches!(i, MirInstr::Const { dst: 0, .. }));
        // slot1 initialized in NEITHER arm now → rejection
        let err = verify_mir(&m).unwrap_err();
        assert!(matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.uninitialized-read"));
    }

    #[test]
    fn mir_admission_loop_init_does_not_escape() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let ti = m.intern_type(MirType::I32);
        let tb = m.intern_type(MirType::Bool);
        m.funcs[fi as usize].slots = vec![
            MirSlot { kind: SlotKind::Temp, ty: tb, name: None }, // 0 cond
            MirSlot { kind: SlotKind::Local, ty: ti, name: Some("r".into()) }, // 1
        ];
        let c_true = m.intern_const(MirConst::Bool(true));
        let c1 = m.intern_const(MirConst::I32(1));
        let f = &mut m.funcs[fi as usize];
        f.blocks[0].instrs = vec![MirInstr::Const { dst: 0, value: c_true }];
        f.blocks[0].term = Terminator::Branch { cond: 0, then: 1, else_: 3 };
        // b1 (loop body): r = 1; jump b2
        f.blocks.push(MirBlock {
            instrs: vec![MirInstr::Const { dst: 1, value: c1 }],
            term: Terminator::Jump { target: 2 },
        });
        // b2 (latch): branch cond → body : exit
        f.blocks.push(MirBlock { instrs: vec![], term: Terminator::Branch { cond: 0, then: 1, else_: 3 } });
        // b3 (exit): return r — r only initialized INSIDE the loop → reject
        f.blocks.push(MirBlock { instrs: vec![], term: Terminator::Return { value: 1 } });
        m.entry = fi;
        let err = verify_mir(&m).unwrap_err();
        assert!(matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.uninitialized-read"), "got {err:?}");
    }

    #[test]
    fn mir_admission_dangling_jump_target() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let f = &mut m.funcs[fi as usize];
        f.blocks[0].term = Terminator::Jump { target: 9 };
        m.entry = fi;
        let err = verify_mir(&m).unwrap_err();
        assert!(matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.jump-target"));
    }

    #[test]
    fn mir_admission_wrong_entry_result_type() {
        let mut m = MirModule::new(owner());
        let tb = m.intern_type(MirType::Bool);
        let sig = m.intern_type(MirType::Fn(FnSig { params: vec![], result: tb }));
        let f = MirFunction {
            owner: owner(),
            name: "main".into(),
            sig,
            slots: vec![MirSlot { kind: SlotKind::Local, ty: tb, name: None }],
            blocks: vec![MirBlock { instrs: vec![], term: Terminator::Return { value: 0 } }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
        };
        m.funcs.push(f);
        m.entry = 0;
        let err = verify_mir(&m).unwrap_err();
        assert!(matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.entry-sig"));
    }

    #[test]
    fn mir_admission_table_budget_is_resource() {
        let mut m = MirModule::new(owner());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig { params: vec![], result: ti }));
        // A function with many empty blocks to blow the 65_536 budget.
        let blocks: Vec<MirBlock> = (0..70_000)
            .map(|_| MirBlock { instrs: vec![], term: Terminator::Jump { target: 0 } })
            .collect();
        m.funcs.push(MirFunction {
            owner: owner(),
            name: "big".into(),
            sig,
            slots: vec![MirSlot { kind: SlotKind::Local, ty: ti, name: None }],
            blocks,
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
        });
        m.entry = 0;
        let err = verify_mir(&m).unwrap_err();
        assert!(matches!(err, MirVerifyError::Resource { kind: ResourceKind::TableEntries, .. }));
    }
}
