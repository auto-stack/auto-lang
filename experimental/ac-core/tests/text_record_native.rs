//! PLAN-782 T-05 native execution tests. Real PEs only: source programs
//! go through `build-source --source-profile auto.source.text-record.draft`
//! → combination verify (resource plan) → the merged Cranelift lowering
//! (inline record aggregates, per-field `ac_rt_*` calls) → rust-lld with
//! the contract-carrying runtime staticlib. Covers the native legs of
//! V-02 (Token-shape construction / char fields / nesting), V-05
//! (replacement and overlap shapes with exact trace oracles), V-09
//! (short-circuit skips never allocate), V-10 (deep nesting + loop
//! scale), V-11 (capability gate + artifact faces) and V-12 (the
//! Token-shaped microprogram + profile coexistence). Every subprocess
//! runs under the shared contained executor.

#![cfg(windows)]

mod common;

use common::run_deadlined;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");

const TR: &str = "auto.source.text-record.draft";
const TEXT_CAP: &str = "ac.runtime.text.v1";
const TRACE_CAP: &str = "hir.test.trace.v1";

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac782-tr-native").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_pe(exe: &Path) -> common::DeadlinedRun {
    run_deadlined(Command::new(exe), Duration::from_secs(120)).expect("spawn PE")
}

fn build_tr(src: &Path, exe: &Path, extra_caps: &[&str]) -> common::DeadlinedRun {
    let lib = common::text_runtime_lib();
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &src.display().to_string(),
        "--source-profile",
        TR,
        "--entry",
        "main",
        "--capability",
        TEXT_CAP,
    ]);
    for cap in extra_caps {
        c.args(["--capability", cap]);
    }
    c.args(["--support-lib"]).arg(&lib).arg("--output").arg(exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert!(!b.timed_out, "build deadline");
    assert_eq!(b.code, 0, "build failed:\n{}", b.stderr);
    b
}

fn write_src(dir: &Path, name: &str, src: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, src).unwrap();
    p
}

// -- V-02: construction / typing / char / nesting (value oracles) --------

#[test]
fn v02_token_shape_values() {
    let dir = temp_dir("v02-tok");
    let src = write_src(
        &dir,
        "t.at",
        r#"type Tok { text Str; kind int; c char; line int }
fn main() int {
    let t Tok = Tok { text: "A中", kind: 7, c: 'Z', line: 42 }
    if t.text.len() != 2 { return 100 }
    if t.kind != 7 { return 101 }
    if t.c != 'Z' { return 102 }
    if t.line != 42 { return 103 }
    let u Tok = Tok(t.text.clone(), t.kind + 1, t.c, t.line - 1)
    if u.kind != 8 { return 104 }
    return u.text.len() + u.line
}
"#,
    );
    let exe = dir.join("t.exe");
    build_tr(&src, &exe, &[TRACE_CAP]);
    let run = run_pe(&exe);
    assert_eq!(run.code, 2 + 41, "2 + 41");
    assert_eq!(
        run.stderr,
        "LIT len=4\nLIT id=1\nTEXT-SCALARS n=2\nCLONE len=4\nCLONE id=2\nTEXT-SCALARS n=2\nFREE id=2\nFREE id=1\n",
        "token-shape lifecycle: reads interleave in source order, LIFO exit frees"
    );
}

#[test]
fn v02_nested_record_fields() {
    let dir = temp_dir("v02-nest");
    let src = write_src(
        &dir,
        "n.at",
        r#"type Inner { s Str; n int }
type Outer { i Inner; flag bool }
fn main() int {
    let o Outer = Outer { i: Inner { s: "deep", n: 5 }, flag: true }
    if o.i.s.len() != 4 { return 100 }
    if o.i.n != 5 { return 101 }
    var p Outer = o.clone()
    p.i.n = 9
    if o.i.n != 5 { return 102 }
    if p.i.n != 9 { return 103 }
    return p.i.s.len() + o.i.n
}
"#,
    );
    let exe = dir.join("n.exe");
    build_tr(&src, &exe, &[TRACE_CAP]);
    let run = run_pe(&exe);
    assert_eq!(
        run.code,
        4 + 5,
        "deep clone is independent (9 written to p only)"
    );
    assert_eq!(
        run.stderr,
        "LIT len=4\nLIT id=1\nTEXT-SCALARS n=4\nCLONE len=4\nCLONE id=2\nTEXT-SCALARS n=4\nFREE id=2\nFREE id=1\n",
        "nested resource record: per-field deep clone, LIFO exit"
    );
}

