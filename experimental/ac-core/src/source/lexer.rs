//! Source-subset lexer for the `auto.source.core-i32.draft` profile
//! (PLAN-745 T-02).
//!
//! UTF-8 input; identifiers are ASCII-only (`[A-Za-z_][A-Za-z0-9_]*`),
//! keywords cannot be used as names. `//` line comments (arbitrary UTF-8
//! content) are skipped. Newlines are significant for statement/expression
//! separation, so every token records its 1-based source line.
//!
//! Excluded-surface tokens (char literals, string literals, suffixed/radix/
//! float numerals, `/*`, non-ASCII characters) are lexed as [`Tok::Raw`] so
//! the parser can reject them with `source.unsupported` at the exact token.

use crate::atom_text::{Diagnostic, Span, Stage};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kw {
    Fn,
    Let,
    Var,
    If,
    Else,
    While,
    Loop,
    Break,
    Continue,
    Return,
    True,
    False,
    // Conditional loop head: consumed by the scalar-i32 source profile
    // (PLAN-767 §5.2.1); the core profile rejects it as unsupported.
    For,
    // Type words are keywords: they cannot be used as names.
    Int,
    I32,
    Bool,
    // Record-surface keywords (PLAN-771): consumed by the record source
    // profile; the core/scalar profiles reject them wherever they appear
    // (same `source.unsupported` as the historical Reserved lexing).
    Type,
    Static,
    Mut,
}

impl Kw {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kw::Fn => "fn",
            Kw::Let => "let",
            Kw::Var => "var",
            Kw::If => "if",
            Kw::Else => "else",
            Kw::While => "while",
            Kw::Loop => "loop",
            Kw::Break => "break",
            Kw::Continue => "continue",
            Kw::Return => "return",
            Kw::True => "true",
            Kw::False => "false",
            Kw::For => "for",
            Kw::Int => "int",
            Kw::I32 => "i32",
            Kw::Bool => "bool",
            Kw::Type => "type",
            Kw::Static => "static",
            Kw::Mut => "mut",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum P {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Colon,
    LBracket,
    RBracket,
    Assign,
    Plus,
    Star,
    Lt,
    Minus,
    // Operators that exist in the wider language but are outside this
    // profile; the parser rejects them with `source.unsupported`.
    Slash,
    Percent,
    Gt,
    Bang,
    AmpAmp,
    PipePipe,
    EqEq,
    NotEq,
    LeEq,
    GeEq,
    Dot,
}

impl P {
    pub fn as_str(&self) -> &'static str {
        match self {
            P::LParen => "(",
            P::RParen => ")",
            P::LBrace => "{",
            P::RBrace => "}",
            P::Comma => ",",
            P::Semi => ";",
            P::Colon => ":",
            P::LBracket => "[",
            P::RBracket => "]",
            P::Assign => "=",
            P::Plus => "+",
            P::Star => "*",
            P::Lt => "<",
            P::Minus => "-",
            P::Slash => "/",
            P::Percent => "%",
            P::Gt => ">",
            P::Bang => "!",
            P::AmpAmp => "&&",
            P::PipePipe => "||",
            P::EqEq => "==",
            P::NotEq => "!=",
            P::LeEq => "<=",
            P::GeEq => ">=",
            P::Dot => ".",
        }
    }
}

/// Shape of an excluded-surface token. All of these render as
/// `source.unsupported` at the token when they appear where the profile
/// expects something else.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RawKind {
    /// `'a'` — char literal (full text including quotes).
    CharLit,
    /// `"..."` — string literal.
    StrLit,
    /// `1i32` / `0x10` / `1.5` — suffixed, radix-prefixed or float numeral.
    AffixedNum,
    /// `/*` — block comment opener (block comments are not in the profile).
    BlockComment,
    /// A single non-ASCII character in code position (Unicode identifiers
    /// are excluded; comments may still carry arbitrary UTF-8).
    NonAscii,
}

impl RawKind {
    pub fn describe(&self) -> &'static str {
        match self {
            RawKind::CharLit => "char literal",
            RawKind::StrLit => "string literal",
            RawKind::AffixedNum => "suffixed/radix/float numeric literal",
            RawKind::BlockComment => "block comment",
            RawKind::NonAscii => "non-ASCII character",
        }
    }
}

