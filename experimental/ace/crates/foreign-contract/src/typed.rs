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
                || !d.symbol.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                return Err(format!("symbol '{}' must be a non-empty C identifier", d.symbol));
            }
            if !seen.insert(d.symbol.clone()) {
                return Err(format!("duplicate symbol '{}'", d.symbol));
            }
            if let Some(bad) = d.params.iter().chain(d.outs.iter()).find(|k| {
                !matches!(k, LaneKind::I32 | LaneKind::U32 | LaneKind::I64 | LaneKind::U64
                    | LaneKind::F64 | LaneKind::Bool | LaneKind::Char)
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
                    if d.params != [LaneKind::I64, LaneKind::I64] || d.ret != LaneKind::I32
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
                        format!("symbol '{}': fallible family requires a status table", d.symbol)
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
            };
            functions.push(FnFact {
                symbol: d.symbol.clone(),
                form,
                error_model: if family_is_total { None } else { d.error_model.clone() },
                semantic_ref: None,
            });
        }
        let facts = InterfaceFacts {
            format: FACTS_FORMAT.into(),
            logical: format!("typed/{}", self.id),
            content_version: crate::blake3_hex(
                serde_json::to_string(self).map_err(|e| e.to_string())?.as_bytes(),
            ),
            source: FactsSource::Typed { id: self.id.clone() },
            functions,
            effects: Vec::new(),
            capabilities: Vec::new(),
        };
        facts.validate()?;
        Ok(facts)
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
            }],
        }
    }

    #[test]
    fn checksum_decls_project_to_valid_facts() {
        let facts = checksum_decls().validate().unwrap();
        assert!(matches!(facts.source, FactsSource::Typed { .. }));
        assert_eq!(facts.function("t937_mix").unwrap().form.signature(),
                   "pure(i32,u32,i64,u64,f64,bool,char,i64)->i64");
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
        };
        assert!(d.validate().unwrap_err().contains("fallible-div family binds"));
    }

    #[test]
    fn content_version_tracks_decl_content() {
        let a = checksum_decls();
        let mut b = a.clone();
        assert_eq!(a.validate().unwrap().content_version, b.validate().unwrap().content_version);
        b.decls[0].params[0] = LaneKind::U32;
        assert_ne!(a.validate().unwrap().content_version, b.validate().unwrap().content_version);
    }
}