// -- V-05: replacement / overlap (exact trace oracles) --------------------

#[test]
fn v05_overlap_shapes_trace() {
    let dir = temp_dir("v05-ov");
    let src = write_src(
        &dir,
        "o.at",
        r#"type Tok { text Str; kind int }
fn consume(move t Tok) int { return t.kind }
fn transform(t Tok) Tok { return t.clone() }
fn main() int {
    var r Tok = Tok { text: "seed", kind: 1 }
    r = r.clone()
    r = transform(r)
    r.text = r.text + "-tail"
    let k int = consume(r.move)
    r = Tok { text: "rebuilt", kind: k + 1 }
    return r.text.len() + r.kind
}
"#,
    );
    let exe = dir.join("o.exe");
    build_tr(&src, &exe, &[TRACE_CAP]);
    let run = run_pe(&exe);
    assert_eq!(run.code, 7 + 2, "\"rebuilt\".len()=7 + kind=2");
    // P4 oracle: every replacement frees its OLD buffer exactly once after
    // the RHS materialized; the moved root's buffer is freed by the CALLEE;
    // the final rebuilt root frees at the exit
    assert_eq!(
        run.stderr,
        "LIT len=4\nLIT id=1\n\
         CLONE len=4\nCLONE id=2\nFREE id=1\n\
         CLONE len=4\nCLONE id=3\nFREE id=2\n\
         CONCAT a=4 b=5\nCONCAT id=4\nFREE id=3\n\
         FREE id=4\n\
         LIT len=7\nLIT id=5\nTEXT-SCALARS n=7\nFREE id=5\n",
        "r=r.clone() frees id=1; transform frees id=2; field replace frees id=3; callee frees id=4; exit frees id=5"
    );
}

// -- V-09: short-circuit skips never allocate ------------------------------

#[test]
fn v09_short_circuit_skips_clone() {
    let dir = temp_dir("v09-sc");
    let src = write_src(
        &dir,
        "s.at",
        r#"fn check(t Tok, v int) bool { return t.kind > v }
type Tok { text Str; kind int }
fn main() int {
    let a Tok = Tok { text: "a", kind: 1 }
    let b Tok = Tok { text: "b", kind: 2 }
    if false && (check(a.clone(), 0)) {
        return 100
    }
    if !(true || (check(b.clone(), 0))) {
        return 101
    }
    return a.text.len() + b.text.len()
}
"#,
    );
    let exe = dir.join("s.exe");
    build_tr(&src, &exe, &[TRACE_CAP]);
    let run = run_pe(&exe);
    assert_eq!(run.code, 2, "a.len()+b.len() = 1+1");
    // the skipped clone arms never run: exactly two allocations (the ctor
    // promotions), two exit frees, nothing else
    assert_eq!(
        run.stderr,
        "LIT len=1\nLIT id=1\nLIT len=1\nLIT id=2\nTEXT-SCALARS n=1\nTEXT-SCALARS n=1\nFREE id=2\nFREE id=1\n",
        "skipped clone arms allocate and free nothing"
    );
}

// -- V-10: scale ------------------------------------------------------------

#[test]
fn v10_loop_4096_fresh_roots() {
    let dir = temp_dir("v10-loop");
    let src = write_src(
        &dir,
        "l.at",
        r#"type Tok { text Str; kind int }
fn main() int {
    var total int = 0
    var i int = 0
    for i < 4096 {
        i = i + 1
        let t Tok = Tok { text: "x", kind: i }
        total = total + t.kind
    }
    return total % 256
}
"#,
    );
    let exe = dir.join("l.exe");
    build_tr(&src, &exe, &[]);
    let run = run_pe(&exe);
    // sum(1..4096) = 4096*4097/2 = 8386560; % 256 = 8386560 - 32760*256 = 0
    assert_eq!(run.code, 0, "4096 iterations, fresh root each");
}

