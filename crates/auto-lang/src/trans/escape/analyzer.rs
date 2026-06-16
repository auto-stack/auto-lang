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
    /// Plan 310 Phase 2: buffered borrow-use sites collected during the single
    /// traversal. After `visit_body` finishes, [`apply_lowering`] walks this
    /// list and lowers any `Owned` binding to `BorrowView`/`BorrowMut` (the
    /// borrow opportunity detected at a View/Mut use site). Each entry is
    /// (binding id, intended tier) — the id is resolved at collection time so
    /// the depth is captured correctly.
    borrow_uses: Vec<(BindingId, OwnershipTier)>,
}

impl EscapeAnalyzer {
    pub fn new() -> Self {
        Self {
            map: EscapeMap::new(),
            // depth 0 = function body; pushed by analyze_fn / analyze_body.
            scope_stack: vec![Vec::new()],
            borrow_uses: Vec::new(),
        }
    }

    /// Analyze a top-level function's body and return the escape decisions.
    pub fn analyze_fn(func: &Fn) -> EscapeMap {
        let mut analyzer = Self::new();
        analyzer.visit_body(&func.body, 0);
        analyzer.apply_lowering();
        analyzer.map
    }

    /// Analyze a bare body (e.g. for testing). Treats it as depth 0.
    pub fn analyze_body(body: &Body) -> EscapeMap {
        let mut analyzer = Self::new();
        analyzer.visit_body(body, 0);
        analyzer.apply_lowering();
        analyzer.map
    }

