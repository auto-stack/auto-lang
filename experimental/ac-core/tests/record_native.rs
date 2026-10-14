//! PLAN-771 T-05 record native execution tests. Real PEs only: the source
//! flagship goes through `build-source --source-profile
//! auto.source.record.draft`, the HIR fixture through `build` (bind →
//! verify_record → lower → link). Covers V-02 (value semantics), V-05
//! (aggregate params/returns, named-arg reorder, nested calls, recursion,
//! receiver-once + source-order traces via the mark intrinsics, out-slot
//! not clobbering live args), V-06 (mut exclusivity semantics at runtime),
//! V-07 (named-ctor evaluation order `ba`, never declaration order),
//! V-08 (static/instance methods on real calls) and V-09's deep inputs
//! (1024-deep type graph, 4096 mixed field-path segments) — all through
//! the shared contained executor. The R1 repair adds the F-771-R1-01
//! aliased-destination replay set (copy elision must materialize the RHS
//! before committing when the destination is still being read), the
//! F-771-R1-02 positional construction PE, and the F-771-R1-04 distinct
//! exit-code HIR oracles with a deliberate failure control.

#![cfg(windows)]

mod common;

use common::run_deadlined;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const RECORD: &str = "auto.source.record.draft";
const TRACE: &str = "hir.test.trace.v1";

fn repo_path(rel: &str) -> PathBuf {
    Path::new(REPO).join(rel)
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac771-record-native").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn build_support_lib() -> PathBuf {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--locked"])
        .current_dir(repo_path("experimental/ac-core/test-support"));
    let r = run_deadlined(cmd, Duration::from_secs(300)).expect("spawn cargo for test-support");
    assert!(!r.timed_out, "test-support build exceeded 300s deadline");
    assert_eq!(r.code, 0, "test-support build failed:\n{}", r.stderr);
    repo_path("experimental/ac-core/test-support/target/debug/ac_trace_support.lib")
}

fn build_source(src: &Path, exe: &Path, extra: &[&str]) -> common::DeadlinedRun {
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &src.display().to_string(),
        "--source-profile",
        RECORD,
        "--entry",
        "main",
        "--output",
    ])
    .arg(exe);
    for e in extra {
        c.arg(e);
    }
    run_deadlined(c, Duration::from_secs(600)).expect("spawn build")
}

fn run_pe(exe: &Path) -> common::DeadlinedRun {
    run_deadlined(Command::new(exe), Duration::from_secs(60)).expect("spawn PE")
}

// -- V-02: the flagship — value copy, .mut writes, methods, nesting -------

#[test]
fn source_flagship_pe_exit0() {
    let dir = temp_dir("flagship");
    let src = repo_path("experimental/ac-core/fixtures/record-source/valid/p01-records.at");
    let exe = dir.join("p01.exe");
    let b = build_source(&src, &exe, &[]);
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0, "all flagship self-checks must hold");
    assert!(run.stderr.is_empty(), "unexpected stderr: {}", run.stderr);
}

