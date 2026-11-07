//! PLAN-904 (AX-C) collections lowering: HIR (collections domain) → the
//! typed Rust IR of [`crate::lower`], driven ENTIRELY by the verifier's
//! collections body facts (own/consume/exit responsibility — plan §5.3/§5.4).
//!
//! The lowering never re-derives ownership: every free — normal exits,
//! assignment replacement, and the error path of every fallible node —
//! comes from [`ace_core::collections_facts::CollectionsBodyFacts`], and
//! the Rust compiler cross-checks the schedule (a wrong schedule is a
//! generated-code compile error, not a silent leak). Representation
//! follows the T-02 freeze: `A2xList<E>` backing, nominal `A2xv<Name>`
//! enums, explicit frees only (no Drop, no implicit Clone), checked
//! views through `a2x_coll_list_borrow_elem`, cooperative polling at
//! loop back-edges and call frames, ctor/call/clone/list-node/Str-byte
//! billing inside the runtime helpers.

use ace_a2x_core::{A2xReject, FnFacts, SemanticView, Ty};
use ace_core::collections_facts::{CollectionsBodyFacts, FreeRef, OwnedKind};
use ace_core::hir;

use crate::lower::{
    rust_fn_name, rust_record_name, rust_variant_name, sanitize, CmpOp, RExpr, RStmt, RustFn,
    RustParam, RustProgram, RustVariant,
};

/// Rust type text of a view type (collections surface).
pub fn rust_ty_str(view: &SemanticView<'_>, ty: Ty) -> String {
    match ty {
        Ty::I32 => "i32".into(),
        // PLAN-961: numeric names are their Rust spellings.
        Ty::I8 => "i8".into(),
        Ty::U8 => "u8".into(),
        Ty::U32 => "u32".into(),
        Ty::I64 => "i64".into(),
        Ty::U64 => "u64".into(),
        Ty::F32 => "f32".into(),
        Ty::F64 => "f64".into(),
        Ty::Bool => "bool".into(),
        Ty::Char => "char".into(),
        Ty::Str => "&str".into(),
        Ty::StrOwned => "A2xStr".into(),
        Ty::Record(id) => view
            .record(id)
            .map(|r| rust_record_name(&r.name))
            .unwrap_or_else(|| "Rec".into()),
        Ty::Variant(id) => view
            .variant(id)
            .map(|v| rust_variant_name(&v.name))
            .unwrap_or_else(|| "A2xv".into()),
        Ty::List(id) => view
            .list_elem(id)
            .ok()
            .map(|e| format!("A2xList<{}>", rust_ty_str(view, e)))
            .unwrap_or_else(|| "A2xList".into()),
        Ty::Unit => "()".into(),
    }
}

/// Lower a whole collections module and select its entry by source name.
pub fn lower_collections_program(
    view: &SemanticView<'_>,
    entry_name: &str,
) -> Result<RustProgram, A2xReject> {
    let entry = view.entry(entry_name)?;
    let mut p = lower_coll_functions(view)?;
    p.entry = rust_fn_name(entry.source_name);
    Ok(p)
}

/// Lower every function of a collections view WITHOUT entry selection
/// (the store's per-fragment produce path needs the function table only).
pub fn lower_coll_functions(view: &SemanticView<'_>) -> Result<RustProgram, A2xReject> {
    let mut functions = Vec::new();
    for f in view.functions() {
        functions.push(lower_coll_fn(view, f)?);
    }
    let mut marks: Vec<&'static str> = view.trace_symbols().into_iter().collect();
    marks.sort_unstable();
    let records = crate::lower::lower_records(view)?;
    let variants = lower_coll_variants(view);
    Ok(RustProgram {
        functions,
        entry: String::new(),
        marks,
        records,
        clones: Vec::new(),
        borrowed: false,
        variants,
        coll: true,
    })
}

/// Lower one function of the collections channel.
pub fn lower_coll_fn(view: &SemanticView<'_>, f: &FnFacts<'_>) -> Result<RustFn, A2xReject> {
    let facts = view.body_facts(f).ok_or_else(|| {
        A2xReject::new(
            "a2x.internal",
            format!(
                "collections lowering requires the verifier's body facts for `{}`",
                f.source_name
            ),
        )
    })?;
    let mut w = CollLower {
        view,
        f,
        facts,
        locals: Vec::new(),
    };
    // param names follow the HIR local table (l<LocalId>_<name>) so body
    // references and the signature agree on one naming
    let mut named_params = Vec::new();
    let mut param_locals: Vec<(u32, &hir::LocalDef)> = f
        .body
        .locals
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.role, hir::LocalRole::Param))
        .map(|(i, l)| (i as u32, l))
        .collect();
    param_locals.sort_by_key(|(_, l)| l.index.unwrap_or(u32::MAX));
    for (p, (idx, l)) in f.params.iter().zip(param_locals.iter()) {
        named_params.push(RustParam {
            name: local_name(&l.name, *idx),
            ty: p.ty,
            mode: p.mode,
        });
    }
    w.locals = f.body.locals.clone();
    let mut body: Vec<RStmt> = Vec::new();
    // callee-frame work charge (billing table: 4 WU per call frame); its
    // failure path frees nothing (the frame is empty at entry)
    body.push(RStmt::EvalExpr {
        expr: RExpr::Wrap {
            inner: Box::new(RExpr::CollCall {
                func: "a2x_coll_charge_call".into(),
                turbo: None,
                args: Vec::new(),
            }),
            frees: Vec::new(),
        },
    });
    body.extend(w.block(f.body.entry)?);
    Ok(RustFn {
        name: rust_fn_name(f.source_name),
        params: named_params,
        result: f.result,
        origin: f.origin,
        body,
    })
}

