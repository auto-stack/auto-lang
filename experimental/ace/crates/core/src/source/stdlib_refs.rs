//! PLAN-1124: which names a program references, for the standard library
//! grafts whose public names are ordinary words (`parse`, `encode`,
//! `minify`).
//!
//! A module is grafted only when the program references one of its public
//! names *and* does not declare that name itself. A reference is a name used
//! as a value, a call target, a type or a spec where no local binding of that
//! name is in scope: a word inside a comment or a string literal is not a
//! reference, and neither is a local variable, a parameter, a nested function
//! or a closure parameter that happens to carry a library name. The walk is
//! exhaustive on purpose — a new AST variant has to be classified here
//! instead of passing unnoticed.
//!
//! Binding is modeled conservatively in one direction. A name is bound only
//! where the binder surely binds it (parameters, `let` and `var` from their
//! declaration on, pattern and loop bindings inside their own block). Where
//! the rule could be read either way — a use before a later `let` in the same
//! block — the name counts as a reference, which at worst grafts a module the
//! program does not need; the adapter still compiles such a program without
//! the graft first and keeps that result when it succeeds.

use std::collections::BTreeSet;

use super::ast::{
    AggArmBody, AggMatchArm, Arg, Block, ClosureBody, ConformRef, EnumArm, Expr, ExprKind, FnItem,
    Ident, Item, MatchArm, MatchPattern, Param, PlaceStep, Program, Stmt, TaskPattern, TypeAnn,
    TypeItem, TypeParamDecl, TypeRef,
};

/// The names a program uses, in two widths.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct NameUse {
    /// Names used as a value or a call target where no local binding of that
    /// name is in scope, plus every name used as a type, a spec or an enum
    /// (locals never shadow those).
    pub referenced: BTreeSet<String>,
    /// Every name the program uses as a value, a call target, a type, a spec
    /// or an enum, local bindings included.
    pub mentioned: BTreeSet<String>,
}

/// Which names the program uses.
pub(crate) fn analyze(program: &Program) -> NameUse {
    let mut refs = Refs {
        use_: NameUse::default(),
        scopes: vec![BTreeSet::new()],
    };
    for item in &program.items {
        refs.item(item);
    }
    refs.use_
}

/// Names of the program's own top-level items (functions, records, enums, …).
pub(crate) fn declared_names(program: &Program) -> BTreeSet<String> {
    program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(f) => Some(&f.name),
            Item::Type(t) => Some(&t.name),
            Item::Enum(e) => Some(&e.name),
            Item::Const(c) => Some(&c.name),
            Item::Store(s) => Some(&s.name),
            Item::Task(t) => Some(&t.name),
            Item::Node(n) => Some(&n.name),
            Item::UseRs(_) | Item::UsePy(_) => None,
        })
        .map(|id| id.name.clone())
        .collect()
}

struct Refs {
    use_: NameUse,
    /// Local bindings in scope, the innermost scope last. The first entry is
    /// the module itself and stays empty: module-level names are read from
    /// `declared_names`.
    scopes: Vec<BTreeSet<String>>,
}

impl Refs {
    fn bind(&mut self, id: &Ident) {
        self.scopes
            .last_mut()
            .expect("the module scope is always present")
            .insert(id.name.clone());
    }

    fn bound(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }

    /// A name used as a value or a call target.
    fn value(&mut self, id: &Ident) {
        self.use_.mentioned.insert(id.name.clone());
        if !self.bound(&id.name) {
            self.use_.referenced.insert(id.name.clone());
        }
    }

    /// A name used as a type, a spec or an enum.
    fn type_name(&mut self, id: &Ident) {
        self.use_.mentioned.insert(id.name.clone());
        self.use_.referenced.insert(id.name.clone());
    }

    /// Walk with `bound` in a new scope.
    fn scoped<I>(&mut self, bound: I, walk: impl FnOnce(&mut Self))
    where
        I: IntoIterator<Item = Ident>,
    {
        self.scopes
            .push(bound.into_iter().map(|id| id.name).collect());
        walk(self);
        self.scopes.pop();
    }

