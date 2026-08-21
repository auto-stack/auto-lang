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

pub mod highlight;
pub mod render;

use std::collections::HashMap;
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
    /// LRU stamp for registry sweeping (§5.4 auto-dispose).
    last_used: AtomicU64,
    /// Cached gutter width for `digits` columns.
    gutter_width_cache: Mutex<(usize, f32)>,
}

#[derive(Debug, Clone, Copy, Default)]
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

/// Family for editor body text. `Family::Monospace` goes through
/// cosmic-text's monospace fallback path, which on Windows picks a font
/// whose CJK glyphs come out as tofu boxes; a named real font uses the
/// ordinary fallback chain (→ Microsoft YaHei for Han) that renders CJK
/// correctly. Consolas ships with every Windows; elsewhere keep the generic
/// monospace family.
fn mono_family() -> Family<'static> {
    if cfg!(windows) {
        Family::Name("Consolas")
    } else {
        Family::Monospace
    }
}

/// Weak handle to the editor's CURRENT buffer, for the draw list (`fill_raw`
/// handoff). Must be acquired per frame and dropped at frame end: a
/// longer-lived weak forces `Arc::make_mut` (cosmic-text's
/// `Edit::with_buffer_mut` on the `BufferRef::Arc` variant) to clone the
/// entire buffer on the next mutation, orphaning the previous handle — the
/// body text then stops rendering (Plan 413 fix).
pub(crate) fn editor_buffer_weak(editor: &ViEditor) -> Option<std::sync::Weak<Buffer>> {
    match editor.buffer_ref() {
        BufferRef::Arc(arc) => Some(Arc::downgrade(arc)),
        _ => None,
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
            layout_info: Mutex::new(LayoutInfo::default()),
            applied_theme: Mutex::new(None),
            search: Mutex::new(SearchState::default()),
            revision: AtomicU64::new(0),
            last_used: AtomicU64::new(0),
            gutter_width_cache: Mutex::new((0, 0.0)),
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
    /// line endings).
    pub fn text(&self) -> String {
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
        let attrs = Attrs::new().family(mono_family());
        // Give the buffer a viewport before rewriting: with no size set,
        // Buffer::set_text's internal shape_until_scroll treats the scroll
        // window as infinite and shapes/highlights the WHOLE document (26s
        // on a 1MB file). Any finite size keeps it lazy; render() applies
        // the real viewport each frame.
        let info = *self.layout_info.lock().unwrap();
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
        // stale selection past the end would panic the engine later.
        editor.set_selection(Selection::None);
        let mut cursor = editor.cursor();
        editor.with_buffer(|b| {
            cursor.line = cursor.line.min(b.lines.len().saturating_sub(1));
            let len = b.lines.get(cursor.line).map(|l| l.text().len()).unwrap_or(0);
            cursor.index = cursor.index.min(len);
        });
        editor.set_cursor(cursor);
        self.revision.fetch_add(1, Ordering::Relaxed);
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
        let info = *self.layout_info.lock().unwrap();
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
        let name = highlight::theme_name(dark, accent);
        let mut applied = self.applied_theme.lock().unwrap();
        if applied.as_deref() == Some(name.as_str()) {
            return;
        }
        let theme = if dark {
            CodeEditorTheme::dark(accent)
        } else {
            CodeEditorTheme::light(accent)
        };
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
    pub fn find_next(&self, font_system: &mut FontSystem) -> bool {
        let regex = match self.search.lock().unwrap().regex.clone() {
            Some(r) => r,
            None => return false,
        };
        let mut editor = self.editor_lock();
        let start = editor.cursor();
        let line_count = editor.with_buffer(|b| b.lines.len());

        // (line, byte start, byte end) of the next match at-or-after the
        // caret, wrapping around the document.
        let mut found: Option<(usize, usize, usize)> = None;
        for offset in 0..=line_count {
            let line_i = (start.line + offset) % line_count.max(1);
            let text = editor
                .with_buffer(|b| b.lines.get(line_i).map(|l| l.text().to_owned()))
                .unwrap_or_default();
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
                self.handle_key(font_system, key, text, modifiers, clipboard)
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
                let mut editor = self.editor_lock();
                editor.insert_string(&content, None);
                drop(editor);
                self.bump_after_edit();
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

    fn handle_key(
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
                    _ => CoreOutput::default(),
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
        let info = *self.layout_info.lock().unwrap();

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
                self.drag_scrollbar_v(y);
                return out.captured();
            }
        }
        if let Some(sb) = info.scrollbar_h {
            if sb.contains(super::draw::Pt::new(x, y)) {
                let grab_offset = (x - sb.x).clamp(0.0, sb.w);
                *self.drag.lock().unwrap() = Drag::ScrollbarH { grab_offset };
                self.drag_scrollbar_h(x);
                return out.captured();
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
        let bx = (x - info.text.x + scroll_x).max(0.0) as i32;
        let by = (y - info.text.y + scroll_y).max(0.0) as i32;

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
        let info = *self.layout_info.lock().unwrap();
        match *self.drag.lock().unwrap() {
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
                let mut by = (y - info.text.y + scroll_y).max(0.0) as i32;
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
                self.drag_scrollbar_v(y);
                CoreOutput { request_redraw: true, ..CoreOutput::default() }.captured()
            }
            Drag::ScrollbarH { .. } => {
                self.drag_scrollbar_h(x);
                CoreOutput { request_redraw: true, ..CoreOutput::default() }.captured()
            }
        }
    }

    /// Map a vertical scrollbar drag to a buffer scroll line.
    fn drag_scrollbar_v(&self, y: f32) {
        let info = *self.layout_info.lock().unwrap();
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
            b.set_scroll(scroll);
        });
    }

    /// Map a horizontal scrollbar drag to a horizontal pixel scroll.
    fn drag_scrollbar_h(&self, x: f32) {
        let info = *self.layout_info.lock().unwrap();
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
        if (shift && dx == 0.0) || dx != 0.0 {
            // Shift+wheel (or horizontal wheel) → horizontal scroll.
            let amount = if dx != 0.0 { dx } else { dy };
            editor.with_buffer_mut(|b| {
                let mut scroll = b.scroll();
                scroll.horizontal = (scroll.horizontal + amount * config.font_size * 0.8).max(0.0);
                b.set_scroll(scroll);
            });
        } else {
            editor.action(font_system, Action::Scroll { pixels: dy * config.line_height() });
        }
        CoreOutput { request_redraw: true, ..CoreOutput::default() }
    }

    fn bump_after_edit(&self) {
        self.revision.fetch_add(1, Ordering::Relaxed);
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

/// Read the cursor position of an editor: (line 0-based, char column,
/// selection length in bytes).
pub fn code_editor_cursor(key: &str) -> Option<(usize, usize, usize)> {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    map.get(&key).map(|core| core.cursor_info())
}

/// Programmatic set-text by key (MCP automation / app code). Returns true
/// when the text changed.
pub fn code_editor_set_text(key: &str, text: &str) -> bool {
    let key = normalize_payload_key(key);
    let map = CODE_EDITORS.lock().unwrap();
    if let Some(core) = map.get(&key) {
        let current = core.text();
        if current == text {
            return false;
        }
        with_font_system(|fs| core.set_text(text, fs));
        true
    } else {
        false
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
    let name = highlight::theme_name(dark, accent);
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
        assert!(list.text.is_some(), "text section must be present");
        assert!(list.gutter.is_some(), "gutter section must be present");
        assert!(list.caret.is_some(), "caret must be placed");
        assert!(!list.gutter.as_ref().unwrap().numbers.is_empty());
        assert_eq!(list.background.map(|(_, c)| c), Some(super::super::theme::CodeEditorTheme::dark("indigo").background));
    }

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
        assert!(list.text.is_some());
        assert!(list.gutter.is_some());
        // shape_as_needed only shapes the visible window; both phases stay
        // well under a frame budget on a dev machine (generous CI margin).
        assert!(set.as_secs() < 10, "set_text took {set:?}");
        assert!(rendered.as_secs() < 10, "render took {rendered:?}");
        eprintln!("large file: set={set:?} render1={rendered:?} render2={rendered2:?}");
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
}
