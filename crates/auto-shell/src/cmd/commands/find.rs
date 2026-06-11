//! find command - Find files matching criteria
//!
//! Walks a directory tree and returns entries matching name patterns
//! or type filters. Uses simple * wildcard matching without external deps.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Obj, Array};
use miette::{IntoDiagnostic, Result};
use std::path::{Path, PathBuf};

pub struct FindCommand;

impl Command for FindCommand {
    fn name(&self) -> &str {
        "find"
    }

    fn signature(&self) -> Signature {
        Signature::new("find", "Find files matching criteria")
            .optional("path", "Root path to search (default: .)")
            .flag_with_short("name", 'n', "Match filename pattern (supports *)")
            .flag_with_short("type", 't', "Filter by type: f=file, d=dir")
            .flag("max-depth", "Maximum directory depth")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let root_arg = args.positionals.iter()
            .find(|p| !looks_like_flag_value(p))
            .map(|s| s.as_str())
            .unwrap_or(".");

        let root = if Path::new(root_arg).is_absolute() {
            PathBuf::from(root_arg)
        } else {
            shell.pwd().join(root_arg)
        };

        // Get name pattern from named args or second positional
        let name_pattern = args.named.get("name").cloned()
            .or_else(|| {
                args.positionals.iter()
                    .skip(1)
                    .find(|p| !looks_like_flag_value(p))
                    .map(|s| s.clone())
            });

        // Get type filter
        let type_filter = args.named.get("type").cloned()
            .or_else(|| {
                if args.has_flag("type") {
                    args.positionals.iter()
                        .find(|p| *p == "f" || *p == "d")
                        .map(|s| s.clone())
                } else {
                    None
                }
            });

        // Get max depth
        let max_depth: Option<usize> = args.named.get("max-depth")
            .and_then(|s| s.parse().ok());

        let mut results = Vec::new();
        find_recursive(&root, &root, name_pattern.as_deref(), type_filter.as_deref(), max_depth, &mut results, 0)?;

        Ok(PipelineData::from_value(Value::Array(Array::from(results))))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy_in = crate::cmd::pipeline_convert::atom_to_pipeline_data(input);
        let legacy_out = self.run(args, legacy_in, shell)?;
        let value = match legacy_out {
            PipelineData::Value(v) => v,
            PipelineData::Text(s) => Value::str(&s),
        };
        Ok(AtomPipeline::from_atom(Atom::file_list(value)))
    }
}

/// Heuristic: skip values that look like flag values already consumed.
fn looks_like_flag_value(p: &str) -> bool {
    // Keep simple — treat everything as a potential path or pattern
    let _ = p;
    false
}

/// Recursively walk directories collecting matches.
fn find_recursive(
    root: &Path,
    current: &Path,
    name_pattern: Option<&str>,
    type_filter: Option<&str>,
    max_depth: Option<usize>,
    results: &mut Vec<Value>,
    depth: usize,
) -> Result<()> {
    // Check depth limit
    if let Some(max) = max_depth {
        if depth > max {
            return Ok(());
        }
    }

    let entries = match std::fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return Ok(()), // Skip unreadable dirs
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden entries
        if file_name.starts_with('.') {
            continue;
        }

        let is_dir = path.is_dir();
        let is_file = path.is_file();

        // Apply type filter
        let type_match = match type_filter {
            Some("f") => is_file,
            Some("d") => is_dir,
            _ => true,
        };

        // Apply name pattern
        let name_match = match name_pattern {
            Some(pat) => wildcard_match(pat, &file_name),
            None => true,
        };

        if type_match && name_match {
            let rel = pathdiff(&root, &path);
            let mut obj = Obj::new();
            obj.set("path", Value::str(&rel));
            obj.set("type", Value::str(if is_dir { "dir" } else { "file" }));
            results.push(Value::Obj(obj));
        }

        // Recurse into directories
        if is_dir {
            find_recursive(root, &path, name_pattern, type_filter, max_depth, results, depth + 1)?;
        }
    }

    Ok(())
}

/// Compute a relative path from root to target.
fn pathdiff(root: &Path, target: &Path) -> String {
    match target.strip_prefix(root) {
        Ok(rel) => rel.to_string_lossy().to_string(),
        Err(_) => target.to_string_lossy().to_string(),
    }
}

/// Simple wildcard matching: supports `*` (any sequence) and `?` (single char).
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    wildcard_match_inner(&p, &t, 0, 0)
}

fn wildcard_match_inner(p: &[char], t: &[char], pi: usize, ti: usize) -> bool {
    if pi == p.len() {
        return ti == t.len();
    }
    if p[pi] == '*' {
        // Try matching zero or more characters
        for i in ti..=t.len() {
            if wildcard_match_inner(p, t, pi + 1, i) {
                return true;
            }
        }
        false
    } else if ti < t.len() && (p[pi] == '?' || p[pi] == t[ti]) {
        wildcard_match_inner(p, t, pi + 1, ti + 1)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_star() {
        assert!(wildcard_match("*.rs", "main.rs"));
        assert!(wildcard_match("*.rs", "lib.rs"));
        assert!(!wildcard_match("*.rs", "main.txt"));
    }

    #[test]
    fn test_wildcard_question() {
        assert!(wildcard_match("?.rs", "a.rs"));
        assert!(!wildcard_match("?.rs", "ab.rs"));
    }

    #[test]
    fn test_wildcard_exact() {
        assert!(wildcard_match("hello", "hello"));
        assert!(!wildcard_match("hello", "world"));
    }

    #[test]
    fn test_wildcard_star_star() {
        assert!(wildcard_match("*test*", "my_test_file.rs"));
        assert!(wildcard_match("*test*", "test"));
    }

    #[test]
    fn test_find_command_name() {
        let cmd = FindCommand;
        assert_eq!(cmd.name(), "find");
    }
}
