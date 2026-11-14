//! Explicit typed-declaration candidates (937 plan §5.1 "typed标量桥").
//!
//! A typed candidate is an INSPECTABLE declaration — the input identity
//! for the typed public-lane proof (V-03). It is not a checked source
//! module and never stands in for one. Normalization here:
//! * lane vocabulary check (no f32; public set only);
//! * symbol hygiene (C identifier);
//! * the frozen body family binding — a typed candidate names one of the
//!   FROZEN typed body families; it never carries a free-form body;
//! * purity — a pure typed declaration must record its totality
//!   evidence; the frozen checksum family is the only v0.1 basis.

use crate::facts::{ErrorModel, FactsSource, FnFact, FnForm, InterfaceFacts, FACTS_FORMAT};
use crate::lane::LaneKind;
use crate::resource::{FieldKind, ResKind, ResOut, ResourceForm, ResourceSig, ViewRole};
use serde::{Deserialize, Serialize};

/// Recorded totality evidence for a PURE typed entry (plan §2: public
/// pure only with actual totality/no-failure contract evidence).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TotalityEvidence {
    /// Frozen basis vocabulary: v0.1 supports only "frozen-family-total"
    /// (the frozen typed body families are total by construction — no
    /// checked arithmetic, no failure path).
    pub basis: String,
    pub note: String,
}

impl TotalityEvidence {
    pub fn frozen_family(note: &str) -> TotalityEvidence {
        TotalityEvidence {
            basis: "frozen-family-total".into(),
            note: note.into(),
        }
    }

    fn valid(&self) -> bool {
        self.basis == "frozen-family-total" && !self.note.trim().is_empty()
    }
}

/// Frozen typed body families (the BUSINESS bodies of typed candidates —
/// 937 plan §2: native bodies stay in the frozen family set; this crate
/// records the identity, the emitters own the implementations).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum TypedBodyFamily {
    /// Heterogeneous typed checksum: mixes every lane into an i64 with
    /// per-position weights, f64 lanes folded by their RAW BITS (NaN
    /// payloads and -0.0 participate exactly), bool as 0/1, char by its
    /// scalar value. Total by construction (wrapping arithmetic only).
    ChecksumV1,
    /// Fallible typed div over two i64 lanes with one i64 out: d==0 is
    /// the controlled failure (status -1, out untouched).
    FallibleDivV1,
    /// PLAN-939 S07: wrapping sum over a borrowed i64-ELEMENT view
    /// `{const int64_t *p, u64 n}` — n counts ELEMENTS (the element-count
    /// len-unit proof; a byte-count misread observably fails). Pure and
    /// total by construction (machine wrapping add; empty view = 0).
    BufViewI64Sum,
    /// PLAN-939 S07: weighted fold over a CONST view of the frozen
    /// RecMix wide-field closure `flag*1000 + count*3 + tag*5 + big`
    /// (natural layout, trailing padding: flag@0 count@4 tag@8 big@16).
    /// flag/tag range validation is a PRE-CALL fact (the caller's lane
    /// values), so the body itself is pure and total.
    RecMixSum,
    /// PLAN-939 S07: the ORIGINAL wide synchronous callback lane —
    /// `long long (*)(long long ctx, long long item)` invoked per item of
    /// an i64-element view with a wide i64 ctx seed
    /// (`(cb, ctx, items, n)`; the 797 p797_fold shape). Pure by
    /// construction; the callback is never stored (noescape).
    WideFold,
}

/// One typed declaration.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedDecl {
    pub symbol: String,
    pub params: Vec<LaneKind>,
    pub ret: LaneKind,
    pub body: TypedBodyFamily,
    /// Present iff the form is pure (validated against the family).
    pub totality: Option<TotalityEvidence>,
    /// Fallible status table (required iff the family is fallible).
    pub error_model: Option<ErrorModel>,
    /// The fallible ctx lane: None = context-free; Some(nullable).
    pub ctx: Option<bool>,
    pub outs: Vec<LaneKind>,
    /// PLAN-939 S07 typed-RESOURCE binding: present iff the body is one
    /// of the S07 families (BufViewI64Sum / RecMixSum / WideFold). The
    /// sig must be a `typed_peer` sig whose shape the family binds
    /// EXACTLY (see validate). Absent on every 937 scalar decl — the
    /// wire and content_version of the frozen scalar set are unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<ResourceSig>,
}

