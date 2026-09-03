// Plan 413 §3.1 ① highlight — syntax system singleton + theme registration.
//
// One leaked `SyntaxSystem` per process: two-face's full syntax set (with
// the AutoLang `.at` definition appended) + a theme set that starts from
// two-face and grows with synthesized AutoUI themes registered under
// stable names. Themes are inserted, never replaced, so the `&Theme`
// references SyntaxEditor holds stay valid.
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

use std::sync::{Mutex, OnceLock};

use cosmic_text::{FontSystem, SyntaxSystem};
use syntect::highlighting::Theme as SynTheme;

/// AutoLang grammar for `lang: "auto"` — keyword set mirrors the VM
/// renderer's AUTO_KEYWORDS (renderer.rs) so both surfaces agree.
const AUTO_SYNTAX_YAML: &str = r#"
%YAML 1.2
---
name: AutoLang
file_extensions: [at]
scope: source.auto
contexts:
  main:
    - match: \b(fn|let|var|const|if|else|for|loop|in|break|return|type|enum|use|pub|mut|static|true|false|is|Some|None|Ok|Err|match|where|col|row|text|button|input|container|scroll|checkbox|radio|select|slider|image|link|list|tab|tabs|sidebar|accordion|nav|textarea|progress|code_editor)\b
      scope: keyword.control.auto
    - match: \b[A-Z][A-Za-z0-9_]*\b
      scope: support.class.auto
    - match: \b[0-9]+(\.[0-9]+)?\b
      scope: constant.numeric.auto
    - match: '"([^"\\]|\\.)*"'
      scope: string.quoted.double.auto
    - match: '\b(f")(.*?)"'
      scope: string.quoted.double.auto
    - match: '//.*$'
      scope: comment.line.double-slash.auto
    - match: '#.*$'
      scope: comment.line.number-sign.auto
"#;

/// Map a DSL `lang` token to a syntect file extension.
pub fn lang_to_extension(lang: &str) -> Option<&'static str> {
    let ext = match lang.to_ascii_lowercase().as_str() {
        "auto" | "at" | "autolang" => "at",
        "rust" | "rs" => "rs",
        "python" | "py" => "py",
        "javascript" | "js" => "js",
        "typescript" | "ts" => "ts",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "markdown" | "md" => "md",
        "html" => "html",
        "css" => "css",
        "c" => "c",
        "cpp" | "c++" => "cpp",
        "go" => "go",
        "java" => "java",
        "shell" | "sh" | "bash" => "sh",
        "xml" => "xml",
        "sql" => "sql",
        "none" | "plain" | "plaintext" | "" => return None,
        other => {
            // Unknown language: try the token itself as an extension so new
            // languages work when two-face knows them.
            return Some(Box::leak(other.to_owned().into_boxed_str()));
        }
    };
    Some(ext)
}

struct SyntaxRegistry {
    system: &'static SyntaxSystem,
}

fn registry() -> &'static Mutex<SyntaxRegistry> {
    static REGISTRY: OnceLock<Mutex<SyntaxRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let system: &'static SyntaxSystem = Box::leak(Box::new(build_syntax_system()));
        Mutex::new(SyntaxRegistry { system })
    })
}

/// Fire-and-forget background warm-up: run one highlight pass per common
/// language so the first real use doesn't pay the regex compilation cost
/// (seconds with the onig backend in debug builds). `SyntaxSet` is
/// `Sync`; compiled-regex caches are shared, so work done here carries
/// over to the UI thread. Best effort — the thread races the first frames
/// and loses nothing if it loses the race.
fn spawn_syntax_warm_up(extensions: Vec<&'static str>) {
    std::thread::spawn(move || {
        let system = syntax_system();
        let Some(theme) = system.theme_set.themes.get("base16-eighties.dark") else {
            return;
        };
        for ext in extensions {
            let Some(syntax) = system.syntax_set.find_syntax_by_extension(ext) else {
                continue;
            };
            let mut hl = syntect::easy::HighlightLines::new(syntax, theme);
            let _ = hl.highlight_line(
                "fn warm(x: int) { let s = \"txt\"; } // comment
",
                &system.syntax_set,
            );
        }
    });
}

