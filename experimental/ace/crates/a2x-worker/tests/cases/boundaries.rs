//! V-13 dependency-direction controls (plan §2/§5.1: the shared A2X core
//! consumes only verified HIR facts — it must NOT depend on
//! auto-mir-lower, the AV executor, Cranelift, the old AST/Database
//! frontend or the old Trans stack).
//!
//! Two faces are checked mechanically:
//! * MANIFEST face — the a2x workspace's own dependency declarations admit
//!   only the sanctioned path deps (a2x siblings, ac-core, ac-artifact-probe
//!   for the 783 store) plus the probe-only windows-sys memory counter;
//! * SOURCE face — no a2x source file references forbidden modules/symbols
//!   (mir_lower, av executor, cranelift, the old trans/ast-database).

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn manifest_face_only_sanctioned_dependencies() {
    let sanctioned = [
        "ace-a2x-c",            // PLAN-921 CA-A: the C target provider (a2x-core + core HIR)
        "ace-foreign-contract", // PLAN-937 CA-D1: the NEUTRAL interface-facts/
        // physical-plan model both real consumers
        // (abi native side + this C side) bind to —
        // pure model crate, no backend stack
        "ace-abi", // PLAN-921: the frozen PLAN-797 cl locator (ctool) reuse —
        // no second MSVC discovery path
        "ace-a2x-core",
        "ace-a2x-rust",
        "ace-a2x-store",
        "ace-core",
        "ace-artifacts",
        // PLAN-934 T-08: the ordinary scalar source cold path is compiled by
        // the SQ-A fact entry (`ace-source-query::compile_scalar_product`),
        // the same shared consumer door AV/native use. The dependency is
        // consumer-side only: source-query never depends on any a2x crate.
        "ace-source-query",
        "ace-semantics-contract", // PLAN-944: backend-free assembly facts/codec only
        // PLAN-936 T-13: CT-carrying scalar sources route through the
        // source-cte single door (the same consumer-side shape: the Rust/C
        // product lanes serve `#{ ... }` demands before the fact lane
        // binds). Consumer-side only: source-cte never depends on any a2x
        // crate (its manifest whitelists core/lowering/av-core/artifacts/
        // semantics-contract/source-query).
        "ace-source-cte",
        "auto-tool-execution", // neutral std-only lifecycle: formal compiler tool and test consumers
        "windows-sys",         // probe-only, cfg(windows): process memory counters
        "serde",               // PLAN-950: data-only module root/fragment codec derive;
        // no backend, frontend or executor dependency
        "serde_json", // probe-only: the a2xworker/1 JSON protocol (the same
                      // crate the 783 store already depends on)
    ];
    let members = [
        "crates/a2x-c",
        "crates/a2x-core",
        "crates/a2x-rust",
        "crates/a2x-store",
        "crates/a2x-worker",
    ];
    for m in members {
        let manifest = std::fs::read_to_string(workspace_root().join(m).join("Cargo.toml"))
            .unwrap_or_else(|e| panic!("read {} manifest: {}", m, e));
        // collect quoted dependency names from dependency tables
        let mut section = String::new();
        for line in manifest.lines() {
            if line.trim_start().starts_with('[') {
                section = line.trim().to_string();
                continue; // the header line is not a dependency
            }
            if section.contains("dependencies") || section.contains("dev-dependencies") {
                if let Some(name) = line.split('=').next() {
                    let name = name.trim();
                    if name.is_empty()
                        || name.starts_with('#')
                        || name.starts_with('"')
                        || name.starts_with(']')
                    {
                        continue;
                    }
                    assert!(
                        sanctioned.contains(&name),
                        "{} declares unsanctioned dependency `{}` (section {})",
                        m,
                        name,
                        section
                    );
                }
            }
        }
    }
}

#[test]
fn source_face_no_forbidden_references() {
    let forbidden = [
        "ace_lowering",
        "mir_lower",
        "cranelift",
        "cranelift_module",
        "ace_av",
        "av_executor",
        "run_autovm",
        "old_trans",
        "RustTrans",
        "a2r_std",
    ];
    let mut scanned = 0usize;
    let dirs = [
        "crates/a2x-c/src",
        "crates/a2x-core/src",
        "crates/a2x-rust/src",
        "crates/a2x-store/src",
        "crates/a2x-worker/src",
        "crates/a2x-worker/tests",
    ];
    for dir in dirs {
        let path = workspace_root().join(dir);
        let mut stack = vec![path];
        while let Some(p) = stack.pop() {
            if p.is_dir() {
                for e in std::fs::read_dir(&p).unwrap() {
                    stack.push(e.unwrap().path());
                }
            } else if p.extension().map(|e| e == "rs").unwrap_or(false) {
                // this control's own source carries the needle list —
                // scanning it would be self-referential
                if p.file_name().map(|n| n == "boundaries.rs").unwrap_or(false) {
                    continue;
                }
                scanned += 1;
                let text = std::fs::read_to_string(&p).unwrap();
                for needle in forbidden {
                    assert!(
                        !text.contains(needle),
                        "{} references forbidden `{}`",
                        p.display(),
                        needle
                    );
                }
            }
        }
    }
    assert!(
        scanned >= 15,
        "expected to scan the a2x sources ({} files)",
        scanned
    );
}

/// PLAN-921 CA-A: the two-target status. Rust keeps its frozen single
/// backend identity; the C target is a REAL second backend with its own
/// identity, its own fragment namespace (AXC1) and a scalar-only family
/// surface (every other family is a named rejection, never a fallback).
/// The shared session layer must not statically depend on either target
/// crate (the provider seam).
#[test]
fn two_target_identities_and_neutral_store_seam() {
    let r = ace_a2x_rust::TargetProfile::v1();
    assert_eq!(r.lang, "rust");
    assert_eq!(r.edition, "2021");
    assert_eq!(r.backend, "auto-a2x-rust");
    let c = ace_a2x_c::CTargetProvider::v1();
    let d = ace_a2x_core::target::TargetProvider::descriptor(&c);
    assert_eq!(d.lang, "c");
    assert_eq!(d.standard, "c11");
    assert_eq!(d.backend, "auto-a2x-c");
    // distinct namespaces: fragment layer ids never collide
    use ace_a2x_core::target::{Family, TargetProvider};
    assert_eq!(c.fragment_layer_id(Family::Scalar).unwrap(), "AXC1");
    assert_eq!(
        ace_a2x_rust::RustTargetProvider::v1()
            .fragment_layer_id(Family::Scalar)
            .unwrap(),
        "AXR1"
    );
    // PLAN-925/926: the C target serves scalar + record-text +
    // collections (AXC1/AXC2/AXC3); the three namespaces never collide
    // with the Rust layers (AXR1/2/3)
    assert_eq!(c.fragment_layer_id(Family::RecordText).unwrap(), "AXC2");
    assert_eq!(c.fragment_layer_id(Family::Collections).unwrap(), "AXC3");
    // the shared session layer is target-neutral: a2x-store carries no
    // target-crate dependency
    let store_manifest =
        std::fs::read_to_string(workspace_root().join("crates/a2x-store/Cargo.toml")).unwrap();
    assert!(
        !store_manifest.contains("ace-a2x-rust ="),
        "a2x-store must not depend on the Rust target crate (PLAN-921 seam)"
    );
    assert!(
        !store_manifest.contains("ace-a2x-c ="),
        "a2x-store must not depend on the C target crate (PLAN-921 seam; the `ace-a2x-core` prefix is a different crate)"
    );
}