#[test]
fn v10_zero_iteration_loop_allocates_nothing() {
    let dir = temp_dir("v10-zero");
    let src = write_src(
        &dir,
        "z.at",
        r#"type Tok { text Str; kind int }
fn main() int {
    var i int = 0
    for i < 0 {
        let t Tok = Tok { text: "never", kind: 1 }
        i = i + 1
    }
    return i
}
"#,
    );
    let exe = dir.join("z.exe");
    build_tr(&src, &exe, &[TRACE_CAP]);
    let run = run_pe(&exe);
    assert_eq!(run.code, 0, "zero iterations");
    assert_eq!(run.stderr, "", "zero allocations, zero frees");
}

#[test]
fn v10_deep_nested_256() {
    let dir = temp_dir("v10-deep");
    // 256 nested wrappers around a Str leaf; read the leaf's length
    let mut tys = String::new();
    tys.push_str("type T0 { s Str }\n");
    for k in 1..=256u32 {
        tys.push_str(&format!("type T{k} {{ r T{} }}\n", k - 1));
    }
    let mut ctor = "T0 { s: \"leaf\" }".to_string();
    for k in 1..=256u32 {
        ctor = format!("T{k} {{ r: {ctor} }}");
    }
    let path = format!("v{}.s.len()", ".r".repeat(256));
    let src = format!("{tys}fn main() int {{\n    let v = {ctor}\n    return {path}\n}}\n");
    let src = write_src(&dir, "d.at", &src);
    let exe = dir.join("d.exe");
    build_tr(&src, &exe, &[TRACE_CAP]);
    let run = run_pe(&exe);
    assert_eq!(run.code, 4, "the leaf's length through 8 path segments");
    assert_eq!(
        run.stderr, "LIT len=4\nLIT id=1\nTEXT-SCALARS n=4\nFREE id=1\n",
        "deep nesting: one buffer, one free"
    );
}

// -- V-11: capability + artifact faces --------------------------------------

#[test]
fn v11_missing_capability_rejects() {
    let dir = temp_dir("v11-cap");
    let src = write_src(
        &dir,
        "c.at",
        "type Tok { text Str; kind int }\nfn main() int {\n    let t Tok = Tok { text: \"x\", kind: 1 }\n    return t.kind\n}\n",
    );
    let lib = common::text_runtime_lib();
    let exe = dir.join("c.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &src.display().to_string(),
        "--source-profile",
        TR,
        "--entry",
        "main",
        "--support-lib",
    ])
    .arg(&lib)
    .arg("--output")
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(120)).expect("spawn build");
    assert_eq!(b.code, 1, "missing capability must reject the build");
    assert!(b.stderr.contains("capability.missing"), "{}", b.stderr);
    assert!(!exe.exists(), "no artifact on rejection");
}

// -- V-12: Token-shaped microprogram + profile coexistence ------------------

#[test]
fn v12_token_microprogram() {
    let dir = temp_dir("v12-tok");
    let src = write_src(
        &dir,
        "m.at",
        r#"type Token {
    text Str
    kind int
    line int
    at int
    len int

    fn is_ident() bool {
        return .kind == 1
    }

    static fn ident(move text Str, line int, at int) Token {
        let n int = text.len()
        return Token { text: text.move, kind: 1, line: line, at: at, len: n }
    }
}

fn weigh(move t Token) int { return t.len + t.line + t.at }

fn main() int {
    let s Str = "keyword"
    let a Token = Token.ident(s.move, 3, 12)
    if a.is_ident() == false { return 100 }
    if a.len != 7 { return 101 }
    var b Token = a.clone()
    b.at = 40
    if a.at != 12 { return 102 }
    let w int = weigh(b.move)
    return w + a.text.len()
}
"#,
    );
    let exe = dir.join("m.exe");
    build_tr(&src, &exe, &[TRACE_CAP]);
    let run = run_pe(&exe);
    // weigh(b) = 7 + 3 + 40 = 50; a.text.len() = 7
    assert_eq!(run.code, 57, "7+3+40 + 7");
    assert_eq!(
        run.stderr,
        "LIT len=7\nLIT id=1\nTEXT-SCALARS n=7\nCLONE len=7\nCLONE id=2\nFREE id=2\nTEXT-SCALARS n=7\nFREE id=1\n",
        "static new + instance method + clone + move: exact pairing, live=0"
    );
}

