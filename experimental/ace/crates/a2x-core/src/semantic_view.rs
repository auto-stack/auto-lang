//! Target-neutral semantic view over verified core/scalar HIR (plan §2:
//! "真实core/scalar Auto源码→既有受检前端/Checked HIR→共享最小语义view").
//!
//! The view is READ-ONLY over the checked bundle: it re-checks the two
//! supported domains' admission surface (defense in depth on top of the
//! frontend, which already enforces the same sets), collects per-function
//! declaration facts (signature, params, result) and exposes borrowed body
//! structure plus source-order evaluation facts. It produces no target
//! code and knows no target — that is the Rust provider's job (a2x-rust).
//!
//! Evaluation-order contract (plan §4): `ExprKind::Call` carries
//! `eval_args` (SOURCE order, evaluate exactly once) plus `bindings`
//! (param ← arg placement). The view exposes both unchanged; a target
//! never re-guesses argument order from the binding map.

use ace_core::hir;
use ace_core::source::SourceParts;
use ace_core::verify::{CheckedFunction, CheckedModule};
use std::collections::BTreeSet;

use crate::A2xReject;

/// The two value types of the core/scalar compute surface.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ty {
    I32,
    Bool,
}

impl Ty {
    pub fn as_str(self) -> &'static str {
        match self {
            Ty::I32 => "i32",
            Ty::Bool => "bool",
        }
    }
}

/// One declared parameter of a function (name + target-neutral type).
#[derive(Clone, Debug)]
pub struct ParamFacts {
    pub name: String,
    pub ty: Ty,
}

/// Declaration-level facts of one user function plus its borrowed verified
/// body. `def_index` is the module def-table index (stable identity for
/// call targets within this view).
#[derive(Clone, Debug)]
pub struct FnFacts<'a> {
    pub def_index: usize,
    /// The HIR-internal def name (`sf_<source>` for source functions).
    pub name: &'a str,
    /// The SOURCE name (the `sf_` prefix the frontend adds to free
    /// functions stripped back off; entry selection and diagnostics use
    /// this).
    pub source_name: &'a str,
    pub params: Vec<ParamFacts>,
    pub result: Ty,
    /// Byte span of the function's def in the source it was compiled from
    /// (source-map v1; plan §5.3 — spans travel with the target artifacts
    /// so diagnostics map back to the original `.at`).
    pub origin: (u32, u32),
    pub body: &'a hir::Body,
}

