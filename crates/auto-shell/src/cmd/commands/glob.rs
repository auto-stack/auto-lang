//! glob command - Expand glob patterns to file paths
//!
//! Expands shell-style glob patterns (* and ?) against the filesystem.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline};
use auto_val::{Value, Obj, Array};
use miette::{IntoDiagnostic, Result};
use std::path::Path;

pub struct GlobCommand;

impl Command for GlobCommand {
    fn name(&self) -> &str {
        "glob"
    }

    fn signature(&self) -> Signature {
        Signature::new("glob", "Expand glob patterns to file paths")
            .required("pattern", "Glob pattern to expand (supports * and ?)")
            .flag_with_short("directory", 'd', "Match directories only")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let pattern = args.first()
            .ok_or_else(|| miette::miette!("glob: pattern argument required"))?;

        let dirs_only = args.has_flag("directory");
        let base_dir = shell.pwd();

        // Split pattern into directory prefix and filename glob
        let (search_dir, file_pattern) = split_pattern(pattern);

        let full_dir = if Path::new(&search_dir).is_absolute() {
            PathBuf::from(&search_dir)
        } else {
            base_dir.join(&search_dir)
        };

        if !full_dir.is_dir() {
            miette::bail!("glob: '{}' is not a directory", &search_dir);
        }

        let entries = std::fs::read_dir(&full_dir).into_diagnostic()?;
        let mut results = Vec::new();

        for entry in entries {
            let entry = entry.into_diagnostic()?;
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files
            if file_name.starts_with('.') {
                continue;
            }

            let path = entry.path();
            let is_dir = path.is_dir();

            // Apply directory-only filter
            if dirs_only && !is_dir {
                continue;
            }

            // Match against pattern
            if wildcard_match(&file_pattern, &file_name) {
                let rel = path.to_string_lossy().to_string();
                let mut obj = Obj::new();
                obj.set("path", Value::str(&rel));
                obj.set("name", Value::str(&file_name));
                obj.set("type", Value::str(if is_dir { "dir" } else { "file" }));
                results.push(Value::Obj(obj));
            }
        }

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

use std::path::PathBuf;

/// Split a pattern into directory prefix and filename glob.
/// E.g. "src/**/*.rs" -> ("src/**", "*.rs"), "*.rs" -> (".", "*.rs")
fn split_pattern(pattern: &str) -> (String, String) {
    let path = Path::new(pattern);
    match (path.parent(), path.file_name()) {
        (Some(dir), Some(file)) => {
            let dir_str = dir.to_string_lossy().to_string();
            let file_str = file.to_string_lossy().to_string();
            if dir_str.is_empty() {
                (".".to_string(), file_str)
            } else {
                (dir_str, file_str)
            }
        }
        _ => (".".to_string(), pattern.to_string()),
    }
}

/// Simple wildcard matching: supports * and ?
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    wm(&p, &t, 0, 0)
}

fn wm(p: &[char], t: &[char], pi: usize, ti: usize) -> bool {
    if pi == p.len() {
        return ti == t.len();
    }
    if p[pi] == '*' {
        for i in ti..=t.len() {
            if wm(p, t, pi + 1, i) {
                return true;
            }
        }
        false
    } else if ti < t.len() && (p[pi] == '?' || p[pi] == t[ti]) {
        wm(p, t, pi + 1, ti + 1)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_pattern_simple() {
        let (dir, pat) = split_pattern("*.rs");
        assert_eq!(dir, ".");
        assert_eq!(pat, "*.rs");
    }

    #[test]
    fn test_split_pattern_with_dir() {
        let (dir, pat) = split_pattern("src/*.rs");
        assert_eq!(dir, "src");
        assert_eq!(pat, "*.rs");
    }

    #[test]
    fn test_wildcard_match_star() {
        assert!(wildcard_match("*.rs", "main.rs"));
        assert!(wildcard_match("*.rs", "lib.rs"));
        assert!(!wildcard_match("*.rs", "main.txt"));
    }

    #[test]
    fn test_wildcard_match_question() {
        assert!(wildcard_match("?.rs", "a.rs"));
        assert!(!wildcard_match("?.rs", "ab.rs"));
    }

    #[test]
    fn test_glob_command_name() {
        let cmd = GlobCommand;
        assert_eq!(cmd.name(), "glob");
    }
}
