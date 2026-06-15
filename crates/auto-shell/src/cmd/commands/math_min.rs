use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Array};
use miette::Result;

pub struct MathMinCommand;

impl Command for MathMinCommand {
    fn name(&self) -> &str {
        "math-min"
    }

    fn signature(&self) -> Signature {
        Signature::new("math-min", "Find minimum value")
            .optional("field", "Field name for array of objects")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let field = args.first();
        match &input {
            PipelineData::Value(Value::Array(arr)) => {
                let min = find_min(arr, field)?;
                Ok(PipelineData::from_value(min))
            }
            _ => miette::bail!("math-min: input must be an array"),
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
        Ok(AtomPipeline::from_atom(Atom::new(val, AtomType::Nothing)))
    }
}

fn find_min(arr: &Array, field: Option<&str>) -> Result<Value> {
    let mut min: Option<f64> = None;

    for val in arr.iter() {
        let num = if let Some(field_name) = field {
            // Extract field from object
            match &val {
                Value::Obj(obj) => obj
                    .get(field_name)
                    .map(|v| value_to_f64(&v))
                    .unwrap_or(f64::NAN),
                _ => f64::NAN,
            }
        } else {
            value_to_f64(&val)
        };

        if num.is_nan() {
            continue;
        }

        min = Some(match min {
            Some(m) => m.min(num),
            None => num,
        });
    }

    match min {
        Some(m) => Ok(f64_to_value(m)),
        None => miette::bail!("math-min: no numeric values found"),
    }
}

fn value_to_f64(val: &Value) -> f64 {
    match val {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => f64::NAN,
    }
}

fn f64_to_value(f: f64) -> Value {
    if f.fract() == 0.0 && f.is_finite() {
        Value::Int(f as i32)
    } else {
        Value::Float(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::{Array, Obj};

    #[test]
    fn test_min_integers() {
        let arr = Array::from(vec![Value::Int(3), Value::Int(1), Value::Int(2)]);
        let result = find_min(&arr, None).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_min_with_field() {
        let mut obj1 = Obj::new();
        obj1.set("age", Value::Int(30));
        let mut obj2 = Obj::new();
        obj2.set("age", Value::Int(20));
        let arr = Array::from(vec![Value::Obj(obj1), Value::Obj(obj2)]);
        let result = find_min(&arr, Some("age")).unwrap();
        assert_eq!(result, Value::Int(20));
    }
}