/// Signature-only facts of a function whose BODY is not part of this view —
/// the scaffold declarations a single-body emission carries for the other
/// functions (PLAN-828). Calls to externals admit by signature; the target
/// names them by def identity and the session layer (T-04) assembles the
/// closure.
#[derive(Clone, Debug)]
pub struct ExternalFn {
    pub def_index: usize,
    pub source_name: String,
    pub params: Vec<Ty>,
    pub result: Ty,
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
    /// Authorized trace intrinsic defs present: def_index → symbol.
    trace_defs: Vec<(usize, &'static str)>,
    /// Document-level capabilities (trimmed to the ones A2X cares about).
    pub capabilities: Vec<String>,
}

const TRACE_CAPABILITY: &str = "hir.test.trace.v1";

impl<'a> SemanticView<'a> {
    /// Build the view from a verified module, admitting only the core/scalar
    /// surface. `CheckedModule` construction is private to the verifier — a
    /// view can only be built over genuinely verified content.
    pub fn from_checked(checked: &'a CheckedModule) -> Result<Self, A2xReject> {
        let domain = checked.domain();
        let profile = profile_of(domain)?;
        let bundle = checked.bundle();
        let module = single_module(bundle)?;
        let functions = collect_functions(module, None)?;
        let trace_defs = collect_trace_defs(module, bundle)?;
        let view = SemanticView {
            module,
            domain,
            profile,
            functions,
            externals: Vec::new(),
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
        credential: &CheckedFunction,
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
        let trace_defs = collect_trace_defs(module, bundle)?;
        let view = SemanticView {
            module,
            domain,
            profile,
            functions,
            externals: collect_externals(module, credential.def_id_text())?,
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

    /// The user function owning `def_index`, if any.
    pub fn fn_by_def(&self, def_index: usize) -> Option<&FnFacts<'a>> {
        self.functions.iter().find(|f| f.def_index == def_index)
    }

    /// The signature-only external owning `def_index`, if any.
    pub fn external_by_def(&self, def_index: usize) -> Option<&ExternalFn> {
        self.externals.iter().find(|f| f.def_index == def_index)
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

    /// Select the entry function by explicit source name: it must exist, be
    /// a zero-parameter function with an i32 result (plan §5.1).
    pub fn entry(&self, name: &str) -> Result<&FnFacts<'a>, A2xReject> {
        let f = self
            .functions
            .iter()
            .find(|f| f.source_name == name)
            .ok_or_else(|| {
                A2xReject::entry(format!("no function named `{}` in this module", name))
            })?;
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
            _ => false,
        }
    }

    /// Walk every function body and admit each local/place/expr/stmt against
    /// the core/scalar surface. Unknown shapes are named rejections — the
    /// view never passes unverified-shape content to a target.
    fn walk_bodies(&self) -> Result<(), A2xReject> {
        for f in &self.functions {
            for local in &f.body.locals {
                resolve_ty(self.module, local.ty)?;
            }
            for place in &f.body.places {
                if !matches!(place.kind, hir::PlaceKind::Local) {
                    return Err(A2xReject::unsupported_place(format!(
                        "in `{}`: only whole-local places are in the compute surface",
                        f.name
                    )));
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

    fn admit_expr(&self, f: &FnFacts<'_>, e: &hir::Expr) -> Result<(), A2xReject> {
        match &e.kind {
            hir::ExprKind::ReadLocal { .. } | hir::ExprKind::Constant { .. } => Ok(()),
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
                if self.domain == hir::HirDomain::Scalar {
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
                {
                    Ok(())
                } else {
                    Err(A2xReject::unsupported_expr(format!(
                        "in `{}`: call target def #{} is neither an admitted function nor an authorized trace intrinsic",
                        f.name, target
                    )))
                }
            }
            other => Err(A2xReject::unsupported_expr(format!(
                "in `{}`: expression kind `{}` is outside the core/scalar compute surface",
                f.name,
                kind_name(other)
            ))),
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
            other => Err(A2xReject::unsupported_stmt(format!(
                "in `{}`: statement kind `{}` is outside the core/scalar compute surface",
                f.name,
                stmt_name(other)
            ))),
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
        other => Err(A2xReject::domain(format!(
            "supported domains are core-i32-draft/scalar-i32-draft; bundle is {}",
            other.as_str()
        ))),
    }
}

/// Collect admitted functions. `only_def` (fragment admission) restricts
/// body-carrying functions to the credential's def; all other Function
/// defs are skipped here (they become externals via `collect_externals`).
fn collect_functions<'m>(
    module: &'m hir::Module,
    only_def: Option<&str>,
) -> Result<Vec<FnFacts<'m>>, A2xReject> {
    let mut functions = Vec::new();
    for (def_index, def) in module.defs.iter().enumerate() {
        if let hir::DefKind::Function { name, ty, body } = &def.kind {
            if let Some(want) = only_def {
                if def.id_text != want {
                    continue;
                }
            }
            let sig = fn_sig(module, *ty)?;
            let body = module
                .bodies
                .get(body.0 as usize)
                .ok_or_else(|| A2xReject::new("a2x.internal", "body index out of range"))?;
            let params = param_facts(module, body, &sig)?;
            let source_name = name.strip_prefix("sf_").ok_or_else(|| {
                A2xReject::new(
                    "a2x.internal",
                    format!("function def name `{}` lacks the `sf_` source prefix", name),
                )
            })?;
            functions.push(FnFacts {
                def_index,
                name,
                source_name,
                params,
                result: sig_result(module, &sig)?,
                origin: (def.span.start as u32, def.span.end as u32),
                body,
            });
        }
    }
    Ok(functions)
}

/// Signature-only externals of a fragment (every Function def except the
/// credential's planned def).
fn collect_externals(
    module: &hir::Module,
    planned_def: &str,
) -> Result<Vec<ExternalFn>, A2xReject> {
    let mut externals = Vec::new();
    for (def_index, def) in module.defs.iter().enumerate() {
        if let hir::DefKind::Function { name, ty, .. } = &def.kind {
            if def.id_text == planned_def {
                continue;
            }
            let sig = fn_sig(module, *ty)?;
            let source_name = name.strip_prefix("sf_").unwrap_or(name).to_string();
            let params = sig
                .params
                .iter()
                .map(|t| resolve_ty(module, *t))
                .collect::<Result<Vec<Ty>, A2xReject>>()?;
            externals.push(ExternalFn {
                def_index,
                source_name,
                params,
                result: sig_result(module, &sig)?,
            });
        }
    }
    Ok(externals)
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
                        "intrinsic `{}` is outside the core/scalar compute surface",
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
        });
    }
    Ok(facts)
}

/// Resolve a use-site type to the two supported value types; everything
/// else is a named rejection.
fn resolve_ty(module: &hir::Module, ty: hir::TypeUse) -> Result<Ty, A2xReject> {
    match ty {
        hir::TypeUse::Builtin(hir::BuiltinType::I32) => Ok(Ty::I32),
        hir::TypeUse::Builtin(hir::BuiltinType::Bool) => Ok(Ty::Bool),
        hir::TypeUse::Ref(id) => match module.types.get(id.0 as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::I32) => Ok(Ty::I32),
            Some(hir::TypeKind::Bool) => Ok(Ty::Bool),
            Some(other) => Err(A2xReject::unsupported_type(format!(
                "type `{}` is outside {{i32, bool}}",
                type_kind_name(other)
            ))),
            None => Err(A2xReject::new("a2x.internal", "type index out of range")),
        },
        other => Err(A2xReject::unsupported_type(format!(
            "builtin type `{:?}` is outside {{i32, bool}}",
            other
        ))),
    }
}

fn kind_name(k: &hir::ExprKind) -> &'static str {
    match k {
        hir::ExprKind::Unit => "Unit",
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
    }
}
