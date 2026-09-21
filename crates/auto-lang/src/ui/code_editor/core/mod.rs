// Plan 413 §3.1 ① core layer — the editor state machine.
//
// `CodeEditorCore` owns the cosmic-text ViEditor plus all interaction state
// (focus, drag, multi-click, modifiers, shift anchor, IME preedit, scroll
// geometry). It is rendering-backend agnostic: input arrives as
// `EditorInput` (backend-neutral), output is a `CoreOutput` plus the
// `EditorDrawList` produced by `render`. This module must never import iced
// (hard layering constraint, §8.3) — a separating-render backend maps host
// events onto the same `EditorInput`.
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

pub mod fold;
pub mod highlight;
pub mod render;
// Plan 673 T-04: self-implemented lightweight rope (document source of
// truth, §3.1/§3.2). Standalone in T-04 — not yet wired into the editor.
pub mod rope;

use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

use cosmic_text::{
    Action, Attrs, Buffer, BufferRef, Cursor, Edit, Family, FontSystem, Metrics, Motion, Selection,
    Shaping, SyntaxEditor, ViEditor, Wrap,
};

use super::draw::Rect;
use super::theme::CodeEditorTheme;

pub use super::draw::EditorDrawList;

/// Multi-click window (ms) for double/triple click detection.
const CLICK_TIMING: Duration = Duration::from_millis(400);
/// Scrollbar thumb thickness (logical px).
pub const SCROLLBAR_THICKNESS: f32 = 8.0;

// ---------------------------------------------------------------------------
// Backend-neutral input events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditorModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub logo: bool,
}

impl EditorModifiers {
    pub const fn none() -> Self {
        Self { shift: false, control: false, alt: false, logo: false }
    }

    pub const fn shift(&self) -> bool {
        self.shift
    }

    pub const fn control(&self) -> bool {
        self.control
    }

    pub const fn alt(&self) -> bool {
        self.alt
    }

    pub const fn logo(&self) -> bool {
        self.logo
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorKey {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Backspace,
    Delete,
    Escape,
    Tab,
    /// A plain character key (already composed by the OS layout layer).
    Char(char),
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorButton {
    Left,
    Right,
    Middle,
    Other,
}

/// Backend-neutral editor input. Coordinates are logical, widget-local.
#[derive(Debug, Clone)]
pub enum EditorInput {
    KeyPressed {
        key: EditorKey,
        text: Option<String>,
        modifiers: EditorModifiers,
    },
    KeyReleased,
    ModifiersChanged(EditorModifiers),
    MousePressed {
        button: EditorButton,
        x: f32,
        y: f32,
    },
    MouseReleased {
        button: EditorButton,
    },
    MouseMoved {
        x: f32,
        y: f32,
    },
    /// Wheel scroll in "lines" (positive = down / right).
    WheelScrolled {
        dx: f32,
        dy: f32,
        shift: bool,
    },
    ImeOpened,
    ImePreedit(String),
    ImeCommit(String),
    ImeClosed,
    FocusGained,
    FocusLost,
    /// Display scale factor changed (fraction, 1.0 = 100%).
    Rescaled(f32),
}

/// What a handled input produced — the adapter turns these into messages,
/// redraws and captures.
#[derive(Debug, Clone, Default)]
pub struct CoreOutput {
    /// Buffer text changed since the previous output.
    pub text_changed: bool,
    /// Caret/selection moved.
    pub cursor_changed: bool,
    /// Right-click position (widget-local) for the context-menu callback.
    pub context_menu: Option<(f32, f32)>,
    /// The event was consumed (adapter should stop propagation).
    pub captured: bool,
    /// A repaint is needed.
    pub request_redraw: bool,
}

impl CoreOutput {
    fn captured(mut self) -> Self {
        self.captured = true;
        self.request_redraw = true;
        self
    }
}

/// Backend-neutral clipboard for copy/cut/paste.
pub trait EditorClipboard {
    fn read(&mut self) -> Option<String>;
    fn write(&mut self, text: &str);
}

/// A clipboard with nothing behind it (headless / tests).
pub struct NullClipboard;

impl EditorClipboard for NullClipboard {
    fn read(&mut self) -> Option<String> {
        None
    }
    fn write(&mut self, _text: &str) {}
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Static per-editor configuration (from the DSL `code_editor` tag).
#[derive(Debug, Clone, PartialEq)]
pub struct CodeEditorConfig {
    /// "rust" | "python" | "auto" (AutoLang grammar) | "none".
    pub lang: String,
    pub line_numbers: bool,
    pub wrap: bool,
    pub vi: bool,
    pub highlight_current_line: bool,
    pub tab_width: u16,
    pub font_size: f32,
}

impl Default for CodeEditorConfig {
    fn default() -> Self {
        Self {
            lang: "none".to_owned(),
            line_numbers: true,
            wrap: false,
            vi: false,
            highlight_current_line: true,
            tab_width: 4,
            font_size: 15.0,
        }
    }
}

impl CodeEditorConfig {
    pub fn line_height(&self) -> f32 {
        (self.font_size * 4.0 / 3.0).round().max(self.font_size + 3.0)
    }
}

// ---------------------------------------------------------------------------
// Interaction state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClickKind {
    Single,
    Double,
    Triple,
}

#[derive(Debug, Clone, Copy)]
enum Drag {
    None,
    /// Selecting text in the buffer.
    Buffer,
    /// Dragging the vertical scrollbar thumb (grab offset in px, scroll at
    /// grab time).
    ScrollbarV { grab_offset: f32 },
    /// Dragging the horizontal scrollbar thumb.
    ScrollbarH { grab_offset: f32 },
}

/// `ViEditor` behind a Send wrapper: syntect's onig engine holds raw
/// pointers, so the type is not automatically `Send`. Every access in this
/// module goes through the enclosing `Mutex`, so there is no concurrent
/// access; the onig regex objects themselves are thread-safe when not
/// shared across threads unsynchronized. This wrapper only asserts the
/// move-between-threads bound, matching how the leaked statics in this
/// module are used (single UI-thread app + serialized MCP access).
struct SendEditor(ViEditor<'static, 'static>);

// SAFETY: see the struct docs — all accesses are serialized through the
// containing Mutex; the value is never touched from two threads at once.
unsafe impl Send for SendEditor {}

/// Plan 673 §4: one entry of the unified delta queue (read side). Offsets
/// are UTF-8 byte offsets into the document BEFORE the edit; both endpoints
/// are char boundaries by construction (producers only cut at boundaries).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TextDelta {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

/// The editor state machine. Shared through the global registry as
/// `&'static CodeEditorCore` (interior mutability via Mutex/atomics — iced
/// is single-threaded on the UI thread, and MCP automation may call the
/// payload accessors from its own thread).
pub struct CodeEditorCore {
    key: String,
    config: Mutex<CodeEditorConfig>,
    /// The engine. Locked on the UI thread for every input/render; MCP
    /// set-text also takes this lock.
    editor: Mutex<SendEditor>,

    focused: std::sync::atomic::AtomicBool,
    drag: Mutex<Drag>,
    click: Mutex<Option<(ClickKind, Instant)>>,
    modifiers: Mutex<EditorModifiers>,
    shift_anchor: Mutex<Option<Cursor>>,
    preedit: Mutex<Option<String>>,
    scale: Mutex<f32>,
    /// The last value pushed from the outside (DSL content binding / MCP).
    /// `code_editor_set_text` diffs against THIS, not the live editor text —
    /// a view rebuild that re-syncs an unchanged DSL value must not clobber
    /// the user's in-progress edits.
    last_external: Mutex<Option<String>>,

    /// Geometry from the latest render (widget-local logical px) — drives
    /// scrollbar hit testing and IME caret placement.
    layout_info: Mutex<LayoutInfo>,
    /// Name of the currently applied syntect theme (tracks the semantic
    /// theme source; update_theme is only called on change).
    applied_theme: Mutex<Option<String>>,
    /// Regex search state: (pattern source, compiled). Empty pattern = off.
    search: Mutex<SearchState>,
    /// Monotonic revision — bumped on every text change; adapters key
    /// raster caches on it.
    revision: AtomicU64,
    /// Plan 673 T-05: the rope is the DOCUMENT SOURCE OF TRUTH (design §3.1):
    /// O(1) summaries (bytes/chars/lines), O(log n) point↔offset conversion,
    /// and COW snapshots for background readers. The cosmic Buffer below is
    /// the materialized VIEW of the full document.
    ///
    /// S2 (视口物化 — windowing the Buffer into viewport rows) is DEFERRED
    /// with recorded evidence (commit "T-05 S2 视口物化延后裁定记录"):
    /// render.rs alone has 4 `layout_runs()` consumers (text runs, selection
    /// bands, caret, preedit) keyed by buffer-local line indices plus
    /// fold-map projections (`is_hidden`/`fold_bands`/`project_y`), mouse
    /// hit testing resolves clicks against buffer rows, and
    /// `sync_external_scroll` clamps against `b.lines.len()` — every one of
    /// these assumes buffer line == document line. A correct window needs a
    /// doc↔viewport map at all of those sites plus a row shift routine;
    /// cosmic Buffer exposes no line-window API, so shifts would re-`set_text`
    /// the viewport (full re-shape, defeating the purpose). Rushed, this
    /// breaks the G5 zero-regression gate — the pre-authorized fallback
    /// ruling lands S1 + snapshot API + receipts instead. 100MB receipts
    /// show the motivation quantitatively: open 7.8s debug (acceptable) but
    /// the per-keystroke O(n) delta snapshots cost ~25s at 100MB — the
    /// viewport-local diff (§4.3) that shrinks them also unblocks cheap
    /// window shifts; both land together in the follow-up.
    doc: Mutex<rope::Rope>,
    /// Plan 673 §4: unified delta queue (read side). Producers push on every
    /// text-changing edit; `code_editor_delta` drains it (destructive read).
    delta_queue: Mutex<Vec<TextDelta>>,
    /// Set by native-driven edits (menu/toolbar handlers: undo/redo/cut/
    /// paste) — they mutate the buffer outside the widget event flow, so
    /// the widget consumes this flag on its next `update` and republishes
    /// on_change/on_cursor. Without it, model bindings (e.g. 041's
    /// `.src_main`) go stale and a subsequent Save persists pre-edit text.
    external_dirty: std::sync::atomic::AtomicBool,
    /// LRU stamp for registry sweeping (§5.4 auto-dispose).
    last_used: AtomicU64,
    /// Cached gutter width for `digits` columns.
    gutter_width_cache: Mutex<(usize, f32)>,
    /// Plan 428 P1: folded opener lines (0-based). Fold state is view
    /// state — deliberately NOT in the undo stack; openers that stop being
    /// valid after an edit are pruned by the next render pass.
    folds: Mutex<BTreeSet<usize>>,
    /// Plan 428 P1: the fold map computed by the last render (regions +
    /// merged hidden ranges). Hit testing and the gutter read this.
    fold_map: Mutex<Arc<fold::FoldMap>>,
    /// PLAN-629 T-03: pending follow-scroll target (content-space y), set
    /// by the widget on keyboard/IME caret moves; drained by dispatch_app.
    caret_follow: Mutex<Option<f32>>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LayoutInfo {
    pub(crate) viewport_w: f32,
    pub(crate) viewport_h: f32,
    /// Text area rect (after gutter).
    pub(crate) text: Rect,
    pub(crate) scrollbar_v: Option<Rect>,
    pub(crate) scrollbar_h: Option<Rect>,
    /// Caret rect from the last render, text-area-local.
    pub(crate) caret: Option<Rect>,
    /// Widest line (px).
    pub(crate) max_line_width: f32,
    /// Number of visible layout lines.
    pub(crate) visible_lines: usize,
    /// Plan 428 P2: visible-line bands for folded-view hit testing —
    /// (original line_i, projected top, original top), render order.
    pub(crate) fold_bands: Vec<(usize, f32, f32)>,
    /// Plan 428 P3: the gutter's fold-tool column (chevron click zone),
    /// present only when line numbers are on.
    pub(crate) fold_column: Option<Rect>,
}

/// MutexGuard that derefs to the ViEditor (hides the Send wrapper).
pub(crate) struct EditorGuard<'a> {
    inner: std::sync::MutexGuard<'a, SendEditor>,
}

impl std::ops::Deref for EditorGuard<'_> {
    type Target = ViEditor<'static, 'static>;
    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

impl std::ops::DerefMut for EditorGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.0
    }
}

/// Callback shape: run `with` while holding the shared font system lock.
/// The iced adapter installs one that locks its global
/// `RwLock<iced_graphics FontSystem>` and passes `raw()` — the exact same
/// cosmic-text instance its `fill_raw` pipeline rasterizes with (single
/// text stack, §8.3).
pub type FontSystemCall = fn(with: &mut dyn FnMut(&mut FontSystem));

static FONT_SYSTEM_CALL: OnceLock<FontSystemCall> = OnceLock::new();

/// Compiled regex search state (case-insensitive).
#[derive(Default)]
pub(crate) struct SearchState {
    pub(crate) pattern: String,
    pub(crate) regex: Option<regex::Regex>,
}

impl SearchState {
    fn set(&mut self, pattern: &str) -> bool {
        if self.pattern == pattern {
            return false;
        }
        self.pattern = pattern.to_owned();
        self.regex = if pattern.is_empty() {
            None
        } else {
            regex::RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
                .ok()
        };
        true
    }
}

/// Install the font system access callback (backend init, once).
pub fn set_font_system_call(call: FontSystemCall) {
    let _ = FONT_SYSTEM_CALL.set(call);
}

/// PLAN-674 T-01：缺省字体系统安装（幂等）——无 iced 后端的宿主（RQ
/// 投影器/VM 进程：`auto run -r vm -q` 的 native-queue 臂）在注册表
/// 首次触达前安装进程级 FontSystem 源。OnceLock 语义：已装零影响
/// （不覆盖——混合进程先装缺省后 iced 同装不生效，双栈各自整形
/// 功能等价）。
pub fn ensure_font_system_call() {
    let _ = FONT_SYSTEM_CALL.set(default_font_system);
}

/// 缺省源：进程级单例 FontSystem（首次触达惰性建——FontSystem::new
/// 载字体表较贵，测试/无编辑器进程零成本）。
fn default_font_system(with: &mut dyn FnMut(&mut FontSystem)) {
    static DEFAULT: std::sync::Mutex<Option<FontSystem>> = std::sync::Mutex::new(None);
    let mut guard = DEFAULT.lock().unwrap();
    let fs = guard.get_or_insert_with(FontSystem::new);
    with(fs);
}

/// Run `f` with the shared font system (locks for the duration). Core code
/// never nests this call — input/render paths receive `&mut FontSystem`
/// from their callers.
pub fn with_font_system<R>(f: impl FnOnce(&mut FontSystem) -> R) -> R {
    let call = FONT_SYSTEM_CALL
        .get()
        .copied()
        .expect("code_editor: font system callback not installed (backend init missing)");
    let mut slot: Option<R> = None;
    let mut f = Some(f);
    call(&mut |fs| {
        if let Some(f) = f.take() {
            slot = Some(f(fs));
        }
    });
    slot.unwrap()
}

/// 014: 不 panic 变体——回调未安装(真实 iced 后端之外:headless 测试、
/// 非编辑器渲染路径)返回 None,调用方落近似值。terminal 组件的实测
/// 字宽走此门(PLAN-009 借道共享 font system,零新依赖)。
pub fn try_with_font_system<R>(f: impl FnOnce(&mut FontSystem) -> R) -> Option<R> {
    let call = FONT_SYSTEM_CALL.get().copied()?;
    let mut slot: Option<R> = None;
    let mut f = Some(f);
    (call)(&mut |fs| {
        if let Some(f) = f.take() {
            slot = Some(f(fs));
        }
    });
    slot
}

/// Family for editor body text. `Family::Monospace` goes through
/// cosmic-text's monospace fallback path, which on Windows picks a font
/// whose CJK glyphs come out as tofu boxes; a named real font uses the
/// ordinary fallback chain (→ Microsoft YaHei for Han) that renders CJK
/// correctly. Consolas ships with every Windows; elsewhere keep the generic
/// monospace family.
pub(crate) fn mono_family() -> Family<'static> {
    if cfg!(windows) {
        Family::Name("Consolas")
    } else {
        Family::Monospace
    }
}

