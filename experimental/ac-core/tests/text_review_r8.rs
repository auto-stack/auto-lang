//! PLAN-770 r8 review replay set (review-r8 finding P770-R8-01): loops
//! whose ONLY exits are breaks/returns guarded by CONSTANT conditions were
//! still misclassified as natural back edges by the const-blind
//! `block_terminates` helper — `if true { break }`, `if false { .. } else
//! { break }` and `if true { return }` fabricated a re-entry and rejected
//! the first read of a live outer owner. The fix makes the terminator
//! analysis consume the SAME constant reachability the walk uses
//! (const_truth): only arms that can actually run decide termination.
//! p01/p02/p04 (the review's mis-rejected inputs) build and run clean;
//! p03 (a REAL back edge with a stale view) stays rejected; c01 (dynamic
//! both-arm break), c02 (real continue with per-iteration refresh) and
//! c03 (direct return) are the controls. p03 is byte-identical to r6's
//! p03 — its frozen-atom leg is permanently held in text_review_r6.

#![cfg(windows)]

mod common;

use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::source::compile_text_source;
use common::run_deadlined;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const TEXT: &str = "auto.source.text.draft";
const TEXT_CAP: &str = "ac.runtime.text.v1";

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac770-review-r8").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn check_err(case: &str, src: &str, code: &str) {
    match compile_text_source(src, "r8.at", &[]) {
        Ok(_) => panic!("{case}: expected rejection with `{code}`, got OK"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn check_ok(case: &str, src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_text_source(src, "r8.at", &[]).unwrap_or_else(|ds| {
        panic!(
            "{case}: expected OK, got {}",
            ds.iter().map(|d| d.render("r8.at", "")).collect::<String>()
        )
    })
}

fn assert(cond: bool, case: &str, code: &str, ds: &[Diagnostic]) {
    assert!(
        cond,
        "{case}: expected code `{code}`, got codes {:?}\n{}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(),
        ds.iter().map(|d| d.render("r8.at", "")).collect::<String>()
    );
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

/// The shared legal shape of this round's mis-rejected inputs: read the
/// live outer owner FIRST, then construct s, rebind v, and leave the loop
/// through an exit that cannot reach the block end — no back edge, so no
/// second iteration may be fabricated.
const TERM_TRACE: &str =
    "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nLIT len=3\nLIT id=2\nFREE id=2\nFREE id=1\n";

#[test]
fn r8_p01_constant_then_break_not_a_back_edge() {
    // `if true { break }`: only the then arm can run and it terminates —
    // the loop has no back edge and the first len(v) (backing the live
    // outer x) must stay legal
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); if true { break } }; return 0 }";
    both_legs("p01", src, 0, TERM_TRACE);
}

#[test]
fn r8_p02_constant_false_else_break_not_a_back_edge() {
    // `if false { .. } else { break }`: only the else arm can run
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); if false { i=i+1 } else { break } }; return 0 }";
    both_legs("p02", src, 0, TERM_TRACE);
}

#[test]
fn r8_p04_constant_return_not_a_back_edge() {
    // `if true { return 0 }`: a constant-guarded RETURN also terminates
    // the body — the function exits before any back edge could run
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); if true { return 0 } }; return 0 }";
    both_legs("p04", src, 0, TERM_TRACE);
}

#[test]
fn r8_nested_constant_termination_not_a_back_edge() {
    // composed matrix: the constant reachability composes — an outer
    // constant-true if whose then arm is itself a constant-false
    // if+else-break still leaves NO fall-through path
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); if true { if false { i=i+1 } else { break } } }; return 0 }";
    both_legs("nested-const", src, 0, TERM_TRACE);
}

#[test]
fn r8_p03_real_backedge_stale_view_still_rejected() {
    // the REAL back edge (continue after the rebind) keeps its r6
    // rejection — constant-aware termination must not weaken actual
    // re-entry validation (byte-identical to r6 p03; the frozen-atom
    // leg lives in text_review_r6)
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<2 { if i>0 { return v.len() }; let s Str=\"abc\"; v=s.to(str); i=i+1; continue }; return 0 }";
    check_err("p03", src, "verify.owned-scope-escape");
}

#[test]
fn r8_c01_dynamic_both_arm_break_legal() {
    // DYNAMIC condition with BOTH arms breaking: every runtime path
    // terminates, so there is still no back edge — legal with the exact
    // expected trace
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); if i>0 { break } else { break } }; return 0 }";
    both_legs("c01", src, 0, TERM_TRACE);
}

#[test]
fn r8_c02_real_continue_refresh_legal() {
    // the real continue back edge with a per-iteration view refresh —
    // re-entry validation runs and stays legal (exit 6, per-id pairing)
    let src = "fn main() int { var v str=\"\"; var i int=0; var n int=0; for i<2 { let s Str=\"abc\"; v=s.to(str); n=n+v.len(); i=i+1; continue }; return n }";
    both_legs(
        "c02",
        src,
        6,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\nLIT len=3\nLIT id=2\nTEXT-SCALARS n=3\nFREE id=2\n",
    );
}

#[test]
fn r8_c03_direct_return_legal() {
    // a bare top-level return terminates the body (r7 covered the direct
    // break; this is the direct-return sibling)
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); return 0 }; return 0 }";
    both_legs("c03", src, 0, TERM_TRACE);
}
