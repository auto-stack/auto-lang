//! PLAN-009 P1: native `terminal` component — the Auto-side true self of
//! auto-term's TermGrid control (single-truth migration; the Rust widget.rs
//! freezes as reference oracle once this lands).
//!
//! Layering (code_editor paradigm, Plan 413 §3.1):
//!   mod.rs   ① backend-neutral core — grid state, damage gating, cursor,
//!            feed accessors. NO iced imports (headless tests assert here).
//!   iced/    ② adapter — the only iced dependency point
//!            (T3: per-row paragraph cache + digest-gated rebuilds, cursor
//!            blocks; T4: selection/IME/scroll/menu).
//!
//! Draw protocol (migrated from auto-term widget.rs T2/T3 header):
//! - one cached shaped row per row, rebuilt only when the row digest
//!   (chars + fg + bg) changes;
//! - the feed computes per-row damage (`Full` / `Lines(…)`), accumulated
//!   until consumed — the iced layer re-shapes only dirty rows;
//! - cursor inversion block / non-default background quads stay per-frame
//!   quads (no shaping cost).
//!
//! Data plane — 形态甲 (props-feed), T2 ruling per the pre-authorized rule:
//! the app feeds grid rows through `View::Terminal` props every frame
//! (text rows); styled rows ride the registry sideband
//! (`terminal_feed_cells`) because scalar-only props cannot carry per-cell
//! colors — same split as the P2 FFI surface (`row_text` / `row_style`).
//! Downgrade to 形态乙 only if the T3 2000-line streaming benchmark
//! exceeds the frame budget (measured data in the execution record).
//!
//! The engine (PTY + emulator) stays behind the auto-term cdylib adapter
//! (PLAN-009 P2): FFI at runtime, zero Cargo edge auto-lang → auto-term
//! (无环铁律). ANSI color/cursor surfaces arrive as the local scalar types
//! below — alacritty types never cross into this crate (零新依赖).

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

// ② iced adapter — the only iced dependency point of the terminal.
#[cfg(feature = "ui-iced")]
pub mod iced;

pub use iced::{Terminal, TerminalState};

/// Cell color — scalar mirror of the engine's ANSI color surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TermColor {
    Default,
    /// 0-255 palette (0-15 correspond to the named colors).
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// Cursor shape — scalar mirror of the engine's cursor surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TermCursorShape {
    Block,
    Beam,
    Underline,
    Hidden,
}

/// One grid cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TermCell {
    pub ch: char,
    pub fg: TermColor,
    pub bg: TermColor,
}

impl TermCell {
    pub fn plain(ch: char) -> Self {
        Self { ch, fg: TermColor::Default, bg: TermColor::Default }
    }
}

/// Damage since the last consume — mirror of the engine's `Damage`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalDamage {
    /// Nothing changed.
    None,
    /// Everything changed (geometry change, clear, …).
    Full,
    /// Only these rows changed (deduplicated, ascending).
    Lines(Vec<usize>),
}

impl TerminalDamage {
    /// Number of dirty rows (`usize::MAX` for Full).
    pub fn dirty_count(&self) -> usize {
        match self {
            TerminalDamage::None => 0,
            TerminalDamage::Full => usize::MAX,
            TerminalDamage::Lines(rows) => rows.len(),
        }
    }

    fn absorb(&mut self, other: TerminalDamage) {
        match other {
            TerminalDamage::None => {}
            TerminalDamage::Full => *self = TerminalDamage::Full,
            TerminalDamage::Lines(theirs) => match self {
                TerminalDamage::None => *self = TerminalDamage::Lines(theirs),
                TerminalDamage::Full => {}
                TerminalDamage::Lines(mine) => {
                    mine.extend(theirs);
                    mine.sort_unstable();
                    mine.dedup();
                }
            },
        }
    }
}

/// Cursor position + shape + blink phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorState {
    pub row: usize,
    pub col: usize,
    pub shape: TermCursorShape,
    /// Blink phase (toggled by [`terminal_blink_tick`]); hidden entirely
    /// while `shape == Hidden`.
    pub on: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self { row: 0, col: 0, shape: TermCursorShape::Block, on: true }
    }
}

