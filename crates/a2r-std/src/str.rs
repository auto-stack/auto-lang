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
    if pattern.is_empty() { return 0; }
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
