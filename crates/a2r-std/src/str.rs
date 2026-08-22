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

/// Find the index of a substring, returns -1 if not found.
/// Accepts `String` or `&str` for both args (the transpiled call site may pass
/// an owned `String` field, e.g. `self.buf`).
pub fn str_find<S: AsRef<str>, P: AsRef<str>>(s: S, pattern: P) -> i32 {
    s.as_ref()
        .find(pattern.as_ref())
        .map(|p| p as i32)
        .unwrap_or(-1)
}

/// Find the index of a substring starting from position, returns -1 if not found
pub fn str_find_from<S: AsRef<str>, P: AsRef<str>>(s: S, pattern: P, start: i32) -> i32 {
    let start = start.max(0) as usize;
    let s = s.as_ref();
    if start >= s.len() {
        return -1;
    }
    s[start..]
        .find(pattern.as_ref())
        .map(|p| (start + p) as i32)
        .unwrap_or(-1)
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

/// Check if a string contains a substring
pub fn str_contains<S: AsRef<str>>(s: S, needle: &str) -> bool {
    s.as_ref().contains(needle)
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

/// Decode a byte slice to a UTF-8 string (lossy). Mirrors Auto's
/// `str.from_bytes(bytes)`. Used by the transpiled client to read HTTP bodies.
/// Accepts `&[u8]` or `Vec<u8>` (the call site may receive either).
pub fn from_bytes(bytes: impl AsRef<[u8]>) -> String {
    String::from_utf8_lossy(bytes.as_ref()).into_owned()
}
