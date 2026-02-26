//! File system commands
//!
//! Implements core file system operations: ls, cd, mkdir, rm, mv, cp

use auto_val::{Value, Obj, Array};
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::data::{Table, Column, Align, FileEntry};

/// List directory contents with table formatting
pub fn ls_command(
    path: &Path,
    current_dir: &Path,
    all: bool,
    long: bool,
    human: bool,
    time_sort: bool,
    reverse: bool,
    recursive: bool,
) -> Result<String> {
    let target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };

    if !target.exists() {
        miette::bail!("ls: {}: No such file or directory", target.display());
    }

    // Handle recursive listing
    if recursive {
        return list_recursive(&target, all, long, human, time_sort, reverse);
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

        // Skip hidden files unless -a flag is set
        if !all && name.starts_with('.') {
            continue;
        }

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

    // Sort files
    files.sort_by(|a, b| {
        let cmp = if time_sort {
            // Sort by modification time (newest first)
            b.modified.as_ref().unwrap_or(&String::new())
                .cmp(a.modified.as_ref().unwrap_or(&String::new()))
        } else {
            // Sort alphabetically
            a.name.cmp(&b.name)
        };

        // Directories first
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            cmp
        }
    });

    if reverse {
        files.reverse();
    }

    // Create table with conditional columns based on -l flag
    let mut table = if long {
        Table::new()
            .add_column(Column::new("Permissions").align(Align::Left))
            .add_column(Column::new("Owner").align(Align::Left))
            .add_column(Column::new("Size").align(Align::Right))
            .add_column(Column::new("Modified").align(Align::Left))
            .add_column(Column::new("Name").align(Align::Left))
    } else {
        Table::new()
            .add_column(Column::new("Name").align(Align::Left))
            .add_column(Column::new("Size").align(Align::Right))
            .add_column(Column::new("Modified").align(Align::Left))
    };

    // Add rows
    for file in &files {
        if long {
            // Long format: permissions, owner, size, modified, name
            // For now, use placeholder values for permissions/owner
            // TODO: Implement platform-specific permission formatting
            let perms = if file.is_dir { "drwxr-xr-x" } else { "-rw-r--r--" }.to_string();
            let owner = "-".to_string();
            let size_str = if human {
                file.format_size()
            } else {
                file.size.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
            };
            let name_with_indicator = if file.is_dir {
                format!("{}/", file.name)
            } else {
                file.name.clone()
            };

            table = table.add_row(vec![
                perms,
                owner,
                size_str,
                file.modified.clone().unwrap_or_else(|| "-".to_string()),
                name_with_indicator,
            ]);
        } else {
            // Default format: name, size, modified
            let name_with_indicator = if file.is_dir {
                format!("{}/", file.name)
            } else {
                file.name.clone()
            };

            let size_str = if human {
                file.format_size()
            } else {
                file.format_size()
            };

            table = table.add_row(vec![
                name_with_indicator,
                size_str,
                file.modified.clone().unwrap_or_else(|| "-".to_string()),
            ]);
        }
    }

    // Calculate widths and render
    table.calculate_widths();
    Ok(table.render())
}

/// List directory contents as structured Value (array of file objects)
///
/// This is the structured data version of ls_command for use in pipelines.
/// Returns an Array of Obj values, where each Obj represents a file entry.
pub fn ls_command_value(
    path: &Path,
    current_dir: &Path,
    all: bool,
    long: bool,
    time_sort: bool,
    reverse: bool,
    recursive: bool,
) -> Result<Value> {
    let target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };

    if !target.exists() {
        miette::bail!("ls: {}: No such file or directory", target.display());
    }

    // Handle recursive listing
    if recursive {
        return list_recursive_value(&target, current_dir, all, long, time_sort, reverse);
    }

    // If it's a file, return single-element array with file info
    if target.is_file() {
        let metadata = fs::metadata(&target).into_diagnostic()?;
        let name = target.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let obj = build_file_entry_obj(&name, false, &metadata, long);
        let mut arr = Array::new();
        arr.push(Value::Obj(obj));
        return Ok(Value::Array(arr));
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

        // Skip hidden files unless -a flag is set
        if !all && name.starts_with('.') {
            continue;
        }

        let is_dir = entry.path().is_dir();

        // Build file entry object
        let obj = build_file_entry_obj(&name, is_dir, &metadata, long);
        files.push((name.clone(), is_dir, metadata.modified().ok(), obj));
    }

    // Sort files
    files.sort_by(|a, b| {
        let cmp = if time_sort {
            // Sort by modification time (newest first)
            match (&a.2, &b.2) {
                (Some(a_time), Some(b_time)) => b_time.cmp(a_time),
                _ => a.0.cmp(&b.0),
            }
        } else {
            // Sort alphabetically
            a.0.cmp(&b.0)
        };

        // Directories first
        if a.1 != b.1 {
            b.1.cmp(&a.1)
        } else {
            cmp
        }
    });

    if reverse {
        files.reverse();
    }

    // Build array
    let mut arr = Array::new();
    for (_, _, _, obj) in files {
        arr.push(Value::Obj(obj));
    }

    Ok(Value::Array(arr))
}