#[test]
fn hir_fixture_pe_exit0() {
    // r01's happy path returns 0 (c1c); the failure arms return 11 (k1,
    // value-copy check) and 12 (k2, mut-visible check) — every branch has
    // a DISTINCT exit code (F-771-R1-04: the previous 1/1/5 shape let a
    // forced failure alias the success code)
    let dir = temp_dir("hir");
    let src = repo_path("experimental/ac-core/fixtures/record-hir/valid/r01-records.atom");
    let exe = dir.join("r01.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &src.display().to_string(),
        "--entry",
        "d_main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0, "happy path exits 0; 11/12 mean a check failed");
}

#[test]
fn hir_broken_copy_control_pe_exit11() {
    // F-771-R1-04 deliberate failure control: identical to r01 except the
    // first copy-check constant forces k1. Proves the copy oracle can
    // actually fire and is distinguishable from the success code.
    let dir = temp_dir("hir-broken");
    let src =
        repo_path("experimental/ac-core/fixtures/record-hir/controls/r02-broken-copy-check.atom");
    let exe = dir.join("r02.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &src.display().to_string(),
        "--entry",
        "d_main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 0, "broken control must still build: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 11, "forced copy-check failure must exit 11");
}

#[test]
fn hir_nested_copy_return_alias_pe() {
    // F-771-R1-01/04 direct-HIR leg (V-02/V-07): nested record value copy
    // through a record-returning call, original preserved, whole-local
    // aliased self-assignment via call and via ctor (written order !=
    // declaration order), nested field-path assignment reading the same
    // root — every failure branch exits a distinct 21..29
    let dir = temp_dir("hir-r3");
    let src =
        repo_path("experimental/ac-core/fixtures/record-hir/valid/r03-copy-return-alias.atom");
    let exe = dir.join("r03.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &src.display().to_string(),
        "--entry",
        "d_main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0, "all copy/return/alias checks hold");
}

#[test]
fn aliased_assignment_value_semantics_pe() {
    // F-771-R1-01 replay set: the assignment destination may alias storage
    // the RHS still reads — copy elision must materialize the RHS first
    // and commit afterwards (whole-local ctor self-assign, record-returning
    // call self-assign, nested field-path self-assign, mut-param inter-
    // leave), with the accepted fresh-destination control
    struct Case {
        name: &'static str,
        src: &'static str,
    }
    let cases = [
        Case {
            name: "alias-ctor",
            src: "type Q { v int }\ntype P { a Q; b Q }\nfn main() int {\n    var p = P { a: Q { v: 1 }, b: Q { v: 2 } }\n    p = P { a: p.b, b: p.a }\n    if p.a.v != 2 { return 11 }\n    if p.b.v != 1 { return 12 }\n    return 0\n}\n",
        },
        Case {
            name: "alias-sret",
            src: "type Q { v int }\ntype P { a Q; b Q }\nfn swap(p P) P { return P { a: p.b, b: p.a } }\nfn main() int {\n    var p = P { a: Q { v: 1 }, b: Q { v: 2 } }\n    p = swap(p)\n    if p.a.v != 2 { return 11 }\n    if p.b.v != 1 { return 12 }\n    return 0\n}\n",
        },
        Case {
            name: "fresh-destination-control",
            src: "type Q { v int }\ntype P { a Q; b Q }\nfn swap(p P) P { return P { a: p.b, b: p.a } }\nfn main() int {\n    let p = P { a: Q { v: 1 }, b: Q { v: 2 } }\n    let q = swap(p)\n    if q.a.v != 2 { return 11 }\n    if q.b.v != 1 { return 12 }\n    if p.a.v != 1 { return 13 }\n    return 0\n}\n",
        },
        Case {
            name: "nested-field-self-assign",
            src: "type Q { x int; y int }\ntype P { a Q; b Q }\nfn main() int {\n    var p = P { a: Q { x: 1, y: 2 }, b: Q { x: 3, y: 4 } }\n    p.a = Q { x: p.a.y, y: p.a.x }\n    if p.a.x != 2 { return 11 }\n    if p.a.y != 1 { return 12 }\n    if p.b.x != 3 { return 13 }\n    return 0\n}\n",
        },
        Case {
            name: "mut-param-interleaves-out-slot",
            src: "type Q { v int }\ntype P { a Q; b Q }\nfn adj(mut p P, d int) P { p.a.v = p.a.v + d; return p }\nfn main() int {\n    var p = P { a: Q { v: 10 }, b: Q { v: 20 } }\n    p = adj(p.mut, 5)\n    if p.a.v != 15 { return 11 }\n    if p.b.v != 20 { return 12 }\n    return 0\n}\n",
        },
        Case {
            name: "view-params-share-one-object",
            src: "type Q { v int }\ntype P { a Q; b Q }\nfn both(x P, y P) int { return x.a.v + y.b.v }\nfn main() int {\n    let p = P { a: Q { v: 7 }, b: Q { v: 9 } }\n    if both(p, p) != 16 { return 11 }\n    return 0\n}\n",
        },
    ];
    for case in cases {
        let dir = temp_dir(&format!("alias-{}", case.name));
        let src = dir.join("case.at");
        std::fs::write(&src, case.src).unwrap();
        let exe = dir.join("case.exe");
        let b = build_source(&src, &exe, &[]);
        assert_eq!(b.code, 0, "{}: stderr: {}", case.name, b.stderr);
        let run = run_pe(&exe);
        assert!(!run.timed_out, "{} timed out", case.name);
        assert_eq!(run.code, 0, "{} exited {}", case.name, run.code);
    }
}

#[test]
fn positional_ctor_pe() {
    // F-771-R1-02: `Q(7)` positional construction — single field, nested
    // positional args, and a positional self-reassignment whose arguments
    // read the destination
    let dir = temp_dir("positional");
    let src = dir.join("pos.at");
    std::fs::write(
        &src,
        "type Q { v int }\ntype P { a Q; b Q }\nfn main() int {\n    let q = Q(7)\n    if q.v != 7 { return 1 }\n    let p = P(Q(1), Q(2))\n    if p.a.v != 1 { return 2 }\n    if p.b.v != 2 { return 3 }\n    var t = P(Q(1), Q(2))\n    t = P(t.b, t.a)\n    if t.a.v != 2 { return 4 }\n    if t.b.v != 1 { return 5 }\n    return 0\n}\n",
    )
    .unwrap();
    let exe = dir.join("pos.exe");
    let b = build_source(&src, &exe, &[]);
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0);
}

// -- V-05/V-07: evaluation-order traces --------------------------------------

#[test]
fn named_ctor_evaluates_in_source_order_ba() {
    let lib = build_support_lib();
    let dir = temp_dir("ctor-order");
    let src = dir.join("order.at");
    std::fs::write(
        &src,
        "type Q {\n    a int\n    b int\n}\nfn main() int {\n    let q = Q { b: mark_b(), a: mark_a() }\n    if q.a != 1 { return 2 }\n    if q.b != 2 { return 3 }\n    return 0\n}\n",
    )
    .unwrap();
    let exe = dir.join("order.exe");
    let b = build_source(
        &src,
        &exe,
        &[
            "--capability",
            TRACE,
            "--support-lib",
            &lib.display().to_string(),
        ],
    );
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0);
    // SOURCE order: b's argument evaluated first — `ba`, never `ab`
    assert_eq!(run.stderr, "ba", "ctor evaluates args as written");
}

