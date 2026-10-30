//! PLAN-865 free-local capture discovery. This pass is deliberately separate
//! from lowering: it records first-use order and writes without selecting a
//! move/clone/borrow mode.

use std::collections::{BTreeMap, BTreeSet};

use crate::atom_text::Span;

use super::ast::{self, Arg, ClosureBody, Expr, ExprKind, Stmt};

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct CaptureUse {
    pub name: String,
    pub span: Span,
    pub written: bool,
}

/// Find reads and writes of the enclosing function's locals that remain free
/// after accounting for lambda parameters and body-local declarations.
/// Results are ordered by first source occurrence, which is also the frozen
/// CREATE_CLOSURE/environment field order.
pub(crate) fn analyze_capture_uses(
    params: &[ast::Param],
    body: &ClosureBody,
    outer_locals: &BTreeSet<String>,
) -> Vec<CaptureUse> {
    let mut scopes = vec![params
        .iter()
        .map(|param| param.name.name.clone())
        .collect::<BTreeSet<_>>()];
    let mut uses = Vec::new();
    let mut by_name = BTreeMap::<String, usize>::new();

    fn record(
        id: &ast::Ident,
        written: bool,
        outer_locals: &BTreeSet<String>,
        scopes: &[BTreeSet<String>],
        uses: &mut Vec<CaptureUse>,
        by_name: &mut BTreeMap<String, usize>,
    ) {
        if !outer_locals.contains(&id.name)
            || scopes.iter().rev().any(|scope| scope.contains(&id.name))
        {
            return;
        }
        if let Some(index) = by_name.get(&id.name).copied() {
            uses[index].written |= written;
        } else {
            by_name.insert(id.name.clone(), uses.len());
            uses.push(CaptureUse {
                name: id.name.clone(),
                span: id.span,
                written,
            });
        }
    }

    fn visit_expr(
        expr: &Expr,
        outer_locals: &BTreeSet<String>,
        scopes: &mut Vec<BTreeSet<String>>,
        uses: &mut Vec<CaptureUse>,
        by_name: &mut BTreeMap<String, usize>,
    ) {
        match &expr.kind {
            ExprKind::Var(id) => record(id, false, outer_locals, scopes, uses, by_name),
            ExprKind::Call { callee, args } => {
                record(callee, false, outer_locals, scopes, uses, by_name);
                for arg in args {
                    visit_arg(arg, outer_locals, scopes, uses, by_name);
                }
            }
            ExprKind::CallValue { callee, args } => {
                visit_expr(callee, outer_locals, scopes, uses, by_name);
                for arg in args {
                    visit_arg(arg, outer_locals, scopes, uses, by_name);
                }
            }
            ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
                visit_expr(recv, outer_locals, scopes, uses, by_name);
                for arg in args {
                    visit_arg(arg, outer_locals, scopes, uses, by_name);
                }
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                visit_expr(lhs, outer_locals, scopes, uses, by_name);
                visit_expr(rhs, outer_locals, scopes, uses, by_name);
            }
            ExprKind::Unary { operand, .. } => {
                visit_expr(operand, outer_locals, scopes, uses, by_name)
            }
            ExprKind::Field { base, .. } => visit_expr(base, outer_locals, scopes, uses, by_name),
            ExprKind::CtorNamed { fields, .. } => {
                for field in fields {
                    visit_expr(&field.value, outer_locals, scopes, uses, by_name);
                }
            }
            ExprKind::Closure { params, body, .. } => {
                scopes.push(params.iter().map(|param| param.name.name.clone()).collect());
                visit_closure_body(body, outer_locals, scopes, uses, by_name);
                scopes.pop();
            }
            ExprKind::IntLit(_)
            | ExprKind::BoolLit(_)
            | ExprKind::StrLit(_)
            | ExprKind::CharLit(_)
            | ExprKind::TypeWordArg(_)
            | ExprKind::MutArg { .. }
            | ExprKind::GenericType { .. } => {}
        }
    }

    fn visit_arg(
        arg: &Arg,
        outer_locals: &BTreeSet<String>,
        scopes: &mut Vec<BTreeSet<String>>,
        uses: &mut Vec<CaptureUse>,
        by_name: &mut BTreeMap<String, usize>,
    ) {
        match arg {
            Arg::Pos(expr) | Arg::MovePos(expr) => {
                visit_expr(expr, outer_locals, scopes, uses, by_name)
            }
            Arg::Named { value, .. } => visit_expr(value, outer_locals, scopes, uses, by_name),
        }
    }

    fn visit_closure_body(
        body: &ClosureBody,
        outer_locals: &BTreeSet<String>,
        scopes: &mut Vec<BTreeSet<String>>,
        uses: &mut Vec<CaptureUse>,
        by_name: &mut BTreeMap<String, usize>,
    ) {
        match body {
            ClosureBody::Expr(expr) => visit_expr(expr, outer_locals, scopes, uses, by_name),
            ClosureBody::Block(block) => visit_block(block, outer_locals, scopes, uses, by_name),
        }
    }

    fn visit_block(
        block: &ast::Block,
        outer_locals: &BTreeSet<String>,
        scopes: &mut Vec<BTreeSet<String>>,
        uses: &mut Vec<CaptureUse>,
        by_name: &mut BTreeMap<String, usize>,
    ) {
        scopes.push(BTreeSet::new());
        for stmt in &block.stmts {
            match stmt {
                Stmt::ExprStmt { value, .. } | Stmt::Return { value, .. } => {
                    visit_expr(value, outer_locals, scopes, uses, by_name)
                }
                Stmt::Let { name, value, .. } => {
                    visit_expr(value, outer_locals, scopes, uses, by_name);
                    scopes.last_mut().unwrap().insert(name.name.clone());
                }
                Stmt::Assign { target, value, .. } => {
                    record(&target.base, true, outer_locals, scopes, uses, by_name);
                    visit_expr(value, outer_locals, scopes, uses, by_name);
                }
                Stmt::If {
                    cond, then, els, ..
                } => {
                    visit_expr(cond, outer_locals, scopes, uses, by_name);
                    visit_block(then, outer_locals, scopes, uses, by_name);
                    if let Some(els) = els {
                        visit_block(els, outer_locals, scopes, uses, by_name);
                    }
                }
                Stmt::While { cond, body, .. } => {
                    visit_expr(cond, outer_locals, scopes, uses, by_name);
                    visit_block(body, outer_locals, scopes, uses, by_name);
                }
                Stmt::Loop { body, .. } => visit_block(body, outer_locals, scopes, uses, by_name),
                Stmt::ForIn {
                    var, iter, body, ..
                } => {
                    visit_expr(iter, outer_locals, scopes, uses, by_name);
                    scopes.push(BTreeSet::from([var.name.clone()]));
                    visit_block(body, outer_locals, scopes, uses, by_name);
                    scopes.pop();
                }
                Stmt::IsMatch {
                    scrutinee, arms, ..
                } => {
                    record(scrutinee, false, outer_locals, scopes, uses, by_name);
                    for arm in arms {
                        scopes.push(
                            arm.binding
                                .as_ref()
                                .map(|binding| BTreeSet::from([binding.name.clone()]))
                                .unwrap_or_default(),
                        );
                        visit_block(&arm.body, outer_locals, scopes, uses, by_name);
                        scopes.pop();
                    }
                }
                Stmt::Break { .. } | Stmt::Continue { .. } => {}
            }
        }
        scopes.pop();
    }

    visit_closure_body(body, outer_locals, &mut scopes, &mut uses, &mut by_name);
    uses
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{parse_program_with, SourceProfile};

    fn closure_from(source: &str) -> (Vec<ast::Param>, Box<ClosureBody>) {
        let program = parse_program_with(source, SourceProfile::Closures).unwrap();
        let ast::Item::Fn(main) = &program.items[0] else {
            panic!("expected a function");
        };
        let Stmt::Let { value, .. } = &main.body.stmts[2] else {
            panic!("expected a closure binding");
        };
        let ExprKind::Closure { params, body, .. } = &value.kind else {
            panic!("expected a closure expression");
        };
        (params.clone(), body.clone())
    }

    #[test]
    fn captures_follow_first_source_use_and_exclude_parameters_and_locals() {
        let source = "fn main() int { let a = 1; let b = 2; let f = (n int) => { let inner = n + b; return a + inner }; return 0 }";
        let (params, body) = closure_from(source);
        let captures = analyze_capture_uses(
            &params,
            &body,
            &BTreeSet::from(["a".to_string(), "b".to_string()]),
        );
        assert_eq!(
            captures
                .iter()
                .map(|capture| capture.name.as_str())
                .collect::<Vec<_>>(),
            ["b", "a"]
        );
        assert!(captures.iter().all(|capture| !capture.written));
    }

    #[test]
    fn writes_to_outer_locals_are_reported_as_writes() {
        let source = "fn main() int { let a = 1; let b = 2; let f = (n int) => { a = n; return b }; return 0 }";
        let (params, body) = closure_from(source);
        let captures = analyze_capture_uses(
            &params,
            &body,
            &BTreeSet::from(["a".to_string(), "b".to_string()]),
        );
        assert_eq!(captures[0].name, "a");
        assert!(captures[0].written);
        assert_eq!(captures[1].name, "b");
        assert!(!captures[1].written);
    }
}
