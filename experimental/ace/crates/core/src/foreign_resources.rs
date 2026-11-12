//! PLAN-939 CA-D3: the SINGLE core authority for foreign RESOURCE
//! declarations — how a resource signature (plan §5.1 matrix) projects
//! onto the checked source type system, and the per-call ownership /
//! exclusivity facts the backends consume without re-deriving.
//!
//! Every reader of a resource declaration (cold/warm signature tables,
//! the type checker, the HIR adapter, the decl-env page and the verifier)
//! goes through [`source_sig`] / [`source_sig_readonly`]; no site keeps
//! its own lane→type table (the 938 `Bool→bool else i32` closure is the
//! scalar special case of this mapping).
//!
//! Frozen T-02 source mapping:
//! * `I32`/`Bool`             → `int`/`bool`;
//! * `ViewText`               → `str` view argument;
//! * `Rec{rec, role}`         → the nominal source record `rec` whose field
//!                               closure EXACTLY matches the declared one
//!                               (same names, same order, same kinds; same
//!                               shape under another name is rejected).
//!                               `Mut` AND `Out` require the `.mut` call-site
//!                               borrow (both write the caller's storage; the
//!                               bridge shadows and commits only on success —
//!                               Mut copies the old bytes in, Out starts from
//!                               a zeroed shadow the provider must not read);
//!                               `Const` takes the plain view argument;
//! * `Handle{..}`             → `int` (physical i64, bridge-checked);
//! * `Callback{..}`           → a NAMED top-level source function
//!                               `fn cb(ctx int, item int) int` referenced
//!                               by name in the callback lane position
//!                               (the thin trampoline: no first-class
//!                               closure value — the text-record domain has
//!                               none; the bridge emits a static
//!                               i64→i32→i64 adapter), the i32 ctx seed
//!                               travels as the next ordinary `int`.
//!
//! Results: pure/fallible value lanes reach the source as `int` (`Bool` as
//! `bool`); wide physical lanes are checked-converted by the bridge.

use crate::hir;
use crate::source::resolve::{ClosureFnType, ResolvedRecord, VT};
use ace_foreign_contract::manual::{SourceDecl, SourceResultKind};
use ace_foreign_contract::resource::{FieldKind, ResKind, ViewRole};

/// Diagnostic code for resource-declaration/source mismatches.
pub const CODE_RECORD: &str = "source.foreign.record";
/// Diagnostic code for an unsupported (non-matrix) resource shape.
pub const CODE_SHAPE: &str = "source.foreign.resource-shape";

/// The resource-aware source-visible signature of one foreign declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignSourceSig {
    pub params: Vec<VT>,
    /// Per-param: the argument must be an explicit `.mut` borrow
    /// (`Rec{role: Mut | Out}`). Scalar/view/handle/callback params are
    /// `false`.
    pub mut_params: Vec<bool>,
    pub result: VT,
}

/// The one callback shape the source may hand to a foreign callback lane.
pub fn callback_type() -> ClosureFnType {
    ClosureFnType {
        params: vec![VT::I32, VT::I32],
        result: VT::I32,
    }
}

fn field_vt(kind: FieldKind) -> Option<VT> {
    match kind {
        FieldKind::I32 => Some(VT::I32),
        FieldKind::Bool => Some(VT::Bool),
        _ => None,
    }
}

/// Nominal record lookup with EXACT field-closure match (plan §5.1:
/// same layout under a different nominal name is a rejection; a
/// different field set under the same name is a rejection).
pub fn record_index(
    rec: &str,
    fields: &[(String, FieldKind)],
    records: &[ResolvedRecord],
) -> Result<usize, (&'static str, String)> {
    let Some(i) = records.iter().position(|r| r.name == rec) else {
        return Err((
            CODE_RECORD,
            format!("foreign record view `{rec}` names no source record type"),
        ));
    };
    let r = &records[i];
    if r.fields.len() != fields.len() {
        return Err((
            CODE_RECORD,
            format!(
                "foreign record view `{rec}` declares {} fields but the source record has {}",
                fields.len(),
                r.fields.len()
            ),
        ));
    }
    for (src, (name, kind)) in r.fields.iter().zip(fields) {
        let Some(want) = field_vt(*kind) else {
            return Err((
                CODE_SHAPE,
                format!(
                    "foreign record view `{rec}` field `{name}` kind {} is outside the checked source record domain",
                    kind.spell()
                ),
            ));
        };
        if src.name != *name || src.vt != want {
            return Err((
                CODE_RECORD,
                format!(
                    "foreign record view `{rec}` field closure differs from the source record (`{}`: {} vs declared `{name}`: {})",
                    src.name,
                    src.vt.as_str(),
                    want.as_str()
                ),
            ));
        }
    }
    Ok(i)
}

