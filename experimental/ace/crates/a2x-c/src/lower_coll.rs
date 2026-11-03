//! PLAN-926 CA-C collections lowering: HIR (collections domain) → C11
//! statement lines, driven ENTIRELY by the verifier's collections body
//! facts (own/consume/exit responsibility — PLAN-904 §5.3/§5.4, the C
//! representation/responsibility/budget freeze of PLAN-926 T-02).
//!
//! The lowering never re-derives ownership: every free — normal exits
//! (`exit_frees`), assignment replacement (`replace_frees`), the loop
//! back-edge poll (per-iteration set + `loop_survivors`), and the error
//! path of every fallible node (`err_frees`) — comes from
//! [`ace_core::collections_facts::CollectionsBodyFacts`]. Representation
//! follows the T-02 freeze: nominal `a2x_list_<etag>` structs with typed
//! per-op helpers, `a2x_var_<name>` tagged unions whose `disc` guards
//! every access, `a2x_rec_<name>` structs, explicit IDEMPOTENT frees
//! only (a second free of the same slot is a no-op — the loop back-edge
//! poll's error path deliberately re-lists the per-iteration set).
//! Fallible nodes materialize their values in `e<expr-id>` temps so the
//! verifier's free schedule can address them by name; a consuming op
//! (push/insert/set) moves its owned input into the helper BY VALUE and
//! the helper owns the uncommitted-input free on its own failure path
//! (the temp drops out of the caller-side error frees — the
//! `wrap_consume` protocol). Billing sits inside the runtime/typed
//! helpers (charge before work, fault gate before the request, poll at
//! loop back-edges); cleanup paths never bill, never poll, never
//! allocate.

use ace_a2x_core::{A2xReject, FnFacts, SemanticView, Ty};
use ace_core::collections_facts::{CollectionsBodyFacts, FreeRef, OwnedKind};
use ace_core::hir;

use crate::lower::{c_fn_name, sanitize};
use crate::lower_rt::c_record_name;
use crate::seed_coll::{c_variant_name, SeedCollTy};

/// The result-wrapper kind of a collections function return type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollRetKind {
    I32,
    Bool,
    StrOwned,
    Record(String),
    Variant(String),
}

impl CollRetKind {
    /// The C return type of a function with this result kind.
    pub fn c_ret_ty(&self) -> String {
        match self {
            CollRetKind::I32 => "a2x_result".into(),
            CollRetKind::Bool => "a2x_bool_result".into(),
            CollRetKind::StrOwned => "a2x_stro_result".into(),
            CollRetKind::Record(n) => format!("a2x_res_{}", c_record_name(n)),
            CollRetKind::Variant(n) => format!("a2x_res_{}", c_variant_name(n)),
        }
    }

    /// The runtime forwarder constructing a trapped result of this kind.
    fn fwd_fn(&self) -> String {
        match self {
            CollRetKind::I32 => "a2x_fwd_i32".into(),
            CollRetKind::Bool => "a2x_fwd_bool".into(),
            CollRetKind::StrOwned => "a2x_fwd_stro".into(),
            CollRetKind::Record(n) => format!("a2x_fwd_{}", c_record_name(n)),
            CollRetKind::Variant(n) => format!("a2x_fwd_{}", c_variant_name(n)),
        }
    }

    /// The value constructor of this kind (the `return` path).
    fn value_fn(&self) -> String {
        match self {
            CollRetKind::I32 => "a2x_value".into(),
            CollRetKind::Bool => "a2x_bool_value".into(),
            CollRetKind::StrOwned => "a2x_stro_value".into(),
            CollRetKind::Record(n) => format!("a2x_{}_value", c_record_name(n)),
            CollRetKind::Variant(n) => format!("a2x_{}_value", c_variant_name(n)),
        }
    }
}

fn ret_kind_of(view: &SemanticView<'_>, ty: Ty) -> Result<CollRetKind, A2xReject> {
    match ty {
        Ty::I32 => Ok(CollRetKind::I32),
        Ty::Bool => Ok(CollRetKind::Bool),
        Ty::StrOwned => Ok(CollRetKind::StrOwned),
        Ty::Record(i) => view
            .record(i)
            .map(|r| CollRetKind::Record(r.name.clone()))
            .ok_or_else(|| A2xReject::unsupported_type("unknown record result type")),
        Ty::Variant(i) => view
            .variant(i)
            .map(|v| CollRetKind::Variant(v.name.clone()))
            .ok_or_else(|| A2xReject::unsupported_type("unknown variant result type")),
        other => Err(A2xReject::unsupported_type(format!(
            "result type `{:?}` outside the C collections channel",
            other
        ))),
    }
}

/// One lowered C collections function.
#[derive(Clone, Debug)]
pub struct CFnColl {
    pub name: String,
    pub ret: CollRetKind,
    /// (c type, c name) per parameter, in parameter order.
    pub params: Vec<(String, String)>,
    pub origin: (u32, u32),
    /// The function body statements (indentation-free lines).
    pub body: Vec<String>,
}

