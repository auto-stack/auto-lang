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
        Mutex::new(SyntaxRegistry {
            system: Box::leak(Box::new(build_syntax_system())),
        })
    })
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
}