/// A validated typed-declaration set.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedDecls {
    pub format: String,
    pub id: String,
    pub decls: Vec<TypedDecl>,
}

pub const TYPED_DECLS_FORMAT: &str = "autoabi.contract.typed-decls/1";

impl TypedDecls {
    /// Validate + normalize into facts. Errors are named rejections the
    /// caller surfaces BEFORE any production.
    pub fn validate(&self) -> Result<InterfaceFacts, String> {
        if self.format != TYPED_DECLS_FORMAT {
            return Err(format!(
                "unknown typed-decls format '{}' (known: '{TYPED_DECLS_FORMAT}')",
                self.format
            ));
        }
        if self.id.trim().is_empty() {
            return Err("typed-decls id must be non-empty".into());
        }
        if self.decls.is_empty() {
            return Err("typed declaration set is empty".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut functions = Vec::new();
        for d in &self.decls {
            if d.symbol.is_empty()
                || !d
                    .symbol
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                return Err(format!(
                    "symbol '{}' must be a non-empty C identifier",
                    d.symbol
                ));
            }
            if !seen.insert(d.symbol.clone()) {
                return Err(format!("duplicate symbol '{}'", d.symbol));
            }
            if d.resource.is_some() {
                // PLAN-939 S07 typed-resource family: the sig binds the
                // lanes; the scalar spine must stay empty and the family
                // shape is enforced exactly (a wrong shape is a named
                // rejection, mirroring the FallibleDivV1 discipline).
                if !d.params.is_empty() || !d.outs.is_empty() || d.ctx.is_some() {
                    return Err(format!(
                        "symbol '{}': typed-resource families carry their lanes in the sig \
                         (scalar spine stays empty)",
                        d.symbol
                    ));
                }
                let sig = d.resource.as_ref().expect("checked above");
                sig.validate()
                    .map_err(|e| format!("symbol '{}': {}", d.symbol, e))?;
                if !sig.typed_peer {
                    return Err(format!(
                        "symbol '{}': typed-resource family sigs must spell the typed_peer \
                         domain",
                        d.symbol
                    ));
                }
                let expect_params: Vec<ResKind> = match d.body {
                    TypedBodyFamily::BufViewI64Sum => vec![ResKind::BufViewI64],
                    TypedBodyFamily::RecMixSum => vec![rec_mix_const()],
                    TypedBodyFamily::WideFold => vec![
                        ResKind::Callback { ctx: true },
                        ResKind::I64,
                        ResKind::BufViewI64,
                    ],
                    other => {
                        return Err(format!(
                            "symbol '{}': body family {other:?} does not take a resource sig",
                            d.symbol
                        ));
                    }
                };
                if sig.params != expect_params {
                    return Err(format!(
                        "symbol '{}': family {:?} binds exactly {:?}, got {:?}",
                        d.symbol,
                        d.body,
                        expect_params.iter().map(|k| k.spell()).collect::<Vec<_>>(),
                        sig.params.iter().map(|k| k.spell()).collect::<Vec<_>>()
                    ));
                }
                let ret_out = match &sig.form {
                    ResourceForm::Pure { ret } => ret.clone(),
                    ResourceForm::Fallible { .. } => {
                        return Err(format!(
                            "symbol '{}': S07 families are pure (no status/out slots)",
                            d.symbol
                        ));
                    }
                };
                if ret_out != ResOut::I64 || d.ret != LaneKind::I64 {
                    return Err(format!("symbol '{}': S07 families return i64", d.symbol));
                }
                if let Some(ev) = &d.totality {
                    if !ev.valid() {
                        return Err(format!(
                            "symbol '{}': totality evidence basis must be \
                             frozen-family-total with a note",
                            d.symbol
                        ));
                    }
                } else {
                    return Err(format!(
                        "symbol '{}': pure family requires recorded totality evidence",
                        d.symbol
                    ));
                }
                if d.error_model.is_some() {
                    return Err(format!(
                        "symbol '{}': total family cannot carry an error model",
                        d.symbol
                    ));
                }
                // Resource entries: the FnFact scalar form is the ignored
                // spine (the sig owns admission/projection) — same
                // convention as the checked-source resource entries.
                functions.push(FnFact {
                    symbol: d.symbol.clone(),
                    form: FnForm::Pure {
                        params: vec![],
                        ret: d.ret,
                    },
                    error_model: None,
                    semantic_ref: None,
                    resource: Some(sig.clone()),
                });
                continue;
            }
            if let Some(bad) = d.params.iter().chain(d.outs.iter()).find(|k| {
                !matches!(
                    k,
                    LaneKind::I32
                        | LaneKind::U32
                        | LaneKind::I64
                        | LaneKind::U64
                        | LaneKind::F64
                        | LaneKind::Bool
                        | LaneKind::Char
                )
            }) {
                return Err(format!(
                    "symbol '{}': lane {:?} is outside the public vocabulary (no f32)",
                    d.symbol, bad
                ));
            }
            let (form, family_is_total) = match d.body {
                TypedBodyFamily::ChecksumV1 => {
                    if d.totality.is_none() {
                        return Err(format!(
                            "symbol '{}': pure family requires recorded totality evidence",
                            d.symbol
                        ));
                    }
                    if let Some(ev) = &d.totality {
                        if !ev.valid() {
                            return Err(format!(
                                "symbol '{}': totality evidence basis must be \
                                 frozen-family-total with a note",
                                d.symbol
                            ));
                        }
                    }
                    if d.error_model.is_some() {
                        return Err(format!(
                            "symbol '{}': total family cannot carry an error model",
                            d.symbol
                        ));
                    }
                    if !d.outs.is_empty() {
                        return Err(format!(
                            "symbol '{}': pure family has no out slots",
                            d.symbol
                        ));
                    }
                    (
                        FnForm::Pure {
                            params: d.params.clone(),
                            ret: d.ret,
                        },
                        true,
                    )
                }
                TypedBodyFamily::FallibleDivV1 => {
                    if d.params != [LaneKind::I64, LaneKind::I64]
                        || d.ret != LaneKind::I32
                        || d.outs != [LaneKind::I64]
                    {
                        return Err(format!(
                            "symbol '{}': fallible-div family binds i64,i64 -> i32 status \
                             with one i64 out",
                            d.symbol
                        ));
                    }
                    if d.totality.is_some() {
                        return Err(format!(
                            "symbol '{}': fallible family cannot claim totality",
                            d.symbol
                        ));
                    }
                    let m = d.error_model.as_ref().ok_or_else(|| {
                        format!(
                            "symbol '{}': fallible family requires a status table",
                            d.symbol
                        )
                    })?;
                    if m.kind != "status-i32" || m.codes.is_empty() {
                        return Err(format!(
                            "symbol '{}': error model must be status-i32 with codes",
                            d.symbol
                        ));
                    }
                    (
                        FnForm::Fallible {
                            ctx: d.ctx,
                            params: d.params.clone(),
                            outs: d.outs.clone(),
                        },
                        false,
                    )
                }
                TypedBodyFamily::BufViewI64Sum
                | TypedBodyFamily::RecMixSum
                | TypedBodyFamily::WideFold => {
                    return Err(format!(
                        "symbol '{}': S07 resource families REQUIRE the resource sig \
                         (resource: Some(...)); the scalar spine cannot spell them",
                        d.symbol
                    ));
                }
            };
            functions.push(FnFact {
                symbol: d.symbol.clone(),
                form,
                error_model: if family_is_total {
                    None
                } else {
                    d.error_model.clone()
                },
                semantic_ref: None,
                resource: None,
            });
        }
        let facts = InterfaceFacts {
            format: FACTS_FORMAT.into(),
            logical: format!("typed/{}", self.id),
            content_version: crate::blake3_hex(
                serde_json::to_string(self)
                    .map_err(|e| e.to_string())?
                    .as_bytes(),
            ),
            source: FactsSource::Typed {
                id: self.id.clone(),
            },
            functions,
            effects: Vec::new(),
            capabilities: if self.decls.iter().any(|d| d.resource.is_some()) {
                vec![crate::resource::RESOURCE_CAP.into()]
            } else {
                Vec::new()
            },
        };
        facts.validate()?;
        Ok(facts)
    }
}

/// The frozen RecMix public closure (PLAN-939 S07 `RecMixSum`): wide
/// fields under natural alignment — bool flag@0, i32 count@4, char
/// (u32 scalar value) tag@8, i64 big@16, size 24 align 8. The same
/// nominal closure the 797 layout model freezes; spelled HERE so this
/// crate stays backend-free.
pub fn rec_mix_fields() -> Vec<(String, FieldKind)> {
    vec![
        ("flag".into(), FieldKind::Bool),
        ("count".into(), FieldKind::I32),
        ("tag".into(), FieldKind::Char),
        ("big".into(), FieldKind::I64),
    ]
}

fn rec_mix_const() -> ResKind {
    ResKind::Rec {
        rec: "RecMix".into(),
        fields: rec_mix_fields(),
        role: ViewRole::Const,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn checksum_decls() -> TypedDecls {
        TypedDecls {
            format: TYPED_DECLS_FORMAT.into(),
            id: "t937".into(),
            decls: vec![TypedDecl {
                symbol: "t937_mix".into(),
                params: vec![
                    LaneKind::I32,
                    LaneKind::U32,
                    LaneKind::I64,
                    LaneKind::U64,
                    LaneKind::F64,
                    LaneKind::Bool,
                    LaneKind::Char,
                    LaneKind::I64,
                ],
                ret: LaneKind::I64,
                body: TypedBodyFamily::ChecksumV1,
                totality: Some(TotalityEvidence::frozen_family(
                    "wrapping mix only; no checked arithmetic, no failure path",
                )),
                error_model: None,
                ctx: None,
                outs: vec![],
                resource: None,
            }],
        }
    }

    #[test]
    fn checksum_decls_project_to_valid_facts() {
        let facts = checksum_decls().validate().unwrap();
        assert!(matches!(facts.source, FactsSource::Typed { .. }));
        assert_eq!(
            facts.function("t937_mix").unwrap().form.signature(),
            "pure(i32,u32,i64,u64,f64,bool,char,i64)->i64"
        );
    }

    #[test]
    fn pure_without_evidence_rejected() {
        let mut d = checksum_decls();
        d.decls[0].totality = None;
        assert!(d.validate().unwrap_err().contains("totality"));
    }

    #[test]
    fn fallible_div_shape_enforced() {
        let mut d = checksum_decls();
        d.decls[0] = TypedDecl {
            symbol: "t937_div".into(),
            params: vec![LaneKind::I64, LaneKind::I32], // wrong second lane
            ret: LaneKind::I32,
            body: TypedBodyFamily::FallibleDivV1,
            totality: None,
            error_model: Some(ErrorModel {
                kind: "status-i32".into(),
                codes: vec![(-1, "div-by-zero".into())],
            }),
            ctx: Some(true),
            outs: vec![LaneKind::I64],
            resource: None,
        };
        assert!(d
            .validate()
            .unwrap_err()
            .contains("fallible-div family binds"));
    }

    #[test]
    fn content_version_tracks_decl_content() {
        let a = checksum_decls();
        let mut b = a.clone();
        assert_eq!(
            a.validate().unwrap().content_version,
            b.validate().unwrap().content_version
        );
        b.decls[0].params[0] = LaneKind::U32;
        assert_ne!(
            a.validate().unwrap().content_version,
            b.validate().unwrap().content_version
        );
    }

    /// The S07 wide decl set (same shapes the abi crate freezes).
    pub fn wide_decls_for_test() -> TypedDecls {
        use crate::resource::RESOURCE_SIG_FORMAT;

        let sig = |params: Vec<ResKind>, noescape: bool| ResourceSig {
            format: RESOURCE_SIG_FORMAT.into(),
            params,
            form: ResourceForm::Pure { ret: ResOut::I64 },
            atomic: false,
            noescape,
            typed_peer: true,
        };
        let decl = |symbol: &str, body: TypedBodyFamily, sig: ResourceSig| TypedDecl {
            symbol: symbol.into(),
            params: vec![],
            ret: LaneKind::I64,
            body,
            totality: Some(TotalityEvidence::frozen_family(
                "wrapping machine arithmetic only; no checked ops, no failure path",
            )),
            error_model: None,
            ctx: None,
            outs: vec![],
            resource: Some(sig),
        };
        TypedDecls {
            format: TYPED_DECLS_FORMAT.into(),
            id: "t939w".into(),
            decls: vec![
                decl(
                    "t939_wide_sum",
                    TypedBodyFamily::BufViewI64Sum,
                    sig(vec![ResKind::BufViewI64], false),
                ),
                decl(
                    "t939_crec_sum",
                    TypedBodyFamily::RecMixSum,
                    sig(vec![rec_mix_const()], false),
                ),
                decl(
                    "t939_wide_fold",
                    TypedBodyFamily::WideFold,
                    sig(
                        vec![
                            ResKind::Callback { ctx: true },
                            ResKind::I64,
                            ResKind::BufViewI64,
                        ],
                        true,
                    ),
                ),
            ],
        }
    }

    #[test]
    fn wide_decls_project_to_resource_facts_and_plan() {
        let facts = wide_decls_for_test().validate().unwrap();
        assert!(facts
            .capabilities
            .iter()
            .any(|c| c == crate::resource::RESOURCE_CAP));
        for sym in ["t939_wide_sum", "t939_crec_sum", "t939_wide_fold"] {
            assert!(facts.function(sym).unwrap().is_resource());
        }
        let plan = crate::PhysicalPlan::project(&facts).unwrap();
        // element-count view = ptr + n pair; fold = fn + wide ctx + ptr + n.
        let fold = plan
            .functions
            .iter()
            .find(|f| f.symbol == "t939_wide_fold")
            .unwrap();
        let params = match &fold.form {
            crate::plan::PhysForm::Pure { params, .. } => params.clone(),
            other => panic!("fold must be pure: {other:?}"),
        };
        let names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["p0_fn", "p1_w", "p2_ptr", "p2_n"]);
    }

    #[test]
    fn wide_family_wrong_shape_is_named_rejection() {
        // BufViewI64Sum must bind EXACTLY one element view.
        let mut d = wide_decls_for_test();
        d.decls[0].resource.as_mut().unwrap().params = vec![ResKind::BufViewI64, ResKind::I64];
        assert!(d.validate().unwrap_err().contains("binds exactly"));
        // RecMixSum with a divergent closure (a valid sig on its own —
        // the FAMILY binding is what rejects the shape change).
        let mut d = wide_decls_for_test();
        if let Some(ResKind::Rec { fields, .. }) =
            d.decls[1].resource.as_mut().unwrap().params.get_mut(0)
        {
            fields.pop();
        }
        assert!(d.validate().unwrap_err().contains("binds exactly"));
        // Fallible form rejected.
        let mut d = wide_decls_for_test();
        d.decls[0].resource.as_mut().unwrap().form = ResourceForm::Fallible {
            ctx: None,
            out: None,
        };
        assert!(d.validate().unwrap_err().contains("pure"));
        // A scalar-family decl carrying a resource sig is rejected (the
        // wide sig on a ChecksumV1 body; spine already empty so the
        // FAMILY match is the rejection point).
        let mut d = wide_decls_for_test();
        d.decls[0].body = TypedBodyFamily::ChecksumV1;
        assert!(d
            .validate()
            .unwrap_err()
            .contains("does not take a resource sig"));
        // A scalar spine on a wide family is rejected too.
        let mut d = checksum_decls();
        d.decls[0].resource = Some(wide_decls_for_test().decls[0].resource.clone().unwrap());
        assert!(d
            .validate()
            .unwrap_err()
            .contains("scalar spine stays empty"));
    }

    #[test]
    fn scalar_decl_wire_is_byte_identical_without_the_resource_field() {
        // The frozen t937 set digests over its serialized bytes; the new
        // optional field must not appear when None.
        let d = checksum_decls();
        let json = serde_json::to_string(&d).unwrap();
        assert!(!json.contains("resource"));
        let back: TypedDecls = serde_json::from_str(&json).unwrap();
        assert!(back.decls[0].resource.is_none());
        assert_eq!(back, d);
    }
}
