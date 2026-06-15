use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::{Value, Array};
use miette::Result;

pub struct UpdateCommand;

impl Command for UpdateCommand {
    fn name(&self) -> &str {
        "update"
    }

    fn signature(&self) -> Signature {
        Signature::new("update", "Update a field in each record")
            .required("field", "Field name to update")
            .required("value", "New value for the field")
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
            miette::bail!("update: field name is required");
        }

        let new_value = parse_value(raw_value);

        match &input {
            PipelineData::Value(Value::Array(arr)) => {
                let result = update_array(arr, field, &new_value);
                Ok(PipelineData::from_value(Value::Array(result)))
            }
            PipelineData::Value(Value::Obj(obj)) => {
                let mut updated = obj.clone();
                updated.set(field, new_value);
                Ok(PipelineData::from_value(Value::Obj(updated)))
            }
            _ => miette::bail!("update: input must be an object or array of objects"),
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

fn update_array(arr: &Array, field: &str, new_value: &Value) -> Array {
    let mut result = Array::new();
    for val in arr.iter() {
        if let Value::Obj(obj) = val {
            let mut updated = obj.clone();
            updated.set(field, new_value.clone());
            result.push(Value::Obj(updated));
        } else {
            result.push(val.clone());
        }
    }
    result
}

/// Parse a raw string value into a typed Value
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
    use auto_val::{Array, Obj};

    #[test]
    fn test_parse_value() {
        assert_eq!(parse_value("42"), Value::Int(42));
        assert_eq!(parse_value("3.14"), Value::Float(3.14));
        assert_eq!(parse_value("true"), Value::Bool(true));
        assert_eq!(parse_value("false"), Value::Bool(false));
        assert_eq!(parse_value("null"), Value::Nil);
        assert!(matches!(parse_value("hello"), Value::Str(_)));
    }

    #[test]
    fn test_update_array() {
        let mut obj1 = Obj::new();
        obj1.set("name", Value::str("alice"));
        obj1.set("age", Value::Int(30));
        let mut obj2 = Obj::new();
        obj2.set("name", Value::str("bob"));
        obj2.set("age", Value::Int(25));

        let arr = Array::from(vec![Value::Obj(obj1), Value::Obj(obj2)]);
        let result = update_array(&arr, "age", &Value::Int(99));

        let items: Vec<Value> = result.iter().cloned().collect();
        assert_eq!(items.len(), 2);
        for item in &items {
            if let Value::Obj(obj) = item {
                assert_eq!(obj.get("age"), Some(Value::Int(99)));
            }
        }
    }
}
