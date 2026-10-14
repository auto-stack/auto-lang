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

/// Single-hunk unified diff of the changed region (Plan 773 F-11): common
/// leading/trailing lines are trimmed and the changed middle is rendered as
/// a unified hunk with `context` surrounding lines. Line numbers are
/// 1-based. `None` when the inputs are equal. This is a REAL content diff
/// for baseline proposals — not just a first-divergence offset.
pub fn unified_diff_hunk(expected: &[u8], actual: &[u8], context: usize) -> Option<String> {
    let e_text = String::from_utf8_lossy(expected);
    let a_text = String::from_utf8_lossy(actual);
    let e: Vec<&str> = e_text.lines().collect();
    let a: Vec<&str> = a_text.lines().collect();
    let mut prefix = 0;
    while prefix < e.len() && prefix < a.len() && e[prefix] == a[prefix] {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < e.len().saturating_sub(prefix)
        && suffix < a.len().saturating_sub(prefix)
        && e[e.len() - 1 - suffix] == a[a.len() - 1 - suffix]
    {
        suffix += 1;
    }
    let e_mid = &e[prefix..e.len() - suffix];
    let a_mid = &a[prefix..a.len() - suffix];
    if e_mid.is_empty() && a_mid.is_empty() {
        return None;
    }
    // Context window: `context` unchanged lines on each side of the whole
    // middle (leading context from the common prefix, trailing context
    // from the common suffix).
    let mid_end_e = e.len() - suffix;
    let mid_end_a = a.len() - suffix;
    let e_lo = prefix.saturating_sub(context);
    let e_hi = mid_end_e.saturating_add(context).min(e.len());
    let a_lo = prefix.saturating_sub(context);
    let a_hi = mid_end_a.saturating_add(context).min(a.len());
    let mut out = format!(
        "@@ -{},{} +{},{} @@\n",
        e_lo + 1,
        e_hi - e_lo,
        a_lo + 1,
        a_hi - a_lo
    );
    for line in &e[e_lo..prefix] {
        out.push_str(&format!(" {}\n", line));
    }
    for line in e_mid {
        out.push_str(&format!("-{}\n", line));
    }
    for line in a_mid {
        out.push_str(&format!("+{}\n", line));
    }
    for line in &e[mid_end_e..e_hi] {
        out.push_str(&format!(" {}\n", line));
    }
    Some(out)
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

    #[test]
    fn unified_diff_shows_the_changed_region() {
        let expected = b"a\nb\nc\nd\ne\n";
        let actual = b"a\nb\nX\nd\ne\n";
        let diff = unified_diff_hunk(expected, actual, 1).expect("must differ");
        // Leading context b, removed c, added X, trailing context d — the
        // hunk spans 3 lines per side starting at line 2.
        assert!(diff.starts_with("@@ -2,3 +2,3 @@"), "{}", diff);
        assert!(diff.contains(" b\n"), "{}", diff);
        assert!(diff.contains("-c\n"), "{}", diff);
        assert!(diff.contains("+X\n"), "{}", diff);
        assert!(diff.contains(" d\n"), "{}", diff);
        assert!(unified_diff_hunk(expected, expected, 3).is_none());
        // Multi-line middle and pure insertion/deletion shapes.
        let del = unified_diff_hunk(b"a\nx\ny\nb\n", b"a\nb\n", 1).unwrap();
        assert!(
            del.contains("-x\n") && del.contains("-y\n") && !del.contains("\n+"),
            "{}",
            del
        );
        let ins = unified_diff_hunk(b"a\nb\n", b"a\nnew\nb\n", 1).unwrap();
        assert!(
            ins.contains("+new\n") && !ins.contains("\n-"),
            "{}",
            ins
        );
    }
}
