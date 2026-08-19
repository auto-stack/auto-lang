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

use crate::ui::code_editor::core::render::{rgba_to_cosmic, GUTTER_PAD};
use crate::ui::code_editor::draw::GutterSection;

/// Widget-tree state for the gutter raster cache.
pub struct GutterCache {
    /// (number, digits) → layout at font size 1.0.
    layouts: HashMap<(usize, usize), Vec<LayoutLine>>,
    /// (image handle, gutter width, revision) of the current raster.
    image: Option<(iced::advanced::image::Handle, f32, u64)>,
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
                let attrs = Attrs::new().family(Family::Monospace);
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
            Some((_, cached_w, cached_rev)) => {
                (*cached_w - section.bounds.w).abs() <= 0.5 && *cached_rev == revision
            }
            None => false,
        };
        if fresh {
            return self.image.as_ref().map(|(h, w, _)| (h.clone(), *w));
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
                let physical = glyph.physical((GUTTER_PAD, line_y), section.font_size);
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

        let handle = iced::advanced::image::Handle::from_rgba(width, height, rgba);
        self.image = Some((handle.clone(), section.bounds.w, revision));
        Some((handle, section.bounds.w))
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
