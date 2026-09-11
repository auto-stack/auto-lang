// Plan 413 §3.1 ② theme layer — replaces cosmic::Theme for the editor.
//
// Colors come from AutoUI semantic state (dark mode + accent), not from
// cosmic-theme. `syntax_theme()` synthesizes a syntect theme from the
// palette so cosmic-text's SyntaxEditor can highlight with it.
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

use cosmic_text::SyntaxTheme;
use syntect::highlighting::{
    Color as SynColor, FontStyle, ScopeSelectors, StyleModifier, Theme as SynTheme,
    ThemeItem, ThemeSettings,
};

/// Backend-agnostic RGBA color (0.0–1.0 components).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const WHITE: Rgba = Rgba::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Rgba = Rgba::rgb(0.0, 0.0, 0.0);
    pub const TRANSPARENT: Rgba = Rgba::new(0.0, 0.0, 0.0, 0.0);

    fn mix(self, other: Rgba, t: f32) -> Rgba {
        Rgba::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    fn syn_color(self) -> SynColor {
        SynColor {
            r: (self.r * 255.0).round().clamp(0.0, 255.0) as u8,
            g: (self.g * 255.0).round().clamp(0.0, 255.0) as u8,
            b: (self.b * 255.0).round().clamp(0.0, 255.0) as u8,
            a: (self.a * 255.0).round().clamp(0.0, 255.0) as u8,
        }
    }
}

/// Syntax palette driving the synthesized syntect theme.
#[derive(Debug, Clone, Copy)]
pub struct SyntaxPalette {
    pub keyword: Rgba,
    pub string: Rgba,
    pub comment: Rgba,
    pub function: Rgba,
    pub number: Rgba,
    pub type_: Rgba,
    pub constant: Rgba,
    pub variable: Rgba,
    pub punctuation: Rgba,
}

/// Complete visual specification for one code editor instance.
#[derive(Debug, Clone)]
pub struct CodeEditorTheme {
    pub background: Rgba,
    pub foreground: Rgba,
    pub caret: Rgba,
    pub selection: Rgba,
    /// Search match highlight (drawn over selection, under text).
    pub search_match: Rgba,
    pub current_line: Rgba,
    pub gutter_background: Rgba,
    pub gutter_foreground: Rgba,
    pub scrollbar: Rgba,
    pub scrollbar_active: Rgba,
    pub syntax: SyntaxPalette,
}

/// Accent palette —— PLAN-593 后单源在 `ui::style::theme::registry`
/// （vue 脚手架 TS ACCENT_PALETTES / VM resolve_semantic_rgb Primary 臂同源）。
/// 历史注：本地表 L 分量曾与主表漂移（indigo 70↔67 / sage 45↔39 /
/// amber 55↔50），但编辑器只消费 H/S（dark 定 L=62 / light 定 L=42），
/// H/S 两表恒一致，故归一为输出中性；原注释指向的 iced_adapter
/// ACCENT_PALETTES 已不存在（失锚顺修，S1 对账 E6）。
fn accent_hsl(name: &str) -> (u16, u8, u8) {
    crate::design_tokens::registry::accent_hsl(name)
        .unwrap_or(crate::design_tokens::registry::ACCENT_DEFAULT)
}

/// f32 版 HSL→RGB（编辑器 0.0–1.0 色域直算）。与 ui::style::theme 的 u8 版
/// 精度域不同（那边逐通道取整 u8），合流会引入取整差——保留本地纯数学实现，
/// 单源约束只针对值表（accent_hsl），不针对无值的转换函数。

fn hsl_to_rgb(h: u16, s: u8, l: u8) -> (f32, f32, f32) {
    let h = h as f32 / 360.0;
    let s = s as f32 / 100.0;
    let l = l as f32 / 100.0;
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue = |mut t: f32| -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    (hue(h + 1.0 / 3.0), hue(h), hue(h - 1.0 / 3.0))
}

impl CodeEditorTheme {
    /// Legacy dark preset (pre-T-10 hardcoded bg/fg). Kept as the fallback
    /// palette when the active theme has no Background/Foreground values.
    pub fn dark(accent: &str) -> Self {
        let (ar, ag, ab) = {
            let (r, g, b) = hsl_to_rgb(accent_hsl(accent).0, accent_hsl(accent).1, 62);
            (r, g, b)
        };
        let bg = Rgba::rgb(0.11, 0.115, 0.14);
        let fg = Rgba::rgb(0.86, 0.87, 0.9);
        dark_body(ar, ag, ab, bg, fg)
    }

