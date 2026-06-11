use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Array};
use miette::Result;

pub struct StrSplitCommand;

impl Command for StrSplitCommand {
    fn name(&self) -> &str {
        "str-split"
    }

    fn signature(&self) -> Signature {
        Signature::new("str-split", "Split text into array")
            .optional("separator", "Separator to split on (default: whitespace)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = extract_text(&input)?;
        let result = if let Some(sep) = args.first() {
            split_by_sep(&text, sep)
        } else {
            split_whitespace(&text)
        };
        Ok(PipelineData::from_value(Value::Array(result)))
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
        Ok(AtomPipeline::from_atom(Atom::new(val, AtomType::Table)))
    }
}

fn extract_text(input: &PipelineData) -> Result<String> {
    match input {
        PipelineData::Text(s) => Ok(s.clone()),
        PipelineData::Value(Value::Str(s)) => Ok(s.to_string()),
        _ => miette::bail!("str-split: input must be text"),
    }
}

fn split_by_sep(text: &str, sep: &str) -> Array {
    let parts: Vec<Value> = text.split(sep).map(|s| Value::str(s)).collect();
    Array::from(parts)
}

fn split_whitespace(text: &str) -> Array {
    let parts: Vec<Value> = text.split_whitespace().map(|s| Value::str(s)).collect();
    Array::from(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_by_sep() {
        let arr = split_by_sep("a,b,c", ",");
        let items: Vec<String> = arr.iter().map(|v| v.as_str().to_string()).collect();
        assert_eq!(items, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_whitespace() {
        let arr = split_whitespace("  hello   world  ");
        let items: Vec<String> = arr.iter().map(|v| v.as_str().to_string()).collect();
        assert_eq!(items, vec!["hello", "world"]);
    }

    #[test]
    fn test_split_empty() {
        let arr = split_by_sep("", ",");
        assert_eq!(arr.len(), 1);
    }
}
