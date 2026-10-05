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

/// R2-QA-01: malformed block graphs must be rejected by `check` with a
/// located diagnostic — no stack overflow, no panic, a bounded wait (AC-16
/// demands a return within 5s), and `build` must refuse before any artifact
/// is created.
#[test]
fn block_graph_violations_rejected_within_deadline() {
    for (name, marker) in [
        ("block-self-cycle.atom", "referenced as a child block"),
        (
            "block-disconnected-cycle.atom",
            "not reachable from the entry block",
        ),
    ] {
        let started = std::time::Instant::now();
        let f = repo_file(&format!("experimental/ac-core/fixtures/invalid/{name}"));

        let r = run_ac(&["check", &f.display().to_string()]);
        assert_eq!(r.code, 1, "{name}: stdout: {}", r.stdout);
        assert!(
            r.stderr.contains("verify.block-structure"),
            "{name}: {}",
            r.stderr
        );
        assert!(r.stderr.contains(marker), "{name}: {}", r.stderr);
        assert!(r.stderr.contains(".atom:"), "{name}: no span: {}", r.stderr);

        let dir = std::env::temp_dir()
            .join("ac741-cli")
            .join(format!("blockgraph-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("out.exe");
        let r = run_ac(&[
            "build",
            &f.display().to_string(),
            "--entry",
            "de",
            "--output",
            &exe.display().to_string(),
        ]);
        assert_eq!(r.code, 1, "{name} build: stdout: {}", r.stdout);
        assert!(!exe.exists(), "{name}: exe artifact produced");
        assert!(
            !dir.join("out.obj").exists(),
            "{name}: obj artifact produced"
        );

        let elapsed = started.elapsed();
        assert!(
            elapsed < std::time::Duration::from_secs(5),
            "{name}: took {elapsed:?} (must return within 5s, no runaway recursion)"
        );
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
    // no stray temp artifacts (names carry a per-process tag)
    assert!(!dir.join("keep.obj").exists());
    assert!(!dir.read_dir().unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains(".tmp-ac741")));
}

/// QA-04: a failure at the *publish* stage (receipt placement) must roll the
/// whole transaction back — the previously successful exe/obj/receipt set
/// stays byte-identical and runnable.
#[test]
fn publish_failure_rolls_back_and_keeps_previous_artifacts() {
    let dir = std::env::temp_dir().join("ac741-cli").join("tx-rollback");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("tx.exe");
    let obj = dir.join("tx.obj");
    let receipt = dir.join("tx.ac-link.txt");

    // 1. Baseline: successful build of add(2,3) → native exit 5.
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
    let before_exe = std::fs::read(&exe).unwrap();
    let before_obj = std::fs::read(&obj).unwrap();
    assert!(receipt.is_file(), "baseline receipt must exist");
    let out = Command::new(&exe).output().unwrap();
    assert_eq!(out.status.code(), Some(5));

    // 2. Occupy the receipt path with a directory: staging succeeds, but the
    //    final receipt rename cannot (and the directory is not "backed up").
    std::fs::remove_file(&receipt).unwrap();
    std::fs::create_dir(&receipt).unwrap();

    // 3. Rebuild a *different* program onto the same target.
    let f2 = repo_file("experimental/ac-core/fixtures/native/add-neg2-3.atom");
    let r = run_ac(&[
        "build",
        &f2.display().to_string(),
        "--entry",
        "d_entry",
        "--output",
        &exe.display().to_string(),
    ]);
    assert_ne!(r.code, 0, "stdout: {}", r.stdout);
    assert!(r.stderr.contains("link.receipt"), "stderr: {}", r.stderr);

    // 4. Previous artifacts are untouched and the old exe still runs.
    assert_eq!(std::fs::read(&exe).unwrap(), before_exe, "exe replaced");
    assert_eq!(std::fs::read(&obj).unwrap(), before_obj, "obj replaced");
    let out = Command::new(&exe).output().unwrap();
    assert_eq!(out.status.code(), Some(5));

    // 5. Rollback left no staging/backup files behind.
    let strays: Vec<String> = dir
        .read_dir()
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.contains(".tmp-ac741") || n.contains(".bak-ac741"))
        .collect();
    assert!(strays.is_empty(), "stray staging files: {strays:?}");
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
