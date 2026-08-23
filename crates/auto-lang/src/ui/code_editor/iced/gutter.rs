// Plan 413 §3.1 ③ iced adapter — gutter (line-number column) CPU raster.
//
// The draw list carries line numbers as data; this module turns them into
// a cached RGBA image: each number is laid out once at font size 1.0
// (cached per (number, digits)), scaled to the real font size via
// `physical()`, and alpha-blended pixel by pixel through the swash cache.
// Rasterization runs at the real size, so `FilterMethod::Nearest` stays
// crisp; the image is only regenerated when the buffer revision or the
// gutter width changes.
//
// In the RenderCommand lowering path (Plan 386) the same numbers are drawn
// as plain text runs — the image is purely this adapter's cache
// optimization (§7.3).
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

use std::collections::HashMap;

use cosmic_text::{
    Attrs, AttrsList, BufferLine, Family, FontSystem, LayoutLine, LineEnding, Shaping, SwashCache,
    Wrap,
};

use crate::ui::code_editor::core::render::{rgba_to_cosmic, FOLD_GUTTER_W, GUTTER_PAD};
use crate::ui::code_editor::draw::GutterSection;

/// Widget-tree state for the gutter raster cache.
pub struct GutterCache {
    /// (number, digits) → layout at font size 1.0.
    layouts: HashMap<(usize, usize), Vec<LayoutLine>>,
    /// (image handle, gutter width, gutter height, revision) of the current
    /// raster. Height must key too: a shorter viewport (e.g. the console
    /// panel opening) would otherwise scale the stale taller image into the
    /// new quad — squeezed digits, misaligned rows.
    image: Option<(iced::advanced::image::Handle, f32, f32, u64)>,
    swash: SwashCache,
}

impl Default for GutterCache {
    fn default() -> Self {
        Self {
            layouts: HashMap::new(),
            image: None,
            swash: SwashCache::new(),
        }
    }
}

impl GutterCache {
    /// Get (or lazily create) the font-size-1.0 layout of one number.
    fn layout(
        &mut self,
        font_system: &mut FontSystem,
        number: usize,
        digits: usize,
    ) -> &Vec<LayoutLine> {
        self.layouts
            .entry((number, digits))
            .or_insert_with(|| {
                // Plan 414 §6.4: same family as the body (baseline parity).
                let attrs = Attrs::new().family(crate::ui::code_editor::core::mono_family());
                let text = format!("{number:>digits$}");
                let mut line = BufferLine::new(
                    text,
                    LineEnding::None,
                    AttrsList::new(&attrs),
                    Shaping::Advanced,
                );
                line.layout(font_system, 1.0, None, Wrap::None, None, 8).to_vec()
            })
    }

    /// Rasterize the gutter if stale; return the (handle, width) pair.
    pub fn image(
        &mut self,
        section: &GutterSection,
        font_system: &mut FontSystem,
        revision: u64,
    ) -> Option<(iced::advanced::image::Handle, f32)> {
        let width = section.bounds.w.ceil() as u32;
        let height = section.bounds.h.ceil() as u32;
        if width == 0 || height == 0 {
            return None;
        }

        let fresh = match &self.image {
            Some((_, cached_w, cached_h, cached_rev)) => {
                (*cached_w - section.bounds.w).abs() <= 0.5
                    && (*cached_h - section.bounds.h).abs() <= 0.5
                    && *cached_rev == revision
            }
            None => false,
        };
        if fresh {
            return self.image.as_ref().map(|(h, w, _, _)| (h.clone(), *w));
        }

        let bg = rgba_to_cosmic(section.background);
        let fg = rgba_to_cosmic(section.foreground);
        let mut rgba = vec![0u8; (width as usize) * (height as usize) * 4];
        for px in rgba.chunks_exact_mut(4) {
            px[0] = bg.r();
            px[1] = bg.g();
            px[2] = bg.b();
            px[3] = bg.a();
        }

        // Duplicate line numbers can only come from wrapped layouts; keep
        // the first occurrence per visible line.
        let mut last_number = 0;
        for entry in &section.numbers {
            if entry.number == last_number {
                continue;
            }
            last_number = entry.number;

            let layout = {
                let layouts = self.layout(font_system, entry.number, section.digits);
                let Some(line) = layouts.first() else { continue };
                line.clone()
            };

            // Scale the 1.0-size layout to the real font size.
            let max_ascent = layout.max_ascent * section.font_size;
            let max_descent = layout.max_descent * section.font_size;
            let glyph_height = max_ascent + max_descent;
            let centering = (section.line_height - glyph_height) / 2.0;
            let line_y = entry.y + centering + max_ascent;

            for glyph in &layout.glyphs {
                // Plan 414 §6.3: RIGHT-align the digits against the fold
                // column — a short "1" must not widen the visual gap.
                let run_w = layout.w * section.font_size;
                let right_edge = width as f32 - FOLD_GUTTER_W;
                let x_anchor = (right_edge - run_w).max(GUTTER_PAD);
                let physical = glyph.physical((x_anchor, line_y), section.font_size);
                self.swash.with_pixels(
                    font_system,
                    physical.cache_key,
                    fg,
                    |x, y, color| {
                        blend_pixel(
                            &mut rgba,
                            width,
                            height,
                            physical.x + x,
                            physical.y + y,
                            color,
                        );
                    },
                );
            }
        }

        // Plan 414 §5 → Plan 428 P3: fold chevrons in the tool column
        // between the numbers and the text — two-state affordances: expanded
        // blocks draw a downward triangle (▾, click to fold), folded blocks
        // a rightward one (▸, click to expand). Slightly dimmed so they
        // read as affordances rather than content.
        let fold_color = cosmic_text::Color::rgba(
            fg.r(),
            fg.g(),
            fg.b(),
            ((fg.a() as f32) * 0.8) as u8,
        );
        for fold in &section.folds {
            let cx = width as f32 - FOLD_GUTTER_W / 2.0;
            let cy = fold.y + section.line_height / 2.0;
            raster_triangle(
                &mut rgba,
                width,
                height,
                cx,
                cy,
                fold_color,
                if fold.folded {
                    TriangleDir::Right
                } else {
                    TriangleDir::Down
                },
            );
        }

        let handle = iced::advanced::image::Handle::from_rgba(width, height, rgba);
        self.image = Some((handle.clone(), section.bounds.w, section.bounds.h, revision));
        Some((handle, section.bounds.w))
    }
}

