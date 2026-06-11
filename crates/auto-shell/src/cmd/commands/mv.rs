//! mv command - Move or rename files and directories
//!
//! Provides cross-platform file moving/renaming.

use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::fs;

pub struct MvCommand;

impl Command for MvCommand {
    fn name(&self) -> &str {
        "mv"
    }

    fn signature(&self) -> Signature {
        Signature::new("mv", "Move or rename files and directories")
            .required("source", "Source file or directory")
            .required("dest", "Destination path")
            .flag_with_short("force", 'f', "Force overwrite without prompting")
            .flag("verbose", "Show what files are being moved")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        if args.positionals.len() < 2 {
            miette::bail!("mv: missing source or destination argument");
        }

        let source = args.positionals.get(0).map(|s| s.as_str()).unwrap_or(".");
        let dest = args.positionals.get(1).map(|s| s.as_str()).unwrap_or(".");

        let force = args.has_flag("force");
        let verbose = args.has_flag("verbose");

        let source_path = shell.pwd().join(source);
        let dest_path = shell.pwd().join(dest);

        if !source_path.exists() {
            miette::bail!("mv: cannot stat '{}': No such file or directory", source);
        }

        // Handle destination
        let final_dest = if dest_path.exists() && dest_path.is_dir() {
            // If dest is a directory, move source into it
            dest_path.join(
                source_path
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("")),
            )
        } else {
            dest_path.clone()
        };

        // Check if destination exists
        if final_dest.exists() && !force {
            miette::bail!(
                "mv: cannot move '{}' to '{}': File exists (use -f to force)",
                source,
                dest
            );
        }

        if verbose {
            eprintln!("'{}' -> '{}'", source_path.display(), final_dest.display());
        }

        // Perform the move
        fs::rename(&source_path, &final_dest).into_diagnostic()?;

        // Return summary as Value
        let mut result = auto_val::Obj::new();
        result.set("source", Value::str(source));
        result.set("destination", Value::str(dest));
        result.set("success", Value::Bool(true));

        Ok(PipelineData::from_value(Value::Obj(result)))
    }

    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy = self.run(args, PipelineData::empty(), shell)?;
        Ok(crate::cmd::pipeline_convert::pipeline_data_to_atom(legacy))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mv_command_name() {
        let cmd = MvCommand;
        assert_eq!(cmd.name(), "mv");
    }

    #[test]
    fn test_mv_signature() {
        let cmd = MvCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "mv");
        assert_eq!(sig.description, "Move or rename files and directories");
        assert_eq!(sig.arguments.iter().filter(|a| a.required).count(), 2);
    }
}
