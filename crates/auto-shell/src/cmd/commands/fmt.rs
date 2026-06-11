use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

pub struct FmtCommand;

impl Command for FmtCommand {
    fn name(&self) -> &str {
        "fmt"
    }

    fn signature(&self) -> Signature {
        Signature::new("fmt", "Reformat paragraph text to a target width")
            .optional("file", "File to reformat (default: stdin)")
            .flag_with_short("width", 'w', "Target line width (default: 79)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = if let Some(path) = args.first() {
            if Path::new(path).exists() {
                std::fs::read_to_string(Path::new(path)).into_diagnostic()?
            } else {
                get_text(input)?
            }
        } else {
            get_text(input)?
        };

        let width: usize = if args.has_flag("width") {
            args.positionals.iter()
                .find_map(|s| s.parse::<usize>().ok())
                .unwrap_or(79)
        } else {
            79
        };

        let result = fmt_text(&text, width);
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
        _ => miette::bail!("fmt: input must be text"),
    }
}

/// Reformat text to target width, preserving paragraph breaks (blank lines)
pub fn fmt_text(text: &str, width: usize) -> String {
    if width == 0 {
        return text.to_string();
    }

    let paragraphs = split_paragraphs(text);
    let formatted: Vec<String> = paragraphs
        .iter()
        .map(|para| reflow_paragraph(para, width))
        .collect();

    formatted.join("\n\n")
}

/// Split text into paragraphs (separated by blank lines)
pub fn split_paragraphs(text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }

    if paragraphs.is_empty() && !text.trim().is_empty() {
        paragraphs.push(text.trim().to_string());
    }

    paragraphs
}

/// Reflow a single paragraph to target width
pub fn reflow_paragraph(text: &str, width: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in &words {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflow_basic() {
        let text = "This is a simple test of the fmt command";
        let result = reflow_paragraph(text, 20);
        for line in result.lines() {
            assert!(line.len() <= 20, "Line too long: '{}' (len={})", line, line.len());
        }
    }

    #[test]
    fn test_reflow_short_text() {
        let text = "short";
        assert_eq!(reflow_paragraph(text, 79), "short");
    }

    #[test]
    fn test_reflow_preserves_words() {
        let text = "hello world foo bar";
        let result = reflow_paragraph(text, 11);
        let words: Vec<&str> = result.split_whitespace().collect();
        assert_eq!(words, vec!["hello", "world", "foo", "bar"]);
    }

    #[test]
    fn test_split_paragraphs() {
        let text = "para one\n\npara two\n\npara three";
        let paras = split_paragraphs(text);
        assert_eq!(paras.len(), 3);
    }

    #[test]
    fn test_split_paragraphs_single() {
        let text = "just one paragraph";
        let paras = split_paragraphs(text);
        assert_eq!(paras.len(), 1);
    }

    #[test]
    fn test_fmt_text() {
        let text = "This is a paragraph that should be reformatted to a shorter width for testing purposes.";
        let result = fmt_text(text, 30);
        for line in result.lines() {
            assert!(line.len() <= 30);
        }
    }

    #[test]
    fn test_fmt_preserves_paragraphs() {
        let text = "first paragraph\n\nsecond paragraph";
        let result = fmt_text(text, 79);
        let parts: Vec<&str> = result.split("\n\n").collect();
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_fmt_empty() {
        assert_eq!(fmt_text("", 79), "");
    }
}
