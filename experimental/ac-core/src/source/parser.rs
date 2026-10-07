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
        depth: 0,
    };
    p.items()
}

// Expression-parser frame types (module scope; PLAN-745 R5-QA-01).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OpFrame {
    Bin(u8),
    Paren,
}

struct CallFrame {
    callee: Ident,
    args: Vec<Arg>,
    mark: usize,
    pending_name: Option<Ident>,
}

enum Frame {
    Op(OpFrame),
    Call(Box<CallFrame>),
}

struct Parser<'s> {
    src: &'s str,
    toks: Vec<Token>,
    pos: usize,
    /// Line of the most recently consumed token.
    last_line: usize,
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
    //
    // Fully ITERATIVE precedence-climbing parser (R5-QA-01): every nesting
    // form — binary operators, parenthesized groups, call argument lists —
    // runs on explicit value/operator stacks, so expression nesting of ANY
    // shape or depth (consecutive or interleaved, complete or unclosed)
    // parses without recursion and without a nesting limit.
    //
    // Frames:
    //   Bin(prec)        pending binary operator (left-associative)
    //   Paren            opened by `(`
    //   Call{callee,..}  opened by `ident (`
    // Line discipline (R1/R2-QA-01): outside ALL frames (the unbracketed
    // top level) an operator must share the line of the previously
    // consumed token, and a primary following an operator must share the
    // operator's line; inside any frame newlines are whitespace. This keeps
    // `1 +\n 2`, `f\n(3)`, `-\n 2` rejected while `f(\n 2\n) + 3`,
    // `(1 +\n 2) + 3` and `((1+\n2)*3)` pass (D01..D05, G02, H01).

    /// Operators that exist in the wider language but are outside the
    /// profile; in operator position they are `source.unsupported`.
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

    fn prec_of(p: P) -> Option<u8> {
        match p {
            P::Plus => Some(2),
            P::Star => Some(3),
            P::Lt => Some(1),
            _ => None,
        }
    }

    /// True when the top of `frames` is a bracket context (parens/call) —
    /// i.e. newlines are whitespace and line checks are off.
    fn in_bracket(frames: &[Frame]) -> bool {
        frames
            .iter()
            .any(|f| !matches!(f, Frame::Op(OpFrame::Bin(_))))
    }

    /// Pop and apply pending binary operators with precedence >= `min`.
    fn reduce_ops(frames: &mut Vec<Frame>, vals: &mut Vec<Expr>) {
        while let Some(Frame::Op(OpFrame::Bin(_))) = frames.last() {
            let Frame::Op(OpFrame::Bin(prec)) = frames.pop().unwrap() else {
                unreachable!()
            };
            let rhs = vals.pop().expect("bin rhs");
            let lhs = vals.pop().expect("bin lhs");
            let op = match prec {
                1 => BinOp::Lt,
                2 => BinOp::Add,
                _ => BinOp::Mul,
            };
            vals.push(Expr {
                span: lhs.span.join(rhs.span),
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            });
        }
    }

    /// Reduce the current argument (values above the call frame's mark)
    /// into the frame's arg list.
    fn finish_arg(
        frames: &mut Vec<Frame>,
        vals: &mut Vec<Expr>,
        src: &str,
    ) -> Result<(), Diagnostic> {
        // apply pending operators above the call frame's mark
        while frames
            .last()
            .is_some_and(|f| matches!(f, Frame::Op(OpFrame::Bin(_))))
        {
            Self::reduce_ops(frames, vals);
        }
        let Some(Frame::Call(cf)) = frames.last_mut() else {
            unreachable!("finish_arg with call frame on top");
        };
        if vals.len() <= cf.mark {
            // no value above the frame's mark: either a bare `name:` or an
            // empty argument slot
            let sp = cf
                .pending_name
                .as_ref()
                .map(|n| n.span)
                .unwrap_or(cf.callee.span);
            return Err(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                "expected expression after named argument",
                sp,
            ));
        }
        let value = vals.pop().expect("arg value");
        let arg = match cf.pending_name.take() {
            Some(name) => Arg::Named {
                span: name.span.join(value.span),
                name,
                value,
            },
            None => Arg::Pos(value),
        };
        cf.args.push(arg);
        cf.mark = vals.len();
        let _ = src;
        Ok(())
    }

    fn expr(&mut self, _bracketed: bool) -> Result<Expr, Diagnostic> {
        let mut frames: Vec<Frame> = Vec::new();
        let mut vals: Vec<Expr> = Vec::new();
        // line anchor for the NEXT primary at the unbracketed top level:
        // the line of the previously consumed token (operator), if any
        'outer: loop {
            // ---- primary ----
            let anchored = !Self::in_bracket(&frames)
                && frames
                    .last()
                    .is_some_and(|f| matches!(f, Frame::Op(OpFrame::Bin(_))));
            if anchored {
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
            match self.peek().cloned() {
                None => {
                    if let Some(Frame::Call(_)) = frames.last() {
                        return Err(self.syntax("expected argument expression"));
                    }
                    if let Some(Frame::Op(OpFrame::Paren)) = frames.last() {
                        return Err(self.syntax("expected `)`"));
                    }
                    if !frames.is_empty() {
                        return Err(self.syntax("unterminated expression"));
                    }
                    if vals.is_empty() {
                        return Err(self.syntax(format!(
                            "expected expression, found {}",
                            self.describe_cur()
                        )));
                    }
                    break 'outer;
                }
                Some(Token { tok, span, line }) => match tok {
                    Tok::Int(digits) => {
                        self.pos += 1;
                        self.last_line = line;
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
                                span,
                            ));
                        }
                        vals.push(Expr {
                            span,
                            kind: ExprKind::IntLit(mag as i32),
                        });
                    }
                    Tok::Punct(P::Minus) => {
                        // signed literal: `-` directly before digits
                        match self.peek2().cloned() {
                            Some(Token {
                                tok: Tok::Int(digits),
                                span: dspan,
                                line: dline,
                            }) => {
                                if !Self::in_bracket(&frames) && dline != line {
                                    return Err(self.err(
                                        "source.syntax",
                                        "unbracketed expression cannot continue across a newline",
                                        dspan,
                                    ));
                                }
                                self.pos += 2;
                                self.last_line = dline;
                                let mag: u128 = match digits.parse() {
                                    Ok(v) => v,
                                    Err(_) => u128::MAX,
                                };
                                let joined = span.join(dspan);
                                if mag > I32_MIN_MAG {
                                    return Err(self.err(
                                        "source.literal-range",
                                        format!(
                                            "integer literal `-{}` is outside the signed 32-bit range of this source profile",
                                            digits
                                        ),
                                        joined,
                                    ));
                                }
                                vals.push(Expr {
                                    span: joined,
                                    kind: ExprKind::IntLit((mag as i128 * -1) as i32),
                                });
                            }
                            Some(Token {
                                tok:
                                    Tok::Ident(_)
                                    | Tok::Kw(Kw::True)
                                    | Tok::Kw(Kw::False)
                                    | Tok::Punct(P::LParen),
                                span: end,
                                ..
                            }) => {
                                return Err(self.unsupported(
                                    "unary minus is only supported directly before an integer literal in this source profile",
                                    span.join(end),
                                ));
                            }
                            _ => return Err(self.syntax("expected expression after `-`")),
                        }
                    }
                    Tok::Kw(Kw::True) => {
                        self.pos += 1;
                        self.last_line = line;
                        vals.push(Expr {
                            span,
                            kind: ExprKind::BoolLit(true),
                        });
                    }
                    Tok::Kw(Kw::False) => {
                        self.pos += 1;
                        self.last_line = line;
                        vals.push(Expr {
                            span,
                            kind: ExprKind::BoolLit(false),
                        });
                    }
                    Tok::Punct(P::Bang) => {
                        return Err(
                            self.unsupported("unary `!` is not part of this source profile", span)
                        );
                    }
                    Tok::Reserved(word) => {
                        return Err(self.unsupported(
                            format!("keyword `{}` is not part of this source profile", word),
                            span,
                        ));
                    }
                    Tok::Raw { kind, text } => {
                        return Err(self.unsupported(
                            format!(
                                "{} `{}` is not part of this source profile",
                                kind.describe(),
                                text
                            ),
                            span,
                        ));
                    }
                    Tok::Punct(P::LParen) => {
                        self.pos += 1;
                        self.last_line = line;
                        frames.push(Frame::Op(OpFrame::Paren));
                        continue 'outer;
                    }
                    Tok::Ident(name) => {
                        // call when `(` sits on the callee's line; inside
                        // brackets a next-line `(` is still a call (parens
                        // make newlines whitespace)
                        let mut skip_var = false;
                        let is_call = matches!(
                            self.peek2(),
                            Some(t2) if t2.tok == Tok::Punct(P::LParen)
                                && (Self::in_bracket(&frames) || t2.line == line)
                        );
                        if is_call {
                            let callee = Ident {
                                name: name.clone(),
                                span,
                            };
                            self.pos += 2;
                            self.last_line = self.toks[self.pos - 1].line;
                            // empty arg list?
                            if matches!(
                                self.peek(),
                                Some(Token {
                                    tok: Tok::Punct(P::RParen),
                                    ..
                                })
                            ) {
                                let close = self.bump();
                                vals.push(Expr {
                                    span: callee.span.join(close.span),
                                    kind: ExprKind::Call {
                                        callee,
                                        args: Vec::new(),
                                    },
                                });
                                // value complete (the Var push below must
                                // be skipped — this token was the call)
                                skip_var = true;
                            } else {
                                frames.push(Frame::Call(Box::new(CallFrame {
                                    callee,
                                    args: Vec::new(),
                                    mark: vals.len(),
                                    pending_name: None,
                                })));
                                // named-arg label?
                                self.take_named_label(&mut frames)?;
                                continue 'outer;
                            }
                        }
                        if !skip_var {
                            self.pos += 1;
                            self.last_line = line;
                            vals.push(Expr {
                                span,
                                kind: ExprKind::Var(Ident { name, span }),
                            });
                        }
                    }
                    other => {
                        return Err(self.syntax(format!(
                            "expected expression, found {}",
                            match &other {
                                Tok::Punct(p) => format!("`{}`", p.as_str()),
                                Tok::Kw(k) => format!("`{}`", k.as_str()),
                                _ => "token".to_string(),
                            }
                        )));
                    }
                },
            }

            // ---- operator / close loop ----
            loop {
                let Some(tok) = self.peek().cloned() else {
                    // EOF: unwind — unclosed frames are errors
                    if let Some(Frame::Call(_)) = frames.last() {
                        if vals.len() > 0 {
                            Self::finish_arg(&mut frames, &mut vals, self.src)?;
                        }
                        return Err(self.syntax("expected `,` or `)`"));
                    }
                    if let Some(Frame::Op(OpFrame::Paren)) = frames.last() {
                        return Err(self.syntax("expected `)`"));
                    }
                    Self::reduce_ops(&mut frames, &mut vals);
                    break 'outer;
                };
                match tok.tok {
                    Tok::Punct(p) if Self::prec_of(p).is_some() => {
                        let in_br = Self::in_bracket(&frames);
                        if !in_br && tok.line != self.last_line {
                            // operator starts a new line: the expression
                            // ends; the statement layer decides
                            Self::reduce_ops(&mut frames, &mut vals);
                            break 'outer;
                        }
                        let my = Self::prec_of(p).unwrap();
                        while let Some(Frame::Op(OpFrame::Bin(top))) = frames.last() {
                            if *top >= my {
                                Self::reduce_once(&mut frames, &mut vals);
                            } else {
                                break;
                            }
                        }
                        self.pos += 1;
                        self.last_line = tok.line;
                        frames.push(Frame::Op(OpFrame::Bin(my)));
                        continue 'outer; // parse RHS primary
                    }
                    Tok::Punct(P::LParen) => {
                        // `(` in operator position after a complete value
                        return Err(self
                            .syntax(format!("expected operator, found {}", self.describe_cur())));
                    }
                    Tok::Punct(P::RParen) => {
                        // pending binary operators bind tighter than the
                        // closing bracket — apply them first
                        while let Some(Frame::Op(OpFrame::Bin(_))) = frames.last() {
                            Self::reduce_once(&mut frames, &mut vals);
                        }
                        match frames.last() {
                            Some(Frame::Op(OpFrame::Paren)) => {
                                Self::reduce_ops(&mut frames, &mut vals);
                                frames.pop();
                                self.pos += 1;
                                self.last_line = tok.line;
                                // continue: another `)` or an operator may follow
                            }
                            Some(Frame::Call(_)) => {
                                if vals.len() > 0 || matches!(frames.last(), Some(Frame::Call(_))) {
                                    let has_val = match frames.last() {
                                        Some(Frame::Call(cf)) => vals.len() > cf.mark,
                                        _ => false,
                                    };
                                    if has_val {
                                        Self::finish_arg(&mut frames, &mut vals, self.src)?;
                                    } else if let Some(Frame::Call(cf)) = frames.last() {
                                        if cf.pending_name.is_some() {
                                            return Err(self.syntax(
                                                "expected expression after named argument",
                                            ));
                                        }
                                    }
                                }
                                let Some(Frame::Call(cf)) = frames.pop() else {
                                    unreachable!()
                                };
                                self.pos += 1;
                                self.last_line = tok.line;
                                vals.push(Expr {
                                    span: cf.callee.span.join(tok.span),
                                    kind: ExprKind::Call {
                                        callee: cf.callee,
                                        args: cf.args,
                                    },
                                });
                                // continue: operator or another `)` may follow
                            }
                            _ => {
                                // no frame: expression ends at `)`
                                Self::reduce_ops(&mut frames, &mut vals);
                                break 'outer;
                            }
                        }
                    }
                    Tok::Punct(P::Comma) => {
                        if let Some(Frame::Call(_)) = frames.last() {
                            Self::finish_arg(&mut frames, &mut vals, self.src)?;
                            self.pos += 1;
                            self.last_line = tok.line;
                            self.take_named_label(&mut frames)?;
                            continue 'outer; // next argument primary
                        }
                        return Err(self.syntax("expected operator, found `,`"));
                    }
                    Tok::Punct(other) if Self::unsupported_op(other) => {
                        return Err(self.unsupported(
                            format!(
                                "operator `{}` is not part of this source profile",
                                other.as_str()
                            ),
                            tok.span,
                        ));
                    }
                    _ => {
                        // not an operator: expression ends (statement layer
                        // handles separators)
                        if let Some(Frame::Call(_)) = frames.last() {
                            return Err(self.syntax("expected `,` or `)`"));
                        }
                        if let Some(Frame::Op(OpFrame::Paren)) = frames.last() {
                            return Err(self.syntax("expected `)`"));
                        }
                        Self::reduce_ops(&mut frames, &mut vals);
                        break 'outer;
                    }
                }
            }
        }
        Ok(vals.pop().expect("expression value"))
    }

    /// At an argument start inside a call frame: consume `name :` as a
    /// named-argument label if present.
    fn take_named_label(&mut self, frames: &mut [Frame]) -> Result<(), Diagnostic> {
        let (
            Some(Token {
                tok: Tok::Ident(name),
                span,
                ..
            }),
            Some(Token {
                tok: Tok::Punct(P::Colon),
                ..
            }),
        ) = (self.peek(), self.peek2())
        else {
            return Ok(());
        };
        let name = name.clone();
        let span = *span;
        self.pos += 2;
        if let Some(Frame::Call(cf)) = frames.last_mut() {
            cf.pending_name = Some(Ident { name, span });
        }
        Ok(())
    }

    /// Pop one pending binary operator and apply it (left-associative).
    fn reduce_once(frames: &mut Vec<Frame>, vals: &mut Vec<Expr>) {
        let Some(Frame::Op(OpFrame::Bin(prec))) = frames.pop() else {
            return;
        };
        let rhs = vals.pop().expect("bin rhs");
        let lhs = vals.pop().expect("bin lhs");
        let op = match prec {
            1 => BinOp::Lt,
            2 => BinOp::Add,
            _ => BinOp::Mul,
        };
        vals.push(Expr {
            span: lhs.span.join(rhs.span),
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
        });
    }
}
