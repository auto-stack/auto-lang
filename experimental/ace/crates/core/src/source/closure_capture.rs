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
            // PLAN-936（T-04）：CT 需求原子——inner 递归（scalar-ct 面无
            // 闭包，本臂仅为穷尽性与未来面安全）。
            ExprKind::HashExpr { expr: inner } => {
                visit_expr(inner, outer_locals, scopes, uses, by_name)
            }
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

/// PLAN-909 shared-write cell grouping (static semantics). Binding identity
/// is the DECLARING span (a `let`/`var` initializer phrase, a param phrase,
/// a for-in/is-match binding) — never a bare name. The scan walks one
/// function body with its own scope model: every declaration inserts its
/// binding span (innermost shadows), closure PARAMETERS shadow at closure
/// entry, and closure captures deliberately PASS THROUGH to the enclosing
/// mapping (a free name in a nested closure names the same outer binding
/// the creating closure captured). For every NON-move closure, free WRITTEN
/// uses resolve against that model; a `var` binding they land on joins the
/// shared-cell set S. `let` bindings and for-in/is-match bindings shadow
/// correctly but are never eligible (writing them stays the static
/// immutable-assignment rejection). The resolver later marks the matching
/// `LocalDecl`s (matched by the identical span join) and the lowering
/// materializes each cell once, at its declaration, before any closure
/// that shares it — a reader created first still observes shared writes.
pub(crate) fn scan_shared_capture_cells(
    params: &[ast::Param],
    body: &ast::Block,
) -> BTreeSet<usize> {
    /// Binding identity: the declaration phrase's START offset (the `let`/
    /// `var` name token, a param name, a for-in/is-match binding) — unique
    /// per token, so distinct declarations never collide. The second
    /// component marks cell eligibility: only a mutable scalar `var`
    /// initializer can join the shared-cell set.
    type Binding = (usize, bool);

    let mut shared = BTreeSet::new();
    let mut scopes: Vec<BTreeMap<String, Binding>> = vec![params
        .iter()
        .map(|param| (param.name.name.clone(), (param.span.start, false)))
        .collect()];

    fn lookup(scopes: &[BTreeMap<String, Binding>], name: &str) -> Option<Binding> {
        scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn record_shared(
        params: &[ast::Param],
        body: &ClosureBody,
        scopes: &[BTreeMap<String, Binding>],
        shared: &mut BTreeSet<usize>,
    ) {
        // Reuse the per-closure free-use analysis: `outer_locals` is every
        // name visible at the closure point under this scope model.
        let visible: BTreeSet<String> = scopes
            .iter()
            .flat_map(|scope| scope.keys().cloned())
            .collect();
        for use_ in analyze_capture_uses(params, body, &visible) {
            if use_.written {
                if let Some((start, eligible)) = lookup(scopes, &use_.name) {
                    if eligible {
                        shared.insert(start);
                    }
                }
            }
        }
    }

    fn visit_expr(
        expr: &Expr,
        scopes: &mut Vec<BTreeMap<String, Binding>>,
        shared: &mut BTreeSet<usize>,
    ) {
        match &expr.kind {
            ExprKind::HashExpr { expr: inner } => visit_expr(inner, scopes, shared),
            ExprKind::Call { args, .. } => {
                for arg in args {
                    visit_arg(arg, scopes, shared);
                }
            }
            ExprKind::CallValue { callee, args } => {
                visit_expr(callee, scopes, shared);
                for arg in args {
                    visit_arg(arg, scopes, shared);
                }
            }
            ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
                visit_expr(recv, scopes, shared);
                for arg in args {
                    visit_arg(arg, scopes, shared);
                }
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                visit_expr(lhs, scopes, shared);
                visit_expr(rhs, scopes, shared);
            }
            ExprKind::Unary { operand, .. } => visit_expr(operand, scopes, shared),
            ExprKind::Field { base, .. } => visit_expr(base, scopes, shared),
            ExprKind::CtorNamed { fields, .. } => {
                for field in fields {
                    visit_expr(&field.value, scopes, shared);
                }
            }
            ExprKind::Closure {
                move_capture,
                params,
                body,
            } => {
                // A move closure's writes are PRIVATE (snapshot semantics),
                // never shared-cell facts (§5.1 mixed-row).
                if !move_capture {
                    record_shared(params, body, scopes, shared);
                }
                // Descend with the closure's own params shadowing; captures
                // pass through to the enclosing binding model.
                scopes.push(
                    params
                        .iter()
                        .map(|param| (param.name.name.clone(), (param.span.start, false)))
                        .collect(),
                );
                match &**body {
                    ClosureBody::Expr(expr) => visit_expr(expr, scopes, shared),
                    ClosureBody::Block(block) => visit_block(block, scopes, shared),
                }
                scopes.pop();
            }
            ExprKind::Var(_)
            | ExprKind::IntLit(_)
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
        scopes: &mut Vec<BTreeMap<String, Binding>>,
        shared: &mut BTreeSet<usize>,
    ) {
        match arg {
            Arg::Pos(expr) | Arg::MovePos(expr) => visit_expr(expr, scopes, shared),
            Arg::Named { value, .. } => visit_expr(value, scopes, shared),
        }
    }

    fn visit_block(
        block: &ast::Block,
        scopes: &mut Vec<BTreeMap<String, Binding>>,
        shared: &mut BTreeSet<usize>,
    ) {
        scopes.push(BTreeMap::new());
        for stmt in &block.stmts {
            match stmt {
                Stmt::ExprStmt { value, .. } | Stmt::Return { value, .. } => {
                    visit_expr(value, scopes, shared)
                }
                Stmt::Let {
                    name,
                    value,
                    mutable,
                    ..
                } => {
                    visit_expr(value, scopes, shared);
                    scopes
                        .last_mut()
                        .unwrap()
                        .insert(name.name.clone(), (name.span.start, *mutable));
                }
                Stmt::Assign { value, .. } => visit_expr(value, scopes, shared),
                Stmt::If {
                    cond, then, els, ..
                } => {
                    visit_expr(cond, scopes, shared);
                    visit_block(then, scopes, shared);
                    if let Some(els) = els {
                        visit_block(els, scopes, shared);
                    }
                }
                Stmt::While { cond, body, .. } => {
                    visit_expr(cond, scopes, shared);
                    visit_block(body, scopes, shared);
                }
                Stmt::Loop { body, .. } => visit_block(body, scopes, shared),
                Stmt::ForIn {
                    var, iter, body, ..
                } => {
                    visit_expr(iter, scopes, shared);
                    scopes.push(BTreeMap::from([(
                        var.name.clone(),
                        (var.span.start, false),
                    )]));
                    visit_block(body, scopes, shared);
                    scopes.pop();
                }
                Stmt::IsMatch { arms, .. } => {
                    for arm in arms {
                        scopes.push(
                            arm.binding
                                .as_ref()
                                .map(|binding| {
                                    BTreeMap::from([(
                                        binding.name.clone(),
                                        (binding.span.start, false),
                                    )])
                                })
                                .unwrap_or_default(),
                        );
                        visit_block(&arm.body, scopes, shared);
                        scopes.pop();
                    }
                }
                Stmt::Break { .. } | Stmt::Continue { .. } => {}
            }
        }
        scopes.pop();
    }

    visit_block(body, &mut scopes, &mut shared);
    shared
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
