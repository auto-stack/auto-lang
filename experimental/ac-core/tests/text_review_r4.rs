//! PLAN-770 r4 review replay set (review-r4 findings P770-R4-01..05): the
//! frozen counterexamples p01..p11 and controls c01..c04 from
//! `docs/plans/attachments/770/review-r4/`, made permanent, plus the
//! composed oracles the fix requirements demand (a statement-later mark
//! still precedes a short-circuit temp's tail free; a loop-body owner's
//! backing is not readable after the loop; wrapper exclusivity propagates
//! through two levels). Same policy as r2/r3: legal programs run as real
//! PEs on BOTH channels (source and direct HIR) with exact traces and no
//! FREE-INVALID; rejections are asserted with their frozen diagnostic
//! codes on both channels.

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
    let dir = std::env::temp_dir().join("ac770-review-r4").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn check_err(case: &str, src: &str, code: &str) {
    match compile_text_source(src, "r4.at", &[]) {
        Ok(_) => panic!("{case}: expected rejection with `{code}`, got OK"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn check_ok(case: &str, src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_text_source(src, "r4.at", &[])
        .unwrap_or_else(|ds| panic!("{case}: expected OK, got {}", render(&ds)))
}

fn render(ds: &[Diagnostic]) -> String {
    ds.iter().map(|d| d.render("r4.at", "")).collect()
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

/// Direct-HIR leg: the same program through `build` from the emitted atom
/// (the verify_text front door is the single gate for both channels).
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
/// at the source channel, so their bundles cannot be produced by
/// compiling the illegal source — the atom is the honest HIR-channel
/// input, bound and verified through the same `verify_text` front door.
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

/// The review's frozen direct-HIR counterexample for R4-01 (p10): the
/// view `v` (l0) is backed by the arm-local owner `x` (l1), read after
/// the arm's block exit.
const P10_ATOM: &str = r#"hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
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
                    local l1 t_Str { name: "x"; role: binding; mutable: false }
                    }
                places {
                    place p0 local { local: l0; type: t_str; access: mutable }
                    }
                expressions {
                    expr e0 str_lit { type: t_str; text: "" }
                    expr e1 constant { type: t_i32; value: 0i32 }
                    expr e2 constant { type: t_i32; value: 1i32 }
                    expr e3 binary { type: t_bool; op: lt_i32; lhs: e1; rhs: e2 }
                    expr e4 str_lit { type: t_Str; text: "abc" }
                    expr e5 read_local { type: t_Str; local: l1 }
                    expr e6 text_to_borrowed { type: t_str; operand: e5 }
                    expr e7 read_local { type: t_str; local: l0 }
                    expr e8 text_len { type: t_i32; operand: e7 }
                    }
                blocks {
                    block k0 {
                        let(local: l0, value: e0) {}
                        if(cond: e3, then: k1) {}
                        return(value: e8) {}
                        }
                    block k1 {
                        let(local: l1, value: e4) {}
                        assign(place: p0, value: e6) {}
                        }
                    }
                }
            }
        }
    }
"#;

/// The review's frozen direct-HIR counterexample for R4-05 (p07): main
/// passes two views of the SAME local into relay, which forwards them
/// into `both(mut a, b)` — the exclusivity pair rides relay's signature.
const P07_ATOM: &str = r#"hir(schema: "auto.hir.text.draft", revision: 1u32, profile: "text-i32-draft") {
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
                    expr e0 read_local { type: t_str; local: l0 }
                    expr e1 read_local { type: t_str; local: l1 }
                    expr e2 call { type: t_i32; callee: d_both; eval_args: [e0, e1]; bindings: [{ param: 0u32, arg: 0u32 }, { param: 1u32, arg: 1u32 }] }
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
                    expr e5 call { type: t_i32; callee: d_relay; eval_args: [e2, e4]; bindings: [{ param: 0u32, arg: 0u32 }, { param: 1u32, arg: 1u32 }] }
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

// -- R4-02: short-circuit RHS owned temporaries follow the executed path ----------

/// Both legs of every case in this group: the two channels agree because
/// the fix is in the shared verifier/lowering, not per-channel.
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

