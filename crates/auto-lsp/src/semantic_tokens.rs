//! Plan 416 5-B: semantic tokens (server side).
//!
//! `textDocument/semanticTokens/full` — a self-contained scanner that merges
//! two sources:
//!   * lexical pass: comments / strings / numbers / keywords (the lexer's
//!     authoritative `Token::all_keywords` list from Plan 416 5-C);
//!   * symbol classification from the parsed AST: fn declarations → function,
//!     parameters → parameter, `let`/`var` locals → variable, type/enum/spec
//!     declarations → type.
//!
//! Identifiers not classified by the AST fall back to heuristics: an ident
//! immediately followed by `(` reads as a function call; a capitalized ident
//! reads as a type. VSCode renders these on top of the TextMate grammar —
//! the extension side needs no changes (vscode-languageclient auto-registers
//! the feature when the server advertises the capability).

use tower_lsp_server::ls_types::*;

/// Legend order IS the wire encoding (token type = index).
pub const TOKEN_TYPES: &[&str] = &[
    "keyword",
    "type",
    "function",
    "variable",
    "parameter",
    "string",
    "number",
    "comment",
];

mod tt {
    pub const KEYWORD: u32 = 0;
    pub const TYPE: u32 = 1;
    pub const FUNCTION: u32 = 2;
    pub const VARIABLE: u32 = 3;
    pub const PARAMETER: u32 = 4;
    pub const STRING: u32 = 5;
    pub const NUMBER: u32 = 6;
    pub const COMMENT: u32 = 7;
}

/// Absolute token (line, char are UTF-16 code units per LSP).
#[derive(Debug, Clone, PartialEq)]
struct AbsTok {
    line: u32,
    char: u32,
    len: u32,
    ty: u32,
}

/// Classify identifiers via the parsed AST.
fn collect_ast_symbols(content: &str) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut functions = Vec::new();
    let mut types = Vec::new();
    let mut params = Vec::new();
    let mut locals = Vec::new();
    let Ok(ast) = auto_lang::parse_preserve_error(content) else {
        return (functions, types, params, locals);
    };
    #[derive(Default)]
    struct SymSets {
        functions: Vec<String>,
        types: Vec<String>,
        params: Vec<String>,
        locals: Vec<String>,
    }
    fn walk(stmts: &[auto_lang::ast::Stmt], out: &mut SymSets) {
        use auto_lang::ast::Stmt;
        for stmt in stmts {
            match stmt {
                Stmt::Fn(f) => {
                    out.functions.push(f.name.to_string());
                    for p in &f.params {
                        out.params.push(p.name.to_string());
                    }
                    walk(&f.body.stmts, out);
                }
                Stmt::TypeDecl(t) => out.types.push(t.name.to_string()),
                Stmt::EnumDecl(e) => out.types.push(e.name.to_string()),
                Stmt::SpecDecl(s) => out.types.push(s.name.to_string()),
                Stmt::Store(store) => {
                    // `let x =` / `var y =` locals (module vars included).
                    out.locals.push(store.name.to_string());
                }
                Stmt::For(f) => {
                    // Loop variables are locals.
                    match &f.iter {
                        auto_lang::ast::Iter::Indexed(_, v)
                        | auto_lang::ast::Iter::Named(v)
                        | auto_lang::ast::Iter::Destructured(_, v) => {
                            out.locals.push(v.to_string());
                        }
                        _ => {}
                    }
                    walk(&f.body.stmts, out);
                }
                Stmt::If(i) => {
                    for b in &i.branches {
                        walk(&b.body.stmts, out);
                    }
                    if let Some(eb) = &i.else_ {
                        walk(&eb.stmts, out);
                    }
                }
                Stmt::Block(b) => walk(&b.stmts, out),
                _ => {}
            }
        }
    }
    let mut sets = SymSets::default();
    walk(&ast.stmts, &mut sets);
    (sets.functions, sets.types, sets.params, sets.locals)
}

