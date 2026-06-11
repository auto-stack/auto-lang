use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Array};
use miette::Result;

pub struct StrJoinCommand;

impl Command for StrJoinCommand {
    fn name(&self) -> &str {
        "str-join"
    }

    fn signature(&self) -> Signature {
        Signature::new("str-join", "Join array elements into string")
            .optional("separator", "Separator between elements (default: \"\")")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let sep = args.first().unwrap_or("");
        let arr = match &input {
            PipelineData::Value(Value::Array(arr)) => arr.clone(),
            _ => miette::bail!("str-join: input must be an array"),
        };

        let parts: Vec<String> = arr.iter().map(|v| v.as_str().to_string()).collect();
        let result = parts.join(sep);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_comma() {
        let arr = Array::from(vec![Value::str("a"), Value::str("b"), Value::str("c")]);
        let parts: Vec<String> = arr.iter().map(|v| v.as_str().to_string()).collect();
        assert_eq!(parts.join(","), "a,b,c");
    }

    #[test]
    fn test_join_empty_sep() {
        let arr = Array::from(vec![Value::str("a"), Value::str("b")]);
        let parts: Vec<String> = arr.iter().map(|v| v.as_str().to_string()).collect();
        assert_eq!(parts.join(""), "ab");
    }
}
