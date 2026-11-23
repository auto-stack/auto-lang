use std::collections::{BTreeMap, BTreeSet};

use crate::source::age_component::{
    ComponentClassPart, ComponentEntryValue, ComponentNode, ComponentStmt, ComponentViewItem,
};
use crate::source::age_package::widget_lists::SelectorInput;
use crate::source::ast::{AggArmBody, AggMatchArm, Arg, BinOp, Expr, ExprKind, MatchPattern};

type Inputs<'a> = BTreeMap<&'a str, &'a SelectorInput>;
type Locals = BTreeMap<String, BTreeSet<String>>;

/// Restrict record selection to the string values supplied at widget sites.
/// Keep runtime selector arguments and iteration order for the original view.
pub(super) fn specialize(component: &mut ComponentNode, bindings: &BTreeMap<usize, SelectorInput>) {
    let mut inputs: Inputs<'_> = component
        .params
        .iter()
        .filter(|param| {
            !component
                .fields
                .iter()
                .any(|field| field.name == param.name)
                && !component
                    .view_data
                    .iter()
                    .any(|field| field.name == param.name)
        })
        .filter_map(|param| Some((param.name.as_str(), bindings.get(&param.ty_span.start)?)))
        .collect();
    if inputs
        .values()
        .any(|value| matches!(value, SelectorInput::TextList(_)))
    {
        let mut escapes = ListEscapes::default();
        for field in &component.fields {
            escapes.expression(&field.init, false);
        }
        for handler in &component.handlers {
            escapes.statements(&handler.stmts);
            for channel in &handler.channel_calls {
                for arg in &channel.args {
                    escapes.expression(arg, false);
                }
            }
        }
        for member in &component.members {
            for value in member
                .value
                .iter()
                .chain(member.stmts.iter().map(|(_, value)| value))
            {
                escapes.expression(value, false);
            }
        }
        for item in &component.design {
            for value in [&item.value, &item.cond, &item.then_value, &item.else_value]
                .into_iter()
                .flatten()
            {
                escapes.expression(value, false);
            }
        }
        escapes.view(&component.view.tree);
        for (_, tree) in &component.view.named_trees {
            escapes.view(tree);
        }
        for instance in &component.view.scene_components {
            for attribute in &instance.attrs {
                escapes.expression(&attribute.value, false);
            }
        }
        inputs.retain(|name, value| {
            matches!(value, SelectorInput::Text(_)) || escapes.read_only(name)
        });
    }
    if inputs.is_empty() {
        return;
    }
    for handler in &mut component.handlers {
        let mut writes = BTreeSet::new();
        let mut pending = vec![handler.stmts.as_slice()];
        while let Some(body) = pending.pop() {
            for statement in body {
                match statement {
                    ComponentStmt::AssignLocal { name, .. } => {
                        writes.insert(name.name.clone());
                    }
                    ComponentStmt::For { body, .. } | ComponentStmt::While { body, .. } => {
                        pending.push(body);
                    }
                    ComponentStmt::If { then, els, .. } => {
                        pending.push(then);
                        pending.extend(els.as_deref());
                    }
                    _ => {}
                }
            }
        }
        specialize_body(&mut handler.stmts, &inputs, &writes, Locals::new());
    }
}

#[derive(Default)]
struct ListEscapes {
    names: BTreeSet<String>,
    unknown_expression: bool,
}

impl ListEscapes {
    fn read_only(&self, name: &str) -> bool {
        !self.unknown_expression && !self.names.contains(name)
    }

