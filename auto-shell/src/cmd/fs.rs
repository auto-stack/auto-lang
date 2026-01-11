//! File system commands
//!
//! Implements core file system operations: ls, cd, mkdir, rm, mv, cp

use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::data::{Table, Column, Align, FileEntry};

/// List directory contents with table formatting
pub fn ls_command(path: &Path, current_dir: &Path) -> Result<String> {
    let target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };

    if !target.exists() {
        miette::bail!("ls: {}: No such file or directory", target.display());
    }

    // If it's a file, just return its name
    if target.is_file() {
        return Ok(target.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string());
    }

    // List directory contents
    let entries = fs::read_dir(&target).into_diagnostic()?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.into_diagnostic()?;
        let metadata = entry.metadata().into_diagnostic()?;

        let name = entry.file_name()
            .into_string()
            .unwrap_or_else(|_| "?".to_string());

        let is_dir = entry.path().is_dir();

        // Get file size
        let size = if is_dir {
            None
        } else {
            Some(metadata.len())
        };

        // Get modified time
        let modified = metadata.modified()
            .ok()
            .and_then(|time| {
                use std::time::UNIX_EPOCH;
                let secs = time.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
                let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)?;
                Some(datetime.format("%Y-%m-%d %H:%M").to_string())
            });

        files.push(FileEntry {
            name,
            is_dir,
            size,
            modified,
        });
    }

    // Sort by name
    files.sort_by(|a, b| {
        // Directories first
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.cmp(&b.name)
        }
    });

    // Create table
    let mut table = Table::new()
        .add_column(Column::new("Name").align(Align::Left))
        .add_column(Column::new("Size").align(Align::Right))
        .add_column(Column::new("Modified").align(Align::Left));

    // Add rows
    for file in &files {
        let name_with_indicator = if file.is_dir {
            format!("{}/", file.name)
        } else {
            file.name.clone()
        };

        table = table.add_row(vec![
            name_with_indicator,
            file.format_size(),
            file.modified.clone().unwrap_or_else(|| "-".to_string()),
        ]);
    }

    // Calculate widths and render
    table.calculate_widths();
    Ok(table.render())
}

/// Change directory (returns new path if successful)
pub fn cd_command(path: &Path, current_dir: &Path) -> Result<PathBuf> {
    let new_dir = if path.is_absolute() {
        path.to_path_buf()
    } else if path.starts_with("~") {
        // Expand ~ to home directory
        dirs::home_dir().unwrap_or_else(|| current_dir.to_path_buf())
            .join(path.strip_prefix("~").unwrap_or(Path::new("")))
    } else {
        current_dir.join(path)
    };

    // Try to canonicalize the path
    let canonical = new_dir.canonicalize().into_diagnostic()?;

    if canonical.is_dir() {
        Ok(canonical)
    } else {
        miette::bail!("cd: {}: Not a directory", path.display());
    }
}

/// Make directory
pub fn mkdir_command(path: &Path, current_dir: &Path, parents: bool) -> Result<String> {
    let target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };

    if parents {
        fs::create_dir_all(&target).into_diagnostic()?;
    } else {
        fs::create_dir(&target).into_diagnostic()?;
    }

    Ok(String::new()) // mkdir typically produces no output
}

/// Remove file or directory
pub fn rm_command(path: &Path, current_dir: &Path, recursive: bool) -> Result<String> {
    let target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };

    if !target.exists() {
        miette::bail!("rm: {}: No such file or directory", target.display());
    }

    if target.is_dir() {
        if recursive {
            fs::remove_dir_all(&target).into_diagnostic()?;
        } else {
            miette::bail!("rm: {}: Is a directory (use -r)", target.display());
        }
    } else {
        fs::remove_file(&target).into_diagnostic()?;
    }

    Ok(String::new())
}

/// Move/rename file
pub fn mv_command(src: &Path, dst: &Path, current_dir: &Path) -> Result<String> {
    let src_path = if src.is_absolute() {
        src.to_path_buf()
    } else {
        current_dir.join(src)
    };

    let dst_path = if dst.is_absolute() {
        dst.to_path_buf()
    } else {
        current_dir.join(dst)
    };

    if !src_path.exists() {
        miette::bail!("mv: {}: No such file or directory", src.display());
    }

    fs::rename(&src_path, &dst_path).into_diagnostic()?;

    Ok(String::new())
}

/// Copy file
pub fn cp_command(src: &Path, dst: &Path, current_dir: &Path, recursive: bool) -> Result<String> {
    let src_path = if src.is_absolute() {
        src.to_path_buf()
    } else {
        current_dir.join(src)
    };

    let dst_path = if dst.is_absolute() {
        dst.to_path_buf()
    } else {
        current_dir.join(dst)
    };

    if !src_path.exists() {
        miette::bail!("cp: {}: No such file or directory", src.display());
    }

    if src_path.is_dir() {
        if recursive {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            miette::bail!("cp: -r not specified: omitting directory '{}'", src.display());
        }
    } else {
        fs::copy(&src_path, &dst_path).into_diagnostic()?;
    }

    Ok(String::new())
}

/// Helper to recursively copy directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst).into_diagnostic()?;
    }

    for entry in fs::read_dir(src).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let ty = entry.file_type().into_diagnostic()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).into_diagnostic()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ls_nonexistent() {
        let path = Path::new("/nonexistent/path/that/does/not/exist");
        let current = Path::new("/");
        assert!(ls_command(path, current).is_err());
    }

    #[test]
    fn test_cd_resolve() {
        let current = std::env::current_dir().unwrap();
        let result = cd_command(Path::new("."), &current);
        assert!(result.is_ok());
        // cd to current dir should resolve to same location
        let resolved = result.unwrap();
        assert!(resolved.exists());
    }

    #[test]
    fn test_path_resolution() {
        let current = Path::new("/test");
        let path = Path::new("subdir");
        let resolved = current.join(path);
        assert_eq!(resolved, Path::new("/test/subdir"));
    }

    #[test]
    fn test_absolute_path() {
        let current = Path::new("/test");
        let path = Path::new("/absolute/path");
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            current.join(path)
        };
        assert_eq!(target, Path::new("/absolute/path"));
    }
}
