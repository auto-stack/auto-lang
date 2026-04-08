//! rm command - Remove files and directories
//!
//! Provides cross-platform file and directory removal.

use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct RmCommand;

impl Command for RmCommand {
    fn name(&self) -> &str {
        "rm"
    }

    fn signature(&self) -> Signature {
        Signature::new("rm", "Remove files and directories")
            .required("path", "Path to remove")
            .flag_with_short("recursive", 'r', "Remove directories and their contents recursively")
            .flag_with_short("force", 'f', "Ignore nonexistent files and arguments")
            .flag("verbose", "Show what files are being removed")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        if args.positionals.is_empty() {
            miette::bail!("rm: missing operand");
        }

        let recursive = args.has_flag("recursive");
        let force = args.has_flag("force");
        let verbose = args.has_flag("verbose");

        let mut removed_count = 0;
        let mut errors = Vec::new();

        for arg in &args.positionals {
            let target_path = if Path::new(arg).is_absolute() {
                PathBuf::from(arg.as_str())
            } else {
                shell.pwd().join(arg.as_str())
            };

            let result = if target_path.is_dir() {
                if recursive {
                    remove_dir(&target_path, verbose)
                } else {
                    miette::bail!("rm: cannot remove '{}': Is a directory", arg);
                }
            } else {
                remove_file(&target_path, verbose)
            };

            match result {
                Ok(count) => removed_count += count,
                Err(e) => {
                    if !force {
                        errors.push(format!("{}: {}", arg, e));
                    }
                }
            }
        }

        // Return summary as Value
        let mut result_obj = auto_val::Obj::new();
        result_obj.set("files_removed", Value::I64(removed_count as i64));

        if !errors.is_empty() {
            let error_values: Vec<Value> = errors
                .iter()
                .map(|e| Value::str(e))
                .collect();
            result_obj.set("errors", Value::from(error_values));
            result_obj.set("success", Value::Bool(false));
        } else {
            result_obj.set("success", Value::Bool(true));
        }

        Ok(PipelineData::from_value(Value::Obj(result_obj)))
    }
}

/// Remove a single file
fn remove_file(path: &Path, verbose: bool) -> Result<usize> {
    if verbose {
        eprintln!("removing '{}'", path.display());
    }

    fs::remove_file(path).into_diagnostic()?;
    Ok(1)
}

/// Remove a directory recursively
fn remove_dir(path: &Path, verbose: bool) -> Result<usize> {
    let mut count = 0;

    // First, remove all contents
    for entry in fs::read_dir(path).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let entry_path = entry.path();

        count += if entry_path.is_dir() {
            remove_dir(&entry_path, verbose)?
        } else {
            if verbose {
                eprintln!("removing '{}'", entry_path.display());
            }
            fs::remove_file(&entry_path).into_diagnostic()?;
            1
        };
    }

    // Then remove the directory itself
    if verbose {
        eprintln!("removing directory '{}'", path.display());
    }
    fs::remove_dir(path).into_diagnostic()?;

    Ok(count + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_command_name() {
        let cmd = RmCommand;
        assert_eq!(cmd.name(), "rm");
    }

    #[test]
    fn test_rm_signature() {
        let cmd = RmCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "rm");
        assert_eq!(sig.description, "Remove files and directories");
        assert_eq!(sig.arguments.iter().filter(|a| a.required).count(), 1);
    }
}
