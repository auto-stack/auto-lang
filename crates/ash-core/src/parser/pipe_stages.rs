//! Parser for shell-DSL pipeline stages (Plan 320).
//!
//! Detects and parses structured-pipeline DSL stages from pipe text:
//!   `.field op value`  → Filter
//!   `.field`           → Map (projection)
//!   `sort .field [desc]` → SortBy
//!   `select .f1 .f2`   → Select
//!   `first N` / `last N` → Take / SkipBack
//!   `count`            → Count
//!
//! Returns `None` for regular commands (they go through the normal command dispatch).

use auto_val::Value;

use crate::pipeline::operators::{CmpOp, PipelineOp};

/// Try to parse a pipe stage text as a [`PipelineOp`].
/// Returns `None` if the text is a regular command (not a DSL stage).
pub fn parse_pipe_stage(text: &str) -> Option<PipelineOp> {
    let text = text.trim();

    // DSL commands.
    if text == "count" {
        return Some(PipelineOp::Count);
    }
    if let Some(rest) = text.strip_prefix("sort ").or_else(|| text.strip_prefix("sort-by ")) {
        return parse_sort(rest);
    }
    if let Some(rest) = text.strip_prefix("first ") {
        return parse_take(rest, false);
    }
    if let Some(rest) = text.strip_prefix("last ") {
        return parse_take(rest, true);
    }
    if let Some(rest) = text.strip_prefix("select ") {
        return parse_select(rest);
    }

    // .field ...
    if text.starts_with('.') {
        return parse_dot_stage(text);
    }

    None // Not a DSL stage → regular command.
}

/// Parse `.field op value` or `.field` (bare projection).
fn parse_dot_stage(text: &str) -> Option<PipelineOp> {
    // Extract the field name after the leading '.'.
    let after_dot = &text[1..];
    let field: String = after_dot
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if field.is_empty() {
        return None;
    }

    let rest = after_dot[field.len()..].trim_start();
    if rest.is_empty() {
        // Bare `.field` → projection.
        return Some(PipelineOp::Map { field });
    }

    // `.field op value` → filter.
    let (op_str, value_str) = split_operator(rest)?;
    let op = CmpOp::from_str(op_str)?;
    let value = parse_value(value_str.trim())?;

    Some(PipelineOp::Filter { field, op, value })
}

/// Parse `sort .field [asc|desc]`.
fn parse_sort(rest: &str) -> Option<PipelineOp> {
    let rest = rest.trim();
    // Expect `.field [direction]`.
    if !rest.starts_with('.') {
        return None;
    }
    let after_dot = &rest[1..];
    let field: String = after_dot
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if field.is_empty() {
        return None;
    }
    let dir = after_dot[field.len()..].trim();
    let descending = dir.eq_ignore_ascii_case("desc")
        || dir.eq_ignore_ascii_case("descending");
    Some(PipelineOp::SortBy { field, descending })
}

/// Parse `select .f1 .f2 ...`.
fn parse_select(rest: &str) -> Option<PipelineOp> {
    let fields: Vec<String> = rest
        .split_whitespace()
        .filter_map(|tok| tok.strip_prefix('.').map(|s| s.to_string()))
        .collect();
    if fields.is_empty() {
        return None;
    }
    Some(PipelineOp::Select { fields })
}

/// Parse `first N` / `last N` → Take / SkipBack.
fn parse_take(rest: &str, is_last: bool) -> Option<PipelineOp> {
    let n: usize = rest.trim().parse().ok()?;
    Some(if is_last {
        PipelineOp::SkipBack(n)
    } else {
        PipelineOp::Take(n)
    })
}

/// Split a predicate into `(operator, value_string)`.
/// Tries multi-char operators first (>=, <=, ==, !=), then single (>, <).
fn split_operator(text: &str) -> Option<(&str, &str)> {
    // Word operators: contains, starts-with, ends-with.
    for word_op in &["contains", "starts-with", "ends-with"] {
        if let Some(pos) = text.find(word_op) {
            let before = &text[..pos];
            let after = &text[pos + word_op.len()..];
            // Ensure there's whitespace or boundary before the word op.
            if before.is_empty() || before.ends_with(' ') {
                return Some((word_op, after.trim_start()));
            }
        }
    }
    // Multi-char symbols: >=, <=, ==, !=
    for sym_op in &["<=", ">=", "==", "!="] {
        if let Some(pos) = text.find(sym_op) {
            return Some((sym_op, text[pos + sym_op.len()..].trim_start()));
        }
    }
    // Single-char symbols: >, <
    // Must not be preceded by another symbol (avoid matching >>= etc).
    for sym_op in &["<", ">"] {
        if let Some(pos) = text.find(sym_op) {
            // Check it's not part of a two-char op (already handled above).
            let before_ok = pos == 0 || !text.as_bytes().get(pos - 1).is_some_and(|&b| b == b'<' || b == b'>' || b == b'=' || b == b'!');
            if before_ok {
                return Some((sym_op, text[pos + 1..].trim_start()));
            }
        }
    }
    None
}

