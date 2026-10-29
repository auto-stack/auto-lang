//! PLAN-843 结构拆分（自 rust.rs 原样搬移；算法/行为零变化）。
use super::*;

impl RustTrans {
    pub(super) fn scan_stmt_err_enum(&self, stmt: &Stmt) -> Option<Option<AutoStr>> {
        match stmt {
            Stmt::Expr(expr) => self.scan_expr_err_enum(expr),
            Stmt::Return(expr) => self.scan_expr_err_enum(expr),
            Stmt::If(if_) => {
                for branch in &if_.branches {
                    for s in &branch.body.stmts {
                        if let Some(r) = self.scan_stmt_err_enum(s) {
                            return Some(r);
                        }
                    }
                }
                if let Some(else_body) = &if_.else_ {
                    for s in &else_body.stmts {
                        if let Some(r) = self.scan_stmt_err_enum(s) {
                            return Some(r);
                        }
                    }
                }
                None
            }
            Stmt::Store(store) => self.scan_expr_err_enum(&store.expr),
            _ => None,
        }
    }

    fn scan_expr_err_enum(&self, expr: &Expr) -> Option<Option<AutoStr>> {
        match expr {
            Expr::Err(inner) => {
                match inner.as_ref() {
                    // EditError.Variant(args) — Call with Dot callee
                    Expr::Call(call) => {
                        if let Expr::Bina(lhs, op, _) = call.name.as_ref() {
                            if matches!(op, Op::Dot) {
                                if let Expr::Ident(type_name) = lhs.as_ref() {
                                    if self.tag_types.contains(type_name) {
                                        return Some(Some(type_name.clone()));
                                    }
                                }
                            }
                        }
                        if let Expr::Dot(obj, _) = call.name.as_ref() {
                            if let Expr::Ident(type_name) = obj.as_ref() {
                                if self.tag_types.contains(type_name) {
                                    return Some(Some(type_name.clone()));
                                }
                            }
                        }
                        Some(None)
                    }
                    // EditError.Variant (no args) — plain Dot expression
                    Expr::Dot(obj, _) => {
                        if let Expr::Ident(type_name) = obj.as_ref() {
                            if self.tag_types.contains(type_name) {
                                return Some(Some(type_name.clone()));
                            }
                        }
                        Some(None)
                    }
                    Expr::Str(_) | Expr::CStr(_) => Some(None),
                    _ => Some(None),
                }
            }
            _ => None,
        }
    }

