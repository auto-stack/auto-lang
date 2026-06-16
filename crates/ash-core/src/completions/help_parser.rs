//! Heuristic parser for command `--help` output → [`CompletionSpec`] (Plan 315).
//!
//! Goal: extract flags + subcommands from the `--help` text of *any* command so
//! ash can offer completion without a hand-written spec. Covers the common
//! formats produced by clap (Rust), argparse/getopt (Python/C), and typical
//! hand-written help (~80% of commands). Anything it can't parse yields an
//! empty spec (the caller writes an empty cache marker and falls back to file
//! completion).
//!
//! This module is pure (no I/O, no process spawning) — the caller runs
//! `cmd --help` and feeds the text here.

use regex::Regex;

use crate::completions::spec::{CompletionSpec, FlagSpec, SubcommandSpec};

/// Parse `--help` output for `cmd` into a [`CompletionSpec`] (flags + subcommands).
///
/// Positional `args` are intentionally left empty — they rarely appear in
/// `--help` in a machine-readable way. Flag values (`-m <MSG>`) set
/// `FlagSpec::arg` so the engine knows the flag takes a value.
pub fn parse_help(cmd: &str, help_text: &str) -> CompletionSpec {
    let mut spec = CompletionSpec::new(cmd);

    // Long flag: --word (allow letters/digits/-/_/. inside). No lookaround in
    // the `regex` crate, so we match greedily and trim trailing punctuation.
    let long_re = Regex::new(r"--([A-Za-z][A-Za-z0-9_.\-]*)").unwrap();
    // Subcommand section heading (case-insensitive), e.g. "Commands:" / "SUBCOMMANDS:".
    let heading_re = Regex::new(r"(?i)^(commands|subcommands)\s*:?\s*$").unwrap();

    let mut section: Option<Section> = None;

    for raw in help_text.lines() {
        let trimmed = raw.trim();

        // Section heading detection: a line that IS a heading (possibly indented
        // for sub-subcommand help). Switches the active section.
        if let Some(_) = heading_re.captures(trimmed) {
            section = Some(Section::Commands);
            continue;
        }
        // Other top-level headings ("Options:", "Usage:", "FLAGS:", …) end a
        // Commands section so we don't swallow option lines as subcommands.
        if is_top_level_heading(raw, trimmed) {
            section = Some(Section::Other);
            continue;
        }
        if trimmed.is_empty() {
            // Blank line: keep section (some help separates items with blanks).
            continue;
        }

        match section {
            Some(Section::Commands) => {
                if let Some(sub) = parse_subcommand_line(trimmed) {
                    spec = spec.subcommand(sub);
                }
                // A commands-section line could also be a flag in disguise; skip
                // flag parsing here to avoid duplicates.
                continue;
            }
            _ => {}
        }

        // Flag detection: indented line containing a flag token.
        if let Some(flag) = parse_flag_line(raw, &long_re) {
            spec = spec.flag(flag);
        }
    }

    spec
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    Commands,
    Other,
}

/// A "top-level heading" is a non-indented line ending in ':' (Usage:, Options:, …).
fn is_top_level_heading(raw: &str, trimmed: &str) -> bool {
    !raw.starts_with(' ')
        && !raw.starts_with('\t')
        && trimmed.ends_with(':')
        && trimmed.len() <= 32
}

/// Parse a subcommand line from a Commands section: `name   description…`.
fn parse_subcommand_line(trimmed: &str) -> Option<SubcommandSpec> {
    let mut parts = trimmed.split_whitespace();
    let name = parts.next()?;
    // Reject flags / placeholders.
    if name.starts_with('-')
        || name.starts_with('<')
        || name.starts_with('[')
        || name.contains('=')
    {
        return None;
    }
    // Only accept identifier-like names (alphanum + _ + -).
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    let desc: String = parts.collect::<Vec<_>>().join(" ");
    let mut sub = SubcommandSpec::new(name);
    if !desc.is_empty() {
        sub = sub.desc(&desc);
    }
    Some(sub)
}

/// Parse an indented line as a flag definition, if it looks like one.
fn parse_flag_line(raw: &str, long_re: &Regex) -> Option<FlagSpec> {
    // Flag lines are indented.
    if !raw.starts_with(' ') && !raw.starts_with('\t') {
        return None;
    }
    let t = raw.trim_start();
    if t.is_empty() || !t.starts_with('-') {
        return None;
    }

    // Capture the "token part" — from start up to a 2+ space gap (description)
    // or end of line. e.g. `-b, --branch <NAME>` | `Create a branch`.
    let token_part: String = split_token_part(t);

    // Long flags.
    let longs: Vec<String> = long_re
        .captures_iter(&token_part)
        .map(|c| c.get(1).unwrap().as_str().trim_end_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_').to_string())
        .collect();

    // Short flags: blank out each `--word` first (so `--branch` doesn't yield
    // a bogus short `-b`), then find `-X` (single letter) in what remains.
    let mut cleaned = String::new();
    let mut last = 0;
    for m in long_re.find_iter(&token_part) {
        cleaned.push_str(&token_part[last..m.start()]);
        cleaned.push_str(&" ".repeat(m.end() - m.start()));
        last = m.end();
    }
    cleaned.push_str(&token_part[last..]);
    let short_re = Regex::new(r"(?:^|[^A-Za-z0-9])-([A-Za-z])(?:[^A-Za-z0-9]|$)").unwrap();
    let shorts: Vec<String> = short_re
        .captures_iter(&cleaned)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .collect();

    let long = longs.into_iter().next();
    let short = shorts.into_iter().next();
    if long.is_none() && short.is_none() {
        return None;
    }

    // Arg placeholder: `<...>` or an ALL-CAPS token in the token part.
    let arg = extract_arg_placeholder(&token_part);

    // Description: the remainder after the token part.
    let desc = t.get(token_part.len()..)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let f = FlagSpec {
        short,
        long,
        desc,
        arg,
    };
    Some(f)
}

/// Take the leading run of non-(2+space) characters as the "token part".
/// `-b, --branch <NAME>    desc` → `-b, --branch <NAME>`.
fn split_token_part(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        // 2+ spaces → end of token part.
        if chars[i] == ' ' && i + 1 < chars.len() && chars[i + 1] == ' ' {
            break;
        }
        out.push(chars[i]);
        i += 1;
    }
    out.trim_end().to_string()
}

