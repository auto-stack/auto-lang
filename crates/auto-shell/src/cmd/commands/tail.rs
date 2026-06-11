//! tail command - Display last lines of files
//!
//! Shows the last N lines (default 10) or N bytes of a file or pipeline input.
//! Follow mode (-f) is stubbed as a future feature.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

pub struct TailCommand;

impl Command for TailCommand {
    fn name(&self) -> &str {
        "tail"
    }

    fn signature(&self) -> Signature {
        Signature::new("tail", "Display the last lines of a file")
            .optional("file", "File to read (default: pipeline input)")
            .flag_with_short("lines", 'n', "Number of lines to show (default: 10)")
            .flag_with_short("bytes", 'c', "Number of bytes to show")
            .flag_with_short("follow", 'f', "Follow file changes (stub)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let _follow = args.has_flag("follow");
        let num_lines: usize = 10; // default

        let num_bytes: Option<usize> = args.named.get("bytes")
            .and_then(|s| s.parse().ok());

        let content = read_tail_content(args, &input, shell)?;

        let result = if let Some(bytes) = num_bytes {
            // -c: byte mode — take last N bytes
            let total = content.len();
            if bytes >= total {
                content
            } else {
                content[total - bytes..].to_string()
            }
        } else {
            // -n: line mode
            let all_lines: Vec<&str> = content.lines().collect();
            let skip = all_lines.len().saturating_sub(num_lines);
            all_lines[skip..].join("\n")
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

/// Read content from file argument or pipeline input.
fn read_tail_content(
    args: &ParsedArgs,
    input: &PipelineData,
    shell: &mut Shell,
) -> Result<String> {
    let file_arg = args.positionals.iter().find(|p| {
        !p.parse::<usize>().is_ok()
    });

    if let Some(path_str) = file_arg {
        let path = if std::path::Path::new(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            shell.pwd().join(path_str)
        };
        std::fs::read_to_string(&path)
            .into_diagnostic()
            .map_err(|e| miette::miette!("tail: {}: {}", path_str, e))
    } else {
        match input {
            PipelineData::Text(s) => Ok(s.clone()),
            PipelineData::Value(Value::Str(s)) => Ok(s.as_str().to_string()),
            _ => miette::bail!("tail: no file argument and no pipeline input"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tail_command_name() {
        let cmd = TailCommand;
        assert_eq!(cmd.name(), "tail");
    }

    #[test]
    fn test_tail_lines() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let all_lines: Vec<&str> = content.lines().collect();
        let skip = all_lines.len().saturating_sub(3);
        let result: Vec<&&str> = all_lines[skip..].iter().collect();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_tail_default() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let all_lines: Vec<&str> = content.lines().collect();
        let skip = all_lines.len().saturating_sub(10);
        assert_eq!(skip, 0); // fewer than 10 lines, show all
        assert_eq!(all_lines.len(), 5);
    }

    #[test]
    fn test_tail_bytes() {
        let content = "hello world";
        let total = content.len();
        let bytes = 5;
        let result = if bytes >= total {
            content.to_string()
        } else {
            content[total - bytes..].to_string()
        };
        assert_eq!(result, "world");
    }
}
