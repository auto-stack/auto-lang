//! PLAN-770 r7 review replay set (review-r7 finding P770-R7-01): the
//! frozen direct-HIR counterexample m01 (a break-only loop with NO back
//! edge was fabricated into a re-entry by the r6 fixpoint) and its
//! controls m02/c05, plus this round's stale-re-entry rejections
//! (p01/p02, through a call and a nested loop) and the refreshed-view /
//! unreachable-continue / nested-fresh-owner controls (c01..c04). The
//! loop-exit classification matrix: natural completion and CONTINUE exits
//! feed the re-entry fixpoint; BREAK/RETURN-terminated bodies never do.
//! Same policy as r2..r6: legal programs run as real PEs on both
//! channels with exact traces; rejections assert frozen codes on both
//! channels (the direct-HIR legs bind the review's frozen atoms
//! byte-for-byte).

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
    let dir = std::env::temp_dir().join("ac770-review-r7").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn check_err(case: &str, src: &str, code: &str) {
    match compile_text_source(src, "r7.at", &[]) {
        Ok(_) => panic!("{case}: expected rejection with `{code}`, got OK"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn check_ok(case: &str, src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_text_source(src, "r7.at", &[])
        .unwrap_or_else(|ds| panic!("{case}: expected OK, got {}", render(&ds)))
}

fn render(ds: &[Diagnostic]) -> String {
    ds.iter().map(|d| d.render("r7.at", "")).collect()
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

/// Build + run a FROZEN direct-HIR atom (the review's counterexamples
/// use the named-id atom dialect; both bind through the same text
/// descriptor and verify_text front door).
fn frozen_atom_pe(case: &str, atom: &str) -> common::DeadlinedRun {
    let dir = temp_dir(case);
    let atom_path = dir.join("p.atom");
    std::fs::write(&atom_path, atom).unwrap();
    let exe = dir.join("p.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &atom_path.display().to_string(),
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
    assert_eq!(b.code, 0, "{case}: frozen atom build failed:\n{}", b.stderr);
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

// -- R7-01: a break-only body has NO back edge — no fabricated re-entry -------

#[test]
fn r7_m01_direct_break_only_not_rewalked() {
    // the body's only exit is the break: the FIRST len(v) reads the live
    // outer owner x; the r6 fixpoint had fed the break-time state (v
    // backing the freed s) back into a second iteration that never runs
    let run = frozen_atom_pe("m01", M01_ATOM);
    assert_eq!(run.code, 0);
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nLIT len=3\nLIT id=2\nFREE id=2\nFREE id=1\n"
    );
    no_double_free("m01", &run);
}

#[test]
fn r7_m02_direct_break_live_owner_control() {
    // the same break-only graph with the second view ALSO taken from the
    // live outer owner — Checked and a clean run either way
    let run = frozen_atom_pe("m02", M02_ATOM);
    assert_eq!(run.code, 0);
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nLIT len=3\nLIT id=2\nFREE id=2\nFREE id=1\n"
    );
    no_double_free("m02", &run);
}

#[test]
fn r7_c05_break_only_source_leg() {
    // the source-level shape of m01 (through the frontend's for-wrapper):
    // the borrowed read precedes the rebind and the break frees the inner
    // owner on the loop's only exit — legal on both channels
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); break }; return 0 }";
    both_legs(
        "c05",
        src,
        0,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nLIT len=3\nLIT id=2\nFREE id=2\nFREE id=1\n",
    );
}

#[test]
fn r7_return_only_body_not_rewalked() {
    // termination/back-edge matrix: a RETURN-only body terminates the
    // function — no re-entry, and the read inside is legal (a fresh view
    // of the live outer owner)
    let src = "fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { return v.len() }; return 0 }";
    both_legs(
        "return-only",
        src,
        3,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
    );
}

// -- stale re-entry rejections carried by this round (r6 semantics held) -------

#[test]
fn r7_p01_stale_reentry_through_call_rejected() {
    // iteration 2 passes the PREVIOUS iteration's view into a call — the
    // read happens inside the callee but the stale source is caught at
    // the front door on both channels
    let src = "fn read(v str) int { return v.len() } fn main() int { var v str=\"\"; var i int=0; for i<2 { let s Str=\"abc\"; if i>0 { return read(v) }; v=s.to(str); i=i+1 }; return 0 }";
    check_err("p01", src, "verify.owned-scope-escape");
    atom_verify_err("p01", P01_ATOM, "verify.owned-scope-escape");
}

#[test]
fn r7_p02_nested_stale_view_rejected() {
    // the stale view is read inside a NESTED loop one frame down
    let src = "fn main() int { var v str=\"\"; var i int=0; var j int=0; for i<2 { let s Str=\"abc\"; j=0; for j<1 { if i>0 { return v.len() }; j=j+1 }; v=s.to(str); i=i+1 }; return 0 }";
    check_err("p02", src, "verify.owned-scope-escape");
    atom_verify_err("p02", P02_ATOM, "verify.owned-scope-escape");
}

// -- refreshed-view and nested-fresh-owner controls ------------------------------

#[test]
fn r7_c01_refreshed_view_natural_reentry_legal() {
    // each iteration REASSIGNS the view before reading it — the re-entry
    // state is validated (a real back edge) and stays legal
    let src = "fn main() int { var v str=\"\"; var i int=0; var n int=0; for i<2 { let s Str=\"abc\"; v=s.to(str); n=n+v.len(); i=i+1 }; return n }";
    both_legs(
        "c01",
        src,
        6,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\nLIT len=3\nLIT id=2\nTEXT-SCALARS n=3\nFREE id=2\n",
    );
}

#[test]
fn r7_c02_refreshed_view_continue_reentry_legal() {
    // the same with an explicit continue back edge
    let src = "fn main() int { var v str=\"\"; var i int=0; var n int=0; for i<2 { let s Str=\"abc\"; v=s.to(str); n=n+v.len(); i=i+1; continue }; return n }";
    both_legs(
        "c02",
        src,
        6,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\nLIT len=3\nLIT id=2\nTEXT-SCALARS n=3\nFREE id=2\n",
    );
}

#[test]
fn r7_c03_true_else_unreachable_continue_legal() {
    // `if true` prunes the else arm: its continue never records an exit
    // state and the loop exits through the then arm — no owned alloc
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<1 { if true { i=i+1 } else { let s Str=\"abc\"; v=s.to(str); continue } }; return v.len() }";
    both_legs("c03", src, 0, "TEXT-SCALARS n=0\n");
}

#[test]
fn r7_c04_nested_fresh_owner_legal() {
    // a nested loop taking a fresh view of the OUTER loop's fresh owner
    // every iteration — 2x2 reads, per-iteration pairing
    let src = "fn main() int { var i int=0; var j int=0; var n int=0; for i<2 { let s Str=\"abc\"; j=0; for j<2 { let v str=s.to(str); n=n+v.len(); j=j+1 }; i=i+1 }; return n }";
    both_legs(
        "c04",
        src,
        12,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nTEXT-SCALARS n=3\nFREE id=1\nLIT len=3\nLIT id=2\nTEXT-SCALARS n=3\nTEXT-SCALARS n=3\nFREE id=2\n",
    );
}

/// The review's frozen direct-HIR counterexample (m01): a break-only loop — the body reads v (backing the live outer x), constructs s, rebinds v to s and breaks; there is NO back edge, so no second iteration may be fabricated.
const M01_ATOM: &str = r#"
hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module(id: "m_src") {
        path: "break-only"
        types {
            type(id: "t_i32", kind: i32) {}
            type(id: "t_bool", kind: bool) {}
            type(id: "t_str", kind: str) {}
            type(id: "t_Str", kind: Str) {}
            type(id: "t_char", kind: char) {}
            type(id: "t_f_main", kind: function) { params: []; result: "t_i32" }
        }
        declarations {
            function(id: "d_main", type: "t_f_main") { name: "sf_main"; body: "b_main" }
        }
        bodies {
            body(id: "b_main") {
                owner: "d_main"; entry: "k0"
                locals {
                    local(id: "l0", type: "t_Str") { name: "x"; role: binding; mutable: false }
                    local(id: "l1", type: "t_str") { name: "v"; role: binding; mutable: true }
                    local(id: "l2", type: "t_i32") { name: "i"; role: binding; mutable: true }
                    local(id: "l3", type: "t_i32") { name: "n"; role: binding; mutable: false }
                    local(id: "l4", type: "t_Str") { name: "s"; role: binding; mutable: false }
                }
                places {
                    place(id: "p0", kind: local) { local: "l1"; type: "t_str"; access: mutable }
                }
                expressions {
                    expr(id: "e0", kind: str_lit) { type: "t_Str"; text: "abc" }
                    expr(id: "e1", kind: read_local) { type: "t_Str"; local: "l0" }
                    expr(id: "e2", kind: text_to_borrowed) { type: "t_str"; operand: "e1" }
                    expr(id: "e3", kind: constant) { type: "t_i32"; value: 0i32 }
                    expr(id: "e7", kind: read_local) { type: "t_str"; local: "l1" }
                    expr(id: "e8", kind: text_len) { type: "t_i32"; operand: "e7" }
                    expr(id: "e9", kind: str_lit) { type: "t_Str"; text: "def" }
                    expr(id: "e10", kind: read_local) { type: "t_Str"; local: "l4" }
                    expr(id: "e11", kind: text_to_borrowed) { type: "t_str"; operand: "e10" }
                    expr(id: "e12", kind: constant) { type: "t_i32"; value: 0i32 }
                }
                blocks {
                    block(id: "k0") {
                        let(local: "l0", value: "e0") {}
                        let(local: "l1", value: "e2") {}
                        let(local: "l2", value: "e3") {}
                        loop(id: "q0", body: "k1") {}
                        return(value: "e12") {}
                    }
                    block(id: "k1") {
                        let(local: "l3", value: "e8") {}
                        let(local: "l4", value: "e9") {}
                        assign(place: "p0", value: "e11") {}
                        break(target: "q0") {}
                    }
                }
            }
        }
    }
}
"#;

/// The review's frozen direct-HIR control (m02): the same break-only graph with the second view taken from the still-live outer owner.
const M02_ATOM: &str = r#"
hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module(id: "m_src") {
        path: "break-only"
        types {
            type(id: "t_i32", kind: i32) {}
            type(id: "t_bool", kind: bool) {}
            type(id: "t_str", kind: str) {}
            type(id: "t_Str", kind: Str) {}
            type(id: "t_char", kind: char) {}
            type(id: "t_f_main", kind: function) { params: []; result: "t_i32" }
        }
        declarations {
            function(id: "d_main", type: "t_f_main") { name: "sf_main"; body: "b_main" }
        }
        bodies {
            body(id: "b_main") {
                owner: "d_main"; entry: "k0"
                locals {
                    local(id: "l0", type: "t_Str") { name: "x"; role: binding; mutable: false }
                    local(id: "l1", type: "t_str") { name: "v"; role: binding; mutable: true }
                    local(id: "l2", type: "t_i32") { name: "i"; role: binding; mutable: true }
                    local(id: "l3", type: "t_i32") { name: "n"; role: binding; mutable: false }
                    local(id: "l4", type: "t_Str") { name: "s"; role: binding; mutable: false }
                }
                places {
                    place(id: "p0", kind: local) { local: "l1"; type: "t_str"; access: mutable }
                }
                expressions {
                    expr(id: "e0", kind: str_lit) { type: "t_Str"; text: "abc" }
                    expr(id: "e1", kind: read_local) { type: "t_Str"; local: "l0" }
                    expr(id: "e2", kind: text_to_borrowed) { type: "t_str"; operand: "e1" }
                    expr(id: "e3", kind: constant) { type: "t_i32"; value: 0i32 }
                    expr(id: "e7", kind: read_local) { type: "t_str"; local: "l1" }
                    expr(id: "e8", kind: text_len) { type: "t_i32"; operand: "e7" }
                    expr(id: "e9", kind: str_lit) { type: "t_Str"; text: "def" }
                    expr(id: "e10", kind: read_local) { type: "t_Str"; local: "l0" }
                    expr(id: "e11", kind: text_to_borrowed) { type: "t_str"; operand: "e10" }
                    expr(id: "e12", kind: constant) { type: "t_i32"; value: 0i32 }
                }
                blocks {
                    block(id: "k0") {
                        let(local: "l0", value: "e0") {}
                        let(local: "l1", value: "e2") {}
                        let(local: "l2", value: "e3") {}
                        loop(id: "q0", body: "k1") {}
                        return(value: "e12") {}
                    }
                    block(id: "k1") {
                        let(local: "l3", value: "e8") {}
                        let(local: "l4", value: "e9") {}
                        assign(place: "p0", value: "e11") {}
                        break(target: "q0") {}
                    }
                }
            }
        }
    }
}
"#;

/// The review's frozen direct-HIR rejection leg (p01): the stale previous-iteration view is read through a call.
const P01_ATOM: &str = r#"
hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m_src {
        path: "r7"
        types {
            type t_i32 i32 {}
            type t_bool bool {}
            type t_str str {}
            type t_Str Str {}
            type t_char char {}
            type t_f_read function { params: ["t_str"]; result: "t_i32" }
            type t_f_main function { params: []; result: "t_i32" }
            }
        declarations {
            function d_read t_f_read { name: "sf_read"; body: "b_read" }
            function d_main t_f_main { name: "sf_main"; body: "b_main" }
            }
        bodies {
            body b_read {
                owner: "d_read"
                entry: "k0"
                locals {
                    local l0 t_str { name: "v"; role: param; index: 0u32 }
                    }
                expressions {
                    expr e0 read_local { type: t_str; local: l0 }
                    expr e1 text_len { type: t_i32; operand: e0 }
                    }
                blocks {
                    block k0 {
                        return(value: e1) {}
                        }
                    }
                }
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
                    expr e5 str_lit { type: t_Str; text: "abc" }
                    expr e6 read_local { type: t_i32; local: l1 }
                    expr e7 constant { type: t_i32; value: 0i32 }
                    expr e8 binary { type: t_bool; op: gt_i32; lhs: e6; rhs: e7 }
                    expr e9 read_local { type: t_str; local: l0 }
                    expr e10 call { type: t_i32; callee: d_read; eval_args: [e9]; bindings: [{ param: 0u32, arg: 0u32 }] }
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
                        let(local: l2, value: e5) {}
                        if(cond: e8, then: k2) {}
                        assign(place: p0, value: e12) {}
                        assign(place: p1, value: e15) {}
                        }
                    block k2 {
                        return(value: e10) {}
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

/// The review's frozen direct-HIR rejection leg (p02): the stale view is read inside a nested loop.
const P02_ATOM: &str = r#"
hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m_src {
        path: "r7"
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
                    local l2 t_i32 { name: "j"; role: binding; mutable: true }
                    local l3 t_Str { name: "s"; role: binding; mutable: false }
                    }
                places {
                    place p0 local { local: l2; type: t_i32; access: mutable }
                    place p1 local { local: l2; type: t_i32; access: mutable }
                    place p2 local { local: l0; type: t_str; access: mutable }
                    place p3 local { local: l1; type: t_i32; access: mutable }
                    }
                expressions {
                    expr e0 str_lit { type: t_str; text: "" }
                    expr e1 constant { type: t_i32; value: 0i32 }
                    expr e2 constant { type: t_i32; value: 0i32 }
                    expr e3 read_local { type: t_i32; local: l1 }
                    expr e4 constant { type: t_i32; value: 2i32 }
                    expr e5 binary { type: t_bool; op: lt_i32; lhs: e3; rhs: e4 }
                    expr e6 str_lit { type: t_Str; text: "abc" }
                    expr e7 constant { type: t_i32; value: 0i32 }
                    expr e8 read_local { type: t_i32; local: l2 }
                    expr e9 constant { type: t_i32; value: 1i32 }
                    expr e10 binary { type: t_bool; op: lt_i32; lhs: e8; rhs: e9 }
                    expr e11 read_local { type: t_i32; local: l1 }
                    expr e12 constant { type: t_i32; value: 0i32 }
                    expr e13 binary { type: t_bool; op: gt_i32; lhs: e11; rhs: e12 }
                    expr e14 read_local { type: t_str; local: l0 }
                    expr e15 text_len { type: t_i32; operand: e14 }
                    expr e16 read_local { type: t_i32; local: l2 }
                    expr e17 constant { type: t_i32; value: 1i32 }
                    expr e18 binary { type: t_i32; op: add_i32; overflow: trap; lhs: e16; rhs: e17 }
                    expr e19 read_local { type: t_Str; local: l3 }
                    expr e20 text_to_borrowed { type: t_str; operand: e19 }
                    expr e21 read_local { type: t_i32; local: l1 }
                    expr e22 constant { type: t_i32; value: 1i32 }
                    expr e23 binary { type: t_i32; op: add_i32; overflow: trap; lhs: e21; rhs: e22 }
                    expr e24 constant { type: t_i32; value: 0i32 }
                    }
                blocks {
                    block k0 {
                        let(local: l0, value: e0) {}
                        let(local: l1, value: e1) {}
                        let(local: l2, value: e2) {}
                        loop(id: q0, body: k7) {}
                        return(value: e24) {}
                        }
                    block k1 {
                        let(local: l3, value: e6) {}
                        assign(place: p0, value: e7) {}
                        loop(id: q1, body: k5) {}
                        assign(place: p2, value: e20) {}
                        assign(place: p3, value: e23) {}
                        }
                    block k2 {
                        if(cond: e13, then: k3) {}
                        assign(place: p1, value: e18) {}
                        }
                    block k3 {
                        return(value: e15) {}
                        }
                    block k4 {
                        break(target: q1) {}
                        }
                    block k5 {
                        if(cond: e10, then: k2, else: k4) {}
                        }
                    block k6 {
                        break(target: q0) {}
                        }
                    block k7 {
                        if(cond: e5, then: k1, else: k6) {}
                        }
                    }
                }
            }
        }
    }
"#;