/// Warm the highlighter for one language in the background (called from
/// editor creation with the configured language).
pub fn warm_language(lang: &str) {
    let ext = lang_to_extension(lang).unwrap_or("txt");
    let ext: &'static str = Box::leak(ext.to_owned().into_boxed_str());
    spawn_syntax_warm_up(vec![ext]);
}

/// Known accent names (mirrors the AutoUI accent palettes). The theme set
/// is built once and immutable afterwards, so all synthesized themes are
/// pre-registered; unknown accents snap to the indigo palette.
pub(crate) const KNOWN_ACCENTS: [&str; 5] = ["indigo", "coral", "ocean", "sage", "amber"];

fn normalize_accent(accent: &str) -> &str {
    if KNOWN_ACCENTS.contains(&accent) {
        accent
    } else {
        "indigo"
    }
}

fn build_syntax_system() -> SyntaxSystem {
    let syntax_set = {
        let mut builder = two_face::syntax::extra_no_newlines().into_builder();
        if let Ok(auto_def) =
            syntect::parsing::SyntaxDefinition::load_from_str(AUTO_SYNTAX_YAML, false, None)
        {
            builder.add(auto_def);
        }
        builder.build()
    };
    let mut theme_set: syntect::highlighting::ThemeSet = {
        let lazy: two_face::theme::LazyThemeSet = two_face::theme::extra().into();
        lazy.into()
    };
    // Pre-register the synthesized AutoUI themes (dark/light × accents).
    for &accent in KNOWN_ACCENTS.iter() {
        for &dark in [true, false].iter() {
            let theme = if dark {
                crate::ui::code_editor::theme::CodeEditorTheme::dark(accent)
            } else {
                crate::ui::code_editor::theme::CodeEditorTheme::light(accent)
            };
            let name = theme_name_inner(dark, accent);
            theme_set.themes.insert(name, theme.syntax_theme());
        }
    }
    // PLAN-041 T4: autodown fence 家族 hljs 主题——autodown-core 的跨轨
    // token 映射表（scope→.hljs-* 类→色板，rust 单源）烘焙为 syntect 主题。
    // VM fence（编辑态 ViEditor / 只读态共享 buffer 实例）着色与 vue 侧
    // lowlight 观感对齐：tokenize 仍是 syntect，仅色板跨轨共享。
    #[cfg(feature = "autodown")]
    for &dark in [true, false].iter() {
        theme_set.themes.insert(hljs_theme_name_inner(dark), hljs_syntax_theme(dark));
    }
    SyntaxSystem {
        syntax_set,
        theme_set,
    }
}

fn theme_name_inner(dark: bool, accent: &str) -> String {
    format!("autoui-{}-{}", if dark { "dark" } else { "light" }, normalize_accent(accent))
}

/// The process-wide syntax system (leaked, shared by every editor).
pub fn syntax_system() -> &'static SyntaxSystem {
    registry().lock().unwrap().system
}

/// Resolve the pre-registered theme by name. All AutoUI themes are baked
/// into the (immutable, leaked) theme set at first use; unknown names fall
/// back to the dark indigo theme.
pub fn register_theme(name: &str, _theme: SynTheme) -> String {
    let reg = registry().lock().unwrap();
    if reg.system.theme_set.themes.contains_key(name) {
        name.to_owned()
    } else {
        theme_name_inner(true, "indigo")
    }
}

/// Stable theme name for a (dark, accent) pair — the pre-registered key.
pub fn theme_name(dark: bool, accent: &str) -> String {
    theme_name_inner(dark, accent)
}

// ── PLAN-041 T4：autodown fence 家族 hljs 主题（跨轨 token 映射表消费）──

