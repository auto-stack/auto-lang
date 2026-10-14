//! PLAN-770 r5 review replay set (review-r5 findings P770-R5-01/02): the
//! frozen counterexamples p01..p06 and controls c01..c03 from
//! `docs/plans/attachments/770/review-r5/`, made permanent, plus composed
//! oracles for the fix requirements (a mark after the whole logical still
//! precedes the LHS temp's tail free; double-arm break/continue with
//! arm-local owners). Same policy as r2/r3/r4: legal programs run as real
//! PEs on BOTH channels with exact traces and no FREE-INVALID/FREE leak;
//! rejections assert their frozen diagnostic codes on both channels
//! (source and the frozen atom through the same `verify_text` front door).

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
    let dir = std::env::temp_dir().join("ac770-review-r5").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn check_err(case: &str, src: &str, code: &str) {
    match compile_text_source(src, "r5.at", &[]) {
        Ok(_) => panic!("{case}: expected rejection with `{code}`, got OK"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn check_ok(case: &str, src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_text_source(src, "r5.at", &[])
        .unwrap_or_else(|ds| panic!("{case}: expected OK, got {}", render(&ds)))
}

fn render(ds: &[Diagnostic]) -> String {
    ds.iter().map(|d| d.render("r5.at", "")).collect()
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

/// Direct-HIR leg: the same legal program through `build` from the
/// compiled atom (the verify_text front door gates both channels).
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

/// Direct-HIR rejection leg from a FROZEN atom input (the review's .atom
/// counterexamples, embedded byte-for-byte): these programs are rejected
/// at the source channel, so the atom is the honest HIR-channel input,
/// bound and verified through the same `verify_text` front door.
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

/// Both legs of a legal case: identical exit code and trace on the source
/// and direct-HIR channels.
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

/// Build the trace marks as a plain COFF object (see text_native.rs).
fn build_trace_marks_obj() -> std::path::PathBuf {
    use std::sync::OnceLock;
    static CACHE: OnceLock<std::path::PathBuf> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let src = root.join("test-support/marks.rs");
            let out = root.join("test-support/target/marks-text.obj");
            let mut cmd = Command::new("rustc");
            cmd.args([
                "--edition",
                "2021",
                "--crate-type",
                "lib",
                "--emit=obj",
                "-C",
                "panic=abort",
                "-o",
            ])
            .arg(&out)
            .arg(&src);
            let r = run_deadlined(cmd, Duration::from_secs(120)).expect("spawn rustc");
            assert_eq!(r.code, 0, "marks build failed:\n{}", r.stderr);
            out
        })
        .clone()
}

// -- R5-01: LHS-owned temps keep their real pointers when the RHS is skipped -----

#[test]
fn r5_p01_lhs_and_skip_frees_lhs_temp() {
    // `&&` with a false LHS skips the RHS; the LHS concat executed on
    // EVERY path — its temp must be freed unconditionally at the
    // statement tail (r5's counterexample: the FREE disappeared because
    // the LHS temp was misclassified as a skipped RHS temp)
    let src = "fn main() int { let ok bool=(\"a\"+\"b\").len()==0 && \"c\".len()>0; if ok { return 1 }; return 0 }";
    both_legs(
        "p01",
        src,
        0,
        "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\nFREE id=1\n",
    );
}

#[test]
fn r5_p02_lhs_or_skip_frees_lhs_temp() {
    // the mirrored `||` shape: a true LHS skips the RHS, the LHS temp
    // still frees at the tail
    let src = "fn main() int { let ok bool=(\"a\"+\"b\").len()>0 || (\"c\"+\"d\").len()>0; if ok { return 1 }; return 0 }";
    both_legs(
        "p02",
        src,
        1,
        "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\nFREE id=1\n",
    );
}

#[test]
fn r5_p03_nested_lhs_skip_frees_all_lhs_temps() {
    // nested logical: the OUTER lhs is itself a logical whose lhs and rhs
    // both allocated (id=1, id=2); the outer && then skips its own RHS.
    // Both lhs temps free at the tail — id=2 (an inner join param, live
    // here) under its null guard, id=1 unconditionally, LIFO
    let src = "fn main() int { let ok bool=((\"a\"+\"b\").len()>0 && (\"c\"+\"d\").len()==0) && (\"e\"+\"f\").len()>0; if ok { return 1 }; return 0 }";
    both_legs(
        "p03",
        src,
        0,
        "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\nCONCAT a=1 b=1\nCONCAT id=2\nTEXT-SCALARS n=2\nFREE id=2\nFREE id=1\n",
    );
}

#[test]
fn r5_c01_both_sides_execute_lifo_tail() {
    // the control: both sides execute — one alloc + one free per temp,
    // LIFO at the statement tail
    let src = "fn main() int { let ok bool=(\"a\"+\"b\").len()>0 && (\"c\"+\"d\").len()>0; if ok { return 1 }; return 0 }";
    both_legs(
        "c01",
        src,
        1,
        "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\nCONCAT a=1 b=1\nCONCAT id=2\nTEXT-SCALARS n=2\nFREE id=2\nFREE id=1\n",
    );
}

#[test]
fn r5_p01_mark_after_logical_precedes_lhs_temp_free() {
    // composed oracle: a mark evaluated AFTER the whole logical (second
    // call argument) still precedes the LHS temp's tail FREE
    let src = "fn pick(b bool, n int) int { if b { return n }; return 0 }\nfn main() int { return pick((\"a\"+\"b\").len()==0 && \"c\".len()>0, mark_a()) }\n";
    let dir = temp_dir("p01-mark");
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
        "--capability",
        "hir.test.trace.v1",
        "--support-lib",
    ])
    .arg(common::text_runtime_lib())
    .arg("--support-lib")
    .arg(build_trace_marks_obj())
    .arg("--output")
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 0, "build failed:\n{}", b.stderr);
    let run = run_deadlined(Command::new(&exe), Duration::from_secs(60)).expect("run");
    assert_eq!(run.code, 0, "pick(false, mark_a()=1)");
    assert_eq!(
        run.stderr, "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\naFREE id=1\n",
        "the mark `a` (later statement effect) precedes the LHS temp's tail FREE"
    );
    no_double_free("p01-mark", &run);
}