    /// Legacy light preset (pre-T-10 hardcoded bg/fg).
    pub fn light(accent: &str) -> Self {
        let (ar, ag, ab) = {
            let (h, s, _) = accent_hsl(accent);
            let (r, g, b) = hsl_to_rgb(h, s, 42);
            (r, g, b)
        };
        let bg = Rgba::rgb(0.985, 0.985, 0.99);
        let fg = Rgba::rgb(0.12, 0.13, 0.16);
        light_body(ar, ag, ab, bg, fg)
    }

    /// PLAN-601 T-10：从活动主题双面值派生（bg/fg 取 registry Background/
    /// Foreground 真值；accent 沿用 dark 定 L=62 / light 定 L=42 约定；其余
    /// 槽位由 bg/fg/accent 混合派生）。这就是 V4 的「编辑器色域映射」：
    /// 语义 token 域 (u8 RGB) → 编辑器域 (f32 0.0-1.0) 的唯一入口。
    pub fn from_resolved(dark: bool, accent: &str, bg_rgb: (u8, u8, u8), fg_rgb: (u8, u8, u8)) -> Self {
        let l = if dark { 62 } else { 42 };
        let (ar, ag, ab) = {
            let (r, g, b) = hsl_to_rgb(accent_hsl(accent).0, accent_hsl(accent).1, l);
            (r, g, b)
        };
        let to_f = |c: u8| c as f32 / 255.0;
        let bg = Rgba::rgb(to_f(bg_rgb.0), to_f(bg_rgb.1), to_f(bg_rgb.2));
        let fg = Rgba::rgb(to_f(fg_rgb.0), to_f(fg_rgb.1), to_f(fg_rgb.2));
        if dark {
            dark_body(ar, ag, ab, bg, fg)
        } else {
            light_body(ar, ag, ab, bg, fg)
        }
    }

    /// 内置主题面：从 registry ThemeSpec 取 Background/Foreground。
    pub fn for_builtin(spec: &crate::design_tokens::registry::ThemeSpec, dark: bool, accent: &str) -> Self {
        use crate::design_tokens::registry::{resolve_rgb, TokenName};
        let bg = resolve_rgb(spec, TokenName::Background, dark)
            .unwrap_or(if dark { (28, 29, 36) } else { (251, 251, 253) });
        let fg = resolve_rgb(spec, TokenName::Foreground, dark)
            .unwrap_or(if dark { (219, 222, 230) } else { (31, 33, 41) });
        Self::from_resolved(dark, accent, bg, fg)
    }

    /// 合成主题面：从 decl ComposedTheme 取 Background/Foreground。
    pub fn for_composed(spec: &crate::design_tokens::decl::ComposedTheme, dark: bool, accent: &str) -> Self {
        use crate::design_tokens::registry::TokenName;
        let bg = spec.resolve_rgb(TokenName::Background, dark)
            .unwrap_or(if dark { (28, 29, 36) } else { (251, 251, 253) });
        let fg = spec.resolve_rgb(TokenName::Foreground, dark)
            .unwrap_or(if dark { (219, 222, 230) } else { (31, 33, 41) });
        Self::from_resolved(dark, accent, bg, fg)
    }

