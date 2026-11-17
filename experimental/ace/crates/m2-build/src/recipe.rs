//! PLAN-978 T-08/T-09 (AC-06/AC-07, V-08/V-09/V-ID-PACK): the
//! relocatable CANDIDATE SDK recipe.
//!
//! Layout (one relocatable directory — the same probing contract the
//! language-side loader already speaks: `install-manifest.json` at the
//! root, `stdlib/auto/...` for the declared stdlib closure):
//!
//! ```text
//! <sdk-root>/
//!   install-manifest.json        <- the recipe document (this schema)
//!   sources/<project>/<file>.at  <- the declared Auto source closure
//!   targets/<target>/...         <- per-target products (exe / filesets)
//!   providers/<id>...            <- precompiled provider artifacts
//!   stdlib/auto/...              <- the declared stdlib closure (optional)
//! ```
//!
//! Frozen rules (plan §5 ID-05, V-ID-PACK):
//! * every path inside the recipe is RELATIVE — absolute paths and `..`
//!   escapes are rejected (`m2build.recipe.dev-tree-escape`): a relocated
//!   SDK must resolve against ITS OWN root, never the development tree;
//! * `load_verify` hash-checks every declared artifact, rejects missing
//!   and UNDECLARED files (`m2build.recipe.extra-file`) and schema drift;
//! * build tools live ONLY in `build_inputs` (provenance) — the runtime
//!   closure is `run_dependencies` and the run lanes never consult the
//!   build inputs (a recorded rustc identity does not make rustc a
//!   runtime requirement, and its absence does not refuse a run).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::compat::ConsumerRequirements;
use crate::identity::{ProjectBuildIdentity, SourceFile};
use crate::selection::ProviderSelection;
use crate::M2Reject;

pub const SDK_RECIPE_SCHEMA: &str = "ace-m2-build/sdk-recipe/1";
pub const INSTALL_MANIFEST: &str = "install-manifest.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildToolDependency {
    pub name: String,
    pub identity: String,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunDependency {
    pub name: String,
    /// crt | interpreter | system-lib | auto-source | av-host | provider | precompiled-program
    pub kind: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdkArtifact {
    /// POSIX-style relative path inside the SDK root.
    pub path: String,
    /// exe | fileset | manifest | provider | receipt
    pub role: String,
    pub sha256: String,
    pub size: u64,
}

/// The candidate SDK recipe — the single document a consumer verifies
/// and runs from, after relocation, with no development tree present.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdkRecipe {
    pub schema: String,
    pub project: String,
    pub entry: String,
    pub caps: Vec<String>,
    pub targets: Vec<String>,
    pub sources: Vec<SourceFile>,
    pub artifacts: Vec<SdkArtifact>,
    pub build_inputs: Vec<BuildToolDependency>,
    pub run_dependencies: Vec<RunDependency>,
    pub consumer_requirements: ConsumerRequirements,
    pub provider_selection: Option<ProviderSelection>,
    pub provenance: Vec<ProjectBuildIdentity>,
}

/// In-memory payload written alongside the recipe document.
pub struct SdkPayload {
    /// relative path → bytes
    pub files: Vec<(String, Vec<u8>)>,
}

impl SdkRecipe {
    /// Verify a relative path is inside the SDK closure (no absolute, no
    /// `..`, no drive letters).
    pub fn safe_rel(path: &str) -> Result<(), M2Reject> {
        let p = Path::new(path);
        if p.is_absolute()
            || path.contains("..")
            || path.contains(':')
            || path.starts_with('/')
            || path.starts_with('\\')
        {
            return Err(M2Reject::new(
                "m2build.recipe.dev-tree-escape",
                format!(
                    "`{path}` is not a safe relative SDK path — a relocated SDK \
                     resolves against its own root, never another tree"
                ),
            ));
        }
        Ok(())
    }

