//! stat command - Display file status
//!
//! Shows metadata about a file: size, type, timestamps, permissions.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Obj};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

pub struct StatCommand;

impl Command for StatCommand {
    fn name(&self) -> &str {
        "stat"
    }

    fn signature(&self) -> Signature {
        Signature::new("stat", "Display file status")
            .required("file", "File to inspect")
            .flag_with_short("format", 'f', "Custom format string (stub)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let path_arg = args.first()
            .ok_or_else(|| miette::miette!("stat: missing file argument"))?;

        let path = if std::path::Path::new(path_arg).is_absolute() {
            PathBuf::from(path_arg)
        } else {
            shell.pwd().join(path_arg)
        };

        let metadata = std::fs::metadata(&path)
            .into_diagnostic()
            .map_err(|e| miette::miette!("stat: cannot stat '{}': {}", path_arg, e))?;

        let file_type = if path.is_dir() {
            "directory"
        } else if path.is_symlink() {
            "symlink"
        } else {
            "file"
        };

        let size = metadata.len();

        let modified = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                let secs = d.as_secs() as i64;
                chrono::DateTime::from_timestamp(secs, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| secs.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let created = metadata.created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                let secs = d.as_secs() as i64;
                chrono::DateTime::from_timestamp(secs, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| secs.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        #[cfg(unix)]
        let (permissions, readonly) = {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            (format!("{:o}", mode & 0o777), mode & 0o200 == 0)
        };

        #[cfg(windows)]
        let (permissions, readonly) = {
            let ro = metadata.permissions().readonly();
            (if ro { "read-only".to_string() } else { "read-write".to_string() }, ro)
        };

        let mut obj = Obj::new();
        obj.set("name", Value::str(path_arg));
        obj.set("size", Value::I64(size as i64));
        obj.set("type", Value::str(file_type));
        obj.set("modified", Value::str(&modified));
        obj.set("created", Value::str(&created));
        obj.set("permissions", Value::str(&permissions));
        obj.set("readonly", Value::Bool(readonly));

        Ok(PipelineData::from_value(Value::Obj(obj)))
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
        Ok(AtomPipeline::from_atom(Atom::new(value, AtomType::Record)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_command_name() {
        let cmd = StatCommand;
        assert_eq!(cmd.name(), "stat");
    }

    #[test]
    fn test_stat_signature() {
        let cmd = StatCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "stat");
        assert!(sig.arguments.iter().any(|a| a.name == "file" && a.required));
    }
}
