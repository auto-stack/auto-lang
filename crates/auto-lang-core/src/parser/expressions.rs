//! PLAN-843 结构拆分（自 parser.rs 原样搬移；算法/行为零变化）。
use super::*;

impl<'a> Parser<'a> {
    /// Create parser
    /// Plan 059 续(§B8):把单个表达式文本(如 ".block.id")解析为 Expr。
    /// 供事件参数求值复用视图表达式解析器。
    pub fn parse_expr_fragment(src: &str) -> Option<crate::ast::Expr> {
        let code = src.trim().trim_start_matches('.').to_string();
        if code.is_empty() {
            return None;
        }
        let mut parser = Parser::new(&code);
        match parser.parse_expr() {
            Ok(expr) => Some(expr),
            Err(_) => None,
        }
    }

    pub fn parse_expr(&mut self) -> AutoResult<Expr> {
        // Plan 364 W5 (D4): explicit `move` closure prefix.
        // `move (params) => body` and `move x => body` → Expr::Closure with
        // is_move=true. We detect `move` at the expression boundary so both the
        // multi-param path (pratt→atom→group→parse_closure) and the single-param
        // path (expr_pratt_with_left's `x =>` arm) are covered. `move` here must
        // be followed by `(` or an identifier to be a closure prefix; otherwise
        // it falls through to its existing roles (param mode `fn(move x)`,
        // soft-identifier use as a variable name).
        if self.is_kind(TokenKind::Move) {
            // Lookahead: consume the token after `move` and push it back.
            let after = self.lexer.next()?;
            let is_closure_prefix = matches!(after.kind, TokenKind::LParen | TokenKind::Ident);
            self.lexer.push_token(after);
            if is_closure_prefix {
                self.next(); // consume 'move'
                let start_pos = self.cur.pos; // Plan 410: expression start for check_symbol's error span
                let mut exp = self.expr_pratt(0)?;
                if let Expr::Closure(ref mut c) = exp {
                    c.is_move = true;
                }
                exp = self.check_symbol(exp, pos_to_span(start_pos))?;
                return Ok(exp);
            }
        }

        // Plan 410: record the expression start BEFORE expr_pratt so
        // check_symbol's UndefinedVariable span points at the offending
        // identifier, not at the cursor (which has moved past the expression).
        let start_pos = self.cur.pos;
        let mut exp = self.expr_pratt(0)?;
        exp = self.check_symbol(exp, pos_to_span(start_pos))?;
        Ok(exp)
    }

    // simple Pratt parser
    // ref: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
    pub fn expr_pratt(&mut self, min_power: u8) -> AutoResult<Expr> {
        // Plan 419 §8.3:表达式递归咽喉(cdb 实测括号链在此自递归)。
        stacker::maybe_grow(512 * 1024, 1024 * 1024, || self.expr_pratt_inner(min_power))
    }

