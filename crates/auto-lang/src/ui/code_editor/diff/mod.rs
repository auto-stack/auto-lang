// PLAN-703 T-02/T-03/T-04 — the kernel diff engine (supply pack 供①a/①b/③).
//
// Pipeline (the ONE pipeline for every entry point — determinism by
// construction; AC-05's parallel≡serial compares thread counts, not
// algorithms):
//   1. universal-newlines line split (CRLF tolerated: trailing '\r'
//      stripped per line; one trailing empty piece absorbed — mirrors the
//      downstream envelope contract, diff-view.md SD-02),
//   2. first-occurrence line interning into dense imara tokens (our own
//      interning — deterministic ids, no per-process hasher state),
//   3. patience anchors: line values occurring exactly once in BOTH middles
//      are forced keeps; they partition the change region into independent
//      segments (the anchor partition is content-determined, so serial and
//      parallel runs slice identically),
//   4. histogram diff (imara-diff, git's algorithm) per segment — ordered
//      merge of the per-segment change streams,
//   5. hunk grouping with ctx expansion (gap ≤ 2·ctx merges, clamped to the
//      file bounds) — field-identical to the downstream envelope contract
//      (hunks 0-based half-open `[a1,a2)`).
//
// Window calls (`diff_lines_windowed`) are a PROJECTION of the full result:
// histogram alignment depends on global line frequencies, so a re-diff of a
// slice is NOT guaranteed to equal the full diff's window slice — the
// windowed face therefore computes the full diff and filters. Downstream
// lazy consumers get a stable projection; true local recomputation is the
// incremental-re-diff follow-up (supply pack §2), not this plan.
//
// Snapshot diffs (`diff_snapshots`) run the T-01 prune walk first: shared
// byte spans skip line processing entirely; each diverged span is
// line-expanded and diffed independently, then merged into one global
// grouping.

use std::collections::HashMap;
use std::ops::Range;

use imara_diff::{Algorithm, Diff as ImaraDiff, Token};

use super::core::rope::RopeSnapshot;

pub mod dirs;
pub mod envelope;
pub use dirs::{DirCounts, DirDiffError, DirDiffIter, DirDiffOptions, DirEntryDiff, DirStatus};

/// Context radius for hunk grouping (lines shown around each change).
/// Zero falls back to the downstream default of 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffOpts {
    pub ctx: usize,
}

impl Default for DiffOpts {
    fn default() -> Self {
        DiffOpts { ctx: 3 }
    }
}

impl DiffOpts {
    pub fn ctx(mut self, ctx: usize) -> Self {
        self.ctx = ctx.max(1);
        self
    }
}

/// One changed region: `[a1,a2)` old lines replaced by `[b1,b2)` new lines
/// (0-based half-open, ctx-expanded — the downstream envelope contract
/// shape). Pure insertions have `a1 == a2`; pure deletions `b1 == b2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hunk {
    pub a1: usize,
    pub a2: usize,
    pub b1: usize,
    pub b2: usize,
}

/// Engine output: envelope hunks + change counts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiffOut {
    pub hunks: Vec<Hunk>,
    pub adds: usize,
    pub dels: usize,
}

impl DiffOut {
    pub fn is_empty(&self) -> bool {
        self.hunks.is_empty()
    }
}

/// A non-keep step of the edit script, in stream order. `i` = a-side line
/// index (valid for deletes), `j` = b-side line index (valid for adds);
/// the other side carries the current cursor (downstream stream shape).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Change {
    pub del: bool,
    pub i: usize,
    pub j: usize,
}

// ---------------------------------------------------------------------------
// Line model
// ---------------------------------------------------------------------------

/// Universal-newlines split mirroring the downstream envelope contract:
/// split on `'\n'`, strip one trailing `'\r'` per line, absorb the single
/// trailing empty piece (a trailing newline opens no phantom line).
pub(crate) fn split_lines_universal(text: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = Vec::new();
    let mut rest = text;
    while let Some(pos) = rest.find('\n') {
        let (line, next) = rest.split_at(pos);
        lines.push(line.strip_suffix('\r').unwrap_or(line));
        rest = &next[1..];
    }
    // Tail without a newline — unless the text ended with one (then `rest`
    // is empty and is absorbed).
    if !rest.is_empty() {
        lines.push(rest.strip_suffix('\r').unwrap_or(rest));
    }
    lines
}

// ---------------------------------------------------------------------------
// Interning + histogram core
// ---------------------------------------------------------------------------

pub(crate) struct Interned {
    pub before: Vec<Token>,
    pub after: Vec<Token>,
}

/// First-occurrence interning: dense deterministic token ids (before lines
/// numbered first, then after lines). No hasher state → cross-run stable.
pub(crate) fn intern_lines(a_lines: &[&str], b_lines: &[&str]) -> Interned {
    let mut map: HashMap<&str, Token> = HashMap::with_capacity((a_lines.len() + b_lines.len()) / 2 + 8);
    let mut before = Vec::with_capacity(a_lines.len());
    for line in a_lines {
        let next_id = Token(map.len() as u32);
        before.push(*map.entry(line).or_insert(next_id));
    }
    let mut after = Vec::with_capacity(b_lines.len());
    for line in b_lines {
        let next_id = Token(map.len() as u32);
        after.push(*map.entry(line).or_insert(next_id));
    }
    Interned { before, after }
}

fn interned_len(inp: &Interned) -> u32 {
    let max_before = inp.before.iter().map(|t| t.0 as usize + 1).max().unwrap_or(0);
    let max_after = inp.after.iter().map(|t| t.0 as usize + 1).max().unwrap_or(0);
    max_before.max(max_after) as u32
}

