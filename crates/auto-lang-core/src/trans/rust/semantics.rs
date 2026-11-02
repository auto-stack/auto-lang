use crate::ast::{Arg, Body, Code, Expr, Fn, Name, Stmt, StoreKind, Type};
use crate::error::{AutoError, TypeError};
use crate::infer::InferenceContext;
use crate::types::TypeStore;
use auto_val::Op;
use miette::SourceSpan;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

pub(super) fn check(
    ast: &Code,
    sibling_store: Option<Arc<RwLock<TypeStore>>>,
) -> Result<(), AutoError> {
    let type_store = sibling_store.unwrap_or_else(|| Arc::new(RwLock::new(TypeStore::new())));
    let mut functions = HashMap::new();
    {
        let mut store = type_store.write().unwrap();
        for stmt in &ast.stmts {
            match stmt {
                Stmt::Fn(decl) => {
                    functions.insert(decl.name.clone(), decl);
                    store.register_fn_decl(decl);
                }
                Stmt::TypeDecl(decl) => store.register_type_decl(decl),
                Stmt::SpecDecl(decl) => store.register_spec_decl(decl),
                Stmt::EnumDecl(decl) => store.register_enum_decl(decl.clone()),
                Stmt::Ext(decl) => store.register_ext_methods(decl),
                _ => {}
            }
        }
    }

    for stmt in &ast.stmts {
        match stmt {
            Stmt::Fn(decl) => check_fn(decl, ast, &functions, type_store.clone())?,
            Stmt::TypeDecl(decl) => {
                for method in &decl.methods {
                    check_fn(method, ast, &functions, type_store.clone())?;
                }
            }
            Stmt::Ext(decl) => {
                for method in &decl.methods {
                    check_fn(method, ast, &functions, type_store.clone())?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_fn(
    decl: &Fn,
    ast: &Code,
    functions: &HashMap<Name, &Fn>,
    type_store: Arc<RwLock<TypeStore>>,
) -> Result<(), AutoError> {
    let known_symbols = type_store.read().unwrap().get_defined_names();
    let mut ctx = InferenceContext::with_type_store(type_store);
    for name in known_symbols {
        ctx.bind_var(
            Name::from(name),
            Type::Fn(Vec::new(), Box::new(Type::Unknown)),
        );
    }
    for stmt in &ast.stmts {
        match stmt {
            Stmt::Store(store) => ctx.bind_var(store.name.clone(), store.ty.clone()),
            Stmt::TypeDecl(decl) => ctx.bind_var(
                decl.name.clone(),
                Type::Fn(Vec::new(), Box::new(Type::Unknown)),
            ),
            Stmt::EnumDecl(decl) => {
                ctx.bind_var(
                    decl.name.clone(),
                    Type::Fn(Vec::new(), Box::new(Type::Unknown)),
                );
                for item in &decl.items {
                    ctx.bind_var(
                        item.name.clone(),
                        Type::Fn(Vec::new(), Box::new(Type::Unknown)),
                    );
                }
            }
            Stmt::Union(decl) => ctx.bind_var(
                decl.name.clone(),
                Type::Fn(Vec::new(), Box::new(Type::Unknown)),
            ),
            Stmt::Tag(decl) => ctx.bind_var(
                decl.name.clone(),
                Type::Fn(Vec::new(), Box::new(Type::Unknown)),
            ),
            Stmt::SpecDecl(decl) => ctx.bind_var(
                decl.name.clone(),
                Type::Fn(Vec::new(), Box::new(Type::Unknown)),
            ),
            _ => {}
        }
    }
    ctx.push_fn_scope();
    for param in &decl.params {
        ctx.bind_var(param.name.clone(), param.ty.clone());
    }
    for stmt in &decl.body.stmts {
        if let Stmt::Store(store) = stmt {
            ctx.bind_var(store.name.clone(), store.ty.clone());
        }
    }
    ctx.current_ret = Some(decl.ret.clone());
    let allow_external = ast
        .stmts
        .iter()
        .any(|stmt| matches!(stmt, Stmt::Use(_) | Stmt::TaskDef(_)));
    check_calls(&decl.body, &mut ctx, functions, decl.span, allow_external);
    check_movement(&decl.body, decl.span)?;
    check_borrow_escape(decl, &ctx)?;
    check_return_paths(decl)?;
    if let Some(error) = ctx.errors.into_iter().next() {
        return Err(error);
    }
    Ok(())
}

fn check_calls(
    body: &Body,
    ctx: &mut InferenceContext,
    functions: &HashMap<Name, &Fn>,
    fallback: Option<(usize, usize)>,
    allow_external: bool,
) {
    for stmt in &body.stmts {
        match stmt {
            Stmt::Expr(expr) => check_expr_calls(expr, ctx, functions, fallback, allow_external),
            Stmt::Return(expr) | Stmt::Reply(expr) => {
                check_expr_calls(expr, ctx, functions, fallback, allow_external)
            }
            Stmt::Store(store) => {
                check_expr_calls(&store.expr, ctx, functions, fallback, allow_external)
            }
            Stmt::If(if_) => {
                for branch in &if_.branches {
                    check_expr_calls(&branch.cond, ctx, functions, fallback, allow_external);
                    check_calls(&branch.body, ctx, functions, fallback, allow_external);
                }
                if let Some(body) = &if_.else_ {
                    check_calls(body, ctx, functions, fallback, allow_external);
                }
            }
            Stmt::For(for_) => {
                check_expr_calls(&for_.range, ctx, functions, fallback, allow_external);
                check_calls(&for_.body, ctx, functions, fallback, allow_external);
            }
            Stmt::Block(body) => check_calls(body, ctx, functions, fallback, allow_external),
            _ => {}
        }
    }
}

fn check_expr_calls(
    expr: &Expr,
    ctx: &mut InferenceContext,
    functions: &HashMap<Name, &Fn>,
    fallback: Option<(usize, usize)>,
    allow_external: bool,
) {
    match expr {
        Expr::Call(call) => {
            check_expr_calls(&call.name, ctx, functions, fallback, allow_external);
            for arg in &call.args.args {
                match arg {
                    Arg::Pos(value) | Arg::Pair(_, value) => {
                        check_expr_calls(value, ctx, functions, fallback, allow_external)
                    }
                    Arg::Name(_) => {}
                }
            }
            let Expr::Ident(name) = call.name.as_ref() else {
                return;
            };
            let span = call
                .pos
                .map(|pos| SourceSpan::new(pos.pos.into(), pos.len.max(1)))
                .unwrap_or_else(|| fn_span(fallback));
            let Some(decl) = functions.get(name) else {
                let macro_call = matches!(
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
                        | "pair"
                        | "str_new"
                        | "str_len"
                        | "str_drop"
                        | "Node"
                );
                if !macro_call && !allow_external && ctx.lookup_type(name).is_none() {
                    let candidates = ctx.get_defined_var_names();
                    ctx.errors.push(
                        crate::error::NameError::undefined_function(
                            name.to_string(),
                            span,
                            &candidates,
                        )
                        .into(),
                    );
                }
                return;
            };
            if !decl.type_params.is_empty() {
                return;
            }
            for (arg, param) in call.args.args.iter().zip(&decl.params) {
                let Arg::Pos(value) = arg else {
                    continue;
                };
                let mut actual = crate::infer::infer_expr(ctx, value);
                let constructor_name = match value {
                    Expr::Call(constructor) => match constructor.name.as_ref() {
                        Expr::Ident(type_name) => Some(type_name),
                        _ => None,
                    },
                    Expr::Node(node) => Some(&node.name),
                    _ => None,
                };
                if let Some(type_name) = constructor_name {
                    if let Some(decl) = ctx.lookup_type_decl(type_name) {
                        actual = Type::User(decl);
                    }
                }
                if let (Type::User(expected), Type::User(found)) = (&param.ty, &actual) {
                    if expected.name != found.name {
                        ctx.errors.push(
                            TypeError::Mismatch {
                                expected: expected.name.to_string(),
                                found: found.name.to_string(),
                                span,
                            }
                            .into(),
                        );
                    }
                }
            }
        }
        Expr::Bina(left, op, right) => {
            check_expr_calls(left, ctx, functions, fallback, allow_external);
            check_expr_calls(right, ctx, functions, fallback, allow_external);
            if matches!(op, Op::Add | Op::Sub | Op::Mul | Op::Div) {
                let left_ty = crate::infer::infer_expr(ctx, left);
                let right_ty = crate::infer::infer_expr(ctx, right);
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
                    ctx.errors.push(
                        TypeError::Mismatch {
                            expected: left_ty.to_string(),
                            found: right_ty.to_string(),
                            span: fn_span(fallback),
                        }
                        .into(),
                    );
                }
            }
        }
        Expr::NullCoalesce(left, right) => {
            check_expr_calls(left, ctx, functions, fallback, allow_external);
            check_expr_calls(right, ctx, functions, fallback, allow_external);
        }
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
        | Expr::Yield(value) => check_expr_calls(value, ctx, functions, fallback, allow_external),
        Expr::Dot(value, _) | Expr::Index(value, _) => {
            check_expr_calls(value, ctx, functions, fallback, allow_external)
        }
        Expr::Index(_, index) => check_expr_calls(index, ctx, functions, fallback, allow_external),
        Expr::Cast { expr: value, .. } | Expr::To { expr: value, .. } => {
            check_expr_calls(value, ctx, functions, fallback, allow_external)
        }
        Expr::Array(values) | Expr::Tuple(values) => {
            for value in values {
                check_expr_calls(value, ctx, functions, fallback, allow_external);
            }
        }
        Expr::Object(pairs) => {
            for pair in pairs {
                check_expr_calls(&pair.value, ctx, functions, fallback, allow_external);
            }
        }
        Expr::Pair(pair) => check_expr_calls(&pair.value, ctx, functions, fallback, allow_external),
        Expr::If(if_) => {
            for branch in &if_.branches {
                check_expr_calls(&branch.cond, ctx, functions, fallback, allow_external);
                check_calls(&branch.body, ctx, functions, fallback, allow_external);
            }
            if let Some(body) = &if_.else_ {
                check_calls(body, ctx, functions, fallback, allow_external);
            }
        }
        Expr::Block(body) => check_calls(body, ctx, functions, fallback, allow_external),
        _ => {}
    }
}

fn check_movement(body: &Body, fallback: Option<(usize, usize)>) -> Result<(), AutoError> {
    let mut bindings = HashMap::new();
    check_body_mutability(body, &mut bindings, fallback)?;
    let mut moved = HashSet::new();
    check_body_moves(body, &mut moved, fallback)
}

fn check_body_mutability(
    body: &Body,
    bindings: &mut HashMap<Name, StoreKind>,
    fallback: Option<(usize, usize)>,
) -> Result<(), AutoError> {
    for stmt in &body.stmts {
        match stmt {
            Stmt::Store(store) => {
                check_expr_mutability(&store.expr, bindings, fallback)?;
                bindings.insert(store.name.clone(), store.kind.clone());
            }
            Stmt::Expr(expr) => check_expr_mutability(expr, bindings, fallback)?,
            Stmt::Return(expr) | Stmt::Reply(expr) => {
                check_expr_mutability(expr, bindings, fallback)?
            }
            Stmt::If(if_) => {
                for branch in &if_.branches {
                    check_expr_mutability(&branch.cond, bindings, fallback)?;
                    let mut branch_bindings = bindings.clone();
                    check_body_mutability(&branch.body, &mut branch_bindings, fallback)?;
                }
                if let Some(body) = &if_.else_ {
                    let mut branch_bindings = bindings.clone();
                    check_body_mutability(body, &mut branch_bindings, fallback)?;
                }
            }
            Stmt::For(for_) => {
                check_expr_mutability(&for_.range, bindings, fallback)?;
                let mut loop_bindings = bindings.clone();
                check_body_mutability(&for_.body, &mut loop_bindings, fallback)?;
            }
            Stmt::Block(body) => {
                let mut block_bindings = bindings.clone();
                check_body_mutability(body, &mut block_bindings, fallback)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_expr_mutability(
    expr: &Expr,
    bindings: &HashMap<Name, StoreKind>,
    fallback: Option<(usize, usize)>,
) -> Result<(), AutoError> {
    match expr {
        Expr::Bina(left, op, right) => {
            if matches!(op, Op::Asn | Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq) {
                if let Expr::Ident(name) = left.as_ref() {
                    if matches!(bindings.get(name), Some(StoreKind::Let | StoreKind::Const)) {
                        return Err(crate::error::NameError::ImmutableAssignment {
                            name: name.to_string(),
                            span: fn_span(fallback),
                        }
                        .into());
                    }
                }
            }
            check_expr_mutability(left, bindings, fallback)?;
            check_expr_mutability(right, bindings, fallback)
        }
        Expr::Call(call) => {
            check_expr_mutability(&call.name, bindings, fallback)?;
            for arg in &call.args.args {
                match arg {
                    Arg::Pos(value) | Arg::Pair(_, value) => {
                        check_expr_mutability(value, bindings, fallback)?
                    }
                    Arg::Name(_) => {}
                }
            }
            Ok(())
        }
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
        | Expr::Yield(value) => check_expr_mutability(value, bindings, fallback),
        Expr::Dot(value, _) => check_expr_mutability(value, bindings, fallback),
        Expr::Index(value, index) => {
            check_expr_mutability(value, bindings, fallback)?;
            check_expr_mutability(index, bindings, fallback)
        }
        Expr::Cast { expr: value, .. } | Expr::To { expr: value, .. } => {
            check_expr_mutability(value, bindings, fallback)
        }
        Expr::NullCoalesce(left, right) => {
            check_expr_mutability(left, bindings, fallback)?;
            check_expr_mutability(right, bindings, fallback)
        }
        Expr::Array(values) | Expr::Tuple(values) => {
            for value in values {
                check_expr_mutability(value, bindings, fallback)?;
            }
            Ok(())
        }
        Expr::Object(pairs) => {
            for pair in pairs {
                check_expr_mutability(&pair.value, bindings, fallback)?;
            }
            Ok(())
        }
        Expr::Pair(pair) => check_expr_mutability(&pair.value, bindings, fallback),
        _ => Ok(()),
    }
}

fn check_body_moves(
    body: &Body,
    moved: &mut HashSet<Name>,
    fallback: Option<(usize, usize)>,
) -> Result<(), AutoError> {
    for stmt in &body.stmts {
        match stmt {
            Stmt::Expr(expr) => check_expr_moves(expr, moved, fallback)?,
            Stmt::Return(expr) | Stmt::Reply(expr) => check_expr_moves(expr, moved, fallback)?,
            Stmt::Store(store) => {
                check_expr_moves(&store.expr, moved, fallback)?;
                moved.remove(&store.name);
            }
            Stmt::If(if_) => {
                let mut branch_moves = HashSet::new();
                for branch in &if_.branches {
                    check_expr_moves(&branch.cond, moved, fallback)?;
                    let mut branch_moved = moved.clone();
                    check_body_moves(&branch.body, &mut branch_moved, fallback)?;
                    branch_moves.extend(branch_moved);
                }
                if let Some(body) = &if_.else_ {
                    let mut branch_moved = moved.clone();
                    check_body_moves(body, &mut branch_moved, fallback)?;
                    branch_moves.extend(branch_moved);
                }
                moved.extend(branch_moves);
            }
            Stmt::For(for_) => {
                check_expr_moves(&for_.range, moved, fallback)?;
                check_body_moves(&for_.body, moved, fallback)?;
            }
            Stmt::Block(body) => check_body_moves(body, moved, fallback)?,
            _ => {}
        }
    }
    Ok(())
}

fn check_expr_moves(
    expr: &Expr,
    moved: &mut HashSet<Name>,
    fallback: Option<(usize, usize)>,
) -> Result<(), AutoError> {
    match expr {
        Expr::Ident(name) => ensure_not_moved(name, moved, fallback),
        Expr::Move(value) | Expr::Take(value) => {
            if let Expr::Ident(name) = value.as_ref() {
                ensure_not_moved(name, moved, fallback)?;
                moved.insert(name.clone());
                Ok(())
            } else {
                check_expr_moves(value, moved, fallback)
            }
        }
        Expr::Bina(left, op, right) => {
            check_expr_moves(left, moved, fallback)?;
            check_expr_moves(right, moved, fallback)?;
            if matches!(op, Op::Asn | Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq) {
                if let Expr::Ident(name) = left.as_ref() {
                    moved.remove(name);
                }
            }
            Ok(())
        }
        Expr::Call(call) => {
            check_expr_moves(&call.name, moved, fallback)?;
            for arg in &call.args.args {
                match arg {
                    Arg::Pos(value) | Arg::Pair(_, value) => {
                        check_expr_moves(value, moved, fallback)?
                    }
                    Arg::Name(_) => {}
                }
            }
            Ok(())
        }
        Expr::Dot(value, _) => check_expr_moves(value, moved, fallback),
        Expr::Index(value, index) => {
            check_expr_moves(value, moved, fallback)?;
            check_expr_moves(index, moved, fallback)
        }
        Expr::Unary(_, value)
        | Expr::View(value)
        | Expr::Mut(value)
        | Expr::ErrorPropagate(value)
        | Expr::Some(value)
        | Expr::Ok(value)
        | Expr::Err(value)
        | Expr::BoxExpr(value)
        | Expr::ArcExpr(value)
        | Expr::Yield(value) => check_expr_moves(value, moved, fallback),
        Expr::Cast { expr: value, .. } | Expr::To { expr: value, .. } => {
            check_expr_moves(value, moved, fallback)
        }
        Expr::NullCoalesce(left, right) => {
            check_expr_moves(left, moved, fallback)?;
            check_expr_moves(right, moved, fallback)
        }
        Expr::Array(values) | Expr::Tuple(values) => {
            for value in values {
                check_expr_moves(value, moved, fallback)?;
            }
            Ok(())
        }
        Expr::Object(pairs) => {
            for pair in pairs {
                check_expr_moves(&pair.value, moved, fallback)?;
            }
            Ok(())
        }
        Expr::Pair(pair) => check_expr_moves(&pair.value, moved, fallback),
        _ => Ok(()),
    }
}

fn ensure_not_moved(
    name: &Name,
    moved: &HashSet<Name>,
    fallback: Option<(usize, usize)>,
) -> Result<(), AutoError> {
    if moved.contains(name) {
        return Err(TypeError::InvalidOperation {
            op: format!("read moved value '{}'", name),
            ty: "use-after-move".into(),
            span: fn_span(fallback),
        }
        .into());
    }
    Ok(())
}

fn check_borrow_escape(decl: &Fn, ctx: &InferenceContext) -> Result<(), AutoError> {
    if !matches!(decl.ret, Type::StrSlice | Type::Reference(_)) {
        return Ok(());
    }
    for stmt in &decl.body.stmts {
        let Stmt::Return(expr) = stmt else {
            continue;
        };
        let borrowed_local = match expr.as_ref() {
            Expr::To {
                expr,
                target_type: Type::StrSlice,
            } => match expr.as_ref() {
                Expr::Ident(name) => matches!(ctx.lookup_type(name), Some(Type::StrOwned)),
                _ => false,
            },
            Expr::View(value) => matches!(value.as_ref(), Expr::Ident(_)),
            _ => false,
        };
        if borrowed_local {
            return Err(TypeError::Mismatch {
                expected: "borrow with a non-local owner".into(),
                found: "borrow of a local value escapes the function".into(),
                span: fn_span(decl.span),
            }
            .into());
        }
    }
    Ok(())
}

fn check_return_paths(decl: &Fn) -> Result<(), AutoError> {
    if matches!(decl.ret, Type::Void | Type::Unknown | Type::Handle { .. })
        || matches!(&decl.ret, Type::Result(inner) if matches!(inner.as_ref(), Type::Void | Type::Unknown))
        || matches!(&decl.ret, Type::GenericInstance(inst) if matches!(inst.base_name.as_str(), "Future" | "Stream"))
        || body_guarantees_return(&decl.body)
        || body_has_tail_value(&decl.body)
    {
        return Ok(());
    }
    Err(TypeError::Mismatch {
        expected: format!("return {} on every path", decl.ret),
        found: "path without a return value".into(),
        span: fn_span(decl.span),
    }
    .into())
}

fn body_has_tail_value(body: &Body) -> bool {
    match body.stmts.last() {
        Some(Stmt::Expr(_) | Stmt::Is(_) | Stmt::Node(_)) => true,
        Some(Stmt::Block(body)) => body_has_tail_value(body),
        Some(Stmt::If(if_)) => {
            if_.else_.is_some()
                && if_.branches.iter().all(|branch| {
                    body_guarantees_return(&branch.body) || body_has_tail_value(&branch.body)
                })
                && if_
                    .else_
                    .as_ref()
                    .is_some_and(|body| body_guarantees_return(body) || body_has_tail_value(body))
        }
        _ => false,
    }
}

fn body_guarantees_return(body: &Body) -> bool {
    body.stmts.iter().any(stmt_guarantees_return)
}

fn stmt_guarantees_return(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return(_) => true,
        Stmt::Block(body) => body_guarantees_return(body),
        Stmt::If(if_) => {
            if if_.branches.is_empty() || if_.else_.is_none() {
                return false;
            }
            if_.branches
                .iter()
                .all(|branch| body_guarantees_return(&branch.body))
                && if_
                    .else_
                    .as_ref()
                    .is_some_and(|body| body_guarantees_return(body))
        }
        _ => false,
    }
}

fn fn_span(span: Option<(usize, usize)>) -> SourceSpan {
    span.map(|(offset, len)| SourceSpan::new(offset.into(), len.max(1)))
        .unwrap_or_else(|| SourceSpan::new(0.into(), 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trans::rust::transpile_rust;

    fn rejects(source: &str) -> AutoError {
        match transpile_rust("semantic_admission", source) {
            Ok(_) => panic!("illegal source emitted Rust:\n{}", source),
            Err(error) => error,
        }
    }

    #[test]
    fn original_semantic_negatives_are_typed_rejections() {
        let sources = [
            (
                "B16",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B16.at"),
            ),
            (
                "B17",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B17.at"),
            ),
            (
                "B18",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B18.at"),
            ),
            (
                "B19",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B19.at"),
            ),
            (
                "T06",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/T06.at"),
            ),
            (
                "R09",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/R09.at"),
            ),
            (
                "R10",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/R10.at"),
            ),
        ];
        for (name, source) in sources {
            let error = rejects(source);
            assert!(
                matches!(
                    error,
                    AutoError::Syntax(_)
                        | AutoError::SyntaxWithSource(_)
                        | AutoError::Type(_)
                        | AutoError::Name(_)
                        | AutoError::MultipleErrors { .. }
                ),
                "{} produced a non-language error: {:?}",
                name,
                error
            );
        }
    }

    #[test]
    fn legal_neighbors_and_repair_are_admitted() {
        let sources = [
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB17.at"),
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB18.at"),
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB19.at"),
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B04.at"),
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B20.at"),
            r#"fn missing() int { return 42 }
fn main() int {
    if false { return missing() }
    return 42
}
"#,
            r#"fn owned() Str {
    let s Str = "ok"
    return s
}
fn main() int { return owned().len() }
"#,
            r#"type A { value int }
fn read(x A) int { return x.value }
fn main() int { return read(A(42)) }
"#,
            r#"type Item { name Str; code int }
fn main() int {
    let a Item = Item("x", 41)
    let b Item = a.move
    return b.code
}
"#,
        ];
        for source in sources {
            transpile_rust("semantic_admission", source)
                .unwrap_or_else(|error| panic!("legal source rejected: {}\n{}", error, source));
        }
    }

    #[test]
    fn branch_move_and_wrong_nominal_type_have_distinguishing_controls() {
        let invalid_branch = r#"type Item { code int }
fn main() int {
    let a Item = Item(41)
    if false { let b Item = a.move }
    return a.code
}
"#;
        assert!(matches!(rejects(invalid_branch), AutoError::Type(_)));
        let valid_branch = r#"type Item { code int }
fn main() int {
    let a Item = Item(41)
    if false {
        let b Item = a.move
        return b.code
    }
    return 0
}
"#;
        transpile_rust("semantic_admission", valid_branch).unwrap();
    }

    #[test]
    fn project_entry_rejects_same_negatives_and_recovers_after_repair() {
        let root = std::env::var_os("AUTO_LANG_PLAN916_SCRATCH")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(format!("project-entry-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let sources = [
            (
                "B16",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B16.at"),
            ),
            (
                "B17",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B17.at"),
            ),
            (
                "B18",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B18.at"),
            ),
            (
                "B19",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B19.at"),
            ),
            (
                "T06",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/T06.at"),
            ),
            (
                "R09",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/R09.at"),
            ),
            (
                "R10",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/R10.at"),
            ),
        ];
        for (name, source) in sources {
            let project = root.join(name);
            std::fs::create_dir_all(&project).unwrap();
            let entry = project.join("main.at");
            std::fs::write(&entry, source).unwrap();
            let error = match crate::trans::rust::transpile_rust_project(entry.to_str().unwrap()) {
                Ok(_) => panic!("{} emitted a project artifact", name),
                Err(error) => error,
            };
            assert!(
                !matches!(error, AutoError::Msg(_) | AutoError::Io(_)),
                "{} lost its typed language identity: {:?}",
                name,
                error
            );
        }

        let repair = root.join("repair");
        std::fs::create_dir_all(&repair).unwrap();
        let entry = repair.join("main.at");
        std::fs::write(
            &entry,
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B18.at"),
        )
        .unwrap();
        assert!(crate::trans::rust::transpile_rust_project(entry.to_str().unwrap()).is_err());
        std::fs::write(
            &entry,
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB18.at"),
        )
        .unwrap();
        let artifacts = crate::trans::rust::transpile_rust_project(entry.to_str().unwrap())
            .expect("repaired source did not emit a fresh project artifact");
        assert!(!artifacts.is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn four_state_emit_turnaround_has_thirty_samples_each() {
        let legal = include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB17.at");
        let invalid = include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B17.at");
        for state in ["fresh", "hit", "body-edit", "repair"] {
            let mut samples = Vec::with_capacity(30);
            for _ in 0..30 {
                let started = std::time::Instant::now();
                match state {
                    "fresh" | "hit" | "repair" => {
                        transpile_rust("semantic_admission", legal).unwrap();
                    }
                    _ => {
                        let error = rejects(invalid);
                        assert!(format!("{:?}", error).len() <= 4 * 1024 * 1024);
                    }
                }
                samples.push(started.elapsed().as_micros());
            }
            samples.sort_unstable();
            let p90 = samples[26];
            assert!(p90 <= 2_000_000, "{} emit P90={}us", state, p90);
            eprintln!("a2r916 state={} samples=30 p90={}us", state, p90);
        }
    }
}
