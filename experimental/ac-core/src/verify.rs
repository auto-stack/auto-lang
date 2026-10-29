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
    /// Derived text-runtime need (PLAN-770 r2 QA-02): computed by
    /// `verify_text` from the verified content itself.
    runtime_text_needed: bool,
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

    /// Whether the VERIFIED CONTENT actually lowers to text runtime calls
    /// (PLAN-770 r2 QA-02): derived from real operations, independent of
    /// the document's own `requires` declaration. The build gates
    /// (capability provision + runtime contract sidecar) fire on this even
    /// if a declaration were absent — although `verify_text` already
    /// rejects that at the front door.
    pub fn requires_runtime_text(&self) -> bool {
        self.runtime_text_needed
    }

    /// The verifier-computed text resource plan. `None` outside the text
    /// domain; for text modules the lowering reads owned sets and exit free
    /// schedules from here instead of re-analyzing ownership.
    pub fn text_resources(&self) -> Option<&crate::text_resources::TextResourcePlan> {
        self.text_plan.as_ref()
    }
}

/// ONE function body verified against its module's declaration context
/// (PLAN-828 function-level increment). Constructed only by
/// [`verify_function`]: the credential binds the verified body, the owning
/// def and a `context_revision` fingerprint of exactly the context the
/// verification consumed (the module's type table and every def's
/// signature) — a later edit that changes any of those invalidates this
/// credential by fingerprint comparison, not by re-slicing a stale
/// [`CheckedModule`].
#[derive(Clone, Debug)]
pub struct CheckedFunction {
    body: hir::Body,
    def: hir::Def,
    /// Owning module id (the 796 owner axis at the credential level).
    owner: String,
    context_revision: String,
    domain: HirDomain,
    /// Intrinsic bindings this body ACTUALLY references — per-REFERENCE
    /// identity: (def ID TEXT, symbol, observable-effect, shape). The id
    /// TEXT is the stable cross-revision identity — indices shift when a
    /// synth program drops other functions' intrinsics; id texts do not
    /// (F-828-R1-06/R4a).
    intrinsics: Vec<(String, String, bool, String)>,
}

impl CheckedFunction {
    pub fn body(&self) -> &hir::Body {
        &self.body
    }

    pub fn into_body(self) -> hir::Body {
        self.body
    }

    pub fn def(&self) -> &hir::Def {
        &self.def
    }

    pub fn def_id_text(&self) -> &str {
        &self.def.id_text
    }

    /// The owning module id this credential was verified within.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// The per-reference intrinsic bindings this body actually reads.
    pub fn referenced_intrinsics(&self) -> &[(String, String, bool, String)] {
        &self.intrinsics
    }

    pub fn context_revision(&self) -> &str {
        &self.context_revision
    }

    pub fn domain(&self) -> HirDomain {
        self.domain
    }
}

/// Verify exactly one function body of a bound bundle against the module's
/// declaration context (PLAN-828 T-01). The bundle may carry placeholder
/// bodies for other functions — only `def_index`'s body is checked — but
/// every def's SIGNATURE participates in the context, so cross-function
/// facts still come from a fully bound declaration set. Scalar/Core
/// domains only: the text/record domains own cross-body fixpoints and keep
/// whole-module [`CheckedModule`] credentials.
pub fn verify_function(
    bundle: hir::Bundle,
    def_index: usize,
) -> Result<CheckedFunction, Vec<Diagnostic>> {
    if !matches!(bundle.domain, HirDomain::Scalar | HirDomain::Core) {
        return Err(vec![Diagnostic::new(
            Stage::Verify,
            "verify.domain",
            format!(
                "function-level verification is available for the scalar/core domains, not `{}`; use the whole-module verifier of the document's own profile",
                bundle.domain.as_str()
            ),
            Span { start: 0, end: 0 },
        )]);
    }
    if bundle.modules.len() != 1 {
        return Err(vec![Diagnostic::new(
            Stage::Verify,
            "verify.modules",
            format!(
                "function-level verification expects exactly one module, found {}",
                bundle.modules.len()
            ),
            Span { start: 0, end: 0 },
        )]);
    }
    let m = &bundle.modules[0];
    let mut v = Verifier {
        diags: Vec::new(),
        domain: bundle.domain,
    };
    // Context construction mirrors `check_module`: type bounds first, then
    // the text-kind isolation gate, the type map, and the signature table
    // over ALL defs (the context the body is verified against).
    if !v.check_module_type_bounds(m) {
        return Err(v.diags);
    }
    for t in &m.types {
        let text_kind = matches!(
            t.kind,
            hir::TypeKind::Str | hir::TypeKind::StrOwned | hir::TypeKind::Char
        );
        // PLAN-864: the collections domain owns owned-`Str` values (the
        // `t_Str` table entry); `char`/borrowed `str` stay excluded.
        let admitted = matches!(t.kind, hir::TypeKind::StrOwned)
            && matches!(v.domain, HirDomain::Collections | HirDomain::Closures);
        if text_kind && !admitted {
            v.error(
                "verify.local-type",
                format!(
                    "type `{}` has text kind `{:?}`; text types exist only in the text-i32-draft profile",
                    t.id_text, t.kind
                ),
                t.span,
            );
        }
    }
    let type_map = Verifier::build_type_map(m);
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
                    .map(|p| v.value_of(p, &type_map))
                    .collect();
                let result = v.value_of(&sig.result, &type_map);
                sigs.insert(i as u32, (params, result));
            }
            _ => {
                v.error(
                    "verify.callee-type",
                    format!("declaration `{}` does not have a function type", d.id_text),
                    d.span,
                );
            }
        }
    }
    let Some(def) = m.defs.get(def_index) else {
        return Err(vec![Diagnostic::new(
            Stage::Verify,
            "verify.def-index",
            format!("def index {def_index} out of range ({} defs)", m.defs.len()),
            Span { start: 0, end: 0 },
        )]);
    };
    let (body_id, name) = match &def.kind {
        hir::DefKind::Function { name, body, .. } => (*body, name.clone()),
        hir::DefKind::Method { .. } => {
            return Err(vec![Diagnostic::new(
                Stage::Verify,
                "verify.domain",
                format!(
                    "def `{}` is a type-bound method; methods belong to the record domain and are outside function-level scalar verification",
                    def.id_text
                ),
                def.span,
            )]);
        }
        hir::DefKind::Intrinsic { .. } => {
            return Err(vec![Diagnostic::new(
                Stage::Verify,
                "verify.def-kind",
                format!(
                    "def `{}` is an intrinsic and has no body to verify",
                    def.id_text
                ),
                def.span,
            )]);
        }
    };
    let Some(body) = m.bodies.get(body_id.0 as usize) else {
        return Err(vec![Diagnostic::new(
            Stage::Verify,
            "verify.body-ref",
            format!(
                "function `{name}` declares body `{}` which is out of range",
                body_id.0
            ),
            def.span,
        )]);
    };
    // def↔body ownership for THIS def (same rule as the module walk).
    if body.owner.0 as usize != def_index {
        let owner = m
            .defs
            .get(body.owner.0 as usize)
            .map(|o| o.id_text.as_str())
            .unwrap_or("<unknown>");
        return Err(vec![Diagnostic::new(
            Stage::Verify,
            "verify.owner-mismatch",
            format!(
                "function `{name}` declares body `{}` but that body is owned by `{owner}`; every body must belong to exactly one function",
                body.id_text
            ),
            def.span,
        )]);
    }
    v.check_profile_ops(m, body);
    v.check_body(m, body_id, body, &type_map, &sigs);
    if v.diags.is_empty() {
        // Collect the intrinsics THIS body actually calls (body-read
        // facts; content-canonical shapes).
        let mut intrinsics: Vec<(String, String, bool, String)> = Vec::new();
        for e in &body.exprs {
            if let hir::ExprKind::Call { callee, .. } = &e.kind {
                if let Some(d) = m.defs.get(callee.0 as usize) {
                    if let hir::DefKind::Intrinsic { symbol, effect, .. } = &d.kind {
                        let shape = d
                            .kind
                            .fn_type()
                            .and_then(|t| m.types.get(t.0 as usize).cloned())
                            .map(|td| match &td.kind {
                                hir::TypeKind::Function(sig) => {
                                    let params: Vec<String> = sig
                                        .params
                                        .iter()
                                        .map(|p| hir::TypeUse::shape(p, m))
                                        .collect();
                                    format!(
                                        "fn({})->{}",
                                        params.join(","),
                                        hir::TypeUse::shape(&sig.result, m)
                                    )
                                }
                                other => format!("{other:?}"),
                            })
                            .unwrap_or_else(|| "?".to_string());
                        let entry = (
                            d.id_text.clone(),
                            symbol.clone(),
                            matches!(effect, hir::Effect::Observable),
                            shape,
                        );
                        if !intrinsics.contains(&entry) {
                            intrinsics.push(entry);
                        }
                    }
                }
            }
        }
        Ok(CheckedFunction {
            body: body.clone(),
            def: def.clone(),
            owner: m.id_text.clone(),
            context_revision: context_fingerprint(m),
            domain: bundle.domain,
            intrinsics,
        })
    } else {
        Err(v.diags)
    }
}

