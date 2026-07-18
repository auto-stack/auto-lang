use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::AppError;

/// A single source file of a project example. `path` is the plain file name
/// (project examples are flat directories of `.at` files).
#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    pub path: String,
    pub source: String,
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
    let original_dir = project_examples_dir().join(project_dir);
    if !original_dir.is_dir() {
        return Err(AppError::Internal(format!(
            "Project directory not found: {}",
            original_dir.display()
        )));
    }

    let temp_id = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let temp_dir = std::env::temp_dir().join(format!("auto-playground-project-{}", temp_id));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| AppError::Internal(format!("Failed to create temp dir: {e}")))?;

    for entry in std::fs::read_dir(&original_dir)
        .map_err(|e| AppError::Internal(format!("Failed to read project dir: {e}")))?
    {
        let entry = entry.map_err(|e| AppError::Internal(format!("Failed to read dir entry: {e}")))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "at") {
            let dest = temp_dir.join(path.file_name().unwrap_or_default());
            std::fs::copy(&path, &dest)
                .map_err(|e| AppError::Internal(format!("Failed to copy {}: {e}", path.display())))?;
        }
    }

    if let Some(files) = overlay {
        for file in files {
            // Sanitize: only plain `.at` file names may be written into the
            // temp dir — never allow path traversal.
            let name = std::path::Path::new(&file.path)
                .file_name()
                .unwrap_or_default();
            if name.is_empty() || std::path::Path::new(name).extension().is_none_or(|ext| ext != "at") {
                continue;
            }
            std::fs::write(temp_dir.join(name), &file.source)
                .map_err(|e| AppError::Internal(format!("Failed to write {}: {e}", file.path)))?;
        }
    }

    let main_path = temp_dir.join("main.at");
    std::fs::write(&main_path, source)
        .map_err(|e| AppError::Internal(format!("Failed to write main.at: {e}")))?;

    Ok(temp_dir)
}
