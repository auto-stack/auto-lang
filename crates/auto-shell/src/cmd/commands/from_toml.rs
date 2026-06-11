//! from_toml command - Parse TOML text into structured Value
//!
//! Implements a simple TOML parser for basic key=value pairs and [sections].
//! Supports: strings, integers, floats, booleans, arrays, inline tables.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::{Array, Obj, Value};
use miette::Result;

pub struct FromTomlCommand;

impl Command for FromTomlCommand {
    fn name(&self) -> &str {
        "from_toml"
    }

    fn signature(&self) -> Signature {
        Signature::new("from_toml", "Parse TOML string into structured Value")
    }

    fn run(
        &self,
        _args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = match input {
            PipelineData::Text(s) => s,
            PipelineData::Value(Value::Str(s)) => s.to_string(),
            _ => miette::bail!("from_toml: input must be text"),
        };

        let value = parse_toml(&text)?;
        Ok(PipelineData::from_value(value))
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
// Simple TOML parser
// ---------------------------------------------------------------------------

/// Parse a TOML string into a Value (Obj at top level).
pub fn parse_toml(text: &str) -> Result<Value> {
    let mut root = Obj::new();
    let mut current_section: Vec<String> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Section header [section] or [[array_of_tables]]
        if trimmed.starts_with('[') {
            if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
                // Array of tables: [[section]]
                let name = trimmed[2..trimmed.len() - 2].trim();
                current_section = name.split('.').map(|s| s.trim().to_string()).collect();
                // Ensure the array exists at the path
                ensure_array_path(&mut root, &current_section);
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let name = trimmed[1..trimmed.len() - 1].trim();
                current_section = name.split('.').map(|s| s.trim().to_string()).collect();
            }
            continue;
        }

        // Key = value
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            let val_str = trimmed[eq_pos + 1..].trim();

            let value = parse_toml_value(val_str)?;
            set_at_path(&mut root, &current_section, key, value);
        }
    }

    Ok(Value::Obj(root))
}

/// Parse a TOML value (string, number, bool, array, inline table).
fn parse_toml_value(s: &str) -> Result<Value> {
    let s = s.trim();

    // Remove inline comment
    let s = strip_comment(s);

    // String (basic "..." or literal '...')
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        let inner = &s[1..s.len() - 1];
        return Ok(Value::str(inner));
    }

    // Boolean
    if s == "true" {
        return Ok(Value::Bool(true));
    }
    if s == "false" {
        return Ok(Value::Bool(false));
    }

    // Inline array [...]
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        let mut arr = Array::new();
        if !inner.trim().is_empty() {
            for item in split_toml_array(inner) {
                arr.push(parse_toml_value(item.trim())?);
            }
        }
        return Ok(Value::Array(arr));
    }

    // Inline table {...}
    if s.starts_with('{') && s.ends_with('}') {
        let inner = &s[1..s.len() - 1];
        let mut obj = Obj::new();
        if !inner.trim().is_empty() {
            for item in split_toml_inline_table(inner) {
                if let Some(eq) = item.find('=') {
                    let key = item[..eq].trim();
                    let val = parse_toml_value(item[eq + 1..].trim())?;
                    obj.set(key, val);
                }
            }
        }
        return Ok(Value::Obj(obj));
    }

    // Integer
    if let Ok(i) = s.parse::<i32>() {
        return Ok(Value::Int(i));
    }

    // Float
    if let Ok(f) = s.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    // Fallback: treat as string
    Ok(Value::str(s))
}

/// Strip an inline comment (after # not inside a string).
fn strip_comment(s: &str) -> &str {
    let mut in_string = false;
    let mut quote_char = b' ';
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            if b == quote_char {
                in_string = false;
            }
        } else if b == b'"' || b == b'\'' {
            in_string = true;
            quote_char = b;
        } else if b == b'#' {
            return &s[..i].trim_end();
        }
    }
    s
}

/// Split array contents by comma, respecting nested brackets/braces.
fn split_toml_array(s: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                items.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        items.push(&s[start..]);
    }
    items
}

/// Split inline table by comma, respecting nested brackets/braces.
fn split_toml_inline_table(s: &str) -> Vec<&str> {
    split_toml_array(s) // same logic
}

/// Set a value at a dotted path within the root object, respecting sections.
/// Uses a clone-and-rebuild approach to avoid borrow-checker issues.
fn set_at_path(root: &mut Obj, section: &[String], key: &str, value: Value) {
    if section.is_empty() {
        root.set(key, value);
        return;
    }

    // Ensure all intermediate objects exist, then set the key in the last one.
    ensure_path(root, section);
    set_in_nested(root, section, key, value);
}

/// Ensure all intermediate objects in the path exist.
fn ensure_path(root: &mut Obj, path: &[String]) {
    if path.is_empty() {
        return;
    }
    let first = path[0].as_str();
    if !root.has(first) {
        root.set(first, Value::Obj(Obj::new()));
    }
    if path.len() > 1 {
        // Clone, recurse, put back
        let sub = root.get(first).unwrap();
        let mut sub_obj = match sub {
            Value::Obj(o) => o,
            _ => Obj::new(),
        };
        ensure_path(&mut sub_obj, &path[1..]);
        root.set(first, Value::Obj(sub_obj));
    }
}

