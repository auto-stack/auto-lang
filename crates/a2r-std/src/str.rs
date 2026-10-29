/// String utility functions
/// Transpiled from auto-lang/stdlib/auto/str.at + str.rs.at

/// UTF-8 byte length. PLAN-781 阶段 E（T-24）：VM 轨 nat#1526
/// （`shim_str_byte_len`）的 a2r 镜像——与 `fs::read_text_range` 的字节
/// 游标同空间（Reader 章 meta 索引偏移累计）。
pub fn str_byte_len(s: &str) -> i32 {
    s.len() as i32
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
/// PLAN-885 G3/G2: returns `i64` — `.at int` maps to Rust `i64` (ty.rs Plan 399
/// Phase 11.1); the historical `i32` return caused E0308 on generated
/// backends (`let hit: i64 = str_find(...)`).
/// PLAN-885 G2: returns a **char** offset (not byte) — the a2r emitter renders
/// `.slice(j, j+1)` as `chars().take(j+1).skip(j)` (char semantics), so a byte
/// offset from `str::find` desynchronizes on multi-byte UTF-8 (list/JSON
/// field extraction read wrong characters). The VM pairs byte `find` with byte
/// `slice`; the transpiled side pairs char `find` with char `slice`.
pub fn str_find<S: AsRef<str>, P: AsRef<str>>(s: S, pattern: P) -> i64 {
    let s = s.as_ref();
    let p = pattern.as_ref();
    match s.find(p) {
        Some(byte_idx) => s[..byte_idx].chars().count() as i64,
        None => -1,
    }
}

/// Find the index of a substring starting from position, returns -1 if not found.
/// `start` is a char offset (matching the char-slice semantics above).
pub fn str_find_from<S: AsRef<str>, P: AsRef<str>>(s: S, pattern: P, start: i32) -> i64 {
    let start = start.max(0) as usize;
    let s = s.as_ref();
    let p = pattern.as_ref();
    let byte_start = s
        .char_indices()
        .nth(start)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    match s[byte_start..].find(p) {
        Some(byte_idx) => {
            start as i64 + s[byte_start..][..byte_idx].chars().count() as i64
        }
        None => -1,
    }
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
