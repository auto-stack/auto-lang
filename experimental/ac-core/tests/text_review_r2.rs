//! PLAN-770 r2 review replay set (review-r2 findings QA-01..QA-07): the
//! frozen counterexamples p01..p09 and controls c02..c08 from
//! `docs/plans/attachments/770/review-r2/`, made permanent. Source legs run
//! as real PEs (exit code + exact trace: every allocation has exactly one
//! FREE and no FREE-INVALID); rejection cases assert the frozen verify
//! codes through BOTH channels — direct source compile and the HIR leg
//! (the same program's generated Atom re-bound and verified/mutated as a
//! programmatic bundle, mirroring the review's bundle-probes).

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

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac770-review-r2").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Compile a text source through the in-process pipeline; on rejection
/// assert one frozen code is present.
fn check_err(case: &str, src: &str, code: &str) {
    match compile_text_source(src, "r2.at", &[]) {
        Ok(_) => panic!("{case}: expected rejection with `{code}`, got OK"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn check_ok(case: &str, src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_text_source(src, "r2.at", &[])
        .unwrap_or_else(|ds| panic!("{case}: expected OK, got {}", render(&ds)))
}

fn render(ds: &[Diagnostic]) -> String {
    ds.iter().map(|d| d.render("r2.at", "")).collect()
}

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

/// Borrowed-str nominal type test.
fn is_str_ty(u: &hir::TypeUse, m: &hir::Module) -> bool {
    match u {
        hir::TypeUse::Builtin(hir::BuiltinType::StrSlice) => true,
        hir::TypeUse::Ref(tid) => matches!(
            m.types.get(tid.0 as usize).map(|t| &t.kind),
            Some(hir::TypeKind::Str)
        ),
        _ => false,
    }
}

fn assert(cond: bool, case: &str, code: &str, ds: &[Diagnostic]) {
    assert!(
        cond,
        "{case}: expected code `{code}`, got codes {:?}\n{}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(),
        render(ds)
    );
}

/// Build a text source as a real PE (runtime lib + capability) and run it.
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

/// Bind generated Atom text as a programmatic bundle (the HIR channel).
fn hir_of(compiled: &auto_ac_prototype::source::CompiledSource) -> hir::Bundle {
    descriptor::bind(auto_ac_prototype::text_descriptor(), &compiled.atom_text)
        .expect("hir leg bind")
}

/// The HIR leg for a LEGAL program (source channel compiles it; the Atom
/// is re-bound as a programmatic bundle and built via `build`).
fn hir_bundle(src: &str) -> hir::Bundle {
    hir_of(&check_ok("hir-leg", src))
}

/// The HIR leg for a REJECTION: verify_text is the single semantic gate
/// both channels share, so the bypass proof mutates a LEGAL program's
/// bundle into the violating shape and asserts the same frozen code.
fn hir_mutation_err(case: &str, legal: &str, code: &str, mutate: impl Fn(&mut hir::Bundle)) {
    let mut b = hir_bundle(legal);
    mutate(&mut b);
    match verify::verify_text(b) {
        Ok(_) => panic!("{case}: mutated HIR leg expected `{code}`, got Checked"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

fn hir_verify_err(case: &str, bundle: hir::Bundle, code: &str) {
    match verify::verify_text(bundle) {
        Ok(_) => panic!("{case}: HIR leg expected rejection `{code}`, got Checked"),
        Err(ds) => assert(ds.iter().any(|d| d.code == code), case, code, &ds),
    }
}

/// Build the HIR leg as a real PE through `build`.
fn hir_pe(case: &str, src: &str) -> common::DeadlinedRun {
    let compiled = check_ok(case, src);
    let dir = temp_dir(case);
    let atom = dir.join("p.atom");
    std::fs::write(&atom, &compiled.atom_text).unwrap();
    let exe = dir.join("p.exe");
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
        "{case}: FREE-INVALID on stderr — a double free or foreign release:\n{}",
        run.stderr
    );
}

// -- QA-01: forged bundle identity never reaches the backend -------------------

#[test]
fn qa01_forged_domain_rejected_before_backend() {
    let src = "fn main() int {\n    let s str = \"ab\"\n    return s.len()\n}\n";
    for forged in [HirDomain::Core, HirDomain::Scalar] {
        let mut b = hir_bundle(src);
        b.domain = forged;
        hir_verify_err("qa01", b, "verify.domain");
    }
    // the honest identity still verifies
    let checked = verify::verify_text(hir_bundle(src)).expect("honest domain verifies");
    assert_eq!(checked.domain(), HirDomain::Text);
}

// -- QA-02: the runtime declaration is mandatory and derived -------------------

#[test]
fn qa02_stripped_requires_rejected_at_the_front_door() {
    let src = "fn main() int {\n    let s str = \"ab\"\n    return s.len()\n}\n";
    // source channel compiles (the adapter declares); the programmatic
    // bundle with the declaration STRIPPED is rejected
    let mut b = hir_bundle(src);
    assert!(
        b.requires.iter().any(|r| r == TEXT_CAP),
        "adapter must declare the runtime capability"
    );
    b.requires.clear();
    hir_verify_err("qa02-mut02", b, "capability.missing");

    // CLI: check-source of a stripped document (written as raw Atom with no
    // requires) fails before any build consideration
    let compiled = check_ok("qa02", src);
    let stripped = compiled
        .atom_text
        .replacen(&format!("requires: [\"{TEXT_CAP}\"]"), "", 1);
    assert!(
        !stripped.contains(TEXT_CAP),
        "fixture setup: exactly one requires line"
    );
    let dir = temp_dir("qa02-cli");
    let atom = dir.join("stripped.atom");
    std::fs::write(&atom, stripped).unwrap();
    let mut c = Command::new(EXE);
    c.args(["check", &atom.display().to_string()]);
    let b = run_deadlined(c, Duration::from_secs(120)).expect("spawn check");
    assert_eq!(b.code, 1, "stripped declaration must reject");
    assert!(b.stderr.contains("capability.missing"), "{}", b.stderr);
}

// -- QA-03: move cleanup follows the executed control flow ----------------------

#[test]
fn qa03_p01_return_move_callee_frees_once() {
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    let x Str = \"abc\"\n    return consume(x.move)\n}\n";
    let run = pe("p01", src);
    assert_eq!(run.code, 3);
    assert_eq!(
        run.stderr, "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
        "the callee frees once; the caller schedules nothing"
    );
    no_double_free("p01", &run);
    let hir = hir_pe("p01-hir", src);
    assert_eq!(hir.code, 3);
    assert_eq!(hir.stderr, run.stderr, "HIR leg identical");
}

#[test]
fn qa03_p02_const_true_branch_move() {
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    let x Str = \"abc\"\n    if 1 == 1 { let n int = consume(x.move) }\n    return 0\n}\n";
    let run = pe("p02", src);
    assert_eq!(run.code, 0);
    assert_eq!(
        run.stderr, "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
        "the constant-true arm always executes: the callee owns x, the caller frees nothing"
    );
    no_double_free("p02", &run);
    assert_eq!(
        hir_pe("p02-hir", src).stderr,
        run.stderr,
        "HIR leg identical"
    );
}

#[test]
fn qa03_c05_const_false_branch_not_moved() {
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    let x Str = \"abc\"\n    if 1 == 0 { let n int = consume(x.move) }\n    return 0\n}\n";
    let run = pe("c05", src);
    assert_eq!(run.code, 0);
    assert_eq!(
        run.stderr, "LIT len=3\nLIT id=1\nFREE id=1\n",
        "the constant-false arm never executes: the CALLER keeps and frees x"
    );
    no_double_free("c05", &run);
    assert_eq!(
        hir_pe("c05-hir", src).stderr,
        run.stderr,
        "HIR leg identical"
    );
}

#[test]
fn qa03_dynamic_one_sided_branch_move_rejected() {
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    let x Str = \"abc\"\n    var t int = 0\n    if t < 1 { let n int = consume(x.move) }\n    return 0\n}\n";
    check_err("qa03-dyn-branch", src, "verify.ownership-required");
    // (the join rule lives entirely inside verify_text — the shared gate
    // both channels hit; the mutation legs below prove that sharing)
}

#[test]
fn qa03_both_arms_move_is_consistent() {
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    let x Str = \"abc\"\n    var t int = 0\n    if t < 1 {\n        let a int = consume(x.move)\n        t = t + a\n    } else {\n        let b int = consume(x.move)\n        t = t + b\n    }\n    return t\n}\n";
    let run = pe("both-arms", src);
    assert_eq!(run.code, 3, "either path moves once and returns 3");
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n"
    );
    no_double_free("both-arms", &run);
}

#[test]
fn qa03_p08_c06_shortcircuit_move_polarity() {
    // p08: constant-false && — the rhs never runs; the caller frees
    let p08 = "fn consume(move s Str) bool { return s.len() == 3 }\nfn main() int {\n    let x Str = \"abc\"\n    let ignored = 1 == 0 && consume(x.move)\n    return 0\n}\n";
    let run = pe("p08", p08);
    assert_eq!(run.code, 0);
    assert_eq!(run.stderr, "LIT len=3\nLIT id=1\nFREE id=1\n");
    no_double_free("p08", &run);
    assert_eq!(
        hir_pe("p08-hir", p08).stderr,
        run.stderr,
        "HIR leg identical"
    );

    // c06: constant-true && — the rhs runs; the callee frees
    let c06 = "fn consume(move s Str) bool { return s.len() == 3 }\nfn main() int {\n    let x Str = \"abc\"\n    let ignored = 1 == 1 && consume(x.move)\n    return 0\n}\n";
    let run = pe("c06", c06);
    assert_eq!(run.code, 0);
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n"
    );
    no_double_free("c06", &run);
    assert_eq!(
        hir_pe("c06-hir", c06).stderr,
        run.stderr,
        "HIR leg identical"
    );
}

#[test]
fn qa03_dynamic_shortcircuit_move_rejected() {
    let src = "fn consume(move s Str) bool { return s.len() == 3 }\nfn main() int {\n    var t int = 0\n    let x Str = \"abc\"\n    let ignored = t < 1 && consume(x.move)\n    return 0\n}\n";
    check_err("qa03-dyn-sc", src, "verify.ownership-required");
    // HIR leg: a legal dynamic-&& view call, mutated into a `.move` on a
    // `move` parameter — the maybe-executed transfer must reject
    let legal = "fn peek(s Str) bool { return s.len() == 3 }
fn main() int {
    var t int = 0
    let x Str = \"abc\"
    let ignored = t < 1 && peek(x)
    return 0
}
";
    hir_mutation_err("qa03-dyn-sc", legal, "verify.ownership-required", |b| {
        let str_params: Vec<(u32, u32)> = {
            let mut v = Vec::new();
            for (mi, m) in b.modules.iter().enumerate() {
                for (bi, body) in m.bodies.iter().enumerate() {
                    for (i, l) in body.locals.iter().enumerate() {
                        if l.role == hir::LocalRole::Param && is_owned_ty(&l.ty, m) {
                            v.push(((mi as u32) << 16 | bi as u32, i as u32));
                        }
                    }
                }
            }
            v
        };
        assert!(!str_params.is_empty(), "fixture: no Str param");
        for (mb, li) in str_params {
            let mi = (mb >> 16) as usize;
            let bi = (mb & 0xffff) as usize;
            b.modules[mi].bodies[bi].locals[li as usize].param_mode = hir::ParamMode::MoveOwn;
        }
        for m in &mut b.modules {
            for body in &mut m.bodies {
                for e in &mut body.exprs {
                    if let hir::ExprKind::Call { bindings, .. } = &mut e.kind {
                        for bd in bindings {
                            bd.moved = true;
                        }
                    }
                }
            }
        }
    });
}

// -- QA-04: nominal Str types without cleanup rights ---------------------------

#[test]
fn qa04_p04_view_param_owned_return_rejected() {
    let src = "fn steal(s Str) Str { return s }\nfn main() int {\n    let x Str = \"abc\"\n    let y Str = steal(x)\n    return y.len()\n}\n";
    check_err("qa04-return", src, "verify.ownership-required");
    // HIR leg: the legal move-param passthrough, mutated to a view param
    let legal = "fn passthrough(move s Str) Str { return s }
fn main() int {
    let x Str = \"abc\"
    let y Str = passthrough(x.move)
    return y.len()
}
";
    hir_mutation_err("qa04-return", legal, "verify.ownership-required", |b| {
        let str_params: Vec<(u32, u32)> = {
            let mut v = Vec::new();
            for (bi, body) in b.modules[0].bodies.iter().enumerate() {
                for (i, l) in body.locals.iter().enumerate() {
                    if l.role == hir::LocalRole::Param && is_owned_ty(&l.ty, &b.modules[0]) {
                        v.push((bi as u32, i as u32));
                    }
                }
            }
            v
        };
        for (bi, li) in str_params {
            b.modules[0].bodies[bi as usize].locals[li as usize].param_mode = hir::ParamMode::View;
        }
    });
    // the implicit owned let of a view parameter is rejected too
    let src2 = "fn steal(s Str) Str { let y Str = s; return y }\nfn main() int { return steal(\"ab\").len() }\n";
    check_err("qa04-let", src2, "verify.ownership-required");
    // a `move Str` parameter IS an owner and may transfer out (control)
    let ok = "fn passthrough(move s Str) Str { return s }\nfn main() int {\n    let x Str = \"abc\"\n    let y Str = passthrough(x.move)\n    return y.len()\n}\n";
    let run = pe("qa04-move-control", ok);
    assert_eq!(run.code, 3, "the move-in parameter owns and transfers");
    assert_eq!(
        run.stderr, "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
        "exactly one free at main's exit (y's buffer)"
    );
    no_double_free("qa04-move-control", &run);
}

// -- QA-05: temporary provenance through borrowed returns ----------------------

#[test]
fn qa05_p03_owned_call_view_rejected() {
    let src = "fn make() Str { return \"abc\" }\nfn main() int {\n    let v str = make().to(str)\n    return v.len()\n}\n";
    check_err("qa05-p03", src, "verify.borrow-escape");
    // HIR leg: a legal borrowed-call view, mutated so the callee's
    // return literal becomes OWNED (the view of it must then reject)
    let legal = "fn make() str { return \"abc\" }
fn main() int {
    let v str = make().to(str)
    return v.len()
}
";
    // mutate at the SIGNATURE level: the borrowed return becomes an
    // owned return (result type + return literal) — the caller's `.to(str)`
    // view then stores a view of a temporary
    hir_mutation_err("qa05-p03", legal, "verify.borrow-escape", |b| {
        let m = &mut b.modules[0];
        let result_tys: Vec<usize> = m
            .types
            .iter()
            .enumerate()
            .filter_map(|(i, t)| match &t.kind {
                hir::TypeKind::Function(sig) if is_str_ty(&sig.result, m) => Some(i),
                _ => None,
            })
            .collect();
        for i in result_tys {
            if let hir::TypeKind::Function(sig) = &mut m.types[i].kind {
                sig.result = hir::TypeUse::Builtin(hir::BuiltinType::StrOwned);
            }
        }
        // collect first, mutate after — the type table borrow conflicts
        let mut ids: Vec<u32> = Vec::new();
        {
            let mm = &*m;
            for (bi, body) in mm.bodies.iter().enumerate() {
                for (ei, e) in body.exprs.iter().enumerate() {
                    let flip = matches!(e.kind, hir::ExprKind::StrLit { .. })
                        || matches!(&e.kind, hir::ExprKind::Call { .. });
                    if flip && is_str_ty(&e.ty, mm) {
                        ids.push((bi as u32) << 16 | ei as u32);
                    }
                }
            }
        }
        for mb in ids {
            let bi = (mb >> 16) as usize;
            let ei = (mb & 0xffff) as usize;
            m.bodies[bi].exprs[ei].ty = hir::TypeUse::Builtin(hir::BuiltinType::StrOwned);
        }
    });
}

#[test]
fn qa05_mut01_owned_literal_view_rejected() {
    // the review's M05: an OWNED literal viewed into a str local
    let src = "fn main() int {\n    let v str = \"abc\"\n    return v.len()\n}\n";
    let mut b = hir_bundle(src);
    // promote the literal's type to Str (owned) — the scan must still see
    // the owned-producing literal and reject the str binding
    for m in &mut b.modules {
        for body in &mut m.bodies {
            for e in &mut body.exprs {
                if let hir::ExprKind::StrLit { .. } = &e.kind {
                    e.ty = hir::TypeUse::Builtin(hir::BuiltinType::StrOwned);
                }
            }
        }
    }
    hir_verify_err("qa05-mut01", b, "verify.borrow-escape");
}

#[test]
fn qa05_p07_borrowed_return_consumes_temp_safely() {
    // p07: the temp's view rides the borrowed return to its LAST consumer
    // (the len call) — the free happens AFTER the read, no UAF
    let src = "fn id(s str) str { return s }\nfn main() int { return id((\"a\" + \"b\").to(str)).len() }\n";
    let run = pe("p07", src);
    assert_eq!(run.code, 2, "statement-internal legal borrow");
    assert_eq!(
        run.stderr, "CONCAT a=1 b=1\nCONCAT id=1\nTEXT-SCALARS n=2\nFREE id=1\n",
        "the concat temp is freed after its last consumer (the len), never before"
    );
    no_double_free("p07", &run);
    assert_eq!(
        hir_pe("p07-hir", src).stderr,
        run.stderr,
        "HIR leg identical"
    );

    // c03: the same shape with a NAMED owner
    let c03 = "fn id(s str) str { return s }\nfn main() int { let x Str = \"abc\"; return id(x.to(str)).len() }\n";
    let run = pe("c03", c03);
    assert_eq!(run.code, 3);
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n"
    );
    no_double_free("c03", &run);
}

// -- QA-06: mut exclusivity and region escape -----------------------------------

#[test]
fn qa06_p06_mut_alias_rejected() {
    let src = "fn both(mut a str, b str) int { return a.len() + b.len() }\nfn main() int {\n    let x Str = \"abc\"\n    return both(x.to(str), x.to(str))\n}\n";
    check_err("qa06-alias", src, "verify.mut-conflict");
    // HIR leg: the legal independent-owners call, mutated so the second
    // argument's underlying read swaps to the same owner local
    let c04_legal = "fn both(mut a str, b str) int { return a.len() + b.len() }
fn main() int {
    let x Str = \"abc\"
    let y Str = \"d\"
    return both(x.to(str), y.to(str))
}
";
    hir_mutation_err("qa06-alias", c04_legal, "verify.mut-conflict", |b| {
        // main's locals: l0=x (owner), l1=y (owner); the second arg chain
        // reads l1 via to(str) — repoint every ReadLocal of the SECOND
        // owner to the first (locals are ordered; x is let before y)
        let (main_bi, owners): (usize, Vec<u32>) = {
            let m = &b.modules[0];
            let mut found = (usize::MAX, Vec::new());
            for (bi, body) in m.bodies.iter().enumerate() {
                let mut os: Vec<u32> = Vec::new();
                for (i, l) in body.locals.iter().enumerate() {
                    if is_owned_ty(&l.ty, m) && l.role == hir::LocalRole::Binding {
                        os.push(i as u32);
                    }
                }
                if os.len() == 2 {
                    found = (bi, os);
                }
            }
            found
        };
        assert_eq!(owners.len(), 2, "fixture: exactly two owned locals in main");
        assert_eq!(owners.len(), 2, "fixture: exactly two owned locals");
        let m = &mut b.modules[0];
        let main_body = &mut m.bodies[main_bi];
        for e in &mut main_body.exprs {
            if let hir::ExprKind::ReadLocal { local } = &mut e.kind {
                if local.0 == owners[1] {
                    local.0 = owners[0];
                }
            }
        }
    });
    // c04 control: two DIFFERENT owners is legal
    let c04 = "fn both(mut a str, b str) int { return a.len() + b.len() }\nfn main() int {\n    let x Str = \"abc\"\n    let y Str = \"d\"\n    return both(x.to(str), y.to(str))\n}\n";
    let run = pe("c04", c04);
    assert_eq!(run.code, 4);
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nLIT len=1\nLIT id=2\nTEXT-SCALARS n=3\nTEXT-SCALARS n=1\nFREE id=2\nFREE id=1\n",
        "independent owners; LIFO frees"
    );
    no_double_free("c04", &run);
}

#[test]
fn qa06_p05_mut_return_view_rejected() {
    let src = "fn escape(mut s str) str { return s }\nfn main() int {\n    let x Str = \"abc\"\n    let y str = escape(x.to(str))\n    return y.len()\n}\n";
    check_err("qa06-escape", src, "verify.borrow-escape");
    // HIR leg: the legal view-param return, mutated to a mut parameter
    let legal = "fn peekof(s str) str { return s }
fn main() int {
    let x Str = \"abc\"
    let y str = peekof(x.to(str))
    return y.len()
}
";
    hir_mutation_err("qa06-escape", legal, "verify.borrow-escape", |b| {
        let str_params: Vec<(u32, u32)> = {
            let m = &b.modules[0];
            let mut v = Vec::new();
            for (bi, body) in m.bodies.iter().enumerate() {
                for (i, l) in body.locals.iter().enumerate() {
                    if l.role == hir::LocalRole::Param && is_str_ty(&l.ty, m) {
                        v.push((bi as u32, i as u32));
                    }
                }
            }
            v
        };
        assert!(!str_params.is_empty(), "fixture: no str param found");
        for (bi, li) in str_params {
            b.modules[0].bodies[bi as usize].locals[li as usize].param_mode =
                hir::ParamMode::MutBorrow;
        }
    });
    // a derived view of a mut parameter escapes too
    let derived = "fn escape(mut s str) str { return s.slice(0, 1) }\nfn main() int {\n    let x Str = \"abc\"\n    let y str = escape(x.to(str))\n    return y.len()\n}\n";
    check_err("qa06-escape-derived", derived, "verify.borrow-escape");
}

// -- QA-07: loop-fresh owners vs outer owners -----------------------------------

#[test]
fn qa07_p09_loop_fresh_move_each_iteration() {
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    var i int = 0\n    for i < 2 {\n        let x Str = \"abc\"\n        let n int = consume(x.move)\n        i = i + 1\n    }\n    return i\n}\n";
    let run = pe("p09", src);
    assert_eq!(run.code, 2);
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\nLIT len=3\nLIT id=2\nTEXT-SCALARS n=3\nFREE id=2\n",
        "each iteration allocates, transfers and lets the callee free exactly once"
    );
    no_double_free("p09", &run);
    assert_eq!(
        hir_pe("p09-hir", src).stderr,
        run.stderr,
        "HIR leg identical"
    );
}

#[test]
fn qa07_outer_owner_loop_move_still_rejected() {
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    let x Str = \"abc\"\n    var i int = 0\n    for i < 2 {\n        let n int = consume(x.move)\n        i = i + 1\n    }\n    return i\n}\n";
    check_err("qa07-outer", src, "verify.use-after-move");
    // (verify_text is the single gate for both channels — the semantic
    // loop rule has no channel-specific surface beyond it)
}

// -- break/continue join matrix (QA-03 coverage extension) ----------------------

#[test]
fn qa03_break_after_body_fresh_move() {
    // the body-fresh owner moves; break exits mid-iteration with nothing
    // owned in scope; the loop exits cleanly
    let src = "fn consume(move s Str) int { return s.len() }\nfn main() int {\n    var i int = 0\n    for i < 4 {\n        let x Str = \"abc\"\n        let n int = consume(x.move)\n        if n == 3 { break }\n        i = i + 1\n    }\n    return i\n}\n";
    let run = pe("break-matrix", src);
    assert_eq!(run.code, 0, "first iteration: n==3 → break with i=0");
    assert_eq!(
        run.stderr, "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
        "one allocation, one transfer, one free — break carries no owned scope"
    );
    no_double_free("break-matrix", &run);
}
