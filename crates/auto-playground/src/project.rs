use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::AppError;

/// A single source file of a project example. `path` is the plain file name
/// (project examples are flat directories of `.at` files).
#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    pub path: String,
    pub source: String,
}

static TEMP_PROJECT_COUNTER: AtomicU64 = AtomicU64::new(0);

fn create_temp_project_dir() -> Result<PathBuf, AppError> {
    let root = std::env::temp_dir();
    for _ in 0..100 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let sequence = TEMP_PROJECT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = root.join(format!(
            "auto-playground-project-{}-{nanos}-{sequence}",
            std::process::id()
        ));
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(AppError::Internal(format!(
                    "Failed to create temp dir: {error}"
                )))
            }
        }
    }
    Err(AppError::Internal(
        "Failed to allocate a unique temp project directory".to_string(),
    ))
}

fn safe_relative_at_path(rel: &str) -> Option<PathBuf> {
    let path = Path::new(rel);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || !path.extension().is_some_and(|ext| ext == "at")
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    Some(path.to_path_buf())
}

fn safe_relative_project_file(rel: &str) -> Option<PathBuf> {
    let path = Path::new(rel);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.file_name().is_none()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    Some(path.to_path_buf())
}

fn safe_relative_project_dir(rel: &str) -> Option<PathBuf> {
    let path = Path::new(rel);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    Some(path.to_path_buf())
}

pub fn project_examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("examples/playground-demo"))
        .unwrap_or_else(|| PathBuf::from("examples/playground-demo"))
}

/// Build a temporary copy of the project directory, overwriting `main.at` with
/// the edited entry source. When `overlay` is provided, those files replace the
/// disk copies as well (the frontend sends the full edited file set).
/// Returns the temp directory path.
pub fn prepare_project_temp_dir(
    source: &str,
    project_dir: &str,
    overlay: Option<&[ProjectFile]>,
) -> Result<PathBuf, AppError> {
    let project_rel = safe_relative_project_dir(project_dir)
        .ok_or_else(|| AppError::VmError(format!("Invalid project_dir: {project_dir}")))?;
    let examples_root = project_examples_dir()
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("Failed to resolve project examples: {e}")))?;
    let original_dir = examples_root.join(project_rel);
    if !original_dir.is_dir() {
        return Err(AppError::Internal(format!(
            "Project directory not found: {}",
            original_dir.display()
        )));
    }
    let original_dir = original_dir
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("Failed to resolve project dir: {e}")))?;
    if !original_dir.starts_with(&examples_root) {
        return Err(AppError::VmError(
            "project_dir escapes the examples root".to_string(),
        ));
    }
    let temp_dir = create_temp_project_dir()?;
    let result = (|| {
        for entry in std::fs::read_dir(&original_dir)
            .map_err(|e| AppError::Internal(format!("Failed to read project dir: {e}")))?
        {
            let entry =
                entry.map_err(|e| AppError::Internal(format!("Failed to read dir entry: {e}")))?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "at") {
                let dest = temp_dir.join(path.file_name().unwrap_or_default());
                std::fs::copy(&path, &dest).map_err(|e| {
                    AppError::Internal(format!("Failed to copy {}: {e}", path.display()))
                })?;
            }
        }

        if let Some(files) = overlay {
            for file in files {
                let rel = safe_relative_project_file(&file.path).ok_or_else(|| {
                    AppError::VmError(format!("Invalid project file path: {}", file.path))
                })?;
                let dest = temp_dir.join(rel);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        AppError::Internal(format!("Failed to create project directory: {e}"))
                    })?;
                }
                std::fs::write(&dest, &file.source).map_err(|e| {
                    AppError::Internal(format!("Failed to write {}: {e}", file.path))
                })?;
            }
        }

        std::fs::write(temp_dir.join("main.at"), source)
            .map_err(|e| AppError::Internal(format!("Failed to write main.at: {e}")))?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(error);
    }
    Ok(temp_dir)
}

/// Materialize a files-only project (no server-side `project_dir` — e.g. notes
/// from the manifest, Plan 582): every file is written at its relative path
/// (nested dirs allowed — `auto/<lib>.at` mirrors the parity lib-root layout),
/// and `source` is written as the entry `main.at` at the temp root. Module
/// resolution seeds `source_dirs` with the entry's dir (= temp root), so
/// `use auto.<lib>` resolves `<temp>/auto/<lib>.at`. Returns `(temp_dir, entry_path)`.
/// Materialize a source file at a caller-selected relative `.at` entry path.
pub fn prepare_files_temp_dir_with_entry(
    source: &str,
    files: &[ProjectFile],
    entry: Option<&str>,
) -> Result<(PathBuf, PathBuf), AppError> {
    let entry = entry.unwrap_or("main.at");
    let entry_rel = safe_relative_at_path(entry)
        .ok_or_else(|| AppError::VmError(format!("Invalid project entry path: {entry}")))?;
    let entry_key = entry_rel.to_string_lossy().replace('\\', "/");
    let mut seen = HashSet::new();
    let mut validated = Vec::with_capacity(files.len());
    for file in files {
        let rel = safe_relative_project_file(&file.path).ok_or_else(|| {
            AppError::VmError(format!("Invalid project file path: {}", file.path))
        })?;
        let key = rel.to_string_lossy().replace('\\', "/");
        if key != entry_key && !seen.insert(key.clone()) {
            return Err(AppError::VmError(format!(
                "Duplicate project file path: {}",
                file.path
            )));
        }
        validated.push((key, rel, file));
    }

    let temp_dir = create_temp_project_dir()?;
    let result = (|| {
        for (key, rel, file) in validated {
            // `source` is authoritative for the entry, even if the frontend
            // also includes an entry record in its complete project file set.
            if key == entry_key {
                continue;
            }
            let dest = temp_dir.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::Internal(format!("Failed to create dir for {}: {e}", file.path))
                })?;
            }
            std::fs::write(&dest, &file.source)
                .map_err(|e| AppError::Internal(format!("Failed to write {}: {e}", file.path)))?;
        }

        let entry_path = temp_dir.join(&entry_rel);
        if let Some(parent) = entry_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Internal(format!("Failed to create entry directory: {e}"))
            })?;
        }
        std::fs::write(&entry_path, source)
            .map_err(|e| AppError::Internal(format!("Failed to write entry {entry}: {e}")))?;
        Ok(entry_path)
    })();
    match result {
        Ok(entry_path) => Ok((temp_dir, entry_path)),
        Err(error) => {
            let _ = std::fs::remove_dir_all(&temp_dir);
            Err(error)
        }
    }
}
