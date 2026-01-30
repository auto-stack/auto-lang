use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::{Value, Array};
use miette::Result;

pub struct GetCommand;

impl Command for GetCommand {
    fn name(&self) -> &str {
        "get"
    }

    fn signature(&self) -> Signature {
        Signature::new("get", "Extract fields from objects in pipeline")
            .required("field", "Field name(s) to extract (can specify multiple)")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        // Extract field names from positionals
        if args.positionals.is_empty() {
            miette::bail!("get: requires at least one field name");
        }

        let fields: Vec<String> = args.positionals.clone();

        match input {
            PipelineData::Value(Value::Array(arr)) => {
                // Extract fields from array of objects
                let mut result = Array::new();

                for item in arr.iter() {
                    if let Value::Obj(obj) = item {
                        // Extract single field
                        if fields.len() == 1 {
                            let field = &fields[0];
                            if let Some(value) = obj.get(field.as_ref()) {
                                result.push(value.clone());
                            }
                        } else {
                            // Extract multiple fields into a new object
                            let mut new_obj = auto_val::Obj::new();
                            for field in &fields {
                                if let Some(value) = obj.get(field.as_ref()) {
                                    new_obj.set(field.as_ref(), value.clone());
                                }
                            }
                            result.push(Value::Obj(new_obj));
                        }
                    }
                }

                Ok(PipelineData::from_value(Value::Array(result)))
            }
            PipelineData::Value(Value::Obj(obj)) => {
                // Extract field from single object
                if fields.len() == 1 {
                    let field = &fields[0];
                    if let Some(value) = obj.get(field.as_ref()) {
                        Ok(PipelineData::from_value(value.clone()))
                    } else {
                        Ok(PipelineData::from_value(Value::Nil))
                    }
                } else {
                    // Extract multiple fields into a new object
                    let mut new_obj = auto_val::Obj::new();
                    for field in &fields {
                        if let Some(value) = obj.get(field.as_ref()) {
                            new_obj.set(field.as_ref(), value.clone());
                        }
                    }
                    Ok(PipelineData::from_value(Value::Obj(new_obj)))
                }
            }
            PipelineData::Value(_) => {
                // Non-object input
                miette::bail!("get: input must be an object or array of objects");
            }
            PipelineData::Text(_) => {
                miette::bail!("get: cannot extract fields from text input");
            }
        }
    }
}
