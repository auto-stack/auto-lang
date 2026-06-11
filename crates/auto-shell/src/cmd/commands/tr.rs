use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::{Atom, AtomPipeline, AtomType};
use auto_val::Value;
use miette::Result;

pub struct TrCommand;

impl Command for TrCommand {
    fn name(&self) -> &str {
        "tr"
    }

    fn signature(&self) -> Signature {
        Signature::new("tr", "Translate, squeeze, or delete characters")
            .required("set1", "Characters to translate or delete")
            .optional("set2", "Characters to translate to")
            .flag_with_short("delete", 'd', "Delete characters in set1")
            .flag_with_short("squeeze-repeats", 's', "Squeeze repeated characters")
            .flag_with_short("complement", 'c', "Use complement of set1")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let text = get_text(input)?;

        let set1 = args.first().unwrap_or("");
        let set2 = args.second();

        let delete = args.has_flag("delete");
        let squeeze = args.has_flag("squeeze-repeats");
        let complement = args.has_flag("complement");

        let result = tr_translate(&text, set1, set2, delete, squeeze, complement);
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
        _ => miette::bail!("tr: input must be text"),
    }
}

/// Translate, squeeze, or delete characters
pub fn tr_translate(
    text: &str,
    set1: &str,
    set2: Option<&str>,
    delete: bool,
    squeeze: bool,
    complement: bool,
) -> String {
    let set1_chars: Vec<char> = set1.chars().collect();
    let set2_chars: Vec<char> = set2
        .map(|s| s.chars().collect::<Vec<char>>())
        .unwrap_or_default();

    /// Look up translation for a character at given index in set1.
    /// If idx >= set2 len, reuse the last char of set2 (or no translation if set2 is empty).
    fn translate_char(_c: char, idx: usize, set2: &[char]) -> Option<char> {
        if set2.is_empty() {
            return None;
        }
        if idx < set2.len() {
            Some(set2[idx])
        } else {
            Some(*set2.last().unwrap())
        }
    }

    if delete {
        let filtered: String = text
            .chars()
            .filter(|&c| {
                let in_set = set1_chars.contains(&c);
                if complement { in_set } else { !in_set }
            })
            .collect();
        return if squeeze {
            squeeze_chars(&filtered, &set1_chars, complement)
        } else {
            filtered
        };
    }

    // Translation mode
    let translated: String = text
        .chars()
        .map(|c| {
            let in_set = set1_chars.contains(&c);
            if complement {
                if in_set {
                    c
                } else {
                    translate_char(c, 0, &set2_chars).unwrap_or(c)
                }
            } else if in_set {
                let idx = set1_chars.iter().position(|&x| x == c).unwrap();
                translate_char(c, idx, &set2_chars).unwrap_or(c)
            } else {
                c
            }
        })
        .collect();

    if squeeze {
        squeeze_chars(&translated, &set2_chars, false)
    } else {
        translated
    }
}

/// Squeeze consecutive repeated characters from a set
fn squeeze_chars(text: &str, squeeze_set: &[char], complement: bool) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_squeeze: Option<char> = None;

    for c in text.chars() {
        let in_squeeze_set = if complement {
            !squeeze_set.contains(&c)
        } else {
            squeeze_set.contains(&c)
        };

        if in_squeeze_set {
            if prev_squeeze != Some(c) {
                result.push(c);
                prev_squeeze = Some(c);
            }
            // else: skip repeated squeeze char
        } else {
            result.push(c);
            prev_squeeze = None; // reset squeeze tracking for non-squeeze chars
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tr_basic_translate() {
        assert_eq!(tr_translate("hello", "el", Some("ip"), false, false, false), "hippo");
    }

    #[test]
    fn test_tr_uppercase() {
        assert_eq!(
            tr_translate("hello world", "abcdefghijklmnopqrstuvwxyz", Some("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), false, false, false),
            "HELLO WORLD"
        );
    }

    #[test]
    fn test_tr_delete() {
        assert_eq!(tr_translate("hello world", "l", None, true, false, false), "heo word");
    }

    #[test]
    fn test_tr_delete_multiple() {
        assert_eq!(tr_translate("hello world", "ol", None, true, false, false), "he wrd");
    }

    #[test]
    fn test_tr_squeeze() {
        // Squeeze only 'e' and 'o'; 'l' is NOT in the squeeze set
        assert_eq!(tr_translate("heeellloooo", "eo", Some("eo"), false, true, false), "helllo");
    }

    #[test]
    fn test_tr_squeeze_spaces() {
        assert_eq!(tr_translate("hello   world", " ", Some(" "), false, true, false), "hello world");
    }

    #[test]
    fn test_tr_complement_delete() {
        // Delete everything NOT in set (keep only digits)
        assert_eq!(
            tr_translate("abc123def456", "0123456789", None, true, false, true),
            "123456"
        );
    }

    #[test]
    fn test_tr_no_change() {
        assert_eq!(tr_translate("hello", "x", Some("y"), false, false, false), "hello");
    }
}