/// Scan the source into absolute tokens, merge with AST classes, and emit
/// LSP relative-encoded data (5 ints per token: deltaLine, deltaStart, len,
/// tokenType, tokenModifiers=0).
pub fn semantic_tokens_full(content: &str) -> Vec<SemanticToken> {
    let keywords: std::collections::HashSet<&str> = auto_lang::token::Token::all_keywords()
        .iter()
        .copied()
        .collect();
    let (functions, types, params, locals) = collect_ast_symbols(content);
    let fn_set: std::collections::HashSet<&str> = functions.iter().map(|s| s.as_str()).collect();
    let ty_set: std::collections::HashSet<&str> = types.iter().map(|s| s.as_str()).collect();
    let pa_set: std::collections::HashSet<&str> = params.iter().map(|s| s.as_str()).collect();
    let lo_set: std::collections::HashSet<&str> = locals.iter().map(|s| s.as_str()).collect();

    let mut toks: Vec<AbsTok> = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0usize;
    let mut line = 0u32;
    let mut col16 = 0u32; // UTF-16 code units on the current line

    let push = |toks: &mut Vec<AbsTok>, len16: u32, ty: u32, line: u32, col16: u32| {
        if len16 > 0 {
            toks.push(AbsTok {
                line,
                char: col16,
                len: len16,
                ty,
            });
        }
    };

    while i < bytes.len() {
        let b = bytes[i];
        // Track (line, col16) as we consume.
        let starts_line_comment = b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/';
        if starts_line_comment {
            let start_col = col16;
            let mut len = 0u32;
            while i < bytes.len() && bytes[i] != b'\n' {
                len += content[i..]
                    .chars()
                    .next()
                    .map(|c| c.len_utf16() as u32)
                    .unwrap_or(1);
                i += content[i..]
                    .chars()
                    .next()
                    .map(|c| c.len_utf8())
                    .unwrap_or(1);
            }
            push(&mut toks, len, tt::COMMENT, line, start_col);
            continue;
        }
        if b == b'"' || b == b'`' {
            let quote = b;
            let start_col = col16;
            let mut len = 0u32;
            // consume opening quote
            len += 1;
            i += 1;
            col16 += 1;
            while i < bytes.len() && bytes[i] != quote {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    // escape: count both chars roughly (2 utf16 units max)
                    len += 2;
                    i += 2;
                    col16 += 2;
                } else if bytes[i] == b'\n' {
                    // multi-line string: flush current segment, reset col
                    push(&mut toks, len, tt::STRING, line, start_col);
                    // re-open on the next line at col 0 with fresh len
                    len = 0;
                    col16 = 0;
                    line += 1;
                    i += 1;
                    // continuation token starts here; note: multiple segments
                    // are separate tokens, which is LSP-legal.
                    // (start_col continuation is handled by pushing at current col.)
                    push_continuation(
                        &mut toks,
                        tt::STRING,
                        line,
                        col16,
                        &mut len,
                        quote,
                        &mut i,
                        &mut col16,
                        content,
                    );
                    break;
                } else {
                    let c16 = content[i..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf16() as u32)
                        .unwrap_or(1);
                    let c8 = content[i..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    len += c16;
                    i += c8;
                    col16 += c16;
                }
            }
            // closing quote
            if i < bytes.len() && bytes[i] == quote {
                len += 1;
                i += 1;
                col16 += 1;
            }
            push(&mut toks, len, tt::STRING, line, start_col);
            continue;
        }
        if b.is_ascii_digit() {
            let start_col = col16;
            let mut len = 0u32;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.') {
                len += 1;
                i += 1;
                col16 += 1;
            }
            push(&mut toks, len, tt::NUMBER, line, start_col);
            continue;
        }
        if b.is_ascii_alphabetic() || b == b'_' {
            let start_col = col16;
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
                col16 += 1;
            }
            let word = &content[start..i];
            let len = (i - start) as u32;
            let ty = if keywords.contains(word) {
                tt::KEYWORD
            } else if fn_set.contains(word) {
                tt::FUNCTION
            } else if ty_set.contains(word) {
                tt::TYPE
            } else if pa_set.contains(word) {
                tt::PARAMETER
            } else if lo_set.contains(word) {
                tt::VARIABLE
            } else {
                // Heuristics: `word(` → call; capitalized → type.
                let next_nonws = content[i..].trim_start().chars().next();
                if next_nonws == Some('(') {
                    tt::FUNCTION
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    tt::TYPE
                } else {
                    continue; // unclassified ident: no token (TextMate base)
                }
            };
            push(&mut toks, len, ty, line, start_col);
            continue;
        }
        // Whitespace / other: advance one char, tracking line/col.
        let c = content[i..].chars().next().unwrap_or(' ');
        let c8 = c.len_utf8();
        let c16 = c.len_utf16() as u32;
        if c == '\n' {
            line += 1;
            col16 = 0;
        } else {
            col16 += c16;
        }
        i += c8;
    }

    // Sort by (line, char) — scanner emits in order, but string continuation
    // segments may interleave; sort defensively.
    toks.sort_by_key(|t| (t.line, t.char));

    // Relative-encode into SemanticToken structs (LSP wire shape).
    let mut data = Vec::with_capacity(toks.len());
    let mut prev_line = 0u32;
    let mut prev_char = 0u32;
    for t in toks {
        let (dl, dc) = if t.line == prev_line {
            (0, t.char - prev_char)
        } else {
            (t.line - prev_line, t.char)
        };
        data.push(SemanticToken {
            delta_line: dl,
            delta_start: dc,
            length: t.len,
            token_type: t.ty,
            token_modifiers_bitset: 0,
        });
        prev_line = t.line;
        prev_char = t.char;
    }
    data
}