#[test]
fn r4_p01_skipped_rhs_temp_no_alloc_no_free() {
    // `false &&` — the RHS concat never executes: no allocation AND no
    // free on the skipped path (previously the tail free used an SSA value
    // the skip predecessor never defined — backend verifier error)
    let src = "fn main() int { let ok bool = false && (\"a\"+\"b\").len()>0; if ok { return 1 }; return 0 }";
    both_legs("p01", src, 0, "");
}

#[test]
fn r4_p02_executed_rhs_temp_one_alloc_one_free() {
    // `true &&` — the executed path allocates once and frees once, at the
    // statement tail
    let src = "fn main() int { let ok bool = true && (\"a\"+\"b\").len()>0; if ok { return 1 }; return 0 }";
    both_legs(
        "p02",
        src,
        1,
        "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\nFREE id=1\n",
    );
}

#[test]
fn r4_p03_dynamic_lhs_skips_rhs_temp() {
    // runtime-false lhs: same skipped-path contract under a dynamic condition
    let src = "fn main() int { var i int=0; let ok bool = i>0 && (\"a\"+\"b\").len()>0; if ok { return 1 }; return 0 }";
    both_legs("p03", src, 0, "");
}

#[test]
fn r4_p04_lhs_rhs_temps_lifo_statement_tail() {
    // temps on BOTH sides of a dynamic `&&`: both execute, LIFO tail frees
    // (id=2 then id=1), lens in left-to-right order
    let src = "fn main() int { let ok bool = (\"c\"+\"d\").len()>0 && (\"a\"+\"b\").len()>0; if ok { return 1 }; return 0 }";
    both_legs(
        "p04",
        src,
        1,
        "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\nCONCAT a=1 b=1\nCONCAT id=2\nTEXT-SCALARS n=2\nFREE id=2\nFREE id=1\n",
    );
}

#[test]
fn r4_p11_or_true_skips_rhs_temp() {
    // `true ||` — the mirrored skip shape
    let src = "fn main() int { let ok bool = true || (\"a\"+\"b\").len()>0; if ok { return 1 }; return 0 }";
    both_legs("p11", src, 1, "");
}

#[test]
fn r4_p02_later_mark_precedes_shortcircuit_temp_free() {
    // composed oracle for the fix requirement "完整语句后续 mark 先于
    // FREE": a mark evaluated AFTER the logical (second call argument) is
    // a later statement effect — the RHS temp's free stays at the
    // statement tail, after it
    let src = "fn pick(b bool, n int) int { if b { return n }; return 0 }\nfn main() int { return pick(1==1 && (\"a\"+\"b\").len()>0, mark_a()) }\n";
    let dir = temp_dir("p02-mark");
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
    assert_eq!(run.code, 1, "pick(true, mark_a()=1)");
    assert_eq!(
        run.stderr, "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\naFREE id=1\n",
        "the mark `a` (later statement effect) precedes the RHS temp's tail FREE"
    );
    no_double_free("p02-mark", &run);
}

#[test]
fn r4_c01_borrowed_shortcircuit_stays_legal() {
    // the control isolating the trigger to OWNED temps: a borrowed-literal
    // RHS was always legal and stays legal
    let src =
        "fn main() int { let ok bool=false && \"abc\".len()>0; if ok { return 1 }; return 0 }";
    both_legs("c01", src, 0, "");
}

// -- R4-03: a terminating arm does not pollute the continuation state ------------

