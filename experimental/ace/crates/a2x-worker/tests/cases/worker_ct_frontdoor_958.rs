//! Actual CLI regressions: a Session-only fix must not satisfy these cases.
use ace_a2x_core::sha256::sha256_hex;
use auto_tool_execution::{run_command_capture, ExecutionResult, ExitKind, TreeScope};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "worker-ct-958-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove only this test's owned scratch");
    }
}

struct Case {
    name: &'static str,
    value: i32,
    extra_cap: Option<&'static str>,
    rust_reference: &'static str,
    c_reference: &'static str,
}
const CASES: &[Case] = &[
    Case {
        name: "sum42",
        value: 42,
        extra_cap: None,
        rust_reference: "20 + 22",
        c_reference: "20 + 22",
    },
    Case {
        name: "sum43",
        value: 43,
        extra_cap: None,
        rust_reference: "20 + 22 + 1",
        c_reference: "20 + 22 + 1",
    },
    Case {
        name: "boollit",
        value: 7,
        extra_cap: None,
        rust_reference: "if true { 7 } else { 9 }",
        c_reference: "1 ? 7 : 9",
    },
    Case {
        name: "meta2",
        value: 42,
        extra_cap: Some("cte.meta.fields/v1"),
        rust_reference: "[0_i32; 2].len() + 40",
        c_reference: "2 + 40",
    },
    Case {
        name: "meta3",
        value: 43,
        extra_cap: Some("cte.meta.fields/v1"),
        rust_reference: "[0_i32; 3].len() + 40",
        c_reference: "3 + 40",
    },
    Case {
        name: "choose42",
        value: 42,
        extra_cap: Some("ac.source.instances.scalar/v1"),
        rust_reference: "if false { 40 } else { 42 }",
        c_reference: "0 ? 40 : 42",
    },
    Case {
        name: "pick40",
        value: 40,
        extra_cap: Some("ac.source.instances.scalar/v1"),
        rust_reference: "if true { 40 } else { 42 }",
        c_reference: "1 ? 40 : 42",
    },
];

fn request(case: &Case, op: &str, root: &Path) -> Value {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src")
        .join(format!("{}.at", case.name));
    let mut caps = vec!["ac.source.cte.scalar/v1"];
    caps.extend(case.extra_cap);
    json!({"op":op,"source_path":source,"profile":"scalar","entry":"main",
           "caps":caps,"out_dir":root.join(format!("{}-{op}",case.name)),
           "nonce":format!("958-{}-{op}-{}",case.name,root.display())})
}

fn execute(request: &Value, root: &Path) -> (ExecutionResult, Value) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_a2x-worker"));
    command
        .arg(serde_json::to_string(request).unwrap())
        .current_dir(root);
    let result = run_command_capture(
        command,
        TreeScope::ProcessTree,
        Some(Duration::from_secs(360)),
    )
    .expect("original owned process facility and cleanup");
    assert!(
        result.stderr.as_ref().unwrap().is_empty(),
        "worker infrastructure stderr"
    );
    let response =
        serde_json::from_slice(result.stdout.as_ref().unwrap()).expect("real CLI strict JSON");
    (result, response)
}

fn assert_daily(request: &Value, response: &Value) {
    assert_eq!(response["status"], "ok");
    assert_eq!(response["spawned_rustc"], 0);
    if request["op"] == "daily-emit-c" {
        assert_eq!(response["spawned_cl"], 0);
    }
    let source = std::fs::read(request["source_path"].as_str().unwrap()).unwrap();
    assert_eq!(response["source_sha256"], sha256_hex(&source));
    let out = Path::new(request["out_dir"].as_str().unwrap());
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 3);
    for file in files {
        let raw = std::fs::read(out.join(file["path"].as_str().unwrap())).unwrap();
        assert_eq!(file["sha256"], sha256_hex(&raw));
        assert_eq!(file["bytes"], raw.len());
    }
    let manifest = std::fs::read(out.join("manifest.json")).unwrap();
    assert_eq!(response["manifest_sha256"], sha256_hex(&manifest));
    serde_json::from_slice::<Value>(&manifest).expect("complete original manifest JSON");
}