#[test]
fn v12_old_profiles_still_build() {
    // profile coexistence: the text and record profiles keep building and
    // running on their own lowerers after the merged path landed
    let dir = temp_dir("v12-old");
    let lib = common::text_runtime_lib();
    let t_src = write_src(
        &dir,
        "t.at",
        "fn main() int {\n    let s Str = \"ab\"\n    return s.len()\n}\n",
    );
    let t_exe = dir.join("t.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &t_src.display().to_string(),
        "--source-profile",
        "auto.source.text.draft",
        "--entry",
        "main",
        "--capability",
        TEXT_CAP,
        "--support-lib",
    ])
    .arg(&lib)
    .arg("--output")
    .arg(&t_exe);
    let b = run_deadlined(c, Duration::from_secs(300)).expect("spawn");
    assert_eq!(b.code, 0, "text build:\n{}", b.stderr);
    assert_eq!(run_pe(&t_exe).code, 2, "text profile still runs");

    let r_src = write_src(
        &dir,
        "r.at",
        "type Pair { left int; right int }\nfn main() int {\n    let p = Pair { left: 20, right: 22 }\n    return p.left + p.right\n}\n",
    );
    let r_exe = dir.join("r.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &r_src.display().to_string(),
        "--source-profile",
        "auto.source.record.draft",
        "--entry",
        "main",
        "--output",
    ])
    .arg(&r_exe);
    let b = run_deadlined(c, Duration::from_secs(300)).expect("spawn");
    assert_eq!(b.code, 0, "record build:\n{}", b.stderr);
    assert_eq!(run_pe(&r_exe).code, 42, "record profile still runs");
}

// -- direct-HIR leg ----------------------------------------------------------

#[test]
fn v12_direct_hir_leg() {
    // the combination descriptor + verify + merged lowering with no source
    // front end: a handwritten Atom exercising ctor/clone/move/field ops
    let dir = temp_dir("v12-hir");
    let lib = common::text_runtime_lib();
    let atom = r#"hir(schema: "auto.hir.text-record.draft", revision: 1u32, profile: "text-record-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m {
        types {
            type t_i32 i32 {}
            type t_Str Str {}
            type t_rec_Tok record { fields: [{ name: "text", type: t_Str }, { name: "kind", type: t_i32 }] }
            type t_f_main function { params: []; result: t_i32 }
        }
        declarations { function d_main t_f_main { name: "sf_main"; body: "b_main" } }
        bodies {
            body b_main {
                owner: "d_main"
                entry: "k0"
                locals {
                    local l0 t_rec_Tok { name: "a"; role: binding }
                    local l1 t_rec_Tok { name: "b"; role: binding }
                }
                expressions {
                    expr e0 str_lit { type: t_Str; text: "direct" }
                    expr e1 constant { type: t_i32; value: 3i32 }
                    expr e2 record_ctor { type: t_rec_Tok; record: t_rec_Tok; field_args: [e0, e1]; ctor_bindings: [{ field: 0u32, arg: 0u32 }, { field: 1u32, arg: 1u32 }] }
                    expr e3 read_local { type: t_rec_Tok; local: l0 }
                    expr e4 record_clone { type: t_rec_Tok; record: t_rec_Tok; base: e3 }
                    expr e5 read_local { type: t_rec_Tok; local: l1 }
                    expr e6 field_get { type: t_i32; base: e5; path: [1u32] }
                }
                blocks { block k0 {
                    let(local: l0, value: e2) {}
                    let(local: l1, value: e4) {}
                    return(value: e6) {}
                } }
            }
        }
    }
}
"#;
    let src = write_src(&dir, "d.atom", atom);
    let exe = dir.join("d.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &src.display().to_string(),
        "--entry",
        "d_main",
        "--capability",
        TEXT_CAP,
        "--support-lib",
    ])
    .arg(&lib)
    .arg("--output")
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(300)).expect("spawn build");
    assert_eq!(b.code, 0, "direct-HIR build:\n{}", b.stderr);
    let run = run_pe(&exe);
    assert_eq!(run.code, 3, "b.kind = 3");
    assert_eq!(
        run.stderr, "LIT len=6\nLIT id=1\nCLONE len=6\nCLONE id=2\nFREE id=2\nFREE id=1\n",
        "direct-HIR lifecycle pairing"
    );
}

