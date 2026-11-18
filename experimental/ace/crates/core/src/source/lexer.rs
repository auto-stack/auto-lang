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
    // Collections-surface keywords (PLAN-864): `enum` declarations, `is`
    // dispatch and the `for x in <list>` connector. Consumed only by the
    // collections source profile; every other profile keeps the historical
    // `source.unsupported` at the same tokens.
    Enum,
    Is,
    In,
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
            Kw::Enum => "enum",
            Kw::Is => "is",
            Kw::In => "in",
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
    /// PLAN-864: `?T` optional-type prefix (collections surface).
    Question,
    /// PLAN-864: `is` arm connector (collections surface).
    Arrow,
    /// PLAN-865: lambda body separator `=>` (closures surface).
    FatArrow,
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
            P::Question => "?",
            P::Arrow => "->",
            P::FatArrow => "=>",
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

/// PLAN-982（④ S2-A）：数值面的**显式门控**（默认关＝今天的表面：数字面一律
/// `RawKind::AffixedNum`）。打开后 `1.5` 成为 `Tok::Float`、超过 i32 的十进制整数成为
/// `Tok::Wide`；其余（`0x10`、`1i32`、`1.5.6`）仍是 `Raw`。
pub const NUMBER_SURFACE_ENV: &str = "AUTO_ACE_NUMBER_SURFACE";

pub fn number_surface_enabled() -> bool {
    matches!(
        std::env::var(NUMBER_SURFACE_ENV).as_deref(),
        Ok("1") | Ok("true") | Ok("yes") | Ok("ace")
    )
}

/// 词法层只做**形状**判定；数值解析与范围由 parser/常量层负责。
fn classify_affixed(text: String) -> Tok {
    if number_surface_enabled() {
        if let Some((int_part, frac)) = text.split_once('.') {
            let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
            if digits(int_part) && digits(frac) {
                return Tok::Float(text);
            }
        } else if text.bytes().all(|b| b.is_ascii_digit())
            && text.parse::<i32>().is_err()
            && text.parse::<u64>().is_ok()
        {
            return Tok::Wide(text);
        }
    }
    Tok::Raw {
        kind: RawKind::AffixedNum,
        text,
    }
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
    /// PLAN-982（④ S2-A）：门控数值面——小数字面量（`1.5`）。
    Float(String),
    /// PLAN-982（④ S2-A）：门控数值面——超 i32 的十进制整数（`18446744073709551615`）。
    Wide(String),
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
        // PLAN-864: collections-surface keywords (recognized everywhere so
        // "excluded surface" stays distinguishable; consumed only there).
        "enum" => Kw::Enum,
        "is" => Kw::Is,
        "in" => Kw::In,
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
            b'(' | b')' | b'{' | b'}' | b',' | b';' | b':' | b'[' | b']' | b'?' => {
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
                    // PLAN-864: the optional-type prefix (`?T`, collections).
                    b'?' => P::Question,
                    _ => unreachable!(),
                };
                lx.push_punct(p, 1, &mut out);
            }
            b'=' => {
                if lx.b.get(lx.pos + 1) == Some(&b'>') {
                    lx.push_punct(P::FatArrow, 2, &mut out);
                } else if lx.b.get(lx.pos + 1) == Some(&b'=') {
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
                    return Err(lx
                        .err_lex("unknown character `&`")
                        .tagged(crate::atom_text::SurfaceFact::OwnedCharacter));
                }
            }
            b'|' => {
                if lx.b.get(lx.pos + 1) == Some(&b'|') {
                    lx.push_punct(P::PipePipe, 2, &mut out);
                } else {
                    return Err(lx
                        .err_lex("unknown character `|`")
                        .tagged(crate::atom_text::SurfaceFact::OwnedCharacter));
                }
            }
            b'-' if lx.b.get(lx.pos + 1) == Some(&b'>') => {
                // PLAN-864: the `is` arm connector lexes as one token so an
                // arm arrow can never fold into minus/greater comparisons.
                lx.push_punct(P::Arrow, 2, &mut out);
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
                    let text = lx.src[start..lx.pos].to_string();
                    out.push(Token {
                        tok: classify_affixed(text),
                        span: Span { start, end: lx.pos },
                        line: lx.line,
                    });
                } else {
                    // PLAN-982（④ S2-A）：无后缀的十进制整数默认就是 `Int`（范围由
                    // parser 的常量层报 `source.literal-range`）；门控打开时超 i32
                    // 且能装进 u64 的才升成 `Wide`。
                    let text = lx.src[start..lx.pos].to_string();
                    let tok = if number_surface_enabled()
                        && text.parse::<i32>().is_err()
                        && text.parse::<u64>().is_ok()
                    {
                        Tok::Wide(text)
                    } else {
                        Tok::Int(text)
                    };
                    out.push(Token {
                        tok,
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
                    let mut diag = lx.err_lex(format!("unknown character `{}`", c as char));
                    if matches!(c, b'&' | b'|' | b'^') {
                        // PLAN-969 (AUDIT-03): numeric-owned operator
                        // characters carry the structured surface fact.
                        diag = diag.tagged(crate::atom_text::SurfaceFact::OwnedCharacter);
                    }
                    return Err(diag);
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
#[cfg(test)]
mod number_surface_tests {
    use super::*;

    /// 两个用例共享**进程级**门控变量，必须串行。
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn kinds(src: &str) -> Vec<String> {
        tokenize(src)
            .expect("the source lexes")
            .into_iter()
            .map(|t| match t.tok {
                Tok::Float(v) => format!("Float({v})"),
                Tok::Wide(v) => format!("Wide({v})"),
                Tok::Int(v) => format!("Int({v})"),
                Tok::Raw { kind, text } => format!("Raw({kind:?},{text})"),
                other => format!("{other:?}"),
            })
            .collect()
    }

    /// 默认（门控关）：数字面一律是被排除的表面——与今天的行为逐字一致。
    #[test]
    fn number_surface_is_closed_by_default() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var(NUMBER_SURFACE_ENV);
        assert!(kinds("1.5")[0].starts_with("Raw("));
        assert!(kinds("2147483648")[0].starts_with("Int("));
        assert!(kinds("0x10")[0].starts_with("Raw("));
    }

    /// 门控打开：小数与超 i32 的十进制整数成词；其余仍是被排除的表面。
    #[test]
    fn number_surface_opens_float_and_wide_integers_only() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var(NUMBER_SURFACE_ENV, "1");
        let float = kinds("1.5");
        let wide = kinds("2147483648");
        let too_big = kinds("18446744073709551616");
        let hex = kinds("0x10");
        let suffix = kinds("1i32");
        let dotted = kinds("1.5.6");
        std::env::remove_var(NUMBER_SURFACE_ENV);

        assert_eq!(float, vec!["Float(1.5)".to_string()]);
        assert_eq!(wide, vec!["Wide(2147483648)".to_string()]);
        // 超出 u64 仍是 `Int`：范围由 parser 的常量层报 `source.literal-range`。
        assert!(too_big[0].starts_with("Int("), "{too_big:?}");
        assert!(hex[0].starts_with("Raw("), "{hex:?}");
        assert!(suffix[0].starts_with("Raw("), "{suffix:?}");
        assert!(dotted[0].starts_with("Raw("), "{dotted:?}");
    }
}
