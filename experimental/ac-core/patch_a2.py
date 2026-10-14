p = 'src/source/adapter.rs'
s = open(p, encoding='utf-8').read()
orig = s

# ---- Assign with path ----
s = s.replace("""                RStmt::Assign { local, value, span } => {""",
"""                RStmt::Assign {
                    local,
                    path,
                    value,
                    span,
                } => {""")
s = s.replace("""                    plan.places.push(PNode {
                        local: local.0,
                        vt: f.locals[local.0].vt,
                        span: *span,
                    });""",
"""                    // PLAN-771: walk the field path to the final type; a
                    // path place writes through the root local's storage.
                    let mut vt = f.locals[local.0].vt;
                    for (ri, fi) in path {
                        vt = self.program.records[*ri].fields[*fi].vt;
                    }
                    plan.places.push(PNode {
                        local: local.0,
                        path: path.iter().map(|(_, fi)| *fi as u32).collect(),
                        vt,
                        span: *span,
                    });""")

# ---- plan_leaf record arms ----
s = s.replace("""            RExprKind::CallIntrinsic { symbol } => {""",
"""            RExprKind::CtorPos { rec, args } => {
                // positional: eval order = source order; binding i -> i
                let mut eval = Vec::new();
                for a in args {
                    eval.push(self.plan_expr(plan, f, &a.value));
                }
                let bindings = (0..eval.len() as u32)
                    .map(|i| (i, i))
                    .collect::<Vec<(u32, u32)>>();
                plan.exprs.push(ENode {
                    kind: EKind::RecordCtor {
                        rec: *rec,
                        eval,
                        bindings,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CtorNamed { rec, fields } => {
                // named: eval order = WRITTEN order; bindings map each
                // declaration field to its argument (PLAN-771 §5.2: never
                // re-sorted by field order)
                let mut eval = Vec::new();
                for (_, v) in fields {
                    eval.push(self.plan_expr(plan, f, v));
                }
                let mut bindings = Vec::new();
                for (ai, (name, _)) in fields.iter().enumerate() {
                    if let Some(fi) = self.program.records[*rec]
                        .fields
                        .iter()
                        .position(|fl| fl.name == *name)
                    {
                        bindings.push((fi as u32, ai as u32));
                    }
                }
                bindings.sort_by_key(|(fi, _)| *fi);
                plan.exprs.push(ENode {
                    kind: EKind::RecordCtor {
                        rec: *rec,
                        eval,
                        bindings,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::FieldGet { base, path } => {
                // flatten chained single-segment FieldGets into one path
                let mut segs: Vec<u32> = path.iter().map(|(_, fi)| *fi as u32).collect();
                let mut cur = base;
                loop {
                    let next = match &cur.kind {
                        RExprKind::FieldGet {
                            base: inner,
                            path: ip,
                        } => {
                            let mut more: Vec<u32> =
                                ip.iter().map(|(_, fi)| *fi as u32).collect();
                            more.extend(segs);
                            segs = more;
                            inner
                        }
                        _ => break,
                    };
                    cur = next;
                }
                let base_slot = self.plan_expr(plan, f, cur);
                plan.exprs.push(ENode {
                    kind: EKind::FieldGet {
                        base: base_slot,
                        path: segs,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::MethodCall {
                fn_index,
                recv,
                args,
            } => {
                let callee = self.program.fns.get(*fn_index);
                let mut eval = Vec::new();
                let mut bindings = Vec::new();
                match recv {
                    // instance: the receiver is evaluated EXACTLY ONCE,
                    // first, as a view borrow (param 0)
                    Some(r) => {
                        let rs = self.plan_expr(plan, f, r);
                        plan.exprs.push(ENode {
                            kind: EKind::BorrowView { base: rs },
                            vt: r.vt,
                            span: r.span,
                        });
                        let bv = plan.exprs.len() - 1;
                        eval.push(bv);
                        bindings.push((0u32, 0u32));
                    }
                    None => {}
                }
                let recv_shift = if recv.is_some() { 1 } else { 0 };
                for a in args {
                    eval.push(self.plan_arg(plan, f, a, callee, args));
                }
                if let Some(c) = callee {
                    for (idx, a) in args.iter().enumerate() {
                        if let Some(pos) = param_position(idx, a, c) {
                            bindings.push((((pos + recv_shift) as u32), idx as u32 + recv_shift));
                        }
                    }
                }
                plan.exprs.push(ENode {
                    kind: EKind::Call {
                        callee_fn: Some(*fn_index),
                        callee_intrinsic: None,
                        eval,
                        bindings,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::BorrowMut { local } => {
                plan.exprs.push(ENode {
                    kind: EKind::BorrowMut { local: local.0 },
                    vt: f.locals[local.0].vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CallIntrinsic { symbol } => {""")