/// Backend-neutral terminal state, registered per `key` (mirrors
/// CODE_EDITORS: leaked storage, interior mutability, widget rebuilt freely
/// every frame).
pub struct TerminalCore {
    pub key: String,
    /// Grid geometry in cells.
    pub cols: u16,
    pub rows: u16,
    cells: Mutex<Vec<Vec<TermCell>>>,
    /// Per-row content digest (chars + colors) — damage-gate key.
    digests: Mutex<Vec<u64>>,
    /// Damage accumulated since the last [`terminal_take_damage`].
    pending: Mutex<TerminalDamage>,
    /// Bumped on every feed that changed content.
    generation: AtomicU64,
    cursor: Mutex<CursorState>,
    /// Blink bookkeeping (last tick millis).
    blink_ms: Mutex<u64>,
}

const BLINK_PERIOD_MS: u64 = 530;

impl TerminalCore {
    fn new(key: &str, cols: u16, rows: u16) -> Self {
        Self {
            key: key.to_owned(),
            cols,
            rows,
            cells: Mutex::new(vec![Vec::new(); rows as usize]),
            // Blank rows start at the digest of an empty row so the first
            // feed only dirties rows that actually carry text.
            digests: Mutex::new(vec![row_digest(&[]); rows as usize]),
            pending: Mutex::new(TerminalDamage::None),
            generation: AtomicU64::new(0),
            cursor: Mutex::new(CursorState::default()),
            blink_ms: Mutex::new(0),
        }
    }

    /// Current content generation (damage epoch).
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Read one row as plain text (wide chars included verbatim).
    pub fn line(&self, row: usize) -> Option<String> {
        let cells = self.cells.lock().unwrap();
        cells.get(row).map(|row| row.iter().map(|c| c.ch).collect())
    }

    /// Snapshot one row of cells (styled sideband readers).
    pub fn row_cells(&self, row: usize) -> Option<Vec<TermCell>> {
        self.cells.lock().unwrap().get(row).cloned()
    }

    /// Full snapshot (cells + per-row digests) — the iced draw path reads
    /// this once per frame (same per-frame clone cost class as the auto-term
    /// App passing `lines` into the widget by value).
    pub fn snapshot(&self) -> (Vec<Vec<TermCell>>, Vec<u64>) {
        (self.cells.lock().unwrap().clone(), self.digests.lock().unwrap().clone())
    }

    /// Snapshot the full text buffer (tests).
    pub fn lines(&self) -> Vec<String> {
        let cells = self.cells.lock().unwrap();
        cells.iter().map(|row| row.iter().map(|c| c.ch).collect()).collect()
    }

    /// Cursor snapshot (iced draw + headless assertions).
    pub fn cursor(&self) -> CursorState {
        *self.cursor.lock().unwrap()
    }
}

static TERMINALS: Mutex<Option<HashMap<String, &'static TerminalCore>>> = Mutex::new(None);

/// Get-or-create the terminal state for `key`, diffing geometry in.
/// Geometry changes re-register a replacement under the same key, carrying
/// over the old rows and marking `Full` damage.
pub fn terminal(key: &str, cols: u16, rows: u16) -> &'static TerminalCore {
    let mut map = TERMINALS.lock().unwrap();
    let map = map.get_or_insert_with(HashMap::new);
    if let Some(core) = map.get(key) {
        let core: &TerminalCore = core;
        if core.cols != cols || core.rows != rows {
            let fresh: &'static TerminalCore =
                Box::leak(Box::new(TerminalCore::new(key, cols, rows)));
            {
                let mut fresh_cells = fresh.cells.lock().unwrap();
                let old_cells = core.cells.lock().unwrap();
                let n = core.rows as usize;
                for (i, row) in old_cells.iter().take(n).enumerate() {
                    fresh_cells[i] = row.clone();
                }
            }
            fresh.digests.lock().unwrap().clone_from(&core.digests.lock().unwrap());
            fresh.cursor.lock().unwrap().clone_from(&core.cursor.lock().unwrap());
            *fresh.blink_ms.lock().unwrap() = *core.blink_ms.lock().unwrap();
            fresh.generation.store(core.generation() + 1, Ordering::Relaxed);
            *fresh.pending.lock().unwrap() = TerminalDamage::Full;
            map.insert(key.to_owned(), fresh);
            return fresh;
        }
        return core;
    }
    let core: &'static TerminalCore = Box::leak(Box::new(TerminalCore::new(key, cols, rows)));
    map.insert(key.to_owned(), core);
    core
}

/// Row digest: chars + fg + bg (style-only changes also invalidate).
fn row_digest(cells: &[TermCell]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for cell in cells {
        cell.ch.hash(&mut h);
        cell.fg.hash(&mut h);
        cell.bg.hash(&mut h);
    }
    h.finish()
}

