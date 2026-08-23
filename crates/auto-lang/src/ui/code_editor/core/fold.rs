// Plan 428 P1 — code folding: brace-paired region discovery and the
// FoldMap projection (merged hidden ranges, y prefix sums, hit testing).
//
// Folding is pure view state: a folded opener hides its body lines
// [opener + 1, end] — INCLUDING the closing `}` line — behind the opener.
// All geometry (selection, caret, search, scrolling) keeps ORIGINAL line
// coordinates; the map only projects y for drawing and un-projects click
// y for hit testing, so the mapping surface stays minimal (Plan 428 §6(c)).
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

use std::collections::BTreeSet;

/// A foldable block: the opener line (trimmed text ends with `{` and
/// net-opens a nesting level) plus the line of its matching `}`. The
/// hidden body is the inclusive range `[opener + 1, end]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldRegion {
    /// 0-based index of the opening line.
    pub opener: usize,
    /// 0-based index of the matching close (the `}` line).
    pub end: usize,
}

/// All foldable regions plus the merged hidden ranges of the folded ones.
/// `render` builds one per frame; natives/tests build fresh ones headless.
#[derive(Clone, Debug, Default)]
pub struct FoldMap {
    /// Every foldable region in the buffer (folded or not) — the gutter's
    /// chevron set. Kept sorted by opener (discovery scans in order).
    regions: Vec<FoldRegion>,
    /// Disjoint inclusive hidden line ranges, sorted ascending.
    hidden: Vec<(usize, usize)>,
    /// Line height of the y projection (hit testing reads it directly).
    pub line_height: f32,
}

/// Discover foldable `{ … }` blocks from raw line texts (no shaping).
///
/// Heuristics (Plan 428 §7.1): a line whose trimmed text ends with `{`
/// opens a region only when it net-raises the brace depth — `} else {`
/// nets 0 and is skipped; a block whose closing line never comes (EOF)
/// is not foldable. Nesting is discovered naturally: every depth-raising
/// opener gets its own region down to its matching close. Braces inside
/// strings/comments are not parsed (same heuristic class as Phase A).
pub fn regions_from_texts(texts: &[&str]) -> Vec<FoldRegion> {
    let n = texts.len();
    // Cumulative brace depth after each line.
    let mut after = vec![0i64; n];
    let mut depth: i64 = 0;
    for (k, line) in texts.iter().enumerate() {
        for ch in line.trim().chars() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
            }
        }
        after[k] = depth;
    }

    let mut regions = Vec::new();
    for i in 0..n {
        if !texts[i].trim().ends_with('{') {
            continue;
        }
        let before = if i == 0 { 0 } else { after[i - 1] };
        if after[i] <= before {
            continue; // `} else {` and friends: no net new level
        }
        // Matching close: the first later line that returns the depth to
        // the pre-opener level (<= also catches imbalanced early dips).
        let Some(end) = (i + 1..n).find(|&j| after[j] <= before) else {
            continue; // unclosed at EOF — not foldable
        };
        regions.push(FoldRegion { opener: i, end });
    }
    regions
}

impl FoldMap {
    /// Merge the bodies of the `folded` openers into disjoint hidden
    /// ranges. Nested folds' bodies overlap and collapse into the outer
    /// range; `line_height` feeds the y projection.
    pub fn build(regions: Vec<FoldRegion>, folded: &BTreeSet<usize>, line_height: f32) -> FoldMap {
        let mut bodies: Vec<(usize, usize)> = folded
            .iter()
            .filter_map(|opener| regions.iter().find(|r| r.opener == *opener))
            .map(|r| (r.opener + 1, r.end))
            .collect();
        bodies.sort_unstable();
        let mut hidden: Vec<(usize, usize)> = Vec::new();
        for (a, b) in bodies {
            match hidden.last_mut() {
                Some(last) if a <= last.1.saturating_add(1) => {
                    if b > last.1 {
                        last.1 = b;
                    }
                }
                _ => hidden.push((a, b)),
            }
        }
        FoldMap { regions, hidden, line_height }
    }

