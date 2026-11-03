//! PLAN-921 CA-A (T-04/T-05): the FULL frozen scalar corpus through the
//! C target — each case
//! source → checked frontend → shared semantic view → C provider →
//! session assembly (store-persisted fragments) → real MSVC cl /std:c11
//! compile → real run → parent-side oracle.
//!
//! Same frozen inputs as the Rust leg (`scalar_semantics::cases()` — the
//! 745/767 corpora + ACE-M1 B-cases + boundary synthetics + trace-mark
//! order + short-circuit polarity/trap anti-controls), INDEPENDENT
//! oracle execution: value/trap protocol and stderr mark order are
//! asserted against the case table, byte-for-byte like the Rust leg.
//! The C channel has ONE flavor (the frozen `-O2 -MT` flags are the
//! channel identity); the Rust leg's release_too rows need no C twin.
//!
//! This is the certification-shaped leg (explicit cl+run spawns). A
//! missing MSVC toolchain is an INFRA failure (never a skip); compile
//! errors are infra facts, never Auto diagnostics/traps (§5.1).

use crate::scalar_semantics::{cases, Expect};
use ace_a2x_c::CTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::source::TRACE_CAPABILITY;
use std::path::PathBuf;

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c")
}

fn store_root() -> PathBuf {
    // fresh store PER PROCESS: the C channel's fragment identity keys on
    // the frozen target identity (backend_version), so a store reused
    // across code changes would legitimately serve stale bytes — the
    // corpus leg always starts from a clean namespace.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/a2x-out-c/store-{}",
        std::process::id()
    ))
}

fn run_case_c(case: &crate::scalar_semantics::Case) {
    let caps = if case.trace {
        vec![TRACE_CAPABILITY.to_string()]
    } else {
        vec![]
    };
    // a C-provider session over a shared scratch store (per-case assembly;
    // cross-case fragment hits are legitimate — same identities)
    let mut s = A2xSession::open(
        &store_root(),
        "a2c-corpus",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .expect("open C session");
    let scalar = case.profile != "core";
    let asm = s
        .cold_assemble(&case.src, scalar, &caps, "main", false)
        .unwrap_or_else(|e| panic!("C assembly failed for {}: {:?}", case.name, e));

    let dir = out_root().join(&case.name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();

    let (stdout, stderr) = ace_a2x_worker::crun::compile_and_run_c(&dir, "a2c_case.exe")
        .unwrap_or_else(|e| panic!("C compile/run failed for {}: {}", case.name, e));
    let expected_line = match &case.expect {
        Expect::Value(x) => format!("value:{}\n", x),
        Expect::Trap(t) => format!("trap:{}\n", t),
    };
    assert_eq!(
        stdout, expected_line,
        "stdout mismatch for {} (C) — marks observed: {:?}",
        case.name, stderr
    );
    let marks: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        marks, case.marks,
        "trace marks mismatch for {} (C)",
        case.name
    );
}

#[test]
fn c_full_corpus_true_chain() {
    let cases = cases();
    assert!(cases.len() >= 55, "corpus table shrank: {}", cases.len());
    for case in &cases {
        run_case_c(case);
    }
}

/// One-case smoke (fast feedback leg; the full corpus is the gate).
#[test]
fn c_smoke_b01() {
    let cs = cases();
    let case = cs
        .iter()
        .find(|c| c.name == "ace-B01")
        .or_else(|| cs.first())
        .expect("at least one corpus case");
    run_case_c(case);
}

/// PLAN-921 §5.1/V-04: the C target REJECTS the non-scalar families by
/// NAME (successor-package surfaces; never a silent fallback), and an
/// illegal SOURCE is rejected by the frontend before any target work.
#[test]
fn c_named_family_and_source_rejections() {
    use ace_core::source::{compile_collections_source, SourceProfile};
    let root = store_root().join("neg");
    let mut s =
        A2xSession::open(&root, "a2c-neg", "probe", 8 << 20, CTargetProvider::v1()).unwrap();
    // collections source through the C provider now SERVES the channel
    // (PLAN-926 CA-C: AXC3) — the frozen L01 shape assembles and carries
    // the collections file-set schema (the true-chain corpus is
    // `a2c_coll`); the old named rejection is retired with 926
    let src =
        "fn main() int {\n    var xs = List<int>.new()\n    xs.push(20)\n    return xs.get(0)\n}";
    let compiled = compile_collections_source(src, "neg", &[]).unwrap();
    let _ = compiled;
    let asm = s
        .cold_assemble_profile(src, SourceProfile::Collections, &[], "main", false)
        .expect("collections through C now serves (PLAN-926)");
    assert!(asm.manifest.contains("auto.a2x.c.collections.fileset/1"));
    // illegal source for the SCALAR channel → frontend rejection (named
    // a2x.compile), before any C work
    let mut s2 =
        A2xSession::open(&root, "a2c-neg", "probe", 8 << 20, CTargetProvider::v1()).unwrap();
    let err = s2
        .cold_assemble(
            "fn main() int { return unknown_name }",
            true,
            &[],
            "main",
            false,
        )
        .expect_err("illegal source must reject");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(r) => {
            assert!(
                r.code.starts_with("a2x."),
                "named a2x rejection expected, got: {}",
                r
            );
        }
        other => panic!("expected Reject, got {:?}", other),
    }
}

/// V-02 manifest identity: the C fileset manifest carries the C schema,
/// the C target identity is distinct from Rust, and the emitted file set
/// is byte-deterministic across two fresh sessions (same inputs → same
/// bytes).
#[test]
fn c_fileset_identity_and_determinism() {
    let src = "fn helper(a int) int { return a * 2 }\nfn main() int { return helper(21) }";
    let assemble = |root: &std::path::Path| {
        let mut s = A2xSession::open(root, "det", "probe", 8 << 20, CTargetProvider::v1()).unwrap();
        s.cold_assemble(src, true, &[], "main", true).unwrap()
    };
    let base = std::env::temp_dir().join("a2c-det");
    let a = assemble(&base.join("a"));
    let b = assemble(&base.join("b"));
    assert!(a.manifest.contains("\"auto.a2x.c.fileset/1\""));
    assert_eq!(a.files, b.files, "file bytes must be deterministic");
    assert_eq!(a.manifest, b.manifest, "manifest must be deterministic");
    let names: Vec<&str> = a.files.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(names, vec!["main.c", "a2x_c_runtime.h", "generated.c"]);
    assert!(a.manifest.contains("\"entry\": \"main\""));
}