    fn statements(&mut self, statements: &[ComponentStmt]) {
        let mut pending = vec![statements];
        while let Some(body) = pending.pop() {
            for statement in body {
                match statement {
                    ComponentStmt::Assign(assign) => self.expression(&assign.rhs, false),
                    ComponentStmt::Let { value, .. }
                    | ComponentStmt::AssignLocal { value, .. }
                    | ComponentStmt::Effect { value, .. }
                    | ComponentStmt::Focus { target: value, .. }
                    | ComponentStmt::Call { call: value, .. } => self.expression(value, false),
                    ComponentStmt::Send { args, .. } => {
                        for arg in args {
                            self.expression(arg, false);
                        }
                    }
                    ComponentStmt::For { iter, body, .. } => {
                        self.expression(iter, true);
                        pending.push(body);
                    }
                    ComponentStmt::While { cond, body, .. } => {
                        self.expression(cond, false);
                        pending.push(body);
                    }
                    ComponentStmt::If {
                        cond, then, els, ..
                    } => {
                        self.expression(cond, false);
                        pending.push(then);
                        pending.extend(els.as_deref());
                    }
                    ComponentStmt::Try { body, catch, .. } => {
                        pending.push(body);
                        pending.push(catch);
                    }
                }
            }
        }
    }

    fn view(&mut self, items: &[ComponentViewItem]) {
        let mut pending = vec![items];
        while let Some(items) = pending.pop() {
            for item in items {
                match item {
                    ComponentViewItem::Text(leaf) => self.expression(&leaf.expr, false),
                    ComponentViewItem::Element(element) => {
                        if let Some(label) = &element.label {
                            self.expression(label, false);
                        }
                        pending.push(&element.items);
                    }
                    ComponentViewItem::For(iteration) => {
                        self.expression(&iteration.iter, true);
                        pending.push(&iteration.items);
                    }
                    ComponentViewItem::Cond(branch) => {
                        self.expression(&branch.cond, false);
                        pending.push(&branch.then_items);
                        pending.push(&branch.else_items);
                    }
                    ComponentViewItem::Entry(entry) => match &entry.value {
                        ComponentEntryValue::Expr(value) => self.expression(value, false),
                        ComponentEntryValue::Lambda { stmts, .. } => self.statements(stmts),
                        ComponentEntryValue::If {
                            cond,
                            then_value,
                            else_chain,
                            else_value,
                        } => {
                            for value in [cond, then_value, else_value] {
                                self.expression(value, false);
                            }
                            // PLAN-106：else-if 链臂 cond/value 一并走查。
                            for (c, v) in else_chain {
                                self.expression(c, false);
                                self.expression(v, false);
                            }
                        }
                        ComponentEntryValue::ClassList(parts) => {
                            for part in parts {
                                match part {
                                    ComponentClassPart::Value(value) => {
                                        self.expression(value, false)
                                    }
                                    ComponentClassPart::If {
                                        cond,
                                        then_value,
                                        else_value,
                                    } => {
                                        for value in [cond, then_value, else_value] {
                                            self.expression(value, false);
                                        }
                                    }
                                }
                            }
                        }
                        ComponentEntryValue::ChildEvent(_) => {}
                    },
                }
            }
        }
    }

    fn expression(&mut self, expr: &Expr, read: bool) {
        let mut pending = vec![(expr, read)];
        while let Some((expr, read)) = pending.pop() {
            match &expr.kind {
                ExprKind::Field { base, name } if matches!(&base.kind, ExprKind::Var(id) if id.name == "self") => {
                    if !read {
                        self.names.insert(name.name.clone());
                    }
                }
                ExprKind::Var(id) if !read => {
                    self.names.insert(id.name.clone());
                }
                ExprKind::Index { base, index } => {
                    pending.extend([(base.as_ref(), true), (index.as_ref(), false)]);
                }
                ExprKind::Method { recv, name, args }
                | ExprKind::MethodCall { recv, name, args, .. } => {
                    pending.push((recv, matches!(name.name.as_str(), "len" | "get")));
                    pending.extend(args.iter().map(|arg| (arg_expr(arg), false)));
                }
                ExprKind::Call { args, .. } => {
                    pending.extend(args.iter().map(|arg| (arg_expr(arg), false)))
                }
                ExprKind::Field { base, .. } => pending.push((base, false)),
                ExprKind::Binary { lhs, rhs, .. } => {
                    pending.extend([(lhs.as_ref(), false), (rhs.as_ref(), false)])
                }
                ExprKind::Unary { operand, .. } | ExprKind::TextDisplay(operand) => {
                    pending.push((operand, false))
                }
                ExprKind::ArrayLit(items) | ExprKind::TupleLit(items) => {
                    pending.extend(items.iter().map(|item| (item, false)))
                }
                ExprKind::ObjectLit { fields } => {
                    pending.extend(fields.iter().map(|entry| (&entry.value, false)))
                }
                ExprKind::CtorNamed { fields, .. } => {
                    pending.extend(fields.iter().map(|entry| (&entry.value, false)))
                }
                ExprKind::IntLit(_)
                | ExprKind::BoolLit(_)
                | ExprKind::StrLit(_)
                | ExprKind::TypedLit { .. }
                | ExprKind::Var(_)
                | ExprKind::CharLit(_)
                | ExprKind::NilLit => {}
                _ => self.unknown_expression = true,
            }
        }
    }
}

