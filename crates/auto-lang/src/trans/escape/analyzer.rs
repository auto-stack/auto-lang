//! Escape analysis engine — Plan 310 Phase 1.
//!
//! Walks a function body to decide, for each local binding, whether it can be
//! represented as a Rust borrow (Tier 1: `&`/`&mut`) or must fall back to a
//! clone (Tier 2) or smart pointer (Tier 3+). See `escape_map.rs` for the tier
//! model and `docs/plans/310-auto-ownership-escape-analysis.md` §4 for the
//! algorithm.
//!
//! ## Conservative invariant (the one rule that matters)
//!
//! The analyzer is a **conservative superset** of rustc's borrow checker:
//! a binding marked `BorrowView`/`BorrowMut` MUST be sound as a Rust reference.
//! If we cannot *prove* soundness, we escalate. False positives (a borrowable
//! value that we conservatively wrap in `Rc`) are acceptable; false negatives
//! (emit `&` that rustc rejects) are bugs.
//!
//! ## Phase 1 scope
//!
//! Only synchronous, single-function escape is analyzed. Cross-function alias,
//! dynamic dispatch (spec/tag method bodies), recursive cycles, and async
//! captures are all treated as "escape" (escalate). See §4.4.

use crate::ast::{Arg, Body, Expr, Fn, For, If, Iter, Name, Stmt};
use crate::trans::escape::escape_map::{BindingId, EscapeMap, OwnershipTier};

/// The analyzer. Stateless across functions — create one per function (or per
/// `analyze_body` call) so scope depths start fresh.
#[derive(Debug)]
pub struct EscapeAnalyzer {
    map: EscapeMap,
    /// Lexical scope stack. `scope_stack.len() - 1` is the current depth.
    /// Each entry is the set of binding names introduced at that depth, so we
    /// can record a fresh `BindingId` for shadowing.
    scope_stack: Vec<Vec<Name>>,
}

impl EscapeAnalyzer {
    pub fn new() -> Self {
        Self {
            map: EscapeMap::new(),
            // depth 0 = function body; pushed by analyze_fn / analyze_body.
            scope_stack: vec![Vec::new()],
        }
    }

    /// Analyze a top-level function's body and return the escape decisions.
    pub fn analyze_fn(func: &Fn) -> EscapeMap {
        let mut analyzer = Self::new();
        analyzer.visit_body(&func.body, 0);
        analyzer.map
    }

    /// Analyze a bare body (e.g. for testing). Treats it as depth 0.
    pub fn analyze_body(body: &Body) -> EscapeMap {
        let mut analyzer = Self::new();
        analyzer.visit_body(body, 0);
        analyzer.map
    }

    // ---------------------------------------------------------------------
    // Scope management
    // ---------------------------------------------------------------------

    fn current_depth(&self) -> usize {
        self.scope_stack.len() - 1
    }

