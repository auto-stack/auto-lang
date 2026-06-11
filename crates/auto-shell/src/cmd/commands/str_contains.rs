use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct StrContainsCommand;

impl Command for StrContainsCommand {
    fn name(&self) -> &str {
        "str-contains"
    }

    fn signature(&self) -> Signature {
        Signature::new("str-contains", "Check if text contains pattern")
            .required("pattern", "Pattern to search for")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let pattern = args.first().unwrap_or("");
        let text = extract_text(&input)?;

        let found = text.contains(pattern);
        Ok(PipelineData::from_value(Value::Bool(found)))
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
        _ => miette::bail!("str-contains: input must be text"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_found() {
        assert!("hello world".contains("world"));
    }

    #[test]
    fn test_contains_not_found() {
        assert!(!"hello world".contains("xyz"));
    }
}