impl CodeEditorCore {
    pub fn new(
        key: impl Into<String>,
        config: CodeEditorConfig,
        font_system: &mut FontSystem,
    ) -> Self {
        let key = key.into();
        let attrs = Attrs::new().family(mono_family());

        let mut buffer = Buffer::new(font_system, Metrics::new(config.font_size, config.line_height()));
        buffer.set_text(font_system, "", &attrs, Shaping::Advanced, None);
        buffer.set_wrap(font_system, if config.wrap { Wrap::Word } else { Wrap::None });

        let arc = Arc::new(buffer);

        let system = highlight::syntax_system();
        let mut syntax_editor = SyntaxEditor::new(arc, system, "base16-eighties.dark")
            .expect("bootstrap syntax theme must exist in two-face defaults");
        if let Some(ext) = highlight::lang_to_extension(&config.lang) {
            syntax_editor.syntax_by_extension(ext);
        }
        let mut vi = ViEditor::new(syntax_editor);
        vi.set_passthrough(!config.vi);
        vi.set_auto_indent(true);
        vi.set_tab_width(font_system, config.tab_width.max(1));

        // Background warm-up for this language's highlighter (F: cold-start
        // regex compilation, seconds with onig in debug builds).
        highlight::warm_language(&config.lang);

        let this = Self {
            key,
            config: Mutex::new(config.clone()),
            editor: Mutex::new(SendEditor(vi)),
            focused: std::sync::atomic::AtomicBool::new(false),
            drag: Mutex::new(Drag::None),
            click: Mutex::new(None),
            modifiers: Mutex::new(EditorModifiers::none()),
            shift_anchor: Mutex::new(None),
            preedit: Mutex::new(None),
            scale: Mutex::new(1.0),
            last_external: Mutex::new(None),
            layout_info: Mutex::new(LayoutInfo::default()),
            applied_theme: Mutex::new(None),
            search: Mutex::new(SearchState::default()),
            revision: AtomicU64::new(0),
            delta_queue: Mutex::new(Vec::new()),
            doc: Mutex::new(rope::Rope::new()),
            external_dirty: std::sync::atomic::AtomicBool::new(false),
            last_used: AtomicU64::new(0),
            gutter_width_cache: Mutex::new((0, 0.0)),
            folds: Mutex::new(BTreeSet::new()),
            fold_map: Mutex::new(Arc::new(fold::FoldMap::default())),
            caret_follow: Mutex::new(None),
        };
        this.apply_config_locked(&config, font_system);
        this
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn config(&self) -> CodeEditorConfig {
        self.config.lock().unwrap().clone()
    }

    /// Apply a (possibly changed) configuration. Diffs field by field so
    /// unchanged settings don't reset the editor.
    pub fn apply_config(&self, new_config: &CodeEditorConfig, font_system: &mut FontSystem) {
        self.apply_config_locked(new_config, font_system);
    }

    fn apply_config_locked(&self, new_config: &CodeEditorConfig, font_system: &mut FontSystem) {
        {
            let mut current = self.config.lock().unwrap();
            let lang_changed = current.lang != new_config.lang;
            let wrap_changed = current.wrap != new_config.wrap;
            let vi_changed = current.vi != new_config.vi;
            let tab_changed = current.tab_width != new_config.tab_width;
            let font_changed = current.font_size != new_config.font_size;
            *current = new_config.clone();

            let mut editor = self.editor.lock().unwrap();
            if lang_changed {
                // ViEditor does not expose a syntax setter — rebuild the
                // editor around the same buffer (text/cursor preserved;
                // undo history resets on language switch).
                if let BufferRef::Arc(arc) = editor.0.buffer_ref().clone() {
                    let system = highlight::syntax_system();
                    let mut syntax_editor =
                        SyntaxEditor::new(arc, system, "base16-eighties.dark")
                            .expect("bootstrap syntax theme must exist");
                    if let Some(ext) = highlight::lang_to_extension(&new_config.lang) {
                        syntax_editor.syntax_by_extension(ext);
                    }
                    let cursor = editor.0.cursor();  // raw guard
                    let passthrough = !new_config.vi;
                    let mut vi = ViEditor::new(syntax_editor);
                    vi.set_passthrough(passthrough);
                    vi.set_auto_indent(true);
                    vi.set_tab_width(font_system, new_config.tab_width.max(1));
                    vi.set_cursor(cursor);
                    *editor = SendEditor(vi);
                    *self.applied_theme.lock().unwrap() = None; // force theme re-sync
                }
            }
            if wrap_changed {
                let wrap = if new_config.wrap { Wrap::Word } else { Wrap::None };
                editor.0.with_buffer_mut(|b| b.set_wrap(font_system, wrap));
            }
            if vi_changed {
                editor.0.set_passthrough(!new_config.vi);
            }
            if tab_changed {
                editor.0.set_tab_width(font_system, new_config.tab_width.max(1));
            }
            let _ = font_changed; // metrics applied in render() each frame
        }
    }

    // ── text access ──────────────────────────────────────────────────────

    /// Full text (lines joined with `\n`; buffer lines never contain their
    /// line endings). Reads the ROPE — the document source of truth (T-05):
    /// O(n), same as the previous buffer join, but no editor lock and the
    /// save/readout path benefits from the rope's contiguous leaves.
    pub fn text(&self) -> String {
        self.doc.lock().unwrap().to_string()
    }

    /// Full text read from the cosmic BUFFER view (lines joined with `\n`).
    /// T-05: only the mutation funnels read this — it is the post-edit state
    /// the interval derivation diffs against the pre-edit rope snapshot.
    /// Everyone else goes through `text()` (rope).
    fn buffer_text(&self) -> String {
        self.editor_lock()
            .with_buffer(|b| b.lines.iter().map(|l| l.text()).collect::<Vec<_>>().join("\n"))
    }

    /// Programmatic set (external value diff). Only rewrites when the text
    /// actually differs, preserving cursor/scroll when it matches (§5.4).
    pub fn set_text(&self, text: &str, font_system: &mut FontSystem) {
        let current = self.text();
        if current == text {
            return;
        }
        self.rewrite(text, font_system);
        self.revision.fetch_add(1, Ordering::Relaxed);
        // Plan 673 §4: a whole-document rewrite is one full-replace delta
        // (old_len measured from the pre-rewrite text; the equality guard
        // above already guarantees the text changed).
        self.push_delta(TextDelta { start: 0, end: current.len(), replacement: text.to_string() });
    }

    /// Plan 673 §4.3 structured write (agent/programmatic edit). Splices
    /// `old[..start] + replacement + old[end..]` through the same rewrite
    /// machinery `set_text` uses, but queues the EXACT caller interval
    /// (the caller's offsets are authoritative — 报错不静默: invalid input
    /// logs the reason and returns false, never clamped). Returns true when
    /// the edit was applied (including a valid no-op edit).
    pub fn edit(
        &self,
        start: usize,
        end: usize,
        replacement: &str,
        font_system: &mut FontSystem,
    ) -> bool {
        let old = self.text();
        let old_len = old.len();
        if start > end || end > old_len {
            eprintln!(
                "code_editor_edit: invalid range [{start}, {end}) for {old_len}-byte document"
            );
            return false;
        }
        if !old.is_char_boundary(start) || !old.is_char_boundary(end) {
            eprintln!("code_editor_edit: offsets [{start}, {end}) are not char boundaries");
            return false;
        }
        let mut new = String::with_capacity(old_len - (end - start) + replacement.len());
        new.push_str(&old[..start]);
        new.push_str(replacement);
        new.push_str(&old[end..]);
        if new == old {
            // Valid but effect-free (replacement == replaced span): nothing
            // to rewrite, nothing to queue.
            return true;
        }
        self.rewrite(&new, font_system);
        self.revision.fetch_add(1, Ordering::Relaxed);
        self.push_delta(TextDelta { start, end, replacement: replacement.to_string() });
        true
    }

    /// Rewrite the buffer with new content: viewport-presizing (lazy
    /// shaping), `Buffer::set_text`, cursor clamp/reset. Shared by
    /// `set_text` and `edit`; does NOT bump the revision or queue a delta —
    /// callers own both (they queue different delta shapes).
    fn rewrite(&self, text: &str, font_system: &mut FontSystem) {
        let attrs = Attrs::new().family(mono_family());
        // Give the buffer a viewport before rewriting: with no size set,
        // Buffer::set_text's internal shape_until_scroll treats the scroll
        // window as infinite and shapes/highlights the WHOLE document (26s
        // on a 1MB file). Any finite size keeps it lazy; render() applies
        // the real viewport each frame.
        let info = self.layout_info.lock().unwrap().clone();
        let (w, h) = (
            if info.viewport_w > 1.0 { info.viewport_w } else { 800.0 },
            if info.viewport_h > 1.0 { info.viewport_h } else { 1.0 },
        );
        let mut editor = self.editor_lock();
        editor.with_buffer_mut(|b| {
            b.set_size(font_system, Some(w), Some(h));
            b.set_text(font_system, text, &attrs, Shaping::Advanced, None)
        });
        // Clamp the cursor to the new text and drop any selection — a
        // stale selection past the end would panic the engine later. A byte
        // offset that was valid in the previous text can land inside a
        // multi-byte char in the replacement (e.g. a CJK edit reverted by a
        // value re-sync), so walk back to the nearest char boundary — a
        // mid-char cursor would panic the engine's next insert_at.
        editor.set_selection(Selection::None);
        let mut cursor = editor.cursor();
        editor.with_buffer(|b| {
            cursor.line = cursor.line.min(b.lines.len().saturating_sub(1));
            let text = b.lines.get(cursor.line).map(|l| l.text()).unwrap_or("");
            let mut index = cursor.index.min(text.len());
            while !text.is_char_boundary(index) {
                index -= 1;
            }
            cursor.index = index;
        });
        editor.set_cursor(cursor);
        drop(editor);
        // Plan 673 T-05: the rope is the document source of truth — rebuild
        // it from the rewritten text. `rewrite` serves the LOW-frequency
        // full-document paths (set_text re-sync, agent `code_editor_edit`),
        // which already pay an O(n) buffer rewrite, so the O(n) rope rebuild
        // is the same asymptotics (justified per design §3.2: the per-
        // keystroke paths go through the interval edit instead). Lock
        // order: the editor guard is dropped before the doc lock.
        *self.doc.lock().unwrap() = rope::Rope::from_str(text);
    }

    /// Plan 673 §4: append one delta to the unified queue. T-01 producer is
    /// `set_text`; T-02 instruments the remaining edit sites (keystrokes,
    /// undo/redo, cut/paste).
    pub(crate) fn push_delta(&self, d: TextDelta) {
        self.delta_queue.lock().unwrap().push(d);
    }

    /// Plan 673 §4.3: derive the single minimal contiguous interval between
    /// a pre-edit and post-edit text (PURE — no rope side effects; the unit
    /// test drives it with arbitrary pairs). Prefix/suffix scans walk char
    /// boundaries only (never slice mid-char). Identical texts give None.
    /// The one-interval form is exact for any single localized edit —
    /// keystroke, IME commit, undo/redo, cut/paste — and needs no diff
    /// dependency.
    fn derive_interval(old: &str, new: &str) -> Option<TextDelta> {
        if old == new {
            return None;
        }
        let old_len = old.len();
        let new_len = new.len();
        // Common prefix, char-boundary exclusive end.
        let mut prefix = 0;
        for ((_, oc), (ni, nc)) in old.char_indices().zip(new.char_indices()) {
            if oc != nc {
                break;
            }
            prefix = ni + nc.len_utf8();
        }
        // Common suffix (byte length), char-boundary start; never overlaps
        // the prefix region on EITHER string (the replacement slice is
        // new[prefix..new_len - suffix] — both guards keep it valid).
        let mut suffix = 0;
        for ((oi, oc), (ni, nc)) in old.char_indices().rev().zip(new.char_indices().rev()) {
            if oc != nc || oi < prefix || ni < prefix {
                break;
            }
            suffix += oc.len_utf8();
        }
        Some(TextDelta {
            start: prefix,
            end: old_len - suffix,
            replacement: new[prefix..new_len - suffix].to_string(),
        })
    }

    /// Plan 673 T-05: the single commit path for typed/native edits — derive
    /// the interval (pure), apply it to the ROPE (the document source of
    /// truth — one code path keeps interval derivation, rope mutation, and
    /// delta emission consistent by construction), and queue the delta.
    ///
    /// Callers pass `old` = the pre-edit ROPE text and `new` = the post-edit
    /// BUFFER text (`buffer_text()`), so the rope stays the diff baseline.
    fn push_delta_from_texts(&self, old: &str, new: &str) {
        let Some(d) = Self::derive_interval(old, new) else {
            return;
        };
        // The delta interval IS the rope edit — O(log n) locate + O(k).
        self.doc.lock().unwrap().replace_bytes(d.start, d.end, &d.replacement);
        self.push_delta(d);
    }

    /// Plan 673 §4: destructive read — drain everything queued since the
    /// last call, paired with the current revision (post-consumption
    /// watermark). A same-watermark re-read returns empty deltas.
    pub fn take_deltas(&self) -> (u64, Vec<TextDelta>) {
        let deltas = std::mem::take(&mut *self.delta_queue.lock().unwrap());
        (self.revision.load(Ordering::Relaxed), deltas)
    }

    /// Mark that a native-driven edit (undo/redo/cut/paste from a menu or
    /// toolbar handler) changed the buffer outside the widget event flow.
    /// The iced widget consumes this on its next `update` and republishes
    /// on_change/on_cursor so model bindings resync.
    pub fn mark_external_dirty(&self) {
        self.external_dirty.store(true, Ordering::Release);
    }

    /// Consume the external-dirty flag (see `mark_external_dirty`).
    pub fn take_external_dirty(&self) -> bool {
        self.external_dirty.swap(false, Ordering::AcqRel)
    }

    /// (line_0based, char_col, selection_bytes)
    pub fn cursor_info(&self) -> (usize, usize, usize) {
        let editor = self.editor_lock();
        let cursor = editor.cursor();
        let col = editor.with_buffer(|b| {
            b.lines
                .get(cursor.line)
                .map(|l| l.text().get(..cursor.index).map(|s| s.chars().count()).unwrap_or(0))
                .unwrap_or(0)
        });
        let sel = editor
            .selection_bounds()
            .map(|(start, end)| {
                editor.with_buffer(|b| {
                    let mut bytes = end.index.saturating_sub(start.index);
                    for line in start.line..end.line {
                        bytes += b.lines.get(line).map(|l| l.text().len()).unwrap_or(0);
                    }
                    bytes
                })
            })
            .unwrap_or(0);
        (cursor.line, col, sel)
    }

    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    pub fn is_focused(&self) -> bool {
        self.focused.load(Ordering::Relaxed)
    }

    pub fn set_focused(&self, focused: bool) {
        self.focused.store(focused, Ordering::Relaxed);
    }

    /// Caret rectangle in widget-local coordinates (for the IME input-area
    /// request). Valid after the first render.
    pub fn caret_rect(&self) -> Option<Rect> {
        let info = self.layout_info.lock().unwrap().clone();
        info.caret.map(|c| Rect::new(info.text.x + c.x, info.text.y + c.y, c.w, c.h))
    }

    pub fn preedit(&self) -> Option<String> {
        self.preedit.lock().unwrap().clone()
    }

    /// Make sure the syntect highlight theme matches the semantic theme
    /// source (dark + accent). Cheap when nothing changed; on change the
    /// synthesized theme is registered under its stable name and the
    /// editor re-highlights.
    pub fn sync_syntax_theme(&self, dark: bool, accent: &str) {
        // PLAN-601 T-10: the key carries the ACTIVE theme id — switching
        // themes (set_theme / set_theme_composed + epoch invalidation)
        // produces a new key and the editor re-applies with theme-derived
        // bg/fg (active_code_theme).
        let theme_id = crate::ui::style::theme::theme_name();
        let name = highlight::theme_name(&theme_id, dark, accent);
        let mut applied = self.applied_theme.lock().unwrap();
        if applied.as_deref() == Some(name.as_str()) {
            return;
        }
        let theme = crate::ui::code_editor::theme::active_code_theme(dark, accent);
        highlight::register_theme(&name, theme.syntax_theme());
        self.editor_lock().update_theme(&name);
        *applied = Some(name);
    }

    pub fn scale(&self) -> f32 {
        *self.scale.lock().unwrap()
    }

    /// Set the regex search pattern ("" clears). Returns true when the
    /// pattern changed (the adapter should request a repaint). Invalid
    /// regexes clear highlighting and keep the pattern for the next edit.
    pub fn set_search(&self, pattern: &str) -> bool {
        self.search.lock().unwrap().set(pattern)
    }

    /// Current search pattern.
    pub fn search_pattern(&self) -> String {
        self.search.lock().unwrap().pattern.clone()
    }

    /// Jump to the next regex match after the caret, selecting it and
    /// scrolling it into view (wraps around). Returns false when there is
    /// no active search or no match.
    ///
    /// Plan 673 T-05: the line scan runs over a ROPE SNAPSHOT taken before
    /// the editor lock — snapshot isolation (design §3.1) in a real path:
    /// the search reads a stable document while edits proceed, and never
    /// competes for the editor lock.
    pub fn find_next(&self, font_system: &mut FontSystem) -> bool {
        let regex = match self.search.lock().unwrap().regex.clone() {
            Some(r) => r,
            None => return false,
        };
        let snap = self.doc.lock().unwrap().snapshot();
        let mut editor = self.editor_lock();
        let start = editor.cursor();
        let line_count = snap.line_count();

        // (line, byte start, byte end) of the next match at-or-after the
        // caret, wrapping around the document.
        let mut found: Option<(usize, usize, usize)> = None;
        for offset in 0..=line_count {
            let line_i = (start.line + offset) % line_count.max(1);
            let text = snap.line(line_i).into_owned();
            let from = if offset == 0 { start.index } else { 0 };
            let hit = regex.find_iter(&text[from.min(text.len())..]).next().map(|m| {
                let s = from + m.start();
                let e = from + m.end();
                (line_i, s, e)
            });
            if let Some(hit) = hit {
                // On the wrap-around pass, stop before reaching the start
                // position again.
                found = Some(hit);
                break;
            }
        }
        let Some((line, s, e)) = found else {
            return false;
        };

        editor.set_cursor(Cursor::new(line, e));
        editor.set_selection(Selection::Normal(Cursor::new(line, s)));
        // Plan 428 P4: a match inside a folded region reveals it — the
        // caret must never rest on an invisible line. `unfold_line`
        // re-locks the editor itself (fresh region discovery), so the
        // guard must be released here and re-taken for the scroll adjust.
        drop(editor);
        self.unfold_line(line);
        let mut editor = self.editor_lock();
        editor.with_buffer_mut(|b| {
            // Bring the matched line a few lines below the viewport top.
            let mut scroll = b.scroll();
            let target = line.saturating_sub(2);
            if target < scroll.line || line >= scroll.line + 20 {
                scroll.line = target;
                b.set_scroll(scroll);
            }
            b.set_redraw(true);
        });
        let _ = font_system;
        true
    }

    pub(crate) fn editor_lock(&self) -> EditorGuard<'_> {
        EditorGuard { inner: self.editor.lock().unwrap() }
    }

