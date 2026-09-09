// Theme state + semantic color resolution — backend-neutral.
//
// Extracted from iced_adapter (Plan 413/418 follow-up): `style/class.rs`
// needs dark-mode-aware semantic alpha blending even when only the
// `code-editor` feature is on (no iced backend compiled), so the pure
// theme logic lives here with zero iced dependencies. iced_adapter
// re-exports everything for call-site compatibility.

use super::Color;

/// PLAN-593（Design 29 Phase 1）：语义 token 值单一事实源——zinc/scaffold
/// （CSS 面）与 stella（VM 面）三套色板 + accent 表 + 封闭词表。
/// 实体在无门的 `crate::design_tokens`（ui_gen 消费方不受 feature="ui" 门，
/// 见该模块头注）；此处 re-export 保持 `theme::registry` 路径稳定。
pub use crate::design_tokens::registry;

// PLAN-601 T-04：活动主题槽——命名主题热切换（VM 面）。epoch 失效回路
// 与 dark_mode 共用（THEME_EPOCH）；mode 仍由 DARK_MODE 承载（主题对内
// light/dark 选择器，`dark:` 门控经 dark_mode() 读值不变=零回归泛化）。
thread_local! {
    static ACTIVE_THEME: std::cell::RefCell<String> =
        std::cell::RefCell::new("stella".to_string());
}

/// 当前活动主题名（内置名；缺省 "stella" = VM 轨现行）。
pub fn theme_name() -> String {
    ACTIVE_THEME.with(|t| t.borrow().clone())
}

/// 切换活动主题（内置名校验；未知名返回 false 且零变化）。变化时
/// THEME_EPOCH 自增——既有失效回路（view 重建→重解析）随之生效。
pub fn set_theme(name: &str) -> bool {
    if registry::builtin(name).is_none() {
        return false;
    }
    let changed = ACTIVE_THEME.with(|t| {
        let mut t = t.borrow_mut();
        if t.as_str() != name {
            *t = name.to_string();
            true
        } else {
            false
        }
    });
    if changed {
        THEME_EPOCH.with(|e| e.set(e.get().wrapping_add(1)));
    }
    changed
}

/// 活动主题 spec（内置表查得；槽值恒为已校验内置名）。
fn active_theme() -> &'static registry::ThemeSpec {
    let name = theme_name();
    registry::builtin(&name).unwrap_or(registry::builtin("stella").expect("stella 恒在"))
}

// Plan 370 D-GAP-2/D-GAP-5: thread-local theme state for dark mode + accent.
// Set by the renderer before each render pass from VmBridge state.
// Plan 408: default to true because the iced window theme is hardcoded to
// Theme::Dark (renderer.rs ~line 4540). Apps that declare a `dark_mode`
// state var can override this; apps that don't (like widgets-gallery) get
// the correct dark palette by default, matching vue's <html class="dark">.
thread_local! {
    static DARK_MODE: std::cell::Cell<bool> = std::cell::Cell::new(true);
    static ACCENT_NAME: std::cell::RefCell<String> = std::cell::RefCell::new("indigo".to_string());
    /// PLAN-053 T12（051-候选修复，转介单①收回自修）：主题代数——
    /// `dark_mode` 每次值变化自增。内容寻址视图缓存
    /// （autodown_render::StreamCache 的结构键无主题维度）构建期记录
    /// 代数、与本值不符即全量重建——否则翻转帧 clone 旧块，fence 静态
    /// 档滞留构建时主题（首帧 D-GAP 同步前取档与翻转不重建双根因）。
    static THEME_EPOCH: std::cell::Cell<u32> = std::cell::Cell::new(0);
}

/// Set the global dark mode flag (called by renderer before rendering).
pub fn set_dark_mode(dark: bool) {
    let changed = DARK_MODE.with(|d| d.replace(dark)) != dark;
    if changed {
        THEME_EPOCH.with(|e| e.set(e.get().wrapping_add(1)));
    }
}

/// Read the global dark mode flag (Plan 413: code editor theme bridge).
pub fn dark_mode() -> bool {
    DARK_MODE.with(|d| d.get())
}

/// Current theme epoch: bump-on-change counter over `dark_mode` writes.
/// Content-keyed view caches compare their build epoch against this to
/// invalidate theme-dependent chrome (StreamCache 消费).
pub fn theme_epoch() -> u32 {
    THEME_EPOCH.with(|e| e.get())
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

/// Accent palette —— PLAN-593 后单源在 registry（vue TS ACCENT_PALETTES 值
/// 与 code_editor 派生同源互锁）；本地 `accent_hsl` 副本已删。

// Plan 458: theme preference + accent preset single source. The CLI
// (`auto run --theme/--accent`), pac.at parsing, VM env injection and the
// vue index.html generator all validate against / read from here.
pub const THEME_PREFS: [&str; 2] = ["dark", "light"];
/// PLAN-593：预设名单值源自 registry（单一事实源）。
pub const ACCENT_PRESETS: [&str; 5] = registry::ACCENT_NAMES;

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
    let (h, s, l) = registry::accent_hsl(name)?;
    let l = if dark { (l + 10).min(85) } else { l };
    Some(format!("{} {}% {}%", h, s, l))
}

