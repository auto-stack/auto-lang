use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Array};
use miette::Result;

pub struct EachCommand;

impl Command for EachCommand {
    fn name(&self) -> &str {
        "each"
    }

    fn signature(&self) -> Signature {
        Signature::new("each", "Extract a field from each record in array")
            .required("field", "Field name to extract from each item")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let field = args.first().unwrap_or("");
        if field.is_empty() {
            miette::bail!("each: field name is required");
        }

        match &input {
            PipelineData::Value(Value::Array(arr)) => {
                let result = extract_field(arr, field);
                Ok(PipelineData::from_value(Value::Array(result)))
            }
            _ => miette::bail!("each: input must be an array"),
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
        let val = match legacy_out {
            PipelineData::Value(v) => v,
            PipelineData::Text(s) => Value::str(&s),
        };
        Ok(AtomPipeline::from_atom(Atom::new(val, AtomType::Table)))
    }
}

fn extract_field(arr: &Array, field: &str) -> Array {
    let mut result = Array::new();
    for val in arr.iter() {
        if let Value::Obj(obj) = val {
            if let Some(v) = obj.get(field) {
                result.push(v);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::{Array, Obj};

    #[test]
    fn test_extract_field() {
        let mut obj1 = Obj::new();
        obj1.set("name", Value::str("alice"));
        obj1.set("age", Value::Int(30));
        let mut obj2 = Obj::new();
        obj2.set("name", Value::str("bob"));
        obj2.set("age", Value::Int(25));

        let arr = Array::from(vec![Value::Obj(obj1), Value::Obj(obj2)]);
        let result = extract_field(&arr, "name");

        let names: Vec<String> = result.iter().map(|v| v.as_str().to_string()).collect();
        assert_eq!(names, vec!["alice", "bob"]);
    }

    #[test]
    fn test_extract_missing_field() {
        let mut obj1 = Obj::new();
        obj1.set("name", Value::str("alice"));
        let mut obj2 = Obj::new();
        obj2.set("age", Value::Int(25));

        let arr = Array::from(vec![Value::Obj(obj1), Value::Obj(obj2)]);
        let result = extract_field(&arr, "name");

        // Only first object has "name"
        assert_eq!(result.len(), 1);
    }
}
