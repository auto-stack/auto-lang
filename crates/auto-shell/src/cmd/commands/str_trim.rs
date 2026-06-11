use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct StrTrimCommand;

impl Command for StrTrimCommand {
    fn name(&self) -> &str {
        "str-trim"
    }

    fn signature(&self) -> Signature {
        Signature::new("str-trim", "Trim whitespace from text")
            .flag_with_short("left", 'l', "Trim only leading whitespace")
            .flag_with_short("right", 'r', "Trim only trailing whitespace")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = extract_text(&input)?;
        let trim_left = args.has_flag("left");
        let trim_right = args.has_flag("right");

        let result = if trim_left && !trim_right {
            text.trim_start()
        } else if trim_right && !trim_left {
            text.trim_end()
        } else {
            text.trim()
        };

        Ok(PipelineData::from_text(result.to_string()))
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

fn extract_text(input: &PipelineData) -> Result<String> {
    match input {
        PipelineData::Text(s) => Ok(s.clone()),
        PipelineData::Value(Value::Str(s)) => Ok(s.to_string()),
        _ => miette::bail!("str-trim: input must be text"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_both() {
        assert_eq!("  hello  ".trim(), "hello");
    }

    #[test]
    fn test_trim_left() {
        assert_eq!("  hello  ".trim_start(), "hello  ");
    }

    #[test]
    fn test_trim_right() {
        assert_eq!("  hello  ".trim_end(), "  hello");
    }

    #[test]
    fn test_trim_no_whitespace() {
        assert_eq!("hello".trim(), "hello");
    }
}
