//! Source-subset parsers for the core-i32 (PLAN-745) and scalar-i32
//! (PLAN-767) source profiles.
//!
//! Recursive descent over [`super::lexer`] tokens. Core grammar (plan §5.2):
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
//! Scalar surface additions (PLAN-767 §5.2/§5.2.1): binary `|| < && <
//! == != < relational < + - < * / %` (same level left-associative),
//! right-associative unary `- !` in primary position, and the conditional
//! loop `for Expr Block` reusing the while's per-round condition semantics
//! (`while` itself is rejected on the scalar surface). Precedence low→high:
//! `||`(1) `&&`(2) `== !=`(3) `< > <= >=`(4) `+ -`(5) `* / %`(6), unary,
//! call/group. Unary frames apply the moment a complete primary value is
//! available, so `-a*b` is `(-a)*b` and `!(a&&b)` negates the group.
//!
//! Statements are separated by `;` and/or a newline; an unbracketed
//! expression never continues across a newline (inside parentheses and
//! argument lists newlines are whitespace). A `-` immediately before digits
//! (whitespace allowed) is one signed literal, range-checked against the
//! exact i32 domain; `a-1` in operator position is binary subtraction.
//!
//! Both walks are ITERATIVE on explicit stacks (R5/R6): expressions via the
//! precedence-climbing frame machine, statement blocks via an open-block
//! stack — nesting of any shape or depth parses without recursion and
//! without a nesting limit.
//!
//! Diagnostic codes (stage `text`): `source.lex` (lexer), `source.syntax`
//! (structural), `source.literal-range` (i32 domain), `source.unsupported`
//! (excluded-surface constructs, pointed at the offending token).

use super::ast::*;
use super::lexer::{Kw, Tok, Token, P};
use super::SourceProfile;
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
    parse_program_with(src, SourceProfile::Core)
}

/// Parse with an explicit profile surface (PLAN-767): the scalar surface
/// accepts the new operators, unary ops and the conditional `for` loop;
/// the core surface keeps its historical accept/reject sets.
pub fn parse_program_with(src: &str, profile: SourceProfile) -> Result<Program, Diagnostic> {
    let toks = super::lexer::tokenize(src)?;
    let mut p = Parser {
        src,
        toks,
        pos: 0,
        last_line: 1,
        surface: profile,
    };
    p.items()
}

// Expression-parser frame types (module scope; PLAN-745 R5-QA-01).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OpFrame {
    /// Pending binary operator (left-associative), carrying the operator
    /// itself plus its precedence level.
    Bin {
        op: BinOp,
        prec: u8,
    },
    /// Pending unary operator; applied as soon as a complete primary value
    /// is available (unary binds tighter than every binary operator).
    Unary(UnaryOp),
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

// Statement-block machine types (R6: block nesting is unlimited — the
// open-block stack replaces the recursive block()/stmt() pair).
struct OpenBlock {
    /// Span of the opening `{`.
    lb: Span,
    stmts: Vec<Stmt>,
    cont: BlockCont,
}

enum BlockCont {
    /// The function body block; closing it finishes the item.
    FnBody {
        fn_kw: Span,
        name: Ident,
        params: Vec<Param>,
        ret: TypeAnn,
    },
    /// `if cond { … }` closed; `else` may still follow.
    IfThen {
        kw: Span,
        cond: Expr,
    },
    /// The `else { … }` block of a pending if.
    IfElse {
        kw: Span,
        cond: Expr,
        then: Block,
    },
    WhileBody {
        kw: Span,
        cond: Expr,
    },
    LoopBody {
        kw: Span,
    },
}

