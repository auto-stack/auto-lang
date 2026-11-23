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
        // PLAN-1080：Inferred 无坐标载荷。
        TypeRef::Inferred => {}
        TypeRef::Record(id) => shift_ident(id, delta),
        // PLAN-935：类型变量引用同 ident 平移。
        TypeRef::TyVar(id) => shift_ident(id, delta),
        TypeRef::Optional(inner) | TypeRef::ResultBang(inner) | TypeRef::Stream(inner) => {
            shift_type_ann(inner, delta)
        }
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

pub(super) fn shift_expr(e: &mut Expr, delta: i64) {
    shift_span(&mut e.span, delta);
    match &mut e.kind {
        // PLAN-936（T-04）：CT 需求原子——inner span 同步平移。
        ExprKind::HashExpr { expr: inner } => shift_expr(inner, delta),
        // PLAN-968: await/go spans shift with their operand.
        ExprKind::Await(inner) | ExprKind::Go(inner) => shift_expr(inner, delta),
        // PLAN-962：范围界 span 平移。
        ExprKind::Range { start, end, .. } => {
            shift_expr(start, delta);
            shift_expr(end, delta);
        }
        ExprKind::ArrayConditionalElem { cond, value } => {
            shift_expr(cond, delta);
            shift_expr(value, delta);
        }
        // PLAN-962：数组字面量元素/索引两端 span 平移。
        ExprKind::ArrayLit(elems) => {
            for e in elems {
                shift_expr(e, delta);
            }
        }
        // PLAN-962 (T-08)：grid 单元 span 平移。
        ExprKind::GridLit { cols, rows } => {
            for c in cols {
                shift_ident(c, delta);
            }
            for row in rows {
                for e in row {
                    shift_expr(e, delta);
                }
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
        ExprKind::PySlice { base, bounds } => {
            shift_expr(base, delta);
            for bound in bounds.iter_mut().flatten() {
                shift_expr(bound, delta);
            }
        }
        ExprKind::IntLit(_)
        | ExprKind::BoolLit(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::TypeWordArg(_)
        // PLAN-961: a typed literal is a leaf.
        | ExprKind::TypedLit { .. } => {}
        ExprKind::Var(id) => shift_ident(id, delta),
        ExprKind::Method { recv, name, args } => {
            shift_expr(recv, delta);
            shift_ident(name, delta);
            for a in args.iter_mut() {
                shift_arg(a, delta);
            }
        }
        ExprKind::MethodCall { recv, name, args, instance } => {
            if let Some(instance) = instance {
                for ty in &mut instance.types {
                    shift_type_ann(ty, delta);
                }
                for c in &mut instance.consts {
                    shift_span(&mut c.span, delta);
                }
            }
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
        // PLAN-963（T-04）：值实参调用——名字/实参/值实参 span 同步平移。
        ExprKind::ConstCall {
            callee,
            types,
            consts,
            args,
        } => {
            shift_ident(callee, delta);
            for ty in types {
                shift_type_ann(ty, delta);
            }
            for c in consts.iter_mut() {
                shift_span(&mut c.span, delta);
            }
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
        ExprKind::CtorNamed { ty, fields, .. } => {
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
        // PLAN-962 (T-08)：`.view` 实参。
        ExprKind::ViewArg { name } => shift_ident(name, delta),
        ExprKind::GenericType { name, args } => {
            shift_ident(name, delta);
            for a in args.iter_mut() {
                shift_type_ann(a, delta);
            }
        }
        ExprKind::Closure { params, result, body, .. } => {
            if let Some(result) = result {
                shift_type_ann(result, delta);
            }
            for p in params.iter_mut() {
                shift_param(p, delta);
            }
            match &mut **body {
                ClosureBody::Expr(e) => shift_expr(e, delta),
                ClosureBody::Block(b) => shift_block(b, delta),
            }
        }
        // PLAN-964: object literal fields shift; nil is a leaf; text
        // display shifts its operand.
        ExprKind::ObjectLit { fields } => {
            for f in fields.iter_mut() {
                shift_span(&mut f.span, delta);
                shift_expr(&mut f.value, delta);
            }
        }
        ExprKind::NilLit => {}
        ExprKind::TextDisplay(inner) => shift_expr(inner, delta),
        ExprKind::Propagate { inner } => shift_expr(inner, delta),
        ExprKind::UnwrapOr { inner, fallback } => {
            shift_expr(inner, delta);
            shift_expr(fallback, delta);
        }
        ExprKind::IsExpr { scrutinee, arms } => {
            shift_expr(scrutinee, delta);
            for a in arms.iter_mut() {
                shift_agg_match_arm(a, delta);
            }
        }
    }
}

fn shift_agg_match_arm(a: &mut AggMatchArm, delta: i64) {
    match &mut a.pat {
        MatchPattern::Variant { qual, name, bindings } => {
            if let Some(q) = qual {
                shift_ident(q, delta);
            }
            shift_ident(name, delta);
            for b in bindings {
                shift_ident(b, delta);
            }
        }
        MatchPattern::IntLit(_) | MatchPattern::BoolLit(_) | MatchPattern::Else => {}
    }
    match &mut a.body {
        AggArmBody::Block(b) => shift_block(b, delta),
        AggArmBody::Expr(e) => shift_expr(e, delta),
    }
    shift_span(&mut a.span, delta);
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
        // PLAN-962 (T-08)：hold 平移。
        Stmt::Hold {
            name,
            bind,
            body,
            span,
        } => {
            shift_ident(name, delta);
            shift_ident(bind, delta);
            for st in body.stmts.iter_mut() {
                shift_stmt(st, delta);
            }
            shift_span(span, delta);
        }
        // PLAN-962：索引赋值平移。
        Stmt::IndexAssign(ia) => {
            shift_expr(&mut ia.index, delta);
            shift_expr(&mut ia.value, delta);
            shift_span(&mut ia.span, delta);
        }
        // PLAN-1082：place 链赋值平移。
        Stmt::PlaceAssign(pa) => {
            shift_ident(&mut pa.base, delta);
            for st in &mut pa.steps {
                match st {
                    PlaceStep::Field(f) => shift_ident(f, delta),
                    PlaceStep::Index(ix) => shift_expr(ix, delta),
                }
            }
            shift_expr(&mut pa.value, delta);
            shift_span(&mut pa.span, delta);
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
        Stmt::AggIsMatch {
            scrutinee,
            arms,
            span,
        } => {
            shift_expr(scrutinee, delta);
            for a in arms.iter_mut() {
                shift_agg_match_arm(a, delta);
            }
            shift_span(span, delta);
        }
        Stmt::Try {
            body,
            catch,
            finally,
            span,
        } => {
            shift_block(body, delta);
            if let Some(c) = catch {
                if let Some(b) = &mut c.bind {
                    shift_ident(b, delta);
                }
                shift_block(&mut c.body, delta);
                shift_span(&mut c.span, delta);
            }
            if let Some(f) = finally {
                shift_block(f, delta);
            }
            shift_span(span, delta);
        }
        // PLAN-1094：`with <expr> [as <name>] { body }` 平移。
        Stmt::With {
            expr,
            bind,
            body,
            span,
        } => {
            shift_expr(expr, delta);
            if let Some(b) = bind {
                shift_ident(b, delta);
            }
            shift_block(body, delta);
            shift_span(span, delta);
        }
        Stmt::Yield { value, span } => {
            shift_expr(value, delta);
            shift_span(span, delta);
        }
        // PLAN-1081（T-03）：嵌套 fn 项平移（名/参/体内语句/span 递归）。
        Stmt::NestedFn { item } => {
            shift_ident(&mut item.name, delta);
            for prm in item.params.iter_mut() {
                shift_ident(&mut prm.name, delta);
            }
            shift_block(&mut item.body, delta);
            shift_span(&mut item.span, delta);
        }
        // PLAN-966（T-02）：语句级 CT 平移（谓词表达式与体内语句/span）。
        Stmt::HashIf {
            cond,
            then,
            els,
            span,
        } => {
            shift_ct_expr(cond, delta);
            shift_block(then, delta);
            if let Some(b) = els {
                shift_block(b, delta);
            }
            shift_span(span, delta);
        }
        Stmt::HashFor {
            var,
            start,
            end,
            body,
            span,
        } => {
            shift_ident(var, delta);
            shift_ct_expr(start, delta);
            shift_ct_expr(end, delta);
            shift_block(body, delta);
            shift_span(span, delta);
        }
        Stmt::HashIs {
            scrutinee,
            arms,
            span,
        } => {
            shift_ct_expr(scrutinee, delta);
            for a in arms.iter_mut() {
                if let Some(p) = &mut a.pattern {
                    shift_ct_expr(p, delta);
                }
                shift_block(&mut a.body, delta);
                shift_span(&mut a.span, delta);
            }
            shift_span(span, delta);
        }
    }
}

/// PLAN-966（T-02）：CT 标量表达式的 span 平移（引用/运算子的源位置）。
fn shift_ct_expr(e: &mut CtExpr, delta: i64) {
    match e {
        CtExpr::Int(_) | CtExpr::Bool(_) => {}
        CtExpr::Ref { span, .. } => shift_span(span, delta),
        CtExpr::Bin { lhs, rhs, span, .. } => {
            shift_ct_expr(lhs, delta);
            shift_ct_expr(rhs, delta);
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
    if let Some(span) = &mut f.test_attribute {
        shift_span(span, delta);
    }
    // PLAN-935：类型参数声明 ident 同面平移（PLAN-963：含 `<T has S>` 约束位）。
    for tp in f.type_params.iter_mut() {
        shift_ident(&mut tp.name, delta);
        if let Some(b) = &mut tp.bound {
            shift_ident(b, delta);
        }
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

/// PLAN-1121: shift a whole `type` item (the ordinary-face HashMap/HashSet
/// migration graft appends record declarations after the user source).
pub(crate) fn shift_type_item(t: &mut TypeItem, off: usize) {
    let delta = off as i64;
    shift_ident(&mut t.name, delta);
    for tp in t.type_params.iter_mut() {
        shift_ident(&mut tp.name, delta);
        if let Some(b) = &mut tp.bound {
            shift_ident(b, delta);
        }
    }
    for c in t.conforms.iter_mut() {
        shift_ident(&mut c.name, delta);
        for a in c.args.iter_mut() {
            shift_type_ann(a, delta);
        }
        for (i, a) in c.assoc.iter_mut() {
            shift_ident(i, delta);
            shift_type_ann(a, delta);
        }
    }
    for f in t.fields.iter_mut() {
        shift_ident(&mut f.name, delta);
        shift_type_ann(&mut f.ty, delta);
        shift_span(&mut f.span, delta);
    }
    for m in t.methods.iter_mut() {
        shift_fn_item_signed(m, delta);
    }
    shift_span(&mut t.span, delta);
}
