//! Target-neutral semantic view over verified HIR (PLAN-892 AX-A: the
//! core/scalar compute surface; PLAN-902 AX-B: widened to the record, text
//! and text-record domains).
//!
//! The view is READ-ONLY over the checked bundle: it re-checks each
//! supported domain's admission surface (defense in depth on top of the
//! frontend, which already enforces the same sets), collects per-function
//! declaration facts (signature, params+modes, result, record shapes,
//! method ownership) and exposes borrowed body structure plus source-order
//! evaluation facts and the verifier-computed text resource plan. It
//! produces no target code and knows no target — that is the Rust
//! provider's job (a2x-rust).
//!
//! Evaluation-order contract (plan §4): `ExprKind::Call` carries
//! `eval_args` (SOURCE order, evaluate exactly once) plus `bindings`
//! (param ← arg placement). The view exposes both unchanged; a target
//! never re-guesses argument order from the binding map.
//!
//! PLAN-902 widening rules (§5.2): the view stays a single source of
//! semantic truth — it never re-derives ownership (the resource plan is
//! the verifier's, consumed verbatim), and the scalar/core paths keep
//! byte-identical fingerprints and rejection behavior (the 892 corpus and
//! cache keys must not move).

use ace_core::hir;
use ace_core::source::SourceParts;
use ace_core::text_resources::TextResourcePlan;
use ace_core::verify::{CheckedFunction, CheckedModule};
use std::collections::BTreeSet;

use crate::A2xReject;

/// The value types of the supported source surfaces (PLAN-892: i32/bool;
/// PLAN-902: char, borrowed str, owned Str, nominal records; PLAN-904:
/// structural `List<T>` and nominal variants).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ty {
    I32,
    Bool,
    /// Independent Unicode scalar (PLAN-770).
    Char,
    /// Borrowed UTF-8 view (`str`).
    Str,
    /// Owned UTF-8 buffer (`Str`).
    StrOwned,
    /// Nominal record type; the payload is the module type-table index.
    /// Nominal identity is the type table entry (name + shape), never the
    /// layout alone.
    Record(u32),
    /// Owned typed list (PLAN-904). The payload is the module type-table
    /// index of the `List { elem }` entry; the frontend interns list types
    /// STRUCTURALLY (same element type ⇒ same entry), so index equality
    /// within one module is structural equality.
    List(u32),
    /// Nominal variant (PLAN-904: `enum`/`?T`/`Result<T,E>`); the payload
    /// is the module type-table index of the `Variant { arms }` entry
    /// (nominal identity by declaration).
    Variant(u32),
    /// The unit type (PLAN-904: the collections `Eval` statement's
    /// discarded result).
    Unit,
    /// PLAN-961 numeric surface: the signed/unsigned/float value types.
    /// `byte`/`uint` resolve to `U8`/`U32` at the source front.
    I8,
    U8,
    U32,
    I64,
    U64,
    F32,
    F64,
}

impl Ty {
    pub fn as_str(self) -> &'static str {
        match self {
            Ty::I32 => "i32",
            Ty::Bool => "bool",
            Ty::Char => "char",
            Ty::Str => "str",
            Ty::StrOwned => "Str",
            // nominal/structural rendering needs the module — see
            // SemanticView::ty_name
            Ty::Record(_) => "record",
            Ty::List(_) => "list",
            Ty::Variant(_) => "variant",
            Ty::Unit => "unit",
            Ty::I8 => "i8",
            Ty::U8 => "u8",
            Ty::U32 => "u32",
            Ty::I64 => "i64",
            Ty::U64 => "u64",
            Ty::F32 => "f32",
            Ty::F64 => "f64",
        }
    }

    pub fn is_text(self) -> bool {
        matches!(self, Ty::Str | Ty::StrOwned)
    }
}

/// One declared parameter of a function (name, target-neutral type and the
/// verified ownership mode; PLAN-902: the mode is semantic identity).
#[derive(Clone, Debug)]
pub struct ParamFacts {
    pub name: String,
    pub ty: Ty,
    pub mode: hir::ParamMode,
}

/// How a function def is owned: a free function or a type-bound method
/// (PLAN-881). Methods carry their owner record and receiver shape; the
/// stable owner-qualified identity mirrors the frozen `m.<Owner>.<name>`
/// lowering identity.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FnKind {
    Free,
    Method {
        owner: u32,
        receiver: hir::ReceiverKind,
    },
}

/// Declaration-level facts of one user function plus its borrowed verified
/// body. `def_index` is the module def-table index (stable identity for
/// call targets within this view).
#[derive(Clone, Debug)]
pub struct FnFacts<'a> {
    pub def_index: usize,
    /// The HIR-internal def name (`sf_<source>` for free functions,
    /// `sm_<Owner>_<name>` for methods).
    pub name: &'a str,
    /// The SOURCE name: `sf_`/`sm_` stripped. For methods this is
    /// owner-qualified (`Counter.new`) so same-named methods of different
    /// owners never collide; entry selection and diagnostics use this.
    pub source_name: &'a str,
    pub kind: FnKind,
    pub params: Vec<ParamFacts>,
    pub result: Ty,
    /// Byte span of the function's def in the source it was compiled from
    /// (source-map v1; plan §5.3 — spans travel with the target artifacts
    /// so diagnostics map back to the original `.at`).
    pub origin: (u32, u32),
    /// Index of this function's body in the module body table (resource
    /// plan lookup key).
    pub body_index: u32,
    pub body: &'a hir::Body,
}

/// Signature-only facts of a function whose BODY is not part of this view —
/// the scaffold declarations a single-body emission carries for the other
/// functions (PLAN-828). Calls to externals admit by signature; the target
/// names them by def identity and the session layer (T-04) assembles the
/// closure. PLAN-902: the verified param MODES travel with the signature —
/// a callee mode change is a dependency change, not a rendering detail.
#[derive(Clone, Debug)]
pub struct ExternalFn {
    pub def_index: usize,
    pub source_name: String,
    pub kind: FnKind,
    pub params: Vec<Ty>,
    pub modes: Vec<hir::ParamMode>,
    pub result: Ty,
}

/// One field of a nominal record, in declaration order (the layout order;
/// 774 D-05 — declaration order participates in semantic equality).
#[derive(Clone, Debug)]
pub struct FieldFacts {
    pub name: String,
    pub ty: Ty,
}

/// Declaration facts of one nominal record type (PLAN-771/782). `resource`
/// is the verifier-side classification: the transitive field closure
/// contains an owned `Str`, so values own buffers and follow the 774
/// data-resource contract (explicit clone/move, planned cleanup).
#[derive(Clone, Debug)]
pub struct RecordFacts {
    pub type_index: u32,
    /// Nominal name (`t_rec_` prefix stripped).
    pub name: String,
    pub fields: Vec<FieldFacts>,
    pub resource: bool,
}

