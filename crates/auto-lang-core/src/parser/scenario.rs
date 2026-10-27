//! PLAN-843 结构拆分（自 parser.rs 原样搬移；算法/行为零变化）。
use super::*;

impl<'a> Parser<'a> {
    /// Parse task definition: `#[single] task Name { ... }`
    ///
    /// ```auto
    /// #[single]
    /// task CounterTask {
    ///     count mut = 0
    ///
    ///     fn start() ! { self.count = 0 }
    ///     fn stop() ! { print("stopping") }
    ///
    ///     on {
    ///         Add(val) => { self.count += val }
    ///         Reset => { self.count = 0 }
    ///         else => { }
    ///     }
    /// }
    /// ```
    pub fn parse_task(&mut self) -> AutoResult<Stmt> {
        self.parse_task_with_attrs(Vec::new())
    }

    /// Parse task with pre-parsed attributes (e.g., #[single])
    pub fn parse_task_with_attrs(&mut self, attrs: Vec<TaskAttr>) -> AutoResult<Stmt> {
        self.expect(TokenKind::Task)?;
        let name = self.parse_name()?;
        let pos = self.prev.pos;

        let mut task = TaskDef::new(name.clone(), attrs, pos);

        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        // Plan 389 R3: task state fields are task-scoped members. Register
        // them in the parser scope so handler/hook bodies can reference them
        // (including f-string interpolation `${field}`) — previously check_symbol
        // raised E0201 for them, which body()'s error recovery re-surfaced as a
        // confusing E0007 "Expected term, got RBrace".
        self.infer_ctx.push_scope();

        // Parse task body
        while !self.is_kind(TokenKind::RBrace) {
            self.skip_empty_lines();
            if self.is_kind(TokenKind::RBrace) {
                break;
            }

            match self.kind() {
                TokenKind::Fn => {
                    // Parse lifecycle hook: fn start() ! { ... } or fn stop() ! { ... }
                    let hook_fn = self.parse_task_lifecycle_hook(&name)?;
                    let fn_name = hook_fn.name.as_str();
                    if fn_name == "start" {
                        task.set_start_hook(hook_fn);
                    } else if fn_name == "stop" {
                        task.set_stop_hook(hook_fn);
                    } else {
                        return Err(SyntaxError::Generic {
                            message: format!("Unknown lifecycle hook '{}'. Only 'start' and 'stop' are allowed in task.", fn_name),
                            span: pos_to_span(self.prev.pos),
                        }.into());
                    }
                }
                TokenKind::On => {
                    // Parse on block: on { ... }
                    let on_block = self.parse_task_on_block()?;
                    task.on_block = on_block;
                }
                TokenKind::Ident => {
                    // Parse state field: name [mut] = expr
                    let (field_name, mutable, init_expr) = self.parse_task_state_field()?;
                    // Plan 389 R3: bind the field name (type unknown — exists()
                    // only checks name presence) so it resolves in check_symbol.
                    self.infer_ctx
                        .bind_var(field_name.clone(), crate::ast::Type::Unknown);
                    task.add_state(field_name, mutable, init_expr);
                }
                _ => {
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "Expected state field, lifecycle hook, or on block in task, got {:?}",
                            self.kind()
                        ),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
            self.skip_empty_lines();
        }

        let close_brace = self.expect(TokenKind::RBrace);
        self.infer_ctx.pop_scope();
        close_brace?;

        // Register task in scope
        self.define(name.as_str(), Meta::Task(task.clone()));

        // Plan 387 W4: register message-variant names (from the `on` block) as
        // symbols so that `h.send(Reset)` / `h.send(Add(5))` resolve in the
        // name-checker. Without this, bare `Reset` / `Add` are "undefined variable"
        // (parser.rs:9908). The a2r transpiler rewrites these to `<Task>Msg::Variant`
        // at transpile time; here we only satisfy name resolution. Mirrors how
        // register_enum_decl registers variants (parser.rs:4954-4965).
        for (pattern, _guard, _body) in &task.on_block.handlers {
            use crate::ast::TaskMsgPattern as P;
            let vname: Option<&crate::ast::Name> = match pattern {
                P::Simple(n) => Some(n),
                P::WithBindings { variant, .. } => Some(variant),
                _ => None,
            };
            if let Some(vn) = vname {
                // Only register if not already defined (avoid clobbering real symbols).
                if !self.exists(vn) {
                    let store = Store {
                        kind: StoreKind::Let,
                        attrs: vec![],
                        is_pub: false,
                        name: vn.clone(),
                        ty: Type::Int,
                        expr: Expr::Int(0),
                    };
                    self.define(vn.as_str(), Meta::Store(store));
                }
            }
        }

        Ok(Stmt::TaskDef(task))
    }

    /// Parse task state field: `name [mut] = expr`
    fn parse_task_state_field(&mut self) -> AutoResult<(Name, bool, Expr)> {
        let name = self.parse_name()?;
        let _pos = self.prev.pos;

        // Check for 'mut' keyword
        let mutable = if self.is_kind(TokenKind::Mut) {
            self.next();
            true
        } else {
            false
        };

        // Expect '='
        self.expect(TokenKind::Asn)?;

        // Parse initial value expression
        let init_expr = self.parse_expr()?;

        Ok((name, mutable, init_expr))
    }

    /// Parse task lifecycle hook: `fn start() ! { ... }` or `fn stop() ! { ... }`
    fn parse_task_lifecycle_hook(&mut self, task_name: &Name) -> AutoResult<Fn> {
        self.next(); // skip 'fn'

        let fn_name = self.parse_name()?;

        // Parse parameters - lifecycle hooks have no parameters
        self.expect(TokenKind::LParen)?;
        self.expect(TokenKind::RParen)?;

        // Expect '!' postfix (async marker for lifecycle hooks)
        if !self.is_kind(TokenKind::Not) {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Lifecycle hook '{}' must have '!' postfix (e.g., fn {}() ! {{ ... }})",
                    fn_name, fn_name
                ),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }
        self.next(); // consume '!'

        // Skip empty lines before body
        self.skip_empty_lines();

        // Parse body
        let body = self.body()?;

        // Create Fn struct
        let hook = Fn::new(
            FnKind::Method, // Lifecycle hooks are methods on the task
            fn_name,
            Some(task_name.clone()),
            Vec::new(), // Lifecycle hooks have no params
            body,
            Type::Void,
        );

        Ok(hook)
    }