/// Render one fragment: origin comment + prototype + definition. The
/// prototype line is the assembler's forward-declaration input.
pub fn print_fragment_coll(f: &CFnColl) -> String {
    let params = if f.params.is_empty() {
        "void".to_string()
    } else {
        f.params
            .iter()
            .map(|(t, n)| format!("{} {}", t, n))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let ret = f.ret.c_ret_ty();
    let mut out = format!(
        "// origin: {}..{} (def `{}`, byte span in the compiled source)\n",
        f.origin.0, f.origin.1, f.name
    );
    out.push_str(&format!("{} {}({});\n", ret, f.name, params));
    out.push_str(&format!("{} {}({}) {{\n", ret, f.name, params));
    for line in &f.body {
        out.push_str("    ");
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

/// How a param local is passed at the C ABI.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParamPass {
    /// By value of the value type (i32/bool/move Str/records/variants).
    Value,
    /// `const a2x_rec_X*` / `const a2x_var_X*` — view params.
    ConstPtr,
    /// `a2x_rec_X*` / `a2x_var_X*` — mut params (writeback through the
    /// pointer).
    MutPtr,
}

fn local_name(name: &str, id: u32) -> String {
    format!("l{}_{}", id, sanitize(name))
}

/// The C value type of a collections type (locals, fields, payloads).
pub fn coll_c_value_ty(view: &SemanticView<'_>, ty: Ty) -> Result<String, A2xReject> {
    match ty {
        Ty::I32 => Ok("int32_t".into()),
        Ty::Bool => Ok("bool".into()),
        Ty::StrOwned => Ok("a2x_str".into()),
        Ty::Record(i) => view
            .record(i)
            .map(|r| c_record_name(&r.name))
            .ok_or_else(|| A2xReject::unsupported_type("unknown record type in value position")),
        Ty::Variant(i) => view
            .variant(i)
            .map(|v| c_variant_name(&v.name))
            .ok_or_else(|| A2xReject::unsupported_type("unknown variant type in value position")),
        Ty::List(i) => {
            let elem = view.list_elem(i)?;
            Ok(format!("a2x_list_{}", seed_ty_of(view, elem)?.elem_tag()))
        }
        other => Err(A2xReject::unsupported_type(format!(
            "the C collections channel does not serve `{:?}` in value position",
            other
        ))),
    }
}

pub(crate) fn seed_ty_of(view: &SemanticView<'_>, ty: Ty) -> Result<SeedCollTy, A2xReject> {
    match ty {
        Ty::I32 => Ok(SeedCollTy::I32),
        Ty::Bool => Ok(SeedCollTy::Bool),
        Ty::StrOwned => Ok(SeedCollTy::StrOwned),
        Ty::Record(i) => view
            .record(i)
            .map(|r| SeedCollTy::Record(r.name.clone()))
            .ok_or_else(|| A2xReject::new("a2x.internal", "unknown record index")),
        Ty::Variant(i) => view
            .variant(i)
            .map(|v| SeedCollTy::Variant(v.name.clone()))
            .ok_or_else(|| A2xReject::new("a2x.internal", "unknown variant index")),
        other => Err(A2xReject::unsupported_type(format!(
            "type `{:?}` is outside the collections domain",
            other
        ))),
    }
}

/// Lower every function of a collections view into C fragments.
pub fn lower_coll_functions_c(view: &SemanticView<'_>) -> Result<Vec<CFnColl>, A2xReject> {
    let mut out = Vec::new();
    for f in view.functions() {
        out.push(lower_coll_fn_c(view, f)?);
    }
    Ok(out)
}

/// Lower one function of the collections channel.
pub fn lower_coll_fn_c(view: &SemanticView<'_>, f: &FnFacts<'_>) -> Result<CFnColl, A2xReject> {
    let facts = view.body_facts(f).ok_or_else(|| {
        A2xReject::new(
            "a2x.internal",
            format!(
                "collections lowering requires the verifier's body facts for `{}`",
                f.source_name
            ),
        )
    })?;
    let ret = ret_kind_of(view, f.result)?;
    // params follow the HIR local table (l<LocalId>_<name>) so body
    // references and the signature agree on one naming
    let mut param_locals: Vec<(u32, &hir::LocalDef)> = f
        .body
        .locals
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.role, hir::LocalRole::Param))
        .map(|(i, l)| (i as u32, l))
        .collect();
    param_locals.sort_by_key(|(_, l)| l.index.unwrap_or(u32::MAX));
    let mut params = Vec::new();
    let mut passes = std::collections::HashMap::new();
    for (p, (idx, l)) in f.params.iter().zip(param_locals.iter()) {
        let ty = view.resolve_use_ty(l.ty)?;
        let pass = match (p.mode, ty) {
            // a scalar view is its ABI copy (mirror of the Rust leg's
            // by-value scalar params in every mode)
            (_, Ty::I32 | Ty::Bool) => ParamPass::Value,
            (hir::ParamMode::MoveOwn, _) => ParamPass::Value,
            (hir::ParamMode::View, Ty::Record(_) | Ty::Variant(_)) => ParamPass::ConstPtr,
            (hir::ParamMode::MutBorrow, Ty::Record(_) | Ty::Variant(_)) => ParamPass::MutPtr,
            (hir::ParamMode::View | hir::ParamMode::MutBorrow, other) => {
                return Err(A2xReject::unsupported_type(format!(
                    "view/mut param of `{:?}` is outside the collections domain",
                    other
                )))
            }
        };
        let cty = match pass {
            ParamPass::Value => coll_c_value_ty(view, ty)?,
            ParamPass::ConstPtr => format!("const {}*", coll_c_value_ty(view, ty)?),
            ParamPass::MutPtr => format!("{}*", coll_c_value_ty(view, ty)?),
        };
        passes.insert(*idx, pass);
        params.push((cty, local_name(&l.name, *idx)));
    }
    let mut w = CollLowerC {
        view,
        f,
        facts,
        ret_fwd: ret.fwd_fn(),
        ret_value: ret.value_fn(),
        locals: f.body.locals.clone(),
        passes,
        out: Vec::new(),
        temp_counter: 0,
    };
    // callee-frame work charge (4 WU per call frame); its failure path
    // frees nothing (the frame is empty at entry)
    let t = w.fresh_temp();
    w.out.push(format!(
        "{{ a2x_trap_code {t} = a2x_coll_charge_call(); if ({t} != A2X_TRAP_NONE) {{ return {fwd}({t}); }} }}",
        t = t,
        fwd = w.ret_fwd
    ));
    w.block(f.body.entry)?;
    Ok(CFnColl {
        name: c_fn_name(f.source_name),
        ret,
        params,
        origin: f.origin,
        body: w.out,
    })
}

