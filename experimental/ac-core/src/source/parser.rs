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
    let toks = super::lexer::tokenize_with(src, profile.admits_text_literals())?;
    let mut p = Parser {
        src,
        toks,
        pos: 0,
        last_line: 1,
        surface: profile,
        pending_is: Vec::new(),
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
    /// The popped receiver when this call frame is actually a METHOD call
    /// (`recv.name(args)`, PLAN-770 text `.to/.move/...` and PLAN-771
    /// record methods alike); `None` for free calls — the close emits a
    /// plain Call.
    method_recv: Option<Box<Expr>>,
    callee: Ident,
    args: Vec<Arg>,
    mark: usize,
    pending_name: Option<Ident>,
}

/// PLAN-771: `Ty { field: expr, ... }` — named constructor frame. Field
/// values are full expressions on the shared value stack; separators are
/// commas or (inside the braces) newlines.
struct CtorFrame {
    ty: Ident,
    fields: Vec<NamedCtorField>,
    mark: usize,
    pending_name: Option<Ident>,
}

enum Frame {
    Op(OpFrame),
    Call(Box<CallFrame>),
    Ctor(Box<CtorFrame>),
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
        receiver: Receiver,
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
    /// PLAN-862: the synthetic script body (top-level statement
    /// sequence). It is closed by END OF FILE, never by `}` — a `}` seen
    /// while this frame is innermost is a syntax error, and the frame is
    /// handled by [`Parser::script_item`] directly instead of
    /// [`Parser::close_block_cont`].
    Script,
    /// PLAN-864: one `is` arm body `{ .. }` closed — the arm completes
    /// into the pending statement; the arm sequence (next header or the
    /// statement's closing `}`) continues from here.
    IsArm {
        qual: Option<Ident>,
        pattern: Ident,
        binding: Option<Ident>,
    },
    /// PLAN-864: a `for x in <list> { .. }` body closed.
    ForInBody {
        kw: Span,
        var: Ident,
        iter: Expr,
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
    /// PLAN-864: in-progress `is` statements (scrutinee, keyword span,
    /// arms accumulated so far). Arm blocks close through the block
    /// machine and append here; the frame's own `}` completes the
    /// statement. Stack discipline mirrors the OpenBlock frames.
    pending_is: Vec<PendingIs>,
}

/// One `is` statement between its header parse and its final `}`
/// (PLAN-864).
struct PendingIs {
    kw: Span,
    scrutinee: Ident,
    arms: Vec<MatchArm>,
}

impl<'s> Parser<'s> {
    // -- plumbing -----------------------------------------------------------

    fn peek(&self) -> Option<&Token> {
        self.toks.get(self.pos)
    }

    fn peek2(&self) -> Option<&Token> {
        self.toks.get(self.pos + 1)
    }

    fn peek3(&self) -> Option<&Token> {
        self.toks.get(self.pos + 2)
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
                Tok::StrLit(_) => "string literal".to_string(),
                Tok::CharLit(_) => "char literal".to_string(),
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
            // PLAN-862 script form: in the entry-unit profile a top-level
            // statement sequence after the declarations is the synthetic
            // `main`. The sequence runs to end of file, so no item can
            // follow it (`source.entry-order`).
            if self.surface.is_entry_unit()
                && !matches!(self.peek().map(|t| &t.tok), Some(Tok::Kw(Kw::Fn)))
            {
                let script = self.script_item()?;
                items.push(Item::Fn(script));
                break;
            }
            items.push(self.item()?);
        }
        Ok(Program { items })
    }

    /// PLAN-862: parse the single-file script — a top-level statement
    /// sequence that becomes the synthetic `main` item. Declarations must
    /// precede it; `return` is rejected at this level
    /// (`source.entry-top-return`) because a script's result is its tail
    /// value, never an early exit.
    fn script_item(&mut self) -> Result<FnItem, Diagnostic> {
        let start = self
            .peek()
            .map(|t| t.span)
            .unwrap_or(Span { start: 0, end: 0 });
        let mut stack: Vec<OpenBlock> = vec![OpenBlock {
            lb: start,
            stmts: Vec::new(),
            cont: BlockCont::Script,
        }];
        loop {
            self.skip_semis();
            match self.peek() {
                None => {
                    if stack.len() != 1 {
                        return Err(self.syntax("expected `}` before end of file"));
                    }
                    let frame = stack.pop().expect("script frame");
                    let end = self
                        .toks
                        .last()
                        .map(|t| t.span)
                        .unwrap_or(Span { start: 0, end: 0 });
                    let span = start.join(end);
                    return Ok(FnItem {
                        name: Ident {
                            name: "main".to_string(),
                            span: start,
                        },
                        params: Vec::new(),
                        // A script's result type is decided by resolve from
                        // its tail statement; the parser records Unit as the
                        // placeholder (no tail = Unit).
                        ret: TypeAnn {
                            kind: TypeRef::Builtin(TypeWord::Void),
                            span: start,
                        },
                        body: Block {
                            stmts: frame.stmts,
                            span,
                        },
                        span,
                        receiver: Receiver::Free,
                        owner: None,
                        script: true,
                    });
                }
                Some(Token {
                    tok: Tok::Punct(P::RBrace),
                    ..
                }) => {
                    let rb = self.bump();
                    let frame = stack.pop().expect("open block");
                    if matches!(frame.cont, BlockCont::Script) {
                        return Err(self.syntax("unexpected `}` at top level"));
                    }
                    let block = Block {
                        stmts: frame.stmts,
                        span: frame.lb.join(rb.span),
                    };
                    self.close_block_cont(&mut stack, frame.cont, block)?;
                }
                Some(Token {
                    tok: Tok::Kw(Kw::Fn),
                    ..
                }) if stack.len() == 1 => {
                    let sp = self.peek().unwrap().span;
                    return Err(Diagnostic::new(
                        Stage::Text,
                        "source.entry-order",
                        "declarations must precede the script statement sequence; a script runs to end of file",
                        sp,
                    ));
                }
                Some(_) => self.parse_stmt_head(&mut stack, true)?,
            }
        }
    }

    fn item(&mut self) -> Result<Item, Diagnostic> {
        match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Kw(Kw::Fn)) => Ok(Item::Fn(self.fn_item(Receiver::Free)?)),
            Some(Tok::Kw(Kw::Type)) if self.surface.is_record() => {
                Ok(Item::Type(self.type_item()?))
            }
            Some(Tok::Kw(Kw::Static)) => {
                let span = self.peek().unwrap().span;
                Err(self.unsupported(
                    "`static` is only legal before `fn` inside a `type` block of the record profile",
                    span,
                ))
            }
            Some(Tok::Kw(Kw::Type)) => {
                let span = self.peek().unwrap().span;
                Err(self.unsupported("`type` is not part of this source profile", span))
            }
            // PLAN-864: finite `enum` declarations (collections surface).
            Some(Tok::Kw(Kw::Enum)) if self.surface.is_collections() => {
                Ok(Item::Enum(self.enum_item()?))
            }
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