    /// Parse task on block: `on { Pattern => { ... } else => { ... } }`
    /// Phase 3 (Plan 125): Also supports `on(ctx) { ... }` with context parameter
    /// and guard expressions: `amount int if amount > 10000 => { ... }`
    fn parse_task_on_block(&mut self) -> AutoResult<TaskOnBlock> {
        self.expect(TokenKind::On)?;
        let pos = self.prev.pos;

        // Phase 3: Check for context parameter: on(ctx) or on { ... }
        let context_param = if self.is_kind(TokenKind::LParen) {
            self.next(); // consume '('
            let name = self.parse_name()?;
            self.expect(TokenKind::RParen)?;
            Some(name)
        } else {
            None
        };

        let mut on_block =
            TaskOnBlock::with_context_and_handlers(context_param, Vec::new(), None, pos);

        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        while !self.is_kind(TokenKind::RBrace) {
            self.skip_empty_lines();
            if self.is_kind(TokenKind::RBrace) {
                break;
            }

            // Check for 'else' handler
            if self.is_kind(TokenKind::Else) {
                self.next(); // consume 'else'
                self.expect(TokenKind::Arrow)?;
                let body = self.body()?;
                on_block.set_else(body);
            } else {
                // Parse message pattern: Name, Literal, or TypeBinding
                let pattern = self.parse_task_msg_pattern()?;

                // Plan 389 R1: register on-pattern binding names (TypeBinding /
                // WithBindings) in a per-handler scope so the guard and body can
                // reference them as expression variables — previously only
                // `is ev { ... }` worked, and only because parse_is skips symbol
                // checks for is-targets. The scope is popped even on parse error
                // so the scope stack stays balanced for body()'s recovery.
                self.infer_ctx.push_scope();
                self.bind_task_pattern_names(&pattern);
                let parsed = self.parse_on_handler_body();
                self.infer_ctx.pop_scope();
                let (guard, body) = parsed?;
                on_block.add_handler_with_guard(pattern, guard, body);
            }

            self.skip_empty_lines();
        }

        self.expect(TokenKind::RBrace)?;

        Ok(on_block)
    }

    /// Plan 389 R1: bind an on-pattern's binding names into the current infer
    /// scope so the handler body (and guard) can reference them as expression
    /// variables. TypeBinding binds the bound name with its declared type;
    /// WithBindings binds each tuple-pattern name as Unknown.
    fn bind_task_pattern_names(&mut self, pattern: &TaskMsgPattern) {
        use crate::ast::TaskMsgPattern as P;
        match pattern {
            P::TypeBinding { name, type_expr } => {
                self.infer_ctx.bind_var(name.clone(), (**type_expr).clone());
            }
            P::WithBindings { bindings, .. } => {
                for (b, bty) in bindings {
                    // Plan 387 follow-up P3: bind the declared type when given,
                    // so guard/body expressions see e.g. `val: String`.
                    let ty = bty.clone().unwrap_or(crate::ast::Type::Unknown);
                    self.infer_ctx.bind_var(b.clone(), ty);
                }
            }
            _ => {}
        }
    }