/// Deterministic FNV-1a fingerprint of the declaration context a function
/// body is verified against: every type (id + kind shape, function
/// signatures recursively) and every def (id + kind + signature type). Two
/// contexts with equal fingerprints provide identical cross-function facts
/// for body verification.
/// The declaration-context fingerprint of a module (the SAME function
/// [`verify_function`] binds into `CheckedFunction`s). Exposed so consumer
/// environments derive their identity from their own bundle instead of
/// trusting a caller-supplied string (F-828-R1-06).
pub fn declaration_context_fingerprint(m: &hir::Module) -> String {
    context_fingerprint(m)
}

fn context_fingerprint(m: &hir::Module) -> String {
    fn mix(h: &mut u64, part: &str) {
        for b in part.bytes() {
            *h ^= b as u64;
            *h = h.wrapping_mul(0x100000001b3);
        }
    }
    // Content-canonical type mixing: type references resolve to their
    // SHAPE, never their table index — two compilations of the same
    // declarations with different type-table layouts (a real body vs a
    // stub program) must produce the SAME fingerprint, or function
    // credentials would falsely invalidate.
    fn mix_use(h: &mut u64, u: &hir::TypeUse, m: &hir::Module, depth: u8) {
        if depth == 0 {
            mix(h, "!");
            return;
        }
        match u {
            hir::TypeUse::Builtin(b) => mix(h, &format!("b:{b:?}")),
            hir::TypeUse::Ref(tid) => match m.types.get(tid.0 as usize).map(|t| &t.kind) {
                Some(hir::TypeKind::Function(sig)) => {
                    mix(h, "fn(");
                    for p in &sig.params {
                        mix_use(h, p, m, depth - 1);
                    }
                    mix(h, ")");
                    mix_use(h, &sig.result, m, depth - 1);
                }
                Some(hir::TypeKind::I32) => mix(h, "i32"),
                Some(hir::TypeKind::Bool) => mix(h, "bool"),
                Some(hir::TypeKind::Str) => mix(h, "str"),
                Some(hir::TypeKind::StrOwned) => mix(h, "Str"),
                Some(hir::TypeKind::Char) => mix(h, "char"),
                Some(hir::TypeKind::Record(_)) => mix(h, "rec"),
                // PLAN-864: List identity is its element; Variant identity is
                // nominal (owner name) plus its arm/payload shape.
                Some(hir::TypeKind::List { elem }) => {
                    mix(h, "list[");
                    mix_use(h, elem, m, depth - 1);
                    mix(h, "]");
                }
                Some(hir::TypeKind::Variant { arms }) => {
                    let name = m
                        .types
                        .get(tid.0 as usize)
                        .map(|t| t.id_text.clone())
                        .unwrap_or_default();
                    mix(h, &format!("variant:{name}("));
                    for a in arms {
                        mix(h, &a.name);
                        mix(h, "(");
                        if let Some(p) = &a.payload {
                            mix_use(h, p, m, depth - 1);
                        }
                        mix(h, ")");
                    }
                    mix(h, ")");
                }
                None => mix(h, "oob"),
            },
        }
    }
    fn mix_fn_type(h: &mut u64, ty: hir::TypeId, m: &hir::Module) {
        match m.types.get(ty.0 as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::Function(sig)) => {
                mix(h, "fn(");
                for p in &sig.params {
                    mix_use(h, p, m, 8);
                }
                mix(h, ")");
                mix_use(h, &sig.result, m, 8);
            }
            other => mix(h, &format!("{other:?}")),
        }
    }
    let mut h: u64 = 0xcbf29ce484222325;
    mix(&mut h, "defs:");
    for d in &m.defs {
        // Intrinsic declarations are excluded from the context identity:
        // they are derived from BODY content (used_intrinsics), and an
        // incremental synth program legitimately carries only the changed
        // function's real body — unchanged functions' intrinsics ride
        // their preserved code units (F-828-R1-08/R2). Function/method
        // declarations (the interface facts every body is verified
        // against) fully participate.
        if matches!(&d.kind, hir::DefKind::Intrinsic { .. }) {
            continue;
        }
        mix(&mut h, &d.id_text);
        match &d.kind {
            hir::DefKind::Function { ty, .. } => {
                mix(&mut h, "f:");
                mix_fn_type(&mut h, *ty, m);
            }
            hir::DefKind::Method { ty, .. } => {
                mix(&mut h, "m:");
                mix_fn_type(&mut h, *ty, m);
            }
            hir::DefKind::Intrinsic { symbol, ty, .. } => {
                mix(&mut h, &format!("i:{symbol}:"));
                mix_fn_type(&mut h, *ty, m);
            }
        }
        mix(&mut h, ";");
    }
    format!("{h:016x}")
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
///
/// Front-door gates (reviews r2 QA-01/QA-02 + r3 R3-04/R3-05): the bundle's
/// OWN identity must be the text domain (a forged Core/Scalar header on
/// text content is a `verify.domain` rejection — it must never reach the
/// scalar lowering's text-unreachable path). The DERIVED runtime need —
/// real text operations AND cleanup obligations (owned locals / exit
/// frees: an emitted function whose only runtime use is `ac_rt_own_free`
/// is gated the same way, r3 R3-04) — is computed AFTER the structural
/// checks and resource plans, so illegal references are controlled
/// `verify.bad-reference` rejections before any need scan runs (r3
/// R3-05), and a module whose content needs the runtime MUST declare
/// `requires: ["ac.runtime.text.v1"]`. The derived need is recorded on
/// the CheckedModule so the build gates (capability + runtime contract)
/// fire on REAL usage, not the caller-controlled declaration alone.
pub fn verify_text(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    if bundle.domain != HirDomain::Text {
        return Err(vec![Diagnostic::new(
            Stage::Verify,
            "verify.domain",
            format!(
                "bundle identity `{}` does not match this verifier entry (`{}`); use the entry of the document's own descriptor profile",
                bundle.domain.as_str(),
                HirDomain::Text.as_str()
            ),
            Span { start: 0, end: 0 },
        )]);
    }
    let mut v = Verifier {
        diags: Vec::new(),
        domain: HirDomain::Text,
    };
    for m in &bundle.modules {
        v.check_module(m);
    }
    let mut plan = crate::text_resources::TextResourcePlan::default();
    if v.diags.is_empty() {
        // r4 R4-05: pairwise parameter exclusivity constraints are computed
        // to a FIXPOINT before the reporting pass — each pass walks every
        // body with the current constraint map, checks callee pairs at its
        // call sites, and discovers constraints for ITS OWN signature; the
        // map grows monotonically (bounded by params²), so the loop
        // terminates and the last (stable) pass carries the diagnostics
        // and plans computed against the final constraints. Earlier passes
        // are seed-only subsets of the final pair set.
        let mut excl = crate::text_resources::seed_param_exclusions(&bundle);
        loop {
            let mut next = excl.clone();
            let mut pass_diags = Vec::new();
            let mut pass_plan = crate::text_resources::TextResourcePlan::default();
            for (mi, m) in bundle.modules.iter().enumerate() {
                let type_map = Verifier::build_type_map(m);
                for (bi, body) in m.bodies.iter().enumerate() {
                    let (bp, adds) = crate::text_resources::check_body(
                        body,
                        m,
                        mi,
                        &type_map,
                        &[],
                        &excl,
                        &mut pass_diags,
                    );
                    if !adds.is_empty() {
                        next.entry((mi, bi as u32)).or_default().extend(adds);
                    }
                    if let Some(bp) = bp {
                        pass_plan.bodies.insert((mi, bi as u32), bp);
                    }
                }
            }
            let stable = next == excl;
            excl = next;
            plan = pass_plan;
            v.diags = pass_diags;
            if stable {
                break;
            }
        }
    }
    if v.diags.is_empty() {
        // r3 R3-04: the derived runtime need covers BOTH real text
        // operations and plan-driven cleanup obligations (ac_rt_own_free).
        // Computed only after structural checks + plans (r3 R3-05: illegal
        // references were already rejected as verify.bad-reference).
        let needs_runtime = crate::text_resources::bundle_needs_text_runtime(&bundle)
            || crate::text_resources::plan_needs_text_runtime(&plan);
        if needs_runtime
            && !bundle
                .requires
                .iter()
                .any(|r| r == crate::text_resources::RUNTIME_TEXT_CAPABILITY)
        {
            return Err(vec![Diagnostic::new(
                Stage::Capability,
                "capability.missing",
                "the module lowers text runtime operations or cleanup obligations but the document does not declare `requires: [\"ac.runtime.text.v1\"]`; the declaration is mandatory, not inferred",
                Span { start: 0, end: 0 },
            )]);
        }
        Ok(CheckedModule {
            bundle,
            text_plan: Some(plan),
            runtime_text_needed: needs_runtime,
        })
    } else {
        Err(v.diags)
    }
}

