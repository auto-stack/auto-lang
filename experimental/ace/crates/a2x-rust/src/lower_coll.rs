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
    RustParam, RustProgram, RustVariant, RStatic,
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
        // PLAN-1115 (T-03): the whole dynamic family (nil/dyn/object) is
        // the checked dynamic value — one runtime type.
        Ty::Nil | Ty::Dyn | Ty::Object(_) => "A2xDyn".into(),
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

/// PLAN-1115 (T-02): lower a whole ORDINARY-language module (profile
/// `auto.source.language.av/v1`, aggregates/closures superset) and select
/// its entry. The ordinary face rides the collections-family lowerer —
/// the unified surface (numerics + text + records + lists/variants +
/// capture-free closures) — under the LANGUAGE channel identity.
pub fn lower_language_program(
    view: &SemanticView<'_>,
    entry_name: &str,
) -> Result<RustProgram, A2xReject> {
    // PLAN-1115 (T-03): the language lane — the collections-family surface
    // WITH the cross-frame error slot machinery enabled.
    let entry = view.entry(entry_name)?;
    let mut p = lower_coll_functions_channel(view, true)?;
    p.entry = rust_fn_name(entry.source_name);
    Ok(p)
}

/// Lower every function of a collections view WITHOUT entry selection
/// (the store's per-fragment produce path needs the function table only).
pub fn lower_coll_functions(view: &SemanticView<'_>) -> Result<RustProgram, A2xReject> {
    lower_coll_functions_channel(view, false)
}

fn lower_coll_functions_channel(
    view: &SemanticView<'_>,
    lang_channel: bool,
) -> Result<RustProgram, A2xReject> {
    let mut functions = Vec::new();
    let mut clones: Vec<u32> = Vec::new();
    for f in view.functions() {
        let (rf, used) = lower_coll_fn_channel(view, f, lang_channel)?;
        for c in used {
            if !clones.contains(&c) {
                clones.push(c);
            }
        }
        functions.push(rf);
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
    // PLAN-1115 (T-02): record FIELDS with list types join the element
    // table too — the ordinary face carries `type Row { xs List<int> }`
    // shapes (nested containers), not only variant payloads.
    for r in &records {
        for (_, ty) in &r.fields {
            collect_list_elems(view, Some(*ty), &mut list_elems);
        }
    }
    // PLAN-1115 (T-03, from 981): module-level globals -> static decls
    // with zero init; the init chain (dependency order) runs in main
    // before the entry. Scalar compute globals only in this slice —
    // owned/record globals stay the compute lane's named boundary.
    let mut statics = Vec::new();
    for g in view.globals_model()? {
        let init = match g.ty {
            Ty::I32 | Ty::I8 | Ty::U8 | Ty::U32 | Ty::I64 | Ty::U64 => "0",
            Ty::F32 | Ty::F64 => "0.0",
            Ty::Bool => "false",
            other => {
                return Err(A2xReject::unsupported_expr(format!(
                    "module global `{}` has type `{}` — only compute-surface scalar globals lower to Rust statics in this slice",
                    g.name,
                    other.as_str()
                )))
            }
        };
        statics.push(RStatic {
            name: crate::lower::global_static_name(&g.symbol),
            ty: g.ty,
            init,
        });
    }
    let init_fns = view
        .global_init_fns()
        .into_iter()
        .map(crate::lower::rust_fn_name)
        .collect();
    Ok(RustProgram {
        statics,
        init_fns,
        functions,
        entry: String::new(),
        marks,
        records,
        clones,
        borrowed: false,
        variants,
        coll: true,
        list_elems,
    })
}

/// Lower one function of the collections channel (PLAN-1115: also returns
/// the record ids whose deep-clone helpers the body referenced).
pub fn lower_coll_fn(
    view: &SemanticView<'_>,
    f: &FnFacts<'_>,
) -> Result<(RustFn, Vec<u32>), A2xReject> {
    lower_coll_fn_channel(view, f, false)
}

/// PLAN-1115 (T-03): channel-aware form — the language lane enables the
/// cross-frame error slot machinery.
pub fn lower_coll_fn_channel(
    view: &SemanticView<'_>,
    f: &FnFacts<'_>,
    lang_channel: bool,
) -> Result<(RustFn, Vec<u32>), A2xReject> {
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
        loop_stack: Vec::new(),
        clones: Vec::new(),
        borrow_list_locals: std::collections::HashSet::new(),
        moved_locals: std::collections::HashSet::new(),
        state_temp_sources: std::collections::HashMap::new(),
        pending_project: None,
        pending_writebacks: Vec::new(),
        protected: None,
        lang_channel,
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
    // PLAN-1115 (T-02): Capture-role locals become the body fn's LEADING
    // params — the CreateClosure literal binds them from the enclosing
    // scope (Move transfers by value; Borrow copies/views the immutable
    // source). Names follow the HIR local table so body references and
    // the signature agree, exactly like declared params.
    let mut capture_locals: Vec<(u32, &hir::LocalDef)> = f
        .body
        .locals
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.role, hir::LocalRole::Capture))
        .map(|(i, l)| (i as u32, l))
        .collect();
    capture_locals.sort_by_key(|(_, l)| l.index.unwrap_or(u32::MAX));
    for (idx, l) in &capture_locals {
        let ty = view.resolve_use_ty(l.ty)?;
        named_params.push(RustParam {
            name: local_name(&l.name, *idx),
            ty,
            mode: hir::ParamMode::View,
            ty_text: Some(rust_ty_str(view, ty)),
        });
    }
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
    Ok((
        RustFn {
            name: rust_fn_name(f.source_name),
            params: named_params,
            result: f.result,
            origin: f.origin,
            body,
            result_text,
        },
        std::mem::take(&mut w.clones),
    ))
}

/// PLAN-1115 (T-03): pack a typed value into the dynamic representation
/// by its own static type (identity for the dynamic family).
fn pack_to_dyn(v: RExpr, ty: Ty) -> RExpr {
    match ty {
        Ty::Dyn | Ty::Nil | Ty::Object(_) => v,
        Ty::I32 => RExpr::PlainCall { func: "a2x_dyn_int".into(), args: vec![v] },
        Ty::Bool => RExpr::PlainCall { func: "a2x_dyn_bool".into(), args: vec![v] },
        Ty::Char => RExpr::PlainCall { func: "a2x_dyn_char".into(), args: vec![v] },
        // owned text packs as a COPY (read semantics — the operand local
        // stays live; AV value reads never alias the source buffer); a
        // borrowed LITERAL packs through the view (it is not an A2xStr
        // binding).
        Ty::StrOwned if matches!(v, RExpr::StrLit(_)) => {
            RExpr::Call { func: "a2x_dyn_str_view".into(), args: vec![v] }
        }
        Ty::StrOwned => {
            RExpr::Call {
                func: "a2x_dyn_str_share".into(),
                args: vec![RExpr::Ref { inner: Box::new(v) }],
            }
        }
        Ty::Str => RExpr::Call { func: "a2x_dyn_str_view".into(), args: vec![v] },
        // a typed dyn-element list packs through the shared-list helper.
        Ty::List(_) => RExpr::Call {
            func: "a2x_dyn_pack_list".into(),
            args: vec![v],
        },
        // Everything else (records, variants, numerics via the catch-all,
        // closures) does not enter the dynamic domain on this slice — the
        // lowering rejects by name at the pack site.
        _ => {
            let _ = v;
            unreachable!("pack_to_dyn is only called for packable operand types")
        }
    }
}

/// A static object-key literal's runtime rendering.
fn key_lit(k: &ace_core::hir::ObjectKey) -> RExpr {
    use ace_core::hir::ObjectKey;
    match k {
        ObjectKey::Text(t) => RExpr::PlainCall {
            func: "a2x_key_text".into(),
            args: vec![RExpr::StrLit(t.clone())],
        },
        ObjectKey::Int(i) => RExpr::PlainCall {
            func: "a2x_key_int".into(),
            args: vec![RExpr::I32Lit(*i)],
        },
        ObjectKey::Bool(b) => RExpr::PlainCall {
            func: "a2x_key_bool".into(),
            args: vec![RExpr::BoolLit(*b)],
        },
    }
}

