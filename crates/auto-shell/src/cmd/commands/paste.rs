use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

pub struct PasteCommand;

impl Command for PasteCommand {
    fn name(&self) -> &str {
        "paste"
    }

    fn signature(&self) -> Signature {
        Signature::new("paste", "Merge lines of files")
            .optional("files", "Files to merge (default: stdin)")
            .flag_with_short("delimiters", 'd', "Delimiter between columns (default: TAB)")
            .flag_with_short("serial", 's', "Process files one at a time (serial)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let delimiter = if args.has_flag("delimiters") {
            args.positionals.iter().find(|s| !Path::new(s).exists() || s.starts_with('-'))
                .map(|s| s.as_str())
                .unwrap_or("\t")
                .to_string()
        } else {
            "\t".to_string()
        };

        let serial = args.has_flag("serial");

        // Collect file paths from positional args that look like files
        let file_args: Vec<&str> = args.positionals.iter()
            .filter(|s| Path::new(s).exists())
            .map(|s| s.as_str())
            .collect();

        if file_args.is_empty() {
            // Merge from pipeline input
            let text = get_text(input)?;
            let result = paste_single(&text, &delimiter, serial);
            Ok(PipelineData::from_text(result))
        } else {
            // Read multiple files and merge side-by-side
            let file_contents: Vec<String> = file_args
                .iter()
                .map(|p| std::fs::read_to_string(Path::new(p)).into_diagnostic())
                .collect::<Result<Vec<String>>>()?;

            let result = if serial {
                paste_serial(&file_contents, &delimiter)
            } else {
                paste_parallel(&file_contents, &delimiter)
            };
            Ok(PipelineData::from_text(result))
        }
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
        Ok(AtomPipeline::from_atom(Atom::new(Value::str(&text), AtomType::Text)))
    }
}

/// Extract text from PipelineData
fn get_text(input: PipelineData) -> Result<String> {
    match input {
        PipelineData::Text(s) => Ok(s),
        PipelineData::Value(Value::Str(s)) => Ok(s.to_string()),
        PipelineData::Value(Value::Array(arr)) => {
            let lines: Vec<String> = arr.iter().map(|v| v.as_str().to_string()).collect();
            Ok(lines.join("\n"))
        }
        _ => miette::bail!("paste: input must be text"),
    }
}

/// Paste single input: join all lines with delimiter (serial mode) or pass through
pub fn paste_single(text: &str, delimiter: &str, serial: bool) -> String {
    if serial {
        text.lines().collect::<Vec<&str>>().join(delimiter)
    } else {
        text.to_string()
    }
}

/// Merge multiple file contents side-by-side (parallel)
pub fn paste_parallel(contents: &[String], delimiter: &str) -> String {
    let all_lines: Vec<Vec<&str>> = contents.iter().map(|c| c.lines().collect()).collect();
    let max_lines = all_lines.iter().map(|l| l.len()).max().unwrap_or(0);

    let mut result = Vec::new();
    for i in 0..max_lines {
        let row: Vec<&str> = all_lines
            .iter()
            .map(|lines| lines.get(i).copied().unwrap_or(""))
            .collect();
        result.push(row.join(delimiter));
    }
    result.join("\n")
}

/// Merge files serially (one file per line, lines joined by delimiter)
pub fn paste_serial(contents: &[String], delimiter: &str) -> String {
    contents
        .iter()
        .map(|c| c.lines().collect::<Vec<&str>>().join(delimiter))
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paste_single_passthrough() {
        let text = "a\nb\nc";
        assert_eq!(paste_single(text, "\t", false), "a\nb\nc");
    }

    #[test]
    fn test_paste_single_serial() {
        let text = "a\nb\nc";
        assert_eq!(paste_single(text, ",", true), "a,b,c");
    }

    #[test]
    fn test_paste_parallel() {
        let f1 = "a1\na2\na3".to_string();
        let f2 = "b1\nb2".to_string();
        let result = paste_parallel(&[f1, f2], "\t");
        assert_eq!(result, "a1\tb1\na2\tb2\na3\t");
    }

    #[test]
    fn test_paste_parallel_custom_delim() {
        let f1 = "a\nb".to_string();
        let f2 = "c\nd".to_string();
        let result = paste_parallel(&[f1, f2], ",");
        assert_eq!(result, "a,c\nb,d");
    }

    #[test]
    fn test_paste_serial() {
        let f1 = "a\nb".to_string();
        let f2 = "c\nd".to_string();
        let result = paste_serial(&[f1, f2], "|");
        assert_eq!(result, "a|b\nc|d");
    }

    #[test]
    fn test_paste_parallel_single_file() {
        let f1 = "x\ny".to_string();
        let result = paste_parallel(&[f1], "\t");
        assert_eq!(result, "x\ny");
    }
}
