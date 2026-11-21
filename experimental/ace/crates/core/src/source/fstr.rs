//! PLAN-964 (L-22/FStr): the front-door f-string desugar.
//!
//! `f"..."` is not a grammar token — the old language lexed it as one
//! `FStrStart` token, but the ACE scanner only knows plain `str-lit`. The
//! desugar runs on the SOURCE TEXT before the grammar parses: every
//! `f"lit $ident lit2 ${expr} lit3"` becomes an `__fstr(...)` call whose
//! parts are ordinary string literals (raw text, escapes preserved) plus
//! the raw `$ident` / `${expr}` source fragments — the main parser then
//! parses the interpolation expressions naturally (no sub-parser needed).
//!
//! The `__fstr` builtin resolver (resolve.rs) flattens the parts into the
//! TextConcat chain with TextDisplay for `int` parts, in source order,
//! each interpolation evaluated exactly once (the concat chain is
//! left-associative — side effects run left-to-right).
//!
//! Known M2-A boundary: the transform rewrites the source, so spans INSIDE
//! an f-string may shift (diagnostics still identify the construct; spans
//! outside any f-string are byte-exact). The ordinary surface also admits
//! quoted literals inside `${...}` interpolation expressions. `\$` is a
//! literal dollar.


use std::ops::Range;