#[cfg(feature = "autodown")]
fn hljs_theme_name_inner(dark: bool) -> String {
    format!("autodown-hljs-{}", if dark { "dark" } else { "light" })
}

/// autodown fence 家族主题名（暗/明两态预烘焙键）。
#[cfg(feature = "autodown")]
pub fn hljs_theme_name(dark: bool) -> String {
    hljs_theme_name_inner(dark)
}

/// 由映射表（scope→类→色板）构造 syntect 主题：每行 scope 一条
/// ThemeItem（theme.rs CodeEditorTheme::syntax_theme 同款模式）；
/// syntect 选择器的原子前缀 + 特异度排序天然实现
/// 「constant.character.escape（String 组）压过 constant（Number 组）」。
#[cfg(feature = "autodown")]
fn hljs_syntax_theme(dark: bool) -> SynTheme {
    use autodown_core::hljs_scope_map::{hljs_group_for_class, hljs_group_rgb, SCOPE_CLASS_TABLE};
    use syntect::highlighting::{
        Color, ScopeSelectors, StyleModifier, ThemeItem, ThemeSettings,
    };

    let fg = if dark { (250, 250, 250) } else { (9, 9, 11) };
    let mut theme = SynTheme::default();
    theme.settings = ThemeSettings {
        foreground: Some(Color { r: fg.0, g: fg.1, b: fg.2, a: 0xFF }),
        ..Default::default()
    };
    for row in SCOPE_CLASS_TABLE.iter() {
        let Some(group) = hljs_group_for_class(row.hljs) else { continue };
        let (r, g, b) = hljs_group_rgb(group, dark);
        let Ok(selector) = row.scope.parse::<ScopeSelectors>() else { continue };
        theme.scopes.push(ThemeItem {
            scope: selector,
            style: StyleModifier {
                foreground: Some(Color { r, g, b, a: 0xFF }),
                background: None,
                font_style: None,
            },
        });
    }
    theme
}

/// Plan 442 A6: highlight-only API for read-only code rendering — the
/// consumer is the VM render target's markdown `code_block` (musk-038 T16
/// decision (a): syntect-native highlighting; the vue track keeps
/// prismjs, visual near-parity per the 038 T15 matrix). No cosmic-text
/// buffer, no editor registry entry — plain syntect spans over the
/// shared singleton, so it is callable from any render path.
///
/// Returns consecutive text segments merged by color: `None` = the
/// theme's base foreground (render with the surrounding text color),
/// `Some((r, g, b))` = token color from the pre-registered `autoui-*`
/// theme for (dark, accent). Unknown languages and `lang: "none"`
/// degrade to a single unstyled segment.
pub fn highlight_segments(
    lang: &str,
    text: &str,
    dark: bool,
    accent: &str,
) -> Vec<(String, Option<(u8, u8, u8)>)> {
    use syntect::easy::HighlightLines;
    use syntect::util::LinesWithEndings;

    let fallback = || vec![(text.to_string(), None)];
    let Some(ext) = lang_to_extension(lang) else {
        return fallback();
    };
    let system = syntax_system();
    let Some(theme) = system.theme_set.themes.get(&theme_name(dark, accent)) else {
        return fallback();
    };
    let Some(syntax) = system.syntax_set.find_syntax_by_extension(ext) else {
        return fallback();
    };
    let base_fg = theme
        .settings
        .foreground
        .map(|fg| (fg.r, fg.g, fg.b));

    let mut hl = HighlightLines::new(syntax, theme);
    let mut out: Vec<(String, Option<(u8, u8, u8)>)> = Vec::new();
    for line in LinesWithEndings::from(text) {
        let Ok(regions) = hl.highlight_line(line, &system.syntax_set) else {
            continue;
        };
        for (style, seg) in regions {
            let rgb = (style.foreground.r, style.foreground.g, style.foreground.b);
            let color = if base_fg == Some(rgb) { None } else { Some(rgb) };
            match out.last_mut() {
                Some((prev, prev_color)) if *prev_color == color => prev.push_str(seg),
                _ => out.push((seg.to_string(), color)),
            }
        }
    }
    if out.is_empty() {
        return fallback();
    }
    out
}