fn result_vt(decl: &SourceDecl) -> VT {
    match decl.source_result_kind() {
        SourceResultKind::Bool => VT::Bool,
        SourceResultKind::I32 => VT::I32,
        // Not reachable for admitted source declarations (exactly one
        // value out is required); defensively spelled as the status lane.
        SourceResultKind::UnitStatus => VT::I32,
    }
}

/// Resource-aware signature; interns the callback closure type when a
/// callback lane is present (append-only — indexes already handed out stay
/// valid, matching the PLAN-909 interning contract).
pub fn source_sig(
    decl: &SourceDecl,
    records: &[ResolvedRecord],
    function_types: &mut Vec<ClosureFnType>,
) -> Result<ForeignSourceSig, (&'static str, String)> {
    let Some(res) = decl.resource() else {
        return Ok(scalar_sig(decl));
    };
    let mut params = Vec::with_capacity(res.params.len());
    let mut mut_params = Vec::with_capacity(res.params.len());
    for p in &res.params {
        let (vt, is_mut) = match p {
            ResKind::I32 => (VT::I32, false),
            ResKind::Bool => (VT::Bool, false),
            ResKind::ViewText => (VT::Str, false),
            ResKind::Rec { rec, fields, role } => (
                VT::Rec(record_index(rec, fields, records)?),
                matches!(role, ViewRole::Mut | ViewRole::Out),
            ),
            ResKind::Handle { .. } => (VT::I32, false),
            // PLAN-939 S07: typed-peer-only wide lanes have NO checked
            // source mapping (the contract validator rejects them for
            // source sigs; this arm keeps the core authority exhaustive).
            ResKind::BufViewI64 | ResKind::I64 => {
                return Err((
                    CODE_SHAPE,
                    "typed-peer-only resource lane has no checked-source mapping".into(),
                ));
            }
            ResKind::Callback { .. } => {
                let sig = callback_type();
                let idx = match function_types.iter().position(|s| *s == sig) {
                    Some(i) => i,
                    None => {
                        function_types.push(sig);
                        function_types.len() - 1
                    }
                };
                (VT::Function(idx), false)
            }
        };
        params.push(vt);
        mut_params.push(is_mut);
    }
    Ok(ForeignSourceSig {
        params,
        mut_params,
        result: result_vt(decl),
    })
}

/// PLAN-939 S06: true when the declaration's resource parameter at `lane`
/// is a callback lane (callers without a foreign-contract dependency ask
/// the core authority instead of re-deriving).
pub fn lane_is_callback(decl: &SourceDecl, lane: usize) -> bool {
    decl.resource()
        .map(|r| lane < r.params.len() && matches!(r.params[lane], ResKind::Callback { .. }))
        .unwrap_or(false)
}

/// Read-only variant for consumers that must not grow the type tables
/// (type checker, HIR adapter, verifier): the callback closure type must
/// already be interned by the signature pass.
pub fn source_sig_readonly(
    decl: &SourceDecl,
    records: &[ResolvedRecord],
    function_types: &[ClosureFnType],
) -> Result<ForeignSourceSig, (&'static str, String)> {
    let Some(res) = decl.resource() else {
        return Ok(scalar_sig(decl));
    };
    let mut params = Vec::with_capacity(res.params.len());
    let mut mut_params = Vec::with_capacity(res.params.len());
    for p in &res.params {
        let (vt, is_mut) = match p {
            ResKind::I32 => (VT::I32, false),
            ResKind::Bool => (VT::Bool, false),
            ResKind::ViewText => (VT::Str, false),
            ResKind::Rec { rec, fields, role } => (
                VT::Rec(record_index(rec, fields, records)?),
                matches!(role, ViewRole::Mut | ViewRole::Out),
            ),
            ResKind::Handle { .. } => (VT::I32, false),
            // PLAN-939 S07: typed-peer-only wide lanes have NO checked
            // source mapping (the contract validator rejects them for
            // source sigs; this arm keeps the core authority exhaustive).
            ResKind::BufViewI64 | ResKind::I64 => {
                return Err((
                    CODE_SHAPE,
                    "typed-peer-only resource lane has no checked-source mapping".into(),
                ));
            }
            ResKind::Callback { .. } => {
                let sig = callback_type();
                let Some(idx) = function_types.iter().position(|s| *s == sig) else {
                    return Err((
                        CODE_SHAPE,
                        format!(
                            "foreign callback lane of `{}` has no interned closure signature",
                            decl.name
                        ),
                    ));
                };
                (VT::Function(idx), false)
            }
        };
        params.push(vt);
        mut_params.push(is_mut);
    }
    Ok(ForeignSourceSig {
        params,
        mut_params,
        result: result_vt(decl),
    })
}