struct Parser<'s> {
    src: &'s str,
    toks: Vec<Token>,
    pos: usize,
    /// Line of the most recently consumed token.
    last_line: usize,
    /// Which operator/loop surface this parse accepts.
    surface: SourceProfile,
}

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
        let item = self.body_machine(fn_kw, name, params, ret)?;
        Ok(item)
    }

    /// Parse one function body with an explicit open-block stack (R6:
    /// statement nesting of any depth parses without recursion). Blocks
    /// open with `if`/`while`/`loop` heads; closing a `}` applies the
    /// frame's continuation (finish the item, arm of an if, loop body).
    fn body_machine(
        &mut self,
        fn_kw: Span,
        name: Ident,
        params: Vec<Param>,
        ret: TypeAnn,
    ) -> Result<FnItem, Diagnostic> {
        let lb = self.expect_punct(P::LBrace, "`{`")?;
        let mut stack: Vec<OpenBlock> = vec![OpenBlock {
            lb,
            stmts: Vec::new(),
            cont: BlockCont::FnBody {
                fn_kw,
                name,
                params,
                ret,
            },
        }];
        loop {
            self.skip_semis();
            match self.peek() {
                None => return Err(self.syntax("expected statement or `}`, found end of file")),
                Some(Token {
                    tok: Tok::Punct(P::RBrace),
                    ..
                }) => {
                    let rb = self.bump();
                    let frame = stack.pop().expect("open block");
                    let block = Block {
                        stmts: frame.stmts,
                        span: frame.lb.join(rb.span),
                    };
                    match frame.cont {
                        BlockCont::FnBody {
                            fn_kw,
                            name,
                            params,
                            ret,
                        } => {
                            debug_assert!(stack.is_empty());
                            let last = self.toks[self.pos - 1].span;
                            return Ok(FnItem {
                                span: fn_kw.join(last),
                                name,
                                params,
                                ret,
                                body: block,
                            });
                        }
                        BlockCont::IfThen { kw, cond } => {
                            if matches!(
                                self.peek(),
                                Some(Token {
                                    tok: Tok::Kw(Kw::Else),
                                    ..
                                })
                            ) {
                                self.pos += 1;
                                self.last_line = self.toks[self.pos - 1].line;
                                let lb = self.expect_punct(P::LBrace, "`{`")?;
                                stack.push(OpenBlock {
                                    lb,
                                    stmts: Vec::new(),
                                    cont: BlockCont::IfElse {
                                        kw,
                                        cond,
                                        then: block,
                                    },
                                });
                                continue;
                            }
                            let span = kw.join(block.span);
                            self.push_stmt(
                                &mut stack,
                                Stmt::If {
                                    cond,
                                    then: block,
                                    els: None,
                                    span,
                                },
                            )?;
                        }
                        BlockCont::IfElse { kw, cond, then } => {
                            let span = kw.join(then.span).join(block.span);
                            self.push_stmt(
                                &mut stack,
                                Stmt::If {
                                    cond,
                                    then,
                                    els: Some(block),
                                    span,
                                },
                            )?;
                        }
                        BlockCont::WhileBody { kw, cond } => {
                            let span = kw.join(block.span);
                            self.push_stmt(
                                &mut stack,
                                Stmt::While {
                                    cond,
                                    body: block,
                                    span,
                                },
                            )?;
                        }
                        BlockCont::LoopBody { kw } => {
                            let span = kw.join(block.span);
                            self.push_stmt(&mut stack, Stmt::Loop { body: block, span })?;
                        }
                    }
                }
                Some(_) => self.parse_stmt_head(&mut stack)?,
            }
        }
    }

    /// Parse one statement. Block-opening heads (`if`/`while`/`loop`) push
    /// an open-block frame and return; everything else completes and lands
    /// in the current block via [`Self::push_stmt`].
    fn parse_stmt_head(&mut self, stack: &mut Vec<OpenBlock>) -> Result<(), Diagnostic> {
        match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Kw(Kw::Let)) | Some(Tok::Kw(Kw::Var)) => {
                let s = self.decl_stmt()?;
                self.push_stmt(stack, s)
            }
            Some(Tok::Kw(Kw::Return)) => {
                let kw = self.bump();
                let value = self.expr(false)?;
                let span = kw.span.join(value.span);
                self.push_stmt(stack, Stmt::Return { value, span })
            }
            Some(Tok::Kw(Kw::If)) => {
                let kw = self.bump();
                let cond = self.expr(false)?;
                let lb = self.expect_punct(P::LBrace, "`{`")?;
                stack.push(OpenBlock {
                    lb,
                    stmts: Vec::new(),
                    cont: BlockCont::IfThen { kw: kw.span, cond },
                });
                Ok(())
            }
            Some(Tok::Kw(Kw::While)) if self.surface.is_scalar() => {
                // PLAN-767 §5.2.1: the scalar profile spells conditional
                // loops `for cond { ... }`; `while` is rejected at the
                // keyword token.
                let t = self.peek().unwrap();
                Err(self.unsupported(
                    "`while` is not part of this source profile; conditional loops use `for <condition> { ... }`",
                    t.span,
                ))
            }
            Some(Tok::Kw(Kw::While)) => {
                let kw = self.bump();
                let cond = self.expr(false)?;
                let lb = self.expect_punct(P::LBrace, "`{`")?;
                stack.push(OpenBlock {
                    lb,
                    stmts: Vec::new(),
                    cont: BlockCont::WhileBody { kw: kw.span, cond },
                });
                Ok(())
            }
            Some(Tok::Kw(Kw::For)) if self.surface.is_scalar() => {
                // Conditional for (PLAN-767 §5.2.1): reuses the while's
                // AST shape — the condition is re-evaluated before every
                // entry into the body, `continue` re-tests it, `break`
                // leaves the nearest loop. The HIR lowering is identical.
                let kw = self.bump();
                let cond = self.expr(false)?;
                let lb = self.expect_punct(P::LBrace, "`{`")?;
                stack.push(OpenBlock {
                    lb,
                    stmts: Vec::new(),
                    cont: BlockCont::WhileBody { kw: kw.span, cond },
                });
                Ok(())
            }
            Some(Tok::Kw(Kw::For)) => {
                // core surface: recognized keyword, excluded profile member
                let t = self.peek().unwrap();
                Err(self.unsupported("`for` is not part of this source profile", t.span))
            }
            Some(Tok::Kw(Kw::Loop)) => {
                let kw = self.bump();
                let lb = self.expect_punct(P::LBrace, "`{`")?;
                stack.push(OpenBlock {
                    lb,
                    stmts: Vec::new(),
                    cont: BlockCont::LoopBody { kw: kw.span },
                });
                Ok(())
            }
            Some(Tok::Kw(Kw::Break)) => {
                let t = self.bump();
                self.push_stmt(stack, Stmt::Break { span: t.span })
            }
            Some(Tok::Kw(Kw::Continue)) => {
                let t = self.bump();
                self.push_stmt(stack, Stmt::Continue { span: t.span })
            }
            Some(Tok::Ident(_))
                if matches!(self.peek2().map(|t| &t.tok), Some(Tok::Punct(P::Assign))) =>
            {
                let name = self.expect_ident("local name")?;
                self.expect_punct(P::Assign, "`=`")?;
                let value = self.expr(false)?;
                self.push_stmt(
                    stack,
                    Stmt::Assign {
                        span: name.span.join(value.span),
                        name,
                        value,
                    },
                )
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

    /// Append a completed statement to the innermost open block and enforce
    /// the statement separator rule (`;`, `}` or a newline).
    fn push_stmt(&mut self, stack: &mut Vec<OpenBlock>, s: Stmt) -> Result<(), Diagnostic> {
        stack.last_mut().expect("open block").stmts.push(s);
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
        Ok(())
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
    // form — binary operators, unary chains, parenthesized groups, call
    // argument lists — runs on explicit value/operator stacks, so
    // expression nesting of ANY shape or depth (consecutive or interleaved,
    // complete or unclosed) parses without recursion and without a nesting
    // limit.
    //
    // Frames:
    //   Bin{op,prec}     pending binary operator (left-associative)
    //   Unary(op)        pending unary operator (applied at value completion)
    //   Paren            opened by `(`
    //   Call{callee,..}  opened by `ident (`
    // Line discipline (R1/R2-QA-01): outside ALL frames (the unbracketed
    // top level) an operator must share the line of the previously
    // consumed token, and a primary following an operator must share the
    // operator's line; inside any bracket frame newlines are whitespace.
    // This keeps `1 +\n 2`, `f\n(3)`, `-\n 2` rejected while `f(\n 2\n) + 3`,
    // `(1 +\n 2) + 3` and `((1+\n2)*3)` pass (D01..D05, G02, H01).

    /// Operators that exist in the wider language but are outside BOTH
    /// profiles; in operator position they are `source.unsupported`.
    fn unsupported_op(p: P) -> bool {
        matches!(p, P::Dot)
    }

    /// Precedence of a binary operator on the SCALAR surface (higher binds
    /// tighter). `None` for tokens that are not binary operators here.
    fn scalar_prec_of(p: P) -> Option<(BinOp, u8)> {
        match p {
            P::PipePipe => Some((BinOp::Or, 1)),
            P::AmpAmp => Some((BinOp::And, 2)),
            P::EqEq => Some((BinOp::Eq, 3)),
            P::NotEq => Some((BinOp::Ne, 3)),
            P::Lt => Some((BinOp::Lt, 4)),
            P::Gt => Some((BinOp::Gt, 4)),
            P::LeEq => Some((BinOp::Le, 4)),
            P::GeEq => Some((BinOp::Ge, 4)),
            P::Plus => Some((BinOp::Add, 5)),
            P::Minus => Some((BinOp::Sub, 5)),
            P::Star => Some((BinOp::Mul, 6)),
            P::Slash => Some((BinOp::Div, 6)),
            P::Percent => Some((BinOp::Rem, 6)),
            _ => None,
        }
    }

    /// Precedence on the CORE surface: `+ * <` only.
    fn core_prec_of(p: P) -> Option<(BinOp, u8)> {
        match p {
            P::Plus => Some((BinOp::Add, 2)),
            P::Star => Some((BinOp::Mul, 3)),
            P::Lt => Some((BinOp::Lt, 1)),
            _ => None,
        }
    }

    fn prec_of(&self, p: P) -> Option<(BinOp, u8)> {
        if self.surface.is_scalar() {
            Self::scalar_prec_of(p)
        } else {
            Self::core_prec_of(p)
        }
    }

    /// Is this punct a binary operator on a surface OTHER than ours? Used
    /// to give the `source.unsupported` (not plain syntax) rejection when
    /// the core surface meets scalar-only operators.
    fn op_of_other_surface(&self, p: P) -> bool {
        if self.surface.is_scalar() {
            false
        } else {
            Self::scalar_prec_of(p).is_some() && Self::core_prec_of(p).is_none()
        }
    }

    /// True when the top of `frames` is a bracket context (parens/call) —
    /// i.e. newlines are whitespace and line checks are off. Unary frames
    /// are operators, not brackets.
    fn in_bracket(frames: &[Frame]) -> bool {
        frames
            .iter()
            .any(|f| matches!(f, Frame::Op(OpFrame::Paren)) || matches!(f, Frame::Call(_)))
    }

    /// True when a primary that is about to be parsed must share the line
    /// of the last consumed token (an operator is pending at the
    /// unbracketed top level).
    fn anchored(frames: &[Frame]) -> bool {
        !Self::in_bracket(frames)
            && frames.last().is_some_and(|f| {
                matches!(
                    f,
                    Frame::Op(OpFrame::Bin { .. }) | Frame::Op(OpFrame::Unary(_))
                )
            })
    }

    /// Pop and apply all pending unary frames on top of the stack (they
    /// bind tighter than any binary operator). Called whenever a complete
    /// value is available: entering the operator/close loop.
    fn apply_unaries(frames: &mut Vec<Frame>, vals: &mut Vec<Expr>) {
        while matches!(frames.last(), Some(Frame::Op(OpFrame::Unary(_)))) {
            let Some(Frame::Op(OpFrame::Unary(op))) = frames.pop() else {
                unreachable!("guarded by the while condition")
            };
            let operand = vals.pop().expect("unary operand");
            let span = operand.span;
            vals.push(Expr {
                span,
                kind: ExprKind::Unary {
                    op,
                    operand: Box::new(operand),
                },
            });
        }
    }

    /// Pop and apply pending binary operators with precedence >= `min`.
    fn reduce_ops(frames: &mut Vec<Frame>, vals: &mut Vec<Expr>) {
        while let Some(Frame::Op(OpFrame::Bin { .. })) = frames.last() {
            Self::reduce_once(frames, vals);
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
            .is_some_and(|f| matches!(f, Frame::Op(OpFrame::Bin { .. })))
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
            let anchored = Self::anchored(&frames);
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
                            _ if self.surface.is_scalar() => {
                                // general unary minus: `-x`, `-(expr)`,
                                // `--x` chains (right-associative via the
                                // pending-unary frame discipline)
                                self.pos += 1;
                                self.last_line = line;
                                frames.push(Frame::Op(OpFrame::Unary(UnaryOp::Neg)));
                                continue 'outer;
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
                        if self.surface.is_scalar() {
                            self.pos += 1;
                            self.last_line = line;
                            frames.push(Frame::Op(OpFrame::Unary(UnaryOp::Not)));
                            continue 'outer;
                        }
                        return Err(
                            self.unsupported("unary `!` is not part of this source profile", span)
                        );
                    }
                    Tok::Punct(P::Plus) if self.surface.is_scalar() => {
                        // unary `+` stays excluded on the scalar surface;
                        // the core surface keeps its historical plain
                        // syntax rejection for a leading `+`
                        return Err(
                            self.unsupported("unary `+` is not part of this source profile", span)
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
                // A complete value is on top of `vals`: every pending unary
                // frame applies now, before any binary operator or closing
                // bracket can claim the value (unary binds tightest).
                Self::apply_unaries(&mut frames, &mut vals);
                let Some(tok) = self.peek().cloned() else {
                    // EOF: reduce the current argument's operators first —
                    // with a Bin on top the pre-reduce checks used to skip
                    // the unclosed Call/Paren below it and silently drop the
                    // frame (R6-QA-02). After the reduction, any frame left
                    // is unclosed and is a controlled error.
                    Self::reduce_ops(&mut frames, &mut vals);
                    match frames.last() {
                        Some(Frame::Call(_)) => {
                            if !vals.is_empty() {
                                Self::finish_arg(&mut frames, &mut vals, self.src)?;
                            }
                            return Err(self.syntax("expected `,` or `)`"));
                        }
                        Some(Frame::Op(OpFrame::Paren)) => {
                            return Err(self.syntax("expected `)`"));
                        }
                        Some(_) => unreachable!("reduce_ops stops only at bracket frames"),
                        None => {}
                    }
                    break 'outer;
                };
                match tok.tok {
                    Tok::Punct(p) if self.prec_of(p).is_some() => {
                        let in_br = Self::in_bracket(&frames);
                        if !in_br && tok.line != self.last_line {
                            // operator starts a new line: the expression
                            // ends; the statement layer decides
                            Self::reduce_ops(&mut frames, &mut vals);
                            break 'outer;
                        }
                        let (op, my) = self.prec_of(p).unwrap();
                        while let Some(Frame::Op(OpFrame::Bin { prec: top, .. })) = frames.last() {
                            if *top >= my {
                                Self::reduce_once(&mut frames, &mut vals);
                            } else {
                                break;
                            }
                        }
                        self.pos += 1;
                        self.last_line = tok.line;
                        frames.push(Frame::Op(OpFrame::Bin { op, prec: my }));
                        continue 'outer; // parse RHS primary
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
                    Tok::Punct(other) if self.op_of_other_surface(other) => {
                        return Err(self.unsupported(
                            format!(
                                "operator `{}` is not part of this source profile",
                                other.as_str()
                            ),
                            tok.span,
                        ));
                    }
                    Tok::Punct(P::LParen) => {
                        // `(` in operator position after a complete value
                        return Err(self
                            .syntax(format!("expected operator, found {}", self.describe_cur())));
                    }
                    Tok::Punct(P::RParen) => {
                        // pending binary operators bind tighter than the
                        // closing bracket — apply them first
                        while let Some(Frame::Op(OpFrame::Bin { .. })) = frames.last() {
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
                        // Reduce the CURRENT argument's pending operators
                        // before looking for the call frame: with an open
                        // argument like `1+2` the stack top is Bin, not Call
                        // (R6-QA-01). reduce_ops stops at the nearest
                        // Call/Paren frame, so the reduction is scoped to
                        // the argument and a comma inside plain parens
                        // (`f((1,2))`) still has no Call on top and rejects.
                        Self::reduce_ops(&mut frames, &mut vals);
                        if let Some(Frame::Call(_)) = frames.last() {
                            Self::finish_arg(&mut frames, &mut vals, self.src)?;
                            self.pos += 1;
                            self.last_line = tok.line;
                            self.take_named_label(&mut frames)?;
                            continue 'outer; // next argument primary
                        }
                        return Err(self.syntax("expected operator, found `,`"));
                    }
                    _ => {
                        // not an operator: the expression ends (the statement
                        // layer handles separators). Reduce first; any frame
                        // left after the reduction is an unclosed call or
                        // paren group — a missing `)` must reject here, never
                        // return a value with the frame silently dropped
                        // (R6-QA-02: `f(1+2 }` used to build+publish).
                        Self::reduce_ops(&mut frames, &mut vals);
                        match frames.last() {
                            Some(Frame::Call(_)) => return Err(self.syntax("expected `,` or `)`")),
                            Some(Frame::Op(OpFrame::Paren)) => {
                                return Err(self.syntax("expected `)`"))
                            }
                            Some(_) => unreachable!("reduce_ops stops only at bracket frames"),
                            None => {}
                        }
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
        let Some(Frame::Op(OpFrame::Bin { op, .. })) = frames.pop() else {
            return;
        };
        let rhs = vals.pop().expect("bin rhs");
        let lhs = vals.pop().expect("bin lhs");
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
