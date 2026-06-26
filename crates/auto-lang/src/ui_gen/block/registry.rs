//! [`BlockRegistry`] — scans `blocks/` and indexes block packages (Plan 342).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::spec::BlockSpec;
use crate::ui_gen::WidgetRegistry;

/// A discovered block package on disk.
#[derive(Debug, Clone)]
pub struct BlockPackage {
    pub spec: BlockSpec,
    /// `<repo>/blocks/<kind>/<name>` directory.
    pub dir: PathBuf,
    /// variant -> `<dir>/reference/<variant>.at` (validated to exist).
    pub references: HashMap<String, PathBuf>,
    /// `<dir>/gotchas.md`, if present.
    pub gotchas: Option<PathBuf>,
}

impl BlockPackage {
    /// Catalog key (`kind/name`).
    pub fn key(&self) -> String {
        format!("{}/{}", self.spec.kind, self.spec.name)
    }
}

/// Indexes block packages under a root directory (default: `<repo>/blocks`).
#[derive(Debug, Clone)]
pub struct BlockRegistry {
    packages: Vec<BlockPackage>,
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self { packages: Vec::new() }
    }

    /// Discover packages under `root` (each child dir of `root/<kind>/<name>/`
    /// that contains a `spec.md`). Errors in individual packages are collected
    /// rather than aborting the whole scan, but at least the well-formed ones
    /// are indexed.
    ///
    /// If `root` does not exist, returns an empty registry (not an error) so
    /// callers can scan provisionally.
    pub fn scan_dir(root: impl AsRef<Path>) -> Result<Self, Vec<String>> {
        let root = root.as_ref();
        let mut packages = Vec::new();
        let mut errors = Vec::new();
        if !root.is_dir() {
            return Ok(Self { packages });
        }
        // root/<kind>/<name>/spec.md
        for kind_entry in fs::read_dir(root).map_err(|e| vec![format!("read {root:?}: {e}")])? {
            let kind_entry = match kind_entry {
                Ok(e) => e,
                Err(e) => {
                    errors.push(format!("readdir entry: {e}"));
                    continue;
                }
            };
            if !kind_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            for name_entry in fs::read_dir(kind_entry.path()).into_iter().flatten() {
                let name_entry = match name_entry {
                    Ok(e) => e,
                    Err(e) => {
                        errors.push(format!("readdir entry: {e}"));
                        continue;
                    }
                };
                if !name_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let dir = name_entry.path();
                match load_package(&dir) {
                    Ok(pkg) => packages.push(pkg),
                    Err(e) => errors.push(format!("{}: {e}", dir.display())),
                }
            }
        }
        packages.sort_by(|a, b| a.key().cmp(&b.key()));
        Ok(Self { packages })
    }

    /// Scan the default `<repo>/blocks` directory, resolving the repo root from
    /// this crate's `CARGO_MANIFEST_DIR` (`crates/auto-lang`).
    pub fn with_defaults() -> Self {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2) // crates/auto-lang -> crates -> <repo>
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let blocks_root = repo_root.join("blocks");
        Self::scan_dir(&blocks_root).unwrap_or_else(|errors| {
            // Surface scan errors at runtime via debug; return empty for resilience.
            log::debug!("block scan errors: {errors:?}");
            Self::new()
        })
    }

    pub fn packages(&self) -> &[BlockPackage] {
        &self.packages
    }

    pub fn iter(&self) -> impl Iterator<Item = &BlockPackage> {
        self.packages.iter()
    }

    pub fn get(&self, kind: &str, name: &str) -> Option<&BlockPackage> {
        self.packages
            .iter()
            .find(|p| p.spec.kind == kind && p.spec.name == name)
    }

    pub fn list_by_kind<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a BlockPackage> + 'a {
        self.packages.iter().filter(move |p| p.spec.kind == kind)
    }

    /// Cross-check every package against the widget registry: each `palette`
    /// entry must exist as an AURA widget tag (exact or prefix-grouped, as in
    /// Plan 337). Returns the list of violations (empty = clean).
    pub fn palette_drift(&self, widgets: &WidgetRegistry) -> Vec<String> {
        let tags: std::collections::HashSet<&str> =
            widgets.all_widgets().keys().map(|s| s.as_str()).collect();
        let mut drift = Vec::new();
        for pkg in &self.packages {
            for w in &pkg.spec.palette {
                let known = tags.contains(w.as_str())
                    || tags.iter().any(|t| t.starts_with(&format!("{w}-")));
                if !known {
                    drift.push(format!("{}: palette widget '{}' not in AURA registry", pkg.key(), w));
                }
            }
        }
        drift
    }
}

fn load_package(dir: &Path) -> Result<BlockPackage, String> {
    let spec_path = dir.join("spec.md");
    let spec_md =
        fs::read_to_string(&spec_path).map_err(|e| format!("read {}: {e}", spec_path.display()))?;
    let (spec, _body) = BlockSpec::parse_document(&spec_md)?;

    // Each declared variant must have reference/<variant>.at.
    let mut references = HashMap::new();
    for variant in &spec.variants {
        let path = dir.join("reference").join(format!("{variant}.at"));
        if !path.is_file() {
            return Err(format!(
                "spec declares variant '{variant}' but {} is missing",
                path.display()
            ));
        }
        references.insert(variant.clone(), path);
    }
    // If no variants declared, accept any `reference/*.at` and treat the stem
    // list as the de-facto variants (so single-variant blocks stay simple).
    if spec.variants.is_empty() {
        let ref_dir = dir.join("reference");
        if ref_dir.is_dir() {
            for entry in fs::read_dir(&ref_dir).map_err(|e| format!("read {ref_dir:?}: {e}"))? {
                let entry = entry.map_err(|e| format!("readdir: {e}"))?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("at") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        references.insert(stem.to_string(), path);
                    }
                }
            }
        }
    }

    let gotchas_path = dir.join("gotchas.md");
    let gotchas = if gotchas_path.is_file() {
        Some(gotchas_path)
    } else {
        None
    };

    // If spec.name differs from the directory name, that's likely a mistake.
    let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if dir_name != spec.name {
        return Err(format!(
            "spec name '{}' does not match directory name '{dir_name}'",
            spec.name
        ));
    }

    Ok(BlockPackage {
        spec,
        dir: dir.to_path_buf(),
        references,
        gotchas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry_when_root_missing() {
        let reg = BlockRegistry::scan_dir("/does/not/exist/anywhere/here").unwrap();
        assert!(reg.packages().is_empty());
    }
}
