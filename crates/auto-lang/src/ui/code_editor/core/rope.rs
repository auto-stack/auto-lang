// Plan 673 T-04 — self-implemented lightweight rope: the document source of
// truth for the editor kernel (design §3.1/§3.2). No new crate dependencies.
//
// Balance strategy: AVL-style height-balanced binary rope. Leaves hold text
// chunks (target `LEAF_TARGET_BYTES`, merge while under `LEAF_MAX_BYTES`);
// `concat` absorbs a small side into the adjacent leaf of a deeper sibling
// and rebalances with single/double rotations when the child height
// difference exceeds 1 — locate/split/insert/delete are O(log n) worst case,
// not amortized. Every node caches a summary (bytes / chars / newlines /
// first-line chars / last-line chars / height), giving O(1) root accessors
// and O(log n) byte↔point conversion.
//
// Line convention: `line_count = newlines + 1` (a trailing `'\n'` opens a
// new empty line), matching the existing editor's `lines.join("\n")` view of
// the cosmic buffer — an empty document is ONE empty line, and line `i`
// excludes the terminating `'\n'`. This mirrors Rust's `str::lines()` for
// non-empty text and keeps `to_string()`/`line(i)` consistent with
// `text.lines().collect()` up to the final-empty-line convention.
//
// Persistence: nodes are immutable behind `Arc`; mutating ops copy only the
// spine (O(log n) nodes), so `Rope::snapshot()` is a cheap root-handle clone
// and old snapshots keep seeing their frozen content while the main rope is
// edited (snapshot isolation for background parse/search/diff/save readers,
// design §3.1). COW serves snapshot isolation only — no OT/CRDT merging.
//
// Byte offsets must be char boundaries (documented precondition):
// `debug_assert!` checks at the kernel level; the .at-facing endpoint
// validates and reports (报错不静默) before calling in.

use std::borrow::Cow;
use std::sync::Arc;

/// Target leaf payload. Oversized inserted text is split toward this size.
const LEAF_TARGET_BYTES: usize = 1024;
/// Two leaves merge into one on concat while the combined payload stays
/// under this; a lone leaf at or under this size is never split.
const LEAF_MAX_BYTES: usize = 4096;

#[derive(Debug)]
enum Node {
    Leaf {
        text: String,
        chars: usize,
        newlines: usize,
    },
    Internal {
        left: Arc<Node>,
        right: Arc<Node>,
        bytes: usize,
        chars: usize,
        newlines: usize,
        /// Chars before the first `'\n'` (first line's length in chars).
        start_chars: usize,
        /// Chars after the last `'\n'` (last line's length in chars).
        end_chars: usize,
        height: u8,
    },
}

impl Node {
    fn leaf(text: String) -> Arc<Node> {
        let chars = text.chars().count();
        let newlines = text.bytes().filter(|&b| b == b'\n').count();
        Arc::new(Node::Leaf { text, chars, newlines })
    }

    fn internal(left: Arc<Node>, right: Arc<Node>) -> Arc<Node> {
        let bytes = left.bytes() + right.bytes();
        let chars = left.chars() + right.chars();
        let newlines = left.newlines() + right.newlines();
        let start_chars = if left.newlines() > 0 {
            left.start_chars()
        } else {
            left.chars() + right.start_chars()
        };
        let end_chars = if right.newlines() > 0 {
            right.end_chars()
        } else {
            right.chars() + left.end_chars()
        };
        let height = left.height().max(right.height()) + 1;
        Arc::new(Node::Internal { left, right, bytes, chars, newlines, start_chars, end_chars, height })
    }

    fn bytes(&self) -> usize {
        match self {
            Node::Leaf { text, .. } => text.len(),
            Node::Internal { bytes, .. } => *bytes,
        }
    }

    fn chars(&self) -> usize {
        match self {
            Node::Leaf { chars, .. } | Node::Internal { chars, .. } => *chars,
        }
    }

    fn newlines(&self) -> usize {
        match self {
            Node::Leaf { newlines, .. } | Node::Internal { newlines, .. } => *newlines,
        }
    }

    fn start_chars(&self) -> usize {
        match self {
            Node::Leaf { text, .. } => text.chars().take_while(|&c| c != '\n').count(),
            Node::Internal { start_chars, .. } => *start_chars,
        }
    }

