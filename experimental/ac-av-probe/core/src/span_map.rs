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
    /// Token-end extensions keyed by MINT end position: a value-equal
    /// numeric token whose spelling LENGTHENED ('0'→'00') keeps its old
    /// mint bytes as one segment, but the token END (one past the last
    /// byte) must move by the spelling-length difference — expression
    /// ends land on token ends (F-828-R1-04/R13a).
    end_ext: Vec<(u64, i64)>,
}

impl SpanSegments {
    /// A fresh identity map over `mint_len` mint bytes.
    pub fn new(mint_len: u64) -> Self {
        SpanSegments {
            segs: vec![(0, mint_len, 0)],
            end_ext: Vec::new(),
        }
    }

    /// Build the alignment between two TEXTS over the same mint range
    /// (a content hit's code re-anchoring to a rewritten fn text):
    /// prefix/suffix anchoring plus substring re-embedding of the
    /// differing middle, falling back to a blob when nothing matches.
    pub fn from_texts(m0: u64, m1: u64, c_base: u64, old: &[u8], new: &[u8]) -> Option<Self> {
        let mut p = 0usize;
        while p < old.len() && p < new.len() && old[p] == new[p] {
            p += 1;
        }
        let mut q = 0usize;
        while q < old.len() - p
            && q < new.len() - p
            && old[old.len() - 1 - q] == new[new.len() - 1 - q]
        {
            q += 1;
        }
        // Byte-level prefix matching can split a digit token the same
        // way ('0 }' vs '00 }': the shared '0' joins the prefix, leaving
        // an after-token insertion). Absorb prefix bytes that continue a
        // digit run in BOTH texts so the mids hold whole tokens.
        while p > 0 && p < new.len() && old[p - 1].is_ascii_digit() && new[p].is_ascii_digit() {
            p -= 1;
        }
        // Byte-level suffix matching can split a NEW-side digit token
        // ('1' old vs '01' new: the '1' suffix-matches, leaving mid_new a
        // bare '0'). Absorb suffix bytes that continue a digit run in the
        // NEW text so both mids hold WHOLE tokens (' 1' vs '01' → the
        // value-equal tokens align, F-828-R1-04/R12a).
        while q > 0
            && p + q < new.len()
            && new[new.len() - q - 1].is_ascii_digit()
            && new[new.len() - q].is_ascii_digit()
        {
            q -= 1;
        }
        let mid_old = &old[p..old.len() - q];
        let mid_new = &new[p..new.len() - q];
        let mid_m0 = m0 + p as u64;
        let mid_m1 = m1 - q as u64;
        let mut segs: Vec<(u64, u64, u64)> = Vec::new();
        let mut exts: Vec<(u64, i64)> = Vec::new();
        if mid_m0 > m0 {
            segs.push((m0, mid_m0, c_base));
        }
        let run_seqs_equal = {
            let a = runs(mid_old);
            let b = runs(mid_new);
            !a.is_empty()
                && a.len() == b.len()
                && a.iter()
                    .zip(b.iter())
                    .all(|((x, _), (y, _))| token_eq(x, y))
        };
        match (
            mid_old.is_empty(),
            run_seqs_equal,
            find_subslice(mid_new, mid_old),
        ) {
            (true, _, _) => {}
            (false, false, Some(k)) => {
                segs.push((mid_m0, mid_m1, c_base + p as u64 + k as u64));
                // A digit run continuing PAST the match extends the last
                // token ('1 / 0' is a prefix of '1 / 00'): the token end
                // moves by the trailing extra digits.
                if mid_old.last().is_some_and(|c| c.is_ascii_digit()) {
                    let mut e = k + mid_old.len();
                    while e < mid_new.len()
                        && mid_new[e].is_ascii_digit()
                        && mid_new[e - 1].is_ascii_digit()
                    {
                        e += 1;
                    }
                    let extra = (e - k - mid_old.len()) as i64;
                    if extra > 0 {
                        exts.push((mid_m1, extra));
                    }
                }
            }
            (false, _, _) => {
                // Token-run alignment: the middles may differ only in
                // INTERNAL whitespace (equivalent rewrite with internal
                // trivia). Align the non-whitespace runs; whitespace
                // between them becomes blob content.
                let old_runs = runs(mid_old);
                let new_runs = runs(mid_new);
                if old_runs.is_empty()
                    || new_runs.is_empty()
                    || old_runs.len() != new_runs.len()
                    || old_runs
                        .iter()
                        .zip(new_runs.iter())
                        .any(|((a, _), (b, _))| !token_eq(a, b))
                {
                    // No correspondence: a full-text rebuild would
                    // collapse the middle — keep the geometric map.
                    return None;
                }
                let mid_base_c = c_base + p as u64;
                for ((ob, os), (nb, ns)) in old_runs.iter().zip(new_runs.iter()) {
                    segs.push((
                        mid_m0 + os.0 as u64,
                        mid_m0 + os.1 as u64,
                        mid_base_c + ns.0 as u64,
                    ));
                    let d_old = (os.1 - os.0) as i64;
                    let d_new = (ns.1 - ns.0) as i64;
                    if ob != nb && d_new != d_old {
                        // Same value, longer spelling: the token end moves.
                        exts.push((mid_m0 + os.1 as u64, d_new - d_old));
                    }
                }
            }
        }
        if m1 > mid_m1 {
            segs.push((mid_m1, m1, c_base + p as u64 + mid_new.len() as u64));
        }
        Some(SpanSegments {
            segs,
            end_ext: exts,
        })
    }

