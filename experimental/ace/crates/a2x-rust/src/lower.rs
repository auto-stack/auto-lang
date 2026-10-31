//! Lower the shared semantic view to a typed Rust program (plan §2:
//! view → structured prepare → Rust provider).
//!
//! Correctness contracts this lowering is responsible for:
//!
//! * **Trap semantics** — every arithmetic op becomes an explicit checked
//!   helper returning `Result<_, AutoTrap>`; `?` propagates. No reliance on
//!   Rust overflow options (debug/release behave identically).
//! * **Source-order, single evaluation of call arguments** — `eval_args`
//!   (source order) are lowered first; every effectful (trap- or
//!   trace-capable) argument is pinned into a temporary `let` BEFORE the
//!   call, then placed per `bindings` (param ← arg). The target never
//!   re-guesses evaluation order from placement.
//! * **Short-circuit laziness** — `&&`/`||` lower to `if`-expressions whose
//!   RHS (including its own temporaries) lives inside the taken branch; an
//!   untaken RHS is never evaluated, temporaries and all.
//! * **Nearest-loop discipline** — Auto break/continue always target the
//!   innermost enclosing loop; the lowering asserts that on the explicit
//!   `LoopId` before emitting plain `break`/`continue`.
//! * **Hygiene** — locals render as `l{idx}_{name}`, functions as
//!   `fn_{name}` (free) / `mt_{Owner}_{name}` (methods), records as
//!   `Rec_{Name}` with `f_{field}` members, temporaries as `t{n}`; runtime
//!   helpers own the reserved `a2x_` prefix. User names can never hijack
//!   or collide.
//!
//! PLAN-902 (AX-B) additions, all behind the record/text/text-record
//! domains (the scalar/core paths keep byte-identical output):
//!
//! * **Nominal records** — `Rec_<Name>` structs, construction by the ctor
//!   binding map in source evaluation order, field-path reads/writes
//!   (value copies for Copy fields, no implicit clone of resource fields).
//! * **Param modes** — view/mut/move legalized per the frozen 881/774
//!   contract: view → shared borrow, mut → exclusive borrow (Rust enforces
//!   the caller-visible write-back and the same-root exclusivity), move →
//!   by-value transfer the callee cleans up.
//! * **Text resources** — owned `Str` maps to the runtime `A2xStr` with
//!   EXPLICIT cleanup exactly where the verifier's `BodyResourcePlan`
//!   schedules it (exit frees, replacement frees); plain Rust lexical Drop
//!   is never relied on. Temporary owned values free immediately after
//!   their single consumer (statement-anchored via pin draining).
//! * **Receiver discipline** — the receiver is `params[0]`; call sites
//!   evaluate it exactly once before explicit args (`eval_args` order).

use ace_a2x_core::{A2xReject, FnFacts, FnKind, RecordFacts, SemanticView, Ty};
use ace_core::hir;
use ace_core::text_resources::BodyResourcePlan;
use std::collections::HashMap;

/// Comparison operators (infallible; operands carry their own `?`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmpOp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl CmpOp {
    fn rust_str(self) -> &'static str {
        match self {
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
            CmpOp::Eq => "==",
            CmpOp::Ne => "!=",
        }
    }
}

/// One emitted Rust statement.
#[derive(Clone, Debug)]
pub enum RStmt {
    Let {
        name: String,
        ty: Ty,
        mutable: bool,
        init: RExpr,
    },
    Assign {
        name: String,
        value: RExpr,
    },
    /// Assign to a field path rooted at a local (PLAN-902): `l0_x.f_a = v`.
    AssignField {
        name: String,
        path: Vec<String>,
        value: RExpr,
    },
    /// Explicit free of an owned value at a plan-scheduled point
    /// (PLAN-902): `a2x_str_free(&mut l0_a);` — never lexical Drop.
    Free {
        name: String,
    },
    If {
        cond: RExpr,
        then: Vec<RStmt>,
        els: Option<Vec<RStmt>>,
    },
    Loop {
        body: Vec<RStmt>,
    },
    Break,
    Continue,
    Return {
        value: RExpr,
    },
}

/// Statements + tail value of a lazily-evaluated short-circuit side. Used
/// only inside [`RExpr::AndSC`]/[`RExpr::OrSC`] so the RHS temporaries stay
/// inside the taken branch.
#[derive(Clone, Debug)]
pub struct RBlock {
    pub stmts: Vec<RStmt>,
    pub value: RExpr,
}

/// One emitted Rust expression. Rendering (print.rs) appends `?` where a
/// fallible root is consumed as a value (`render_val`); nested fallible
/// operands are pinned into temporaries by the lowering, so call/checked
/// arguments are always side-effect-free here.
#[derive(Clone, Debug)]
pub enum RExpr {
    I32Lit(i32),
    BoolLit(bool),
    /// Borrowed UTF-8 literal (PLAN-902) — a `&'static str` view.
    StrLit(String),
    /// Unicode scalar literal (PLAN-902).
    CharLit(char),
    Local(String),
    /// Checked arithmetic helper call (`a2x_add`/`a2x_sub`/`a2x_mul`/
    /// `a2x_neg`/`a2x_div`/`a2x_rem`) — fallible.
    Checked {
        func: &'static str,
        args: Vec<RExpr>,
    },
    Cmp {
        op: CmpOp,
        lhs: Box<RExpr>,
        rhs: Box<RExpr>,
    },
    Not(Box<RExpr>),
    /// Lazy `lhs && { rhs-block }` (renders as an if-expression).
    AndSC {
        lhs: Box<RExpr>,
        rhs: Box<RBlock>,
    },
    /// Lazy `lhs || { rhs-block }`.
    OrSC {
        lhs: Box<RExpr>,
        rhs: Box<RBlock>,
    },
    /// Call of a lowered user function — fallible.
    Call {
        func: String,
        args: Vec<RExpr>,
    },
    /// Authorized trace intrinsic call — fallible, effectful.
    Trace {
        func: &'static str,
    },
    // ---- PLAN-902 record/text surface (never produced for scalar/core) ----
    /// Fallible typed text helper (`a2x_str_*`) — allocation/trap carrying.
    StrFn {
        func: &'static str,
        args: Vec<RExpr>,
    },
    /// Full-byte equality/inequality of two text views (pure, no
    /// normalization).
    StrCmp {
        eq: bool,
        lhs: Box<RExpr>,
        rhs: Box<RExpr>,
    },
    /// Borrowed view of a text operand (`x.as_str()` for owned values; a
    /// plain pass-through rendering for already-borrowed operands).
    AsStr {
        val: Box<RExpr>,
        owned: bool,
    },
    /// Record construction: nominal struct with placed field values.
    RecCtor {
        record: String,
        fields: Vec<(String, RExpr)>,
    },
    /// Field-path read through a base place (`base.f_a.f_b`).
    FieldPath {
        base: Box<RExpr>,
        path: Vec<String>,
    },
    /// Shared borrow `&place` (view args, readonly receivers).
    Ref {
        inner: Box<RExpr>,
    },
    /// Exclusive borrow `&mut place` (mut args).
    RefMut {
        inner: Box<RExpr>,
    },
    /// `char` code point as i32 (pure widening).
    CharToI32 {
        inner: Box<RExpr>,
    },
}

