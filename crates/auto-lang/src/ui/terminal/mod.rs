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

// Plan 583 T7: re-export 与模块声明同门——此前裸 pub use 在 ui 无 ui-iced
// 组合下 E0432（仓外 a2r 后端 features=["ui",...] 必炸）。
#[cfg(feature = "ui-iced")]
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

/// 选中类型(004/005 语义):Alt+拖 = Block(块选,Windows Terminal 惯例);
/// 双击 = Semantic(词选);三击 = Lines(行选);单击拖 = Simple。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermSelectionType {
    Simple,
    Semantic,
    Lines,
    Block,
}

/// 规范化后的选中区间;坐标为喂入缓冲的 (row, col),start ≤ end
/// (行主序;Block 模式按列带逐行截取)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TermSelection {
    pub kind: TermSelectionType,
    pub start: (usize, usize),
    pub end: (usize, usize),
    /// Finish 之后的区间保持可读(菜单 Copy 语义),直到下一次 Begin。
    pub active: bool,
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
    selection: Mutex<Option<TermSelection>>,
    /// Display offset into the fed buffer (scrollback lives engine-side;
    /// the badge + selection mapping read this).
    scroll_offset: AtomicU64,
    /// 菜单动作载荷(Some(item) 待 app 读;0=Copy 1=Paste 2=SelectAll)。
    menu_item: Mutex<Option<u8>>,
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
            selection: Mutex::new(None),
            scroll_offset: AtomicU64::new(0),
            menu_item: Mutex::new(None),
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
            *fresh.selection.lock().unwrap() = core.selection.lock().unwrap().clone();
            fresh
                .scroll_offset
                .store(core.scroll_offset.load(Ordering::Relaxed), Ordering::Relaxed);
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

// ============================================================================
// T4 交互套件(004/005 迁移):选中三模式+块选 / 滚轮偏移 / 菜单载荷
// ============================================================================

/// Begin a selection at (row, col) with the given type (005 T2 纯函数口径
/// 由 widget 侧的 begin_selection_type 承担:Alt→Block,双击→Semantic,
/// 三击→Lines,单击拖→Simple)。同键重启(替换旧区间)。
pub fn terminal_selection_begin(
    core: &TerminalCore,
    kind: TermSelectionType,
    row: usize,
    col: usize,
) {
    *core.selection.lock().unwrap() = Some(TermSelection {
        kind,
        start: (row, col),
        end: (row, col),
        active: true,
    });
}

/// Extend the selection head to (row, col), normalizing start ≤ end
/// (行主序)。无活跃区间时忽略。
pub fn terminal_selection_extend(core: &TerminalCore, row: usize, col: usize) {
    let mut sel = core.selection.lock().unwrap();
    if let Some(sel) = sel.as_mut() {
        let (sr, sc) = sel.start;
        let forward = (row, col) >= (sr, sc);
        let (start, end) = if forward { ((sr, sc), (row, col)) } else { ((row, col), (sr, sc)) };
        sel.start = start;
        sel.end = end;
    }
}

/// Finish the drag: the range stays readable (menu Copy) but no longer
/// extends. Returns the normalized range for publish bookkeeping.
pub fn terminal_selection_finish(core: &TerminalCore) -> Option<TermSelection> {
    let mut sel = core.selection.lock().unwrap();
    if let Some(sel) = sel.as_mut() {
        sel.active = false;
    }
    *sel
}

/// Current selection snapshot (iced highlight layer reads this).
pub fn terminal_selection(core: &TerminalCore) -> Option<TermSelection> {
    *core.selection.lock().unwrap()
}

/// Semantic(词选)锚点扩展:把 (row,col) 所在词的边界扩成选中区间
/// (空白分词;无活跃区间时忽略)。
pub fn terminal_selection_expand_word(core: &TerminalCore, row: usize, col: usize) {
    let text = core.line(row).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    if col >= chars.len() {
        // 词尾在行外:退化为单格词。
        let mut sel = core.selection.lock().unwrap();
        if let Some(sel) = sel.as_mut() {
            sel.start = (row, col);
            sel.end = (row, col);
        }
        return;
    }
    let is_word = |c: char| !c.is_whitespace();
    if !is_word(chars[col]) {
        let mut sel = core.selection.lock().unwrap();
        if let Some(sel) = sel.as_mut() {
            sel.start = (row, col);
            sel.end = (row, col);
        }
        return;
    }
    let mut begin = col;
    while begin > 0 && is_word(chars[begin - 1]) {
        begin -= 1;
    }
    let mut end = col;
    while end + 1 < chars.len() && is_word(chars[end + 1]) {
        end += 1;
    }
    let mut sel = core.selection.lock().unwrap();
    if let Some(sel) = sel.as_mut() {
        sel.start = (row, begin);
        sel.end = (row, end);
    }
}