#[test]
fn hir_fixture_str_typed_field_get_pe() {
    // T-06 gate-work defect lock: the committed T-02 fixture spells the
    // owned-field READ `field_get { type: Str }` — verify's compatibility
    // accepts both spellings (the retagged `str` and the declared `Str`),
    // so the merged lowering must treat them as the same borrowing view.
    // Before the fix the Str spelling fell into the scalar arm and the
    // consuming text op hit an unreachable panic (native lowerer contract
    // breach on a Checked bundle).
    let dir = temp_dir("hir-str-field");
    let lib = common::text_runtime_lib();
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/text-record-hir/valid/tr01-combination.atom");
    let exe = dir.join("tr01.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &src.display().to_string(),
        "--entry",
        "d_main",
        "--capability",
        TEXT_CAP,
        "--support-lib",
    ])
    .arg(&lib)
    .arg("--output")
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(300)).expect("spawn build");
    assert_eq!(b.code, 0, "tr01 fixture build:\n{}", b.stderr);
    let run = run_pe(&exe);
    assert_eq!(run.code, 2, "consume(t.move) = \"hi\".len() = 2");
    assert_eq!(
        run.stderr, "LIT len=2\nLIT id=1\nTEXT-SCALARS n=2\nFREE id=1\n",
        "make LIT, view read, callee frees the move-in root"
    );
}

// -- F-782-R1-02/03/04 permanent regressions (review R1 counterexamples) --

#[test]
fn f782_r1_02_same_statement_move_return_rebuild() {
    // `a = consume(a.move)` where consume returns its move param whole:
    // the returned aggregate carries the SAME buffers the RHS moved out,
    // so no replace_frees may run for them — one LIT, one final FREE
    let dir = temp_dir("f2-move-ret");
    let src = write_src(
        &dir,
        "m.at",
        r#"type T { s Str }
fn consume(move t T) T { return t.move }
fn main() int {
    var a T = T { s: "abc" }
    a = consume(a.move)
    return a.s.len()
}
"#,
    );
    let exe = dir.join("m.exe");
    build_tr(&src, &exe, &[]);
    let run = run_pe(&exe);
    assert_eq!(run.code, 3, "a.s.len() after the rebuild");
    assert_eq!(
        run.stderr, "LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n",
        "single free at the final exit (before: FREE-then-RT-FAIL -7, exit 70)"
    );
    // the fresh-destination control keeps the identical shape
    let src2 = write_src(
        &dir,
        "c.at",
        r#"type T { s Str }
fn consume(move t T) T { return t.move }
fn main() int {
    let a T = T { s: "abc" }
    let b T = consume(a.move)
    return b.s.len()
}
"#,
    );
    let exe2 = dir.join("c.exe");
    build_tr(&src2, &exe2, &[]);
    let run2 = run_pe(&exe2);
    assert_eq!(run2.code, 3);
    assert_eq!(run2.stderr, run.stderr, "same target vs fresh target");
}