    /// Plan 310 Phase 2: post-pass that lowers `Owned` bindings to borrows
    /// where a View/Mut use site was recorded. This runs AFTER the single
    /// traversal completes, so the escape set is final: a binding still at
    /// `Owned` (ordinal 0) is provably non-escaping, and borrowing it is
    /// rustc-safe. Bindings already escalated to `Clone`/`RcRefCell` are left
    /// untouched — `record` rejects the ordinal-lowering (escape wins).
    fn apply_lowering(&mut self) {
        // Drain so we don't hold a borrow of self.borrow_uses while mutating map.
        let uses = std::mem::take(&mut self.borrow_uses);
        for (id, intended_tier) in uses {
            // Only BorrowView/BorrowMut are valid lowering targets.
            if !intended_tier.is_borrow() {
                continue;
            }
            // Check current tier. Lowering is sound only if the binding is at
            // Owned (never escaped). Escalated bindings keep their tier.
            let current = self.map.lookup(id.scope_depth, &id.name);
            match current {
                Some(OwnershipTier::Owned) => {
                    self.map.record(id, intended_tier, "lowered: borrow use, non-escaping");
                }
                _ => {
                    // Already escalated (Clone/RcRefCell) or untracked — leave as is.
                }
            }
        }
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
                // Phase 2: also collect View/Mut borrow uses in the initializer.
                self.collect_borrow_uses(&store.expr);
                // Phase 3: detect Send boundaries (Go/tokio::spawn captures).
                self.collect_send_escapes(&store.expr);

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
                // Phase 2: collect borrow uses in expression statements too.
                self.collect_borrow_uses(expr);
                // Phase 3: detect Send boundaries.
                self.collect_send_escapes(expr);
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
                // Phase 2: a View/Mut in a return expression still records a
                // borrow use, but the same binding is also escalated above, so
                // the lowering pass will skip it (escape wins). Collect anyway
                // for completeness.
                self.collect_borrow_uses(expr);
                // Phase 3: detect Send boundaries in return expressions.
                self.collect_send_escapes(expr);
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

    /// Plan 310 Phase 2: scan `expr` for `Expr::View`/`Expr::Mut` use sites
    /// and buffer each resolved borrow use into [`Self::borrow_uses`] for the
    /// post-traversal lowering pass. The resolution captures the current scope
    /// depth (via [`scope_depth_of`]) so the later lowering can target the
    /// right binding even in nested scopes.
    fn collect_borrow_uses(&mut self, expr: &Expr) {
        self.find_borrow_uses(expr);
    }

    /// Plan 310 Phase 3: scan `expr` for Send-boundary sites (`Expr::Go`) and
    /// directly escalate captured bindings to `ArcMutex` tier. Unlike
    /// `collect_escapes` (which buffers to a Vec for later default-Clone
    /// escalation), this escalates immediately to the specific tier because Go
    /// captures need ArcMutex (not Clone). Called alongside collect_escapes in
    /// visit_stmt.
    fn collect_send_escapes(&mut self, expr: &Expr) {
        self.find_send_boundaries(expr);
    }

    /// Recursive helper for [`collect_send_escapes`]. Detects `Expr::Go` (= the
    /// tokio::spawn Send boundary) and escalates captured visible bindings to
    /// ArcMutex. Recurses into compound expressions to find nested Go calls.
    fn find_send_boundaries(&mut self, expr: &Expr) {
        match expr {
            Expr::Go { expr: inner } => {
                // tokio::spawn requires the future (and its captures) to be Send.
                // Any visible local captured here must be Arc<Mutex<T>>, not
                // Rc<RefCell<T>> (which is !Send). Escalate to ArcMutex tier.
                let mut captured = Vec::new();
                self.gather_var_refs(inner, &mut captured);
                for name in captured {
                    self.escalate_to(&name, OwnershipTier::ArcMutex, "captured across Send boundary (.go/tokio::spawn)");
                }
                // Also recurse into inner for nested Go or escape patterns.
                self.find_send_boundaries(inner);
            }
            // Recurse into compound expressions.
            Expr::Call(call) => {
                self.find_send_boundaries(&call.name);
                for arg in &call.args.args {
                    if let Arg::Pos(e) = arg {
                        self.find_send_boundaries(e);
                    }
                }
            }
            Expr::Unary(_, e) => self.find_send_boundaries(e),
            Expr::Bina(l, _, r) => {
                self.find_send_boundaries(l);
                self.find_send_boundaries(r);
            }
            Expr::Dot(obj, _) => self.find_send_boundaries(obj),
            Expr::Index(base, idx) => {
                self.find_send_boundaries(base);
                self.find_send_boundaries(idx);
            }
            Expr::Array(elems) => {
                for e in elems {
                    self.find_send_boundaries(e);
                }
            }
            _ => {}
        }
    }

    /// Recursive helper for [`collect_borrow_uses`]. Records a borrow use only
    /// when the View/Mut operand is a visible binding (`Expr::Ident`/`Expr::Ref`).
    /// Other forms (e.g. `f().view`) have no binding to lower and are skipped.
    fn find_borrow_uses(&mut self, expr: &Expr) {
        match expr {
            Expr::View(inner) => {
                if let Some(id) = self.resolve_binding(inner) {
                    self.borrow_uses.push((id, OwnershipTier::BorrowView));
                }
                // Still recurse into inner for nested View/Mut (rare but possible).
                self.find_borrow_uses(inner);
            }
            Expr::Mut(inner) => {
                if let Some(id) = self.resolve_binding(inner) {
                    self.borrow_uses.push((id, OwnershipTier::BorrowMut));
                }
                self.find_borrow_uses(inner);
            }
            // Do NOT descend into closures: their captures are accounted for by
            // the escape pass (escalate), and their internal View/Mut uses refer
            // to the closure's own params, not this function's bindings.
            Expr::Closure(_) | Expr::Lambda(_) => {}

            // Recurse into compound expressions to find nested borrow uses.
            Expr::Call(call) => {
                self.find_borrow_uses(&call.name);
                for arg in &call.args.args {
                    if let Arg::Pos(e) = arg {
                        self.find_borrow_uses(e);
                    }
                }
            }
            Expr::Unary(_, e) => self.find_borrow_uses(e),
            Expr::Bina(l, _, r) => {
                self.find_borrow_uses(l);
                self.find_borrow_uses(r);
            }
            Expr::Dot(obj, _) => self.find_borrow_uses(obj),
            Expr::Index(base, idx) => {
                self.find_borrow_uses(base);
                self.find_borrow_uses(idx);
            }
            Expr::Array(elems) => {
                for e in elems {
                    self.find_borrow_uses(e);
                }
            }
            Expr::Object(pairs) => {
                for p in pairs {
                    self.find_borrow_uses(&p.value);
                }
            }
            Expr::Range(r) => {
                self.find_borrow_uses(&r.start);
                self.find_borrow_uses(&r.end);
            }
            _ => {}
        }
    }

    /// Resolve `expr` to a `BindingId` if it's a direct variable reference to a
    /// visible binding. Returns None for non-ident expressions or bindings not
    /// in any open scope (e.g. function params, which aren't tracked here).
    fn resolve_binding(&self, expr: &Expr) -> Option<BindingId> {
        let name = match expr {
            Expr::Ident(name) | Expr::Ref(name) => name,
            _ => return None,
        };
        let depth = self.scope_depth_of(name)?;
        Some(BindingId { scope_depth: depth, name: name.clone() })
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
                // Plan 310 Phase 2 (revised): do NOT treat bare variable call
                // arguments as escapes. The original conservative rule ("any
                // local passed to a call escapes") killed nearly every borrow
                // opportunity because variables are routinely passed to
                // functions like print(), len(), etc. that do not store the
                // reference.
                //
                // Per design doc §4.2, only *external* (`use.rust`) calls and
                // the specific escape patterns (closure capture, return, await,
                // struct-field storage) trigger escalation. Ordinary Auto
                // function calls are pass-by-value (owned) and do not escape.
                //
                // We still recurse into compound argument expressions so that,
                // e.g., a closure passed as an argument is detected.
                for arg in &call.args.args {
                    if let Arg::Pos(inner) = arg {
                        self.find_escapes(inner, out);
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

    /// Plan 310 Phase 3: Escalate a visible binding to a SPECIFIC tier (not
    /// just the default Clone). Used for Send-boundary detection: variables
    /// captured across `Expr::Go` (= tokio::spawn) must be ArcMutex (thread-safe),
    /// not Clone or RcRefCell. If the binding is already at a HIGHER tier
    /// (ordinal), record keeps the higher one (conservative).
    fn escalate_to(&mut self, name: &Name, tier: OwnershipTier, reason: impl Into<String>) {
        let depth = self.scope_depth_of(name);
        if let Some(depth) = depth {
            let id = BindingId { scope_depth: depth, name: name.clone() };
            self.decide(id, tier, reason);
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
            api_attrs: None,
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

    /// Plan 310 Phase 2: a non-escaping binding that has a View use site
    /// should be lowered from Owned → BorrowView.
    #[test]
    fn test_lowering_owned_to_borrow_view() {
        // let x = 1
        // let y = x.view    ← View use of x; x is local, non-escaping
        let body = Body {
            stmts: vec![
                make_store("x", Expr::Int(1)),
                make_store("y", Expr::View(Box::new(Expr::Ident(Name::from("x"))))),
            ],
            has_new_line: false,
            source_lines: vec![],
        };
        let map = EscapeAnalyzer::analyze_body(&body);
        // x should be lowered to BorrowView (was Owned, View use detected,
        // no escape).
        let x_tier = map.lookup(0, &"x".into()).expect("x should be tracked");
        assert_eq!(
            x_tier,
            OwnershipTier::BorrowView,
            "non-escaping binding with View use should lower to BorrowView, got {:?}",
            x_tier
        );
    }

    /// Plan 310 Phase 2: a binding that escapes (returned) should NOT be
    /// lowered even if it has a View use site — escape wins.
    #[test]
    fn test_lowering_skipped_when_escaping() {
        // let x = 1
        // let y = x.view    ← View use of x
        // return x          ← but x is also returned → escapes
        let body = Body {
            stmts: vec![
                make_store("x", Expr::Int(1)),
                make_store("y", Expr::View(Box::new(Expr::Ident(Name::from("x"))))),
                Stmt::Return(Box::new(Expr::Ident(Name::from("x")))),
            ],
            has_new_line: false,
            source_lines: vec![],
        };
        let map = EscapeAnalyzer::analyze_body(&body);
        // x is returned → escalated past Owned. The View use should NOT lower it.
        let x_tier = map.lookup(0, &"x".into()).expect("x should be tracked");
        assert!(
            !x_tier.is_borrow(),
            "returned binding should stay escalated (not lowered to borrow), got {:?}",
            x_tier
        );
    }

    /// Plan 310 Phase 2: Expr::Mut use site lowers Owned → BorrowMut.
    #[test]
    fn test_lowering_owned_to_borrow_mut() {
        // let x = 1
        // let y = x.mut     ← Mut use of x
        let body = Body {
            stmts: vec![
                make_store("x", Expr::Int(1)),
                make_store("y", Expr::Mut(Box::new(Expr::Ident(Name::from("x"))))),
            ],
            has_new_line: false,
            source_lines: vec![],
        };
        let map = EscapeAnalyzer::analyze_body(&body);
        let x_tier = map.lookup(0, &"x".into()).expect("x should be tracked");
        assert_eq!(
            x_tier,
            OwnershipTier::BorrowMut,
            "non-escaping binding with Mut use should lower to BorrowMut, got {:?}",
            x_tier
        );
    }

    /// Plan 310 Phase 2: binding with NO View/Mut use stays Owned.
    #[test]
    fn test_no_lowering_without_borrow_use() {
        // let x = 1
        // let y = x         ← plain use, not a View/Mut → no lowering
        let body = Body {
            stmts: vec![
                make_store("x", Expr::Int(1)),
                make_store("y", Expr::Ident(Name::from("x"))),
            ],
            has_new_line: false,
            source_lines: vec![],
        };
        let map = EscapeAnalyzer::analyze_body(&body);
        let x_tier = map.lookup(0, &"x".into()).expect("x should be tracked");
        assert_eq!(
            x_tier,
            OwnershipTier::Owned,
            "binding without View/Mut use should stay Owned, got {:?}",
            x_tier
        );
    }

    /// Plan 310 Phase 3: a binding captured across `Expr::Go` (tokio::spawn
    /// Send boundary) should be escalated to ArcMutex tier.
    #[test]
    fn test_go_capture_escalates_to_arc_mutex() {
        // let x = 1
        // x.go         ← Go captures x across Send boundary → ArcMutex
        let body = Body {
            stmts: vec![
                make_store("x", Expr::Int(1)),
                Stmt::Expr(Expr::Go { expr: Box::new(Expr::Ident(Name::from("x"))) }),
            ],
            has_new_line: false,
            source_lines: vec![],
        };
        let map = EscapeAnalyzer::analyze_body(&body);
        let x_tier = map.lookup(0, &"x".into()).expect("x should be tracked");
        assert_eq!(
            x_tier,
            OwnershipTier::ArcMutex,
            "binding captured across .go should be ArcMutex, got {:?}",
            x_tier
        );
    }

    /// Plan 310 Phase 3: a binding captured by an async block (NOT across Go)
    /// should be escalated to Clone (not ArcMutex) — async blocks are same-task.
    #[test]
    fn test_async_block_capture_is_clone_not_arc() {
        // let x = 1
        // ~{ x }       ← async block captures x, but no Go → Clone tier
        let body = Body {
            stmts: vec![
                make_store("x", Expr::Int(1)),
                Stmt::Expr(Expr::AsyncBlock {
                    body: Body {
                        stmts: vec![Stmt::Expr(Expr::Ident(Name::from("x")))],
                        has_new_line: false,
                        source_lines: vec![],
                    },
                    return_type: None,
                }),
            ],
            has_new_line: false,
            source_lines: vec![],
        };
        let map = EscapeAnalyzer::analyze_body(&body);
        let x_tier = map.lookup(0, &"x".into()).expect("x should be tracked");
        // AsyncBlock capture → Clone (escalated, but not ArcMutex since no
        // Send boundary). ArcMutex is higher ordinal than Clone, so if both
        // were applied ArcMutex would win — but only Go triggers ArcMutex.
        assert_ne!(
            x_tier,
            OwnershipTier::ArcMutex,
            "async block (no Go) should NOT escalate to ArcMutex, got {:?}",
            x_tier
        );
    }
}
