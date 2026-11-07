//! Logical Interface Facts (937 plan §5.1 "逻辑接口" row).
//!
//! Facts are the NEUTRAL identity both real consumers bind to:
//! * nominal identity — a logical id plus a CONTENT VERSION minted
//!   through the merged semantic contract; two interfaces with the same
//!   name but different content versions are DIFFERENT interfaces, and
//!   no string-name equality ever substitutes for that check (the plan's
//!   "字符串同名=同名义" prohibition);
//! * per-function logical forms — params/ret lanes, the fallible ctx
//!   lane, the ordered OUT SLOT LIST, and the per-function EXPLICIT
//!   ERROR TABLE (status code + reason), so failure semantics travel
//!   with the signature instead of being guessed from exit codes;
//! * effects/capabilities — the v0.1 frozen vocabulary;
//! * FACTS SOURCE — `Typed` (an explicit typed candidate) or
//!   `CheckedSource` (real checked source). The two are never conflated:
//!   a typed candidate never settles a checked-source export's
//!   acceptance and vice versa (937 plan §1.2 input-identity split).
//!
//! This module is a pure model: validation only, no emission, no calls.

use crate::lane::{LaneKind, LaneValue};
use serde::{Deserialize, Serialize};

/// Facts codec identity (strict, versioned; see [`crate::codec`]).
pub const FACTS_FORMAT: &str = "autoabi.contract.facts/1";

/// Where the facts came from — the input-identity separation.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FactsSource {
    /// An explicit typed-declaration candidate (no checked source body
    /// behind it; business body comes from the frozen typed families).
    Typed { id: String },
    /// Real checked Auto source: the module identity plus the source
    /// content digest the facts were derived from.
    CheckedSource {
        module: String,
        source_sha256: String,
    },
}

/// Explicit error model of a fallible entry: an ordered status table.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorModel {
    /// `status-i32`: i32 status returned by the fallible form; success 0.
    pub kind: String,
    /// Frozen (status, reason) pairs; codes are negative, unique, sorted.
    pub codes: Vec<(i32, String)>,
}

/// Per-function logical form. Mirrors the 797 descriptor form at the
/// LOGICAL level: lane kinds (not physical spellings), the fallible ctx
/// lane (`Some(nullable)`), and the full ordered out-slot list.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FnForm {
    Pure {
        params: Vec<LaneKind>,
        ret: LaneKind,
    },
    Fallible {
        /// Explicit context first parameter; `Some(nullable)` spells a
        /// context-free (`None`) or nullable/required (`Some`) entry.
        ctx: Option<bool>,
        params: Vec<LaneKind>,
        outs: Vec<LaneKind>,
    },
}

impl FnForm {
    /// Canonical signature spelling — the exact comparison key between
    /// two forms (same token family as the 797 descriptor spellings).
    pub fn signature(&self) -> String {
        match self {
            FnForm::Pure { params, ret } => format!(
                "pure({})->{}",
                params
                    .iter()
                    .map(|p| p.spell())
                    .collect::<Vec<_>>()
                    .join(","),
                ret.spell()
            ),
            FnForm::Fallible { ctx, params, outs } => {
                let ctx_s = match ctx {
                    Some(nullable) => {
                        format!("ctx:{},", if *nullable { "nullable" } else { "required" })
                    }
                    None => String::new(),
                };
                if outs.is_empty() {
                    format!(
                        "fallible({}{})->status",
                        ctx_s,
                        params
                            .iter()
                            .map(|p| p.spell())
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                } else {
                    format!(
                        "fallible({}{})->status,out {}",
                        ctx_s,
                        params
                            .iter()
                            .map(|p| p.spell())
                            .collect::<Vec<_>>()
                            .join(","),
                        outs.iter().map(|p| p.spell()).collect::<Vec<_>>().join(",")
                    )
                }
            }
        }
    }
}

/// One public function's logical facts.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FnFact {
    pub symbol: String,
    pub form: FnForm,
    /// Fallible entries carry their explicit status table here (empty
    /// for pure entries).
    #[serde(default)]
    pub error_model: Option<ErrorModel>,
    /// Nominal semantic ref of the backing declaration when the facts
    /// bind to one (checked-source exports always do; typed candidates
    /// may mint one through the contract).
    #[serde(default)]
    pub semantic_ref: Option<(String, String)>,
    /// PLAN-939 resource signature: when present, THIS is the
    /// authoritative form (the scalar `form` field is then ignored for
    /// projection/admission and the entry is a resource entry of an
    /// `ace.foreign.manual/2` manifest). `None` keeps the 937/938 wire
    /// bytes and digests byte-identical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<crate::resource::ResourceSig>,
}