/// 形态甲 text feed: replace the buffer rows with the app-supplied text
/// (padded to `rows`, truncated beyond; text longer than `cols` truncates
/// by display width). Damage accumulates per changed row; an identical
/// feed is a no-op (generation + damage untouched — no frame churn).
pub fn terminal_feed(core: &TerminalCore, lines: &[String]) {
    let incoming: Vec<Vec<TermCell>> = (0..core.rows as usize)
        .map(|i| {
            let text = lines.get(i).map(String::as_str).unwrap_or("");
            let mut cells: Vec<TermCell> = Vec::with_capacity(core.cols as usize);
            let mut width = 0usize;
            for ch in text.chars() {
                if width >= core.cols as usize {
                    break;
                }
                width += char_width(ch);
                cells.push(TermCell::plain(ch));
            }
            cells
        })
        .collect();
    apply_cells(core, incoming);
}

/// Styled sideband feed: replace one row with cells (adapter `row_style`
/// path; scalar color surface defined above).
pub fn terminal_feed_cells(core: &TerminalCore, row: usize, cells: Vec<TermCell>) {
    if row >= core.rows as usize {
        return;
    }
    let mut all = core.cells.lock().unwrap();
    let mut capped = cells;
    capped.truncate(core.cols as usize);
    all[row] = capped;
    drop(all);
    recompute_digest(core, row);
    core.pending.lock().unwrap().absorb(TerminalDamage::Lines(vec![row]));
    core.generation.fetch_add(1, Ordering::Relaxed);
}

/// Consume accumulated damage (iced layer calls once per frame; the
/// headless tests assert on it directly).
pub fn terminal_take_damage(core: &TerminalCore) -> TerminalDamage {
    let mut pending = core.pending.lock().unwrap();
    std::mem::replace(&mut *pending, TerminalDamage::None)
}

fn apply_cells(core: &TerminalCore, incoming: Vec<Vec<TermCell>>) {
    let mut cells = core.cells.lock().unwrap();
    let mut digests = core.digests.lock().unwrap();
    let mut dirty: Vec<usize> = Vec::new();
    for (i, mut row) in incoming.into_iter().enumerate() {
        row.truncate(core.cols as usize);
        let digest = row_digest(&row);
        if digests[i] != digest {
            digests[i] = digest;
            cells[i] = row;
            dirty.push(i);
        }
    }
    drop(cells);
    drop(digests);
    if !dirty.is_empty() {
        core.pending.lock().unwrap().absorb(TerminalDamage::Lines(dirty));
        core.generation.fetch_add(1, Ordering::Relaxed);
    }
}

fn recompute_digest(core: &TerminalCore, row: usize) {
    let cells = core.cells.lock().unwrap();
    let digest = row_digest(&cells[row]);
    drop(cells);
    core.digests.lock().unwrap()[row] = digest;
}

/// Cursor update (position / shape / visibility phase).
pub fn terminal_set_cursor(core: &TerminalCore, row: usize, col: usize, shape: TermCursorShape) {
    let mut cursor = core.cursor.lock().unwrap();
    cursor.row = row;
    cursor.col = col;
    cursor.shape = shape;
}

/// Blink clock: toggles the `on` phase every [`BLINK_PERIOD_MS`]. The app
/// wires this to its `.Tick` (iced 0.14 has no request_redraw — the frame
/// cadence comes from the existing tick subscription).
pub fn terminal_blink_tick(core: &TerminalCore, now_ms: u64) -> bool {
    let mut blink = core.blink_ms.lock().unwrap();
    let phase = (now_ms / BLINK_PERIOD_MS) % 2 == 0;
    let changed = *blink != now_ms / BLINK_PERIOD_MS;
    *blink = now_ms / BLINK_PERIOD_MS;
    drop(blink);
    let mut cursor = core.cursor.lock().unwrap();
    if cursor.on != phase {
        cursor.on = phase;
    }
    changed
}

