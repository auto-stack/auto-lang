// Plan 413 §3.1 ② draw-list layer — backend-neutral rendering contract.
//
// `EditorDrawList` is what `core::render` produces and what any backend
// adapter consumes. It carries no iced types and no renderer state; the
// iced adapter lowers it to `fill_quad` / `fill_text` / image calls, a
// future RenderCommand lowering (Plan 386 Stage 1) serializes it to quads
// and text runs.
//
// Plan 428 P2: the body text is no longer a weak buffer handed to a single
// `fill_raw` — folding must skip hidden lines, so render extracts the
// visible runs itself (per syntax-color span) as owned `TextRun` pieces.
// All geometry stays in ORIGINAL buffer coordinates; each piece's y is the
// folded-view projection (`fold::FoldMap::project_y`).
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

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

/// One piece of body text (Plan 428 P2): a syntax-color span slice of one
/// visible layout run, positioned in widget-local logical pixels. Pieces are
/// owned strings — backends shape them through the same cosmic-text stack
/// (same font system, family, size, `Wrapping::None`), which the P0 spike
/// verified keeps monospace advance widths identical to the buffer's own
/// shaping (§6(a)).
#[derive(Debug, Clone)]
pub struct TextRun {
    pub text: String,
    /// Left edge of the piece (widget-local; includes the gutter offset).
    pub x: f32,
    /// Top of the line in FOLDED-VIEW coordinates (hidden lines removed).
    pub y: f32,
    pub size: f32,
    pub line_height: f32,
    pub color: Rgba,
}

/// One fold affordance in the gutter's fold column (Plan 428 P3: two-state
/// chevron; `y` is the projected top of the opener line).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GutterFold {
    pub y: f32,
    pub folded: bool,
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
    /// Fold affordances in the fold column (Plan 428 P3: two-state
    /// chevrons — expanded ▾ / folded ▸ — at foldable opener lines).
    pub folds: Vec<GutterFold>,
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
    /// Body text pieces (Plan 428 P2: per visible run × syntax span).
    pub text_runs: Vec<TextRun>,
    /// Lines currently hidden by folding — adapters mix this into their
    /// raster cache keys (fold toggles must invalidate the gutter image
    /// even though the text revision is unchanged).
    pub fold_hidden: usize,
    pub caret: Option<CaretDraw>,
    pub preedit: Option<PreeditDraw>,
    pub scrollbar_v: Option<ScrollbarDraw>,
    pub scrollbar_h: Option<ScrollbarDraw>,
    /// Monotonic revision of the underlying buffer — adapters key their
    /// raster caches (gutter image) on this.
    pub revision: u64,
}
