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
/// [`verify`]/[`verify_scalar`]; the backend consumes nothing else.
#[derive(Clone, Debug)]
pub struct CheckedModule {
    bundle: hir::Bundle,
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

/// Verify a bound record-domain bundle (`record-i32-draft`, PLAN-771). The
/// scalar compute surface plus nominal records: construction, nested field
/// paths, view/mut borrows (774 D-04) and type-bound methods.
pub fn verify_record(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    verify_domain(bundle, HirDomain::Record)
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
        Ok(CheckedModule { bundle })
    } else {
        Err(v.diags)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ValueType {
    I32,
    Bool,
    Function,
    /// Nominal record identity (PLAN-771): the module-level `TypeId`.
    /// Records are nominal — two distinct `TypeId`s never unify even with
    /// identical field shapes.
    Record(u32),
}

fn vt_name(v: ValueType) -> &'static str {
    match v {
        ValueType::I32 => "i32",
        ValueType::Bool => "bool",
        ValueType::Function => "function",
        ValueType::Record(_) => "record",
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

    /// Core-domain defense (PLAN-767 §5.1): the shared HIR enums were
    /// widened for the scalar profile; a programmatically mutated core
    /// bundle must still be rejected at the operator set, not silently
    /// reinterpreted under the new semantics.
    ///
    /// Record defense (PLAN-771 §5.4): the same widening argument applies
    /// to the record node kinds — outside the record domain every record
    /// type/expr/place/method form is rejected.
    fn check_profile_ops(&mut self, m: &hir::Module, body: &hir::Body) {
        if self.domain == HirDomain::Core {
            for e in &body.exprs {
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
        }
        if self.domain != HirDomain::Record {
            for t in &m.types {
                if matches!(t.kind, hir::TypeKind::Record(_)) {
                    self.error(
                        "verify.profile-op",
                        format!(
                            "record type `{}` is not part of the {} profile",
                            t.id_text,
                            self.domain.as_str()
                        ),
                        t.span,
                    );
                }
            }
            for d in &m.defs {
                if matches!(d.kind, hir::DefKind::Method { .. }) {
                    self.error(
                        "verify.profile-op",
                        format!(
                            "method declaration `{}` is not part of the {} profile",
                            d.id_text,
                            self.domain.as_str()
                        ),
                        d.span,
                    );
                }
            }
            for e in &body.exprs {
                let record_kind = match &e.kind {
                    hir::ExprKind::RecordCtor { .. } => Some("record_ctor"),
                    hir::ExprKind::FieldGet { .. } => Some("field_get"),
                    hir::ExprKind::BorrowView { .. } => Some("borrow_view"),
                    hir::ExprKind::BorrowMut { .. } => Some("borrow_mut"),
                    _ => None,
                };
                if let Some(kind) = record_kind {
                    self.error(
                        "verify.profile-op",
                        format!(
                            "`{}` expression `{}` is not part of the {} profile",
                            kind,
                            e.id_text,
                            self.domain.as_str()
                        ),
                        e.span,
                    );
                }
            }
            for p in &body.places {
                if matches!(p.kind, hir::PlaceKind::FieldPath(_)) {
                    self.error(
                        "verify.profile-op",
                        format!(
                            "field-path place `{}` is not part of the {} profile",
                            p.id_text,
                            self.domain.as_str()
                        ),
                        p.span,
                    );
                }
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
                    hir::TypeKind::Function(_) => ValueType::Function,
                    hir::TypeKind::Record(_) => ValueType::Record(i as u32),
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
        let type_map = Self::build_type_map(m);
        if self.domain == HirDomain::Record {
            self.check_record_module(m, &type_map);
        }
        // Function signatures by def index.
        let mut sigs: SigTable = BTreeMap::new();
        for (i, d) in m.defs.iter().enumerate() {
            let ty_id = match &d.kind {
                hir::DefKind::Function { ty, .. }
                | hir::DefKind::Method { ty, .. }
                | hir::DefKind::Intrinsic { ty, .. } => *ty,
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
            let def_body = match &d.kind {
                hir::DefKind::Function { body, .. } | hir::DefKind::Method { body, .. } => *body,
                hir::DefKind::Intrinsic { .. } => continue,
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
            self.check_profile_ops(m, body);
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
            match &t.kind {
                hir::TypeKind::Function(sig) => {
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
                // PLAN-771: record field type refs get the same bounds
                // gate (a mutated bundle must not smuggle an out-of-range
                // field type into the layout walk).
                hir::TypeKind::Record(shape) => {
                    for f in &shape.fields {
                        if let hir::TypeUse::Ref(tid) = &f.ty {
                            if (tid.0 as usize) >= n_types {
                                diags.push(Diagnostic::new(
                                    Stage::Verify,
                                    "verify.bad-reference",
                                    format!(
                                        "type index `{}` in record type `{}`'s field `{}` is out of range for the module's type table (length {})",
                                        tid.0, t.id_text, f.name, n_types
                                    ),
                                    t.span,
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        let ok = diags.is_empty();
        self.diags.extend(diags);
        ok
    }

    // -- record-domain rules (PLAN-771) --------------------------------------

    /// Whether a local may be written (directly or through a borrow): a
    /// mutable binding (`var`), or an explicit `mut` param (PLAN-771; the
    /// mutable bit is schema-forbidden on params, mode carries it). For the
    /// core/scalar domains param_mode is always View, so behavior there is
    /// unchanged.
    fn local_writable(l: &hir::LocalDef) -> bool {
        l.mutable || (l.role == hir::LocalRole::Param && l.param_mode == hir::ParamMode::Mut)
    }

    /// Record shape of a module type, if it is one.
    fn record_shape(m: &hir::Module, tid: u32) -> Option<&hir::RecordShape> {
        match m.types.get(tid as usize)?.kind {
            hir::TypeKind::Record(ref s) => Some(s),
            _ => None,
        }
    }

    /// Module-level record checks: field well-formedness (non-empty, unique
    /// names, closed scalar/record field types), the type containment graph
    /// (acyclic — iterative worklist, PLAN-771 probe P1 shape; no recursion
    /// anywhere on this path), and method ownership/receiver contracts.
    fn check_record_module(&mut self, m: &hir::Module, type_map: &BTreeMap<u32, ValueType>) {
        let n = m.types.len();
        // 1. per-record well-formedness
        for t in &m.types {
            let hir::TypeKind::Record(shape) = &t.kind else {
                continue;
            };
            if shape.fields.is_empty() {
                self.error(
                    "verify.record-empty",
                    format!(
                        "record type `{}` has no fields; empty records are not part of the record-i32-draft profile",
                        t.id_text
                    ),
                    t.span,
                );
            }
            let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
            for f in &shape.fields {
                if seen.insert(f.name.as_str(), ()).is_some() {
                    self.error(
                        "verify.duplicate-decl",
                        format!(
                            "record type `{}` declares field `{}` more than once",
                            t.id_text, f.name
                        ),
                        t.span,
                    );
                }
                let vt = match &f.ty {
                    hir::TypeUse::Builtin(b) => match b {
                        hir::BuiltinType::I32 => ValueType::I32,
                        hir::BuiltinType::Bool => ValueType::Bool,
                    },
                    hir::TypeUse::Ref(tid) => {
                        match type_map.get(&tid.0) {
                            Some(v) => *v,
                            None => continue, // out-of-range already reported
                        }
                    }
                };
                if matches!(vt, ValueType::Function) {
                    self.error(
                        "verify.type-mismatch",
                        format!(
                            "record type `{}`'s field `{}` has a function type; record fields are i32/bool/nested records only",
                            t.id_text, f.name
                        ),
                        t.span,
                    );
                }
            }
        }

        // 2. containment graph: iterate record->record field edges with an
        // explicit worklist; self loops, mutual loops and disconnected
        // loops are all rejected (probe P1's four shapes).
        let mut mark = vec![0u8; n]; // 0 new, 1 on stack, 2 done
        for start in 0..n {
            if mark[start] != 0 || Self::record_shape(m, start as u32).is_none() {
                continue;
            }
            let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
            mark[start] = 1;
            while let Some(&(node, fi)) = stack.last() {
                let Some(shape) = Self::record_shape(m, node as u32) else {
                    break;
                };
                if fi < shape.fields.len() {
                    stack.last_mut().unwrap().1 += 1;
                    if let hir::TypeUse::Ref(tid) = shape.fields[fi].ty {
                        if Self::record_shape(m, tid.0).is_some() {
                            match mark[tid.0 as usize] {
                                0 => {
                                    mark[tid.0 as usize] = 1;
                                    stack.push((tid.0 as usize, 0));
                                }
                                1 => {
                                    let pos = stack
                                        .iter()
                                        .position(|&(x, _)| x == tid.0 as usize)
                                        .unwrap_or(0);
                                    let chain: Vec<String> = stack[pos..]
                                        .iter()
                                        .map(|&(x, _)| m.types[x].id_text.clone())
                                        .chain(std::iter::once(
                                            m.types[tid.0 as usize].id_text.clone(),
                                        ))
                                        .collect();
                                    self.error(
                                        "verify.record-cycle",
                                        format!(
                                            "record containment graph has a cycle: {}",
                                            chain.join(" -> ")
                                        ),
                                        m.types[tid.0 as usize].span,
                                    );
                                    // keep walking to collect further cycles
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    mark[node] = 2;
                    stack.pop();
                }
            }
        }

        // 3. methods: owner kind, receiver contract, per-record unique names
        let mut method_names: BTreeMap<(u32, &str), ()> = BTreeMap::new();
        for d in &m.defs {
            let hir::DefKind::Method {
                name,
                ty,
                record,
                receiver,
                ..
            } = &d.kind
            else {
                continue;
            };
            if Self::record_shape(m, record.0).is_none() {
                self.error(
                    "verify.type-mismatch",
                    format!(
                        "method `{}`'s record `{}` is not a record type",
                        name,
                        m.types
                            .get(record.0 as usize)
                            .map(|t| t.id_text.as_str())
                            .unwrap_or("<out of range>")
                    ),
                    d.span,
                );
            } else if method_names.insert((record.0, name.as_str()), ()).is_some() {
                self.error(
                    "verify.duplicate-decl",
                    format!(
                        "record `{}` declares method `{}` more than once",
                        m.types[record.0 as usize].id_text, name
                    ),
                    d.span,
                );
            }
            let Some(hir::TypeKind::Function(sig)) = m.types.get(ty.0 as usize).map(|t| &t.kind)
            else {
                continue; // callee-type error already recorded
            };
            match receiver {
                hir::ReceiverKind::View => {
                    if sig.params.is_empty() {
                        self.error(
                            "verify.type-mismatch",
                            format!(
                                "instance method `{}` has no receiver param; a view receiver is params[0] of the owner record type",
                                name
                            ),
                            d.span,
                        );
                    } else if sig.params[0] != hir::TypeUse::Ref(*record) {
                        self.error(
                            "verify.type-mismatch",
                            format!(
                                "instance method `{}`'s receiver param must be its owner record type",
                                name
                            ),
                            d.span,
                        );
                    }
                }
                hir::ReceiverKind::Static => {}
            }
        }
    }

    /// Resolve a field path against the module's record shapes, checking
    /// every segment's owner. Returns the final TypeUse and its computed
    /// value type; scalar-mid-path continuation (a segment through an
    /// i32/bool field) is `verify.field-owner`.
    #[allow(clippy::too_many_arguments)]
    fn resolve_field_path(
        &mut self,
        m: &hir::Module,
        type_map: &BTreeMap<u32, ValueType>,
        root: hir::TypeUse,
        path: &[hir::FieldId],
        subject: &str,
        span: Span,
    ) -> Option<(hir::TypeUse, ValueType)> {
        let mut cur_use = root;
        for (i, seg) in path.iter().enumerate() {
            let cur_vt = match &cur_use {
                hir::TypeUse::Builtin(b) => match b {
                    hir::BuiltinType::I32 => ValueType::I32,
                    hir::BuiltinType::Bool => ValueType::Bool,
                },
                hir::TypeUse::Ref(t) => *type_map.get(&t.0)?,
            };
            let ValueType::Record(owner) = cur_vt else {
                self.error(
                    "verify.field-owner",
                    format!(
                        "{}: path segment {} goes through the non-record type `{}`",
                        subject,
                        i,
                        vt_name(cur_vt)
                    ),
                    span,
                );
                return None;
            };
            let shape = Self::record_shape(m, owner)?;
            if (seg.0 as usize) >= shape.fields.len() {
                self.error(
                    "verify.field-owner",
                    format!(
                        "{}: field index {} is out of range for record `{}` ({} fields)",
                        subject,
                        seg.0,
                        m.types[owner as usize].id_text,
                        shape.fields.len()
                    ),
                    span,
                );
                return None;
            }
            cur_use = shape.fields[seg.0 as usize].ty;
        }
        let final_vt = match &cur_use {
            hir::TypeUse::Builtin(b) => match b {
                hir::BuiltinType::I32 => ValueType::I32,
                hir::BuiltinType::Bool => ValueType::Bool,
            },
            hir::TypeUse::Ref(t) => *type_map.get(&t.0)?,
        };
        Some((cur_use, final_vt))
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
        let (def_body, def_is_fn) = match &def.kind {
            hir::DefKind::Function { body, .. } | hir::DefKind::Method { body, .. } => {
                (*body, true)
            }
            hir::DefKind::Intrinsic { .. } => (hir::BodyId(0), false),
        };
        if !def_is_fn {
            self.error(
                "verify.owner-mismatch",
                format!(
                    "body `{}` is owned by `{}`, which is not a function",
                    body.id_text, def.id_text
                ),
                body.span,
            );
            return;
        }
        if def_body != body_id {
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
        if self.domain == HirDomain::Record {
            self.check_record_body_rules(m, body, type_map);
        }
        self.check_params(body, sig_params, type_map);
        self.check_blocks(m, body, *fn_result, type_map, sigs);
    }

    /// Record-domain body rules (PLAN-771 §5.4; 774 D-04): borrows live
    /// only as direct call arguments, call-site borrow modes match the
    /// callee's param modes, at most one active borrow per object per call
    /// (exclusive mut), and field-path places resolve with exact types.
    fn check_record_body_rules(
        &mut self,
        m: &hir::Module,
        body: &hir::Body,
        type_map: &BTreeMap<u32, ValueType>,
    ) {
        // Instance-method receiver local must be a view param (mut
        // receivers = `mut fn` are not in this batch).
        if let Some(def) = m.defs.get(body.owner.0 as usize) {
            if let hir::DefKind::Method { receiver, .. } = &def.kind {
                if *receiver == hir::ReceiverKind::View {
                    for l in &body.locals {
                        if l.role == hir::LocalRole::Param && l.index == Some(0) {
                            if l.param_mode != hir::ParamMode::View {
                                self.error(
                                    "verify.immutable-write",
                                    format!(
                                        "receiver local `{}` of an instance method is a read-only view; a mut receiver is not part of this profile",
                                        l.id_text
                                    ),
                                    l.span,
                                );
                            }
                        }
                    }
                }
            }
        }

        // Direct-call-arg membership for the borrow-position rule: the
        // exprs that appear as a Call's own eval_args.
        let mut direct_call_arg: BTreeSet<u32> = BTreeSet::new();
        for e in &body.exprs {
            let mut children: Vec<u32> = Vec::new();
            let mut is_call = false;
            match &e.kind {
                hir::ExprKind::Call { eval_args, .. } => {
                    is_call = true;
                    children.extend(eval_args.iter().map(|a| a.0));
                }
                hir::ExprKind::RecordCtor { eval_args, .. } => {
                    children.extend(eval_args.iter().map(|a| a.0));
                }
                hir::ExprKind::Binary { lhs, rhs, .. }
                | hir::ExprKind::Logical { lhs, rhs, .. } => {
                    children.push(lhs.0);
                    children.push(rhs.0);
                }
                hir::ExprKind::Unary { operand, .. } => children.push(operand.0),
                hir::ExprKind::FieldGet { base, .. } => children.push(base.0),
                hir::ExprKind::BorrowView { base } => children.push(base.0),
                hir::ExprKind::ReadLocal { .. } | hir::ExprKind::Constant { .. } => {}
                hir::ExprKind::BorrowMut { .. } => {}
            }
            if is_call {
                direct_call_arg.extend(children);
            }
        }
        // A. borrow exprs must be direct args of a call — never statement
        // roots, operator operands, ctor args or nested positions.
        for (i, e) in body.exprs.iter().enumerate() {
            let is_borrow = matches!(
                e.kind,
                hir::ExprKind::BorrowView { .. } | hir::ExprKind::BorrowMut { .. }
            );
            if is_borrow && !direct_call_arg.contains(&(i as u32)) {
                self.error(
                    "verify.eval-position",
                    format!(
                        "expr `{}`: a borrow expression is only legal as a direct call argument",
                        e.id_text
                    ),
                    e.span,
                );
            }
        }

        // B+C. per call: mode matching + exclusivity; plus places.
        for e in body.exprs.iter() {
            let hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } = &e.kind
            else {
                continue;
            };
            let Some(def) = m.defs.get(callee.0 as usize) else {
                continue;
            };
            let ty_id = match def.kind.fn_type() {
                Some(t) => t,
                None => continue,
            };
            let Some(hir::TypeKind::Function(sig)) = m.types.get(ty_id.0 as usize).map(|t| &t.kind)
            else {
                continue;
            };
            // callee param modes come from the callee's own param locals
            let mut modes: Vec<hir::ParamMode> = vec![hir::ParamMode::View; sig.params.len()];
            if let Some(bid) = def.kind.fn_body() {
                if let Some(cb) = m.bodies.get(bid.0 as usize) {
                    for l in &cb.locals {
                        if l.role == hir::LocalRole::Param {
                            if let Some(idx) = l.index {
                                if (idx as usize) < modes.len() {
                                    modes[idx as usize] = l.param_mode;
                                }
                            }
                        }
                    }
                }
            }
            // mode matching through the bindings
            for b in bindings {
                let Some(mode) = modes.get(b.param as usize) else {
                    continue;
                };
                let Some(arg_e) = body.exprs.get(eval_args[b.arg as usize].0 as usize) else {
                    continue;
                };
                let is_record_param = matches!(
                    type_map.get(&match sig.params.get(b.param as usize) {
                        Some(hir::TypeUse::Ref(t)) => t.0,
                        _ => u32::MAX,
                    }),
                    Some(ValueType::Record(_))
                );
                match (&arg_e.kind, *mode, is_record_param) {
                    (hir::ExprKind::BorrowMut { .. }, hir::ParamMode::Mut, true) => {}
                    (hir::ExprKind::BorrowView { .. }, hir::ParamMode::View, true) => {}
                    (hir::ExprKind::BorrowView { .. }, hir::ParamMode::Mut, true) => {
                        self.error(
                            "verify.ownership-required",
                            format!(
                                "expr `{}`: param {} is declared `mut` and requires a `.mut` argument",
                                e.id_text, b.param
                            ),
                            arg_e.span,
                        );
                    }
                    (hir::ExprKind::BorrowMut { .. }, hir::ParamMode::View, true) => {
                        self.error(
                            "verify.ownership-required",
                            format!(
                                "expr `{}`: param {} is a read-only view and does not accept a `.mut` argument",
                                e.id_text, b.param
                            ),
                            arg_e.span,
                        );
                    }
                    (_, _, true) => {
                        self.error(
                            "verify.ownership-required",
                            format!(
                                "expr `{}`: record param {} requires a borrow argument (view or `.mut`)",
                                e.id_text, b.param
                            ),
                            arg_e.span,
                        );
                    }
                    // scalar params: borrow args are typed out by expr_type
                    (_, _, false) => {}
                }
            }
            // exclusivity: count access events per local across the whole
            // combined arg trees of this one call; any local with a mut
            // borrow and more than one event is a conflict
            let mut events: BTreeMap<u32, (usize, bool)> = BTreeMap::new();
            let mut stack: Vec<u32> = eval_args.iter().map(|a| a.0).collect();
            let mut visited: BTreeSet<u32> = BTreeSet::new();
            while let Some(id) = stack.pop() {
                if !visited.insert(id) {
                    continue;
                }
                let arg = &body.exprs[id as usize];
                match &arg.kind {
                    hir::ExprKind::BorrowMut { local } => {
                        let slot = events.entry(local.0).or_insert((0, false));
                        slot.0 += 1;
                        slot.1 = true;
                    }
                    hir::ExprKind::ReadLocal { local } => {
                        let slot = events.entry(local.0).or_insert((0, false));
                        slot.0 += 1;
                    }
                    _ => {
                        let mut children: Vec<u32> = Vec::new();
                        match &arg.kind {
                            hir::ExprKind::Call { eval_args: a2, .. }
                            | hir::ExprKind::RecordCtor { eval_args: a2, .. } => {
                                children.extend(a2.iter().map(|x| x.0));
                            }
                            hir::ExprKind::Binary { lhs, rhs, .. }
                            | hir::ExprKind::Logical { lhs, rhs, .. } => {
                                children.push(lhs.0);
                                children.push(rhs.0);
                            }
                            hir::ExprKind::Unary { operand, .. } => children.push(operand.0),
                            hir::ExprKind::FieldGet { base, .. } => children.push(base.0),
                            hir::ExprKind::BorrowView { base } => children.push(base.0),
                            _ => {}
                        }
                        stack.extend(children);
                    }
                }
            }
            for (local, (count, has_mut)) in events {
                if has_mut && count > 1 {
                    let name = body
                        .locals
                        .get(local as usize)
                        .map(|l| l.id_text.as_str())
                        .unwrap_or("<unknown>");
                    self.error(
                        "verify.mut-conflict",
                        format!(
                            "expr `{}`: local `{}` is mutably borrowed while also accessed {} time(s) in the same call; a mut borrow is exclusive",
                            e.id_text,
                            name,
                            count - 1
                        ),
                        e.span,
                    );
                }
            }
        }

        // places: field-path owner resolution + exact final type
        for p in &body.places {
            if let hir::PlaceKind::FieldPath(path) = &p.kind {
                let root_local = &body.locals[p.local.0 as usize];
                let subject = format!("place `{}`", p.id_text);
                if let Some((_, final_vt)) =
                    self.resolve_field_path(m, type_map, root_local.ty, path, &subject, p.span)
                {
                    let place_vt = self.value_of(&p.ty, type_map);
                    if final_vt != place_vt {
                        self.error(
                            "verify.type-mismatch",
                            format!(
                                "{}: field path type does not match the place's declared type",
                                subject
                            ),
                            p.span,
                        );
                    }
                }
            }
        }
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
                hir::ExprKind::RecordCtor {
                    record, eval_args, ..
                } => {
                    if (record.0 as usize) >= n_types {
                        bad("type", record.0, n_types, e.span);
                    }
                    for a in eval_args {
                        if (a.0 as usize) >= n_exprs {
                            bad("expression", a.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::FieldGet { base, .. } => {
                    if (base.0 as usize) >= n_exprs {
                        bad("expression", base.0, n_exprs, e.span);
                    }
                }
                hir::ExprKind::BorrowView { base } => {
                    if (base.0 as usize) >= n_exprs {
                        bad("expression", base.0, n_exprs, e.span);
                    }
                }
                hir::ExprKind::BorrowMut { local } => {
                    if (local.0 as usize) >= n_locals {
                        bad("local", local.0, n_locals, e.span);
                    }
                }
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
            if lvt == ValueType::Function {
                self.error(
                    "verify.local-type",
                    format!(
                        "local `{}` has a function type; only i32/bool locals exist in this profile",
                        l.id_text
                    ),
                    l.span,
                );
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
        m: &hir::Module,
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
                        hir::ExprKind::RecordCtor { eval_args, .. } => {
                            // source evaluation order: args entered left to
                            // right exactly as written (never field order)
                            stack.push(Visit::Exit(id));
                            for a in eval_args.iter().rev() {
                                stack.push(Visit::Enter(*a));
                            }
                        }
                        hir::ExprKind::FieldGet { base, .. } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*base));
                        }
                        hir::ExprKind::BorrowView { base } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*base));
                        }
                        hir::ExprKind::BorrowMut { local } => {
                            let l = &body.locals[local.0 as usize];
                            let lt = self.value_of(&l.ty, type_map);
                            if !matches!(lt, ValueType::Record(_)) {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: a mut borrow requires a record-typed local",
                                        e.id_text
                                    ),
                                    e.span,
                                );
                            }
                            if !Self::local_writable(l) {
                                self.error(
                                    "verify.immutable-write",
                                    format!(
                                        "expr `{}`: local `{}` is immutable and cannot be mutably borrowed",
                                        e.id_text, l.id_text
                                    ),
                                    e.span,
                                );
                            }
                            tm.gray.remove(&id.0);
                            tm.memo.insert(id.0, lt);
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
                                    if lt != rt
                                        || !matches!(
                                            (lt, rt),
                                            (ValueType::I32, ValueType::I32)
                                                | (ValueType::Bool, ValueType::Bool)
                                        )
                                    {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: {} operands must both be i32 or both be bool (found {} / {})",
                                                e.id_text,
                                                op.as_str(),
                                                vt_name(lt),
                                                vt_name(rt)
                                            ),
                                            e.span,
                                        );
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
                        // leaves were fully handled at Enter
                        hir::ExprKind::ReadLocal { .. } | hir::ExprKind::Constant { .. } => {
                            tm.memo.get(&id.0).copied().unwrap_or(declared)
                        }
                        hir::ExprKind::BorrowMut { .. } => {
                            // fully handled at Enter (leaf)
                            tm.memo.get(&id.0).copied().unwrap_or(declared)
                        }
                        hir::ExprKind::BorrowView { base } => {
                            let bt =
                                operand_type(tm, *base, declared_of(&body.exprs[base.0 as usize]));
                            match bt {
                                ValueType::Record(rec) => {
                                    if declared != ValueType::Record(rec) {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: view borrow type does not match the borrowed record",
                                                e.id_text
                                            ),
                                            e.span,
                                        );
                                    }
                                    ValueType::Record(rec)
                                }
                                ValueType::Function | ValueType::I32 | ValueType::Bool => {
                                    self.error(
                                        "verify.type-mismatch",
                                        format!(
                                            "expr `{}`: a view borrow requires a record-typed value",
                                            e.id_text
                                        ),
                                        e.span,
                                    );
                                    declared
                                }
                            }
                        }
                        hir::ExprKind::FieldGet { base, path } => {
                            let bt =
                                operand_type(tm, *base, declared_of(&body.exprs[base.0 as usize]));
                            if !matches!(bt, ValueType::Record(_)) {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: field access requires a record-typed base",
                                        e.id_text
                                    ),
                                    e.span,
                                );
                                continue;
                            }
                            let root_use = body.exprs[base.0 as usize].ty.clone();
                            match self.resolve_field_path(
                                m,
                                type_map,
                                root_use,
                                path,
                                &format!("expr `{}`", e.id_text),
                                e.span,
                            ) {
                                Some((_, final_vt)) => {
                                    if final_vt != declared {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: field path type does not match the accessed field",
                                                e.id_text
                                            ),
                                            e.span,
                                        );
                                    }
                                    final_vt
                                }
                                None => declared,
                            }
                        }
                        hir::ExprKind::RecordCtor {
                            record,
                            eval_args,
                            bindings,
                        } => {
                            let rec_vt = ValueType::Record(record.0);
                            if declared != rec_vt {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "expr `{}`: constructor type must be the constructed record",
                                        e.id_text
                                    ),
                                    e.span,
                                );
                            }
                            let shape = match Self::record_shape(m, record.0) {
                                Some(sh) => sh,
                                None => {
                                    self.error(
                                        "verify.type-mismatch",
                                        format!(
                                            "expr `{}`: constructor target is not a record type",
                                            e.id_text
                                        ),
                                        e.span,
                                    );
                                    return declared;
                                }
                            };
                            // every field initialized exactly once, each arg
                            // consumed exactly once (mirrors the call
                            // binding discipline)
                            let mut seen_fields: BTreeSet<u32> = BTreeSet::new();
                            let mut seen_args: BTreeSet<u32> = BTreeSet::new();
                            for b in bindings {
                                if !seen_fields.insert(b.field) {
                                    self.error(
                                        "verify.ctor-arity",
                                        format!(
                                            "expr `{}`: field {} is initialized more than once",
                                            e.id_text, b.field
                                        ),
                                        e.span,
                                    );
                                }
                                if (b.field as usize) >= shape.fields.len() {
                                    self.error(
                                        "verify.ctor-arity",
                                        format!(
                                            "expr `{}`: field index {} is out of range for record `{}` ({} fields)",
                                            e.id_text,
                                            b.field,
                                            m.types[record.0 as usize].id_text,
                                            shape.fields.len()
                                        ),
                                        e.span,
                                    );
                                }
                                if (b.arg as usize) >= eval_args.len() {
                                    self.error(
                                        "verify.ctor-arity",
                                        format!(
                                            "expr `{}`: ctor binding arg {} indexes past {} eval_args",
                                            e.id_text,
                                            b.arg,
                                            eval_args.len()
                                        ),
                                        e.span,
                                    );
                                } else if !seen_args.insert(b.arg) {
                                    self.error(
                                        "verify.ctor-arity",
                                        format!(
                                            "expr `{}`: eval_args[{}] is consumed by more than one field",
                                            e.id_text, b.arg
                                        ),
                                        e.span,
                                    );
                                }
                            }
                            if seen_fields.len() != shape.fields.len() {
                                self.error(
                                    "verify.ctor-arity",
                                    format!(
                                        "expr `{}`: {} of {} fields initialized; every field must be initialized exactly once",
                                        e.id_text,
                                        seen_fields.len(),
                                        shape.fields.len()
                                    ),
                                    e.span,
                                );
                            }
                            for i in 0..eval_args.len() as u32 {
                                if !seen_args.contains(&i) {
                                    self.error(
                                        "verify.ctor-arity",
                                        format!(
                                            "expr `{}`: eval_args[{}] is evaluated but no field consumes it",
                                            e.id_text, i
                                        ),
                                        e.span,
                                    );
                                }
                            }
                            // argument types against field types (nominal:
                            // Record(tid) equality)
                            for b in bindings {
                                if let (Some(field), Some(arg_id)) = (
                                    shape.fields.get(b.field as usize),
                                    eval_args.get(b.arg as usize),
                                ) {
                                    let arg_e = &body.exprs[arg_id.0 as usize];
                                    let at = operand_type(tm, *arg_id, declared_of(arg_e));
                                    let ft = match &field.ty {
                                        hir::TypeUse::Builtin(b2) => match b2 {
                                            hir::BuiltinType::I32 => ValueType::I32,
                                            hir::BuiltinType::Bool => ValueType::Bool,
                                        },
                                        hir::TypeUse::Ref(t2) => match type_map.get(&t2.0) {
                                            Some(v) => *v,
                                            None => continue,
                                        },
                                    };
                                    if at != ft {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: field `{}` ({}) does not match eval_args[{}] ({})",
                                                e.id_text,
                                                field.name,
                                                vt_name(ft),
                                                b.arg,
                                                vt_name(at)
                                            ),
                                            e.span,
                                        );
                                    }
                                }
                            }
                            rec_vt
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
                hir::ExprKind::RecordCtor { eval_args, .. } => {
                    for a in eval_args {
                        targets.push(a.0);
                    }
                }
                hir::ExprKind::FieldGet { base, .. } => {
                    targets.push(base.0);
                }
                hir::ExprKind::BorrowView { base } => {
                    targets.push(base.0);
                }
                hir::ExprKind::BorrowMut { .. } => {}
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
        m: &hir::Module,
        body: &hir::Body,
        fn_result: ValueType,
        type_map: &BTreeMap<u32, ValueType>,
        sigs: &SigTable,
    ) {
        self.check_expr_graph(body);
        let mut tm = TypeMemo::default();
        for i in 0..body.exprs.len() {
            self.expr_type(m, ExprId(i as u32), body, type_map, sigs, &mut tm);
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
                m,
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
        m: &hir::Module,
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
                            let vt = self.expr_type(m, value, body, type_map, sigs, tm);
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
                            let cond_ty = self.expr_type(m, cond, body, type_map, sigs, tm);
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
                                if !Self::local_writable(target) {
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
                hir::ExprKind::RecordCtor { eval_args, .. } => {
                    for a in eval_args.iter().rev() {
                        stack.push(*a);
                    }
                }
                hir::ExprKind::FieldGet { base, .. } => {
                    stack.push(*base);
                }
                hir::ExprKind::BorrowView { base } => {
                    stack.push(*base);
                }
                hir::ExprKind::BorrowMut { local } => {
                    if !init.contains(local) {
                        let name = body
                            .locals
                            .get(local.0 as usize)
                            .map(|l| l.id_text.as_str())
                            .unwrap_or("<unknown>");
                        self.error(
                            "verify.uninitialized-read",
                            format!("mut borrow of uninitialized local `{}`", name),
                            e.span,
                        );
                    }
                }
                hir::ExprKind::Constant { .. } => {}
            }
        }
    }
}