struct CollLowerC<'a, 'v> {
    view: &'v SemanticView<'a>,
    f: &'v FnFacts<'a>,
    facts: &'v CollectionsBodyFacts,
    ret_fwd: String,
    ret_value: String,
    locals: Vec<hir::LocalDef>,
    passes: std::collections::HashMap<u32, ParamPass>,
    out: Vec<String>,
    temp_counter: usize,
}

impl<'a, 'v> CollLowerC<'a, 'v> {
    fn fresh_temp(&mut self) -> String {
        let t = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        t
    }

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

    /// The reading expression of a local (pointer params deref).
    fn local_read(&self, id: u32) -> String {
        let name = self.local_name_of(hir::LocalId(id));
        match self.passes.get(&id) {
            Some(ParamPass::ConstPtr) | Some(ParamPass::MutPtr) => format!("(*{})", name),
            _ => name,
        }
    }

    fn seed_ty(&self, ty: Ty) -> Result<SeedCollTy, A2xReject> {
        seed_ty_of(self.view, ty)
    }

    /// The owned-kind classification of a type (mirror of the facts'
    /// TypeClassifier, resolved through the view).
    fn owned_kind_of_ty(&self, ty: Ty) -> Option<OwnedKind> {
        match ty {
            Ty::StrOwned => Some(OwnedKind::Str),
            Ty::List(_) => Some(OwnedKind::List),
            Ty::Record(id) => {
                let rec = self.view.record(id)?;
                rec.resource.then_some(OwnedKind::Deep(hir::TypeId(id)))
            }
            Ty::Variant(id) => {
                let v = self.view.variant(id)?;
                let owns = v.arms.iter().any(|a| {
                    a.payload
                        .as_ref()
                        .and_then(|p| self.owned_kind_of_ty(*p))
                        .is_some()
                });
                owns.then_some(OwnedKind::Deep(hir::TypeId(id)))
            }
            _ => None,
        }
    }

    /// The free-helper call of an owned kind (idempotent frees by
    /// construction — the T-02 freeze).
    fn free_call(&self, kind: OwnedKind, ty_of_local: Option<Ty>) -> String {
        match kind {
            OwnedKind::Str => "a2x_str_free".into(),
            OwnedKind::List => {
                if let Some(Ty::List(li)) = ty_of_local {
                    if let Ok(elem) = self.view.list_elem(li) {
                        if let Ok(tag) = self.seed_ty(elem).map(|t| t.elem_tag()) {
                            return format!("a2x_coll_list_{}_free", tag);
                        }
                    }
                }
                "a2x_coll_list_free".into()
            }
            OwnedKind::Deep(tid) => {
                if let Some(r) = self.view.record(tid.0) {
                    format!("a2x_coll_free_rec_{}", sanitize(&r.name))
                } else if let Some(v) = self.view.variant(tid.0) {
                    format!("a2x_coll_free_var_{}", sanitize(&v.name))
                } else {
                    format!("a2x_coll_free_T{}", tid.0)
                }
            }
        }
    }

    /// Render one scheduled free set as statements.
    fn free_stmts(&self, frees: &[FreeRef]) -> Vec<String> {
        frees.iter().filter_map(|r| self.free_stmt_of(r)).collect()
    }

    fn free_stmt_of(&self, r: &FreeRef) -> Option<String> {
        match r {
            FreeRef::Local(l) => {
                let ty = self.local_ty(hir::LocalId(*l)).ok()?;
                let kind = self.owned_kind_of_ty(ty)?;
                Some(format!(
                    "{}(&{});",
                    self.free_call(kind, Some(ty)),
                    self.local_name_of(hir::LocalId(*l))
                ))
            }
            FreeRef::Temp(e, kind) => Some(format!("{}(&e{});", self.free_call(*kind, None), e)),
        }
    }

    /// The err_frees of a node as statements, optionally EXCLUDING a
    /// consumed operand temp (the `wrap_consume` protocol: the helper
    /// owns its failure-path free of the moved input).
    fn err_frees_of(&self, expr_id: u32, exclude: Option<u32>) -> Vec<String> {
        match self.facts.err_frees.get(&expr_id) {
            Some(set) => set
                .iter()
                .filter(|r| match r {
                    FreeRef::Temp(e, _) => Some(*e) != exclude,
                    _ => true,
                })
                .filter_map(|r| self.free_stmt_of(r))
                .collect(),
            None => Vec::new(),
        }
    }

