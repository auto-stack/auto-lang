//! PLAN-978 T-09 (AC-LOAD, V-LOAD): candidate-SDK source-layer admission
//! — the file-level selection of the PLAN-1043 suffix assembly
//! (public `.at` + `<base>.<suffix>.at` provider layers) into a declared
//! SDK closure.
//!
//! This is the SDK-side SELECTION ledger only: which layer files a
//! declared target selects, with named rejections for the wrong-suffix /
//! missing-public / duplicate-implementation / unknown-suffix classes.
//! The GRAMMAR-level merge semantics stay with their owner (PLAN-1043 /
//! the language-side assembly plan); this crate never invents a merge.

use serde::{Deserialize, Serialize};

use crate::M2Reject;

pub const LOAD_SELECTION_SCHEMA: &str = "ace-m2-build/load-selection/1";

/// The layer suffixes the SDK closure understands (the 1043 frozen set).
pub const KNOWN_SUFFIXES: &[&str] = &["rs", "c", "py"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerFile {
    /// Bare module name (the suffix-stripped stem).
    pub module: String,
    /// `public` or a known suffix.
    pub layer: String,
    pub file: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadSelection {
    pub schema: String,
    pub target: String,
    /// The layers this target SELECTS (public + the target's suffix layer
    /// where present).
    pub selected: Vec<LayerFile>,
    /// Same-directory unselected provider layers (never "actual").
    pub candidate_files: Vec<LayerFile>,
}

/// Split a source file name into (module, Option<suffix>). `main.at` →
/// ("main", None); `math.rs.at` → ("math", Some("rs")); `a.at.at` →
/// unknown suffix (rejected); bare `x.rs` is not an Auto layer.
pub fn split_layer_name(file_name: &str) -> Result<(String, Option<String>), M2Reject> {
    let stem = file_name.strip_suffix(".at").ok_or_else(|| {
        M2Reject::new(
            "m2build.load.not-auto-layer",
            format!("`{file_name}` is not an Auto layer file (expected `.at`)"),
        )
    })?;
    let (module, suffix) = match stem.rsplit_once('.') {
        Some((m, s)) => (m, Some(s.to_string())),
        None => (stem, None),
    };
    if let Some(s) = &suffix {
        if !KNOWN_SUFFIXES.contains(&s.as_str()) {
            return Err(M2Reject::new(
                "m2build.load.unknown-suffix",
                format!(
                    "`{file_name}`: unknown provider suffix `.{s}` (known: {})",
                    KNOWN_SUFFIXES.join("/")
                ),
            ));
        }
    }
    Ok((module.to_string(), suffix))
}

/// Select the SDK closure's layers for one target.
///
/// Rejections (all BEFORE any binding):
/// * `m2build.load.missing-public` — a provider layer exists for a module
///   with no public layer;
/// * `m2build.load.duplicate-layer` — two files claim the same (module,
///   layer) role;
/// * `m2build.load.unknown-suffix` / `not-auto-layer` — from splitting.
pub fn select_layers(
    files: &[(String, String)], // (file NAME, content) — caller hashes
    target_suffix: Option<&str>,
) -> Result<LoadSelection, M2Reject> {
    let mut layers: Vec<LayerFile> = Vec::new();
    for (name, content) in files {
        let (module, suffix) = split_layer_name(name)?;
        let entry = LayerFile {
            module,
            layer: suffix.unwrap_or_else(|| "public".into()),
            file: name.clone(),
            sha256: ace_a2x_core::sha256::sha256_hex(content.as_bytes()),
        };
        if layers
            .iter()
            .any(|l| l.module == entry.module && l.layer == entry.layer)
        {
            return Err(M2Reject::new(
                "m2build.load.duplicate-layer",
                format!(
                    "two files claim the same ({}, {}) layer role",
                    entry.module, entry.layer
                ),
            ));
        }
        layers.push(entry);
    }
    // A provider layer without its public layer cannot be selected by any
    // target (the public face defines the module).
    for l in &layers {
        if l.layer != "public"
            && !layers
                .iter()
                .any(|p| p.module == l.module && p.layer == "public")
        {
            return Err(M2Reject::new(
                "m2build.load.missing-public",
                format!(
                    "provider layer `{}` has no public `{}` layer in the declared closure",
                    l.file, l.module
                ),
            ));
        }
    }
    let (selected, candidate_files): (Vec<LayerFile>, Vec<LayerFile>) = layers
        .into_iter()
        .partition(|l| l.layer == "public" || target_suffix == Some(l.layer.as_str()));
    Ok(LoadSelection {
        schema: LOAD_SELECTION_SCHEMA.into(),
        target: target_suffix.unwrap_or("public").into(),
        selected,
        candidate_files,
    })
}
