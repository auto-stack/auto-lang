/// String utility functions
/// Transpiled from auto-lang/stdlib/auto/str.at + str.rs.at

/// UTF-8 byte length. PLAN-781 阶段 E（T-24）：VM 轨 nat#1526
/// （`shim_str_byte_len`）的 a2r 镜像——与 `fs::read_text_range` 的字节
/// 游标同空间（Reader 章 meta 索引偏移累计）。
pub fn str_byte_len(s: &str) -> i64 {
    s.len() as i64
}

/// Get the character at a given index (returns code point as i32)
/// Returns 0 if index is out of bounds
pub fn char_at(s: &str, i: i32) -> i32 {
    match s.chars().nth(i as usize) {
        Some(c) => c as i32,
        None => 0,
    }
}

/// Count non-overlapping occurrences of a substring
pub fn match_count(s: &str, pattern: &str) -> i64 {
    s.matches(pattern).count() as i64
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
/// PLAN-885 G3/G2: returns `i64` — `.at int` maps to Rust `i64` (ty.rs Plan 399
/// Phase 11.1); the historical `i32` return caused E0308 on generated
/// backends (`let hit: i64 = str_find(...)`).
/// PLAN-956: byte offsets replace the former Rust-only character offsets.
/// UTF-8 byte offset, paired with `str_substr` and `str_slice` on both backends.
pub fn str_find<S: AsRef<str>, P: AsRef<str>>(s: S, pattern: P) -> i64 {
    s.as_ref()
        .find(pattern.as_ref())
        .map(|i| i as i64)
        .unwrap_or(-1)
}

/// Byte offset search. A start inside a UTF-8 character is clamped forward.
pub fn str_find_from<S: AsRef<str>, P: AsRef<str>, I: TryInto<usize>>(
    s: S,
    pattern: P,
    start: I,
) -> i64 {
    let s = s.as_ref();
    let mut start = start.try_into().unwrap_or(0);
    if start > s.len() {
        return -1;
    }
    while start < s.len() && !s.is_char_boundary(start) {
        start += 1;
    }
    s[start..]
        .find(pattern.as_ref())
        .map(|i| (start + i) as i64)
        .unwrap_or(-1)
}

/// END-semantics slicing with the same byte offsets and UTF-8 clamps as VM.
pub fn str_slice<S: AsRef<str>, I: TryInto<usize>, N: TryInto<usize>>(
    s: S,
    start: I,
    end: N,
) -> String {
    let s = s.as_ref();
    let (Ok(start), Ok(end)) = (start.try_into(), end.try_into()) else {
        return String::new();
    };
    if start > end || end > s.len() {
        return String::new();
    }
    str_substr(s, start, end - start)
}
/// Extract a substring from start index with given length
pub fn str_substr<I: TryInto<usize>, N: TryInto<usize>>(s: &str, start: I, len: N) -> String {
    let (Ok(mut start), Ok(len)) = (start.try_into(), len.try_into()) else {
        return String::new();
    };
    if start > s.len() {
        return String::new();
    }
    let mut end = start.saturating_add(len).min(s.len());
    // VM substr uses byte offsets and clamps partial UTF-8 characters inward.
    while start < s.len() && !s.is_char_boundary(start) {
        start += 1;
    }
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    if end < start {
        return String::new();
    }
    s[start..end].to_string()
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

#[cfg(test)]
mod byte_index_tests {
    use super::*;

    #[test]
    fn chinese_emoji_find_slice_and_substr_share_byte_offsets() {
        let text = "山😀月abc";
        assert_eq!(str_find(text, "月"), 7);
        assert_eq!(str_substr(text, 7_i64, 3_i64), "月");
        assert_eq!(str_slice(text, 3_i64, 10_i64), "😀月");
        assert_eq!(str_find_from(text, "月", 4_i64), 7);
        assert_eq!(str_find_from(text, "😀", 4_i64), -1);
    }

    #[test]
    fn partial_characters_negative_and_overflow_do_not_panic() {
        let text = "山😀月";
        assert_eq!(str_substr(text, 1_i64, 8_i64), "😀");
        assert_eq!(str_substr(text, -1_i64, 3_i64), "");
        assert_eq!(str_substr(text, 3_usize, usize::MAX), "😀月");
        assert_eq!(str_slice(text, 0_i64, 99_i64), "");
        assert_eq!(str_slice(text, 7_i64, 3_i64), "");
        assert_eq!(str_find_from(text, "山", -1_i64), 0);
        assert_eq!(str_find_from(text, "", 99_i64), -1);
    }
}