/// Histogram diff over interned token slices → the change stream (del-first
/// deterministic walk) + counts.
pub(crate) fn histogram_changes(before: &[Token], after: &[Token], num_tokens: u32) -> Vec<Change> {
    let mut diff = ImaraDiff::default();
    diff.compute_with(Algorithm::Histogram, before, after, num_tokens);
    let mut changes = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    let (la, lb) = (before.len(), after.len());
    while i < la || j < lb {
        if i < la && diff.is_removed(i as u32) {
            changes.push(Change { del: true, i, j });
            i += 1;
        } else if j < lb && diff.is_added(j as u32) {
            changes.push(Change { del: false, i, j });
            j += 1;
        } else {
            i += 1;
            j += 1;
        }
    }
    changes
}

/// Patience anchor partition of a token pair: positions `(a_pos, b_pos)` of
/// line values occurring exactly once on EACH side (forced matches).
/// Content-determined — the same inputs always slice the same way.
pub(crate) fn anchor_partition(before: &[Token], after: &[Token], num_tokens: u32) -> Vec<(usize, usize)> {
    let n = num_tokens as usize;
    let mut count_a = vec![0u32; n];
    let mut count_b = vec![0u32; n];
    let mut first_a: Vec<i64> = vec![-1; n];
    for (idx, &t) in before.iter().enumerate() {
        count_a[t.0 as usize] += 1;
        if first_a[t.0 as usize] < 0 {
            first_a[t.0 as usize] = idx as i64;
        }
    }
    for &t in after {
        count_b[t.0 as usize] += 1;
    }
    let mut anchors = Vec::new();
    for (j, &t) in after.iter().enumerate() {
        let id = t.0 as usize;
        if count_a[id] == 1 && count_b[id] == 1 && first_a[id] >= 0 {
            anchors.push((first_a[id] as usize, j));
        }
    }
    anchors.sort_unstable();
    // PLAN-704 D-2: unique-per-side anchor sets are not order-consistent by
    // construction — reorder/move families yield block-swapped anchors whose
    // b positions bounce between blocks, and the segment builder below walks
    // them as if monotone (`cursor = (ai+1, bj+1)` walks backwards), corrupts
    // the segment list, and produces a wrong edit script (a 620-line swap
    // degraded to "620 adds / 0 dels" — downstream PLAN-016 registration).
    // a is already strictly increasing (distinct tokens have distinct
    // first-occurrence positions); filter to the longest strictly
    // b-increasing subsequence — patience LIS, O(n log n).
    // Content-determined: the same anchor set always yields the same
    // subsequence.
    longest_monotone_anchors(anchors)
}

/// Longest subsequence of `anchors` strictly increasing on BOTH coordinates
/// (a arrives pre-sorted and strict; the filter enforces b). Standard
/// patience LIS with predecessor reconstruction — deterministic for a given
/// input, no hash-order dependence.
fn longest_monotone_anchors(anchors: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    if anchors.len() < 2 {
        return anchors;
    }
    debug_assert!(
        anchors.windows(2).all(|w| w[0].0 < w[1].0),
        "a strictly increasing by construction"
    );
    let mut tails: Vec<usize> = Vec::with_capacity(anchors.len());
    let mut prev: Vec<usize> = vec![usize::MAX; anchors.len()];
    for (i, &(_, b)) in anchors.iter().enumerate() {
        let pos = tails.partition_point(|&k| anchors[k].1 < b);
        if pos == tails.len() {
            tails.push(i);
        } else {
            tails[pos] = i;
        }
        if pos > 0 {
            prev[i] = tails[pos - 1];
        }
    }
    let mut out = Vec::with_capacity(tails.len());
    let mut k = tails[tails.len() - 1];
    while k != usize::MAX {
        out.push(anchors[k]);
        k = prev[k];
    }
    out.reverse();
    debug_assert!(out.windows(2).all(|w| w[0].0 < w[1].0 && w[0].1 < w[1].1));
    out
}

/// Segment size below which the anchor partition is skipped (the histogram
/// handles small middles directly; partitioning overhead not worth it).
const ANCHOR_MIN_SEG: usize = 512;

