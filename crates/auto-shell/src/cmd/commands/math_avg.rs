use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Array};
use miette::Result;

pub struct MathAvgCommand;

impl Command for MathAvgCommand {
    fn name(&self) -> &str {
        "math-avg"
    }

    fn signature(&self) -> Signature {
        Signature::new("math-avg", "Average numeric values in an array")
    }

    fn run(
        &self,
        _args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        match &input {
            PipelineData::Value(Value::Array(arr)) => {
                let avg = avg_array(arr);
                Ok(PipelineData::from_value(Value::Float(avg)))
            }
            _ => miette::bail!("math-avg: input must be an array"),
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

fn avg_array(arr: &Array) -> f64 {
    if arr.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0f64;
    let mut count = 0usize;
    for val in arr.iter() {
        match val {
            Value::Int(i) => {
                sum += *i as f64;
                count += 1;
            }
            Value::Float(f) => {
                sum += f;
                count += 1;
            }
            _ => {}
        }
    }
    if count == 0 {
        0.0
    } else {
        sum / count as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avg_integers() {
        let arr = Array::from(vec![Value::Int(2), Value::Int(4), Value::Int(6)]);
        assert_eq!(avg_array(&arr), 4.0);
    }

    #[test]
    fn test_avg_floats() {
        let arr = Array::from(vec![Value::Float(1.0), Value::Float(3.0)]);
        assert_eq!(avg_array(&arr), 2.0);
    }

    #[test]
    fn test_avg_empty() {
        let arr = Array::new();
        assert_eq!(avg_array(&arr), 0.0);
    }

    #[test]
    fn test_avg_skips_non_numeric() {
        let arr = Array::from(vec![Value::Int(10), Value::str("x"), Value::Int(20)]);
        assert_eq!(avg_array(&arr), 15.0);
    }
}