/// Build a file entry object from metadata
fn build_file_entry_obj(name: &str, is_dir: bool, metadata: &fs::Metadata, long: bool) -> Obj {
    let mut obj = Obj::new();
    obj.set("name", Value::str(name));
    obj.set("type", Value::str(if is_dir { "dir" } else { "file" }));

    if !is_dir {
        obj.set("size", Value::Int(metadata.len() as i32));
    }

    if let Ok(modified) = metadata.modified() {
        if let Some(modified_str) = format_modified_time(&modified) {
            obj.set("modified", Value::str(modified_str));
        }
    }

    if long {
        // Add permissions and owner for long format
        let perms = format_permissions(metadata, is_dir);
        obj.set("permissions", Value::str(perms));

        let owner = get_owner(metadata);
        obj.set("owner", Value::str(owner));
    }

    obj
}

/// Recursive directory listing helper (structured data version)
fn list_recursive_value(
    path: &Path,
    current_dir: &Path,
    all: bool,
    long: bool,
    time_sort: bool,
    reverse: bool,
) -> Result<Value> {
    let mut all_entries = Array::new();

    // List current directory
    if let Value::Array(entries) = ls_command_value(path, current_dir, all, long, time_sort, reverse, false)? {
        for entry in entries.iter() {
            all_entries.push(entry.clone());
        }
    }

    // Find subdirectories and recurse
    let entries = fs::read_dir(path).into_diagnostic()?;
    for entry in entries {
        let entry = entry.into_diagnostic()?;
        if entry.path().is_dir() {
            let name = entry.file_name().into_string().unwrap_or_default();
            // Skip hidden directories unless -a flag is set
            if !all && name.starts_with('.') {
                continue;
            }
            // Skip . and ..
            if name == "." || name == ".." {
                continue;
            }

            // Recurse
            if let Value::Array(sub_entries) = list_recursive_value(&entry.path(), current_dir, all, long, time_sort, reverse)? {
                for sub_entry in sub_entries.iter() {
                    all_entries.push(sub_entry.clone());
                }
            }
        }
    }

    Ok(Value::Array(all_entries))
}

/// Format system time as "YYYY-MM-DD HH:MM" string
fn format_modified_time(modified: &std::time::SystemTime) -> Option<String> {
    use std::time::UNIX_EPOCH;
    let secs = modified.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)?;
    Some(datetime.format("%Y-%m-%d %H:%M").to_string())
}

/// Format file permissions (Unix-style rwxrwxrwx)
#[cfg(unix)]
fn format_permissions(metadata: &fs::Metadata, is_dir: bool) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    let file_type = if is_dir {
        'd'
    } else {
        '-'
    };

    let user = format_mode_bits(mode & 0o700);
    let group = format_mode_bits(mode & 0o070);
    let other = format_mode_bits(mode & 0o007);

    format!("{}{}{}{}", file_type, user, group, other)
}

/// Format permission bits (e.g., "rwx")
#[cfg(unix)]
fn format_mode_bits(bits: u32) -> String {
    format!(
        "{}{}{}",
        if bits & 0o400 != 0 { 'r' } else { '-' },
        if bits & 0o200 != 0 { 'w' } else { '-' },
        if bits & 0o100 != 0 { 'x' } else { '-' }
    )
}

/// Get owner name from metadata
#[cfg(unix)]
fn get_owner(metadata: &fs::Metadata) -> String {
    use std::os::unix::fs::MetadataExt;
    metadata.uid().to_string()
}

/// Format file permissions (Windows - simplified)
#[cfg(windows)]
fn format_permissions(_metadata: &fs::Metadata, is_dir: bool) -> String {
    if is_dir {
        "drwxr-xr-x".to_string()
    } else {
        "-rw-r--r--".to_string()
    }
}

/// Get owner name (Windows - no owner info)
#[cfg(windows)]
fn get_owner(_metadata: &fs::Metadata) -> String {
    "-".to_string()
}

/// Recursive directory listing helper
fn list_recursive(
    path: &Path,
    all: bool,
    long: bool,
    human: bool,
    time_sort: bool,
    reverse: bool,
) -> Result<String> {
    let mut output = String::new();

    // If path is relative, make it relative to current dir for display
    let display_path = if path.is_absolute() {
        path.display().to_string()
    } else {
        format!("./{}", path.display())
    };

    output.push_str(&format!("{}:\n", display_path));

    // List current directory (non-recursive call)
    // For recursive, we pass false to avoid infinite recursion
    let current_listing = ls_command(path, path, all, long, human, time_sort, reverse, false)?;
    output.push_str(&current_listing);

    // Find subdirectories and recurse
    let entries = fs::read_dir(path).into_diagnostic()?;
    let mut subdirs: Vec<PathBuf> = Vec::new();

    for entry in entries {
        let entry = entry.into_diagnostic()?;
        if entry.path().is_dir() {
            let name = entry.file_name().into_string().unwrap_or_default();
            // Skip hidden directories unless -a flag is set
            if !all && name.starts_with('.') {
                continue;
            }
            // Skip . and ..
            if name == "." || name == ".." {
                continue;
            }
            subdirs.push(entry.path());
        }
    }

    // Sort subdirs
    subdirs.sort();

    // Recurse into subdirectories
    for subdir in subdirs {
        output.push_str("\n");
        output.push_str(&list_recursive(&subdir, all, long, human, time_sort, reverse)?);
    }

    Ok(output)
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
        assert!(ls_command(path, current, false, false, false, false, false, false).is_err());
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