    /// The one-line trap-check block of a trap-code call:
    /// `{ a2x_trap_code t<n> = <call>; if (t<n> != A2X_TRAP_NONE)
    /// { <frees> return <fwd>(t<n>); } }`.
    fn trap_check(&mut self, call: String, frees: &[String]) {
        let t = self.fresh_temp();
        let frees = if frees.is_empty() {
            String::new()
        } else {
            format!(" {}", frees.join(" "))
        };
        self.out.push(format!(
            "{{ a2x_trap_code {t} = {call}; if ({t} != A2X_TRAP_NONE) {{{frees} return {fwd}({t}); }} }}",
            t = t,
            call = call,
            frees = frees,
            fwd = self.ret_fwd
        ));
    }

    /// Emit a fallible TRAP-CODE call with an out-param value: declare
    /// `e<id>` of `ety`, call with `&e<id>` spliced by the caller, then
    /// the trap check with the node's err frees.
    fn fall_out_pre(&mut self, id: hir::ExprId, ety: &str) -> String {
        let name = format!("e{}", id.0);
        self.out.push(format!("{} {};", ety, name));
        name
    }

    /// Emit a fallible STRUCT-RESULT call (checked i32, str len, user
    /// calls): declare `e<id>` of the result type, call, trap-check.
    /// Returns the `.value` reading expression.
    fn fall_res(&mut self, id: hir::ExprId, rty: &str, call: String) -> Result<String, A2xReject> {
        let name = format!("e{}", id.0);
        let frees = self.err_frees_of(id.0, None);
        let frees = if frees.is_empty() {
            String::new()
        } else {
            format!(" {}", frees.join(" "))
        };
        self.out.push(format!(
            "{rty} {name} = {call}; if ({name}.trap != A2X_TRAP_NONE) {{{frees} return {fwd}({name}.trap); }}",
            rty = rty,
            name = name,
            call = call,
            frees = frees,
            fwd = self.ret_fwd
        ));
        Ok(format!("{}.value", name))
    }

    fn block(&mut self, block: hir::BlockId) -> Result<(), A2xReject> {
        let stmts = self.f.body.blocks[block.0 as usize].stmts.clone();
        for (si, stmt) in stmts.iter().enumerate() {
            self.stmt(block.0, si, stmt)?;
        }
        // fall-through end: the scoped deaths of locals defined in this
        // block (exit_frees[(block, None)]) — a trailing terminator
        // carries its own exit key and leaves nothing fall-through-live
        let terminated = matches!(
            stmts.last(),
            Some(hir::Stmt::Return { .. })
                | Some(hir::Stmt::Break { .. })
                | Some(hir::Stmt::Continue { .. })
        );
        if !terminated {
            if let Some(frees) = self.facts.exit_frees.get(&(block.0, None)) {
                for line in self.free_stmts(frees) {
                    self.out.push(line);
                }
            }
        }
        Ok(())
    }

    fn stmt(&mut self, block: u32, si: usize, s: &hir::Stmt) -> Result<(), A2xReject> {
        match s {
            hir::Stmt::Let { local, value } => {
                let ty = self.local_ty(*local)?;
                let cty = coll_c_value_ty(self.view, ty)?;
                // an owned-position literal promotes (a2x_str_from_lit)
                let v = if is_lit(self.f.body, *value) && matches!(ty, Ty::StrOwned) {
                    self.owned_or_plain_child(*value)?
                } else {
                    self.expr(*value)?
                };
                let name = self.local_name_of(*local);
                self.out.push(format!("{} {} = {};", cty, name, v));
                Ok(())
            }
            hir::Stmt::Return { value } => {
                let owned_ret = self
                    .view
                    .resolve_use_ty(self.f.body.exprs[value.0 as usize].ty)
                    .map(|t| matches!(t, Ty::StrOwned))
                    .unwrap_or(false);
                let v = if is_lit(self.f.body, *value) && owned_ret {
                    self.owned_or_plain_child(*value)?
                } else {
                    self.expr(*value)?
                };
                if let Some(frees) = self.facts.exit_frees.get(&(block, Some(si))) {
                    for line in self.free_stmts(frees) {
                        self.out.push(line);
                    }
                }
                self.out.push(format!("return {}({});", self.ret_value, v));
                Ok(())
            }
            hir::Stmt::Assign { place, value } => {
                let p = &self.f.body.places[place.0 as usize];
                if !matches!(p.kind, hir::PlaceKind::Local) {
                    return Err(A2xReject::unsupported_place(
                        "collections field-path assignment is outside the surface",
                    ));
                }
                let ty = self.local_ty(p.local)?;
                let owned_place = matches!(ty, Ty::StrOwned);
                let v = if is_lit(self.f.body, *value) && owned_place {
                    self.owned_or_plain_child(*value)?
                } else {
                    self.expr(*value)?
                };
                // the OLD value frees at commit — after the RHS fully
                // materialized (the replace_frees schedule)
                if self.facts.replace_frees.contains_key(&(block, si)) {
                    if let Some(kind) = self.owned_kind_of_ty(ty) {
                        self.out.push(format!(
                            "{}(&{});",
                            self.free_call(kind, Some(ty)),
                            self.local_name_of(p.local)
                        ));
                    }
                }
                self.out
                    .push(format!("{} = {};", self.local_name_of(p.local), v));
                Ok(())
            }
            hir::Stmt::If { cond, then, els } => {
                let c = self.expr(*cond)?;
                self.out.push(format!("if ({}) {{", c));
                self.block(*then)?;
                if let Some(eb) = els {
                    self.out.push("} else {".to_string());
                    self.block(*eb)?;
                }
                self.out.push("}".to_string());
                Ok(())
            }
            hir::Stmt::Loop { body: lb, .. } => {
                self.out.push("for (;;) {".to_string());
                self.block(*lb)?;
                // back-edge: the body block's fall-through scoped deaths
                // were already emitted by `block()`; the cooperative
                // poll's error path re-lists them plus the loop
                // survivors (idempotent frees make the overlap safe —
                // the T-02 freeze)
                let mut poll_frees: Vec<String> = Vec::new();
                if let Some(frees) = self.facts.exit_frees.get(&(lb.0, None)) {
                    poll_frees.extend(self.free_stmts(frees));
                }
                if let Some(surv) = self.facts.loop_survivors.get(&lb.0) {
                    for l in surv {
                        if let Ok(ty) = self.local_ty(hir::LocalId(*l)) {
                            if let Some(kind) = self.owned_kind_of_ty(ty) {
                                poll_frees.push(format!(
                                    "{}(&{});",
                                    self.free_call(kind, Some(ty)),
                                    self.local_name_of(hir::LocalId(*l))
                                ));
                            }
                        }
                    }
                }
                let t = self.fresh_temp();
                let frees = if poll_frees.is_empty() {
                    String::new()
                } else {
                    format!(" {}", poll_frees.join(" "))
                };
                self.out.push(format!(
                    "{{ a2x_trap_code {t} = a2x_coll_poll(); if ({t} != A2X_TRAP_NONE) {{{frees} return {fwd}({t}); }} }}",
                    t = t,
                    frees = frees,
                    fwd = self.ret_fwd
                ));
                self.out.push("}".to_string());
                Ok(())
            }
            hir::Stmt::Break { .. } => {
                if let Some(frees) = self.facts.exit_frees.get(&(block, Some(si))) {
                    for line in self.free_stmts(frees) {
                        self.out.push(line);
                    }
                }
                self.out.push("break;".to_string());
                Ok(())
            }
            hir::Stmt::Continue { .. } => {
                if let Some(frees) = self.facts.exit_frees.get(&(block, Some(si))) {
                    for line in self.free_stmts(frees) {
                        self.out.push(line);
                    }
                }
                self.out.push("continue;".to_string());
                Ok(())
            }
            hir::Stmt::Eval { value } => {
                // side statements and trap checks already emitted inside
                // the lowering; the value (if any) is dropped
                self.expr(*value)?;
                Ok(())
            }
            hir::Stmt::Drop { .. } => Err(A2xReject::unsupported_stmt(
                "explicit Drop is outside the collections lowering surface",
            )),
        }
    }