/// Verify a bound record-domain bundle (`record-i32-draft`, PLAN-771). The
/// scalar compute surface plus nominal records: construction, nested field
/// paths, view/mut borrows (774 D-04) and type-bound methods.
pub fn verify_record(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    verify_domain(bundle, HirDomain::Record)
}

/// Verify a bound text-record-domain bundle (`text-record-i32-draft`,
/// PLAN-782): the union of the text and record surfaces — owned `Str`
/// fields inside nominal records with derived resource classification,
/// explicit aggregate move / deep clone. Structural gates (identity,
/// nominal typing, containment, construction, profile isolation) land in
/// T-02; the full resource walk (ownership liveness, cleanup schedules)
/// extends the PLAN-770 machinery in T-04.
pub fn verify_text_record(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    if bundle.domain != HirDomain::TextRecord {
        return Err(vec![Diagnostic::new(
            Stage::Verify,
            "verify.domain",
            format!(
                "bundle identity `{}` does not match this verifier entry (`{}`); use the entry of the document's own descriptor profile",
                bundle.domain.as_str(),
                HirDomain::TextRecord.as_str()
            ),
            Span { start: 0, end: 0 },
        )]);
    }
    let mut v = Verifier {
        diags: Vec::new(),
        domain: HirDomain::TextRecord,
    };
    for m in &bundle.modules {
        v.check_module(m);
    }
    let mut plan = crate::text_resources::TextResourcePlan::default();
    if v.diags.is_empty() {
        // PLAN-782 T-04: the resource walk (770 machinery extended to
        // record roots). Same exclusivity fixpoint shape as verify_text;
        // the per-module resource classification comes from the shapes
        // (never trusted from the document).
        let mut excl = crate::text_resources::seed_param_exclusions(&bundle);
        loop {
            let mut next = excl.clone();
            let mut pass_diags = Vec::new();
            let mut pass_plan = crate::text_resources::TextResourcePlan::default();
            for (mi, m) in bundle.modules.iter().enumerate() {
                let type_map = Verifier::build_type_map(m);
                let resource = record_resource_map(m);
                for (bi, body) in m.bodies.iter().enumerate() {
                    let (bp, adds) = crate::text_resources::check_body(
                        body,
                        m,
                        mi,
                        &type_map,
                        &resource,
                        &excl,
                        &mut pass_diags,
                    );
                    if !adds.is_empty() {
                        next.entry((mi, bi as u32)).or_default().extend(adds);
                    }
                    if let Some(bp) = bp {
                        pass_plan.bodies.insert((mi, bi as u32), bp);
                    }
                }
            }
            let stable = next == excl;
            excl = next;
            plan = pass_plan;
            v.diags = pass_diags;
            if stable {
                break;
            }
        }
    }
    if v.diags.is_empty() {
        // Derived runtime need: real text operations OR plan-driven
        // cleanup obligations (owned roots and scheduled frees).
        let needs_runtime = crate::text_resources::bundle_needs_text_runtime(&bundle)
            || crate::text_resources::plan_needs_text_runtime(&plan);
        if needs_runtime
            && !bundle
                .requires
                .iter()
                .any(|r| r == crate::text_resources::RUNTIME_TEXT_CAPABILITY)
        {
            return Err(vec![Diagnostic::new(
                Stage::Capability,
                "capability.missing",
                "the module lowers text runtime operations or cleanup obligations but the document does not declare `requires: [\"ac.runtime.text.v1\"]`; the declaration is mandatory, not inferred",
                Span { start: 0, end: 0 },
            )]);
        }
        Ok(CheckedModule {
            bundle,
            text_plan: Some(plan),
            runtime_text_needed: needs_runtime,
        })
    } else {
        Err(v.diags)
    }
}

/// Verify a bound entry-unit bundle (`entry-unit-draft`, PLAN-862): the
/// scalar compute surface plus the ENTRY surface — a `unit` (`void`)
/// function RESULT and the unit value expression a Unit body returns.
///
/// Structural rules are the scalar domain's (a Unit domain must not borrow
/// the text/record relaxations), and this entry adds the two placement
/// rules the plan freezes:
///
/// - **result-only Unit** (`verify.unit-placement`): `unit` may appear as a
///   function RESULT and nowhere else — never as a parameter, a local, a
///   place, a record field or a named type entry;
/// - **unit value pairing** (`verify.unit-expr`, in the expression walk):
///   an `ExprKind::Unit` is only legal in this domain and only with declared
///   type `unit`; a `return` operand must match its function's result, so a
///   Unit body returns the unit value and an `int` body can never return it.
/// PLAN-864: the collections-domain verifier entry. Enforces the shared
/// structural/type/init/return walks plus the collections contracts on the
/// new expression kinds (contract C4/C6/C10): variant dispatch roots are
/// whole-local reads, ListGet never appears over an owned-element list
/// outside a field/view projection (checked here as a structural pattern
/// walk), and owned-terminal field projections are only consumed by
/// `.len()` (the TextLen view shape). Cleanup carries no plan — profile 4
/// releases frame-owned values at exits.
pub fn verify_collections(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    verify_collection_family(bundle, HirDomain::Collections)
}

/// PLAN-865: source closures retain every collections-domain rule while
/// using their own descriptor/verifier identity.
pub fn verify_closures(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    verify_collection_family(bundle, HirDomain::Closures)
}

fn verify_collection_family(
    bundle: hir::Bundle,
    expected: HirDomain,
) -> Result<CheckedModule, Vec<Diagnostic>> {
    if bundle.domain != expected {
        return Err(vec![Diagnostic::new(
            Stage::Verify,
            "verify.domain",
            format!(
                "bundle identity `{}` does not match this verifier entry (`{}`); use the entry of the document's own descriptor profile",
                bundle.domain.as_str(),
                expected.as_str()
            ),
            Span { start: 0, end: 0 },
        )]);
    }
    let mut v = Verifier {
        diags: Vec::new(),
        domain: expected,
    };
    v.check_bundle(&bundle);
    for m in &bundle.modules {
        v.check_module(m);
        v.check_collections_rules(m);
    }
    if v.diags.is_empty() {
        // no text resource plan: profile 4 releases frame-owned values at
        // exits (runtime LIFO sweep), and this domain never emits the
        // ac_rt text runtime.
        Ok(CheckedModule {
            bundle,
            text_plan: None,
            runtime_text_needed: false,
        })
    } else {
        Err(v.diags)
    }
}

pub fn verify_entry_unit(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    let checked = verify_domain(bundle, HirDomain::EntryUnit)?;
    let mut diags = Vec::new();
    for m in &checked.bundle.modules {
        check_unit_placement(m, &mut diags);
    }
    if diags.is_empty() {
        Ok(checked)
    } else {
        Err(diags)
    }
}

