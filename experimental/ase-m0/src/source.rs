//! Reading the registered source closure under the allowed root
//! (PLAN-1021 AC-02): paths come from the trusted metadata directory and
//! are never built from the URI. The reader refuses, by name and before
//! any file is opened for content: root escapes (`..`, absolute paths),
//! reparse points (junctions/symlinks) anywhere strictly below the
//! registry directory, and missing pieces. This bounded root check is NOT
//! a sandbox claim.

use crate::error::{AseError, Kind, Phase};
use crate::registry::{Registry, RegistryApp};
use sha2::{Digest, Sha256};
use std::path::{Component, Path};

/// One loaded source module with its content digest (sha256, first 16 hex).
#[derive(Clone, Debug)]
pub struct LoadedSource {
    pub module: String,
    pub file: String,
    pub source: String,
    pub sha256_16: String,
}

fn escape(message: impl Into<String>) -> AseError {
    AseError::new(Phase::Source, Kind::SourceEscape, message)
}

/// Every component must be a plain relative `Normal` segment — no
/// parent/curdir references, no absolute prefixes.
fn require_normal(path: &Path, what: &str) -> Result<(), AseError> {
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            other => {
                return Err(escape(format!(
                    "{what} {path:?} contains non-normal component {other:?}"
                )))
            }
        }
    }
    Ok(())
}

/// The joined candidate path, checked component by component strictly
/// below the registry directory: any reparse point is refused by name.
fn reject_reparse_below(registry_dir: &Path, candidate: &Path) -> Result<(), AseError> {
    let mut current = registry_dir.to_path_buf();
    for component in candidate
        .components()
        .skip(registry_dir.components().count())
    {
        current.push(component);
        let file_type = std::fs::symlink_metadata(&current)
            .map_err(|e| {
                AseError::new(
                    Phase::Source,
                    Kind::SourceMissing,
                    format!("stat {}: {e}", current.display()),
                )
            })?
            .file_type();
        if file_type.is_symlink() {
            return Err(AseError::new(
                Phase::Source,
                Kind::SourceReparse,
                format!(
                    "reparse point {} refused inside the allowed root",
                    current.display()
                ),
            ));
        }
    }
    Ok(())
}

pub fn read_sources(registry: &Registry, app: &RegistryApp) -> Result<Vec<LoadedSource>, AseError> {
    if app.root.is_absolute() {
        return Err(escape(format!(
            "root {:?} for {} must be relative to the metadata directory",
            app.root.display(),
            app.identity
        )));
    }
    require_normal(&app.root, "root")?;
    let registry_canon = std::fs::canonicalize(&registry.dir).map_err(|e| {
        AseError::new(
            Phase::Source,
            Kind::SourceIo,
            format!("canonicalize {}: {e}", registry.dir.display()),
        )
    })?;
    let mut loaded = Vec::with_capacity(app.sources.len());
    for file in &app.sources {
        let file_path = Path::new(file);
        require_normal(file_path, "source file")?;
        if file_path.components().count() != 1 {
            return Err(escape(format!(
                "source file {file:?} must be a single name under the app root"
            )));
        }
        let candidate = registry.dir.join(&app.root).join(file_path);
        reject_reparse_below(&registry.dir, &candidate)?;
        let canon = std::fs::canonicalize(&candidate).map_err(|_| {
            AseError::new(
                Phase::Source,
                Kind::SourceMissing,
                format!("registered source {} is missing", candidate.display()),
            )
        })?;
        if !canon.starts_with(&registry_canon) {
            return Err(escape(format!(
                "registered source {} escapes the metadata directory {}",
                candidate.display(),
                registry.dir.display()
            )));
        }
        let metadata = std::fs::metadata(&canon).map_err(|_| {
            AseError::new(
                Phase::Source,
                Kind::SourceMissing,
                format!("registered source {} is missing", candidate.display()),
            )
        })?;
        if !metadata.is_file() {
            return Err(AseError::new(
                Phase::Source,
                Kind::SourceMissing,
                format!(
                    "registered source {} is not a regular file",
                    candidate.display()
                ),
            ));
        }
        let source = std::fs::read_to_string(&canon).map_err(|e| {
            AseError::new(
                Phase::Source,
                Kind::SourceIo,
                format!("read {}: {e}", canon.display()),
            )
        })?;
        let module = file_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| file.clone());
        let sha256_16 = {
            let digest = Sha256::digest(source.as_bytes());
            let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
            hex[..16].to_string()
        };
        loaded.push(LoadedSource {
            module,
            file: file.clone(),
            source,
            sha256_16,
        });
    }
    Ok(loaded)
}
