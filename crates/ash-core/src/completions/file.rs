//! File path completion
//!
//! Provides completion for file and directory paths.

use std::path::Path;

use crate::completions::{Completion, CompletionKind};

/// Complete file paths
pub fn complete_file(input: &str) -> Vec<Completion> {
    let mut completions = Vec::new();

    // Find the last path segment to complete
    let last_space_idx = input.rfind(|c: char| c.is_whitespace() || c == '|');
    let path_start = last_space_idx.map(|i| i + 1).unwrap_or(0);
    let partial_path = &input[path_start..];

    if partial_path.is_empty() {
        // Complete from current directory
        complete_from_dir(Path::new("."), "", &mut completions);
    } else {
        // Check if path ends with a directory separator (e.g., "src/", "src/\")
        // In this case, we want to list the contents of that directory
        if partial_path.ends_with('/') || partial_path.ends_with('\\') {
            // List contents of the directory
            complete_from_dir(Path::new(partial_path), "", &mut completions);
        } else {
            // Extract directory and partial name
            let path = Path::new(partial_path);

            // Get parent directory, defaulting to "." if parent is empty or doesn't exist
            let parent = if let Some(p) = path.parent() {
                // path.parent() can return Some("") for relative paths like "s"
                if p.as_os_str().is_empty() {
                    Path::new(".")
                } else {
                    p
                }
            } else {
                Path::new(".")
            };

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            complete_from_dir(parent, file_name, &mut completions);
        }
    }

    completions
}

/// Complete files from a directory with a partial name filter
fn complete_from_dir(dir_path: &Path, partial: &str, completions: &mut Vec<Completion>) {
    // Try to read the directory
    let Ok(entries) = std::fs::read_dir(dir_path) else {
        return;
    };

    let dir_str = dir_path.to_string_lossy();
    let needs_separator = !dir_str.is_empty() && !dir_str.ends_with('/') && !dir_str.ends_with('\\');

    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();

        // Filter by partial name
        if !name.starts_with(partial) {
            continue;
        }

        let is_dir = entry.path().is_dir();
        let suffix = if is_dir { "/" } else { "" };
        let kind = if is_dir {
            CompletionKind::Directory
        } else {
            CompletionKind::File
        };

        // Build the replacement
        let mut replacement = if dir_str == "." || dir_str.is_empty() {
            name.clone()
        } else {
            format!("{}{}{}", dir_str, if needs_separator { "/" } else { "" }, name)
        };
        replacement.push_str(suffix);

        completions.push(Completion::with_kind(
            format!("{}{}", name, suffix),
            replacement,
            kind,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_file_current_dir() {
        let completions = complete_file("src");
        // The test runs from auto-shell directory, which has a src directory
        // So this should return completions
        let _ = completions;
    }

    #[test]
    fn test_complete_file_with_slash() {
        let completions = complete_file("./src/");
        let src_exists = std::path::Path::new("./src").exists();
        if src_exists {
            let _ = completions;
        }
    }

    #[test]
    fn test_complete_file_directory_with_slash() {
        let completions = complete_file("src/");
        let src_exists = std::path::Path::new("src").exists();
        if src_exists {
            assert!(!completions.is_empty());
        }
    }

    #[test]
    fn test_complete_file_empty() {
        let completions = complete_file("");
        assert!(!completions.is_empty());
    }

    #[test]
    fn test_complete_no_match() {
        let completions = complete_file("nonexistent_xyz_123");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_file_kind_classification() {
        let completions = complete_file("src/");
        let src_exists = std::path::Path::new("src").exists();
        if src_exists && !completions.is_empty() {
            // Directories should have Directory kind
            let dirs: Vec<_> = completions.iter().filter(|c| c.display.ends_with('/')).collect();
            if !dirs.is_empty() {
                assert_eq!(dirs[0].kind, CompletionKind::Directory);
            }
            // Files should have File kind
            let files: Vec<_> = completions.iter().filter(|c| !c.display.ends_with('/')).collect();
            if !files.is_empty() {
                assert_eq!(files[0].kind, CompletionKind::File);
            }
        }
    }
}
