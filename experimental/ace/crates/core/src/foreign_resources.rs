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