/// Reset all buffers' syntax after the syntax set changed (not currently
/// needed — the set is static after boot).
#[allow(dead_code)]
pub fn warm_up(font_system: &mut FontSystem) {
    // Touch the lazy system once so the first editor doesn't pay the cost.
    let _ = syntax_system();
    let _ = font_system;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_syntax_is_available() {
        let system = syntax_system();
        assert!(
            system.syntax_set.find_syntax_by_extension("at").is_some(),
            "AutoLang .at syntax must be registered"
        );
        assert!(system.syntax_set.find_syntax_by_extension("rs").is_some());
    }

    #[test]
    fn theme_registration_is_stable() {
        // Pre-registered names resolve to themselves.
        let dark = theme_name(true, "indigo");
        assert_eq!(register_theme(&dark, cosmic_text::SyntaxTheme::default()), dark);
        // Unknown names snap to the dark indigo fallback.
        assert_eq!(
            register_theme("no-such-theme", cosmic_text::SyntaxTheme::default()),
            theme_name(true, "indigo")
        );
        // Unknown accents normalize when building names.
        assert_eq!(theme_name(true, "purple"), theme_name(true, "indigo"));
        assert_ne!(theme_name(true, "coral"), theme_name(false, "coral"));
        assert!(syntax_system().theme_set.themes.contains_key(&dark));
    }

    #[test]
    fn lang_mapping() {
        assert_eq!(lang_to_extension("rust"), Some("rs"));
        assert_eq!(lang_to_extension("Rust"), Some("rs"));
        assert_eq!(lang_to_extension("auto"), Some("at"));
        assert_eq!(lang_to_extension("none"), None);
        assert_eq!(lang_to_extension("python"), Some("py"));
    }

    /// Plan 442 A6: the highlight-only contract — segments concatenated
    /// reproduce the input, keywords/strings get non-base colors, unknown
    /// langs and `none` degrade to a single unstyled segment, and adjacent
    /// same-color regions merge (grammars split comment punctuation from
    /// the comment body and string quotes from the body — exact token
    /// boundaries are grammar-defined, the contract is color runs).
    #[test]
    fn highlight_segments_roundtrip_and_colors() {
        let code = "fn main() { let s = \"hi\"; }";
        let segs = highlight_segments("rust", code, true, "indigo");
        let joined: String = segs.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(joined, code);

        // Keywords (`fn`/`let`) and string bodies must carry non-base colors.
        // String quotes are separate grammar regions — match by content.
        let fn_seg = segs.iter().find(|(s, _)| s == "fn").expect("fn segment");
        assert!(fn_seg.1.is_some(), "keyword must be colored: {segs:?}");
        let str_seg = segs.iter().find(|(s, _)| s.contains("hi")).expect("string segment");
        assert!(str_seg.1.is_some(), "string must be colored: {segs:?}");
        // Base-fg runs are None and merge across ident+space boundaries
        // (e.g. " x " between `let` and `=` arrives as one unstyled run).
        assert!(segs.iter().any(|(_, c)| c.is_none()), "base-fg runs are None: {segs:?}");

        // Degradations: unknown language token, explicit none, empty text.
        assert_eq!(
            highlight_segments("none", code, true, "indigo"),
            vec![(code.to_string(), None)]
        );
        assert_eq!(
            highlight_segments("", code, true, "indigo"),
            vec![(code.to_string(), None)]
        );
        assert_eq!(highlight_segments("no-such-lang", code, true, "indigo").len(), 1);
        assert_eq!(
            highlight_segments("rust", "", true, "indigo"),
            vec![("".to_string(), None)]
        );

        // Multi-line input roundtrips with the trailing newline preserved.
        let multi = "let x = 1;\nfn f() {}\n";
        let segs = highlight_segments("rust", multi, true, "indigo");
        let joined: String = segs.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(joined, multi);
    }
}