/// The full change stream for two interned line tables: common
/// prefix/suffix trim, anchor partition of the middle into independent
/// segments, histogram per segment, positional merge (keeps between
/// segments are implicit — the change stream only carries non-keep steps).
pub(crate) fn engine_changes(inp: &Interned, parallel: bool) -> Vec<Change> {
    let (before, after) = (&inp.before, &inp.after);
    let mut p = 0usize;
    while p < before.len() && p < after.len() && before[p] == after[p] {
        p += 1;
    }
    let mut s = 0usize;
    while s < before.len() - p && s < after.len() - p && before[before.len() - 1 - s] == after[after.len() - 1 - s] {
        s += 1;
    }
    let a_mid = &before[p..before.len() - s];
    let b_mid = &after[p..after.len() - s];
    if a_mid.is_empty() && b_mid.is_empty() {
        return Vec::new();
    }
    let num_tokens = interned_len(inp);

    // Anchors only pay off on large middles.
    let anchors = if a_mid.len().max(b_mid.len()) >= ANCHOR_MIN_SEG {
        anchor_partition(a_mid, b_mid, num_tokens)
    } else {
        Vec::new()
    };

    // Segment list [sa, sb, ea, eb) in middle coordinates; anchors are the
    // forced keeps BETWEEN segments (never inside one).
    let mut segs: Vec<(usize, usize, usize, usize)> = Vec::new();
    let mut cursor = (0usize, 0usize);
    for &(ai, bj) in &anchors {
        if ai > cursor.0 || bj > cursor.1 {
            segs.push((cursor.0, cursor.1, ai, bj));
        }
        cursor = (ai + 1, bj + 1);
    }
    if cursor.0 < a_mid.len() || cursor.1 < b_mid.len() {
        segs.push((cursor.0, cursor.1, a_mid.len(), b_mid.len()));
    }

    // Per-segment id compression: global token ids are dense across the
    // whole document, so a late single-line segment still carries ids
    // ~2.9M — and imara sizes its occurrence table by the token bound,
    // making every tiny segment allocate a table the size of the document
    // (the 100MB/1% bench sat in exactly this O(segments × tokens)
    // quadratic). Remapping to per-segment rank keeps ids deterministic
    // (first-occurrence order within the segment) and every table O(seg).
    fn local_remap(seg_a: &[Token], seg_b: &[Token]) -> (Vec<Token>, Vec<Token>, u32) {
        let mut map: HashMap<Token, Token> = HashMap::with_capacity(seg_a.len() + seg_b.len());
        let mut out_a = Vec::with_capacity(seg_a.len());
        for t in seg_a {
            let n = Token(map.len() as u32);
            out_a.push(*map.entry(*t).or_insert(n));
        }
        let mut out_b = Vec::with_capacity(seg_b.len());
        for t in seg_b {
            let n = Token(map.len() as u32);
            out_b.push(*map.entry(*t).or_insert(n));
        }
        (out_a, out_b, map.len() as u32)
    }

    let mut results: Vec<Vec<Change>> = vec![Vec::new(); segs.len()];
    if parallel && segs.len() > 1 {
        let workers = segs.len().min(std::thread::available_parallelism().map_or(4, |n| n.get()));
        let chunk = segs.len().div_ceil(workers);
        let segs_ref = &segs;
        std::thread::scope(|scope| {
            let mut rest: &mut [Vec<Change>] = &mut results;
            let mut next_base = 0usize;
            for _ in 0..workers {
                let take = chunk.min(segs_ref.len() - next_base);
                if take == 0 {
                    break;
                }
                let (head, tail) = rest.split_at_mut(take);
                rest = tail;
                let base = next_base;
                scope.spawn(move || {
                    for (k, slot) in head.iter_mut().enumerate() {
                        let idx = base + k;
                        let (sa, sb, ea, eb) = segs_ref[idx];
                        let (ta, tb, bound) = local_remap(&a_mid[sa..ea], &b_mid[sb..eb]);
                        *slot = histogram_changes(&ta, &tb, bound)
                            .into_iter()
                            .map(|c| Change { del: c.del, i: c.i + p + sa, j: c.j + p + sb })
                            .collect();
                    }
                });
                next_base += take;
            }
        });
    } else {
        for (idx, slot) in results.iter_mut().enumerate() {
            let (sa, sb, ea, eb) = segs[idx];
            let (ta, tb, bound) = local_remap(&a_mid[sa..ea], &b_mid[sb..eb]);
            *slot = histogram_changes(&ta, &tb, bound)
                .into_iter()
                .map(|c| Change { del: c.del, i: c.i + p + sa, j: c.j + p + sb })
                .collect();
        }
    }

    let mut out = Vec::new();
    for mut seg in results {
        out.append(&mut seg);
    }
    out
}

// ---------------------------------------------------------------------------
// Hunk grouping (downstream envelope semantics)
// ---------------------------------------------------------------------------

/// A hunk with its change-stream annotations (downstream internal shape:
/// `fc`/`lc` = first/last change's stream index, `fi` = first change's
/// a-coordinate, `ah` = a-side consumption watermark) — the rows builder
/// slices the keep+change stream by these.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GroupedHunk {
    pub hunk: Hunk,
    pub fc: usize,
    pub lc: usize,
    pub fi: usize,
    pub ah: usize,
}

/// Group a change stream into ctx-expanded hunks. Gap rule (downstream ⑤):
/// a change joins the open group iff it is within `2·ctx` of the previous
/// change on BOTH coordinates.
pub(crate) fn group_hunks(changes: &[Change], la: usize, lb: usize, ctx: usize) -> Vec<Hunk> {
    group_hunks_annotated(changes, la, lb, ctx).into_iter().map(|g| g.hunk).collect()
}

// The grouping macro's trailing `open = false` is a dead write after the
// final close — structural to the close-at-both-ends pattern.
#[allow(unused_assignments)]
pub(crate) fn group_hunks_annotated(changes: &[Change], la: usize, lb: usize, ctx: usize) -> Vec<GroupedHunk> {
    let ctx2 = 2 * ctx;
    let mut out: Vec<GroupedHunk> = Vec::new();
    let mut open = false;
    let (mut ga1, mut ga2, mut gb1, mut gb2) = (0usize, 0usize, 0usize, 0usize);
    let (mut last_i, mut last_j) = (0usize, 0usize);
    macro_rules! close_group {
        () => {
            if open {
                let g = out.last_mut().unwrap();
                g.hunk = emit(ga1, ga2, gb1, gb2, la, lb, ctx);
                open = false;
            }
        };
    }
    for (idx, ch) in changes.iter().enumerate() {
        if open {
            let same = ch.i.saturating_sub(last_i) <= ctx2 && ch.j.saturating_sub(last_j) <= ctx2;
            if !same {
                close_group!();
            }
        }
        if !open {
            open = true;
            ga1 = ch.i;
            ga2 = ch.i;
            gb1 = ch.j;
            gb2 = ch.j;
            out.push(GroupedHunk { hunk: Hunk { a1: 0, a2: 0, b1: 0, b2: 0 }, fc: idx, lc: idx, fi: ch.i, ah: ch.i });
        } else {
            ga1 = ga1.min(ch.i);
            gb1 = gb1.min(ch.j);
            out.last_mut().unwrap().lc = idx;
        }
        let a2 = if ch.del { ch.i + 1 } else { ch.i };
        let b2 = if ch.del { ch.j } else { ch.j + 1 };
        ga2 = ga2.max(a2);
        gb2 = gb2.max(b2);
        let g = out.last_mut().unwrap();
        g.ah = g.ah.max(a2);
        last_i = ch.i;
        last_j = ch.j;
    }
    close_group!();
    out
}

