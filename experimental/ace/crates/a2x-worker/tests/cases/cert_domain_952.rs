//! Real producer profile/domain identity regression (F-941-02).
use std::path::PathBuf;
use std::process::Command;

fn check(profile: &str, domain: &str) {
    let home = std::env::var_os("AUTO_PLAN_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let root = home.join(format!("cert-domain-952-{profile}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let source = root.join("source.at");
    let reference = root.join("reference-input.rs");
    // One closed-form oracle across all domains isolates identity from syntax.
    std::fs::write(&source, "fn main() int { return 42 }\n").unwrap();
    std::fs::write(&reference, "fn main() { println!(\"value:42\"); }\n").unwrap();
    let actual = root.join("actual");
    let request = serde_json::json!({
        "op": "certify", "profile": profile, "source_path": source,
        "reference_path": reference, "out_dir": actual,
        "entry": "main", "caps": [], "oracle_kind": "value", "oracle_value": 42,
        "case_id": format!("952/domain/{profile}"),
    });
    let output = Command::new(env!("CARGO_BIN_EXE_a2x-worker"))
        .arg(request.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{profile}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["actual_observed"], "value:42");
    assert_eq!(response["reference_observed"], "value:42");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(actual.join("manifest.json")).unwrap()).unwrap();
    let cert: serde_json::Value =
        serde_json::from_slice(&std::fs::read(actual.join("case.cert.json")).unwrap()).unwrap();
    assert_eq!(manifest["hir_domain"], domain, "formal domain of {profile}");
    assert_eq!(
        cert["hir_domain"], domain,
        "certificate must bind actual {profile} domain"
    );
    assert_eq!(cert["hir_domain"], manifest["hir_domain"]);
    assert_eq!(cert["profile"], manifest["source_profile"]);
}

#[test]
fn core() {
    check("core", "core-i32-draft");
}
#[test]
fn scalar() {
    check("scalar", "scalar-i32-draft");
}
#[test]
fn record() {
    check("record", "record-i32-draft");
}
#[test]
fn text() {
    check("text", "text-i32-draft");
}
#[test]
fn text_record() {
    check("text-record", "text-record-i32-draft");
}
#[test]
fn collections() {
    check("collections", "collections-draft");
}