/// Chevron pointing direction (Plan 428 P3: expanded ▾ / folded ▸).
#[derive(Clone, Copy, PartialEq)]
enum TriangleDir {
    Down,
    Right,
}

/// Scanline-fill a small triangle (chevron affordance) centered at (cx, cy):
/// 7px wide, ~5px tall. `Down` points at the folded-away body; `Right`
/// points along the collapsed marker row.
fn raster_triangle(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    cx: f32,
    cy: f32,
    color: cosmic_text::Color,
    dir: TriangleDir,
) {
    let half_w = 3.5f32;
    let half_h = 2.5f32;
    match dir {
        TriangleDir::Down => {
            let top = (cy - half_h).round() as i32;
            let bottom = (cy + half_h).round() as i32;
            for py in top..=bottom {
                let t = ((py as f32) - (cy - half_h)) / (half_h * 2.0);
                let t = t.clamp(0.0, 1.0);
                let row_half = half_w * t;
                let x0 = (cx - row_half).round() as i32;
                let x1 = (cx + row_half).round() as i32;
                for px in x0..=x1 {
                    blend_pixel(rgba, width, height, px, py, color);
                }
            }
        }
        TriangleDir::Right => {
            // Base (full half-height) at the left edge, tip at the right:
            // 7px wide, ~5px tall, mirroring the Down variant's footprint.
            let left = (cx - half_w).round() as i32;
            let right = (cx + half_w).round() as i32;
            for px in left..=right {
                let t = ((px as f32) - (cx - half_w)) / (half_w * 2.0);
                let t = t.clamp(0.0, 1.0);
                let col_half = half_h * (1.0 - t);
                let y0 = (cy - col_half).round() as i32;
                let y1 = (cy + col_half).round() as i32;
                for py in y0..=y1 {
                    blend_pixel(rgba, width, height, px, py, color);
                }
            }
        }
    }
}

/// Alpha-blend one pixel of a `cosmic_text::Color` onto the RGBA canvas.
fn blend_pixel(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    color: cosmic_text::Color,
) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let idx = (y as usize * width as usize + x as usize) * 4;
    let sa = color.a() as f32 / 255.0;
    let da = rgba[idx + 3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return;
    }
    for (c, sc) in [(0, color.r()), (1, color.g()), (2, color.b())] {
        let dst = rgba[idx + c] as f32 / 255.0;
        let src = sc as f32 / 255.0;
        rgba[idx + c] = (((src * sa + dst * da * (1.0 - sa)) / out_a) * 255.0).round() as u8;
    }
    rgba[idx + 3] = (out_a * 255.0).round() as u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::code_editor::draw::{GutterNumber, GutterSection, Rect};
    use crate::ui::code_editor::theme::Rgba;

    fn section(height: f32) -> GutterSection {
        GutterSection {
            bounds: Rect::new(0.0, 0.0, 60.0, height),
            background: Rgba { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
            foreground: Rgba { r: 0.8, g: 0.8, b: 0.8, a: 1.0 },
            digits: 2,
            font_size: 14.0,
            line_height: 19.0,
            numbers: vec![GutterNumber { number: 1, y: 0.0 }, GutterNumber { number: 2, y: 19.0 }],
            folds: Vec::new(),
        }
    }

    /// Plan 428 实机验收发现(413 期回归):console 打开使编辑器变矮,缓存
    /// 只按宽度+revision 判新鲜,旧高光栅被缩放进矮 quad —— 行号纵向压缩、
    /// 与正文错位。高度变化必须触发重光栅。
    #[test]
    fn shorter_viewport_rasterizes_not_scales() {
        let mut fs = FontSystem::new();
        let mut cache = GutterCache::default();
        let (handle1, _) = cache.image(&section(120.0), &mut fs, 7).expect("raster");
        let _ = handle1;
        // Same width + revision, shorter viewport (console opened): the
        // cache must NOT serve the stale 120px-tall raster.
        let _ = cache.image(&section(80.0), &mut fs, 7).expect("re-raster");
        let (_, _, cached_h, cached_rev) = cache.image.as_ref().unwrap();
        assert!(
            (cached_h - 80.0).abs() <= 0.5,
            "height change must re-rasterize (cache h={cached_h})"
        );
        assert_eq!(*cached_rev, 7);
        // Unchanged geometry + revision keeps the cache (no churn per frame).
        let _ = cache.image(&section(80.0), &mut fs, 7).expect("cached serve");
        let (_, _, h2, r2) = cache.image.as_ref().unwrap();
        assert!((h2 - 80.0).abs() <= 0.5 && *r2 == 7);
    }
}
