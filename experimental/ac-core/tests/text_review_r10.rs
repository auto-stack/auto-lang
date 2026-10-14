//! PLAN-770 r10 review replay set (review-r10 finding P770-R10-01): the
//! r9 after-loop responsibility rule dropped the caller's free duty
//! whenever the dead-UNION contained an entry owner — so a loop that
//! executes ZERO times (or breaks BEFORE the transfer) leaked the
//! never-transferred buffer (LIT without FREE). The fix decides the duty
//! per INCOMING state exactly like the if-join: the after-loop entries
//! are the natural completion (only with a real back edge) and every
//! BREAK exit (the wrapped for's zero-iteration path arrives as its
//! condition-false break state); RETURN exits never reach the after-loop.
//! Live on every entry: caller keeps the duty (c02/c03/c04/c06/p05 free
//! once in the caller). Dead on every entry: the transfer happened on
//! all of them (c01/c05 — the callee frees once). MIXED: no single
//! static responsibility exists — verify.ownership-required (p01/p02,
//! and p04 through the body's if-join). Never LIT without FREE.

#![cfg(windows)]

mod common;

use auto_ac_prototype::source::compile_text_source;
use common::run_deadlined;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const TEXT: &str = "auto.source.text.draft";
const TEXT_CAP: &str = "ac.runtime.text.v1";

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac770-review-r10").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn check_err(case: &str, src: &str, code: &str) {
    match compile_text_source(src, "r10.at", &[]) {
        Ok(_) => panic!("{case}: expected rejection with `{code}`, got OK"),
        Err(ds) => {
            let codes: Vec<&str> = ds.iter().map(|d| d.code.as_str()).collect();
            assert!(
                ds.iter().any(|d| d.code == code),
                "{case}: expected code `{code}`, got codes {codes:?}"
            );
        }
    }
}

fn check_ok(case: &str, src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_text_source(src, "r10.at", &[]).unwrap_or_else(|ds| {
        panic!(
            "{case}: expected OK, got {}",
            ds.iter()
                .map(|d| d.render("r10.at", ""))
                .collect::<String>()
        )
    })
}

fn pe(case: &str, src: &str) -> common::DeadlinedRun {
    let dir = temp_dir(case);
    let src_path = dir.join("p.at");
    std::fs::write(&src_path, src).unwrap();
    let exe = dir.join("p.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &src_path.display().to_string(),
        "--source-profile",
        TEXT,
        "--entry",
        "main",
        "--capability",
        TEXT_CAP,
        "--support-lib",
    ])
    .arg(common::text_runtime_lib())
    .arg("--output")
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert!(!b.timed_out, "{case}: build deadline");
    assert_eq!(b.code, 0, "{case}: build failed:\n{}", b.stderr);
    let run = run_deadlined(Command::new(&exe), Duration::from_secs(60)).expect("spawn PE");
    assert!(!run.timed_out, "{case}: run deadline");
    run
}

fn hir_pe(case: &str, src: &str) -> common::DeadlinedRun {
    let compiled = check_ok(case, src);
    let dir = temp_dir(case);
    let atom = dir.join("p.atom");
    std::fs::write(&atom, &compiled.atom_text).unwrap();
    let exe = dir.join("p-hir.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &atom.display().to_string(),
        "--entry",
        "d_main",
        "--capability",
        TEXT_CAP,
        "--support-lib",
    ])
    .arg(common::text_runtime_lib())
    .arg("--output")
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 0, "{case}: HIR build failed:\n{}", b.stderr);
    let run = run_deadlined(Command::new(&exe), Duration::from_secs(60)).expect("spawn PE");
    assert!(!run.timed_out, "{case}: run deadline");
    run
}

fn no_double_free(case: &str, run: &common::DeadlinedRun) {
    assert!(
        !run.stderr.contains("FREE-INVALID"),
        "{case}: FREE-INVALID on stderr:\n{}",
        run.stderr
    );
}

/// Every allocation LIT must pair with exactly one FREE (alloc=free, V04).
fn one_free_per_alloc(case: &str, run: &common::DeadlinedRun) {
    let lits = run
        .stderr
        .lines()
        .filter(|l| l.starts_with("LIT id="))
        .count();
    let frees = run
        .stderr
        .lines()
        .filter(|l| l.starts_with("FREE id="))
        .count();
    assert_eq!(
        lits, frees,
        "{case}: alloc=free violated (LIT x{lits} vs FREE x{frees}):\n{}",
        run.stderr
    );
    no_double_free(case, run);
}