fn certify_request(case: &Case, c: bool, root: &Path) -> Value {
    let extension = if c { "c" } else { "rs" };
    let reference = root.join(format!("{}-reference.{extension}", case.name));
    let text = if c {
        format!("#include <stdio.h>\n#include <fcntl.h>\n#include <io.h>\nint main(void) {{ _setmode(_fileno(stdout), _O_BINARY); printf(\"value:%d\\n\", (int)({})); return 0; }}\n", case.c_reference)
    } else {
        format!(
            "fn main() {{ println!(\"value:{{}}\", {}); }}\n",
            case.rust_reference
        )
    };
    std::fs::write(&reference, text).unwrap();
    let mut request = request(case, if c { "certify-c" } else { "certify" }, root);
    request["reference_path"] = json!(reference);
    request["oracle_kind"] = json!("value");
    request["oracle_value"] = json!(case.value);
    request["case_id"] = json!(format!("958/{}", case.name));
    request
}

fn assert_certified(case: &Case, c: bool, request: &Value, response: &Value) {
    assert_eq!(response["verdict"], "PASS");
    let out = Path::new(request["out_dir"].as_str().unwrap());
    // The mainline worker has no opt-in wire observation extension. Verify
    // the freshly compiled actual AND reference executables directly with the
    // original owned executor instead of trusting their display summaries.
    let binaries = if c {
        [out.join("actual.exe"), out.join("_reference/reference.exe")]
    } else {
        [out.join("a2x_actual.exe"), out.join("a2x_reference.exe")]
    };
    for binary in binaries {
        let raw = run_command_capture(
            Command::new(binary),
            TreeScope::ProcessTree,
            Some(Duration::from_secs(30)),
        )
        .expect("independently execute the freshly compiled leg");
        assert!(matches!(raw.kind, ExitKind::Success));
        assert_eq!(
            raw.stdout.as_deref(),
            Some(format!("value:{}\n", case.value).as_bytes())
        );
        assert_eq!(raw.stderr.as_deref(), Some(b"".as_slice()));
    }
    let raw = std::fs::read(out.join("case.cert.json")).unwrap();
    assert_eq!(response["cert_sha256"], sha256_hex(&raw));
    let cert: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cert["hir_domain"], "scalar-i32-draft");
    assert_eq!(cert["oracle_value"], case.value.to_string());
    assert_eq!(cert["capabilities"], request["caps"]);
    assert_eq!(
        cert["schema"],
        if c {
            "ace.a2c-hir.scalar.cert/1"
        } else {
            "ace.a2r-hir.cert/1"
        }
    );
    assert_eq!(
        cert["source_sha256"],
        sha256_hex(&std::fs::read(request["source_path"].as_str().unwrap()).unwrap())
    );
    for file in cert["actual_files"].as_array().unwrap() {
        assert_eq!(
            file["sha256"],
            sha256_hex(&std::fs::read(out.join(file["path"].as_str().unwrap())).unwrap())
        );
    }
}

#[test]
fn worker_ct_958_daily_seven_sources_both_targets() {
    let scratch = Scratch::new();
    for case in CASES {
        for op in ["daily-emit", "daily-emit-c"] {
            let request = request(case, op, &scratch.0);
            let (result, response) = execute(&request, &scratch.0);
            assert!(
                matches!(result.kind, ExitKind::Success),
                "{} {op}: {response}",
                case.name
            );
            assert_daily(&request, &response);
        }
    }
}

#[test]
fn worker_ct_958_rust_certifies_seven_independent_oracles() {
    let scratch = Scratch::new();
    for case in CASES {
        let request = certify_request(case, false, &scratch.0);
        let (result, response) = execute(&request, &scratch.0);
        assert!(matches!(result.kind, ExitKind::Success), "{response}");
        assert_certified(case, false, &request, &response);
    }
}

#[cfg(windows)]
#[test]
fn worker_ct_958_c_certifies_seven_independent_oracles() {
    let scratch = Scratch::new();
    for case in CASES {
        let request = certify_request(case, true, &scratch.0);
        let (result, response) = execute(&request, &scratch.0);
        assert!(matches!(result.kind, ExitKind::Success), "{response}");
        assert_certified(case, true, &request, &response);
    }
}