use super::SourceProfile;

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}
fn is_ident_char(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

/// One split part of an f-string: literal text (raw bytes) or an
/// interpolation source fragment.
pub(crate) enum Part {
    Text(String, Range<usize>),
    Expr(Range<usize>),
}

/// Split one inline formatted literal while retaining its original byte ranges.
pub(crate) fn split_fstring(src: &str) -> Result<Vec<Part>, String> {
    split_fstring_dialect(src, 0, false, false).map(|(parts, _)| parts)
}

/// Scan an f-string starting at `f"` in `src[i..]`; return the split parts
/// and the byte length consumed (through the closing quote).
/// PLAN-968: the scheduler face additionally admits the task sources'
/// legacy bare-brace interpolation `{ident}` (idents only); the shared
/// `$ident` / `${expr}` dialect stays byte-identical everywhere else.
fn split_fstring_dialect(
    src: &str,
    i: usize,
    bare_braces: bool,
    quoted_interpolations: bool,
) -> Result<(Vec<Part>, usize), String> {
    let bytes = src.as_bytes();
    debug_assert!(bytes[i] == b'f');
    split_quoted_dialect(src, i + 2, b'"', bare_braces, quoted_interpolations, i)
}

/// PLAN-980（B-DIALECT-I）：widget 方言反引号 f-string 的分割（`` `text ${expr}` `
/// ——auto-ui 传统工程形）。`start` 指向开引号后首字节；`origin` 仅为长度合计。
fn split_widget_fstring(src: &str, start: usize) -> Result<(Vec<Part>, usize), String> {
    let bytes = src.as_bytes();
    debug_assert!(bytes[start - 1] == b'`');
    split_quoted_dialect(src, start, b'`', false, false, start - 1)
}

fn split_quoted_dialect(
    src: &str,
    start: usize,
    quote: u8,
    bare_braces: bool,
    quoted_interpolations: bool,
    origin: usize,
) -> Result<(Vec<Part>, usize), String> {
    let bytes = src.as_bytes();
    let mut j = start;
    let mut parts: Vec<Part> = Vec::new();
    let mut text = String::new();
    let mut text_start = start;
    let mut closed = false;
    while j < bytes.len() {
        let c = bytes[j];
        match c {
            b'\\' => {
                // escape: copy the escape verbatim (preserves the original
                // decoding); `\$` becomes a literal-dollar pair `\\$`.
                if j + 1 < bytes.len() && bytes[j + 1] == b'$' {
                    text.push('\\');
                    text.push('\\');
                    text.push('$');
                    j += 2;
                } else if j + 1 < bytes.len() {
                    let ch = src[j + 1..].chars().next().unwrap();
                    text.push('\\');
                    text.push(ch);
                    j += 1 + ch.len_utf8();
                } else {
                    j += 1;
                }
            }
            b'$' => {
                if !text.is_empty() {
                    parts.push(Part::Text(std::mem::take(&mut text), text_start..j));
                }
                // `$ident`
                if let Some(n) = bytes.get(j + 1) {
                    if is_ident_start(*n) {
                        let mut k = j + 1;
                        while k < bytes.len() && is_ident_char(bytes[k]) {
                            k += 1;
                        }
                        parts.push(Part::Expr(j + 1..k));
                        j = k;
                        text_start = j;
                        continue;
                    }
                }
                // `${ expr }`
                if bytes.get(j + 1) == Some(&b'{') {
                    let mut depth = 1u32;
                    let mut k = j + 2;
                    while k < bytes.len() {
                        match bytes[k] {
                            b'{' => depth += 1,
                            b'}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            q @ (b'"' | b'\'') if quoted_interpolations => {
                                let end = quoted_end(src, k, q).ok_or_else(|| {
                                    "unterminated quoted literal in an FStr interpolation".to_owned()
                                })?;
                                k = end;
                                continue;
                            }
                            b'"' => {
                                // no string literals inside `${...}` in this
                                // slice (a plain quote would fight the outer
                                // closing-quote scan).
                                return Err(
                                    "a quoted string inside an FStr interpolation is not in this profile"
                                        .to_string(),
                                );
                            }
                            _ => {}
                        }
                        let ch = src[k..].chars().next().unwrap();
                        k += ch.len_utf8();
                    }
                    if depth != 0 || k >= bytes.len() {
                        return Err("unterminated `${` interpolation in an FStr".to_string());
                    }
                    parts.push(Part::Expr(j + 2..k));
                    j = k + 1;
                    text_start = j;
                    continue;
                }
                // a bare `$` not followed by ident or `{` is a literal.
                text_start = j;
                text.push('$');
                j += 1;
            }
            c if c == quote => {
                if !text.is_empty() {
                    parts.push(Part::Text(std::mem::take(&mut text), text_start..j));
                }
                closed = true;
                j += 1;
                break;
            }
            // PLAN-968: the scheduler face's legacy `{ident}` form.
            b'{' if bare_braces && bytes.get(j + 1).is_some_and(|n| is_ident_start(*n)) => {
                let mut k = j + 1;
                while k < bytes.len() && is_ident_char(bytes[k]) {
                    k += 1;
                }
                if bytes.get(k) == Some(&b'}') {
                    if !text.is_empty() {
                        parts.push(Part::Text(std::mem::take(&mut text), text_start..j));
                    }
                    parts.push(Part::Expr(j + 1..k));
                    j = k + 1;
                    text_start = j;
                    continue;
                }
                text.push('{');
                j += 1;
            }
            _ => {
                let ch = src[j..].chars().next().unwrap();
                text.push(ch);
                j += ch.len_utf8();
            }
        }
    }
    if !closed {
        return Err("unterminated f-string".to_string());
    }
    if parts.is_empty() {
        // An empty f-string `f""` still yields one empty literal part.
        parts.push(Part::Text(String::new(), start..start));
    }
    Ok((parts, j - origin))
}

/// Desugar every `f"..."` in `src`. A parse-stage error string on a
/// malformed f-string; the caller maps it to a `source.fstr-syntax`
/// diagnostic.
/// PLAN-968: the scheduler face's f-string desugar (adds the legacy
/// bare-brace `{ident}` interpolation of the task sources).
pub fn desugar_fstrings_sched(src: &str) -> Result<String, String> {
    desugar_fstrings_dialect(src, true, false, None)
}

pub fn desugar_fstrings(src: &str) -> Result<String, String> {
    desugar_fstrings_dialect(src, false, false, None)
}