/// Same as `accent_primary_hsl` but as an RGB tuple for native renderers
/// (iced window palette). Falls back to the caller on None (unknown name).
pub fn accent_primary_rgb(name: &str, dark: bool) -> Option<(u8, u8, u8)> {
    let (h, s, l) = registry::accent_hsl(name)?;
    let l = if dark { (l + 10).min(85) } else { l };
    Some(hsl_to_rgb(h, s, l))
}

// Plan 527 T5: font-sans/serif/mono 字体栈契约 —— Tailwind fontFamily 默认栈的
// 跨平台族名表。iced 端按 generic Family(SansSerif/Serif/Monospace) 交给
// cosmic-text 平台解析,此表是「栈语义」的成文契约与未来自定义字体回退链
// (fontFamily.config) 的锚点;docs/style-coverage.md 文本族行引用。
pub fn font_stack(kind: &str) -> &'static [&'static str] {
    match kind {
        "sans" => &[
            "ui-sans-serif", "system-ui", "-apple-system", "Segoe UI", "Roboto",
            "Helvetica Neue", "Arial", "Noto Sans", "sans-serif",
        ],
        "serif" => &[
            "ui-serif", "Georgia", "Cambria", "Times New Roman", "Times",
            "Noto Serif", "serif",
        ],
        "mono" => &[
            "ui-monospace", "SFMono-Regular", "Cascadia Mono", "Consolas",
            "Menlo", "Monaco", "DejaVu Sans Mono", "monospace",
        ],
        _ => &[],
    }
}

/// PLAN-593（Design 29 §5.4）：`Color` 枚举（VM 侧词表子集）→ registry
/// TokenName 的投影契约——card/popover/surface 三键在 VM 投影收敛为 Card
/// （词表全集以 registry 为准）。投影完备性由 plan593 T-c 测试钉死。
fn color_token(color: &Color) -> Option<registry::TokenName> {
    use registry::TokenName as T;
    Some(match color {
        Color::Secondary => T::Secondary,
        Color::Background => T::Background,
        Color::Surface => T::Card,
        Color::Muted => T::Muted,
        Color::Error => T::Error,
        Color::Warning => T::Warning,
        Color::Success => T::Success,
        Color::Info => T::Info,
        Color::OnPrimary => T::PrimaryForeground,
        Color::OnSecondary => T::SecondaryForeground,
        Color::OnDestructive => T::DestructiveForeground,
        Color::OnBackground => T::Foreground,
        Color::OnSurface => T::MutedForeground,
        Color::Border => T::Border,
        // Primary 保持运行时 accent 驱动（accent 降维为主题覆盖层属 Phase 2）；
        // 调色板/字面量色不在语义域。
        _ => return None,
    })
}

/// Resolve a semantic color to RGB, considering dark mode and accent.
/// PLAN-593 V2：静态语义色查 registry（stella 单源），零字面 RGB 臂。
pub fn resolve_semantic_rgb(color: &Color) -> Option<(u8, u8, u8)> {
    let is_dark = DARK_MODE.with(|d| d.get());
    if let Color::Primary = color {
        // Accent-driven: look up current accent name
        let name = ACCENT_NAME.with(|n| n.borrow().clone());
        let (h, s, l) = registry::accent_hsl(&name).unwrap_or(registry::ACCENT_DEFAULT);
        // Dark mode: align to vue index.css .dark `--primary: 239 84% 77%` (L=77%)
        let l_adjusted = if is_dark { (l + 10).min(85) } else { l };
        return Some(hsl_to_rgb(h, s, l_adjusted));
    }
    let token = color_token(color)?;
    registry::resolve_rgb(active_theme(), token, is_dark)
}

