//! Semantic verifier for the core-i32 and scalar-i32 profiles (PLAN-741 /
//! PLAN-767).
//!
//! Consumes a bound `hir::Bundle` and produces a `CheckedModule`. The only
//! way to obtain a `CheckedModule` is to run this verifier — a `checked`
//! marker inside an input file carries no authority (the binder rejects
//! unknown fields, so no such marker can even be expressed).
//!
//! Profile entries (PLAN-767 §5.1): [`verify`] keeps its historical
//! compatibility semantics LIMITED to the core domain — a bundle bound under
//! any other identity is rejected with `verify.domain`. The scalar domain has
//! its own explicit entry, [`verify_scalar`]. Within the core domain the
//! verifier also rejects every scalar-only operator and the unary/logical
//! node kinds (`verify.profile-op`), so programmatic bundle mutation cannot
//! smuggle the widened shared enums past the old profile.
//!
//! Checks (see plan §5.2 and the rejection matrix in
//! docs/design/strategy/auto-hir-atom-text.md §9):
//! - owner consistency: bodies are owned by function defs whose `body` field
//!   points back at them; intrinsics own no bodies,
//! - signatures: param index bounds/uniqueness, param local types matching
//!   the signature, value-only local types (i32/bool),
//! - result types per operator (arithmetic produce i32, comparisons/eq/ne
//!   produce bool with same-type operands, unary neg i32→i32 / not bool→bool,
//!   logical and/or bool×bool→bool), operand types, call argument/result
//!   types, `if` conditions must be bool, `return` values must match the
//!   function result,
//! - definite initialization with structured dataflow: `let` initializes,
//!   `assign` requires an initialized mutable local, if-exit initialization
//!   is the intersection of reachable branches, loops are treated as
//!   possibly zero-iteration (initializations inside a loop do not escape),
//! - mutability on writes (place access and target local),
//! - loop targets must be loops enclosing the current block,
//! - block structure: every non-entry block has exactly one incoming
//!   reference (unique entry role, no sharing), all blocks are reachable
//!   from the entry, every block ends in a control transfer, nothing follows
//!   a terminator, and every function body has a return path,
//! - evaluation positions: every expression has exactly one owning reference
//!   (no sharing, no dead expressions) and the expression graph is acyclic.
//!   Short-circuit `Logical` RHS is statically owned exactly once even when
//!   a given run never executes it.

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::hir::{self, BodyId, ExprId, HirDomain, LocalId, LoopId};
use std::collections::{BTreeMap, BTreeSet};

/// A module that has passed semantic verification. Constructed only by
/// [`verify`]/[`verify_scalar`]/[`verify_text`]; the backend consumes
/// nothing else.
#[derive(Clone, Debug)]
pub struct CheckedModule {
    bundle: hir::Bundle,
    /// The text resource plan (PLAN-770): present only for bundles verified
    /// under the text domain. Owned locals, per-exit LIFO free schedules and
    /// owned-return statements, computed by the verifier — the backend
    /// consumes this plan without re-deriving ownership.
    text_plan: Option<crate::text_resources::TextResourcePlan>,
}

impl CheckedModule {
    pub fn bundle(&self) -> &hir::Bundle {
        &self.bundle
    }

    pub fn into_bundle(self) -> hir::Bundle {
        self.bundle
    }

    /// The descriptor identity this module was verified under.
    pub fn domain(&self) -> HirDomain {
        self.bundle.domain
    }

    /// Capabilities this module requires (from the document-level
    /// `requires` list). The build path must check them against the
    /// capabilities the target actually provides.
    pub fn required_capabilities(&self) -> &[String] {
        &self.bundle.requires
    }

    /// The verifier-computed text resource plan. `None` outside the text
    /// domain; for text modules the lowering reads owned sets and exit free
    /// schedules from here instead of re-analyzing ownership.
    pub fn text_resources(&self) -> Option<&crate::text_resources::TextResourcePlan> {
        self.text_plan.as_ref()
    }
}

/// Verify a bound core-domain bundle (historical entry; compatibility
/// semantics limited to `core-i32-draft`). Collects all verifier diagnostics.
pub fn verify(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    verify_domain(bundle, HirDomain::Core)
}

/// Verify a bound scalar-domain bundle (`scalar-i32-draft`, PLAN-767). The
/// explicit new-identity entry; never upgrades or reinterprets a bundle of
/// another domain.
pub fn verify_scalar(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    verify_domain(bundle, HirDomain::Scalar)
}

/// Verify a bound text-domain bundle (`text-i32-draft`, PLAN-770): the
/// scalar surface plus str/Str/char under the 774 data-resource contract.
/// On success the `CheckedModule` carries the text resource plan (owned
/// locals, per-exit LIFO free schedules, owned returns) that the backend
/// consumes verbatim — the backend performs no ownership analysis of its
/// own.
pub fn verify_text(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    let mut v = Verifier {
        diags: Vec::new(),
        domain: HirDomain::Text,
    };
    let mut plan = crate::text_resources::TextResourcePlan::default();
    for m in &bundle.modules {
        v.check_module(m);
    }
    if v.diags.is_empty() {
        for (mi, m) in bundle.modules.iter().enumerate() {
            for (bi, body) in m.bodies.iter().enumerate() {
                let type_map = Verifier::build_type_map(m);
                if let Some(bp) =
                    crate::text_resources::check_body(body, m, &type_map, &mut v.diags)
                {
                    plan.bodies.insert((mi, bi as u32), bp);
                }
            }
        }
    }
    if v.diags.is_empty() {
        Ok(CheckedModule {
            bundle,
            text_plan: Some(plan),
        })
    } else {
        Err(v.diags)
    }
}