    fn expr_pratt_inner(&mut self, min_power: u8) -> AutoResult<Expr> {
        // hold expression (Phase 3) - special case that returns directly
        if self.is_kind(TokenKind::Hold) {
            let start_pos = self.cur.pos; // Record start position

            self.next(); // skip hold

            // Parse the path expression (identifier with optional member access)
            let path = self.parse_path_expr()?;

            // Expect 'as' keyword
            self.expect(TokenKind::As)?;

            // Get the binding name
            let name = self.cur.text.clone();
            self.expect(TokenKind::Ident)?;

            // Skip empty lines before body (allows newline between name and {)
            if self.is_kind(TokenKind::Newline) {
                self.skip_empty_lines();
            }

            // Disable symbol checking inside hold body (bindings are created at runtime)
            let old_skip_check = self.skip_check;
            self.skip_check = true;

            // Parse the body (a block)
            let body = self.body()?;

            // Restore symbol checking
            self.skip_check = old_skip_check;

            // Calculate span: (offset, length)
            let span = Some((start_pos.pos, self.cur.pos.pos - start_pos.pos));

            // Hold is a complete expression, return directly
            return Ok(Expr::Hold(crate::ast::Hold {
                path: Box::new(path),
                name,
                body,
                span,
            }));
        }

        // Prefix
        let lhs = match self.kind() {
            // if expression
            TokenKind::If => self.if_expr()?,
            // unary
            TokenKind::Add | TokenKind::Sub | TokenKind::Not | TokenKind::Star => {
                let op = self.op();
                let span = pos_to_span(self.cur.pos);
                let power = prefix_power(op, span)?;
                self.next(); // skip unary op
                let lhs = self.expr_pratt(power.r)?;
                Expr::Unary(op, Box::new(lhs))
            }
            // group or multi-param closure
            TokenKind::LParen => {
                // Plan 060: Check if this is a multi-param closure: (a, b) => expr
                // Or zero-param closure: () => expr
                self.next(); // skip (

                // Quick check for zero-param closure: () => expr
                if self.is_kind(TokenKind::RParen) {
                    // Peek ahead to check for =>
                    if let Ok(next_token) = self.lexer.next() {
                        let is_closure = next_token.kind == TokenKind::DoubleArrow;
                        self.lexer.push_token(next_token);
                        if is_closure {
                            // This is a zero-param closure: () => expr
                            // Restore state so parse_closure sees the ( token
                            let saved_token = self.cur.clone();
                            self.cur = Token {
                                kind: TokenKind::LParen,
                                text: AutoStr::from("("),
                                pos: saved_token.pos,
                            };
                            self.lexer.push_token(saved_token);
                            return self.parse_closure();
                        }
                    }
                    // Not a closure, continue with regular group expression
                    // Plan 391 D5: `()` (empty, not a closure) is the unit
                    // value — e.g. `Ok(())`, `return ()`. Emit an empty tuple
                    // expression (a2r codegen renders Expr::Tuple([]) as `()`).
                    if self.is_kind(TokenKind::RParen) {
                        self.next(); // skip )
                        Expr::Tuple(vec![])
                    } else {
                        let lhs = self.expr_pratt(0)?;
                        self.expect(TokenKind::RParen)?;
                        lhs
                    }
                } else if self.is_kind(TokenKind::Ident) {
                    // Collect tokens for lookahead and rollback
                    let mut tokens = Vec::new();
                    let mut found_comma = false;
                    let mut found_double_arrow = false;
                    let mut depth = 1; // Track nesting for types like Array<T>

                    // Scan ahead to detect closure pattern
                    loop {
                        let token = self.lexer.next()?;

                        // Check for pattern before consuming token into buffer
                        let is_closing_paren = depth == 1 && token.kind == TokenKind::RParen;

                        tokens.push(token.clone());

                        match token.kind {
                            TokenKind::LParen | TokenKind::LSquare | TokenKind::LBrace => {
                                depth += 1
                            }
                            TokenKind::RParen | TokenKind::RSquare | TokenKind::RBrace => {
                                depth -= 1;
                            }
                            TokenKind::Comma if depth == 1 => found_comma = true,
                            TokenKind::EOF => break,
                            _ => {}
                        }

                        // If we found the closing paren at depth 0, check for =>
                        if is_closing_paren {
                            // Peek at the next token without consuming it
                            if let Ok(next_token) = self.lexer.next() {
                                if next_token.kind == TokenKind::DoubleArrow {
                                    found_double_arrow = true;
                                }
                                // Don't add the peeked token to our buffer
                                // We need to push it back so it's available later
                                self.lexer.push_token(next_token);
                            }
                            break;
                        }
                    }

                    // Push tokens back to lexer buffer in reverse order
                    for token in tokens.into_iter().rev() {
                        self.lexer.push_token(token);
                    }

                    // Plan 200: Tuple detection — comma found but no => means it's a tuple
                    let is_closure = found_double_arrow;
                    if found_comma && !found_double_arrow {
                        // Tuple expression: (expr1, expr2, ...)
                        // We already consumed (, parse first expr, then comma-separated rest
                        let first = self.expr_pratt(0)?;
                        let mut elems = vec![first];
                        while self.is_kind(TokenKind::Comma) {
                            self.next(); // skip ,
                            elems.push(self.expr_pratt(0)?);
                        }
                        self.expect(TokenKind::RParen)?;
                        Expr::Tuple(elems)
                    } else if is_closure {
                        // Multi-param closure: (a, b) => expr
                        // Need to restore state so parse_closure sees the ( token
                        // Save current token (identifier)
                        let ident_token = self.cur.clone();

                        // Set current token back to (
                        self.cur = Token {
                            kind: TokenKind::LParen,
                            text: AutoStr::from("("),
                            pos: ident_token.pos, // Use identifier's position for better error messages
                        };

                        // Push the identifier back to lexer
                        self.lexer.push_token(ident_token);

                        // Now parse_closure will see ( as current token
                        self.parse_closure()?
                    } else {
                        // Regular group expression: (expr)
                        let lhs = self.expr_pratt(0)?;
                        self.expect(TokenKind::RParen)?; // skip )
                        lhs
                    }
                } else {
                    // Regular group expression or tuple starting with non-identifier
                    let first = self.expr_pratt(0)?;
                    if self.is_kind(TokenKind::Comma) {
                        // Tuple expression: (expr1, expr2, ...)
                        let mut elems = vec![first];
                        while self.is_kind(TokenKind::Comma) {
                            self.next(); // skip ,
                            elems.push(self.expr_pratt(0)?);
                        }
                        self.expect(TokenKind::RParen)?;
                        Expr::Tuple(elems)
                    } else {
                        self.expect(TokenKind::RParen)?; // skip )
                        first
                    }
                }
            }
            // array
            TokenKind::LSquare => self.array()?,
            // object
            TokenKind::LBrace => Expr::Object(self.object()?),
            // fstr
            TokenKind::FStrStart => self.fstr()?,
            // grid (Plan 408: 'grid' is no longer a hard keyword — the lexer
            // emits TokenKind::Ident for it. Detect the AutoData grid(...)
            // literal textually, mirroring Plan 354's text-based nav() check
            // in call(). TokenKind::Grid is retained for compatibility but no
            // longer produced by the lexer.)
            TokenKind::Grid => Expr::Grid(self.grid()?),
            // Only treat `grid(` as the grid literal (one-token lookahead); a
            // bare `grid` stays an ordinary identifier (variable reference).
            TokenKind::Ident if self.cur.text.as_str() == "grid" => {
                let saved_cur = self.cur.clone();
                self.next();
                let is_literal = self.is_kind(TokenKind::LParen);
                self.lexer.push_token(self.cur.clone());
                self.cur = saved_cur;
                if is_literal {
                    Expr::Grid(self.grid()?)
                } else {
                    self.atom()?
                }
            }
            // dot
            TokenKind::Dot => self.dot_item()?,
            // Plan 095: Compile-time expression #{ expr }
            TokenKind::HashBrace => {
                self.next(); // skip #{
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RBrace)?;
                Expr::Comptime(Box::new(HashBrace { expr }))
            }
            // Plan 223: is as expression (e.g., `let x = is y { ... }`)
            TokenKind::Is => {
                let is = self.parse_is()?;
                Expr::Is(Box::new(is))
            }
            // normal
            _ => self.atom()?,
        };
        self.expr_pratt_with_left(lhs, min_power)
    }

    /// Parse a path expression (identifier with optional member access)
    /// Used for hold expressions to parse paths like `obj.field.subfield`
    /// Stops before keywords like 'as', 'in', etc.
    fn parse_path_expr(&mut self) -> AutoResult<Expr> {
        let mut lhs = self.atom()?;

        // Allow member access (dot operator) for paths like obj.field
        loop {
            if self.is_kind(TokenKind::Dot) {
                self.next(); // skip .
                let field_name = self.cur.text.clone();
                // Allow @, * as special field names for pointer operations
                // Allow numeric literals for integer-keyed objects: a.1, a.2
                // Allow boolean keywords: a.true, a.false
                // Allow 'type' keyword as property name: expr.type
                if self.is_kind(TokenKind::Ident)
                    || self.is_kind(TokenKind::At)
                    || self.is_kind(TokenKind::Star)
                    || self.is_kind(TokenKind::Int)
                    || self.is_kind(TokenKind::Uint)
                    || self.is_kind(TokenKind::I8)
                    || self.is_kind(TokenKind::U8)
                    || self.is_kind(TokenKind::Float)
                    || self.is_kind(TokenKind::Double)
                    || self.is_kind(TokenKind::True)
                    || self.is_kind(TokenKind::False)
                    || self.is_kind(TokenKind::Nil)
                    || self.is_kind(TokenKind::Type)
                    || self.is_kind(TokenKind::Spawn)
                    || self.is_kind(TokenKind::Tag)
                // Allow .type property and .spawn method;
                // 2026-08-22(tag 软关键字):.tag 字段访问 —— TaggedCell.tag 等
                {
                    self.next();
                    // Use Expr::Dot for semantic clarity
                    lhs = Expr::Dot(Box::new(lhs), field_name);
                } else {
                    let message = format!(
                        "Expected identifier, @, *, number, or boolean after dot, got {:?}",
                        self.cur.kind
                    );
                    let span = pos_to_span(self.cur.pos);
                    return Err(SyntaxError::Generic { message, span }.into());
                }
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    pub(crate) fn dot_item(&mut self) -> AutoResult<Expr> {
        self.next(); // skip dot
        let name = self.cur.text.clone();
        self.next(); // skip name
                     // Use Expr::Dot for semantic clarity (.field is shorthand for self.field)
        let mut lhs = Expr::Dot(Box::new(Expr::Ident("self".into())), name);

        // Handle chained property access like .section.title
        while self.is_kind(TokenKind::Dot) {
            // Plan 162: Check for .as(Type) / .to(Type) — peek at next token
            let next_is_as_or_to = if let Ok(tok) = self.lexer.next() {
                let is_special = matches!(tok.kind, TokenKind::As | TokenKind::To);
                self.lexer.push_token(tok);
                is_special
            } else {
                false
            };
            if next_is_as_or_to {
                // This is .as(Type) or .to(Type), not a field access — let the Pratt parser handle it
                break;
            }
            self.next(); // skip dot
            let field_name = self.cur.text.clone();
            self.next(); // skip field name
            lhs = Expr::Dot(Box::new(lhs), field_name);
        }

        // Plan 408 P7 / §7.5 缺陷 6: handle dynamic indexing on a dot lhs,
        // e.g. `.row[.col]` (object value by dynamic key) or `.list[.i]`.
        // dot_item previously returned just `.row`, leaving `[.col]` in the
        // token stream — which then parsed as stray children/props, breaking
        // `text .row[.col]` into multiple nodes. Wrap as Expr::Index so the
        // whole expression stays one node → `{{ row[col] }}`.
        while self.is_kind(TokenKind::LSquare) {
            self.next(); // skip [
            let index = self.parse_expr()?;
            self.expect(TokenKind::RSquare)?;
            lhs = Expr::Index(Box::new(lhs), Box::new(index));
        }

        // Plan 043: `.field = expr` — consume the assignment here so the
        // resulting Dot-lhs never re-enters expr_pratt_with_left's Asn path
        // (which recursed deeply for ash-gui api.at). Only fires when the dot
        // chain is immediately followed by `=`; method calls / field access
        // continue through the normal Pratt path.
        if self.is_kind(TokenKind::Asn) {
            self.next(); // consume =
            let rhs = self.parse_expr()?;
            lhs = Expr::Bina(Box::new(lhs), Op::Asn, Box::new(rhs));
        }
        Ok(lhs)
    }

    /// Plan 395: is `expr` a plausible call target — a name, a method chain
    /// (`a.b.c`), or a path (`env::var`)? Only these can carry `<Type>` generic
    /// args. Backtracking (in try_parse_call_generic_args) guards the rest.
    fn is_callable_lhs(expr: &Expr) -> bool {
        match expr {
            Expr::Ident(_) => true,
            Expr::Dot(_, _) => true,
            Expr::Bina(_, op, _) if matches!(op, Op::Dot) => true,
            _ => false,
        }
    }

    /// Plan 395: speculatively parse `<T1, T2>` generic type args on a call
    /// (turbofish). Returns:
    /// - `Some(types)` — the full `<types>(` shape parsed; caller must set
    ///   `pending_generic_args` (the `(` is still the current token).
    /// - `None` — not a generic-args call; the lexer/parser state has been
    ///   rolled back so the `<` can be re-consumed as a comparison operator.
    fn try_parse_call_generic_args(&mut self) -> AutoResult<Option<Vec<Type>>> {
        // Cheap guard (no state to roll back): the token after `<` must look
        // like a type start. Mirrors next_token_is_type's token set.
        let after_lt_is_type = match self.lexer.next() {
            Ok(tok) => {
                let is_type = matches!(
                    tok.kind,
                    TokenKind::Ident
                        | TokenKind::Question
                        | TokenKind::LSquare
                        | TokenKind::Star
                        | TokenKind::Lt
                );
                self.lexer.push_token(tok);
                is_type
            }
            Err(_) => false,
        };
        if !after_lt_is_type {
            return Ok(None);
        }

        // Speculative parse with full rollback on any failure.
        let lex_state = self.lexer.save_state();
        let saved_cur = self.cur.clone();
        let saved_prev = self.prev.clone();
        let saved_errors_len = self.errors.len();
        let saved_warnings_len = self.warnings.len();
        let mut rollback = |slf: &mut Self| {
            slf.lexer.restore_state(lex_state);
            slf.cur = saved_cur.clone();
            slf.prev = saved_prev.clone();
            slf.errors.truncate(saved_errors_len);
            slf.warnings.truncate(saved_warnings_len);
        };

        self.next(); // consume '<'
        let mut types = Vec::new();
        match self.parse_type() {
            Ok(t) => types.push(t),
            Err(_) => {
                rollback(self);
                return Ok(None);
            }
        }
        while self.is_kind(TokenKind::Comma) {
            self.next();
            match self.parse_type() {
                Ok(t) => types.push(t),
                Err(_) => {
                    rollback(self);
                    return Ok(None);
                }
            }
        }
        if !self.is_kind(TokenKind::Gt) {
            rollback(self);
            return Ok(None);
        }
        self.next(); // consume '>'
        if !self.is_kind(TokenKind::LParen) {
            rollback(self);
            return Ok(None);
        }
        Ok(Some(types))
    }

    pub(super) fn expr_pratt_with_left(&mut self, lhs: Expr, min_power: u8) -> AutoResult<Expr> {
        // Plan 419 §8.3:表达式递归咽喉挂分段栈(见 parse 注释)。
        stacker::maybe_grow(512 * 1024, 1024 * 1024, || {
            self.expr_pratt_with_left_inner(lhs, min_power)
        })
    }

    fn expr_pratt_with_left_inner(&mut self, mut lhs: Expr, min_power: u8) -> AutoResult<Expr> {
        // Plan 060: Check for single-param closure:  x => expr
        // If lhs is an identifier and next token is =>, parse as closure
        if matches!(lhs, Expr::Ident(_)) && self.is_kind(TokenKind::DoubleArrow) {
            use crate::ast::{Closure, ClosureParam};

            // Extract parameter name from identifier
            let param_name = match &lhs {
                Expr::Ident(name) => name.clone(),
                _ => unreachable!(),
            };

            // Expect =>
            self.expect(TokenKind::DoubleArrow)?;

            // Register parameter in scope so body can reference it
            self.infer_ctx.bind_var(
                crate::ast::Name::from(param_name.as_str()),
                crate::ast::Type::Unknown,
            );

            // Parse body (expression or block)
            let body = if self.is_kind(TokenKind::LBrace) {
                Expr::Block(self.body()?)
            } else {
                self.parse_expr()?
            };

            // Note: we don't unbind the param — the InferenceContext is flat
            // and doesn't support scope pops. This is acceptable because closure
            // param names don't conflict with outer scope names.

            return Ok(Expr::Closure(Closure::new(
                vec![ClosureParam::new(param_name, None)],
                None,
                body,
            )));
        }

        loop {
            // Plan 514 W5: 换行后行首 `|>` = 显式续行。越过换行探测 PipeGt
            // （save/restore 不误伤空行计数：命中时这些换行是续行分隔而非
            // 段落边界；未命中则完整回退照常 break）。门控与管道臂一致。
            if self.is_kind(TokenKind::Newline) {
                const PREC_PIPE_PEEK: InfixPrec = infix_prec(2);
                if PREC_PIPE_PEEK.l >= min_power {
                    let saved_lexer = self.lexer.save_state();
                    let saved_cur = self.cur.clone();
                    let saved_prev = self.prev.clone();
                    while self.is_kind(TokenKind::Newline) {
                        self.next();
                    }
                    if self.is_kind(TokenKind::PipeGt) {
                        continue; // 交给下面的管道臂
                    }
                    self.lexer.restore_state(saved_lexer);
                    self.cur = saved_cur;
                    self.prev = saved_prev;
                }
            }
            // Plan 514 W5: 管道算子 `|>`（docs/design/pipe-operator.md）。
            // 脱糖层位在 parser：`expr |> .m(args)` → `expr.m(args)`、
            // `expr |> .f` → `expr.f`（字段投影形）；左结合；优先级在
            // 赋值之上、比较之下（PREC_PIPE=infix_prec(2)）。行首形态即
            // 显式续行（替代已移除的换行流式链糖）。AST 零新节点——
            // VM/主 a2r 走既有方法调用路径。
            if self.is_kind(TokenKind::PipeGt) {
                const PREC_PIPE: InfixPrec = infix_prec(2);
                if PREC_PIPE.l >= min_power {
                    self.next(); // consume '|>'
                    if !self.is_kind(TokenKind::Dot) {
                        return Err(SyntaxError::Generic {
                            message: "expected `.method(...)` or `.field` after `|>`".to_string(),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                    self.next(); // consume '.'
                    let name = self.parse_name()?;
                    if self.is_kind(TokenKind::LParen) {
                        // 方法形：`|> .m(args)` → lhs.m(args)
                        let args = self.args()?;
                        lhs = Expr::Call(crate::ast::Call {
                            name: Box::new(Expr::Dot(Box::new(lhs), name)),
                            args,
                            ret: crate::ast::Type::Unknown,
                            type_args: Vec::new(),
                            generic_args: Vec::new(),
                            pos: Some(self.prev.pos),
                        });
                    } else {
                        // 字段投影形：`|> .f` → lhs.f
                        lhs = Expr::Dot(Box::new(lhs), name);
                    }
                    continue;
                }
            }
            // Plan 391 D4: path separator `::` in expressions (e.g. `env::var("X").ok()`).
            // Previously `env::var` parsed as a single Ident, then `.ok()` couldn't
            // chain onto the `env::var(...)` call (RHS method chain on a path-call
            // failed with "Expected Asn, but found ."). Treat `::` like `.` so the
            // path-call result becomes a valid lhs for the infix Dot loop.
            if self.is_kind(TokenKind::Colon) {
                let second_is_colon = if let Ok(tok) = self.lexer.next() {
                    let is_colon = tok.kind == TokenKind::Colon;
                    self.lexer.push_token(tok);
                    is_colon
                } else {
                    false
                };
                if second_is_colon && PREC_DOT.l >= min_power {
                    self.next(); // consume first ':'
                    self.next(); // consume second ':'
                    let segment = self.parse_name()?;
                    lhs = Expr::Dot(Box::new(lhs), segment);
                    continue;
                }
            }
            // Plan 395: turbofish-style generic type args on calls —
            // `obj.method<Type>(args)` or `fn<Type>(args)`. This is a
            // speculative parse: `a < b` comparisons (b not a type-start) and
            // `x < Foo > y` chains (no `(` after `>`) roll back to the plain
            // `<` comparison below. Only callable-shaped lhs can carry args.
            if self.is_kind(TokenKind::Lt) && Self::is_callable_lhs(&lhs) {
                if let Some(types) = self.try_parse_call_generic_args()? {
                    self.pending_generic_args = types;
                    continue;
                }
            }
            let mut op = match self.kind() {
                TokenKind::EOF
                | TokenKind::Newline
                | TokenKind::Semi
                | TokenKind::LBrace
                | TokenKind::RBrace
                | TokenKind::Comma
                | TokenKind::Arrow
                | TokenKind::DoubleArrow
                | TokenKind::VBar => break,
                // Plan 567 T12（P560-D1）: with 头部窗口内 `as` 是绑定位——
                // 表达式在此终止，由 with_stmt 消费。
                TokenKind::As if self.with_header => break,
                TokenKind::Add
                | TokenKind::Sub
                | TokenKind::Star
                | TokenKind::Div
                | TokenKind::Mod
                | TokenKind::Not => self.op(),
                TokenKind::AddEq
                | TokenKind::SubEq
                | TokenKind::MulEq
                | TokenKind::DivEq
                | TokenKind::ModEq => self.op(),
                TokenKind::DotView
                | TokenKind::DotMut
                | TokenKind::DotMove
                | TokenKind::DotTake
                | TokenKind::DotQuestion => {
                    // Property keywords: .view, .mut, .move, .take (Phase 3 / Plan 122)
                    // Error propagation: ?. (Phase 1b.3)
                    // These are postfix operators with same precedence as dot
                    self.op()
                }
                TokenKind::As if !self.with_header => {
                    // Infix `as` type cast: expr as Type
                    self.next(); // consume 'as'
                    let target_type = self.parse_type()?;
                    lhs = Expr::Cast {
                        expr: Box::new(lhs),
                        target_type,
                    };
                    continue;
                }
                TokenKind::Question => {
                    // Ternary conditional: condition ? true_expr : false_expr
                    // Precedence: very low (between assignment=1 and comparison)
                    const PREC_TERNARY: u8 = 2;
                    if PREC_TERNARY < min_power {
                        break;
                    }
                    self.next(); // consume ?
                                 // Parse true branch with min_power > PREC_PAIR.l (8) so : isn't consumed as pair
                    let true_expr = self.expr_pratt(9)?;
                    self.expect(TokenKind::Colon)?;
                    let false_expr = self.expr_pratt(PREC_TERNARY)?;
                    // Desugar into If expression
                    let true_body = Body {
                        stmts: vec![Stmt::Expr(true_expr)],
                        has_new_line: false,
                        source_lines: vec![],
                    };
                    let false_body = Body {
                        stmts: vec![Stmt::Expr(false_expr)],
                        has_new_line: false,
                        source_lines: vec![],
                    };
                    let branch = Branch {
                        cond: lhs,
                        body: true_body,
                    };
                    lhs = Expr::If(If {
                        branches: vec![branch],
                        else_: Some(false_body),
                    });
                    continue;
                }
                TokenKind::Dot => self.op(),
                TokenKind::Colon => self.op(),
                TokenKind::Range | TokenKind::RangeEq => self.op(),
                TokenKind::LSquare => self.op(),
                TokenKind::LParen => self.op(),
                TokenKind::Asn => self.op(),
                TokenKind::Eq
                | TokenKind::Neq
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::Le
                | TokenKind::Ge => self.op(),
                TokenKind::And | TokenKind::Or => self.op(),
                TokenKind::QuestionQuestion => self.op(),
                // Plan 560 T07 (C5/C6)：`a @ b` / `a ** b` 脚本糖——直接
                // 解析为桥调用（免新增 Op 变量的全仓波及；@T 引用类型
                // 在类型位不受影响——此臂仅在表达式 infix 位命中）。
                // Plan 560 T08 (C7)：`a is b` 同一性糖——词位判别（is 为
                // 普通标识符 token，非保留字；仅 infix 位命中，变量名
                // is 的常规用法在 atom 位不受影响）。
                TokenKind::Is => {
                    self.next(); // skip `is`
                    let rhs = self.atom()?;
                    lhs = mk_infix_sugar_call("py_is", lhs, rhs);
                    continue;
                }
                TokenKind::At | TokenKind::Power => {
                    let is_matmul = matches!(self.cur.kind, TokenKind::At);
                    self.next(); // skip @ / **
                    let rhs = self.atom()?;
                    lhs = mk_infix_sugar_call(
                        if is_matmul { "py_matmul" } else { "py_pow" },
                        lhs,
                        rhs,
                    );
                    continue;
                }
                TokenKind::RSquare => break,
                TokenKind::RParen => break,
                _ => {
                    let message = format!("Expected infix operator, got {:?}", self.peek());
                    let span = pos_to_span(self.cur.pos);
                    return Err(SyntaxError::Generic { message, span }.into());
                }
            };

            // Special case: .! is the bang operator for eager collection
            // When we see Op::Dot, check if the next token is ! and treat it as postfix Op::Not
            if matches!(op, Op::Dot) {
                // Try to peek at next token - if it's Not, convert this to a postfix Not operation
                // We need to be very careful here to not corrupt the token stream
                let next_is_bang = if let Ok(tok) = self.lexer.next() {
                    self.lexer.push_token(tok.clone());
                    matches!(tok.kind, TokenKind::Not)
                } else {
                    false
                };

                if next_is_bang {
                    // This is .! - skip the . and let postfix ! handle it
                    self.next(); // consume the .
                    op = Op::Not; // change to postfix Not operator
                }
            }

            // Postfix

            if let Ok(Some(power)) = postfix_power(op) {
                if power.l < min_power {
                    break;
                }

                match op {
                    // Index or slice
                    Op::LSquare => {
                        self.next(); // skip [
                        let rhs = if self.is_kind(TokenKind::Range)
                            || self.is_kind(TokenKind::RangeEq)
                        {
                            // Leading ..: expr[..end] or expr[..=end]
                            let eq = self.is_kind(TokenKind::RangeEq);
                            self.next(); // skip .. or ..=
                            let end_expr = self.expr_pratt(0)?;
                            Expr::Range(Range {
                                start: Box::new(Expr::Nil),
                                end: Box::new(end_expr),
                                eq,
                            })
                        } else if self.is_kind(TokenKind::RSquare) {
                            Expr::Nil
                        } else {
                            // Parse first expr with high priority to stop before consuming ..
                            let first = self.expr_pratt(18)?;
                            if self.is_kind(TokenKind::Range) || self.is_kind(TokenKind::RangeEq) {
                                let eq = self.is_kind(TokenKind::RangeEq);
                                self.next(); // skip .. or ..=
                                if self.is_kind(TokenKind::RSquare) {
                                    // Trailing ..: expr[pos..]
                                    Expr::Range(Range {
                                        start: Box::new(first),
                                        end: Box::new(Expr::Nil),
                                        eq,
                                    })
                                } else {
                                    // expr[start..end] or expr[start..end..step]
                                    let rhs_rest = self.expr_pratt(0)?;
                                    Expr::Range(Range {
                                        start: Box::new(first),
                                        end: Box::new(rhs_rest),
                                        eq,
                                    })
                                }
                            } else {
                                first
                            }
                        };
                        self.expect(TokenKind::RSquare)?;
                        lhs = Expr::Index(Box::new(lhs), Box::new(rhs));
                        continue;
                    }
                    // Call or Node Instance
                    Op::LParen => {
                        let args = self.args()?;
                        // Plan 395: mount explicit generic args (`<T>` turbofish)
                        // parsed by the loop-top intercept / node_or_call_expr.
                        let generic_args = std::mem::take(&mut self.pending_generic_args);
                        let mut expr = self.call(lhs, args)?;
                        if !generic_args.is_empty() {
                            if let Expr::Call(call) = &mut expr {
                                call.generic_args = generic_args;
                            }
                        }
                        lhs = expr;
                        continue;
                    }
                    // Pair
                    Op::Colon => {
                        self.next(); // skip :
                        let key = match &lhs {
                            Expr::Ident(name) => Key::NamedKey(name.clone()),
                            Expr::Int(i) => Key::IntKey(*i),
                            Expr::Bool(b) => Key::BoolKey(*b),
                            Expr::Str(s) => Key::StrKey(s.clone()),
                            _ => {
                                let message = format!("Invalid key: {}", lhs);
                                let span = pos_to_span(self.cur.pos);
                                return Err(SyntaxError::Generic { message, span }.into());
                            }
                        };
                        let rhs = self.pair_expr()?;
                        lhs = Expr::Pair(Pair {
                            key,
                            value: Box::new(rhs),
                        });
                        return Ok(lhs);
                    }
                    // Bang operator (!) for eager collection
                    // Converts expr! into expr.collect()
                    Op::Not => {
                        // Plan 442 C2: Rust 宏形态 `format!(...)` — musk 后端
                        // 语料直用(conversation/relay_api/task_plan/_engine ×7)。
                        // 不转 expr.collect()(那会把后随 (args) 的调用名变成
                        // format.collect 垃圾形状,VM codegen 的 Dynamic-call
                        // unimplemented 即由此来);保留 Ident(format),后随
                        // 括号按普通调用编译,由 codegen 路由 fmt.sprintf。
                        if let Expr::Ident(n) = &lhs {
                            if n.as_str() == "format" {
                                self.next(); // skip !
                                continue;
                            }
                        }
                        self.next(); // skip !
                        let collect_name = crate::ast::Name::from("collect");
                        let collect_expr =
                            Expr::Bina(Box::new(lhs), Op::Dot, Box::new(Expr::Ident(collect_name)));
                        lhs = Expr::Call(crate::ast::Call {
                            name: Box::new(collect_expr),
                            args: crate::ast::Args::new(),
                            ret: crate::ast::Type::Unknown,
                            type_args: Vec::new(),
                            generic_args: Vec::new(),
                            pos: Some(self.prev.pos),
                        });
                        continue;
                    }
                    _ => {
                        let message = format!("Invalid postfix operator: {}", op);
                        let span = pos_to_span(self.cur.pos);
                        return Err(SyntaxError::Generic { message, span }.into());
                    }
                }
            }
            // Infix
            let span = pos_to_span(self.cur.pos);
            let power = infix_power(op, span)?;
            if power.l < min_power {
                break;
            }
            self.next(); // skip binary op
                         // Plan 317: Allow line continuation after || and && (and other
                         // binary ops) so multi-line conditions work:
                         //   if a.contains(x) ||
                         //      b.contains(x) {
            if matches!(op, Op::Or | Op::And) {
                self.skip_empty_lines();
            }
            // Check for whether assignment is allowed
            match op {
                Op::Asn => {
                    self.check_asn(&lhs)?;
                }
                _ => {}
            }
            match op {
                // Property keywords (Phase 3): postfix operators, no rhs needed
                Op::DotView => {
                    lhs = Expr::View(Box::new(lhs));
                    continue;
                }
                Op::DotMut => {
                    lhs = Expr::Mut(Box::new(lhs));
                    continue;
                }
                Op::DotMove => {
                    // Plan 122: .move accessor for ownership transfer
                    lhs = Expr::Move(Box::new(lhs));
                    continue;
                }
                Op::DotTake => {
                    // Plan 122: Deprecated - emit warning and treat as .move
                    self.warnings.push(Warning::DeprecatedFeature {
                        name: ".take".to_string(),
                        message: "use '.move' instead".to_string(),
                        span: pos_to_span(self.cur.pos),
                    });
                    lhs = Expr::Move(Box::new(lhs));
                    continue;
                }
                // May type operators (Phase 1b.3): ?. error propagation
                // ?.       → ErrorPropagate (unwrap + propagate)
                Op::DotQuestion => {
                    if self.is_kind(TokenKind::LParen) {
                        // ?.(default) — safe unwrap with default value
                        self.next(); // consume (
                        let default = self.expr_pratt(0)?;
                        self.expect(TokenKind::RParen)?;
                        lhs = Expr::NullCoalesce(Box::new(lhs), Box::new(default));
                    } else {
                        lhs = Expr::ErrorPropagate(Box::new(lhs));
                    }
                    continue;
                }
                // Plan 120: .? error propagation (new Option/Result style)
                // .?       → ErrorPropagate (unwrap + propagate)
                // .?(expr) → NullCoalesce (unwrap with default)
                Op::DotQuest => {
                    if self.is_kind(TokenKind::LParen) {
                        // .?(default) — safe unwrap with default value
                        self.next(); // consume (
                        let default = self.expr_pratt(0)?;
                        self.expect(TokenKind::RParen)?;
                        lhs = Expr::NullCoalesce(Box::new(lhs), Box::new(default));
                    } else {
                        lhs = Expr::ErrorPropagate(Box::new(lhs));
                    }
                    continue;
                }
                _ => {
                    // Regular infix operators need rhs
                    // Plan 124: Special case for Dot operator - check for .await before parsing rhs
                    if matches!(op, Op::Dot) && self.is_kind(TokenKind::Await) {
                        // .await suffix - consume the 'await' token
                        self.next();
                        lhs = Expr::Await {
                            expr: Box::new(lhs),
                        };
                        continue;
                    }
                    // Plan 126: .go postfix operator - spawn background task
                    if matches!(op, Op::Dot) && self.is_kind(TokenKind::Go) {
                        // .go suffix - consume the 'go' token
                        self.next();
                        lhs = Expr::Go {
                            expr: Box::new(lhs),
                        };
                        continue;
                    }
                    // Plan 162: .as(Type) type conversion
                    if matches!(op, Op::Dot) && self.is_kind(TokenKind::As) {
                        // .as( Type ) — type conversion
                        self.next(); // consume 'as'
                        self.expect(TokenKind::LParen)?;
                        let target_type = self.parse_type()?;
                        self.expect(TokenKind::RParen)?;
                        lhs = Expr::Cast {
                            expr: Box::new(lhs),
                            target_type,
                        };
                        continue;
                    }
                    // Plan 162: .to(Type) explicit type conversion
                    if matches!(op, Op::Dot) && self.is_kind(TokenKind::To) {
                        // .to( Type ) — explicit type conversion (may allocate)
                        self.next(); // consume 'to'
                        self.expect(TokenKind::LParen)?;
                        let target_type = self.parse_type()?;
                        self.expect(TokenKind::RParen)?;
                        lhs = Expr::To {
                            expr: Box::new(lhs),
                            target_type,
                        };
                        continue;
                    }
                    // Handle enum variant constructors after dot: MayInt.Err(1), MayInt.Ok(val), etc.
                    // These keywords (Err, Ok, Some, None) should be treated as identifiers in this context
                    if matches!(op, Op::Dot)
                        && matches!(
                            self.cur.kind,
                            TokenKind::ErrKW
                                | TokenKind::OkKW
                                | TokenKind::SomeKW
                                | TokenKind::NoneKW
                        )
                    {
                        let variant_name: AutoStr = self.cur.text.clone().into();
                        self.next(); // consume the keyword token
                                     // Check if followed by parentheses (constructor call)
                        if self.is_kind(TokenKind::LParen) {
                            self.next(); // consume '('
                            let mut args = crate::ast::Args::new();
                            if !self.is_kind(TokenKind::RParen) {
                                args.args.push(crate::ast::Arg::Pos(self.parse_expr()?));
                                while self.is_kind(TokenKind::Comma) {
                                    self.next();
                                    args.args.push(crate::ast::Arg::Pos(self.parse_expr()?));
                                }
                            }
                            self.expect(TokenKind::RParen)?;
                            // Build: lhs.variant_name(args) as a method call
                            let method = Expr::Dot(Box::new(lhs), variant_name);
                            lhs = Expr::Call(crate::ast::Call {
                                name: Box::new(method),
                                args,
                                ret: Type::Unknown,
                                type_args: Vec::new(),
                                generic_args: Vec::new(),
                                pos: Some(self.prev.pos),
                            });
                        } else {
                            // Just field access: MayInt.Err (no args)
                            lhs = Expr::Dot(Box::new(lhs), variant_name);
                        }
                        continue;
                    }
                    // Plan 043 M5: the RHS of a Dot is a FIELD/METHOD name —
                    // suppress struct-literal widening so `cell.Text { }` keeps
                    // the `{` for the element's props instead of parsing
                    // `Text{...}` as a Node (see in_dot_rhs). Only Dot (not
                    // Asn/other infix ops) sets the flag, so `x = Type{...}`
                    // still widens.
                    let saved_dot_rhs = self.in_dot_rhs;
                    if matches!(op, Op::Dot) {
                        self.in_dot_rhs = true;
                    }
                    // PLAN-778 R1（F778-R1-02）：Err 平衡恢复（同类泄漏位点）。
                    let rhs_res = self.expr_pratt(power.r);
                    self.in_dot_rhs = saved_dot_rhs;
                    let rhs = rhs_res?;
                    match op {
                        Op::Range => {
                            lhs = Expr::Range(Range {
                                start: Box::new(lhs),
                                end: Box::new(rhs),
                                eq: false,
                            });
                        }
                        Op::RangeEq => {
                            lhs = Expr::Range(Range {
                                start: Box::new(lhs),
                                end: Box::new(rhs),
                                eq: true,
                            });
                        }
                        Op::Dot => {
                            // Dot expression: handle both field access and method calls
                            // Field access: object.field
                            // Method call: object.method(args)
                            match rhs {
                                Expr::Ident(field_name) => {
                                    // Simple field access: object.field
                                    lhs = Expr::Dot(Box::new(lhs), field_name);
                                }
                                // Plan 043: numeric tuple/element index access, e.g.
                                // `field.0`, `pair.1`. The RHS is an integer literal;
                                // render as `Expr::Dot(lhs, "<n>")` so downstream
                                // codegen emits `field.0` (valid JS/TS tuple index).
                                Expr::Int(n) => {
                                    lhs = Expr::Dot(Box::new(lhs), Name::from(n.to_string()));
                                }
                                Expr::Uint(n) => {
                                    lhs = Expr::Dot(Box::new(lhs), Name::from(n.to_string()));
                                }
                                Expr::Call(call) => {
                                    // Method call: object.method(args)
                                    // The RHS is a Call like: Call { name: Ident("push"), args: [...] }
                                    // We need to transform this into: Call { name: Dot(object, "push"), args: [...] }
                                    let method_name = match call.name.as_ref() {
                                        Expr::Ident(name) => name.clone(),
                                        _ => {
                                            let message = format!(
                                                "Method name must be an identifier, got {}",
                                                call.name
                                            );
                                            let span = pos_to_span(self.cur.pos);
                                            return Err(
                                                SyntaxError::Generic { message, span }.into()
                                            );
                                        }
                                    };
                                    lhs = Expr::Call(Call {
                                        name: Box::new(Expr::Dot(Box::new(lhs), method_name)),
                                        args: call.args,
                                        ret: call.ret,
                                        type_args: Vec::new(), // Plan 061: No type args for method calls yet
                                        generic_args: Vec::new(), // Plan 395: explicit turbofish args
                                        pos: call.pos,
                                    });
                                }
                                _ => {
                                    // Error: right-hand side of dot must be an identifier or method call
                                    let message = format!("Invalid field name after dot: {}", rhs);
                                    let span = pos_to_span(self.cur.pos);
                                    return Err(SyntaxError::Generic { message, span }.into());
                                }
                            }
                        }
                        Op::QuestionQuestion => {
                            // Null-coalescing operator: left ?? right
                            lhs = Expr::NullCoalesce(Box::new(lhs), Box::new(rhs));
                        }
                        _ => {
                            lhs = Expr::Bina(Box::new(lhs), op, Box::new(rhs));
                        }
                    }
                }
            }
        }
        Ok(lhs)
    }

    fn check_asn(&mut self, lhs: &Expr) -> AutoResult<()> {
        match lhs {
            Expr::Ident(name) => {
                let meta = self.lookup_meta(name.as_str());
                if let Some(Meta::Store(store)) = meta.as_deref() {
                    if matches!(store.kind, StoreKind::Let) {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "Syntax error: Assignment not allowed for let store: {}",
                                store.name
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn op(&mut self) -> Op {
        match self.kind() {
            TokenKind::Add => Op::Add,
            TokenKind::Sub => Op::Sub,
            TokenKind::Star => Op::Mul,
            TokenKind::Div => Op::Div,
            TokenKind::Mod => Op::Mod,
            TokenKind::AddEq => Op::AddEq,
            TokenKind::SubEq => Op::SubEq,
            TokenKind::MulEq => Op::MulEq,
            TokenKind::DivEq => Op::DivEq,
            TokenKind::ModEq => Op::ModEq,
            TokenKind::LSquare => Op::LSquare,
            TokenKind::LParen => Op::LParen,
            TokenKind::LBrace => Op::LBrace,
            TokenKind::Not => Op::Not,
            TokenKind::Asn => Op::Asn,
            TokenKind::Eq => Op::Eq,
            TokenKind::Neq => Op::Neq,
            TokenKind::Lt => Op::Lt,
            TokenKind::Gt => Op::Gt,
            TokenKind::Le => Op::Le,
            TokenKind::Ge => Op::Ge,
            TokenKind::Range => Op::Range,
            TokenKind::RangeEq => Op::RangeEq,
            TokenKind::Dot => Op::Dot,
            TokenKind::Colon => Op::Colon,
            TokenKind::DotView => Op::DotView,
            TokenKind::DotMut => Op::DotMut,
            TokenKind::DotMove => Op::DotMove,
            TokenKind::DotTake => Op::DotTake,
            TokenKind::QuestionQuestion => Op::QuestionQuestion,
            TokenKind::DotQuestion => Op::DotQuestion,
            TokenKind::DotQuest => Op::DotQuest, // Plan 120: .? error propagation
            TokenKind::And => Op::And,
            TokenKind::Or => Op::Or,
            _ => {
                // This should never happen if called from correct match branches
                // Return a default operator to avoid panic
                Op::Add
            }
        }
    }

    pub fn group(&mut self) -> AutoResult<Expr> {
        // Plan 419 §8.3:括号递归咽喉挂分段栈(见 parse 注释)。
        stacker::maybe_grow(512 * 1024, 1024 * 1024, || {
            self.next(); // skip (
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen)?; // skip )
            Ok(expr)
        })
    }

    pub fn sep_array(&mut self) -> AutoResult<()> {
        let mut has_sep = false;
        while self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            has_sep = true;
            self.next();
        }
        if self.is_kind(TokenKind::RSquare) {
            return Ok(());
        }
        if !has_sep {
            let message = format!("Expected array separator, got {:?}", self.kind());
            let span = pos_to_span(self.cur.pos);
            return Err(SyntaxError::Generic { message, span }.into());
        }
        Ok(())
    }

    pub fn array(&mut self) -> AutoResult<Expr> {
        self.expect(TokenKind::LSquare)?;
        self.skip_empty_lines();
        let mut elems = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RSquare) {
            elems.push(self.rhs_expr()?);
            self.sep_array()?;
        }
        self.expect(TokenKind::RSquare)?; // skip ]
        Ok(Expr::Array(elems))
    }

    pub fn sep_args(&mut self) -> AutoResult<()> {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            return Ok(());
        }
        if self.is_kind(TokenKind::RParen) {
            return Ok(());
        }
        let pos = self.cur.pos;
        Err(SyntaxError::UnexpectedToken {
            expected: "argument separator (comma, newline, or ))".to_string(),
            found: format!("{:?}", self.kind()),
            span: pos_to_span(pos),
        }
        .into())
    }

    pub fn args(&mut self) -> AutoResult<Args> {
        self.expect(TokenKind::LParen)?;
        let mut args = Args::new();
        self.skip_empty_lines();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RParen) {
            let expr = if self.is_kind(TokenKind::Ident) {
                let e = self.node_or_call_expr()?;
                e
            // } else {
            // let name = Expr::Ident(self.cur.text.clone());
            // self.next();
            // if self.is_kind(TokenKind::Comma) {
            //     name
            // } else {
            //     self.expr_pratt_with_left(name, 0)?
            // }
            } else {
                self.parse_expr()?
            };
            match &expr {
                // Named args
                Expr::Pair(p) => {
                    let k = p.key.clone();
                    match k {
                        Key::NamedKey(name) => {
                            args.args.push(Arg::Pair(name.clone(), (*p.value).clone()));
                            // args.map.push((name, *p.value));
                        }
                        _ => {
                            return Err(SyntaxError::Generic {
                                message: format!(
                                    "named args should have named key instead of {}",
                                    &k
                                ),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }
                    }
                }
                // Positional args
                Expr::Ident(name) => {
                    // name arg without value
                    let name = name.clone();
                    // Under skip_check (DSL lenient-parsing contexts such as
                    // widget/store `computed` blocks) unknown bare identifiers
                    // are kept as positional expressions — the Name-as-string
                    // fallback is itself a symbol-resolution decision, and the
                    // code generator resolves props/state names later.
                    if self.skip_check {
                        args.args.push(Arg::Pos(expr.clone()));
                    } else {
                        match self.lookup_meta(&name) {
                            Some(_) => {
                                args.args.push(Arg::Pos(expr.clone()));
                            }
                            None => {
                                args.args.push(Arg::Name(name));
                            }
                        }
                    }
                }
                _ => {
                    args.args.push(Arg::Pos(expr.clone()));
                }
            }
            self.sep_args()?;
            self.skip_empty_lines();
        }
        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    pub fn object(&mut self) -> AutoResult<Vec<Pair>> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut entries = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            entries.push(self.pair()?);
            self.sep_obj()?;
        }
        self.expect(TokenKind::RBrace)?;
        Ok(entries)
    }

    pub fn pair_expr(&mut self) -> AutoResult<Expr> {
        self.rhs_expr()
        // let exp = self.expr_pratt(0)?;
        // if let Expr::Ident(ident) = &exp {
        //     if !self.exists(ident) {
        //         return Ok(Expr::Str(ident.clone()));
        //     }
        // }
        // Ok(exp)
    }

    pub fn pair(&mut self) -> AutoResult<Pair> {
        let key = self.key()?;
        self.expect(TokenKind::Colon)?;
        let value = self.pair_expr()?;
        Ok(Pair {
            key,
            value: Box::new(value),
        })
    }

    #[allow(unused)]
    pub fn is_key(expr: &Expr) -> bool {
        match expr {
            Expr::Ident(_) => true,
            Expr::Int(_) => true,
            Expr::Bool(_) => true,
            _ => false,
        }
    }

    pub fn sep_obj(&mut self) -> AutoResult<()> {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            self.skip_empty_lines();
            return Ok(());
        }
        if self.is_kind(TokenKind::RBrace) {
            return Ok(());
        }
        let pos = self.cur.pos;
        Err(SyntaxError::UnexpectedToken {
            expected: "pair separator (comma, newline, or })".to_string(),
            found: format!("{:?}", self.kind()),
            span: pos_to_span(pos),
        }
        .into())
    }

    pub fn ident_name(&mut self) -> AutoResult<Name> {
        Ok(self.cur.text.clone())
    }

    pub fn ident(&mut self) -> AutoResult<Expr> {
        let name = self.cur.text.clone();

        // Plan 120: Check for Option and Result constructors
        match name.as_str() {
            "Some" => {
                self.next(); // consume 'Some'
                self.expect(TokenKind::LParen)?;
                let value = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expr::Some(Box::new(value)));
            }
            "None" => {
                self.next(); // consume 'None'
                return Ok(Expr::None);
            }
            "Ok" => {
                self.next(); // consume 'Ok'
                self.expect(TokenKind::LParen)?;
                let value = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expr::Ok(Box::new(value)));
            }
            "Err" => {
                self.next(); // consume 'Err'
                self.expect(TokenKind::LParen)?;
                let msg = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expr::Err(Box::new(msg)));
            }
            _ => {}
        }

        // Regular identifier
        Ok(Expr::Ident(name))
    }

    pub fn parse_ident(&mut self) -> AutoResult<Expr> {
        let name = self.cur.text.clone();
        self.next();
        Ok(Expr::Ident(name))
    }

    /// 2026-08-22(tag 上下文化):语句位置的 `tag` 仅在后随 Ident(枚举名,
    /// 如 `tag Shape { ... }` / `tag Kind u16`)时才是(废弃别名的)enum
    /// 声明;`tag: value` 属性对、`tag.field`、`tag(x)`、`tag = ...` 等
    /// 形态一律按普通标识符走表达式路径(此前无差别进 enum_stmt,节点体
    /// 里 `Cell{ tag: "x" }` 的 tag 键即被吞成 enum 名)。
    pub(super) fn tag_starts_enum_decl(&mut self) -> bool {
        if let Ok(tok) = self.lexer.next() {
            let is_ident = tok.kind == TokenKind::Ident;
            self.lexer.push_token(tok);
            is_ident
        } else {
            false
        }
    }

    pub fn parse_name(&mut self) -> AutoResult<Name> {
        let name = self.cur.text.clone();
        self.next();
        Ok(name)
    }

    /// Plan 368 FU-1: parse a function name that may be a dotted
    /// `Type.method` form (used by stdlib `.at` files to declare methods on
    /// an existing type externally, e.g. `pub fn JsonValue.as_int(self JsonValue) int;`).
    ///
    /// Returns `(method_name, Some(parent_type))` when the name is dotted,
    /// else `(name, None)`. The caller feeds `parent_type` into the existing
    /// `parent_name` machinery (same path `ext Type { fn ... }` blocks use),
    /// which auto-defines `self` and sets `Fn.parent`. This does NOT collide
    /// with the `fn.c`/`fn.vm` backward-compat path (parser.rs ~7020), which
    /// fires only when the token right after `fn` is a `Dot` — here `parse_name`
    /// has already consumed the leading `Type` identifier before we see the dot.
    pub fn parse_fn_name(&mut self) -> AutoResult<(Name, Option<Name>)> {
        let first = self.parse_name()?;
        if self.is_kind(TokenKind::Dot) {
            self.next(); // skip '.'
            let method = self.parse_name()?;
            Ok((method, Some(first)))
        } else {
            Ok((first, None))
        }
    }

    pub fn parse_ints(&mut self) -> AutoResult<Expr> {
        let res = match self.cur.kind {
            TokenKind::Int => self.parse_int(),
            TokenKind::Uint => self.parse_uint(),
            TokenKind::U8 => self.parse_u8(),
            TokenKind::I8 => self.parse_i8(),
            _ => {
                let message = format!("Expected integer, got {:?}", self.kind());
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
        };
        if res.is_ok() {
            self.next();
        }
        res
    }

    pub fn parse_int(&mut self) -> AutoResult<Expr> {
        if self.cur.text.starts_with("0x") {
            // trim 0x
            let trim = &self.cur.text[2..];
            // Try parsing as i64 first, fall back to u64 if it overflows
            match i64::from_str_radix(trim, 16) {
                Ok(val) => {
                    if val > i32::MAX as i64 {
                        Ok(Expr::I64(val))
                    } else {
                        Ok(Expr::Int(val as i32))
                    }
                }
                Err(_) => {
                    // Value too large for i64, parse as u64
                    let val = u64::from_str_radix(trim, 16).unwrap();
                    Ok(Expr::U64(val))
                }
            }
        } else if self.cur.text.starts_with("0b") {
            // Plan 178: binary literal (0b prefix)
            let trim = &self.cur.text[2..];
            let val = i64::from_str_radix(trim, 2).unwrap();
            if val > i32::MAX as i64 {
                Ok(Expr::I64(val))
            } else {
                Ok(Expr::Int(val as i32))
            }
        } else {
            // Try parsing as i64 first, fall back to u64 if it overflows
            let text = self.cur.text.as_str();
            match text.parse::<i64>() {
                Ok(val) => {
                    if val > i32::MAX as i64 {
                        Ok(Expr::I64(val))
                    } else {
                        Ok(Expr::Int(val as i32))
                    }
                }
                Err(_) => {
                    // Value too large for i64 (e.g., u64::MAX), parse as u64
                    let val = text.parse::<u64>().unwrap();
                    Ok(Expr::U64(val))
                }
            }
        }
    }

    fn parse_uint(&mut self) -> AutoResult<Expr> {
        if self.cur.text.starts_with("0x") {
            // trim 0x
            let trim = &self.cur.text[2..];
            // Try parsing as u32 first, fall back to u64 if it overflows
            match u32::from_str_radix(trim, 16) {
                Ok(val) => Ok(Expr::Uint(val)),
                Err(_) => {
                    // Value too large for u32, parse as u64
                    let val = u64::from_str_radix(trim, 16).unwrap();
                    Ok(Expr::U64(val))
                }
            }
        } else if self.cur.text.starts_with("0b") {
            let trim = &self.cur.text[2..];
            let val = u64::from_str_radix(trim, 2).unwrap();
            if val > u32::MAX as u64 {
                Ok(Expr::U64(val))
            } else {
                Ok(Expr::Uint(val as u32))
            }
        } else {
            // Try parsing as u32 first, fall back to u64 if it overflows
            let text = self.cur.text.as_str();
            match text.parse::<u32>() {
                Ok(val) => Ok(Expr::Uint(val)),
                Err(_) => {
                    // Value too large for u32, parse as u64
                    let val = text.parse::<u64>().unwrap();
                    Ok(Expr::U64(val))
                }
            }
        }
    }

    fn parse_u8(&mut self) -> AutoResult<Expr> {
        if self.cur.text.starts_with("0x") {
            let trim = &self.cur.text[2..];
            let val = u8::from_str_radix(trim, 16).unwrap();
            Ok(Expr::U8(val as u8))
        } else if self.cur.text.starts_with("0b") {
            let trim = &self.cur.text[2..];
            let val = u8::from_str_radix(trim, 2).unwrap();
            Ok(Expr::U8(val))
        } else {
            let val = self.cur.text.as_str().parse::<u8>().unwrap();
            Ok(Expr::U8(val as u8))
        }
    }

    #[allow(dead_code)]
    fn parse_u64(&mut self) -> AutoResult<Expr> {
        if self.cur.text.starts_with("0x") {
            let trim = &self.cur.text[2..];
            let val = u64::from_str_radix(trim, 16).unwrap();
            Ok(Expr::U64(val))
        } else if self.cur.text.starts_with("0b") {
            let trim = &self.cur.text[2..];
            let val = u64::from_str_radix(trim, 2).unwrap();
            Ok(Expr::U64(val))
        } else {
            let val = self.cur.text.as_str().parse::<u64>().unwrap();
            Ok(Expr::U64(val))
        }
    }

    fn parse_i8(&mut self) -> AutoResult<Expr> {
        if self.cur.text.starts_with("0x") {
            let trim = &self.cur.text[2..];
            let val = i8::from_str_radix(trim, 16).unwrap();
            Ok(Expr::I8(val as i8))
        } else if self.cur.text.starts_with("0b") {
            let trim = &self.cur.text[2..];
            let val = i8::from_str_radix(trim, 2).unwrap();
            Ok(Expr::I8(val))
        } else {
            let val = self.cur.text.as_str().parse::<i8>().unwrap();
            Ok(Expr::I8(val as i8))
        }
    }

    pub fn is_literal(&mut self) -> bool {
        self.is_kind(TokenKind::Uint)
            || self.is_kind(TokenKind::Int)
            || self.is_kind(TokenKind::U8)
            || self.is_kind(TokenKind::I8)
            || self.is_kind(TokenKind::Float)
            || self.is_kind(TokenKind::Double)
            || self.is_kind(TokenKind::True)
            || self.is_kind(TokenKind::False)
            || self.is_kind(TokenKind::Nil)
            || self.is_kind(TokenKind::Str)
            || self.is_kind(TokenKind::CStr)
            || self.is_kind(TokenKind::FStrStart)
            || self.is_kind(TokenKind::Char)
    }

    pub fn literal(&mut self) -> AutoResult<Expr> {
        let e = match self.kind() {
            TokenKind::FStrStart => self.fstr(),
            TokenKind::Uint => self.parse_int(),
            TokenKind::Int => self.parse_int(),
            TokenKind::U8 => self.parse_u8(),
            TokenKind::I8 => self.parse_i8(),
            TokenKind::Float => self.parse_float(),
            TokenKind::Double => self.parse_double(),
            TokenKind::True => Ok(Expr::Bool(true)),
            TokenKind::False => Ok(Expr::Bool(false)),
            TokenKind::Nil => {
                // Plan 550 T09: 'nil' 拼写退役为 null 的 deprecated 别名
                // （语义不变——运行期同落 PUSH_NIL/encode_null）。
                // Plan 550 T10: 裸 null 家族字面量是脚本内容三信号之一。
                self.saw_bare_null = true;
                let span = pos_to_span(self.cur.pos);
                self.warn(Warning::DeprecatedFeature {
                    name: "nil".to_string(),
                    message: "use 'null' instead".to_string(),
                    span,
                });
                Ok(Expr::Nil)
            }
            TokenKind::Str => self.parse_str(),
            TokenKind::CStr => Ok(Expr::CStr(self.cur.text.clone())),
            TokenKind::Char => Ok(Expr::Char(self.cur.text.chars().nth(0).unwrap())),
            _ => Err(format!("UnexpectedToken {}", self.cur).into()),
        };
        self.next();
        e
    }

    fn parse_float(&mut self) -> AutoResult<Expr> {
        Ok(Expr::Float(
            self.cur.text.as_str().parse().unwrap(),
            self.cur.text.clone(),
        ))
    }

    fn parse_double(&mut self) -> AutoResult<Expr> {
        Ok(Expr::Double(
            self.cur.text.as_str().parse().unwrap(),
            self.cur.text.clone(),
        ))
    }

    fn parse_str(&mut self) -> AutoResult<Expr> {
        Ok(Expr::Str(self.cur.text.clone()))
    }

    pub fn atom(&mut self) -> AutoResult<Expr> {
        // Plan 419 §8.3:原子表达式递归咽喉挂分段栈(见 parse 注释)。
        stacker::maybe_grow(512 * 1024, 1024 * 1024, || self.atom_inner())
    }

    fn atom_inner(&mut self) -> AutoResult<Expr> {
        if self.is_kind(TokenKind::LParen) {
            return self.group();
        }

        // Plan 317: Anonymous function expression: fn(params) ret_type { body }
        // Used as method args, e.g. notes.filter(fn(n Note) bool { n.id > 0 }).
        if self.is_kind(TokenKind::Fn) {
            self.next(); // consume 'fn'
                         // Parse params: (name Type, name Type, ...) or ()
            self.expect(TokenKind::LParen)?;
            let mut params = Vec::new();
            if !self.is_kind(TokenKind::RParen) {
                loop {
                    let pname = self.parse_name()?;
                    let pty = if self.is_type_name() {
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    params.push(crate::ast::ClosureParam::new(pname, pty));
                    if !self.is_kind(TokenKind::Comma) {
                        break;
                    }
                    self.next();
                }
            }
            self.expect(TokenKind::RParen)?;
            // Optional return type
            if self.is_type_name() {
                let _ = self.parse_type()?;
            }
            // Plan 390 §15 Phase H (L5): Register all parameters in scope so the
            // body can reference them. The other two closure parse paths
            // (`x => ...` at ~1999, `(a,b) => ...` at ~3842) already do this;
            // this `fn(params){}` path omitted it, so `fn(ev str){ ev }` reported
            // "Variable 'ev' is not defined in this scope". Capture of external
            // variables already works in a2r + VM — this only fixes param binding.
            for param in &params {
                self.infer_ctx.bind_var(
                    crate::ast::Name::from(param.name.as_str()),
                    param.ty.clone().unwrap_or(crate::ast::Type::Unknown),
                );
            }
            // Body
            self.skip_empty_lines();
            let body = if self.is_kind(TokenKind::LBrace) {
                Expr::Block(self.body()?)
            } else {
                self.parse_expr()?
            };
            return Ok(Expr::Closure(crate::ast::Closure::new(params, None, body)));
        }

        // Handle generic type instances specially (e.g., List<int, Heap>)
        // This is checked before the match statement since we need to consume
        // the identifier and potentially parse type parameters
        //
        // Plan 356 follow-up #2: also accept soft-keyword tokens here (e.g.
        // `tag` → TokenKind::Tag). Such words are legitimate identifiers in
        // expression position (a loop variable used in an `if` condition like
        // `if .active_tag == tag`), but the lexer reserves them as keyword
        // tokens. Without this, atom() rejected them → "Expected term".
        if self.is_kind(TokenKind::Ident) || self.cur_is_soft_ident() {
            let name = self.cur.text.clone();
            if name == "Arc" || name == "Box" {}
            self.next(); // consume the identifier

            // Check if this is a generic type instance (followed by <)
            // Only treat as generic type if the identifier is a known TYPE (not a variable)
            // This prevents false positives like "x < 10" being treated as generic type
            // Plan 087: Also check type_registry for REPL support
            // Plan 091: Use wrapper method
            let mut is_type = self.lookup_ident_type(&name).is_some();
            if !is_type {
                // Check type registry for REPL (Plan 087)
                if let Some(ref registry) = self.type_registry {
                    is_type = registry.borrow().is_type(&name);
                }
            }
            // Plan 317: Built-in container types (List, Array, Map) are always
            // recognized as types so that List<Note>.new(...) parses as a
            // GenName generic instance rather than "List < Note" comparison.
            if !is_type && matches!(name.as_str(), "List" | "Array" | "Map") {
                is_type = true;
            }

            // Check for generic type instance: Identifier<Type, ...>
            if self.is_kind(TokenKind::Lt) && is_type {
                // Parse as generic type instance: List<int, Heap>
                self.expect(TokenKind::Lt)?;
                let mut args = Vec::new();
                args.push(self.parse_type()?);

                while self.cur.kind == TokenKind::Comma {
                    self.next(); // Consume ','
                    args.push(self.parse_type()?);
                }

                self.expect(TokenKind::Gt)?;

                // Generate descriptive name: "List<int, Heap>"
                let args_str = args
                    .iter()
                    .map(|t| t.unique_name().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let generic_name = format!("{}<{}>", name, args_str);

                return Ok(Expr::GenName(generic_name.into()));
            }

            // Check for node instance: Identifier { ... }
            // This handles type construction syntax like Pair {x: 1, y: 2}
            // Plan 043 M5: also accept this form for PascalCase identifiers
            // that are NOT known variables, even when the type isn't registered
            // (e.g. an imported type whose module file wasn't resolvable). This
            // matches the existing usage in 013-todo/015-notes, where imported
            // types like `Todo`/`Note` are constructed with `Note { ... }`. The
            // `is_type` gate alone failed for unresolved imports because
            // `lookup_ident_type` returns None when the type_decl isn't found.
            // Lowercase idents stay identifiers (never struct construction) to
            // avoid clashing with a `name { ... }` block expression.
            let accepts_as_type_construction = is_type
                || {
                    // Plan 043 M5: only widen for the UI scenario (store/widget
                    // model fields, where `Type{...}` construction is idiomatic
                    // and imported types may not be registered). Other dialects
                    // (notably gdscript) reuse atom() and their `Ident {` forms
                    // must not be reinterpreted as struct construction — doing so
                    // overflowed the parser stack on godot scene files.
                    self.is_ui_scenario()
                        && !self.in_dot_rhs
                        && name
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                        && !matches!(self.lookup_meta(&name), Some(m) if matches!(m.as_ref(), Meta::Store(_) | Meta::Ref(_)))
                };
            if self.is_kind(TokenKind::LBrace) && accepts_as_type_construction {
                if !self.is_ui_scenario() {
                    // Non-UI dialects (notably gdscript) reuse atom() and their
                    // `Ident {` forms must keep the OLD path — empty args, then
                    // parse_node reads the braces as a body. Re-parsing the
                    // braces as object() pairs here overflowed the parser stack
                    // on godot scene files (dodge_player).
                    let _ident = Expr::Ident(name.clone());
                    return Ok(Expr::Node(self.parse_node(
                        &name,
                        None,
                        Args::new(),
                        &AutoStr::new(),
                    )?));
                }
                // Struct literal `Type{ field: value, ... }` (UI scenario) —
                // parse the braces as NAMED ARGS (object() returns the Pair
                // list), NOT as a node body. Plan 043 M5: parse_node's `{...}`
                // → parse_node_body path can't parse `field: value` as
                // statements ("Expected term, got RBrace" — the error is
                // collected and the fields silently dropped), so codegen
                // received an empty-args Node and emitted `{}` — e.g.
                // `Block{ id: id, command: cmd }` became `let block = {}` and
                // command results never matched the block.
                let pairs = self.object()?;
                let mut args = Args::new();
                for p in pairs {
                    match p.key {
                        Key::NamedKey(name) => {
                            args.args.push(Arg::Pair(name, (*p.value).clone()));
                        }
                        _ => {
                            // Non-named keys can't be struct fields — keep the
                            // pair as a positional expression.
                            args.args.push(Arg::Pos(Expr::Pair(p)));
                        }
                    }
                }

                return Ok(Expr::Node(self.parse_node(
                    &name,
                    None,
                    args,
                    &AutoStr::new(),
                )?));
            }

            // Not a special type expression, just a regular identifier
            // Plan 6B-4.14: Smart pointer constructors Box(expr) and Arc(expr)
            match name.as_str() {
                "Box" => {
                    if self.is_kind(TokenKind::LParen) {
                        self.next(); // consume '('
                        let value = self.parse_expr()?;
                        self.expect(TokenKind::RParen)?;
                        return Ok(Expr::BoxExpr(Box::new(value)));
                    }
                }
                "Arc" => {
                    if self.is_kind(TokenKind::LParen) {
                        self.next(); // consume '('
                        let value = self.parse_expr()?;
                        self.expect(TokenKind::RParen)?;
                        return Ok(Expr::ArcExpr(Box::new(value)));
                    }
                }
                _ => {}
            }
            return Ok(Expr::Ident(name));
        }

        let expr = match self.kind() {
            TokenKind::Uint => self.parse_uint()?,
            TokenKind::Int => self.parse_int()?,
            TokenKind::U8 => self.parse_u8()?,
            TokenKind::I8 => self.parse_i8()?,
            TokenKind::Float => self.parse_float()?,
            TokenKind::Double => self.parse_double()?,
            TokenKind::Str => self.parse_str()?,
            TokenKind::True => Expr::Bool(true),
            TokenKind::False => Expr::Bool(false),
            TokenKind::CStr => Expr::CStr(self.cur.text.clone()),
            TokenKind::Char => Expr::Char(self.cur.text.chars().nth(0).unwrap()),
            TokenKind::Ident => self.ident()?,
            // Plan 121: Treat 'spawn' as identifier in expressions (e.g., Task.spawn())
            TokenKind::Spawn => Expr::Ident(self.cur.text.clone()),
            // Allow @ and * as special identifiers for pointer operations
            TokenKind::At => Expr::Ident("@".into()),
            TokenKind::Star => Expr::Ident("*".into()),
            TokenKind::Nil => {
                // Plan 550 T09: 'nil' deprecated 警告（语义不变，见 literal 臂注记）。
                self.saw_bare_null = true; // Plan 550 T10: 脚本内容三信号
                let span = pos_to_span(self.cur.pos);
                self.warn(Warning::DeprecatedFeature {
                    name: "nil".to_string(),
                    message: "use 'null' instead".to_string(),
                    span,
                });
                Expr::Nil
            }
            TokenKind::Null => {
                self.saw_bare_null = true; // Plan 550 T10: 脚本内容三信号
                Expr::Null
            }
            // Plan 120: Option and Result constructors
            TokenKind::NoneKW => Expr::None,
            TokenKind::SomeKW => {
                // Some(value) - expect parentheses
                self.next(); // consume 'Some'
                if self.is_kind(TokenKind::LParen) {
                    self.next(); // consume '('
                    let value = self.parse_expr()?;
                    if !self.is_kind(TokenKind::RParen) {
                        let span = pos_to_span(self.cur.pos);
                        return Err(SyntaxError::UnexpectedToken {
                            expected: ")".to_string(),
                            found: format!("{}", self.cur.text),
                            span,
                        }
                        .into());
                    }
                    self.next(); // consume ')'
                                 // Return early - don't let the final self.next() run
                    return Ok(Expr::Some(Box::new(value)));
                } else {
                    // `Some` without parens - treat as identifier (variable reference)
                    // Return early since we already consumed 'Some'
                    return Ok(Expr::Ident("Some".into()));
                }
            }
            TokenKind::OkKW => {
                // Ok(value) - expect parentheses
                self.next(); // consume 'Ok'
                if self.is_kind(TokenKind::LParen) {
                    self.next(); // consume '('
                    let value = self.parse_expr()?;
                    if !self.is_kind(TokenKind::RParen) {
                        let span = pos_to_span(self.cur.pos);
                        return Err(SyntaxError::UnexpectedToken {
                            expected: ")".to_string(),
                            found: format!("{}", self.cur.text),
                            span,
                        }
                        .into());
                    }
                    self.next(); // consume ')'
                    return Ok(Expr::Ok(Box::new(value)));
                } else {
                    return Ok(Expr::Ident("Ok".into()));
                }
            }
            TokenKind::ErrKW => {
                // Err(message) - expect parentheses
                self.next(); // consume 'Err'
                if self.is_kind(TokenKind::LParen) {
                    self.next(); // consume '('
                    let msg = self.parse_expr()?;
                    if !self.is_kind(TokenKind::RParen) {
                        let span = pos_to_span(self.cur.pos);
                        return Err(SyntaxError::UnexpectedToken {
                            expected: ")".to_string(),
                            found: format!("{}", self.cur.text),
                            span,
                        }
                        .into());
                    }
                    self.next(); // consume ')'
                    return Ok(Expr::Err(Box::new(msg)));
                } else {
                    return Ok(Expr::Ident("Err".into()));
                }
            }
            // Allow 'type' keyword as identifier in certain contexts (e.g., expr.type)
            TokenKind::Type => Expr::Ident(self.cur.text.clone()),
            // Plan 012 P2: 'link'/'task' are contextual keywords — usable as
            // identifiers in expressions (loop vars, locals named `link`/
            // `task`, jade gaps 18/29). Element/keyword positions (view
            // router-link, task decls) intercept the token before atom()
            // runs, so this only affects expression positions.
            TokenKind::Link | TokenKind::Task => Expr::Ident(self.cur.text.clone()),
            // Plan 124: Async block: ~{ stmts }
            TokenKind::Tilde => {
                self.next(); // consume '~'
                             // Expect a block { ... }
                if !self.is_kind(TokenKind::LBrace) {
                    let span = pos_to_span(self.cur.pos);
                    return Err(SyntaxError::UnexpectedToken {
                        expected: "{".to_string(),
                        found: format!("{}", self.cur.text),
                        span,
                    }
                    .into());
                }
                let body = self.body()?;
                // Return early since we already consumed '~' and the block
                return Ok(Expr::AsyncBlock {
                    body,
                    return_type: None, // Will be inferred later
                });
            }
            _ => {
                return Err(SyntaxError::Generic {
                    message: format!("Expected term, got {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        };

        self.next();
        Ok(expr)
    }

    /// 解析fstr
    /// fstr: [FStrSart, (FStrParts | ${Expr} | $Ident)*, FStrEnd]
    pub fn fstr(&mut self) -> AutoResult<Expr> {
        // skip fstrs (e.g. ` or f")
        self.expect(TokenKind::FStrStart)?;

        // parse fstr parts or interpolated exprs
        let mut parts = Vec::new();
        while !(self.is_kind(TokenKind::FStrEnd) || self.is_kind(TokenKind::EOF)) {
            // $ expressions
            if self.is_kind(TokenKind::FStrNote) {
                self.next(); // skip $
                if self.is_kind(TokenKind::LBrace) {
                    // ${Expr}
                    self.expect(TokenKind::LBrace)?;
                    // NOTE: allow only one expression in a ${} block
                    if !self.is_kind(TokenKind::RBrace) {
                        let expr = self.rhs_expr()?;
                        parts.push(expr);
                        // self.expect_eos()?;
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    // $Ident or $Ident.field.chain
                    let ident_expr = self.ident()?;
                    self.next(); // consume the Ident token
                    let mut expr = ident_expr;
                    // Check for field access chain in f-string: $turn_result.iterations
                    // The .field part may be embedded in the next fstr_part text token
                    while self.is_kind(TokenKind::Dot) {
                        self.next();
                        let field_name = self.ident_name()?;
                        self.next();
                        expr = Expr::Dot(Box::new(expr), field_name);
                    }
                    // Also check if next token is an fstr_part starting with "."
                    if self.is_kind(TokenKind::FStrPart) {
                        let text = self.cur.text.clone();
                        if text.starts_with('.') {
                            let rest = &text[1..];
                            let field_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_');
                            let (field_name, remaining) = if let Some(pos) = field_end {
                                (&rest[..pos], &rest[pos..])
                            } else {
                                (rest, "")
                            };
                            if !field_name.is_empty()
                                && field_name
                                    .chars()
                                    .next()
                                    .map_or(false, |c| c.is_alphabetic() || c == '_')
                            {
                                expr = Expr::Dot(Box::new(expr), field_name.into());
                                if remaining.is_empty() {
                                    self.next();
                                } else {
                                    let new_text: AutoStr = remaining.into();
                                    self.cur = Token::fstr_part(self.cur.pos, new_text);
                                }
                            }
                        }
                    }
                    parts.push(expr);
                }
            } else {
                // normal text parts
                let text = self.cur.text.clone();
                parts.push(Expr::Str(text));
                self.next();
            }
        }
        self.expect(TokenKind::FStrEnd)?;
        Ok(Expr::FStr(FStr::new(parts)))
    }

    pub fn if_expr(&mut self) -> AutoResult<Expr> {
        let (branches, else_stmt) = self.if_contents()?;
        Ok(Expr::If(If {
            branches,
            else_: else_stmt,
        }))
    }

    pub fn cond_expr(&mut self) -> AutoResult<Expr> {
        self.rhs_expr()
    }

    // An Expression that can be assigned to a variable, e.g. right-hand side of an assignment
    pub fn rhs_expr(&mut self) -> AutoResult<Expr> {
        // Support node instance syntax with primary prop in rhs position:
        // e.g., var x = ctr CanTrcvGeneral { ... }
        // e.g., var x = v CanTrcvBaudRate int { ... }  (with secondaryProp)
        // Pattern: Ident Ident { or Ident Ident ( or Ident Ident Ident {
        // Only delegate to node_or_call_expr when we see these patterns,
        // to avoid changing semantics of function calls like Point(x: 1, y: 2)
        if self.is_kind(TokenKind::Ident) {
            // Plan 375: also support a node instance in rhs position when the
            // value is a plain node call with a body block, e.g.
            //   icf: file("x") {}
            //   ddf: file(`${sdk}/y`) {}
            // Without this, `pair_expr` -> `rhs_expr` falls through to
            // `parse_expr`, parses `file("x")` as a function call, and leaves
            // the trailing `{}` — yielding "Expected end of statement, got LBrace".
            // Only delegate when we can confirm a `{` follows the node head, so
            // ordinary function calls keep their semantics.
            if self.rhs_lookahead_is_node() {
                return self.node_or_call_expr();
            }
            let saved_cur = self.cur.clone();
            self.next(); // consume first identifier
            if self.is_kind(TokenKind::Ident) {
                // "name ident" pattern — lookahead to confirm it's a node
                let next_ident = self.cur.clone();
                let after = self.lexer.next()?;
                let is_node = match after.kind {
                    TokenKind::LBrace | TokenKind::LParen => true,
                    TokenKind::Ident => {
                        // Three-ident pattern: "name ident ident" — look one more ahead
                        // e.g., v CanTrcvBaudRate int { ... }
                        let third_ident = after.clone();
                        let after2 = self.lexer.next()?;
                        let found = matches!(after2.kind, TokenKind::LBrace | TokenKind::LParen);
                        self.lexer.push_token(after2);
                        self.lexer.push_token(third_ident);
                        found
                    }
                    _ => false,
                };
                self.lexer.push_token(after);
                if is_node {
                    // Confirmed node pattern — delegate to node_or_call_expr
                    self.lexer.push_token(next_ident);
                    self.cur = saved_cur;
                    return self.node_or_call_expr();
                }
                // Not a node pattern, restore and fall through
                self.lexer.push_token(next_ident);
                self.cur = saved_cur;
            } else {
                // "name" without another ident following — restore and fall through
                self.lexer.push_token(self.cur.clone());
                self.cur = saved_cur;
            }
        }
        self.parse_expr()
    }

    /// Plan 375: lookahead helper for `rhs_expr`. Returns true when the tokens
    /// starting at the current cursor form a node *instance* (i.e. carry a body
    /// block), as opposed to a plain function call or a bare identifier.
    ///
    /// Recognized shapes:
    ///   `Ident { ... }`            — node with no args
    ///   `Ident ( ... ) { ... }`    — node call with args, then a body block
    ///
    /// It scans with the lexer's pushback buffer, balancing `()`, `[]` while
    /// skipping over the argument group, so nested calls inside the args don't
    /// fool it. Returns false for `Ident ( ... )` with no trailing `{` (a real
    /// function call) — this is what keeps `Point(x: 1)` semantics intact.
    fn rhs_lookahead_is_node(&mut self) -> bool {
        debug_assert!(self.is_kind(TokenKind::Ident));
        let mut depth = 0i32; // tracks () nesting (node args use parens only)
        let mut pushed: Vec<Token> = Vec::new();
        let mut result = false;

        // First token is the Ident (current cursor) — consume it for the scan.
        loop {
            let tok = match self.lexer.next() {
                Ok(t) => t,
                Err(_) => break,
            };
            pushed.push(tok.clone());
            match tok.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => depth -= 1,
                TokenKind::LBrace if depth == 0 => {
                    // Top-level `{` immediately after the node head / arg group.
                    result = true;
                    break;
                }
                TokenKind::RBrace | TokenKind::Semi | TokenKind::Newline | TokenKind::EOF
                    if depth == 0 =>
                {
                    // Statement separator without ever seeing a `{` — not a node.
                    break;
                }
                // The generic-args form `Ident<...>` uses Lt/Gt; we don't need to
                // special-case it — depth stays 0 and a following `{` still wins.
                _ => {}
            }
            // If depth went negative (unbalanced), bail.
            if depth < 0 {
                break;
            }
        }

        // Push everything back so the real parser re-reads from the Ident.
        for t in pushed.into_iter().rev() {
            self.lexer.push_token(t);
        }
        result
    }

    fn tag_cover(&mut self, tag_name: &Name) -> AutoResult<Expr> {
        self.expect(TokenKind::Dot)?;
        // tag field
        let tag_field = self.parse_name()?;

        // Check if there's a parentheses (for variants with values like May.val(v))
        // or no parentheses (for nil variants like May.nil)
        if self.is_kind(TokenKind::LParen) {
            self.next(); // consume (
            let mut bindings = vec![];
            if !self.is_kind(TokenKind::RParen) {
                bindings.push(self.parse_name()?);
                while self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Ident) {
                    if self.is_kind(TokenKind::Comma) {
                        self.next();
                    }
                    bindings.push(self.parse_name()?);
                }
            }
            self.expect(TokenKind::RParen)?;
            return Ok(Expr::Cover(Cover::Tag(TagCover {
                kind: tag_name.clone(),
                tag: tag_field,
                bindings,
            })));
        } else {
            // Nil variant without value binding - use underscore as placeholder
            return Ok(Expr::Cover(Cover::Tag(TagCover {
                kind: tag_name.clone(),
                tag: tag_field,
                bindings: vec![Name::from("_")],
            })));
        }
    }

    pub fn is_branch_cond_expr(&mut self) -> AutoResult<Expr> {
        // Skip symbol checking during pattern parsing — bindings like Some(x) or Type.Variant(y)
        // have not-yet-defined variables that would trigger false "undefined variable" errors.
        let prev_skip = self.skip_check;
        self.skip_check = true;

        let result = self.is_branch_cond_expr_inner();

        self.skip_check = prev_skip;
        result
    }

    fn is_branch_cond_expr_inner(&mut self) -> AutoResult<Expr> {
        // Plan 120: Check for Option/Result patterns first
        // Some(binding) => ...  or None => ...
        // Ok(binding) => ...    or Err(binding) => ...
        if self.is_kind(TokenKind::SomeKW) {
            return self.parse_option_pattern();
        }
        if self.is_kind(TokenKind::NoneKW) {
            self.next(); // consume None
            return Ok(Expr::OptionPattern(OptionCover {
                variant: OptionVariant::None,
                binding: None,
            }));
        }
        if self.is_kind(TokenKind::OkKW) {
            return self.parse_result_pattern(ResultVariant::Ok);
        }
        if self.is_kind(TokenKind::ErrKW) {
            return self.parse_result_pattern(ResultVariant::Err);
        }

        // Parse the left-hand side expression (identifier or tag)
        let lhs = if self.is_kind(TokenKind::Ident) {
            self.lhs_expr()?
        } else {
            self.atom()?
        };

        // Plan 165: Check for plain struct destructuring: Point { x, y }
        if let Expr::Ident(name) = &lhs {
            if self.is_kind(TokenKind::LBrace) {
                let name = name.clone();
                return self.parse_struct_cover(name, None);
            }
        }

        // Continue parsing to handle member access (e.g., Msg.Inc)
        let mut result = self.expr_pratt_with_left(lhs, 0)?;

        // Plan 396/B11(b): qualified struct-variant pattern WITHOUT payload
        // parens: `Type.Variant { field, ... }` (and `module.Type.Variant
        // { ... }` — strip the module segment). expr_pratt yields a Dot
        // chain; the trailing `{` starts the struct cover.
        if let Expr::Dot(base, variant) = &result {
            let tv = match base.as_ref() {
                Expr::Ident(type_name) => Some((type_name.clone(), variant.clone())),
                Expr::Dot(mod_name, sub_type) => {
                    // module.Type.Variant { .. } — ensure base is module.Ident
                    if matches!(mod_name.as_ref(), Expr::Ident(_)) {
                        Some((sub_type.clone(), variant.clone()))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some((type_name, variant)) = tv {
                if self.is_kind(TokenKind::LBrace) {
                    return self.parse_struct_cover(type_name, Some(variant));
                }
            }
        }

        // Convert Call(Dot(Ident(type_name), variant_name), [bindings]) into Cover::Tag
        // This handles enum variant patterns for imported types where lookup_type returns User instead of Enum
        // Also handles 3-level paths: Call(Dot(Dot(Ident(mod), type_name), variant_name), [bindings])
        // Also handles 3-level via Cover::Tag: Call(Dot(Cover::Tag(mod.type), variant), [bindings])
        if let Expr::Call(call) = &result {
            if let Expr::Dot(base, variant) = call.name.as_ref() {
                let type_name = match base.as_ref() {
                    Expr::Ident(name) => Some(name.clone()),
                    Expr::Dot(_mod_name, sub_type) => {
                        // 3-level: module.Type.Variant(binding)
                        // Use just the type name (sub_type), stripping module prefix
                        Some(sub_type.clone())
                    }
                    Expr::Cover(Cover::Tag(inner_tag)) => {
                        // Cover::Tag from earlier dot: module.Type -> TagCover, then .Variant(binding)
                        // Use just the tag (type name), stripping module prefix
                        Some(inner_tag.tag.clone())
                    }
                    _ => None,
                };
                if let Some(type_name) = type_name {
                    let bindings: Vec<Name> = call
                        .args
                        .args
                        .iter()
                        .map(|arg| match arg {
                            Arg::Pos(Expr::Ident(name)) => name.clone(),
                            Arg::Name(name) => name.clone(),
                            _ => Name::from("_"),
                        })
                        .collect();
                    return Ok(Expr::Cover(Cover::Tag(TagCover {
                        kind: type_name,
                        tag: variant.clone(),
                        bindings,
                    })));
                }
            }
        }

        // Plan 396 §2.5: qualified unit-variant pattern WITHOUT payload
        // parens: `module.Type.UnitVariant`. When the head is a use.rust
        // module (not a known enum), lhs_expr's tag_cover consumed
        // `module.Type` as a nil placeholder TagCover (bindings ["_"]) and
        // the trailing Dot field is the actual variant. Strip the module
        // segment — mirroring the Call conversion above — so a2r emits
        // `Type::UnitVariant` instead of invalid `module::Type.UnitVariant`
        // (ai-config loader.at: `auto_val.Value.Nil` → `Value::Nil`).
        if let Expr::Dot(base, variant) = &result {
            if let Expr::Cover(Cover::Tag(inner_tag)) = base.as_ref() {
                if inner_tag.bindings.iter().all(|b| b.as_str() == "_") {
                    let rewritten = Expr::Cover(Cover::Tag(TagCover {
                        kind: inner_tag.tag.clone(),
                        tag: variant.clone(),
                        bindings: vec![Name::from("_")],
                    }));
                    // A following `{` is a struct-variant payload — let the
                    // Plan 165 check below destructure it.
                    if !self.is_kind(TokenKind::LBrace) {
                        return Ok(rewritten);
                    }
                    result = rewritten;
                }
            }
        }

        // Plan 165: Check for enum variant struct destructuring: Msg.User { content }
        if let Expr::Cover(Cover::Tag(tag)) = &result {
            if self.is_kind(TokenKind::LBrace) {
                let type_name = tag.kind.clone();
                let variant = Some(tag.tag.clone());
                return self.parse_struct_cover(type_name, variant);
            }
        }

        Ok(result)
    }

    /// Plan 165: Parse struct destructuring pattern: Name { field1, field2: alias }
    /// Called when we see `{` after a type name in an is branch.
    fn parse_struct_cover(
        &mut self,
        type_name: AutoStr,
        variant: Option<AutoStr>,
    ) -> AutoResult<Expr> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        let mut fields = Vec::new();
        while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::EOF) {
            let field = self.parse_name()?;
            let binding = if self.is_kind(TokenKind::Colon) {
                // field: alias
                self.next(); // skip ':'
                self.parse_name()?
            } else {
                // shorthand: field name = binding name
                field.clone()
            };
            fields.push(crate::ast::cover::FieldBinding { field, binding });

            // Optional comma separator
            if self.is_kind(TokenKind::Comma) {
                self.next();
            }
            self.skip_empty_lines();
        }
        self.expect(TokenKind::RBrace)?;

        Ok(Expr::StructPattern(crate::ast::cover::StructCover {
            type_name,
            variant,
            fields,
        }))
    }

    /// Parse Option pattern: Some(binding) for is statement
    fn parse_option_pattern(&mut self) -> AutoResult<Expr> {
        self.next(); // consume 'Some'
        if self.is_kind(TokenKind::LParen) {
            self.next(); // consume '('
                         // Expect an identifier (binding variable)
            if !self.is_kind(TokenKind::Ident) {
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic {
                    message: "Pattern Some(x) expects an identifier".to_string(),
                    span,
                }
                .into());
            }
            let binding = self.parse_name()?;
            self.expect(TokenKind::RParen)?; // consume ')'

            Ok(Expr::OptionPattern(OptionCover {
                variant: OptionVariant::Some,
                binding: Some(binding),
            }))
        } else {
            // Some without parens - treat as identifier
            Ok(Expr::Ident("Some".into()))
        }
    }

    /// Parse Result pattern: Ok(binding) / Err(binding) for is statement.
    /// Plan 396/B11(b): the parenthesized inner may also be a NESTED PATTERN
    /// (e.g. `Err(ToolError.SecurityDenied { kind, path, .. })` or
    /// `Err(Enum.Variant(x))`) — parsed via is_branch_cond_expr_inner.
    fn parse_result_pattern(&mut self, variant: ResultVariant) -> AutoResult<Expr> {
        self.next(); // consume 'Ok' or 'Err'
        if self.is_kind(TokenKind::LParen) {
            self.next(); // consume '('
                         // Parse the inner via the branch-pattern parser — it handles
                         // plain idents (normalized to a binding below) AND nested
                         // patterns: Type.Variant(binds), Type { fields },
                         // module.Type.Variant { fields } (Plan 396/B11(b)).
            let prev_skip = self.skip_check;
            self.skip_check = true;
            let inner = self.is_branch_cond_expr_inner();
            self.skip_check = prev_skip;
            let inner = inner?;
            self.expect(TokenKind::RParen)?; // consume ')'
                                             // A bare nested ident is normalized to a plain binding.
            if let Expr::Ident(name) = inner {
                return Ok(Expr::ResultPattern(ResultCover {
                    variant,
                    binding: Some(name),
                    inner: None,
                }));
            }
            Ok(Expr::ResultPattern(ResultCover {
                variant,
                binding: None,
                inner: Some(Box::new(inner)),
            }))
        } else {
            // Ok/Err without parens - treat as identifier
            Ok(Expr::Ident(variant.to_string().into()))
        }
    }

    pub fn lhs_expr(&mut self) -> AutoResult<Expr> {
        if !self.is_kind(TokenKind::Ident) {
            return Err(SyntaxError::Generic {
                message: format!("Expected LHS expr with ident, got {}", self.peek().kind),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }
        let name = self.parse_name()?;

        // Check if it's a variable/parameter FIRST — before checking type names.
        // A function param named `m` of type `Binary` should be treated as an
        // identifier, not as the enum type `Binary`.
        if let Some(meta) = self.lookup_meta(&name) {
            match meta.as_ref() {
                Meta::Store(_) | Meta::Ref(_) => {
                    let mut expr = Expr::Ident(name);
                    while self.is_kind(TokenKind::Dot) {
                        self.next();
                        let field = self.parse_name()?;
                        expr = Expr::Dot(Box::new(expr), field);
                    }
                    return Ok(expr);
                }
                _ => {}
            }
        }

        // if expr is A Tag/Enum, could be a Tag Creation Expr,
        // format is: EnumName.Variant(elem)
        let typ = self.lookup_type(&name);
        let is_enum_or_tag = matches!(*typ.borrow(), Type::Enum(_) | Type::Tag(_));
        // Type::User may shadow an enum — check type_store explicitly
        let is_user_enum = matches!(*typ.borrow(), Type::User(_))
            && self
                .type_store
                .read()
                .map(|store| store.lookup_enum_decl_str(&name).is_some())
                .unwrap_or(false);
        // Also treat as enum if it's a known type followed by .Variant pattern
        // But NOT if the name is an imported module (e.g., `types` from `use types`)
        let is_imported_module = self.use_imports.iter().any(|s| s.as_str() == name.as_str());
        let is_user_with_variant = !is_imported_module
            && matches!(*typ.borrow(), Type::User(_))
            && !matches!(*typ.borrow(), Type::Unknown | Type::Void)
            && self.is_kind(TokenKind::Dot);
        if is_enum_or_tag || is_user_enum || is_user_with_variant {
            return self.tag_cover(&name);
        }

        // Support dot chains: self.field, obj.field.nested, etc.
        let mut expr = Expr::Ident(name);
        while self.is_kind(TokenKind::Dot) {
            self.next(); // skip .
            let field = self.parse_name()?;
            expr = Expr::Dot(Box::new(expr), field);
        }
        Ok(expr)
    }

    pub fn iterable_expr(&mut self) -> AutoResult<Expr> {
        // TODO: how to check for range/array but reject other cases?
        self.parse_expr()
    }

    // Plan 060: Parse JavaScript/TypeScript-style closure: ` x => body` or `(a, b) => body`
    pub fn parse_closure(&mut self) -> AutoResult<Expr> {
        use crate::ast::{Closure, ClosureParam};

        // Check if this is a single-param or multi-param closure
        let params = if self.is_kind(TokenKind::LParen) {
            // Multi-param closure: (a, b) => body or (a int, b int) => body
            // Or zero-param closure: () => body
            self.next(); // skip (

            let mut params = Vec::new();

            // Check for empty parameter list: () => body
            if self.is_kind(TokenKind::RParen) {
                self.next(); // skip )
            } else {
                loop {
                    let name = self.parse_name()?;

                    // Optional type annotation (no colon - Auto syntax: a int, b int)
                    // Same logic as fn_params: check if next token is a type
                    let ty = if self.is_type_name() {
                        Some(self.parse_type()?)
                    } else {
                        None
                    };

                    params.push(ClosureParam::new(name, ty));

                    if !self.is_kind(TokenKind::Comma) {
                        break;
                    }
                    self.next(); // skip ,
                }

                self.expect(TokenKind::RParen)?; // skip )
            }
            params
        } else {
            // Single-param closure:  x => body (no parentheses)
            let name = self.parse_name()?;
            vec![ClosureParam::new(name, None)]
        };

        // Expect =>
        self.expect(TokenKind::DoubleArrow)?;

        // Register all parameters in scope so body can reference them
        for param in &params {
            self.infer_ctx.bind_var(
                crate::ast::Name::from(param.name.as_str()),
                param.ty.clone().unwrap_or(crate::ast::Type::Unknown),
            );
        }

        // Parse body (expression or block)
        let body = if self.is_kind(TokenKind::LBrace) {
            // Block body: { stmts }
            Expr::Block(self.body()?)
        } else {
            // Expression body
            self.parse_expr()?
        };

        Ok(Expr::Closure(Closure::new(params, None, body)))
    }
    /// Parse a string literal (returns the content without quotes)
    pub(super) fn parse_string_literal(&mut self) -> AutoResult<String> {
        if !self.is_kind(TokenKind::Str) {
            return Err(SyntaxError::UnexpectedToken {
                expected: "string".to_string(),
                found: self.cur.text.to_string(),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }
        let text = self.cur.text.to_string();
        self.next();
        Ok(text)
    }

    /// Get file extensions to load based on compile destination
    /// Plan 036: Returns extensions in load order (bottom layer first, then top layer)
    #[allow(dead_code)]
    fn get_file_extensions(&self) -> Vec<&'static str> {
        match self.compile_dest {
            CompileDest::Interp => vec![".at", ".vm.at"], // Interpreter: Interface (first) → VM implementation (second)
            CompileDest::TransC => vec![".at", ".c.at"], // Transpiler: Interface (first) → C implementation (second)
            CompileDest::TransRust => vec![".at", ".rs.at"], // Rust transpiler: Interface → Rust implementation
            CompileDest::Config => vec![".at"], // Plan 364: config evaluates a single pac.at, no layered impl
        }
    }

    /// Check if a file exists at the given path
    #[allow(dead_code)]
    fn file_exists(&self, dir: &std::path::Path, name: &str, ext: &str) -> bool {
        let file_path = dir.join(format!("{}{}", name, ext));
        file_path.exists()
    }

    pub(super) fn get_int_expr(&mut self, num: &Expr) -> i64 {
        match num {
            Expr::Int(n) => *n as i64,
            Expr::Uint(n) => *n as i64,
            Expr::I8(n) => *n as i64,
            Expr::U8(n) => *n as i64,
            _ => 0,
        }
    }

    pub(super) fn get_usize(&mut self, size: &Expr) -> AutoResult<usize> {
        match size {
            Expr::Int(size) => {
                if *size <= 0 {
                    return Err(SyntaxError::Generic {
                        message: "Array size must be greater than 0".to_string(),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                } else {
                    Ok(*size as usize)
                }
            }
            Expr::Uint(size) => Ok(*size as usize),
            Expr::I8(size) => {
                if *size <= 0 {
                    return Err(SyntaxError::Generic {
                        message: "Array size must be greater than 0".to_string(),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                } else {
                    Ok(*size as usize)
                }
            }
            Expr::U8(size) => Ok(*size as usize),
            _ => {
                return Err(SyntaxError::Generic {
                    message: "Invalid array size expression, Not a Integer Number!".to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }
    }
}