/// Plan 411 P2-A④: vue `border-border` 语义色(shadcn --border 变量)——
/// PLAN-593 V2：值改查 registry（stella 表），零字面 RGB。
/// （历史校准记录：Plan 518 暖灰 light #e3ddd1 / 蓝黑 dark #283146。）
pub fn resolve_border_rgb() -> (u8, u8, u8) {
    let is_dark = DARK_MODE.with(|d| d.get());
    registry::resolve_rgb(active_theme(), registry::TokenName::Border, is_dark)
        .expect("活动主题 Border 槽由 plan601 themes_core_complete 钉死")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(color: Color) -> (u8, u8, u8) {
        resolve_semantic_rgb(&color).expect("semantic color must resolve")
    }

    /// Plan 518 T1: 双主题语义值表——light 暖纸 / dark 精修蓝黑(stella 对齐)。
    /// PLAN-601 T-04：活动主题热切换——resolve 全翻转 + epoch 自增 +
    /// 未知名拒绝 + 还原防污染。
    #[test]
    fn theme_switch_flips_resolution_and_bumps_epoch() {
        assert_eq!(super::theme_name(), "stella");
        super::set_dark_mode(false); // 先定 mode，再取 epoch 基线（dark 翻转也自增）
        let e0 = super::theme_epoch();
        assert!(!super::set_theme("nonsense"), "未知名拒绝");
        assert_eq!(super::theme_name(), "stella");
        assert_eq!(super::theme_epoch(), e0);
        assert!(super::set_theme("zinc"));
        assert_eq!(super::theme_epoch(), e0.wrapping_add(1));
        super::set_dark_mode(false);
        assert_eq!(
            super::resolve_semantic_rgb(&Color::Background),
            Some((255, 255, 255)),
            "zinc light background = 纯白"
        );
        assert!(super::set_theme("stella"));
        assert_eq!(
            super::resolve_semantic_rgb(&Color::Background),
            Some((245, 241, 232)),
            "stella 暖纸恢复"
        );
        assert_eq!(super::theme_epoch(), e0.wrapping_add(2), "两次主题切换各 +1（无 dark 翻转）");
        super::set_dark_mode(true); // 还原默认档，防污染其他用例
    }

    #[test]
    fn stella_light_palette() {
        set_dark_mode(false);
        assert_eq!(rgb(Color::Background), (245, 241, 232)); // #f5f1e8 暖纸
        assert_eq!(rgb(Color::Surface), (251, 248, 242)); // #fbf8f2 卡片微浮
        assert_eq!(rgb(Color::OnBackground), (42, 39, 35)); // 墨色 #2a2723
        assert_eq!(rgb(Color::Border), (227, 221, 209)); // 暖灰 #e3ddd1
        assert_eq!(rgb(Color::Muted), (240, 235, 226)); // 暖 muted #f0ebe2
        // PLAN-571: secondary 分档（≠muted）——light 暖灰一档深 #e3ddd1。
        assert_eq!(rgb(Color::Secondary), (227, 221, 209));
        assert_eq!(rgb(Color::OnSurface), (125, 119, 109)); // 暖次级文本 #7d776d
        assert_eq!(resolve_border_rgb(), (227, 221, 209));
    }

    #[test]
    fn stella_dark_palette() {
        set_dark_mode(true);
        assert_eq!(rgb(Color::Background), (20, 26, 41)); // #141a29 深蓝黑
        assert_eq!(rgb(Color::Surface), (26, 34, 53)); // #1a2235 面板
        assert_eq!(rgb(Color::Border), (40, 49, 70)); // 低对比 #283146
        assert_eq!(resolve_border_rgb(), (40, 49, 70));
        // PLAN-571: secondary 分档（≠muted）——dark slate-700 #334155。
        assert_eq!(rgb(Color::Secondary), (51, 65, 85));
        assert_ne!(rgb(Color::Secondary), rgb(Color::Muted));
    }

    /// 待澄清②裁定:stella 玫瑰粉 = 既有 coral 预设(503 已校准 light
    /// hsl(4,43%,59%) ≈ #c4706a,权威图实测 #C96B62),不新增 rose 预设。
    #[test]
    fn coral_matches_stella_rose_accent() {
        set_dark_mode(false);
        set_accent_name("coral");
        assert_eq!(rgb(Color::Primary), (195, 111, 105)); // hsl(4,43%,59%) 截断值
        set_dark_mode(true);
        assert_eq!(rgb(Color::Primary), (209, 146, 141)); // dark L+10 → 69%
        set_accent_name("indigo"); // 还原默认,防污染其他用例
    }

    #[test]
    fn border_resolver_consistent_with_border_token() {
        for dark in [false, true] {
            set_dark_mode(dark);
            assert_eq!(resolve_border_rgb(), rgb(Color::Border));
        }
        set_dark_mode(true); // 还原默认
    }

    /// Plan 527 T5: 字体栈契约 —— 三栈齐备且平台主流族名在册。
    #[test]
    fn font_stacks_cover_three_families() {
        for kind in ["sans", "serif", "mono"] {
            let stack = font_stack(kind);
            assert!(!stack.is_empty(), "{kind} 栈非空");
            assert!(
                stack.iter().any(|f| f.to_ascii_lowercase().contains(kind))
                    || stack.contains(&"Segoe UI")
                    || stack.contains(&"Georgia")
                    || stack.contains(&"Consolas"),
                "{kind} 栈应含 generic 兜底或平台主流族名"
            );
        }
        assert!(font_stack("unknown").is_empty(), "未知族返回空栈");
    }
}