/// The i-th param type of a view fn (an arg-classification helper).
fn param_ty_cache(w: &CollLower<'_, '_>, target: usize, i: usize) -> Option<Ty> {
    w.view.fn_by_def(target).and_then(|t| t.params.get(i).map(|p| p.ty))
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
    /// PLAN-1036 (AUDIT-1024 F35): the enclosing loops' (id, body block) —
    /// a `continue` must pay the same back-edge poll as the fall-through.
    loop_stack: Vec<(hir::LoopId, hir::BlockId)>,
    /// PLAN-1115 (T-02): record ids whose `a2x_rt_clone_*` deep-clone
    /// helpers this body referenced (threaded to the program's clone set).
    clones: Vec<u32>,
    /// PLAN-1115 (T-02): locals bound to an ALIAS of a container (List)
    /// field — rendered as `&mut A2xList<E>` bindings (the ordinary face
    /// shares container handles; writes flow through). They never free
    /// (the owning record does) and list operands pass them bare.
    borrow_list_locals: std::collections::HashSet<u32>,
    /// PLAN-1115 (T-02): locals whose owned value already MOVED into a
    /// record construction (or another consumer) — later error-path and
    /// exit frees of these names are stale (the consumer owns them now).
    moved_locals: std::collections::HashSet<u32>,
    /// PLAN-1116 (T-02): generator STATE-field unwrap temps (`__gen_v*`
    /// materializations of a `&mut __ace_iter_*` field) →
    /// (state local, field, option variant). AV aliases every field read
    /// through the handle; A2R clones per read, so a mutation through
    /// such a temp must be written back to its field (the frontend
    /// re-reads the field per use and never rewraps).
    state_temp_sources: std::collections::HashMap<String, (String, String, String)>,
    /// PLAN-1116 (T-02): the VariantProject just lowered out of a
    /// generator state field — (state local, field, option variant). The
    /// enclosing `tmp_val = project(...)` Assign registers the write-back
    /// mapping under its target name.
    pending_project: Option<(String, String, String)>,
    /// PLAN-1116 (T-02): state temps mutated by the CURRENT statement's
    /// `&mut` receivers — written back to their fields right after it.
    pending_writebacks: Vec<String>,
    /// PLAN-1115 (T-03): the nearest enclosing PROTECTED region's
    /// (kind-local, message-local) names — fallible expressions inside
    /// intercept LangError and break out to the catch arm.
    protected: Option<(String, String)>,
    /// PLAN-1115 (T-03): the LANGUAGE channel flag — the cross-frame
    /// message slot (`a2x_lang_fail`/`a2x_lang_take_err`) exists only in
    /// the language runtime; the collections channel keeps its frozen
    /// `a2x_coll_fail` shape.
    lang_channel: bool,
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

    /// PLAN-1036 (AUDIT-1024 F35): the cooperative back-edge statement for
    /// the loop whose body block is `lb` — 1 WU + the cancel check, with the
    /// per-iteration deaths AND the loop survivors on its error path (the
    /// frees are idempotent, so the overlap with the fall-through schedule
    /// stays safe). Emitted once per real back edge: at the body's
    /// fall-through and at every `continue` into this loop.
    fn back_edge_poll(&self, lb: hir::BlockId) -> RStmt {
        let mut poll_frees = self
            .facts
            .exit_frees
            .get(&(lb.0, None))
            .map(|f| {
                f.iter()
                    .filter_map(|r| match r {
                        FreeRef::Local(l) => {
                            // PLAN-1115 (T-05): a local whose value moved
                            // into a binding/consumer frees nowhere here.
                            if self.borrow_list_locals.contains(l) || self.moved_locals.contains(l)
                            {
                                return None;
                            }
                            let ty = self.local_ty(hir::LocalId(*l)).ok()?;
                            let k = self.owned_kind_of_ty(ty)?;
                            Some((self.local_name_of(hir::LocalId(*l)), self.free_call(k)))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if let Some(surv) = self.facts.loop_survivors.get(&lb.0) {
            for l in surv {
                if self.borrow_list_locals.contains(l) || self.moved_locals.contains(l) {
                    continue;
                }
                if let Some(ty) = self.local_ty(hir::LocalId(*l)).ok() {
                    if let Some(k) = self.owned_kind_of_ty(ty) {
                        poll_frees.push((self.local_name_of(hir::LocalId(*l)), self.free_call(k)));
                    }
                }
            }
        }
        RStmt::EvalExpr {
            expr: RExpr::Wrap {
                inner: Box::new(RExpr::CollCall {
                    func: "a2x_coll_poll".into(),
                    turbo: None,
                    args: Vec::new(),
                }),
                frees: poll_frees,
            },
        }
    }

    /// Render one scheduled free set as statements.
    fn free_stmts(&self, frees: &[FreeRef]) -> Vec<RStmt> {
        frees
            .iter()
            .filter_map(|r| match r {
                FreeRef::Local(l) => {
                    // PLAN-1115 (T-02): alias bindings borrow — the owning
                    // record frees; skip (a free would double-free). Moved
                    // locals are the consumer's responsibility.
                    if self.borrow_list_locals.contains(l) || self.moved_locals.contains(l) {
                        return None;
                    }
                    let ty = self.local_ty(hir::LocalId(*l)).ok();
                    let call = ty
                        .and_then(|t| self.owned_kind_of_ty(t))
                        .map(|k| self.free_call(k));
                    // PLAN-1115 (T-03): no owned kind (the dynamic family
                    // rides Rc) -> no caller-side free.
                    let call = call?;
                    Some(RStmt::FreeSlot {
                        name: self.local_name_of(hir::LocalId(*l)),
                        call,
                    })
                }
                FreeRef::Temp(e, kind) => Some(RStmt::FreeSlot {
                    name: format!("e{}", e),
                    call: self.free_call(*kind),
                }),
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
                    .filter_map(|r| match r {
                        FreeRef::Local(l) => {
                            // PLAN-1115 (T-02): an alias binding borrows —
                            // the owning record frees; a free here would
                            // double-free through the borrow. A local whose
                            // value already MOVED into a consumer is the
                            // consumer's responsibility from the move on.
                            if self.borrow_list_locals.contains(l) || self.moved_locals.contains(l)
                            {
                                return None;
                            }
                            let ty = self.local_ty(hir::LocalId(*l)).ok();
                            let call = ty
                                .and_then(|t| self.owned_kind_of_ty(t))
                                .map(|k| self.free_call(k));
                            // PLAN-1115 (T-03): no owned kind -> no free.
                            let call = call?;
                            Some((self.local_name_of(hir::LocalId(*l)), call))
                        }
                        FreeRef::Temp(e, kind) => Some((format!("e{}", e), self.free_call(*kind))),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Wrap a fallible expression with its verifier-scheduled error frees.
    /// PLAN-1115 (T-03): inside a PROTECTED region the wrap INTERCEPTS the
    /// language error (kind=4 + the stored message + break to the catch
    /// arm); other traps keep propagating.
    fn wrap(&self, id: hir::ExprId, inner: RExpr) -> RExpr {
        let frees = self.err_frees_of(id.0);
        if let Some((kind, msg)) = &self.protected.clone().filter(|_| self.lang_channel) {
            return RExpr::WrapIntercept(Box::new(crate::lower::WrapInterceptData {
                inner,
                frees,
                kind: kind.clone(),
                msg: msg.clone(),
            }));
        }
        RExpr::Wrap {
            inner: Box::new(inner),
            frees,
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
                    // PLAN-1115 (T-03): the ordinary face's whole-root
                    // move is a PLAIN local-to-local binding (`.move` or
                    // implicit) — an owned source dies here; later reads
                    // reject by name (mir.use-after-move's A2R form).
                    // Reassignment below revives the local.
                    if let (hir::ExprKind::ReadLocal { local: src }, true) =
                        (&self.f.body.exprs[value.0 as usize].kind, true)
                    {
                        let src_ty = self.local_ty(*src).ok();
                        if src_ty.is_some_and(|t| self.owned_kind_of_ty(t).is_some())
                            && src.0 != local.0
                        {
                            self.moved_locals.insert(src.0);
                        }
                    }
                    // PLAN-1115 (T-02): a let bound to a container-field
                    // ALIAS renders as an exclusive borrow binding — the
                    // local table keeps the LIST type (the facts address
                    // it as a list operand), the Rust binding borrows.
                    if matches!(
                        &v,
                        RExpr::RefMut { inner }
                            if matches!(inner.as_ref(), RExpr::FieldPath { .. })
                    ) {
                        self.borrow_list_locals.insert(local.0);
                        out.push(RStmt::CollLet {
                            name: self.local_name_of(*local),
                            ty_str: format!("&mut {}", rust_ty_str(self.view, ty)),
                            mutable: false,
                            init: v,
                        });
                    } else {
                        // closure-typed locals infer at their binding
                        // (annotation-free — `impl Trait` is not a binding
                        // type; the compute lane's rule).
                        let ty_str = match ty {
                            Ty::Closure(_) => String::new(),
                            _ => rust_ty_str(self.view, ty),
                        };
                        out.push(RStmt::CollLet {
                            name: self.local_name_of(*local),
                            ty_str,
                            mutable: self.local_needs_mut(local.0),
                            init: v,
                        });
                    }
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
                    // PLAN-1115 (T-03): assigning a whole local revives a
                    // previously moved root (a fresh value binds here).
                    if matches!(p_pre.kind, hir::PlaceKind::Local) {
                        self.moved_locals.remove(&p_pre.local.0);
                    }
                    // PLAN-1116 (T-02): assigning one OWNED local from
                    // another (`x = y` on a record/list/owned text) is a
                    // MOVE in Rust — the source dies (the frontend's
                    // iterator-swap protocol relies on this: the terminal
                    // temp must not be freed after it moves into the
                    // active slot). Copy scalars/views keep plain reads.
                    let src_read = match &self.f.body.exprs[value.0 as usize].kind {
                        hir::ExprKind::ReadLocal { local } if Some(*local) != (match &p_pre.kind {
                            hir::PlaceKind::Local => Some(p_pre.local),
                            _ => None,
                        }) =>
                        {
                            Some(local.0)
                        }
                        _ => None,
                    };
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
                    // PLAN-1116 (T-02): `tmp_val = take(state.field)` —
                    // the target temp becomes the state field's alias;
                    // mutations through it are written back after the
                    // mutating statement.
                    if let Some((state, field, variant)) = self.pending_project.take() {
                        if matches!(
                            &self.f.body.exprs[value.0 as usize].kind,
                            hir::ExprKind::VariantProject { .. }
                        ) && matches!(p_pre.kind, hir::PlaceKind::Local)
                        {
                            let target = self.local_name_of(p_pre.local);
                            self.state_temp_sources
                                .insert(target, (state, field, variant));
                        }
                    }
                    let p = &self.f.body.places[place.0 as usize];
                    match &p.kind {
                        hir::PlaceKind::Local => {
                            if let Some(old) = self.facts.replace_frees.get(&(block.0, si)) {
                                let ty = self.local_ty(p.local).ok();
                                let call = ty
                                    .and_then(|t| self.owned_kind_of_ty(t))
                                    .map(|k| self.free_call(k));
                                // PLAN-1115 (T-03): no owned kind -> no free.
                                if let Some(call) = call {
                                    out.push(RStmt::FreeSlot {
                                        name: self.local_name_of(p.local),
                                        call,
                                    });
                                }
                                let _ = old;
                            }
                            out.push(RStmt::Assign {
                                name: self.local_name_of(p.local),
                                value: v,
                            });
                            if let Some(src) = src_read {
                                if let Ok(ty) = self.local_ty(hir::LocalId(src)) {
                                    if self.owned_kind_of_ty(ty).is_some() {
                                        self.moved_locals.insert(src);
                                    }
                                }
                            }
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
                    // PLAN-1053: the move-kill set is a may-kill analysis.
                    // A kill inside one branch must not mark the root dead
                    // on the SIBLING path (the `else` may be the taken one)
                    // nor after the merge unless EVERY path kills it — merge
                    // the branch-end sets by INTERSECTION (the pre-if kills
                    // are in both, so they survive). Without this, the
                    // control-flow-insensitive set false-positives the AAC
                    // idiom `if p.len() == 0 { p = part.move } else { p =
                    // p + "." + part }` (source_catalog.import_paths) and
                    // any kill placed inside a conditional.
                    let saved = self.moved_locals.clone();
                    let t = self.block(*then)?;
                    let then_kills = self.moved_locals.clone();
                    self.moved_locals = saved;
                    let e = match els {
                        Some(eb) => Some(self.block(*eb)?),
                        None => None,
                    };
                    let merged: std::collections::HashSet<u32> = then_kills
                        .intersection(&self.moved_locals)
                        .copied()
                        .collect();
                    self.moved_locals = merged;
                    out.push(RStmt::If {
                        cond: c,
                        then: t,
                        els: e,
                    });
                }
                hir::Stmt::Loop {
                    id,
                    body: lb,
                    unwind,
                } => {
                    // PLAN-1036 (F35): lower the body with this loop on the
                    // stack so every `continue` into it carries the same
                    // back-edge poll.
                    self.loop_stack.push((*id, *lb));
                    // PLAN-1115 (T-03): a loop carrying an unwind spec is a
                    // PROTECTED region — fallible expressions inside
                    // intercept the language error into (kind, message).
                    let saved = self.protected.take();
                    if let (true, Some(u)) = (self.lang_channel, unwind) {
                        self.protected = Some((
                            self.local_name_of(u.kind),
                            self.local_name_of(u.message),
                        ));
                    }
                    // PLAN-1053: a kill inside the loop body executes only
                    // if the body runs, so it must not escape past the
                    // header — restore the pre-loop set (kills before the
                    // loop stay in effect inside and after; real
                    // use-after-move is still refused by the frozen MIR
                    // ownership verification upstream).
                    let saved_moves = self.moved_locals.clone();
                    let lowered_body = self.block(*lb);
                    self.moved_locals = saved_moves;
                    self.protected = saved;
                    self.loop_stack.pop();
                    let mut body_stmts = lowered_body?;
                    // back-edge: per-iteration scoped frees (recorded at the
                    // body block's fall-through), then the cooperative poll
                    // whose error path frees the survivors too
                    if let Some(frees) = self.facts.exit_frees.get(&(lb.0, None)) {
                        body_stmts.extend(self.free_stmts(frees));
                    }
                    if !ace_a2x_core::semantic_view::block_always_terminates(self.f.body, *lb) {
                        body_stmts.push(self.back_edge_poll(*lb));
                    }
                    out.push(RStmt::Loop { body: body_stmts });
                }
                hir::Stmt::Break { .. } => {
                    if let Some(frees) = self.facts.exit_frees.get(&(block.0, Some(si))) {
                        out.extend(self.free_stmts(frees));
                    }
                    out.push(RStmt::Break);
                }
                hir::Stmt::Continue { target } => {
                    if let Some(frees) = self.facts.exit_frees.get(&(block.0, Some(si))) {
                        out.extend(self.free_stmts(frees));
                    }
                    // PLAN-1036 (F35): every `continue` IS a back edge — it
                    // pays the same 1-WU poll and cancel check the
                    // fall-through back edge pays, and its error path frees
                    // the same survivors.
                    let poll = self
                        .loop_stack
                        .iter()
                        .rev()
                        .find(|(id, _)| id == target)
                        .map(|(_, lb)| (*lb, self.back_edge_poll(*lb)))
                        .map(|(_, poll)| poll);
                    if let Some(poll) = poll {
                        out.push(poll);
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
                    // PLAN-1115 (T-03): the language error carries its
                    // message — inside a protected region the catch arm
                    // reads the (kind, message) locals directly (break's
                    // never type needs no value); outside, the frame's
                    // error slot carries it across the call boundary.
                    if let Some((kind, msg)) = self.protected.clone() {
                        out.push(RStmt::Assign {
                            name: kind.clone(),
                            value: RExpr::I32Lit(4),
                        });
                        out.push(RStmt::Assign {
                            name: msg.clone(),
                            value: v,
                        });
                        out.push(RStmt::Break);
                        continue;
                    }
                    let frees = self
                        .facts
                        .exit_frees
                        .get(&(block.0, Some(si)))
                        .map(|fs| {
                            fs.iter()
                                .filter_map(|r| match r {
                                    FreeRef::Local(l) => {
                                        // PLAN-1116 (T-02): alias/moved
                                        // locals keep the same filter as
                                        // every other free schedule (a
                                        // state-field temp written back is
                                        // owned by the field now).
                                        if self.borrow_list_locals.contains(l)
                                            || self.moved_locals.contains(l)
                                        {
                                            return None;
                                        }
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
                    let frees: Vec<(String, String)> = frees;
                    // the message operand MOVED into the fail call — its
                    // own error-path free drops out of the wrap.
                    let msg_name = match &v {
                        RExpr::Local(n) => Some(n.clone()),
                        _ => None,
                    };
                    let frees = frees
                        .into_iter()
                        .filter(|(n, _)| Some(n) != msg_name.as_ref())
                        .collect();
                    // the language channel carries the message through
                    // the frame slot; the collections channel keeps the
                    // frozen arg-less shape.
                    let (func, args) = if self.lang_channel {
                        ("a2x_lang_fail".to_string(), vec![v])
                    } else {
                        ("a2x_coll_fail".to_string(), Vec::new())
                    };
                    out.push(RStmt::Return {
                        value: RExpr::Wrap {
                            inner: Box::new(RExpr::CollCall { func, turbo: None, args }),
                            frees,
                        },
                    });
                }
                hir::Stmt::Drop { local } => {
                    // PLAN-1115 (T-05): explicit cleanup of an owned local
                    // — the verifier-scheduled free by the local's owned
                    // kind (the compiler-emitted form; programmatic bundles
                    // carry it).
                    let ty = self.local_ty(*local).ok();
                    if let Some(kind) = ty.and_then(|t| self.owned_kind_of_ty(t)) {
                        out.push(RStmt::FreeSlot {
                            name: self.local_name_of(*local),
                            call: self.free_call(kind),
                        });
                    }
                }
            }
            // PLAN-1116 (T-02): write back every generator state-field
            // temp this statement mutated through a `&mut` receiver (AV
            // aliases in place; A2R clones per field read).
            self.drain_state_writebacks(&mut out);
        }
        self.drain_state_writebacks(&mut out);
        Ok(out)
    }

    /// PLAN-1116 (T-02): emit `state.field = Some(temp)` for every state
    /// temp the current statement mutated; the temp's value now lives in
    /// the field (frees skip it, later field reads see the mutation).
    fn drain_state_writebacks(&mut self, out: &mut Vec<RStmt>) {
        if self.pending_writebacks.is_empty() {
            return;
        }
        let temps = std::mem::take(&mut self.pending_writebacks);
        for t in temps {
            let Some((state, field, variant)) = self.state_temp_sources.get(&t).cloned() else {
                continue;
            };
            out.push(RStmt::AssignField {
                name: state,
                path: vec![field],
                value: RExpr::VariantCtorExpr {
                    variant,
                    arm: "Some".into(),
                    payload: Some(Box::new(RExpr::Local(t.clone()))),
                },
            });
            if let Some(id) = self.local_id_of_name(&t) {
                self.moved_locals.insert(id);
            }
        }
    }

    /// PLAN-1116 (T-02): the HIR local id rendering to this name.
    fn local_id_of_name(&self, name: &str) -> Option<u32> {
        self.f
            .body
            .locals
            .iter()
            .enumerate()
            .find(|(i, _)| self.local_name_of(hir::LocalId(*i as u32)) == name)
            .map(|(i, _)| i as u32)
    }

    /// PLAN-1116 (T-02): whether `name` is the `&mut __ace_iter_*` state
    /// struct parameter of a generator next/close function.
    fn is_state_struct_local(&self, name: &str) -> bool {
        let Some(id) = self.local_id_of_name(name) else {
            return false;
        };
        let Some(decl) = self.f.body.locals.get(id as usize) else {
            return false;
        };
        if !matches!(decl.role, hir::LocalRole::Param) {
            return false;
        }
        self.view
            .resolve_use_ty(decl.ty)
            .ok()
            .and_then(|ty| match ty {
                Ty::Record(rec) => self
                    .view
                    .record(rec)
                    .map(|r| r.name.contains("__ace_iter")),
                _ => None,
            })
            .unwrap_or(false)
    }

    /// PLAN-1116 (T-02): the HIR initializer expression binding a local
    /// (`let x = e` / `x = e`), searched across the whole fn body (the
    /// synthesized generator unwrap temps bind in enclosing blocks).
    fn hir_local_binding(&self, local: hir::LocalId) -> Option<hir::ExprId> {
        for b in &self.f.body.blocks {
            for s in &b.stmts {
                match s {
                    hir::Stmt::Let { local: l, value } if *l == local => return Some(*value),
                    hir::Stmt::Assign { place, value } => {
                        let p = &self.f.body.places[place.0 as usize];
                        if matches!(p.kind, hir::PlaceKind::Local) && p.local == local {
                            return Some(*value);
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    /// PLAN-1116 (T-02): whether `value` reads one field of the generator
    /// state struct parameter (`&(state.field)` — the `__gen_r*`
    /// initializer shape), returning (state local name, field name).
    fn state_field_of_expr(&self, value: hir::ExprId) -> Option<(String, String)> {
        let hir::ExprKind::FieldGet { base, path } =
            &self.f.body.exprs[value.0 as usize].kind
        else {
            return None;
        };
        if path.len() != 1 {
            return None;
        }
        let hir::ExprKind::ReadLocal { local: bl } =
            &self.f.body.exprs[base.0 as usize].kind
        else {
            return None;
        };
        let state = self.local_name_of(*bl);
        if !self.is_state_struct_local(&state) {
            return None;
        }
        let base_ty = self.local_ty(*bl).ok()?;
        let (names, _) = self.resolve_field_names(base_ty, path).ok()?;
        if names.len() == 1 {
            Some((state, names[0].clone()))
        } else {
            None
        }
    }

    /// A reference to a local's value place (for &list / &mut list args).
    fn place_ref(&mut self, e: hir::ExprId, mut_: bool, out: &mut Vec<RStmt>) -> Result<RExpr, A2xReject> {
        let inner = match &self.f.body.exprs[e.0 as usize].kind {
            hir::ExprKind::ReadLocal { local } => {
                if self.moved_locals.contains(&local.0) {
                    return Err(A2xReject::unsupported_expr(format!(
                        "use-after-move: the root `{}` moved at its binding site and is dead here (read at {}..{})",
                        self.local_name_of(*local),
                        self.f.body.exprs[e.0 as usize].span.start,
                        self.f.body.exprs[e.0 as usize].span.end
                    )));
                }
                let n = self.local_name_of(*local);
                // PLAN-1116 (T-02): a mutation through a generator
                // state-field temp must be written back to its field
                // after the current statement (AV aliases in place).
                if mut_ && self.state_temp_sources.contains_key(&n) {
                    self.pending_writebacks.push(n.clone());
                }
                // PLAN-946: a `&'a mut A2xList` param place passes the name
                // bare (Rust reborrows at the argument position).
                // PLAN-1115 (T-02): an ALIAS binding (a `&mut A2xList`
                // local) passes bare too — Rust reborrows/derefs at the
                // operand position for both & and &mut consumers.
                if self.shared_list_params.contains(&n)
                    || self.borrow_list_locals.contains(&local.0)
                {
                    return Ok(RExpr::Local(n));
                }
                RExpr::Local(n)
            }
            // PLAN-1115 (T-05): a record-FIELD place base (`r.xs[...]`) —
            // borrow the field path into the base's storage.
            hir::ExprKind::FieldGet { base, path } => {
                let b = self.expr(*base, out)?;
                let base_ty = self
                    .view
                    .resolve_use_ty(self.f.body.exprs[base.0 as usize].ty)?;
                let (names, _) = self.resolve_field_names(base_ty, path)?;
                let inner = RExpr::FieldPath {
                    base: Box::new(b),
                    path: names,
                };
                return Ok(if mut_ {
                    RExpr::RefMut {
                        inner: Box::new(inner),
                    }
                } else {
                    RExpr::Ref {
                        inner: Box::new(inner),
                    }
                });
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
            hir::ExprKind::ReadLocal { local } => {
                if self.moved_locals.contains(&local.0) {
                    return Err(A2xReject::unsupported_expr(format!(
                        "use-after-move: the root `{}` moved at its `.move` site and is dead here (read at {}..{})",
                        self.local_name_of(local),
                        e.span.start,
                        e.span.end
                    )));
                }
                Ok(RExpr::Local(self.local_name_of(local)))
            }
            hir::ExprKind::Constant { value } => Ok(RExpr::I32Lit(value)),
            // PLAN-1115 (T-02): the zero-width unit value — the return
            // operand of a Unit-result function on the ordinary face.
            hir::ExprKind::Unit => Ok(RExpr::UnitExpr),
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
            // a `move` closure over the (already emitted) body fn.
            // PLAN-1115 (T-02): capturing closures ride the same shape —
            // each CaptureBinding becomes the body fn's LEADING argument
            // (the body declares matching Capture-role locals as leading
            // params). Move = by-value transfer (the Rust `move` closure
            // owns it); Borrow = copy/view of the IMMUTABLE source (exact
            // for the Copy-scalar and borrowed-str view domain; owned
            // borrows stay the named closure-env debt); Shared (var cell
            // write-through) stays the named closure-env debt.
            hir::ExprKind::CreateClosure { callee, captures } => {
                let f = self.view.fn_by_def(callee.0 as usize).ok_or_else(|| {
                    A2xReject::new("a2x.internal", "closure callee def not in view")
                })?;
                let mut env_args: Vec<RExpr> = Vec::new();
                for c in &captures {
                    match c.mode {
                        hir::CaptureMode::Move => {
                            env_args.push(RExpr::Local(self.local_name_of(c.local)));
                        }
                        hir::CaptureMode::Borrow => {
                            let ty = self.local_ty(c.local)?;
                            match ty {
                                Ty::I32
                                | Ty::I8
                                | Ty::U8
                                | Ty::U32
                                | Ty::I64
                                | Ty::U64
                                | Ty::F32
                                | Ty::F64
                                | Ty::Bool
                                | Ty::Char
                                | Ty::Str => {
                                    env_args.push(RExpr::Local(self.local_name_of(c.local)));
                                }
                                other => {
                                    return Err(A2xReject::unsupported_op(format!(
                                        "borrow capture of `{}` awaits the closure-env unit \
                                         (PLAN-1115 debt: immutable Copy/str-view domain only)",
                                        other.as_str()
                                    )));
                                }
                            }
                        }
                        hir::CaptureMode::Shared => {
                            return Err(A2xReject::unsupported_op(
                                "shared (var-cell write-through) captures await the \
                                 closure-env unit (PLAN-1115 debt)",
                            ));
                        }
                    }
                }
                let func = crate::lower::rust_fn_name(f.source_name);
                let params: Vec<(String, String)> = f
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (format!("p{}", i), rust_ty_str(self.view, p.ty)))
                    .collect();
                Ok(RExpr::ClosureLit(Box::new(
                    crate::lower::ClosureLitData {
                        func,
                        params,
                        env_args,
                    },
                )))
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
                // PLAN-1115 (T-03): dynamic-family equality (nil, dyn and
                // object operands) — the shared tagged comparator.
                if matches!(op, hir::BinOp::Eq | hir::BinOp::Ne)
                    && matches!(lt, Ty::Dyn | Ty::Nil | Ty::Object(_))
                {
                    let l = self.expr(lhs, out)?;
                    let r = self.expr(rhs, out)?;
                    let eq = matches!(op, hir::BinOp::Eq);
                    let call = RExpr::PlainCall {
                        func: "a2x_dyn_eq".into(),
                        args: vec![
                            RExpr::Ref { inner: Box::new(l) },
                            RExpr::Ref { inner: Box::new(r) },
                        ],
                    };
                    return if eq {
                        self.coll_temp_plain(id, "bool".into(), call, out)
                    } else {
                        self.coll_temp_plain(
                            id,
                            "bool".into(),
                            RExpr::Not(Box::new(call)),
                            out,
                        )
                    };
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
        // PLAN-1115 (T-05): closure-typed temps infer at their binding
        // (`impl Trait` is not a binding type).
        let ty_str = if ty_str.starts_with("impl Fn") {
            String::new()
        } else {
            ty_str
        };
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
        // PLAN-1115 (T-05): closure-typed temps infer at their binding.
        let ty_str = if ty_str.starts_with("impl Fn") {
            String::new()
        } else {
            ty_str
        };
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
        } else if rest.starts_with("await_deferred_") {
            // `await` on a Deferred value (an ask reply future's live port
            // or a lazy async body) — the runtime discriminates by sign.
            // Checked BEFORE the plain `await_` prefix (a Deferred's
            // symbol is `await_deferred_<sig>`).
            unwrap(plain("a2x_sched_await_any", vec![args[0].clone()]), &ty)
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
        } else if rest.starts_with("future_") {
            // PLAN-1116 T-02: an async block `~{ ... }`. The desugar hands
            // the zero-argument thunk as a literal closure; its captures
            // ride the adapter's argument vec (the same env contract the
            // task/go adapters use), the thunk runs on its own thread and
            // the reply port IS the Deferred value.
            let sig = &rest["future_".len()..];
            let name: String = sig
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
                .collect();
            let thunk_expr = self
                .f
                .body
                .exprs
                .get(eval_args[0].0 as usize)
                .ok_or_else(|| A2xReject::new("a2x.internal", "future thunk expr out of range"))?;
            let hir::ExprKind::CreateClosure { callee, captures } = &thunk_expr.kind else {
                return Err(A2xReject::unsupported_expr(format!(
                    "`{symbol}` requires a literal async body"
                )));
            };
            let f = self.view.fn_by_def(callee.0 as usize).ok_or_else(|| {
                A2xReject::new("a2x.internal", "future thunk def not in view")
            })?;
            let mut env_tys: Vec<Ty> = Vec::new();
            let mut env_vals: Vec<RExpr> = Vec::new();
            for c in captures {
                let ty = self.local_ty(c.local)?;
                match c.mode {
                    hir::CaptureMode::Move => {
                        // The env takes the value: a Move-captured local
                        // dies here (the verifier's scheduled free is
                        // filtered by the moved set — same contract as
                        // every other consuming move site).
                        self.moved_locals.insert(c.local.0);
                    }
                    hir::CaptureMode::Borrow => match ty {
                        Ty::I32
                        | Ty::I8
                        | Ty::U8
                        | Ty::U32
                        | Ty::I64
                        | Ty::U64
                        | Ty::F32
                        | Ty::F64
                        | Ty::Bool
                        | Ty::Char
                        | Ty::Str => {}
                        other => {
                            return Err(A2xReject::unsupported_op(format!(
                                "borrow capture of `{}` in an async body awaits the closure-env \
                                 unit (PLAN-1115 debt: immutable Copy/str-view domain only)",
                                other.as_str()
                            )));
                        }
                    },
                    hir::CaptureMode::Shared => {
                        return Err(A2xReject::unsupported_op(
                            "shared (var-cell write-through) captures in an async body await \
                             the closure-env unit (PLAN-1115 debt)",
                        ));
                    }
                }
                env_tys.push(ty);
                env_vals.push(wrap(RExpr::Local(self.local_name_of(c.local)), &ty));
            }
            if !matches!(f.result, Ty::I32 | Ty::Bool | Ty::Str | Ty::StrOwned) {
                return Err(A2xReject::unsupported_type(format!(
                    "an async body returning `{}` is outside the scheduler value domain",
                    f.result.as_str()
                )));
            }
            let adapter = format!("__sched_future_adapter_{name}");
            out.push(RStmt::RawFn {
                text: sched_future_adapter_text(
                    &name,
                    &crate::lower::rust_fn_name(f.source_name),
                    &env_tys,
                    &f.result,
                ),
            });
            let vec_call = match env_vals.len() {
                0 => plain("a2x_sched_vec0", Vec::new()),
                1 => plain("a2x_sched_vec1", env_vals),
                2 => plain("a2x_sched_vec2", env_vals),
                3 => plain("a2x_sched_vec3", env_vals),
                _ => plain("a2x_sched_vec", env_vals),
            };
            plain(
                "a2x_sched_future_lazy",
                vec![RExpr::Local(adapter), vec_call],
            )
        } else if rest.starts_with("await_deferred_") {
            // `await` on a Deferred value (an ask reply future's live port
            // or a lazy async body) — the runtime discriminates by sign.
            unwrap(plain("a2x_sched_await_any", vec![args[0].clone()]), &ty)
        } else if rest == "detach" {
            // `go` on a future: the body already runs eagerly; detach only
            // discards the reply port.
            plain("a2x_sched_detach", vec![args[0].clone()])
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
                callee,
                eval_args,
                bindings,
            } => {
                let target = callee.0 as usize;
                // PLAN-1116 (T-03): a foreign callee bridges through the
                // generated wrapper (the SAME checked-shim exports every
                // other lane binds; A2xResult shape). Text lanes ride as
                // views; scalars pass plain.
                if let Some(decl) = self.view.foreign_by_def(target) {
                    // The bridge domain is validated HERE (the named
                    // rejection surfaces at lowering, before assembly).
                    crate::foreign_bridge::wrapper_source(decl).map_err(|r| {
                        A2xReject::unsupported_expr(format!(
                            "foreign entry `{}` is outside the A2R bridge domain: {r}",
                            decl.name
                        ))
                    })?;
                    let wrapper = format!("a2x_foreign_{}", decl.name);
                    let lanes: Vec<ace_foreign_contract::ResKind> = decl
                        .resource()
                        .map(|r| r.params.clone())
                        .unwrap_or_default();
                    let mut args = Vec::new();
                    for (arg, lane) in eval_args.iter().zip(lanes.iter()) {
                        let a = match lane {
                            ace_foreign_contract::ResKind::ViewText => {
                                self.operand_str_view(*arg, out)?
                            }
                            _ => self.expr(*arg, out)?,
                        };
                        args.push(a);
                    }
                    let bool_ret = matches!(
                        decl.resource().map(|r| &r.form),
                        Some(ace_foreign_contract::ResourceForm::Pure {
                            ret: ace_foreign_contract::ResOut::Bool
                        })
                    );
                    let ty = self.view.resolve_use_ty(e.ty)?;
                    let _ = ty;
                    let ty_str = if bool_ret { "bool".to_string() } else { "i32".to_string() };
                    return self.coll_temp(
                        id,
                        ty_str,
                        RExpr::Call {
                            func: wrapper,
                            args,
                        },
                        out,
                    );
                }
                // PLAN-968: scheduler-surface intrinsics map to the
                // appended scheduler runtime (the AV-equivalent threads +
                // bounded channels semantics).
                if let Some(sym) = self.view.sched_symbol(target) {
                    let ty = self.view.resolve_use_ty(e.ty)?;
                    return self.lower_sched_call_coll(sym, target, id, ty, &eval_args, out);
                }
                // PLAN-1115 (T-05): the checked text-scan intrinsics map
                // to the language runtime's text helpers (str params as
                // views; results fallible).
                if let Some(sym) = self.view.text_scan_symbol(target) {
                    let helper = match sym {
                        "__ace_text_find" => "a2x_str_find",
                        "__ace_text_trim" => "a2x_str_trim",
                        "__ace_text_trim_len" => "a2x_str_trim_len",
                        "__ace_text_lower" => "a2x_str_lower",
                        _ => "a2x_str_upper",
                    };
                    let mut sargs = Vec::new();
                    for a in eval_args.iter() {
                        sargs.push(self.operand_str_view(*a, out)?);
                    }
                    let ty = self.view.resolve_use_ty(e.ty)?;
                    let ty_text = match ty {
                        Ty::StrOwned => "A2xStr".to_string(),
                        _ => "i32".to_string(),
                    };
                    return self.coll_temp(
                        id,
                        ty_text,
                        RExpr::Call {
                            func: helper.into(),
                            args: sargs,
                        },
                        out,
                    );
                }
                // PLAN-1115 (T-03, from 981): module.global accessors ->
                // static read/write (admitted only with the facts plan).
                if let Some((sym, is_set)) = self.view.global_accessor(target) {
                    let name = crate::lower::global_static_name(sym);
                    if is_set {
                        let value = eval_args.first().copied().ok_or_else(|| {
                            A2xReject::new(
                                "a2x.internal",
                                "module.global.set call without exactly one argument",
                            )
                        })?;
                        let v = self.expr(value, out)?;
                        // the setter contract yields i32 (the write expr
                        // renders as an assignment producing 0) — the
                        // compute lane's shape.
                        return Ok(RExpr::GlobalWrite {
                            name,
                            value: Box::new(v),
                        });
                    }
                    if !eval_args.is_empty() {
                        return Err(A2xReject::new(
                            "a2x.internal",
                            "module.global.get call carries arguments",
                        ));
                    }
                    return self.coll_temp_plain(id, "i32".into(), RExpr::GlobalRead { name }, out);
                }
                let mut trace_name: Option<&'static str> = None;
                let mut fn_call = false;
                let func = if let Some(sym) = self.view.trace_symbol(target) {
                    // PLAN-1115 (T-02): the authorized trace marks are
                    // INFALLIBLE i32 runtime fns on this lane — a plain
                    // temp, never the fallible `?` wrap.
                    trace_name = Some(if sym == "hir.test.mark_a" {
                        "a2x_mark_a"
                    } else {
                        "a2x_mark_b"
                    });
                    String::new()
                } else if let Some(t) = self.view.fn_by_def(target) {
                    fn_call = true;
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
                // PLAN-1115 (T-05): the callee's param modes drive both the
                // by-value classification and the variant-clone wrap below.
                let modes: Vec<hir::ParamMode> = self
                    .view
                    .fn_by_def(target)
                    .map(|t| t.params.iter().map(|p| p.mode).collect())
                    .unwrap_or_default();
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
                        // PLAN-1115 (T-02): an ALIAS binding (a
                        // `&mut A2xList` local) passes bare the same way.
                        let bare_param = matches!(&v, RExpr::Local(n) if self.shared_list_params.contains(n))
                            || matches!(
                                &self.f.body.exprs[a.0 as usize].kind,
                                hir::ExprKind::ReadLocal { local }
                                    if self.borrow_list_locals.contains(&local.0)
                            );
                        if !bare_param {
                            args.push(RExpr::RefMut { inner: Box::new(v) });
                        } else {
                            args.push(v);
                        }
                    } else if modes.get(i) == Some(&hir::ParamMode::View)
                        && matches!(param_ty_cache(&self, target, i), Some(Ty::Variant(_)))
                    {
                        // PLAN-1115 (T-05): a variant passed under View mode
                        // crosses BY VALUE — clone at the call (the source
                        // stays live; AV's call-site clone semantics).
                        let vid = match root_ty_of(self, *a).ok() {
                            Some(Ty::Variant(v)) => Some(v),
                            _ => None,
                        };
                        let cloned = match vid.and_then(|v| self.view.variant(v)) {
                            Some(vv) => RExpr::Call {
                                func: format!("a2x_coll_clone_{}", rust_variant_name(&vv.name)),
                                args: vec![RExpr::Ref { inner: Box::new(v) }],
                            },
                            None => v,
                        };
                        args.push(cloned);
                    } else {
                        args.push(v);
                    }
                }
                let ty = self.view.resolve_use_ty(e.ty)?;
                // PLAN-1115 (T-05): an OWNED local moved by value into a
                // callee (MoveOwn params) is the callee's from the call on
                // — its caller-side error free drops out of this wrap and
                // later reads reject by name (the whole-root transfer).
                let mut consumed_names: Vec<String> = Vec::new();
                if fn_call {
                    for (i, a) in eval_args.iter().enumerate() {
                        // by-value = MoveOwn, or a View param whose type is
                        // NOT in the borrowing set (variants pass by value).
                        let by_value = modes.get(i) == Some(&hir::ParamMode::MoveOwn);
                        if !by_value {
                            continue;
                        }
                        match &self.f.body.exprs[a.0 as usize].kind {
                            hir::ExprKind::ReadLocal { local } => {
                                if self
                                    .local_ty(*local)
                                    .ok()
                                    .and_then(|t| self.owned_kind_of_ty(t))
                                    .is_some()
                                {
                                    consumed_names.push(self.local_name_of(*local));
                                    self.moved_locals.insert(local.0);
                                }
                            }
                            _ => {
                                // a materialized temp (e{id}) whose value
                                // moved — drop its error free too
                                if let Ok(v) = self.expr(*a, out) {
                                    if let RExpr::Local(n) = v {
                                        consumed_names.push(n);
                                    }
                                }
                            }
                        }
                    }
                }
                let mut args = args;
                if fn_call && !bindings.is_empty() {
                    // PLAN-1115 (T-02): named/reordered calls place by the
                    // HIR binding map — `sub(b: f(), a: g())` evaluates in
                    // source order (the temps above) but binds by NAME.
                    let n = bindings.iter().map(|b| b.param as usize + 1).max().unwrap_or(0);
                    let mut placed: Vec<Option<RExpr>> = (0..n).map(|_| None).collect();
                    for b in bindings {
                        let v = args.get(b.arg as usize).cloned().ok_or_else(|| {
                            A2xReject::new("a2x.internal", "call binding arg out of range")
                        })?;
                        placed[b.param as usize] = Some(v);
                    }
                    args = placed.into_iter().collect::<Option<Vec<_>>>().ok_or_else(|| {
                        A2xReject::new("a2x.internal", "call binding missing a param slot")
                    })?;
                }
                if let Some(name) = trace_name {
                    // PLAN-1115 (T-02): the authorized trace marks are the
                    // INFALLIBLE effect node (`RExpr::Trace`) — `RExpr::Call`
                    // is a fallible root by contract and would grow a `?`.
                    self.coll_temp_plain(
                        id,
                        rust_ty_str(self.view, ty),
                        RExpr::Trace { func: name },
                        out,
                    )
                } else if consumed_names.is_empty() {
                    let call = RExpr::Call { func, args };
                    self.coll_temp(id, rust_ty_str(self.view, ty), call, out)
                } else {
                    // consumed owned args: fallible temp with the filtered
                    // error frees (the callee owns them from here on)
                    let call = RExpr::Call { func, args };
                    let name = format!("e{}", id.0);
                    let frees: Vec<(String, String)> = self
                        .err_frees_of(id.0)
                        .into_iter()
                        .filter(|(n, _)| !consumed_names.contains(n))
                        .collect();
                    let init = if let Some((kind, msg)) =
                        self.protected.clone().filter(|_| self.lang_channel)
                    {
                        RExpr::WrapIntercept(Box::new(crate::lower::WrapInterceptData {
                            inner: call,
                            frees,
                            kind,
                            msg,
                        }))
                    } else {
                        RExpr::Wrap {
                            inner: Box::new(call),
                            frees,
                        }
                    };
                    out.push(RStmt::CollLet {
                        name: name.clone(),
                        ty_str: rust_ty_str(self.view, ty),
                        mutable: self.temp_needs_mut(id),
                        init,
                    });
                    Ok(RExpr::Local(name))
                }
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
                if root_ty_of(self, operand)? == Ty::F32 {
                    return Err(A2xReject::unsupported_expr("Rust f32 text display"));
                }
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
            // ---- PLAN-1115 (T-03): the ordinary dynamic surface --------
            hir::ExprKind::NilLit => Ok(RExpr::PlainCall {
                func: "a2x_dyn_nil".into(),
                args: Vec::new(),
            }),
            hir::ExprKind::DynCast { operand, test_nil } => {
                let o = self.expr(operand, out)?;
                let from = root_ty_of(self, operand)?;
                let to = self.view.resolve_use_ty(e.ty)?;
                if test_nil {
                    // dyn -> bool: the nil test.
                    return self.coll_temp_plain(
                        id,
                        "bool".into(),
                        RExpr::PlainCall {
                            func: "a2x_dyn_is_nil".into(),
                            args: vec![RExpr::Ref { inner: Box::new(o) }],
                        },
                        out,
                    );
                }
                if matches!(from, Ty::Dyn | Ty::Nil | Ty::Object(_))
                    && matches!(to, Ty::Dyn | Ty::Nil | Ty::Object(_))
                {
                    // already the dynamic representation
                    return Ok(o);
                }
                if matches!(to, Ty::Dyn | Ty::Nil | Ty::Object(_)) {
                    // T -> Dyn: pack by the operand's own type (a consumed
                    // owned LIST operand dies at the pack).
                    if matches!(from, Ty::List(_)) {
                        if let hir::ExprKind::ReadLocal { local } =
                            &self.f.body.exprs[operand.0 as usize].kind
                        {
                            self.moved_locals.insert(local.0);
                        }
                    }
                    return Ok(pack_to_dyn(o, from));
                }
                // Dyn -> T: the checked recover (type mismatch = DynType).
                let func = match to {
                    Ty::I32 => "a2x_dyn_as_int",
                    Ty::Bool => "a2x_dyn_as_bool",
                    Ty::Char => "a2x_dyn_as_char",
                    Ty::StrOwned | Ty::Str => "a2x_dyn_as_str",
                    _ => {
                        return Err(A2xReject::unsupported_op(format!(
                            "dyn recover to `{}` awaits the dyn-conversion unit (PLAN-1115 debt)",
                            to.as_str()
                        )))
                    }
                };
                self.coll_temp(
                    id,
                    rust_ty_str(self.view, to),
                    RExpr::Call {
                        func: func.into(),
                        args: vec![RExpr::Ref { inner: Box::new(o) }],
                    },
                    out,
                )
            }
            hir::ExprKind::ObjectCtor {
                eval_args, keys, ..
            } => {
                // source-order entry values (each dyn-packed by its own
                // type), then the shared map handle; static keys only.
                let mut vals = Vec::new();
                for a in &eval_args {
                    let v = self.expr(*a, out)?;
                    let aty = root_ty_of(self, *a)?;
                    // a consumed owned LIST operand dies at the pack.
                    if matches!(aty, Ty::List(_)) {
                        if let hir::ExprKind::ReadLocal { local } =
                            &self.f.body.exprs[a.0 as usize].kind
                        {
                            self.moved_locals.insert(local.0);
                        }
                    }
                    vals.push(pack_to_dyn(v, aty));
                }
                let obj_name = format!("e{}_obj", id.0);
                out.push(RStmt::CollLet {
                    name: obj_name.clone(),
                    ty_str: "A2xObject".into(),
                    mutable: false,
                    init: RExpr::PlainCall {
                        func: "a2x_obj_new".into(),
                        args: Vec::new(),
                    },
                });
                for (k, v) in keys.iter().zip(vals.into_iter()) {
                    out.push(RStmt::EvalExpr {
                        expr: RExpr::PlainCall {
                            func: "a2x_obj_init".into(),
                            args: vec![
                                RExpr::Ref {
                                    inner: Box::new(RExpr::Local(obj_name.clone())),
                                },
                                key_lit(k),
                                v,
                            ],
                        },
                    });
                }
                self.coll_temp_plain(
                    id,
                    "A2xDyn".into(),
                    RExpr::PlainCall {
                        func: "a2x_dyn_object".into(),
                        args: vec![RExpr::Local(obj_name)],
                    },
                    out,
                )
            }
            hir::ExprKind::ObjectGet { base, key } => {
                let b = self.expr(base, out)?;
                let k = self.expr(key, out)?;
                let kty = root_ty_of(self, key)?;
                let k = pack_to_dyn(k, kty);
                self.coll_temp(
                    id,
                    "A2xDyn".into(),
                    RExpr::Call {
                        func: "a2x_dyn_get".into(),
                        args: vec![
                            RExpr::Ref { inner: Box::new(b) },
                            RExpr::Ref { inner: Box::new(k) },
                        ],
                    },
                    out,
                )
            }
            hir::ExprKind::ObjectSet { base, key, value } => {
                let b = self.expr(base, out)?;
                let k = self.expr(key, out)?;
                let kty = root_ty_of(self, key)?;
                let k = pack_to_dyn(k, kty);
                let v = self.expr(value, out)?;
                let vty = root_ty_of(self, value)?;
                let v = pack_to_dyn(v, vty);
                out.push(RStmt::EvalExpr {
                    expr: self.wrap(
                        id,
                        RExpr::Call {
                            func: "a2x_dyn_set".into(),
                            args: vec![
                                RExpr::Ref { inner: Box::new(b) },
                                RExpr::Ref { inner: Box::new(k) },
                                v,
                            ],
                        },
                    ),
                });
                Ok(RExpr::UnitExpr)
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
                // PLAN-1115 (T-02): a whole owned LOCAL moved into a field
                // dies here — later error-path/exit frees of its name are
                // stale (the record owns the value from this point on).
                for b in &bindings {
                    let field_owned = rec
                        .fields
                        .get(b.field as usize)
                        .map(|f| f.ty)
                        .is_some_and(|t| self.owned_kind_of_ty(t).is_some());
                    if !field_owned {
                        continue;
                    }
                    if let (
                        Some(hir::ExprKind::ReadLocal { local }),
                        Some(RExpr::Local(_)),
                    ) = (
                        eval_args.get(b.arg as usize).map(|a| &self.f.body.exprs[a.0 as usize].kind),
                        arg_vals.get(b.arg as usize),
                    ) {
                        self.moved_locals.insert(local.0);
                    }
                }
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
                let (names, fty) = self.resolve_field_names(base_ty, &path)?;
                // PLAN-1115 (T-02): a container (List) field READ is an
                // ALIAS on the ordinary face (AV handles share; the
                // resolver's place hoists rely on write-through). The Rust
                // representation is an exclusive mutable borrow of the
                // field path — never a copy (independence needs an
                // explicit `.clone()`).
                if matches!(fty, Ty::List(_)) {
                    return Ok(RExpr::RefMut {
                        inner: Box::new(RExpr::FieldPath {
                            base: Box::new(b),
                            path: names,
                        }),
                    });
                }
                // PLAN-1115 (T-05): an OWNED field read (text / record /
                // variant) CLONES — a partial move out of the record would
                // kill later whole-record borrows; reading is an rvalue
                // copy.
                match fty {
                    Ty::StrOwned => {
                        let fp = RExpr::FieldPath {
                            base: Box::new(b),
                            path: names,
                        };
                        return self.coll_temp(
                            id,
                            "A2xStr".into(),
                            RExpr::Call {
                                func: "a2x_str_clone".into(),
                                args: vec![RExpr::AsStr { val: Box::new(fp), owned: true }],
                            },
                            out,
                        );
                    }
                    Ty::Record(rid) | Ty::Variant(rid) => {
                        let name = if matches!(fty, Ty::Record(_)) {
                            self.view.record(rid).map(|r| rust_record_name(&r.name))
                        } else {
                            self.view.variant(rid).map(|v| rust_variant_name(&v.name))
                        };
                        if let Some(name) = name {
                            let fp = RExpr::FieldPath {
                                base: Box::new(b),
                                path: names,
                            };
                            return self.coll_temp(
                                id,
                                rust_ty_str(self.view, fty),
                                RExpr::Call {
                                    func: format!("a2x_coll_clone_{name}"),
                                    args: vec![RExpr::Ref { inner: Box::new(fp) }],
                                },
                                out,
                            );
                        }
                    }
                    _ => {}
                }
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
            hir::ExprKind::Move { inner } => {
                // PLAN-1115 (T-03): the whole-root transfer KILLS the
                // source local — lower the read FIRST, then record the
                // kill; any later read rejects by name (the AV lane's
                // mir.use-after-move equivalent).
                let v = self.expr(inner, out)?;
                if let hir::ExprKind::ReadLocal { local } =
                    &self.f.body.exprs[inner.0 as usize].kind
                {
                    self.moved_locals.insert(local.0);
                }
                Ok(v)
            }
            // PLAN-1115 (T-05): the ordinary face's independent copy —
            // containers deep-clone (element shares for Rc payloads), the
            // dynamic value deep-copies.
            hir::ExprKind::OwnedClone { operand } => {
                let o = self.expr(operand, out)?;
                let oty = root_ty_of(self, operand)?;
                match oty {
                    Ty::List(list_ty) => {
                        let elem = self.view.list_elem(list_ty).ok();
                        self.coll_temp(
                            id,
                            rust_ty_str(self.view, oty),
                            RExpr::CollCall {
                                func: "a2x_coll_list_clone".into(),
                                turbo: elem.map(|e| rust_ty_str(self.view, e)),
                                args: vec![RExpr::Ref { inner: Box::new(o) }],
                            },
                            out,
                        )
                    }
                    Ty::Dyn | Ty::Nil | Ty::Object(_) => {
                        let ty = self.view.resolve_use_ty(e.ty)?;
                        self.coll_temp(
                            id,
                            rust_ty_str(self.view, ty),
                            RExpr::Call {
                                func: "a2x_dyn_deep".into(),
                                args: vec![RExpr::Ref { inner: Box::new(o) }],
                            },
                            out,
                        )
                    }
                    // owned text clones through the runtime view helper;
                    // records ride their generated deep-clone helper.
                    Ty::StrOwned | Ty::Str => {
                        let sv = self.operand_str_view(operand, out)?;
                        self.coll_temp(
                            id,
                            "A2xStr".into(),
                            RExpr::Call {
                                func: "a2x_str_clone".into(),
                                args: vec![sv],
                            },
                            out,
                        )
                    }
                    Ty::Record(rec_id) => {
                        let rec = self.view.record(rec_id).ok_or_else(|| {
                            A2xReject::new("a2x.internal", "record out of range in OwnedClone")
                        })?;
                        let func = format!("a2x_coll_clone_{}", rust_record_name(&rec.name));
                        self.coll_temp(
                            id,
                            rust_ty_str(self.view, oty),
                            RExpr::Call {
                                func,
                                args: vec![RExpr::Ref { inner: Box::new(o) }],
                            },
                            out,
                        )
                    }
                    other => Err(A2xReject::unsupported_op(format!(
                        "owned clone of `{}` awaits the copy unit (PLAN-1115 debt)",
                        other.as_str()
                    ))),
                }
            }
            // PLAN-1115 (T-02): explicit deep clone (PLAN-782) rides the
            // collections family — the same per-record generated helper
            // set as the record-text lane (one allocation per non-empty
            // owned Str field; fallible: Resource).
            hir::ExprKind::RecordClone { record, base } => {
                let rec = self.view.record(record.0).ok_or_else(|| {
                    A2xReject::new("a2x.internal", "unknown record type in RecordClone")
                })?;
                if !self.clones.contains(&record.0) {
                    self.clones.push(record.0);
                }
                let b = self.expr(base, out)?;
                Ok(self.wrap(
                    id,
                    RExpr::StrFn {
                        func: format!("a2x_rt_clone_{}", rust_record_name(&rec.name)),
                        args: vec![RExpr::Ref { inner: Box::new(b) }],
                    },
                ))
            }
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
                // PLAN-1115 (T-05): a DYNAMIC list operand routes through
                // the dyn helpers (the shared-list len reads the buffer).
                if matches!(root_ty_of(self, list).ok(), Some(Ty::Dyn) | Some(Ty::Object(_))) {
                    let b = self.expr(list, out)?;
                    return self.coll_temp(
                        id,
                        "i32".into(),
                        RExpr::PlainCall {
                            func: "a2x_dyn_list_len".into(),
                            args: vec![RExpr::Ref { inner: Box::new(b) }],
                        },
                        out,
                    );
                }
                let l = self.place_ref(list, false, out)?;
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
                let l = self.place_ref(list, true, out)?;
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
                let l = self.place_ref(list, true, out)?;
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
                let l = self.place_ref(list, true, out)?;
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
                // PLAN-1115 (T-05): a DYNAMIC list operand routes through
                // the dyn get (the shared handle semantics).
                if matches!(root_ty_of(self, list).ok(), Some(Ty::Dyn) | Some(Ty::Object(_))) {
                    let b = self.expr(list, out)?;
                    let i = self.expr(index, out)?;
                    let ity = root_ty_of(self, index)?;
                    let k = pack_to_dyn(i, ity);
                    let ty = self.view.resolve_use_ty(e.ty)?;
                    let inner = match ty {
                        Ty::Dyn | Ty::Object(_) => "A2xDyn",
                        Ty::I32 => "i32",
                        Ty::Bool => "bool",
                        _ => "A2xDyn",
                    };
                    let call = RExpr::Call {
                        func: "a2x_dyn_get".into(),
                        args: vec![
                            RExpr::Ref { inner: Box::new(b) },
                            RExpr::Ref { inner: Box::new(k) },
                        ],
                    };
                    if inner == "A2xDyn" {
                        return self.coll_temp(id, "A2xDyn".into(), call, out);
                    }
                    let rec = format!("a2x_dyn_as_{}", if inner == "i32" { "int" } else { "bool" });
                    let got = self.coll_temp(id, "A2xDyn".into(), call, out)?;
                    return self.coll_temp(
                        id,
                        inner.to_string(),
                        RExpr::Call {
                            func: rec,
                            args: vec![RExpr::Ref { inner: Box::new(got) }],
                        },
                        out,
                    );
                }
                let l = self.place_ref(list, false, out)?;
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
                let l = self.place_ref(list, true, out)?;
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
                let l = self.place_ref(list, true, out)?;
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
                // PLAN-1116 (T-02): a take out of a generator state field
                // — resolve the root temp's HIR initializer to its field
                // (the enclosing Assign's target registers the write-back
                // mapping). The HIR scan (not the emitted stmts) sees
                // through nested blocks.
                if let hir::ExprKind::ReadLocal { local: r } =
                    &self.f.body.exprs[root.0 as usize].kind
                {
                    if let Some(init) = self.hir_local_binding(*r) {
                        if let Some((state, field)) = self.state_field_of_expr(init) {
                            self.pending_project = Some((state, field, vname.clone()));
                        }
                    }
                }
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
        // PLAN-1116 (T-02): an owned slot CONSUMES its operand — a local
        // read here is a move (the yield-value into `Some(..)`, record
        // fields): the source dies and the verifier's exit-edge frees
        // must skip it. Marked moved AFTER the read is lowered (the
        // use-after-move guard must not see this very read).
        let moved_owned_read = match &ce.kind {
            hir::ExprKind::ReadLocal { local } => {
                self.local_ty(*local)
                    .ok()
                    .and_then(|ty| self.owned_kind_of_ty(ty))
                    .map(|_| local.0)
            }
            _ => None,
        };
        let lowered = self.expr(child, out)?;
        if let Some(local) = moved_owned_read {
            self.moved_locals.insert(local);
        }
        Ok(lowered)
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
                        // PLAN-1116 T-02: a closure body's CAPTURE params
                        // render as borrows too (`&'a str`, the same
                        // mode-legalized contract as plain params) — a
                        // captured text is already a view; `.as_str()` on
                        // it is the unstable str method.
                        matches!(d.role, hir::LocalRole::Param | hir::LocalRole::Capture)
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

/// PLAN-1116 (T-02): find a single-segment field path rooted at a local
/// inside an expression (the clone receiver `&(state.field)` of a
/// generator state-field read — possibly under a fallibility Wrap or a
/// clone call's arguments).
fn field_path_in(e: &RExpr) -> Option<(String, String)> {
    match e {
        RExpr::FieldPath { base, path } => {
            if let RExpr::Local(state) = &**base {
                if path.len() == 1 {
                    return Some((state.clone(), path[0].clone()));
                }
            }
            field_path_in(base)
        }
        RExpr::Ref { inner } | RExpr::RefMut { inner } | RExpr::Wrap { inner, .. } => {
            field_path_in(inner)
        }
        RExpr::CollCall { args, .. } | RExpr::Call { args, .. } | RExpr::PlainCall { args, .. } => {
            args.iter().find_map(field_path_in)
        }
        RExpr::Local(_) | RExpr::I32Lit(_) | RExpr::BoolLit(_) | RExpr::StrLit(_)
        | RExpr::CharLit(_) | RExpr::GlobalRead { .. } | RExpr::GlobalWrite { .. }
        | RExpr::UnitExpr => None,
        _ => None,
    }
}

/// PLAN-968: the synthesized `fn(Vec<SchedValue>)` adapter that runs a
/// task's driver with typed initializers (built by plain concatenation —
/// no brace escaping).
fn sched_task_adapter_text(task: &str, field_tys: &[Ty]) -> String {    let mut b = String::new();
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

/// PLAN-1116 T-02: the typed wrap helper name for an adapter result.
fn wrap_helper(t: &Ty) -> &'static str {
    match t {
        Ty::Bool => "a2x_v_b",
        Ty::Str | Ty::StrOwned => "a2x_v_s",
        _ => "a2x_v_i",
    }
}

/// PLAN-1116 T-02: the synthesized `fn(Vec<SchedValue>) -> SchedValue`
/// adapter that runs an async block's thunk with its captures (the env
/// rides the argument vec; the result joins the scheduler value domain).
/// The thunk is fallible like every language-channel function: a trap in
/// an async body is published as the process-level sched failure (a
/// catchable cross-thread trap propagation awaits the catch-chain unit,
/// the PLAN-1115 named debt).
fn sched_future_adapter_text(name: &str, thunk: &str, env_tys: &[Ty], result: &Ty) -> String {
    let mut b = String::new();
    b.push_str("fn __sched_future_adapter_");
    b.push_str(name);
    b.push_str(
        "(a: ::std::vec::Vec<crate::a2x_runtime::SchedValue>) -> crate::a2x_runtime::SchedValue {",
    );
    b.push('\n');
    b.push_str("    let _ = &a;");
    b.push('\n');
    b.push_str("    let __result = ");
    b.push_str(thunk);
    b.push('(');
    for (k, t) in env_tys.iter().enumerate() {
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
    b.push_str(");\n");
    b.push_str("    let __ok = match __result {\n        Ok(v) => v,\n        Err(trap) => a2x_sched_fail(&format!(\"sched: async body trapped: {}\", trap.name())),\n    };\n    ");
    b.push_str(wrap_helper(result));
    b.push_str("(__ok)\n}");
    b.push('\n');
    b
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
