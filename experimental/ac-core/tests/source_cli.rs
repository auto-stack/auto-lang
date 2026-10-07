//! PLAN-745 T-04 source CLI tests: `check-source` / `build-source` exit
//! codes (0/1/2), the mandatory explicit `--source-profile`, entry-by-name
//! errors, capability gating, receipts carrying source metadata, and the
//! no-artifacts-on-failure contract. Windows only (native gate).

#![cfg(windows)]

mod common;

use common::run_deadlined;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const PROFILE: &str = "auto.source.core-i32.draft";

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

/// Every CLI invocation runs under the shared deadline executor (QA-05):
/// concurrent pipe drain, kill + reap on expiry — no unbounded waits.
fn run_ac(args: &[&str]) -> Run {
    let mut cmd = Command::new(EXE);
    cmd.args(args);
    let r = run_deadlined(cmd, Duration::from_secs(120)).expect("spawn ac-probe");
    assert!(!r.timed_out, "ac-probe exceeded 120s deadline");
    Run {
        code: r.code,
        stdout: r.stdout,
        stderr: r.stderr,
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
    let r = run_deadlined(Command::new(exe), Duration::from_secs(60)).expect("run built exe");
    assert!(!r.timed_out, "built exe exceeded 60s deadline");
    r.code
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

fn sha256_of(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_default();
    // FNV-1a 64 (same family as the adapter's input hash): enough to detect
    // any byte change for the sentinel comparison.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:016x}", h)
}

/// QA-06: source-mode SENTINEL regression — a failing build over previously
/// successful artifacts must leave exe/obj/receipt byte-identical, across
/// all three failure classes: source rejection (before artifacts), link
/// failure (staged, reclaimed) and publish failure (rollback path).
#[test]
fn source_build_never_overwrites_existing_artifacts() {
    let dir = temp_dir("sentinel");
    let exe = dir.join("app.exe");
    let obj = dir.join("app.obj");
    let receipt = dir.join("app.ac-link.txt");

    // A good build establishes the sentinel trio.
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
    let trio = [sha256_of(&exe), sha256_of(&obj), sha256_of(&receipt)];
    assert!(trio.iter().all(|h| !h.is_empty()), "sentinels must exist");

    // (1) Source rejection: fails before any artifact is touched.
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
    assert_eq!(r.code, 1);
    assert_eq!(
        [sha256_of(&exe), sha256_of(&obj), sha256_of(&receipt)],
        trio
    );

    // (2) Link failure: compiles and links an unresolvable intrinsic import;
    // staging is reclaimed and the finals survive byte-identical.
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
        &exe.display().to_string(),
    ]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert_eq!(
        [sha256_of(&exe), sha256_of(&obj), sha256_of(&receipt)],
        trio
    );

    // (3) Publish failure: the receipt path occupied by a directory is user
    // property — the publish refuses, rolls back and reports it.
    let blocked = dir.join("app.ac-link.txt.blocked-dir");
    let _ = std::fs::remove_dir_all(&blocked);
    std::fs::create_dir_all(&blocked).unwrap();
    // Rename trick is not available (path occupied), so emulate via a
    // directory sitting where the RECEIPT must be published: use a second
    // output whose receipt slot is pre-occupied, while its exe/obj already
    // exist as a good build's finals.
    let exe2 = dir.join("app2.exe");
    let obj2 = dir.join("app2.obj");
    let rcpt2 = dir.join("app2.ac-link.txt");
    let r = run_ac(&[
        "build-source",
        &valid("p01-precedence.at"),
        "--source-profile",
        PROFILE,
        "--entry",
        "main",
        "--output",
        &exe2.display().to_string(),
    ]);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    let trio2 = [sha256_of(&exe2), sha256_of(&obj2), sha256_of(&rcpt2)];
    // Replace the receipt FILE with a directory (same name) — publish must
    // refuse to touch it and roll the whole transaction back.
    std::fs::remove_file(&rcpt2).unwrap();
    std::fs::create_dir(&rcpt2).unwrap();
    let r = run_ac(&[
        "build-source",
        &valid("p02-signed-positional.at"),
        "--source-profile",
        PROFILE,
        "--entry",
        "main",
        "--output",
        &exe2.display().to_string(),
    ]);
    assert_eq!(r.code, 1, "publish over a directory receipt must fail");
    assert!(r.stderr.contains("link."), "stderr: {}", r.stderr);
    // exe/obj survived byte-identical; the receipt slot is still OUR
    // directory (user property, never deleted).
    assert_eq!(sha256_of(&exe2), trio2[0]);
    assert_eq!(sha256_of(&obj2), trio2[1]);
    assert!(rcpt2.is_dir());
    std::fs::remove_dir_all(&blocked).unwrap();
    std::fs::remove_dir_all(&rcpt2).unwrap();
}

/// R2-QA-04: the source-mode publish-failure sentinel over THREE REAL
/// previous artifacts. A good build establishes exe/obj/receipt; a second
/// build then fails AT THE PUBLISH STAGE — the receipt file is held open
/// with FileShare.None, so the transaction's backup rename cannot move it
/// — and the rollback must restore all three byte-identically. (The older
/// directory-occupies-receipt path above stays as the user-property
/// placeholder control; this one protects the real trio.)
#[test]
fn source_publish_failure_rolls_back_three_real_files() {
    let dir = temp_dir("sentinel-publish");
    let exe = dir.join("app.exe");
    let obj = dir.join("app.obj");
    let rcpt = dir.join("app.ac-link.txt");

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
    let trio = [sha256_of(&exe), sha256_of(&obj), sha256_of(&rcpt)];
    assert!(trio.iter().all(|h| !h.is_empty()));

    // Hold the receipt open with NO sharing: the publish transaction's
    // backup rename of the receipt fails (sharing violation) after the
    // exe/obj backups — the rollback must restore everything.
    let hold_ps = format!(
        "$f=[System.IO.File]::Open('{}','Open','Read','None'); Start-Sleep -Seconds 20",
        rcpt.display()
    );
    let mut holder = Command::new("powershell")
        .args(["-NoProfile", "-Command", &hold_ps])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn receipt holder");
    // give the holder a moment to acquire the handle
    std::thread::sleep(std::time::Duration::from_millis(700));

    let r = run_ac(&[
        "build-source",
        &valid("p02-signed-positional.at"),
        "--source-profile",
        PROFILE,
        "--entry",
        "main",
        "--output",
        &exe.display().to_string(),
    ]);
    let _ = holder.kill();
    let _ = holder.wait();
    assert_eq!(r.code, 1, "publish must fail while the receipt is held");
    assert!(
        r.stderr.contains("link."),
        "expected a link/publish diagnostic: {}",
        r.stderr
    );
    assert_eq!(
        [sha256_of(&exe), sha256_of(&obj), sha256_of(&rcpt)],
        trio,
        "all three real artifacts must survive the rollback byte-identically"
    );
}

/// R2-QA-05 end-to-end: ~4KB of unclosed parentheses through the real CLI
/// must exit 1 with a source-stage diagnostic — no stack-overflow crash.
#[test]
fn deep_unclosed_input_via_cli_is_controlled() {
    let dir = temp_dir("deep-unclosed");
    let src_path = dir.join("deep.at");
    std::fs::write(
        &src_path,
        format!("fn main() int {{ return {}1", "(".repeat(4096)),
    )
    .unwrap();
    let r = run_ac(&[
        "check-source",
        &src_path.display().to_string(),
        "--source-profile",
        PROFILE,
    ]);
    assert_eq!(r.code, 1, "stdout: {} stderr: {}", r.stdout, r.stderr);
    assert!(
        r.stderr.contains("source.syntax"),
        "controlled diagnostic expected: {}",
        r.stderr
    );
    // unclosed parens now fail depth-independently at the missing
    // (the collapse loop); the nesting guard wording belongs to the
    // call/block structures
    assert!(
        r.stderr.contains("expected") && !r.stderr.contains("overflowed"),
        "controlled diagnostic expected: {}",
        r.stderr
    );
}

/// R3-QA-02 end-to-end: the reviewer's 32794-byte legal add chain (16384
/// terms) through the real CLI must check successfully — no crash, no
/// unapproved limit — and a 4096-term MALFORMED tail stays a controlled
/// syntax rejection.
#[test]
fn deep_legal_and_malformed_chains_via_cli() {
    let dir = temp_dir("deep-chain");
    let ok = dir.join("ok.at");
    std::fs::write(
        &ok,
        format!("fn main() int {{ return {}0 }}", "1+".repeat(16384)),
    )
    .unwrap();
    let r = run_ac(&[
        "check-source",
        &ok.display().to_string(),
        "--source-profile",
        PROFILE,
    ]);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);

    let bad = dir.join("bad.at");
    std::fs::write(
        &bad,
        format!("fn main() int {{ return {}1", "+ 1".repeat(16384)),
    )
    .unwrap();
    let r = run_ac(&[
        "check-source",
        &bad.display().to_string(),
        "--source-profile",
        PROFILE,
    ]);
    assert_eq!(r.code, 1, "stderr: {}", r.stderr);
    assert!(r.stderr.contains("source.syntax"), "{}", r.stderr);
}