    /// Synthesize a syntect theme carrying this palette. The theme is
    /// registered under `registered_name` by the caller (highlight.rs).
    pub fn syntax_theme(&self) -> SyntaxTheme {
        let scope = |selectors: &[&str], color: Rgba, style: Option<FontStyle>| {
            selectors
                .iter()
                .map(|sel| ThemeItem {
                    scope: sel
                        .parse::<ScopeSelectors>()
                        .unwrap_or_default(),
                    style: StyleModifier {
                        foreground: Some(color.syn_color()),
                        background: None,
                        font_style: style,
                    },
                })
                .collect::<Vec<_>>()
        };

        // Comments stay upright on purpose: cosmic-text filters font
        // candidates by exact face style, and no Windows CJK font ships an
        // italic face — Han in italic spans shapes to .notdef tofu
        // (glyph_id 0, verified: Consolas-Italic has no CJK fallback).
        let scopes = [
            scope(&["keyword", "storage"], self.syntax.keyword, None),
            scope(
                &["string", "string.regexp"],
                self.syntax.string,
                None,
            ),
            scope(&["comment"], self.syntax.comment, None),
            scope(
                &["entity.name.function", "support.function"],
                self.syntax.function,
                None,
            ),
            scope(&["constant.numeric"], self.syntax.number, None),
            scope(
                &[
                    "entity.name.type",
                    "entity.name.class",
                    "support.class",
                    "support.type",
                    "storage.type",
                ],
                self.syntax.type_,
                None,
            ),
            scope(
                &["constant", "constant.language"],
                self.syntax.constant,
                None,
            ),
            scope(
                &["variable", "variable.other"],
                self.syntax.variable,
                None,
            ),
            scope(
                &["punctuation", "meta.brace"],
                self.syntax.punctuation,
                None,
            ),
        ]
        .concat();

        SynTheme {
            name: Some("autoui-synthesized".to_owned()),
            author: None,
            settings: ThemeSettings {
                foreground: Some(self.foreground.syn_color()),
                background: Some(self.background.syn_color()),
                caret: Some(self.caret.syn_color()),
                line_highlight: Some(self.current_line.syn_color()),
                gutter: Some(self.gutter_background.syn_color()),
                gutter_foreground: Some(self.gutter_foreground.syn_color()),
                ..ThemeSettings::default()
            },
            scopes,
        }
    }
}

/// Dark 槽位派生体（bg/fg 入参化后 dark/light 两预设的剩余固定派生）。
fn dark_body(ar: f32, ag: f32, ab: f32, bg: Rgba, fg: Rgba) -> CodeEditorTheme {
    let muted = fg.mix(bg, 0.45);
    CodeEditorTheme {
        background: bg,
        foreground: fg,
        caret: Rgba::rgb(ar, ag, ab),
        selection: Rgba::new(ar, ag, ab, 0.30),
        search_match: Rgba::new(0.95, 0.8, 0.25, 0.30),
        current_line: fg.mix(bg, 0.96),
        // Plan 414 §5.3: gutter shares the editor background (Zed-style
        // seamless columns; the old darker wash broke the illusion).
        gutter_background: bg,
        gutter_foreground: muted,
        scrollbar: fg.mix(bg, 0.65),
        scrollbar_active: Rgba::rgb(ar, ag, ab).mix(bg, 0.2),
        syntax: SyntaxPalette {
            keyword: Rgba::rgb(ar * 0.75 + 0.25, ag, ab),
            string: Rgba::rgb(0.62, 0.8, 0.52),
            comment: muted,
            function: Rgba::rgb(0.55, 0.75, 0.95),
            number: Rgba::rgb(0.92, 0.7, 0.45),
            type_: Rgba::rgb(0.78, 0.62, 0.95),
            constant: Rgba::rgb(0.92, 0.7, 0.45),
            variable: fg,
            punctuation: fg.mix(bg, 0.25),
        },
    }
}

/// Light 槽位派生体。
fn light_body(ar: f32, ag: f32, ab: f32, bg: Rgba, fg: Rgba) -> CodeEditorTheme {
    let muted = fg.mix(bg, 0.45);
    CodeEditorTheme {
        background: bg,
        foreground: fg,
        caret: Rgba::rgb(ar, ag, ab),
        selection: Rgba::new(ar, ag, ab, 0.22),
        search_match: Rgba::new(0.98, 0.85, 0.3, 0.38),
        current_line: fg.mix(bg, 0.955),
        // Plan 414 §5.3: same as dark — gutter matches the editor bg.
        gutter_background: bg,
        gutter_foreground: muted,
        scrollbar: fg.mix(bg, 0.55),
        scrollbar_active: Rgba::rgb(ar, ag, ab).mix(bg, 0.25),
        syntax: SyntaxPalette {
            keyword: Rgba::rgb(ar, ag, ab),
            string: Rgba::rgb(0.2, 0.55, 0.25),
            comment: muted,
            function: Rgba::rgb(0.15, 0.35, 0.75),
            number: Rgba::rgb(0.7, 0.4, 0.05),
            type_: Rgba::rgb(0.5, 0.25, 0.7),
            constant: Rgba::rgb(0.7, 0.4, 0.05),
            variable: fg,
            punctuation: fg.mix(bg, 0.2),
        },
    }
}

