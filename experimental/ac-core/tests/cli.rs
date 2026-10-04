//! PLAN-741 T-05 CLI tests: `ac-probe check` and `ac-probe build` exit
//! codes, artifact placement, receipts, entry/capability/linker error paths
//! and no-overwrite-on-failure. Windows only (native gate).

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn repo_file(rel: &str) -> PathBuf {
    Path::new(REPO).join(rel)
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_ac(args: &[&str]) -> Run {
    let out = Command::new(EXE)
        .args(args)
        .output()
        .expect("spawn ac-probe");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

#[test]
fn check_valid_document_succeeds() {
    let f = repo_file("docs/design/strategy/hir-examples/01-add.atom");
    let r = run_ac(&["check", &f.display().to_string()]);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    assert!(r.stdout.contains("OK 1 module"), "stdout: {}", r.stdout);
}

#[test]
fn check_invalid_document_fails_with_code() {
    let f = repo_file("experimental/ac-core/fixtures/invalid/duplicate-field.atom");
    let r = run_ac(&["check", &f.display().to_string()]);
    assert_ne!(r.code, 0);
    assert!(
        r.stderr.contains("bind.duplicate-field"),
        "stderr: {}",
        r.stderr
    );
    // diagnostics carry location
    assert!(r.stderr.contains(".atom:"), "stderr: {}", r.stderr);
}

#[test]
fn check_verify_stage_failure_reports_verify_code() {
    let f = repo_file("experimental/ac-core/fixtures/invalid/delete-let-before-return.atom");
    let r = run_ac(&["check", &f.display().to_string()]);
    assert_ne!(r.code, 0);
    assert!(
        r.stderr.contains("verify.uninitialized-read"),
        "stderr: {}",
        r.stderr
    );
}

#[test]
fn build_add_fixture_produces_running_exe_with_receipt() {
    let dir = std::env::temp_dir().join("ac741-cli").join("build-add");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("add.exe");
    let f = repo_file("experimental/ac-core/fixtures/native/add-2-3.atom");
    let r = run_ac(&[
        "build",
        &f.display().to_string(),
        "--entry",
        "d_entry",
        "--output",
        &exe.display().to_string(),
    ]);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    assert!(exe.is_file());
    assert!(dir.join("add.obj").is_file());
    assert!(dir.join("add.ac-link.txt").is_file());
    let receipt = std::fs::read_to_string(dir.join("add.ac-link.txt")).unwrap();
    assert!(receipt.contains("PLAN-741 link receipt"), "{}", receipt);
    assert!(receipt.contains("rust-lld"), "{}", receipt);
    // run it: add(2,3) == 5
    let out = Command::new(&exe).output().unwrap();
    assert_eq!(out.status.code(), Some(5));
}

#[test]
fn build_with_missing_entry_fails_without_touching_artifacts() {
    let dir = std::env::temp_dir().join("ac741-cli").join("no-overwrite");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("keep.exe");
    std::fs::write(&exe, b"SENTINEL").unwrap();

    let f = repo_file("experimental/ac-core/fixtures/native/add-2-3.atom");
    let r = run_ac(&[
        "build",
        &f.display().to_string(),
        "--entry",
        "d_does_not_exist",
        "--output",
        &exe.display().to_string(),
    ]);
    assert_ne!(r.code, 0);
    assert!(r.stderr.contains("entry.not-found"), "stderr: {}", r.stderr);
    assert_eq!(std::fs::read(&exe).unwrap(), b"SENTINEL");
    // no stray temp artifacts
    assert!(!dir.join("keep.obj").exists());
    assert!(!dir.join("keep.exe.tmp-ac741").exists());
}

#[test]
fn build_entry_with_wrong_signature_is_rejected() {
    let dir = std::env::temp_dir().join("ac741-cli").join("bad-signature");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("sig.exe");
    let f = repo_file("experimental/ac-core/fixtures/native/add-2-3.atom");
    let r = run_ac(&[
        "build",
        &f.display().to_string(),
        "--entry",
        "d_add", // two params — not a valid entry
        "--output",
        &exe.display().to_string(),
    ]);
    assert_ne!(r.code, 0);
    assert!(r.stderr.contains("entry.signature"), "stderr: {}", r.stderr);
    assert!(!exe.exists());
}

#[test]
fn build_without_capability_is_rejected() {
    let dir = std::env::temp_dir().join("ac741-cli").join("capability");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("trace.exe");
    let f = repo_file("docs/design/strategy/hir-examples/03-call-order.atom");
    let r = run_ac(&[
        "build",
        &f.display().to_string(),
        "--entry",
        "d_caller",
        "--output",
        &exe.display().to_string(),
    ]);
    assert_ne!(r.code, 0);
    assert!(
        r.stderr.contains("capability.missing"),
        "stderr: {}",
        r.stderr
    );
    assert!(
        r.stderr.contains("hir.test.trace.v1"),
        "stderr: {}",
        r.stderr
    );
    assert!(!exe.exists());
}

#[test]
fn build_with_capability_fails_at_link_until_support_lib_exists() {
    // The trace capability is declared, so lowering proceeds; the link then
    // fails on the intrinsic symbols because no host support library is in
    // this test's link inputs. The failure must be clean and leave no exe.
    let dir = std::env::temp_dir().join("ac741-cli").join("trace-link");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("trace.exe");
    let f = repo_file("docs/design/strategy/hir-examples/03-call-order.atom");
    let r = run_ac(&[
        "build",
        &f.display().to_string(),
        "--entry",
        "d_caller",
        "--output",
        &exe.display().to_string(),
        "--capability",
        "hir.test.trace.v1",
    ]);
    assert_ne!(r.code, 0);
    assert!(r.stderr.contains("link.failed"), "stderr: {}", r.stderr);
    assert!(!exe.exists());
    assert!(!dir.join("trace.obj").exists());
}

#[test]
fn linker_override_failure_is_reported() {
    let dir = std::env::temp_dir().join("ac741-cli").join("bad-lld");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("x.exe");
    let f = repo_file("experimental/ac-core/fixtures/native/add-2-3.atom");
    let r = Command::new(EXE)
        .args([
            "build",
            &f.display().to_string(),
            "--entry",
            "d_entry",
            "--output",
            &exe.display().to_string(),
        ])
        .env("AC741_RUST_LLD", r"C:\definitely\missing\rust-lld.exe")
        .output()
        .unwrap();
    assert_ne!(r.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&r.stderr);
    assert!(stderr.contains("link.lld-not-found"), "stderr: {}", stderr);
    assert!(!exe.exists());
}
