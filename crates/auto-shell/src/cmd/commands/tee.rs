//! tee command - Read from stdin and write to stdout and file
//!
//! Takes pipeline input and writes it to both the output and a specified file,
//! optionally appending instead of overwriting.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

pub struct TeeCommand;

impl Command for TeeCommand {
    fn name(&self) -> &str {
        "tee"
    }

    fn signature(&self) -> Signature {
        Signature::new("tee", "Read from stdin and write to stdout and file")
            .required("file", "File to write to")
            .flag_with_short("append", 'a', "Append to the file instead of overwriting")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let file_arg = args.first()
            .ok_or_else(|| miette::miette!("tee: missing file argument"))?;

        let append = args.has_flag("append");

        let path = if std::path::Path::new(file_arg).is_absolute() {
            PathBuf::from(file_arg)
        } else {
            shell.pwd().join(file_arg)
        };

        // Extract text content from pipeline input
        let content = match &input {
            PipelineData::Text(s) => s.clone(),
            PipelineData::Value(Value::Str(s)) => s.as_str().to_string(),
            PipelineData::Value(v) => {
                // Format non-string values as text
                format_value_text(v)
            }
        };

        // Write to file
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).into_diagnostic()?;
            }
        }

        if append {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .into_diagnostic()?;
            f.write_all(content.as_bytes()).into_diagnostic()?;
        } else {
            std::fs::write(&path, &content).into_diagnostic()?;
        }

        // Pass through the content as output
        Ok(input)
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy_in = crate::cmd::pipeline_convert::atom_to_pipeline_data(input.clone());
        let _legacy_out = self.run(args, legacy_in, shell)?;
        // Return the original atom pipeline (pass-through)
        Ok(input)
    }
}

/// Format a Value as text for writing to file.
fn format_value_text(v: &Value) -> String {
    match v {
        Value::Str(s) => s.as_str().to_string(),
        Value::String(s) => s.as_str().to_string(),
        Value::Int(i) => i.to_string(),
        Value::I64(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => {
            arr.iter()
                .map(|item| format_value_text(item))
                .collect::<Vec<_>>()
                .join("\n")
        }
        Value::Obj(obj) => {
            let pairs: Vec<String> = obj.iter()
                .map(|(k, v)| format!("{}: {}", k, format_value_text(v)))
                .collect();
            pairs.join(", ")
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_command_name() {
        let cmd = TeeCommand;
        assert_eq!(cmd.name(), "tee");
    }

    #[test]
    fn test_tee_signature() {
        let cmd = TeeCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "tee");
        assert!(sig.arguments.iter().any(|a| a.name == "file" && a.required));
    }

    #[test]
    fn test_format_value_text() {
        assert_eq!(format_value_text(&Value::Int(42)), "42");
        assert_eq!(format_value_text(&Value::Bool(true)), "true");
        assert_eq!(format_value_text(&Value::str("hello")), "hello");
    }
}
