//! PLAN-770 r6 review replay set (review-r6 findings P770-R6-01/02 plus
//! the R5-01 sibling controls c03/c04): the frozen counterexamples p01..p04
//! and controls c01..c04 from `docs/plans/attachments/770/review-r6/`, made
//! permanent, plus a composed oracle for the re-entry generation model (a
//! view captured against a PREVIOUS construction of a re-let owner reads
//! as stale even after the fresh `let`). Same policy as r2..r5: legal
//! programs run as real PEs on BOTH channels with exact traces; rejections
//! assert their frozen diagnostic codes on both channels.

#![cfg(windows)]

mod common;

use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::descriptor;
use auto_ac_prototype::hir;
use auto_ac_prototype::source::compile_text_source;
use auto_ac_prototype::verify;
use common::run_deadlined;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const TEXT: &str = "auto.source.text.draft";
const TEXT_CAP: &str = "ac.runtime.text.v1";

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac770-review-r6").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn check_err(case: &str, src: &str, code: &str) {
    match compile_text_source(src, "r6.at", &[]) {
        Ok(_) => panic!("{case}: expected rejection with `{code}`, got OK"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn check_ok(case: &str, src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_text_source(src, "r6.at", &[])
        .unwrap_or_else(|ds| panic!("{case}: expected OK, got {}", render(&ds)))
}

fn render(ds: &[Diagnostic]) -> String {
    ds.iter().map(|d| d.render("r6.at", "")).collect()
}

fn assert(cond: bool, case: &str, code: &str, ds: &[Diagnostic]) {
    assert!(
        cond,
        "{case}: expected code `{code}`, got codes {:?}\n{}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(),
        render(ds)
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

fn hir_verify_err(case: &str, bundle: hir::Bundle, code: &str) {
    match verify::verify_text(bundle) {
        Ok(_) => panic!("{case}: HIR leg expected rejection `{code}`, got Checked"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn atom_verify_err(case: &str, atom: &str, code: &str) {
    let bundle = descriptor::bind(auto_ac_prototype::text_descriptor(), atom)
        .unwrap_or_else(|_| panic!("{case}: frozen atom bind failed"));
    hir_verify_err(case, bundle, code);
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

// -- R6-02: statically unreachable break/continue never pollutes exits -----------

#[test]
fn r6_p01_unreachable_break_view_not_polluted() {
    // the `if false` arm never executes: the executable path keeps the
    // static empty view — no owned allocation, no FREE, and the post-loop
    // read stays legal (the r5 exit collector had recorded the dead arm)
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<1 { if false { let s Str=\"abc\"; v=s.to(str); break }; i=i+1 }; return v.len() }";
    both_legs("p01", src, 0, "TEXT-SCALARS n=0\n");
}

#[test]
fn r6_p02_unreachable_continue_view_not_polluted() {
    // the mirrored continue shape
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<1 { if false { let s Str=\"abc\"; v=s.to(str); continue }; i=i+1 }; return v.len() }";
    both_legs("p02", src, 0, "TEXT-SCALARS n=0\n");
}

// -- R6-01: re-entry states validate reads inside the loop ------------------------

#[test]
fn r6_p03_continue_reentry_read_rejected() {
    // iteration 1 stores a view of s and continues (s freed at the
    // continue); iteration 2 reads v BEFORE the fresh let — the view
    // points at the freed buffer, front-door rejection
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<2 { if i>0 { return v.len() }; let s Str=\"abc\"; v=s.to(str); i=i+1; continue }; return 0 }";
    check_err("p03", src, "verify.owned-scope-escape");
    atom_verify_err("p03", P03_ATOM, "verify.owned-scope-escape");
}

#[test]
fn r6_p04_natural_reentry_read_rejected() {
    // the same through the natural back-edge (no continue keyword)
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<2 { if i>0 { return v.len() }; let s Str=\"abc\"; v=s.to(str); i=i+1 }; return 0 }";
    check_err("p04", src, "verify.owned-scope-escape");
    atom_verify_err("p04", P04_ATOM, "verify.owned-scope-escape");
}

#[test]
fn r6_c01_reentry_live_owner_stays_legal() {
    // control: the backing owner lives OUTSIDE the loop and survives every
    // iteration — re-entry reads stay legal, unique FREE
    let src = "fn main() int { let s Str=\"abc\"; var v str=\"\"; var i int=0; for i<2 { if i>0 { return v.len() }; v=s.to(str); i=i+1; continue }; return 0 }";
    both_legs(
        "c01",
        src,
        3,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
    );
}

#[test]
fn r6_c02_reentry_static_view_stays_legal() {
    // control: a static (literal) view has no backing owner — no FREE at
    // all; the loop itself is legal
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<2 { if i>0 { return v.len() }; v=\"abc\"; i=i+1 }; return 0 }";
    both_legs("c02", src, 3, "TEXT-SCALARS n=3\n");
}

#[test]
fn r6_stale_generation_after_relet_rejected() {
    // composed oracle for the generation model (r6 R6-01): the view is
    // captured in iteration 1; iteration 2 RE-LETS s (a new construction)
    // and reads v AFTER the fresh let but BEFORE reassigning it — the view
    // still points at the freed previous buffer and reads as STALE even
    // though the local id is alive again
    let src = "fn main() int {\n    var v str = \"\"\n    var i int = 0\n    for i < 2 {\n        let s Str = \"abc\"\n        if i > 0 { return v.len() }\n        v = s.to(str)\n        i = i + 1\n    }\n    return 0\n}\n";
    check_err("stale-gen", src, "verify.owned-scope-escape");
}

// -- R5-01 sibling controls carried by this round's set ---------------------------

#[test]
fn r6_c03_nested_rhs_skips() {
    // outer lhs executes; the inner && skips ITS rhs; the outer && then
    // skips its own rhs — every allocated temp frees exactly once, LIFO
    let src = "fn main() int { let b bool=(\"a\"+\"b\").len()>0 && ((\"c\"+\"d\").len()==0 && (\"e\"+\"f\").len()>0); if b { return 1 }; return 0 }";
    both_legs(
        "c03",
        src,
        0,
        "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\nCONCAT a=1 b=1\nCONCAT id=2\nTEXT-SCALARS n=2\nFREE id=2\nFREE id=1\n",
    );
}

#[test]
fn r6_c04_inner_lhs_skips_outer_or_executes_rhs() {
    // the inner && is false (lhs false → rhs skipped); the outer || then
    // EXECUTES its rhs — both freed, LIFO
    let src = "fn main() int { let b bool=((\"a\"+\"b\").len()==0 && (\"c\"+\"d\").len()>0) || (\"e\"+\"f\").len()>0; if b { return 1 }; return 0 }";
    both_legs(
        "c04",
        src,
        1,
        "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\nCONCAT a=1 b=1\nCONCAT id=2\nTEXT-SCALARS n=2\nFREE id=2\nFREE id=1\n",
    );
}

/// The review's frozen direct-HIR counterexample for R6-01 (p03): the
/// continue back-edge frees the loop-scoped owner whose view iteration 2
/// reads before the fresh let.
const P03_ATOM: &str = r#"
hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m_src {
        path: "r6"
        types {
            type t_i32 i32 {}
            type t_bool bool {}
            type t_str str {}
            type t_Str Str {}
            type t_char char {}
            type t_f_main function { params: []; result: "t_i32" }
            }
        declarations {
            function d_main t_f_main { name: "sf_main"; body: "b_main" }
            }
        bodies {
            body b_main {
                owner: "d_main"
                entry: "k0"
                locals {
                    local l0 t_str { name: "v"; role: binding; mutable: true }
                    local l1 t_i32 { name: "i"; role: binding; mutable: true }
                    local l2 t_Str { name: "s"; role: binding; mutable: false }
                    }
                places {
                    place p0 local { local: l0; type: t_str; access: mutable }
                    place p1 local { local: l1; type: t_i32; access: mutable }
                    }
                expressions {
                    expr e0 str_lit { type: t_str; text: "" }
                    expr e1 constant { type: t_i32; value: 0i32 }
                    expr e2 read_local { type: t_i32; local: l1 }
                    expr e3 constant { type: t_i32; value: 2i32 }
                    expr e4 binary { type: t_bool; op: lt_i32; lhs: e2; rhs: e3 }
                    expr e5 read_local { type: t_i32; local: l1 }
                    expr e6 constant { type: t_i32; value: 0i32 }
                    expr e7 binary { type: t_bool; op: gt_i32; lhs: e5; rhs: e6 }
                    expr e8 read_local { type: t_str; local: l0 }
                    expr e9 text_len { type: t_i32; operand: e8 }
                    expr e10 str_lit { type: t_Str; text: "abc" }
                    expr e11 read_local { type: t_Str; local: l2 }
                    expr e12 text_to_borrowed { type: t_str; operand: e11 }
                    expr e13 read_local { type: t_i32; local: l1 }
                    expr e14 constant { type: t_i32; value: 1i32 }
                    expr e15 binary { type: t_i32; op: add_i32; overflow: trap; lhs: e13; rhs: e14 }
                    expr e16 constant { type: t_i32; value: 0i32 }
                    }
                blocks {
                    block k0 {
                        let(local: l0, value: e0) {}
                        let(local: l1, value: e1) {}
                        loop(id: q0, body: k4) {}
                        return(value: e16) {}
                        }
                    block k1 {
                        if(cond: e7, then: k2) {}
                        let(local: l2, value: e10) {}
                        assign(place: p0, value: e12) {}
                        assign(place: p1, value: e15) {}
                        continue(target: q0) {}
                        }
                    block k2 {
                        return(value: e9) {}
                        }
                    block k3 {
                        break(target: q0) {}
                        }
                    block k4 {
                        if(cond: e4, then: k1, else: k3) {}
                        }
                    }
                }
            }
        }
    }
"#;

/// The review's frozen direct-HIR counterexample for R6-01 (p04): the
/// same shape through the natural back-edge.
const P04_ATOM: &str = r#"hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m_src {
        path: "r6"
        types {
            type t_i32 i32 {}
            type t_bool bool {}
            type t_str str {}
            type t_Str Str {}
            type t_char char {}
            type t_f_main function { params: []; result: "t_i32" }
            }
        declarations {
            function d_main t_f_main { name: "sf_main"; body: "b_main" }
            }
        bodies {
            body b_main {
                owner: "d_main"
                entry: "k0"
                locals {
                    local l0 t_str { name: "v"; role: binding; mutable: true }
                    local l1 t_i32 { name: "i"; role: binding; mutable: true }
                    local l2 t_Str { name: "s"; role: binding; mutable: false }
                    }
                places {
                    place p0 local { local: l0; type: t_str; access: mutable }
                    place p1 local { local: l1; type: t_i32; access: mutable }
                    }
                expressions {
                    expr e0 str_lit { type: t_str; text: "" }
                    expr e1 constant { type: t_i32; value: 0i32 }
                    expr e2 read_local { type: t_i32; local: l1 }
                    expr e3 constant { type: t_i32; value: 2i32 }
                    expr e4 binary { type: t_bool; op: lt_i32; lhs: e2; rhs: e3 }
                    expr e5 read_local { type: t_i32; local: l1 }
                    expr e6 constant { type: t_i32; value: 0i32 }
                    expr e7 binary { type: t_bool; op: gt_i32; lhs: e5; rhs: e6 }
                    expr e8 read_local { type: t_str; local: l0 }
                    expr e9 text_len { type: t_i32; operand: e8 }
                    expr e10 str_lit { type: t_Str; text: "abc" }
                    expr e11 read_local { type: t_Str; local: l2 }
                    expr e12 text_to_borrowed { type: t_str; operand: e11 }
                    expr e13 read_local { type: t_i32; local: l1 }
                    expr e14 constant { type: t_i32; value: 1i32 }
                    expr e15 binary { type: t_i32; op: add_i32; overflow: trap; lhs: e13; rhs: e14 }
                    expr e16 constant { type: t_i32; value: 0i32 }
                    }
                blocks {
                    block k0 {
                        let(local: l0, value: e0) {}
                        let(local: l1, value: e1) {}
                        loop(id: q0, body: k4) {}
                        return(value: e16) {}
                        }
                    block k1 {
                        if(cond: e7, then: k2) {}
                        let(local: l2, value: e10) {}
                        assign(place: p0, value: e12) {}
                        assign(place: p1, value: e15) {}
                        }
                    block k2 {
                        return(value: e9) {}
                        }
                    block k3 {
                        break(target: q0) {}
                        }
                    block k4 {
                        if(cond: e4, then: k1, else: k3) {}
                        }
                    }
                }
            }
        }
    }
"#;
