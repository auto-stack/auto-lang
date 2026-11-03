//! 片段绝对坐标平移（T-07）：片段在授权区间自身的快照上解析（0 基），
//! 投影后按 `range.start` 平移全部 span——与旧 parse_function_fragment
//! 的 token 偏移合同一致（绝对 UTF-8 byte span，不改源码序/结构）。

use crate::atom_text::{Diagnostic, Span};
use crate::source::ast::*;

pub(crate) fn shift_span(s: &mut Span, off: usize) {
    s.start += off;
    s.end += off;
}

pub(crate) fn shift_diag(d: &mut Diagnostic, off: usize) {
    shift_span(&mut d.span, off);
}

fn shift_ident(i: &mut Ident, off: usize) {
    shift_span(&mut i.span, off);
}

fn shift_type_ann(t: &mut TypeAnn, off: usize) {
    shift_span(&mut t.span, off);
    match &mut t.kind {
        TypeRef::Record(id) => shift_ident(id, off),
        TypeRef::Optional(inner) => shift_type_ann(inner, off),
        TypeRef::Generic { name, args } => {
            shift_ident(name, off);
            for a in args.iter_mut() {
                shift_type_ann(a, off);
            }
        }
        TypeRef::Function { params, result } => {
            for p in params.iter_mut() {
                shift_type_ann(p, off);
            }
            shift_type_ann(result, off);
        }
        TypeRef::Builtin(_) => {}
    }
}

fn shift_param(p: &mut Param, off: usize) {
    shift_ident(&mut p.name, off);
    shift_type_ann(&mut p.ty, off);
    shift_span(&mut p.span, off);
}

fn shift_arg(a: &mut Arg, off: usize) {
    match a {
        Arg::Pos(e) | Arg::MovePos(e) => shift_expr(e, off),
        Arg::Named { name, value, span } => {
            shift_ident(name, off);
            shift_expr(value, off);
            shift_span(span, off);
        }
    }
}

fn shift_named_ctor_field(f: &mut NamedCtorField, off: usize) {
    shift_ident(&mut f.name, off);
    shift_expr(&mut f.value, off);
    shift_span(&mut f.span, off);
}

fn shift_expr(e: &mut Expr, off: usize) {
    shift_span(&mut e.span, off);
    match &mut e.kind {
        ExprKind::IntLit(_) | ExprKind::BoolLit(_) | ExprKind::StrLit(_)
        | ExprKind::CharLit(_) | ExprKind::TypeWordArg(_) => {}
        ExprKind::Var(id) => shift_ident(id, off),
        ExprKind::Method { recv, name, args }
        | ExprKind::MethodCall { recv, name, args } => {
            shift_expr(recv, off);
            shift_ident(name, off);
            for a in args.iter_mut() {
                shift_arg(a, off);
            }
        }
        ExprKind::Call { callee, args } => {
            shift_ident(callee, off);
            for a in args.iter_mut() {
                shift_arg(a, off);
            }
        }
        ExprKind::CallValue { callee, args } => {
            shift_expr(callee, off);
            for a in args.iter_mut() {
                shift_arg(a, off);
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            shift_expr(lhs, off);
            shift_expr(rhs, off);
        }
        ExprKind::Unary { operand, .. } => shift_expr(operand, off),
        ExprKind::CtorNamed { ty, fields } => {
            shift_ident(ty, off);
            for f in fields.iter_mut() {
                shift_named_ctor_field(f, off);
            }
        }
        ExprKind::Field { base, name } => {
            shift_expr(base, off);
            shift_ident(name, off);
        }
        ExprKind::MutArg { name } => shift_ident(name, off),
        ExprKind::GenericType { name, args } => {
            shift_ident(name, off);
            for a in args.iter_mut() {
                shift_type_ann(a, off);
            }
        }
        ExprKind::Closure { params, body, .. } => {
            for p in params.iter_mut() {
                shift_param(p, off);
            }
            match &mut **body {
                ClosureBody::Expr(e) => shift_expr(e, off),
                ClosureBody::Block(b) => shift_block(b, off),
            }
        }
    }
}

fn shift_match_arm(a: &mut MatchArm, off: usize) {
    if let Some(q) = &mut a.qual {
        shift_ident(q, off);
    }
    shift_ident(&mut a.pattern, off);
    if let Some(b) = &mut a.binding {
        shift_ident(b, off);
    }
    shift_block(&mut a.body, off);
    shift_span(&mut a.span, off);
}

fn shift_stmt(s: &mut Stmt, off: usize) {
    match s {
        Stmt::ExprStmt { value, span } => {
            shift_expr(value, off);
            shift_span(span, off);
        }
        Stmt::Let { kw, name, ann, value, span, .. } => {
            shift_span(kw, off);
            shift_ident(name, off);
            if let Some(a) = ann {
                shift_type_ann(a, off);
            }
            shift_expr(value, off);
            shift_span(span, off);
        }
        Stmt::Assign { target, value, span } => {
            shift_ident(&mut target.base, off);
            for p in target.path.iter_mut() {
                shift_ident(p, off);
            }
            shift_expr(value, off);
            shift_span(span, off);
        }
        Stmt::Return { value, span } => {
            shift_expr(value, off);
            shift_span(span, off);
        }
        Stmt::If { cond, then, els, span } => {
            shift_expr(cond, off);
            shift_block(then, off);
            if let Some(b) = els {
                shift_block(b, off);
            }
            shift_span(span, off);
        }
        Stmt::While { cond, body, span } => {
            shift_expr(cond, off);
            shift_block(body, off);
            shift_span(span, off);
        }
        Stmt::Loop { body, span } => {
            shift_block(body, off);
            shift_span(span, off);
        }
        Stmt::Break { span } | Stmt::Continue { span } => shift_span(span, off),
        Stmt::IsMatch { scrutinee, arms, span } => {
            shift_ident(scrutinee, off);
            for a in arms.iter_mut() {
                shift_match_arm(a, off);
            }
            shift_span(span, off);
        }
        Stmt::ForIn { var, iter, body, span } => {
            shift_ident(var, off);
            shift_expr(iter, off);
            shift_block(body, off);
            shift_span(span, off);
        }
    }
}

fn shift_block(b: &mut Block, off: usize) {
    for s in b.stmts.iter_mut() {
        shift_stmt(s, off);
    }
    shift_span(&mut b.span, off);
}

pub(crate) fn shift_fn_item(f: &mut FnItem, off: usize) {
    shift_ident(&mut f.name, off);
    for p in f.params.iter_mut() {
        shift_param(p, off);
    }
    shift_type_ann(&mut f.ret, off);
    shift_block(&mut f.body, off);
    shift_span(&mut f.span, off);
    if let Some(o) = &mut f.owner {
        shift_ident(o, off);
    }
}
