//! Plan 428 P0 spike — Route A (per-layout-run drawing) microbenchmark.
//!
//! Mirrors the editor's rendering pipeline (crates/auto-lang/src/ui/code_editor):
//! - fake/lazy viewport: finite size (800 x ~screenful) so `set_text` /
//!   `shape_until_scroll` only shape the visible window (Plan 413's 26s/1MB
//!   lesson);
//! - font: Consolas on Windows (same as `core::mono_family`), size 14,
//!   line_height = (14 * 4/3).round() = 19 (same as `CodeEditorConfig::line_height`);
//! - Shaping::Advanced, attrs Family::Name("Consolas").
//!
//! Measures, for ~1000-line and ~10000-line synthetic Auto-like sources:
//!   (a) set_text (with finite viewport => lazy) + shape_until_scroll(prune)
//!       for the first screenful, and for a deep scroll (line ~ N/2);
//!   (b) per-frame Route A extraction: iterate `layout_runs()` for one
//!       screenful and collect (owned text, rect) per run — the data iced's
//!       fill_text path would need per frame;
//!   (c) layout run count per screenful (the per-run fill_text call count).

use std::time::{Duration, Instant};

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Scroll, Shaping};

const VIEWPORT_W: f32 = 800.0;
const FONT_SIZE: f32 = 14.0;
const LINES_PER_SCREEN: usize = 50;
// CodeEditorConfig::line_height: (font_size * 4/3).round()
const LINE_H: f32 = 19.0;

fn mono_attrs() -> Attrs<'static> {
    // core::mono_family(): Consolas on Windows, Monospace otherwise.
    if cfg!(windows) {
        Attrs::new().family(cosmic_text::Family::Name("Consolas"))
    } else {
        Attrs::new().family(cosmic_text::Family::Monospace)
    }
}

/// Synthetic Auto-language-like source (~40 chars/line average):
/// braces, nested blocks, strings, comments.
fn synth_source(n_lines: usize) -> String {
    let mut out = String::with_capacity(n_lines * 48);
    let mut depth: usize = 0;
    let mut i = 0;
    while i < n_lines {
        let r = i % 10;
        let line = match r {
            0 => {
                depth += 1;
                format!("{}fn handler_{}(ctx: &Ctx, ev: Event) {{\n", "    ".repeat(depth.saturating_sub(1).min(3)), i)
            }
            1 | 2 | 3 => format!("{}let value_{} = compute(\"payload_{}\", 42);\n", "    ".repeat(depth.min(4)), i, i),
            4 => format!("{}// fast path: skip when event is a no-op\n", "    ".repeat(depth.min(4))),
            5 => format!("{}if value_{}.kind == Kind::Live {{\n", "    ".repeat(depth.min(4)), i),
            6 => format!("{}    ctx.emit(Event::Done(value_{}.id));\n", "    ".repeat(depth.min(4)), i),
            7 => format!("{}}}\n", "    ".repeat(depth.min(4))),
            8 => format!("{}return Ok(value_{}.score);\n", "    ".repeat(depth.min(4)), i),
            _ => {
                depth = depth.saturating_sub(1);
                format!("{}}}\n", "    ".repeat(depth.min(4)))
            }
        };
        out.push_str(&line);
        i += 1;
    }
    out
}

