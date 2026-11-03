//! PLAN-925 T-08 (V-10/11): the C RECORD-TEXT certification contract —
//! the real two-leg certification (actual + hand-written reference, both
//! compiled and run independently under the real MSVC cl against one
//! closed-form oracle) through the record-text channel's OWN identities
//! (recipe `ac-a2c/hir-record-text-emit-v1`, schema
//! `ace.a2c-hir.record-text.cert/1`), and the STRICT bad-evidence
//! controls: wrong oracle, cross-CHANNEL impersonation (a scalar
//! schema/recipe pair under the record-text cert and the reverse), trimmed
//! evidence and non-PASS verdicts are all REJECTED.

use std::path::PathBuf;
use std::process::Command;

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/a2c-hir-record-text-925")
}

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c-rt/cert-925")
}

fn worker_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/a2x-worker.exe");
    assert!(p.exists(), "prebuilt release worker required (rebuild it)");
    p
}

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
    (out.status.code() == Some(0) && json["status"] == "ok", json)
}

fn base_req(tag: &str) -> serde_json::Value {
    let dir = out_base().join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    serde_json::json!({
        "op": "certify-c",
        "source_path": suite_dir().join("src/R07.at").display().to_string(),
        "profile": "record",
        "entry": "main",
        "caps": [],
        "out_dir": dir.display().to_string(),
        "reference_path": suite_dir().join("reference-R07.c").display().to_string(),
        "oracle_kind": "value",
        "oracle_value": 42,
        "case_id": "R07",
        "nonce": "cert-rt",
    })
}

/// The two-leg pass through the RECORD-TEXT channel identities; byte-stable
/// certificate serialization; the emitted daily set carries the rt fileset.
#[test]
fn rt_certify_two_leg_pass_and_stable_bytes() {
    let (ok, r) = certify(&base_req("pass-a"));
    assert!(ok, "{}", r);
    assert_eq!(r["verdict"], "PASS");
    assert_eq!(r["actual_stdout"], "value:42");
    assert_eq!(r["reference_stdout"], "value:42");
    assert_eq!(r["recipe"], "ac-a2c/hir-record-text-emit-v1");
    assert_eq!(r["spawned_rustc"], 0, "the C channel never spawns rustc");
    assert!(r["spawned_cl"].as_i64().unwrap() >= 2, "two real cl legs");
    let cert_a = std::fs::read_to_string(r["cert_path"].as_str().unwrap()).unwrap();
    assert!(cert_a.contains("ace.a2c-hir.record-text.cert/1"));
    assert!(cert_a.contains("\"backend\": \"auto-a2x-c\""));
    assert!(cert_a.contains("\"standard\": \"c11\""));
    assert!(cert_a.contains("\"vcvars_sha256\""));
    // the actual set is the RECORD-TEXT fileset (rt runtime header)
    let dir = out_base().join("pass-a");
    assert!(dir.join("a2x_c_rt_runtime.h").is_file());
    let (_, r2) = certify(&base_req("pass-b"));
    let cert_b = std::fs::read_to_string(r2["cert_path"].as_str().unwrap()).unwrap();
    assert_eq!(cert_a, cert_b, "cert serialization is byte-stable");
}

/// A WRONG oracle is refused (both legs observe the closed-form oracle).
#[test]
fn rt_certify_wrong_oracle_refused() {
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

/// STRICT cross-channel controls: the schema/recipe PAIR must match the
/// channel — a scalar pair under the record-text schema (and the reverse)
/// is rejected tamper, as are trimmed files and non-PASS verdicts.
#[test]
fn rt_cert_draft_strict_controls() {
    use ace_a2x_c::cert::{
        BoundFile, CCertDraft, CToolchainIdentity, C_CERT_SCHEMA, C_RT_CERT_SCHEMA,
    };
    fn draft(schema: &str, recipe: &str) -> CCertDraft {
        CCertDraft {
            schema: schema.to_string(),
            recipe: recipe.to_string(),
            case: "a2c-hir-925/R07".into(),
            profile: "record".into(),
            hir_domain: "auto.hir.text-record".into(),
            entry: "main".into(),
            verdict: "PASS".into(),
            oracle_kind: "value".into(),
            oracle_value: "42".into(),
            input_hash: "ih".into(),
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
                cl_version: "19.xx".into(),
                cl_hash: "e".repeat(64),
                vcvars_sha256: "f".repeat(64),
                flags: vec!["-nologo".into()],
            },
            capabilities: vec![],
        }
    }
    // the exact pairing validates
    assert!(draft(C_RT_CERT_SCHEMA, "ac-a2c/hir-record-text-emit-v1")
        .validate()
        .is_ok());
    // cross-channel impersonation is rejected BOTH ways
    assert!(
        draft(C_RT_CERT_SCHEMA, "ac-a2c/hir-scalar-emit-v1")
            .validate()
            .is_err(),
        "a scalar recipe under the record-text schema is tamper"
    );
    assert!(
        draft(C_CERT_SCHEMA, "ac-a2c/hir-record-text-emit-v1")
            .validate()
            .is_err(),
        "the record-text recipe under the scalar schema is tamper"
    );
    // old/foreign schemas stay untrusted
    assert!(draft("ace.m1.cert/1", "ac-a2c/hir-record-text-emit-v1")
        .validate()
        .is_err());
    assert!(draft(
        "ace.a2r-hir.record-text.cert/1",
        "ac-a2c/hir-record-text-emit-v1"
    )
    .validate()
    .is_err());
    // trimmed evidence / non-PASS verdicts
    let mut d = draft(C_RT_CERT_SCHEMA, "ac-a2c/hir-record-text-emit-v1");
    d.actual_files.clear();
    assert!(d.validate().is_err(), "trimmed actual files rejected");
    let mut d = draft(C_RT_CERT_SCHEMA, "ac-a2c/hir-record-text-emit-v1");
    d.verdict = "MAYBE".into();
    assert!(d.validate().is_err(), "non-PASS/REJECTED verdict rejected");
}
