//! Piecewise source-alignment map (PLAN-828 F-04, R7a/R8a).
//!
//! One [`SpanSegments`] tracks, for ONE code object minted from a known
//! source text (the MINT frame, local decl coordinates), how the mint
//! bytes align to the CURRENT text. The mint text is split into disjoint
//! SEGMENTS `(m0, m1, c0)` — mint range `[m0, m1)` currently sits at
//! current offset `c0` — and everything else in the current text is
//! inserted content with NO mint backing (blobs live in the current gaps
//! between segments). An edit is interval surgery on this table; mapping
//! a mint position is a binary search. There are no boundary heuristics:
//! token semantics (an insertion exactly before a token pushes it right,
//! an insertion at the token's end does not, straddles move only the
//! end) all fall out of where segments sit relative to the blobs.

/// Alignment table: disjoint mint segments, sorted by `m0`, covering the
/// surviving mint bytes; `c0` is each segment's CURRENT offset.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpanSegments {
    segs: Vec<(u64, u64, u64)>,
}

impl SpanSegments {
    /// A fresh identity map over `mint_len` mint bytes.
    pub fn new(mint_len: u64) -> Self {
        SpanSegments {
            segs: vec![(0, mint_len, 0)],
        }
    }

    /// An identity map for one ABSOLUTE mint range `[m0, m1)` whose
    /// current position is `c0` (usually the same absolute offset).
    pub fn identity_range(m0: u64, m1: u64, c0: u64) -> Self {
        SpanSegments {
            segs: vec![(m0, m1, c0)],
        }
    }

    /// Apply an edit: current range [c0, c1) is replaced by `new_len`
    /// bytes. Segments fully inside the range are dropped (their mint
    /// text is gone); a straddling segment is split, keeping the part
    /// outside the range; segments after the range shift by the length
    /// delta; the replacement text itself is an unbacked blob.
    pub fn apply(&mut self, c0: u64, c1: u64, new_len: u64) {
        let cur_len = c1.saturating_sub(c0);
        let delta = new_len as i64 - cur_len as i64;
        let mut out: Vec<(u64, u64, u64)> = Vec::with_capacity(self.segs.len() + 2);
        for &(m0, m1, s0) in self.segs.iter() {
            let s1 = s0 + (m1 - m0); // segment's current end
            if s1 <= c0 || s0 >= c1 {
                // Entirely outside the edit.
                let shift = if s0 >= c1 { delta } else { 0 };
                out.push((m0, m1, (s0 as i64 + shift) as u64));
                continue;
            }
            // Overlaps [c0, c1): keep the surviving head/tail pieces.
            if s0 < c0 {
                let keep_len = c0 - s0;
                out.push((m0, m0 + keep_len, s0));
            }
            if s1 > c1 {
                let drop_len = c1.saturating_sub(s0);
                let tail_m0 = m0 + drop_len;
                out.push((tail_m0, m1, c0 + new_len));
            }
        }
        out.sort();
        self.segs = out;
    }

    /// Map a mint position to its CURRENT position (the mint byte's
    /// segment offset). Positions past the last surviving segment clamp
    /// to the current end of the last segment.
    /// Map a mint position NEUTRALLY (segment lookup; a split point
    /// resolves to the next segment's start).
    pub fn map(&self, x: u64) -> u64 {
        let mut best: u64 = x;
        for &(m0, m1, s0) in self.segs.iter() {
            if m0 <= x && x < m1 {
                return s0 + (x - m0);
            }
            if m0 > x {
                break;
            }
            // x is at/after this segment's end: remember its current end
            // (positions whose mint bytes were replaced or deleted map
            // to the surviving content boundary).
            best = s0 + (m1 - m0);
        }
        best
    }

    /// Map a half-open mint span [s, e) with TOKEN semantics: the START
    /// is the token's FIRST byte — at a split point it follows the blob
    /// inserted exactly before it (whitespace before a token pushes it
    /// right); the END is one past the LAST byte — at a split point it
    /// stays with the preceding segment (an insertion exactly at the end
    /// does not extend the token).
    pub fn map_span(&self, s: u64, e: u64) -> (u64, u64) {
        let start = self.map_token_start(s);
        let end = self.map_token_end(e);
        (start, end.max(start))
    }

    fn map_token_start(&self, x: u64) -> u64 {
        self.map(x)
    }

    fn map_token_end(&self, x: u64) -> u64 {
        for &(m0, m1, s0) in self.segs.iter() {
            if m0 <= x && x < m1 {
                return s0 + (x - m0);
            }
            if m0 > x {
                break;
            }
            if m1 == x {
                return s0 + (m1 - m0);
            }
        }
        self.map(x)
    }

    /// The count of surviving segments (diagnostics).
    pub fn segment_count(&self) -> usize {
        self.segs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity: identity, insert-before, straddle, delete, replace, undo,
    /// and mixed spanning edits — the semantics the review matrix pins.
    #[test]
    fn span_segments_semantics() {
        // "return 1 / 0": the trap token is mint [8, 13) in a 13-byte body.
        let mut m = SpanSegments::new(13);
        assert_eq!(m.map(8), 8);
        // Insert 3 bytes exactly at the token start: token pushes right.
        m.apply(8, 8, 3);
        assert_eq!(m.map(8), 11);
        // Insert 1 byte inside the token range (at 11): end moves, start not.
        m.apply(12, 12, 1);
        assert_eq!(m.map(8), 11);
        // Insert exactly at the token END (current end = 15): no move.
        m.apply(15, 15, 2);
        assert_eq!(m.map(8), 11);
        // Delete 2 of the blob bytes inserted exactly at the token start
        // (the blob occupies current [8, 11); the token byte itself is at
        // 11): the token moves back by exactly 2.
        m.apply(9, 11, 0);
        assert_eq!(m.map(8), 9);
        // Full undo of everything: identity restored.
        let mut u = SpanSegments::new(13);
        u.apply(8, 8, 32);
        u.apply(8, 40, 0);
        assert_eq!(u.map(8), 8);
        // Replace 1 mint byte with 64, then trim inside the blob.
        let mut r = SpanSegments::new(13);
        r.apply(4, 5, 64);
        assert_eq!(r.map(8), 8 + 63);
        r.apply(20, 36, 0);
        assert_eq!(r.map(8), 8 + 63 - 16);
        // A spanning edit removes mint bytes AND inserted bytes together.
        let mut s2 = SpanSegments::new(20);
        s2.apply(4, 4, 8); // blob of 8 before mint byte 4
        assert_eq!(s2.map(4), 12);
        assert_eq!(s2.map(12), 20);
        // Replace current [10, 18) with 2 bytes: covers 6 blob bytes plus
        // mint bytes 4..10 (current 12..18); mint 10.. survives at 12.
        s2.apply(10, 18, 2);
        assert_eq!(s2.map(10), 12);
        assert_eq!(s2.map(12), 14);
    }
}
