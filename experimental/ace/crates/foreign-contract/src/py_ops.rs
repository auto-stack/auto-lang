//! PLAN-1107 T-02: the **target-neutral Python op contract**.
//!
//! The frozen Python op set (`py_import`, `py_call`, `py_getattr`,
//! `py_getitem`, `py_binop`, `py_to_f64`/`py_to_i32`/`py_to_bool`,
//! `py_stage_str`, …) is part of the language↔host contract: one shared front
//! door resolves these operations, the AV lane lowers them to its
//! `p967.frame/1` provider calls, and a source-generating target (A2PY) emits
//! Python that implements them natively.
//!
//! The contract therefore must not live only in the AV provider source. This
//! module is its neutral home: a frozen declaration table plus a
//! **declaration-only admission** that any target can consume without PyO3,
//! without a provider binary and without a loaded symbol table. The AV
//! provider source (`crates/foreign-contract/py/py_bridge_provider.rs`) is the
//! AV *implementation* of the same table; a consistency test in `ace-abi`
//! binds the two (names, symbols, forms, error codes, resource lane kinds) so
//! neither side can drift silently.
//!
//! Identity: the admission's review id, contract digest and digests are
//! derived from this table, never supplied by a caller. `available_symbols`
//! names the declarations themselves — for a source-generating target the
//! generated runtime IS their implementation, and no C symbol is resolved on
//! that path.

use crate::manual::{AdmittedForeign, HostAuthority, ManualDecl, ManualManifest, SOURCE_CAP};
use crate::resource::{ResKind, ResOut, ResourceForm, ResourceSig, RESOURCE_CAP, RESOURCE_SIG_FORMAT};
use crate::{ErrorModel, FactsSource, FnFact, FnForm, InterfaceFacts, LaneKind, FACTS_FORMAT};

/// Review identity of the neutral declaration contract (never the AV
/// provider's review id — the two admissions are distinct by construction).
pub const PY_CONTRACT_REVIEW_ID: &str = "p1107/py-language-contract/1";
/// Provider identity of the frozen Python op set.
pub const PY_CONTRACT_PROVIDER: &str = "py";
/// Logical symbol namespace of the provider table.
pub const PY_CONTRACT_LOGICAL: &str = "rust/py";
/// Content version of the frozen table (bumped only with a semantic change).
pub const PY_CONTRACT_CONTENT_VERSION: &str = "p1107-py-ops-v1";

/// One frozen Python op declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PyOpDecl {
    /// Source-facing name (`use.py` member calls resolve to this).
    pub name: &'static str,
    /// Provider symbol (`rust/py/<symbol>`).
    pub symbol: &'static str,
    /// `Some((status code, reason))` for the fallible form, `None` for pure.
    pub error: Option<(i32, &'static str)>,
}

const fn op(name: &'static str, symbol: &'static str, error: Option<(i32, &'static str)>) -> PyOpDecl {
    PyOpDecl {
        name,
        symbol,
        error,
    }
}

/// The 17 frozen provider ops (PLAN-1026 §4). Order is the declaration order.
pub const PY_OPS: &[PyOpDecl] = &[
    op("py_admission", "py_admission_v1", Some((-100, "py.init"))),
    op("py_alive", "py_alive_v1", None),
    op("py_binop", "py_binop_v1", Some((-104, "py.binop"))),
    op("py_call", "py_call_v1", Some((-103, "py.call"))),
    op("py_error_class", "py_error_class_v1", None),
    op("py_error_stage", "py_error_stage_v1", None),
    op("py_f64_lo", "py_f64_lo_v1", None),
    op("py_getattr", "py_getattr_v1", Some((-102, "py.attr"))),
    op("py_getitem", "py_getitem_v1", Some((-105, "py.index"))),
    op("py_import", "py_import_v1", Some((-101, "py.import"))),
    op("py_release", "py_release_v1", Some((-110, "py.release"))),
    op(
        "py_session_close",
        "py_session_close_v1",
        Some((-111, "py.close")),
    ),
    op("py_stage_str", "py_stage_str_v1", Some((-109, "py.stage-str"))),
    op("py_str_word", "py_str_word_v1", None),
    op("py_to_bool", "py_to_bool_v1", Some((-108, "py.to-bool"))),
    op("py_to_f64", "py_to_f64_v1", Some((-106, "py.to-f64"))),
    op("py_to_i32", "py_to_i32_v1", Some((-107, "py.to-i32"))),
];

/// The frozen op names in declaration order.
pub fn op_names() -> Vec<&'static str> {
    PY_OPS.iter().map(|o| o.name).collect()
}