    /// The region opened by `opener`, if the line is foldable.
    pub fn region_at(&self, opener: usize) -> Option<&FoldRegion> {
        self.regions.iter().find(|r| r.opener == opener)
    }

    /// Whether `line` is hidden inside a folded body.
    pub fn is_hidden(&self, line: usize) -> bool {
        self.hidden.iter().any(|&(a, b)| a <= line && line <= b)
    }

    /// Total number of hidden lines (merged ranges; nested not double-cut).
    pub fn hidden_count(&self) -> usize {
        self.hidden.iter().map(|&(a, b)| b - a + 1).sum()
    }

    /// The merged hidden range containing `line`, if any.
    pub fn hidden_range_containing(&self, line: usize) -> Option<(usize, usize)> {
        self.hidden.iter().copied().find(|&(a, b)| a <= line && line <= b)
    }

    /// Hidden lines strictly above `line` — the y prefix-sum step. If
    /// `line` itself is hidden (caller misuse), stops at its top.
    pub fn hidden_above(&self, line: usize) -> usize {
        let mut n = 0;
        for &(a, b) in &self.hidden {
            if b < line {
                n += b - a + 1;
            } else if a < line {
                n += line - a;
            }
        }
        n
    }

    /// Project an original-space y (top of `line`) into folded-view space.
    pub fn project_y(&self, line: usize, orig_y: f32) -> f32 {
        orig_y - self.hidden_above(line) as f32 * self.line_height
    }

