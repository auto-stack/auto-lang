//! PLAN-921 T-10 (V-08): the C scalar CERTIFICATION contract — the real
//! two-leg certification (actual + hand-written reference, both compiled
//! and run independently under the real MSVC cl against one closed-form
//! oracle) and the STRICT bad-evidence controls: wrong oracle, old-schema
//! impersonation, trimmed/blank evidence, non-PASS verdicts and incomplete
//! toolchain identities are all REJECTED. The daily channel stays
//! spawn-zero (V-09's leg lives in the suite adapter).

use std::path::PathBuf;
use std::process::Command;

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/a2c-hir-scalar-921")
}

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c/cert-921")
}

fn worker_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/a2x-worker.exe");
    if p.exists() {
        return p;
    }
    p.join("../../../target/release/a2x-worker.exe")
}

/// Drive the worker's certify-c op; returns (exit ok, parsed json or the
/// error detail).
fn certify(req: &serde_json::Value) -> (bool, serde_json::Value) {
    use std::io::Write as _;
    let mut proc = Command::new(worker_path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("worker spawn");
    proc.stdin
        .as_mut()
        .unwrap()
        .write_all(req.to_string().as_bytes())
        .expect("write request");
    let out = proc.wait_with_output().expect("worker complete");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_default();
    (out.status.code() == Some(0), json)
}

fn base_req(tag: &str) -> serde_json::Value {
    let dir = out_base().join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    serde_json::json!({
        "op": "certify-c",
        "source_path": suite_dir().join("src/C01.at").display().to_string(),
        "profile": "scalar",
        "entry": "main",
        "caps": [],
        "out_dir": dir.display().to_string(),
        "reference_path": suite_dir().join("reference-C01.c").display().to_string(),
        "oracle_kind": "value",
        "oracle_value": 42,
        "case_id": "C01",
        "nonce": "cert",
    })
}

/// The two-leg pass: actual and reference each REALLY compile and run
/// under cl and match the oracle; the cert validates; serialization is
/// byte-stable across runs.
#[test]
fn c_certify_two_leg_pass_and_stable_bytes() {
    let (ok, r) = certify(&base_req("pass-a"));
    assert!(ok, "{}", r);
    assert_eq!(r["verdict"], "PASS");
    assert_eq!(r["actual_stdout"], "value:42");
    assert_eq!(r["reference_stdout"], "value:42");
    assert_eq!(r["recipe"], "ac-a2c/hir-scalar-emit-v1");
    assert_eq!(r["spawned_rustc"], 0, "the C channel never spawns rustc");
    assert!(r["spawned_cl"].as_i64().unwrap() >= 2, "two real cl legs");
    let cert_a = std::fs::read_to_string(r["cert_path"].as_str().unwrap()).unwrap();
    assert!(cert_a.contains("ace.a2c-hir.scalar.cert/1"));
    assert!(cert_a.contains("\"backend\": \"auto-a2x-c\""));
    assert!(cert_a.contains("\"standard\": \"c11\""));
    assert!(cert_a.contains("\"vcvars_sha256\""));
    // byte-stable: a second certification of the same identity produces
    // the same certificate bytes
    let (_, r2) = certify(&base_req("pass-b"));
    let cert_b = std::fs::read_to_string(r2["cert_path"].as_str().unwrap()).unwrap();
    assert_eq!(cert_a, cert_b, "cert serialization is byte-stable");
}

/// A WRONG oracle is refused on BOTH legs (the actual observation must
/// equal the claimed closed-form oracle).
#[test]
fn c_certify_wrong_oracle_refused() {
    let mut req = base_req("wrong-oracle");
    req["oracle_value"] = 43.into();
    let (ok, r) = certify(&req);
    assert!(!ok, "wrong oracle must fail: {}", r);
    assert!(
        r["detail"].as_str().unwrap_or("").contains("a2x.oracle"),
        "{}",
        r
    );
}

/// STRICT bad-evidence controls at the draft level: old-schema
/// impersonation, trimmed actual files, non-PASS verdicts, unknown
/// oracle kinds, wrong backend/standard and incomplete toolchain
/// identities are all REJECTED by validation.
#[test]
fn c_cert_draft_strict_controls() {
    use ace_a2x_c::cert::{BoundFile, CCertDraft, CToolchainIdentity, C_CERT_SCHEMA};
    fn draft() -> CCertDraft {
        CCertDraft {
            schema: C_CERT_SCHEMA.to_string(),
            recipe: "ac-a2c/hir-scalar-emit-v1".to_string(),
            case: "a2c-hir-921/C01".into(),
            profile: "scalar".into(),
            hir_domain: "auto.hir.scalar".into(),
            entry: "main".into(),
            verdict: "PASS".into(),
            oracle_kind: "value".into(),
            oracle_value: "42".into(),
            input_hash: "d9ec0abfb8a930f7".into(),
            source_sha256: "a".repeat(64),
            actual_files: vec![BoundFile {
                path: "main.c".into(),
                sha256: "b".repeat(64),
            }],
            golden_sha256: None,
            wrapper_sha256: "c".repeat(64),
            reference_sha256: "d".repeat(64),
            backend: "auto-a2x-c".into(),
            backend_version: "0.1.0".into(),
            standard: "c11".into(),
            shared_view_version: "0.1.0".into(),
            protocol: "a2x-observation/1".into(),
            toolchain: CToolchainIdentity {
                cl_version: "19.44".into(),
                cl_hash: "e".repeat(64),
                vcvars_sha256: "f".repeat(64),
                flags: vec!["-nologo".into(), "-std:c11".into()],
            },
            execution_config: None,
            capabilities: vec![],
        }
    }
    assert!(draft().validate().is_ok(), "the clean draft passes");

    let mut d = draft();
    d.schema = "ace.m1.cert/1".into(); // old A2R schema impersonation
    assert!(d.validate().is_err());

    let mut d = draft();
    d.schema = "ace.a2r-hir.cert/1".into(); // Rust scalar impersonation
    assert!(d.validate().is_err());

    let mut d = draft();
    d.recipe = "ac-a2r/hir-scalar-emit-v1".into(); // Rust recipe
    assert!(d.validate().is_err());

    let mut d = draft();
    d.actual_files.clear(); // trimmed evidence (compile-only cert)
    assert!(d.validate().is_err());

    let mut d = draft();
    d.verdict = "MAYBE".into();
    assert!(d.validate().is_err());

    let mut d = draft();
    d.oracle_kind = "exit-code".into();
    assert!(d.validate().is_err());

    let mut d = draft();
    d.backend = "auto-a2x-rust".into();
    assert!(d.validate().is_err());

    let mut d = draft();
    d.standard = "c17".into();
    assert!(d.validate().is_err());

    let mut d = draft();
    d.toolchain.cl_hash = "not-a-hash".into();
    assert!(d.validate().is_err());

    let mut d = draft();
    d.toolchain.flags.clear(); // blank toolchain
    assert!(d.validate().is_err());

    let mut d = draft();
    d.actual_files[0].sha256 = "0".repeat(63); // malformed file binding
    assert!(d.validate().is_err());
}
