use super::*;

impl RustTrans {
    pub(super) fn expr_condition(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        if self.expr_is_dynamic_json(expr) {
            self.a2r_std_used.set(true);
            write!(out, "a2r_std::json_condition(&(")?;
            self.expr(expr, out)?;
            write!(out, "))")?;
            Ok(())
        } else {
            self.expr(expr, out)
        }
    }
    pub(super) fn is_json_type(ty: &Type) -> bool {
        matches!(ty, Type::User(t) if matches!(t.name.as_str(), "Value" | "JsonValue"))
    }

    pub(super) fn expr_is_dynamic_json(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => self
                .local_var_types
                .get(name)
                .is_some_and(Self::is_json_type),
            Expr::Dot(base, _) | Expr::Index(base, _) => self.expr_is_dynamic_json(base),
            Expr::Bina(base, Op::Dot, _) => self.expr_is_dynamic_json(base),
            Expr::Call(call) => match call.name.as_ref() {
                Expr::Dot(base, method) => {
                    (matches!(base.as_ref(), Expr::Ident(n) if matches!(n.as_str(), "json" | "Json") && !self.local_var_types.contains_key(n))
                        && matches!(method.as_str(), "parse" | "get" | "get_at"))
                        || (method == "clone" && self.expr_is_dynamic_json(base))
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Resolve null-initialized slots to their own declarations, so an inner
    /// declaration with the same spelling cannot change an outer slot's type.
    pub(super) fn collect_json_null_slots(decl: &Fn) -> HashSet<usize> {
        let initial = decl
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        Self::collect_json_null_slots_in(&decl.body, &initial)
    }

    pub(super) fn collect_json_null_slots_in(
        block: &Body,
        initial: &HashMap<AutoStr, Type>,
    ) -> HashSet<usize> {
        #[derive(Clone, Copy)]
        struct Binding {
            slot: Option<usize>,
            json: bool,
        }
        type Scopes = Vec<HashMap<AutoStr, Binding>>;
        fn json(expr: &Expr, scopes: &Scopes) -> bool {
            match expr {
                Expr::Ident(n) => scopes
                    .iter()
                    .rev()
                    .find_map(|s| s.get(n))
                    .is_some_and(|b| b.json),
                Expr::Dot(base, _) | Expr::Index(base, _) => json(base, scopes),
                Expr::Call(c) => match c.name.as_ref() {
                    Expr::Dot(base, m) => {
                        (matches!(base.as_ref(), Expr::Ident(n) if matches!(n.as_str(), "json" | "Json") && !scopes.iter().any(|s|s.contains_key(n)))
                            && matches!(m.as_str(), "parse" | "get" | "get_at"))
                            || (m == "clone" && json(base, scopes))
                    }
                    _ => false,
                },
                _ => false,
            }
        }
        fn body(block: &Body, scopes: &mut Scopes, out: &mut HashSet<usize>) {
            scopes.push(HashMap::new());
            for stmt in &block.stmts {
                match stmt {
                    Stmt::Store(s) => {
                        let is_json = RustTrans::is_json_type(&s.ty) || json(&s.expr, scopes);
                        let null = matches!(s.expr, Expr::Null | Expr::Nil)
                            && (matches!(s.ty, Type::Unknown)
                                || matches!(&s.ty, Type::Option(t) if matches!(t.as_ref(), Type::Unknown)));
                        scopes.last_mut().unwrap().insert(
                            s.name.clone(),
                            Binding {
                                slot: null.then_some(s as *const Store as usize),
                                json: is_json,
                            },
                        );
                    }
                    Stmt::Expr(Expr::Bina(lhs, Op::Asn, rhs)) => {
                        if json(rhs, scopes) {
                            if let Expr::Ident(n) = lhs.as_ref() {
                                if let Some(b) = scopes.iter_mut().rev().find_map(|s| s.get_mut(n))
                                {
                                    b.json = true;
                                    if let Some(slot) = b.slot {
                                        out.insert(slot);
                                    }
                                }
                            }
                        }
                    }
                    Stmt::If(i) => {
                        for b in &i.branches {
                            body(&b.body, scopes, out);
                        }
                        if let Some(b) = &i.else_ {
                            body(b, scopes, out);
                        }
                    }
                    Stmt::Try(t) => {
                        body(&t.body, scopes, out);
                        body(&t.catch_body, scopes, out);
                    }
                    Stmt::For(f) => {
                        let element_json = json(&f.range, scopes);
                        let names: Vec<_> = match &f.iter {
                            Iter::Named(n) => vec![n.clone()],
                            Iter::Indexed(a, b) | Iter::Destructured(a, b) => {
                                vec![a.clone(), b.clone()]
                            }
                            _ => vec![],
                        };
                        scopes.push(
                            names
                                .into_iter()
                                .map(|name| {
                                    (
                                        name,
                                        Binding {
                                            slot: None,
                                            json: element_json,
                                        },
                                    )
                                })
                                .collect(),
                        );
                        body(&f.body, scopes, out);
                        scopes.pop();
                    }
                    Stmt::Block(b) => body(b, scopes, out),
                    _ => {}
                }
            }
            scopes.pop();
        }
        let mut scopes = vec![initial
            .iter()
            .map(|(name, ty)| {
                (
                    name.clone(),
                    Binding {
                        slot: None,
                        json: Self::is_json_type(ty),
                    },
                )
            })
            .collect()];
        let mut out = HashSet::new();
        body(block, &mut scopes, &mut out);
        out
    }
}

#[cfg(test)]
mod scope_tests {
    use super::*;

    #[test]
    fn inner_null_and_loop_bindings_do_not_retype_outer_slots() {
        let mut parser = crate::Parser::from(
            r#"
fn scopes() {
    var slot = null
    {
        var slot = null
        slot = json.parse("{}")
    }
    for slot in [1] { slot = json.parse("{}") }
}
"#,
        );
        let code = parser.parse().unwrap();
        let Stmt::Fn(decl) = &code.stmts[0] else {
            panic!("fn")
        };
        let slots = RustTrans::collect_json_null_slots(decl);
        let Stmt::Store(outer) = &decl.body.stmts[0] else {
            panic!("outer")
        };
        let Stmt::Block(block) = &decl.body.stmts[1] else {
            panic!("block")
        };
        let Stmt::Store(inner) = &block.stmts[0] else {
            panic!("inner")
        };
        assert!(!slots.contains(&(outer as *const Store as usize)));
        assert!(slots.contains(&(inner as *const Store as usize)));
        assert_eq!(slots.len(), 1);
    }
}
