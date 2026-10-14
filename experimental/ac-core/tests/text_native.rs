//! PLAN-770 T-05 text native execution tests. Real PEs only: source
//! programs and one direct-HIR fixture go through `build-source
//! --source-profile auto.source.text.draft` / `build` → text verify (with
//! the resource plan) → Cranelift text lowering → rust-lld with the
//! contract-carrying `ac_rt_*` runtime staticlib. Covers:
//!
//! - V-02 Unicode surface (scalar-indexed len/slice/eq, NUL, empty,
//!   combining marks; source AND direct-HIR legs),
//! - V-03 char conversions incl. the invalid-scalar trap70,
//! - V-04 lifecycle observability through the runtime trace (LIT/CONCAT/
//!   CLONE/FREE pairing, LIFO, `.move` transfer to the callee, cross-frame
//!   owned return via the ABI-C channel),
//! - V-07 evaluation order with trace intrinsics and text ops side by side,
//! - V-09 scale (4096 concat chain, 4096-iteration resource loop, 4096-deep
//!   call chain with text args) — full 32-bit exit codes (values ≥ 256
//!   distinguish real results from any 8-bit truncation),
//! - V-10 contract gate faces (missing/hash/api/target mutations, missing
//!   capability, sentinel preservation) and scalar-surface isolation.
//!
//! Every subprocess runs under the shared contained executor.

#![cfg(windows)]

mod common;

use common::run_deadlined;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const TEXT: &str = "auto.source.text.draft";
const TEXT_CAP: &str = "ac.runtime.text.v1";

fn repo_path(rel: &str) -> PathBuf {
    Path::new(REPO).join(rel)
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac770-text-native").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_pe(exe: &Path) -> common::DeadlinedRun {
    run_deadlined(Command::new(exe), Duration::from_secs(60)).expect("spawn PE")
}

/// Build a text source program into a real PE (runtime lib + capability
/// wired); asserts the build succeeded.
fn build_text(src: &Path, exe: &Path) -> common::DeadlinedRun {
    let lib = common::text_runtime_lib();
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &src.display().to_string(),
        "--source-profile",
        TEXT,
        "--entry",
        "main",
        "--capability",
        TEXT_CAP,
        "--support-lib",
    ])
    .arg(&lib)
    .arg("--output")
    .arg(exe);
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

// -- V-02: Unicode surface ---------------------------------------------------

#[test]
fn v02_unicode_len_slice_eq() {
    let dir = temp_dir("v02-uni");
    let src = write_src(
        &dir,
        "u.at",
        r#"fn main() int {
    let s str = "A中😀"
    if s.len() != 3 { return 100 }
    let v str = s.slice(1, 3)
    if v == "中😀" { return 0 }
    return 101
}
"#,
    );
    let exe = dir.join("u.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 0, "A中😀 len=3, slice(1,3)=中😀");
    // exact trace oracle: scalar count (3), byte length of the slice (7)
    assert_eq!(
        run.stderr, "TEXT-SCALARS n=3\nSLICE s=1 e=3 len=7\n",
        "exact stderr oracle"
    );
}

#[test]
fn v02_combining_marks_len_two() {
    let dir = temp_dir("v02-comb");
    let src = write_src(
        &dir,
        "c.at",
        // e + U+0301 COMBINING ACUTE: two scalars, three UTF-8 bytes
        r#"fn main() int {
    let s str = "e\u{0301}"
    if s.len() == 2 { return s.len_bytes() }
    return 1
}
"#,
    );
    let exe = dir.join("c.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 3, "combining e+U+0301: len=2 scalars, 3 bytes");
}

#[test]
fn v02_nul_not_a_terminator() {
    let dir = temp_dir("v02-nul");
    let src = write_src(
        &dir,
        "n.at",
        r#"fn main() int {
    let a str = "x\u{0}y"
    let b str = "x\u{0}y\u{0}"
    if a == b { return 1 }
    if a.len() != 3 { return 2 }
    let c str = "x\u{0}y\u{0}"
    if b == c { return 0 }
    return 3
}
"#,
    );
    let exe = dir.join("n.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 0, "NUL is compared as a byte, never a terminator");
}

