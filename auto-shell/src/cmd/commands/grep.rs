use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::{Value, Obj, Array};
use miette::{IntoDiagnostic, Result, miette};
use regex::Regex;
use std::path::Path;

pub struct GrepCommand;

impl Command for GrepCommand {
    fn name(&self) -> &str {
        "grep"
    }

    fn signature(&self) -> Signature {
        Signature::new("grep", "Search for patterns in text")
            .required("pattern", "Regular expression pattern to search for")
            .optional("path", "File or directory to search (default: stdin)")
            .flag_with_short("ignore-case", 'i', "Case insensitive search")
            .flag_with_short("invert-match", 'v', "Invert match (show non-matching lines)")
            .flag_with_short("count", 'c', "Only show count of matching lines")
            .flag_with_short("line-number", 'n', "Show line numbers")
            .flag_with_short("files-with-matches", 'l', "Only show filenames with matches")
            .flag_with_short("recursive", 'r', "Search recursively in directories")
            .flag("hidden", "Search hidden files (requires -r)")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        // Extract pattern
        let pattern = args.positionals.get(0)
            .map(|s| s.as_str())
            .ok_or_else(|| miette::miette!("grep: pattern argument required"))?;

        // Extract path if provided
        let path_arg = args.positionals.get(1).map(|s| s.as_str());

        // Extract flags
        let ignore_case = args.has_flag("ignore-case");
        let invert_match = args.has_flag("invert-match");
        let count_only = args.has_flag("count");
        let show_line_number = args.has_flag("line-number");
        let files_with_matches = args.has_flag("files-with-matches");
        let recursive = args.has_flag("recursive");
        let hidden = args.has_flag("hidden");

        // Create regex
        let re = if ignore_case {
            Regex::new(&format!("(?i){}", pattern))
                .into_diagnostic()?
        } else {
            Regex::new(pattern)
                .into_diagnostic()?
        };

        // Handle input based on whether path is provided
        if let Some(path_str) = path_arg {
            let path = Path::new(path_str);
            if path.is_dir() {
                // Search directory
                let results = if recursive {
                    search_directory_recursive(path, &re, invert_match, count_only, show_line_number, files_with_matches, hidden)?
                } else {
                    search_directory(path, &re, invert_match, count_only, show_line_number, files_with_matches)?
                };
                Ok(PipelineData::from_value(Value::Array(Array::from(results))))
            } else {
                // Search single file
                let content = std::fs::read_to_string(path).into_diagnostic()?;
                let results = search_text(&content, &re, invert_match, count_only, show_line_number, path.to_string_lossy().as_ref())?;
                Ok(PipelineData::from_value(Value::Array(Array::from(results))))
            }
        } else {
            // Search from pipeline input
            match input {
                PipelineData::Value(Value::Str(text)) => {
                    let results = search_text(text.as_ref(), &re, invert_match, count_only, show_line_number, "<stdin>")?;
                    Ok(PipelineData::from_value(Value::Array(Array::from(results))))
                }
                PipelineData::Text(text) => {
                    let results = search_text(&text, &re, invert_match, count_only, show_line_number, "<stdin>")?;
                    Ok(PipelineData::from_value(Value::Array(Array::from(results))))
                }
                PipelineData::Value(Value::Array(arr)) => {
                    // Search through array elements
                    let mut results = Vec::new();
                    for (index, item) in arr.iter().enumerate() {
                        if let Value::Str(text) = item {
                            let item_results = search_text(
                                text.as_ref(),
                                &re,
                                invert_match,
                                count_only,
                                show_line_number,
                                &format!("<stream[{}]>", index)
                            )?;
                            results.extend(item_results);
                        }
                    }
                    Ok(PipelineData::from_value(Value::Array(Array::from(results))))
                }
                _ => {
                    miette::bail!("grep: input must be text or array of texts");
                }
            }
        }
    }
}

/// Search in text string
fn search_text(
    text: &str,
    re: &Regex,
    invert_match: bool,
    count_only: bool,
    show_line_number: bool,
    path: &str,
) -> Result<Vec<Value>> {
    let mut results = Vec::new();
    let mut match_count = 0;

    for (line_num, line) in text.lines().enumerate() {
        let is_match = re.is_match(line);
        let should_include = if invert_match { !is_match } else { is_match };

        if should_include {
            if count_only {
                match_count += 1;
            } else {
                let mut obj = Obj::new();
                obj.set("file", Value::str(path));

                if show_line_number {
                    obj.set("line_number", Value::Int((line_num + 1) as i32));
                }

                obj.set("text", Value::str(line.trim()));
                results.push(Value::Obj(obj));
            }
        }
    }

    if count_only && match_count > 0 {
        let mut obj = Obj::new();
        obj.set("file", Value::str(path));
        obj.set("count", Value::Int(match_count as i32));
        results.push(Value::Obj(obj));
    }

    Ok(results)
}

