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

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
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
    /// 键入队列(widget 键盘捕获翻译成的 VT 字节串,FIFO;宿主引擎泵
    /// `drain` 取走后裸写 PTY——载荷不经消息,.at 消息只当触发器)。
    pending_input: Mutex<Vec<String>>,
    /// 待应用几何(014:widget layout 由可用空间反推的 cols×rows;与
    /// 当前几何不同才落位,宿主 `apply_resize` 泵取走后调引擎 resize)。
    pending_resize: Mutex<Option<(u16, u16)>>,
    /// PLAN-018 D10:配色方案(-1 = 跟随桌面主题 dark→0/light→1;≥0 =
    /// 显式 scheme id)。widget 绘制时解析,零每格开销。
    scheme: AtomicI32,
    /// PLAN-019 滚轮回灌队列:widget 滚轮增量累计(引擎约定,正=上翻
    /// 历史),宿主引擎泵排水后调引擎 scroll——display_offset 在引擎侧,
    /// 本组件只缓存视口快照,本地无历史可滚。
    scroll_delta: Mutex<i32>,
    /// 引擎回滚历史行数(泵回读;滚动条拇指比例用;0=无历史不画)。
    history: std::sync::atomic::AtomicUsize,
    /// PLAN-022 T-01 官方滚动条虚拟滚动同步态(视图投影缓存,非状态源;
    /// 引擎 display_offset 仍是滚动状态单源)。view_target = 观察臂
    /// (视图 y 变化)与引擎回写共同维护的最后已知目标 offset(行);
    /// bound = 写臂去重基线(上次程序化绑定的引擎 offset;初始 -1 =
    /// 首帧强制绑定贴底);bind_suppress = 程序化绑定后吞一次视图观察
    /// (iced scroll_to 回声,防回灌环路)。
    scroll_view_target: std::sync::atomic::AtomicI64,
    scroll_bound_offset: std::sync::atomic::AtomicI64,
    scroll_bind_suppress: std::sync::atomic::AtomicBool,
    // PLAN-022 T-06 回声判别(用户实点 2026-09-19:滚轮上翻一格即卡、
    // 下翻阶梯卡顿):一次性 suppress 无法区分 scroll_to 回声与用户
    // 下一格滚轮(iced 键盘/滚轮事件广播,读出臂先到先吞)→ 用户增量
    // 被吞、引擎不动、下一格跳两格。改判别式:bind 时记录期望回声位与
    // 当时的滚轮代数,observe 时对照——代数未变且落点贴合=回声;
    // 代数已进=用户增量,不得吞。
    wheel_gen: std::sync::atomic::AtomicU64,
    bind_gen: std::sync::atomic::AtomicU64,
    bind_echo_y_px: std::sync::atomic::AtomicI32,
    /// PLAN-024:绑定回声登记的引擎 offset(回声对齐时 view_target 基线
    /// 取此值而非按现时 history 重算——重绑飞行期内容再增长时,重算会
    /// 把基线污染成"增长量",增长漂移判别失效回灌复发)。
    bind_echo_offset: std::sync::atomic::AtomicI64,
    /// PLAN-024 抖动持续环:前一绑定的回声位/offset(交替在途场景下,
    /// 读出臂看到的常是**上一个** scroll_to 的落点;仅记当前回声会把
    /// 它当真实观察回灌 → 两个 scroll_to 交错永续抖动,TRACE 直捕
    /// view 0↔底 交替)。valid 标记区分"无前绑定"(0 是合法 y 值)。
    bind_prev_echo_y_px: std::sync::atomic::AtomicI32,
    bind_prev_echo_offset: std::sync::atomic::AtomicI64,
    bind_prev_echo_valid: std::sync::atomic::AtomicBool,
    /// 首次绑定标记(prev 回声移位的门)。
    bind_echo_seen_flag: std::sync::atomic::AtomicBool,
    // PLAN-024 增长漂移判别(桌面轨实录:输出期滚动条反复抖动)——iced
    // scrollable 内容增长时保持绝对像素位,贴底视口被"顶离"底部,读出臂
    // 若误判为用户滚动回灌引擎,回灌与绑定交替 = 抖动。判别式:**贴底
    // 基线(view_target==0)+ 视图 y 未动 + 滚轮代数未进** 下历史增长
    // = 内容增长漂移 → 不回灌,挂贴底重绑(scroll_repin_pending)由写臂
    // 把视口拉回画布底(跟随输出;已滚向上 base>0 的内容锚定路径不变)。
    scroll_last_history: std::sync::atomic::AtomicUsize,
    scroll_last_view_y_bits: std::sync::atomic::AtomicU32,
    scroll_last_wheel_gen: std::sync::atomic::AtomicU64,
    scroll_repin_pending: std::sync::atomic::AtomicBool,
    // PLAN-025 T-02 快照窗绝对行锚:泵回写 core.cells 行窗首行的绝对
    // 行号(id = 引擎 history − display_offset;不可变 scrollback 语义
    // 下同 id 恒同内容)。i64::MIN = 未锚(泵未跑——vm/desktop 臂;
    // widget 回落槽位键语义,零回归)。
    window_anchor: std::sync::atomic::AtomicI64,
    // PLAN-025 T-03 预取窗存储:绝对行 id → WindowRow(cells+digest)。
    // 泵喂入可见区 ± 预取行(引擎瞬态 scroll 采样);draw 按视口 id
    // 区间消费。独立于 cells 槽位面(props 文本/cursor/selection/vue
    // 文本零变)。
    window_store: Mutex<std::collections::BTreeMap<i64, WindowRow>>,
}