fn emit(ga1: usize, ga2: usize, gb1: usize, gb2: usize, la: usize, lb: usize, ctx: usize) -> Hunk {
    Hunk {
        a1: ga1.saturating_sub(ctx),
        a2: (ga2 + ctx).min(la),
        b1: gb1.saturating_sub(ctx),
        b2: (gb2 + ctx).min(lb),
    }
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Line-level diff of two texts (histogram, anchor-partitioned on large
/// middles). Deterministic: the same inputs always produce the same hunks.
pub fn diff_lines(a: &str, b: &str, opts: DiffOpts) -> DiffOut {
    let a_lines = split_lines_universal(a);
    let b_lines = split_lines_universal(b);
    diff_line_tables(&a_lines, &b_lines, opts, false)
}

/// The parallel variant of [`diff_lines`]: identical output — segments are
/// dispatched to a bounded worker pool, merge is positional (AC-05).
pub fn diff_lines_parallel(a: &str, b: &str, opts: DiffOpts) -> DiffOut {
    let a_lines = split_lines_universal(a);
    let b_lines = split_lines_universal(b);
    diff_line_tables(&a_lines, &b_lines, opts, true)
}

pub(crate) fn diff_line_tables(a_lines: &[&str], b_lines: &[&str], opts: DiffOpts, parallel: bool) -> DiffOut {
    let inp = intern_lines(a_lines, b_lines);
    let changes = engine_changes(&inp, parallel);
    let adds = changes.iter().filter(|c| !c.del).count();
    let dels = changes.iter().filter(|c| c.del).count();
    let hunks = group_hunks(&changes, a_lines.len(), b_lines.len(), opts.ctx);
    DiffOut { hunks, adds, dels }
}

/// Window projection: the full diff filtered to the given 0-based
/// half-open line windows. A hunk belongs to the projection iff either of
/// its sides intersects the respective window; it is clipped to the window
/// rectangle. `adds`/`dels` count the changes whose coordinates fall inside
/// the windows. See the module header for why this is a projection, not a
/// re-diff (AC-04 consistency by construction).
pub fn diff_lines_windowed(a: &str, b: &str, opts: DiffOpts, aw: Range<usize>, bw: Range<usize>) -> DiffOut {
    let a_lines = split_lines_universal(a);
    let b_lines = split_lines_universal(b);
    let inp = intern_lines(&a_lines, &b_lines);
    let changes = engine_changes(&inp, false);
    let mut adds = 0usize;
    let mut dels = 0usize;
    for ch in &changes {
        if ch.del {
            if aw.contains(&ch.i) {
                dels += 1;
            }
        } else if bw.contains(&ch.j) {
            adds += 1;
        }
    }
    let mut hunks = Vec::new();
    for h in group_hunks(&changes, a_lines.len(), b_lines.len(), opts.ctx) {
        let intersects_a = h.a1 < aw.end && aw.start < h.a2;
        let intersects_b = h.b1 < bw.end && bw.start < h.b2;
        if intersects_a || intersects_b {
            hunks.push(Hunk {
                a1: h.a1.max(aw.start),
                a2: h.a2.min(aw.end),
                b1: h.b1.max(bw.start),
                b2: h.b2.min(bw.end),
            });
        }
    }
    DiffOut { hunks, adds, dels }
}

/// Snapshot diff: the T-01 prune walk first — the O(1) `subtree_equal`
/// fast path, then the DIVERGENCE ENVELOPE (the outer bounds of the walk's
/// diverged spans). Content outside the envelope is byte-identical shared
/// prefix/suffix and skips line processing entirely; inside the envelope
/// ONE global line diff runs (the same pipeline as [`diff_lines`]), so
/// snapshot results are alignment-identical to text diffs of the same
/// contents. (Per-region diffing around each diverged span would recover
/// more sharing for scattered edits, but region-local histogram alignment
/// diverges from the whole-document pass — recorded in SD-02 as the
/// future refinement.)
pub fn diff_snapshots(sa: &RopeSnapshot, sb: &RopeSnapshot, opts: DiffOpts) -> DiffOut {
    if sa.subtree_equal(sb) {
        return DiffOut::default();
    }
    let spans = sa.prune_spans(sb);
    let Some(&(a_lo, _, b_lo, _)) = spans.diverged.iter().min() else {
        return DiffOut::default();
    };
    let (a_hi, b_hi) = spans
        .diverged
        .iter()
        .map(|&(_, a2, _, b2)| (a2, b2))
        .max()
        .unwrap();
    let (la0, la1) = line_span(sa, a_lo, a_hi);
    let (lb0, lb1) = line_span(sb, b_lo, b_hi);
    if la0 == la1 && lb0 == lb1 {
        return DiffOut::default();
    }
    let a_lines: Vec<String> = (la0..la1).map(|li| strip_cr(&sa.line(li))).collect();
    let b_lines: Vec<String> = (lb0..lb1).map(|li| strip_cr(&sb.line(li))).collect();
    let a_refs: Vec<&str> = a_lines.iter().map(String::as_str).collect();
    let b_refs: Vec<&str> = b_lines.iter().map(String::as_str).collect();
    let inp = intern_lines(&a_refs, &b_refs);
    let local = engine_changes(&inp, false);
    let adds = local.iter().filter(|c| !c.del).count();
    let dels = local.iter().filter(|c| c.del).count();
    let rebased: Vec<Change> = local.into_iter().map(|c| Change { del: c.del, i: c.i + la0, j: c.j + lb0 }).collect();
    let hunks = group_hunks(&rebased, sa.line_count(), sb.line_count(), opts.ctx);
    DiffOut { hunks, adds, dels }
}
fn strip_cr(line: &str) -> String {
    line.strip_suffix('\r').unwrap_or(line).to_string()
}

/// Line span (exclusive end) covering the byte range `[start, end)` — the
/// diverged byte span widened to whole lines.
fn line_span(s: &RopeSnapshot, start: usize, end: usize) -> (usize, usize) {
    let l0 = s.byte_to_point(start).0;
    let l1 = if end >= s.len_bytes() {
        s.line_count()
    } else {
        let (l, c) = s.byte_to_point(end);
        if c == 0 {
            l
        } else {
            l + 1
        }
    };
    (l0, l1.max(l0))
}

// ---------------------------------------------------------------------------
// T-03: inline character refinement
// ---------------------------------------------------------------------------

/// Char-level prefix/suffix refinement of a PAIRED row — the three-segment
/// marking of the downstream envelope contract (`lpre/lmid/lpost` vs
/// `rpre/rmid/rpost`): the shared head and tail are factored out; the mid
/// ranges are the highlighted remainder. Char-indexed (rows carry char
/// substrings). Unpaired rows use the whole line as mid — that decision
/// belongs to the envelope builder (Q-4 form).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refinement {
    /// Common prefix length in chars.
    pub pre: usize,
    /// Common suffix length in chars.
    pub post: usize,
}

impl Refinement {
    /// a-side mid char range given the line's char length.
    pub fn a_mid(&self, a_chars: usize) -> Range<usize> {
        self.pre..a_chars - self.post
    }
    /// b-side mid char range given the line's char length.
    pub fn b_mid(&self, b_chars: usize) -> Range<usize> {
        self.pre..b_chars - self.post
    }
}

pub fn refine_inline(a_line: &str, b_line: &str) -> Refinement {
    let a: Vec<char> = a_line.chars().collect();
    let b: Vec<char> = b_line.chars().collect();
    let mut pre = 0usize;
    while pre < a.len() && pre < b.len() && a[pre] == b[pre] {
        pre += 1;
    }
    let mut post = 0usize;
    while post < a.len() - pre && post < b.len() - pre && a[a.len() - 1 - post] == b[b.len() - 1 - post] {
        post += 1;
    }
    Refinement { pre, post }
}

// ---------------------------------------------------------------------------
// Tests (PLAN-703 T-02/T-03; golden family from supply pack §1 + AC-02..04)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::core::rope::Rope;
    use super::*;

    #[test]
    fn split_lines_universal_matches_downstream_contract() {
        assert_eq!(split_lines_universal("a\nb"), vec!["a", "b"]);
        assert_eq!(split_lines_universal("a\nb\n"), vec!["a", "b"], "trailing newline absorbed");
        assert_eq!(split_lines_universal("a\r\nb\r\n"), vec!["a", "b"], "CR tolerated");
        assert_eq!(split_lines_universal(""), Vec::<&str>::new());
        assert_eq!(split_lines_universal("\n"), vec![""], "lone newline = one empty line");
        assert_eq!(split_lines_universal("a\rb\n"), vec!["a\rb"], "lone CR kept (only CRLF stripped)");
        assert_eq!(split_lines_universal("a\r"), vec!["a"], "trailing lone CR stripped (split-on-LF semantics)");
    }

    // ── golden eight shapes (supply pack §1) ─────────────────────────────

    #[test]
    fn golden_pure_insertion() {
        // ctx=1 keeps the golden tight (ctx=3 would legitimately absorb the
        // whole 3-line document — expansion clamps to the file bounds).
        let out = diff_lines("a\nb\nc\n", "a\nb\nX\nc\n", DiffOpts { ctx: 1 });
        assert_eq!(out.hunks, vec![Hunk { a1: 1, a2: 3, b1: 1, b2: 4 }]);
        assert_eq!(out.adds, 1);
        assert_eq!(out.dels, 0);
    }

    #[test]
    fn golden_pure_deletion() {
        let out = diff_lines("a\nb\nX\nc\n", "a\nb\nc\n", DiffOpts { ctx: 1 });
        // Del of X at a-line 2: a-window [2-1, 2+1+1)=[1,4); the del keeps
        // the b cursor at 2 → b-window [2-1, 2+1)=[1,3) (ctx rows b/c).
        assert_eq!(out.hunks, vec![Hunk { a1: 1, a2: 4, b1: 1, b2: 3 }]);
        assert_eq!(out.adds, 0);
        assert_eq!(out.dels, 1);
    }

    #[test]
    fn golden_modification() {
        let out = diff_lines("a\nb\nc\n", "a\nB\nc\n", DiffOpts { ctx: 1 });
        assert_eq!(out.hunks, vec![Hunk { a1: 0, a2: 3, b1: 0, b2: 3 }]);
        assert_eq!((out.adds, out.dels), (1, 1));
    }

    #[test]
    fn golden_line_move_recognized_by_histogram() {
        // A moved line pairs as keep+keep (histogram's human-readable
        // alignment), not a full replace.
        let a = "alpha\nbeta\ngamma\ndelta\n";
        let b = "delta\nalpha\nbeta\ngamma\n";
        let out = diff_lines(a, b, DiffOpts::default());
        assert_eq!(out.adds + out.dels, 2, "a move = one del + one add: {:?}", out.hunks);
    }

    #[test]
    fn golden_both_empty() {
        let out = diff_lines("", "", DiffOpts::default());
        assert!(out.is_empty());
        assert_eq!((out.adds, out.dels), (0, 0));
    }

    #[test]
    fn golden_empty_vs_content() {
        let out = diff_lines("", "x\n", DiffOpts::default());
        assert_eq!(out.hunks, vec![Hunk { a1: 0, a2: 0, b1: 0, b2: 1 }]);
        let out = diff_lines("x\n", "", DiffOpts::default());
        assert_eq!(out.hunks, vec![Hunk { a1: 0, a2: 1, b1: 0, b2: 0 }]);
    }

    #[test]
    fn golden_single_line_large() {
        let big = "x".repeat(200_000);
        let big2 = format!("{big}!");
        let out = diff_lines(&big, &big2, DiffOpts::default());
        assert_eq!(out.hunks.len(), 1);
        assert_eq!(out.hunks[0], Hunk { a1: 0, a2: 1, b1: 0, b2: 1 });
    }

    #[test]
    fn golden_identical() {
        let out = diff_lines("same\ntext\n", "same\ntext\n", DiffOpts::default());
        assert!(out.is_empty());
    }

    /// AC-02 determinism: same input twice → identical hunks.
    #[test]
    fn determinism_double_run_byte_equal() {
        let a = "fn main() {\n    let x = 1;\n}\n\nfn helper() {}\nfn extra() {}\n";
        let b = "fn main() {\n    let y = 2;\n    let z = 3;\n}\n\nfn helper() {}\n";
        let r1 = format!("{:?}", diff_lines(a, b, DiffOpts::default()));
        let r2 = format!("{:?}", diff_lines(a, b, DiffOpts::default()));
        assert_eq!(r1, r2);
    }

    /// Multi-hunk document: grouping with ctx, clamping at the bounds.
    #[test]
    fn grouping_ctx_merge_and_clamp() {
        let mut a = String::new();
        let mut b = String::new();
        for i in 0..60 {
            a.push_str(&format!("L{i}\n"));
            if i == 10 {
                b.push_str("INS-A\n");
            }
            b.push_str(&format!("L{i}\n"));
            if i == 50 {
                b.push_str("INS-B\n");
            }
        }
        let out = diff_lines(&a, &b, DiffOpts::default());
        assert_eq!(out.hunks.len(), 2, "far-apart inserts stay separate: {:?}", out.hunks);
        let (h0, h1) = (out.hunks[0], out.hunks[1]);
        // Semantic form: each hunk contains its insert line with ctx=3
        // margins (exact field goldens live in the eight-shape family).
        // INS-A sits at b-line 10; INS-B follows L50 at b-line 52.
        assert!(h0.b1 <= 10 && 10 < h0.b2, "h0 covers INS-A's b line: {h0:?}");
        assert!(h0.a2 - h0.a1 <= 8 && h0.b2 - h0.b1 <= 8, "ctx=3 margins: {h0:?}");
        assert!(h1.b1 <= 52 && 52 < h1.b2, "h1 covers INS-B's b line: {h1:?}");
        assert!(h1.a2 - h1.a1 <= 8 && h1.b2 - h1.b1 <= 8, "ctx=3 margins: {h1:?}");
    }

    // ── AC-04 window projection consistency ──────────────────────────────

    #[test]
    fn windowed_projection_matches_full_result_clipped() {
        let mut a = String::new();
        let mut b = String::new();
        for i in 0..120 {
            a.push_str(&format!("L{i}\n"));
            if i % 17 == 0 {
                b.push_str(&format!("X{i}\n"));
            }
            b.push_str(&format!("L{i}\n"));
        }
        let full = diff_lines(&a, &b, DiffOpts::default());
        // Sample multiple window positions.
        for &(s, e) in &[(0usize, 10usize), (10, 40), (40, 41), (50, 90), (90, 120), (119, 120)] {
            let win = diff_lines_windowed(&a, &b, DiffOpts::default(), s..e, s..e);
            let want: Vec<Hunk> = full
                .hunks
                .iter()
                .filter(|h| (h.a1 < e && s < h.a2) || (h.b1 < e && s < h.b2))
                .map(|h| Hunk {
                    a1: h.a1.max(s),
                    a2: h.a2.min(e),
                    b1: h.b1.max(s),
                    b2: h.b2.min(e),
                })
                .collect();
            assert_eq!(win.hunks, want, "window [{s},{e}) projection");
        }
        // Fully disjoint window → empty.
        let win = diff_lines_windowed(&a, &b, DiffOpts::default(), 0..0, 0..0);
        assert!(win.is_empty());
    }

    // ── AC-05 parallel ≡ serial ──────────────────────────────────────────

    #[test]
    fn parallel_equals_serial() {
        // Large document with periodic unique anchor lines so the partition
        // produces several segments.
        let mut a = String::new();
        let mut b = String::new();
        for i in 0..4000 {
            a.push_str(&format!("common line body {}/3\n", i % 3));
            if i % 500 == 0 {
                a.push_str(&format!("anchor {i}\n"));
                b.push_str(&format!("anchor {i}\n"));
                b.push_str(&format!("changed {i}\n"));
            } else {
                b.push_str(&format!("common line body {}/3\n", i % 3));
            }
        }
        let serial = diff_lines(&a, &b, DiffOpts::default());
        let par = diff_lines_parallel(&a, &b, DiffOpts::default());
        assert_eq!(serial, par, "thread count must not change the result");
        assert!(!serial.is_empty());
    }

    // ── snapshot diff via the T-01 prune walk ────────────────────────────

    #[test]
    fn snapshot_diff_matches_text_diff_on_edited_copy() {
        let mut model = String::new();
        for i in 0..800 {
            model.push_str(&format!("line {i} of a reasonably long document\n"));
        }
        let mut rope = Rope::from_str(&model);
        let s0 = rope.snapshot();
        let edit_at = rope.line_start_byte(400);
        rope.insert_bytes(edit_at, "INSERTED LINE\n");
        rope.delete_bytes(edit_at + 100, edit_at + 110);
        let s1 = rope.snapshot();

        let out = diff_snapshots(&s0, &s1, DiffOpts::default());
        // Reference: text-level diff of the two frozen contents. Change
        // COUNTS must agree exactly (line identity is objective); hunk
        // boundaries may legitimately differ because the snapshot path
        // diffs prune-walk regions with region-local histograms, whose
        // alignment choices can split/merge hunks differently than the
        // whole-document pass (SD-02 records this boundary).
        let text_out = diff_lines(&s0.to_string(), &s1.to_string(), DiffOpts::default());
        assert_eq!((out.adds, out.dels), (text_out.adds, text_out.dels), "change counts");
        assert!(!out.hunks.is_empty());
        for w in out.hunks.windows(2) {
            assert!(w[0].a2 <= w[1].a1 && w[0].b2 <= w[1].b1, "hunks ordered, non-overlapping");
        }

        // Identical snapshots → O(1) empty.
        assert!(diff_snapshots(&s1, &s1, DiffOpts::default()).is_empty());
        assert!(diff_snapshots(&s0, &s0, DiffOpts::default()).is_empty());
    }

    #[test]
    fn snapshot_diff_handles_append_and_truncate() {
        let mut rope = Rope::from_str("a\nb\nc\n");
        let s0 = rope.snapshot();
        rope.insert_bytes(rope.len_bytes(), "d\ne\n");
        let s1 = rope.snapshot();
        let out = diff_snapshots(&s0, &s1, DiffOpts::default());
        assert_eq!(out.adds, 2);
        assert_eq!(out.dels, 0);
        // Reverse direction (shorter other side).
        let out = diff_snapshots(&s1, &s0, DiffOpts::default());
        assert_eq!(out.adds, 0);
        assert_eq!(out.dels, 2);
    }

    // ── T-03 refinement ──────────────────────────────────────────────────

    #[test]
    fn refinement_prefix_suffix_trim() {
        let r = refine_inline("let x = 1;", "let x = 2;");
        assert_eq!(r.pre, 8, "common head 'let x = '");
        assert_eq!(r.post, 1, "common tail ';'");
        assert_eq!(r.a_mid(10), 8..9);
        assert_eq!(r.b_mid(10), 8..9);

        // Full replacement: no common head/tail.
        let r = refine_inline("abc", "xyz");
        assert_eq!((r.pre, r.post), (0, 0));
        assert_eq!(r.a_mid(3), 0..3);

        // Identical lines: pre covers everything, mids empty.
        let r = refine_inline("same", "same");
        assert_eq!(r.pre, 4);
        assert_eq!(r.a_mid(4), 4..4);
    }

    #[test]
    fn refinement_char_indexed_multibyte() {
        // Char-indexed mids over multibyte content.
        let r = refine_inline("中文中", "中文中!");
        assert_eq!(r.pre, 3);
        assert_eq!(r.post, 0);
        assert_eq!(r.b_mid(4), 3..4);
    }
}

