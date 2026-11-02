use crate::ast::{
    Arg, Body, Code, Expr, Fn, If, Is, IsBranch, Iter, Name, Stmt, StoreKind, Type, UseKind,
};
use crate::error::{AutoError, TypeError};
use crate::infer::InferenceContext;
use std::collections::{HashMap, HashSet};

pub(super) fn check_fn(
    decl: &Fn,
    ast: &Code,
    functions: &HashMap<Name, &Fn>,
    ctx: InferenceContext,
) -> Result<(), AutoError> {
    let mut analysis = Analysis::new(ctx, functions, ast, decl.span);
    analysis.push_scope();
    for param in &decl.params {
        analysis.declare(param.name.clone(), param.ty.clone(), StoreKind::Var);
    }
    analysis.walk_body(&decl.body)?;
    analysis.pop_scope();
    super::check_return_paths(decl)
}

struct Analysis<'a> {
    ctx: InferenceContext,
    functions: &'a HashMap<Name, &'a Fn>,
    fallback: Option<(usize, usize)>,
    scopes: Vec<HashMap<Name, usize>>,
    kinds: HashMap<usize, StoreKind>,
    types: HashMap<usize, Type>,
    moved: HashSet<usize>,
    next_id: usize,
    external_names: HashSet<String>,
    allow_external: bool,
    has_task: bool,
    return_depth: usize,
}

