//! KNOWN-DEBT 396 signature-parity harness (Plan 415-B1 rider).
//!
//! `crates/a2r-std/src/*.rs` are hand-copied from `stdlib/auto/*.rs.at` with
//! no generation loop, so the two sides can drift silently (the `time.rs`
//! i32-vs-i64 drift was caught by luck, not by a check). This test compares
//! the `pub fn` signature surface of each pair and fails on:
//!
//!   1. name-set drift beyond the per-pair allowlist below, and
//!   2. arity drift on any name shared by both sides (after self
//!      normalization — see `PairOpt::implicit_self`).
//!
//! It deliberately does NOT compare param/return *types*: the Auto→Rust type
//! mapping (str/&str/String, int/i32/i64) is not mechanically invertible
//! here. Full fidelity belongs to the generation route named by the debt
//! entry; this harness is the cheap CI tripwire for the name/arity half.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// Per-pair comparison options.
struct PairOpt {
    /// a2r-std module file (under crates/a2r-std/src/).
    rs_mod: &'static str,
    /// The .rs.at side declares methods inside a `type X<T> { ... }` block
    /// with an IMPLICIT self (e.g. list.rs.at `pub fn len() int`), while the
    /// Rust copy takes `&self`. Normalize by counting the implicit self on
    /// the .rs.at side.
    implicit_self: bool,
    /// The .rs.at side declares methods as `pub fn Type.name(self Type, ..)`
    /// (qualified form, self explicit). Normalize by stripping the type
    /// prefix.
    qualified_methods: bool,
    /// Auto names that the a2r dispatch tables intentionally rename on the
    /// Rust side (at-name → rust-name).
    renames: &'static [(&'static str, &'static str)],
    /// Names allowed to exist on only one side of the pair (legacy drift,
    /// recorded in KNOWN-DEBT-AND-RISKS.md; new entries need a debt note).
    allow_only_at: &'static [&'static str],
    allow_only_rs: &'static [&'static str],
}

fn pairs() -> Vec<(&'static str, PairOpt)> {
    vec![
        (
            "env.rs.at",
            PairOpt {
                rs_mod: "env",
                implicit_self: false,
                qualified_methods: false,
                renames: &[],
                allow_only_at: &[],
                // mkdir_all: Rust-side addition without an .rs.at entry.
                allow_only_rs: &["mkdir_all"],
            },
        ),
        (
            "file.rs.at",
            PairOpt {
                // NOTE: the .at module is `file` but the a2r-std copy lives
                // in fs.rs — the pairing itself is part of the 396 debt
                // surface and is recorded here explicitly.
                rs_mod: "fs",
                implicit_self: false,
                qualified_methods: false,
                renames: &[],
                allow_only_at: &[],
                allow_only_rs: &["mkdir_all"],
            },
        ),
        (
            "json.rs.at",
            PairOpt {
                rs_mod: "json",
                implicit_self: false,
                qualified_methods: false,
                renames: &[
                    ("json_value_type", "value_type"),
                    ("json_is_null", "is_null"),
                    ("json_as_string", "as_string"),
                    ("json_as_number", "as_number"),
                    ("json_as_array", "as_array"),
                    ("json_as_bool", "as_bool"),
                    ("json_get_at", "get_at"),
                    ("json_has_key", "has_key"),
                    ("json_keys", "keys"),
                ],
                allow_only_at: &["try_decode", "json_as_int"],
                allow_only_rs: &[
                    "parse_opt",
                    "as_string_str",
                    "as_int",
                    "as_float",
                    "as_object",
                    "as_bool_str",
                    "as_int_str",
                    "get",
                    "get_str",
                    "len",
                    "len_str",
                    "to_string",
                ],
            },
        ),
        (
            "list.rs.at",
            PairOpt {
                rs_mod: "list",
                implicit_self: true,
                qualified_methods: false,
                renames: &[],
                allow_only_at: &["drop", "iter", "next"],
                allow_only_rs: &[
                    "new",
                    "with_capacity",
                    "first",
                    "last",
                    "insert",
                    "remove",
                    "to_vec",
                ],
            },
        ),
        (
            "math.rs.at",
            PairOpt {
                rs_mod: "math",
                implicit_self: false,
                qualified_methods: false,
                renames: &[],
                allow_only_at: &[],
                allow_only_rs: &["abs_i64", "min_i64", "max_i64", "square", "cube"],
            },
        ),
        (
            "str.rs.at",
            PairOpt {
                rs_mod: "str",
                implicit_self: false,
                qualified_methods: false,
                renames: &[],
                allow_only_at: &[],
                allow_only_rs: &[
                    "match_count",
                    "replace_first",
                    "str_substr",
                    "str_ends_with",
                    "str_starts_with",
                    "str_split",
                    "from_bytes",
                    "str_contains",
                    "str_find",
                    "str_find_from",
                    "str_to_lower",
                    "str_to_upper",
                    "str_trim",
                ],
            },
        ),
        (
            "time.rs.at",
            PairOpt {
                rs_mod: "time",
                implicit_self: false,
                qualified_methods: false,
                renames: &[],
                allow_only_at: &[],
                allow_only_rs: &["time_now"],
            },
        ),
        (
            "sqlite.rs.at",
            PairOpt {
                rs_mod: "sqlite",
                implicit_self: false,
                qualified_methods: true,
                renames: &[],
                allow_only_at: &[],
                allow_only_rs: &[],
            },
        ),
    ]
}