    pub(crate) fn record_layout(&self, info: LayoutInfo) {
        *self.layout_info.lock().unwrap() = info;
    }
    pub(crate) fn gutter_width_cache(&self) -> (usize, f32) {
        *self.gutter_width_cache.lock().unwrap()
    }

    pub(crate) fn set_gutter_width_cache(&self, digits: usize, width: f32) {
        *self.gutter_width_cache.lock().unwrap() = (digits, width);
    }

    pub(crate) fn search_regex(&self) -> Option<regex::Regex> {
        self.search.lock().unwrap().regex.clone()
    }

    // ── Plan 428 P1: code folding ────────────────────────────────────────

    /// Toggle the fold at `line_0` (0-based). Returns `true` when the region
    /// is folded afterwards, `false` when unfolded or the line carries no
    /// foldable opener. Toggling is view state — no undo entry, no
    /// `text_changed` (revision untouched).
    pub fn fold_toggle(&self, line_0: usize) -> bool {
        // Region validity is computed FRESH (native callers may toggle
        // before any render stored a map); the render map only owns the
        // y-projection geometry.
        let map = self.fresh_fold_map();
        if map.region_at(line_0).is_none() {
            return false;
        }
        let mut folds = self.folds.lock().unwrap();
        if !folds.insert(line_0) {
            folds.remove(&line_0);
            false
        } else {
            true
        }
    }

    /// Regions + merged bodies computed from the CURRENT text (the render
    /// map may be stale or absent — natives and tests toggle headless).
    /// Plan 673 T-05: reads the rope (document source of truth) — O(n) line
    /// walk over rope slices without touching the editor lock.
    fn fresh_fold_map(&self) -> fold::FoldMap {
        let folded = self.folds.lock().unwrap().clone();
        let line_height = self.config.lock().unwrap().line_height();
        let doc = self.doc.lock().unwrap();
        let owned: Vec<String> = doc.lines().map(|l| l.into_owned()).collect();
        let texts: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        fold::FoldMap::build(fold::regions_from_texts(&texts), &folded, line_height)
    }

    /// Whether `line_0` currently opens a folded region.
    pub fn fold_is_folded(&self, line_0: usize) -> bool {
        self.folds.lock().unwrap().contains(&line_0)
    }

    /// Unfold every region whose body contains `line_0` (P4 auto-expand:
    /// cursor or search landing inside a hidden range reveals it).
    /// Returns `true` when anything was unfolded.
    pub fn unfold_line(&self, line_0: usize) -> bool {
        let map = self.fresh_fold_map();
        let Some((a, b)) = map.hidden_range_containing(line_0) else {
            return false;
        };
        // The hidden range body [a, b] may merge several folded regions
        // (nested folds); drop every opener whose body intersects it.
        let mut folds = self.folds.lock().unwrap();
        let before = folds.len();
        folds.retain(|opener| {
            map.region_at(*opener)
                .map(|r| r.end < a || r.opener + 1 > b)
                .unwrap_or(true)
        });
        folds.len() != before
    }

    /// Plan 428 P4: reveal any fold hiding the caret's line (keyboard
    /// motion can walk into a folded body; the caret must never rest on an
    /// invisible line). Merges a repaint request into `out` when it fires.
    pub(crate) fn auto_unfold_at_cursor(&self, out: &mut CoreOutput) {
        let line = self.editor_lock().cursor().line;
        if self.unfold_line(line) {
            out.request_redraw = true;
        }
    }

    /// Number of lines currently hidden by folding (tests / MCP assertions).
    pub fn fold_hidden_count(&self) -> usize {
        self.fresh_fold_map().hidden_count()
    }

    /// The fold map snapshot from the last render (hit testing).
    pub(crate) fn fold_map_snapshot(&self) -> Arc<fold::FoldMap> {
        self.fold_map.lock().unwrap().clone()
    }

    /// Currently folded opener lines (render builds the map from these).
    pub(crate) fn folded_openers(&self) -> BTreeSet<usize> {
        self.folds.lock().unwrap().clone()
    }

    /// Render-side update: store the freshly computed map and prune fold
    /// openers that are no longer valid foldable lines (text changed under
    /// them). Returns the number of hidden lines for the cache key.
    pub(crate) fn set_fold_map(&self, map: fold::FoldMap) -> usize {
        let hidden = map.hidden_count();
        {
            let mut folds = self.folds.lock().unwrap();
            folds.retain(|opener| map.region_at(*opener).is_some());
        }
        *self.fold_map.lock().unwrap() = Arc::new(map);
        hidden
    }

    // ── input handling ───────────────────────────────────────────────────

    /// Feed one backend-neutral input. The caller owns the font system
    /// guard (single UI-thread lock discipline: editor lock is taken inside,
    /// never the font system lock re-entrantly).
    pub fn handle_input(
        &self,
        font_system: &mut FontSystem,
        input: EditorInput,
        clipboard: &mut dyn EditorClipboard,
    ) -> CoreOutput {
        match input {
            EditorInput::FocusGained => {
                self.focused.store(true, Ordering::Relaxed);
                CoreOutput { request_redraw: true, ..CoreOutput::default() }
            }
            EditorInput::FocusLost => {
                self.focused.store(false, Ordering::Relaxed);
                *self.preedit.lock().unwrap() = None;
                *self.drag.lock().unwrap() = Drag::None;
                CoreOutput { request_redraw: true, ..CoreOutput::default() }
            }
            EditorInput::Rescaled(f) => {
                *self.scale.lock().unwrap() = f;
                CoreOutput::default()
            }
            EditorInput::ModifiersChanged(m) => {
                // Track the shift anchor: pressing shift marks the current
                // caret as the selection anchor; releasing clears it.
                let mut anchor = self.shift_anchor.lock().unwrap();
                if m.shift && !anchor.is_some() {
                    *anchor = Some(self.editor_lock().cursor());
                } else if !m.shift {
                    *anchor = None;
                }
                *self.modifiers.lock().unwrap() = m;
                CoreOutput::default()
            }
            EditorInput::KeyReleased => CoreOutput::default(),

            EditorInput::KeyPressed { key, text, modifiers } => {
                *self.modifiers.lock().unwrap() = modifiers;
                if !self.focused.load(Ordering::Relaxed) {
                    return CoreOutput::default();
                }
                let mut out =
                    self.handle_key(font_system, key, text, modifiers, clipboard);
                // Plan 428 P4: arrow/edit motion may walk the caret into a
                // folded body — reveal it before the frame paints.
                self.auto_unfold_at_cursor(&mut out);
                out
            }

            EditorInput::MousePressed { button, x, y } => {
                self.handle_mouse_press(font_system, button, x, y)
            }
            EditorInput::MouseReleased { button } => {
                *self.drag.lock().unwrap() = Drag::None;
                if button == EditorButton::Left || button == EditorButton::Right {
                    CoreOutput::default().captured()
                } else {
                    CoreOutput::default()
                }
            }
            EditorInput::MouseMoved { x, y } => self.handle_mouse_move(font_system, x, y),
            EditorInput::WheelScrolled { dx, dy, shift } => {
                self.handle_wheel(font_system, dx, dy, shift).captured()
            }

            EditorInput::ImeOpened => CoreOutput { request_redraw: true, ..CoreOutput::default() },
            EditorInput::ImePreedit(p) => {
                *self.preedit.lock().unwrap() = if p.is_empty() { None } else { Some(p) };
                CoreOutput { request_redraw: true, ..CoreOutput::default() }.captured()
            }
            EditorInput::ImeCommit(content) => {
                *self.preedit.lock().unwrap() = None;
                // Plan 673 §4.3: IME commits are typed edits — same stream.
                let old = self.text();
                let mut editor = self.editor_lock();
                editor.insert_string(&content, None);
                drop(editor);
                self.bump_after_edit();
                self.push_delta_from_texts(&old, &self.buffer_text());
                CoreOutput {
                    text_changed: true,
                    cursor_changed: true,
                    ..CoreOutput::default()
                }
                .captured()
            }
            EditorInput::ImeClosed => {
                *self.preedit.lock().unwrap() = None;
                CoreOutput { request_redraw: true, ..CoreOutput::default() }.captured()
            }
        }
    }

    /// Plan 673 §4.3 三来源同流: typed keys join the same delta queue as
    /// agent writes and set_text. Wraps `handle_key_inner` with a
    /// before/after text snapshot, deriving one minimal interval delta per
    /// actual mutation. Pure motions/selection keys skip the O(N) read
    /// (`key_may_mutate`). Texts are compared rather than the revision
    /// watermark: the Ctrl+Z/Y undo/redo arms historically do NOT bump the
    /// revision, so it cannot serve as the changed-signal here.
    fn handle_key(
        &self,
        font_system: &mut FontSystem,
        key: EditorKey,
        text: Option<String>,
        modifiers: EditorModifiers,
        clipboard: &mut dyn EditorClipboard,
    ) -> CoreOutput {
        if !Self::key_may_mutate(&key, text.as_deref(), modifiers) {
            return self.handle_key_inner(font_system, key, text, modifiers, clipboard);
        }
        let old = self.text();
        let out = self.handle_key_inner(font_system, key, text, modifiers, clipboard);
        self.push_delta_from_texts(&old, &self.buffer_text());
        out
    }

    /// Whether a key event can mutate the buffer text — motions, selection
    /// and scroll inputs are excluded (delta snapshot skipped).
    fn key_may_mutate(key: &EditorKey, text: Option<&str>, modifiers: EditorModifiers) -> bool {
        // These map to buffer-mutating actions REGARDLESS of modifiers — the
        // action mapping below ignores modifiers for them (Ctrl+Backspace/
        // Delete = word delete, Ctrl+Enter still inserts a newline). T-05:
        // an over-strict gate here desyncs the rope (the document source of
        // truth) from the buffer, since text() now reads the rope.
        match key {
            EditorKey::Enter | EditorKey::Backspace | EditorKey::Delete => return true,
            _ => {}
        }
        if modifiers.control() || modifiers.alt() || modifiers.logo {
            // Only the X/V/Z/Y chords mutate (cut/paste/undo/redo); C/A are
            // copy/select-all and unhandled chords bubble out.
            return matches!(
                key,
                EditorKey::Char('x' | 'X' | 'v' | 'V' | 'z' | 'Z' | 'y' | 'Y')
            );
        }
        match key {
            // Ctrl/Alt+Tab bubbles without an action (early return inside).
            EditorKey::Tab => true,
            EditorKey::Char(c) => !c.is_control(),
            // Non-character keys mutate only when carrying a text payload
            // (the Char/Other arm inserts it).
            EditorKey::Other(_) => text
                .map(|t| !t.is_empty() && !t.chars().any(|c| c.is_control()))
                .unwrap_or(false),
            _ => false,
        }
    }

    fn handle_key_inner(
        &self,
        font_system: &mut FontSystem,
        key: EditorKey,
        text: Option<String>,
        modifiers: EditorModifiers,
        clipboard: &mut dyn EditorClipboard,
    ) -> CoreOutput {
        // Clipboard / history shortcuts first (work in both passthrough and
        // vi modes; the vi parser gets the raw keys for everything else).
        // Non-character ctrl combos (Ctrl+Backspace/Delete, Ctrl+arrows)
        // fall through to the regular mapping below.
        if modifiers.control() {
            if let EditorKey::Char(c) = &key {
                let out = match c {
                'c' | 'C' => {
                    if let Some(selection) = self.editor_lock().copy_selection() {
                        clipboard.write(&selection);
                    }
                    CoreOutput::default()
                }
                'x' | 'X' => {
                    let mut editor = self.editor_lock();
                    if let Some(selection) = editor.copy_selection() {
                        clipboard.write(&selection);
                        editor.action(font_system, Action::Backspace);
                        drop(editor);
                        self.bump_after_edit();
                        CoreOutput { text_changed: true, cursor_changed: true, ..CoreOutput::default() }
                    } else {
                        CoreOutput::default()
                    }
                }
                'v' | 'V' => {
                    if let Some(contents) = clipboard.read() {
                        self.editor_lock().insert_string(&contents, None);
                        self.bump_after_edit();
                        CoreOutput { text_changed: true, cursor_changed: true, ..CoreOutput::default() }
                    } else {
                        CoreOutput::default()
                    }
                }
                'z' | 'Z' => {
                    let mut editor = self.editor_lock();
                    // Clear the selection first: a selection spanning text
                    // that undo is about to remove would panic the engine's
                    // delete_range on the next edit.
                    editor.set_selection(Selection::None);
                    if modifiers.shift {
                        editor.redo();
                    } else {
                        editor.undo();
                    }
                    CoreOutput { text_changed: true, cursor_changed: true, ..CoreOutput::default() }
                }
                'y' | 'Y' => {
                    let mut editor = self.editor_lock();
                    editor.set_selection(Selection::None);
                    editor.redo();
                    CoreOutput { text_changed: true, cursor_changed: true, ..CoreOutput::default() }
                }
                'a' | 'A' => {
                    let mut editor = self.editor_lock();
                    let end = editor.with_buffer(|b| {
                        Cursor::new(
                            b.lines.len().saturating_sub(1),
                            b.lines.last().map(|l| l.text().len()).unwrap_or(0),
                        )
                    });
                    editor.set_cursor(Cursor::new(0, 0));
                    editor.set_selection(Selection::Normal(end));
                    CoreOutput { cursor_changed: true, ..CoreOutput::default() }
                }
                    // Unhandled ctrl+letters must BUBBLE: the app-level
                    // shortcut layer (action-config fallback, Plan 418 P2-4)
                    // only receives uncaptured events — force-capturing here
                    // would eat shortcuts like Ctrl+J/Ctrl+D/Ctrl+S whenever
                    // the editor holds focus (same principle as Ctrl+Tab
                    // bubbling below).
                    _ => return CoreOutput::default(),
                };
                return out.captured();
            }
        }

        let motion = match key {
            EditorKey::Left => Some(Motion::Left),
            EditorKey::Right => Some(Motion::Right),
            EditorKey::Up => Some(Motion::Up),
            EditorKey::Down => Some(Motion::Down),
            EditorKey::Home => Some(Motion::Home),
            EditorKey::End => Some(Motion::End),
            EditorKey::PageUp => Some(Motion::PageUp),
            EditorKey::PageDown => Some(Motion::PageDown),
            _ => None,
        };
        if let Some(motion) = motion {
            self.apply_motion(font_system, motion, modifiers);
            return CoreOutput { cursor_changed: true, ..CoreOutput::default() }.captured();
        }

        let action = match key {
            EditorKey::Enter => Action::Enter,
            EditorKey::Escape => Action::Escape,
            EditorKey::Tab => {
                if modifiers.control() || modifiers.alt() {
                    return CoreOutput::default();
                }
                if modifiers.shift {
                    Action::Unindent
                } else {
                    Action::Indent
                }
            }
            EditorKey::Backspace => {
                // Ctrl+Backspace: delete the word behind the caret.
                if modifiers.control() {
                    self.select_word_before(font_system, Motion::PreviousWord);
                }
                Action::Backspace
            }
            EditorKey::Delete => {
                if modifiers.control() {
                    self.select_word_after(font_system, Motion::NextWord);
                }
                Action::Delete
            }
            // Motions were handled above; anything else unexpected is a no-op.
            EditorKey::Left
            | EditorKey::Right
            | EditorKey::Up
            | EditorKey::Down
            | EditorKey::Home
            | EditorKey::End
            | EditorKey::PageUp
            | EditorKey::PageDown => return CoreOutput::default(),
            EditorKey::Char(_) | EditorKey::Other(_) => {
                // Plain text input only without chord modifiers (matches
                // upstream text_editor and cosmic-edit).
                if modifiers.logo || modifiers.control || modifiers.alt {
                    return CoreOutput::default();
                }
                if let Some(text) = text.filter(|t| !t.is_empty()) {
                    if text.chars().any(|c| c.is_control()) {
                        return CoreOutput::default();
                    }
                    let mut editor = self.editor_lock();
                    for c in text.chars() {
                        editor.action(font_system, Action::Insert(c));
                    }
                    drop(editor);
                    self.bump_after_edit();
                    return CoreOutput {
                        text_changed: true,
                        cursor_changed: true,
                        ..CoreOutput::default()
                    }
                    .captured();
                }
                // Fallback: some platforms/layouts deliver plain characters
                // as Key::Char without a `text` payload (e.g. Windows with the
                // IME context disabled). Insert the character directly — but
                // only while no IME composition is active, or the raw letters
                // would double up with the committed text.
                if self.preedit.lock().unwrap().is_none() {
                    if let EditorKey::Char(c) = key {
                        if !c.is_control() {
                            let mut editor = self.editor_lock();
                            editor.action(font_system, Action::Insert(c));
                            drop(editor);
                            self.bump_after_edit();
                            return CoreOutput {
                                text_changed: true,
                                cursor_changed: true,
                                ..CoreOutput::default()
                            }
                            .captured();
                        }
                    }
                }
                return CoreOutput::default();
            }
        };

        let mut editor = self.editor_lock();
        editor.action(font_system, action);
        drop(editor);
        self.bump_after_edit();
        CoreOutput {
            text_changed: true,
            cursor_changed: true,
            ..CoreOutput::default()
        }
        .captured()
    }

