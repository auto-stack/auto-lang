//! PLAN-770 r9 review replay set (review-r9 finding P770-R9-01): the
//! legacy loop-ENTRY pre-kill (r2 QA-07) still marked every entry-live
//! owner moved anywhere in the body as dead BEFORE the first entry,
//! ignoring the real back-edge classification the walk has had since
//! r7/r8 — a return-only loop executes at most once, so its single
//! consume(x.move) of an OUTER owner was misjudged as a repeated move.
//! The fix gates the pre-kill on a real back edge (the body can re-enter
//! at all). p03 (the mis-rejected once-return-move) builds and runs
//! clean; p04 (a REAL repeatable loop double-moving x) keeps its
//! use-after-move rejection; c05/c06 isolate the loop-entry factor
//! (fresh-owner-in-loop and no-loop-at-all controls); c01..c04 carry
//! this round's constant-termination and refreshed-view controls, and
//! p01/p02 hold the real-back-edge stale-view rejections.

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
    let dir = std::env::temp_dir().join("ac770-review-r9").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn check_err(case: &str, src: &str, code: &str) {
    match compile_text_source(src, "r9.at", &[]) {
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
    compile_text_source(src, "r9.at", &[]).unwrap_or_else(|ds| {
        panic!(
            "{case}: expected OK, got {}",
            ds.iter().map(|d| d.render("r9.at", "")).collect::<String>()
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

fn both_legs(case: &str, src: &str, code: i32, stderr: &str) {
    let run = pe(case, src);
    assert_eq!(run.code, code, "{case}: source leg exit");
    assert_eq!(run.stderr, stderr, "{case}: source leg trace");
    no_double_free(case, &run);
    let hir = hir_pe(case, src);
    assert_eq!(hir.code, code, "{case}: HIR leg exit");
    assert_eq!(hir.stderr, stderr, "{case}: HIR leg trace");
    no_double_free(case, &hir);
}

/// The single-transfer trace: one LIT, the callee's len, the callee's
/// single FREE — the caller never frees a moved value.
const MOVE_TRACE: &str = "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n";

// -- R9-01: a return-only loop executes at most once — the first move is
// the only move ---------------------------------------------------------------

#[test]
fn r9_p03_once_return_move_legal() {
    // the loop condition is constant true and the body RETURNS through
    // consume(x.move): one execution, one transfer, the callee frees once.
    // The legacy entry pre-kill had marked x dead before the first entry
    // and rejected the FIRST move as a repeat.
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; for true { return consume(x.move) }; return 0 }";
    both_legs("p03", src, 3, MOVE_TRACE);
}

#[test]
fn r9_p04_real_repeated_move_still_rejected() {
    // a REAL natural loop (i<2) moving the outer x every iteration would
    // transfer twice — the back-edge pre-kill (now fixpoint-backed) keeps
    // the use-after-move rejection
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; var i int=0; for i<2 { let n int=consume(x.move); i=i+1 }; return 0 }";
    check_err("p04", src, "verify.use-after-move");
}

#[test]
fn r9_c05_fresh_owner_return_move_legal() {
    // the same return-only shape with the owner constructed INSIDE the
    // loop — a fresh owner per iteration was never pre-killed; holds the
    // callee-signature/move-spelling/lowering factors constant
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { for true { let x Str=\"abc\"; return consume(x.move) }; return 0 }";
    both_legs("c05", src, 3, MOVE_TRACE);
}

#[test]
fn r9_c06_outside_loop_return_move_legal() {
    // no loop at all: a plain return consume(x.move) — isolates the
    // loop-entry pre-kill as the only difference from p03
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; return consume(x.move) }";
    both_legs("c06", src, 3, MOVE_TRACE);
}

#[test]
fn r9_constant_terminated_break_move_legal() {
    // composed matrix: a zero-re-entry loop terminated by a CONSTANT
    // break allows the single transfer of an outer owner. The wrapper
    // condition must itself be provably true: a dynamic condition keeps
    // the zero-iteration break exit alive, making the responsibility
    // MIXED (r10 rejects that shape — see text_review_r10)
    let src = "fn consume(move s Str) int { return s.len() } fn main() int { let x Str=\"abc\"; var i int=0; for true { let n int=consume(x.move); if true { break }; i=i+1 }; return 0 }";
    both_legs("const-break-move", src, 0, MOVE_TRACE);
}

// -- this round's constant-termination and refreshed-view controls ---------------

#[test]
fn r9_c01_nested_constant_break_legal() {
    // nested constant termination (outer true + inner false-else-break)
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); if true { if false { i=i+1 } else { break } } }; return 0 }";
    both_legs(
        "c01",
        src,
        0,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nLIT len=3\nLIT id=2\nFREE id=2\nFREE id=1\n",
    );
}

#[test]
fn r9_c02_false_then_true_return_legal() {
    // const-false outer arm + const-true inner return: only the
    // return path can run
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); if false { i=i+1 } else { if true { return 0 } } }; return 0 }";
    both_legs(
        "c02",
        src,
        0,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nLIT len=3\nLIT id=2\nFREE id=2\nFREE id=1\n",
    );
}

#[test]
fn r9_c03_constant_continue_refresh_legal() {
    // a real continue back edge guarded by a CONSTANT, with the view
    // refreshed before every read — re-entry validation runs and holds
    let src = "fn main() int { var i int=0; var n int=0; var v str=\"\"; for i<2 { let s Str=\"abc\"; v=s.to(str); n=n+v.len(); i=i+1; if true { continue } }; return n }";
    both_legs(
        "c03",
        src,
        6,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\nLIT len=3\nLIT id=2\nTEXT-SCALARS n=3\nFREE id=2\n",
    );
}

#[test]
fn r9_c04_unknown_natural_refresh_legal() {
    // an UNKNOWN possible-break arm with per-iteration refresh: the
    // natural back edge stays validated
    let src = "fn main() int { var i int=0; var n int=0; var v str=\"\"; for i<2 { let s Str=\"abc\"; v=s.to(str); n=n+v.len(); i=i+1; if i>1 { break } }; return n }";
    both_legs(
        "c04",
        src,
        6,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\nLIT len=3\nLIT id=2\nTEXT-SCALARS n=3\nFREE id=2\n",
    );
}

// -- real-back-edge stale-view rejections held ------------------------------------

#[test]
fn r9_p01_constant_continue_stale_rejected() {
    // a real constant-guarded continue back edge with a stale view —
    // the re-entry fixpoint keeps the rejection
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<2 { if i>0 { return v.len() }; let s Str=\"abc\"; v=s.to(str); i=i+1; if true { continue } }; return 0 }";
    check_err("p01", src, "verify.owned-scope-escape");
}

#[test]
fn r9_p02_false_no_else_natural_stale_rejected() {
    // `if false { break }` (no else) leaves the natural fall-through as
    // a real back edge — the stale view stays rejected
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<2 { if i>0 { return v.len() }; let s Str=\"abc\"; v=s.to(str); i=i+1; if false { break } }; return 0 }";
    check_err("p02", src, "verify.owned-scope-escape");
}