impl FnFact {
    pub fn is_resource(&self) -> bool {
        self.resource.is_some()
    }
}

/// The neutral interface facts object.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceFacts {
    pub format: String,
    /// Nominal logical id (owner-qualified where an owner exists).
    pub logical: String,
    /// Content version — minted through the semantic contract by the
    /// producer; a same-name interface with a different version is a
    /// different interface.
    pub content_version: String,
    pub source: FactsSource,
    pub functions: Vec<FnFact>,
    /// Frozen v0.1 effects vocabulary: subset of {"pure", "checked"} —
    /// "pure" only with recorded totality evidence per pure entry.
    pub effects: Vec<String>,
    pub capabilities: Vec<String>,
}

impl InterfaceFacts {
    /// Structural validation (identity, per-function rules, error-table
    /// discipline). Pure-form entries in a `CheckedSource` interface
    /// without a pure effect entry are rejected: ordinary checked
    /// arithmetic is fallible (937 plan §2).
    pub fn validate(&self) -> Result<(), String> {
        if self.format != FACTS_FORMAT {
            return Err(format!(
                "unknown facts format '{}' (known: '{FACTS_FORMAT}')",
                self.format
            ));
        }
        if self.logical.trim().is_empty() {
            return Err("logical id must be non-empty".into());
        }
        if self.content_version.trim().is_empty() {
            return Err("content version must be non-empty".into());
        }
        validate_functions(
            &self.functions,
            matches!(self.source, FactsSource::CheckedSource { .. }),
            &self.effects,
            &self.capabilities,
        )
    }
    /// Find a function by symbol (nominal lookup only; content identity
    /// is the whole-object content version, never the symbol).
    pub fn function(&self, symbol: &str) -> Option<&FnFact> {
        self.functions.iter().find(|f| f.symbol == symbol)
    }

    /// Canonical JSON for digesting: functions sorted by symbol, error
    /// codes kept in declared (already sorted) order, effects/caps sorted.
    pub fn canonical(&self) -> Result<String, String> {
        let mut sorted = self.clone();
        sorted.functions.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        sorted.effects.sort();
        sorted.capabilities.sort();
        serde_json::to_string(&sorted).map_err(|e| e.to_string())
    }

    /// Content digest over the canonical form.
    pub fn digest(&self) -> Result<String, String> {
        Ok(crate::blake3_hex(self.canonical()?.as_bytes()))
    }

    /// Per-value PRE-CALL validation for one call to `symbol`
    /// (params + ctx value); returns a named rejection BEFORE any call
    /// is made (plan §5.3). `args` excludes the ctx lane.
    pub fn validate_call_values(
        &self,
        symbol: &str,
        ctx: Option<&LaneValue>,
        args: &[LaneValue],
    ) -> Result<(), String> {
        let f = self
            .function(symbol)
            .ok_or_else(|| format!("unknown symbol '{symbol}'"))?;
        let (declared_ctx, params) = match &f.form {
            FnForm::Pure { params, .. } => (None, params),
            FnForm::Fallible { ctx, params, .. } => (Some(*ctx), params),
        };
        if declared_ctx.is_some() != ctx.is_some() {
            return Err(format!(
                "entry '{symbol}' ctx lane mismatch: declared {:?}, supplied {}",
                declared_ctx.is_some(),
                ctx.is_some()
            ));
        }
        if args.len() != params.len() {
            return Err(format!(
                "entry '{symbol}' expects {} params, got {}",
                params.len(),
                args.len()
            ));
        }
        for (i, (k, v)) in params.iter().zip(args).enumerate() {
            LaneValue::validate(*k, v).map_err(|e| format!("entry '{symbol}' param {i}: {e}"))?;
        }
        Ok(())
    }
}

