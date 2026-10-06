//! PLAN-745 T-04 source CLI tests: `check-source` / `build-source` exit
//! codes (0/1/2), the mandatory explicit `--source-profile`, entry-by-name
//! errors, capability gating, receipts carrying source metadata, and the
//! no-artifacts-on-failure contract. Windows only (native gate).

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const PROFILE: &str = "auto.source.core-i32.draft";

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

fn src_file(name: &str) -> String {
    Path::new(REPO)
        .join("experimental/ac-core/fixtures/source")
        .join(name)
        .display()
        .to_string()
}

fn valid(name: &str) -> String {
    src_file(&format!("valid/{name}"))
}

fn invalid(name: &str) -> String {
    src_file(&format!("invalid/{name}"))
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac745-src-cli").join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_exe(exe: &Path) -> i32 {
    Command::new(exe)
        .output()
        .expect("run built exe")
        .status
        .code()
        .unwrap_or(-1)
}

#[test]
fn check_source_valid_exits_zero() {
    let r = run_ac(&[
        "check-source",
        &valid("p01-precedence.at"),
        "--source-profile",
        PROFILE,
    ]);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    assert!(r.stdout.contains("OK source"), "stdout: {}", r.stdout);
    assert!(r.stdout.contains("profile=auto.source.core-i32.draft"));
    assert!(r.stdout.contains("input-hash="));
}

#[test]
fn check_source_requires_explicit_profile() {
    // Missing --source-profile is a usage error (2), never a mode guess.
    let r = run_ac(&["check-source", &valid("p01-precedence.at")]);
    assert_eq!(r.code, 2, "stderr: {}", r.stderr);
    assert!(r.stderr.contains("--source-profile"), "{}", r.stderr);

    // Unknown profile value is also a usage error.
    let r = run_ac(&[
        "check-source",
        &valid("p01-precedence.at"),
        "--source-profile",
        "auto.source.core-i64.draft",
    ]);
    assert_eq!(r.code, 2, "stderr: {}", r.stderr);
    assert!(r.stderr.contains("unknown source profile"), "{}", r.stderr);
}

#[test]
fn check_source_pipeline_rejection_exits_one() {
    let r = run_ac(&[
        "check-source",
        &invalid("n01-literal-range-max.at"),
        "--source-profile",
        PROFILE,
    ]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert!(r.stderr.contains("source.literal-range"), "{}", r.stderr);
    assert!(r.stderr.contains("2147483648"), "{}", r.stderr);

    // Later-stage rejections render against the original source file.
    let r = run_ac(&[
        "check-source",
        &invalid("n03-undefined-name.at"),
        "--source-profile",
        PROFILE,
    ]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert!(
        r.stderr.contains("error[bind/source.undefined]"),
        "{}",
        r.stderr
    );
    assert!(r.stderr.contains(":2:24:"), "{}", r.stderr);
}

#[test]
fn check_source_capability_gates_marks() {
    let p15 = valid("p15-named-eval-order-x2.at");
    // Without the capability, mark_a is just an undefined name.
    let r = run_ac(&["check-source", &p15, "--source-profile", PROFILE]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert!(r.stderr.contains("source.undefined"), "{}", r.stderr);

    // With it, the module compiles and requires the capability.
    let r = run_ac(&[
        "check-source",
        &p15,
        "--source-profile",
        PROFILE,
        "--capability",
        "hir.test.trace.v1",
    ]);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    assert!(
        r.stdout.contains("hir.test.trace.v1"),
        "stdout: {}",
        r.stdout
    );
}

#[test]
fn build_source_produces_running_exe_with_receipt() {
    let dir = temp_dir("build-p01");
    let exe = dir.join("p01.exe");
    let r = run_ac(&[
        "build-source",
        &valid("p01-precedence.at"),
        "--source-profile",
        PROFILE,
        "--entry",
        "main",
        "--output",
        &exe.display().to_string(),
    ]);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    assert!(exe.exists(), "exe missing");
    assert!(dir.join("p01.obj").exists(), "obj missing");
    let receipt = std::fs::read_to_string(dir.join("p01.ac-link.txt")).unwrap();
    assert!(receipt.contains("source-profile: auto.source.core-i32.draft"));
    assert!(receipt.contains("source-input-hash: "));
    assert_eq!(run_exe(&exe), 14); // 2 + 3 * 4
}

#[test]
fn build_source_trap_exit_code() {
    let dir = temp_dir("build-p13");
    let exe = dir.join("p13.exe");
    let r = run_ac(&[
        "build-source",
        &valid("p13-add-overflow-trap.at"),
        "--source-profile",
        PROFILE,
        "--entry",
        "main",
        "--output",
        &exe.display().to_string(),
    ]);
    assert_eq!(r.code, 0, "build must not compile-time-reject overflow");
    assert_eq!(run_exe(&exe), 70);
}

#[test]
fn build_source_rejections_leave_no_artifacts() {
    let dir = temp_dir("build-n01");
    let exe = dir.join("n01.exe");
    let r = run_ac(&[
        "build-source",
        &invalid("n01-literal-range-max.at"),
        "--source-profile",
        PROFILE,
        "--entry",
        "main",
        "--output",
        &exe.display().to_string(),
    ]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert!(!exe.exists(), "no exe on failed source");
    assert!(
        std::fs::read_dir(&dir).unwrap().count() == 0,
        "no artifacts at all on failure"
    );

    // Usage error: missing --entry.
    let r = run_ac(&[
        "build-source",
        &valid("p01-precedence.at"),
        "--source-profile",
        PROFILE,
        "--output",
        &exe.display().to_string(),
    ]);
    assert_eq!(r.code, 2, "stderr: {}", r.stderr);
}

#[test]
fn build_source_entry_errors() {
    let dir = temp_dir("build-entry");
    let p02 = valid("p02-signed-positional.at");

    // Unknown name.
    let r = run_ac(&[
        "build-source",
        &p02,
        "--source-profile",
        PROFILE,
        "--entry",
        "absent",
        "--output",
        &dir.join("a.exe").display().to_string(),
    ]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert!(r.stderr.contains("entry.not-found"), "{}", r.stderr);
    assert!(r.stderr.contains("absent"), "{}", r.stderr);

    // A parameterized function cannot be the entry.
    let r = run_ac(&[
        "build-source",
        &p02,
        "--source-profile",
        PROFILE,
        "--entry",
        "add",
        "--output",
        &dir.join("b.exe").display().to_string(),
    ]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert!(r.stderr.contains("entry.signature"), "{}", r.stderr);
}

#[test]
fn build_source_trace_requires_support_lib_at_link_time() {
    // With the capability declared but no support library, the intrinsic
    // import cannot resolve: the LINK fails after compilation, exit 1.
    let dir = temp_dir("build-p15-nolib");
    let r = run_ac(&[
        "build-source",
        &valid("p15-named-eval-order-x2.at"),
        "--source-profile",
        PROFILE,
        "--entry",
        "main",
        "--capability",
        "hir.test.trace.v1",
        "--output",
        &dir.join("p15.exe").display().to_string(),
    ]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert!(r.stderr.contains("link"), "{}", r.stderr);
    assert!(!dir.join("p15.exe").exists());
}

#[test]
fn old_check_still_atom_only() {
    // The HIR-mode `check` must not silently interpret a source file: the
    // Atom binder rejects source syntax (extension never selects a mode).
    let r = run_ac(&["check", &valid("p01-precedence.at")]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert!(
        r.stderr.contains("text/") || r.stderr.contains("bind/"),
        "expected text/bind diagnostics, got: {}",
        r.stderr
    );
}