    /// Plan 389 R1: parse a single on-handler's guard + body. Called with the
    /// pattern-binding scope already pushed (the caller pops it afterwards, even
    /// on error, so the scope stack stays balanced under error recovery).
    fn parse_on_handler_body(&mut self) -> AutoResult<(Option<Expr>, Body)> {
        // Phase 3: Parse optional guard expression
        let guard = if self.is_kind(TokenKind::If) {
            self.next(); // consume 'if'
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::Arrow)?;
        let body = self.body()?;
        Ok((guard, body))
    }

    /// Parse task message pattern
    ///
    /// Phase 1/2 patterns:
    /// - `Reset` - Simple variant
    /// - `Add(val)` - Variant with bindings
    ///
    /// Phase 3 patterns (Plan 125):
    /// - `"ping"` - String literal
    /// - `404` - Integer literal
    /// - `true` / `false` - Boolean literal
    /// - `msg string` - Type binding (identifier followed by type)
    fn parse_task_msg_pattern(&mut self) -> AutoResult<TaskMsgPattern> {
        use crate::ast::LiteralValue;

        // Phase 3: Check for literal patterns first
        match &self.cur.kind {
            TokenKind::Str => {
                let s = self.cur.text.clone();
                let lit = LiteralValue::String(s);
                self.next();
                return Ok(TaskMsgPattern::Literal(lit));
            }
            TokenKind::Int | TokenKind::I8 => {
                // Parse as integer literal
                let n = self.cur.text.parse::<i64>().unwrap_or(0);
                let lit = LiteralValue::Int(n);
                self.next();
                return Ok(TaskMsgPattern::Literal(lit));
            }
            TokenKind::Uint | TokenKind::U8 | TokenKind::Byte => {
                // Parse as unsigned integer literal
                let n = self.cur.text.parse::<u64>().unwrap_or(0);
                let lit = LiteralValue::Uint(n);
                self.next();
                return Ok(TaskMsgPattern::Literal(lit));
            }
            TokenKind::True => {
                self.next();
                return Ok(TaskMsgPattern::Literal(LiteralValue::Bool(true)));
            }
            TokenKind::False => {
                self.next();
                return Ok(TaskMsgPattern::Literal(LiteralValue::Bool(false)));
            }
            _ => {}
        }

        // Phase 1/2: Parse identifier-based pattern
        let name = self.parse_name()?;
        let _pos = self.prev.pos;

        // Check for parentheses with bindings: Add(val)
        if self.is_kind(TokenKind::LParen) {
            self.next(); // consume '('
            let mut bindings: Vec<(Name, Option<crate::ast::Type>)> = Vec::new();

            // Parse bindings. Plan 387 follow-up P3: each binding may carry an
            // optional declared type — `Add(val: String)` / `Add(val String)` —
            // which drives the generated enum variant's payload type.
            while !self.is_kind(TokenKind::RParen) {
                let binding = self.parse_name()?;
                let mut bty = None;
                if self.is_kind(TokenKind::Colon) {
                    self.next(); // consume ':'
                }
                if self.is_type_name() {
                    bty = Some(self.parse_type()?);
                }
                bindings.push((binding, bty));

                if self.is_kind(TokenKind::Comma) {
                    self.next(); // consume ','
                } else if !self.is_kind(TokenKind::RParen) {
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "Expected ',' or ')' in message pattern, got {:?}",
                            self.kind()
                        ),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }

            self.expect(TokenKind::RParen)?;

            Ok(TaskMsgPattern::with_bindings(name, bindings))
        } else if self.is_type_name() {
            // Phase 3: Type binding pattern: msg string, u User
            // Next token is a type, so this is a TypeBinding pattern
            let type_expr = self.parse_type()?;
            Ok(TaskMsgPattern::TypeBinding {
                name,
                type_expr: Box::new(type_expr),
            })
        } else {
            // Simple variant: Reset
            Ok(TaskMsgPattern::simple(name))
        }
    }

    pub fn parse_node_body(&mut self) -> AutoResult<Body> {
        self.parse_body(true)
    }