/// Set a key in a nested object at the given path.
fn set_in_nested(root: &mut Obj, path: &[String], key: &str, value: Value) {
    if path.len() == 1 {
        let last = path[0].as_str();
        if let Some(Value::Obj(mut obj)) = root.get(last) {
            obj.set(key, value);
            root.set(last, Value::Obj(obj));
        } else {
            let mut obj = Obj::new();
            obj.set(key, value);
            root.set(last, Value::Obj(obj));
        }
    } else {
        let first = path[0].as_str();
        if let Some(Value::Obj(mut obj)) = root.get(first) {
            set_in_nested(&mut obj, &path[1..], key, value);
            root.set(first, Value::Obj(obj));
        }
    }
}

/// Ensure an array-of-tables entry exists at the given path.
fn ensure_array_path(root: &mut Obj, path: &[String]) {
    if path.is_empty() {
        return;
    }

    let key = path[path.len() - 1].as_str();
    let parent_path = &path[..path.len() - 1];

    // Ensure parent objects exist
    ensure_path(root, parent_path);

    // Get or create the array at the key
    if let Some(Value::Array(mut arr)) = get_at_path(root, parent_path).and_then(|v| match v {
        Value::Obj(obj) => obj.get(key),
        _ => None,
    }) {
        arr.push(Value::Obj(Obj::new()));
        set_array_at_path(root, parent_path, key, arr);
    } else {
        let mut arr = Array::new();
        arr.push(Value::Obj(Obj::new()));
        set_array_at_path(root, parent_path, key, arr);
    }
}

/// Get the Obj at a path from root.
fn get_at_path(root: &Obj, path: &[String]) -> Option<Value> {
    if path.is_empty() {
        return Some(Value::Obj(root.clone()));
    }
    let first = path[0].as_str();
    let sub = root.get(first)?;
    match sub {
        Value::Obj(obj) if path.len() > 1 => get_at_path(&obj, &path[1..]),
        other => Some(other),
    }
}

/// Set an array at a key within a nested path.
fn set_array_at_path(root: &mut Obj, path: &[String], key: &str, arr: Array) {
    if path.is_empty() {
        root.set(key, Value::Array(arr));
        return;
    }
    let first = path[0].as_str();
    if let Some(Value::Obj(mut obj)) = root.get(first) {
        set_array_at_path(&mut obj, &path[1..], key, arr);
        root.set(first, Value::Obj(obj));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_key_value() {
        let toml = r#"
title = "TOML Example"
owner = "Alice"
"#;
        let val = parse_toml(toml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("title").unwrap().as_str(), "TOML Example");
                assert_eq!(obj.get("owner").unwrap().as_str(), "Alice");
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_types() {
        let toml = r#"
int_val = 42
float_val = 3.14
bool_val = true
str_val = "hello"
"#;
        let val = parse_toml(toml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("int_val").unwrap(), Value::Int(42));
                assert_eq!(obj.get("float_val").unwrap(), Value::Float(3.14));
                assert_eq!(obj.get("bool_val").unwrap(), Value::Bool(true));
                assert_eq!(obj.get("str_val").unwrap().as_str(), "hello");
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_section() {
        let toml = r#"
[server]
host = "localhost"
port = 8080
"#;
        let val = parse_toml(toml).unwrap();
        match val {
            Value::Obj(obj) => {
                let server = obj.get("server").unwrap();
                match server {
                    Value::Obj(s) => {
                        assert_eq!(s.get("host").unwrap().as_str(), "localhost");
                        assert_eq!(s.get("port").unwrap(), Value::Int(8080));
                    }
                    _ => panic!("expected object"),
                }
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_inline_array() {
        let toml = r#"ports = [80, 443, 8080]"#;
        let val = parse_toml(toml).unwrap();
        match val {
            Value::Obj(obj) => match obj.get("ports").unwrap() {
                Value::Array(arr) => assert_eq!(arr.len(), 3),
                _ => panic!("expected array"),
            },
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_inline_table() {
        let toml = r#"point = {x = 1, y = 2}"#;
        let val = parse_toml(toml).unwrap();
        match val {
            Value::Obj(obj) => match obj.get("point").unwrap() {
                Value::Obj(pt) => {
                    assert_eq!(pt.get("x").unwrap(), Value::Int(1));
                    assert_eq!(pt.get("y").unwrap(), Value::Int(2));
                }
                _ => panic!("expected object"),
            },
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_comments() {
        let toml = r#"
# This is a comment
key = "value" # inline comment
"#;
        let val = parse_toml(toml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("key").unwrap().as_str(), "value");
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_command_name() {
        let cmd = FromTomlCommand;
        assert_eq!(cmd.name(), "from_toml");
    }
}
