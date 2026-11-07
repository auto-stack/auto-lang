//! The finite, independently reviewed C fixture policy used by source-import
//! certification. Candidate manifests NEVER supply their own authorization.
use ace_foreign_contract::{
    manual::*, FactsSource, FnFact, FnForm, InterfaceFacts, LaneKind, FACTS_FORMAT,
};
pub const REVIEW: &str = "plan938-independent-provider-v1";
pub const HEADER: &[u8] = include_bytes!("../cside/source_foreign938.h");
pub const TABLE: &[u8] = include_bytes!("../cside/source_foreign938.contract");
pub const ADD_CONTRACT_DIGEST: &str =
    "cef4c3885f1686a00c0bb8c44ca6b95559a634fff297f8193becca2d1d35d71c";
pub const SCALAR_CONTRACT_DIGEST: &str =
    "aaa3a8b32d0723d77c2c12202d86445ca8626a9368374fd787517801601ebd7e";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Review {
    Add,
    Scalar,
}
impl Review {
    pub fn for_reference(reference: &str) -> Option<Self> {
        match reference {
            "add" => Some(Self::Add),
            "bool-false" | "bool-true" | "fallible" | "zero" | "overflow" | "order" | "and"
            | "or" | "mixed" | "named-order" => Some(Self::Scalar),
            _ => None,
        }
    }
    pub fn for_digest(digest: &str) -> Option<Self> {
        match digest {
            ADD_CONTRACT_DIGEST => Some(Self::Add),
            SCALAR_CONTRACT_DIGEST => Some(Self::Scalar),
            _ => None,
        }
    }
    pub fn digest(self) -> &'static str {
        match self {
            Self::Add => ADD_CONTRACT_DIGEST,
            Self::Scalar => SCALAR_CONTRACT_DIGEST,
        }
    }
    pub fn manifest(self) -> ManualManifest {
        let mut manifest = reviewed_add_manifest();
        if self == Self::Scalar {
            manifest.facts.content_version = "scalar-v1".into();
            for (name, symbol, form, context) in [
                (
                    "foreign_not",
                    "provider_not",
                    FnForm::Pure {
                        params: vec![LaneKind::Bool],
                        ret: LaneKind::Bool,
                    },
                    None,
                ),
                (
                    "foreign_div",
                    "provider_div",
                    FnForm::Fallible {
                        ctx: Some(false),
                        params: vec![LaneKind::I32, LaneKind::I32],
                        outs: vec![LaneKind::I32],
                    },
                    Some("provider_ctx"),
                ),
                (
                    "foreign_tick",
                    "provider_tick",
                    FnForm::Pure {
                        params: vec![LaneKind::I32],
                        ret: LaneKind::I32,
                    },
                    None,
                ),
                (
                    "foreign_trace",
                    "provider_trace",
                    FnForm::Pure {
                        params: vec![],
                        ret: LaneKind::I32,
                    },
                    None,
                ),
                (
                    "foreign_calls",
                    "provider_calls",
                    FnForm::Pure {
                        params: vec![],
                        ret: LaneKind::I32,
                    },
                    None,
                ),
                (
                    "foreign_mix",
                    "provider_mix",
                    FnForm::Pure {
                        params: vec![
                            LaneKind::I32,
                            LaneKind::Bool,
                            LaneKind::I32,
                            LaneKind::Bool,
                            LaneKind::I32,
                        ],
                        ret: LaneKind::I32,
                    },
                    None,
                ),
            ] {
                let error_model = matches!(form, FnForm::Fallible { .. }).then(|| {
                    ace_foreign_contract::ErrorModel {
                        kind: "status-i32".into(),
                        codes: vec![
                            (-3, "context".into()),
                            (-2, "overflow".into()),
                            (-1, "zero".into()),
                        ],
                    }
                });
                manifest.facts.functions.push(FnFact {
                    symbol: symbol.into(),
                    form,
                    error_model,
                    semantic_ref: None,
                    resource: None,
                });
                manifest.declarations.push(ManualDecl {
                    source_name: name.into(),
                    symbol: symbol.into(),
                    context_symbol: context.map(String::from),
                });
            }
            manifest.facts.effects.push("checked".into());
        }
        manifest
    }
    pub fn authority(
        self,
        provider_digest: &str,
        generation: u64,
        caps: &[String],
    ) -> HostAuthority {
        let symbols: &[&str] = match self {
            Self::Add => &["provider_add"],
            Self::Scalar => &[
                "provider_add",
                "provider_not",
                "provider_div",
                "provider_ctx",
                "provider_tick",
                "provider_trace",
                "provider_calls",
                "provider_mix",
            ],
        };
        HostAuthority {
            review_id: REVIEW.into(),
            contract_digest: self.digest().into(),
            provider: "independent-c938".into(),
            provider_digest: provider_digest.into(),
            header_digest: hash(HEADER),
            physical_table_digest: hash(TABLE),
            target: ace_foreign_contract::plan::TARGET.into(),
            capabilities: caps.iter().cloned().collect(),
            available_symbols: symbols.iter().map(|s| (*s).into()).collect(),
            binding_generation: generation,
        }
    }
    pub fn qualify(self, candidate: &ManualManifest, caps: &[String]) -> Result<(), String> {
        candidate.validate()?;
        if candidate != &self.manifest() {
            return Err(
                "foreign.authority: candidate is outside the independent review registry".into(),
            );
        }
        if candidate
            .facts
            .capabilities
            .iter()
            .any(|c| !caps.contains(c))
        {
            return Err("foreign.capability: current host denies source FFI".into());
        }
        Ok(())
    }
}
/// The original pointer format could only name this independently pinned
/// contract. Restore still deep-checks that identity against required pages.
pub fn legacy_add_contract() -> String {
    ADD_CONTRACT_DIGEST.into()
}
pub fn reference_case(reference: &str) -> Option<u8> {
    [
        "add",
        "bool-false",
        "bool-true",
        "fallible",
        "zero",
        "overflow",
        "order",
        "and",
        "or",
        "mixed",
        "named-order",
    ]
    .iter()
    .position(|id| *id == reference)
    .map(|id| id as u8)
}
pub fn hash(b: &[u8]) -> String {
    blake3::hash(b).to_hex().to_string()
}
/// Hand-reviewed logical declaration, separate from the submitted manifest.
/// Additional fixture contracts must receive their own finite policy here.
pub fn reviewed_add_manifest() -> ManualManifest {
    ManualManifest {
        schema: MANUAL_SCHEMA.into(),
        review_id: REVIEW.into(),
        provider: "independent-c938".into(),
        header_digest: hash(HEADER),
        physical_table_digest: hash(TABLE),
        target: ace_foreign_contract::plan::TARGET.into(),
        profile: ace_foreign_contract::plan::PROFILE.into(),
        callconv: "windows-x64".into(),
        direction: "source-import".into(),
        facts: InterfaceFacts {
            format: FACTS_FORMAT.into(),
            logical: "independent-c938".into(),
            content_version: "add-v1".into(),
            source: FactsSource::Typed { id: REVIEW.into() },
            functions: vec![FnFact {
                symbol: "provider_add".into(),
                form: FnForm::Pure {
                    params: vec![LaneKind::I32, LaneKind::I32],
                    ret: LaneKind::I32,
                },
                error_model: None,
                semantic_ref: None,
                resource: None,
            }],
            effects: vec!["pure:provider_add".into()],
            capabilities: vec![SOURCE_CAP.into()],
        },
        declarations: vec![ManualDecl {
            source_name: "foreign_add".into(),
            symbol: "provider_add".into(),
            context_symbol: None,
        }],
    }
}
pub fn admit(
    candidate: ManualManifest,
    provider_digest: &str,
    generation: u64,
    caps: &[String],
) -> Result<AdmittedForeign, String> {
    candidate.admit(&authority(provider_digest, generation, caps)?)
}
pub fn authority(
    provider_digest: &str,
    generation: u64,
    caps: &[String],
) -> Result<HostAuthority, String> {
    Ok(Review::Add.authority(provider_digest, generation, caps))
}
pub fn qualify_candidate(candidate: &ManualManifest, caps: &[String]) -> Result<(), String> {
    Review::Add.qualify(candidate, caps)
}
pub const EVENT_RUNTIME:&str="#include <stdio.h>\nvoid source938_event(const char *name) { fprintf(stderr, \"source938.call=%s\\n\",name); fflush(stderr); }\n";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scalar_review_has_independent_contract_and_never_authorizes_a_modified_candidate() {
        let manifest = Review::Scalar.manifest();
        let digest = manifest.contract_digest().unwrap();
        eprintln!("scalar contract BLAKE3={digest}");
        assert_eq!(digest, SCALAR_CONTRACT_DIGEST);
        let caps = vec![SOURCE_CAP.into()];
        Review::Scalar.qualify(&manifest, &caps).unwrap();
        assert!(Review::Add.qualify(&manifest, &caps).is_err());
        assert!(Review::Scalar.qualify(&manifest, &[]).is_err());
        let host = Review::Scalar.authority(&"b".repeat(64), 7, &caps);
        manifest.clone().admit(&host).unwrap();
        for axis in 0..4 {
            let mut changed = manifest.clone();
            match axis {
                0 => changed.declarations[2].context_symbol = None,
                1 => {
                    changed.facts.functions[1].form = FnForm::Pure {
                        params: vec![LaneKind::I32],
                        ret: LaneKind::I32,
                    }
                }
                2 => {
                    changed.facts.functions[2]
                        .error_model
                        .as_mut()
                        .unwrap()
                        .codes
                        .pop()
                        .unwrap();
                }
                _ => changed.declarations[1].source_name = "unreviewed_name".into(),
            }
            assert!(
                Review::Scalar.qualify(&changed, &caps).is_err(),
                "axis {axis}"
            );
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../crates/auto-lang/test/cases/ace-ca-d-source-938/src");
        for reference in [
            "bool-false",
            "bool-true",
            "fallible",
            "zero",
            "overflow",
            "order",
            "and",
            "or",
            "mixed",
            "named-order",
        ] {
            let deck: crate::worker_source::Deck = serde_json::from_slice(
                &std::fs::read(root.join(format!("{reference}.atomdata"))).unwrap(),
            )
            .unwrap();
            assert_eq!(deck.reference, reference);
            assert_eq!(deck.manifest, manifest);
            assert_eq!(Review::for_reference(reference), Some(Review::Scalar));
            Review::Scalar
                .qualify(&deck.manifest, &deck.capabilities)
                .unwrap();
        }
    }
    #[test]
    fn review_registry_pins_and_all_fixtures_share_the_same_contract_provenance() {
        let reviewed = reviewed_add_manifest();
        let contract = reviewed.contract_digest().unwrap();
        eprintln!(
            "review table BLAKE3={}; add contract BLAKE3={contract}",
            hash(TABLE)
        );
        assert_eq!(contract, ADD_CONTRACT_DIGEST);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../crates/auto-lang/test/cases/ace-ca-d-source-938/src");
        for name in ["add", "bias", "denied", "unreviewed", "wrong-type"] {
            let value: serde_json::Value = serde_json::from_slice(
                &std::fs::read(root.join(format!("{name}.atomdata"))).unwrap(),
            )
            .unwrap();
            let candidate: ManualManifest =
                serde_json::from_value(value["manifest"].clone()).unwrap();
            let capabilities: Vec<String> =
                serde_json::from_value(value["capabilities"].clone()).unwrap();
            assert_eq!(candidate.header_digest, reviewed.header_digest, "{name}");
            assert_eq!(
                candidate.physical_table_digest, reviewed.physical_table_digest,
                "{name}"
            );
            let result = qualify_candidate(&candidate, &capabilities);
            if matches!(name, "denied" | "unreviewed") {
                assert!(
                    result.is_err(),
                    "negative fixture became authorized: {name}"
                );
            } else {
                result.unwrap();
                assert_eq!(
                    candidate.contract_digest().unwrap(),
                    ADD_CONTRACT_DIGEST,
                    "{name}"
                );
            }
        }
    }
}
