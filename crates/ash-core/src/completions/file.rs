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

/// Complete files from a directory with a partial name filter.
///
/// Matching rules (case-insensitive):
/// - Pure prefix match has highest priority (e.g. `au` → `autostack/`)
/// - Prefix-substring match: split input at the longest common prefix with the
///   candidate, then check if the remaining input chars appear in order in the
///   rest of the candidate name.
///   Example: input `al` vs candidate `auto-lang` → prefix `a`, remaining `l`
///   appears in `uto-lang` → match.
///   Example: input `au` vs candidate `a2r-check.txt` → prefix `a`, remaining
///   `u` does NOT appear in `2r-check.txt` → no match.
fn complete_from_dir(dir_path: &Path, partial: &str, completions: &mut Vec<Completion>) {
    // Try to read the directory
    let Ok(entries) = std::fs::read_dir(dir_path) else {
        return;
    };

    let partial_lower = partial.to_lowercase();
    let dir_str = dir_path.to_string_lossy();
    let needs_separator = !dir_str.is_empty() && !dir_str.ends_with('/') && !dir_str.ends_with('\\');

    let entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    let mut prefix_matches = Vec::new();
    let mut fuzzy_matches = Vec::new();

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let name_lower = name.to_lowercase();

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

        let completion = Completion::with_kind(
            format!("{}{}", name, suffix),
            replacement,
            kind,
        );

        if name_lower.starts_with(&partial_lower) {
            // Exact prefix match — highest priority
            prefix_matches.push(completion);
        } else if is_prefix_subseq_match(&partial_lower, &name_lower) {
            // Prefix-subsequence match — lower priority
            fuzzy_matches.push(completion.as_fuzzy());
        }
    }

    // Prefix matches first, fuzzy matches as fallback
    completions.extend(prefix_matches);
    completions.extend(fuzzy_matches);
}

/// Check if `input` matches `candidate` as a "prefix-subsequence" pattern.
///
/// The algorithm finds the longest common prefix between input and candidate,
/// then checks if the remaining input characters appear in order (but not
/// necessarily contiguously) in the rest of the candidate.
///
/// Examples:
/// - input `al`, candidate `auto-lang` → prefix `a`, remaining `l` found in `uto-lang` ✓
/// - input `au`, candidate `a2r-check` → prefix `a`, remaining `u` NOT in `2r-check` ✗
/// - input `al`, candidate `auto-man` → prefix `a`, remaining `l` NOT in `uto-man` ✗
fn is_prefix_subseq_match(input: &str, candidate: &str) -> bool {
    if input.is_empty() || candidate.is_empty() {
        return false;
    }

    // Find longest common prefix length
    let prefix_len = input
        .chars()
        .zip(candidate.chars())
        .take_while(|(a, b)| a == b)
        .count();

    // If no common prefix at all, no match
    if prefix_len == 0 {
        return false;
    }

    // Remaining input chars after the common prefix
    let remaining_input: Vec<char> = input.chars().skip(prefix_len).collect();
    if remaining_input.is_empty() {
        // Input is exactly a prefix of candidate — this would be caught by starts_with
        return false;
    }

    // Remaining candidate after the common prefix
    let remaining_candidate: Vec<char> = candidate.chars().skip(prefix_len).collect();

    // Check if remaining_input chars appear in order in remaining_candidate (subsequence)
    let mut input_idx = 0;
    for &c in &remaining_candidate {
        if input_idx < remaining_input.len() && c == remaining_input[input_idx] {
            input_idx += 1;
        }
    }

    input_idx == remaining_input.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_file_current_dir() {
        let completions = complete_file("src");
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
            let dirs: Vec<_> = completions.iter().filter(|c| c.display.ends_with('/')).collect();
            if !dirs.is_empty() {
                assert_eq!(dirs[0].kind, CompletionKind::Directory);
            }
            let files: Vec<_> = completions.iter().filter(|c| !c.display.ends_with('/')).collect();
            if !files.is_empty() {
                assert_eq!(files[0].kind, CompletionKind::File);
            }
        }
    }

    // --- Prefix-subsequence matching tests ---

    #[test]
    fn test_prefix_subseq_basic() {
        // 'al' matches 'auto-lang': prefix 'a', remaining 'l' found in 'uto-lang'
        assert!(is_prefix_subseq_match("al", "auto-lang"));
    }

    #[test]
    fn test_prefix_subseq_no_match_wrong_remaining() {
        // 'al' does NOT match 'auto-man': prefix 'a', remaining 'l' NOT in 'uto-man'
        assert!(!is_prefix_subseq_match("al", "auto-man"));
    }

    #[test]
    fn test_prefix_subseq_no_match_no_common_prefix() {
        // 'au' vs 'a2r-check': prefix 'a', remaining 'u' NOT in '2r-check'
        assert!(!is_prefix_subseq_match("au", "a2r-check"));
    }

    #[test]
    fn test_prefix_subseq_multi_char_remaining() {
        // 'alg' matches 'auto-lang': prefix 'a', remaining 'lg' found as subseq in 'uto-lang'
        assert!(is_prefix_subseq_match("alg", "auto-lang"));
    }

    #[test]
    fn test_prefix_subseq_exact_prefix_is_not_fuzzy() {
        // Input equals a prefix of candidate — this is handled by starts_with, not fuzzy
        assert!(!is_prefix_subseq_match("au", "autostack"));
    }

    #[test]
    fn test_prefix_subseq_empty_input() {
        assert!(!is_prefix_subseq_match("", "anything"));
    }

    #[test]
    fn test_prefix_subseq_empty_candidate() {
        assert!(!is_prefix_subseq_match("abc", ""));
    }

    #[test]
    fn test_prefix_subseq_full_match_but_no_extra() {
        // Input exactly equals candidate — not a fuzzy match
        assert!(!is_prefix_subseq_match("auto", "auto"));
    }

    #[test]
    fn test_prefix_subseq_case_insensitive() {
        // Caller lowercases both args before calling, so this function
        // itself is case-sensitive. Test the contract correctly:
        assert!(is_prefix_subseq_match("al", "auto-lang"));
        // Mixed case would NOT match because function is case-sensitive;
        // the caller is responsible for lowercasing.
        assert!(!is_prefix_subseq_match("al", "Auto-Lang"));
    }

    #[test]
    fn test_complete_file_prefix_priority() {
        // Prefix matches should appear before fuzzy matches
        let completions = complete_file("Cargo");
        if completions.len() >= 2 {
            let first = &completions[0];
            assert!(
                first.display.to_lowercase().starts_with("cargo"),
                "Prefix matches should come first, got: {}",
                first.display
            );
        }
    }
}
