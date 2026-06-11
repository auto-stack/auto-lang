use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Array};
use miette::Result;

pub struct MathSumCommand;

impl Command for MathSumCommand {
    fn name(&self) -> &str {
        "math-sum"
    }

    fn signature(&self) -> Signature {
        Signature::new("math-sum", "Sum numeric values in an array or record")
    }

    fn run(
        &self,
        _args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        match &input {
            PipelineData::Value(Value::Array(arr)) => {
                let sum = sum_array(arr);
                Ok(PipelineData::from_value(sum))
            }
            PipelineData::Value(Value::Obj(obj)) => {
                // Sum all numeric fields in a record
                let mut sum = 0.0f64;
                for (_, val) in obj.iter() {
                    sum += value_to_f64(val);
                }
                Ok(PipelineData::from_value(f64_to_value(sum)))
            }
            _ => miette::bail!("math-sum: input must be an array or record"),
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

fn sum_array(arr: &Array) -> Value {
    let mut sum = 0.0f64;
    for val in arr.iter() {
        sum += value_to_f64(&val);
    }
    f64_to_value(sum)
}

/// Extract numeric value, returns 0.0 for non-numeric types
pub fn value_to_f64(val: &Value) -> f64 {
    match val {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => 0.0,
    }
}

/// Convert f64 back to Value (Int if whole number, Float otherwise)
pub fn f64_to_value(f: f64) -> Value {
    if f.fract() == 0.0 && f.is_finite() {
        Value::Int(f as i32)
    } else {
        Value::Float(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_f64() {
        assert_eq!(value_to_f64(&Value::Int(42)), 42.0);
        assert_eq!(value_to_f64(&Value::Float(3.14)), 3.14);
        assert_eq!(value_to_f64(&Value::Bool(true)), 0.0);
    }

    #[test]
    fn test_f64_to_value() {
        assert_eq!(f64_to_value(42.0), Value::Int(42));
        assert_eq!(f64_to_value(3.14), Value::Float(3.14));
    }

    #[test]
    fn test_sum_array() {
        let arr = Array::from(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = sum_array(&arr);
        assert_eq!(result, Value::Int(6));
    }

    #[test]
    fn test_sum_mixed() {
        let arr = Array::from(vec![Value::Int(1), Value::Float(2.5), Value::Int(3)]);
        let result = sum_array(&arr);
        assert_eq!(result, Value::Float(6.5));
    }
}