impl<'a> Analysis<'a> {
    fn new(
        ctx: InferenceContext,
        functions: &'a HashMap<Name, &'a Fn>,
        ast: &Code,
        fallback: Option<(usize, usize)>,
    ) -> Self {
        let mut external_names = HashSet::new();
        let mut allow_external = false;
        let mut has_task = false;
        for stmt in &ast.stmts {
            match stmt {
                Stmt::Use(use_) => {
                    external_names.extend(use_.items.iter().map(|name| name.to_string()));
                    if let Some(name) = use_.paths.last() {
                        external_names.insert(name.to_string());
                    }
                    allow_external |= use_.is_wildcard || matches!(use_.kind, UseKind::C);
                }
                Stmt::TaskDef(_) => has_task = true,
                _ => {}
            }
        }
        Self {
            ctx,
            functions,
            fallback,
            scopes: Vec::new(),
            kinds: HashMap::new(),
            types: HashMap::new(),
            moved: HashSet::new(),
            next_id: 0,
            external_names,
            allow_external,
            has_task,
            return_depth: 0,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Name, ty: Type, kind: StoreKind) {
        let id = self.next_id;
        self.next_id += 1;
        self.scopes.last_mut().unwrap().insert(name, id);
        self.kinds.insert(id, kind);
        self.types.insert(id, ty);
    }

    fn lookup_id(&self, name: &Name) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn lookup_type(&self, name: &Name) -> Option<Type> {
        self.lookup_id(name)
            .and_then(|id| self.types.get(&id).cloned())
            .or_else(|| self.ctx.lookup_type(name))
    }

    fn walk_body(&mut self, body: &Body) -> Result<(), AutoError> {
        for stmt in &body.stmts {
            self.walk_stmt(stmt)?;
        }
        Ok(())
    }

    fn walk_scoped_body(&mut self, body: &Body) -> Result<(), AutoError> {
        self.push_scope();
        self.walk_body(body)?;
        self.pop_scope();
        Ok(())
    }

    fn walk_stmt(&mut self, stmt: &Stmt) -> Result<(), AutoError> {
        match stmt {
            Stmt::Expr(expr) => self.walk_expr(expr),
            Stmt::Return(expr) => {
                self.return_depth += 1;
                let result = self.walk_expr(expr);
                self.return_depth -= 1;
                result
            }
            Stmt::Reply(expr) => self.walk_expr(expr),
            Stmt::Store(store) => {
                self.walk_expr(&store.expr)?;
                let ty = if matches!(store.ty, Type::Unknown) {
                    crate::infer::infer_expr(&mut self.ctx, &store.expr)
                } else {
                    store.ty.clone()
                };
                self.declare(store.name.clone(), ty, store.kind.clone());
                Ok(())
            }
            Stmt::If(if_) => self.walk_if(if_),
            Stmt::For(for_) => {
                self.walk_expr(&for_.range)?;
                self.push_scope();
                match &for_.iter {
                    Iter::Named(name) => self.declare(name.clone(), Type::Unknown, StoreKind::Var),
                    Iter::Indexed(index, value) => {
                        self.declare(index.clone(), Type::Int, StoreKind::Var);
                        self.declare(value.clone(), Type::Unknown, StoreKind::Var);
                    }
                    Iter::Destructured(key, value) => {
                        self.declare(key.clone(), Type::Unknown, StoreKind::Var);
                        self.declare(value.clone(), Type::Unknown, StoreKind::Var);
                    }
                    Iter::Cond | Iter::Call(_) | Iter::Ever => {}
                }
                self.walk_body(&for_.body)?;
                self.pop_scope();
                Ok(())
            }
            Stmt::Block(body) => self.walk_scoped_body(body),
            Stmt::Try(try_) => {
                self.walk_scoped_body(&try_.body)?;
                self.push_scope();
                if let Some(name) = &try_.catch_param {
                    self.declare(Name::from(name.as_str()), Type::Unknown, StoreKind::Var);
                }
                self.walk_body(&try_.catch_body)?;
                self.pop_scope();
                if let Some(body) = &try_.finally_body {
                    self.walk_scoped_body(body)?;
                }
                Ok(())
            }
            Stmt::Is(is_) => self.walk_is(is_),
            Stmt::Fn(decl) => self.walk_fn(decl),
            Stmt::TypeDecl(decl) => {
                for method in &decl.methods {
                    self.walk_fn(method)?;
                }
                Ok(())
            }
            Stmt::EnumDecl(decl) => {
                if let crate::ast::EnumKind::Heterogeneous { methods, .. } = &decl.kind {
                    for method in methods {
                        self.walk_fn(method)?;
                    }
                }
                Ok(())
            }
            Stmt::Ext(decl) => {
                for method in &decl.methods {
                    self.walk_fn(method)?;
                }
                Ok(())
            }
            Stmt::TaskDef(decl) => {
                for (_, _, value) in &decl.state {
                    self.walk_expr(value)?;
                }
                if let Some(hook) = &decl.start_hook {
                    self.walk_fn(hook)?;
                }
                if let Some(hook) = &decl.stop_hook {
                    self.walk_fn(hook)?;
                }
                for (_, guard, body) in &decl.on_block.handlers {
                    if let Some(guard) = guard {
                        self.walk_expr(guard)?;
                    }
                    self.walk_scoped_body(body)?;
                }
                if let Some(body) = &decl.on_block.else_handler {
                    self.walk_scoped_body(body)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn walk_fn(&mut self, decl: &Fn) -> Result<(), AutoError> {
        self.push_scope();
        for param in &decl.params {
            self.declare(param.name.clone(), param.ty.clone(), StoreKind::Var);
        }
        self.walk_body(&decl.body)?;
        self.pop_scope();
        super::check_return_paths(decl)
    }

    fn walk_if(&mut self, if_: &If) -> Result<(), AutoError> {
        for branch in &if_.branches {
            self.walk_expr(&branch.cond)?;
            self.walk_scoped_body(&branch.body)?;
        }
        if let Some(body) = &if_.else_ {
            self.walk_scoped_body(body)?;
        }
        Ok(())
    }

    fn walk_is(&mut self, is_: &Is) -> Result<(), AutoError> {
        self.walk_expr(&is_.target)?;
        for branch in &is_.branches {
            self.push_scope();
            match branch {
                IsBranch::EqBranch(patterns, body) => {
                    for pattern in patterns {
                        self.walk_pattern(pattern)?;
                    }
                    self.walk_body(body)?;
                }
                IsBranch::IfBranch(expr, body) => {
                    self.walk_expr(expr)?;
                    self.walk_body(body)?;
                }
                IsBranch::ElseBranch(body) => self.walk_body(body)?,
            }
            self.pop_scope();
        }
        Ok(())
    }

    fn walk_pattern(&mut self, expr: &Expr) -> Result<(), AutoError> {
        match expr {
            Expr::OptionPattern(pattern) => {
                if let Some(name) = &pattern.binding {
                    self.declare(name.clone(), Type::Unknown, StoreKind::Var);
                }
            }
            Expr::ResultPattern(pattern) => {
                if let Some(name) = &pattern.binding {
                    self.declare(name.clone(), Type::Unknown, StoreKind::Var);
                }
                if let Some(inner) = &pattern.inner {
                    self.walk_pattern(inner)?;
                }
            }
            Expr::StructPattern(pattern) => {
                for field in &pattern.fields {
                    self.declare(field.binding.clone(), Type::Unknown, StoreKind::Var);
                }
            }
            Expr::Cover(pattern) => {
                let crate::ast::cover::Cover::Tag(pattern) = pattern;
                for name in &pattern.bindings {
                    self.declare(name.clone(), Type::Unknown, StoreKind::Var);
                }
            }
            _ => self.walk_expr(expr)?,
        }
        Ok(())
    }

    fn walk_expr(&mut self, expr: &Expr) -> Result<(), AutoError> {
        self.enter_expr(expr)?;
        match expr {
            Expr::Int(_)
            | Expr::Uint(_)
            | Expr::I8(_)
            | Expr::U8(_)
            | Expr::I64(_)
            | Expr::U64(_)
            | Expr::Byte(_)
            | Expr::Float(_, _)
            | Expr::Double(_, _)
            | Expr::Bool(_)
            | Expr::Char(_)
            | Expr::Str(_)
            | Expr::CStr(_)
            | Expr::Ident(_)
            | Expr::GenName(_)
            | Expr::Ref(_)
            | Expr::Nil
            | Expr::Null
            | Expr::None
            | Expr::OptionUncover(_)
            | Expr::ResultUncover(_)
            | Expr::StructPattern(_) => {}
            Expr::Unary(_, value)
            | Expr::View(value)
            | Expr::Mut(value)
            | Expr::Move(value)
            | Expr::Take(value)
            | Expr::ErrorPropagate(value)
            | Expr::Some(value)
            | Expr::Ok(value)
            | Expr::Err(value)
            | Expr::BoxExpr(value)
            | Expr::ArcExpr(value)
            | Expr::Yield(value)
            | Expr::Await { expr: value }
            | Expr::Go { expr: value } => self.walk_expr(value)?,
            Expr::Bina(left, _, right)
            | Expr::NullCoalesce(left, right)
            | Expr::Index(left, right) => {
                self.walk_expr(left)?;
                self.walk_expr(right)?;
            }
            Expr::Dot(value, _) => self.walk_expr(value)?,
            Expr::Range(range) => {
                self.walk_expr(&range.start)?;
                self.walk_expr(&range.end)?;
            }
            Expr::Array(values)
            | Expr::Tuple(values)
            | Expr::FStr(crate::ast::FStr { parts: values }) => {
                for value in values {
                    self.walk_expr(value)?;
                }
            }
            Expr::Pair(pair) => self.walk_expr(&pair.value)?,
            Expr::Object(pairs) => {
                for pair in pairs {
                    self.walk_expr(&pair.value)?;
                }
            }
            Expr::Block(body) => self.walk_scoped_body(body)?,
            Expr::If(if_) => self.walk_if(if_)?,
            Expr::Call(call) => {
                self.walk_expr(&call.name)?;
                for arg in &call.args.args {
                    match arg {
                        Arg::Pos(value) | Arg::Pair(_, value) => self.walk_expr(value)?,
                        Arg::Name(_) => {}
                    }
                }
            }
            Expr::Node(node) => {
                for arg in &node.args.args {
                    match arg {
                        Arg::Pos(value) | Arg::Pair(_, value) => self.walk_expr(value)?,
                        Arg::Name(_) => {}
                    }
                }
                self.walk_body(&node.body)?;
            }
            Expr::Lambda(decl) => self.walk_fn(decl)?,
            Expr::Closure(closure) => {
                self.push_scope();
                for param in &closure.params {
                    self.declare(
                        param.name.clone(),
                        param.ty.clone().unwrap_or(Type::Unknown),
                        StoreKind::Var,
                    );
                }
                self.walk_expr(&closure.body)?;
                self.pop_scope();
            }
            Expr::Hold(hold) => {
                self.walk_expr(&hold.path)?;
                self.push_scope();
                self.declare(hold.name.clone(), Type::Unknown, StoreKind::Var);
                self.walk_body(&hold.body)?;
                self.pop_scope();
            }
            Expr::Grid(grid) => {
                for arg in &grid.head.args {
                    match arg {
                        Arg::Pos(value) | Arg::Pair(_, value) => self.walk_expr(value)?,
                        Arg::Name(_) => {}
                    }
                }
                for row in &grid.data {
                    for value in row {
                        self.walk_expr(value)?;
                    }
                }
            }
            Expr::Cast { expr: value, .. } | Expr::To { expr: value, .. } => {
                self.walk_expr(value)?
            }
            Expr::TupleDestruct { expr: value, .. } => self.walk_expr(value)?,
            Expr::NavCall { path, params } => {
                self.walk_expr(path)?;
                for pair in params {
                    self.walk_expr(&pair.value)?;
                }
            }
            Expr::AsyncBlock { body, .. } => self.walk_scoped_body(body)?,
            Expr::Comptime(hash) => self.walk_expr(&hash.expr)?,
            Expr::Is(is_) => self.walk_is(is_)?,
            Expr::OptionPattern(_) => {}
            Expr::ResultPattern(pattern) => {
                if let Some(inner) = &pattern.inner {
                    self.walk_pattern(inner)?;
                }
            }
            Expr::Cover(_) | Expr::Uncover(_) => {}
        }
        self.exit_expr(expr)
    }

    fn enter_expr(&mut self, expr: &Expr) -> Result<(), AutoError> {
        if self.return_depth > 0 {
            self.check_borrow(expr)?;
        }
        match expr {
            Expr::Ident(name) => {
                if let Some(id) = self.lookup_id(name) {
                    if self.moved.contains(&id) {
                        return Err(TypeError::InvalidOperation {
                            op: format!("read moved value '{}'", name),
                            ty: "use-after-move".into(),
                            span: super::fn_span(self.fallback),
                        }
                        .into());
                    }
                }
            }
            Expr::Call(call) => self.check_call(call)?,
            Expr::Bina(left, op, right) => {
                if matches!(
                    op,
                    auto_val::Op::Asn
                        | auto_val::Op::AddEq
                        | auto_val::Op::SubEq
                        | auto_val::Op::MulEq
                        | auto_val::Op::DivEq
                ) {
                    if let Expr::Ident(name) = left.as_ref() {
                        if let Some(id) = self.lookup_id(name) {
                            if matches!(
                                self.kinds.get(&id),
                                Some(StoreKind::Let | StoreKind::Const)
                            ) {
                                return Err(crate::error::NameError::ImmutableAssignment {
                                    name: name.to_string(),
                                    span: super::fn_span(self.fallback),
                                }
                                .into());
                            }
                        }
                    }
                }
                if matches!(
                    op,
                    auto_val::Op::Add | auto_val::Op::Sub | auto_val::Op::Mul | auto_val::Op::Div
                ) {
                    let left_ty = crate::infer::infer_expr(&mut self.ctx, left);
                    let right_ty = crate::infer::infer_expr(&mut self.ctx, right);
                    let numeric = |ty: &Type| {
                        matches!(
                            ty,
                            Type::Byte
                                | Type::Int
                                | Type::Uint
                                | Type::USize
                                | Type::I64
                                | Type::U64
                                | Type::Float
                                | Type::Double
                        )
                    };
                    if (matches!(left_ty, Type::Bool) && numeric(&right_ty))
                        || (matches!(right_ty, Type::Bool) && numeric(&left_ty))
                    {
                        return Err(TypeError::Mismatch {
                            expected: left_ty.to_string(),
                            found: right_ty.to_string(),
                            span: super::fn_span(self.fallback),
                        }
                        .into());
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn exit_expr(&mut self, expr: &Expr) -> Result<(), AutoError> {
        match expr {
            Expr::Move(value) | Expr::Take(value) => {
                if let Expr::Ident(name) = value.as_ref() {
                    if let Some(id) = self.lookup_id(name) {
                        self.moved.insert(id);
                    }
                }
            }
            Expr::Bina(left, op, _) => {
                if matches!(
                    op,
                    auto_val::Op::Asn
                        | auto_val::Op::AddEq
                        | auto_val::Op::SubEq
                        | auto_val::Op::MulEq
                        | auto_val::Op::DivEq
                ) {
                    if let Expr::Ident(name) = left.as_ref() {
                        if let Some(id) = self.lookup_id(name) {
                            self.moved.remove(&id);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn check_call(&mut self, call: &crate::ast::Call) -> Result<(), AutoError> {
        let Expr::Ident(name) = call.name.as_ref() else {
            return Ok(());
        };
        let span = call
            .pos
            .map(|pos| miette::SourceSpan::new(pos.pos.into(), pos.len.max(1)))
            .unwrap_or_else(|| super::fn_span(self.fallback));
        let Some(decl) = self.functions.get(name) else {
            let builtin = matches!(
                name.as_str(),
                "assert"
                    | "assert_eq"
                    | "assert_ne"
                    | "print"
                    | "println"
                    | "eprint"
                    | "eprintln"
                    | "format"
                    | "panic"
                    | "todo"
                    | "unimplemented"
                    | "unreachable"
                    | "vec"
                    | "write"
                    | "writeln"
                    | "say"
                    | "printf"
                    | "not"
                    | "sleep_ms"
                    | "stream_next"
                    | "stream_is_done"
                    | "stream_close"
                    | "pair"
                    | "str_new"
                    | "str_len"
                    | "str_drop"
                    | "Node"
                    | "fetch_data"
                    | "read_text"
                    | "read_to_string"
                    | "include_str"
            );
            let generated_runtime = name.as_str().starts_with("cabi_");
            let dynamic_task =
                self.has_task && name.as_str().chars().next().is_some_and(char::is_uppercase);
            if !builtin
                && !generated_runtime
                && !dynamic_task
                && !self.allow_external
                && !self.external_names.contains(name.as_str())
                && self.lookup_id(name).is_none()
                && self.ctx.lookup_type(name).is_none()
            {
                let candidates = self.ctx.get_defined_var_names();
                return Err(crate::error::NameError::undefined_function(
                    name.to_string(),
                    span,
                    &candidates,
                )
                .into());
            }
            return Ok(());
        };
        if !decl.type_params.is_empty() {
            return Ok(());
        }
        for (arg, param) in call.args.args.iter().zip(&decl.params) {
            let Arg::Pos(value) = arg else {
                continue;
            };
            let mut actual = crate::infer::infer_expr(&mut self.ctx, value);
            if matches!(actual, Type::Unknown) {
                if let Expr::Ident(name) = value {
                    if let Some(ty) = self.lookup_type(name) {
                        actual = ty;
                    }
                }
            }
            let constructor_name = match value {
                Expr::Call(constructor) => match constructor.name.as_ref() {
                    Expr::Ident(type_name) => Some(type_name),
                    _ => None,
                },
                Expr::Node(node) => Some(&node.name),
                _ => None,
            };
            if let Some(type_name) = constructor_name {
                if let Some(decl) = self.ctx.lookup_type_decl(type_name) {
                    actual = Type::User(decl);
                }
            }
            if let (Type::User(expected), Type::User(found)) = (&param.ty, &actual) {
                if expected.name != found.name {
                    return Err(TypeError::Mismatch {
                        expected: expected.name.to_string(),
                        found: found.name.to_string(),
                        span,
                    }
                    .into());
                }
            }
        }
        Ok(())
    }

    fn check_borrow(&mut self, expr: &Expr) -> Result<(), AutoError> {
        let borrowed_local = match expr {
            Expr::To {
                expr,
                target_type: Type::StrSlice,
            } => match expr.as_ref() {
                Expr::Ident(name) => matches!(self.lookup_type(name), Some(Type::StrOwned)),
                _ => false,
            },
            Expr::View(value) => match value.as_ref() {
                Expr::Ident(name) => matches!(
                    self.lookup_type(name),
                    Some(Type::StrOwned | Type::User(_) | Type::List(_) | Type::GenericInstance(_))
                ),
                _ => false,
            },
            _ => false,
        };
        if borrowed_local {
            return Err(TypeError::Mismatch {
                expected: "borrow with a non-local owner".into(),
                found: "borrow of a local value escapes the function".into(),
                span: super::fn_span(self.fallback),
            }
            .into());
        }
        Ok(())
    }
}
