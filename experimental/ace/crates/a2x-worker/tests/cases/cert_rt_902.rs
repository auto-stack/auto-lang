//! PLAN-902 T-06 record-text CERTIFICATION controls (V-09/V-10): the
//! explicit certify axis for the record-text channel — the same actual
//! file set REALLY compiled and run against an INDEPENDENT hand-written
//! reference and the closed-form oracle; the strict certificate draft
//! (`ace.a2r-hir.record-text.cert/1`) binds the actual files, reference
//! bytes and the OBSERVED toolchain identity; wrong oracles, wrong
//! references, late failures, cross-channel impersonation and tampered
//! cert shapes are REJECTED, never passed. The zero-spawn daily contract
//! is judged parent-side by the registered case group (spawned_rustc==0
//! asserted there); the protocol-level certify legs additionally run
//! through the REAL prebuilt worker when it is available.

use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{
    assemble_record_text_file_set, lower_program, CertDraft, ToolchainIdentity, CERT_SCHEMA,
    RECIPE, RT_CERT_SCHEMA, RT_RECIPE,
};
use ace_core::source::compile_text_record_source;
use std::path::{Path, PathBuf};
use std::process::Command;

const T03: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/T03.at");
const ORACLE: i32 = 23;

/// Independent hand-written reference (the frozen suite copy's bytes:
/// std String form, NOT the generated runtime).
const REFERENCE_T03: &str = r#"// PLAN-902 hand-written independent reference — T03 (std String form,
// independent of the generated a2x runtime): clone, move, concat with
// replacement, Unicode-scalar lengths ("ab"=2, "ab中"=3).
fn main() {
    let a: &str = "ab";
    let b = a.to_string();
    let mut c = b; // move: b dies here
    c.push('中');
    let alen = a.chars().count() as i32;
    let clen = c.chars().count() as i32;
    println!("value:{}", alen * 10 + clen);
}
"#;

/// A WRONG reference (independently computes a different value).
const REFERENCE_WRONG: &str = "fn main() { println!(\"value:3\"); }\n";