// -- R5-02: break/continue target-exit states reach the loop's continuation ------

#[test]
fn r5_p04_break_borrow_escape_rejected() {
    // the breaking arm's owner s is freed at the break; the view stored
    // into v outlives it — the TARGET exit state must reach the loop's
    // after-state so the post-loop read is a front-door rejection (the
    // if join correctly drops the terminating arm, r4 R4-03)
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<1 { if true { let s Str=\"abc\"; v=s.to(str); break } }; return v.len() }";
    check_err("p04", src, "verify.owned-scope-escape");
    atom_verify_err("p04", P04_ATOM, "verify.owned-scope-escape");
}

#[test]
fn r5_p05_continue_borrow_escape_rejected() {
    // the same shape through continue (re-entry): the freed loop-scoped
    // owner's view is still unreadable past the loop
    let src = "fn main() int { var v str=\"\"; var i int=0; for i<1 { if true { let s Str=\"abc\"; v=s.to(str); i=i+1; continue } }; return v.len() }";
    check_err("p05", src, "verify.owned-scope-escape");
    atom_verify_err("p05", P05_ATOM, "verify.owned-scope-escape");
}

#[test]
fn r5_c02_loop_live_owner_through_break_legal() {
    // the control: the backing owner x is LIVE OUTSIDE the loop and is
    // not freed at the break — the read stays legal with a unique FREE
    let src = "fn main() int { let x Str=\"abc\"; var v str=\"\"; var i int=0; for i<1 { if true { v=x.to(str); break } }; return v.len() }";
    both_legs(
        "c02",
        src,
        3,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
    );
}

#[test]
fn r5_double_arm_break_continue_backing_rejected() {
    // composed oracle: BOTH arms terminate (break / continue), each
    // setting the view from its OWN arm-local owner — every path to the
    // post-loop read crosses a freed backing owner
    let src = "fn main() int {\n    var v str = \"\"\n    var i int = 0\n    for i < 2 {\n        if i == 0 {\n            let s Str = \"abc\"\n            v = s.to(str)\n            break\n        } else {\n            let t Str = \"d\"\n            v = t.to(str)\n            continue\n        }\n    }\n    return v.len()\n}\n";
    check_err("double-arm", src, "verify.owned-scope-escape");
}

// -- r5 side findings kept: named-argument wrapper exclusivity --------------------

#[test]
fn r5_p06_wrapper_named_alias_still_rejected() {
    // named arguments route each view to a different parameter index —
    // the exclusivity pair check follows the BINDING, not the written
    // order; the same source still rejects
    let src = "fn both(mut a str,b str) int { return a.len()+b.len() } fn relay(a str,b str) int { return both(b:b,a:a) } fn main() int { let x Str=\"abc\"; return relay(b:x.to(str),a:x.to(str)) }";
    check_err("p06", src, "verify.mut-conflict");
    atom_verify_err("p06", P06_ATOM, "verify.mut-conflict");
}

#[test]
fn r5_c03_wrapper_named_distinct_stays_legal() {
    // named arguments with distinct sources stay legal — evaluation
    // follows parameter order (a=x first), frees LIFO at main's tail
    let src = "fn both(mut a str,b str) int { return a.len()+b.len() } fn relay(a str,b str) int { return both(b:b,a:a) } fn main() int { let x Str=\"abc\"; let y Str=\"d\"; return relay(b:y.to(str),a:x.to(str)) }";
    both_legs(
        "c03",
        src,
        4,
        "LIT len=3\nLIT id=1\nLIT len=1\nLIT id=2\nTEXT-SCALARS n=3\nTEXT-SCALARS n=1\nFREE id=2\nFREE id=1\n",
    );
}

/// The review's frozen direct-HIR counterexample for R5-02 (p04): a
/// break inside an if arm frees the arm-local owner `s` (l2) whose view
/// is stored into `v` (l0), read after the loop.
const P04_ATOM: &str = r#"hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m_src {
        path: "r4"
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
                    }
                expressions {
                    expr e0 str_lit { type: t_str; text: "" }
                    expr e1 constant { type: t_i32; value: 0i32 }
                    expr e2 read_local { type: t_i32; local: l1 }
                    expr e3 constant { type: t_i32; value: 1i32 }
                    expr e4 binary { type: t_bool; op: lt_i32; lhs: e2; rhs: e3 }
                    expr e5 constant { type: t_i32; value: 0i32 }
                    expr e6 constant { type: t_i32; value: 1i32 }
                    expr e7 binary { type: t_bool; op: lt_i32; lhs: e5; rhs: e6 }
                    expr e8 str_lit { type: t_Str; text: "abc" }
                    expr e9 read_local { type: t_Str; local: l2 }
                    expr e10 text_to_borrowed { type: t_str; operand: e9 }
                    expr e11 read_local { type: t_str; local: l0 }
                    expr e12 text_len { type: t_i32; operand: e11 }
                    }
                blocks {
                    block k0 {
                        let(local: l0, value: e0) {}
                        let(local: l1, value: e1) {}
                        loop(id: q0, body: k4) {}
                        return(value: e12) {}
                        }
                    block k1 {
                        if(cond: e7, then: k2) {}
                        }
                    block k2 {
                        let(local: l2, value: e8) {}
                        assign(place: p0, value: e10) {}
                        break(target: q0) {}
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

/// The review's frozen direct-HIR counterexample for R5-02 (p05): the
/// same shape through continue (re-entry).
const P05_ATOM: &str = r#"hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m_src {
        path: "r4"
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
                    expr e3 constant { type: t_i32; value: 1i32 }
                    expr e4 binary { type: t_bool; op: lt_i32; lhs: e2; rhs: e3 }
                    expr e5 constant { type: t_i32; value: 0i32 }
                    expr e6 constant { type: t_i32; value: 1i32 }
                    expr e7 binary { type: t_bool; op: lt_i32; lhs: e5; rhs: e6 }
                    expr e8 str_lit { type: t_Str; text: "abc" }
                    expr e9 read_local { type: t_Str; local: l2 }
                    expr e10 text_to_borrowed { type: t_str; operand: e9 }
                    expr e11 read_local { type: t_i32; local: l1 }
                    expr e12 constant { type: t_i32; value: 1i32 }
                    expr e13 binary { type: t_i32; op: add_i32; overflow: trap; lhs: e11; rhs: e12 }
                    expr e14 read_local { type: t_str; local: l0 }
                    expr e15 text_len { type: t_i32; operand: e14 }
                    }
                blocks {
                    block k0 {
                        let(local: l0, value: e0) {}
                        let(local: l1, value: e1) {}
                        loop(id: q0, body: k4) {}
                        return(value: e15) {}
                        }
                    block k1 {
                        if(cond: e7, then: k2) {}
                        }
                    block k2 {
                        let(local: l2, value: e8) {}
                        assign(place: p0, value: e10) {}
                        assign(place: p1, value: e13) {}
                        continue(target: q0) {}
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

/// The review's frozen direct-HIR input for the named-argument wrapper
/// finding (p06): relay routes its two same-source views to `both(mut a,
/// b)` through SWAPPED bindings — the pair check must follow the binding
/// indices, not the written argument order.
const P06_ATOM: &str = r#"hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m_src {
        path: "r4"
        types {
            type t_i32 i32 {}
            type t_bool bool {}
            type t_str str {}
            type t_Str Str {}
            type t_char char {}
            type t_f_both function { params: ["t_str", "t_str"]; result: "t_i32" }
            type t_f_relay function { params: ["t_str", "t_str"]; result: "t_i32" }
            type t_f_main function { params: []; result: "t_i32" }
            }
        declarations {
            function d_both t_f_both { name: "sf_both"; body: "b_both" }
            function d_relay t_f_relay { name: "sf_relay"; body: "b_relay" }
            function d_main t_f_main { name: "sf_main"; body: "b_main" }
            }
        bodies {
            body b_both {
                owner: "d_both"
                entry: "k0"
                locals {
                    local l0 t_str { name: "a"; role: param; index: 0u32; mode: mut }
                    local l1 t_str { name: "b"; role: param; index: 1u32 }
                    }
                expressions {
                    expr e0 read_local { type: t_str; local: l0 }
                    expr e1 text_len { type: t_i32; operand: e0 }
                    expr e2 read_local { type: t_str; local: l1 }
                    expr e3 text_len { type: t_i32; operand: e2 }
                    expr e4 binary { type: t_i32; op: add_i32; overflow: trap; lhs: e1; rhs: e3 }
                    }
                blocks {
                    block k0 {
                        return(value: e4) {}
                        }
                    }
                }
            body b_relay {
                owner: "d_relay"
                entry: "k0"
                locals {
                    local l0 t_str { name: "a"; role: param; index: 0u32 }
                    local l1 t_str { name: "b"; role: param; index: 1u32 }
                    }
                expressions {
                    expr e0 read_local { type: t_str; local: l1 }
                    expr e1 read_local { type: t_str; local: l0 }
                    expr e2 call { type: t_i32; callee: d_both; eval_args: [e0, e1]; bindings: [{ param: 1u32, arg: 0u32 }, { param: 0u32, arg: 1u32 }] }
                    }
                blocks {
                    block k0 {
                        return(value: e2) {}
                        }
                    }
                }
            body b_main {
                owner: "d_main"
                entry: "k0"
                locals {
                    local l0 t_Str { name: "x"; role: binding; mutable: false }
                    }
                expressions {
                    expr e0 str_lit { type: t_Str; text: "abc" }
                    expr e1 read_local { type: t_Str; local: l0 }
                    expr e2 text_to_borrowed { type: t_str; operand: e1 }
                    expr e3 read_local { type: t_Str; local: l0 }
                    expr e4 text_to_borrowed { type: t_str; operand: e3 }
                    expr e5 call { type: t_i32; callee: d_relay; eval_args: [e2, e4]; bindings: [{ param: 1u32, arg: 0u32 }, { param: 0u32, arg: 1u32 }] }
                    }
                blocks {
                    block k0 {
                        let(local: l0, value: e0) {}
                        return(value: e5) {}
                        }
                    }
                }
            }
        }
    }
"#;