/// One arm of a nominal variant, in declaration order (PLAN-904). The arm
/// INDEX is the runtime discriminant; `payload: None` is the no-payload
/// case.
#[derive(Clone, Debug)]
pub struct VariantArmFacts {
    pub name: String,
    pub payload: Option<Ty>,
}

/// Declaration facts of one nominal variant type (PLAN-904).
#[derive(Clone, Debug)]
pub struct VariantFacts {
    pub type_index: u32,
    /// Nominal name (`t_variant_` prefix stripped).
    pub name: String,
    pub arms: Vec<VariantArmFacts>,
}

/// The read-only semantic view. Constructed only via
/// [`SemanticView::from_checked`]; every admission failure is a named
/// [`A2xReject`].
#[derive(Debug)]
pub struct SemanticView<'a> {
    module: &'a hir::Module,
    pub domain: hir::HirDomain,
    /// The source profile identity this domain corresponds to.
    pub profile: &'static str,
    functions: Vec<FnFacts<'a>>,
    /// Signature-only functions (single-body fragments; see `ExternalFn`).
    externals: Vec<ExternalFn>,
    records: Vec<RecordFacts>,
    /// Nominal variant declarations, in type-table order (PLAN-904).
    variants: Vec<VariantFacts>,
    /// The verifier-computed text resource plan (text/text-record domains),
    /// consumed verbatim — the view never re-derives ownership.
    resource_plan: Option<&'a TextResourcePlan>,
    /// PLAN-902 T-03: a single-body credential's OWN plan (fragment views);
    /// bound to the credential's body index, lookup-checked per function.
    fragment_plan: Option<(&'a ace_core::text_resources::BodyResourcePlan, u32)>,
    /// PLAN-904: the verifier-computed collections facts plan
    /// (collections/closures domains), consumed verbatim.
    coll_plan: Option<&'a ace_core::collections_facts::CollectionsPlan>,
    /// PLAN-939 T-04: the verifier-computed foreign resource credential
    /// plan (resource-foreign text-record modules), consumed verbatim —
    /// per-call ownership facts (view borrows, exclusive lanes, handle
    /// credentials, callback targets) are READ, never re-derived.
    foreign_plan: Option<&'a ace_core::foreign_resources::ForeignResourcePlan>,
    /// PLAN-904: a single-body credential's OWN collections facts (fragment
    /// views); bound to the credential's body index, lookup-checked.
    fragment_facts: Option<(&'a ace_core::collections_facts::CollectionsBodyFacts, u32)>,
    /// PLAN-939 T-04: a single-body credential's OWN foreign resource
    /// credential rows (fragment views); the per-call ownership authority
    /// when no module plan is present.
    fragment_foreign: Option<(
        &'a std::collections::BTreeMap<u32, ace_core::foreign_resources::ForeignCallCredential>,
        u32,
    )>,
    /// Authorized trace intrinsic defs present: def_index → symbol.
    trace_defs: Vec<(usize, &'static str)>,
    /// Document-level capabilities (trimmed to the ones A2X cares about).
    pub capabilities: Vec<String>,
}

const TRACE_CAPABILITY: &str = "hir.test.trace.v1";

impl<'a> SemanticView<'a> {
    /// Build the view from a verified module, admitting only the supported
    /// source surfaces (core/scalar since 892; record/text/text-record
    /// since PLAN-902). `CheckedModule` construction is private to the
    /// verifier — a view can only be built over genuinely verified content.
    pub fn from_checked(checked: &'a CheckedModule) -> Result<Self, A2xReject> {
        let domain = checked.domain();
        let profile = profile_of(domain)?;
        let bundle = checked.bundle();
        let module = single_module(bundle)?;
        let functions = collect_functions(module, None)?;
        let records = collect_records(module)?;
        let variants = collect_variants(module)?;
        let trace_defs = collect_trace_defs(module, bundle)?;
        let view = SemanticView {
            module,
            domain,
            profile,
            functions,
            externals: Vec::new(),
            records,
            variants,
            resource_plan: checked.text_resources(),
            fragment_plan: None,
            coll_plan: checked.collections_plan(),
            fragment_facts: None,
            foreign_plan: checked.foreign_resource_plan(),
            fragment_foreign: None,
            trace_defs,
            capabilities: bundle.requires.clone(),
        };
        view.walk_bodies()?;
        Ok(view)
    }

    /// Build a FRAGMENT view over one single-body emission
    /// (`emit_function_body`, PLAN-828) together with its verification
    /// credential. The credential — not body presence — is the authority
    /// for which def was planned and verified: the function owning
    /// `credential.def_id_text()` becomes the view's one admitted function;
    /// every other function def stays a signature-only [`ExternalFn`].
    /// Nothing unverified is ever admitted (plan §5.3: 磁盘checked只是待验
    /// 数据，恢复由原producer rebind/reverify).
    pub fn from_parts(
        parts: &'a SourceParts,
        credential: &'a CheckedFunction,
    ) -> Result<Self, A2xReject> {
        let bundle = &parts.bundle;
        let module = single_module(bundle)?;
        let domain = bundle.domain;
        let profile = profile_of(domain)?;
        if parts.profile != profile {
            return Err(A2xReject::domain(format!(
                "parts profile `{}` disagrees with domain `{}`",
                parts.profile, profile
            )));
        }
        if credential.domain() != domain {
            return Err(A2xReject::domain(format!(
                "credential domain `{}` disagrees with bundle domain `{}`",
                credential.domain().as_str(),
                domain.as_str()
            )));
        }
        // PLAN-902 V-01: the credential is bound to ITS declaration context
        // by the verifier's fingerprint (types + def signatures). Def ids
        // are positional — a same-shape bundle of a DIFFERENT owner would
        // match by id text alone, so the context revision is the binding:
        // a stale or foreign credential is a named rejection, never a
        // silently admitted look-alike.
        let ctx = ace_core::verify::declaration_context_fingerprint_in(module, domain);
        if credential.context_revision() != ctx {
            return Err(A2xReject::new(
                "a2x.credential",
                format!(
                    "credential context revision `{}` does not bind this bundle's declaration context `{}` (stale or foreign credential)",
                    credential.context_revision(),
                    ctx
                ),
            ));
        }
        let functions = collect_functions(module, Some(credential.def_id_text()))?;
        if functions.len() != 1 {
            return Err(A2xReject::new(
                "a2x.internal",
                format!(
                    "fragment admission matched {} planned functions, expected exactly the credential's",
                    functions.len()
                ),
            ));
        }
        // PLAN-904 (V-13): a declaration-context match does NOT prove the
        // credential was verified against THIS body. Bind the credential
        // to the exact body content by the canonical span-free digest —
        // the same owner + the same declaration context + a different
        // body can never mix (A's credential + B's SourceParts is a
        // named rejection, and B's own legitimate credential still
        // admits).
        let parts_digest = ace_core::collections_facts::body_content_digest(functions[0].body);
        if credential.body_digest() != parts_digest {
            return Err(A2xReject::new(
                "a2x.credential",
                format!(
                    "credential body digest `{}` does not bind this fragment's body `{}` (same context, different body)",
                    credential.body_digest(),
                    parts_digest
                ),
            ));
        }
        let trace_defs = collect_trace_defs(module, bundle)?;
        // PLAN-902 T-03: the credential's OWN verified resource plan
        // serves the one admitted function (lookup-checked by body
        // index — a mismatched plan can never leak to another body).
        let fragment_plan = credential
            .resource_plan()
            .map(|p| (p, functions[0].body_index));
        // PLAN-904: the same binding for the collections body facts.
        let fragment_facts = credential
            .coll_facts()
            .map(|p| (p, functions[0].body_index));
        // PLAN-939 T-04: the same binding for the foreign credential rows.
        let fragment_foreign = credential
            .foreign_rows()
            .map(|rows| (rows, functions[0].body_index));
        let view = SemanticView {
            module,
            domain,
            profile,
            functions,
            externals: collect_externals(module, credential.def_id_text())?,
            records: collect_records(module)?,
            variants: collect_variants(module)?,
            resource_plan: None,
            fragment_plan,
            coll_plan: None,
            fragment_facts,
            // PLAN-939 T-04: a single-body fragment view has no whole-
            // module foreign plan; its OWN credential rows ride instead.
            foreign_plan: None,
            fragment_foreign,
            trace_defs,
            capabilities: bundle.requires.clone(),
        };
        view.walk_bodies()?;
        Ok(view)
    }

    /// All declared user functions with bodies in this view, in def order.
    pub fn functions(&self) -> &[FnFacts<'a>] {
        &self.functions
    }

    /// Signature-only externals (fragment views only).
    pub fn externals(&self) -> &[ExternalFn] {
        &self.externals
    }

    /// All nominal record declarations, in type-table order.
    pub fn records(&self) -> &[RecordFacts] {
        &self.records
    }

    /// The record owning `type_index`, if any.
    pub fn record(&self, type_index: u32) -> Option<&RecordFacts> {
        self.records.iter().find(|r| r.type_index == type_index)
    }

    /// All nominal variant declarations, in type-table order (PLAN-904).
    pub fn variants(&self) -> &[VariantFacts] {
        &self.variants
    }

    /// The variant owning `type_index`, if any (PLAN-904).
    pub fn variant(&self, type_index: u32) -> Option<&VariantFacts> {
        self.variants.iter().find(|v| v.type_index == type_index)
    }

    /// The element type of the list type-table entry (PLAN-904).
    pub fn list_elem(&self, type_index: u32) -> Result<Ty, A2xReject> {
        match self.module.types.get(type_index as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::List { elem }) => resolve_ty(self.module, *elem),
            _ => Err(A2xReject::new(
                "a2x.internal",
                format!("type #{type_index} is not a list entry"),
            )),
        }
    }

    /// All list type-table entries, in table order (PLAN-926: the C
    /// target's typed list instantiations are seed-derived; a read-only
    /// projection — no policy, no analysis).
    pub fn list_types(&self) -> Vec<u32> {
        self.module
            .types
            .iter()
            .enumerate()
            .filter_map(|(idx, t)| {
                matches!(t.kind, hir::TypeKind::List { .. }).then_some(idx as u32)
            })
            .collect()
    }

    /// The verifier-computed text resource plan, when this domain carries
    /// one (text/text-record). `None` for core/scalar/record.
    pub fn resource_plan(&self) -> Option<&TextResourcePlan> {
        self.resource_plan
    }

    /// PLAN-939 T-04: the verifier-computed foreign resource credential
    /// plan (per-call ownership rows); `None` outside resource-foreign
    /// modules.
    pub fn foreign_resource_plan(
        &self,
    ) -> Option<&ace_core::foreign_resources::ForeignResourcePlan> {
        self.foreign_plan
    }

    /// PLAN-939 T-04: the credential rows of ONE foreign call — the
    /// module plan first, then this view's own single-body credential
    /// rows (body-index lookup-checked); `None` when neither carries a
    /// row (the consumer falls back to no plan: fragment views without
    /// foreign calls).
    pub fn foreign_call_credential(
        &self,
        body_index: u32,
        expr: u32,
    ) -> Option<&ace_core::foreign_resources::ForeignCallCredential> {
        if let Some(plan) = self.foreign_plan {
            if let Some(rows) = plan.bodies.get(&(0, body_index)) {
                if let Some(row) = rows.get(&expr) {
                    return Some(row);
                }
            }
        }
        let (rows, bound_body) = self.fragment_foreign.as_ref()?;
        if *bound_body != body_index {
            return None;
        }
        rows.get(&expr)
    }

    /// The resource plan of one function's body (module 0; body index of
    /// `f`). `None` when the domain carries no plan or the body has none.
    /// A fragment view serves the credential's own plan, body-index
    /// checked (PLAN-902 T-03).
    pub fn body_plan(
        &self,
        f: &FnFacts<'a>,
    ) -> Option<&ace_core::text_resources::BodyResourcePlan> {
        if let Some((plan, body_index)) = self.fragment_plan {
            if body_index == f.body_index {
                return Some(plan);
            }
            return None;
        }
        self.resource_plan
            .and_then(|p| p.bodies.get(&(0, f.body_index)))
    }

    /// The verifier-computed collections body facts of one function's body
    /// (PLAN-904; module 0). A fragment view serves the credential's own
    /// facts, body-index checked.
    pub fn body_facts(
        &self,
        f: &FnFacts<'a>,
    ) -> Option<&ace_core::collections_facts::CollectionsBodyFacts> {
        if let Some((facts, body_index)) = self.fragment_facts {
            if body_index == f.body_index {
                return Some(facts);
            }
            return None;
        }
        self.coll_plan
            .and_then(|p| p.bodies.get(&(0, f.body_index)))
    }

    /// The user function owning `def_index`, if any.
    pub fn fn_by_def(&self, def_index: usize) -> Option<&FnFacts<'a>> {
        self.functions.iter().find(|f| f.def_index == def_index)
    }

    /// The signature-only external owning `def_index`, if any.
    pub fn external_by_def(&self, def_index: usize) -> Option<&ExternalFn> {
        self.externals.iter().find(|f| f.def_index == def_index)
    }

    /// A real source foreign declaration; never a sibling ExternalFn scaffold.
    pub fn foreign_by_def(
        &self,
        def_index: usize,
    ) -> Option<&ace_foreign_contract::manual::SourceDecl> {
        match &self.module.defs.get(def_index)?.kind {
            hir::DefKind::Foreign { declaration, .. } => Some(declaration),
            _ => None,
        }
    }

    pub fn foreign_declarations(
        &self,
    ) -> impl Iterator<Item = &ace_foreign_contract::manual::SourceDecl> {
        self.module.defs.iter().filter_map(|d| match &d.kind {
            hir::DefKind::Foreign { declaration, .. } => Some(declaration),
            _ => None,
        })
    }

    /// The authorized trace symbol of `def_index`, if it is one.
    pub fn trace_symbol(&self, def_index: usize) -> Option<&'static str> {
        self.trace_defs
            .iter()
            .find(|(d, _)| *d == def_index)
            .map(|(_, s)| *s)
    }

    /// The distinct authorized trace symbols present (helper-closure input).
    pub fn trace_symbols(&self) -> BTreeSet<&'static str> {
        self.trace_defs.iter().map(|(_, s)| *s).collect()
    }

    /// Upper bound of the entries `trace_symbols()` collects (set nodes and a
    /// caller's `Vec`), so a host can admit that working metadata BEFORE it
    /// exists (PLAN-938). Not a count of distinct symbols.
    pub fn trace_symbol_upper_bound(&self) -> usize {
        self.trace_defs.len()
    }

    /// Select the entry function by explicit source name: it must exist, be
    /// a zero-parameter free function with an i32 result (plan §5.1).
    pub fn entry(&self, name: &str) -> Result<&FnFacts<'a>, A2xReject> {
        let f = self
            .functions
            .iter()
            .find(|f| f.source_name == name)
            .ok_or_else(|| {
                A2xReject::entry(format!("no function named `{}` in this module", name))
            })?;
        if f.kind != FnKind::Free {
            return Err(A2xReject::entry(format!(
                "entry `{}` must be a free function, not a method",
                name
            )));
        }
        if !f.params.is_empty() {
            return Err(A2xReject::entry(format!(
                "entry `{}` must take no parameters (found {})",
                name,
                f.params.len()
            )));
        }
        if f.result != Ty::I32 {
            return Err(A2xReject::entry(format!(
                "entry `{}` must result in i32 (found {})",
                name,
                f.result.as_str()
            )));
        }
        Ok(f)
    }

    /// Resolve a use-site type through this view's module (public for the
    /// target providers' local/param type mapping).
    pub fn resolve_use_ty(&self, ty: hir::TypeUse) -> Result<Ty, A2xReject> {
        resolve_ty(self.module, ty)
    }

    /// Nominal type rendering (semantic identity text): builtins as their
    /// surface names, records/variants by NAME, lists structurally. Used by
    /// the semantic fingerprint and canonical signature — never spans or
    /// table indices.
    pub fn ty_name(&self, ty: Ty) -> String {
        match ty {
            Ty::Record(id) => self
                .record(id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("record#{}", id)),
            Ty::Variant(id) => self
                .variant(id)
                .map(|v| v.name.clone())
                .unwrap_or_else(|| format!("variant#{}", id)),
            Ty::List(id) => match self.list_elem(id) {
                Ok(elem) => format!("List<{}>", self.ty_name(elem)),
                Err(_) => format!("list#{}", id),
            },
            other => other.as_str().to_string(),
        }
    }

    /// Canonical signature with modes and nominal names (PLAN-902 §5.4:
    /// owner/mode/type closure are key inputs). Scalar signatures render
    /// byte-identically to the 892 `canonical_signature` (all-view params,
    /// builtin types only).
    pub fn canonical_signature(&self, f: &FnFacts<'_>) -> String {
        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| {
                let mode_prefix = if p.mode == hir::ParamMode::View {
                    String::new()
                } else {
                    format!("{} ", p.mode.as_str())
                };
                format!("{}{}", mode_prefix, self.ty_name(p.ty))
            })
            .collect();
        format!("({})->{}", params.join(","), self.ty_name(f.result))
    }

    /// The same canonical rendering for a signature-only external
    /// (PLAN-902: callee dependency facts carry modes + nominal types).
    pub fn canonical_external_signature(&self, x: &ExternalFn) -> String {
        let params: Vec<String> = x
            .params
            .iter()
            .zip(x.modes.iter())
            .map(|(ty, mode)| {
                let mode_prefix = if *mode == hir::ParamMode::View {
                    String::new()
                } else {
                    format!("{} ", mode.as_str())
                };
                format!("{}{}", mode_prefix, self.ty_name(*ty))
            })
            .collect();
        format!("({})->{}", params.join(","), self.ty_name(x.result))
    }

    /// The operator set of this domain, as the admission table the body
    /// walk enforces (mirrors the frozen 745/767 accept sets).
    fn op_admitted(&self, op: hir::BinOp) -> bool {
        match self.domain {
            hir::HirDomain::Core => op.in_core_profile(),
            // Scalar: full shared-enum arithmetic + comparison surface the
            // scalar profile admits (sub/div/rem, all four comparisons,
            // same-type eq/ne).
            hir::HirDomain::Scalar => matches!(
                op,
                hir::BinOp::AddI32
                    | hir::BinOp::SubI32
                    | hir::BinOp::MulI32
                    | hir::BinOp::DivI32
                    | hir::BinOp::RemI32
                    | hir::BinOp::LtI32
                    | hir::BinOp::LeI32
                    | hir::BinOp::GtI32
                    | hir::BinOp::GeI32
                    | hir::BinOp::Eq
                    | hir::BinOp::Ne
            ),
            // PLAN-902: the record/text/text-record surfaces carry the
            // full scalar operator set (their verifiers are supersets of
            // the scalar verifier for the compute nodes).
            // PLAN-904: the collections surface carries it too (its
            // verifier is the scalar verifier plus the collections rules).
            hir::HirDomain::Text
            | hir::HirDomain::Record
            | hir::HirDomain::TextRecord
            | hir::HirDomain::Collections => matches!(
                op,
                hir::BinOp::AddI32
                    | hir::BinOp::SubI32
                    | hir::BinOp::MulI32
                    | hir::BinOp::DivI32
                    | hir::BinOp::RemI32
                    | hir::BinOp::LtI32
                    | hir::BinOp::LeI32
                    | hir::BinOp::GtI32
                    | hir::BinOp::GeI32
                    | hir::BinOp::Eq
                    | hir::BinOp::Ne
            ),
            // PLAN-961: the closures domain carries the numeric surface
            // (the numeric source profile binds under it): the legacy i32
            // set plus every typed Num op — semantic legality (float bit
            // ops) was rejected by the HIR verifier before this point.
            hir::HirDomain::Closures => matches!(
                op,
                hir::BinOp::AddI32
                    | hir::BinOp::SubI32
                    | hir::BinOp::MulI32
                    | hir::BinOp::DivI32
                    | hir::BinOp::RemI32
                    | hir::BinOp::LtI32
                    | hir::BinOp::LeI32
                    | hir::BinOp::GtI32
                    | hir::BinOp::GeI32
                    | hir::BinOp::Eq
                    | hir::BinOp::Ne
            ) || matches!(op, hir::BinOp::Num { .. }),
            _ => false,
        }
    }

    /// Walk every function body and admit each local/place/expr/stmt against
    /// the supported surface. Unknown shapes are named rejections — the
    /// view never passes unverified-shape content to a target.
    fn walk_bodies(&self) -> Result<(), A2xReject> {
        for f in &self.functions {
            for local in &f.body.locals {
                resolve_ty(self.module, local.ty)?;
            }
            for place in &f.body.places {
                match place.kind {
                    hir::PlaceKind::Local => {}
                    hir::PlaceKind::FieldPath(_) if self.record_places_admitted() => {}
                    _ => {
                        return Err(A2xReject::unsupported_place(format!(
                            "in `{}`: only whole-local places are in the compute surface",
                            f.name
                        )))
                    }
                }
            }
            for expr in &f.body.exprs {
                self.admit_expr(f, expr)?;
            }
            for block in &f.body.blocks {
                for stmt in &block.stmts {
                    self.admit_stmt(f, stmt)?;
                }
            }
        }
        Ok(())
    }

    fn record_places_admitted(&self) -> bool {
        matches!(
            self.domain,
            hir::HirDomain::Record | hir::HirDomain::TextRecord
        )
    }

    fn admit_expr(&self, f: &FnFacts<'_>, e: &hir::Expr) -> Result<(), A2xReject> {
        match &e.kind {
            hir::ExprKind::ReadLocal { .. } | hir::ExprKind::Constant { .. } => Ok(()),
            // PLAN-961: the typed numeric const and the conversion are
            // closures-domain (numeric profile) surface.
            hir::ExprKind::NumericConst { .. } | hir::ExprKind::Cast { .. }
                if self.domain == hir::HirDomain::Closures =>
            {
                Ok(())
            }
            hir::ExprKind::Binary { op, .. } => {
                if self.op_admitted(*op) {
                    Ok(())
                } else {
                    Err(A2xReject::unsupported_op(format!(
                        "in `{}`: `{}` is outside the {} profile's operator set",
                        f.name,
                        op.as_str(),
                        self.profile
                    )))
                }
            }
            hir::ExprKind::Unary { .. } | hir::ExprKind::Logical { .. } => {
                if self.domain == hir::HirDomain::Scalar
                    || self.text_record_family(self.domain).is_some()
                    || self.domain == hir::HirDomain::Collections
                    || self.domain == hir::HirDomain::Closures
                {
                    Ok(())
                } else {
                    Err(A2xReject::unsupported_op(format!(
                        "in `{}`: unary negation/not and short-circuit logic are scalar-only",
                        f.name
                    )))
                }
            }
            hir::ExprKind::Call { callee, .. } => {
                let target = callee.0 as usize;
                if self.fn_by_def(target).is_some()
                    || self.trace_symbol(target).is_some()
                    || self.external_by_def(target).is_some()
                    || self.foreign_by_def(target).is_some()
                {
                    Ok(())
                } else {
                    Err(A2xReject::unsupported_expr(format!(
                        "in `{}`: call target def #{} is neither an admitted function nor an authorized trace intrinsic",
                        f.name, target
                    )))
                }
            }
            other => {
                // PLAN-902: the record/text/text-record node surfaces.
                // PLAN-904: the collections node surface.
                if let Some(family) = self.text_record_family(self.domain) {
                    if kind_in_family(other, family) {
                        return Ok(());
                    }
                    return Err(A2xReject::unsupported_expr(format!(
                        "in `{}`: expression kind `{}` is outside the {} surface",
                        f.name,
                        kind_name(other),
                        self.profile
                    )));
                }
                if self.domain == hir::HirDomain::Collections {
                    if kind_in_family(other, Family::Collections) {
                        return Ok(());
                    }
                    return Err(A2xReject::unsupported_expr(format!(
                        "in `{}`: expression kind `{}` is outside the {} surface",
                        f.name,
                        kind_name(other),
                        self.profile
                    )));
                }
                Err(A2xReject::unsupported_expr(format!(
                    "in `{}`: expression kind `{}` is outside the core/scalar compute surface",
                    f.name,
                    kind_name(other)
                )))
            }
        }
    }

    fn admit_stmt(&self, f: &FnFacts<'_>, s: &hir::Stmt) -> Result<(), A2xReject> {
        match s {
            hir::Stmt::Let { .. }
            | hir::Stmt::Return { .. }
            | hir::Stmt::If { .. }
            | hir::Stmt::Loop { .. }
            | hir::Stmt::Assign { .. }
            | hir::Stmt::Break { .. }
            | hir::Stmt::Continue { .. } => Ok(()),
            // PLAN-904: the collections unit-effect statement (`xs.push(v)`
            // and friends); its value is verified Unit/copy-discard only.
            hir::Stmt::Eval { .. } if self.domain == hir::HirDomain::Collections => Ok(()),
            other => Err(A2xReject::unsupported_stmt(format!(
                "in `{}`: statement kind `{}` is outside the compute surface",
                f.name,
                stmt_name(other)
            ))),
        }
    }

    /// The record/text family classifier: `None` for non-902 domains.
    fn text_record_family(&self, domain: hir::HirDomain) -> Option<Family> {
        match domain {
            hir::HirDomain::Text => Some(Family::Text),
            hir::HirDomain::Record => Some(Family::Record),
            hir::HirDomain::TextRecord => Some(Family::TextRecord),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Text,
    Record,
    TextRecord,
    /// PLAN-904: the collections surface — the restricted Str face
    /// (literal, `.len()`, clone), record construction/field reads and
    /// borrows, whole-root move, and the List/Variant node set.
    Collections,
}

/// Whether the expression kind belongs to the domain family's verified
/// surface (the frontend verifier already enforces the same sets; this is
/// the target-facing second gate).
fn kind_in_family(k: &hir::ExprKind, family: Family) -> bool {
    let text_kinds = matches!(
        k,
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
    let record_kinds = matches!(
        k,
        hir::ExprKind::RecordCtor { .. }
            | hir::ExprKind::FieldGet { .. }
            | hir::ExprKind::BorrowView { .. }
            | hir::ExprKind::BorrowMut { .. }
    );
    let collections_kinds = matches!(
        k,
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
            | hir::ExprKind::VariantProject { .. }
    );
    match family {
        Family::Text => text_kinds,
        Family::Record => record_kinds,
        Family::TextRecord => {
            text_kinds
                || record_kinds
                // PLAN-939 S06: a named foreign-callback target argument.
                || matches!(k, hir::ExprKind::CallbackRef { .. })
                || matches!(
                    k,
                    hir::ExprKind::RecordClone { .. } | hir::ExprKind::Move { .. }
                )
        }
        // restricted Str face (literal + len + clone), record ctor/field
        // reads/borrows, whole-root move, and the List/Variant set — the
        // wider text surface (slices/char ops/concat) stays out
        Family::Collections => {
            matches!(
                k,
                hir::ExprKind::StrLit { .. }
                    | hir::ExprKind::TextLen { .. }
                    | hir::ExprKind::TextClone { .. }
                    | hir::ExprKind::Move { .. }
            ) || record_kinds
                || collections_kinds
        }
    }
}

fn single_module(bundle: &hir::Bundle) -> Result<&hir::Module, A2xReject> {
    if bundle.modules.len() != 1 {
        return Err(A2xReject::domain(format!(
            "expected exactly one module (source-frontend form), found {}",
            bundle.modules.len()
        )));
    }
    Ok(&bundle.modules[0])
}

fn profile_of(domain: hir::HirDomain) -> Result<&'static str, A2xReject> {
    match domain {
        hir::HirDomain::Core => Ok("auto.source.core-i32.draft"),
        hir::HirDomain::Scalar => Ok("auto.source.scalar-i32.draft"),
        hir::HirDomain::Text => Ok("auto.source.text.draft"),
        hir::HirDomain::Record => Ok("auto.source.record.draft"),
        hir::HirDomain::TextRecord => Ok("auto.source.text-record.draft"),
        // PLAN-904: the collections source domain (List/Variant surface).
        hir::HirDomain::Collections => Ok("auto.source.collections.draft"),
        // PLAN-961: the numeric surface binds under the closures domain —
        // its semantic view carries the numeric operator/const/cast set.
        hir::HirDomain::Closures => Ok("auto.source.numeric.draft"),
        other => Err(A2xReject::domain(format!(
            "supported domains are core/scalar/record/text/text-record/collections/numeric drafts; bundle is {}",
            other.as_str()
        ))),
    }
}

/// Collect admitted functions (free functions and methods). `only_def`
/// (fragment admission) restricts body-carrying functions to the
/// credential's def; all other function/method defs are skipped here (they
/// become externals via `collect_externals`).
fn collect_functions<'m>(
    module: &'m hir::Module,
    only_def: Option<&str>,
) -> Result<Vec<FnFacts<'m>>, A2xReject> {
    let mut functions = Vec::new();
    for (def_index, def) in module.defs.iter().enumerate() {
        let (name, ty, body_id, kind) = match &def.kind {
            hir::DefKind::Function { name, ty, body } => (name, *ty, *body, FnKind::Free),
            hir::DefKind::Method {
                name,
                ty,
                body,
                record,
                receiver,
            } => (
                name,
                *ty,
                *body,
                FnKind::Method {
                    owner: record.0,
                    receiver: *receiver,
                },
            ),
            hir::DefKind::Intrinsic { .. } | hir::DefKind::Foreign { .. } => continue,
        };
        if let Some(want) = only_def {
            if def.id_text != want {
                continue;
            }
        }
        let sig = fn_sig(module, ty)?;
        let body = module
            .bodies
            .get(body_id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "body index out of range"))?;
        let params = param_facts(module, body, sig)?;
        let source_name = source_name_of(name, kind)?;
        functions.push(FnFacts {
            def_index,
            name,
            source_name,
            kind,
            params,
            result: sig_result(module, sig)?,
            origin: (def.span.start as u32, def.span.end as u32),
            body_index: body_id.0,
            body,
        });
    }
    Ok(functions)
}