fn verify_domain(bundle: hir::Bundle, expect: HirDomain) -> Result<CheckedModule, Vec<Diagnostic>> {
    if bundle.domain != expect {
        return Err(vec![Diagnostic::new(
            Stage::Verify,
            "verify.domain",
            format!(
                "bundle identity `{}` does not match this verifier entry (`{}`); use the entry of the document's own descriptor profile",
                bundle.domain.as_str(),
                expect.as_str()
            ),
            Span { start: 0, end: 0 },
        )]);
    }
    let mut v = Verifier {
        diags: Vec::new(),
        domain: expect,
    };
    v.check_bundle(&bundle);
    if v.diags.is_empty() {
        Ok(CheckedModule {
            bundle,
            text_plan: None,
        })
    } else {
        Err(v.diags)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ValueType {
    I32,
    Bool,
    Function,
    /// Borrowed text slice (`str`, PLAN-770).
    Str,
    /// Owned text buffer (`Str`, PLAN-770).
    StrOwned,
    /// Unicode scalar char (PLAN-770).
    Char,
}

pub(crate) fn vt_name(v: ValueType) -> &'static str {
    match v {
        ValueType::I32 => "i32",
        ValueType::Bool => "bool",
        ValueType::Function => "function",
        ValueType::Str => "str",
        ValueType::StrOwned => "Str",
        ValueType::Char => "char",
    }
}

fn type_index(u: &hir::TypeUse) -> u32 {
    match u {
        hir::TypeUse::Ref(t) => t.0,
        _ => u32::MAX,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Flow {
    /// Execution continues to the next statement / block end.
    Continue,
    /// Block exited via `return`.
    Returned,
    /// Block exited via `break`/`continue`.
    Diverged,
}

type Init = BTreeSet<LocalId>;
type SigTable = BTreeMap<u32, (Vec<ValueType>, ValueType)>;

/// Memo for expression type computation, with an in-progress (gray) set so
/// that illegal cyclic expressions cannot recurse infinitely — the cycle is
/// already reported by `check_expr_graph`.
#[derive(Default)]
struct TypeMemo {
    memo: BTreeMap<u32, ValueType>,
    gray: BTreeSet<u32>,
}

struct Verifier {
    diags: Vec<Diagnostic>,
    domain: HirDomain,
}

impl Verifier {
    fn error(&mut self, code: &str, msg: impl Into<String>, span: Span) {
        self.diags
            .push(Diagnostic::new(Stage::Verify, code, msg, span));
    }

    /// Core/scalar-domain defense (PLAN-767 §5.1, PLAN-770 isolation): the
    /// shared HIR enums were widened by later profiles; a programmatically
    /// mutated bundle of an OLDER profile must still be rejected at its own
    /// surface, never silently reinterpreted under the new semantics. Core
    /// additionally rejects every scalar-only operator; both reject the
    /// entire text surface (expression kinds, drop statements, parameter
    /// modes, str/Str/char types on locals and signatures).
    fn check_profile_ops(&mut self, body: &hir::Body) {
        if self.domain == HirDomain::Text {
            return;
        }
        for e in &body.exprs {
            let text_kind = matches!(
                &e.kind,
                hir::ExprKind::StrLit { .. }
                    | hir::ExprKind::CharLit { .. }
                    | hir::ExprKind::TextLen { .. }
                    | hir::ExprKind::TextLenBytes { .. }
                    | hir::ExprKind::TextCharAt { .. }
                    | hir::ExprKind::TextSlice { .. }
                    | hir::ExprKind::TextConcat { .. }
                    | hir::ExprKind::TextClone { .. }
                    | hir::ExprKind::TextToOwned { .. }
                    | hir::ExprKind::TextToBorrowed { .. }
                    | hir::ExprKind::CharToCodepoint { .. }
                    | hir::ExprKind::CharFromCodepoint { .. }
            );
            if text_kind {
                self.error(
                    "verify.profile-op",
                    format!(
                        "text expression `{}` is not part of the {} profile",
                        e.id_text,
                        self.domain.as_str()
                    ),
                    e.span,
                );
                continue;
            }
            if self.domain != HirDomain::Core {
                continue;
            }
            match &e.kind {
                hir::ExprKind::Binary { op, .. } if !op.in_core_profile() => {
                    self.error(
                        "verify.profile-op",
                        format!(
                            "op `{}` is not part of the core-i32-draft profile",
                            op.as_str()
                        ),
                        e.span,
                    );
                }
                hir::ExprKind::Unary { op, .. } => {
                    self.error(
                        "verify.profile-op",
                        format!(
                            "unary op `{}` is not part of the core-i32-draft profile",
                            op.as_str()
                        ),
                        e.span,
                    );
                }
                hir::ExprKind::Logical { op, .. } => {
                    self.error(
                        "verify.profile-op",
                        format!(
                            "logical op `{}` is not part of the core-i32-draft profile",
                            op.as_str()
                        ),
                        e.span,
                    );
                }
                _ => {}
            }
        }
        for bl in &body.blocks {
            for s in &bl.stmts {
                if matches!(s, hir::Stmt::Drop { .. }) {
                    self.error(
                        "verify.profile-op",
                        format!(
                            "drop statements are not part of the {} profile",
                            self.domain.as_str()
                        ),
                        bl.span,
                    );
                }
            }
        }
        for l in &body.locals {
            if l.param_mode != hir::ParamMode::View {
                self.error(
                    "verify.profile-op",
                    format!(
                        "parameter mode `{}` is not part of the {} profile",
                        l.param_mode.as_str(),
                        self.domain.as_str()
                    ),
                    l.span,
                );
            }
        }
    }

    fn check_bundle(&mut self, bundle: &hir::Bundle) {
        for m in &bundle.modules {
            self.check_module(m);
        }
    }

    fn value_of(&self, u: &hir::TypeUse, type_map: &BTreeMap<u32, ValueType>) -> ValueType {
        match u {
            hir::TypeUse::Builtin(b) => match b {
                hir::BuiltinType::I32 => ValueType::I32,
                hir::BuiltinType::Bool => ValueType::Bool,
                hir::BuiltinType::StrSlice => ValueType::Str,
                hir::BuiltinType::StrOwned => ValueType::StrOwned,
                hir::BuiltinType::Char => ValueType::Char,
            },
            hir::TypeUse::Ref(tid) => type_map.get(&tid.0).copied().unwrap_or(ValueType::I32),
        }
    }

    fn build_type_map(m: &hir::Module) -> BTreeMap<u32, ValueType> {
        m.types
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let vt = match &t.kind {
                    hir::TypeKind::I32 => ValueType::I32,
                    hir::TypeKind::Bool => ValueType::Bool,
                    hir::TypeKind::Str => ValueType::Str,
                    hir::TypeKind::StrOwned => ValueType::StrOwned,
                    hir::TypeKind::Char => ValueType::Char,
                    hir::TypeKind::Function(_) => ValueType::Function,
                };
                (i as u32, vt)
            })
            .collect()
    }

    fn check_module(&mut self, m: &hir::Module) {
        // Module-level type-reference bounds FIRST (review P767-R2): a
        // programmatically mutated bundle may carry out-of-range TypeIds in
        // function-signature params/results. `value_of` would silently
        // default them to i32 and hand a Checked to the native backend,
        // which then indexes the type table and panics. Reject in a
        // controlled way and skip this module's signature/body walks.
        if !self.check_module_type_bounds(m) {
            return;
        }
        // Text kinds in the type table exist only in the text profile
        // (PLAN-770 isolation): a programmatic mutation cannot smuggle
        // str/Str/char named types into an older profile's signatures.
        if self.domain != HirDomain::Text {
            for t in &m.types {
                if matches!(
                    t.kind,
                    hir::TypeKind::Str | hir::TypeKind::StrOwned | hir::TypeKind::Char
                ) {
                    self.error(
                        "verify.local-type",
                        format!(
                            "type `{}` has text kind `{}`; text types exist only in the text-i32-draft profile",
                            t.id_text,
                            match t.kind {
                                hir::TypeKind::Str => "str",
                                hir::TypeKind::StrOwned => "Str",
                                _ => "char",
                            }
                        ),
                        t.span,
                    );
                }
            }
        }
        let type_map = Self::build_type_map(m);
        // Function signatures by def index.
        let mut sigs: SigTable = BTreeMap::new();
        for (i, d) in m.defs.iter().enumerate() {
            let ty_id = match &d.kind {
                hir::DefKind::Function { ty, .. } | hir::DefKind::Intrinsic { ty, .. } => *ty,
            };
            match m.types.get(ty_id.0 as usize).map(|t| &t.kind) {
                Some(hir::TypeKind::Function(sig)) => {
                    let params: Vec<ValueType> = sig
                        .params
                        .iter()
                        .map(|p| self.value_of(p, &type_map))
                        .collect();
                    let result = self.value_of(&sig.result, &type_map);
                    sigs.insert(i as u32, (params, result));
                }
                _ => {
                    self.error(
                        "verify.callee-type",
                        format!("declaration `{}` does not have a function type", d.id_text),
                        d.span,
                    );
                }
            }
        }

        // function -> body direction: each function's body field must point
        // at a body owned by exactly this function. This rejects shared
        // bodies (including same-signature aliases) and functions whose
        // declared body belongs to someone else (QA-03: r1 only walked the
        // body -> owner direction, so a zero-param alias sharing a two-param
        // body passed verify and panicked the backend).
        for (di, d) in m.defs.iter().enumerate() {
            let hir::DefKind::Function { body: def_body, .. } = &d.kind else {
                continue;
            };
            let Some(b) = m.bodies.get(def_body.0 as usize) else {
                continue; // binder rejects dangling body refs
            };
            if b.owner.0 as usize != di {
                let owner = m
                    .defs
                    .get(b.owner.0 as usize)
                    .map(|o| o.id_text.as_str())
                    .unwrap_or("<unknown>");
                self.error(
                    "verify.owner-mismatch",
                    format!(
                        "function `{}` declares body `{}` but that body is owned by `{}`; every body must belong to exactly one function",
                        d.id_text, b.id_text, owner
                    ),
                    d.span,
                );
            }
        }

        for (bi, body) in m.bodies.iter().enumerate() {
            self.check_profile_ops(body);
            self.check_body(m, BodyId(bi as u32), body, &type_map, &sigs);
        }
    }

    /// Bounds-check every `TypeUse::Ref` inside the module's function-type
    /// signatures (params and result) against the module's own type table.
    /// Text-bound bundles cannot violate this (the binder resolves type
    /// references against the table); mutated bundles can, and must be
    /// rejected before any signature-driven walk or native lowering.
    fn check_module_type_bounds(&mut self, m: &hir::Module) -> bool {
        let n_types = m.types.len();
        let mut diags: Vec<Diagnostic> = Vec::new();
        for t in &m.types {
            let hir::TypeKind::Function(sig) = &t.kind else {
                continue;
            };
            let mut check_use = |role: &str, u: &hir::TypeUse| {
                if let hir::TypeUse::Ref(tid) = u {
                    if (tid.0 as usize) >= n_types {
                        diags.push(Diagnostic::new(
                            Stage::Verify,
                            "verify.bad-reference",
                            format!(
                                "type index `{}` in function type `{}`'s {} is out of range for the module's type table (length {})",
                                tid.0, t.id_text, role, n_types
                            ),
                            t.span,
                        ));
                    }
                }
            };
            for p in &sig.params {
                check_use("param", p);
            }
            check_use("result", &sig.result);
        }
        let ok = diags.is_empty();
        self.diags.extend(diags);
        ok
    }

    fn check_body(
        &mut self,
        m: &hir::Module,
        body_id: BodyId,
        body: &hir::Body,
        type_map: &BTreeMap<u32, ValueType>,
        sigs: &SigTable,
    ) {
        // Programmatic bundles may carry ids from another body's tables
        // (PLAN-767 V-07: cross-body references). The indexing walks below
        // assume in-range ids — bounds-check everything first and reject in
        // a controlled way instead of panicking.
        if !self.check_reference_bounds(m, body) {
            return;
        }
        let Some(def) = m.defs.get(body.owner.0 as usize) else {
            return; // binder guarantees resolution; errors already recorded
        };
        let hir::DefKind::Function { body: def_body, .. } = &def.kind else {
            self.error(
                "verify.owner-mismatch",
                format!(
                    "body `{}` is owned by `{}`, which is not a function",
                    body.id_text, def.id_text
                ),
                body.span,
            );
            return;
        };
        if *def_body != body_id {
            self.error(
                "verify.owner-mismatch",
                format!(
                    "body `{}` is owned by `{}` but that function's body field points elsewhere",
                    body.id_text, def.id_text
                ),
                body.span,
            );
        }
        let Some((sig_params, fn_result)) = sigs.get(&body.owner.0) else {
            return;
        };
        self.check_params(body, sig_params, type_map);
        self.check_blocks(body, *fn_result, type_map, sigs);
    }

    /// Bounds-check every table reference inside one body (plus its type
    /// uses against the module's type table). Text-bound bundles cannot
    /// violate these (the binder resolves against the body's own tables);
    /// mutated bundles can, and must be rejected before any indexing walk.
    fn check_reference_bounds(&mut self, m: &hir::Module, body: &hir::Body) -> bool {
        fn type_use_ok(u: &hir::TypeUse, n_types: usize) -> bool {
            match u {
                hir::TypeUse::Ref(t) => (t.0 as usize) < n_types,
                _ => true,
            }
        }
        let n_types = m.types.len();
        let n_defs = m.defs.len();
        let n_locals = body.locals.len();
        let n_places = body.places.len();
        let n_exprs = body.exprs.len();
        let n_blocks = body.blocks.len();
        let n_loops = body.loops.len();
        let mut diags: Vec<Diagnostic> = Vec::new();
        let mut bad = |what: &str, idx: u32, len: usize, span: Span| {
            diags.push(Diagnostic::new(
                Stage::Verify,
                "verify.bad-reference",
                format!(
                    "{} index `{}` is out of range for this body's tables (length {})",
                    what, idx, len
                ),
                span,
            ));
        };
        if (body.entry.0 as usize) >= n_blocks {
            bad("block", body.entry.0, n_blocks, body.span);
        }
        for l in &body.locals {
            if !type_use_ok(&l.ty, n_types) {
                bad("type", type_index(&l.ty), n_types, l.span);
            }
        }
        for p in &body.places {
            if (p.local.0 as usize) >= n_locals {
                bad("local", p.local.0, n_locals, p.span);
            }
            if !type_use_ok(&p.ty, n_types) {
                bad("type", type_index(&p.ty), n_types, p.span);
            }
        }
        for e in &body.exprs {
            if !type_use_ok(&e.ty, n_types) {
                bad("type", type_index(&e.ty), n_types, e.span);
            }
            match &e.kind {
                hir::ExprKind::ReadLocal { local } => {
                    if (local.0 as usize) >= n_locals {
                        bad("local", local.0, n_locals, e.span);
                    }
                }
                hir::ExprKind::Binary { lhs, rhs, .. } => {
                    for id in [lhs, rhs] {
                        if (id.0 as usize) >= n_exprs {
                            bad("expression", id.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::Unary { operand, .. } => {
                    if (operand.0 as usize) >= n_exprs {
                        bad("expression", operand.0, n_exprs, e.span);
                    }
                }
                hir::ExprKind::Logical { lhs, rhs, .. } => {
                    for id in [lhs, rhs] {
                        if (id.0 as usize) >= n_exprs {
                            bad("expression", id.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::Call {
                    callee, eval_args, ..
                } => {
                    if (callee.0 as usize) >= n_defs {
                        bad("declaration", callee.0, n_defs, e.span);
                    }
                    for a in eval_args {
                        if (a.0 as usize) >= n_exprs {
                            bad("expression", a.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::TextLen { operand }
                | hir::ExprKind::TextLenBytes { operand }
                | hir::ExprKind::TextClone { operand }
                | hir::ExprKind::TextToOwned { operand }
                | hir::ExprKind::TextToBorrowed { operand }
                | hir::ExprKind::CharToCodepoint { operand }
                | hir::ExprKind::CharFromCodepoint { operand } => {
                    if (operand.0 as usize) >= n_exprs {
                        bad("expression", operand.0, n_exprs, e.span);
                    }
                }
                hir::ExprKind::TextCharAt { operand, index } => {
                    for id in [operand, index] {
                        if (id.0 as usize) >= n_exprs {
                            bad("expression", id.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::TextSlice {
                    operand,
                    start,
                    end,
                } => {
                    for id in [operand, start, end] {
                        if (id.0 as usize) >= n_exprs {
                            bad("expression", id.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::TextConcat { lhs, rhs } => {
                    for id in [lhs, rhs] {
                        if (id.0 as usize) >= n_exprs {
                            bad("expression", id.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::StrLit { .. } | hir::ExprKind::CharLit { .. } => {}
                hir::ExprKind::Constant { .. } => {}
            }
        }
        for bl in &body.blocks {
            for s in &bl.stmts {
                match s {
                    hir::Stmt::Let { local, value } => {
                        if (local.0 as usize) >= n_locals {
                            bad("local", local.0, n_locals, bl.span);
                        }
                        if (value.0 as usize) >= n_exprs {
                            bad("expression", value.0, n_exprs, bl.span);
                        }
                    }
                    hir::Stmt::Return { value } => {
                        if (value.0 as usize) >= n_exprs {
                            bad("expression", value.0, n_exprs, bl.span);
                        }
                    }
                    hir::Stmt::If { cond, then, els } => {
                        if (cond.0 as usize) >= n_exprs {
                            bad("expression", cond.0, n_exprs, bl.span);
                        }
                        if (then.0 as usize) >= n_blocks {
                            bad("block", then.0, n_blocks, bl.span);
                        }
                        if let Some(e) = els {
                            if (e.0 as usize) >= n_blocks {
                                bad("block", e.0, n_blocks, bl.span);
                            }
                        }
                    }
                    hir::Stmt::Loop { id, body: b } => {
                        if (id.0 as usize) >= n_loops {
                            bad("loop", id.0, n_loops, bl.span);
                        }
                        if (b.0 as usize) >= n_blocks {
                            bad("block", b.0, n_blocks, bl.span);
                        }
                    }
                    hir::Stmt::Assign { place, value } => {
                        if (place.0 as usize) >= n_places {
                            bad("place", place.0, n_places, bl.span);
                        }
                        if (value.0 as usize) >= n_exprs {
                            bad("expression", value.0, n_exprs, bl.span);
                        }
                    }
                    hir::Stmt::Break { target } | hir::Stmt::Continue { target } => {
                        if (target.0 as usize) >= n_loops {
                            bad("loop", target.0, n_loops, bl.span);
                        }
                    }
                    hir::Stmt::Drop { local } => {
                        if (local.0 as usize) >= n_locals {
                            bad("local", local.0, n_locals, bl.span);
                        }
                    }
                }
            }
        }
        let ok = diags.is_empty();
        self.diags.extend(diags);
        ok
    }

    fn check_params(
        &mut self,
        body: &hir::Body,
        sig_params: &[ValueType],
        type_map: &BTreeMap<u32, ValueType>,
    ) {
        let mut seen: BTreeSet<u32> = BTreeSet::new();
        for l in &body.locals {
            let lvt = self.value_of(&l.ty, type_map);
            let text_vt = matches!(lvt, ValueType::Str | ValueType::StrOwned | ValueType::Char);
            if lvt == ValueType::Function {
                self.error(
                    "verify.local-type",
                    format!(
                        "local `{}` has a function type; only i32/bool locals exist in this profile",
                        l.id_text
                    ),
                    l.span,
                );
            } else if text_vt && self.domain != HirDomain::Text {
                self.error(
                    "verify.local-type",
                    format!(
                        "local `{}` has text type `{}`; text types exist only in the text-i32-draft profile",
                        l.id_text,
                        vt_name(lvt)
                    ),
                    l.span,
                );
            }
            // Parameter ownership modes (PLAN-770): `move` transfers an
            // owned buffer — only an owned Str parameter can carry it;
            // `mut` is an exclusive text borrow — only text parameters.
            if self.domain == HirDomain::Text && l.role == hir::LocalRole::Param {
                match l.param_mode {
                    hir::ParamMode::View => {}
                    hir::ParamMode::MutBorrow if text_vt => {}
                    hir::ParamMode::MoveOwn if lvt == ValueType::StrOwned => {}
                    hir::ParamMode::MutBorrow | hir::ParamMode::MoveOwn => {
                        self.error(
                            "verify.ownership-required",
                            format!(
                                "parameter `{}` declares mode `{}` but has type `{}`; `move` requires Str, `mut` requires a text type",
                                l.id_text,
                                l.param_mode.as_str(),
                                vt_name(lvt)
                            ),
                            l.span,
                        );
                    }
                }
            }
            if l.role == hir::LocalRole::Param {
                let Some(idx) = l.index else {
                    self.error(
                        "verify.binding-invalid",
                        format!("param local `{}` has no index", l.id_text),
                        l.span,
                    );
                    continue;
                };
                if !seen.insert(idx) {
                    self.error(
                        "verify.binding-invalid",
                        format!(
                            "param index {} is used more than once in body `{}`",
                            idx, body.id_text
                        ),
                        l.span,
                    );
                    continue;
                }
                if idx as usize >= sig_params.len() {
                    self.error(
                        "verify.binding-invalid",
                        format!(
                            "param index {} is out of range for signature with {} params",
                            idx,
                            sig_params.len()
                        ),
                        l.span,
                    );
                    continue;
                }
                if lvt != sig_params[idx as usize] {
                    self.error(
                        "verify.type-mismatch",
                        format!(
                            "param local `{}` type does not match signature param {}",
                            l.id_text, idx
                        ),
                        l.span,
                    );
                }
            }
        }
    }

    // -- expression types ----------------------------------------------------

    fn expr_type(
        &mut self,
        eid: ExprId,
        body: &hir::Body,
        type_map: &BTreeMap<u32, ValueType>,
        sigs: &SigTable,
        tm: &mut TypeMemo,
    ) -> ValueType {
        // Iterative post-order walk (PLAN-745 R3-QA-02): operator chains
        // within one statement are unbounded, so the operand walk must not
        // recurse. Semantics are unchanged: memo/gray discipline, the same
        // per-node checks, and the same left-to-right error order (the
        // explicit stack visits lhs fully, then rhs, then the node's own
        // checks — exactly the previous recursion order).
        enum Visit {
            Enter(ExprId),
            Exit(ExprId),
        }
        // local, self-free projection so error emission (&mut self) can
        // interleave freely
        let declared_of = |e: &hir::Expr| -> ValueType {
            match &e.ty {
                hir::TypeUse::Builtin(hir::BuiltinType::I32) => ValueType::I32,
                hir::TypeUse::Builtin(hir::BuiltinType::Bool) => ValueType::Bool,
                hir::TypeUse::Builtin(hir::BuiltinType::StrSlice) => ValueType::Str,
                hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => ValueType::StrOwned,
                hir::TypeUse::Builtin(hir::BuiltinType::Char) => ValueType::Char,
                hir::TypeUse::Ref(tid) => type_map.get(&tid.0).copied().unwrap_or(ValueType::I32),
            }
        };
        let operand_type = |tm: &TypeMemo, id: ExprId, declared: ValueType| -> ValueType {
            tm.memo.get(&id.0).copied().unwrap_or(declared)
        };
        let mut stack: Vec<Visit> = vec![Visit::Enter(eid)];
        while let Some(visit) = stack.pop() {
            match visit {
                Visit::Enter(id) => {
                    if tm.memo.contains_key(&id.0) {
                        continue;
                    }
                    if !tm.gray.insert(id.0) {
                        // cyclic operand graph; already reported by
                        // check_expr_graph — behave like the recursive form
                        // (contribute the declared type, no memo entry).
                        continue;
                    }
                    let e = &body.exprs[id.0 as usize];
                    let declared = declared_of(e);
                    match &e.kind {
                        hir::ExprKind::ReadLocal { local } => {
                            let l = &body.locals[local.0 as usize];
                            let lt = self.value_of(&l.ty, type_map);
                            if lt != declared {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}` type does not match local `{}` type",
                                        e.id_text, l.id_text
                                    ),
                                    e.span,
                                );
                            }
                            tm.gray.remove(&id.0);
                            tm.memo.insert(id.0, lt);
                        }
                        hir::ExprKind::Constant { .. } => {
                            if declared != ValueType::I32 {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "constant `{}` must have type i32 in this profile",
                                        e.id_text
                                    ),
                                    e.span,
                                );
                            }
                            tm.gray.remove(&id.0);
                            tm.memo.insert(id.0, ValueType::I32);
                        }
                        hir::ExprKind::Binary { lhs, rhs, .. } => {
                            // LIFO: lhs pops first, then rhs, then this
                            // node's own checks — the previous recursion
                            // order.
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*rhs));
                            stack.push(Visit::Enter(*lhs));
                        }
                        hir::ExprKind::Unary { operand, .. } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*operand));
                        }
                        hir::ExprKind::Logical { lhs, rhs, .. } => {
                            // both sides are walked statically even though
                            // the RHS is conditionally executed at runtime
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*rhs));
                            stack.push(Visit::Enter(*lhs));
                        }
                        hir::ExprKind::Call { eval_args, .. } => {
                            stack.push(Visit::Exit(id));
                            for a in eval_args.iter().rev() {
                                stack.push(Visit::Enter(*a));
                            }
                        }
                        hir::ExprKind::StrLit { .. } | hir::ExprKind::CharLit { .. } => {
                            // leaves; typed at Exit from the declared type
                            stack.push(Visit::Exit(id));
                        }
                        hir::ExprKind::TextLen { operand }
                        | hir::ExprKind::TextLenBytes { operand }
                        | hir::ExprKind::TextClone { operand }
                        | hir::ExprKind::TextToOwned { operand }
                        | hir::ExprKind::TextToBorrowed { operand }
                        | hir::ExprKind::CharToCodepoint { operand }
                        | hir::ExprKind::CharFromCodepoint { operand } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*operand));
                        }
                        hir::ExprKind::TextCharAt { operand, index } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*index));
                            stack.push(Visit::Enter(*operand));
                        }
                        hir::ExprKind::TextSlice {
                            operand,
                            start,
                            end,
                        } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*end));
                            stack.push(Visit::Enter(*start));
                            stack.push(Visit::Enter(*operand));
                        }
                        hir::ExprKind::TextConcat { lhs, rhs } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*rhs));
                            stack.push(Visit::Enter(*lhs));
                        }
                    }
                }
                Visit::Exit(id) => {
                    let e = &body.exprs[id.0 as usize];
                    let declared = declared_of(e);
                    let computed = match &e.kind {
                        hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                            let lt =
                                operand_type(tm, *lhs, declared_of(&body.exprs[lhs.0 as usize]));
                            let rt =
                                operand_type(tm, *rhs, declared_of(&body.exprs[rhs.0 as usize]));
                            use hir::BinOp::*;
                            match op {
                                AddI32 | SubI32 | MulI32 | DivI32 | RemI32 => {
                                    for (role, vt) in [("lhs", lt), ("rhs", rt)] {
                                        if vt != ValueType::I32 {
                                            self.error(
                                                "verify.type-mismatch",
                                                format!(
                                                    "expr `{}`: {} operand of {} must be i32",
                                                    e.id_text,
                                                    role,
                                                    op.as_str()
                                                ),
                                                e.span,
                                            );
                                        }
                                    }
                                    if declared != ValueType::I32 {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: {} must produce i32",
                                                e.id_text,
                                                op.as_str()
                                            ),
                                            e.span,
                                        );
                                    }
                                    ValueType::I32
                                }
                                LtI32 | LeI32 | GtI32 | GeI32 => {
                                    for (role, vt) in [("lhs", lt), ("rhs", rt)] {
                                        if vt != ValueType::I32 {
                                            self.error(
                                                "verify.type-mismatch",
                                                format!(
                                                    "expr `{}`: {} operand of {} must be i32",
                                                    e.id_text,
                                                    role,
                                                    op.as_str()
                                                ),
                                                e.span,
                                            );
                                        }
                                    }
                                    if declared != ValueType::Bool {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: {} must produce bool",
                                                e.id_text,
                                                op.as_str()
                                            ),
                                            e.span,
                                        );
                                    }
                                    ValueType::Bool
                                }
                                Eq | Ne => {
                                    let same_pair = lt == rt
                                        && matches!(
                                            (lt, rt),
                                            (ValueType::I32, ValueType::I32)
                                                | (ValueType::Bool, ValueType::Bool)
                                                | (ValueType::Str, ValueType::Str)
                                                | (ValueType::StrOwned, ValueType::StrOwned)
                                                | (ValueType::Char, ValueType::Char)
                                        );
                                    if !same_pair {
                                        // keep the old profiles' exact
                                        // wording (observable diagnostic
                                        // surface); the text profile adds
                                        // its wider type vocabulary.
                                        let msg = if self.domain == HirDomain::Text {
                                            format!(
                                                "expr `{}`: {} operands must both be i32, bool, str, Str or char of the same type (found {} / {})",
                                                e.id_text,
                                                op.as_str(),
                                                vt_name(lt),
                                                vt_name(rt)
                                            )
                                        } else {
                                            format!(
                                                "expr `{}`: {} operands must both be i32 or both be bool (found {} / {})",
                                                e.id_text,
                                                op.as_str(),
                                                vt_name(lt),
                                                vt_name(rt)
                                            )
                                        };
                                        self.error("verify.type-mismatch", msg, e.span);
                                    }
                                    if declared != ValueType::Bool {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: {} must produce bool",
                                                e.id_text,
                                                op.as_str()
                                            ),
                                            e.span,
                                        );
                                    }
                                    ValueType::Bool
                                }
                            }
                        }
                        hir::ExprKind::Unary { op, operand } => {
                            let ot = operand_type(
                                tm,
                                *operand,
                                declared_of(&body.exprs[operand.0 as usize]),
                            );
                            match op {
                                hir::UnaryOp::NegI32 => {
                                    if ot != ValueType::I32 {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: operand of neg_i32 must be i32",
                                                e.id_text
                                            ),
                                            e.span,
                                        );
                                    }
                                    if declared != ValueType::I32 {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: neg_i32 must produce i32",
                                                e.id_text
                                            ),
                                            e.span,
                                        );
                                    }
                                    ValueType::I32
                                }
                                hir::UnaryOp::NotBool => {
                                    if ot != ValueType::Bool {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: operand of not_bool must be bool",
                                                e.id_text
                                            ),
                                            e.span,
                                        );
                                    }
                                    if declared != ValueType::Bool {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: not_bool must produce bool",
                                                e.id_text
                                            ),
                                            e.span,
                                        );
                                    }
                                    ValueType::Bool
                                }
                            }
                        }
                        hir::ExprKind::Logical { op, lhs, rhs } => {
                            let lt =
                                operand_type(tm, *lhs, declared_of(&body.exprs[lhs.0 as usize]));
                            let rt =
                                operand_type(tm, *rhs, declared_of(&body.exprs[rhs.0 as usize]));
                            for (role, vt) in [("lhs", lt), ("rhs", rt)] {
                                if vt != ValueType::Bool {
                                    self.error(
                                        "verify.type-mismatch",
                                        format!(
                                            "expr `{}`: {} operand of logical {} must be bool",
                                            e.id_text,
                                            role,
                                            op.as_str()
                                        ),
                                        e.span,
                                    );
                                }
                            }
                            if declared != ValueType::Bool {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: logical {} must produce bool",
                                        e.id_text,
                                        op.as_str()
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::Bool
                        }
                        hir::ExprKind::Call {
                            callee,
                            eval_args,
                            bindings,
                        } => {
                            let Some((params, result)) = sigs.get(&callee.0) else {
                                self.error(
                                    "verify.callee-type",
                                    format!(
                                        "expr `{}`: callee `{}` has no usable signature",
                                        e.id_text,
                                        // safe: binder guarantees the def exists
                                        callee.0
                                    ),
                                    e.span,
                                );
                                tm.gray.remove(&id.0);
                                tm.memo.insert(id.0, declared);
                                continue;
                            };
                            if eval_args.len() != params.len() {
                                self.error(
                                    "verify.eval-args-arity",
                                    format!(
                                        "expr `{}`: {} eval_args for signature with {} params",
                                        e.id_text,
                                        eval_args.len(),
                                        params.len()
                                    ),
                                    e.span,
                                );
                            }
                            // Arguments are evaluated left to right (side-effect order
                            // contract); their types are checked against form params
                            // through the bindings map, not by position (QA-02: r1
                            // compared eval_args[i] with params[i], rejecting legal
                            // swapped mappings and passing illegal ones).
                            let arg_types: Vec<ValueType> = eval_args
                                .iter()
                                .map(|a| {
                                    operand_type(tm, *a, declared_of(&body.exprs[a.0 as usize]))
                                })
                                .collect();
                            let mut seen_params: BTreeSet<u32> = BTreeSet::new();
                            let mut seen_args: BTreeSet<u32> = BTreeSet::new();
                            for b in bindings {
                                if !seen_params.insert(b.param) {
                                    self.error(
                                        "verify.binding-invalid",
                                        format!(
                                            "expr `{}`: form param {} is bound more than once",
                                            e.id_text, b.param
                                        ),
                                        e.span,
                                    );
                                }
                                if b.param as usize >= params.len() {
                                    self.error(
                                        "verify.binding-invalid",
                                        format!(
                                            "expr `{}`: binding param {} is out of range",
                                            e.id_text, b.param
                                        ),
                                        e.span,
                                    );
                                }
                                if b.arg as usize >= eval_args.len() {
                                    self.error(
                                        "verify.binding-invalid",
                                        format!(
                                            "expr `{}`: binding arg {} indexes past {} eval_args",
                                            e.id_text,
                                            b.arg,
                                            eval_args.len()
                                        ),
                                        e.span,
                                    );
                                } else if !seen_args.insert(b.arg) {
                                    self.error(
                                        "verify.binding-invalid",
                                        format!(
                                            "expr `{}`: eval_args[{}] is consumed by more than one binding",
                                            e.id_text, b.arg
                                        ),
                                        e.span,
                                    );
                                }
                            }
                            if seen_params.len() < params.len() {
                                self.error(
                                    "verify.binding-invalid",
                                    format!(
                                        "expr `{}`: {} of {} form params are bound",
                                        e.id_text,
                                        seen_params.len(),
                                        params.len()
                                    ),
                                    e.span,
                                );
                            }
                            for i in 0..eval_args.len() as u32 {
                                if !seen_args.contains(&i) {
                                    self.error(
                                        "verify.binding-invalid",
                                        format!(
                                            "expr `{}`: eval_args[{}] is evaluated but no binding consumes it",
                                            e.id_text, i
                                        ),
                                        e.span,
                                    );
                                }
                            }
                            for b in bindings {
                                if let (Some(pt), Some(at)) =
                                    (params.get(b.param as usize), arg_types.get(b.arg as usize))
                                {
                                    if at != pt {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: param {} ({}) does not match eval_args[{}] ({})",
                                                e.id_text,
                                                b.param,
                                                vt_name(*pt),
                                                b.arg,
                                                vt_name(*at)
                                            ),
                                            e.span,
                                        );
                                    }
                                }
                            }
                            if declared != *result {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: call type does not match callee result",
                                        e.id_text
                                    ),
                                    e.span,
                                );
                            }
                            *result
                        }
                        // -- PLAN-770 text typing --------------------------------
                        hir::ExprKind::StrLit { .. } => {
                            // A literal has no named type of its own: the
                            // declared type IS the contextual borrowing or
                            // owning choice (auto-promotion to Str happens
                            // in owning contexts).
                            if !matches!(declared, ValueType::Str | ValueType::StrOwned) {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: string literal must have text type str or Str (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            declared
                        }
                        hir::ExprKind::CharLit { scalar } => {
                            if declared != ValueType::Char {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: char literal must have type char (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            // The binder range-checks the literal; a mutated
                            // programmatic bundle may carry a surrogate or
                            // out-of-range code point — reject it here so no
                            // invalid scalar reaches lowering.
                            if char::from_u32(*scalar).is_none() {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: char literal scalar {:#x} is not a Unicode scalar value",
                                        e.id_text, scalar
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::Char
                        }
                        hir::ExprKind::TextLen { operand }
                        | hir::ExprKind::TextLenBytes { operand } => {
                            let ot = operand_type(
                                tm,
                                *operand,
                                declared_of(&body.exprs[operand.0 as usize]),
                            );
                            if !matches!(ot, ValueType::Str | ValueType::StrOwned) {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: text length operand must be str or Str (found {})",
                                        e.id_text,
                                        vt_name(ot)
                                    ),
                                    e.span,
                                );
                            }
                            if declared != ValueType::I32 {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: text length must produce i32 (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::I32
                        }
                        hir::ExprKind::TextCharAt { operand, index } => {
                            let ot = operand_type(
                                tm,
                                *operand,
                                declared_of(&body.exprs[operand.0 as usize]),
                            );
                            let it = operand_type(
                                tm,
                                *index,
                                declared_of(&body.exprs[index.0 as usize]),
                            );
                            if !matches!(ot, ValueType::Str | ValueType::StrOwned) {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: char_at operand must be str or Str (found {})",
                                        e.id_text,
                                        vt_name(ot)
                                    ),
                                    e.span,
                                );
                            }
                            if it != ValueType::I32 {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: char_at index must be i32 (found {})",
                                        e.id_text,
                                        vt_name(it)
                                    ),
                                    e.span,
                                );
                            }
                            if declared != ValueType::Char {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: char_at must produce char (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::Char
                        }
                        hir::ExprKind::TextSlice {
                            operand,
                            start,
                            end,
                        } => {
                            let ot = operand_type(
                                tm,
                                *operand,
                                declared_of(&body.exprs[operand.0 as usize]),
                            );
                            let st = operand_type(
                                tm,
                                *start,
                                declared_of(&body.exprs[start.0 as usize]),
                            );
                            let et =
                                operand_type(tm, *end, declared_of(&body.exprs[end.0 as usize]));
                            if !matches!(ot, ValueType::Str | ValueType::StrOwned) {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: slice operand must be str or Str (found {})",
                                        e.id_text,
                                        vt_name(ot)
                                    ),
                                    e.span,
                                );
                            }
                            for (role, vt) in [("start", st), ("end", et)] {
                                if vt != ValueType::I32 {
                                    self.error(
                                        "verify.type-mismatch",
                                        format!(
                                            "expr `{}`: slice {} must be i32 (found {})",
                                            e.id_text,
                                            role,
                                            vt_name(vt)
                                        ),
                                        e.span,
                                    );
                                }
                            }
                            if declared != ValueType::Str {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: slice must produce str (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::Str
                        }
                        hir::ExprKind::TextConcat { lhs, rhs } => {
                            let lt =
                                operand_type(tm, *lhs, declared_of(&body.exprs[lhs.0 as usize]));
                            let rt =
                                operand_type(tm, *rhs, declared_of(&body.exprs[rhs.0 as usize]));
                            for (role, vt) in [("lhs", lt), ("rhs", rt)] {
                                if !matches!(vt, ValueType::Str | ValueType::StrOwned) {
                                    self.error(
                                        "verify.type-mismatch",
                                        format!(
                                            "expr `{}`: concat {} must be str or Str (found {})",
                                            e.id_text,
                                            role,
                                            vt_name(vt)
                                        ),
                                        e.span,
                                    );
                                }
                            }
                            if declared != ValueType::StrOwned {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: concat must produce Str (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::StrOwned
                        }
                        hir::ExprKind::TextClone { operand }
                        | hir::ExprKind::TextToOwned { operand } => {
                            let ot = operand_type(
                                tm,
                                *operand,
                                declared_of(&body.exprs[operand.0 as usize]),
                            );
                            if !matches!(ot, ValueType::Str | ValueType::StrOwned) {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: owned conversion operand must be str or Str (found {})",
                                        e.id_text,
                                        vt_name(ot)
                                    ),
                                    e.span,
                                );
                            }
                            if declared != ValueType::StrOwned {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: owned conversion must produce Str (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::StrOwned
                        }
                        hir::ExprKind::TextToBorrowed { operand } => {
                            let ot = operand_type(
                                tm,
                                *operand,
                                declared_of(&body.exprs[operand.0 as usize]),
                            );
                            if !matches!(ot, ValueType::Str | ValueType::StrOwned) {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: borrowed view operand must be str or Str (found {})",
                                        e.id_text,
                                        vt_name(ot)
                                    ),
                                    e.span,
                                );
                            }
                            if declared != ValueType::Str {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: borrowed view must produce str (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::Str
                        }
                        hir::ExprKind::CharToCodepoint { operand } => {
                            let ot = operand_type(
                                tm,
                                *operand,
                                declared_of(&body.exprs[operand.0 as usize]),
                            );
                            if ot != ValueType::Char {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: to_codepoint operand must be char (found {})",
                                        e.id_text,
                                        vt_name(ot)
                                    ),
                                    e.span,
                                );
                            }
                            if declared != ValueType::I32 {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: to_codepoint must produce i32 (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::I32
                        }
                        hir::ExprKind::CharFromCodepoint { operand } => {
                            let ot = operand_type(
                                tm,
                                *operand,
                                declared_of(&body.exprs[operand.0 as usize]),
                            );
                            if ot != ValueType::I32 {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: from_codepoint operand must be i32 (found {})",
                                        e.id_text,
                                        vt_name(ot)
                                    ),
                                    e.span,
                                );
                            }
                            if declared != ValueType::Char {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: from_codepoint must produce char (found {})",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            ValueType::Char
                        }
                        // leaves were fully handled at Enter
                        hir::ExprKind::ReadLocal { .. } | hir::ExprKind::Constant { .. } => {
                            tm.memo.get(&id.0).copied().unwrap_or(declared)
                        }
                    };
                    tm.gray.remove(&id.0);
                    tm.memo.insert(id.0, computed);
                }
            }
        }
        tm.memo
            .get(&eid.0)
            .copied()
            .unwrap_or_else(|| declared_of(&body.exprs[eid.0 as usize]))
    }

    /// Check in-degree == 1 for every expression and reject operand cycles.
    /// Statement positions own one expression each; operand slots
    /// (lhs/rhs/eval_args) own one expression each.
    fn check_expr_graph(&mut self, body: &hir::Body) {
        let mut in_deg: BTreeMap<u32, usize> = BTreeMap::new();
        let mut edges: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for i in 0..body.exprs.len() {
            in_deg.insert(i as u32, 0);
            edges.insert(i as u32, Vec::new());
        }
        for bl in &body.blocks {
            for s in &bl.stmts {
                let owned: Vec<ExprId> = match s {
                    hir::Stmt::Let { value, .. }
                    | hir::Stmt::Return { value }
                    | hir::Stmt::Assign { value, .. } => vec![*value],
                    hir::Stmt::If { cond, .. } => vec![*cond],
                    _ => vec![],
                };
                for v in owned {
                    if let Some(d) = in_deg.get_mut(&v.0) {
                        *d += 1;
                    }
                }
            }
        }
        for (i, e) in body.exprs.iter().enumerate() {
            let mut targets: Vec<u32> = Vec::new();
            match &e.kind {
                hir::ExprKind::Binary { lhs, rhs, .. } => {
                    targets.push(lhs.0);
                    targets.push(rhs.0);
                }
                hir::ExprKind::Unary { operand, .. } => {
                    targets.push(operand.0);
                }
                hir::ExprKind::Logical { lhs, rhs, .. } => {
                    targets.push(lhs.0);
                    targets.push(rhs.0);
                }
                hir::ExprKind::Call { eval_args, .. } => {
                    for a in eval_args {
                        targets.push(a.0);
                    }
                }
                hir::ExprKind::TextLen { operand }
                | hir::ExprKind::TextLenBytes { operand }
                | hir::ExprKind::TextClone { operand }
                | hir::ExprKind::TextToOwned { operand }
                | hir::ExprKind::TextToBorrowed { operand }
                | hir::ExprKind::CharToCodepoint { operand }
                | hir::ExprKind::CharFromCodepoint { operand } => {
                    targets.push(operand.0);
                }
                hir::ExprKind::TextCharAt { operand, index } => {
                    targets.push(operand.0);
                    targets.push(index.0);
                }
                hir::ExprKind::TextSlice {
                    operand,
                    start,
                    end,
                } => {
                    targets.push(operand.0);
                    targets.push(start.0);
                    targets.push(end.0);
                }
                hir::ExprKind::TextConcat { lhs, rhs } => {
                    targets.push(lhs.0);
                    targets.push(rhs.0);
                }
                hir::ExprKind::StrLit { .. } | hir::ExprKind::CharLit { .. } => {}
                _ => {}
            }
            for t in targets {
                if let Some(d) = in_deg.get_mut(&t) {
                    *d += 1;
                }
                edges.get_mut(&(i as u32)).unwrap().push(t);
            }
        }
        for (i, e) in body.exprs.iter().enumerate() {
            match in_deg.get(&(i as u32)).copied().unwrap_or(0) {
                0 => self.error(
                    "verify.eval-position",
                    format!("expr `{}` is never evaluated (no owning position)", e.id_text),
                    e.span,
                ),
                1 => {}
                n => self.error(
                    "verify.eval-position",
                    format!(
                        "expr `{}` is referenced from {} evaluation positions; every expression must have exactly one",
                        e.id_text, n
                    ),
                    e.span,
                ),
            }
        }
        // Cycle detection over operand edges.
        let mut color: BTreeMap<u32, u8> = BTreeMap::new(); // 1=gray 2=black
        for i in 0..body.exprs.len() {
            self.dfs_cycles(i as u32, &edges, &mut color, body);
        }
    }

    fn dfs_cycles(
        &mut self,
        root: u32,
        edges: &BTreeMap<u32, Vec<u32>>,
        color: &mut BTreeMap<u32, u8>,
        body: &hir::Body,
    ) {
        // Iterative DFS (PLAN-745 R3-QA-02): operand chains can be as deep
        // as the input is long. Same gray/black discipline and the same
        // first-detection reporting as the recursive form.
        enum Step {
            Enter(u32),
            Leave(u32),
        }
        let mut stack = vec![Step::Enter(root)];
        while let Some(step) = stack.pop() {
            match step {
                Step::Enter(node) => {
                    match color.get(&node).copied() {
                        Some(1) => {
                            let e = &body.exprs[node as usize];
                            self.error(
                                "verify.expr-cycle",
                                format!(
                                    "expression graph contains a cycle through `{}`",
                                    e.id_text
                                ),
                                e.span,
                            );
                            continue;
                        }
                        Some(_) => continue,
                        None => {}
                    }
                    color.insert(node, 1);
                    stack.push(Step::Leave(node));
                    for t in edges
                        .get(&node)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .rev()
                    {
                        stack.push(Step::Enter(t));
                    }
                }
                Step::Leave(node) => {
                    color.insert(node, 2);
                }
            }
        }
    }

    // -- blocks: structure + dataflow ----------------------------------------

    fn check_blocks(
        &mut self,
        body: &hir::Body,
        fn_result: ValueType,
        type_map: &BTreeMap<u32, ValueType>,
        sigs: &SigTable,
    ) {
        self.check_expr_graph(body);
        let mut tm = TypeMemo::default();
        for i in 0..body.exprs.len() {
            self.expr_type(ExprId(i as u32), body, type_map, sigs, &mut tm);
        }

        // Block containment graph first (QA-01): entry zero in-degree, every
        // other block exactly one, and every block reachable from the entry
        // via an iterative DFS. Structural failures gate the dataflow walk —
        // r1/r2 counted in-degrees only, so a self-referencing entry block
        // overflowed the stack inside walk_block and a disconnected block
        // cycle (each node in-degree 1, reachable from nothing) passed into
        // CheckedModule.
        let structure_ok = self.check_block_graph(body);

        // Dataflow + termination walk. Params start initialized.
        if structure_ok {
            let mut init: Init = BTreeSet::new();
            for (i, l) in body.locals.iter().enumerate() {
                if l.role == hir::LocalRole::Param {
                    init.insert(LocalId(i as u32));
                }
            }
            let mut tm = TypeMemo::default();
            let mut walked: BTreeSet<u32> = BTreeSet::new();
            let (flow, _exit_init) = self.walk_block(
                body.entry,
                body,
                &init,
                fn_result,
                type_map,
                sigs,
                &mut tm,
                &mut walked,
            );
            if flow != Flow::Returned {
                self.error(
                    "verify.no-return-path",
                    format!(
                        "body `{}` has no return path (entry block must return on all paths)",
                        body.id_text
                    ),
                    body.span,
                );
            }
        }
    }

    /// Validate the block containment graph (If then/else and Loop body
    /// edges): ID-range sanity, entry zero in-degree, every other block at
    /// most one parent, and full reachability from the entry via an explicit
    /// stack. The visited set makes the walk terminate on any malformed
    /// input, including cycles. With the in-degree invariants satisfied,
    /// full reachability implies acyclicity: a reachable cycle would give
    /// some cycle node a second parent, or put the entry itself on the
    /// cycle; unreachable cycles (disconnected SCCs) are caught by the
    /// reachability check. Runtime loop repeats, break/continue LoopId
    /// targets and recursive calls are not containment edges and stay
    /// accepted. Returns false when structural errors were recorded; the
    /// caller must not run the dataflow walk.
    fn check_block_graph(&mut self, body: &hir::Body) -> bool {
        let mut out_edges: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        let mut in_deg: BTreeMap<u32, usize> = BTreeMap::new();
        for i in 0..body.blocks.len() {
            out_edges.insert(i as u32, Vec::new());
            in_deg.insert(i as u32, 0);
        }
        let mut push_edge = |from: usize, to: u32| {
            if let Some(v) = out_edges.get_mut(&(from as u32)) {
                v.push(to);
            }
            *in_deg.entry(to).or_insert(0) += 1;
        };
        for (bi, bl) in body.blocks.iter().enumerate() {
            for s in &bl.stmts {
                match s {
                    hir::Stmt::If { then, els, .. } => {
                        push_edge(bi, then.0);
                        if let Some(e) = els {
                            push_edge(bi, e.0);
                        }
                    }
                    hir::Stmt::Loop { body: b, .. } => push_edge(bi, b.0),
                    _ => {}
                }
            }
        }

        let n_blocks = body.blocks.len();
        let mut ok = true;
        for (i, bl) in body.blocks.iter().enumerate() {
            let n = in_deg.get(&(i as u32)).copied().unwrap_or(0);
            if i as u32 == body.entry.0 {
                if n != 0 {
                    self.error(
                        "verify.block-structure",
                        format!(
                            "entry block `{}` is also referenced as a child block; entry role must be unique",
                            bl.id_text
                        ),
                        bl.span,
                    );
                    ok = false;
                }
            } else if n > 1 {
                self.error(
                    "verify.block-structure",
                    format!(
                        "block `{}` has {} incoming references; every block must have exactly one entry role",
                        bl.id_text, n
                    ),
                    bl.span,
                );
                ok = false;
            }
        }

        // Reachability: iterative DFS with an explicit stack and a visited
        // set — in-degree counts alone cannot distinguish "referenced once"
        // from "reachable", and disconnected cycles must not slip through.
        let mut reachable: BTreeSet<u32> = BTreeSet::new();
        if (body.entry.0 as usize) < n_blocks {
            reachable.insert(body.entry.0);
            let mut stack = vec![body.entry.0];
            while let Some(t) = stack.pop() {
                for c in out_edges.get(&t).cloned().unwrap_or_default() {
                    if (c as usize) < n_blocks && reachable.insert(c) {
                        stack.push(c);
                    }
                }
            }
        }
        for (i, bl) in body.blocks.iter().enumerate() {
            if !reachable.contains(&(i as u32)) {
                self.error(
                    "verify.block-structure",
                    format!(
                        "block `{}` is not reachable from the entry block (block graph must be connected and acyclic)",
                        bl.id_text
                    ),
                    bl.span,
                );
                ok = false;
            }
        }
        ok
    }

    /// Walk the block tree: dataflow (initialization) + termination checks.
    /// Fully ITERATIVE (R6: block nesting is unlimited): an explicit frame
    /// stack replaces the recursion at if/loop statements; child results
    /// land in slots and the frame resumes to merge Flow/Init exactly like
    /// the recursive form. LIFO guarantees a resumed frame's awaited slots
    /// are filled (children complete above their parent). Returns the entry
    /// block's exit flow and exit initialization set.
    #[allow(clippy::too_many_arguments)]
    fn walk_block(
        &mut self,
        entry: crate::hir::BlockId,
        body: &hir::Body,
        init: &Init,
        fn_result: ValueType,
        type_map: &BTreeMap<u32, ValueType>,
        sigs: &SigTable,
        tm: &mut TypeMemo,
        walked: &mut BTreeSet<u32>,
    ) -> (Flow, Init) {
        struct WFrame {
            bid: u32,
            /// Length of the shared loop stack when this frame opened (the
            /// frame's own loop id — for a block containing a loop stmt —
            /// lives above this mark only while the body walks).
            loops_len: usize,
            idx: usize,
            cur: Init,
            flow: Flow,
            ret_slot: usize,
            state: WState,
        }
        enum WState {
            Run,
            AwaitIf {
                then_slot: usize,
                /// `usize::MAX` when the if has no else arm.
                else_slot: usize,
            },
            AwaitLoop {
                body_slot: usize,
            },
        }
        enum Action {
            /// Run this statement (the frame cursor already advanced).
            Stmt { bid: u32, stmt: hir::Stmt },
            /// A terminator precedes remaining statements: structure error.
            AfterTerm,
            /// The frame ran out of statements.
            Complete,
            /// Children finished; merge their results.
            Resume(WState),
        }
        let mut loops: Vec<LoopId> = Vec::new();
        let mut results: Vec<Option<(Flow, Init)>> = vec![None];
        let mut frames: Vec<WFrame> = Vec::new();
        // Defensive cycle guard (QA-01): check_block_graph gates the walk on
        // a valid containment graph, so a block can never legitimately be
        // walked twice; treat a repeat as unreachable and bail out instead
        // of looping forever if that invariant is ever broken.
        if !walked.insert(entry.0) {
            return (Flow::Continue, init.clone());
        }
        frames.push(WFrame {
            bid: entry.0,
            loops_len: 0,
            idx: 0,
            cur: init.clone(),
            flow: Flow::Continue,
            ret_slot: 0,
            state: WState::Run,
        });
        while !frames.is_empty() {
            let action = {
                let frame = frames.last_mut().expect("walk frame");
                if let WState::Run = frame.state {
                    let bid = frame.bid;
                    if frame.idx >= body.blocks[bid as usize].stmts.len() {
                        Action::Complete
                    } else if frame.flow != Flow::Continue {
                        Action::AfterTerm
                    } else {
                        let stmt = body.blocks[bid as usize].stmts[frame.idx].clone();
                        frame.idx += 1;
                        Action::Stmt { bid, stmt }
                    }
                } else {
                    Action::Resume(std::mem::replace(&mut frame.state, WState::Run))
                }
            };
            // Push a child block frame (with its incoming init); a repeat
            // visit feeds the slot the defensive Continue result instead.
            fn push_child(
                frames: &mut Vec<WFrame>,
                results: &mut Vec<Option<(Flow, Init)>>,
                walked: &mut BTreeSet<u32>,
                bid: crate::hir::BlockId,
                loops_len: usize,
                init_in: Init,
            ) -> usize {
                let slot = results.len();
                results.push(None);
                if walked.insert(bid.0) {
                    frames.push(WFrame {
                        bid: bid.0,
                        loops_len,
                        idx: 0,
                        cur: init_in,
                        flow: Flow::Continue,
                        ret_slot: slot,
                        state: WState::Run,
                    });
                } else {
                    results[slot] = Some((Flow::Continue, init_in));
                }
                slot
            }
            match action {
                Action::Stmt { bid, stmt } => {
                    let frame = frames.last_mut().expect("walk frame");
                    match stmt {
                        hir::Stmt::Let { local, value } => {
                            self.walk_value(value, body, &frame.cur);
                            frame.cur.insert(local);
                        }
                        hir::Stmt::Return { value } => {
                            self.walk_value(value, body, &frame.cur);
                            let vt = self.expr_type(value, body, type_map, sigs, tm);
                            if vt != fn_result {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "return value type does not match function result in body `{}`",
                                        body.blocks[bid as usize].id_text
                                    ),
                                    body.exprs[value.0 as usize].span,
                                );
                            }
                            frame.flow = Flow::Returned;
                        }
                        hir::Stmt::If { cond, then, els } => {
                            self.walk_value(cond, body, &frame.cur);
                            let cond_ty = self.expr_type(cond, body, type_map, sigs, tm);
                            if cond_ty != ValueType::Bool {
                                self.error(
                                    "verify.type-mismatch",
                                    "if condition must be bool".to_string(),
                                    body.blocks[bid as usize].span,
                                );
                            }
                            let cur_in = frame.cur.clone();
                            let loops_len = frame.loops_len;
                            let parent_idx = frames.len() - 1;
                            // then-arm walks fully before the else-arm: push
                            // the else child first so it pops second.
                            let else_slot = match els {
                                Some(e) => push_child(
                                    &mut frames,
                                    &mut results,
                                    walked,
                                    e,
                                    loops_len,
                                    cur_in.clone(),
                                ),
                                None => usize::MAX,
                            };
                            let then_slot = push_child(
                                &mut frames,
                                &mut results,
                                walked,
                                then,
                                loops_len,
                                cur_in,
                            );
                            frames[parent_idx].state = WState::AwaitIf {
                                then_slot,
                                else_slot,
                            };
                        }
                        hir::Stmt::Loop { id, body: b } => {
                            let cur_in = frame.cur.clone();
                            let parent_idx = frames.len() - 1;
                            loops.push(id);
                            let body_slot = push_child(
                                &mut frames,
                                &mut results,
                                walked,
                                b,
                                loops.len(),
                                cur_in,
                            );
                            frames[parent_idx].state = WState::AwaitLoop { body_slot };
                        }
                        hir::Stmt::Assign { place, value } => {
                            self.walk_value(value, body, &frame.cur);
                            if let Some(p) = body.places.get(place.0 as usize) {
                                if p.access != hir::Access::Mutable {
                                    self.error(
                                        "verify.immutable-write",
                                        format!("place `{}` is not writable", p.id_text),
                                        p.span,
                                    );
                                }
                                let target = &body.locals[p.local.0 as usize];
                                if !target.mutable {
                                    self.error(
                                        "verify.immutable-write",
                                        format!(
                                            "local `{}` is immutable and cannot be assigned",
                                            target.id_text
                                        ),
                                        p.span,
                                    );
                                }
                                if !frame.cur.contains(&p.local) {
                                    self.error(
                                        "verify.uninitialized-read",
                                        format!(
                                            "assignment to local `{}` before initialization",
                                            target.id_text
                                        ),
                                        p.span,
                                    );
                                }
                            }
                        }
                        hir::Stmt::Break { target } => {
                            if !loops.contains(&target) {
                                self.error(
                                    "verify.loop-target-scope",
                                    "break target is not a loop enclosing this block".to_string(),
                                    body.blocks[bid as usize].span,
                                );
                            }
                            frame.flow = Flow::Diverged;
                        }
                        hir::Stmt::Continue { target } => {
                            if !loops.contains(&target) {
                                self.error(
                                    "verify.loop-target-scope",
                                    "continue target is not a loop enclosing this block"
                                        .to_string(),
                                    body.blocks[bid as usize].span,
                                );
                            }
                            frame.flow = Flow::Diverged;
                        }
                        // Explicit drops are not terminators and read no
                        // uninitialized value: their resource semantics are
                        // checked by the text resource walk (PLAN-770).
                        hir::Stmt::Drop { .. } => {}
                    }
                }
                Action::AfterTerm | Action::Complete => {
                    let frame = frames.pop().expect("walk frame");
                    if matches!(action, Action::AfterTerm) {
                        let bl = &body.blocks[frame.bid as usize];
                        self.error(
                            "verify.block-structure",
                            format!(
                                "block `{}` has executable statements after a terminating statement",
                                bl.id_text
                            ),
                            bl.span,
                        );
                    }
                    results[frame.ret_slot] = Some((frame.flow, frame.cur));
                    loops.truncate(frame.loops_len);
                }
                Action::Resume(state) => {
                    let frame = frames.last_mut().expect("walk frame");
                    match state {
                        WState::AwaitIf {
                            then_slot,
                            else_slot,
                        } => {
                            let (then_f, then_init) =
                                results[then_slot].clone().expect("then result");
                            let (else_f, else_init) = if else_slot == usize::MAX {
                                (Flow::Continue, frame.cur.clone())
                            } else {
                                results[else_slot].clone().expect("else result")
                            };
                            frame.flow = match (then_f, else_f) {
                                (Flow::Returned, Flow::Returned) => Flow::Returned,
                                (Flow::Diverged, Flow::Diverged) => Flow::Diverged,
                                _ => Flow::Continue,
                            };
                            // Exit initialization = intersection over reachable
                            // (continuing) branch exits. A branch that returned
                            // or diverged contributes nothing.
                            if frame.flow == Flow::Continue {
                                let mut merged: Init = BTreeSet::new();
                                if then_f == Flow::Continue && else_f == Flow::Continue {
                                    for id in &then_init {
                                        if else_init.contains(id) {
                                            merged.insert(*id);
                                        }
                                    }
                                } else if then_f == Flow::Continue {
                                    merged = then_init;
                                } else if else_f == Flow::Continue {
                                    merged = else_init;
                                } else {
                                    merged = frame.cur.clone();
                                }
                                frame.cur = merged;
                            }
                        }
                        WState::AwaitLoop { body_slot } => {
                            let (body_f, _body_init) =
                                results[body_slot].clone().expect("loop body result");
                            // A `return` inside the loop body leaves the loop;
                            // a body that completes or diverges via
                            // break/continue hands control back to the loop /
                            // after-loop code.
                            frame.flow = if body_f == Flow::Returned {
                                Flow::Returned
                            } else {
                                Flow::Continue
                            };
                            // The loop id's scope was the body walk alone:
                            // statements after the loop must not target it.
                            loops.truncate(frame.loops_len);
                            // Loops may run zero times: initializations inside
                            // the body do not escape; `cur` stays as-is.
                        }
                        WState::Run => unreachable!("Run is not resumed"),
                    }
                }
            }
        }
        // A block that completes without a terminator simply hands control
        // to its enclosing structure (after-if join, loop repeat, or — at
        // the entry — a missing return path, reported by the caller).
        results[0]
            .clone()
            .unwrap_or_else(|| (Flow::Continue, init.clone()))
    }

    fn walk_value(&mut self, eid: ExprId, body: &hir::Body, init: &Init) {
        let mut visited: BTreeSet<u32> = BTreeSet::new();
        self.walk_value_in(eid, body, init, &mut visited);
    }

    fn walk_value_in(
        &mut self,
        eid: ExprId,
        body: &hir::Body,
        init: &Init,
        visited: &mut BTreeSet<u32>,
    ) {
        // Iterative traversal (PLAN-745 R3-QA-02): operator chains are
        // unbounded; operands are visited left-to-right exactly like the
        // recursive form (LIFO with reversed pushes).
        let mut stack: Vec<ExprId> = vec![eid];
        while let Some(id) = stack.pop() {
            if !visited.insert(id.0) {
                continue; // cyclic graph; already reported by check_expr_graph
            }
            let e = &body.exprs[id.0 as usize];
            match &e.kind {
                hir::ExprKind::ReadLocal { local } => {
                    if !init.contains(local) {
                        let name = body
                            .locals
                            .get(local.0 as usize)
                            .map(|l| l.id_text.as_str())
                            .unwrap_or("<unknown>");
                        self.error(
                            "verify.uninitialized-read",
                            format!("read of uninitialized local `{}`", name),
                            e.span,
                        );
                    }
                }
                hir::ExprKind::Binary { lhs, rhs, .. } => {
                    stack.push(*rhs);
                    stack.push(*lhs);
                }
                hir::ExprKind::Unary { operand, .. } => {
                    stack.push(*operand);
                }
                hir::ExprKind::Logical { lhs, rhs, .. } => {
                    // both sides statically — a conditionally executed RHS
                    // still has to pass the initialization walk
                    stack.push(*rhs);
                    stack.push(*lhs);
                }
                hir::ExprKind::Call { eval_args, .. } => {
                    for a in eval_args.iter().rev() {
                        stack.push(*a);
                    }
                }
                hir::ExprKind::TextLen { operand }
                | hir::ExprKind::TextLenBytes { operand }
                | hir::ExprKind::TextClone { operand }
                | hir::ExprKind::TextToOwned { operand }
                | hir::ExprKind::TextToBorrowed { operand }
                | hir::ExprKind::CharToCodepoint { operand }
                | hir::ExprKind::CharFromCodepoint { operand } => {
                    stack.push(*operand);
                }
                hir::ExprKind::TextCharAt { operand, index } => {
                    stack.push(*index);
                    stack.push(*operand);
                }
                hir::ExprKind::TextSlice {
                    operand,
                    start,
                    end,
                } => {
                    stack.push(*end);
                    stack.push(*start);
                    stack.push(*operand);
                }
                hir::ExprKind::TextConcat { lhs, rhs } => {
                    stack.push(*rhs);
                    stack.push(*lhs);
                }
                hir::ExprKind::Constant { .. }
                | hir::ExprKind::StrLit { .. }
                | hir::ExprKind::CharLit { .. } => {}
            }
        }
    }
}
