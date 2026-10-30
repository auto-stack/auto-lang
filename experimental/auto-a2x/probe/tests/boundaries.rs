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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn manifest_face_only_sanctioned_dependencies() {
    let sanctioned = [
        "auto-a2x-core",
        "auto-a2x-rust",
        "auto-a2x-store",
        "auto-ac-prototype",
        "ac-artifact-probe",
        "windows-sys", // probe-only, cfg(windows): process memory counters
        "serde_json",  // probe-only: the a2xworker/1 JSON protocol (the same
                       // crate the 783 store already depends on)
    ];
    let members = ["a2x-core", "a2x-rust", "a2x-store", "probe"];
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
        "auto_mir_lower",
        "mir_lower",
        "cranelift",
        "cranelift_module",
        "auto_av",
        "av_executor",
        "run_autovm",
        "old_trans",
        "RustTrans",
        "a2r_std",
    ];
    let mut scanned = 0usize;
    let dirs = [
        "a2x-core/src",
        "a2x-rust/src",
        "a2x-store/src",
        "probe/src",
        "probe/tests",
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

/// The single-target status: Rust is the only implemented provider; a
/// second target is explicitly NOT claimed (plan non-goals). The frozen
/// TargetProfile pins exactly one backend identity.
#[test]
fn single_target_profile_no_false_multi_target_claims() {
    let t = auto_a2x_rust::TargetProfile::v1();
    assert_eq!(t.lang, "rust");
    assert_eq!(t.edition, "2021");
    assert_eq!(t.backend, "auto-a2x-rust");
    // one backend identity, one edition — nothing else is claimed
}