/// Shared borrowed structural rules; source declarations must not clone a
/// whole interface (including module-wide effects) merely to validate one call.
pub(crate) fn validate_functions(
    functions: &[FnFact],
    checked_source: bool,
    effects: &[String],
    capabilities: &[String],
) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    if functions.is_empty() {
        return Err("interface must export at least one function".into());
    }
    for f in functions {
        if f.symbol.is_empty()
            || !f
                .symbol
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(format!(
                "symbol '{}' must be a non-empty C identifier",
                f.symbol
            ));
        }
        if !seen.insert(f.symbol.as_str()) {
            return Err(format!("duplicate symbol '{}'", f.symbol));
        }
        if let Some(res) = &f.resource {
            // PLAN-939 resource entry: the resource sig's own form is the
            // SINGLE authoritative form — the scalar `form` field is
            // ignored for admission/projection of resource entries.
            res.validate()
                .map_err(|e| format!("resource entry '{}': {e}", f.symbol))?;
            match res.form {
                crate::resource::ResourceForm::Pure { .. } => {
                    if f.error_model.is_some() {
                        return Err(format!(
                            "pure resource entry '{}' must not carry an error model",
                            f.symbol
                        ));
                    }
                }
                crate::resource::ResourceForm::Fallible { .. } => {
                    let m = f.error_model.as_ref().ok_or_else(|| {
                        format!(
                            "fallible resource entry '{}' must carry an explicit error model",
                            f.symbol
                        )
                    })?;
                    validate_error_table(&f.symbol, m)?;
                }
            }
            if !capabilities.iter().any(|c| c == crate::resource::RESOURCE_CAP) {
                return Err(format!(
                    "resource entry '{}' requires the {} capability",
                    f.symbol,
                    crate::resource::RESOURCE_CAP
                ));
            }
            continue;
        }
        match &f.form {
            FnForm::Pure { .. } => {
                if f.error_model.is_some() {
                    return Err(format!(
                        "pure entry '{}' must not carry an error model",
                        f.symbol
                    ));
                }
                // A checked-source pure entry needs recorded pure
                // evidence in the effects vocabulary.
                if checked_source
                    && !effects
                        .iter()
                        .any(|e| e.strip_prefix("pure:") == Some(f.symbol.as_str()))
                {
                    return Err(format!(
                        "checked-source entry '{}' is pure without totality evidence \
                             (effects must record pure:<symbol>; ordinary checked \
                             arithmetic is fallible)",
                        f.symbol
                    ));
                }
            }
            FnForm::Fallible { outs, .. } => {
                let m = f.error_model.as_ref().ok_or_else(|| {
                    format!(
                        "fallible entry '{}' must carry an explicit error model",
                        f.symbol
                    )
                })?;
                validate_error_table(&f.symbol, m)?;
                // Status-only fallible entries (empty outs) are legal;
                // no additional v0.1 constraint beyond the table above.
                let _ = outs;
            }
        }
    }
    Ok(())
}