/// Full-language keywords that this profile family does not use (mirrors
/// the production lexer's `Token::all_keywords()` set plus `with`; the
/// subset the profiles actually consume stays in [`Kw`] — `for` moved there
/// in PLAN-767). They lex as their own token kind so "recognized but
/// excluded" (source.unsupported) stays distinguishable from genuinely bad
/// tokens (source.lex) and bad grammar (source.syntax). A keyword can never
/// be used as a name.
pub const RESERVED_KEYWORDS: &[&str] = &[
    "alias", "as", "await", "catch", "const", "copy", "dep", "enum", "ext", "go", "has", "hold",
    "impl", "in", "is", "link", "mod", "move", "nil", "null", "outlet", "pac", "reply", "routes",
    "shared", "spawn", "spec", "super", "super2", "super3", "super4", "tag", "take", "task", "to",
    "try", "union", "use", "view", "with", "yield",
];

pub fn is_reserved_keyword(name: &str) -> bool {
    RESERVED_KEYWORDS.contains(&name)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Kw(Kw),
    /// Full-language keyword outside this profile: recognized, but rejected
    /// with `source.unsupported` wherever it appears (never usable as a
    /// name).
    Reserved(String),
    Punct(P),
    /// Unsigned decimal digits; a leading `-` is a separate [`P::Minus`]
    /// token folded into a signed literal by the parser.
    Int(String),
    Raw {
        kind: RawKind,
        text: String,
    },
    /// Decoded string literal (text profile only, PLAN-770): escapes
    /// resolved, cross-line line breaks kept verbatim in the value.
    StrLit(String),
    /// Decoded char literal (text profile only): one Unicode scalar.
    CharLit(u32),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
    /// 1-based line the token starts on.
    pub line: usize,
}

pub fn keyword_of(name: &str) -> Option<Kw> {
    Some(match name {
        "fn" => Kw::Fn,
        "let" => Kw::Let,
        "var" => Kw::Var,
        "if" => Kw::If,
        "else" => Kw::Else,
        "while" => Kw::While,
        "loop" => Kw::Loop,
        "break" => Kw::Break,
        "continue" => Kw::Continue,
        "return" => Kw::Return,
        "true" => Kw::True,
        "false" => Kw::False,
        "for" => Kw::For,
        "int" => Kw::Int,
        "i32" => Kw::I32,
        "bool" => Kw::Bool,
        "type" => Kw::Type,
        "static" => Kw::Static,
        "mut" => Kw::Mut,
        _ => return None,
    })
}

struct Lexer<'s> {
    src: &'s str,
    b: &'s [u8],
    pos: usize,
    line: usize,
    text: bool,
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, Diagnostic> {
    tokenize_with(src, false)
}

