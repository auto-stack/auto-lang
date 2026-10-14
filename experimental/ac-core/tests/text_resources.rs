//! PLAN-770 T-05 text resource/failure tests — the PE legs of V-05 plus
//! resource-boundary observations that only real execution can make.
//!
//! Failure oracle (design §3.4): every runtime rejection crosses back as a
//! negative status the runtime itself already traced (`RT-FAIL code=<n>` on
//! stderr); the generated code then exits 70. Distinct failure classes are
//! distinguished by their code — a plain `return 70` never counts as a trap
//! (the RT-FAIL line is the trap evidence).
//!
//! Recorded boundary (plan V-05): budget exhaustion (`ac_rt_set_budget`) and
//! allocation fault injection (`ac_rt_fail_nth_alloc`) have no language-level
//! entry point in the first batch — those faces are owned by the runtime
//! workspace's own tests (`runtime/text/tests/fault.rs`); here the reachable
//! language faces are the range/scalar failure classes and the
//! allocation-free lifetime observations.

#![cfg(windows)]

mod common;

use common::run_deadlined;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const TEXT: &str = "auto.source.text.draft";
const TEXT_CAP: &str = "ac.runtime.text.v1";

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("ac770-text-res").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Build a text source program; returns the PE run outcome directly.
fn build_and_run(case: &str, src: &str) -> common::DeadlinedRun {
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

/// Assert the trap70 oracle: exit 70 and the exact RT-FAIL line present.
fn assert_trap(case: &str, run: &common::DeadlinedRun, code: i32, prefix: &str) {
    assert_eq!(
        run.code, 70,
        "{case}: expected the trap70 exit (RT-FAIL code={code}); stderr:\n{}",
        run.stderr
    );
    let needle = format!("RT-FAIL code={code}");
    assert!(
        run.stderr.lines().any(|l| l.starts_with(&needle)),
        "{case}: stderr must carry `{needle}`; got:\n{}",
        run.stderr
    );
    // a normal return 70 is NOT a trap: the RT-FAIL line is the evidence
    assert!(
        run.stderr.lines().any(|l| l.starts_with(prefix)),
        "{case}: failure-class prefix `{prefix}` expected on stderr:\n{}",
        run.stderr
    );
}

// -- V-05: range failure classes ------------------------------------------------

#[test]
fn v05_slice_end_beyond_len() {
    let run = build_and_run(
        "slice-oob",
        "fn main() int {\n    let s str = \"ab\"\n    let v str = s.slice(1, 5)\n    return v.len()\n}\n",
    );
    assert_trap("slice end > len", &run, -10, "RT-FAIL");
    // the SLICE success trace must NOT appear (rejection precedes it)
    assert!(!run.stderr.contains("SLICE s="), "{}", run.stderr);
}

#[test]
fn v05_slice_reversed() {
    let run = build_and_run(
        "slice-rev",
        "fn main() int {\n    let s str = \"abc\"\n    let v str = s.slice(2, 1)\n    return v.len()\n}\n",
    );
    assert_trap("slice start > end", &run, -9, "RT-FAIL");
}

#[test]
fn v05_slice_negative_start() {
    let run = build_and_run(
        "slice-neg",
        "fn main() int {\n    let n int = 0 - 1\n    let s str = \"abc\"\n    let v str = s.slice(n, 2)\n    return v.len()\n}\n",
    );
    assert_trap("slice negative bound", &run, -10, "RT-FAIL");
}

#[test]
fn v05_char_at_index_out_of_range() {
    // the frozen runtime maps an out-of-range scalar index to the range
    // failure class (-10): char_at has no separate index code on this ABI
    let run = build_and_run(
        "charat-oob",
        "fn main() int {\n    let s str = \"A中😀\"\n    let c char = s.char_at(3)\n    return c.to_codepoint()\n}\n",
    );
    assert_trap("char_at index == len", &run, -10, "RT-FAIL");
}

#[test]
fn v05_char_at_negative_index() {
    let run = build_and_run(
        "charat-neg",
        "fn main() int {\n    let n int = 0 - 2\n    let s str = \"ab\"\n    let c char = s.char_at(n)\n    return c.to_codepoint()\n}\n",
    );
    assert_trap("char_at negative index", &run, -10, "RT-FAIL");
}

#[test]
fn v05_char_from_codepoint_surrogate_and_negative() {
    let run = build_and_run(
        "cp-surr",
        "fn main() int {\n    let c char = char.from_codepoint(55423)\n    return c.to_codepoint()\n}\n",
    );
    assert_trap("surrogate scalar", &run, -8, "FROM-CP");

    let run = build_and_run(
        "cp-neg",
        "fn main() int {\n    let n int = 0 - 1\n    let c char = char.from_codepoint(n)\n    return c.to_codepoint()\n}\n",
    );
    assert_trap("negative scalar", &run, -8, "FROM-CP");
}

#[test]
fn v05_trap_does_not_run_later_statements() {
    // the trap path exits at the failing slice; the later concat must not
    // allocate (no CONCAT trace on stderr)
    let run = build_and_run(
        "trap-order",
        "fn main() int {\n    let s str = \"ab\"\n    let v str = s.slice(0, 9)\n    let t Str = \"x\" + \"y\"\n    return v.len() + t.len()\n}\n",
    );
    assert_trap("trap precedes later allocations", &run, -10, "RT-FAIL");
    assert!(
        !run.stderr.contains("CONCAT"),
        "nothing after the trap may run; stderr:\n{}",
        run.stderr
    );
}

// -- boundary observations -------------------------------------------------------

#[test]
fn zero_length_owned_round_trip() {
    // an owned promotion of the empty literal allocates nothing (null+0 is
    // the valid empty representation) and frees nothing
    let run = build_and_run(
        "zero-owned",
        "fn main() int {\n    let o Str = \"\"\n    let c Str = o.clone()\n    if c == o { return o.len() + c.len() }\n    return 1\n}\n",
    );
    assert_eq!(run.code, 0, "empty owned values compare equal");
    assert_eq!(
        run.stderr, "LIT len=0\nCLONE len=0\nTEXT-SCALARS n=0\nTEXT-SCALARS n=0\n",
        "zero-length allocations make no allocator call (no FREE lines, no ids)"
    );
}

#[test]
fn lifo_order_two_locals_branch_join() {
    // both arms bind one owned local each; the join's frees stay LIFO by
    // declaration order, one allocation per arm on the taken path
    let run = build_and_run(
        "lifo-join",
        "fn main() int {\n    let pick int = 1\n    if pick == 1 {\n        let a Str = \"aaaa\"\n        return a.len()\n    }\n    let b Str = \"bb\"\n    return b.len()\n}\n",
    );
    assert_eq!(run.code, 4);
    assert_eq!(
        run.stderr,
        "LIT len=4\nLIT id=1\nTEXT-SCALARS n=4\nFREE id=1\n"
    );
}

#[test]
fn moved_value_not_freed_by_caller() {
    // the caller moves x into consume; only ONE free exists in the whole
    // run and it happens inside the callee (before the caller's later
    // allocation gets its own id — ids prove no double free)
    let run = build_and_run(
        "move-once",
        "fn consume(move o Str) int { return o.len_bytes() }\nfn main() int {\n    let x Str = \"xyz\"\n    let a int = consume(x.move)\n    let y Str = \"q\"\n    return a + y.len()\n}\n",
    );
    assert_eq!(run.code, 4, "3 + 1");
    assert_eq!(
        run.stderr,
        "LIT len=3\nLIT id=1\nTEXT-BYTES n=3\nFREE id=1\nLIT len=1\nLIT id=2\nTEXT-SCALARS n=1\nFREE id=2\n",
        "id=1 freed exactly once (callee), then id=2's full lifecycle"
    );
}
