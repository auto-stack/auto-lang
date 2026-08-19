// Plan 413 §3.1 ① render — core::render(state, viewport, theme) -> EditorDrawList.
//
// Pure with respect to the backend: shapes the buffer with the provided
// font system, computes all decoration geometry (gutter, current line,
// selection, caret, scrollbars, preedit overlay) and returns the draw list.
// No iced imports (hard layering constraint).
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

use std::sync::atomic::Ordering;

use cosmic_text::{
    Attrs, AttrsList, BufferLine, Edit, Family, FontSystem, LineEnding, Metrics, Selection, Wrap,
};

use super::{
    CodeEditorCore, CodeEditorConfig, LayoutInfo, SCROLLBAR_THICKNESS,
};
use crate::ui::code_editor::draw::{
    CaretDraw, EditorDrawList, GutterNumber, GutterSection, PreeditDraw, Pt, Rect, ScrollbarDraw,
    TextSection,
};
use crate::ui::code_editor::theme::{current_theme, CodeEditorTheme, Rgba};

/// Padding between gutter digits and the text area.
pub const GUTTER_PAD: f32 = 6.0;
/// Caret width in logical px.
pub const CARET_WIDTH: f32 = 2.0;

/// Render one frame of the editor into a backend-neutral draw list.
///
/// `viewport` is the widget's inner size in logical pixels. The theme is
/// resolved from the semantic source when `None` is given.
pub fn render(
    core: &CodeEditorCore,
    font_system: &mut FontSystem,
    viewport_w: f32,
    viewport_h: f32,
    theme: Option<&CodeEditorTheme>,
) -> EditorDrawList {
    let owned_theme;
    let theme = match theme {
        Some(t) => t,
        None => {
            owned_theme = current_theme();
            &owned_theme
        }
    };

    let config = core.config();
    let mut list = EditorDrawList {
        revision: core.revision.load(Ordering::Relaxed),
        ..EditorDrawList::default()
    };

    if viewport_w <= 1.0 || viewport_h <= 1.0 {
        return list;
    }

    let mut editor = core.editor_lock();

    // ── gutter width (line numbers) ───────────────────────────────────────
    let (gutter_total, digits) = if config.line_numbers {
        let line_count = editor.with_buffer(|b| b.lines.len()).max(1);
        let digits = digits_of(line_count);
        let mut width = 0.0f32;
        {
            let (cached_digits, cached_w) = core.gutter_width_cache();
            if cached_digits == digits && cached_w > 0.0 {
                width = cached_w;
            } else if let Some(line) =
                gutter_probe_layout(font_system, digits).first()
            {
                width = line.w * config.font_size;
                core.set_gutter_width_cache(digits, width);
            }
        }
        ((width + GUTTER_PAD * 2.0).ceil(), digits)
    } else {
        (0.0, 0)
    };

    // ── size + shape ──────────────────────────────────────────────────────
    let text_w = (viewport_w - gutter_total).max(0.0);
    editor.with_buffer_mut(|b| {
        b.set_metrics_and_size(
            font_system,
            Metrics::new(config.font_size, config.line_height()),
            Some(text_w),
            Some(viewport_h),
        )
    });
    editor.shape_as_needed(font_system, true);

    let text_rect = Rect::new(gutter_total, 0.0, text_w, viewport_h);

    // ── visible runs scan ─────────────────────────────────────────────────
    let mut gutter_numbers: Vec<GutterNumber> = Vec::new();
    let mut max_line_width = 0.0f32;
    let mut first_visible_line = usize::MAX;
    let mut last_visible_line = 0usize;
    editor.with_buffer(|b| {
        let mut last_number = 0;
        for run in b.layout_runs() {
            max_line_width = max_line_width.max(run.line_w);
            first_visible_line = first_visible_line.min(run.line_i);
            last_visible_line = last_visible_line.max(run.line_i);
            let number = run.line_i + 1;
            if number != last_number {
                last_number = number;
                gutter_numbers.push(GutterNumber { number, y: run.line_top });
            }
        }
    });
    let visible_lines = if last_visible_line >= first_visible_line {
        last_visible_line - first_visible_line + 1
    } else {
        0
    };

    // ── background + gutter section ───────────────────────────────────────
    list.background = Some((Rect::new(0.0, 0.0, viewport_w, viewport_h), theme.background));
    if config.line_numbers {
        list.gutter = Some(GutterSection {
            bounds: Rect::new(0.0, 0.0, gutter_total, viewport_h),
            background: theme.gutter_background,
            foreground: theme.gutter_foreground,
            digits,
            font_size: config.font_size,
            line_height: config.line_height(),
            numbers: gutter_numbers,
        });
    }

    // ── current line highlight (only without selection) ───────────────────
    let has_selection = !matches!(editor.selection(), Selection::None);
    if config.highlight_current_line && !has_selection {
        let cursor_line = editor.cursor().line;
        editor.with_buffer(|b| {
            for run in b.layout_runs() {
                if run.line_i == cursor_line {
                    list.current_line = Some((
                        Rect::new(text_rect.x, run.line_top, text_w, run.line_height),
                        theme.current_line,
                    ));
                    break;
                }
            }
        });
    }

    // ── selection quads ───────────────────────────────────────────────────
    if let Some((start, end)) = editor.selection_bounds() {
        editor.with_buffer(|b| {
            for run in b.layout_runs() {
                if run.line_i < start.line || run.line_i > end.line {
                    continue;
                }
                let lo = if run.line_i == start.line { start.index } else { 0 };
                let hi = if run.line_i == end.line {
                    end.index
                } else {
                    run.text.len()
                };
                if hi <= lo {
                    continue;
                }
                if let (Some(x0), Some(x1)) = (index_x(&run, lo), index_x(&run, hi)) {
                    list.selection.push((
                        Rect::new(
                            text_rect.x + x0.min(x1),
                            run.line_top,
                            (x1 - x0).abs().max(2.0),
                            run.line_height,
                        ),
                        theme.selection,
                    ));
                }
            }
        });
    }

    // ── regex search highlights ───────────────────────────────────────────
    if let Some(regex) = core.search_regex() {
        editor.with_buffer(|b| {
            for run in b.layout_runs() {
                let Some(text) = b.lines.get(run.line_i).map(|l| l.text()) else {
                    continue;
                };
                for m in regex.find_iter(text) {
                    let (lo, hi) = (m.start(), m.end());
                    if hi <= lo {
                        continue;
                    }
                    // Soft-wrapped lines repeat the line text per run;
                    // clamp the match to this run's glyph span.
                    let run_start = run.glyphs.first().map(|g| g.start).unwrap_or(lo);
                    let run_end = run.glyphs.last().map(|g| g.end).unwrap_or(hi);
                    let lo_c = lo.clamp(run_start, run_end);
                    let hi_c = hi.clamp(run_start, run_end);
                    if hi_c <= lo_c {
                        continue;
                    }
                    if let (Some(x0), Some(x1)) = (index_x(&run, lo_c), index_x(&run, hi_c)) {
                        list.search_matches.push((
                            Rect::new(
                                text_rect.x + x0.min(x1),
                                run.line_top,
                                (x1 - x0).abs().max(2.0),
                                run.line_height,
                            ),
                            theme.search_match,
                        ));
                    }
                }
            }
        });
    }

    // ── body text ─────────────────────────────────────────────────────────
    list.text = Some(TextSection {
        buffer: core.buffer_weak(),
        origin: Pt::new(text_rect.x, text_rect.y),
        color: theme.foreground,
        clip: text_rect,
    });

    // ── caret + preedit overlay ───────────────────────────────────────────
    let mut caret_rect: Option<Rect> = None;
    if let Some((cx, cy)) = editor.cursor_position() {
        let rect = Rect::new(
            cx as f32,
            cy as f32,
            CARET_WIDTH,
            config.font_size * 1.15,
        );
        caret_rect = Some(rect);
        list.caret = Some(CaretDraw {
            rect: Rect::new(text_rect.x + rect.x, text_rect.y + rect.y, rect.w, rect.h),
            color: theme.caret,
        });
        let preedit = core.preedit();
        if let Some(preedit) = preedit.filter(|p| !p.is_empty()) {
            let px = text_rect.x + cx as f32;
            let py = text_rect.y + cy as f32;
            list.preedit = Some(PreeditDraw {
                text: preedit,
                origin: Pt::new(px, py),
                font_size: config.font_size,
                color: theme.foreground,
                underline: Rect::new(
                    px,
                    py + config.font_size * 1.15 - 2.0,
                    text_w.min(240.0),
                    1.5,
                ),
            });
        }
    }

    // ── scrollbars ────────────────────────────────────────────────────────
    let total_lines = editor.with_buffer(|b| b.lines.len());
    let (scroll_x, scroll_y) = editor.with_buffer(|b| {
        let s = b.scroll();
        (s.horizontal, s.vertical)
    });
    let scrollbar_v = if visible_lines < total_lines {
        let track_h = viewport_h - SCROLLBAR_THICKNESS - 2.0;
        let thumb_h =
            (viewport_h * (visible_lines.max(1) as f32 / total_lines as f32)).clamp(SCROLLBAR_THICKNESS, track_h);
        let frac = if visible_lines > 0 {
            (first_visible_line as f32) / (total_lines as f32)
        } else {
            0.0
        };
        let _ = scroll_y;
        Some(Rect::new(
            viewport_w - SCROLLBAR_THICKNESS - 2.0,
            2.0 + frac * (track_h - thumb_h),
            SCROLLBAR_THICKNESS,
            thumb_h,
        ))
    } else {
        None
    };
    let scrollbar_h = if max_line_width > text_w + 0.5 {
        let track_w = text_w - SCROLLBAR_THICKNESS - 2.0;
        let thumb_w = (track_w * (text_w / max_line_width)).clamp(SCROLLBAR_THICKNESS, track_w);
        let frac = (scroll_x / max_line_width).clamp(0.0, 1.0);
        Some(Rect::new(
            2.0 + frac * (track_w - thumb_w),
            viewport_h - SCROLLBAR_THICKNESS - 2.0,
            thumb_w,
            SCROLLBAR_THICKNESS,
        ))
    } else {
        None
    };
    if let Some(sb) = scrollbar_v {
        list.scrollbar_v = Some(ScrollbarDraw { thumb: sb, color: theme.scrollbar });
    }
    if let Some(sb) = scrollbar_h {
        list.scrollbar_h = Some(ScrollbarDraw { thumb: sb, color: theme.scrollbar });
    }

    // ── record layout for hit testing ─────────────────────────────────────
    core.record_layout(LayoutInfo {
        viewport_w,
        viewport_h,
        text: text_rect,
        scrollbar_v,
        scrollbar_h,
        caret: caret_rect,
        max_line_width,
        visible_lines,
    });

    // Clear the redraw flag after a successful paint pass.
    let needs_redraw = editor.redraw();
    if needs_redraw {
        editor.set_redraw(false);
    }

    list
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

fn digits_of(mut n: usize) -> usize {
    let mut digits = 1;
    while n >= 10 {
        n /= 10;
        digits += 1;
    }
    digits
}

/// Lay out a gutter-width probe (the number `1` zero-padded to `digits`)
/// at font size 1.0 — the width, scaled by the real font size, is the
/// gutter's content width. cosmic-text caches shaped lines, so repeated
/// probes are cheap.
fn gutter_probe_layout(font_system: &mut FontSystem, digits: usize) -> Vec<cosmic_text::LayoutLine> {
    let attrs = Attrs::new().family(Family::Monospace);
    let text = format!("{:>width$}", 1, width = digits);
    let mut line = BufferLine::new(
        text,
        LineEnding::None,
        AttrsList::new(&attrs),
        cosmic_text::Shaping::Advanced,
    );
    line.layout(font_system, 1.0, None, Wrap::None, None, 8).to_vec()
}

/// Byte index → x offset within a laid-out run (walks glyph clusters,
/// interpolating inside multi-char clusters).
fn index_x(run: &cosmic_text::LayoutRun, index: usize) -> Option<f32> {
    let mut prev_end = 0.0f32;
    for glyph in run.glyphs {
        if index < glyph.start {
            return Some(prev_end);
        }
        if index <= glyph.end {
            let cluster = &run.text[glyph.start..glyph.end];
            let total = cluster.chars().count().max(1) as f32;
            let before = run.text[glyph.start..index.min(glyph.end)].chars().count() as f32;
            return Some(glyph.x + (glyph.w / total) * before);
        }
        prev_end = glyph.x + glyph.w;
    }
    Some(prev_end)
}

/// Re-export for the iced adapter: the default theme resolution used when
/// no explicit theme is passed.
pub fn resolved_theme() -> CodeEditorTheme {
    current_theme()
}

/// Convert an [`Rgba`] to cosmic-text's color for the raster path.
pub fn rgba_to_cosmic(color: Rgba) -> cosmic_text::Color {
    cosmic_text::Color::rgba(
        (color.r * 255.0).round().clamp(0.0, 255.0) as u8,
        (color.g * 255.0).round().clamp(0.0, 255.0) as u8,
        (color.b * 255.0).round().clamp(0.0, 255.0) as u8,
        (color.a * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// Expose config construction used by tests.
pub fn default_config() -> CodeEditorConfig {
    CodeEditorConfig::default()
}
