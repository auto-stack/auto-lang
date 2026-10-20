//! Checked HIR → Typed MIR lowering (PLAN-798 §5.1/§5.2).
//!
//! Consumes an `auto_ac_prototype::verify::CheckedModule` of the Scalar (or
//! Core subset) domain and produces an UNCHECKED `MirModule`; the result
//! must still pass `verify_mir` — the adapter grants nothing.
//!
//! Mapping rules (frozen by contract-decision.md):
//! - HIR locals → typed slots (params lead in signature order); one slot
//!   per `LocalId` (shadowing creates new HIR locals).
//! - HIR's structured block tree (If/Loop child blocks) flattens into
//!   explicit MIR basic blocks; scalar `for` is already HIR-expanded into
//!   Loop/If/Break by the front end, so it needs no special case here.
//! - Expressions evaluate into temp slots in SOURCE order; call arguments
//!   evaluate in `eval_args` order then map through `bindings` into
//!   parameter order — never re-ordered by the binding table.
//! - Lazy `Logical` lowers to branch-based short-circuit with a bool result
//!   slot defined in BOTH arms (verify's intersection init covers the join).
//! - `hir.test.mark_a`/`mark_b` intrinsics map to AV trace events; per the
//!   landed native contract (`test-support/marks.rs`) their i32 results are
//!   1 and 2. `true`/`false` literals arrive HIR-expanded as `0<1`/`0<0`,
//!   so HIR constants are i32-only here.
//!
//! Statement walking is iterative (explicit task stack — HIR block nesting
//! is unbounded); expression evaluation recurses with a depth ceiling well
//! above the 128-level source front door, rejecting deeper hand-made trees
//! with a controlled `mir.lower.depth` error.

use auto_ac_prototype::hir;
use auto_ac_prototype::verify::CheckedModule;
use auto_av_core::contract::SpanInfo;
use auto_av_core::mir::*;
use std::collections::BTreeMap;

/// Adapter rejection taxonomy (mirrors the core taxonomy).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdapterError {
    Unsupported { code: String, message: String },
    Invalid { code: String, message: String },
}

impl AdapterError {
    fn uns(code: &str, message: String) -> Self {
        AdapterError::Unsupported {
            code: code.to_string(),
            message,
        }
    }
    fn inv(code: &str, message: String) -> Self {
        AdapterError::Invalid {
            code: code.to_string(),
            message,
        }
    }
    pub fn code(&self) -> String {
        match self {
            AdapterError::Unsupported { code, .. } | AdapterError::Invalid { code, .. } => {
                code.clone()
            }
        }
    }
}

/// Expression-evaluation recursion ceiling. The source front door is 128;
/// anything deeper reaching this adapter is a hand-made bundle.
const MAX_EXPR_DEPTH: u32 = 512;