    pub(super) fn parse_node(
        &mut self,
        name: &AutoStr,
        primary: Option<Expr>,
        args: Args,
        kind: &AutoStr,
    ) -> AutoResult<Node> {
        let mut node = Node::new(name.clone());

        if let Some(prime) = primary {
            match prime {
                Expr::Ident(id) => {
                    // define a variable for this node instance with id
                    self.define(id.as_str(), Meta::Node(node.clone()));
                    node.id = id;
                }
                _ => {
                    // Convert to unified API: add args before body props
                    node.add_arg_unified("content", prime.clone());
                }
            }
        }

        // optional kind argument - add using unified API
        if !kind.is_empty() {
            node.add_arg_unified("kind", Expr::Str(kind.into()));
        }

        // Add remaining args using unified API
        for arg in args.args {
            match arg {
                Arg::Pos(expr) => {
                    // Positional arg
                    node.add_pos_arg_unified(expr);
                }
                Arg::Name(pair_name) => {
                    // Named arg
                    node.add_name_arg_unified(pair_name);
                }
                Arg::Pair(key, value) => {
                    // Pair arg - check type before adding
                    // 查找类型定义进行类型检查
                    let typ = self.lookup_type(&node.name);

                    if let Type::User(type_decl) = &*typ.borrow() {
                        // 查找成员定义
                        if let Some(member) = type_decl.members.iter().find(|m| &m.name == &key) {
                            // 推断表达式类型
                            let value_ty = self.infer_type_expr(&value);

                            // 检查类型匹配（使用当前 token 位置作为近似位置）
                            // Plan 167: skip type checking when skip_check is set
                            if !self.skip_check {
                                if let Err(err) =
                                    check_field_type(member, &value_ty, pos_to_span(self.cur.pos))
                                {
                                    self.errors.push(err);
                                }
                            }
                        }
                    }

                    node.add_arg_unified(key, value);
                }
            }
        }

        if self.is_kind(TokenKind::Newline) || self.is_kind(TokenKind::Semi) {
        } else {
            if self.is_kind(TokenKind::LBrace) {
                node.body = self.parse_node_body()?;
            }
        }

        // check node type
        let typ = self.lookup_type(&node.name);
        node.typ = typ.clone();

        // Attach doc comments (///) collected during body parsing.
        // In .at files, /// inside a node's body describes that node itself.
        node.doc = self.take_docs();

        Ok(node)
    }

    // 节点实例和函数调用有类似语法：
    // 函数调用：
    // 1. hello(x, y)， 这个是函数调用
    // 2. hello(), 这个是参数为空的函数调用
    // 节点实例化：
    // 1. hello (x, y) { ... }， 这个是节点实例
    // 2. hello () { ... }， 这个是参数为空的节点实例
    // 3. hello {...}，当参数为空时，可以省略()。但不能省略{}，否则和函数调用就冲突了。
    // 4. hello(x, y) {}, 这个是子节点为空的节点实例
    // 5. hello () {}， 这个是参数为空，子节点也为空的节点实例
    // 6. hello {}， 上面的()也可以省略。
    // 7. hello name (x, y) { ... }， 这是新的带变量名称的语法
    // 8. hello name { ... } 参数可以省略
    // 总之，节点实例的关键特征是`{}`，而函数调用没有`{}`
    pub fn parse_node_or_call_stmt(&mut self) -> AutoResult<Stmt> {
        let expr = self.node_or_call_expr()?;
        match expr {
            Expr::Node(node) => Ok(Stmt::Node(node)),
            other => {
                // Plan 028 T14: `local.field = rhs`（及任何以 Ident 起头的二元/
                // 赋值语句）路由到此，但 node_or_call_expr 在 Dot 链后即返回，
                // `= rhs` 无人消费 → 续接 Pratt 解析补齐操作符右侧。
                let full = self.expr_pratt_with_left(other, 0)?;
                Ok(Stmt::Expr(full))
            }
        }
    }