impl RExpr {
    /// True when this root returns `Result` and needs `?` at a value use.
    /// Trace marks return a plain i32 (frozen native contract) — they are
    /// EFFECTFUL but not fallible.
    pub fn is_fallible_root(&self) -> bool {
        matches!(
            self,
            RExpr::Checked { .. } | RExpr::Call { .. } | RExpr::StrFn { .. }
        )
    }

    /// True when the root must be pinned into a temporary before reuse
    /// (traps OR observable effects — single evaluation in source order).
    pub fn needs_pin(&self) -> bool {
        self.is_fallible_root() || matches!(self, RExpr::Trace { .. })
    }

    /// Pretty operator tag for diagnostics/dumps.
    pub fn tag(&self) -> &'static str {
        match self {
            RExpr::I32Lit(_) => "i32",
            RExpr::BoolLit(_) => "bool",
            RExpr::StrLit(_) => "str",
            RExpr::CharLit(_) => "char",
            RExpr::Local(_) => "local",
            RExpr::Checked { func, .. } => func,
            RExpr::Cmp { .. } => "cmp",
            RExpr::Not(_) => "not",
            RExpr::AndSC { .. } => "and",
            RExpr::OrSC { .. } => "or",
            RExpr::Call { .. } => "call",
            RExpr::Trace { func } => func,
            RExpr::StrFn { func, .. } => func,
            RExpr::StrCmp { .. } => "str-cmp",
            RExpr::AsStr { .. } => "as-str",
            RExpr::RecCtor { .. } => "ctor",
            RExpr::FieldPath { .. } => "field",
            RExpr::Ref { .. } => "ref",
            RExpr::RefMut { .. } => "ref-mut",
            RExpr::CharToI32 { .. } => "char-to-i32",
        }
    }
}

/// One lowered parameter (PLAN-902 adds the verified ownership mode; the
/// name carries Rust `mut` when the local is an assignment target).
#[derive(Clone, Debug)]
pub struct RustParam {
    pub name: String,
    pub ty: Ty,
    pub mode: hir::ParamMode,
}

/// One nominal record rendered into `generated.rs`.
#[derive(Clone, Debug)]
pub struct RustRecord {
    /// Module type-table index (name resolution for `Ty::Record`).
    pub type_index: u32,
    /// Sanitized Rust struct name (`Rec_Counter`).
    pub name: String,
    /// Sanitized field names in declaration order with their types.
    pub fields: Vec<(String, Ty)>,
    /// Resource records (transitive owned `Str`) derive nothing — clone
    /// and cleanup are explicit; Copy records derive `Clone, Copy`.
    pub resource: bool,
}

/// One lowered user function. `origin` is the def's byte span in the
/// compiled source (source map v1 — printed as a comment and carried in the
/// manifest so target diagnostics map back to the original `.at`).
#[derive(Clone, Debug)]
pub struct RustFn {
    pub name: String,
    pub params: Vec<RustParam>,
    pub result: Ty,
    pub origin: (u32, u32),
    pub body: Vec<RStmt>,
}

/// The whole lowered program (all module functions + entry name).
#[derive(Clone, Debug)]
pub struct RustProgram {
    pub functions: Vec<RustFn>,
    /// Rust name of the entry function.
    pub entry: String,
    /// Distinct trace symbols used (runtime emission input).
    pub marks: Vec<&'static str>,
    /// Nominal record declarations (PLAN-902; empty for scalar/core).
    pub records: Vec<RustRecord>,
    /// Whether any signature needs the elided-all `'a` lifetime
    /// (PLAN-902: any reference parameter or borrowed result).
    pub borrowed: bool,
}

/// Sanitize a source name into Rust-identifier-safe ASCII ([A-Za-z0-9_]).
fn sanitize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    out
}

/// Hygienic Rust name of a module function — derived from the SOURCE name
/// only. A module's source names are unique (frontend rule) and STABLE
/// across whole-module and single-body bundles, unlike def-table indices
/// which shift when a fragment bundle carries scaffolds in another order
/// (F-828-R1). The `fn_` prefix can never collide with Rust keywords or
/// the runtime's reserved `a2x_` helpers.
pub fn rust_fn_name(name: &str) -> String {
    let s = sanitize(name);
    if s.is_empty() {
        "fn_".to_string()
    } else {
        format!("fn_{}", s)
    }
}

/// Hygienic Rust name of a type-bound method (PLAN-881/902): methods are
/// owner-qualified (`sm_<Owner>_<name>` → `mt_<Owner>_<name>`), never
/// colliding with free functions (`fn_*`) or the `a2x_` runtime prefix.
pub fn rust_method_name(owner_qualified: &str) -> String {
    let s = sanitize(owner_qualified);
    format!("mt_{}", s)
}

/// Rust struct name of a nominal record type: upper-camel, `Rec`-prefixed
/// (hygienic against keywords and runtime names, warning-free).
pub fn rust_record_name(name: &str) -> String {
    let s = sanitize(name);
    let mut cs = s.chars();
    match cs.next() {
        Some(c) => {
            let first = c.to_ascii_uppercase();
            format!("Rec{}{}", first, cs.as_str())
        }
        None => "Rec".to_string(),
    }
}

/// Lower a whole module for the given explicit entry function name.
pub fn lower_functions<'a>(view: &SemanticView<'a>) -> Result<RustProgram, A2xReject> {
    let mut functions = Vec::new();
    for f in view.functions() {
        functions.push(lower_fn(view, f)?);
    }
    let mut marks: Vec<&'static str> = view.trace_symbols().into_iter().collect();
    marks.sort_unstable();
    let records = lower_records(view)?;
    let borrowed = functions
        .iter()
        .any(|f| f.params.iter().any(|p| borrows(p.ty, p.mode)) || borrows_result(f.result));
    Ok(RustProgram {
        functions,
        entry: String::new(),
        marks,
        records,
        borrowed,
    })
}

