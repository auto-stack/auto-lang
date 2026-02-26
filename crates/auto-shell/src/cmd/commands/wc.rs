use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::{Value, Obj, Array};
use miette::Result;

pub struct WcCommand;

impl Command for WcCommand {
    fn name(&self) -> &str {
        "wc"
    }

    fn signature(&self) -> Signature {
        Signature::new("wc", "Count lines, words, bytes, and characters")
            .flag_with_short("lines", 'l', "Count lines")
            .flag_with_short("words", 'w', "Count words")
            .flag_with_short("bytes", 'c', "Count bytes")
            .flag_with_short("chars", 'm', "Count characters (Unicode-aware)")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        // Extract flags
        let count_lines = args.has_flag("lines");
        let count_words = args.has_flag("words");
        let count_bytes = args.has_flag("bytes");
        let count_chars = args.has_flag("chars");

        // If no flags specified, count all
        let count_all = !count_lines && !count_words && !count_bytes && !count_chars;

        match input {
            PipelineData::Value(Value::Array(arr)) => {
                // For counting lines (-l), or counting all with non-text array, count array elements
                if count_lines && !count_words && !count_bytes && !count_chars {
                    // Special case: just counting elements (like "ls | wc -l")
                    let mut obj = Obj::new();
                    obj.set("lines", Value::Int(arr.len() as i32));
                    Ok(PipelineData::from_value(Value::Obj(obj)))
                } else if count_all {
                    // When counting all and array contains objects (like ls output),
                    // just count the array elements
                    let mut obj = Obj::new();
                    obj.set("lines", Value::Int(arr.len() as i32));
                    Ok(PipelineData::from_value(Value::Obj(obj)))
                } else {
                    // Count text content in each element
                    let mut results = Vec::new();

                    for (index, item) in arr.iter().enumerate() {
                        if let Some(text) = extract_text(item) {
                            let mut obj = Obj::new();

                            if count_lines || count_all {
                                let lines = count_lines_in_text(&text);
                                obj.set("lines", Value::Int(lines as i32));
                            }

                            if count_words || count_all {
                                let words = count_words_in_text(&text);
                                obj.set("words", Value::Int(words as i32));
                            }

                            if count_bytes || count_all {
                                let bytes = text.len();
                                obj.set("bytes", Value::Int(bytes as i32));
                            }

                            if count_chars || count_all {
                                let chars = text.chars().count();
                                obj.set("chars", Value::Int(chars as i32));
                            }

                            // Add index or filename if available
                            if let Value::Str(filename) = item {
                                obj.set("file", Value::str(filename.as_ref()));
                            } else {
                                obj.set("index", Value::Int(index as i32));
                            }

                            results.push(Value::Obj(obj));
                        }
                    }

                    // Add total if counting multiple items
                    if results.len() > 1 {
                        let mut total = Obj::new();
                        let mut total_lines = 0;
                        let mut total_words = 0;
                        let mut total_bytes = 0;
                        let mut total_chars = 0;

                        for result in &results {
                            if let Value::Obj(obj) = result {
                                if let Some(Value::Int(n)) = obj.get("lines") {
                                    total_lines += n;
                                }
                                if let Some(Value::Int(n)) = obj.get("words") {
                                    total_words += n;
                                }
                                if let Some(Value::Int(n)) = obj.get("bytes") {
                                    total_bytes += n;
                                }
                                if let Some(Value::Int(n)) = obj.get("chars") {
                                    total_chars += n;
                                }
                            }
                        }

                        if count_lines || count_all {
                            total.set("lines", Value::Int(total_lines));
                        }
                        if count_words || count_all {
                            total.set("words", Value::Int(total_words));
                        }
                        if count_bytes || count_all {
                            total.set("bytes", Value::Int(total_bytes));
                        }
                        if count_chars || count_all {
                            total.set("chars", Value::Int(total_chars));
                        }
                        total.set("file", Value::str("total"));

                        results.push(Value::Obj(total));
                    }

                    if results.is_empty() {
                        // If no text elements found, just return element count
                        let mut obj = Obj::new();
                        obj.set("lines", Value::Int(arr.len() as i32));
                        Ok(PipelineData::from_value(Value::Obj(obj)))
                    } else {
                        Ok(PipelineData::from_value(Value::Array(Array::from(results))))
                    }
                }
            }
            PipelineData::Value(Value::Str(text)) => {
                // Count single string
                let mut obj = Obj::new();

                if count_lines || count_all {
                    let lines = count_lines_in_text(text.as_ref());
                    obj.set("lines", Value::Int(lines as i32));
                }

                if count_words || count_all {
                    let words = count_words_in_text(text.as_ref());
                    obj.set("words", Value::Int(words as i32));
                }

                if count_bytes || count_all {
                    let bytes = text.len();
                    obj.set("bytes", Value::Int(bytes as i32));
                }

                if count_chars || count_all {
                    let chars = text.as_ref().chars().count();
                    obj.set("chars", Value::Int(chars as i32));
                }

                Ok(PipelineData::from_value(Value::Obj(obj)))
            }
            PipelineData::Value(Value::Obj(obj)) => {
                // If it's an object with a "content" field, count that
                if let Some(Value::Str(content)) = obj.get("content") {
                    let mut result = Obj::new();

                    if count_lines || count_all {
                        let lines = count_lines_in_text(content.as_ref());
                        result.set("lines", Value::Int(lines as i32));
                    }

                    if count_words || count_all {
                        let words = count_words_in_text(content.as_ref());
                        result.set("words", Value::Int(words as i32));
                    }

                    if count_bytes || count_all {
                        let bytes = content.len();
                        result.set("bytes", Value::Int(bytes as i32));
                    }

                    if count_chars || count_all {
                        let chars = content.as_ref().chars().count();
                        result.set("chars", Value::Int(chars as i32));
                    }

                    // Copy filename if present
                    if let Some(filename) = obj.get("name") {
                        result.set("file", filename.clone());
                    }

                    Ok(PipelineData::from_value(Value::Obj(result)))
                } else {
                    miette::bail!("wc: cannot count non-text object");
                }
            }
            PipelineData::Text(text) => {
                // Count plain text input
                let mut obj = Obj::new();

                if count_lines || count_all {
                    let lines = count_lines_in_text(&text);
                    obj.set("lines", Value::Int(lines as i32));
                }

                if count_words || count_all {
                    let words = count_words_in_text(&text);
                    obj.set("words", Value::Int(words as i32));
                }

                if count_bytes || count_all {
                    let bytes = text.len();
                    obj.set("bytes", Value::Int(bytes as i32));
                }

                if count_chars || count_all {
                    let chars = text.chars().count();
                    obj.set("chars", Value::Int(chars as i32));
                }

                Ok(PipelineData::from_value(Value::Obj(obj)))
            }
            PipelineData::Value(_) => {
                miette::bail!("wc: input must be text, string, or array of texts");
            }
        }
    }
}