    fn end_chars(&self) -> usize {
        match self {
            Node::Leaf { text, .. } => text.chars().rev().take_while(|&c| c != '\n').count(),
            Node::Internal { end_chars, .. } => *end_chars,
        }
    }

    fn height(&self) -> u8 {
        match self {
            Node::Leaf { .. } => 0,
            Node::Internal { height, .. } => *height,
        }
    }

    /// Whether `index` is the start of a char (or the end of the text).
    fn is_char_boundary(&self, index: usize) -> bool {
        if index == 0 || index == self.bytes() {
            return true;
        }
        // A byte offset is a boundary iff the byte AT the offset is not a
        // UTF-8 continuation byte (0b10xxxxxx).
        let mut remaining = index;
        let mut cur = self;
        loop {
            match cur {
                Node::Leaf { text, .. } => return text.as_bytes()[remaining] & 0xC0 != 0x80,
                Node::Internal { left, right, .. } => {
                    if remaining < left.bytes() {
                        cur = left;
                    } else {
                        remaining -= left.bytes();
                        cur = right;
                    }
                }
            }
        }
    }

    fn children(&self) -> (&Arc<Node>, &Arc<Node>) {
        match self {
            Node::Internal { left, right, .. } => (left, right),
            Node::Leaf { .. } => unreachable!("children of a leaf"),
        }
    }
}

fn leaf_text(node: &Node) -> &str {
    match node {
        Node::Leaf { text, .. } => text,
        Node::Internal { .. } => unreachable!("leaf_text on an internal node"),
    }
}

/// Build a balanced tree from `text`, splitting oversized payloads toward
/// `LEAF_TARGET_BYTES` leaves on char boundaries.
fn leaf_or_tree(text: &str) -> Arc<Node> {
    if text.len() <= LEAF_TARGET_BYTES {
        return Node::leaf(text.to_string());
    }
    let mut mid = text.len() / 2;
    while !text.is_char_boundary(mid) {
        mid -= 1;
    }
    Node::internal(leaf_or_tree(&text[..mid]), leaf_or_tree(&text[mid..]))
}

/// Concatenate two ropes/trees, merging small leaves (the "squeeze"
/// heuristic) and rebalancing like an AVL tree. Shares untouched subtrees.
fn concat(a: Arc<Node>, b: Arc<Node>) -> Arc<Node> {
    // Two leaves that fit in one: merge.
    if let (Node::Leaf { text: at, .. }, Node::Leaf { text: bt, .. }) = (&*a, &*b) {
        if at.len() + bt.len() <= LEAF_MAX_BYTES {
            let mut merged = String::with_capacity(at.len() + bt.len());
            merged.push_str(at);
            merged.push_str(bt);
            return Node::leaf(merged);
        }
    }
    // Small left leaf + deeper right tree: absorb into the right's leftmost
    // leaf instead of stacking a useless level.
    if let Node::Leaf { text, .. } = &*a {
        if let Node::Internal { .. } = &*b {
            let (bl, br) = b.children();
            if text.len() + bl.bytes() <= LEAF_MAX_BYTES && matches!(&**bl, Node::Leaf { .. }) {
                let mut merged = String::with_capacity(text.len() + bl.bytes());
                merged.push_str(text);
                merged.push_str(leaf_text(bl));
                return rebalance(Node::internal(Node::leaf(merged), br.clone()));
            }
        }
    }
    // Symmetric: deeper left tree + small right leaf.
    if let Node::Internal { .. } = &*a {
        if let Node::Leaf { text, .. } = &*b {
            let (al, ar) = a.children();
            if text.len() + ar.bytes() <= LEAF_MAX_BYTES && matches!(&**ar, Node::Leaf { .. }) {
                let mut merged = String::with_capacity(text.len() + ar.bytes());
                merged.push_str(leaf_text(ar));
                merged.push_str(text);
                return rebalance(Node::internal(al.clone(), Node::leaf(merged)));
            }
        }
    }
    rebalance(Node::internal(a, b))
}

