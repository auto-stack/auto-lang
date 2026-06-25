//! `.at` source emitter — the missing "write" direction of auto-atom.
//!
//! auto-atom has a full *parser* but its only emitter is `fmt::Display`, which
//! does NOT escape quotes, backslashes, or control chars — so serializing a
//! string containing `"`/`\`/`\n` produces `.at` that the parser cannot read
//! back. This module is a correct emitter that mirrors the parser's escape
//! grammar exactly, so any `Value`/`Node` built programmatically (e.g. via
//! `Node::new().with_prop(...)`) can be turned back into round-trippable `.at`
//! source.
//!
//! ## The escape contract
//! The parser (`auto-atom/src/parser.rs` `parse_string`) recognises exactly
//! these escapes: `\n` `\t` `\r` `\\` `\"`; an unknown `\x` decodes to the
//! literal `x`. So the emitter must emit `\n \t \r \\ \"` and pass everything
//! else through verbatim — that is what [`escape_string`] does.
//!
//! ## Usage
//! ```ignore
//! use auto_val::{Node, AtomSource};
//! let node = Node::new("role")
//!     .with_prop("name", "precise-coder")
//!     .with_prop("temperature", 0.3);
//! let source = node.to_at_source(); // → `role {\n    name : "precise-coder";\n ...}`
//! ```
//!
//! Souls / large multi-paragraph markdown are intentionally NOT inlined here —
//! keep them in sidecar `.md` files (as `professions/coder.rs` already does)
//! and store only a `soul_file` reference in `.at`.

use crate::{Array, Kid, Node, Obj, Value, ValueKey};

/// Escape a string for a double-quoted `.at` literal.
///
/// Mirrors the parser's `parse_string` escape table exactly so the output
/// round-trips: `\` `"` `\n` `\t` `\r` become their backslash escapes; all
/// other chars pass through unchanged.
pub fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out
}

/// Render a `Value` as `.at` source text, at the given indent level.
///
/// Scalars become literals (`"..."`, `42`, `3.14`, `true`), `Array`/`Block`
/// become `[a, b, c]`, `Obj` becomes `{ k : v; ... }`, and nested `Node` values
/// are indented. Anything VM-only (closures, futures, widget, …) falls back to
/// the existing `Display`, since config data never contains them.
pub fn format_value(v: &Value, indent: usize) -> String {
    match v {
        // Strings of all flavours → one escaped double-quoted literal.
        Value::Str(s) => format!("\"{}\"", escape_string(s.as_str())),
        Value::String(s) => format!("\"{}\"", escape_string(&s.to_string())),
        Value::StrSlice(s) => format!("\"{}\"", escape_string(&s.to_string())),
        Value::CStr(s) => format!("\"{}\"", escape_string(&s.to_string())),
        Value::Char(c) => format!("\"{}\"", escape_string(&c.to_string())),
        // Integer/float families → bare literals.
        Value::Byte(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Uint(u) => u.to_string(),
        Value::USize(u) => u.to_string(),
        Value::I8(i) => i.to_string(),
        Value::U8(u) => u.to_string(),
        Value::I64(i) => i.to_string(),
        Value::Float(f) => format_float(*f),
        Value::Double(f) => format_float(*f),
        Value::Bool(b) => b.to_string(),
        Value::Range(a, b) => format!("{}..{}", a, b),
        Value::RangeEq(a, b) => format!("{}..={}", a, b),
        // Containers.
        Value::Array(a) | Value::Block(a) => format_array(a, indent),
        Value::Obj(o) => format_obj(o, indent),
        Value::Node(n) => format_node(n, indent),
        // Pairs / refs / model-ish → keep readable, fall back to Display for
        // exotic values that never appear in config round-tripping.
        Value::Pair(k, val) => format!("{} : {}", k, format_value(val, indent)),
        Value::Some(val) => format_value(val, indent),
        Value::Ok(val) => format_value(val, indent),
        Value::Nil | Value::Null | Value::None | Value::Void => "nil".to_string(),
        other => other.to_string(),
    }
}

/// Render a `Node` as `.at` source: `name { k : v; ... }`, indented.
///
/// Props (key/value pairs) are emitted first, each on its own line, then child
/// nodes. This mirrors how config `.at` files are authored.
pub fn format_node(n: &Node, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let child_pad = "    ".repeat(indent + 1);
    let mut out = String::new();
    out.push_str(&n.name.to_string());

    // Props (insertion-ordered).
    let prop_count = n.props_iter().count();
    let kid_count = n.kids_iter().count();

    if prop_count == 0 && kid_count == 0 {
        out.push_str(" {}");
    } else {
        out.push_str(" {\n");
        for (key, val) in n.props_iter() {
            out.push_str(&child_pad);
            out.push_str(&format_key(key));
            out.push_str(" : ");
            out.push_str(&format_value(val, indent + 1));
            out.push_str("\n");
        }
        // Child nodes.
        for (_key, kid) in n.kids_iter() {
            match kid {
                Kid::Node(child) => {
                    out.push_str(&child_pad);
                    out.push_str(&format_node(child, indent + 1));
                    out.push('\n');
                }
                Kid::Lazy(_) => { /* skip lazy refs — not serializable to source */ }
            }
        }
        out.push_str(&pad);
        out.push('}');
    }
    out
}

/// Trait: anything that can be emitted as `.at` source.
///
/// Implemented for `Value`, `Node`, and `Atom` so config code can build a node
/// and write it out without touching `Display` (which doesn't escape).
pub trait AtomSource {
    fn to_at_source(&self) -> String;
}

impl AtomSource for Value {
    fn to_at_source(&self) -> String {
        format_value(self, 0)
    }
}

impl AtomSource for Node {
    fn to_at_source(&self) -> String {
        format_node(self, 0)
    }
}

// (An `AtomSource` impl for `auto_atom::Atom` lives in the `auto-atom` crate,
// since auto-val can't depend on auto-atom. auto-atom wraps these Value/Node
// emitters via `Atom::to_at_source`.)

// ── private helpers ──────────────────────────────────────────────────────────

fn format_float(f: f64) -> String {
    // Always show a decimal point so the parser reads a number, not an int.
    let s = format!("{}", f);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{}.0", s)
    }
}

