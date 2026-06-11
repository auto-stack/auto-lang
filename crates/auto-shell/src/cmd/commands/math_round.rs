use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Value, Array};
use miette::Result;

pub struct MathRoundCommand;

impl Command for MathRoundCommand {
    fn name(&self) -> &str {
        "math-round"
    }

    fn signature(&self) -> Signature {
        Signature::new("math-round", "Round numbers")
            .optional("precision", "Decimal places (default: 0)")
            .flag("floor", "Round down")
            .flag("ceil", "Round up")
            .flag("abs", "Absolute value")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let precision: i32 = args
            .first()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let do_floor = args.has_flag("floor");
        let do_ceil = args.has_flag("ceil");
        let do_abs = args.has_flag("abs");

        let result = match &input {
            PipelineData::Value(Value::Int(i)) => {
                let mut f = *i as f64;
                if do_abs {
                    f = f.abs();
                }
                // Int with no fractional op stays as int
                Value::Int(f as i32)
            }
            PipelineData::Value(Value::Float(f)) => {
                let mut v = *f;
                if do_abs {
                    v = v.abs();
                }
                let rounded = if do_floor {
                    v.floor()
                } else if do_ceil {
                    v.ceil()
                } else {
                    round_to_precision(v, precision)
                };
                f64_to_value(rounded)
            }
            PipelineData::Value(Value::Array(arr)) => {
                let results: Vec<Value> = arr
                    .iter()
                    .map(|v| {
                        let mut f = value_to_f64(&v);
                        if do_abs {
                            f = f.abs();
                        }
                        let rounded = if do_floor {
                            f.floor()
                        } else if do_ceil {
                            f.ceil()
                        } else {
                            round_to_precision(f, precision)
                        };
                        f64_to_value(rounded)
                    })
                    .collect();
                Value::Array(Array::from(results))
            }
            _ => miette::bail!("math-round: input must be a number or array of numbers"),
        };

        Ok(PipelineData::from_value(result))
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

fn value_to_f64(val: &Value) -> f64 {
    match val {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => 0.0,
    }
}

fn f64_to_value(f: f64) -> Value {
    if f.fract() == 0.0 && f.is_finite() {
        Value::Int(f as i32)
    } else {
        Value::Float(f)
    }
}

fn round_to_precision(f: f64, precision: i32) -> f64 {
    let factor = 10_f64.powi(precision);
    (f * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_default() {
        assert_eq!(round_to_precision(3.7, 0), 4.0);
        assert_eq!(round_to_precision(3.3, 0), 3.0);
    }

    #[test]
    fn test_round_precision() {
        assert_eq!(round_to_precision(3.14159, 2), 3.14);
        assert_eq!(round_to_precision(3.14159, 4), 3.1416);
    }

    #[test]
    fn test_f64_to_value() {
        assert_eq!(f64_to_value(4.0), Value::Int(4));
        assert_eq!(f64_to_value(3.14), Value::Float(3.14));
    }
}