/// Lower a checked scalar/core module to an unchecked MIR module with
/// `entry_name` selected as the entry (zero-param i32 per the frozen entry
/// contract — enforced by `verify_mir`, not here).
pub fn lower_checked(
    checked: &CheckedModule,
    entry_name: &str,
    source_snapshot: u64,
) -> Result<MirModule, AdapterError> {
    match checked.domain() {
        hir::HirDomain::Scalar | hir::HirDomain::Core => {}
        other => {
            return Err(AdapterError::uns(
                "lower.domain",
                format!(
                    "HIR domain `{}` is outside the av.core-i32/1 profile",
                    other.as_str()
                ),
            ));
        }
    }
    if checked.bundle().modules.len() != 1 {
        return Err(AdapterError::uns(
            "lower.modules",
            format!(
                "expected exactly one module, found {} (multi-module lowering is future work)",
                checked.bundle().modules.len()
            ),
        ));
    }
    let module = &checked.bundle().modules[0];
    let owner = OwnerScope {
        schema: auto_av_core::contract::MIR_SCHEMA.to_string(),
        module: module.id_text.clone(),
    };
    let mut out = MirModule::new(owner);
    out.capabilities = checked.required_capabilities().to_vec();

    // Pass 1: scan defs — intern imports and fix function indices before any
    // body lowers (calls may reference later defs).
    let mut def_map: BTreeMap<u32, CalleeRef> = BTreeMap::new();
    let mut fn_order: Vec<(u32, hir::TypeId, hir::BodyId, String)> = Vec::new();
    let mut next_fn_idx: u32 = 0;
    for (di, d) in module.defs.iter().enumerate() {
        match &d.kind {
            hir::DefKind::Function { ty, body, .. } => {
                def_map.insert(di as u32, CalleeRef::Func(next_fn_idx));
                next_fn_idx += 1;
                // MIR units are named by the def id (the entry contract's
                // textual id, e.g. `d_main`); the declared name is a
                // frontend-internal symbol.
                fn_order.push((di as u32, *ty, *body, d.id_text.clone()));
            }
            hir::DefKind::Method { .. } => {
                return Err(AdapterError::uns(
                    "lower.method",
                    "type-bound methods belong to the record domain, outside the av profile"
                        .to_string(),
                ));
            }
            hir::DefKind::Intrinsic { symbol, effect, ty } => {
                if symbol != "hir.test.mark_a" && symbol != "hir.test.mark_b" {
                    return Err(AdapterError::uns(
                        "lower.intrinsic",
                        format!("intrinsic `{}` is not in the av profile", symbol),
                    ));
                }
                let sig = fn_type_index(*ty, module, &mut out)?;
                let idx = out.intern_import(MirImport {
                    symbol: symbol.clone(),
                    effect: match effect {
                        hir::Effect::Observable => Effect::Observable,
                        hir::Effect::Pure => Effect::Pure,
                    },
                    sig,
                });
                def_map.insert(di as u32, CalleeRef::Import(idx));
            }
        }
    }

    // Pass 2: lower bodies.
    let mut entry_idx: Option<u32> = None;
    for (di, ty, body_id, name) in fn_order {
        let sig = fn_type_index(ty, module, &mut out)?;
        if name == entry_name {
            entry_idx = Some(out.funcs.len() as u32);
        }
        let mir_fn = lower_function(
            module,
            &mut out,
            di,
            body_id,
            sig,
            source_snapshot,
            &def_map,
            name.clone(),
        )?;
        out.funcs.push(mir_fn);
    }
    let entry = entry_idx.ok_or_else(|| {
        AdapterError::inv(
            "lower.entry",
            format!("no function named `{}` in module", entry_name),
        )
    })?;
    out.entry = entry;
    Ok(out)
}

fn scalar_shape_of_use(u: &hir::TypeUse, m: &hir::Module) -> Result<ScalarShape, AdapterError> {
    let s = match u {
        hir::TypeUse::Builtin(b) => match b {
            hir::BuiltinType::I32 => Some(ScalarShape::I32),
            hir::BuiltinType::Bool => Some(ScalarShape::Bool),
            _ => None,
        },
        hir::TypeUse::Ref(tid) => match m.types.get(tid.0 as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::I32) => Some(ScalarShape::I32),
            Some(hir::TypeKind::Bool) => Some(ScalarShape::Bool),
            _ => None,
        },
    };
    s.ok_or_else(|| {
        AdapterError::uns(
            "lower.type-use",
            format!("type use {:?} is not an i32/bool scalar", u),
        )
    })
}

fn scalar_type_idx_of(
    u: &hir::TypeUse,
    m: &hir::Module,
    out: &mut MirModule,
) -> Result<TypeIdx, AdapterError> {
    match scalar_shape_of_use(u, m)? {
        ScalarShape::I32 => Ok(out.intern_type(MirType::I32)),
        ScalarShape::Bool => Ok(out.intern_type(MirType::Bool)),
    }
}

fn fn_type_index(
    ty: hir::TypeId,
    m: &hir::Module,
    out: &mut MirModule,
) -> Result<TypeIdx, AdapterError> {
    let td = m.types.get(ty.0 as usize).ok_or_else(|| {
        AdapterError::inv("lower.type-ref", format!("type id {} out of range", ty.0))
    })?;
    match &td.kind {
        hir::TypeKind::Function(sig) => {
            let mut params = Vec::with_capacity(sig.params.len());
            for p in &sig.params {
                params.push(scalar_type_idx_of(p, m, out)?);
            }
            let result = scalar_type_idx_of(&sig.result, m, out)?;
            Ok(out.intern_type(MirType::Fn(FnSig { params, result })))
        }
        _ => Err(AdapterError::inv(
            "lower.fn-type",
            format!("type `{}` is not a function type", td.id_text),
        )),
    }
}

// ---------------------------------------------------------------------------
// Function lowering
// ---------------------------------------------------------------------------

