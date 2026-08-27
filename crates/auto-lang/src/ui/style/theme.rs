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

// Plan 458: theme preference + accent preset single source. The CLI
// (`auto run --theme/--accent`), pac.at parsing, VM env injection and the
// vue index.html generator all validate against / read from here.
pub const THEME_PREFS: [&str; 2] = ["dark", "light"];
pub const ACCENT_PRESETS: [&str; 5] = ["indigo", "coral", "ocean", "sage", "amber"];

/// Effective theme preference injected by `auto run` (AUTO_UI_THEME env,
/// validated), or the built-in default "dark". Read by the vue/tauri
/// index.html generators so all scaffolding paths agree on the theme.
pub fn theme_pref_from_env() -> &'static str {
    match std::env::var("AUTO_UI_THEME").as_deref() {
        Ok(t) if THEME_PREFS.contains(&t) => match t {
            "light" => "light",
            _ => "dark",
        },
        _ => "dark",
    }
}

/// Effective accent preset injected by `auto run` (AUTO_UI_ACCENT env,
/// validated), or None (generators keep their stylesheet default = indigo).
pub fn accent_pref_from_env() -> Option<&'static str> {
    match std::env::var("AUTO_UI_ACCENT").as_deref() {
        Ok(a) if ACCENT_PRESETS.contains(&a) => Some(match a {
            "coral" => "coral",
            "ocean" => "ocean",
            "sage" => "sage",
            "amber" => "amber",
            _ => "indigo",
        }),
        _ => None,
    }
}

/// `--primary` shadcn token as an HSL triplet string ("H S% L%") for the
/// given accent preset + theme, or None for unknown preset names. Dark mode
/// gets the same L-boost as `resolve_semantic_rgb` (aligning to the
/// generated index.css `.dark --primary`). Consumers: vue index.html inline
/// bootstrap (Plan 458), code editors, etc.
pub fn accent_primary_hsl(name: &str, dark: bool) -> Option<String> {
    let (h, s, l) = accent_hsl(name)?;
    let l = if dark { (l + 10).min(85) } else { l };
    Some(format!("{} {}% {}%", h, s, l))
}

/// Same as `accent_primary_hsl` but as an RGB tuple for native renderers
/// (iced window palette). Falls back to the caller on None (unknown name).
pub fn accent_primary_rgb(name: &str, dark: bool) -> Option<(u8, u8, u8)> {
    let (h, s, l) = accent_hsl(name)?;
    let l = if dark { (l + 10).min(85) } else { l };
    Some(hsl_to_rgb(h, s, l))
}

/// Resolve a semantic color to RGB, considering dark mode and accent.
pub fn resolve_semantic_rgb(color: &Color) -> Option<(u8, u8, u8)> {
    let is_dark = DARK_MODE.with(|d| d.get());
    match color {
        Color::Primary => {
            // Accent-driven: look up current accent name
            let name = ACCENT_NAME.with(|n| n.borrow().clone());
            let (h, s, l) = accent_hsl(&name).unwrap_or((239, 84, 67));
            // Dark mode: align to vue index.css .dark `--primary: 239 84% 77%` (L=77%)
            let l_adjusted = if is_dark { (l + 10).min(85) } else { l };
            Some(hsl_to_rgb(h, s, l_adjusted))
        }
        Color::Secondary => {
            if is_dark { Some((30, 41, 59)) } else { Some((241, 245, 249)) }
        }
        // Plan 448 对齐批:暗色语义色从 Tailwind gray 系改为生成端 shadcn
        // 令牌(index.css .dark)的精确 HSL 换算值 —— 此前 VM 用 gray-900/800
        // 近似,与 vue 产物的蓝灰调(明暗关系相反:vue 卡片亮于页面)不一致。
        // --background: 222.2 47.4% 7% / --card: 222.2 47.4% 10%
        Color::Background => {
            if is_dark { Some((9, 14, 26)) } else { Some((255, 255, 255)) }
        }
        // --card: 222.2 47.4% 10%(亮于 --background,与 vue 暗色卡片浮起方向一致)
        Color::Surface => {
            if is_dark { Some((13, 21, 38)) } else { Some((249, 250, 251)) } // gray-50(light 沿用)
        }
        Color::Error => Some((239, 68, 68)),
        Color::Warning => Some((234, 179, 8)),
        Color::Success => Some((34, 197, 94)),
        Color::Info => Some((59, 130, 246)),
        // Plan 455 对齐批: shadcn-vue 暗色反转主按钮前景色
        // light: --primary-foreground = 210 40% 98% (白字)
        // dark: --primary-foreground = 222.2 47.4% 11.2% (黑字 rgb(15, 23, 42))
        Color::OnPrimary => {
            if is_dark { Some((15, 23, 42)) } else { Some((248, 250, 252)) }
        }
        Color::OnSecondary => {
            if is_dark { Some((248, 250, 252)) } else { Some((15, 23, 42)) }
        }
        Color::OnDestructive => Some((248, 250, 252)),
        // --foreground: 210 40% 98%
        Color::OnBackground => {
            if is_dark { Some((248, 250, 252)) } else { Some((17, 24, 39)) }
        }
        // --muted-foreground: 215.4 16.3% 65.1%
        Color::OnSurface => {
            if is_dark { Some((151, 163, 181)) } else { Some((107, 114, 128)) }
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
