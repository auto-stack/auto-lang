//! PLAN-926 CA-C (T-03/T-04): the C collections TRUE-CHAIN conformance —
//! source → checked collections frontend (with the verifier's
//! CollectionsBodyFacts) → shared semantic view → C provider (AXC3) →
//! session assembly (store-persisted fragments, nominal type-table seed)
//! → REAL MSVC cl /std:c11 compile → real run → parent-side oracle.
//!
//! The first frozen 904 originals (byte-exact includes) run against the
//! frozen oracle table — the same values the Rust leg pins
//! (`collections_904`), INDEPENDENTLY executed through the C channel:
//! L01=42, L04=42, L05=42, V01=42, V02=42, V03=3, LSTR864=51,
//! N889-nested=42; L06 traps `ListIndex` (the ONLY cause — never value
//! 70, an abort, or any other nonzero). The resource knobs run through
//! the REAL runtime: mid-allocation fault = typed `Resource` (the
//! uncommitted input dies exactly once — the event ledger stays
//! balanced), tight budget = `Budget`, armed cancel at the loop poll =
//! `Cancelled`. File-set identity carries the frozen
//! `auto.a2x.c.collections.fileset/1` schema and the AXC3 layer.
//!
//! A missing MSVC toolchain is an INFRA failure (never a skip); compile
//! errors are infra facts, never Auto diagnostics/traps.

use ace_a2x_c::{CTargetProvider, C_COLL_FILESET_SCHEMA};
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;
use std::path::PathBuf;
use std::process::Command;

const L01: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/L01.at"
);
const L02: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/L02.at"
);
const L03: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/L03.at"
);
const L04: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/L04.at"
);
const L05: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/L05.at"
);
const L06: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/L06.at"
);
const L07: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/L07.at"
);
const V01: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/V01.at"
);
const V02: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/V02.at"
);
const V03: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/V03.at"
);
const V04: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/V04.at"
);
const V05: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/V05.at"
);
const V06: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/V06.at"
);
const V07: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/V07.at"
);
const LSTR864: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/LSTR864.at"
);
const FIN864A: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/FIN864A.at"
);
const FIN864B: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/FIN864B.at"
);
const N889_LIST: &str = include_str!("../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/N889-list-variant.at");
const N889_NESTED: &str = include_str!("../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/N889-nested-variant.at");
const N889_OPTIONAL: &str = include_str!("../../../../../../crates/auto-lang/test/cases/a2r-hir-collections-904/src/N889-optional-variant.at");

/// A for-in loop over an int list (the poll sits on the desugared loop's
/// back-edge; per-iteration scoped bindings free at the back-edge).
const LOOP_SUM: &str = "fn main() int {\n    var xs = List<int>.new()\n    xs.push(1)\n    xs.push(2)\n    var total int = 0\n    for x in xs {\n        total = total + x\n    }\n    return total\n}";

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c-coll")
}

fn store_root() -> PathBuf {
    // fresh store PER PROCESS (fragment identities key on the frozen
    // target identity; a reused store across code changes would serve
    // stale bytes)
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/a2x-out-c-coll/store-{}",
        std::process::id()
    ))
}