#[test]
fn v02_empty_and_empty_slice() {
    let dir = temp_dir("v02-empty");
    let src = write_src(
        &dir,
        "e.at",
        r#"fn main() int {
    let s str = ""
    if s.len() != 0 { return 1 }
    let v str = s.slice(0, 0)
    let w str = "ab".slice(2, 2)
    if v == w { return 0 }
    return 2
}
"#,
    );
    let exe = dir.join("e.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 0, "empty text and empty ranges are legal");
}

#[test]
fn v02_direct_hir_leg() {
    // the SAME surface through a programmatic HIR document (`build`)
    let dir = temp_dir("v02-hir");
    let exe = dir.join("h.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build",
        &repo_path("experimental/ac-core/fixtures/text-hir/valid/n01-native-ops.atom")
            .display()
            .to_string(),
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
    assert_eq!(b.code, 0, "HIR-mode text build failed:\n{}", b.stderr);
    let run = run_pe(&exe);
    assert_eq!(run.code, 3, "A中😀 len=3 through the direct HIR leg");
    assert_eq!(run.stderr, "TEXT-SCALARS n=3\n");
    // the receipt carries the runtime contract fingerprint (V-10 identity)
    let receipt = std::fs::read_to_string(dir.join("h.ac-link.txt")).unwrap();
    assert!(receipt.contains("runtime-lib: "));
    assert!(receipt.contains("runtime-contract-sha256: "));
}

// -- V-03: char conversions ---------------------------------------------------

#[test]
fn v03_char_roundtrip_and_eq() {
    let dir = temp_dir("v03-char");
    let src = write_src(
        &dir,
        "r.at",
        r#"fn cp(c char) int { return c.to_codepoint() }
fn mk(n int) char { return char.from_codepoint(n) }
fn main() int {
    let c char = mk(20013)
    if c != '中' { return 1 }
    let a char = 'A'
    return cp(a) + cp(c) - 20078
}
"#,
    );
    let exe = dir.join("r.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 0, "65 + 20013 - 20078 == 0");
    // from_codepoint traces FROM-CP; to_codepoint is a value passthrough
    // (no trace on the success path)
    assert_eq!(run.stderr, "FROM-CP cp=20013\n");
}

#[test]
fn v03_invalid_scalar_traps_70() {
    // 0xD800 is a surrogate: a runtime rejection, not a verify error
    let dir = temp_dir("v03-surr");
    let src = write_src(
        &dir,
        "s.at",
        r#"fn main() int {
    let c char = char.from_codepoint(55296)
    return c.to_codepoint()
}
"#,
    );
    let exe = dir.join("s.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 70, "invalid scalar is the trap70 exit");
    assert_eq!(
        run.stderr, "FROM-CP cp=55296\nRT-FAIL code=-8\n",
        "trap evidence = RT-FAIL line + exit 70 (design §3.4)"
    );
}

#[test]
fn v03_codepoint_upper_bound() {
    let dir = temp_dir("v03-max");
    let src = write_src(
        &dir,
        "m.at",
        r#"fn main() int {
    let c char = char.from_codepoint(1114111)
    if c.to_codepoint() != 1114111 { return 1 }
    let d char = char.from_codepoint(1114112)
    return d.to_codepoint()
}
"#,
    );
    let exe = dir.join("m.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(
        run.code, 70,
        "U+10FFFF is the last valid scalar; one past rejects"
    );
    assert!(
        run.stderr.contains("RT-FAIL code=-8"),
        "stderr:\n{}",
        run.stderr
    );
}

// -- V-04: lifecycle observability --------------------------------------------

#[test]
fn v04_lifecycle_traces_and_move_transfer() {
    let dir = temp_dir("v04-life");
    let src = write_src(
        &dir,
        "l.at",
        r#"fn consume(move o Str) int { return o.len_bytes() }
fn main() int {
    let x Str = "xyz"
    let a int = consume(x.move)
    let z Str = "ab" + "cd"
    let w Str = z.clone()
    return a + w.len() + z.len()
}
"#,
    );
    let exe = dir.join("l.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 11, "3 + 4 + 4");
    // exact oracle: promotion allocates id=1 and the CALLEE frees it at its
    // return (ownership moved); the final local frees are LIFO (w then z)
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nTEXT-BYTES n=3\nFREE id=1\n\
         CONCAT a=2 b=2\nCONCAT id=2\nCLONE len=4\nCLONE id=3\n\
         TEXT-SCALARS n=4\nTEXT-SCALARS n=4\nFREE id=3\nFREE id=2\n",
        "every allocation has exactly one free; ids pair 1:1"
    );
}

#[test]
fn v04_cross_frame_owned_return() {
    let dir = temp_dir("v04-ret");
    let src = write_src(
        &dir,
        "x.at",
        r#"fn make() Str { return "hello" }
fn eqs(a str, b str) bool { return a == b }
fn main() int {
    let o Str = make()
    let v str = o.to(str)
    if eqs(v, "hello") { return o.len() }
    return 1
}
"#,
    );
    let exe = dir.join("x.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 5, "owned return transfers to the caller");
    // id=1 allocated in make() (ABI-C return channel), freed at main's exit
    assert_eq!(
        run.stderr,
        "LIT len=5\nLIT id=1\nTEXT-SCALARS n=5\nFREE id=1\n"
    );
}

#[test]
fn v04_borrow_never_frees() {
    let dir = temp_dir("v04-view");
    let src = write_src(
        &dir,
        "b.at",
        r#"fn main() int {
    let o Str = "abcdef"
    let v str = o.to(str)
    let n int = v.slice(2, 5).len()
    if n != 3 { return 1 }
    return o.len() + n
}
"#,
    );
    let exe = dir.join("b.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(
        run.code, 9,
        "o.len()=6 + n=3; the view's uses stay inside the owner's scope"
    );
    // exactly ONE allocation (the promotion) and one free: views never free
    assert_eq!(
        run.stderr,
        "LIT len=6\nLIT id=1\nSLICE s=2 e=5 len=3\nTEXT-SCALARS n=3\nTEXT-SCALARS n=6\nFREE id=1\n"
    );
}

// -- V-07: evaluation order ---------------------------------------------------
//
// Recorded boundary: combining the scalar trace support library with the
// text runtime in ONE PE does not link — both are Rust no_std staticlibs
// and each carries `__rustc::rust_begin_unwind` (duplicate symbol at
// rust-lld). The mark_a/mark_b×text combination therefore has no first-
// batch PE face; the order oracle below uses the text runtime's OWN trace
// lines (every ac_rt_* call is ordered evidence), which prove the same
// invariants: left→right argument evaluation, name-based parameter mapping
// (no reorder), and a short-circuited RHS that never runs (its allocation
// never happens). The mark_a/b order legs themselves stay covered by the
// scalar suites (772/767).

#[test]
fn v07_arg_order_and_short_circuit_with_text() {
    let dir = temp_dir("v07-order");
    // lhs→rhs positional args: the lhs's concat + count run first
    let base = write_src(
        &dir,
        "a.at",
        r#"fn pair(a int, b int) int { return a + b }
fn main() int {
    let n int = pair(("ab" + "cd").len(), "a".len())
    return n
}
"#,
    );
    let exe = dir.join("a.exe");
    build_text(&base, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 5);
    assert_eq!(
        run.stderr, "CONCAT a=2 b=2\nCONCAT id=1\nTEXT-SCALARS n=4\nFREE id=1\nTEXT-SCALARS n=1\n",
        "lhs argument fully evaluated (concat + count + temp free) before the rhs count"
    );

    // named args map by NAME in the callee, evaluation stays left→right:
    // b evaluates first (n=3), a second (n=1); the callee sums a+b either way
    let named = write_src(
        &dir,
        "n.at",
        r#"fn diff(a int, b int) int { return b - a }
fn main() int {
    return diff(a: "a".len(), b: "abc".len())
}
"#,
    );
    let exe = dir.join("n.exe");
    build_text(&named, &exe);
    let run = run_pe(&exe);
    assert_eq!(
        run.code, 2,
        "b=3 - a=1: named mapping, not positional reorder"
    );
    assert_eq!(
        run.stderr, "TEXT-SCALARS n=1\nTEXT-SCALARS n=3\n",
        "evaluation order (a first, b second) is independent of the mapping"
    );

    // eq operand order: lhs count before rhs count
    let eqsrc = write_src(
        &dir,
        "e.at",
        r#"fn main() int {
    if "abc" == "abcd" { return 1 }
    return 0
}
"#,
    );
    let exe = dir.join("e.exe");
    build_text(&eqsrc, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 0);
    assert_eq!(
        run.stderr, "",
        "eq itself is trace-free; length mismatch short-circuits"
    );

    // short-circuit: the rhs (with its allocation) never runs
    let sc = write_src(
        &dir,
        "s.at",
        r#"fn rhs() bool { let t Str = "ab" + "cd"; return t.len() == 4 }
fn main() int {
    let r = 1 < 0 && rhs()
    if r { return 1 }
    let t Str = "xy" + "zw"
    if 0 < 1 && t.len() == 4 { return 0 }
    return 2
}
"#,
    );
    let exe = dir.join("s.exe");
    build_text(&sc, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 0, "the executed && row runs its rhs");
    assert_eq!(
        run.stderr, "CONCAT a=2 b=2\nCONCAT id=1\nTEXT-SCALARS n=4\nFREE id=1\n",
        "no trace of rhs()'s concat: the skipped side neither evaluated nor allocated"
    );
}