/// AVL rebalance: single/double rotations when the child height difference
/// exceeds 1. Rotations rebuild only O(1) nodes; summaries recompute bottom
/// up through `Node::internal`.
fn rebalance(node: Arc<Node>) -> Arc<Node> {
    if matches!(&*node, Node::Leaf { .. }) {
        return node;
    }
    let (left, right) = node.children();
    let (left, right) = (left.clone(), right.clone());
    let balance = left.height() as i32 - right.height() as i32;
    if balance > 1 {
        // Left deeper. Double rotation if the inner (right) side is deeper.
        let (ll, lr) = left.children();
        let (ll, lr) = (ll.clone(), lr.clone());
        if lr.height() > ll.height() {
            let rl = rotate_left(ll, lr);
            let (rll, rlr) = rl.children();
            Node::internal(rll.clone(), Node::internal(rlr.clone(), right))
        } else {
            Node::internal(ll, Node::internal(lr, right))
        }
    } else if balance < -1 {
        let (rl, rr) = right.children();
        let (rl, rr) = (rl.clone(), rr.clone());
        if rl.height() > rr.height() {
            let rr2 = rotate_right(rl, rr);
            let (rl2, rr2) = rr2.children();
            Node::internal(Node::internal(left, rl2.clone()), rr2.clone())
        } else {
            Node::internal(Node::internal(left, rl), rr)
        }
    } else {
        node
    }
}

/// `N(l, N(rl, rr))` → `N(N(l, rl), rr)`.
fn rotate_left(l: Arc<Node>, r: Arc<Node>) -> Arc<Node> {
    let (rl, rr) = r.children();
    Node::internal(Node::internal(l, rl.clone()), rr.clone())
}

/// `N(N(ll, lr), r)` → `N(ll, N(lr, r))`.
fn rotate_right(l: Arc<Node>, r: Arc<Node>) -> Arc<Node> {
    let (ll, lr) = l.children();
    Node::internal(ll.clone(), Node::internal(lr.clone(), r))
}

