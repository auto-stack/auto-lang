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
//! outside any f-string are byte-exact). `${...}` segments must not nest
//! braces or contain string literals in this slice (`source.fstr-syntax`
//! rejects them). `\$` is a literal dollar.

use super::SourceProfile;

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}
fn is_ident_char(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

/// One split part of an f-string: literal text (raw bytes) or an
/// interpolation source fragment.
enum Part {
    Text(String),
    Ident(String),
    Expr(String),
}

/// Scan an f-string starting at `f"` in `src[i..]`; return the split parts
/// and the byte length consumed (through the closing quote).
fn split_fstring(src: &str, i: usize) -> Result<(Vec<Part>, usize), String> {
    let bytes = src.as_bytes();
    debug_assert!(bytes[i] == b'f');
    let mut j = i + 2; // past `f"`
    let mut parts: Vec<Part> = Vec::new();
    let mut text = String::new();
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
                } else {
                    if let Some(e) = bytes.get(j + 1) {
                        text.push('\\');
                        text.push(*e as char);
                    }
                    j += 2;
                }
            }
            b'$' => {
                if !text.is_empty() {
                    parts.push(Part::Text(std::mem::take(&mut text)));
                }
                // `$ident`
                if let Some(n) = bytes.get(j + 1) {
                    if is_ident_start(*n) {
                        let mut k = j + 1;
                        while k < bytes.len() && is_ident_char(bytes[k]) {
                            k += 1;
                        }
                        parts.push(Part::Ident(src[j + 1..k].to_string()));
                        j = k;
                        continue;
                    }
                }
                // `${ expr }`
                if bytes.get(j + 1) == Some(&b'{') {
                    let mut depth = 1u32;
                    let mut k = j + 2;
                    let mut expr = String::new();
                    while k < bytes.len() {
                        match bytes[k] {
                            b'{' => depth += 1,
                            b'}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
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
                        expr.push(bytes[k] as char);
                        k += 1;
                    }
                    if depth != 0 || k >= bytes.len() {
                        return Err("unterminated `${` interpolation in an FStr".to_string());
                    }
                    parts.push(Part::Expr(expr));
                    j = k + 1;
                    continue;
                }
                // a bare `$` not followed by ident or `{` is a literal.
                text.push('$');
                j += 1;
            }
            b'"' => {
                if !text.is_empty() {
                    parts.push(Part::Text(std::mem::take(&mut text)));
                }
                closed = true;
                j += 1;
                break;
            }
            _ => {
                text.push(c as char);
                j += 1;
            }
        }
    }
    if !closed {
        return Err("unterminated f-string".to_string());
    }
    if parts.is_empty() {
        // An empty f-string `f""` still yields one empty literal part.
        parts.push(Part::Text(String::new()));
    }
    Ok((parts, j - i))
}

/// Desugar every `f"..."` in `src`. A parse-stage error string on a
/// malformed f-string; the caller maps it to a `source.fstr-syntax`
/// diagnostic.
pub fn desugar_fstrings(src: &str) -> Result<String, String> {
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
                    b'\\' => j += 2,
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
        if c == b'f'
            && bytes.get(i + 1) == Some(&b'"')
            && (i == 0 || !is_ident_char(bytes[i - 1]))
        {
            let (parts, len) = match split_fstring(src, i) {
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
                    Part::Text(t) => {
                        out.push('"');
                        out.push_str(&t);
                        out.push('"');
                    }
                    Part::Ident(name) => out.push_str(&name),
                    Part::Expr(e) => out.push_str(&e),
                }
            }
            if first {
                // an empty f-string `f""` still yields one empty literal.
                out.push_str("\"\"");
            }
            out.push(')');
            i += len;
            continue;
        }
        out.push(c as char);
        i += 1;
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
        assert_eq!(desugar_fstrings(src).unwrap(), r#"let s = __fstr("cost: \\$5")"#);
    }

    #[test]
    fn fstr_unterminated_rejects() {
        assert!(desugar_fstrings(r#"let s = f"no close"#).is_err());
    }

    #[test]
    fn fstr_empty() {
        assert_eq!(desugar_fstrings("let s = f\"\"").unwrap(), "let s = __fstr(\"\")");
    }
}
