//! PLAN-998 (`legacy-a2r.resources-observe/1`): emission hooks that make the
//! REAL ownership paths of the emitted Rust observable. The profile is
//! opt-in and exact-id admitted; with `ResourceProfile::None` none of these
//! hooks fire and the frozen default bytes are untouched.
//!
//! Hook map (frozen with the profile id — see `res_obs_helper_block`):
//! - `Stmt::Store` (let/var of an owned type) → guard binding
//!   `let __a2r_o<var> = __a2r_own("<fn>:<var>");` — the guard's Drop is the
//!   REAL drop observation (per-frame LIFO is Rust's own drop order).
//! - enum-with-owned-payload reassignment → `__a2r_replace(&mut r, owner#k, v)`
//!   — the old payload's drop is observed at the actual replacement point
//!   (owner generations `#1..#k`).
//! - `Expr::Move(ident)` with a live guard → `__a2r_xfer(guard, v)` — a
//!   transfer DISARMS the source guard (a move is NOT a release).
//! - `Expr::Str`/`CStr` literals → `{ __a2r_chk(); "lit" }` — chk is the
//!   fault sentinel: it fails (typed, unwinding) right before the next heap
//!   allocation when `A2R_FAULT=heap#k` matches, otherwise it is a no-op.
//! - str concat / FStr `format!` and `.clone()` sites → same chk sentinel
//!   (these are the allocating preparation constructs).
//! - structural list mutation statements (push/insert/remove/set on a Vec
//!   binding) → `__a2r_list_pre` (chk + generation bump + `mut` event) so
//!   views minted before a structural mutation go stale.
//! - Vec index reads → `__a2r_read` event (the buffer_reads counter face).
//! - loop heads → `__a2r_step()` (budget/cancel bite in real spins).
//! - move-closures capturing owned bindings → `__a2r_envgo([...])` (the
//!   captures TRANSFER into the closure env, not release).

use super::*;

impl RustTrans {
    /// Whether the resource-observation profile is active.
    pub(super) fn res_on(&self) -> bool {
        self.resource_profile == ResourceProfile::Observe
    }

    /// Mark the helper block as needed (call at every hook site).
    pub(super) fn res_mark(&mut self) {
        if !self.res_obs_used {
            self.res_obs_used = true;
        }
    }

    /// Owned-ness of a binding for guard purposes: does the emitted Rust
    /// binding own heap-backed resources (String buffer, Vec buffer, boxed
    /// closure env, struct/enum with owned payloads)? Conservative at the
    /// recursion cap (treat as owned — an extra guard never lies about a
    /// release; a missing one would).
    pub(super) fn res_owned_type(&self, ty: &Type, depth: usize) -> bool {
        if depth > 8 {
            return true;
        }
        match ty {
            Type::StrOwned | Type::StrFixed(_) => true,
            Type::List(_) => true,
            Type::Fn(..) => true,
            Type::Option(inner) | Type::Result(inner) => self.res_owned_type(inner, depth + 1),
            Type::User(td) => td
                .members
                .iter()
                .any(|m| self.res_owned_type(&m.ty, depth + 1)),
            Type::Enum(e) => e.borrow().items.iter().any(|it| {
                it.payload_type
                    .as_ref()
                    .map(|p| self.res_owned_type(p, depth + 1))
                    .unwrap_or(false)
                    || it
                        .payload_types
                        .iter()
                        .any(|p| self.res_owned_type(p, depth + 1))
            }),
            Type::GenericInstance(inst) => {
                inst.args.iter().any(|a| self.res_owned_type(a, depth + 1))
            }
            _ => false,
        }
    }

    /// Enum-with-owned-payload binding (reassignment models payload
    /// generations: `fn:var#k` — the old payload's drop is OBSERVED at the
    /// replacement point).
    pub(super) fn res_enum_owned(&self, ty: &Type) -> bool {
        match ty {
            Type::Result(_) => true,
            Type::Enum(e) => e.borrow().items.iter().any(|it| {
                it.payload_type
                    .as_ref()
                    .map(|p| self.res_owned_type(p, 0))
                    .unwrap_or(false)
                    || it.payload_types.iter().any(|p| self.res_owned_type(p, 0))
            }),
            Type::GenericInstance(inst) => {
                // `Result<T, Str>`-shaped instances: any owned arg makes the
                // slot payload-owning.
                inst.args.iter().any(|a| self.res_owned_type(a, 1))
            }
            _ => false,
        }
    }

    /// Frame label for owner strings (`main:xs`, `relay:x`).
    pub(super) fn res_owner(&self, var: &str) -> String {
        format!("{}:{}", self.current_fn_name, var)
    }

    /// Deterministic guard ident for a binding (`__a2r_o<var>`).
    pub(super) fn res_guard_ident(var: &str) -> AutoStr {
        format!("__a2r_o{}", var).into()
    }

