use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct StrReplaceCommand;

impl Command for StrReplaceCommand {
    fn name(&self) -> &str {
        "str-replace"
    }

    fn signature(&self) -> Signature {
        Signature::new("str-replace", "Replace text in pipeline input")
            .required("pattern", "Pattern to find")
            .required("replacement", "Text to replace with")
            .flag("all", "Replace all occurrences (default)")
            .flag("first", "Replace only the first occurrence")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let pattern = args.first().unwrap_or("");
        let replacement = args.second().unwrap_or("");
        let replace_all = !args.has_flag("first");

        let text = match &input {
            PipelineData::Text(s) => s.clone(),
            PipelineData::Value(Value::Str(s)) => s.to_string(),
            _ => miette::bail!("str-replace: input must be text"),
        };

        let result = if replace_all {
            text.replace(pattern, replacement)
        } else {
            replacen(&text, pattern, replacement, 1)
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

/// Replace first N occurrences (std .replace replaces all, we need this for --first)
fn replacen(text: &str, pattern: &str, replacement: &str, max: usize) -> String {
    if pattern.is_empty() {
        return text.to_string();
    }
    let mut result = String::with_capacity(text.len());
    let mut count = 0;
    let mut last_end = 0;
    for (start, _) in text.match_indices(pattern) {
        if count >= max {
            break;
        }
        result.push_str(&text[last_end..start]);
        result.push_str(replacement);
        last_end = start + pattern.len();
        count += 1;
    }
    result.push_str(&text[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replacen_first() {
        assert_eq!(replacen("aaa", "a", "b", 1), "baa");
        assert_eq!(replacen("aaa", "a", "b", 2), "bba");
        assert_eq!(replacen("aaa", "a", "b", 10), "bbb");
    }

    #[test]
    fn test_replacen_empty_pattern() {
        assert_eq!(replacen("hello", "", "x", 1), "hello");
    }

    #[test]
    fn test_replacen_no_match() {
        assert_eq!(replacen("hello", "x", "y", 1), "hello");
    }
}
