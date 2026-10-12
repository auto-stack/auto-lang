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
//
// Content digests (PLAN-703 T-01): every node caches a 64-bit composable
// polynomial digest (Mersenne-61 field) — leaves digest their text;
// internals apply the concatenation rule to the child digests. Digests ride
// the existing O(log n) edit path for free (every constructor recomputes).
// Semantics: CONTENT-DETERMINED (equal bytes yield equal digests regardless
// of tree shape; equal digests imply equal content up to the ~n/2^61
// random-input collision bound — non-adversarial inputs, fixed base for
// cross-run determinism), which turns "did these two subtrees diverge"
// into an O(1)/O(log n) Merkle check. The exact equality face is
// `subtree_equal`, which pairs the shortcuts (ptr_eq structural sharing,
// whole-node digest match) with an aligned exact walk — verdicts are
// proven, never guessed.
//
// File-backed pages (PLAN-728): a `Node::Chunk` is a leaf whose text lives
// in the immutable base FILE (an {store, page, range} descriptor plus
// prescan-computed summaries). The summary/digest machinery above works
// over chunks WITHOUT materializing them; text-resolving paths fault the
// page in through the store's LRU cache (see `rope/file_backing.rs`).
// Base pages are never modified in place — offsets stay true, which the
// merge-save copy path depends on.

use std::borrow::Cow;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) mod file_backing;

pub(crate) use file_backing::PageConfig;
pub use file_backing::{PageState, PageStore};

/// Target leaf payload. Oversized inserted text is split toward this size.
const LEAF_TARGET_BYTES: usize = 1024;
/// Two leaves merge into one on concat while the combined payload stays
/// under this; a lone leaf at or under this size is never split.
const LEAF_MAX_BYTES: usize = 4096;

// --- content digest primitives (PLAN-703 T-01) -----------------------------
//
// Composable polynomial digest over the Mersenne prime field 2^61-1:
// H(s) is CONTENT-DETERMINED (shape-independent — equal bytes digest equal
// regardless of how the rope partitions them into leaves) and CONCATENATION-
// COMPOSABLE: H(x concatenated y) = (H(x)-1)·B^|y| + H(y) mod P. The two
// properties together give O(1) subtree digests verifiable against the flat
// content (tests) and foldable across aligned range pieces (`range_hash`),
// with a random-input collision bound of ~n/2^61 (non-adversarial inputs —
// the base is a fixed constant, not per-process seeded, for determinism).

/// Field prime 2^61 − 1 (Mersenne).
const POLY_P: u64 = (1 << 61) - 1;
/// Fixed polynomial base (golden-ratio odd constant, reduced into the field
/// at compile time — all `mulmod` operands must stay below P for the
/// Mersenne fold to fit; constant, not seeded, for cross-run determinism).
const POLY_B: u64 = 0x9e37_79b9_7f4a_7c15 % POLY_P;

fn mulmod(a: u64, b: u64) -> u64 {
    // Mersenne fold: x = lo + hi·2^61 ≡ lo + hi (mod 2^61−1); the sum fits
    // 63 bits, but `lo` may itself equal P, so two conditional subtractions
    // normalize (s ≤ 2P → after two, s < P).
    let x = a as u128 * b as u128;
    let mut s = ((x & POLY_P as u128) + (x >> 61)) as u64;
    if s >= POLY_P {
        s -= POLY_P;
    }
    if s >= POLY_P {
        s -= POLY_P;
    }
    s
}

fn powmod(mut base: u64, mut exp: u64) -> u64 {
    let mut acc: u64 = 1;
    while exp > 0 {
        if exp & 1 == 1 {
            acc = mulmod(acc, base);
        }
        base = mulmod(base, base);
        exp >>= 1;
    }
    acc
}

/// Digest of a byte slice: h starts at the sentinel 1 (which makes leading
/// zero bytes significant), then h = h·B + byte per byte. Empty = 1.
fn poly_digest(bytes: &[u8]) -> u64 {
    let mut h: u64 = 1;
    for &b in bytes {
        h = mulmod(h, POLY_B) + b as u64;
        if h >= POLY_P {
            h -= POLY_P;
        }
    }
    h
}

/// Concatenation rule: digest(x appended y) from digest(x), digest(y), |y|.
fn combine_hash(left: u64, right: u64, right_bytes: usize) -> u64 {
    let l = if left >= 1 { left - 1 } else { POLY_P - 1 }; // strip the sentinel
    let h = mulmod(l, powmod(POLY_B, right_bytes as u64)) + right;
    if h >= POLY_P {
        h - POLY_P
    } else {
        h
    }
}

#[derive(Debug)]
enum Node {
    Leaf {
        text: String,
        chars: usize,
        newlines: usize,
        /// Content digest of the leaf text (PLAN-703 T-01).
        hash: u64,
    },
    /// File-backed page slice (PLAN-728): text lives in the immutable base
    /// file at `store.pages[page] + off`, length `len` bytes. Summaries are
    /// prescan- (whole page) or split- (sub-range) computed, so every
    /// summary/digest query answers without materializing; only
    /// text-resolving paths fault the page in through the LRU cache.
    Chunk {
        store: Arc<PageStore>,
        page: u32,
        off: u32,
        len: u32,
        chars: u32,
        newlines: u32,
        start_chars: u32,
        end_chars: u32,
        hash: u64,
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
        /// `combine_hash(left, right, bytes)` — pure child-derived digest
        /// (PLAN-703 T-01); recomputed on every spine rebuild at O(1).
        hash: u64,
    },
}

impl Node {
    fn leaf(text: String) -> Arc<Node> {
        let chars = text.chars().count();
        let newlines = text.bytes().filter(|&b| b == b'\n').count();
        let hash = poly_digest(text.as_bytes());
        Arc::new(Node::Leaf {
            text,
            chars,
            newlines,
            hash,
        })
    }

    /// Whole-page chunk — summaries come from the prescan `PageDesc`
    /// (zero IO; the page need not be resident).
    fn chunk_whole(store: Arc<PageStore>, page: u32) -> Arc<Node> {
        // Copy the descriptor (PageDesc is Copy) so the store Arc can move
        // into the node while the fields ride along.
        let d = *store.page_desc(page);
        Arc::new(Node::Chunk {
            store,
            page,
            off: 0,
            len: d.len,
            chars: d.chars,
            newlines: d.newlines,
            start_chars: d.start_chars,
            end_chars: d.end_chars,
            hash: d.hash,
        })
    }

