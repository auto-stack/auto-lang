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
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// The repository root (= probe crate dir / ../../..).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap()
}

/// V-09: `cargo tree` over the DEFAULT build of ac-sema-core shows no
/// auto-lang / UI / VM / Cranelift / auto-man anywhere — the semantic
/// model is provably backend-free (plan §2).
#[test]
fn sema_boundary_core_tree_has_no_product_backends() {
    let out = Command::new("cargo")
        .args(["tree", "-p", "ace-semantics", "-e", "normal", "--offline"])
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
        .map(|l| {
            l.trim()
                .trim_start_matches(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_'))
                .to_string()
        })
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .collect();
    assert!(
        tree.contains("ace-semantics"),
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
            !crate_names
                .iter()
                .any(|n| n == forbidden || n.starts_with(&format!("{forbidden}-"))),
            "ac-sema-core default tree must not contain `{forbidden}`:\n{tree}"
        );
    }
    // sanity: the tree really resolved the contract dependency
    assert!(tree.contains("ace-semantics-contract"));
    // the contract crate itself only hashes
    let contract = Command::new("cargo")
        .args([
            "tree",
            "-p",
            "ace-semantics-contract",
            "-e",
            "normal",
            "--offline",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("cargo tree runs");
    let ct = String::from_utf8_lossy(&contract.stdout).to_string();
    let ct_names: Vec<String> = ct
        .lines()
        .map(|l| l.split(" (").next().unwrap_or("").to_string())
        .map(|l| {
            l.trim()
                .trim_start_matches(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_'))
                .to_string()
        })
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .collect();
    assert!(
        ct_names.iter().any(|n| n == "blake3")
            && !ct_names.iter().any(|n| n.starts_with("auto-lang")),
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
    // PLAN-888: this historical zero-touch obligation belongs to PLAN-796,
    // not whichever later Plan happens to execute this independent module.
    // These reachable landed anchors are the parent of its first implementation
    // commit (0ddeeca9312d...) and its archived delivery. The final rebase
    // changed the old review hashes; the archived merge receipt binds delivery.
    const BASE: &str = "5666c1232892b01aec9e15b370de1d4ea1c1f128";
    const DELIVERY: &str = "b76eca8d201ae313cef3b6f20dea55744406f287";
    let ancestry = Command::new("git")
        .args(["merge-base", "--is-ancestor", BASE, DELIVERY])
        .current_dir(&root)
        .output()
        .expect("frozen PLAN-796 ancestry check runs");
    assert!(
        ancestry.status.success(),
        "frozen PLAN-796 history is required"
    );
    let diff = Command::new("git")
        .args(["diff", "--name-only", &format!("{BASE}..{DELIVERY}")])
        .current_dir(&root)
        .output()
        .expect("frozen PLAN-796 diff runs");
    assert!(diff.status.success());
    assert_sanctioned_paths(&String::from_utf8_lossy(&diff.stdout));
}

fn assert_sanctioned_paths(changed: &str) {
    assert!(
        !changed.trim().is_empty(),
        "original implementation diff must not be empty"
    );
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
                // merge-phase SD-01..08 deposit faces (auto-plan:merge):
                // the canonical spec docs, the generated INDEX, the
                // overview group row and the spec-index generator
                || line.starts_with("docs/specs/")
                || line.starts_with("scripts/spec-index.py")
                || SANCTIONED_SHARED.iter().any(|s| line.starts_with(s))
                || line.ends_with(".case.ad"),
            "unexpected changed path `{line}` outside sanctioned namespaces"
        );
    }
}

/// The TRACKED frozen-authority document (PLAN-933 SD-01): the
/// canonical spec section transcribing the PLAN-796 D2–D8 decisions.
/// The original `docs/plans/attachments/796/contract-decision.md` is
/// git-ignored by the 2026-10-31 evidence policy, so a fresh checkout
/// must rely on THIS tracked source (D-925-1 closure).
fn frozen_authority_doc() -> String {
    std::fs::read_to_string(repo_root().join("docs/specs/auto-semantics/semantic-contract.md"))
        .expect("semantic-contract.md (frozen D2–D8 provenance) present")
}

/// The frozen-version provenance assertions, factored so mutation
/// controls can feed doctored copies through the SAME judging face
/// (assertion strength is the original test's, path-swapped only).
fn assert_frozen_versions_documented(doc: &str) {
    for (constant, value) in [
        ("SEMA_PRODUCER_ID", ace_semantics_contract::SEMA_PRODUCER_ID),
        (
            "SEMA_REF_SCHEMA_V1",
            ace_semantics_contract::SEMA_REF_SCHEMA_V1,
        ),
        (
            "SEMA_PAGE_SCHEMA_V1",
            ace_semantics_contract::SEMA_PAGE_SCHEMA_V1,
        ),
        (
            "SEMA_PAGE_SCHEMA_V2",
            ace_semantics_contract::SEMA_PAGE_SCHEMA_V2,
        ),
        (
            "SEMA_LOGICAL_DOMAIN",
            ace_semantics_contract::SEMA_LOGICAL_DOMAIN,
        ),
    ] {
        assert!(
            doc.contains(value),
            "{constant} = `{value}` must appear in the frozen provenance authority"
        );
    }
    assert!(
        doc.contains(ace_semantics::query::INFER_ALGO_ID),
        "algo id documented"
    );
    assert!(
        doc.contains(ace_semantics::reflect::CT_SEAM_SCHEMA),
        "ct seam schema documented"
    );
    assert!(
        doc.contains(ace_semantics_store::SEMA_LAYER),
        "store layer documented"
    );
    assert!(doc.contains("SnapshotRecord"), "carrier kind documented");
    // code constants are the single mint: the layer name never claims
    // an A-layer registration
    assert_ne!(ace_semantics_store::SEMA_LAYER, "A06");
}

/// The description-surface token assertions (D6), factored the same way.
fn assert_description_surface_documented(doc: &str) {
    for token in [
        "U32",
        "U64",
        "F64",
        "ByteBuffer",
        "Opaque",
        "view|mut|owned",
        "间接",
        "ac796/ct-seam/v1",
    ] {
        assert!(
            doc.contains(token),
            "D6 documents `{token}` for the 797 surface"
        );
    }
}

/// V-09 / PLAN-933 V-01: every frozen interface version HAS a written
/// provenance — the constants the code mints are exactly the ones the
/// frozen decision authority (the tracked spec section transcribing the
/// 796 contract decision, SD-01) names. No git-ignored input.
#[test]
fn sema_boundary_frozen_versions_have_provenance() {
    assert_frozen_versions_documented(&frozen_authority_doc());
}

/// V-09 / PLAN-933 V-01: the 797-consumption surface is COMPLETE per the
/// frozen description set (D6 / §5.1) — compile-time exhaustiveness plus
/// a documented-set cross-check against the tracked authority.
#[test]
fn sema_boundary_description_surface_frozen() {
    use ace_semantics_contract::TypeDesc::*;
    // every variant must be handled here — adding one without updating
    // the frozen set fails to compile
    let count = |t: &ace_semantics_contract::TypeDesc| match t {
        Unit
        | I32
        | U32
        | I64
        | U64
        | F64
        | Bool
        | Char
        | ByteBuffer
        | Opaque { .. }
        | Fn { .. }
        | Record { .. }
        | Mode { .. } => 13usize,
    };
    assert_eq!(count(&ace_semantics_contract::TypeDesc::I32), 13);
    assert_description_surface_documented(&frozen_authority_doc());
    // modes are the three frozen ones
    assert_eq!(ace_semantics_contract::Mode::View.as_str(), "view");
    assert_eq!(ace_semantics_contract::Mode::Mut.as_str(), "mut");
    assert_eq!(ace_semantics_contract::Mode::Owned.as_str(), "owned");
}

/// PLAN-933 V-02: the frozen-authority assertions are REAL checks — a
/// doctored authority copy (wrong producer/schema/algo/layer, dropped
/// description or schema token, near-neighbor values) must be rejected
/// by the same judging face the tracked spec goes through. The real
/// tracked document is the accepted control (the two tests above).
#[test]
fn sema_boundary_frozen_authority_rejects_mutations() {
    let doc = frozen_authority_doc();
    let versions_muts: &[(&str, &str)] = &[
        // wrong producer
        ("ac796.sema-probe/1", "ac796.sema-probe/2"),
        // dropped ref schema
        ("ac796/semaref/v1", "ac796/semaref/REDACTED"),
        // near-neighbor page schema (v3 for v2)
        ("ac796/sema-page/v2", "ac796/sema-page/v3"),
        // dropped logical domain
        ("ac796/logical/v1", "ac796/logical/REDACTED"),
        // near-neighbor algo id
        ("ac796-bidir-worklist/1", "ac796-bidir-worklist/2"),
        // dropped ct seam schema
        ("ac796/ct-seam/v1", "ac796/ct-seam/REDACTED"),
        // dropped carrier kind
        ("SnapshotRecord", "CarrierRedacted"),
        // near-neighbor store layer (S797 keeps the != A06 property, so
        // only the documented-layer check can catch it)
        ("`S796`", "`S797`"),
    ];
    for (from, to) in versions_muts {
        let mutated = doc.replace(from, to);
        assert_ne!(&mutated, &doc, "mutation `{from}` must change the doc");
        assert!(
            std::panic::catch_unwind(|| { assert_frozen_versions_documented(&mutated) }).is_err(),
            "mutated authority (`{from}` -> `{to}`) must be rejected"
        );
    }
    for (from, to) in [
        ("ByteBuffer", "ByteBufRedacted"),
        ("view|mut|owned", "view|mut|REDACTED"),
        ("间接", "直接"),
    ] {
        let mutated = doc.replace(from, to);
        assert_ne!(&mutated, &doc, "mutation `{from}` must change the doc");
        assert!(
            std::panic::catch_unwind(|| { assert_description_surface_documented(&mutated) })
                .is_err(),
            "mutated description surface (`{from}` -> `{to}`) must be rejected"
        );
    }
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
            assert!(
                text.contains("sema/query-v1"),
                "{} names the registered recipe",
                path.display()
            );
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
        ("crates/semantics-contract/Cargo.toml", vec!["blake3"]),
        (
            "crates/semantics/Cargo.toml",
            vec!["ace-semantics-contract", "ace-core"],
        ),
        (
            "crates/semantics-store/Cargo.toml",
            vec![
                "ace-semantics-contract",
                "ace-semantics",
                "ace-artifacts",
                "ace-core",
                "serde",
                "serde_json",
            ],
        ),
    ] {
        let text = std::fs::read_to_string(root.join(manifest)).unwrap();
        // dependency scan ignores comment lines — the manifests'
        // constraint COMMENTS name the forbidden crates on purpose
        let dep_text: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join(
                "
",
            );
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
    let core_manifest = std::fs::read_to_string(root.join("crates/semantics/Cargo.toml")).unwrap();
    assert!(
        core_manifest.contains("optional = true"),
        "auto-ac-prototype stays optional"
    );
    assert!(
        core_manifest.contains("[features]"),
        "feature table present"
    );
}

#[test]
fn sema_boundary_historical_scope_rejects_forbidden_paths() {
    assert_sanctioned_paths("experimental/ac-semantics/core/src/lib.rs\ncrates/auto-lang/src/test_cases/sema.rs\ndocs/specs/auto-semantics/project.md");
    for path in [
        "",
        "Cargo.toml",
        "experimental/ac-artifact-probe/src/store.rs",
        "crates/auto-lang-core/src/vm/engine.rs",
        ".agents/skills/autoui-verifier/SKILL.md",
    ] {
        assert!(
            std::panic::catch_unwind(|| assert_sanctioned_paths(path)).is_err(),
            "original boundary must reject {path:?}"
        );
    }
}