fn arg_expr(arg: &Arg) -> &Expr {
    match arg {
        Arg::Pos(value) | Arg::MovePos(value) | Arg::Named { value, .. } => value,
    }
}

fn property<'a>(expr: &Expr, inputs: &Inputs<'a>) -> Option<&'a SelectorInput> {
    let ExprKind::Field { base, name } = &expr.kind else {
        return None;
    };
    if !matches!(&base.kind, ExprKind::Var(root) if root.name == "self") {
        return None;
    }
    inputs.get(name.name.as_str()).copied()
}

fn text(expr: &Expr, inputs: &Inputs<'_>, locals: &Locals) -> Option<BTreeSet<String>> {
    match &expr.kind {
        ExprKind::StrLit(value) => Some([value.clone()].into()),
        ExprKind::Var(id) => locals.get(&id.name).cloned(),
        _ => match property(expr, inputs)? {
            SelectorInput::Text(value) => Some(value.clone()),
            SelectorInput::TextList(_) => None,
        },
    }
}

fn specialize_body(
    statements: &mut [ComponentStmt],
    inputs: &Inputs<'_>,
    writes: &BTreeSet<String>,
    mut locals: Locals,
) {
    for statement in statements {
        match statement {
            ComponentStmt::Assign(assign) => specialize_expr(&mut assign.rhs, inputs, &locals),
            ComponentStmt::AssignLocal { value, .. }
            | ComponentStmt::Effect { value, .. }
                    | ComponentStmt::Focus { target: value, .. }
            | ComponentStmt::Call { call: value, .. } => {
                specialize_expr(value, inputs, &locals);
            }
            ComponentStmt::Send { args, .. } => {
                for arg in args {
                    specialize_expr(arg, inputs, &locals);
                }
            }
            ComponentStmt::Let { name, value, .. } => {
                specialize_expr(value, inputs, &locals);
                let known = (!writes.contains(&name.name))
                    .then(|| text(value, inputs, &locals))
                    .flatten();
                locals.remove(&name.name);
                if let Some(value) = known {
                    locals.insert(name.name.clone(), value);
                }
            }
            ComponentStmt::For {
                name, iter, body, ..
            } => {
                specialize_expr(iter, inputs, &locals);
                let mut inner = locals.clone();
                inner.remove(&name.name);
                if !writes.contains(&name.name) {
                    if let Some(SelectorInput::TextList(values)) = property(iter, inputs) {
                        inner.insert(name.name.clone(), values.clone());
                    }
                }
                specialize_body(body, inputs, writes, inner);
            }
            ComponentStmt::While { cond, body, .. } => {
                specialize_expr(cond, inputs, &locals);
                specialize_body(body, inputs, writes, locals.clone());
            }
            ComponentStmt::If {
                cond, then, els, ..
            } => {
                specialize_expr(cond, inputs, &locals);
                specialize_body(then, inputs, writes, locals.clone());
                if let Some(otherwise) = els {
                    specialize_body(otherwise, inputs, writes, locals.clone());
                }
            }
            ComponentStmt::Try { body, catch, .. } => {
                specialize_body(body, inputs, writes, locals.clone());
                specialize_body(catch, inputs, writes, locals.clone());
            }
        }
    }
}