/// PLAN-019 启动自动聚焦:窗口内焦点持有者(terminal key;None = 自由,
/// 首个 terminal 于 update 自动持有)。点击换焦/点击他处释放照旧。
static FOCUS_OWNER: Mutex<Option<String>> = Mutex::new(None);

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
            pending_input: Mutex::new(Vec::new()),
            pending_resize: Mutex::new(None),
            scheme: AtomicI32::new(TERMINAL_SCHEME_FOLLOW_THEME),
            scroll_delta: Mutex::new(0),
            history: std::sync::atomic::AtomicUsize::new(0),
            scroll_view_target: std::sync::atomic::AtomicI64::new(0),
            scroll_bound_offset: std::sync::atomic::AtomicI64::new(-1),
            scroll_bind_suppress: std::sync::atomic::AtomicBool::new(false),
            wheel_gen: std::sync::atomic::AtomicU64::new(0),
            bind_gen: std::sync::atomic::AtomicU64::new(0),
            bind_echo_y_px: std::sync::atomic::AtomicI32::new(0),
            bind_echo_offset: std::sync::atomic::AtomicI64::new(0),
            bind_prev_echo_y_px: std::sync::atomic::AtomicI32::new(0),
            bind_prev_echo_offset: std::sync::atomic::AtomicI64::new(0),
            bind_prev_echo_valid: std::sync::atomic::AtomicBool::new(false),
            bind_echo_seen_flag: std::sync::atomic::AtomicBool::new(false),
            scroll_last_history: std::sync::atomic::AtomicUsize::new(0),
            scroll_last_view_y_bits: std::sync::atomic::AtomicU32::new(0),
            scroll_last_wheel_gen: std::sync::atomic::AtomicU64::new(0),
            scroll_repin_pending: std::sync::atomic::AtomicBool::new(false),
            window_anchor: std::sync::atomic::AtomicI64::new(WINDOW_ANCHOR_UNSET),
            window_store: Mutex::new(std::collections::BTreeMap::new()),
        }
    }

    /// PLAN-018 D10: 当前配色方案(-1 = 跟随主题)。
    pub fn scheme(&self) -> i32 {
        self.scheme.load(Ordering::Relaxed)
    }

    /// PLAN-018 D10: 设置配色方案(scheme prop 显式覆盖;-1 = 跟随主题)。
    pub fn set_scheme(&self, scheme: i32) {
        self.scheme.store(scheme, Ordering::Relaxed);
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

    // ── PLAN-022 官方滚动条虚拟滚动同步态(pub(crate):iced widget/
    //    renderer 臂消费;纯视图投影,引擎 display_offset 仍是单源)──

    pub(crate) fn scroll_view_target(&self) -> i64 {
        self.scroll_view_target.load(Ordering::Relaxed)
    }

    pub(crate) fn set_scroll_view_target(&self, v: i64) {
        self.scroll_view_target.store(v, Ordering::Relaxed);
    }

    pub(crate) fn scroll_bound_offset(&self) -> i64 {
        self.scroll_bound_offset.load(Ordering::Relaxed)
    }

    pub(crate) fn set_scroll_bound_offset(&self, v: i64) {
        self.scroll_bound_offset.store(v, Ordering::Relaxed);
    }

    /// 取走并清除程序化绑定回声抑制(返回旧值)。
    pub(crate) fn take_scroll_bind_suppress(&self) -> bool {
        self.scroll_bind_suppress.swap(false, Ordering::Relaxed)
    }

    pub(crate) fn set_scroll_bind_suppress(&self) {
        self.scroll_bind_suppress.store(true, Ordering::Relaxed);
    }

    pub(crate) fn clear_scroll_bind_suppress(&self) {
        self.scroll_bind_suppress.store(false, Ordering::Relaxed);
    }

    pub(crate) fn scroll_bind_suppress_pending(&self) -> bool {
        self.scroll_bind_suppress.load(Ordering::Relaxed)
    }

    /// 用户滚轮活动计数(WheelScrolled 事件即 bump,方向无关)。
    pub(crate) fn bump_wheel_gen(&self) -> u64 {
        self.wheel_gen.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub(crate) fn wheel_gen(&self) -> u64 {
        self.wheel_gen.load(Ordering::Relaxed)
    }

    /// bind 时登记期望回声位(px)+ 绑定 offset,并快照滚轮代数;
    /// 当前回声移位为前一回声(交替在途判别用)。
    pub(crate) fn record_bind_echo(&self, y_px: f32, offset: i64) {
        self.bind_gen
            .store(self.wheel_gen.load(Ordering::Relaxed), Ordering::Relaxed);
        if self.bind_echo_seen() {
            self.bind_prev_echo_y_px
                .store(self.bind_echo_y_px.load(Ordering::Relaxed), Ordering::Relaxed);
            self.bind_prev_echo_offset
                .store(self.bind_echo_offset.load(Ordering::Relaxed), Ordering::Relaxed);
            self.bind_prev_echo_valid.store(true, Ordering::Relaxed);
        }
        self.bind_echo_y_px.store(y_px.round() as i32, Ordering::Relaxed);
        self.bind_echo_offset.store(offset, Ordering::Relaxed);
        self.bind_echo_seen_flag.store(true, Ordering::Relaxed);
    }

    fn bind_echo_seen(&self) -> bool {
        self.bind_echo_seen_flag.load(Ordering::Relaxed)
    }

    /// 前一绑定回声(offset, y_px);无前绑定 = None。
    pub(crate) fn bind_prev_echo(&self) -> Option<(i64, i32)> {
        if self.bind_prev_echo_valid.load(Ordering::Relaxed) {
            Some((
                self.bind_prev_echo_offset.load(Ordering::Relaxed),
                self.bind_prev_echo_y_px.load(Ordering::Relaxed),
            ))
        } else {
            None
        }
    }

    pub(crate) fn bind_echo(&self) -> (u64, i32) {
        (
            self.bind_gen.load(Ordering::Relaxed),
            self.bind_echo_y_px.load(Ordering::Relaxed),
        )
    }

    pub(crate) fn bind_echo_offset(&self) -> i64 {
        self.bind_echo_offset.load(Ordering::Relaxed)
    }

    // ── PLAN-024 增长漂移判别态(见字段注) ──────────────────────────

    pub(crate) fn scroll_last_history(&self) -> usize {
        self.scroll_last_history.load(Ordering::Relaxed)
    }

    pub(crate) fn set_scroll_last_history(&self, v: usize) {
        self.scroll_last_history.store(v, Ordering::Relaxed);
    }

    pub(crate) fn scroll_last_view_y(&self) -> f32 {
        f32::from_bits(self.scroll_last_view_y_bits.load(Ordering::Relaxed))
    }

    pub(crate) fn set_scroll_last_view_y(&self, v: f32) {
        self.scroll_last_view_y_bits
            .store(v.to_bits(), Ordering::Relaxed);
    }

    pub(crate) fn scroll_last_wheel_gen(&self) -> u64 {
        self.scroll_last_wheel_gen.load(Ordering::Relaxed)
    }

    pub(crate) fn set_scroll_last_wheel_gen(&self, v: u64) {
        self.scroll_last_wheel_gen.store(v, Ordering::Relaxed);
    }

    pub(crate) fn set_scroll_repin_pending(&self) {
        self.scroll_repin_pending.store(true, Ordering::Relaxed);
    }

    pub(crate) fn take_scroll_repin_pending(&self) -> bool {
        self.scroll_repin_pending.swap(false, Ordering::Relaxed)
    }
}

/// BTreeMap(非 HashMap):`terminal_drain_all_inputs` 的键序要稳定。
static TERMINALS: Mutex<Option<BTreeMap<String, &'static TerminalCore>>> = Mutex::new(None);

/// Get-or-create the terminal state for `key`, diffing geometry in.
/// Geometry changes re-register a replacement under the same key, carrying
/// over the old rows and marking `Full` damage.
pub fn terminal(key: &str, cols: u16, rows: u16) -> &'static TerminalCore {
    let mut map = TERMINALS.lock().unwrap();
    let map = map.get_or_insert_with(BTreeMap::new);
    if let Some(core) = map.get(key) {
        let core: &TerminalCore = core;
        if core.cols != cols || core.rows != rows {
            let fresh: &'static TerminalCore =
                Box::leak(Box::new(TerminalCore::new(key, cols, rows)));
            {
                let mut fresh_cells = fresh.cells.lock().unwrap();
                let old_cells = core.cells.lock().unwrap();
                // 收缩几何时按两者较小值拷贝(旧几何可能大于新几何)。
                let n = (core.rows as usize).min(fresh_cells.len());
                for (i, row) in old_cells.iter().take(n).enumerate() {
                    fresh_cells[i] = row.clone();
                }
            }
            {
                let mut fresh_digests = fresh.digests.lock().unwrap();
                let old_digests = core.digests.lock().unwrap();
                let n = (core.rows as usize).min(fresh_digests.len());
                for (i, digest) in old_digests.iter().take(n).enumerate() {
                    fresh_digests[i] = *digest;
                }
            }
            fresh.cursor.lock().unwrap().clone_from(&core.cursor.lock().unwrap());
            *fresh.blink_ms.lock().unwrap() = *core.blink_ms.lock().unwrap();
            *fresh.selection.lock().unwrap() = core.selection.lock().unwrap().clone();
            *fresh.pending_input.lock().unwrap() = core.pending_input.lock().unwrap().clone();
            *fresh.pending_resize.lock().unwrap() = *core.pending_resize.lock().unwrap();
            fresh.set_scheme(core.scheme());
            fresh
                .scroll_offset
                .store(core.scroll_offset.load(Ordering::Relaxed), Ordering::Relaxed);
            fresh.scroll_view_target.store(
                core.scroll_view_target.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
            fresh.scroll_bound_offset.store(
                core.scroll_bound_offset.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
            fresh.scroll_bind_suppress.store(
                core.scroll_bind_suppress.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
            fresh
                .window_anchor
                .store(core.window_anchor.load(Ordering::Relaxed), Ordering::Relaxed);
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

/// PLAN-018 D4/D5:按 key 取注册表 core(引擎 glue 的定向泵出口:
/// `drain_inputs_for` / `take_resize_for` 需要柄;缺 key = None,调用方
/// no-op)。只读不创建——落表权限仍归渲染面 [`terminal`]。
pub fn terminal_core(key: &str) -> Option<&'static TerminalCore> {
    let map = TERMINALS.lock().unwrap();
    let map = map.as_ref()?;
    map.get(key).copied()
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
///
/// 014: text feed does **not** reset styling — where the incoming char
/// equals the existing cell's char, its fg/bg are kept. Styled rows ride
/// the [`terminal_feed_cells`] sideband (refreshed by the engine glue each
/// harvest); the per-frame props feed must not wipe them between harvests.
pub fn terminal_feed(core: &TerminalCore, lines: &[String]) {
    let existing = core.cells.lock().unwrap().clone();
    let incoming: Vec<Vec<TermCell>> = (0..core.rows as usize)
        .map(|i| {
            let text = lines.get(i).map(String::as_str).unwrap_or("");
            let prev = existing.get(i);
            let mut cells: Vec<TermCell> = Vec::with_capacity(core.cols as usize);
            let mut width = 0usize;
            for (ci, ch) in text.chars().enumerate() {
                if width >= core.cols as usize {
                    break;
                }
                width += char_width(ch);
                let styled = prev.and_then(|p| p.get(ci)).filter(|pc| pc.ch == ch);
                match styled {
                    Some(pc) => cells.push(TermCell { ch, fg: pc.fg, bg: pc.bg }),
                    None => cells.push(TermCell::plain(ch)),
                }
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

/// 014: styled sideband feed for the engine-glue pumps — feed one styled
/// row to **every** registered terminal (the glue runs host-side next to
/// the renderer but doesn't know the widget's key; single-terminal apps
/// are the current consumer shape, same as [`terminal_drain_all_inputs`]).
pub fn terminal_feed_cells_all(row: usize, cells: Vec<TermCell>) {
    let mut map = TERMINALS.lock().unwrap();
    let Some(map) = map.as_mut() else { return };
    for core in map.values() {
        terminal_feed_cells(core, row, cells.clone());
    }
}

/// PLAN-018 D4 per-key 定向投喂:styled sideband 只喂 `key` 对应的
/// terminal(缺 key = no-op)。多 Pane 各 Pane 各 key,广播退役为兼容面
/// (广播三件原样保留,其余消费者零扰)。
pub fn terminal_feed_cells_for(key: &str, row: usize, cells: Vec<TermCell>) {
    let map = TERMINALS.lock().unwrap();
    let Some(map) = map.as_ref() else { return };
    if let Some(core) = map.get(key) {
        terminal_feed_cells(core, row, cells);
    }
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
    // 防御:incoming 按 core.rows 构造,但几何替换窗口内 cells/digests
    // 可能比 core.rows 短(注册表替换与 feed 的交错)——越界即止。
    let n = incoming.len().min(cells.len()).min(digests.len());
    for (i, mut row) in incoming.into_iter().take(n).enumerate() {
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

/// PLAN-025 T-05 实机复测修:把当前 offset 标记为已绑定(用户驱动
/// 滚动回灌路径专用)。iced 滚轮每 notch 60px ≠ CELL_H 行距,读出臂
/// 量化回灌后引擎落整行位——若 bind 写臂仍按行量化 scroll_to 回拉,
/// 每拍 4px 反复回拉 = 滚动条抖动(用户 2026-09-20 实机实录)。用户
/// 滚动产生的 offset 变化源自视图本身(视图已知位),泵排水后标
/// bound 即可;外部源(键入贴底/程序滚动)不标,bind 照常跟随。
pub fn terminal_mark_view_bound(core: &TerminalCore) {
    let off = core.scroll_offset.load(Ordering::Relaxed) as i64;
    core.scroll_bound_offset.store(off, Ordering::Relaxed);
}

/// Wheel/trackpad scroll: adjust the display offset (clamped at ≥0; the
/// upper bound is the app's scrollback height, unenforced here).
pub fn terminal_scroll(core: &TerminalCore, delta: i32) {
    let current = core.scroll_offset.load(Ordering::Relaxed) as i64;
    let next = (current + delta as i64).max(0) as u64;
    core.scroll_offset.store(next, Ordering::Relaxed);
}

/// Explicit offset set (app round-trips the engine's scrollback position).
/// PLAN-022:这也是引擎泵回写的唯一入口——回写即"引擎此刻在该 offset",
/// 同步更新视图投影目标(官方滚动条虚拟滚动的读出基线)。
pub fn terminal_set_scroll_offset(core: &TerminalCore, offset: usize) {
    core.scroll_offset.store(offset as u64, Ordering::Relaxed);
    core.set_scroll_view_target(offset as i64);
}

/// PLAN-019 滚轮回灌:滚轮增量入队(引擎约定:**正=上翻历史**)。组件
/// 本地无历史可滚(视口快照缓存),滚动语义在引擎 display_offset——
/// 宿主引擎泵排水后调引擎 scroll,同拍快照即反映滚动视图。
pub fn terminal_queue_scroll_delta(core: &TerminalCore, delta: i32) {
    *core.scroll_delta.lock().unwrap() += delta;
}

/// PLAN-025 T-04 即时泵探针:滚动增量是否待排(peek,不排空)。泵
/// 节拍门消费——视图先行量超预取覆盖时提前泵,消灭快拖半屏空白。
pub fn terminal_scroll_delta_pending(core: &TerminalCore) -> i32 {
    *core.scroll_delta.lock().unwrap()
}

/// 排水:取走累计滚动增量(读后即清零;引擎泵每拍调用)。
pub fn terminal_take_scroll_delta(core: &TerminalCore) -> i32 {
    std::mem::take(&mut *core.scroll_delta.lock().unwrap())
}

/// 引擎回滚历史行数(泵回读;滚动条比例)。
pub fn terminal_history(core: &TerminalCore) -> usize {
    core.history.load(Ordering::Relaxed)
}

/// 引擎回滚历史行数回写(泵每拍刷新)。
pub fn terminal_set_history(core: &TerminalCore, rows: usize) {
    core.history.store(rows, Ordering::Relaxed);
}

// ============================================================================
// PLAN-025 T-02/T-03:快照窗绝对行锚 + 预取窗存储
// ============================================================================

/// 行窗未锚哨兵(泵未回写;widget 回落槽位键语义)。
pub const WINDOW_ANCHOR_UNSET: i64 = i64::MIN;

/// 泵回写行窗首行绝对行号(与 offset/history 同拍;id = h − o)。
pub fn terminal_set_window_anchor(core: &TerminalCore, anchor: i64) {
    core.window_anchor.store(anchor, Ordering::Relaxed);
}

/// 行窗首行绝对行号([`WINDOW_ANCHOR_UNSET`] = 未锚)。
pub fn terminal_window_anchor(core: &TerminalCore) -> i64 {
    core.window_anchor.load(Ordering::Relaxed)
}

/// 预取窗存储条目(cells + 内容 digest,与槽位面 digest 同一
/// [`row_digest`] 口径)。
pub struct WindowRow {
    pub cells: Vec<TermCell>,
    pub digest: u64,
}

/// 预取窗喂入(per-key 定向;`rows[i]` 的绝对行 id = `base + i`)。
/// 重复喂同 id = 覆盖(digest 变则 widget 重建)。溢出护栏:总量超
/// [`WINDOW_STORE_CAP`] 时按距 base 的距离逐远逐出。
pub fn terminal_feed_window_for(key: &str, base: i64, rows: &[Vec<TermCell>]) {
    let Some(core) = terminal_core(key) else { return };
    let mut store = core.window_store.lock().unwrap();
    for (i, cells) in rows.iter().enumerate() {
        let id = base + i as i64;
        let digest = row_digest(cells);
        store.insert(id, WindowRow { cells: cells.clone(), digest });
    }
    // 溢出逐出:距 base 最远的条目先行(泵窗滚动方向不定,双侧余量)。
    while store.len() > WINDOW_STORE_CAP {
        let far = store
            .keys()
            .max_by_key(|id| (*id - base).abs())
            .copied()
            .expect("len > cap implies non-empty");
        store.remove(&far);
    }
}

/// 预取窗存储上限(条;可见区 + 双侧预取的量级上界)。
pub const WINDOW_STORE_CAP: usize = 512;

/// 读一行预取窗存储(缺 = None;draw 的视口 id 区间消费面)。
pub fn terminal_window_row(core: &TerminalCore, id: i64) -> Option<WindowRow> {
    core.window_store
        .lock()
        .unwrap()
        .get(&id)
        .map(|r| WindowRow { cells: r.cells.clone(), digest: r.digest })
}

/// PLAN-024:虚拟画布总高(px;widget canvas_height 的 glue 侧镜像,
/// 写臂 bind 无 widget self 时换算贴底钳位位置用)。与
/// `Terminal::canvas_height` 虚拟分支同式(rows+history)×CELL_H+2PAD。
/// PLAN-025 合并修:引用 `terminal::iced`(ui-iced 门控模块)——无
/// iced 档(ui-only back 生成体)编译炸,方法随特性门控(component.rs
/// drain_bitmap_uploads 同款收口);消费面(bind 写臂)在 widget 侧
/// 恒 ui-iced。
#[cfg(feature = "ui-iced")]
pub fn terminal_canvas_height(core: &TerminalCore) -> f32 {
    use crate::ui::terminal::iced::widget::{CELL_H, PAD};
    (core.rows as usize + core.history.load(Ordering::Relaxed)) as f32 * CELL_H + 2.0 * PAD
}

/// 焦点空闲(无任何 terminal 持有):启动自动聚焦的门控。
pub fn terminal_focus_free() -> bool {
    FOCUS_OWNER.lock().unwrap().is_none()
}

/// 本 core 是否当前持焦(键盘门控单一事实源)。
/// PLAN-023 T-04 用户门实录(2026-09-19):widget Tree 级 `state.focused`
/// 在他端换焦后残留 true(iced 键盘事件广播全树,无人清旧持者标志)→
/// 双 pane 同键入同执行。输入门控必须查注册表 owner,禁用 per-widget 标志。
pub fn terminal_is_focused(core: &TerminalCore) -> bool {
    FOCUS_OWNER
        .lock()
        .unwrap()
        .as_deref()
        .is_some_and(|k| k == core.key.as_str())
}

/// 持焦(启动自动聚焦/点击换焦;后来者顶替先来者)。
pub fn terminal_claim_focus(core: &TerminalCore) {
    *FOCUS_OWNER.lock().unwrap() = Some(core.key.clone());
}

/// 释放焦点(点击组件外;仅本人持有时)。
pub fn terminal_release_focus(core: &TerminalCore) {
    let mut owner = FOCUS_OWNER.lock().unwrap();
    if owner.as_deref() == Some(core.key.as_str()) {
        *owner = None;
    }
}

/// 菜单动作载荷写入(widget 命中菜单项时;app 在收到 on_menu 后读取)。
pub fn terminal_set_menu_item(core: &TerminalCore, item: u8) {
    *core.menu_item.lock().unwrap() = Some(item);
}

/// 菜单动作载荷读取(0=Copy 1=Paste 2=SelectAll 3=Interrupt/PLAN-015 D1)。
/// **读取即取走**(take)。
pub fn terminal_take_menu_item(core: &TerminalCore) -> Option<u8> {
    core.menu_item.lock().unwrap().take()
}

/// PLAN-015 D2:registry 级菜单载荷取走(任意 terminal;BTreeMap 键序
/// 稳定,首个 Some 即返,镜像 [`terminal_take_any_resize`])——宿主泵
/// 模式无 core 引用侧的载荷出口(at-app db.term_menu_take 消费;单
/// Pane 应用为当前消费形态,多 Pane 定向化留 Mux 后续)。
pub fn terminal_take_menu_item_any() -> Option<u8> {
    let mut map = TERMINALS.lock().unwrap();
    let map = map.as_mut()?;
    for core in map.values() {
        if let Some(item) = core.menu_item.lock().unwrap().take() {
            return Some(item);
        }
    }
    None
}

// ============================================================================
// 键入队列(直键入:widget 键盘捕获 → 队列 → 宿主引擎泵裸写 PTY)
// ============================================================================

/// Queue one keystroke payload (the widget's keyboard capture translates the
/// iced key event into the VT byte string first). Order preserved (FIFO).
/// 014: 有界——积压超过 [`INPUT_QUEUE_CAP`] 丢最旧(消费侧停摆时保护
/// 内存;丢弃计数进 [`terminal_input_dropped`])。
pub const INPUT_QUEUE_CAP: usize = 4096;

pub fn terminal_push_input(core: &TerminalCore, payload: &str) {
    if payload.is_empty() {
        return;
    }
    let mut queue = core.pending_input.lock().unwrap();
    if queue.len() >= INPUT_QUEUE_CAP {
        queue.remove(0);
        INPUT_DROPPED.fetch_add(1, Ordering::Relaxed);
    }
    queue.push(payload.to_owned());
}

/// 累计被挤掉的键入条数(消费侧停摆取证)。
pub fn terminal_input_dropped() -> u64 {
    INPUT_DROPPED.load(Ordering::Relaxed)
}

static INPUT_DROPPED: AtomicU64 = AtomicU64::new(0);

/// Drain this terminal's queued keystrokes (oldest first). Empty = None.
pub fn terminal_drain_input(core: &TerminalCore) -> Option<String> {
    let mut queue = core.pending_input.lock().unwrap();
    if queue.is_empty() {
        None
    } else {
        Some(queue.remove(0))
    }
}

/// Drain **all** registered terminals' keystroke queues, key order stable
/// (BTreeMap). The engine pump in the host process calls this after the
/// `oninput` message: one call flushes every keystroke regardless of which
/// terminal keyed it (single-terminal apps are the current consumer shape).
pub fn terminal_drain_all_inputs() -> Vec<String> {
    let mut map = TERMINALS.lock().unwrap();
    let Some(map) = map.as_mut() else { return Vec::new() };
    let mut out = Vec::new();
    for core in map.values() {
        out.append(&mut core.pending_input.lock().unwrap());
    }
    out
}

/// PLAN-018 D4 定向排空:只取 `core` 自己的键入队列(FIFO 整段取走;
/// 其他 terminal 的队列不受扰)——焦点 Pane 全量泵的载荷出口。
pub fn terminal_drain_inputs_for(core: &TerminalCore) -> Vec<String> {
    let mut queue = core.pending_input.lock().unwrap();
    std::mem::take(&mut *queue)
}

// ============================================================================
// PLAN-018 D10: 配色方案解析面(scheme 表 + palette 缓存 + 主题跟随)
// ============================================================================

/// scheme prop 缺省哨兵:跟随桌面主题(theme::dark_mode:dark→0/light→1)。
pub const TERMINAL_SCHEME_FOLLOW_THEME: i32 = -1;
/// scheme 0 = classic-dark(与 016 定型行为逐字节一致)。
pub const TERMINAL_SCHEME_CLASSIC_DARK: i32 = 0;
/// scheme 1 = light(浅底深字;Windows Terminal "Solarized Light" 官方盘)。
pub const TERMINAL_SCHEME_LIGHT: i32 = 1;

/// 一个方案的 [18] rgb 表:[0]=def-fg、[1]=def-bg、[2..18]=base16。
/// 内置表与 autoterm-core palette.rs 单源同值(渲染端回退基线;引擎在线
/// 时由 glue 经 FFI `palette_color` 查询装载覆盖——引擎单源契约)。
pub const TERMINAL_PALETTE_SLOTS: usize = 18;

/// scheme 0 内置表(016 像素金样逐字节基线)。
pub const PALETTE_CLASSIC_DARK: [u32; TERMINAL_PALETTE_SLOTS] = [
    0xE8E8E8, 0x060709, 0x000000, 0x800000, 0x008000, 0x808000, 0x000080, 0x800080, 0x008080,
    0xC0C0C0, 0x808080, 0xFF0000, 0x00FF00, 0xFFFF00, 0x0000FF, 0xFF00FF, 0x00FFFF, 0xFFFFFF,
];

/// scheme 1 内置表(Solarized Light 族底;2026-09-17 用户实机裁定可读性
/// 修订,与 autoterm-core palette.rs LIGHT 同值同修——白/亮白族反转为
/// 深色(7=base01/8=base00/15=base02,原官方盘 15=FDF6E3 与 bg 同色致
/// cmd 亮白文本隐身)、亮色族 vivid 化、def_fg=base02、def_bg=base2),
/// xterm 序。
pub const PALETTE_LIGHT: [u32; TERMINAL_PALETTE_SLOTS] = [
    0x073642, 0xEEE8D5, 0x002B36, 0xDC322F, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198,
    0x586E75, 0x657B83, 0xCB4B16, 0x859900, 0xB58900, 0x268BD2, 0x6C71C4, 0x2AA198, 0x073642,
];

/// 引擎装载的方案表缓存(scheme id → [18] rgb;glue 经 FFI 查询后写入,
/// 未装载的方案回落内置表——无 DLL/旧 DLL 场景渲染不中断)。
static PALETTES: Mutex<Option<HashMap<i32, [u32; TERMINAL_PALETTE_SLOTS]>>> = Mutex::new(None);

/// glue 装载口(engine 单源覆盖内置表;同 id 重复装载 = 覆盖 = scheme
/// 切换失效语义的装载臂)。
pub fn terminal_palette_load(scheme: i32, table: [u32; TERMINAL_PALETTE_SLOTS]) {
    let mut map = PALETTES.lock().unwrap();
    map.get_or_insert_with(HashMap::new).insert(scheme, table);
}

/// 解析方案生效表:引擎装载缓存优先,否则内置回退表(未知 id 回落
/// classic-dark)。
pub fn terminal_effective_palette(scheme: i32) -> [u32; TERMINAL_PALETTE_SLOTS] {
    if let Ok(map) = PALETTES.lock() {
        if let Some(map) = map.as_ref() {
            if let Some(table) = map.get(&scheme) {
                return *table;
            }
        }
    }
    builtin_palette(scheme)
}

/// 内置回退表(未知 scheme = classic-dark)。
pub fn builtin_palette(scheme: i32) -> [u32; TERMINAL_PALETTE_SLOTS] {
    if scheme == TERMINAL_SCHEME_LIGHT {
        PALETTE_LIGHT
    } else {
        PALETTE_CLASSIC_DARK
    }
}

/// 方案解析:显式 scheme ≥0 直用;否则跟随桌面主题(dark→0/light→1)。
pub fn terminal_resolve_scheme(core: &TerminalCore) -> i32 {
    let scheme = core.scheme();
    if scheme >= 0 {
        return scheme;
    }
    if crate::ui::style::theme::dark_mode() {
        TERMINAL_SCHEME_CLASSIC_DARK
    } else {
        TERMINAL_SCHEME_LIGHT
    }
}

// ============================================================================
// 几何随动(014:窗口 resize → 引擎 resize → View 几何回流)
// ============================================================================

/// resize 请求的安全钳位(glue 快照面按每行 512B / 256 行采样)。
/// 014 爆炸护栏:列数下限 2——1 列几何(最小化/零尺寸产物)会触发引擎侧
/// 折叠重排病理(满历史网格 → GB 级瞬态分配,19:13 案;责任划分与完整
/// 链路见 autoterm DEBTS #15)。退化空间=可见性事件,不该进几何通道;
/// 引擎 TermSession::resize 另有兜底,此处是第一道闸。
pub const MIN_RESIZE_COLS: u16 = 2;
pub const MAX_RESIZE_COLS: u16 = 500;
pub const MAX_RESIZE_ROWS: u16 = 200;

/// Widget layout 由可用空间反推网格几何;与当前几何不同才落位(覆盖
/// 旧请求——只关心最新值)。返回 true = 本次落了待定请求。
pub fn terminal_request_resize(core: &TerminalCore, cols: u16, rows: u16) -> bool {
    let cols = cols.clamp(MIN_RESIZE_COLS, MAX_RESIZE_COLS);
    let rows = rows.clamp(1, MAX_RESIZE_ROWS);
    if cols == core.cols && rows == core.rows {
        return false;
    }
    *core.pending_resize.lock().unwrap() = Some((cols, rows));
    true
}

/// PLAN-028 T-03:分体轨桥写入口——后端进程(无渲染面)按 key 惰性建核
/// 并落 per-key 待定几何,供同进程引擎泵(apply_resize_for)消费。
/// 落表权限从渲染面 [`terminal`] 扩至此处:(0,0) 占位建核(后端不渲染,
/// cells 空表;几何只在引擎侧落位),钳位/同值 no-op 语义与
/// [`terminal_request_resize`] 同款。返回 true = 落了待定请求。
pub fn terminal_pend_resize(key: &str, cols: u16, rows: u16) -> bool {
    let cols = cols.clamp(MIN_RESIZE_COLS, MAX_RESIZE_COLS);
    let rows = rows.clamp(1, MAX_RESIZE_ROWS);
    let core = terminal(key, 0, 0);
    terminal_request_resize(core, cols, rows)
}

/// Take the pending resize request (any terminal; key order stable).
/// `Some((cols, rows))` = the host engine pump should `resize` and surface
/// the new geometry back to the app model.
pub fn terminal_take_any_resize() -> Option<(u16, u16)> {
    let mut map = TERMINALS.lock().unwrap();
    let map = map.as_mut()?;
    for core in map.values() {
        if let Some(geom) = core.pending_resize.lock().unwrap().take() {
            return Some(geom);
        }
    }
    None
}

/// PLAN-018 D4 定向几何出口:只取 `core` 自己的待定几何请求(广播/任意
/// 语义的 `terminal_take_any_resize` 退役为兼容面)——多 Pane 请求互不
/// 串线,焦点 Pane 的 resize 请求由其宿主泵定向消费。
pub fn terminal_take_resize_for(core: &TerminalCore) -> Option<(u16, u16)> {
    core.pending_resize.lock().unwrap().take()
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

    /// PLAN-015 D2:registry 级菜单载荷取走——多终端取走语义(BTreeMap
    /// 键序稳定,首个 Some 即返;取走即清;注册但无载荷 = None)。
    #[test]
    fn menu_item_any_takes_across_terminals() {
        terminal_dispose("p015-menu-1");
        terminal_dispose("p015-menu-2");
        assert_eq!(terminal_take_menu_item_any(), None, "无载荷 = None");
        let a = terminal("p015-menu-1", 40, 6);
        let b = terminal("p015-menu-2", 40, 6);
        assert_eq!(terminal_take_menu_item_any(), None, "注册但无载荷 = None");
        // 单端载荷:跨键可取(宿主泵模式无 core 引用)。
        terminal_set_menu_item(b, 3); // Interrupt
        assert_eq!(terminal_take_menu_item_any(), Some(3));
        assert_eq!(terminal_take_menu_item_any(), None, "取走即清");
        // 双端载荷:键序稳定先取 a(p015-menu-1 < p015-menu-2)。
        terminal_set_menu_item(a, 0);
        terminal_set_menu_item(b, 3);
        assert_eq!(terminal_take_menu_item_any(), Some(0), "键序稳定,先 a");
        assert_eq!(terminal_take_menu_item_any(), Some(3));
        assert_eq!(terminal_take_menu_item_any(), None);
        terminal_dispose("p015-menu-1");
        terminal_dispose("p015-menu-2");
    }

    #[test]
    fn input_queue_fifo_and_drain_all() {
        terminal_dispose("t4-in-1");
        terminal_dispose("t4-in-2");
        let a = terminal("t4-in-1", 40, 6);
        let b = terminal("t4-in-2", 40, 6);
        assert_eq!(terminal_drain_input(a), None, "空队列 drain = None");

        terminal_push_input(a, "a");
        terminal_push_input(a, "\r");
        terminal_push_input(a, ""); // 空载荷丢弃
        terminal_push_input(b, "\u{1b}[A");
        // FIFO 单端排空。
        assert_eq!(terminal_drain_input(a).as_deref(), Some("a"));
        assert_eq!(terminal_drain_input(a).as_deref(), Some("\r"));
        assert_eq!(terminal_drain_input(a), None);

        // drain_all:两个终端的余量一次取走(键序稳定,单端内 FIFO)。
        terminal_push_input(a, "x");
        assert_eq!(terminal_drain_all_inputs(), vec!["x".to_owned(), "\u{1b}[A".to_owned()]);
        assert_eq!(terminal_drain_all_inputs(), Vec::<String>::new(), "排空后为空");
        terminal_dispose("t4-in-1");
        terminal_dispose("t4-in-2");
    }

    #[test]
    fn resize_request_pends_and_takes() {
        terminal_dispose("t4-rs-1");
        let core = terminal("t4-rs-1", 80, 24);
        // 与当前几何相同:不落位。
        terminal_request_resize(core, 80, 24);
        assert_eq!(terminal_take_any_resize(), None);
        // 不同:落位并被取走(覆盖旧请求,只留最新)。
        terminal_request_resize(core, 100, 30);
        terminal_request_resize(core, 120, 40);
        assert_eq!(terminal_take_any_resize(), Some((120, 40)));
        assert_eq!(terminal_take_any_resize(), None, "取走即清");
        // 钳位(014 护栏:列数下限 2——1 列触发引擎重排病理,DEBTS #15)。
        terminal_request_resize(core, 0, u16::MAX);
        assert_eq!(terminal_take_any_resize(), Some((MIN_RESIZE_COLS, MAX_RESIZE_ROWS)));
        terminal_dispose("t4-rs-1");
    }

    // ==== PLAN-018 D4:per-key 定向泵(多 key 互不取走;缺 key no-op)====
    #[test]
    fn palette_fallback_and_engine_override() {
        terminal_dispose("p18-scheme-1");
        let core = terminal("p18-scheme-1", 20, 4);
        // 缺省 = 跟随主题(-1);测试线程 dark_mode 缺省 true → classic-dark。
        assert_eq!(core.scheme(), TERMINAL_SCHEME_FOLLOW_THEME);
        assert_eq!(terminal_resolve_scheme(core), TERMINAL_SCHEME_CLASSIC_DARK);
        // classic-dark 内置表与 016 定型常量逐字节一致(像素金样基线)。
        let pal = terminal_effective_palette(TERMINAL_SCHEME_CLASSIC_DARK);
        assert_eq!(pal, PALETTE_CLASSIC_DARK);
        assert_eq!(pal[0], 0xE8E8E8);
        assert_eq!(pal[1], 0x060709);
        // 显式覆盖:scheme prop ≥0 直用。
        core.set_scheme(TERMINAL_SCHEME_LIGHT);
        assert_eq!(terminal_resolve_scheme(core), TERMINAL_SCHEME_LIGHT);
        // 未装载时 light 走内置表(浅底深字;2026-09-17 可读性修订表)。
        let light = terminal_effective_palette(TERMINAL_SCHEME_LIGHT);
        assert_eq!(light, PALETTE_LIGHT);
        assert_eq!(light[1], 0xEEE8D5);
        assert_eq!(light[17], 0x073642, "亮白槽=base02 深色(非 bg 同色)");
        // 引擎装载覆盖内置(单源生效;同 id 重复装载 = 覆盖)。
        let mut loaded = PALETTE_LIGHT;
        loaded[1] = 0x11_22_33;
        terminal_palette_load(TERMINAL_SCHEME_LIGHT, loaded);
        assert_eq!(terminal_effective_palette(TERMINAL_SCHEME_LIGHT)[1], 0x11_22_33);
        // 未知 scheme 回落 classic-dark(渲染不中断)。
        assert_eq!(terminal_effective_palette(99), PALETTE_CLASSIC_DARK);
        core.set_scheme(TERMINAL_SCHEME_FOLLOW_THEME);
        terminal_dispose("p18-scheme-1");
    }

    #[test]
    fn per_key_feed_cells_is_directional() {
        terminal_dispose("p18-feed-a");
        terminal_dispose("p18-feed-b");
        let a = terminal("p18-feed-a", 20, 4);
        let _b = terminal("p18-feed-b", 20, 4);
        let cells = vec![TermCell { ch: 'x', fg: TermColor::Indexed(1), bg: TermColor::Default }];

        // 缺 key:no-op(不 panic、不投喂任何端)。
        terminal_feed_cells_for("p18-missing", 0, cells.clone());
        assert_eq!(a.line(0).as_deref(), Some(""), "缺 key 喂养不得落到任何端");
        assert_eq!(terminal_take_damage(a), TerminalDamage::None);

        // 命中 key:只喂它,另一端不动。
        terminal_feed_cells_for("p18-feed-a", 0, cells);
        assert_eq!(a.line(0).as_deref(), Some("x"));
        assert_eq!(a.row_cells(0).unwrap()[0].fg, TermColor::Indexed(1));
        assert_eq!(terminal_take_damage(a), TerminalDamage::Lines(vec![0]));
        let _ = _b;
        terminal_dispose("p18-feed-a");
        terminal_dispose("p18-feed-b");
    }

    #[test]
    fn per_key_drain_and_resize_are_directional() {
        terminal_dispose("p18-pump-a");
        terminal_dispose("p18-pump-b");
        let a = terminal("p18-pump-a", 20, 4);
        let b = terminal("p18-pump-b", 20, 4);

        // 键入:A 两键、B 一键——定向排空各取各的,互不串。
        terminal_push_input(a, "a1");
        terminal_push_input(a, "a2");
        terminal_push_input(b, "b1");
        assert_eq!(terminal_drain_inputs_for(a), vec!["a1".to_owned(), "a2".to_owned()]);
        assert_eq!(terminal_drain_inputs_for(a), Vec::<String>::new(), "排空即净");
        assert_eq!(terminal_drain_inputs_for(b), vec!["b1".to_owned()], "A 的排空不得带走 B 的载荷");
        // 广播旧件兼容面仍在(此处两端皆空 → 空)。
        assert_eq!(terminal_drain_all_inputs(), Vec::<String>::new());

        // 几何:A 落请求,定向取只动 A,B 的 None 不受扰。
        terminal_request_resize(a, 60, 20);
        assert_eq!(terminal_take_resize_for(b), None, "A 的请求不得被 B 的定向泵取走");
        assert_eq!(terminal_take_resize_for(a), Some((60, 20)));
        assert_eq!(terminal_take_resize_for(a), None, "取走即清");
        terminal_dispose("p18-pump-a");
        terminal_dispose("p18-pump-b");
    }

    #[test]
    fn pend_resize_bridge_lazy_creates_and_feeds_pump() {
        // PLAN-028 T-03:分体轨桥写入口——后端进程(无渲染面)按 key 惰性
        // 建核落 pending,既有定向泵出口(apply_resize_for 消费面)取走。
        terminal_dispose("p028-bridge");
        assert!(
            terminal_core("p028-bridge").is_none(),
            "前置:注册表无核(terminal_core 只读不建)"
        );
        assert!(terminal_pend_resize("p028-bridge", 100, 30), "首次请求应落位");
        let core = terminal_core("p028-bridge").expect("桥写应惰性建核");
        assert_eq!(terminal_take_resize_for(core), Some((100, 30)));
        assert_eq!(terminal_take_resize_for(core), None, "取走即清");
        // 覆盖旧请求语义:未消费前重推,泵只取最新值。
        assert!(terminal_pend_resize("p028-bridge", 80, 24));
        assert!(terminal_pend_resize("p028-bridge", 120, 40));
        assert_eq!(terminal_take_resize_for(core), Some((120, 40)), "只关心最新值");
        // clamp 面:退化 (0,0) 在本层 clamp 为 (2,1) 落位——退化**拒收**
        // 在 geom 解析层(term_engine p028 测试),本层只负责钳位与
        // 同值 no-op(重推同值:core 占位 (0,0)≠(2,1) 仍落位,去重归
        // 前端推送面)。
        assert!(terminal_pend_resize("p028-bridge", 0, 0));
        assert_eq!(terminal_take_resize_for(core), Some((2, 1)));
        terminal_dispose("p028-bridge");
    }

    #[test]
    fn styled_sideband_survives_text_feed() {
        // 014 at-app 管线:样式经 feed_cells 旁路(引擎 glue 每拍刷),
        // props 文本每帧重喂不得擦色;字符变化处落回默认色。
        terminal_dispose("t5-style-keep");
        let core = terminal("t5-style-keep", 20, 4);
        terminal_feed(core, &["hello".into()]);
        terminal_feed_cells(
            core,
            0,
            vec![
                TermCell { ch: 'h', fg: TermColor::Indexed(1), bg: TermColor::Default },
                TermCell { ch: 'e', fg: TermColor::Indexed(2), bg: TermColor::Default },
                TermCell { ch: 'l', fg: TermColor::Default, bg: TermColor::Indexed(4) },
                TermCell { ch: 'l', fg: TermColor::Indexed(3), bg: TermColor::Default },
                TermCell { ch: 'o', fg: TermColor::Indexed(5), bg: TermColor::Default },
            ],
        );
        assert_eq!(terminal_take_damage(core), TerminalDamage::Lines(vec![0]));

        // 同文本 props 重喂:颜色保留、代数不动(无帧间抖动)。
        let gen = core.generation();
        terminal_feed(core, &["hello".into()]);
        let cells = core.row_cells(0).unwrap();
        assert_eq!(cells[0].fg, TermColor::Indexed(1));
        assert_eq!(cells[2].bg, TermColor::Indexed(4));
        assert_eq!(core.generation(), gen, "同文本重喂应零损伤");

        // 文本变化:变化字符落回默认色,未变字符保留。
        terminal_feed(core, &["hellO".into()]);
        let cells = core.row_cells(0).unwrap();
        assert_eq!(cells[4].ch, 'O');
        assert_eq!(cells[4].fg, TermColor::Default, "变化字符不带旧色");
        assert_eq!(cells[0].fg, TermColor::Indexed(1), "未变字符保留");
        terminal_dispose("t5-style-keep");
    }

    #[test]
    fn geometry_change_shrink_keeps_consistent_lengths() {
        // 014 resize 随动实测踩中:digests 经 clone_from 整体换成旧长度,
        // 增长后 apply_cells 以 digests[i] 越界。收缩/增长都必须保持
        // cells/digests 与新几何等长。
        terminal_dispose("t4-geo-shrink");
        let core = terminal("t4-geo-shrink", 40, 6);
        terminal_feed(core, &["a".into(), "b".into(), "c".into(), "d".into(), "e".into(), "f".into()]);
        // 收缩 6 → 3。
        let core2 = terminal("t4-geo-shrink", 40, 3);
        assert_eq!(core2.rows, 3);
        assert_eq!(core2.line(0).as_deref(), Some("a"));
        assert_eq!(core2.line(2).as_deref(), Some("c"));
        // 增长 3 → 5(feed 5 行不再越界)。
        let core3 = terminal("t4-geo-shrink", 40, 5);
        terminal_feed(core3, &["1".into(), "2".into(), "3".into(), "4".into(), "5".into()]);
        assert_eq!(core3.line(4).as_deref(), Some("5"));
        terminal_dispose("t4-geo-shrink");
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