# borrow wrapping for record args of plain calls: rewrite the CallFn arm
s = s.replace("""            RExprKind::CallFn { fn_index, args } => {
                let mut eval = Vec::new();
                for a in args {
                    eval.push(self.plan_expr(plan, f, &a.value));
                }
                let mut bindings = Vec::new();
                if let Some(callee) = self.program.fns.get(*fn_index) {
                    for (idx, a) in args.iter().enumerate() {
                        if let Some(pos) = param_position(idx, a, callee) {
                            bindings.push((pos as u32, idx as u32));
                        }
                    }
                }
                plan.exprs.push(ENode {
                    kind: EKind::Call {
                        callee_fn: Some(*fn_index),
                        callee_intrinsic: None,
                        eval,
                        bindings,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }""",
"""            RExprKind::CallFn { fn_index, args } => {
                let callee = self.program.fns.get(*fn_index);
                let mut eval = Vec::new();
                for a in args {
                    eval.push(self.plan_arg(plan, f, a, callee, args));
                }
                let mut bindings = Vec::new();
                if let Some(c) = callee {
                    for (idx, a) in args.iter().enumerate() {
                        if let Some(pos) = param_position(idx, a, c) {
                            bindings.push((pos as u32, idx as u32));
                        }
                    }
                }
                plan.exprs.push(ENode {
                    kind: EKind::Call {
                        callee_fn: Some(*fn_index),
                        callee_intrinsic: None,
                        eval,
                        bindings,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }""")

# plan_arg helper: record args become borrows per the callee's param mode
s = s.replace("""    fn plan_leaf(&mut self, plan: &mut FnPlan, f: &ResolvedFn, e: &ResolvedExpr) -> usize {""",
"""    /// Plan one call argument: a record-typed argument to a view param is
    /// wrapped in a view borrow (ABI-D); `mut` record params receive the
    /// explicit `p.mut` borrow unchanged (a missing `.mut` stays a plain
    /// value and is rejected downstream); scalar args pass by value
    /// (PLAN-771 / 774 D-04).
    fn plan_arg(
        &mut self,
        plan: &mut FnPlan,
        f: &ResolvedFn,
        a: &ResolvedArg,
        idx: usize,
        callee: Option<&ResolvedFn>,
    ) -> usize {
        let is_record = matches!(a.value.vt, VT::Rec(_));
        let is_borrow_mut = matches!(a.value.kind, RExprKind::BorrowMut { .. });
        if !is_record || is_borrow_mut {
            return self.plan_expr(plan, f, &a.value);
        }
        let pos = callee.and_then(|c| match &a.name {
            None => {
                if idx < c.params.len() {
                    Some(idx)
                } else {
                    None
                }
            }
            Some(n) => c.param_names.iter().position(|p| *p == n.name),
        });
        let param_mut = pos
            .and_then(|i| callee.and_then(|c| c.param_modes.get(i).copied()))
            .unwrap_or(false);
        let slot = self.plan_expr(plan, f, &a.value);
        if param_mut {
            return slot; // missing `.mut` — plain value, rejected downstream
        }
        plan.exprs.push(ENode {
            kind: EKind::BorrowView { base: slot },
            vt: a.value.vt,
            span: a.value.span,
        });
        plan.exprs.len() - 1
    }

    fn plan_leaf(&mut self, plan: &mut FnPlan, f: &ResolvedFn, e: &ResolvedExpr) -> usize {""")

