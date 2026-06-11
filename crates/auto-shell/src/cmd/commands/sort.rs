use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

pub struct SortCommand;

impl Command for SortCommand {
    fn name(&self) -> &str {
        "sort"
    }

    fn signature(&self) -> Signature {
        Signature::new("sort", "Sort lines of text")
            .optional("file", "File to sort (default: stdin)")
            .flag_with_short("reverse", 'r', "Reverse sort order")
            .flag_with_short("numeric", 'n', "Numeric sort")
            .flag_with_short("unique", 'u', "Remove duplicate lines")
            .flag_with_short("ignore-case", 'f', "Fold lower case to upper case for comparison")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = if let Some(path) = args.first() {
            std::fs::read_to_string(Path::new(path)).into_diagnostic()?
        } else {
            get_text(input)?
        };

        let reverse = args.has_flag("reverse");
        let numeric = args.has_flag("numeric");
        let unique = args.has_flag("unique");
        let ignore_case = args.has_flag("ignore-case");

        let sorted = sort_lines(&text, reverse, numeric, unique, ignore_case);
        Ok(PipelineData::from_text(sorted))
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
        _ => miette::bail!("sort: input must be text"),
    }
}

/// Sort lines according to the given flags
pub fn sort_lines(
    text: &str,
    reverse: bool,
    numeric: bool,
    unique: bool,
    ignore_case: bool,
) -> String {
    let mut lines: Vec<&str> = text.lines().collect();

    lines.sort_by(|a, b| {
        let cmp = if numeric {
            compare_numeric(a, b, ignore_case)
        } else if ignore_case {
            a.to_lowercase().cmp(&b.to_lowercase())
        } else {
            a.cmp(b)
        };
        if reverse {
            cmp.reverse()
        } else {
            cmp
        }
    });

    if unique {
        lines.dedup_by(|a, b| {
            if ignore_case {
                a.eq_ignore_ascii_case(b)
            } else {
                a == b
            }
        });
    }

    lines.join("\n")
}

/// Compare two strings numerically (leading numeric prefix)
fn compare_numeric(a: &str, b: &str, ignore_case: bool) -> std::cmp::Ordering {
    let na = extract_leading_number(a);
    let nb = extract_leading_number(b);

    match (na, nb) {
        (Some(va), Some(vb)) => va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal),
        _ => {
            if ignore_case {
                a.to_lowercase().cmp(&b.to_lowercase())
            } else {
                a.cmp(b)
            }
        }
    }
}

/// Extract leading numeric value from a string
fn extract_leading_number(s: &str) -> Option<f64> {
    let trimmed = s.trim_start();
    let num_str: String = trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    num_str.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_basic() {
        let text = "cherry\napple\nbanana";
        assert_eq!(sort_lines(text, false, false, false, false), "apple\nbanana\ncherry");
    }

    #[test]
    fn test_sort_reverse() {
        let text = "apple\nbanana\ncherry";
        assert_eq!(sort_lines(text, true, false, false, false), "cherry\nbanana\napple");
    }

    #[test]
    fn test_sort_numeric() {
        let text = "10\n2\n1\n20\n3";
        assert_eq!(sort_lines(text, false, true, false, false), "1\n2\n3\n10\n20");
    }

    #[test]
    fn test_sort_unique() {
        let text = "apple\nbanana\napple\ncherry\nbanana";
        assert_eq!(sort_lines(text, false, false, true, false), "apple\nbanana\ncherry");
    }

    #[test]
    fn test_sort_ignore_case() {
        let text = "Banana\napple\nCherry";
        assert_eq!(sort_lines(text, false, false, false, true), "apple\nBanana\nCherry");
    }

    #[test]
    fn test_extract_leading_number() {
        assert_eq!(extract_leading_number("42abc"), Some(42.0));
        assert_eq!(extract_leading_number("abc"), None);
        assert_eq!(extract_leading_number("3.14"), Some(3.14));
    }
}