#[test]
fn f782_r1_03_short_circuit_loop_aggregate_temps() {
    // take->skip across loop iterations: iteration 1 executes the RHS
    // clone (freed at ITS statement tail), iteration 2 skips — the
    // aggregate temp's presence marker must keep the skip side free-less
    // (before: the stale slot was walked again -> FREE-INVALID)
    let dir = temp_dir("f3-short-loop");
    let src = write_src(
        &dir,
        "s.at",
        r#"type Tok { text Str; kind int }
fn check(t Tok) bool { return t.kind > 0 }
fn cond(i int) bool { return i == 0 }
fn main() int {
    let a Tok = Tok { text: "a", kind: 1 }
    var i int = 0
    loop {
        if cond(i) && check(a.clone()) { i = i + 1 } else { i = i + 1 }
        if i == 2 { break }
    }
    return a.text.len()
}
"#,
    );
    let exe = dir.join("s.exe");
    build_tr(&src, &exe, &[]);
    let run = run_pe(&exe);
    assert_eq!(run.code, 1);
    assert_eq!(
        run.stderr,
        "LIT len=1\nLIT id=1\nCLONE len=1\nCLONE id=2\nFREE id=2\nTEXT-SCALARS n=1\nFREE id=1\n",
        "iteration 1 clones and frees once; iteration 2 frees NOTHING"
    );

    // skip->take: iteration 1 skips, iteration 2 executes
    let src2 = write_src(
        &dir,
        "r.at",
        r#"type Tok { text Str; kind int }
fn check(t Tok) bool { return t.kind > 0 }
fn cond(i int) bool { return i == 1 }
fn main() int {
    let a Tok = Tok { text: "a", kind: 1 }
    var i int = 0
    loop {
        if cond(i) && check(a.clone()) { i = i + 1 } else { i = i + 1 }
        if i == 2 { break }
    }
    return a.text.len()
}
"#,
    );
    let exe2 = dir.join("r.exe");
    build_tr(&src2, &exe2, &[]);
    let run2 = run_pe(&exe2);
    assert_eq!(run2.code, 1);
    assert_eq!(
        run2.stderr,
        "LIT len=1\nLIT id=1\nCLONE len=1\nCLONE id=2\nFREE id=2\nTEXT-SCALARS n=1\nFREE id=1\n",
        "mirrored loop: one clone, one free, in the TAKING iteration only"
    );

    // nested logical RHS with two aggregate temps: both freed LIFO at the
    // statement tail on the executed path; the outer-skipped second if
    // allocates nothing
    let src3 = write_src(
        &dir,
        "n.at",
        r#"type Tok { text Str; kind int }
fn check(t Tok) bool { return t.kind > 0 }
fn yes() bool { return true }
fn no() bool { return false }
fn main() int {
    let a Tok = Tok { text: "a", kind: 1 }
    if yes() && (check(a.clone()) && check(a.clone())) { return a.text.len() }
    if no() && (check(a.clone()) && check(a.clone())) { return 100 }
    return 100
}
"#,
    );
    let exe3 = dir.join("n.exe");
    build_tr(&src3, &exe3, &[]);
    let run3 = run_pe(&exe3);
    assert_eq!(run3.code, 1);
    assert_eq!(
        run3.stderr,
        "LIT len=1\nLIT id=1\nCLONE len=1\nCLONE id=2\nCLONE len=1\nCLONE id=3\nFREE id=3\nFREE id=2\nTEXT-SCALARS n=1\nFREE id=1\n",
        "nested RHS temps free LIFO at the tail; the skipped second if has zero clones"
    );
}

#[test]
fn f782_r1_04_mixed_creation_order_lifo() {
    // statement temps: text id1 created FIRST, record id2 second — the
    // tail must free 2 then 1 (before: type-category order freed 1 then 2)
    let dir = temp_dir("f4-order");
    let src = write_src(
        &dir,
        "rev.at",
        r#"type T { s Str }
fn f(a str, b T) int { return a.len() + b.s.len() }
fn main() int { return f("first".to(Str).to(str), T { s: "second" }) }
"#,
    );
    let exe = dir.join("rev.exe");
    build_tr(&src, &exe, &[]);
    let run = run_pe(&exe);
    assert_eq!(run.code, 11);
    assert_eq!(
        run.stderr, "CLONE len=5\nCLONE id=1\nLIT len=6\nLIT id=2\nTEXT-SCALARS n=5\nTEXT-SCALARS n=6\nFREE id=2\nFREE id=1\n",
        "creation-order LIFO across types (record freed first)"
    );

    // locals at the function exit: text id1 first, record id2 second
    let src2 = write_src(
        &dir,
        "loc.at",
        r#"type T { s Str }
fn main() int {
    let s Str = "first"
    let a T = T { s: "second" }
    return s.len() + a.s.len()
}
"#,
    );
    let exe2 = dir.join("loc.exe");
    build_tr(&src2, &exe2, &[]);
    let run2 = run_pe(&exe2);
    assert_eq!(run2.code, 11);
    assert_eq!(
        run2.stderr, "LIT len=5\nLIT id=1\nLIT len=6\nLIT id=2\nTEXT-SCALARS n=5\nTEXT-SCALARS n=6\nFREE id=2\nFREE id=1\n",
        "exit frees the record root before the earlier text root"
    );

    // the mirrored control (record created first) keeps its correct order
    let src3 = write_src(
        &dir,
        "ctl.at",
        r#"type T { s Str }
fn f(a T, b str) int { return a.s.len() + b.len() }
fn main() int { return f(T { s: "first" }, "second".to(Str).to(str)) }
"#,
    );
    let exe3 = dir.join("ctl.exe");
    build_tr(&src3, &exe3, &[]);
    let run3 = run_pe(&exe3);
    assert_eq!(run3.code, 11);
    assert_eq!(
        run3.stderr, "LIT len=5\nLIT id=1\nCLONE len=6\nCLONE id=2\nTEXT-SCALARS n=5\nTEXT-SCALARS n=6\nFREE id=2\nFREE id=1\n",
        "mirrored creation order: unchanged correct LIFO"
    );
}

