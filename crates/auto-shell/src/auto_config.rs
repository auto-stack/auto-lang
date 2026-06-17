//! Auto-language (Atom) configuration parser (Plan 318 unified config).
//!
//! Parses a subset of Auto's object-literal syntax for shell config files
//! (`~/.config/ash/config.at`, `prompt.at`, etc.). Supports **nested blocks**
//! (`prompt { git_branch { symbol : "⎇ " } }`), `//` comments, string and bare
//! values. Self-contained — no dependency on auto-lang.
//!
//! Block nesting is flattened to a dotted path in the result map:
//! `prompt { git_branch { symbol : "⎇ " } }` → key `"prompt.git_branch"` →
//! `{ symbol: "⎇ " }`.

use std::collections::HashMap;
use std::path::PathBuf;

// ── Parser ───────────────────────────────────────────────────────────────

/// Parse Auto-format config into `dotted_block_path → (key → value)`.
///
/// Top-level `key : value` pairs (outside any block) are stored under `""`.
/// Nested blocks flatten: `a { b { k : v } }` → `"a.b" → { k: v }`.
pub fn parse_auto_config(content: &str) -> HashMap<String, HashMap<String, String>> {
    let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut i = 0usize;
    let mut block_stack: Vec<String> = Vec::new();

    while i < n {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '}' {
            block_stack.pop();
            i += 1;
            continue;
        }
        if c == '{' {
            block_stack.push(String::new());
            i += 1;
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();
            while i < n && chars[i].is_whitespace() {
                i += 1;
            }

            if i < n && chars[i] == '{' {
                block_stack.push(ident);
                i += 1;
                continue;
            }
            if i < n && chars[i] == ':' {
                i += 1;
                while i < n && chars[i].is_whitespace() {
                    i += 1;
                }
                let value = read_value(&chars, &mut i, n);
                let block_key = if block_stack.is_empty() {
                    String::new()
                } else {
                    // Filter out empty (anonymous) segments.
                    block_stack
                        .iter()
                        .filter(|s| !s.is_empty())
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(".")
                };
                result.entry(block_key).or_default().insert(ident, value);
                continue;
            }
            continue;
        }
        i += 1;
    }
    result
}

/// Read a value: quoted string or bare token (until newline/comma/brace).
fn read_value(chars: &[char], i: &mut usize, n: usize) -> String {
    if *i < n && chars[*i] == '"' {
        *i += 1;
        let mut val = String::new();
        while *i < n && chars[*i] != '"' {
            let c = chars[*i];
            if c == '\\' && *i + 1 < n {
                match chars[*i + 1] {
                    '"' => { val.push('"'); *i += 2; continue; }
                    '\\' => { val.push('\\'); *i += 2; continue; }
                    'n' => { val.push('\n'); *i += 2; continue; }
                    't' => { val.push('\t'); *i += 2; continue; }
                    _ => {}
                }
            }
            val.push(c);
            *i += 1;
        }
        if *i < n && chars[*i] == '"' {
            *i += 1;
        }
        val
    } else {
        let mut val = String::new();
        while *i < n && chars[*i] != '\n' && chars[*i] != ',' && chars[*i] != '}' {
            val.push(chars[*i]);
            *i += 1;
        }
        val.trim().to_string()
    }
}

// ── Typed getters ────────────────────────────────────────────────────────

/// Get a string value from `block.key`.
pub fn get_str(
    cfg: &HashMap<String, HashMap<String, String>>,
    block: &str,
    key: &str,
) -> Option<String> {
    cfg.get(block).and_then(|b| b.get(key)).cloned()
}

