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
