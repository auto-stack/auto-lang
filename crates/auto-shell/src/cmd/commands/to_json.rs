//! to_json command - Convert structured Value to JSON text

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::Value;
use miette::Result;

pub struct ToJsonCommand;

impl Command for ToJsonCommand {
    fn name(&self) -> &str {
        "to_json"
    }

    fn signature(&self) -> Signature {
        Signature::new("to_json", "Convert structured Value to JSON string")
            .flag("pretty", "Indent output with 2 spaces")
            .flag("compact", "Single line output (default)")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let value = match input {
            PipelineData::Value(v) => v,
            PipelineData::Text(s) => Value::str(&s),
        };

        let pretty = args.has_flag("pretty");
        let json = value_to_json(&value, if pretty { 2 } else { 0 }, 0);
        Ok(PipelineData::from_text(json))
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

// ---------------------------------------------------------------------------
// Minimal JSON serializer (no external crates)
// ---------------------------------------------------------------------------

/// Convert a Value to a JSON string.
///
/// `indent_size` = 0 means compact (no indentation).
/// `depth` is the current nesting level.
pub fn value_to_json(value: &Value, indent_size: usize, depth: usize) -> String {
    match value {
        Value::Nil => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                "null".to_string()
            } else {
                let s = format!("{:.}", f);
                // Ensure there's always a decimal point for floats
                if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                    format!("{}.0", s)
                } else {
                    s
                }
            }
        }
        Value::Str(s) => string_to_json(s.as_ref()),
        Value::Array(arr) => array_to_json(arr, indent_size, depth),
        Value::Obj(obj) => object_to_json(obj, indent_size, depth),
        _ => string_to_json(&value.as_str()),
    }
}

fn string_to_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn object_to_json(obj: &auto_val::Obj, indent_size: usize, depth: usize) -> String {
    let entries: Vec<(String, Value)> = obj.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
    if entries.is_empty() {
        return "{}".to_string();
    }

    let pretty = indent_size > 0;
    let indent_inner = if pretty {
        " ".repeat(indent_size * (depth + 1))
    } else {
        String::new()
    };
    let indent_close = if pretty {
        " ".repeat(indent_size * depth)
    } else {
        String::new()
    };

    let items: Vec<String> = entries
        .into_iter()
        .map(|(k, v)| {
            let key_json = string_to_json(&k);
            let val_json = value_to_json(&v, indent_size, depth + 1);
            if pretty {
                format!("{}{}: {}", indent_inner, key_json, val_json)
            } else {
                format!("{}:{}", key_json, val_json)
            }
        })
        .collect();

    if pretty {
        format!("{{\n{}\n{}}}", items.join(",\n"), indent_close)
    } else {
        format!("{{{}}}", items.join(","))
    }
}

fn array_to_json(arr: &auto_val::Array, indent_size: usize, depth: usize) -> String {
    if arr.is_empty() {
        return "[]".to_string();
    }

    let pretty = indent_size > 0;
    let indent_inner = if pretty {
        " ".repeat(indent_size * (depth + 1))
    } else {
        String::new()
    };
    let indent_close = if pretty {
        " ".repeat(indent_size * depth)
    } else {
        String::new()
    };

    let items: Vec<String> = arr
        .iter()
        .map(|v| {
            let val_json = value_to_json(&v, indent_size, depth + 1);
            if pretty {
                format!("{}{}", indent_inner, val_json)
            } else {
                val_json
            }
        })
        .collect();

    if pretty {
        format!("[\n{}\n{}]", items.join(",\n"), indent_close)
    } else {
        format!("[{}]", items.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::{Array, Obj};

    #[test]
    fn test_null() {
        assert_eq!(value_to_json(&Value::Nil, 0, 0), "null");
    }

    #[test]
    fn test_bool() {
        assert_eq!(value_to_json(&Value::Bool(true), 0, 0), "true");
        assert_eq!(value_to_json(&Value::Bool(false), 0, 0), "false");
    }

    #[test]
    fn test_int() {
        assert_eq!(value_to_json(&Value::Int(42), 0, 0), "42");
    }

    #[test]
    fn test_float() {
        let result = value_to_json(&Value::Float(3.14), 0, 0);
        assert!(result.contains('.'));
    }

    #[test]
    fn test_string() {
        assert_eq!(value_to_json(&Value::str("hello"), 0, 0), r#""hello""#);
    }

    #[test]
    fn test_string_escapes() {
        assert_eq!(
            value_to_json(&Value::str("a\nb\tc"), 0, 0),
            r#""a\nb\tc""#
        );
    }

    #[test]
    fn test_empty_array() {
        let arr = Array::new();
        assert_eq!(value_to_json(&Value::Array(arr), 0, 0), "[]");
    }

    #[test]
    fn test_array() {
        let arr = Array::from(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(value_to_json(&Value::Array(arr), 0, 0), "[1,2,3]");
    }

    #[test]
    fn test_empty_object() {
        let obj = Obj::new();
        assert_eq!(value_to_json(&Value::Obj(obj), 0, 0), "{}");
    }

    #[test]
    fn test_object() {
        let mut obj = Obj::new();
        obj.set("name", Value::str("Alice"));
        obj.set("age", Value::Int(30));
        let json = value_to_json(&Value::Obj(obj), 0, 0);
        assert!(json.contains(r#""name":"Alice""#));
        assert!(json.contains(r#""age":30"#));
    }

    #[test]
    fn test_pretty_output() {
        let mut obj = Obj::new();
        obj.set("x", Value::Int(1));
        let json = value_to_json(&Value::Obj(obj), 2, 0);
        assert!(json.contains('\n'));
        assert!(json.contains("  "));
    }

    #[test]
    fn test_nested() {
        let inner = Array::from(vec![Value::Int(1), Value::Int(2)]);
        let mut obj = Obj::new();
        obj.set("nums", Value::Array(inner));
        let json = value_to_json(&Value::Obj(obj), 0, 0);
        assert_eq!(json, r#"{"nums":[1,2]}"#);
    }

    #[test]
    fn test_command_name() {
        let cmd = ToJsonCommand;
        assert_eq!(cmd.name(), "to_json");
    }
}