    fn block(&mut self, block: &Block) {
        self.scoped(std::iter::empty(), |this| {
            for stmt in &block.stmts {
                this.stmt(stmt);
            }
        });
    }

    fn block_binding<I>(&mut self, block: &Block, bound: I)
    where
        I: IntoIterator<Item = Ident>,
    {
        self.scoped(bound, |this| {
            for stmt in &block.stmts {
                this.stmt(stmt);
            }
        });
    }

    /// Parameters are visible throughout the body; their types and defaults
    /// are read outside it. The caller supplies the scope.
    fn bind_params(&mut self, params: &[Param]) {
        for param in params {
            self.type_ann(&param.ty);
            if let Some(default) = &param.default {
                self.expr(default);
            }
        }
        for param in params {
            self.bind(&param.name);
        }
    }

    fn type_params(&mut self, params: &[TypeParamDecl]) {
        for param in params {
            if let Some(bound) = &param.bound {
                self.type_name(bound);
            }
        }
    }

    fn conform(&mut self, conform: &[ConformRef]) {
        for claim in conform {
            self.type_name(&claim.name);
            for arg in &claim.args {
                self.type_ann(arg);
            }
            for (_, ty) in &claim.assoc {
                self.type_ann(ty);
            }
        }
    }

    fn type_ann(&mut self, ty: &TypeAnn) {
        match &ty.kind {
            TypeRef::Builtin(_) | TypeRef::Inferred => {}
            TypeRef::TyVar(id) | TypeRef::Record(id) => self.type_name(id),
            TypeRef::Optional(inner) | TypeRef::ResultBang(inner) | TypeRef::Stream(inner) => {
                self.type_ann(inner);
            }
            TypeRef::Generic { name, args } => {
                self.type_name(name);
                for arg in args {
                    self.type_ann(arg);
                }
            }
            TypeRef::Tuple(parts) => {
                for part in parts {
                    self.type_ann(part);
                }
            }
            TypeRef::Function { params, result } => {
                for param in params {
                    self.type_ann(param);
                }
                self.type_ann(result);
            }
        }
    }

    fn args(&mut self, args: &[Arg]) {
        for arg in args {
            match arg {
                Arg::Pos(value) | Arg::MovePos(value) => self.expr(value),
                Arg::Named { value, .. } => self.expr(value),
            }
        }
    }

    /// The payload bindings of an `is` arm, in source order. The arm's own
    /// name is not a variable, and its qualification names an enum.
    fn pattern(&mut self, pattern: &MatchPattern) -> Vec<Ident> {
        match pattern {
            MatchPattern::Variant { qual, bindings, .. } => {
                if let Some(qual) = qual {
                    self.type_name(qual);
                }
                bindings.clone()
            }
            MatchPattern::IntLit(_) | MatchPattern::BoolLit(_) | MatchPattern::Else => Vec::new(),
        }
    }

