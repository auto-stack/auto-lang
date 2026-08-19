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
    pub current_line: Rgba,
    pub gutter_background: Rgba,
    pub gutter_foreground: Rgba,
    pub scrollbar: Rgba,
    pub scrollbar_active: Rgba,
    pub syntax: SyntaxPalette,
}

/// Accent palettes (aligned with `ui/style/iced_adapter.rs` ACCENT_PALETTES,
/// duplicated here so the core layer stays independent of the style module's
/// iced-gated parts).
fn accent_hsl(name: &str) -> (u16, u8, u8) {
    match name {
        "coral" => (350, 75, 64),
        "ocean" => (217, 91, 60),
        "sage" => (160, 84, 45),
        "amber" => (38, 92, 55),
        _ => (239, 84, 70), // indigo
    }
}

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
    /// Dark preset (the AutoUI iced window is hardcoded `Theme::Dark`,
    /// renderer.rs ~4540) with the given accent name.
    pub fn dark(accent: &str) -> Self {
        let (ar, ag, ab) = {
            let (r, g, b) = hsl_to_rgb(accent_hsl(accent).0, accent_hsl(accent).1, 62);
            (r, g, b)
        };
        let bg = Rgba::rgb(0.11, 0.115, 0.14);
        let fg = Rgba::rgb(0.86, 0.87, 0.9);
        let muted = fg.mix(bg, 0.45);
        Self {
            background: bg,
            foreground: fg,
            caret: Rgba::rgb(ar, ag, ab),
            selection: Rgba::new(ar, ag, ab, 0.30),
            current_line: fg.mix(bg, 0.96),
            gutter_background: bg.mix(Rgba::BLACK, 0.35),
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

    /// Light preset with the given accent name.
    pub fn light(accent: &str) -> Self {
        let (ar, ag, ab) = {
            let (h, s, _) = accent_hsl(accent);
            let (r, g, b) = hsl_to_rgb(h, s, 42);
            (r, g, b)
        };
        let bg = Rgba::rgb(0.985, 0.985, 0.99);
        let fg = Rgba::rgb(0.12, 0.13, 0.16);
        let muted = fg.mix(bg, 0.45);
        Self {
            background: bg,
            foreground: fg,
            caret: Rgba::rgb(ar, ag, ab),
            selection: Rgba::new(ar, ag, ab, 0.22),
            current_line: fg.mix(bg, 0.955),
            gutter_background: bg.mix(Rgba::BLACK, 0.05),
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

        let italic = Some(FontStyle::ITALIC);
        let scopes = [
            scope(&["keyword", "storage"], self.syntax.keyword, None),
            scope(
                &["string", "string.regexp"],
                self.syntax.string,
                None,
            ),
            scope(&["comment"], self.syntax.comment, italic),
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

/// Resolve the current theme from the semantic source.
pub fn current_theme() -> CodeEditorTheme {
    let (dark, accent) = theme_source();
    if dark {
        CodeEditorTheme::dark(&accent)
    } else {
        CodeEditorTheme::light(&accent)
    }
}

/// Read the semantic source (dark flag + accent name).
pub fn theme_source() -> (bool, String) {
    let dark = THEME_DARK.with(|d| d.get());
    let accent = THEME_ACCENT.with(|a| a.borrow().clone());
    (dark, accent)
}