/// Shared explicit-error-table discipline (negative, unique, ascending,
/// non-empty, `status-i32`).
pub(crate) fn validate_error_table(symbol: &str, m: &ErrorModel) -> Result<(), String> {
    if m.kind != "status-i32" {
        return Err(format!(
            "entry '{symbol}': unknown error model kind '{}' (v0.1: status-i32)",
            m.kind
        ));
    }
    if m.codes.windows(2).any(|pair| pair[0].0 > pair[1].0) {
        return Err(format!(
            "entry '{symbol}': error codes must be sorted ascending"
        ));
    }
    if m.codes.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(format!("entry '{symbol}': duplicate status codes"));
    }
    if m.codes.iter().any(|(c, _)| *c >= 0) {
        return Err(format!(
            "entry '{symbol}': status codes must be negative (0 is success)"
        ));
    }
    if m.codes.is_empty() {
        return Err(format!(
            "entry '{symbol}': fallible form with an EMPTY status table cannot \
                 spell its failure semantics; declare the failure codes"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn typed_facts() -> InterfaceFacts {
        InterfaceFacts {
            format: FACTS_FORMAT.into(),
            logical: "probe937/typed-mix".into(),
            content_version: "v-test".into(),
            source: FactsSource::Typed {
                id: "typed-1".into(),
            },
            functions: vec![FnFact {
                symbol: "t937_mix".into(),
                form: FnForm::Pure {
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
                },
                error_model: None,
                semantic_ref: None,
                resource: None,
            }],
            effects: vec![],
            capabilities: vec![],
        }
    }

    #[test]
    fn typed_pure_facts_validate() {
        assert!(typed_facts().validate().is_ok());
    }

    #[test]
    fn checked_source_pure_needs_evidence() {
        let mut f = typed_facts();
        f.source = FactsSource::CheckedSource {
            module: "m".into(),
            source_sha256: "0".into(),
        };
        let err = f.validate().unwrap_err();
        assert!(err.contains("totality evidence"), "{err}");
        f.effects.push("pure:t937_mix".into());
        assert!(f.validate().is_ok());
    }

    #[test]
    fn fallible_needs_sorted_negative_unique_codes() {
        let mut f = typed_facts();
        f.functions[0].form = FnForm::Fallible {
            ctx: Some(true),
            params: vec![LaneKind::I64, LaneKind::I64],
            outs: vec![LaneKind::I64],
        };
        assert!(f.validate().unwrap_err().contains("error model"));
        f.functions[0].error_model = Some(ErrorModel {
            kind: "status-i32".into(),
            codes: vec![(-1, "div-by-zero".into())],
        });
        assert!(f.validate().is_ok());
        // non-negative / duplicate / unsorted each rejected
        f.functions[0].error_model = Some(ErrorModel {
            kind: "status-i32".into(),
            codes: vec![(1, "bad".into())],
        });
        assert!(f.validate().unwrap_err().contains("negative"));
        f.functions[0].error_model = Some(ErrorModel {
            kind: "status-i32".into(),
            codes: vec![(-2, "b".into()), (-1, "a".into()), (-2, "dup".into())],
        });
        let e = f.validate().unwrap_err();
        assert!(e.contains("sorted") || e.contains("duplicate"), "{e}");
    }

    #[test]
    fn content_digest_tracks_signature_and_source() {
        let a = typed_facts();
        let mut b = a.clone();
        assert_eq!(a.digest().unwrap(), b.digest().unwrap());
        if let FnForm::Pure { params, .. } = &mut b.functions[0].form {
            params[0] = LaneKind::U32;
        }
        assert_ne!(a.digest().unwrap(), b.digest().unwrap());
        let mut c = a.clone();
        c.source = FactsSource::CheckedSource {
            module: "m".into(),
            source_sha256: "1".into(),
        };
        assert_ne!(a.digest().unwrap(), c.digest().unwrap());
    }

    #[test]
    fn resource_entry_needs_capability_and_own_discipline() {
        use crate::resource::{ResKind, ResourceForm, ResourceSig, RESOURCE_SIG_FORMAT};
        let mut f = typed_facts();
        f.functions[0].resource = Some(ResourceSig {
            format: RESOURCE_SIG_FORMAT.into(),
            params: vec![ResKind::ViewText],
            form: ResourceForm::Pure {
                ret: crate::resource::ResOut::I32,
            },
            atomic: false,
            noescape: false,
        });
        // missing foreign.resource capability -> named reject
        let e = f.validate().unwrap_err();
        assert!(e.contains("foreign.resource"), "{e}");
        f.capabilities.push("foreign.resource".into());
        assert!(f.validate().is_ok());
        // wire bytes of a None-resource fact stay unchanged (digest
        // identity of 937/938 facts is not disturbed by this extension)
        let mut scalar = typed_facts();
        scalar.capabilities = f.capabilities.clone();
        let plain = serde_json::to_string(&FnFact {
            symbol: "s".into(),
            form: FnForm::Pure {
                params: vec![],
                ret: LaneKind::I32,
            },
            error_model: None,
            semantic_ref: None,
            resource: None,
        })
        .unwrap();
        assert!(!plain.contains("resource"));
    }

    #[test]
    fn pre_call_value_validation_rejects_before_any_call() {
        let f = typed_facts();
        let args = vec![
            LaneValue::I32(1),
            LaneValue::U32(2),
            LaneValue::I64(3),
            LaneValue::U64(4),
            LaneValue::F64Bits(0),
            LaneValue::Bool(true),
            LaneValue::CharScalar(0xD800), // lone surrogate
            LaneValue::I64(5),
        ];
        let e = f.validate_call_values("t937_mix", None, &args).unwrap_err();
        assert!(e.contains("Unicode scalar"), "{e}");
        let mut ok = args.clone();
        ok[6] = LaneValue::CharScalar('é' as u32);
        assert!(f.validate_call_values("t937_mix", None, &ok).is_ok());
        // arity mismatch is a named rejection too
        assert!(f
            .validate_call_values("t937_mix", None, &args[..7])
            .unwrap_err()
            .contains("expects 8 params"));
    }
}