/// The frozen declaration contract (manual manifest) of the Python op set.
pub fn language_manifest() -> ManualManifest {
    let mut functions = Vec::with_capacity(PY_OPS.len());
    let mut declarations = Vec::with_capacity(PY_OPS.len());
    let mut effects: Vec<String> = Vec::new();
    for d in PY_OPS {
        let resource = ResourceSig {
            format: RESOURCE_SIG_FORMAT.to_string(),
            // every op rides ONE view-text frame parameter
            params: vec![ResKind::ViewText],
            form: match d.error {
                Some(_) => ResourceForm::Fallible {
                    ctx: None,
                    out: Some(ResOut::I32),
                },
                None => ResourceForm::Pure { ret: ResOut::I32 },
            },
            atomic: false,
            noescape: true,
            typed_peer: false,
        };
        let (form, error_model) = match d.error {
            Some((code, reason)) => (
                FnForm::Fallible {
                    ctx: None,
                    params: Vec::new(),
                    outs: vec![LaneKind::I32],
                },
                Some(ErrorModel {
                    kind: "status-i32".to_string(),
                    codes: vec![(code, reason.to_string())],
                }),
            ),
            None => (
                FnForm::Pure {
                    params: Vec::new(),
                    ret: LaneKind::I32,
                },
                None,
            ),
        };
        functions.push(FnFact {
            symbol: d.symbol.to_string(),
            form,
            error_model,
            semantic_ref: None,
            resource: Some(resource),
        });
        declarations.push(ManualDecl {
            source_name: d.name.to_string(),
            symbol: d.symbol.to_string(),
            context_symbol: None,
        });
        match d.error {
            Some(_) => {
                if !effects.iter().any(|e| e == "checked") {
                    effects.push("checked".to_string());
                }
            }
            None => effects.push(format!("pure:{}", d.symbol)),
        }
    }
    let capabilities = vec![SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];
    let header_digest = crate::blake3_hex(b"p1107/py-language-contract/header/1");
    let physical_table_digest = crate::blake3_hex(b"p1107/py-language-contract/physical/1");
    ManualManifest {
        // resource entries require the /2 manifest schema
        schema: crate::manual::MANUAL_SCHEMA_V2.to_string(),
        review_id: PY_CONTRACT_REVIEW_ID.to_string(),
        provider: PY_CONTRACT_PROVIDER.to_string(),
        header_digest,
        physical_table_digest,
        target: crate::plan::TARGET.to_string(),
        profile: crate::plan::PROFILE.to_string(),
        callconv: "windows-x64".to_string(),
        direction: "source-import".to_string(),
        facts: InterfaceFacts {
            format: FACTS_FORMAT.to_string(),
            logical: PY_CONTRACT_LOGICAL.to_string(),
            content_version: PY_CONTRACT_CONTENT_VERSION.to_string(),
            source: FactsSource::Typed {
                id: format!("manual/{PY_CONTRACT_REVIEW_ID}"),
            },
            functions,
            effects,
            capabilities,
        },
        declarations,
    }
}

/// The declaration-only admission of the frozen Python op set.
pub fn language_contract() -> Result<AdmittedForeign, String> {
    let manifest = language_manifest();
    let contract_digest = manifest.contract_digest()?;
    let provider_digest = crate::blake3_hex(
        format!("{PY_CONTRACT_LOGICAL}/{PY_CONTRACT_CONTENT_VERSION}").as_bytes(),
    );
    let mut capabilities: std::collections::BTreeSet<String> =
        manifest.facts.capabilities.iter().cloned().collect();
    capabilities.insert(SOURCE_CAP.to_string());
    let available_symbols: std::collections::BTreeSet<String> = manifest
        .declarations
        .iter()
        .map(|d| d.symbol.clone())
        .collect();
    let host = HostAuthority {
        review_id: manifest.review_id.clone(),
        contract_digest,
        provider: manifest.provider.clone(),
        provider_digest,
        header_digest: manifest.header_digest.clone(),
        physical_table_digest: manifest.physical_table_digest.clone(),
        target: manifest.target.clone(),
        capabilities,
        available_symbols,
        binding_generation: 1,
    };
    manifest.admit(&host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_the_frozen_seventeen_ops() {
        assert_eq!(PY_OPS.len(), 17);
        let mut names: Vec<&str> = op_names();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 17, "op names are unique");
        let mut symbols: Vec<&str> = PY_OPS.iter().map(|o| o.symbol).collect();
        symbols.sort_unstable();
        symbols.dedup();
        assert_eq!(symbols.len(), 17, "provider symbols are unique");
        for want in [
            "py_import",
            "py_call",
            "py_getattr",
            "py_getitem",
            "py_binop",
            "py_to_f64",
            "py_to_i32",
            "py_to_bool",
            "py_stage_str",
        ] {
            assert!(names.contains(&want), "{want}");
        }
        // the fallible/pure split is explicit, one status code per fallible op
        let mut codes: Vec<i32> = PY_OPS.iter().filter_map(|o| o.error.map(|e| e.0)).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), 12, "twelve fallible ops, unique status codes");
    }

    #[test]
    fn manifest_validates_and_admits_without_a_provider() {
        let m = language_manifest();
        m.validate().expect("the neutral manifest validates");
        let admitted = language_contract().expect("declaration-only admission");
        assert_eq!(admitted.source_declarations_ref().len(), 17);
        for decl in admitted.source_declarations_ref() {
            assert_eq!(decl.provider, PY_CONTRACT_PROVIDER);
            let res = decl.resource().expect("frame resource sig");
            assert_eq!(res.params, vec![ResKind::ViewText]);
        }
        // deterministic identities: rebuilding the same table yields the same
        // contract, i.e. no host-mutable state participates
        let again = language_contract().expect("rebuilds");
        assert_eq!(again.contract_digest(), admitted.contract_digest());
    }
}