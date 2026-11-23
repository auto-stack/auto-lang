//! PLAN-1124: which names a program references, for the standard library
//! grafts whose public names are ordinary words (`parse`, `encode`,
//! `minify`).
//!
//! A module is grafted only when the program references one of its public
//! names *and* does not declare that name itself: the program's own
//! `fn parse` or `type JsonValue` is the program's, and a word inside a
//! comment or a string literal is not a reference. The walk is exhaustive on
//! purpose — a new AST variant has to be classified here instead of passing
//! unnoticed.

use std::collections::BTreeSet;

use super::ast::{
    AggArmBody, AggMatchArm, Arg, Block, ClosureBody, ConformRef, EnumArm, Expr, ExprKind, FnItem,
    Ident, Item, MatchArm, Param, PlaceStep, Program, Stmt, TaskPattern, TypeAnn, TypeItem,
    TypeParamDecl, TypeRef,
};

/// Every name the program uses as a value, a call target, a type or a spec.
pub(crate) fn referenced_names(program: &Program) -> BTreeSet<String> {
    let mut refs = Refs {
        names: BTreeSet::new(),
    };
    for item in &program.items {
        refs.item(item);
    }
    refs.names
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
    names: BTreeSet<String>,
}

impl Refs {
    fn ident(&mut self, id: &Ident) {
        self.names.insert(id.name.clone());
    }

    fn params(&mut self, params: &[Param]) {
        for param in params {
            self.type_ann(&param.ty);
            if let Some(default) = &param.default {
                self.expr(default);
            }
        }
    }

    fn type_params(&mut self, params: &[TypeParamDecl]) {
        for param in params {
            if let Some(bound) = &param.bound {
                self.ident(bound);
            }
        }
    }

    fn conform(&mut self, conform: &[ConformRef]) {
        for claim in conform {
            self.ident(&claim.name);
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
            TypeRef::TyVar(id) | TypeRef::Record(id) => self.ident(id),
            TypeRef::Optional(inner) | TypeRef::ResultBang(inner) | TypeRef::Stream(inner) => {
                self.type_ann(inner);
            }
            TypeRef::Generic { name, args } => {
                self.ident(name);
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

    fn arms(&mut self, arms: &[AggMatchArm]) {
        for arm in arms {
            if let Some(guard) = &arm.guard {
                self.expr(guard);
            }
            match &arm.body {
                AggArmBody::Block(block) => self.block(block),
                AggArmBody::Expr(expr) => self.expr(expr),
            }
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
            ExprKind::Var(id) => self.ident(id),
            ExprKind::Method { recv, args, .. } => {
                self.expr(recv);
                self.args(args);
            }
            ExprKind::Call { callee, args } => {
                self.ident(callee);
                self.args(args);
            }
            ExprKind::ConstCall {
                callee,
                types,
                consts: _,
                args,
            } => {
                self.ident(callee);
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
                self.ident(ty);
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
                self.ident(name);
                for arg in args {
                    self.type_ann(arg);
                }
            }
            ExprKind::Closure {
                params,
                result,
                body,
                ..
            } => {
                self.params(params);
                if let Some(result) = result {
                    self.type_ann(result);
                }
                match &**body {
                    ClosureBody::Expr(expr) => self.expr(expr),
                    ClosureBody::Block(block) => self.block(block),
                }
            }
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

    fn block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.stmt(stmt);
        }
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::ExprStmt { value, .. }
            | Stmt::Return { value, .. }
            | Stmt::Yield { value, .. } => {
                self.expr(value);
            }
            Stmt::Let { ann, value, .. } => {
                if let Some(ann) = ann {
                    self.type_ann(ann);
                }
                self.expr(value);
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
            Stmt::LetDestruct { value, .. } => self.expr(value),
            Stmt::Hold { body, .. } | Stmt::Loop { body, .. } => self.block(body),
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
            Stmt::IsMatch { arms, .. } => {
                for MatchArm { qual, body, .. } in arms {
                    if let Some(qual) = qual {
                        self.ident(qual);
                    }
                    self.block(body);
                }
            }
            Stmt::ForIn { iter, body, .. } => {
                self.expr(iter);
                self.block(body);
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
                    self.block(&catch.body);
                }
                if let Some(finally) = finally {
                    self.block(finally);
                }
            }
            Stmt::With { expr, body, .. } => {
                self.expr(expr);
                self.block(body);
            }
            Stmt::HashIf { then, els, .. } => {
                self.block(then);
                if let Some(els) = els {
                    self.block(els);
                }
            }
            Stmt::HashFor { body, .. } => self.block(body),
            Stmt::HashIs { arms, .. } => {
                for arm in arms {
                    self.block(&arm.body);
                }
            }
            Stmt::NestedFn { item } => self.fn_item(item),
        }
    }

    fn fn_item(&mut self, f: &FnItem) {
        self.type_params(&f.type_params);
        self.params(&f.params);
        self.type_ann(&f.ret);
        self.block(&f.body);
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
                    match &arm.pattern {
                        TaskPattern::Lit(lit) => self.expr(lit),
                        TaskPattern::Bind { ty, .. } => self.type_ann(ty),
                        TaskPattern::Named { .. } | TaskPattern::Wildcard(_) => {}
                    }
                    if let Some(guard) = &arm.guard {
                        self.expr(guard);
                    }
                    self.block(&arm.body);
                }
            }
            Item::Node(_) | Item::UseRs(_) | Item::UsePy(_) => {}
        }
    }
}