# fix the two plan_arg call sites to pass idx
s = s.replace("""                for a in args {
                    eval.push(self.plan_arg(plan, f, a, callee, args));
                }
                let mut bindings = Vec::new();
                if let Some(c) = callee {
                    for (idx, a) in args.iter().enumerate() {
                        if let Some(pos) = param_position(idx, a, c) {
                            bindings.push((((pos + recv_shift) as u32), idx as u32 + recv_shift));
                        }
                    }
                }""",
"""                for (idx, a) in args.iter().enumerate() {
                    eval.push(self.plan_arg(plan, f, a, idx, callee));
                }
                let mut bindings = Vec::new();
                if let Some(c) = callee {
                    for (idx, a) in args.iter().enumerate() {
                        if let Some(pos) = param_position(idx, a, c) {
                            bindings.push((((pos + recv_shift) as u32), idx as u32 + recv_shift));
                        }
                    }
                }""")
s = s.replace("""                let callee = self.program.fns.get(*fn_index);
                let mut eval = Vec::new();
                for a in args {
                    eval.push(self.plan_arg(plan, f, a, callee, args));
                }""",
"""                let callee = self.program.fns.get(*fn_index);
                let mut eval = Vec::new();
                for (idx, a) in args.iter().enumerate() {
                    eval.push(self.plan_arg(plan, f, a, idx, callee));
                }""")
    fn plan_leaf(&mut self, plan: &mut FnPlan, f: &ResolvedFn, e: &ResolvedExpr) -> usize {""",
"""    /// Plan one call argument: a record-typed argument to a view param is
    /// wrapped in a view borrow (ABI-D); `mut` record params receive the
    /// explicit `p.mut` borrow unchanged (typecheck rejected anything
    /// else); scalar args pass by value (PLAN-771 / 774 D-04).
    fn plan_arg(
        &mut self,
        plan: &mut FnPlan,
        f: &ResolvedFn,
        a: &ResolvedArg,
        callee: Option<&ResolvedFn>,
        all_args: &[ResolvedArg],
    ) -> usize {
        // scalar or already-borrowed args plan as-is
        if !matches!(a.value.vt, VT::Rec(_))
            || matches!(
                a.value.kind,
                RExprKind::BorrowMut { .. } | RExprKind::ReadLocal(_) if false
            )
        {
            let slot = self.plan_expr(plan, f, &a.value);
            // BorrowMut args pass through; plain record values may need a
            // view wrap below, so only return early for scalars here.
            if !matches!(a.value.vt, VT::Rec(_)) {
                return slot;
            }
            if matches!(a.value.kind, RExprKind::BorrowMut { .. }) {
                return slot;
            }
            // fall through to the wrap decision
            let pos = callee.and_then(|c| match &a.name {
                None => all_args
                    .iter()
                    .position(|x| std::ptr::eq(x, a))
                    .map(|i| i.min(c.params.len().saturating_sub(1))),
                Some(n) => c.param_names.iter().position(|p| *p == n.name),
            });
            let is_mut = pos.map(|i| callee.unwrap().param_modes.get(i).copied() == Some(true));
            if is_mut == Some(true) {
                // missing `.mut`: keep the plain value; typecheck/verify
                // reject it (ownership-required)
                return slot;
            }
            let _ = slot;
            // view wrap
            let slot2 = self.plan_expr(plan, f, &a.value);
            plan.exprs.push(ENode {
                kind: EKind::BorrowView { base: slot2 },
                vt: a.value.vt,
                span: a.value.span,
            });
            return plan.exprs.len() - 1;
        }
        self.plan_expr(plan, f, &a.value)
    }

    fn plan_leaf(&mut self, plan: &mut FnPlan, f: &ResolvedFn, e: &ResolvedExpr) -> usize {""")

assert s != orig
open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('adapter patch B ok')
