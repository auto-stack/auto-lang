//! PLAN-767 T-04/T-05 scalar native execution tests. Real PEs only:
//! HIR-domain fixtures go through `build` (bind_document → domain verify →
//! lower → link), source fixtures through `build-source --source-profile
//! auto.source.scalar-i32.draft`. Covers V-02 (signed truncation table),
//! V-03 (trap matrix + unexecuted-trap control), V-04 (short-circuit
//! polarity incl. trap RHS), V-05/V-11/V-12 (source corpus incl. conditional
//! for and all declaration contexts), V-09's small mixed PE, V-01/V-12
//! CLI-level profile isolation, and V-10's receipt identity + transaction
//! preservation. Every subprocess runs under the shared contained executor.

#![cfg(windows)]

mod common;

use common::run_deadlined;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const SCALAR: &str = "auto.source.scalar-i32.draft";
const CORE: &str = "auto.source.core-i32.draft";
const TRACE: &str = "hir.test.trace.v1";

fn repo_path(rel: &str) -> PathBuf {
    Path::new(REPO).join(rel)
}

fn scalar_src(name: &str) -> String {
    repo_path("experimental/ac-core/fixtures/scalar-source/valid")
        .join(name)
        .display()
        .to_string()
}

fn scalar_hir(name: &str) -> String {
    repo_path("experimental/ac-core/fixtures/scalar-hir/valid")
        .join(name)
        .display()
        .to_string()
}

fn build_support_lib() -> PathBuf {
    // Standalone no_std crate; separate target dir to avoid cargo lock
    // contention with the outer test run (same policy as source_native).
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--locked"])
        .current_dir(repo_path("experimental/ac-core/test-support"));
    let r = run_deadlined(cmd, Duration::from_secs(300)).expect("spawn cargo for test-support");
    assert!(!r.timed_out, "test-support build exceeded 300s deadline");
    assert_eq!(r.code, 0, "test-support build failed:\n{}", r.stderr);
    let lib = repo_path("experimental/ac-core/test-support/target/debug/ac_trace_support.lib");
    assert!(lib.is_file(), "support lib missing at {}", lib.display());
    lib
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac767-scalar-native").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run the CLI binary under the contained executor; 600s build gate per
/// plan §6.1.
fn cli(args: &[&str]) -> common::DeadlinedRun {
    let mut c = Command::new(EXE);
    c.args(args);
    run_deadlined(c, Duration::from_secs(600)).expect("spawn CLI")
}

fn run_pe(exe: &Path) -> common::DeadlinedRun {
    run_deadlined(Command::new(exe), Duration::from_secs(60)).expect("spawn PE")
}

fn sha256(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    // FNV is enough for change detection here (not a security claim)
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:016x}", h)
}

// -- V-02: signed truncation table (HIR PE, self-asserting exit 0) ---------

#[test]
fn hir_pe_divrem_table_exit0() {
    let dir = temp_dir("divrem");
    let exe = dir.join("n01.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &scalar_hir("n01-divrem-table.atom"),
        "--entry",
        "d_main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert!(!b.timed_out);
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0, "all six rows must hold");
    assert!(run.stderr.is_empty(), "unexpected stderr: {}", run.stderr);
}

// -- V-03: trap matrix (exit 70, pre-trap stderr exactly "a") --------------

#[test]
fn hir_pe_trap_matrix_exit70_stderr_a() {
    let lib = build_support_lib();
    let entries = [
        "d_t_maxadd",
        "d_t_minsub",
        "d_t_minneg",
        "d_t_divm1",
        "d_t_remm1",
        "d_t_div0",
        "d_t_rem0",
    ];
    for entry in entries {
        let dir = temp_dir(entry);
        let exe = dir.join("t.exe");
        let mut c = Command::new(EXE);
        c.args([
            "build",
            &scalar_hir("n02-trap-matrix.atom"),
            "--entry",
            entry,
            "--capability",
            TRACE,
            "--support-lib",
        ])
        .arg(&lib)
        .arg("--output")
        .arg(&exe);
        let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
        assert!(!b.timed_out, "{}: build deadline", entry);
        assert_eq!(b.code, 0, "{}: build failed\n{}", entry, b.stderr);
        let run = run_pe(&exe);
        assert!(!run.timed_out, "{}: run deadline", entry);
        assert_eq!(run.code, 70, "{}: expected trap exit 70", entry);
        assert_eq!(run.stderr, "a", "{}: pre-trap stderr oracle", entry);
    }
}

