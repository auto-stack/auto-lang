use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::{Value, Array};
use miette::Result;

pub struct WhereCommand;

impl Command for WhereCommand {
    fn name(&self) -> &str {
        "where"
    }

    fn signature(&self) -> Signature {
        Signature::new("where", "Filter objects in pipeline based on condition")
            .required("field", "Field name to compare")
            .required("operator", "Comparison operator (==, !=, <, >, <=, >=)")
            .required("value", "Value to compare against")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        if args.positionals.len() < 3 {
            miette::bail!("where: requires field, operator, and value arguments");
        }

        let field = args.positionals[0].clone();
        let operator = args.positionals[1].clone();
        let value_str = args.positionals[2].clone();

        match input {
            PipelineData::Value(Value::Array(arr)) => {
                let mut result = Array::new();

                for item in arr.iter() {
                    if let Value::Obj(obj) = item {
                        if let Some(ref field_value) = obj.get(field.as_str()) {
                            if compare_values(field_value, operator.as_str(), value_str.as_str()) {
                                result.push(item.clone());
                            }
                        }
                    }
                }

                Ok(PipelineData::from_value(Value::Array(result)))
            }
            PipelineData::Value(_) => {
                miette::bail!("where: input must be an array of objects");
            }
            PipelineData::Text(_) => {
                miette::bail!("where: cannot filter text input");
            }
        }
    }
}

/// Compare a field value with a string value using the specified operator
fn compare_values(field_value: &Value, operator: &str, value_str: &str) -> bool {
    match operator {
        "==" => field_matches(field_value, value_str),
        "!=" => !field_matches(field_value, value_str),
        _ => {
            // For numeric comparisons, try to parse as numbers
            match (field_value, value_str.parse::<i64>()) {
                (Value::Int(fv), Ok(rv)) => match operator {
                    "<" => *fv < rv as i32,
                    ">" => *fv > rv as i32,
                    "<=" => *fv <= rv as i32,
                    ">=" => *fv >= rv as i32,
                    _ => false,
                },
                _ => false,
            }
        }
    }
}

/// Check if a field value matches a string value
fn field_matches(field_value: &Value, value_str: &str) -> bool {
    match field_value {
        Value::Str(s) => s.as_str() == value_str,
        Value::Int(i) => {
            if let Ok(num) = value_str.parse::<i32>() {
                *i == num
            } else {
                false
            }
        }
        Value::Bool(b) => {
            if let Ok(bool_val) = value_str.parse::<bool>() {
                *b == bool_val
            } else {
                false
            }
        }
        _ => false,
    }
}