    /// Write the SDK directory: the recipe document plus every payload
    /// file at its declared relative path. Refuses unsafe paths and
    /// payload files not declared in the recipe.
    pub fn write_to_dir(&self, root: &Path, payload: &SdkPayload) -> Result<(), M2Reject> {
        std::fs::create_dir_all(root).map_err(|e| {
            M2Reject::new(
                "m2build.recipe.io",
                format!("create {}: {e}", root.display()),
            )
        })?;
        let declared: BTreeSet<&str> = self
            .artifacts
            .iter()
            .map(|a| a.path.as_str())
            .chain(self.sources.iter().map(|s| {
                // sources live under sources/<project>/
                s.path.as_str()
            }))
            .collect();
        for (rel, bytes) in &payload.files {
            Self::safe_rel(rel)?;
            if !declared.contains(rel.as_str())
                && !self
                    .sources
                    .iter()
                    .any(|s| format!("sources/{}/{}", self.project, s.path) == *rel)
                && !self.stdlib_files().iter().any(|p| p == rel)
            {
                return Err(M2Reject::new(
                    "m2build.recipe.undeclared-file",
                    format!("payload file `{rel}` is not declared in the recipe"),
                ));
            }
            let full = root.join(PathBuf::from(rel.replace('/', "\\")));
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    M2Reject::new(
                        "m2build.recipe.io",
                        format!("create {}: {e}", parent.display()),
                    )
                })?;
            }
            std::fs::write(&full, bytes).map_err(|e| {
                M2Reject::new(
                    "m2build.recipe.io",
                    format!("write {}: {e}", full.display()),
                )
            })?;
        }
        // The recipe document itself, last (its presence marks a complete
        // write).
        let doc = serde_json::to_vec_pretty(self)
            .map_err(|e| M2Reject::new("m2build.recipe.io", format!("encode recipe: {e}")))?;
        std::fs::write(root.join(INSTALL_MANIFEST), doc).map_err(|e| {
            M2Reject::new(
                "m2build.recipe.io",
                format!("write {}: {e}", root.join(INSTALL_MANIFEST).display()),
            )
        })?;
        Ok(())
    }

    fn stdlib_files(&self) -> Vec<String> {
        self.provenance
            .iter()
            .flat_map(|p| p.stdlib.iter())
            .map(|s| s.path.clone())
            .collect()
    }

    /// Load + verify a candidate SDK directory:
    /// * the recipe document parses and its schema is exact;
    /// * every declared artifact exists and its bytes hash to the
    ///   declared sha256 (a corrupt or truncated artifact is a named
    ///   rejection, never a silent substitution);
    /// * every file under the root (except the recipe document and the
    ///   declared stdlib closure) is declared — undeclared extras are
    ///   rejected (a smuggled dependency must not ride along);
    /// * every declared path is a safe relative path.
    pub fn load_verify(root: &Path) -> Result<(SdkRecipe, Vec<(String, Vec<u8>)>), M2Reject> {
        let doc_path = root.join(INSTALL_MANIFEST);
        let doc = std::fs::read(&doc_path).map_err(|e| {
            M2Reject::new(
                "m2build.recipe.missing",
                format!("read {}: {e}", doc_path.display()),
            )
        })?;
        let recipe: SdkRecipe = serde_json::from_slice(&doc)
            .map_err(|e| M2Reject::new("m2build.recipe.schema", format!("decode recipe: {e}")))?;
        if recipe.schema != SDK_RECIPE_SCHEMA {
            return Err(M2Reject::new(
                "m2build.recipe.schema",
                format!(
                    "recipe schema `{}` != expected `{SDK_RECIPE_SCHEMA}`",
                    recipe.schema
                ),
            ));
        }
        let mut out: Vec<(String, Vec<u8>)> = Vec::new();
        for artifact in &recipe.artifacts {
            Self::safe_rel(&artifact.path)?;
            let full = root.join(PathBuf::from(artifact.path.replace('/', "\\")));
            let bytes = std::fs::read(&full).map_err(|_| {
                M2Reject::new(
                    "m2build.recipe.missing",
                    format!("declared artifact `{}` is missing", artifact.path),
                )
            })?;
            let sha = ace_a2x_core::sha256::sha256_hex(&bytes);
            if sha != artifact.sha256 {
                return Err(M2Reject::new(
                    "m2build.recipe.integrity",
                    format!(
                        "artifact `{}` hash drift: declared {}, on-disk {sha}",
                        artifact.path, artifact.sha256
                    ),
                ));
            }
            out.push((artifact.path.clone(), bytes));
        }
        // source closure
        for source in &recipe.sources {
            Self::safe_rel(&source.path)?;
            let rel = format!("sources/{}/{}", recipe.project, source.path);
            let full = root.join(PathBuf::from(rel.replace('/', "\\")));
            let bytes = std::fs::read_to_string(&full).map_err(|_| {
                M2Reject::new(
                    "m2build.recipe.missing",
                    format!("declared source `{rel}` is missing"),
                )
            })?;
            let sha = ace_a2x_core::sha256::sha256_hex(bytes.as_bytes());
            if sha != source.sha256 {
                return Err(M2Reject::new(
                    "m2build.recipe.integrity",
                    format!("source `{rel}` hash drift: declared {}", source.sha256),
                ));
            }
            out.push((rel, bytes.into_bytes()));
        }
        // stdlib closure (declared under provenance, lives at its own path)
        for path in recipe.stdlib_files() {
            Self::safe_rel(&path)?;
            let full = root.join(PathBuf::from(path.replace('/', "\\")));
            if full.is_file() {
                let bytes = std::fs::read(&full)
                    .map_err(|e| M2Reject::new("m2build.recipe.io", format!("read {path}: {e}")))?;
                out.push((path.clone(), bytes));
            }
        }
        // undeclared-extra scan
        let mut allowed: BTreeSet<String> =
            recipe.artifacts.iter().map(|a| a.path.clone()).collect();
        for s in &recipe.sources {
            allowed.insert(format!("sources/{}/{}", recipe.project, s.path));
        }
        for p in recipe.stdlib_files() {
            allowed.insert(p);
        }
        allowed.insert(INSTALL_MANIFEST.to_string());
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let entries = std::fs::read_dir(&dir)
                .map_err(|e| M2Reject::new("m2build.recipe.io", format!("read_dir: {e}")))?;
            for entry in entries.flatten() {
                let full = entry.path();
                if full.is_dir() {
                    stack.push(full);
                    continue;
                }
                let rel = full
                    .strip_prefix(root)
                    .unwrap_or(&full)
                    .to_string_lossy()
                    .replace('\\', "/");
                if !allowed.contains(&rel) {
                    return Err(M2Reject::new(
                        "m2build.recipe.extra-file",
                        format!(
                            "`{rel}` is present but not declared — a smuggled \
                             dependency cannot ride along"
                        ),
                    ));
                }
            }
        }
        Ok((recipe, out))
    }

    /// The source text of the project's public entry source, verified.
    pub fn entry_source(
        recipe: &SdkRecipe,
        files: &[(String, Vec<u8>)],
    ) -> Result<String, M2Reject> {
        let publics: Vec<&SourceFile> = recipe
            .sources
            .iter()
            .filter(|s| s.role == "public")
            .collect();
        if publics.len() != 1 {
            return Err(M2Reject::new(
                "m2build.recipe.entry-source",
                format!(
                    "expected exactly one `public` source, found {}",
                    publics.len()
                ),
            ));
        }
        let rel = format!("sources/{}/{}", recipe.project, publics[0].path);
        let bytes = files.iter().find(|(p, _)| *p == rel).ok_or_else(|| {
            M2Reject::new(
                "m2build.recipe.missing",
                format!("verified source `{rel}` not in the loaded payload"),
            )
        })?;
        String::from_utf8(bytes.1.clone())
            .map_err(|e| M2Reject::new("m2build.recipe.schema", format!("source not UTF-8: {e}")))
    }
}