/// Split into `[0, byte)` and `[byte, len)`. Shares untouched subtrees with
/// the original — persistence-preserving (path copy only).
fn split(node: &Arc<Node>, byte: usize) -> (Arc<Node>, Arc<Node>) {
    match &**node {
        Node::Leaf { text, .. } => {
            let (l, r) = text.split_at(byte);
            (Node::leaf(l.to_string()), Node::leaf(r.to_string()))
        }
        Node::Internal { left, right, .. } => {
            if byte < left.bytes() {
                let (ll, lr) = split(left, byte);
                (ll, concat(lr, right.clone()))
            } else {
                let (rl, rr) = split(right, byte - left.bytes());
                (concat(left.clone(), rl), rr)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Query engine (free fns over a root, shared by Rope and RopeSnapshot).
// ---------------------------------------------------------------------------

/// (line, char column) for a byte offset — O(log n) locate + O(leaf).
fn q_byte_to_point(root: &Node, byte: usize) -> (usize, usize) {
    debug_assert!(byte <= root.bytes(), "byte_to_point past end");
    let mut line = 0usize;
    let mut col = 0usize;
    let mut remaining = byte;
    let mut cur = root;
    loop {
        match cur {
            Node::Leaf { text, .. } => {
                let head = &text[..remaining];
                line += head.bytes().filter(|&b| b == b'\n').count();
                col = if head.contains('\n') {
                    head.chars().rev().take_while(|&c| c != '\n').count()
                } else {
                    col + head.chars().count()
                };
                return (line, col);
            }
            Node::Internal { left, right, .. } => {
                if remaining < left.bytes() {
                    cur = left;
                } else {
                    remaining -= left.bytes();
                    line += left.newlines();
                    // After crossing a subtree with newlines, the column
                    // restarts from its last line's char length.
                    col = if left.newlines() > 0 { left.end_chars() } else { col + left.chars() };
                    cur = right;
                }
            }
        }
    }
}

/// Byte offset for (line, char column). None when the line is out of range
/// or `char_col` is past the line's last char (the `'\n'` is not part of
/// the line). O(log n) locate + O(char_col) advance — touching each char up
/// to the target is inherent to char-point → byte conversion.
fn q_point_to_byte(root: &Node, line: usize, char_col: usize) -> Option<usize> {
    if line >= root.newlines() + 1 {
        return None;
    }
    let start = q_line_start_byte(root, line);
    let end = q_line_end_byte(root, line);
    let line_text = q_slice_bytes(root, start, end);
    let mut byte = start;
    let mut consumed = 0usize;
    for c in line_text.chars() {
        if consumed == char_col {
            return Some(byte);
        }
        consumed += 1;
        byte += c.len_utf8();
    }
    if consumed == char_col {
        Some(byte)
    } else {
        None
    }
}

/// Byte offset where line `line` starts (just after the previous `'\n'`).
/// Precondition: `line < line_count`.
fn q_line_start_byte(root: &Node, line: usize) -> usize {
    let line_count = root.newlines() + 1;
    assert!(line < line_count, "line_start_byte: line {line} out of range ({line_count} lines)");
    if line == 0 {
        return 0;
    }
    // Find newline #(line-1) (0-based) and land just past it.
    let target_nl = line - 1;
    let mut node_start = 0usize;
    let mut nls_before = 0usize;
    let mut cur = root;
    loop {
        match cur {
            Node::Leaf { text, .. } => {
                let mut remaining = target_nl - nls_before;
                for (i, c) in text.char_indices() {
                    if c == '\n' {
                        if remaining == 0 {
                            return node_start + i + 1;
                        }
                        remaining -= 1;
                    }
                }
                unreachable!("line_start_byte: newline not found (bounds checked)");
            }
            Node::Internal { left, right, .. } => {
                if target_nl < nls_before + left.newlines() {
                    cur = left;
                } else {
                    node_start += left.bytes();
                    nls_before += left.newlines();
                    cur = right;
                }
            }
        }
    }
}

/// Byte offset where line `line`'s text ends (exclusive of the terminating
/// `'\n'`). Precondition: `line < line_count`.
fn q_line_end_byte(root: &Node, line: usize) -> usize {
    let line_count = root.newlines() + 1;
    if line + 1 < line_count {
        q_line_start_byte(root, line + 1) - 1 // skip the '\n'
    } else {
        root.bytes()
    }
}

/// Text in `[start, end)` — borrowed when the span sits in one leaf
/// (zero-copy for the common case), gathered otherwise.
/// O(log n) locate + O(span).
fn q_slice_bytes<'a>(root: &'a Node, start: usize, end: usize) -> Cow<'a, str> {
    debug_assert!(start <= end && end <= root.bytes(), "slice_bytes out of range");
    let mut node_start = 0usize;
    let mut cur = root;
    loop {
        match cur {
            Node::Leaf { text, .. } => {
                let a = start - node_start;
                let b = end - node_start;
                return Cow::Borrowed(&text[a..b]);
            }
            Node::Internal { left, right, .. } => {
                if end - node_start <= left.bytes() {
                    cur = left;
                } else if start - node_start >= left.bytes() {
                    node_start += left.bytes();
                    cur = right;
                } else {
                    // Span crosses the split: gather (O(k)).
                    let mut out = String::with_capacity(end - start);
                    gather(cur, start - node_start, end - node_start, &mut out);
                    return Cow::Owned(out);
                }
            }
        }
    }
}

fn gather(node: &Node, start: usize, end: usize, out: &mut String) {
    match node {
        Node::Leaf { text, .. } => out.push_str(&text[start..end]),
        Node::Internal { left, right, .. } => {
            let lb = left.bytes();
            if start < lb {
                gather(left, start, end.min(lb), out);
            }
            if end > lb {
                gather(right, start.saturating_sub(lb), end - lb, out);
            }
        }
    }
}

fn q_line<'a>(root: &'a Node, i: usize) -> Cow<'a, str> {
    q_slice_bytes(root, q_line_start_byte(root, i), q_line_end_byte(root, i))
}

fn q_to_string(root: &Node) -> String {
    let mut out = String::with_capacity(root.bytes());
    gather(root, 0, root.bytes(), &mut out);
    out
}

// ---------------------------------------------------------------------------
// Public handles
// ---------------------------------------------------------------------------

/// Persistent rope: the editor-kernel document store (design §3.1).
/// Cheap to snapshot; shareable across threads (`Send + Sync`).
#[derive(Debug, Clone)]
pub struct Rope {
    root: Arc<Node>,
}

impl Default for Rope {
    fn default() -> Self {
        Self { root: Arc::new(Node::Leaf { text: String::new(), chars: 0, newlines: 0 }) }
    }
}