/// The 938 scalar mapping (the resource-free special case).
pub fn scalar_sig(decl: &SourceDecl) -> ForeignSourceSig {
    let vt = |k: ace_foreign_contract::LaneKind| {
        if k == ace_foreign_contract::LaneKind::Bool {
            VT::Bool
        } else {
            VT::I32
        }
    };
    ForeignSourceSig {
        params: decl.params().iter().copied().map(vt).collect(),
        mut_params: vec![false; decl.params().len()],
        result: vt(decl.result()),
    }
}

/// Resource lane classification used by backends when marshalling the
/// argument at position `pos` (None for scalar declarations).
pub fn param_kind(decl: &SourceDecl, pos: usize) -> Option<&ResKind> {
    decl.resource().and_then(|r| r.params.get(pos))
}

// ---------------------------------------------------------------------------
// PLAN-939 T-04 (§5.2): the UNIQUE per-body foreign resource credential
// plan. Computed by the core verifier from the checked declarations and
// the call sites; carried on the CheckedModule and exposed through the
// semantic view. Backends consume it for every OWNERSHIP-bearing fact of
// a foreign call — who borrows what (view extent), which lanes are
// exclusive (shadow/commit), which handle credentials cross in/out, and
// which callback lanes invoke which named targets — and never re-derive
// those facts. Physical MARSHALLING (private representation conversion)
// stays the backends' own job per the plan §2 split.
// ---------------------------------------------------------------------------

/// The credential rows of ONE foreign call site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignCallCredential {
    /// The foreign def index in the owning module.
    pub def: u32,
    /// The reviewed provider symbol.
    pub symbol: String,
    /// Borrowed view lanes (`ViewText` and const `Rec` views):
    /// (lane, owner local). `None` owner = a static literal (immortal)
    /// or a call-materialized temp — the consumer extent is the call by
    /// construction. The owner's storage outlives the WHOLE call (V-02:
    /// owned temps live to the last consumer; the text plan schedules
    /// frees at exits, structurally after every call).
    pub view_borrows: Vec<(u32, Option<u32>)>,
    /// Exclusive lanes (`Rec` Mut/Out views): the adapter shadows and
    /// commits once on success — a failure leaves every caller byte
    /// untouched (V-13; the sig's `atomic` promise is checked at
    /// admission for exactly these lanes).
    pub exclusive: Vec<u32>,
    /// Handle credentials crossing IN: (lane, owning provider). The
    /// provider runtime's ledger owns identity/generation (§5.4); the
    /// plan pins the provider tag each lane expects so a mismatched
    /// credential is a controlled rejection, never a silent rebind.
    pub handles_in: Vec<(u32, String)>,
    /// A fresh handle credential is CREATED by this call (a pure
    /// handle-return): the new credential's provider (V-03 owned
    /// returns; the provider lease to its real last release is the
    /// runtime's, T-08).
    pub owned_out: Option<String>,
    /// Callback lanes: (lane, named target def) — strictly synchronous
    /// per-call (noescape is a sig admission requirement); the target is
    /// a top-level source function, never a stored value.
    pub callbacks: Vec<(u32, u32)>,
    /// The fallible ctx binding symbol (the 938 context lane), if the
    /// declaration names one. The symbol is a module-level static —
    /// borrow returns are outside this package's matrix (§5.2 clause
    /// recorded as a frozen boundary: no foreign entry returns a
    /// borrow).
    pub ctx_symbol: Option<String>,
    /// The sig's failure-atomicity promise (shadow/commit discipline of
    /// the exclusive lanes).
    pub atomic: bool,
}

