//! to_toml command - Convert structured Value to TOML text
//!
//! Implements a simple TOML serializer: key = value for scalars,
//! [section] for nested objects, [[array]] for arrays of tables.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::{Array, Obj, Value};
use miette::Result;

pub struct ToTomlCommand;

impl Command for ToTomlCommand {
    fn name(&self) -> &str {
        "to_toml"
    }

    fn signature(&self) -> Signature {
        Signature::new("to_toml", "Convert structured Value to TOML string")
    }

    fn run(
        &self,
        _args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let value = match input {
            PipelineData::Value(v) => v,
            PipelineData::Text(_s) => miette::bail!("to_toml: cannot convert text to TOML; expected structured data"),
        };

        let toml = value_to_toml(&value, &[], 0);
        Ok(PipelineData::from_text(toml))
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
// Simple TOML serializer
// ---------------------------------------------------------------------------

/// Convert a Value to TOML text.
///
/// `path` tracks the current section path for nested objects.
/// `depth` is the indentation level (for formatting only).
pub fn value_to_toml(value: &Value, path: &[&str], depth: usize) -> String {
    match value {
        Value::Obj(obj) => object_to_toml(obj, path, depth),
        _ => {
            // Top-level non-object: wrap in a dummy section
            let mut result = String::new();
            result.push_str(&format_value(value));
            result
        }
    }
}

fn object_to_toml(obj: &Obj, path: &[&str], depth: usize) -> String {
    let mut result = String::new();
    let indent = "  ".repeat(depth);

    // Separate scalars from nested objects/arrays
    let mut scalars: Vec<(String, Value)> = Vec::new();
    let mut nested: Vec<(String, Value)> = Vec::new();
    let mut array_tables: Vec<(String, Value)> = Vec::new();

    for (k, v) in obj.iter() {
        let ks = k.to_string();
        match v {
            Value::Obj(_) => nested.push((ks, v.clone())),
            Value::Array(arr) if is_array_of_tables(arr) => {
                array_tables.push((ks, Value::Array(arr.clone())));
            }
            _ => scalars.push((ks, v.clone())),
        }
    }

    // Section header
    if !path.is_empty() {
        result.push_str(&format!("[{}]\n", path.join(".")));
    }

    // Scalars
    for (key, val) in &scalars {
        result.push_str(&format!("{}{} = {}\n", indent, key, format_value(val)));
    }

    // Nested objects (sub-sections)
    for (key, val) in &nested {
        if let Value::Obj(inner) = val {
            let mut new_path: Vec<&str> = path.to_vec();
            new_path.push(key);
            result.push('\n');
            result.push_str(&object_to_toml(inner, &new_path, depth));
        }
    }

    // Array of tables [[...]]
    for (key, val) in &array_tables {
        if let Value::Array(arr) = val {
            for item in arr.iter() {
                result.push_str(&format!("[[{}]]\n", key));
                if let Value::Obj(inner) = item {
                    for (ik, iv) in inner.iter() {
                        result.push_str(&format!("{} = {}\n", ik, format_value(iv)));
                    }
                }
                result.push('\n');
            }
        }
    }

    result
}

/// Check if an array contains only objects (array-of-tables pattern).
fn is_array_of_tables(arr: &Array) -> bool {
    !arr.is_empty() && arr.iter().all(|v| matches!(v, Value::Obj(_)))
}

/// Format a single value as TOML.
fn format_value(value: &Value) -> String {
    match value {
        Value::Nil => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            let s = format!("{}", f);
            if !s.contains('.') {
                format!("{}.0", s)
            } else {
                s
            }
        }
        Value::Str(s) => format!("\"{}\"", escape_toml_string(s.as_ref())),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(|v| format_value(v)).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Obj(obj) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{} = {}", k, format_value(v)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
        _ => format!("\"{}\"", value.as_str()),
    }
}

/// Escape a string for TOML basic strings.
fn escape_toml_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_scalars() {
        let mut obj = Obj::new();
        obj.set("title", Value::str("Test"));
        obj.set("count", Value::Int(42));
        obj.set("active", Value::Bool(true));

        let toml = value_to_toml(&Value::Obj(obj), &[], 0);
        assert!(toml.contains("title = \"Test\""));
        assert!(toml.contains("count = 42"));
        assert!(toml.contains("active = true"));
    }

    #[test]
    fn test_nested_section() {
        let mut server = Obj::new();
        server.set("host", Value::str("localhost"));
        server.set("port", Value::Int(8080));

        let mut root = Obj::new();
        root.set("title", Value::str("My App"));
        root.set("server", Value::Obj(server));

        let toml = value_to_toml(&Value::Obj(root), &[], 0);
        assert!(toml.contains("[server]"));
        assert!(toml.contains("host = \"localhost\""));
    }

    #[test]
    fn test_inline_array() {
        let mut obj = Obj::new();
        obj.set("ports", Value::Array(Array::from(vec![Value::Int(80), Value::Int(443)])));

        let toml = value_to_toml(&Value::Obj(obj), &[], 0);
        assert!(toml.contains("ports = [80, 443]"));
    }

    #[test]
    fn test_string_escaping() {
        let mut obj = Obj::new();
        obj.set("msg", Value::str("hello\nworld"));

        let toml = value_to_toml(&Value::Obj(obj), &[], 0);
        assert!(toml.contains("msg = \"hello\\nworld\""));
    }

    #[test]
    fn test_float() {
        let mut obj = Obj::new();
        obj.set("pi", Value::Float(3.14));

        let toml = value_to_toml(&Value::Obj(obj), &[], 0);
        assert!(toml.contains("pi = 3.14"));
    }

    #[test]
    fn test_inline_table() {
        let mut point = Obj::new();
        point.set("x", Value::Int(1));
        point.set("y", Value::Int(2));

        let mut root = Obj::new();
        root.set("origin", Value::Obj(point));

        let toml = value_to_toml(&Value::Obj(root), &[], 0);
        // A single-object nested gets a section header
        assert!(toml.contains("x = 1"));
        assert!(toml.contains("y = 2"));
    }

    #[test]
    fn test_command_name() {
        let cmd = ToTomlCommand;
        assert_eq!(cmd.name(), "to_toml");
    }
}