/// Strip line/block comments so doc-comment mentions of `pub fn` do not
/// pollute the scan.
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut prev = ' ';
                for c in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                    if c == '\n' {
                        out.push('\n');
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Count top-level (paren-free) comma-separated params; empty → 0.
fn param_count(params: &str) -> usize {
    let p = params.trim();
    if p.is_empty() {
        return 0;
    }
    let mut depth = 0usize;
    let mut n = 1usize;
    for c in p.chars() {
        match c {
            '(' | '<' | '[' => depth += 1,
            ')' | '>' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => n += 1,
            _ => {}
        }
    }
    n
}

/// Extract `pub fn [Type.]name(params)` signatures from a comment-stripped
/// .rs.at source. Returns name → arity (self counted when written).
fn scan_at(src: &str) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = src[i..].find("pub fn ") {
        let start = i + rel;
        let mut j = start + "pub fn ".len();
        // Optional `Type.` qualifier.
        let mut name = String::new();
        while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
            name.push(bytes[j] as char);
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'.' {
            // qualified method — restart name capture after the dot
            j += 1;
            name.clear();
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                name.push(bytes[j] as char);
                j += 1;
            }
        }
        // Optional generics, then the param list.
        while j < bytes.len() && bytes[j] != b'(' {
            j += 1;
        }
        if j >= bytes.len() || name.is_empty() {
            i = start + "pub fn ".len();
            continue;
        }
        // Balanced paren scan.
        let mut depth = 0usize;
        let mut k = j;
        while k < bytes.len() {
            match bytes[k] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            k += 1;
        }
        if k >= bytes.len() {
            i = start + "pub fn ".len();
            continue;
        }
        let params = &src[j + 1..k];
        out.insert(name, param_count(params));
        i = k + 1;
    }
    out
}

/// Extract `pub fn name(params)` signatures from a comment-stripped Rust
/// source (covers both free fns and impl methods).
fn scan_rs(src: &str) -> BTreeMap<String, usize> {
    scan_at(src)
}

#[test]
fn a2r_std_signature_parity() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let stdlib_dir = manifest.join("../../stdlib/auto");
    let rs_dir = manifest.join("../a2r-std/src");

    let mut failures: Vec<String> = Vec::new();

    for (at_file, opt) in pairs() {
        let at_src = fs::read_to_string(stdlib_dir.join(at_file))
            .unwrap_or_else(|e| panic!("read {at_file}: {e}"));
        let rs_src = fs::read_to_string(rs_dir.join(format!("{}.rs", opt.rs_mod)))
            .unwrap_or_else(|e| panic!("read a2r-std/{}.rs: {e}", opt.rs_mod));

        let mut at = scan_at(&strip_comments(&at_src));
        // Self normalization.
        if opt.implicit_self {
            // Every fn in a type-block module is a method with implicit self;
            // the Rust copy takes &self, so both sides get +1 conceptually —
            // instead, discount the Rust &self so the user params line up.
            at = at.into_iter().map(|(k, v)| (k, v + 1)).collect();
        }

        let rs = scan_rs(&strip_comments(&rs_src));

        // Apply the dispatch rename table to the .at side.
        for (at_name, rs_name) in opt.renames {
            if let Some(arity) = at.remove(*at_name) {
                at.insert(rs_name.to_string(), arity);
            }
        }

        // Rust side: a leading &self/&mut self/self param is the receiver —
        // it lines up with the Auto side's explicit `self X` (already
        // counted) or implicit self (already +1). No adjustment needed.

        let mut at_only: Vec<String> = at
            .keys()
            .filter(|k| !rs.contains_key(*k) && !opt.allow_only_at.contains(&k.as_str()))
            .cloned()
            .collect();
        at_only.sort_unstable();
        let mut rs_only: Vec<String> = rs
            .keys()
            .filter(|k| !at.contains_key(*k) && !opt.allow_only_rs.contains(&k.as_str()))
            .cloned()
            .collect();
        rs_only.sort_unstable();

        if !at_only.is_empty() {
            failures.push(format!(
                "{at_file}: only on .rs.at side (missing from a2r-std::{}) — add the copy or extend the allowlist with a debt note: {:?}",
                opt.rs_mod, at_only
            ));
        }
        if !rs_only.is_empty() {
            failures.push(format!(
                "{at_file}: only on a2r-std::{} side (missing from .rs.at) — mirror the declaration or extend the allowlist with a debt note: {:?}",
                opt.rs_mod, rs_only
            ));
        }
        for (name, at_arity) in &at {
            if let Some(rs_arity) = rs.get(name) {
                if at_arity != rs_arity {
                    failures.push(format!(
                        "{at_file}: arity drift on `{name}`: .rs.at={at_arity} vs a2r-std::{}={rs_arity}",
                        opt.rs_mod
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "KNOWN-DEBT 396 signature drift detected:\n{}",
        failures.join("\n")
    );
}