    /// Un-project a folded-view click y back into original space using the
    /// render's visible-line bands `(line, projected_top, original_top)`
    /// (sorted by projected y, no gaps — projection is continuous). The
    /// click lands on the line the user SAW; outside the bands the same
    /// offset is extrapolated (cosmic hit clamps to the buffer edges).
    /// Returns `None` when no bands exist (nothing rendered yet).
    pub fn unfold_y(&self, y: f32, bands: &[(usize, f32, f32)]) -> Option<f32> {
        let first = *bands.first()?;
        let last = *bands.last()?;
        if y < first.1 {
            return Some(first.2 + (y - first.1));
        }
        for &(_line, proj_top, orig_top) in bands {
            if y < proj_top + self.line_height {
                return Some(orig_top + (y - proj_top));
            }
        }
        Some(last.2 + (y - last.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Same shape as the integration fixture in `core/mod.rs`: two sibling
    /// `fn` blocks (openers at 0-based 1 and 5, bodies [2,4] and [6,7]).
    const SRC: &str = "// header
fn add(a int, b int) int {
    let s = a + b
    return s
}
fn sub(a int, b int) int {
    return a - b
}
// tail
";

    const LH: f32 = 19.0;

    fn texts_of(src: &str) -> Vec<&str> {
        src.lines().collect()
    }

    fn map_of(src: &str, folded: &[usize]) -> FoldMap {
        let texts = texts_of(src);
        let regions = regions_from_texts(&texts);
        let set: BTreeSet<usize> = folded.iter().copied().collect();
        FoldMap::build(regions, &set, LH)
    }

    #[test]
    fn discovers_brace_regions() {
        let regions = regions_from_texts(&texts_of(SRC));
        assert_eq!(
            regions,
            vec![
                FoldRegion { opener: 1, end: 4 },
                FoldRegion { opener: 5, end: 7 },
            ]
        );
    }

    #[test]
    fn else_arm_is_not_an_opener() {
        let src = "fn f() {
    if x {
        a
    } else {
        b
    }
}
";
        // `} else {` nets depth 0 → skipped; the if-arm still folds.
        let regions = regions_from_texts(&texts_of(src));
        assert_eq!(
            regions,
            vec![
                FoldRegion { opener: 0, end: 6 },
                FoldRegion { opener: 1, end: 5 },
            ]
        );
    }

    #[test]
    fn unclosed_brace_not_foldable() {
        let src = "fn f() {
    a
    b
";
        assert!(regions_from_texts(&texts_of(src)).is_empty());
        // Opener as the very last line is equally unclosed.
        let src2 = "x
fn f() {";
        assert!(regions_from_texts(&texts_of(src2)).is_empty());
    }

    #[test]
    fn nested_regions_discovered() {
        let src = "fn a() {
    if x {
        y
    }
    tail
}
";
        let regions = regions_from_texts(&texts_of(src));
        assert_eq!(
            regions,
            vec![
                FoldRegion { opener: 0, end: 5 },
                FoldRegion { opener: 1, end: 3 },
            ]
        );
    }

    #[test]
    fn build_merges_nested_bodies() {
        let src = "fn a() {
    if x {
        y
    }
    tail
}
";
        // Folding both the outer and inner openers hides the outer body
        // exactly once: [1,5] — the inner body is a subset.
        let map = map_of(src, &[0, 1]);
        assert_eq!(map.hidden, vec![(1, 5)]);
        assert_eq!(map.hidden_count(), 5);
    }

    #[test]
    fn is_hidden_bounds() {
        let map = map_of(SRC, &[1]);
        assert!(!map.is_hidden(0), "header visible");
        assert!(!map.is_hidden(1), "opener stays visible");
        assert!(map.is_hidden(2) && map.is_hidden(3) && map.is_hidden(4));
        assert!(!map.is_hidden(5), "line after the fold visible");
    }

    #[test]
    fn hidden_range_containing() {
        let map = map_of(SRC, &[1]);
        assert_eq!(map.hidden_range_containing(3), Some((2, 4)));
        assert_eq!(map.hidden_range_containing(2), Some((2, 4)));
        assert_eq!(map.hidden_range_containing(4), Some((2, 4)));
        assert_eq!(map.hidden_range_containing(1), None, "opener not hidden");
        assert_eq!(map.hidden_range_containing(0), None);
        assert_eq!(map.hidden_range_containing(5), None);
    }

    #[test]
    fn project_y_shifts_below_fold_only() {
        let map = map_of(SRC, &[1]);
        // Above and at the opener: unchanged.
        assert_eq!(map.project_y(0, 0.0), 0.0);
        assert_eq!(map.project_y(1, LH), LH);
        // Below the 3-line body: shifted up by 3 line heights.
        assert_eq!(map.project_y(5, 5.0 * LH), 5.0 * LH - 3.0 * LH);
    }

    #[test]
    fn unfold_y_roundtrips_visible_lines() {
        let map = map_of(SRC, &[1]);
        // Rebuild the render's bands: visible lines only, projected tops.
        let bands: Vec<(usize, f32, f32)> = (0..9)
            .filter(|&i| !map.is_hidden(i))
            .map(|i| {
                let orig = i as f32 * LH;
                (i, map.project_y(i, orig), orig)
            })
            .collect();
        // A click mid-band on every visible line maps back to that line.
        for &(line, proj_top, orig_top) in &bands {
            let y = proj_top + LH * 0.5;
            let back = map.unfold_y(y, &bands).expect("band hit maps");
            assert_eq!(back, orig_top + LH * 0.5, "line {line}");
        }
        // Below the last band: extrapolated with the same offset.
        let &(_, last_proj, last_orig) = bands.last().unwrap();
        let below = map.unfold_y(last_proj + 4.0 * LH, &bands).expect("below maps");
        assert_eq!(below, last_orig + 4.0 * LH);
    }

    #[test]
    fn no_folds_hide_nothing() {
        let map = map_of(SRC, &[]);
        assert_eq!(map.hidden_count(), 0);
        assert!(!map.is_hidden(2));
        assert_eq!(map.project_y(5, 95.0), 95.0, "identity projection");
        assert_eq!(map.unfold_y(42.0, &[]), None, "no bands → nothing mapped");
        // Regions still discovered — the gutter can draw chevrons.
        assert!(map.region_at(1).is_some() && map.region_at(5).is_some());
        assert!(map.region_at(0).is_none(), "comment line not foldable");
    }
}
