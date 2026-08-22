// Plan 413 §3.1 ② draw-list layer — backend-neutral rendering contract.
//
// `EditorDrawList` is what `core::render` produces and what any backend
// adapter consumes. It carries no iced types and no renderer state; the
// iced adapter lowers it to `fill_quad` / `fill_raw` / image calls, a future
// RenderCommand lowering (Plan 386 Stage 1) serializes it to quads/text
// runs. Text is expressed as a weak buffer reference — valid in-process
// (pointer semantics for `fill_raw`); the lowering path re-expresses each
// visible run as shaped glyphs (design 20 §4.1).
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

use std::sync::Weak;

use cosmic_text::Buffer;

use super::theme::Rgba;

/// Point in logical (widget-local) coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pt {
    pub x: f32,
    pub y: f32,
}

impl Pt {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Rectangle in logical (widget-local) coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, p: Pt) -> bool {
        p.x >= self.x && p.x < self.x + self.w && p.y >= self.y && p.y < self.y + self.h
    }
}

/// The body text: a weak reference to the cosmic-text buffer plus placement.
/// In-process backends hand this straight to `fill_raw`; a separating
/// backend lowers it per visible run (see module docs).
#[derive(Debug, Clone)]
pub struct TextSection {
    /// Weak reference to the engine-owned buffer (same-process handoff).
    pub buffer: Weak<Buffer>,
    /// Top-left of the text area, relative to the widget.
    pub origin: Pt,
    pub color: Rgba,
    /// Clip rectangle (the text area).
    pub clip: Rect,
}

/// One visible line number (stable per line index — the key a separating
/// backend would cache by, Plan 413 §7.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GutterNumber {
    /// 1-based line number.
    pub number: usize,
    /// Top of the line's layout run, relative to the widget.
    pub y: f32,
}

/// Gutter (line-number column) section.
#[derive(Debug, Clone)]
pub struct GutterSection {
    pub bounds: Rect,
    pub background: Rgba,
    pub foreground: Rgba,
    /// Number of digit columns the gutter is sized for.
    pub digits: usize,
    pub font_size: f32,
    pub line_height: f32,
    pub numbers: Vec<GutterNumber>,
    /// Y positions of foldable-block opener lines (Plan 414 §5 Phase A:
    /// visual chevrons in the fold column between numbers and text).
    pub folds: Vec<f32>,
}

/// Caret (text cursor) rectangle.
#[derive(Debug, Clone, Copy)]
pub struct CaretDraw {
    pub rect: Rect,
    pub color: Rgba,
}

/// On-the-spot IME preedit overlay (drawn at the caret).
#[derive(Debug, Clone)]
pub struct PreeditDraw {
    pub text: String,
    /// Where the preedit text starts.
    pub origin: Pt,
    pub font_size: f32,
    pub color: Rgba,
    /// Underline below the preedit text.
    pub underline: Rect,
}

/// Scrollbar thumb rectangle + track.
#[derive(Debug, Clone, Copy)]
pub struct ScrollbarDraw {
    /// The draggable thumb.
    pub thumb: Rect,
    pub color: Rgba,
}

/// Complete rendering instructions for one frame of a code editor.
#[derive(Debug, Clone, Default)]
pub struct EditorDrawList {
    /// Editor panel background (full widget bounds).
    pub background: Option<(Rect, Rgba)>,
    pub gutter: Option<GutterSection>,
    /// Current-line highlight (drawn under the text).
    pub current_line: Option<(Rect, Rgba)>,
    /// Selection rectangles (one per affected layout run).
    pub selection: Vec<(Rect, Rgba)>,
    /// Regex search match rectangles (one per match per affected run).
    pub search_matches: Vec<(Rect, Rgba)>,
    pub text: Option<TextSection>,
    pub caret: Option<CaretDraw>,
    pub preedit: Option<PreeditDraw>,
    pub scrollbar_v: Option<ScrollbarDraw>,
    pub scrollbar_h: Option<ScrollbarDraw>,
    /// Monotonic revision of the underlying buffer — adapters key their
    /// raster caches (gutter image) on this.
    pub revision: u64,
}