    pub fn node_or_call_expr(&mut self) -> AutoResult<Expr> {
        // Handle 'not' as prefix unary operator for boolean negation
        // e.g., assert(not expr) -> assert(!expr)
        if self.cur.text == "not" {
            self.next(); // skip 'not'
            let inner = self.parse_expr()?;
            return Ok(Expr::Unary(Op::Not, Box::new(inner)));
        }

        // Parse identifier or generic type instance (e.g., List or List<int>)
        let name = self.cur.text.clone();
        self.next(); // skip the identifier
                     // Plan 410: prev is now the expression's first token — its position is
                     // the error span hint for check_symbol (UndefinedVariable must point at
                     // the identifier, not at the cursor after the whole expression).
        let start_pos = self.prev.pos;

        // Plan 408: 'grid' is lexed as Ident now, so the statement-level path
        // reaches this fn instead of the old TokenKind::Grid arm in parse_expr.
        // Detect the AutoData grid(...) literal textually: the ident is already
        // consumed, so the current token is the '(' — put the ident back so
        // grid() can consume it (mirrors the old keyword path). A bare `grid`
        // (no paren) falls through to node/ident parsing as usual.
        if name.as_str() == "grid" && self.is_kind(TokenKind::LParen) {
            self.lexer.push_token(self.cur.clone());
            self.cur = self.prev.clone();
            return Ok(Expr::Grid(self.grid()?));
        }

        // Plan 390 §14 L1: Smart pointer constructors Box(expr) and Arc(expr)
        // must be recognized in argument position too. `args()` routes Ident
        // args through this fn (not atom), so without this check `set(k, Arc(v))`
        // parses Arc(v) as a plain Expr::Call (→ "Arc(v)" in Rust) instead of
        // Expr::ArcExpr (→ "Arc::new(v)"). Mirrors atom()'s Box/Arc recognition.
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

        // Check if this is a generic type instance (e.g., List<int>, Heap<T>)
        let mut ident = if self.is_kind(TokenKind::Lt) && self.next_token_is_type() {
            // Parse as generic instance and create GenName expression
            self.expect(TokenKind::Lt)?;

            let mut args = Vec::new();
            args.push(self.parse_type()?);

            while self.cur.kind == TokenKind::Comma {
                self.next(); // Consume ','
                args.push(self.parse_type()?);
            }

            self.expect(TokenKind::Gt)?;

            // Plan 395: if a call `(` immediately follows `>`, this is a generic
            // CALL (`foo<Bar>(x)` turbofish), not a GenName type-instance value
            // (`List<int>`). Set pending_generic_args and return the bare ident —
            // the Pratt LParen arm (reached via expr_pratt_with_left below) builds
            // the Call with generic_args. `List<int>` (no paren) stays GenName.
            if self.is_kind(TokenKind::LParen) {
                self.pending_generic_args = args;
                Expr::Ident(name)
            } else {
                // Generate descriptive name: "List<int>", "Heap<T>", etc.
                let args_str = args
                    .iter()
                    .map(|t| t.unique_name().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let generic_name = format!("{}<{}>", name, args_str);

                Expr::GenName(generic_name.into())
            }
        } else {
            // Regular identifier
            Expr::Ident(name)
        };

        // Plan 056: Use Expr::Dot for semantic clarity
        let mut broke_for_pratt = false;
        while self.is_kind(TokenKind::Dot) {
            // Plan 162: Don't consume .as(Type) / .to(Type) — delegate to
            // Pratt parser.
            // PLAN-668 R-12④: .await/.go postfix 同理委托 Pratt 层
            // （expr_pratt_with_left 的 Await/Go 臂）——`work(1).await`
            // 嵌套位此前在本链落 "Expected identifier... got Await"。
            let next_is_postfix = if let Ok(tok) = self.lexer.next() {
                let is_special = matches!(
                    tok.kind,
                    TokenKind::As | TokenKind::To | TokenKind::Await | TokenKind::Go
                );
                self.lexer.push_token(tok);
                is_special
            } else {
                false
            };
            if next_is_postfix {
                broke_for_pratt = true;
                break;
            }
            self.next(); // skip dot
                         // Allow @, * as special field names for pointer operations
                         // Allow numeric literals for integer-keyed objects: a.1, a.2
                         // Allow boolean keywords: a.true, a.false
            let field_name = if self.is_kind(TokenKind::Ident)
                // 2026-08-22(tag 软关键字):.tag 字段访问
                || self.is_kind(TokenKind::Tag)
            {
                let text = self.cur.text.clone();
                self.next();
                text
            } else if self.is_kind(TokenKind::At) || self.is_kind(TokenKind::Star) {
                let text = self.cur.text.clone();
                self.next();
                text
            } else if self.is_kind(TokenKind::Int)
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
            // Allow .type property and .spawn method
            {
                let text = self.cur.text.clone();
                self.next();
                text
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected identifier, @, *, number, or boolean after dot, got {:?}",
                        self.cur.kind
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            };
            ident = Expr::Dot(Box::new(ident), field_name);
        }

        // Plan 162: If we broke the dot loop for .as/.to, delegate to Pratt parser
        // so it can handle the type cast/conversion properly
        if broke_for_pratt {
            return self.expr_pratt_with_left(ident, 0);
        }

        let is_constructor = match &ident {
            Expr::Ident(n) => {
                let meta = self.lookup_meta(n);
                if let Some(m) = meta {
                    match m.as_ref() {
                        Meta::Type(_) => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Expr::GenName(name) => {
                // Generic instance like "List<int>" or "Heap<T>"
                // Extract base type name (before the first '<')
                let name_str = name.as_str();
                let base_name = if let Some(pos) = name_str.find('<') {
                    &name_str[..pos]
                } else {
                    name_str
                };

                let meta = self.lookup_meta(base_name);
                if let Some(m) = meta {
                    match m.as_ref() {
                        Meta::Type(_) => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        let mut has_paren = false;

        // 节点实例的primary prop
        let primary_prop = if self.is_kind(TokenKind::Ident) {
            let id = self.ident_name()?;
            self.next();
            Some(Expr::Ident(id))
        } else if self.is_literal() {
            Some(self.literal()?)
        } else if self.is_kind(TokenKind::Dot) {
            // Handle .field as primary prop (e.g., Text .title)
            Some(self.dot_item()?)
        } else {
            None
        };

        let mut args = Args::new();
        // If has paren, maybe a call or node instance
        if self.is_kind(TokenKind::LParen) {
            args = self.args()?;
            has_paren = true;
        }

        // Parse secondary prop (kind): after primaryProp and args, before {
        // e.g., v CanTrcvBaudRate int { ... }
        //           ^primaryProp    ^secondaryProp
        let mut secondary_prop: AutoStr = AutoStr::new();
        if primary_prop.is_some() && !has_paren && self.is_kind(TokenKind::Ident) {
            let next_text = self.cur.text.clone();
            // Don't consume if it's actually the opening brace content
            // (Ident followed by { is not a secondaryProp, it's a new statement in body)
            // secondaryProp is a simple type-like identifier
            secondary_prop = next_text;
            self.next();
        }

        // If has brace, must be a node instance
        // NOTE: If ident is a Dot expression (e.g., TypeName.method), treat as call even if is_constructor=true
        // IMPORTANT: When is_constructor=true but no brace/primary_prop/paren, and next token is Colon,
        // this is a Pair expression (key: value), NOT a node instance.
        // Plan 442 C2 (Bug A): a paren-arg call followed by `.` is a method
        // CHAIN on the call's return value (e.g. musk backend's axum route
        // builders `get(h1).put(h2)`). use.rust fn imports register as
        // synthetic type decls (register_rust_type), so is_constructor is
        // true for them and this gate would route the call into node-instance
        // parsing — consuming `(h1)` as node args and leaving the chain dot
        // dangling ("expected argument separator, found Dot" from args()).
        // A node instance never chains, so a paren followed by `.` is a call.
        let paren_chained = has_paren && self.is_kind(TokenKind::Dot);
        let is_dot_call = matches!(ident, Expr::Dot(_, _));
        // Plan 442 C2 (Bug B, parser side): bare axum routing-fn calls
        // (`patch(h)` WITHOUT a chained dot) must also parse as calls.
        // use.rust items register as synthetic type decls
        // (register_rust_type), so is_constructor is true for them and the
        // node-instance gate below would swallow `(h)` as node args — the
        // chained form is already covered by paren_chained above; this arm
        // covers the standalone form.
        let is_axum_routing_fn = match &ident {
            Expr::Ident(n) => self
                .type_store
                .read()
                .map(|s| {
                    s.get_rust_type_path(n.as_str())
                        .map(|p| p.starts_with("axum::routing::"))
                        .unwrap_or(false)
                })
                .unwrap_or(false),
            _ => false,
        };
        let is_colon_pair = is_constructor
            && !is_dot_call
            && primary_prop.is_none()
            && !self.is_kind(TokenKind::LBrace)
            && self.is_kind(TokenKind::Colon);
        if (self.is_kind(TokenKind::LBrace)
            || primary_prop.is_some()
            || (is_constructor && !is_dot_call && !paren_chained && !is_axum_routing_fn))
            && !is_colon_pair
        {
            // node instance
            // with node instance, pair args also defines as properties
            for arg in &args.args {
                if let Arg::Pair(name, value) = arg {
                    self.define(
                        name.as_str(),
                        Meta::Pair(Pair {
                            key: Key::NamedKey(name.clone()),
                            value: Box::new(value.clone()),
                        }),
                    );
                }
            }

            // Plan 043 M5: a TYPE constructor in rhs position
            // (`var b Block = Block{ id: id, ... }`) must parse the braces as
            // STRUCT FIELDS (named args), not a node body. parse_node's
            // `{...}` → parse_node_body can't parse `field: value` as
            // statements, so fields were dropped and codegen emitted `{}`.
            // UI scenario only; the object() parse lives in a helper so this
            // hot fn's stack frame stays small (gdscript dodge_player barely
            // fits in the 2MB test stack).
            let node_args = if self.is_ui_scenario()
                && self.is_ui_struct_construction(&ident, is_constructor)
                && !has_paren
                && primary_prop.is_none()
                && self.is_kind(TokenKind::LBrace)
            {
                self.parse_braced_struct_args(&ident, args)?
            } else {
                args
            };
            match ident {
                Expr::Ident(name) => {
                    return Ok(Expr::Node(self.parse_node(
                        &name,
                        primary_prop,
                        node_args,
                        &secondary_prop,
                    )?));
                }
                _ => {
                    return Err(SyntaxError::Generic {
                        message: format!("Expected node name, got {:?}", ident),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
        } else {
            // no brace, might be a function call, constructor call or simple expression
            if has_paren {
                // Use the call result as the left side of a binary expression
                // and continue with Pratt parser to handle operators like +, -, etc.
                let call_expr = self.call(ident, args)?;
                self.expr_pratt_with_left(call_expr, 0)
            } else {
                // Something else with a starting Ident
                if primary_prop.is_some() {
                    if self.is_kind(TokenKind::Ident) {
                        let kind = self.parse_name()?;
                        if self.is_kind(TokenKind::LBrace)
                            || self.is_kind(TokenKind::Newline)
                            || self.is_kind(TokenKind::Semi)
                            || self.is_kind(TokenKind::RBrace)
                        {
                            let nd = self.parse_node(&ident.repr(), primary_prop, args, &kind)?;
                            return Ok(Expr::Node(nd));
                        } else {
                            return Err(SyntaxError::Generic {
                                message: format!(
                                    "expect node, got {} {} {}",
                                    ident.repr(),
                                    primary_prop.unwrap().repr(),
                                    kind
                                ),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }
                    } else {
                        // name id <nl>
                        if self.is_kind(TokenKind::Newline)
                            || self.is_kind(TokenKind::Semi)
                            || self.is_kind(TokenKind::RBrace)
                        {
                            let nd =
                                self.parse_node(&ident.repr(), primary_prop, args, &"".into())?;
                            return Ok(Expr::Node(nd));
                        }
                    }
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "Expected simple expression, got `{} {}`",
                            ident.repr(),
                            primary_prop.unwrap()
                        ),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
                let expr = self.expr_pratt_with_left(ident, 0)?;
                let expr = self.check_symbol(expr, pos_to_span(start_pos))?;
                Ok(expr)
            }
        }
    }

    pub(super) fn call(&mut self, ident: Expr, args: Args) -> AutoResult<Expr> {
        // Check for special builtin functions (Plan 105)
        if let Expr::Ident(name) = &ident {
            if name.as_str() == "nav" {
                return self.parse_nav_call(args);
            }
        }

        let ret_type = self.return_type(&ident)?;
        let call_pos = self.prev.pos;
        let expr = Expr::Call(Call {
            name: Box::new(ident),
            args,
            ret: ret_type,
            type_args: Vec::new(), // Plan 061: Will be filled in during type inference
            generic_args: Vec::new(), // Plan 395: explicit turbofish args
            pos: Some(call_pos),
        });
        // Plan 410: the Call branch of check_symbol emits no UndefinedVariable
        // error (the function-name check is disabled), so keep the original
        // cursor-based span here rather than threading a start position
        // through this fn's 3 callers for zero behavioral gain.
        self.check_symbol(expr, pos_to_span(self.cur.pos))
    }

    /// Parse nav() function call for router navigation (Plan 105)
    ///
    /// Syntax: nav("/path", id: 123, name: "john")
    /// - First argument: path string (required)
    /// - Remaining arguments: key-value pairs for route parameters
    fn parse_nav_call(&mut self, args: Args) -> AutoResult<Expr> {
        // First argument must be the path string
        let first_arg = args.first_arg();
        let path = match first_arg {
            Some(expr) => expr,
            None => {
                return Err(SyntaxError::Generic {
                    message: "nav() requires at least one argument (path)".to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        };

        // Collect named arguments as params
        let params: Vec<Pair> = args
            .args
            .iter()
            .skip(1)
            .filter_map(|arg| match arg {
                Arg::Pair(name, expr) => Some(Pair {
                    key: Key::NamedKey(name.clone()),
                    value: Box::new(expr.clone()),
                }),
                _ => None,
            })
            .collect();

        Ok(Expr::NavCall {
            path: Box::new(path),
            params,
        })
    }

    pub fn grid(&mut self) -> AutoResult<Grid> {
        self.next(); // skip grid
                     // args
        let mut data = Vec::new();
        let args = self.args()?;
        // data
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut row_index = 0;
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            let row = self.array()?;
            if let Expr::Array(array) = row {
                data.push(array);
            }
            let is_first = row_index == 0;
            self.expect_eos(is_first)?;
            row_index += 1;
        }
        self.expect(TokenKind::RBrace)?;
        let grid = Grid { head: args, data };
        Ok(grid)
    }

    pub fn parse_event_src(&mut self) -> AutoResult<Option<Expr>> {
        let src = if self.is_kind(TokenKind::Ident) {
            Some(self.parse_ident()?)
        } else if self.is_kind(TokenKind::Int) {
            Some(self.atom()?)
        } else if self.is_kind(TokenKind::Str) {
            Some(self.atom()?)
        } else if self.is_kind(TokenKind::FStrStart) {
            Some(self.fstr()?)
        } else if self.is_kind(TokenKind::Arrow) {
            None
        } else {
            None
        };

        Ok(src)
    }

    pub fn parse_cond_arrow(&mut self, src: Option<Expr>) -> AutoResult<CondArrow> {
        if self.is_kind(TokenKind::Question) {
            self.next(); // skip ?
            let cond = self.parse_expr()?;
            let subs = self.parse_arrow_list()?;
            return Ok(CondArrow::new(src, cond, subs));
        } else {
            return Err(format!("Expected condition arrow").into());
        }
    }

    /// Deffirent forms:
    /// 1. EV -> State : Handler
    /// 2. -> State : Handler
    /// 3. EV : Handler
    /// 4. EV -> State
    /// 5. EV ? ConditionCheck {
    ///       -> State1 : Handler1
    ///       -> State2 : handler2
    ///    }
    pub fn parse_arrow(&mut self, src: Option<Expr>) -> AutoResult<Arrow> {
        if self.is_kind(TokenKind::Colon) {
            self.next();
            let with = self.parse_expr()?;
            return Ok(Arrow::new(src, None, Some(with)));
        } else {
            self.expect(TokenKind::Arrow)?;
            let to = self.parse_expr()?;
            if let Expr::Pair(p) = to {
                let to = p.key.into();
                let value = *p.value;
                return Ok(Arrow::new(src, Some(to), Some(value)));
            } else {
                return Ok(Arrow::new(src, Some(to), None));
            }
        };
    }

    fn parse_arrow_list(&mut self) -> AutoResult<Vec<Arrow>> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut events = Vec::new();
        while !self.is_kind(TokenKind::RBrace) {
            let src = self.parse_event_src()?;
            events.push(self.parse_arrow(src)?);
            self.skip_empty_lines();
        }
        self.expect(TokenKind::RBrace)?;
        Ok(events)
    }

    pub fn parse_event(&mut self) -> AutoResult<Event> {
        let src = self.parse_event_src()?;
        // println!("NEXT I S {}", self.kind()); // LSP: disabled
        if self.is_kind(TokenKind::Arrow) || self.is_kind(TokenKind::Colon) {
            return Ok(Event::Arrow(self.parse_arrow(src)?));
        } else if self.is_kind(TokenKind::Question) {
            return Ok(Event::CondArrow(self.parse_cond_arrow(src)?));
        } else {
            return Err(format!("expected arrow, colon or question mark.").into());
        }
    }

    fn parse_goto_branch(&mut self) -> AutoResult<Event> {
        self.parse_event()
    }

    pub fn parse_on_events(&mut self) -> AutoResult<OnEvents> {
        // skip on
        self.expect(TokenKind::On)?;

        let mut branches = Vec::new();

        // multiple branches
        if self.is_kind(TokenKind::LBrace) {
            self.next(); // skip {
            self.skip_empty_lines();
            while !self.is_kind(TokenKind::RBrace) {
                self.skip_empty_lines();
                // println!("cchecking branch: {}", self.cur); // LSP: disabled
                branches.push(self.parse_goto_branch()?);
                self.skip_empty_lines();
            }
            self.expect(TokenKind::RBrace)?;
        } else {
            // single branch
            branches.push(self.parse_goto_branch()?);
        }
        Ok(OnEvents::new(branches))
    }

    // ========================================================================
    // Plan 096: UI Contextual Keyword Parsing
    // ========================================================================

    /// Parse widget declaration (UI scenario only)
    ///
    /// ```auto
    /// widget Counter {
    ///     msg Msg { Inc, Dec }
    ///     model { count int = 0 }
    ///     view { col { ... } }
    ///     on { .Inc => { .count += 1 } }
    /// }
    /// ```

    // ========================================================================
    // Plan 306: Godot scene declaration parsing
    // ========================================================================

    pub fn parse_scene_decl(&mut self) -> AutoResult<Stmt> {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        crate::scene_grammar::parse_scene_decl(&mut bridge)
    }

    pub(super) fn looks_like_scene_decl(&mut self) -> bool {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        crate::scene_grammar::looks_like_scene_decl(&mut bridge)
    }
}
