use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct RevCommand;

impl Command for RevCommand {
    fn name(&self) -> &str {
        "rev"
    }

    fn signature(&self) -> Signature {
        Signature::new("rev", "Reverse each line of text")
    }

    fn run(
        &self,
        _args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = get_text(input)?;
        let result = reverse_lines(&text);
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
        _ => miette::bail!("rev: input must be text"),
    }
}

/// Reverse each line of text
pub fn reverse_lines(text: &str) -> String {
    text.lines().map(|line| line.chars().rev().collect()).collect::<Vec<String>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_basic() {
        assert_eq!(reverse_lines("hello"), "olleh");
    }

    #[test]
    fn test_reverse_multiline() {
        assert_eq!(reverse_lines("abc\ndef"), "cba\nfed");
    }

    #[test]
    fn test_reverse_palindrome() {
        assert_eq!(reverse_lines("madam"), "madam");
    }

    #[test]
    fn test_reverse_empty() {
        assert_eq!(reverse_lines(""), "");
    }

    #[test]
    fn test_reverse_unicode() {
        assert_eq!(reverse_lines("rust"), "tsur");
    }

    #[test]
    fn test_reverse_spaces() {
        assert_eq!(reverse_lines("hello world"), "dlrow olleh");
    }
}