#[cfg(test)]
mod p703_dbg {
    use super::*;
    #[test]
    fn probe_imara() {
        let a = ["a", "b", "c"];
        let b = ["a", "b", "X", "c"];
        let inp = intern_lines(&a, &b);
        println!("before={:?} after={:?}", inp.before.iter().map(|t| t.0).collect::<Vec<_>>(), inp.after.iter().map(|t| t.0).collect::<Vec<_>>());
        let mut d = ImaraDiff::default();
        d.compute_with(Algorithm::Histogram, &inp.before, &inp.after, 4);
        let rem: Vec<bool> = (0..3).map(|i| d.is_removed(i)).collect();
        let add: Vec<bool> = (0..4).map(|i| d.is_added(i)).collect();
        println!("removed={rem:?} added={add:?}");
        let hunks: Vec<_> = d.hunks().collect();
        println!("imara hunks={hunks:?}");
    }
}

// ── PLAN-703 T-04: 100MB-class synthetic benchmarks (AC-05) ────────────────
//
// #[ignore]-gated (the gallery-fence pattern): run explicitly with
//   cargo test --release -p auto-lang diff_bench -- --ignored --nocapture
// Debug builds hash/compare ~10x slower and would drown the relative signal.
// AC-05 judges RELATIVE quantities only — the absolute ≤2s verdict belongs
// to the downstream auto-edit L2 bench (plan §10 Q-2). Numbers land in
// docs/specs/auto-lang/ui/design/diff-engine.md (SD-02).