    fn arms(&mut self, arms: &[AggMatchArm]) {
        for arm in arms {
            let bound = self.pattern(&arm.pat);
            self.scoped(bound, |this| {
                if let Some(guard) = &arm.guard {
                    this.expr(guard);
                }
                match &arm.body {
                    AggArmBody::Block(block) => this.block(block),
                    AggArmBody::Expr(expr) => this.expr(expr),
                }
            });
        }
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::IntLit(_)
            | ExprKind::TypedLit { .. }
            | ExprKind::BoolLit(_)
            | ExprKind::NilLit
            | ExprKind::StrLit(_)
            | ExprKind::CharLit(_)
            | ExprKind::TypeWordArg(_) => {}
            ExprKind::TupleLit(parts) | ExprKind::ArrayLit(parts) => {
                for part in parts {
                    self.expr(part);
                }
            }
            ExprKind::GridLit { rows, .. } => {
                for row in rows {
                    for cell in row {
                        self.expr(cell);
                    }
                }
            }
            ExprKind::ArrayConditionalElem { cond, value } => {
                self.expr(cond);
                self.expr(value);
            }
            ExprKind::Index { base, index } => {
                self.expr(base);
                self.expr(index);
            }
            ExprKind::PySlice { base, bounds } => {
                self.expr(base);
                for bound in bounds.iter().flatten() {
                    self.expr(bound);
                }
            }
            ExprKind::ObjectLit { fields } => {
                for field in fields {
                    self.expr(&field.value);
                }
            }
            ExprKind::TextDisplay(inner)
            | ExprKind::Await(inner)
            | ExprKind::Go(inner)
            | ExprKind::Unary { operand: inner, .. }
            | ExprKind::HashExpr { expr: inner }
            | ExprKind::Propagate { inner } => self.expr(inner),
            ExprKind::Range { start, end, .. } => {
                self.expr(start);
                self.expr(end);
            }
            ExprKind::Var(id) => self.value(id),
            ExprKind::Method { recv, args, .. } => {
                self.expr(recv);
                self.args(args);
            }
            ExprKind::Call { callee, args } => {
                self.value(callee);
                self.args(args);
            }
            ExprKind::ConstCall {
                callee,
                types,
                consts: _,
                args,
            } => {
                self.value(callee);
                for ty in types {
                    self.type_ann(ty);
                }
                self.args(args);
            }
            ExprKind::CallValue { callee, args } => {
                self.expr(callee);
                self.args(args);
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            ExprKind::CtorNamed { ty, args, fields } => {
                self.type_name(ty);
                for arg in args {
                    self.type_ann(arg);
                }
                for field in fields {
                    self.expr(&field.value);
                }
            }
            ExprKind::Field { base, .. } => self.expr(base),
            ExprKind::MethodCall {
                recv,
                args,
                instance,
                ..
            } => {
                self.expr(recv);
                self.args(args);
                if let Some(instance) = instance {
                    for ty in &instance.types {
                        self.type_ann(ty);
                    }
                }
            }
            ExprKind::ViewArg { .. } | ExprKind::MutArg { .. } => {}
            ExprKind::GenericType { name, args } => {
                self.type_name(name);
                for arg in args {
                    self.type_ann(arg);
                }
            }
            ExprKind::Closure {
                params,
                result,
                body,
                ..
            } => self.scoped(std::iter::empty(), |this| {
                this.bind_params(params);
                if let Some(result) = result {
                    this.type_ann(result);
                }
                match &**body {
                    ClosureBody::Expr(expr) => this.expr(expr),
                    ClosureBody::Block(block) => {
                        for stmt in &block.stmts {
                            this.stmt(stmt);
                        }
                    }
                }
            }),
            ExprKind::UnwrapOr { inner, fallback } => {
                self.expr(inner);
                self.expr(fallback);
            }
            ExprKind::IsExpr { scrutinee, arms } => {
                self.expr(scrutinee);
                self.arms(arms);
            }
        }
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::ExprStmt { value, .. }
            | Stmt::Return { value, .. }
            | Stmt::Yield { value, .. } => {
                self.expr(value);
            }
            // `let` and `var` bind from their declaration on.
            Stmt::Let {
                name, ann, value, ..
            } => {
                if let Some(ann) = ann {
                    self.type_ann(ann);
                }
                self.expr(value);
                self.bind(name);
            }
            Stmt::Assign { value, .. } => self.expr(value),
            Stmt::IndexAssign(assign) => {
                self.expr(&assign.index);
                self.expr(&assign.value);
            }
            Stmt::PlaceAssign(assign) => {
                for step in &assign.steps {
                    if let PlaceStep::Index(index) = step {
                        self.expr(index);
                    }
                }
                self.expr(&assign.value);
            }
            Stmt::LetDestruct { names, value, .. } => {
                self.expr(value);
                for name in names {
                    self.bind(name);
                }
            }
            Stmt::Hold {
                name, bind, body, ..
            } => {
                self.value(name);
                self.block_binding(body, [bind.clone()]);
            }
            Stmt::Loop { body, .. } => self.block(body),
            Stmt::If {
                cond, then, els, ..
            } => {
                self.expr(cond);
                self.block(then);
                if let Some(els) = els {
                    self.block(els);
                }
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::IsMatch {
                scrutinee, arms, ..
            } => {
                self.value(scrutinee);
                for MatchArm {
                    qual,
                    binding,
                    body,
                    ..
                } in arms
                {
                    if let Some(qual) = qual {
                        self.type_name(qual);
                    }
                    self.block_binding(body, binding.iter().cloned());
                }
            }
            Stmt::ForIn {
                var, iter, body, ..
            } => {
                self.expr(iter);
                self.block_binding(body, [var.clone()]);
            }
            Stmt::AggIsMatch {
                scrutinee, arms, ..
            } => {
                self.expr(scrutinee);
                self.arms(arms);
            }
            Stmt::Try {
                body,
                catch,
                finally,
                ..
            } => {
                self.block(body);
                if let Some(catch) = catch {
                    self.block_binding(&catch.body, catch.bind.iter().cloned());
                }
                if let Some(finally) = finally {
                    self.block(finally);
                }
            }
            Stmt::With {
                expr, bind, body, ..
            } => {
                self.expr(expr);
                self.block_binding(body, bind.iter().cloned());
            }
            Stmt::HashIf { then, els, .. } => {
                self.block(then);
                if let Some(els) = els {
                    self.block(els);
                }
            }
            // The `#for` variable lives in the compile-time domain only; a
            // runtime use of its name is an ordinary undefined name.
            Stmt::HashFor { body, .. } => self.block(body),
            Stmt::HashIs { arms, .. } => {
                for arm in arms {
                    self.block(&arm.body);
                }
            }
            // A nested function is visible where it is declared, as a call
            // target only.
            Stmt::NestedFn { item } => {
                self.bind(&item.name);
                self.fn_item(item);
            }
        }
    }

