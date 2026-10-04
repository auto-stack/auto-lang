//! Atom text reader for PLAN-741: tokenizer + generic spanned syntax tree.
//!
//! This reader accepts the Schema-bound Atom text forms used by the core-i32
//! profile: node tags with positional primary slots, named field lists in
//! parentheses and/or braces (interchangeable, duplicates rejected at bind
//! time), strings, suffixed integers, bareword enums, lists and inline
//! objects. Comments (`//`) are skipped; layout is not preserved.
//!
//! The reader is purely syntactic: it never interprets barewords as enums and
//! never guesses reference categories. That is the descriptor binder's job
//! (`crate::descriptor`).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn join(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn slice<'a>(&self, src: &'a str) -> &'a str {
        &src[self.start..self.end.min(src.len())]
    }
}

/// Pipeline stage that produced a diagnostic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Text,
    Bind,
    Verify,
    Capability,
    Backend,
    Link,
    Run,
}

impl Stage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Stage::Text => "text",
            Stage::Bind => "bind",
            Stage::Verify => "verify",
            Stage::Capability => "capability",
            Stage::Backend => "backend",
            Stage::Link => "link",
            Stage::Run => "run",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub stage: Stage,
    pub code: String,
    pub message: String,
    pub span: Span,
}

impl Diagnostic {
    pub fn new(stage: Stage, code: &str, message: impl Into<String>, span: Span) -> Self {
        Diagnostic {
            stage,
            code: code.to_string(),
            message: message.into(),
            span,
        }
    }

    /// Human-readable one-line rendering with 1-based line/column.
    pub fn render(&self, path: &str, src: &str) -> String {
        let (line, col) = line_col(src, self.span.start);
        format!(
            "{}:{}:{}: error[{}/{}]: {}",
            path,
            line,
            col,
            self.stage.as_str(),
            self.code,
            self.message
        )
    }
}

pub fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(src.len());
    let before = &src[..offset];
    let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
    let col = before.rfind('\n').map_or(offset + 1, |p| offset - p);
    (line, col)
}

