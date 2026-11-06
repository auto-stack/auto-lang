use super::*;

impl RustTrans {
    /// Identify empty declarations constrained by filling and a byte consumer.
    /// Preserve Rust's native element inference across both statement orders.
    /// Pointer slots distinguish declarations, as in dynamic-JSON tracking.
    pub(super) fn collect_byte_list_slots(
        block: &Body,
        params: &[AutoStr],
        signatures: &HashMap<AutoStr, Vec<Type>>,
    ) -> HashSet<usize> {
        type Scopes = Vec<HashMap<AutoStr, Option<usize>>>;
        #[derive(Default)]
        struct Constraints {
            consumed: HashSet<usize>,
            elements: HashSet<usize>,
            assignments: Vec<(usize, usize)>,
        }
        fn empty_literal(e: &Expr) -> bool {
            match e {
                Expr::Array(items) => items.is_empty(),
                Expr::Call(c) if c.args.args.is_empty() => matches!(c.name.as_ref(),
                    Expr::Dot(base, method) if method == "clone" && empty_literal(base)),
                _ => false,
            }
        }

        fn origin(e: &Expr, scopes: &Scopes) -> Option<usize> {
            match e {
                Expr::Ident(n) => scopes
                    .iter()
                    .rev()
                    .find_map(|s| s.get(n))
                    .copied()
                    .flatten(),
                Expr::Call(c) if c.args.args.is_empty() => match c.name.as_ref() {
                    Expr::Dot(base, m) if m == "clone" => origin(base, scopes),
                    _ => None,
                },
                _ => None,
            }
        }
        fn expression(
            e: &Expr,
            scopes: &Scopes,
            out: &mut Constraints,
            signatures: &HashMap<AutoStr, Vec<Type>>,
        ) {
            match e {
                Expr::Call(c) => {
                    // A typed call supplies an element type even when no
                    // runtime member is ever appended. This is the same
                    // signature table used by normal call-site borrowing.
                    if let Expr::Ident(name) = c.name.as_ref() {
                        if !scopes.iter().any(|scope| scope.contains_key(name)) {
                            if let Some(types) = signatures.get(name) {
                                for (arg, ty) in c.args.args.iter().zip(types) {
                                    let elem = match ty {
                                        Type::Slice(slice) => Some(slice.elem.as_ref()),
                                        Type::Array(array) => Some(array.elem.as_ref()),
                                        Type::List(elem) => Some(elem.as_ref()),
                                        _ => None,
                                    };
                                    let concrete = elem.is_some_and(|elem| {
                                        matches!(
                                            elem,
                                            Type::Int
                                                | Type::I64
                                                | Type::Uint
                                                | Type::U64
                                                | Type::USize
                                                | Type::Byte
                                                | Type::Char
                                                | Type::Float
                                                | Type::Double
                                                | Type::Bool
                                                | Type::StrSlice
                                                | Type::StrOwned
                                                | Type::StrFixed(_)
                                                | Type::CStrLit
                                        ) || RustTrans::is_json_type(elem)
                                    });
                                    if concrete {
                                        if let Arg::Pos(value) = arg {
                                            if let Some(slot) = origin(value, scopes) {
                                                out.elements.insert(slot);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let file_write = match c.name.as_ref() {
                        Expr::Dot(base, m) => {
                            matches!(base.as_ref(), Expr::Ident(n) if n == "File")
                                && m == "write_bytes"
                        }
                        Expr::Bina(base, Op::Dot, m) => {
                            matches!(base.as_ref(), Expr::Ident(n) if n == "File")
                                && matches!(m.as_ref(), Expr::Ident(n) if n == "write_bytes")
                        }
                        _ => false,
                    };
                    if file_write && !scopes.iter().any(|s| s.contains_key("File")) {
                        if let Some(Arg::Pos(data)) = c.args.args.get(1) {
                            if let Some(slot) = origin(data, scopes) {
                                out.consumed.insert(slot);
                            }
                        }
                    }
                    if let Expr::Dot(base, method) = c.name.as_ref() {
                        if method == "push" {
                            if let Some(slot) = origin(base, scopes) {
                                out.elements.insert(slot);
                            }
                        }
                    }
                    expression(&c.name, scopes, out, signatures);
                    for arg in &c.args.args {
                        if let Arg::Pos(e) = arg {
                            expression(e, scopes, out, signatures);
                        }
                    }
                }
                Expr::Bina(a, op, b) => {
                    if *op == Op::Asn {
                        if let Some(slot) = origin(a, scopes) {
                            if let Some(rhs) = origin(b, scopes) {
                                // Equal element types, not evidence of a type.
                                out.assignments.push((slot, rhs));
                            } else if !empty_literal(b) {
                                out.elements.insert(slot);
                            }
                        }
                    }
                    expression(a, scopes, out, signatures);
                    expression(b, scopes, out, signatures);
                }
                Expr::Index(a, b) | Expr::NullCoalesce(a, b) => {
                    expression(a, scopes, out, signatures);
                    expression(b, scopes, out, signatures);
                }
                Expr::Dot(base, _) | Expr::Mut(base) => expression(base, scopes, out, signatures),
                Expr::Array(items) | Expr::Tuple(items) => {
                    for item in items {
                        expression(item, scopes, out, signatures);
                    }
                }
                _ => {}
            }
        }
        fn body(
            block: &Body,
            scopes: &mut Scopes,
            out: &mut Constraints,
            signatures: &HashMap<AutoStr, Vec<Type>>,
        ) {
            scopes.push(HashMap::new());
            for stmt in &block.stmts {
                match stmt {
                    Stmt::Store(s) => {
                        expression(&s.expr, scopes, out, signatures);
                        let slot = if matches!(s.ty, Type::Unknown)
                            && matches!(&s.expr, Expr::Array(a) if a.is_empty())
                        {
                            Some(s as *const Store as usize)
                        } else {
                            origin(&s.expr, scopes)
                        };
                        scopes.last_mut().unwrap().insert(s.name.clone(), slot);
                    }
                    Stmt::Expr(e) => expression(e, scopes, out, signatures),
                    Stmt::Return(e) => expression(e, scopes, out, signatures),
                    Stmt::If(i) => {
                        for branch in &i.branches {
                            expression(&branch.cond, scopes, out, signatures);
                            body(&branch.body, scopes, out, signatures);
                        }
                        if let Some(b) = &i.else_ {
                            body(b, scopes, out, signatures);
                        }
                    }
                    Stmt::For(f) => {
                        expression(&f.range, scopes, out, signatures);
                        let names = match &f.iter {
                            Iter::Named(n) => vec![n.clone()],
                            Iter::Indexed(a, b) | Iter::Destructured(a, b) => {
                                vec![a.clone(), b.clone()]
                            }
                            _ => vec![],
                        };
                        scopes.push(names.into_iter().map(|n| (n, None)).collect());
                        body(&f.body, scopes, out, signatures);
                        scopes.pop();
                    }
                    Stmt::Try(t) => {
                        body(&t.body, scopes, out, signatures);
                        body(&t.catch_body, scopes, out, signatures);
                    }
                    Stmt::Block(b) => body(b, scopes, out, signatures),
                    Stmt::Is(i) => {
                        for branch in &i.branches {
                            let b = match branch {
                                crate::ast::IsBranch::EqBranch(_, b)
                                | crate::ast::IsBranch::IfBranch(_, b)
                                | crate::ast::IsBranch::ElseBranch(b) => b,
                            };
                            body(b, scopes, out, signatures);
                        }
                    }
                    _ => {}
                }
            }
            scopes.pop();
        }
        let mut scopes = vec![params.iter().cloned().map(|n| (n, None)).collect()];
        let mut out = Constraints::default();
        body(block, &mut scopes, &mut out, signatures);
        // Assignment equates element types in either direction. Only real
        // constraints seed propagation; an empty alias chain/cycle stays empty.
        loop {
            let before = out.elements.len();
            for &(lhs, rhs) in &out.assignments {
                if out.elements.contains(&lhs) || out.elements.contains(&rhs) {
                    out.elements.extend([lhs, rhs]);
                }
            }
            if out.elements.len() == before {
                break;
            }
        }
        out.consumed.intersection(&out.elements).copied().collect()
    }
}