fn both_legs(case: &str, src: &str, code: i32, stderr: &str) {
    let run = pe(case, src);
    assert_eq!(run.code, code, "{case}: source leg exit");
    assert_eq!(run.stderr, stderr, "{case}: source leg trace");
    one_free_per_alloc(case, &run);
    let hir = hir_pe(case, src);
    assert_eq!(hir.code, code, "{case}: HIR leg exit");
    assert_eq!(hir.stderr, stderr, "{case}: HIR leg trace");
    one_free_per_alloc(case, &hir);
}

/// The caller frees once (no callee involvement in the trace).
const CALLER_TRACE: &str = "LIT len=3\nLIT id=1\nFREE id=1\n";
/// The callee consumes and frees once (with the len read).
const CALLEE_TRACE: &str = "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n";

// -- the leaks (r10 P1): mixed responsibility is REJECTED, never leaked ------

#[test]
fn r10_p01_zero_iteration_move_mixed_rejected() {
    // the wrapper condition is statically unknown: the zero-iteration
    // break (x live) and the moved break (x dead) both reach after-loop —
    // no single static free responsibility exists
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; var i int=0; for i<0 { let n int=consume(x.move); break }; return 0 }";
    check_err("p01", src, "verify.ownership-required");
}

#[test]
fn r10_p02_break_before_move_mixed_rejected() {
    // constant-true wrapper, but the FIRST break fires before the move
    // (x live) while the second path moves then breaks (x dead)
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; var i int=0; for true { if i==0 { break }; let n int=consume(x.move); break }; return 0 }";
    check_err("p02", src, "verify.ownership-required");
}

#[test]
fn r10_p04_conditional_move_break_mixed_rejected() {
    // a conditional transfer inside the body — the body's own if-join
    // reports the inconsistent responsibility
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; var i int=0; for true { if i>0 { let n int=consume(x.move) }; break }; return 0 }";
    check_err("p04", src, "verify.ownership-required");
}

#[test]
fn r10_p03_continue_repeat_move_still_rejected() {
    // a REAL continue back edge moving the outer x would transfer twice —
    // the back-edge pre-kill keeps the use-after-move rejection
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; var i int=0; for i<2 { let n int=consume(x.move); i=i+1; continue }; return 0 }";
    check_err("p03", src, "verify.use-after-move");
}

// -- legal shapes: the caller keeps the duty (zero-iteration / no move) ------

#[test]
fn r10_p05_zero_return_move_caller_frees() {
    // zero-iteration wrapper with a return-move body: the ONLY after-loop
    // entry is the condition-false break (x live) — the return exits
    // never reach after-loop, so the caller keeps a consistent duty
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; var i int=0; for i<0 { return consume(x.move) }; return 0 }";
    both_legs("p05", src, 0, CALLER_TRACE);
}

#[test]
fn r10_c02_zero_no_move_caller_frees() {
    let src = "fn main() int { let x Str=\"abc\"; var i int=0; for i<0 { break }; return 0 }";
    both_legs("c02", src, 0, CALLER_TRACE);
}

#[test]
fn r10_c03_break_no_move_caller_frees() {
    let src = "fn main() int { let x Str=\"abc\"; for true { break }; return 0 }";
    both_legs("c03", src, 0, CALLER_TRACE);
}

#[test]
fn r10_c04_return_versus_break_caller_frees() {
    // the return path transfers (and never comes back); the break path
    // keeps x — every after-loop entry has x live
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; var i int=0; for true { if i>0 { return consume(x.move) } else { break } }; return 0 }";
    both_legs("c04", src, 0, CALLER_TRACE);
}

#[test]
fn r10_c06_first_return_before_transfer_caller_frees() {
    // the taken path returns BEFORE the transfer — x stays owned and the
    // caller frees exactly once
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; var i int=0; for true { if i==0 { return 0 }; return consume(x.move) }; return 0 }";
    both_legs("c06", src, 0, CALLER_TRACE);
}

// -- legal shapes: the transfer happens on EVERY after-loop entry ------------

#[test]
fn r10_c01_unconditional_move_break_callee_frees() {
    // constant-true wrapper: the zero-iteration break is statically
    // unreachable, the only exit moves first — the duty drops to the
    // callee
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; for true { let n int=consume(x.move); break }; return 0 }";
    both_legs("c01", src, 0, CALLEE_TRACE);
}

#[test]
fn r10_c05_once_return_move_callee_frees() {
    // the return-only loop (r9's p03) — no after-loop entry at all; the
    // callee frees once, exit 3
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; for true { return consume(x.move) }; return 0 }";
    both_legs("c05", src, 3, CALLEE_TRACE);
}