#[cfg(test)]
mod diff_bench {
    use super::*;
    use super::super::core::rope::Rope;
    use std::time::Instant;

    /// Deterministic pseudo-random filler so the shapes are reproducible.
    /// The line text carries its index (a realistic near-unique line
    /// alphabet — a 997-value dictionary made every line repeat ~1000×,
    /// killing all patience anchors and pushing the dense shape into the
    /// histogram→Myers pathological fallback for wall-clock hours).
    fn filler(seed: usize, i: usize) -> String {
        let h = (i.wrapping_mul(2654435761).wrapping_add(seed * 0x9E3779B9)) % 997;
        format!("fn item_{i}_{h}(arg: u32) -> u32 {{ arg.wrapping_mul({h}) + {seed} }}")
    }

    /// `changed_pct` = share of b lines that differ from their a position.
    fn build_pair(target_bytes: usize, changed_pct: usize) -> (String, String) {
        let line = filler(1, 0);
        let approx = target_bytes / (line.len() + 1);
        let mut a = String::with_capacity(target_bytes + 64);
        let mut b = String::with_capacity(target_bytes + 64);
        for i in 0..approx {
            a.push_str(&filler(1, i));
            a.push('\n');
            if i % 100 < changed_pct {
                b.push_str(&filler(2, i));
            } else {
                b.push_str(&filler(1, i));
            }
            b.push('\n');
        }
        (a, b)
    }

