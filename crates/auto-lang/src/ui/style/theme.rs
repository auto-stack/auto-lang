// Theme state + semantic color resolution — backend-neutral.
//
// Extracted from iced_adapter (Plan 413/418 follow-up): `style/class.rs`
// needs dark-mode-aware semantic alpha blending even when only the
// `code-editor` feature is on (no iced backend compiled), so the pure
// theme logic lives here with zero iced dependencies. iced_adapter
// re-exports everything for call-site compatibility.

use super::Color;

// Plan 370 D-GAP-2/D-GAP-5: thread-local theme state for dark mode + accent.
// Set by the renderer before each render pass from VmBridge state.
// Plan 408: default to true because the iced window theme is hardcoded to
// Theme::Dark (renderer.rs ~line 4540). Apps that declare a `dark_mode`
// state var can override this; apps that don't (like widgets-gallery) get
// the correct dark palette by default, matching vue's <html class="dark">.
thread_local! {
    static DARK_MODE: std::cell::Cell<bool> = std::cell::Cell::new(true);
    static ACCENT_NAME: std::cell::RefCell<String> = std::cell::RefCell::new("indigo".to_string());
}

/// Set the global dark mode flag (called by renderer before rendering).
pub fn set_dark_mode(dark: bool) {
    DARK_MODE.with(|d| d.set(dark));
}

/// Read the global dark mode flag (Plan 413: code editor theme bridge).
pub fn dark_mode() -> bool {
    DARK_MODE.with(|d| d.get())
}

/// Read the current accent name (Plan 413: code editor theme bridge).
pub fn accent_name() -> String {
    ACCENT_NAME.with(|n| n.borrow().clone())
}

/// Set the global accent color name (called by renderer before rendering).
pub fn set_accent_name(name: &str) {
    ACCENT_NAME.with(|n| *n.borrow_mut() = name.to_string());
}

// Plan 409 §10 续 11: 窗口宽度,供 VM builder 做响应式布局(如 category-section
// 的 grid 列数)。renderer 在 view() 前设值(同 set_dark_mode);window_resized
// 时 mark view_dirty 触发重建,让列数随窗口宽度更新。
thread_local! {
    static WINDOW_WIDTH: std::cell::Cell<f32> = std::cell::Cell::new(1024.0);
}
/// Set the current window width (called by renderer before rendering).
pub fn set_window_width(w: f32) {
    WINDOW_WIDTH.with(|c| c.set(w));
}
/// Read the current window width (for responsive layout in view builder).
pub fn window_width() -> f32 {
    WINDOW_WIDTH.with(|c| c.get())
}

/// HSL → RGB conversion (for accent palettes).
fn hsl_to_rgb(h: u16, s: u8, l: u8) -> (u8, u8, u8) {
    let h = h as f64 / 360.0;
    let s = s as f64 / 100.0;
    let l = l as f64 / 100.0;
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue_to_rgb = |p: f64, q: f64, mut t: f64| -> f64 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
        if t < 1.0 / 2.0 { return q; }
        if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
        p
    };
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Accent palette (HSL triplets, aligned with Vue ACCENT_PALETTES + auto-forge).
fn accent_hsl(name: &str) -> Option<(u16, u8, u8)> {
    match name {
        "indigo" => Some((239, 84, 67)),
        "coral"  => Some((350, 75, 64)),
        "ocean"  => Some((217, 91, 60)),
        "sage"   => Some((160, 84, 39)),
        "amber"  => Some((38, 92, 50)),
        _ => None,
    }
}

/// Resolve a semantic color to RGB, considering dark mode and accent.
pub fn resolve_semantic_rgb(color: &Color) -> Option<(u8, u8, u8)> {
    let is_dark = DARK_MODE.with(|d| d.get());
    match color {
        Color::Primary => {
            // Accent-driven: look up current accent name
            let name = ACCENT_NAME.with(|n| n.borrow().clone());
            let (h, s, l) = accent_hsl(&name).unwrap_or((239, 84, 67));
            // Dark mode: boost lightness +4% for contrast
            let l_adjusted = if is_dark { (l + 4).min(85) } else { l };
            Some(hsl_to_rgb(h, s, l_adjusted))
        }
        Color::Secondary => {
            if is_dark { Some((129, 140, 248)) } else { Some((99, 102, 241)) } // indigo-400/500
        }
        Color::Background => {
            if is_dark { Some((17, 24, 39)) } else { Some((255, 255, 255)) } // gray-900/white
        }
        Color::Surface => {
            if is_dark { Some((31, 41, 55)) } else { Some((249, 250, 251)) } // gray-800/50
        }
        Color::Error => Some((239, 68, 68)),
        Color::Warning => Some((234, 179, 8)),
        Color::Success => Some((34, 197, 94)),
        Color::Info => Some((59, 130, 246)),
        Color::OnPrimary | Color::OnSecondary => Some((255, 255, 255)),
        Color::OnBackground => {
            if is_dark { Some((229, 231, 235)) } else { Some((17, 24, 39)) } // gray-200/900
        }
        Color::OnSurface => {
            if is_dark { Some((156, 163, 175)) } else { Some((107, 114, 128)) } // gray-400/500
        }
        _ => None,
    }
}

/// Plan 411 P2-A④: vue `border-border` 语义色(shadcn --border 变量)——
/// light `hsl(240 5.9% 90%)` ≈ zinc-200,dark `hsl(240 3.7% 15.9%)` ≈ zinc-800。
/// 表格行分隔线等单侧描边使用。
pub fn resolve_border_rgb() -> (u8, u8, u8) {
    let is_dark = DARK_MODE.with(|d| d.get());
    if is_dark { (0x27, 0x27, 0x2a) } else { (0xe4, 0xe4, 0xe7) }
}