    /// A child in an owned-value slot: owned-producing children lower to
    /// their temps; a literal promotes through `a2x_str_from_lit` into a
    /// named `e<child>` temp exactly where the facts model the pending.
    fn owned_or_plain_child(&mut self, child: hir::ExprId) -> Result<String, A2xReject> {
        if let hir::ExprKind::StrLit { value } = &self.f.body.exprs[child.0 as usize].kind {
            let lit = c_str_lit(value);
            let name = self.fall_out_pre(child, "a2x_str");
            let frees = self.err_frees_of(child.0, None);
            self.trap_check(
                format!(
                    "a2x_str_from_lit({lit}.data, {lit}.len, &{name})",
                    lit = lit,
                    name = name
                ),
                &frees,
            );
            return Ok(name);
        }
        self.expr(child)
    }

    /// A whole-local reference to a list operand (mutable or shared).
    fn list_place(&self, e: hir::ExprId) -> Result<(String, Ty), A2xReject> {
        match &self.f.body.exprs[e.0 as usize].kind {
            hir::ExprKind::ReadLocal { local } => {
                let ty = self.local_ty(*local)?;
                Ok((format!("&{}", self.local_name_of(*local)), ty))
            }
            other => Err(A2xReject::unsupported_expr(format!(
                "list operand must be a whole-local read (found `{:?}`)",
                other
            ))),
        }
    }

    fn list_tag_of(&self, ty: Ty) -> Result<String, A2xReject> {
        match ty {
            Ty::List(li) => {
                let elem = self.view.list_elem(li)?;
                Ok(self.seed_ty(elem)?.elem_tag())
            }
            other => Err(A2xReject::unsupported_type(format!(
                "expected list operand, found `{:?}`",
                other
            ))),
        }
    }

    fn elem_ty_of(&self, ty: Ty) -> Result<Ty, A2xReject> {
        match ty {
            Ty::List(li) => self.view.list_elem(li),
            other => Err(A2xReject::unsupported_type(format!(
                "expected list operand, found `{:?}`",
                other
            ))),
        }
    }

    /// A borrowed text operand: owned locals read as `&local`, literals
    /// as a compound-literal view, and checked-view POINTER temps pass
    /// as-is (no promotion — the facts walker does not model a temp
    /// here).
    fn text_operand(&mut self, operand: hir::ExprId) -> Result<String, A2xReject> {
        let o = self.expr(operand)?;
        if self.is_view_pointer_node(operand) {
            return Ok(o);
        }
        Ok(format!("(&{})", o))
    }

