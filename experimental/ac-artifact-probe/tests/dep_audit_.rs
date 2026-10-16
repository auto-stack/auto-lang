//! V-12 scope/boundary audit tests: the probe workspace stays standalone,
//! ac-core does not gain reverse dependencies on product crates, and the
//! experimental entry is wired WITHOUT any production read/GC switch.

mod common;

use std::path::Path;

fn manifest_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn dep_audit_ac_core_has_no_product_dependencies() {
    let ac_core = manifest_dir().join("../ac-core/Cargo.toml");
    let text = std::fs::read_to_string(&ac_core).expect("ac-core Cargo.toml");
    // ac-core must never depend on product crates (its own doc contract).
    for banned in [
        "auto-cache",
        "auto-lang",
        "auto-atom",
        "auto-val",
        "auto-vm",
        "auto-down",
    ] {
        assert!(
            !text.contains(banned),
            "ac-core must not depend on {} (found in Cargo.toml)",
            banned
        );
    }
    assert!(
        text.contains("[workspace]"),
        "ac-core remains a standalone workspace"
    );
}

#[test]
fn dep_audit_probe_is_standalone_and_names_its_dependencies() {
    let probe = manifest_dir().join("Cargo.toml");
    let text = std::fs::read_to_string(&probe).unwrap();
    assert!(text.contains("[workspace]"), "probe is its own workspace");
    // The probe consumes ac-core's public API and the BlobStore adapter.
    assert!(text.contains("auto-ac-prototype"));
    assert!(text.contains("auto-cache"));
    // And nothing else from the product surface.
    for banned in ["auto-lang", "auto-atom", "auto-val", "auto-vm"] {
        assert!(
            !text.contains(banned),
            "probe must not depend on {}",
            banned
        );
    }
}

#[test]
fn dep_audit_probe_lives_only_under_experimental() {
    let probe_dir = manifest_dir();
    // Normalize via parent chain (CARGO_MANIFEST_DIR may carry ../ segments).
    let repo_root = probe_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("probe dir under experimental/")
        .to_path_buf();
    let rel = probe_dir.strip_prefix(&repo_root).unwrap();
    assert_eq!(
        Path::new("experimental/ac-artifact-probe"),
        rel,
        "the probe must live at experimental/ac-artifact-probe"
    );
}

#[test]
fn dep_audit_no_production_cache_entry_switch() {
    // The production cache entry points are unchanged: auto-cache's lib
    // exports and the ac-core lib exports mention nothing about the probe.
    let cache_lib = manifest_dir().join("../../crates/auto-cache/src/lib.rs");
    let text = std::fs::read_to_string(&cache_lib).unwrap();
    assert!(
        !text.contains("ac-artifact-probe"),
        "no probe coupling in auto-cache"
    );
    assert!(!text.contains("ac_artifact_probe"));
    let core_lib = manifest_dir().join("../ac-core/src/lib.rs");
    let text = std::fs::read_to_string(&core_lib).unwrap();
    assert!(
        !text.contains("artifact-probe"),
        "no probe coupling in ac-core"
    );
}

#[test]
fn dep_audit_schema_registry_covers_declared_codec_layers() {
    // The A00–A12 registry's codec subset matches what the driver actually
    // produces (V-12: 支持面与证据相符). PLAN-787 T-06: A01/A02/A03 carry real
    // syntax producers (ac-syntax-artifact-adapter)——codec 面含三者；其 chunk
    // 生产不在本探针 driver（生产者审计见 syntax 工作区 compatibility_）。
    let codec_layers: Vec<&str> = ac_artifact_probe::schema::codec_layers()
        .iter()
        .map(|l| l.id)
        .collect();
    assert_eq!(codec_layers, vec!["A00", "A01", "A02", "A03", "A04", "A07", "A09", "A10"]);
    // Every layer has a full boundary entry (sample coverage asserted in
    // the lib unit tests; here assert the public surface is queryable).
    for id in ["A00", "A03", "A07", "A12"] {
        let l = ac_artifact_probe::schema::layer(id).expect(id);
        assert!(!l.producer.is_empty());
        assert!(!l.storage.is_empty());
        assert!(!l.residency.is_empty());
    }
    // A03 (CST)：PLAN-787 T-06 起为真实语法产物（分页 CST 生产者 =
    // ac-syntax-artifact-adapter；探针合成人口已退役）。仍非完整语言产品
    // 主张——边界见 auto-syntax project Spec。
    assert!(matches!(
        ac_artifact_probe::schema::layer("A03").unwrap().coverage,
        ac_artifact_probe::schema::LayerCoverage::Codec
    ));
}