struct RunExtract {
    text: String,
    // rect: (x, y_top, w, h) — x starts at 0; a real widget adds gutter offset.
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

/// Route A per-frame extraction: collect (text, rect) per layout run for one
/// screenful. This is the honest cost floor of the data iced fill_text calls
/// would consume. `fold_skip` emulates folding by dropping runs whose line_i
/// falls in the given hidden set (vertical compaction not counted — it's a
/// per-run y-offset adjustment folded into the same pass).
fn extract_runs(buffer: &Buffer, hidden: &[bool], out: &mut Vec<RunExtract>) -> usize {
    out.clear();
    let mut runs = 0usize;
    for run in buffer.layout_runs() {
        runs += 1;
        if hidden.get(run.line_i).copied().unwrap_or(false) {
            continue;
        }
        let (s, e) = match (run.glyphs.first(), run.glyphs.last()) {
            (Some(f), Some(l)) => (f.start, l.end),
            _ => (0, run.text.len()),
        };
        out.push(RunExtract {
            text: run.text[s.min(run.text.len())..e.min(run.text.len())].to_owned(),
            x: run.glyphs.first().map(|g| g.x).unwrap_or(0.0),
            y: run.line_top,
            w: run.line_w,
            h: run.line_height,
        });
    }
    runs
}

fn bench_case(name: &str, n_lines: usize) {
    let source = synth_source(n_lines);

    let mut font_system = FontSystem::new();

    // Buffer with finite viewport (editor-style lazy shaping).
    let metrics = Metrics::new(FONT_SIZE, LINE_H);
    let mut buffer = Buffer::new_empty(metrics);
    let viewport_h = LINE_H * LINES_PER_SCREEN as f32;
    buffer.set_size(&mut font_system, Some(VIEWPORT_W), Some(viewport_h));

    // (a1) set_text with finite viewport (the fake-viewport trick).
    let t0 = Instant::now();
    buffer.set_text(&mut font_system, &source, &mono_attrs(), Shaping::Advanced, None);
    let set_text = t0.elapsed();

    // (a2) shape_until_scroll at top of file (mirrors render()'s
    // shape_as_needed(true) after set_size).
    let t0 = Instant::now();
    buffer.shape_until_scroll(&mut font_system, true);
    let shape_top = t0.elapsed();

    // (b) Route A extraction at top, repeated (steady-state scrolling frames
    // don't reshaped already-shaped lines).
    let mut sink: Vec<RunExtract> = Vec::new();
    let hidden = vec![false; n_lines];
    let mut iters = 0usize;
    let mut runs_per_screen = 0usize;
    let t0 = Instant::now();
    let reps = 200;
    for _ in 0..reps {
        runs_per_screen = extract_runs(&buffer, &hidden, &mut sink);
        iters += sink.len();
    }
    let extract_top = t0.elapsed() / reps as u32;
    let _ = iters;

    // (a3) deep scroll: jump to line n/2 (editor would set_scroll then
    // shape_until_scroll — new window must be shaped).
    buffer.set_scroll(Scroll::new(n_lines / 2, 0.0, 0.0));
    let t0 = Instant::now();
    buffer.shape_until_scroll(&mut font_system, true);
    let shape_deep = t0.elapsed();

    // (b2) extraction at deep scroll.
    let t0 = Instant::now();
    for _ in 0..reps {
        runs_per_screen = extract_runs(&buffer, &hidden, &mut sink);
    }
    let extract_deep = t0.elapsed() / reps as u32;

    // (b3) folded variant: hide ~60% of the openers' bodies (every block whose
    // opener line_i % 10 == 0, hide the next 4 lines) — a heavily folded doc
    // has FEWER visible runs, so this is the friendly case; the pass still
    // walks the run iterator the same way.
    let mut hidden_folded = vec![false; n_lines];
    for (i, h) in hidden_folded.iter_mut().enumerate() {
        *h = i % 10 == 1 || i % 10 == 2 || i % 10 == 3;
    }
    let t0 = Instant::now();
    for _ in 0..reps {
        extract_runs(&buffer, &hidden_folded, &mut sink);
    }
    let extract_folded = t0.elapsed() / reps as u32;
    let visible_runs_folded = sink.len();

    // (a4) scroll-by-one-line frame: the incremental shape cost of a typical
    // scrolling frame (advance scroll by one line, shape, then next frame back).
    buffer.set_scroll(Scroll::new(n_lines / 2 + 1, 0.0, 0.0));
    let t0 = Instant::now();
    buffer.shape_until_scroll(&mut font_system, true);
    let shape_next_line = t0.elapsed();

    let report = |label: &str, d: Duration| {
        println!("  {label:<42} {:>10.3} ms", d.as_secs_f64() * 1e3);
    };
    println!("== {name} ({n_lines} lines, viewport {VIEWPORT_W}x{:.0}) ==", viewport_h);
    report("set_text (lazy viewport)", set_text);
    report("shape_until_scroll @ top", shape_top);
    report("shape_until_scroll @ line n/2", shape_deep);
    report("shape_until_scroll @ next line", shape_next_line);
    report("extract (text,rect) x200 @ top", extract_top);
    report("extract x200 @ deep scroll", extract_deep);
    report("extract x200 (60% folded)", extract_folded);
    println!("  layout runs walked per screenful: {runs_per_screen}");
    println!("  visible fill_text calls (60% folded): {visible_runs_folded}");
    // Estimated per-run GPU/driver overhead for fill_text: 1-5 us each.
    let runs = runs_per_screen as f64;
    println!(
        "  est. fill_text overhead @1us/run: {:.3} ms, @5us/run: {:.3} ms",
        runs * 1e-3,
        runs * 5e-3
    );
    let frame_a = extract_top.as_secs_f64() * 1e3 + runs * 1e-3;
    let frame_b = extract_top.as_secs_f64() * 1e3 + runs * 5e-3;
    println!(
        "  Route A per-frame est. (extract + fill_text): {:.3} - {:.3} ms  [60fps budget 16.6 ms]",
        frame_a, frame_b
    );
    // Touch the sink so extraction is not optimized away.
    let total: usize = sink.iter().map(|r| r.text.len()).sum();
    println!("  (sink bytes: {total})");
    println!();
}

fn main() {
    println!("cosmic-text 0.15.0 (pinned, matches root Cargo.lock)");
    bench_case("small", 1_000);
    bench_case("large", 10_000);
}