#[test]
fn hir_pe_unexecuted_trap_exit3() {
    let dir = temp_dir("unex");
    let exe = dir.join("n04.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &scalar_hir("n04-unexecuted-trap.atom"),
        "--entry",
        "d_main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 3, "the false arm must not trap");
    assert!(run.stderr.is_empty());
}

// -- V-04: short-circuit polarity incl. trapping RHS -----------------------

#[test]
fn hir_pe_shortcircuit_polarity() {
    let lib = build_support_lib();
    // (entry, expected exit, expected stderr) — the exit code IS the value
    // of the logical expression (then=0 / else=1), so skipped-RHS rows show
    // the polarity's result while executed rows show exactly one "b".
    let rows: &[(&str, i32, &str)] = &[
        ("d_sc1", 1, ""),  // false && rhs — skipped, result false
        ("d_sc2", 0, ""),  // true || rhs — skipped, result true
        ("d_sc3", 0, "b"), // true && rhs — rhs exactly once, result true
        ("d_sc4", 0, "b"), // false || rhs — rhs exactly once, result true
        ("d_sc5", 1, ""),  // false && (1/0 > 0) — div skipped, no trap
        ("d_sc6", 70, ""), // true && (1/0 > 0) — div executed, trap
    ];
    for (entry, want_exit, want_err) in rows {
        let dir = temp_dir(entry);
        let exe = dir.join("sc.exe");
        let mut c = Command::new(EXE);
        c.args([
            "build",
            &scalar_hir("n03-shortcircuit.atom"),
            "--entry",
            entry,
            "--capability",
            TRACE,
            "--support-lib",
        ])
        .arg(&lib)
        .arg("--output")
        .arg(&exe);
        let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
        assert_eq!(b.code, 0, "{}: build failed\n{}", entry, b.stderr);
        let run = run_pe(&exe);
        assert!(!run.timed_out, "{}: run deadline", entry);
        assert_eq!(run.code, *want_exit, "{}: exit", entry);
        assert_eq!(run.stderr, *want_err, "{}: stderr oracle", entry);
    }
}

// -- V-05/V-11/V-12: source corpus through real PEs -------------------------

#[test]
fn source_pe_scalar_corpus() {
    let support = build_support_lib();
    // (fixture, exit, stderr, needs-trace+lib)
    let rows: &[(&str, i32, &str, bool)] = &[
        ("p-gcd-euclid.at", 0, "", false),
        ("p-count-digits.at", 0, "", false),
        ("p-is-digit.at", 0, "", false),
        ("p-cmp-matrix.at", 0, "", false),
        ("p-a-minus-1.at", 0, "", false),
        ("p-for-loops.at", 0, "", false),
        ("p-binop-args.at", 0, "", false),
        ("p-crlf.at", 0, "", false),
        ("p-short-circuit-trap.at", 70, "", false),
        ("p-short-circuit-trace.at", 0, "aaaba", true),
        // contexts: let/assign/if each "ab"; the for condition is
        // `lhs_a() && n < 2` — its rhs carries no mark, so three condition
        // evaluations emit "aaa"; positional and named call args each "ab"
        ("p-short-circuit-contexts.at", 0, "abababaaaabab", true),
    ];
    for (name, want_exit, want_err, trace) in rows {
        let dir = temp_dir(name);
        let exe = dir.join("app.exe");
        let mut c = Command::new(EXE);
        c.args([
            "build-source",
            &scalar_src(name),
            "--source-profile",
            SCALAR,
            "--entry",
            "main",
            "--output",
        ])
        .arg(&exe);
        if *trace {
            c.args(["--capability", TRACE, "--support-lib"])
                .arg(&support);
        }
        let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
        assert!(!b.timed_out, "{}: build deadline", name);
        assert_eq!(b.code, 0, "{}: build failed\n{}", name, b.stderr);
        let run = run_pe(&exe);
        assert!(!run.timed_out, "{}: run deadline", name);
        assert_eq!(run.code, *want_exit, "{}: exit", name);
        assert_eq!(run.stderr, *want_err, "{}: stderr oracle", name);
        // the publish transaction left all three artifacts
        assert!(dir.join("app.obj").is_file(), "{}: obj missing", name);
        assert!(
            dir.join("app.ac-link.txt").is_file(),
            "{}: receipt missing",
            name
        );
    }
}

// -- V-09: small mixed-deep real PE -----------------------------------------

#[test]
fn source_pe_deep_mixed_1000() {
    let dir = temp_dir("deep1000");
    let depth = 1000usize;
    let src = format!(
        "fn id(x bool) bool {{ return x }}\nfn main() int {{\n    let x = {}true && false{}\n    if x {{ return 1 }}\n    return 0\n}}\n",
        "id(".repeat(depth),
        ")".repeat(depth)
    );
    let src_path = dir.join("deep1000.at");
    std::fs::write(&src_path, src).unwrap();
    let exe = dir.join("deep1000.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &src_path.display().to_string(),
        "--source-profile",
        SCALAR,
        "--entry",
        "main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(
        run.code, 0,
        "innermost false must propagate through 1000 calls"
    );
}

// -- V-01/V-12: CLI-level profile isolation ----------------------------------

#[test]
fn cli_profile_isolation() {
    // check: scalar Atom document dispatches by its header and reports the
    // domain; an unknown triple is a controlled bind diagnostic (exit 1)
    let r = cli(&["check", &scalar_hir("n01-divrem-table.atom")]);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    assert!(
        r.stdout.contains("hir-profile scalar-i32-draft"),
        "stdout: {}",
        r.stdout
    );
    let r = cli(&[
        "check",
        &repo_path("experimental/ac-core/fixtures/scalar-hir/invalid/i09-unknown-triple.atom")
            .display()
            .to_string(),
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("bind.descriptor-unknown"),
        "stderr: {}",
        r.stderr
    );

    // the SAME scalar source under the OLD source profile: rejected at the
    // offending operator (pipeline exit 1)
    let r = cli(&[
        "check-source",
        &scalar_src("p-cmp-matrix.at"),
        "--source-profile",
        CORE,
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("source.unsupported") && r.stderr.contains("operator `-`"),
        "stderr: {}",
        r.stderr
    );

    // unknown profile value remains a usage error (exit 2)
    let r = cli(&[
        "check-source",
        &scalar_src("p-cmp-matrix.at"),
        "--source-profile",
        "auto.source.future.draft",
    ]);
    assert_eq!(r.code, 2);

    // while/for isolation both ways (V-12)
    let dir = temp_dir("loops");
    let while_src = dir.join("w.at");
    std::fs::write(
        &while_src,
        "fn main() int {\n    var i = 0\n    while i < 1 { i = i + 1 }\n    return i\n}\n",
    )
    .unwrap();
    let for_src = dir.join("f.at");
    std::fs::write(
        &for_src,
        "fn main() int {\n    var i = 0\n    for i < 1 { i = i + 1 }\n    return i\n}\n",
    )
    .unwrap();
    let w = while_src.display().to_string();
    let f = for_src.display().to_string();

    let r = cli(&["check-source", &w, "--source-profile", CORE]);
    assert_eq!(r.code, 0, "core while accepted: {}", r.stderr);
    let r = cli(&["check-source", &w, "--source-profile", SCALAR]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("source.unsupported") && r.stderr.contains("`while`"),
        "stderr: {}",
        r.stderr
    );
    let r = cli(&["check-source", &f, "--source-profile", SCALAR]);
    assert_eq!(r.code, 0, "scalar for accepted: {}", r.stderr);
    let r = cli(&["check-source", &f, "--source-profile", CORE]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("source.unsupported") && r.stderr.contains("`for`"),
        "stderr: {}",
        r.stderr
    );
}

// -- V-10: receipt identity + transaction preservation -----------------------

#[test]
fn receipt_identity_and_transaction_preservation() {
    let dir = temp_dir("txn");
    let exe = dir.join("app.exe");
    let obj = dir.join("app.obj");
    let receipt = dir.join("app.ac-link.txt");

    // successful scalar build: identity travels with the receipt
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &scalar_src("p-gcd-euclid.at"),
        "--source-profile",
        SCALAR,
        "--entry",
        "main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    assert!(b
        .stdout
        .contains("source-profile: auto.source.scalar-i32.draft"));
    assert!(b
        .stdout
        .contains("hir-identity: auto.hir.scalar.draft/1/scalar-i32-draft"));
    let receipt_text = std::fs::read_to_string(&receipt).unwrap();
    assert!(receipt_text.contains("source-profile: auto.source.scalar-i32.draft"));
    assert!(receipt_text.contains("hir-schema: auto.hir.scalar.draft"));
    assert!(receipt_text.contains("hir-profile: scalar-i32-draft"));
    assert!(receipt_text.contains("source-input-hash: "));

    // simulate precious prior outputs: overwrite all three with sentinels
    const SENTINEL: &[u8] = b"PLAN-767-SENTINEL-PRIOR-ARTIFACT";
    std::fs::write(&exe, SENTINEL).unwrap();
    std::fs::write(&obj, SENTINEL).unwrap();
    std::fs::write(&receipt, SENTINEL).unwrap();
    let before = (sha256(&exe), sha256(&obj), sha256(&receipt));

    // a REJECTING scalar source build must not touch them
    let bad = repo_path("experimental/ac-core/fixtures/scalar-source/invalid/e01-mixed-eq.at")
        .display()
        .to_string();
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &bad,
        "--source-profile",
        SCALAR,
        "--entry",
        "main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 1, "expected pipeline rejection");
    assert_eq!(before, (sha256(&exe), sha256(&obj), sha256(&receipt)));

    // a rejecting HIR-mode build (unknown identity) preserves them too
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &repo_path("experimental/ac-core/fixtures/scalar-hir/invalid/i09-unknown-triple.atom")
            .display()
            .to_string(),
        "--entry",
        "d_main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 1, "unknown identity must reject");
    assert_eq!(before, (sha256(&exe), sha256(&obj), sha256(&receipt)));
}
