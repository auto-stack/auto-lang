//! from_yaml command - Parse YAML text into structured Value
//!
//! Implements a simple YAML parser for basic structures:
//! mappings, sequences, and scalars (strings, numbers, booleans, null).

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Array, Obj, Value};
use miette::Result;

pub struct FromYamlCommand;

impl Command for FromYamlCommand {
    fn name(&self) -> &str {
        "from_yaml"
    }

    fn signature(&self) -> Signature {
        Signature::new("from_yaml", "Parse YAML string into structured Value")
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
            _ => miette::bail!("from_yaml: input must be text"),
        };

        let value = parse_yaml(&text)?;
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
// Simple YAML parser
// ---------------------------------------------------------------------------

/// A pre-parsed line for processing.
struct YamlLine {
    indent: usize,
    content: String,
}

/// Parse YAML text into a Value.
pub fn parse_yaml(text: &str) -> Result<Value> {
    let lines = prepare_lines(text);
    if lines.is_empty() {
        return Ok(Value::Nil);
    }

    let (value, _) = parse_block(&lines, 0, 0)?;
    Ok(value)
}

/// Prepare lines: strip comments, compute indentation, skip blanks.
fn prepare_lines(text: &str) -> Vec<YamlLine> {
    let mut result = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Strip inline comment (not inside a string)
        let content = strip_yaml_comment(trimmed);
        let indent = line.len() - line.trim_start().len();
        result.push(YamlLine {
            indent,
            content: content.trim_end().to_string(),
        });
    }
    result
}

fn strip_yaml_comment(s: &str) -> &str {
    let mut in_quote = false;
    let mut quote_char = ' ';
    for (i, c) in s.char_indices() {
        if in_quote {
            if c == quote_char {
                in_quote = false;
            }
        } else if c == '"' || c == '\'' {
            in_quote = true;
            quote_char = c;
        } else if c == '#' {
            return &s[..i];
        }
    }
    s
}

/// Parse a block of YAML starting at `start_idx` with minimum indentation `min_indent`.
/// Returns (value, next_index).
fn parse_block(lines: &[YamlLine], start_idx: usize, min_indent: usize) -> Result<(Value, usize)> {
    if start_idx >= lines.len() || lines[start_idx].indent < min_indent {
        return Ok((Value::Nil, start_idx));
    }

    let first = &lines[start_idx];

    // Sequence item (starts with "- ")
    if first.content.starts_with("- ") {
        return parse_sequence(lines, start_idx, min_indent);
    }

    // Mapping (contains ": ")
    if first.content.contains(": ") || first.content.ends_with(':') {
        return parse_mapping(lines, start_idx, min_indent);
    }

    // Plain scalar
    let val = parse_scalar(&first.content);
    Ok((val, start_idx + 1))
}

/// Parse a YAML mapping.
fn parse_mapping(lines: &[YamlLine], start_idx: usize, min_indent: usize) -> Result<(Value, usize)> {
    let mut obj = Obj::new();
    let mut idx = start_idx;

    while idx < lines.len() && lines[idx].indent >= min_indent {
        let line = &lines[idx];

        if line.indent < min_indent {
            break;
        }

        // Sequence items break out of mapping
        if line.content.starts_with("- ") {
            break;
        }

        // Parse key: value
        if let Some(colon_pos) = find_mapping_colon(&line.content) {
            let key = line.content[..colon_pos].trim();
            let after_colon = line.content[colon_pos + 1..].trim();

            if after_colon.is_empty() {
                // Value is on subsequent lines (nested block)
                let nested_indent = line.indent + 2;
                idx += 1;
                if idx < lines.len() && lines[idx].indent >= nested_indent {
                    let (val, next) = parse_block(lines, idx, nested_indent)?;
                    obj.set(key, val);
                    idx = next;
                } else {
                    obj.set(key, Value::Nil);
                }
            } else {
                // Inline value — could be a sequence or scalar
                if after_colon.starts_with("- ") {
                    // Inline sequence: key: - a\n  - b (or continuation)
                    // Treat the rest as first element, then continue sequence
                    let first_val = parse_scalar(&after_colon[2..]);
                    let mut arr = Array::new();
                    arr.push(first_val);
                    let nested_indent = line.indent + 2;
                    idx += 1;
                    while idx < lines.len()
                        && lines[idx].indent >= nested_indent
                        && lines[idx].content.starts_with("- ")
                    {
                        let item_content = &lines[idx].content[2..];
                        let val = parse_scalar(item_content.trim());
                        arr.push(val);
                        idx += 1;
                    }
                    obj.set(key, Value::Array(arr));
                } else {
                    let val = parse_scalar(after_colon);
                    obj.set(key, val);
                    idx += 1;
                }
            }
        } else {
            break;
        }
    }

    Ok((Value::Obj(obj), idx))
}

/// Find the colon that separates key from value in a mapping line.
/// Returns None if no valid mapping colon found.
fn find_mapping_colon(s: &str) -> Option<usize> {
    let mut in_quote = false;
    let mut quote_char = ' ';
    for (i, c) in s.char_indices() {
        if in_quote {
            if c == quote_char {
                in_quote = false;
            }
            continue;
        }
        if c == '"' || c == '\'' {
            in_quote = true;
            quote_char = c;
            continue;
        }
        if c == ':' {
            // Colon is a mapping separator if followed by space or end of string
            let rest = s.get(i + 1..);
            if rest.map_or(true, |r| r.starts_with(' ') || r.is_empty()) {
                return Some(i);
            }
        }
    }
    None
}