    fn expr(&mut self, id: hir::ExprId) -> Result<String, A2xReject> {
        let e = self
            .f
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
        let kind = e.kind.clone();
        match kind {
            hir::ExprKind::ReadLocal { local } => Ok(self.local_read(local.0)),
            hir::ExprKind::Constant { value } => Ok(value.to_string()),
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let l = self.expr(lhs)?;
                let r = self.expr(rhs)?;
                match op {
                    hir::BinOp::AddI32 => {
                        self.fall_res(id, "a2x_result", format!("a2x_add({l}, {r})"))
                    }
                    hir::BinOp::SubI32 => {
                        self.fall_res(id, "a2x_result", format!("a2x_sub({l}, {r})"))
                    }
                    hir::BinOp::MulI32 => {
                        self.fall_res(id, "a2x_result", format!("a2x_mul({l}, {r})"))
                    }
                    hir::BinOp::DivI32 => {
                        self.fall_res(id, "a2x_result", format!("a2x_div({l}, {r})"))
                    }
                    hir::BinOp::RemI32 => {
                        self.fall_res(id, "a2x_result", format!("a2x_rem({l}, {r})"))
                    }
                    hir::BinOp::LtI32 => Ok(format!("({l} < {r})")),
                    hir::BinOp::LeI32 => Ok(format!("({l} <= {r})")),
                    hir::BinOp::GtI32 => Ok(format!("({l} > {r})")),
                    hir::BinOp::GeI32 => Ok(format!("({l} >= {r})")),
                    hir::BinOp::Eq => {
                        let ty = self
                            .view
                            .resolve_use_ty(self.f.body.exprs[lhs.0 as usize].ty)?;
                        if matches!(ty, Ty::StrOwned) {
                            Ok(format!("a2x_str_eq(&{l}, &{r})"))
                        } else {
                            Ok(format!("({l} == {r})"))
                        }
                    }
                    hir::BinOp::Ne => {
                        let ty = self
                            .view
                            .resolve_use_ty(self.f.body.exprs[lhs.0 as usize].ty)?;
                        if matches!(ty, Ty::StrOwned) {
                            Ok(format!("a2x_str_ne(&{l}, &{r})"))
                        } else {
                            Ok(format!("({l} != {r})"))
                        }
                    }
                }
            }
            hir::ExprKind::Unary { op, operand } => {
                let o = self.expr(operand)?;
                match op {
                    hir::UnaryOp::NegI32 => {
                        self.fall_res(id, "a2x_result", format!("a2x_neg({o})"))
                    }
                    hir::UnaryOp::NotBool => Ok(format!("!({o})")),
                }
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                // bool operands are pure inline expressions in this
                // domain (comparisons / is-tests / logicals); a
                // side-statement operand would break C short-circuit
                let l = self.expr(lhs)?;
                let r = self.expr(rhs)?;
                Ok(match op {
                    hir::LogicOp::And => format!("({l} && {r})"),
                    hir::LogicOp::Or => format!("({l} || {r})"),
                })
            }
            hir::ExprKind::Call {
                callee, eval_args, ..
            } => {
                let mut args = Vec::new();
                for a in &eval_args {
                    args.push(self.expr(*a)?);
                }
                let target = callee.0 as usize;
                let func = if let Some(sym) = self.view.trace_symbol(target) {
                    let suffix = if sym == "hir.test.mark_a" { "a" } else { "b" };
                    format!("a2x_mark_{}", suffix)
                } else if let Some(t) = self.view.fn_by_def(target) {
                    c_fn_name(t.source_name)
                } else if let Some(x) = self.view.external_by_def(target) {
                    c_fn_name(&x.source_name)
                } else {
                    return Err(A2xReject::unsupported_expr("call target not admitted"));
                };
                let rty = ret_kind_of(
                    self.view,
                    self.view
                        .resolve_use_ty(self.f.body.exprs[id.0 as usize].ty)?,
                )?
                .c_ret_ty();
                self.fall_res(id, &rty, format!("{}({})", func, args.join(", ")))
            }
            hir::ExprKind::StrLit { value } => {
                // borrowed literal; owned positions promote via the parent
                Ok(c_str_lit(&value))
            }
            hir::ExprKind::TextLen { operand } => {
                let o = self.text_operand(operand)?;
                self.fall_res(id, "a2x_result", format!("a2x_str_len({o})"))
            }
            hir::ExprKind::TextLenBytes { operand } => {
                let o = self.text_operand(operand)?;
                self.fall_res(id, "a2x_result", format!("a2x_str_len_bytes({o})"))
            }
            hir::ExprKind::TextClone { operand } => {
                let o = self.expr(operand)?;
                let name = self.fall_out_pre(id, "a2x_str");
                let frees = self.err_frees_of(id.0, None);
                self.trap_check(
                    format!(
                        "a2x_str_clone(({o}).data, ({o}).len, &{name})",
                        o = o,
                        name = name
                    ),
                    &frees,
                );
                Ok(name)
            }
            hir::ExprKind::RecordCtor {
                record,
                eval_args,
                bindings,
            } => {
                // source-order argument temps first (owned/promoted
                // values materialize exactly where the facts model them)
                let mut arg_vals: Vec<String> = Vec::new();
                for a in &eval_args {
                    arg_vals.push(self.owned_or_plain_child(*a)?);
                }
                // ctor charge (fallible; frees = the pending arg temps)
                let frees = self.err_frees_of(id.0, None);
                self.trap_check("a2x_coll_charge_ctor()".to_string(), &frees);
                let rec = self
                    .view
                    .record(record.0)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "record out of range"))?;
                let rname = c_record_name(&rec.name);
                // the ctor value always materializes as a named temp (the
                // facts model it as pending when the record is owned)
                let name = format!("e{}", id.0);
                self.out.push(format!("{} {};", rname, name));
                for b in bindings {
                    let fname = rec
                        .fields
                        .get(b.field as usize)
                        .map(|f| format!("f_{}", sanitize(&f.name)))
                        .ok_or_else(|| {
                            A2xReject::new("a2x.internal", "field index out of range")
                        })?;
                    let v = arg_vals
                        .get(b.arg as usize)
                        .cloned()
                        .unwrap_or_else(|| "0".to_string());
                    self.out.push(format!("{}.{} = {};", name, fname, v));
                }
                Ok(name)
            }
            hir::ExprKind::FieldGet { base, path } => {
                let b = self.expr(base)?;
                // a checked-view base (an owned-element ListGet) is a
                // POINTER temp — deref before the first field hop
                let b = if self.is_view_pointer_node(base) {
                    format!("(*{})", b)
                } else {
                    b
                };
                let mut base_ty = self
                    .view
                    .resolve_use_ty(self.f.body.exprs[base.0 as usize].ty)?;
                let mut place = b;
                for fid in &path {
                    let Ty::Record(rid) = base_ty else {
                        return Err(A2xReject::unsupported_type(
                            "field path through a non-record",
                        ));
                    };
                    let rec = self
                        .view
                        .record(rid)
                        .ok_or_else(|| A2xReject::new("a2x.internal", "record out of range"))?;
                    let fld = rec.fields.get(fid.0 as usize).ok_or_else(|| {
                        A2xReject::new("a2x.internal", "field index out of range")
                    })?;
                    place.push('.');
                    place.push_str(&format!("f_{}", sanitize(&fld.name)));
                    base_ty = fld.ty;
                }
                Ok(place)
            }
            hir::ExprKind::BorrowView { base } => {
                let b = self.expr(base)?;
                Ok(format!("&{}", b))
            }
            hir::ExprKind::BorrowMut { local } => Ok(format!("&{}", self.local_name_of(local))),
            hir::ExprKind::Move { inner } => self.expr(inner),
            hir::ExprKind::ListNew { list } => {
                let elem = self.view.list_elem(list.0)?;
                let tag = self.seed_ty(elem)?.elem_tag();
                let name = self.fall_out_pre(id, &format!("a2x_list_{}", tag));
                let frees = self.err_frees_of(id.0, None);
                self.trap_check(
                    format!("a2x_coll_list_{tag}_new(&{name})", tag = tag, name = name),
                    &frees,
                );
                Ok(name)
            }
            hir::ExprKind::ListLen { list } => {
                let (l, ty) = self.list_place(list)?;
                let tag = self.list_tag_of(ty)?;
                Ok(format!("a2x_coll_list_{}_len({})", tag, l))
            }
            hir::ExprKind::ListPush { list, value } => {
                let (l, ty) = self.list_place(list)?;
                let tag = self.list_tag_of(ty)?;
                let v = self.owned_or_plain_child(value)?;
                let frees = self.err_frees_of(id.0, Some(value.0));
                self.trap_check(
                    format!(
                        "a2x_coll_list_{tag}_push({l}, {v})",
                        tag = tag,
                        l = l,
                        v = v
                    ),
                    &frees,
                );
                Ok("void".to_string())
            }
            hir::ExprKind::ListInsert { list, index, value } => {
                let (l, ty) = self.list_place(list)?;
                let tag = self.list_tag_of(ty)?;
                let i = self.expr(index)?;
                let v = self.owned_or_plain_child(value)?;
                let frees = self.err_frees_of(id.0, Some(value.0));
                self.trap_check(
                    format!(
                        "a2x_coll_list_{tag}_insert({l}, {i}, {v})",
                        tag = tag,
                        l = l,
                        i = i,
                        v = v
                    ),
                    &frees,
                );
                Ok("void".to_string())
            }
            hir::ExprKind::ListSet { list, index, value } => {
                let (l, ty) = self.list_place(list)?;
                let tag = self.list_tag_of(ty)?;
                let i = self.expr(index)?;
                let v = self.owned_or_plain_child(value)?;
                let frees = self.err_frees_of(id.0, Some(value.0));
                self.trap_check(
                    format!(
                        "a2x_coll_list_{tag}_set({l}, {i}, {v})",
                        tag = tag,
                        l = l,
                        i = i,
                        v = v
                    ),
                    &frees,
                );
                Ok("void".to_string())
            }
            hir::ExprKind::ListGet { list, index } => {
                let (l, ty) = self.list_place(list)?;
                let tag = self.list_tag_of(ty)?;
                let elem = self.elem_ty_of(ty)?;
                let i = self.expr(index)?;
                let ety = coll_c_value_ty(self.view, elem)?;
                let frees = self.err_frees_of(id.0, None);
                let name = if self.is_copy(elem) {
                    // Copy element: a checked value OUT (a snapshot copy)
                    let n = self.fall_out_pre(id, &ety);
                    self.trap_check(
                        format!(
                            "a2x_coll_list_{tag}_get({l}, {i}, &{n})",
                            tag = tag,
                            l = l,
                            i = i,
                            n = n
                        ),
                        &frees,
                    );
                    n
                } else {
                    // owned element: the CHECKED VIEW (a const pointer; no
                    // materialization, no clone)
                    let n = self.fall_out_pre(id, &format!("const {}*", ety));
                    self.trap_check(
                        format!(
                            "a2x_coll_list_{tag}_view({l}, {i}, &{n})",
                            tag = tag,
                            l = l,
                            i = i,
                            n = n
                        ),
                        &frees,
                    );
                    n
                };
                Ok(name)
            }
            hir::ExprKind::ListRemove { list, index } => {
                let (l, ty) = self.list_place(list)?;
                let tag = self.list_tag_of(ty)?;
                let elem = self.elem_ty_of(ty)?;
                let i = self.expr(index)?;
                let ety = coll_c_value_ty(self.view, elem)?;
                let name = self.fall_out_pre(id, &ety);
                let frees = self.err_frees_of(id.0, None);
                self.trap_check(
                    format!(
                        "a2x_coll_list_{tag}_remove({l}, {i}, &{name})",
                        tag = tag,
                        l = l,
                        i = i,
                        name = name
                    ),
                    &frees,
                );
                Ok(name)
            }
            hir::ExprKind::ListClear { list } => {
                let (l, ty) = self.list_place(list)?;
                let tag = self.list_tag_of(ty)?;
                self.out
                    .push(format!("a2x_coll_list_{}_clear({});", tag, l));
                Ok("void".to_string())
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
                    Some(p) => Some(self.owned_or_plain_child(p)?),
                    None => None,
                };
                let frees = self.err_frees_of(id.0, None);
                self.trap_check("a2x_coll_charge_ctor()".to_string(), &frees);
                // the variant value always materializes as a named temp;
                // zero the union then set the ACTIVE arm only (no
                // unguarded inactive-field access ever executes)
                let name = format!("e{}", id.0);
                let vname = c_variant_name(&v.name);
                self.out.push(format!("{} {};", vname, name));
                self.out.push(format!(
                    "memset(&{n}.as, 0, sizeof {n}.as); {n}.disc = {d};",
                    n = name,
                    d = disc
                ));
                if let (Some(pv), Some(_)) = (payload_expr.as_ref(), arm.payload.as_ref()) {
                    self.out
                        .push(format!("{}.as.p_{} = {};", name, sanitize(&arm.name), pv));
                }
                Ok(name)
            }
            hir::ExprKind::VariantGuardTest { root, disc } => {
                let r = self.expr(root)?;
                let r = if self.is_view_pointer_node(root) {
                    format!("(*{})", r)
                } else {
                    r
                };
                Ok(format!("({r}.disc == {})", disc))
            }
            hir::ExprKind::VariantProject { root, disc } => {
                let (vname, aname, pty) = self.variant_arm_of(root, disc)?;
                // the root must be addressable (a whole-local read): the
                // take MOVES the payload out and resets the disc — a
                // wrong arm leaves the root untouched (the caller's
                // error frees still own it)
                let root_ref = match &self.f.body.exprs[root.0 as usize].kind {
                    hir::ExprKind::ReadLocal { local } => {
                        format!("&{}", self.local_name_of(*local))
                    }
                    _ => {
                        return Err(A2xReject::unsupported_expr(
                            "variant projection root must be a whole-local read",
                        ))
                    }
                };
                let ety = coll_c_value_ty(self.view, pty)?;
                let name = self.fall_out_pre(id, &ety);
                let frees = self.err_frees_of(id.0, None);
                self.trap_check(
                    format!(
                        "a2x_coll_take_{v}_{a}({root}, &{name})",
                        v = sanitize(&vname),
                        a = sanitize(&aname),
                        root = root_ref,
                        name = name
                    ),
                    &frees,
                );
                Ok(name)
            }
            other => Err(A2xReject::unsupported_expr(format!(
                "expression kind `{:?}` outside the C collections surface",
                other
            ))),
        }
    }

    fn is_copy(&self, ty: Ty) -> bool {
        matches!(ty, Ty::I32 | Ty::Bool)
    }

    /// Whether this node's lowered value is a checked-view POINTER temp
    /// (an owned-element ListGet) — consumers must deref once.
    fn is_view_pointer_node(&self, id: hir::ExprId) -> bool {
        match &self.f.body.exprs[id.0 as usize].kind {
            hir::ExprKind::ListGet { list, .. } => {
                matches!(
                    &self.f.body.exprs[list.0 as usize].kind,
                    hir::ExprKind::ReadLocal { .. }
                ) && self
                    .list_place(*list)
                    .ok()
                    .and_then(|(_, ty)| self.elem_ty_of(ty).ok())
                    .is_some_and(|e| !self.is_copy(e))
            }
            _ => false,
        }
    }

    fn variant_arm_of(
        &self,
        root: hir::ExprId,
        disc: u32,
    ) -> Result<(String, String, Ty), A2xReject> {
        let ty = self
            .view
            .resolve_use_ty(self.f.body.exprs[root.0 as usize].ty)?;
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
        let pty = arm.payload.ok_or_else(|| {
            A2xReject::unsupported_expr("unit-arm projection is outside the surface")
        })?;
        Ok((v.name.clone(), arm.name.clone(), pty))
    }
}

/// A C compound-literal borrowed view of a source string literal. Every
/// byte is emitted as `\xHH` (charset-independent; no adjacency hazard
/// because every byte is its own two-digit escape); never NUL-terminated
/// — every operation carries the explicit length. The cast matches the
/// struct member's pointer type exactly (a borrowed literal is
/// read-only; nothing ever writes through it).
fn c_str_lit(value: &str) -> String {
    let mut bytes = String::new();
    for b in value.bytes() {
        bytes.push_str(&format!("\\x{:02x}", b));
    }
    format!(
        "(a2x_str){{(uint8_t*)\"{bytes}\", {len}}}",
        bytes = bytes,
        len = value.len()
    )
}

fn is_lit(body: &hir::Body, id: hir::ExprId) -> bool {
    matches!(body.exprs[id.0 as usize].kind, hir::ExprKind::StrLit { .. })
}
