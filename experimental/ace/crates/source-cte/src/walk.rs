//! Shared structural walk for demand discovery and frozen-value backfill.

use ace_core::source::ast::{
    AggArmBody, Arg, Block, ClosureBody, Expr, ExprKind, Item, PlaceStep, Program, Stmt,
};

// The borrowed and mutably borrowed walks must visit the same children.
// A visitor returns false to prune a subtree (for example, a nested CT demand).
macro_rules! tree_walk {
    ($module:ident, $iter:ident, [$($visit_lifetime:tt)*], $($borrow:tt)*) => {
        pub(crate) mod $module {
            use super::*;

            pub(crate) enum Node<'a> {
                Value(&'a $($borrow)* Expr),
                Block(&'a $($borrow)* Block),
                Statement(&'a $($borrow)* Stmt),
            }

            pub(crate) fn visit<'a>(
                root: Node<'a>,
                mut on_expression: impl FnMut(&$($visit_lifetime)* $($borrow)* Expr) -> bool,
            ) {
                let mut pending = vec![root];
                while let Some(node) = pending.pop() {
                    let start = pending.len();
                    match node {
                        Node::Value(value) => walk_value(value, &mut pending, &mut on_expression),
                        Node::Block(block) => {
                            pending.extend(block.stmts.$iter().map(Node::Statement));
                        }
                        Node::Statement(statement) => walk_statement(statement, &mut pending),
                    }
                    pending[start..].reverse();
                }
            }

            fn walk_value<'a>(
                value: &'a $($borrow)* Expr,
                pending: &mut Vec<Node<'a>>,
                on_expression: &mut impl FnMut(&$($visit_lifetime)* $($borrow)* Expr) -> bool,
            ) {
                // Mutating traversal needs a short reborrow before children
                // can be borrowed. The visitor doesn't keep it.
                if !on_expression(value) {
                    return;
                }
                match &$($borrow)* value.kind {
                    ExprKind::TupleLit(values) | ExprKind::ArrayLit(values) => {
                        pending.extend(values.$iter().map(Node::Value));
                    }
                    ExprKind::GridLit { rows, .. } => {
                        pending.extend(rows.$iter().flat_map(|row| row.$iter()).map(Node::Value));
                    }
                    ExprKind::ObjectLit { fields } => {
                        pending.extend(fields.$iter().map(|field| Node::Value(&$($borrow)* field.value)));
                    }
                    ExprKind::CtorNamed { fields, .. } => {
                        pending.extend(fields.$iter().map(|field| Node::Value(&$($borrow)* field.value)));
                    }
                    ExprKind::Binary { lhs, rhs, .. }
                    | ExprKind::Index { base: lhs, index: rhs }
                    | ExprKind::Range { start: lhs, end: rhs, .. }
                    | ExprKind::ArrayConditionalElem { cond: lhs, value: rhs }
                    | ExprKind::UnwrapOr { inner: lhs, fallback: rhs } => {
                        pending.extend([Node::Value(lhs), Node::Value(rhs)]);
                    }
                    ExprKind::PySlice { base, bounds } => {
                        pending.push(Node::Value(base));
                        pending.extend(bounds.$iter().flatten().map(Node::Value));
                    }
                    ExprKind::Unary { operand, .. }
                    | ExprKind::HashExpr { expr: operand }
                    | ExprKind::Field { base: operand, .. }
                    | ExprKind::Propagate { inner: operand }
                    | ExprKind::TextDisplay(operand)
                    | ExprKind::Await(operand)
                    | ExprKind::Go(operand) => pending.push(Node::Value(operand)),
                    ExprKind::Call { args, .. } | ExprKind::ConstCall { args, .. } => {
                        pending.extend(args.$iter().map(argument));
                    }
                    ExprKind::CallValue { callee: recv, args }
                    | ExprKind::Method { recv, args, .. }
                    | ExprKind::MethodCall { recv, args, .. } => {
                        pending.push(Node::Value(recv));
                        pending.extend(args.$iter().map(argument));
                    }
                    ExprKind::Closure { body, .. } => pending.push(match &$($borrow)* **body {
                        ClosureBody::Expr(value) => Node::Value(value),
                        ClosureBody::Block(block) => Node::Block(block),
                    }),
                    ExprKind::IsExpr { scrutinee, arms } => {
                        pending.push(Node::Value(scrutinee));
                        pending.extend(arms.$iter().map(|arm| arm_body(&$($borrow)* arm.body)));
                    }
                    // PLAN-117（U05 029 真语料）：if 表达式——cond/then/链/末
                    // else 逐支走读（CT 需求发现与冻结回填不跳过条件侧）。
                    ExprKind::If {
                        cond,
                        then,
                        else_chain,
                        els,
                        ..
                    } => {
                        pending.push(Node::Value(cond));
                        pending.push(Node::Value(then));
                        for (c, t) in else_chain.$iter() {
                            pending.push(Node::Value(c));
                            pending.push(Node::Value(t));
                        }
                        if let Some(e) = els {
                            pending.push(Node::Value(e));
                        }
                    }
                    ExprKind::IntLit(_) | ExprKind::BoolLit(_) | ExprKind::TypedLit { .. }
                    | ExprKind::NilLit | ExprKind::Var(_) | ExprKind::StrLit(_) | ExprKind::CharLit(_)
                    | ExprKind::TypeWordArg(_) | ExprKind::GenericType { .. }
                    | ExprKind::ViewArg { .. } | ExprKind::MutArg { .. } => {}
                }
            }

            fn argument(arg: &$($borrow)* Arg) -> Node<'_> {
                Node::Value(match arg {
                    Arg::Pos(value) | Arg::MovePos(value) | Arg::Named { value, .. } => value,
                })
            }

            fn arm_body(body: &$($borrow)* AggArmBody) -> Node<'_> {
                match body {
                    AggArmBody::Block(block) => Node::Block(block),
                    AggArmBody::Expr(value) => Node::Value(value),
                }
            }

            fn walk_statement<'a>(statement: &'a $($borrow)* Stmt, pending: &mut Vec<Node<'a>>) {
                match statement {
                    Stmt::Return { value, .. }
                    | Stmt::ExprStmt { value, .. }
                    | Stmt::Let { value, .. }
                    | Stmt::LetDestruct { value, .. }
                    | Stmt::Yield { value, .. }
                    | Stmt::Assign { value, .. } => pending.push(Node::Value(value)),
                    Stmt::IndexAssign(assign) => {
                        pending.extend([Node::Value(&$($borrow)* assign.index), Node::Value(&$($borrow)* assign.value)]);
                    }
                    Stmt::PlaceAssign(assign) => {
                        for step in &$($borrow)* assign.steps {
                            if let PlaceStep::Index(index) = step {
                                pending.push(Node::Value(index));
                            }
                        }
                        pending.push(Node::Value(&$($borrow)* assign.value));
                    }
                    Stmt::Hold { body, .. }
                    | Stmt::HashFor { body, .. }
                    | Stmt::Loop { body, .. } => pending.push(Node::Block(body)),
                    Stmt::NestedFn { item } => pending.push(Node::Block(&$($borrow)* item.body)),
                    Stmt::If { cond, then, els, .. } => {
                        pending.push(Node::Value(cond));
                        pending.push(Node::Block(then));
                        pending.extend(els.$iter().map(Node::Block));
                    }
                    Stmt::HashIf { then, els, .. } => {
                        pending.push(Node::Block(then));
                        pending.extend(els.$iter().map(Node::Block));
                    }
                    Stmt::While { cond, body, .. } => {
                        pending.extend([Node::Value(cond), Node::Block(body)]);
                    }
                    Stmt::ForIn { iter, body, .. } => {
                        pending.extend([Node::Value(iter), Node::Block(body)]);
                    }
                    Stmt::HashIs { arms, .. } => {
                        pending.extend(arms.$iter().map(|arm| Node::Block(&$($borrow)* arm.body)));
                    }
                    Stmt::IsMatch { arms, .. } => {
                        pending.extend(arms.$iter().map(|arm| Node::Block(&$($borrow)* arm.body)));
                    }
                    Stmt::AggIsMatch { scrutinee, arms, .. } => {
                        pending.push(Node::Value(scrutinee));
                        pending.extend(arms.$iter().map(|arm| arm_body(&$($borrow)* arm.body)));
                    }
                    Stmt::Try { body, catch, finally, .. } => {
                        pending.push(Node::Block(body));
                        pending.extend(catch.$iter().map(|clause| Node::Block(&$($borrow)* clause.body)));
                        pending.extend(finally.$iter().map(Node::Block));
                    }
                    Stmt::With { expr, body, .. } => {
                        pending.push(Node::Value(expr));
                        pending.push(Node::Block(body));
                    }
                    Stmt::Break { .. } | Stmt::Continue { .. } => {}
                }
            }

            pub(crate) fn program_roots<'a>(
                program: &'a $($borrow)* Program,
                mut on_root: impl FnMut(&str, Node<'a>),
            ) {
                for item in &$($borrow)* program.items {
                    match item {
                        Item::Fn(function) => on_root(&function.name.name, Node::Block(&$($borrow)* function.body)),
                        Item::Type(record) => {
                            for method in &$($borrow)* record.methods {
                                let owner = format!("{}.{}", record.name.name, method.name.name);
                                on_root(&owner, Node::Block(&$($borrow)* method.body));
                            }
                        }
                        Item::Const(constant) => on_root(&constant.name.name, Node::Value(&$($borrow)* constant.value)),
                        Item::Store(store) => on_root(&store.name.name, Node::Value(&$($borrow)* store.value)),
                        Item::Enum(_) | Item::UseRs(_) | Item::UsePy(_) | Item::Task(_) | Item::Node(_) => {}
                    }
                }
            }
        }
    };
}

tree_walk!(read, iter, ['a],);
tree_walk!(write, iter_mut, [], mut);