impl Rope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(text: &str) -> Self {
        Self { root: leaf_or_tree(text) }
    }

    /// O(1) frozen view; further edits to this rope leave it untouched.
    pub fn snapshot(&self) -> RopeSnapshot {
        RopeSnapshot { root: self.root.clone() }
    }

    /// Document length in UTF-8 bytes. O(1) (root summary).
    pub fn len_bytes(&self) -> usize {
        self.root.bytes()
    }
    /// Document length in chars (点). O(1) (root summary).
    pub fn len_chars(&self) -> usize {
        self.root.chars()
    }
    /// `newlines + 1` — see the module-level line convention note. O(1).
    pub fn line_count(&self) -> usize {
        self.root.newlines() + 1
    }
    pub fn byte_to_point(&self, byte: usize) -> (usize, usize) {
        q_byte_to_point(&self.root, byte)
    }
    pub fn point_to_byte(&self, line: usize, char_col: usize) -> Option<usize> {
        q_point_to_byte(&self.root, line, char_col)
    }
    pub fn line_start_byte(&self, line: usize) -> usize {
        q_line_start_byte(&self.root, line)
    }
    pub fn line_end_byte(&self, line: usize) -> usize {
        q_line_end_byte(&self.root, line)
    }
    pub fn slice_bytes(&self, start: usize, end: usize) -> Cow<'_, str> {
        q_slice_bytes(&self.root, start, end)
    }
    pub fn line(&self, i: usize) -> Cow<'_, str> {
        q_line(&self.root, i)
    }
    pub fn to_string(&self) -> String {
        q_to_string(&self.root)
    }

    /// Iterate lines 0..line_count (each without its terminating `'\n'`).
    pub fn lines(&self) -> impl Iterator<Item = Cow<'_, str>> + '_ {
        let root = &*self.root;
        let n = root.newlines() + 1;
        (0..n).map(move |i| q_line(root, i))
    }

    /// Insert `text` at char-boundary byte `offset`.
    /// O(log n) locate + O(k) local work + O(log n) spine rebuild.
    ///
    /// Precondition: `offset` is a char boundary (kernel level asserts only;
    /// the .at-facing endpoint validates and reports).
    pub fn insert_bytes(&mut self, offset: usize, text: &str) {
        self.replace_bytes(offset, offset, text);
    }

    /// Delete `[start, end)`. Same preconditions/complexity as `insert_bytes`.
    pub fn delete_bytes(&mut self, start: usize, end: usize) {
        self.replace_bytes(start, end, "");
    }

    /// Replace `[start, end)` with `text`. Same preconditions/complexity.
    pub fn replace_bytes(&mut self, start: usize, end: usize, text: &str) {
        debug_assert!(start <= end && end <= self.len_bytes(), "replace_bytes out of range");
        debug_assert!(self.root.is_char_boundary(start), "replace_bytes: start not a char boundary");
        debug_assert!(self.root.is_char_boundary(end), "replace_bytes: end not a char boundary");
        let (left, right) = split(&self.root, end);
        let (left, _) = split(&left, start);
        self.root = concat(concat(left, leaf_or_tree(text)), right);
    }
}

/// Read-only frozen view of a rope at one point in time (design §3.1
/// snapshot isolation: background parse/search/diff/save readers hold this
/// while the main rope keeps editing — zero lock contention, O(1) to take).
#[derive(Debug, Clone)]
pub struct RopeSnapshot {
    root: Arc<Node>,
}