/// The per-bundle foreign resource credential plan: (module, body) →
/// per-call credentials keyed by the CALL expression id. Present only
/// for text-record bundles that actually contain resource foreign calls.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ForeignResourcePlan {
    pub bodies: std::collections::BTreeMap<
        (usize, u32),
        std::collections::BTreeMap<u32, ForeignCallCredential>,
    >,
}

impl ForeignResourcePlan {
    pub fn is_empty(&self) -> bool {
        self.bodies.values().all(|m| m.is_empty())
    }
}

/// Derive the owner local of one view-lane argument, when the argument
/// names a place (a local read or a borrow of a local base). Literals
/// and constructor temps have no owner local — their extent is the call.
fn view_owner(body: &hir::Body, arg: u32) -> Option<u32> {
    match &body.exprs[arg as usize].kind {
        hir::ExprKind::ReadLocal { local } => Some(local.0),
        hir::ExprKind::BorrowMut { local } => Some(local.0),
        hir::ExprKind::BorrowView { base } => match &body.exprs[base.0 as usize].kind {
            hir::ExprKind::ReadLocal { local } => Some(local.0),
            _ => None,
        },
        _ => None,
    }
}

/// Compute one body's per-call credentials. Only calls whose callee is a
/// FOREIGN def with a resource sig produce rows; scalar foreign calls
/// (938) and ordinary calls produce nothing (their facts live on the 938
/// scalar chain / the ordinary verifier).
pub fn body_foreign_plan(
    m: &hir::Module,
    body: &hir::Body,
) -> std::collections::BTreeMap<u32, ForeignCallCredential> {
    use ace_foreign_contract::resource::ResourceForm;
    let mut out = std::collections::BTreeMap::new();
    for (ei, e) in body.exprs.iter().enumerate() {
        let hir::ExprKind::Call {
            callee, bindings, ..
        } = &e.kind
        else {
            continue;
        };
        let Some(hir::DefKind::Foreign { declaration, .. }) =
            m.defs.get(callee.0 as usize).map(|d| &d.kind)
        else {
            continue;
        };
        let Some(res) = declaration.resource() else {
            continue;
        };
        let arg_of = |lane: usize| -> Option<u32> {
            bindings
                .iter()
                .find(|b| b.param as usize == lane)
                .map(|b| b.arg)
        };
        let mut cred = ForeignCallCredential {
            def: callee.0,
            symbol: declaration.function.symbol.clone(),
            view_borrows: Vec::new(),
            exclusive: Vec::new(),
            handles_in: Vec::new(),
            owned_out: None,
            callbacks: Vec::new(),
            ctx_symbol: declaration.context_symbol.clone(),
            atomic: res.atomic,
        };
        for (lane, p) in res.params.iter().enumerate() {
            let lane = lane as u32;
            match p {
                ResKind::ViewText => {
                    let owner = arg_of(lane as usize).and_then(|a| view_owner(body, a));
                    cred.view_borrows.push((lane, owner));
                }
                ResKind::Rec { role, .. } => match role {
                    ViewRole::Const => {
                        let owner = arg_of(lane as usize).and_then(|a| view_owner(body, a));
                        cred.view_borrows.push((lane, owner));
                    }
                    ViewRole::Mut | ViewRole::Out => {
                        cred.exclusive.push(lane);
                    }
                },
                ResKind::Handle { provider } => {
                    cred.handles_in.push((lane, provider.clone()));
                }
                ResKind::Callback { .. } => {
                    if let Some(a) = arg_of(lane as usize) {
                        if let hir::ExprKind::CallbackRef { def } = &body.exprs[a as usize].kind {
                            cred.callbacks.push((lane, def.0));
                        }
                    }
                }
                ResKind::I32 | ResKind::Bool => {}
                // Typed-peer-only lanes never reach a checked source
                // module (admission rejects them before verification).
                ResKind::BufViewI64 | ResKind::I64 => {}
            }
        }
        if let ResourceForm::Pure {
            ret: ace_foreign_contract::resource::ResOut::Handle { provider },
        } = &res.form
        {
            cred.owned_out = Some(provider.clone());
        }
        out.insert(ei as u32, cred);
    }
    out
}

