//! from_csv command - Parse CSV text into a table (Array of Obj)

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::{Array, Obj, Value};
use miette::Result;

pub struct FromCsvCommand;

impl Command for FromCsvCommand {
    fn name(&self) -> &str {
        "from_csv"
    }

    fn signature(&self) -> Signature {
        Signature::new("from_csv", "Parse CSV text into a table")
            .flag_with_short("delimiter", 'd', "Field delimiter (default: comma)")
            .flag("header", "First row is header (default: true)")
            .flag("no-header", "No header row, fields named col0, col1, ...")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = match input {
            PipelineData::Text(s) => s,
            PipelineData::Value(Value::Str(s)) => s.to_string(),
            _ => miette::bail!("from_csv: input must be text"),
        };

        let delimiter = if args.has_flag("delimiter") {
            args.positionals
                .first()
                .map(|s| s.as_str())
                .unwrap_or(",")
                .to_string()
        } else {
            ",".to_string()
        };

        let has_header = !args.has_flag("no-header");
        let table = parse_csv(&text, &delimiter, has_header)?;
        Ok(PipelineData::from_value(Value::Array(table)))
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
// Simple CSV parser
// ---------------------------------------------------------------------------

/// Parse CSV text into an Array of Obj (one Obj per row).
pub fn parse_csv(text: &str, delimiter: &str, has_header: bool) -> Result<Array> {
    let rows = parse_csv_rows(text, delimiter);

    if rows.is_empty() {
        return Ok(Array::new());
    }

    let (headers, data_rows) = if has_header {
        let h = rows[0].clone();
        (h, &rows[1..])
    } else {
        let col_count = rows[0].len();
        let h: Vec<String> = (0..col_count).map(|i| format!("col{}", i)).collect();
        (h, &rows[..])
    };

    let mut result = Array::new();
    for row in data_rows {
        if row.is_empty() || (row.len() == 1 && row[0].is_empty()) {
            continue; // skip blank rows
        }
        let mut obj = Obj::new();
        for (i, val) in row.iter().enumerate() {
            let key = headers.get(i).cloned().unwrap_or_else(|| format!("col{}", i));
            obj.set(key.as_str(), Value::str(val));
        }
        result.push(Value::Obj(obj));
    }

    Ok(result)
}

/// Parse CSV text into rows of fields. Handles quoted fields.
fn parse_csv_rows(text: &str, delimiter: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = text.chars().collect();
    let delim_chars: Vec<char> = delimiter.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if in_quotes {
            if chars[i] == '"' {
                // Check for escaped quote ""
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    current_field.push('"');
                    i += 2;
                } else {
                    in_quotes = false;
                    i += 1;
                }
            } else {
                current_field.push(chars[i]);
                i += 1;
            }
        } else {
            // Check for delimiter
            if starts_with(&chars, i, &delim_chars) {
                current_row.push(current_field.clone());
                current_field.clear();
                i += delim_chars.len();
            } else if chars[i] == '"' && current_field.is_empty() {
                in_quotes = true;
                i += 1;
            } else if chars[i] == '\n' {
                current_row.push(current_field.clone());
                current_field.clear();
                rows.push(std::mem::take(&mut current_row));
                i += 1;
                // Handle \r\n
            } else if chars[i] == '\r' {
                current_row.push(current_field.clone());
                current_field.clear();
                rows.push(std::mem::take(&mut current_row));
                i += 1;
                if i < chars.len() && chars[i] == '\n' {
                    i += 1;
                }
            } else {
                current_field.push(chars[i]);
                i += 1;
            }
        }
    }

    // Last field / last row
    if !current_field.is_empty() || !current_row.is_empty() {
        current_row.push(current_field);
        rows.push(current_row);
    }

    rows
}

fn starts_with(chars: &[char], pos: usize, prefix: &[char]) -> bool {
    if pos + prefix.len() > chars.len() {
        return false;
    }
    &chars[pos..pos + prefix.len()] == prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_csv() {
        let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA";
        let arr = parse_csv(csv, ",", true).unwrap();
        assert_eq!(arr.len(), 2);

        let first = &arr[0];
        if let Value::Obj(obj) = first {
            assert_eq!(obj.get("name").unwrap().as_str(), "Alice");
            assert_eq!(obj.get("age").unwrap().as_str(), "30");
        }
    }

    #[test]
    fn test_no_header() {
        let csv = "Alice,30\nBob,25";
        let arr = parse_csv(csv, ",", false).unwrap();
        assert_eq!(arr.len(), 2);
        if let Value::Obj(obj) = &arr[0] {
            assert_eq!(obj.get("col0").unwrap().as_str(), "Alice");
        }
    }

    #[test]
    fn test_quoted_fields() {
        let csv = "name,desc\nAlice,\"Hello, World\"";
        let arr = parse_csv(csv, ",", true).unwrap();
        if let Value::Obj(obj) = &arr[0] {
            assert_eq!(obj.get("desc").unwrap().as_str(), "Hello, World");
        }
    }

    #[test]
    fn test_escaped_quotes() {
        let csv = "name,msg\nAlice,\"said \"\"hi\"\"\"";
        let arr = parse_csv(csv, ",", true).unwrap();
        if let Value::Obj(obj) = &arr[0] {
            assert_eq!(obj.get("msg").unwrap().as_str(), r#"said "hi""#);
        }
    }

    #[test]
    fn test_custom_delimiter() {
        let csv = "name;age\nAlice;30";
        let arr = parse_csv(csv, ";", true).unwrap();
        if let Value::Obj(obj) = &arr[0] {
            assert_eq!(obj.get("name").unwrap().as_str(), "Alice");
        }
    }

    #[test]
    fn test_empty_csv() {
        let arr = parse_csv("", ",", true).unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_command_name() {
        let cmd = FromCsvCommand;
        assert_eq!(cmd.name(), "from_csv");
    }
}
