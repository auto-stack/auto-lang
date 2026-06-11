//! cat command - Concatenate and display files
//!
//! Reads files (or stdin via pipeline) and outputs their contents,
//! with optional line numbering and blank-line squeezing.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

pub struct CatCommand;

impl Command for CatCommand {
    fn name(&self) -> &str {
        "cat"
    }

    fn signature(&self) -> Signature {
        Signature::new("cat", "Concatenate files and print on the standard output")
            .optional("file", "Files to concatenate (default: read from pipeline)")
            .flag_with_short("number", 'n', "Number all output lines")
            .flag_with_short("number-nonblank", 'b', "Number non-blank output lines")
            .flag_with_short("squeeze-blank", 's', "Suppress repeated blank lines")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        let number = args.has_flag("number");
        let number_nonblank = args.has_flag("number-nonblank");
        let squeeze = args.has_flag("squeeze-blank");

        let content = if !args.positionals.is_empty() {
            // Read from files
            let mut combined = String::new();
            for arg in &args.positionals {
                let path = resolve_path(arg, shell);
                let text = std::fs::read_to_string(&path)
                    .into_diagnostic()
                    .map_err(|e| miette::miette!("cat: {}: {}", arg, e))?;
                combined.push_str(&text);
                if !text.ends_with('\n') && args.positionals.len() > 1 {
                    combined.push('\n');
                }
            }
            combined
        } else {
            // Read from pipeline input
            match input {
                PipelineData::Text(s) => s,
                PipelineData::Value(Value::Str(s)) => s.as_str().to_string(),
                PipelineData::Value(_) => {
                    miette::bail!("cat: no files given and pipeline input is not text");
                }
            }
        };

        let result = apply_cat_options(&content, number, number_nonblank, squeeze);
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

/// Resolve a path relative to the shell's CWD.
fn resolve_path(arg: &str, shell: &Shell) -> PathBuf {
    let path = std::path::Path::new(arg);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        shell.pwd().join(arg)
    }
}

/// Apply cat options: line numbering and blank-line squeezing.
fn apply_cat_options(content: &str, number: bool, number_nonblank: bool, squeeze: bool) -> String {
    let mut out = String::new();
    let mut line_num = 1;
    let mut prev_blank = false;

    for line in content.lines() {
        let is_blank = line.trim().is_empty();

        // Squeeze blank lines
        if squeeze && is_blank && prev_blank {
            continue;
        }
        prev_blank = is_blank;

        if number_nonblank {
            // -b: number only non-blank lines
            if is_blank {
                out.push('\n');
            } else {
                out.push_str(&format!("{:6}\t{}\n", line_num, line));
                line_num += 1;
            }
        } else if number {
            // -n: number all lines
            out.push_str(&format!("{:6}\t{}\n", line_num, line));
            line_num += 1;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    // Remove trailing newline if the original didn't have one
    if !content.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cat_number_all() {
        let content = "hello\n\nworld";
        let result = apply_cat_options(content, true, false, false);
        assert!(result.contains("1\thello"));
        assert!(result.contains("2\t"));
        assert!(result.contains("3\tworld"));
    }

    #[test]
    fn test_cat_number_nonblank() {
        let content = "hello\n\nworld";
        let result = apply_cat_options(content, false, true, false);
        assert!(result.contains("1\thello"));
        assert!(result.contains("2\tworld"));
    }

    #[test]
    fn test_cat_squeeze_blank() {
        let content = "a\n\n\n\nb";
        let result = apply_cat_options(content, false, false, true);
        // Should have at most one blank line between a and b
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() <= 3);
    }

    #[test]
    fn test_cat_plain() {
        let content = "hello\nworld";
        let result = apply_cat_options(content, false, false, false);
        assert_eq!(result, "hello\nworld");
    }

    #[test]
    fn test_cat_command_name() {
        let cmd = CatCommand;
        assert_eq!(cmd.name(), "cat");
    }
}
