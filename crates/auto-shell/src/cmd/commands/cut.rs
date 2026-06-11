use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

pub struct CutCommand;

impl Command for CutCommand {
    fn name(&self) -> &str {
        "cut"
    }

    fn signature(&self) -> Signature {
        Signature::new("cut", "Remove sections from each line of text")
            .optional("file", "File to read (default: stdin)")
            .flag_with_short("delimiter", 'd', "Field delimiter (default: TAB)")
            .flag_with_short("fields", 'f', "Field list (e.g., 1,3 or 1-3)")
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

        let delimiter = args.positional_or(1, "\t").to_string();
        // Check named flags for -f and -d
        let delim = if args.has_flag("delimiter") {
            args.positionals.get(1).map(|s| s.as_str()).unwrap_or("\t").to_string()
        } else {
            delimiter
        };

        let _fields_str = args.positionals.iter()
            .find(|s| s.contains(',') || s.contains('-') || s.chars().all(|c| c.is_ascii_digit()))
            .map(|s| s.as_str());

        // Try to get fields from named args or positional
        let fields_spec = if let Some(named_f) = args.positionals.get(1) {
            if named_f.contains(',') || named_f.contains('-') || named_f.chars().all(|c| c.is_ascii_digit()) {
                Some(named_f.clone())
            } else {
                None
            }
        } else {
            None
        };

        let spec = fields_spec.unwrap_or_else(|| "1".to_string());
        let field_indices = parse_field_list(&spec)?;

        let result = cut_fields(&text, &delim, &field_indices);
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
        _ => miette::bail!("cut: input must be text"),
    }
}

/// Parse a field specification like "1,3" or "1-3" or "1,3-5" into sorted 1-based indices
pub fn parse_field_list(spec: &str) -> Result<Vec<usize>> {
    let mut indices = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let bounds: Vec<&str> = part.split('-').collect();
            if bounds.len() != 2 {
                miette::bail!("cut: invalid field range: {}", part);
            }
            let start: usize = bounds[0].parse().into_diagnostic()?;
            let end: usize = bounds[1].parse().into_diagnostic()?;
            if start == 0 || end == 0 {
                miette::bail!("cut: field indices must be >= 1");
            }
            for i in start..=end {
                indices.push(i);
            }
        } else {
            let idx: usize = part.parse().into_diagnostic()?;
            if idx == 0 {
                miette::bail!("cut: field index must be >= 1");
            }
            indices.push(idx);
        }
    }
    indices.sort();
    indices.dedup();
    Ok(indices)
}

/// Extract specified fields from each line
pub fn cut_fields(text: &str, delimiter: &str, field_indices: &[usize]) -> String {
    text.lines()
        .map(|line| {
            let fields: Vec<&str> = line.split(delimiter).collect();
            field_indices
                .iter()
                .filter_map(|&idx| fields.get(idx - 1).copied())
                .collect::<Vec<&str>>()
                .join(delimiter)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_list_single() {
        assert_eq!(parse_field_list("1").unwrap(), vec![1]);
        assert_eq!(parse_field_list("3").unwrap(), vec![3]);
    }

    #[test]
    fn test_parse_field_list_comma() {
        assert_eq!(parse_field_list("1,3").unwrap(), vec![1, 3]);
    }

    #[test]
    fn test_parse_field_list_range() {
        assert_eq!(parse_field_list("1-3").unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_field_list_mixed() {
        assert_eq!(parse_field_list("1,3-5").unwrap(), vec![1, 3, 4, 5]);
    }

    #[test]
    fn test_parse_field_list_dedup() {
        assert_eq!(parse_field_list("1,1,2").unwrap(), vec![1, 2]);
    }

    #[test]
    fn test_parse_field_list_zero_fails() {
        assert!(parse_field_list("0").is_err());
    }

    #[test]
    fn test_cut_fields_basic() {
        let text = "one:two:three\nfour:five:six";
        assert_eq!(cut_fields(text, ":", &[1]), "one\nfour");
    }

    #[test]
    fn test_cut_fields_multiple() {
        let text = "a:b:c:d\ne:f:g:h";
        assert_eq!(cut_fields(text, ":", &[1, 3]), "a:c\ne:g");
    }

    #[test]
    fn test_cut_fields_range() {
        let text = "a:b:c:d\ne:f:g:h";
        assert_eq!(cut_fields(text, ":", &[1, 2, 3]), "a:b:c\ne:f:g");
    }

    #[test]
    fn test_cut_fields_tab() {
        let text = "one\ttwo\tthree";
        assert_eq!(cut_fields(text, "\t", &[2]), "two");
    }
}