/// Open one C collections session on a per-test store namespace
/// (SQLite single-writer: tests run in parallel threads).
fn open_session(test: &str) -> A2xSession {
    A2xSession::open(
        &store_root().join(test),
        "a2c-coll",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .expect("open C coll session")
}

/// Emit + compile + run one case through the C session chain; returns
/// (stdout, stderr) of the RUN.
fn run_c_chain(
    s: &mut A2xSession,
    name: &str,
    src: &str,
    env: &[(&str, &str)],
) -> (String, String) {
    let asm = s
        .cold_assemble_profile(src, SourceProfile::Collections, &[], "main", false)
        .unwrap_or_else(|e| panic!("C coll assembly failed for {}: {:?}", name, e));
    let dir = out_root().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
    let exe = dir.join("a2c_coll_case.exe");
    ace_a2x_worker::crun::compile_c_fileset(&dir, &exe)
        .unwrap_or_else(|e| panic!("cl failed for {}: {}", name, e));
    let mut command = Command::new(&exe);
    command.current_dir(&dir);
    for (k, v) in env {
        command.env(k, v);
    }
    let run = auto_tool_execution::run_command_capture(
        command,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run exe");
    assert_eq!(
        run.kind,
        auto_tool_execution::ExitKind::Success,
        "wrapper exit non-zero for {}",
        name
    );
    (
        String::from_utf8_lossy(run.stdout.as_ref().expect("captured stdout")).to_string(),
        String::from_utf8_lossy(run.stderr.as_ref().expect("captured stderr")).to_string(),
    )
}

fn assert_value(name: &str, out: &str, expected: i32) {
    assert_eq!(
        out,
        format!("value:{expected}\n"),
        "stdout mismatch for {name}"
    );
}

fn assert_trap(name: &str, out: &str, trap: &str) {
    assert_eq!(out, format!("trap:{trap}\n"), "stdout mismatch for {name}");
}

/// Count `coll:alloc:<n>` / `coll:free-backing:<n>` event lines of the
/// run's stderr and their byte totals.
fn ledger(stderr: &str) -> (usize, u64, usize, u64) {
    let mut allocs = 0usize;
    let mut alloc_bytes = 0u64;
    let mut frees = 0usize;
    let mut free_bytes = 0u64;
    for line in stderr.lines() {
        if let Some(rest) = line.strip_prefix("coll:alloc:") {
            allocs += 1;
            alloc_bytes += rest.parse::<u64>().unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("coll:free-backing:") {
            frees += 1;
            free_bytes += rest.parse::<u64>().unwrap_or(0);
        }
    }
    (allocs, alloc_bytes, frees, free_bytes)
}

/// V-01 first cut: ALL TWENTY frozen 904 originals through the C
/// channel against the frozen oracle table — the same values the Rust
/// leg pins, independently executed here (L06 traps in its own test).
#[test]
fn a2c_coll_required_values() {
    let cases: &[(&str, &str, i32)] = &[
        ("L01", L01, 42),
        ("L02", L02, 42),
        ("L03", L03, 0),
        ("L04", L04, 42),
        ("L05", L05, 42),
        ("L07", L07, 42),
        ("V01", V01, 42),
        ("V02", V02, 42),
        ("V03", V03, 3),
        ("V04", V04, 42),
        ("V05", V05, 42),
        ("V06", V06, 42),
        ("V07", V07, 42),
        ("LSTR864", LSTR864, 51),
        ("FIN864A", FIN864A, 42),
        ("FIN864B", FIN864B, 53),
        ("N889-list", N889_LIST, 42),
        ("N889-nested", N889_NESTED, 42),
        ("N889-optional", N889_OPTIONAL, 42),
    ];
    let mut s = open_session("required_values");
    for (name, src, value) in cases {
        let (out, err) = run_c_chain(&mut s, name, src, &[("A2X_RT_EVENTS", "1")]);
        assert_value(name, &out, *value);
        // the event ledger closes on EVERY case: every real allocation
        // released exactly once (leak / double-free counterexample)
        let (allocs, alloc_bytes, frees, free_bytes) = ledger(&err);
        assert_eq!(allocs, frees, "alloc/free count unbalanced for {name}");
        assert_eq!(
            alloc_bytes, free_bytes,
            "ledger bytes unbalanced for {name}"
        );
    }
}

/// L06: the out-of-bounds get traps the typed `ListIndex` — never a
/// value, never an abort.
#[test]
fn a2c_coll_list_index_trap() {
    let mut s = open_session("list_index_trap");
    let (out, _err) = run_c_chain(&mut s, "L06", L06, &[]);
    assert_trap("L06", &out, "ListIndex");
}

/// The loop back-edge poll: an armed cancellation observes at the FIRST
/// poll as the typed `Cancelled` trap; without the knob the same source
/// sums to its value.
#[test]
fn a2c_coll_loop_poll_cancel() {
    let mut s = open_session("loop_poll_cancel");
    let (out, _err) = run_c_chain(&mut s, "LOOP_SUM", LOOP_SUM, &[("A2X_RT_CANCEL", "1")]);
    assert_trap("LOOP_SUM-cancel", &out, "Cancelled");
    let (out, _err) = run_c_chain(&mut s, "LOOP_SUM-run", LOOP_SUM, &[]);
    assert_value("LOOP_SUM", &out, 3);
}

/// A tight budget turns the next charge into the typed `Budget` trap
/// (charged BEFORE the work — nothing partial commits).
#[test]
fn a2c_coll_budget_trap() {
    let mut s = open_session("budget_trap");
    let (out, _err) = run_c_chain(&mut s, "L04-budget", L04, &[("A2X_RT_BUDGET", "10")]);
    assert_trap("L04-budget", &out, "Budget");
}

/// The mid-allocation fault: the n-th REAL allocation fails as the typed
/// `Resource` trap with the uncommitted input freed exactly once — the
/// event ledger's alloc/free-backing byte totals stay BALANCED (a leak
/// or a double free would unbalance them). L05 has exactly two
/// allocation sites (the list base + the "hello" literal promotion), so
/// a fault ordinal beyond them is the NO-INJECTION control (the program
/// completes).
#[test]
fn a2c_coll_fault_resource_and_ledger_balance() {
    let mut s = open_session("fault_ledger");
    for n in [1usize, 2] {
        let (out, err) = run_c_chain(
            &mut s,
            &format!("L05-fault{n}"),
            L05,
            &[
                ("A2X_RT_FAULT_ALLOC", &n.to_string()),
                ("A2X_RT_EVENTS", "1"),
            ],
        );
        assert_trap(&format!("L05-fault{n}"), &out, "Resource");
        let (allocs, alloc_bytes, frees, free_bytes) = ledger(&err);
        assert_eq!(
            alloc_bytes, free_bytes,
            "ledger unbalanced at fault {n}: allocs {allocs}/{alloc_bytes}B frees {frees}/{free_bytes}B"
        );
    }
    // N+1 / far-beyond: no allocation to fail — the run completes and
    // its ledger still closes
    for n in [3usize, 9] {
        let (out, err) = run_c_chain(
            &mut s,
            &format!("L05-fault{n}"),
            L05,
            &[
                ("A2X_RT_FAULT_ALLOC", &n.to_string()),
                ("A2X_RT_EVENTS", "1"),
            ],
        );
        assert_value(&format!("L05-fault{n}"), &out, 42);
        let (allocs, alloc_bytes, frees, free_bytes) = ledger(&err);
        assert_eq!(allocs, frees);
        assert_eq!(alloc_bytes, free_bytes);
    }
    // the happy path's ledger closes too (every allocation released)
    let (out, err) = run_c_chain(&mut s, "L05-events", L05, &[("A2X_RT_EVENTS", "1")]);
    assert_value("L05-events", &out, 42);
    let (allocs, alloc_bytes, frees, free_bytes) = ledger(&err);
    assert_eq!(allocs, frees);
    assert_eq!(alloc_bytes, free_bytes);
}

/// File-set identity: the collections manifest carries the frozen
/// schema; the fragment layer is AXC3 (the store's layer registry), not
/// the Rust AXR3 or the 921/925 C layers.
#[test]
fn a2c_coll_fileset_identity() {
    let mut s = open_session("fileset_identity");
    let asm = s
        .cold_assemble_profile(L01, SourceProfile::Collections, &[], "main", false)
        .expect("assembly");
    assert!(asm.manifest.contains(C_COLL_FILESET_SCHEMA));
    assert!(asm.manifest.contains("\"entry\": \"main\""));
    // determinism: the same session assembles byte-identical files
    let asm2 = s
        .cold_assemble_profile(L01, SourceProfile::Collections, &[], "main", false)
        .expect("assembly 2");
    assert_eq!(asm.files, asm2.files);
    assert_eq!(asm.manifest, asm2.manifest);
}

/// V-04 (fault atomicity matrix): EVERY allocation ordinal of a
/// multi-site case (list base + literal promotions + clone + growth) —
/// faulting each one yields the typed `Resource` trap with a BALANCED
/// event ledger (every real allocated BYTE released exactly once; a leak
/// or double free would unbalance it — counts may legitimately differ
/// when a growth `realloc` merges the base and the delta into one
/// release); N+1 and the no-knob control complete with the oracle value.
/// The allocation count is SELF-CALIBRATED from the event ledger of the
/// clean run (no brittle hand-counted ordinals).
#[test]
fn a2c_coll_fault_matrix_every_allocation() {
    // allocation sites: list base, literal promotions, clone, growth
    // (cap 4 -> 8 at the 5th push)
    const SRC: &str = "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"alpha\", 1))\n    var s Str = \"beta\"\n    xs.push(Item(s.clone(), 2))\n    xs.push(Item(\"gamma\", 3))\n    xs.push(Item(\"delta\", 4))\n    xs.push(Item(\"eps\", 5))\n    return xs.get(0).code + xs.get(4).code + xs.len()\n}";
    let mut s = open_session("fault_matrix");
    // clean run: oracle + ledger closes (also calibrates N)
    let (out, err) = run_c_chain(&mut s, "FAULT_MATRIX-clean", SRC, &[("A2X_RT_EVENTS", "1")]);
    assert_value("FAULT_MATRIX-clean", &out, 1 + 5 + 5);
    let (allocs, alloc_bytes, frees, free_bytes) = ledger(&err);
    assert_eq!(alloc_bytes, free_bytes, "clean-run ledger bytes must close");
    let _ = frees;
    let n = allocs;
    assert!(n >= 5, "expected several allocation sites, ledger says {n}");
    // every ordinal faults cleanly
    for k in 1..=n {
        let (out, err) = run_c_chain(
            &mut s,
            &format!("FAULT_MATRIX-f{k}"),
            SRC,
            &[
                ("A2X_RT_FAULT_ALLOC", &k.to_string()),
                ("A2X_RT_EVENTS", "1"),
            ],
        );
        assert_trap(&format!("FAULT_MATRIX-f{k}"), &out, "Resource");
        let (a, ab, f, fb) = ledger(&err);
        assert_eq!(
            ab, fb,
            "ledger unbalanced at fault {k}/{n}: allocs {a}/{ab}B frees {f}/{fb}B"
        );
    }
    // N+1: no allocation to fail — completes
    let (out, err) = run_c_chain(
        &mut s,
        "FAULT_MATRIX-n1",
        SRC,
        &[
            ("A2X_RT_FAULT_ALLOC", &(n + 1).to_string()),
            ("A2X_RT_EVENTS", "1"),
        ],
    );
    assert_value("FAULT_MATRIX-n1", &out, 11);
    let (_a, ab, _f, fb) = ledger(&err);
    assert_eq!(ab, fb);
}

/// V-05 (exit/cleanup matrix): every exit path — normal return, an
/// early return in each if arm, a loop with a survivor list read inside
/// and used after — frees its live set exactly (ledger balance), and the
/// cancel/budget knobs trap at the poll/charge with the SAME balance
/// (cleanup is zero-charge: the frees after a Budget/Cancelled trap
/// never bill, never poll, never allocate).
#[test]
fn a2c_coll_exit_cleanup_matrix() {
    // survivor `ys` lives across the loop (read inside, after the loop);
    // `zs` dies inside the iteration (scoped per-iteration death at the
    // back-edge); both arms of the match-style if return early
    const SRC: &str = "fn main() int {\n    var ys = List<int>.new()\n    ys.push(10)\n    var xs = List<int>.new()\n    xs.push(1)\n    xs.push(2)\n    var t int = 0\n    for x in xs {\n        var zs = List<int>.new()\n        zs.push(x)\n        t = t + x + zs.get(0) + ys.get(0)\n    }\n    if t > 100 { return t }\n    return t + ys.len()\n}";
    let mut s = open_session("exit_cleanup");
    // clean: 3 iterations of (1,2) each adding 2x + 10 => t = (2+10)*... x=1: 1+1+10=12; x=2: 2+2+10=14; t=26; +1 = 27
    let (out, err) = run_c_chain(&mut s, "EXIT_MATRIX-clean", SRC, &[("A2X_RT_EVENTS", "1")]);
    assert_value("EXIT_MATRIX-clean", &out, 27);
    let (a, ab, f, fb) = ledger(&err);
    assert_eq!((a, ab), (f, fb), "clean-run ledger must close");
    // cancel at the FIRST poll: the poll's error path frees the
    // per-iteration scoped deaths AND the survivors — balance proves it
    let (out, err) = run_c_chain(
        &mut s,
        "EXIT_MATRIX-cancel",
        SRC,
        &[("A2X_RT_CANCEL", "1"), ("A2X_RT_EVENTS", "1")],
    );
    assert_trap("EXIT_MATRIX-cancel", &out, "Cancelled");
    let (a, ab, f, fb) = ledger(&err);
    assert_eq!((a, ab), (f, fb), "cancel-run ledger must close");
    // budget exhaustion at the first charge: same cleanup discipline
    let (out, err) = run_c_chain(
        &mut s,
        "EXIT_MATRIX-budget",
        SRC,
        &[("A2X_RT_BUDGET", "3"), ("A2X_RT_EVENTS", "1")],
    );
    assert_trap("EXIT_MATRIX-budget", &out, "Budget");
    let (a, ab, f, fb) = ledger(&err);
    assert_eq!((a, ab), (f, fb), "budget-run ledger must close");
}

/// V-05 (multi-exit arms): both if arms return early with different
/// live sets; each path closes (clean run only needs value + balance —
/// the fault legs above already cover trapped paths).
#[test]
fn a2c_coll_branch_exit_frees() {
    const SRC: &str = "fn pick(k int) int {\n    var a = List<int>.new()\n    a.push(k)\n    var b = List<Str>.new()\n    b.push(\"xy\")\n    if k > 0 { return a.get(0) }\n    return b.get(0).len()\n}\nfn main() int {\n    return pick(4) + pick(0)\n}";
    let mut s = open_session("branch_exit");
    let (out, err) = run_c_chain(&mut s, "BRANCH_EXIT", SRC, &[("A2X_RT_EVENTS", "1")]);
    assert_value("BRANCH_EXIT", &out, 6);
    let (a, ab, f, fb) = ledger(&err);
    assert_eq!((a, ab), (f, fb));
}

/// V-06 (helper resource probes): a HAND-WRITTEN C probe compiled against
/// the SAME generated runtime + typed helpers of a real session assembly
/// (`#include "generated.c"` — same TU, direct access to the runtime
/// statics). The probe is scaffolding by contract: it drives the runtime
/// primitives directly (never the lowered entry, never MIR/ABC, never
/// source List/Variant.clone semantics) and witnesses, on a 256 KiB
/// stack thread: the 3072-link iterative deep chain; mid-fault deep-clone
/// ROLLBACK with same-process retry; mid-fault push completion;
/// cooperative cancel at the probe's own poll; budget exhaustion with
/// zero-charge cleanup; typed list op atomicity (faulted growth push
/// leaves len/content intact, retry completes; a charge-failed set frees
/// only its uncommitted input).
#[test]
fn a2c_coll_helper_probe_same_runtime() {
    let mut s = open_session("helper_probe");
    let asm = s
        .cold_assemble_profile(L01, SourceProfile::Collections, &[], "main", false)
        .expect("assembly");
    let dir = out_root().join("HELPER_PROBE");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        if path == "main.c" {
            continue; // the probe carries its own main
        }
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("probe.c"), HELPER_PROBE_C).unwrap();
    let exe = dir.join("probe.exe");
    ace_a2x_worker::crun::compile_c_standalone(&dir.join("probe.c"), &exe)
        .unwrap_or_else(|e| panic!("probe cl failed: {e}"));
    let run = auto_tool_execution::run_command_capture(
        {
            let mut c = Command::new(&exe);
            c.current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run probe");
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success);
    let out = String::from_utf8_lossy(run.stdout.as_ref().expect("stdout")).to_string();
    assert_eq!(
        out, "P6:ok\nP1:ok\nP2:ok\nP3:ok\nP4:ok\nP5:ok\n",
        "probe scenarios"
    );
}

/// The hand-written helper probe source (test scaffolding against the
/// generated runtime; not lowered code, not source semantics).
const HELPER_PROBE_C: &str = r#"#include <fcntl.h>
#include <io.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include "a2x_c_coll_runtime.h"
#include "generated.c"

static int g_fail = 0;

/* the deep-chain scenarios run on a 256 KiB stack thread */
static void deep_scenario(void *unused) {
    a2x_deep_link *chain = NULL;
    a2x_deep_link *clone = NULL;
    a2x_trap_code t;
    long i;
    (void)unused;
    /* P1: 3072-link iterative chain */
    for (i = 0; i < 3072; i++) {
        t = a2x_deep_push(&chain, (const uint8_t *)"n", 1);
        if (t != A2X_TRAP_NONE) { g_fail = 1; return; }
    }
    if (a2x_deep_len(chain) != 3072) { g_fail = 1; return; }
    t = a2x_deep_clone(chain, &clone);
    if (t != A2X_TRAP_NONE || a2x_deep_len(clone) != 3072) { g_fail = 1; return; }
    a2x_deep_free(&clone);
    a2x_deep_free(&chain);
    if (a2x_coll_live_str_bytes() != 0 || a2x_coll_live_backing_bytes() != 0) { g_fail = 1; return; }
    printf("P1:ok\n");

    /* P2: mid-fault deep-clone rollback + same-process retry */
    for (i = 0; i < 3072; i++) {
        t = a2x_deep_push(&chain, (const uint8_t *)"n", 1);
        if (t != A2X_TRAP_NONE) { g_fail = 1; return; }
    }
    {
        long before_bytes = a2x_coll_live_backing_bytes();
        a2x_coll_cfg_fault = a2x_coll_alloc_count() + 100; /* fires inside the clone prefix */
        t = a2x_deep_clone(chain, &clone);
        if (t != A2X_TRAP_RESOURCE) { g_fail = 1; return; }
        /* the partial clone was freed; the original is intact */
        if (a2x_deep_len(chain) != 3072) { g_fail = 1; return; }
        if (a2x_coll_live_backing_bytes() != before_bytes) { g_fail = 1; return; }
        a2x_coll_cfg_fault = 0; /* clear -> SAME-PROCESS retry completes */
        t = a2x_deep_clone(chain, &clone);
        if (t != A2X_TRAP_NONE || a2x_deep_len(clone) != 3072) { g_fail = 1; return; }
        a2x_deep_free(&clone);
        a2x_deep_free(&chain);
        if (a2x_coll_live_str_bytes() != 0) { g_fail = 1; return; }
    }
    printf("P2:ok\n");

    /* P3: mid-fault push (partial chain intact, then completed) */
    a2x_coll_cfg_fault = a2x_coll_alloc_count() + 300;
    for (i = 0; i < 3072; i++) {
        t = a2x_deep_push(&chain, (const uint8_t *)"n", 1);
        if (t == A2X_TRAP_RESOURCE) break;
        if (t != A2X_TRAP_NONE) { g_fail = 1; return; }
    }
    if (i != 299 || a2x_deep_len(chain) != 299) { g_fail = 1; return; }
    a2x_coll_cfg_fault = 0;
    for (; i < 3072; i++) {
        t = a2x_deep_push(&chain, (const uint8_t *)"n", 1);
        if (t != A2X_TRAP_NONE) { g_fail = 1; return; }
    }
    if (a2x_deep_len(chain) != 3072) { g_fail = 1; return; }
    a2x_deep_free(&chain);
    if (a2x_coll_live_str_bytes() != 0) { g_fail = 1; return; }
    printf("P3:ok\n");

    /* P4: cooperative cancel observed at the probe's own poll loop
     * (the poll is a lowered back-edge's observation point) */
    {
        int polls = 0;
        a2x_coll_cfg_cancel = 1;
        for (;;) {
            t = a2x_coll_poll();
            if (t != A2X_TRAP_NONE) break;
            polls++;
        }
        if (t != A2X_TRAP_CANCELLED || polls != 0) { g_fail = 1; return; }
        a2x_coll_cfg_cancel = 0;
    }
    /* a loop iteration shape: push + poll; cancel at the 2nd poll */
    for (i = 0; i < 3; i++) {
        t = a2x_deep_push(&chain, (const uint8_t *)"n", 1);
        if (t != A2X_TRAP_NONE) { g_fail = 1; return; }
        if (i == 1) a2x_coll_cfg_cancel = 1;
        t = a2x_coll_poll();
        if (t == A2X_TRAP_CANCELLED) break;
        if (t != A2X_TRAP_NONE) { g_fail = 1; return; }
    }
    if (t != A2X_TRAP_CANCELLED || a2x_deep_len(chain) != 2) { g_fail = 1; return; }
    a2x_coll_cfg_cancel = 0;
    a2x_deep_free(&chain);
    if (a2x_coll_live_str_bytes() != 0) { g_fail = 1; return; }
    printf("P4:ok\n");

    /* P5: budget exhaustion; cleanup stays zero-charge (frees still run) */
    a2x_coll_work = -1; /* next charge (2 WU for 10 bytes) exhausts */
    t = a2x_deep_push(&chain, (const uint8_t *)"nnnnnnnnnn", 10);
    if (t != A2X_TRAP_BUDGET) { g_fail = 1; return; }
    a2x_deep_free(&chain); /* must not charge/poll/allocate */
    if (a2x_coll_live_str_bytes() != 0 || a2x_coll_live_backing_bytes() != 0) { g_fail = 1; return; }
    printf("P5:ok\n");
}

int main(void) {
    HANDLE th;
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
    /* P6: typed list op atomicity from the generated table (main thread) */
    {
        a2x_list_i32 l;
        a2x_trap_code t;
        int32_t v = 0;
        long i;
        long nodes_before;
        t = a2x_coll_list_i32_new(&l);
        if (t != A2X_TRAP_NONE) { g_fail = 1; }
        for (i = 0; i < 4 && !g_fail; i++) {
            t = a2x_coll_list_i32_push(&l, (int32_t)(i + 10));
            if (t != A2X_TRAP_NONE) { g_fail = 1; }
        }
        /* fault the 5th push's GROWTH allocation: old target intact */
        a2x_coll_cfg_fault = (long)(a2x_coll_alloc_count() + 1);
        t = a2x_coll_list_i32_push(&l, 50);
        if (t != A2X_TRAP_RESOURCE) { g_fail = 1; }
        if (a2x_coll_list_i32_len(&l) != 4) { g_fail = 1; }
        t = a2x_coll_list_i32_get(&l, 3, &v);
        if (t != A2X_TRAP_NONE || v != 13) { g_fail = 1; } /* content intact */
        a2x_coll_cfg_fault = 0; /* same-process retry completes */
        t = a2x_coll_list_i32_push(&l, 50);
        if (t != A2X_TRAP_NONE || a2x_coll_list_i32_len(&l) != 5) { g_fail = 1; }
        /* a charge-failed set frees only its uncommitted input */
        nodes_before = a2x_coll_live_nodes();
        a2x_coll_work = -1; /* next charge fails -> set frees its input */
        t = a2x_coll_list_i32_set(&l, 0, 99);
        if (t != A2X_TRAP_BUDGET) { g_fail = 1; }
        if (a2x_coll_live_nodes() != nodes_before) { g_fail = 1; }
        t = a2x_coll_list_i32_get(&l, 0, &v);
        if (t != A2X_TRAP_NONE || v != 10) { g_fail = 1; } /* old element kept */
        a2x_coll_work = -1000000;
        t = a2x_coll_list_i32_set(&l, 0, 99);
        if (t != A2X_TRAP_NONE) { g_fail = 1; }
        t = a2x_coll_list_i32_get(&l, 0, &v);
        if (t != A2X_TRAP_NONE || v != 99) { g_fail = 1; }
        if (a2x_coll_live_nodes() != nodes_before) { g_fail = 1; } /* swap, not growth */
        a2x_coll_list_i32_free(&l);
        if (a2x_coll_live_nodes() != 0) { g_fail = 1; }
        if (!g_fail) printf("P6:ok\n");
    }
    th = CreateThread(NULL, 256 * 1024, (LPTHREAD_START_ROUTINE)deep_scenario, NULL, 0, NULL);
    if (th == NULL) return 2;
    WaitForSingleObject(th, INFINITE);
    CloseHandle(th);
    return g_fail ? 1 : 0;
}
"#;