/// Parse a value token: string, number (with optional unit), or bool.
fn parse_value(s: &str) -> Option<Value> {
    let s = s.trim();

    // Quoted string.
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return Some(Value::str(&s[1..s.len() - 1]));
    }
    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        return Some(Value::str(&s[1..s.len() - 1]));
    }

    // Boolean.
    if s == "true" {
        return Some(Value::Bool(true));
    }
    if s == "false" {
        return Some(Value::Bool(false));
    }

    // Number with unit: 10.mb, 5.kb, 1.gb, 2.tb
    if let Some(v) = parse_number_with_unit(s) {
        return Some(v);
    }

    // Plain integer.
    if let Ok(n) = s.parse::<i64>() {
        return Some(Value::I64(n));
    }
    // Plain float.
    if let Ok(f) = s.parse::<f64>() {
        return Some(Value::Float(f));
    }

    // Bare string (no quotes) — treat as string literal.
    Some(Value::str(s))
}

/// Parse `10.mb` → 10 * 1024 * 1024. Returns None if no unit suffix.
fn parse_number_with_unit(s: &str) -> Option<Value> {
    let lower = s.to_ascii_lowercase();
    for (suffix, multiplier) in [
        (".tb", 1024u64 * 1024 * 1024 * 1024),
        (".gb", 1024u64 * 1024 * 1024),
        (".mb", 1024u64 * 1024),
        (".kb", 1024u64),
    ] {
        if let Some(num_str) = lower.strip_suffix(suffix) {
            if let Ok(n) = num_str.parse::<u64>() {
                return Some(Value::I64((n * multiplier) as i64));
            }
            if let Ok(f) = num_str.parse::<f64>() {
                return Some(Value::I64((f * multiplier as f64) as i64));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_filter_gt() {
        let op = parse_pipe_stage(".size > 10").unwrap();
        match op {
            PipelineOp::Filter { field, op, value } => {
                assert_eq!(field, "size");
                assert_eq!(op, CmpOp::Gt);
                assert!(matches!(value, Value::I64(10)));
            }
            _ => panic!("expected Filter"),
        }
    }

    #[test]
    fn parse_filter_with_unit() {
        let op = parse_pipe_stage(".size > 10.mb").unwrap();
        match op {
            PipelineOp::Filter { value, .. } => {
                assert!(matches!(value, Value::I64(n) if n == 10_485_760));
            }
            _ => panic!("expected Filter"),
        }
    }

    #[test]
    fn parse_filter_string_eq() {
        let op = parse_pipe_stage(".type == \"dir\"").unwrap();
        match op {
            PipelineOp::Filter { field, op, value } => {
                assert_eq!(field, "type");
                assert_eq!(op, CmpOp::Eq);
                // Value::Str.to_string() may add quotes; check substring.
                assert!(value.to_string().contains("dir"), "value: {}", value.to_string());
            }
            _ => panic!("expected Filter"),
        }
    }

    #[test]
    fn parse_filter_contains() {
        let op = parse_pipe_stage(".name contains test").unwrap();
        match op {
            PipelineOp::Filter { field, op, .. } => {
                assert_eq!(field, "name");
                assert_eq!(op, CmpOp::Contains);
            }
            _ => panic!("expected Filter"),
        }
    }

    #[test]
    fn parse_bare_field_map() {
        let op = parse_pipe_stage(".name").unwrap();
        match op {
            PipelineOp::Map { field } => assert_eq!(field, "name"),
            _ => panic!("expected Map"),
        }
    }

    #[test]
    fn parse_sort() {
        let op = parse_pipe_stage("sort .modified").unwrap();
        match op {
            PipelineOp::SortBy { field, descending } => {
                assert_eq!(field, "modified");
                assert!(!descending);
            }
            _ => panic!("expected SortBy"),
        }
        let op = parse_pipe_stage("sort-by .size desc").unwrap();
        match op {
            PipelineOp::SortBy { field, descending } => {
                assert_eq!(field, "size");
                assert!(descending);
            }
            _ => panic!("expected SortBy"),
        }
    }

    #[test]
    fn parse_select() {
        let op = parse_pipe_stage("select .name .size .type").unwrap();
        match op {
            PipelineOp::Select { fields } => {
                assert_eq!(fields, vec!["name", "size", "type"]);
            }
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn parse_first_last_count() {
        assert!(matches!(parse_pipe_stage("first 5"), Some(PipelineOp::Take(5))));
        assert!(matches!(parse_pipe_stage("last 3"), Some(PipelineOp::SkipBack(3))));
        assert!(matches!(parse_pipe_stage("count"), Some(PipelineOp::Count)));
    }

    #[test]
    fn regular_command_returns_none() {
        assert!(parse_pipe_stage("grep foo").is_none());
        assert!(parse_pipe_stage("ls -la").is_none());
        assert!(parse_pipe_stage("echo hello").is_none());
        assert!(parse_pipe_stage("sort").is_none()); // bare sort without field
    }
}