    fn fn_item(&mut self, f: &FnItem) {
        self.scoped(std::iter::empty(), |this| {
            this.type_params(&f.type_params);
            this.bind_params(&f.params);
            this.type_ann(&f.ret);
            for stmt in &f.body.stmts {
                this.stmt(stmt);
            }
        });
    }

    fn type_item(&mut self, t: &TypeItem) {
        self.type_params(&t.type_params);
        if let Some(instance) = &t.instance {
            for arg in &instance.arguments {
                self.type_ann(arg);
            }
        }
        self.conform(&t.conforms);
        for field in &t.fields {
            self.type_ann(&field.ty);
        }
        for method in &t.methods {
            self.fn_item(method);
        }
    }

    fn item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => self.fn_item(f),
            Item::Type(t) => self.type_item(t),
            Item::Enum(e) => {
                self.type_params(&e.type_params);
                if let Some(instance) = &e.instance {
                    for arg in instance.arguments.iter().flatten() {
                        self.type_ann(arg);
                    }
                }
                for EnumArm {
                    payload, fields, ..
                } in &e.arms
                {
                    if let Some(payload) = payload {
                        self.type_ann(payload);
                    }
                    for (_, ty) in fields {
                        self.type_ann(ty);
                    }
                }
                for method in &e.methods {
                    self.fn_item(method);
                }
            }
            Item::Const(c) => {
                self.type_ann(&c.ty);
                self.expr(&c.value);
            }
            Item::Store(s) => {
                self.type_ann(&s.ty);
                self.expr(&s.value);
            }
            Item::Task(t) => {
                for field in &t.fields {
                    self.expr(&field.default);
                }
                for hook in [&t.start, &t.stop].into_iter().flatten() {
                    self.fn_item(hook);
                }
                for arm in &t.arms {
                    let bound = match &arm.pattern {
                        TaskPattern::Lit(lit) => {
                            self.expr(lit);
                            Vec::new()
                        }
                        TaskPattern::Bind { name, ty } => {
                            self.type_ann(ty);
                            vec![name.clone()]
                        }
                        TaskPattern::Named { bindings, .. } => bindings.clone(),
                        TaskPattern::Wildcard(_) => Vec::new(),
                    };
                    self.scoped(bound, |this| {
                        if let Some(guard) = &arm.guard {
                            this.expr(guard);
                        }
                        for stmt in &arm.body.stmts {
                            this.stmt(stmt);
                        }
                    });
                }
            }
            Item::Node(_) | Item::UseRs(_) | Item::UsePy(_) => {}
        }
    }
}