/// Lower a whole module and select its entry by explicit source name.
pub fn lower_program<'a>(
    view: &SemanticView<'a>,
    entry_name: &str,
) -> Result<RustProgram, A2xReject> {
    let entry = view.entry(entry_name)?;
    let mut p = lower_functions(view)?;
    p.entry = rust_fn_name(entry.source_name);
    Ok(p)
}

/// Whether a param of this type+mode passes by reference.
fn borrows(ty: Ty, mode: hir::ParamMode) -> bool {
    match mode {
        hir::ParamMode::MutBorrow => true,
        hir::ParamMode::MoveOwn => false,
        hir::ParamMode::View => matches!(ty, Ty::Str | Ty::StrOwned | Ty::Record(_)),
    }
}

/// Whether a RESULT is borrowed (str results borrow from view params).
fn borrows_result(ty: Ty) -> bool {
    matches!(ty, Ty::Str)
}

fn lower_records(view: &SemanticView<'_>) -> Result<Vec<RustRecord>, A2xReject> {
    let mut out = Vec::new();
    for rec in view.records() {
        let fields = rec
            .fields
            .iter()
            .map(|f| (format!("f_{}", sanitize(&f.name)), f.ty))
            .collect();
        out.push(RustRecord {
            type_index: rec.type_index,
            name: rust_record_name(&rec.name),
            fields,
            resource: rec.resource,
        });
    }
    Ok(out)
}

fn lower_fn<'a>(view: &SemanticView<'a>, f: &FnFacts<'a>) -> Result<RustFn, A2xReject> {
    let mut lx = FnLowering::new(view, f)?;
    let body = lx.lower_block(f.body.entry)?;
    let params = lx.rust_params();
    Ok(RustFn {
        name: rust_name_of(f),
        params,
        result: f.result,
        origin: f.origin,
        body,
    })
}

/// The Rust function name of a def: free functions keep the 892 rule;
/// methods are owner-qualified (the FnFacts source name of a method is
/// already `<Owner>_<name>`).
fn rust_name_of(f: &FnFacts<'_>) -> String {
    match f.kind {
        FnKind::Free => rust_fn_name(f.source_name),
        FnKind::Method { .. } => rust_method_name(f.source_name),
    }
}

struct FnLowering<'v, 'a> {
    view: &'v SemanticView<'a>,
    facts: &'v FnFacts<'a>,
    /// LocalId index → hygienic Rust local name.
    local_names: HashMap<u32, String>,
    /// LocalId index → target-neutral type.
    local_tys: HashMap<u32, Ty>,
    /// LocalId index → param mode (params only).
    local_modes: HashMap<u32, hir::ParamMode>,
    /// Param LocalId indices in signature order.
    param_locals: Vec<u32>,
    /// LocalId indices that need Rust `mut` bindings: whole-local
    /// reassignment targets plus explicitly freed owned locals (PLAN-902).
    /// Field-path writes go through the root's borrow and never rebind.
    assigned: std::collections::BTreeSet<u32>,
    /// The verifier's resource plan for THIS body (text/text-record
    /// domains); cleanup is emitted from it verbatim.
    plan: Option<&'v BodyResourcePlan>,
    /// View-consumed pinned owned temporaries pending a free (drained at
    /// the next pin or the end of the current statement).
    pending_frees: Vec<String>,
    /// Current block id + statement index (plan exit keys).
    cur_block: u32,
    cur_stmt: usize,
    temp_counter: usize,
    /// LoopId stack of enclosing loops (innermost last).
    loop_stack: Vec<u32>,
    /// Current expression/statement nesting depth (budget guard).
    depth: usize,
}

impl<'v, 'a> FnLowering<'v, 'a> {
    fn new(view: &'v SemanticView<'a>, facts: &'v FnFacts<'a>) -> Result<Self, A2xReject> {
        let mut local_names = HashMap::new();
        let mut local_tys = HashMap::new();
        let mut local_modes = HashMap::new();
        for (idx, local) in facts.body.locals.iter().enumerate() {
            let s = sanitize(&local.name);
            let name = if s.is_empty() {
                format!("l{}", idx)
            } else {
                format!("l{}_{}", idx, s)
            };
            local_names.insert(idx as u32, name);
            local_tys.insert(idx as u32, view.resolve_use_ty(local.ty)?);
            if matches!(local.role, hir::LocalRole::Param) {
                local_modes.insert(idx as u32, local.param_mode);
            }
        }
        let mut param_locals: Vec<(u32, u32)> = facts
            .body
            .locals
            .iter()
            .enumerate()
            .filter(|(_, l)| matches!(l.role, hir::LocalRole::Param))
            .map(|(i, l)| (l.index.unwrap_or(u32::MAX), i as u32))
            .collect();
        param_locals.sort();
        let param_locals: Vec<u32> = param_locals.into_iter().map(|(_, i)| i).collect();
        if param_locals.len() != facts.params.len() {
            return Err(A2xReject::new(
                "a2x.internal",
                "param locals disagree with collected params",
            ));
        }

        // Rust `mut` bindings: whole-local assignment targets (params and
        // vars alike), every local the resource plan FREES (an owned
        // buffer freed by `a2x_str_free(&mut ..)` must be rebindable as
        // mutable) and every exclusively-borrowed local (`&mut l_x` call
        // arguments). Field-path writes borrow the root — never rebind it.
        let mut assigned = std::collections::BTreeSet::new();
        collect_assignees(facts, &mut assigned);
        for e in &facts.body.exprs {
            if let hir::ExprKind::BorrowMut { local } = &e.kind {
                assigned.insert(local.0);
            }
        }
        if let Some(bp) = view.body_plan(facts) {
            for frees in bp.exit_frees.values() {
                assigned.extend(frees.iter().copied());
            }
            for bufs in bp.replace_frees.values() {
                assigned.extend(bufs.iter().map(|b| b.root));
            }
        }

        // PLAN-902: reject the surface this backend does not legalize yet
        // (named, never silent): mut text params and resource-record
        // cleanup belong to the T-04 widening.
        for p in &facts.params {
            match (p.mode, p.ty) {
                (hir::ParamMode::MutBorrow, Ty::Str | Ty::StrOwned) => {
                    return Err(A2xReject::new(
                        "a2x.param-mode",
                        format!(
                            "in `{}`: `mut` text parameters are not yet in this backend's frozen surface (T-04)",
                            facts.source_name
                        ),
                    ));
                }
                (hir::ParamMode::MutBorrow, Ty::Record(_))
                    if record_of(view, p.ty).map(|r| r.resource).unwrap_or(false) =>
                {
                    return Err(A2xReject::new(
                        "a2x.param-mode",
                        format!(
                            "in `{}`: `mut` resource-record parameters are not yet in this backend's frozen surface (T-04)",
                            facts.source_name
                        ),
                    ));
                }
                _ => {}
            }
        }
        if let Some(rec) = record_of(view, facts.result) {
            if rec.resource {
                return Err(A2xReject::new(
                    "a2x.unsupported-type",
                    format!(
                        "in `{}`: resource-record results are not yet in this backend's frozen surface (T-04)",
                        facts.source_name
                    ),
                ));
            }
        }

        let plan = view.body_plan(facts);
        if let Some(bp) = plan {
            // T-02: only plain owned-Str roots are cleaned here; resource
            // record buffers (non-empty paths) land with T-04.
            for bufs in bp.buffer_frees.values() {
                for b in bufs {
                    if !b.path.is_empty() {
                        return Err(A2xReject::new(
                            "a2x.unsupported-expr",
                            format!(
                                "in `{}`: resource-record field cleanup is not yet in this backend's frozen surface (T-04)",
                                facts.source_name
                            ),
                        ));
                    }
                }
            }
            for bufs in bp.replace_frees.values() {
                for b in bufs {
                    if !b.path.is_empty() {
                        return Err(A2xReject::new(
                            "a2x.unsupported-expr",
                            format!(
                                "in `{}`: resource-record field replacement is not yet in this backend's frozen surface (T-04)",
                                facts.source_name
                            ),
                        ));
                    }
                }
            }
            // resource-record owned roots (whole records owning Str fields)
            for l in &bp.owned {
                if let Some(ty) = local_tys.get(l) {
                    if let Ty::Record(id) = ty {
                        if view.record(*id).map(|r| r.resource).unwrap_or(false) {
                            return Err(A2xReject::new(
                                "a2x.unsupported-expr",
                                format!(
                                    "in `{}`: resource-record owned roots are not yet in this backend's frozen surface (T-04)",
                                    facts.source_name
                                ),
                            ));
                        }
                    }
                }
            }
        }

        Ok(FnLowering {
            view,
            facts,
            local_names,
            local_tys,
            local_modes,
            param_locals,
            assigned,
            plan,
            pending_frees: Vec::new(),
            cur_block: 0,
            cur_stmt: 0,
            temp_counter: 0,
            loop_stack: Vec::new(),
            depth: 0,
        })
    }