    fn run_shape(name: &str, target_bytes: usize, changed_pct: usize) {
        let t0 = Instant::now();
        let (a, b) = build_pair(target_bytes, changed_pct);
        let built = t0.elapsed();
        let t1 = Instant::now();
        let serial = diff_lines(&a, &b, DiffOpts::default());
        let serial_ms = t1.elapsed().as_millis();
        let t2 = Instant::now();
        let par = diff_lines_parallel(&a, &b, DiffOpts::default());
        let par_ms = t2.elapsed().as_millis();
        assert_eq!(serial, par, "{name}: parallel must equal serial");
        println!(
            "BENCH {name}: {} bytes / {} lines | serial {serial_ms} ms | parallel {par_ms} ms | hunks {} adds {} dels {} | build {built:?}",
            a.len(),
            a.len() / 40,
            serial.hunks.len(),
            serial.adds,
            serial.dels,
        );
    }

    #[test]
    #[ignore = "100MB-class benchmark — run with cargo test --release -p auto-lang diff_bench -- --ignored --nocapture"]
    fn bench_diff_scale_100mb() {
        // 100MB ≈ 2.5M lines at ~40 bytes/line. The fully-changed shape
        // runs at 10MB: with a near-unique alphabet it is a pure O(n)
        // replace, and the all-repeat pathological variant is already
        // documented as the histogram→Myers fallback boundary.
        for &(mb, pct) in &[(100usize, 0usize), (100, 1), (100, 10), (10, 100)] {
            run_shape(&format!("{mb}MB changed={pct}%"), mb * 1024 * 1024, pct);
        }
    }

