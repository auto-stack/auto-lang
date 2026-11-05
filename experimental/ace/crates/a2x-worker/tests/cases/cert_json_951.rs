//! Parse the producer's actual serialized bytes, not merely its typed draft.
use ace_a2x_rust::{BoundFile, CertDraft, ToolchainIdentity};

fn certificate(schema: &'static str, recipe: &'static str, config: Option<String>) -> CertDraft {
    CertDraft {
        schema,
        recipe,
        case: "json-roundtrip-951".into(),
        profile: "scalar".into(),
        hir_domain: "scalar-i32-draft".into(),
        entry: "main".into(),
        verdict: "PASS".into(),
        oracle_kind: "value",
        oracle_value: "42".into(),
        input_hash: "0123456789abcdef".into(),
        source_sha256: "a".repeat(64),
        actual_files: vec![BoundFile {
            path: "main.rs".into(),
            sha256: "b".repeat(64),
        }],
        golden_sha256: None,
        wrapper_sha256: "c".repeat(64),
        reference_sha256: "d".repeat(64),
        backend: "auto-a2x-rust",
        backend_version: "0.1.0",
        shared_view_version: "0.1.0",
        protocol: "a2x-observation/1",
        toolchain: ToolchainIdentity::observed(
            "rustc fixture\nrelease: fixture".into(),
            vec!["--edition".into(), "2021".into()],
        ),
        capabilities: vec![],
        execution_config: config,
    }
}

fn roundtrip(cert: CertDraft) {
    cert.validate_as(cert.schema, cert.recipe)
        .expect("legitimate typed certificate");
    let bytes = cert.to_json();
    let value: serde_json::Value =
        serde_json::from_str(&bytes).expect("producer must emit valid JSON");
    assert_eq!(value["schema"], cert.schema);
    assert_eq!(value["recipe"], cert.recipe);
    assert_eq!(value["case"], cert.case);
    assert_eq!(value["oracle_value"], cert.oracle_value);
    assert_eq!(
        value["execution_config"],
        serde_json::to_value(&cert.execution_config).unwrap()
    );
    assert_eq!(
        value["capabilities"],
        serde_json::to_value(&cert.capabilities).unwrap()
    );
    assert_eq!(value["toolchain"]["version_vv"], cert.toolchain.version_vv);
    assert_eq!(value["source_sha256"], cert.source_sha256);
    assert_eq!(
        value["actual_files"][0]["sha256"],
        cert.actual_files[0].sha256
    );
    assert_eq!(cert.to_json(), bytes, "same input serializes byte-stably");
    // A permissive parser or repairing consumer would incorrectly accept this.
    let malformed = bytes.replacen("\"execution_config\":", "\"execution_config\"", 1);
    assert_ne!(malformed, bytes);
    assert!(serde_json::from_str::<serde_json::Value>(&malformed).is_err());
}

#[test]
fn scalar_and_record_text_json_roundtrip() {
    for (schema, recipe) in [
        (ace_a2x_rust::CERT_SCHEMA, ace_a2x_rust::RECIPE),
        (ace_a2x_rust::RT_CERT_SCHEMA, ace_a2x_rust::RT_RECIPE),
    ] {
        for capabilities in [vec![], vec!["trace".into(), "resource".into()]] {
            let mut cert = certificate(schema, recipe, None);
            if schema == ace_a2x_rust::RT_CERT_SCHEMA {
                cert.profile = "record".into();
                cert.hir_domain = "record-i32-draft".into();
            }
            cert.capabilities = capabilities;
            cert.case = "quoted \"case\"\\path\nnext\tcolumn\rreturn".into();
            roundtrip(cert);
        }
    }
}

#[test]
fn collections_config_json_roundtrip() {
    for mode in ["debug", "release"] {
        for capabilities in [vec![], vec!["trace".into(), "resource".into()]] {
            let config =
                ace_a2x_rust::canonical_execution_config(mode, 16777216, Some(3), true, false);
            let mut cert = certificate(
                ace_a2x_rust::COLL_CERT_SCHEMA,
                ace_a2x_rust::COLL_RECIPE,
                Some(config),
            );
            cert.profile = "collections".into();
            cert.hir_domain = "collections-draft".into();
            cert.capabilities = capabilities;
            roundtrip(cert);
        }
    }
}
