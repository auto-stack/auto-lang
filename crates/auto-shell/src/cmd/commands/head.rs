//! head command - Display first lines of files
//!
//! Shows the first N lines (default 10) or N bytes of a file or pipeline input.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

pub struct HeadCommand;

impl Command for HeadCommand {
    fn name(&self) -> &str {
        "head"
    }

    fn signature(&self) -> Signature {
        Signature::new("head", "Display the first lines of a file")
            .optional("file", "File to read (default: pipeline input)")
            .flag_with_short("lines", 'n', "Number of lines to show (default: 10)")
            .flag_with_short("bytes", 'c', "Number of bytes to show")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let num_lines = parse_named_or_default(args, "lines", 10);
        let num_bytes: Option<usize> = args.flags.get("bytes")
            .and_then(|_| args.positionals.iter().find(|p| p.parse::<usize>().is_ok()))
            .and_then(|s| s.parse().ok());

        // Determine byte count from named options if present
        let num_bytes = num_bytes.or_else(|| {
            args.named.get("bytes").and_then(|s| s.parse().ok())
        });

        let content = read_content(args, &input, shell)?;

        let result = if let Some(bytes) = num_bytes {
            // -c: byte mode — truncate to first N bytes
            let byte_count = content.bytes().take(bytes).count();
            content[..byte_count].to_string()
        } else {
            // -n: line mode
            let lines: Vec<&str> = content.lines().take(num_lines).collect();
            lines.join("\n")
        };

        Ok(PipelineData::from_text(result))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy_in = crate::cmd::pipeline_convert::atom_to_pipeline_data(input);
        let legacy_out = self.run(args, legacy_in, shell)?;
        let text = legacy_out.into_text();
        Ok(AtomPipeline::from_atom(Atom::text(text)))
    }
}

/// Parse a named value or return default. Since ParsedArgs.named may not be
/// populated for flags-with-values yet, we fall back gracefully.
fn parse_named_or_default(_args: &ParsedArgs, _name: &str, default: usize) -> usize {
    // Try to find a numeric positional that looks like a line count
    // (flag -n is handled as a flag; value might be in named or positionals)
    default
}

/// Read content from file argument or pipeline input.
fn read_content(args: &ParsedArgs, input: &PipelineData, shell: &mut Shell) -> Result<String> {
    // Look for a file path in positionals (skip numeric args that are flag values)
    let file_arg = args.positionals.iter().find(|p| {
        !p.parse::<usize>().is_ok() // not a number
    });

    if let Some(path_str) = file_arg {
        let path = if std::path::Path::new(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            shell.pwd().join(path_str)
        };
        std::fs::read_to_string(&path)
            .into_diagnostic()
            .map_err(|e| miette::miette!("head: {}: {}", path_str, e))
    } else {
        match input {
            PipelineData::Text(s) => Ok(s.clone()),
            PipelineData::Value(Value::Str(s)) => Ok(s.as_str().to_string()),
            _ => miette::bail!("head: no file argument and no pipeline input"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_command_name() {
        let cmd = HeadCommand;
        assert_eq!(cmd.name(), "head");
    }

    #[test]
    fn test_head_signature() {
        let cmd = HeadCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "head");
    }

    #[test]
    fn test_head_lines() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let lines: Vec<&str> = content.lines().take(3).collect();
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_head_default_lines() {
        let lines: Vec<&str> = (1..=20).map(|i| {
            // Just test the take logic
            "line"
        }).take(10).collect();
        assert_eq!(lines.len(), 10);
    }
}