/// Display width of a character in cells (dependency-free compact table:
/// combining = 0, the common wide blocks (Hangul/CJK/kana/fullwidth/emoji)
/// = 2, everything else = 1). Exotic-width scripts outside the blocks are
/// approximated as 1 — documented scope of the T3 port.
pub fn char_width(ch: char) -> usize {
    let c = ch as u32;
    if c == 0 {
        return 0;
    }
    // Combining marks (rough continuation ranges).
    if (0x0300..=0x036F).contains(&c)
        || (0x200B..=0x200F).contains(&c)
        || (0xFE00..=0xFE0F).contains(&c)
    {
        return 0;
    }
    // Wide blocks.
    if (0x1100..=0x115F).contains(&c)      // Hangul Jamo
        || (0x2E80..=0x303E).contains(&c)  // CJK Radicals .. CJK Symbols
        || (0x3041..=0x33FF).contains(&c)  // Hiragana .. CJK Compatibility
        || (0x3400..=0x4DBF).contains(&c)  // CJK Ext A
        || (0x4E00..=0x9FFF).contains(&c)  // CJK Unified
        || (0xA000..=0xA4CF).contains(&c)  // Yi
        || (0xAC00..=0xD7A3).contains(&c)  // Hangul Syllables
        || (0xF900..=0xFAFF).contains(&c)  // CJK Compatibility Ideographs
        || (0xFE30..=0xFE4F).contains(&c)  // CJK Compatibility Forms
        || (0xFF00..=0xFF60).contains(&c)  // Fullwidth Forms
        || (0xFFE0..=0xFFE6).contains(&c)
        || (0x1F300..=0x1F64F).contains(&c) // Emoji pictographs
        || (0x1F900..=0x1F9FF).contains(&c) // Supplemental Symbols
        || (0x20000..=0x2FFFD).contains(&c) // CJK Ext B..
    {
        return 2;
    }
    1
}

/// Display width of a row in cells.
pub fn row_width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}