    /// Cursor motion with Ctrl word/buffer jumps and Shift selection
    /// (anchored by the shift-anchor tracked on modifier changes).
    fn apply_motion(&self, font_system: &mut FontSystem, motion: Motion, modifiers: EditorModifiers) {
        let motion = match (motion, modifiers.control()) {
            (Motion::Left, true) => Motion::LeftWord,
            (Motion::Right, true) => Motion::RightWord,
            (Motion::Home, true) => Motion::BufferStart,
            (Motion::End, true) => Motion::BufferEnd,
            (m, _) => m,
        };
        let mut editor = self.editor_lock();
        if modifiers.shift {
            if matches!(editor.selection(), Selection::None) {
                let anchor = self.shift_anchor.lock().unwrap().unwrap_or_else(|| editor.cursor());
                editor.set_selection(Selection::Normal(anchor));
            }
        } else if !matches!(editor.selection(), Selection::None) {
            editor.set_selection(Selection::None);
        }
        editor.action(font_system, Action::Motion(motion));
    }

    /// Select from the caret to the previous word boundary (for
    /// Ctrl+Backspace, which then deletes the selection).
    fn select_word_before(&self, font_system: &mut FontSystem, motion: Motion) {
        let mut editor = self.editor_lock();
        let caret = editor.cursor();
        editor.set_selection(Selection::Normal(caret));
        editor.action(font_system, Action::Motion(motion));
    }

    /// Select from the caret to the next word boundary (Ctrl+Delete).
    fn select_word_after(&self, font_system: &mut FontSystem, motion: Motion) {
        let mut editor = self.editor_lock();
        let caret = editor.cursor();
        editor.set_selection(Selection::Normal(caret));
        editor.action(font_system, Action::Motion(motion));
    }

    fn handle_mouse_press(
        &self,
        font_system: &mut FontSystem,
        button: EditorButton,
        x: f32,
        y: f32,
    ) -> CoreOutput {
        let info = self.layout_info.lock().unwrap().clone();

        if !Rect::new(0.0, 0.0, info.viewport_w, info.viewport_h).contains(super::draw::Pt::new(x, y))
        {
            // Click outside the widget unfocuses (cosmic-edit behavior).
            self.focused.store(false, Ordering::Relaxed);
            return CoreOutput { request_redraw: true, ..CoreOutput::default() };
        }
        self.focused.store(true, Ordering::Relaxed);
        let mut out = CoreOutput { request_redraw: true, ..CoreOutput::default() };

        match button {
            EditorButton::Right => {
                out.context_menu = Some((x, y));
                return out.captured();
            }
            EditorButton::Left => {}
            _ => return out,
        }

        // Scrollbar hit testing (on top of the buffer).
        if let Some(sb) = info.scrollbar_v {
            if sb.contains(super::draw::Pt::new(x, y)) {
                let grab_offset = (y - sb.y).clamp(0.0, sb.h);
                *self.drag.lock().unwrap() = Drag::ScrollbarV { grab_offset };
                self.drag_scrollbar_v(font_system, y);
                return out.captured();
            }
        }
        if let Some(sb) = info.scrollbar_h {
            if sb.contains(super::draw::Pt::new(x, y)) {
                let grab_offset = (x - sb.x).clamp(0.0, sb.w);
                *self.drag.lock().unwrap() = Drag::ScrollbarH { grab_offset };
                self.drag_scrollbar_h(font_system, x);
                return out.captured();
            }
        }

        // Plan 428 P3: fold-column hit — a click on a foldable opener's
        // chevron band toggles that region. The bands are the projected
        // visible-line tops from the last render.
        if let Some(fc) = info.fold_column {
            if fc.contains(super::draw::Pt::new(x, y)) {
                let map = self.fold_map_snapshot();
                let mut hit_line = None;
                for &(line_i, proj_top, _orig) in info.fold_bands.iter() {
                    if proj_top <= y && y < proj_top + map.line_height {
                        hit_line = Some(line_i);
                    }
                }
                if let Some(line_i) = hit_line {
                    if map.region_at(line_i).is_some() {
                        self.fold_toggle(line_i);
                        // Folding is view state — no text_changed, but
                        // the frame must repaint (chevron + body).
                        return out.captured();
                    }
                }
                // A miss in the fold column falls through to text click
                // (the column overlaps nothing else).
            }
        }

        // Click inside the text area: multi-click cycle + shift anchor.
        let (scroll_x, scroll_y) = {
            let editor = self.editor_lock();
            editor.with_buffer(|b| {
                let s = b.scroll();
                (s.horizontal, s.vertical)
            })
        };
        // Plan 428 P2: y arrives in FOLDED-VIEW coordinates; map back to
        // the original document y through the visible bands before handing
        // it to cosmic's hit machinery (which knows nothing of folding).
        let by_view = (y - info.text.y).max(0.0);
        let by_orig_view = if info.fold_bands.is_empty() {
            by_view
        } else {
            self.fold_map_snapshot()
                .unfold_y(by_view, &info.fold_bands)
                .unwrap_or(by_view)
        };
        let bx = (x - info.text.x + scroll_x).max(0.0) as i32;
        let by = (by_orig_view + scroll_y).max(0.0) as i32;

        let click_kind = {
            let mut click = self.click.lock().unwrap();
            let kind = match click.take() {
                Some((kind, at)) if at.elapsed() < CLICK_TIMING => match kind {
                    ClickKind::Single => ClickKind::Double,
                    ClickKind::Double => ClickKind::Triple,
                    ClickKind::Triple => ClickKind::Single,
                },
                _ => ClickKind::Single,
            };
            *click = Some((kind, Instant::now()));
            kind
        };

        {
            let mut editor = self.editor_lock();
            // Shift+click extends the current selection from its anchor.
            let anchor = if self.modifiers.lock().unwrap().shift {
                self.shift_anchor.lock().unwrap().or(Some(editor.cursor()))
            } else {
                None
            };
            if let Some(anchor) = anchor {
                editor.set_selection(Selection::Normal(anchor));
            }
            let action = match click_kind {
                ClickKind::Single => Action::Click { x: bx, y: by },
                ClickKind::Double => Action::DoubleClick { x: bx, y: by },
                ClickKind::Triple => Action::TripleClick { x: bx, y: by },
            };
            editor.action(font_system, action);
        }
        *self.drag.lock().unwrap() = Drag::Buffer;
        out.cursor_changed = true;
        out.captured()
    }

    fn handle_mouse_move(&self, font_system: &mut FontSystem, x: f32, y: f32) -> CoreOutput {
        let info = self.layout_info.lock().unwrap().clone();
        // PLAN-626 rev2 T-09: clone the drag state out — the match scrutinee
        // temporary held the `drag` mutex across the whole match body, and
        // drag_scrollbar_v/h re-lock it → guaranteed self-deadlock on the
        // first mouse move of any scrollbar drag ("not responding").
        let drag = *self.drag.lock().unwrap();
        match drag {
            Drag::None => CoreOutput::default(),
            Drag::Buffer => {
                // Auto-scroll when dragging past the visible edges.
                let (scroll_x, scroll_y) = {
                    let editor = self.editor_lock();
                    editor.with_buffer(|b| {
                        let s = b.scroll();
                        (s.horizontal, s.vertical)
                    })
                };
                // Plan 428 P2: same folded-view → original y mapping as the
                // press path (drag selection must follow the drawn lines).
                let by_view = (y - info.text.y).max(0.0);
                let by_fold = if info.fold_bands.is_empty() {
                    by_view
                } else {
                    self.fold_map_snapshot()
                        .unfold_y(by_view, &info.fold_bands)
                        .unwrap_or(by_view)
                };
                let mut by = (by_fold + scroll_y).max(0.0) as i32;
                if y > info.viewport_h {
                    let mut editor = self.editor_lock();
                    editor.action(font_system, Action::Scroll { pixels: info.viewport_h - y });
                    by = (info.viewport_h - info.text.y + scroll_y).max(0.0) as i32;
                } else if y < info.text.y {
                    let mut editor = self.editor_lock();
                    editor.action(font_system, Action::Scroll { pixels: y - info.text.y });
                    by = scroll_y.max(0.0) as i32;
                }
                let bx = (x - info.text.x + scroll_x).max(0.0) as i32;
                let mut editor = self.editor_lock();
                editor.action(font_system, Action::Drag { x: bx, y: by });
                CoreOutput { cursor_changed: true, request_redraw: true, ..CoreOutput::default() }
                    .captured()
            }
            Drag::ScrollbarV { .. } => {
                self.drag_scrollbar_v(font_system, y);
                CoreOutput { request_redraw: true, ..CoreOutput::default() }.captured()
            }
            Drag::ScrollbarH { .. } => {
                self.drag_scrollbar_h(font_system, x);
                CoreOutput { request_redraw: true, ..CoreOutput::default() }.captured()
            }
        }
    }

    /// Map a vertical scrollbar drag to a buffer scroll line.
    fn drag_scrollbar_v(&self, font_system: &mut FontSystem, y: f32) {
        let info = self.layout_info.lock().unwrap().clone();
        let grab = match *self.drag.lock().unwrap() {
            Drag::ScrollbarV { grab_offset } => grab_offset,
            _ => return,
        };
        let Some(track_h) = info.scrollbar_v.map(|sb| sb.h) else { return };
        let total_lines = self.editor_lock().with_buffer(|b| b.lines.len()).max(1);
        // Thumb of height `thumb_h` moves in a track of `track_h`; line
        // position proportional to thumb center.
        let thumb_h = (info.viewport_h * (info.visible_lines.max(1) as f32 / total_lines as f32))
            .clamp(SCROLLBAR_THICKNESS, track_h);
        let travel = (track_h - thumb_h).max(1.0);
        let center = (y - grab + thumb_h / 2.0 - info.scrollbar_v.map(|s| s.y).unwrap_or(0.0))
            .clamp(0.0, track_h);
        let line = ((center - thumb_h / 2.0).max(0.0) / travel * (total_lines as f32 - 1.0))
            .round() as usize;
        let mut editor = self.editor_lock();
        editor.with_buffer_mut(|b| {
            let mut scroll = b.scroll();
            scroll.line = line;
            // PLAN-626 rev2 T-09: drag lands the target line at the viewport
            // top — zero the intra-line offset, then shape + normalize so the
            // next frame's layout_runs always sees shaped lines. Previously
            // the stale vertical + unshaped target line made layout_runs end
            // early (first_visible_line stuck at usize::MAX → NaN scrollbar
            // geometry → frozen frame loop while dragging).
            scroll.vertical = 0.0;
            b.set_scroll(scroll);
            b.shape_until_scroll(font_system, false);
        });
    }

    /// Map a horizontal scrollbar drag to a horizontal pixel scroll.
    fn drag_scrollbar_h(&self, _font_system: &mut FontSystem, x: f32) {
        let info = self.layout_info.lock().unwrap().clone();
        let grab = match *self.drag.lock().unwrap() {
            Drag::ScrollbarH { grab_offset } => grab_offset,
            _ => return,
        };
        let Some(sb) = info.scrollbar_h else { return };
        let max_w = info.max_line_width.max(1.0);
        let frac = ((x - grab - sb.x).clamp(0.0, sb.w)) / sb.w.max(1.0);
        let mut editor = self.editor_lock();
        editor.with_buffer_mut(|b| {
            let mut scroll = b.scroll();
            scroll.horizontal = (frac * (max_w - b.size().0.unwrap_or(max_w)).max(0.0))
                .clamp(0.0, max_w);
            b.set_scroll(scroll);
        });
    }

    fn handle_wheel(
        &self,
        font_system: &mut FontSystem,
        dx: f32,
        dy: f32,
        shift: bool,
    ) -> CoreOutput {
        let config = self.config.lock().unwrap().clone();
        let mut editor = self.editor_lock();
        let before = editor.with_buffer(|b| b.scroll());
        if (shift && dx == 0.0) || dx != 0.0 {
            // Shift+wheel (or horizontal wheel) → horizontal scroll.
            let amount = if dx != 0.0 { dx } else { dy };
            editor.with_buffer_mut(|b| {
                let mut scroll = b.scroll();
                scroll.horizontal = (scroll.horizontal + amount * config.font_size * 0.8).max(0.0);
                b.set_scroll(scroll);
            });
        } else {
            // PLAN-626 rev2 T-08: winit 滚轮向下 y 为负，而 cosmic-text 的
            // scroll.vertical 向下增大——取负对齐（此前方向反直觉）。
            // Action::Scroll 只累加不归一：scroll.line 停在 0，每帧
            // layout_runs 从头全文件行走，大文件（T-05 起打开真实仓库
            // 文件）滚轮风暴拖垮事件循环 = 窗口"未响应"；且上下游都不
            // 钳制，越过底/顶后内容滚丢、永远停不住。滚轮后立即
            // shape_until_scroll 归一 scroll.line 并钳进有效区间。
            let pixels = -dy * config.line_height();
            editor.action(font_system, Action::Scroll { pixels });
            editor.with_buffer_mut(|b| b.shape_until_scroll(font_system, false));
        }
        let after = editor.with_buffer(|b| b.scroll());
        if before == after {
            // Scroll position unchanged (already at a boundary): no repaint.
            return CoreOutput::default();
        }
        CoreOutput { request_redraw: true, ..CoreOutput::default() }
    }

    fn bump_after_edit(&self) {
        self.revision.fetch_add(1, Ordering::Relaxed);
    }

    // ── PLAN-629 T-01: hosted-scroller integration ─────────────────────
    // The widget lives inside the COMMON AutoUI scroller (Plan 629): the
    // scroller owns the scrollbar UI + wheel handling; the editor owns
    // virtualized rendering. Three contract pieces:
    //   content_height      → 高度上报（scroller 更新滚动范围/thumb 比例）
    //   sync_external_scroll → scroller offset → 内部 scroll（归一化复用
    //                          PLAN-626 T-08 的 shape_until_scroll 机制）
    //   caret_offset_y       → 光标跟随（scroll_to 命令的目标位）

    /// Fold-aware content height: effective visible line count × line
    /// height. Backed by the last render's fold map; before any render it
    /// degrades to the full line count. The scroller re-measures whenever
    /// this changes (folds toggling → request_layout → 高度变化通知).
    pub fn content_height(&self) -> f32 {
        // Plan 673 T-05: total from the rope summary — O(1), no editor lock.
        let total = self.doc.lock().unwrap().line_count().max(1);
        // Fresh fold map (not the last render's snapshot): folds toggle in
        // update — the height report must be current by the time the next
        // layout pass queries it, one frame earlier than a render would be.
        let hidden = self.fresh_fold_map().hidden_count();
        let effective = total.saturating_sub(hidden).max(1);
        let line_height = self.config.lock().unwrap().line_height();
        effective as f32 * line_height
    }

