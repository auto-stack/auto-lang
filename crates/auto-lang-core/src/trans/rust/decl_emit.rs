//! PLAN-843 结构拆分（自 rust.rs 原样搬移；算法/行为零变化）。
use super::*;

impl RustTrans {
    /// Plan musk-022 CRUD 智能扩展: transpile a single `Fn` AST to Rust text
    /// (no module header/imports). Reuses `fn_decl` (proven by trans_incremental).
    pub fn transpile_fn(&mut self, f: &Fn) -> AutoResult<String> {
        let mut sink = Sink::new(AutoStr::from(f.name.as_str()));
        self.fn_decl(f, &mut sink)?;
        let out = String::from_utf8(sink.done()?.to_vec())
            .map_err(|e| format!("Invalid UTF-8 in transpile_fn output: {}", e))?;
        Ok(out)
    }

    pub(super) fn fn_decl(&mut self, fn_decl: &Fn, sink: &mut Sink) -> AutoResult<()> {
        // Skip C/VM function declarations (implemented externally)
        if matches!(fn_decl.kind, FnKind::CFunction | FnKind::VmFunction) {
            return Ok(());
        }

        // Plan 610 ⑤: `#[export]` fn — emit the cdylib wrapper module ahead of
        // the safe body emitted below (body keeps its original name).
        if let Some(abi) = fn_decl.export_abi.clone() {
            self.export_wrapper_module(fn_decl, &abi, sink)?;
        }

        // Clear local var type cache for this function, register params
        self.local_var_types.clear();
        // Plan 427: is-arm &str bindings are function-scoped (mirrors
        // local_var_types above).
        self.str_slice_pattern_bindings.clear();
        // Plan 016 (auto-down R1): loop-var tracking sets are function-scoped
        // as well; see transpile_body_stmts.
        self.borrowed_iter_vars.clear();
        self.by_value_iter_bindings.clear();
        for param in &fn_decl.params {
            self.local_var_types
                .insert(param.name.clone(), param.ty.clone());
        }
        // Plan 514 W3:方法体注册 `self` 的宿主类型(占位 User)——
        // 使 str 参数 as_str 强转/struct 守卫等按接收者类型的机制对
        // `self.method(...)`/`self.field` 统一生效(lib 方法体实测:
        // `self.fail(format!...)` 此前缺 as_str → E0308)。
        if fn_decl.parent.is_some() && !fn_decl.is_static {
            if let Some(parent) = &fn_decl.parent {
                self.local_var_types.insert(
                    "self".into(),
                    Type::User(crate::ast::TypeDecl {
                        consts: Vec::new(),
                        name: parent.clone(),
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
                        impl_attrs: Vec::new(),
                        doc: None,
                        is_pub: false,
                    }),
                );
            }
        }
        // Plan 399 Phase 11.5: scan the fn body for `let` bindings that are
        // later mutated (push/insert/extend/assign) — those need `let mut`.
        self.mutated_let_bindings.clear();
        let rust_types = self.rust_imported_type_names();
        self.mutated_let_bindings =
            Self::scan_mutated_bindings(&fn_decl.body, &self.all_mut_method_names, &rust_types);
        // Plan 447 H5: per-function is-scrutinee use counts drive the narrow
        // `match &v` emission (>= 2 uses of the same ident only).
        self.fn_is_scrutinee_counts = Self::scan_is_scrutinee_uses(&fn_decl.body);

        // Plan 433 A1: pre-register local types for un-annotated `let/var x = fn()`
        // bindings from the fn return-type cache, so later List.get/field/str-param
        // resolution (is_auto_list_expr, struct_field_types lookups) can key off
        // them. Mirrors what an annotated `let x T = ...` registers at store().
        {
            let mut locals: Vec<(AutoStr, Type)> = Vec::new();
            Self::scan_call_init_bindings(&fn_decl.body.stmts, &self.fn_ret_types, &mut locals);
            for (name, ty) in locals {
                // Annotated stores already registered at store() emission time
                // (later, in order); only fill gaps so we never clobber.
                self.local_var_types.entry(name).or_insert(ty);
            }
        }

        // Plan 310 Phase 2: Set current function context for escape-tier queries.
        // The escape_results key matches how transpile_rust registers functions:
        // top-level fns use fn.name; methods inside TypeDecl use "Type.method".
        // We reconstruct that key here. Scope depth resets to 0 at body entry.
        self.current_fn_name = if let Some(parent) = &fn_decl.parent {
            format!("{}.{}", parent, fn_decl.name).into()
        } else {
            fn_decl.name.clone()
        };
        self.current_scope_depth = 0;

        // Plan 204 Phase 3: Track whether current function returns !T or Result<T,E> (for Err boxing)
        self.current_fn_is_result = matches!(fn_decl.ret, Type::Result(_))
            || matches!(&fn_decl.ret, Type::GenericInstance(inst) if inst.base_name == "Result");

        // Infer concrete error type from Err() calls in function body
        self.current_fn_err_type = None;
        if self.current_fn_is_result {
            self.current_fn_err_type = self.infer_err_enum(&fn_decl.body.stmts);
        }

        // Emit doc comments
        if let Some(ref doc) = fn_decl.doc {
            let is_method = fn_decl.parent.is_some();
            for line in doc.split('\n') {
                if is_method {
                    self.print_indent(&mut sink.body)?;
                }
                write!(sink.body, "/// {}\n", line)?;
            }
        }

        // Check if this is a method (has parent)
        let is_method = fn_decl.parent.is_some();

        // Print indent for methods (inside impl block)
        if is_method {
            self.print_indent(&mut sink.body)?;
        }

        // Plan 163: #[tokio::main] for async main.
        // Plan 364 Phase 8 F1: also trigger for for-over-Stream (injects .next().await).
        // Plan 387: a program with any `task` definition forces main to be async
        // AND uses the current_thread flavor to match the VM's single-threaded
        // cooperative actor scheduling (Plan 317 path B).
        let main_refs: Vec<&Stmt> = fn_decl.body.stmts.iter().collect();
        let is_main_with_await = !is_method
            && fn_decl.name.as_ref() == "main"
            && (Self::has_await(&fn_decl.body.stmts) || self.body_has_stream_for(&main_refs));
        let is_main_actor =
            !is_method && fn_decl.name.as_ref() == "main" && self.program_has_actors;
        if is_main_actor {
            if is_method {
                // already indented
            } else {
                self.print_indent(&mut sink.body)?;
            }
            // Plan 387: actor programs use the multi_thread runtime. We tried
            // current_thread for VM single-thread parity, but it deadlocks when
            // `run_to_completion().await` joins an actor that is itself waiting
            // for the `TaskHandle` (last sender) to drop — which only happens
            // when main returns, i.e. after the join. multi_thread avoids this
            // because spawned tasks run on worker threads concurrently with the
            // join. Observable stdout behavior still matches the VM (Plan 317).
            write!(sink.body, "#[tokio::main]\n")?;
            if is_method {
                self.print_indent(&mut sink.body)?;
            }
        } else if is_main_with_await {
            if is_method {
                // already indented
            } else {
                self.print_indent(&mut sink.body)?;
            }
            write!(sink.body, "#[tokio::main]\n")?;
            if is_method {
                self.print_indent(&mut sink.body)?;
            }
        }

        // Plan 163: Output pub prefix
        // Methods in pub types inherit pub visibility even if fn_decl.is_pub is false.
        // EXCEPT in trait impl blocks — trait methods are never `pub` (E0449).
        if !self.in_trait_impl && (fn_decl.is_pub || self.inside_pub_type) {
            write!(sink.body, "pub ")?;
        }

        // Function signature
        // Auto-detect async: functions returning ~T (Future/Handle) are async in Rust
        // Also detect async main (has .await in body)
        // Plan 373 G2: ~Result methods → async fn (for trait impls with #[async_trait])
        // Plan 382 (A.1): `Type::Result` = `!T` (SYNC Result<T, Box<dyn Error>>) —
        // must NOT be async (Plan 204); `~Result` → GenericInstance("Future").
        let is_async_fn = is_main_actor
            || is_main_with_await
            || matches!(fn_decl.ret, Type::Handle { .. })
            || matches!(&fn_decl.ret, Type::GenericInstance(inst) if inst.base_name == "Future");

        // Plan 321: Detect generator functions (return ~Iter<T> or ~Stream<T>)
        let is_generator_fn = matches!(&fn_decl.ret, Type::GenericInstance(inst)
            if inst.base_name == "Iter" || inst.base_name == "Stream");
        // Extract inner type for the impl Iterator/Stream return
        let generator_inner_type: Option<String> = if is_generator_fn {
            if let Type::GenericInstance(inst) = &fn_decl.ret {
                if let Some(inner) = inst.args.first() {
                    Some(self.rust_type_name(inner))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Plan 364 W2: fn-level pass-through attrs (#[tokio.main], #[allow(...)], dotted macros)
        for attr in &fn_decl.attrs {
            write!(sink.body, "#[{}]\n", attr)?;
        }
        if is_async_fn {
            write!(sink.body, "async ")?;
        }
        write!(sink.body, "fn {}", fn_decl.name)?;

        // Plan 166: Emit generic type parameters from #[with(T as Trait)]
        // Plan 052-followup (417 项④): const generic params (<N u32>) emit as
        // Rust const generics (`<T, const N: u32>`) — previously dropped, so
        // `fn f<T, N u32>` lost the N dimension entirely.
        if !fn_decl.type_params.is_empty() || !fn_decl.const_params.is_empty() {
            write!(sink.body, "<")?;
            let mut first = true;
            for tp in &fn_decl.type_params {
                if !first {
                    write!(sink.body, ", ")?;
                }
                first = false;
                write!(sink.body, "{}", tp.name)?;
                // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                if !tp.constraint.is_empty() {
                    write!(sink.body, ": ")?;
                    for (ci, ct) in tp.constraint.iter().enumerate() {
                        if ci > 0 {
                            write!(sink.body, " + ")?;
                        }
                        write!(sink.body, "{}", self.rust_bound_name(ct))?;
                    }
                }
            }
            for cp in &fn_decl.const_params {
                if !first {
                    write!(sink.body, ", ")?;
                }
                first = false;
                write!(
                    sink.body,
                    "const {}: {}",
                    cp.name,
                    self.rust_type_name(&cp.typ)
                )?;
            }
            write!(sink.body, ">")?;
        }

        // Parameters
        write!(sink.body, "(")?;

        // Plan 347: Collect identifiers that are matched against Ok/Err
        // patterns inside this function body. Untyped params (which default to
        // `int` at parse time) that appear in this set are really `Result`
        // values, so emit them as `Result<String, String>`.
        let result_idents = Self::result_pattern_idents(&fn_decl.body.stmts);
        // Add &self as first parameter for methods (except constructors)
        let skip_first_self = is_method
            && !fn_decl.is_static
            && fn_decl.name.as_str() != "new"
            && fn_decl
                .params
                .first()
                .map_or(false, |p| p.name.as_str() == "self");
        if is_method && !fn_decl.is_static && fn_decl.name.as_str() != "new" {
            // Plan 163: &mut self for mut methods
            // Plan 373: also auto-detect self-mutation (self.field = ... or
            // self.field.push/insert/...) when source doesn't explicitly say mut fn.
            let needs_mut = fn_decl.is_mut
                || Self::method_mutates_self(&fn_decl.body.stmts)
                // Plan 514 W3: 跨方法 &mut 传递——体内调用同类型可变方法
                // (经发射前预计算的传递闭包)。
                || self
                    .current_type_mut_methods
                    .as_ref()
                    .map_or(false, |s| s.contains(fn_decl.name.as_str()));
            // Plan 016 Phase A A4: builder pattern detection. A `mut fn` that
            // returns the enclosing type and ends with `return self` is a
            // consuming-self builder (rust-ref style: `fn(self) -> Self`).
            // Emit `mut self` so `return self` compiles (returns owned self,
            // enabling chaining `.with_x(...).with_y(...)`). Without this, the
            // method gets `&mut self` and `return self` returns the borrow →
            // E0308 "expected Self, found &mut Self".
            let is_builder = needs_mut && {
                let ret_is_parent = fn_decl
                    .parent
                    .as_ref()
                    .map(|p| p.to_string())
                    .map(|parent_name| {
                        // ret is the enclosing type (User named like the parent,
                        // or the implicit Self).
                        match &fn_decl.ret {
                            Type::User(td) => td.name.as_str() == parent_name,
                            _ => false,
                        }
                    })
                    .unwrap_or(false);
                let ends_with_return_self = fn_decl
                    .body
                    .stmts
                    .iter()
                    .last()
                    .map(|s| {
                        matches!(s, Stmt::Return(expr)
                        if matches!(expr.as_ref(), Expr::Ident(name) if name.as_str() == "self"))
                    })
                    .unwrap_or(false);
                ret_is_parent && ends_with_return_self
            };
            if is_builder {
                write!(sink.body, "mut self")?;
            } else if needs_mut {
                write!(sink.body, "&mut self")?;
            } else {
                write!(sink.body, "&self")?;
            }
            // Skip the 'self' param if it was the receiver in Auto
            let params_to_emit: Vec<_> = if skip_first_self {
                fn_decl.params.iter().skip(1).collect()
            } else {
                fn_decl.params.iter().collect()
            };
            if !params_to_emit.is_empty() {
                write!(sink.body, ", ")?;
            }
            for (i, param) in params_to_emit.iter().enumerate() {
                // Plan 380 P3: destructure params output as `TypeName(name): Type`
                let param_name = if let Some(ref d) = param.destructure {
                    format!("{}({})", d.wrapper_type, param.name)
                } else {
                    param.name.to_string()
                };
                if param.mode == crate::ast::ParamMode::Mut {
                    // C1 (Plan 018 §12 a2r-11): `mut p T` on a METHOD param →
                    // `p: &mut T` (mirror of the free-function branch below).
                    // Was missing → ext methods emitted `mut p: T` (by-value),
                    // so `mut doc Doc` couldn't mutate the caller's doc.
                    write!(
                        sink.body,
                        "{}: &mut {}",
                        param_name,
                        self.effective_param_type_name(param, &result_idents)
                    )?;
                } else {
                    write!(
                        sink.body,
                        "{}: {}",
                        param_name,
                        self.effective_param_type_name(param, &result_idents)
                    )?;
                }
                if i < params_to_emit.len() - 1 {
                    write!(sink.body, ", ")?;
                }
            }
        } else {
            for (i, param) in fn_decl.params.iter().enumerate() {
                // Plan 610 ⑤: export-fn HANDLE params — the sibling wrapper
                // derefs the raw `*mut T` null-safely; the body receives
                // `&mut T` so field/method access on the session is native.
                if fn_decl.export_abi.is_some()
                    && matches!(&param.ty, Type::Ptr(_))
                    && Self::export_ptr_pointee_is_handle(&param.ty)
                {
                    let pointee = match &param.ty {
                        Type::Ptr(p) => self.rust_type_name(&p.of.borrow()),
                        _ => unreachable!(),
                    };
                    write!(sink.body, "{}: &mut {}", param.name, pointee)?;
                } else if self.merge_mode && Self::is_merge_mut_type(&param.ty) {
                    write!(
                        sink.body,
                        "{}: &mut {}",
                        param.name,
                        self.rust_type_name(&param.ty)
                    )?;
                } else if Self::is_sb_ref_type(&param.ty) {
                    // Plan 347: StringBuilder params are shared mutable buffers
                    // threaded through recursion — emit `mut sb: &mut a2r_std::StringBuilder`.
                    // The `mut` on the binding is required so the `&mut` reference
                    // can be re-borrowed (`&mut sb`) when forwarded to another
                    // recursive helper; without it Rust rejects the reborrow
                    // (E0596). Appends accumulate into one buffer across frames.
                    write!(
                        sink.body,
                        "mut {}: &mut {}",
                        param.name,
                        self.rust_type_name(&param.ty)
                    )?;
                } else {
                    // Plan 380 P3: destructure params output as `TypeName(name): Type`
                    let param_name = if let Some(ref d) = param.destructure {
                        format!("{}({})", d.wrapper_type, param.name)
                    } else {
                        param.name.to_string()
                    };
                    if param.mode == crate::ast::ParamMode::Mut {
                        // C11 (Plan 018 §12 a2r-11): `mut p T` → `p: &mut T`
                        // (reference, enables in-place mutation of the caller's
                        // value). Previously emitted `mut p: T` (mutable by-value),
                        // which no .at/golden used.
                        write!(
                            sink.body,
                            "{}: &mut {}",
                            param_name,
                            self.effective_param_type_name(param, &result_idents)
                        )?;
                    } else {
                        write!(
                            sink.body,
                            "{}: {}",
                            param_name,
                            self.effective_param_type_name(param, &result_idents)
                        )?;
                    }
                }
                if i < fn_decl.params.len() - 1 {
                    write!(sink.body, ", ")?;
                }
            }
        }
        write!(sink.body, ")")?;

        // Cache which params are str (&str) type for auto-borrow at call sites
        self.current_fn_str_params.clear();
        self.current_fn_mut_params.clear();
        let str_param_flags: Vec<bool> = fn_decl
            .params
            .iter()
            .map(|p| {
                matches!(
                    p.ty,
                    Type::StrFixed(_) | Type::StrSlice | Type::StrOwned | Type::CStrLit
                )
            })
            .collect();
        for param in &fn_decl.params {
            if matches!(
                param.ty,
                Type::StrFixed(_) | Type::StrSlice | Type::StrOwned | Type::CStrLit
            ) {
                self.current_fn_str_params.insert(param.name.clone());
                self.fn_param_str_slice.insert(param.name.clone());
            }
            // Track &mut params (merge mode context types) — skip &mut at call sites
            if self.merge_mode && Self::is_merge_mut_type(&param.ty) {
                self.current_fn_mut_params.insert(param.name.clone());
            }
            // C11 (Plan 018 §12 a2r-11): `mut p T` params are &mut refs —
            // `p = x` must emit `*p = x` (deref-assign into the caller's value).
            if param.mode == crate::ast::ParamMode::Mut {
                self.current_fn_mut_params.insert(param.name.clone());
            }
            // Plan 390 §15.11 (L2 转正): spec-typed params are already
            // `Box<dyn Trait>` — track them like spec-bound idents so
            // `Arc(tool)` renders `Arc::from(tool)` (Box→Arc single wrap)
            // instead of `Arc::new(tool)` (`Arc<Box<dyn Trait>>` double wrap).
            if matches!(param.ty, Type::Spec(_)) {
                self.spec_bound_idents.insert(param.name.clone());
            }
        }
        self.fn_str_param_indices
            .insert(fn_decl.name.clone(), str_param_flags);

        // Cache which params are non-Copy types (need .clone() at call sites).
        // Plan 387 §16 P0-2: TaskRef<T> is a move-only single-owner type (not
        // Clone) — passing it must MOVE, never clone. Exclude it from the
        // struct-param clone set so call sites emit `forward(h)` not `forward(h.clone())`.
        let struct_param_flags: Vec<bool> = fn_decl
            .params
            .iter()
            .map(|p| {
                let is_taskref =
                    matches!(&p.ty, Type::GenericInstance(inst) if inst.base_name == "TaskRef");
                !Self::is_copy_type(&p.ty) && !is_taskref
            })
            .collect();
        self.fn_struct_param_indices
            .insert(fn_decl.name.clone(), struct_param_flags);

        // Cache full parameter types for type-aware call site generation
        let param_types: Vec<Type> = fn_decl.params.iter().map(|p| p.ty.clone()).collect();
        self.fn_param_types
            .insert(fn_decl.name.clone(), param_types.clone());
        if let Some(parent) = &fn_decl.parent {
            let qualified: AutoStr = format!("{}.{}", parent, fn_decl.name).into();
            self.fn_param_types.insert(qualified, param_types);
        }

        // Plan 523 H3: 未注解 fn 返回推断值（签名/缓存/调用点三处同源）。
        // 未注解 fn 在 parser 即坍缩为 Void(:8761)，故以 Void 为触发位；
        // 体无显式 return 表达式时推断回 Unknown → 维持 Void 原样。
        // main 排除（Rust 要求 fn main 返回 ()；main 体内 return 值仅作
        // 退出语义，002_const_generics/002_list_storage 金样实证）。
        let inferred_ret: Type =
            if matches!(fn_decl.ret, Type::Void) && fn_decl.name.as_str() != "main" {
                let t = self.infer_fn_ret_type(&fn_decl.body);
                if matches!(t, Type::Unknown | Type::Void) {
                    fn_decl.ret.clone()
                } else {
                    t
                }
            } else {
                fn_decl.ret.clone()
            };
        // Plan 373: Cache return type for .await insertion (keyed same as params).
        self.fn_ret_types
            .insert(fn_decl.name.clone(), inferred_ret.clone());
        if let Some(parent) = &fn_decl.parent {
            let qualified: AutoStr = format!("{}.{}", parent, fn_decl.name).into();
            self.fn_ret_types.insert(qualified, inferred_ret.clone());
        }

        // In merge mode, track which params are context types (need &mut instead of .clone())
        if self.merge_mode {
            let merge_mut_flags: Vec<bool> = fn_decl
                .params
                .iter()
                .map(|p| Self::is_merge_mut_type(&p.ty))
                .collect();
            self.fn_merge_mut_params
                .insert(fn_decl.name.clone(), merge_mut_flags);
        }

        // C11 (Plan 018 §12 a2r-11): track `mut p T` params → call sites pass `&mut arg`.
        let mut_param_flags: Vec<bool> = fn_decl
            .params
            .iter()
            .map(|p| p.mode == crate::ast::ParamMode::Mut)
            .collect();
        self.fn_mut_params
            .insert(fn_decl.name.clone(), mut_param_flags);

        // Cache which params are spec types (need Box::new() at call sites)
        let spec_param_flags: Vec<bool> = fn_decl
            .params
            .iter()
            .map(|p| matches!(p.ty, Type::Spec(_)))
            .collect();
        self.fn_spec_param_indices
            .insert(fn_decl.name.clone(), spec_param_flags);

        // Cache which params are Int type (need enum→i32 cast at call sites).
        // Plan 347: exclude params whose effective type was inferred as Result
        // from Ok/Err pattern matching (they are not really Int).
        let int_param_flags: Vec<bool> = fn_decl
            .params
            .iter()
            .map(|p| matches!(p.ty, Type::Int) && !result_idents.contains(p.name.as_str()))
            .collect();
        self.fn_int_param_indices
            .insert(fn_decl.name.clone(), int_param_flags);

        // Plan 240: If function returns void but body uses .? (ErrorPropagate),
        // auto-wrap return type as Result<(), Box<dyn std::error::Error>>
        let fn_body_has_try =
            matches!(fn_decl.ret, Type::Void) && Self::has_error_propagate(&fn_decl.body.stmts);
        // Plan 347: If a void-declared function body returns explicit
        // `Ok(...)` / `Err(...)` values, infer a `Result<String, String>`
        // return type. This covers library functions written without an
        // explicit return-type annotation (e.g. `fn decode(...) { ... return Ok(s) }`).
        let fn_body_returns_result = matches!(fn_decl.ret, Type::Void)
            && !fn_body_has_try
            && Self::body_returns_result(&fn_decl.body.stmts);

        // Plan 232: Track str-type parameter names for .to_string() on return
        // (populated above at line 5274-5278)
        // Plan 240: When fn body has .? but declared as void, treat as Result<(), ...>
        // so that Ok("hello") -> Ok("hello".to_string()) works correctly
        let effective_ret_type = if fn_body_has_try {
            Type::Result(Box::new(Type::Void))
        } else if fn_body_returns_result {
            // Plan 347: infer the Ok payload type from `return Ok(X)` in the
            // body. Struct constructions (`Url { ... }`) produce
            // `Result<Url, String>`; string payloads fall back to the
            // historical `Result<String, String>`.
            let ok_ty = self.infer_result_ok_type(&fn_decl.body.stmts);
            Type::Result(Box::new(ok_ty))
        } else {
            inferred_ret.clone()
        };
        self.current_fn_ret_type = Some(effective_ret_type.clone());
        // Plan 417-E3: record this fn's type-parameter names so return/param
        // positions referencing one render as the bare name (`-> T`), never
        // `impl T` (the PascalCase-trait heuristic misfires on single letters).
        self.current_fn_type_params = fn_decl
            .type_params
            .iter()
            .map(|tp| tp.name.clone())
            .collect();
        // Plan 204 Phase 1B: Use rust_return_type_name for return positions (str -> String)
        if fn_body_has_try {
            write!(sink.body, " -> Result<(), Box<dyn std::error::Error>>")?;
        } else if fn_body_returns_result {
            // Plan 347: emit the inferred Ok type. rust_return_type_name maps
            // Type::User(Url) -> "Url" and Type::StrOwned -> "String".
            let ok_ty = match &effective_ret_type {
                Type::Result(inner) => inner.as_ref().clone(),
                _ => Type::StrOwned,
            };
            let ok_str = self.rust_return_type_name(&ok_ty);
            write!(sink.body, " -> Result<{}, String>", ok_str)?;
        } else if is_generator_fn {
            // Plan 321: Generator functions return impl Iterator/Stream
            let inner = generator_inner_type.as_deref().unwrap_or("String");
            // Plan 417-D2: BOTH ~Stream and ~Iter emit `impl futures::Stream`.
            // The generator body is wrapped in async_stream::stream! which
            // yields a Stream — declaring `impl Iterator` (the old ~Iter
            // spelling) never compiled (001_simple_yield was left
            // unregistered for exactly this reason). ~Iter maps onto the
            // same lazy-sequence lowering; Auto-side semantics are identical.
            let stream_or_iter = if matches!(&fn_decl.ret, Type::GenericInstance(inst)
                if matches!(inst.base_name.as_str(), "Stream" | "Iter"))
            {
                "impl futures::Stream<Item = "
            } else {
                "impl Iterator<Item = "
            };
            write!(sink.body, " -> {}{}>", stream_or_iter, inner)?;
        } else if !matches!(effective_ret_type, Type::Void) {
            let ret_str = if is_async_fn {
                match &effective_ret_type {
                    Type::Handle { task_type } => self.rust_return_type_name(task_type),
                    Type::GenericInstance(inst) if inst.base_name == "Future" => {
                        self.rust_return_type_name(inst.args.first().unwrap_or(&Type::Unknown))
                    }
                    other => self.rust_return_type_name(other),
                }
            } else {
                self.rust_return_type_name(&effective_ret_type)
            };
            write!(sink.body, " -> {}", ret_str)?;
        }

        // Function body
        write!(sink.body, " ")?;

        // Plan 321: Generator functions wrap body in async_stream::stream! {}
        // Plan musk-022: only wrap when the body actually yields. A `~Stream<T>`
        // function written as `return <expr>` (no yield) is a stream consumer
        // and must NOT be wrapped (return inside the macro wouldn't produce a stream).
        let body_yields = Self::scan_body_has_yield(&fn_decl.body);
        if is_generator_fn && body_yields {
            write!(sink.body, "{{ async_stream::stream! {{")?;
        }

        // Plan 387 §16: actor `fn main` lets in-flight messages process before
        // exit. No `__rt` binding or `drop(var)` injection — spawn helpers are
        // parameterless (track_join uses a thread-local registry) and `TaskRef`
        // RAII-drop closes mailboxes when main returns. drain_all just yields so
        // already-sent messages get processed before main exits.
        if is_main_actor {
            // No prologue needed (no __rt to declare).
            self.main_actor_prologue = None;
            self.main_actor_epilogue = Some("a2r_std::task::drain_all().await;\n".to_string());
        }

        // Plan 091: scope removed
        self.in_fn_body = true; // Plan 523 W3:fn 内全局 store 走写块形
                                // PLAN-724 T-06: HTTP 分派的 async 上下文窗口（async fn / async
                                // main / generator body 内 await 合法；同步 fn 内回落同步桥接面）。
        let prev_async_ctx = self
            .in_async_ctx
            .replace(is_async_fn || (is_generator_fn && body_yields));
        self.body(&fn_decl.body, sink, &effective_ret_type, "")?;
        self.in_async_ctx.set(prev_async_ctx);
        self.in_fn_body = false;
        // Plan 091: scope removed

        // Plan 387: clear actor prologue/epilogue after use.
        self.main_actor_prologue = None;
        self.main_actor_epilogue = None;

        // Plan 321: Close async_stream::stream! wrapper
        if is_generator_fn && body_yields {
            write!(sink.body, " }} }}")?;
        }

        Ok(())
    }

    // Type declaration (struct)
    pub(super) fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
        // Plan 013 (B1/BUG3): register this struct's name as locally-defined so
        // expression-position construction isn't spuriously crate-prefixed.
        self.local_struct_types.insert(type_decl.name.clone());
        // Register struct→spec mapping for spec array inference
        for spec_name in &type_decl.specs {
            self.struct_to_spec
                .insert(type_decl.name.clone(), spec_name.clone());
        }

        // Plan 373 G2 + Plan 379: `has Spec` generates a real `impl Trait for
        // Type` with method bodies (not a synthetic `{Name}Trait`) for ANY spec
        // known to this transpile — declared in this file (pre-scan) or
        // imported from a sibling module (both tracked in spec_decls). The
        // Plan 373 G2 hardcoded Tool/Role/Client/AgentFactory list is
        // generalized; the old `known_spec_traits` const is gone.
        // Emit doc comments
        if let Some(ref doc) = type_decl.doc {
            for line in doc.split('\n') {
                write!(sink.body, "/// {}\n", line)?;
            }
        }

        // Generate traits for composed types
        for has_type in &type_decl.has {
            if let Type::User(has_decl) = has_type {
                // Plan 379: a spec is "known" (real trait, skip the synthetic
                // {Name}Trait) iff it's in spec_decls (this file or sibling).
                let is_known = self.spec_decls.contains_key(has_decl.name.as_str());
                if is_known {
                    continue;
                }
                // Check if this type is already defined (has members or methods)
                let is_trait_only = has_decl.members.is_empty() && has_decl.methods.is_empty();

                // Generate trait definition
                // Use {Name}Trait to avoid conflict with struct name
                let trait_name = format!("{}Trait", has_decl.name);
                write!(sink.body, "trait {} {{\n", trait_name)?;
                self.indent();

                for method in &has_decl.methods {
                    // Generate method signature with &self
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "fn {}(&self", method.name)?;

                    // Parameters (skip self which is already added)
                    for (i, param) in method.params.iter().enumerate() {
                        write!(
                            sink.body,
                            ", {}: {}",
                            param.name,
                            self.rust_type_name(&param.ty)
                        )?;
                        if i < method.params.len() - 1 {
                            write!(sink.body, ", ")?;
                        }
                    }

                    // Return type
                    if !matches!(method.ret, Type::Void) {
                        write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                    } else {
                        write!(sink.body, ")")?;
                    }

                    write!(sink.body, ";\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n\n")?;

                // If this is a trait-only type (no struct definition), also generate a default impl
                if is_trait_only && !has_decl.methods.is_empty() {
                    let trait_name = format!("{}Trait", has_decl.name);
                    write!(sink.body, "impl {} for {} {{\n", trait_name, has_decl.name)?;
                    self.indent();

                    for method in &has_decl.methods {
                        self.print_indent(&mut sink.body)?;
                        write!(sink.body, "fn {}(&self", method.name)?;

                        // Parameters
                        for (i, param) in method.params.iter().enumerate() {
                            write!(
                                sink.body,
                                ", {}: {}",
                                param.name,
                                self.rust_type_name(&param.ty)
                            )?;
                            if i < method.params.len() - 1 {
                                write!(sink.body, ", ")?;
                            }
                        }

                        // Return type
                        if !matches!(method.ret, Type::Void) {
                            write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                        } else {
                            write!(sink.body, ")")?;
                        }

                        write!(sink.body, " {{\n")?;
                        self.indent();
                        self.print_indent(&mut sink.body)?;
                        write!(
                            sink.body,
                            "// Method implementation for {}\n",
                            has_decl.name
                        )?;
                        self.dedent();
                        self.print_indent(&mut sink.body)?;
                        write!(sink.body, "}}\n")?;
                    }

                    self.dedent();
                    write!(sink.body, "}}\n\n")?;
                }
            }
        }

        // Plan 159 Phase 6B-2: Output derive/serde attributes
        // Plan 204 Phase 2A: Add default #[derive(Clone, Debug, PartialEq)] if no attrs specified
        // T6: Add Eq, PartialOrd, Ord if no float/HashMap fields present
        if type_decl.attrs.is_empty() {
            // Recursively check field types for float/map/enum
            let has_float_field = type_decl
                .members
                .iter()
                .any(|m| Self::type_has_float(&m.ty));
            let has_map_field = type_decl.members.iter().any(|m| Self::type_has_map(&m.ty));
            // Enums don't derive Eq, so struct fields containing enum types can't derive Eq either
            // Also check nested types: List<EnumType>, Option<EnumType>, etc.
            let has_enum_field = type_decl
                .members
                .iter()
                .any(|m| Self::type_contains_enum(&m.ty));
            // Plan 384 A5: detect `dyn Trait` fields (incl. inside Arc<dyn T> /
            // Box<dyn T>). `dyn Trait` does not implement PartialEq/Eq/Ord, so
            // structs containing such fields must derive only Clone, Debug.
            let has_dyn_field = type_decl
                .members
                .iter()
                .any(|m| Self::type_contains_dyn(&m.ty));
            // Plan 387 follow-up: a `TaskRef<T>` field is move-only (RAII sole
            // owner — not Clone/Debug/PartialEq/Eq/Ord), so structs containing
            // one can only derive Debug.
            let has_taskref_field = type_decl
                .members
                .iter()
                .any(|m| Self::type_is_taskref(&m.ty));
            // PLAN-009 T1 (F1, 006 S2): the shape heuristics above grant a
            // comparison-trait set; intersect it with what the field types'
            // own derive surfaces actually provide. A field type that
            // explicitly derives only `Clone, Copy, Debug` must not end up
            // inside a container deriving `PartialEq, Eq, PartialOrd, Ord`
            // (rustc E0369). External/unknown field types impose no
            // constraint (unchanged behavior).
            let field_types: Vec<&Type> = type_decl.members.iter().map(|m| &m.ty).collect();
            let mask = self.field_cmp_mask(&field_types);
            let want = if has_taskref_field || has_dyn_field {
                0
            } else if has_float_field || has_map_field || has_enum_field {
                Self::D_PE
            } else {
                Self::D_FULL
            };
            let cmp = want & mask;
            if cmp != want {
                self.ord_restricted_names.insert(type_decl.name.clone());
            }
            if has_taskref_field {
                writeln!(sink.body, "#[derive(Debug)]")?;
            } else if has_dyn_field {
                // Plan 599 (004 §5④): a `dyn Trait` field only derives if the
                // trait's supertraits provide it — for unconstrained FOREIGN
                // traits (the engine shape, Box<dyn MasterPty + Send>) even
                // Clone/Debug fail (E0277). Emit no default derives; explicit
                // `#[derive(...)]` attrs passthrough covers bounded traits.
                self.ord_restricted_names.insert(type_decl.name.clone());
            } else {
                let mut traits = String::from("Clone, Debug");
                if cmp & Self::D_PE != 0 {
                    traits.push_str(", PartialEq");
                }
                if cmp & Self::D_EQ != 0 {
                    traits.push_str(", Eq");
                }
                if cmp & Self::D_PO != 0 {
                    traits.push_str(", PartialOrd");
                }
                if cmp & Self::D_OR != 0 {
                    traits.push_str(", Ord");
                }
                writeln!(sink.body, "#[derive({})]", traits)?;
            }
        } else {
            for attr in &type_decl.attrs {
                write!(sink.body, "#[{}]\n", attr)?;
            }
        }

        // Plan 163: Output pub prefix
        if type_decl.is_pub {
            write!(sink.body, "pub ")?;
        }

        // Track pub type context so methods inherit visibility
        self.inside_pub_type = type_decl.is_pub;

        // Struct definition with generic parameters
        write!(sink.body, "struct {}", type_decl.name)?;

        // Add generic parameters if present
        if !type_decl.generic_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in type_decl.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => {
                        write!(sink.body, "{}", tp.name)?;
                        // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                        if !tp.constraint.is_empty() {
                            write!(sink.body, ": ")?;
                            for (ci, ct) in tp.constraint.iter().enumerate() {
                                if ci > 0 {
                                    write!(sink.body, " + ")?;
                                }
                                write!(sink.body, "{}", self.rust_bound_name(ct))?;
                            }
                        }
                    }
                    GenericParam::Const(cp) => {
                        write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                    }
                }
            }
            write!(sink.body, ">")?;
        }

        write!(sink.body, " {{")?;

        // Collect all members (including from parent and composed types)
        // Use a set to avoid duplicates
        let mut all_members = Vec::new();
        let mut seen_fields = std::collections::HashSet::new();

        // First add members from parent type (inheritance)
        if let Some(ref parent_type) = type_decl.parent {
            if let Type::User(parent_decl) = parent_type.as_ref() {
                for member in &parent_decl.members {
                    if seen_fields.insert(member.name.clone()) {
                        all_members.push(member);
                    }
                }
            }
        }

        // Then add members from composed types
        for has_type in &type_decl.has {
            if let Type::User(has_decl) = has_type {
                for member in &has_decl.members {
                    if seen_fields.insert(member.name.clone()) {
                        all_members.push(member);
                    }
                }
            }
        }

        // Then add own members (can override inherited and composed ones)
        for member in &type_decl.members {
            if seen_fields.insert(member.name.clone()) {
                all_members.push(member);
            }
        }

        // Cache struct field names for positional arg mapping in struct_init
        let field_names: Vec<AutoStr> = all_members.iter().map(|m| m.name.clone()).collect();
        self.struct_fields
            .insert(type_decl.name.clone(), field_names);

        // Cache struct field types for .to_string() auto-insertion
        let field_types: Vec<(AutoStr, Type)> = all_members
            .iter()
            .map(|m| (m.name.clone(), m.ty.clone()))
            .collect();
        self.struct_field_types
            .insert(type_decl.name.clone(), field_types);

        // Plan 310 Phase 4.1: Reject direct self-referential struct fields.
        // Rust cannot represent `struct Node { next: Node }` (infinite size).
        // Direct Type::User(self_name) is rejected; indirect (List<Self>,
        // Option<Self>) is legal and gets a W0008 warning (Phase 4.2 below).
        let self_name = type_decl.name.clone();
        let has_direct_self = all_members
            .iter()
            .any(|m| matches!(&m.ty, Type::User(td) if td.name == self_name));
        if has_direct_self {
            return Err(AutoError::Msg(format!(
                "type '{}' contains a direct self-reference in a field; \
                 Rust cannot represent this without indirection. \
                 Consider using List<{}>, or restructuring to break the cycle.",
                type_decl.name, type_decl.name
            )));
        }

        // Plan 310 Phase 4.2: Warn about indirect self-reference (potential
        // reference cycle). E.g. `type Tree { children List<Tree> }` compiles
        // but may form an Rc/Arc cycle if shared — suggest Weak<T>.
        let has_indirect_self = all_members
            .iter()
            .any(|m| Self::type_contains_self_indirect(&m.ty, &self_name));
        if has_indirect_self {
            self.warnings.push(crate::error::Warning::RcCycle {
                name: self_name.to_string(),
                reason: "field contains indirect self-reference (e.g. List<Self>); \
                         shared references may form a cycle"
                    .to_string(),
                span: crate::error::span_from(0, 0),
            });
        }

        // Add delegation members to seen_fields and generate them separately
        for delegation in &type_decl.delegations {
            // PLAN-010 T2: cache delegation component fields for struct-literal
            // default-init (E0063 missing-field debt, 13_delegation 族).
            self.struct_delegation_types
                .entry(type_decl.name.clone())
                .or_default()
                .push((
                    delegation.member_name.clone(),
                    delegation.member_type.clone(),
                ));
            seen_fields.insert(delegation.member_name.clone());
        }

        if !all_members.is_empty() || !type_decl.delegations.is_empty() {
            sink.body.write(b"\n")?;
            self.indent();

            // First, write regular members
            for member in all_members {
                // Plan 163: Output per-field attributes
                for attr in &member.attrs {
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "#[{}]\n", attr)?;
                }
                self.print_indent(&mut sink.body)?;
                // Plan 013 (B16): Auto struct fields are public by design (no
                // Rust-style private fields). In standalone output, emit `pub`
                // so cross-module `p.models` field access works. (merge mode
                // keeps fields private — all access is intra-file.)
                let field_pub = if !self.merge_mode { "pub " } else { "" };
                // Plan 019 Phase 1: field names colliding with Rust keywords
                // (`type`) get raw-identifier escaping.
                write!(
                    sink.body,
                    "{}{}: {},",
                    field_pub,
                    Self::rust_ident(member.name.as_str()),
                    self.rust_type_name(&member.ty)
                )?;
                sink.body.write(b"\n")?;
            }

            // Then, write delegation members
            for delegation in &type_decl.delegations {
                self.print_indent(&mut sink.body)?;
                let field_pub = if !self.merge_mode { "pub " } else { "" };
                write!(
                    sink.body,
                    "{}{}: {},",
                    field_pub,
                    Self::rust_ident(delegation.member_name.as_str()),
                    self.rust_type_name(&delegation.member_type)
                )?;
                sink.body.write(b"\n")?;
            }

            self.dedent();
            self.print_indent(&mut sink.body)?;
        }

        sink.body.write(b"}\n")?;

        // Implement traits for composed types
        for has_type in &type_decl.has {
            // Plan 379: `has Spec` parses as Type::Spec when the spec is
            // declared in the same file (SpecDecl embedded) and as Type::User
            // when imported. Both get a real `impl <Spec> for <Type>` with
            // ACTUAL method bodies (generalizes Plan 373 G2's hardcoded
            // Tool/Role/Client/AgentFactory list to ANY spec known to this
            // transpile). The trait name is emitted plain (not
            // `crate::<module>::<Spec>`): the module's own `use` lines (e.g.
            // `use crate::fmod::{Formatter};`) resolve it, matching how
            // `Box<dyn Spec>` refs are emitted elsewhere.
            let known_spec: Option<(AutoStr, Vec<&Fn>)> = match has_type {
                Type::Spec(spec) => {
                    let spec = spec.borrow();
                    Some((
                        spec.name.clone(),
                        type_decl
                            .methods
                            .iter()
                            .filter(|m| spec.methods.iter().any(|s| s.name == m.name))
                            .collect(),
                    ))
                }
                Type::User(has_decl) => {
                    let spec_method_names: Vec<AutoStr> = self
                        .spec_decls
                        .get(has_decl.name.as_str())
                        .map(|ms| ms.iter().map(|s| s.name.clone()).collect())
                        .unwrap_or_default();
                    if spec_method_names.is_empty() {
                        None
                    } else {
                        Some((
                            has_decl.name.clone(),
                            type_decl
                                .methods
                                .iter()
                                .filter(|m| spec_method_names.contains(&m.name))
                                .collect(),
                        ))
                    }
                }
                _ => None,
            };

            if let Some((spec_name, spec_has_methods)) = known_spec {
                if spec_has_methods.is_empty() {
                    continue;
                }

                // Insert spec method names into the cache so they get filtered
                // out of the `impl Type` block (generated below).
                let method_names: Vec<SpecMethod> = spec_has_methods
                    .iter()
                    .map(|m| SpecMethod {
                        name: m.name.clone(),
                        params: m.params.clone(),
                        ret: m.ret.clone(),
                        body: None,
                    })
                    .collect();
                self.spec_decls.insert(spec_name.clone(), method_names);

                // Plan 373 G2: if any method is async (~Result lowered to
                // Future<Result<...>>), the trait uses #[async_trait] and the
                // impl block must carry it too.
                // Plan 382 (A.1): `!T` → Type::Result is SYNC — excluded.
                let has_async = spec_has_methods.iter().any(
                    |m| matches!(&m.ret, Type::GenericInstance(inst) if inst.base_name == "Future"),
                );
                if has_async {
                    write!(sink.body, "\n#[async_trait::async_trait]")?;
                }
                write!(sink.body, "\nimpl {}", spec_name)?;
                // Add generic parameters from the type declaration
                if !type_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in type_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => {
                                write!(sink.body, "{}", tp.name)?;
                                // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                                if !tp.constraint.is_empty() {
                                    write!(sink.body, ": ")?;
                                    for (ci, ct) in tp.constraint.iter().enumerate() {
                                        if ci > 0 {
                                            write!(sink.body, " + ")?;
                                        }
                                        write!(sink.body, "{}", self.rust_bound_name(ct))?;
                                    }
                                }
                            }
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }
                write!(sink.body, " for {}", type_decl.name)?;
                writeln!(sink.body, " {{")?;
                self.indent();

                for method in spec_has_methods {
                    // Plan 373 G2: trait impl methods must not have `pub`.
                    let saved_pub = self.inside_pub_type;
                    let saved_trait = self.in_trait_impl;
                    self.inside_pub_type = false;
                    self.in_trait_impl = true;
                    self.fn_decl(method, sink)?;
                    self.inside_pub_type = saved_pub;
                    self.in_trait_impl = saved_trait;
                    sink.body.write(b"\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n")?;
                continue;
            }

            // Unknown specs keep the synthetic {Name}Trait path. Same-file
            // specs parse as Type::Spec (handled above) and never reach here.
            if let Type::User(has_decl) = has_type {
                // Original path for unknown specs: synthetic {Name}Trait
                // Build the impl signature with generic parameters
                // Use {Name}Trait to avoid conflict with struct name
                let trait_name = format!("{}Trait", has_decl.name);
                write!(sink.body, "\nimpl {}", trait_name)?;

                // Add generic parameters from has_decl (trait)
                if !has_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in has_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => {
                                write!(sink.body, "{}", tp.name)?;
                                // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                                if !tp.constraint.is_empty() {
                                    write!(sink.body, ": ")?;
                                    for (ci, ct) in tp.constraint.iter().enumerate() {
                                        if ci > 0 {
                                            write!(sink.body, " + ")?;
                                        }
                                        write!(sink.body, "{}", self.rust_bound_name(ct))?;
                                    }
                                }
                            }
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                write!(sink.body, " for {}", type_decl.name)?;

                // Add generic parameters from type_decl (type)
                if !type_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in type_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => {
                                write!(sink.body, "{}", tp.name)?;
                                // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                                if !tp.constraint.is_empty() {
                                    write!(sink.body, ": ")?;
                                    for (ci, ct) in tp.constraint.iter().enumerate() {
                                        if ci > 0 {
                                            write!(sink.body, " + ")?;
                                        }
                                        write!(sink.body, "{}", self.rust_bound_name(ct))?;
                                    }
                                }
                            }
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                writeln!(sink.body, " {{")?;
                self.indent();

                for method in &has_decl.methods {
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "fn {}(&self", method.name)?;

                    // Parameters
                    for (i, param) in method.params.iter().enumerate() {
                        write!(
                            sink.body,
                            ", {}: {}",
                            param.name,
                            self.rust_type_name(&param.ty)
                        )?;
                        if i < method.params.len() - 1 {
                            write!(sink.body, ", ")?;
                        }
                    }

                    // Return type
                    if !matches!(method.ret, Type::Void) {
                        write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                    } else {
                        write!(sink.body, ")")?;
                    }

                    write!(sink.body, " {{\n")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    write!(
                        sink.body,
                        "// TODO: Implement {} method body from {}\n",
                        method.name, has_decl.name
                    )?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "}}\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n")?;
            }
        }

        // Generate trait implementations for delegations
        for delegation in &type_decl.delegations {
            let spec_name = delegation.spec_name.clone();
            let member_name = delegation.member_name.clone();

            // Plan 310 Phase 0.3: Resolve spec methods without depending on the
            // Database (which is empty in the single-file transpile_rust path).
            // Prefer the spec_decls cache populated during the pre-scan (handles
            // forward declarations); fall back to lookup_meta for multi-file/db.
            let spec_methods: Vec<SpecMethod> =
                if let Some(methods) = self.spec_decls.get(spec_name.as_str()) {
                    methods.clone()
                } else if let Some(meta) = self.lookup_meta(spec_name.as_str()) {
                    if let crate::scope::Meta::Spec(spec_decl) = meta.as_ref() {
                        spec_decl.methods.clone()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

            // Now generate the delegation impl if we found any spec methods
            if !spec_methods.is_empty() {
                write!(sink.body, "\nimpl {}", spec_name)?;

                write!(sink.body, " for {}", type_decl.name)?;

                // Add generic parameters from type_decl (type)
                if !type_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in type_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => {
                                write!(sink.body, "{}", tp.name)?;
                                // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                                if !tp.constraint.is_empty() {
                                    write!(sink.body, ": ")?;
                                    for (ci, ct) in tp.constraint.iter().enumerate() {
                                        if ci > 0 {
                                            write!(sink.body, " + ")?;
                                        }
                                        write!(sink.body, "{}", self.rust_bound_name(ct))?;
                                    }
                                }
                            }
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                writeln!(sink.body, " {{")?;
                self.indent();

                // Generate methods that delegate to the member
                for spec_method in &spec_methods {
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "fn {}(&self", spec_method.name)?;

                    // Parameters
                    for param in &spec_method.params {
                        write!(
                            sink.body,
                            ", {}: {}",
                            param.name,
                            self.rust_param_type_name(&param.ty)
                        )?;
                    }

                    // Return type
                    if !matches!(spec_method.ret, Type::Void) {
                        write!(
                            sink.body,
                            ") -> {}",
                            self.rust_return_type_name(&spec_method.ret)
                        )?;
                    } else {
                        write!(sink.body, ")")?;
                    }

                    write!(sink.body, " {{\n")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "self.{}.{}(", member_name, spec_method.name)?;

                    // Forward parameters
                    for (i, param) in spec_method.params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        write!(sink.body, "{}", param.name)?;
                    }

                    write!(sink.body, ")\n")?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "}}\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n")?;
            }
        }

        // Generate impl block with own methods (excluding spec methods)
        // Collect spec method names to avoid duplication in impl Type block.
        // Check both type_decl.specs (from `as Spec`) and type_decl.has entries
        // (from `has Spec`), because the parser may use different fields.
        let spec_method_names: HashSet<AutoStr> = {
            let mut names = HashSet::new();
            for spec_name in &type_decl.specs {
                if let Some(methods) = self.spec_decls.get(spec_name) {
                    for m in methods {
                        names.insert(m.name.clone());
                    }
                }
            }
            for has_type in &type_decl.has {
                match has_type {
                    Type::User(has_decl) => {
                        if let Some(methods) = self.spec_decls.get(&has_decl.name) {
                            for m in methods {
                                names.insert(m.name.clone());
                            }
                        }
                    }
                    // Plan 379: same-file specs parse as Type::Spec — their
                    // implemented methods are also excluded from the inherent
                    // `impl Type` block (they live in the trait impl instead).
                    Type::Spec(spec) => {
                        let spec_name = spec.borrow().name.clone();
                        if let Some(methods) = self.spec_decls.get(&spec_name) {
                            for m in methods {
                                names.insert(m.name.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            names
        };

        let own_methods: Vec<_> = type_decl
            .methods
            .iter()
            .filter(|m| !spec_method_names.contains(&m.name))
            .collect();

        // C8: emit the inherent impl also when there are associated consts
        // (even with zero methods).
        if !own_methods.is_empty() || !type_decl.consts.is_empty() {
            sink.body.write(b"\n")?;
            // Plan 364 W1: impl-level attribute macros (#[zbus::interface]) before `impl Type {`
            for attr in &type_decl.impl_attrs {
                write!(sink.body, "#[{}]\n", attr)?;
            }
            write!(sink.body, "impl {}", type_decl.name)?;

            // Add generic parameters if present
            if !type_decl.generic_params.is_empty() {
                write!(sink.body, "<")?;
                for (i, param) in type_decl.generic_params.iter().enumerate() {
                    if i > 0 {
                        write!(sink.body, ", ")?;
                    }
                    match param {
                        GenericParam::Type(tp) => {
                            write!(sink.body, "{}", tp.name)?;
                            // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                            if !tp.constraint.is_empty() {
                                write!(sink.body, ": ")?;
                                for (ci, ct) in tp.constraint.iter().enumerate() {
                                    if ci > 0 {
                                        write!(sink.body, " + ")?;
                                    }
                                    write!(sink.body, "{}", self.rust_bound_name(ct))?;
                                }
                            }
                        }
                        GenericParam::Const(cp) => {
                            write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                        }
                    }
                }
                write!(sink.body, ">")?;
            }

            writeln!(sink.body, " {{")?;
            self.indent();

            // C8: associated consts (`[pub] const NAME TYPE = value`)
            for c in &type_decl.consts {
                self.print_indent(&mut sink.body)?;
                let ty_name = if matches!(c.ty, Type::StrFixed(_) | Type::StrSlice | Type::StrOwned)
                {
                    "&str".to_string()
                } else {
                    self.rust_type_name(&c.ty)
                };
                if c.is_pub {
                    write!(sink.body, "pub const {}: {} = ", c.name, ty_name)?;
                } else {
                    write!(sink.body, "const {}: {} = ", c.name, ty_name)?;
                }
                self.expr(&c.expr, &mut sink.body)?;
                sink.body.write(b";\n")?;
            }

            // Plan 514 W3: 预计算传递性可变方法集,供 fn_decl 接收者判定。
            self.current_type_mut_methods = Some(Self::compute_type_mut_methods(&own_methods));
            if let Some(s) = &self.current_type_mut_methods {
                self.all_mut_method_names.extend(s.iter().cloned());
            }
            self.current_impl_type = Some(type_decl.name.clone());
            for method in &own_methods {
                self.fn_decl(method, sink)?;
                sink.body.write(b"\n")?;
            }
            self.current_impl_type = None;
            self.current_type_mut_methods = None;

            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}\n")?;
        }

        // Reset pub type context
        self.inside_pub_type = false;

        // Generate trait implementations for specs
        if !type_decl.specs.is_empty() {
            // Plan 359 DIV-TRAIT-A2R-2: spec_impls carries the *concrete* type
            // arguments for generic spec impls (e.g. `as Comparable<i32>`).
            // Index them by spec_name so the impl generator can emit
            // `impl Comparable<i32>` instead of falling back to the trait's
            // declared generic params (`impl Comparable<T>`) or dropping them.
            let concrete_args: std::collections::HashMap<&str, &[Type]> = type_decl
                .spec_impls
                .iter()
                .map(|si| (si.spec_name.as_str(), si.type_args.as_slice()))
                .collect();

            // Collect spec declarations: prefer local cache, fallback to database lookup
            let spec_decls: Vec<_> = type_decl
                .specs
                .iter()
                .filter_map(|spec_name| {
                    // Plan 159 6B-2.2: Use cached spec methods first
                    if let Some(methods) = self.spec_decls.get(spec_name) {
                        Some(SpecDecl::new(spec_name.clone(), methods.clone()))
                    } else if let Some(meta) = self.lookup_meta(spec_name) {
                        if let crate::scope::Meta::Spec(spec_decl) = meta.as_ref() {
                            Some(spec_decl.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // Generate impl block for each spec (only if type provides matching methods)
            for spec_decl in spec_decls {
                // Collect matching methods first — skip empty impls
                let matched_methods: Vec<_> = spec_decl
                    .methods
                    .iter()
                    .filter(|spec_method| {
                        type_decl.methods.iter().any(|m| m.name == spec_method.name)
                    })
                    .collect();

                if matched_methods.is_empty() {
                    continue; // Skip empty impl blocks
                }

                // Plan 380 P7: if any matched method is async (~T → Future),
                // the impl block needs #[async_trait] (same as trait decl).
                let has_async = matched_methods.iter().any(
                    |m| matches!(&m.ret, Type::GenericInstance(inst) if inst.base_name == "Future"),
                );

                sink.body.write(b"\n")?;

                if has_async {
                    write!(sink.body, "#[async_trait::async_trait]\n")?;
                }

                // Build impl signature with generic parameters
                write!(sink.body, "impl {}", spec_decl.name)?;

                // Plan 359 DIV-TRAIT-A2R-2: prefer concrete type args from
                // spec_impls (`as Comparable<i32>`); fall back to the trait's
                // declared generic params (`Comparable<T>`) for non-concrete
                // impls (`as Storage<T>`).
                if let Some(args) = concrete_args.get(spec_decl.name.as_str()) {
                    if !args.is_empty() {
                        write!(sink.body, "<")?;
                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 {
                                write!(sink.body, ", ")?;
                            }
                            write!(sink.body, "{}", self.rust_type_name(arg))?;
                        }
                        write!(sink.body, ">")?;
                    }
                } else if !spec_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in spec_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => {
                                write!(sink.body, "{}", tp.name)?;
                                // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                                if !tp.constraint.is_empty() {
                                    write!(sink.body, ": ")?;
                                    for (ci, ct) in tp.constraint.iter().enumerate() {
                                        if ci > 0 {
                                            write!(sink.body, " + ")?;
                                        }
                                        write!(sink.body, "{}", self.rust_bound_name(ct))?;
                                    }
                                }
                            }
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                write!(sink.body, " for {}", type_decl.name)?;

                // Add generic parameters from type_decl (type)
                if !type_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in type_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => {
                                write!(sink.body, "{}", tp.name)?;
                                // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                                if !tp.constraint.is_empty() {
                                    write!(sink.body, ": ")?;
                                    for (ci, ct) in tp.constraint.iter().enumerate() {
                                        if ci > 0 {
                                            write!(sink.body, " + ")?;
                                        }
                                        write!(sink.body, "{}", self.rust_bound_name(ct))?;
                                    }
                                }
                            }
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                writeln!(sink.body, " {{")?;
                self.indent();

                // Plan 417-E2 (DIV-TRAIT-LANG-1): emit the associated-type
                // bindings from the impl clause (`as Container<Item=int>`)
                // as Rust associated-type assignments (`type Item = i64;`).
                // They must precede the methods in the impl block.
                if let Some(si) = type_decl
                    .spec_impls
                    .iter()
                    .find(|si| si.spec_name == spec_decl.name)
                {
                    for (assoc_name, assoc_ty) in &si.assoc_bindings {
                        self.print_indent(&mut sink.body)?;
                        writeln!(
                            sink.body,
                            "type {} = {};",
                            assoc_name,
                            self.rust_type_name(assoc_ty)
                        )?;
                    }
                }

                // Generate matched methods
                for spec_method in &matched_methods {
                    // Find the implementation in type_decl
                    if let Some(method) = type_decl
                        .methods
                        .iter()
                        .find(|m| m.name == spec_method.name)
                    {
                        self.print_indent(&mut sink.body)?;

                        // Plan 380 P6: detect async method (~T → Future<T> ret).
                        // trait impl methods need `async fn` + unwrapped return type,
                        // matching the trait declaration (fn_decl does this, but this
                        // path generates the signature manually — it was hardcoded to
                        // `fn` without async, producing `fn execute() -> Future<T>`
                        // while the trait declares `async fn execute() -> T`).
                        let method_is_async = matches!(&method.ret, Type::GenericInstance(inst) if inst.base_name == "Future");

                        // Method signature
                        if method_is_async {
                            write!(sink.body, "async ")?;
                        }
                        write!(sink.body, "fn {}(&self", method.name)?;

                        // Parameters
                        for param in &method.params {
                            write!(
                                sink.body,
                                ", {}: {}",
                                param.name,
                                self.rust_param_type_name(&param.ty)
                            )?;
                        }

                        // Return type — unwrap Future<T> → T for async fn
                        if !matches!(method.ret, Type::Void) {
                            let ret_str = if method_is_async {
                                match &method.ret {
                                    Type::GenericInstance(inst) if inst.base_name == "Future" => {
                                        self.rust_return_type_name(
                                            inst.args.first().unwrap_or(&Type::Unknown),
                                        )
                                    }
                                    other => self.rust_return_type_name(other),
                                }
                            } else {
                                self.rust_return_type_name(&method.ret)
                            };
                            write!(sink.body, ") -> {}", ret_str)?;
                        } else {
                            write!(sink.body, ")")?;
                        }

                        // Generate method body (body() writes its own { })
                        write!(sink.body, " ")?;
                        self.body(&method.body, sink, &method.ret, "")?;
                        writeln!(sink.body)?;
                    }
                }

                self.dedent();
                writeln!(sink.body, "}}")?;
            }
        }

        Ok(())
    }

    // Enum declaration
    pub(super) fn enum_decl(&mut self, enum_decl: &EnumDecl, sink: &mut Sink) -> AutoResult<()> {
        // Cache enum name as tag type for construction detection
        self.tag_types.insert(enum_decl.name.clone());
        self.known_enum_names.insert(enum_decl.name.clone());

        // Emit doc comments
        if let Some(ref doc) = enum_decl.doc {
            for line in doc.split('\n') {
                write!(sink.body, "/// {}\n", line)?;
            }
        }

        // Plan 204 Phase 2C: Add #[derive(Clone, Debug, PartialEq)] to enums
        // Scalar enums with repr type also need Copy
        // Heterogeneous enums with all-empty variants (no data) also get Copy
        //
        // Plan 013 (B1/BUG1): enums additionally derive Eq, PartialOrd, Ord when
        // every payload type is Eq-safe (no float, no Map, no nested enum/tag).
        // This mirrors the struct derive logic at the TypeDecl handler — fieldless
        // enums (e.g. ModelTier) are trivially Eq-safe, and downstream code
        // commonly compares enum values (`m.tier == desired`), which needs Eq.
        // The old comment "Enums don't derive Eq" only holds for float-bearing
        // heterogeneous enums, not the common fieldless/scalar case.
        let all_variants_empty = matches!(&enum_decl.kind, EnumKind::Heterogeneous { .. })
            && enum_decl.items.iter().all(|item| {
                item.fields.is_empty()
                    && item.payload_type.is_none()
                    && item.payload_types.is_empty()
            });

        // Collect every payload Type the enum carries (Homogeneous shared type,
        // Heterogeneous per-variant payloads, struct-variant fields). Scalar
        // enums carry none.
        let mut payload_types: Vec<&Type> = Vec::new();
        if let EnumKind::Homogeneous { payload_type } = &enum_decl.kind {
            payload_types.push(payload_type);
        }
        for item in &enum_decl.items {
            if let Some(pt) = &item.payload_type {
                payload_types.push(pt);
            }
            for pt in &item.payload_types {
                payload_types.push(pt);
            }
            for f in &item.fields {
                payload_types.push(&f.field_type);
            }
        }

        let payload_is_eq_safe = payload_types.iter().all(|ty| {
            !Self::type_has_float(ty) && !Self::type_has_map(ty) && !Self::type_contains_enum(ty)
        });

        let derive_attrs = match &enum_decl.kind {
            EnumKind::Scalar { repr_type: Some(_) } if payload_is_eq_safe => {
                // Plan 577 ③ (DEBTS 016 Phase 0): +Hash — enums as HashMap
                // keys are the map-index dispatch's natural consumer; Eq-safe
                // payload coverage implies Hash-safe (same exclusions).
                "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]"
            }
            EnumKind::Scalar { repr_type: Some(_) } => "#[derive(Clone, Debug, PartialEq, Copy)]",
            // Plan 447 ③-P1: plain scalars are all-unit by definition —
            // enum values passed by value to helpers then reused
            // (op_display(op) + binop_result(op)) are E0382 without Copy
            // (lib TokenKind/Op/OpCode are all this shape).
            EnumKind::Scalar { repr_type: None } if payload_is_eq_safe => {
                "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]"
            }
            EnumKind::Scalar { repr_type: None } => "#[derive(Clone, Copy, Debug, PartialEq)]",
            _ if all_variants_empty && payload_is_eq_safe => {
                "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]"
            }
            _ if all_variants_empty => "#[derive(Clone, Copy, Debug, PartialEq)]",
            _ if payload_is_eq_safe => {
                "#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]"
            }
            _ => "#[derive(Clone, Debug, PartialEq)]",
        };
        // Plan 376: If the user supplied explicit attrs (e.g. `#[derive(Debug)]`),
        // emit them verbatim instead of the auto-generated derive. Mirrors the
        // TypeDecl (struct) handler. Needed when foreign payload types (e.g.
        // `ClientError`) don't impl Clone/PartialEq, which the default derive
        // requires — the user opts into the conservative `#[derive(Debug)]`.
        if !enum_decl.attrs.is_empty() {
            for attr in &enum_decl.attrs {
                write!(sink.body, "#[{}]\n", attr)?;
            }
        } else {
            // PLAN-009 T1 (F1, 006 S2): restrict comparison traits by the
            // payload types' explicit derive surfaces (E0369 guard; mirrors
            // the TypeDecl handler).
            let mask = self.field_cmp_mask(&payload_types);
            if mask != Self::D_FULL {
                let filtered = Self::filter_cmp_traits(derive_attrs, mask);
                if filtered != derive_attrs && filtered.contains("PartialEq") {
                    self.ord_restricted_names.insert(enum_decl.name.clone());
                }
                writeln!(sink.body, "{}", filtered)?;
            } else {
                writeln!(sink.body, "{}", derive_attrs)?;
            }
        }

        // Plan 163: Output pub prefix
        if enum_decl.is_pub {
            sink.body.write(b"pub ")?;
        }
        self.inside_pub_type = enum_decl.is_pub;

        match &enum_decl.kind {
            EnumKind::Scalar { .. } => {
                // C-style scalar enum: emit Rust enum with values + Display impl
                sink.body.write(b"enum ")?;
                sink.body.write(enum_decl.name.as_bytes())?;
                sink.body.write(b" {\n")?;
                self.indent();

                for (_i, item) in enum_decl.items.iter().enumerate() {
                    // Plan 018 §Phase 3.3: variant-level attrs (`#[default]`)
                    // were silently dropped for scalar enums → derive(Default)
                    // on the enum without any `#[default]` variant is E0665.
                    // Emit them so `#[default]` reaches the Rust enum.
                    for attr in &item.attrs {
                        self.print_indent(&mut sink.body)?;
                        write!(sink.body, "#[{}]\n", attr)?;
                    }
                    self.print_indent(&mut sink.body)?;
                    sink.body
                        .write(format!("{} = {},", item.name, item.value()).as_bytes())?;
                    sink.body.write(b"\n")?;
                }

                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}\n")?;

                // Generate Display trait implementation
                sink.body.write(b"\n")?;
                writeln!(
                    sink.body,
                    "impl std::fmt::Display for {} {{",
                    enum_decl.name
                )?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                writeln!(
                    sink.body,
                    "fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{"
                )?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "match self {{")?;
                self.indent();

                for item in &enum_decl.items {
                    self.print_indent(&mut sink.body)?;
                    writeln!(
                        sink.body,
                        "{}::{} => write!(f, \"{}\"),",
                        enum_decl.name, item.name, item.name
                    )?;
                }

                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                self.dedent();
                writeln!(sink.body, "}}")?;

                // Generate from_id() method: EnumType::from_id(name) → Option<EnumType>
                writeln!(sink.body, "impl {} {{", enum_decl.name)?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "pub fn from_id(id: &str) -> Self {{")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "match id {{")?;
                self.indent();
                for item in &enum_decl.items {
                    self.print_indent(&mut sink.body)?;
                    writeln!(
                        sink.body,
                        "\"{}\" | \"{}\" => {}::{},",
                        item.name,
                        item.name.to_lowercase(),
                        enum_decl.name,
                        item.name.clone()
                    )?;
                }
                self.print_indent(&mut sink.body)?;
                writeln!(
                    sink.body,
                    "_ => {}::{}",
                    enum_decl.name,
                    enum_decl
                        .items
                        .first()
                        .map(|i| i.name.as_str())
                        .unwrap_or("Unknown")
                )?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                self.dedent();
                writeln!(sink.body, "}}")?;
            }
            EnumKind::Homogeneous { payload_type } => {
                // Generate Rust enum where all variants wrap the same type
                write!(sink.body, "enum {}", enum_decl.name)?;
                writeln!(sink.body, " {{")?;
                self.indent();
                for item in &enum_decl.items {
                    self.print_indent(&mut sink.body)?;
                    writeln!(
                        sink.body,
                        "{}({}),",
                        item.name,
                        self.rust_type_name(payload_type)
                    )?;
                }
                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                sink.body.write(b"\n")?;
            }
            EnumKind::Heterogeneous { generic_params, .. } => {
                // Plan 204 Phase 2C: Generate heterogeneous enum directly
                // Supports both single-payload tuple variants and multi-field struct variants
                write!(sink.body, "enum {}", enum_decl.name)?;

                // Add generic parameters if present
                if !generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => {
                                write!(sink.body, "{}", tp.name)?;
                                // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                                if !tp.constraint.is_empty() {
                                    write!(sink.body, ": ")?;
                                    for (ci, ct) in tp.constraint.iter().enumerate() {
                                        if ci > 0 {
                                            write!(sink.body, " + ")?;
                                        }
                                        write!(sink.body, "{}", self.rust_bound_name(ct))?;
                                    }
                                }
                            }
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                writeln!(sink.body, " {{")?;
                self.indent();

                for item in &enum_decl.items {
                    // Clear any hardcoded struct-variant seed for this variant
                    // (seed_known_struct_enum_variants, line ~11278). The real
                    // declaration below is authoritative: if the .at declares a
                    // tuple variant `Text(str)`, the seed's struct entry
                    // (`Text { text }`) must NOT survive — otherwise construction
                    // (call(), ~line 5705/6520) emits struct syntax for a tuple
                    // variant. Removing here makes the source declaration win.
                    let item_key = (enum_decl.name.clone(), item.name.clone());
                    self.enum_struct_variants.remove(&item_key);

                    self.print_indent(&mut sink.body)?;
                    if item.has_fields() {
                        // Register struct variant for pattern matching
                        let field_names: Vec<AutoStr> =
                            item.fields.iter().map(|f| f.name.clone()).collect();
                        self.enum_struct_variants
                            .insert((enum_decl.name.clone(), item.name.clone()), field_names);
                        // Multi-field struct variant: Name { field1: Type1, field2: Type2 }
                        write!(sink.body, "{} {{ ", item.name)?;
                        for (j, field) in item.fields.iter().enumerate() {
                            if j > 0 {
                                write!(sink.body, ", ")?;
                            }
                            write!(
                                sink.body,
                                "{}: {}",
                                field.name,
                                self.rust_type_name(&field.field_type)
                            )?;
                        }
                        writeln!(sink.body, " }},")?;
                    } else if item.has_tuple_payload() {
                        // Register tuple variant for bare-match detection
                        self.enum_tuple_variants.insert(
                            (enum_decl.name.clone(), item.name.clone()),
                            item.payload_types.len(),
                        );
                        // Cache tuple variant field types for .to_string() auto-insertion
                        self.enum_tuple_field_types.insert(
                            (enum_decl.name.clone(), item.name.clone()),
                            item.payload_types.clone(),
                        );
                        // Multi-arg tuple variant: ToolUse str str str → ToolUse(String, String, String)
                        write!(sink.body, "{}(", item.name)?;
                        for (j, pt) in item.payload_types.iter().enumerate() {
                            if j > 0 {
                                write!(sink.body, ", ")?;
                            }
                            write!(sink.body, "{}", self.rust_type_name(pt))?;
                        }
                        writeln!(sink.body, "),")?;
                    } else if let Some(ref payload) = item.payload_type {
                        // Register single-payload tuple variant
                        self.enum_tuple_variants
                            .insert((enum_decl.name.clone(), item.name.clone()), 1);
                        // Cache single-payload type for .to_string() auto-insertion
                        self.enum_tuple_field_types.insert(
                            (enum_decl.name.clone(), item.name.clone()),
                            vec![payload.clone()],
                        );
                        // Single-payload tuple variant: Name(Type)
                        writeln!(
                            sink.body,
                            "{}({}),",
                            item.name,
                            self.rust_type_name(payload)
                        )?;
                    } else {
                        // Unit variant (no data): Name
                        writeln!(sink.body, "{},", item.name)?;
                    }
                }

                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                sink.body.write(b"\n")?;

                // Plan 382: variant `#[from]` attributes → From conversion impls.
                // Makes `?` on Result<_, Payload> auto-convert to EnumName (the
                // `?` operator only needs `From`, not thiserror). Only single-
                // payload variants qualify (a From impl maps one value).
                for item in &enum_decl.items {
                    if !item.attrs.iter().any(|a| a.as_str() == "from") {
                        continue;
                    }
                    let Some(payload) = &item.payload_type else {
                        continue;
                    };
                    let pty = self.rust_type_name(payload);
                    writeln!(sink.body, "impl From<{}> for {} {{", pty, enum_decl.name)?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "fn from(e: {}) -> Self {{", pty)?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "{}::{}(e)", enum_decl.name, item.name)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "}}")?;
                    self.dedent();
                    writeln!(sink.body, "}}")?;
                    sink.body.write(b"\n")?;
                }

                // For heterogeneous enums that are all unit variants (like SpecStatus with methods),
                // generate Display and from_id similar to scalar enums
                let all_unit = enum_decl.items.iter().all(|item| {
                    item.payload_type.is_none()
                        && item.payload_types.is_empty()
                        && !item.has_fields()
                });
                if all_unit {
                    // Display impl
                    writeln!(
                        sink.body,
                        "impl std::fmt::Display for {} {{",
                        enum_decl.name
                    )?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(
                        sink.body,
                        "fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{"
                    )?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "match self {{")?;
                    self.indent();
                    for item in &enum_decl.items {
                        self.print_indent(&mut sink.body)?;
                        writeln!(
                            sink.body,
                            "{}::{} => write!(f, \"{}\"),",
                            enum_decl.name, item.name, item.name
                        )?;
                    }
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "}}")?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "}}")?;
                    self.dedent();
                    writeln!(sink.body, "}}")?;

                    // from_id impl
                    writeln!(sink.body, "impl {} {{", enum_decl.name)?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "pub fn from_id(id: &str) -> Self {{")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "match id {{")?;
                    self.indent();
                    for item in &enum_decl.items {
                        self.print_indent(&mut sink.body)?;
                        writeln!(
                            sink.body,
                            "\"{}\" | \"{}\" => {}::{},",
                            item.name,
                            item.name.to_lowercase(),
                            enum_decl.name,
                            item.name
                        )?;
                    }
                    self.print_indent(&mut sink.body)?;
                    let first = enum_decl
                        .items
                        .first()
                        .map(|i| i.name.as_str())
                        .unwrap_or("Unknown");
                    writeln!(sink.body, "_ => {}::{}", enum_decl.name, first)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "}}")?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "}}")?;
                    self.dedent();
                    writeln!(sink.body, "}}")?;
                }
            }
        }

        self.inside_pub_type = false;
        Ok(())
    }
    // ---- PLAN-884 T-06：自 rust.rs 原样迁入 ----

    // **Phase 6: Generic Programming**
    // Type alias declaration
    pub(super) fn type_alias_decl(
        &mut self,
        type_alias: &TypeAlias,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        // Generate type alias: type List<T> = List<T, Heap>;
        // In Rust: type List<T> = List<T, Heap>;
        write!(sink.body, "type {}", type_alias.name)?;

        // Type parameters
        if !type_alias.params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in type_alias.params.iter().enumerate() {
                write!(sink.body, "{}", param)?;
                if i < type_alias.params.len() - 1 {
                    write!(sink.body, ", ")?;
                }
            }
            write!(sink.body, ">")?;
        }

        // For the target type, if it's a GenericInstance with Unknown args,
        // we need to use the type parameter names instead of "Unknown"
        if let Type::GenericInstance(inst) = &type_alias.target {
            write!(sink.body, " = {}<", inst.base_name)?;
            // Use type parameters if available, otherwise use Unknown count
            let args: Vec<String> = if !type_alias.params.is_empty() {
                type_alias.params.iter().map(|p| p.to_string()).collect()
            } else {
                inst.args
                    .iter()
                    .map(|t| match t {
                        Type::Unknown => "_".to_string(),
                        _ => self.rust_type_name(t),
                    })
                    .collect()
            };
            write!(sink.body, "{}>;", args.join(", "))?;
        } else {
            write!(sink.body, " = {};", self.rust_type_name(&type_alias.target))?;
        }
        sink.body.write(b"\n")?;

        Ok(())
    }

    /// Convert a Heterogeneous EnumDecl to a Tag for reusing tag code generation.
    #[allow(dead_code)]
    fn enum_decl_to_tag(enum_decl: &EnumDecl) -> Tag {
        let fields: Vec<TagField> = enum_decl
            .items
            .iter()
            .map(|item| TagField {
                name: item.name.clone().into(),
                ty: item.payload_type.clone().unwrap_or(Type::Void),
            })
            .collect();
        let (generic_params, methods) = match &enum_decl.kind {
            EnumKind::Heterogeneous {
                generic_params,
                methods,
            } => (generic_params.clone(), methods.clone()),
            _ => (vec![], vec![]),
        };
        Tag {
            name: enum_decl.name.clone().into(),
            generic_params,
            fields,
            methods,
        }
    }

    // **Phase 1.2: Union Types (test: 013_union)**
    pub(super) fn union_decl(&mut self, union: &Union, sink: &mut Sink) -> AutoResult<()> {
        // Cache union type name so construction and field-access sites can be
        // rewritten to safe accessor methods (Plan 310 Phase 0.2).
        // In Rust, union field access/construct is unsafe; we wrap it.
        self.union_types.insert(union.name.clone());

        // Generate union definition
        writeln!(sink.body, "union {} {{", union.name)?;
        self.indent();

        for field in &union.fields {
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "{}: {},",
                field.name,
                self.rust_type_name(&field.ty)
            )?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;

        // Plan 310 Phase 0.2: Generate safe accessor methods for each field.
        // Union field construction (`Union { f: v }`) and read (`u.f`) require
        // `unsafe` in Rust. We expose `new_<f>(v)` constructors and `<f>()`
        // readers so generated code stays in safe Rust.
        write!(sink.body, "impl {} {{", union.name)?;
        sink.body.write(b"\n")?;
        self.indent();
        for field in &union.fields {
            let fname = field.name.as_str();
            let fty = self.rust_type_name(&field.ty);
            // Constructor: fn new_<f>(v: T) -> Self { unsafe { Self { f: v } } }
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "pub fn new_{}(value: {}) -> Self {{ unsafe {{ Self {{ {}: value }} }} }}",
                fname, fty, fname
            )?;
            // Reader: fn <f>(&self) -> T { unsafe { self.f } }
            // For non-Copy field types (e.g. String) reading is unsafe-by-copy;
            // we only emit readers for Copy-like field types to avoid footguns.
            if Self::is_copy_type(&field.ty) {
                self.print_indent(&mut sink.body)?;
                writeln!(
                    sink.body,
                    "pub fn {}(&self) -> {} {{ unsafe {{ self.{} }} }}",
                    fname, fty, fname
                )?;
            }
        }
        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;

        Ok(())
    }

    // **Phase 1.3: Tag Types (test: 014_tag)**
    pub(super) fn tag_decl(&mut self, tag: &Tag, sink: &mut Sink) -> AutoResult<()> {
        // Cache tag type name for tag construction detection
        self.tag_types.insert(tag.name.clone());

        // Generate enum definition for tag
        // AutoLang tags are algebraic data types that map to Rust enums
        write!(sink.body, "enum {}", tag.name)?;

        // Add generic parameters if present
        if !tag.generic_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in tag.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => {
                        write!(sink.body, "{}", tp.name)?;
                        // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                        if !tp.constraint.is_empty() {
                            write!(sink.body, ": ")?;
                            for (ci, ct) in tp.constraint.iter().enumerate() {
                                if ci > 0 {
                                    write!(sink.body, " + ")?;
                                }
                                write!(sink.body, "{}", self.rust_bound_name(ct))?;
                            }
                        }
                    }
                    GenericParam::Const(cp) => {
                        write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                    }
                }
            }
            write!(sink.body, ">")?;
        }

        writeln!(sink.body, " {{")?;
        self.indent();

        for field in &tag.fields {
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "{}({}),",
                field.name,
                self.rust_type_name(&field.ty)
            )?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;
        sink.body.write(b"\n")?;

        // TODO: Generate impl block for tag methods (if any)
        for method in &tag.methods {
            // Tag methods will be added here
            let _ = method;
        }

        Ok(())
    }
}
