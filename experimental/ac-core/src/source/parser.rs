//! Source-subset parser for the `auto.source.core-i32.draft` profile
//! (PLAN-745 T-02).
//!
//! Recursive descent over [`super::lexer`] tokens. Grammar (plan §5.2):
//!
//! ```text
//! Program  := FnItem (sep FnItem)* sep? EOF        // at least one item
//! FnItem   := "fn" Ident "(" [Param ("," Param)*] ")" RetType Block
//! Param    := Ident ("int"|"i32"|"bool")
//! RetType  := ("int"|"i32"|"bool")                 // required
//! Block    := "{" Stmt* "}"
//! Stmt     := ("let"|"var") Ident Ann? "=" Expr
//!           | "return" Expr | "if" Expr Block ["else" Block]
//!           | "while" Expr Block | "loop" Block
//!           | "break" | "continue" | Ident "=" Expr
//! Expr     := primary (op Expr)*                   // "<" < "+" < "*", left
//! primary  := ["-"] digits | "true" | "false" | Ident ["(" args ")"] | "(" Expr ")"
//! args     := arg ("," arg)* ; arg := Expr | Ident ":" Expr
//! ```
//!
//! Statements are separated by `;` and/or a newline; an unbracketed
//! expression never continues across a newline (inside parentheses and
//! argument lists newlines are whitespace). A `-` immediately before digits
//! (whitespace allowed) is one signed literal, range-checked against the
//! exact i32 domain.
//!
//! Diagnostic codes (stage `text`): `source.lex` (lexer), `source.syntax`
//! (structural), `source.literal-range` (i32 domain), `source.unsupported`
//! (excluded-surface constructs, pointed at the offending token).

use super::ast::*;
use super::lexer::{Kw, Tok, Token, P};
use crate::atom_text::{Diagnostic, Span, Stage};

/// Full-language top-level starter words that this profile rejects with
/// `source.unsupported` (rather than a plain syntax error) when they appear
/// in item position. Anything else that is not `fn` is a syntax error.
const TOPLEVEL_UNSUPPORTED_WORDS: &[&str] = &[
    "use", "type", "enum", "struct", "trait", "impl", "const", "static", "extern", "pub",
    "checked", "unsafe", "macro", "mod", "import", "package",
];

const I32_MIN_MAG: u128 = 2147483648;
const I32_MAX: u128 = 2147483647;

pub fn parse_program(src: &str) -> Result<Program, Diagnostic> {
    let toks = super::lexer::tokenize(src)?;
    let mut p = Parser {
        src,
        toks,
        pos: 0,
        last_line: 1,
        unbracketed_entry: false,
        depth: 0,
    };
    p.items()
}

struct Parser<'s> {
    src: &'s str,
    toks: Vec<Token>,
    pos: usize,
    /// Line of the most recently consumed token.
    last_line: usize,
    /// True until the current unbracketed expression consumes its first
    /// token (see [`Parser::expr`]).
    unbracketed_entry: bool,
    /// Syntactic nesting depth of the RECURSIVE structures only — call
    /// argument lists and blocks (paren groups collapse iteratively and
    /// carry no count; R4-QA-02). Bounded by [`NEST_LIMIT`] so deeply
    /// malformed input gets a controlled diagnostic instead of a stack
    /// overflow.
    depth: usize,
}

/// Call/argument-list and block nesting limit — a resource guard tied to
/// the actual per-level recursion (parser + resolve + emit + verify walk
/// each level). 1000 is ~4x below the measured overflow depth of the
/// smallest-stack stage on this toolchain, and orders of magnitude above
/// the corpus (max ~4) and the review boundary probes (199..300 levels).
const NEST_LIMIT: usize = 1000;

impl<'s> Parser<'s> {
    // -- plumbing -----------------------------------------------------------

    fn peek(&self) -> Option<&Token> {
        self.toks.get(self.pos)
    }

