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
    let policy = reviewed_add_manifest();
    Ok(HostAuthority {
        review_id: REVIEW.into(),
        contract_digest: ADD_CONTRACT_DIGEST.into(),
        provider: policy.provider,
        provider_digest: provider_digest.into(),
        header_digest: hash(HEADER),
        physical_table_digest: hash(TABLE),
        target: ace_foreign_contract::plan::TARGET.into(),
        capabilities: caps.iter().cloned().collect(),
        available_symbols: ["provider_add".into()].into_iter().collect(),
        binding_generation: generation,
    })
}
pub fn qualify_candidate(candidate: &ManualManifest, caps: &[String]) -> Result<(), String> {
    candidate.validate()?;
    if candidate != &reviewed_add_manifest() {
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
pub const EVENT_RUNTIME:&str="#include <stdio.h>\nvoid source938_event(const char *name) { fprintf(stderr, \"source938.call=%s\\n\",name); fflush(stderr); }\n";

#[cfg(test)]
mod tests {
    use super::*;
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