    /// `type Name { fields; methods }` (record profile only; PLAN-771
    /// §5.2/design.md §2.1 — mirrors the real parser's member grammar:
    /// `name [":"] Type`, separators `;`/newline; `static fn`/`fn`
    /// methods; no defaults/generics/inheritance).
    fn type_item(&mut self) -> Result<TypeItem, Diagnostic> {
        let kw = self.expect_kw(Kw::Type, "`type`")?;
        let name = self.expect_ident("type name")?;
        self.expect_punct(P::LBrace, "`{`")?;
        let mut fields: Vec<FieldDecl> = Vec::new();
        let mut methods: Vec<FnItem> = Vec::new();
        let mut seen_fields: Vec<String> = Vec::new();
        let mut any_member = false;
        loop {
            // member separator: `;`, newline(s) or the closing brace; the
            // FIRST member may share the `{`'s line, and a member may share
            // a `;`'s line
            let after_semi = matches!(
                self.toks.get(self.pos.wrapping_sub(1)),
                Some(Token {
                    tok: Tok::Punct(P::Semi),
                    ..
                })
            );
            match self.peek() {
                None => return Err(self.syntax("expected `}` before end of file")),
                Some(Token {
                    tok: Tok::Punct(P::Semi),
                    ..
                }) => {
                    self.pos += 1;
                    continue;
                }
                Some(Token {
                    tok: Tok::Punct(P::RBrace),
                    ..
                }) => break,
                Some(t) if !any_member || after_semi || t.line > self.toks[self.pos - 1].line => {}
                Some(_) => {
                    return Err(self.syntax(format!(
                        "expected `;`, newline or `}}`, found {}",
                        self.describe_cur()
                    )))
                }
            }
            any_member = true;
            match self.peek().map(|t| t.tok.clone()) {
                Some(Tok::Kw(Kw::Static)) => {
                    let sp = self.peek().unwrap().span;
                    self.pos += 1;
                    self.expect_kw(Kw::Fn, "`fn` after `static`")
                        .map_err(|_| self.syntax("expected `fn` after `static`"))?;
                    // PLAN-881: the item span STARTS at the `static`
                    // keyword — a span begun at `fn` would leave a
                    // dangling `static ` behind any byte-range masking
                    // of the method (the single-method document path).
                    methods.push(self.fn_body_from_fn_kw_span(
                        name.clone(),
                        Receiver::Static,
                        sp,
                    )?);
                }
                Some(Tok::Kw(Kw::Fn)) => {
                    self.pos += 1;
                    methods.push(self.fn_body_from_fn_kw(name.clone(), Receiver::View)?);
                }
                Some(Tok::Kw(Kw::Mut)) => {
                    let sp = self.peek().unwrap().span;
                    return Err(self.unsupported(
                        "`mut fn` (mutable instance methods) is not part of this profile; methods are readonly views",
                        sp,
                    ));
                }
                Some(Tok::Ident(fname)) => {
                    let fname_ident = self.expect_ident("field name")?;
                    if fname == "self" {
                        return Err(
                            self.unsupported("`self` cannot be a field name", fname_ident.span)
                        );
                    }
                    // optional `:`
                    if matches!(
                        self.peek(),
                        Some(Token {
                            tok: Tok::Punct(P::Colon),
                            ..
                        })
                    ) {
                        self.pos += 1;
                    }
                    let ty = self.member_type(&fname_ident)?;
                    if seen_fields.contains(&fname) {
                        return Err(self.err(
                            "source.duplicate",
                            format!(
                                "field `{}` is declared more than once in type `{}`",
                                fname, name.name
                            ),
                            fname_ident.span,
                        ));
                    }
                    seen_fields.push(fname);
                    fields.push(FieldDecl {
                        span: fname_ident.span.join(ty.span),
                        name: fname_ident,
                        ty,
                    });
                }
                Some(Tok::Kw(k)) => {
                    let sp = self.peek().unwrap().span;
                    return Err(self.unsupported(
                        format!("`{}` is not a valid type member here", k.as_str()),
                        sp,
                    ));
                }
                _ => {
                    return Err(self.syntax(format!(
                        "expected field or method, found {}",
                        self.describe_cur()
                    )))
                }
            }
        }
        let rb = self.expect_punct(P::RBrace, "`}`")?;
        if fields.is_empty() {
            return Err(self.unsupported(
                "empty records are not part of this profile; a record must declare at least one field",
                kw.join(rb),
            ));
        }
        Ok(TypeItem {
            span: kw.join(rb),
            name,
            fields,
            methods,
        })
    }

    /// Method body after the `fn` keyword was consumed (receiver/owner
    /// set); shared by `static fn` and `fn` members.
    fn fn_body_from_fn_kw(
        &mut self,
        owner: Ident,
        receiver: Receiver,
    ) -> Result<FnItem, Diagnostic> {
        let fn_span = self.toks[self.pos - 1].span; // the `fn` keyword
        let mut item = self.fn_item_tail(fn_span, receiver)?;
        item.owner = Some(owner);
        Ok(item)
    }

    /// [`fn_body_from_fn_kw`] with an explicit leading span (the `static`
    /// keyword) joined into the item span — static methods span from
    /// `static`, view methods from `fn` (PLAN-881).
    fn fn_body_from_fn_kw_span(
        &mut self,
        owner: Ident,
        receiver: Receiver,
        lead: crate::atom_text::Span,
    ) -> Result<FnItem, Diagnostic> {
        let fn_span = self.toks[self.pos - 1].span; // the `fn` keyword
        let mut item = self.fn_item_tail(fn_span.join(lead), receiver)?;
        item.owner = Some(owner);
        Ok(item)
    }

    fn fn_item(&mut self, receiver: Receiver) -> Result<FnItem, Diagnostic> {
        let fn_kw = self.expect_kw(Kw::Fn, "`fn`")?;
        self.fn_item_tail(fn_kw, receiver)
    }