    /// Recursively check if an expression tree contains string-typed elements
    pub(super) fn expr_contains_string(&self, e: &Expr) -> bool {
        match e {
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => true,
            // Plan 523 H4: str 非区间下标读发射为码点十进制 String 形态
            // (chars().nth(..).to_string())——`s[i] + s[j]` 的 + 须走
            // format! 拼接(参考口径 "6667"),裸 + 为 String+String E0308。
            Expr::Index(arr, idx)
                if !matches!(idx.as_ref(), Expr::Range(_)) && self.expr_is_string_like(arr) =>
            {
                true
            }
            Expr::Ident(name) => {
                if let Some(ty) = self.local_var_types.get(name) {
                    return matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice);
                }
                // Unknown type: conservatively assume string if the name
                // suggests string content (heuristic to catch let-bound vars)
                false
            }
            Expr::Call(c) => {
                if let Expr::Ident(name) = c.name.as_ref() {
                    matches!(name.as_str(),
                        "to_string" | "format" | "trim" | "replace"
                        | "to_lowercase" | "to_uppercase" | "read_to_string"
                        | "read_line" | "collect")
                    // Plan 016 Phase 4: a user fn whose DECLARED return is
                    // `str` renders as an owned String — `blockText(a) +
                    // blockText(b)` must format!-concat (E0308 otherwise).
                    || self.fn_ret_types.get(name.as_str())
                        .map(|ty| matches!(ty,
                            Type::StrOwned | Type::StrFixed(_) | Type::StrSlice
                            | Type::CStrLit))
                        .unwrap_or(false)
                } else if let Expr::Dot(obj, m) = c.name.as_ref() {
                    // Plan 381 (Layer 2): bridged `json.to_string(v)` /
                    // `json.get_str(v, k)` return owned String — treat as
                    // string-containing so the &str-param auto-borrow appends
                    // .as_str() (E0308 `&str` vs `String` otherwise).
                    let json_bridge_str = if let Expr::Ident(o) = obj.as_ref() {
                        o.as_str() == "json"
                            && matches!(m.as_str(), "to_string" | "get_str" | "as_string")
                    } else {
                        false
                    };
                    // Plan 016 Phase 4: method-call string producers
                    // (`x.to_string()`, `s.trim()`, ...) on ANY receiver
                    // shape — `list[i].to_string() + a.text` parses the lhs
                    // as Call(Dot(Index(..), to_string)), which the
                    // json-only check above missed (E0308 String + String).
                    json_bridge_str
                        || matches!(
                            m.as_str(),
                            "to_string"
                                | "trim"
                                | "trim_start"
                                | "trim_end"
                                | "replace"
                                | "to_lowercase"
                                | "to_uppercase"
                        )
                } else {
                    false
                }
            }
            Expr::Dot(_, method) => {
                matches!(method.as_str(),
                    "to_string" | "trim" | "replace" | "to_lowercase"
                    | "to_uppercase" | "display" | "format")
                // Plan 016 Phase 4: a struct-field read of a str field
                // (`a.text`) is an owned String — count it for string-concat
                // detection. The owner's type is often unavailable (chained
                // receivers), so accept the field name being a str field of
                // ANY known struct (mirrors the Plan 433 A1 call-arg scan).
                || self.struct_field_types.values()
                    .any(|fields| fields.iter()
                        .any(|(f, t)| f.as_str() == method.as_str()
                            && matches!(t, Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit)))
            }
            Expr::Bina(inner_lhs, _, inner_rhs) => {
                self.expr_contains_string(inner_lhs) || self.expr_contains_string(inner_rhs)
            }
            _ => false,
        }
    }

    /// Plan 381 (Layer 2): true when a `json.X(...)` call argument is a JSON
    /// *Value*-producing expression (json.get/get_at/parse/get_str/get_u64) or
    /// a variable previously assigned from one (tracked in json_value_vars).
    /// Used by the json as_int/as_string/as_bool dispatch to pick the
    /// `a2r_std::json::as_*(&Value)` variant instead of the `as_*_str(&str)`
    /// one (the latter broke `json.as_int(json.get(v, "k"))` → E0308
    /// Option<&str>).
    pub(super) fn json_arg_is_value(&self, arg: &Arg) -> bool {
        if let Arg::Pos(expr) = arg {
            match expr {
                Expr::Call(call) => {
                    if let Expr::Dot(obj, m) = call.name.as_ref() {
                        if let Expr::Ident(o) = obj.as_ref() {
                            if o.as_str() == "json" {
                                return matches!(
                                    m.as_str(),
                                    "get" | "get_at" | "parse" | "get_str" | "get_u64"
                                );
                            }
                        }
                    }
                    false
                }
                Expr::Ident(name) => {
                    // P738-R6-01：json 值生产绑定的 def-use 集（新）与历史
                    // 跟踪集并集——json 值实参正确分派 Value 面而非 str 变体。
                    self.json_value_vars.contains(name) || self.json_value_bindings.contains(name)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Plan 310 Phase 2: Query the escape-analysis tier for a binding visible
    /// at the current scope depth in the current function. Returns Owned when
    /// no analysis data is available (e.g. params, globals, or functions not
    /// in escape_results) — Owned is the safe conservative default.
    /// Plan 419 §4.5: 当前函数中该绑定是否为闭包写捕获(→ a2r 自动升级
    /// Rc<RefCell<T>>)。
    pub(super) fn is_write_captured(&self, name: &str) -> bool {
        if self.current_fn_name.is_empty() {
            return false;
        }
        self.escape_results
            .get(&self.current_fn_name)
            .map_or(false, |m| m.is_write_capture(&name.into()))
    }

    /// Plan 419 §4.5: 当前函数的全部写捕获绑定名。
    fn write_capture_names(&self) -> Vec<AutoStr> {
        if self.current_fn_name.is_empty() {
            return Vec::new();
        }
        self.escape_results
            .get(&self.current_fn_name)
            .map(|m| m.write_capture_names())
            .unwrap_or_default()
    }

    pub(super) fn current_escape_tier(&self, name: &str) -> crate::trans::escape::OwnershipTier {
        use crate::trans::escape::OwnershipTier;
        if self.current_fn_name.is_empty() {
            return OwnershipTier::Owned;
        }
        match self.escape_results.get(&self.current_fn_name) {
            Some(map) => map
                .lookup(self.current_scope_depth, &name.into())
                .unwrap_or(OwnershipTier::Owned),
            None => OwnershipTier::Owned,
        }
    }

    /// Plan 310 Phase 2: Emit a borrow expression (`x.view` / `x.mut`) according
    /// to the escape-analysis tier of the referenced binding.
    ///
    /// **Backward-compatibility contract**: when the binding is not at an escape
    /// tier (Clone/RcRefCell), this produces the SAME output as before Phase 2
    /// (`&x` / `&mut x`). Only escape tiers change the output — that's what
    /// guarantees existing .expected.rs files stay byte-identical unless the
    /// analyzer detected an escape.
    ///
    /// - `inner` is the operand of View/Mut (e.g. the `x` in `x.view`).
    /// - `is_mut` true for `Expr::Mut`, false for `Expr::View`.
    ///
    /// Tier mapping:
    ///   Clone    → `x.clone()`         + W0007 warning
    ///   RcRefCell→ `Rc::clone(&x)`     + W0007 warning
    ///   _ (Owned/BorrowView/BorrowMut/None) → `&x` / `&mut x` (unchanged)
    /// Plan 387 follow-up: true if `expr` resolves to a `TaskRef<T>` value — a
    /// direct binding of TaskRef type, or a field access whose field type is
    /// TaskRef. TaskRef is move-only (RAII sole owner): such values must never
    /// be cloned, so the escape-analysis auto-clone paths must skip them.
    pub(super) fn expr_is_taskref(&self, expr: &Expr) -> bool {
        use crate::ast::Expr as E;
        match expr {
            E::Ident(name) | E::Ref(name) => self
                .local_var_types
                .get(name.as_str())
                .map(|ty| matches!(ty, Type::GenericInstance(inst) if inst.base_name == "TaskRef"))
                .unwrap_or(false),
            E::Dot(obj, field) => {
                // Resolve the object's struct type via local var types, then
                // check whether the accessed field's type is TaskRef.
                let obj_ty = match obj.as_ref() {
                    E::Ident(n) => self.local_var_types.get(n.as_str()),
                    _ => None,
                };
                let type_name = match obj_ty {
                    Some(Type::User(td)) => td.name.as_str(),
                    Some(Type::GenericInstance(inst)) => inst.base_name.as_str(),
                    _ => return false,
                };
                self.struct_field_types
                    .get(type_name)
                    .map(|fields| {
                        fields.iter().any(|(fname, fty)| {
                            fname.as_str() == field.as_str()
                                && matches!(fty, Type::GenericInstance(inst) if inst.base_name == "TaskRef")
                        })
                    })
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    fn emit_borrow(&mut self, inner: &Expr, is_mut: bool, out: &mut impl Write) -> AutoResult<()> {
        use crate::trans::escape::OwnershipTier;
        // Resolve the binding name. Only direct variable references can be
        // tier-checked; anything else (e.g. `f().view`) falls back to default.
        let binding_name = match inner {
            Expr::Ident(name) | Expr::Ref(name) => Some(name.as_str()),
            _ => None,
        };

        // Plan 310 Phase 2: Copy types (int, bool, char, float, ...) never need
        // borrowing or cloning — they're passed by value in Rust regardless.
        // `x.view` on an i32 just yields `x` (a copy). This avoids generating
        // `&x` (type-mismatch on Copy) or `x.clone()` (Copy types have no clone).
        // NOTE: we use a strict primitive-only check here, NOT the broader
        // is_copy_type (which includes String/slices). String is NOT Copy in
        // Rust, so it must go through the normal borrow/clone path.
        let is_copy = binding_name
            .and_then(|n| self.local_var_types.get(n))
            .map(|ty| Self::is_primitive_copy(ty))
            .unwrap_or(false);
        if is_copy {
            self.expr(inner, out)?;
            return Ok(());
        }

        // Plan 383: 命名函数引用（如 axum `.route("/", handler)`）在 Rust 里是
        // `fn` 函数项/函数指针，自动实现 Copy。既不需要借用也不需要 clone ——
        // 直接输出裸 ident。对标 VM 路径的函数引用分支（codegen.rs:5079）。
        if let Expr::Ident(name) | Expr::Ref(name) = inner {
            if self.function_names.contains(name) {
                self.expr(inner, out)?;
                return Ok(());
            }
        }

        let tier = match binding_name {
            Some(name) => self.current_escape_tier(name),
            None => OwnershipTier::Owned,
        };

        // PLAN-667 (F-05): 已知不能保持语义的 fallback 改为明确 Auto 侧诊断
        // （不推给 rustc，也不再用 .clone() 伪装支持）。
        if let Some(name) = binding_name {
            if matches!(tier, OwnershipTier::ArcMutex) {
                return Err(crate::error::AutoError::Msg(format!(
                    "cannot use '.{}' on '{}': the value is captured across a Send boundary \
                     (.go/tokio::spawn) and requires Arc<Mutex<T>>, which a2r does not generate \
                     yet (Plan 667 boundary) — remove the capture or restructure the spawn",
                    if is_mut { "mut" } else { "view" },
                    name
                )));
            }
            if is_mut && !tier.is_borrow() && tier != OwnershipTier::Owned {
                return Err(crate::error::AutoError::Msg(format!(
                    "cannot take '.mut' of '{}': the value escapes its scope ({}) — \
                     mutating a clone/Rc copy would silently diverge from the VM, where \
                     shared .mut is rejected at runtime (auto.rc.assert_unique); \
                     restructure to keep the binding non-escaping or use a fresh local",
                    name,
                    self.escape_reason_or_default(name, &tier)
                )));
            }
        }

        match tier {
            OwnershipTier::Clone => {
                // Plan 387 §16: TaskRef<T> is a single-owner move type (not
                // Clone). Passing it to a function must MOVE it, not clone.
                // Detect by the value's type being a TaskRef generic instance
                // (direct binding or a TaskRef-typed struct field).
                if self.expr_is_taskref(inner) {
                    self.expr(inner, out)?;
                } else {
                    // Escape detected: clone instead of borrow.
                    self.expr(inner, out)?;
                    write!(out, ".clone()")?;
                    if let Some(name) = binding_name {
                        self.emit_escape_warning(name, tier, "value escapes its scope");
                    }
                }
            }
            OwnershipTier::RcRefCell => {
                // Escape detected: share via Rc instead of borrow.
                write!(out, "Rc::clone(&")?;
                self.expr(inner, out)?;
                write!(out, ")")?;
                if let Some(name) = binding_name {
                    self.emit_escape_warning(name, tier, "value escapes its scope");
                }
            }
            OwnershipTier::ArcMutex => {
                // Phase 3: Send boundary detected (.go/tokio::spawn capture).
                // The value needs Arc<Mutex<T>> for thread-safe sharing, but
                // Phase 3 only does detection + warning + clone fallback.
                // Full Arc declaration rewrite is deferred to Phase 4.
                self.expr(inner, out)?;
                write!(out, ".clone()")?;
                if let Some(name) = binding_name {
                    self.emit_escape_warning(
                        name,
                        tier,
                        "captured across Send boundary (.go); consider Arc<Mutex<T>> for thread-safe sharing",
                    );
                }
            }
            // Owned, BorrowView, BorrowMut, or unknown → unchanged behavior.
            _ => {
                if is_mut {
                    write!(out, "&mut ")?;
                } else {
                    write!(out, "&")?;
                }
                self.expr(inner, out)?;
            }
        }
        Ok(())
    }

    /// Plan 310 Phase 2: Buffer an EscapeFallback (W0007) warning. Never writes
    /// to Sink — warnings go to self.warnings only, preserving .expected.rs
    /// byte-diff integrity.
    fn emit_escape_warning(
        &mut self,
        name: &str,
        tier: crate::trans::escape::OwnershipTier,
        reason: &str,
    ) {
        use crate::trans::escape::report;
        // Source span is unknown during transpilation (we don't carry source
        // positions through to this layer); use a zero-length placeholder.
        let span = report::span_at(0, 0);
        let warning = report::build_warning(&name.into(), tier, reason, span);
        self.warnings.push(warning);
    }

    /// PLAN-667 (F-05): 逃逸 tier 的可读理由（诊断用；无记录时按 tier 兜底）。
    fn escape_reason_or_default(
        &self,
        name: &str,
        tier: &crate::trans::escape::OwnershipTier,
    ) -> String {
        if self.current_fn_name.is_empty() {
            return tier.to_string();
        }
        self.escape_results
            .get(&self.current_fn_name)
            .and_then(|m| m.reason_for(self.current_scope_depth, &name.into()))
            .map(|s| s.to_string())
            .unwrap_or_else(|| tier.to_string())
    }

    /// Check if the Map expression's value type is a String type.
    /// Returns true when Map value type is StrOwned/StrSlice/StrFixed (meaning insert value
    /// needs .to_string() for &str literals), false for non-string Maps or unknown types.
    pub(super) fn expr_map_value_is_string(&self, map_expr: &Expr) -> bool {
        if let Expr::Ident(name) = map_expr {
            if let Some(ty) = self.local_var_types.get(name) {
                if let Some(v) = self.map_value_ty(ty) {
                    return matches!(v, Type::StrOwned | Type::StrSlice | Type::StrFixed(_));
                }
            }
        }
        // Unknown: default to no .to_string() on the value arg. The old
        // conservative-true fallback appended `.to_string()` to ANY unresolved
        // map value — which breaks struct values (e.g. `MutexGuard<HashMap<str, WikiPage>>`
        // resolved through wrappers below; when unresolvable, a String value
        // is already String so `.to_string()` was a no-op anyway).
        false
    }

    /// Resolve the value type of a map-typed type, unwrapping common wrapper
    /// generics (Mutex, MutexGuard, Arc, …) and `GenericInstance` map spellings
    /// (`HashMap<K, V>`, `BTreeMap<K, V>`) so insert value-arg coercion sees
    /// through `self.pages.lock().unwrap()` (a `MutexGuard<HashMap<…>>`).
    fn map_value_ty(&self, ty: &Type) -> Option<Type> {
        match ty {
            Type::Map(_, v) => Some((**v).clone()),
            Type::GenericInstance(inst) => {
                let base = inst.base_name.as_str();
                match base {
                    "Map" | "HashMap" | "BTreeMap" | "IndexMap" => inst.args.get(1).cloned(),
                    // Wrapper generics: unwrap to the inner type and recurse.
                    "Mutex" | "MutexGuard" | "RwLock" | "RwLockReadGuard" | "RwLockWriteGuard"
                    | "RefCell" | "Cell" | "Arc" | "Rc" | "Box" => {
                        inst.args.first().and_then(|inner| self.map_value_ty(inner))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Check if an expression produces &str that needs .to_string() for String return
    pub(super) fn expr_needs_string_coercion(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Str(_) | Expr::CStr(_) => true,
            // PLAN-010 T2 (009 ledger 017): `#{...}` comptime embeds evaluate
            // to raw literals (&str) — in a String-returning fn they need the
            // same materialization as plain string literals (E0308).
            Expr::Comptime(_) => true,
            Expr::Index(_, idx) => matches!(idx.as_ref(), Expr::Range(_)),
            Expr::Ident(name) => self.current_fn_str_params.contains(name),
            // x.slice(...) is transpiled to x[n..] which produces &str
            Expr::Call(call) => {
                if let Expr::Dot(_, method) = call.name.as_ref() {
                    method == "slice"
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Plan 019 Phase 1: does this expression denote an Auto `str` value?
    /// Auto str semantics are code-unit (JS-like); a2r must count/slice by
    /// CHARS, not bytes, or any non-ASCII text mis-scans. True for str-typed
    /// params (current fn's str params + tracked local types) and string
    /// literals; false conservatively otherwise (list receivers keep .len()).
    pub(super) fn expr_is_string_like(&self, e: &Expr) -> bool {
        match e {
            Expr::Ident(name) => {
                if self.current_fn_str_params.contains(name) {
                    return true;
                }
                matches!(
                    self.local_var_types.get(name),
                    Some(Type::StrOwned)
                        | Some(Type::StrFixed(_))
                        | Some(Type::StrSlice)
                        | Some(Type::CStrLit)
                )
            }
            Expr::Str(_) | Expr::CStr(_) => true,
            _ => false,
        }
    }

    /// Plan 019 Phase 1: emit Auto str slicing for a string receiver using
    /// CHAR positions. s.slice(a) → s.chars().skip(a)...collect;
    /// s.slice(a, b) → s.chars().take(b).skip(a)...collect (take-then-skip
    /// equals skip-then-take(b-a) but emits each argument exactly once).
    /// Byte slicing would panic on non-boundary positions in non-ASCII text.
    pub(super) fn emit_str_slice_chars(
        &mut self,
        recv: &Expr,
        args: &[Arg],
        out: &mut impl Write,
    ) -> AutoResult<()> {
        self.expr(recv, out)?;
        write!(out, ".chars()")?;
        if args.len() >= 2 {
            if let Some(Arg::Pos(b)) = args.get(1) {
                write!(out, ".take((")?;
                self.expr(b, out)?;
                write!(out, ") as usize)")?;
            }
        }
        if let Some(Arg::Pos(a)) = args.first() {
            write!(out, ".skip((")?;
            self.expr(a, out)?;
            write!(out, ") as usize)")?;
        }
        write!(out, ".collect::<String>()")?;
        Ok(())
    }

    /// PLAN-018: List 元素非 Copy(str 系)时索引改写后置 .clone()
    /// (`return G[i]` / `let x = G[i]` 的 String 移出即 E0507;int/bool
    /// 元素 Copy 不加,维持既有金样)。
    pub(super) fn list_elem_needs_index_clone(&self, object: &Expr) -> bool {
        let ty = match object {
            Expr::Ident(name) => self
                .global_var_types
                .get(name.as_str())
                .or_else(|| self.local_var_types.get(name.as_str()))
                .cloned(),
            _ => None,
        };
        matches!(ty, Some(Type::List(inner)) if matches!(inner.as_ref(), Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit))
    }

    /// Write a return expression with automatic .to_string() coercion when needed.
    /// `add_semi`: whether to append a semicolon (false for match arm bodies).
    pub(super) fn write_return_expr(
        &mut self,
        expr: &Expr,
        out: &mut impl Write,
        add_semi: bool,
    ) -> AutoResult<()> {
        // If returning a &str parameter ident directly, wrap in .to_string()
        if let Expr::Ident(name) = expr {
            if self.current_fn_str_params.contains(name) {
                write!(out, "return {}.to_string()", name)?;
                if add_semi {
                    out.write(b";")?;
                }
                return Ok(());
            }
        }
        let needs_to_string =
            self.ret_type_needs_string_coercion() && self.expr_needs_string_coercion(expr);
        // Plan 013 (B1/BUG2): returning `self.field` of an owned non-Copy type
        // from a &self method needs `.clone()` (E0507 otherwise).
        let needs_self_clone = Self::is_self_dot(expr) && self.ret_type_is_owned_noncopy();
        // Plan 399 Phase 11.3: returning a global var emits `G.lock().unwrap()`
        // .clone()` directly (skip self.expr which prepends `*`, avoiding the
        // `*guard.clone()` precedence trap — MutexGuard isn't Clone).
        let global_ret_name = match expr {
            Expr::Ident(name) if self.is_global_var(name) && self.ret_type_is_owned_noncopy() => {
                Some(name.clone())
            }
            _ => None,
        };
        if let Some(name) = &global_ret_name {
            let static_name = self.global_var_static_name(name);
            write!(out, "return {}.lock().unwrap().clone()", static_name)?;
            if add_semi {
                out.write(b";")?;
            }
            return Ok(());
        }
        // Plan 399 Phase 11.4: `return Some(iter_var)` where iter_var is a
        // borrowed loop variable (&T) — Some(&T) vs expected Some<T> is E0308.
        // Clone the inner: Some(iter_var.clone()).
        if let Expr::Call(call) = expr {
            if let Expr::Ident(callee) = call.name.as_ref() {
                if callee.as_ref() == "Some" {
                    if let Some(crate::ast::Arg::Pos(inner)) = call.args.args.first() {
                        if let Expr::Ident(inner_name) = inner {
                            if self.borrowed_iter_vars.contains(inner_name) {
                                write!(out, "return Some(")?;
                                self.expr(inner, out)?;
                                write!(out, ".clone())")?;
                                if add_semi {
                                    out.write(b";")?;
                                }
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        // Plan 032 G2-core: `Some(v) => return v` over a `.get()` scrutinee —
        // the binding is `&V`; returning the owned value needs `.clone()`
        // (E0515/E0308 otherwise). Also the wrapped forms `return Ok(v)` /
        // `return Some(v)` (dedicated Expr::Ok / Expr::Some nodes, plus a
        // bare-Call shape for module-qualified constructors).
        if let Expr::Ident(name) = expr {
            if self.get_ref_bindings.contains(name) {
                write!(out, "return {}.clone()", name)?;
                if add_semi {
                    out.write(b";")?;
                }
                return Ok(());
            }
        }
        if let Expr::Ok(inner) = expr {
            if let Expr::Ident(name) = inner.as_ref() {
                if self.get_ref_bindings.contains(name) {
                    write!(out, "return Ok({}.clone())", name)?;
                    if add_semi {
                        out.write(b";")?;
                    }
                    return Ok(());
                }
            }
        }
        // NOTE: deliberately NOT extended to borrowed_iter_vars — that set is
        // function-scoped and accumulates, so a later is-arm binding that
        // shadowed an earlier same-named loop var would get a spurious
        // `.clone()` (server.at `Some(m)`). Loop-var Some(c) clones are the
        // consumer sources' explicit `.clone()` to write instead.
        if let Expr::Some(inner) = expr {
            if let Expr::Ident(name) = inner.as_ref() {
                if self.get_ref_bindings.contains(name) {
                    write!(out, "return Some({}.clone())", name)?;
                    if add_semi {
                        out.write(b";")?;
                    }
                    return Ok(());
                }
            }
        }
        if let Expr::Call(call) = expr {
            if let Expr::Ident(callee) = call.name.as_ref() {
                if matches!(callee.as_ref(), "Ok" | "Some") {
                    if let Some(crate::ast::Arg::Pos(Expr::Ident(inner))) = call.args.args.first() {
                        if self.get_ref_bindings.contains(inner) {
                            write!(out, "return {}({}.clone())", callee, inner)?;
                            if add_semi {
                                out.write(b";")?;
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
        // PLAN-010 T2 (009 ledger 008_arc_dyn_spec): `return map.get(k)` in a
        // fn returning Option<T> (owned, non-Copy inner) yields Option<&T> —
        // E0308. Clone the inner (Arc/struct/enum payload: Clone-derived).
        if let Expr::Call(call) = expr {
            if matches!(call.name.as_ref(), Expr::Dot(_, m) if m.as_str() == "get")
                && call.args.args.len() == 1
            {
                if let Some(Type::Option(inner)) = &self.current_fn_ret_type {
                    if !Self::is_primitive_copy(inner)
                        && !matches!(inner.as_ref(), Type::Unknown | Type::Void)
                    {
                        out.write(b"return ")?;
                        self.expr(expr, out)?;
                        out.write(b".cloned()")?;
                        if add_semi {
                            out.write(b";")?;
                        }
                        return Ok(());
                    }
                }
            }
        }
        // Plan 531 P525-4: May 裸值 return——`fn f() ?int { return n * 10 }`
        // 的裸值表达式按 Auto 语义即 Some(值);此前直发 `return n * 10;`
        // 于 Option<T> 返回位 E0308(g34 语料被迫取显式 Some 规避形态)。
        // 仅包裹已知裸标量形;Option 形态(Some/Ok/None/Call-Some)与
        // Unknown 不动(保守避免双重包裹)。
        let already_option_shaped = match expr {
            Expr::Some(_) | Expr::None | Expr::Ok(_) => true,
            Expr::Call(c) => matches!(c.name.as_ref(),
                Expr::Ident(n) if matches!(n.as_str(), "Some" | "Ok")),
            _ => false,
        };
        let expr_bare_scalar = matches!(
            self.infer_type_from_expr(expr),
            Type::Int
                | Type::Uint
                | Type::USize
                | Type::I64
                | Type::U64
                | Type::Float
                | Type::Double
                | Type::Bool
                | Type::Char
                | Type::Byte
                | Type::StrOwned
                | Type::StrSlice
                | Type::StrFixed(_)
                | Type::CStrLit
        );
        let may_wrap_bare = matches!(&self.current_fn_ret_type, Some(Type::Option(_)))
            && !already_option_shaped
            && expr_bare_scalar;
        out.write(b"return ")?;
        if may_wrap_bare {
            write!(out, "Some(")?;
        }
        self.expr(expr, out)?;
        if needs_to_string {
            out.write(b".to_string()")?;
        }
        if needs_self_clone && !needs_to_string {
            out.write(b".clone()")?;
        }
        // Plan 376F: Integer cast on return when fn return type differs from expr type.
        if let Some(ret_ty) = &self.current_fn_ret_type {
            let expr_ty = self.infer_type_from_expr(expr);
            let need_cast = match (ret_ty, &expr_ty) {
                (Type::Int, Type::Uint) => Some(" as i64"),
                (Type::Uint, Type::Int) => Some(" as u32"),
                (Type::USize, Type::Int) => Some(" as usize"),
                (Type::USize, Type::Uint) => Some(" as usize"),
                (Type::Int, Type::USize) => Some(" as i64"),
                (Type::Uint, Type::USize) => Some(" as u32"),
                _ => None,
            };
            if let Some(cast) = need_cast {
                write!(out, "{}", cast)?;
            }
        }
        if may_wrap_bare {
            write!(out, ")")?;
        }
        if add_semi {
            out.write(b";")?;
        }
        Ok(())
    }

    /// Plan 433 A1: is this operand char-like — a char literal, or an ident
    /// whose known type is char? Used by the Bina char-coercion below.
    fn expr_is_char_like(&self, e: &Expr) -> bool {
        match e {
            Expr::Char(_) => true,
            Expr::Ident(n) => self
                .local_var_types
                .get(n.as_str())
                .map(|t| matches!(t, Type::Char))
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Plan 433 A1: bare `.len` on a known struct with a `len` FIELD
    /// (Tok.len) parses as a zero-arg method call but is a property read —
    /// emit the field access instead of Vec::len(). Returns true when emitted.
    pub(super) fn try_emit_struct_len_field(
        &mut self,
        object: &Expr,
        method_name: &Name,
        arg_count: usize,
        out: &mut impl Write,
    ) -> AutoResult<bool> {
        if method_name.as_str() != "len" || arg_count != 0 {
            return Ok(false);
        }
        if let Expr::Ident(name) = object {
            if let Some(Type::User(td)) = self.local_var_types.get(name.as_str()) {
                let has_len_field = self
                    .struct_field_types
                    .get(td.name.as_str())
                    .map(|fields| fields.iter().any(|(f, _)| f.as_str() == "len"))
                    .unwrap_or(false);
                if has_len_field {
                    self.expr(object, out)?;
                    write!(out, ".len")?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Plan 433 A1: does this expression denote an Auto `List` receiver — a
    /// List-typed local/param, or a List-typed field of a known struct var
    /// (`c.ins`, `p.toks`)? Auto List semantics (get=i-th element, set=in-place
    /// element write) differ from HashMap's get/insert; without this check,
    /// `list.get(i)` falls through to Vec::get → Option and `list.set(i,v)`
    /// becomes insert (shifts elements instead of replacing).
    /// Plan 531 真缺陷②: .get 接收者是否 List/Array。Ident 接收者查
    /// local_var_types;self.<field> 接收者经 current_impl_type 查
    /// struct_field_types 字段实型——514 W3 曾无条件视 self.* 为 List,
    /// 使 Map 字段 .get(key) 被改写 [(key) as usize](str as usize 非法
    /// 强转)。实型不可解析的 self 字段回落 false(514 W3 前行为:
    /// .get 方法形由 Vec::get/HashMap::get 各自正确分派)。
    pub(super) fn recv_is_list_like(&self, object: &Expr) -> bool {
        if let Expr::Ident(name) = object {
            // PLAN-018: 全局 var List(Array 同款)——.get(i)/.set(i,v)
            // 索引改写须覆盖全局接收者(`(*G.lock().unwrap())[(i) as usize]`),
            // 否则落回 Vec::get(Option/i64 实参 E0277)。
            if let Some(ty) = self.global_var_types.get(name.as_str()) {
                return matches!(ty, Type::List(_) | Type::Array(_));
            }
            self.local_var_types
                .get(name.as_str())
                .map(|ty| matches!(ty, Type::List(_) | Type::Array(_)))
                .unwrap_or(false)
        } else if let Expr::Dot(base, field) = object {
            if matches!(base.as_ref(), Expr::Ident(n) if n.as_str() == "self") {
                self.current_impl_type
                    .as_ref()
                    .and_then(|ty| self.struct_field_types.get(ty.as_str()))
                    .map(|fields| {
                        fields
                            .iter()
                            .any(|(f, t)| f == field && matches!(t, Type::List(_) | Type::Array(_)))
                    })
                    .unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Plan 433 A1: the Auto VM coerces char literals to their code point
    /// when mixed with int operands (`c >= '0'`, `char_at(i) - '0'`). Rust
    /// char vs i64 is E0308 — when exactly one side of a binary op is
    /// char-like and the other is not, cast the char side to i64. Both-char
    /// operands keep the Rust char comparison (existing goldens unchanged).
    fn bina_char_cast(&self, lhs: &Expr, rhs: &Expr) -> (bool, bool) {
        match (self.expr_is_char_like(lhs), self.expr_is_char_like(rhs)) {
            (true, false) => (true, false),
            (false, true) => (false, true),
            _ => (false, false),
        }
    }

    /// PLAN-885 G2: `c >= "0"` — `c` is an owned String (from a single-char
    /// slice/char iteration), `"0"` a &str literal. Rust has no `String`
    /// PartialOrd with `&str` (E0277). When exactly one side of a comparison
    /// is a known str variable (String) and the other is a str literal,
    /// borrow the variable side with `.as_str()`.
    fn bina_str_literal_borrow(&self, lhs: &Expr, rhs: &Expr) -> (bool, bool) {
        let is_str_var = |e: &Expr| match e {
            Expr::Ident(n) => self
                .local_var_types
                .get(n.as_str())
                .map(|t| {
                    matches!(
                        t,
                        Type::StrOwned | Type::StrFixed(_) | Type::CStrLit | Type::StrSlice
                    )
                })
                .unwrap_or(false),
            _ => false,
        };
        let is_str_lit = |e: &Expr| matches!(e, Expr::Str(_) | Expr::CStr(_));
        match (is_str_var(lhs), is_str_lit(rhs)) {
            (true, true) => (true, false),
            _ => match (is_str_lit(lhs), is_str_var(rhs)) {
                (true, true) => (false, true),
                _ => (false, false),
            },
        }
    }

    /// Escape Rust reserved keywords used as identifiers.
    /// Only applies to variable/parameter binding contexts, NOT type names or module paths.
    /// Plan 391 §7 follow-up: extract a dotted path from a chain of
    /// `Expr::Dot` rooted at an `Expr::Ident` (e.g. `std::env::var` parses as
    /// `Dot(Dot(Ident("std"), "env"), "var")` → `Some("std.env.var")`).
    ///
    /// Returns `None` if the chain isn't a pure ident-rooted dot chain (e.g. a
    /// real `obj.field` where `obj` is a call/local). Used to detect module
    /// paths created by the parser's `::` → `Dot` normalization (parser.rs
    /// Plan 391 D4) so codegen can emit `::` between module segments instead of
    /// `.` — `std.env.var(...)` would be invalid Rust; it must be
    /// `std::env::var(...)`.
    pub(super) fn dot_chain_path(expr: &Expr) -> Option<String> {
        // Walk the Dot chain collecting segments right-to-left, requiring the
        // root to be a bare Ident.
        let mut segs: Vec<&str> = Vec::new();
        let mut cur = expr;
        loop {
            match cur {
                Expr::Dot(inner, field) => {
                    segs.push(field.as_str());
                    cur = inner;
                }
                Expr::Ident(name) => {
                    segs.push(name.as_str());
                    segs.reverse();
                    return Some(segs.join("."));
                }
                _ => return None,
            }
        }
    }

    /// Plan 391 §7 follow-up: does `path` (a dotted path like "std.env" or
    /// "std.env.var") correspond to a known `use.rs` import? Matches if any
    /// use-path equals `path` (with `.` → `::`) or starts with it as a module
    /// prefix — so `std.env` matches `use.rust std::env` (the import is
    /// "std::env"), and `std.env.var` also matches (the fn lives under that
    /// module). This is what lets a Dot chain be recognized as a module path
    /// rather than object field access.
    pub(super) fn path_matches_use_rust(&self, path: &str) -> bool {
        // Convert dotted path to `::` form once for prefix comparisons.
        let path_colon = path.replace('.', "::");
        self.uses.iter().any(|u| {
            let u_str = u.as_str();
            u_str == path_colon                     // use.rs std::env  matches "std.env"
                || u_str == path                    // defensive: use stored as dotted
                || u_str.starts_with(&format!("{}::", path_colon)) // use.rs std::env::var matches "std.env" (prefix)
        })
    }

    /// Plan 514 W2 (P511-1 根因②): Auto 二元算子优先级（镜像 a2r.at
    /// parser.at infix_l 表；越大越紧）。供 Bina 发射的优先级括号判定。
    fn auto_op_prec(op: &Op) -> u8 {
        match op {
            Op::Asn => 1,
            Op::AddEq | Op::SubEq => 3,
            Op::MulEq | Op::DivEq | Op::ModEq => 5,
            Op::Or | Op::QuestionQuestion => 9,
            Op::And => 11,
            Op::Eq | Op::Neq => 13,
            Op::Lt | Op::Gt | Op::Le | Op::Ge => 15,
            Op::Range | Op::RangeEq => 17,
            Op::Add | Op::Sub => 19,
            Op::Mul | Op::Div | Op::Mod => 21,
            _ => 100,
        }
    }

    /// Plan 577 ② (DEBTS 016 Phase 0): is this type a Map container (native
    /// `Map<K,V>` or the GenericInstance aliases the map-literal path accepts)?
    fn type_is_map_like(ty: &Type) -> bool {
        matches!(ty, Type::Map(_, _))
            || matches!(ty, Type::GenericInstance(g)
                if matches!(g.base_name.as_str(), "Map" | "HashMap" | "BTreeMap"))
    }

    /// Plan 577 ②: does this index receiver denote a Map? Mirrors the
    /// `contains`→contains_key dispatch: Ident → local_var_types; field read
    /// → any known struct field of Map type (owner often unresolvable).
    fn recv_is_map(&self, recv: &Expr) -> bool {
        match recv {
            Expr::Ident(name) => self
                .local_var_types
                .get(name)
                .map(Self::type_is_map_like)
                .unwrap_or(false),
            Expr::Dot(inner, field) => {
                if matches!(inner.as_ref(), Expr::Ident(_)) {
                    self.struct_field_types.values().any(|fields| {
                        fields
                            .iter()
                            .any(|(fname, fty)| fname == field && Self::type_is_map_like(fty))
                    })
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub(super) fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            // Literals
            Expr::Int(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::Uint(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::I8(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::U8(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::I64(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::U64(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::Byte(b) => write!(out, "{}", b).map_err(Into::into),
            Expr::Float(f, _) => {
                let s = format!("{}", f);
                // Ensure float literal has decimal point (e.g. 2 -> 2.0)
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    write!(out, "{}", s)
                } else {
                    write!(out, "{}.0", s)
                }
                .map_err(Into::into)
            }
            Expr::Double(d, _) => {
                let s = format!("{}", d);
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    write!(out, "{}", s)
                } else {
                    write!(out, "{}.0", s)
                }
                .map_err(Into::into)
            }
            Expr::Bool(b) => write!(out, "{}", b).map_err(Into::into),
            Expr::Char(c) => {
                // In a2r, Auto char maps to Rust char (not i32)
                // Plan 447 ③-前置: is-on-int scrutinee (codepoint locals like
                // the lexer's char_at results) — char PATTERNS must match the
                // i64, so emit the code point as a decimal literal
                // (`match esc { 110 => ... }`; a `'n'` pattern is E0308).
                if self.int_match_scrutinee {
                    let code = *c as u32;
                    return write!(out, "{}", code).map_err(Into::into);
                }
                if *c == '\n' {
                    write!(out, "'\\n'")
                } else if *c == '\t' {
                    write!(out, "'\\t'")
                } else if *c == '\r' {
                    write!(out, "'\\r'")
                } else if *c == '\0' {
                    write!(out, "'\\0'")
                } else if *c == '\\' {
                    write!(out, "'\\\\'")
                } else if *c == '\'' {
                    write!(out, "'\\''")
                } else {
                    write!(out, "'{}'", c)
                }
            }
            .map_err(Into::into),
            Expr::Str(s) => write!(out, "\"{}\"", escape_str(s)).map_err(Into::into),
            Expr::CStr(s) => write!(out, "\"{}\"", escape_str(s)).map_err(Into::into),
            Expr::Ident(name) => {
                // Plan 387: inside a task hook/handler body, a bare state-field
                // identifier reads `self.<field>`.
                if self.in_task_body && self.task_state_fields.contains(name.as_str()) {
                    return write!(out, "self.{}", Self::rust_ident(name.as_str()))
                        .map_err(Into::into);
                }
                // Plan 151: Global variable access - add .lock().unwrap() pattern.
                // Plan 347: reads must dereference the MutexGuard (`*G.lock()`)
                // so the value is usable in arithmetic, comparisons, indexing,
                // and casts — otherwise Rust sees a `MutexGuard<i32>` and
                // rejects `g + 1`, `g < n`, `g as usize`, etc. The assignment
                // LHS path emits its own `*` (store() write path), so this
                // read-only `*` never conflicts.
                if self.is_global_var(name) {
                    let static_name = self.global_var_static_name(name);
                    // PLAN-018: 括号绑定解引用范围。裸 `*G.lock().unwrap()`
                    // 后接方法/字段时整链被解引用(`*…len()` = deref usize,
                    // E0614);括号后方法作用于 MutexGuard 的 Deref 目标
                    // (Vec/i64),标量读/算术/转型语义不变。
                    write!(out, "(*{}.lock().unwrap())", static_name)
                } else if self.is_write_captured(name.as_str()) {
                    // Plan 419 §4.5: 写捕获绑定是 Rc<RefCell<T>> —— 读走
                    // .borrow(),赋值 LHS(assign_lhs_depth>0)走 .borrow_mut()。
                    if self.assign_lhs_depth > 0 {
                        write!(out, "*{}.borrow_mut()", Self::rust_ident(name.as_str()))
                    } else {
                        write!(out, "*{}.borrow()", Self::rust_ident(name.as_str()))
                    }
                } else if let Some(rust_name) = Self::auto_type_to_rust(name.as_str()) {
                    write!(out, "{}", rust_name)
                } else if name.as_str() == "StringBuilder" {
                    // Plan 347: Auto VM `StringBuilder` type -> a2r-std runtime
                    // type. Emitted fully-qualified so it resolves with or
                    // without the glob `use a2r_std::*` import (covers the
                    // `StringBuilder.new(...)` constructor call site).
                    self.a2r_std_used.set(true);
                    write!(out, "a2r_std::StringBuilder")
                } else {
                    write!(out, "{}", Self::rust_ident(name.as_str()))
                }
            }
            .map_err(Into::into),
            Expr::GenName(name) => {
                write!(out, "{}", Self::rust_ident(name.as_str())).map_err(Into::into)
            }
            Expr::Nil => write!(out, "None").map_err(Into::into),
            Expr::Null => write!(out, "None").map_err(Into::into),

            // Plan 120/159: Option and Result constructors
            Expr::Some(e) => {
                write!(out, "Some(")?;
                self.expr(e, out)?;
                if matches!(e.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                    write!(out, ".to_string()")?;
                }
                // Plan 019 Phase 1: Some(&str param) into an Option<String>
                // field/return — the payload renders as &str and needs the
                // same materialization as string literals.
                if let Expr::Ident(name) = e.as_ref() {
                    if self.current_fn_str_params.contains(name) {
                        write!(out, ".to_string()")?;
                    }
                }
                // Plan 013 (B16): when returning into Option<uint> and the
                // payload is a bare ident bound to an Int variant (i32), cast
                // to u32 — Rust won't auto-widen i32→u32 in Some(...).
                if matches!(&self.current_fn_ret_type, Some(Type::Option(inner)) if matches!(inner.as_ref(), Type::Uint))
                    && matches!(e.as_ref(), Expr::Ident(_))
                {
                    write!(out, " as u32")?;
                }
                write!(out, ")").map_err(Into::into)
            }
            Expr::None => write!(out, "None").map_err(Into::into),
            Expr::Ok(e) => {
                write!(out, "Ok(")?;
                self.expr(e, out)?;
                // When Ok contains a string literal but the function returns Result<String, ...>,
                // add .to_string() to convert &str -> String
                if matches!(e.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                    if let Some(ref ret) = self.current_fn_ret_type {
                        if Self::ret_is_result_string(ret) {
                            write!(out, ".to_string()")?;
                        }
                    }
                }
                // PLAN-010 T2 (009 ledger 032_fn_result_enum): Ok(str-param)
                // into Result<String, _> — the param renders as &str and needs
                // the same materialization Some(...) already applies
                // (E0308 expected String, found &str otherwise).
                if let Expr::Ident(name) = e.as_ref() {
                    if self.current_fn_str_params.contains(name) {
                        if let Some(ref ret) = self.current_fn_ret_type {
                            if Self::ret_is_result_string(ret) {
                                write!(out, ".to_string()")?;
                            }
                        }
                    }
                }
                write!(out, ")").map_err(Into::into)
            }
            Expr::Err(e) => {
                write!(out, "Err(")?;
                // Plan 013 (B14-followup): a concrete error enum variant
                // (Type.Variant(...) where Type is a known enum/tag) must NOT
                // be wrapped in Box::new — the function returns Result<_, E>
                // with a concrete E, and Box<E> is a type mismatch. Detect this
                // even when current_fn_err_type wasn't inferred (timing/order).
                let is_concrete_enum_err = match e.as_ref() {
                    Expr::Call(call) => {
                        if let Expr::Dot(obj, _) = call.name.as_ref() {
                            if let Expr::Ident(type_name) = obj.as_ref() {
                                self.tag_types.contains(type_name)
                                    || self.known_enum_names.contains(type_name)
                                    || self.local_struct_types.contains(type_name)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    Expr::Dot(obj, _) => {
                        if let Expr::Ident(type_name) = obj.as_ref() {
                            self.tag_types.contains(type_name)
                                || self.known_enum_names.contains(type_name)
                                || self.local_struct_types.contains(type_name)
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                if self.current_fn_err_type.is_some() || is_concrete_enum_err {
                    // Concrete error type — no Box::new needed
                    self.expr(e, out)?;
                } else if self.current_fn_is_result && matches!(e.as_ref(), Expr::Ident(_)) {
                    // Plan 013 (B16): re-throwing a matched error ident
                    // (`Err(e) => return Err(e)`) in a Result<_, Concrete> fn —
                    // the ident already holds the concrete error, no Box.
                    self.expr(e, out)?;
                } else if matches!(e.as_ref(), Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_)) {
                    // Plan 016 Phase A: f-string (format!()) also returns String,
                    // so .into() works for Result<_, String> without Box::new.
                    self.expr(e, out)?;
                    write!(out, ".into()")?;
                } else {
                    // Box::new() for concrete types -> Box<dyn Error>
                    write!(out, "Box::new(")?;
                    self.expr(e, out)?;
                    write!(out, ")")?;
                }
                write!(out, ")").map_err(Into::into)
            }
            // Plan 6B-4.14: Smart pointer constructors
            // Plan 390 §15.11 (L2 转正): wrapping a spec-bound ident (already
            // `Box<dyn Trait>`) uses `Arc::from`/`Box::from` — `Arc::from(box)`
            // converts `Box<dyn Tool>` → `Arc<dyn Tool>` (single wrap). Plain
            // `Arc::new(x)` would produce `Arc<Box<dyn Tool>>` (double wrap).
            Expr::BoxExpr(e) => {
                let spec_bound = matches!(e.as_ref(), Expr::Ident(name) if self.spec_bound_idents.contains(name));
                write!(out, "Box::{}(", if spec_bound { "from" } else { "new" })?;
                self.expr(e, out)?;
                write!(out, ")").map_err(Into::into)
            }
            Expr::ArcExpr(e) => {
                let spec_bound = matches!(e.as_ref(), Expr::Ident(name) if self.spec_bound_idents.contains(name));
                write!(out, "Arc::{}(", if spec_bound { "from" } else { "new" })?;
                self.expr(e, out)?;
                write!(out, ")").map_err(Into::into)
            }

            // Operators
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Dot => {
                        // **Phase 1.1 & 2: Special field names (@, *, view, mut, take)**
                        if let Expr::Ident(field_name) = rhs.as_ref() {
                            match field_name.as_str() {
                                "@" => {
                                    // x.@ -> raw pointer (address-of)
                                    self.expr(lhs, out)?;
                                    write!(out, " as *mut _")?;
                                    return Ok(());
                                }
                                "*" => {
                                    // y.* -> dereference
                                    write!(out, "*")?;
                                    self.expr(lhs, out)?;
                                    return Ok(());
                                }
                                "view" => {
                                    // Plan 310 Phase 2: route through emit_borrow.
                                    self.emit_borrow(lhs, false, out)?;
                                    return Ok(());
                                }
                                "mut" => {
                                    self.emit_borrow(lhs, true, out)?;
                                    return Ok(());
                                }
                                "take" => {
                                    // x.take -> x (move semantics)
                                    self.expr(lhs, out)?;
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }

                        // Member access: expr.field or .field (shorthand for self.field)
                        match lhs.as_ref() {
                            Expr::Nil | Expr::Null => {
                                // .field -> self.field
                                write!(out, "self.")?;
                                self.expr(rhs, out)?;
                            }
                            _ => {
                                // Check if this is enum variant access: Type::Variant
                                // Use :: if rhs is an identifier starting with uppercase (enum variant convention)
                                // OR if lhs starts with uppercase (type name for static method: Type.method())
                                let is_enum_variant = if let Expr::Ident(rhs_name) = rhs.as_ref() {
                                    rhs_name
                                        .chars()
                                        .next()
                                        .map(|c| c.is_uppercase())
                                        .unwrap_or(false)
                                } else {
                                    false
                                };

                                // Check if lhs is a type name (starts with uppercase, is a Rust primitive,
                                // or is a known module from use.rust imports)
                                let is_type_name = if let Expr::Ident(lhs_name) = lhs.as_ref() {
                                    let name = lhs_name.as_str();
                                    // Plan 384 S2: a known local variable / param / `self`
                                    // is a value, NOT a type/module — even if its name happens
                                    // to match a `use` path leaf (e.g. local `sse` vs module
                                    // `axum::response::sse`). Short-circuit before the uses check.
                                    let is_local =
                                        name == "self" || self.local_var_types.contains_key(name);
                                    !is_local
                                        && (
                                            name.chars()
                                                .next()
                                                .map(|c| c.is_uppercase())
                                                .unwrap_or(false)
                                                || matches!(
                                                    name,
                                                    "u8" | "u16"
                                                        | "u32"
                                                        | "u64"
                                                        | "u128"
                                                        | "usize"
                                                        | "i8"
                                                        | "i16"
                                                        | "i32"
                                                        | "i64"
                                                        | "i128"
                                                        | "isize"
                                                        | "f32"
                                                        | "f64"
                                                        | "bool"
                                                        | "char"
                                                )
                                                || self.uses.iter().any(|u| {
                                                    let u_str = u.as_str();
                                                    u_str == name
                                                        || u_str.ends_with(&format!("::{}", name))
                                                })
                                                || self.module_types.contains_key(name)
                                            // Plan 264: known module name
                                        )
                                } else {
                                    false
                                };

                                // Check if lhs is a type-like expression (identifier or module.Type chain)
                                let lhs_is_type = if matches!(lhs.as_ref(), Expr::Ident(_)) {
                                    is_enum_variant || is_type_name
                                } else if let Expr::Dot(il, ir) = lhs.as_ref() {
                                    // module.Type or nested field like circle.center
                                    // Only treat as type-like if inner field starts with uppercase
                                    // or leftmost segment is a known module
                                    let inner_is_type = ir
                                        .chars()
                                        .next()
                                        .map(|c| c.is_uppercase())
                                        .unwrap_or(false);
                                    let leftmost_is_module = if let Expr::Ident(name) = il.as_ref()
                                    {
                                        self.uses.iter().any(|u| {
                                            let u_str = u.as_str();
                                            u_str == name.as_str()
                                                || u_str.ends_with(&format!("::{}", name))
                                        }) || self.module_types.contains_key(name.as_str())
                                    // Plan 264
                                    } else {
                                        false
                                    };
                                    inner_is_type || leftmost_is_module
                                } else if let Expr::Bina(il, Op::Dot, ir) = lhs.as_ref() {
                                    // module.Type or module.module.Type chain (nested Dot via Bina)
                                    // Same check: inner field must be type-like or leftmost must be a module
                                    let inner_is_type = if let Expr::Ident(name) = ir.as_ref() {
                                        name.chars()
                                            .next()
                                            .map(|c| c.is_uppercase())
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };
                                    let leftmost_is_module = if let Expr::Ident(name) = il.as_ref()
                                    {
                                        self.uses.iter().any(|u| {
                                            let u_str = u.as_str();
                                            u_str == name.as_str()
                                                || u_str.ends_with(&format!("::{}", name))
                                        }) || self.module_types.contains_key(name.as_str())
                                    // Plan 264
                                    } else {
                                        false
                                    };
                                    inner_is_type || leftmost_is_module
                                } else {
                                    false
                                };

                                if lhs_is_type {
                                    // Type::Variant or Type::method()
                                    // Plan 264: If lhs is a known module name, qualify with crate::
                                    if let Expr::Ident(lhs_name) = lhs.as_ref() {
                                        if self.module_types.contains_key(lhs_name.as_str()) {
                                            if self.merge_mode {
                                                write!(out, "{}::", lhs_name.as_str())?;
                                            } else if lhs_name.as_str() == self.current_module_name
                                            {
                                                write!(out, "{}::", lhs_name.as_str())?;
                                            } else {
                                                write!(out, "crate::{}::", lhs_name.as_str())?;
                                            }
                                            self.expr(rhs, out)?;
                                        } else {
                                            self.expr(lhs, out)?;
                                            write!(out, "::")?;
                                            self.expr(rhs, out)?;
                                        }
                                    } else {
                                        self.expr(lhs, out)?;
                                        write!(out, "::")?;
                                        self.expr(rhs, out)?;
                                    }
                                } else {
                                    // expr.field or expr.method()
                                    // Parenthesize lhs if it's a binary op (e.g., (a / b).method())
                                    // or a unary deref (Plan 379: (*x).clone() — otherwise
                                    // `*x.clone()` would parse as *(x.clone()) in Rust).
                                    let needs_parens =
                                        matches!(lhs.as_ref(),
                                            Expr::Bina(_, op, _) if !matches!(op, Op::Dot)
                                        ) || matches!(lhs.as_ref(), Expr::Unary(Op::Mul, _));
                                    if needs_parens {
                                        write!(out, "(")?;
                                    }
                                    self.expr(lhs, out)?;
                                    if needs_parens {
                                        write!(out, ")")?;
                                    }
                                    // Auto's list/str length property `.length`
                                    // (field form, no parens) maps to Rust's
                                    // `.len()` (2026-08-23 auto-ai report
                                    // defect 2 — bare `.length` is E0609 on
                                    // Vec/String). Plan 019 Phase 1: string
                                    // receivers count chars (Auto str length
                                    // is code-unit based; bytes mis-scan
                                    // non-ASCII text).
                                    if let Expr::Ident(field_name) = rhs.as_ref() {
                                        if field_name.as_str() == "length" {
                                            // Plan 577 ④: same int-model cast
                                            // as the Expr::Dot field form and
                                            // the call() method form (bare
                                            // `.len()` breaks int contexts).
                                            let cast = !self.len_i32_cast_suppressed;
                                            if cast {
                                                write!(out, "(")?;
                                            }
                                            if self.expr_is_string_like(lhs) {
                                                write!(out, ".chars().count()")?;
                                            } else {
                                                write!(out, ".len()")?;
                                            }
                                            if cast {
                                                write!(out, " as i64)")?;
                                            }
                                            return Ok(());
                                        }
                                    }
                                    write!(out, ".")?;
                                    self.expr(rhs, out)?;
                                }
                            }
                        }
                    }
                    Op::Range => {
                        // Range: start..end
                        self.expr(lhs, out)?;
                        write!(out, "..")?;
                        self.expr(rhs, out)?;
                    }
                    Op::RangeEq => {
                        // Inclusive range: start..=end
                        self.expr(lhs, out)?;
                        write!(out, "..=")?;
                        self.expr(rhs, out)?;
                    }
                    Op::Add => {
                        if self.expr_contains_string(&lhs) || self.expr_contains_string(&rhs) {
                            // String involved — use format!
                            write!(out, "format!(\"{{}}{{}}\", ")?;
                            self.expr(&lhs, out)?;
                            write!(out, ", ")?;
                            self.expr(&rhs, out)?;
                            write!(out, ")")?;
                        } else {
                            // Default to numeric +
                            let (cast_l, cast_r) = self.bina_char_cast(&lhs, &rhs);
                            if cast_l {
                                write!(out, "(")?;
                            }
                            self.expr(&lhs, out)?;
                            if cast_l {
                                write!(out, ") as i64")?;
                            }
                            write!(out, " + ")?;
                            if cast_r {
                                write!(out, "(")?;
                            }
                            self.expr(&rhs, out)?;
                            if cast_r {
                                write!(out, ") as i64")?;
                            }
                        }
                    }
                    Op::Asn | Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq | Op::ModEq => {
                        // Plan 151: Handle global variable assignment
                        // Check if lhs is a global variable identifier
                        if let Expr::Ident(name) = lhs.as_ref() {
                            if self.is_global_var(name) {
                                // Global variable assignment: needs *VAR.lock().unwrap() OP= rhs
                                let static_name = self.global_var_static_name(name);

                                // Write the operator (without = for compound ops)
                                let op_str = match op {
                                    Op::Asn => "=",
                                    Op::AddEq => "+=",
                                    Op::SubEq => "-=",
                                    Op::MulEq => "*=",
                                    Op::DivEq => "/=",
                                    Op::ModEq => "%=",
                                    _ => op.op(),
                                };
                                // Plan 347: emit the assignment as a block that
                                // binds the RHS to a local `let` BEFORE taking the
                                // write lock. A `let` statement is a temporary
                                // scope, so any MutexGuard the RHS creates (e.g.
                                // reading the SAME global, `POS = POS + 1`) is
                                // dropped at the `;`, before the LHS locks the
                                // Mutex. Without this, `*POS.lock().unwrap() =
                                // *POS.lock().unwrap() + 1` deadlocks: the RHS
                                // guard lives until the end of the full statement
                                // and std::sync::Mutex is non-reentrant. (Mere
                                // `{ rhs }` braces do NOT create a temporary
                                // scope for inner temporaries — verified.)
                                write!(out, "{{ let __a2r_gv = ")?;
                                self.expr(rhs, out)?;
                                write!(
                                    out,
                                    "; *{}.lock().unwrap() {} __a2r_gv; }}",
                                    static_name, op_str
                                )?;
                                return Ok(());
                            }
                        }

                        // Plan 577 ② (DEBTS 016 Phase 0): `m[key] = value` on
                        // a Map receiver is an insert (Rust maps have no
                        // index-assign). Key/value str conversions mirror the
                        // .insert method-call path: string-like keys get
                        // .to_string(); values only when the Map's value type
                        // is String.
                        if matches!(op, Op::Asn) {
                            if let Expr::Index(arr, idx) = lhs.as_ref() {
                                if !matches!(idx.as_ref(), Expr::Range(_)) && self.recv_is_map(arr)
                                {
                                    self.expr(arr, out)?;
                                    write!(out, ".insert(")?;
                                    let key_is_string_like =
                                        matches!(idx.as_ref(), Expr::Str(_) | Expr::CStr(_))
                                            || if let Expr::Ident(n) = idx.as_ref() {
                                                self.local_var_types
                                                    .get(n)
                                                    .map(|ty| {
                                                        matches!(
                                                            ty,
                                                            Type::StrFixed(_)
                                                                | Type::StrSlice
                                                                | Type::StrOwned
                                                        )
                                                    })
                                                    .unwrap_or(false)
                                            } else {
                                                false
                                            };
                                    self.expr(idx, out)?;
                                    if key_is_string_like {
                                        write!(out, ".to_string()")?;
                                    }
                                    write!(out, ", ")?;
                                    let map_val_is_string = self.expr_map_value_is_string(arr);
                                    self.expr(rhs, out)?;
                                    if map_val_is_string {
                                        write!(out, ".to_string()")?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                            }
                        }

                        // Normal assignment: lhs OP rhs
                        // C11 (Plan 018 §12 a2r-11): mark LHS emission so the
                        // List `.get()` → index conversion skips `.clone()` —
                        // in-place element mutation writes the real element.
                        // Also: assigning to a `mut p T` (&mut) param emits
                        // `*p = x` (deref-assign into the caller's value).
                        if matches!(op, Op::Asn)
                            && matches!(lhs.as_ref(), Expr::Ident(n) if self.current_fn_mut_params.contains(n))
                        {
                            write!(out, "*")?;
                        }
                        self.assign_lhs_depth += 1;
                        self.expr(lhs, out)?;
                        self.assign_lhs_depth -= 1;
                        let op_str = match op {
                            Op::And => "&&",
                            Op::Or => "||",
                            Op::QuestionQuestion => "??",
                            _ => op.op(),
                        };
                        write!(out, " {} ", op_str)?;
                        // Plan 019 批次七 (E9): `s += rhs` on a String local —
                        // Rust's AddAssign is only implemented for &str, so
                        // borrow a String-typed rhs (`buf += &cs` coerces to
                        // push_str semantics; str params/literals pass as-is).
                        let addeq_str_borrow = matches!(op, Op::AddEq)
                            && matches!(lhs.as_ref(), Expr::Ident(n)
                                if matches!(self.local_var_types.get(n),
                                    Some(Type::StrOwned) | Some(Type::StrFixed(_))
                                    | Some(Type::StrSlice) | Some(Type::CStrLit)))
                            && (matches!(
                                self.infer_type_from_expr(rhs),
                                Type::StrOwned | Type::StrFixed(_) | Type::StrSlice | Type::CStrLit
                            ) || matches!(rhs.as_ref(), Expr::Call(c)
                                    if matches!(c.name.as_ref(),
                                        Expr::Dot(m, f)
                                            if matches!(m.as_ref(), Expr::Ident(mi) if mi.as_str() == "String")
                                                && f.as_str() == "fromCharCode")));
                        if addeq_str_borrow {
                            write!(out, "&")?;
                        }
                        // Plan 391 D1: reassignment `x = <expr>.len()` where x is a
                        // u64/i64/usize local — suppress the `as i32` cast (same
                        // rationale as the let-binding case in store()).
                        let saved_suppress = self.len_i32_cast_suppressed;
                        self.len_i32_cast_suppressed = matches!(op, Op::Asn)
                            && Self::expr_is_len_call(rhs)
                            && matches!(lhs.as_ref(), Expr::Ident(n)
                                if self.local_var_types.get(n)
                                    .map(|t| matches!(t, Type::U64 | Type::I64 | Type::USize))
                                    .unwrap_or(false));
                        // PLAN-026 T-03 配套: int 语境赋值 + List<int> 索引
                        // rhs —— as i32 窄化(lhs 为 int 局部或字段;字段情形
                        // 由 rhs 语义保证为 int 字段)。
                        let narrow_asn = self.expr_has_int_list_index(rhs)
                            && match lhs.as_ref() {
                                Expr::Ident(n) => {
                                    matches!(self.local_var_types.get(n.as_str()), Some(Type::Int))
                                }
                                Expr::Dot(_, _) => true,
                                _ => false,
                            };
                        if narrow_asn {
                            write!(out, "(")?;
                        }
                        self.expr(rhs, out)?;
                        if narrow_asn {
                            write!(out, ") as i32")?;
                        }
                        self.len_i32_cast_suppressed = saved_suppress;
                        // When assigning &str literal to a variable, add .to_string()
                        // In Auto, all str variables are String in Rust, so this is always correct
                        if matches!(op, Op::Asn)
                            && matches!(rhs.as_ref(), Expr::Str(_) | Expr::CStr(_))
                        {
                            // Plan 016 Phase A A4 cat 5f: also cover self.field
                            // assignment (Expr::Dot), e.g. self.buf = "" →
                            // self.buf = "".to_string(). Previously only bare idents
                            // got the coercion, so self.field = "" stayed &str.
                            if matches!(lhs.as_ref(), Expr::Ident(_) | Expr::Dot(_, _)) {
                                write!(out, ".to_string()")?;
                            }
                        }
                        // Plan 434 (242 #18-2): mirror store()'s let-init
                        // auto-clones for reassignment. `x = a.field` moves the
                        // field out — behind a &mut owner that's E0507; behind an
                        // owned owner reused later it's E0382. `x = y` rebinds a
                        // non-Copy local the same way. The let path has had both
                        // since Plan 013/348; the assignment path had neither.
                        if matches!(op, Op::Asn) && !self.expr_is_taskref(rhs) {
                            if let Expr::Dot(obj, _field) = rhs.as_ref() {
                                if let Expr::Ident(obj_name) = obj.as_ref() {
                                    // Only owners behind a reference (&mut params —
                                    // `mut a T` / merge-mode &mut) need the clone:
                                    // moving out of them is E0507. An OWNED local
                                    // may keep the bare move (19_ownership goldens
                                    // lock `acc = wp.description;` bare).
                                    let obj_is_mut_ref =
                                        self.current_fn_mut_params.contains(obj_name)
                                            || self
                                                .local_var_types
                                                .get(obj_name)
                                                .map(|ty| matches!(ty, Type::Reference(_)))
                                                .unwrap_or(false);
                                    let obj_is_copy = self
                                        .local_var_types
                                        .get(obj_name)
                                        .map(|ty| Self::is_copy_type(ty))
                                        .unwrap_or(true);
                                    if obj_is_mut_ref && !obj_is_copy {
                                        write!(out, ".clone()")?;
                                    }
                                }
                            }
                            if let Expr::Ident(src_name) = rhs.as_ref() {
                                if let Expr::Ident(dst_name) = lhs.as_ref() {
                                    if src_name != dst_name && !self.is_global_var(src_name) {
                                        if let Some(ty) = self.local_var_types.get(src_name) {
                                            if matches!(
                                                ty,
                                                Type::StrFixed(_) | Type::StrSlice | Type::CStrLit
                                            ) {
                                                write!(out, ".to_string()")?;
                                            } else if !matches!(ty, Type::Unknown)
                                                && !Self::is_primitive_copy(ty)
                                            {
                                                write!(out, ".clone()")?;
                                            }
                                        } else if self.current_fn_str_params.contains(src_name) {
                                            // Untyped `str` param rebind — &str at runtime.
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                }
                            }
                            // A `str` param (&str at runtime) assigned INTO a
                            // field of a non-Copy owner is a String write —
                            // `a.ty = callee` needs the conversion. A non-Copy
                            // local ident moved into a field and reused later
                            // (`a.cur_fn = name` … `format!(…, name)`) needs
                            // `.clone()` — the field-READ mirror below covers
                            // the opposite direction only.
                            if let (Expr::Dot(_, _), Expr::Ident(src_name)) =
                                (lhs.as_ref(), rhs.as_ref())
                            {
                                if self.current_fn_str_params.contains(src_name) {
                                    write!(out, ".to_string()")?;
                                } else if let Some(ty) = self.local_var_types.get(src_name) {
                                    if !matches!(ty, Type::Unknown) && !Self::is_primitive_copy(ty)
                                    {
                                        write!(out, ".clone()")?;
                                    }
                                }
                            }
                        }
                    }
                    Op::Eq | Op::Neq => {
                        // Auto char literals ('a') are emitted as i32, string literals stay as strings
                        let op_str = op.op();
                        // Plan 396/B11(a): cast .len() to the partner's type
                        // instead of the default i64 (see partner_len_cast).
                        let rhs_is_len = Self::expr_is_len_call(rhs);
                        let lhs_is_len = Self::expr_is_len_call(lhs);
                        let rhs_cast = if rhs_is_len {
                            self.partner_len_cast(lhs)
                        } else {
                            None
                        };
                        let lhs_cast = if lhs_is_len {
                            self.partner_len_cast(rhs)
                        } else {
                            None
                        };
                        let saved_suppress = self.len_i32_cast_suppressed;
                        if rhs_cast.is_some() || lhs_cast.is_some() {
                            self.len_i32_cast_suppressed = true;
                        }
                        // Plan 433 A1: int vs char-literal mix → cast char side to i64
                        let (char_l, char_r) = self.bina_char_cast(lhs, rhs);
                        // PLAN-026 T-03 配套: 比较一侧为 List<int> 索引、
                        // 另一侧为 i32 语境 —— 索引侧 as i32(两侧同为索引
                        // 时不窄化,i64 域自洽)。
                        let li = self.expr_has_int_list_index(lhs);
                        let ri = self.expr_has_int_list_index(rhs);
                        let narrow_l = li && !ri;
                        let narrow_r = ri && !li;
                        if char_l || narrow_l {
                            write!(out, "(")?;
                        }
                        self.expr(lhs, out)?;
                        if char_l {
                            write!(out, ") as i64")?;
                        } else if narrow_l {
                            write!(out, ") as i32")?;
                        }
                        if let Some(c) = lhs_cast {
                            write!(out, "{}", c)?;
                        }
                        write!(out, " {} ", op_str)?;
                        if char_r || narrow_r {
                            write!(out, "(")?;
                        }
                        self.expr(rhs, out)?;
                        if char_r {
                            write!(out, ") as i64")?;
                        } else if narrow_r {
                            write!(out, ") as i32")?;
                        }
                        if let Some(c) = rhs_cast {
                            write!(out, "{}", c)?;
                        }
                        self.len_i32_cast_suppressed = saved_suppress;
                    }
                    _ => {
                        // Binary operators: lhs OP rhs
                        // Plan 514 W2 (P511-1 根因②): 优先级括号——嵌套二元
                        // 在低优先级算子下方时必须括号化，否则 `(a || b) && c`
                        // 发射为 `a || b && c`（Rust && 更紧）语义反转
                        // （packages/aavm/lib/codegen.at 全局注册条件实证，b27/b29/b30）。
                        // 规则镜像 a2r.at ar_paren_if：lhs.prec < op.prec 加括号，
                        // rhs.prec <= op.prec 加括号。
                        let lhs_paren = matches!(lhs.as_ref(),
                            Expr::Bina(_, lop, _) if Self::auto_op_prec(lop) < Self::auto_op_prec(op));
                        let rhs_paren = matches!(rhs.as_ref(),
                            Expr::Bina(_, rop, _) if Self::auto_op_prec(rop) <= Self::auto_op_prec(op));
                        // Plan 396/B11(a): for comparisons, cast .len() to the
                        // partner's type instead of the default i64 (E0308
                        // when the partner is a u32 var/field).
                        let is_cmp = matches!(op, Op::Lt | Op::Gt | Op::Le | Op::Ge);
                        let rhs_cast = if is_cmp && Self::expr_is_len_call(rhs) {
                            self.partner_len_cast(lhs)
                        } else {
                            None
                        };
                        let lhs_cast = if is_cmp && Self::expr_is_len_call(lhs) {
                            self.partner_len_cast(rhs)
                        } else {
                            None
                        };
                        let saved_suppress = self.len_i32_cast_suppressed;
                        if rhs_cast.is_some() || lhs_cast.is_some() {
                            self.len_i32_cast_suppressed = true;
                        }
                        // Plan 433 A1: int vs char-literal mix → cast char side to i64
                        let (char_l, char_r) = self.bina_char_cast(lhs, rhs);
                        // PLAN-885 G2: `c >= "0"` — owned String vs str literal
                        // (E0277 String<&str). Borrow the String side.
                        let (str_borrow_l, str_borrow_r) = if is_cmp {
                            self.bina_str_literal_borrow(lhs, rhs)
                        } else {
                            (false, false)
                        };
                        if char_l {
                            write!(out, "(")?;
                        }
                        if lhs_paren {
                            write!(out, "(")?;
                        }
                        self.expr(lhs, out)?;
                        if str_borrow_l {
                            write!(out, ".as_str()")?;
                        }
                        if lhs_paren {
                            write!(out, ")")?;
                        }
                        if char_l {
                            write!(out, ") as i64")?;
                        }
                        if let Some(c) = lhs_cast {
                            write!(out, "{}", c)?;
                        }
                        // Plan 072: Convert and/or to Rust's &&/||
                        // Plan 067: Support ?? operator (May system)
                        let op_str = match op {
                            Op::And => "&&",
                            Op::Or => "||",
                            Op::QuestionQuestion => "??",
                            _ => op.op(),
                        };
                        write!(out, " {} ", op_str)?;
                        if char_r {
                            write!(out, "(")?;
                        }
                        if rhs_paren {
                            write!(out, "(")?;
                        }
                        self.expr(rhs, out)?;
                        if str_borrow_r {
                            write!(out, ".as_str()")?;
                        }
                        if rhs_paren {
                            write!(out, ")")?;
                        }
                        if char_r {
                            write!(out, ") as i64")?;
                        }
                        if let Some(c) = rhs_cast {
                            write!(out, "{}", c)?;
                        }
                        self.len_i32_cast_suppressed = saved_suppress;
                    }
                }
                Ok(())
            }

            Expr::Unary(op, expr) => {
                // Plan 052: Unary operators - handle address-of and dereference
                let op_str = match op {
                    Op::Add => "&", // Unary & for address-of
                    Op::Mul => "*", // Unary * for dereference
                    _ => op.op(),
                };
                // Plan 204 Phase 1C: Wrap operand in parens for ! to avoid
                // precedence issues (e.g., !expr <= val should be !(expr <= val))
                if matches!(op, Op::Not) {
                    write!(out, "!(",)?;
                    self.expr(expr, out)?;
                    write!(out, ")")?;
                } else {
                    write!(out, "{}", op_str)?;
                    // Plan 523 H7: 一元符作用于二元操作数需括号——此前
                    // `-(x + y)` 发射 `-x + y` 语义反转（b40/g24：参考 -4，
                    // 产物 -10；auto_op_prec 全体二元 1–21 < 前缀绑定）。
                    let operand_needs_paren = matches!(expr.as_ref(), Expr::Bina(_, _, _));
                    if operand_needs_paren {
                        write!(out, "(")?;
                    }
                    self.expr(expr, out)?;
                    if operand_needs_paren {
                        write!(out, ")")?;
                    }
                }
                Ok(())
            }

            // **Phase 2: Borrow Checking System**
            Expr::View(expr) => {
                // Plan 310 Phase 2: route through emit_borrow so escape tiers
                // (Clone/RcRefCell) generate clone/Rc instead of a raw &.
                self.emit_borrow(expr, false, out)
            }

            Expr::Mut(expr) => self.emit_borrow(expr, true, out),

            Expr::Move(expr) | Expr::Take(expr) => {
                // e.move / e.take -> e (move semantics, default in Rust)
                // Plan 122: .move is preferred, .take is deprecated
                self.expr(expr, out)?;
                Ok(())
            }

            // Collections
            Expr::Array(arr) => {
                write!(out, "vec![")?;
                for (i, elem) in arr.iter().enumerate() {
                    self.expr(elem, out)?;
                    // Plan 016 Phase 4: `vec![target]` MOVES a struct/enum
                    // element out of its binding — a later use is E0382.
                    // Clone aggregate-typed idents (mirrors the conservative
                    // `.clone()` on non-range Index reads below). Str/List
                    // elements keep the historical move behavior.
                    if let Expr::Ident(name) = elem {
                        let agg = self
                            .local_var_types
                            .get(name)
                            .map(|ty| matches!(ty, Type::User(_) | Type::Enum(_)))
                            .unwrap_or(false);
                        if agg {
                            write!(out, ".clone()")?;
                        }
                    }
                    if i < arr.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]").map_err(Into::into)
            }

            Expr::Tuple(elems) => {
                write!(out, "(")?;
                for (i, elem) in elems.iter().enumerate() {
                    self.expr(elem, out)?;
                    if i < elems.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, ")").map_err(Into::into)
            }

            Expr::TupleDestruct { names, expr } => {
                write!(out, "let (")?;
                for (i, name) in names.iter().enumerate() {
                    write!(out, "{}", name)?;
                    if i < names.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, ") = ")?;
                self.expr(expr, out)?;
                Ok(())
            }

            Expr::Index(arr, idx) => {
                // Plan 523 H4: str 非区间下标读 → chars().nth 码点形态
                // （b39 考古：`s[0]` 直发 str 不可整数下标，E0277）。语义
                // 镜像 v2 engine GetElem VStr 臂：nth(j).unwrap_or('\0')
                // as i64；负下标回绕（VM: len+j）不在语料面，登记差异。
                // 区间下标（Range）走原臂（str 切片合法 Rust）。
                if !matches!(idx.as_ref(), Expr::Range(_)) && self.expr_is_string_like(arr) {
                    write!(out, "(")?;
                    self.expr(arr, out)?;
                    write!(out, ".chars().nth(")?;
                    if Self::needs_usize_cast(idx) {
                        write!(out, "(")?;
                        self.expr(idx, out)?;
                        write!(out, ") as usize")?;
                    } else {
                        self.expr(idx, out)?;
                    }
                    write!(out, ").unwrap_or('\\0') as i64).to_string()")?;
                    return Ok(());
                }
                // Plan 577 ② (DEBTS 016 Phase 0): map container dispatch —
                // `m[key]` on a Map receiver indexes by key (`m[&key]`), NOT
                // by usize (the list conversion emitted `m[(k) as usize]`,
                // E0308/E0277). Key borrow mirrors the contains_key arg rules:
                // str literals and &str params index directly; everything else
                // (owned String locals, enum/int keys) takes `&`.
                if !matches!(idx.as_ref(), Expr::Range(_)) && self.recv_is_map(arr) {
                    self.expr(arr, out)?;
                    write!(out, "[")?;
                    let key_already_borrowed = matches!(idx.as_ref(), Expr::Str(_) | Expr::CStr(_))
                        || if let Expr::Ident(n) = idx.as_ref() {
                            self.current_fn_str_params.contains(n)
                        } else {
                            false
                        };
                    if !key_already_borrowed {
                        write!(out, "&")?;
                    }
                    self.expr(idx, out)?;
                    write!(out, "]")?;
                    if self.assign_lhs_depth == 0 {
                        write!(out, ".clone()")?;
                    }
                    return Ok(());
                }
                self.expr(arr, out)?;
                write!(out, "[")?;
                match idx.as_ref() {
                    Expr::Range(range) => {
                        // source[p..p+1] -> source[(p) as usize..(p + 1) as usize]
                        if Self::needs_usize_cast(&range.start) {
                            write!(out, "(")?;
                            self.expr(&range.start, out)?;
                            write!(out, ") as usize")?;
                        } else {
                            self.expr(&range.start, out)?;
                        }
                        write!(out, "{}", if range.eq { "..=" } else { ".." })?;
                        if Self::needs_usize_cast(&range.end) {
                            write!(out, "(")?;
                            self.expr(&range.end, out)?;
                            write!(out, ") as usize")?;
                        } else {
                            self.expr(&range.end, out)?;
                        }
                    }
                    _ => {
                        if Self::needs_usize_cast(idx) {
                            write!(out, "(")?;
                            self.expr(idx, out)?;
                            write!(out, ") as usize")?;
                        } else {
                            self.expr(idx, out)?;
                        }
                    }
                }
                write!(out, "]")?;
                // Non-range index access may move non-Copy types (String, struct);
                // add .clone() to safely handle all element types. C11: on an
                // assignment LHS (in-place element mutation) skip it — writing
                // to the clone would be a no-op.
                if !matches!(idx.as_ref(), Expr::Range(_)) && self.assign_lhs_depth == 0 {
                    write!(out, ".clone()")?;
                }
                Ok(())
            }

            Expr::Range(range) => {
                self.expr(&range.start, out)?;
                if range.eq {
                    write!(out, "..=")?;
                } else {
                    write!(out, "..")?;
                }
                self.expr(&range.end, out).map_err(Into::into)
            }

            Expr::Pair(pair) => {
                // Pair expression: key: value
                let key = match &pair.key {
                    crate::ast::Key::NamedKey(name) => name.clone(),
                    crate::ast::Key::IntKey(n) => format!("{}", n).into(),
                    crate::ast::Key::BoolKey(b) => format!("{}", b).into(),
                    crate::ast::Key::StrKey(s) => s.clone(),
                };
                write!(out, "{}: ", key)?;
                self.expr(&pair.value, out).map_err(Into::into)
            }

            Expr::Object(pairs) => {
                // Object literal: {key1: value1, key2: value2}
                write!(out, "{{")?;
                for (i, pair) in pairs.iter().enumerate() {
                    self.expr(&Expr::Pair(pair.clone()), out)?;
                    if i < pairs.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "}}").map_err(Into::into)
            }

            Expr::Grid(grid) => {
                // Grid expression: 2D array
                // Convert to nested vec: vec![vec![...], ...]
                write!(out, "vec![")?;
                for (i, row) in grid.data.iter().enumerate() {
                    write!(out, "vec![")?;
                    for (j, cell) in row.iter().enumerate() {
                        self.expr(cell, out)?;
                        if j < row.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                    write!(out, "]")?;
                    if i < grid.data.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]").map_err(Into::into)
            }

            Expr::Cover(cover) => {
                // Cover expression for tagged unions
                match cover {
                    crate::ast::Cover::Tag(tag_cover) => {
                        let key = (tag_cover.kind.clone(), tag_cover.tag.clone());
                        let is_struct = self.enum_struct_variants.contains_key(&key);
                        let tuple_arity = self.enum_tuple_variants.get(&key).copied();

                        // Plan 013 (B16): record non-wildcard bindings so a
                        // later call-site auto-clone can deref a Box<T> before
                        // cloning. Only Kid.Node and Atom.Node carry Box<T> in
                        // the auto_val/auto_atom API; other variants (Value.Obj,
                        // Value.Str, ...) hold plain values, so deref'ing them
                        // is an error (E0614). Gate on the known Box variants.
                        let is_box_variant = matches!(
                            (tag_cover.kind.as_str(), tag_cover.tag.as_str()),
                            ("Kid", "Node") | ("Atom", "Node")
                        );
                        if is_box_variant {
                            for b in &tag_cover.bindings {
                                if b != "_" {
                                    self.bridge_pattern_bound_idents.insert(b.clone());
                                }
                            }
                        }

                        // Bare variant check (no bindings): Enum::Variant
                        if tag_cover.bindings.iter().all(|b| b.as_str() == "_") {
                            if let Some(arity) = tuple_arity {
                                // Tuple variant needs (_, _, ...): Enum::Variant(_, _, ...)
                                write!(out, "{}::{}(", tag_cover.kind, tag_cover.tag)?;
                                for j in 0..arity {
                                    if j > 0 {
                                        write!(out, ", ")?;
                                    }
                                    write!(out, "_")?;
                                }
                                write!(out, ")").map_err(Into::into)
                            } else if is_struct {
                                // Struct variant needs { .. }: Enum::Variant { .. }
                                write!(out, "{}::{} {{ .. }}", tag_cover.kind, tag_cover.tag)
                                    .map_err(Into::into)
                            } else {
                                write!(out, "{}::{}", tag_cover.kind, tag_cover.tag)
                                    .map_err(Into::into)
                            }
                        } else if is_struct {
                            // Struct variant: Enum::Variant { field1, field2 }
                            let field_names = self
                                .enum_struct_variants
                                .get(&key)
                                .map(|v| v.as_slice())
                                .unwrap_or(&[]);
                            write!(out, "{}::{} {{ ", tag_cover.kind, tag_cover.tag)?;
                            for (i, binding) in tag_cover
                                .bindings
                                .iter()
                                .filter(|b| b.as_str() != "_")
                                .enumerate()
                            {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                // Use field name if available, otherwise binding name
                                if let Some(field_name) = field_names.get(i) {
                                    if field_name.as_str() == binding.as_str() {
                                        write!(out, "{}", field_name)?;
                                    } else {
                                        write!(out, "{}: {}", field_name, binding)?;
                                    }
                                } else {
                                    write!(out, "{}", binding)?;
                                }
                            }
                            write!(out, " }}").map_err(Into::into)
                        } else {
                            // Tuple variant or unknown: Enum::Variant(a, b)
                            let binding_str = tag_cover
                                .bindings
                                .iter()
                                .filter(|b| b.as_str() != "_")
                                .map(|b| b.as_str())
                                .collect::<Vec<_>>()
                                .join(", ");
                            write!(
                                out,
                                "{}::{}({})",
                                tag_cover.kind, tag_cover.tag, binding_str
                            )
                            .map_err(Into::into)
                        }
                    }
                }
            }

            Expr::Uncover(uncover) => {
                // Tag pattern matching: the binding was already created in the match arm pattern
                // (e.g., Atom::Int(i)), so just emit the binding variable name
                write!(out, "{}", uncover.binding).map_err(Into::into)
            }

            // Plan 120/159: Option/Result uncover (extract inner value)
            Expr::OptionUncover(uncover) => {
                // OptionUncover: extract binding from Some variant
                // e.g., after `is x { Some(val) => ... }`, val is the binding
                write!(out, "{}", uncover.binding).map_err(Into::into)
            }
            Expr::ResultUncover(uncover) => {
                // ResultUncover: extract binding from Ok/Err variant
                write!(out, "{}", uncover.binding).map_err(Into::into)
            }

            // Plan 165: Struct destructuring pattern
            Expr::StructPattern(sc) => {
                match &sc.variant {
                    Some(variant) => {
                        write!(out, "{}::{}", sc.type_name, variant)?;
                    }
                    None => {
                        write!(out, "{}", sc.type_name)?;
                    }
                }
                write!(out, " {{ ")?;
                for (i, fb) in sc.fields.iter().enumerate() {
                    if fb.field == fb.binding {
                        write!(out, "{}", fb.field)?;
                    } else {
                        write!(out, "{}: {}", fb.field, fb.binding)?;
                    }
                    if i < sc.fields.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, " }}").map_err(Into::into)
            }

            // Plan 120/159: Option/Result patterns (used in is statement branches)
            // These are handled in is_stmt, not as standalone expressions.
            // Provide a fallback for cases where they appear as expressions.
            Expr::OptionPattern(cover) => match cover.variant {
                crate::ast::cover::OptionVariant::Some => {
                    if let Some(ref binding) = cover.binding {
                        write!(out, "Some({})", binding).map_err(Into::into)
                    } else {
                        write!(out, "Some(_)").map_err(Into::into)
                    }
                }
                crate::ast::cover::OptionVariant::None => write!(out, "None").map_err(Into::into),
            },
            Expr::ResultPattern(cover) => {
                // Plan 396/B11(b): nested pattern (e.g. Err(Enum.Variant { x }))
                // — emit Ok(<pattern>)/Err(<pattern>) via the inner expr.
                if let Some(ref inner) = cover.inner {
                    let kw = match cover.variant {
                        crate::ast::cover::ResultVariant::Ok => "Ok",
                        crate::ast::cover::ResultVariant::Err => "Err",
                    };
                    write!(out, "{}(", kw)?;
                    self.expr(inner, out)?;
                    write!(out, ")")?;
                    return Ok(());
                }
                match cover.variant {
                    crate::ast::cover::ResultVariant::Ok => {
                        if let Some(ref binding) = cover.binding {
                            write!(out, "Ok({})", binding).map_err(Into::into)
                        } else {
                            write!(out, "Ok(_)").map_err(Into::into)
                        }
                    }
                    crate::ast::cover::ResultVariant::Err => {
                        if let Some(ref binding) = cover.binding {
                            write!(out, "Err({})", binding).map_err(Into::into)
                        } else {
                            write!(out, "Err(_)").map_err(Into::into)
                        }
                    }
                }
            }

            Expr::Ref(name) => {
                // Reference expression: &name
                write!(out, "&{}", name).map_err(Into::into)
            }

            // Struct construction: Point(1, 2) -> Point { x: 1, y: 2 }
            // Special case: loop { body } -> loop { body }
            Expr::Node(node) => {
                if node.name == "not" {
                    write!(out, "!(")?;
                    if !node.id.is_empty() {
                        write!(out, "{}", node.id)?;
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                if node.name == "loop" {
                    write!(out, "loop {{")?;
                    if !node.body.stmts.is_empty() {
                        write!(out, "\n")?;
                        self.indent();

                        for stmt in &node.body.stmts {
                            self.print_indent(out)?;
                            match stmt {
                                Stmt::Expr(expr) => {
                                    self.expr(expr, out)?;
                                    out.write(b";\n")?;
                                }
                                Stmt::Store(store) => {
                                    self.store(store, out)?;
                                    out.write(b";\n")?;
                                }
                                Stmt::Break => {
                                    out.write(b"break;\n")?;
                                }
                                _ => {
                                    // For other statement types, format inline
                                    match stmt {
                                        Stmt::If(if_) => {
                                            // Inline if statement
                                            write!(out, "if ")?;
                                            for (i, branch) in if_.branches.iter().enumerate() {
                                                if i == 0 {
                                                } else {
                                                    write!(out, " else if ")?;
                                                }
                                                self.expr(&branch.cond, out)?;
                                                write!(out, " {{ ")?;
                                                // Multi-statement body
                                                for stmt in branch.body.stmts.iter() {
                                                    match stmt {
                                                        Stmt::Expr(expr) => {
                                                            self.expr(expr, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        Stmt::Break => {
                                                            write!(out, "break; ")?;
                                                        }
                                                        Stmt::Return(ret) => {
                                                            write!(out, "return ")?;
                                                            self.expr(ret, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        Stmt::Store(store) => {
                                                            self.store(store, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                write!(out, "}}")?;
                                            }
                                            if let Some(else_) = &if_.else_ {
                                                write!(out, " else {{ ")?;
                                                for stmt in else_.stmts.iter() {
                                                    match stmt {
                                                        Stmt::Expr(expr) => {
                                                            self.expr(expr, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        Stmt::Break => {
                                                            write!(out, "break; ")?;
                                                        }
                                                        Stmt::Return(ret) => {
                                                            write!(out, "return ")?;
                                                            self.expr(ret, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        Stmt::Store(store) => {
                                                            self.store(store, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                write!(out, "}}")?;
                                            }
                                            write!(out, "\n")?;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        self.dedent();
                        self.print_indent(out)?;
                    }
                    write!(out, "}}")
                } else {
                    // Regular struct construction
                    write!(out, "{} {{", node.name)?;
                    if !node.args.args.is_empty() || !node.body.stmts.is_empty() {
                        write!(out, " ")?;
                    }

                    // Get cached field names for this type (same as struct_init)
                    let field_names = self
                        .struct_fields
                        .get(&node.name)
                        .cloned()
                        .unwrap_or_default();

                    // Get cached field types for .to_string() auto-insertion
                    let field_types = self
                        .struct_field_types
                        .get(&node.name)
                        .cloned()
                        .unwrap_or_default();
                    for (i, arg) in node.args.args.iter().enumerate() {
                        let needs_to_string = match arg {
                            Arg::Pos(expr) => i < field_types.len()
                                && matches!(field_types[i].1, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice)
                                && !matches!(expr, Expr::Str(_) | Expr::CStr(_)),
                            Arg::Pair(key, _) => {
                                field_types.iter()
                                    .find(|(n, _)| *n == *key)
                                    .map(|(_, ty)| matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice))
                                    .unwrap_or(false)
                            }
                            _ => false,
                        };
                        // Plan 016 Phase 4: same Dot-field-move guard as
                        // struct_init — an owned non-Copy field fed by a
                        // Dot-chain read (`parent.kind`, `parent.inlines`)
                        // moves out of the source binding (E0382 on reuse).
                        // Expr::Node is the parser's other struct-literal
                        // form (e.g. as a method-call arg), so the fix must
                        // live here too.
                        let needs_field_clone = !needs_to_string && {
                            let dot_nonborrowed = match arg {
                                Arg::Pos(expr) | Arg::Pair(_, expr) => {
                                    matches!(expr, Expr::Dot(_, _))
                                        && !Self::is_self_dot(expr)
                                        && !(if let Expr::Dot(obj, _) = expr {
                                            matches!(obj.as_ref(), Expr::Ident(n) if self.borrowed_iter_vars.contains(n))
                                        } else { false })
                                }
                                Arg::Name(_) => false,
                            };
                            let field_owned = match arg {
                                Arg::Pos(_) => field_types.get(i).map(|(_, t)| t),
                                Arg::Pair(key, _) => field_types.iter()
                                    .find(|(n, _)| *n == *key)
                                    .map(|(_, t)| t),
                                Arg::Name(_) => None,
                            };
                            dot_nonborrowed && field_owned
                                .map(|t| !Self::is_primitive_copy(t)
                                    && !matches!(t, Type::Fn(_, _) | Type::Void))
                                .unwrap_or(false)
                        };
                        match arg {
                            Arg::Pos(expr) => {
                                let field_name = if i < field_names.len() {
                                    field_names[i].clone()
                                } else {
                                    format!("field{}", i).into()
                                };
                                // Plan 019 Phase 1: keyword-colliding field
                                // names (`type`) need raw-identifier form.
                                write!(out, "{}: ", Self::rust_ident(field_name.as_str()))?;
                                self.write_expr_for_struct_field(expr, out)?;
                            }
                            Arg::Name(name) => {
                                write!(out, "{}: ", Self::rust_ident(name.as_str()))?;
                            }
                            Arg::Pair(key, expr) => {
                                write!(out, "{}: ", Self::rust_ident(key.as_str()))?;
                                self.write_expr_for_struct_field(expr, out)?;
                            }
                        }
                        if needs_to_string {
                            write!(out, ".to_string()")?;
                        } else if needs_field_clone {
                            write!(out, ".clone()")?;
                        }
                        if i < node.args.args.len() - 1 || !node.body.stmts.is_empty() {
                            write!(out, ", ")?;
                        }
                    }

                    // Handle body statements (field initializers)
                    for (i, stmt) in node.body.stmts.iter().enumerate() {
                        let (field_name, field_expr): (AutoStr, &Expr) = match stmt {
                            Stmt::Store(store) => (store.name.clone(), &store.expr),
                            Stmt::Expr(Expr::Pair(pair)) => {
                                let name = match &pair.key {
                                    crate::ast::Key::NamedKey(name) => name.clone(),
                                    crate::ast::Key::IntKey(n) => format!("{}", n).into(),
                                    crate::ast::Key::BoolKey(b) => format!("{}", b).into(),
                                    crate::ast::Key::StrKey(s) => s.clone(),
                                };
                                (name, &pair.value)
                            }
                            _ => {
                                if i < node.body.stmts.len() - 1 {
                                    write!(out, ", ")?;
                                }
                                continue;
                            }
                        };

                        // Check if this field needs .to_string() (String field but &str value)
                        // write_expr_for_struct_field already handles string literals
                        let field_is_string = field_types.iter()
                            .find(|(n, _)| *n == field_name)
                            .map(|(_, ty)| matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice))
                            .unwrap_or(false);
                        let expr_is_str_slice = match field_expr {
                            Expr::Ident(name) => {
                                // Check if variable is &str type (not String)
                                self.local_var_types.get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false)
                            }
                            _ => false,
                        };
                        let needs_to_string = field_is_string && expr_is_str_slice;

                        write!(out, "{}: ", field_name)?;
                        self.write_expr_for_struct_field(field_expr, out)?;
                        if needs_to_string {
                            write!(out, ".to_string()")?;
                        }
                        if i < node.body.stmts.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }

                    if !node.args.args.is_empty() || !node.body.stmts.is_empty() {
                        write!(out, " ")?;
                    }
                    write!(out, "}}")
                }
                .map_err(Into::into)
            }

            // Function calls
            Expr::Call(call) => self.call(call, out),

            // F-strings: f"hello $name" -> format!("hello {}", name)
            Expr::FStr(fstr) => {
                write!(out, "format!(\"")?;
                let mut _arg_count = 0;
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => {
                            let escaped = s
                                .replace("\\", "\\\\")
                                .replace("\"", r##"\""##)
                                .replace("{", "{{")
                                .replace("}", "}}");
                            write!(out, "{}", escaped)?;
                        }
                        Expr::Char(c) => {
                            write!(out, "{}", c)?;
                        }
                        _ => {
                            // Expression placeholder
                            write!(out, "{{}}")?;
                            _arg_count += 1;
                        }
                    }
                }
                write!(out, "\"")?;

                // Add arguments after format string
                for part in &fstr.parts {
                    match part {
                        Expr::Str(_) | Expr::CStr(_) | Expr::Char(_) => {}
                        _ => {
                            write!(out, ", ")?;
                            self.expr(part, out)?;
                        }
                    }
                }

                write!(out, ")").map_err(Into::into)
            }

            // Control flow (stub for now)
            Expr::If(if_) => {
                // Transpile if/else if/else chains
                for (i, branch) in if_.branches.iter().enumerate() {
                    if i == 0 {
                        write!(out, "if ")?;
                    } else {
                        write!(out, " else if ")?;
                    }

                    // Condition
                    self.expr(&branch.cond, out)?;
                    write!(out, " {{")?;

                    // Body
                    if !branch.body.stmts.is_empty() {
                        write!(out, "\n")?;
                        self.indent();
                        let stmt_count = branch.body.stmts.len();
                        for (i, stmt) in branch.body.stmts.iter().enumerate() {
                            self.print_indent(out)?;
                            // Handle different statement types
                            let is_last = i == stmt_count - 1;
                            match stmt {
                                Stmt::Expr(Expr::If(inner_if)) => {
                                    // If expression - don't add semicolon
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Expr(expr) => {
                                    self.expr(expr, out)?;
                                    if is_last
                                        && self.ret_type_needs_string_coercion()
                                        && self.expr_needs_string_coercion(expr)
                                    {
                                        write!(out, ".to_string()")?;
                                    }
                                    if !is_last {
                                        out.write(b";\n")?;
                                    } else {
                                        out.write(b"\n")?;
                                    }
                                }
                                Stmt::If(inner_if) => {
                                    // Nested if statement - handle as expression
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Store(store) => {
                                    self.store(store, out)?;
                                    out.write(b";\n")?;
                                }
                                _ => {
                                    // Other statement types - handle Break, Return, etc.
                                    match stmt {
                                        Stmt::Break => {
                                            out.write(b"break;\n")?;
                                        }
                                        Stmt::Return(ret) => {
                                            out.write(b"return ")?;
                                            self.expr(ret, out)?;
                                            out.write(b";\n")?;
                                        }
                                        _ => {
                                            write!(
                                                out,
                                                "/* unsupported statement in if body */\n"
                                            )?;
                                        }
                                    }
                                }
                            }
                        }
                        self.dedent();
                        self.print_indent(out)?;
                    }
                    write!(out, "}}")?;
                }

                // Else clause
                if let Some(else_body) = &if_.else_ {
                    write!(out, " else {{")?;
                    if !else_body.stmts.is_empty() {
                        write!(out, "\n")?;
                        self.indent();
                        let stmt_count = else_body.stmts.len();
                        for (i, stmt) in else_body.stmts.iter().enumerate() {
                            self.print_indent(out)?;
                            let is_last = i == stmt_count - 1;
                            match stmt {
                                Stmt::Expr(Expr::If(inner_if)) => {
                                    // Nested if expression in else
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Expr(expr) => {
                                    self.expr(expr, out)?;
                                    if is_last
                                        && self.ret_type_needs_string_coercion()
                                        && self.expr_needs_string_coercion(expr)
                                    {
                                        write!(out, ".to_string()")?;
                                    }
                                    if !is_last {
                                        out.write(b";\n")?;
                                    } else {
                                        out.write(b"\n")?;
                                    }
                                }
                                Stmt::If(inner_if) => {
                                    // Nested if statement - handle as expression
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Store(store) => {
                                    self.store(store, out)?;
                                    out.write(b";\n")?;
                                }
                                _ => match stmt {
                                    Stmt::Break => {
                                        out.write(b"break;\n")?;
                                    }
                                    Stmt::Return(ret) => {
                                        out.write(b"return ")?;
                                        self.expr(ret, out)?;
                                        out.write(b";\n")?;
                                    }
                                    _ => {
                                        write!(out, "/* unsupported statement in else body */\n")?;
                                    }
                                },
                            }
                        }
                        self.dedent();
                        self.print_indent(out)?;
                    }
                    write!(out, "}}")?;
                } else {
                    // No else clause
                    write!(out, "")?;
                }

                Ok(())
            }

            // Lambda/closure: |params| body
            Expr::Lambda(lambda) => {
                write!(out, "|")?;
                for (i, param) in lambda.params.iter().enumerate() {
                    write!(out, "{}", self.rust_type_name(&param.ty))?;
                    write!(out, " {}", param.name)?;
                    if i < lambda.params.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "| ")?;

                // Lambda body - if it's a single expression, write it directly
                if lambda.body.stmts.len() == 1 {
                    match &lambda.body.stmts[0] {
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, out)?;
                        }
                        _ => {
                            write!(out, "{{ /* unsupported lambda body */ }}")?;
                        }
                    }
                } else {
                    // Multiple statements - use block
                    // Plan 151 Phase 1.4: Support return statements in closures
                    write!(out, "{{ ")?;
                    for (i, stmt) in lambda.body.stmts.iter().enumerate() {
                        match stmt {
                            Stmt::Expr(expr) => {
                                self.expr(expr, out)?;
                                if i < lambda.body.stmts.len() - 1 {
                                    write!(out, "; ")?;
                                }
                            }
                            Stmt::Store(store) => {
                                self.store(store, out)?;
                                write!(out, "; ")?;
                            }
                            Stmt::Return(ret_expr) => {
                                // Return statement in closure
                                write!(out, "return ")?;
                                self.expr(ret_expr, out)?;
                                write!(out, "; ")?;
                            }
                            _ => {
                                write!(out, "/* unsupported statement */ ")?;
                            }
                        }
                    }
                    write!(out, "}}")?;
                }
                Ok(())
            }

            // Block expression: { stmt1; stmt2; expr }
            Expr::Block(body) => {
                write!(out, "{{ ")?;
                let block_len = body.stmts.len();
                for (bi, stmt) in body.stmts.iter().enumerate() {
                    let block_is_last = bi == block_len - 1;
                    match stmt {
                        // PLAN-010 T2 (009 ledger 25_lifecycle 001/004): the
                        // LAST expr of a block used as a value (closure body)
                        // is the block's VALUE — a trailing `;` types the
                        // closure as returning `()` (E0277 when printed).
                        Stmt::Expr(expr) if !block_is_last => {
                            self.expr(expr, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::Return(ret_expr) => {
                            write!(out, "return ")?;
                            self.expr(ret_expr, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::For(for_stmt) => {
                            self.for_stmt_inline(for_stmt, out)?;
                        }
                        Stmt::EmptyLine(n) => {
                            for _ in 0..*n {
                                write!(out, "\n")?;
                            }
                        }
                        _ => {
                            write!(out, "/* unsupported stmt in block */ ")?;
                        }
                    }
                }
                write!(out, "}}")?;
                Ok(())
            }

            // Closure (Plan 060): (params) => body or param => body
            Expr::Closure(closure) => {
                // Plan 419 §4.5: 写捕获自由变量在闭包内持 Rc 克隆 ——
                // `{ let x = Rc::clone(&x); move |..| .. }`(体内访问由 Ident
                // 改写统一走 borrow/borrow_mut,克隆名与原绑定同名)。
                let captured: Vec<AutoStr> = self
                    .write_capture_names()
                    .into_iter()
                    .filter(|n| Self::expr_references_ident(&closure.body, n))
                    .collect();
                if !captured.is_empty() {
                    write!(out, "{{ ")?;
                    for n in &captured {
                        write!(
                            out,
                            "let {} = std::rc::Rc::clone(&{}); ",
                            Self::rust_ident(n.as_str()),
                            Self::rust_ident(n.as_str())
                        )?;
                    }
                    write!(out, "move ")?;
                }
                // Plan 364 W5 (D4): explicit `move` prefix → `move |..| ..`
                if closure.is_move {
                    write!(out, "move ")?;
                }
                write!(out, "|")?;
                for (i, param) in closure.params.iter().enumerate() {
                    // Name first, then optional type annotation
                    write!(out, "{}", param.name)?;
                    if let Some(ref ty) = param.ty {
                        write!(out, ": {}", self.rust_type_name(ty))?;
                    }
                    if i < closure.params.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "| ")?;

                // Closure body - it's a boxed expression
                self.expr(&closure.body, out)?;
                if !captured.is_empty() {
                    write!(out, " }}")?;
                }
                Ok(())
            }

            // Plan 056: Dot expression for field access
            Expr::Dot(object, field) => {
                // Plan 391 §7 follow-up: a `::`-separated module path (e.g.
                // `std::env::var`) is parsed as a Dot chain (parser.rs Plan 391
                // D4 normalizes `::` to `Dot`). When such a chain is a known
                // use.rust module path, emit it with `::` separators — not `.`.
                // This early-return covers ALL value-position Dot emits (the
                // object of a method call, a bare path in let RHS, etc.), so
                // `std.env` → `std::env` and the whole `std::env::var` renders
                // correctly. Without this, multi-segment paths emitted invalid
                // Rust like `std.env.var(...)`.
                //
                // Only fire when the FULL chain (including this `field`) is a
                // known module path — `obj.field` (a real field access) never
                // matches use.rust, so it falls through unchanged.
                if let Some(path) = Self::dot_chain_path(&Expr::Dot(object.clone(), field.clone()))
                {
                    if self.path_matches_use_rust(&path) {
                        write!(out, "{}", path.replace('.', "::"))?;
                        return Ok(());
                    }
                }
                // **Phase 1.1: Pointer Operators (test: 005_pointer)**
                // Handle @ (address-of) and * (dereference) as special field names
                match field.as_str() {
                    "@" => {
                        // x.@ -> raw pointer to x (address-of operator)
                        // In Rust, we need to cast reference to raw pointer
                        // x as *mut T
                        self.expr(object, out)?;
                        write!(out, " as *mut _")?; // Use _ for type inference
                        return Ok(());
                    }
                    "*" => {
                        // y.* -> *y (dereference operator)
                        // In Rust, we use * for dereference
                        write!(out, "*")?;
                        self.expr(object, out)?;
                        return Ok(());
                    }
                    // **Phase 2: Borrow Checking System**
                    "view" => {
                        // Plan 310 Phase 2: route through emit_borrow.
                        self.emit_borrow(object, false, out)?;
                        return Ok(());
                    }
                    "mut" => {
                        self.emit_borrow(object, true, out)?;
                        return Ok(());
                    }
                    "take" => {
                        // s.take -> s (move semantics, default in Rust)
                        // Just emit the object itself (no additional syntax needed)
                        self.expr(object, out)?;
                        return Ok(());
                    }
                    // Plan 162: Array .ptr -> .as_mut_ptr() for raw pointer access
                    "ptr" => {
                        self.expr(object, out)?;
                        write!(out, ".as_mut_ptr()")?;
                        return Ok(());
                    }
                    _ => {}
                }

                // PLAN-681 T-05（N2/P2）：Value 基座字段访问——base 为局部
                // Value 型（bare List 元素动态桶,P1a/P1b 派生登记）时,字段读
                // 直发 `name["field"]` 索引形。裸 Value 读不加访问器：后续 ==
                // 与 json! 经 serde_json PartialEq/to_value 直通;typed 收口
                // （P4）与形参收口（P5）在消费点按判定叠加降链。
                if let Expr::Ident(base) = object.as_ref() {
                    if self.local_is_json_value(base) {
                        write!(out, "{}[\"{}\"]", Self::rust_ident(base.as_str()), field)?;
                        return Ok(());
                    }
                }

                // Check if this is an enum access or static method: Enum.Value -> Enum::Value
                // Use heuristic: if object is an identifier starting with uppercase or a known module
                // Also handle module.Type.method() where object is a nested Dot chain
                if let Expr::Ident(type_name) = object.as_ref() {
                    let is_type_name = type_name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                        || self.uses.iter().any(|u| {
                            let u_str = u.as_str();
                            u_str == type_name || u_str.ends_with(&format!("::{}", type_name))
                        })
                        || self.module_types.contains_key(type_name.as_str()); // Plan 264
                    if is_type_name {
                        // Type::Variant (enum) or Type::method (static method)
                        // Plan 264: If type_name is a known module, qualify with crate::
                        if self.module_types.contains_key(type_name.as_str()) {
                            if self.merge_mode || type_name.as_str() == self.current_module_name {
                                write!(out, "{}::{}", type_name, field)?;
                            } else {
                                write!(out, "crate::{}::{}", type_name, field)?;
                            }
                        } else {
                            write!(out, "{}::{}", type_name, field)?;
                        }
                        return Ok(());
                    }
                } else if let Expr::Bina(_, Op::Dot, _) = object.as_ref() {
                    // module.Type.method() — the object is a Dot chain, treat as type-like
                    self.expr(object, out)?;
                    write!(out, "::{}", field)?;
                    return Ok(());
                } else if let Expr::Dot(il, inner_field) = object.as_ref() {
                    // module.Type.method() via Expr::Dot variant
                    // Only use :: if the inner field looks like a type (starts with uppercase)
                    // or the leftmost segment is a known module — otherwise it's nested
                    // struct field access like circle.center.x which should use .
                    let inner_is_type = inner_field
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);
                    let leftmost_is_module = if let Expr::Ident(name) = il.as_ref() {
                        self.uses.iter().any(|u| {
                            let u_str = u.as_str();
                            u_str == name.as_str() || u_str.ends_with(&format!("::{}", name))
                        }) || self.module_types.contains_key(name.as_str()) // Plan 264
                    } else {
                        false
                    };
                    if inner_is_type || leftmost_is_module {
                        self.expr(object, out)?;
                        write!(out, "::{}", field)?;
                        return Ok(());
                    }
                }

                // Regular field access: object.field
                // Some AutoLang properties map to Rust method calls
                let is_rust_method = matches!(
                    field.as_str(),
                    "len" | "is_empty" | "capacity" | "count" | "push" | "pop"
                );
                // Auto's list/str length property `.length` (field form, no
                // parens — distinct from the `.length()` method form handled
                // in call()) maps to Rust's `.len()` (2026-08-23 auto-ai
                // report defect 2: previously emitted a bogus `.length`
                // field, E0609 on Vec/String).
                // Plan 019 Phase 1: for STRING receivers Auto's str length
                // is code-unit based — emit chars().count() so positions
                // stay consistent with char_at/slice (byte len would
                // mis-scan any non-ASCII text).
                if field.as_str() == "length" {
                    // Plan 577 ④ (DEBTS 016 Phase 0): the field form now
                    // matches the method form's int-model cast — bare `.len()`
                    // is usize and breaks int contexts (`h.items.length + 1`
                    // E0308). Same wide-int suppression gate as call()
                    // (Plan 391 D1).
                    let cast = !self.len_i32_cast_suppressed;
                    if cast {
                        write!(out, "(")?;
                    }
                    self.expr(object, out)?;
                    if self.expr_is_string_like(object) {
                        write!(out, ".chars().count()")?;
                    } else {
                        write!(out, ".len()")?;
                    }
                    if cast {
                        write!(out, " as i64)")?;
                    }
                    return Ok(());
                }

                // Plan 310 Phase 0.2: Union field read `u.field` → `u.field()`.
                // Rust union fields require `unsafe` to read; we route through
                // the safe accessor methods generated in union_decl (only for
                // Copy-type fields, which is what we emit accessors for).
                let is_union_access = if let Expr::Ident(var_name) = object.as_ref() {
                    if let Some(Type::User(td)) = self.local_var_types.get(var_name) {
                        self.union_types.contains(&td.name)
                    } else {
                        false
                    }
                } else {
                    false
                };
                if is_union_access {
                    self.expr(object, out)?;
                    write!(out, ".{}()", field)?;
                    return Ok(());
                }

                self.expr(object, out)?;
                // Plan 019 Phase 1: field names colliding with Rust keywords
                // (`type`) get raw-identifier escaping.
                write!(out, ".{}", Self::rust_ident(field))?;
                // Plan 433 A1: a known struct with a field of this name (e.g.
                // Tok.len) is a FIELD read — the method-call parens only apply
                // to collection receivers (String/Vec property sugar).
                let object_has_such_field = if let Expr::Ident(var_name) = object.as_ref() {
                    if let Some(Type::User(td)) = self.local_var_types.get(var_name.as_str()) {
                        self.struct_field_types
                            .get(td.name.as_str())
                            .map(|fields| fields.iter().any(|(f, _)| f.as_str() == field.as_str()))
                            .unwrap_or(false)
                    } else {
                        false
                    }
                } else {
                    false
                };
                if is_rust_method && !object_has_such_field {
                    write!(out, "()")?;
                }
                Ok(())
            }

            Expr::NullCoalesce(lhs, rhs) => {
                // Null coalescing: lhs ?? rhs
                // In Rust, this becomes: lhs.unwrap_or(rhs)
                // Plan 019 Phase 1: a struct field of a BORROWED loop var
                // (`for w in nodes` → &WNode) moves out of the shared ref
                // (E0507) — clone the Option before unwrapping.
                let lhs_needs_clone = matches!(lhs.as_ref(), Expr::Dot(obj, _)
                    if matches!(obj.as_ref(), Expr::Ident(n) if self.borrowed_iter_vars.contains(n)));
                // PLAN-010 T2: type-aware lowering. A non-optional receiver can
                // never be nil in Auto — `x ?? d` (x: int) is just `x`
                // (`.unwrap_or` on i64 is E0599). An index receiver lowers to
                // nil-safe `.get(idx)` (Auto indexing is nil-safe; `[]` panics).
                match lhs.as_ref() {
                    Expr::Index(base, idx)
                        if self
                            .expr_type_is_option_like(base)
                            .map(|o| !o)
                            .unwrap_or(false) =>
                    {
                        self.expr(base, out)?;
                        write!(out, ".get((")?;
                        self.expr(idx, out)?;
                        write!(out, ") as usize)")?;
                        write!(out, ".cloned().unwrap_or(")?;
                        self.expr(rhs, out)?;
                        if matches!(rhs.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                            write!(out, ".to_string()")?;
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    Expr::Ident(n)
                        if self.expr_type_is_known_scalar(lhs)
                            && !self.borrowed_iter_vars.contains(n)
                            && !self.by_value_iter_bindings.contains(n) =>
                    {
                        self.expr(lhs, out)?;
                        return Ok(());
                    }
                    _ => {}
                }
                self.expr(lhs, out)?;
                if lhs_needs_clone {
                    write!(out, ".clone()")?;
                }
                write!(out, ".unwrap_or(")?;
                self.expr(rhs, out)?;
                if matches!(rhs.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                    write!(out, ".to_string()")?;
                }
                write!(out, ")")?;
                Ok(())
            }

            Expr::ErrorPropagate(expr) => {
                // Error propagation: expr.?
                // Plan 067: May system support
                // PLAN-010 T2: type-aware lowering. `.?` on a non-optional,
                // non-Result receiver is the value itself (`x.?` with x: int —
                // `x?` in Rust is E0277). An index receiver lowers to nil-safe
                // `.get(idx)` + ok_or (Auto `list[i].?` propagates out-of-bounds
                // as an error; `list[i]?` on a bare index is E0277).
                match expr.as_ref() {
                    Expr::Index(base, idx)
                        if self
                            .expr_type_is_option_like(base)
                            .map(|o| !o)
                            .unwrap_or(false) =>
                    {
                        self.expr(base, out)?;
                        write!(out, ".get((")?;
                        self.expr(idx, out)?;
                        write!(out, ") as usize)")?;
                        write!(out, ".cloned().ok_or(\"index out of bounds\")?")?;
                        return Ok(());
                    }
                    Expr::Ident(n)
                        if self.expr_type_is_known_scalar(expr)
                        // Iter-var bindings may carry Rust-side Result items
                        // (read_dir loops) even when Auto types them as
                        // scalars — their `?` is real propagation.
                        && !self.borrowed_iter_vars.contains(n)
                        && !self.by_value_iter_bindings.contains(n) =>
                    {
                        self.expr(expr, out)?;
                        return Ok(());
                    }
                    _ => {}
                }
                self.expr(expr, out)?;
                write!(out, "?")?;
                Ok(())
            }

            // Plan 162: Type cast: expr.as(Type) -> (expr as Type)
            Expr::Cast { expr, target_type } => {
                write!(out, "(")?;
                self.expr(expr, out)?;
                write!(out, " as {})", self.rust_type_name(target_type))?;
                Ok(())
            }

            // Plan 162: Explicit type conversion: expr.to(Type)
            // Strategy: .to(str) generates .to_string() (always valid);
            // for string literal sources targeting numeric types, generate .parse::<T>().unwrap();
            // for all other numeric targets, degrade to `as` cast (same as .as()).
            // Future: refine based on source type inference.
            Expr::To { expr, target_type } => {
                match target_type {
                    Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => {
                        // x.to(str) / x.to(String) → Display for dep rust-typed
                        // values (PLAN-596 T-07, DIV-DEP-8: three-way parity with
                        // the VM leg); Auto types keep format!("{:?}", x) as the
                        // safe default — existing corpora depend on Debug here.
                        let fmt = if self.receiver_is_dep_rust_value(expr) {
                            "{}"
                        } else {
                            "{:?}"
                        };
                        write!(out, "format!(\"{}\", ", fmt)?;
                        self.expr(expr, out)?;
                        write!(out, ")")?;
                    }
                    // For string literal sources, parse works; for others, use `as`
                    // Heuristic: check if expr is a string literal
                    Type::Int => {
                        if matches!(expr.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                            self.expr(expr, out)?;
                            write!(out, ".parse::<i64>().unwrap()")?;
                        } else if Self::expr_is_len_call(expr) {
                            // Plan 577 ④: the .length/.len forms already carry
                            // the int-model cast (`(x.len() as i64)`) — skip
                            // the wrap (double `as i64` otherwise).
                            self.expr(expr, out)?;
                        } else {
                            write!(out, "(")?;
                            self.expr(expr, out)?;
                            write!(out, " as i64)")?;
                        }
                    }
                    Type::Float | Type::Double => {
                        if matches!(expr.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                            self.expr(expr, out)?;
                            write!(out, ".parse::<f64>().unwrap()")?;
                        } else {
                            write!(out, "(")?;
                            self.expr(expr, out)?;
                            write!(out, " as f64)")?;
                        }
                    }
                    _ => {
                        // Check if target is a string-like type name (String, str, etc.)
                        let ty_name = self.rust_type_name(target_type);
                        if ty_name == "String" || ty_name == "str" || ty_name == "&str" {
                            // x.to(String) / x.to(str) → Display for dep rust-typed
                            // values, Debug fallback otherwise (PLAN-596 T-07, D8).
                            let fmt = if self.receiver_is_dep_rust_value(expr) {
                                "{}"
                            } else {
                                "{:?}"
                            };
                            write!(out, "format!(\"{}\", ", fmt)?;
                            self.expr(expr, out)?;
                            write!(out, ")")?;
                        } else {
                            // Fallback: treat as cast (same as .as())
                            write!(out, "(")?;
                            self.expr(expr, out)?;
                            write!(out, " as {})", ty_name)?;
                        }
                    }
                }
                Ok(())
            }

            // Plan 124: Async/Future/Await system
            Expr::AsyncBlock {
                body,
                return_type: _,
            } => {
                // Plan 310 Phase 3: ~{ stmts } -> async move { stmts }
                // Force move capture: Auto's ownership model defaults async blocks
                // to owning their captured variables, avoiding lifetime issues
                // across await points (the #1 async footgun in Rust).
                //
                // Plan 364 W4 (D5): delegate every statement to the unified
                // `stmt()` entry instead of hand-matching a few variants here.
                // Previously only Expr/Store/Return/Reply were emitted and all
                // other classes (if/for/break/continue/is/...) were silently
                // dropped via `_ => {}` — a correctness bug. `stmt()` writes to
                // a `Sink` while this arm has `out: impl Write`, so we bridge
                // with a throwaway `Sink::dummy()`, draining its body into
                // `out` after each statement. `stmt()`'s catch-all is
                // `_ => Err(...)`, so genuinely-unsupported statements (Try,
                // Block) now fail loudly instead of vanishing silently.
                write!(out, "async move {{ ")?;
                for (i, stmt) in body.stmts.iter().enumerate() {
                    // Fresh sink per statement: stmt() may call sink.record()
                    // internally (e.g. emit_loop_body), which slices
                    // body[record_pos..]; reusing one sink across statements
                    // without resetting record_pos would slice out of bounds.
                    let mut sink = Sink::dummy();
                    self.stmt(stmt, &mut sink)?;
                    out.write_all(&sink.body)?;
                    // stmt()'s Store/Return/Reply/Break/Continue arms already
                    // emit their own trailing ';'. Only Expr omits it (callers
                    // add it), so append ';' just for Expr, then a separator
                    // space between statements.
                    if i + 1 < body.stmts.len() {
                        if matches!(stmt, Stmt::Expr(_)) {
                            write!(out, "; ")?;
                        } else {
                            write!(out, " ")?;
                        }
                    }
                    // PLAN-668 R-12(4): tail Expr keeps NO trailing ';' -- the
                    // block's tail expression IS its value (`let f = ~{a+b}`;
                    // the old unconditional ';' collapsed it to (), gate E0277).
                    // Discarding call sites get their own ';' from stmt layer.
                }
                write!(out, " }}")?;
                Ok(())
            }

            Expr::Await { expr } => {
                // Check if the inner expression is a self-awaited call (like http.post_sync)
                // that already contains .await internally — if so, skip the outer .await
                if let Expr::Call(call) = expr.as_ref() {
                    if let Expr::Dot(obj, method) = call.name.as_ref() {
                        if let Expr::Ident(obj_name) = obj.as_ref() {
                            let m = method.as_str();
                            if obj_name.as_str() == "http"
                                && (m == "post_sync"
                                    || m == "post_bearer"
                                    || m == "post_bearer_sync")
                            {
                                // http.post_sync/post_bearer/post_bearer_sync with .await: generate with .as_str() for str args
                                // PLAN-724 T-06：async 上下文走 *_async 内核面（post_bearer 本就 async）。
                                let func_name = if self.in_async_ctx.get() && m != "post_bearer" {
                                    format!("a2r_std::http::{}_async", m)
                                } else {
                                    format!("a2r_std::http::{}", m)
                                };
                                self.a2r_std_used.set(true);
                                let needs_await = m == "post_bearer" || self.in_async_ctx.get();
                                // PLAN-738 T-10：该臂在 fn expr 内直接发射侧信道壳，
                                // 不经 call() 的验证钩子——先收集发射文本，走同一
                                // strict 门（无证明/漂移均拒绝），再落线。
                                let mut emitted = Vec::new();
                                write!(emitted, "{{ let __resp = {}(", func_name)?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(emitted, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr_as_str(expr, &mut emitted)?;
                                    }
                                }
                                if needs_await {
                                    write!(emitted, ").await")?;
                                } else {
                                    write!(emitted, ")")?;
                                }
                                write!(
                                    emitted,
                                    "; a2r_std::http::set_last_status(__resp.0); __resp.1 }}"
                                )?;
                                let emitted =
                                    String::from_utf8(emitted).map_err(|e| e.to_string())?;
                                if let Some(proof) =
                                    crate::stdlib_assembly::host::verify_rust_reference(
                                        "http",
                                        m,
                                        &emitted,
                                        self.assembly_runtime,
                                    )?
                                {
                                    self.assembly_references.push(proof);
                                }
                                write!(out, "{}", emitted)?;
                                return Ok(());
                            }
                        }
                    }
                }
                // expr.await -> expr.await
                // Plan 417-final: the Call arm auto-inserts `.await` for
                // async callees (Plan 373, ~line 8210). When the awaited
                // expression is exactly such a call, that insertion already
                // satisfies this `.await` — appending a second produced
                // `double(a).await.await` (E0277) and broke the tokio a2r
                // backend. Skip the outer `.await` in that case.
                //
                // 2026-08-23 regression fix (auto-ai report): the skip must
                // ONLY apply to free-fn calls (`Expr::Ident` callee). The
                // auto-insert lives at call()'s tail; every `Expr::Dot`
                // (method) call is intercepted by the generic method-call
                // block (~line 5690) which returns early and never reaches
                // it — so skipping the outer `.await` for Dot calls silently
                // dropped explicit `.await` on `self.run_inner(...).await`
                // etc. (E0308 future). Method calls always keep the explicit
                // `.await`.
                let inner_auto_awaits = match expr.as_ref() {
                    Expr::Call(call) => {
                        if let Expr::Ident(fname) = call.name.as_ref() {
                            self.fn_ret_types
                                .get(fname.as_str())
                                .map(|ret| Self::type_is_async(ret))
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                self.expr(expr, out)?;
                if !inner_auto_awaits {
                    write!(out, ".await")?;
                }
                Ok(())
            }

            // Plan 126: .go postfix operator - fire-and-forget spawn
            // expr.go -> tokio::spawn(async move { expr.await })
            // The expression is spawned as a background task, result is discarded
            Expr::Go { expr } => {
                write!(out, "tokio::spawn(async move {{ ")?;
                let prev_async_ctx = self.in_async_ctx.replace(true);
                self.expr(expr, out)?;
                self.in_async_ctx.set(prev_async_ctx);
                write!(out, ".await; }})")?;
                Ok(())
            }

            // Plan 321: yield expression — in a2r mode, generators use
            // async-stream crate's stream! macro. yield expr → yield expr;
            // Note: the trailing ';' is added by the statement emitter, so we
            // do NOT add it here (doing so produces `yield expr;;` double-semi).
            Expr::Yield(expr) => {
                write!(out, "yield ")?;
                self.expr(expr, out)?;
                Ok(())
            }

            // Plan 223: is as expression → Rust match expression
            Expr::Is(is) => {
                // Plan 016 Phase A A.4: set json_parse_as_opt flag if this `is`
                // expression's scrutinee is json.parse matched against Some/None.
                let parse_as_opt = self.is_json_parse_scrutinee(&is.target)
                    && is.branches.iter().any(|b| matches!(b, crate::ast::IsBranch::EqBranch(patterns, _)
                        if patterns.iter().any(|p| match p {
                            Expr::Ident(n) => n.as_str() == "Some" || n.as_str() == "None",
                            Expr::Call(c) => matches!(c.name.as_ref(), Expr::Ident(n) if n.as_str() == "Some"),
                            Expr::OptionPattern(_) => true,
                            _ => false,
                        })));
                let prev = self.json_parse_as_opt;
                self.json_parse_as_opt = parse_as_opt;

                // Plan 447 H5: Fix double match E0382 — but only in the narrow case
                // where the SAME identifier scrutinee is matched >= 2 times in this
                // function (first by-value match would move it). Single-use idents
                // keep plain `match v` so arm payload bindings stay owned values
                // (and a2r goldens stay stable).
                let needs_ref_match = match &is.target {
                    Expr::Ident(name) => {
                        self.fn_is_scrutinee_counts
                            .get(name.as_str())
                            .copied()
                            .unwrap_or(0)
                            >= 2
                    }
                    _ => false,
                };

                write!(out, "match ")?;
                if needs_ref_match {
                    write!(out, "&")?;
                }
                self.expr(&is.target, out)?;
                self.json_parse_as_opt = prev;
                write!(out, " {{ ")?;
                for (i, branch) in is.branches.iter().enumerate() {
                    if i > 0 {
                        write!(out, " ")?;
                    }
                    match branch {
                        crate::ast::IsBranch::EqBranch(patterns, body) => {
                            let saved_get_refs = self.get_ref_bindings.clone();
                            if Self::is_borrowing_get_scrutinee(&is.target) {
                                self.register_get_ref_bindings(patterns);
                            }
                            // Plan 447 ③-前置: int-typed scrutinee — char
                            // patterns as code-point decimals (pattern list only).
                            let saved_int_scrut = self.int_match_scrutinee;
                            self.int_match_scrutinee = matches!(
                                self.infer_type_from_expr(&is.target),
                                Type::Int | Type::Uint | Type::I64 | Type::U64 | Type::USize
                            );
                            for (j, pat) in patterns.iter().enumerate() {
                                if j > 0 {
                                    write!(out, " | ")?;
                                }
                                self.expr(pat, out)?;
                            }
                            self.int_match_scrutinee = saved_int_scrut;
                            write!(out, " => ")?;
                            self.write_body_inline(body, out)?;
                            write!(out, ",")?;
                            self.get_ref_bindings = saved_get_refs;
                        }
                        crate::ast::IsBranch::IfBranch(cond, body) => {
                            // Plan 447 H4: 卫语句臂应生成 `<绑定> if <cond> =>`
                            // 例如: `x if x > 100 => println!("large")`
                            write!(out, "_guard if ")?;
                            self.expr(cond, out)?;
                            write!(out, " => ")?;
                            self.write_body_inline(body, out)?;
                            write!(out, ",")?;
                        }
                        crate::ast::IsBranch::ElseBranch(body) => {
                            write!(out, "_ => ")?;
                            self.write_body_inline(body, out)?;
                            write!(out, ",")?;
                        }
                    }
                }
                write!(out, " }}")?;
                Ok(())
            }

            // Plan 380 G1: compile-time expression (hash-brace comptime).
            // For a2r, we evaluate the inner expression at transpile time.
            // Supported: read_text("file") and string/int literals.
            Expr::Comptime(hb) => {
                let inner = &hb.expr;
                // Try to evaluate: read_text("path") → read file contents
                if let Expr::Call(call) = inner {
                    if let Expr::Ident(name) = call.name.as_ref() {
                        let fname = name.to_string();
                        if fname == "read_text"
                            || fname == "read_to_string"
                            || fname == "include_str"
                        {
                            if let Some(arg) = call.args.args.first() {
                                let arg_expr = arg.get_expr();
                                if let Expr::Str(path) = arg_expr {
                                    let content = std::fs::read_to_string(path.as_str())
                                        .unwrap_or_else(|e| {
                                            eprintln!("[a2r warning] #{{read_text(\"{}\")}} failed: {}, embedding empty string", path, e);
                                            String::new()
                                        });
                                    // Escape for Rust string literal
                                    write!(
                                        out,
                                        "\"{}\"",
                                        content.replace('\\', "\\\\").replace('"', "\\\"")
                                    )?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                // Fallback: try literal expressions
                match inner {
                    Expr::Str(s) => {
                        let escaped = s.as_str().replace('\\', "\\\\").replace('"', "\\\"");
                        write!(out, "\"{}\"", escaped)?;
                        Ok(())
                    }
                    Expr::Int(i) => { write!(out, "{}", i)?; Ok(()) }
                    Expr::Bool(b) => { write!(out, "{}", b)?; Ok(()) }
                    _ => Err(format!(
                        "Rust Transpiler: #{{}} comptime expression not supported for a2r (only read_text(\"file\") and literals). Got: {}",
                        inner
                    ).into()),
                }
            }

            _ => Err(format!("Rust Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    /// Plan 395: emit `::<T1, T2>` turbofish for a call's explicit generic type
    /// args (`expr.method<Type>(args)` → `expr.method::<Type>(args)`). Types are
    /// rendered through rust_type_name (uint→u32, str→String, User → name, …).
    /// a2r has no bridge signature registry, so it can't know whether the Rust
    /// method is actually generic — `<T>` in .at is an explicit author assertion
    /// and always becomes turbofish (a non-generic callee fails in Rust).
    pub(super) fn emit_turbofish_args(&self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        if !call.generic_args.is_empty() {
            let args_str = call
                .generic_args
                .iter()
                .map(|t| self.rust_type_name(t))
                .collect::<Vec<_>>()
                .join(", ");
            write!(out, "::<{}>", args_str)?;
        }
        Ok(())
    }

    /// 位置实参的发射文本（逗号连接；AST 事实——与发射文本互补，部分
    /// 直发臂丢参时仍可构造等效限定形式）。
    pub(super) fn serialize_positional_args(&mut self, call: &Call) -> String {
        let mut buffer = Vec::new();
        for (index, arg) in call.args.args.iter().enumerate() {
            if index > 0 {
                buffer.extend_from_slice(b", ");
            }
            match arg {
                Arg::Pos(expr) => {
                    if self.expr(expr, &mut buffer).is_err() {
                        return String::new();
                    }
                }
                other => {
                    if self.arg(other, &mut buffer).is_err() {
                        return String::new();
                    }
                }
            }
        }
        String::from_utf8_lossy(&buffer).into_owned()
    }

    /// P738-R5-02：receiver 调用的公共方法归属验证。三种发射形态统一收
    /// 敛到模块公共方法分母（`Owner.method`，含发射适配别名）：
    /// ① 模块限定 `a2r_std::<core>::<callee>(..)`——文本直证；
    /// ② 扁平 helper（`a2r_std::value_len/value_to_int`）——发射器内部
    ///    producer 面，映射到公共方法后以等效限定形式验证；
    /// ③ 接收者为 json 值（json_value_vars 跟踪或类型 User("JsonValue")）
    ///    的直发方法（`v.len()`）——按等效限定形式验证。
    /// 用户同名类型方法不经②③（类型/跟踪集不命中），其无核心 FQN 的发
    /// 射也不经①；无公共面映射的 legacy 变体维持已登记边界（P738-D2）。
    pub(super) fn collect_core_method_reference(
        &mut self,
        object: &Expr,
        method: &str,
        text: &str,
        args_text: &str,
    ) -> AutoResult<()> {
        for module in crate::stdlib_assembly::validate::CORE_MODULES {
            if !text.contains(&format!("a2r_std::{module}::")) {
                continue;
            }
            // 形态守卫：只验证自足的单表达式自由调用。部分发射臂（如
            // `a2r_std::json::len((&` 拆段写）在 call 缓冲内留下不完整
            // 片段、链式调用（`parse(..).len()`）不是自由 Call 形态——
            // 这些形态不构成可独立核对的调用点文本，跳过并记边界（公共
            // 方法证明由完整/构造形态路径承载，见 R5-02 测试）。
            let self_contained = !text.contains(").") && {
                let mut depth: i32 = 0;
                for byte in text.bytes() {
                    match byte {
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        _ => {}
                    }
                    if depth < 0 {
                        break;
                    }
                }
                depth == 0
            };
            if let Some(symbol) = crate::stdlib_assembly::host::public_method_symbol(module, method)
            {
                if self_contained {
                    if let Some(proof) = crate::stdlib_assembly::host::verify_rust_reference(
                        module,
                        &symbol,
                        text,
                        self.assembly_runtime,
                    )? {
                        self.assembly_references.push(proof);
                    }
                } else {
                    // R7-01：公共方法面存在而形态不可核对——诚实拒绝。
                    return Err(format!(
                        "STDASSEMBLY.SIGNATURE_UNVERIFIED: {module}.{symbol}: emission form is not a self-contained call and cannot be verified (fragmented/chained emission)"
                    )
                    .into());
                }
            }
            return Ok(());
        }
        // ② 扁平 value_* helper——发射器内部 producer 面的公共方法适配。
        const FLAT_HELPERS: &[(&str, &str, &str)] = &[
            ("a2r_std::value_len(", "json", "len"),
            ("a2r_std::value_to_int(", "json", "as_int"),
        ];
        for (marker, module, callee) in FLAT_HELPERS {
            if let Some(arg_start) = text.find(marker).map(|i| i + marker.len()) {
                if let Some(inner) = text[arg_start..].strip_suffix(')') {
                    let qualified = format!("a2r_std::{module}::{callee}({inner})");
                    if let Some(symbol) =
                        crate::stdlib_assembly::host::public_method_symbol(module, method)
                    {
                        if let Some(proof) = crate::stdlib_assembly::host::verify_rust_reference(
                            module,
                            &symbol,
                            &qualified,
                            self.assembly_runtime,
                        )? {
                            self.assembly_references.push(proof);
                        }
                    }
                    return Ok(());
                }
            }
        }
        // ③ json 值接收者的直发方法（发射保留 `v.keys()`/`v.len()` 等接收者
        // 形态，无核心 FQN）——按公共方法分母归属（R6-01 泛化：全部公共
        // 方法，不限 len），以等效零参限定形式验证（&Value producer 的
        // 1 参面）。
        if self.receiver_is_json_value(object) {
            if let Some(symbol) = crate::stdlib_assembly::host::public_method_symbol("json", method)
            {
                // 公共方法名→producer callee 反向别名（`type` 是 Rust
                // 关键字，producer 面为 value_type——与 host 别名表对偶）。
                let method_face = match symbol.split('.').next_back().unwrap_or(method) {
                    "type" => "value_type",
                    face => face,
                };
                let name = match object {
                    Expr::Ident(n) => n.to_string(),
                    _ => "receiver".to_string(),
                };
                // R7-03：携带真实实参面（`v.get("k")` → `get(&v, "k")`）。
                let args = args_text;
                let separator = if args.is_empty() { "" } else { ", " };
                if let Some(proof) = crate::stdlib_assembly::host::verify_rust_reference(
                    "json",
                    &symbol,
                    &format!("a2r_std::json::{method_face}(&{name}{separator}{args})"),
                    self.assembly_runtime,
                )? {
                    self.assembly_references.push(proof);
                }
            }
        }
        // ④ 模块方法拼写的未接管发射（`json.is_null(v)` 被通用 fall-through
        // 原样保留、实参丢失——产物本不可编译）：公共方法面存在而该路径
        // 不产生可核对的限定调用——诚实拒绝，不静默放行（R6-01）。
        if let Expr::Ident(owner) = object {
            if matches!(owner.as_str(), "json" | "Json")
                && !self.local_var_types.contains_key(owner)
                && !self.local_modules.contains(owner.as_str())
                && !self.sibling_modules.contains(owner.as_str())
                && crate::stdlib_assembly::host::public_method_symbol("json", method).is_some()
            {
                return Err(format!(
                    "STDASSEMBLY.SIGNATURE_UNVERIFIED: json.{method}: module-qualified method spelling has no verifiable emission (public method face exists; emitter left the call unresolved)"
                )
                .into());
            }
        }
        Ok(())
    }

    /// 接收者是否 json 值：发射器跟踪集（json.get/get_at/parse 赋值）或
    /// 静态类型 User("JsonValue")。与发射臂的 value-helper 判据同源。
    fn receiver_is_json_value(&self, object: &Expr) -> bool {
        if let Expr::Ident(name) = object {
            if self.json_value_vars.contains(name) || self.json_value_bindings.contains(name) {
                return true;
            }
            return matches!(
                self.local_var_types.get(name.as_str()),
                Some(Type::User(decl)) if decl.name.as_str() == "JsonValue"
            );
        }
        false
    }

    pub(super) fn stmt_references_ident(stmt: &Stmt, ident: &AutoStr) -> bool {
        match stmt {
            Stmt::Expr(expr) => Self::expr_references_ident(expr, ident),
            Stmt::Store(store) => {
                store.name == *ident || Self::expr_references_ident(&store.expr, ident)
            }
            Stmt::Return(expr) => Self::expr_references_ident(expr, ident),
            Stmt::If(if_) => {
                if_.branches.iter().any(|b| {
                    Self::expr_references_ident(&b.cond, ident)
                        || b.body
                            .stmts
                            .iter()
                            .any(|s| Self::stmt_references_ident(s, ident))
                }) || if_.else_.as_ref().map_or(false, |e| {
                    e.stmts
                        .iter()
                        .any(|s| Self::stmt_references_ident(s, ident))
                })
            }
            Stmt::Is(is_stmt) => Self::expr_references_ident(&is_stmt.target, ident),
            Stmt::For(for_stmt) => {
                Self::expr_references_ident(&for_stmt.range, ident)
                    || for_stmt
                        .body
                        .stmts
                        .iter()
                        .any(|s| Self::stmt_references_ident(s, ident))
            }
            _ => false,
        }
    }

    fn expr_references_ident(expr: &Expr, ident: &AutoStr) -> bool {
        match expr {
            Expr::Ident(name) => name == ident,
            // Plan 419 §4.5: 闭包体多为块表达式 —— 必须下钻(此前 `_ => false`
            // 令写捕获自由变量检测漏检)。
            Expr::Block(body) => body
                .stmts
                .iter()
                .any(|s| Self::stmt_references_ident(s, ident)),
            Expr::Closure(c) => Self::expr_references_ident(&c.body, ident),
            Expr::Call(call) => {
                Self::expr_references_ident(&call.name, ident)
                    || call.args.args.iter().any(|a| match a {
                        Arg::Pos(e) => Self::expr_references_ident(e, ident),
                        Arg::Pair(_, e) => Self::expr_references_ident(e, ident),
                        Arg::Name(_) => false,
                    })
            }
            Expr::Dot(obj, _) => Self::expr_references_ident(obj, ident),
            Expr::Bina(l, _, r) => {
                Self::expr_references_ident(l, ident) || Self::expr_references_ident(r, ident)
            }
            Expr::Unary(_, e) => Self::expr_references_ident(e, ident),
            Expr::OptionPattern(c) => c.binding.as_ref().map_or(false, |b| b == ident),
            _ => false,
        }
    }

    /// Plan 364 Phase 8 F1: Recursively scan a statement list for any for-loop
    /// whose iterable is a `~Stream<T>` generator call. Such loops get rewritten
    /// to `while let Some(x) = s.next().await`, injecting an `.await` that the
    /// static `has_await_refs` cannot see. Used to force `main` to be async.
    pub(super) fn body_has_stream_for(&self, stmts: &[&Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::For(for_stmt) => {
                    if self.iterable_is_stream(&for_stmt.range) {
                        return true;
                    }
                    // Recurse into the for-loop body.
                    let refs: Vec<&Stmt> = for_stmt.body.stmts.iter().collect();
                    if self.body_has_stream_for(&refs) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        let refs: Vec<&Stmt> = branch.body.stmts.iter().collect();
                        if self.body_has_stream_for(&refs) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        let refs: Vec<&Stmt> = else_body.stmts.iter().collect();
                        if self.body_has_stream_for(&refs) {
                            return true;
                        }
                    }
                }
                Stmt::Block(body) => {
                    let refs: Vec<&Stmt> = body.stmts.iter().collect();
                    if self.body_has_stream_for(&refs) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// 2026-08-23 auto-ai report defect 3: is this receiver a `List<Value>`
    /// local (Value = the serde_json::Value alias used by JSON-typed code)?
    /// Push targets with a Value element type take record literals via
    /// serde_json::json!, not bare struct-literal braces.
    pub(super) fn receiver_elem_is_json_value(&self, object: &Expr) -> bool {
        if let Expr::Ident(name) = object {
            // PLAN-714 r3 R3-T2: .at 裸 `list`（解析为 User("list")）=
            // 动态 Value 桶——记录字面量 push 走 json! 构造（同 bare List
            // Unknown 元素语义；a2r 形=Vec<Value> 别名）。
            if let Some(ty) = self.local_var_types.get(name.as_str()) {
                if matches!(ty, Type::User(td) if td.name.as_str() == "list") {
                    return true;
                }
            }
            if let Some(Type::List(elem)) = self.local_var_types.get(name.as_str()) {
                return match elem.as_ref() {
                    Type::User(td) => td.name.as_str() == "Value",
                    Type::GenericInstance(inst) => inst.base_name.as_str() == "Value",
                    // PLAN-681 T-05（N2）：bare List = 动态桶（元素 Value，
                    // rust_type_name 缺省翻转同源）——记录字面量 push 走
                    // json! 构造。
                    Type::Unknown => true,
                    _ => false,
                };
            }
        }
        false
    }

    /// PLAN-681 T-05（N2）：局部是否 Value 型——bare List 元素的动态记录桶
    /// （User("Value")/GenericInstance("Value") 占位型,经 P1a/P1b 派生登记
    /// 或既有 Value 语境登记）。字段读（P2）与收口判定（P4/P5）按此触发。
    fn local_is_json_value(&self, name: &AutoStr) -> bool {
        match self.local_var_types.get(name.as_str()) {
            Some(Type::User(td)) => td.name.as_str() == "Value",
            Some(Type::GenericInstance(g)) => g.base_name.as_str() == "Value",
            _ => false,
        }
    }

    /// PLAN-681 T-05（N2）：表达式是否 Value 读——Value 型局部的字段读
    /// （nd.field），或 bare List（元素 Value 桶，含 Unknown 元素）的索引
    /// 读（stack[i]），含 .clone() 链。typed let 收口（P4）与调用实参收口
    /// （P5）按此判定叠加访问器。
    pub(super) fn expr_is_json_value_read(&self, e: &Expr) -> bool {
        match e {
            Expr::Dot(obj, _) => {
                matches!(obj.as_ref(), Expr::Ident(n) if self.local_is_json_value(n))
            }
            Expr::Index(base, _) => {
                matches!(base.as_ref(), Expr::Ident(n)
                    if self.receiver_elem_is_json_value(&Expr::Ident(n.clone())))
            }
            Expr::Call(call) => {
                if let Expr::Dot(obj, m) = call.name.as_ref() {
                    m.as_str() == "clone" && self.expr_is_json_value_read(obj)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// PLAN-681 T-05（N2/P3）：Value 桶 push 实参是否需 json! 包裹——
    /// 字面量（str/int/float/bool/记录/数组）与已知非 Value 型表达式
    /// （String 局部、int 算术、Vec 局部等）包裹成 Value；Value 型读
    /// （Value 局部字段读/bare List 索引读,推断为 Value）与 Unknown
    /// （可能是 Value 读,如动态字段链）保持直发。
    pub(super) fn value_push_arg_needs_json_wrap(&self, e: &Expr) -> bool {
        match e {
            Expr::Object(_) | Expr::Array(_) => true,
            Expr::Str(_)
            | Expr::CStr(_)
            | Expr::Int(_)
            | Expr::I64(_)
            | Expr::Uint(_)
            | Expr::U64(_)
            | Expr::Float(_, _)
            | Expr::Double(_, _)
            | Expr::Bool(_) => true,
            _ => match self.infer_type_from_expr(e) {
                Type::Unknown => false,
                Type::User(td) if td.name.as_str() == "Value" => false,
                Type::GenericInstance(g) if g.base_name.as_str() == "Value" => false,
                _ => true,
            },
        }
    }

    /// 2026-08-23 auto-ai report defect 3: emit an object/array record literal
    /// as a `serde_json::json!(...)` invocation. Keys are quoted; nested
    /// literals recurse; any other expression (a Value-typed ident, a call
    /// returning Value, ...) is embedded as-is — json! accepts arbitrary
    /// Serialize values.
    pub(super) fn emit_json_macro(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        write!(out, "serde_json::json!(")?;
        self.emit_json_body(expr, out)?;
        write!(out, ")")?;
        Ok(())
    }

    fn emit_json_body(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            Expr::Object(pairs) => {
                write!(out, "{{")?;
                for (i, pair) in pairs.iter().enumerate() {
                    let key = match &pair.key {
                        crate::ast::Key::NamedKey(name) => name.to_string(),
                        crate::ast::Key::IntKey(n) => format!("{}", n),
                        crate::ast::Key::BoolKey(b) => format!("{}", b),
                        crate::ast::Key::StrKey(s) => s.to_string(),
                    };
                    write!(out, "\"{}\": ", key)?;
                    self.emit_json_body(&pair.value, out)?;
                    if i < pairs.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "}}")?;
            }
            Expr::Array(elems) => {
                write!(out, "[")?;
                for (i, e) in elems.iter().enumerate() {
                    self.emit_json_body(e, out)?;
                    if i < elems.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]")?;
            }
            _ => self.expr(expr, out)?,
        }
        Ok(())
    }

    /// Check if an arg is a string literal ("...") — doesn't need & at call site
    pub(super) fn is_string_literal_arg(arg: &Arg) -> bool {
        if let Arg::Pos(expr) = arg {
            matches!(expr, Expr::Str(_) | Expr::CStr(_))
        } else {
            false
        }
    }

    /// Plan 384: does this return type (possibly wrapped in Future/async)
    /// boil down to Result<String, ...>? (PLAN-010 T2: hoisted from the
    /// Expr::Ok emission arm for reuse by the Ok(str-param) coercion.)
    fn ret_is_result_string(ty: &Type) -> bool {
        match ty {
            Type::Result(inner) => matches!(
                inner.as_ref(),
                Type::StrSlice | Type::StrOwned | Type::StrFixed(_)
            ),
            Type::GenericInstance(inst) => {
                if inst.base_name == "Result" {
                    inst.args
                        .first()
                        .map(|i| matches!(i, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                        .unwrap_or(false)
                } else {
                    // Future<Result<String,_>> / other wrapper: recurse into first arg
                    inst.args
                        .first()
                        .map(Self::ret_is_result_string)
                        .unwrap_or(false)
                }
            }
            _ => false,
        }
    }

    /// Check if an arg is a &str variable — already borrowed, no .as_str() needed.
    /// A variable is truly &str in Rust only if it's a function parameter declared as `str`
    /// (which maps to `mut x: &str` in Rust). Local variables typed `str` map to `String`.
    pub(super) fn is_str_slice_var(&self, arg: &Arg) -> bool {
        if let Arg::Pos(Expr::Ident(name)) = arg {
            // Function params declared as `str` are truly `&str` in Rust.
            // Plan 396 §2.4 / Plan 427: `Some(x)` is-arm bindings registered
            // at the is-arm emitters (is_str_returning_scrutinee) are ALSO
            // `&str` — appending `.as_str()` on them is E0658 (str_as_str,
            // unstable). §2.4 keyed this off local_var_types' StrSlice
            // entries, which also matched plain `let x str` locals (owned
            // Strings in Rust) and suppressed the call-site `.as_str()` →
            // E0308 (Plan 427); the dedicated set keeps §2.4's scope.
            self.current_fn_str_params.contains(name)
                || self.str_slice_pattern_bindings.contains(name)
        } else {
            false
        }
    }

    /// Check if an arg is an integer-typed variable (i32/u32/usize)
    pub(super) fn is_int_var(arg: &Arg, local_var_types: &HashMap<AutoStr, Type>) -> bool {
        match arg {
            Arg::Pos(Expr::Ident(name)) => local_var_types
                .get(name)
                .map(|ty| matches!(ty, Type::Int | Type::Uint))
                .unwrap_or(false),
            Arg::Pos(Expr::Dot(obj, field)) => {
                // self.uint_field → check if it's a known uint struct field
                if let Expr::Ident(_) = obj.as_ref() {
                    // Check struct_field_types for this field
                    // Heuristic: if field name ends with common integer suffixes
                    let fname = field.as_str();
                    fname == "current_step"
                        || fname == "cumulative_tokens"
                        || fname == "step_count"
                        || fname == "run_id"
                        || fname.ends_with("_count")
                        || fname.ends_with("_index")
                        || fname.ends_with("_idx")
                        || fname.ends_with("_id")
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if an expression likely needs .as_str() to convert String → &str.
    /// Returns true for Expr::Ident variables that may be String at runtime.
    pub(super) fn needs_as_str(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => {
                if self.current_fn_str_params.contains(name) {
                    return false;
                }
                // C9 codegen (Plan 018): PathBuf/Path-typed locals (e.g.
                // `let page_path PathBuf = self.wiki_dir.join(...)`) have no
                // `.as_str()` (E0599) — fs.* accept AsRef<Path> directly.
                if let Some(ty) = self.local_var_types.get(name) {
                    if let Type::User(decl) = ty {
                        let n = decl.name.as_str();
                        if n == "PathBuf"
                            || n == "Path"
                            || n == "std::path::PathBuf"
                            || n == "std::path::Path"
                        {
                            return false;
                        }
                    }
                }
                true
            }
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => false,
            Expr::Int(_) | Expr::Float(_, _) => false,
            // Plan 380: char literals are valid str Patterns — never .as_str()
            // (char has no as_str — E0599).
            Expr::Char(_) => false,
            // Plan 368 R-AREG: Dot access that returns &str
            Expr::Dot(_, field) => {
                let f = field.as_str();
                f == "trim"
                    || f == "trim_start"
                    || f == "trim_end"
                    || f == "trim_matches"
                    || f == "as_str"
            }
            // Plan 368 R-AREG: Method calls that return &str don't need .as_str()
            // (calling .as_str() on &str triggers E0658 unstable str_as_str).
            Expr::Call(call) => {
                // Extract method name from Dot(obj, method) — get_name_text_safe
                // only handles Ident, not Dot.
                let method_name: Option<&str> = match call.name.as_ref() {
                    Expr::Dot(_, method) => Some(method.as_str()),
                    Expr::Ident(name) => Some(name.as_str()),
                    _ => None,
                };
                if let Some(m) = method_name {
                    // trim* methods and as_str return &str in Rust
                    if m == "trim"
                        || m == "trim_start"
                        || m == "trim_end"
                        || m == "trim_matches"
                        || m == "trim_start_matches"
                        || m == "trim_end_matches"
                        || m == "as_str"
                    {
                        return false;
                    }
                    // C9 codegen (Plan 018): `join` on a PathBuf returns a
                    // PathBuf, which has no `.as_str()` (E0599) — but
                    // `fs.read_to_string(path)` accepts `AsRef<Path>` directly.
                    // Without this guard the emitted
                    // `self.wiki_dir.join("_manifest.json").as_str()` fails to
                    // compile. (No other module uses `.join`, so this is safe.)
                    if m == "join" {
                        return false;
                    }
                }
                true
            }
            _ => true,
        }
    }

    /// Plan 447 ③-前置: `x.char_at(i)` emits with a trailing `as i64` cast —
    /// a method chained on it must parenthesize the receiver
    /// (`(... as i64).to_string()`; bare `... as i64.to_string()` is a parse
    /// error, "cast cannot be followed by a method call").
    pub(super) fn is_char_at_call(expr: &Expr) -> bool {
        matches!(expr, Expr::Call(c) if matches!(c.name.as_ref(),
            Expr::Dot(_, m) if m.as_str() == "char_at"))
    }

    /// True when `expr` is a call to a str-returning trim-like method
    /// (trim/trim_start/trim_end/trim_matches) or an explicit .as_str().
    pub(super) fn is_trim_method_call(expr: &Expr) -> bool {
        if let Expr::Call(call) = expr {
            if let Expr::Dot(_, m) = call.name.as_ref() {
                let f = m.as_str();
                return f == "trim"
                    || f == "trim_start"
                    || f == "trim_end"
                    || f == "trim_matches"
                    || f == "as_str";
            }
        }
        false
    }

    /// Emit an expression with .as_str() appended if needed for &str parameter.
    pub(super) fn expr_as_str(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        // Plan 368 R-AREG: For trim* methods, expr() appends .to_string() which
        // makes it String. But we need &str here, so render to a temp buffer,
        // strip the .to_string() suffix, and write the base (which is already &str).
        let is_trim_method = Self::is_trim_method_call(expr);
        if is_trim_method {
            let mut buf: Vec<u8> = Vec::new();
            self.expr(expr, &mut buf)?;
            let mut buf_str = String::from_utf8_lossy(&buf).to_string();
            // Strip the .to_string() that the trim handler appends
            if buf_str.ends_with(".to_string()") {
                buf_str.truncate(buf_str.len() - ".to_string()".len());
            }
            write!(out, "{}", buf_str)?;
            return Ok(()); // trim() returns &str, no .as_str() needed
        }
        self.expr(expr, out)?;
        if self.needs_as_str(expr) {
            write!(out, ".as_str()")?;
        }
        Ok(())
    }

    /// PLAN-010 T2: Auto `{}` as a Map/List field initializer means "empty
    /// collection". Rendered verbatim it becomes a unit-struct literal
    /// (`env: {}`), which is E0308 against a HashMap/Vec field type. Emit the
    /// typed constructor instead. Returns false when `expr` is not an empty
    /// literal or the field type is unknown/non-collection (caller falls back
    /// to the normal value path).
    fn write_collection_default_for_empty_node(
        expr: &Expr,
        field_ty: Option<&Type>,
        out: &mut impl Write,
    ) -> AutoResult<bool> {
        let is_empty_node = matches!(expr, Expr::Node(n)
            if n.args.args.is_empty() && n.body.stmts.is_empty())
            || matches!(expr, Expr::Object(pairs) if pairs.is_empty());
        if !is_empty_node {
            return Ok(false);
        }
        let Some(ty) = field_ty else {
            return Ok(false);
        };
        match ty {
            Type::Map(..) => write!(out, "std::collections::HashMap::new()")?,
            Type::List(_) => write!(out, "Vec::new()")?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(super) fn struct_init(
        &mut self,
        type_name: &AutoStr,
        args: &Args,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        // Plan 380 P0: tuple-struct / newtype positional construction.
        // When ALL args are positional (Arg::Pos) AND the type has no known
        // named fields (external/newtype types like axum::Json, Option::Some,
        // Result::Ok), emit positional construction `Type(a, b, ...)` instead
        // of the named-field form `Type { field0: a, ... }` (which fails to
        // compile — E0560 — because tuple structs have no named fields).
        // Plan 014: skip when args is EMPTY — a bare `Type()` for a unit /
        // empty-member struct must stay `Type {}` (E0423 otherwise). bd4c475e
        // broke this: `Assistant()` (empty members, not in struct_fields) got
        // `Assistant()`, which also broke fix_spec_trait_boxing's
        // `Some(X {})` regex, so builtin_roles regeneration failed to compile.
        let has_named_field = args
            .args
            .iter()
            .any(|a| matches!(a, Arg::Name(_) | Arg::Pair(_, _)));
        let known_fields = self.struct_fields.get(type_name.as_str()).is_some();
        if !has_named_field && !known_fields && !args.args.is_empty() {
            write!(out, "{}(", type_name)?;
            for (i, arg) in args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ")?;
                }
                if let Arg::Pos(expr) = arg {
                    self.expr(expr, out)?;
                }
            }
            write!(out, ")")?;
            return Ok(());
        }

        // Generate struct initialization: Type { field1: value1, field2: value2 }
        if args.args.is_empty() {
            // PLAN-010 T2 (009 ledger 13_delegation 族): a struct with
            // delegation component fields (`has core WarpDrive for Engine`)
            // must default-initialize them in every literal — `Starship {}`
            // alone is E0063 (missing field `core`).
            if let Some(deleg) = self.struct_delegation_types.get(type_name) {
                if !deleg.is_empty() {
                    write!(out, "{} {{ ", type_name)?;
                    for (i, (fname, fty)) in deleg.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ")?;
                        }
                        write!(out, "{}: {} {{}}", fname, self.rust_type_name(fty))?;
                    }
                    write!(out, " }}")?;
                    return Ok(());
                }
            }
            // Empty struct: Type {}
            write!(out, "{} {{}}", type_name)?;
            return Ok(());
        }

        write!(out, "{} {{ ", type_name)?;

        // Get cached field names for this type
        let field_names = self
            .struct_fields
            .get(type_name)
            .cloned()
            .unwrap_or_default();

        // Get cached field types for .to_string() auto-insertion
        let field_types = self
            .struct_field_types
            .get(type_name)
            .cloned()
            .unwrap_or_default();

        for (i, arg) in args.args.iter().enumerate() {
            let (field_name, needs_to_string) = match arg {
                Arg::Pos(expr) => {
                    let name = if i < field_names.len() {
                        field_names[i].clone()
                    } else {
                        format!("field{}", i).into()
                    };
                    // Check if field type is String but expr is &str
                    let needs_ts = i < field_types.len()
                        && matches!(
                            field_types[i].1,
                            Type::StrOwned | Type::StrFixed(_) | Type::StrSlice
                        )
                        && !matches!(expr, Expr::Str(_) | Expr::CStr(_));
                    (name, needs_ts)
                }
                Arg::Name(name) => {
                    // Plan 399 Phase 11.2: shorthand `Type { field }` means
                    // `field: field` (same-name local). If the field is a String
                    // type, the local must be .to_string()'d (a2r fn params are
                    // &str). Mirrors the Arg::Pair logic below.
                    let needs_ts = field_types
                        .iter()
                        .find(|(n, _)| n == name)
                        .map(|(_, ty)| {
                            matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice)
                        })
                        .unwrap_or(false);
                    (name.clone(), needs_ts)
                }
                Arg::Pair(key, expr) => {
                    let needs_ts = field_types
                        .iter()
                        .find(|(n, _)| n == key)
                        .map(|(_, ty)| {
                            matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice)
                        })
                        .unwrap_or(false)
                        && !matches!(expr, Expr::Str(_) | Expr::CStr(_));
                    (key.clone(), needs_ts)
                }
            };
            // Plan 016 Phase 4: a Dot-chain field read (`a.pos`,
            // `a.sel.anchor`) as the value of an OWNED non-Copy struct field
            // (String/Vec/struct/enum) MOVES out of the source binding — the
            // next use of the same path is E0382. Clone it. Skipped when
            // needs_to_string fired (that already produces an owned copy),
            // for `self.field` / borrowed-iter-var roots (the inner
            // write_expr_for_struct_field already clones those), and for
            // shorthand `Type { field }` idents.
            let needs_field_clone = !needs_to_string
                && {
                    let dot_nonborrowed = match arg {
                        Arg::Pos(expr) | Arg::Pair(_, expr) => {
                            matches!(expr, Expr::Dot(_, _))
                                && !Self::is_self_dot(expr)
                                && !(if let Expr::Dot(obj, _) = expr {
                                    matches!(obj.as_ref(), Expr::Ident(n) if self.borrowed_iter_vars.contains(n))
                                } else {
                                    false
                                })
                        }
                        Arg::Name(_) => false,
                    };
                    let field_owned = match arg {
                        Arg::Pos(_) => field_types.get(i).map(|(_, t)| t),
                        Arg::Pair(key, _) => {
                            field_types.iter().find(|(n, _)| n == key).map(|(_, t)| t)
                        }
                        Arg::Name(_) => None,
                    };
                    dot_nonborrowed
                        && field_owned
                            .map(|t| {
                                !Self::is_primitive_copy(t)
                                    && !matches!(t, Type::Fn(_, _) | Type::Void)
                            })
                            .unwrap_or(false)
                };
            // Plan 019 Phase 1: keyword-colliding field names (`type`) get
            // raw-identifier escaping in construction literals.
            write!(out, "{}: ", Self::rust_ident(field_name.as_str()))?;
            match arg {
                Arg::Pos(expr) | Arg::Pair(_, expr) => {
                    // PLAN-010 T2 (009 ledger 031/008/009): Auto `{}` as a
                    // Map/List field initializer means "empty collection".
                    // Rendered verbatim it's a unit-struct literal (`env: {}`)
                    // — E0308 against the HashMap/Vec field type.
                    let empty_node_field_ty = match arg {
                        Arg::Pos(_) => field_types.get(i).map(|(_, t)| t.clone()),
                        Arg::Pair(k, _) => field_types
                            .iter()
                            .find(|(n, _)| n == k)
                            .map(|(_, t)| t.clone()),
                        Arg::Name(_) => None,
                    };
                    if !Self::write_collection_default_for_empty_node(
                        expr,
                        empty_node_field_ty.as_ref(),
                        out,
                    )? {
                        self.write_expr_for_struct_field(expr, out)?;
                    }
                }
                Arg::Name(name) => {
                    // Plan 399 Phase 11.2: shorthand `Type { field }` → `field: field`.
                    write!(out, "{}", Self::rust_ident(name.as_str()))?;
                }
            }
            if needs_to_string {
                write!(out, ".to_string()")?;
            } else if needs_field_clone {
                write!(out, ".clone()")?;
            }
            // Plan 032 G4 (auto-ai consumer): a `uint` (u32) struct field fed
            // an i64-typed expr (int literal, i64 local, or a2r-std
            // `json.as_int(...)` — the canonical i64 source) needs the
            // narrowing cast. Consumers sed-patched 13 such Usage-token sites.
            if let Some(expr) = match arg {
                Arg::Pos(e) | Arg::Pair(_, e) => Some(e),
                Arg::Name(_) => None,
            } {
                let field_ty = match arg {
                    Arg::Pos(_) => field_types.get(i).map(|(_, t)| t.clone()),
                    Arg::Pair(k, _) => field_types
                        .iter()
                        .find(|(n, _)| n == k)
                        .map(|(_, t)| t.clone()),
                    Arg::Name(_) => None,
                };
                if matches!(field_ty, Some(Type::Uint)) {
                    let expr_is_i64 = match expr {
                        Expr::Uint(_) => false, // already u32
                        Expr::Int(_) | Expr::I64(_) => true,
                        // a2r-std json.as_int returns i64
                        Expr::Call(c) => {
                            matches!(c.name.as_ref(),
                                Expr::Dot(obj, m)
                                    if m.as_str() == "as_int"
                                        && matches!(obj.as_ref(), Expr::Ident(o) if o.as_str() == "json"))
                        }
                        Expr::Ident(n) => {
                            matches!(self.local_var_types.get(n.as_str()), Some(Type::Int))
                        }
                        _ => false,
                    };
                    if expr_is_i64 {
                        write!(out, " as u32")?;
                    }
                }
            }
            if i < args.args.len() - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, " }}").map_err(Into::into)
    }

    pub(super) fn arg(&mut self, arg: &Arg, out: &mut impl Write) -> AutoResult<()> {
        match arg {
            Arg::Pos(expr) => {
                self.expr(expr, out)?;
                if Self::is_self_dot(expr) {
                    write!(out, ".clone()")?;
                }
                self.maybe_clone_borrowed_iter_field(expr, out)?;
                Ok(())
            }
            Arg::Name(name) => write!(out, "{}", name).map_err(Into::into),
            Arg::Pair(_, expr) => {
                self.expr(expr, out)?;
                if Self::is_self_dot(expr) {
                    write!(out, ".clone()")?;
                }
                self.maybe_clone_borrowed_iter_field(expr, out)?;
                Ok(())
            }
        }
    }

    fn write_expr_for_struct_field(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        // Array literals in struct fields should be vec![] (fields are typically Vec<T>)
        if let Expr::Array(elems) = expr {
            write!(out, "vec![")?;
            for (i, elem) in elems.iter().enumerate() {
                self.expr(elem, out)?;
                // Plan 016 Phase 4: `vec![target]` MOVES a struct/enum
                // element out of its binding — a later use is E0382. Clone
                // aggregate-typed idents (mirrors the Expr::Array arm in
                // expr(), which this field-specific path bypasses).
                if let Expr::Ident(name) = elem {
                    let agg = self
                        .local_var_types
                        .get(name)
                        .map(|ty| matches!(ty, Type::User(_) | Type::Enum(_)))
                        .unwrap_or(false);
                    if agg {
                        write!(out, ".clone()")?;
                    }
                }
                if i < elems.len() - 1 {
                    write!(out, ", ")?;
                }
            }
            write!(out, "]")?;
        } else {
            self.expr(expr, out)?;
            // Auto str literals are &str but struct fields are String
            if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                write!(out, ".to_string()")?;
            }
        }
        // self.field in &self context needs .clone()
        if Self::is_self_dot(expr) {
            write!(out, ".clone()")?;
        }
        // Plan 399 Phase 11.4: `iter_var.field` where iter_var is a borrowed
        // loop variable (&T) — moving a non-Copy field out of a shared ref is
        // E0507. Add .clone() (conservative: also clones Copy fields like id,
        // which is harmless). Matches the `obj.field` non-Copy pattern in store.
        if let Expr::Dot(obj, _field) = expr {
            if let Expr::Ident(obj_name) = obj.as_ref() {
                if self.borrowed_iter_vars.contains(obj_name) {
                    write!(out, ".clone()")?;
                }
            }
        }
        Ok(())
    }

    /// PLAN-026 T-03 配套: List<int> 变量判定(db 层 List<int> 惯例发射
    /// Vec<i64>,元素 i64;进入 i32 语境需 as i32 窄化)。
    fn expr_is_int_list_var(&self, e: &Expr) -> bool {
        if let Expr::Ident(n) = e {
            matches!(self.local_var_types.get(n.as_str()),
                Some(Type::List(inner)) if matches!(**inner, Type::Int))
        } else {
            false
        }
    }

    /// 表达式树含 List<int> 变量的索引消费(Bina 递归)。
    pub(super) fn expr_has_int_list_index(&self, e: &Expr) -> bool {
        match e {
            Expr::Index(base, _) => self.expr_is_int_list_var(base),
            Expr::Bina(l, _, r) => {
                self.expr_has_int_list_index(l) || self.expr_has_int_list_index(r)
            }
            _ => false,
        }
    }

    pub(super) fn expr_is_len_call(expr: &Expr) -> bool {
        if let Expr::Call(call) = expr {
            return match call.name.as_ref() {
                Expr::Dot(_, m) => matches!(m.as_str(), "len" | "length"),
                Expr::Bina(_, op, rhs) if matches!(op, Op::Dot) => {
                    matches!(rhs.as_ref(), Expr::Ident(m) if matches!(m.as_str(), "len" | "length"))
                }
                _ => false,
            };
        }
        // Plan 577 ④: the FIELD form `x.length` (no parens) carries the
        // int-model cast now too — recognized so `.to(int)` skips its wrap
        // and the wide-int binding suppression covers both forms alike.
        match expr {
            Expr::Dot(_, m) => matches!(m.as_str(), "len" | "length"),
            Expr::Bina(_, op, rhs) if matches!(op, Op::Dot) => {
                matches!(rhs.as_ref(), Expr::Ident(m) if matches!(m.as_str(), "len" | "length"))
            }
            _ => false,
        }
    }
}