    #[test]
    #[ignore = "smaller ladder for quick relative checks"]
    fn bench_diff_scale_ladder() {
        for &mb in &[1usize, 10] {
            for &pct in &[0usize, 10] {
                run_shape(&format!("{mb}MB changed={pct}%"), mb * 1024 * 1024, pct);
            }
        }
    }

    /// Snapshot-path prune evidence: the envelope skips everything outside
    /// the edited region; the identical case is O(1) via subtree_equal.
    #[test]
    #[ignore = "snapshot prune-share evidence — run with the release bench"]
    fn bench_snapshot_prune_share() {
        let mut model = String::new();
        for i in 0..1_500_000 {
            model.push_str(&filler(1, i));
            model.push('\n');
        }
        let mut rope = Rope::from_str(&model);
        let s0 = rope.snapshot();
        // Identical: O(1) equal fast path.
        let t0 = Instant::now();
        assert!(diff_snapshots(&s0, &s0, DiffOpts::default()).is_empty());
        println!("BENCH snapshot identical ({} bytes): {:?}", model.len(), t0.elapsed());
        // Single edit mid-document: envelope ≈ the edit's neighbourhood.
        let at = rope.line_start_byte(700_000);
        rope.insert_bytes(at, "INSERTED LINE\n");
        let s1 = rope.snapshot();
        let t1 = Instant::now();
        let out = diff_snapshots(&s0, &s1, DiffOpts::default());
        println!(
            "BENCH snapshot single-edit ({} bytes): {:?} | hunks {} adds {} dels {}",
            model.len(),
            t1.elapsed(),
            out.hunks.len(),
            out.adds,
            out.dels
        );
        assert_eq!((out.adds, out.dels), (1, 0));
    }
}

// ── PLAN-704: D-2 defect-fix regression (anchor monotonicity) ──────────────
//
// Downstream registration: auto-edit docs/upstream/2026-09-diff-engine-
// supply.md §6.2 (PLAN-016 execution-era finding). The unique-per-side
// anchor set of a block swap spans three blocks whose b positions bounce;
// the segment builder used to walk them as monotone and corrupt the edit
// script (620-line swap → "620 adds / 0 dels", both directions).

#[cfg(test)]
mod plan704_d2 {
    use super::*;

    fn numbered(tag: &str, n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("{tag}{i}")).collect()
    }

    fn swap_pair() -> (Vec<String>, Vec<String>) {
        let mut a = numbered("P", 5);
        a.extend(numbered("A", 300));
        a.extend(numbered("M", 10));
        a.extend(numbered("B", 300));
        a.extend(numbered("S", 5));
        let mut b = numbered("P", 5);
        b.extend(numbered("B", 300));
        b.extend(numbered("M", 10));
        b.extend(numbered("A", 300));
        b.extend(numbered("S", 5));
        (a, b)
    }

    fn joined(v: &[String]) -> String {
        let mut s = v.join("\n");
        s.push('\n');
        s
    }

    #[test]
    fn d2_anchor_partition_monotone_on_reorder() {
        let (a, b) = swap_pair();
        let a_mid: Vec<&str> = a[5..615].iter().map(|s| s.as_str()).collect();
        let b_mid: Vec<&str> = b[5..615].iter().map(|s| s.as_str()).collect();
        let inp = intern_lines(&a_mid, &b_mid);
        let anchors = anchor_partition(&inp.before, &inp.after, interned_len(&inp));
        assert_eq!(anchors.len(), 300, "LIS keeps exactly one swapped block (A or B)");
        assert!(
            anchors.windows(2).all(|w| w[0].0 < w[1].0 && w[0].1 < w[1].1),
            "anchors strictly monotone on both coordinates"
        );
    }

    #[test]
    fn d2_swap_edit_script_counts_and_symmetry() {
        let (a, b) = swap_pair();
        let (at, bt) = (joined(&a), joined(&b));
        let ab = diff_lines(&at, &bt, DiffOpts::default());
        let ba = diff_lines(&bt, &at, DiffOpts::default());
        // kept accounting: len - dels == len - adds == alignment size (320 =
        // P5 + one anchored block 300 + M10 + S5); the script itself correct.
        assert_eq!((ab.adds, ab.dels), (310, 310), "swap script correct, not 620/0");
        assert_eq!((ba.adds, ba.dels), (310, 310), "symmetric both directions");
        assert_eq!(620 - ab.dels, 620 - ab.adds, "kept-side accounting invariant");
        assert_eq!(620 - ba.dels, 620 - ba.adds);
    }
}