// -- V-07 (F-782-R1-06 repair): REAL fault injection through the runtime
// -- control surface (--rt-fail-nth-alloc / --rt-set-budget /
// -- rt-fail-ledger-nth emit wrapper calls to the EXISTING runtime
// -- symbols) on actual combination construction/clone/replacement code.
// -- Oracle: the failing statement's PARTIAL OUTPUT rolls back in reverse
// -- before the fatal trap; the old target is never committed/overwritten;
// -- exact-fit budgets and zero-length fields are the boundary controls.

fn build_tr_inj(
    src: &Path,
    exe: &Path,
    fail_nth: Option<i32>,
    budget: Option<i64>,
    ledger_nth: Option<i32>,
) {
    let lib = common::text_runtime_lib();
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &src.display().to_string(),
        "--source-profile",
        TR,
        "--entry",
        "main",
        "--capability",
        TEXT_CAP,
    ]);
    if let Some(n) = fail_nth {
        c.args(["--rt-fail-nth-alloc"]).arg(n.to_string());
    }
    if let Some(b) = budget {
        c.args(["--rt-set-budget"]).arg(b.to_string());
    }
    if let Some(n) = ledger_nth {
        c.args(["--rt-fail-ledger-nth"]).arg(n.to_string());
    }
    c.args(["--support-lib"]).arg(&lib).arg("--output").arg(exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert!(!b.timed_out, "build deadline");
    assert_eq!(b.code, 0, "build failed:\n{}", b.stderr);
}

#[test]
fn v07_clone_partial_output_rolls_back_before_trap() {
    // allocs: "aa"(1) "bb"(2) clone-a(3) clone-b(4) — failing the 4th
    // rolls the successful clone-a back (FREE id=3) before RT-FAIL/70;
    // the source root's own buffers are untouched
    let dir = temp_dir("v07-clone-f4");
    let src = write_src(
        &dir,
        "c.at",
        r#"type T { a Str; b Str }
fn main() int {
    let x T = T { a: "aa", b: "bb" }
    let y T = x.clone()
    return y.a.len()
}
"#,
    );
    let exe = dir.join("c.exe");
    build_tr_inj(&src, &exe, Some(4), None, None);
    let run = run_pe(&exe);
    assert_eq!(run.code, 70, "fatal trap exit");
    assert_eq!(
        run.stderr,
        "LIT len=2\nLIT id=1\nLIT len=2\nLIT id=2\nCLONE len=2\nCLONE id=3\nCLONE len=2\nRT-FAIL code=-5\nFREE id=3\n",
        "partial clone rolls back in reverse; source buffers never freed"
    );
}

#[test]
fn v07_ctor_field_failure_rolls_back_before_trap() {
    // the 2nd field's literal allocation fails: the 1st field's buffer is
    // the statement's partial output and frees before the trap
    let dir = temp_dir("v07-ctor-f2");
    let src = write_src(
        &dir,
        "n.at",
        r#"type T { a Str; b Str }
fn main() int {
    let x T = T { a: "aa", b: "bb" }
    return x.a.len()
}
"#,
    );
    let exe = dir.join("n.exe");
    build_tr_inj(&src, &exe, Some(2), None, None);
    let run = run_pe(&exe);
    assert_eq!(run.code, 70);
    assert_eq!(
        run.stderr, "LIT len=2\nLIT id=1\nLIT len=2\nRT-FAIL code=-5\nFREE id=1\n",
        "first field buffer rolled back; no id assigned to the failed one"
    );
}