/// A LATE-FAILURE reference: prints the correct observation, then exits
/// non-zero — a successful observation followed by an abnormal exit is
/// never a PASS.
const REFERENCE_LATE_EXIT: &str = "fn main() { println!(\"value:23\"); std::process::exit(1); }\n";

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/cert-rt-902")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Compile-and-run one leg; returns Err(detail) for a NON-ZERO exit after
/// output (the worker's controlled refusal shape — never a language pass).
fn compile_run(dir: &Path, src_name: &str, exe_name: &str) -> Result<String, String> {
    let exe = dir.join(exe_name);
    let mut compiler = Command::new(rustc());
    compiler
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg(src_name)
        .current_dir(dir);
    let out = auto_tool_execution::run_command_output(compiler).expect("spawn rustc");
    assert!(
        out.status.success(),
        "rustc failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&exe).current_dir(dir).output().expect("run");
    if !run.status.success() {
        return Err(format!(
            "exit code {:?} after output {:?}",
            run.status.code(),
            String::from_utf8_lossy(&run.stdout)
        ));
    }
    Ok(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

fn write(dir: &Path, name: &str, content: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(name), content).unwrap();
}

/// The judge of the certify axis: actual and reference each really
/// compile+run; PASS only when BOTH observed lines equal the oracle AND
/// neither leg aborted after observing.
fn judge(actual_dir: &Path, ref_src: &str, oracle: i32) -> (String, Result<String, String>, bool) {
    let actual = compile_run(actual_dir, "main.rs", "rt_actual.exe").expect("actual leg");
    let ref_dir = actual_dir.join("ref");
    write(&ref_dir, "main.rs", ref_src);
    let reference = compile_run(&ref_dir, "main.rs", "rt_ref.exe");
    let pass = match &reference {
        Ok(line) => line == &format!("value:{oracle}") && actual == format!("value:{oracle}"),
        Err(_) => false,
    };
    (actual, reference, pass)
}

fn rt_cert_draft(
    case: &str,
    verdict: &str,
    files: &[(String, String)],
    reference_bytes: &str,
    vv: &str,
) -> CertDraft {
    CertDraft {
        schema: RT_CERT_SCHEMA,
        recipe: RT_RECIPE,
        case: case.to_string(),
        profile: "auto.source.text-record.draft".to_string(),
        hir_domain: "text-record-i32-draft".to_string(),
        entry: "main".to_string(),
        verdict: verdict.to_string(),
        oracle_kind: "value",
        oracle_value: ORACLE.to_string(),
        input_hash: "57d3dd675332ceb5".to_string(),
        source_sha256: sha256_hex(T03.as_bytes()),
        actual_files: files
            .iter()
            .map(|(p, c)| ace_a2x_rust::BoundFile {
                path: p.clone(),
                sha256: sha256_hex(c.as_bytes()),
            })
            .collect(),
        golden_sha256: None,
        wrapper_sha256: sha256_hex(
            files
                .iter()
                .find(|(p, _)| p == "main.rs")
                .map(|(_, c)| c.as_bytes())
                .unwrap_or(&[]),
        ),
        reference_sha256: sha256_hex(reference_bytes.as_bytes()),
        backend: "auto-a2x-rust",
        backend_version: "0.1.0",
        shared_view_version: "0.1.0",
        protocol: "a2x-observation/1",
        toolchain: ToolchainIdentity::observed(
            vv.to_string(),
            vec!["--edition".into(), "2021".into()],
        ),
        capabilities: vec!["ac.runtime.text.v1".to_string()],
    }
}

/// V-09 PASS leg: the frozen T03 through the record-text channel — actual
/// and the independent reference both really compile and run to the
/// closed-form oracle; the strict rt certificate validates and serializes
/// byte-stably.
#[test]
fn rt_certify_pass_leg_and_binding() {
    let compiled =
        compile_text_record_source(T03, "cert", &["ac.runtime.text.v1".to_string()]).unwrap();
    let view = SemanticView::from_checked(&compiled.checked).unwrap();
    let program = lower_program(&view, "main").unwrap();
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");
    let dir = out_root().join("pass");
    let _ = std::fs::remove_dir_all(&dir);
    for (p, c) in &fs.files {
        write(&dir, p, c);
    }
    std::fs::write(dir.join("manifest.json"), &fs.manifest).unwrap();

    let (actual, reference, pass) = judge(&dir, REFERENCE_T03, ORACLE);
    assert_eq!(actual, "value:23");
    assert_eq!(reference.as_deref(), Ok("value:23"));
    assert!(pass, "both legs agree with the closed-form oracle");

    // observed toolchain identity + the strict rt certificate
    let vv = String::from_utf8_lossy(
        &Command::new(rustc())
            .arg("-vV")
            .output()
            .expect("rustc -vV")
            .stdout,
    )
    .to_string();
    let cert = rt_cert_draft(
        "a2r-hir-record-text-902.t03",
        "PASS",
        &fs.files,
        REFERENCE_T03,
        &vv,
    );
    cert.validate_as(RT_CERT_SCHEMA, RT_RECIPE)
        .expect("valid rt cert");
    assert_eq!(cert.to_json(), cert.to_json(), "byte-stable serialization");
    assert_eq!(cert.toolchain.version_vv_sha256.len(), 64);
    assert_eq!(cert.toolchain.flags, vec!["--edition", "2021"]);
    assert_eq!(cert.reference_sha256, sha256_hex(REFERENCE_T03.as_bytes()));
}

/// V-09: a WRONG oracle rejects (both legs observe 23; the oracle says 43).
#[test]
fn rt_certify_wrong_oracle_rejected() {
    let compiled =
        compile_text_record_source(T03, "cert", &["ac.runtime.text.v1".to_string()]).unwrap();
    let view = SemanticView::from_checked(&compiled.checked).unwrap();
    let program = lower_program(&view, "main").unwrap();
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");
    let dir = out_root().join("wrong-oracle");
    let _ = std::fs::remove_dir_all(&dir);
    for (p, c) in &fs.files {
        write(&dir, p, c);
    }
    let (actual, reference, pass) = judge(&dir, REFERENCE_T03, 43);
    assert_eq!(actual, "value:23");
    assert!(reference.is_ok());
    assert!(!pass, "a wrong oracle must REJECT");
}

/// V-09: a WRONG reference rejects (actual matches the oracle; the
/// independent reference does not).
#[test]
fn rt_certify_wrong_reference_rejected() {
    let compiled =
        compile_text_record_source(T03, "cert", &["ac.runtime.text.v1".to_string()]).unwrap();
    let view = SemanticView::from_checked(&compiled.checked).unwrap();
    let program = lower_program(&view, "main").unwrap();
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");
    let dir = out_root().join("wrong-ref");
    let _ = std::fs::remove_dir_all(&dir);
    for (p, c) in &fs.files {
        write(&dir, p, c);
    }
    let (actual, reference, pass) = judge(&dir, REFERENCE_WRONG, ORACLE);
    assert_eq!(actual, "value:23");
    assert_eq!(reference.as_deref(), Ok("value:3"));
    assert!(!pass, "a wrong reference must REJECT");
}

/// V-09: a LATE FAILURE (correct observation, then a non-zero exit) is a
/// controlled refusal — never a PASS.
#[test]
fn rt_certify_late_failure_not_pass() {
    let compiled =
        compile_text_record_source(T03, "cert", &["ac.runtime.text.v1".to_string()]).unwrap();
    let view = SemanticView::from_checked(&compiled.checked).unwrap();
    let program = lower_program(&view, "main").unwrap();
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");
    let dir = out_root().join("late-fail");
    let _ = std::fs::remove_dir_all(&dir);
    for (p, c) in &fs.files {
        write(&dir, p, c);
    }
    let (actual, reference, pass) = judge(&dir, REFERENCE_LATE_EXIT, ORACLE);
    assert_eq!(actual, "value:23");
    assert!(reference.is_err(), "the abnormal exit must be a refusal");
    assert!(
        !pass,
        "an observation followed by an abnormal exit never passes"
    );
}

/// V-09/V-10: cross-channel impersonation and tampered cert shapes are
/// REJECTED by strict validation — the scalar identities never validate a
/// record-text certificate and vice versa.
#[test]
fn rt_certify_strict_validation_negatives() {
    let vv = "rustc 1.98.0 (88d9e12ae 2026-08-18)\nbinary: rustc\n";
    let files = vec![
        ("main.rs".to_string(), "fn main() {}".to_string()),
        ("a2x_runtime.rs".to_string(), "// rt".to_string()),
        ("generated.rs".to_string(), "// rt".to_string()),
    ];
    let good = rt_cert_draft("c", "PASS", &files, REFERENCE_T03, vv);
    good.validate_as(RT_CERT_SCHEMA, RT_RECIPE).expect("valid");

    // scalar identities must NOT validate an rt certificate
    let err = good
        .validate_as(CERT_SCHEMA, RECIPE)
        .expect_err("cross-channel impersonation");
    assert!(err.contains("schema must be"), "{err}");
    // tampered verdict
    let bad_verdict = rt_cert_draft("c", "MAYBE", &files, REFERENCE_T03, vv);
    assert!(bad_verdict.validate_as(RT_CERT_SCHEMA, RT_RECIPE).is_err());
    // empty file binding
    let empty_files = rt_cert_draft("c", "PASS", &[], REFERENCE_T03, vv);
    assert!(empty_files.validate_as(RT_CERT_SCHEMA, RT_RECIPE).is_err());
    // missing toolchain identity
    let no_tool = CertDraft {
        toolchain: ToolchainIdentity {
            kind: "rustc",
            version_vv: String::new(),
            version_vv_sha256: String::new(),
            flags: vec![],
        },
        ..rt_cert_draft("c", "PASS", &files, REFERENCE_T03, vv)
    };
    assert!(no_tool.validate_as(RT_CERT_SCHEMA, RT_RECIPE).is_err());
}

/// V-10: the PROTOCOL-level certify legs through the REAL prebuilt worker
/// (the same tool identity the registered case group spawns). Skipped
/// with the maintenance pointer when the tool has not been prepared — the
/// case group (`tools = ["a2x"]`) always runs it for real.
#[test]
fn rt_certify_protocol_legs_via_prebuilt_worker() {
    let worker = std::env::var("AC_A2X_HIR_WORKER")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            let mut dir: Option<PathBuf> = Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
            while let Some(d) = dir {
                let exe = d
                    .join("experimental/ace/target/release")
                    .join(if cfg!(windows) {
                        "a2x-worker.exe"
                    } else {
                        "a2x-worker"
                    });
                if exe.is_file() {
                    return Some(exe);
                }
                dir = d.parent().map(|p| p.to_path_buf());
            }
            None
        });
    let Some(worker) = worker else {
        eprintln!(
            "SKIP: prebuilt a2x-worker not found — run the fixed group \
             case-a2r-hir-record-text-902 (tools=[a2x]) or build: \
             cargo build --release --manifest-path experimental/ace/Cargo.toml \
             -p ace-a2x-worker --bin a2x-worker"
        );
        return;
    };

    let src_path = out_root().join("t03.at");
    write(&out_root(), "t03.at", T03);
    let ref_path = out_root().join("reference-T03.rs");
    write(&out_root(), "reference-T03.rs", REFERENCE_T03);

    let run_certify = |oracle: i64| -> String {
        // JSON-escape Windows path separators in the request payloads
        let esc = |p: &Path| -> String { p.display().to_string().replace('\\', "\\\\") };
        let req = format!(
            "{{\"op\":\"certify\",\"source_path\":\"{}\",\"profile\":\"text-record\",\"entry\":\"main\",\"caps\":[\"ac.runtime.text.v1\"],\"out_dir\":\"{}\",\"reference_path\":\"{}\",\"oracle_kind\":\"value\",\"oracle_value\":{},\"case_id\":\"rt-protocol\"}}",
            esc(&src_path),
            esc(&out_root().join(format!("proto-{}", oracle))),
            esc(&ref_path),
            oracle
        );
        let out = Command::new(&worker)
            .arg(req)
            .output()
            .expect("spawn worker");
        assert!(
            out.status.success(),
            "worker failed: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    };

    // right oracle -> PASS with the rt identities bound
    let pass = run_certify(ORACLE as i64);
    assert_eq!(json_field(&pass, "verdict"), "PASS");
    assert_eq!(
        json_field(&pass, "recipe"),
        "ac-a2r/hir-record-text-emit-v1"
    );
    assert_eq!(json_field(&pass, "actual_observed"), "value:23");
    assert_eq!(json_field(&pass, "reference_observed"), "value:23");

    // wrong oracle -> REJECTED (both legs observe 23)
    let rejected = run_certify(43);
    assert_eq!(json_field(&rejected, "verdict"), "REJECTED");

    // the emitted certificate carries the observed toolchain binding
    let cert_text =
        std::fs::read_to_string(out_root().join("proto-23").join("case.cert.json")).unwrap();
    assert!(cert_text.contains("ace.a2r-hir.record-text.cert/1"));
    assert!(cert_text.contains("ac-a2r/hir-record-text-emit-v1"));
    assert!(cert_text.contains("version_vv_sha256"));
}
/// Extract one top-level string field from the worker's JSON response
/// (a few named fields only — no JSON model needed here).
fn json_field(raw: &str, name: &str) -> String {
    let needle = format!("\"{}\"", name);
    let pos = raw
        .find(&needle)
        .unwrap_or_else(|| panic!("field {name} in response"));
    let rest = raw[pos + needle.len()..].trim_start();
    let rest = rest.strip_prefix(':').expect("colon").trim_start();
    let end = rest.find([',', '\n', '}']).expect("value end");
    rest[..end].trim().trim_matches('"').to_string()
}
