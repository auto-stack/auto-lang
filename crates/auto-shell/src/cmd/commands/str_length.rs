use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct StrLengthCommand;

impl Command for StrLengthCommand {
    fn name(&self) -> &str {
        "str-length"
    }

    fn signature(&self) -> Signature {
        Signature::new("str-length", "Get string length (Unicode-aware)")
    }

    fn run(
        &self,
        _args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = extract_text(&input)?;
        let len = text.chars().count() as i32;
        Ok(PipelineData::from_value(Value::Int(len)))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy_in = crate::cmd::pipeline_convert::atom_to_pipeline_data(input);
        let legacy_out = self.run(args, legacy_in, shell)?;
        let val = match legacy_out {
            PipelineData::Value(v) => v,
            PipelineData::Text(s) => Value::str(&s),
        };
        Ok(AtomPipeline::from_atom(Atom::new(val, AtomType::Nothing)))
    }
}

fn extract_text(input: &PipelineData) -> Result<String> {
    match input {
        PipelineData::Text(s) => Ok(s.clone()),
        PipelineData::Value(Value::Str(s)) => Ok(s.to_string()),
        _ => miette::bail!("str-length: input must be text"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_ascii() {
        assert_eq!("hello".chars().count(), 5);
    }

    #[test]
    fn test_length_unicode() {
        assert_eq!("你好世界".chars().count(), 4);
    }

    #[test]
    fn test_length_empty() {
        assert_eq!("".chars().count(), 0);
    }
}