/// PLAN-862 placement gate: Unit is a RESULT type only.
fn check_unit_placement(m: &hir::Module, diags: &mut Vec<Diagnostic>) {
    let is_unit = |u: &hir::TypeUse| matches!(u, hir::TypeUse::Builtin(hir::BuiltinType::Unit));
    for t in &m.types {
        match &t.kind {
            hir::TypeKind::Function(sig) => {
                for (i, p) in sig.params.iter().enumerate() {
                    if is_unit(p) {
                        diags.push(Diagnostic::new(
                            Stage::Verify,
                            "verify.unit-placement",
                            format!(
                                "parameter #{} of type `{}` is `unit`; Unit is a RESULT type only",
                                i + 1,
                                t.id_text
                            ),
                            t.span,
                        ));
                    }
                }
            }
            hir::TypeKind::Record(shape) => {
                for f in &shape.fields {
                    if is_unit(&f.ty) {
                        diags.push(Diagnostic::new(
                            Stage::Verify,
                            "verify.unit-placement",
                            format!(
                                "field `{}` of type `{}` is `unit`; Unit cannot be stored",
                                f.name, t.id_text
                            ),
                            t.span,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    for body in &m.bodies {
        for l in &body.locals {
            if is_unit(&l.ty) {
                diags.push(Diagnostic::new(
                    Stage::Verify,
                    "verify.unit-placement",
                    format!(
                        "local `{}` is `unit`; Unit is a RESULT type only",
                        l.id_text
                    ),
                    l.span,
                ));
            }
        }
        for p in &body.places {
            if is_unit(&p.ty) {
                diags.push(Diagnostic::new(
                    Stage::Verify,
                    "verify.unit-placement",
                    format!("place `{}` is `unit`; Unit cannot be stored", p.id_text),
                    p.span,
                ));
            }
        }
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
            runtime_text_needed: false,
        })
    } else {
        Err(v.diags)
    }
}

/// Derived resource classification (PLAN-782): `resource[i]` is true iff
/// the module type `i` is a record from which a `Str` field is transitively
/// reachable through nested record fields. Computed by a fixpoint over the
/// field-type graph (the containment graph is verified acyclic first, so
/// this terminates regardless of input shapes); never trusted from the
/// input. Records that own no text anywhere stay copyable and keep the 771
/// value-copy semantics.
pub fn record_resource_map(m: &hir::Module) -> Vec<bool> {
    let n = m.types.len();
    let mut resource = vec![false; n];
    let mut settled = vec![false; n];
    let mut changed = true;
    while changed {
        changed = false;
        for (i, t) in m.types.iter().enumerate() {
            if settled[i] {
                continue;
            }
            let hir::TypeKind::Record(shape) = &t.kind else {
                settled[i] = true;
                changed = true;
                continue;
            };
            let mut owns = false;
            let mut pending = false;
            for f in &shape.fields {
                match &f.ty {
                    hir::TypeUse::Builtin(hir::BuiltinType::StrOwned)
                    | hir::TypeUse::Builtin(hir::BuiltinType::StrSlice) => {
                        owns = true;
                        break;
                    }
                    hir::TypeUse::Ref(tid) => {
                        let idx = tid.0 as usize;
                        match m.types.get(idx).map(|t| &t.kind) {
                            // a named text type entry carries the same
                            // semantics as the inline builtin use (the
                            // source adapter emits `type: t_Str` refs)
                            Some(hir::TypeKind::StrOwned) | Some(hir::TypeKind::Str) => {
                                owns = true;
                                break;
                            }
                            Some(hir::TypeKind::Record(_)) => {
                                if resource[idx] {
                                    owns = true;
                                    break;
                                }
                                if !settled[idx] {
                                    pending = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if owns {
                resource[i] = true;
                settled[i] = true;
                changed = true;
            } else if !pending {
                settled[i] = true;
                changed = true;
            }
        }
    }
    resource
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
    /// Nominal record identity (PLAN-771): the module-level `TypeId`.
    /// Records are nominal — two distinct `TypeId`s never unify even with
    /// identical field shapes.
    Record(u32),
    /// Unit (PLAN-862): the no-value RESULT type. It types exactly one
    /// expression (ExprKind::Unit) and never a local, place, parameter or
    /// operator operand.
    Unit,
    /// PLAN-864: an opaque composite (List/Variant type-table entry). The
    /// shared record-domain walks treat it as a non-unifiable leaf; the
    /// collections verification entry owns its real rules.
    Opaque,
}

pub(crate) fn vt_name(v: ValueType) -> &'static str {
    match v {
        ValueType::I32 => "i32",
        ValueType::Bool => "bool",
        ValueType::Function => "function",
        ValueType::Str => "str",
        ValueType::StrOwned => "Str",
        ValueType::Char => "char",
        ValueType::Record(_) => "record",
        ValueType::Unit => "unit",
        ValueType::Opaque => "composite",
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

    /// Profile-surface defense (PLAN-767 5.1 + PLAN-770/PLAN-771
    /// isolation): the shared HIR enums were widened by later profiles; a
    /// programmatically mutated bundle of one profile must still be
    /// rejected at its own surface, never silently reinterpreted under
    /// another's semantics. Core additionally rejects every scalar-only
    /// operator; every NON-TEXT domain rejects the entire text surface
    /// (text expression kinds, drop statements, and parameter modes --
    /// `move` is text-only, `mut` is legal only in text and record);
    /// every NON-RECORD domain rejects the record surface (record types,
    /// method declarations, record expressions, field-path places).
    fn check_profile_ops(&mut self, m: &hir::Module, body: &hir::Body) {
        if !matches!(
            self.domain,
            HirDomain::Text | HirDomain::TextRecord | HirDomain::Collections | HirDomain::Closures
        ) {
            for e in &body.exprs {
                // PLAN-864: the collections surface admits exactly the
                // restricted text face: literals, `.len()` and `.move`.
                let text_kind = match self.domain {
                    // PLAN-864: only the non-admitted text face rejects.
                    HirDomain::Collections | HirDomain::Closures => matches!(
                        &e.kind,
                        hir::ExprKind::CharLit { .. }
                            | hir::ExprKind::TextLenBytes { .. }
                            | hir::ExprKind::TextCharAt { .. }
                            | hir::ExprKind::TextSlice { .. }
                            | hir::ExprKind::TextConcat { .. }
                            | hir::ExprKind::TextClone { .. }
                            | hir::ExprKind::TextToOwned { .. }
                            | hir::ExprKind::TextToBorrowed { .. }
                            | hir::ExprKind::CharToCodepoint { .. }
                            | hir::ExprKind::CharFromCodepoint { .. }
                    ),
                    _ => matches!(
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
                            | hir::ExprKind::Move { .. }
                    ),
                };
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
                let bad_mode = match l.param_mode {
                    hir::ParamMode::View => false,
                    hir::ParamMode::MutBorrow => !matches!(
                        self.domain,
                        HirDomain::Text | HirDomain::Record | HirDomain::TextRecord
                    ),
                    hir::ParamMode::MoveOwn => {
                        !matches!(self.domain, HirDomain::Text | HirDomain::TextRecord)
                    }
                };
                if bad_mode {
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
        if !matches!(
            self.domain,
            HirDomain::Record
                | HirDomain::TextRecord
                | HirDomain::Collections
                | HirDomain::Closures
        ) {
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
                let record_kind = match (&e.kind, self.domain) {
                    // PLAN-864: the collections surface admits construction
                    // and field READS (records as elements/locals); borrows
                    // and clone stay on the record/combination surfaces.
                    (
                        hir::ExprKind::BorrowView { .. }
                        | hir::ExprKind::BorrowMut { .. }
                        | hir::ExprKind::RecordClone { .. },
                        HirDomain::Collections | HirDomain::Closures,
                    ) => None,
                    (_, HirDomain::Collections | HirDomain::Closures) => match &e.kind {
                        hir::ExprKind::RecordCtor { .. } => Some("record_ctor"),
                        hir::ExprKind::FieldGet { .. } => Some("field_get"),
                        _ => None,
                    },
                    (_, _) => match &e.kind {
                        hir::ExprKind::RecordCtor { .. } => Some("record_ctor"),
                        hir::ExprKind::FieldGet { .. } => Some("field_get"),
                        hir::ExprKind::BorrowView { .. } => Some("borrow_view"),
                        hir::ExprKind::BorrowMut { .. } => Some("borrow_mut"),
                        hir::ExprKind::RecordClone { .. } => Some("record_clone"),
                        _ => None,
                    },
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
                hir::BuiltinType::StrSlice => ValueType::Str,
                hir::BuiltinType::StrOwned => ValueType::StrOwned,
                hir::BuiltinType::Char => ValueType::Char,
                // PLAN-862: the unit RESULT type.
                hir::BuiltinType::Unit => ValueType::Unit,
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
                    hir::TypeKind::Record(_) => ValueType::Record(i as u32),
                    // PLAN-864: totality for the shared walk; collections
                    // verification gives these types their own rules.
                    hir::TypeKind::List { .. } => ValueType::Opaque,
                    hir::TypeKind::Variant { .. } => ValueType::Opaque,
                };
                (i as u32, vt)
            })
            .collect()
    }

    /// PLAN-864: collections-domain contract checks (contract C4/C6):
    /// - a whole owned element is never a value: ListGet over an
    ///   owned-element list is legal ONLY as a FieldGet-chain base or a
    ///   `.len()` operand (the checked read-only view shapes);
    /// - an owned-terminal field projection is only consumed by `.len()`;
    /// - variant dispatch/project roots are whole-local reads;
    /// - `eval` statement values carry the unit type.
    fn check_collections_rules(&mut self, m: &hir::Module) {
        fn owned_elem(m: &hir::Module, list_ty: &hir::TypeUse, v: &Verifier) -> bool {
            let hir::TypeUse::Ref(tid) = list_ty else {
                return false;
            };
            let Some(hir::TypeKind::List { elem }) = m.types.get(tid.0 as usize).map(|t| &t.kind)
            else {
                return false;
            };
            match elem {
                hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => true,
                hir::TypeUse::Builtin(_) => false,
                hir::TypeUse::Ref(e) => {
                    matches!(
                        m.types.get(e.0 as usize).map(|t| &t.kind),
                        Some(hir::TypeKind::Variant { .. })
                    ) || v.type_reaches_str(m, *e)
                }
            }
        }
        for body in &m.bodies {
            // projection/consumption marks (expr indices)
            let mut projected: BTreeSet<u32> = BTreeSet::new();
            let mut len_consumed: BTreeSet<u32> = BTreeSet::new();
            for e in &body.exprs {
                match &e.kind {
                    hir::ExprKind::FieldGet { base, .. } => {
                        projected.insert(base.0);
                    }
                    hir::ExprKind::TextLen { operand } => {
                        len_consumed.insert(operand.0);
                    }
                    _ => {}
                }
            }
            for (ei, e) in body.exprs.iter().enumerate() {
                match &e.kind {
                    hir::ExprKind::ListGet { list, .. } => {
                        if owned_elem(m, &body.exprs[list.0 as usize].ty, self)
                            && !projected.contains(&(ei as u32))
                            && !len_consumed.contains(&(ei as u32))
                        {
                            self.error(
                                "verify.view-escape",
                                format!(
                                    "expr `{}`: a whole owned element is not a value; project a field or use `remove`",
                                    e.id_text
                                ),
                                e.span,
                            );
                        }
                    }
                    hir::ExprKind::FieldGet { base, path } => {
                        // walk to a ListGet root through FieldGet spines
                        let mut cur = base.0;
                        let mut root_list_ty = None;
                        loop {
                            match &body.exprs[cur as usize].kind {
                                hir::ExprKind::FieldGet { base, .. } => cur = base.0,
                                hir::ExprKind::ListGet { list, .. } => {
                                    root_list_ty = Some(body.exprs[list.0 as usize].ty);
                                    break;
                                }
                                _ => break,
                            }
                        }
                        if let Some(list_ty) = root_list_ty {
                            if owned_elem(m, &list_ty, self) {
                                // terminal field type through the record table
                                let term =
                                    Self::field_terminal(m, &body.exprs[cur as usize].ty, path);
                                let owned_term = match term {
                                    Some(hir::TypeUse::Builtin(hir::BuiltinType::StrOwned)) => true,
                                    Some(hir::TypeUse::Ref(t)) => self.type_reaches_str(m, t),
                                    _ => false,
                                };
                                if owned_term && !len_consumed.contains(&(ei as u32)) {
                                    self.error(
                                        "verify.view-escape",
                                        format!(
                                            "expr `{}`: an owned field view is only consumed by `.len()` in this profile",
                                            e.id_text
                                        ),
                                        e.span,
                                    );
                                }
                            }
                        }
                    }
                    hir::ExprKind::VariantGuardTest { root, .. }
                    | hir::ExprKind::VariantProject { root, .. } => {
                        if !matches!(
                            &body.exprs[root.0 as usize].kind,
                            hir::ExprKind::ReadLocal { .. }
                        ) {
                            self.error(
                                "verify.pattern-scrutinee",
                                format!(
                                    "expr `{}`: variant dispatch roots must be whole-local reads",
                                    e.id_text
                                ),
                                e.span,
                            );
                        }
                    }
                    _ => {}
                }
            }
            for bl in &body.blocks {
                for st in &bl.stmts {
                    if let hir::Stmt::Eval { value } = st {
                        let ve = &body.exprs[value.0 as usize];
                        let unit = matches!(ve.ty, hir::TypeUse::Builtin(hir::BuiltinType::Unit));
                        // PLAN-864: a discarded `remove` is legal when the
                        // element is copyable (the temp releases with the
                        // frame); discarding an OWNED element transfer is
                        // rejected — bind it or drop it explicitly.
                        let copy_remove = match &ve.kind {
                            hir::ExprKind::ListGet { list, .. } => {
                                // the fused shape keeps ListGet at HIR for
                                // owned lists; a bare (unprojected) ListGet
                                // over a COPY-element list is the remove
                                // discard's... no — bare ListGet is not a
                                // statement form. Only real removes count.
                                let _ = list;
                                false
                            }
                            hir::ExprKind::ListRemove { list, .. } => {
                                let lt = &body.exprs[list.0 as usize].ty;
                                !Self::list_elem_owned(m, lt, self)
                            }
                            _ => false,
                        };
                        if !unit && !copy_remove {
                            self.error(
                                "verify.type-mismatch",
                                "an expression statement discards only unit-typed calls (push/insert/set/clear) or a copy-element `remove`".to_string(),
                                bl.span,
                            );
                        }
                    }
                }
            }
        }
    }

    /// PLAN-864: whether a List type's element is owned (Str / record
    /// reaching Str / variant).
    fn list_elem_owned(m: &hir::Module, list_ty: &hir::TypeUse, v: &Verifier) -> bool {
        let hir::TypeUse::Ref(tid) = list_ty else {
            return false;
        };
        let Some(hir::TypeKind::List { elem }) = m.types.get(tid.0 as usize).map(|t| &t.kind)
        else {
            return false;
        };
        match elem {
            hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => true,
            hir::TypeUse::Builtin(_) => false,
            hir::TypeUse::Ref(e) => {
                matches!(
                    m.types.get(e.0 as usize).map(|t| &t.kind),
                    Some(hir::TypeKind::Variant { .. })
                ) || v.type_reaches_str(m, *e)
            }
        }
    }

    /// PLAN-864: transitive Str-reachability over the module type table.
    fn type_reaches_str(&self, m: &hir::Module, t: hir::TypeId) -> bool {
        let mut stack = vec![t.0];
        let mut seen = BTreeSet::new();
        while let Some(i) = stack.pop() {
            if !seen.insert(i) {
                continue;
            }
            match m.types.get(i as usize).map(|x| &x.kind) {
                Some(hir::TypeKind::StrOwned) | Some(hir::TypeKind::Str) => return true,
                Some(hir::TypeKind::Record(shape)) => {
                    for f in &shape.fields {
                        match f.ty {
                            hir::TypeUse::Ref(t2) => stack.push(t2.0),
                            hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => return true,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// PLAN-864: the terminal field type of `path` walked from `base_ty`
    /// through the record table (`None` when a segment does not resolve).
    fn field_terminal(
        m: &hir::Module,
        base_ty: &hir::TypeUse,
        path: &[hir::FieldId],
    ) -> Option<hir::TypeUse> {
        let hir::TypeUse::Ref(tid) = base_ty else {
            return None;
        };
        let mut cur = tid.0;
        for seg in path {
            match m.types.get(cur as usize).map(|t| &t.kind) {
                Some(hir::TypeKind::Record(shape)) => {
                    let f = shape.fields.get(seg.0 as usize)?;
                    match f.ty {
                        hir::TypeUse::Ref(t) => cur = t.0,
                        other => return Some(other),
                    }
                }
                _ => return None,
            }
        }
        Some(hir::TypeUse::Ref(hir::TypeId(cur)))
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
        // Text kinds in the type table exist only in the text profiles
        // (PLAN-770 isolation): a programmatic mutation cannot smuggle
        // str/Str/char named types into an older profile's signatures.
        if !matches!(
            self.domain,
            HirDomain::Text | HirDomain::TextRecord | HirDomain::Collections | HirDomain::Closures
        ) {
            for t in &m.types {
                // PLAN-864: the collections domain owns the named owned-Str
                // entry (`t_Str`); borrowed `str` and `char` stay excluded.
                let admitted = matches!(t.kind, hir::TypeKind::StrOwned)
                    && matches!(self.domain, HirDomain::Collections | HirDomain::Closures);
                if matches!(
                    t.kind,
                    hir::TypeKind::Str | hir::TypeKind::StrOwned | hir::TypeKind::Char
                ) && !admitted
                {
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
        if matches!(self.domain, HirDomain::Record | HirDomain::TextRecord) {
            self.check_record_module(m, &type_map);
        }
        // Derived resource classification (PLAN-782): a record owns buffers
        // iff a Str field is transitively reachable. Computed from the
        // shapes, never trusted from the input; consumed by the combination
        // body rules (and, in T-04, the resource walk).
        let resource_map = if self.domain == HirDomain::TextRecord {
            record_resource_map(m)
        } else {
            Vec::new()
        };
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
                // T01-C04: a body-less declaration (the binder's extern
                // scaffold, sentinel BodyId) or a mutated bundle's dangling
                // ref must not receive the WHOLE-MODULE credential — the
                // native backend indexes every def's body, so u32::MAX
                // would panic. Body-less scaffolds stay legal ONLY inside
                // function-level `verify_function` (single-body documents).
                self.error(
                    "verify.missing-body",
                    format!(
                        "declaration `{}` carries no body; whole-module verification requires a real body for every function (body-less declarations are function-level scaffolds only)",
                        d.id_text
                    ),
                    d.span,
                );
                continue;
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
            // F-782-R1-05: structural/reference bounds checks come FIRST
            // (check_body opens with check_reference_bounds and rejects
            // controlled); the text-record semantic rules index expr tables
            // and only run on a structurally valid body.
            self.check_body(m, BodyId(bi as u32), body, &type_map, &sigs);
            if self.domain == HirDomain::TextRecord
                && Self::reference_bounds_diags(m, body).is_empty()
            {
                self.check_text_record_rules(m, body, &type_map, &resource_map);
            }
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
    /// unchanged. On the combination surface (782 §6.3) a `move`-in param
    /// is a legal replacement root as well — the callee owns the root; the
    /// plain text profile keeps 770 semantics.
    fn local_writable(&self, l: &hir::LocalDef) -> bool {
        if l.mutable {
            return true;
        }
        if l.role != hir::LocalRole::Param {
            return false;
        }
        match l.param_mode {
            hir::ParamMode::MutBorrow => true,
            hir::ParamMode::MoveOwn => self.domain == HirDomain::TextRecord,
            hir::ParamMode::View => false,
        }
    }

    /// PLAN-782 combination-domain structural rules (T-02): `move` wrapper
    /// shape (whole-local reads only), `move` param modes restricted to
    /// owning types, and the classification-based bare transfer rejection —
    /// an existing resource record never copies implicitly through
    /// let/assign/return; transfer needs the explicit `move` wrapper or a
    /// `record_clone`. The liveness half (use-after-move, cleanup
    /// schedules) extends the PLAN-770 walk in T-04.
    fn check_text_record_rules(
        &mut self,
        m: &hir::Module,
        body: &hir::Body,
        type_map: &BTreeMap<u32, ValueType>,
        resource: &[bool],
    ) {
        let _ = m;
        for e in &body.exprs {
            if let hir::ExprKind::Move { inner } = &e.kind {
                if !matches!(
                    body.exprs[inner.0 as usize].kind,
                    hir::ExprKind::ReadLocal { .. }
                ) {
                    self.error(
                        "verify.partial-move",
                        format!(
                            "expr `{}`: `.move` must wrap a whole local — moving a field or sub-expression of an owned record is not part of the text-record profile",
                            e.id_text
                        ),
                        e.span,
                    );
                }
            }
        }
        for l in &body.locals {
            if l.param_mode == hir::ParamMode::MoveOwn {
                let vt = self.value_of(&l.ty, type_map);
                let owning = match vt {
                    ValueType::StrOwned => true,
                    ValueType::Record(tid) => resource.get(tid as usize).copied().unwrap_or(false),
                    _ => false,
                };
                if !owning {
                    self.error(
                        "verify.ownership-required",
                        format!(
                            "parameter `{}` declares `move` but its type `{}` does not own buffers; `move` parameters are for `Str` and resource records",
                            l.id_text,
                            vt_name(vt)
                        ),
                        l.span,
                    );
                }
            }
        }
        // Bare transfer of an existing resource record: the RHS/return
        // value must be an OWNING expression form (fresh construction,
        // clone, owning call result, or a `move` wrapper). Reading a whole
        // local (F-782-R1-01 original face) AND reading a resource record
        // FIELD (a view of the root's nested aggregate, never a copy of
        // its owner) are both implicit-copy attempts. Copyable records
        // keep the 771 value-copy semantics and are untouched here.
        let is_owning_form = |k: &hir::ExprKind| {
            matches!(
                k,
                hir::ExprKind::RecordCtor { .. }
                    | hir::ExprKind::RecordClone { .. }
                    | hir::ExprKind::Call { .. }
                    | hir::ExprKind::Move { .. }
            )
        };
        for bl in &body.blocks {
            for s in &bl.stmts {
                let (value, what, what_verb, code) = match s {
                    hir::Stmt::Assign { value, .. } => (
                        value,
                        "assign to a resource record",
                        "assign to",
                        "verify.resource-assign",
                    ),
                    hir::Stmt::Let { value, .. } => (
                        value,
                        "let-bind an existing resource record",
                        "let-bind",
                        "verify.resource-assign",
                    ),
                    hir::Stmt::Return { value } => (
                        value,
                        "return an owned record local",
                        "return",
                        "verify.ownership-required",
                    ),
                    _ => continue,
                };
                let eid = *value;
                let e = &body.exprs[eid.0 as usize];
                if is_owning_form(&e.kind) {
                    continue;
                }
                let vt = self.value_of(&e.ty, type_map);
                if let ValueType::Record(tid) = vt {
                    if resource.get(tid as usize).copied().unwrap_or(false) {
                        let msg = if let hir::ExprKind::ReadLocal { local } = &e.kind {
                            format!(
                                "cannot {} `{}` by implicit copy: its type owns text buffers; use `<local>.move` to transfer it or `<local>.clone()` for an independent copy{}",
                                what,
                                body.locals[local.0 as usize].id_text,
                                if code == "verify.ownership-required" {
                                    " (or a fresh construction)"
                                } else {
                                    ""
                                },
                            )
                        } else {
                            format!(
                                "cannot {} a resource record field value by implicit copy: the field view never owns its buffers; clone it, construct a fresh value, or move the whole root",
                                what_verb,
                            )
                        };
                        self.error(code, msg, e.span);
                    }
                }
            }
        }
        // F-782-R1-01 ctor entry: a record-constructor FIELD argument of
        // resource-record type must likewise be an owning form — a bare
        // field read stores a bitwise copy of the nested aggregate and the
        // new root schedules frees for buffers it does not own.
        for e in &body.exprs {
            if let hir::ExprKind::RecordCtor {
                eval_args,
                bindings,
                ..
            } = &e.kind
            {
                for b in bindings {
                    let Some(&aid) = eval_args.get(b.arg as usize) else {
                        continue;
                    };
                    let Some(arg) = body.exprs.get(aid.0 as usize) else {
                        continue;
                    };
                    if is_owning_form(&arg.kind) {
                        continue;
                    }
                    if let ValueType::Record(tid) = self.value_of(&arg.ty, type_map) {
                        if resource.get(tid as usize).copied().unwrap_or(false) {
                            self.error(
                                "verify.ownership-required",
                                format!(
                                    "constructor field argument implicitly copies a resource record field; clone the value, construct a fresh one, or move the whole root",
                                ),
                                arg.span,
                            );
                        }
                    }
                }
            }
        }
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
                        hir::BuiltinType::StrSlice => ValueType::Str,
                        hir::BuiltinType::StrOwned => ValueType::StrOwned,
                        hir::BuiltinType::Char => ValueType::Char,
                        // PLAN-862: unit is a RESULT type (totality).
                        hir::BuiltinType::Unit => ValueType::Unit,
                    },
                    hir::TypeUse::Ref(tid) => {
                        match type_map.get(&tid.0) {
                            Some(v) => *v,
                            None => continue, // out-of-range already reported
                        }
                    }
                };
                // Text-field semantics are decided by the RESOLVED type, not
                // the TypeUse's shape: the source adapter emits named
                // `t_Str`/`t_str`/`t_char` refs while programmatic bundles
                // use the inline builtins — both spellings meet one gate.
                // In the text-record domain (782) `Str`/`char` fields
                // own/carry values and are the combination face; a borrowed
                // `str` field is the 774 D-03 `verify.field-borrow`
                // rejection. Older domains reject text kinds at the type
                // table already; the resolved vt collapses back to i32
                // there exactly as the inline-builtin path always did.
                let vt = if self.domain == HirDomain::TextRecord {
                    if vt == ValueType::Str {
                        self.error(
                            "verify.field-borrow",
                            format!(
                                "record type `{}`'s field `{}` has borrowed type `str`; record fields must own their text (`Str`), borrowing fields are not part of the text-record profile",
                                t.id_text, f.name
                            ),
                            t.span,
                        );
                    }
                    vt
                } else if matches!(vt, ValueType::Str | ValueType::StrOwned | ValueType::Char) {
                    ValueType::I32
                } else {
                    vt
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
                    // text builtins are legal field types only in the
                    // combination domain (782); elsewhere they surface as
                    // a non-record path exactly as before
                    hir::BuiltinType::StrSlice => ValueType::Str,
                    hir::BuiltinType::StrOwned => {
                        if self.domain == HirDomain::TextRecord {
                            ValueType::StrOwned
                        } else {
                            ValueType::I32
                        }
                    }
                    hir::BuiltinType::Char => {
                        if self.domain == HirDomain::TextRecord {
                            ValueType::Char
                        } else {
                            ValueType::I32
                        }
                    }
                    // PLAN-862: Unit is never a field/place type, so a path
                    // through it is a non-record path (report, don't guess).
                    hir::BuiltinType::Unit => ValueType::Unit,
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
                hir::BuiltinType::StrSlice => ValueType::Str,
                hir::BuiltinType::StrOwned => {
                    if self.domain == HirDomain::TextRecord {
                        ValueType::StrOwned
                    } else {
                        ValueType::I32
                    }
                }
                hir::BuiltinType::Char => {
                    if self.domain == HirDomain::TextRecord {
                        ValueType::Char
                    } else {
                        ValueType::I32
                    }
                }
                // PLAN-862: no place/final-value is ever Unit.
                hir::BuiltinType::Unit => ValueType::Unit,
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
        if matches!(self.domain, HirDomain::Record | HirDomain::TextRecord) {
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
                // PLAN-862: the unit value has no operands.
                hir::ExprKind::Unit => {}
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
                hir::ExprKind::RecordClone { base, .. } => children.push(base.0),
                hir::ExprKind::Move { inner } => children.push(inner.0),
                hir::ExprKind::ReadLocal { .. } | hir::ExprKind::Constant { .. } => {}
                hir::ExprKind::BorrowMut { .. } => {}
                // PLAN-864 collections kinds: their child edges mirror the
                // same operand fields the walk already understands.
                hir::ExprKind::ListLen { list }
                | hir::ExprKind::ListClear { list }
                | hir::ExprKind::ListGet { list, .. }
                | hir::ExprKind::ListRemove { list, .. } => children.push(list.0),
                hir::ExprKind::ListPush { list, value }
                | hir::ExprKind::ListSet { list, value, .. } => {
                    children.push(list.0);
                    children.push(value.0);
                }
                hir::ExprKind::ListInsert { list, index, value } => {
                    children.push(list.0);
                    children.push(index.0);
                    children.push(value.0);
                }
                hir::ExprKind::ListNew { .. } => {}
                hir::ExprKind::VariantCtor { payload, .. } => {
                    if let Some(p) = payload {
                        children.push(p.0);
                    }
                }
                hir::ExprKind::VariantGuardTest { root, .. }
                | hir::ExprKind::VariantProject { root, .. } => children.push(root.0),
                // text kinds never appear in a record module (profile
                // defense); totality only
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
                | hir::ExprKind::CharFromCodepoint { .. } => {}
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
                // F-771-R1-03: a mutated bundle can carry an out-of-range
                // binding arg — the dedicated binding validation in
                // check_blocks reports it; this earlier ownership walk
                // must skip the binding instead of indexing past eval_args
                let Some(arg_e) = eval_args
                    .get(b.arg as usize)
                    .and_then(|a| body.exprs.get(a.0 as usize))
                else {
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
                    (hir::ExprKind::BorrowMut { .. }, hir::ParamMode::MutBorrow, true) => {}
                    (hir::ExprKind::BorrowView { .. }, hir::ParamMode::View, true) => {}
                    (hir::ExprKind::BorrowView { .. }, hir::ParamMode::MutBorrow, true) => {
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
                    // PLAN-782 combination face: a `move`-in param takes
                    // the whole root through a `.move` wrapper argument —
                    // the marker and the mode are symmetric (771 double
                    // gate extended)
                    (hir::ExprKind::Move { .. }, hir::ParamMode::MoveOwn, true) => {}
                    (hir::ExprKind::Move { .. }, _, true) => {
                        self.error(
                            "verify.ownership-required",
                            format!(
                                "expr `{}`: param {} is not a `move` parameter and does not accept a `.move` argument",
                                e.id_text, b.param
                            ),
                            arg_e.span,
                        );
                    }
                    (_, hir::ParamMode::MoveOwn, true) => {
                        self.error(
                            "verify.ownership-required",
                            format!(
                                "expr `{}`: param {} is declared `move` and requires a `.move` argument (whole-root transfer)",
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
        let diags = Self::reference_bounds_diags(m, body);
        let ok = diags.is_empty();
        self.diags.extend(diags);
        ok
    }

    /// Pure half of the reference-bounds walk (F-782-R1-05): callers that
    /// only need the outcome — e.g. gating the text-record semantic rules
    /// on structural validity — must not double-emit the diagnostics.
    fn reference_bounds_diags(m: &hir::Module, body: &hir::Body) -> Vec<Diagnostic> {
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
                // PLAN-862: no references to bounds-check.
                hir::ExprKind::Unit => {}
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
                hir::ExprKind::RecordClone { base, .. } => {
                    if (base.0 as usize) >= n_exprs {
                        bad("expression", base.0, n_exprs, e.span);
                    }
                }
                hir::ExprKind::Move { inner } => {
                    if (inner.0 as usize) >= n_exprs {
                        bad("expression", inner.0, n_exprs, e.span);
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
                // PLAN-864 collections kinds: operand bounds.
                hir::ExprKind::ListNew { .. } => {}
                hir::ExprKind::ListLen { list } | hir::ExprKind::ListClear { list } => {
                    if (list.0 as usize) >= n_exprs {
                        bad("expression", list.0, n_exprs, e.span);
                    }
                }
                hir::ExprKind::ListPush { list, value } => {
                    for id in [list, value] {
                        if (id.0 as usize) >= n_exprs {
                            bad("expression", id.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::ListGet { list, index }
                | hir::ExprKind::ListRemove { list, index } => {
                    for id in [list, index] {
                        if (id.0 as usize) >= n_exprs {
                            bad("expression", id.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::ListInsert { list, index, value }
                | hir::ExprKind::ListSet { list, index, value } => {
                    for id in [list, index, value] {
                        if (id.0 as usize) >= n_exprs {
                            bad("expression", id.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::VariantCtor { payload, .. } => {
                    if let Some(p) = payload {
                        if (p.0 as usize) >= n_exprs {
                            bad("expression", p.0, n_exprs, e.span);
                        }
                    }
                }
                hir::ExprKind::VariantGuardTest { root, .. }
                | hir::ExprKind::VariantProject { root, .. } => {
                    if (root.0 as usize) >= n_exprs {
                        bad("expression", root.0, n_exprs, e.span);
                    }
                }
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
                    hir::Stmt::Eval { value } => {
                        if (value.0 as usize) >= n_exprs {
                            bad("expression", value.0, n_exprs, bl.span);
                        }
                    }
                }
            }
        }
        diags
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
            if lvt == ValueType::Function && self.domain != HirDomain::Closures {
                self.error(
                    "verify.local-type",
                    format!(
                        "local `{}` has a function type; function values exist only in the closures profile",
                        l.id_text,
                    ),
                    l.span,
                );
            } else if text_vt
                && !(matches!(self.domain, HirDomain::Text | HirDomain::TextRecord)
                    || (matches!(self.domain, HirDomain::Collections | HirDomain::Closures)
                        && lvt == ValueType::StrOwned))
            {
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
                hir::TypeUse::Builtin(hir::BuiltinType::StrSlice) => ValueType::Str,
                hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => ValueType::StrOwned,
                hir::TypeUse::Builtin(hir::BuiltinType::Char) => ValueType::Char,
                // PLAN-862: the unit RESULT type.
                hir::TypeUse::Builtin(hir::BuiltinType::Unit) => ValueType::Unit,
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
                        // PLAN-862: the unit value. Legal ONLY in the
                        // entry-unit domain, typed `unit` exactly, and (the
                        // result-position rule) consumed only by a `return`
                        // of a Unit function — checked where returns are
                        // verified.
                        hir::ExprKind::Unit => {
                            if self.domain != HirDomain::EntryUnit {
                                self.error(
                                    "verify.unit-expr",
                                    format!(
                                        "the unit value `{}` exists only in the entry-unit profile (this bundle is `{}`)",
                                        e.id_text,
                                        self.domain.as_str()
                                    ),
                                    e.span,
                                );
                            } else if declared != ValueType::Unit {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                                        "unit value `{}` must have type unit, found `{}`",
                                        e.id_text,
                                        vt_name(declared)
                                    ),
                                    e.span,
                                );
                            }
                            tm.gray.remove(&id.0);
                            tm.memo.insert(id.0, ValueType::Unit);
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
                        hir::ExprKind::RecordClone { base, .. } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*base));
                        }
                        hir::ExprKind::Move { inner } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*inner));
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
                            if !self.local_writable(l) {
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
                        // -- PLAN-864 collections kinds ----------------------
                        // Operand traversal + memoization only; the
                        // cross-expression invariants (element domain, disc
                        // bounds, view discipline) live in the collections
                        // verification pass.
                        hir::ExprKind::ListNew { .. } => {
                            tm.gray.remove(&id.0);
                            tm.memo.insert(id.0, declared);
                        }
                        hir::ExprKind::ListLen { list }
                        | hir::ExprKind::ListClear { list }
                        | hir::ExprKind::ListPush { list, .. }
                        | hir::ExprKind::ListInsert { list, .. }
                        | hir::ExprKind::ListGet { list, .. }
                        | hir::ExprKind::ListSet { list, .. }
                        | hir::ExprKind::ListRemove { list, .. } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*list));
                        }
                        hir::ExprKind::VariantCtor { payload, .. } => {
                            stack.push(Visit::Exit(id));
                            if let Some(p) = payload {
                                stack.push(Visit::Enter(*p));
                            }
                        }
                        hir::ExprKind::VariantGuardTest { root, .. }
                        | hir::ExprKind::VariantProject { root, .. } => {
                            stack.push(Visit::Exit(id));
                            stack.push(Visit::Enter(*root));
                        }
                    }
                }
                Visit::Exit(id) => {
                    let e = &body.exprs[id.0 as usize];
                    let declared = declared_of(e);
                    let computed = match &e.kind {
                        // PLAN-864 collections kinds: the declared type IS
                        // the computed type (list/variant ctors, len i32,
                        // guard bool, projection payload); the operand
                        // invariants are the collections pass's job.
                        hir::ExprKind::ListNew { .. }
                        | hir::ExprKind::ListLen { .. }
                        | hir::ExprKind::ListPush { .. }
                        | hir::ExprKind::ListInsert { .. }
                        | hir::ExprKind::ListGet { .. }
                        | hir::ExprKind::ListSet { .. }
                        | hir::ExprKind::ListRemove { .. }
                        | hir::ExprKind::ListClear { .. }
                        | hir::ExprKind::VariantCtor { .. }
                        | hir::ExprKind::VariantGuardTest { .. }
                        | hir::ExprKind::VariantProject { .. } => declared,
                        // PLAN-862: leaf, already checked at Enter.
                        hir::ExprKind::Unit => declared,
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
                        hir::ExprKind::RecordClone { record, base } => {
                            let bt =
                                operand_type(tm, *base, declared_of(&body.exprs[base.0 as usize]));
                            match bt {
                                ValueType::Record(rec) if rec == record.0 => {
                                    if declared != ValueType::Record(record.0) {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                "expr `{}`: record clone type does not match the cloned record",
                                e.id_text
                            ),
                                            e.span,
                                        );
                                    }
                                    ValueType::Record(record.0)
                                }
                                _ => {
                                    self.error(
                        "verify.type-mismatch",
                        format!(
                            "expr `{}`: `clone` here deep-copies a record value of the same record type (found `{}`)",
                            e.id_text,
                            vt_name(bt)
                        ),
                        e.span,
                    );
                                    declared
                                }
                            }
                        }
                        hir::ExprKind::Move { inner } => {
                            let it = operand_type(
                                tm,
                                *inner,
                                declared_of(&body.exprs[inner.0 as usize]),
                            );
                            if declared != it {
                                self.error(
                                    "verify.type-mismatch",
                                    format!(
                        "expr `{}`: the move wrapper carries its operand's type (found `{}`)",
                        e.id_text,
                        vt_name(it)
                    ),
                                    e.span,
                                );
                            }
                            it
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
                                // PLAN-864: composites (List/Variant) are never
                                // view-borrowed in this surface.
                                ValueType::Opaque => {
                                    self.error(
                                        "verify.type-mismatch",
                                        format!(
                                            "expr `{}`: a composite value has no view borrow",
                                            e.id_text
                                        ),
                                        e.span,
                                    );
                                    ValueType::Opaque
                                }
                                // PLAN-862: a unit value can never be borrowed;
                                // the mismatch below reports it (no record shape).
                                ValueType::Unit => {
                                    self.error(
                                        "verify.type-mismatch",
                                        format!(
                                            "expr `{}`: a unit value has no view borrow",
                                            e.id_text
                                        ),
                                        e.span,
                                    );
                                    ValueType::Unit
                                }
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
                                ValueType::Function
                                | ValueType::I32
                                | ValueType::Bool
                                | ValueType::Str
                                | ValueType::StrOwned
                                | ValueType::Char => {
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
                                    // PLAN-782 5.2: the source front end
                                    // retags an owned Str-field READ as the
                                    // borrowing view `str` (a read never
                                    // copies the owner); both spellings are
                                    // the same field.
                                    let compatible = final_vt == declared
                                        || (final_vt == ValueType::StrOwned
                                            && declared == ValueType::Str);
                                    if !compatible {
                                        self.error(
                                            "verify.type-mismatch",
                                            format!(
                                                "expr `{}`: field path type does not match the accessed field",
                                                e.id_text
                                            ),
                                            e.span,
                                        );
                                    }
                                    // the retagged view spelling IS the
                                    // expression's type on the surface
                                    declared
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
                                            // legal combination field types
                                            // (782); dead mapping elsewhere
                                            hir::BuiltinType::StrSlice => ValueType::Str,
                                            hir::BuiltinType::StrOwned => ValueType::StrOwned,
                                            hir::BuiltinType::Char => ValueType::Char,
                                            // PLAN-862: a record field is never
                                            // unit (placement is rejected).
                                            hir::BuiltinType::Unit => ValueType::Unit,
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
                    | hir::Stmt::Assign { value, .. }
                    | hir::Stmt::Eval { value } => vec![*value],
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
                hir::ExprKind::RecordClone { base, .. } => {
                    targets.push(base.0);
                }
                hir::ExprKind::Move { inner } => {
                    targets.push(inner.0);
                }
                // PLAN-864 collections kinds: operand edges.
                hir::ExprKind::ListLen { list } | hir::ExprKind::ListClear { list } => {
                    targets.push(list.0);
                }
                hir::ExprKind::ListPush { list, value } => {
                    targets.push(list.0);
                    targets.push(value.0);
                }
                hir::ExprKind::ListGet { list, index }
                | hir::ExprKind::ListRemove { list, index } => {
                    targets.push(list.0);
                    targets.push(index.0);
                }
                hir::ExprKind::ListInsert { list, index, value }
                | hir::ExprKind::ListSet { list, index, value } => {
                    targets.push(list.0);
                    targets.push(index.0);
                    targets.push(value.0);
                }
                hir::ExprKind::VariantCtor { payload, .. } => {
                    if let Some(p) = payload {
                        targets.push(p.0);
                    }
                }
                hir::ExprKind::VariantGuardTest { root, .. }
                | hir::ExprKind::VariantProject { root, .. } => {
                    targets.push(root.0);
                }
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
                        hir::Stmt::Eval { value } => {
                            self.walk_value(value, body, &frame.cur);
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
                                if !self.local_writable(target) {
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
                // PLAN-862: the unit value reads no local.
                hir::ExprKind::Unit => {}
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
                // PLAN-864 collections kinds: traverse operands.
                hir::ExprKind::ListLen { list }
                | hir::ExprKind::ListClear { list }
                | hir::ExprKind::ListPush { list, .. }
                | hir::ExprKind::ListInsert { list, .. }
                | hir::ExprKind::ListGet { list, .. }
                | hir::ExprKind::ListSet { list, .. }
                | hir::ExprKind::ListRemove { list, .. } => stack.push(*list),
                hir::ExprKind::ListNew { .. } => {}
                hir::ExprKind::VariantCtor { payload, .. } => {
                    if let Some(p) = payload {
                        stack.push(*p);
                    }
                }
                hir::ExprKind::VariantGuardTest { root, .. }
                | hir::ExprKind::VariantProject { root, .. } => stack.push(*root),
                hir::ExprKind::RecordClone { base, .. } => {
                    stack.push(*base);
                }
                hir::ExprKind::Move { inner } => {
                    stack.push(*inner);
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
            }
        }
    }
}

impl hir::TypeUse {
    /// Content-canonical shape of a type use (public helper for
    /// body-read intrinsic binding facts).
    pub fn shape(u: &hir::TypeUse, m: &hir::Module) -> String {
        match u {
            hir::TypeUse::Builtin(b) => format!("{b:?}"),
            hir::TypeUse::Ref(tid) => match m.types.get(tid.0 as usize).map(|t| &t.kind) {
                Some(hir::TypeKind::I32) => "i32".to_string(),
                Some(hir::TypeKind::Bool) => "bool".to_string(),
                _ => "ty".to_string(),
            },
        }
    }
}

/// PLAN-871 T-06: the build-side capability backstop, MOVED here from the
/// retiring `native` lowering module — it reads verifier facts only (the
/// required-capability set plus the DERIVED text-runtime need) and never
/// touches lowering. Every backend and consumer enforces the same set
/// through this one entry.
pub fn capabilities_check(
    checked: &CheckedModule,
    provided: &[String],
) -> Result<(), Vec<Diagnostic>> {
    let mut diags = Vec::new();
    // r2 QA-02: the DERIVED runtime need gates even without a declaration
    // (verify_text already rejects undeclared usage; this is the build-side
    // backstop).
    if checked.requires_runtime_text()
        && !provided
            .iter()
            .any(|p| p == crate::text_resources::RUNTIME_TEXT_CAPABILITY)
    {
        diags.push(Diagnostic::new(
            Stage::Capability,
            "capability.missing",
            format!(
                "document requires capability `{}`, which this build does not provide",
                crate::text_resources::RUNTIME_TEXT_CAPABILITY
            ),
            Span { start: 0, end: 0 },
        ));
    }
    for req in checked.required_capabilities() {
        if !provided.iter().any(|p| p == req) {
            diags.push(Diagnostic::new(
                Stage::Capability,
                "capability.missing",
                format!(
                    "document requires capability `{}`, which this build does not provide",
                    req
                ),
                Span { start: 0, end: 0 },
            ));
        }
    }
    if diags.is_empty() {
        Ok(())
    } else {
        Err(diags)
    }
}