/// Tokenize with the text-profile literal surface enabled (PLAN-770):
/// string/char literals decode to real tokens (escape whitelist, scalar
/// validation, cross-line strings whose line breaks enter the value
/// verbatim). The historical core/scalar behavior — raw excluded tokens
/// and the unterminated-at-newline rule — is preserved bit-for-bit when
/// `text` is false.
pub fn tokenize_with(src: &str, text: bool) -> Result<Vec<Token>, Diagnostic> {
    let mut lx = Lexer {
        src,
        b: src.as_bytes(),
        pos: 0,
        line: 1,
        text,
    };
    let mut out = Vec::new();
    while lx.pos < lx.b.len() {
        let c = lx.b[lx.pos];
        match c {
            b' ' | b'\t' | b'\r' => lx.pos += 1,
            b'\n' => {
                lx.pos += 1;
                lx.line += 1;
            }
            b'/' if lx.b.get(lx.pos + 1) == Some(&b'/') => {
                while lx.pos < lx.b.len() && lx.b[lx.pos] != b'\n' {
                    lx.pos += 1;
                }
            }
            b'/' if lx.b.get(lx.pos + 1) == Some(&b'*') => {
                let start = lx.pos;
                lx.pos += 2;
                out.push(Token {
                    tok: Tok::Raw {
                        kind: RawKind::BlockComment,
                        text: "/*".to_string(),
                    },
                    span: Span { start, end: lx.pos },
                    line: lx.line,
                });
            }
            b'(' | b')' | b'{' | b'}' | b',' | b';' | b':' | b'[' | b']' => {
                let p = match c {
                    b'(' => P::LParen,
                    b')' => P::RParen,
                    b'{' => P::LBrace,
                    b'}' => P::RBrace,
                    b',' => P::Comma,
                    b';' => P::Semi,
                    b':' => P::Colon,
                    b'[' => P::LBracket,
                    b']' => P::RBracket,
                    _ => unreachable!(),
                };
                lx.push_punct(p, 1, &mut out);
            }
            b'=' => {
                if lx.b.get(lx.pos + 1) == Some(&b'=') {
                    lx.push_punct(P::EqEq, 2, &mut out);
                } else {
                    lx.push_punct(P::Assign, 1, &mut out);
                }
            }
            b'<' => {
                if lx.b.get(lx.pos + 1) == Some(&b'=') {
                    lx.push_punct(P::LeEq, 2, &mut out);
                } else {
                    lx.push_punct(P::Lt, 1, &mut out);
                }
            }
            b'>' => {
                if lx.b.get(lx.pos + 1) == Some(&b'=') {
                    lx.push_punct(P::GeEq, 2, &mut out);
                } else {
                    lx.push_punct(P::Gt, 1, &mut out);
                }
            }
            b'!' => {
                if lx.b.get(lx.pos + 1) == Some(&b'=') {
                    lx.push_punct(P::NotEq, 2, &mut out);
                } else {
                    lx.push_punct(P::Bang, 1, &mut out);
                }
            }
            b'&' => {
                if lx.b.get(lx.pos + 1) == Some(&b'&') {
                    lx.push_punct(P::AmpAmp, 2, &mut out);
                } else {
                    return Err(lx.err_lex("unknown character `&`"));
                }
            }
            b'|' => {
                if lx.b.get(lx.pos + 1) == Some(&b'|') {
                    lx.push_punct(P::PipePipe, 2, &mut out);
                } else {
                    return Err(lx.err_lex("unknown character `|`"));
                }
            }
            b'+' | b'*' | b'-' | b'/' | b'%' | b'.' => {
                let p = match c {
                    b'+' => P::Plus,
                    b'*' => P::Star,
                    b'-' => P::Minus,
                    b'/' => P::Slash,
                    b'%' => P::Percent,
                    b'.' => P::Dot,
                    _ => unreachable!(),
                };
                lx.push_punct(p, 1, &mut out);
            }
            b'"' if lx.text => {
                let start = lx.pos;
                lx.pos += 1;
                let mut value = String::new();
                loop {
                    match lx.b.get(lx.pos) {
                        None => {
                            return Err(Diagnostic::new(
                                Stage::Text,
                                "source.lex",
                                "unterminated string literal",
                                Span { start, end: lx.pos },
                            ));
                        }
                        Some(b'"') => {
                            lx.pos += 1;
                            break;
                        }
                        Some(b'\n') => {
                            // cross-line literal (774 r2 R2-02): the line
                            // break enters the value verbatim — LF sources
                            // keep LF, CRLF sources keep CRLF (the CR byte
                            // is consumed by the raw byte arm below)
                            value.push('\n');
                            lx.pos += 1;
                            lx.line += 1;
                        }
                        Some(b'\\') => {
                            let esc_start = lx.pos;
                            lx.pos += 1;
                            match lx.b.get(lx.pos) {
                                Some(b'n') => {
                                    value.push('\n');
                                    lx.pos += 1;
                                }
                                Some(b't') => {
                                    value.push('\t');
                                    lx.pos += 1;
                                }
                                Some(b'r') => {
                                    value.push('\r');
                                    lx.pos += 1;
                                }
                                Some(b'0') => {
                                    value.push('\0');
                                    lx.pos += 1;
                                }
                                Some(b'\\') => {
                                    value.push('\\');
                                    lx.pos += 1;
                                }
                                Some(b'"') => {
                                    value.push('"');
                                    lx.pos += 1;
                                }
                                Some(b'u') => {
                                    lx.pos += 1;
                                    if lx.b.get(lx.pos) != Some(&b'{') {
                                        return Err(Diagnostic::new(
                                            Stage::Text,
                                            "text.invalid-escape",
                                            "escape \\u expects `{`",
                                            Span {
                                                start: esc_start,
                                                end: lx.pos,
                                            },
                                        ));
                                    }
                                    lx.pos += 1;
                                    let mut hex = String::new();
                                    while matches!(lx.b.get(lx.pos), Some(c) if c.is_ascii_hexdigit())
                                    {
                                        hex.push(lx.b[lx.pos] as char);
                                        lx.pos += 1;
                                    }
                                    if lx.b.get(lx.pos) != Some(&b'}')
                                        || hex.is_empty()
                                        || hex.len() > 6
                                    {
                                        return Err(Diagnostic::new(
                                            Stage::Text,
                                            "text.invalid-escape",
                                            "escape \\u{...} expects 1-6 hex digits and `}`",
                                            Span {
                                                start: esc_start,
                                                end: lx.pos,
                                            },
                                        ));
                                    }
                                    lx.pos += 1;
                                    let cp = u32::from_str_radix(&hex, 16).unwrap();
                                    match char::from_u32(cp) {
                                        Some(ch) => value.push(ch),
                                        None => {
                                            return Err(Diagnostic::new(
                                                Stage::Text,
                                                "text.invalid-scalar",
                                                format!(
                                                    "\\u{{{}}} is not a Unicode scalar value",
                                                    hex
                                                ),
                                                Span {
                                                    start: esc_start,
                                                    end: lx.pos,
                                                },
                                            ));
                                        }
                                    }
                                }
                                _ => {
                                    return Err(Diagnostic::new(
                                        Stage::Text,
                                        "text.invalid-escape",
                                        "unknown escape sequence (whitelist: \\n \\t \\r \\0 \\\\ \\\" \\u{hex})",
                                        Span {
                                            start: esc_start,
                                            end: (lx.pos + 1).min(lx.b.len()),
                                        },
                                    ));
                                }
                            }
                        }
                        Some(&byte) if byte < 0x80 => {
                            value.push(byte as char);
                            lx.pos += 1;
                        }
                        Some(_) => {
                            let ch = lx.src[lx.pos..].chars().next().unwrap();
                            value.push(ch);
                            lx.pos += ch.len_utf8();
                        }
                    }
                }
                out.push(Token {
                    tok: Tok::StrLit(value),
                    span: Span { start, end: lx.pos },
                    line: lx.line,
                });
            }
            b'\'' if lx.text => {
                let start = lx.pos;
                lx.pos += 1;
                // empty literal: the closing quote is the first body byte
                if lx.b.get(lx.pos) == Some(&b'\'') {
                    lx.pos += 1;
                    return Err(Diagnostic::new(
                        Stage::Text,
                        "text.char-empty",
                        "char literal must hold exactly one scalar",
                        Span { start, end: lx.pos },
                    ));
                }
                let scalar: char = match lx.b.get(lx.pos) {
                    None | Some(b'\n') => {
                        return Err(Diagnostic::new(
                            Stage::Text,
                            "source.lex",
                            "unterminated char literal",
                            Span {
                                start,
                                end: lx.pos.min(lx.b.len()),
                            },
                        ));
                    }
                    Some(b'\\') => {
                        let esc_start = lx.pos;
                        lx.pos += 1;
                        match lx.b.get(lx.pos) {
                            Some(b'n') => {
                                lx.pos += 1;
                                '\n'
                            }
                            Some(b't') => {
                                lx.pos += 1;
                                '\t'
                            }
                            Some(b'r') => {
                                lx.pos += 1;
                                '\r'
                            }
                            Some(b'0') => {
                                lx.pos += 1;
                                '\0'
                            }
                            Some(b'\\') => {
                                lx.pos += 1;
                                '\\'
                            }
                            Some(b'\'') => {
                                lx.pos += 1;
                                '\''
                            }
                            Some(b'u') => {
                                lx.pos += 1;
                                if lx.b.get(lx.pos) != Some(&b'{') {
                                    return Err(Diagnostic::new(
                                        Stage::Text,
                                        "text.invalid-escape",
                                        "escape \\u expects `{`",
                                        Span {
                                            start: esc_start,
                                            end: lx.pos,
                                        },
                                    ));
                                }
                                lx.pos += 1;
                                let mut hex = String::new();
                                while matches!(lx.b.get(lx.pos), Some(c) if c.is_ascii_hexdigit()) {
                                    hex.push(lx.b[lx.pos] as char);
                                    lx.pos += 1;
                                }
                                if lx.b.get(lx.pos) != Some(&b'}')
                                    || hex.is_empty()
                                    || hex.len() > 6
                                {
                                    return Err(Diagnostic::new(
                                        Stage::Text,
                                        "text.invalid-escape",
                                        "escape \\u{...} expects 1-6 hex digits and `}`",
                                        Span {
                                            start: esc_start,
                                            end: lx.pos,
                                        },
                                    ));
                                }
                                lx.pos += 1;
                                let cp = u32::from_str_radix(&hex, 16).unwrap();
                                match char::from_u32(cp) {
                                    Some(ch) => ch,
                                    None => {
                                        return Err(Diagnostic::new(
                                            Stage::Text,
                                            "text.invalid-scalar",
                                            format!("\\u{{{}}} is not a Unicode scalar value", hex),
                                            Span {
                                                start: esc_start,
                                                end: lx.pos,
                                            },
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Err(Diagnostic::new(
                                    Stage::Text,
                                    "text.invalid-escape",
                                    "unknown escape sequence in char literal",
                                    Span {
                                        start: esc_start,
                                        end: (lx.pos + 1).min(lx.b.len()),
                                    },
                                ));
                            }
                        }
                    }
                    Some(&byte) if byte < 0x80 => {
                        lx.pos += 1;
                        byte as char
                    }
                    Some(_) => {
                        let ch = lx.src[lx.pos..].chars().next().unwrap();
                        lx.pos += ch.len_utf8();
                        ch
                    }
                };
                if lx.b.get(lx.pos) != Some(&b'\'') {
                    return Err(Diagnostic::new(
                        Stage::Text,
                        "text.char-multi",
                        format!(
                            "char literal must hold exactly one scalar (found more after `{}`)",
                            scalar
                        ),
                        Span {
                            start,
                            end: (lx.pos + 1).min(lx.b.len()),
                        },
                    ));
                }
                lx.pos += 1;
                out.push(Token {
                    tok: Tok::CharLit(scalar as u32),
                    span: Span { start, end: lx.pos },
                    line: lx.line,
                });
            }
            b'"' => {
                let start = lx.pos;
                lx.pos += 1;
                let mut text = String::from("\"");
                loop {
                    match lx.b.get(lx.pos) {
                        None | Some(b'\n') => {
                            return Err(Diagnostic::new(
                                Stage::Text,
                                "source.lex",
                                "unterminated string literal",
                                Span { start, end: lx.pos },
                            ));
                        }
                        Some(b'"') => {
                            lx.pos += 1;
                            text.push('"');
                            break;
                        }
                        // An escape pair belongs to the literal: it must not
                        // end it, so `"a\"b"` lexes as ONE excluded token.
                        Some(b'\\') if matches!(lx.b.get(lx.pos + 1), Some(&e) if e != b'\n') => {
                            let esc = lx.src[lx.pos..].chars().nth(1).unwrap();
                            text.push('\\');
                            text.push(esc);
                            lx.pos += 1 + esc.len_utf8();
                        }
                        Some(&byte) if byte < 0x80 => {
                            text.push(byte as char);
                            lx.pos += 1;
                        }
                        Some(_) => {
                            let ch = lx.src[lx.pos..].chars().next().unwrap();
                            text.push(ch);
                            lx.pos += ch.len_utf8();
                        }
                    }
                }
                out.push(Token {
                    tok: Tok::Raw {
                        kind: RawKind::StrLit,
                        text,
                    },
                    span: Span { start, end: lx.pos },
                    line: lx.line,
                });
            }
            b'\'' => {
                let start = lx.pos;
                lx.pos += 1;
                let mut text = String::from("'");
                loop {
                    match lx.b.get(lx.pos) {
                        None | Some(b'\n') => {
                            return Err(Diagnostic::new(
                                Stage::Text,
                                "source.lex",
                                "unterminated char literal",
                                Span {
                                    start,
                                    end: lx.pos.min(lx.b.len()),
                                },
                            ));
                        }
                        Some(b'\'') => {
                            lx.pos += 1;
                            text.push('\'');
                            break;
                        }
                        // Escape pair inside a char literal (`'\''`) stays in
                        // the literal.
                        Some(b'\\') if matches!(lx.b.get(lx.pos + 1), Some(&e) if e != b'\n') => {
                            let esc = lx.src[lx.pos..].chars().nth(1).unwrap();
                            text.push('\\');
                            text.push(esc);
                            lx.pos += 1 + esc.len_utf8();
                        }
                        Some(&byte) if byte < 0x80 => {
                            text.push(byte as char);
                            lx.pos += 1;
                        }
                        Some(_) => {
                            let ch = lx.src[lx.pos..].chars().next().unwrap();
                            text.push(ch);
                            lx.pos += ch.len_utf8();
                        }
                    }
                }
                out.push(Token {
                    tok: Tok::Raw {
                        kind: RawKind::CharLit,
                        text,
                    },
                    span: Span { start, end: lx.pos },
                    line: lx.line,
                });
            }
            b'0'..=b'9' => {
                let start = lx.pos;
                while lx.pos < lx.b.len() && lx.b[lx.pos].is_ascii_digit() {
                    lx.pos += 1;
                }
                // A digit run followed by '.', an alphabetic or '_' carries an
                // affix/prefix/fraction: `1i32`, `0x10`, `1.5`. The whole run
                // becomes one excluded token.
                let affixed = matches!(lx.b.get(lx.pos), Some(c) if c.is_ascii_alphabetic() || *c == b'_' || *c == b'.');
                if affixed {
                    while lx.pos < lx.b.len()
                        && (lx.b[lx.pos].is_ascii_alphanumeric()
                            || lx.b[lx.pos] == b'_'
                            || lx.b[lx.pos] == b'.')
                    {
                        lx.pos += 1;
                    }
                    out.push(Token {
                        tok: Tok::Raw {
                            kind: RawKind::AffixedNum,
                            text: lx.src[start..lx.pos].to_string(),
                        },
                        span: Span { start, end: lx.pos },
                        line: lx.line,
                    });
                } else {
                    out.push(Token {
                        tok: Tok::Int(lx.src[start..lx.pos].to_string()),
                        span: Span { start, end: lx.pos },
                        line: lx.line,
                    });
                }
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = lx.pos;
                while lx.pos < lx.b.len()
                    && (lx.b[lx.pos].is_ascii_alphanumeric() || lx.b[lx.pos] == b'_')
                {
                    lx.pos += 1;
                }
                let name = &lx.src[start..lx.pos];
                let tok = match keyword_of(name) {
                    Some(k) => Tok::Kw(k),
                    None if is_reserved_keyword(name) => Tok::Reserved(name.to_string()),
                    None => Tok::Ident(name.to_string()),
                };
                out.push(Token {
                    tok,
                    span: Span { start, end: lx.pos },
                    line: lx.line,
                });
            }
            _ => {
                // Non-ASCII in code position: excluded surface (Unicode
                // identifiers), reported at the character. ASCII bytes that
                // are not any token start are lexer errors.
                if c >= 0x80 {
                    let ch = lx.src[lx.pos..].chars().next().unwrap();
                    let start = lx.pos;
                    lx.pos += ch.len_utf8();
                    out.push(Token {
                        tok: Tok::Raw {
                            kind: RawKind::NonAscii,
                            text: ch.to_string(),
                        },
                        span: Span { start, end: lx.pos },
                        line: lx.line,
                    });
                } else {
                    return Err(lx.err_lex(format!("unknown character `{}`", c as char)));
                }
            }
        }
    }
    Ok(out)
}

impl<'s> Lexer<'s> {
    fn push_punct(&mut self, p: P, len: usize, out: &mut Vec<Token>) {
        let start = self.pos;
        self.pos += len;
        out.push(Token {
            tok: Tok::Punct(p),
            span: Span {
                start,
                end: self.pos,
            },
            line: self.line,
        });
    }

    fn err_lex(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(
            Stage::Text,
            "source.lex",
            msg,
            Span {
                start: self.pos,
                end: (self.pos + 1).min(self.b.len()),
            },
        )
    }
}
