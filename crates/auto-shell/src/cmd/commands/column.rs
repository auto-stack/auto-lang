use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct ColumnCommand;

impl Command for ColumnCommand {
    fn name(&self) -> &str {
        "column"
    }

    fn signature(&self) -> Signature {
        Signature::new("column", "Columnate lists")
            .flag_with_short("table", 't', "Determine columns from input")
            .flag_with_short("separator", 's', "Column separator for input parsing")
            .flag_with_short("columns", 'c', "Number of output columns")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = get_text(input)?;
        let table_mode = args.has_flag("table");
        let separator = if args.has_flag("separator") {
            args.positionals.first().map(|s| s.as_str()).unwrap_or("  ")
        } else {
            "  "
        };
        let num_columns: usize = if args.has_flag("columns") {
            args.positionals.iter()
                .find_map(|s| s.parse::<usize>().ok())
                .unwrap_or(4)
        } else {
            4
        };

        let result = if table_mode {
            columnate_table(&text, separator)
        } else {
            columnate_items(&text, num_columns)
        };

        Ok(PipelineData::from_text(result))
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy_in = crate::cmd::pipeline_convert::atom_to_pipeline_data(input);
        let legacy_out = self.run(args, legacy_in, shell)?;
        let text = legacy_out.into_text();
        Ok(AtomPipeline::from_atom(Atom::new(Value::str(&text), AtomType::Text)))
    }
}

/// Extract text from PipelineData
fn get_text(input: PipelineData) -> Result<String> {
    match input {
        PipelineData::Text(s) => Ok(s),
        PipelineData::Value(Value::Str(s)) => Ok(s.to_string()),
        PipelineData::Value(Value::Array(arr)) => {
            let lines: Vec<String> = arr.iter().map(|v| v.as_str().to_string()).collect();
            Ok(lines.join("\n"))
        }
        _ => miette::bail!("column: input must be text"),
    }
}

/// Columnate as table: align columns based on separator
pub fn columnate_table(text: &str, separator: &str) -> String {
    let rows: Vec<Vec<&str>> = text
        .lines()
        .map(|line| line.split(separator).map(|s| s.trim()).collect())
        .collect();

    if rows.is_empty() {
        return String::new();
    }

    // Find max width for each column
    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut col_widths = vec![0usize; max_cols];

    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < max_cols {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    // Format rows
    rows.iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(i, cell)| {
                    if i < max_cols - 1 {
                        format!("{:width$}", cell, width = col_widths[i] + 2)
                    } else {
                        cell.to_string()
                    }
                })
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Columnate items into N columns
pub fn columnate_items(text: &str, num_columns: usize) -> String {
    let items: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    if items.is_empty() || num_columns == 0 {
        return String::new();
    }

    let max_width = items.iter().map(|s| s.len()).max().unwrap_or(0) + 2;
    let num_rows = (items.len() + num_columns - 1) / num_columns;

    let mut result = Vec::new();
    for row in 0..num_rows {
        let line: String = (0..num_columns)
            .filter_map(|col| {
                let idx = row + col * num_rows;
                items.get(idx).map(|item| format!("{:width$}", item, width = max_width))
            })
            .collect();
        result.push(line.trim_end().to_string());
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_columnate_table_basic() {
        let text = "Name  Age  City\nAlice 30  NYC\nBob 25  LA";
        let result = columnate_table(text, "  ");
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("Name"));
        assert!(lines[0].contains("Age"));
    }

    #[test]
    fn test_columnate_table_aligns() {
        let text = "a  bb\nccc  d";
        let result = columnate_table(text, "  ");
        let lines: Vec<&str> = result.lines().collect();
        // First column should be padded to width of "ccc" (3) + 2 = 5
        assert!(lines[0].starts_with("a    ")); // "a" padded to 5
    }

    #[test]
    fn test_columnate_items_basic() {
        let text = "apple\nbanana\ncherry\ndate\nelderberry";
        let result = columnate_items(text, 3);
        let lines: Vec<&str> = result.lines().collect();
        // 5 items, 3 columns, 2 rows (ceil division)
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_columnate_items_even() {
        let text = "a\nb\nc\nd";
        let result = columnate_items(text, 2);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_columnate_items_empty() {
        assert_eq!(columnate_items("", 4), "");
    }

    #[test]
    fn test_columnate_table_empty() {
        assert_eq!(columnate_table("", "  "), "");
    }
}