/// Multi-line string continuation: keep consuming until the closing quote,
/// emitting one token per line segment.
#[allow(clippy::too_many_arguments)]
fn push_continuation(
    toks: &mut Vec<AbsTok>,
    ty: u32,
    mut line: u32,
    mut col: u32,
    len: &mut u32,
    quote: u8,
    i: &mut usize,
    col16: &mut u32,
    content: &str,
) {
    let bytes = content.as_bytes();
    let start_col = col;
    while *i < bytes.len() && bytes[*i] != quote {
        if bytes[*i] == b'\n' {
            push(toks, *len, ty, line, start_col);
            *len = 0;
            col = 0;
            *col16 = 0;
            line += 1;
            *i += 1;
        } else if bytes[*i] == b'\\' && *i + 1 < bytes.len() {
            *len += 2;
            *i += 2;
            col += 2;
            *col16 += 2;
        } else {
            let c16 = content[*i..]
                .chars()
                .next()
                .map(|c| c.len_utf16() as u32)
                .unwrap_or(1);
            let c8 = content[*i..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            *len += c16;
            *i += c8;
            col += c16;
            *col16 += c16;
        }
    }
    if *len > 0 {
        push(toks, *len, ty, line, start_col);
        *len = 0;
    }
}

#[allow(dead_code)]
fn push(toks: &mut Vec<AbsTok>, len: u32, ty: u32, line: u32, col: u32) {
    if len > 0 {
        toks.push(AbsTok {
            line,
            char: col,
            len,
            ty,
        });
    }
}

/// The legend options object advertised in `initialize`.
pub fn semantic_tokens_options() -> SemanticTokensOptions {
    SemanticTokensOptions {
        legend: SemanticTokensLegend {
            token_types: TOKEN_TYPES
                .iter()
                .map(|t| SemanticTokenType::from(*t))
                .collect(),
            token_modifiers: Vec::new(),
        },
        full: Some(SemanticTokensFullOptions::Bool(true)),
        range: Some(false),
        work_done_progress_options: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode relative data back into (line, char, len, type) tuples.
    fn decode(data: &[SemanticToken]) -> Vec<(u32, u32, u32, u32)> {
        let mut out = Vec::new();
        let mut line = 0;
        let mut ch = 0;
        for t in data {
            line += t.delta_line;
            ch = if t.delta_line == 0 {
                ch + t.delta_start
            } else {
                t.delta_start
            };
            out.push((line, ch, t.length, t.token_type));
        }
        out
    }

    /// Plan 416 5-B: lock the token sequence for a representative source —
    /// keyword, fn declaration + call, type, parameter, local, string, number,
    /// comment all classify and encode in order.
    #[test]
    fn test_semantic_token_sequence() {
        let src = "type Point {\n    x int\n}\n\nfn make(x int) Point {\n    let p = Point { x: x }\n    return p\n}\n\n// hi\nlet n = 42\n";
        let data = semantic_tokens_full(src);
        let toks = decode(&data);

        let find = |toks: &[(u32, u32, u32, u32)], word: &str, ty: u32| -> Option<(u32, u32)> {
            let col = src
                .lines()
                .enumerate()
                .find_map(|(li, l)| l.find(word).map(|c| (li as u32, c as u32)))
                .unwrap();
            toks.iter()
                .find(|(l, c, len, t)| *l == col.0 && *c == col.1 && *t == ty)
                .map(|(l, c, len, _)| (*l, *c))
                .or_else(|| {
                    eprintln!("missing {word} ty={ty} at {col:?}; got {toks:?}");
                    None
                })
        };

        // keyword `type` (line 0, col 0), fn `make` (line 4), type `Point`
        // (line 4 decl return / line 5 usage), parameter `x` (line 4),
        // local `p` (line 5), string? none here; number 42 (line 10),
        // comment `// hi` (line 9).
        assert!(find(&toks, "type", tt::KEYWORD).is_some());
        assert!(find(&toks, "fn", tt::KEYWORD).is_some());
        assert!(find(&toks, "let", tt::KEYWORD).is_some());
        assert!(find(&toks, "make", tt::FUNCTION).is_some());
        assert!(find(&toks, "Point", tt::TYPE).is_some());
        assert!(find(&toks, "42", tt::NUMBER).is_some());
        assert!(find(&toks, "// hi", tt::COMMENT).is_some());
        // parameter `x`: line 4 `fn make(x int)` — col of first x on that line
        let x_col = src.lines().nth(4).unwrap().find("x").unwrap() as u32;
        assert!(
            toks.iter()
                .any(|(l, c, _, t)| *l == 4 && *c == x_col && *t == tt::PARAMETER),
            "param x classified: {toks:?}"
        );
        // local `p`: line 5
        let p_col = src.lines().nth(5).unwrap().find("p").unwrap() as u32;
        assert!(
            toks.iter()
                .any(|(l, c, _, t)| *l == 5 && *c == p_col && *t == tt::VARIABLE),
            "local p classified: {toks:?}"
        );
    }

    #[test]
    fn test_string_and_call_heuristic() {
        let src = "fn main() {\n    greet(\"world\")\n}\n";
        let toks = decode(&semantic_tokens_full(src));
        // string literal classified
        assert!(toks.iter().any(|(_, _, _, t)| *t == tt::STRING), "{toks:?}");
        // `greet(` — unclassified ident followed by ( → function heuristic
        let gcol = src.lines().nth(1).unwrap().find("greet").unwrap() as u32;
        assert!(
            toks.iter()
                .any(|(l, c, _, t)| *l == 1 && *c == gcol && *t == tt::FUNCTION),
            "{toks:?}"
        );
    }

    /// Legend order is the wire contract — lock it.
    #[test]
    fn test_legend_order() {
        assert_eq!(TOKEN_TYPES[0], "keyword");
        let opts = semantic_tokens_options();
        assert_eq!(opts.legend.token_types.len(), TOKEN_TYPES.len());
        assert_eq!(opts.legend.token_types[5].as_str(), "string");
    }
}
