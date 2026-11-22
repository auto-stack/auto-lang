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
use ace_core::verify::{CheckedFunction, CheckedModule, ModuleGlobalsPlan};
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
    /// PLAN-972 (T-03): an owned closure VALUE — the payload is the module
    /// type-table index of its `TypeKind::Closure(FnSig)` entry (signatures
    /// are interned by index; `Ty` stays `Copy`).
    Closure(u32),
    /// PLAN-961 numeric surface: the signed/unsigned/float value types.
    /// `byte`/`uint` resolve to `U8`/`U32` at the source front.
    I8,
    U8,
    U32,
    I64,
    U64,
    F32,
    F64,
    /// PLAN-1115 (T-03, from PLAN-964): the zero-width null value type —
    /// only Nil-typed equality is admitted at the source.
    Nil,
    /// PLAN-1115 (T-03, from PLAN-964): a checked value with an erased,
    /// independently recoverable type tag.
    Dyn,
    /// PLAN-1115 (T-03, from PLAN-964): a shared mutable string-keyed
    /// object map; the payload is the module type-table index of the
    /// `Object { elem }` entry (the resolve freezes the homogeneous entry
    /// value type; entries may be nil when `elem = Nil`).
    Object(u32),
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
            Ty::Closure(_) => "closure",
            Ty::Nil => "nil",
            Ty::Dyn => "dyn",
            Ty::Object(_) => "object",
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
/// PLAN-981（D-981-MODULE-GLOBAL-A2R）：一个模块全局的已验模型（代码生成用）。
#[derive(Debug)]
pub struct GlobalModel {
    /// 限定符号（`module.global.get/set:<sym>` 的 `<sym>`）。
    pub symbol: String,
    /// 作者显示名。
    pub name: String,
    /// 值类型（目标中性）。
    pub ty: Ty,
    /// `var`（可写）还是 `const`。
    pub mutable: bool,
}

