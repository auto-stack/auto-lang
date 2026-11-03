//! PLAN-904 T-07/T-08 certify conformance (V-09/V-10/V-16): the explicit
//! two-leg certification of the collections channel through the REAL
//! prebuilt worker — the same-session ACTUAL file set and the case's
//! HAND-WRITTEN std-only reference both really compile and run against
//! the independent oracle; wrong oracles/references are REJECTED; the
//! execution-config identity (mode/budget/fault/events/cancel) is key
//! material of the collections certificate schema (mutually exclusive
//! with the older schemas).

use std::path::PathBuf;
use std::process::Command;

const WORKER: &str = "../../target/release/a2x-worker.exe";

fn worker_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(WORKER);
    if p.exists() {
        return p;
    }
    // dev-tree fallback
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../target/release/a2x-worker.exe")
}

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/cert-coll-904")
}

fn certify(
    case: &str,
    oracle_kind: &str,
    oracle_value: i64,
    reference: &str,
    mode: &str,
) -> serde_json::Value {
    let dir = out_base().join(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dir = std::fs::canonicalize(&dir).unwrap();
    let ref_path = dir.join("reference.rs");
    std::fs::write(&ref_path, reference).unwrap();
    let src_path = dir.join("main.at");
    std::fs::write(
        &src_path,
        "fn main() int {\n    var xs = List<int>.new()\n    xs.push(41)\n    xs.push(1)\n    xs.set(1, 1)\n    return xs.get(0) + xs.get(1)\n}",
    )
    .unwrap();
    let req = serde_json::json!({
        "op": "certify",
        "source_path": src_path.display().to_string(),
        "profile": "collections",
        "entry": "main",
        "caps": [],
        "out_dir": dir.display().to_string(),
        "reference_path": ref_path.display().to_string(),
        "oracle_kind": oracle_kind,
        "oracle_value": oracle_value,
        "case_id": case,
        "mode": mode,
    });
    // stdin form (Windows argv quoting mangles long JSON requests)
    use std::io::Write as _;
    let mut child = std::process::Stdio::piped();
    let _ = &mut child;
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
    assert_eq!(
        out.status.code(),
        Some(0),
        "worker failed: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    serde_json::from_slice(&out.stdout).expect("worker json")
}

/// The L01-shaped reference (independent std-only implementation).
const REF_L01: &str = r#"fn main() {
    let mut xs: Vec<i32> = Vec::new();
    xs.push(41);
    xs.push(1);
    xs[1] = 1;
    println!("value:{}", xs[0] + xs[1]);
}
"#;

#[test]
fn cert_coll_two_leg_pass_and_wrong_oracle_rejected() {
    // PASS: both legs equal the independent oracle (42)
    let r = certify("pass", "value", 42, REF_L01, "debug");
    assert_eq!(r["verdict"], "PASS", "{r}");
    assert_eq!(r["recipe"], "ac-a2r/hir-collections-emit-v1");
    assert_eq!(r["actual_observed"], "value:42");
    assert_eq!(r["reference_observed"], "value:42");
    // the collections certificate carries the execution-config identity
    let cert = std::fs::read_to_string(out_base().join("pass").join("case.cert.json")).unwrap();
    assert!(cert.contains("\"execution_config\": \"{\\\"budget_work_units\\\":16777216"));
    assert!(cert.contains("ace.a2r-hir.collections.cert/1"));

    // WRONG oracle (70 — the value L06-style traps would never produce as
    // a plain value): both legs disagree -> REJECTED
    let r = certify("wrong-oracle", "value", 70, REF_L01, "debug");
    assert_eq!(r["verdict"], "REJECTED", "{r}");

    // WRONG reference (independently computes a different value): the
    // reference leg disagrees with the oracle -> REJECTED
    let bad_ref = REF_L01.replace("value:{}", "value:{} + 1");
    let bad_ref = bad_ref.replace(
        "println!(\"value:{} + 1\", xs[0] + xs[1]);",
        "println!(\"value:{}\", xs[0] + xs[1] + 1);",
    );
    let r = certify("wrong-ref", "value", 42, &bad_ref, "debug");
    assert_eq!(r["verdict"], "REJECTED", "{r}");
}

#[test]
fn cert_coll_trap_case_named_identity() {
    // L06-shaped trap: the ONLY certified outcome is the named ListIndex
    // trap — never a value, a panic, or an arbitrary nonzero
    let dir = out_base().join("trap");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dir = std::fs::canonicalize(&dir).unwrap();
    let ref_path = dir.join("reference.rs");
    std::fs::write(
        &ref_path,
        "fn main() {\n    let xs: Vec<i32> = vec![42];\n    match xs.get(1) {\n        Some(v) => println!(\"value:{}\", v),\n        None => println!(\"trap:ListIndex\"),\n    }\n}\n",
    )
    .unwrap();
    let src_path = dir.join("main.at");
    std::fs::write(
        &src_path,
        "fn main() int {\n    var xs = List<int>.new()\n    xs.push(42)\n    return xs.get(1)\n}",
    )
    .unwrap();
    let req = serde_json::json!({
        "op": "certify",
        "source_path": src_path.display().to_string(),
        "profile": "collections",
        "entry": "main",
        "caps": [],
        "out_dir": dir.display().to_string(),
        "reference_path": ref_path.display().to_string(),
        "oracle_kind": "trap",
        "oracle_trap": "ListIndex",
        "case_id": "trap",
        "mode": "debug",
    });
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
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let r: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(r["verdict"], "PASS", "{r}");
    assert_eq!(r["actual_observed"], "trap:ListIndex");
    assert_eq!(r["reference_observed"], "trap:ListIndex");
}

#[test]
fn cert_coll_mode_is_key_material() {
    // V-16: debug and release are DIFFERENT config identities — the
    // certificate's execution_config differs per mode (separate keys,
    // never cross-trusted)
    let d = certify("mode-debug", "value", 42, REF_L01, "debug");
    assert_eq!(d["verdict"], "PASS");
    let r = certify("mode-release", "value", 42, REF_L01, "release");
    assert_eq!(r["verdict"], "PASS");
    let cert_d =
        std::fs::read_to_string(out_base().join("mode-debug").join("case.cert.json")).unwrap();
    let cert_r =
        std::fs::read_to_string(out_base().join("mode-release").join("case.cert.json")).unwrap();
    assert!(
        cert_d.contains("mode\\\":\\\"debug"),
        "debug config: {cert_d}"
    );
    assert!(
        cert_r.contains("mode\\\":\\\"release"),
        "release config: {cert_r}"
    );
    assert_ne!(cert_d, cert_r, "configs never share one certificate");
}

#[test]
fn cert_coll_schema_isolation() {
    // the collections certificate schema is strictly disjoint from the
    // older channels: a collections draft cannot validate as scalar/rt
    let draft = ace_a2x_rust::CertDraft {
        schema: ace_a2x_rust::COLL_CERT_SCHEMA,
        recipe: ace_a2x_rust::coll::COLL_RECIPE,
        case: "c".into(),
        profile: "auto.source.collections.draft".into(),
        hir_domain: "collections-draft".into(),
        entry: "main".into(),
        verdict: "PASS".into(),
        oracle_kind: "value",
        oracle_value: "42".into(),
        input_hash: "0123456789abcdef".into(),
        source_sha256: "a".repeat(64),
        actual_files: vec![ace_a2x_rust::BoundFile {
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
        toolchain: ace_a2x_rust::ToolchainIdentity::observed(
            "rustc x".into(),
            vec!["--edition".into(), "2021".into()],
        ),
        capabilities: vec![],
        execution_config: Some(ace_a2x_rust::coll::canonical_execution_config(
            "debug", 16777216, None, false, false,
        )),
    };
    assert!(draft
        .validate_as(
            ace_a2x_rust::COLL_CERT_SCHEMA,
            ace_a2x_rust::coll::COLL_RECIPE
        )
        .is_ok());
    // impersonating the older schemas is a named rejection
    assert!(draft
        .validate_as(ace_a2x_rust::CERT_SCHEMA, ace_a2x_rust::RECIPE)
        .is_err());
    assert!(draft
        .validate_as(ace_a2x_rust::RT_CERT_SCHEMA, ace_a2x_rust::RT_RECIPE)
        .is_err());
    // and a collections draft WITHOUT the execution-config identity is
    // invalid (config is key material, never optional here)
    let no_cfg = ace_a2x_rust::CertDraft {
        execution_config: None,
        ..draft
    };
    assert!(no_cfg
        .validate_as(
            ace_a2x_rust::COLL_CERT_SCHEMA,
            ace_a2x_rust::coll::COLL_RECIPE
        )
        .is_err());
    // while the OLDER schemas must NOT carry one
    let old = ace_a2x_rust::CertDraft {
        schema: ace_a2x_rust::CERT_SCHEMA,
        recipe: ace_a2x_rust::RECIPE,
        profile: "auto.source.scalar-i32.draft".into(),
        hir_domain: "scalar-i32-draft".into(),
        execution_config: None,
        ..no_cfg
    };
    assert!(old.validate().is_ok());
}
