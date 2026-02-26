use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::{Value, Array};
use miette::Result;

pub struct SelectCommand;

impl Command for SelectCommand {
    fn name(&self) -> &str {
        "select"
    }

    fn signature(&self) -> Signature {
        Signature::new("select", "Select specific fields from objects in pipeline")
            .required("fields", "Field name(s) to select (can specify multiple)")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        // Extract field names from positionals
        if args.positionals.is_empty() {
            miette::bail!("select: requires at least one field name");
        }

        let fields: Vec<&str> = args.positionals.iter().map(|s| s.as_str()).collect();

        match input {
            PipelineData::Value(Value::Array(arr)) => {
                // Select fields from array of objects
                let mut result = Array::new();

                for item in arr.iter() {
                    if let Value::Obj(obj) = item {
                        let mut new_obj = auto_val::Obj::new();
                        for field in &fields {
                            if let Some(value) = obj.get(*field) {
                                new_obj.set(*field, value.clone());
                            }
                        }
                        result.push(Value::Obj(new_obj));
                    }
                }

                Ok(PipelineData::from_value(Value::Array(result)))
            }
            PipelineData::Value(Value::Obj(obj)) => {
                // Select fields from single object
                let mut new_obj = auto_val::Obj::new();
                for field in &fields {
                    if let Some(value) = obj.get(*field) {
                        new_obj.set(*field, value.clone());
                    }
                }
                Ok(PipelineData::from_value(Value::Obj(new_obj)))
            }
            PipelineData::Value(_) => {
                miette::bail!("select: input must be an object or array of objects");
            }
            PipelineData::Text(_) => {
                miette::bail!("select: cannot select fields from text input");
            }
        }
    }
}