    fn peek2(&self) -> Option<&Token> {
        self.toks.get(self.pos + 1)
    }

    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        self.pos += 1;
        self.last_line = t.line;
        t
    }

    fn eof_span(&self) -> Span {
        Span {
            start: self.src.len(),
            end: self.src.len(),
        }
    }

    fn err(&self, code: &str, msg: impl Into<String>, span: Span) -> Diagnostic {
        Diagnostic::new(Stage::Text, code, msg, span)
    }

    fn err_at(&self, code: &str, msg: impl Into<String>) -> Diagnostic {
        match self.peek() {
            Some(t) => self.err(code, msg, t.span),
            None => self.err(code, msg, self.eof_span()),
        }
    }

    fn syntax(&self, msg: impl Into<String>) -> Diagnostic {
        self.err_at("source.syntax", msg)
    }

    fn unsupported(&self, msg: impl Into<String>, span: Span) -> Diagnostic {
        self.err("source.unsupported", msg, span)
    }

    fn describe_cur(&self) -> String {
        match self.peek() {
            Some(t) => match &t.tok {
                Tok::Ident(s) => format!("`{}`", s),
                Tok::Kw(k) => format!("`{}`", k.as_str()),
                Tok::Reserved(w) => format!("`{}`", w),
                Tok::Punct(p) => format!("`{}`", p.as_str()),
                Tok::Int(_) => "integer literal".to_string(),
                Tok::Raw { kind, .. } => kind.describe().to_string(),
            },
            None => "end of file".to_string(),
        }
    }

    fn expect_kw(&mut self, kw: Kw, what: &str) -> Result<Span, Diagnostic> {
        match self.peek() {
            Some(Token {
                tok: Tok::Kw(k),
                span,
                ..
            }) if *k == kw => {
                let s = *span;
                self.pos += 1;
                self.last_line = self.toks[self.pos - 1].line;
                Ok(s)
            }
            _ => Err(self.syntax(format!("expected {}, found {}", what, self.describe_cur()))),
        }
    }

    fn expect_punct(&mut self, p: P, _what: &str) -> Result<Span, Diagnostic> {
        match self.peek() {
            Some(Token {
                tok: Tok::Punct(q),
                span,
                ..
            }) if *q == p => {
                let s = *span;
                self.pos += 1;
                self.last_line = self.toks[self.pos - 1].line;
                Ok(s)
            }
            _ => Err(self.syntax(format!(
                "expected `{}`, found {}",
                p.as_str(),
                self.describe_cur()
            ))),
        }
    }

    fn expect_ident(&mut self, what: &str) -> Result<Ident, Diagnostic> {
        match self.peek().cloned() {
            Some(Token {
                tok: Tok::Ident(name),
                span,
                line,
            }) => {
                self.pos += 1;
                self.last_line = line;
                Ok(Ident { name, span })
            }
            // An excluded-surface token where a name is expected is an
            // unsupported surface, not a structural error (N49). Keywords
            // — the profile's own or reserved full-language ones — can
            // never be names (C12).
            Some(Token {
                tok: Tok::Raw { kind, .. },
                span,
                ..
            }) => Err(self.unsupported(
                format!(
                    "{} cannot be used as a name in this source profile",
                    kind.describe()
                ),
                span,
            )),
            Some(Token { tok, span, .. }) if matches!(tok, Tok::Kw(_) | Tok::Reserved(_)) => {
                let word = match tok {
                    Tok::Kw(k) => k.as_str().to_string(),
                    Tok::Reserved(w) => w.clone(),
                    _ => unreachable!(),
                };
                Err(self.unsupported(format!("keyword `{}` cannot be used as a name", word), span))
            }
            _ => Err(self.syntax(format!("expected {}, found {}", what, self.describe_cur()))),
        }
    }

    /// Enter one nesting level; input that would drive the sinks past
    /// [`NEST_LIMIT`] gets a controlled diagnostic (never a stack overflow).
    fn enter(&mut self) -> Result<(), Diagnostic> {
        self.depth += 1;
        if self.depth > NEST_LIMIT {
            return Err(self.err_at(
                "source.syntax",
                format!("nesting deeper than {} levels is not accepted", NEST_LIMIT),
            ));
        }
        Ok(())
    }

    fn skip_semis(&mut self) {
        while matches!(
            self.peek(),
            Some(Token {
                tok: Tok::Punct(P::Semi),
                ..
            })
        ) {
            self.pos += 1;
        }
    }

    // -- top level ----------------------------------------------------------

    fn items(&mut self) -> Result<Program, Diagnostic> {
        if self.toks.is_empty() {
            // N53: an empty file has no function item at all.
            return Err(self.syntax("expected `fn`, found end of file"));
        }
        let mut items = Vec::new();
        loop {
            self.skip_semis();
            if self.peek().is_none() {
                break;
            }
            items.push(self.item()?);
        }
        Ok(Program { items })
    }

    fn item(&mut self) -> Result<FnItem, Diagnostic> {
        match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Kw(Kw::Fn)) => self.fn_item(),
            Some(Tok::Ident(name)) => {
                if TOPLEVEL_UNSUPPORTED_WORDS.contains(&name.as_str()) {
                    let span = self.peek().unwrap().span;
                    Err(self.unsupported(
                        format!("`{}` is not part of this source profile", name),
                        span,
                    ))
                } else {
                    Err(self.syntax(format!("expected `fn`, found `{}`", name)))
                }
            }
            Some(Tok::Kw(k)) => {
                let span = self.peek().unwrap().span;
                Err(self.unsupported(
                    format!("`{}` is not part of this source profile", k.as_str()),
                    span,
                ))
            }
            Some(Tok::Reserved(word)) => {
                let span = self.peek().unwrap().span;
                Err(self.unsupported(
                    format!("`{}` is not part of this source profile", word),
                    span,
                ))
            }
            Some(Tok::Raw { kind, .. }) => {
                let span = self.peek().unwrap().span;
                Err(self.unsupported(
                    format!("{} is not part of this source profile", kind.describe()),
                    span,
                ))
            }
            _ => Err(self.syntax(format!("expected `fn`, found {}", self.describe_cur()))),
        }
    }

    fn fn_item(&mut self) -> Result<FnItem, Diagnostic> {
        let fn_kw = self.expect_kw(Kw::Fn, "`fn`")?;
        let name = self.expect_ident("function name")?;
        self.expect_punct(P::LParen, "`(`")?;
        let mut params = Vec::new();
        if !matches!(
            self.peek(),
            Some(Token {
                tok: Tok::Punct(P::RParen),
                ..
            })
        ) {
            loop {
                let pname = self.expect_ident("parameter name")?;
                let ty = self.param_type(&pname)?;
                params.push(Param {
                    span: pname.span.join(ty.span),
                    name: pname,
                    ty,
                });
                match self.peek().map(|t| t.tok.clone()) {
                    Some(Tok::Punct(P::Comma)) => {
                        self.pos += 1;
                    }
                    Some(Tok::Punct(P::RParen)) => break,
                    _ => {
                        return Err(self.syntax(format!(
                            "expected `,` or `)`, found {}",
                            self.describe_cur()
                        )))
                    }
                }
            }
        }
        let rparen = self.expect_punct(P::RParen, "`)`")?;
        let ret = match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Kw(Kw::Int)) => self.type_word(),
            Some(Tok::Kw(Kw::I32)) => self.type_word(),
            Some(Tok::Kw(Kw::Bool)) => self.type_word(),
            Some(Tok::Punct(P::LBrace)) => {
                // N42: missing return annotation — point at the signature.
                return Err(self.unsupported(
                    "function return type annotation (`int`/`i32`/`bool`) is required",
                    name.span.join(rparen),
                ));
            }
            Some(Tok::Ident(_)) | Some(Tok::Reserved(_)) => {
                let t = self.peek().unwrap();
                return Err(self.unsupported(
                    format!(
                        "type `{}` is not part of this source profile",
                        t.span.slice(self.src)
                    ),
                    t.span,
                ));
            }
            _ => {
                return Err(self.syntax(format!(
                    "expected return type, found {}",
                    self.describe_cur()
                )))
            }
        };
        let body = self.block()?;
        let last = self.toks[self.pos - 1].span;
        Ok(FnItem {
            span: fn_kw.join(last),
            name,
            params,
            ret,
            body,
        })
    }

    fn type_word(&mut self) -> TypeAnn {
        let t = self.bump();
        let word = match t.tok {
            Tok::Kw(Kw::Int) => TypeWord::Int,
            Tok::Kw(Kw::I32) => TypeWord::I32,
            Tok::Kw(Kw::Bool) => TypeWord::Bool,
            _ => unreachable!("type_word called on non-type token"),
        };
        TypeAnn { word, span: t.span }
    }

    fn is_type_word(tok: &Tok) -> bool {
        matches!(tok, Tok::Kw(Kw::Int) | Tok::Kw(Kw::I32) | Tok::Kw(Kw::Bool))
    }

    /// Type annotation in parameter position: required, only the three
    /// profile words. Any other identifier is an excluded type surface.
    fn param_type(&mut self, pname: &Ident) -> Result<TypeAnn, Diagnostic> {
        match self.peek().map(|t| t.tok.clone()) {
            Some(t) if Self::is_type_word(&t) => Ok(self.type_word()),
            Some(Tok::Ident(_)) | Some(Tok::Reserved(_)) => {
                let t = self.peek().unwrap();
                Err(self.unsupported(
                    format!(
                        "type `{}` is not part of this source profile",
                        t.span.slice(self.src)
                    ),
                    t.span,
                ))
            }
            _ => Err(self.unsupported(
                "parameter type annotation (`int`/`i32`/`bool`) is required",
                pname.span,
            )),
        }
    }

    /// Optional annotation in `let`/`var` declarations.
    fn decl_ann(&mut self) -> Result<Option<TypeAnn>, Diagnostic> {
        match self.peek().map(|t| t.tok.clone()) {
            Some(t) if Self::is_type_word(&t) => Ok(Some(self.type_word())),
            Some(Tok::Ident(_)) | Some(Tok::Reserved(_)) => {
                let t = self.peek().unwrap();
                Err(self.unsupported(
                    format!(
                        "type `{}` is not part of this source profile",
                        t.span.slice(self.src)
                    ),
                    t.span,
                ))
            }
            _ => Ok(None),
        }
    }

    // -- statements ---------------------------------------------------------

    fn block(&mut self) -> Result<Block, Diagnostic> {
        self.enter()?;
        let lb = self.expect_punct(P::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        loop {
            self.skip_semis();
            match self.peek() {
                None => return Err(self.syntax("expected statement or `}`, found end of file")),
                Some(Token {
                    tok: Tok::Punct(P::RBrace),
                    ..
                }) => {
                    let rb = self.bump();
                    self.depth -= 1;
                    return Ok(Block {
                        stmts,
                        span: lb.join(rb.span),
                    });
                }
                Some(_) => {
                    stmts.push(self.stmt()?);
                    // Statement separator: `;`, `}` or a newline; anything
                    // else on the same line is a structural error.
                    match self.peek() {
                        None => {}
                        Some(Token {
                            tok: Tok::Punct(P::Semi),
                            ..
                        })
                        | Some(Token {
                            tok: Tok::Punct(P::RBrace),
                            ..
                        }) => {}
                        Some(t) if t.line > self.last_line => {}
                        Some(_) => {
                            return Err(self.syntax(format!(
                                "expected `;`, newline or `}}`, found {}",
                                self.describe_cur()
                            )))
                        }
                    }
                }
            }
        }
    }

    fn stmt(&mut self) -> Result<Stmt, Diagnostic> {
        match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Kw(Kw::Let)) | Some(Tok::Kw(Kw::Var)) => self.decl_stmt(),
            Some(Tok::Kw(Kw::Return)) => {
                let kw = self.bump();
                let value = self.expr(false)?;
                let span = kw.span.join(value.span);
                Ok(Stmt::Return { value, span })
            }
            Some(Tok::Kw(Kw::If)) => {
                let kw = self.bump();
                let cond = self.expr(false)?;
                let then = self.block()?;
                let mut end = self.toks[self.pos - 1].span;
                let els = match self.peek() {
                    Some(Token {
                        tok: Tok::Kw(Kw::Else),
                        ..
                    }) => {
                        self.pos += 1;
                        self.last_line = self.toks[self.pos - 1].line;
                        Some(self.block()?)
                    }
                    _ => None,
                };
                if let Some(b) = &els {
                    end = end.join(b.span);
                }
                Ok(Stmt::If {
                    cond,
                    then,
                    els,
                    span: kw.span.join(end),
                })
            }
            Some(Tok::Kw(Kw::While)) => {
                let kw = self.bump();
                let cond = self.expr(false)?;
                let body = self.block()?;
                let span = kw.span.join(body.span);
                Ok(Stmt::While { cond, body, span })
            }
            Some(Tok::Kw(Kw::Loop)) => {
                let kw = self.bump();
                let body = self.block()?;
                let span = kw.span.join(body.span);
                Ok(Stmt::Loop { body, span })
            }
            Some(Tok::Kw(Kw::Break)) => {
                let t = self.bump();
                Ok(Stmt::Break { span: t.span })
            }
            Some(Tok::Kw(Kw::Continue)) => {
                let t = self.bump();
                Ok(Stmt::Continue { span: t.span })
            }
            Some(Tok::Ident(_))
                if matches!(self.peek2().map(|t| &t.tok), Some(Tok::Punct(P::Assign))) =>
            {
                let name = self.expect_ident("local name")?;
                self.expect_punct(P::Assign, "`=`")?;
                let value = self.expr(false)?;
                Ok(Stmt::Assign {
                    span: name.span.join(value.span),
                    name,
                    value,
                })
            }
            Some(Tok::Ident(_))
            | Some(Tok::Kw(Kw::True))
            | Some(Tok::Kw(Kw::False))
            | Some(Tok::Int(_)) => {
                // N41: an expression statement (e.g. a bare call) — calls are
                // only allowed where a value is consumed.
                let e = self.expr(false)?;
                Err(self.unsupported(
                    "expression statements are not part of this source profile; calls are only allowed in initializers, assignments, returns and conditions",
                    e.span,
                ))
            }
            Some(Tok::Reserved(word)) => {
                let span = self.peek().unwrap().span;
                Err(self.unsupported(
                    format!("`{}` is not part of this source profile", word),
                    span,
                ))
            }
            _ => Err(self.syntax(format!("expected statement, found {}", self.describe_cur()))),
        }
    }

    fn decl_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let kw_tok = self.bump();
        let mutable = kw_tok.tok == Tok::Kw(Kw::Var);
        let name = self.expect_ident("local name")?;
        let ann = self.decl_ann()?;
        match self.peek() {
            Some(Token {
                tok: Tok::Punct(P::Assign),
                ..
            }) => {
                self.pos += 1;
                self.last_line = self.toks[self.pos - 1].line;
            }
            _ => {
                // N19: declarations are always initialized; the span runs
                // from the keyword to the last consumed token.
                let last = self.toks[self.pos - 1].span;
                return Err(self.unsupported(
                    "locals must be initialized at declaration in this source profile",
                    kw_tok.span.join(last),
                ));
            }
        }
        let value = self.expr(false)?;
        Ok(Stmt::Let {
            span: kw_tok.span.join(value.span),
            kw: kw_tok.span,
            name,
            ann,
            mutable,
            value,
        })
    }

    // -- expressions --------------------------------------------------------

    /// Parse an expression. `bracketed` disables the same-line rule (inside
    /// parentheses/argument lists, newlines are whitespace).
    ///
    /// Line discipline (R1-QA-01 / R2-QA-01): an unbracketed expression may
    /// START on the current or a later line, but every token AFTER the
    /// first must sit on the same line as the previously consumed token.
    /// The anchor moves forward legitimately when a bracketed
    /// subexpression (parenthesized group or call arguments) closes on a
    /// later line, so `f(\n 2\n) + 3` continues on the `)`'s line, while
    /// `1 +\n 2` and `f\n(3)` still reject. Implemented through
    /// [`Parser::unbracketed_entry`] and the running `last_line`.
    fn expr(&mut self, bracketed: bool) -> Result<Expr, Diagnostic> {
        if !bracketed {
            self.unbracketed_entry = true;
        }
        self.binary(1, bracketed)
    }

    /// Operators that exist in the wider language but are outside the
    /// profile; encountering one in operator position is `source.unsupported`
    /// at the operator token.
    fn unsupported_op(p: P) -> bool {
        matches!(
            p,
            P::Minus
                | P::Slash
                | P::Percent
                | P::Gt
                | P::EqEq
                | P::NotEq
                | P::LeEq
                | P::GeEq
                | P::AmpAmp
                | P::PipePipe
                | P::Bang
                | P::Dot
        )
    }

    fn binary(&mut self, min_prec: u8, bracketed: bool) -> Result<Expr, Diagnostic> {
        let lhs = self.unary(bracketed)?;
        self.binary_continue(min_prec, bracketed, lhs)
    }

    /// Continue the operator loop from an already-parsed left operand
    /// (used by the paren collapse below; precedence/line rules unchanged).
    fn binary_continue(
        &mut self,
        min_prec: u8,
        bracketed: bool,
        mut lhs: Expr,
    ) -> Result<Expr, Diagnostic> {
        loop {
            let Some(tok) = self.peek().cloned() else {
                break;
            };
            let Tok::Punct(p) = tok.tok else { break };
            if !bracketed && tok.line > self.last_line {
                // An unbracketed expression does not continue across a
                // newline: the operator starts a new line, so the expression
                // ends here and the caller decides what the token means.
                break;
            }
            let (op, prec) = match p {
                P::Plus => (BinOp::Add, 2),
                P::Star => (BinOp::Mul, 3),
                P::Lt => (BinOp::Lt, 1),
                other => {
                    if Self::unsupported_op(other) {
                        return Err(self.unsupported(
                            format!(
                                "operator `{}` is not part of this source profile",
                                other.as_str()
                            ),
                            tok.span,
                        ));
                    }
                    break;
                }
            };
            if prec < min_prec {
                break;
            }
            self.pos += 1;
            self.last_line = tok.line;
            let rhs = self.binary(prec + 1, bracketed)?;
            lhs = Expr {
                span: lhs.span.join(rhs.span),
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            };
        }
        Ok(lhs)
    }

    fn unary(&mut self, bracketed: bool) -> Result<Expr, Diagnostic> {
        // Line boundary of an unbracketed expression: the ENTRY token may
        // start on any line (a statement may carry its value on the next
        // line); every subsequent token must share the line of the token
        // consumed just before it. `last_line` advances when a bracketed
        // subexpression closes, so continuation after `)` keys to the
        // CLOSING line (R2-QA-01: D01..D05 legal) while `1 +\n 2`, `f\n(3)`
        // and `-\n 2` still reject.
        let entry = self.unbracketed_entry;
        self.unbracketed_entry = false;
        if !bracketed && !entry {
            if let Some(t) = self.peek() {
                if t.line != self.last_line {
                    return Err(self.err(
                        "source.syntax",
                        "unbracketed expression cannot continue across a newline",
                        t.span,
                    ));
                }
            }
        }
        self.unary_inner(bracketed)
    }

    fn unary_inner(&mut self, bracketed: bool) -> Result<Expr, Diagnostic> {
        match self.peek().map(|t| (t.tok.clone(), t.span)) {
            Some((Tok::Punct(P::Minus), mspan)) => {
                match self.peek2().map(|t| t.tok.clone()) {
                    Some(Tok::Int(_)) => {
                        self.pos += 1; // minus
                        self.last_line = self.toks[self.pos - 1].line;
                        if !bracketed {
                            if let Some(t2) = self.peek() {
                                if t2.line != self.last_line {
                                    return Err(self.err(
                                        "source.syntax",
                                        "unbracketed expression cannot continue across a newline",
                                        t2.span,
                                    ));
                                }
                            }
                        }
                        let t = self.bump();
                        let Tok::Int(digits) = t.tok else {
                            unreachable!()
                        };
                        let span = mspan.join(t.span);
                        let mag: u128 = match digits.parse() {
                            Ok(v) => v,
                            Err(_) => u128::MAX, // absurd magnitude: out of range
                        };
                        if mag > I32_MIN_MAG {
                            return Err(self.err(
                                "source.literal-range",
                                format!(
                                    "integer literal `-{}` is outside the signed 32-bit range of this source profile",
                                    digits
                                ),
                                span,
                            ));
                        }
                        Ok(Expr {
                            span,
                            kind: ExprKind::IntLit((mag as i128 * -1) as i32),
                        })
                    }
                    Some(Tok::Ident(_))
                    | Some(Tok::Kw(Kw::True))
                    | Some(Tok::Kw(Kw::False))
                    | Some(Tok::Punct(P::LParen)) => {
                        // N36: unary minus on anything but a literal is
                        // outside the profile; span covers `-x`.
                        let end = self.peek2().unwrap().span;
                        return Err(self.unsupported(
                            "unary minus is only supported directly before an integer literal in this source profile",
                            mspan.join(end),
                        ));
                    }
                    _ => Err(self.syntax("expected expression after `-`")),
                }
            }
            Some((Tok::Int(_), _)) => {
                let t = self.bump();
                let Tok::Int(digits) = t.tok else {
                    unreachable!()
                };
                let mag: u128 = match digits.parse() {
                    Ok(v) => v,
                    Err(_) => u128::MAX,
                };
                if mag > I32_MAX {
                    return Err(self.err(
                        "source.literal-range",
                        format!(
                            "integer literal `{}` is outside the signed 32-bit range of this source profile",
                            digits
                        ),
                        t.span,
                    ));
                }
                Ok(Expr {
                    span: t.span,
                    kind: ExprKind::IntLit(mag as i32),
                })
            }
            Some((Tok::Kw(Kw::True), _)) => {
                let t = self.bump();
                Ok(Expr {
                    span: t.span,
                    kind: ExprKind::BoolLit(true),
                })
            }
            Some((Tok::Kw(Kw::False), _)) => {
                let t = self.bump();
                Ok(Expr {
                    span: t.span,
                    kind: ExprKind::BoolLit(false),
                })
            }
            Some((Tok::Ident(_), _)) => {
                // A call's `(` must sit on the callee's line unless the
                // whole expression is bracketed (C03: a parenthesized
                // argument list on the NEXT line is a new statement, not a
                // call of the previous identifier). The callee may itself
                // be the expression's entry token, so anchor to IT, not to
                // the line of whatever came before.
                let paren_same_line = match (self.peek(), self.peek2()) {
                    (Some(idt), Some(t)) if t.tok == Tok::Punct(P::LParen) => {
                        bracketed || t.line == idt.line
                    }
                    _ => false,
                };
                if paren_same_line {
                    let callee = self.expect_ident("callee")?;
                    // one SYNTACTIC nesting level (call argument list)
                    self.enter()?;
                    self.expect_punct(P::LParen, "`(`")?;
                    let r = self.call_args(callee);
                    self.depth -= 1;
                    r
                } else {
                    let id = self.expect_ident("name")?;
                    Ok(Expr {
                        span: id.span,
                        kind: ExprKind::Var(id),
                    })
                }
            }
            Some((Tok::Punct(P::Bang), bspan)) => {
                Err(self.unsupported("unary `!` is not part of this source profile", bspan))
            }
            Some((Tok::Reserved(word), span)) => Err(self.unsupported(
                format!("keyword `{}` is not part of this source profile", word),
                span,
            )),
            Some((Tok::Punct(P::LParen), _)) => {
                // Consecutive parenthesized groups parse ITERATIVELY
                // (R4-QA-02): every `(` is consumed on a flat loop, the
                // core expression parses once (inside parens newlines are
                // whitespace), and each `)` closes one pending group. When
                // a `)` arrives before all groups are closed, the
                // expression CONTINUES at top precedence with the current
                // left operand (`((1+2)*3)` resumes after the inner
                // close), so mixed nesting stays correct while homogeneous
                // `(((expr)))` costs zero recursion. Pure grouping adds no
                // AST node, so legal paren nesting carries no limit;
                // unclosed inputs fail at EOF depth-independently.
                let mut opens = 0usize;
                while matches!(
                    self.peek(),
                    Some(Token {
                        tok: Tok::Punct(P::LParen),
                        ..
                    })
                ) {
                    self.pos += 1;
                    self.last_line = self.toks[self.pos - 1].line;
                    opens += 1;
                }
                let mut inner = self.expr(true)?;
                let mut closed = 0usize;
                while closed < opens {
                    while closed < opens
                        && matches!(
                            self.peek(),
                            Some(Token {
                                tok: Tok::Punct(P::RParen),
                                ..
                            })
                        )
                    {
                        self.pos += 1;
                        self.last_line = self.toks[self.pos - 1].line;
                        closed += 1;
                    }
                    if closed == opens {
                        break;
                    }
                    let before = self.pos;
                    inner = self.binary_continue(1, true, inner)?;
                    if self.pos == before {
                        // neither a `)` nor a continuation operator:
                        // unterminated or malformed group
                        return Err(self.syntax(format!(
                            "expected `)` or operator, found {}",
                            self.describe_cur()
                        )));
                    }
                }
                Ok(inner)
            }
            Some((Tok::Raw { kind, text }, span)) => Err(self.unsupported(
                format!(
                    "{} `{}` is not part of this source profile",
                    kind.describe(),
                    text
                ),
                span,
            )),
            _ => Err(self.syntax(format!(
                "expected expression, found {}",
                self.describe_cur()
            ))),
        }
    }

    /// Parse a call's argument list after the opening `(` was consumed.
    /// The nesting level is accounted by the caller.
    fn call_args(&mut self, callee: Ident) -> Result<Expr, Diagnostic> {
        let mut args = Vec::new();
        if !matches!(
            self.peek(),
            Some(Token {
                tok: Tok::Punct(P::RParen),
                ..
            })
        ) {
            loop {
                // Named argument: Ident `:` expr.
                if matches!(self.peek().map(|t| &t.tok), Some(Tok::Ident(_)))
                    && matches!(self.peek2().map(|t| &t.tok), Some(Tok::Punct(P::Colon)))
                {
                    let name = self.expect_ident("argument name")?;
                    self.expect_punct(P::Colon, "`:`")?;
                    let value = self.expr(true)?;
                    args.push(Arg::Named {
                        span: name.span.join(value.span),
                        name,
                        value,
                    });
                } else {
                    args.push(Arg::Pos(self.expr(true)?));
                }
                match self.peek().map(|t| t.tok.clone()) {
                    Some(Tok::Punct(P::Comma)) => {
                        self.pos += 1;
                    }
                    Some(Tok::Punct(P::RParen)) => break,
                    _ => {
                        return Err(self.syntax(format!(
                            "expected `,` or `)`, found {}",
                            self.describe_cur()
                        )))
                    }
                }
            }
        }
        let rparen = self.expect_punct(P::RParen, "`)`")?;
        Ok(Expr {
            span: callee.span.join(rparen),
            kind: ExprKind::Call { callee, args },
        })
    }
}