/// Get a bool value (`true`/`false`/`1`/`0`/`on`/`off`).
pub fn get_bool(
    cfg: &HashMap<String, HashMap<String, String>>,
    block: &str,
    key: &str,
) -> Option<bool> {
    let v = get_str(cfg, block, key)?;
    match v.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Get an integer value.
pub fn get_int(
    cfg: &HashMap<String, HashMap<String, String>>,
    block: &str,
    key: &str,
) -> Option<i64> {
    get_str(cfg, block, key)?.trim().parse().ok()
}

// ── Path helpers (Plan 318: unified ~/.config/ash/) ──────────────────────

/// The Ash config directory: `~/.config/ash/` (or `%APPDATA%/ash/` fallback).
/// Creates it if it doesn't exist.
pub fn ash_dir() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".config").join("ash"));
    }
    if let Some(cfg) = dirs::config_dir() {
        candidates.push(cfg.join("ash"));
    }
    for c in &candidates {
        if c.exists() {
            return Some(c.clone());
        }
    }
    // Create the first candidate.
    if let Some(first) = candidates.first() {
        let _ = std::fs::create_dir_all(first);
        return Some(first.clone());
    }
    None
}

/// Resolve a config file name within the Ash config dir (`config.at`, `prompt.at`, etc.).
/// Falls back to old flat-location candidates for backward compatibility.
pub fn config_file(name: &str) -> Option<PathBuf> {
    // 1. New unified path: ~/.config/ash/<name>
    if let Some(dir) = ash_dir() {
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    // 2. Old flat location: ~/.config/<name> (e.g. ~/.config/ash.at)
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".config").join(name);
        if p.exists() {
            return Some(p);
        }
    }
    // 3. Old platform-config flat: %APPDATA%/<name>
    if let Some(cfg) = dirs::config_dir() {
        let p = cfg.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Load and parse a named config file from the Ash config dir.
/// Returns an empty map if the file doesn't exist.
pub fn load_file(name: &str) -> HashMap<String, HashMap<String, String>> {
    match config_file(name) {
        Some(p) => {
            let content = std::fs::read_to_string(&p).unwrap_or_default();
            parse_auto_config(&content)
        }
        None => HashMap::new(),
    }
}

/// Load the main config (`config.at`). Falls back to old `ash.at` for compat.
pub fn load() -> HashMap<String, HashMap<String, String>> {
    let mut result = load_file("config.at");
    if result.is_empty() {
        // Backward compat: old flat ash.at.
        result = load_file("ash.at");
    }
    result
}

/// Where a config file *would* be written (creates parent dir).
pub fn write_path(name: &str) -> Option<PathBuf> {
    ash_dir().map(|d| d.join(name))
}

// ── Serialize (write .at config) ─────────────────────────────────────────

/// Serialize a flat `block → (key → value)` map back to .at text.
pub fn serialize(config: &HashMap<String, HashMap<String, String>>) -> String {
    let mut out = String::new();
    // Group by top-level block (before any '.').
    let mut top_level: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut top_keys: Vec<(String, String)> = Vec::new();

    for (block, entries) in config {
        if let Some((top, sub)) = block.split_once('.') {
            // Nested: prompt.git_branch → top=prompt, sub=git_branch
            for (k, v) in entries {
                top_level
                    .entry(format!("{}.{}", top, sub))
                    .or_default()
                    .push((k.clone(), v.clone()));
            }
        } else if block.is_empty() {
            for (k, v) in entries {
                top_keys.push((k.clone(), v.clone()));
            }
        } else {
            for (k, v) in entries {
                top_level
                    .entry(block.clone())
                    .or_default()
                    .push((k.clone(), v.clone()));
            }
        }
    }

    // Top-level keys first.
    for (k, v) in &top_keys {
        out.push_str(&format!("{} : {}\n", k, quote_val(v)));
    }
    // Then blocks (sorted for determinism).
    let mut blocks: Vec<_> = top_level.keys().collect();
    blocks.sort();
    for block_key in blocks {
        let entries = &top_level[block_key];
        if let Some((top, sub)) = block_key.split_once('.') {
            out.push_str(&format!("{} {{\n", top));
            out.push_str(&format!("    {} {{\n", sub));
            for (k, v) in entries {
                out.push_str(&format!("        {} : {}\n", k, quote_val(v)));
            }
            out.push_str("    }\n");
            out.push_str("}\n");
        } else {
            out.push_str(&format!("{} {{\n", block_key));
            for (k, v) in entries {
                out.push_str(&format!("    {} : {}\n", k, quote_val(v)));
            }
            out.push_str("}\n");
        }
    }
    out
}

fn quote_val(v: &str) -> String {
    // Quote if the value contains spaces or special chars; bare otherwise.
    if v.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') && !v.is_empty() {
        v.to_string()
    } else {
        format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flat_block() {
        let cfg = parse_auto_config(r#"ls { icons : "nerdfont" }"#);
        assert_eq!(get_str(&cfg, "ls", "icons"), Some("nerdfont".to_string()));
    }

    #[test]
    fn parse_nested_two_levels() {
        let cfg = parse_auto_config(
            r#"
            prompt {
                format : "$directory"
                git_branch {
                    symbol : "⎇ "
                    style : "green bold"
                }
            }
            "#,
        );
        assert_eq!(get_str(&cfg, "prompt", "format"), Some("$directory".to_string()));
        assert_eq!(get_str(&cfg, "prompt.git_branch", "symbol"), Some("⎇ ".to_string()));
        assert_eq!(get_str(&cfg, "prompt.git_branch", "style"), Some("green bold".to_string()));
    }

    #[test]
    fn parse_bare_values() {
        let cfg = parse_auto_config(
            r#"
            shell {
                history_size : 10000
                autosuggestion : true
                edit_mode : emacs
            }
            "#,
        );
        assert_eq!(get_str(&cfg, "shell", "history_size"), Some("10000".to_string()));
        assert_eq!(get_bool(&cfg, "shell", "autosuggestion"), Some(true));
        assert_eq!(get_str(&cfg, "shell", "edit_mode"), Some("emacs".to_string()));
    }

    #[test]
    fn typed_getters() {
        let cfg = parse_auto_config(
            r#"
            data {
                count : 42
                enabled : true
                disabled : false
                name : "test"
                bad_int : abc
                bad_bool : maybe
            }
            "#,
        );
        assert_eq!(get_int(&cfg, "data", "count"), Some(42));
        assert_eq!(get_bool(&cfg, "data", "enabled"), Some(true));
        assert_eq!(get_bool(&cfg, "data", "disabled"), Some(false));
        assert_eq!(get_str(&cfg, "data", "name"), Some("test".to_string()));
        assert_eq!(get_int(&cfg, "data", "bad_int"), None);
        assert_eq!(get_bool(&cfg, "data", "bad_bool"), None);
    }

    #[test]
    fn parse_comments() {
        let cfg = parse_auto_config(
            r#"
            ls {
                // icons : "plain"
                icons : "emoji"   // inline
            }
            "#,
        );
        assert_eq!(get_str(&cfg, "ls", "icons"), Some("emoji".to_string()));
    }

    #[test]
    fn parse_top_level_key() {
        let cfg = parse_auto_config(r#"theme : "dark""#);
        assert_eq!(get_str(&cfg, "", "theme"), Some("dark".to_string()));
    }

    #[test]
    fn parse_empty_and_garbage() {
        assert!(parse_auto_config("").is_empty());
        assert!(parse_auto_config("{{{").is_empty());
    }

    #[test]
    fn serialize_round_trip() {
        let cfg = parse_auto_config(
            r#"
            shell {
                history_size : 10000
                autosuggestion : true
            }
            ls {
                icons : nerdfont
            }
            "#,
        );
        let text = serialize(&cfg);
        let back = parse_auto_config(&text);
        assert_eq!(get_str(&back, "shell", "history_size"), Some("10000".to_string()));
        assert_eq!(get_bool(&back, "shell", "autosuggestion"), Some(true));
        assert_eq!(get_str(&back, "ls", "icons"), Some("nerdfont".to_string()));
    }
}