// -- V-09: scale ---------------------------------------------------------------

#[test]
fn v09_concat_chain_4096() {
    let dir = temp_dir("v09-chain");
    let src = write_src(
        &dir,
        "chain.at",
        &format!(
            "fn main() int {{\n    let s Str = {}\n    return s.len()\n}}\n",
            vec!["\"a\""; 4096].join(" + ")
        ),
    );
    let exe = dir.join("chain.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert!(!run.timed_out);
    assert_eq!(run.code, 4096, "full 32-bit exit: 4096, not 0 mod 256");
    let lines: Vec<&str> = run.stderr.lines().collect();
    let concats = lines.iter().filter(|l| l.starts_with("CONCAT id")).count();
    let frees = lines.iter().filter(|l| l.starts_with("FREE id")).count();
    assert_eq!(concats, 4095, "4095 temporaries + final, each freed once");
    assert_eq!(frees, 4095);
    assert_eq!(lines.last().copied(), Some("FREE id=4095"));
}

#[test]
fn v09_resource_loop_4096() {
    let dir = temp_dir("v09-loop");
    let src = write_src(
        &dir,
        "loop.at",
        r#"fn main() int {
    var total = 0
    var i = 0
    for i < 4096 {
        let s Str = "ab" + "cd"
        total = total + s.len()
        i = i + 1
    }
    return total
}
"#,
    );
    let exe = dir.join("loop.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 16384, "4096 iterations x 4");
    let lines: Vec<&str> = run.stderr.lines().collect();
    let concats = lines.iter().filter(|l| l.starts_with("CONCAT id")).count();
    let frees = lines.iter().filter(|l| l.starts_with("FREE id")).count();
    assert_eq!(concats, 4096, "one allocation per iteration");
    assert_eq!(
        frees, 4096,
        "cleaned at the body's natural end, every iteration"
    );
    assert_eq!(lines.last().copied(), Some("FREE id=4096"));
}

#[test]
fn v09_call_chain_4096_with_text_args() {
    let dir = temp_dir("v09-call");
    // deep call chain (compile-safety leg): the innermost view param rides
    // 4096 frames back out through view parameters
    let depth = 4096usize;
    let mut inner = String::from("fn id0(v str) int { return v.len() }\n");
    for i in 1..=depth {
        inner.push_str(&format!(
            "fn id{i}(v str) int {{ return id{}(v) }}\n",
            i - 1
        ));
    }
    inner.push_str(&format!(
        "fn main() int {{\n    let s str = \"abc\"\n    return id{depth}(s)\n}}\n"
    ));
    let src = write_src(&dir, "call.at", &inner);
    let exe = dir.join("call.exe");
    build_text(&src, &exe);
    let run = run_pe(&exe);
    assert_eq!(run.code, 3, "the borrowed view crosses 4096 frames");
    assert_eq!(run.stderr, "TEXT-SCALARS n=3\n");
}

// -- V-10: contract gate + isolation -------------------------------------------

#[test]
fn v10_contract_gate_rejections() {
    let dir = temp_dir("v10-gate");
    let lib = common::text_runtime_lib();
    let (lib2, contract2) = common::text_runtime_pair_copy(&dir);
    let sha = common::text_runtime_sha();
    let src = write_src(
        &dir,
        "g.at",
        "fn main() int {\n    let s str = \"ab\"\n    return s.len()\n}\n",
    );
    let exe = dir.join("g.exe");

    // helper: build with a given support-lib list; returns (code, stderr)
    let try_build = |libs: &[PathBuf]| -> common::DeadlinedRun {
        let mut c = Command::new(EXE);
        c.args([
            "build-source",
            &src.display().to_string(),
            "--source-profile",
            TEXT,
            "--entry",
            "main",
            "--capability",
            TEXT_CAP,
            "--output",
        ])
        .arg(&exe);
        for l in libs {
            c.arg("--support-lib").arg(l);
        }
        run_deadlined(c, Duration::from_secs(600)).expect("spawn build")
    };

    // capability missing → capability.missing (before any lib consideration)
    {
        let mut c = Command::new(EXE);
        c.args([
            "build-source",
            &src.display().to_string(),
            "--source-profile",
            TEXT,
            "--entry",
            "main",
            "--support-lib",
        ])
        .arg(&lib)
        .arg("--output")
        .arg(&exe);
        let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
        assert_eq!(b.code, 1, "capability gate first");
        assert!(b.stderr.contains("capability.missing"), "{}", b.stderr);
    }

    // valid pair passes (control)
    let b = try_build(&[lib2.clone()]);
    assert_eq!(b.code, 0, "valid contract builds: {}", b.stderr);

    // no sidecar → runtime.contract-missing
    {
        let dir2 = dir.join("noside");
        std::fs::create_dir_all(&dir2).unwrap();
        let bare = dir2.join("rt.lib");
        std::fs::copy(&lib, &bare).unwrap();
        let b = try_build(&[bare]);
        assert_eq!(b.code, 1);
        assert!(
            b.stderr.contains("runtime.contract-missing"),
            "{}",
            b.stderr
        );
    }

    // wrong hash → runtime.contract-hash
    common::write_sidecar(
        &contract2,
        "ac.runtime.text.v1",
        1,
        "x86_64-pc-windows-msvc",
        "deadbeef",
    );
    let b = try_build(&[lib2.clone()]);
    assert_eq!(b.code, 1);
    assert!(b.stderr.contains("runtime.contract-hash"), "{}", b.stderr);

    // wrong api → runtime.contract-api
    common::write_sidecar(
        &contract2,
        "ac.runtime.text.v9",
        1,
        "x86_64-pc-windows-msvc",
        &sha,
    );
    let b = try_build(&[lib2.clone()]);
    assert_eq!(b.code, 1);
    assert!(b.stderr.contains("runtime.contract-api"), "{}", b.stderr);

    // wrong revision → runtime.contract-api
    common::write_sidecar(
        &contract2,
        "ac.runtime.text.v1",
        2,
        "x86_64-pc-windows-msvc",
        &sha,
    );
    let b = try_build(&[lib2.clone()]);
    assert_eq!(b.code, 1);
    assert!(b.stderr.contains("runtime.contract-api"), "{}", b.stderr);

    // wrong target → runtime.contract-target
    common::write_sidecar(
        &contract2,
        "ac.runtime.text.v1",
        1,
        "x86_64-unknown-linux-gnu",
        &sha,
    );
    let b = try_build(&[lib2.clone()]);
    assert_eq!(b.code, 1);
    assert!(b.stderr.contains("runtime.contract-target"), "{}", b.stderr);

    // two contract carriers → ambiguous → runtime.contract-missing
    common::write_sidecar(
        &contract2,
        "ac.runtime.text.v1",
        1,
        "x86_64-pc-windows-msvc",
        &sha,
    );
    let dir3 = dir.join("second");
    std::fs::create_dir_all(&dir3).unwrap();
    let (lib3, contract3) = common::text_runtime_pair_copy(&dir3);
    let _ = contract3;
    let b = try_build(&[lib2.clone(), lib3]);
    assert_eq!(b.code, 1);
    assert!(
        b.stderr.contains("runtime.contract-missing"),
        "{}",
        b.stderr
    );
}

#[test]
fn v10_rejecting_build_preserves_sentinels() {
    let dir = temp_dir("v10-sentinel");
    let exe = dir.join("app.exe");
    let obj = dir.join("app.obj");
    let receipt = dir.join("app.ac-link.txt");
    let src = write_src(
        &dir,
        "ok.at",
        "fn main() int {\n    let s str = \"ok\"\n    return s.len()\n}\n",
    );
    // a successful build produces all three artifacts
    build_text(&src, &exe);
    assert!(obj.is_file() && receipt.is_file());

    const SENTINEL: &[u8] = b"PLAN-770-SENTINEL-PRIOR-ARTIFACT";
    std::fs::write(&exe, SENTINEL).unwrap();
    std::fs::write(&obj, SENTINEL).unwrap();
    std::fs::write(&receipt, SENTINEL).unwrap();
    let before = (
        std::fs::read(&exe).unwrap(),
        std::fs::read(&obj).unwrap(),
        std::fs::read(&receipt).unwrap(),
    );

    // a VERIFY-rejected text build must not touch them
    let bad = write_src(
        &dir,
        "bad.at",
        "fn consume(move o Str) int { return o.len_bytes() }\nfn main() int {\n    let x Str = \"q\"\n    let a int = consume(x.move)\n    return a + x.len()\n}\n",
    );
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &bad.display().to_string(),
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
    assert_eq!(b.code, 1, "use-after-move must reject");
    assert!(b.stderr.contains("verify.use-after-move"), "{}", b.stderr);
    let after = (
        std::fs::read(&exe).unwrap(),
        std::fs::read(&obj).unwrap(),
        std::fs::read(&receipt).unwrap(),
    );
    assert_eq!(before, after, "the publish transaction preserved all three");
}

#[test]
fn v10_scalar_surface_needs_no_runtime() {
    // a text-profile program with zero text ops requires no capability and
    // links without the runtime (profile isolation, V-01/V-10 face)
    let dir = temp_dir("v10-scalar");
    let src = write_src(
        &dir,
        "s.at",
        "fn gcd(a int, b int) int {\n    if b == 0 { return a }\n    return gcd(b, a % b)\n}\nfn main() int { return gcd(252, 105) }\n",
    );
    let exe = dir.join("s.exe");
    let mut c = Command::new(EXE);
    c.args([
        "build-source",
        &src.display().to_string(),
        "--source-profile",
        TEXT,
        "--entry",
        "main",
        "--output",
    ])
    .arg(&exe);
    let b = run_deadlined(c, Duration::from_secs(600)).expect("spawn build");
    assert_eq!(b.code, 0, "no capability, no support lib: {}", b.stderr);
    let run = run_pe(&exe);
    assert_eq!(run.code, 21, "gcd(252,105)=21");
    assert!(run.stderr.is_empty(), "no runtime traces without text ops");
}