fn specialize_expr(expr: &mut Expr, inputs: &Inputs<'_>, locals: &Locals) {
    let mut pending = vec![expr];
    while let Some(expr) = pending.pop() {
        if let Some(mut replacement) = specialized_index(expr, inputs, locals) {
            replacement.span = expr.span;
            *expr = replacement;
            continue;
        }
        match &mut expr.kind {
            ExprKind::Index { base, index } => {
                pending.push(base);
                pending.push(index);
            }
            ExprKind::Field { base, .. } => pending.push(base),
            ExprKind::Binary { lhs, rhs, .. } => {
                pending.push(lhs);
                pending.push(rhs);
            }
            ExprKind::Unary { operand, .. } | ExprKind::TextDisplay(operand) => {
                pending.push(operand)
            }
            ExprKind::ArrayLit(items) | ExprKind::TupleLit(items) => pending.extend(items),
            ExprKind::ObjectLit { fields } => {
                pending.extend(fields.iter_mut().map(|entry| &mut entry.value))
            }
            ExprKind::CtorNamed { fields, .. } => {
                pending.extend(fields.iter_mut().map(|entry| &mut entry.value))
            }
            ExprKind::Call { args, .. } => push_args(args, &mut pending),
            ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
                pending.push(recv);
                push_args(args, &mut pending);
            }
            _ => {}
        }
    }
}

fn specialized_index(expr: &Expr, inputs: &Inputs<'_>, locals: &Locals) -> Option<Expr> {
    let ExprKind::Index { base, index } = &expr.kind else {
        return None;
    };
    if !matches!(base.kind, ExprKind::Var(_)) || matches!(index.kind, ExprKind::StrLit(_)) {
        return None;
    }
    let keys = text(index, inputs, locals).filter(|keys| !keys.is_empty())?;
    let keys: Vec<_> = keys.iter().map(String::as_str).collect();
    Some(select_key(base, index, &keys))
}

fn select_key(base: &Expr, index: &Expr, keys: &[&str]) -> Expr {
    let span = base.span.join(index.span);
    let kind = if keys.len() == 1 {
        ExprKind::Index {
            base: Box::new(base.clone()),
            index: Box::new(Expr {
                kind: ExprKind::StrLit(keys[0].into()),
                span: index.span,
            }),
        }
    } else {
        let (left, right) = keys.split_at(keys.len() / 2);
        ExprKind::IsExpr {
            scrutinee: Box::new(matches_key(index, left)),
            arms: [(true, left), (false, right)]
                .into_iter()
                .map(|(matches, keys)| AggMatchArm {
                    pat: MatchPattern::BoolLit(matches),
                    body: AggArmBody::Expr(select_key(base, index, keys)),
                    guard: None,
                    span,
                })
                .collect(),
        }
    };
    Expr { kind, span }
}

fn matches_key(index: &Expr, keys: &[&str]) -> Expr {
    let span = index.span;
    let (op, lhs, rhs) = if keys.len() == 1 {
        (
            BinOp::Eq,
            index.clone(),
            Expr {
                kind: ExprKind::StrLit(keys[0].into()),
                span,
            },
        )
    } else {
        let (left, right) = keys.split_at(keys.len() / 2);
        (
            BinOp::Or,
            matches_key(index, left),
            matches_key(index, right),
        )
    };
    Expr {
        kind: ExprKind::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        span,
    }
}

fn push_args<'a>(args: &'a mut [Arg], pending: &mut Vec<&'a mut Expr>) {
    pending.extend(args.iter_mut().map(|arg| match arg {
        Arg::Pos(value) | Arg::MovePos(value) | Arg::Named { value, .. } => value,
    }));
}
