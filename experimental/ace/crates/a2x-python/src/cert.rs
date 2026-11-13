//! PLAN-1042: strict Python certification identity, expectation/outcome structs,
//! and manifest integrity verifier (`verify_python_fileset_manifest`).

use ace_core::runtime_contract::sha256_hex;

use crate::provider::PY_PRODUCER;

/// Certificate schema for Python Scalar runs.
pub const PY_CERT_SCHEMA: &str = "ace.a2py-hir.scalar.cert/1";
/// Certificate schema for Python RecordText runs.
pub const PY_RT_CERT_SCHEMA: &str = "ace.a2py-hir.record-text.cert/1";
/// Certificate schema for Python Collections runs.
pub const PY_COLL_CERT_SCHEMA: &str = "ace.a2py-hir.collections.cert/1";

/// Concrete Python interpreter + runtime identity bound into run artifacts and cache keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonRuntimeIdentity {
    pub executable: String,
    pub version: String,
    pub executable_sha256: String,
    pub runtime_sha256: String,
    pub target_triple: String,
}

impl PythonRuntimeIdentity {
    /// Deterministic fingerprint string used for artifact invalidation keys.
    pub fn identity_key(&self) -> String {
        sha256_hex(
            format!(
                "{}\0{}\0{}\0{}\0{}",
                self.executable,
                self.version,
                self.executable_sha256,
                self.runtime_sha256,
                self.target_triple
            )
            .as_bytes(),
        )
    }
}

/// Independent expectation oracle for a Python test execution (never self-reported by target output).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonExecExpectation {
    pub expected_value: Option<i32>,
    pub expected_trap: Option<String>,
    pub expected_stdout_sha256: Option<String>,
    pub expected_stdout_exact: Option<Vec<u8>>,
    pub expected_effect_count: Option<usize>,
}

/// Captured real Python execution outcome from a contained OS process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonExecOutcome {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub value: Option<i32>,
    pub trap: Option<String>,
    pub events_json: Option<String>,
}

/// Verify that `manifest` binds the expected schema, the trusted `PY_PRODUCER`,
/// and exact `(path, sha256)` pairs matching `files`.
pub fn verify_python_fileset_manifest(
    manifest: &str,
    files: &[(String, String)],
    expected_producer: &str,
    expected_schema: &str,
) -> Result<(), String> {
    if expected_producer != PY_PRODUCER {
        return Err(format!(
            "untrusted Python producer `{expected_producer}` (expected `{PY_PRODUCER}`)"
        ));
    }
    let schema_line = format!("\"schema\": \"{expected_schema}\"");
    if !manifest.contains(&schema_line) {
        return Err(format!(
            "manifest missing expected schema `{expected_schema}`"
        ));
    }
    let producer_line = format!("\"producer\": \"{expected_producer}\"");
    if !manifest.contains(&producer_line) {
        return Err(format!(
            "manifest missing expected producer `{expected_producer}`"
        ));
    }
    if manifest.contains("PythonTrans")
        || manifest.contains("crates/auto-lang-core/src/trans/python")
    {
        return Err("legacy PythonTrans origin forbidden in ACE Python manifest".to_string());
    }
    for (path, content) in files {
        let digest = sha256_hex(content.as_bytes());
        let expected_pair = format!("\"path\": \"{path}\", \"sha256\": \"{digest}\"");
        if !manifest.contains(&expected_pair) {
            return Err(format!(
                "manifest file digest mismatch or missing entry for `{path}`"
            ));
        }
    }
    Ok(())
}
