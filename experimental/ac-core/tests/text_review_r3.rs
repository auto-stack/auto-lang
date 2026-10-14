//! PLAN-770 r3 review replay set (review-r3 findings R3-01..R3-05): the
//! frozen counterexamples q01..q05 and controls c01/c02 from
//! `docs/plans/attachments/770/review-r3/`, made permanent. Same policy as
//! text_review_r2: source legs as real PEs (exact trace, single free, no
//! FREE-INVALID), rejection cases on both channels, HIR legs via
//! programmatic-bundle mutation where the review used one.

#![cfg(windows)]

mod common;

use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::descriptor;
use auto_ac_prototype::hir::{self, HirDomain};
use auto_ac_prototype::source::compile_text_source;
use auto_ac_prototype::verify;
use common::run_deadlined;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const TEXT: &str = "auto.source.text.draft";
const TEXT_CAP: &str = "ac.runtime.text.v1";

/// Owned-Str nominal type test (builtin shorthand or named t_Str ref —
/// the source adapter emits named types).
fn is_owned_ty(u: &hir::TypeUse, m: &hir::Module) -> bool {
    match u {
        hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => true,
        hir::TypeUse::Ref(tid) => matches!(
            m.types.get(tid.0 as usize).map(|t| &t.kind),
            Some(hir::TypeKind::StrOwned)
        ),
        _ => false,
    }
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac770-review-r3").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn check_err(case: &str, src: &str, code: &str) {
    match compile_text_source(src, "r3.at", &[]) {
        Ok(_) => panic!("{case}: expected rejection with `{code}`, got OK"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn check_ok(case: &str, src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_text_source(src, "r3.at", &[])
        .unwrap_or_else(|ds| panic!("{case}: expected OK, got {}", render(&ds)))
}

fn render(ds: &[Diagnostic]) -> String {
    ds.iter().map(|d| d.render("r3.at", "")).collect()
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

fn hir_of(src: &str) -> hir::Bundle {
    let compiled = check_ok("hir-leg", src);
    descriptor::bind(auto_ac_prototype::text_descriptor(), &compiled.atom_text)
        .expect("hir leg bind")
}

fn hir_verify_err(case: &str, bundle: hir::Bundle, code: &str) {
    match verify::verify_text(bundle) {
        Ok(_) => panic!("{case}: HIR leg expected rejection `{code}`, got Checked"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn no_double_free(case: &str, run: &common::DeadlinedRun) {
    assert!(
        !run.stderr.contains("FREE-INVALID"),
        "{case}: FREE-INVALID on stderr:\n{}",
        run.stderr
    );
}

// -- R3-01: join responsibility from ACTUAL arm states -------------------------

#[test]
fn r3_q01_nested_unreachable_move_is_not_a_transfer() {
    // the inner constant-false arm never executes: the caller keeps x and
    // frees it exactly once at return
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    let x Str = \"abc\"\n    if 1 == 1 {\n        if 1 == 0 { let n int = consume(x.move) }\n    }\n    return 0\n}\n";
    let run = pe("q01", src);
    assert_eq!(run.code, 0);
    assert_eq!(
        run.stderr, "LIT len=3\nLIT id=1\nFREE id=1\n",
        "no reachable transfer happened — the caller owns and frees x once"
    );
    no_double_free("q01", &run);
}

#[test]
fn r3_q04_drop_in_reachable_arm_ends_responsibility() {
    // programmatic Drop in a constant-true arm: the Drop consumed the
    // buffer there — the join must NOT schedule the parent's free again
    // (build the bundle from the legal source, then append the Drop)
    let src = "fn main() int {\n    let x Str = \"abc\"\n    let n int = x.len()\n    if 1 == 1 { }\n    return n\n}\n";
    let mut b = hir_of(src);
    let m = &mut b.modules[0];
    // main is the body owning the only Str local (x); local ids are
    // body-scoped, so capture (body, local) together
    let mut found: Option<(usize, usize)> = None;
    for (bi, body) in m.bodies.iter().enumerate() {
        for (li, l) in body.locals.iter().enumerate() {
            if is_owned_ty(&l.ty, m) {
                assert!(found.is_none(), "fixture: exactly one Str local");
                found = Some((bi, li));
            }
        }
    }
    let (main_bi, x_local) = found.expect("x local");
    // find the then-arm block of the if and append Drop(x)
    let then_arms: Vec<u32> = m.bodies[main_bi]
        .blocks
        .iter()
        .flat_map(|blk| blk.stmts.iter())
        .filter_map(|st| match st {
            hir::Stmt::If { then, .. } => Some(then.0),
            _ => None,
        })
        .collect();
    assert_eq!(then_arms.len(), 1, "fixture: exactly one if");
    m.bodies[main_bi].blocks[then_arms[0] as usize]
        .stmts
        .push(hir::Stmt::Drop {
            local: hir::LocalId(x_local as u32),
        });
    let checked = verify::verify_text(b).expect("drop in a reachable arm is legal");
    assert_eq!(checked.domain(), HirDomain::Text);
    // and the built PE frees exactly once (at the Drop), never twice
    let dir = temp_dir("q04");
    let atom = dir.join("q04.atom");
    std::fs::write(&atom, "").unwrap();
    // render via the checked bundle is not exposed; rebuild from the
    // ORIGINAL source path plus a runtime check instead: run the source
    // variant with an explicit drop is impossible in the source language,
    // so assert the plan directly: the Drop position owns the only free
    let plan = checked.text_resources().expect("plan");
    let bp = plan
        .bodies
        .values()
        .find(|bp| !bp.owned.is_empty())
        .expect("main plan");
    let frees: Vec<&Vec<u32>> = bp.exit_frees.values().collect();
    // exactly one scheduled free position, containing only x
    assert_eq!(frees.len(), 1, "exactly one free site: {frees:?}");
    assert_eq!(frees[0].as_slice(), &[x_local as u32]);
}

#[test]
fn r3_c01_unreachable_move_then_legal_read() {
    // the constant-false arm never moves; reading x afterwards is legal
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    let x Str = \"abc\"\n    if 1 == 0 { let n int = consume(x.move) }\n    return x.len()\n}\n";
    let run = pe("c01", src);
    assert_eq!(run.code, 3, "x is still owned and readable");
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n"
    );
    no_double_free("c01", &run);
}

#[test]
fn r3_dynamic_one_sided_still_rejected() {
    // the r2 rule survives the actual-state join: a maybe-transfer has no
    // consistent static responsibility
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    let x Str = \"abc\"\n    var t int = 0\n    if t < 1 { let n int = consume(x.move) }\n    return 0\n}\n";
    check_err("r3-dyn", src, "verify.ownership-required");
}

// -- R3-02: view-parameter alias identity --------------------------------------

#[test]
fn r3_q02_same_view_param_into_mut_and_view_rejected() {
    let src = "fn both(mut a str, b str) int { return a.len() + b.len() }\nfn relay(s str) int { return both(s, s) }\nfn main() int {\n    let x Str = \"abc\"\n    return relay(x.to(str))\n}\n";
    check_err("q02", src, "verify.mut-conflict");
    // c02 control: two DIFFERENT borrowed sources stay legal
    let c02 = "fn both(mut a str, b str) int { return a.len() + b.len() }\nfn relay2(s str, t str) int { return both(s, t) }\nfn main() int {\n    let x Str = \"abc\"\n    let y Str = \"d\"\n    return relay2(x.to(str), y.to(str))\n}\n";
    let run = pe("c02", c02);
    assert_eq!(run.code, 4);
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nLIT len=1\nLIT id=2\nTEXT-SCALARS n=3\nTEXT-SCALARS n=1\nFREE id=2\nFREE id=1\n",
        "independent borrowed sources; LIFO frees"
    );
    no_double_free("c02", &run);
}

// -- R3-03: full-statement-tail temp lifetime -----------------------------------

#[test]
fn r3_q03_temp_outlives_all_statement_effects() {
    // the mark (evaluated AFTER the len in the add's rhs) must precede
    // the concat temp's FREE — the temp lives to the statement's full tail
    let src = "fn main() int { return (\"a\" + \"b\").len() + mark_a() }\n";
    let dir = temp_dir("q03");
    let src_path = dir.join("q.at");
    std::fs::write(&src_path, src).unwrap();
    let exe = dir.join("q.exe");
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
    assert_eq!(run.code, 3);
    assert_eq!(
        run.stderr, "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\naFREE id=1\n",
        "the mark's `a` (a later statement effect) precedes the temp's FREE"
    );
    no_double_free("q03", &run);
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

#[test]
fn r3_q03_chain_4096_statement_tail_probed_frame() {
    // the statement-tail model keeps 4094 temporaries live — the inline
    // stack probe makes the >4KB frame safe; every temp freed exactly
    // once at the let's tail, the root by the owning let
    let src = format!(
        "fn main() int {{\n    let s Str = {}\n    return s.len()\n}}\n",
        vec!["\"a\""; 4096].join(" + ")
    );
    let run = pe("q03-chain", &src);
    assert_eq!(run.code, 4096);
    let lines: Vec<&str> = run.stderr.lines().collect();
    let concats = lines.iter().filter(|l| l.starts_with("CONCAT id")).count();
    let frees = lines.iter().filter(|l| l.starts_with("FREE id")).count();
    assert_eq!(concats, 4095);
    assert_eq!(frees, 4095);
    assert_eq!(lines.last().copied(), Some("FREE id=4095"));
    no_double_free("q03-chain", &run);
}

// -- R3-04: derived runtime need covers pure-cleanup obligations -----------------

#[test]
fn r3_q05_cleanup_only_module_requires_the_runtime() {
    // an EMITTED function whose only runtime use is freeing its move-in
    // parameter at exit must be gated like a text-op function
    let src = "fn unused(move s Str) int { return 0 }\nfn main() int { return 0 }\n";
    let mut b = hir_of(src);
    // the source adapter declares; the programmatic bundle with the
    // declaration STRIPPED must be rejected at the front door
    assert!(
        b.requires.iter().any(|r| r == TEXT_CAP),
        "adapter must declare for cleanup-only functions too"
    );
    b.requires.clear();
    hir_verify_err("q05", b, "capability.missing");
}

// -- R3-05: the need scan is total; illegal references reject controlled ---------

#[test]
fn r3_q06_out_of_range_binary_operand_rejects_not_panics() {
    let src2 = "fn main() int {\n    let a str = \"ab\"\n    if a == \"cd\" { return 1 }\n    return 0\n}\n";
    let mut b2 = hir_of(src2);
    let m2 = &mut b2.modules[0];
    let mut hit = 0usize;
    for body in &mut m2.bodies {
        for e in &mut body.exprs {
            if let hir::ExprKind::Binary {
                op: hir::BinOp::Eq | hir::BinOp::Ne,
                lhs,
                ..
            } = &mut e.kind
            {
                *lhs = hir::ExprId(u32::MAX);
                hit += 1;
            }
        }
    }
    assert_eq!(hit, 1, "fixture: exactly one text eq");
    hir_verify_err("q06-lhs", b2, "verify.bad-reference");

    // same for the operand of a text op and a call callee
    let src3 = "fn main() int {\n    let a str = \"abc\"\n    return a.len()\n}\n";
    let mut b3 = hir_of(src3);
    let m3 = &mut b3.modules[0];
    let mut hit3 = 0usize;
    for body in &mut m3.bodies {
        for e in &mut body.exprs {
            if let hir::ExprKind::TextLen { operand } = &mut e.kind {
                *operand = hir::ExprId(u32::MAX);
                hit3 += 1;
            }
        }
    }
    assert_eq!(hit3, 1, "fixture: one len");
    hir_verify_err("q06-operand", b3, "verify.bad-reference");
}