/// Where a segment hands control when it completes without its own
/// terminator: a concrete block (loop latch / inherited continuation) or
/// the ON-DEMAND after-if join (F798-R7-05).
#[derive(Clone, Copy)]
enum Exit {
    Block(BlockId),
    Join(usize),
}

/// Where a segment starts: a concrete block, or the after-if join
/// allocated on first need — a join both of whose arms terminated is
/// never created, so no unreachable unwired block remains.
#[derive(Clone, Copy)]
enum Start {
    Block(BlockId),
    JoinOnDemand(usize),
}

/// One pending HIR-block segment: lower stmts from `stmt_idx` into the
/// start block; when the segment completes without its own terminator,
/// jump to `exit` (after-if join / loop latch / inherited continuation).
struct Task {
    hir_block: hir::BlockId,
    stmt_idx: usize,
    mir_block: Start,
    exit: Option<Exit>,
}

struct FnLower<'a> {
    hir_module: &'a hir::Module,
    body: &'a hir::Body,
    def_map: &'a BTreeMap<u32, CalleeRef>,
    out: &'a mut MirModule,
    slot_of_local: Vec<SlotId>,
    base_slots: u32,
    extra_slots: Vec<MirSlot>,
    t_i32: TypeIdx,
    t_bool: TypeIdx,
    blocks: Vec<MirBlock>,
    /// Current open MIR block cursor.
    cur: BlockId,
    /// Whether the current segment already emitted its terminator.
    terminated: bool,
    /// (header, exit) MIR blocks per HIR LoopId.
    loop_targets: Vec<(BlockId, BlockId)>,
    func_index: u32,
    /// Running flat instruction counter — emission order equals block-major
    /// order because blocks are created in order and never revisited.
    flat: u32,
    spans: Vec<(u32, SpanInfo)>,
    /// Whether block 0 got a real (non-placeholder) terminator.
    root_wired: bool,
    /// After-if join cells (F798-R7-05): `None` until some arm falls
    /// through (or a continuation with remaining statements starts) —
    /// a both-arms-terminated `if` never materializes its join.
    pending_joins: Vec<Option<BlockId>>,
}

impl<'a> FnLower<'a> {
    fn new_block(&mut self) -> BlockId {
        self.blocks.push(MirBlock {
            instrs: vec![],
            // Placeholder; every created block is the target of exactly one
            // task and receives a real terminator before the walk ends.
            term: Terminator::Jump { target: u32::MAX },
        });
        (self.blocks.len() - 1) as BlockId
    }

    fn emit(&mut self, instr: MirInstr, span: Option<SpanInfo>) {
        self.blocks[self.cur as usize].instrs.push(instr);
        if let Some(sp) = span {
            self.spans.push((self.flat, sp));
        }
        self.flat += 1;
    }

    /// Allocate (once) the join block of an on-demand join cell.
    fn join_block(&mut self, slot: usize) -> BlockId {
        if let Some(b) = self.pending_joins[slot] {
            return b;
        }
        let b = self.new_block();
        self.pending_joins[slot] = Some(b);
        b
    }

    fn set_term(&mut self, term: Terminator) {
        self.blocks[self.cur as usize].term = term;
        self.terminated = true;
        if self.cur == 0 {
            self.root_wired = true;
        }
    }

    fn temp(&mut self, shape: ScalarShape) -> SlotId {
        let ty = match shape {
            ScalarShape::I32 => self.t_i32,
            ScalarShape::Bool => self.t_bool,
        };
        let idx = self.base_slots + self.extra_slots.len() as SlotId;
        self.extra_slots.push(MirSlot {
            kind: SlotKind::Temp,
            ty,
            name: None,
        });
        idx
    }

    fn const_i32(&mut self, v: i32) -> u32 {
        self.out.intern_const(MirConst::I32(v))
    }

    fn const_bool(&mut self, v: bool) -> u32 {
        self.out.intern_const(MirConst::Bool(v))
    }

    fn expr_shape(&self, eid: hir::ExprId) -> Result<ScalarShape, AdapterError> {
        let e = self.body.exprs.get(eid.0 as usize).ok_or_else(|| {
            AdapterError::inv("lower.expr-ref", format!("expr {} out of range", eid.0))
        })?;
        scalar_shape_of_use(&e.ty, self.hir_module)
    }