#[test]
fn r4_p05_returning_move_arm_keeps_later_read_legal() {
    // the `return consume(x.move)` arm TERMINATES: it keeps its transfer
    // but contributes no dead/backing at the join — the fall-through read
    // of x stays legal with a unique FREE
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int { let x Str=\"abc\"; var i int=0; if i>0 { return consume(x.move) }; return x.len() }";
    both_legs(
        "p05",
        src,
        3,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
    );
}

#[test]
fn r4_p06_nested_returning_move_arm_same_verdict() {
    // the same shape one level deeper: the inner constant-true `if` wraps
    // the returning arm; nothing leaks to the outer join either
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int { let x Str=\"abc\"; var i int=0; if i>0 { if true { return consume(x.move) } }; return x.len() }";
    both_legs(
        "p06",
        src,
        3,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
    );
}

// -- R4-04: a branch's OWN owned local is not a cross-arm responsibility ----------

#[test]
fn r4_p08_dynamic_branch_local_owned_legal() {
    // dynamic single arm with its own owned local: constructed, read and
    // freed inside the arm — never a join duty of the parent
    let src =
        "fn main() int { var i int=0; if i==0 { let s Str=\"abc\"; let n int=s.len() }; return 0 }";
    both_legs(
        "p08",
        src,
        0,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
    );
}

#[test]
fn r4_p09_both_branch_locals_legal() {
    // both arms declare INDEPENDENT owned locals — two legitimate
    // constructions, one executed at runtime
    let src = "fn main() int { var i int=0; if i==0 { let s Str=\"abc\"; let n int=s.len() } else { let t Str=\"d\"; let n int=t.len() }; return 0 }";
    both_legs(
        "p09",
        src,
        0,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
    );
}

// -- R4-01: block-exit lifetimes enter the state; a backing owner that
// exited its scope is not a legal backing ------------------------------

#[test]
fn r4_p10_borrowed_scope_escape_rejected() {
    // `let x` lives inside the arm; the view stored into `v` outlives x's
    // block exit — use-after-free at runtime, must reject at the front
    // door on BOTH channels (HIR leg: the frozen atom from the review)
    let src = "fn main() int { var v str=\"\"; if true { let x Str=\"abc\"; v=x.to(str) }; return v.len() }";
    check_err("p10", src, "verify.owned-scope-escape");
    atom_verify_err("p10", P10_ATOM, "verify.owned-scope-escape");
}

#[test]
fn r4_c04_live_owner_borrow_stays_legal() {
    // the live-owner control: x is owned by the ENCLOSING block, still
    // alive where `v` reads it — legal, one FREE
    let src = "fn main() int { let x Str=\"abc\"; var v str=\"\"; if true { v=x.to(str) }; return v.len() }";
    both_legs(
        "c04",
        src,
        3,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
    );
}

#[test]
fn r4_loop_body_owner_backing_not_readable_after_loop() {
    // composed oracle for the "条件/循环出口" requirement: the backing
    // owner is loop-body-scoped — freed on every exit path (iteration end
    // or break), so reading the view after the loop is a scope escape
    let src = "fn main() int {\n    var v str = \"\"\n    var i int = 0\n    for i < 2 {\n        let s Str = \"abc\"\n        v = s.to(str)\n        i = i + 1\n    }\n    return v.len()\n}\n";
    check_err("loop-exit", src, "verify.owned-scope-escape");
}

// -- R4-05: pairwise parameter exclusivity propagates to the call boundary --------

#[test]
fn r4_p07_wrapper_same_source_mut_view_rejected() {
    // relay forwards its two view parameters into `both(mut a, b)` — the
    // exclusivity pair rides relay's signature, so the original boundary
    // (main passing the same x twice) rejects
    let src = "fn both(mut a str,b str) int { return a.len()+b.len() }\nfn relay(a str,b str) int { return both(a,b) }\nfn main() int { let x Str=\"abc\"; return relay(x.to(str),x.to(str)) }";
    check_err("p07", src, "verify.mut-conflict");
    atom_verify_err("p07", P07_ATOM, "verify.mut-conflict");
}

#[test]
fn r4_p07_two_level_wrapper_propagation_rejected() {
    // the same constraint through TWO wrapper levels — the fixpoint
    // composes instead of stopping at the first boundary
    let src = "fn both(mut a str,b str) int { return a.len()+b.len() }\nfn relay(a str,b str) int { return both(a,b) }\nfn relay2(a str,b str) int { return relay(a,b) }\nfn main() int { let x Str=\"abc\"; return relay2(x.to(str),x.to(str)) }";
    check_err("p07-2lvl", src, "verify.mut-conflict");
}

#[test]
fn r4_c02_wrapper_independent_sources_stay_legal() {
    // the positive control: distinct sources through the wrapper stay
    // legal — read-only two-parameter calls are NOT blanket-disabled
    let src = "fn both(mut a str,b str) int { return a.len()+b.len() }\nfn relay(a str,b str) int { return both(a,b) }\nfn main() int { let x Str=\"abc\"; let y Str=\"d\"; return relay(x.to(str),y.to(str)) }";
    both_legs(
        "c02",
        src,
        4,
        "LIT len=3\nLIT id=1\nLIT len=1\nLIT id=2\nTEXT-SCALARS n=3\nTEXT-SCALARS n=1\nFREE id=2\nFREE id=1\n",
    );
}
