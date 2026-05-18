/// String utility functions
/// Transpiled from auto-lang/stdlib/auto/str.at + str.rs.at

/// Get the character at a given index (returns code point as i32)
/// Returns 0 if index is out of bounds
pub fn char_at(s: &str, i: i32) -> i32 {
    match s.chars().nth(i as usize) {
        Some(c) => c as i32,
        None => 0,
    }
}

/// Count non-overlapping occurrences of a substring
pub fn match_count(s: &str, pattern: &str) -> i32 {
    s.matches(pattern).count() as i32
}

/// Replace first occurrence of a pattern in a string
pub fn replace_first(s: &str, from: &str, to: &str) -> String {
    if let Some(pos) = s.find(from) {
        let mut result = String::with_capacity(s.len() - from.len() + to.len());
        result.push_str(&s[..pos]);
        result.push_str(to);
        result.push_str(&s[pos + from.len()..]);
        result
    } else {
        s.to_string()
    }
}

/// Find the index of a substring, returns -1 if not found
pub fn str_find(s: &str, pattern: &str) -> i32 {
    s.find(pattern).map(|p| p as i32).unwrap_or(-1)
}

/// Extract a substring from start index with given length
pub fn str_substr(s: &str, start: i32, len: i32) -> String {
    let start = start.max(0) as usize;
    let len = len.max(0) as usize;
    s.chars().skip(start).take(len).collect()
}

/// Check if a string ends with a suffix
pub fn str_ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// Check if a string starts with a prefix
pub fn str_starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Split a string by a delimiter
pub fn str_split(s: &str, delimiter: &str) -> Vec<String> {
    s.split(delimiter).map(|p| p.to_string()).collect()
}

/// Trim whitespace from both ends
pub fn str_trim(s: &str) -> String {
    s.trim().to_string()
}

/// Convert string to lowercase
pub fn str_to_lower(s: &str) -> String {
    s.to_lowercase()
}

/// Convert string to uppercase
pub fn str_to_upper(s: &str) -> String {
    s.to_uppercase()
}
