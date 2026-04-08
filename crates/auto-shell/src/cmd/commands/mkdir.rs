//! mkdir command - Create directories
//!
//! Provides cross-platform directory creation.

use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct MkdirCommand;

impl Command for MkdirCommand {
    fn name(&self) -> &str {
        "mkdir"
    }

    fn signature(&self) -> Signature {
        Signature::new("mkdir", "Create directories")
            .required("path", "Directory path to create")
            .flag_with_short("parents", 'p', "Create parent directories as needed")
            .flag_with_short("verbose", 'v', "Print a message for each created directory")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        if args.positionals.is_empty() {
            miette::bail!("mkdir: missing operand");
        }

        let parents = args.has_flag("parents");
        let verbose = args.has_flag("verbose");

        let mut created_count = 0;
        let mut errors = Vec::new();

        for arg in &args.positionals {
            let target_path = if Path::new(arg).is_absolute() {
                PathBuf::from(arg.as_str())
            } else {
                shell.pwd().join(arg.as_str())
            };

            let result = if parents {
                create_dir_all(&target_path, verbose)
            } else {
                create_dir(&target_path, verbose)
            };

            match result {
                Ok(count) => created_count += count,
                Err(e) => {
                    errors.push(format!("{}: {}", arg, e));
                }
            }
        }

        // Return summary as Value
        let mut result_obj = auto_val::Obj::new();
        result_obj.set("directories_created", Value::I64(created_count as i64));

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

/// Create a single directory
fn create_dir(path: &Path, verbose: bool) -> Result<usize> {
    if path.exists() {
        miette::bail!("mkdir: cannot create directory '{}': File exists", path.display());
    }

    if verbose {
        eprintln!("mkdir: created directory '{}'", path.display());
    }

    fs::create_dir(path).into_diagnostic()?;
    Ok(1)
}

/// Create directory and all parent directories
fn create_dir_all(path: &Path, verbose: bool) -> Result<usize> {
    if path.exists() {
        if !path.is_dir() {
            miette::bail!("mkdir: cannot create directory '{}': File exists", path.display());
        }
        return Ok(0);
    }

    if verbose {
        eprintln!("mkdir: created directory '{}'", path.display());
    }

    fs::create_dir_all(path).into_diagnostic()?;

    // Count how many directories were actually created
    let mut count = 0;
    let mut current = path.to_path_buf();
    while current != PathBuf::from("/") && current != PathBuf::from("") {
        if current.exists() {
            break;
        }
        count += 1;
        current = current.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mkdir_command_name() {
        let cmd = MkdirCommand;
        assert_eq!(cmd.name(), "mkdir");
    }

    #[test]
    fn test_mkdir_signature() {
        let cmd = MkdirCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "mkdir");
        assert_eq!(sig.description, "Create directories");
        assert_eq!(sig.arguments.iter().filter(|a| a.required).count(), 1);
    }
}
