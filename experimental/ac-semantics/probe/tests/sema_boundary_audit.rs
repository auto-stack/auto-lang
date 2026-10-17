//! PLAN-796 T-07 boundary/parallel audit tests (`sema_boundary_`
//! filter, V-09): the core library has NO product-backend dependency,
//! the split-route hotspots are untouched, every frozen interface
//! version has a written provenance, the 797-consumption surface is
//! complete per the frozen description set, and the pending AutoDown
//! contract stays honestly marked until T-06.

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// The ac-semantics workspace root (= probe crate dir / ..).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").canonicalize().unwrap()
}

/// The repository root (= probe crate dir / ../../..).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

/// V-09: `cargo tree` over the DEFAULT build of ac-sema-core shows no
/// auto-lang / UI / VM / Cranelift / auto-man anywhere — the semantic
/// model is provably backend-free (plan §2).
#[test]
fn sema_boundary_core_tree_has_no_product_backends() {
    let out = Command::new("cargo")
        .args(["tree", "-p", "ac-sema-core", "-e", "normal", "--offline"])
        .current_dir(workspace_root())
        .output()
        .expect("cargo tree runs");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let tree = if out.status.success() { stdout } else { stderr };
    // compare CRATE NAMES only — the worktree PATH itself contains
    // "auto-lang", so raw-substring checks would false-positive
    let crate_names: Vec<String> = tree
        .lines()
        .map(|l| l.split(" (").next().unwrap_or("").to_string())
        .map(|l| l.trim().trim_start_matches(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_')).to_string())
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .collect();
    assert!(
        tree.contains("ac-sema-core"),
        "tree resolved (output: {})",
        tree.chars().take(300).collect::<String>()
    );
    for forbidden in [
        "auto-lang",
        "cranelift",
        "auto-man",
        "auto-vm",
        "iced",
        "auto-val",
        "auto-cache",
    ] {
        assert!(
            !crate_names.iter().any(|n| n == forbidden || n.starts_with(&format!("{forbidden}-"))),
            "ac-sema-core default tree must not contain `{forbidden}`:\n{tree}"
        );
    }
    // sanity: the tree really resolved the contract dependency
    assert!(tree.contains("ac-sema-contract"));
    // the contract crate itself only hashes
    let contract = Command::new("cargo")
        .args(["tree", "-p", "ac-sema-contract", "-e", "normal", "--offline"])
        .current_dir(workspace_root())
        .output()
        .expect("cargo tree runs");
    let ct = String::from_utf8_lossy(&contract.stdout).to_string();
    let ct_names: Vec<String> = ct
        .lines()
        .map(|l| l.split(" (").next().unwrap_or("").to_string())
        .map(|l| l.trim().trim_start_matches(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_')).to_string())
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .collect();
    assert!(
        ct_names.iter().any(|n| n == "blake3") && !ct_names.iter().any(|n| n.starts_with("auto-lang")),
        "contract tree names: {ct_names:?}"
    );
}

/// V-09: the implementation diff stays inside the SANCTIONED
/// namespaces. Through T-05 (pre-D-788) that meant zero touches under
/// crates/; since T-06 (D-788 satisfied, single-writer window open) the
/// shared TestCase protocol files ARE the sanctioned write face — while
/// the 783 store, ac-core, root manifest and cargo config remain
/// byte-identical to the fork point on master.
#[test]
fn sema_boundary_split_route_hotspots_untouched() {
    let root = repo_root();
    // fork point of this branch against master
    let base = Command::new("git")
        .args(["merge-base", "HEAD", "master"])
        .current_dir(&root)
        .output()
        .expect("git merge-base runs");
    assert!(base.status.success(), "merge-base failed");
    let base = String::from_utf8_lossy(&base.stdout).trim().to_string();
    assert_eq!(base.len(), 40, "40-hex base commit, got {base}");

    let diff = Command::new("git")
        .args(["diff", "--name-only", &format!("{base}..HEAD")])
        .current_dir(&root)
        .output()
        .expect("git diff runs");
    assert!(diff.status.success());
    let changed = String::from_utf8_lossy(&diff.stdout);
    let forbidden_prefixes = [
        "crates/",
        "experimental/ac-core/",
        "experimental/ac-artifact-probe/",
        ".cargo/",
    ];
    let forbidden_exact = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"];
    // sanctioned shared-protocol faces (T-06 single-writer window): the
    // decoder/suite/ir in auto-test-case, the test_cases adapter tree +
    // runner tests + corpus in auto-lang, and the 788 contract test that
    // mechanically absorbs the new ExpectSpec field (compile compat).
    const SANCTIONED_SHARED: &[&str] = &[
        "crates/auto-test-case/",
        "crates/auto-lang/src/test_cases/",
        "crates/auto-lang/src/tests",
        "crates/auto-lang/test/cases/ac-semantics/",
        "crates/auto-lang/tests/plan788_r3_.rs",
    ];
    for line in changed.lines() {
        for p in forbidden_prefixes {
            if SANCTIONED_SHARED.iter().any(|s| line.starts_with(s)) {
                continue;
            }
            assert!(
                !line.starts_with(p),
                "hotspot path `{line}` modified ({p}) — split-route zero-touch violated"
            );
        }
        assert!(
            !forbidden_exact.contains(&line),
            "root file `{line}` modified — root workspace must stay untouched"
        );
        // implementation lives ONLY under the sanctioned namespaces
        let legacy_root = "test/cases/ac-semantics/"; // pre-T-06 location (moved)
        assert!(
            line.starts_with("experimental/ac-semantics/")
                || line.starts_with(legacy_root)
                || line.starts_with("docs/reports/plan796/")
                || line.starts_with("docs/plans/attachments/796/")
                || line.starts_with("scripts/verify-ac-semantics-796.ps1")
                || line.starts_with("docs/plans/796-")
                || SANCTIONED_SHARED.iter().any(|s| line.starts_with(s))
                || line.ends_with(".case.ad"),
            "unexpected changed path `{line}` outside sanctioned namespaces"
        );
    }
}

/// V-09: every frozen interface version HAS a written provenance — the
/// constants the code mints are exactly the ones the frozen decision
/// document names (single source of truth check).
#[test]
fn sema_boundary_frozen_versions_have_provenance() {
    let doc = std::fs::read_to_string(
        repo_root().join("docs/plans/attachments/796/contract-decision.md"),
    )
    .expect("contract-decision.md present");
    for (constant, value) in [
        ("SEMA_PRODUCER_ID", ac_sema_contract::SEMA_PRODUCER_ID),
        ("SEMA_REF_SCHEMA_V1", ac_sema_contract::SEMA_REF_SCHEMA_V1),
        ("SEMA_PAGE_SCHEMA_V1", ac_sema_contract::SEMA_PAGE_SCHEMA_V1),
        ("SEMA_LOGICAL_DOMAIN", ac_sema_contract::SEMA_LOGICAL_DOMAIN),
    ] {
        assert!(
            doc.contains(value),
            "{constant} = `{value}` must appear in contract-decision.md"
        );
    }
    assert!(doc.contains(ac_sema_core::query::INFER_ALGO_ID), "algo id documented");
    assert!(doc.contains(ac_sema_core::reflect::CT_SEAM_SCHEMA), "ct seam schema documented");
    assert!(doc.contains(ac_sema_probe::SEMA_LAYER), "store layer documented");
    assert!(doc.contains("SnapshotRecord"), "carrier kind documented");
    // code constants are the single mint: the layer name never claims
    // an A-layer registration
    assert_ne!(ac_sema_probe::SEMA_LAYER, "A06");
}

/// V-09: the 797-consumption surface is COMPLETE per the frozen
/// description set (D6 / §5.1) — compile-time exhaustiveness plus a
/// documented-set cross-check.
#[test]
fn sema_boundary_797_description_surface_frozen() {
    use ac_sema_contract::TypeDesc::*;
    // every variant must be handled here — adding one without updating
    // the frozen set fails to compile
    let count = |t: &ac_sema_contract::TypeDesc| match t {
        Unit | I32 | U32 | I64 | U64 | F64 | Bool | Char | ByteBuffer | Opaque { .. }
        | Fn { .. } | Record { .. } | Mode { .. } => 13usize,
    };
    assert_eq!(count(&ac_sema_contract::TypeDesc::I32), 13);
    let doc = std::fs::read_to_string(
        repo_root().join("docs/plans/attachments/796/contract-decision.md"),
    )
    .unwrap();
    for token in ["U32", "U64", "F64", "ByteBuffer", "Opaque", "view|mut|owned", "间接", "ac796/ct-seam/v1"] {
        assert!(doc.contains(token), "D6 documents `{token}` for the 797 surface");
    }
    // modes are the three frozen ones
    assert_eq!(ac_sema_contract::Mode::View.as_str(), "view");
    assert_eq!(ac_sema_contract::Mode::Mut.as_str(), "mut");
    assert_eq!(ac_sema_contract::Mode::Owned.as_str(), "owned");
}

/// V-09 (T-06 landed): the corpus files carry the EXECUTED status and
/// run through the unified runner (probe_adapter_corpus_executes_green
/// is the execution proof); none may regress to a pending marker, and
/// none may claim a shell/executable payload.
#[test]
fn sema_boundary_case_files_executed_status_honest() {
    let dir = repo_root().join("crates/auto-lang/test/cases/ac-semantics");
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ad")
            && path.file_name().and_then(|n| n.to_str()) != Some("ac-semantics.suite.ad")
        {
            let text = std::fs::read_to_string(&path).unwrap();
            assert!(
                text.to_lowercase().contains("status: executed"),
                "{} must carry the executed status (T-06 landed)",
                path.display()
            );
            assert!(
                !text.to_lowercase().contains("not-executed"),
                "{} must not regress to a pending marker",
                path.display()
            );
            assert!(
                !text.contains("shell") && !text.contains(".exe"),
                "{} must not carry shell/executable payloads",
                path.display()
            );
            assert!(text.contains("sema/query-v1"), "{} names the registered recipe", path.display());
            checked += 1;
        }
    }
    assert!(checked >= 3, "all case files audited, got {checked}");
}

/// V-09: the three-crate dependency audit — no product dependency in
/// ANY manifest; the ac-core/783 paths exist only as feature-gated or
/// probe-side read-only inputs.
#[test]
fn sema_boundary_manifest_dependency_audit() {
    let root = workspace_root();
    for (manifest, extra_allowed) in [
        ("contract/Cargo.toml", vec!["blake3"]),
        ("core/Cargo.toml", vec!["ac-sema-contract", "auto-ac-prototype"]),
        ("probe/Cargo.toml", vec!["ac-sema-contract", "ac-sema-core", "ac-artifact-probe", "auto-ac-prototype", "serde", "serde_json"]),
    ] {
        let text = std::fs::read_to_string(root.join(manifest)).unwrap();
        // dependency scan ignores comment lines — the manifests'
        // constraint COMMENTS name the forbidden crates on purpose
        let dep_text: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("
");
        for forbidden in [
            "auto-lang",
            "cranelift",
            "auto-man",
            "auto-vm",
            "auto-val",
            "iced",
            "auto-cache",
        ] {
            assert!(
                !dep_text.contains(forbidden),
                "{manifest} must not depend on `{forbidden}`"
            );
        }
        for allowed in extra_allowed {
            assert!(dep_text.contains(allowed), "{manifest} declares {allowed}");
        }
    }
    // the ONLY bridging path to backends is the feature-gated adapter
    let core_manifest = std::fs::read_to_string(root.join("core/Cargo.toml")).unwrap();
    assert!(core_manifest.contains("optional = true"), "auto-ac-prototype stays optional");
    assert!(core_manifest.contains("[features]"), "feature table present");
}
