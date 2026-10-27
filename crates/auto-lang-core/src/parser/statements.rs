//! PLAN-843 结构拆分（自 parser.rs 原样搬移；算法/行为零变化）。
use super::*;

impl<'a> Parser<'a> {
    fn break_stmt(&mut self) -> AutoResult<Stmt> {
        self.next();
        Ok(Stmt::Break)
    }

    fn continue_stmt(&mut self) -> AutoResult<Stmt> {
        self.next();
        Ok(Stmt::Continue)
    }

    /// Plan 200 Task 1.1: loop { body } desugars to for ever { body }
    fn loop_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `loop`
        let body = self.body()?;
        let has_new_line = body.has_new_line;
        Ok(Stmt::For(For {
            iter: Iter::Ever,
            range: Expr::Nil,
            body,
            new_line: has_new_line,
            init: None,
        }))
    }

    /// Plan 560 T09 (E2)：`with expr { body }` / `with expr as x { body }`。
    ///
    /// 无 as：直产 `py_with(expr, () => { body })`——全复用 539 通道
    /// （host __enter__/闭包体/__exit__；异常抑制默认不做，§10 裁决）。
    /// as x（Plan 567 T12/T13，P560-D1 清偿）：`{ var __w = expr; var x =
    /// py_enter(__w); try { body } catch (__we) { py_exit(__w); py_raise(__we) }
    /// finally { py_exit(__w) } }`——块形态（绑定 + 出口保证）。绑定值 =
    /// py_enter 返回的 `__enter__` 结果；出口保证 = 正常路径走 finally，
    /// body 的 Err 值传播被 T08 值通道拦截进 catch（py_exit + py_raise
    /// 再抛外传；py_raise 报错即外传、按 Plan 012 P2 语义跳过 finally——
    /// `__exit__` 恰好一次）。
    fn with_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `with`
                     // P560-D1/567-T12: with 头部窗口内 `as` Cast 臂截断——绑定位由本
                     // 函数消费（`with x as T {}` 的 T 不再被误吞为 Cast 类型位）。
        self.with_header = true;
        let ctx_res = self.parse_expr();
        self.with_header = false;
        let ctx = ctx_res?;
        let binding = if self.is_kind(TokenKind::As) {
            self.next(); // skip `as`
            if !matches!(self.cur.kind, TokenKind::Ident) {
                return Err(SyntaxError::Generic {
                    message: "with-as binding expects an identifier after 'as'".to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            let name = self.cur.text.clone();
            self.next();
            Some(name)
        } else {
            None
        };
        self.skip_empty_lines();
        let body = self.body()?;
        match binding {
            None => {
                // 直产 `py_with(ctx, () => { body })` 调用（Stmt::Expr——非
                // Block：body 尾块的 convert_last_block 会把 Block 末语句转
                // 对象字面量，T09 实证）。539 py_with 通道：host __enter__/体/
                // __exit__；异常抑制默认不做（§10 裁决）。A5 规则（T06）随后
                // 自动包 py_callable。
                let closure = crate::ast::Closure {
                    params: vec![],
                    ret: None,
                    body: Box::new(Expr::Block(body)),
                    is_move: false,
                };
                let call = crate::ast::Call {
                    name: Box::new(Expr::Ident("py_with".into())),
                    args: crate::ast::Args {
                        args: vec![
                            crate::ast::Arg::Pos(ctx),
                            crate::ast::Arg::Pos(Expr::Closure(closure)),
                        ],
                    },
                    ret: crate::ast::Type::Unknown,
                    type_args: Vec::new(),
                    generic_args: Vec::new(),
                    pos: None,
                };
                Ok(Stmt::Expr(Expr::Call(call)))
            }
            Some(name) => {
                // 块形态：绑定 + 出口保证（规范序列，a2py 回译 `with e as x:`）。
                let mk_store = |n: &str, e: Expr| {
                    Stmt::Store(crate::ast::Store {
                        kind: crate::ast::StoreKind::Var,
                        name: n.into(),
                        ty: crate::ast::Type::Unknown,
                        expr: e,
                        attrs: Vec::new(),
                        is_pub: false,
                    })
                };
                let mk_bridge = |n: &str, arg: Expr| {
                    Expr::Call(crate::ast::Call {
                        name: Box::new(Expr::Ident(n.into())),
                        args: crate::ast::Args {
                            args: vec![crate::ast::Arg::Pos(arg)],
                        },
                        ret: crate::ast::Type::Unknown,
                        type_args: Vec::new(),
                        generic_args: Vec::new(),
                        pos: None,
                    })
                };
                let w = "__w".to_string();
                let exit_call = || Stmt::Expr(mk_bridge("py_exit", Expr::Ident("__w".into())));
                let mut catch_stmts = vec![exit_call()];
                catch_stmts.push(Stmt::Expr(mk_bridge(
                    "py_raise",
                    Expr::Ident("__we".into()),
                )));
                let try_stmt = Stmt::Try(crate::ast::Try {
                    body,
                    catch_param: Some("__we".to_string()),
                    catch_body: crate::ast::Body {
                        stmts: catch_stmts,
                        has_new_line: true,
                        source_lines: Vec::new(),
                    },
                    // 正常路径的出口收口（Plan 012 P2：body/after 与 catch 两
                    // 路都汇入 finally——catch 臂内 py_raise 报错即外传、跳过
                    // finally，__exit__ 恰好一次）。
                    finally_body: Some(crate::ast::Body {
                        stmts: vec![exit_call()],
                        has_new_line: true,
                        source_lines: Vec::new(),
                    }),
                    new_line: true,
                });
                let block = crate::ast::Body {
                    stmts: vec![
                        mk_store("__w", ctx),
                        mk_store(&name, mk_bridge("py_enter", Expr::Ident("__w".into()))),
                        try_stmt,
                    ],
                    has_new_line: true,
                    source_lines: Vec::new(),
                };
                // Stmt::Expr(Expr::Block) 而非 Stmt::Block——convert_last_block
                // 会把函数体尾部的 Stmt::Block 转对象字面量（560 T09 同款坑，
                // "Last block must be an object!"）；块表达式走语句序编译且
                // 尾语句值 pop 语义正确（codegen Expr::Block 臂）。
                Ok(Stmt::Expr(Expr::Block(block)))
            }
        }
    }

    /// Plan 010 (MS3-A): `while (cond) { body }` — desugars to a conditional
    /// `for` (Iter::Cond), identical semantics to `for cond { body }`.
    fn while_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `while`
                     // Optional parentheses around the condition: `while (cond)` or `while cond`.
        let had_paren = if self.is_kind(TokenKind::LParen) {
            self.next();
            true
        } else {
            false
        };
        // Skip newlines after `while`/`(` to allow condition on the next line.
        while self.is_kind(TokenKind::Newline) {
            self.next();
        }
        let condition = self.parse_expr()?;
        if had_paren {
            self.expect(TokenKind::RParen)?;
        }
        let body = self.body()?;
        let has_new_line = body.has_new_line;
        Ok(Stmt::For(For {
            iter: Iter::Cond,
            range: condition,
            body,
            new_line: has_new_line,
            init: None,
        }))
    }

    /// Plan 010 (MS3-A): `try { body } catch (e) { handler }`.
    /// `catch` is required; the binding `(e)` is optional.
    /// Plan 012 P2 (gap 4): optional trailing `finally { cleanup }`.
    fn try_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `try`
        let body = self.body()?;
        let body_new_line = body.has_new_line;

        // `catch` is required.
        // Skip newlines/semicolons between body and catch.
        while self.is_kind(TokenKind::Newline) || self.is_kind(TokenKind::Semi) {
            self.next();
        }
        self.expect(TokenKind::Catch)?;

        // Optional binding: catch (e) or catch e. None for bare `catch { }`.
        let mut catch_param: Option<String> = None;
        let had_paren = if self.is_kind(TokenKind::LParen) {
            self.next();
            true
        } else {
            false
        };
        // If the next token is an identifier, parse the binding name.
        if had_paren {
            let name = self.parse_name()?;
            self.expect(TokenKind::RParen)?;
            catch_param = Some(name.to_string());
        } else if self.is_kind(TokenKind::Ident) {
            let name = self.parse_name()?;
            catch_param = Some(name.to_string());
        }

        // Scope the catch binding like for-loop variables do: enter a scope,
        // define the param, parse the catch body, then exit — so `e` resolves
        // inside the handler but stays invisible to `finally` and beyond.
        let has_catch_binding = catch_param.is_some();
        if let Some(ref param) = catch_param {
            self.enter_scope();
            let meta = Meta::Store(Store {
                kind: StoreKind::Var,
                attrs: vec![],
                is_pub: false,
                name: Name::from(param.as_str()),
                expr: Expr::Nil,
                ty: Type::Unknown,
            });
            self.define(param.as_str(), meta);
        }

        let catch_body = self.body()?;
        let catch_new_line = catch_body.has_new_line;
        if has_catch_binding {
            self.exit_scope();
        }

        // Plan 012 P2: optional `finally { cleanup }`. `finally` is not a
        // lexer keyword — it lexes as Ident, matched by text (contextual).
        // Plan 029 T15: newline 前瞻必须可回退——原实现无条件吞掉换行/分号，
        // 调用方 expect_eos 随之看不到分隔符，try/catch 之后的任何语句都会
        // 报 "Expected end of statement"（fn 体/on 体皆然）。lexer 是流式
        // 的，回退用 Plan 395 的 save_state/restore_state + cur/prev。
        let saved_lexer = self.lexer.save_state();
        let saved_cur = self.cur.clone();
        let saved_prev = self.prev.clone();
        while self.is_kind(TokenKind::Newline) || self.is_kind(TokenKind::Semi) {
            self.next();
        }
        let mut finally_body = None;
        let mut finally_new_line = false;
        if self.cur.text.as_str() == "finally" {
            self.next();
            let fb = self.body()?;
            finally_new_line = fb.has_new_line;
            finally_body = Some(fb);
        } else {
            self.lexer.restore_state(saved_lexer);
            self.cur = saved_cur;
            self.prev = saved_prev;
        }

        Ok(Stmt::Try(crate::ast::Try {
            body,
            catch_param,
            catch_body,
            finally_body,
            new_line: body_new_line || catch_new_line || finally_new_line,
        }))
    }

    fn return_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip return keyword
                     // Plan 012 batch F (early return): a newline after `return` ends the
                     // statement — bare early return. Previously newlines were skipped to
                     // allow the value on the next line, which silently swallowed the
                     // following statement as the return expression:
                     //     return
                     //     .count += 1   // parsed as `return (.count += 1)`
                     // Now the return value must be on the same line as `return`; no
                     // source in the repo used the next-line form.
        if self.is_kind(TokenKind::Newline)
            || self.is_kind(TokenKind::Semi)
            || self.is_kind(TokenKind::RBrace)
            || self.is_kind(TokenKind::EOF)
        {
            return Ok(Stmt::Return(Box::new(Expr::Nil)));
        }
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(Box::new(expr)))
    }

    /// Plan 321: Parse yield expression as a statement.
    /// `yield expr` suspends the generator and sends expr's value to the caller.
    /// Can also be used bare as `yield` (yields nil).
    fn yield_expr_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip 'yield' keyword
                     // Skip newlines
        while self.is_kind(TokenKind::Newline) {
            self.next();
        }
        if self.is_kind(TokenKind::Semi)
            || self.is_kind(TokenKind::RBrace)
            || self.is_kind(TokenKind::EOF)
        {
            // bare `yield` → yield nil
            return Ok(Stmt::Expr(Expr::Yield(Box::new(Expr::Nil))));
        }
        let expr = self.parse_expr()?;
        Ok(Stmt::Expr(Expr::Yield(Box::new(expr))))
    }

    /// Plan 124 Phase 2.3: Parse reply statement for ask/reply RPC
    ///
    /// reply expr
    fn reply_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip reply keyword
        let expr = self.parse_expr()?;
        Ok(Stmt::Reply(Box::new(expr)))
    }

    pub fn parse_stmt(&mut self) -> AutoResult<Stmt> {
        // Plan 6B-4.19: pub keyword prefix — unified visibility handling
        if self.cur.text.as_str() == "pub" && self.cur.kind == TokenKind::Ident {
            let saved_cur = self.cur.clone();
            let saved_prev = self.prev.clone();
            self.next(); // consume "pub"

            // Check what follows "pub"
            let stmt = match self.kind() {
                TokenKind::Use => {
                    let mut stmt = self.use_stmt()?;
                    if let Stmt::Use(ref mut u) = stmt {
                        u.is_pub = true;
                    }
                    stmt
                }
                TokenKind::Fn => self.fn_decl_stmt_with_annotations(
                    "",
                    false,
                    false,
                    false,
                    false,
                    true,
                    Vec::new(),
                    false,
                )?,
                TokenKind::Static => {
                    // pub static fn ...
                    self.next(); // skip static
                    self.fn_decl_stmt_with_annotations(
                        "",
                        false,
                        false,
                        false,
                        true,
                        true,
                        Vec::new(),
                        false,
                    )?
                }
                TokenKind::Type => self.type_decl_stmt_with_annotation(false, true, Vec::new())?,
                TokenKind::Enum | TokenKind::Tag => {
                    let mut stmt = self.enum_stmt()?;
                    if let Stmt::EnumDecl(ref mut e) = stmt {
                        e.is_pub = true;
                    }
                    stmt
                }
                TokenKind::Spec => {
                    let mut stmt = self.spec_decl_stmt()?;
                    if let Stmt::SpecDecl(ref mut s) = stmt {
                        s.is_pub = true;
                    }
                    stmt
                }
                TokenKind::Ext => {
                    // pub ext — ext block itself doesn't carry pub, just parse normally
                    self.parse_ext_stmt()?
                }
                TokenKind::Const => {
                    // pub const NAME TYPE = value (C8) — parse_store_stmt reads the
                    // leading `const` token itself.
                    let mut stmt = self.parse_store_stmt(vec![])?;
                    if let Stmt::Store(ref mut s) = stmt {
                        s.is_pub = true;
                    }
                    stmt
                }
                _ => {
                    // Not a recognized pub declaration — put the token back
                    self.lexer.push_token(self.cur.clone());
                    self.cur = saved_cur;
                    self.prev = saved_prev;
                    return self.parse_stmt_inner();
                }
            };
            return Ok(stmt);
        }

        self.parse_stmt_inner()
    }

    fn parse_stmt_inner(&mut self) -> AutoResult<Stmt> {
        // Plan 514 W1（447-① 债②）：`struct` 不是 Auto 关键字——C 风格
        // `struct Name {...}` / `struct {...}` 误用此前被当节点/表达式静默
        // 吞掉（或后随语句报误导性 E0201 名字解析错）。语句位以裸
        // `struct` 开头（后随 Ident/LBrace）直接报语法错并指出 `type`。
        if self.is_kind(TokenKind::Ident)
            && self.cur.text.as_str() == "struct"
            && !self.exists("struct")
        {
            let after = self.lexer.next()?;
            let misused = matches!(after.kind, TokenKind::Ident | TokenKind::LBrace);
            self.lexer.push_token(after);
            if misused {
                return Err(SyntaxError::Generic {
                    message: "unexpected `struct`: Auto 的具名结构体声明是 `type Name { ... }`"
                        .to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }
        let stmt = match self.kind() {
            TokenKind::Break => self.break_stmt()?,
            TokenKind::Continue => self.continue_stmt()?,
            TokenKind::Return => self.return_stmt()?,
            TokenKind::Yield => self.yield_expr_stmt()?, // Plan 321: yield
            TokenKind::Reply => self.reply_stmt()?,      // Plan 124 Phase 2.3: reply statement
            TokenKind::Use => self.use_stmt()?,
            TokenKind::Dep => {
                // Plan 364 Step 3: In Config mode, manifest `dep xmen {}` is a
                // dependency *node*, not the lang-level `dep name(version=...)`
                // declaration. `dep_stmt` assumes the latter and chokes on `{`.
                // Delegate to node parsing so bare-name dep nodes parse cleanly.
                if self.compile_dest == CompileDest::Config {
                    self.parse_node_or_call_stmt()?
                } else {
                    self.dep_stmt()?
                }
            }
            TokenKind::Pac | TokenKind::Link => {
                // Plan 364 Step 3: `pac` and `link` are reserved keywords, but
                // config/manifest files (including auto-man's index.at) use them
                // as node names: `pac("kernel") {...}`, `link foo {...}`. In
                // Config mode, delegate to node parsing so they parse as nodes
                // instead of falling through to parse_term which rejects the
                // keyword. In other modes they keep their original handling.
                if self.compile_dest == CompileDest::Config {
                    self.parse_node_or_call_stmt()?
                } else {
                    self.expr_stmt()?
                }
            }
            TokenKind::If => self.if_stmt()?,
            TokenKind::For => self.for_stmt()?,
            TokenKind::Loop => self.loop_stmt()?, // Plan 200 Task 1.1
            TokenKind::While => self.while_stmt()?, // Plan 010 (MS3-A)
            // Plan 560 T09 (E2)：with 上下文管理器糖。
            TokenKind::With => self.with_stmt()?,
            TokenKind::Try => self.try_stmt()?, // Plan 010 (MS3-A)
            TokenKind::Is => self.is_stmt()?,
            // Plan 095: Compile-time execution statements
            TokenKind::HashIf => self.hash_if_stmt()?,
            TokenKind::HashFor => self.hash_for_stmt()?,
            TokenKind::HashIs => self.hash_is_stmt()?,
            TokenKind::HashBrace => self.hash_brace_expr()?,
            TokenKind::HashIdent => self.macro_call_stmt()?,
            TokenKind::Var => self.parse_store_stmt(vec![])?,
            TokenKind::Let => self.parse_store_stmt(vec![])?,
            TokenKind::Mut => self.parse_store_stmt(vec![])?,
            TokenKind::Const => self.parse_store_stmt(vec![])?,
            TokenKind::Shared => self.parse_store_stmt(vec![])?,
            TokenKind::Fn => self.fn_decl_stmt("")?,
            TokenKind::Hash => {
                // #[...] annotation syntax (Rust-style)
                // Use centralized parse_fn_annotations() function
                let mut ann = self.parse_fn_annotations()?;

                // Skip empty lines after annotation
                self.skip_empty_lines();

                // Check if this annotation is compatible with current compile destination
                let should_skip = match self.compile_dest {
                    CompileDest::TransC if ann.has_vm && !ann.has_c => true, // Skip #[vm] in C transpiler
                    CompileDest::TransC if ann.has_rs && !ann.has_c => true, // Skip #[rs] in C transpiler
                    CompileDest::TransRust if ann.has_vm && !ann.has_rs => true, // Skip #[vm] in Rust transpiler
                    CompileDest::TransRust if ann.has_c && !ann.has_rs => true, // Skip #[c] in Rust transpiler
                    CompileDest::Interp if ann.has_c && !ann.has_vm => true, // Skip #[c] in interpreter
                    CompileDest::Interp if ann.has_rs && !ann.has_vm => true, // Skip #[rs] in interpreter
                    _ => false,
                };

                if should_skip {
                    // Skip the entire function/type declaration by parsing it normally but discarding the result
                    if self.is_kind(TokenKind::Fn) || self.is_kind(TokenKind::Static) {
                        // Skip function declaration
                        let is_static = self.is_kind(TokenKind::Static);
                        if is_static {
                            self.next(); // skip static keyword
                        }
                        // Parse with the actual annotation flags to correctly handle the function syntax
                        // For #[vm] functions, parse as VM function to allow newline termination
                        // For #[c] functions, parse as C function to allow semicolon termination
                        let _ = self.fn_decl_stmt_with_annotations(
                            "",
                            ann.has_c,
                            ann.has_vm,
                            ann.has_rs,
                            is_static,
                            ann.has_pub,
                            ann.with_params.clone(),
                            ann.has_test,
                        );
                        return Ok(Stmt::Expr(Expr::Nil));
                    } else if self.is_kind(TokenKind::Type) {
                        // Skip type declaration
                        let _ = self.type_decl_stmt();
                        return Ok(Stmt::Expr(Expr::Nil));
                    }
                }

                // Check what comes next
                // Plan 312: handle optional `pub` keyword between annotation and fn/type
                let had_pub_prefix =
                    self.cur.text.as_str() == "pub" && self.cur.kind == TokenKind::Ident;
                if had_pub_prefix {
                    ann.has_pub = true;
                    self.next(); // consume "pub"
                }
                if self.is_kind(TokenKind::Fn) || self.is_kind(TokenKind::Static) {
                    // Function declaration
                    let is_static = self.is_kind(TokenKind::Static);
                    if is_static {
                        self.next(); // skip static keyword
                    }
                    self.pending_api_attrs = ann.api_endpoint.take(); // Plan 312
                    self.pending_export_abi = ann.export_abi.take(); // Plan 610 ⑤
                    self.fn_decl_stmt_with_annotations(
                        "",
                        ann.has_c,
                        ann.has_vm,
                        ann.has_rs,
                        is_static,
                        ann.has_pub,
                        ann.with_params.clone(),
                        ann.has_test,
                    )?
                } else if self.is_kind(TokenKind::Type) {
                    // Type declaration
                    self.type_decl_stmt_with_annotation(
                        ann.has_c,
                        ann.has_pub,
                        ann.with_params.clone(),
                    )?
                } else if self.is_kind(TokenKind::Enum) || self.is_kind(TokenKind::Tag) {
                    // Plan 376: #[derive(...)] / #[serde(...)] before an enum/tag.
                    // enum_stmt() picks up the attrs we just collected via raw_attrs.
                    let mut stmt = self.enum_stmt()?;
                    if let Stmt::EnumDecl(ref mut e) = stmt {
                        if ann.has_pub {
                            e.is_pub = true;
                        }
                    }
                    stmt
                } else if self.is_kind(TokenKind::Use) {
                    // Use statement with annotation
                    // Check if this is a C/Rust import (use.c or use.rust style with angle brackets)
                    self.next(); // skip use to check next token
                    let is_c_import = self.is_kind(TokenKind::Lt) || self.is_kind(TokenKind::Str);
                    // Put the use token back
                    self.lexer.push_token(self.cur.clone());
                    self.cur = self.prev.clone(); // Go back to use token

                    if ann.has_c && is_c_import {
                        // C import: #[c] use <stdio.h>
                        self.next(); // skip use
                                     // Call use_c directly since we know it's a C import
                        let mut paths = Vec::new();
                        if self.is_kind(TokenKind::Lt) {
                            self.next(); // skip <
                            let mut name = "<".to_string();
                            while !self.is_kind(TokenKind::Gt) {
                                name.push_str(self.cur.text.as_str());
                                self.next();
                            }
                            name.push_str(">");
                            self.expect(TokenKind::Gt)?;
                            paths.push(name.into());
                        } else if self.is_kind(TokenKind::Str) {
                            let name = self.cur.text.clone();
                            self.next();
                            paths.push(format!("\"{}\"", name).into());
                        }

                        let (items, _is_wildcard) = self.parse_use_items()?;
                        // Plan 085: Import is now handled by CompileSession.resolve_uses()
                        let uses = Use {
                            kind: UseKind::C,
                            module_path: None,
                            paths,
                            items,
                            is_pub: false,
                            is_wildcard: false,
                        };
                        Stmt::Use(uses)
                    } else {
                        // Regular Auto use statement
                        self.use_stmt()?
                    }
                } else if self.is_kind(TokenKind::Let)
                    || self.is_kind(TokenKind::Var)
                    || self.is_kind(TokenKind::Mut)
                    || self.is_kind(TokenKind::Const)
                    || self.is_kind(TokenKind::Shared)
                {
                    // Variable declaration with annotation (Plan 306 2c/3):
                    // e.g. `#[export] var speed float = 300.0` → `@export var speed: float = 300.0`
                    //      `#[export_range(0,100)] var hp int = 50` → `@export_range(0, 100) var hp: int = 50`
                    let attrs = ann.store_attrs.clone();
                    self.parse_store_stmt(attrs)?
                } else if self.is_kind(TokenKind::Task) {
                    // Plan 121: Task with #[single] annotation
                    // Parse single annotation - we already consumed #[single], now check what annotation it was
                    // For now, we just treat any annotation before task as single
                    // TODO: Parse specific annotation types
                    self.parse_task_with_attrs(vec![TaskAttr::Single])?
                } else if self.is_kind(TokenKind::Ext) || self.is_kind(TokenKind::Impl) {
                    // Plan 364 W1: annotated `ext`/`impl` block — dotted macro
                    // paths (`#[zbus.interface]`) pass through to the impl block.
                    self.parse_ext_stmt()?
                } else {
                    return Err(SyntaxError::Generic {
                        message:
                            "Expected 'fn', 'type', 'use', 'let', 'var', or 'task' after annotation"
                                .to_string(),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
            TokenKind::Type => self.type_decl_stmt()?,
            TokenKind::Union => self.union_stmt()?,
            // DEPRECATED: tag → enum 别名;2026-08-22 起上下文化(见
            // tag_starts_enum_decl):非声明形态的 tag 按标识符表达式处理。
            TokenKind::Tag => {
                if self.tag_starts_enum_decl() {
                    self.enum_stmt()?
                } else {
                    self.expr_stmt()?
                }
            }
            TokenKind::Spec => self.spec_decl_stmt()?,
            TokenKind::LBrace => Stmt::Block(self.body()?),
            // UI 场景：view 块（如 `view { Column { ... } }`）。
            // 非 UI 场景：view 是参数模式关键字，不应出现在语句位置，回退到表达式。
            TokenKind::View => {
                if let Some(stmt) = self.try_token_dialect_stmt(TokenKind::View)? {
                    stmt
                } else {
                    self.expr_stmt()?
                }
            }
            // Node Instance or UI contextual keywords
            TokenKind::Ident => {
                // Plan 306: Godot scene declaration (recognized in any scenario).
                // Checked before borrowing `ident` to satisfy the borrow checker.
                if self.looks_like_scene_decl() {
                    self.parse_scene_decl()?
                } else {
                    let ident = self.cur.text.to_string();
                    if let Some(stmt) = self.try_dialect_stmt(&ident)? {
                        stmt
                    } else {
                        self.parse_node_or_call_stmt()?
                    }
                }
            }
            // Enum Definition
            TokenKind::Enum => self.enum_stmt()?,
            // Plan 121: Task Definition
            TokenKind::Task => self.parse_task()?,
            // On Events Switch —— 委托方言（UI 场景由 UiDialect 接管，非 UI 回退原行为）
            TokenKind::On => {
                if let Some(stmt) = self.try_token_dialect_stmt(TokenKind::On)? {
                    stmt
                } else {
                    Stmt::OnEvents(self.parse_on_events()?)
                }
            }
            // Alias stmt
            TokenKind::Alias => self.parse_alias_stmt()?,
            // Ext statement (Plan 035)
            TokenKind::Ext => self.parse_ext_stmt()?,
            // Impl statement (Plan 059: synonym for ext, Rust-compatible syntax)
            TokenKind::Impl => self.parse_ext_stmt()?,
            // mod name { ... } — flatten inner statements into current scope
            // pub fn names get prefixed with "mod_name." so they are accessible
            // via module_name.method() calls (e.g., network.connect())
            TokenKind::Mod => {
                self.next(); // skip 'mod'
                let mod_name = self.cur.text.clone();
                self.next(); // skip module name
                self.skip_empty_lines();
                self.expect(TokenKind::LBrace)?;
                // Don't use self.body() — it would try expect(LBrace) again.
                // Manually collect statements until RBrace.
                let mut stmts = Vec::new();
                let mut source_lines = Vec::new();
                while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
                    self.skip_empty_lines();
                    if self.is_kind(TokenKind::RBrace) {
                        break;
                    }
                    let stmt_line = self.cur.pos.line;
                    stmts.push(self.parse_stmt()?);
                    source_lines.push(stmt_line);
                }
                self.expect(TokenKind::RBrace)?;
                let inner_body = Body {
                    stmts,
                    source_lines,
                    ..Default::default()
                };
                // Prefix pub fn names with mod_name.
                let prefixed_stmts: Vec<Stmt> = inner_body
                    .stmts
                    .into_iter()
                    .map(|mut stmt| {
                        if let Stmt::Fn(ref mut fn_decl) = stmt {
                            if fn_decl.is_pub {
                                fn_decl.name = auto_val::AutoStr::from(format!(
                                    "{}.{}",
                                    mod_name, fn_decl.name
                                ));
                            }
                        }
                        stmt
                    })
                    .collect();
                Stmt::Block(Body {
                    stmts: prefixed_stmts,
                    ..inner_body
                })
            }
            // Otherwise, try to parse as an expression
            _ => self.expr_stmt()?,
        };
        Ok(stmt)
    }

    fn parse_alias_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip alias
        let alias = self.cur.text.clone();
        self.next();
        self.expect(TokenKind::Asn)?;
        let target = self.cur.text.clone();
        self.next();
        // define alias meta in scope
        self.define_alias(alias.clone(), target.clone());
        Ok(Stmt::Alias(Alias { alias, target }))
    }

    /// Parse ext statement: ext Target { methods... }
    ///
    /// Adds methods to an existing type (like Rust's impl block).
    /// Both instance methods (fn) and static methods (static fn) are supported.
    ///
    /// # Example
    ///
    /// ```auto
    /// ext str {
    ///     fn len() int {
    ///         return .size
    ///     }
    ///
    ///     static fn new(data *char, size int) str {
    ///         return str_new(data, size)
    ///     }
    /// }
    /// ```
    fn parse_ext_stmt(&mut self) -> AutoResult<Stmt> {
        // Plan 391 D6: record whether this block was opened by `impl` (Rust-style
        // trait-impl attempt) vs `ext` (Auto's legitimate spec-impl syntax
        // `ext Type for Spec { ... }`). Only `impl ... for` is an error.
        let opened_by_impl = self.is_kind(TokenKind::Impl);
        self.next(); // skip `ext` or `impl` keyword

        // Plan 364 W1: impl-level macro attrs from the annotation dispatch
        // (`#[zbus.interface(...)] ext Foo { ... }`). Taken BEFORE the method
        // loop so methods' own fn-level annotations don't steal them.
        let ext_attrs = std::mem::take(&mut self.impl_attrs);

        // Plan 059: Parse optional generic parameters for impl blocks
        // e.g., impl<T, S> ListIter<T, S> { ... }
        let mut generic_params = Vec::new();
        if self.is_kind(TokenKind::Lt) {
            self.next(); // Consume '<'
            generic_params.push(self.parse_generic_param()?);

            while self.is_kind(TokenKind::Comma) {
                self.next(); // Consume ','
                generic_params.push(self.parse_generic_param()?);
            }

            self.expect(TokenKind::Gt)?; // Consume '>'
        }

        // Parse target type name (e.g., "str", "Point", "ListIter")
        // Note: We only parse the base name, not generic instance like ListIter<T, S>
        let target = self.parse_name()?;

        // Plan 391 D6: Auto has no trait-impl syntax. Rust-style
        // `impl Trait for Type { ... }` would otherwise be silently misparsed
        // here (target=Trait, `for` mishandled → impl Type for Trait, reversed).
        // Detect the `... for <ident>` tail and raise a clear error instead of
        // producing a wrong-arity impl block. NOTE: `ext Type for Spec` is a
        // LEGITIMATE Auto spec-impl syntax — only flag `impl ... for`.
        // (Skipping any generic args first, since `impl TryFrom<Node> for Foo`
        // has `<Node>` between name and for.)
        if opened_by_impl && self.is_kind(TokenKind::Lt) {
            // skip `<...>` generic args on the trait name
            self.next();
            let mut depth = 1i32;
            while depth > 0 {
                let tok = self.lexer.next()?;
                match tok.kind {
                    TokenKind::Lt => depth += 1,
                    TokenKind::Gt => depth -= 1,
                    _ => {}
                }
            }
        }
        if opened_by_impl && self.is_kind(TokenKind::For) {
            return Err(SyntaxError::Generic {
                message: "Auto does not support trait impl syntax (`impl Trait for Type`). \
                          Use a `static fn`/`ext` method with an equivalent name instead."
                    .to_string(),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }

        // Skip generic instance syntax if present (e.g., <T, S> after ListIter)
        // For now, we just extract the base type name and skip the generic parameters
        // This allows `impl<T, S> ListIter<T, S>` to work
        if self.is_kind(TokenKind::Lt) {
            // Generic instance syntax like ListIter<T, S>
            self.next(); // skip '<'

            // Parse type arguments
            if self.next_token_is_type() {
                let _ = self.parse_type()?;
            }

            // Parse additional type arguments separated by commas
            while self.is_kind(TokenKind::Comma) {
                self.next(); // skip ','
                if self.next_token_is_type() {
                    let _ = self.parse_type()?;
                }
            }

            self.expect(TokenKind::Gt)?; // skip '>'
        }

        // Plan 164: Parse optional "for TraitName" for external trait implementation
        // e.g., ext Point for Display { ... }
        let mut trait_name: Option<Name> = None;
        let mut trait_generic_args: Vec<crate::ast::Type> = Vec::new();
        if self.is_kind(TokenKind::For) {
            self.next(); // skip 'for'
            trait_name = Some(self.parse_name()?);

            // Plan 6B-2.7: Store generic args on trait name, e.g., ext MyType for From<String>
            if self.is_kind(TokenKind::Lt) {
                self.next(); // skip '<'
                if self.next_token_is_type() {
                    trait_generic_args.push(self.parse_type()?);
                }
                while self.is_kind(TokenKind::Comma) {
                    self.next();
                    if self.next_token_is_type() {
                        trait_generic_args.push(self.parse_type()?);
                    }
                }
                self.expect(TokenKind::Gt)?;
            }
        }

        // Expect opening brace
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        // Parse fields, consts and methods
        let mut fields = Vec::new();
        let mut consts = Vec::new();
        let mut methods = Vec::new();

        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            // Check for annotations: #[c], #[vm], #[rs], #[c,vm] before function declarations
            // Note: pub is a keyword prefix, not an annotation (pub fn, pub static fn)
            let ann = self.parse_fn_annotations()?;

            self.skip_empty_lines(); // Skip newlines after annotations

            // Check if this annotation should be skipped for current compile destination
            let should_skip = match self.compile_dest {
                CompileDest::TransC if ann.has_vm && !ann.has_c => true, // Skip #[vm] in C transpiler
                CompileDest::TransC if ann.has_rs && !ann.has_c => true, // Skip #[rs] in C transpiler
                CompileDest::TransRust if ann.has_vm && !ann.has_rs => true, // Skip #[vm] in Rust transpiler
                CompileDest::TransRust if ann.has_c && !ann.has_rs => true, // Skip #[c] in Rust transpiler
                CompileDest::Interp if ann.has_c && !ann.has_vm => true, // Skip #[c] in interpreter
                CompileDest::Interp if ann.has_rs && !ann.has_vm => true, // Skip #[rs] in interpreter
                _ => false,
            };

            // Plan 6B-4.19: Handle `pub` keyword prefix inside ext body
            // Must be checked BEFORE field detection since `pub` is also an Ident
            let local_has_pub =
                if self.cur.text.as_str() == "pub" && self.cur.kind == TokenKind::Ident {
                    self.next(); // consume "pub"
                    true
                } else {
                    ann.has_pub
                };

            // Parse field declarations: name Type (same syntax as type members)
            // Fields must come before methods
            if self.is_kind(TokenKind::Ident) {
                // Check if this is a field (next token is a type)
                // Lookahead: if current token is Ident and next token is NOT a keyword that starts a statement
                let is_field = match self.peek().kind {
                    TokenKind::Fn | TokenKind::Static | TokenKind::Has | TokenKind::RBrace => false,
                    TokenKind::Colon => {
                        // Old syntax with colon - reject with helpful error
                        return Err(SyntaxError::Generic {
                            message: "ext field syntax should be 'name Type' (without colon), same as type members. Use: '_fp *FILE' not '_fp: *FILE'".to_string(),
                            span: pos_to_span(self.cur.pos),
                        }.into());
                    }
                    _ => true,
                };

                if is_field {
                    // Parse field: name Type [optional_default_value]
                    let field_name = self.parse_name()?;
                    let field_type = self.parse_type()?;
                    let mut value = None;
                    if self.is_kind(TokenKind::Asn) {
                        self.next(); // skip =
                        let expr = self.parse_expr()?;
                        value = Some(expr);
                    }

                    // ext fields are always private
                    let mut member = crate::ast::Member::new(field_name, field_type, value);
                    // Plan 163: Collect per-field attributes from raw_attrs
                    if !self.raw_attrs.is_empty() {
                        member.attrs = std::mem::take(&mut self.raw_attrs);
                    }
                    fields.push(member);

                    self.expect_eos(false)?;
                    self.skip_empty_lines();
                    continue;
                }
            }

            // C8: `const NAME TYPE = value` / `pub const NAME TYPE = value`
            // inside an ext block → associated const on the impl block.
            if self.is_kind(TokenKind::Const) {
                let stmt = self.parse_store_stmt(vec![])?;
                if let Stmt::Store(mut store) = stmt {
                    store.is_pub = local_has_pub;
                    consts.push(store);
                }
                self.expect_eos(false)?;
                self.skip_empty_lines();
                continue;
            }

            // Parse method declarations (fn or static fn)
            // IMPORTANT: Check Static BEFORE Fn, since "static fn" starts with Static
            if self.is_kind(TokenKind::Static)
                || self.is_kind(TokenKind::Mut)
                || self.is_kind(TokenKind::Fn)
            {
                // Track if this is a static method (Plan 035 Phase 4)
                let is_static_method = self.is_kind(TokenKind::Static);
                // Plan 163: Track if this is a mut method (&mut self)
                let is_mut_method = self.is_kind(TokenKind::Mut);

                // If static fn, skip the static keyword first
                if is_static_method {
                    self.next(); // skip `static` keyword
                                 // Now we expect `fn` keyword
                    if !self.is_kind(TokenKind::Fn) {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "expected 'fn' after 'static', found {:?}",
                                self.kind()
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                }

                // Plan 163: If mut fn, skip the mut keyword first
                if is_mut_method {
                    self.next(); // skip `mut` keyword
                    if !self.is_kind(TokenKind::Fn) {
                        return Err(SyntaxError::Generic {
                            message: format!("expected 'fn' after 'mut', found {:?}", self.kind()),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                }

                // Check if we should skip this based on annotation
                if should_skip {
                    // Skip the entire function declaration
                    // Parse with the actual annotation flags to correctly handle the function syntax
                    let _ = self.fn_decl_stmt_with_annotations(
                        &target,
                        ann.has_c,
                        ann.has_vm,
                        ann.has_rs,
                        is_static_method,
                        local_has_pub,
                        ann.with_params.clone(),
                        ann.has_test,
                    );
                    self.expect_eos(false)?;
                    self.skip_empty_lines();
                    continue;
                }

                let fn_stmt = self.fn_decl_stmt_with_annotations(
                    &target,
                    ann.has_c,
                    ann.has_vm,
                    ann.has_rs,
                    is_static_method,
                    local_has_pub,
                    ann.with_params.clone(),
                    ann.has_test,
                )?;
                if let Stmt::Fn(mut fn_expr) = fn_stmt {
                    // Set is_static flag for static methods (Plan 035 Phase 4.2)
                    if is_static_method {
                        fn_expr.is_static = true;
                    }
                    // Plan 163: Set is_mut flag for mutable methods
                    if is_mut_method {
                        fn_expr.is_mut = true;
                    }
                    methods.push(fn_expr);
                }
                // For VM/C/RS methods, they can end with newline (interface contract)
                // For regular methods, expect EOS (semicolon or newline after statement)
                if !ann.has_vm && !ann.has_c && !ann.has_rs {
                    self.expect_eos(false)?;
                }
                self.skip_empty_lines();
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "ext blocks can only contain field (name Type) or method declarations (fn or static fn), found {:?}",
                        self.kind()
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }

        // Expect closing brace
        self.expect(TokenKind::RBrace)?;

        // TODO: Implement proper module tracking
        // For now, assume same-module (will be enhanced in future task)
        let module_path: AutoStr = "".into();
        let is_same_module = true;

        // Plan 059: Create Ext with generic params, fields, consts, and methods
        let ext = Ext {
            target,
            trait_name,
            trait_generic_args,
            generic_params,
            fields,
            consts,
            methods,
            module_path,
            is_same_module,
            // Plan 364 W1: impl-level attribute macros (`#[zbus.interface]`) from annotations
            attrs: ext_attrs,
        };
        Ok(Stmt::Ext(ext))
    }

    fn parse_use_items(&mut self) -> AutoResult<(Vec<AutoStr>, bool)> {
        let mut items = Vec::new();
        let mut is_wildcard = false;
        // end of path, next should be a colon (for items), LBrace (Rust-style {items}), or end-of-statement
        if self.is_kind(TokenKind::Colon) {
            self.next(); // skip :
                         // Plan 167: Support wildcard import (use module: *)
            if self.is_kind(TokenKind::Star) {
                self.next(); // skip *
                is_wildcard = true;
            } else if self.is_kind(TokenKind::Ident) {
                let name = self.expect_ident_str()?;
                items.push(name);
            } else {
                return Err(SyntaxError::Generic {
                    message: format!("Expected identifier or *, got {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            while self.is_kind(TokenKind::Comma) {
                self.next(); // skip ,
                let name = self.expect_ident_str()?;
                items.push(name);
            }
        } else if self.is_kind(TokenKind::LBrace) {
            // Rust-style grouped import: use.rust std::collections::{HashMap, HashSet}
            self.next(); // skip {
            while !self.is_kind(TokenKind::RBrace) {
                if self.is_kind(TokenKind::Star) {
                    self.next();
                    is_wildcard = true;
                } else {
                    let name = self.expect_ident_str()?;
                    items.push(name);
                }
                if self.is_kind(TokenKind::Comma) {
                    self.next(); // skip ,
                }
            }
            self.expect(TokenKind::RBrace)?; // skip }
        }
        Ok((items, is_wildcard))
    }

    pub fn use_c_stmt(&mut self) -> AutoResult<Stmt> {
        let mut paths = Vec::new();
        // include "<lib.h>"
        if self.is_kind(TokenKind::Lt) {
            self.next(); // skip <
            let mut name = "<".to_string();
            while !self.is_kind(TokenKind::Gt) {
                name.push_str(self.cur.text.as_str());
                self.next(); // skip lib
            }
            name.push_str(">");
            self.expect(TokenKind::Gt)?;
            paths.push(name.into());
        } else if self.is_kind(TokenKind::Str) {
            // include "lib.h"
            let name = self.cur.text.clone();
            self.next(); // skip lib
            paths.push(format!("\"{}\"", name).into());
        } else {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Expected <lib> or \"lib\", got {:?}, {}",
                    self.kind(),
                    self.cur.text
                ),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }

        let (items, _is_wildcard) = self.parse_use_items()?;

        for item in items.iter() {
            // add item to scope
            self.define(item.as_str(), Meta::Use(item.into()));
        }

        let uses = Use {
            kind: UseKind::C,
            module_path: None,
            paths,
            items,
            is_pub: false,
            is_wildcard: false,
        };

        Ok(Stmt::Use(uses))
    }

    /// Parse `use.rust crate::module::{item1, item2}`
    /// Plan 092: Rust FFI import
    pub fn use_rust_stmt(&mut self) -> AutoResult<Stmt> {
        // Already consumed: use . rust
        // Now parse: crate::module::{items}

        let mut paths = Vec::new();

        // Get crate name (first identifier)
        let name = self.expect_ident_str()?;
        paths.push(name.into());

        // Parse module path (:: separated)
        while self.is_kind(TokenKind::Colon) {
            self.next(); // skip first :
            if !self.is_kind(TokenKind::Colon) {
                // Single colon - might be type annotation, stop here
                break;
            }
            self.next(); // skip second :

            // Check for { which starts import items
            if self.is_kind(TokenKind::LBrace) {
                break;
            }

            let segment = self.expect_ident_str()?;
            paths.push(segment.into());
        }

        // Parse import items
        let (items, is_wildcard) = self.parse_use_items()?;

        let uses = Use {
            kind: UseKind::Rust,
            module_path: None,
            paths,
            items,
            is_pub: false,
            is_wildcard,
        };

        Ok(Stmt::Use(uses))
    }

    /// Plan 214/300.1: Parse `use.py module: items` statement
    ///
    /// Syntax: `use.py a.b.c` or `use.py a.b.c: x, y`
    pub fn use_py_stmt(&mut self) -> AutoResult<Stmt> {
        // Already consumed: use . py
        // Now parse: module: items

        let mut paths = Vec::new();

        let name = self.expect_ident_str()?;
        paths.push(name.into());

        // Parse module path (. separated for Python)
        while self.is_kind(TokenKind::Dot) {
            self.next();
            let segment = self.expect_ident_str()?;
            paths.push(segment.into());
        }

        // Parse items after colon: use.py math: sqrt, pow
        let (items, is_wildcard) = self.parse_use_items()?;

        // Plan 369 P4 (Task 13): record each imported item name so the symbol
        // checker (check_symbol / exists) treats bare references to py-imported
        // constants and functions (e.g. NAMESPACE_DNS, pi, uuid5) as defined.
        // Without this, py constants only resolve in method-call position
        // (`pi.to(str)`) and fail with "undefined variable" when used as a call
        // argument or assignment RHS (`uuid5(NAMESPACE_DNS, ...)`).
        for it in &items {
            let name = it.as_str();
            if !self.py_item_imports.iter().any(|n| n.as_str() == name) {
                self.py_item_imports.push(name.into());
            }
        }

        let uses = Use {
            kind: UseKind::Py,
            module_path: None,
            paths,
            items,
            is_pub: false,
            is_wildcard,
        };

        Ok(Stmt::Use(uses))
    }

    /// Parse `use.web NAME from "path"` (PLAN-037 T6) — the frontend sibling
    /// of `use.rust` / `use.py`. Forms:
    ///
    ///   use.web agentAvatarData from "src/front/forge_helpers.at"
    ///   use.web component MessageSquare, ListTodo from "lucide-vue-next"
    ///   use.web composable useT from "src/front/composables/useT.ts"
    ///   use.web composable useI18n refs: [locale] from "vue-i18n"
    ///
    /// Default (no modifier) is the plain import — the imported binding may
    /// be a function, object, or constant (ES imports don't distinguish).
    /// `component` additionally registers instantiable view tags;
    /// `composable` auto-invokes once at setup and binds the result.
    // 1. auto: use std.io: println
    // 2. c: use c <stdio.h>
    // 3. rust: use rust std::fs
    pub fn use_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip use

        // Plan 131: Check for pac. or super. prefix
        // Support super (1 level), super2 (2 levels), super3 (3 levels), super4 (4 levels)
        let prefix = if self.is_kind(TokenKind::Pac) {
            self.next(); // skip 'pac'
            self.expect(TokenKind::Dot)?;
            PathPrefix::Pac
        } else if self.is_kind(TokenKind::Super2) {
            self.next(); // skip 'super2'
            self.expect(TokenKind::Dot)?;
            PathPrefix::Super(2)
        } else if self.is_kind(TokenKind::Super3) {
            self.next(); // skip 'super3'
            self.expect(TokenKind::Dot)?;
            PathPrefix::Super(3)
        } else if self.is_kind(TokenKind::Super4) {
            self.next(); // skip 'super4'
            self.expect(TokenKind::Dot)?;
            PathPrefix::Super(4)
        } else if self.is_kind(TokenKind::Super) {
            self.next(); // skip 'super'
            self.expect(TokenKind::Dot)?;
            PathPrefix::Super(1)
        } else {
            PathPrefix::None
        };

        let mut segments = Vec::new();

        // check use.c or use.rust (only when no prefix)
        if prefix == PathPrefix::None && self.is_kind(TokenKind::Dot) {
            self.next(); // skip .
                         // Plan 470: capture the lang keyword's span before consuming it,
                         // so the deprecation label points at `use.rust` itself.
            let kw_span = pos_to_span(self.cur.pos);
            let name = self.expect_ident_str()?;

            if name == "c" {
                return self.use_c_stmt();
            } else if name == "rust" || name == "rs" {
                // Plan 470: `use.rs` is the canonical spelling (aligns with
                // `use.py`); `use.rust` keeps parsing but warns (W0005).
                if name == "rust" {
                    self.warn(Warning::DeprecatedFeature {
                        name: "use.rust".to_string(),
                        message: "use 'use.rs' instead".to_string(),
                        span: kw_span,
                    });
                }
                return self.use_rust_stmt();
            } else if name == "py" {
                return self.use_py_stmt();
            } else {
                segments.push(name.into());
            }
        } else {
            let name = self.expect_ident_str()?;
            segments.push(name.into());
        }

        while self.is_kind(TokenKind::Dot) {
            self.next(); // skip .
                         // Keywords pac/super/super2/super3/super4 not allowed after dot
            if self.is_kind(TokenKind::Pac)
                || self.is_kind(TokenKind::Super)
                || self.is_kind(TokenKind::Super2)
                || self.is_kind(TokenKind::Super3)
                || self.is_kind(TokenKind::Super4)
            {
                return Err(SyntaxError::Generic {
                    message:
                        "Keywords 'pac' and 'super' can only appear at the start of a module path"
                            .into(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            let segment = self.expect_ident_str()?;
            segments.push(segment.into());
        }

        let (items, is_wildcard) = self.parse_use_items()?;

        // Build ModulePath (Plan 131)
        let module_path = Some(ModulePath::new(
            prefix.clone(),
            segments.clone(),
            items.clone(),
        ));

        // Legacy paths for backward compat
        let paths = if prefix == PathPrefix::Pac {
            // Skip "pac" prefix for legacy paths (pac.db -> ["db"])
            segments
        } else if matches!(prefix, PathPrefix::Super(_)) {
            // Include "super" in legacy paths (super.utils -> ["super", "utils"])
            let super_count = match &prefix {
                PathPrefix::Super(n) => *n,
                _ => 0,
            };
            let mut p: Vec<AutoStr> = (0..super_count).map(|_| "super".into()).collect();
            p.extend(segments);
            p
        } else {
            segments
        };

        // Track imported module names for check_symbol (e.g., "flow" from "use super.flow")
        if let Some(last) = paths.last() {
            let name = last.as_str();
            if name != "super" {
                let already = self.use_imports.iter().any(|s| s.as_str() == name);
                if !already {
                    self.use_imports.push(last.clone());
                }
            }
        }

        // Create the Use statement
        // Plan 085: Import is now handled by CompileSession.resolve_uses()
        // The symbols should already be in type_store before parsing
        let uses = Use {
            kind: UseKind::Auto,
            module_path,
            paths,
            items,
            is_pub: false,
            is_wildcard,
        };
        Ok(Stmt::Use(uses))
    }

    /// Parse `dep crate_name(version: "...", features: [...])`
    /// Plan 092: Dependency declaration
    pub fn dep_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip 'dep'

        // Get crate name
        let name = self.expect_ident_str()?;

        let mut version = None;
        let mut features = Vec::new();
        let mut path = None;
        let mut git = None;
        let mut git_ref = None;

        // Check for optional properties in parentheses
        if self.is_kind(TokenKind::LParen) {
            self.next(); // skip '('

            while !self.is_kind(TokenKind::RParen) {
                let key = self.expect_ident_str()?;
                self.expect(TokenKind::Colon)?;

                match key.as_str() {
                    "version" => {
                        let v = self.parse_string_literal()?;
                        version = Some(v.into());
                    }
                    "features" => {
                        // Parse array: ["derive", "rc"]
                        self.expect(TokenKind::LSquare)?;
                        while !self.is_kind(TokenKind::RSquare) {
                            let f = self.parse_string_literal()?;
                            features.push(f.into());
                            if self.is_kind(TokenKind::Comma) {
                                self.next();
                            }
                        }
                        self.expect(TokenKind::RSquare)?;
                    }
                    "path" => {
                        let p = self.parse_string_literal()?;
                        path = Some(p.into());
                    }
                    "git" => {
                        let g = self.parse_string_literal()?;
                        git = Some(g.into());
                    }
                    "branch" | "tag" | "rev" => {
                        let r = self.parse_string_literal()?;
                        git_ref = Some(r.into());
                    }
                    _ => {
                        return Err(SyntaxError::Generic {
                            message: format!("Unknown dep property: {}", key),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                }

                if self.is_kind(TokenKind::Comma) {
                    self.next();
                }
            }

            self.expect(TokenKind::RParen)?;
        }

        let dep = DepStmt {
            name: name.into(),
            version,
            features,
            path,
            git,
            git_ref,
        };

        Ok(Stmt::Dep(dep))
    }

    pub(super) fn parse_body(&mut self, is_node: bool) -> AutoResult<Body> {
        // Plan 419 §8.3:深度护栏(见 parse 注释)。
        self.body_depth += 1;
        if self.body_depth > MAX_BODY_NESTING {
            self.body_depth -= 1;
            return Err(SyntaxError::Generic {
                message: format!(
                    "block nesting too deep (limit {}) [non-recoverable]",
                    MAX_BODY_NESTING
                ),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }
        let r = stacker::maybe_grow(512 * 1024, 1024 * 1024, || self.parse_body_inner(is_node));
        self.body_depth -= 1;
        r
    }

    fn parse_body_inner(&mut self, is_node: bool) -> AutoResult<Body> {
        self.expect(TokenKind::LBrace)?;
        self.enter_scope();

        let mut stmts = Vec::new();
        let mut source_lines = Vec::new();
        let new_lines = self.skip_empty_lines();

        // Snapshot docs collected right after `{` (and optional newlines) —
        // these belong to this node's body.
        // Clear pending_docs so child node bodies don't pollute this snapshot.
        let body_doc_snapshot: Vec<_> = self.pending_docs.drain(..).collect();

        if new_lines > 1 {
            stmts.push(Stmt::EmptyLine(new_lines - 1));
            source_lines.push(0); // EmptyLine placeholder
        }
        let has_new_line = new_lines > 0;
        let mut stmt_index = 0; // Track statement index for is_first_stmt check

        // Track whether each statement started with a `.` token (implicit self)
        let mut stmt_starts_with_dot: Vec<bool> = Vec::new();

        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            let stmt_line = self.cur.pos.line;
            let starts_with_dot = self.is_kind(TokenKind::Dot);

            match self.parse_stmt() {
                Ok(stmt) => {
                    if is_node {
                        if let Stmt::Expr(Expr::Pair(Pair { key, value })) = &stmt {
                            // define as a property
                            self.define(
                                key.to_string().as_str(),
                                Meta::Pair(Pair {
                                    key: key.clone(),
                                    value: value.clone(),
                                }),
                            );
                        }
                    }
                    stmts.push(stmt);
                    source_lines.push(stmt_line);
                    stmt_starts_with_dot.push(starts_with_dot);

                    // Method chaining: merge `.method()` lines into previous expression.
                    // Only chain when the current statement STARTS WITH `.` (implicit self),
                    // NOT when it starts with `self.` (explicit self — a normal method call).
                    //
                    // Plan 041a⑥-a(musk 041 Phase 5 T20):on-handler/watch 体内
                    // 禁用链合并——该语境句首 `.X(args)` 是消息派发语义
                    // (emit 链),`let draft = {...};` 后接 `.go(draft)` 曾被
                    // 链成 `let draft = {...}.go(draft)`(musk TestEditor/GoalEditor
                    // 的 DoSave 实测踩中)。链式糖保留给普通 fn 体。
                    let stmts_len = stmts.len();
                    if self.in_on_body {
                        stmt_starts_with_dot.pop();
                    } else if stmts_len >= 2 && *stmt_starts_with_dot.last().unwrap_or(&false) {
                        fn is_dot_self_call(expr: &Expr) -> bool {
                            match expr {
                                Expr::Dot(obj, _) => {
                                    matches!(obj.as_ref(), Expr::Ident(name) if name == "self")
                                }
                                Expr::Call(call) => {
                                    matches!(call.name.as_ref(), Expr::Dot(obj, _) if matches!(obj.as_ref(), Expr::Ident(name) if name == "self"))
                                }
                                Expr::ErrorPropagate(inner) => is_dot_self_call(inner),
                                _ => false,
                            }
                        }

                        // Get the current expression to check if it's a self.dot call
                        let curr_is_chainable = match stmts.get(stmts_len - 1) {
                            Some(Stmt::Expr(e)) => is_dot_self_call(e),
                            _ => false,
                        };

                        if curr_is_chainable {
                            // Find the previous expression to chain to.
                            // Walk back through stmts to find the last non-self-dot expression.
                            // This handles: let x = A.new()\n  .b()\n  .c()
                            //
                            // Plan 043 cat-3 Guard 1: skip dot-prefix statements
                            // (`.field = expr`) as chain targets — they are independent
                            // statements, not method-chain receivers. A real receiver
                            // (`let x = A.new()`) never starts with `.`.
                            //
                            // If we cross ANY dot-prefix statement while walking back,
                            // don't chain at all — a method chain's receiver must be
                            // the *immediately preceding* statement, not an earlier one
                            // reached by skipping over assignments. This prevents
                            // `.RefreshGit()` at the end of an assignment sequence from
                            // chaining back to `let snap = command_list()` at the top.
                            let mut chain_target_idx = None;
                            let mut crossed_dot_prefix = false;
                            for i in (0..stmts_len - 1).rev() {
                                if *stmt_starts_with_dot.get(i).unwrap_or(&false) {
                                    crossed_dot_prefix = true;
                                    continue;
                                }
                                match &stmts[i] {
                                    Stmt::Expr(e)
                                        if !is_dot_self_call(e)
                                            && !matches!(e, Expr::Nil | Expr::Null) =>
                                    {
                                        chain_target_idx = Some(i);
                                        break;
                                    }
                                    Stmt::Store(s) if !matches!(s.expr, Expr::Nil | Expr::Null) => {
                                        chain_target_idx = Some(i);
                                        break;
                                    }
                                    _ => continue,
                                }
                            }
                            if crossed_dot_prefix {
                                chain_target_idx = None;
                            }

                            if chain_target_idx.is_some() {
                                // Plan 514（用户裁定 2026-09-02）：取消流式链糖的
                                // 换行续接——句首 `.method()` 语句不再自动合并进
                                // 上一条语句。该糖与方法体内隐式 self 的句首点
                                // 存在静默歧义（`(i = 1).next()` 形态实测），旧
                                // 形态直接报错，等待显式管道算子（|> 设计中）。
                                // 同一行的 `.b().c()` 链不受影响；方法体内首条
                                // 语句的句首点仍为隐式 self。
                                let message = "换行后的句首 `.method()` 不再自动续接上一条语句（流式链糖已移除）[non-recoverable]: 请将调用链写在同一行、拆成中间变量绑定，或用管道算子 `|>` 显式续行（如 `x |> .method()`）"
                                    .to_string();
                                let err: crate::error::AutoError = SyntaxError::Generic {
                                    message,
                                    span: pos_to_span(self.cur.pos),
                                }
                                .into();
                                // 已入错误表 + cur=EOF 强停:高层(type/fn 体)恢复
                                // 循环不再级联次生错误(镜像 error-limit 中止形)
                                self.errors.push(err.clone());
                                self.cur = Token {
                                    kind: TokenKind::EOF,
                                    pos: crate::token::Pos {
                                        line: 0,
                                        at: 0,
                                        pos: 0,
                                        len: 0,
                                    },
                                    text: "".into(),
                                };
                                self.exit_scope();
                                return Err(err);
                            }
                        }
                    }

                    let is_first = stmt_index == 0;
                    // Check for ambiguous syntax errors in expect_eos
                    let newline_count = match self.expect_eos(is_first) {
                        Ok(count) => count,
                        Err(e) => {
                            // Ambiguous syntax errors should not be recovered from
                            if e.to_string().contains("Ambiguous syntax") {
                                self.exit_scope();
                                return Err(e);
                            }
                            // Other EOS errors are added to collection and synchronized
                            if !self.add_error(e) {
                                self.exit_scope();
                                return Err(self.error_limit_exceeded());
                            }
                            self.synchronize();
                            0
                        }
                    };
                    // Insert EmptyLine statement for 2+ consecutive newlines
                    if newline_count > 1 {
                        stmts.push(Stmt::EmptyLine(newline_count - 1));
                        source_lines.push(stmt_line);
                    }
                    stmt_index += 1;
                }
                Err(e) => {
                    // Plan 419 §8.3:护栏错误不可恢复 —— 立即中止(防逐层
                    // re-catch 嵌套放大错误树,见 add_error 注释)。
                    if e.to_string().contains("[non-recoverable]") {
                        self.exit_scope();
                        return Err(e);
                    }
                    // Add error to collection and synchronize
                    if !self.add_error(e) {
                        self.exit_scope();
                        return Err(self.error_limit_exceeded()); // Error limit exceeded
                    }
                    self.synchronize();
                }
            }
        }

        stmts = self.convert_last_block(stmts)?;
        self.exit_scope();

        // Restore the body doc snapshot at the front of pending_docs,
        // so parse_node's take_docs() picks up this node's own docs (not child nodes' docs).
        // Any docs accumulated after the last child stmt are also included.
        let remaining: Vec<_> = self.pending_docs.drain(..).collect();
        self.pending_docs = body_doc_snapshot;
        self.pending_docs.extend(remaining);

        self.expect(TokenKind::RBrace)?;
        Ok(Body {
            stmts,
            has_new_line,
            source_lines,
        })
    }

    /// Plan 043 M5: is `Type{...}` a struct-literal construction (braces =
    /// named field args) in the UI scenario? Mirrors atom()'s gate: a
    /// registered type, or PascalCase + not a known var (unresolved imports).
    /// Kept in its own fn so the callers' stack frames stay small.
    pub(super) fn is_ui_struct_construction(&mut self, ident: &Expr, is_constructor: bool) -> bool {
        if is_constructor {
            return true;
        }
        let ident_name = match ident {
            Expr::Ident(n) => n.as_str(),
            Expr::GenName(n) => n.as_str().split('<').next().unwrap_or(""),
            _ => "",
        };
        if ident_name.is_empty() {
            return false;
        }
        ident_name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
            && !matches!(
                self.lookup_meta(ident_name),
                Some(m) if matches!(m.as_ref(), Meta::Store(_) | Meta::Ref(_))
            )
    }

    /// Plan 043 M5: parse `Type{ field: value, ... }` braces as NAMED ARGS
    /// and append them to the given args. Separate fn so node_or_call_expr /
    /// atom (hot recursive paths) don't grow their stack frames — godot scene
    /// files (dodge_player) barely fit in the 2MB test stack.
    pub(super) fn parse_braced_struct_args(&mut self, ident: &Expr, mut args: Args) -> AutoResult<Args> {
        let _ = ident; // name is used by object()/pair resolution implicitly
        let pairs = self.object()?;
        for p in pairs {
            match p.key {
                Key::NamedKey(name) => {
                    args.args.push(Arg::Pair(name, (*p.value).clone()));
                }
                _ => {
                    // Non-named keys can't be struct fields — keep the pair
                    // as a positional expression.
                    args.args.push(Arg::Pos(Expr::Pair(p)));
                }
            }
        }
        Ok(args)
    }

    pub fn body(&mut self) -> AutoResult<Body> {
        self.parse_body(false)
    }

    pub fn if_contents(&mut self) -> AutoResult<(Vec<Branch>, Option<Body>)> {
        let mut branches = Vec::new();
        self.next(); // skip if
        let cond = self.parse_expr()?;
        let body = self.body()?;
        branches.push(Branch { cond, body });

        let mut else_stmt = None;
        while self.is_kind(TokenKind::Else) {
            self.next(); // skip else
                         // more branches
            if self.is_kind(TokenKind::If) {
                self.next(); // skip if
                let cond = self.parse_expr()?;
                let body = self.body()?;
                branches.push(Branch { cond, body });
            } else {
                // last else
                else_stmt = Some(self.body()?);
            }
        }
        Ok((branches, else_stmt))
    }

    pub fn if_stmt(&mut self) -> AutoResult<Stmt> {
        // Check for `if let` syntax sugar
        // if let Some(x) = expr { body }  desugars to  is expr { Some(x) -> body }
        self.next(); // skip `if`
        if self.is_kind(TokenKind::Let) {
            return self.if_let_stmt();
        }

        // Normal if: we already consumed `if`, handle the rest inline
        let cond = self.parse_expr()?;
        let body = self.body()?;
        let mut branches = vec![Branch { cond, body }];

        let mut else_stmt = None;
        while self.is_kind(TokenKind::Else) {
            self.next(); // skip else
            if self.is_kind(TokenKind::If) {
                self.next(); // skip if
                if self.is_kind(TokenKind::Let) {
                    // else if let — wrap in else { is ... }
                    let is_stmt = self.if_let_stmt_inner()?;
                    let mut else_body = Body::new();
                    else_body.stmts.push(is_stmt);
                    else_stmt = Some(else_body);
                    break;
                }
                let cond = self.parse_expr()?;
                let body = self.body()?;
                branches.push(Branch { cond, body });
            } else {
                else_stmt = Some(self.body()?);
            }
        }

        Ok(Stmt::If(If {
            branches,
            else_: else_stmt,
        }))
    }

    /// Parse `if let Pattern = expr { body } else { ... }`
    /// `let` has already been consumed by the caller.
    fn if_let_stmt(&mut self) -> AutoResult<Stmt> {
        self.if_let_stmt_inner()
    }

    /// Inner implementation — `let` has NOT been consumed yet.
    fn if_let_stmt_inner(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `let`
        let pattern = self.is_branch_cond_expr()?;
        self.expect(TokenKind::Asn)?;
        let target = self.parse_expr()?;

        // Bind pattern variables into scope before parsing body
        let body = self.bind_pattern_and_parse_body(&pattern, &target)?;

        let mut branches = vec![IsBranch::EqBranch(vec![pattern], body)];

        // Optional else clause
        if self.is_kind(TokenKind::Else) {
            self.next(); // skip else
            let else_body = self.parse_expr_or_body()?;
            branches.push(IsBranch::ElseBranch(else_body));
        }

        Ok(Stmt::Is(Is { target, branches }))
    }

    /// Bind pattern variables into parser scope and parse the body.
    /// Mirrors the binding logic in `parse_is_branch`.
    fn bind_pattern_and_parse_body(&mut self, pattern: &Expr, target: &Expr) -> AutoResult<Body> {
        if let Expr::OptionPattern(opt_cover) = pattern {
            if let Some(binding) = &opt_cover.binding {
                self.enter_scope();
                self.define(
                    binding.as_str(),
                    Meta::Store(Store {
                        name: binding.clone(),
                        kind: StoreKind::Let,
                        attrs: vec![],
                        is_pub: false,
                        ty: Type::Unknown,
                        expr: Expr::OptionUncover(crate::ast::cover::OptionUncover {
                            src: target.repr(),
                            variant: opt_cover.variant,
                            binding: binding.clone(),
                        }),
                    }),
                );
                let body = self.parse_expr_or_body()?;
                self.exit_scope();
                return Ok(body);
            }
        }
        if let Expr::ResultPattern(res_cover) = pattern {
            if let Some(binding) = &res_cover.binding {
                self.enter_scope();
                self.define(
                    binding.as_str(),
                    Meta::Store(Store {
                        name: binding.clone(),
                        kind: StoreKind::Let,
                        attrs: vec![],
                        is_pub: false,
                        ty: Type::Unknown,
                        expr: Expr::ResultUncover(crate::ast::cover::ResultUncover {
                            src: target.repr(),
                            variant: res_cover.variant,
                            binding: binding.clone(),
                        }),
                    }),
                );
                let body = self.parse_expr_or_body()?;
                self.exit_scope();
                return Ok(body);
            }
        }
        // Tag pattern or simple pattern — no binding
        self.parse_expr_or_body()
    }

    pub fn for_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `for`
        if self.is_kind(TokenKind::LBrace) {
            // for {...}
            let body = self.body()?;
            let has_new_line = body.has_new_line;
            return Ok(Stmt::For(For {
                iter: Iter::Ever,
                range: Expr::Nil,
                body,
                new_line: has_new_line,
                init: None,
            }));
        }

        // Check if this has an initializer: for let x = 0; condition { ... }
        if self.is_kind(TokenKind::Let)
            || self.is_kind(TokenKind::Var)
            || self.is_kind(TokenKind::Mut)
        {
            // Parse the initializer statement
            let init_stmt = Some(Box::new(self.parse_store_stmt(vec![])?));

            // Expect semicolon after initializer
            self.expect(TokenKind::Semi)?;

            // After initializer, must parse condition
            let condition = self.parse_expr()?;
            let body = self.body()?;
            let has_new_line = body.has_new_line;
            return Ok(Stmt::For(For {
                iter: Iter::Cond,
                range: condition,
                body,
                new_line: has_new_line,
                init: init_stmt,
            }));
        }

        // Destructured for-in: for (k, v) in expr { ... }
        if self.is_kind(TokenKind::LParen) {
            self.next(); // skip '('
            let key = self.parse_name()?;
            self.expect(TokenKind::Comma)?;
            let val = self.parse_name()?;
            self.expect(TokenKind::RParen)?;
            self.expect(TokenKind::In)?;
            self.enter_scope();
            let meta_key = Meta::Store(Store {
                kind: StoreKind::Var,
                attrs: vec![],
                is_pub: false,
                name: key.clone(),
                expr: Expr::Nil,
                ty: Type::Int,
            });
            self.define(key.as_str(), meta_key);
            let meta_val = Meta::Store(Store {
                kind: StoreKind::Var,
                attrs: vec![],
                is_pub: false,
                name: val.clone(),
                expr: Expr::Nil,
                ty: Type::Int,
            });
            self.define(val.as_str(), meta_val);
            let iter = Iter::Destructured(key, val);
            let range = self.iterable_expr()?;
            let body = self.body()?;
            let has_new_line = body.has_new_line;
            self.exit_scope();
            return Ok(Stmt::For(For {
                iter,
                range,
                body,
                new_line: has_new_line,
                init: None,
            }));
        }

        // No initializer, try to parse as iterator pattern: for ident in range { ... }
        // Or conditional loop: for ident < max { ... }
        if self.is_kind(TokenKind::Ident) {
            // Consume the identifier first
            let ident = self.parse_name()?;

            // Now check what comes next to determine the pattern
            if self.is_kind(TokenKind::In) {
                // for ident in range { ... }
                self.next(); // skip 'in'
                self.enter_scope();
                let meta = Meta::Store(Store {
                    kind: StoreKind::Var,
                    attrs: vec![],
                    is_pub: false,
                    name: ident.clone(),
                    expr: Expr::Nil,
                    ty: Type::Int,
                });
                self.define(ident.as_str(), meta);
                let iter = Iter::Named(ident.clone());
                let range = self.iterable_expr()?;
                let body = self.body()?;
                let has_new_line = body.has_new_line;
                self.exit_scope();
                return Ok(Stmt::For(For {
                    iter,
                    range,
                    body,
                    new_line: has_new_line,
                    init: None,
                }));
            } else if self.is_kind(TokenKind::Comma) {
                // for ident, ident2 in range { ... } - indexed iterator
                self.next(); // skip ','
                let ident2 = self.parse_name()?;
                self.expect(TokenKind::In)?; // this calls self.next() internally
                self.enter_scope();
                let meta = Meta::Store(Store {
                    kind: StoreKind::Var,
                    attrs: vec![],
                    is_pub: false,
                    name: ident2.clone(),
                    expr: Expr::Nil,
                    ty: Type::Int,
                });
                self.define(ident2.as_str(), meta);
                let iter = Iter::Indexed(ident.clone(), ident2.clone());
                let range = self.iterable_expr()?;
                let body = self.body()?;
                let has_new_line = body.has_new_line;
                self.exit_scope();
                return Ok(Stmt::For(For {
                    iter,
                    range,
                    body,
                    new_line: has_new_line,
                    init: None,
                }));
            } else if self.is_kind(TokenKind::LParen) {
                let args = self.args()?;
                let call_expr = self.call(Expr::Ident(ident.clone()), args)?;

                // Check if this is an iterator call: for call(args)? { ... }
                if self.is_kind(TokenKind::Question) {
                    self.next(); // skip '?'
                    let Expr::Call(call) = call_expr else {
                        return Err(SyntaxError::Generic {
                            message: "Strange call in for statement".to_string(),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    };
                    self.enter_scope();
                    let body = self.body()?;
                    self.exit_scope();
                    return Ok(Stmt::For(For {
                        iter: Iter::Call(call),
                        range: Expr::Nil,
                        body,
                        new_line: false,
                        init: None,
                    }));
                }

                // Otherwise it's a conditional loop: for call(args) { ... }
                // or for call(args) || other { ... }
                let condition = self.expr_pratt_with_left(call_expr, 0)?;
                let body = self.body()?;
                let has_new_line = body.has_new_line;
                return Ok(Stmt::For(For {
                    iter: Iter::Cond,
                    range: condition,
                    body,
                    new_line: has_new_line,
                    init: None,
                }));
            }

            // Check if next token is a binary operator (like <, >, <=, >=, ==, !=, +, -, *, /)
            let next_is_binop = matches!(
                self.kind(),
                TokenKind::Lt
                    | TokenKind::Gt
                    | TokenKind::Le
                    | TokenKind::Ge
                    | TokenKind::Eq
                    | TokenKind::Neq
                    | TokenKind::Add
                    | TokenKind::Sub
                    | TokenKind::Star
                    | TokenKind::Div
            );

            if next_is_binop {
                // This is a binary expression like "for i < max { ... }"
                // We've already consumed 'i', so we need to prepend it to the expression
                let ident_expr = Expr::Ident(ident.clone());
                let rest = self.expr_pratt_with_left(ident_expr, 0)?;
                let body = self.body()?;
                let has_new_line = body.has_new_line;
                return Ok(Stmt::For(For {
                    iter: Iter::Cond,
                    range: rest,
                    body,
                    new_line: has_new_line,
                    init: None,
                }));
            }

            // Identifier followed by something else (member access, etc.)
            // Fall through to parse as a general expression starting with this ident
            let ident_expr = Expr::Ident(ident);
            let condition = self.expr_pratt_with_left(ident_expr, 0)?;
            let body = self.body()?;
            let has_new_line = body.has_new_line;
            return Ok(Stmt::For(For {
                iter: Iter::Cond,
                range: condition,
                body,
                new_line: has_new_line,
                init: None,
            }));
        }

        // Otherwise, parse as conditional for loop: for condition { ... }
        let condition = self.parse_expr()?;
        let body = self.body()?;
        let has_new_line = body.has_new_line;
        Ok(Stmt::For(For {
            iter: Iter::Cond,
            range: condition,
            body,
            new_line: has_new_line,
            init: None,
        }))
    }

    pub fn is_stmt(&mut self) -> AutoResult<Stmt> {
        Ok(Stmt::Is(self.parse_is()?))
    }

    // Plan 095: Compile-time execution statements

    /// Parse #if statement for compile-time conditional compilation
    /// Syntax: #if condition { ... } else { ... }
    /// Returns HashIf directly (not wrapped in Stmt) for recursive elif parsing
    fn hash_if_inner(&mut self) -> AutoResult<HashIf> {
        self.next(); // skip #if

        // Parse condition - use parse_expr just like normal if statement
        // This allows: #if DEBUG { ... } and #if x == 10 { ... }
        let cond = self.parse_expr()?;
        let then_block = self.body()?;

        // Check for else clause
        let else_block = if self.is_kind(TokenKind::Else) {
            self.next(); // skip else
            if self.is_kind(TokenKind::HashIf) {
                // else #if - elif chain
                Some(HashIfElse::ElseIf(Box::new(self.hash_if_inner()?)))
            } else {
                // else { ... }
                Some(HashIfElse::Block(self.body()?))
            }
        } else {
            None
        };

        Ok(HashIf {
            cond,
            then_block,
            else_block,
        })
    }

    /// Public wrapper that returns Stmt::HashIf
    pub fn hash_if_stmt(&mut self) -> AutoResult<Stmt> {
        Ok(Stmt::HashIf(self.hash_if_inner()?))
    }

    /// Parse #for statement for compile-time loop unrolling
    /// Syntax: #for var in iterable { ... }
    pub fn hash_for_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip #for
        let var = self.parse_name()?;
        self.expect(TokenKind::In)?;

        // Enter scope and define loop variable (same as normal for loop)
        self.enter_scope();
        let meta = Meta::Store(Store {
            kind: StoreKind::Var,
            attrs: vec![],
            is_pub: false,
            name: var.clone(),
            expr: Expr::Nil,
            ty: Type::Int, // Default type, will be inferred later
        });
        self.define(var.as_str(), meta);

        let iter = self.iterable_expr()?;
        let body = self.body()?;

        self.exit_scope();

        Ok(Stmt::HashFor(HashFor { var, iter, body }))
    }

    /// Plan 212 Phase 2.4: Parse #name(args) macro invocation
    /// Syntax: #debug("msg"), #info(f"value = $x")
    /// Token: HashIdent (text = "debug") followed by LParen
    pub fn macro_call_stmt(&mut self) -> AutoResult<Stmt> {
        let name = self.cur.text.to_string();
        self.next(); // skip HashIdent
        self.expect(TokenKind::LParen)?;
        let mut args = Vec::new();
        if !self.is_kind(TokenKind::RParen) {
            args.push(self.parse_expr()?);
            while self.is_kind(TokenKind::Comma) {
                self.next();
                args.push(self.parse_expr()?);
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(Stmt::MacroCall(MacroCall {
            name: name.into(),
            args,
        }))
    }

    /// Parse #is statement for compile-time type matching
    /// Syntax: #is target { pattern1 => body1 pattern2 => body2 else => body3 }
    /// Same syntax as normal is statement, just with # prefix
    pub fn hash_is_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip #is
        let target = self.lhs_expr()?;

        self.expect(TokenKind::LBrace)?; // {
        self.skip_empty_lines();

        let mut branches = Vec::new();

        while !self.is_kind(TokenKind::RBrace) {
            let branch = self.parse_hash_is_branch(&target)?;
            branches.push(branch);
        }
        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::HashIs(HashIs { target, branches }))
    }

    /// Parse a single branch in #is statement
    /// Reuses the same logic as normal is branch parsing
    fn parse_hash_is_branch(&mut self, tgt: &Expr) -> AutoResult<HashIsBranch> {
        match self.cur.kind {
            TokenKind::If => {
                self.next(); // skip if
                let expr = self.cond_expr()?;
                self.expect(TokenKind::Arrow)?;
                let body = self.parse_expr_or_body()?;
                let branch = HashIsBranch::IfBranch(expr, body);
                self.skip_empty_lines();
                return Ok(branch);
            }
            TokenKind::Else => {
                self.next(); // skip else
                self.expect(TokenKind::Arrow)?;
                let body = self.parse_expr_or_body()?;
                let branch = HashIsBranch::ElseBranch(body);
                self.skip_empty_lines();
                return Ok(branch);
            }
            _ => {
                // Pattern expression (e.g., "x64", type_name)
                let expr = self.is_branch_cond_expr()?;
                self.expect(TokenKind::Arrow)?;

                // Check for pattern binding cases (same as normal is)
                let body = if let Expr::Cover(Cover::Tag(cover)) = &expr {
                    // Tag pattern: Msg.Inc(value) => ...
                    // Skip if all bindings are underscore (no-op)
                    let has_bindings = cover.bindings.iter().any(|b| b.as_str() != "_");
                    if has_bindings {
                        self.enter_scope();
                        let tag_typ = self.lookup_type(&cover.kind);
                        let tag_field_type = match *tag_typ.borrow() {
                            Type::Tag(ref t) => t.borrow().get_field_type(&cover.tag),
                            _ => {
                                return Err(SyntaxError::Generic {
                                    message: format!("Invalid tag type: {}", cover.kind),
                                    span: pos_to_span(self.cur.pos),
                                }
                                .into());
                            }
                        };

                        for binding in &cover.bindings {
                            if binding.as_str() != "_" {
                                self.define(
                                    binding.as_str(),
                                    Meta::Store(Store {
                                        name: binding.clone(),
                                        kind: StoreKind::Let,
                                        attrs: vec![],
                                        is_pub: false,
                                        ty: tag_field_type.clone(),
                                        expr: Expr::Uncover(TagUncover {
                                            src: tgt.repr(),
                                            cover: cover.clone(),
                                            binding: binding.clone(),
                                        }),
                                    }),
                                );
                            }
                        }

                        let body = self.parse_expr_or_body()?;
                        self.exit_scope();
                        body
                    } else {
                        self.parse_expr_or_body()?
                    }
                } else {
                    self.parse_expr_or_body()?
                };

                let branch = HashIsBranch::EqBranch(expr, body);
                self.skip_empty_lines();
                return Ok(branch);
            }
        }
    }

    /// Parse #{} expression for compile-time code execution
    /// Syntax: #{ expression }
    /// Used for compile-time evaluation and string interpolation
    pub fn hash_brace_expr(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip #{
        let expr = self.parse_expr()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::HashBrace(HashBrace { expr }))
    }

    pub fn parse_is(&mut self) -> AutoResult<Is> {
        self.next(); // skip is
                     // Disable symbol checking for is-target (it's a name to match, not a variable to resolve)
        let old_skip_check = self.skip_check;
        self.skip_check = true;
        let target = self.parse_expr()?;
        self.skip_check = old_skip_check;

        self.expect(TokenKind::LBrace)?; // {
        self.skip_empty_lines();

        let mut branches = Vec::new();

        while !self.is_kind(TokenKind::RBrace) {
            let branch = self.parse_is_branch(&target)?;
            branches.push(branch);
            // Skip optional comma separator between branches
            if self.is_kind(TokenKind::Comma) {
                self.next();
                self.skip_empty_lines();
            }
        }
        self.expect(TokenKind::RBrace)?;

        let is = Is { target, branches };
        return Ok(is);
    }

    pub fn parse_expr_or_body(&mut self) -> AutoResult<Body> {
        if self.is_kind(TokenKind::LBrace) {
            self.body()
        } else {
            // Single-statement fallback (e.g. `Pattern -> print(...)` in an is
            // arm): record the statement's line so codegen emits SOURCE_LINE
            // and the arm maps to bytecode for source linking.
            let mut body = Body::new();
            let stmt_line = self.cur.pos.line;
            let stmt = if self.is_kind(TokenKind::Return) {
                self.return_stmt()?
            } else if self.is_kind(TokenKind::Break) {
                self.break_stmt()?
            } else if self.is_kind(TokenKind::Continue) {
                self.continue_stmt()?
            } else {
                Stmt::Expr(self.parse_expr()?)
            };
            body.stmts.push(stmt);
            body.source_lines.push(stmt_line as usize);
            Ok(body)
        }
    }

    pub fn parse_is_branch(&mut self, tgt: &Expr) -> AutoResult<IsBranch> {
        match self.cur.kind {
            TokenKind::If => {
                self.next(); // skip is
                let expr = self.cond_expr()?;
                self.expect(TokenKind::Arrow)?;
                let body = self.parse_expr_or_body()?;
                let branch = IsBranch::IfBranch(expr, body);
                self.skip_empty_lines();
                return Ok(branch);
            }
            TokenKind::Else => {
                self.next(); // skip else
                self.expect(TokenKind::Arrow)?;
                let body = self.parse_expr_or_body()?;
                let branch = IsBranch::ElseBranch(body);
                self.skip_empty_lines();
                return Ok(branch);
            }
            _ => {
                let mut patterns = vec![self.is_branch_cond_expr()?];
                // Collect additional patterns separated by |
                while self.is_kind(TokenKind::VBar) {
                    self.next(); // skip |
                    self.skip_empty_lines();
                    patterns.push(self.is_branch_cond_expr()?);
                    self.skip_empty_lines();
                }
                let expr = patterns.swap_remove(0); // take first for compatibility
                self.expect(TokenKind::Arrow)?;
                self.skip_empty_lines();

                // Check for pattern binding cases
                let body = if let Expr::Cover(Cover::Tag(cover)) = &expr {
                    // Empty variant (all bindings == "_"): no binding needed
                    let has_bindings = cover.bindings.iter().any(|b| b.as_str() != "_");
                    if !has_bindings {
                        self.parse_expr_or_body()?
                    } else {
                        // Tag pattern with binding: Msg.Inc(value) => ...
                        self.enter_scope();
                        let tag_typ = self.lookup_type(&cover.kind);
                        let tag_field_type = match *tag_typ.borrow() {
                            Type::Tag(ref t) => t.borrow().get_field_type(&cover.tag),
                            Type::Enum(ref en) => {
                                // Heterogeneous enum pattern: Atom.Int(i) => ...
                                let en_ref = en.borrow();
                                match &en_ref.kind {
                                    EnumKind::Heterogeneous { .. } => {
                                        // Find the variant's payload type
                                        en_ref
                                            .items
                                            .iter()
                                            .find(|item| item.name == cover.tag.as_str())
                                            .and_then(|item| item.payload_type.clone())
                                            .unwrap_or(Type::Unknown)
                                    }
                                    _ => {
                                        return Err(SyntaxError::Generic {
                                            message: format!(
                                                "Invalid enum type for tag pattern: {}",
                                                cover.kind
                                            ),
                                            span: pos_to_span(self.cur.pos),
                                        }
                                        .into());
                                    }
                                }
                            }
                            // Imported enum types appear as User — use Unknown for field types
                            _ => Type::Unknown,
                        };

                        for binding in &cover.bindings {
                            if binding.as_str() != "_" {
                                self.define(
                                    binding.as_str(),
                                    Meta::Store(Store {
                                        name: binding.clone(),
                                        kind: StoreKind::Let,
                                        attrs: vec![],
                                        is_pub: false,
                                        ty: tag_field_type.clone(),
                                        expr: Expr::Uncover(TagUncover {
                                            src: tgt.repr(),
                                            cover: cover.clone(),
                                            binding: binding.clone(),
                                        }),
                                    }),
                                );
                            }
                        }
                        let body = self.parse_expr_or_body()?;
                        self.exit_scope();
                        body
                    }
                } else if let Expr::OptionPattern(opt_cover) = &expr {
                    // Plan 120: Option pattern: Some(x) => ... or None => ...
                    if let Some(binding) = &opt_cover.binding {
                        self.enter_scope();
                        // Define variable with unknown type (will be inferred)
                        self.define(
                            binding.as_str(),
                            Meta::Store(Store {
                                name: binding.clone(),
                                kind: StoreKind::Let,
                                attrs: vec![],
                                is_pub: false,
                                ty: Type::Unknown, // TODO: Infer from Option<T>
                                expr: Expr::OptionUncover(crate::ast::cover::OptionUncover {
                                    src: tgt.repr(),
                                    variant: opt_cover.variant,
                                    binding: binding.clone(),
                                }),
                            }),
                        );
                        let body = self.parse_expr_or_body()?;
                        self.exit_scope();
                        body
                    } else {
                        // None pattern - no binding
                        self.parse_expr_or_body()?
                    }
                } else if let Expr::StructPattern(sc) = &expr {
                    // Plan 396/B11(b): struct-variant destructuring pattern
                    // `Type.Variant { field, .. }` (or bare `Type { .. }`) —
                    // register the field bindings for the arm body. Without
                    // this the body's uses of the bindings are "undefined
                    // variable" and error recovery mangles the rest.
                    self.enter_scope();
                    for fb in &sc.fields {
                        if fb.binding.as_str() != "_" {
                            self.define(
                                fb.binding.as_str(),
                                Meta::Store(Store {
                                    name: fb.binding.clone(),
                                    kind: StoreKind::Let,
                                    attrs: vec![],
                                    is_pub: false,
                                    ty: Type::Unknown,
                                    expr: Expr::Int(0), // pattern-bound placeholder
                                }),
                            );
                        }
                    }
                    let body = self.parse_expr_or_body()?;
                    self.exit_scope();
                    body
                } else if let Expr::ResultPattern(res_cover) = &expr {
                    // Plan 120: Result pattern: Ok(x) => ... or Err(e) => ...
                    if let Some(binding) = &res_cover.binding {
                        self.enter_scope();
                        // Define variable with unknown type (will be inferred)
                        self.define(
                            binding.as_str(),
                            Meta::Store(Store {
                                name: binding.clone(),
                                kind: StoreKind::Let,
                                attrs: vec![],
                                is_pub: false,
                                ty: Type::Unknown, // TODO: Infer from Result<T, E>
                                expr: Expr::ResultUncover(crate::ast::cover::ResultUncover {
                                    src: tgt.repr(),
                                    variant: res_cover.variant,
                                    binding: binding.clone(),
                                }),
                            }),
                        );
                        let body = self.parse_expr_or_body()?;
                        self.exit_scope();
                        body
                    } else if let Some(inner) = &res_cover.inner {
                        // Plan 396/B11(b): nested pattern inside Ok()/Err(),
                        // e.g. `Err(ToolError.SecurityDenied { kind, path })`.
                        // Define the inner pattern's bindings for the arm body.
                        self.enter_scope();
                        match inner.as_ref() {
                            Expr::StructPattern(sc) => {
                                for fb in &sc.fields {
                                    if fb.binding.as_str() != "_" {
                                        self.define(
                                            fb.binding.as_str(),
                                            Meta::Store(Store {
                                                name: fb.binding.clone(),
                                                kind: StoreKind::Let,
                                                attrs: vec![],
                                                is_pub: false,
                                                ty: Type::Unknown,
                                                expr: Expr::Int(0), // pattern-bound placeholder
                                            }),
                                        );
                                    }
                                }
                            }
                            Expr::Cover(Cover::Tag(tc)) => {
                                for b in &tc.bindings {
                                    if b.as_str() != "_" {
                                        self.define(
                                            b.as_str(),
                                            Meta::Store(Store {
                                                name: b.clone(),
                                                kind: StoreKind::Let,
                                                attrs: vec![],
                                                is_pub: false,
                                                ty: Type::Unknown,
                                                expr: Expr::Int(0), // pattern-bound placeholder
                                            }),
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                        let body = self.parse_expr_or_body()?;
                        self.exit_scope();
                        body
                    } else {
                        // Pattern without binding (shouldn't happen for Ok/Err but handle gracefully)
                        self.parse_expr_or_body()?
                    }
                } else {
                    // Default case: simple expression match
                    self.parse_expr_or_body()?
                };

                let mut all_patterns = vec![expr];
                all_patterns.extend(patterns);
                let branch = IsBranch::EqBranch(all_patterns, body);
                self.skip_empty_lines();
                return Ok(branch);
            }
        }
    }

    pub fn parse_store_stmt(&mut self, attrs: Vec<AutoStr>) -> AutoResult<Stmt> {
        // Plan 6B-4.19: Check for 'shared' modifier
        let is_shared = self.is_kind(TokenKind::Shared);
        if is_shared {
            self.next(); // skip 'shared'
        }

        // store kind: var/let (mut keyword is now aliased to var)
        let mut store_kind = self.store_kind()?;
        self.next(); // skip var/let/mut

        // Plan 6B-4.19: shared var/let → StoreKind::Shared
        if is_shared {
            store_kind = StoreKind::Shared;
        }

        // Check for tuple destructuring: let (a, b) = expr
        if self.is_kind(TokenKind::LParen) {
            self.next(); // skip (
            let mut names = Vec::new();
            if !self.is_kind(TokenKind::RParen) {
                names.push(self.parse_name()?);
                while self.is_kind(TokenKind::Comma) {
                    self.next();
                    self.skip_empty_lines();
                    names.push(self.parse_name()?);
                }
            }
            self.expect(TokenKind::RParen)?;
            self.expect(TokenKind::Asn)?;
            let expr = self.rhs_expr()?;
            // Emit as a single Store with the tuple expr; downstream handles the destructuring
            // Use the first name as primary, store the full tuple and names
            // For now, emit a special Expr::TupleDestruct that the transpiler can handle
            let _name = names.first().cloned().unwrap_or_else(|| Name::from("_"));
            return Ok(Stmt::Expr(Expr::TupleDestruct {
                names,
                expr: Box::new(expr),
            }));
        }

        // identifier name
        let mut name = self.parse_name()?;

        // Capture the position of the variable name for LSP
        let name_pos = self.prev.pos;

        // special case: c decl
        if name == "." {
            self.next();
            name = self.parse_name()?;
            if name == "c" {
                store_kind = StoreKind::CVar;
            }
        }

        // type (optional)
        let mut ty = Type::Unknown;
        if self.is_type_name() {
            ty = self.parse_type()?;
        }

        // `=`, a store stmt must have an assignment unless:
        // 1. It's a C variable decl (StoreKind::CVar)
        // 2. It has an explicit type annotation (Plan 052: allow uninitialized typed variables)
        let has_explicit_type = !matches!(ty, Type::Unknown);
        let expr = if matches!(store_kind, StoreKind::CVar) {
            Expr::Nil
        } else if self.is_kind(TokenKind::Asn) {
            self.expect(TokenKind::Asn)?;
            // inital value: expression
            let expr = self.rhs_expr()?;
            // TODO: check type compatibility
            if matches!(ty, Type::Unknown) {
                ty = self.infer_type_expr(&expr);
            }
            expr
        } else if has_explicit_type {
            // Plan 052: Allow uninitialized variables with explicit type annotation
            Expr::Nil
        } else {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Variable '{}' must have either a type annotation or an initial value",
                    name
                ),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        };

        let store = Store {
            kind: store_kind,
            name: name.clone(),
            ty,
            expr: expr.clone(),
            attrs,
            is_pub: false,
        };
        if let Expr::Lambda(lambda) = &expr {
            self.define(name.as_str(), Meta::Ref(lambda.name.clone()));
        } else {
            self.define(name.as_str(), Meta::Store(store.clone()));
        }

        // Register symbol location for LSP
        let loc = SymbolLocation::new(
            name_pos.line.saturating_sub(1), // Convert from 1-based to 0-based
            name_pos.at,
            name_pos.pos,
        );
        // Plan 091: Use wrapper method
        self.define_symbol_location(name.clone(), loc);

        Ok(Stmt::Store(store))
    }

    pub(super) fn infer_type_expr(&mut self, expr: &Expr) -> Type {
        // Plan 089: Use infer module as primary type inference system
        // Previously: Universe-based lookup as primary, infer as fallback
        // Now: infer module as primary, Universe as fallback for backward compatibility

        let mut typ = Type::Unknown;

        match expr {
            // Literals - directly return type (same in both old and new systems)
            Expr::I8(..) => typ = Type::Int,
            Expr::Int(..) => typ = Type::Int,
            Expr::Float(..) => typ = Type::Float,
            Expr::Double(..) => typ = Type::Double,
            Expr::Bool(..) => typ = Type::Bool,
            Expr::Str(n) => typ = Type::StrFixed(n.len()),
            Expr::CStr(..) => typ = Type::CStrLit,
            Expr::FStr(..) => typ = Type::StrFixed(0),

            // Identifier - prioritize infer module
            Expr::Ident(id) => {
                use crate::infer::infer_expr;

                // Primary: Try infer module first
                let inferred = infer_expr(&mut self.infer_ctx, expr);

                if !matches!(&inferred, Type::Unknown) {
                    typ = inferred;
                } else {
                    // Fallback: Try Universe-based lookup for backward compatibility
                    let meta = self.lookup_meta(id);
                    if let Some(m) = meta {
                        if let Meta::Store(store) = m.as_ref() {
                            typ = store.ty.clone();
                        }
                    } else {
                        // Try type lookup
                        let ltyp = self.lookup_type(id);
                        if !matches!(*ltyp.borrow(), Type::Unknown) {
                            typ = ltyp.borrow().clone();
                        }
                    }
                }
            }

            // Node - defer to infer module
            Expr::Node(_nd) => {
                use crate::infer::infer_expr;
                typ = infer_expr(&mut self.infer_ctx, expr);
            }

            // Array - defer to infer module
            Expr::Array(_arr) => {
                use crate::infer::infer_expr;
                typ = infer_expr(&mut self.infer_ctx, expr);
            }

            // Call - defer to infer module
            Expr::Call(_call) => {
                use crate::infer::infer_expr;
                typ = infer_expr(&mut self.infer_ctx, expr);
            }

            // Index - defer to infer module
            Expr::Index(_arr, _idx) => {
                use crate::infer::infer_expr;
                typ = infer_expr(&mut self.infer_ctx, expr);
            }

            // Binary op - defer to infer module
            Expr::Bina(_lhs, _op, _rhs) => {
                use crate::infer::infer_expr;
                typ = infer_expr(&mut self.infer_ctx, expr);
            }

            // All other expressions - use infer module directly
            _ => {
                use crate::infer::infer_expr;
                typ = infer_expr(&mut self.infer_ctx, expr);
            }
        }

        typ
    }

    pub fn store_kind(&mut self) -> AutoResult<StoreKind> {
        match self.kind() {
            TokenKind::Var => Ok(StoreKind::Var),
            TokenKind::Let => Ok(StoreKind::Let),
            TokenKind::Const => Ok(StoreKind::Const),
            TokenKind::Mut => {
                let message = "'mut' is not supported as a storage modifier. Use 'var' for mutable variables.".to_string();
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
            _ => {
                let message = format!("Expected store kind (let or var), got {:?}", self.kind());
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
        }
    }

}
