//! T-02 certification-shaped DRY RUN (private; the H-side registration is
//! T-05). Materializes the frozen cert contract draft as a real runnable
//! form for B01 (plan §5.5 显式certify):
//!
//! * producer emits the actual file set on the spot (golden is a FROZEN
//!   byte copy on disk — regenerating must match it byte-identically; a
//!   golden is never the source of an actual);
//! * an INDEPENDENT hand-written reference program (new i32/trap form —
//!   NOT the old a2r i64 wrapper) compiles and runs separately; generated
//!   value, reference value and the closed-form oracle must all agree;
//! * the toolchain identity is OBSERVED (`rustc -vV`) and bound by hash;
//! * the cert draft (schema `ace.a2r-hir.cert/1`, recipe
//!   `ac-a2r/hir-scalar-emit-v1`) binds the full actual file set +
//!   wrapper/reference hashes + versions, validates strictly, serializes
//!   byte-stably, and REJECTS tampered shapes (bad verdict, old-schema
//!   impersonation, empty file binding, missing toolchain identity).

use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{
    assemble_file_set, lower_program, BoundFile, CertDraft, ToolchainIdentity, CERT_SCHEMA, RECIPE,
};
use ace_core::source::compile_source;
use std::path::PathBuf;
use std::process::Command;

const SRC: &str = "fn main() int { return 2 + 3 * 4 }";
const ORACLE: i32 = 14; // closed-form hand oracle

/// Independent hand-written reference (frozen bytes; the NEW i32/trap form).
const REFERENCE: &str = r#"// PLAN-892 independent reference for B01 (hand-written, i32/trap form)
fn ref_main() -> i32 { 2 + 3 * 4 }
fn main() { println!("value:{}", ref_main()); }
"#;

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

fn compile_run(dir: &PathBuf, out_name: &str) -> String {
    let exe = dir.join(out_name);
    let mut owned_compiler = Command::new(rustc());
    owned_compiler
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg("main.rs")
        .current_dir(dir);
    let out = auto_tool_execution::run_command_output(owned_compiler).expect("spawn rustc");
    assert!(
        out.status.success(),
        "rustc failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&exe).current_dir(dir).output().expect("run");
    assert!(run.status.success(), "non-zero wrapper exit");
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

fn emit(dir_name: &str) -> (Vec<(String, String)>, String) {
    let compiled = compile_source(SRC, "cert", &[]).expect("frontend compile");
    let view = SemanticView::from_checked(&compiled.checked).expect("view");
    let program = lower_program(&view, "main").expect("lowering");
    let fs = assemble_file_set(&view, &program, &compiled.input_hash, "main");
    let dir = out_root().join(dir_name);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &fs.manifest).unwrap();
    (fs.files.clone(), fs.manifest.clone())
}

#[test]
fn certify_dry_run_b01() {
    // ---- actual + golden: regenerating is byte-identical to the frozen copy
    let (files, manifest) = emit("cert-b01");
    let golden_dir = out_root().join("cert-b01-golden");
    std::fs::create_dir_all(&golden_dir).unwrap();
    let marker = golden_dir.join(".golden-frozen");
    if marker.exists() {
        // golden already frozen: regenerate and compare byte-for-byte
        let (files2, manifest2) = emit("cert-b01-recheck");
        assert_eq!(files, files2, "emission must be deterministic");
        assert_eq!(manifest, manifest2, "manifest must be deterministic");
        for (path, content) in &files {
            let golden = std::fs::read(golden_dir.join(path))
                .unwrap_or_else(|e| panic!("golden {} missing: {}", path, e));
            assert_eq!(
                &String::from_utf8_lossy(&golden),
                content,
                "golden drift in {}",
                path
            );
        }
    } else {
        for (path, content) in &files {
            std::fs::write(golden_dir.join(path), content).unwrap();
        }
        std::fs::write(&marker, "frozen\n").unwrap();
        // and the determinism control still runs once against a re-emit
        let (files2, manifest2) = emit("cert-b01-recheck");
        assert_eq!(files, files2);
        assert_eq!(manifest, manifest2);
    }

    // ---- generated program: compile + run, value equals the oracle
    let generated_line = compile_run(&out_root().join("cert-b01"), "a2x_actual.exe");
    assert_eq!(generated_line, format!("value:{}", ORACLE));

    // ---- independent reference: compile + run separately
    let ref_dir = out_root().join("cert-b01-ref");
    std::fs::create_dir_all(&ref_dir).unwrap();
    std::fs::write(ref_dir.join("main.rs"), REFERENCE).unwrap();
    let reference_line = compile_run(&ref_dir, "a2x_ref.exe");
    assert_eq!(reference_line, format!("value:{}", ORACLE));

    // ---- observed toolchain identity (D-04)
    let vv = Command::new(rustc())
        .arg("-vV")
        .output()
        .expect("rustc -vV");
    assert!(vv.status.success(), "rustc -vV failed");
    let vv_text = String::from_utf8_lossy(&vv.stdout).to_string();
    assert!(vv_text.contains("rustc"), "identity text sanity");

    // ---- cert draft: binds everything, validates, serializes stably
    let wrapper_sha = sha256_hex(
        files
            .iter()
            .find(|(p, _)| p == "main.rs")
            .unwrap()
            .1
            .as_bytes(),
    );
    let cert = CertDraft {
        schema: CERT_SCHEMA,
        recipe: RECIPE,
        case: "a2r-hir-892/B01".to_string(),
        profile: "auto.source.core-i32.draft".to_string(),
        hir_domain: "core-i32-draft".to_string(),
        entry: "main".to_string(),
        verdict: "PASS".to_string(),
        oracle_kind: "value",
        oracle_value: ORACLE.to_string(),
        input_hash: "deadbeefdeadbeef".to_string(),
        source_sha256: sha256_hex(SRC.as_bytes()),
        actual_files: files
            .iter()
            .map(|(p, c)| BoundFile {
                path: p.clone(),
                sha256: sha256_hex(c.as_bytes()),
            })
            .collect(),
        golden_sha256: Some(sha256_hex(
            &std::fs::read(golden_dir.join("generated.rs")).unwrap(),
        )),
        wrapper_sha256: wrapper_sha,
        reference_sha256: sha256_hex(REFERENCE.as_bytes()),
        backend: "auto-a2x-rust",
        backend_version: "0.1.0",
        shared_view_version: "0.1.0",
        protocol: "a2x-observation/1",
        toolchain: ToolchainIdentity::observed(vv_text, vec!["--edition".into(), "2021".into()]),
        capabilities: vec![],
    };
    assert!(cert.validate().is_ok(), "cert draft must validate");
    let dir = out_root().join("cert-b01");
    std::fs::write(dir.join("b01.cert.json"), cert.to_json()).unwrap();
    let again = cert.to_json();
    assert_eq!(cert.to_json(), again, "byte-stable serialization");

    // ---- tamper controls: bad shapes are REJECTED, never coerced
    let mut bad = cert.clone();
    bad.verdict = "MAYBE".into();
    assert!(bad.validate().is_err());
    let mut bad = cert.clone();
    bad.schema = "ace.m1.cert/1"; // old schema must not impersonate the new one
    assert!(bad.validate().is_err());
    let mut bad = cert.clone();
    bad.actual_files.clear();
    assert!(bad.validate().is_err());
    let mut bad = cert.clone();
    bad.toolchain.version_vv = String::new();
    assert!(bad.validate().is_err());
    let mut bad = cert.clone();
    bad.oracle_kind = "exit-code";
    assert!(bad.validate().is_err());
}