    /// An identity map for one ABSOLUTE mint range `[m0, m1)` whose
    /// current position is `c0` (usually the same absolute offset).
    pub fn identity_range(m0: u64, m1: u64, c0: u64) -> Self {
        SpanSegments {
            segs: vec![(m0, m1, c0)],
            end_ext: Vec::new(),
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
        let base = {
            let mut r = None;
            for &(m0, m1, s0) in self.segs.iter() {
                if m0 <= x && x < m1 {
                    r = Some(s0 + (x - m0));
                    break;
                }
                if m0 > x {
                    break;
                }
                if m1 == x {
                    r = Some(s0 + (m1 - m0));
                    break;
                }
            }
            match r {
                Some(v) => v,
                None => return self.map(x),
            }
        };
        // A lengthened same-value token's END moves by the spelling delta.
        let extra: i64 = self
            .end_ext
            .iter()
            .filter(|&&(k, _)| k == x)
            .map(|&(_, d)| d)
            .sum();
        (base as i64 + extra) as u64
    }

    /// Content-aware composition (F-828-R1-04/R8a-check): like [`apply`],
    /// but when the replaced bytes reappear INSIDE the replacement (an
    /// equivalent token rewritten with different surrounding trivia — a
    /// content hit), the mint bytes keep their anchor at the re-embedded
    /// position instead of collapsing into an unbacked blob. Falls back
    /// to geometric surgery when no re-embedding exists.
    pub fn apply_aligned(&mut self, c0: u64, c1: u64, old: &[u8], new: &[u8]) {
        // Trim the common prefix/suffix: those bytes never moved.
        let mut p = 0usize;
        while p < old.len() && p < new.len() && old[p] == new[p] {
            p += 1;
        }
        let mut q = 0usize;
        while q < old.len() - p
            && q < new.len() - p
            && old[old.len() - 1 - q] == new[new.len() - 1 - q]
        {
            q += 1;
        }
        let mid_old = &old[p..old.len() - q];
        let mid_new = &new[p..new.len() - q];
        let mid_c0 = c0 + p as u64;
        let mid_c1 = c1 - q as u64;
        if mid_old.is_empty() {
            // Pure insertion in the middle.
            self.apply(mid_c0, mid_c1, mid_new.len() as u64);
            return;
        }
        if let Some(k) = find_subslice(mid_new, mid_old) {
            // The replaced mint bytes re-appear at offset k inside the
            // replacement: keep them anchored at the re-embedded position
            // when the middle covers ONE contiguous mint run.
            if let Some((m0, m1)) = self.contiguous_mint_range(mid_c0, mid_c1) {
                let shift = (mid_c0 + k as u64) as i64 - mid_c0 as i64;
                let delta = new.len() as i64 - old.len() as i64;
                let mut out: Vec<(u64, u64, u64)> = Vec::with_capacity(self.segs.len() + 2);
                for &(s0_m, s1_m, s_pos) in self.segs.iter() {
                    let s_end = s_pos + (s1_m - s0_m);
                    if s_end <= mid_c0 || s_pos >= mid_c1 {
                        let mv = if s_pos >= mid_c1 { delta } else { 0 };
                        out.push((s0_m, s1_m, (s_pos as i64 + mv) as u64));
                    } else if s0_m < m0 {
                        out.push((s0_m, m0, s_pos));
                    } else if s1_m > m1 {
                        out.push((m1, s1_m, (mid_c0 + k as u64 + mid_old.len() as u64)));
                    }
                }
                out.push((m0, m1, mid_c0 + k as u64));
                out.sort();
                self.segs = out;
                return;
            }
        }
        self.apply(mid_c0, mid_c1, mid_new.len() as u64);
    }

    /// The mint range covered by current [c0, c1) when it lies inside ONE
    /// segment (None when it spans segments or blobs).
    fn contiguous_mint_range(&self, c0: u64, c1: u64) -> Option<(u64, u64)> {
        for &(m0, m1, s0) in self.segs.iter() {
            let s1 = s0 + (m1 - m0);
            if s0 <= c0 && c1 <= s1 {
                return Some((m0 + (c0 - s0), m0 + (c1 - s0)));
            }
        }
        None
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

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > hay.len() {
        return None;
    }
    let comments = comment_ranges(hay);
    let hit = hay.windows(needle.len()).enumerate().find_map(|(i, w)| {
        if w == needle && !comments.iter().any(|&(a, b)| a < i + needle.len() && i < b) {
            Some(i)
        } else {
            None
        }
    })?;
    // A match landing mid-digit-run ('1' matching the suffix of '01') is
    // NOT the token start: snap left to the enclosing lexical token's
    // start — a same-value spelling change re-anchors the WHOLE token
    // (F-828-R1-04/R12a).
    let mut k = hit;
    while k > 0 && hay[k - 1].is_ascii_digit() && hay[k].is_ascii_digit() {
        k -= 1;
    }
    Some(k)
}

/// Token equality: byte equality, or numeric equality for decimal
/// literals ('1' and '01' are the same value token).
fn token_eq(a: &[u8], b: &[u8]) -> bool {
    if a == b {
        return true;
    }
    let digits = |x: &[u8]| !x.is_empty() && x.iter().all(|c| c.is_ascii_digit());
    if !digits(a) || !digits(b) {
        return false;
    }
    fn strip(x: &[u8]) -> &[u8] {
        let t = x.iter().position(|c| *c != b'0').unwrap_or(x.len() - 1);
        &x[t..]
    }
    strip(a) == strip(b)
}

/// Comment regions of the text: from `//` to end of line (excluded from
/// expression anchoring — comment TEXT is not code, F-828-R1-04/R11a).
fn in_ranges(ranges: &[(usize, usize)], i: usize) -> bool {
    ranges.iter().any(|&(a, b)| i >= a && i < b)
}

fn comment_ranges(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'/' {
            let start = i;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            out.push((start, i.min(bytes.len())));
        } else {
            i += 1;
        }
    }
    out
}

/// LEXICAL tokens of a byte string: numbers, identifiers, and single-
/// character operators, skipping whitespace (F-828-R1-04/R10a: a
/// non-whitespace RUN is not a token — `1/0` is three tokens).
fn runs(bytes: &[u8]) -> Vec<(&[u8], (usize, usize))> {
    let comments = comment_ranges(bytes);
    let skip = |i: usize| bytes[i].is_ascii_whitespace() || in_ranges(&comments, i);
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if skip(i) {
            i += 1;
        } else if b.is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() && !skip(i) {
                i += 1;
            }
            out.push((&bytes[start..i], (start, i)));
        } else if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                && !skip(i)
            {
                i += 1;
            }
            out.push((&bytes[start..i], (start, i)));
        } else {
            out.push((&bytes[i..i + 1], (i, i + 1)));
            i += 1;
        }
    }
    out
}