/// Extract text content from a Value
fn extract_text(value: &Value) -> Option<String> {
    match value {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    }
}

/// Count lines in text (number of newlines + 1 if non-empty)
fn count_lines_in_text(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.lines().count()
    }
}

/// Count words in text (whitespace-separated sequences)
fn count_words_in_text(text: &str) -> usize {
    text.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_lines() {
        let text = "line1\nline2\nline3";
        assert_eq!(count_lines_in_text(text), 3);

        let text = "";
        assert_eq!(count_lines_in_text(text), 0);

        let text = "single line";
        assert_eq!(count_lines_in_text(text), 1);
    }

    #[test]
    fn test_count_words() {
        let text = "hello world test";
        assert_eq!(count_words_in_text(text), 3);

        let text = "  extra  spaces  ";
        assert_eq!(count_words_in_text(text), 2);

        let text = "";
        assert_eq!(count_words_in_text(text), 0);
    }

    #[test]
    fn test_extract_text() {
        let value = Value::str("hello world");
        assert_eq!(extract_text(&value), Some("hello world".to_string()));

        let value = Value::Int(42);
        assert_eq!(extract_text(&value), None);
    }

    #[test]
    fn test_wc_counts_array_elements() {
        // Test that wc -l counts array elements (like ls output)
        let wc = WcCommand;
        let mut flags = std::collections::HashMap::new();
        flags.insert("lines".to_string(), true);

        let args = crate::cmd::parser::ParsedArgs {
            positionals: vec![],
            flags,
            named: std::collections::HashMap::new(),
        };

        // Create an array like ls would return
        let mut arr = Array::new();
        let mut obj1 = Obj::new();
        obj1.set("name", Value::str("file1.txt"));
        let mut obj2 = Obj::new();
        obj2.set("name", Value::str("file2.txt"));
        let mut obj3 = Obj::new();
        obj3.set("name", Value::str("file3.txt"));

        arr.push(Value::Obj(obj1));
        arr.push(Value::Obj(obj2));
        arr.push(Value::Obj(obj3));

        let input = PipelineData::from_value(Value::Array(arr));
        let result = wc.run(&args, input, &mut Shell::new()).unwrap();

        if let PipelineData::Value(Value::Obj(obj)) = result {
            assert_eq!(obj.get("lines"), Some(Value::Int(3)));
        } else {
            panic!("Expected Value::Obj with lines count");
        }
    }
}
