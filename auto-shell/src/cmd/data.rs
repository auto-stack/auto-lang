//! Data manipulation commands
//!
//! Implements commands for filtering and transforming text data

use miette::Result;

/// Sort lines of input
pub fn sort_command(input: &str, reverse: bool, unique: bool) -> String {
    let mut lines: Vec<&str> = input.lines().collect();

    if unique {
        lines.sort();
        lines.dedup();
    } else {
        lines.sort();
    }

    if reverse {
        lines.reverse();
    }

    lines.join("\n")
}

/// Remove duplicate lines
pub fn uniq_command(input: &str, count: bool, _repeated: bool) -> String {
    let lines: Vec<&str> = input.lines().collect();

    if lines.is_empty() {
        return String::new();
    }

    if count {
        // Count consecutive duplicates
        let mut result = Vec::new();
        let mut current = lines[0];
        let mut count = 1;

        for line in &lines[1..] {
            if *line == current {
                count += 1;
            } else {
                result.push(format!("\t{} {}", count, current));
                current = *line;
                count = 1;
            }
        }
        result.push(format!("\t{} {}", count, current));

        result.join("\n")
    } else {
        // Remove consecutive duplicates
        let mut result = Vec::new();
        let mut last = "";

        for line in &lines {
            if *line != last {
                result.push(*line);
                last = *line;
            }
        }

        result.join("\n")
    }
}

/// Get first N lines
pub fn head_command(input: &str, lines: usize) -> String {
    input.lines()
        .take(lines)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Get last N lines
pub fn tail_command(input: &str, lines: usize) -> String {
    let all_lines: Vec<&str> = input.lines().collect();
    let start = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };

    all_lines[start..].join("\n")
}

/// Count lines, words, and bytes
pub fn wc_command(input: &str) -> String {
    let lines = input.lines().count();
    let words = input.split_whitespace().count();
    let bytes = input.len();

    format!("{} {} {}", lines, words, bytes)
}

/// Grep pattern matching (simple implementation)
pub fn grep_command(input: &str, pattern: &str, invert_match: bool) -> String {
    let mut result = Vec::new();

    for line in input.lines() {
        let matches = line.contains(pattern);
        if matches != invert_match {
            result.push(line);
        }
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_basic() {
        let input = "c\na\nb";
        let result = sort_command(input, false, false);
        assert_eq!(result, "a\nb\nc");
    }

    #[test]
    fn test_sort_reverse() {
        let input = "a\nb\nc";
        let result = sort_command(input, true, false);
        assert_eq!(result, "c\nb\na");
    }

    #[test]
    fn test_sort_unique() {
        let input = "a\na\nb\nb\nc";
        let result = sort_command(input, false, true);
        assert_eq!(result, "a\nb\nc");
    }

    #[test]
    fn test_uniq_basic() {
        let input = "a\na\nb\nc\nc";
        let result = uniq_command(input, false, false);
        assert_eq!(result, "a\nb\nc");
    }

    #[test]
    fn test_uniq_count() {
        let input = "a\na\nb\nc\nc";
        let result = uniq_command(input, true, false);
        assert!(result.contains("\t2 a"));
        assert!(result.contains("\t1 b"));
        assert!(result.contains("\t2 c"));
    }

    #[test]
    fn test_head() {
        let input = "a\nb\nc\nd\ne";
        let result = head_command(input, 3);
        assert_eq!(result, "a\nb\nc");
    }

    #[test]
    fn test_tail() {
        let input = "a\nb\nc\nd\ne";
        let result = tail_command(input, 2);
        assert_eq!(result, "d\ne");
    }

    #[test]
    fn test_wc() {
        let input = "hello world\nthis is a test";
        let result = wc_command(input);
        assert!(result.contains("2")); // 2 lines
        assert!(result.contains("6")); // 6 words (hello, world, this, is, a, test)
    }

    #[test]
    fn test_grep_basic() {
        let input = "hello\nworld\ntest\nhello world";
        let result = grep_command(input, "hello", false);
        assert_eq!(result, "hello\nhello world");
    }

    #[test]
    fn test_grep_invert() {
        let input = "hello\nworld\ntest";
        let result = grep_command(input, "hello", true);
        assert_eq!(result, "world\ntest");
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(head_command("", 10), "");
        assert_eq!(tail_command("", 10), "");
        assert_eq!(sort_command("", false, false), "");
        assert_eq!(uniq_command("", false, false), "");
    }
}