pub(super) struct SourceRewrite {
    original: std::ops::Range<usize>,
    projected: std::ops::Range<usize>,
}

pub(super) fn restore_span(
    edits: &[SourceRewrite],
    span: crate::atom_text::Span,
) -> crate::atom_text::Span {
    let position = |offset: usize, end: bool| {
        let count = edits.partition_point(|edit| edit.projected.start <= offset);
        let Some(edit) = count.checked_sub(1).map(|index| &edits[index]) else {
            return offset;
        };
        if offset == edit.projected.start {
            edit.original.start
        } else if offset < edit.projected.end {
            if end {
                edit.original.end
            } else {
                edit.original.start
            }
        } else {
            edit.original.end + offset - edit.projected.end
        }
    };
    crate::atom_text::Span {
        start: position(span.start, false),
        end: position(span.end, true),
    }
}

pub(super) fn desugar_fstrings_auto(src: &str) -> Result<(String, Vec<SourceRewrite>), String> {
    let mut edits = Vec::new();
    let output = desugar_fstrings_dialect(src, false, true, Some(&mut edits))?;
    Ok((output, edits))
}

fn quoted_end(src: &str, start: usize, quote: u8) -> Option<usize> {
    let mut pos = start + 1;
    let bytes = src.as_bytes();
    while let Some(&c) = bytes.get(pos) {
        pos += 1;
        if c == quote {
            return Some(pos);
        }
        if c == b'\\' {
            pos += src.get(pos..)?.chars().next()?.len_utf8();
        }
    }
    None
}

