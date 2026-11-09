//! Original certification observations, separately from the worker's verdict.
use std::path::PathBuf;
use std::process::Command;

#[test]
fn captured_bytes_do_not_use_lossy_or_trimmed_text() {
    let captured = ace_core::link::CapturedRun {
        exit_code: Some(7),
        stdout: vec![0xff, 0, b' ', b'\n'],
        stderr: vec![b'\n', 0xfe],
    };
    let record = ace_a2x_worker::crun::captured_observation(&captured);
    assert_eq!(record["exit_code"], 7);
    assert_eq!(record["stdout_hex"], "ff00200a");
    assert_eq!(record["stderr_hex"], "0afe");
    assert!(record.get("verdict").is_none());
}

fn certify(target: &str) {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let fixture = repo.join("crates/auto-lang/test/cases/ace-m1-expanded");
    let home = std::env::var_os("AUTO_PLAN_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let output = home.join(format!(
        "cert-execution-941-{target}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&output).unwrap();
    let reference = if target == "c" {
        "reference-Q4-001.c"
    } else {
        "reference-Q4-001.rs"
    };
    let request = serde_json::json!({
        "op": if target == "c" { "certify-c" } else { "certify" },
        "source_path": fixture.join("src/Q4-001.at"),
        "profile": "scalar", "entry": "main", "caps": [], "out_dir": output,
        "reference_path": fixture.join(reference),
        "oracle_kind": "value", "oracle_value": 42,
        "case_id": "941/cert-observation-control", "nonce": "941-cert-observation-control",
    });
    let identity_output = Command::new(env!("CARGO_BIN_EXE_a2x-worker"))
        .arg(serde_json::json!({ "op": "identity", "certification_target": target }).to_string())
        .output()
        .unwrap();
    assert!(identity_output.status.success());
    let identity: serde_json::Value = serde_json::from_slice(&identity_output.stdout).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_a2x-worker"))
        .arg(request.to_string())
        .env("AUTO_CASE_WIRE_EVIDENCE", "1")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let certificate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("case.cert.json")).unwrap()).unwrap();
    assert_eq!(
        certificate["toolchain"],
        identity["certification_toolchain"]
    );
    let observations = &response["execution_observations"];
    assert_eq!(observations["schema"], "ace.a2x.cert-execution/1");
    for leg in ["actual", "reference"] {
        let compilation = &observations[leg]["compile"];
        let run = &observations[leg]["run"];
        assert_eq!(compilation["exit_code"], 0, "{leg} did compile");
        assert_eq!(run["exit_code"], 0, "{leg} did run");
        assert_eq!(
            run["stdout_hex"], "76616c75653a34320a",
            "independent value:42 newline"
        );
        assert_eq!(run["stderr_hex"], "", "no hidden marks in scalar oracle");
        assert!(run.get("verdict").is_none());
    }
}

#[test]
fn rust_real_actual_and_reference_raw_terminals() {
    certify("rust");
}

#[test]
fn c_real_actual_and_reference_raw_terminals() {
    certify("c");
}
