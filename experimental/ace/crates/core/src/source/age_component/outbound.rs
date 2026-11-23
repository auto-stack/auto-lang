//! Ordered scalar events produced by a checked widget transition.

use super::{
    clone_handler_value, payload_type, text_diag, Arg, Block, ComponentNode, ComponentStmt,
    Diagnostic, Expr, ExprKind, HandlerLocals, Ident, Span, Stmt, TypeAnn, TypeRef, TypeWord,
};

pub const FIELD: &str = "__age_p_hidden_outbound";
pub const FOCUS_FIELD: &str = "__age_p_hidden_focus";

pub(crate) fn has_focus(node: &ComponentNode) -> bool {
    fn requests(stmts: &[ComponentStmt]) -> bool {
        stmts.iter().any(|stmt| match stmt {
            ComponentStmt::Focus { .. } => true,
            ComponentStmt::If { then, els, .. } => {
                requests(then) || els.as_deref().is_some_and(requests)
            }
            ComponentStmt::While { body, .. } => requests(body),
            // PLAN-117（U05 029 真语料）：try 体同样可含 focus 请求。
            ComponentStmt::Try { body, catch, .. } => {
                requests(body) || requests(catch)
            }
            _ => false,
        })
    }
    node.handlers.iter().any(|handler| requests(&handler.stmts))
        || node.members.iter().any(|row| row.watch_body.as_deref().is_some_and(requests))
}

pub(crate) fn has_messages(node: &ComponentNode) -> bool {
    fn emits(stmts: &[ComponentStmt], node: &ComponentNode) -> bool {
        stmts.iter().any(|stmt| match stmt {
            ComponentStmt::Send { message, .. } => {
                node.messages.iter().any(|decl| decl.name == *message)
                    && !node.handlers.iter().any(|handler| handler.msg == *message)
            }
            ComponentStmt::If { then, els, .. } => {
                emits(then, node) || els.as_ref().is_some_and(|branch| emits(branch, node))
            }
            ComponentStmt::While { body, .. } => emits(body, node),
            // PLAN-117（U05 029 真语料）：try/catch 体同样可承载对外发送
            //（`try { ."queryRequested"(…) } catch { }`）——漏判会把
            // has_messages 判 false，handler 的 outbound 发送被误拒。
            ComponentStmt::Try { body, catch, .. } => emits(body, node) || emits(catch, node),
            _ => false,
        })
    }
    node.handlers
        .iter()
        .any(|handler| emits(&handler.stmts, node))
}

pub(crate) fn queue_type(span: Span) -> TypeAnn {
    TypeAnn {
        kind: TypeRef::Generic {
            name: ident("List", span),
            args: vec![scalar_type(TypeWord::StrOwned, span)],
        },
        span,
    }
}

pub(crate) fn empty_queue(field: &str, span: Span) -> Stmt {
    Stmt::Let {
        kw: span,
        name: ident(field, span),
        ann: Some(queue_type(span)),
        mutable: false,
        value: Expr {
            kind: ExprKind::ArrayLit(Vec::new()),
            span,
        },
        span,
    }
}

pub(super) fn append(
    message: &str,
    types: &[String],
    values: Vec<Expr>,
    instance: usize,
    sequence: usize,
    span: Span,
    stmts: &mut Vec<Stmt>,
    locals: &mut HandlerLocals,
) -> Result<(), Vec<Diagnostic>> {
    stmts.push(push(FIELD, literal(message, span)));
    for (ty, mut value) in types.iter().zip(values) {
        let span = value.span;
        let ty = payload_type(ty).ok_or_else(|| {
            vec![text_diag(
                "source.age.handler-outbound-payload",
                format!("outbound message `{message}` has unsupported payload type `{ty}`"),
                span,
            )]
        })?;
        if ty == TypeWord::StrOwned
            && matches!(
                value.kind,
                ExprKind::Var(_) | ExprKind::Field { .. } | ExprKind::Index { .. }
            )
        {
            value = clone_handler_value(value, span);
        }
        let name = ident(&locals.bind(instance, sequence, "outbound_arg"), span);
        stmts.push(binding(name.clone(), ty, value));
        match ty {
            TypeWord::Bool => stmts.push(Stmt::If {
                cond: var(name),
                then: Block {
                    stmts: vec![push(FIELD, literal("true", span))],
                    span,
                },
                els: Some(Block {
                    stmts: vec![push(FIELD, literal("false", span))],
                    span,
                }),
                span,
            }),
            TypeWord::I32 => {
                let text = ident(&locals.bind(instance, sequence, "outbound_text"), span);
                stmts.push(binding(
                    text.clone(),
                    TypeWord::StrOwned,
                    Expr {
                        kind: ExprKind::Call {
                            callee: ident("__fstr", span),
                            args: vec![Arg::Pos(var(name))],
                        },
                        span,
                    },
                ));
                stmts.push(push(FIELD, owned(text)));
            }
            TypeWord::StrOwned => stmts.push(push(FIELD, owned(name))),
            _ => unreachable!("outbound scalar type"),
        }
    }
    Ok(())
}

pub(super) fn append_focus(mut target: Expr, local: &str, stmts: &mut Vec<Stmt>) {
    let span = target.span;
    if matches!(target.kind, ExprKind::Var(_) | ExprKind::Field { .. } | ExprKind::Index { .. }) {
        target = clone_handler_value(target, span);
    }
    let name = ident(local, span);
    stmts.push(binding(name.clone(), TypeWord::StrOwned, target));
    stmts.push(push(FOCUS_FIELD, owned(name)));
}

fn ident(name: &str, span: Span) -> Ident {
    Ident {
        name: name.to_owned(),
        span,
    }
}

fn scalar_type(ty: TypeWord, span: Span) -> TypeAnn {
    TypeAnn {
        kind: TypeRef::Builtin(ty),
        span,
    }
}

fn literal(value: &str, span: Span) -> Expr {
    Expr {
        kind: ExprKind::StrLit(value.to_owned()),
        span,
    }
}

fn var(name: Ident) -> Expr {
    Expr {
        span: name.span,
        kind: ExprKind::Var(name),
    }
}

fn binding(name: Ident, ty: TypeWord, value: Expr) -> Stmt {
    let span = name.span;
    Stmt::Let {
        kw: span,
        name,
        ann: Some(scalar_type(ty, span)),
        mutable: false,
        value,
        span,
    }
}

fn owned(name: Ident) -> Expr {
    let span = name.span;
    Expr {
        kind: ExprKind::Method {
            recv: Box::new(var(name)),
            name: ident("move", span),
            args: Vec::new(),
        },
        span,
    }
}

fn push(field: &str, value: Expr) -> Stmt {
    let span = value.span;
    Stmt::ExprStmt {
        value: Expr {
            kind: ExprKind::Method {
            recv: Box::new(var(ident(field, span))),
                name: ident("push", span),
                args: vec![Arg::Pos(value)],
            },
            span,
        },
        span,
    }
}
