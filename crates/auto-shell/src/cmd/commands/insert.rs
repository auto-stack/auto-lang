use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Array, Obj};
use miette::Result;

pub struct InsertCommand;

impl Command for InsertCommand {
    fn name(&self) -> &str {
        "insert"
    }

    fn signature(&self) -> Signature {
        Signature::new("insert", "Insert a new field in each record (only if not present)")
            .required("field", "Field name to insert")
            .required("value", "Value to set for the field")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let field = args.first().unwrap_or("");
        let raw_value = args.second().unwrap_or("");

        if field.is_empty() {
            miette::bail!("insert: field name is required");
        }

        let new_value = parse_value(raw_value);

        match &input {
            PipelineData::Value(Value::Array(arr)) => {
                let result = insert_array(arr, field, &new_value);
                Ok(PipelineData::from_value(Value::Array(result)))
            }
            PipelineData::Value(Value::Obj(obj)) => {
                let mut updated = obj.clone();
                // Only insert if field doesn't exist
                if updated.get(field).is_none() {
                    updated.set(field, new_value);
                }
                Ok(PipelineData::from_value(Value::Obj(updated)))
            }
            _ => miette::bail!("insert: input must be an object or array of objects"),
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
        Ok(crate::cmd::pipeline_convert::pipeline_data_to_atom(legacy_out))
    }
}

fn insert_array(arr: &Array, field: &str, new_value: &Value) -> Array {
    let mut result = Array::new();
    for val in arr.iter() {
        if let Value::Obj(obj) = val {
            let mut updated = obj.clone();
            // Only insert if field doesn't exist
            if updated.get(field).is_none() {
                updated.set(field, new_value.clone());
            }
            result.push(Value::Obj(updated));
        } else {
            result.push(val.clone());
        }
    }
    result
}

fn parse_value(s: &str) -> Value {
    if s == "true" {
        Value::Bool(true)
    } else if s == "false" {
        Value::Bool(false)
    } else if s == "null" || s == "nil" {
        Value::Nil
    } else if let Ok(i) = s.parse::<i32>() {
        Value::Int(i)
    } else if let Ok(f) = s.parse::<f64>() {
        Value::Float(f)
    } else {
        Value::str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_new_field() {
        let mut obj = Obj::new();
        obj.set("name", Value::str("alice"));
        let arr = Array::from(vec![Value::Obj(obj)]);
        let result = insert_array(&arr, "age", &Value::Int(30));
        for item in result.iter() {
            if let Value::Obj(o) = item {
                assert_eq!(o.get("age"), Some(Value::Int(30)));
            }
        }
    }

    #[test]
    fn test_insert_existing_field_skipped() {
        let mut obj = Obj::new();
        obj.set("name", Value::str("alice"));
        obj.set("age", Value::Int(25));
        let arr = Array::from(vec![Value::Obj(obj)]);
        let result = insert_array(&arr, "age", &Value::Int(99));
        for item in result.iter() {
            if let Value::Obj(o) = item {
                // Should keep original value
                assert_eq!(o.get("age"), Some(Value::Int(25)));
            }
        }
    }
}
