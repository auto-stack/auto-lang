//! 片段绝对坐标平移（T-07）：片段在授权区间自身的快照上解析（0 基），
//! 投影后按 `range.start` 平移全部 span——与旧 parse_function_fragment
//! 的 token 偏移合同一致（绝对 UTF-8 byte span，不改源码序/结构）。

use crate::atom_text::{Diagnostic, Span};
use crate::source::ast::*;

/// PLAN-929（T-05）：核心改为带符号平移（饱和到 0；合成宿主片段桥需要
/// 负向回退）。usize 公开面委托正 delta。
pub(crate) fn shift_span(s: &mut Span, delta: i64) {
    s.start = ((s.start as i64 + delta).max(0)) as usize;
    s.end = ((s.end as i64 + delta).max(0)) as usize;
}

pub(crate) fn shift_diag(d: &mut Diagnostic, delta: i64) {
    shift_span(&mut d.span, delta);
}

fn shift_ident(i: &mut Ident, delta: i64) {
    shift_span(&mut i.span, delta);
}

fn shift_type_ann(t: &mut TypeAnn, delta: i64) {
    shift_span(&mut t.span, delta);
    match &mut t.kind {
        TypeRef::Record(id) => shift_ident(id, delta),
        // PLAN-935：类型变量引用同 ident 平移。
        TypeRef::TyVar(id) => shift_ident(id, delta),
        TypeRef::Optional(inner) => shift_type_ann(inner, delta),
        // PLAN-962：tuple 类型界平移。
        TypeRef::Tuple(args) => {
            for a in args {
                shift_type_ann(a, delta);
            }
        }
        TypeRef::Generic { name, args } => {
            shift_ident(name, delta);
            for a in args.iter_mut() {
                shift_type_ann(a, delta);
            }
        }
        TypeRef::Function { params, result } => {
            for p in params.iter_mut() {
                shift_type_ann(p, delta);
            }
            shift_type_ann(result, delta);
        }
        TypeRef::Builtin(_) => {}
    }
}

fn shift_param(p: &mut Param, delta: i64) {
    shift_ident(&mut p.name, delta);
    shift_type_ann(&mut p.ty, delta);
    shift_span(&mut p.span, delta);
}

fn shift_arg(a: &mut Arg, delta: i64) {
    match a {
        Arg::Pos(e) | Arg::MovePos(e) => shift_expr(e, delta),
        Arg::Named { name, value, span } => {
            shift_ident(name, delta);
            shift_expr(value, delta);
            shift_span(span, delta);
        }
    }
}

fn shift_named_ctor_field(f: &mut NamedCtorField, delta: i64) {
    shift_ident(&mut f.name, delta);
    shift_expr(&mut f.value, delta);
    shift_span(&mut f.span, delta);
}