/// PLAN-980（B-DIALECT-I）：widget 工程源的反引号 f-string desugar。逐字节扫描：
/// 跳过行注释/普通字符串/字符字面量/`f"` 形，把每个 `` `...` `` 重写为
/// `__fstr(...)`（与 `f"..."` 同一生成物）。与 PLAN-964 同边界：f-string 内部
/// span 可能移位（诊断仍指认构造）；外部字节保持。
pub fn desugar_fstrings_widget(src: &str) -> Result<String, String> {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'/' && bytes.get(i + 1) == Some(&b'/') {
            let end = src[i..]
                .find('\n')
                .map(|p| i + p + 1)
                .unwrap_or(bytes.len());
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        if c == b'"' {
            let mut j = i + 1;
            while j < bytes.len() {
                match bytes[j] {
                    b'\\' => {
                        j += 1;
                        if j < bytes.len() {
                            j += src[j..].chars().next().unwrap().len_utf8();
                        }
                    }
                    b'"' => {
                        j += 1;
                        break;
                    }
                    _ => j += 1,
                }
            }
            let j = j.min(bytes.len());
            out.push_str(&src[i..j]);
            i = j;
            continue;
        }
        if c == b'\'' {
            let mut j = i + 1;
            while j < bytes.len() {
                match bytes[j] {
                    b'\\' => {
                        j += 1;
                        if j < bytes.len() {
                            j += src[j..].chars().next().unwrap().len_utf8();
                        }
                    }
                    b'\'' => {
                        j += 1;
                        break;
                    }
                    _ => j += 1,
                }
            }
            let j = j.min(bytes.len());
            out.push_str(&src[i..j]);
            i = j;
            continue;
        }
        if c == b'`' {
            let (parts, _len) = match split_widget_fstring(src, i + 1) {
                Ok(x) => x,
                Err(e) => return Err(e),
            };
            out.push_str("__fstr(");
            let mut first = true;
            for p in parts {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                match p {
                    Part::Text(t, _) => {
                        out.push('"');
                        for ch in t.chars() {
                            if ch == '"' || ch == '\\' {
                                out.push('\\');
                            }
                            out.push(ch);
                        }
                        out.push('"');
                    }
                    Part::Expr(range) => out.push_str(&src[range]),
                }
            }
            out.push(')');
            // advance past the closing backtick
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'`' {
                j += 1;
            }
            i = (j + 1).min(bytes.len());
            continue;
        }
        let ch = src[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    Ok(out)
}

fn desugar_fstrings_dialect(
    src: &str,
    bare_braces: bool,
    quoted_interpolations: bool,
    mut edits: Option<&mut Vec<SourceRewrite>>,
) -> Result<String, String> {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        // line comment — copy verbatim
        if c == b'/' && bytes.get(i + 1) == Some(&b'/') {
            let end = src[i..]
                .find('\n')
                .map(|p| i + p + 1)
                .unwrap_or(bytes.len());
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        // plain string literal — copy verbatim (may contain `f"` text)
        if c == b'"' {
            let mut j = i + 1;
            while j < bytes.len() {
                match bytes[j] {
                    b'\\' => {
                        j += 1;
                        if j < bytes.len() {
                            j += src[j..].chars().next().unwrap().len_utf8();
                        }
                    }
                    b'"' => {
                        j += 1;
                        break;
                    }
                    _ => j += 1,
                }
            }
            if j > bytes.len() {
                j = bytes.len();
            }
            out.push_str(&src[i..j]);
            i = j;
            continue;
        }
        // f-string start: `f"` with a non-ident char before the `f`
        if c == b'f' && bytes.get(i + 1) == Some(&b'"') && (i == 0 || !is_ident_char(bytes[i - 1]))
        {
            let (parts, len) =
                match split_fstring_dialect(src, i, bare_braces, quoted_interpolations) {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                };
            let projected_start = out.len();
            out.push_str("__fstr(");
            let mut first = true;
            for p in parts {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                match p {
                    Part::Text(t, _) => {
                        out.push('"');
                        out.push_str(&t);
                        out.push('"');
                    }
                    Part::Expr(range) => out.push_str(&src[range]),
                }
            }
            if first {
                // an empty f-string `f""` still yields one empty literal.
                out.push_str("\"\"");
            }
            out.push(')');
            if let Some(edits) = edits.as_deref_mut() {
                edits.push(SourceRewrite {
                    original: i..i + len,
                    projected: projected_start..out.len(),
                });
            }
            i += len;
            continue;
        }
        let ch = src[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    Ok(out)
}

/// PLAN-982（B-CONFIG-I，V-CFG-TEMPLATE）：配置文档的模板插值 desugar。
///
/// 真实 `pac.at` 的写法是**普通字符串**里的 `${expr}`（legacy config 同样插值），
/// 以及节点头里的裸 `${expr}`（`lib ${name} {}`）。本函数把它们重写成与
/// `f"..."` **同一生成物** `__fstr(...)`：
/// - 字符串里的 `${…}` → `__fstr("前缀", expr, "后缀")`（前缀/后缀是原样文本）；
/// - 字符串外的裸 `${…}` → `__fstr("", expr)`（节点头因此成为"节点名 + 一个
///   文本实参"，与 legacy 的 `_arg0` 形状一致）。
///
/// 边界与既有 f-string 同一纪律：`${…}` 内不嵌套花括号、不含字符串字面量；
/// `\$` 是字面美元。行注释（`//`、config 面的 `#`）、块注释与字符字面量按原样
/// 复制（不误插值）。重写会移动 span（构造级诊断仍指认构造）。
/// PLAN-982（用户裁定 ①B）：节点位插值的**显式开关**——默认关，保持与 legacy
/// 逐字一致；`AUTO_ACE_TEMPLATE_NODE_INTERP=1|true|yes|ace` 时节点位的裸 `${…}`
/// 也重写成 `__fstr(expr)`，使 `lib ${name} {}` 真正插值。
pub const CONFIG_NODE_INTERP_ENV: &str = "AUTO_ACE_TEMPLATE_NODE_INTERP";

fn node_interp_enabled() -> bool {
    matches!(
        std::env::var(CONFIG_NODE_INTERP_ENV).as_deref(),
        Ok("1") | Ok("true") | Ok("yes") | Ok("ace")
    )
}

pub fn desugar_config_templates(src: &str) -> Result<String, String> {
    fn part_text(out: &mut String, t: &str) {
        out.push('"');
        out.push_str(t);
        out.push('"');
    }
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        // 行注释（`//` 与 config 面的 `#`）原样复制。
        if (c == b'/' && bytes.get(i + 1) == Some(&b'/')) || c == b'#' {
            let end = src[i..]
                .find('\n')
                .map(|p| i + p + 1)
                .unwrap_or(bytes.len());
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        // 块注释原样复制。
        if c == b'/' && bytes.get(i + 1) == Some(&b'*') {
            let end = src[i..]
                .find("*/")
                .map(|p| i + p + 2)
                .unwrap_or(bytes.len());
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        // 字符字面量原样复制。
        if c == b'\'' {
            let mut j = i + 1;
            while j < bytes.len() {
                match bytes[j] {
                    b'\\' => {
                        j += 1;
                        if j < bytes.len() {
                            j += src[j..].chars().next().unwrap().len_utf8();
                        }
                    }
                    b'\'' => {
                        j += 1;
                        break;
                    }
                    _ => j += 1,
                }
            }
            out.push_str(&src[i..j]);
            i = j;
            continue;
        }
        // 普通字符串：含未转义 `${` 时变成 `__fstr(...)`。
        if c == b'"' {
            let mut j = i + 1;
            let mut has_tpl = false;
            while j < bytes.len() {
                match bytes[j] {
                    b'\\' => {
                        j += 1;
                        if j < bytes.len() {
                            j += src[j..].chars().next().unwrap().len_utf8();
                        }
                    }
                    b'$' => {
                        if bytes.get(j + 1) == Some(&b'{') {
                            has_tpl = true;
                        }
                        j += 1;
                    }
                    b'"' => {
                        j += 1;
                        break;
                    }
                    _ => j += 1,
                }
            }
            if !has_tpl {
                out.push_str(&src[i..j]);
                i = j;
                continue;
            }
            let (parts, len) = split_quoted_dialect(src, i + 1, b'"', false, false, i)?;
            out.push_str("__fstr(");
            let mut first = true;
            for p in parts {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                match p {
                    Part::Text(t, _) => part_text(&mut out, &t),
                    Part::Expr(range) => out.push_str(&src[range]),
                }
            }
            if first {
                out.push_str("\"\"");
            }
            out.push(')');
            i += len;
            continue;
        }
        // 字符串外的裸 `${expr}`：节点头里的模板占位。legacy config 在**节点头**
        // 不插值（实测 `lib ${name} {}` 的 id/`_arg0` 都是字面 `${name}`），所以
        // 这里重写成同文本的字符串字面量——head 变成普通字符串身份，投影层再按
        // `${…}` 形补一个 `_arg0` 实参（与 legacy 逐字一致）。
        if c == b'$' && bytes.get(i + 1) == Some(&b'{') {
            let mut depth = 0usize;
            let mut j = i + 1;
            let mut close = None;
            while j < bytes.len() {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(j);
                            break;
                        }
                    }
                    b'"' | b'\'' => {
                        return Err("`${…}` segments may not contain string literals".to_string())
                    }
                    _ => {}
                }
                j += 1;
            }
            let Some(close) = close else {
                return Err("unterminated `${…}` template segment".to_string());
            };
            // 位置决定语义（legacy 口径）：`key: ${x}` 是**取值位的插值**；
            // 节点头 `name ${x} {}` 是字面占位（legacy 实测不插值）。
            let mut back = i;
            while back > 0 && matches!(bytes[back - 1], b' ' | b'\t') {
                back -= 1;
            }
            let value_position = back > 0 && bytes[back - 1] == b':';
            if value_position {
                out.push_str("__fstr(");
                out.push_str(&src[i + 2..close]);
                out.push(')');
            } else if node_interp_enabled() {
                // PLAN-982（用户裁定 ①B）：开关打开时**节点位也插值**——与取值位
                // 同一生成物 `__fstr(expr)`；默认走下方字面量分支（legacy 对齐）。
                out.push_str("__fstr(");
                out.push_str(&src[i + 2..close]);
                out.push(')');
            } else {
                out.push('"');
                out.push_str(&src[i..=close]);
                out.push('"');
            }
            i = close + 1;
            continue;
        }
        let ch = src[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    Ok(out)
}
/// The profiles whose surfaces admit FStr (the textual family plus the
/// aggregates/closures collections surface — the FStr case corpus runs on
/// the aggregates compile entry).
pub fn profile_admits_fstr(profile: SourceProfile) -> bool {
    matches!(
        profile,
        SourceProfile::Text
            | SourceProfile::TextRecord
            | SourceProfile::HostConsole
            | SourceProfile::Collections
            | SourceProfile::Closures
            | SourceProfile::Numeric
            | SourceProfile::Aggregates
            | SourceProfile::Scheduler
            // PLAN-1078：Auto 面（aggregates 身份）同走 f-string 变换。
            | SourceProfile::Auto
            // PLAN-982（B-CONFIG-I）：配置文档模板插值走同一 f-string 变换。
            | SourceProfile::ConfigDocument
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fstr_basic_parts() {
        let src = r#"let s = f"Hello, $name""#;
        assert_eq!(
            desugar_fstrings(src).unwrap(),
            r#"let s = __fstr("Hello, ", name)"#
        );
    }

    #[test]
    fn fstr_expression_parts() {
        let src = r#"let s = f"Sum: ${a + b}, Product: ${a * b}""#;
        assert_eq!(
            desugar_fstrings(src).unwrap(),
            r#"let s = __fstr("Sum: ", a + b, ", Product: ", a * b)"#
        );
    }

    #[test]
    fn fstr_ignores_plain_strings_and_comments() {
        let src = "// f\"not an fstr\"\nlet x = \"f\"\nlet s = f\"a $b\"";
        let out = desugar_fstrings(src).unwrap();
        assert!(out.contains("// f\"not an fstr\""));
        assert!(out.contains("let x = \"f\""));
        assert!(out.contains("let s = __fstr(\"a \", b)"));
    }

    #[test]
    fn fstr_dollar_escape_is_literal() {
        let src = r#"let s = f"cost: \$5""#;
        assert_eq!(
            desugar_fstrings(src).unwrap(),
            r#"let s = __fstr("cost: \\$5")"#
        );
    }

    #[test]
    fn fstr_unterminated_rejects() {
        assert!(desugar_fstrings(r#"let s = f"no close"#).is_err());
    }

    #[test]
    fn fstr_empty() {
        assert_eq!(
            desugar_fstrings("let s = f\"\"").unwrap(),
            "let s = __fstr(\"\")"
        );
    }

    #[test]
    fn fstr_preserves_utf8_chars_and_char_literals() {
        let src = "fn main() int {\n print('界')\n eprint('😀')\n let s = f\"你好 ${x} 😀\"\n return 0\n}\n";
        let out = desugar_fstrings(src).unwrap();
        assert_eq!(
            out,
            "fn main() int {\n print('界')\n eprint('😀')\n let s = __fstr(\"你好 \", x, \" 😀\")\n return 0\n}\n"
        );
    }
}

/// PLAN-105 T-02（U03 真语料）：widget 工程源**统一** f-string 脱糖入口——
/// 先反引号形（`` `text ${expr}` ``），再 `f"…"` 形。此前 component/包片段
/// 车道只脱反引号形，传统 AutoUI 源（012/013/016 的 `f"…${.x}…"`）解析期
/// 直接回滚。span 边界沿 PLAN-964：f-string 内部可能移位，外部字节保持。
pub fn desugar_widget_source(src: &str) -> Result<String, String> {
    let backticked = desugar_fstrings_widget(src)?;
    desugar_fstrings(&backticked)
}