    /// Sync the editor's internal scroll to an ABSOLUTE content-space pixel
    /// offset (the hosted scroller is the source of truth). Form (b) from
    /// the plan's bounded verification: coarse line pre-position first
    /// (uniform line height), then `shape_until_scroll` fixes the residual
    /// and clamps both ends. Form (a) — raw vertical + normalize — advances
    /// one line per pass (O(N) scan each) and janks deep thumb jumps on
    /// large files.
    pub fn sync_external_scroll(&self, font_system: &mut FontSystem, offset_y: f32) {
        let config = self.config.lock().unwrap().clone();
        let offset = offset_y.max(0.0);
        let lh = config.line_height().max(1.0);
        let mut editor = self.editor_lock();
        editor.with_buffer_mut(|b| {
            let coarse = ((offset / lh) as usize).min(b.lines.len().saturating_sub(1));
            let residual = offset - coarse as f32 * lh;
            let mut scroll = b.scroll();
            if scroll.line == coarse && (scroll.vertical - residual).abs() < 1.0 {
                return; // already converged (idempotent per frame)
            }
            scroll.line = coarse;
            scroll.vertical = residual;
            b.set_scroll(scroll);
            b.shape_until_scroll(font_system, false);
        });
    }

    /// Caret top in content space — the scroll-to-caret command target.
    /// Computed directly (fold projection over the uniform line-height grid)
    /// instead of reading the last render's layout record: the caret is
    /// usually OFF-viewport when a follow-scroll is needed, so a render-
    /// derived value would not exist. Wrap mode under-estimates by prior
    /// wrapped rows (uniform-height approximation; auto-edit ships wrap off).
    /// Queue a follow-scroll to `target_y` (content-space offset). The
    /// session funnel drains it the same message pass (scroll_to task).
    pub fn request_caret_follow(&self, target_y: f32) {
        *self.caret_follow.lock().unwrap() = Some(target_y);
    }

    /// Drain the pending follow-scroll request, if any.
    pub fn take_caret_follow(&self) -> Option<f32> {
        self.caret_follow.lock().unwrap().take()
    }

    /// Line height from the active config (hosted follow-scroll math).
    pub fn config_line_height(&self) -> f32 {
        self.config.lock().unwrap().line_height()
    }

    pub fn caret_offset_y(&self) -> Option<f32> {
        let map = self.fresh_fold_map();
        let editor = self.editor_lock();
        let cursor = editor.cursor();
        if map.is_hidden(cursor.line) {
            // P4 auto-expand reveals it on the next input; nothing sane to
            // scroll to while still hidden.
            return None;
        }
        let line_height = self.config.lock().unwrap().line_height();
        Some(map.project_y(cursor.line, cursor.line as f32 * line_height))
    }


    // ── Plan 418: programmatic actions (menu/toolbar handlers) ──────────
    // Same semantics as the Ctrl+Z/Y/A/C/X/V arms of handle_key, callable
    // from VM handler natives via the registry functions below. The OS
    // clipboard goes through the arboard bridge (`ui::clipboard`) — handler
    // context has no iced clipboard handle to pass as EditorClipboard.

    fn do_undo(&self) {
        // Plan 673 §4.3: native-driven edits join the same delta queue.
        let old = self.text();
        let mut editor = self.editor_lock();
        // Clear the selection first: a selection spanning text that undo is
        // about to remove would panic the engine's delete_range later.
        editor.set_selection(Selection::None);
        editor.undo();
        drop(editor);
        self.bump_after_edit();
        self.push_delta_from_texts(&old, &self.buffer_text());
    }

    fn do_redo(&self) {
        let old = self.text();
        let mut editor = self.editor_lock();
        editor.set_selection(Selection::None);
        editor.redo();
        drop(editor);
        self.bump_after_edit();
        self.push_delta_from_texts(&old, &self.buffer_text());
    }

    fn do_select_all(&self) {
        let mut editor = self.editor_lock();
        let end = editor.with_buffer(|b| {
            Cursor::new(
                b.lines.len().saturating_sub(1),
                b.lines.last().map(|l| l.text().len()).unwrap_or(0),
            )
        });
        editor.set_cursor(Cursor::new(0, 0));
        editor.set_selection(Selection::Normal(end));
    }

    #[cfg(feature = "ui-clipboard")]
    fn do_copy(&self) {
        if let Some(selection) = self.editor_lock().copy_selection() {
            crate::ui::clipboard::clipboard_set(&selection);
        }
    }

    #[cfg(feature = "ui-clipboard")]
    fn do_cut(&self, font_system: &mut FontSystem) {
        let old = self.text();
        let mut editor = self.editor_lock();
        if let Some(selection) = editor.copy_selection() {
            crate::ui::clipboard::clipboard_set(&selection);
            editor.action(font_system, Action::Backspace);
            drop(editor);
            self.bump_after_edit();
            self.push_delta_from_texts(&old, &self.buffer_text());
        }
    }

    #[cfg(feature = "ui-clipboard")]
    fn do_paste(&self) {
        if let Some(contents) = crate::ui::clipboard::clipboard_get() {
            let old = self.text();
            self.editor_lock().insert_string(&contents, None);
            self.bump_after_edit();
            self.push_delta_from_texts(&old, &self.buffer_text());
        }
    }
}

// ---------------------------------------------------------------------------
// Global keyed storage (TEXTAREA_CONTENTS pattern, §1.3 / §5.4)
// ---------------------------------------------------------------------------

lazy_static::lazy_static! {
    static ref CODE_EDITORS: Mutex<HashMap<String, &'static CodeEditorCore>> =
        Mutex::new(HashMap::new());
}

/// Namespace prefix for editor storage keys (mirrors the textarea's
/// `__code_editor_{widget}_{event}` convention).
pub fn storage_key(widget: &str) -> String {
    format!("__code_editor_{widget}")
}

/// Normalize a payload key: the registry stores editors under
/// `__code_editor_{key}`, but `.at` handlers and a2r codegen call the payload
/// natives with the raw DSL key (e.g. `code_editor_text("editor1")`). Accept
/// both forms — prefix the raw key, pass full storage keys through unchanged.
fn normalize_payload_key(key: &str) -> String {
    if key.starts_with("__code_editor_") {
        key.to_string()
    } else {
        storage_key(key)
    }
}

/// Get or create the core for `key`, applying `config` (diffed). The core
/// is leaked once and lives for the process (explicit disposal via
/// [`code_editor_dispose`], §5.4).
/// Registry capacity: editors beyond this are auto-disposed LRU (the
/// leaked cores themselves persist, but their registry slots — and thus
/// their identity/state — recycle, bounding growth for long-running apps
/// that route across many pages; §5.4).
const CODE_EDITOR_LRU_CAP: usize = 32;

fn lru_tick() -> u64 {
    use std::sync::atomic::AtomicU64 as A64;
    static TICK: A64 = A64::new(0);
    TICK.fetch_add(1, Ordering::Relaxed)
}

pub fn code_editor(key: &str, config: &CodeEditorConfig) -> &'static CodeEditorCore {
    let mut map = CODE_EDITORS.lock().unwrap();
    if let Some(core) = map.get(key) {
        let core: &'static CodeEditorCore = core;
        core.last_used.store(lru_tick(), Ordering::Relaxed);
        with_font_system(|fs| core.apply_config(config, fs));
        return core;
    }
    // LRU sweep: drop the stalest entries beyond the capacity.
    if map.len() >= CODE_EDITOR_LRU_CAP {
        let mut stamped: Vec<(u64, String)> = map
            .iter()
            .map(|(k, core)| (core.last_used.load(Ordering::Relaxed), k.clone()))
            .collect();
        stamped.sort_unstable();
        let excess = map.len() + 1 - CODE_EDITOR_LRU_CAP;
        for (_, k) in stamped.into_iter().take(excess) {
            map.remove(&k);
        }
    }
    let core: &'static CodeEditorCore = with_font_system(|fs| {
        Box::leak(Box::new(CodeEditorCore::new(key, config.clone(), fs)))
    });
    core.last_used.store(lru_tick(), Ordering::Relaxed);
    map.insert(key.to_owned(), core);
    core
}

/// Explicitly dispose an editor (route-change cleanup, §5.4).
pub fn code_editor_dispose(key: &str) {
    let key = normalize_payload_key(key);
    let mut map = CODE_EDITORS.lock().unwrap();
    // The core itself stays leaked (safe); dropping the map entry releases
    // the registry slot and lets the key be reused fresh.
    map.remove(&key);
}

/// Read the current text of an editor (payload accessor, §3.2).
pub fn code_editor_text(key: &str) -> Option<String> {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    map.get(&key).map(|core| core.text())
}

/// Plan 673 §4.2: destructive read of the unified delta queue, serialized
/// as `{"revision":N,"deltas":[{"start":n,"end":n,"replacement":"s"},...]}`.
/// Shape is constant — an empty queue yields `"deltas":[]` at the current
/// watermark. None = no editor registered for `key`.
pub fn code_editor_delta(key: &str) -> Option<String> {
    #[derive(serde::Serialize)]
    struct Envelope {
        revision: u64,
        deltas: Vec<TextDelta>,
    }
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    map.get(&key).map(|core| {
        let (revision, deltas) = core.take_deltas();
        serde_json::to_string(&Envelope { revision, deltas })
            .expect("delta envelope serialization cannot fail")
    })
}

/// PLAN-629 T-01: caret top in content space (hosted-scroller scroll-to-
/// caret command target). None = unknown editor or no layout record yet.
pub fn code_editor_caret_offset_y(key: &str) -> Option<f32> {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    map.get(&key).and_then(|core| core.caret_offset_y())
}

/// PLAN-629 T-03: drain pending follow-scroll requests from ALL editors —
/// (key, content-space target offset) pairs; the session funnel turns them
/// into `operation::scroll_to` tasks targeting `editor-scroll-<key>`.
pub fn code_editor_drain_caret_follows() -> Vec<(String, f32)> {
    let map = CODE_EDITORS.lock().unwrap();
    let mut out = Vec::new();
    for (key, core) in map.iter() {
        if let Some(y) = core.take_caret_follow() {
            out.push((key.clone(), y));
        }
    }
    out
}

/// Read the cursor position of an editor: (line 0-based, char column,
/// selection length in bytes).
pub fn code_editor_cursor(key: &str) -> Option<(usize, usize, usize)> {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    map.get(&key).map(|core| core.cursor_info())
}

/// Programmatic set-text by key (MCP automation / app code). Returns true
/// when the text changed. The diff is against the last EXTERNAL value, not
/// the live editor text: the DSL content binding re-pushes its value on
/// every view rebuild, and an unchanged value must leave the user's
/// in-progress edits alone (a naive current-text diff would revert every
/// keystroke).
pub fn code_editor_set_text(key: &str, text: &str) -> bool {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    if let Some(core) = map.get(&key) {
        let mut last = core.last_external.lock().unwrap();
        if last.as_deref() == Some(text) {
            return false;
        }
        *last = Some(text.to_owned());
        drop(last);
        with_font_system(|fs| core.set_text(text, fs));
        true
    } else {
        false
    }
}

/// Plan 673 §4.3 structured write (agent edit): `code_editor_edit(key,
/// start, end, replacement)` — one API, three forms (insert/delete/replace
/// per the interval shape). Returns false on invalid input (no such editor,
/// start > end, out of bounds, or offsets not on char boundaries — 报错不
/// 静默) and queues nothing.
///
/// This path intentionally does NOT touch `last_external` (the registry
/// `code_editor_set_text` diff guard): after an agent edit, a view rebuild
/// re-pushing the stale DSL value hits the `last_external` equality
/// early-return at `code_editor_set_text` and does not clobber the edit.
/// The queued delta enters the SAME stream as human typing — consumers of
/// `code_editor_delta` see agent and human writes identically.
pub fn code_editor_edit(key: &str, start: usize, end: usize, replacement: &str) -> bool {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    match map.get(&key) {
        Some(core) => with_font_system(|fs| core.edit(start, end, replacement, fs)),
        None => {
            eprintln!("code_editor_edit: no editor registered for key {key:?}");
            false
        }
    }
}

/// Run a closure with the core registered under `key` (if any).
pub fn code_editor_with<R>(key: &str, f: impl FnOnce(&CodeEditorCore) -> R) -> Option<R> {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    map.get(&key).map(|core| f(core))
}

/// Jump to the next search match of the editor under `key` (wraps).
/// Returns false when the editor, pattern or match is missing.
pub fn code_editor_find(key: &str) -> bool {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    if let Some(core) = map.get(&key) {
        with_font_system(|fs| core.find_next(fs))
    } else {
        false
    }
}

/// Plan 418: programmatic undo on the editor under `key`. Returns true when
/// an editor exists (undo itself no-ops on empty history).
pub fn code_editor_undo(key: &str) -> bool {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    map.get(&key).map(|core| { core.do_undo(); core.mark_external_dirty(); }).is_some()
}

/// Plan 418: programmatic redo (mirrors the Ctrl+Y arm of handle_key).
pub fn code_editor_redo(key: &str) -> bool {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    map.get(&key).map(|core| { core.do_redo(); core.mark_external_dirty(); }).is_some()
}

/// Plan 418: select all text (mirrors the Ctrl+A arm of handle_key).
pub fn code_editor_select_all(key: &str) -> bool {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    map.get(&key).map(|core| core.do_select_all()).is_some()
}

/// Plan 418: menu-driven cut/copy/paste via the OS clipboard (arboard
/// bridge — handler context has no iced clipboard handle). Cut needs the
/// font system for the deletion action, same as the Ctrl+X arm.
#[cfg(feature = "ui-clipboard")]
pub fn code_editor_clipboard_op(key: &str, op: ClipboardOp) -> bool {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    if let Some(core) = map.get(&key) {
        match op {
            // Cut/Paste change the text → resync model bindings via the
            // external-dirty flag; Copy only touches the OS clipboard.
            ClipboardOp::Cut => with_font_system(|fs| core.do_cut(fs)),
            ClipboardOp::Copy => core.do_copy(),
            ClipboardOp::Paste => core.do_paste(),
        }
        if !matches!(op, ClipboardOp::Copy) {
            core.mark_external_dirty();
        }
        true
    } else {
        false
    }
}

/// Which clipboard operation `code_editor_clipboard_op` performs.
#[cfg(feature = "ui-clipboard")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipboardOp {
    Cut,
    Copy,
    Paste,
}

/// Shared lock for tests that touch the global editor registry (the LRU
/// sweep in `code_editor()` can evict editors other tests are using).
#[cfg(test)]
pub(crate) static REGISTRY_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Number of live editors (diagnostics/tests).
pub fn code_editor_count() -> usize {
    CODE_EDITORS.lock().unwrap().len()
}

/// Resolve the effective theme for an editor and make sure the syntect
/// theme is registered under its stable name.
pub fn registered_theme_name(theme: &CodeEditorTheme, dark: bool, accent: &str) -> String {
    let theme_id = crate::ui::style::theme::theme_name();
    let name = highlight::theme_name(&theme_id, dark, accent);
    highlight::register_theme(&name, theme.syntax_theme())
}

