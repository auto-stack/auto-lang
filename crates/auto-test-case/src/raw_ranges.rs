//! Byte-accurate raw fence locator.
//!
//! Extracts fenced-code payload byte ranges from the ORIGINAL document bytes
//! (no serializer, no re-render, EOLs preserved verbatim), replicating the
//! vendored tree parser's fence rules exactly (Plan 768 feasibility §2):
//!
//! - a fence line has leading indent < 4 (space = 1, tab = 4, like
//!   `indentWidth` upstream);
//! - an OPENING line's trimmed content starts with a run of 3+ `` ` `` or
//!   `~` markers; everything after the run is the info string (trimmed);
//! - a CLOSING line's trimmed content is EXACTLY the same-length run of the
//!   same marker (no other characters, exact length);
//! - anything else inside is payload: bytes from after the opening line's
//!   EOL up to the start of the closing line — including every newline that
//!   actually precedes the closing fence.
//!
//! The locator carries no heading/case semantics of its own; pairing with the
//! block tree is validated in [`crate::document`], and a mismatch is an error
//! there, never silently tolerated here.

/// One located fence with its raw payload range in the original document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFence {
    /// Trimmed info string following the opening marker run.
    pub info: String,
    /// Byte range of the payload in the original document bytes.
    pub payload: std::ops::Range<usize>,
    /// Whether a matching closing fence line was found.
    pub closed: bool,
    /// 1-based line number of the opening fence.
    pub open_line: usize,
    /// 1-based line number of the closing fence, when closed.
    pub close_line: Option<usize>,
}

/// Leading indent width: each space counts 1, each tab counts 4 — replicates
/// upstream `indentWidth`.
pub fn indent_width(line: &str) -> usize {
    let mut n = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => n += 1,
            '\t' => n += 4,
            _ => break,
        }
    }
    n
}

/// Line table over raw bytes: entries are (content_start, content_end) with
/// content_end excluding the `\r` of a `\r\n` EOL and the `\n` itself.
fn line_table(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            let mut e = i;
            if e > start && bytes[e - 1] == b'\r' {
                e -= 1;
            }
            lines.push((start, e));
            start = i + 1;
        }
    }
    if start < bytes.len() {
        lines.push((start, bytes.len()));
    }
    lines
}

/// Locate every fence in the document, in source order.
///
/// The input must be valid UTF-8 (enforced by [`crate::document`]); structure
/// scanning works on bytes because every syntactically relevant character is
/// ASCII. Fences that never close run to end-of-input with `closed == false`.
pub fn locate_fences(bytes: &[u8]) -> Vec<RawFence> {
    let lines = line_table(bytes);
    let mut out: Vec<RawFence> = Vec::new();
    let mut li = 0usize;
    while li < lines.len() {
        let (s, e) = lines[li];
        let line = String::from_utf8_lossy(&bytes[s..e]).to_string();
        if indent_width(&line) >= 4 {
            li += 1;
            continue;
        }
        let t = line.trim();
        let marker = match t.chars().next() {
            Some('`') if t.starts_with("```") => '`',
            Some('~') if t.starts_with("~~~") => '~',
            _ => '\0',
        };
        if marker == '\0' {
            li += 1;
            continue;
        }
        let run: String = t.chars().take_while(|ch| *ch == marker).collect();
        let info = t
            .chars()
            .skip(run.chars().count())
            .collect::<String>()
            .trim()
            .to_string();
        let mut j = li + 1;
        let mut closed = false;
        while j < lines.len() {
            let (cs, ce) = lines[j];
            let cline = String::from_utf8_lossy(&bytes[cs..ce]).to_string();
            if indent_width(&cline) < 4 {
                let ct = cline.trim();
                if ct.chars().count() == run.chars().count() && ct.chars().all(|ch| ch == marker) {
                    closed = true;
                    break;
                }
            }
            j += 1;
        }
        let payload_start = if li + 1 < lines.len() {
            lines[li + 1].0
        } else {
            bytes.len()
        };
        let payload_end = if closed { lines[j].0 } else { bytes.len() };
        out.push(RawFence {
            info,
            payload: payload_start..payload_end,
            closed,
            open_line: li + 1,
            close_line: if closed { Some(j + 1) } else { None },
        });
        if closed {
            li = j + 1;
        } else {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent_width_spaces_and_tabs() {
        assert_eq!(indent_width("x"), 0);
        assert_eq!(indent_width("  x"), 2);
        assert_eq!(indent_width("\tx"), 4);
        assert_eq!(indent_width(" \t x"), 6);
        assert_eq!(indent_width("    x"), 4);
    }

    #[test]
    fn empty_document_has_no_fences() {
        assert!(locate_fences(b"").is_empty());
        assert!(locate_fences(b"plain text\r\nno fences\n").is_empty());
    }
}
