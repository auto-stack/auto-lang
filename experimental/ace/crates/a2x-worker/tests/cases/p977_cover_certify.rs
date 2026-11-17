//! PLAN-977 B-REVERSE-V / AC-COVERAGE: the A2R and A2C **run** legs for the
//! cover-977 public source (`fn main() int { return 2 + 3 * 4 }`, expected
//! value 14, hand-computed).
//!
//! The daily case legs for these recipes are *emission* locks (worker
//! identity + byte goldens). This file is the explicit CERTIFY axis for the
//! same source: the worker emits, then REALLY compiles and runs (a) the
//! emitted program and (b) a HAND-WRITTEN reference in the target language
//! (`reference-cover.rs` / `reference-cover.c`), each compared to the
//! closed-form oracle `value:14`. The two legs are compiled and run
//! independently; the reference is never derived from the transpiler
//! output.
//!
//! Run: `cargo t ace-a2x --test-name p977_cover_certify` (the group's
//! preparation builds the release worker these tests spawn).

use std::path::PathBuf;
use std::process::Command;

const COVER_DIR: &str = "crates/auto-lang/test/cases/ace-m2/cover-977";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

fn certify(target: &str) {
    let root = repo_root();
    let cover = root.join(COVER_DIR);
    let home = std::env::var_os("AUTO_PLAN_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let output = home.join(format!(
        "p977-cover-certify-{target}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&output);
    std::fs::create_dir_all(&output).expect("certify scratch dir");
    let reference = if target == "c" {
        cover.join("reference-cover.c")
    } else {
        cover.join("reference-cover.rs")
    };
    let request = serde_json::json!({
        "op": if target == "c" { "certify-c" } else { "certify" },
        "source_path": cover.join("src/scalar.at"),
        "profile": "scalar",
        "entry": "main",
        "caps": [],
        "out_dir": output,
        "reference_path": reference,
        "oracle_kind": "value",
        "oracle_value": 14,
        "case_id": "977/cover-scalar",
        "nonce": "p977-cover-certify",
    });

    let identity_output = Command::new(env!("CARGO_BIN_EXE_a2x-worker"))
        .arg(serde_json::json!({ "op": "identity", "certification_target": target }).to_string())
        .output()
        .expect("worker identity spawn");
    assert!(
        identity_output.status.success(),
        "identity ({target}): {}",
        String::from_utf8_lossy(&identity_output.stderr)
    );
    let identity: serde_json::Value =
        serde_json::from_slice(&identity_output.stdout).expect("identity json");

    let result = Command::new(env!("CARGO_BIN_EXE_a2x-worker"))
        .arg(request.to_string())
        .env("AUTO_CASE_WIRE_EVIDENCE", "1")
        .output()
        .expect("worker certify spawn");
    assert!(
        result.status.success(),
        "certify ({target}): {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&result.stdout).expect("certify json");
    assert_eq!(response["status"], "ok", "{response}");
    // The two certify ops spell the closed-form oracle differently (the
    // Rust op renders `value:14`, the C op echoes kind + value); both are
    // the same hand-computed expectation.
    if target == "c" {
        assert_eq!(response["oracle_kind"], "value", "{response}");
        assert_eq!(response["oracle_value"], "14", "{response}");
    } else {
        assert_eq!(response["oracle"], "value:14", "{response}");
    }
    assert_eq!(response["verdict"], "PASS", "{response}");

    let certificate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("case.cert.json")).expect("certificate"))
            .expect("certificate json");
    assert_eq!(
        certificate["toolchain"], identity["certification_toolchain"],
        "the certificate must carry the observed toolchain identity"
    );

    let observations = &response["execution_observations"];
    assert_eq!(observations["schema"], "ace.a2x.cert-execution/1");
    for leg in ["actual", "reference"] {
        let compilation = &observations[leg]["compile"];
        let run = &observations[leg]["run"];
        assert_eq!(compilation["exit_code"], 0, "{target}/{leg} compiles");
        assert_eq!(run["exit_code"], 0, "{target}/{leg} runs");
        assert_eq!(
            run["stdout_hex"], "76616c75653a31340a",
            "{target}/{leg} prints the independent value:14"
        );
        assert_eq!(
            run["stderr_hex"], "",
            "{target}/{leg} has no hidden marks in the scalar oracle"
        );
        assert!(
            run.get("verdict").is_none(),
            "{target}/{leg}: the observation is not a verdict"
        );
    }
}

#[test]
fn p977_cover_a2r_certify_real_rustc() {
    certify("rust");
}

#[test]
fn p977_cover_a2c_certify_real_cl() {
    certify("c");
}
