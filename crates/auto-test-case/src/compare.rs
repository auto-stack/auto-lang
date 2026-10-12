//! Oracle comparison: strict, per-observation, no global normalization.
//!
//! Default is raw bytes; `lf` converts CRLF→LF and nothing else; the VM
//! result channel compares legacy-display strings exactly (no trim). An
//! undeclared observation is NOT checked — `text: ""` checks for emptiness;
//! the two are never conflated. Failures carry a first-divergence offset and
//! escaped snippets so reports point at the byte that differs.

use crate::contract::{Compare, MessageMatcher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    /// Observation label, e.g. "stdout", "result", "artifact main.rs".
    pub observation: String,
    pub kind: &'static str, // "bytes" | "lf" | "text" | "message"
    pub expected: String,
    pub actual: String,
    /// First differing byte offset (bytes/lf kinds).
    pub first_diff: Option<usize>,
}

impl std::fmt::Display for Mismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.first_diff {
            Some(off) => write!(
                f,
                "{}[{}] mismatch at byte {}: expected {:?}, actual {:?}",
                self.observation, self.kind, off, self.expected, self.actual
            ),
            None => write!(
                f,
                "{}[{}] mismatch: expected {:?}, actual {:?}",
                self.observation, self.kind, self.expected, self.actual
            ),
        }
    }
}

fn escape(bytes: &[u8]) -> String {
    String::from_utf8_lossy(&bytes[..bytes.len().min(120)])
        .escape_debug()
        .to_string()
}

fn first_diff(expected: &[u8], actual: &[u8]) -> (Option<usize>, String, String) {
    let off = expected
        .iter()
        .zip(actual.iter())
        .position(|(e, a)| e != a)
        .unwrap_or(expected.len().min(actual.len()));
    (Some(off), escape(expected), escape(actual))
}

/// Compare raw bytes under a mode; `None` = equal.
pub fn compare_bytes(
    expected: &[u8],
    actual: &[u8],
    mode: Compare,
    observation: &str,
) -> Option<Mismatch> {
    let equal = match mode {
        Compare::Bytes => expected == actual,
        Compare::Lf => to_lf(expected) == to_lf(actual),
    };
    if equal {
        return None;
    }
    let (off, e, a) = first_diff(expected, actual);
    Some(Mismatch {
        observation: observation.to_string(),
        kind: match mode {
            Compare::Bytes => "bytes",
            Compare::Lf => "lf",
        },
        expected: e,
        actual: a,
        first_diff: off,
    })
}

/// Exact string comparison (legacy-display): never trimmed.
pub fn compare_text_exact(expected: &str, actual: &str, observation: &str) -> Option<Mismatch> {
    if expected == actual {
        return None;
    }
    Some(Mismatch {
        observation: observation.to_string(),
        kind: "text",
        expected: escape(expected.as_bytes()),
        actual: escape(actual.as_bytes()),
        first_diff: expected
            .chars()
            .zip(actual.chars())
            .position(|(e, a)| e != a)
            .or_else(|| Some(expected.len().min(actual.len()))),
    })
}

/// Message identification (exact / explicit substring).
pub fn match_message(matcher: &MessageMatcher, actual: &str) -> Result<(), Mismatch> {
    let ok = match matcher {
        MessageMatcher::Exact(want) => actual == want,
        MessageMatcher::Contains(want) => actual.contains(want.as_str()),
    };
    if ok {
        Ok(())
    } else {
        Err(Mismatch {
            observation: "diagnostic.message".into(),
            kind: "message",
            expected: format!("{:?}", matcher),
            actual: escape(actual.as_bytes()),
            first_diff: None,
        })
    }
}

/// CRLF → LF only; every other byte (including lone CR) untouched.
pub fn to_lf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            out.push(b'\n');
            i += 2;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_mode_is_raw() {
        assert!(compare_bytes(b"a\r\nb", b"a\r\nb", Compare::Bytes, "x").is_none());
        let m = compare_bytes(b"a\r\nb", b"a\nb", Compare::Bytes, "x").unwrap();
        assert_eq!(m.first_diff, Some(1));
        assert!(m.to_string().contains("byte 1"));
    }

    #[test]
    fn lf_mode_converts_only_crlf() {
        assert!(compare_bytes(b"a\r\nb", b"a\nb", Compare::Lf, "x").is_none());
        assert!(
            compare_bytes(b"a\rb", b"a\nb", Compare::Lf, "x").is_some(),
            "lone CR must NOT be normalized away"
        );
    }

    #[test]
    fn text_exact_never_trims() {
        assert!(compare_text_exact("x\n", "x\n", "r").is_none());
        let m = compare_text_exact("x", "x\n", "r").unwrap();
        assert_eq!(m.kind, "text");
    }

    #[test]
    fn message_matchers() {
        assert!(match_message(
            &MessageMatcher::Exact("DivisionByZero".into()),
            "DivisionByZero"
        )
        .is_ok());
        assert!(match_message(
            &MessageMatcher::Contains("DivisionByZero".into()),
            "DivisionByZero at line 3"
        )
        .is_ok());
        assert!(match_message(
            &MessageMatcher::Contains("overflow".into()),
            "DivisionByZero"
        )
        .is_err());
    }
}