/// Compute the whole bundle's foreign resource plan; `None` when no body
/// contains a resource foreign call (plain 782 modules keep their exact
/// previous shape).
pub fn bundle_foreign_plan(bundle: &hir::Bundle) -> Option<ForeignResourcePlan> {
    let mut plan = ForeignResourcePlan::default();
    for (mi, m) in bundle.modules.iter().enumerate() {
        for (bi, body) in m.bodies.iter().enumerate() {
            let rows = body_foreign_plan(m, body);
            if !rows.is_empty() {
                plan.bodies.insert((mi, bi as u32), rows);
            }
        }
    }
    if plan.is_empty() {
        None
    } else {
        Some(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom_text::Span;
    use crate::source::resolve::ResolvedField;
    use ace_foreign_contract::resource::{ResOut, ResourceForm, ResourceSig, RESOURCE_SIG_FORMAT};
    use ace_foreign_contract::{ErrorModel, FnFact, FnForm, LaneKind};

    fn decl(params: Vec<ResKind>, form: ResourceForm, atomic: bool, noescape: bool) -> SourceDecl {
        let fallible = matches!(form, ResourceForm::Fallible { .. });
        SourceDecl {
            name: "t939".into(),
            provider: "r939".into(),
            logical: "r939/v1".into(),
            content_version: "v1".into(),
            review_id: "review".into(),
            header_digest: "a".repeat(64),
            physical_table_digest: "b".repeat(64),
            context_symbol: None,
            function: FnFact {
                symbol: "t939".into(),
                form: FnForm::Pure {
                    params: vec![],
                    ret: LaneKind::I32,
                },
                error_model: if fallible {
                    Some(ErrorModel {
                        kind: "status-i32".into(),
                        codes: vec![(-1, "fail".into())],
                    })
                } else {
                    None
                },
                semantic_ref: None,
                resource: Some(ResourceSig {
                    format: RESOURCE_SIG_FORMAT.into(),
                    params,
                    form,
                    atomic,
                    noescape,
                    typed_peer: false,
                }),
            },
            effects: vec!["checked".into()],
            capabilities: vec!["foreign.scalar".into(), "foreign.resource".into()],
        }
    }

    fn counter() -> ResolvedRecord {
        ResolvedRecord {
            name: "Counter".into(),
            name_span: Span { start: 0, end: 0 },
            fields: vec![ResolvedField {
                name: "value".into(),
                vt: VT::I32,
                span: Span { start: 0, end: 0 },
            }],
            methods: vec![],
        }
    }

    #[test]
    fn view_text_maps_to_str_and_result_i32() {
        let d = decl(
            vec![ResKind::ViewText],
            ResourceForm::Pure { ret: ResOut::U64 },
            false,
            false,
        );
        let s = source_sig(&d, &[], &mut vec![]).unwrap();
        assert_eq!(s.params, vec![VT::Str]);
        assert_eq!(s.result, VT::I32);
        assert_eq!(s.mut_params, vec![false]);
    }

    #[test]
    fn rec_mut_requires_exact_nominal_closure() {
        let fields = vec![("value".to_string(), FieldKind::I32)];
        let d = decl(
            vec![
                ResKind::Rec {
                    rec: "Counter".into(),
                    fields: fields.clone(),
                    role: ViewRole::Mut,
                },
                ResKind::I32,
            ],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::I32),
            },
            true,
            false,
        );
        let s = source_sig(&d, &[counter()], &mut vec![]).unwrap();
        assert_eq!(s.params, vec![VT::Rec(0), VT::I32]);
        assert_eq!(s.mut_params, vec![true, false]);
        // same shape, different nominal name -> reject
        let mut other = counter();
        other.name = "Gauge".into();
        let e = source_sig(&d, &[other], &mut vec![]).unwrap_err();
        assert_eq!(e.0, CODE_RECORD);
        // same name, different field closure -> reject
        let mut wide = counter();
        wide.fields.push(ResolvedField {
            name: "extra".into(),
            vt: VT::Bool,
            span: Span { start: 0, end: 0 },
        });
        let e = source_sig(&d, &[wide], &mut vec![]).unwrap_err();
        assert_eq!(e.0, CODE_RECORD);
    }

    #[test]
    fn callback_interns_once_and_readonly_finds_it() {
        let d = decl(
            vec![ResKind::Callback { ctx: true }, ResKind::I32],
            ResourceForm::Pure { ret: ResOut::I64 },
            false,
            true,
        );
        let mut table = vec![];
        let s = source_sig(&d, &[], &mut table).unwrap();
        assert_eq!(s.params, vec![VT::Function(0), VT::I32]);
        assert_eq!(table.len(), 1);
        let s2 = source_sig(&d, &[], &mut table).unwrap();
        assert_eq!(s2, s);
        assert_eq!(table.len(), 1);
        let ro = source_sig_readonly(&d, &[], &table).unwrap();
        assert_eq!(ro, s);
        assert!(source_sig_readonly(&d, &[], &[]).is_err());
    }

    fn local(id: &str, name: &str) -> hir::LocalDef {
        hir::LocalDef {
            id_text: format!("l_{id}"),
            ty: hir::TypeUse::Builtin(hir::BuiltinType::I32),
            name: name.into(),
            role: hir::LocalRole::Binding,
            index: None,
            mutable: true,
            mutable_explicit: false,
            param_mode: hir::ParamMode::View,
            span: Span { start: 0, end: 0 },
        }
    }

    fn plan_expr(kind: hir::ExprKind) -> hir::Expr {
        hir::Expr {
            id_text: String::new(),
            ty: hir::TypeUse::Builtin(hir::BuiltinType::I32),
            kind,
            span: Span { start: 0, end: 0 },
        }
    }

    fn foreign_def(name: &str, declaration: SourceDecl) -> hir::Def {
        hir::Def {
            id_text: name.into(),
            kind: hir::DefKind::Foreign {
                ty: hir::TypeId(0),
                declaration,
            },
            span: Span { start: 0, end: 0 },
        }
    }

    fn call(callee: u32, bindings: Vec<(u32, u32)>, eval: Vec<u32>) -> hir::Expr {
        plan_expr(hir::ExprKind::Call {
            callee: hir::DefId(callee),
            eval_args: eval.into_iter().map(hir::ExprId).collect(),
            bindings: bindings
                .into_iter()
                .map(|(param, arg)| hir::CallBinding {
                    param,
                    arg,
                    moved: false,
                })
                .collect(),
        })
    }

    /// T-04: the credential plan derives every ownership row of every
    /// resource call — view borrow (owner), exclusive lanes, handle
    /// credentials in/out, callback targets — from the checked
    /// declaration + call site alone.
    #[test]
    fn body_foreign_plan_carries_every_credential_row() {
        use ace_foreign_contract::resource::{ResOut, ResourceForm, ViewRole};
        let fields = vec![("value".to_string(), FieldKind::I32)];
        // def 0: two MUT Counter lanes (atomic) — the S03 shape
        let pair = decl(
            vec![
                ResKind::Rec {
                    rec: "Counter".into(),
                    fields: fields.clone(),
                    role: ViewRole::Mut,
                },
                ResKind::Rec {
                    rec: "Counter".into(),
                    fields,
                    role: ViewRole::Mut,
                },
            ],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::I32),
            },
            true,
            false,
        );
        // def 1: the named callback target (an ordinary fn def)
        let cb_def = || hir::Def {
            id_text: "cb".into(),
            kind: hir::DefKind::Function {
                name: "cb".into(),
                ty: hir::TypeId(0),
                body: hir::BodyId(0),
            },
            span: Span { start: 0, end: 0 },
        };
        // def 2: callback + ctx seed
        let fold = decl(
            vec![ResKind::Callback { ctx: true }, ResKind::I32],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::I32),
            },
            false,
            true,
        );
        // def 3: text view
        let blen = decl(
            vec![ResKind::ViewText],
            ResourceForm::Pure { ret: ResOut::U64 },
            false,
            false,
        );
        // def 4: handle create (pure handle return)
        let create = decl(
            vec![ResKind::I32],
            ResourceForm::Pure {
                ret: ResOut::Handle {
                    provider: "r939".into(),
                },
            },
            false,
            false,
        );
        // def 5: handle get (handle in)
        let get = decl(
            vec![ResKind::Handle {
                provider: "r939".into(),
            }],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::I32),
            },
            false,
            false,
        );
        let mut m = hir::Module {
            id_text: "m".into(),
            path: None,
            types: vec![],
            defs: vec![
                foreign_def("t939_pair", pair),
                cb_def(),
                foreign_def("t939_fold", fold),
                foreign_def("t939_byte_length", blen),
                foreign_def("t939_create", create),
                foreign_def("t939_get", get),
            ],
            bodies: vec![],
            span: Span { start: 0, end: 0 },
        };
        // body exprs:
        //  0 ReadLocal(local 0)  1/2 BorrowMut(local 1/2)
        //  3 CallbackRef(def 1)  4 I32 literal (seed)
        //  5 Call byte_length{view: arg0}
        //  6 Call pair{mut: arg1, mut: arg2}
        //  7 Call fold{cb: arg3, seed: arg4}
        //  8 Call create{size: arg4}
        //  9 Call get{handle: arg4}
        let mut exprs = vec![
            plan_expr(hir::ExprKind::ReadLocal {
                local: hir::LocalId(0),
            }),
            plan_expr(hir::ExprKind::BorrowMut {
                local: hir::LocalId(1),
            }),
            plan_expr(hir::ExprKind::BorrowMut {
                local: hir::LocalId(2),
            }),
            plan_expr(hir::ExprKind::CallbackRef { def: hir::DefId(1) }),
            plan_expr(hir::ExprKind::Constant { value: 7 }),
            call(3, vec![(0, 0)], vec![0]),
            call(0, vec![(0, 1), (1, 2)], vec![1, 2]),
            call(2, vec![(0, 3), (1, 4)], vec![3, 4]),
            call(4, vec![(0, 4)], vec![4]),
            call(5, vec![(0, 4)], vec![4]),
        ];
        exprs[0].ty = hir::TypeUse::Builtin(hir::BuiltinType::StrSlice);
        let body = hir::Body {
            id_text: "main".into(),
            owner: hir::DefId(1),
            entry: hir::BlockId(0),
            locals: vec![local("s", "s"), local("a", "a"), local("b", "b")],
            places: vec![],
            exprs,
            blocks: vec![],
            loops: vec![],
            span: Span { start: 0, end: 0 },
        };
        m.bodies.push(body);
        let rows = body_foreign_plan(&m, &m.bodies[0]);
        assert_eq!(rows.len(), 5, "every resource call has a row");
        // byte_length: one view borrow with a DERIVED owner local
        let bl = &rows[&5];
        assert_eq!(bl.symbol, "t939"); // the shared test-helper symbol
        assert_eq!(bl.view_borrows, vec![(0, Some(0))]);
        assert!(bl.exclusive.is_empty());
        // pair: two exclusive lanes, atomic promise
        let pr = &rows[&6];
        assert_eq!(pr.exclusive, vec![0, 1]);
        assert!(pr.atomic);
        assert!(pr.view_borrows.is_empty());
        // fold: the callback target def rides the plan (no re-derivation)
        let fd = &rows[&7];
        assert_eq!(fd.callbacks, vec![(0, 1)]);
        // create: fresh credential out
        let cr = &rows[&8];
        assert_eq!(cr.owned_out.as_deref(), Some("r939"));
        // get: credential in with its provider tag
        let gt = &rows[&9];
        assert_eq!(gt.handles_in, vec![(0, "r939".to_string())]);
        // bundle plan: present, one body
        let bundle = hir::Bundle {
            domain: hir::HirDomain::TextRecord,
            modules: vec![m],
            requires: vec![],
        };
        let plan = bundle_foreign_plan(&bundle).expect("plan present");
        assert_eq!(plan.bodies.len(), 1);
    }

    #[test]
    fn handle_is_source_int() {
        let d = decl(
            vec![ResKind::Handle {
                provider: "r939".into(),
            }],
            ResourceForm::Pure {
                ret: ResOut::Handle {
                    provider: "r939".into(),
                },
            },
            false,
            false,
        );
        let s = source_sig(&d, &[], &mut vec![]).unwrap();
        assert_eq!(s.params, vec![VT::I32]);
        assert_eq!(s.result, VT::I32);
    }
}