/// PLAN-601 T-10：活动主题 → 编辑器色板（内置表/合成体统一；主题槽位缺值
/// 时回退 legacy 硬编码）。set_theme/set_theme_composed 换槽 + THEME_EPOCH
/// 失效 → 下帧 sync 重取 = 编辑器随主题翻转。
pub fn active_code_theme(dark: bool, accent: &str) -> CodeEditorTheme {
    use crate::design_tokens::registry::TokenName;
    let bg = crate::ui::style::theme::active_theme_rgb(TokenName::Background, dark);
    let fg = crate::ui::style::theme::active_theme_rgb(TokenName::Foreground, dark);
    match (bg, fg) {
        (Some(bg), Some(fg)) => CodeEditorTheme::from_resolved(dark, accent, bg, fg),
        _ => {
            if dark {
                CodeEditorTheme::dark(accent)
            } else {
                CodeEditorTheme::light(accent)
            }
        }
    }
}

// ─── theme source (set by the iced renderer each frame; core stays iced-free)

thread_local! {
    static THEME_DARK: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static THEME_ACCENT: std::cell::RefCell<String> =
        std::cell::RefCell::new("indigo".to_owned());
}

/// Update the semantic theme source (dark flag + accent name). The iced
/// renderer calls this before each render pass, mirroring
/// `style::iced_adapter::set_dark_mode` / `set_accent_name`.
pub fn set_theme_source(dark: bool, accent: &str) {
    THEME_DARK.with(|d| d.set(dark));
    THEME_ACCENT.with(|a| *a.borrow_mut() = accent.to_owned());
}

/// Resolve the current theme from the semantic source. PLAN-601 T-10:
/// bg/fg/caret/syntax derive from the ACTIVE theme slot (named theme or
/// composed declaration) — switching themes flips the editor via the
/// THEME_EPOCH invalidation loop.
pub fn current_theme() -> CodeEditorTheme {
    let (dark, accent) = theme_source();
    active_code_theme(dark, &accent)
}

/// Read the semantic source (dark flag + accent name).
pub fn theme_source() -> (bool, String) {
    let dark = THEME_DARK.with(|d| d.get());
    let accent = THEME_ACCENT.with(|a| a.borrow().clone());
    (dark, accent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntect::highlighting::FontStyle;

    /// PLAN-601 T-10: for_builtin maps the registry Background/Foreground
    /// truth into the editor's 0.0-1.0 domain (V4 derivation contract).
    #[test]
    fn for_builtin_derives_from_registry_values() {
        let stella = crate::design_tokens::registry::builtin("stella").unwrap();
        let bg = crate::design_tokens::registry::resolve_rgb(
            stella,
            crate::design_tokens::registry::TokenName::Background,
            true,
        )
        .unwrap();
        let t = CodeEditorTheme::for_builtin(stella, true, "indigo");
        assert_eq!(t.background.r, bg.0 as f32 / 255.0);
        assert_eq!(t.background.g, bg.1 as f32 / 255.0);
        assert_eq!(t.background.b, bg.2 as f32 / 255.0);
    }

    /// PLAN-601 T-10: the active resolution follows theme switches (the
    /// editor flip contract). Thread-local slot → per-test isolated.
    #[test]
    fn active_code_theme_follows_theme_switch() {
        let fg_of = |t: &CodeEditorTheme| {
            ((t.foreground.r * 255.0).round() as i32,
             (t.foreground.g * 255.0).round() as i32,
             (t.foreground.b * 255.0).round() as i32)
        };
        crate::ui::style::theme::set_theme("zinc");
        let zinc = active_code_theme(true, "indigo");
        crate::ui::style::theme::set_theme("stella");
        let stella = active_code_theme(true, "indigo");
        assert_ne!(fg_of(&zinc), fg_of(&stella), "切主题后编辑器前景须跟随翻转");
    }

    /// Comments must not request an italic face. cosmic-text filters font
    /// candidates by exact face style and no Windows CJK font ships an
    /// italic face, so Han in italic comment spans shapes to .notdef tofu
    /// (Plan 413 regression: 041's Chinese comments rendered as boxes).
    #[test]
    fn comment_scope_is_not_italic() {
        for theme in [CodeEditorTheme::dark("indigo"), CodeEditorTheme::light("indigo")] {
            let syn = theme.syntax_theme();
            let mut saw_comment_scope = false;
            for item in &syn.scopes {
                if format!("{:?}", item.scope).contains("comment") {
                    saw_comment_scope = true;
                    assert_ne!(
                        item.style.font_style,
                        Some(FontStyle::ITALIC),
                        "italic comments turn CJK into tofu on Windows"
                    );
                }
            }
            assert!(saw_comment_scope, "synthesized theme must style comments");
        }
    }
}