    /// Sub-range chunk — summaries computed from the given slice (the
    /// caller resolves the page text; this never IOs on its own).
    fn chunk_from_slice(store: Arc<PageStore>, page: u32, off: u32, text: &str) -> Arc<Node> {
        debug_assert!(
            off as usize + text.len() <= store.page_desc(page).len as usize,
            "chunk slice exceeds its page"
        );
        Arc::new(Node::Chunk {
            store,
            page,
            off,
            len: text.len() as u32,
            chars: text.chars().count() as u32,
            newlines: text.bytes().filter(|&b| b == b'\n').count() as u32,
            start_chars: text.chars().take_while(|&c| c != '\n').count() as u32,
            end_chars: text.chars().rev().take_while(|&c| c != '\n').count() as u32,
            hash: poly_digest(text.as_bytes()),
        })
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
        let hash = combine_hash(left.hash(), right.hash(), right.bytes());
        Arc::new(Node::Internal {
            left,
            right,
            bytes,
            chars,
            newlines,
            start_chars,
            end_chars,
            height,
            hash,
        })
    }

    fn bytes(&self) -> usize {
        match self {
            Node::Leaf { text, .. } => text.len(),
            Node::Chunk { len, .. } => *len as usize,
            Node::Internal { bytes, .. } => *bytes,
        }
    }

    fn chars(&self) -> usize {
        match self {
            Node::Leaf { chars, .. } | Node::Internal { chars, .. } => *chars,
            Node::Chunk { chars, .. } => *chars as usize,
        }
    }

    fn newlines(&self) -> usize {
        match self {
            Node::Leaf { newlines, .. } | Node::Internal { newlines, .. } => *newlines,
            Node::Chunk { newlines, .. } => *newlines as usize,
        }
    }

    fn start_chars(&self) -> usize {
        match self {
            Node::Leaf { text, .. } => text.chars().take_while(|&c| c != '\n').count(),
            Node::Internal { start_chars, .. } => *start_chars,
            Node::Chunk { start_chars, .. } => *start_chars as usize,
        }
    }

    fn end_chars(&self) -> usize {
        match self {
            Node::Leaf { text, .. } => text.chars().rev().take_while(|&c| c != '\n').count(),
            Node::Internal { end_chars, .. } => *end_chars,
            Node::Chunk { end_chars, .. } => *end_chars as usize,
        }
    }

    fn height(&self) -> u8 {
        match self {
            Node::Leaf { .. } | Node::Chunk { .. } => 0,
            Node::Internal { height, .. } => *height,
        }
    }

    /// Cached content digest (PLAN-703 T-01). O(1).
    fn hash(&self) -> u64 {
        match self {
            Node::Leaf { hash, .. } | Node::Internal { hash, .. } => *hash,
            Node::Chunk { hash, .. } => *hash,
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
                Node::Chunk { .. } => {
                    let (store, page, off) = chunk_key(cur);
                    let text = fault_page(store, page);
                    let abs = off as usize + remaining;
                    return text.as_bytes()[abs] & 0xC0 != 0x80;
                }
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
            Node::Leaf { .. } | Node::Chunk { .. } => unreachable!("children of a leaf"),
        }
    }
}

fn chunk_key(node: &Node) -> (&Arc<PageStore>, u32, u32) {
    match node {
        Node::Chunk {
            store, page, off, ..
        } => (store, *page, *off),
        _ => unreachable!("chunk_key on a non-chunk node"),
    }
}

/// Resolve a chunk's page text (sync fault-in through the LRU cache).
/// IO/decode failure aborts the process with the cause — the base handle
/// is held open, so a mid-session failure means storage vanished; a rope
/// query cannot return a wrong answer silently.
fn fault_page(store: &Arc<PageStore>, page: u32) -> Arc<str> {
    match store.page_text(page) {
        Ok(text) => text,
        Err(e) => panic!("paged-rope: page {page} fault-in failed: {e}"),
    }
}

/// Borrowed-or-owned leaf text view — the uniform face the comparison
/// walks use over `Leaf` (borrowed, zero-copy) and `Chunk` (faulted page,
/// owned Arc). `None` for internal nodes.
enum LeafView<'a> {
    Mem(&'a str),
    Paged(Arc<str>),
}

impl LeafView<'_> {
    fn slice(&self, a: usize, b: usize) -> &str {
        match self {
            LeafView::Mem(s) => &s[a..b],
            LeafView::Paged(s) => &s[a..b],
        }
    }

    /// Byte view — the comparison walks cut leaves at the OTHER side's
    /// node edges, which are arbitrary byte positions in THIS side's
    /// content once shapes diverge over multi-byte text (a latent 703
    /// hazard that paged chunks surface deterministically). Comparisons
    /// are byte-exact anyway, so the walks slice bytes, not str.
    fn bytes(&self) -> &[u8] {
        match self {
            LeafView::Mem(s) => s.as_bytes(),
            LeafView::Paged(s) => s.as_bytes(),
        }
    }
}

/// Page-internal byte offset of a chunk node (0 for mem leaves) —
/// comparison walks carry NODE-relative offsets, page text is PAGE-relative.
fn chunk_off(node: &Node) -> usize {
    match node {
        Node::Chunk { off, .. } => *off as usize,
        _ => 0,
    }
}

fn leaf_view(node: &Node) -> Option<LeafView<'_>> {
    match node {
        Node::Leaf { text, .. } => Some(LeafView::Mem(text)),
        Node::Chunk { .. } => {
            let (store, page, _) = chunk_key(node);
            Some(LeafView::Paged(fault_page(store, page)))
        }
        Node::Internal { .. } => None,
    }
}

