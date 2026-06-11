//! to_yaml command - Convert structured Value to YAML text
//!
//! Implements a simple YAML serializer for basic structures.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::{Array, Obj, Value};
use miette::Result;

pub struct ToYamlCommand;

impl Command for ToYamlCommand {
    fn name(&self) -> &str {
        "to_yaml"
    }

    fn signature(&self) -> Signature {
        Signature::new("to_yaml", "Convert structured Value to YAML string")
    }

    fn run(
        &self,
        _args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let value = match input {
            PipelineData::Value(v) => v,
            PipelineData::Text(_s) => miette::bail!("to_yaml: cannot convert text to YAML; expected structured data"),
        };

        let yaml = value_to_yaml(&value, 0);
        Ok(PipelineData::from_text(yaml))
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
// Simple YAML serializer
// ---------------------------------------------------------------------------

/// Convert a Value to YAML text with the given indentation level.
pub fn value_to_yaml(value: &Value, indent: usize) -> String {
    match value {
        Value::Nil => "null".to_string(),
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
        Value::Str(s) => format_yaml_string(s.as_ref()),
        Value::Array(arr) => array_to_yaml(arr, indent),
        Value::Obj(obj) => object_to_yaml(obj, indent),
        _ => format_yaml_string(&value.as_str()),
    }
}

fn format_yaml_string(s: &str) -> String {
    // Quote if it contains special YAML characters
    let needs_quoting = s.is_empty()
        || s.contains(':')
        || s.contains('#')
        || s.contains('\n')
        || s.contains('\t')
        || s.starts_with('-')
        || s.starts_with(' ')
        || s.ends_with(' ')
        || ["true", "false", "null", "yes", "no", "True", "False", "Yes", "No"].contains(&s);

    if needs_quoting {
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

fn object_to_yaml(obj: &Obj, indent: usize) -> String {
    let entries: Vec<(String, Value)> = obj.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
    if entries.is_empty() {
        return "{}".to_string();
    }

    let prefix = "  ".repeat(indent);
    let mut lines = Vec::new();

    for (key, val) in &entries {
        match val {
            Value::Obj(inner) => {
                let inner_yaml = object_to_yaml(inner, indent + 1);
                if inner_yaml == "{}" {
                    lines.push(format!("{}{}: {{}}", prefix, key));
                } else {
                    lines.push(format!("{}{}:", prefix, key));
                    lines.push(inner_yaml);
                }
            }
            Value::Array(arr) => {
                lines.push(format!("{}{}:", prefix, key));
                lines.push(array_to_yaml(arr, indent + 1));
            }
            _ => {
                lines.push(format!("{}{}: {}", prefix, key, value_to_yaml(val, indent + 1)));
            }
        }
    }

    lines.join("\n")
}

fn array_to_yaml(arr: &Array, indent: usize) -> String {
    if arr.is_empty() {
        return "[]".to_string();
    }

    let prefix = "  ".repeat(indent);
    let mut lines = Vec::new();

    for item in arr.iter() {
        match item {
            Value::Obj(obj) => {
                let entries: Vec<(String, Value)> = obj.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
                if entries.is_empty() {
                    lines.push(format!("{}- {{}}", prefix));
                } else {
                    let first_prefix = format!("{}- ", prefix);
                    let rest_prefix = "  ".repeat(indent + 1);

                    let (first_key, first_val) = &entries[0];
                    match first_val {
                        Value::Obj(_) | Value::Array(_) => {
                            lines.push(format!("{}- {}:", first_prefix, first_key));
                            lines.push(value_to_yaml(first_val, indent + 2));
                        }
                        _ => {
                            lines.push(format!("{}{}: {}", first_prefix, first_key, value_to_yaml(first_val, indent + 2)));
                        }
                    }

                    for (key, val) in &entries[1..] {
                        match val {
                            Value::Obj(_) | Value::Array(_) => {
                                lines.push(format!("{}{}:", rest_prefix, key));
                                lines.push(value_to_yaml(val, indent + 2));
                            }
                            _ => {
                                lines.push(format!("{}{}: {}", rest_prefix, key, value_to_yaml(val, indent + 2)));
                            }
                        }
                    }
                }
            }
            Value::Array(inner) => {
                lines.push(format!("{}-", prefix));
                lines.push(array_to_yaml(inner, indent + 1));
            }
            _ => {
                lines.push(format!("{}- {}", prefix, value_to_yaml(item, indent + 1)));
            }
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalars() {
        assert_eq!(value_to_yaml(&Value::Nil, 0), "null");
        assert_eq!(value_to_yaml(&Value::Bool(true), 0), "true");
        assert_eq!(value_to_yaml(&Value::Int(42), 0), "42");
        assert_eq!(value_to_yaml(&Value::Float(3.14), 0), "3.14");
    }

    #[test]
    fn test_string_plain() {
        assert_eq!(value_to_yaml(&Value::str("hello"), 0), "hello");
    }

    #[test]
    fn test_string_quoted() {
        assert_eq!(value_to_yaml(&Value::str("hello: world"), 0), "\"hello: world\"");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(value_to_yaml(&Value::str(""), 0), "\"\"");
    }

    #[test]
    fn test_simple_object() {
        let mut obj = Obj::new();
        obj.set("name", Value::str("Alice"));
        obj.set("age", Value::Int(30));

        let yaml = value_to_yaml(&Value::Obj(obj), 0);
        assert!(yaml.contains("name: Alice"));
        assert!(yaml.contains("age: 30"));
    }

    #[test]
    fn test_nested_object() {
        let mut server = Obj::new();
        server.set("host", Value::str("localhost"));
        server.set("port", Value::Int(8080));

        let mut root = Obj::new();
        root.set("server", Value::Obj(server));

        let yaml = value_to_yaml(&Value::Obj(root), 0);
        assert!(yaml.contains("server:"));
        assert!(yaml.contains("  host: localhost"));
        assert!(yaml.contains("  port: 8080"));
    }

    #[test]
    fn test_simple_array() {
        let arr = Array::from(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let yaml = value_to_yaml(&Value::Array(arr), 0);
        let lines: Vec<&str> = yaml.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("- "));
    }

    #[test]
    fn test_array_of_objects() {
        let mut obj1 = Obj::new();
        obj1.set("name", Value::str("Alice"));

        let mut obj2 = Obj::new();
        obj2.set("name", Value::str("Bob"));

        let arr = Array::from(vec![Value::Obj(obj1), Value::Obj(obj2)]);
        let yaml = value_to_yaml(&Value::Array(arr), 0);
        assert!(yaml.contains("- name: Alice"));
        assert!(yaml.contains("- name: Bob"));
    }

    #[test]
    fn test_empty_object() {
        let obj = Obj::new();
        assert_eq!(value_to_yaml(&Value::Obj(obj), 0), "{}");
    }

    #[test]
    fn test_empty_array() {
        let arr = Array::new();
        assert_eq!(value_to_yaml(&Value::Array(arr), 0), "[]");
    }

    #[test]
    fn test_command_name() {
        let cmd = ToYamlCommand;
        assert_eq!(cmd.name(), "to_yaml");
    }
}