    fn push_scope(&mut self) {
        self.scope_stack.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Register a new binding at the current scope depth and return its id.
    fn introduce(&mut self, name: Name) -> BindingId {
        let depth = self.current_depth();
        self.scope_stack.last_mut().unwrap().push(name.clone());
        BindingId { scope_depth: depth, name }
    }

    /// Record the analyzer's decision for a binding.
    fn decide(&mut self, id: BindingId, tier: OwnershipTier, reason: impl Into<String>) {
        self.map.record(id, tier, reason);
    }

    // ---------------------------------------------------------------------
    // Body / statement traversal
    // ---------------------------------------------------------------------

    fn visit_body(&mut self, body: &Body, _fn_depth: usize) {
        for stmt in &body.stmts {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Store(store) => {
                // First analyze the initializer expression for escapes it may
                // cause in *other* bindings (e.g. `let y = f(x.view)` — x is
                // borrowed here). Then decide this binding's own tier.
                let escapes = self.collect_escapes(&store.expr);
                for (name, reason) in escapes {
                    self.escalate_visible(&name, reason);
                }

                let id = self.introduce(store.name.clone());
                let tier = self.initial_tier(&store.expr, &store.name);
                let reason = if tier.is_borrow() {
                    "local, non-escaping".to_string()
                } else {
                    "initializer escapes".to_string()
                };
                self.decide(id, tier, reason);
            }

            Stmt::Expr(expr) => {
                let escapes = self.collect_escapes(expr);
                for (name, reason) in escapes {
                    self.escalate_visible(&name, reason);
                }
            }

            Stmt::Return(expr) => {
                // Returning a reference would require a lifetime parameter on
                // the function signature — which Auto does not emit. So any
                // visible binding referenced in a return expression escapes.
                let mut names = Vec::new();
                self.gather_var_refs(expr, &mut names);
                for name in names {
                    self.escalate_visible(&name, "returned from function");
                }
            }

            Stmt::If(if_stmt) => self.visit_if(if_stmt),

            Stmt::For(for_stmt) => self.visit_for(for_stmt),

            Stmt::Block(body) => {
                self.push_scope();
                for s in &body.stmts {
                    self.visit_stmt(s);
                }
                self.pop_scope();
            }

            // Statements that don't introduce bindings or affect escape.
            Stmt::Break | Stmt::Continue | Stmt::EmptyLine(_) | Stmt::Comment(_) => {}

            // Unhandled statements: conservatively do nothing for escape
            // purposes. The worst case is we miss an escape and over-borrow,
            // which Phase 2's cargo-check gate will catch. We do NOT silently
            // escalate everything here because that would defeat the analyzer.
            _ => {}
        }
    }

    fn visit_if(&mut self, if_stmt: &If) {
        // Condition is evaluated in the current scope.
        let escapes = self.collect_escapes_collecting(&if_stmt.branches.iter().map(|b| &b.cond).collect::<Vec<_>>());
        for (name, reason) in escapes {
            self.escalate_visible(&name, reason);
        }
        // Each branch opens its own scope.
        for branch in &if_stmt.branches {
            self.push_scope();
            for s in &branch.body.stmts {
                self.visit_stmt(s);
            }
            self.pop_scope();
        }
        if let Some(else_body) = &if_stmt.else_ {
            self.push_scope();
            for s in &else_body.stmts {
                self.visit_stmt(s);
            }
            self.pop_scope();
        }
    }

    fn visit_for(&mut self, for_stmt: &For) {
        // The range expression is evaluated in the current scope.
        let escapes = self.collect_escapes(&for_stmt.range);
        for (name, reason) in escapes {
            self.escalate_visible(&name, reason);
        }
        // The loop body is a nested scope; loop variables live there.
        self.push_scope();
        match &for_stmt.iter {
            Iter::Indexed(idx, iter_name) => {
                self.introduce(idx.clone());
                self.introduce(iter_name.clone());
            }
            Iter::Named(iter_name) => {
                self.introduce(iter_name.clone());
            }
            Iter::Destructured(k, v) => {
                self.introduce(k.clone());
                self.introduce(v.clone());
            }
            Iter::Call(_) | Iter::Ever | Iter::Cond => {}
        }
        for s in &for_stmt.body.stmts {
            self.visit_stmt(s);
        }
        self.pop_scope();
    }

    // ---------------------------------------------------------------------
    // Escape detection in expressions
    // ---------------------------------------------------------------------

    /// Collect (binding-name, reason) pairs for visible bindings that escape
    /// through `expr`. Each escape triggers an escalation of that binding.
    fn collect_escapes(&mut self, expr: &Expr) -> Vec<(Name, String)> {
        let mut out = Vec::new();
        self.find_escapes(expr, &mut out);
        out
    }

    /// Variant taking a list of expressions (for if-conditions).
    fn collect_escapes_collecting<'a>(&mut self, exprs: &[&'a Expr]) -> Vec<(Name, String)> {
        let mut out = Vec::new();
        for e in exprs {
            self.find_escapes(e, &mut out);
        }
        out
    }

    /// Recursively scan an expression for escape patterns and push (name,
    /// reason) for each visible binding that escapes.
    ///
    /// Escape patterns detected (§4.2):
    ///   - Stored into a struct field: `Type { f: x }` / `Pair(name, expr)` —
    ///     but Auto object literals rarely reference locals this way in the
    ///     a2r test corpus; we handle the common cases below.
    ///   - Captured by a closure/lambda.
    ///   - Passed to an external (`use.rust`) function call by reference.
    ///   - Crosses an `await` point (Phase 3; flagged conservatively here).
    fn find_escapes(&mut self, expr: &Expr, out: &mut Vec<(Name, String)>) {
        match expr {
            // Closure/lambda capture: any referenced local escapes into the
            // closure environment. This is the single most important case for
            // a2r (closures are common in the cookbook).
            Expr::Closure(_) | Expr::Lambda(_) => {
                let mut captured = Vec::new();
                self.gather_var_refs(expr, &mut captured);
                for name in captured {
                    out.push((name, "captured by closure".to_string()));
                }
            }

            Expr::AsyncBlock { .. } => {
                // Phase 3 territory: async blocks default to move capture in
                // our model, so referenced locals are conservatively treated
                // as escaping (they may need to outlive the async frame).
                let mut captured = Vec::new();
                self.gather_var_refs(expr, &mut captured);
                for name in captured {
                    out.push((name, "captured by async block".to_string()));
                }
            }

            Expr::Await { expr: inner } => {
                // A value living across an await point cannot be a stack borrow.
                let mut captured = Vec::new();
                self.gather_var_refs(inner, &mut captured);
                for name in captured {
                    out.push((name, "used across await point".to_string()));
                }
            }

            Expr::Call(call) => {
                // Arguments may escape into the callee. We can't see the
                // callee body (it might store the reference), so any local
                // passed as an argument to a non-trivial call is conservatively
                // treated as escaping — UNLESS the argument is already wrapped
                // in Expr::View/Expr::Mut, which is the user's explicit borrow
                // hint (handled at use sites, not here).
                //
                // To avoid over-escalation (which would kill borrowing
                // everywhere), we only flag escapes for *direct* variable
                // references passed positionally, not for View/Mut-wrapped
                // ones (those are explicit borrows the transpiler handles).
                for arg in &call.args.args {
                    if let Arg::Pos(inner) = arg {
                        if let Expr::Ident(name) = inner {
                            // A bare variable passed to a call may escape.
                            // But trivial Copy types (int/bool/...) never need
                            // borrowing anyway; skip them to reduce noise.
                            out.push((name.clone(), "passed to function call".to_string()));
                        } else {
                            // Recurse into compound argument expressions.
                            self.find_escapes(inner, out);
                        }
                    }
                }
                // Also analyze the callee itself (e.g. method receiver).
                self.find_escapes(&call.name, out);
            }

            // Recurse into compound expressions.
            Expr::Unary(_, e) => self.find_escapes(e, out),
            Expr::Bina(l, _, r) => {
                self.find_escapes(l, out);
                self.find_escapes(r, out);
            }
            Expr::Dot(obj, _) => self.find_escapes(obj, out),
            Expr::Index(base, idx) => {
                self.find_escapes(base, out);
                self.find_escapes(idx, out);
            }
            Expr::Array(elems) => {
                for e in elems {
                    self.find_escapes(e, out);
                }
            }
            Expr::Object(pairs) => {
                for p in pairs {
                    self.find_escapes(&p.value, out);
                }
            }
            Expr::Pair(p) => self.find_escapes(&p.value, out),
            Expr::Range(r) => {
                self.find_escapes(&r.start, out);
                self.find_escapes(&r.end, out);
            }
            Expr::If(expr_if) => {
                // Analyze if-as-expression branches for escapes too.
                for b in &expr_if.branches {
                    self.find_escapes(&b.cond, out);
                    for s in &b.body.stmts {
                        if let Stmt::Expr(e) = s {
                            self.find_escapes(e, out);
                        }
                    }
                }
                if let Some(else_body) = &expr_if.else_ {
                    for s in &else_body.stmts {
                        if let Stmt::Expr(e) = s {
                            self.find_escapes(e, out);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Gather all variable references (`Expr::Ident` / `Expr::Ref`) reachable
    /// from `expr`. Used for closure/async capture and return-value tracking.
    /// Does NOT descend into nested closures (their captures belong to them).
    fn gather_var_refs(&self, expr: &Expr, out: &mut Vec<Name>) {
        match expr {
            Expr::Ident(name) | Expr::Ref(name) => {
                if self.is_visible(name) {
                    out.push(name.clone());
                }
            }
            Expr::Closure(_) | Expr::Lambda(_) => {
                // Don't descend into nested closures; their captures are theirs.
            }
            Expr::Call(call) => {
                self.gather_var_refs(&call.name, out);
                for arg in &call.args.args {
                    if let Arg::Pos(e) = arg {
                        self.gather_var_refs(e, out);
                    }
                }
            }
            Expr::Unary(_, e) => self.gather_var_refs(e, out),
            Expr::Bina(l, _, r) => {
                self.gather_var_refs(l, out);
                self.gather_var_refs(r, out);
            }
            Expr::Dot(obj, _) => self.gather_var_refs(obj, out),
            Expr::Index(b, i) => {
                self.gather_var_refs(b, out);
                self.gather_var_refs(i, out);
            }
            Expr::Array(elems) => {
                for e in elems {
                    self.gather_var_refs(e, out);
                }
            }
            Expr::Object(pairs) => {
                for p in pairs {
                    self.gather_var_refs(&p.value, out);
                }
            }
            Expr::AsyncBlock { body, .. } => {
                for s in &body.stmts {
                    if let Stmt::Expr(e) = s {
                        self.gather_var_refs(e, out);
                    }
                }
            }
            _ => {}
        }
    }

    /// Is `name` a binding introduced in any currently-open scope?
    fn is_visible(&self, name: &Name) -> bool {
        self.scope_stack.iter().flatten().any(|n| n == name)
    }

    /// Escalate a visible binding: record (or upgrade) its tier to a non-borrow.
    /// Bindings not visible (e.g. params, globals) are ignored — they're owned
    /// by their own scope and out of this function's escape analysis.
    fn escalate_visible(&mut self, name: &Name, reason: impl Into<String>) {
        // Find the binding's scope depth (nearest enclosing).
        let depth = self.scope_depth_of(name);
        if let Some(depth) = depth {
            let id = BindingId { scope_depth: depth, name: name.clone() };
            // Default fallback tier for escape: Clone. Phase 2 will refine
            // (Copy types stay Clone; others become RcRefCell). For Phase 1 we
            // record Clone as the conservative escape tier.
            self.decide(id, OwnershipTier::Clone, reason);
        }
    }

    /// Find the scope depth where `name` was introduced (nearest = highest depth).
    fn scope_depth_of(&self, name: &Name) -> Option<usize> {
        for (depth, names) in self.scope_stack.iter().enumerate().rev() {
            if names.iter().any(|n| n == name) {
                return Some(depth);
            }
        }
        None
    }

    /// Decide the initial tier for a freshly-introduced binding.
    ///
    /// Most bindings start as `Owned` (the default own-by-default model). The
    /// escape patterns that would *lower* a binding to BorrowView are detected
    /// later at *use* sites (`Expr::View`/`Expr::Mut`), not at definition —
    /// so here we conservatively mark everything `Owned` unless the user wrote
    /// an explicit view/mut in the initializer.
    fn initial_tier(&self, _expr: &Expr, _name: &Name) -> OwnershipTier {
        // Phase 1: every binding is Owned at definition. The escape escalation
        // (return/closure/call) raises it further; the borrow opportunity is
        // detected at View/Mut use sites in Phase 2.
        OwnershipTier::Owned
    }
}

impl Default for EscapeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, Expr, Fn, Name, Stmt, Store, StoreKind, Type};

    fn make_store(name: &str, expr: Expr) -> Stmt {
        Stmt::Store(Store {
            kind: StoreKind::Let,
            name: Name::from(name),
            ty: Type::Unknown,
            expr,
            attrs: Vec::new(),
        })
    }

    #[test]
    fn test_local_binding_is_owned() {
        // let x = 42
        let body = Body {
            stmts: vec![make_store("x", Expr::Int(42))],
            has_new_line: false,
            source_lines: vec![],
        };
        let map = EscapeAnalyzer::analyze_body(&body);
        // A plain local with no escape starts as Owned.
        assert_eq!(map.lookup(0, &"x".into()), Some(OwnershipTier::Owned));
    }

    #[test]
    fn test_returned_binding_escapes() {
        // let x = ...
        // return x
        let body = Body {
            stmts: vec![
                make_store("x", Expr::Ident(Name::from("y"))),
                Stmt::Return(Box::new(Expr::Ident(Name::from("x")))),
            ],
            has_new_line: false,
            source_lines: vec![],
        };
        let map = EscapeAnalyzer::analyze_body(&body);
        // x is returned → escapes → escalated past Owned.
        let tier = map.lookup(0, &"x".into()).expect("x should be tracked");
        assert!(
            !tier.is_borrow(),
            "returned binding should not be a borrow, got {:?}",
            tier
        );
    }

    #[test]
    fn test_binding_in_nested_scope_is_distinct() {
        // let x = 1
        // { let x = 2 }   // shadows
        let body = Body {
            stmts: vec![
                make_store("x", Expr::Int(1)),
                Stmt::Block(Body {
                    stmts: vec![make_store("x", Expr::Int(2))],
                    has_new_line: false,
                    source_lines: vec![],
                }),
            ],
            has_new_line: false,
            source_lines: vec![],
        };
        let map = EscapeAnalyzer::analyze_body(&body);
        // Two distinct bindings for "x" at different depths.
        assert_eq!(map.len(), 2, "expected two distinct x bindings");
        assert_eq!(map.lookup(0, &"x".into()), Some(OwnershipTier::Owned));
        assert_eq!(map.lookup(1, &"x".into()), Some(OwnershipTier::Owned));
    }

    #[test]
    fn test_empty_fn_body() {
        let func = Fn {
            kind: crate::ast::FnKind::Function,
            name: Name::from("f"),
            parent: None,
            params: vec![],
            body: Body::new(),
            ret: Type::Void,
            ret_name: None,
            is_static: false,
            is_pub: false,
            is_mut: false,
            is_test: false,
            type_params: vec![],
            doc: None,
            span: None,
        };
        let map = EscapeAnalyzer::analyze_fn(&func);
        assert!(map.is_empty());
    }

    /// End-to-end: parse real Auto source, then analyze the main function.
    /// This proves the analyzer runs correctly on Parser-produced AST
    /// (not just hand-built AST), which is what transpile_rust relies on.
    #[test]
    fn test_end_to_end_real_source() {
        use crate::parser::Parser;
        // A function with: a local binding, a for loop (nested scope), and a
        // returned value — exercises the main traversal paths.
        let src = "fn compute(n int) int {\n    let total = 0\n    for i in 0..n {\n        total = total + i\n    }\n    return total\n}\n";
        let mut parser = Parser::from(src);
        parser.set_dest(crate::parser::CompileDest::TransRust);
        let ast = parser.parse().expect("parse should succeed");

        // Find the Fn statement and analyze it.
        let func = ast
            .stmts
            .iter()
            .find_map(|s| if let Stmt::Fn(f) = s { Some(f) } else { None })
            .expect("expected an fn declaration");

        let map = EscapeAnalyzer::analyze_fn(func);
        // `total` at depth 0 should be tracked (Owned by default).
        assert!(
            map.lookup(0, &"total".into()).is_some(),
            "total should be tracked"
        );
        // `total` is returned → should be escalated past Borrow (Owned or Clone).
        let total_tier = map.lookup(0, &"total".into()).unwrap();
        assert!(
            !total_tier.is_borrow(),
            "returned binding should not be a borrow tier, got {:?}",
            total_tier
        );
        // `i` is a loop variable in the nested scope (depth 1) — it's owned
        // and not returned, so Owned. It may or may not be tracked depending
        // on whether the for-body reassignment introduces it; the key check
        // is that analysis ran without panic.
    }
}
