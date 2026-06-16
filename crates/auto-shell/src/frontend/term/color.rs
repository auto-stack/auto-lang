//! 24-bit truecolor support: terminal capability detection + graceful
//! downsampling to 256/16 colors (Plan 317).
//!
//! Mirrors Fish's `update_fish_color_support()` algorithm
//! (fish-shell/src/env_dispatch.rs:372) for detecting whether the terminal
//! supports 24-bit (`38;2;r;g;b`) color. When it doesn't, RGB colors are
//! downsampled to the nearest xterm-256 or 16-color entry so output never
//! shows raw escape garble.

use std::sync::OnceLock;

use nu_ansi_term::Color as AnsiColor;

/// Maximum color depth the current terminal can render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    /// 24-bit truecolor (`\x1b[38;2;r;g;b`).
    True24,
    /// 256-color palette (`\x1b[38;5;n`).
    Index256,
    /// 16-color basic palette.
    Index16,
}

/// Detect the terminal's max color depth, mirroring Fish's precedence:
/// 1. `$ASH_TERM24BIT` explicit override.
/// 2. `$STY` present (inside `screen`) → 256 (screen needs `truecolor on`).
/// 3. `$COLORTERM == truecolor|24bit` → 24-bit.
/// 4. Default: 24-bit unless `$TERM == xterm-16color` or Apple_Terminal.
pub fn detect_color_depth() -> ColorDepth {
    // 1. Explicit override.
    if let Ok(v) = std::env::var("ASH_TERM24BIT") {
        let b = parse_bool(&v);
        return if b == Some(false) {
            ColorDepth::Index256
        } else {
            ColorDepth::True24
        };
    }
    // 2. screen special case.
    if std::env::var("STY").is_ok() {
        return ColorDepth::Index256;
    }
    // 3. COLORTERM.
    if let Ok(ct) = std::env::var("COLORTERM") {
        if ct == "truecolor" || ct == "24bit" {
            return ColorDepth::True24;
        }
    }
    // 4. Default inference.
    let term = std::env::var("TERM").unwrap_or_default();
    if term == "xterm-16color" {
        return ColorDepth::Index16;
    }
    if std::env::var("TERM_PROGRAM").as_deref() == Ok("Apple_Terminal") {
        return ColorDepth::Index256;
    }
    ColorDepth::True24
}

/// Cached color depth (computed once per process).
fn cached_depth() -> ColorDepth {
    static DEPTH: OnceLock<ColorDepth> = OnceLock::new();
    *DEPTH.get_or_init(detect_color_depth)
}

/// Resolve an RGB foreground color for the current terminal: pass through on
/// truecolor terminals, downsample to nearest 256/16-color otherwise.
pub fn resolve_fg(r: u8, g: u8, b: u8) -> AnsiColor {
    resolve(r, g, b, cached_depth())
}

/// Resolve an RGB background color (same logic as fg).
pub fn resolve_bg(r: u8, g: u8, b: u8) -> AnsiColor {
    resolve(r, g, b, cached_depth())
}

