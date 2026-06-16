//! Auto-language (Atom) configuration parser.
//!
//! Parses a small subset of Auto's object-literal syntax for shell config
//! (`~/.config/ash.at`), e.g.:
//!
//! ```auto
//! ls {
//!     icons : "nerdfont"   // plain | nerdfont | emoji | off
//! }
//! ```
//!
//! This is intentionally a tiny, self-contained scanner — NOT the full Auto
//! parser — because config values are static `key : "string"` pairs. It avoids
//! spinning up the Auto VM just to read settings, and stays decoupled from
//! auto-lang internals. Supports nested `block { … }` with `key : "value"`
//! entries and `//` line comments (outside strings).

use std::collections::HashMap;

/// Parse Auto-format config into `block → (key → value)`.
///
/// Top-level `key : value` pairs (outside any block) are stored under the
/// empty-string block `""`.
pub fn parse_auto_config(content: &str) -> HashMap<String, HashMap<String, String>> {
    let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut i = 0usize;
    // Current block name; None means "between blocks" (top-level keys go to "").
    let mut current_block: Option<String> = None;

    while i < n {
        let c = chars[i];

        // Whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        // Line comment `//` (we're never inside a string here — strings are
        // consumed wholesale when reading a value).
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // Closing brace → leave current block.
        if c == '}' {
            current_block = None;
            i += 1;
            continue;
        }
        // A stray `{` (no preceding identifier) → anonymous block.
        if c == '{' {
            current_block = Some(String::new());
            i += 1;
            continue;
        }

        // Identifier — could be a block name (`ident {`) or a key (`ident : value`).
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();

            // Skip whitespace before the next significant char.
            while i < n && chars[i].is_whitespace() {
                i += 1;
            }

            // `ident {` → enter a named block.
            if i < n && chars[i] == '{' {
                current_block = Some(ident);
                i += 1;
                continue;
            }

            // `ident : value` → record under the current block.
            if i < n && chars[i] == ':' {
                i += 1; // consume ':'
                while i < n && chars[i].is_whitespace() {
                    i += 1;
                }
                let value = read_value(&chars, &mut i, n);
                let block = current_block.clone().unwrap_or_default();
                result.entry(block).or_default().insert(ident, value);
                continue;
            }

            // Identifier with neither `{` nor `:` — ignore.
            continue;
        }

        // Any other punctuation (commas, etc.) — skip.
        i += 1;
    }

    result
}

/// Read a value token starting at `i`: a double-quoted string, or (for
/// non-string values) everything up to the next newline / `,` / `}`.
fn read_value(chars: &[char], i: &mut usize, n: usize) -> String {
    if *i < n && chars[*i] == '"' {
        *i += 1; // consume opening quote
        let mut val = String::new();
        while *i < n && chars[*i] != '"' {
            let c = chars[*i];
            // Minimal escape handling: \" \\
            if c == '\\' && *i + 1 < n {
                let next = chars[*i + 1];
                match next {
                    '"' => {
                        val.push('"');
                        *i += 2;
                        continue;
                    }
                    '\\' => {
                        val.push('\\');
                        *i += 2;
                        continue;
                    }
                    _ => {}
                }
            }
            val.push(c);
            *i += 1;
        }
        if *i < n && chars[*i] == '"' {
            *i += 1; // consume closing quote
        }
        val
    } else {
        // Bare value: read until newline / comma / closing brace.
        let mut val = String::new();
        while *i < n && chars[*i] != '\n' && chars[*i] != ',' && chars[*i] != '}' {
            val.push(chars[*i]);
            *i += 1;
        }
        val.trim().to_string()
    }
}

/// Path to the Auto-format config file (`~/.config/ash.at`), if a config dir exists.
pub fn config_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("ash.at"))
}

/// Load and parse `~/.config/ash.at`. Returns an empty map if missing/unreadable.
pub fn load() -> HashMap<String, HashMap<String, String>> {
    match config_path() {
        Some(p) if p.exists() => {
            let content = std::fs::read_to_string(&p).unwrap_or_default();
            parse_auto_config(&content)
        }
        _ => HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_block_with_string_value() {
        let cfg = parse_auto_config(
            r#"
            ls {
                icons : "nerdfont"
            }
            "#,
        );
        assert_eq!(cfg.get("ls").and_then(|b| b.get("icons")), Some(&"nerdfont".to_string()));
    }

    #[test]
    fn parse_ignores_comments() {
        // Only the active (uncommented) line should win.
        let cfg = parse_auto_config(
            r#"
            ls {
                // icons : "plain"
                icons : "emoji"   // default-ish
                // icons : "off"
            }
            "#,
        );
        assert_eq!(cfg.get("ls").and_then(|b| b.get("icons")), Some(&"emoji".to_string()));
    }

    #[test]
    fn parse_multiple_blocks_and_keys() {
        let cfg = parse_auto_config(
            r#"
            ls {
                icons : "plain"
                long  : "on"
            }
            completion {
                case_sensitive : "false"
            }
            "#,
        );
        assert_eq!(cfg.get("ls").and_then(|b| b.get("icons")), Some(&"plain".to_string()));
        assert_eq!(cfg.get("ls").and_then(|b| b.get("long")), Some(&"on".to_string()));
        assert_eq!(
            cfg.get("completion").and_then(|b| b.get("case_sensitive")),
            Some(&"false".to_string())
        );
    }

    #[test]
    fn parse_empty_and_garbage_safe() {
        assert!(parse_auto_config("").is_empty());
        assert!(parse_auto_config("not valid {{{{").is_empty());
        // Unterminated string → take to end, no panic.
        let cfg = parse_auto_config(r#"ls { icons : "plain "#);
        assert_eq!(cfg.get("ls").and_then(|b| b.get("icons")).map(|s| s.as_str()), Some("plain "));
    }

    #[test]
    fn parse_top_level_key() {
        let cfg = parse_auto_config(r#"theme : "dark""#);
        assert_eq!(cfg.get("").and_then(|b| b.get("theme")), Some(&"dark".to_string()));
    }
}
