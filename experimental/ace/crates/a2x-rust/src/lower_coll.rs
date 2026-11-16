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
        // PLAN-972 (T-03): closure values render as argument-position
        // `impl Fn(P...) -> R` (params take them by value). The signature
        // is interned in the type table — the infallible lookup mirrors
        // the Record/Variant precedent.
        Ty::Closure(idx) => match view.closure_sig(idx) {
            Some(sig) => {
                let ps: Vec<String> = sig
                    .params
                    .iter()
                    .map(|t| {
                        let ty = view.resolve_use_ty(*t).ok()?;
                        Some(rust_ty_str(view, ty))
                    })
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or_default();
                let r = view
                    .resolve_use_ty(sig.result)
                    .map(|ty| rust_ty_str(view, ty))
                    .unwrap_or_else(|_| "()".into());
                // the closure body fn is fallible (A2xResult) — the literal
                // forwards it and the call site unwraps with `?`.
                format!("impl Fn({}) -> A2xResult<{}>", ps.join(", "), r)
            }
            None => "impl Fn".into(),
        },
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
    // PLAN-973: list element table for the printer's A2xList<E> payloads.
    let mut list_elems = std::collections::BTreeMap::new();
    for v in view.variants() {
        for a in &v.arms {
            collect_list_elems(view, a.payload, &mut list_elems);
        }
    }
    Ok(RustProgram {
        functions,
        entry: String::new(),
        marks,
        records,
        clones: Vec::new(),
        borrowed: false,
        variants,
        coll: true,
        list_elems,
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
        shared_list_params: std::collections::HashSet::new(),
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
        if p.mode == hir::ParamMode::View && matches!(p.ty, Ty::List(_)) {
            w.shared_list_params.insert(local_name(&l.name, *idx));
        }
        named_params.push(RustParam {
            name: local_name(&l.name, *idx),
            ty: p.ty,
            mode: p.mode,
            // PLAN-946: bake the element-typed list text with the live view
            // (the printer's fragment ctx carries no element table).
            ty_text: Some(rust_ty_str(view, p.ty)),
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
    // PLAN-972 (T-03): List/closure results need the view-resolved text
    // (the PrintCtx has no element table); other results keep the
    // historical rendering.
    let result_text = match f.result {
        Ty::List(_) | Ty::Closure(_) => Some(rust_ty_str(view, f.result)),
        _ => None,
    };
    Ok(RustFn {
        name: rust_fn_name(f.source_name),
        params: named_params,
        result: f.result,
        origin: f.origin,
        body,
        result_text,
    })
}

fn local_name(name: &str, id: u32) -> String {
    format!("l{}_{}", id, sanitize(name))
}

struct CollLower<'a, 'v> {
    /// PLAN-946: names of View-mode List params (rendered `&'a mut
    /// A2xList<E>`); their in-body place refs pass the name bare — Rust
    /// reborrows/coerces at the argument position.
    shared_list_params: std::collections::HashSet<String>,
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

    /// Resolve a sequence of field declaration IDs into sanitized Rust field
    /// names (`f_<name>`) along with the final field's type.
    fn resolve_field_names(
        &self,
        mut owner: Ty,
        path: &[hir::FieldId],
    ) -> Result<(Vec<String>, Ty), A2xReject> {
        let mut names = Vec::with_capacity(path.len());
        for fid in path {
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
            let fld = rec
                .fields
                .get(fid.0 as usize)
                .ok_or_else(|| A2xReject::new("a2x.internal", "field index out of range"))?;
            names.push(format!("f_{}", sanitize(&fld.name)));
            owner = fld.ty;
        }
        Ok((names, owner))
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
                    let mut v = if is_lit(self.f.body, *value) && owned_target(self, local.0) {
                        self.owned_or_plain_child(*value, &mut out)?
                    } else {
                        self.expr(*value, &mut out)?
                    };
                    // PLAN-968: a borrowed str initializing an owned field
                    // (the scheduler driver's state locals take spawn
                    // initializers through the Str-view import seam)
                    // promotes through the runtime's owned conversion.
                    // PLAN-968: a scheduler driver's state field local
                    // may initialize from a borrowed str PARAM read (the
                    // import seam types spawn initializers as the str
                    // view). The let TARGET is owned — promote the source
                    // local read through the runtime's owned conversion.
                    if owned_target(self, local.0) && !is_lit(self.f.body, *value) {
                        // A PARAM read renders as a borrowed view (view
                        // mode) even when its declared type is owned —
                        // promote the param-read initializer.
                        let src_param = match &self.f.body.exprs[value.0 as usize].kind {
                            hir::ExprKind::ReadLocal { local: src } => self
                                .locals
                                .get(src.0 as usize)
                                .map(|l| {
                                    matches!(l.role, hir::LocalRole::Param)
                                        && matches!(
                                            self.view.resolve_use_ty(l.ty),
                                            Ok(Ty::StrOwned | Ty::Str)
                                        )
                                })
                                .unwrap_or(false),
                            _ => false,
                        };
                        if src_param {
                            v = RExpr::StrFn {
                                func: "a2x_str_to_owned".into(),
                                args: vec![v],
                            };
                        }
                    }
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
                    let target_ty = match &p_pre.kind {
                        hir::PlaceKind::Local => self.local_ty(p_pre.local).ok(),
                        hir::PlaceKind::FieldPath(path) => self
                            .local_ty(p_pre.local)
                            .and_then(|ty| self.resolve_field_names(ty, path).map(|(_, fty)| fty))
                            .ok(),
                    };
                    let owned_place = matches!(target_ty, Some(Ty::StrOwned));
                    let v = if is_lit(self.f.body, *value) && owned_place {
                        self.owned_or_plain_child(*value, &mut out)?
                    } else {
                        self.expr(*value, &mut out)?
                    };
                    let p = &self.f.body.places[place.0 as usize];
                    match &p.kind {
                        hir::PlaceKind::Local => {
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
                            out.push(RStmt::Assign {
                                name: self.local_name_of(p.local),
                                value: v,
                            });
                        }
                        hir::PlaceKind::FieldPath(path) => {
                            let base_ty = self.local_ty(p.local)?;
                            let (names, _) = self.resolve_field_names(base_ty, path)?;
                            out.push(RStmt::AssignField {
                                name: self.local_name_of(p.local),
                                path: names,
                                value: v,
                            });
                        }
                    }
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
                    if !ace_a2x_core::semantic_view::block_always_terminates(self.f.body, *lb) {
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
                    }
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
                hir::Stmt::Fail { value } => {
                    let owned_msg = self
                        .view
                        .resolve_use_ty(self.f.body.exprs[value.0 as usize].ty)
                        .map(|t| matches!(t, Ty::StrOwned))
                        .unwrap_or(false);
                    let v = if is_lit(self.f.body, *value) && owned_msg {
                        self.owned_or_plain_child(*value, &mut out)?
                    } else {
                        self.expr(*value, &mut out)?
                    };
                    let _ = v;
                    let frees = self
                        .facts
                        .exit_frees
                        .get(&(block.0, Some(si)))
                        .map(|fs| {
                            fs.iter()
                                .filter_map(|r| match r {
                                    FreeRef::Local(l) => {
                                        self.local_ty(hir::LocalId(*l)).ok().and_then(|ty| {
                                            self.owned_kind_of_ty(ty).map(|k| {
                                                (
                                                    self.local_name_of(hir::LocalId(*l)),
                                                    self.free_call(k),
                                                )
                                            })
                                        })
                                    }
                                    FreeRef::Temp(e, kind) => {
                                        Some((format!("e{}", e), self.free_call(*kind)))
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    out.push(RStmt::Return {
                        value: RExpr::Wrap {
                            inner: Box::new(RExpr::CollCall {
                                func: "a2x_coll_fail".into(),
                                turbo: None,
                                args: Vec::new(),
                            }),
                            frees,
                        },
                    });
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
            hir::ExprKind::ReadLocal { local } => {
                let n = self.local_name_of(*local);
                // PLAN-946: a `&'a mut A2xList` param place passes the name
                // bare (Rust reborrows at the argument position).
                if self.shared_list_params.contains(&n) {
                    return Ok(RExpr::Local(n));
                }
                RExpr::Local(n)
            }
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
            // PLAN-972 (T-03): typed numeric constants ride the shared
            // numeric-surface literal forms in the collections family too
            // (the typed binops already ride the num_bin matrix above).
            hir::ExprKind::NumericConst { ty: bty, bits } => crate::lower::num_lit(bty, bits),
            // PLAN-972 (T-03): the 961 conversion matrix rides the
            // collections family as well (conv helper keyed on the
            // operand's builtin type; identity casts pass through).
            hir::ExprKind::Cast { operand, to } => {
                let o = self.expr(operand, out)?;
                let from = root_ty_of(self, operand)?;
                let from_b = crate::lower::ty_builtin(from).ok_or_else(|| {
                    A2xReject::unsupported_type(format!(
                        "cast source {from:?} is not a numeric builtin"
                    ))
                })?;
                if from_b == to {
                    return Ok(o);
                }
                let func = crate::lower::conv_helper(from_b, to)?;
                Ok(RExpr::Checked {
                    func,
                    args: vec![o],
                })
            }
            // PLAN-972 (T-03): capture-free closure values — the literal is
            // a `move` closure over the (already emitted) body fn. Capturing
            // closures await the a2x closure-env unit (named rejection).
            hir::ExprKind::CreateClosure { callee, captures } => {
                if !captures.is_empty() {
                    return Err(A2xReject::unsupported_op(
                        "capturing closures await the a2x closure-env unit (PLAN-972)",
                    ));
                }
                let f = self.view.fn_by_def(callee.0 as usize).ok_or_else(|| {
                    A2xReject::new("a2x.internal", "closure callee def not in view")
                })?;
                let func = crate::lower::rust_fn_name(f.source_name);
                let params: Vec<(String, String)> = f
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (format!("p{}", i), rust_ty_str(self.view, p.ty)))
                    .collect();
                Ok(RExpr::ClosureLit { func, params })
            }
            // PLAN-972 (T-03): the char expression surface (scalar-indexed
            // codepoints; invalid scalars are the typed trap).
            hir::ExprKind::CharLit { scalar } => {
                let c = char::from_u32(scalar).ok_or_else(|| {
                    A2xReject::new("a2x.internal", "invalid scalar in checked CharLit")
                })?;
                Ok(RExpr::CharLit(c))
            }
            hir::ExprKind::CharToCodepoint { operand } => {
                let o = self.expr(operand, out)?;
                Ok(RExpr::CharToI32 { inner: Box::new(o) })
            }
            hir::ExprKind::CharFromCodepoint { operand } => {
                let o = self.expr(operand, out)?;
                self.checked_temp_ty("a2x_char_from_codepoint", vec![o], id, out, Ty::Char)
            }
            // PLAN-975 B1: `c.to(Str)` — char materialized as a fresh
            // owned 1-scalar Str (the shared decode/parse primitive).
            hir::ExprKind::TextFromChar { operand } => {
                let o = self.expr(operand, out)?;
                self.checked_temp_ty("a2x_str_from_char", vec![o], id, out, Ty::StrOwned)
            }
            // PLAN-972 (T-03): the scalar-indexed text expression surface
            // (the shared runtime helpers; slices stay borrowed views).
            hir::ExprKind::TextLenBytes { operand } => {
                let o = self.operand_str_view(operand, out)?;
                self.checked_temp("a2x_str_len_bytes", vec![o], id, out)
            }
            hir::ExprKind::TextCharAt { operand, index } => {
                let o = self.operand_str_view(operand, out)?;
                let i = self.expr(index, out)?;
                self.checked_temp_ty("a2x_str_char_at", vec![o, i], id, out, Ty::Char)
            }
            hir::ExprKind::TextSlice {
                operand,
                start,
                end,
            } => {
                let o = self.operand_str_view(operand, out)?;
                let a = self.expr(start, out)?;
                let b = self.expr(end, out)?;
                self.checked_temp_ty("a2x_str_slice", vec![o, a, b], id, out, Ty::Str)
            }
            hir::ExprKind::TextToOwned { operand } => {
                let o = self.operand_str_view(operand, out)?;
                self.checked_temp_ty("a2x_str_to_owned", vec![o], id, out, Ty::StrOwned)
            }
            hir::ExprKind::TextConcat { lhs, rhs } => {
                let l = self.operand_str_view(lhs, out)?;
                let r = self.operand_str_view(rhs, out)?;
                self.checked_temp_ty("a2x_str_concat", vec![l, r], id, out, Ty::StrOwned)
            }
            hir::ExprKind::TextClone { operand } => {
                let o = self.operand_str_view(operand, out)?;
                self.checked_temp_ty("a2x_str_clone", vec![o], id, out, Ty::StrOwned)
            }
            // PLAN-972 (T-03): indirect call through the closure value.
            hir::ExprKind::CallClosure { callee, eval_args } => {
                let f = self.expr(callee, out)?;
                let mut args = Vec::with_capacity(eval_args.len());
                for a in eval_args {
                    args.push(self.expr(a, out)?);
                }
                Ok(RExpr::IndirectCall {
                    func: Box::new(f),
                    args,
                })
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let lt = self
                    .view
                    .resolve_use_ty(self.f.body.exprs[lhs.0 as usize].ty)?;
                if matches!(op, hir::BinOp::Eq | hir::BinOp::Ne) && lt.is_text() {
                    let l = self.operand_str_view(lhs, out)?;
                    let r = self.operand_str_view(rhs, out)?;
                    return Ok(RExpr::StrCmp {
                        eq: matches!(op, hir::BinOp::Eq),
                        lhs: Box::new(l),
                        rhs: Box::new(r),
                    });
                }
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
        self.checked_temp_ty(func, args, id, out, Ty::I32)
    }

    /// PLAN-972 (T-03): typed checked helpers declare their temp with the
    /// helper's REAL result type (the i32 ops keep Ty::I32; the 961 typed
    /// matrix ops pass their builtin result type — never a hard-coded i32).
    fn checked_temp_ty(
        &self,
        func: &'static str,
        args: Vec<RExpr>,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
        ty: Ty,
    ) -> Result<RExpr, A2xReject> {
        let name = format!("e{}", id.0);
        let m = self.temp_needs_mut(id);
        out.push(RStmt::CollLet {
            name: name.clone(),
            ty_str: rust_ty_str(self.view, ty).to_string(),
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
    /// PLAN-968: lower one scheduler intrinsic call on the collections
    /// path — the same runtime mapping as the scalar lane (threads +
    /// bounded channels; AV-equivalent observable semantics).
    fn lower_sched_call_coll(
        &mut self,
        symbol: &str,
        def_id: usize,
        id: hir::ExprId,
        ty: Ty,
        eval_args: &[hir::ExprId],
        out: &mut Vec<RStmt>,
    ) -> Result<RExpr, A2xReject> {
        // Console print spellings: println!/eprintln! helpers.
        if let Some(tail) = symbol.strip_prefix("auto.console.") {
            let (is_print, ty_word) = if let Some(w) = tail.strip_prefix("print.") {
                (true, w)
            } else if let Some(w) = tail.strip_prefix("eprint.") {
                (false, w)
            } else {
                return Err(A2xReject::unsupported_expr(format!(
                    "console intrinsic `{symbol}` is outside the scheduler surface"
                )));
            };
            let helper = match (is_print, ty_word) {
                (true, "str.v1") => "a2x_print_str",
                (true, "i32.v1") => "a2x_print_i32",
                (true, "bool.v1") => "a2x_print_bool",
                (false, "str.v1") => "a2x_eprint_str",
                (false, "i32.v1") => "a2x_eprint_i32",
                (false, "bool.v1") => "a2x_eprint_bool",
                _ => {
                    return Err(A2xReject::unsupported_expr(format!(
                        "console intrinsic `{symbol}` is outside the scheduler surface"
                    )))
                }
            };
            let v = self.expr(eval_args[0], out)?;
            out.push(RStmt::EvalExpr {
                expr: RExpr::PlainCall {
                    func: helper.to_string(),
                    args: vec![v],
                },
            });
            return self.coll_temp_plain(id, "i32".into(), RExpr::I32Lit(0), out);
        }

        let Some(Ok((sig_params, _sig_result))) = self.view.sched_sig_types(def_id) else {
            return Err(A2xReject::new(
                "a2x.internal",
                format!("scheduler intrinsic `{symbol}` lacks its fn-type seam"),
            ));
        };

        let mut args = Vec::with_capacity(eval_args.len());
        for a in eval_args {
            args.push(self.expr(*a, out)?);
        }
        let wrap = |e: RExpr, t: &Ty| -> RExpr {
            let text_lit = matches!(&e, RExpr::StrLit(_));
            let f = match t {
                Ty::Bool => "a2x_v_b",
                Ty::Str | Ty::StrOwned => {
                    if text_lit {
                        "a2x_v_lit"
                    } else {
                        "a2x_v_s"
                    }
                }
                _ => "a2x_v_i",
            };
            RExpr::PlainCall {
                func: f.to_string(),
                args: vec![e],
            }
        };
        let unwrap = |e: RExpr, t: &Ty| -> RExpr {
            let f = match t {
                Ty::Bool => "a2x_un_b",
                Ty::Str | Ty::StrOwned => "a2x_un_s",
                _ => "a2x_un_i",
            };
            RExpr::PlainCall {
                func: f.to_string(),
                args: vec![e],
            }
        };
        let rest = symbol.strip_prefix("__sched_").unwrap_or(symbol);

        let plain = |func: &str, args: Vec<RExpr>| RExpr::PlainCall {
            func: func.to_string(),
            args,
        };

        let call: RExpr = if rest == "sleep" || rest == "stop" {
            plain(&format!("a2x_sched_{rest}"), args)
        } else if rest.starts_with("send_") {
            plain(
                "a2x_sched_send",
                vec![args[0].clone(), wrap(args[1].clone(), &sig_params[1])],
            )
        } else if rest.starts_with("ask_") {
            plain(
                "a2x_sched_ask",
                vec![args[0].clone(), wrap(args[1].clone(), &sig_params[1])],
            )
        } else if rest.starts_with("reply_") {
            plain(
                "a2x_sched_reply",
                vec![wrap(args[0].clone(), &sig_params[0])],
            )
        } else if rest.starts_with("await_") {
            unwrap(plain("a2x_sched_await", vec![args[0].clone()]), &ty)
        } else if rest.starts_with("recv_") {
            unwrap(plain("a2x_sched_recv", Vec::new()), &ty)
        } else if rest.starts_with("port_") {
            plain("a2x_sched_port", Vec::new())
        } else if rest.starts_with("stopping_") {
            plain("a2x_sched_stopping", Vec::new())
        } else if let Some(t) = rest.strip_prefix("spawn_") {
            let cap = args[1].clone();
            let mut vals = Vec::new();
            for (k, a) in args[2..].iter().enumerate() {
                vals.push(wrap(a.clone(), &sig_params[2 + k]));
            }
            let adapter = format!("__sched_task_adapter_{t}");
            out.push(RStmt::RawFn {
                text: sched_task_adapter_text(t, &sig_params[2..]),
            });
            let vec_call = match vals.len() {
                0 => plain("a2x_sched_vec0", Vec::new()),
                1 => plain("a2x_sched_vec1", vals),
                2 => plain("a2x_sched_vec2", vals),
                3 => plain("a2x_sched_vec3", vals),
                _ => plain("a2x_sched_vec", vals),
            };
            plain(
                "a2x_sched_spawn",
                vec![cap, vec_call, RExpr::Local(adapter)],
            )
        } else if let Some(rest2) = rest.strip_prefix("go_") {
            let name = rest2.splitn(2, '_').nth(1).unwrap_or("").to_string();
            let mut vals = Vec::new();
            for (k, a) in args[1..].iter().enumerate() {
                vals.push(wrap(a.clone(), &sig_params[1 + k]));
            }
            let adapter = format!("__sched_go_adapter_{name}");
            out.push(RStmt::RawFn {
                text: sched_go_adapter_text(&name, &sig_params[1..]),
            });
            let vec_call = match vals.len() {
                0 => plain("a2x_sched_vec0", Vec::new()),
                1 => plain("a2x_sched_vec1", vals),
                2 => plain("a2x_sched_vec2", vals),
                3 => plain("a2x_sched_vec3", vals),
                _ => plain("a2x_sched_vec", vals),
            };
            plain("a2x_sched_go", vec![vec_call, RExpr::Local(adapter)])
        } else {
            return Err(A2xReject::unsupported_expr(format!(
                "scheduler intrinsic `{symbol}` is outside the a2r support domain"
            )));
        };
        let ty_str = rust_ty_str(self.view, ty);
        self.coll_temp_plain(id, ty_str, call, out)
    }

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
                        let ty = crate::lower::builtin_ty(bty)?;
                        self.checked_temp_ty(func, vec![o], id, out, ty)
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
                let target = callee.0 as usize;
                // PLAN-968: scheduler-surface intrinsics map to the
                // appended scheduler runtime (the AV-equivalent threads +
                // bounded channels semantics).
                if let Some(sym) = self.view.sched_symbol(target) {
                    let ty = self.view.resolve_use_ty(e.ty)?;
                    return self.lower_sched_call_coll(sym, target, id, ty, &eval_args, out);
                }
                let func = if let Some(sym) = self.view.trace_symbol(target) {
                    let suffix = if sym == "hir.test.mark_a" { "a" } else { "b" };
                    format!("a2x_mark_{}", suffix)
                } else if let Some(t) = self.view.fn_by_def(target) {
                    rust_fn_name(t.source_name)
                } else if let Some(x) = self.view.external_by_def(target) {
                    rust_fn_name(&x.source_name)
                } else if let Some(sym) = self.view.platform_symbol(target) {
                    // PLAN-973: system-library platform intrinsics map to
                    // the plain target-runtime stubs (a2x_<callee>) and are
                    // re-wrapped as infallible results in this failable
                    // lane; the shared op table owns the callee spelling
                    // and the Str-parameter positions.
                    let op = ace_core::source::platform_io::PLATFORM_SOURCE_OPS
                        .iter()
                        .find(|o| o.symbol == sym)
                        .ok_or_else(|| {
                            A2xReject::unsupported_expr(format!(
                                "platform symbol {sym} is not in the op table"
                            ))
                        })?;
                    let mut args = Vec::new();
                    for (i, a) in eval_args.iter().enumerate() {
                        let is_str =
                            matches!(op.params.get(i), Some(ace_core::source::resolve::VT::Str));
                        let v = if is_str {
                            self.operand_str_view(*a, out)?
                        } else {
                            self.expr(*a, out)?
                        };
                        args.push(v);
                    }
                    let call = RExpr::PlainCall {
                        func: format!("a2x_{}", op.callee),
                        args,
                    };
                    let ty = self.view.resolve_use_ty(e.ty)?;
                    return self.coll_temp_plain(id, rust_ty_str(self.view, ty), call, out);
                } else {
                    return Err(A2xReject::unsupported_expr(format!(
                        "call target not admitted (def #{target})"
                    )));
                };
                // PLAN-946: View-mode List parameters borrow exclusively at
                // the call (`&mut x`) — the caller keeps its list (callee
                // mutations visible), mirroring the AV's shared handle.
                let shared_list_param: Vec<bool> = if let Some(t) = self.view.fn_by_def(target) {
                    t.params
                        .iter()
                        .map(|p| p.mode == hir::ParamMode::View && matches!(p.ty, Ty::List(_)))
                        .collect()
                } else if let Some(x) = self.view.external_by_def(target) {
                    x.params
                        .iter()
                        .zip(x.modes.iter())
                        .map(|(ty, m)| *m == hir::ParamMode::View && matches!(ty, Ty::List(_)))
                        .collect()
                } else {
                    Vec::new()
                };
                // PLAN-972 (T-03): str-borrowed callee params take the
                // operand's VIEW form (owned temps borrow through
                // `.as_str()`; already-borrowed params pass as-is).
                let str_view_param: Vec<bool> = if let Some(t) = self.view.fn_by_def(target) {
                    t.params
                        .iter()
                        .map(|p| {
                            matches!(p.ty, Ty::Str | Ty::StrOwned)
                                && p.mode != hir::ParamMode::MoveOwn
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let mut args = Vec::new();
                for (i, a) in eval_args.iter().enumerate() {
                    let v = if str_view_param.get(i).copied().unwrap_or(false) {
                        self.operand_str_view(*a, out)?
                    } else {
                        self.expr(*a, out)?
                    };
                    if shared_list_param.get(i).copied().unwrap_or(false) {
                        // PLAN-972 (T-02 map slice): an arg that is itself
                        // the bare name of a View-List param ALREADY renders
                        // as `&'a mut A2xList` — pass it bare (Rust
                        // reborrows at the argument position). Wrapping it
                        // in `&mut` would borrow the binding itself
                        // (E0596), not the list — the same discipline the
                        // BorrowMut arm below applies to place refs.
                        let bare_param =
                            matches!(&v, RExpr::Local(n) if self.shared_list_params.contains(n));
                        if !bare_param {
                            args.push(RExpr::RefMut { inner: Box::new(v) });
                        } else {
                            args.push(v);
                        }
                    } else {
                        args.push(v);
                    }
                }
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
            hir::ExprKind::TextToBorrowed { operand } => {
                // PLAN-968: the scheduler f-string chain borrows owned
                // parts — a plain as_str view, no ownership transfer.
                let o = self.operand_str_view(operand, out)?;
                Ok(o)
            }
            hir::ExprKind::TextDisplay { operand } => {
                // PLAN-968: the scheduler f-string chain renders an i32
                // into an owned buffer.
                let o = self.expr(operand, out)?;
                self.coll_temp(
                    id,
                    "A2xStr".into(),
                    RExpr::StrFn {
                        func: "a2x_str_display_i32".into(),
                        args: vec![o],
                    },
                    out,
                )
            }
            hir::ExprKind::TextConcat { lhs, rhs } => {
                // PLAN-968: the scheduler f-string chain concatenates
                // borrowed parts into an owned buffer.
                let l = self.operand_str_view(lhs, out)?;
                let r = self.operand_str_view(rhs, out)?;
                self.coll_temp(
                    id,
                    "A2xStr".into(),
                    RExpr::StrFn {
                        func: "a2x_str_concat".into(),
                        args: vec![l, r],
                    },
                    out,
                )
            }
            hir::ExprKind::TextFromChar { operand } => {
                // PLAN-975 B1: the scheduler chain can materialize a
                // dynamic codepoint the same way (fresh owned 1-scalar).
                let o = self.expr(operand, out)?;
                self.coll_temp(
                    id,
                    "A2xStr".into(),
                    RExpr::StrFn {
                        func: "a2x_str_from_char".into(),
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
                let (names, _) = self.resolve_field_names(base_ty, &path)?;
                Ok(RExpr::FieldPath {
                    base: Box::new(b),
                    path: names,
                })
            }
            hir::ExprKind::BorrowView { base } => {
                let inner = self.expr(base, out)?;
                if let RExpr::Local(n) = &inner {
                    if self.shared_list_params.contains(n) {
                        return Ok(inner);
                    }
                }
                Ok(RExpr::Ref {
                    inner: Box::new(inner),
                })
            }
            hir::ExprKind::BorrowMut { local } => {
                let n = self.local_name_of(local);
                if self.shared_list_params.contains(&n) {
                    return Ok(RExpr::Local(n));
                }
                Ok(RExpr::RefMut {
                    inner: Box::new(RExpr::Local(n)),
                })
            }
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
                // PLAN-973: pin through a temp — an exit sweep between the
                // lowering and the return would otherwise free the list
                // BEFORE this inline call renders (len()==0 bug).
                let call = RExpr::CollCall {
                    func: "a2x_coll_list_len".into(),
                    turbo: None,
                    args: vec![l],
                };
                self.coll_temp_plain(id, "i32".into(), call, out)
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
        // PLAN-972 (T-03): a View/MutBorrow Str param already RENDERS as a
        // borrow (`&'a str`) — `.as_str()` on it is the unstable str
        // method. Only genuinely owned values (MoveOwn params, owned
        // temps) carry the A2xStr borrow step.
        // a literal renders as a borrowed `&str` already — `.as_str()`
        // on it is the unstable str method.
        let is_lit = matches!(
            &self.f.body.exprs[operand.0 as usize].kind,
            hir::ExprKind::StrLit { .. }
        );
        let borrowed_param = is_lit
            || match &self.f.body.exprs[operand.0 as usize].kind {
                hir::ExprKind::ReadLocal { local } => self
                    .f
                    .body
                    .locals
                    .get(local.0 as usize)
                    .map(|d| {
                        matches!(d.role, hir::LocalRole::Param)
                            && !matches!(d.param_mode, hir::ParamMode::MoveOwn)
                    })
                    .unwrap_or(false),
                _ => false,
            };
        Ok(match ty {
            Ty::StrOwned if !borrowed_param => RExpr::AsStr {
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

/// PLAN-968: the synthesized `fn(Vec<SchedValue>)` adapter that runs a
/// task's driver with typed initializers (built by plain concatenation —
/// no brace escaping).
fn sched_task_adapter_text(task: &str, field_tys: &[Ty]) -> String {
    let mut b = String::new();
    b.push_str("fn __sched_task_adapter_");
    b.push_str(task);
    b.push_str("(a: ::std::vec::Vec<crate::a2x_runtime::SchedValue>) {");
    b.push('\n');
    b.push_str("    let _ = a;");
    b.push('\n');
    b.push_str("    let _ = fn___task_");
    b.push_str(task);
    b.push_str("(");
    for (k, t) in field_tys.iter().enumerate() {
        if k > 0 {
            b.push_str(", ");
        }
        if matches!(t, Ty::Str | Ty::StrOwned) {
            b.push_str(unwrap_helper(t));
            b.push_str("(a[");
            b.push_str(&k.to_string());
            b.push_str("].clone()).as_str()");
        } else {
            b.push_str(unwrap_helper(t));
            b.push_str("(a[");
            b.push_str(&k.to_string());
            b.push_str("].clone())");
        }
    }
    b.push_str(");");
    b.push('\n');
    b.push_str("}");
    b.push('\n');
    b
}

/// PLAN-968: the synthesized `fn(Vec<SchedValue>)` adapter for a `go` call.
fn sched_go_adapter_text(name: &str, param_tys: &[Ty]) -> String {
    let mut b = String::new();
    b.push_str("fn __sched_go_adapter_");
    b.push_str(name);
    b.push_str("(a: ::std::vec::Vec<crate::a2x_runtime::SchedValue>) {");
    b.push('\n');
    b.push_str("    let _ = a;");
    b.push('\n');
    b.push_str("    let _ = fn_");
    b.push_str(name);
    b.push_str("(");
    for (k, t) in param_tys.iter().enumerate() {
        if k > 0 {
            b.push_str(", ");
        }
        if matches!(t, Ty::Str | Ty::StrOwned) {
            b.push_str(unwrap_helper(t));
            b.push_str("(a[");
            b.push_str(&k.to_string());
            b.push_str("].clone()).as_str()");
        } else {
            b.push_str(unwrap_helper(t));
            b.push_str("(a[");
            b.push_str(&k.to_string());
            b.push_str("].clone())");
        }
    }
    b.push_str(");");
    b.push('\n');
    b.push_str("}");
    b.push('\n');
    b
}

/// PLAN-968: the typed unwrap helper name for an adapter parameter.
fn unwrap_helper(t: &Ty) -> &'static str {
    match t {
        Ty::Bool => "a2x_un_b",
        Ty::Str | Ty::StrOwned => "a2x_un_s",
        _ => "a2x_un_i",
    }
}

/// PLAN-973: collect list type-index -> element type over a payload tree
/// (variant payloads today; signatures can join the same table).
pub fn collect_list_elems(
    view: &SemanticView<'_>,
    ty: Option<Ty>,
    out: &mut std::collections::BTreeMap<u32, Ty>,
) {
    let Some(t) = ty else { return };
    match t {
        Ty::List(id) => {
            if let Ok(e) = view.list_elem(id) {
                out.insert(id, e);
                collect_list_elems(view, Some(e), out);
            }
        }
        _ => {}
    }
}