// ---------------------------------------------------------------------------
// Core-layer tests — no backend required (layering promise: the state
// machine and render contract run headless, without iced).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::RwLock;

    fn none_mods() -> EditorModifiers {
        EditorModifiers::none()
    }

    fn ctrl_mods() -> EditorModifiers {
        EditorModifiers { control: true, ..EditorModifiers::none() }
    }

    fn shift_mods() -> EditorModifiers {
        EditorModifiers { shift: true, ..EditorModifiers::none() }
    }

    #[test]
    fn core_pipeline_types_selects_undoes_and_renders() {
        let mut fs = FontSystem::new();
        let config = CodeEditorConfig {
            lang: "rust".to_owned(),
            ..CodeEditorConfig::default()
        };
        let core = CodeEditorCore::new("test-core-pipeline", config, &mut fs);
        core.set_focused(true);
        core.set_text("hello world", &mut fs);
        assert_eq!(core.text(), "hello world");

        let mut clip = NullClipboard;
        let press = |key: EditorKey, text: Option<&str>, m: EditorModifiers| EditorInput::KeyPressed {
            key,
            text: text.map(|t| t.to_owned()),
            modifiers: m,
        };

        // End + type
        let out = core.handle_input(&mut fs, press(EditorKey::End, None, none_mods()), &mut clip);
        assert!(out.cursor_changed);
        let out = core.handle_input(
            &mut fs,
            press(EditorKey::Char('!'), Some("!"), none_mods()),
            &mut clip,
        );
        assert!(out.text_changed && out.captured);
        assert_eq!(core.text(), "hello world!");

        // Shift+Home selects
        core.handle_input(&mut fs, EditorInput::ModifiersChanged(shift_mods()), &mut clip);
        core.handle_input(&mut fs, press(EditorKey::Home, None, shift_mods()), &mut clip);
        let (start, end) = core.editor_lock().selection_bounds().expect("selection");
        assert_eq!((start.index, end.index), (0, 12));

        // Undo removes the '!'
        core.handle_input(&mut fs, press(EditorKey::Char('z'), None, ctrl_mods()), &mut clip);
        assert_eq!(core.text(), "hello world");

        // IME commit inserts
        core.handle_input(&mut fs, EditorInput::ImeCommit("你".to_owned()), &mut clip);
        assert!(core.text().contains('你'));

        // Ctrl+Backspace deletes a word
        core.handle_input(
            &mut fs,
            press(EditorKey::Backspace, None, ctrl_mods()),
            &mut clip,
        );
        assert!(!core.text().contains("你"));

        // Render contract: text + gutter + caret present
        let list = render::render(&core, &mut fs, 400.0, 300.0, None);
        assert!(!list.text_runs.is_empty(), "body text runs must be present");
        assert!(list.gutter.is_some(), "gutter section must be present");
        assert!(list.caret.is_some(), "caret must be placed");
        assert!(!list.gutter.as_ref().unwrap().numbers.is_empty());
        // PLAN-601 T-10: background derives from the ACTIVE theme's
        // Background token (stella dark by default), not a hardcoded preset.
        assert_eq!(
            list.background.map(|(_, c)| c),
            Some(crate::ui::code_editor::theme::active_code_theme(true, "indigo").background)
        );
    }

    /// Plan 414 §4: the line-number gutter keeps at least two digit columns
    /// even for single-digit line counts.
    #[test]
    fn gutter_minimum_two_digits() {
        let mut fs = FontSystem::new();
        let core = CodeEditorCore::new("test-gutter-min", CodeEditorConfig::default(), &mut fs);
        core.set_text("one line only", &mut fs);
        let list = render::render(&core, &mut fs, 400.0, 200.0, None);
        let gutter = list.gutter.expect("gutter present");
        assert_eq!(gutter.digits, 2, "short files keep a 2-digit slot");
    }

    /// Plan 414 §5 Phase A: block-opener lines (trim ends with `{`) carry a
    /// fold chevron; brace-free text carries none.
    #[test]
    fn gutter_fold_chevrons_on_block_openers() {
        let mut fs = FontSystem::new();
        let config = CodeEditorConfig { lang: "auto".to_owned(), ..CodeEditorConfig::default() };
        let core = CodeEditorCore::new("test-gutter-folds", config, &mut fs);
        core.set_text(
            "// header
fn add(a int, b int) int {
    return a + b
}
",
            &mut fs,
        );
        let list = render::render(&core, &mut fs, 400.0, 200.0, None);
        let gutter = list.gutter.expect("gutter present");
        assert_eq!(gutter.folds.len(), 1, "one opener line (fn add ... brace)");
        // The chevron y matches the opener line's top (line 2 of 4).
        let expected_y = gutter
            .numbers
            .iter()
            .find(|n| n.number == 2)
            .map(|n| n.y)
            .expect("line 2 visible");
        assert!((gutter.folds[0].y - expected_y).abs() < 0.5);
        // Plan 428 P3: unfolded openers show the expanded state.
        assert!(!gutter.folds[0].folded, "fresh render: nothing folded yet");

        // Brace-free text has no fold markers.
        let core2 = CodeEditorCore::new("test-gutter-nofolds", CodeEditorConfig::default(), &mut fs);
        core2.set_text("plain
text
only
", &mut fs);
        let list2 = render::render(&core2, &mut fs, 400.0, 200.0, None);
        assert!(list2.gutter.expect("gutter").folds.is_empty());
    }

    /// External-dirty handshake: natives mark, the widget consumes once.
    #[test]
    fn core_external_dirty_roundtrip() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(test_font_system);
        let key = storage_key("test-ext-dirty");
        code_editor_dispose(&key);
        let core = code_editor(&key, &CodeEditorConfig::default());
        assert!(!core.take_external_dirty(), "fresh core is clean");

        // Text-mutating natives mark the flag.
        code_editor_undo(&key);
        assert!(core.take_external_dirty());
        assert!(!core.take_external_dirty(), "consume-once semantics");

        // Copy must NOT mark (clipboard-only op); cut/paste mark even when
        // the buffer ends up unchanged — the widget republishes
        // unconditionally, which is harmless (model reads current text).
        code_editor_clipboard_op(&key, ClipboardOp::Copy);
        assert!(!core.take_external_dirty(), "copy is not a text change");
        code_editor_clipboard_op(&key, ClipboardOp::Cut);
        assert!(core.take_external_dirty(), "cut marks");
        code_editor_dispose(&key);
    }

    /// Config diffs: wrap toggling flips the horizontal scrollbar, and vi
    /// mode on top of wrap keeps the cursor anchored (line 0). This test sat
    /// dead (no `#[test]`) after an editing accident — reactivated with the
    /// 413 search-highlight fix batch.
    #[test]
    fn core_config_diff_toggles_wrap_and_vi() {
        let mut fs = FontSystem::new();
        let core = CodeEditorCore::new(
            "test-core-config",
            CodeEditorConfig::default(),
            &mut fs,
        );
        core.set_text("aaaa bbbb cccc dddd eeee ffff gggg hhhh iiii jjjj", &mut fs);
        let list = render::render(&core, &mut fs, 120.0, 100.0, None);
        assert!(
            list.scrollbar_h.is_some(),
            "wide single line in a narrow viewport shows the h scrollbar"
        );

        let wrapped = CodeEditorConfig { wrap: true, ..CodeEditorConfig::default() };
        core.apply_config(&wrapped, &mut fs);
        let vi = CodeEditorConfig { vi: true, wrap: true, ..CodeEditorConfig::default() };
        core.apply_config(&vi, &mut fs);
        let (line, _col, _sel) = core.cursor_info();
        assert_eq!(line, 0);
    }

    #[test]
    fn core_search_highlights_and_finds() {
        let mut fs = FontSystem::new();
        let core = CodeEditorCore::new(
            "test-core-search",
            CodeEditorConfig::default(),
            &mut fs,
        );
        core.set_text("let alpha = 1;
let beta = alpha + 2;
", &mut fs);

        // No pattern → no matches.
        let list = render::render(&core, &mut fs, 400.0, 200.0, None);
        assert!(list.search_matches.is_empty());

        // Highlight matches on visible lines.
        assert!(core.set_search("alpha"));
        assert!(!core.set_search("alpha")); // diffed no-op
        let list = render::render(&core, &mut fs, 400.0, 200.0, None);
        assert_eq!(list.search_matches.len(), 2, "alpha appears twice");

        // Invalid regex clears highlighting without panicking.
        assert!(core.set_search("(unclosed"));
        let list = render::render(&core, &mut fs, 400.0, 200.0, None);
        assert!(list.search_matches.is_empty());

        // find_next jumps + selects (case-insensitive).
        assert!(core.set_search("BETA"));
        assert!(core.find_next(&mut fs));
        let (line, _col, sel) = core.cursor_info();
        assert_eq!(line, 1);
        assert_eq!(sel, 4); // "beta" is 4 bytes
        let no_more = core.set_search("zzz-not-there");
        assert!(no_more);
        assert!(!core.find_next(&mut fs));
    }

    #[test]
    fn registry_lru_caps_growth() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(test_font_system);
        let config = CodeEditorConfig::default();
        // Fill beyond the cap with distinct keys.
        for i in 0..40 {
            let _ = code_editor(&storage_key(&format!("lru-{i}")), &config);
        }
        assert!(
            code_editor_count() <= 32,
            "registry must stay within the LRU cap, got {}",
            code_editor_count()
        );
        // Recently used keys survive.
        assert!(code_editor_text(&storage_key("lru-39")).is_some());
        assert!(code_editor_text(&storage_key("lru-0")).is_none(), "oldest swept");
    }

    // ── global registry keying (Plan 413 §5.4) ──────────────────────────

    // Uses the crate-shared registry test lock (see REGISTRY_TEST_LOCK
    // at the core module level).

    /// PLAN-626 rev2 T-08: wheel semantics — winit's wheel-down y is
    /// negative and must scroll DOWN (cosmic-text scroll.vertical grows
    /// downwards); hammering past the bottom must clamp (shape_until_scroll
    /// normalizes scroll.line instead of accumulating an unbounded vertical,
    /// which used to walk the whole file per frame = "not responding" on
    /// large files); wheel at a boundary is a no-op without repaint.
    #[test]
    fn plan626_wheel_direction_clamps_at_bottom_and_noops_at_boundary() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(test_font_system);
        let key = storage_key("test-wheel-clamp");
        code_editor_dispose(&key);
        let config = CodeEditorConfig::default();
        let line_height = config.line_height();
        let core = code_editor(&key, &config);
        let line_count = 60usize;
        let text = (0..line_count)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        with_font_system(|fs| core.set_text(&text, fs));
        // Simulate the iced draw-path sizing (render normally does this).
        with_font_system(|fs| {
            let mut editor = core.editor_lock();
            editor.with_buffer_mut(|b| b.set_size(fs, Some(300.0), Some(200.0)));
        });
        let scroll_of = || {
            with_font_system(|fs| {
                let editor = core.editor_lock();
                editor.with_buffer(|b| b.scroll())
            })
        };

        // 1) Wheel DOWN (winit y negative) scrolls down.
        with_font_system(|fs| core.handle_wheel(fs, 0.0, -3.0, false));
        let after_down = scroll_of();
        assert!(
            after_down.line > 0 || after_down.vertical > 0.0,
            "wheel down must scroll down, got {after_down:?}"
        );

        // 2) Hammering far past the bottom clamps and stays stable.
        for _ in 0..60 {
            with_font_system(|fs| core.handle_wheel(fs, 0.0, -3.0, false));
        }
        let bottom = scroll_of();
        let viewport_lines = (200.0 / line_height).floor().max(1.0);
        assert!(
            bottom.line as f32 + viewport_lines >= line_count as f32 - 1.0,
            "bottom must be reachable: scroll {bottom:?} vs {line_count} lines"
        );
        let out = with_font_system(|fs| core.handle_wheel(fs, 0.0, -3.0, false));
        assert_eq!(scroll_of(), bottom, "clamped at bottom: no further movement");
        assert!(
            !out.request_redraw,
            "at-boundary wheel must not request a repaint"
        );

        // 3) Wheel UP past the top clamps back to line 0 / vertical 0.
        for _ in 0..80 {
            with_font_system(|fs| core.handle_wheel(fs, 0.0, 3.0, false));
        }
        let top = scroll_of();
        assert_eq!(top.line, 0, "top clamp");
        assert_eq!(top.vertical, 0.0, "top clamp");
        let out = with_font_system(|fs| core.handle_wheel(fs, 0.0, 3.0, false));
        assert!(!out.request_redraw, "at-top wheel must not request a repaint");
    }

    /// PLAN-629 T-01: hosted-scroller contract — content height reporting,
    /// absolute-offset sync (coarse line + normalize), bottom clamping,
    /// fold shrink, and the caret offset readback.
    #[test]
    fn plan629_content_height_external_scroll_and_caret() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(test_font_system);
        let key = storage_key("test-scroller-sync");
        code_editor_dispose(&key);
        let config = CodeEditorConfig::default();
        let line_height = config.line_height();
        let core = code_editor(&key, &config);
        let text = "fn f() {
    a
    b
    c
}
"
            .to_string()
            + &(5..60).map(|i| format!("fill {i}")).collect::<Vec<_>>().join("
").as_str();
        with_font_system(|fs| core.set_text(&text, fs));
        with_font_system(|fs| {
            let mut editor = core.editor_lock();
            editor.with_buffer_mut(|b| b.set_size(fs, Some(300.0), Some(200.0)));
        });

        // Height report: 60 lines, nothing folded.
        let h0 = core.content_height();
        assert!((h0 - 60.0 * line_height).abs() < 1.0, "content height, got {h0}");

        // Absolute-offset sync lands the viewport near the request.
        with_font_system(|fs| core.sync_external_scroll(fs, 300.0));
        let abs = with_font_system(|fs| {
            let editor = core.editor_lock();
            editor.with_buffer(|b| {
                let s = b.scroll();
                s.line as f32 * line_height + s.vertical
            })
        });
        assert!((abs - 300.0).abs() <= line_height, "absolute offset ≈ request, got {abs}");

        // Clamps past the bottom: content_height - viewport_h.
        with_font_system(|fs| core.sync_external_scroll(fs, 100_000.0));
        let abs_bottom = with_font_system(|fs| {
            let editor = core.editor_lock();
            editor.with_buffer(|b| {
                let s = b.scroll();
                s.line as f32 * line_height + s.vertical
            })
        });
        let want = (h0 - 200.0).max(0.0);
        assert!((abs_bottom - want).abs() <= 2.0 * line_height, "bottom clamp ≈ {want}, got {abs_bottom}");

        // Folding the fn block hides its 4-line body (lines 1-4 — the `…`
        // marker rides on the opener line), shrinking the height by 4 lines.
        core.fold_toggle(0);
        let h1 = core.content_height();
        assert!(
            (h1 - (h0 - 4.0 * line_height)).abs() < 1.0,
            "fold shrink: {h0} -> {h1}"
        );

        // Caret readback: computed directly, no render dependency — the
        // caret sits on line 0 (the visible fold opener), content y 0.
        let caret = core.caret_offset_y();
        assert!(
            matches!(caret, Some(y) if y.abs() < line_height),
            "caret offset at line 0 ≈ 0, got {caret:?}"
        );
        // Registry getter mirrors it.
        assert_eq!(
            code_editor_caret_offset_y(&key),
            core.caret_offset_y(),
            "registry getter matches core"
        );
    }

    /// PLAN-626 rev2 T-09: scrollbar drag — pressing the thumb then moving
    /// must land shaped, normalized scroll (vertical zeroed, target lines
    /// shaped) so the next frame's layout_runs never degenerates (the old
    /// path left first_visible_line at usize::MAX → NaN scrollbar geometry →
    /// frozen drag).
    #[test]
    fn plan626_scrollbar_drag_lands_shaped_normalized_scroll() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(test_font_system);
        let key = storage_key("test-wheel-drag");
        code_editor_dispose(&key);
        let core = code_editor(&key, &CodeEditorConfig::default());
        let line_count = 60usize;
        let text = (0..line_count)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("
");
        with_font_system(|fs| core.set_text(&text, fs));

        // A render pass populates layout_info (scrollbar rects, visible
        // lines) exactly like the iced draw path does.
        with_font_system(|fs| {
            crate::ui::code_editor::core::render::render(&core, fs, 300.0, 200.0, None);
        });
        let sb = core
            .layout_info
            .lock()
            .unwrap()
            .scrollbar_v
            .clone()
            .expect("v scrollbar after render");

        // Press on the thumb, then drag well below it.
        let grab_y = sb.y + sb.h / 2.0;
        with_font_system(|fs| {
            core.handle_mouse_press(fs, EditorButton::Left, sb.x + 1.0, grab_y);
        });
        let drag_y = sb.y + sb.h * 4.0;
        with_font_system(|fs| core.handle_mouse_move(fs, sb.x + 1.0, drag_y));

        let (line, vertical) = with_font_system(|fs| {
            let editor = core.editor_lock();
            editor.with_buffer(|b| {
                let s = b.scroll();
                (s.line, s.vertical)
            })
        });
        assert!(line > 0, "dragging down must advance scroll.line, got {line}");
        assert!(
            vertical.abs() < line_count as f32 * 100.0,
            "vertical must stay normalized, got {vertical}"
        );

        // The next frame must be sane: finite scrollbar geometry.
        let list = with_font_system(|fs| {
            crate::ui::code_editor::core::render::render(&core, fs, 300.0, 200.0, None)
        });
        let sb2 = list.scrollbar_v.expect("scrollbar present after drag");
        assert!(
            sb2.thumb.x.is_finite() && sb2.thumb.y.is_finite(),
            "scrollbar geometry must stay finite after drag: {:?}",
            sb2.thumb
        );
    }

    /// Test font-system callback: one process-wide FontSystem behind a
    /// RwLock, mirroring the iced adapter's install.
    fn test_font_system(with: &mut dyn FnMut(&mut FontSystem)) {
        static FS: OnceLock<RwLock<FontSystem>> = OnceLock::new();
        let fs = FS.get_or_init(|| RwLock::new(FontSystem::new()));
        let mut guard = fs.write().unwrap();
        with(&mut guard);
    }

    #[test]
    fn registry_keys_dispose_and_recreates() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(test_font_system);
        let key = storage_key("test-registry-key");
        code_editor_dispose(&key);

        let config = CodeEditorConfig {
            lang: "rust".to_owned(),
            ..CodeEditorConfig::default()
        };
        let core = code_editor(&key, &config);
        with_font_system(|fs| core.set_text("one", fs));
        assert_eq!(code_editor_text(&key).as_deref(), Some("one"));
        assert_eq!(code_editor_cursor(&key).map(|(l, _, _)| l), Some(0));

        // Same key returns the same instance (config diffed, text kept).
        let again = code_editor(&key, &config);
        assert_eq!(again.text(), "one");
        assert_eq!(code_editor_count() >= 1, true);

        // Dispose drops the registration; a new core starts fresh.
        code_editor_dispose(&key);
        assert_eq!(code_editor_text(&key), None);
        let fresh = code_editor(&key, &config);
        assert_eq!(fresh.text(), "");

        // Programmatic set-text by key (MCP automation path).
        assert!(code_editor_set_text(&key, "changed"));
        assert!(!code_editor_set_text(&key, "changed")); // no-op on equal
        assert_eq!(code_editor_text(&key).as_deref(), Some("changed"));
    }

    /// Plan 413 regression: a view rebuild re-syncs the DSL content value on
    /// every frame; re-pushing an UNCHANGED value must not revert the user's
    /// in-progress edits (the naive current-text diff did exactly that).
    #[test]
    fn registry_resync_does_not_clobber_user_edits() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(test_font_system);
        let key = storage_key("test-resync-keeps-edit");
        code_editor_dispose(&key);
        let config = CodeEditorConfig { lang: "rust".to_owned(), ..CodeEditorConfig::default() };
        let core = code_editor(&key, &config);
        // First external push seeds the editor (top-level: the registry
        // wrapper takes the font-system lock internally — must never be
        // called from inside a with_font_system closure, it would deadlock).
        assert!(code_editor_set_text(&key, "hello world"));
        // A keystroke diverges the live text from the external value.
        with_font_system(|fs| {
            core.set_focused(true);
            core.handle_input(
                fs,
                EditorInput::KeyPressed {
                    key: EditorKey::End,
                    text: None,
                    modifiers: EditorModifiers::none(),
                },
                &mut NullClipboard,
            );
            core.handle_input(
                fs,
                EditorInput::KeyPressed {
                    key: EditorKey::Char('!'),
                    text: Some("!".to_owned()),
                    modifiers: EditorModifiers::none(),
                },
                &mut NullClipboard,
            );
        });
        assert_eq!(core.text(), "hello world!");
        // View rebuild re-syncs the SAME external value — the user's edit
        // must survive.
        assert!(!code_editor_set_text(&key, "hello world"));
        assert_eq!(
            core.text(),
            "hello world!",
            "unchanged re-sync must not revert the user's edit"
        );
        // A genuinely changed external value still rewrites.
        assert!(code_editor_set_text(&key, "reset"));
        assert_eq!(core.text(), "reset");
    }

    // ── Plan 673 T-01: unified delta queue (read side) ─────────────────

    /// A successful `set_text` rewrite pushes exactly one full-replace
    /// delta: `start: 0`, `end` = byte length of the pre-rewrite text,
    /// `replacement` = the new text (§4.3).
    #[test]
    fn set_text_pushes_full_replace_delta() {
        let mut fs = FontSystem::new();
        let core =
            CodeEditorCore::new("test-delta-set-text", CodeEditorConfig::default(), &mut fs);
        // Fresh core: old text is empty → old_len 0.
        core.set_text("hello world", &mut fs);
        let (rev, deltas) = core.take_deltas();
        assert_eq!(rev, 1, "one edit bumps the watermark to 1");
        assert_eq!(
            deltas,
            vec![TextDelta { start: 0, end: 0, replacement: "hello world".to_string() }]
        );
        // Second rewrite: end = byte length of the PREVIOUS text (11 bytes).
        core.set_text("héllo", &mut fs);
        let (rev2, deltas2) = core.take_deltas();
        assert_eq!(rev2, 2);
        assert_eq!(
            deltas2,
            vec![TextDelta { start: 0, end: 11, replacement: "héllo".to_string() }]
        );
    }

    /// Core-level no-change `set_text` (same text twice) rewrites nothing
    /// and therefore pushes nothing — no noise deltas.
    #[test]
    fn set_text_no_change_pushes_nothing() {
        let mut fs = FontSystem::new();
        let core =
            CodeEditorCore::new("test-delta-no-change", CodeEditorConfig::default(), &mut fs);
        core.set_text("same", &mut fs);
        let (_, deltas) = core.take_deltas();
        assert_eq!(deltas.len(), 1);
        core.set_text("same", &mut fs); // early return — no rewrite
        let (_, deltas) = core.take_deltas();
        assert!(deltas.is_empty(), "no-change set_text must not push a delta");
    }

    /// Plan 673 §4.2 registry read面: `code_editor_delta` serializes
    /// `{"revision":N,"deltas":[...]}`, drains destructively (a second call
    /// at the same watermark returns `"deltas":[]`), and tracks old_len
    /// across successive edits.
    #[test]
    fn registry_code_editor_delta_destructive_read() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(test_font_system);
        let key = storage_key("test-delta-registry");
        code_editor_dispose(&key);
        let config = CodeEditorConfig { lang: "rust".to_owned(), ..CodeEditorConfig::default() };
        let _core = code_editor(&key, &config);

        // Unknown key → None (mirrors code_editor_text).
        assert!(code_editor_delta("no-such-editor-673").is_none());

        assert!(code_editor_set_text(&key, "hello"));
        let json = code_editor_delta(&key).expect("delta envelope");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["revision"], serde_json::json!(1));
        assert_eq!(
            v["deltas"],
            serde_json::json!([{"start": 0, "end": 0, "replacement": "hello"}])
        );

        // Destructive read: second call drains nothing new, same watermark,
        // shape恒定 (no special-casing to an empty string).
        let json2 = code_editor_delta(&key).expect("delta envelope");
        assert_eq!(json2, r#"{"revision":1,"deltas":[]}"#);

        // The next edit queues the next delta at the next watermark
        // (old_len = 5 bytes of "hello").
        assert!(code_editor_set_text(&key, "hello!"));
        let json3 = code_editor_delta(&key).expect("delta envelope");
        let v3: serde_json::Value = serde_json::from_str(&json3).unwrap();
        assert_eq!(v3["revision"], serde_json::json!(2));
        assert_eq!(
            v3["deltas"],
            serde_json::json!([{"start": 0, "end": 5, "replacement": "hello!"}])
        );
    }

    // ── Plan 673 T-02: write side + three-source same stream ───────────

    /// `derive_interval` (pure): the single minimal contiguous interval,
    /// scanned on char boundaries only — mid-string insertion, multi-byte
    /// CJK deletion, 4-byte emoji replacement, append, whole-doc replace,
    /// and no-change → None.
    #[test]
    fn push_delta_from_texts_interval_derivation() {
        let d = |old: &str, new: &str| CodeEditorCore::derive_interval(old, new);

        // Mid-string insertion (suffix absorbs the shared " world").
        assert_eq!(
            d("hello world", "hello brave world"),
            Some(TextDelta { start: 6, end: 6, replacement: "brave ".to_string() })
        );
        // Deletion of a CJK char (3-byte boundaries).
        assert_eq!(
            d("你好world", "你world"),
            Some(TextDelta { start: 3, end: 6, replacement: "".to_string() })
        );
        // Replacement touching a 4-byte emoji.
        assert_eq!(
            d("a😀b", "a🎉b"),
            Some(TextDelta { start: 1, end: 5, replacement: "🎉".to_string() })
        );
        // Append at the end.
        assert_eq!(
            d("abc", "abc!"),
            Some(TextDelta { start: 3, end: 3, replacement: "!".to_string() })
        );
        // Whole-document replace.
        assert_eq!(
            d("old", "new"),
            Some(TextDelta { start: 0, end: 3, replacement: "new".to_string() })
        );
        // No change → None.
        assert_eq!(d("same", "same"), None);
    }

    /// T-05: the commit path applies the derived interval to the ROPE
    /// (document source of truth) and queues the same interval as the delta —
    /// rope, buffer view, and delta stream cannot drift by construction.
    #[test]
    fn commit_derived_edit_syncs_rope_and_delta() {
        let mut fs = FontSystem::new();
        let core =
            CodeEditorCore::new("test-delta-commit", CodeEditorConfig::default(), &mut fs);
        core.set_text("hello world", &mut fs);
        core.take_deltas();
        // Simulate a typed edit: pre-edit rope text + post-edit buffer text.
        core.push_delta_from_texts("hello world", "hello brave world");
        assert_eq!(core.text(), "hello brave world", "rope must receive the derived interval");
        assert_eq!(
            core.take_deltas().1,
            vec![TextDelta { start: 6, end: 6, replacement: "brave ".to_string() }]
        );
        // No-change commit: rope untouched, nothing queued.
        core.push_delta_from_texts(core.text().as_str(), "hello brave world");
        assert_eq!(core.text(), "hello brave world");
        assert!(core.take_deltas().1.is_empty());
    }

    /// `code_editor_edit` three forms produce the EXACT caller-interval delta
    /// (not a prefix/suffix derivation); invalid input is refused and queues
    /// nothing (报错不静默).
    #[test]
    fn registry_code_editor_edit_forms_and_validation() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap();
        set_font_system_call(test_font_system);
        let key = storage_key("test-edit-registry");
        code_editor_dispose(&key);
        let config = CodeEditorConfig { lang: "rust".to_owned(), ..CodeEditorConfig::default() };
        let _core = code_editor(&key, &config);
        assert!(code_editor_set_text(&key, "hello world"));
        // Drain the seeding full-replace delta.
        let (_, initial) = code_editor_with(&key, |c| c.take_deltas()).unwrap();
        assert_eq!(initial.len(), 1);

        // Insert form (start == end).
        assert!(code_editor_edit(&key, 5, 5, ","));
        let (_, d) = code_editor_with(&key, |c| c.take_deltas()).unwrap();
        assert_eq!(d, vec![TextDelta { start: 5, end: 5, replacement: ",".to_string() }]);
        assert_eq!(code_editor_text(&key).as_deref(), Some("hello, world"));

        // Replace form.
        assert!(code_editor_edit(&key, 0, 5, "goodbye"));
        let (_, d) = code_editor_with(&key, |c| c.take_deltas()).unwrap();
        assert_eq!(d, vec![TextDelta { start: 0, end: 5, replacement: "goodbye".to_string() }]);
        assert_eq!(code_editor_text(&key).as_deref(), Some("goodbye, world"));

        // Delete form (empty replacement): drop ", world" (bytes 7..14).
        assert!(code_editor_edit(&key, 7, 14, ""));
        let (_, d) = code_editor_with(&key, |c| c.take_deltas()).unwrap();
        assert_eq!(d, vec![TextDelta { start: 7, end: 14, replacement: "".to_string() }]);
        assert_eq!(code_editor_text(&key).as_deref(), Some("goodbye"));

        // Invalid: start > end, out of bounds, non-char boundary on CJK —
        // all refused, queue untouched.
        assert!(!code_editor_edit(&key, 4, 2, "x"));
        assert!(!code_editor_edit(&key, 0, 99, "x"));
        assert!(code_editor_set_text(&key, "你好"));
        let _ = code_editor_with(&key, |c| c.take_deltas()); // drain the reseed
        assert!(!code_editor_edit(&key, 1, 3, "x"));
        assert!(!code_editor_edit(&key, 3, 1, "x"));
        let (_, d) = code_editor_with(&key, |c| c.take_deltas()).unwrap();
        assert!(d.is_empty(), "invalid edits must queue nothing");
        // Unknown key → false.
        assert!(!code_editor_edit("no-such-editor-673", 0, 0, "x"));
    }

    /// Plan 673 §4.3 三来源同流: an agent structured write, a human typed
    /// keystroke, and an undo land in ONE ordered delta queue — the exact
    /// interval sequence a consumer of `code_editor_delta` would apply.
    #[test]
    fn three_sources_share_one_delta_stream() {
        let mut fs = FontSystem::new();
        let core =
            CodeEditorCore::new("test-delta-3src", CodeEditorConfig::default(), &mut fs);
        core.set_text("fn main() {", &mut fs);
        core.take_deltas(); // drain the seeding full-replace

        // Source 1: agent structured write (insert at the top).
        assert!(core.edit(0, 0, "// ", &mut fs));
        // Source 2: human typing — End, then '!'.
        core.set_focused(true);
        let mut clip = NullClipboard;
        core.handle_input(
            &mut fs,
            EditorInput::KeyPressed {
                key: EditorKey::End,
                text: None,
                modifiers: EditorModifiers::none(),
            },
            &mut clip,
        );
        core.handle_input(
            &mut fs,
            EditorInput::KeyPressed {
                key: EditorKey::Char('!'),
                text: Some("!".to_owned()),
                modifiers: EditorModifiers::none(),
            },
            &mut clip,
        );
        // Source 3: undo (native menu path) removes the typed '!'.
        core.do_undo();

        let (_, deltas) = core.take_deltas();
        assert_eq!(
            deltas,
            vec![
                TextDelta { start: 0, end: 0, replacement: "// ".to_string() },
                TextDelta { start: 14, end: 14, replacement: "!".to_string() },
                TextDelta { start: 14, end: 15, replacement: "".to_string() },
            ],
            "agent write, typed keystroke and undo must share one ordered stream"
        );
        assert_eq!(core.text(), "// fn main() {");
    }

    /// Plan 413 §6.4 performance criterion: a ~1MB source shapes and renders
    /// without pathological stalls. Ignored by default (slow init); run with
    /// `cargo test --lib code_editor -- --ignored`.
    #[test]
    #[ignore = "perf smoke: ~1MB shaping"]
    fn large_file_renders() {
        set_font_system_call(test_font_system);
        let line = "    let value = compute_something(x_i + y_i * 3) / total; // keep going\n";
        let n_lines = 1_000_000 / line.len();
        let big: String = line.repeat(n_lines);
        let config = CodeEditorConfig { lang: "rust".to_owned(), ..CodeEditorConfig::default() };
        let core = code_editor(&storage_key("test-large"), &config);
        let t0 = std::time::Instant::now();
        code_editor_set_text(&storage_key("test-large"), &big);
        let set = t0.elapsed();
        // Optional: give the background syntax warm-up time to finish, to
        // measure the warmed cold-render (AUTO_CE_WARMUP_WAIT=1).
        if std::env::var("AUTO_CE_WARMUP_WAIT").is_ok() {
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        let t1 = std::time::Instant::now();
        let list = with_font_system(|fs| render::render(core, fs, 800.0, 600.0, None));
        let rendered = t1.elapsed();
        let t2 = std::time::Instant::now();
        with_font_system(|fs| render::render(core, fs, 800.0, 600.0, None));
        let rendered2 = t2.elapsed();
        assert!(!list.text_runs.is_empty());
        assert!(list.gutter.is_some());
        // shape_as_needed only shapes the visible window; both phases stay
        // well under a frame budget on a dev machine (generous CI margin).
        assert!(set.as_secs() < 10, "set_text took {set:?}");
        assert!(rendered.as_secs() < 10, "render took {rendered:?}");
        eprintln!("large file: set={set:?} render1={rendered:?} render2={rendered2:?}");
    }

    /// Plan 673 T-05 S3 — 100MB smoke receipt (AC-05 material for plan §9).
    /// Ignored by default (CI unaffected, no heavy-tier registration); run
    /// manually and capture the printed receipts:
    ///   cargo test -p auto-lang --lib --features ui-iced smoke_100mb -- --ignored --nocapture
    ///
    /// Generates ~100MB of varied-length code-ish lines in the system temp
    /// dir (deleted after). Phase receipts:
    ///   1. open        — rope build + full initial buffer materialization
    ///   2. summaries   — O(1) root accessors under repetition
    ///   3. locate      — line_start_byte + byte_to_point at 100 depths
    ///   4. typed edit  — ONE keystroke via handle_input (the wrapper's
    ///                    before/after snapshots are the honest current-state
    ///                    per-keystroke cost — windowing/local-diff deferred)
    ///   5. agent edit  — code_editor_edit path (full O(n) rewrite receipt)
    ///   6. save        — text() readout through the rope
    ///   7. concurrency — snapshot to_string on a worker thread while the
    ///                    main rope takes edits (snapshot isolation receipt)
    ///
    /// No cross-platform RSS in-repo: peak is approximated by holding the
    /// corpus (100MB) + rope + buffer + one readout copy, and reported as
    /// the corpus size + phase notes instead.
    #[test]
    #[ignore = "100MB smoke: manual receipt run only (Plan 673 T-05 S3)"]
    fn smoke_100mb_rope_receipts() {
        use std::time::Instant;

        // ── corpus generation (~100MB, varied line lengths) ──
        let t_gen = Instant::now();
        let target_bytes = 100 * 1024 * 1024usize;
        let templates = [
            "    let value_{i} = compute_something(x_{i} + y_{i} * 3) / total_{i};",
            "    if value_{i} > threshold {{ return fallback({i}); }}",
            "    // comment line padding to vary lengths a bit more {i}",
            "fn helper_{i}(a int, b int) int {{",
            "    return a + b - {i}",
            "}}",
        ];
        let mut text = String::with_capacity(target_bytes + 4096);
        let mut i = 0usize;
        while text.len() < target_bytes {
            let t = templates[i % templates.len()];
            let mut line = t.replace("{i}", &i.to_string());
            // Vary length deterministically within a small band.
            let extra = i % 17;
            line.push_str(&" ".repeat(extra));
            line.push('\n');
            text.push_str(&line);
            i += 1;
        }
        let path = std::env::temp_dir().join("auto_lang_673_smoke_100mb.txt");
        std::fs::write(&path, &text).unwrap();
        println!(
            "[receipt] corpus: {} bytes, {} lines (gen {:?}, {:?} avg/line)",
            text.len(),
            text.bytes().filter(|&b| b == b'\n').count() + 1,
            t_gen.elapsed(),
            t_gen.elapsed() / i.max(1) as u32,
        );

        let mut fs = FontSystem::new();
        let config = CodeEditorConfig { lang: "rust".to_owned(), ..CodeEditorConfig::default() };
        let core = CodeEditorCore::new("smoke-100mb", config, &mut fs);

        // ── Phase 1: open (rope build + full buffer materialization) ──
        let t = Instant::now();
        core.set_text(&text, &mut fs);
        let open = t.elapsed();
        let lines = core.doc.lock().unwrap().line_count();
        println!("[receipt] phase 1 open: {open:?} (rope build + materialize {lines} lines)");

        // ── Phase 2: O(1) summaries ──
        let t = Instant::now();
        for _ in 0..100_000 {
            std::hint::black_box(core.doc.lock().unwrap().line_count());
            std::hint::black_box(core.doc.lock().unwrap().len_bytes());
        }
        println!("[receipt] phase 2 summaries: {:?} for 100k×(line_count+len_bytes)", t.elapsed());

        // ── Phase 3: locate at 100 depths (O(log n) each) ──
        let t = Instant::now();
        let mut check = 0usize;
        {
            let doc = core.doc.lock().unwrap();
            for k in 0..100 {
                let line = (lines - 2) * k / 100;
                let b = doc.line_start_byte(line);
                assert_eq!(doc.byte_to_point(b), (line, 0));
                check += b;
            }
        }
        println!("[receipt] phase 3 locate: {:?} for 100×(line_start_byte+byte_to_point) (checksum {check})", t.elapsed());

        // ── Phase 4: one typed keystroke at EOF (wrapper snapshot cost) ──
        core.set_focused(true);
        let t = Instant::now();
        core.handle_input(
            &mut fs,
            EditorInput::KeyPressed {
                key: EditorKey::Char('!'),
                text: Some("!".to_owned()),
                modifiers: EditorModifiers::none(),
            },
            &mut NullClipboard,
        );
        println!("[receipt] phase 4 typed edit (1 keystroke, incl. O(n) snapshots): {:?}", t.elapsed());

        // ── Phase 5: agent edit (code_editor_edit = full rewrite path) ──
        let mid = text.len() / 2;
        let t = Instant::now();
        assert!(core.edit(mid, mid, "// agent\n", &mut fs));
        let agent = t.elapsed();
        assert!(core.edit(mid, mid + 9, "", &mut fs));
        println!("[receipt] phase 5 agent edit: {agent:?} (edit + inverse, full O(n) rewrite each)");

        // ── Phase 6: save readout through the rope ──
        let expected_len = core.text().len(); // corpus + phase-4 '!' (phase 5 cancels out)
        let t = Instant::now();
        let saved = core.text();
        let save = t.elapsed();
        assert_eq!(saved.len(), expected_len, "save readout must match the live document");
        println!("[receipt] phase 6 save readout: {save:?} for {} bytes", saved.len());

        // ── Phase 7: snapshot isolation under concurrent edits ──
        let snap = core.doc.lock().unwrap().snapshot();
        let t = Instant::now();
        let worker = std::thread::spawn(move || {
            let tw = Instant::now();
            let s = snap.to_string();
            (s.len(), tw.elapsed())
        });
        {
            let mut doc = core.doc.lock().unwrap();
            for k in 0..100 {
                let at = doc.line_start_byte(k * 1000);
                doc.insert_bytes(at, "// x\n");
                doc.delete_bytes(at, at + 5);
            }
        }
        let (snap_len, snap_time) = worker.join().unwrap();
        assert_eq!(snap_len, expected_len, "snapshot sees the frozen pre-edit document");
        println!(
            "[receipt] phase 7 concurrency: worker to_string({} bytes) {:?} WHILE main rope applied 100 edit pairs ({:?} total)",
            snap_len, snap_time, t.elapsed()
        );

        std::fs::remove_file(&path).ok();
        println!("[receipt] cleanup: corpus file deleted; corpus was {:.1} MB on disk", target_bytes as f64 / 1024.0 / 1024.0);
    }
    // ── Plan 428: folding render + interaction integration ──────────────

    const FOLD_SRC: &str = "// header
fn add(a int, b int) int {
    let s = a + b
    return s
}
fn sub(a int, b int) int {
    return a - b
}
// tail
";

    /// P1+P2: toggle → render skips the body, projects y, marks the opener.
    #[test]
    fn fold_toggle_projects_render() {
        set_font_system_call(test_font_system);
        let mut fs = FontSystem::new();
        let core = CodeEditorCore::new(
            "test-fold-render",
            CodeEditorConfig::default(),
            &mut fs,
        );
        core.set_text(FOLD_SRC, &mut fs);

        // Fold `fn add` (opener line index 1).
        assert!(core.fold_toggle(1), "line 2 opens a region");
        assert!(core.fold_is_folded(1));
        assert_eq!(core.fold_hidden_count(), 3, "body = lines 3-5 hidden");

        let list = render::render(&core, &mut fs, 400.0, 200.0, None);
        assert_eq!(list.fold_hidden, 3);
        let gutter = list.gutter.expect("gutter");
        // The opener's chevron is folded; `fn sub`'s is not.
        assert!(gutter.folds.iter().any(|f| f.folded), "folded chevron");
        assert!(gutter.folds.iter().any(|f| !f.folded), "expanded chevron");
        // Hidden line numbers are absent from the gutter.
        for n in &gutter.numbers {
            assert!(
                !(3..=5).contains(&n.number),
                "hidden line {} leaked into gutter",
                n.number
            );
        }
        // Hidden body text is absent from the text runs; the fold marker
        // rides after the opener line.
        let joined: String = list.text_runs.iter().map(|r| r.text.as_str()).collect();
        assert!(!joined.contains("let s = a + b"), "body text hidden");
        assert!(!joined.contains("return s"), "body text hidden");
        assert!(joined.contains('⋯'), "fold marker present");
        assert!(joined.contains("fn sub"), "later blocks still visible");

        // Unfold → everything back.
        assert!(!core.fold_toggle(1));
        assert_eq!(core.fold_hidden_count(), 0);
        let list2 = render::render(&core, &mut fs, 400.0, 200.0, None);
        let joined2: String = list2.text_runs.iter().map(|r| r.text.as_str()).collect();
        assert!(joined2.contains("let s = a + b"), "body text restored");
        assert!(!joined2.contains('⋯'), "marker gone");
    }

    /// P3: clicking a chevron band in the fold column toggles the fold.
    #[test]
    fn fold_column_click_toggles() {
        set_font_system_call(test_font_system);
        let mut fs = FontSystem::new();
        let core = CodeEditorCore::new(
            "test-fold-click",
            CodeEditorConfig::default(),
            &mut fs,
        );
        core.set_text(FOLD_SRC, &mut fs);
        // First render records the fold column + bands.
        let _ = render::render(&core, &mut fs, 400.0, 200.0, None);

        // Click the opener's chevron: x inside the fold column, y on the
        // opener's projected band (read from the gutter of a fresh list).
        let list = render::render(&core, &mut fs, 400.0, 200.0, None);
        let gutter = list.gutter.expect("gutter");
        let opener_y = gutter
            .numbers
            .iter()
            .find(|n| n.number == 2)
            .map(|n| n.y)
            .expect("opener visible");
        let fc_x = gutter.bounds.w - 9.5; // middle of the 19px fold column
        let mut clip = NullClipboard;
        let out = core.handle_input(
            &mut fs,
            EditorInput::MousePressed { button: EditorButton::Left, x: fc_x, y: opener_y + 1.0 },
            &mut clip,
        );
        assert!(out.captured, "fold click captured");
        assert!(core.fold_is_folded(1), "fold engaged by click");
        assert_eq!(core.fold_hidden_count(), 3);

        // Click again (projected band unchanged — opener stays put).
        let _ = render::render(&core, &mut fs, 400.0, 200.0, None);
        let _ = core.handle_input(
            &mut fs,
            EditorInput::MousePressed { button: EditorButton::Left, x: fc_x, y: opener_y + 1.0 },
            &mut clip,
        );
        assert!(!core.fold_is_folded(1), "second click unfolds");
        assert_eq!(core.fold_hidden_count(), 0);
    }

    /// P4: caret motion into a folded body auto-expands.
    #[test]
    fn cursor_into_fold_auto_expands() {
        set_font_system_call(test_font_system);
        let mut fs = FontSystem::new();
        let core = CodeEditorCore::new(
            "test-fold-autoexpand",
            CodeEditorConfig::default(),
            &mut fs,
        );
        core.set_text(FOLD_SRC, &mut fs);
        assert!(core.fold_toggle(1));

        // Park the caret ON the hidden line 3 (0-based 2) — keyboard-only
        // state — then press Down: auto-expand must reveal it.
        {
            let mut editor = core.editor_lock();
            editor.set_cursor(Cursor::new(2, 0));
        }
        let mut clip = NullClipboard;
        let none_mods = EditorModifiers::none();
        core.handle_input(&mut fs, EditorInput::FocusGained, &mut clip);
        core.handle_input(
            &mut fs,
            EditorInput::KeyPressed { key: EditorKey::Down, text: None, modifiers: none_mods },
            &mut clip,
        );
        assert_eq!(
            core.fold_hidden_count(),
            0,
            "caret inside a folded body auto-expands it"
        );
    }

    #[test]
    fn core_value_diff_preserves_cursor() {
        let mut fs = FontSystem::new();
        let core = CodeEditorCore::new("test-core-diff", CodeEditorConfig::default(), &mut fs);
        core.set_text("line one
line two", &mut fs);
        // Move cursor to line 2
        core.set_focused(true);
        let mut clip = NullClipboard;
        core.handle_input(&mut fs, EditorInput::KeyPressed {
            key: EditorKey::Down,
            text: None,
            modifiers: EditorModifiers::none(),
        }, &mut clip);
        let (line, _, _) = core.cursor_info();
        assert_eq!(line, 1);
        // Same-value set_text is a no-op (cursor must not jump)
        core.set_text("line one
line two", &mut fs);
        let (line, col, _) = core.cursor_info();
        assert_eq!((line, col), (1, 0));
    }

    /// Plan 413 regression: clicking into a line containing multi-byte UTF-8
    /// (CJK) must never leave the cursor on a non-char-boundary byte offset —
    /// the next typed character would panic cosmic-text's `insert_at`
    /// (`String::split_off`). Sweeps a grid of click x-positions across the
    /// first (CJK) line, typing after each click.
    #[test]
    fn cjk_click_keeps_cursor_on_char_boundary() {
        let mut fs = FontSystem::new();
        let config = CodeEditorConfig { lang: "auto".to_owned(), ..CodeEditorConfig::default() };
        let core = CodeEditorCore::new("test-cjk-click", config, &mut fs);
        core.set_text(
            "// 预填文本显示测试 — 你好世界\nfn add(a int, b int) int {\n    return a + b\n}\n",
            &mut fs,
        );
        core.set_focused(true);
        // Establish layout (records the text rect for hit testing).
        render::render(&core, &mut fs, 600.0, 400.0, None);

        let mut clip = NullClipboard;
        let line_h = core.config().line_height();
        let mut clicked_line: Option<usize> = None;
        for x in (0..560).step_by(8) {
            for y in [line_h / 2.0, line_h * 1.5, line_h * 2.5] {
                core.handle_input(
                    &mut fs,
                    EditorInput::MousePressed {
                        button: EditorButton::Left,
                        x: x as f32,
                        y,
                    },
                    &mut clip,
                );
                let (line, col, _) = core.cursor_info();
                clicked_line = Some(line);
                // Typing must not panic regardless of where the click landed.
                core.handle_input(
                    &mut fs,
                    EditorInput::KeyPressed {
                        key: EditorKey::Char('x'),
                        text: Some("x".to_owned()),
                        modifiers: EditorModifiers::none(),
                    },
                    &mut clip,
                );
            }
        }
        assert!(clicked_line.is_some());
        assert!(core.text().contains('x'));
    }

    /// Plan 428 实机验收发现(413 期回归):编辑器强制捕获未处理的
    /// Ctrl+字母,而应用级快捷键层(action-config 回退层,Plan 418 P2-4)
    /// 只收未捕获事件——Ctrl+J/Ctrl+D/Ctrl+S 在编辑器有焦点时全部失效。
    /// 未识别组合必须放行;编辑器自己处理的组合保持捕获。
    #[test]
    fn unhandled_ctrl_letter_bubbles_for_app_shortcuts() {
        let mut fs = FontSystem::new();
        let core = CodeEditorCore::new("test-ctrl-bubble", CodeEditorConfig::default(), &mut fs);
        core.set_text("some text\nmore", &mut fs);
        core.set_focused(true);
        let mut clip = NullClipboard;
        let ctrl = EditorModifiers { control: true, ..EditorModifiers::none() };
        let press = |key: EditorKey| EditorInput::KeyPressed {
            key,
            text: None,
            modifiers: ctrl,
        };
        let out_j = core.handle_input(&mut fs, press(EditorKey::Char('j')), &mut clip);
        assert!(!out_j.captured, "Ctrl+J is the app's console toggle — must bubble");
        let out_s = core.handle_input(&mut fs, press(EditorKey::Char('s')), &mut clip);
        assert!(!out_s.captured, "Ctrl+S is the app's save — must bubble");
        // Handled combos stay captured (editor bindings keep working).
        let out_a = core.handle_input(&mut fs, press(EditorKey::Char('a')), &mut clip);
        assert!(out_a.captured, "Ctrl+A select-all is an editor binding");
    }

    /// Plan 413 regression: an external value diff that rewrites the buffer
    /// (view rebuild re-syncs `content:` to the DSL value) must not leave the
    /// cursor on a mid-char byte offset when the old text had multi-byte
    /// chars the new one lacks. Sequence: click into CJK text → type → the
    /// framework rewrites the buffer back to the original value → type again.
    #[test]
    fn set_text_resync_after_edit_keeps_cursor_on_boundary() {
        let mut fs = FontSystem::new();
        let config = CodeEditorConfig { lang: "auto".to_owned(), ..CodeEditorConfig::default() };
        let core = CodeEditorCore::new("test-cjk-resync", config, &mut fs);
        let original = "// 预填文本显示测试 — 你好世界\nfn add(a int, b int) int {\n    return a + b\n}\n";
        core.set_text(original, &mut fs);
        core.set_focused(true);
        render::render(&core, &mut fs, 600.0, 400.0, None);

        let mut clip = NullClipboard;
        // Click after "// " (byte 3: start of the first Han char).
        core.handle_input(
            &mut fs,
            EditorInput::MousePressed {
                button: EditorButton::Left,
                x: 60.0,
                y: core.config().line_height() / 2.0,
            },
            &mut clip,
        );
        // Type one character — the editor now differs from the DSL value.
        core.handle_input(
            &mut fs,
            EditorInput::KeyPressed {
                key: EditorKey::Char('x'),
                text: Some("x".to_owned()),
                modifiers: EditorModifiers::none(),
            },
            &mut clip,
        );
        assert!(core.text() != original, "edit must diverge from the DSL value");
        // The framework re-syncs the value: buffer rewritten + cursor clamped.
        core.set_text(original, &mut fs);
        // The next keystroke must not panic (cursor on a char boundary).
        core.handle_input(
            &mut fs,
            EditorInput::KeyPressed {
                key: EditorKey::Char('y'),
                text: Some("y".to_owned()),
                modifiers: EditorModifiers::none(),
            },
            &mut clip,
        );
    }
}