/// The source-facing name of a def: free functions strip `sf_`; methods
/// (`sm_<Owner>_<name>`) strip `sm_` and keep the owner-qualified
/// `<Owner>_<name>` — same-named methods of different owners stay distinct,
/// mirroring the frozen `m.<Owner>.<name>` lowering identity (PLAN-881).
fn source_name_of<'n>(name: &'n str, kind: FnKind) -> Result<&'n str, A2xReject> {
    match kind {
        FnKind::Free => name.strip_prefix("sf_").ok_or_else(|| {
            A2xReject::new(
                "a2x.internal",
                format!("function def name `{}` lacks the `sf_` source prefix", name),
            )
        }),
        FnKind::Method { .. } => name.strip_prefix("sm_").ok_or_else(|| {
            A2xReject::new(
                "a2x.internal",
                format!("method def name `{}` lacks the `sm_` source prefix", name),
            )
        }),
    }
}

/// Signature-only externals of a fragment (every Function/Method def except
/// the credential's planned def). PLAN-902: the verified param MODES are
/// collected from each callee's own body (scaffold bodies still carry the
/// declaration's param locals — the 881 retained-environment rule: modes
/// are never guessed from a missing body).
fn collect_externals(
    module: &hir::Module,
    planned_def: &str,
) -> Result<Vec<ExternalFn>, A2xReject> {
    let mut externals = Vec::new();
    for (def_index, def) in module.defs.iter().enumerate() {
        let (name, ty, body_id, kind) = match &def.kind {
            hir::DefKind::Function { name, ty, body } => (name, *ty, *body, FnKind::Free),
            hir::DefKind::Method {
                name,
                ty,
                body,
                record,
                receiver,
            } => (
                name,
                *ty,
                *body,
                FnKind::Method {
                    owner: record.0,
                    receiver: *receiver,
                },
            ),
            hir::DefKind::Intrinsic { .. } | hir::DefKind::Foreign { .. } => continue,
        };
        if def.id_text == planned_def {
            continue;
        }
        let sig = fn_sig(module, ty)?;
        let source_name = match kind {
            FnKind::Free => name.strip_prefix("sf_").unwrap_or(name).to_string(),
            FnKind::Method { .. } => name.strip_prefix("sm_").unwrap_or(name).to_string(),
        };
        let params = sig
            .params
            .iter()
            .map(|t| resolve_ty(module, *t))
            .collect::<Result<Vec<Ty>, A2xReject>>()?;
        // verified param modes from the callee's own param locals, in
        // signature order
        let mut modes = vec![hir::ParamMode::View; params.len()];
        if let Some(body) = module.bodies.get(body_id.0 as usize) {
            let mut param_locals: Vec<&hir::LocalDef> = body
                .locals
                .iter()
                .filter(|l| matches!(l.role, hir::LocalRole::Param))
                .collect();
            param_locals.sort_by_key(|l| l.index.unwrap_or(u32::MAX));
            for (l, slot) in param_locals.iter().zip(modes.iter_mut()) {
                *slot = l.param_mode;
            }
        }
        externals.push(ExternalFn {
            def_index,
            source_name,
            kind,
            params,
            modes,
            result: sig_result(module, sig)?,
        });
    }
    Ok(externals)
}