#[test]
fn v07_replacement_failure_preserves_old_target() {
    // r = r.clone() failing at the 2nd clone field: the old root's buffers
    // (id1/id2) are NOT freed — the commit (replace_frees) only runs after
    // a fully materialized RHS; only the partial clone rolls back
    let dir = temp_dir("v07-repl-f4");
    let src = write_src(
        &dir,
        "r.at",
        r#"type T { a Str; b Str }
fn main() int {
    var r T = T { a: "aa", b: "bb" }
    r = r.clone()
    return r.a.len()
}
"#,
    );
    let exe = dir.join("r.exe");
    build_tr_inj(&src, &exe, Some(4), None, None);
    let run = run_pe(&exe);
    assert_eq!(run.code, 70);
    assert_eq!(
        run.stderr,
        "LIT len=2\nLIT id=1\nLIT len=2\nLIT id=2\nCLONE len=2\nCLONE id=3\nCLONE len=2\nRT-FAIL code=-5\nFREE id=3\n",
        "old target id1/id2 survive uncommitted; partial clone id3 rolled back"
    );
}

#[test]
fn v07_budget_boundary_exact_fit_vs_one_short() {
    // four 2-byte buffers: budget 8 = exact fit (success, full pairing);
    // budget 6 rejects the 4th request with ERR_BUDGET (-4) — the
    // budget rejection makes no allocator call and still rolls back
    let dir = temp_dir("v07-budget");
    let src = write_src(
        &dir,
        "b.at",
        r#"type T { a Str; b Str }
fn main() int {
    let x T = T { a: "aa", b: "bb" }
    let y T = x.clone()
    return y.a.len()
}
"#,
    );
    let exe = dir.join("b8.exe");
    build_tr_inj(&src, &exe, None, Some(8), None);
    let run = run_pe(&exe);
    assert_eq!(run.code, 2, "exact-fit budget succeeds");
    assert_eq!(
        run.stderr,
        "LIT len=2\nLIT id=1\nLIT len=2\nLIT id=2\nCLONE len=2\nCLONE id=3\nCLONE len=2\nCLONE id=4\nTEXT-SCALARS n=2\nFREE id=4\nFREE id=3\nFREE id=2\nFREE id=1\n",
        "exact fit: full lifecycle pairing"
    );
    let exe6 = dir.join("b6.exe");
    build_tr_inj(&src, &exe6, None, Some(6), None);
    let run6 = run_pe(&exe6);
    assert_eq!(run6.code, 70, "one byte short traps");
    assert_eq!(
        run6.stderr,
        "LIT len=2\nLIT id=1\nLIT len=2\nLIT id=2\nCLONE len=2\nCLONE id=3\nCLONE len=2\nRT-FAIL code=-4\nFREE id=3\n",
        "ERR_BUDGET (-4) distinct from env-OOM (-5); rollback still runs"
    );
}

#[test]
fn v07_ledger_failure_and_zero_length_control() {
    // the 1st ledger insert fails: the runtime itself already rolled the
    // user buffer back (HeapFree inside try_alloc), so no id exists and
    // the compiler rollback has nothing to free
    let dir = temp_dir("v07-ledger");
    let src = write_src(
        &dir,
        "l.at",
        r#"type T { a Str; b Str }
fn main() int {
    let x T = T { a: "aa", b: "bb" }
    return x.a.len()
}
"#,
    );
    let exe = dir.join("l.exe");
    build_tr_inj(&src, &exe, None, None, Some(1));
    let run = run_pe(&exe);
    assert_eq!(run.code, 70);
    assert_eq!(
        run.stderr, "LIT len=2\nRT-FAIL code=-5\n",
        "ledger-refused allocation never gets an id; no double handling"
    );
    // zero-length fields allocate nothing: the injection can never fire
    let zsrc = write_src(
        &dir,
        "z.at",
        r#"type T { a Str; b Str }
fn main() int {
    let x T = T { a: "", b: "" }
    return x.a.len()
}
"#,
    );
    let zexe = dir.join("z.exe");
    build_tr_inj(&zsrc, &zexe, Some(1), None, None);
    let zrun = run_pe(&zexe);
    assert_eq!(zrun.code, 0, "zero-length = zero allocations = no failure");
    assert_eq!(zrun.stderr, "LIT len=0\nLIT len=0\nTEXT-SCALARS n=0\n");
}