/// Search directory (non-recursive)
fn search_directory(
    path: &Path,
    re: &Regex,
    invert_match: bool,
    count_only: bool,
    show_line_number: bool,
    files_with_matches: bool,
) -> Result<Vec<Value>> {
    let mut all_results = Vec::new();

    let entries = std::fs::read_dir(path).into_diagnostic()?;
    for entry in entries {
        let entry = entry.into_diagnostic()?;
        let entry_path = entry.path();

        // Skip hidden files/dirs unless requested
        if let Some(name) = entry_path.file_name() {
            if name.to_string_lossy().starts_with('.') {
                continue;
            }
        }

        if entry_path.is_file() {
            match std::fs::read_to_string(&entry_path) {
                Ok(content) => {
                    let path_str = entry_path.to_string_lossy().to_string();
                    let file_results = search_text(&content, re, invert_match, count_only, show_line_number, &path_str)?;

                    if files_with_matches && !file_results.is_empty() {
                        let mut obj = Obj::new();
                        obj.set("file", Value::str(&path_str));
                        all_results.push(Value::Obj(obj));
                    } else {
                        all_results.extend(file_results);
                    }
                }
                Err(_) => continue, // Skip files that can't be read
            }
        }
    }

    Ok(all_results)
}

/// Search directory recursively
fn search_directory_recursive(
    path: &Path,
    re: &Regex,
    invert_match: bool,
    count_only: bool,
    show_line_number: bool,
    files_with_matches: bool,
    hidden: bool,
) -> Result<Vec<Value>> {
    let mut all_results = Vec::new();

    let entries = std::fs::read_dir(path).into_diagnostic()?;
    for entry in entries {
        let entry = entry.into_diagnostic()?;
        let entry_path = entry.path();

        // Skip hidden files/dirs unless requested
        if !hidden {
            if let Some(name) = entry_path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }
        }

        if entry_path.is_dir() {
            let mut subdir_results = search_directory_recursive(
                &entry_path,
                re,
                invert_match,
                count_only,
                show_line_number,
                files_with_matches,
                hidden,
            )?;
            all_results.append(&mut subdir_results);
        } else if entry_path.is_file() {
            match std::fs::read_to_string(&entry_path) {
                Ok(content) => {
                    let path_str = entry_path.to_string_lossy().to_string();
                    let file_results = search_text(&content, re, invert_match, count_only, show_line_number, &path_str)?;

                    if files_with_matches && !file_results.is_empty() {
                        let mut obj = Obj::new();
                        obj.set("file", Value::str(&path_str));
                        all_results.push(Value::Obj(obj));
                    } else {
                        all_results.extend(file_results);
                    }
                }
                Err(_) => continue, // Skip files that can't be read
            }
        }
    }

    Ok(all_results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_text_basic() {
        let re = Regex::new("hello").unwrap();
        let results = search_text("hello world\nhello rust\nfoo bar", &re, false, false, false, "<test>").unwrap();
        assert_eq!(results.len(), 2);

        if let Value::Obj(obj) = &results[0] {
            assert_eq!(obj.get("text"), Some(&Value::str("hello world")).cloned());
        }
    }

    #[test]
    fn test_search_text_case_insensitive() {
        let re = Regex::new("(?i)hello").unwrap();
        let results = search_text("HELLO world\nhello rust\nfoo bar", &re, false, false, false, "<test>").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_text_invert() {
        let re = Regex::new("hello").unwrap();
        let results = search_text("hello world\nfoo bar\nhello rust", &re, true, false, false, "<test>").unwrap();
        assert_eq!(results.len(), 1);

        if let Value::Obj(obj) = &results[0] {
            assert_eq!(obj.get("text"), Some(&Value::str("foo bar")).cloned());
        }
    }

    #[test]
    fn test_search_text_count() {
        let re = Regex::new("hello").unwrap();
        let results = search_text("hello world\nhello rust\nfoo bar", &re, false, true, false, "<test>").unwrap();
        assert_eq!(results.len(), 1);

        if let Value::Obj(obj) = &results[0] {
            assert_eq!(obj.get("count"), Some(&Value::Int(2)).cloned());
        }
    }

    #[test]
    fn test_search_text_line_numbers() {
        let re = Regex::new("hello").unwrap();
        let results = search_text("foo\nhello world\nbar\nhello rust", &re, false, false, true, "<test>").unwrap();
        assert_eq!(results.len(), 2);

        if let Value::Obj(obj) = &results[0] {
            assert_eq!(obj.get("line_number"), Some(&Value::Int(2)).cloned());
        }
    }
}
