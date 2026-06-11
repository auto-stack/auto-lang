use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct UniqCommand;

impl Command for UniqCommand {
    fn name(&self) -> &str {
        "uniq"
    }

    fn signature(&self) -> Signature {
        Signature::new("uniq", "Report or omit repeated lines")
            .flag_with_short("count", 'c', "Prefix lines with occurrence count")
            .flag_with_short("repeated", 'd', "Only print duplicate lines")
            .flag_with_short("unique", 'u', "Only print unique lines")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = get_text(input)?;
        let show_count = args.has_flag("count");
        let only_dups = args.has_flag("repeated");
        let only_unique = args.has_flag("unique");

        let result = uniq_lines(&text, show_count, only_dups, only_unique);
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
        _ => miette::bail!("uniq: input must be text"),
    }
}

/// Process lines for uniq behavior
pub fn uniq_lines(
    text: &str,
    show_count: bool,
    only_dups: bool,
    only_unique: bool,
) -> String {
    if text.is_empty() {
        return String::new();
    }

    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Group adjacent identical lines
    let mut groups: Vec<(&str, usize)> = Vec::new();
    let mut current = lines[0];
    let mut count = 1;

    for &line in &lines[1..] {
        if line == current {
            count += 1;
        } else {
            groups.push((current, count));
            current = line;
            count = 1;
        }
    }
    groups.push((current, count));

    // Filter and format output
    let mut output = Vec::new();
    for (line, cnt) in groups {
        let is_dup = cnt > 1;
        if only_dups && !is_dup {
            continue;
        }
        if only_unique && is_dup {
            continue;
        }
        if show_count {
            output.push(format!("{:>7} {}", cnt, line));
        } else {
            output.push(line.to_string());
        }
    }

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniq_basic() {
        let text = "apple\napple\nbanana\ncherry\ncherry\ncherry";
        assert_eq!(uniq_lines(text, false, false, false), "apple\nbanana\ncherry");
    }

    #[test]
    fn test_uniq_count() {
        let text = "apple\napple\nbanana\ncherry\ncherry";
        let result = uniq_lines(text, true, false, false);
        assert!(result.contains("2 apple"));
        assert!(result.contains("1 banana"));
        assert!(result.contains("2 cherry"));
    }

    #[test]
    fn test_uniq_only_dups() {
        let text = "apple\napple\nbanana\ncherry\ncherry";
        assert_eq!(uniq_lines(text, false, true, false), "apple\ncherry");
    }

    #[test]
    fn test_uniq_only_unique() {
        let text = "apple\napple\nbanana\ncherry\ncherry";
        assert_eq!(uniq_lines(text, false, false, true), "banana");
    }

    #[test]
    fn test_uniq_empty() {
        assert_eq!(uniq_lines("", false, false, false), "");
    }

    #[test]
    fn test_uniq_all_same() {
        let text = "a\na\na";
        assert_eq!(uniq_lines(text, false, false, false), "a");
        assert_eq!(uniq_lines(text, false, false, true), "");
        assert_eq!(uniq_lines(text, false, true, false), "a");
    }
}