    /// Note a guarded binding in the innermost scope.
    pub(super) fn res_note_guard(&mut self, var: &str, owner: String) {
        let ident = Self::res_guard_ident(var);
        if let Some(scope) = self.res_guard_scopes.last_mut() {
            scope.insert(var.into(), (ident, owner));
        }
    }

    /// Innermost-first guard lookup for `.move` sites.
    pub(super) fn res_guard_for(&self, var: &str) -> Option<(AutoStr, String)> {
        for scope in self.res_guard_scopes.iter().rev() {
            if let Some((ident, owner)) = scope.get(var) {
                return Some((ident.clone(), owner.clone()));
            }
        }
        None
    }

    /// Push/pop a lexical scope for guard visibility (blocks, if/for/is
    /// bodies, fn frames).
    pub(super) fn res_push_scope(&mut self) {
        self.res_guard_scopes
            .push(std::collections::BTreeMap::new());
    }

    pub(super) fn res_pop_scope(&mut self) {
        self.res_guard_scopes.pop();
    }

    /// Enter a function frame: fresh scope + fresh enum generation counters.
    pub(super) fn res_frame_enter(&mut self) {
        self.res_push_scope();
    }

    pub(super) fn res_frame_exit(&mut self) {
        self.res_pop_scope();
    }

    /// Next owner label for an enum-with-owned binding, bumping the
    /// generation counter (`main:r` → `main:r#1`, then `main:r#2`, ...).
    pub(super) fn res_enum_gen_owner(&mut self, var: &str) -> String {
        let base = self.res_owner(var);
        let n = self.res_enum_gens.entry(var.into()).or_insert(0);
        *n += 1;
        format!("{}#{}", base, n)
    }

    /// Emit the guard binding line after a store (caller adds indent/newline
    /// context). Writes `let __a2r_o<var> = __a2r_own("<owner>");`.
    pub(super) fn res_write_guard(
        &mut self,
        var: &str,
        owner: &str,
        out: &mut impl Write,
    ) -> std::io::Result<()> {
        self.res_mark();
        write!(
            out,
            "let {} = __a2r_own(\"{}\");",
            Self::res_guard_ident(var),
            owner
        )
    }

    /// Does this store bind an owned value (guard-worthy)? Declared type is
    /// authoritative; an Unknown type falls back to the init shape (closure
    /// literal, owned ident move, call returning an owned fn type).
    pub(super) fn res_store_needs_guard(&self, store: &Store) -> bool {
        if !matches!(
            store.kind,
            crate::ast::StoreKind::Let | crate::ast::StoreKind::Var
        ) {
            return false;
        }
        if !matches!(store.ty, Type::Unknown) {
            return self.res_owned_type(&store.ty, 0);
        }
        // The store() emission registers the binding's inferred type before
        // this hook runs — a List-typed `var xs = List<T>.new()` resolves
        // here.
        if let Some(t) = self.local_var_types.get(store.name.as_str()) {
            if !matches!(t, Type::Unknown) {
                return self.res_owned_type(t, 0);
            }
        }
        match &store.expr {
            Expr::Closure(_) => true,
            Expr::Ident(name) => self
                .local_var_types
                .get(name)
                .map(|t| self.res_owned_type(t, 0))
                .unwrap_or(false),
            Expr::Call(c) => match c.name.as_ref() {
                Expr::Ident(f) => self
                    .fn_ret_types
                    .get(f)
                    .map(|t| self.res_owned_type(t, 0))
                    .unwrap_or(false),
                _ => false,
            },
            _ => false,
        }
    }

    /// The shared post-store hook: guard line + scope note (used by both
    /// `stmt()`'s Store arm and `body()`'s own Store path).
    pub(super) fn res_after_store(&mut self, store: &Store, out: &mut impl Write) {
        if !self.res_store_needs_guard(store) {
            return;
        }
        let is_enum = self
            .local_var_types
            .get(store.name.as_str())
            .map(|t| self.res_enum_owned(t))
            .unwrap_or(false)
            || (!matches!(store.ty, Type::Unknown) && self.res_enum_owned(&store.ty));
        let owner = if is_enum {
            self.res_enum_gen_owner(store.name.as_str())
        } else {
            self.res_owner(store.name.as_str())
        };
        let _ = write!(out, "\n");
        let _ = self.print_indent(out);
        let _ = self.res_write_guard(store.name.as_str(), &owner, out);
        let _ = write!(out, ";");
        self.res_note_guard(store.name.as_str(), owner);
    }