/// Collect nominal record declarations with resolved field types and the
/// resource classification (transitive owned-Str closure, iterative —
/// record nesting is unbounded, recursion is verifier-forbidden but the
/// walk stays defensive).
fn collect_records(module: &hir::Module) -> Result<Vec<RecordFacts>, A2xReject> {
    let mut records = Vec::new();
    for (idx, td) in module.types.iter().enumerate() {
        if let hir::TypeKind::Record(shape) = &td.kind {
            let mut fields = Vec::with_capacity(shape.fields.len());
            for f in &shape.fields {
                fields.push(FieldFacts {
                    name: f.name.clone(),
                    ty: resolve_ty(module, f.ty)?,
                });
            }
            records.push(RecordFacts {
                type_index: idx as u32,
                name: td
                    .id_text
                    .strip_prefix("t_rec_")
                    .unwrap_or(&td.id_text)
                    .to_string(),
                fields,
                resource: false,
            });
        }
    }
    // resource classification fixpoint: a record is a resource root when
    // any field is StrOwned or (transitively) a resource record.
    let mut resource: std::collections::BTreeMap<u32, bool> = std::collections::BTreeMap::new();
    for rec in &records {
        resource.entry(rec.type_index).or_insert(false);
    }
    loop {
        let mut changed = false;
        for rec in &records {
            if resource[&rec.type_index] {
                continue;
            }
            let is_res = rec.fields.iter().any(|f| match f.ty {
                Ty::StrOwned => true,
                Ty::Record(id) => resource.get(&id).copied().unwrap_or(false),
                _ => false,
            });
            if is_res {
                resource.insert(rec.type_index, true);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for rec in &mut records {
        rec.resource = resource[&rec.type_index];
    }
    Ok(records)
}

/// Collect nominal variant declarations with resolved arm payloads
/// (PLAN-904). Arms are in declaration order; the arm INDEX is the
/// runtime discriminant.
fn collect_variants(module: &hir::Module) -> Result<Vec<VariantFacts>, A2xReject> {
    let mut variants = Vec::new();
    for (idx, td) in module.types.iter().enumerate() {
        if let hir::TypeKind::Variant { arms } = &td.kind {
            let mut facts = Vec::with_capacity(arms.len());
            for a in arms {
                facts.push(VariantArmFacts {
                    name: a.name.clone(),
                    payload: match a.payload {
                        Some(p) => Some(resolve_ty(module, p)?),
                        None => None,
                    },
                });
            }
            variants.push(VariantFacts {
                type_index: idx as u32,
                name: td
                    .id_text
                    .strip_prefix("t_variant_")
                    .unwrap_or(&td.id_text)
                    .to_string(),
                arms: facts,
            });
        }
    }
    Ok(variants)
}

fn collect_trace_defs(
    module: &hir::Module,
    bundle: &hir::Bundle,
) -> Result<Vec<(usize, &'static str)>, A2xReject> {
    let trace_authorized = bundle.requires.iter().any(|c| c == TRACE_CAPABILITY);
    let mut trace_defs = Vec::new();
    for (def_index, def) in module.defs.iter().enumerate() {
        if let hir::DefKind::Intrinsic { symbol, .. } = &def.kind {
            let sym = match symbol.as_str() {
                "hir.test.mark_a" => "hir.test.mark_a",
                "hir.test.mark_b" => "hir.test.mark_b",
                other => {
                    return Err(A2xReject::unsupported_expr(format!(
                        "intrinsic `{}` is outside the compute surface",
                        other
                    )))
                }
            };
            if !trace_authorized {
                return Err(A2xReject::unauthorized_intrinsic(format!(
                    "trace intrinsic `{}` present without capability `{}`",
                    sym, TRACE_CAPABILITY
                )));
            }
            trace_defs.push((def_index, sym));
        }
    }
    Ok(trace_defs)
}

fn fn_sig(module: &hir::Module, ty: hir::TypeId) -> Result<&hir::FnSig, A2xReject> {
    let def = module
        .types
        .get(ty.0 as usize)
        .ok_or_else(|| A2xReject::new("a2x.internal", "type index out of range"))?;
    match &def.kind {
        hir::TypeKind::Function(sig) => Ok(sig),
        other => Err(A2xReject::unsupported_type(format!(
            "function def has non-function type `{}`",
            type_kind_name(other)
        ))),
    }
}

fn sig_result(module: &hir::Module, sig: &hir::FnSig) -> Result<Ty, A2xReject> {
    resolve_ty(module, sig.result)
}

/// Collect the function's parameters from its body's param locals, in
/// signature order (`LocalDef.index`), and cross-check against the
/// signature's arity/types.
fn param_facts(
    module: &hir::Module,
    body: &hir::Body,
    sig: &hir::FnSig,
) -> Result<Vec<ParamFacts>, A2xReject> {
    let mut params: Vec<&hir::LocalDef> = body
        .locals
        .iter()
        .filter(|l| matches!(l.role, hir::LocalRole::Param))
        .collect();
    params.sort_by_key(|l| l.index.unwrap_or(u32::MAX));
    if params.len() != sig.params.len() {
        return Err(A2xReject::new(
            "a2x.internal",
            format!(
                "param locals ({}) disagree with signature arity ({})",
                params.len(),
                sig.params.len()
            ),
        ));
    }
    let mut facts = Vec::with_capacity(params.len());
    for (l, sig_ty) in params.iter().zip(sig.params.iter()) {
        let lt = resolve_ty(module, l.ty)?;
        let st = resolve_ty(module, *sig_ty)?;
        if lt != st {
            return Err(A2xReject::new(
                "a2x.internal",
                format!("param `{}` type disagrees with signature", l.name),
            ));
        }
        facts.push(ParamFacts {
            name: l.name.clone(),
            ty: lt,
            mode: l.param_mode,
        });
    }
    Ok(facts)
}

/// Resolve a use-site type to the supported value types; everything else is
/// a named rejection.
fn resolve_ty(module: &hir::Module, ty: hir::TypeUse) -> Result<Ty, A2xReject> {
    match ty {
        hir::TypeUse::Builtin(hir::BuiltinType::I32) => Ok(Ty::I32),
        // PLAN-961 numeric surface.
        hir::TypeUse::Builtin(hir::BuiltinType::I8) => Ok(Ty::I8),
        hir::TypeUse::Builtin(hir::BuiltinType::U8) => Ok(Ty::U8),
        hir::TypeUse::Builtin(hir::BuiltinType::U32) => Ok(Ty::U32),
        hir::TypeUse::Builtin(hir::BuiltinType::I64) => Ok(Ty::I64),
        hir::TypeUse::Builtin(hir::BuiltinType::U64) => Ok(Ty::U64),
        hir::TypeUse::Builtin(hir::BuiltinType::F32) => Ok(Ty::F32),
        hir::TypeUse::Builtin(hir::BuiltinType::F64) => Ok(Ty::F64),
        hir::TypeUse::Builtin(hir::BuiltinType::Bool) => Ok(Ty::Bool),
        hir::TypeUse::Builtin(hir::BuiltinType::StrSlice) => Ok(Ty::Str),
        hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => Ok(Ty::StrOwned),
        hir::TypeUse::Builtin(hir::BuiltinType::Char) => Ok(Ty::Char),
        hir::TypeUse::Builtin(hir::BuiltinType::Unit) => Ok(Ty::Unit),
        hir::TypeUse::Ref(id) => match module.types.get(id.0 as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::I32) => Ok(Ty::I32),
            // PLAN-961 numeric surface.
            Some(hir::TypeKind::I8) => Ok(Ty::I8),
            Some(hir::TypeKind::U8) => Ok(Ty::U8),
            Some(hir::TypeKind::U32) => Ok(Ty::U32),
            Some(hir::TypeKind::I64) => Ok(Ty::I64),
            Some(hir::TypeKind::U64) => Ok(Ty::U64),
            Some(hir::TypeKind::F32) => Ok(Ty::F32),
            Some(hir::TypeKind::F64) => Ok(Ty::F64),
            Some(hir::TypeKind::Bool) => Ok(Ty::Bool),
            Some(hir::TypeKind::Str) => Ok(Ty::Str),
            Some(hir::TypeKind::StrOwned) => Ok(Ty::StrOwned),
            Some(hir::TypeKind::Char) => Ok(Ty::Char),
            Some(hir::TypeKind::Record(_)) => Ok(Ty::Record(id.0)),
            // PLAN-904: structural lists and nominal variants
            Some(hir::TypeKind::List { .. }) => Ok(Ty::List(id.0)),
            Some(hir::TypeKind::Variant { .. }) => Ok(Ty::Variant(id.0)),
            Some(other) => Err(A2xReject::unsupported_type(format!(
                "type `{}` is outside the supported value types",
                type_kind_name(other)
            ))),
            None => Err(A2xReject::new("a2x.internal", "type index out of range")),
        },
    }
}

fn kind_name(k: &hir::ExprKind) -> &'static str {
    match k {
        hir::ExprKind::Unit => "Unit",
        hir::ExprKind::CallbackRef { .. } => "CallbackRef",
        hir::ExprKind::CallClosure { .. } => "CallClosure",
        hir::ExprKind::CreateClosure { .. } => "CreateClosure",
        hir::ExprKind::StrLit { .. } => "StrLit",
        hir::ExprKind::CharLit { .. } => "CharLit",
        hir::ExprKind::TextLen { .. } => "TextLen",
        hir::ExprKind::TextLenBytes { .. } => "TextLenBytes",
        hir::ExprKind::TextCharAt { .. } => "TextCharAt",
        hir::ExprKind::TextSlice { .. } => "TextSlice",
        hir::ExprKind::TextConcat { .. } => "TextConcat",
        hir::ExprKind::TextClone { .. } => "TextClone",
        hir::ExprKind::ClosureClone { .. } => "ClosureClone",
        hir::ExprKind::TextToOwned { .. } => "TextToOwned",
        hir::ExprKind::TextToBorrowed { .. } => "TextToBorrowed",
        hir::ExprKind::CharToCodepoint { .. } => "CharToCodepoint",
        hir::ExprKind::CharFromCodepoint { .. } => "CharFromCodepoint",
        hir::ExprKind::RecordCtor { .. } => "RecordCtor",
        hir::ExprKind::FieldGet { .. } => "FieldGet",
        hir::ExprKind::BorrowView { .. } => "BorrowView",
        hir::ExprKind::BorrowMut { .. } => "BorrowMut",
        hir::ExprKind::RecordClone { .. } => "RecordClone",
        hir::ExprKind::Move { .. } => "Move",
        hir::ExprKind::ListNew { .. } => "ListNew",
        hir::ExprKind::ListLen { .. } => "ListLen",
        hir::ExprKind::ListPush { .. } => "ListPush",
        hir::ExprKind::ListInsert { .. } => "ListInsert",
        hir::ExprKind::ListGet { .. } => "ListGet",
        hir::ExprKind::ListSet { .. } => "ListSet",
        hir::ExprKind::ListRemove { .. } => "ListRemove",
        hir::ExprKind::ListClear { .. } => "ListClear",
        hir::ExprKind::VariantCtor { .. } => "VariantCtor",
        hir::ExprKind::VariantGuardTest { .. } => "VariantGuardTest",
        hir::ExprKind::VariantProject { .. } => "VariantProject",
        _ => "non-core-scalar",
    }
}

fn stmt_name(s: &hir::Stmt) -> &'static str {
    match s {
        hir::Stmt::Drop { .. } => "Drop",
        hir::Stmt::Eval { .. } => "Eval",
        _ => "non-core-scalar",
    }
}

fn type_kind_name(k: &hir::TypeKind) -> &'static str {
    match k {
        hir::TypeKind::I32 => "i32",
        hir::TypeKind::Bool => "bool",
        hir::TypeKind::Str => "str",
        hir::TypeKind::StrOwned => "Str",
        hir::TypeKind::Char => "char",
        hir::TypeKind::Function(_) => "fn",
        hir::TypeKind::Closure(_) => "closure",
        hir::TypeKind::Record(_) => "record",
        hir::TypeKind::List { .. } => "list",
        hir::TypeKind::Variant { .. } => "variant",
        hir::TypeKind::I8 => "i8",
        hir::TypeKind::U8 => "u8",
        hir::TypeKind::U32 => "u32",
        hir::TypeKind::I64 => "i64",
        hir::TypeKind::U64 => "u64",
        hir::TypeKind::F32 => "f32",
        hir::TypeKind::F64 => "f64",
    }
}