impl RopeSnapshot {
    pub fn len_bytes(&self) -> usize {
        self.root.bytes()
    }
    pub fn len_chars(&self) -> usize {
        self.root.chars()
    }
    /// `newlines + 1` — see the module-level line convention note. O(1).
    pub fn line_count(&self) -> usize {
        self.root.newlines() + 1
    }
    pub fn byte_to_point(&self, byte: usize) -> (usize, usize) {
        q_byte_to_point(&self.root, byte)
    }
    pub fn point_to_byte(&self, line: usize, char_col: usize) -> Option<usize> {
        q_point_to_byte(&self.root, line, char_col)
    }
    pub fn line_start_byte(&self, line: usize) -> usize {
        q_line_start_byte(&self.root, line)
    }
    pub fn line_end_byte(&self, line: usize) -> usize {
        q_line_end_byte(&self.root, line)
    }
    pub fn slice_bytes(&self, start: usize, end: usize) -> Cow<'_, str> {
        q_slice_bytes(&self.root, start, end)
    }
    pub fn line(&self, i: usize) -> Cow<'_, str> {
        q_line(&self.root, i)
    }
    pub fn to_string(&self) -> String {
        q_to_string(&self.root)
    }
    pub fn lines(&self) -> impl Iterator<Item = Cow<'_, str>> + '_ {
        let root = &*self.root;
        let n = root.newlines() + 1;
        (0..n).map(move |i| q_line(root, i))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// xorshift64* — tiny seeded PRNG, no dev-deps.
    struct Rng(u64);

    impl Rng {
        fn new(seed: u64) -> Self {
            Rng(seed.max(1))
        }
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545F4914F6CDD1D)
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n.max(1) as u64) as usize
        }
    }

    /// Alphabet exercising 1/2/3/4-byte UTF-8 plus newlines and spaces.
    const POOL: &[&str] = &[
        "a", "b", "Z", "0", " ", "~", "\n", "\n", "é", "中", "文", "🦀", "🎉", "ß", "fn ", "{", "}", "\t",
    ];

    fn random_text(rng: &mut Rng, max_chars: usize) -> String {
        let n = rng.below(max_chars);
        let mut s = String::new();
        for _ in 0..n {
            s.push_str(POOL[rng.below(POOL.len())]);
        }
        s
    }

    /// Char-boundary byte offsets of `s`, ascending (0 and len included).
    fn boundaries(s: &str) -> Vec<usize> {
        std::iter::once(0).chain(s.char_indices().map(|(i, c)| i + c.len_utf8())).collect()
    }

    fn assert_invariants(rope: &Rope, model: &str, ctx: &str) {
        assert_eq!(rope.to_string(), model, "content mismatch ({ctx})");
        assert_eq!(rope.len_bytes(), model.len(), "len_bytes ({ctx})");
        assert_eq!(rope.len_chars(), model.chars().count(), "len_chars ({ctx})");
        let want_lines = model.bytes().filter(|&b| b == b'\n').count() + 1;
        assert_eq!(rope.line_count(), want_lines, "line_count ({ctx})");
        // Root summaries agree with the ground truth.
        assert_eq!(rope.line(0), model.lines().next().unwrap_or(""), "line(0) ({ctx})");
    }

    /// Randomized differential test vs `String` — the correctness gate.
    /// On failure the seed + op log pin the repro.
    fn differential(seed: u64, ops: usize) {
        let mut rng = Rng::new(seed);
        let mut model = String::new();
        let mut rope = Rope::new();
        let mut log = String::new();
        for op_i in 0..ops {
            let bounds = boundaries(&model);
            let at = bounds[rng.below(bounds.len())];
            let kind = rng.below(3);
            let text = random_text(&mut rng, 12);
            let desc = if kind == 0 {
                model.insert_str(at, &text);
                rope.insert_bytes(at, &text);
                format!("insert({at}, {text:?})")
            } else if kind == 1 && at < model.len() {
                let pos = bounds.iter().position(|&b| b == at).unwrap();
                let end = bounds[(pos + 1 + rng.below(bounds.len() - pos - 1)).min(bounds.len() - 1)];
                model.replace_range(at..end, "");
                rope.delete_bytes(at, end);
                format!("delete({at}, {end})")
            } else {
                let end = if at < model.len() {
                    let pos = bounds.iter().position(|&b| b == at).unwrap();
                    bounds[(pos + 1 + rng.below(bounds.len() - pos - 1)).min(bounds.len() - 1)]
                } else {
                    at
                };
                model.replace_range(at..end, &text);
                rope.replace_bytes(at, end, &text);
                format!("replace({at}, {end}, {text:?})")
            };
            log = format!("{log}\n{op_i}: {desc}");
            let ctx = format!("seed={seed} op={op_i}\nops:{log}");
            assert_invariants(&rope, &model, &ctx);

            // Random point roundtrip at a random char boundary.
            let bounds = boundaries(&model);
            let b = bounds[rng.below(bounds.len())];
            let (line, col) = rope.byte_to_point(b);
            assert_eq!(
                rope.point_to_byte(line, col),
                Some(b),
                "point roundtrip failed at byte {b} (line {line} col {col}) seed={seed} op={op_i}"
            );
        }
    }

    #[test]
    fn differential_seeded_batch() {
        for seed in [1u64, 42, 1337, 99_991] {
            differential(seed, 600);
        }
    }

    #[test]
    fn boundary_every_char_position() {
        // Multi-byte soup: exercise every char-boundary edit position with
        // insert / delete / replace, including 0 and EOF.
        let base = "a中🦀b\né";
        for &pos in &boundaries(base) {
            let mut r = Rope::from_str(base);
            r.insert_bytes(pos, "X中");
            assert_eq!(r.to_string(), format!("{}X中{}", &base[..pos], &base[pos..]));

            let mut r = Rope::from_str(base);
            if pos < base.len() {
                let next = boundaries(base).into_iter().find(|&b| b > pos).unwrap();
                r.delete_bytes(pos, next);
                assert_eq!(r.to_string(), format!("{}{}", &base[..pos], &base[next..]));
            }

            let mut r = Rope::from_str(base);
            let end = boundaries(base).into_iter().find(|&b| b > pos).unwrap_or(base.len());
            r.replace_bytes(pos, end, "🎉");
            assert_eq!(r.to_string(), format!("{}🎉{}", &base[..pos], &base[end..]));
        }
    }

    #[test]
    fn empty_rope_ops() {
        let mut r = Rope::new();
        assert_eq!(r.to_string(), "");
        assert_eq!(r.line_count(), 1, "empty document is ONE empty line (lines.join convention)");
        assert_eq!(r.line(0), "");
        r.insert_bytes(0, "hi");
        assert_eq!(r.to_string(), "hi");
        r.insert_bytes(2, "\n");
        assert_eq!(r.line_count(), 2);
        r.delete_bytes(0, 3);
        assert_eq!(r.to_string(), "");
        assert_eq!(r.line_count(), 1);
    }

    #[test]
    fn edit_at_zero_and_eof() {
        let mut r = Rope::from_str("abc");
        r.insert_bytes(0, "🦀");
        assert_eq!(r.to_string(), "🦀abc");
        r.insert_bytes(r.len_bytes(), "é");
        assert_eq!(r.to_string(), "🦀abcé");
        r.delete_bytes(0, r.len_bytes());
        assert_eq!(r.to_string(), "");
    }

    #[test]
    fn consecutive_and_trailing_newlines() {
        let mut r = Rope::from_str("a\n\nb");
        assert_eq!(r.line_count(), 3);
        assert_eq!(r.line(1), "");
        r.insert_bytes(3, "\n\n");
        assert_eq!(r.to_string(), "a\n\n\n\nb");
        assert_eq!(r.line_count(), 5);
        // Trailing newline opens a new empty line.
        r.insert_bytes(r.len_bytes(), "\n");
        assert_eq!(r.line_count(), 6);
        assert_eq!(r.line(5), "");
        assert_eq!(r.line_end_byte(4), r.len_bytes() - 1);
    }

    #[test]
    fn point_queries_multibyte_columns() {
        let text = "ab中\n🦀éx";
        let r = Rope::from_str(text);
        assert_eq!(r.line_count(), 2);
        let mut byte = 0;
        let mut col = 0;
        let mut line = 0;
        for c in text.chars() {
            assert_eq!(r.byte_to_point(byte), (line, col), "byte {byte}");
            assert_eq!(r.point_to_byte(line, col), Some(byte));
            byte += c.len_utf8();
            col += 1;
            if c == '\n' {
                line += 1;
                col = 0;
            }
        }
        assert_eq!(r.byte_to_point(byte), (line, col), "EOF byte");
        assert_eq!(r.point_to_byte(line, col), Some(byte));
        // Col == chars-in-line is the position AT the line's terminating
        // '\n' (a valid cursor spot — byte_to_point roundtrips it); past
        // that → None (the '\n' itself is not part of the line).
        assert_eq!(r.point_to_byte(0, 3), Some(5));
        assert_eq!(r.point_to_byte(0, 4), None);
        assert_eq!(r.point_to_byte(1, 3), Some(13)); // == len: col 3 is the line's end
        assert_eq!(r.point_to_byte(1, 4), None);
        assert_eq!(r.point_to_byte(2, 0), None, "line out of range");
    }

    #[test]
    fn snapshots_stay_frozen() {
        let mut r = Rope::from_str("hello world");
        let s1 = r.snapshot();
        r.insert_bytes(5, " brave");
        let s2 = r.snapshot();
        r.delete_bytes(0, 12); // drop "hello brave "
        let s3 = r.snapshot();
        r.replace_bytes(0, 0, "🦀");

        assert_eq!(s1.to_string(), "hello world", "s1 frozen");
        assert_eq!(s2.to_string(), "hello brave world", "s2 frozen");
        assert_eq!(s3.to_string(), "world", "s3 frozen");
        assert_eq!(r.to_string(), "🦀world");

        // Query API on snapshots reflects the frozen content.
        assert_eq!(s2.line_count(), 1);
        assert_eq!(s2.byte_to_point(6), (0, 6));
        assert_eq!(s2.slice_bytes(6, 11), "brave");
        assert_eq!(s3.len_bytes(), 5);
        assert_eq!(s3.line(0), "world");
    }

    #[test]
    fn snapshot_thread_isolation() {
        // Nodes are immutable behind Arc (COW copies only the spine), so a
        // snapshot moved to another thread reads a stable document while the
        // main rope keeps mutating — the design §3.1 background-reader shape.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Rope>();
        assert_send_sync::<RopeSnapshot>();

        let mut r = Rope::from_str("line zero\n");
        let snap = r.snapshot();
        let handle = std::thread::spawn(move || {
            for i in 0..200 {
                assert_eq!(snap.line_count(), 2, "snapshot line_count drifted at {i}");
                assert_eq!(snap.line(0), "line zero");
                assert_eq!(snap.to_string(), "line zero\n");
            }
        });
        for i in 1..200 {
            r.insert_bytes(r.len_bytes(), &format!("line {i}\n"));
        }
        handle.join().unwrap();
        assert_eq!(r.line_count(), 201);
    }

    #[test]
    fn slice_and_line_cow() {
        let r = Rope::from_str("alpha\nbeta\ngamma");
        assert!(matches!(r.slice_bytes(0, 5), Cow::Borrowed("alpha")));
        assert_eq!(r.slice_bytes(3, 12), "ha\nbeta\ng");
        assert_eq!(r.line(1), "beta");
        let owned: Vec<String> = r.lines().map(|l| l.into_owned()).collect();
        assert_eq!(owned, vec!["alpha", "beta", "gamma"]);
        // Line past a trailing newline: last line is empty.
        let r2 = Rope::from_str("x\n");
        assert_eq!(r2.line_count(), 2);
        assert_eq!(r2.line(1), "");
    }

    #[test]
    fn scale_sanity_large_document() {
        // ~1M lines (~11MB) — far above leaf sizes, peak well under the 100MB
        // light-pool budget (text + rope + one to_string copy). Generous
        // debug-build margins; the point is that locate/insert stay fast and
        // root summaries are instant.
        let line = "let x = 1;\n";
        let n = 1_000_000usize;
        let mut text = String::with_capacity(line.len() * n);
        for _ in 0..n {
            text.push_str(line);
        }
        let t0 = std::time::Instant::now();
        let mut rope = Rope::from_str(&text);
        assert_eq!(rope.line_count(), n + 1);
        assert!(t0.elapsed().as_secs() < 30, "build took {:?}", t0.elapsed());

        let t1 = std::time::Instant::now();
        let mid = rope.line_start_byte(n / 2);
        assert_eq!(rope.byte_to_point(mid), (n / 2, 0));
        assert!(t1.elapsed().as_millis() < 1000, "byte_to_point took {:?}", t1.elapsed());

        let t2 = std::time::Instant::now();
        rope.insert_bytes(mid, "// inserted\n");
        rope.delete_bytes(mid, mid + 12);
        assert_eq!(rope.to_string(), text, "edits must cancel out");
        assert!(t2.elapsed().as_secs() < 10, "mid edits took {:?}", t2.elapsed());

        // O(1) summaries: repeated reads are instant.
        let t3 = std::time::Instant::now();
        for _ in 0..10_000 {
            std::hint::black_box(rope.line_count());
            std::hint::black_box(rope.len_bytes());
        }
        assert!(t3.elapsed().as_millis() < 500, "root summaries not O(1): {:?}", t3.elapsed());
    }
}