    /// Statement-position pre-hooks for the observe profile:
    /// - structural list mutation statements (`xs.push(v)` etc. on a Vec
    ///   binding) → `__a2r_list_pre("<fn>:<var>");` line (chk sentinel +
    ///   generation bump + `mut` event);
    /// - assignment to an enum-with-owned-payload binding → the payload
    ///   replace (`__a2r_replace_go`) models generations `#k` so the old
    ///   payload's drop is OBSERVED at the replacement point. The original
    ///   expression is suppressed in this case (the hook emits the full
    ///   replacement statement); callers detect suppression via
    ///   [`Self::res_suppressed_last`].
    pub(super) fn res_expr_stmt_pre(
        &mut self,
        expr: &Expr,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        // Structural list mutation: `<vec-ident>.<method>(...)` statement.
        if let Expr::Call(call) = expr {
            if let Expr::Dot(base, method) = call.name.as_ref() {
                let m = method.as_str();
                if matches!(m, "push" | "insert" | "remove" | "set" | "pop" | "clear") {
                    if let Expr::Ident(name) = base.as_ref() {
                        if self.res_is_vec_binding(name) {
                            // Emit the full wrapped statement (the original
                            // emission is suppressed): the list events fire
                            // before, and __a2r_mut_call runs the growth
                            // fault check AFTER the argument allocations —
                            // immediately before the mutation's allocation.
                            self.res_mark();
                            self.res_suppressed_last.set(true);
                            let owner = self.res_owner(name.as_str());
                            let var = name.as_str();
                            let _ = self.print_indent(out);
                            // Arguments evaluate FIRST into dedicated
                            // bindings — their allocations happen before the
                            // growth-imminent fault check inside
                            // __a2r_mut_call. (Boundary: an argument that
                            // borrows the receiver itself is outside this
                            // statement-wrap shape.)
                            let mut arg_names: Vec<String> = Vec::new();
                            for (ai, arg) in call.args.args.iter().enumerate() {
                                if let crate::ast::Arg::Pos(e) = arg {
                                    let an = format!("__a2r_a{ai}_{var}");
                                    let _ = write!(out, "let {an} = ");
                                    self.expr(e, out)?;
                                    // A str-literal argument into a
                                    // String-element list keeps its original
                                    // coercion (the plain call emission
                                    // would have added .to_string()).
                                    let elem_str = matches!(
                                        self.local_var_types.get(name),
                                        Some(Type::List(elem))
                                            if matches!(
                                                **elem,
                                                Type::StrFixed(_)
                                                    | Type::StrSlice
                                                    | Type::StrOwned
                                                    | Type::CStrLit
                                            )
                                    );
                                    if elem_str && matches!(e, Expr::Str(_) | Expr::CStr(_)) {
                                        let _ = write!(out, ".to_string()");
                                    }
                                    let _ = write!(out, "; ");
                                    arg_names.push(an);
                                }
                            }
                            let _ = write!(out, "__a2r_list_pre(\"{}\"); ", owner);
                            let _ =
                                write!(out, "__a2r_mut_call(&mut {var}, |__a2r_v| __a2r_v.{m}(");
                            for (ai, an) in arg_names.iter().enumerate() {
                                if ai > 0 {
                                    let _ = write!(out, ", ");
                                }
                                let _ = write!(out, "{an}");
                            }
                            let _ = writeln!(out, "));");
                        }
                    }
                }
            }
        }
        // Assignment `x = v` where x is an enum-with-owned-payload guarded
        // binding → replace_go statement + re-shadowed guard.
        if let Expr::Bina(lhs, op, rhs) = expr {
            if matches!(op, Op::Asn) {
                if let Expr::Ident(name) = lhs.as_ref() {
                    let owned_enum = self
                        .local_var_types
                        .get(name)
                        .map(|t| self.res_enum_owned(t))
                        .unwrap_or(false);
                    if owned_enum && self.res_guard_for(name.as_str()).is_some() {
                        self.res_mark();
                        let gen_owner = self.res_enum_gen_owner(name.as_str());
                        let (guard_ident, _owner) =
                            self.res_guard_for(name.as_str()).unwrap_or_default();
                        self.res_suppressed_last.set(true);
                        let _ = self.print_indent(out);
                        // The full replacement statement replaces the
                        // original assignment emission entirely.
                        let mut rhs_buf = std::io::Cursor::new(Vec::new());
                        self.expr(rhs, &mut rhs_buf)?;
                        let rhs_text = String::from_utf8(rhs_buf.into_inner())
                            .map_err(|e| format!("non-UTF-8 rhs: {e}"))?;
                        let _ = writeln!(
                            out,
                            "let {guard_ident} = __a2r_replace_go({guard_ident}, &mut {var}, \"{gen_owner}\", {rhs_text});",
                            var = name.as_str(),
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn res_is_vec_binding(&self, name: &AutoStr) -> bool {
        self.local_var_types
            .get(name)
            .map(|t| matches!(t, Type::List(_)))
            .unwrap_or(false)
    }
}