#[test]
fn receiver_once_then_args_in_order() {
    let lib = build_support_lib();
    let dir = temp_dir("recv-once");
    let src = dir.join("recv.at");
    std::fs::write(
        &src,
        "type P {\n    v int\n    static fn mk(x int) P { return P { v: x } }\n    fn plus(a int, b int) int { return .v + a + b }\n}\nfn main() int {\n    let r = P.mk(mark_a()).plus(mark_a(), mark_b())\n    if r != 4 { return 9 }\n    return 0\n}\n",
    )
    .unwrap();
    let exe = dir.join("recv.exe");
    let b = build_source(
        &src,
        &exe,
        &[
            "--capability",
            TRACE,
            "--support-lib",
            &lib.display().to_string(),
        ],
    );
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0);
    // receiver (ctor arg mark_a) exactly once FIRST, then mark_a, mark_b
    assert_eq!(run.stderr, "aab");
}

#[test]
fn named_args_reorder_and_nested_calls_pe() {
    let dir = temp_dir("named-args");
    let src = dir.join("na.at");
    std::fs::write(
        &src,
        "type P {\n    v int\n    w int\n}\nfn sub(a int, b int) int { return a - b }\nfn pick(p P, k int) int { return p.v * 10 + p.w + k }\nfn main() int {\n    let r = pick(P { w: 3, v: sub(b: 2, a: 9) }, 1)\n    if r != 74 { return r }\n    let n = sub(a: 10, b: 4)\n    if n != 6 { return 2 }\n    return 0\n}\n",
    )
    .unwrap();
    let exe = dir.join("na.exe");
    let b = build_source(&src, &exe, &[]);
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(
        run.code, 0,
        "named args bind by name; nested record args live"
    );
}

#[test]
fn record_return_through_out_slot_pe() {
    // record-returning functions chain through the ABI-E out slot; the
    // inner call's out slot must not clobber the outer call's live args
    let dir = temp_dir("out-slot");
    let src = dir.join("out.at");
    std::fs::write(
        &src,
        "type P {\n    v int\n    w int\n}\nfn mk(a int, b int) P { return P { v: a, w: b } }\nfn swap(p P) P { return P { v: p.w, w: p.v } }\nfn main() int {\n    let q = swap(mk(7, 3))\n    if q.v != 3 { return 1 }\n    if q.w != 7 { return 2 }\n    let r = swap(swap(mk(1, 2)))\n    if r.v != 1 { return 3 }\n    if r.w != 2 { return 4 }\n    return 0\n}\n",
    )
    .unwrap();
    let exe = dir.join("out.exe");
    let b = build_source(&src, &exe, &[]);
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0);
}