/// Parse a YAML sequence.
fn parse_sequence(lines: &[YamlLine], start_idx: usize, min_indent: usize) -> Result<(Value, usize)> {
    let mut arr = Array::new();
    let mut idx = start_idx;

    while idx < lines.len() {
        let line = &lines[idx];
        if line.indent < min_indent || !line.content.starts_with("- ") {
            break;
        }

        let item_content = line.content[2..].trim();

        // Check if the item is a nested mapping
        if item_content.contains(": ") || item_content.ends_with(':') {
            // The item itself is a mapping on the same line or continues on next lines
            // Create a synthetic line for the mapping at the correct indent
            let item_indent = line.indent + 2;
            let synthetic = YamlLine {
                indent: item_indent,
                content: item_content.to_string(),
            };
            let (val, _) = parse_mapping(&[synthetic], 0, item_indent)?;
            arr.push(val);
            idx += 1;

            // Continue reading nested lines that belong to this item
            while idx < lines.len() && lines[idx].indent >= item_indent
                && !lines[idx].content.starts_with("- ")
            {
                // These lines are part of the last mapping; re-parse the full block
                // For simplicity, collect all lines of this item and re-parse
                let mut block_lines = vec![YamlLine {
                    indent: item_indent,
                    content: item_content.to_string(),
                }];
                let block_start = idx;
                while idx < lines.len() && lines[idx].indent >= item_indent
                    && !lines[idx].content.starts_with("- ")
                {
                    block_lines.push(YamlLine {
                        indent: lines[idx].indent,
                        content: lines[idx].content.clone(),
                    });
                    idx += 1;
                }
                let (val, _) = parse_mapping(&block_lines, 0, item_indent)?;
                // Replace last entry with the full parsed value
                if let Some(last) = arr.iter_mut().last() {
                    *last = val;
                }
                // No need to continue, the block is fully parsed
                break;
            }
        } else {
            // Check for multi-line value (next lines at deeper indent)
            let item_indent = line.indent + 2;
            idx += 1;
            if idx < lines.len() && lines[idx].indent >= item_indent
                && !lines[idx].content.starts_with("- ")
            {
                let (val, next) = parse_block(lines, idx, item_indent)?;
                arr.push(val);
                idx = next;
            } else {
                arr.push(parse_scalar(item_content));
            }
        }
    }

    Ok((Value::Array(arr), idx))
}

/// Parse a scalar YAML value.
fn parse_scalar(s: &str) -> Value {
    let s = s.trim();

    // Quoted strings
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        return Value::str(&s[1..s.len() - 1]);
    }

    // Special values
    match s {
        "null" | "~" | "" => return Value::Nil,
        "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" => return Value::Bool(true),
        "false" | "False" | "FALSE" | "no" | "No" | "NO" => return Value::Bool(false),
        _ => {}
    }

    // Integer
    if let Ok(i) = s.parse::<i32>() {
        return Value::Int(i);
    }

    // Float
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }

    // Default: string
    Value::str(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mapping() {
        let yaml = "name: Alice\nage: 30";
        let val = parse_yaml(yaml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("name").unwrap().as_str(), "Alice");
                assert_eq!(obj.get("age").unwrap(), Value::Int(30));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_sequence() {
        let yaml = "- apple\n- banana\n- cherry";
        let val = parse_yaml(yaml).unwrap();
        match val {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0].as_str(), "apple");
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn test_nested_mapping() {
        let yaml = "server:\n  host: localhost\n  port: 8080";
        let val = parse_yaml(yaml).unwrap();
        match val {
            Value::Obj(obj) => match obj.get("server").unwrap() {
                Value::Obj(server) => {
                    assert_eq!(server.get("host").unwrap().as_str(), "localhost");
                    assert_eq!(server.get("port").unwrap(), Value::Int(8080));
                }
                _ => panic!("expected object"),
            },
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_sequence_of_mappings() {
        let yaml = "- name: Alice\n  age: 30\n- name: Bob\n  age: 25";
        let val = parse_yaml(yaml).unwrap();
        match val {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn test_quoted_strings() {
        let yaml = "msg: \"hello world\"\npath: '/usr/bin'";
        let val = parse_yaml(yaml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("msg").unwrap().as_str(), "hello world");
                assert_eq!(obj.get("path").unwrap().as_str(), "/usr/bin");
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_bool_and_null() {
        let yaml = "active: true\ndisabled: false\nempty: null";
        let val = parse_yaml(yaml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("active").unwrap(), Value::Bool(true));
                assert_eq!(obj.get("disabled").unwrap(), Value::Bool(false));
                assert_eq!(obj.get("empty").unwrap(), Value::Nil);
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_float() {
        let yaml = "pi: 3.14";
        let val = parse_yaml(yaml).unwrap();
        match val {
            Value::Obj(obj) => {
                assert_eq!(obj.get("pi").unwrap(), Value::Float(3.14));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_empty() {
        let val = parse_yaml("").unwrap();
        assert_eq!(val, Value::Nil);
    }

    #[test]
    fn test_command_name() {
        let cmd = FromYamlCommand;
        assert_eq!(cmd.name(), "from_yaml");
    }
}