    /// Everything after the `fn` keyword: name, params (with the record
    /// profile's `mut name Type` mode prefix), return type (the record
    /// profile also accepts a record type name), body.
    fn fn_item_tail(&mut self, fn_kw: Span, receiver: Receiver) -> Result<FnItem, Diagnostic> {
        let name = self.expect_ident("function name")?;
        // PLAN-771: an instance method's receiver is implicit; an explicit
        // `self` param would shadow it.
        if receiver == Receiver::View && name.name == "self" {
            return Err(self.unsupported(
                "`self` is the implicit receiver of an instance method and cannot be a method name",
                name.span,
            ));
        }
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
                // mode word before the name: text surfaces use the
                // reserved words `mut`/`move` (PLAN-770 convention `fn
                // f(mut s str)`); the record surface uses the `mut`
                // keyword (PLAN-771 `mut name Type`). Other surfaces keep
                // rejecting the words exactly as before.
                let mut mode = super::ast::ParamMode::Default;
                let mode_span = match self.peek() {
                    Some(Token {
                        tok: Tok::Reserved(w),
                        span,
                        ..
                    }) if self.surface.is_text() && (w == "mut" || w == "move") => {
                        let sp = *span;
                        mode = if w == "mut" {
                            super::ast::ParamMode::Mut
                        } else {
                            super::ast::ParamMode::Move
                        };
                        self.pos += 1;
                        Some(sp)
                    }
                    // the shared lexer keyword-izes `mut` (PLAN-771's
                    // record spelling); the TEXT surface accepts the same
                    // keyword token for its `mut` mode word
                    Some(Token {
                        tok: Tok::Kw(Kw::Mut),
                        span,
                        ..
                    }) if self.surface.is_text() || self.surface.is_record() => {
                        let sp = *span;
                        mode = super::ast::ParamMode::Mut;
                        self.pos += 1;
                        Some(sp)
                    }
                    _ => None,
                };
                let pname = self.expect_ident("parameter name")?;
                if receiver == Receiver::View && pname.name == "self" {
                    return Err(self.unsupported(
                        "`self` is the implicit receiver of an instance method and cannot be declared as a parameter",
                        pname.span,
                    ));
                }
                let ty = self.param_type(&pname)?;
                let span = match mode_span {
                    Some(ms) => ms.join(ty.span),
                    None => pname.span.join(ty.span),
                };
                params.push(Param {
                    span,
                    name: pname,
                    ty,
                    mode,
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
            Some(Tok::Kw(Kw::Fn)) if self.surface.is_closures() => self.closures_type()?,
            // PLAN-864: the collections type grammar (?T, List<T>,
            // Result<T,E>, records, int/i32/bool/Str) is the whole surface.
            Some(Tok::Punct(P::Question)) if self.surface.is_collections() => {
                self.collections_type()?
            }
            Some(Tok::Ident(_)) if self.surface.is_collections() => self.collections_type()?,
            Some(Tok::Kw(Kw::Int)) => self.type_word(),
            Some(Tok::Kw(Kw::I32)) => self.type_word(),
            Some(Tok::Kw(Kw::Bool)) => self.type_word(),
            Some(Tok::Ident(ref w))
                if self.surface.is_text() && (w == "str" || w == "Str" || w == "char") =>
            {
                self.type_word()
            }
            // PLAN-862: `void` is a RESULT type word; the entry-unit
            // profile accepts it here and nowhere else (params/fields keep
            // rejecting it).
            Some(Tok::Ident(ref w)) if self.surface.is_entry_unit() && w == "void" => {
                let t = self.bump();
                TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::Void),
                    span: t.span,
                }
            }
            Some(Tok::Ident(_)) if self.surface.is_record() => {
                let t = self.bump();
                let id = match &t.tok {
                    Tok::Ident(n) => Ident {
                        name: n.clone(),
                        span: t.span,
                    },
                    _ => unreachable!(),
                };
                TypeAnn {
                    kind: TypeRef::Record(id),
                    span: t.span,
                }
            }
            Some(Tok::Punct(P::LBrace)) => {
                // N42: missing return annotation — point at the signature.
                return Err(self.unsupported(
                    "function return type annotation (`int`/`i32`/`bool` or a record type) is required",
                    name.span.join(rparen),
                ));
            }
            Some(Tok::Ident(t) | Tok::Reserved(t)) => {
                let sp = self.peek().unwrap().span;
                return Err(self.unsupported(
                    format!("type `{}` is not part of this source profile", t),
                    sp,
                ));
            }
            _ => {
                return Err(self.syntax(format!(
                    "expected return type, found {}",
                    self.describe_cur()
                )))
            }
        };
        self.body_machine(fn_kw, name, params, ret, receiver)
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
        receiver: Receiver,
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
                receiver,
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
                    if let Some(item) = self.close_block_cont(&mut stack, frame.cont, block)? {
                        debug_assert!(stack.is_empty());
                        return Ok(item);
                    }
                }
                Some(_) => self.parse_stmt_head(&mut stack, false)?,
            }
        }
    }

    /// Apply one closed block's continuation: either finish the enclosing
    /// function item ([`BlockCont::FnBody`], returned to the caller) or
    /// push the completed statement into the parent frame.
    ///
    /// Shared by the declared-function machine and the PLAN-862 script
    /// machine so the if/else/while/loop closing rules live in ONE place.
    fn close_block_cont(
        &mut self,
        stack: &mut Vec<OpenBlock>,
        cont: BlockCont,
        block: Block,
    ) -> Result<Option<FnItem>, Diagnostic> {
        match cont {
            BlockCont::FnBody {
                fn_kw,
                name,
                params,
                ret,
                receiver,
            } => {
                let last = self.toks[self.pos - 1].span;
                Ok(Some(FnItem {
                    span: fn_kw.join(last),
                    name,
                    params,
                    ret,
                    body: block,
                    receiver,
                    owner: None,
                    script: false,
                }))
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
                    return Ok(None);
                }
                let span = kw.join(block.span);
                self.push_stmt(
                    stack,
                    Stmt::If {
                        cond,
                        then: block,
                        els: None,
                        span,
                    },
                )?;
                Ok(None)
            }
            BlockCont::IfElse { kw, cond, then } => {
                let span = kw.join(then.span).join(block.span);
                self.push_stmt(
                    stack,
                    Stmt::If {
                        cond,
                        then,
                        els: Some(block),
                        span,
                    },
                )?;
                Ok(None)
            }
            BlockCont::WhileBody { kw, cond } => {
                let span = kw.join(block.span);
                self.push_stmt(
                    stack,
                    Stmt::While {
                        cond,
                        body: block,
                        span,
                    },
                )?;
                Ok(None)
            }
            BlockCont::LoopBody { kw } => {
                let span = kw.join(block.span);
                self.push_stmt(stack, Stmt::Loop { body: block, span })?;
                Ok(None)
            }
            // PLAN-864: a `for x in .. { .. }` body completes the statement.
            BlockCont::ForInBody { kw, var, iter } => {
                let span = kw.join(block.span);
                self.push_stmt(
                    stack,
                    Stmt::ForIn {
                        var,
                        iter,
                        body: block,
                        span,
                    },
                )?;
                Ok(None)
            }
            // PLAN-864: one arm body closed. The arm completes into the
            // pending `is`; then either the next arm header opens another
            // block or the statement's closing `}` finishes everything.
            BlockCont::IsArm {
                qual,
                pattern,
                binding,
            } => {
                let span = pattern.span.join(block.span);
                {
                    let pend = self
                        .pending_is
                        .last_mut()
                        .expect("is-arm close with no pending `is`");
                    pend.arms.push(MatchArm {
                        qual,
                        pattern,
                        binding,
                        body: block,
                        span,
                    });
                }
                self.continue_is_arms(stack)?;
                Ok(None)
            }
            // The script frame is closed by end of file only.
            BlockCont::Script => unreachable!("the script frame is never closed by a brace"),
        }
    }

    /// Parse one statement. Block-opening heads (`if`/`while`/`loop`) push
    /// an open-block frame and return; everything else completes and lands
    /// in the current block via [`Self::push_stmt`].
    ///
    /// `top` is true only for statements directly in the PLAN-862 script
    /// root: a script's result is its tail value, so `return` is rejected
    /// there (`source.entry-top-return`) instead of being an early exit.
    fn parse_stmt_head(&mut self, stack: &mut Vec<OpenBlock>, top: bool) -> Result<(), Diagnostic> {
        match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Kw(Kw::Let)) | Some(Tok::Kw(Kw::Var)) => {
                let s = self.decl_stmt()?;
                self.push_stmt(stack, s)
            }
            Some(Tok::Kw(Kw::Return)) => {
                let kw = self.bump();
                if top {
                    return Err(Diagnostic::new(
                        Stage::Text,
                        "source.entry-top-return",
                        "a script's result is its trailing expression; `return` is not part of the script form (use a declared `fn main` for an explicit exit)",
                        kw.span,
                    ));
                }
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
            Some(Tok::Kw(Kw::While)) if self.surface.surface_is_scalar_or_wider() => {
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
            // PLAN-864: `for x in <list> { .. }` — bounded List iteration.
            // The connector keyword (`in`) and the body-block continuation
            // belong to the collections surface only; the scalar surface's
            // conditional-for arm below is unreachable here.
            Some(Tok::Kw(Kw::For)) if self.surface.is_collections() => {
                let kw = self.bump();
                let var = self.expect_ident("for binding name")?;
                match self.peek().map(|t| t.tok.clone()) {
                    Some(Tok::Kw(Kw::In)) => {
                        self.pos += 1;
                        self.last_line = self.toks[self.pos - 1].line;
                    }
                    _ => {
                        return Err(self.syntax(format!(
                            "expected `in` after the `for` binding, found {}",
                            self.describe_cur()
                        )))
                    }
                }
                // PLAN-864 contract C7: the iterator is a bare List local —
                // parsed as an IDENT (never a general expression), which
                // also keeps the body's `{` from parsing as a named
                // constructor (`xs { .. }`).
                let iter_name = self.expect_ident("for-in iterator (a List local)")?;
                let iter = Expr {
                    span: iter_name.span,
                    kind: ExprKind::Var(iter_name),
                };
                let lb = self.expect_punct(P::LBrace, "`{`")?;
                stack.push(OpenBlock {
                    lb,
                    stmts: Vec::new(),
                    cont: BlockCont::ForInBody {
                        kw: kw.span,
                        var,
                        iter,
                    },
                });
                Ok(())
            }
            Some(Tok::Kw(Kw::For)) if self.surface.surface_is_scalar_or_wider() => {
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
            // PLAN-864: `is <local> { Arm(v)? -> { .. } ... }` — exhaustive
            // variant dispatch. Arm bodies ride the block machine; the arm
            // sequence and the statement's `}` are driven by
            // `continue_is_arms`.
            Some(Tok::Kw(Kw::Is)) if self.surface.is_collections() => {
                let kw = self.bump();
                let scrutinee = self.expect_ident("`is` scrutinee (a local)")?;
                self.expect_punct(P::LBrace, "`{`")?;
                self.pending_is.push(PendingIs {
                    kw: kw.span,
                    scrutinee: Ident {
                        name: scrutinee.name.clone(),
                        span: scrutinee.span,
                    },
                    arms: Vec::new(),
                });
                self.continue_is_arms(stack)
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
                if matches!(
                    self.peek2().map(|t| &t.tok),
                    Some(Tok::Punct(P::Assign)) | Some(Tok::Punct(P::Dot))
                ) =>
            {
                // Speculative: `a.b = v` is an assignment, `a.b(c)` / `a.b`
                // continue as an expression (PLAN-864 method-call
                // statements). Rewind when no `=` terminates the path.
                let mark = self.pos;
                let base = self.expect_ident("local name")?;
                let mut path: Vec<Ident> = Vec::new();
                let mut is_assign = true;
                while matches!(
                    self.peek(),
                    Some(Token {
                        tok: Tok::Punct(P::Dot),
                        ..
                    })
                ) {
                    if !self.surface.is_record() {
                        return Err(self.unsupported(
                            "field access is not part of this source profile",
                            self.peek().unwrap().span,
                        ));
                    }
                    self.pos += 1;
                    path.push(self.expect_ident("field name after `.`")?);
                }
                if !matches!(
                    self.peek(),
                    Some(Token {
                        tok: Tok::Punct(P::Assign),
                        ..
                    })
                ) {
                    is_assign = false;
                }
                if !is_assign {
                    self.pos = mark;
                    let e = self.expr(false)?;
                    if self.surface.is_entry_unit() {
                        let span = e.span;
                        return self.push_stmt(stack, Stmt::ExprStmt { value: e, span });
                    }
                    if self.surface.is_collections() {
                        let span = e.span;
                        return self.push_stmt(stack, Stmt::ExprStmt { value: e, span });
                    }
                    return Err(self.unsupported(
                        "expression statements are not part of this source profile; calls are only allowed in initializers, assignments, returns and conditions",
                        e.span,
                    ));
                }
                let target = AssignTarget { base, path };
                self.expect_punct(P::Assign, "`=`")?;
                let value = self.expr(false)?;
                self.push_stmt(
                    stack,
                    Stmt::Assign {
                        span: target.base.span.join(value.span),
                        target,
                        value,
                    },
                )
            }
            Some(Tok::Ident(_))
            | Some(Tok::Kw(Kw::True))
            | Some(Tok::Kw(Kw::False))
            | Some(Tok::Int(_)) => {
                let e = self.expr(false)?;
                if self.surface.is_entry_unit() {
                    // PLAN-862: an expression statement is parsed so the
                    // script tail can carry the result; RESOLVE owns the
                    // position rule (only the script's trailing statement
                    // is legal, everything else is `source.entry-tail`).
                    let span = e.span;
                    return self.push_stmt(stack, Stmt::ExprStmt { value: e, span });
                }
                if self.surface.is_collections() {
                    // PLAN-864: an expression statement is parsed so the
                    // unit-effect method calls (`xs.push(v)` & co.) have a
                    // statement form; RESOLVE owns the type rule (only
                    // unit-typed expressions discard —
                    // `source.syntax` names anything else).
                    let span = e.span;
                    return self.push_stmt(stack, Stmt::ExprStmt { value: e, span });
                }
                // N41: an expression statement (e.g. a bare call) — calls are
                // only allowed where a value is consumed.
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
            Tok::Ident(w) if self.surface.is_text() => match w.as_str() {
                "str" => TypeWord::Str,
                "Str" => TypeWord::StrOwned,
                "char" => TypeWord::Char,
                _ => unreachable!("type_word called on non-type token"),
            },
            _ => unreachable!("type_word called on non-type token"),
        };
        TypeAnn {
            kind: TypeRef::Builtin(word),
            span: t.span,
        }
    }

    /// PLAN-864: whether a generic type argument list starting at token
    /// `lt_pos` (the `<`) balances and is followed by `.` — the commit
    /// condition for a receiver-position `Name<T>` expression. Purely a
    /// lookahead over the token vector; consumes nothing.
    fn generic_scan_commits(&self, lt_pos: usize) -> bool {
        let mut depth: usize = 0;
        let mut i = lt_pos;
        while let Some(t) = self.toks.get(i) {
            match &t.tok {
                Tok::Punct(P::Lt) => {
                    depth += 1;
                    i += 1;
                }
                Tok::Punct(P::Gt) => {
                    depth = depth.saturating_sub(1);
                    i += 1;
                    if depth == 0 {
                        return matches!(
                            self.toks.get(i).map(|t2| &t2.tok),
                            Some(Tok::Punct(P::Dot))
                        );
                    }
                }
                Tok::Ident(_)
                | Tok::Kw(Kw::Int)
                | Tok::Kw(Kw::I32)
                | Tok::Kw(Kw::Bool)
                | Tok::Punct(P::Question)
                | Tok::Punct(P::Comma) => {
                    i += 1;
                }
                _ => return false,
            }
        }
        false
    }

    fn is_type_word(tok: &Tok) -> bool {
        match tok {
            Tok::Kw(Kw::Int) | Tok::Kw(Kw::I32) | Tok::Kw(Kw::Bool) => true,
            Tok::Ident(w) => w == "str" || w == "Str" || w == "char",
            _ => false,
        }
    }

    /// Record-member field type (PLAN-771): a builtin word or a record
    /// type name (resolved in later stages; forward references legal).
    fn member_type(&mut self, fname: &Ident) -> Result<TypeAnn, Diagnostic> {
        if self.surface.is_closures() {
            return self.closures_type();
        }
        // PLAN-864: the collections field-type grammar (?T, List<T>,
        // Result<T,E>, records, int/i32/bool/Str).
        if self.surface.is_collections() {
            return self.collections_type();
        }
        match self.peek().map(|t| t.tok.clone()) {
            // Text type words in FIELD position of a non-text profile are a
            // controlled `source.unsupported` rejection — the same guard
            // `param_type`/`decl_ann` already carry. Without it the shared
            // `is_type_word` match below reaches `type_word`'s
            // surface-gated ident arm and panics (PLAN-782 T-01 probe F-A:
            // `type Tok { text Str }` under the record profile crashed with
            // exit 101 instead of rejecting).
            Some(Tok::Ident(w))
                if !self.surface.is_text() && (w == "str" || w == "Str" || w == "char") =>
            {
                let t = self.peek().unwrap();
                Err(self.unsupported(
                    format!(
                        "type `{}` is not part of this source profile",
                        t.span.slice(self.src)
                    ),
                    t.span,
                ))
            }
            Some(t) if Self::is_type_word(&t) => Ok(self.type_word()),
            Some(Tok::Ident(_)) if self.surface.is_record() => {
                let t = self.bump();
                let id = match &t.tok {
                    Tok::Ident(n) => Ident {
                        name: n.clone(),
                        span: t.span,
                    },
                    _ => unreachable!(),
                };
                Ok(TypeAnn {
                    kind: TypeRef::Record(id),
                    span: t.span,
                })
            }
            _ => Err(self.unsupported(
                "field type annotation (`int`/`i32`/`bool` or a record type) is required",
                fname.span,
            )),
        }
    }

    /// Type annotation in parameter position: required, only the three
    /// profile words. Any other identifier is an excluded type surface.
    fn param_type(&mut self, pname: &Ident) -> Result<TypeAnn, Diagnostic> {
        if self.surface.is_closures() {
            return self.closures_type();
        }
        // PLAN-864: the collections parameter-type grammar. `List`-typed
        // parameters PARSE so resolve can name the rejection precisely
        // (`source.list-value-context`); the surface itself accepts the
        // same grammar as fields.
        if self.surface.is_collections() {
            return self.collections_type();
        }
        match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Ident(w))
                if !self.surface.is_text() && (w == "str" || w == "Str" || w == "char") =>
            {
                let t = self.peek().unwrap();
                Err(self.unsupported(
                    format!(
                        "type `{}` is not part of this source profile",
                        t.span.slice(self.src)
                    ),
                    t.span,
                ))
            }
            Some(t) if Self::is_type_word(&t) => Ok(self.type_word()),
            // PLAN-862 T-07 F-862-R1-02: `void` parses as a type word in
            // parameter position of the entry-unit surface so the FROZEN
            // placement rejection (`source.unit-placement`: Unit is a result
            // type only) is what the caller observes, not a generic
            // `source.unsupported`. No program becomes legal — resolve
            // rejects it by name.
            Some(Tok::Ident(ref w)) if self.surface.is_entry_unit() && w == "void" => {
                let t = self.bump();
                Ok(TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::Void),
                    span: t.span,
                })
            }
            Some(Tok::Ident(_)) if self.surface.is_record() => {
                let t = self.bump();
                let id = match &t.tok {
                    Tok::Ident(n) => Ident {
                        name: n.clone(),
                        span: t.span,
                    },
                    _ => unreachable!(),
                };
                Ok(TypeAnn {
                    kind: TypeRef::Record(id),
                    span: t.span,
                })
            }
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
        // PLAN-864: the collections annotation grammar (?T, List<T>,
        // Result<T,E>, records, int/i32/bool/Str); absent annotation falls
        // through to inference.
        if self.surface.is_collections() {
            let starts_type = matches!(
                self.peek().map(|t| &t.tok),
                Some(Tok::Punct(P::Question))
                    | Some(Tok::Kw(Kw::Fn))
                    | Some(Tok::Kw(Kw::Int))
                    | Some(Tok::Kw(Kw::I32))
                    | Some(Tok::Kw(Kw::Bool))
                    | Some(Tok::Ident(_))
            );
            return if starts_type {
                let ty = if self.surface.is_closures() {
                    self.closures_type()?
                } else {
                    self.collections_type()?
                };
                Ok(Some(ty))
            } else {
                Ok(None)
            };
        }
        match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Ident(w))
                if !self.surface.is_text() && (w == "str" || w == "Str" || w == "char") =>
            {
                let t = self.peek().unwrap();
                Err(self.unsupported(
                    format!(
                        "type `{}` is not part of this source profile",
                        t.span.slice(self.src)
                    ),
                    t.span,
                ))
            }
            Some(t) if Self::is_type_word(&t) => Ok(Some(self.type_word())),
            // PLAN-862 T-07 F-862-R1-02: `void` parses as a type word in
            // `let`/`var` position of the entry-unit surface so
            // `source.unit-placement` ("Unit cannot be stored") is reachable
            // and named, instead of a generic `source.unsupported` from the
            // annotation guard below.
            Some(Tok::Ident(ref w)) if self.surface.is_entry_unit() && w == "void" => {
                let t = self.bump();
                Ok(Some(TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::Void),
                    span: t.span,
                }))
            }
            Some(Tok::Ident(_)) if self.surface.is_record() => {
                let t = self.bump();
                let id = match &t.tok {
                    Tok::Ident(n) => Ident {
                        name: n.clone(),
                        span: t.span,
                    },
                    _ => unreachable!(),
                };
                Ok(Some(TypeAnn {
                    kind: TypeRef::Record(id),
                    span: t.span,
                }))
            }
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

    /// Continue an `is` statement's arm sequence (PLAN-864): either another
    /// `Arm(v)? -> { .. }` header opens an arm block, or the statement's
    /// closing `}` completes the whole `is`.
    fn continue_is_arms(&mut self, stack: &mut Vec<OpenBlock>) -> Result<(), Diagnostic> {
        match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Ident(_)) => {
                let (qual, pattern, binding) = self.is_arm_pattern()?;
                self.expect_punct(P::Arrow, "`->`")?;
                let lb = self.expect_punct(P::LBrace, "`{`")?;
                stack.push(OpenBlock {
                    lb,
                    stmts: Vec::new(),
                    cont: BlockCont::IsArm {
                        qual,
                        pattern,
                        binding,
                    },
                });
                Ok(())
            }
            Some(Tok::Punct(P::RBrace)) => {
                self.pos += 1;
                self.last_line = self.toks[self.pos - 1].line;
                let pend = self
                    .pending_is
                    .pop()
                    .expect("is completion with no pending `is`");
                let close = self.toks[self.pos - 1].span;
                let span = pend.kw.join(close);
                self.push_stmt(
                    stack,
                    Stmt::IsMatch {
                        scrutinee: pend.scrutinee,
                        arms: pend.arms,
                        span,
                    },
                )
            }
            _ => Err(self.syntax(format!(
                "expected another `is` arm or `}}`, found {}",
                self.describe_cur()
            ))),
        }
    }

    /// One `is` arm pattern: `Name`, `Enum.Name` or `Name(v)` (PLAN-864).
    fn is_arm_pattern(&mut self) -> Result<(Option<Ident>, Ident, Option<Ident>), Diagnostic> {
        let mut qual: Option<Ident> = None;
        let mut pattern = self.expect_ident("arm pattern name")?;
        if matches!(
            self.peek(),
            Some(Token {
                tok: Tok::Punct(P::Dot),
                ..
            })
        ) {
            self.pos += 1;
            self.last_line = self.toks[self.pos - 1].line;
            qual = Some(pattern);
            pattern = self.expect_ident("arm name after `.`")?;
        }
        let binding = match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Punct(P::LParen)) => {
                self.pos += 1;
                self.last_line = self.toks[self.pos - 1].line;
                let b = self.expect_ident("arm payload binding")?;
                self.expect_punct(P::RParen, "`)`")?;
                Some(b)
            }
            _ => None,
        };
        Ok((qual, pattern, binding))
    }

    /// `enum Name { Arm(T)? ... }` (PLAN-864, collections surface): arms
    /// are whitespace separated; `,`/`;` separators are tolerated. The
    /// declaration order is the runtime discriminant order.
    fn enum_item(&mut self) -> Result<EnumItem, Diagnostic> {
        let kw = self.expect_kw(Kw::Enum, "`enum`")?;
        let name = self.expect_ident("enum name")?;
        self.expect_punct(P::LBrace, "`{`")?;
        let mut arms: Vec<EnumArm> = Vec::new();
        loop {
            match self.peek().map(|t| t.tok.clone()) {
                Some(Tok::Punct(P::RBrace)) => {
                    self.pos += 1;
                    self.last_line = self.toks[self.pos - 1].line;
                    break;
                }
                Some(Tok::Ident(_)) => {
                    let an = self.expect_ident("enum arm name")?;
                    let payload = if matches!(
                        self.peek(),
                        Some(Token {
                            tok: Tok::Punct(P::LParen),
                            ..
                        })
                    ) {
                        self.pos += 1;
                        self.last_line = self.toks[self.pos - 1].line;
                        let t = self.collections_type()?;
                        self.expect_punct(P::RParen, "`)`")?;
                        Some(t)
                    } else {
                        None
                    };
                    let arm_span = match &payload {
                        Some(pl) => an.span.join(pl.span),
                        None => an.span,
                    };
                    arms.push(EnumArm {
                        name: an,
                        payload,
                        span: arm_span,
                    });
                    while matches!(
                        self.peek(),
                        Some(Token {
                            tok: Tok::Punct(P::Comma) | Tok::Punct(P::Semi),
                            ..
                        })
                    ) {
                        self.pos += 1;
                        self.last_line = self.toks[self.pos - 1].line;
                    }
                }
                _ => {
                    return Err(self.syntax(format!(
                        "expected an enum arm or `}}`, found {}",
                        self.describe_cur()
                    )))
                }
            }
        }
        let close = self.toks[self.pos - 1].span;
        Ok(EnumItem {
            name,
            arms,
            span: kw.join(close),
        })
    }

    // -- collections types (PLAN-864) ---------------------------------------

    /// One collections-surface type: `?T` sugar, a builtin word (int/i32/
    /// bool/Str), a nominal record name, or a parameterized nominal
    /// `Name<T, ...>`. `str`/`char` stay named rejections (the collections
    /// surface owns `Str` values only).
    fn collections_type(&mut self) -> Result<TypeAnn, Diagnostic> {
        if matches!(
            self.peek(),
            Some(Token {
                tok: Tok::Punct(P::Question),
                ..
            })
        ) {
            let q = self.bump();
            let inner = self.collections_type()?;
            let span = q.span.join(inner.span);
            return Ok(TypeAnn {
                kind: TypeRef::Optional(Box::new(inner)),
                span,
            });
        }
        match self.peek().map(|t| t.tok.clone()) {
            Some(Tok::Kw(Kw::Int)) | Some(Tok::Kw(Kw::I32)) | Some(Tok::Kw(Kw::Bool)) => {
                Ok(self.type_word())
            }
            Some(Tok::Ident(w)) if w == "Str" => {
                let t = self.bump();
                Ok(TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::StrOwned),
                    span: t.span,
                })
            }
            Some(Tok::Ident(w)) if w == "str" || w == "char" => {
                let t = self.peek().unwrap();
                Err(self.unsupported(
                    format!("type `{}` is not part of this source profile", w),
                    t.span,
                ))
            }
            Some(Tok::Ident(_)) => {
                let t = self.bump();
                let id = match &t.tok {
                    Tok::Ident(n) => Ident {
                        name: n.clone(),
                        span: t.span,
                    },
                    _ => unreachable!(),
                };
                if matches!(
                    self.peek(),
                    Some(Token {
                        tok: Tok::Punct(P::Lt),
                        ..
                    })
                ) {
                    let args = self.type_arg_list()?;
                    let span = t.span.join(self.toks[self.pos - 1].span);
                    Ok(TypeAnn {
                        kind: TypeRef::Generic { name: id, args },
                        span,
                    })
                } else {
                    Ok(TypeAnn {
                        kind: TypeRef::Record(id),
                        span: t.span,
                    })
                }
            }
            _ => Err(self.syntax(format!("expected a type, found {}", self.describe_cur()))),
        }
    }

    /// PLAN-865 function type grammar: `fn(T0, T1) -> R`. Nested function
    /// signatures are permitted; the resolver enforces supported placements.
    fn closures_type(&mut self) -> Result<TypeAnn, Diagnostic> {
        if !matches!(self.peek().map(|t| &t.tok), Some(Tok::Kw(Kw::Fn))) {
            if matches!(
                self.peek(),
                Some(Token {
                    tok: Tok::Ident(name),
                    ..
                }) if name == "void"
            ) {
                let t = self.bump();
                return Ok(TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::Void),
                    span: t.span,
                });
            }
            return self.collections_type();
        }
        let kw = self.expect_kw(Kw::Fn, "`fn`")?;
        self.expect_punct(P::LParen, "`(`")?;
        let mut params = Vec::new();
        if !matches!(self.peek().map(|t| &t.tok), Some(Tok::Punct(P::RParen))) {
            loop {
                params.push(self.closures_type()?);
                match self.peek().map(|t| &t.tok) {
                    Some(Tok::Punct(P::Comma)) => self.pos += 1,
                    Some(Tok::Punct(P::RParen)) => break,
                    _ => return Err(self.syntax("expected `,` or `)` in function type")),
                }
            }
        }
        self.expect_punct(P::RParen, "`)`")?;
        self.expect_punct(P::Arrow, "`->`")?;
        let result = self.closures_type()?;
        Ok(TypeAnn {
            span: kw.join(result.span),
            kind: TypeRef::Function {
                params,
                result: Box::new(result),
            },
        })
    }

    /// `<T0, T1, ...>` — the type-argument list of a parameterized nominal
    /// type (PLAN-864). The closing `>` is consumed.
    fn type_arg_list(&mut self) -> Result<Vec<TypeAnn>, Diagnostic> {
        self.expect_punct(P::Lt, "`<`")?;
        let mut args = Vec::new();
        loop {
            args.push(self.collections_type()?);
            match self.peek().map(|t| t.tok.clone()) {
                Some(Tok::Punct(P::Comma)) => {
                    self.pos += 1;
                    self.last_line = self.toks[self.pos - 1].line;
                }
                Some(Tok::Punct(P::Gt)) => {
                    self.pos += 1;
                    self.last_line = self.toks[self.pos - 1].line;
                    return Ok(args);
                }
                _ => {
                    return Err(self.syntax(format!(
                        "expected `,` or `>` in type arguments, found {}",
                        self.describe_cur()
                    )))
                }
            }
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
        if self.surface.surface_is_scalar_or_wider() {
            Self::scalar_prec_of(p)
        } else {
            Self::core_prec_of(p)
        }
    }

    /// Is this punct a binary operator on a surface OTHER than ours? Used
    /// to give the `source.unsupported` (not plain syntax) rejection when
    /// the core surface meets scalar-only operators.
    fn op_of_other_surface(&self, p: P) -> bool {
        if self.surface.surface_is_scalar_or_wider() {
            false
        } else {
            Self::scalar_prec_of(p).is_some() && Self::core_prec_of(p).is_none()
        }
    }

    /// True when the top of `frames` is a bracket context (parens/call) —
    /// i.e. newlines are whitespace and line checks are off. Unary frames
    /// are operators, not brackets.
    fn in_bracket(frames: &[Frame]) -> bool {
        frames.iter().any(|f| {
            matches!(f, Frame::Op(OpFrame::Paren))
                || matches!(f, Frame::Call(_))
                || matches!(f, Frame::Ctor(_))
        })
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

    /// Reduce and append the current constructor-field value into the
    /// Ctor frame (mirrors `finish_arg` for named ctor fields — the label
    /// is required there, so a missing one is a syntax error).
    fn finish_ctor_field(frames: &mut Vec<Frame>, vals: &mut Vec<Expr>) -> Result<(), Diagnostic> {
        while frames
            .last()
            .is_some_and(|f| matches!(f, Frame::Op(OpFrame::Bin { .. })))
        {
            Self::reduce_ops(frames, vals);
        }
        let Some(Frame::Ctor(cf)) = frames.last_mut() else {
            unreachable!("finish_ctor_field with ctor frame on top");
        };
        let name = cf
            .pending_name
            .take()
            .expect("ctor field requires a `name:` label");
        if vals.len() <= cf.mark {
            return Err(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                "expected expression after field name",
                name.span,
            ));
        }
        let value = vals.pop().expect("ctor field value");
        let span = name.span.join(value.span);
        cf.fields.push(NamedCtorField { name, value, span });
        cf.mark = vals.len();
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
                    Tok::Ident(ref name)
                        if self.surface.is_collections()
                            && matches!(
                                self.peek2().map(|t2| &t2.tok),
                                Some(Tok::Punct(P::Lt))
                            )
                            && self.generic_scan_commits(self.pos + 1) =>
                    {
                        // PLAN-864: `List<T>` in receiver position
                        // (`List<int>.new()`). The scan only commits when a
                        // balanced `<Type...>` is followed by `.` — a plain
                        // `a < b` comparison never enters this arm.
                        let gname = Ident {
                            name: name.clone(),
                            span,
                        };
                        self.pos += 1; // the identifier
                        self.last_line = self.toks[self.pos - 1].line;
                        let args = self.type_arg_list()?;
                        let gspan = gname.span.join(self.toks[self.pos - 1].span);
                        // Falls through to the operator/postfix loop so a
                        // `.new(...)` method call can chain onto the type.
                        vals.push(Expr {
                            span: gspan,
                            kind: ExprKind::GenericType { name: gname, args },
                        });
                    }
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
                            _ if self.surface.surface_is_scalar_or_wider() => {
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
                    Tok::StrLit(value) => {
                        self.pos += 1;
                        self.last_line = line;
                        vals.push(Expr {
                            span,
                            kind: ExprKind::StrLit(value),
                        });
                    }
                    Tok::CharLit(scalar) => {
                        self.pos += 1;
                        self.last_line = line;
                        vals.push(Expr {
                            span,
                            kind: ExprKind::CharLit(scalar),
                        });
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
                        if self.surface.surface_is_scalar_or_wider() {
                            self.pos += 1;
                            self.last_line = line;
                            frames.push(Frame::Op(OpFrame::Unary(UnaryOp::Not)));
                            continue 'outer;
                        }
                        return Err(
                            self.unsupported("unary `!` is not part of this source profile", span)
                        );
                    }
                    Tok::Punct(P::Plus) if self.surface.surface_is_scalar_or_wider() => {
                        // unary `+` stays excluded on the scalar surface;
                        // the core surface keeps its historical plain
                        // syntax rejection for a leading `+`
                        return Err(
                            self.unsupported("unary `+` is not part of this source profile", span)
                        );
                    }
                    Tok::Kw(Kw::Type) | Tok::Kw(Kw::Static) | Tok::Kw(Kw::Mut) => {
                        // never value-forming on any profile (mut exists
                        // only as a param prefix / `.mut` suffix on the
                        // record surface)
                        return Err(self.unsupported(
                            format!(
                                "`{}` is not a value in this source profile",
                                match tok {
                                    Tok::Kw(Kw::Type) => "type",
                                    Tok::Kw(Kw::Static) => "static",
                                    _ => "mut",
                                }
                            ),
                            span,
                        ));
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
                    Tok::Punct(P::Dot) if self.surface.is_record() => {
                        // PLAN-771: leading `.name` — implicit `self` in
                        // method bodies (real parser's dot_item: the base is
                        // Dot(Ident self, name)). Inside any bracket a call
                        // form `.name(args)` opens a method frame on `self`.
                        self.pos += 1;
                        self.last_line = line;
                        let Some(Token {
                            tok: Tok::Ident(mname),
                            span: mspan,
                            line: mline,
                        }) = self.peek().cloned()
                        else {
                            return Err(self.syntax(format!(
                                "expected field or method name after `.`, found {}",
                                self.describe_cur()
                            )));
                        };
                        let is_call = matches!(
                            self.peek2(),
                            Some(t2) if t2.tok == Tok::Punct(P::LParen)
                                && (Self::in_bracket(&frames) || t2.line == mline)
                        );
                        if is_call {
                            let callee = Ident {
                                name: mname.clone(),
                                span: mspan,
                            };
                            self.pos += 2;
                            self.last_line = self.toks[self.pos - 1].line;
                            if matches!(
                                self.peek(),
                                Some(Token {
                                    tok: Tok::Punct(P::RParen),
                                    ..
                                })
                            ) {
                                let close = self.bump();
                                let recv = Expr {
                                    span,
                                    kind: ExprKind::Var(Ident {
                                        name: "self".to_string(),
                                        span,
                                    }),
                                };
                                vals.push(Expr {
                                    span: span.join(close.span),
                                    kind: ExprKind::MethodCall {
                                        recv: Box::new(recv),
                                        name: callee,
                                        args: Vec::new(),
                                    },
                                });
                                continue 'outer;
                            }
                            let recv = Expr {
                                span,
                                kind: ExprKind::Var(Ident {
                                    name: "self".to_string(),
                                    span,
                                }),
                            };
                            frames.push(Frame::Call(Box::new(CallFrame {
                                callee,
                                args: Vec::new(),
                                mark: vals.len(),
                                pending_name: None,
                                method_recv: Some(Box::new(recv)),
                            })));
                            self.take_named_label(&mut frames)?;
                            continue 'outer;
                        }
                        self.pos += 1;
                        self.last_line = mline;
                        vals.push(Expr {
                            span: span.join(mspan),
                            kind: ExprKind::Field {
                                base: Box::new(Expr {
                                    span,
                                    kind: ExprKind::Var(Ident {
                                        name: "self".to_string(),
                                        span,
                                    }),
                                }),
                                name: Ident {
                                    name: mname,
                                    span: mspan,
                                },
                            },
                        });
                    }
                    Tok::Ident(name) => {
                        // PLAN-771: named constructor `Ty { field: v, ... }`
                        // when `{` follows on the callee's line (or inside
                        // brackets). Whether `name` really is a record type
                        // is resolved later; the parse is purely syntactic.
                        if self.surface.is_record()
                            && matches!(
                                self.peek2(),
                                Some(t2) if t2.tok == Tok::Punct(P::LBrace)
                                    && (Self::in_bracket(&frames) || t2.line == line)
                            )
                        {
                            let ty = Ident {
                                name: name.clone(),
                                span,
                            };
                            self.pos += 2; // ident + `{`
                            self.last_line = self.toks[self.pos - 1].line;
                            // empty ctor `Ty { }`?
                            if matches!(
                                self.peek(),
                                Some(Token {
                                    tok: Tok::Punct(P::RBrace),
                                    ..
                                })
                            ) {
                                let close = self.bump();
                                vals.push(Expr {
                                    span: ty.span.join(close.span),
                                    kind: ExprKind::CtorNamed {
                                        ty,
                                        fields: Vec::new(),
                                    },
                                });
                                continue 'outer;
                            }
                            frames.push(Frame::Ctor(Box::new(CtorFrame {
                                ty,
                                fields: Vec::new(),
                                mark: vals.len(),
                                pending_name: None,
                            })));
                            self.take_named_label(&mut frames)?;
                            continue 'outer;
                        }
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
                                    method_recv: None,
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
                // PLAN-771: the dot postfix (field read / method call /
                // `p.mut`) binds tighter than any binary operator; it is
                // consumed here, before the operator table is consulted.
                if self.surface.is_record()
                    // the text `.to(str|Str)` postfix stays with the text
                    // arm on the combination surface (`to` is a Reserved
                    // word this record arm would otherwise reject)
                    && !matches!(
                        self.peek2(),
                        Some(Token {
                            tok: Tok::Reserved(w),
                            ..
                        }) if self.surface.is_text() && w == "to"
                    )
                    && matches!(
                        self.peek(),
                        Some(Token {
                            tok: Tok::Punct(P::Dot),
                            ..
                        })
                    )
                {
                    let dline = self.peek().unwrap().line;
                    if !Self::in_bracket(&frames) && dline != self.last_line {
                        return Err(
                            self.syntax("unbracketed expression cannot continue across a newline")
                        );
                    }
                    self.pos += 1;
                    self.last_line = dline;
                    match self.peek().cloned() {
                        // PLAN-782: in the combination profile a `.move`
                        // postfix on the record surface reuses the text
                        // wrapper form — resolve gates the named-local
                        // receiver, verify owns the resource typing.
                        Some(Token {
                            tok: Tok::Reserved(w),
                            span: mspan,
                            line: mline,
                        }) if (self.surface.is_text() || self.surface.is_collections())
                            && w == "move" =>
                        {
                            self.pos += 1;
                            self.last_line = mline;
                            let recv = vals.pop().expect("dot receiver");
                            let joined = recv.span.join(mspan);
                            vals.push(Expr {
                                span: joined,
                                kind: ExprKind::Method {
                                    recv: Box::new(recv),
                                    name: Ident {
                                        name: "move".to_string(),
                                        span: mspan,
                                    },
                                    args: Vec::new(),
                                },
                            });
                            continue;
                        }
                        Some(Token {
                            tok: Tok::Kw(Kw::Mut),
                            span: mspan,
                            ..
                        }) => {
                            // `p.mut` — the receiver must be a bare local
                            self.pos += 1;
                            let recv = vals.pop().expect("dot receiver");
                            let name = match &recv.kind {
                                ExprKind::Var(id) => id.clone(),
                                _ => {
                                    return Err(self.unsupported(
                                        "`.mut` applies to a local variable",
                                        recv.span.join(mspan),
                                    ))
                                }
                            };
                            vals.push(Expr {
                                span: name.span.join(mspan),
                                kind: ExprKind::MutArg { name },
                            });
                            continue;
                        }
                        Some(Token {
                            tok: Tok::Ident(mname),
                            span: mspan,
                            line: mline,
                        }) => {
                            let is_call = matches!(
                                self.peek2(),
                                Some(t2) if t2.tok == Tok::Punct(P::LParen)
                                    && (Self::in_bracket(&frames) || t2.line == mline)
                            );
                            if is_call {
                                // `recv.name(args)` — a method-call frame
                                // carrying the evaluated receiver
                                let callee = Ident {
                                    name: mname.clone(),
                                    span: mspan,
                                };
                                self.pos += 2; // name + `(`
                                self.last_line = self.toks[self.pos - 1].line;
                                if matches!(
                                    self.peek(),
                                    Some(Token {
                                        tok: Tok::Punct(P::RParen),
                                        ..
                                    })
                                ) {
                                    let close = self.bump();
                                    let recv = vals.pop().expect("method receiver");
                                    vals.push(Expr {
                                        span: recv.span.join(close.span),
                                        kind: ExprKind::MethodCall {
                                            recv: Box::new(recv),
                                            name: callee,
                                            args: Vec::new(),
                                        },
                                    });
                                    continue;
                                }
                                let recv = vals.pop().expect("method receiver");
                                frames.push(Frame::Call(Box::new(CallFrame {
                                    callee,
                                    args: Vec::new(),
                                    mark: vals.len(),
                                    pending_name: None,
                                    method_recv: Some(Box::new(recv)),
                                })));
                                self.take_named_label(&mut frames)?;
                                continue 'outer;
                            }
                            // plain field read
                            self.pos += 1;
                            self.last_line = mline;
                            let base = vals.pop().expect("field base");
                            let joined = base.span.join(mspan);
                            vals.push(Expr {
                                span: joined,
                                kind: ExprKind::Field {
                                    base: Box::new(base),
                                    name: Ident {
                                        name: mname,
                                        span: mspan,
                                    },
                                },
                            });
                            continue;
                        }
                        _ => {
                            return Err(self.syntax(format!(
                                "expected field or method name after `.`, found {}",
                                self.describe_cur()
                            )))
                        }
                    }
                }
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
                    Tok::Punct(P::Dot) if self.surface.is_text() => {
                        // PLAN-770 method postfix: `.name(args)`, `.to(str|Str)`,
                        // bare `.move`. Non-text surfaces keep the historical
                        // end-of-expression behavior (the statement layer owns
                        // the rejection).
                        match self.peek2().cloned() {
                            Some(Token {
                                tok: Tok::Reserved(w),
                                span: mspan,
                                line: mline,
                            }) if w == "move" => {
                                if !Self::in_bracket(&frames) && mline != self.last_line {
                                    return Err(self.syntax(
                                        "unbracketed expression cannot continue across a newline",
                                    ));
                                }
                                self.pos += 2;
                                self.last_line = mline;
                                let Some(recv) = vals.pop() else {
                                    return Err(self.syntax("`.move` needs a value before it"));
                                };
                                let span = recv.span.join(mspan);
                                vals.push(Expr {
                                    span,
                                    kind: ExprKind::Method {
                                        recv: Box::new(recv),
                                        name: Ident {
                                            name: "move".to_string(),
                                            span: mspan,
                                        },
                                        args: Vec::new(),
                                    },
                                });
                                continue; // re-examine what follows the postfix
                            }
                            Some(Token {
                                tok: Tok::Reserved(w),
                                span: tspan,
                                ..
                            }) if w == "to" => {
                                // `.to(str)` / `.to(Str)`: single type word
                                let _ = tspan;
                                self.pos += 1; // consume dot; `to` next
                                let to_tok = self.bump();
                                self.last_line = to_tok.line;
                                self.expect_punct(P::LParen, "`(` after `.to`")?;
                                let word = match self.peek().cloned() {
                                    Some(Token {
                                        tok: Tok::Ident(w),
                                        span: wspan,
                                        ..
                                    }) if w == "str" => (TypeWord::Str, wspan),
                                    Some(Token {
                                        tok: Tok::Ident(w),
                                        span: wspan,
                                        ..
                                    }) if w == "Str" => (TypeWord::StrOwned, wspan),
                                    _ => return Err(self.syntax("`.to` expects `str` or `Str`")),
                                };
                                self.pos += 1;
                                self.last_line = self.toks[self.pos - 1].line;
                                self.expect_punct(P::RParen, "`)` after the `.to` type")?;
                                let Some(recv) = vals.pop() else {
                                    return Err(self.syntax("`.to` needs a value before it"));
                                };
                                let span = recv.span.join(word.1);
                                vals.push(Expr {
                                    span,
                                    kind: ExprKind::Method {
                                        recv: Box::new(recv),
                                        name: Ident {
                                            name: "to".to_string(),
                                            span: word.1,
                                        },
                                        args: vec![Arg::Pos(Expr {
                                            span: word.1,
                                            kind: ExprKind::TypeWordArg(word.0),
                                        })],
                                    },
                                });
                                continue;
                            }
                            Some(Token {
                                tok: Tok::Ident(mname),
                                span: mspan,
                                line: mline,
                            }) => {
                                let is_call = matches!(
                                    self.peek3(),
                                    Some(t3) if t3.tok == Tok::Punct(P::LParen)
                                        && (Self::in_bracket(&frames) || t3.line == mline)
                                );
                                if !is_call {
                                    return Err(self.syntax(format!(
                                        "method `.{}` requires parentheses",
                                        mname
                                    )));
                                }
                                // consume dot + name + `(`
                                self.pos += 3;
                                self.last_line = self.toks[self.pos - 1].line;
                                let mut recv = vals.pop().ok_or_else(|| {
                                    self.syntax(format!(
                                        "method `.{}` needs a value before it",
                                        mname
                                    ))
                                })?;
                                // type-level receiver: `char.from_codepoint(...)`
                                // — only when `char` is the bare receiver word
                                if mname == "from_codepoint" {
                                    if let ExprKind::Var(id) = &recv.kind {
                                        if id.name == "char" {
                                            recv = Expr {
                                                span: id.span,
                                                kind: ExprKind::TypeWordArg(TypeWord::Char),
                                            };
                                        }
                                    }
                                }
                                let callee = Ident {
                                    name: mname,
                                    span: mspan,
                                };
                                if matches!(
                                    self.peek(),
                                    Some(Token {
                                        tok: Tok::Punct(P::RParen),
                                        ..
                                    })
                                ) {
                                    let close = self.bump();
                                    let span = recv.span.join(close.span);
                                    vals.push(Expr {
                                        span,
                                        kind: ExprKind::Method {
                                            recv: Box::new(recv),
                                            name: callee,
                                            args: Vec::new(),
                                        },
                                    });
                                    continue;
                                }
                                frames.push(Frame::Call(Box::new(CallFrame {
                                    callee,
                                    method_recv: Some(Box::new(recv)),
                                    args: Vec::new(),
                                    mark: vals.len(),
                                    pending_name: None,
                                })));
                                self.take_named_label(&mut frames)?;
                                continue 'outer;
                            }
                            _ => {
                                // dot without a method name on the text surface
                                return Err(self.syntax("expected a method name after `.`"));
                            }
                        }
                    }
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
                    Tok::Punct(P::RBrace) => {
                        // PLAN-771: closes a named-constructor frame
                        // (`Ty { a: 1,\n b: 2 }` — commas or newlines
                        // separate fields; newlines end the previous field
                        // expression exactly like the statement layer)
                        while let Some(Frame::Op(OpFrame::Bin { .. })) = frames.last() {
                            Self::reduce_once(&mut frames, &mut vals);
                        }
                        match frames.last() {
                            Some(Frame::Ctor(_)) => {
                                if let Some(Frame::Ctor(cf)) = frames.last() {
                                    if vals.len() > cf.mark || cf.pending_name.is_some() {
                                        Self::finish_ctor_field(&mut frames, &mut vals)?;
                                    }
                                }
                                let Some(Frame::Ctor(cf)) = frames.pop() else {
                                    unreachable!()
                                };
                                self.pos += 1;
                                self.last_line = tok.line;
                                vals.push(Expr {
                                    span: cf.ty.span.join(tok.span),
                                    kind: ExprKind::CtorNamed {
                                        ty: cf.ty,
                                        fields: cf.fields,
                                    },
                                });
                                // continue: an operator or another `}` may follow
                            }
                            _ => {
                                // no ctor frame: expression ends at `}`
                                Self::reduce_ops(&mut frames, &mut vals);
                                match frames.last() {
                                    Some(Frame::Call(_)) => {
                                        return Err(self.syntax("expected `,` or `)`"))
                                    }
                                    Some(Frame::Op(OpFrame::Paren)) => {
                                        return Err(self.syntax("expected `)`"))
                                    }
                                    Some(_) => unreachable!(),
                                    None => {}
                                }
                                break 'outer;
                            }
                        }
                    }
                    Tok::Punct(P::Comma) if matches!(frames.last(), Some(Frame::Ctor(_))) => {
                        Self::finish_ctor_field(&mut frames, &mut vals)?;
                        self.pos += 1;
                        self.last_line = tok.line;
                        self.take_named_label(&mut frames)?;
                        continue 'outer; // next field value primary
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
                                match cf.method_recv {
                                    Some(recv) if self.surface.is_record() => {
                                        let span = cf.callee.span.join(tok.span);
                                        vals.push(Expr {
                                            span: recv.span.join(span),
                                            kind: ExprKind::MethodCall {
                                                recv,
                                                name: cf.callee,
                                                args: cf.args,
                                            },
                                        });
                                    }
                                    Some(recv) => {
                                        let span = recv.span.join(tok.span);
                                        vals.push(Expr {
                                            span,
                                            kind: ExprKind::Method {
                                                recv,
                                                name: cf.callee,
                                                args: cf.args,
                                            },
                                        });
                                    }
                                    None => {
                                        vals.push(Expr {
                                            span: cf.callee.span.join(tok.span),
                                            kind: ExprKind::Call {
                                                callee: cf.callee,
                                                args: cf.args,
                                            },
                                        });
                                    }
                                }
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
                        // PLAN-771: inside a named ctor braces, a NEWLINE
                        // separates fields exactly like a comma (the real
                        // parser's ctor_node accepts both).
                        if matches!(frames.last(), Some(Frame::Ctor(_)))
                            && tok.line > self.last_line
                        {
                            if let Some(Frame::Ctor(cf)) = frames.last() {
                                if vals.len() > cf.mark || cf.pending_name.is_some() {
                                    Self::finish_ctor_field(&mut frames, &mut vals)?;
                                }
                            }
                            self.take_named_label(&mut frames)?;
                            continue 'outer; // next field value primary
                        }
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
        match frames.last_mut() {
            Some(Frame::Call(cf)) => {
                cf.pending_name = Some(Ident { name, span });
            }
            Some(Frame::Ctor(cf)) => {
                cf.pending_name = Some(Ident { name, span });
            }
            _ => {}
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
