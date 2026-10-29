//! PLAN-884 T-06：mutates-self/await/Result/dep-crate AST 扫描族
//!（自 rust.rs 原样迁入，方法体逐字保持）。
use super::*;

impl RustTrans {
    pub(super) fn method_mutates_self(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_mutates_self(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_mutates_self(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_mutates_self(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    if Self::method_mutates_self(&body.stmts) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::method_mutates_self(&branch.body.stmts) {
                            return true;
                        }
                    }
                    if let Some(els) = &if_stmt.else_ {
                        if Self::method_mutates_self(&els.stmts) {
                            return true;
                        }
                    }
                }
                Stmt::For(for_stmt) => {
                    if Self::method_mutates_self(&for_stmt.body.stmts) {
                        return true;
                    }
                }
                Stmt::Is(is_stmt) => {
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, b) => b,
                            crate::ast::IsBranch::IfBranch(_, b) => b,
                            crate::ast::IsBranch::ElseBranch(b) => b,
                        };
                        if Self::method_mutates_self(&body.stmts) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Check if an expression mutates self (assignment to self.field, or
    /// mutating method call on self.field like push/insert).
    fn expr_mutates_self(expr: &Expr) -> bool {
        match expr {
            // self.field = value  (Op::Asn, AddEq, SubEq, etc.)
            Expr::Bina(lhs, op, _) => {
                if matches!(
                    op,
                    auto_val::Op::Asn
                        | auto_val::Op::AddEq
                        | auto_val::Op::SubEq
                        | auto_val::Op::MulEq
                        | auto_val::Op::DivEq
                        | auto_val::Op::ModEq
                ) {
                    // Plan 514 W3-16: 索引位赋值 self.scopes[i] = v 的根
                    // 也是 self——Index 基座为 self 点链同列(vpush 实证)。
                    if Self::is_self_dot(lhs) {
                        return true;
                    }
                    if let Expr::Index(base, _) = lhs.as_ref() {
                        if Self::is_self_dot(base) {
                            return true;
                        }
                    }
                }
                false
            }
            // self.field.push(...) / self.field.insert(...) / etc.
            Expr::Call(call) => {
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    let mut_methods = [
                        "push", "pop", "insert", "remove", "clear", "next", "extend", "truncate",
                        "retain", "sort", "sort_by", "reverse", "dedup", "swap", "splice", "drain",
                        "append", "resize",
                        // Plan 514 W3-16: .set(i, v) 在发射期转索引赋值——
                        // 同为接收者变异(vpush 的 self.scopes.set 实证)。
                        "set",
                    ];
                    // Plan 514 W3: 裸 `self.method()`(obj 即 Ident self)与
                    // 字段中介形 `self.field.method()` 同列——此前只匹配后者,
                    // 方法体内 `.next()`/`self.next()` 语句漏判 → 接收者误发
                    // &self(E0596)。
                    let obj_is_self = Self::is_self_dot(obj)
                        || matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "self");
                    if mut_methods.contains(&method.as_str()) && obj_is_self {
                        return true;
                    }
                    // Also check nested: self.inner.push(...)
                    if mut_methods.contains(&method.as_str()) {
                        if let Expr::Dot(inner_obj, _) = obj.as_ref() {
                            if Self::is_self_dot(inner_obj) {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Plan 514 W3: 表达式内是否存在对 targets 集合中方法的 self 接收者调用
    /// (`.m(...)`/`self.m(...)`——前导点解析为 Dot(Ident(\"self\"), m))。
    fn expr_calls_self_method(expr: &Expr, targets: &std::collections::HashSet<AutoStr>) -> bool {
        match expr {
            Expr::Call(call) => {
                if let Expr::Dot(obj, m) = call.name.as_ref() {
                    if targets.contains(m.as_str())
                        && matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "self")
                    {
                        return true;
                    }
                    if Self::expr_calls_self_method(obj, targets) {
                        return true;
                    }
                }
                call.args.args.iter().any(|a| {
                    matches!(a,
                    Arg::Pos(e) if Self::expr_calls_self_method(e, targets))
                })
            }
            Expr::Dot(o, _) => Self::expr_calls_self_method(o, targets),
            Expr::Bina(a, _, b) | Expr::Index(a, b) | Expr::NullCoalesce(a, b) => {
                Self::expr_calls_self_method(a, targets) || Self::expr_calls_self_method(b, targets)
            }
            Expr::Unary(_, e)
            | Expr::ErrorPropagate(e)
            | Expr::Some(e)
            | Expr::Ok(e)
            | Expr::Err(e)
            | Expr::BoxExpr(e)
            | Expr::ArcExpr(e) => Self::expr_calls_self_method(e, targets),
            Expr::Array(items) | Expr::Tuple(items) => items
                .iter()
                .any(|e| Self::expr_calls_self_method(e, targets)),
            Expr::Object(pairs) => pairs
                .iter()
                .any(|pr| Self::expr_calls_self_method(&pr.value, targets)),
            _ => false,
        }
    }

    /// Plan 514 W3: 语句体内是否存在对 targets 中方法的 self 接收者调用。
    fn body_calls_self_method(
        stmts: &[Stmt],
        targets: &std::collections::HashSet<AutoStr>,
    ) -> bool {
        for stmt in stmts {
            let hit = match stmt {
                Stmt::Expr(e) => Self::expr_calls_self_method(e, targets),
                Stmt::Store(st) => Self::expr_calls_self_method(&st.expr, targets),
                Stmt::Return(e) => Self::expr_calls_self_method(e, targets),
                Stmt::If(i) => {
                    i.branches.iter().any(|b| {
                        Self::expr_calls_self_method(&b.cond, targets)
                            || Self::body_calls_self_method(&b.body.stmts, targets)
                    }) || i
                        .else_
                        .as_ref()
                        .map_or(false, |e| Self::body_calls_self_method(&e.stmts, targets))
                }
                Stmt::For(f) => {
                    Self::expr_calls_self_method(&f.range, targets)
                        || Self::body_calls_self_method(&f.body.stmts, targets)
                }
                Stmt::Block(b) => Self::body_calls_self_method(&b.stmts, targets),
                Stmt::Is(is) => is.branches.iter().any(|b| {
                    let body = match b {
                        crate::ast::IsBranch::EqBranch(_, bd) => bd,
                        crate::ast::IsBranch::IfBranch(_, bd) => bd,
                        crate::ast::IsBranch::ElseBranch(bd) => bd,
                    };
                    Self::body_calls_self_method(&body.stmts, targets)
                }),
                Stmt::Try(t) => {
                    Self::body_calls_self_method(&t.body.stmts, targets)
                        || Self::body_calls_self_method(&t.catch_body.stmts, targets)
                        || t.finally_body
                            .as_ref()
                            .map_or(false, |f| Self::body_calls_self_method(&f.stmts, targets))
                }
                _ => false,
            };
            if hit {
                return true;
            }
        }
        false
    }

    /// Plan 514 W3: type/ext 方法集的传递性可变方法名——直接判定(is_mut/
    /// method_mutates_self)为种子,经 self 接收者调用闭包扩张(方法体内
    /// 只调 `.fail(...)` 等用户可变方法的方法,接收者同样需要 &mut)。
    pub(super) fn compute_type_mut_methods(methods: &[&Fn]) -> std::collections::HashSet<AutoStr> {
        let mut set: std::collections::HashSet<AutoStr> = methods
            .iter()
            .filter(|m| m.is_mut || Self::method_mutates_self(&m.body.stmts))
            .map(|m| m.name.clone())
            .collect();
        loop {
            let mut grew = false;
            for m in methods {
                if set.contains(m.name.as_str()) {
                    continue;
                }
                if Self::body_calls_self_method(&m.body.stmts, &set) {
                    set.insert(m.name.clone());
                    grew = true;
                }
            }
            if !grew {
                break;
            }
        }
        set
    }

    /// Incremental transpilation (Phase 066)
    /// Plan 163: Check if statements contain any await expression
    pub(super) fn has_await(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_await(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    if Self::has_await(&body.stmts) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::has_await(&branch.body.stmts) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        if Self::has_await(&else_body.stmts) {
                            return true;
                        }
                    }
                }
                // Plan 364 Phase 8 F1: recurse into for-loop bodies (previously
                // fell into `_ => {}`, so an .await inside a for-loop didn't
                // trigger #[tokio::main] async fn main).
                Stmt::For(for_stmt) => {
                    if Self::has_await(&for_stmt.body.stmts) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Like has_await but takes a slice of references (for use with split stmts)
    pub(super) fn has_await_refs(stmts: &[&Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_await(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    let refs: Vec<&Stmt> = body.stmts.iter().collect();
                    if Self::has_await_refs(&refs) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        let refs: Vec<&Stmt> = branch.body.stmts.iter().collect();
                        if Self::has_await_refs(&refs) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        let refs: Vec<&Stmt> = else_body.stmts.iter().collect();
                        if Self::has_await_refs(&refs) {
                            return true;
                        }
                    }
                }
                // Plan 364 Phase 8 F1: recurse into for-loop bodies.
                Stmt::For(for_stmt) => {
                    let refs: Vec<&Stmt> = for_stmt.body.stmts.iter().collect();
                    if Self::has_await_refs(&refs) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Plan 240: Check if statements contain ErrorPropagate (`.?` operator)
    pub(super) fn has_error_propagate(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_error_propagate(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_error_propagate(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_error_propagate(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    if Self::has_error_propagate(&body.stmts) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::has_error_propagate(&branch.body.stmts) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        if Self::has_error_propagate(&else_body.stmts) {
                            return true;
                        }
                    }
                }
                Stmt::For(for_stmt) => {
                    if Self::has_error_propagate(&for_stmt.body.stmts) {
                        return true;
                    }
                }
                Stmt::Is(is_stmt) => {
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        if Self::has_error_propagate(&body.stmts) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Plan 347: Detect whether a function body returns an explicit
    /// `Ok(...)` / `Err(...)` value. The Auto source for some library
    /// functions (e.g. `fn decode(input str) { ... return Ok(...) }`) declares
    /// no return type but actually produces a `Result`. The transpiler must
    /// infer a `Result` return type in that case, otherwise Rust rejects the
    /// `return Ok(...)` against the implicit `()` return type. Recurses into
    /// nested blocks/if/for like `has_error_propagate`.
    pub(super) fn body_returns_result(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Return(expr) => {
                    if matches!(&**expr, Expr::Ok(_) | Expr::Err(_)) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    if Self::body_returns_result(&body.stmts) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::body_returns_result(&branch.body.stmts) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        if Self::body_returns_result(&else_body.stmts) {
                            return true;
                        }
                    }
                }
                Stmt::For(for_stmt) => {
                    if Self::body_returns_result(&for_stmt.body.stmts) {
                        return true;
                    }
                }
                Stmt::Is(is_stmt) => {
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        if Self::body_returns_result(&body.stmts) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Plan 347 (url): Infer the Ok payload type for an un-annotated function
    /// whose body returns `Ok(...)` / `Err(...)`. Previously the inferred
    /// Result was always `Result<String, String>`, which miscompiles when the
    /// `Ok` payload is a struct (e.g. `fn parse(...) { return Ok(Url { ... }) }`
    /// would be typed `Result<String, String>`). This scans the body's
    /// `return Ok(X)` / tail `Ok(X)` expressions and returns:
    ///   * `Type::User(TypeDecl{ name, .. })` when X is a struct construction
    ///     (`Expr::Node { name }`),
    ///   * `Type::StrOwned` (the original default) otherwise.
    /// The first struct-typed Ok payload wins; mixed payloads fall back to
    /// `StrOwned` to stay safe.
    pub(super) fn infer_result_ok_type(&self, stmts: &[Stmt]) -> Type {
        let mut found = None;
        self.scan_result_ok_type(stmts, &mut found);
        found.unwrap_or(Type::StrOwned)
    }

    fn scan_result_ok_type(&self, stmts: &[Stmt], found: &mut Option<Type>) {
        if found.is_some() {
            return;
        }
        for stmt in stmts {
            match stmt {
                Stmt::Return(expr) => {
                    if let Expr::Ok(inner) = &**expr {
                        self.classify_ok_payload(inner, found);
                    }
                }
                // A bare `Ok(...)` expression as the last statement (tail expr).
                Stmt::Expr(e) => {
                    if let Expr::Ok(inner) = e {
                        self.classify_ok_payload(inner, found);
                    }
                }
                Stmt::Block(body) => self.scan_result_ok_type(&body.stmts, found),
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        self.scan_result_ok_type(&branch.body.stmts, found);
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        self.scan_result_ok_type(&else_body.stmts, found);
                    }
                }
                Stmt::For(for_stmt) => self.scan_result_ok_type(&for_stmt.body.stmts, found),
                Stmt::Is(is_stmt) => {
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        self.scan_result_ok_type(&body.stmts, found);
                    }
                }
                _ => {}
            }
            if found.is_some() {
                return;
            }
        }
    }

    /// Classify a single `Ok(...)` payload expression into a Rust type.
    /// Struct construction (`Url { ... }`) parses as `Expr::Node { name }`.
    fn classify_ok_payload(&self, inner: &Expr, found: &mut Option<Type>) {
        if found.is_some() {
            return;
        }
        match inner {
            Expr::Node(node) => {
                // Struct construction: `Url { scheme: ..., ... }`.
                let ty = Type::User(crate::ast::TypeDecl {
                    consts: Vec::new(),
                    name: node.name.clone(),
                    kind: crate::ast::TypeDeclKind::UserType,
                    parent: None,
                    has: Vec::new(),
                    specs: Vec::new(),
                    spec_impls: Vec::new(),
                    generic_params: Vec::new(),
                    members: Vec::new(),
                    delegations: Vec::new(),
                    methods: Vec::new(),
                    attrs: Vec::new(),
                    impl_attrs: vec![],
                    doc: None,
                    is_pub: false,
                });
                *found = Some(ty);
            }
            Expr::Str(_) | Expr::CStr(_) => {
                // String payload -> keep the historical default.
                *found = Some(Type::StrOwned);
            }
            _ => {
                // Anything else (idents, calls, ...): leave unset so other
                // payloads can still refine; falls back to StrOwned at the end.
            }
        }
    }

    /// Plan 347: Collect the names of identifiers that are used as the
    /// scrutinee of an `is`/`match` expression whose branches pattern-match on
    /// `Ok(...)` or `Err(...)`. Such identifiers must be `Result` values. This
    /// lets the transpiler infer a `Result` type for untyped function
    /// parameters (which otherwise default to `i32`) that are matched this way,
    /// e.g. `fn ok_value(r) { is r { Ok(v) -> ... } }`.
    pub(super) fn result_pattern_idents(stmts: &[Stmt]) -> std::collections::HashSet<String> {
        let mut out = std::collections::HashSet::new();
        Self::collect_result_pattern_idents(stmts, &mut out);
        out
    }

    fn collect_result_pattern_idents(stmts: &[Stmt], out: &mut std::collections::HashSet<String>) {
        for stmt in stmts {
            match stmt {
                Stmt::Is(is_stmt) => {
                    let has_ok_err = is_stmt.branches.iter().any(|branch| {
                        if let crate::ast::IsBranch::EqBranch(patterns, _) = branch {
                            patterns.iter().any(|p| matches!(p, Expr::ResultPattern(_)))
                        } else {
                            false
                        }
                    });
                    if has_ok_err {
                        if let Expr::Ident(name) = &is_stmt.target {
                            out.insert(name.to_string());
                        }
                    }
                    // Recurse into branch bodies.
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        Self::collect_result_pattern_idents(&body.stmts, out);
                    }
                }
                Stmt::Block(body) => {
                    Self::collect_result_pattern_idents(&body.stmts, out);
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        Self::collect_result_pattern_idents(&branch.body.stmts, out);
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        Self::collect_result_pattern_idents(&else_body.stmts, out);
                    }
                }
                Stmt::For(for_stmt) => {
                    Self::collect_result_pattern_idents(&for_stmt.body.stmts, out);
                }
                _ => {}
            }
        }
    }

    /// Plan 240: Check if an expression contains ErrorPropagate (`.?` operator)
    fn expr_has_error_propagate(expr: &Expr) -> bool {
        match expr {
            Expr::ErrorPropagate(_) => true,
            Expr::Call(call) => {
                if Self::expr_has_error_propagate(call.name.as_ref()) {
                    return true;
                }
                for arg in &call.args.args {
                    match arg {
                        Arg::Pos(e) | Arg::Pair(_, e) => {
                            if Self::expr_has_error_propagate(e) {
                                return true;
                            }
                        }
                        Arg::Name(_) => {}
                    }
                }
                false
            }
            Expr::Block(body) => Self::has_error_propagate(&body.stmts),
            Expr::Bina(left, _, right) => {
                Self::expr_has_error_propagate(left) || Self::expr_has_error_propagate(right)
            }
            Expr::Unary(_, e) => Self::expr_has_error_propagate(e),
            Expr::Dot(obj, _) => Self::expr_has_error_propagate(obj),
            Expr::Index(arr, idx) => {
                Self::expr_has_error_propagate(arr) || Self::expr_has_error_propagate(idx)
            }
            Expr::View(e) | Expr::Mut(e) | Expr::Move(e) | Expr::Take(e) => {
                Self::expr_has_error_propagate(e)
            }
            Expr::FStr(fstr) => fstr.parts.iter().any(|p| Self::expr_has_error_propagate(p)),
            Expr::Array(arr) => arr.iter().any(|e| Self::expr_has_error_propagate(e)),
            _ => false,
        }
    }

    /// Plan 220 Task 4: Check if an expression needs an `as usize` cast
    /// when used as a slice/array index in Rust.
    ///
    /// Integer literals do NOT need a cast -- Rust infers the correct type
    /// automatically in index position (e.g., `arr[0]` just works).
    /// Non-trivial expressions (variables, binary ops, calls) may be u32/i32
    /// and need explicit `as usize` for Rust indexing and range bounds.
    pub(super) fn needs_usize_cast(expr: &Expr) -> bool {
        match expr {
            // Integer literals: Rust infers correct type in index position
            Expr::Int(_)
            | Expr::Uint(_)
            | Expr::I8(_)
            | Expr::U8(_)
            | Expr::I64(_)
            | Expr::U64(_)
            | Expr::Byte(_) => false,
            // Range: bounds are handled individually, not the range itself
            Expr::Range(_) => false,
            // Non-integer literals: not used as indices
            Expr::Bool(_) | Expr::Nil | Expr::Null => false,
            // Variables, binary ops, calls, dot access, etc. may be u32/i32
            _ => true,
        }
    }

    /// PLAN-596 T-07 (DIV-DEP-8): a use.rs-imported dep TYPE — PascalCase
    /// name present as a use leaf, never declared locally. Local decls
    /// (struct/tag/enum/union) always win, keeping Debug-dependent存量语料
    /// on the `{:?}` path (AC-04 isolation arm).
    fn is_dep_type_name(&self, name: &str) -> bool {
        // PascalCase required: SCREAMING_CASE use.rs imports are consts
        // (DIV-DEP-13), never types — `STANDARD.encode(..)` is a const-value
        // method call, not a ctor chain.
        if !name.chars().next().map_or(false, |c| c.is_uppercase())
            || !name.chars().any(|c| c.is_ascii_lowercase())
        {
            return false;
        }
        if self.local_struct_types.contains(name)
            || self.struct_fields.contains_key(name)
            || self.struct_field_types.contains_key(name)
            || self.tag_types.contains(name)
            || self.known_enum_names.contains(name)
            || self.union_types.contains(name)
        {
            return false;
        }
        self.uses.iter().any(|u| {
            let u = u.as_str();
            u == name || u.ends_with(&format!("::{}", name)) || u.contains(&format!("{{{}}}", name))
        })
    }

    /// PLAN-596 T-07 (DIV-DEP-13): a SCREAMING_CASE use.rs import (STANDARD,
    /// NIL, OP_XXX) is a const VALUE receiver, not a type — `CONST.method(..)`
    /// must lower to a dot-call (trait-method consts like base64's
    /// `STANDARD.encode` are E0224 as `CONST::method`). Rust types are
    /// PascalCase; local declarations take precedence.
    pub(super) fn is_dep_constant_name(&self, name: &str) -> bool {
        let has_upper = name.chars().any(|c| c.is_ascii_uppercase());
        let no_lower = !name.chars().any(|c| c.is_ascii_lowercase());
        has_upper
            && no_lower
            && !self.local_struct_types.contains(name)
            && !self.tag_types.contains(name)
            && !self.known_enum_names.contains(name)
            && !self.union_types.contains(name)
            && self.uses.iter().any(|u| {
                let u = u.as_str();
                u == name
                    || u.ends_with(&format!("::{}", name))
                    || u.contains(&format!("{{{}}}", name))
            })
    }

    /// PLAN-596 T-07 (DIV-DEP-8): does this expression denote a rust-typed
    /// VALUE from a dep crate (Display-stringified, three-way parity) rather
    /// than an Auto type (Debug fallback)? Ident receivers resolve through
    /// local_var_types (fed by dep_ctor_type at store time); inline ctor
    /// chains (`Url.parse(..).unwrap()`) walk the chain directly.
    pub(super) fn receiver_is_dep_rust_value(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => matches!(self.local_var_types.get(name),
                Some(Type::User(td)) if self.is_dep_type_name(td.name.as_str())),
            Expr::Call(call) => match call.name.as_ref() {
                Expr::Dot(obj, m) if m.as_str() == "unwrap" => self.receiver_is_dep_rust_value(obj),
                Expr::Dot(obj, _) => {
                    matches!(obj.as_ref(), Expr::Ident(n) if self.is_dep_type_name(n))
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// PLAN-596 T-07: dep ctor chain detection for store-time typing —
    /// `let v = Version.parse(..).unwrap()` / `let u = Url.parse(..)` yield
    /// the dep type name, so the binding gets a User type and downstream
    /// `.to(str)`/print resolve Display instead of the Unknown fallback.
    pub(super) fn dep_ctor_type(&self, expr: &Expr) -> Option<AutoStr> {
        match expr {
            Expr::Call(call) => match call.name.as_ref() {
                Expr::Dot(obj, m) if m.as_str() == "unwrap" => self.dep_ctor_type(obj),
                Expr::Dot(obj, _) => match obj.as_ref() {
                    Expr::Ident(n) if self.is_dep_type_name(n) => Some(n.clone()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    /// Check if an expression likely produces a Debug-only type (no Display impl).
    /// Detects patterns like `.elapsed()`, `Instant::now()`, and variables named
    /// duration/elapsed/instant.
    pub(super) fn needs_debug_format(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => {
                // Only use name heuristics for types strongly associated with non-Display:
                // Duration, Instant, DirEntry, etc. Common variable names like "count",
                // "value", "avg" are often primitives (i32, f64) that implement Display.
                let lower = name.as_str().to_lowercase();
                if lower.contains("duration")
                    || lower.contains("elapsed")
                    || lower.contains("instant")
                    || lower.contains("dir_entry")
                {
                    return true;
                }
                // Check local_var_types for non-Display types
                if let Some(ty) = self.local_var_types.get(name) {
                    return matches!(
                        ty,
                        Type::List(_)
                            | Type::Map(_, _)
                            | Type::Array(_)
                            | Type::RuntimeArray(_)
                            | Type::Slice(_)
                            | Type::Option(_)
                            | Type::Result(_)
                            | Type::Tuple(_)
                            | Type::Tag(_)
                            | Type::Enum(_)
                    );
                }
                false
            }
            Expr::Dot(obj, method) => method == "elapsed" || self.needs_debug_format(obj),
            Expr::Bina(lhs, op, rhs) => {
                if matches!(op, Op::Dot) {
                    // Check for expr.elapsed()
                    if let Expr::Ident(m) = rhs.as_ref() {
                        if m.as_str() == "elapsed" {
                            return true;
                        }
                    }
                    self.needs_debug_format(lhs)
                } else {
                    self.needs_debug_format(lhs) || self.needs_debug_format(rhs)
                }
            }
            Expr::Call(call) => self.needs_debug_format(&call.name),
            Expr::ErrorPropagate(inner) => self.needs_debug_format(inner),
            _ => false,
        }
    }

    pub(super) fn is_self_dot(expr: &Expr) -> bool {
        matches!(expr, Expr::Dot(obj, _) if matches!(obj.as_ref(), Expr::Ident(name) if name == "self"))
    }

    /// Plan 013 (B16): collect bare-identifier names used as variant-pattern
    /// arguments (e.g. the `child` in `Kid.Node(child)`), so a later auto-clone
    /// at a call site can deref a Box<T> before cloning. Recurses into nested
    /// calls so multi-field variants are covered too.

    /// Plan 163: Check if an expression contains await
    fn expr_has_await(expr: &Expr) -> bool {
        match expr {
            Expr::Await { .. } => true,
            Expr::Call(call) => {
                if Self::expr_has_await(call.name.as_ref()) {
                    return true;
                }
                for arg in &call.args.args {
                    match arg {
                        Arg::Pos(e) | Arg::Pair(_, e) => {
                            if Self::expr_has_await(e) {
                                return true;
                            }
                        }
                        Arg::Name(_) => {}
                    }
                }
                false
            }
            Expr::Block(body) => Self::has_await(&body.stmts),
            Expr::Bina(left, _, right) => Self::expr_has_await(left) || Self::expr_has_await(right),
            Expr::Unary(_, expr) => Self::expr_has_await(expr),
            Expr::Dot(obj, _) => Self::expr_has_await(obj),
            Expr::Index(arr, idx) => Self::expr_has_await(arr) || Self::expr_has_await(idx),
            Expr::View(e) | Expr::Mut(e) | Expr::Move(e) | Expr::Take(e) => Self::expr_has_await(e),
            Expr::AsyncBlock { body, .. } => Self::has_await(&body.stmts),
            Expr::Cast { expr, .. } | Expr::To { expr, .. } => Self::expr_has_await(expr),
            Expr::NullCoalesce(l, r) => Self::expr_has_await(l) || Self::expr_has_await(r),
            Expr::ErrorPropagate(e) => Self::expr_has_await(e),
            _ => false,
        }
    }
}

// ---- PLAN-884 T-06：跨模块签名收集器（project/merged 两管线共用单份；
// collect_fn_param_types 保持双份——merged 版额外收集 merge-mut 标志，
// 语义分叉为载荷，见 KNOWN-DEBT） ----

pub(super) fn collect_fn_spec_params(
    stmts: &[Stmt],
    type_name: &str,
    map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
) {
    let generic_methods = [
        "get", "set", "insert", "push", "remove", "contains", "len", "is_empty", "iter", "keys",
        "values", "clone", "new", "update", "delete", "find", "index",
    ];
    let scan_fn =
        |fn_decl: &Fn, parent: &str, map: &mut std::collections::HashMap<AutoStr, Vec<bool>>| {
            let spec_flags: Vec<bool> = fn_decl
                .params
                .iter()
                .map(|p| matches!(p.ty, Type::Spec(_)))
                .collect();
            if !spec_flags.is_empty() {
                if !generic_methods.contains(&fn_decl.name.as_str()) {
                    map.insert(fn_decl.name.clone(), spec_flags.clone());
                }
                if !parent.is_empty() || fn_decl.parent.is_some() {
                    let p = fn_decl
                        .parent
                        .as_ref()
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| parent.to_string());
                    let qualified = format!("{}.{}", p, fn_decl.name);
                    map.insert(AutoStr::from(qualified), spec_flags);
                }
            }
        };
    for stmt in stmts {
        if let Stmt::Fn(fn_decl) = stmt {
            scan_fn(fn_decl, type_name, map);
        }
        if let Stmt::TypeDecl(type_decl) = stmt {
            for method in &type_decl.methods {
                scan_fn(method, &type_decl.name.to_string(), map);
            }
        }
        if let Stmt::Ext(ext) = stmt {
            for method in &ext.methods {
                scan_fn(method, &ext.target.to_string(), map);
            }
        }
    }
}

pub(super) fn collect_fn_str_params(
    stmts: &[Stmt],
    type_name: &str,
    map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
) {
    // Generic method names that should never be stored as bare-name keys
    // to avoid false positive .as_str() on unrelated calls
    let generic_methods = [
        "get", "set", "insert", "push", "remove", "contains", "len", "is_empty", "iter", "keys",
        "values", "clone", "new", "update", "delete", "find", "index",
    ];
    for stmt in stmts {
        if let Stmt::Fn(fn_decl) = stmt {
            let str_flags: Vec<bool> = fn_decl
                .params
                .iter()
                .map(|p| matches!(p.ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                .collect();
            if !str_flags.is_empty() {
                // Only store bare name for non-generic method names
                if !generic_methods.contains(&fn_decl.name.as_str()) {
                    map.insert(fn_decl.name.clone(), str_flags.clone());
                }
                // Always store qualified key "TypeName.method_name" for methods
                if !type_name.is_empty() || fn_decl.parent.is_some() {
                    let parent = fn_decl
                        .parent
                        .as_ref()
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| type_name.to_string());
                    let qualified = format!("{}.{}", parent, fn_decl.name);
                    map.insert(AutoStr::from(qualified), str_flags);
                }
            }
        }
        // Also scan inside type declarations for methods
        if let Stmt::TypeDecl(type_decl) = stmt {
            let type_name_str = type_decl.name.to_string();
            for method in &type_decl.methods {
                let str_flags: Vec<bool> = method
                    .params
                    .iter()
                    .map(|p| matches!(p.ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                    .collect();
                if !str_flags.is_empty() {
                    if !generic_methods.contains(&method.name.as_str()) {
                        map.insert(method.name.clone(), str_flags.clone());
                    }
                    let qualified = format!("{}.{}", type_name_str, method.name);
                    map.insert(AutoStr::from(qualified), str_flags);
                }
            }
        }
    }
}