#[test]
fn recursion_with_record_and_scalar_pe() {
    let dir = temp_dir("recursion");
    let src = dir.join("rec.at");
    std::fs::write(
        &src,
        "type Acc {\n    total int\n}\nfn drill(mut a Acc, n int) int {\n    a.total = a.total + n\n    if n > 0 { return drill(a.mut, n - 1) }\n    return a.total\n}\nfn main() int {\n    var a = Acc { total: 0 }\n    let r = drill(a.mut, 5)\n    if r != 15 { return r }\n    if a.total != 15 { return 99 }\n    return 0\n}\n",
    )
    .unwrap();
    let exe = dir.join("rec.exe");
    let b = build_source(&src, &exe, &[]);
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0, "mut borrow propagates through recursion");
}

// -- V-06: mut semantics at runtime ------------------------------------------

#[test]
fn mut_writes_target_only_pe() {
    let dir = temp_dir("mut-target");
    let src = dir.join("mt.at");
    std::fs::write(
        &src,
        "type P {\n    v int\n}\nfn bump(mut p P, d int) int { p.v = p.v + d; return p.v }\nfn main() int {\n    var x = P { v: 10 }\n    var y = P { v: 100 }\n    let r1 = bump(x.mut, 5)\n    if r1 != 15 { return 1 }\n    if x.v != 15 { return 2 }\n    let r2 = bump(y.mut, 1)\n    if r2 != 101 { return 3 }\n    if y.v != 101 { return 4 }\n    if x.v != 15 { return 5 }\n    return 0\n}\n",
    )
    .unwrap();
    let exe = dir.join("mt.exe");
    let b = build_source(&src, &exe, &[]);
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0);
}

// -- V-09: deep inputs ---------------------------------------------------------

/// 1024-deep record chain + 4096 mixed field-path segments across four
/// walks; runs as one PE. Types generate as text (~100 KB) — parse, verify,
/// layout, lower and run must all stay iterative and complete.
#[test]
fn deep_type_graph_and_field_paths_pe() {
    let dir = temp_dir("deep");
    let src = dir.join("deep.at");
    let mut s = String::with_capacity(200_000);
    s.push_str("type T0 { x int }\n");
    for k in 1..1024u32 {
        s.push_str(&format!("type T{k} {{ a int; r T{} }}\n", k - 1));
    }
    // four walks: constructor path + three read paths, each exactly 1024
    // segments (1023 `r` descents + the `x` leaf) = 4096 mixed segments
    let walk = format!("r.{}", "r.".repeat(1022));
    let walk = format!("{}x", walk);
    s.push_str(&format!(
        "fn deep(v T1023) int {{ return v.{} }}
",
        walk
    ));
    let mut ctor = "T0 { x: 7 }".to_string();
    for k in 1..1024u32 {
        ctor = format!("T{k} {{ a: {}, r: {} }}", (k % 5) as i32, ctor);
    }
    let w1024 = format!("{}x", "r.".repeat(1023));
    let w1022 = format!("{}x", "r.".repeat(1021));
    let w1021 = format!("{}x", "r.".repeat(1020));
    s.push_str(&format!(
        "fn main() int {{
    let v = {ctor}
    let d1 = deep(v)
    if d1 != 7 {{ return 1 }}
    let d2 = v.{w2}
    if d2 != 7 {{ return 2 }}
    let d3 = v.r.r.{w3}
    if d3 != 7 {{ return 3 }}
    let d4 = v.r.r.r.{w4}
    if d4 != 7 {{ return 4 }}
    if v.a != 3 {{ return 5 }}
    return 0
}}
",
        ctor = ctor,
        w2 = w1024,
        w3 = w1022,
        w4 = w1021,
    ));
    std::fs::write(&src, s).unwrap();
    let exe = dir.join("deep.exe");
    let b = build_source(&src, &exe, &[]);
    assert_eq!(b.code, 0, "stderr: {}", b.stderr);
    let run = run_pe(&exe);
    assert!(!run.timed_out, "deep PE must finish within the deadline");
    assert_eq!(run.code, 0, "deep walks read the bottom field");
}