fn shift_expr(e: &mut Expr, delta: i64) {
    shift_span(&mut e.span, delta);
    match &mut e.kind {
        // PLAN-936（T-04）：CT 需求原子——inner span 同步平移。
        ExprKind::HashExpr { expr: inner } => shift_expr(inner, delta),
        // PLAN-962：范围界 span 平移。
        ExprKind::Range { start, end, .. } => {
            shift_expr(start, delta);
            shift_expr(end, delta);
        }
        // PLAN-962：数组字面量元素/索引两端 span 平移。
        ExprKind::ArrayLit(elems) => {
            for e in elems {
                shift_expr(e, delta);
            }
        }
        // PLAN-962：tuple 字面量元素 span 平移。
        ExprKind::TupleLit(elems) => {
            for e in elems {
                shift_expr(e, delta);
            }
        }
        ExprKind::Index { base, index } => {
            shift_expr(base, delta);
            shift_expr(index, delta);
        }
        ExprKind::IntLit(_)
        | ExprKind::BoolLit(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::TypeWordArg(_)
        // PLAN-961: a typed literal is a leaf.
        | ExprKind::TypedLit { .. } => {}
        ExprKind::Var(id) => shift_ident(id, delta),
        ExprKind::Method { recv, name, args } | ExprKind::MethodCall { recv, name, args } => {
            shift_expr(recv, delta);
            shift_ident(name, delta);
            for a in args.iter_mut() {
                shift_arg(a, delta);
            }
        }
        ExprKind::Call { callee, args } => {
            shift_ident(callee, delta);
            for a in args.iter_mut() {
                shift_arg(a, delta);
            }
        }
        ExprKind::CallValue { callee, args } => {
            shift_expr(callee, delta);
            for a in args.iter_mut() {
                shift_arg(a, delta);
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            shift_expr(lhs, delta);
            shift_expr(rhs, delta);
        }
        ExprKind::Unary { operand, .. } => shift_expr(operand, delta),
        ExprKind::CtorNamed { ty, fields } => {
            shift_ident(ty, delta);
            for f in fields.iter_mut() {
                shift_named_ctor_field(f, delta);
            }
        }
        ExprKind::Field { base, name } => {
            shift_expr(base, delta);
            shift_ident(name, delta);
        }
        ExprKind::MutArg { name } => shift_ident(name, delta),
        ExprKind::GenericType { name, args } => {
            shift_ident(name, delta);
            for a in args.iter_mut() {
                shift_type_ann(a, delta);
            }
        }
        ExprKind::Closure { params, body, .. } => {
            for p in params.iter_mut() {
                shift_param(p, delta);
            }
            match &mut **body {
                ClosureBody::Expr(e) => shift_expr(e, delta),
                ClosureBody::Block(b) => shift_block(b, delta),
            }
        }
    }
}

fn shift_match_arm(a: &mut MatchArm, delta: i64) {
    if let Some(q) = &mut a.qual {
        shift_ident(q, delta);
    }
    shift_ident(&mut a.pattern, delta);
    if let Some(b) = &mut a.binding {
        shift_ident(b, delta);
    }
    shift_block(&mut a.body, delta);
    shift_span(&mut a.span, delta);
}

fn shift_stmt(s: &mut Stmt, delta: i64) {
    match s {
        // PLAN-962：索引赋值平移。
        Stmt::IndexAssign(ia) => {
            shift_expr(&mut ia.index, delta);
            shift_expr(&mut ia.value, delta);
            shift_span(&mut ia.span, delta);
        }
        // PLAN-962：tuple 解构平移。
        Stmt::LetDestruct { names, value, span } => {
            for n in names {
                shift_ident(n, delta);
            }
            shift_expr(value, delta);
            shift_span(span, delta);
        }
        Stmt::ExprStmt { value, span } => {
            shift_expr(value, delta);
            shift_span(span, delta);
        }
        Stmt::Let {
            kw,
            name,
            ann,
            value,
            span,
            ..
        } => {
            shift_span(kw, delta);
            shift_ident(name, delta);
            if let Some(a) = ann {
                shift_type_ann(a, delta);
            }
            shift_expr(value, delta);
            shift_span(span, delta);
        }
        Stmt::Assign {
            target,
            value,
            span,
        } => {
            shift_ident(&mut target.base, delta);
            for p in target.path.iter_mut() {
                shift_ident(p, delta);
            }
            shift_expr(value, delta);
            shift_span(span, delta);
        }
        Stmt::Return { value, span } => {
            shift_expr(value, delta);
            shift_span(span, delta);
        }
        Stmt::If {
            cond,
            then,
            els,
            span,
        } => {
            shift_expr(cond, delta);
            shift_block(then, delta);
            if let Some(b) = els {
                shift_block(b, delta);
            }
            shift_span(span, delta);
        }
        Stmt::While { cond, body, span } => {
            shift_expr(cond, delta);
            shift_block(body, delta);
            shift_span(span, delta);
        }
        Stmt::Loop { body, span } => {
            shift_block(body, delta);
            shift_span(span, delta);
        }
        Stmt::Break { span } | Stmt::Continue { span } => shift_span(span, delta),
        Stmt::IsMatch {
            scrutinee,
            arms,
            span,
        } => {
            shift_ident(scrutinee, delta);
            for a in arms.iter_mut() {
                shift_match_arm(a, delta);
            }
            shift_span(span, delta);
        }
        Stmt::ForIn {
            var,
            iter,
            body,
            span,
        } => {
            shift_ident(var, delta);
            shift_expr(iter, delta);
            shift_block(body, delta);
            shift_span(span, delta);
        }
    }
}

fn shift_block(b: &mut Block, delta: i64) {
    for s in b.stmts.iter_mut() {
        shift_stmt(s, delta);
    }
    shift_span(&mut b.span, delta);
}

pub(crate) fn shift_fn_item(f: &mut FnItem, off: usize) {
    shift_fn_item_signed(f, off as i64)
}

/// PLAN-929（T-05）：带符号 FnItem 平移（同一 visitor 覆盖面）。
pub(crate) fn shift_fn_item_signed(f: &mut FnItem, delta: i64) {
    shift_ident(&mut f.name, delta);
    // PLAN-935：类型参数声明 ident 同面平移。
    for tp in f.type_params.iter_mut() {
        shift_ident(tp, delta);
    }
    for p in f.params.iter_mut() {
        shift_param(p, delta);
    }
    shift_type_ann(&mut f.ret, delta);
    shift_block(&mut f.body, delta);
    shift_span(&mut f.span, delta);
    if let Some(o) = &mut f.owner {
        shift_ident(o, delta);
    }
}
