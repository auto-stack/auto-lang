//! to_csv command - Convert table data (Array of Obj) to CSV text

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use auto_val::{Value};
use miette::Result;

pub struct ToCsvCommand;

impl Command for ToCsvCommand {
    fn name(&self) -> &str {
        "to_csv"
    }

    fn signature(&self) -> Signature {
        Signature::new("to_csv", "Convert table data to CSV string")
            .flag_with_short("delimiter", 'd', "Field delimiter (default: comma)")
            .flag("header", "Include header row (default: true)")
            .flag("no-header", "Omit header row")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let value = match input {
            PipelineData::Value(v) => v,
            PipelineData::Text(_s) => miette::bail!("to_csv: cannot convert text to CSV; expected table data"),
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

        let include_header = !args.has_flag("no-header");
        let csv = value_to_csv(&value, &delimiter, include_header)?;
        Ok(PipelineData::from_text(csv))
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
// CSV serializer
// ---------------------------------------------------------------------------

/// Convert a Value (expected Array of Obj) to CSV text.
pub fn value_to_csv(value: &Value, delimiter: &str, include_header: bool) -> Result<String> {
    let arr = match value {
        Value::Array(a) => a,
        Value::Obj(obj) => {
            // Single object → wrap in array
            let mut a = auto_val::Array::new();
            a.push(Value::Obj(obj.clone()));
            return value_to_csv(&Value::Array(a), delimiter, include_header);
        }
        _ => miette::bail!("to_csv: expected array of objects"),
    };

    if arr.is_empty() {
        return Ok(String::new());
    }

    // Collect all keys preserving order from first object
    let mut headers: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for item in arr.iter() {
        if let Value::Obj(obj) = item {
            for (k, _) in obj.iter() {
                if seen.insert(k.to_string()) {
                    headers.push(k.to_string());
                }
            }
        }
    }

    let mut lines = Vec::new();

    // Header row
    if include_header {
        let header_line: String = headers
            .iter()
            .map(|h| escape_csv_field(h, delimiter))
            .collect::<Vec<_>>()
            .join(delimiter);
        lines.push(header_line);
    }

    // Data rows
    for item in arr.iter() {
        if let Value::Obj(obj) = item {
            let row: String = headers
                .iter()
                .map(|h| {
                    let val = obj.get(h.to_string()).map(|v| v.as_str().to_string()).unwrap_or_default();
                    escape_csv_field(&val, delimiter)
                })
                .collect::<Vec<_>>()
                .join(delimiter);
            lines.push(row);
        }
    }

    Ok(lines.join("\n"))
}

/// Escape a CSV field: quote if it contains delimiter, quote, or newline.
fn escape_csv_field(field: &str, delimiter: &str) -> String {
    let needs_quoting = field.contains(delimiter)
        || field.contains('"')
        || field.contains('\n')
        || field.contains('\r');

    if needs_quoting {
        let escaped = field.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::{Array, Obj};

    #[test]
    fn test_simple_csv() {
        let mut obj1 = Obj::new();
        obj1.set("name", Value::str("Alice"));
        obj1.set("age", Value::str("30"));

        let mut obj2 = Obj::new();
        obj2.set("name", Value::str("Bob"));
        obj2.set("age", Value::str("25"));

        let arr = Array::from(vec![Value::Obj(obj1), Value::Obj(obj2)]);
        let csv = value_to_csv(&Value::Array(arr), ",", true).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "name,age");
        assert_eq!(lines[1], "Alice,30");
        assert_eq!(lines[2], "Bob,25");
    }

    #[test]
    fn test_no_header() {
        let mut obj = Obj::new();
        obj.set("x", Value::str("1"));

        let arr = Array::from(vec![Value::Obj(obj)]);
        let csv = value_to_csv(&Value::Array(arr), ",", false).unwrap();
        assert_eq!(csv, "1");
    }

    #[test]
    fn test_quoted_field() {
        let mut obj = Obj::new();
        obj.set("msg", Value::str("hello, world"));

        let arr = Array::from(vec![Value::Obj(obj)]);
        let csv = value_to_csv(&Value::Array(arr), ",", true).unwrap();
        assert!(csv.contains(r#""hello, world""#));
    }

    #[test]
    fn test_escaped_quotes() {
        let mut obj = Obj::new();
        obj.set("msg", Value::str(r#"said "hi""#));

        let arr = Array::from(vec![Value::Obj(obj)]);
        let csv = value_to_csv(&Value::Array(arr), ",", true).unwrap();
        assert!(csv.contains(r#""said ""hi"""#));
    }

    #[test]
    fn test_custom_delimiter() {
        let mut obj = Obj::new();
        obj.set("a", Value::str("1"));
        obj.set("b", Value::str("2"));

        let arr = Array::from(vec![Value::Obj(obj)]);
        let csv = value_to_csv(&Value::Array(arr), ";", true).unwrap();
        assert_eq!(csv, "a;b\n1;2");
    }

    #[test]
    fn test_empty_array() {
        let arr = Array::new();
        let csv = value_to_csv(&Value::Array(arr), ",", true).unwrap();
        assert_eq!(csv, "");
    }

    #[test]
    fn test_single_object() {
        let mut obj = Obj::new();
        obj.set("key", Value::str("val"));

        let csv = value_to_csv(&Value::Obj(obj), ",", true).unwrap();
        assert_eq!(csv, "key\nval");
    }

    #[test]
    fn test_command_name() {
        let cmd = ToCsvCommand;
        assert_eq!(cmd.name(), "to_csv");
    }
}