// ---------------------------------------------------------------------------
// Tokens
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Str(String),
    Int {
        negative: bool,
        digits: String,
        suffix: Option<String>,
    },
    Punct(char),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, Diagnostic> {
    let b = src.as_bytes();
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i];
        match c {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'(' | b')' | b'{' | b'}' | b'[' | b']' | b':' | b';' | b',' => {
                out.push(Token {
                    tok: Tok::Punct(c as char),
                    span: Span {
                        start: i,
                        end: i + 1,
                    },
                });
                i += 1;
            }
            b'"' => {
                let start = i;
                i += 1;
                let mut s = String::new();
                loop {
                    match b.get(i) {
                        None => {
                            return Err(Diagnostic::new(
                                Stage::Text,
                                "text.unterminated-string",
                                "unterminated string literal",
                                Span {
                                    start,
                                    end: src.len(),
                                },
                            ))
                        }
                        Some(b'"') => {
                            i += 1;
                            break;
                        }
                        Some(b'\\') => {
                            i += 1;
                            match b.get(i) {
                                Some(b'"') => {
                                    s.push('"');
                                    i += 1;
                                }
                                Some(b'\\') => {
                                    s.push('\\');
                                    i += 1;
                                }
                                Some(b'n') => {
                                    s.push('\n');
                                    i += 1;
                                }
                                Some(b't') => {
                                    s.push('\t');
                                    i += 1;
                                }
                                _ => {
                                    return Err(Diagnostic::new(
                                        Stage::Text,
                                        "text.bad-escape",
                                        "unsupported string escape",
                                        Span {
                                            start: i.saturating_sub(1),
                                            end: (i + 1).min(src.len()),
                                        },
                                    ))
                                }
                            }
                        }
                        Some(&byte) if byte < 0x80 => {
                            s.push(byte as char);
                            i += 1;
                        }
                        Some(&_) => {
                            // copy one UTF-8 scalar
                            let ch = src[i..].chars().next().unwrap();
                            s.push(ch);
                            i += ch.len_utf8();
                        }
                    }
                }
                out.push(Token {
                    tok: Tok::Str(s),
                    span: Span { start, end: i },
                });
            }
            b'-' | b'0'..=b'9' => {
                let start = i;
                if c == b'-' {
                    i += 1;
                }
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                let text = &src[start..i];
                let (negative, rest) = match text.strip_prefix('-') {
                    Some(r) => (true, r),
                    None => (false, text),
                };
                let split = rest
                    .find(|ch: char| ch.is_ascii_alphabetic() || ch == '_')
                    .unwrap_or(rest.len());
                let digits = &rest[..split];
                let suffix = if split < rest.len() {
                    Some(rest[split..].to_string())
                } else {
                    None
                };
                if digits.is_empty() || !digits.bytes().all(|d| d.is_ascii_digit()) {
                    return Err(Diagnostic::new(
                        Stage::Text,
                        "text.invalid-number",
                        format!("invalid numeric literal `{}`", text),
                        Span { start, end: i },
                    ));
                }
                out.push(Token {
                    tok: Tok::Int {
                        negative,
                        digits: digits.to_string(),
                        suffix,
                    },
                    span: Span { start, end: i },
                });
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                out.push(Token {
                    tok: Tok::Ident(src[start..i].to_string()),
                    span: Span { start, end: i },
                });
            }
            _ => {
                return Err(Diagnostic::new(
                    Stage::Text,
                    "text.unexpected-char",
                    format!(
                        "unexpected character `{}`",
                        src[i..].chars().next().unwrap()
                    ),
                    Span {
                        start: i,
                        end: i + src[i..].chars().next().unwrap().len_utf8(),
                    },
                ))
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Syntax tree
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct IntLit {
    pub negative: bool,
    pub digits: String,
    pub suffix: Option<String>,
    pub span: Span,
}

impl IntLit {
    pub fn to_u32(&self) -> Option<u32> {
        if self.negative {
            return None;
        }
        self.digits
            .parse::<u128>()
            .ok()
            .filter(|v| *v <= u32::MAX as u128)
            .map(|v| v as u32)
    }

    pub fn to_i32(&self) -> Option<i32> {
        let magnitude: Option<u128> = self.digits.parse().ok();
        let magnitude = magnitude?;
        if self.negative {
            if magnitude <= i32::MIN.unsigned_abs() as u128 {
                Some((-i128::try_from(magnitude).unwrap()) as i32)
            } else {
                None
            }
        } else if magnitude <= i32::MAX as u128 {
            Some(magnitude as i32)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Ident(String),
    Str(String),
    Int(IntLit),
    List(Vec<Value>),
    Obj(Vec<NamedValue>),
}

#[derive(Clone, Debug)]
pub struct NamedValue {
    pub name: String,
    pub name_span: Span,
    pub value: Value,
    pub value_span: Span,
}

#[derive(Clone, Debug)]
pub enum Item {
    Field(NamedValue),
    Node(SyntaxNode),
}

#[derive(Clone, Debug)]
pub struct SyntaxNode {
    pub tag: String,
    pub tag_span: Span,
    /// Positional bareword/str/int slots before the argument list.
    pub primary: Vec<(Value, Span)>,
    /// Fields declared inside `(...)`.
    pub head: Vec<NamedValue>,
    /// Interleaved fields and child nodes inside `{ ... }`, in order.
    pub body: Vec<Item>,
    pub span: Span,
}

impl SyntaxNode {
    /// All named fields: head + body fields, in occurrence order. Duplicates
    /// are reported by the binder, not silently merged here.
    pub fn all_fields(&self) -> Vec<&NamedValue> {
        let mut out: Vec<&NamedValue> = self.head.iter().collect();
        for item in &self.body {
            if let Item::Field(f) = item {
                out.push(f);
            }
        }
        out
    }

    pub fn child_nodes(&self) -> Vec<&SyntaxNode> {
        self.body
            .iter()
            .filter_map(|i| match i {
                Item::Node(n) => Some(n),
                Item::Field(_) => None,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser<'s> {
    src: &'s str,
    toks: Vec<Token>,
    pos: usize,
}

pub fn parse(src: &str) -> Result<SyntaxNode, Diagnostic> {
    let toks = tokenize(src)?;
    let mut p = Parser { src, toks, pos: 0 };
    if p.toks.is_empty() {
        return Err(Diagnostic::new(
            Stage::Text,
            "text.empty",
            "empty source",
            Span { start: 0, end: 0 },
        ));
    }
    let node = p.parse_node()?;
    if p.pos < p.toks.len() {
        let t = &p.toks[p.pos];
        return Err(Diagnostic::new(
            Stage::Text,
            "text.trailing-tokens",
            format!("unexpected trailing token `{}`", t.span.slice(src)),
            t.span,
        ));
    }
    Ok(node)
}

fn value_starts(tok: &Tok) -> bool {
    matches!(tok, Tok::Ident(_) | Tok::Str(_) | Tok::Int { .. })
}

impl<'s> Parser<'s> {
    fn peek(&self) -> Option<&Token> {
        self.toks.get(self.pos)
    }

    fn peek2(&self) -> Option<&Token> {
        self.toks.get(self.pos + 1)
    }

    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        self.pos += 1;
        t
    }

    fn err(&self, code: &str, msg: impl Into<String>, span: Span) -> Diagnostic {
        Diagnostic::new(Stage::Text, code, msg, span)
    }

    fn expect_punct(&mut self, c: char) -> Result<Span, Diagnostic> {
        match self.peek() {
            Some(Token {
                tok: Tok::Punct(p),
                span,
            }) if *p == c => {
                let s = *span;
                self.pos += 1;
                Ok(s)
            }
            Some(t) => Err(self.err(
                "text.unexpected-token",
                format!("expected `{}`, found `{}`", c, t.span.slice(self.src)),
                t.span,
            )),
            None => Err(self.err(
                "text.unexpected-eof",
                format!("expected `{}`", c),
                eof_span(self.src),
            )),
        }
    }

    fn parse_node(&mut self) -> Result<SyntaxNode, Diagnostic> {
        let tag_tok = match self.peek() {
            Some(Token {
                tok: Tok::Ident(name),
                span,
            }) => {
                let name = name.clone();
                let span = *span;
                self.pos += 1;
                (name, span)
            }
            Some(t) => {
                return Err(self.err(
                    "text.unexpected-token",
                    format!("expected node tag, found `{}`", t.span.slice(self.src)),
                    t.span,
                ))
            }
            None => {
                return Err(self.err(
                    "text.unexpected-eof",
                    "expected node tag",
                    eof_span(self.src),
                ))
            }
        };
        let mut end = tag_tok.1;

        // Primary slots: value tokens NOT followed by `:` (those are fields).
        let mut primary = Vec::new();
        loop {
            let is_value = self.peek().map_or(false, |t| value_starts(&t.tok));
            if !is_value {
                break;
            }
            let followed_by_colon = self.peek2().map_or(false, |t| t.tok == Tok::Punct(':'));
            if followed_by_colon {
                break;
            }
            let t = self.bump();
            let v = match t.tok {
                Tok::Ident(s) => Value::Ident(s),
                Tok::Str(s) => Value::Str(s),
                Tok::Int {
                    negative,
                    digits,
                    suffix,
                } => Value::Int(IntLit {
                    negative,
                    digits,
                    suffix,
                    span: t.span,
                }),
                _ => unreachable!("primary loop only enters on value tokens"),
            };
            end = end.join(t.span);
            primary.push((v, t.span));
        }

        // Optional head field list in parentheses.
        let mut head = Vec::new();
        if matches!(
            self.peek(),
            Some(Token {
                tok: Tok::Punct('('),
                ..
            })
        ) {
            self.pos += 1;
            loop {
                if matches!(
                    self.peek(),
                    Some(Token {
                        tok: Tok::Punct(')'),
                        ..
                    })
                ) {
                    self.pos += 1;
                    break;
                }
                head.push(self.parse_field(")")?);
                match self.peek().map(|t| t.tok.clone()) {
                    Some(Tok::Punct(',')) => {
                        self.pos += 1;
                    }
                    Some(Tok::Punct(')')) => {}
                    Some(_) => {
                        let sp = self.toks[self.pos].span;
                        return Err(self.err(
                            "text.unexpected-token",
                            format!("expected `,` or `)`, found `{}`", sp.slice(self.src)),
                            sp,
                        ));
                    }
                    None => {
                        return Err(self.err(
                            "text.unexpected-eof",
                            "expected `,` or `)`",
                            eof_span(self.src),
                        ))
                    }
                }
            }
            end = end.join(Span {
                start: end.end,
                end: end.end,
            });
        }

        // Optional body.
        let mut body = Vec::new();
        if matches!(
            self.peek(),
            Some(Token {
                tok: Tok::Punct('{'),
                ..
            })
        ) {
            self.pos += 1;
            loop {
                match self.peek().map(|t| t.tok.clone()) {
                    Some(Tok::Punct('}')) => {
                        let close = self.bump();
                        end = end.join(close.span);
                        break;
                    }
                    Some(Tok::Punct(';')) | Some(Tok::Punct(',')) => {
                        self.pos += 1;
                    }
                    Some(Tok::Ident(_)) => {
                        let followed_by_colon =
                            self.peek2().map_or(false, |t| t.tok == Tok::Punct(':'));
                        if followed_by_colon {
                            body.push(Item::Field(self.parse_field("}")?));
                        } else {
                            body.push(Item::Node(self.parse_node()?));
                        }
                    }
                    Some(tok) => {
                        let sp = self.toks[self.pos].span;
                        return Err(self.err(
                            "text.unexpected-token",
                            format!(
                                "expected field, child node or `}}`, found `{}`",
                                render_tok(&tok)
                            ),
                            sp,
                        ));
                    }
                    None => {
                        return Err(self.err(
                            "text.unexpected-eof",
                            "unterminated node body",
                            eof_span(self.src),
                        ))
                    }
                }
            }
        }

        let tag_span = tag_tok.1;
        Ok(SyntaxNode {
            tag: tag_tok.0,
            tag_span,
            primary,
            head,
            body,
            span: tag_span.join(end),
        })
    }

    fn parse_field(&mut self, _ctx: &str) -> Result<NamedValue, Diagnostic> {
        let name_tok = self.bump();
        let name = match &name_tok.tok {
            Tok::Ident(s) => s.clone(),
            _ => {
                return Err(self.err(
                    "text.unexpected-token",
                    "expected field name",
                    name_tok.span,
                ))
            }
        };
        self.expect_punct(':')?;
        let (value, value_span) = self.parse_value()?;
        Ok(NamedValue {
            name,
            name_span: name_tok.span,
            value,
            value_span,
        })
    }

    fn parse_value(&mut self) -> Result<(Value, Span), Diagnostic> {
        let t = self.bump();
        let span = t.span;
        match t.tok {
            Tok::Ident(s) => Ok((Value::Ident(s), span)),
            Tok::Str(s) => Ok((Value::Str(s), span)),
            Tok::Int {
                negative,
                digits,
                suffix,
            } => Ok((
                Value::Int(IntLit {
                    negative,
                    digits,
                    suffix,
                    span,
                }),
                span,
            )),
            Tok::Punct('[') => {
                let mut items = Vec::new();
                loop {
                    if matches!(
                        self.peek(),
                        Some(Token {
                            tok: Tok::Punct(']'),
                            ..
                        })
                    ) {
                        self.pos += 1;
                        break;
                    }
                    let (v, _) = self.parse_value()?;
                    items.push(v);
                    match self.peek().map(|t| t.tok.clone()) {
                        Some(Tok::Punct(',')) => self.pos += 1,
                        Some(Tok::Punct(']')) => {}
                        _ => {
                            let sp = self.peek().map(|t| t.span).unwrap_or(eof_span(self.src));
                            return Err(self.err(
                                "text.unexpected-token",
                                "expected `,` or `]` in list",
                                sp,
                            ));
                        }
                    }
                }
                Ok((Value::List(items), span))
            }
            Tok::Punct('{') => {
                let mut fields = Vec::new();
                loop {
                    if matches!(
                        self.peek(),
                        Some(Token {
                            tok: Tok::Punct('}'),
                            ..
                        })
                    ) {
                        self.pos += 1;
                        break;
                    }
                    let f = self.parse_field("}")?;
                    fields.push(f);
                    match self.peek().map(|t| t.tok.clone()) {
                        Some(Tok::Punct(',')) => self.pos += 1,
                        Some(Tok::Punct('}')) => {}
                        _ => {
                            let sp = self.peek().map(|t| t.span).unwrap_or(eof_span(self.src));
                            return Err(self.err(
                                "text.unexpected-token",
                                "expected `,` or `}` in object",
                                sp,
                            ));
                        }
                    }
                }
                Ok((Value::Obj(fields), span))
            }
            other => Err(self.err(
                "text.unexpected-token",
                format!("expected value, found `{}`", render_tok(&other)),
                span,
            )),
        }
    }
}

fn render_tok(tok: &Tok) -> String {
    match tok {
        Tok::Ident(s) => s.clone(),
        Tok::Str(_) => "string".to_string(),
        Tok::Int { .. } => "number".to_string(),
        Tok::Punct(c) => c.to_string(),
    }
}

fn eof_span(src: &str) -> Span {
    Span {
        start: src.len(),
        end: src.len(),
    }
}