fn local_name(name: &str, id: u32) -> String {
    format!("l{}_{}", id, sanitize(name))
}

struct CollLower<'a, 'v> {
    view: &'v SemanticView<'a>,
    f: &'v FnFacts<'a>,
    facts: &'v CollectionsBodyFacts,
    locals: Vec<hir::LocalDef>,
}

impl<'a, 'v> CollLower<'a, 'v> {
    fn local_name_of(&self, id: hir::LocalId) -> String {
        match self.locals.get(id.0 as usize) {
            Some(l) => local_name(&l.name, id.0),
            None => format!("l{}", id.0),
        }
    }

    fn local_ty(&self, id: hir::LocalId) -> Result<Ty, A2xReject> {
        let l = self
            .locals
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "local index out of range"))?;
        self.view.resolve_use_ty(l.ty)
    }

    /// The free-helper call for an owned kind.
    fn free_call(&self, kind: OwnedKind) -> String {
        match kind {
            OwnedKind::Str => "a2x_str_free".into(),
            OwnedKind::List => "a2x_coll_list_free".into(),
            OwnedKind::Deep(tid) => {
                let name = if let Some(r) = self.view.record(tid.0) {
                    rust_record_name(&r.name)
                } else if let Some(v) = self.view.variant(tid.0) {
                    rust_variant_name(&v.name)
                } else {
                    format!("T{}", tid.0)
                };
                format!("a2x_coll_free_{}", name)
            }
        }
    }

    /// Render one scheduled free set as statements.
    fn free_stmts(&self, frees: &[FreeRef]) -> Vec<RStmt> {
        frees
            .iter()
            .map(|r| match r {
                FreeRef::Local(l) => {
                    let ty = self.local_ty(hir::LocalId(*l)).ok();
                    let call = ty
                        .and_then(|t| self.owned_kind_of_ty(t))
                        .map(|k| self.free_call(k))
                        .unwrap_or_else(|| "a2x_str_free".into());
                    RStmt::FreeSlot {
                        name: self.local_name_of(hir::LocalId(*l)),
                        call,
                    }
                }
                FreeRef::Temp(e, kind) => RStmt::FreeSlot {
                    name: format!("e{}", e),
                    call: self.free_call(*kind),
                },
            })
            .collect()
    }

    fn owned_kind_of_ty(&self, ty: Ty) -> Option<OwnedKind> {
        match ty {
            Ty::StrOwned => Some(OwnedKind::Str),
            Ty::List(_) => Some(OwnedKind::List),
            Ty::Record(id) => {
                let rec = self.view.record(id)?;
                rec.resource
                    .then(|| OwnedKind::Deep(hir::TypeId(id)))
                    .or(if rec.resource {
                        None
                    } else {
                        Some(OwnedKind::Deep(hir::TypeId(id)))
                    })
            }
            Ty::Variant(id) => {
                let v = self.view.variant(id)?;
                let owns = v.arms.iter().any(|a| {
                    a.payload
                        .as_ref()
                        .and_then(|p| self.owned_kind_of_ty(*p))
                        .is_some()
                });
                owns.then(|| OwnedKind::Deep(hir::TypeId(id)))
            }
            _ => None,
        }
    }

    fn err_frees_of(&self, expr_id: u32) -> Vec<(String, String)> {
        self.facts
            .err_frees
            .get(&expr_id)
            .map(|set| {
                set.iter()
                    .map(|r| match r {
                        FreeRef::Local(l) => {
                            let ty = self.local_ty(hir::LocalId(*l)).ok();
                            let call = ty
                                .and_then(|t| self.owned_kind_of_ty(t))
                                .map(|k| self.free_call(k))
                                .unwrap_or_else(|| "a2x_str_free".into());
                            (self.local_name_of(hir::LocalId(*l)), call)
                        }
                        FreeRef::Temp(e, kind) => (format!("e{}", e), self.free_call(*kind)),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Wrap a fallible expression with its verifier-scheduled error frees.
    fn wrap(&self, id: hir::ExprId, inner: RExpr) -> RExpr {
        RExpr::Wrap {
            inner: Box::new(inner),
            frees: self.err_frees_of(id.0),
        }
    }

    /// The consuming-variant wrap (push/insert/set): the value operand has
    /// MOVED into the runtime helper, which owns and frees the uncommitted
    /// input on its own failure path — the operand temp itself drops out
    /// of the caller-side error frees.
    fn wrap_consume(&self, id: hir::ExprId, value: hir::ExprId, inner: RExpr) -> RExpr {
        let frees = self
            .err_frees_of(id.0)
            .into_iter()
            .filter(|(name, _)| name != &format!("e{}", value.0))
            .collect();
        RExpr::Wrap {
            inner: Box::new(inner),
            frees,
        }
    }

    fn block(&mut self, block: hir::BlockId) -> Result<Vec<RStmt>, A2xReject> {
        let mut out: Vec<RStmt> = Vec::new();
        let stmts = self.f.body.blocks[block.0 as usize].stmts.clone();
        for (si, stmt) in stmts.iter().enumerate() {
            match stmt {
                hir::Stmt::Let { local, value } => {
                    // an owned-position literal promotes (a2x_str_from_lit)
                    let v = if is_lit(self.f.body, *value) && owned_target(self, local.0) {
                        self.owned_or_plain_child(*value, &mut out)?
                    } else {
                        self.expr(*value, &mut out)?
                    };
                    let l = self.locals.get(local.0 as usize).ok_or_else(|| {
                        A2xReject::new("a2x.internal", "local index out of range")
                    })?;
                    let ty = self.view.resolve_use_ty(l.ty)?;
                    out.push(RStmt::CollLet {
                        name: self.local_name_of(*local),
                        ty_str: rust_ty_str(self.view, ty),
                        mutable: self.local_needs_mut(local.0),
                        init: v,
                    });
                }
                hir::Stmt::Return { value } => {
                    // an owned-result literal promotes at the return slot
                    let owned_ret = self
                        .view
                        .resolve_use_ty(self.f.body.exprs[value.0 as usize].ty)
                        .map(|t| matches!(t, Ty::StrOwned))
                        .unwrap_or(false);
                    let v = if is_lit(self.f.body, *value) && owned_ret {
                        self.owned_or_plain_child(*value, &mut out)?
                    } else {
                        self.expr(*value, &mut out)?
                    };
                    if let Some(frees) = self.facts.exit_frees.get(&(block.0, Some(si))) {
                        out.extend(self.free_stmts(frees));
                    }
                    out.push(RStmt::Return { value: v });
                }
                hir::Stmt::Assign { place, value } => {
                    let p_pre = &self.f.body.places[place.0 as usize];
                    let owned_place = p_pre.kind == hir::PlaceKind::Local
                        && self
                            .local_ty(p_pre.local)
                            .map(|t| matches!(t, Ty::StrOwned))
                            .unwrap_or(false);
                    let v = if is_lit(self.f.body, *value) && owned_place {
                        self.owned_or_plain_child(*value, &mut out)?
                    } else {
                        self.expr(*value, &mut out)?
                    };
                    let p = &self.f.body.places[place.0 as usize];
                    if let hir::PlaceKind::Local = p.kind {
                        if let Some(old) = self.facts.replace_frees.get(&(block.0, si)) {
                            let ty = self.local_ty(p.local).ok();
                            let call = ty
                                .and_then(|t| self.owned_kind_of_ty(t))
                                .map(|k| self.free_call(k))
                                .unwrap_or_else(|| "a2x_str_free".into());
                            out.push(RStmt::FreeSlot {
                                name: self.local_name_of(p.local),
                                call,
                            });
                            let _ = old;
                        }
                    } else {
                        return Err(A2xReject::unsupported_place(
                            "collections field-path assignment is outside the surface",
                        ));
                    }
                    out.push(RStmt::Assign {
                        name: self.local_name_of(p.local),
                        value: v,
                    });
                }
                hir::Stmt::If { cond, then, els } => {
                    let c = self.expr(*cond, &mut out)?;
                    let t = self.block(*then)?;
                    let e = match els {
                        Some(eb) => Some(self.block(*eb)?),
                        None => None,
                    };
                    out.push(RStmt::If {
                        cond: c,
                        then: t,
                        els: e,
                    });
                }
                hir::Stmt::Loop { body: lb, .. } => {
                    let mut body_stmts = self.block(*lb)?;
                    // back-edge: per-iteration scoped frees (recorded at the
                    // body block's fall-through), then the cooperative poll
                    // whose error path frees the survivors too
                    if let Some(frees) = self.facts.exit_frees.get(&(lb.0, None)) {
                        body_stmts.extend(self.free_stmts(frees));
                    }
                    let mut poll_frees = self
                        .facts
                        .exit_frees
                        .get(&(lb.0, None))
                        .map(|f| {
                            f.iter()
                                .filter_map(|r| match r {
                                    FreeRef::Local(l) => {
                                        let ty = self.local_ty(hir::LocalId(*l)).ok()?;
                                        let k = self.owned_kind_of_ty(ty)?;
                                        Some((
                                            self.local_name_of(hir::LocalId(*l)),
                                            self.free_call(k),
                                        ))
                                    }
                                    _ => None,
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if let Some(surv) = self.facts.loop_survivors.get(&lb.0) {
                        for l in surv {
                            if let Some(ty) = self.local_ty(hir::LocalId(*l)).ok() {
                                if let Some(k) = self.owned_kind_of_ty(ty) {
                                    poll_frees.push((
                                        self.local_name_of(hir::LocalId(*l)),
                                        self.free_call(k),
                                    ));
                                }
                            }
                        }
                    }
                    body_stmts.push(RStmt::EvalExpr {
                        expr: RExpr::Wrap {
                            inner: Box::new(RExpr::CollCall {
                                func: "a2x_coll_poll".into(),
                                turbo: None,
                                args: Vec::new(),
                            }),
                            frees: poll_frees,
                        },
                    });
                    out.push(RStmt::Loop { body: body_stmts });
                }
                hir::Stmt::Break { .. } => {
                    if let Some(frees) = self.facts.exit_frees.get(&(block.0, Some(si))) {
                        out.extend(self.free_stmts(frees));
                    }
                    out.push(RStmt::Break);
                }
                hir::Stmt::Continue { .. } => {
                    if let Some(frees) = self.facts.exit_frees.get(&(block.0, Some(si))) {
                        out.extend(self.free_stmts(frees));
                    }
                    out.push(RStmt::Continue);
                }
                hir::Stmt::Eval { value } => {
                    let v = self.expr(*value, &mut out)?;
                    if !matches!(v, RExpr::UnitExpr) {
                        out.push(RStmt::EvalExpr { expr: v });
                    }
                }
                hir::Stmt::Drop { .. } => {
                    return Err(A2xReject::unsupported_stmt(
                        "explicit Drop is outside the collections lowering surface",
                    ))
                }
            }
        }
        Ok(out)
    }

    /// A reference to a local's value place (for &list / &mut list args).
    fn place_ref(&self, e: hir::ExprId, mut_: bool) -> Result<RExpr, A2xReject> {
        let inner = match &self.f.body.exprs[e.0 as usize].kind {
            hir::ExprKind::ReadLocal { local } => RExpr::Local(self.local_name_of(*local)),
            _ => {
                return Err(A2xReject::unsupported_expr(
                    "list operand must be a whole-local read",
                ))
            }
        };
        Ok(if mut_ {
            RExpr::RefMut {
                inner: Box::new(inner),
            }
        } else {
            RExpr::Ref {
                inner: Box::new(inner),
            }
        })
    }

    /// Whether an expression node produces an owned value that the facts
    /// model as a pending temp (mirrors `Walker::temp_kind`).
    fn temp_kind_of(&self, e: &hir::Expr) -> Option<OwnedKind> {
        match &e.kind {
            hir::ExprKind::TextConcat { .. }
            | hir::ExprKind::TextClone { .. }
            | hir::ExprKind::TextToOwned { .. } => Some(OwnedKind::Str),
            hir::ExprKind::ListNew { .. } => Some(OwnedKind::List),
            _ => self.owned_kind_of_expr_ty(e),
        }
    }

    fn owned_kind_of_expr_ty(&self, e: &hir::Expr) -> Option<OwnedKind> {
        self.view
            .resolve_use_ty(e.ty)
            .ok()
            .and_then(|t| self.owned_kind_of_ty(t))
    }

    /// Emit one expression; fallible/owned nodes materialize as named
    /// temps (`e<expr-id>`) so the verifier's free schedule can address
    /// them. `out` receives the side statements in evaluation order.
    fn expr(&mut self, id: hir::ExprId, out: &mut Vec<RStmt>) -> Result<RExpr, A2xReject> {
        let e = self
            .f
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
        let kind = e.kind.clone();
        match kind {
            hir::ExprKind::ReadLocal { local } => Ok(RExpr::Local(self.local_name_of(local))),
            hir::ExprKind::Constant { value } => Ok(RExpr::I32Lit(value)),
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let l = self.expr(lhs, out)?;
                let r = self.expr(rhs, out)?;
                match op {
                    hir::BinOp::AddI32 => self.checked_temp("a2x_add", vec![l, r], id, out),
                    hir::BinOp::SubI32 => self.checked_temp("a2x_sub", vec![l, r], id, out),
                    hir::BinOp::MulI32 => self.checked_temp("a2x_mul", vec![l, r], id, out),
                    hir::BinOp::DivI32 => self.checked_temp("a2x_div", vec![l, r], id, out),
                    hir::BinOp::RemI32 => self.checked_temp("a2x_rem", vec![l, r], id, out),
                    hir::BinOp::LtI32 => Ok(RExpr::Cmp {
                        op: CmpOp::Lt,
                        lhs: Box::new(l),
                        rhs: Box::new(r),
                    }),
                    hir::BinOp::LeI32 => Ok(RExpr::Cmp {
                        op: CmpOp::Le,
                        lhs: Box::new(l),
                        rhs: Box::new(r),
                    }),
                    hir::BinOp::GtI32 => Ok(RExpr::Cmp {
                        op: CmpOp::Gt,
                        lhs: Box::new(l),
                        rhs: Box::new(r),
                    }),
                    hir::BinOp::GeI32 => Ok(RExpr::Cmp {
                        op: CmpOp::Ge,
                        lhs: Box::new(l),
                        rhs: Box::new(r),
                    }),
                    hir::BinOp::Eq => Ok(RExpr::Cmp {
                        op: CmpOp::Eq,
                        lhs: Box::new(l),
                        rhs: Box::new(r),
                    }),
                    hir::BinOp::Ne => Ok(RExpr::Cmp {
                        op: CmpOp::Ne,
                        lhs: Box::new(l),
                        rhs: Box::new(r),
                    }),
                    // PLAN-961: typed numeric ops ride the shared
                    // num_bin matrix (checked helpers / native compares).
                    hir::BinOp::Num { op, ty } => {
                        let (v, _) = crate::lower::num_bin_expr(op, ty, l, r)?;
                        Ok(v)
                    }
                }
            }
            other_kind => self.expr_rest(id, e, other_kind, out),
        }
    }

    fn checked_temp(
        &self,
        func: &'static str,
        args: Vec<RExpr>,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<RExpr, A2xReject> {
        let name = format!("e{}", id.0);
        let m = self.temp_needs_mut(id);
        out.push(RStmt::CollLet {
            name: name.clone(),
            ty_str: "i32".into(),
            mutable: m,
            init: self.wrap(id, RExpr::Checked { func, args }),
        });
        Ok(RExpr::Local(name))
    }

    /// Whether this expr id appears in any error free set (its temp may
    /// be freed on an error path, so its binding must be `mut`).
    /// Whether a LOCAL appears in any free schedule (its binding needs
    /// `mut` for the explicit free, independent of source mutability).
    fn local_needs_mut(&self, local: u32) -> bool {
        let in_frees = self.facts.exit_frees.values().any(|set| {
            set.iter()
                .any(|r| matches!(r, FreeRef::Local(l) if *l == local))
        }) || self.facts.err_frees.values().any(|set| {
            set.iter()
                .any(|r| matches!(r, FreeRef::Local(l) if *l == local))
        }) || self.facts.replace_frees.values().any(|&l| l == local);
        in_frees
            || self
                .locals
                .get(local as usize)
                .map(|l| l.mutable)
                .unwrap_or(false)
    }

    fn temp_needs_mut(&self, id: hir::ExprId) -> bool {
        self.facts.err_frees.values().any(|set| {
            set.iter()
                .any(|r| matches!(r, FreeRef::Temp(e, _) if *e == id.0))
        })
    }

    /// Emit an infallible value into a typed temp (no Wrap).
    fn coll_temp_plain(
        &self,
        id: hir::ExprId,
        ty_str: String,
        value: RExpr,
        out: &mut Vec<RStmt>,
    ) -> Result<RExpr, A2xReject> {
        let name = format!("e{}", id.0);
        let m = self.temp_needs_mut(id);
        out.push(RStmt::CollLet {
            name: name.clone(),
            ty_str,
            mutable: m,
            init: value,
        });
        Ok(RExpr::Local(name))
    }

    /// Emit a fallible collections call into a typed temp.
    fn coll_temp(
        &self,
        id: hir::ExprId,
        ty_str: String,
        call: RExpr,
        out: &mut Vec<RStmt>,
    ) -> Result<RExpr, A2xReject> {
        let name = format!("e{}", id.0);
        let m = self.temp_needs_mut(id);
        out.push(RStmt::CollLet {
            name: name.clone(),
            ty_str,
            mutable: m,
            init: self.wrap(id, call),
        });
        Ok(RExpr::Local(name))
    }

    #[allow(clippy::too_many_lines)]
    fn expr_rest(
        &mut self,
        id: hir::ExprId,
        e: &hir::Expr,
        kind: hir::ExprKind,
        out: &mut Vec<RStmt>,
    ) -> Result<RExpr, A2xReject> {
        match kind {
            hir::ExprKind::Unary { op, operand } => {
                let o = self.expr(operand, out)?;
                match op {
                    hir::UnaryOp::NegI32 => self.checked_temp("a2x_neg", vec![o], id, out),
                    hir::UnaryOp::NotBool => Ok(RExpr::Not(Box::new(o))),
                    // PLAN-961: typed negation via the runtime helpers.
                    hir::UnaryOp::NegNumeric(bty) => {
                        let func = match bty {
                            hir::BuiltinType::I8 => "a2x_neg_i8",
                            hir::BuiltinType::I64 => "a2x_neg_i64",
                            hir::BuiltinType::F32 => "a2x_neg_f32",
                            hir::BuiltinType::F64 => "a2x_neg_f64",
                            other => {
                                return Err(A2xReject::unsupported_op(format!(
                                    "typed negation on {other:?}"
                                )))
                            }
                        };
                        self.checked_temp(func, vec![o], id, out)
                    }
                }
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                let l = self.expr(lhs, out)?;
                let mut rhs_stmts = Vec::new();
                let r = self.expr(rhs, &mut rhs_stmts)?;
                let block = crate::lower::RBlock {
                    stmts: rhs_stmts,
                    value: r,
                };
                Ok(match op {
                    hir::LogicOp::And => RExpr::AndSC {
                        lhs: Box::new(l),
                        rhs: Box::new(block),
                    },
                    hir::LogicOp::Or => RExpr::OrSC {
                        lhs: Box::new(l),
                        rhs: Box::new(block),
                    },
                })
            }
            hir::ExprKind::Call {
                callee, eval_args, ..
            } => {
                let mut args = Vec::new();
                for a in eval_args {
                    args.push(self.expr(a, out)?);
                }
                let target = callee.0 as usize;
                let func = if let Some(sym) = self.view.trace_symbol(target) {
                    let suffix = if sym == "hir.test.mark_a" { "a" } else { "b" };
                    format!("a2x_mark_{}", suffix)
                } else if let Some(t) = self.view.fn_by_def(target) {
                    rust_fn_name(t.source_name)
                } else if let Some(x) = self.view.external_by_def(target) {
                    rust_fn_name(&x.source_name)
                } else {
                    return Err(A2xReject::unsupported_expr("call target not admitted"));
                };
                let call = RExpr::Call { func, args };
                let ty = self.view.resolve_use_ty(e.ty)?;
                self.coll_temp(id, rust_ty_str(self.view, ty), call, out)
            }
            hir::ExprKind::StrLit { value } => {
                // borrowed literal; owned positions promote via the parent
                Ok(RExpr::StrLit(value))
            }
            hir::ExprKind::TextLen { operand } => {
                let o = self.operand_str_view(operand, out)?;
                self.coll_temp(
                    id,
                    "i32".into(),
                    RExpr::StrFn {
                        func: "a2x_str_len".into(),
                        args: vec![o],
                    },
                    out,
                )
            }
            hir::ExprKind::TextClone { operand } => {
                let o = self.operand_str_view(operand, out)?;
                self.coll_temp(
                    id,
                    "A2xStr".into(),
                    RExpr::StrFn {
                        func: "a2x_str_clone".into(),
                        args: vec![o],
                    },
                    out,
                )
            }
            hir::ExprKind::RecordCtor {
                record,
                eval_args,
                bindings,
            } => {
                // source-order argument temps first (owned/promoted values
                // materialize exactly where the facts model them)
                let mut arg_vals: Vec<RExpr> = Vec::new();
                for a in &eval_args {
                    arg_vals.push(self.owned_or_plain_child(*a, out)?);
                }
                // ctor charge (fallible; frees = the pending arg temps)
                out.push(RStmt::EvalExpr {
                    expr: self.wrap(
                        id,
                        RExpr::CollCall {
                            func: "a2x_coll_charge_ctor".into(),
                            turbo: None,
                            args: Vec::new(),
                        },
                    ),
                });
                let rec = self
                    .view
                    .record(record.0)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "record out of range"))?;
                let rname = rust_record_name(&rec.name);
                let mut fields = Vec::new();
                for b in &bindings {
                    let fname = rec
                        .fields
                        .get(b.field as usize)
                        .map(|f| format!("f_{}", sanitize(&f.name)))
                        .unwrap_or_else(|| format!("f{}", b.field));
                    let v = arg_vals
                        .get(b.arg as usize)
                        .cloned()
                        .unwrap_or(RExpr::I32Lit(0));
                    fields.push((fname, v));
                }
                let ctor = RExpr::RecCtor {
                    record: rname,
                    fields,
                };
                if self.temp_kind_of(e).is_some() {
                    let ty = self.view.resolve_use_ty(e.ty)?;
                    self.coll_temp_plain(id, rust_ty_str(self.view, ty), ctor, out)
                } else {
                    Ok(ctor)
                }
            }
            hir::ExprKind::FieldGet { base, path } => {
                let b = self.expr(base, out)?;
                let base_ty = self
                    .view
                    .resolve_use_ty(self.f.body.exprs[base.0 as usize].ty)?;
                let mut names = Vec::new();
                let mut owner = base_ty;
                for fid in &path {
                    let rec = match owner {
                        Ty::Record(rid) => self
                            .view
                            .record(rid)
                            .ok_or_else(|| A2xReject::new("a2x.internal", "record out of range"))?,
                        other => {
                            return Err(A2xReject::unsupported_type(format!(
                                "field path through non-record `{}`",
                                other.as_str()
                            )))
                        }
                    };
                    let fld = rec.fields.get(fid.0 as usize).ok_or_else(|| {
                        A2xReject::new("a2x.internal", "field index out of range")
                    })?;
                    names.push(format!("f_{}", sanitize(&fld.name)));
                    owner = fld.ty;
                }
                Ok(RExpr::FieldPath {
                    base: Box::new(b),
                    path: names,
                })
            }
            hir::ExprKind::BorrowView { base } => Ok(RExpr::Ref {
                inner: Box::new(self.expr(base, out)?),
            }),
            hir::ExprKind::BorrowMut { local } => Ok(RExpr::RefMut {
                inner: Box::new(RExpr::Local(self.local_name_of(local))),
            }),
            hir::ExprKind::Move { inner } => self.expr(inner, out),
            hir::ExprKind::ListNew { list } => {
                let elem = self.view.list_elem(list.0)?;
                let call = RExpr::CollCall {
                    func: "a2x_coll_list_new".into(),
                    turbo: Some(rust_ty_str(self.view, elem)),
                    args: Vec::new(),
                };
                let ty = self.view.resolve_use_ty(e.ty)?;
                self.coll_temp(id, rust_ty_str(self.view, ty), call, out)
            }
            hir::ExprKind::ListLen { list } => {
                let l = self.place_ref(list, false)?;
                Ok(RExpr::CollCall {
                    func: "a2x_coll_list_len".into(),
                    turbo: None,
                    args: vec![l],
                })
            }
            hir::ExprKind::ListPush { list, value } => {
                let l = self.place_ref(list, true)?;
                let v = self.owned_or_plain_child(value, out)?;
                out.push(RStmt::EvalExpr {
                    expr: self.wrap_consume(
                        id,
                        value,
                        RExpr::CollCall {
                            func: "a2x_coll_list_push".into(),
                            turbo: None,
                            args: vec![l, v],
                        },
                    ),
                });
                Ok(RExpr::UnitExpr)
            }
            hir::ExprKind::ListInsert { list, index, value } => {
                let l = self.place_ref(list, true)?;
                let i = self.expr(index, out)?;
                let v = self.owned_or_plain_child(value, out)?;
                out.push(RStmt::EvalExpr {
                    expr: self.wrap_consume(
                        id,
                        value,
                        RExpr::CollCall {
                            func: "a2x_coll_list_insert".into(),
                            turbo: None,
                            args: vec![l, i, v],
                        },
                    ),
                });
                Ok(RExpr::UnitExpr)
            }
            hir::ExprKind::ListSet { list, index, value } => {
                let l = self.place_ref(list, true)?;
                let i = self.expr(index, out)?;
                let v = self.owned_or_plain_child(value, out)?;
                out.push(RStmt::EvalExpr {
                    expr: self.wrap_consume(
                        id,
                        value,
                        RExpr::CollCall {
                            func: "a2x_coll_list_set".into(),
                            turbo: None,
                            args: vec![l, i, v],
                        },
                    ),
                });
                Ok(RExpr::UnitExpr)
            }
            hir::ExprKind::ListGet { list, index } => {
                let l = self.place_ref(list, false)?;
                let i = self.expr(index, out)?;
                let ty = self.view.resolve_use_ty(e.ty)?;
                if self.temp_kind_of(e).is_some() || !is_copy(ty) {
                    // owned-element read: the CHECKED VIEW (no materialization)
                    let elem = self.list_elem_of(list)?;
                    let call = RExpr::CollCall {
                        func: "a2x_coll_list_borrow_elem".into(),
                        turbo: None,
                        args: vec![l, i],
                    };
                    let name = format!("e{}", id.0);
                    let m = self.temp_needs_mut(id);
                    out.push(RStmt::CollLet {
                        name: name.clone(),
                        ty_str: format!("&{}", rust_ty_str(self.view, elem)),
                        mutable: m,
                        init: self.wrap(id, call),
                    });
                    Ok(RExpr::Local(name))
                } else {
                    let call = RExpr::CollCall {
                        func: "a2x_coll_list_get".into(),
                        turbo: None,
                        args: vec![l, i],
                    };
                    self.coll_temp(id, rust_ty_str(self.view, ty), call, out)
                }
            }
            hir::ExprKind::ListRemove { list, index } => {
                let l = self.place_ref(list, true)?;
                let i = self.expr(index, out)?;
                let ty = self.view.resolve_use_ty(e.ty)?;
                let call = RExpr::CollCall {
                    func: "a2x_coll_list_remove".into(),
                    turbo: None,
                    args: vec![l, i],
                };
                self.coll_temp(id, rust_ty_str(self.view, ty), call, out)
            }
            hir::ExprKind::ListClear { list } => {
                let l = self.place_ref(list, true)?;
                out.push(RStmt::EvalExpr {
                    expr: RExpr::CollCall {
                        func: "a2x_coll_list_clear".into(),
                        turbo: None,
                        args: vec![l],
                    },
                });
                Ok(RExpr::UnitExpr)
            }
            hir::ExprKind::VariantCtor {
                variant,
                disc,
                payload,
            } => {
                let v = self
                    .view
                    .variant(variant.0)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "variant out of range"))?;
                let arm = v
                    .arms
                    .get(disc as usize)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "arm out of range"))?;
                let payload_expr = match payload {
                    Some(p) => Some(self.owned_or_plain_child(p, out)?),
                    None => None,
                };
                out.push(RStmt::EvalExpr {
                    expr: self.wrap(
                        id,
                        RExpr::CollCall {
                            func: "a2x_coll_charge_ctor".into(),
                            turbo: None,
                            args: Vec::new(),
                        },
                    ),
                });
                let vname = rust_variant_name(&v.name);
                let aname = sanitize(&arm.name);
                let ctor = RExpr::VariantCtorExpr {
                    variant: vname,
                    arm: aname,
                    payload: payload_expr.map(Box::new),
                };
                if self.temp_kind_of(e).is_some() {
                    let ty = self.view.resolve_use_ty(e.ty)?;
                    self.coll_temp_plain(id, rust_ty_str(self.view, ty), ctor, out)
                } else {
                    Ok(ctor)
                }
            }
            hir::ExprKind::VariantGuardTest { root, disc } => {
                let (vname, aname, has_payload) =
                    self.variant_arm(root_ty_of(self, root)?, disc)?;
                let r = self.expr(root, out)?;
                Ok(RExpr::VariantIs {
                    variant: vname,
                    arm: aname,
                    payload: has_payload,
                    root: Box::new(r),
                })
            }
            hir::ExprKind::VariantProject { root, disc } => {
                let (vname, aname, _) = self.variant_arm(root_ty_of(self, root)?, disc)?;
                let r = self.expr(root, out)?;
                let call = RExpr::VariantTake {
                    helper: format!("a2x_coll_take_{}_{}", vname, aname),
                    root: Box::new(r),
                };
                let ty = self.view.resolve_use_ty(e.ty)?;
                self.coll_temp(id, rust_ty_str(self.view, ty), call, out)
            }
            other => Err(A2xReject::unsupported_expr(format!(
                "expression kind `{:?}` outside the collections surface",
                other
            ))),
        }
    }

    /// A child in an owned-value slot: owned-producing children lower to
    /// their temps (by value); a literal promotes through
    /// `a2x_str_from_lit` into a named temp exactly where the facts model
    /// the pending; plain values pass through.
    fn owned_or_plain_child(
        &mut self,
        child: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<RExpr, A2xReject> {
        let ce = self
            .f
            .body
            .exprs
            .get(child.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
        if let hir::ExprKind::StrLit { value } = &ce.kind {
            let name = format!("e{}", child.0);
            let m = self.temp_needs_mut(child);
            out.push(RStmt::CollLet {
                name: name.clone(),
                ty_str: "A2xStr".into(),
                mutable: m,
                init: self.wrap(
                    child,
                    RExpr::StrFn {
                        func: "a2x_str_from_lit".into(),
                        args: vec![RExpr::StrLit(value.clone())],
                    },
                ),
            });
            return Ok(RExpr::Local(name));
        }
        self.expr(child, out)
    }

    /// A text operand as a borrowed `&str` view.
    fn operand_str_view(
        &mut self,
        operand: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<RExpr, A2xReject> {
        let o = self.expr(operand, out)?;
        let ty = self
            .view
            .resolve_use_ty(self.f.body.exprs[operand.0 as usize].ty)?;
        Ok(match ty {
            Ty::StrOwned => RExpr::AsStr {
                val: Box::new(o),
                owned: true,
            },
            _ => o,
        })
    }

    fn list_elem_of(&self, list_place: hir::ExprId) -> Result<Ty, A2xReject> {
        // list operand is a ReadLocal of list type
        match &self.f.body.exprs[list_place.0 as usize].kind {
            hir::ExprKind::ReadLocal { local } => {
                let l = self
                    .locals
                    .get(local.0 as usize)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "local index out of range"))?;
                match self.view.resolve_use_ty(l.ty)? {
                    Ty::List(id) => self.view.list_elem(id),
                    other => Err(A2xReject::unsupported_type(format!(
                        "expected list operand, found `{}`",
                        other.as_str()
                    ))),
                }
            }
            _ => Err(A2xReject::unsupported_expr(
                "list operand must be a whole-local read",
            )),
        }
    }

    fn variant_arm(&self, ty: Ty, disc: u32) -> Result<(String, String, bool), A2xReject> {
        let Ty::Variant(id) = ty else {
            return Err(A2xReject::unsupported_type(
                "variant guard/projection root must be variant-typed",
            ));
        };
        let v = self
            .view
            .variant(id)
            .ok_or_else(|| A2xReject::new("a2x.internal", "variant out of range"))?;
        let arm = v
            .arms
            .get(disc as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "arm out of range"))?;
        Ok((
            rust_variant_name(&v.name),
            sanitize(&arm.name),
            arm.payload.is_some(),
        ))
    }
}

fn root_ty_of(w: &CollLower<'_, '_>, root: hir::ExprId) -> Result<Ty, A2xReject> {
    w.view.resolve_use_ty(w.f.body.exprs[root.0 as usize].ty)
}

fn is_lit(body: &hir::Body, id: hir::ExprId) -> bool {
    matches!(body.exprs[id.0 as usize].kind, hir::ExprKind::StrLit { .. })
}

fn owned_target(w: &CollLower<'_, '_>, local: u32) -> bool {
    w.local_ty(hir::LocalId(local))
        .map(|t| matches!(t, Ty::StrOwned))
        .unwrap_or(false)
}

fn is_copy(ty: Ty) -> bool {
    matches!(ty, Ty::I32 | Ty::Bool | Ty::Char)
}

/// The nominal variant table of a collections view (PLAN-904).
pub fn lower_coll_variants(view: &SemanticView<'_>) -> Vec<RustVariant> {
    view.variants()
        .iter()
        .map(|v| RustVariant {
            type_index: v.type_index,
            name: rust_variant_name(&v.name),
            arms: v
                .arms
                .iter()
                .map(|a| (sanitize(&a.name), a.payload))
                .collect(),
        })
        .collect()
}
