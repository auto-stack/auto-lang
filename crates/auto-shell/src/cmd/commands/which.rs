//! `which` command - Locate a command in PATH
//!
//! Searches the system PATH directories for an executable matching the
//! given command name. Returns the first match as a Path.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Obj};
use miette::Result;

pub struct WhichCommand;

impl Command for WhichCommand {
    fn name(&self) -> &str {
        "which"
    }

    fn signature(&self) -> Signature {
        Signature::new("which", "Locate a command in PATH")
            .required("command", "Command name to search for")
            .flag("all", "Show all matches, not just the first")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let name = args
            .first()
            .ok_or_else(|| miette::miette!("which: missing command name"))?;

        let matches = find_in_path(name);

        if matches.is_empty() {
            miette::bail!("which: '{}' not found in PATH", name);
        }

        if args.has_flag("all") {
            let values: Vec<Value> = matches.iter().map(|p| Value::str(p)).collect();
            Ok(PipelineData::from_value(Value::Array(
                auto_val::Array::from(values),
            )))
        } else {
            Ok(PipelineData::from_text(matches[0].clone()))
        }
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        _input: AtomPipeline,
        _shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let name = args
            .first()
            .ok_or_else(|| miette::miette!("which: missing command name"))?;

        let matches = find_in_path(name);

        if matches.is_empty() {
            miette::bail!("which: '{}' not found in PATH", name);
        }

        if args.has_flag("all") {
            let values: Vec<Value> = matches.iter().map(|p| Value::str(p)).collect();
            Ok(AtomPipeline::from_atom(Atom::new(
                Value::Array(auto_val::Array::from(values)),
                AtomType::Path,
            )))
        } else {
            Ok(AtomPipeline::from_atom(Atom::path(&matches[0])))
        }
    }
}

/// Search PATH directories for an executable with the given name.
/// On Windows, also tries appending common executable extensions.
fn find_in_path(name: &str) -> Vec<String> {
    let mut results = Vec::new();
    let path_var = std::env::var_os("PATH").unwrap_or_default();

    // On Windows, try common executable extensions
    let extensions: Vec<String> = if cfg!(windows) {
        let pathext = std::env::var_os("PATHEXT")
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_else(|| ".EXE;.CMD;.BAT;.COM".to_string());
        pathext.split(';').map(|s| s.to_string()).collect()
    } else {
        vec![String::new()]
    };

    for dir in std::env::split_paths(&path_var) {
        for ext in &extensions {
            let mut file_name = name.to_string();
            if !ext.is_empty() {
                file_name.push_str(ext);
            }
            let candidate = dir.join(&file_name);
            if candidate.is_file() {
                let path_str = candidate.to_string_lossy().to_string();
                if !results.contains(&path_str) {
                    results.push(path_str);
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_command_name() {
        let cmd = WhichCommand;
        assert_eq!(cmd.name(), "which");
    }

    #[test]
    fn test_which_signature() {
        let cmd = WhichCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "which");
    }

    #[test]
    fn test_find_in_path_known_command() {
        // "cmd" exists on Windows; "ls" or "sh" on Unix
        let name = if cfg!(windows) { "cmd" } else { "sh" };
        let results = find_in_path(name);
        assert!(!results.is_empty(), "expected to find '{}' in PATH", name);
    }

    #[test]
    fn test_find_in_path_nonexistent() {
        let results = find_in_path("this_command_definitely_does_not_exist_xyz");
        assert!(results.is_empty());
    }
}