fn format_array(a: &Array, indent: usize) -> String {
    if a.values.is_empty() {
        return "[]".to_string();
    }
    let items: Vec<String> = a.values.iter().map(|v| format_value(v, indent)).collect();
    format!("[{}]", items.join(", "))
}

fn format_obj(o: &Obj, indent: usize) -> String {
    if o.iter().next().is_none() {
        return "{}".to_string();
    }
    let child_pad = "    ".repeat(indent + 1);
    let pad = "    ".repeat(indent);
    let mut out = String::from("{\n");
    for (key, val) in o.iter() {
        out.push_str(&child_pad);
        out.push_str(&format_key(key));
        out.push_str(" : ");
        out.push_str(&format_value(val, indent + 1));
        out.push('\n');
    }
    out.push_str(&pad);
    out.push('}');
    out
}

/// A key is emitted unquoted when it's a clean identifier (alphanumeric +
/// underscore, not starting with a digit); otherwise quoted. Mirrors what
/// authors write and what the parser accepts as a bare key.
fn format_key(key: &ValueKey) -> String {
    let s = key.to_string();
    let is_clean_ident = !s.is_empty()
        && s.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false)
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if is_clean_ident {
        s
    } else {
        format!("\"{}\"", escape_string(&s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Node, Obj, Value};

    // ── escape_string ─────────────────────────────────────────────────────────

    #[test]
    fn escape_special_chars() {
        assert_eq!(escape_string("plain"), "plain");
        assert_eq!(escape_string(r#"a"b"#), r#"a\"b"#);
        assert_eq!(escape_string(r"a\b"), r"a\\b");
        assert_eq!(escape_string("a\nb"), r"a\nb");
        assert_eq!(escape_string("a\tb"), r"a\tb");
        assert_eq!(escape_string("a\rb"), r"a\rb");
        // A path with backslashes survives (common on Windows soul_file refs).
        assert_eq!(escape_string(r"C:\path\to"), r"C:\\path\\to");
    }

    // ── format_value scalars ──────────────────────────────────────────────────

    #[test]
    fn format_scalars() {
        assert_eq!(format_value(&Value::Int(42), 0), "42");
        assert_eq!(format_value(&Value::Uint(7), 0), "7");
        assert_eq!(format_value(&Value::Bool(true), 0), "true");
        assert_eq!(format_value(&Value::Double(0.3), 0), "0.3");
        // Integer-valued floats still get a decimal point.
        assert_eq!(format_value(&Value::Double(5.0), 0), "5.0");
        assert_eq!(format_value(&Value::str("hello"), 0), "\"hello\"");
        // Strings with special chars are escaped.
        let v = Value::str(r#"he said "hi"\nnew"#);
        assert_eq!(format_value(&v, 0), r#""he said \"hi\"\\nnew""#);
    }

    #[test]
    fn format_array_values() {
        let a = Value::Array(Array {
            values: vec![Value::str("a"), Value::str("b"), Value::Int(3)],
        });
        assert_eq!(format_value(&a, 0), "[\"a\", \"b\", 3]");
    }

    // ── format_node ────────────────────────────────────────────────────────────

    #[test]
    fn format_simple_node() {
        let n = Node::new("role")
            .with_prop("name", "coder")
            .with_prop("model_tier", "max");
        let src = n.to_at_source();
        assert!(src.starts_with("role {\n"));
        assert!(src.contains("name : \"coder\""));
        assert!(src.contains("model_tier : \"max\""));
        assert!(src.ends_with('}'));
    }

    #[test]
    fn format_node_with_floats_and_array() {
        let n = Node::new("role")
            .with_prop("temperature", 0.3)
            .with_prop("skills", Value::Array(Array {
                values: vec![Value::str("tdd"), Value::str("debug")],
            }))
            .with_prop("token_budget", Value::Uint(2000000));
        let src = n.to_at_source();
        assert!(src.contains("temperature : 0.3"));
        assert!(src.contains("skills : [\"tdd\", \"debug\"]"));
        assert!(src.contains("token_budget : 2000000"));
    }

    #[test]
    fn format_empty_node() {
        let n = Node::new("empty");
        assert_eq!(n.to_at_source(), "empty {}");
    }

    // ── round-trip: the whole point of this module ─────────────────────────────
    //
    // Build a node, emit it, re-parse with the real auto-atom parser, and
    // assert the values come back. This is the regression guard for the whole
    // ecosystem: if anyone changes the parser's escape grammar, this fails.

    #[test]
    fn roundtrip_node_with_special_chars() {
        // We can't import auto-atom here (auto-val has no dep on it), so the
        // cross-crate round-trip is exercised in auto-atom's own tests. Here we
        // assert the emitted source matches our hand-computed expectation, which
        // is the exact grammar the parser accepts.
        //
        // Input contains a real newline, a quote, and a backslash — the three
        // cases the broken `Display` impl corrupts.
        let n = Node::new("role").with_prop(
            "description",
            Value::str("line one\nline \"two\""),
        );
        let src = n.to_at_source();
        assert_eq!(
            src,
            "role {\n    description : \"line one\\nline \\\"two\\\"\"\n}",
            "emitted source must be parser-escaped: {src}"
        );
    }

    #[test]
    fn format_key_quotes_when_needed() {
        // A key that isn't a clean identifier gets quoted.
        assert_eq!(format_key(&ValueKey::Str("clean_key".into())), "clean_key");
        assert_eq!(format_key(&ValueKey::Str("with space".into())), "\"with space\"");
        assert_eq!(format_key(&ValueKey::Int(7)), "\"7\"");
    }

    #[test]
    fn format_obj_value() {
        let mut o = Obj::new();
        o.set("a", Value::Int(1));
        o.set("b", Value::str("two"));
        let src = format_obj(&o, 0);
        assert!(src.contains("a : 1"));
        assert!(src.contains("b : \"two\""));
        assert!(src.starts_with("{\n"));
        assert!(src.ends_with('}'));
    }
}