    fn rust_params(&self) -> Vec<RustParam> {
        self.param_locals
            .iter()
            .map(|&id| {
                let mut name = self.local_names[&id].clone();
                if self.assigned.contains(&id) {
                    name = format!("mut {}", name);
                }
                RustParam {
                    name,
                    ty: self.local_tys[&id],
                    mode: *self.local_modes.get(&id).unwrap_or(&hir::ParamMode::View),
                }
            })
            .collect()
    }

    fn fresh_temp(&mut self) -> String {
        let t = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        t
    }

    /// Emit the pending view-consumer frees now (the immediate-after-
    /// consumer anchor: called right before a NEW fallible pin so earlier
    /// temps free before the next allocation point).
    fn drain_frees(&mut self, out: &mut Vec<RStmt>) {
        for name in std::mem::take(&mut self.pending_frees) {
            out.push(RStmt::Free { name });
        }
    }

    fn lower_block(&mut self, block_id: hir::BlockId) -> Result<Vec<RStmt>, A2xReject> {
        let block = self
            .facts
            .body
            .blocks
            .get(block_id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "block index out of range"))?;
        let mut out = Vec::new();
        let prev_block = self.cur_block;
        self.cur_block = block_id.0;
        for (idx, stmt) in block.stmts.iter().enumerate() {
            self.cur_stmt = idx;
            self.lower_stmt(stmt, &mut out)?;
        }
        // the block's natural fall-through end: frees only its own lets,
        // and only when the block actually falls through (a trailing
        // terminator statement carries its own exit key).
        let terminated = matches!(
            block.stmts.last(),
            Some(hir::Stmt::Return { .. }) | Some(hir::Stmt::Break { .. }) | Some(hir::Stmt::Continue { .. })
        );
        if !terminated {
            if let Some(bp) = self.plan {
                if let Some(frees) = bp.exit_frees.get(&(self.cur_block, None)) {
                    self.emit_local_frees(frees, &mut out);
                }
            }
            self.drain_frees(&mut out);
        }
        self.cur_block = prev_block;
        Ok(out)
    }

    /// `a2x_str_free` for plain owned-Str locals (plan order verbatim).
    fn emit_local_frees(&self, frees: &[u32], out: &mut Vec<RStmt>) {
        for l in frees {
            if let Some(name) = self.local_names.get(l) {
                out.push(RStmt::Free { name: name.clone() });
            }
        }
    }

    fn lower_stmt(&mut self, s: &hir::Stmt, out: &mut Vec<RStmt>) -> Result<(), A2xReject> {
        match s {
            hir::Stmt::Let { local, value } => {
                let (v, _) = self.lower_move_value(*value, out)?;
                let name = self.local_name(local.0)?;
                let ty = self.local_ty(local.0)?;
                let mutable = self.assigned.contains(&local.0);
                self.drain_frees(out);
                out.push(RStmt::Let {
                    name,
                    ty,
                    mutable,
                    init: v,
                });
            }
            hir::Stmt::Return { value } => {
                let (v, vty) = self.lower_move_value(*value, out)?;
                // plan-scheduled exit frees: the return value must be
                // materialized (pinned when it is not a bare local read)
                // BEFORE any freed local dies — the frees render as
                // statements preceding the return.
                let mut value = v;
                let frees = self
                    .plan
                    .and_then(|bp| bp.exit_frees.get(&(self.cur_block, Some(self.cur_stmt))))
                    .cloned()
                    .unwrap_or_default();
                if !frees.is_empty() {
                    let bare = matches!(value, RExpr::Local(_));
                    if !bare {
                        let t = self.fresh_temp();
                        out.push(RStmt::Let {
                            name: t.clone(),
                            ty: vty,
                            mutable: false,
                            init: value.clone(),
                        });
                        value = RExpr::Local(t);
                    }
                    self.emit_local_frees(&frees, out);
                }
                self.drain_frees(out);
                out.push(RStmt::Return { value });
            }
            hir::Stmt::If { cond, then, els } => {
                let (c, _) = self.lower_expr(*cond, out)?;
                let t = self.lower_block(*then)?;
                let e = match els {
                    Some(b) => Some(self.lower_block(*b)?),
                    None => None,
                };
                self.drain_frees(out);
                out.push(RStmt::If {
                    cond: c,
                    then: t,
                    els: e,
                });
            }
            hir::Stmt::Loop { id, body } => {
                self.loop_stack.push(id.0);
                let b = self.lower_block(*body)?;
                self.loop_stack.pop();
                out.push(RStmt::Loop { body: b });
            }
            hir::Stmt::Assign { place, value } => {
                let place = self
                    .facts
                    .body
                    .places
                    .get(place.0 as usize)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "place index out of range"))?;
                let (v, _) = self.lower_move_value(*value, out)?;
                // replacement frees: the OLD buffer dies only after the
                // RHS fully materialized (894 §5.1).
                let replaces = self
                    .plan
                    .and_then(|bp| bp.replace_frees.get(&(self.cur_block, self.cur_stmt)))
                    .map(|bufs| bufs.iter().map(|b| b.root).collect::<Vec<_>>())
                    .unwrap_or_default();
                if !replaces.is_empty() {
                    self.emit_local_frees(&replaces, out);
                }
                let name = self.local_name(place.local.0)?;
                match &place.kind {
                    hir::PlaceKind::Local => {
                        self.drain_frees(out);
                        out.push(RStmt::Assign { name, value: v });
                    }
                    hir::PlaceKind::FieldPath(path) => {
                        let path = self.resolve_field_names(place.local.0, path)?;
                        self.drain_frees(out);
                        out.push(RStmt::AssignField { name, path, value: v });
                    }
                }
            }
            hir::Stmt::Break { target } => {
                self.assert_nearest(target.0, "break")?;
                if let Some(bp) = self.plan {
                    if let Some(frees) = bp.exit_frees.get(&(self.cur_block, Some(self.cur_stmt))) {
                        let frees = frees.clone();
                        self.emit_local_frees(&frees, out);
                    }
                }
                self.drain_frees(out);
                out.push(RStmt::Break);
            }
            hir::Stmt::Continue { target } => {
                self.assert_nearest(target.0, "continue")?;
                if let Some(bp) = self.plan {
                    if let Some(frees) = bp.exit_frees.get(&(self.cur_block, Some(self.cur_stmt))) {
                        let frees = frees.clone();
                        self.emit_local_frees(&frees, out);
                    }
                }
                self.drain_frees(out);
                out.push(RStmt::Continue);
            }
            other => {
                return Err(A2xReject::unsupported_stmt(format!(
                    "in `{}`: statement kind `{:?}` is outside the compute surface",
                    self.facts.name, other
                )))
            }
        }
        Ok(())
    }

    /// Resolve a field path (declaration indices) into Rust field names by
    /// walking the record table from the root local's type.
    fn resolve_field_names(
        &self,
        root: u32,
        path: &[hir::FieldId],
    ) -> Result<Vec<String>, A2xReject> {
        let mut names = Vec::with_capacity(path.len());
        let mut cur = self.local_ty(root)?;
        for fid in path {
            let Ty::Record(tid) = cur else {
                return Err(A2xReject::new(
                    "a2x.internal",
                    "field path traverses a non-record type",
                ));
            };
            let rec = self
                .view
                .record(tid)
                .ok_or_else(|| A2xReject::new("a2x.internal", "unknown record type"))?;
            let fld = rec.fields.get(fid.0 as usize).ok_or_else(|| {
                A2xReject::new("a2x.internal", "field index out of range")
            })?;
            names.push(format!("f_{}", sanitize(&fld.name)));
            cur = fld.ty;
        }
        Ok(names)
    }

    /// Auto break/continue always target the innermost enclosing loop; a
    /// non-nearest target would mistranslate to Rust's plain break/continue.
    fn assert_nearest(&self, target: u32, what: &str) -> Result<(), A2xReject> {
        match self.loop_stack.last() {
            Some(inner) if *inner == target => Ok(()),
            other => Err(A2xReject::new(
                "a2x.internal",
                format!(
                    "`{}` targets loop #{} but the innermost enclosing loop is {:?}",
                    what, target, other
                ),
            )),
        }
    }

    fn local_name(&self, id: u32) -> Result<String, A2xReject> {
        self.local_names
            .get(&id)
            .cloned()
            .ok_or_else(|| A2xReject::new("a2x.internal", format!("unknown local #{}", id)))
    }

    fn local_ty(&self, id: u32) -> Result<Ty, A2xReject> {
        self.local_tys
            .get(&id)
            .copied()
            .ok_or_else(|| A2xReject::new("a2x.internal", format!("unknown local #{}", id)))
    }

    /// Borrowed-view form of a lowered text operand: owned values render
    /// `.as_str()`, borrowed values pass through.
    fn as_str_form(&self, v: RExpr, ty: Ty) -> RExpr {
        match ty {
            Ty::StrOwned => RExpr::AsStr {
                val: Box::new(v),
                owned: true,
            },
            _ => v,
        }
    }

    /// Lower one expression, appending any pinning preludes to `out`.
    /// Returns the value expression and its type. Effectful subexpressions
    /// (fallible roots) are pinned into temporaries at their source-order
    /// position — EXCEPT inside a short-circuit RHS, whose preludes land in
    /// that side's own lazy block.
    fn lower_expr(
        &mut self,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<(RExpr, Ty), A2xReject> {
        self.depth += 1;
        if self.depth > ace_a2x_core::MAX_STRUCTURE_DEPTH {
            self.depth -= 1;
            return Err(A2xReject::new(
                "a2x.too-deep",
                format!(
                    "function `{}` lowers deeper than the frozen budget {}",
                    self.facts.source_name,
                    ace_a2x_core::MAX_STRUCTURE_DEPTH
                ),
            ));
        }
        let r = self.lower_expr_inner(id, out);
        self.depth -= 1;
        r
    }

    fn lower_expr_inner(
        &mut self,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<(RExpr, Ty), A2xReject> {
        let e = self
            .facts
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
        let ty = self.view.resolve_use_ty(e.ty)?;
        match &e.kind {
            hir::ExprKind::ReadLocal { local } => Ok((
                RExpr::Local(self.local_name(local.0)?),
                self.local_ty(local.0)?,
            )),
            hir::ExprKind::Constant { value } => Ok((RExpr::I32Lit(*value), Ty::I32)),
            hir::ExprKind::StrLit { value } => match ty {
                // owned promotion context (AP-4): an allocation
                Ty::StrOwned => Ok((
                    RExpr::StrFn {
                        func: "a2x_str_from_lit",
                        args: vec![RExpr::StrLit(value.clone())],
                    },
                    Ty::StrOwned,
                )),
                Ty::Str => Ok((RExpr::StrLit(value.clone()), Ty::Str)),
                other => Err(A2xReject::unsupported_type(format!(
                    "string literal typed `{}`",
                    other.as_str()
                ))),
            },
            hir::ExprKind::CharLit { scalar } => {
                let c = char::from_u32(*scalar).ok_or_else(|| {
                    A2xReject::new("a2x.internal", "invalid scalar in checked CharLit")
                })?;
                Ok((RExpr::CharLit(c), Ty::Char))
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let lt = self.view.resolve_use_ty(
                    self.facts.body.exprs[lhs.0 as usize].ty,
                )?;
                let rt = self.view.resolve_use_ty(
                    self.facts.body.exprs[rhs.0 as usize].ty,
                )?;
                // text equality: full bytes, no normalization (pure)
                if matches!(op, hir::BinOp::Eq | hir::BinOp::Ne) && lt.is_text() {
                    let (l, _) = self.lower_view_value(*lhs, out)?;
                    let (r, _) = self.lower_view_value(*rhs, out)?;
                    let l = self.as_str_form(l, lt);
                    let r = self.as_str_form(r, rt);
                    return Ok((
                        RExpr::StrCmp {
                            eq: matches!(op, hir::BinOp::Eq),
                            lhs: Box::new(l),
                            rhs: Box::new(r),
                        },
                        Ty::Bool,
                    ));
                }
                let (l, _) = self.lower_pure(*lhs, out)?;
                let (r, _) = self.lower_pure(*rhs, out)?;
                match op {
                    hir::BinOp::AddI32 => Ok((arith("a2x_add", l, r), Ty::I32)),
                    hir::BinOp::SubI32 => Ok((arith("a2x_sub", l, r), Ty::I32)),
                    hir::BinOp::MulI32 => Ok((arith("a2x_mul", l, r), Ty::I32)),
                    hir::BinOp::DivI32 => Ok((arith("a2x_div", l, r), Ty::I32)),
                    hir::BinOp::RemI32 => Ok((arith("a2x_rem", l, r), Ty::I32)),
                    hir::BinOp::LtI32 => Ok((cmp(CmpOp::Lt, l, r), Ty::Bool)),
                    hir::BinOp::LeI32 => Ok((cmp(CmpOp::Le, l, r), Ty::Bool)),
                    hir::BinOp::GtI32 => Ok((cmp(CmpOp::Gt, l, r), Ty::Bool)),
                    hir::BinOp::GeI32 => Ok((cmp(CmpOp::Ge, l, r), Ty::Bool)),
                    hir::BinOp::Eq => Ok((cmp(CmpOp::Eq, l, r), Ty::Bool)),
                    hir::BinOp::Ne => Ok((cmp(CmpOp::Ne, l, r), Ty::Bool)),
                }
            }
            hir::ExprKind::Unary { op, operand } => {
                let (o, _) = self.lower_pure(*operand, out)?;
                match op {
                    hir::UnaryOp::NegI32 => Ok((
                        RExpr::Checked {
                            func: "a2x_neg",
                            args: vec![o],
                        },
                        Ty::I32,
                    )),
                    hir::UnaryOp::NotBool => Ok((RExpr::Not(Box::new(o)), Ty::Bool)),
                }
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                // LHS always evaluates: its preludes go to the outer stream.
                let (l, _) = self.lower_pure(*lhs, out)?;
                // RHS is lazy: its preludes live inside the side block.
                let mut side = Vec::new();
                let (r, _) = self.lower_pure(*rhs, &mut side)?;
                // drain the side's own view-consumer temps into it
                for name in std::mem::take(&mut self.pending_frees) {
                    side.push(RStmt::Free { name });
                }
                let block = RBlock {
                    stmts: side,
                    value: r,
                };
                match op {
                    hir::LogicOp::And => Ok((
                        RExpr::AndSC {
                            lhs: Box::new(l),
                            rhs: Box::new(block),
                        },
                        Ty::Bool,
                    )),
                    hir::LogicOp::Or => Ok((
                        RExpr::OrSC {
                            lhs: Box::new(l),
                            rhs: Box::new(block),
                        },
                        Ty::Bool,
                    )),
                }
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                let target_id = callee.0 as usize;
                if let Some(sym) = self.view.trace_symbol(target_id) {
                    if !eval_args.is_empty() {
                        return Err(A2xReject::new(
                            "a2x.internal",
                            "trace intrinsic call carries arguments",
                        ));
                    }
                    let func = match sym {
                        "hir.test.mark_a" => "a2x_mark_a",
                        _ => "a2x_mark_b",
                    };
                    return Ok((RExpr::Trace { func }, Ty::I32));
                }
                // Internal function/method or fragment external.
                let (rust_name, target_params, result_ty) =
                    if let Some(t) = self.view.fn_by_def(target_id) {
                        (rust_name_of(t), t.params.clone(), t.result)
                    } else if let Some(x) = self.view.external_by_def(target_id) {
                        (
                            match x.kind {
                                FnKind::Free => rust_fn_name(&x.source_name),
                                FnKind::Method { .. } => rust_method_name(&x.source_name),
                            },
                            x.params
                                .iter()
                                .map(|ty| ace_a2x_core::ParamFacts {
                                    name: String::new(),
                                    ty: *ty,
                                    mode: hir::ParamMode::View,
                                })
                                .collect(),
                            x.result,
                        )
                    } else {
                        return Err(A2xReject::unsupported_expr(format!(
                            "call target def #{} is not an admitted function",
                            target_id
                        )));
                    };
                // Source order first: EVERY argument pins at its
                // source-order position (single evaluation, effects before
                // placement) — owning args pin as owning values, view args
                // of owned text as view-consumed temporaries.
                let n = target_params.len();
                let mode_of_arg = |arg_idx: u32| -> hir::ParamMode {
                    bindings
                        .iter()
                        .find(|b| b.arg == arg_idx)
                        .and_then(|b| target_params.get(b.param as usize))
                        .map(|p| p.mode)
                        .unwrap_or(hir::ParamMode::View)
                };
                let mut slots: Vec<RExpr> = Vec::with_capacity(eval_args.len());
                for (i, &arg) in eval_args.iter().enumerate() {
                    let mode = mode_of_arg(i as u32);
                    let pty = bindings
                        .iter()
                        .find(|b| b.arg == i as u32)
                        .and_then(|b| target_params.get(b.param as usize))
                        .map(|p| p.ty)
                        .unwrap_or(Ty::I32);
                    let view_owned = matches!(mode, hir::ParamMode::View)
                        && matches!(pty, Ty::StrOwned);
                    let (v, _) = if view_owned {
                        self.lower_view_value(arg, out)?
                    } else {
                        self.lower_pure(arg, out)?
                    };
                    slots.push(v);
                }
                // Placement by the binding map, in param order, legalized
                // per the target param's mode.
                let mut placed: Vec<Option<RExpr>> = vec![None; n];
                for b in bindings {
                    let arg = slots.get(b.arg as usize).cloned().ok_or_else(|| {
                        A2xReject::new("a2x.internal", "binding arg index out of range")
                    })?;
                    let p = b.param as usize;
                    if p >= n {
                        return Err(A2xReject::new(
                            "a2x.internal",
                            "binding param index out of range",
                        ));
                    }
                    if placed[p].is_some() {
                        return Err(A2xReject::new("a2x.internal", "duplicate param binding"));
                    }
                    placed[p] = Some(arg);
                }
                let mut args = Vec::with_capacity(n);
                for (i, p) in placed.into_iter().enumerate() {
                    let raw = p.ok_or_else(|| {
                        A2xReject::new(
                            "a2x.internal",
                            format!(
                                "param #{} of call target def #{} left unbound",
                                i, target_id
                            ),
                        )
                    })?;
                    let mode = target_params[i].mode;
                    let pty = target_params[i].ty;
                    args.push(self.legalize_arg(raw, mode, pty)?);
                }
                Ok((
                    RExpr::Call {
                        func: rust_name,
                        args,
                    },
                    result_ty,
                ))
            }
            // ---- PLAN-902 text operations ----
            hir::ExprKind::TextLen { operand } => {
                let (v, vt) = self.lower_view_value(*operand, out)?;
                Ok((
                    RExpr::StrFn {
                        func: "a2x_str_len",
                        args: vec![self.as_str_form(v, vt)],
                    },
                    Ty::I32,
                ))
            }
            hir::ExprKind::TextLenBytes { operand } => {
                let (v, vt) = self.lower_view_value(*operand, out)?;
                Ok((
                    RExpr::StrFn {
                        func: "a2x_str_len_bytes",
                        args: vec![self.as_str_form(v, vt)],
                    },
                    Ty::I32,
                ))
            }
            hir::ExprKind::TextCharAt { operand, index } => {
                let (v, vt) = self.lower_view_value(*operand, out)?;
                let (i, _) = self.lower_pure(*index, out)?;
                Ok((
                    RExpr::StrFn {
                        func: "a2x_str_char_at",
                        args: vec![self.as_str_form(v, vt), i],
                    },
                    Ty::Char,
                ))
            }
            hir::ExprKind::TextSlice {
                operand,
                start,
                end,
            } => {
                let (v, vt) = self.lower_view_value(*operand, out)?;
                let (s, _) = self.lower_pure(*start, out)?;
                let (e, _) = self.lower_pure(*end, out)?;
                Ok((
                    RExpr::StrFn {
                        func: "a2x_str_slice",
                        args: vec![self.as_str_form(v, vt), s, e],
                    },
                    Ty::Str,
                ))
            }
            hir::ExprKind::TextConcat { lhs, rhs } => {
                let (l, lt) = self.lower_view_value(*lhs, out)?;
                let (r, rt) = self.lower_view_value(*rhs, out)?;
                Ok((
                    RExpr::StrFn {
                        func: "a2x_str_concat",
                        args: vec![self.as_str_form(l, lt), self.as_str_form(r, rt)],
                    },
                    Ty::StrOwned,
                ))
            }
            hir::ExprKind::TextClone { operand } => {
                let (v, vt) = self.lower_view_value(*operand, out)?;
                Ok((
                    RExpr::StrFn {
                        func: "a2x_str_clone",
                        args: vec![self.as_str_form(v, vt)],
                    },
                    Ty::StrOwned,
                ))
            }
            hir::ExprKind::TextToOwned { operand } => {
                let (v, vt) = self.lower_view_value(*operand, out)?;
                Ok((
                    RExpr::StrFn {
                        func: "a2x_str_to_owned",
                        args: vec![self.as_str_form(v, vt)],
                    },
                    Ty::StrOwned,
                ))
            }
            hir::ExprKind::TextToBorrowed { operand } => {
                let (v, vt) = self.lower_view_value(*operand, out)?;
                Ok((self.as_str_form(v, vt), Ty::Str))
            }
            hir::ExprKind::CharToCodepoint { operand } => {
                let (v, _) = self.lower_pure(*operand, out)?;
                Ok((
                    RExpr::CharToI32 { inner: Box::new(v) },
                    Ty::I32,
                ))
            }
            hir::ExprKind::CharFromCodepoint { operand } => {
                let (v, _) = self.lower_pure(*operand, out)?;
                Ok((
                    RExpr::StrFn {
                        func: "a2x_char_from_codepoint",
                        args: vec![v],
                    },
                    Ty::Char,
                ))
            }
            // ---- PLAN-902 record operations ----
            hir::ExprKind::RecordCtor {
                record,
                eval_args,
                bindings,
            } => {
                let rec = self.view.record(record.0).ok_or_else(|| {
                    A2xReject::new("a2x.internal", "unknown record type in RecordCtor")
                })?;
                // source-order evaluation (pins), then placement
                let mut slots: Vec<RExpr> = Vec::with_capacity(eval_args.len());
                for &a in eval_args {
                    let (v, _) = self.lower_pure(a, out)?;
                    slots.push(v);
                }
                let mut fields: Vec<Option<(String, RExpr)>> =
                    vec![None; rec.fields.len()];
                for b in bindings {
                    let v = slots.get(b.arg as usize).cloned().ok_or_else(|| {
                        A2xReject::new("a2x.internal", "ctor arg index out of range")
                    })?;
                    let f = b.field as usize;
                    if f >= rec.fields.len() {
                        return Err(A2xReject::new("a2x.internal", "ctor field out of range"));
                    }
                    if fields[f].is_some() {
                        return Err(A2xReject::new("a2x.internal", "duplicate ctor binding"));
                    }
                    fields[f] = Some((format!("f_{}", sanitize(&rec.fields[f].name)), v));
                }
                let mut placed = Vec::with_capacity(fields.len());
                for (i, f) in fields.into_iter().enumerate() {
                    placed.push(f.ok_or_else(|| {
                        A2xReject::new(
                            "a2x.internal",
                            format!("ctor field #{} left unbound", i),
                        )
                    })?);
                }
                Ok((
                    RExpr::RecCtor {
                        record: rust_record_name(&rec.name),
                        fields: placed,
                    },
                    ty,
                ))
            }
            hir::ExprKind::FieldGet { base, path } => {
                let (b, _) = self.lower_pure(*base, out)?;
                let root_ty = self.view.resolve_use_ty(
                    self.facts.body.exprs[base.0 as usize].ty,
                )?;
                let names = self.field_names_of(root_ty, path)?;
                Ok((RExpr::FieldPath { base: Box::new(b), path: names }, ty))
            }
            hir::ExprKind::BorrowView { base } => {
                let (v, _) = self.lower_pure(*base, out)?;
                Ok((RExpr::Ref { inner: Box::new(v) }, ty))
            }
            hir::ExprKind::BorrowMut { local } => {
                let name = self.local_name(local.0)?;
                Ok((RExpr::RefMut { inner: Box::new(RExpr::Local(name)) }, ty))
            }
            hir::ExprKind::Move { inner } => {
                // whole-root ownership transfer: the Rust move IS the
                // transfer; the source dies here (verifier-enforced).
                let (v, t) = self.lower_expr(*inner, out)?;
                if !matches!(v, RExpr::Local(_)) {
                    return Err(A2xReject::unsupported_expr(
                        "`.move` wrapper must wrap a whole-local read",
                    ));
                }
                Ok((v, t))
            }
            hir::ExprKind::RecordClone { .. } => Err(A2xReject::new(
                "a2x.unsupported-expr",
                format!(
                    "in `{}`: record deep-clone is not yet in this backend's frozen surface (T-04)",
                    self.facts.source_name
                ),
            )),
            other => Err(A2xReject::unsupported_expr(format!(
                "in `{}`: expression kind `{:?}` is outside the compute surface",
                self.facts.name, other
            ))),
        }
    }

    /// Legalize a placed call argument against its target param's mode.
    fn legalize_arg(&self, raw: RExpr, mode: hir::ParamMode, pty: Ty) -> Result<RExpr, A2xReject> {
        match mode {
            hir::ParamMode::View => match pty {
                // text views: pass `&str` content views
                Ty::Str | Ty::StrOwned => Ok(raw),
                // record views: borrow the place
                Ty::Record(_) => match raw {
                    RExpr::Ref { .. } => Ok(raw),
                    other => Ok(RExpr::Ref { inner: Box::new(other) }),
                },
                _ => Ok(raw),
            },
            hir::ParamMode::MutBorrow => Ok(raw),
            hir::ParamMode::MoveOwn => Ok(raw),
        }
    }

    /// Field names of a path rooted at an expression type (reads).
    fn field_names_of(
        &self,
        root_ty: Ty,
        path: &[hir::FieldId],
    ) -> Result<Vec<String>, A2xReject> {
        let mut names = Vec::with_capacity(path.len());
        let mut cur = root_ty;
        for fid in path {
            let Ty::Record(tid) = cur else {
                return Err(A2xReject::new(
                    "a2x.internal",
                    "field path traverses a non-record type",
                ));
            };
            let rec = self
                .view
                .record(tid)
                .ok_or_else(|| A2xReject::new("a2x.internal", "unknown record type"))?;
            let fld = rec.fields.get(fid.0 as usize).ok_or_else(|| {
                A2xReject::new("a2x.internal", "field index out of range")
            })?;
            names.push(format!("f_{}", sanitize(&fld.name)));
            cur = fld.ty;
        }
        Ok(names)
    }

    /// Lower a subexpression and pin its effectful root into a temporary so
    /// the caller receives a side-effect-free value (source-order, exactly
    /// once). Pure roots pass through unchanged.
    fn lower_pure(
        &mut self,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<(RExpr, Ty), A2xReject> {
        let (v, ty) = self.lower_expr(id, out)?;
        if v.needs_pin() {
            // immediate-after-consumer anchor: earlier view-consumed temps
            // free before this new allocation point
            self.drain_frees(out);
            let t = self.fresh_temp();
            out.push(RStmt::Let {
                name: t.clone(),
                ty,
                mutable: false,
                init: v,
            });
            Ok((RExpr::Local(t), ty))
        } else {
            Ok((v, ty))
        }
    }

    /// Like [`lower_pure`] for OWNING positions (let/assign/return/move
    /// args): an owned pinned temporary transfers its ownership onward, so
    /// it is NOT registered for a free.
    fn lower_move_value(
        &mut self,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<(RExpr, Ty), A2xReject> {
        let (v, ty) = self.lower_expr(id, out)?;
        if v.needs_pin() {
            self.drain_frees(out);
            let t = self.fresh_temp();
            out.push(RStmt::Let {
                name: t.clone(),
                ty,
                mutable: false,
                init: v,
            });
            Ok((RExpr::Local(t), ty))
        } else {
            Ok((v, ty))
        }
    }

    /// Like [`lower_pure`] for VIEW positions (len/concat/eq operands,
    /// view args): an owned pinned temporary is consumed as a view, so it
    /// frees right after its consumer (next pin or statement end).
    fn lower_view_value(
        &mut self,
        id: hir::ExprId,
        out: &mut Vec<RStmt>,
    ) -> Result<(RExpr, Ty), A2xReject> {
        let (v, ty) = self.lower_expr(id, out)?;
        if v.needs_pin() {
            self.drain_frees(out);
            let t = self.fresh_temp();
            out.push(RStmt::Let {
                name: t.clone(),
                ty,
                mutable: false,
                init: v,
            });
            if matches!(ty, Ty::StrOwned) {
                self.pending_frees.push(t.clone());
            }
            Ok((RExpr::Local(t), ty))
        } else {
            Ok((v, ty))
        }
    }
}

fn arith(func: &'static str, l: RExpr, r: RExpr) -> RExpr {
    RExpr::Checked {
        func,
        args: vec![l, r],
    }
}

fn cmp(op: CmpOp, l: RExpr, r: RExpr) -> RExpr {
    RExpr::Cmp {
        op,
        lhs: Box::new(l),
        rhs: Box::new(r),
    }
}

/// Collect every local that is a WHOLE-LOCAL assignment target anywhere in
/// the body (Rust `mut` decision; field-path writes are excluded — they
/// write through the root's reference, never rebind it).
fn collect_assignees(f: &FnFacts<'_>, into: &mut std::collections::BTreeSet<u32>) {
    for block in &f.body.blocks {
        for stmt in &block.stmts {
            if let hir::Stmt::Assign { place, .. } = stmt {
                if let Some(p) = f.body.places.get(place.0 as usize) {
                    if matches!(p.kind, hir::PlaceKind::Local) {
                        into.insert(p.local.0);
                    }
                }
            }
        }
    }
}

pub(super) fn cmp_rust_str(op: CmpOp) -> &'static str {
    op.rust_str()
}

/// Record facts lookup by raw `Ty` (None for non-records).
fn record_of<'v>(view: &'v SemanticView<'_>, ty: Ty) -> Option<&'v RecordFacts> {
    match ty {
        Ty::Record(id) => view.record(id),
        _ => None,
    }
}