/// Extract an argument placeholder from a flag token part: `<NAME>` (preferred),
/// `=VALUE`, or a standalone ALL-CAPS token.
fn extract_arg_placeholder(token_part: &str) -> Option<String> {
    // <NAME> style.
    let angle_re = Regex::new(r"<([^>]+)>").unwrap();
    if let Some(c) = angle_re.captures(token_part) {
        return Some(c.get(1).unwrap().as_str().to_string());
    }
    // =VALUE style (e.g. `--color=WHEN`).
    let eq_re = Regex::new(r"=([A-Z][A-Z0-9_]*)").unwrap();
    if let Some(c) = eq_re.captures(token_part) {
        return Some(c.get(1).unwrap().as_str().to_string());
    }
    // Standalone ALL-CAPS token (FILE, PATH, DIR, …).
    let caps_re = Regex::new(r"(?:^|[^A-Za-z0-9])([A-Z][A-Z0-9_]{1,})(?:[^A-Za-z0-9]|$)").unwrap();
    if let Some(c) = caps_re.captures(token_part) {
        let w = c.get(1).unwrap().as_str();
        // Avoid treating the flag's own text as a placeholder.
        if w != token_part.trim() {
            return Some(w.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completions::spec::CompletionSource;

    const CLAP_HELP: &str = r#"
ripgrep 13.0
USAGE: rg [OPTIONS] PATTERN [PATH...]

Arguments:
  [PATTERN]  Regular expression

Options:
  -i, --ignore-case      Case insensitive search
  -A, --after-context <NUM>  Show NUM lines after each match
      --type <TYPE>      Only search files matching TYPE
  -h, --help             Print help
  -V, --version          Print version

Commands:
  no subcommands here
"#;

    #[test]
    fn parses_long_and_short_flags() {
        let spec = parse_help("rg", CLAP_HELP);
        let longs: Vec<&str> = spec.flags.iter().filter_map(|f| f.long.as_deref()).collect();
        assert!(longs.contains(&"ignore-case"), "longs: {:?}", longs);
        assert!(longs.contains(&"after-context"));
        assert!(longs.contains(&"type"));
        assert!(longs.contains(&"help"));
        assert!(longs.contains(&"version"));
        let shorts: Vec<&str> = spec.flags.iter().filter_map(|f| f.short.as_deref()).collect();
        assert!(shorts.contains(&"i"), "shorts: {:?}", shorts);
        assert!(shorts.contains(&"A"));
        assert!(shorts.contains(&"h"));
    }

    #[test]
    fn detects_takes_arg() {
        let spec = parse_help("rg", CLAP_HELP);
        let after = spec.flags.iter().find(|f| f.long.as_deref() == Some("after-context")).unwrap();
        assert_eq!(after.arg.as_deref(), Some("NUM"));
        let typ = spec.flags.iter().find(|f| f.long.as_deref() == Some("type")).unwrap();
        assert_eq!(typ.arg.as_deref(), Some("TYPE"));
        // Boolean flag → no arg.
        let ign = spec.flags.iter().find(|f| f.long.as_deref() == Some("ignore-case")).unwrap();
        assert!(ign.arg.is_none());
    }

    #[test]
    fn parses_subcommands_section() {
        let help = r#"
USAGE: mytool [COMMAND]

Commands:
  build    Build the project
  test     Run tests
  deploy   Deploy somewhere

Options:
  -v, --verbose    Verbose
"#;
        let spec = parse_help("mytool", help);
        let subs: Vec<&str> = spec.subcommands.iter().map(|s| s.name.as_str()).collect();
        assert!(subs.contains(&"build"), "subs: {:?}", subs);
        assert!(subs.contains(&"test"));
        assert!(subs.contains(&"deploy"));
        // Options section after Commands shouldn't be swallowed as subcommands.
        assert!(!subs.contains(&"verbose"));
        // And the flag in Options is still parsed.
        let longs: Vec<&str> = spec.flags.iter().filter_map(|f| f.long.as_deref()).collect();
        assert!(longs.contains(&"verbose"));
    }

    #[test]
    fn empty_help_yields_empty_spec() {
        let spec = parse_help("weird", "totally unstructured text\nno flags at all");
        assert!(spec.flags.is_empty());
        assert!(spec.subcommands.is_empty());
        assert_eq!(spec.command, "weird");
    }

    #[test]
    fn command_name_is_set() {
        let spec = parse_help("rg", CLAP_HELP);
        assert_eq!(spec.command, "rg");
    }

    /// Ensure the empty-spec type is usable (source field exists for symmetry).
    #[test]
    fn empty_spec_can_still_register() {
        let spec = parse_help("x", "");
        assert_eq!(spec.command, "x");
        let _ = CompletionSource::Static(vec![]); // touch the enum to avoid unused import
    }
}