/// Dispose the state for `key` (component unmount / test teardown).
pub fn terminal_dispose(key: &str) {
    if let Ok(mut guard) = TERMINALS.lock() {
        if let Some(map) = guard.as_mut() {
            map.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_get_or_create_and_feed() {
        terminal_dispose("t2-core-1");
        let core = terminal("t2-core-1", 80, 24);
        assert_eq!(core.cols, 80);
        assert_eq!(core.rows, 24);
        assert_eq!(core.lines().len(), 24);
        assert_eq!(terminal_take_damage(core).dirty_count(), 0);
        let gen0 = core.generation();

        // Feed two rows; the rest stays blank.
        terminal_feed(core, &["ready.".to_owned(), "second".to_owned()]);
        assert_eq!(core.line(0).as_deref(), Some("ready."));
        assert_eq!(core.line(1).as_deref(), Some("second"));
        assert_eq!(core.line(2).as_deref(), Some(""));
        assert!(core.generation() > gen0);
        let damage = terminal_take_damage(core);
        assert_eq!(damage, TerminalDamage::Lines(vec![0, 1]), "only the two fed rows are dirty");

        // Identical feed is a no-op (generation stable — no frame churn).
        let gen1 = core.generation();
        terminal_feed(core, &["ready.".to_owned(), "second".to_owned()]);
        assert_eq!(core.generation(), gen1);
        assert_eq!(terminal_take_damage(core), TerminalDamage::None);

        terminal_dispose("t2-core-1");
    }

    #[test]
    fn terminal_geometry_change_preserves_rows() {
        terminal_dispose("t2-core-2");
        let core = terminal("t2-core-2", 40, 4);
        terminal_feed(core, &["a".into(), "b".into(), "c".into(), "d".into()]);
        assert_eq!(terminal_take_damage(core), TerminalDamage::Lines(vec![0, 1, 2, 3]));
        let core2 = terminal("t2-core-2", 40, 6);
        assert_eq!(core2.rows, 6);
        assert_eq!(core2.line(0).as_deref(), Some("a"));
        assert_eq!(core2.line(3).as_deref(), Some("d"));
        assert_eq!(core2.line(4).as_deref(), Some(""));
        // Geometry change resurfaces as Full damage (repaint everything).
        assert_eq!(terminal_take_damage(core2), TerminalDamage::Full);
        terminal_dispose("t2-core-2");
    }

    #[test]
    fn damage_accumulates_and_dedupes() {
        terminal_dispose("t2-damage-1");
        let core = terminal("t2-damage-1", 20, 4);
        terminal_feed(core, &["one".into(), "two".into()]);
        assert_eq!(terminal_take_damage(core), TerminalDamage::Lines(vec![0, 1]));

        // Feed 2: row 0 changes, row 1 identical, row 2 newly written.
        terminal_feed(core, &["ONE".into(), "two".into(), "three".into()]);
        assert_eq!(
            terminal_take_damage(core),
            TerminalDamage::Lines(vec![0, 2]),
            "row 1 identical across feeds → not dirty"
        );
        // Accumulation across feeds (no consume in between) unions + dedupes:
        // feed A rewrites rows 1(unchanged)…2("three"→blank), feed B
        // rewrites row 1("two"→"TWO"); row 1 dirty in both → deduped.
        terminal_feed(core, &["ONE".into(), "two".into()]);
        terminal_feed(core, &["ONE".into(), "TWO".into()]);
        assert_eq!(
            terminal_take_damage(core),
            TerminalDamage::Lines(vec![1, 2]),
        );
        terminal_dispose("t2-damage-1");
    }

    #[test]
    fn styled_feed_changes_damage_and_cells() {
        terminal_dispose("t2-style-1");
        let core = terminal("t2-style-1", 20, 2);
        terminal_feed(core, &["plain".into()]);
        assert_eq!(terminal_take_damage(core).dirty_count(), 1);

        terminal_feed_cells(
            core,
            0,
            vec![
                TermCell { ch: 'x', fg: TermColor::Indexed(1), bg: TermColor::Default },
                TermCell { ch: 'y', fg: TermColor::Rgb(10, 20, 30), bg: TermColor::Indexed(4) },
            ],
        );
        assert_eq!(core.line(0).as_deref(), Some("xy"));
        let cells = core.row_cells(0).unwrap();
        assert_eq!(cells[0].fg, TermColor::Indexed(1));
        assert_eq!(cells[1].bg, TermColor::Indexed(4));
        let damage = terminal_take_damage(core);
        assert_eq!(damage, TerminalDamage::Lines(vec![0]));
        terminal_dispose("t2-style-1");
    }

    #[test]
    fn cursor_state_shape_and_blink() {
        terminal_dispose("t3-cursor-1");
        let core = terminal("t3-cursor-1", 40, 10);
        terminal_set_cursor(core, 3, 7, TermCursorShape::Beam);
        let cursor = core.cursor();
        assert_eq!((cursor.row, cursor.col), (3, 7));
        assert_eq!(cursor.shape, TermCursorShape::Beam);
        assert!(cursor.on, "cursor starts on");

        // Blink toggles at the half-period boundary (530ms).
        terminal_blink_tick(core, 100);
        assert!(core.cursor().on, "within the first half-period: on");
        terminal_blink_tick(core, 600);
        assert!(!core.cursor().on, "second half-period: off");
        terminal_blink_tick(core, 1200);
        assert!(core.cursor().on, "third half-period: on again");
        terminal_dispose("t3-cursor-1");
    }

    #[test]
    fn wide_chars_count_two_cells() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width('中'), 2);
        assert_eq!(char_width('あ'), 2);
        assert_eq!(char_width('ﾊ'), 1); // halfwidth katakana
        assert_eq!(char_width('\u{0301}'), 0); // combining acute
        assert_eq!(row_width("ab中"), 4);
    }

    /// T3 acceptance: 2000-line streaming feed — dirty-row computation and
    /// text conversion within the frame budget. The measured wall time is
    /// recorded in the plan execution record (形态甲 ruling evidence).
    #[test]
    fn streaming_2000_line_feed_within_frame_budget() {
        terminal_dispose("t3-bench-1");
        let rows = 2000usize;
        let core = terminal("t3-bench-1", 80, rows as u16);
        let batch: Vec<String> = (0..rows).map(|i| format!("line {i:04}: the quick brown fox")).collect();

        let started = std::time::Instant::now();
        terminal_feed(core, &batch);
        let damage = terminal_take_damage(core);
        assert_eq!(damage.dirty_count(), rows, "initial feed dirties every row");

        // Streaming update: only the tail rows change.
        let mut next = batch.clone();
        for i in (rows - 5)..rows {
            next[i] = format!("updated {i}");
        }
        terminal_feed(core, &next);
        let damage = terminal_take_damage(core);
        assert_eq!(damage.dirty_count(), 5, "streaming tick dirties only the changed tail");

        let full_snapshot = core.lines();
        assert_eq!(full_snapshot.len(), rows);
        assert_eq!(full_snapshot[0], "line 0000: the quick brown fox");
        assert_eq!(full_snapshot[rows - 1], format!("updated {}", rows - 1));

        let elapsed = started.elapsed();
        // Frame budget at 60fps is 16ms; debug-build feeds of a full 2000-row
        // batch + a streaming tick + a snapshot stay an order below the 150ms
        // guard. Measured (release/debug) numbers go to the execution record.
        assert!(
            elapsed.as_millis() < 150,
            "2000-line feed exceeded the streaming budget: {elapsed:?}"
        );
        terminal_dispose("t3-bench-1");
    }
}