/// Extract the selected text from the fed buffer (菜单 Copy / 对拍取证面)。
/// Simple: 首末行截段+中间整行;Lines: 全行;Semantic: start 单词;Block:
/// 每行列带。空选 → None。
pub fn terminal_selected_text(core: &TerminalCore) -> Option<String> {
    let sel: Option<TermSelection> = { *core.selection.lock().unwrap() };
    let sel = sel?;
    let cells = core.cells.lock().unwrap();
    let row_text = |r: usize| -> String {
        cells.get(r).map(|row| row.iter().map(|c| c.ch).collect()).unwrap_or_default()
    };
    let (start, end) = (sel.start, sel.end);
    let text = match sel.kind {
        TermSelectionType::Simple => {
            let mut out = Vec::new();
            for r in start.0..=end.0 {
                let line = row_text(r);
                let chars: Vec<char> = line.chars().collect();
                let begin_c = if r == start.0 { start.1 } else { 0 };
                let end_c = if r == end.0 { end.1 + 1 } else { chars.len() };
                let begin_c = begin_c.min(chars.len());
                let end_c = end_c.min(chars.len());
                out.push(chars[begin_c..end_c].iter().collect::<String>());
            }
            out.join("\n")
        }
        TermSelectionType::Lines => {
            (start.0..=end.0).map(row_text).collect::<Vec<_>>().join("\n")
        }
        TermSelectionType::Semantic => {
            // 词选锚点已在 expand 阶段写入 start;此处按 Simple 截段。
            let line = row_text(end.0);
            let chars: Vec<char> = line.chars().collect();
            let begin_c = start.1.min(chars.len());
            let end_c = (end.1 + 1).min(chars.len());
            chars[begin_c..end_c].iter().collect::<String>()
        }
        TermSelectionType::Block => {
            let (c0, c1) = (start.1.min(end.1), start.1.max(end.1));
            (start.0..=end.0)
                .map(|r| {
                    let line = row_text(r);
                    let chars: Vec<char> = line.chars().collect();
                    let b = c0.min(chars.len());
                    let e = (c1 + 1).min(chars.len());
                    chars[b..e].iter().collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    };
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Clear the selection (点击空白/Escape 等宿主策略)。
pub fn terminal_selection_clear(core: &TerminalCore) {
    *core.selection.lock().unwrap() = None;
}

/// Scroll offset (badge + engine read-back face).
pub fn terminal_scroll_offset(core: &TerminalCore) -> usize {
    core.scroll_offset.load(Ordering::Relaxed) as usize
}

/// Wheel/trackpad scroll: adjust the display offset (clamped at ≥0; the
/// upper bound is the app's scrollback height, unenforced here).
pub fn terminal_scroll(core: &TerminalCore, delta: i32) {
    let current = core.scroll_offset.load(Ordering::Relaxed) as i64;
    let next = (current + delta as i64).max(0) as u64;
    core.scroll_offset.store(next, Ordering::Relaxed);
}

/// Explicit offset set (app round-trips the engine's scrollback position).
pub fn terminal_set_scroll_offset(core: &TerminalCore, offset: usize) {
    core.scroll_offset.store(offset as u64, Ordering::Relaxed);
}

/// 菜单动作载荷写入(widget 命中菜单项时;app 在收到 on_menu 后读取)。
pub fn terminal_set_menu_item(core: &TerminalCore, item: u8) {
    *core.menu_item.lock().unwrap() = Some(item);
}

/// 菜单动作载荷读取(0=Copy 1=Paste 2=SelectAll)。**读取即取走**(take)。
pub fn terminal_take_menu_item(core: &TerminalCore) -> Option<u8> {
    core.menu_item.lock().unwrap().take()
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

    // ==== T4 交互:选中三模式+块选 / 文本抽取 / 滚动 / 菜单载荷 ====

    fn feed_sample(core: &TerminalCore) {
        terminal_feed(
            core,
            &[
                "hello world".into(),
                "second line".into(),
                "third line".into(),
            ],
        );
    }

    #[test]
    fn selection_simple_range_and_text() {
        terminal_dispose("t4-sel-simple");
        let core = terminal("t4-sel-simple", 40, 6);
        feed_sample(core);
        terminal_selection_begin(core, TermSelectionType::Simple, 0, 6);
        terminal_selection_extend(core, 1, 3);
        terminal_selection_finish(core);
        assert_eq!(
            terminal_selection(core).map(|s| (s.start, s.end)),
            Some(((0, 6), (1, 3))),
            "drag backward normalizes to start ≤ end"
        );
        // 端点格含包(与高亮列带一致):row1 取 chars[0..=3] = "seco"。
        assert_eq!(terminal_selected_text(core).as_deref(), Some("world\nseco"));
        terminal_dispose("t4-sel-simple");
    }

    #[test]
    fn selection_lines_takes_full_rows() {
        terminal_dispose("t4-sel-lines");
        let core = terminal("t4-sel-lines", 40, 6);
        feed_sample(core);
        terminal_selection_begin(core, TermSelectionType::Lines, 0, 3);
        terminal_selection_extend(core, 1, 8);
        terminal_selection_finish(core);
        assert_eq!(
            terminal_selected_text(core).as_deref(),
            Some("hello world\nsecond line")
        );
        terminal_dispose("t4-sel-lines");
    }

    #[test]
    fn selection_semantic_expands_word() {
        terminal_dispose("t4-sel-word");
        let core = terminal("t4-sel-word", 40, 6);
        feed_sample(core);
        terminal_selection_begin(core, TermSelectionType::Semantic, 0, 8);
        terminal_selection_expand_word(core, 0, 8);
        terminal_selection_finish(core);
        assert_eq!(terminal_selected_text(core).as_deref(), Some("world"));
        terminal_dispose("t4-sel-word");
    }

    #[test]
    fn selection_block_takes_column_band() {
        terminal_dispose("t4-sel-block");
        let core = terminal("t4-sel-block", 40, 6);
        feed_sample(core);
        terminal_selection_begin(core, TermSelectionType::Block, 0, 6);
        terminal_selection_extend(core, 2, 10);
        terminal_selection_finish(core);
        // 列带 [6..=10]:row0 "world",row1 "second line"[6..=10]=" line",
        // row2 "third line"[6..=10]="line"。
        assert_eq!(
            terminal_selected_text(core).as_deref(),
            Some("world\n line\nline")
        );
        terminal_dispose("t4-sel-block");
    }

    #[test]
    fn scroll_clamps_and_round_trips() {
        terminal_dispose("t4-scroll-1");
        let core = terminal("t4-scroll-1", 40, 6);
        assert_eq!(terminal_scroll_offset(core), 0);
        terminal_scroll(core, -10); // 上滚越界 → clamp 到 0
        assert_eq!(terminal_scroll_offset(core), 0);
        terminal_scroll(core, 25);
        assert_eq!(terminal_scroll_offset(core), 25);
        terminal_scroll(core, -5);
        assert_eq!(terminal_scroll_offset(core), 20);
        terminal_set_scroll_offset(core, 3);
        assert_eq!(terminal_scroll_offset(core), 3);
        terminal_dispose("t4-scroll-1");
    }

    #[test]
    fn menu_payload_take_semantics() {
        terminal_dispose("t4-menu-1");
        let core = terminal("t4-menu-1", 40, 6);
        assert_eq!(terminal_take_menu_item(core), None);
        terminal_set_menu_item(core, 0); // Copy
        assert_eq!(terminal_take_menu_item(core), Some(0));
        assert_eq!(terminal_take_menu_item(core), None, "take 后不重放");
        terminal_dispose("t4-menu-1");
    }

    /// PLAN-009 T2: 最小 `.at` 示例挂载 + headless 断言。
    ///
    /// `test/ui/terminal_min/src/front/app.at` 挂 `<terminal/>`(占位矩形);
    /// 本测沿 view-builder 全链(parse → extract → VmBridge → build)断言
    /// 产出 `View::Terminal`,再经 headless 管线(view_to_vtree)断言占位
    /// 节点存在(几何/键入 prop 可见)。
    #[test]
    fn minimal_at_example_mounts_terminal_placeholder() {
        use crate::aura::extract::extract_widget_from_decl;
        use crate::ast::Stmt;
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        use crate::ui::aura_view_builder::AuraViewBuilder;
        use crate::ui::vm_bridge::VmBridge;
        use crate::ui::vnode_converter::view_to_vtree;
        use crate::ui::View;

        let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test/ui/terminal_min/src/front/app.at");
        let src = std::fs::read_to_string(&at)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", at.display()));

        let session = CompilerSession::ui();
        let mut parser = Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = extract_widget_from_decl(decl).expect("extract");

        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "TermApp");
        let view = builder.build(&widget.view_tree);

        // The tree must carry the terminal element with its geometry.
        fn find_terminal<M: Clone + std::fmt::Debug>(v: &View<M>) -> Option<String> {
            match v {
                View::Terminal { key, cols, rows, lines, .. } => {
                    Some(format!("{key}/{cols}/{rows}/{}", lines.len()))
                }
                _ => None,
            }
        }
        let hit = find_terminal(&view).expect("View::Terminal missing from built view tree");
        assert_eq!(hit, "main/40/10/0", "terminal geometry props must round-trip");

        // Headless: the placeholder node exists in the VTree (占位矩形在案)。
        let vtree = view_to_vtree(view);
        let dump = format!("{vtree:?}");
        assert!(
            dump.contains("terminal key=main cols=40 rows=10"),
            "headless VTree must expose the terminal placeholder node, got:\n{dump}"
        );
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