#[test]
fn worker_ct_958_capability_refusal_and_ordinary_source_compatibility() {
    let scratch = Scratch::new();
    let plain = scratch.0.join("ordinary.at");
    std::fs::write(&plain, "fn main() int { return 20 + 22 }\n").unwrap();
    for op in ["daily-emit", "daily-emit-c"] {
        let mut request = request(&CASES[0], op, &scratch.0);
        request["caps"] = json!([]);
        let (result, refused) = execute(&request, &scratch.0);
        assert!(matches!(result.kind, ExitKind::Failure(2)));
        assert_eq!(refused["status"], "error");
        assert!(refused["detail"]
            .as_str()
            .unwrap()
            .contains("source.syntax"));
        assert!(!Path::new(request["out_dir"].as_str().unwrap()).exists());
        request["source_path"] = json!(plain);
        let (result, accepted) = execute(&request, &scratch.0);
        assert!(matches!(result.kind, ExitKind::Success));
        assert_daily(&request, &accepted);
    }
}

#[test]
fn worker_ct_958_wrong_oracle_cannot_certify() {
    let scratch = Scratch::new();
    let mut request = certify_request(&CASES[0], false, &scratch.0);
    request["oracle_value"] = json!(43);
    let (result, response) = execute(&request, &scratch.0);
    assert!(matches!(result.kind, ExitKind::Success));
    assert_eq!(response["verdict"], "REJECTED");
    assert_eq!(response["actual_observed"], "value:42");
    assert_eq!(response["reference_observed"], "value:42");
    request["oracle_value"] = json!(42);
    std::fs::write(
        request["reference_path"].as_str().unwrap(),
        "fn main() { println!(\"value:43\"); }\n",
    )
    .unwrap();
    let (result, response) = execute(&request, &scratch.0);
    assert!(matches!(result.kind, ExitKind::Success));
    assert_eq!(response["verdict"], "REJECTED");
    assert_eq!(response["actual_observed"], "value:42");
    assert_eq!(response["reference_observed"], "value:43");
}

#[test]
fn worker_ct_958_certification_capability_and_ordinary_source_control() {
    let scratch = Scratch::new();
    let plain = scratch.0.join("ordinary.at");
    std::fs::write(&plain, "fn main() int { return 20 + 22 }\n").unwrap();
    for c in [false, true] {
        if c && !cfg!(windows) {
            continue;
        }
        let mut request = certify_request(&CASES[0], c, &scratch.0);
        request["caps"] = json!([]);
        let (result, response) = execute(&request, &scratch.0);
        assert!(matches!(result.kind, ExitKind::Failure(2)));
        assert_eq!(response["status"], "error");
        assert!(response["detail"]
            .as_str()
            .unwrap()
            .contains("source.syntax"));
        assert!(!Path::new(request["out_dir"].as_str().unwrap()).exists());
        request["source_path"] = json!(plain);
        let (result, response) = execute(&request, &scratch.0);
        assert!(matches!(result.kind, ExitKind::Success));
        assert_certified(&CASES[0], c, &request, &response);
    }
}

#[cfg(windows)]
#[test]
fn worker_ct_958_c_rejects_wrong_oracle_and_reference() {
    let scratch = Scratch::new();
    let mut request = certify_request(&CASES[0], true, &scratch.0);
    request["oracle_value"] = json!(43);
    let (result, response) = execute(&request, &scratch.0);
    assert!(matches!(result.kind, ExitKind::Failure(2)));
    assert!(response["detail"]
        .as_str()
        .unwrap()
        .contains("a2x.oracle: actual"));
    request["oracle_value"] = json!(42);
    std::fs::write(request["reference_path"].as_str().unwrap(),
        "#include <stdio.h>\n#include <fcntl.h>\n#include <io.h>\nint main(void) { _setmode(_fileno(stdout), _O_BINARY); printf(\"value:43\\n\"); return 0; }\n").unwrap();
    let (result, response) = execute(&request, &scratch.0);
    assert!(matches!(result.kind, ExitKind::Failure(2)));
    assert!(response["detail"]
        .as_str()
        .unwrap()
        .contains("a2x.oracle: reference"));
}