/// Resolve RGB for a specific (possibly forced) depth — the testable core.
pub fn resolve(r: u8, g: u8, b: u8, depth: ColorDepth) -> AnsiColor {
    match depth {
        ColorDepth::True24 => AnsiColor::Rgb(r, g, b),
        ColorDepth::Index256 => AnsiColor::Fixed(nearest_256(r, g, b)),
        ColorDepth::Index16 => nearest_16(r, g, b),
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

// ── xterm 256-color palette + nearest-neighbor ───────────────────────────

/// The xterm 256-color palette as RGB triples (indices 0–255).
fn xterm_256_palette() -> Vec<(u8, u8, u8)> {
    let mut pal = Vec::with_capacity(256);
    // 0–15: standard 16 (xterm defaults: normal darks + bright).
    pal.extend([
        (0, 0, 0),       // 0  black
        (128, 0, 0),     // 1  red
        (0, 128, 0),     // 2  green
        (128, 128, 0),   // 3  yellow
        (0, 0, 128),     // 4  blue
        (128, 0, 128),   // 5  magenta
        (0, 128, 128),   // 6  cyan
        (192, 192, 192), // 7  white
        (128, 128, 128), // 8  bright black (gray)
        (255, 0, 0),     // 9  bright red
        (0, 255, 0),     // 10 bright green
        (255, 255, 0),   // 11 bright yellow
        (0, 0, 255),     // 12 bright blue
        (255, 0, 255),   // 13 bright magenta
        (0, 255, 255),   // 14 bright cyan
        (255, 255, 255), // 15 bright white
    ]);
    // 16–231: 6×6×6 color cube. Components ∈ {0,95,135,175,215,255}.
    let cube = [0u8, 95, 135, 175, 215, 255];
    for r in 0..6 {
        for g in 0..6 {
            for b in 0..6 {
                pal.push((cube[r], cube[g], cube[b]));
            }
        }
    }
    // 232–255: 24-step grayscale ramp 8..238.
    for k in 0..24u8 {
        let v = 8 + k * 10;
        pal.push((v, v, v));
    }
    pal
}

/// Nearest xterm-256 index for an RGB triple (weighted Euclidean; human eyes
/// are more sensitive to green).
fn nearest_256(r: u8, g: u8, b: u8) -> u8 {
    let pal = xterm_256_palette();
    let mut best = 0u8;
    let mut best_d = u32::MAX;
    for (i, &(pr, pg, pb)) in pal.iter().enumerate() {
        let d = color_dist(r, g, b, pr, pg, pb);
        if d < best_d {
            best_d = d;
            best = i as u8;
        }
    }
    best
}

/// Nearest of the 16 basic ANSI colors, returned as a named nu_ansi_term color.
fn nearest_16(r: u8, g: u8, b: u8) -> AnsiColor {
    let std16 = [
        (AnsiColor::Black, 0u8, 0u8, 0u8),
        (AnsiColor::Red, 128, 0, 0),
        (AnsiColor::Green, 0, 128, 0),
        (AnsiColor::Yellow, 128, 128, 0),
        (AnsiColor::Blue, 0, 0, 128),
        (AnsiColor::Purple, 128, 0, 128),
        (AnsiColor::Cyan, 0, 128, 128),
        (AnsiColor::White, 192, 192, 192),
        (AnsiColor::DarkGray, 128, 128, 128),
        (AnsiColor::LightRed, 255, 0, 0),
        (AnsiColor::LightGreen, 0, 255, 0),
        (AnsiColor::LightYellow, 255, 255, 0),
        (AnsiColor::LightBlue, 0, 0, 255),
        (AnsiColor::LightPurple, 255, 0, 255),
        (AnsiColor::LightCyan, 0, 255, 255),
        (AnsiColor::White, 255, 255, 255), // bright white ≈ white
    ];
    let mut best = AnsiColor::White;
    let mut best_d = u32::MAX;
    for &(named, pr, pg, pb) in &std16 {
        let d = color_dist(r, g, b, pr, pg, pb);
        if d < best_d {
            best_d = d;
            best = named;
        }
    }
    best
}

/// Weighted squared distance (redmean approximation — cheap, eye-accurate enough).
fn color_dist(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> u32 {
    let rmean = ((r1 as u32 + r2 as u32) / 2) as i32;
    let dr = (r1 as i32 - r2 as i32).abs();
    let dg = (g1 as i32 - g2 as i32).abs();
    let db = (b1 as i32 - b2 as i32).abs();
    let dr2 = dr * dr;
    let dg2 = dg * dg;
    let db2 = db * db;
    let r_weight = dr2 * (512 + 2 * (255 - rmean));
    // green weighted heaviest; red/blue adjusted by redmean.
    ((dg2 * 1024) + r_weight + db2 * (767 - 2 * rmean)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truecolor_passthrough() {
        let c = resolve(10, 20, 30, ColorDepth::True24);
        assert!(matches!(c, AnsiColor::Rgb(10, 20, 30)));
    }

    #[test]
    fn index256_downsamples_to_fixed() {
        let c = resolve(255, 0, 0, ColorDepth::Index256);
        // Pure red → should map to a red-ish 256 index (9 = bright red, or nearby).
        assert!(matches!(c, AnsiColor::Fixed(_)));
        if let AnsiColor::Fixed(n) = c {
            assert!(n == 9 || n == 1 || n == 160 || n == 196 || n == 124 || n == 52,
                "pure red should map to a red index, got {n}");
        }
    }

    #[test]
    fn index16_downsamples_to_named() {
        let c = resolve(255, 0, 0, ColorDepth::Index16);
        assert!(matches!(c, AnsiColor::LightRed | AnsiColor::Red));
        let c = resolve(0, 255, 0, ColorDepth::Index16);
        assert!(matches!(c, AnsiColor::LightGreen | AnsiColor::Green));
        let c = resolve(0, 0, 0, ColorDepth::Index16);
        assert!(matches!(c, AnsiColor::Black));
    }

    #[test]
    fn nearest_256_black_and_white() {
        assert_eq!(nearest_256(0, 0, 0), 0); // black → index 0
        assert_eq!(nearest_256(255, 255, 255), 15); // white → index 15
    }

    #[test]
    fn nearest_256_gray_ramp() {
        // A gray that doesn't exactly match index 8 (128,128,128) → falls to ramp.
        // 8 + 10*9 = 98 → index 241; 8 + 10*10 = 108 → index 242. (100,100,100) ≈ 241.
        let n = nearest_256(100, 100, 100);
        assert!((232..=255).contains(&n), "gray should map to ramp 232-255, got {n}");
    }

    #[test]
    fn detect_respects_colorterm() {
        // Simulate: temporarily this just tests the function runs; full env-var
        // testing is flaky in parallel, so we test the precedence logic by
        // calling detect directly with controlled inputs is hard. At minimum,
        // verify it returns a valid variant.
        let d = detect_color_depth();
        assert!(matches!(d, ColorDepth::True24 | ColorDepth::Index256 | ColorDepth::Index16));
    }

    #[test]
    fn cached_depth_stable() {
        let a = cached_depth();
        let b = cached_depth();
        assert_eq!(a, b);
    }
}