    fn span(&self, eid: hir::ExprId) -> Option<SpanInfo> {
        self.body.exprs.get(eid.0 as usize).map(|e| SpanInfo {
            func: self.func_index,
            start: e.span.start as u32,
            end: e.span.end as u32,
        })
    }

    /// Evaluate one expression into a slot (existing local slot or fresh
    /// temp), emitting into the current block. Short-circuit `Logical`
    /// opens new blocks; the cursor lands on the join.
    fn eval_expr(&mut self, eid: hir::ExprId, depth: u32) -> Result<SlotId, AdapterError> {
        if depth > MAX_EXPR_DEPTH {
            return Err(AdapterError::uns(
                "mir.lower.depth",
                format!(
                    "expression nesting exceeds the lowering ceiling {}",
                    MAX_EXPR_DEPTH
                ),
            ));
        }
        let shape = self.expr_shape(eid)?;
        let sp = self.span(eid);
        let expr = &self.body.exprs[eid.0 as usize];
        match &expr.kind {
            hir::ExprKind::ReadLocal { local } => Ok(self.slot_of_local[local.0 as usize]),
            hir::ExprKind::Constant { value } => {
                let dst = self.temp(shape);
                let c = self.const_i32(*value);
                self.emit(MirInstr::Const { dst, value: c }, sp);
                Ok(dst)
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let l = self.eval_expr(*lhs, depth + 1)?;
                let r = self.eval_expr(*rhs, depth + 1)?;
                let dst = self.temp(shape);
                let mop = map_binop(*op)?;
                self.emit(
                    MirInstr::Bin {
                        op: mop,
                        dst,
                        lhs: l,
                        rhs: r,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::Unary { op, operand } => {
                let v = self.eval_expr(*operand, depth + 1)?;
                let dst = self.temp(shape);
                let mop = match op {
                    hir::UnaryOp::NegI32 => UnOp::NegI32,
                    hir::UnaryOp::NotBool => UnOp::NotBool,
                };
                self.emit(
                    MirInstr::Un {
                        op: mop,
                        dst,
                        operand: v,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                // Short-circuit: the result slot is defined in BOTH arms.
                let l = self.eval_expr(*lhs, depth + 1)?;
                let result = self.temp(ScalarShape::Bool);
                let rhs_b = self.new_block();
                let skip_b = self.new_block();
                let join = self.new_block();
                let (then_b, else_b) = match op {
                    hir::LogicOp::And => (rhs_b, skip_b),
                    hir::LogicOp::Or => (skip_b, rhs_b),
                };
                self.set_term(Terminator::Branch {
                    cond: l,
                    then: then_b,
                    else_: else_b,
                });
                // Skipped arm: result = the short-circuit value.
                self.cur = skip_b;
                self.terminated = false;
                let c = self.const_bool(matches!(op, hir::LogicOp::Or));
                self.emit(
                    MirInstr::Const {
                        dst: result,
                        value: c,
                    },
                    sp,
                );
                self.set_term(Terminator::Jump { target: join });
                // Taken arm: result = rhs.
                self.cur = rhs_b;
                self.terminated = false;
                let r = self.eval_expr(*rhs, depth + 1)?;
                let rsp = self.span(*rhs);
                self.emit(
                    MirInstr::Copy {
                        dst: result,
                        src: r,
                    },
                    rsp,
                );
                self.set_term(Terminator::Jump { target: join });
                self.cur = join;
                self.terminated = false;
                Ok(result)
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                // Source-order evaluation, then binding-map reorder.
                let mut arg_slots = Vec::with_capacity(eval_args.len());
                for a in eval_args {
                    arg_slots.push(self.eval_expr(*a, depth + 1)?);
                }
                let callee_ref = self.def_map.get(&callee.0).cloned().ok_or_else(|| {
                    AdapterError::inv(
                        "lower.callee",
                        format!("callee def {} out of range", callee.0),
                    )
                })?;
                if let CalleeRef::Import(imp) = callee_ref {
                    // Test marks: trace event + frozen i32 result.
                    let symbol = self.out.imports[imp as usize].symbol.clone();
                    let dst = self.temp(ScalarShape::I32);
                    let (event, value) = match symbol.as_str() {
                        "hir.test.mark_a" => (TraceEvent::MarkA, 1i32),
                        "hir.test.mark_b" => (TraceEvent::MarkB, 2),
                        other => {
                            return Err(AdapterError::uns(
                                "lower.intrinsic",
                                format!("intrinsic `{}` is not in the av profile", other),
                            ));
                        }
                    };
                    self.emit(MirInstr::Trace { event }, sp);
                    let c = self.const_i32(value);
                    self.emit(MirInstr::Const { dst, value: c }, sp);
                    return Ok(dst);
                }
                // The callee signature comes from the HIR def table —
                // forward and self references resolve before any body of
                // that function has been lowered.
                let callee_def = *callee;
                let hir_ty = self
                    .hir_module
                    .defs
                    .get(callee_def.0 as usize)
                    .and_then(|d| d.kind.fn_type())
                    .ok_or_else(|| {
                        AdapterError::inv(
                            "lower.callee-sig",
                            "callee lacks a function type".to_string(),
                        )
                    })?;
                let sig_idx = fn_type_index(hir_ty, self.hir_module, self.out)?;
                let n_params = self
                    .out
                    .sig_of(sig_idx)
                    .map(|s| s.params.len())
                    .unwrap_or(0);
                let mut actuals: Vec<SlotId> = vec![0; n_params];
                for b in bindings {
                    let pos = b.param as usize;
                    if pos >= n_params {
                        return Err(AdapterError::inv(
                            "lower.binding",
                            format!("call binding param {} out of range", b.param),
                        ));
                    }
                    actuals[pos] = arg_slots[b.arg as usize];
                }
                let dst = self.temp(shape);
                self.emit(
                    MirInstr::Call {
                        callee: callee_ref,
                        dst,
                        args: actuals,
                    },
                    sp,
                );
                Ok(dst)
            }
            other => Err(AdapterError::uns(
                "lower.expr",
                format!(
                    "expression kind `{}` is outside the av profile",
                    expr_kind_name(other)
                ),
            )),
        }
    }
}

fn map_binop(op: hir::BinOp) -> Result<BinOp, AdapterError> {
    Ok(match op {
        hir::BinOp::AddI32 => BinOp::AddI32,
        hir::BinOp::SubI32 => BinOp::SubI32,
        hir::BinOp::MulI32 => BinOp::MulI32,
        hir::BinOp::DivI32 => BinOp::DivI32,
        hir::BinOp::RemI32 => BinOp::RemI32,
        hir::BinOp::LtI32 => BinOp::LtI32,
        hir::BinOp::LeI32 => BinOp::LeI32,
        hir::BinOp::GtI32 => BinOp::GtI32,
        hir::BinOp::GeI32 => BinOp::GeI32,
        hir::BinOp::Eq => BinOp::EqI32,
        hir::BinOp::Ne => BinOp::NeI32,
    })
}

fn expr_kind_name(k: &hir::ExprKind) -> &'static str {
    use hir::ExprKind::*;
    match k {
        ReadLocal { .. } => "read-local",
        Constant { .. } => "constant",
        Binary { .. } => "binary",
        Unary { .. } => "unary",
        Logical { .. } => "logical",
        Call { .. } => "call",
        StrLit { .. } => "str-lit",
        CharLit { .. } => "char-lit",
        TextLen { .. } => "text-len",
        TextLenBytes { .. } => "text-len-bytes",
        TextCharAt { .. } => "text-char-at",
        TextSlice { .. } => "text-slice",
        TextConcat { .. } => "text-concat",
        TextClone { .. } => "text-clone",
        TextToOwned { .. } => "text-to-owned",
        TextToBorrowed { .. } => "text-to-borrowed",
        CharToCodepoint { .. } => "char-to-codepoint",
        CharFromCodepoint { .. } => "char-from-codepoint",
        RecordCtor { .. } => "record-ctor",
        FieldGet { .. } => "field-get",
        BorrowView { .. } => "borrow-view",
        BorrowMut { .. } => "borrow-mut",
        RecordClone { .. } => "record-clone",
        Move { .. } => "move",
    }
}

fn lower_function(
    hir_module: &hir::Module,
    out: &mut MirModule,
    def_index: u32,
    body_id: hir::BodyId,
    sig: TypeIdx,
    source_snapshot: u64,
    def_map: &BTreeMap<u32, CalleeRef>,
    name: String,
) -> Result<MirFunction, AdapterError> {
    let body = hir_module.bodies.get(body_id.0 as usize).ok_or_else(|| {
        AdapterError::inv("lower.body-ref", format!("body {} out of range", body_id.0))
    })?;

    let t_i32 = out.intern_type(MirType::I32);
    let t_bool = out.intern_type(MirType::Bool);

    // Base slots: params lead in signature order (HIR verify guarantees the
    // param locals lead the local table in order), then the other locals.
    let mut slots: Vec<MirSlot> = Vec::with_capacity(body.locals.len());
    let mut slot_of_local = vec![0u32; body.locals.len()];
    for (li, l) in body.locals.iter().enumerate() {
        let ty = scalar_type_idx_of(&l.ty, hir_module, out)?;
        let idx = slots.len() as SlotId;
        slots.push(MirSlot {
            kind: if l.role == hir::LocalRole::Param {
                SlotKind::Param
            } else {
                SlotKind::Local
            },
            ty,
            name: Some(l.name.clone()),
        });
        slot_of_local[li] = idx;
    }
    let base_slots = slots.len() as u32;

    let mut lower = FnLower {
        hir_module,
        body,
        def_map,
        out,
        slot_of_local,
        base_slots,
        extra_slots: vec![],
        t_i32,
        t_bool,
        blocks: vec![],
        cur: 0,
        terminated: false,
        loop_targets: vec![(0, 0); body.loops.len()],
        func_index: def_index,
        flat: 0,
        spans: vec![],
        root_wired: false,
        pending_joins: vec![],
    };
    let entry_b = lower.new_block();
    debug_assert_eq!(entry_b, 0);

    let mut stack: Vec<Task> = vec![Task {
        hir_block: body.entry,
        stmt_idx: 0,
        mir_block: Start::Block(0),
        exit: None,
    }];
    while let Some(mut task) = stack.pop() {
        // F798-R7-05: an after-if continuation whose join was never
        // allocated (both arms terminated) and that has no remaining
        // statements is dead bookkeeping — skipping it leaves no
        // unreachable unwired block.
        let start_block = match task.mir_block {
            Start::Block(b) => b,
            Start::JoinOnDemand(slot) => {
                let has_stmts = body
                    .blocks
                    .get(task.hir_block.0 as usize)
                    .map(|b| task.stmt_idx < b.stmts.len())
                    .unwrap_or(false);
                if lower.pending_joins[slot].is_none() && !has_stmts {
                    continue;
                }
                lower.join_block(slot)
            }
        };
        lower.cur = start_block;
        lower.terminated = false;
        loop {
            let block = body.blocks.get(task.hir_block.0 as usize).ok_or_else(|| {
                AdapterError::inv(
                    "lower.block-ref",
                    format!("block {} out of range", task.hir_block.0),
                )
            })?;
            if task.stmt_idx >= block.stmts.len() {
                break;
            }
            let stmt = &block.stmts[task.stmt_idx];
            task.stmt_idx += 1;
            match stmt {
                hir::Stmt::Let { local, value } => {
                    let v = lower.eval_expr(*value, 0)?;
                    let dst = lower.slot_of_local[local.0 as usize];
                    let sp = lower.span(*value);
                    lower.emit(MirInstr::Copy { dst, src: v }, sp);
                }
                hir::Stmt::Assign { place, value } => {
                    let p = body.places.get(place.0 as usize).ok_or_else(|| {
                        AdapterError::inv(
                            "lower.place-ref",
                            format!("place {} out of range", place.0),
                        )
                    })?;
                    if !matches!(p.kind, hir::PlaceKind::Local) {
                        return Err(AdapterError::uns(
                            "lower.place-kind",
                            "field-path assignment is record-domain, outside the av profile"
                                .to_string(),
                        ));
                    }
                    let dst = lower.slot_of_local[p.local.0 as usize];
                    let v = lower.eval_expr(*value, 0)?;
                    let sp = lower.span(*value);
                    lower.emit(MirInstr::Copy { dst, src: v }, sp);
                }
                hir::Stmt::Return { value } => {
                    let v = lower.eval_expr(*value, 0)?;
                    lower.set_term(Terminator::Return { value: v });
                    break; // HIR verify: nothing follows a terminator
                }
                hir::Stmt::If { cond, then, els } => {
                    let c = lower.eval_expr(*cond, 0)?;
                    let then_b = lower.new_block();
                    let then_exit;
                    match els {
                        Some(else_hir) => {
                            // F798-R7-05: with both arms present the join
                            // is ON DEMAND — `if a { return } else { return }`
                            // never materializes an unreachable join block.
                            let else_b = lower.new_block();
                            let slot = lower.pending_joins.len();
                            lower.pending_joins.push(None);
                            then_exit = Some(Exit::Join(slot));
                            lower.set_term(Terminator::Branch {
                                cond: c,
                                then: then_b,
                                else_: else_b,
                            });
                            // LIFO: continuation processed last.
                            stack.push(Task {
                                hir_block: task.hir_block,
                                stmt_idx: task.stmt_idx,
                                mir_block: Start::JoinOnDemand(slot),
                                exit: task.exit,
                            });
                            stack.push(Task {
                                hir_block: *else_hir,
                                stmt_idx: 0,
                                mir_block: Start::Block(else_b),
                                exit: Some(Exit::Join(slot)),
                            });
                        }
                        None => {
                            // No else: the branch's false edge targets the
                            // join, so it is required eagerly.
                            let join = lower.new_block();
                            then_exit = Some(Exit::Block(join));
                            lower.set_term(Terminator::Branch {
                                cond: c,
                                then: then_b,
                                else_: join,
                            });
                            stack.push(Task {
                                hir_block: task.hir_block,
                                stmt_idx: task.stmt_idx,
                                mir_block: Start::Block(join),
                                exit: task.exit,
                            });
                        }
                    }
                    stack.push(Task {
                        hir_block: *then,
                        stmt_idx: 0,
                        mir_block: Start::Block(then_b),
                        exit: then_exit,
                    });
                    break;
                }
                hir::Stmt::Loop {
                    id,
                    body: loop_body,
                } => {
                    let header = lower.new_block();
                    let body_b = lower.new_block();
                    let exit = lower.new_block();
                    lower.loop_targets[id.0 as usize] = (header, exit);
                    lower.set_term(Terminator::Jump { target: header });
                    lower.cur = header;
                    lower.terminated = false;
                    lower.set_term(Terminator::Jump { target: body_b });
                    stack.push(Task {
                        hir_block: task.hir_block,
                        stmt_idx: task.stmt_idx,
                        mir_block: Start::Block(exit),
                        exit: task.exit,
                    });
                    stack.push(Task {
                        hir_block: *loop_body,
                        stmt_idx: 0,
                        mir_block: Start::Block(body_b),
                        exit: Some(Exit::Block(header)),
                    });
                    break;
                }
                hir::Stmt::Break { target } => {
                    let (_, exit) = lower.loop_targets[target.0 as usize];
                    lower.set_term(Terminator::Jump { target: exit });
                    break;
                }
                hir::Stmt::Continue { target } => {
                    let (header, _) = lower.loop_targets[target.0 as usize];
                    lower.set_term(Terminator::Jump { target: header });
                    break;
                }
                hir::Stmt::Drop { local: _ } => {
                    // Scalar locals own no resources; programmatic drops are
                    // no-ops in this profile (cleanup is compiler-scheduled).
                }
            }
        }
        // Segment completed without its own terminator: hand control to the
        // recorded exit (after-if join / loop latch / inherited).
        if !lower.terminated {
            match task.exit {
                Some(Exit::Block(b)) => {
                    lower.set_term(Terminator::Jump { target: b });
                }
                Some(Exit::Join(slot)) => {
                    let j = lower.join_block(slot);
                    lower.set_term(Terminator::Jump { target: j });
                }
                // exit == None (root): the placeholder stays; rejected below.
                None => {}
            }
        }
    }

    // Sanity: every block carries a real terminator; only the root may be
    // left open (missing return path — checked input cannot reach this).
    for (bi, b) in lower.blocks.iter().enumerate() {
        if matches!(b.term, Terminator::Jump { target: u32::MAX }) {
            if bi == 0 && !lower.root_wired {
                return Err(AdapterError::inv(
                    "lower.missing-return",
                    "entry block completed without a return path".to_string(),
                ));
            }
            return Err(AdapterError::inv(
                "lower.unwired-block",
                format!("block {} never received a terminator", bi),
            ));
        }
    }

    // Spans are appended in emission order == flat order.
    debug_assert!(lower.spans.windows(2).all(|w| w[0].0 < w[1].0));

    let owner = lower.out.owner.clone();
    let mut slots_final = slots;
    slots_final.append(&mut lower.extra_slots);
    Ok(MirFunction {
        owner,
        name,
        sig,
        slots: slots_final,
        blocks: lower.blocks,
        entry: 0,
        source_snapshot,
        spans: lower.spans,
    })
}