fn leaf_text(node: &Node) -> &str {
    match node {
        Node::Leaf { text, .. } => text,
        Node::Internal { .. } | Node::Chunk { .. } => unreachable!("leaf_text on a non-mem leaf"),
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
    if matches!(&*node, Node::Leaf { .. } | Node::Chunk { .. }) {
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
        // PLAN-728: split a chunk by slicing its page text ONCE (fault-in
        // of one page) and building two sub-range descriptors — the base
        // stays untouched; both halves keep page-backed residency.
        Node::Chunk {
            store, page, off, ..
        } => {
            let text = fault_page(store, *page);
            let abs = *off as usize;
            let (l, r) = text[abs..abs + node.bytes()].split_at(byte);
            (
                Node::chunk_from_slice(store.clone(), *page, *off, l),
                Node::chunk_from_slice(store.clone(), *page, *off + byte as u32, r),
            )
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
            Node::Chunk { off, .. } => {
                let view = leaf_view(cur).expect("chunk leaf view");
                let text = view.slice(*off as usize, *off as usize + remaining);
                line += text.bytes().filter(|&b| b == b'\n').count();
                col = if text.contains('\n') {
                    text.chars().rev().take_while(|&c| c != '\n').count()
                } else {
                    col + text.chars().count()
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
                    col = if left.newlines() > 0 {
                        left.end_chars()
                    } else {
                        col + left.chars()
                    };
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
    assert!(
        line < line_count,
        "line_start_byte: line {line} out of range ({line_count} lines)"
    );
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
            Node::Chunk { off, .. } => {
                let view = leaf_view(cur).expect("chunk leaf view");
                let text = view.slice(*off as usize, *off as usize + cur.bytes());
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
    debug_assert!(
        start <= end && end <= root.bytes(),
        "slice_bytes out of range"
    );
    let mut node_start = 0usize;
    let mut cur = root;
    loop {
        match cur {
            Node::Leaf { text, .. } => {
                let a = start - node_start;
                let b = end - node_start;
                return Cow::Borrowed(&text[a..b]);
            }
            Node::Chunk { off, .. } => {
                // Same node-relative cut as the leaf arm, but the text is a
                // faulted page — the borrow lives in the page cache's Arc,
                // not in `&root`, so the span comes back owned.
                let a = start - node_start + *off as usize;
                let b = end - node_start + *off as usize;
                let view = leaf_view(cur).expect("chunk leaf view");
                return Cow::Owned(view.slice(a, b).to_string());
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
        Node::Chunk { off, .. } => {
            let view = leaf_view(node).expect("chunk leaf view");
            out.push_str(view.slice(*off as usize + start, *off as usize + end));
        }
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

// --- content digest queries (PLAN-703 T-01) --------------------------------

/// Collect the aligned pieces covering `[start, end)`: fully-covered
/// subtrees contribute their cached digest at O(1) each; the at most two
/// straddling leaves hash their slice. Piece count is O(log n).
fn collect_digest_pieces(
    node: &Node,
    node_start: usize,
    start: usize,
    end: usize,
    pieces: &mut Vec<(u64, usize)>,
) {
    let node_end = node_start + node.bytes();
    if start <= node_start && node_end <= end {
        pieces.push((node.hash(), node.bytes()));
        return;
    }
    match node {
        Node::Leaf { text, .. } => {
            let a = start.max(node_start) - node_start;
            let b = end.min(node_end) - node_start;
            pieces.push((poly_digest(text[a..b].as_bytes()), b - a));
        }
        Node::Chunk { off, .. } => {
            let a = start.max(node_start) - node_start;
            let b = end.min(node_end) - node_start;
            let view = leaf_view(node).expect("chunk leaf view");
            let s = view.slice(*off as usize + a, *off as usize + b);
            pieces.push((poly_digest(s.as_bytes()), b - a));
        }
        Node::Internal { left, right, .. } => {
            let mid = node_start + left.bytes();
            if start < mid {
                collect_digest_pieces(left, node_start, start, end.min(mid), pieces);
            }
            if end > mid {
                collect_digest_pieces(right, mid, start.max(mid), end, pieces);
            }
        }
    }
}

/// Digest of the byte range `[start, end)` — pieces fold through the
/// concatenation rule ([`combine_hash`]), so the value equals the flat
/// content digest of the slice (shape-independent; the full range equals
/// the root digest). O(log n) pieces + O(leaf) boundary hashing. Empty
/// range = the empty digest (sentinel 1).
fn q_range_hash(root: &Node, start: usize, end: usize) -> u64 {
    debug_assert!(
        start <= end && end <= root.bytes(),
        "range_hash out of range"
    );
    let mut pieces = Vec::new();
    collect_digest_pieces(root, 0, start, end, &mut pieces);
    let mut acc = 1; // empty digest
    let mut first = true;
    for (h, b) in pieces {
        acc = if first { h } else { combine_hash(acc, h, b) };
        first = false;
    }
    acc
}

/// Exact content equality of aligned `[a_off, a_off+len)` vs
/// `[b_off, b_off+len)` spans. Shortcuts: `Arc::ptr_eq` (structural
/// sharing — the O(1) dividend) and whole-node digest+summary match (the
/// content-determined Merkle verdict, ~n/2^61 collision bound). Beyond
/// the shortcuts the walk is byte-exact — verdicts are proven, not guessed.
/// Cost: O(1) shared / O(divergence spine) typical.
fn range_content_equal(a: &Node, a_off: usize, b: &Node, b_off: usize, len: usize) -> bool {
    // One aligned work item: `[a_off, a_off+len)` on `a` vs the same
    // content span on `b`. Invariant: the span fits inside both nodes.
    // Iterative so a tail that still straddles a boundary re-enters the
    // loop instead of overflowing a partially-covering child.
    if len == 0 {
        return true;
    }
    let mut stack: Vec<(&Node, usize, &Node, usize, usize)> = vec![(a, a_off, b, b_off, len)];
    while let Some((a, a_off, b, b_off, len)) = stack.pop() {
        if len == 0 {
            continue;
        }
        if std::ptr::eq(a, b) && a_off == b_off {
            // Same Arc at the same offset: identical remaining bytes. (The
            // same Arc at different offsets still needs a content compare —
            // shifted structural sharing across edits.)
            continue;
        }
        if a.bytes() == len
            && b.bytes() == len
            && a.hash() == b.hash()
            && a.chars() == b.chars()
            && a.newlines() == b.newlines()
        {
            continue; // content-determined Merkle verdict
        }
        match (a, b) {
            (Node::Internal { .. }, Node::Internal { .. }) => {
                let (al, ar) = a.children();
                let (bl, br) = b.children();
                // Boundary of each side inside the span (0 = span starts at
                // or past the left child; len = span ends inside it).
                let ca = if a_off >= al.bytes() {
                    0
                } else {
                    (al.bytes() - a_off).min(len)
                };
                let cb = if b_off >= bl.bytes() {
                    0
                } else {
                    (bl.bytes() - b_off).min(len)
                };
                if ca == 0 {
                    stack.push((ar, a_off - al.bytes(), b, b_off, len));
                } else if cb == 0 {
                    stack.push((a, a_off, br, b_off - bl.bytes(), len));
                } else {
                    // Both boundaries interior: split at both, giving up to
                    // three single-child pieces per side.
                    let (c1, c2) = if ca <= cb { (ca, cb) } else { (cb, ca) };
                    // [0, c1): both sides in their left children.
                    stack.push((al, a_off, bl, b_off, c1));
                    // [c1, c2): the side with the smaller boundary is in its
                    // right child, the other still in its left.
                    if c2 > c1 {
                        if ca <= cb {
                            stack.push((ar, 0, bl, b_off + c1, c2 - c1));
                        } else {
                            stack.push((al, a_off + c1, br, 0, c2 - c1));
                        }
                    }
                    // [c2, len): both sides in their right children, each
                    // from its own boundary offset — non-empty whenever
                    // c2 < len even when c1 == c2 (empty at c2 == len is
                    // skipped by the loop guard).
                    stack.push((ar, c2 - ca, br, c2 - cb, len - c2));
                }
            }
            // At least one leaf-like side (mem Leaf or file-backed Chunk —
            // both expose a contiguous text view via `leaf_view`; PLAN-728).
            _ => {
                let (va, vb) = (leaf_view(a), leaf_view(b));
                match (va, vb) {
                    (Some(ta), Some(tb)) => {
                        let (oa, ob) = (chunk_off(a), chunk_off(b));
                        if &ta.bytes()[oa + a_off..oa + a_off + len]
                            != &tb.bytes()[ob + b_off..ob + b_off + len]
                        {
                            return false;
                        }
                    }
                    _ => {
                        // One side has no internal boundary: split at the
                        // internal side's child boundary (it lies strictly
                        // inside the span).
                        let leaf_is_a = matches!(a, Node::Leaf { .. } | Node::Chunk { .. });
                        let (leaf, l_off, inner, i_off) = if leaf_is_a {
                            (a, a_off, b, b_off)
                        } else {
                            (b, b_off, a, a_off)
                        };
                        let (il, ir) = inner.children();
                        let cut = if i_off >= il.bytes() {
                            0
                        } else {
                            (il.bytes() - i_off).min(len)
                        };
                        if cut == 0 {
                            // Span starts at/past the internal side's boundary.
                            if leaf_is_a {
                                stack.push((leaf, l_off, ir, i_off - il.bytes(), len));
                            } else {
                                stack.push((ir, i_off - il.bytes(), leaf, l_off, len));
                            }
                            continue;
                        }
                        if cut == len {
                            // Span ends inside the internal side's left child.
                            if leaf_is_a {
                                stack.push((leaf, l_off, il, i_off, len));
                            } else {
                                stack.push((il, i_off, leaf, l_off, len));
                            }
                            continue;
                        }
                        if leaf_is_a {
                            stack.push((leaf, l_off, il, i_off, cut));
                            stack.push((leaf, l_off + cut, ir, 0, len - cut));
                        } else {
                            stack.push((il, i_off, leaf, l_off, cut));
                            stack.push((ir, 0, leaf, l_off + cut, len - cut));
                        }
                    }
                }
            }
        }
    }
    true
}

/// Exact content equality of two whole subtrees — [`range_content_equal`]
/// over the roots. This is the `subtree_equal` query face (PLAN-703 T-01).
fn q_subtree_equal(a: &Node, b: &Node) -> bool {
    if std::ptr::eq(a, b) {
        return true;
    }
    if a.bytes() != b.bytes() || a.chars() != b.chars() || a.newlines() != b.newlines() {
        return false;
    }
    range_content_equal(a, 0, b, 0, a.bytes())
}

/// Length tails beyond the common prefix are divergence by definition (a
/// pure insertion/deletion at the tail end) — appended after the aligned
/// walk by both `prune_spans` faces.
fn append_length_tails(a: &Arc<Node>, b: &Arc<Node>, out: &mut PruneSpans) {
    let (a_len, b_len) = (a.bytes(), b.bytes());
    let common = a_len.min(b_len);
    if a_len > common {
        out.diverged.push((common, a_len, b_len, b_len));
    }
    if b_len > common {
        out.diverged.push((a_len, a_len, common, b_len));
    }
}

/// Aligned prune spans for diff preprocessing (PLAN-703 T-04 "剪后算"):
/// walks two roots in lockstep and classifies aligned byte ranges as
/// *shared* (ptr_eq structural sharing or Merkle-shortcut equal) or
/// *diverged* (needs line-level diffing). Shared spans are byte-identical
/// on both sides by construction; diverged spans carry the (a, b) ranges.
/// Shape divergences (different splits) split at content-aligned cuts so
/// shared tails on either side still register.
///
/// Consumed by the diff engine's snapshot preprocessing (PLAN-703 T-04).
#[derive(Debug, Default)]
pub(crate) struct PruneSpans {
    /// `(a_start, a_end, b_start, b_end)` — byte-identical content on both
    /// sides (positions may shift apart across edits — structural sharing
    /// means the same Arc can sit at different document offsets).
    pub shared: Vec<(usize, usize, usize, usize)>,
    /// `(a_start, a_end, b_start, b_end)` — content that may differ.
    pub diverged: Vec<(usize, usize, usize, usize)>,
}

fn collect_prune_spans(
    a: &Node,
    a_off: usize,
    b: &Node,
    b_off: usize,
    len: usize,
    out: &mut PruneSpans,
) {
    // Same aligned walk as `range_content_equal`, classifying pieces into
    // `shared` (Merkle shortcut / byte-equal leaf pieces) vs `diverged`
    // (content differs — needs line-level diffing). Items carry node-
    // relative offsets (a_rel/b_rel) AND absolute document positions
    // (a_abs/b_abs) — recorded spans must be absolute.
    if len == 0 {
        return;
    }
    let mut stack: Vec<(&Node, usize, usize, &Node, usize, usize, usize)> =
        vec![(a, a_off, 0, b, b_off, 0, len)];
    // NOTE: a_abs/b_abs below are the span's absolute start; the initial
    // call normalizes them to 0 (caller passes byte offsets relative to the
    // roots, which ARE document coordinates).
    while let Some((a, a_rel, a_abs, b, b_rel, b_abs, len)) = stack.pop() {
        if len == 0 {
            continue;
        }
        // NB: no ptr_eq shortcut here — the same Arc can sit at shifted
        // document positions across an edit, and the recorded absolute
        // coordinates then need shift-tracking this v1 walk does not do.
        // The whole-node Merkle shortcut below carries the pruning load
        // (content-determined digest + length ⇒ same content, wherever it
        // sits); everything else classifies by leaf comparison.
        if a.bytes() == len
            && b.bytes() == len
            && a.hash() == b.hash()
            && a.chars() == b.chars()
            && a.newlines() == b.newlines()
        {
            out.shared.push((a_abs, a_abs + len, b_abs, b_abs + len));
            continue;
        }
        match (a, b) {
            (Node::Internal { .. }, Node::Internal { .. }) => {
                let (al, ar) = a.children();
                let (bl, br) = b.children();
                let ca = if a_rel >= al.bytes() {
                    0
                } else {
                    (al.bytes() - a_rel).min(len)
                };
                let cb = if b_rel >= bl.bytes() {
                    0
                } else {
                    (bl.bytes() - b_rel).min(len)
                };
                if ca == 0 {
                    stack.push((ar, a_rel - al.bytes(), a_abs, b, b_rel, b_abs, len));
                } else if cb == 0 {
                    stack.push((a, a_rel, a_abs, br, b_rel - bl.bytes(), b_abs, len));
                } else {
                    let (c1, c2) = if ca <= cb { (ca, cb) } else { (cb, ca) };
                    // [0, c1): both sides in their left children.
                    stack.push((al, a_rel, a_abs, bl, b_rel, b_abs, c1));
                    if c2 > c1 {
                        // [c1, c2): the side with the smaller boundary has
                        // crossed into its right child (empty when the
                        // boundaries coincide).
                        if ca <= cb {
                            stack.push((ar, 0, a_abs + c1, bl, b_rel + c1, b_abs + c1, c2 - c1));
                        } else {
                            stack.push((al, a_rel + c1, a_abs + c1, br, 0, b_abs + c1, c2 - c1));
                        }
                    }
                    // [c2, len): both sides in their right children — each
                    // from its own boundary offset (c2 − ca / c2 − cb; zero
                    // on the side that crossed at c2). Non-empty whenever
                    // c2 < len even when c1 == c2; empty at c2 == len is
                    // skipped by the loop guard.
                    stack.push((ar, c2 - ca, a_abs + c2, br, c2 - cb, b_abs + c2, len - c2));
                }
            }
            // At least one leaf-like side (mem Leaf or file-backed Chunk —
            // both expose a contiguous text view via `leaf_view`; PLAN-728).
            _ => {
                let (va, vb) = (leaf_view(a), leaf_view(b));
                match (va, vb) {
                    (Some(ta), Some(tb)) => {
                        // Leaf-leaf pieces classify by content (the only
                        // place a mismatch can surface — everything above
                        // classifies via the shortcuts).
                        let (oa, ob) = (chunk_off(a), chunk_off(b));
                        if &ta.bytes()[oa + a_rel..oa + a_rel + len]
                            == &tb.bytes()[ob + b_rel..ob + b_rel + len]
                        {
                            out.shared.push((a_abs, a_abs + len, b_abs, b_abs + len));
                        } else {
                            out.diverged.push((a_abs, a_abs + len, b_abs, b_abs + len));
                        }
                    }
                    _ => {
                        let leaf_is_a = matches!(a, Node::Leaf { .. } | Node::Chunk { .. });
                        let (leaf, l_rel, l_abs, inner, i_rel, i_abs) = if leaf_is_a {
                            (a, a_rel, a_abs, b, b_rel, b_abs)
                        } else {
                            (b, b_rel, b_abs, a, a_rel, a_abs)
                        };
                        let (il, ir) = inner.children();
                        let cut = if i_rel >= il.bytes() {
                            0
                        } else {
                            (il.bytes() - i_rel).min(len)
                        };
                        if cut == 0 {
                            // Span starts at/past the internal side's boundary.
                            if leaf_is_a {
                                stack.push((
                                    leaf,
                                    l_rel,
                                    l_abs,
                                    ir,
                                    i_rel - il.bytes(),
                                    i_abs,
                                    len,
                                ));
                            } else {
                                stack.push((
                                    ir,
                                    i_rel - il.bytes(),
                                    i_abs,
                                    leaf,
                                    l_rel,
                                    l_abs,
                                    len,
                                ));
                            }
                            continue;
                        }
                        if cut == len {
                            // Span ends inside the internal side's left child.
                            if leaf_is_a {
                                stack.push((leaf, l_rel, l_abs, il, i_rel, i_abs, len));
                            } else {
                                stack.push((il, i_rel, i_abs, leaf, l_rel, l_abs, len));
                            }
                            continue;
                        }
                        if leaf_is_a {
                            stack.push((leaf, l_rel, l_abs, il, i_rel, i_abs, cut));
                            stack.push((
                                leaf,
                                l_rel + cut,
                                l_abs + cut,
                                ir,
                                0,
                                i_abs + cut,
                                len - cut,
                            ));
                        } else {
                            stack.push((il, i_rel, i_abs, leaf, l_rel, l_abs, cut));
                            stack.push((
                                ir,
                                0,
                                i_abs + cut,
                                leaf,
                                l_rel + cut,
                                l_abs + cut,
                                len - cut,
                            ));
                        }
                    }
                }
            }
        }
    }
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
        Self {
            root: Node::leaf(String::new()),
        }
    }
}

impl Rope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            root: leaf_or_tree(text),
        }
    }

    /// O(1) frozen view; further edits to this rope leave it untouched.
    pub fn snapshot(&self) -> RopeSnapshot {
        RopeSnapshot {
            root: self.root.clone(),
        }
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

    /// Cached content digest of the whole document — O(1) root read
    /// (PLAN-703 T-01). Content-determined polynomial digest: see the
    /// module header. Empty document digests to the sentinel 1.
    pub fn content_hash(&self) -> u64 {
        self.root.hash()
    }

    /// Digest of the byte range `[start, end)` — O(log n) aligned pieces +
    /// O(leaf) boundary hashing (PLAN-703 T-01). Equals the flat content
    /// digest of the slice; pruning shortcut for aligned comparisons.
    pub fn range_hash(&self, start: usize, end: usize) -> u64 {
        q_range_hash(&self.root, start, end)
    }

    /// Exact content equality with structural-sharing and Merkle shortcuts
    /// (PLAN-703 T-01 `subtree_equal`). O(1) for shared roots or digest
    /// matches; otherwise an exact aligned walk — verdicts stay exact.
    pub fn subtree_equal(&self, other: &Rope) -> bool {
        q_subtree_equal(&self.root, &other.root)
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
        debug_assert!(
            start <= end && end <= self.len_bytes(),
            "replace_bytes out of range"
        );
        debug_assert!(
            self.root.is_char_boundary(start),
            "replace_bytes: start not a char boundary"
        );
        debug_assert!(
            self.root.is_char_boundary(end),
            "replace_bytes: end not a char boundary"
        );
        let (left, right) = split(&self.root, end);
        let (left, _) = split(&left, start);
        self.root = concat(concat(left, leaf_or_tree(text)), right);
    }

    // ── PLAN-728: file-backed paging (供⑮ 面①..⑦) ─────────────────────

    /// Open `path` as a file-backed (paged) rope: prescan builds the page
    /// table + per-page summaries (one sequential pass — `line_count` and
    /// digests answer O(1)/O(log n) WITHOUT materializing; text fault-ins
    /// per page through the store's LRU cache). Returns the store handle
    /// (prefetch/baseline probes) alongside the rope. Errors mirror
    /// `read_to_string` semantics (IO / invalid UTF-8).
    pub(crate) fn open_file_backed(
        path: &Path,
        cfg: PageConfig,
    ) -> io::Result<(Rope, Arc<PageStore>)> {
        let store = PageStore::prescan(path, &cfg)?;
        file_backing::spawn_prefetcher(&store);
        let root = build_chunk_tree(&store);
        Ok((Rope { root }, store))
    }

    /// Whether byte `offset` starts a char (or is EOF) — the validation
    /// face for endpoint offsets (PLAN-728: paged cores validate up front).
    pub fn is_char_boundary(&self, offset: usize) -> bool {
        offset <= self.len_bytes() && self.root.is_char_boundary(offset)
    }

    /// Flat content digest of a plain `&str` — the comparison face for
    /// `content_hash` fast paths (content-determined: equal bytes ⇒ equal
    /// digests regardless of rope shape).
    pub(crate) fn content_hash_of(text: &str) -> u64 {
        poly_digest(text.as_bytes())
    }

    /// Streaming merge save (PLAN-728 面⑤): unchanged base spans copy
    /// disk→disk from the base handle (bypassing the page cache), edited
    /// spans write from memory; the output lands in a sibling temp file
    /// then atomically renames over `dest` (crash-safe — an interrupted
    /// save leaves the original intact). Byte-for-byte with the rope
    /// content. Errors — including an externally-mutated base (len/mtime
    /// baseline mismatch) — abort before any rename (报错不静默).
    /// Returns the bytes written.
    pub fn write_backed(&self, dest: &Path) -> io::Result<u64> {
        use std::io::Write as _;
        // 1. Verify every base store that IS the destination still matches
        //    its prescan baseline (a mutated base would corrupt the copy).
        let stores = collect_stores(&self.root);
        for store in &stores {
            // ANY mutated base invalidates its chunk offsets — refuse
            // regardless of the destination path (报错不静默).
            if !store.baseline_matches() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "paged-rope save: base file {:?} changed on disk since open \
                         (external modification) — refusing to save",
                        store.path()
                    ),
                ));
            }
        }
        // 2. Write the merged content to a sibling temp file.
        let tmp = sibling_temp_path(dest);
        {
            let out = std::fs::File::create(&tmp)?;
            let mut w = io::BufWriter::with_capacity(1024 * 1024, out);
            let mut copy_buf = vec![0u8; 256 * 1024];
            let mut written = 0u64;
            write_node_for_save(&self.root, &mut w, &mut copy_buf, &mut written)?;
            w.flush()?;
            let file = w
                .into_inner()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            file.sync_all()?;
        }
        // 3. Atomic replace, then re-stamp the baselines we just changed.
        std::fs::rename(&tmp, dest)?;
        for store in &stores {
            if store.path() == dest {
                store.refresh_baseline();
            }
        }
        Ok(self.len_bytes() as u64)
    }

    /// Structural residency estimate: every tree node counts as its own
    /// allocation size (an upper bound on the real per-node overhead —
    /// chunk descriptors and internal summaries included, text of memory
    /// leaves excluded). With `PageStore::structural_resident_bytes` this
    /// is the ≤12MB@1GB contract assertion face (AC-03).
    pub fn structural_resident_estimate(&self) -> usize {
        let mut nodes = 0usize;
        count_nodes(&self.root, &mut nodes);
        nodes * std::mem::size_of::<Node>()
    }
}

/// Recursively count tree nodes (early-exit free — the whole tree counts;
/// O(#nodes), used only by the residency estimate face).
fn count_nodes(node: &Arc<Node>, out: &mut usize) {
    *out += 1;
    if let Node::Internal { left, right, .. } = &**node {
        count_nodes(left, out);
        count_nodes(right, out);
    }
}

/// Collect the distinct page stores referenced by the tree (dedup by Arc
/// identity). A single-open rope references exactly one; the walk stays
/// general for composed/edited shapes.
fn collect_stores(node: &Arc<Node>) -> Vec<Arc<PageStore>> {
    fn walk(node: &Arc<Node>, out: &mut Vec<Arc<PageStore>>) {
        match &**node {
            Node::Chunk { store, .. } => {
                if !out.iter().any(|s| Arc::ptr_eq(s, store)) {
                    out.push(store.clone());
                }
            }
            Node::Internal { left, right, .. } => {
                walk(left, out);
                walk(right, out);
            }
            Node::Leaf { .. } => {}
        }
    }
    let mut out = Vec::new();
    walk(node, &mut out);
    out
}

/// `<dest>.<pid>.p728tmp` in the destination's own directory (same volume
/// — rename stays atomic).
fn sibling_temp_path(dest: &Path) -> PathBuf {
    let mut name = dest
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    name.push_str(&format!(".{}.p728tmp", std::process::id()));
    dest.with_file_name(name)
}

/// Merge-save walk: chunk spans stream from the base handle through the
/// copy buffer (never through the page cache); leaf spans write their
/// text; internal spans recurse left-to-right (document order).
fn write_node_for_save(
    node: &Arc<Node>,
    out: &mut dyn io::Write,
    copy_buf: &mut [u8],
    written: &mut u64,
) -> io::Result<()> {
    match &**node {
        Node::Leaf { text, .. } => {
            out.write_all(text.as_bytes())?;
            *written += text.len() as u64;
        }
        Node::Chunk {
            store,
            page,
            off,
            len,
            ..
        } => {
            let mut offset = store.range_offset(*page, *off);
            let mut remaining = *len as usize;
            while remaining > 0 {
                let want = remaining.min(copy_buf.len());
                store.read_base_range(offset, &mut copy_buf[..want])?;
                out.write_all(&copy_buf[..want])?;
                *written += want as u64;
                offset += want as u64;
                remaining -= want;
            }
        }
        Node::Internal { left, right, .. } => {
            write_node_for_save(left, out, copy_buf, written)?;
            write_node_for_save(right, out, copy_buf, written)?;
        }
    }
    Ok(())
}

/// Balanced tree over the store's whole-page chunk nodes (midpoint build —
/// height-balanced by construction; chunks never merge like small leaves,
/// so no `concat` squeeze applies). An empty base becomes an empty memory
/// leaf (empty documents are one empty line either way).
fn build_chunk_tree(store: &Arc<PageStore>) -> Arc<Node> {
    fn build(store: &Arc<PageStore>, lo: u32, hi: u32) -> Arc<Node> {
        if lo + 1 == hi {
            return Node::chunk_whole(store.clone(), lo);
        }
        let mid = lo + (hi - lo) / 2;
        Node::internal(build(store, lo, mid), build(store, mid, hi))
    }
    match store.page_count() {
        0 => Node::leaf(String::new()),
        n => build(store, 0, n as u32),
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

    /// Cached content digest — O(1) root read (PLAN-703 T-01). Same
    /// shape-sensitive Merkle semantics as [`Rope::content_hash`].
    pub fn content_hash(&self) -> u64 {
        self.root.hash()
    }
    /// Digest of the byte range `[start, end)` — see [`Rope::range_hash`].
    pub fn range_hash(&self, start: usize, end: usize) -> u64 {
        q_range_hash(&self.root, start, end)
    }
    /// Exact content equality with shortcuts — see [`Rope::subtree_equal`].
    pub fn subtree_equal(&self, other: &RopeSnapshot) -> bool {
        q_subtree_equal(&self.root, &other.root)
    }
    /// Aligned prune spans vs `other` — see [`Rope::prune_spans`].
    pub(crate) fn prune_spans(&self, other: &RopeSnapshot) -> PruneSpans {
        let mut out = PruneSpans::default();
        collect_prune_spans(
            &self.root,
            0,
            &other.root,
            0,
            self.root.bytes().min(other.root.bytes()),
            &mut out,
        );
        append_length_tails(&self.root, &other.root, &mut out);
        out
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
        "a", "b", "Z", "0", " ", "~", "\n", "\n", "é", "中", "文", "🦀", "🎉", "ß", "fn ", "{",
        "}", "\t",
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
        std::iter::once(0)
            .chain(s.char_indices().map(|(i, c)| i + c.len_utf8()))
            .collect()
    }

    fn assert_invariants(rope: &Rope, model: &str, ctx: &str) {
        assert_eq!(rope.to_string(), model, "content mismatch ({ctx})");
        assert_eq!(rope.len_bytes(), model.len(), "len_bytes ({ctx})");
        assert_eq!(rope.len_chars(), model.chars().count(), "len_chars ({ctx})");
        let want_lines = model.bytes().filter(|&b| b == b'\n').count() + 1;
        assert_eq!(rope.line_count(), want_lines, "line_count ({ctx})");
        // Root summaries agree with the ground truth.
        assert_eq!(
            rope.line(0),
            model.lines().next().unwrap_or(""),
            "line(0) ({ctx})"
        );
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
                let end =
                    bounds[(pos + 1 + rng.below(bounds.len() - pos - 1)).min(bounds.len() - 1)];
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
            // PLAN-703 T-01: digest tracks the model content exactly, and
            // every byte range digests as its flat content digest.
            assert_eq!(
                rope.content_hash(),
                poly_digest(model.as_bytes()),
                "content_hash ({ctx})"
            );
            let bounds = boundaries(&model);
            let bi = rng.below(bounds.len());
            let hi_i = bi + rng.below(bounds.len() - bi);
            let (lo, hi) = (bounds[bi], bounds[hi_i]);
            assert_eq!(
                rope.range_hash(lo, hi),
                poly_digest(&model.as_bytes()[lo..hi]),
                "range_hash [{lo},{hi}) ({ctx})"
            );

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
            assert_eq!(
                r.to_string(),
                format!("{}X中{}", &base[..pos], &base[pos..])
            );

            let mut r = Rope::from_str(base);
            if pos < base.len() {
                let next = boundaries(base).into_iter().find(|&b| b > pos).unwrap();
                r.delete_bytes(pos, next);
                assert_eq!(r.to_string(), format!("{}{}", &base[..pos], &base[next..]));
            }

            let mut r = Rope::from_str(base);
            let end = boundaries(base)
                .into_iter()
                .find(|&b| b > pos)
                .unwrap_or(base.len());
            r.replace_bytes(pos, end, "🎉");
            assert_eq!(r.to_string(), format!("{}🎉{}", &base[..pos], &base[end..]));
        }
    }

    #[test]
    fn empty_rope_ops() {
        let mut r = Rope::new();
        assert_eq!(r.to_string(), "");
        assert_eq!(
            r.line_count(),
            1,
            "empty document is ONE empty line (lines.join convention)"
        );
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
        // PLAN-703 T-01: digests stay frozen with the content.
        assert_eq!(s1.content_hash(), poly_digest(b"hello world"));
        assert_eq!(s2.content_hash(), poly_digest(b"hello brave world"));
        assert_eq!(s3.content_hash(), poly_digest(b"world"));

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

    // ── PLAN-703 T-01: content digests + subtree equality ───────────────

    /// Multi-leaf text (spans several leaves at 1KB target size).
    fn leaf_spanning_text() -> String {
        let mut s = String::new();
        for i in 0..80 {
            s.push_str(&format!(
                "line {i} — some content to spread across leaves 🦀\n"
            ));
        }
        s
    }

    #[test]
    fn digest_basics_and_range_agreement() {
        let text = leaf_spanning_text();
        assert!(
            text.len() > LEAF_MAX_BYTES,
            "fixture must span multiple leaves"
        );
        let r = Rope::from_str(&text);
        assert_eq!(r.content_hash(), poly_digest(text.as_bytes()));
        assert_eq!(
            r.range_hash(0, r.len_bytes()),
            r.content_hash(),
            "full range == root digest"
        );
        assert_eq!(
            r.range_hash(0, 0),
            1,
            "empty range = empty digest (sentinel)"
        );
        // Sampled subrange agreement (the fold is the concatenation rule, so
        // decomposition cannot change the value); plus an exhaustive sweep
        // over leaf-aligned pieces.
        let bytes = text.as_bytes();
        let mut rng = Rng::new(703);
        let bounds = boundaries(&text);
        for _ in 0..400 {
            let bi = rng.below(bounds.len());
            let hi_i = bi + rng.below(bounds.len() - bi);
            let (lo, hi) = (bounds[bi], bounds[hi_i]);
            assert_eq!(
                r.range_hash(lo, hi),
                poly_digest(&bytes[lo..hi]),
                "range [{lo},{hi})"
            );
        }
        // Snapshots expose the same faces with identical values.
        let s = r.snapshot();
        assert_eq!(s.content_hash(), r.content_hash());
        assert_eq!(s.range_hash(100, 200), r.range_hash(100, 200));
        // Digest of the empty rope = the empty digest (sentinel 1).
        assert_eq!(Rope::new().content_hash(), 1);
    }

    /// AC-01 three families: shared / forked / re-edit-back-to-equal.
    #[test]
    fn subtree_equal_shared_forked_revert() {
        let text = leaf_spanning_text();

        // Family 1 — structural sharing: snapshots and clones of one rope
        // share root Arcs → equal at O(1) (ptr_eq shortcut).
        let mut r = Rope::from_str(&text);
        let s1 = r.snapshot();
        assert!(
            s1.subtree_equal(&r.snapshot()),
            "shared snapshot roots equal"
        );
        let clone = r.clone();
        assert!(r.subtree_equal(&clone), "clone shares structure");

        // Family 2 — fork: an edit diverges content → unequal.
        let edit_at = r.line_start_byte(40);
        let mut fork = r.clone();
        fork.insert_bytes(edit_at, "// edited\n");
        assert!(!r.subtree_equal(&fork), "forked content unequal");
        let s_mid = fork.snapshot();
        assert!(
            !s1.subtree_equal(&s_mid),
            "frozen original vs edited snapshot"
        );

        // Family 3 — re-edit back to the original content: content-determined
        // digests make the verdict equal again at the root shortcut (the
        // exact walk backs it for any residual shape drift).
        fork.delete_bytes(edit_at, edit_at + "// edited\n".len());
        assert_eq!(fork.to_string(), text);
        assert!(r.subtree_equal(&fork), "re-edit back to equal content");
        // Snapshot-level verdicts agree with rope-level ones.
        let s2 = fork.snapshot();
        assert!(
            s2.subtree_equal(&s1),
            "reverted content equals frozen original"
        );

        // Independent builds of the same content → equal (identical
        // canonical shape → Merkle shortcut fires at the root).
        let a = Rope::from_str(&text);
        let b = Rope::from_str(&text);
        assert!(a.subtree_equal(&b));

        // Same length, different content → false (fast digest/summary exit
        // or exact leaf compare — either way, no false positive).
        let mut flipped = text.into_bytes();
        let mid = flipped.len() / 2;
        flipped[mid] = if flipped[mid] == b'x' { b'y' } else { b'x' };
        let c = Rope::from_str(std::str::from_utf8(&flipped).unwrap());
        assert!(!a.subtree_equal(&c));
        // Empty vs empty.
        assert!(Rope::new().subtree_equal(&Rope::new()));
        assert!(!Rope::new().subtree_equal(&Rope::from_str("x")));
    }

    /// Prune spans (T-04 preprocessing face): shared regions are truly
    /// byte-identical, diverged regions carry the actual edits.
    #[test]
    fn prune_spans_classify_shared_and_diverged() {
        let mut model = String::new();
        for i in 0..300 {
            model.push_str(&format!(
                "line {i} with padding text to cross leaf boundaries\n"
            ));
        }
        let mut r = Rope::from_str(&model);

        // Identical snapshot: one shared span, nothing diverged.
        let s0 = r.snapshot();
        let spans = r.snapshot().prune_spans(&s0);
        assert_eq!(spans.shared, vec![(0, r.len_bytes(), 0, r.len_bytes())]);
        assert!(spans.diverged.is_empty());

        // Edit in the middle: shared prefix + suffix (with the structural
        // sharing dividend — the untouched tails are the same Arcs), the
        // edit region registers as diverged.
        let insert_at = r.line_start_byte(150);
        r.insert_bytes(insert_at, "INSERTED\n");
        let mut b_model = model.clone();
        b_model.insert_str(insert_at, "INSERTED\n");
        let spans = r.snapshot().prune_spans(&s0);
        assert!(
            !spans.diverged.is_empty(),
            "the edit region must register as diverged"
        );
        // Shared spans must be byte-identical across the (possibly shifted)
        // coordinate pair. a-coords index the POST tree (self = r = the
        // edited rope), b-coords the PRE snapshot (other = s0).
        for (a1, a2, b1, b2) in &spans.shared {
            assert_eq!(
                &b_model[*a1..*a2],
                &model[*b1..*b2],
                "shared span a[{a1},{a2}) b[{b1},{b2})"
            );
        }
        // The inserted text must sit inside some diverged span (post side).
        assert!(
            spans
                .diverged
                .iter()
                .any(|&(a1, a2, _, _)| a1 <= insert_at && insert_at + "INSERTED\n".len() <= a2),
            "insertion covered by a diverged span: {:?}",
            spans.diverged
        );
        // Union coverage: shared + diverged partitions the a-side document.
        let mut covered: Vec<(usize, usize)> = spans
            .shared
            .iter()
            .map(|&(a1, a2, _, _)| (a1, a2))
            .chain(spans.diverged.iter().map(|&(a1, a2, _, _)| (a1, a2)))
            .collect();
        covered.sort();
        let mut pos = 0;
        for (lo, hi) in covered {
            assert_eq!(lo, pos, "spans must tile the document without gaps");
            pos = hi;
        }
        assert_eq!(pos, b_model.len());
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
        assert!(
            t1.elapsed().as_millis() < 1000,
            "byte_to_point took {:?}",
            t1.elapsed()
        );

        let t2 = std::time::Instant::now();
        rope.insert_bytes(mid, "// inserted\n");
        rope.delete_bytes(mid, mid + 12);
        assert_eq!(rope.to_string(), text, "edits must cancel out");
        assert!(
            t2.elapsed().as_secs() < 10,
            "mid edits took {:?}",
            t2.elapsed()
        );

        // O(1) summaries: repeated reads are instant.
        let t3 = std::time::Instant::now();
        for _ in 0..10_000 {
            std::hint::black_box(rope.line_count());
            std::hint::black_box(rope.len_bytes());
        }
        assert!(
            t3.elapsed().as_millis() < 500,
            "root summaries not O(1): {:?}",
            t3.elapsed()
        );
    }
}

#[cfg(test)]
mod p703_debug {
    use super::*;
    #[test]
    fn debug_digest_layers() {
        let text = "a".repeat(1200);
        let r = Rope::from_str(&text);
        let (l0, _) = split(&r.root, 600);
        let h1 = poly_digest(&text.as_bytes()[..600]);
        let h2 = poly_digest(&text.as_bytes()[600..]);
        println!(
            "flat={} root={} left600={} poly600={} poly2={} combine={}",
            poly_digest(text.as_bytes()),
            r.content_hash(),
            l0.hash(),
            h1,
            h2,
            combine_hash(h1, h2, 600)
        );
        // single leaf vs combine
        let s = Rope::from_str("hello");
        println!(
            "small: leafhash={} poly={}",
            s.content_hash(),
            poly_digest(b"hello")
        );
    }
}

#[cfg(test)]
mod prune_span_reference {
    use super::*;

    /// Naive position-aligned reference: shared = equal runs at the same
    /// offset, diverged = the rest.
    fn naive_spans(
        a: &str,
        b: &str,
    ) -> (
        Vec<(usize, usize, usize, usize)>,
        Vec<(usize, usize, usize, usize)>,
    ) {
        let ab = a.as_bytes();
        let bb = b.as_bytes();
        let common = ab.len().min(bb.len());
        let mut shared = vec![];
        let mut diverged = vec![];
        let mut i = 0;
        while i < common {
            let start = i;
            while i < common && ab[i] == bb[i] {
                i += 1;
            }
            if i > start {
                shared.push((start, i, start, i));
            }
            let dstart = i;
            while i < common && ab[i] != bb[i] {
                i += 1;
            }
            if i > dstart {
                diverged.push((dstart, i, dstart, i));
            }
        }
        (shared, diverged)
    }

    /// Differential cross-check against a naive position-aligned classifier:
    /// every walk-shared span must be content-equal AND backed by the naive
    /// shared runs (the walk may only be MORE conservative, never wrong).
    #[test]
    fn prune_spans_agree_with_naive_classifier() {
        let mut model = String::new();
        for i in 0..300 {
            model.push_str(&format!(
                "line {i} with padding text to cross leaf boundaries\n"
            ));
        }
        let insert_at = 7699;
        let mut b_model = model.clone();
        b_model.insert_str(insert_at, "INSERTED\n");
        let r = Rope::from_str(&model);
        let mut r2 = Rope::from_str(&model);
        r2.insert_bytes(insert_at, "INSERTED\n");
        let spans = r2.snapshot().prune_spans(&r.snapshot());
        let (nsh, ndi) = naive_spans(&b_model, &model);
        // every walk-shared span must be content-equal in the models
        for (a1, a2, b1, b2) in &spans.shared {
            assert_eq!(
                &b_model.as_bytes()[*a1..*a2],
                &model.as_bytes()[*b1..*b2],
                "bogus shared a[{a1},{a2}) b[{b1},{b2})"
            );
        }
        // walk-shared must be a subset of naive-shared (conservative OK)
        for (a1, a2, b1, b2) in &spans.shared {
            assert!(
                nsh.iter()
                    .any(|&(c1, c2, d1, d2)| c1 <= *a1 && *a2 <= c2 && d1 <= *b1 && *b2 <= d2),
                "shared a[{a1},{a2}) b[{b1},{b2}) not backed by naive classifier"
            );
        }
        let _ = (nsh, ndi);
    }
}