#[derive(Debug)]
pub struct SemanticView<'a> {
    module: &'a hir::Module,
    pub domain: hir::HirDomain,
    /// The source profile identity this domain corresponds to.
    pub profile: &'static str,
    age_package: bool,
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
    /// PLAN-968: scheduler-surface intrinsics (`__sched_*` and the console
    /// print spellings admitted in scheduler modules).
    sched_defs: Vec<(usize, String)>,
    /// PLAN-1041: platform-operation intrinsic defs present (the common
    /// stdlib output source's reserved calls): def_index → symbol
    /// (`auto.platform.io.*` / `auto.platform.mem.*`). The target lowers
    /// map these to runtime stubs (ports on A2PY, runtime helpers on
    /// A2R/A2C); errors are deterministic negative i32 codes, never traps.
    platform_defs: Vec<(usize, String)>,
    /// PLAN-981（A2C 宿主面）：AGE gfx **宿主车道** op 的 def（def_index →
    /// symbol，HostAlgorithm/HostIdentity 16 op——`age_resources` 受检表为
    /// 唯一事实源）。符号面进视图后由**目标车道**各自处置：A2C 发射
    /// `a2x_gfx_*` 宿主 extern；A2R 在自身 lowering 具名拒（AC/C 宿主车道
    /// 才是消费方）。
    gfx_defs: Vec<(usize, String)>,
    /// PLAN-1107 T-02: py-lane extraction intrinsics present (def_index →
    /// symbol: `__py_f64_bits` / `__py_staged_text`). Admitted only when the
    /// shared `py_ops` plan is present; a target lowers the OPERATIONS via
    /// the plan, so this list exists to keep the checked bodies admissible
    /// rather than to expose an AV encoding.
    py_defs: Vec<(usize, String)>,
    /// PLAN-981（D-981-MODULE-GLOBAL-A2R）：module.global.get/set 的 def
    /// （索引、global 符号、是否 set）。仅当 facts plan 在场（AGE 包前门）
    /// 时参与放行；碎片视图恒为空。
    global_defs: Vec<(usize, String, bool)>,
    /// PLAN-981：绑定器钉进 CheckedModule 的全局事实（可变性/init 链）。
    globals_plan: Option<&'a ModuleGlobalsPlan>,
    /// PLAN-1107 T-02: the shared verified Python-operation facts (the py
    /// lane's front door pins them onto `CheckedModule`). Present only for
    /// modules compiled through the py lane; targets consume the facts and
    /// never recover Python operations from frame text or names.
    py_ops: Option<&'a ace_core::py_ops::PyOpPlan>,
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
        Self::from_checked_module(checked, false)
    }

    /// Admit the checked expressions used by the AGE package fragment.
    pub fn from_checked_age_package(checked: &'a CheckedModule) -> Result<Self, A2xReject> {
        if checked.domain() == hir::HirDomain::Closures
            && checked.required_capabilities().iter().any(|cap| cap == "av.aggregates/1")
        {
            return Self::from_checked_module(checked, true);
        }
        if checked.domain() != hir::HirDomain::TextRecord {
            return Err(A2xReject::domain("AGE package view requires checked text-record HIR"));
        }
        Self::from_checked_module(checked, true)
    }

    fn from_checked_module(
        checked: &'a CheckedModule,
        age_package: bool,
    ) -> Result<Self, A2xReject> {
        let domain = checked.domain();
        // PLAN-962: the aggregates surface binds under the closures domain
        // — the document capability distinguishes it from numeric.
        let profile = if domain == hir::HirDomain::Closures
            && checked
                .required_capabilities()
                .iter()
                .any(|c| c == "av.aggregates/1")
        {
            "auto.source.aggregates.draft"
        } else {
            profile_of(domain)?
        };
        let bundle = checked.bundle();
        let module = single_module(bundle)?;
        let functions = collect_functions(module, None)?;
        let records = collect_records(module)?;
        let variants = collect_variants(module)?;
        let (trace_defs, platform_defs, sched_defs, global_defs, gfx_defs, py_defs) =
            collect_trace_defs(module, bundle, checked.py_ops().is_some())?;
        // PLAN-981：全局访问器只在**事实在场**时放行（AGE 包前门钉的
        // plan）——无 plan 的路径维持旧的 compute-surface 拒绝。
        let globals_plan = checked.module_globals();
        if globals_plan.is_none() && !global_defs.is_empty() {
            return Err(A2xReject::unsupported_expr(format!(
                "module.global accessors are present without the package globals plan                  (this path does not carry module const/var facts)"
            )));
        }
        if let Some(plan) = globals_plan {
            for (_, sym, _) in &global_defs {
                if !plan.facts.iter().any(|f| &f.symbol == sym) {
                    return Err(A2xReject::unsupported_expr(format!(
                        "module.global accessor `{sym}` has no binder fact (unknown global)"
                    )));
                }
            }
        }
        let view = SemanticView {
            module,
            domain,
            profile,
            age_package,
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
            sched_defs,
            platform_defs,
            gfx_defs,
            py_defs,
            global_defs,
            globals_plan,
            // PLAN-1107 T-02: the py lane pins the shared operation facts onto
            // the verified module; absent for every other compile path.
            py_ops: checked.py_ops(),
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
        let (trace_defs, platform_defs, sched_defs, global_defs, gfx_defs, py_defs) =
            collect_trace_defs(module, bundle, false)?;
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
            age_package: false,
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
            sched_defs,
            platform_defs,
            gfx_defs,
            py_defs,
            global_defs,
            globals_plan: None,
            // PLAN-1107 T-02: a fragment view carries no whole-module py-op
            // plan (the fragment credential does not cover it).
            py_ops: None,
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

    /// PLAN-1115 (T-03): the frozen homogeneous entry value type of an
    /// `Object { elem }` type-table entry.
    pub fn object_elem(&self, type_index: u32) -> Result<Ty, A2xReject> {
        match self.module.types.get(type_index as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::Object { elem }) => resolve_ty(self.module, *elem),
            _ => Err(A2xReject::new(
                "a2x.internal",
                format!("type #{type_index} is not an object entry"),
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
    /// PLAN-972 (T-03): the interned signature of a closure type entry.
    pub fn closure_sig(&self, idx: u32) -> Option<&hir::FnSig> {
        match self.module.types.get(idx as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::Closure(sig)) => Some(sig),
            _ => None,
        }
    }

    pub fn fn_by_def(&self, def_index: usize) -> Option<&FnFacts<'a>> {
        self.functions.iter().find(|f| f.def_index == def_index)
    }

    pub fn def_name_for_diagnostic(&self, def_index: usize) -> Option<&str> {
        self.module.defs.get(def_index).map(hir::Def::name_for_diagnostic)
    }

    /// The signature-only external owning `def_index`, if any.
    pub fn external_by_def(&self, def_index: usize) -> Option<&ExternalFn> {
        self.externals.iter().find(|f| f.def_index == def_index)
    }

    /// PLAN-1107 T-02: the shared verified Python-operation plan of this
    /// module (the py lane's front-door facts: imports, calls, attribute /
    /// index / binary operations and explicit extractions with their
    /// source-order operand slots and spans). `None` for modules that carry
    /// no Python operations. Targets consume this plan instead of decoding
    /// the AV `p967.frame/1` wire text.
    pub fn py_ops(&self) -> Option<&'a ace_core::py_ops::PyOpPlan> {
        self.py_ops
    }

    /// PLAN-1107 T-02: the py-lane extraction intrinsics present in the
    /// admitted bodies (`__py_f64_bits` / `__py_staged_text`), with their def
    /// indices. Present only together with `py_ops()`.
    pub fn py_defs(&self) -> &[(usize, String)] {
        &self.py_defs
    }

    /// PLAN-1107 T-02: the py-lane intrinsic symbol at `def_index`.
    pub fn py_symbol(&self, def_index: usize) -> Option<&str> {
        self.py_defs
            .iter()
            .find(|(i, _)| *i == def_index)
            .map(|(_, s)| s.as_str())
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

    pub fn is_text_find(&self, def_index: usize) -> bool {
        matches!(self.module.defs.get(def_index).map(|def| &def.kind),
            Some(hir::DefKind::Intrinsic { symbol, .. }) if symbol == "__ace_text_find")
    }

    /// PLAN-1115 (T-05): the checked text-scan intrinsic family's symbol
    /// (the assembled stdlib's text helpers call them).
    pub fn text_scan_symbol(&self, def_index: usize) -> Option<&'static str> {
        let sym = match self.module.defs.get(def_index).map(|def| &def.kind) {
            Some(hir::DefKind::Intrinsic { symbol, .. }) => symbol.as_str(),
            _ => return None,
        };
        match sym {
            "__ace_text_find" => Some("__ace_text_find"),
            "__ace_text_trim" => Some("__ace_text_trim"),
            "__ace_text_trim_len" => Some("__ace_text_trim_len"),
            "__ace_text_upper" => Some("__ace_text_upper"),
            _ => None,
        }
    }

    /// PLAN-981：`def_index` 是否是 module.global.get/set 访问器；是则返回
    /// （global 符号, 是否 set）。
    pub fn global_accessor(&self, def_index: usize) -> Option<(&str, bool)> {
        self.global_defs
            .iter()
            .find(|(d, _, _)| *d == def_index)
            .map(|(_, sym, is_set)| (sym.as_str(), *is_set))
    }

    /// Whether the view was admitted for checked AGE package emission.
    pub fn is_age_package(&self) -> bool {
        self.age_package
    }

    /// PLAN-981（D-981-MODULE-GLOBAL-A2R）：本视图的全局模型（Rust 代码生
    /// 成消费）——每 global：限定符号、值类型（get 结果型；仅写未读时取 set
    /// 形参型）、`var` 可变性、显示名。
    pub fn globals_model(&self) -> Result<Vec<GlobalModel>, A2xReject> {
        let mut out: Vec<GlobalModel> = Vec::new();
        for (_, sym, is_set) in &self.global_defs {
            if out.iter().any(|g| g.symbol == *sym) {
                continue;
            }
            let ty = self.global_value_ty(sym, *is_set)?;
            let mutable = self
                .globals_plan
                .and_then(|p| p.facts.iter().find(|f| &f.symbol == sym))
                .map(|f| f.mutable);
            let Some(mutable) = mutable else {
                return Err(A2xReject::unsupported_expr(format!(
                    "module.global accessor `{sym}` has no binder fact (unknown global)"
                )));
            };
            let name = self
                .globals_plan
                .and_then(|p| p.facts.iter().find(|f| &f.symbol == sym))
                .map(|f| f.name.clone())
                .unwrap_or_else(|| sym.clone());
            out.push(GlobalModel {
                symbol: sym.clone(),
                name,
                ty,
                mutable,
            });
        }
        Ok(out)
    }

    /// PLAN-981：init 链（模块 init 函数符号，依赖序）——Rust 目标在入口前执行。
    pub fn global_init_fns(&self) -> Vec<&str> {
        self.globals_plan
            .map(|p| p.init_fns.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    fn global_value_ty(&self, sym: &str, is_set: bool) -> Result<Ty, A2xReject> {
        let entry = self
            .global_defs
            .iter()
            .find(|(_, s, set)| s == sym && *set == is_set);
        let Some((def_index, _, _)) = entry else {
            return Err(A2xReject::new(
                "a2x.internal",
                format!("global `{sym}` accessor def missing"),
            ));
        };
        let def = &self.module.defs[*def_index];
        let ty = match &def.kind {
            hir::DefKind::Intrinsic { ty, .. } => *ty,
            _ => {
                return Err(A2xReject::new(
                    "a2x.internal",
                    format!("global `{sym}` def is not an intrinsic"),
                ))
            }
        };
        let sig = fn_sig(self.module, ty)?;
        if is_set {
            let p = sig.params.first().copied().ok_or_else(|| {
                A2xReject::new(
                    "a2x.internal",
                    format!("global set `{sym}` sig has no param"),
                )
            })?;
            resolve_ty(self.module, p)
        } else {
            resolve_ty(self.module, sig.result)
        }
    }

    /// PLAN-968: the scheduler-surface intrinsic symbol for a def.
    pub fn sched_symbol(&self, def_index: usize) -> Option<&str> {
        self.sched_defs
            .iter()
            .find(|(d, _)| *d == def_index)
            .map(|(_, s)| s.as_str())
    }

    /// PLAN-968: whether any scheduler intrinsics are present (targets
    /// append the scheduler runtime only then).
    pub fn has_sched_ops(&self) -> bool {
        self.sched_defs
            .iter()
            .any(|(_, s)| s.starts_with("__sched_"))
    }

    /// PLAN-1116 (T-02): whether any console print spellings are present
    /// (they bind to the scheduler runtime's print helpers — a program can
    /// print without any scheduler operation, e.g. a bare generator loop).
    pub fn has_console_ops(&self) -> bool {
        self.sched_defs
            .iter()
            .any(|(_, s)| s.starts_with("auto.console."))
    }

    /// PLAN-968: the resolved (param, result) types of a scheduler
    /// intrinsic's fn-type def (the import seam carries the exact shape).
    pub fn sched_sig_types(&self, def_index: usize) -> Option<Result<(Vec<Ty>, Ty), A2xReject>> {
        let def = self.module.defs.get(def_index)?;
        if let hir::DefKind::Intrinsic { ty, .. } = &def.kind {
            if let hir::TypeKind::Function(sig) = &self.module.types.get(ty.0 as usize)?.kind {
                let mut params = Vec::with_capacity(sig.params.len());
                for p in &sig.params {
                    match resolve_ty(self.module, *p) {
                        Ok(t) => params.push(t),
                        Err(e) => return Some(Err(e)),
                    }
                }
                let result = resolve_ty(self.module, sig.result);
                return Some(match result {
                    Ok(r) => Ok((params, r)),
                    Err(e) => Err(e),
                });
            }
        }
        None
    }

    /// PLAN-1041: whether any platform-operation intrinsics are present
    /// (targets append their platform runtime stubs only then).
    pub fn has_platform_ops(&self) -> bool {
        !self.platform_defs.is_empty()
    }

    /// PLAN-1041: the platform-operation symbol an intrinsic def carries,
    /// if any (call targets reaching the target lowers' runtime stubs).
    pub fn platform_symbol(&self, def_index: usize) -> Option<&str> {
        self.platform_defs
            .iter()
            .find(|(d, _)| *d == def_index)
            .map(|(_, s)| s.as_str())
    }

    /// PLAN-981（A2C 宿主面）：模块是否携带 AGE gfx 宿主车道 op
    ///（运行时声明页只在携带时追加）。
    pub fn has_gfx_ops(&self) -> bool {
        !self.gfx_defs.is_empty()
    }

    /// PLAN-981（A2C 宿主面）：该 intrinsic def 携带的 gfx 宿主 symbol
    ///（按 `age_resources` 受检表收面）。
    pub fn gfx_symbol(&self, def_index: usize) -> Option<&str> {
        self.gfx_defs
            .iter()
            .find(|(d, _)| *d == def_index)
            .map(|(_, s)| s.as_str())
    }

    /// PLAN-981（A2C 宿主面）：模块携带的**去重** gfx 宿主 symbol 集
    ///（运行时声明页按它逐条发 extern 原型）。
    pub fn gfx_symbols(&self) -> Vec<&str> {
        let mut syms: Vec<&str> = self.gfx_defs.iter().map(|(_, s)| s.as_str()).collect();
        syms.sort();
        syms.dedup();
        syms
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
            hir::HirDomain::Text | hir::HirDomain::Record | hir::HirDomain::Collections => {
                matches!(
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
                )
            }
            // PLAN-981（D-981-AGE-F32-SURFACE，A2R 同源腿）：text-record 族
            // （AGE 包/组件面的实际域）同样携带 typed `Num` 面——但只放 **f32**
            // （AGE 的值面只有 f32；f64/宽整型仍不在本面，由前门先行拒绝）。
            // 语义合法性（float 位算等）已由 HIR verifier 在前门判定。
            hir::HirDomain::TextRecord => {
                matches!(
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
                ) || matches!(
                    op,
                    hir::BinOp::Num {
                        ty: hir::BuiltinType::F32,
                        ..
                    }
                )
            }
            // PLAN-961: the closures domain carries the numeric surface
            // (the numeric source profile binds under it): the legacy i32
            // set plus every typed Num op — semantic legality (float bit
            // ops) was rejected by the HIR verifier before this point.
            hir::HirDomain::Closures => {
                matches!(
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
                ) || matches!(op, hir::BinOp::Num { .. })
            }
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
        ) || self.profile == "auto.source.aggregates.draft"
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
            // PLAN-981（D-981-AGE-F32-SURFACE，A2R 同源腿）：f32 字面量在
            // text-record 族（AGE 包/组件面的实际 HIR 域）也进 compute surface。
            // 此前该豁免只给 Closures（numeric profile），AGE 的 f32 模型初值/
            // 运算右操作数到 A2R 被具名拒（AV 车道不经过本视图，故 AV 先绿）。
            // 只放 F32 常量：`f64`/整型宽字面量仍不在本面；转换（Cast）不放——
            // AGE 作者面本来拼写不出 `as`，生成的脱糖代码也不产 Cast。
            hir::ExprKind::NumericConst {
                ty: hir::BuiltinType::F32,
                ..
            } if self.text_record_family(self.domain).is_some() => Ok(()),
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
                    || self.is_text_find(target)
                    // PLAN-1115 (T-05): the whole checked text-scan family.
                    || self.text_scan_symbol(target).is_some()
                    || self.external_by_def(target).is_some()
                    || self.foreign_by_def(target).is_some()
                    // PLAN-981: module.global accessors reach the targets'
                    // static lowering (admitted only with the facts plan).
                    || self.global_accessor(target).is_some()
                    // PLAN-1041: platform-operation intrinsics reach the
                    // target lowers' runtime stubs.
                    || self.platform_symbol(target).is_some()
                    // PLAN-981（A2C 宿主面）：AGE gfx 宿主 op 到达目标车道
                    //（A2C 发射 a2x_gfx_* extern；A2R 在自身 lowering 具名拒）。
                    || self.gfx_symbol(target).is_some()
                    // PLAN-968: scheduler-surface intrinsics reach the
                    // targets' scheduler runtimes.
                    || self.sched_symbol(target).is_some()
                    // PLAN-1107 T-02: py-lane extraction intrinsics appear in
                    // the checked py bodies; the target lowers the OPERATION
                    // from `py_ops()`, so the intrinsic itself is admitted
                    // here only to keep the frozen body admissible.
                    || self.py_symbol(target).is_some()
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
                    if kind_in_family(other, family)
                        || (self.age_package && matches!(other, hir::ExprKind::TextDisplay { .. }))
                    {
                        return Ok(());
                    }
                    return Err(A2xReject::unsupported_expr(format!(
                        "in `{}`: expression kind `{}` is outside the {} surface",
                        f.name,
                        kind_name(other),
                        self.profile
                    )));
                }
                // PLAN-962: the aggregates surface (closures domain + the
                // av.aggregates/1 capability) admits the FULL collections
                // family (records + collections kinds — tuples ride
                // records, Box/grid ride lists).
                if self.domain == hir::HirDomain::Collections
                    || self.profile == "auto.source.aggregates.draft"
                {
                    if kind_in_family(other, Family::Collections)
                        || (self.age_package && matches!(other, hir::ExprKind::RecordClone { .. }))
                    {
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
            // PLAN-946: the aggregates packages lane carries the same
            // unit-effect calls (its checked HIR is the closures-superset
            // domain, so the profile string carries the admission — mirrors
            // the expression gate above). PLAN-965's Fail rides the same
            // guard.
            hir::Stmt::Eval { .. } | hir::Stmt::Fail { .. }
                if self.domain == hir::HirDomain::Collections
                    || self.profile == "auto.source.aggregates.draft" =>
            {
                Ok(())
            }
            hir::Stmt::Drop { .. } if self.profile == "auto.source.aggregates.draft" => Ok(()),
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
            // PLAN-975 B1: char materialization joins the text surface.
            | hir::ExprKind::TextFromChar { .. }
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
        // wider text surface (slices/char ops) stays out. PLAN-964: the
        // FStr interpolation chain (TextConcat + TextDisplay) joins the
        // collections face — the aggregates/closures FStr slice.
        Family::Collections => {
            matches!(
                k,
                hir::ExprKind::StrLit { .. }
                    | hir::ExprKind::TextLen { .. }
                    | hir::ExprKind::TextClone { .. }
                    | hir::ExprKind::TextConcat { .. }
                    | hir::ExprKind::TextDisplay { .. }
                    | hir::ExprKind::RecordClone { .. }
                    | hir::ExprKind::Move { .. }
                    // PLAN-968: the scheduler face's f-string chain
                    // borrows owned parts (the legacy brace dialect's
                    // interpolation reads state locals).
                    | hir::ExprKind::TextToBorrowed { .. }
                    // PLAN-1115 (T-02): the zero-width unit value of a
                    // Unit-result function body (the ordinary language
                    // face emits it as the implicit/explicit return
                    // operand). The 904/962 frozen goldens never carry a
                    // standalone Unit expr, so their channels are inert.
                    | hir::ExprKind::Unit
                    // PLAN-1115 (T-03, PLAN-964): the ordinary dynamic
                    // surface — nil literal, dyn pack/recover, and the
                    // shared object map node set (language lane only in
                    // practice; the frozen 904/962 goldens never carry
                    // them).
                    | hir::ExprKind::NilLit
                    | hir::ExprKind::DynCast { .. }
                    | hir::ExprKind::ObjectCtor { .. }
                    | hir::ExprKind::ObjectGet { .. }
                    | hir::ExprKind::ObjectSet { .. }
                    // PLAN-1115 (T-05): the independent-copy node of the
                    // ordinary face's container clones (1088 copy-
                    // independence family).
                    | hir::ExprKind::OwnedClone { .. }
            ) || record_kinds
                || collections_kinds
                // PLAN-972 (T-03): the assembled stdlib's text/char
                // expression surface (scalar-indexed slices/char ops) and
                // capture-free closure values join the collections family —
                // the same widening the packages-aggregates-ct lane admits
                // upstream; capturing closures stay rejected downstream.
                || text_kinds
                || matches!(
                    k,
                    hir::ExprKind::CreateClosure { .. }
                        | hir::ExprKind::CallClosure { .. }
                        | hir::ExprKind::TextDisplay { .. }
                )
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
    allow_py_lane: bool,
) -> Result<
    (
        Vec<(usize, &'static str)>,
        Vec<(usize, String)>,
        Vec<(usize, String)>,
        Vec<(usize, String, bool)>,
        Vec<(usize, String)>,
        Vec<(usize, String)>,
    ),
    A2xReject,
> {
    let trace_authorized = bundle.requires.iter().any(|c| c == TRACE_CAPABILITY);
    let mut trace_defs = Vec::new();
    let mut platform_defs = Vec::new();
    let mut sched_defs = Vec::new();
    // PLAN-1107 T-02: the py lane's lowering intrinsics. They belong to the
    // AV `.to(float)`/`.to(str)` extraction encodings; a source-generating
    // target consumes the shared `py_ops` facts (the operation identity) and
    // lowers them natively. Admitted ONLY when the py lane armed the module's
    // operation plan — a non-python module can never smuggle them in.
    let mut py_defs: Vec<(usize, String)> = Vec::new();
    // PLAN-981（A2C 宿主面）：AGE gfx 宿主车道 op（HostAlgorithm/HostIdentity，
    // `age_resources::is_gfx_host_symbol` 为唯一事实源）。符号面在此收进
    // 视图；目标车道各自处置（A2C 发射宿主 extern，A2R 具名拒）。
    let mut gfx_defs = Vec::new();
    // PLAN-981（D-981-MODULE-GLOBAL-A2R）：模块级 const/var 的访问器
    // intrinsic——收集为全局模型（是否放行由 facts plan 在场性决定，见
    // `SemanticView::new` 的 admission 检查）。
    let mut global_defs: Vec<(usize, String, bool)> = Vec::new();
    for (def_index, def) in module.defs.iter().enumerate() {
        if let hir::DefKind::Intrinsic { symbol, .. } = &def.kind {
            if symbol == "__ace_text_find" && bundle.domain == hir::HirDomain::Closures {
                continue;
            }
            if let Some(rest) = symbol.strip_prefix("module.global.get:") {
                global_defs.push((def_index, rest.to_string(), false));
                continue;
            }
            if let Some(rest) = symbol.strip_prefix("module.global.set:") {
                global_defs.push((def_index, rest.to_string(), true));
                continue;
            }
            if symbol.starts_with("auto.platform.") {
                // PLAN-1041: platform-operation intrinsics — the target
                // runtimes implement them (negative i32 error codes).
                platform_defs.push((def_index, symbol.clone()));
                continue;
            }
            // PLAN-981（A2C 宿主面）：AGE gfx 宿主车道 op——按受检表收符号面。
            if ace_core::source::age_resources::is_gfx_host_symbol(symbol) {
                gfx_defs.push((def_index, symbol.clone()));
                continue;
            }
            // PLAN-968: the scheduler surface's intrinsics (task actor
            // operations + console print in scheduler modules).
            if symbol.starts_with("__sched_") || symbol.starts_with("auto.console.") {
                sched_defs.push((def_index, symbol.clone()));
                continue;
            }
            // PLAN-1107 T-02: py-lane extraction intrinsics, only with the op plan
            // present (see `py_defs` above).
            if allow_py_lane && matches!(symbol.as_str(), "__py_f64_bits" | "__py_staged_text") {
                py_defs.push((def_index, symbol.clone()));
                continue;
            }
            // PLAN-1115 (T-05): the checked text-scan intrinsics — the
            // assembled stdlib's text helpers call them; the language lane
            // lowers them onto the runtime text helpers.
            if matches!(
                symbol.as_str(),
                "__ace_text_find" | "__ace_text_trim" | "__ace_text_trim_len"
                    | "__ace_text_upper"
            ) {
                trace_defs.push((def_index, "hir.text.scan"));
                continue;
            }
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
    Ok((trace_defs, platform_defs, sched_defs, global_defs, gfx_defs, py_defs))
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
            // PLAN-972 (T-03): closure values — the payload is the type
            // table index (the signature is interned, not copied).
            Some(hir::TypeKind::Closure(_)) => Ok(Ty::Closure(id.0)),
            Some(hir::TypeKind::Record(_)) => Ok(Ty::Record(id.0)),
            // PLAN-904: structural lists and nominal variants
            Some(hir::TypeKind::List { .. }) => Ok(Ty::List(id.0)),
            Some(hir::TypeKind::Variant { .. }) => Ok(Ty::Variant(id.0)),
            // PLAN-1115 (T-03, PLAN-964): the ordinary dynamic surface.
            Some(hir::TypeKind::Nil) => Ok(Ty::Nil),
            Some(hir::TypeKind::Dyn) => Ok(Ty::Dyn),
            Some(hir::TypeKind::Object { .. }) => Ok(Ty::Object(id.0)),
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
        hir::ExprKind::NumericConst { .. } => "NumericConst",
        hir::ExprKind::Cast { .. } => "Cast",
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
        hir::ExprKind::TextFromChar { .. } => "TextFromChar",
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
        hir::ExprKind::NilLit => "NilLit",
        hir::ExprKind::DynCast { .. } => "DynCast",
        hir::ExprKind::ObjectCtor { .. } => "ObjectCtor",
        hir::ExprKind::ObjectGet { .. } => "ObjectGet",
        hir::ExprKind::ObjectSet { .. } => "ObjectSet",
        hir::ExprKind::OwnedClone { .. } => "OwnedClone",
        _ => "non-core-scalar",
    }
}

fn stmt_name(s: &hir::Stmt) -> &'static str {
    match s {
        hir::Stmt::Drop { .. } => "Drop",
        hir::Stmt::Eval { .. } => "Eval",
        hir::Stmt::Fail { .. } => "Fail",
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
        hir::TypeKind::Nil => "nil",
        hir::TypeKind::Dyn => "dyn",
        hir::TypeKind::Object { .. } => "object",
        hir::TypeKind::I8 => "i8",
        hir::TypeKind::U8 => "u8",
        hir::TypeKind::U32 => "u32",
        hir::TypeKind::I64 => "i64",
        hir::TypeKind::U64 => "u64",
        hir::TypeKind::F32 => "f32",
        hir::TypeKind::F64 => "f64",
    }
}

/// Returns true when every fall-through path out of `id` ends in a control
/// terminator (`return`, `fail`, `break`, `continue`), so any code appended
/// after a loop body built from `id` would be unreachable.
pub fn block_always_terminates(body: &hir::Body, id: hir::BlockId) -> bool {
    body.blocks[id.0 as usize]
        .stmts
        .last()
        .is_some_and(|s| match s {
            hir::Stmt::Return { .. }
            | hir::Stmt::Fail { .. }
            | hir::Stmt::Break { .. }
            | hir::Stmt::Continue { .. } => true,
            hir::Stmt::If {
                then,
                els: Some(eb),
                ..
            } => block_always_terminates(body, *then) && block_always_terminates(body, *eb),
            _ => false,
        })
}
