use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct StrCaseCommand;

impl Command for StrCaseCommand {
    fn name(&self) -> &str {
        "str-case"
    }

    fn signature(&self) -> Signature {
        Signature::new("str-case", "Change string case")
            .required("operation", "Operation: upper, lower, capitalize, title, camel, snake, kebab")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let op = args.first().unwrap_or("");
        let text = extract_text(&input)?;

        let result = match op {
            "upper" => text.to_uppercase(),
            "lower" => text.to_lowercase(),
            "capitalize" => capitalize(&text),
            "title" => title_case(&text),
            "camel" => to_camel_case(&text),
            "snake" => to_snake_case(&text),
            "kebab" => to_kebab_case(&text),
            _ => miette::bail!(
                "str-case: unknown operation '{}'. Use: upper, lower, capitalize, title, camel, snake, kebab",
                op
            ),
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

fn extract_text(input: &PipelineData) -> Result<String> {
    match input {
        PipelineData::Text(s) => Ok(s.clone()),
        PipelineData::Value(Value::Str(s)) => Ok(s.to_string()),
        _ => miette::bail!("str-case: input must be text"),
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + &chars.as_str().to_lowercase()
        }
    }
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| capitalize(word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert snake/kebab/space-separated to camelCase
fn to_camel_case(s: &str) -> String {
    let words = split_into_words(s);
    let mut result = String::new();
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            result.push_str(&word.to_lowercase());
        } else {
            result.push_str(&capitalize(word));
        }
    }
    result
}

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
    let words = split_into_words(s);
    words.join("_").to_lowercase()
}

/// Convert to kebab-case
fn to_kebab_case(s: &str) -> String {
    let words = split_into_words(s);
    words.join("-").to_lowercase()
}

/// Split string into words by separators (_, -, spaces, or camelCase boundaries)
fn split_into_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
        } else if ch.is_uppercase() {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            current.push(ch.to_lowercase().next().unwrap_or(ch));
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("HELLO"), "Hello");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("hello world"), "Hello World");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello world"), "helloWorld");
        assert_eq!(to_camel_case("hello-world"), "helloWorld");
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("helloWorld"), "helloWorld");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("hello-world"), "hello_world");
        assert_eq!(to_snake_case("Hello World"), "hello_world");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("helloWorld"), "hello-world");
        assert_eq!(to_kebab_case("hello_world"), "hello-world");
        assert_eq!(to_kebab_case("Hello World"), "hello-world");
    }

    #[test]
    fn test_split_into_words() {
        assert_eq!(split_into_words("helloWorld"), vec!["hello", "world"]);
        assert_eq!(split_into_words("hello_world"), vec!["hello", "world"]);
        assert_eq!(split_into_words("hello-world"), vec!["hello", "world"]);
        assert_eq!(split_into_words("Hello World"), vec!["hello", "world"]);
    }
}
