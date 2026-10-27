//! PLAN-843 结构拆分（自 codegen.rs 原样搬移；算法/行为零变化）。
use super::*;

impl Codegen {
    pub fn compile_stmt(&mut self, stmt: &Stmt) -> AutoResult<()> {
        self.assembly_compile_depth += 1;
        let result = self.compile_stmt_inner(stmt);
        self.assembly_compile_depth -= 1;
        result?;
        if self.assembly_compile_depth == 0 {
            self.verify_assembly_references()?;
        }
        Ok(())
    }

    fn compile_stmt_inner(&mut self, stmt: &Stmt) -> AutoResult<()> {
        match stmt {
            Stmt::Expr(expr) => {
                // Plan 364 Step 5: Config-mode accumulation. In config_mode,
                // a block-level Pair/Node/Object is accumulated into the
                // enclosing node container (set prop / add kid / merge fields)
                // rather than being left on the stack and popped (Script).
                if self.config_mode {
                    return self.compile_config_stmt_expr(expr);
                }
                self.compile_expr(expr)?;
                // Plan 089: Evaluate and discard result if this is not the last expression
                // of a block or script. This keeps the stack clean for subsequent ops.
                // User-defined void functions (CALL+RET) leave a return value on the stack,
                // so POP is needed. Native void shims (CALL_NAT) don't push return values,
                // so POP must be skipped to avoid eating a value below.
                // Plan 348 C4: a native void shim (last_was_native_void) NEVER pushes a
                // value, so we must never POP in that case regardless of last_expr_type.
                // The previous `(!Void || !native_void)` OR-logic popped whenever the
                // type was non-void even when a native shim left nothing on the stack,
                // corrupting the stack (eating a value from below). Now: pop iff we
                // should discard AND the last call was not a native void shim.
                let needs_pop = self.should_pop_expr_result && !self.last_was_native_void;
                if needs_pop {
                    if matches!(self.last_expr_type, ObjectType::Double | ObjectType::Uint) {
                        self.emit(OpCode::POP_N);
                        self.code.push(2);
                    } else {
                        self.emit(OpCode::POP);
                    }
                }
            }
            Stmt::Block(body) => {
                // Plan 118 Phase 5: Blocks create a new scope for variable shadowing
                // Variables declared inside the block are scoped to the block
                self.push_scope();

                let n = body.stmts.len();
                for (i, s) in body.stmts.iter().enumerate() {
                    // Plan 199: Emit SOURCE_LINE for debugging
                    if i < body.source_lines.len() {
                        self.emit_source_line(body.source_lines[i]);
                    }
                    let is_last = i == n - 1;
                    let old_pop = self.should_pop_expr_result;
                    if is_last {
                        // Keep outer context's should_pop_expr_result for the last
                        // statement. Expression contexts (function bodies) will have
                        // it as false (keep value as return). Statement contexts
                        // (for-loop bodies) will have it as true (discard value).
                    } else {
                        self.should_pop_expr_result = true;
                    }
                    self.compile_stmt(s)?;
                    self.should_pop_expr_result = old_pop;
                }

                self.pop_scope();
            }
            Stmt::If(if_stmt) => {
                let mut jumps_to_end = Vec::new();

                // Plan 348 C5: branch bodies are compiled directly in the
                // ENCLOSING scope (via compile_body_inline, which does NOT wrap
                // in Stmt::Block and therefore introduces no extra scope). This
                // means a `var x = ...` declared inside an if-branch leaks to
                // the enclosing scope and remains visible after the if —
                // matching the if-branch scoping of many languages. We do NOT
                // push a new scope here, so the binding survives the if.
                for branch in &if_stmt.branches {
                    // Cond
                    self.compile_expr(&branch.cond)?;

                    // JMP_IF_Z to Next Branch (or Else/End)
                    self.emit(OpCode::JMP_IF_Z);
                    let jump_to_next = self.emit_placeholder_i16();

                    // Body — compile statements directly (no extra Stmt::Block
                    // wrapper, so no extra scope is introduced per branch).
                    self.compile_body_inline(&branch.body)?;

                    // If True, JMP to End (skip other branches/else)
                    // Optimization: We could skip this for the very last block, but keeping it uniform is safer/easier.
                    self.emit(OpCode::JMP);
                    let jump_to_end = self.emit_placeholder_i16();
                    jumps_to_end.push(jump_to_end);

                    // Patch JMP_IF_Z to point here (Start of Next Branch)
                    self.patch_jump(jump_to_next);
                }

                if let Some(else_body) = &if_stmt.else_ {
                    self.compile_body_inline(else_body)?;
                }

                // Patch all "JMP to End" to point here
                for jump in jumps_to_end {
                    self.patch_jump(jump);
                }
            }
            Stmt::Try(try_stmt) => {
                // Plan 010 (MS3-A): try/catch codegen.
                // Plan 012 P2 (gap 4): optional finally — both the normal path
                // and the post-catch path converge on the finally body.
                //
                //   PUSH_HANDLER <catch_pc>   ; install catch frame
                //   <try body>
                //   POP_HANDLER               ; normal exit — disarm
                //   JMP <finally_pc|after>
                // catch_pc:
                //   <STORE_LOCAL param | POP> ; bind/ignore the error value
                //   <catch body>              ; falls through into finally
                // finally_pc:                 ; (only when finally present)
                //   <finally body>
                // after:
                //
                // On a runtime error mid-body, execute_single_frame pops the
                // handler frame, restores bp/sp, pushes the error message, and
                // jumps to catch_pc — so the catch handler's first instruction
                // must consume that pushed value.
                //
                // Deviation (documented): an error thrown INSIDE the catch body
                // propagates without running finally (no nested handler frame
                // is installed). The common paths — normal exit and
                // error-then-catch — both run finally.
                self.emit(OpCode::PUSH_HANDLER);
                let handler_placeholder = self.emit_placeholder_i16();

                // Try body.
                self.compile_stmt(&Stmt::Block(try_stmt.body.clone()))?;

                // Normal exit: disarm handler and skip catch.
                self.emit(OpCode::POP_HANDLER);
                self.emit(OpCode::JMP);
                let jmp_after = self.emit_placeholder_i16();

                // catch handler entry — patch PUSH_HANDLER to land here.
                self.patch_jump(handler_placeholder);

                // The error message was pushed onto the stack by the engine.
                // Bind it to the catch parameter (if any) or pop it.
                self.push_scope();
                if let Some(param) = &try_stmt.catch_param {
                    let idx = self.add_var(param);
                    self.var_mutability.insert(param.clone(), true);
                    self.emit_store_loc(idx);
                } else {
                    self.emit(OpCode::POP);
                }

                // Catch body.
                self.compile_stmt(&Stmt::Block(try_stmt.catch_body.clone()))?;
                self.pop_scope();

                // Plan 012 P2: finally — the catch path falls through into it;
                // the normal path's JMP targets it (or `after` when absent).
                if let Some(finally_body) = &try_stmt.finally_body {
                    self.patch_jump(jmp_after);
                    self.compile_stmt(&Stmt::Block(finally_body.clone()))?;
                } else {
                    // after: patch the JMP-over-catch to land here.
                    self.patch_jump(jmp_after);
                }
            }
            Stmt::Fn(fn_decl) => {
                // Reset last_expr_type for each function to avoid stale type from previous compilation
                self.last_expr_type = ObjectType::Void;
                self.last_expr_may_py = false; // Plan 569 D2: 侧表粘性位同幅重置

                // #[vm] functions are implemented by native Rust shims, not VM bytecode.
                // If the native registry doesn't have a matching entry, the codegen will
                // emit a regular CALL to this function. Generate a runtime error stub
                // so the user gets a clear message instead of silent wrong behavior.
                if matches!(fn_decl.kind, crate::ast::FnKind::VmFunction) {
                    let fn_name_str = fn_decl.name.to_string();
                    // PLAN-013 T1: record the decl so calls can prefer the
                    // declared native over the stub (see import_scope arm).
                    self.vm_fn_names.insert(fn_name_str.clone());
                    vm_debug!("DEBUG: Compiling #[vm] stub for '{}' — will panic at runtime if native not found",
                        fn_name_str
                    );

                    // Export the function so linker can resolve it
                    let entry_point = self.code.len() as u32;
                    self.exports.insert(fn_name_str.clone(), entry_point);

                    // FN_PROLOG with correct arg count
                    let n_args = fn_decl.params.len() as u8;
                    self.emit(OpCode::FN_PROLOG);
                    self.code.push(n_args);
                    self.code.push(0); // n_locals = 0

                    // Push error message as string constant
                    let err_msg = format!(
                        "Runtime error: #[vm] function '{}' has no native implementation. \
                        Check that the native registry has a matching entry (e.g., \"str.{}\" or \"Str.{}\").",
                        fn_name_str,
                        fn_name_str.split('.').last().unwrap_or(&fn_name_str),
                        fn_name_str.split('.').last().unwrap_or(&fn_name_str)
                    );
                    let msg_idx = self.strings.len() as u32;
                    self.strings.push(err_msg.as_bytes().to_vec());
                    self.emit(OpCode::LOAD_STR);
                    self.code.extend_from_slice(&msg_idx.to_le_bytes());

                    // Call NATIVE_RUNTIME_PANIC — pops the message string and returns VMError
                    self.emit(OpCode::CALL_NAT);
                    self.code
                        .extend_from_slice(&crate::vm::native::NATIVE_RUNTIME_PANIC.to_le_bytes());

                    // RET — unreachable but needed for well-formed bytecode
                    let n_args_i16 = n_args as i16;
                    self.emit(OpCode::RET);
                    self.code.extend_from_slice(&n_args_i16.to_le_bytes());

                    // Record return type
                    self.fn_return_types
                        .insert(fn_name_str.clone(), fn_decl.ret.clone());

                    return Ok(());
                }

                // 1. Jump over function body (so it's not executed during definition flow)
                self.emit(OpCode::JMP);
                let jump_over = self.emit_placeholder_i16();

                // 2. Record function entry point (export)
                // Entry point is HERE (after JMP instruction)
                let entry_point = self.code.len() as u32;
                vm_debug!(
                    "DEBUG: Exporting function '{}' at address {:#04x}",
                    fn_decl.name,
                    entry_point
                );
                self.exports.insert(fn_decl.name.to_string(), entry_point);

                // Plan 312 Phase 2: Collect #[api] routes for HTTP server.
                // If this function has api_attrs, record (method, path, fn_name)
                // so the VM can build a route table at startup.
                if let Some(api) = &fn_decl.api_attrs {
                    self.api_routes.push((
                        api.method.clone(),
                        api.path.clone(),
                        fn_decl.name.to_string(),
                    ));
                    // PLAN-669: publish declared params (name + type display)
                    // so the HTTP server binds request args by name
                    // (path segment → body field → query param). Dep-module
                    // Codegen instances pass here too; the run pipeline resets
                    // the table before compiling (clear_api_param_sigs).
                    crate::vm::ffi::http_server::record_api_param_sigs(
                        &fn_decl.name.to_string(),
                        fn_decl
                            .params
                            .iter()
                            .map(|p| crate::vm::ffi::http_server::ApiParamSig {
                                name: p.name.to_string(),
                                ty: p.ty.to_string(),
                            })
                            .collect(),
                    );
                    // PLAN-698 T-02/SD-02: publish the declared return type
                    // too — the HTTP server's SSE publisher arm reads it to
                    // gate broadcasts (has_sse) and name New{Type} events.
                    // unique_name (not Display) — Display dumps the full
                    // type-decl s-expr for User types; the publisher needs
                    // the bare entity name ("Message" → "NewMessage").
                    let ret_display = fn_decl.ret.unique_name().to_string();
                    crate::vm::ffi::http_server::record_api_return_type(
                        &fn_decl.name.to_string(),
                        ret_display.clone(),
                    );
                    // PLAN-776 T-06：用户同名遮蔽——返回裸名命中编译单元类型
                    // 声明（TypeStore）时冻结遮蔽标记（VM 编组门 → Json；
                    // stdlib HTTP 词汇不注册 TypeStore，命中即用户来源）。
                    {
                        let bare = crate::api::contract::return_bare_name(&ret_display).to_string();
                        let shadowed = self
                            .type_store
                            .read()
                            .map(|ts| ts.contains_user_type(&bare))
                            .unwrap_or(false);
                        if shadowed {
                            crate::vm::ffi::http_server::record_api_return_shadowed(
                                &fn_decl.name.to_string(),
                            );
                        }
                    }
                    // PLAN-730 T-02: 上传端点加载期合同诊断——UploadRequest
                    // 注入参数仅 POST/PUT；其余参数必须 path/query/meta 可
                    // 绑定（不允许同时声明 JSON whole-body 注入）；返回类型
                    // 不得是 UploadRequest/UploadSession（必须是 UploadReceipt）。
                    let sigs = &fn_decl.params;
                    let upload_param = sigs.iter().find(|p| {
                        crate::api::contract::is_upload_param(&p.ty.unique_name().to_string())
                    });
                    if let Some(up) = upload_param {
                        if !matches!(api.method.to_uppercase().as_str(), "POST" | "PUT") {
                            return Err(crate::error::AutoError::Msg(format!(
                                "api fn '{}' declares an UploadRequest param '{}' but method is {} (uploads are POST/PUT only)",
                                fn_decl.name, up.name, api.method
                            )));
                        }
                        for p in sigs {
                            // PLAN-734 T-03：类型身份用 unique_name（User 的
                            // Display 是 s-expr 形态——旧 contains 恰好子串命中）。
                            let ty = p.ty.unique_name().to_string();
                            if crate::api::contract::is_upload_param(&ty)
                                || ty.trim() == "UploadReceipt"
                                || ty.trim() == "UploadSession"
                            {
                                continue;
                            }
                            // path 参数必须在路径模板中；其余仅 meta 约定名。
                            let in_path = api.path.contains(&format!(":{}", p.name));
                            let is_meta = matches!(p.name.as_str(), "meta" | "metadata");
                            if !in_path && !is_meta {
                                return Err(crate::error::AutoError::Msg(format!(
                                    "api fn '{}': upload endpoints allow only path params and meta alongside UploadRequest (param '{}' would be JSON-body bound)",
                                    fn_decl.name, p.name
                                )));
                            }
                        }
                        let ret_name = fn_decl.ret.unique_name().to_string();
                        if crate::api::contract::ResponseKind::from_return_string(&ret_name)
                            != crate::api::contract::ResponseKind::Upload
                        {
                            return Err(crate::error::AutoError::Msg(format!(
                                "api fn '{}' declares an UploadRequest param but returns {} (must return UploadReceipt / ~UploadReceipt)",
                                fn_decl.name, ret_name
                            )));
                        }
                    } else {
                        let ret_name = fn_decl.ret.unique_name().to_string();
                        if crate::api::contract::ResponseKind::from_return_string(&ret_name)
                            == crate::api::contract::ResponseKind::Upload
                        {
                            return Err(crate::error::AutoError::Msg(format!(
                                "api fn '{}' returns UploadReceipt but has no UploadRequest param",
                                fn_decl.name
                            )));
                        }
                    }
                }

                // Plan 321/327: Detect generator functions (return type ~Iter<T>
                // or ~Stream<T> AND body contains at least one yield). Marking
                // only by return type would misclassify wrapper functions like
                // `fn get() ~Iter<int> { return counter() }` (no yield of its
                // own) as generators — short-circuiting them skips the return,
                // yielding an empty iterator (sum=0 bug). Requiring a yield in
                // the body matches the runtime is_generator_fn byte-scan.
                if matches!(&fn_decl.ret, Type::GenericInstance(inst)
                    if inst.base_name.as_str() == "Iter" || inst.base_name.as_str() == "Stream")
                    && body_contains_yield(&fn_decl.body)
                {
                    self.generator_fns.insert(fn_decl.name.to_string());
                }

                // PLAN-705 T-04/SD-01: publish async-return metadata for ALL
                // fns declaring `~T`（解析期脱糖为 Future<T>）——HTTP 编组的
                // 元数据门（返回 future bits 只在声明方为异步时才按 future
                // 消费，普通 int 位模式永不误判）。run 管线在编译前经
                // clear_api_param_sigs 一并清空本表。
                if matches!(&fn_decl.ret, Type::GenericInstance(inst)
                    if inst.base_name.as_str() == "Future")
                {
                    crate::vm::ffi::http_server::record_api_async_return(&fn_decl.name.to_string());
                }

                // 3. Push new scope for function locals
                self.push_scope();

                // Plan 317: For `fn main`, emit module-level global var
                // initializers at the very top of the body. execute_autovm
                // spawns main directly (skipping the script wrapper's top-level
                // code), so without this globals would never be initialized.
                if fn_decl.name == "main" {
                    for (name, expr) in &self.global_inits.clone() {
                        self.compile_expr(expr)?;
                        self.emit_global_store(name);
                    }
                }

                // 4. Compile function parameters
                // Plan 088 Phase 4: Store parameter types and modes for smart parameter passing
                let param_infos: Vec<ParamInfo> = fn_decl
                    .params
                    .iter()
                    .map(|param| ParamInfo {
                        ty: param.ty.clone(),
                        mode: param.mode,
                    })
                    .collect();

                // Store parameter information in fn_params map
                self.fn_params
                    .insert(fn_decl.name.to_string(), param_infos.clone());

                // Plan 442 C2: axum serve adapter — publish declared param
                // type display strings (Type's Display: "State<AppState>",
                // "Json<LoginRequest>", ...) directly to the adapter's
                // process-global table. Dep-module Codegen instances publish
                // here too, so extractor shapes resolve for cross-module
                // fn-ref closures; the run pipeline resets the table up front.
                crate::vm::ffi::axum_adapter::record_param_sig(
                    &fn_decl.name.to_string(),
                    fn_decl.params.iter().map(|p| p.ty.to_string()).collect(),
                );

                // Plan 087 Phase 3: Store function return type for .type property support
                self.fn_return_types
                    .insert(fn_decl.name.to_string(), fn_decl.ret.clone());

                // Plan 087 Phase 3: Set current function parameter count
                self.current_fn_n_args = fn_decl.params.len();
                self.current_fn_ret_type = fn_decl.ret.clone();

                // PLAN-019 T-06 (vm delegate): record the fn's module prefix so
                // bare intra-module calls in its body bind to its own module
                // (see current_fn_module field doc). Flattened-qualified names
                // (`db.mux_resize_pane`) carry the prefix; bare names → None.
                let saved_fn_module = self.current_fn_module.take();
                self.current_fn_module = fn_decl
                    .name
                    .to_string()
                    .rsplit_once('.')
                    .map(|(prefix, _)| prefix.to_string())
                    // PLAN-733: 裸名合成 fn（store handler/computed 隐藏 fn）
                    // 无点前缀——回退 store 域绑定（handler_codegen per-widget
                    // 设置；非 store 域为 None，行为不变）。
                    .or_else(|| self.store_scope_module.clone());

                // Plan 417-E3: record this fn's type-parameter names so method
                // calls on receivers typed as one of them dispatch dynamically
                // (CALL_SPEC on the runtime heap tag) instead of a static
                // `T.method` reloc that can never link.
                let saved_fn_type_params = self.current_fn_type_params.clone();
                self.current_fn_type_params = fn_decl
                    .type_params
                    .iter()
                    .map(|tp| tp.name.to_string())
                    .collect();

                // Plan 417-E3-P4: publish this fn's bounds for call-site
                // checking (see check_generic_call_bounds). Overwrite on
                // recompile so the table always matches the latest decl.
                if !fn_decl.type_params.is_empty() {
                    self.fn_type_param_bounds.insert(
                        fn_decl.name.to_string(),
                        fn_decl
                            .type_params
                            .iter()
                            .map(|tp| (tp.name.clone(), tp.constraint.clone()))
                            .collect(),
                    );
                }

                // Plan 087 Phase 3: Record starting index for function scope
                // This is needed because outer scope variables affect parameter indices
                // Parameters will have indices: fn_scope_start, fn_scope_start+1, ...
                self.fn_scope_start = self.scope_stack.iter().map(|s| s.len()).sum();

                // Save var_types and infer_ctx state so function parameter types don't leak
                let saved_var_types = self.var_types.clone();
                let saved_type_env = self.infer_ctx.type_env.clone();

                // Save and reset max_locals for this function
                let old_max_locals = self.max_locals;
                self.max_locals = 0;

                // Add parameters to scope
                // Plan 087 Phase 3: Record self parameter type for field access
                for param in &fn_decl.params {
                    self.add_var(&param.name);

                    // Register parameter type for method resolution (e.g., s.upper())
                    if !matches!(param.ty, Type::Unknown) {
                        self.var_types
                            .insert(param.name.to_string(), param.ty.clone());
                    }

                    // Check if this is a 'self' parameter in a method
                    if param.name.to_string() == "self" {
                        // Extract type name from method name (e.g., "Counter.get" → "Counter")
                        if let Some(dot_pos) = fn_decl.name.to_string().find('.') {
                            let type_name = fn_decl.name.to_string()[..dot_pos].to_string();
                            vm_debug!("DEBUG: Recording self parameter type: {}", type_name);
                            // Create a synthetic TypeDecl for type tracking
                            if let Some(_type_info) = self.get_type(&type_name) {
                                let type_decl = crate::ast::TypeDecl {
                                    consts: Vec::new(),
                                    name: crate::ast::Name::from(type_name),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: vec![],
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    impl_attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                };
                                self.var_types
                                    .insert("self".to_string(), Type::User(type_decl));
                            }
                        }
                    }
                }

                // 5. Compile body FIRST to count locals
                self.compile_stmt(&Stmt::Block(fn_decl.body.clone()))?;

                // Plan 321/326: Generator functions need an explicit nil return
                // because yield's last statement pops its value, leaving nothing
                // for RET. Push nil + RET so the generator terminates cleanly.
                let is_gen = matches!(&fn_decl.ret, Type::GenericInstance(inst)
                    if inst.base_name == "Iter" || inst.base_name == "Stream");
                if is_gen {
                    self.emit(OpCode::CREATE_NONE); // push nil as return value
                    let n_args = fn_decl.params.len() as i16;
                    self.emit(OpCode::RET);
                    self.code.extend_from_slice(&n_args.to_le_bytes());
                }

                // Plan 118 Phase 7: Update function return type based on body inference
                // If parser defaulted to Void but body has implicit return, update the type
                // This allows proper void detection for calls like: fn hi(s str) { print(s); }; hi("hello")
                if matches!(fn_decl.ret, Type::Void) {
                    // Check if body actually returns a value (has implicit return)
                    if self.last_expr_type != ObjectType::Void {
                        // Body has implicit return - mark as non-void
                        // Use Unknown to indicate "has value but type unknown"
                        self.fn_return_types
                            .insert(fn_decl.name.to_string(), Type::Unknown);
                    }
                } else {
                    // Function has explicit return type — record it for callers
                    self.fn_return_types
                        .insert(fn_decl.name.to_string(), fn_decl.ret.clone());
                }
                // Plan 569 D2: fn 尾表达式 may_py 回填——fn 体编译后粘性位
                // 反映尾表达式；命中记 fn_may_py_returns，调用点查表传导
                // （单层流型保守正报：错报=obj_* 组合子 Auto 臂照常分派，安全；
                // 漏报=维持现状规避）。
                if self.last_expr_may_py {
                    self.fn_may_py_returns.insert(fn_decl.name.to_string());
                }

                // 6. Get number of locals and INSERT stack reservation at function entry
                let n_args = fn_decl.params.len();
                // Use max_locals to account for nested scopes correctly
                let n_locals = if self.max_locals > n_args {
                    self.max_locals - n_args
                } else {
                    0
                };

                // Plan 088 Phase 4: Always emit FN_PROLOG at function entry
                // This provides function metadata for dynamic parameter counting
                // IMPORTANT: Adjust exports FIRST (before inserting FN_PROLOG and RESERVE_STACK)
                // All function addresses > entry_point (after current function) will shift after insertion
                // NOTE: Current function (at entry_point) should NOT be adjusted!
                let mut adjusted_exports = std::collections::HashMap::new();
                for (name, &addr) in &self.exports {
                    if addr > entry_point {
                        // Note: > not >=
                        let shift = if n_locals > 0 { 5 } else { 3 }; // FN_PROLOG (3 bytes) + optional RESERVE_STACK (2 bytes)
                        adjusted_exports.insert(name.clone(), addr + shift);
                    }
                }
                // Apply the adjustments
                for (name, new_addr) in adjusted_exports {
                    self.exports.insert(name, new_addr);
                }

                // IMPORTANT: Adjust reloc offsets too!
                // Relocations that target positions >= entry_point will have their placeholder
                // positions shifted after insertion.
                let shift = if n_locals > 0 { 5 } else { 3 };
                for reloc in &mut self.relocs {
                    if reloc.offset >= entry_point {
                        reloc.offset += shift;
                    }
                }

                // Plan 088 Phase 4: Adjust jump placeholder indices BEFORE insertion!
                // Jump placeholders AFTER entry_point need to be shifted
                // Jump placeholders BEFORE or AT entry_point are NOT affected
                // (e.g., current function's jump_over at entry_point-2 stays at same position)
                // This MUST happen BEFORE code.insert() so patch_jump uses correct indices
                for placeholder_idx in &mut self.jump_placeholders {
                    if *placeholder_idx > entry_point as usize {
                        *placeholder_idx += shift as usize;
                    }
                }

                // Insert FN_PROLOG at entry_point (before function body)
                // This is 3 bytes: 1 byte opcode + 1 byte n_args + 1 byte n_locals
                vm_debug!(
                    "DEBUG: Emitting FN_PROLOG for '{}' at address {}, n_args={}, n_locals={}",
                    fn_decl.name,
                    entry_point,
                    n_args,
                    n_locals
                );
                self.code
                    .insert(entry_point as usize, OpCode::FN_PROLOG as u8);
                self.code.insert(entry_point as usize + 1, n_args as u8);
                self.code.insert(entry_point as usize + 2, n_locals as u8);

                // Insert RESERVE_STACK after FN_PROLOG (if needed)
                if n_locals > 0 {
                    // Insert RESERVE_STACK at entry_point + 3 (after FN_PROLOG)
                    // This is 2 bytes: 1 byte opcode + 1 byte operand
                    self.code
                        .insert(entry_point as usize + 3, OpCode::RESERVE_STACK as u8);
                    self.code.insert(entry_point as usize + 4, n_locals as u8);
                }

                // Plan 118 Phase 5: Recalculate jump offsets after FN_PROLOG insertion
                // After inserting FN_PROLOG (+ RESERVE_STACK if any), all jump targets have shifted.
                // We need to recalculate all jump offsets for jumps that were patched before this insertion.
                //
                // For each (placeholder_idx, target_idx) pair in jump_targets:
                // - Both placeholder and target may have shifted if they were after entry_point
                // - The offset needs to be recalculated using the new positions
                //
                // Cases:
                // 1. placeholder > entry_point, target > entry_point: Both shifted by `shift` bytes
                //    New offset = (target + shift) - (placeholder + shift + 2) = target - placeholder - 2
                //    Same as before! No change needed.
                // 2. placeholder <= entry_point, target > entry_point: Only target shifted
                //    New offset = (target + shift) - (placeholder + 2) = old_offset + shift
                // 3. placeholder > entry_point, target <= entry_point: Only placeholder shifted
                //    This shouldn't happen for forward jumps (target is always after placeholder)
                // 4. placeholder <= entry_point, target <= entry_point: Neither shifted
                //    No change needed.
                //
                // So we only need to fix case 2: jumps that START BEFORE entry_point and END AFTER entry_point
                let _shift_amount = shift as isize;
                for (old_placeholder, old_target) in &self.jump_targets {
                    // Calculate new positions
                    let new_placeholder = if *old_placeholder > entry_point as usize {
                        old_placeholder + shift as usize
                    } else {
                        *old_placeholder
                    };
                    let new_target = if *old_target > entry_point as usize {
                        old_target + shift as usize
                    } else {
                        *old_target
                    };

                    // Check if this jump crosses the insertion point
                    // (placeholder before or at entry_point, target after entry_point)
                    if *old_placeholder <= entry_point as usize
                        && *old_target > entry_point as usize
                    {
                        // Recalculate offset with shifted target
                        let new_anchor = new_placeholder + 2;
                        let new_offset = (new_target as isize) - (new_anchor as isize);

                        vm_debug!("DEBUG: Recalculating jump at {} (was {}): old_target={}, new_target={}, new_offset={}",
                            new_placeholder, old_placeholder, old_target, new_target, new_offset
                        );

                        if new_offset > i16::MAX as isize || new_offset < i16::MIN as isize {
                            panic!("Jump offset too large after recalculation: {}", new_offset);
                        }

                        let bytes = (new_offset as i16).to_le_bytes();
                        self.code[new_placeholder] = bytes[0];
                        self.code[new_placeholder + 1] = bytes[1];
                    }
                }

                // Restore max_locals
                self.max_locals = old_max_locals;

                // 7. Emit RET at end of body.
                // Plan 377: 全值单槽化后恒用 RET（f64/u64/i64 返回值亦单槽，无需 RET_D）。
                let n_args = fn_decl.params.len() as u8;
                // 若函数声明返回 f64 但 body 末尾是 f32，仍需 PROMOTE_F64 提升编码
                if matches!(self.current_fn_ret_type, Type::Double)
                    && !matches!(self.last_expr_type, ObjectType::Double)
                    && matches!(self.last_expr_type, ObjectType::Float)
                {
                    self.emit(OpCode::PROMOTE_F64);
                }
                self.emit(OpCode::RET);
                self.code.push(n_args);

                // 8. Pop function scope
                self.pop_scope();

                // Restore var_types and infer_ctx to prevent type leakage between functions
                self.var_types = saved_var_types;
                self.infer_ctx.type_env = saved_type_env;

                // Plan 087 Phase 3: Reset current function parameter count and scope start
                self.current_fn_n_args = 0;
                self.current_fn_ret_type = Type::Void;
                self.current_fn_type_params = saved_fn_type_params; // Plan 417-E3
                self.current_fn_module = saved_fn_module; // PLAN-019 T-06
                self.fn_scope_start = 0;

                // 9. Patch jump to skip body
                self.patch_jump(jump_over);
            }
            Stmt::Store(store) => {
                // Plan 567 T19（W3 nullability lint）: 已知 `T | None` 返回的
                // py 调用直接赋值未判空（W 级提示，不阻断；判空=.?/==null/
                // NullCoalesce 形态由表达式层自带）。
                self.check_nullable_unguarded(&store.expr);
                // Plan 569 D2: Store 边界重置粘性位——数组分配等无 compile_expr
                // 的路径不得残留上一语句的 may_py；RHS 编译后按新鲜值落盘。
                self.last_expr_may_py = false;
                // Variable declaration: let/mut/var name = expr
                //
                // Immutability checking:
                // - let x = 5: creates immutable binding
                // - mut x = 5: creates mutable binding
                // - var x = 5: creates mutable binding
                // - x = 7: reassignment (error if x was declared with let)

                let name_str = store.name.to_string();
                // Plan 322: auto-register top-level `var` as a global so it's
                // visible from functions via LOAD_GLOBAL. Without this, script-path
                // module-level vars are treated as locals (invisible to fns).
                // Plan 348 E1: also admit top-level `const` so a module-level
                // const (e.g. `const MAX int = 42`) is stored in vm.globals and
                // is visible across module boundaries via `use mod: MAX`. `let`
                // intentionally stays local (mutable-in-scope semantics differ).
                if matches!(
                    store.kind,
                    crate::ast::StoreKind::Var | crate::ast::StoreKind::Const
                ) && self.scope_stack.len() <= 1
                {
                    // Plan 348 Task 20: also record the initializer so `fn main`
                    // can run it. execute_autovm spawns main directly (skipping
                    // the script wrapper's top-level code), so without recording
                    // the init here, module-level var/const globals are never
                    // stored when a `main` fn exists — functions that read them
                    // via LOAD_GLOBAL get the default 0 / wrong value. The init
                    // is emitted at the top of main's body (see Stmt::Fn handling
                    // of `fn main` below). Dedup by name so re-declaration or the
                    // API-routes pre-pass doesn't double-emit.
                    if !self.global_vars.contains(&name_str) {
                        self.global_inits
                            .push((name_str.clone(), store.expr.clone()));
                    }
                    self.global_vars.insert(name_str.clone());
                }
                // Plan 080/118/087: record the initializer's type into var_types
                // BEFORE any storage-path branching. The global fast path below
                // early-returns, and skipping the tracking there (Plan 338
                // regression, dstr suite) made `var s = String.new(); s.push()`
                // lose compile-time String.push → auto.str.push resolution and
                // degrade to runtime CALL_SPEC, where only the heap tag
                // ("StringBuilder") is known and dispatch fails.
                self.track_store_var_type(store);
                // Plan 317: module-level global variable (top-level `var`).
                // Compile the init expr and STORE_GLOBAL; skip local slot
                // registration so the value lives in vm.globals (cross-fn).
                // Plan 333: force_global_store (set inside __module_init) makes
                // this apply even within a fn body, so module-level initializers
                // wrapped in __module_init store globally rather than as locals.
                // Plan 322: global_vars variables ALWAYS go through STORE_GLOBAL
                // regardless of scope — a function that reassigns a module-level
                // global (e.g. db.at's `notes = new_notes`) must update the global,
                // not a local shadow. Without this, delete_note/create_note's
                // `notes = ...` writes to a local, and list_notes() reads the
                // stale global.
                if self.global_vars.contains(&name_str) {
                    // Only treat as global at top level (scope_stack.len() <= 1
                    // = script wrapper scope). Inside fns, a `var x` with the
                    // same name as a global is a local shadow, not a global store.
                    self.compile_expr(&store.expr)?;
                    self.emit_global_store(&name_str);
                    // Plan 569 D2: 全局路径的侧表落盘（与下方局部路径同款）。
                    if self.last_expr_may_py {
                        self.py_typed_vars.insert(name_str.clone());
                    } else {
                        self.py_typed_vars.remove(&name_str);
                    }
                    // Plan 377: 记录 global 的类型到 var_types，使后续 LOAD_GLOBAL 的
                    // `Expr::Ident` 路径能设对 last_expr_type（U64/Double → PRINT_U64/F64）。
                    // 否则全局 u64/f64 变量在 print 时会被当作 Int 路由到 PRINT_I32 而截断。
                    if !matches!(store.ty, Type::Unknown) {
                        self.var_types.insert(name_str.clone(), store.ty.clone());
                    } else if matches!(self.last_expr_type, ObjectType::Double) {
                        self.var_types.insert(name_str.clone(), Type::Double);
                    } else if matches!(self.last_expr_type, ObjectType::Uint) {
                        self.var_types.insert(name_str.clone(), Type::U64);
                    }
                    return Ok(());
                }
                let scope = self
                    .scope_stack
                    .last_mut()
                    .expect("Scope stack should never be empty");

                // Plan 091: Check if this is a new declaration or reassignment
                // New declaration (let/var) allows shadowing
                // Only assignment expressions should check immutability
                let is_new_declaration = matches!(
                    store.kind,
                    crate::ast::StoreKind::Let
                        | crate::ast::StoreKind::Var
                        | crate::ast::StoreKind::Const
                        | crate::ast::StoreKind::CVar
                        | crate::ast::StoreKind::Shared
                );

                if !is_new_declaration && scope.contains_key(&name_str) {
                    // Reassignment of existing variable — check immutability
                    if let Some(is_mutable) = self.var_mutability.get(&name_str) {
                        if !is_mutable {
                            return Err(crate::error::AutoError::Msg(format!(
                                "Cannot reassign to immutable variable '{}' (declared with 'let')",
                                name_str
                            )));
                        }
                    }
                }

                if is_new_declaration || !scope.contains_key(&name_str) {
                    // First-time declaration - track mutability based on StoreKind
                    let is_mutable = matches!(
                        store.kind,
                        crate::ast::StoreKind::Var
                            | crate::ast::StoreKind::CVar
                            | crate::ast::StoreKind::Shared
                    );
                    self.var_mutability.insert(name_str.clone(), is_mutable);

                    // Plan 087 Phase 1: Handle generic type instantiations
                    // If the variable has an explicit generic type annotation (e.g., let p: Pair<int, string>),
                    // register the instantiation in the GenericRegistry
                    if let Type::GenericInstance(ref inst) = store.ty {
                        // Extract type arguments from GenericInstance
                        let type_args: Vec<Type> = inst.args.clone();

                        // Register or get the ClassType from GenericRegistry
                        if let Ok(_class_type) = self
                            .generic_registry
                            .get_or_create_type(&inst.base_name.to_string(), type_args)
                        {
                            // Store the complete type in var_types
                            self.var_types.insert(name_str.clone(), store.ty.clone());
                        } else {
                            eprintln!(
                                "Warning: Failed to register generic instance '{}'",
                                inst.base_name
                            );
                        }
                    } else if !matches!(store.ty, Type::Unknown) {
                        // Plan 118: Store the explicit type annotation for proper output formatting
                        // Plan 197 Task 14: For arrays with Unknown element type, try to refine from actual elements
                        if let Type::Array(ref arr) = store.ty {
                            if matches!(arr.elem.as_ref(), Type::Unknown) {
                                if let Expr::Array(elems) = &store.expr {
                                    if !elems.is_empty() {
                                        let elem_ty = self.infer_expr_type(&elems[0]);
                                        if !matches!(elem_ty, Type::Unknown) {
                                            let refined = Type::Array(crate::ast::ArrayType {
                                                elem: Box::new(elem_ty),
                                                len: elems.len(),
                                            });
                                            self.var_types.insert(name_str.clone(), refined);
                                        } else {
                                            self.var_types
                                                .insert(name_str.clone(), store.ty.clone());
                                        }
                                    } else {
                                        self.var_types.insert(name_str.clone(), store.ty.clone());
                                    }
                                } else {
                                    self.var_types.insert(name_str.clone(), store.ty.clone());
                                }
                            } else {
                                self.var_types.insert(name_str.clone(), store.ty.clone());
                            }
                        } else {
                            // String range slice: e.g., let text = src[0..2]
                            // Parser may infer Char, but range slice on string produces string
                            if let Expr::Index(container, idx) = &store.expr {
                                if let Expr::Range(_) = idx.as_ref() {
                                    if self.is_string_expr(container) {
                                        self.var_types.insert(name_str.clone(), Type::StrFixed(0));
                                    } else {
                                        self.var_types.insert(name_str.clone(), store.ty.clone());
                                    }
                                } else {
                                    self.var_types.insert(name_str.clone(), store.ty.clone());
                                }
                            } else if let Expr::Ident(src_name) = &store.expr {
                                if let Some(src_type) = self.var_types.get(src_name.as_str()) {
                                    if !matches!(src_type, Type::Int | Type::Unknown) {
                                        self.var_types.insert(name_str.clone(), src_type.clone());
                                    } else {
                                        self.var_types.insert(name_str.clone(), store.ty.clone());
                                    }
                                } else {
                                    self.var_types.insert(name_str.clone(), store.ty.clone());
                                }
                            } else {
                                self.var_types.insert(name_str.clone(), store.ty.clone());
                            }
                        }
                    } else {
                        // Plan 118 Phase 4: Infer type from expression when annotation is Unknown
                        // Plan 118 Phase 7: Add closure type inference
                        let inferred_type = match &store.expr {
                            Expr::Str(s) => Type::StrFixed(s.len()),
                            Expr::CStr(_) => Type::CStrLit,
                            Expr::Char(_) => Type::Char,
                            Expr::Int(_) => Type::Int,
                            Expr::I8(_) => Type::Int, // I8 maps to Int
                            Expr::I64(_) => Type::I64,
                            Expr::Uint(_) => Type::Uint,
                            Expr::U8(_) => Type::Int, // U8 maps to Int (result is plain integer)
                            Expr::U64(_) => Type::U64,
                            Expr::Byte(_) => Type::Byte,
                            Expr::Float(_, _) => Type::Float,
                            Expr::Double(_, _) => Type::Double,
                            Expr::Bool(_) => Type::Bool,
                            // Plan 118 Phase 7: Closure type inference
                            // Infer fn(params) return_type for closure expressions
                            Expr::Closure(closure) => {
                                let param_types: Vec<Type> = closure
                                    .params
                                    .iter()
                                    .map(|p| p.ty.clone().unwrap_or(Type::Unknown))
                                    .collect();
                                // Infer return type from body expression
                                let ret_type = self.infer_expr_type(&closure.body);
                                Type::Fn(param_types, Box::new(ret_type))
                            }
                            // Plan 158: Infer type from function call
                            Expr::Call(call) => {
                                match call.name.as_ref() {
                                    Expr::Ident(fn_name) => self
                                        .fn_return_types
                                        .get(fn_name.as_ref())
                                        .cloned()
                                        .unwrap_or(Type::Unknown),
                                    Expr::Dot(obj, method) => {
                                        if let Expr::Ident(obj_name) = obj.as_ref() {
                                            let full_name = format!("{}.{}", obj_name, method);
                                            // Plan 368 W6-cross-domain: try qualified name,
                                            // then fall back to bare method name (same as R-W4
                                            // at the CALL reloc site, line 7894).
                                            let ret_ty =
                                                self.fn_return_types.get(&full_name).or_else(
                                                    || self.fn_return_types.get(method.as_str()),
                                                );
                                            if let Some(ty) = ret_ty {
                                                ty.clone()
                                            } else if let Some(type_name) =
                                                self.infer_type_from_var(obj_name.as_ref())
                                            {
                                                let type_method =
                                                    format!("{}.{}", type_name, method);
                                                if let Some(ty) =
                                                    self.fn_return_types.get(&type_method)
                                                {
                                                    ty.clone()
                                                } else {
                                                    // Array HOF methods return arrays
                                                    let hof_methods = ["map", "filter"];
                                                    if (type_name == "Array" || type_name == "List")
                                                        && hof_methods.contains(&method.as_str())
                                                    {
                                                        Type::Array(crate::ast::ArrayType {
                                                            elem: Box::new(Type::Int),
                                                            len: 0,
                                                        })
                                                    } else {
                                                        Type::Unknown
                                                    }
                                                }
                                            } else {
                                                Type::Unknown
                                            }
                                        } else {
                                            let fn_name = format!(
                                                "{}.{}",
                                                self.expr_to_name(obj.as_ref()),
                                                method.as_ref()
                                            );
                                            // Plan 368 W6-cross-domain: bare-name fallback
                                            let ret_ty =
                                                self.fn_return_types.get(&fn_name).or_else(|| {
                                                    self.fn_return_types.get(method.as_ref())
                                                });
                                            if let Some(ty) = ret_ty {
                                                ty.clone()
                                            } else if let Some(type_name) =
                                                self.infer_user_type_name(obj.as_ref())
                                            {
                                                let type_method =
                                                    format!("{}.{}", type_name, method.as_ref());
                                                self.fn_return_types
                                                    .get(&type_method)
                                                    .cloned()
                                                    .unwrap_or(Type::Unknown)
                                            } else {
                                                // Plan 202: Try with receiver's type name (e.g., "str" for string literals)
                                                let obj_ot = self.infer_object_type(obj.as_ref());
                                                let type_name = match obj_ot {
                                                    ObjectType::String => Some("str"),
                                                    ObjectType::Int => Some("int"),
                                                    ObjectType::Array => Some("List"),
                                                    ObjectType::Float | ObjectType::Double => {
                                                        Some("float")
                                                    }
                                                    ObjectType::Bool => Some("bool"),
                                                    ObjectType::Char => Some("char"),
                                                    _ => None,
                                                };
                                                if let Some(tn) = type_name {
                                                    let qualified =
                                                        format!("auto.{}.{}", tn, method);
                                                    self.fn_return_types
                                                        .get(&qualified)
                                                        .cloned()
                                                        .unwrap_or_else(|| {
                                                            let short =
                                                                format!("{}.{}", tn, method);
                                                            self.fn_return_types
                                                                .get(&short)
                                                                .cloned()
                                                                .unwrap_or(Type::Unknown)
                                                        })
                                                } else {
                                                    Type::Unknown
                                                }
                                            }
                                        }
                                    }
                                    _ => Type::Unknown,
                                }
                            }
                            // Plan 197 Task 4: Infer type from field access (e.g., let v = r.value)
                            Expr::Dot(_, _) => self.infer_expr_type(&store.expr),
                            // Plan 197 Task 14: Infer type from array indexing (e.g., let first = list[0])
                            Expr::Index(container, idx) => {
                                // Range slice on a string container produces a string, not char
                                if let Expr::Range(_) = idx.as_ref() {
                                    if self.is_string_expr(container) {
                                        Type::StrFixed(0)
                                    } else {
                                        self.infer_expr_type(&store.expr)
                                    }
                                } else {
                                    self.infer_expr_type(&store.expr)
                                }
                            }
                            // Plan 197 Task 14: Infer type from array literal (e.g., let list = [a, b])
                            Expr::Array(elems) => {
                                if elems.is_empty() {
                                    // Plan 445 M3：空数组字面量默认**动态 List**——
                                    // 此前落静态 Array{Int, len:0}，decl 路径（vm_bridge
                                    // new_from_decls，即 `auto run -r vm` 实机）的
                                    // `out.push(x)` 按静态数组/字符串方法解析到
                                    // auto.str.push(178)（应为 auto.list.push=101），
                                    // push 静默失效 → 滑窗重建列表恒空 → 024-charts
                                    // 实机 monthly=[] → 重算 510/0 DivisionByZero
                                    // 中止（ASH_DEBUG_VM_LOG 实锤）。与
                                    // synthesize_state_type 的 "var days = [] →
                                    // dynamic array" 语义对齐。
                                    if matches!(store.ty, Type::Unknown) {
                                        Type::List(Box::new(Type::Unknown))
                                    } else {
                                        store.ty.clone()
                                    }
                                } else {
                                    let elem_ty = self.infer_expr_type(&elems[0]);
                                    Type::Array(crate::ast::ArrayType {
                                        elem: Box::new(elem_ty),
                                        len: elems.len(),
                                    })
                                }
                            }
                            // Plan 202: Propagate type from source variable (e.g., let s = item)
                            Expr::Ident(src_name) => {
                                // Plan 383: 若 src_name 是已定义函数（函数引用赋值，
                                // 如 `let f = double`），推断 Type::Fn，使后续 `f()`
                                // 能被 is_closure_call（codegen.rs:7255）识别走 CALL_CLOSURE。
                                if self.is_defined_function(src_name.as_str()) {
                                    let params = self
                                        .fn_params
                                        .get(src_name.as_ref())
                                        .map(|p| p.iter().map(|pi| pi.ty.clone()).collect())
                                        .unwrap_or_default();
                                    let ret = self
                                        .fn_return_types
                                        .get(src_name.as_ref())
                                        .cloned()
                                        .unwrap_or(Type::Unknown);
                                    Type::Fn(params, Box::new(ret))
                                } else {
                                    self.var_types
                                        .get(src_name.as_str())
                                        .cloned()
                                        .unwrap_or_else(|| store.ty.clone())
                                }
                            }
                            // Plan 437: 混合算术的类型记录——任一侧含 float/double
                            // 字面量或变量即结果为 Float/Double。此前落 store.ty
                            // （Unknown→按 Int 消费），导致 `var b = a * 1.0` 的
                            // f-string part tag 错标 Int（f32 位模式打印成
                            // 1084227584），下游所有 Ident 类型查询随之走错分支。
                            Expr::Bina(lhs, _, rhs) => {
                                if self.contains_double(lhs) || self.contains_double(rhs) {
                                    Type::Double
                                } else if self.contains_float(lhs) || self.contains_float(rhs) {
                                    Type::Float
                                } else {
                                    store.ty.clone()
                                }
                            }
                            _ => store.ty.clone(),
                        };
                        self.var_types.insert(name_str.clone(), inferred_type);
                        // Plan 212 Phase 2.2: Track opaque type variables
                        // When `let re = Regex.new(...)` is compiled, record that `re` maps to "regex"
                        if let Some(opaque_crate) = self.infer_opaque_crate_from_expr(&store.expr) {
                            self.opaque_var_crates
                                .insert(name_str.clone(), opaque_crate);
                        }
                    }
                }

                // Plan 052: Handle runtime array allocation (var arr [N]int)
                // If the type is RuntimeArray or static Array AND there's no initializer expression,
                // allocate the array. If there IS an initializer, compile it normally (CREATE_ARRAY).
                if let Type::RuntimeArray(ref rta) = store.ty {
                    if matches!(&store.expr, Expr::Nil | Expr::Int(0)) {
                        // No initializer - allocate zeroed array
                        self.compile_expr(&rta.size_expr)?;
                        self.emit(OpCode::CALL_NAT);
                        self.emit_u16(
                            BIGVM_NATIVES
                                .lock()
                                .unwrap()
                                .resolve_qualified("auto.alloc.array")
                                .unwrap(),
                        );
                    } else {
                        // Has initializer - compile the expression
                        self.compile_expr(&store.expr)?;
                    }
                } else if let Type::Array(ref arr) = store.ty {
                    if matches!(&store.expr, Expr::Nil | Expr::Int(0)) {
                        // No initializer - allocate zeroed array of static size
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(arr.len as i32);
                        self.emit(OpCode::CALL_NAT);
                        self.emit_u16(
                            BIGVM_NATIVES
                                .lock()
                                .unwrap()
                                .resolve_qualified("auto.alloc.array")
                                .unwrap(),
                        );
                    } else {
                        // Has initializer (e.g., [1, 2, 3]) - compile the expression
                        self.compile_expr(&store.expr)?;
                    }
                } else {
                    // Plan 348 A3: When a 64-bit integer literal (I64/U64) is stored
                    // into a 32-bit Int/Uint variable, emit a truncated CONST_I32 so
                    // the stack stays 1-slot. Otherwise CONST_I64/CONST_U64 push 2
                    // slots while the Int store consumes only 1, corrupting the value
                    // (e.g. `var z int = 0x80000000` parsed as Expr::I64).
                    let store_is_32bit_int = matches!(store.ty, Type::Int | Type::Uint);
                    if store_is_32bit_int {
                        let truncated = match &store.expr {
                            Expr::I64(v) => Some(*v as i32),
                            Expr::U64(v) => Some(*v as i32),
                            _ => None,
                        };
                        if let Some(trunc) = truncated {
                            self.last_expr_type = if matches!(store.ty, Type::Uint) {
                                ObjectType::Uint
                            } else {
                                ObjectType::Int
                            };
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(trunc);
                        } else {
                            self.compile_expr(&store.expr)?;
                        }
                    } else {
                        // Compile the RHS expression (pushes result on stack)
                        self.compile_expr(&store.expr)?;
                    }

                    // Plan 569 D2: 局部路径的侧表落盘——RHS 编译后的新鲜粘性位
                    // 传导进 py_typed_vars（方法分派决策核消费）。重赋值为非 py
                    // 表达式时移除（零回归红线：Auto 变量回到原生静态路由）。
                    if self.last_expr_may_py {
                        self.py_typed_vars.insert(name_str.clone());
                    } else {
                        self.py_typed_vars.remove(&name_str);
                    }

                    // Infer 2-slot type from last_expr_type when store.ty is Unknown or narrower
                    // Double expression result overrides Float/Unknown store type
                    // Uint expression result overrides Int/Unknown store type
                    if matches!(self.last_expr_type, ObjectType::Double) {
                        if matches!(store.ty, Type::Unknown) || matches!(store.ty, Type::Float) {
                            self.var_types.insert(name_str.clone(), Type::Double);
                        }
                    } else if matches!(self.last_expr_type, ObjectType::String) {
                        if matches!(store.ty, Type::Unknown) {
                            self.var_types.insert(name_str.clone(), Type::StrFixed(0));
                        }
                    } else if matches!(self.last_expr_type, ObjectType::Uint) {
                        if matches!(store.ty, Type::Unknown) || matches!(store.ty, Type::Int) {
                            self.var_types.insert(name_str.clone(), Type::U64);
                        }
                    } else if matches!(self.last_expr_type, ObjectType::NestedObject) {
                        // Plan 197 Task 13: Track enum variant instance types for field access
                        // Plan 197 Task 16: Only override if type wasn't already inferred (e.g., from fn_return_types)
                        if let Some(ref variant_mono) = self.last_enum_variant_mono {
                            let existing_type = self.var_types.get(&name_str);
                            let should_override = matches!(
                                existing_type,
                                Some(Type::Unknown) | Some(Type::Int) | None
                            );
                            if should_override {
                                let type_decl = crate::ast::TypeDecl {
                                    consts: Vec::new(),
                                    name: crate::ast::Name::from(variant_mono.clone()),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: vec![],
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    impl_attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                };
                                self.var_types
                                    .insert(name_str.clone(), Type::User(type_decl));
                                vm_debug!(
                                    "DEBUG: Stored enum variant type '{}' for variable '{}'",
                                    variant_mono,
                                    name_str
                                );
                            }
                            self.last_enum_variant_mono = None;
                        }
                    }
                }

                // Promote i32 to u64 if variable type is u64/i64 but expression is not 64-bit
                let stored_type = self.var_types.get(&name_str).cloned();
                // PLAN-026 T-03 配套: I64 目标走 TYPE_CAST_I64(sign-extend +
                // TAG_I64);U64 保持零扩展。此前共用 TYPE_CAST_U64——i64 var
                // 持 U64 tag,I64 语义链(下标/串化/数值比较)全不接。
                if matches!(stored_type, Some(Type::I64)) && !self.contains_u64(&store.expr) {
                    self.emit(OpCode::TYPE_CAST_I64);
                } else if matches!(stored_type, Some(Type::U64)) && !self.contains_u64(&store.expr)
                {
                    self.emit(OpCode::TYPE_CAST_U64);
                } else if matches!(stored_type, Some(Type::Double))
                    && self.last_expr_type != ObjectType::Double
                {
                    match self.last_expr_type {
                        ObjectType::Float => {
                            self.emit(OpCode::PROMOTE_F64);
                        }
                        ObjectType::Int | ObjectType::Byte | ObjectType::Bool => {
                            self.emit(OpCode::I32_TO_F32);
                            self.emit(OpCode::PROMOTE_F64);
                        }
                        ObjectType::Uint => {
                            self.emit(OpCode::U64_TO_F64);
                        }
                        ObjectType::NestedObject | ObjectType::Void | ObjectType::String => {
                            // Opaque handle or non-numeric — skip f64 coercion
                            // The value stays as 1-slot i32 handle on stack
                        }
                        _ => {
                            self.emit(OpCode::PROMOTE_F64);
                        }
                    }
                }
                // Plan 437: float 声明的 i32→f32 存储强转（与上面 Double 臂对称）。
                // 此前只支持 Double——`var f float = <int>` 不转换，int 位模式
                // 直接当 f32 消费（chart 几何 vmax==0.0 恒真、y 全落底边的病灶）。
                else if matches!(stored_type, Some(Type::Float))
                    && self.last_expr_type != ObjectType::Float
                {
                    match self.last_expr_type {
                        ObjectType::Int | ObjectType::Byte | ObjectType::Bool => {
                            self.emit(OpCode::I32_TO_F32);
                        }
                        // Double → Float 无降精度指令；值保持 f64 tag，由
                        // tag 驱动的下游（算术/提取）自行解码。
                        _ => {}
                    }
                }

                // Re-run the initializer-type tracking AFTER the annotation/
                // inference chain above, so a constructor-derived type (e.g.
                // `let s = String.from(..)` -> User("String")) wins over the
                // native return type the chain may have recorded (StrFixed).
                // This preserves the pre-refactor ordering for the local path;
                // the top-of-handler call covers the global early-return path.
                self.track_store_var_type(store);

                // Add variable to symbol table and get its index.
                //
                // Key insight: StoreKind::Var is used for BOTH `var x = 5` (new declaration)
                // and `x = 5` (reassignment). There is no separate Assign kind.
                //
                // Look up the variable across ALL scopes first:
                //   - If found: this is a reassignment or same-name re-declaration.
                //     For StoreKind::Let, always shadow (create new slot at inner scope).
                //     For StoreKind::Var/CVar, reuse the existing slot (don't shadow outer scope).
                //   - If not found: always create a new slot via add_var.
                //
                // This fixes the bug where `__out__ = __out__ + ...` inside a for-loop body
                // would create a new `__out__` slot in the inner scope instead of updating
                // the outer scope's slot.
                let var_index = if let Some(existing_index) = self.lookup_var(&name_str) {
                    match store.kind {
                        crate::ast::StoreKind::Let
                        | crate::ast::StoreKind::Const
                        | crate::ast::StoreKind::Shared => {
                            // `let x = ...` always creates a new slot, even if x exists in outer scope
                            self.add_var(&store.name)
                        }
                        crate::ast::StoreKind::Var | crate::ast::StoreKind::CVar => {
                            // `var x = ...` or `x = ...` (reassignment) reuses the existing slot
                            // from any scope to avoid accidental inner-scope shadowing
                            existing_index
                        }
                        crate::ast::StoreKind::Field => {
                            // Struct field: create new slot
                            self.add_var(&store.name)
                        }
                    }
                } else {
                    // Variable not found anywhere: always create new slot
                    self.add_var(&store.name)
                };

                // Plan 377: 全值单槽化 —— 恒为单槽 STORE_LOC。
                let stored_type = self.var_types.get(&name_str).cloned();
                self.emit_store_loc(var_index);
                // Plan 378 遗留：为未标注 binding（如 `let n = s.to_uint()`）推断类型，
                // 供后续 print 路由 / opcode 选择使用（单槽时代仅影响类型，不影响 slot）。
                if stored_type.is_none()
                    && matches!(self.last_expr_type, ObjectType::Double | ObjectType::Uint)
                {
                    let inferred = match self.last_expr_type {
                        ObjectType::Double => Type::Double,
                        _ => Type::U64,
                    };
                    self.var_types.insert(name_str.clone(), inferred);
                }

                // Plan 080: DON'T load the value back to stack
                // This avoids overlapping variable storage and stack
                // REPL will display the value from the expression result on stack
            }
            Stmt::Return(expr) => {
                self.compile_expr(expr)?;
                // PLAN-776 R5-01：SSE 凭据包装移至 generator 生产点
                //（CREATE_GENERATOR 后，Call 臂）——`return <expr>` 不再
                // 包装。生产点包装覆盖直接调用/局部/helper/Internal 尾等
                // 全部传播形态；返回的普通数值恒为数据（R2-01 反例保持）。
                let n_args = self.current_fn_n_args as u8;
                // Plan 377: 全值单槽化后恒用 RET。f32→f64 仍需 PROMOTE_F64 编码提升。
                if matches!(self.current_fn_ret_type, Type::Double)
                    && !matches!(self.last_expr_type, ObjectType::Double)
                    && matches!(self.last_expr_type, ObjectType::Float)
                {
                    self.emit(OpCode::PROMOTE_F64);
                }
                self.emit(OpCode::RET);
                self.code.push(n_args);
            }
            // Plan 124 Phase 2.3: reply statement for ask/reply RPC
            // reply expr -> compile expr, then send to oneshot channel
            Stmt::Reply(expr) => {
                // Phase 2.3: Simplified implementation
                // Full implementation would:
                // 1. Look up the reply channel from the current message context
                // 2. Compile the expression
                // 3. Send the value through the channel
                // For now, just compile the expression and leave it on stack
                self.compile_expr(expr)?;
                // TODO: Implement actual channel send when oneshot channels are ready
            }
            // Plan 073: TypeDecl support - register type metadata
            Stmt::TypeDecl(type_decl) => {
                // Register the type in the type registry
                self.register_type(type_decl);

                // Plan 417-E2 (DIV-TRAIT-LANG-1): associated types have no
                // runtime entity — methods are compiled with `Item` swapped
                // for the concrete binding declared at the impl clause
                // (`type IntBox as Container<Item=int>`). Bindings from all
                // implemented specs merge into one table (assoc names are
                // unique per spec; cross-spec collisions are pathological).
                let (assoc_names, assoc_types): (Vec<crate::ast::Name>, Vec<Type>) = {
                    let mut names = Vec::new();
                    let mut types = Vec::new();
                    for spec_impl in &type_decl.spec_impls {
                        for (n, t) in &spec_impl.assoc_bindings {
                            names.push(n.clone());
                            types.push(t.clone());
                        }
                    }
                    (names, types)
                };
                let subst_assoc = |ty: &Type| -> Type {
                    if assoc_names.is_empty() {
                        ty.clone()
                    } else {
                        ty.substitute(&assoc_names, &assoc_types)
                    }
                };

                // Plan 087 Phase 3: Compile type methods as standalone functions
                // Method naming: TypeName.method_name (e.g., Counter.increment)
                // self becomes the first parameter
                let type_name = type_decl.name.to_string();
                let member_names: Vec<String> = type_decl
                    .members
                    .iter()
                    .map(|m| m.name.to_string())
                    .collect();
                for method in &type_decl.methods {
                    // Create mangled method name: TypeName.method_name
                    let mangled_name = format!("{}.{}", type_name, method.name);

                    // Clone and modify the method for standalone compilation
                    let mut method_fn = method.clone();
                    method_fn.name = crate::ast::Name::from(mangled_name.as_str());
                    method_fn.parent = Some(crate::ast::Name::from(type_name.as_str()));
                    // Plan 417-E2: substitute associated-type references in
                    // the compiled signature (params + return).
                    for param in &mut method_fn.params {
                        param.ty = subst_assoc(&param.ty);
                    }
                    method_fn.ret = subst_assoc(&method_fn.ret);

                    // For instance methods (non-static), inject 'self' as first parameter
                    if !method.is_static {
                        let has_self = method_fn
                            .params
                            .first()
                            .map(|p| p.name.to_string() == "self")
                            .unwrap_or(false);
                        if !has_self {
                            method_fn.params.insert(
                                0,
                                crate::ast::Param {
                                    name: crate::ast::Name::from("self"),
                                    ty: Type::User(type_decl.clone()),
                                    default: None,
                                    mode: crate::ast::ParamMode::View,
                                    destructure: None,
                                },
                            );
                        }
                    }

                    // Store member names for implicit field access resolution
                    self.current_type_members = Some(member_names.clone());

                    // Compile as a standalone function
                    self.compile_stmt(&Stmt::Fn(method_fn))?;

                    self.current_type_members = None;
                }

                // Plan 417-E4 (DIV-TRAIT-VM-2): synthesize INHERITED spec
                // default methods the type did not re-declare. The spec body
                // is compiled exactly like a declared method (mangled
                // Type.method, self injected as the first param), so
                // `r.announce()` on an implementer resolves without the
                // historical re-declaration workaround.
                for spec_name in &type_decl.specs {
                    // Plan 417-E4: look up via the shared TypeStore — specs are
                    // pre-registered there before codegen (compile.rs), so the
                    // synthesis is independent of statement order (the script
                    // path compiles TypeDecls BEFORE other statements).
                    let spec_decl = {
                        let ts = self.type_store.read().unwrap();
                        ts.lookup_spec_decl(spec_name).cloned()
                    };
                    let Some(spec_decl) = spec_decl else {
                        continue; // unknown spec — checker reports
                    };
                    for spec_method in &spec_decl.methods {
                        let Some(default_body) = &spec_method.body else {
                            continue; // abstract method — checker enforces
                        };
                        if type_decl.methods.iter().any(|m| m.name == spec_method.name) {
                            continue; // implementer overrides the default
                        }
                        let mangled_name = format!("{}.{}", type_name, spec_method.name);
                        let mut method_fn = crate::ast::Fn {
                            name: crate::ast::Name::from(mangled_name.as_str()),
                            params: spec_method.params.clone(),
                            ret: spec_method.ret.clone(),
                            body: match default_body.as_ref() {
                                crate::ast::Expr::Block(b) => b.clone(),
                                other => crate::ast::Body {
                                    stmts: vec![Stmt::Expr(other.clone())],
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        };
                        // Plan 417-E2: the synthesized default inherits the
                        // spec signature — substitute associated-type
                        // references (Item → the implementer's binding) so the
                        // compiled Type.method matches the declared methods.
                        for param in &mut method_fn.params {
                            param.ty = subst_assoc(&param.ty);
                        }
                        method_fn.ret = subst_assoc(&method_fn.ret);
                        method_fn.parent = Some(crate::ast::Name::from(type_name.as_str()));
                        let has_self = method_fn
                            .params
                            .first()
                            .map(|p| p.name.to_string() == "self")
                            .unwrap_or(false);
                        if !has_self {
                            method_fn.params.insert(
                                0,
                                crate::ast::Param {
                                    name: crate::ast::Name::from("self"),
                                    ty: Type::User(type_decl.clone()),
                                    default: None,
                                    mode: crate::ast::ParamMode::View,
                                    destructure: None,
                                },
                            );
                        }
                        self.current_type_members = Some(member_names.clone());
                        self.compile_stmt(&Stmt::Fn(method_fn))?;
                        self.current_type_members = None;
                    }
                }
            }
            Stmt::Ext(ext_block) => {
                // Compile ext methods as standalone functions (same pattern as TypeDecl methods)
                let type_name = ext_block.target.to_string();

                // Look up the TypeDecl for self parameter typing
                let self_type = self
                    .infer_ctx
                    .lookup_type_decl(&ext_block.target)
                    .map(|td| Type::User(td))
                    .unwrap_or(Type::Unknown);

                // Store member names for implicit .field resolution
                let member_names: Vec<String> = self
                    .get_type(&type_name)
                    .map(|ti| ti.member_names.clone())
                    .unwrap_or_default();

                for method in &ext_block.methods {
                    let mangled_name = format!("{}.{}", type_name, method.name);
                    let mut method_fn = method.clone();
                    method_fn.name = crate::ast::Name::from(mangled_name.as_str());
                    method_fn.parent = Some(crate::ast::Name::from(type_name.as_str()));

                    if !method.is_static {
                        let has_self = method_fn
                            .params
                            .first()
                            .map(|p| p.name.to_string() == "self")
                            .unwrap_or(false);
                        if !has_self {
                            method_fn.params.insert(
                                0,
                                crate::ast::Param {
                                    name: crate::ast::Name::from("self"),
                                    ty: self_type.clone(),
                                    default: None,
                                    mode: crate::ast::ParamMode::View,
                                    destructure: None,
                                },
                            );
                        }
                    }

                    // Set member names for implicit .field access resolution
                    self.current_type_members = Some(member_names.clone());
                    self.compile_stmt(&Stmt::Fn(method_fn))?;

                    // Also register short name (e.g., "upper") as export alias
                    // so `use auto.str: upper` can resolve via linker
                    let mangled_entry = self.exports.get(&mangled_name).copied();
                    if let Some(addr) = mangled_entry {
                        self.exports.insert(method.name.to_string(), addr);
                    }

                    self.current_type_members = None;
                }
            }
            Stmt::EnumDecl(enum_decl) => {
                // Register enum variant values for Cover(TagCover) compilation
                let enum_name = enum_decl.name.to_string();
                for (i, item) in enum_decl.items.iter().enumerate() {
                    let value = item.scalar_value.unwrap_or(i as i32);
                    let key = format!("{}.{}", enum_name, item.name);
                    self.enum_values.insert(key, value);
                }

                // Plan 197 Task 11: Register enum data variants in GenericRegistry
                // Each variant with a payload type gets registered as a template
                // so the VM can instantiate them at runtime.
                use crate::vm::generic_registry::{ClassTemplate, FieldDef};

                // Determine payload type for each item based on enum kind
                let homogeneous_payload = match &enum_decl.kind {
                    crate::ast::EnumKind::Homogeneous { payload_type } => {
                        Some(payload_type.clone())
                    }
                    _ => None,
                };

                for item in &enum_decl.items {
                    let variant_mono = format!("{}.{}", enum_decl.name, item.name);

                    // Plan 201 Phase 1C: Multi-field struct-like variant
                    if item.has_fields() {
                        let fields: Vec<FieldDef> = item
                            .fields
                            .iter()
                            .map(|f| FieldDef::new(f.name.as_str(), f.field_type.clone()))
                            .collect();
                        let template = ClassTemplate::new(&variant_mono, vec![], fields, vec![]);
                        self.generic_registry.register_or_update_template(template);
                        continue;
                    }

                    // Single-field payload variant (existing code)
                    let payload = item.payload_type.as_ref().or(homogeneous_payload.as_ref());

                    if let Some(payload_type) = payload {
                        // Handle tuple payload like Rect(int, int) -> fields _0, _1
                        let fields = if let crate::ast::Type::Tuple(ref types) = payload_type {
                            types
                                .iter()
                                .enumerate()
                                .map(|(i, t)| FieldDef::new(&format!("_{}", i), t.clone()))
                                .collect()
                        } else {
                            vec![FieldDef::new("_0", payload_type.clone())]
                        };
                        let template = ClassTemplate::new(
                            &variant_mono,
                            vec![], // No generic params for enum variants
                            fields,
                            vec![], // No methods for enum variants
                        );
                        self.generic_registry.register_or_update_template(template);
                    } else if !item.payload_types.is_empty() {
                        // Multi-field anonymous variant: ToolUse str str str -> fields _0, _1, _2
                        let fields: Vec<FieldDef> = item
                            .payload_types
                            .iter()
                            .enumerate()
                            .map(|(i, t)| FieldDef::new(&format!("_{}", i), t.clone()))
                            .collect();
                        let template = ClassTemplate::new(&variant_mono, vec![], fields, vec![]);
                        self.generic_registry.register_or_update_template(template);
                    }
                }

                // Plan 325 缺陷 1: 编译 enum 实例方法为独立函数（仿 TypeDecl methods，
                // codegen.rs:2181）。parser 把 enum 方法存进 EnumKind::Heterogeneous { methods }，
                // 但 codegen 此前从未提取编译 —— 导致 Color.Red.name() / Result.is_ok() 等
                // enum 实例方法调用时无目标，静默返回 None 或 CALL_SPEC 报错。
                // 方法名 mangle 为 EnumName.method，实例方法注入 self（enum 值）作首参。
                if let crate::ast::EnumKind::Heterogeneous { methods, .. } = &enum_decl.kind {
                    for method in methods {
                        let mangled_name = format!("{}.{}", enum_name, method.name);
                        let mut method_fn = method.clone();
                        method_fn.name = crate::ast::Name::from(mangled_name.as_str());
                        method_fn.parent = Some(crate::ast::Name::from(enum_name.as_str()));
                        // 实例方法（非 static）注入 self 作首参 —— 持有 enum 值
                        // （scalar 变体是 i32 discriminant，带 payload 变体是 heap 对象 id）。
                        // 方法体内通过 `is self {...}` 匹配变体（既有 is 机制按值工作）。
                        if !method.is_static {
                            let has_self = method_fn
                                .params
                                .first()
                                .map(|p| p.name.to_string() == "self")
                                .unwrap_or(false);
                            if !has_self {
                                method_fn.params.insert(
                                    0,
                                    crate::ast::Param {
                                        name: crate::ast::Name::from("self"),
                                        ty: Type::Unknown, // enum 值类型（运行时 i32 或对象 id）
                                        default: None,
                                        mode: crate::ast::ParamMode::View,
                                        destructure: None,
                                    },
                                );
                            }
                        }
                        self.compile_stmt(&Stmt::Fn(method_fn))?;
                    }
                }
            }
            Stmt::SpecDecl(spec_decl) => {
                // Plan 073 Phase 8.6: Spec declaration support
                // Spec declarations (traits) don't generate bytecode at compile time
                // They register method signatures for type checking and constraint validation
                // For now, specs are metadata-only and used during type checking
            }
            // Plan 073: For statement support
            Stmt::For(for_stmt) => {
                vm_debug!("DEBUG FOR: Compiling for loop");
                // Push new loop exit + continue tracking
                self.loop_exits.push(Vec::new());
                self.loop_continues.push(Vec::new());
                self.loop_continue_positions.push(0); // placeholder, will be set per variant

                // Handle range-based for loops: for x in start..end { ... }
                // Only support simple range iteration for now
                match &for_stmt.iter {
                    Iter::Named(var_name) => {
                        // Check if range is a Range expression (for x in 0..10)
                        if let Expr::Range(range) = &for_stmt.range {
                            vm_debug!(
                                "DEBUG FOR: Range-based loop, start={:?}, end={:?}, eq={}",
                                range.start,
                                range.end,
                                range.eq
                            );
                            // Compile start expression and initialize loop variable
                            self.compile_expr(&range.start)?;
                            vm_debug!(
                                "DEBUG FOR: After start expr, code len = {}",
                                self.code.len()
                            );

                            // Store to loop variable
                            let var_str = var_name.to_string();
                            vm_debug!("DEBUG FOR: Loop var = {}", var_str);
                            self.push_scope(); // New scope for loop variable
                                               // Calculate total index across all scopes
                            let var_index = self.add_var(&var_str);
                            vm_debug!("DEBUG FOR: var_index = {}", var_index);
                            self.emit_store_loc(var_index);
                            vm_debug!("DEBUG FOR: After store_loc, code len = {}", self.code.len());

                            // Loop start label
                            let loop_start = self.code.len();
                            vm_debug!("DEBUG FOR: Loop start at {}", loop_start);

                            // Load loop variable
                            self.emit_load_loc(var_index);

                            // Compile end expression
                            self.compile_expr(&range.end)?;

                            // Compare: if range.eq is true, use LE (<=), else use LT (<)
                            if range.eq {
                                self.emit(OpCode::LE); // Inclusive range: start..=end
                            } else {
                                self.emit(OpCode::LT); // Exclusive range: start..end
                            }

                            // JMP_IF_Z to end (exit loop if condition false)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Compile loop body — discard body's return value
                            let old_pop = self.should_pop_expr_result;
                            self.should_pop_expr_result = true;
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                            self.should_pop_expr_result = old_pop;

                            // Continue target: increment loop variable
                            let continue_pos = self.code.len();
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = continue_pos;
                            }
                            self.emit_load_loc(var_index);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(1);
                            self.emit(OpCode::ADD);
                            self.emit_store_loc(var_index);

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len();
                            // Offset is from IP after reading the offset (current_pos + 2)
                            self.emit_i16((loop_start as isize - current_pos as isize - 2) as i16);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            // Patch all continue statements to the increment position
                            let continues = self.loop_continues.pop().unwrap();
                            for continue_placeholder in continues {
                                self.patch_jump_to(continue_placeholder, continue_pos);
                            }
                            self.loop_continue_positions.pop();
                        } else if let Expr::Call(_call) = &for_stmt.range {
                            // Plan 454 E5b(§M 缺口③) + PLAN-057 T3 泛化(等价性
                            // 缺陷族②):Call 源统一走「临时数组句柄 + 索引循环 +
                            // GET_ELEM」通道——默认的 .iter() 迭代器通道对 List
                            // 句柄零迭代(句柄≠iterator)。原 E5b 仅按名放行
                            // .values/.keys;泛化后承接全部 Call 源,例外仅留
                            // 迭代器协议族与流式源(惰性拉帧,len() 不可表达):
                            //   - .iter/.take/.skip/.rev/.chain/.zip 适配链
                            //     (通道选择由 vm_types 编译形态测试钉住)
                            //   - 生成器 fn 调用(generator_fns 在案——
                            //     CREATE_GENERATOR 产物是迭代器对象,索引通道
                            //     len() 不可表达;generator_tests 5 例回归钉)
                            //   - 名含 stream / sse_ 前缀(plan341 SSE 惰性流)
                            //   - 未知 callee 形态保守走原迭代器通道
                            let callee_iter_protocol = match &for_stmt.range {
                                Expr::Call(c) => {
                                    let seg = match c.name.as_ref() {
                                        Expr::Dot(_, m) => Some(m.as_str()),
                                        Expr::Ident(n) => Some(n.as_str()),
                                        _ => None,
                                    };
                                    match seg {
                                        Some(m) => {
                                            self.generator_fns.contains(m)
                                                || matches!(
                                                    m,
                                                    "iter"
                                                        | "take"
                                                        | "skip"
                                                        | "rev"
                                                        | "chain"
                                                        | "zip"
                                                )
                                                || m.contains("stream")
                                                || m.starts_with("sse_")
                                        }
                                        None => true,
                                    }
                                }
                                _ => false,
                            };
                            if !callee_iter_protocol {
                                self.push_scope(); // 迭代变量 + 句柄 + 计数器
                                self.compile_expr(&for_stmt.range)?;
                                let arr_index = self.add_var("_iterarr");
                                self.emit_store_loc(arr_index);

                                let var_str = var_name.to_string();
                                let var_index = self.add_var(&var_str);
                                // 计数器置 0
                                let idx_index = self.add_var("_iteridx");
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(0);
                                self.emit_store_loc(idx_index);

                                let list_len_native = BIGVM_NATIVES
                                    .lock()
                                    .unwrap()
                                    .resolve_qualified("auto.list.len");

                                let loop_start = self.code.len();
                                // 条件: idx < len
                                self.emit_load_loc(idx_index);
                                if let Some(len_id) = list_len_native {
                                    self.emit_load_loc(arr_index);
                                    self.emit(OpCode::CALL_NAT);
                                    self.code.extend_from_slice(&len_id.to_le_bytes());
                                } else {
                                    return Err(AutoError::Msg(
                                        "auto.list.len native not found".to_string(),
                                    ));
                                }
                                self.emit(OpCode::LT);
                                self.emit(OpCode::JMP_IF_Z);
                                let jump_to_end = self.emit_placeholder_i16();

                                // 元素: e = arr[idx]
                                self.emit_load_loc(arr_index);
                                self.emit_load_loc(idx_index);
                                self.emit(OpCode::GET_ELEM);
                                self.emit_store_loc(var_index);

                                // 循环体
                                let old_pop = self.should_pop_expr_result;
                                self.should_pop_expr_result = true;
                                self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                                self.should_pop_expr_result = old_pop;

                                // continue 目标: 计数器 +1
                                let continue_pos = self.code.len();
                                if let Some(pos) = self.loop_continue_positions.last_mut() {
                                    *pos = continue_pos;
                                }
                                self.emit_load_loc(idx_index);
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(1);
                                self.emit(OpCode::ADD);
                                self.emit_store_loc(idx_index);
                                self.emit(OpCode::JMP);
                                let cur_pos = self.code.len();
                                self.emit_i16((loop_start as isize - cur_pos as isize - 2) as i16);

                                let _loop_exit = self.code.len();
                                self.patch_jump(jump_to_end);
                                self.pop_scope();
                                let exits = self.loop_exits.pop().unwrap();
                                for exit_placeholder in exits {
                                    self.patch_jump(exit_placeholder);
                                }
                                let continues = self.loop_continues.pop().unwrap();
                                for cp in continues {
                                    self.patch_jump_to(cp, continue_pos);
                                }
                                self.loop_continue_positions.pop();
                                return Ok(());
                            }
                            // Plan 073: Iterator-based for loop: for x in list.iter() { ... }
                            // Compile the iterator call to get the iterator object
                            self.compile_expr(&for_stmt.range)?;

                            // Store iterator in a local variable
                            self.push_scope(); // New scope for loop variable and iterator
                            let iter_index = self.add_var("_iterator");
                            self.emit_store_loc(iter_index);

                            // Loop start label
                            let loop_start = self.code.len();

                            // Call iter.next() to get next element
                            self.emit_load_loc(iter_index); // Load iterator

                            // Emit CALL_NAT for Iterator.next
                            // Look up the native function ID
                            let native_id = if let Some(id) = BIGVM_NATIVES
                                .lock()
                                .unwrap()
                                .resolve_qualified("auto.iterator.next")
                            {
                                id
                            } else {
                                self.loop_exits.pop();
                                return Err(AutoError::Msg(
                                    "Iterator.next native function not found".to_string(),
                                ));
                            };
                            self.emit(OpCode::CALL_NAT);
                            self.emit_u16(native_id);

                            // Check if result is nil (end of iteration)
                            // Nil is represented as -1 in our VM
                            // DUP the result so we can both check for nil AND store to loop variable
                            self.emit(OpCode::DUP);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(-1);
                            self.emit(OpCode::EQ);
                            self.emit(OpCode::JMP_IF_NZ);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Store the element to the loop variable
                            let var_str = var_name.to_string();
                            self.var_types.insert(var_str.clone(), Type::Int);
                            let var_index = self.add_var(&var_str);
                            self.emit_store_loc(var_index);

                            // Continue target: loop_start (re-check iterator.next())
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = loop_start as usize;
                            }

                            // Compile loop body — discard body's return value
                            let old_pop = self.should_pop_expr_result;
                            self.should_pop_expr_result = true;
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                            self.should_pop_expr_result = old_pop;

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len();
                            // Offset is from IP after reading the offset (current_pos + 2)
                            self.emit_i16((loop_start as isize - current_pos as isize - 2) as i16);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            self.loop_continue_positions.pop();
                            // Plan 447 H2 fix: Patch continue statements to loop start (re-check iterator.next())
                            let continues = self.loop_continues.pop().unwrap();
                            for continue_placeholder in continues {
                                self.patch_jump_to(continue_placeholder, loop_start as usize);
                            }
                        } else {
                            // PLAN-707 D-5 形态 2：变量形态——持流变量
                            //（`var s = Http.get_stream(..)` 登记的 stream_vars）
                            // 走迭代器通道：CALL auto.iterator.next 逐拉（句柄
                            // 即流 id，shim 的流句柄惰性消费臂就地 pull，无数据
                            // 时挂 waiting_http_stream_id）。数组通道对句柄源
                            // 零迭代（ARRAY_LEN 不可表达），故此前置分流。
                            let stream_source = match &for_stmt.range {
                                Expr::Ident(name) => self.stream_vars.contains(name.as_str()),
                                _ => false,
                            };
                            if stream_source {
                                self.push_scope(); // _iterator + 循环变量
                                self.compile_expr(&for_stmt.range)?;
                                let iter_index = self.add_var("_iterator");
                                self.emit_store_loc(iter_index);

                                let loop_start = self.code.len();
                                self.emit_load_loc(iter_index);
                                let native_id = BIGVM_NATIVES
                                    .lock()
                                    .unwrap()
                                    .resolve_qualified("auto.iterator.next")
                                    .ok_or_else(|| {
                                        AutoError::Msg(
                                            "Iterator.next native function not found".to_string(),
                                        )
                                    })?;
                                self.emit(OpCode::CALL_NAT);
                                self.emit_u16(native_id);
                                self.emit(OpCode::DUP);
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(-1);
                                self.emit(OpCode::EQ);
                                self.emit(OpCode::JMP_IF_NZ);
                                let jump_to_end = self.emit_placeholder_i16();

                                let var_str = var_name.to_string();
                                self.var_types.insert(var_str.clone(), Type::StrOwned);
                                let var_index = self.add_var(&var_str);
                                self.emit_store_loc(var_index);

                                if let Some(pos) = self.loop_continue_positions.last_mut() {
                                    *pos = loop_start as usize;
                                }
                                let old_pop = self.should_pop_expr_result;
                                self.should_pop_expr_result = true;
                                self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                                self.should_pop_expr_result = old_pop;

                                self.emit(OpCode::JMP);
                                let current_pos = self.code.len();
                                self.emit_i16(
                                    (loop_start as isize - current_pos as isize - 2) as i16,
                                );

                                let _loop_exit = self.code.len();
                                self.patch_jump(jump_to_end);
                                self.pop_scope();
                                let exits = self.loop_exits.pop().unwrap();
                                for exit_placeholder in exits {
                                    self.patch_jump(exit_placeholder);
                                }
                                let continues = self.loop_continues.pop().unwrap();
                                for continue_placeholder in continues {
                                    self.patch_jump_to(continue_placeholder, loop_start as usize);
                                }
                                self.loop_continue_positions.pop();
                                return Ok(());
                            }
                            // Plan 089: Array-based for loop: for x in expr { ... }
                            // Supports any iterable expression: variable, field access, call, etc.
                            // Infer element type before compiling range (for var_types tracking)
                            let elem_type = {
                                let range_type = self.infer_expr_type(&for_stmt.range);
                                match &range_type {
                                    Type::Array(arr) => (*arr.elem).clone(),
                                    Type::RuntimeArray(rta) => (*rta.elem).clone(),
                                    Type::StrFixed(_) | Type::StrOwned => Type::Char,
                                    _ => {
                                        // Try var_types for the range variable
                                        if let Expr::Ident(name) = &for_stmt.range {
                                            self.var_types
                                                .get(name.as_str())
                                                .and_then(|t| match t {
                                                    Type::Array(arr) => Some((*arr.elem).clone()),
                                                    Type::RuntimeArray(rta) => {
                                                        Some((*rta.elem).clone())
                                                    }
                                                    Type::List(elem) => Some((**elem).clone()),
                                                    _ => None,
                                                })
                                                .unwrap_or(Type::Unknown)
                                        } else {
                                            Type::Unknown
                                        }
                                    }
                                }
                            };

                            // Load the array variable
                            self.compile_expr(&for_stmt.range)?;

                            // DUP to keep array reference for GET_ELEM later
                            self.emit(OpCode::DUP);

                            // Get array length (consumes the duped array_id)
                            self.emit(OpCode::ARRAY_LEN);

                            // Initialize loop counter to 0
                            // Stack now: [array_id, length]
                            self.emit(OpCode::CONST_0);
                            // Stack now: [array_id, length, 0]

                            // Store to loop variable
                            let var_str = var_name.to_string();
                            self.push_scope(); // New scope for loop variable

                            // Store counter (0) - stack top
                            let counter_index = self.add_var("_counter");
                            self.emit_store_loc(counter_index);
                            // Stack now: [array_id, length]

                            // Store length
                            let len_index = self.add_var("_array_len");
                            self.emit_store_loc(len_index);
                            // Stack now: [array_id]

                            // Store array reference for GET_ELEM
                            let array_ref_index = self.add_var("_array_ref");
                            self.emit_store_loc(array_ref_index);
                            // Stack now: []

                            // Create the actual loop variable slot (will be overwritten each iteration)
                            let var_index = self.add_var(&var_str);

                            // Plan 201: Register element type for method dispatch on loop variable
                            if !matches!(elem_type, Type::Unknown) {
                                self.var_types.insert(var_str.clone(), elem_type);
                            }

                            // Loop start label
                            let loop_start = self.code.len();

                            // Load counter for comparison
                            self.emit_load_loc(counter_index);

                            // Compare with length
                            self.emit_load_loc(len_index);
                            self.emit(OpCode::LT);

                            // JMP_IF_Z to end (exit loop if counter >= length)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Load array reference
                            self.emit_load_loc(array_ref_index);

                            // Load current counter as index
                            self.emit_load_loc(counter_index);

                            // Get element at index (GET_ELEM: array_id, index -> value)
                            self.emit(OpCode::GET_ELEM);

                            // Store element to loop variable
                            self.emit_store_loc(var_index);

                            // Compile loop body — discard body's return value
                            let old_pop = self.should_pop_expr_result;
                            self.should_pop_expr_result = true;
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                            self.should_pop_expr_result = old_pop;

                            // Continue target: increment counter
                            let continue_pos = self.code.len();
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = continue_pos;
                            }
                            self.emit_load_loc(counter_index);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(1);
                            self.emit(OpCode::ADD);
                            self.emit_store_loc(counter_index);

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len();
                            self.emit_i16((loop_start as isize - current_pos as isize - 2) as i16);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            self.loop_continue_positions.pop();
                            // Plan 447 H2 fix: Patch continue statements to increment position
                            let continues = self.loop_continues.pop().unwrap();
                            for continue_placeholder in continues {
                                self.patch_jump_to(continue_placeholder, continue_pos);
                            }
                        }
                    }
                    Iter::Indexed(index_name, iter_name) => {
                        // Plan 073: Indexed iteration: for i, x in 0..10 { ... }
                        // Check if range is a Range expression
                        if let Expr::Range(range) = &for_stmt.range {
                            // Compile start expression and initialize loop variables
                            self.compile_expr(&range.start)?;

                            // Store to both index and value variables
                            let index_str = index_name.to_string();
                            let iter_str = iter_name.to_string();
                            self.push_scope(); // New scope for loop variables

                            // Store to index variable
                            let index_index = self.add_var(&index_str);
                            self.emit_store_loc(index_index);

                            // Store same value to iter variable
                            let iter_index = self.add_var(&iter_str);
                            self.emit_store_loc(iter_index);

                            // Loop start label
                            let loop_start = self.code.len();

                            // Load index variable for comparison
                            self.emit_load_loc(index_index);

                            // Compile end expression
                            self.compile_expr(&range.end)?;

                            // Compare: if range.eq is true, use LE (<=), else use LT (<)
                            if range.eq {
                                self.emit(OpCode::LE); // Inclusive range: start..=end
                            } else {
                                self.emit(OpCode::LT); // Exclusive range: start..end
                            }

                            // JMP_IF_Z to end (exit loop if condition false)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Compile loop body — discard body's return value
                            let old_pop = self.should_pop_expr_result;
                            self.should_pop_expr_result = true;
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                            self.should_pop_expr_result = old_pop;

                            // Continue target: increment both loop variables
                            let continue_pos = self.code.len();
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = continue_pos;
                            }
                            self.emit_load_loc(index_index);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(1);
                            self.emit(OpCode::ADD);
                            self.emit_store_loc(index_index);

                            // Update iter variable to match index
                            self.emit_load_loc(index_index);
                            self.emit_store_loc(iter_index);

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len();
                            // Offset is from IP after reading the offset (current_pos + 2)
                            self.emit_i16((loop_start as isize - current_pos as isize - 2) as i16);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            self.loop_continue_positions.pop();
                            // Plan 447 H2 fix: Patch continue statements to increment position
                            let continues = self.loop_continues.pop().unwrap();
                            for continue_placeholder in continues {
                                self.patch_jump_to(continue_placeholder, continue_pos);
                            }
                        } else if let Expr::Ident(_) = &for_stmt.range {
                            // Plan 089: Indexed array iteration: for i, x in array_var { ... }
                            let index_str = index_name.to_string();
                            let iter_str = iter_name.to_string();
                            self.push_scope(); // New scope for loop variables

                            // Load the array variable
                            self.compile_expr(&for_stmt.range)?;
                            // Stack: [array_id]

                            // DUP to keep array reference
                            self.emit(OpCode::DUP);
                            // Stack: [array_id, array_id]

                            // Get array length (consumes one array_id)
                            self.emit(OpCode::ARRAY_LEN);
                            // Stack: [array_id, length]

                            // Initialize loop counter to 0
                            self.emit(OpCode::CONST_0);
                            // Stack: [array_id, length, 0]

                            // Store counter (0)
                            let counter_index = self.add_var("_counter");
                            self.emit_store_loc(counter_index);
                            // Stack: [array_id, length]

                            // Store length
                            let len_index = self.add_var("_array_len");
                            self.emit_store_loc(len_index);
                            // Stack: [array_id]

                            // Store array reference
                            let array_ref_index = self.add_var("_array_ref");
                            self.emit_store_loc(array_ref_index);
                            // Stack: []

                            // Store index variable
                            let index_var_index = self.add_var(&index_str);

                            // Store iter variable
                            let iter_var_index = self.add_var(&iter_str);

                            // Loop start label
                            let loop_start = self.code.len();

                            // Load counter
                            self.emit_load_loc(counter_index);

                            // Compare with length
                            self.emit_load_loc(len_index);
                            self.emit(OpCode::LT);

                            // JMP_IF_Z to end (exit loop if counter >= length)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Store current index to index variable
                            self.emit_load_loc(counter_index);
                            self.emit_store_loc(index_var_index);

                            // Load array reference
                            self.emit_load_loc(array_ref_index);

                            // Load current index
                            self.emit_load_loc(counter_index);

                            // Get element at index (GET_ELEM: array_id, index -> value)
                            self.emit(OpCode::GET_ELEM);

                            // Store to iter variable
                            self.emit_store_loc(iter_var_index);

                            // Compile loop body — discard body's return value
                            let old_pop = self.should_pop_expr_result;
                            self.should_pop_expr_result = true;
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                            self.should_pop_expr_result = old_pop;

                            // Continue target: increment counter
                            let continue_pos = self.code.len();
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = continue_pos;
                            }
                            self.emit_load_loc(counter_index);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(1);
                            self.emit(OpCode::ADD);
                            self.emit_store_loc(counter_index);

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len();
                            self.emit_i16((loop_start as isize - current_pos as isize - 2) as i16);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            self.loop_continue_positions.pop();
                            // Plan 447 H2 fix: Patch continue statements to increment position
                            let continues = self.loop_continues.pop().unwrap();
                            for continue_placeholder in continues {
                                self.patch_jump_to(continue_placeholder, continue_pos);
                            }
                        } else {
                            // For now, only support range and array identifier expressions
                            self.loop_exits.pop();
                            self.loop_continue_positions.pop();
                            return Err(AutoError::Msg(
                                "Indexed for loops with non-range/non-array expressions not supported yet"
                                    .to_string(),
                            ));
                        }
                    }
                    Iter::Destructured(key_name, val_name) => {
                        // for (k, v) in map { ... } — Map iteration with destructuring
                        self.push_scope();

                        // Compile the iterable expression (the map)
                        self.compile_expr(&for_stmt.range)?;
                        // Stack: [map_id]

                        // Save map reference
                        let map_ref_index = self.add_var("_map_ref");
                        self.emit_store_loc(map_ref_index);
                        // Stack: []

                        // Load map ref and call auto.hashmap.keys to get keys list
                        self.emit_load_loc(map_ref_index);
                        // Stack: [map_id]

                        // Emit CALL_NAT for auto.hashmap.keys
                        self.emit(OpCode::CALL_NAT);
                        self.emit_u16(
                            BIGVM_NATIVES
                                .lock()
                                .unwrap()
                                .resolve_qualified("auto.hashmap.keys")
                                .unwrap(),
                        );
                        // Stack: [keys_list_id]

                        // Get keys list length
                        self.emit(OpCode::DUP);
                        self.emit(OpCode::ARRAY_LEN);
                        // Stack: [keys_list_id, length]

                        // Initialize counter to 0
                        self.emit(OpCode::CONST_0);
                        // Stack: [keys_list_id, length, 0]

                        let counter_index = self.add_var("_counter");
                        self.emit_store_loc(counter_index);
                        // Stack: [keys_list_id, length]

                        let len_index = self.add_var("_keys_len");
                        self.emit_store_loc(len_index);
                        // Stack: [keys_list_id]

                        let keys_ref_index = self.add_var("_keys_ref");
                        self.emit_store_loc(keys_ref_index);
                        // Stack: []

                        // Add key and value variables
                        let key_var_index = self.add_var(key_name);
                        let val_var_index = self.add_var(val_name);

                        // Loop start
                        let loop_start = self.code.len();

                        if let Some(pos) = self.loop_continue_positions.last_mut() {
                            *pos = loop_start as usize;
                        }

                        // Load counter and compare with length
                        self.emit_load_loc(counter_index);
                        self.emit_load_loc(len_index);
                        self.emit(OpCode::LT);

                        // JMP_IF_Z to end
                        self.emit(OpCode::JMP_IF_Z);
                        let jump_to_end = self.emit_placeholder_i16();

                        // Get key: keys_list[counter]
                        self.emit_load_loc(keys_ref_index);
                        self.emit_load_loc(counter_index);
                        self.emit(OpCode::GET_ELEM);
                        // Stack: [key_value]

                        // Store key to key variable
                        self.emit_store_loc(key_var_index);

                        // Get value: map.get(key)
                        // Push map ref, then push key, then call auto.hashmap.get_str (ID 122)
                        self.emit_load_loc(map_ref_index);
                        self.emit_load_loc(key_var_index);
                        // CALL_SPEC: dispatch "get" on HashMap type
                        // Simpler: use CALL_NAT with auto.hashmap.get
                        // The shim expects: push map_id, push key_str_idx
                        self.emit(OpCode::CALL_NAT);
                        self.emit_u16(
                            BIGVM_NATIVES
                                .lock()
                                .unwrap()
                                .resolve_qualified("auto.hashmap.get")
                                .unwrap(),
                        );
                        // Stack: [value (as Option-encoded)]

                        // Store value to val variable
                        self.emit_store_loc(val_var_index);

                        // Compile loop body — discard body's return value
                        let old_pop = self.should_pop_expr_result;
                        self.should_pop_expr_result = true;
                        self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                        self.should_pop_expr_result = old_pop;

                        // Continue: increment counter
                        let continue_pos = self.code.len();
                        if let Some(pos) = self.loop_continue_positions.last_mut() {
                            *pos = continue_pos;
                        }
                        self.emit_load_loc(counter_index);
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(1);
                        self.emit(OpCode::ADD);
                        self.emit_store_loc(counter_index);

                        // JMP back
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len();
                        self.emit_i16((loop_start as isize - current_pos as isize - 2) as i16);

                        // Patch exit jump
                        self.patch_jump(jump_to_end);

                        self.pop_scope();

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
                        self.loop_continue_positions.pop();
                        // Plan 447 H2 fix: Patch continue statements to increment position
                        let continues = self.loop_continues.pop().unwrap();
                        for continue_placeholder in continues {
                            self.patch_jump_to(continue_placeholder, continue_pos);
                        }
                    }
                    Iter::Cond => {
                        // Conditional for loop: for condition { ... } (like while)
                        // Plan 348 C3: push_scope/pop_scope around the body, matching
                        // every other for-loop variant (Iter::Named, Iter::Indexed, ...).
                        // Without this, a `break` inside the body could leave the scope
                        // stack inconsistent with the emitted bytecode.
                        self.push_scope();

                        // Loop start label
                        let loop_start = self.code.len();

                        // Set continue position: loop_start (re-evaluate condition)
                        if let Some(pos) = self.loop_continue_positions.last_mut() {
                            *pos = loop_start;
                        }

                        // Compile condition
                        self.compile_expr(&for_stmt.range)?;

                        // JMP_IF_Z to end (exit loop if condition false)
                        self.emit(OpCode::JMP_IF_Z);
                        let jump_to_end = self.emit_placeholder_i16();

                        // Compile loop body — force should_pop_expr_result = true
                        // so the Block handler keeps it for the last statement
                        // (loop body is a statement, its value is discarded).
                        let old_pop = self.should_pop_expr_result;
                        self.should_pop_expr_result = true;
                        self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                        self.should_pop_expr_result = old_pop;

                        // JMP back to loop start
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len();
                        if current_pos > 32765 {
                            // Use JMP_FAR (i32 offset) for large code
                            self.code.pop(); // remove JMP opcode
                            self.emit(OpCode::JMP_FAR);
                            self.emit_i32(loop_start as i32 - current_pos as i32 - 4);
                        } else {
                            self.emit_i16(loop_start as i16 - current_pos as i16 - 2);
                        }

                        // This is the loop exit point - patch all break jumps here
                        let _loop_exit = self.code.len();

                        // Patch exit jump (for loop condition)
                        self.patch_jump(jump_to_end);

                        // Pop loop scope (matches Iter::Named: before patching breaks)
                        self.pop_scope();

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
                        self.loop_continue_positions.pop();
                        // Plan 447 H2 fix: Patch continue statements to loop start (re-evaluate condition)
                        let continues = self.loop_continues.pop().unwrap();
                        for continue_placeholder in continues {
                            self.patch_jump_to(continue_placeholder, loop_start);
                        }
                    }
                    Iter::Ever => {
                        // Infinite loop: for ever { ... }
                        let loop_start = self.code.len();

                        // Set continue position: loop_start
                        if let Some(pos) = self.loop_continue_positions.last_mut() {
                            *pos = loop_start;
                        }

                        // Compile loop body — discard body's return value
                        let old_pop = self.should_pop_expr_result;
                        self.should_pop_expr_result = true;
                        self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                        self.should_pop_expr_result = old_pop;

                        // JMP back to loop start
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len();
                        self.emit_i16((loop_start as isize - current_pos as isize - 2) as i16);

                        // This is the loop exit point - patch all break jumps here
                        let _loop_exit = self.code.len();

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
                        self.loop_continue_positions.pop();
                        // Plan 447 H2 fix: Patch continue statements to loop start
                        let continues = self.loop_continues.pop().unwrap();
                        for continue_placeholder in continues {
                            self.patch_jump_to(continue_placeholder, loop_start);
                        }
                    }
                    Iter::Call(call) => {
                        // Plan 073: Iterator-based for loop: for x in list.iter() { ... }
                        // Compile the iterator call to get the iterator object
                        self.compile_expr(&Expr::Call(call.clone()))?;

                        // Store iterator in a local variable
                        self.push_scope(); // New scope for loop variable and iterator
                        let iter_index = self.add_var("_iterator");
                        self.emit_store_loc(iter_index);

                        // Loop start label
                        let loop_start = self.code.len();

                        // Call iter.next() to get next element
                        self.emit_load_loc(iter_index); // Load iterator

                        // Call next() method - this is a method call on the iterator
                        // For AutoVM, we need to call this as a native function
                        // iterator.next() should be compiled as a method call
                        // For now, we'll emit a CALL_NAT instruction for "Iterator.next"

                        // Get the variable name from the call
                        // The call should be like: list.iter() or iterator.next()
                        // For for x in list.iter(), the variable name should be extracted from context
                        // Since we don't have easy access to the variable name here, we'll use a placeholder
                        // The user should have: for x in list.iter()

                        // Emit CALL_NAT for Iterator.next
                        self.emit(OpCode::CALL_NAT);
                        // TODO: Get the native function ID for Iterator.next
                        // For now, use a placeholder ID
                        self.code.extend_from_slice(&0u32.to_le_bytes()); // Placeholder for native function ID

                        // Check if result is nil (end of iteration)
                        // Nil is represented as -1 in our VM
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(-1);
                        self.emit(OpCode::EQ);
                        self.emit(OpCode::JMP_IF_Z);
                        let jump_to_end = self.emit_placeholder_i16();

                        // Store the element to the loop variable
                        // Get variable name from context - for now, use "x" as default
                        let var_str = "x"; // TODO: Extract actual variable name from AST
                        let var_index = self.add_var(var_str);
                        self.emit_store_loc(var_index);

                        // Continue target: loop_start (re-check iterator.next())
                        if let Some(pos) = self.loop_continue_positions.last_mut() {
                            *pos = loop_start as usize;
                        }

                        // Compile loop body — discard body's return value
                        let old_pop = self.should_pop_expr_result;
                        self.should_pop_expr_result = true;
                        self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;
                        self.should_pop_expr_result = old_pop;

                        // JMP back to loop start
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len();
                        self.emit_i16((loop_start as isize - current_pos as isize - 2) as i16);

                        // This is the loop exit point - patch all break jumps here
                        let _loop_exit = self.code.len();

                        // Patch exit jump (for loop condition)
                        self.patch_jump(jump_to_end);

                        // Pop loop scope
                        self.pop_scope();

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
                        self.loop_continue_positions.pop();
                        // Plan 447 H2 fix: Patch continue statements to loop start
                        let continues = self.loop_continues.pop().unwrap();
                        for continue_placeholder in continues {
                            self.patch_jump_to(continue_placeholder, loop_start as usize);
                        }
                    }
                }
            }
            Stmt::Break => {
                // Plan 073: Break statement support
                // Check if we're inside a loop
                if self.loop_exits.last().is_some() {
                    // Emit JMP instruction
                    self.emit(OpCode::JMP);
                    // Add placeholder to current loop's exit list
                    let exit_placeholder = self.emit_placeholder_i16();
                    self.loop_exits.last_mut().unwrap().push(exit_placeholder);
                } else {
                    return Err(AutoError::Msg(
                        "Break statement outside of loop".to_string(),
                    ));
                }
            }
            Stmt::Continue => {
                // Continue statement: emit a JMP placeholder, to be patched
                // to the loop's continue target (increment/next-iteration) after
                // the body is compiled. This mirrors how break uses loop_exits.
                // The old code read loop_continue_positions directly, but that
                // value is 0 (placeholder) during body compilation → jump to
                // offset 0 → infinite loop.
                if self.loop_continues.last().map_or(false, |v| !v.is_empty())
                    || self.loop_continues.last().is_some()
                {
                    // New placeholder-based path (range-for and any variant
                    // that pushed loop_continues).
                    self.emit(OpCode::JMP);
                    let continue_placeholder = self.emit_placeholder_i16();
                    self.loop_continues
                        .last_mut()
                        .unwrap()
                        .push(continue_placeholder);
                } else if let Some(&continue_pos) = self.loop_continue_positions.last() {
                    if continue_pos > 0 {
                        // Legacy path for variants that set loop_continue_positions
                        // before body compilation (rare; most set it after → bug).
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len();
                        let offset = (continue_pos as isize - current_pos as isize - 2) as i16;
                        self.emit_i16(offset);
                    } else {
                        return Err(AutoError::Msg(
                            "Continue statement in unsupported loop variant".to_string(),
                        ));
                    }
                } else {
                    return Err(AutoError::Msg(
                        "Continue statement outside of loop".to_string(),
                    ));
                }
            }
            // Plan 073: Is pattern matching statement
            Stmt::Is(is_stmt) => {
                // Evaluate target expression once and store in temp variable
                self.compile_expr(&is_stmt.target)?;
                let target_var = self.add_var("_is_target");
                self.emit_store_loc(target_var);

                let mut end_jumps = Vec::new();

                // Process each branch
                for branch in &is_stmt.branches {
                    match branch {
                        crate::ast::IsBranch::EqBranch(patterns, body) => {
                            // Plan 120: Check for Option/Result pattern matching
                            // Use first pattern for special pattern types (Option/Result/Cover)
                            let pattern = &patterns[0];
                            match pattern {
                                crate::ast::Expr::StructPattern(sc) if sc.variant.is_none() => {
                                    // Plan 525 W2: is-struct 解构臂——plain struct
                                    // 静态恒命中(a2r 规范=match 解构绑定;此前
                                    // StructPattern 直落 10117 panic 洞顺修)。
                                    // 发射镜像 aavm cg_is struct-cover 臂:逐绑定
                                    // load target → GET_FIELD(名字池)→ store
                                    // 绑定 → 体 → JMP end。
                                    for fb in &sc.fields {
                                        if fb.binding.as_str() != "_" {
                                            self.emit_load_loc(target_var);
                                            let field_bytes = fb.field.as_bytes().to_vec();
                                            let field_idx = self.strings.len() as u32;
                                            self.strings.push(field_bytes);
                                            self.emit(OpCode::GET_FIELD);
                                            self.code.extend_from_slice(&field_idx.to_le_bytes());
                                            let var_idx = self.add_var(fb.binding.as_str());
                                            self.emit_store_loc(var_idx);
                                        }
                                    }
                                    self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;
                                    self.emit(OpCode::JMP);
                                    let jump_to_end = self.emit_placeholder_i16();
                                    end_jumps.push(jump_to_end);
                                    continue;
                                }
                                crate::ast::Expr::None => {
                                    self.emit_load_loc(target_var);
                                    self.emit(OpCode::IS_VARIANT);
                                    let variant_name = "Option.None";
                                    let name_bytes = variant_name.as_bytes();
                                    self.emit_u16(name_bytes.len() as u16);
                                    for &byte in name_bytes {
                                        self.code.push(byte);
                                    }
                                }
                                crate::ast::Expr::OptionPattern(opt_cover) => {
                                    match opt_cover.variant {
                                        crate::ast::OptionVariant::Some => {
                                            self.emit_load_loc(target_var);
                                            self.emit(OpCode::IS_VARIANT);
                                            let variant_name = "Option.Some";
                                            let name_bytes = variant_name.as_bytes();
                                            self.emit_u16(name_bytes.len() as u16);
                                            for &byte in name_bytes {
                                                self.code.push(byte);
                                            }

                                            self.emit(OpCode::JMP_IF_Z);
                                            let jump_to_next = self.emit_placeholder_i16();

                                            if let Some(binding) = &opt_cover.binding {
                                                self.emit_load_loc(target_var);
                                                self.emit(OpCode::GET_GENERIC_FIELD);
                                                self.emit_u32(0);

                                                let var_idx = self.add_var(binding.as_str());
                                                self.emit_store_loc(var_idx);

                                                if binding.as_str() != "_" {
                                                    let inner_type = self
                                                        .infer_option_inner_type(&is_stmt.target);
                                                    self.var_types
                                                        .insert(binding.to_string(), inner_type);
                                                }
                                            }

                                            self.compile_stmt(&crate::ast::Stmt::Block(
                                                body.clone(),
                                            ))?;

                                            self.emit(OpCode::JMP);
                                            let jump_to_end = self.emit_placeholder_i16();
                                            end_jumps.push(jump_to_end);

                                            self.patch_jump(jump_to_next);
                                            continue;
                                        }
                                        crate::ast::OptionVariant::None => {
                                            self.emit_load_loc(target_var);
                                            self.emit(OpCode::IS_VARIANT);
                                            let variant_name = "Option.None";
                                            let name_bytes = variant_name.as_bytes();
                                            self.emit_u16(name_bytes.len() as u16);
                                            for &byte in name_bytes {
                                                self.code.push(byte);
                                            }
                                        }
                                    }
                                }
                                // Plan 120: ResultPattern - Ok(x) or Err(e) in is statement
                                // Plan 208: Use IS_VARIANT + GET_GENERIC_FIELD instead of IS_OK + UNWRAP
                                crate::ast::Expr::ResultPattern(res_cover) => {
                                    match res_cover.variant {
                                        crate::ast::ResultVariant::Ok => {
                                            self.emit_load_loc(target_var);
                                            self.emit(OpCode::IS_VARIANT);
                                            let variant_name = "Result.Ok";
                                            let name_bytes = variant_name.as_bytes();
                                            self.emit_u16(name_bytes.len() as u16);
                                            for &byte in name_bytes {
                                                self.code.push(byte);
                                            }

                                            // Jump to next branch if not matched
                                            self.emit(OpCode::JMP_IF_Z);
                                            let jump_to_next = self.emit_placeholder_i16();
                                            // If we have a binding, extract the value and store it
                                            if let Some(binding) = &res_cover.binding {
                                                self.emit_load_loc(target_var);
                                                self.emit(OpCode::GET_GENERIC_FIELD);
                                                self.emit_u32(0);
                                                let var_idx = self.add_var(binding.as_str());
                                                self.emit_store_loc(var_idx);
                                            }

                                            // Compile branch body
                                            self.compile_stmt(&crate::ast::Stmt::Block(
                                                body.clone(),
                                            ))?;

                                            // Jump to end of is statement
                                            self.emit(OpCode::JMP);
                                            let jump_to_end = self.emit_placeholder_i16();
                                            end_jumps.push(jump_to_end);

                                            // Patch jump to next branch
                                            self.patch_jump(jump_to_next);
                                            continue;
                                        }
                                        crate::ast::ResultVariant::Err => {
                                            self.emit_load_loc(target_var);
                                            self.emit(OpCode::IS_VARIANT);
                                            let variant_name = "Result.Err";
                                            let name_bytes = variant_name.as_bytes();
                                            self.emit_u16(name_bytes.len() as u16);
                                            for &byte in name_bytes {
                                                self.code.push(byte);
                                            }

                                            // Jump to next branch if not matched
                                            self.emit(OpCode::JMP_IF_Z);
                                            let jump_to_next = self.emit_placeholder_i16();
                                            // If we have a binding, extract the error value and store it
                                            if let Some(binding) = &res_cover.binding {
                                                self.emit_load_loc(target_var);
                                                self.emit(OpCode::GET_GENERIC_FIELD);
                                                self.emit_u32(0);
                                                let var_idx = self.add_var(binding.as_str());
                                                self.emit_store_loc(var_idx);
                                            }

                                            // Compile branch body
                                            self.compile_stmt(&crate::ast::Stmt::Block(
                                                body.clone(),
                                            ))?;

                                            // Jump to end of is statement
                                            self.emit(OpCode::JMP);
                                            let jump_to_end = self.emit_placeholder_i16();
                                            end_jumps.push(jump_to_end);

                                            // Patch jump to next branch
                                            self.patch_jump(jump_to_next);
                                            continue;
                                        }
                                    }
                                }
                                // Plan 197 Task 16: Legacy Some as expression pattern (backward compatibility)
                                // Uses IS_VARIANT("Option.Some") instead of IS_SOME
                                crate::ast::Expr::Some(inner) => {
                                    self.emit_load_loc(target_var);
                                    self.emit(OpCode::IS_VARIANT);
                                    let variant_name = "Option.Some";
                                    let name_bytes = variant_name.as_bytes();
                                    self.emit_u16(name_bytes.len() as u16);
                                    for &byte in name_bytes {
                                        self.code.push(byte);
                                    }

                                    let _ = inner; // Suppress unused warning
                                }
                                crate::ast::Expr::Ok(inner) => {
                                    self.emit_load_loc(target_var);
                                    self.emit(OpCode::IS_VARIANT);
                                    let variant_name = "Result.Ok";
                                    let name_bytes = variant_name.as_bytes();
                                    self.emit_u16(name_bytes.len() as u16);
                                    for &byte in name_bytes {
                                        self.code.push(byte);
                                    }

                                    let _ = inner; // Suppress unused warning
                                }
                                crate::ast::Expr::Err(msg) => {
                                    self.emit_load_loc(target_var);
                                    self.emit(OpCode::IS_VARIANT);
                                    let variant_name = "Result.Err";
                                    let name_bytes = variant_name.as_bytes();
                                    self.emit_u16(name_bytes.len() as u16);
                                    for &byte in name_bytes {
                                        self.code.push(byte);
                                    }

                                    let _ = msg; // Suppress unused warning
                                }
                                // Plan 197 Task 15: Enum variant destructuring (e.g., Atom.Int(n))
                                crate::ast::Expr::Cover(crate::ast::Cover::Tag(tag_cover)) => {
                                    let variant_mono =
                                        format!("{}.{}", tag_cover.kind, tag_cover.tag);
                                    // Only use IS_VARIANT path for data variants (registered in generic_registry)
                                    // Scalar enums (C-style, no payload) fall through to EQ comparison.
                                    // Plan 442 C2: variants UNKNOWN to the VM (use.rust-imported enums,
                                    // e.g. musk backend's `AdvanceResult.ExecuteStep(sid, role)`) also take
                                    // the IS_VARIANT path — the old fallback compiled the PATTERN as an
                                    // expression and hit the Cover-construction arm's hard "Unknown enum
                                    // variant" error. Runtime IS_VARIANT on a non-matching value simply
                                    // falls through to the next branch (graceful); field count falls back
                                    // to the binding arity when no template is registered.
                                    // Plan 447 H1 fix: For scalar enums with multiple patterns, let them
                                    // fall through to multi-pattern handling below instead of early continue.
                                    let known_scalar = self.enum_values.contains_key(&variant_mono);
                                    let has_data_payload =
                                        self.generic_registry.has_template(&variant_mono)
                                            || !known_scalar;

                                    if has_data_payload
                                        && tag_cover.bindings.iter().any(|b| b.as_str() != "_")
                                    {
                                        // Binding destructuring pattern: Atom.Int(n) -> ...
                                        self.emit_load_loc(target_var);
                                        self.emit(OpCode::IS_VARIANT);
                                        let name_bytes = variant_mono.as_bytes();
                                        self.emit_u16(name_bytes.len() as u16);
                                        for &byte in name_bytes {
                                            self.code.push(byte);
                                        }

                                        // Jump to next branch if variant doesn't match
                                        self.emit(OpCode::JMP_IF_Z);
                                        let jump_to_next = self.emit_placeholder_i16();

                                        // Variant matched — extract the payload fields and bind them

                                        // Determine field count and types from the generic registry
                                        let (field_count, field_types) = if let Some(template) =
                                            self.generic_registry.get_template(&variant_mono)
                                        {
                                            let types: Vec<crate::ast::Type> = template
                                                .fields
                                                .iter()
                                                .map(|f| f.field_type.clone())
                                                .collect();
                                            (template.fields.len(), types)
                                        } else {
                                            // Plan 442 C2: unregistered variant (use.rust enum) —
                                            // trust the pattern's binding arity as the payload width.
                                            (tag_cover.bindings.len().max(1), vec![])
                                        };

                                        // Extract each field and bind to variables
                                        let binding_count =
                                            tag_cover.bindings.len().min(field_count);
                                        for i in 0..binding_count {
                                            let binding = &tag_cover.bindings[i];
                                            if binding.as_str() != "_" {
                                                self.emit_load_loc(target_var);
                                                self.emit(OpCode::GET_GENERIC_FIELD);
                                                self.emit_u32(i as u32);
                                                if let Some(ref ty) = field_types.get(i) {
                                                    self.var_types
                                                        .insert(binding.to_string(), (*ty).clone());
                                                }
                                                let var_idx = self.add_var(binding.as_str());
                                                self.emit_store_loc(var_idx);
                                            }
                                        }

                                        // Compile branch body with bindings in scope
                                        self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                                        // Jump to end of is statement
                                        self.emit(OpCode::JMP);
                                        let jump_to_end = self.emit_placeholder_i16();
                                        end_jumps.push(jump_to_end);

                                        // Patch jump to next branch
                                        self.patch_jump(jump_to_next);
                                        continue; // Skip the default handling
                                    } else if has_data_payload {
                                        self.emit_load_loc(target_var);
                                        self.emit(OpCode::IS_VARIANT);
                                        let name_bytes = variant_mono.as_bytes();
                                        self.emit_u16(name_bytes.len() as u16);
                                        for &byte in name_bytes {
                                            self.code.push(byte);
                                        }
                                    } else if patterns.len() == 1 {
                                        // Scalar enum, single pattern: one EQ,
                                        // the shared JMP_IF_Z tail below handles it.
                                        self.emit_load_loc(target_var);
                                        self.compile_expr(pattern)?;
                                        self.emit(OpCode::EQ);
                                    } else {
                                        // Plan 447 H1: scalar enum multi-pattern arm
                                        // (`A | B -> ...`) — per-pattern EQ with
                                        // short-circuit OR, same emission as the
                                        // literal multi-pattern path (formerly this
                                        // case emitted nothing and the arm never matched).
                                        self.emit_is_or_arm(
                                            target_var,
                                            patterns,
                                            body,
                                            &mut end_jumps,
                                        )?;
                                        continue;
                                    }
                                }
                                _ => {
                                    // Plan 325: 通配符 `_` —— 总匹配（catch-all），不比较直接执行 body。
                                    // `_` 被词法化为 Ident("_")，无专门通配 token；裸 `_` arm（不包裹在
                                    // Some/Ok 里）此前被当变量引用 → Undefined variable: _。
                                    // 多 pattern 里混 `_` 罕见且语义含糊，只处理单 pattern 的纯 `_`。
                                    let is_wildcard = patterns.len() == 1
                                        && matches!(&patterns[0], crate::ast::Expr::Ident(n) if n.as_str() == "_");
                                    if is_wildcard {
                                        // 总匹配：直接编译 body，跳到 is 末尾（仿 ElseBranch）
                                        self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;
                                        self.emit(OpCode::JMP);
                                        let jump_to_end = self.emit_placeholder_i16();
                                        end_jumps.push(jump_to_end);
                                        continue; // 跳过下方 EQ 比较 + 默认 body 编译
                                    }
                                    // Standard equality comparison for patterns
                                    if patterns.len() == 1 {
                                        self.emit_load_loc(target_var);
                                        self.compile_expr(pattern)?;
                                        self.emit(OpCode::EQ);
                                    } else {
                                        // Multi-pattern: per-pattern EQ + short-circuit OR
                                        // (Plan 447 H1: via the shared emit_is_or_arm helper)
                                        self.emit_is_or_arm(
                                            target_var,
                                            patterns,
                                            body,
                                            &mut end_jumps,
                                        )?;
                                        continue; // Skip the default handling below
                                    }
                                }
                            }

                            // Jump to next branch if not matched
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_next = self.emit_placeholder_i16();

                            // Compile branch body
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            // Jump to end of is statement
                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            end_jumps.push(jump_to_end);

                            // Patch jump to next branch
                            self.patch_jump(jump_to_next);
                        }
                        crate::ast::IsBranch::IfBranch(condition, body) => {
                            // Plan 073: Evaluate condition expression
                            self.compile_expr(condition)?;

                            // Jump to next branch if condition is false (zero)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_next = self.emit_placeholder_i16();

                            // Compile branch body
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            // Jump to end of is statement
                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            end_jumps.push(jump_to_end);

                            // Patch jump to next branch
                            self.patch_jump(jump_to_next);
                        }
                        crate::ast::IsBranch::ElseBranch(body) => {
                            // This is the default case - just compile body
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            // Jump to end (in case there are more branches after else)
                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            end_jumps.push(jump_to_end);
                        }
                    }
                }

                // Patch all jump_to_end placeholders
                for jump_to_end in end_jumps {
                    self.patch_jump(jump_to_end);
                }
            }
            // Plan 121/127: Task/Msg support - compile task definition with handlers
            Stmt::TaskDef(task_def) => {
                // Register task type in the local types registry for lookup
                let task_name = task_def.name.to_string();

                // Store task metadata in local types HashMap
                self.types.insert(
                    task_name.clone(),
                    TypeInfo {
                        _name: if task_def.is_single() {
                            format!("{}#single", task_name)
                        } else {
                            task_name.clone()
                        },
                        member_names: Vec::new(),
                    },
                );

                // Plan 127: Create handler table for this task type
                let mut handler_table =
                    crate::vm::task_handler::TaskHandlerTable::new(task_name.clone());

                // Plan 390 §14 L3: collect variant definitions (WithBindings +
                // Simple) so send call-sites can construct Value::Obj messages.
                let mut variants: Vec<(String, Vec<String>)> = Vec::new();
                for (pattern, _guard, _body) in &task_def.on_block.handlers {
                    match pattern {
                        crate::ast::TaskMsgPattern::WithBindings { variant, bindings } => {
                            let bind_names: Vec<String> =
                                bindings.iter().map(|(n, _)| n.to_string()).collect();
                            variants.push((variant.to_string(), bind_names));
                        }
                        crate::ast::TaskMsgPattern::Simple(name) => {
                            variants.push((name.to_string(), Vec::new()));
                        }
                        _ => {}
                    }
                }
                if !variants.is_empty() {
                    self.task_variants.insert(task_name.clone(), variants);
                }
                // Plan 317: Register state fields so bare identifier references
                // inside hooks/handlers resolve to LOAD/STORE_STATE_FIELD. Each
                // field gets a stable idx (0, 1, 2, ...).
                self.current_task_state_fields.clear();
                for (i, (field_name, _mutable, _init)) in task_def.state.iter().enumerate() {
                    self.current_task_state_fields
                        .insert(field_name.to_string(), i as u8);
                }

                // Compile lifecycle hooks if present
                // Start hook
                let has_handlers = !task_def.on_block.handlers.is_empty();
                // VM bug fix: emit a #start hook (state-field initializers + TASK_LOOP
                // + #start export) whenever the task has an explicit `fn start()` OR
                // any state fields. Previously this whole block was gated on
                // `start_hook.is_some()`, so a task with state fields but no
                // `fn start()` never emitted the STORE_STATE_FIELD default initializers
                // AND never registered `#start` — shim_task_spawn_vm then fell back to
                // ip=0 and the declared defaults (e.g. `count = 5`) were silently lost.
                let has_state = !task_def.state.is_empty();
                // Plan 317 §11 Phase 8 (P4'): ALSO emit `#start` when the task has
                // `on` handlers but no `fn start()` and no state fields. Without a
                // `#start` export, shim_task_spawn_vm falls back to start_offset=0,
                // the spawned task runs from the wrong ip, and message payload
                // bindings (`on { n int -> }`) read 0 instead of the sent value.
                // Every actor that can receive messages needs a #start that parks
                // it in the message loop via TASK_LOOP.
                let needs_start = task_def.start_hook.is_some() || has_state || has_handlers;
                if needs_start {
                    let start_offset = self.code.len() as u32;
                    // Plan 317: emit state field initializers at the very start
                    // of the start hook. spawn_task points the actor's ip here,
                    // so these run first and initialize state_vars before any
                    // handler can read them.
                    for (field_name, _mutable, init_expr) in &task_def.state {
                        self.compile_expr(init_expr)?;
                        if let Some(&idx) = self.current_task_state_fields.get(field_name.as_str())
                        {
                            self.emit(OpCode::STORE_STATE_FIELD);
                            self.code.push(idx);
                        }
                    }
                    // Compile the hook body (only if an explicit fn start() exists).
                    if let Some(ref start_hook) = task_def.start_hook {
                        self.push_scope();
                        for stmt in &start_hook.body.stmts {
                            self.compile_stmt(stmt)?;
                        }
                        self.pop_scope();
                    }
                    // Plan 317 Phase 1: if the task has message handlers, the
                    // start hook does NOT simply RET — instead it parks the task
                    // in the message loop (TASK_LOOP sets in_message_loop + Waiting)
                    // so it can receive subsequent TaskHandle.send messages.
                    // Without this, the actor task terminates after start and
                    // never processes messages.
                    if has_handlers {
                        let task_name_bytes = task_name.as_bytes().to_vec();
                        let task_name_idx = self.strings.len() as u32;
                        self.strings.push(task_name_bytes);
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(task_name_idx as i32);
                        self.emit(OpCode::TASK_LOOP);
                    }
                    self.emit(OpCode::RET);
                    // Store start hook offset (will be registered with TaskRegistry)
                    self.exports
                        .insert(format!("{}#start", task_name), start_offset);
                }

                // Stop hook
                if let Some(ref stop_hook) = task_def.stop_hook {
                    let stop_offset = self.code.len() as u32;
                    // Compile the hook body
                    self.push_scope();
                    for stmt in &stop_hook.body.stmts {
                        self.compile_stmt(stmt)?;
                    }
                    self.pop_scope();
                    self.emit(OpCode::RET);
                    // Store stop hook offset
                    self.exports
                        .insert(format!("{}#stop", task_name), stop_offset);
                }

                // Plan 127: Compile each message handler in the on block
                let on_block = &task_def.on_block;
                let has_context = on_block.context_param.is_some();

                for (pattern, _guard, body) in &on_block.handlers {
                    // Record handler bytecode offset
                    let handler_offset = self.code.len() as u32;

                    // Add handler entry to table (pattern will be serialized)
                    #[allow(unused_variables)]
                    let pattern_idx =
                        handler_table.add_handler(pattern, handler_offset, has_context);

                    // Plan 390 §G2.1: bind the message payload to the handler's pattern
                    // variable. The runtime pushes the message value onto the value
                    // stack at handler entry (engine.rs run_task_loop), but previously
                    // nothing stored it into a named local — so a TypeBinding pattern
                    // like `n int` left `n` unbound ("Undefined variable: n"). Here we
                    // open a scope, claim a local slot for the binding name, and emit
                    // STORE_LOC to pop the on-stack message into it before the body.
                    self.push_scope();
                    if let Some(bind_name) = pattern.binding_name() {
                        let slot = self.add_var(bind_name.as_str());
                        match slot {
                            0 => self.emit(OpCode::STORE_LOC_0),
                            1 => self.emit(OpCode::STORE_LOC_1),
                            _ => {
                                self.emit(OpCode::STORE_LOCAL);
                                self.code.push(slot as u8);
                            }
                        }
                    } else if let crate::ast::TaskMsgPattern::WithBindings { bindings, .. } =
                        pattern
                    {
                        // Plan 390 §14 L3: multi-field variant message. The wake
                        // path pushed a VmRef to the Obj {__variant, <fields>}.
                        // For each binding, DUP the VmRef, GET_FIELD(<name>),
                        // STORE_LOC into the binding's slot. The VmRef is
                        // consumed by a final POP.
                        let emit_store = |codegen: &mut Self, slot: usize| match slot {
                            0 => codegen.emit(OpCode::STORE_LOC_0),
                            1 => codegen.emit(OpCode::STORE_LOC_1),
                            _ => {
                                codegen.emit(OpCode::STORE_LOCAL);
                                codegen.code.push(slot as u8);
                            }
                        };
                        for (bind_name, _) in bindings {
                            self.emit(OpCode::DUP);
                            let field_str_idx = {
                                let idx = self.strings.len();
                                self.strings
                                    .push(bind_name.as_str().to_string().into_bytes());
                                idx as u32
                            };
                            self.emit(OpCode::GET_FIELD);
                            self.code.extend_from_slice(&field_str_idx.to_le_bytes());
                            let slot = self.add_var(bind_name.as_str());
                            emit_store(self, slot);
                        }
                        // Consume the remaining VmRef on the stack.
                        self.emit(OpCode::POP);
                    }

                    // Compile handler body
                    // The handler receives message value on stack
                    // If has_context, also receives context id
                    for stmt in &body.stmts {
                        self.compile_stmt(stmt)?;
                    }
                    self.pop_scope();

                    // Handler must return - emit RET.
                    // Plan 390 §15 G2-refactor (方案 B): handler 现运行在真 bp 帧内
                    // （唤醒路径建帧），bp≠0，RET 会执行 new_sp = bp - n_args。handler
                    // 无栈参数，故 n_args = 0 —— 必须显式 emit 0 字节（裸 RET 会读到
                    // 下一条字节码当 n_args）。对照普通函数 RET（codegen.rs:1422-1423）。
                    self.emit(OpCode::RET);
                    self.code.push(0); // n_args = 0

                    if crate::is_vm_debug() {
                        eprintln!(
                            "[TaskDef] Compiled handler {} for task {} at offset {}",
                            pattern_idx, task_name, handler_offset
                        );
                    }
                }

                // Compile else handler if present
                if let Some(ref else_body) = on_block.else_handler {
                    let else_offset = self.code.len() as u32;
                    // Add else as a special handler with pattern_idx = 0xFFFFFFFF
                    // (handled specially at runtime)
                    for stmt in &else_body.stmts {
                        self.compile_stmt(stmt)?;
                    }
                    // Plan 390 §15 G2-refactor (方案 B): else handler 同样运行在真 bp 帧内，
                    // RET 需跟 n_args=0 字节（见上方 handler RET 注释）。
                    self.emit(OpCode::RET);
                    self.code.push(0); // n_args = 0
                    self.exports
                        .insert(format!("{}#else", task_name), else_offset);
                }

                // Register handler table
                self.task_handler_registry.register(handler_table);

                // Emit TASK_LOOP opcode if task has handlers
                if !on_block.handlers.is_empty() {
                    // Push task type name string index
                    let task_name_bytes = task_name.as_bytes().to_vec();
                    let task_name_idx = self.strings.len() as u32;
                    self.strings.push(task_name_bytes);

                    self.emit(OpCode::CONST_I32);
                    self.emit_i32(task_name_idx as i32);
                    self.emit(OpCode::TASK_LOOP);
                }

                // Note: Lifecycle hooks and message handlers are now compiled to bytecode.
                // The FFI shim_task_spawn function creates a TaskInstance and
                // registers it in the TaskRegistry.

                // Plan 317: clear state fields now that this TaskDef is compiled.
                self.current_task_state_fields.clear();
            }
            Stmt::Node(node) => {
                // Plan 364 Step 5: In config_mode, a block-level node statement
                // (e.g. `dep("x") {}` / `app p1 {}`) accumulates into the
                // enclosing container as a kid — same path as Stmt::Expr(Node).
                if self.config_mode {
                    return self.compile_config_stmt_expr(&Expr::Node(node.clone()));
                }
                // Stmt::Node wraps an Expr::Node — compile the expression
                self.compile_expr(&Expr::Node(node.clone()))?;
                // Plan 348 C4: see Stmt::Expr — never POP when a native void shim
                // left nothing on the stack.
                let needs_pop = self.should_pop_expr_result && !self.last_was_native_void;
                if needs_pop {
                    self.emit(OpCode::POP);
                }
            }
            // Plan 203 Phase 2: Handle use statements for import scope resolution
            Stmt::Use(use_stmt) => {
                self.handle_use_stmt(use_stmt);
            }
            // Plan 212 Phase 2.4: Macro invocation — #debug("msg"), #info("msg")
            Stmt::MacroCall(macro_call) => {
                let routed_name = match macro_call.name.as_str() {
                    "debug" => "Log.debug",
                    "info" => "Log.info",
                    "warn" => "Log.warn",
                    "error" => "Log.error",
                    _ => {
                        // Unknown macro — silently ignore in VM
                        return Ok(());
                    }
                };
                // Compile arguments
                for arg in &macro_call.args {
                    self.compile_expr(arg)?;
                }
                // Look up native ID and emit CALL_NAT
                if let Some(&id) = self.intrinsics.get(routed_name) {
                    self.emit(OpCode::CALL_NAT);
                    self.emit_u16(id);
                } else {
                    // Fallback: try BIGVM_NATIVES
                    let mut reg = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
                    if let Some(native_id) = reg.resolve_qualified(routed_name) {
                        drop(reg);
                        self.emit(OpCode::CALL_NAT);
                        self.emit_u16(native_id);
                    }
                }
                self.last_expr_type = ObjectType::Void;
                self.last_was_native_void = true;
            }
            crate::ast::Stmt::Dep(dep) => {
                // Register dep name as module prefix in rust_native_map
                // so that e.g. `env_logger.init()` is resolved instead of
                // erroring with "Undefined variable: env_logger"
                let crate_name = dep.name.to_string();
                self.declared_dep_crates.insert(crate_name.clone());
                if !self.rust_native_map.contains_key(&crate_name) {
                    self.rust_native_map
                        .insert(crate_name.clone(), (crate_name.clone(), crate_name.clone()));
                }
            }
            _ => {
                // TODO: Implement other statements
            }
        }
        Ok(())
    }

    /// Plan 364 Step 5: Compile a block-level expression statement in Config
    /// mode, accumulating its result into the enclosing node container rather
    /// than leaving it on the stack.
    ///
    /// Dispatch:
    /// - `Expr::Pair { key, value }` → compile `value`, emit key string idx,
    ///   `ACCUM_PAIR` (sets a prop on the container).
    /// - `Expr::Node(_)` → compile the node (config-mode `Expr::Node` path
    ///   leaves a Node id on the stack), emit `ACCUM_NODE` (adds as `_exprN`
    ///   sub-node).
    /// - `Expr::Object(_)` → compile the object literal (`CREATE_OBJ`), emit
    ///   `ACCUM_MERGE` (folds fields into the container).
    /// - other (bare function call, ident ref, etc.) → compile and `POP`
    ///   (non-accumulating; its value is discarded).
    fn compile_config_stmt_expr(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            Expr::Pair(pair) => {
                let key_str = match &pair.key {
                    crate::ast::Key::NamedKey(name) => name.to_string(),
                    crate::ast::Key::StrKey(s) => s.to_string(),
                    crate::ast::Key::IntKey(i) => i.to_string(),
                    crate::ast::Key::BoolKey(b) => b.to_string(),
                };
                // Plan 364 Step 5: bind the value to a same-named local so later
                // statements can reference it — `kernel: {...}` makes
                // `kernel.heap` / `${kernel.heap}` work, `modules: [...]` makes
                // `for d in modules` work. DUP keeps one copy for the
                // accumulator and stores the other as a local.
                //
                // Plan 375: a top-level Pair is a *default* parameter value. If
                // an arg of the same name was injected via
                // `compile_config_program` (a dep override), the injected value
                // wins — discard this default literal and load the injected
                // local instead, so BOTH the local binding and the accumulated
                // root prop reflect the override. This is what lets SCU001's
                // `dep osal { kernel: kernel_config }` override osal's built-in
                // `kernel: {...}` default.
                // Plan 375: only a *top-level* Pair (depth 0, directly in the
                // root container) is a parameter default subject to override.
                // Pairs inside nested nodes (dir/app/dep bodies, depth ≥1) are
                // structural field definitions and must keep their literal
                // value — otherwise an injected `at` would rewrite every nested
                // dir's `at:`, producing doubled paths like Bsp/Mcal/Bsp/Mcal.
                let is_top_level_default =
                    self.config_accum_depth == 0 && self.config_injected_keys.contains(&key_str);
                if is_top_level_default {
                    // Reuse the slot allocated at injection time; do NOT call
                    // add_var (it would mint a fresh, never-written slot and
                    // shadow the injected value). Push the injected override
                    // value for ACCUM_PAIR (the default literal is not compiled
                    // in this branch, so nothing to discard).
                    if let Some(slot) = self.lookup_var(&key_str) {
                        self.emit_load_loc(slot);
                    } else {
                        // Fallback: treat as a normal default binding.
                        self.compile_expr(&pair.value)?;
                        let s = self.add_var(&key_str);
                        self.emit(OpCode::DUP);
                        self.emit_store_loc(s);
                    }
                } else if self.config_accum_depth == 0 {
                    // Top-level non-injected Pair: a parameter default. Bind it
                    // to a same-named local so later statements / f-strings can
                    // reference it (`kernel: {...}` → `${kernel.port}`).
                    self.compile_expr(&pair.value)?;
                    let s = self.add_var(&key_str);
                    self.emit(OpCode::DUP);
                    self.emit_store_loc(s);
                } else {
                    // Plan 375: nested Pair (depth ≥1) is a structural field
                    // def inside a node body. It must NOT bind a local —
                    // config_mode has no scope isolation, so a STORE_LOCAL here
                    // would leak into / shadow the enclosing scope (e.g. osal's
                    // `dep("kernel") { config: `${kernel.config}` }` would
                    // overwrite the top-level `config` local, corrupting a
                    // later `lib("osal") { incs: [`${config}`] }`). The value
                    // is only accumulated into the node container below.
                    self.compile_expr(&pair.value)?;
                }
                let key_idx = self.add_string(&key_str);
                self.emit(OpCode::ACCUM_PAIR);
                self.code.extend_from_slice(&key_idx.to_le_bytes());
                Ok(())
            }
            Expr::Node(_) => {
                self.compile_expr(expr)?;
                self.emit(OpCode::ACCUM_NODE);
                Ok(())
            }
            Expr::Object(_) => {
                self.compile_expr(expr)?;
                self.emit(OpCode::ACCUM_MERGE);
                Ok(())
            }
            _ => {
                self.compile_expr(expr)?;
                self.emit(OpCode::POP);
                Ok(())
            }
        }
    }

    /// Plan 364 Step 5: Compile a node expression in Config mode.
    ///
    /// Emits `PUSH_ACCUM(name, id)` then compiles the body statements (each
    /// Pair → ACCUM_PAIR into the container, each sub-node → recursive
    /// compile_config_node + ACCUM_NODE) and finally `POP_ACCUM`, leaving the
    /// assembled Node's id on the stack.
    ///
    /// Positional/named args (e.g. `dep("x")`) are folded into the container
    /// as `_argN` props so they survive in the materialized node; bare-name
    /// ids (e.g. `port lanshan`) are passed via PUSH_ACCUM's id operand.
    pub(super) fn compile_config_node(&mut self, node: &crate::ast::Node) -> AutoResult<()> {
        // PUSH_ACCUM(name_str_idx, id_str_idx). The id carries the node's
        // identity: a bare-name id (`port lanshan`) when present, else the
        // first positional arg if it is a string literal (`app("main")` → id
        // "main", which `main_arg()` returns via the `!id.is_empty()` branch),
        // else none (0xFFFF sentinel — main_arg falls back to args/name prop).
        let name_idx = self.add_string(&node.name.to_string());
        let id_str: String = if !node.id.is_empty() {
            node.id.to_string()
        } else {
            // First positional arg as string literal → use as id.
            node.args
                .args
                .first()
                .and_then(|a| match a {
                    crate::ast::Arg::Pos(crate::ast::Expr::Str(s)) => Some(s.to_string()),
                    crate::ast::Arg::Name(n) => Some(n.to_string()),
                    _ => None,
                })
                .unwrap_or_default()
        };
        let id_idx = if !id_str.is_empty() {
            self.add_string(&id_str)
        } else {
            0xFFFF // sentinel: no id
        };
        self.emit(OpCode::PUSH_ACCUM);
        self.code.extend_from_slice(&name_idx.to_le_bytes());
        self.code.extend_from_slice(&id_idx.to_le_bytes());

        // Fold positional/named args into the container as `_argN` props so
        // non-id args (e.g. `dep("x", "1.0")` → _arg1="1.0") survive in the
        // materialized node. (When the first positional string arg became the
        // id above, it is still emitted here as _arg0 for completeness.)
        let mut arg_n = 0u32;
        for arg in &node.args.args {
            let (key_opt, value_expr) = match arg {
                crate::ast::Arg::Pos(expr) => (Some(format!("_arg{}", arg_n)), expr.clone()),
                crate::ast::Arg::Pair(key, expr) => (Some(key.to_string()), expr.clone()),
                crate::ast::Arg::Name(name) => (
                    Some(format!("_arg{}", arg_n)),
                    crate::ast::Expr::Str(name.clone()),
                ),
            };
            if let Some(key) = key_opt {
                self.compile_expr(&value_expr)?;
                let key_idx = self.add_string(&key);
                self.emit(OpCode::ACCUM_PAIR);
                self.code.extend_from_slice(&key_idx.to_le_bytes());
            }
            arg_n += 1;
        }

        // Compile the body. Each statement dispatches via compile_stmt, which
        // in config_mode accumulates Pairs/sub-nodes into THIS container (the
        // top of accum_stack). Nested control flow (if/for) also accumulates
        // directly into this container per legacy Config semantics.
        //
        // Plan 375: bump the compile-time accum depth so Pairs inside this
        // node body are treated as structural field defs, not parameter
        // defaults (an injected `at` must not rewrite a nested dir's `at:`).
        self.config_accum_depth += 1;
        for stmt in &node.body.stmts {
            self.compile_stmt(stmt)?;
        }
        self.config_accum_depth -= 1;

        // POP_ACCUM: pop the assembled Node, store in registry, push its id.
        self.emit(OpCode::POP_ACCUM);
        Ok(())
    }

    /// Plan 359 C5: Compile a `Body`'s statements inline (in the current scope),
    /// WITHOUT introducing a new scope. This mirrors the inner loop of the
    /// `Stmt::Block` handler (SOURCE_LINE emission + should_pop_expr_result for
    /// non-last statements) but skips push_scope/pop_scope, so `var` bindings
    /// declared in the body remain visible in the enclosing scope. Used by
    /// `Stmt::If` so variables declared inside an if-branch leak out.
    fn compile_body_inline(&mut self, body: &crate::ast::Body) -> AutoResult<()> {
        let n = body.stmts.len();
        for (i, s) in body.stmts.iter().enumerate() {
            if i < body.source_lines.len() {
                self.emit_source_line(body.source_lines[i]);
            }
            let is_last = i == n - 1;
            let old_pop = self.should_pop_expr_result;
            if !is_last {
                // Non-last statements always discard their value.
                self.should_pop_expr_result = true;
            }
            self.compile_stmt(s)?;
            self.should_pop_expr_result = old_pop;
        }
        Ok(())
    }

    fn handle_use_stmt(&mut self, use_stmt: &crate::ast::Use) {
        // Handle Rust imports (Plan 212b Task 3)
        if matches!(use_stmt.kind, crate::ast::UseKind::Rust) {
            self.handle_rust_import(use_stmt);
            return;
        }

        // Plan 216 Phase 2: Handle C imports
        if matches!(use_stmt.kind, crate::ast::UseKind::C) {
            self.handle_c_import(use_stmt);
            return;
        }

        // Plan 214: Handle Python imports
        if matches!(use_stmt.kind, crate::ast::UseKind::Py) {
            self.handle_py_import(use_stmt);
            return;
        }

        // Only handle Auto imports (not C/Rust use)
        if !matches!(use_stmt.kind, crate::ast::UseKind::Auto) {
            return;
        }

        // Plan 317: Register the module name so codegen knows `db.func()`
        // is a cross-module call (generates CALL with reloc "db.func").
        // PLAN-633: `use auto.X`（原生命名空间导入）不得把原生根 "auto" 注册
        // 为文件模块——否则全库 `auto.*` native 调用（clipboard/fs/...）被
        // is_auto_module_call 抢路由成交叉模块 CALL reloc，链接期 Undefined
        // symbol（画廊宿主 031-image-viewer `use auto.image` 实证，011/027
        // 的 dom.copy_text 连带炸）。末段照旧注册，保 Plan 347 同名库遮蔽。
        if !use_stmt.paths.is_empty() {
            let is_native_root_use =
                use_stmt.paths.len() > 1 && use_stmt.paths[0].as_str() == "auto";
            if !is_native_root_use {
                self.auto_modules.insert(use_stmt.paths[0].to_string());
                // PLAN-013 T1: file-module qualifier (single-path use, e.g.
                // `use base64`) — the only shape that may shadow a native
                // namespace of the same name at the import_scope arm.
                if use_stmt.paths.len() == 1 {
                    self.file_modules.insert(use_stmt.paths[0].to_string());
                }
            }
        }
        // Plan 347: Also track the import qualifier (last path segment, e.g.
        // "base64" from `use auto.base64`) so native-opaque-module routing can
        // be suppressed when the user has loaded a same-named Auto library.
        // Without this, `use auto.base64: decode` resolves `decode` to the
        // built-in `auto.base64.decode` native instead of the user's library.
        if !use_stmt.paths.is_empty() {
            self.auto_modules
                .insert(use_stmt.paths.last().unwrap().to_string());
        }

        // Get the module path string
        let module_path = if let Some(ref mp) = use_stmt.module_path {
            mp.display()
        } else if !use_stmt.paths.is_empty() {
            use_stmt.paths.join(".")
        } else {
            return; // No module path, nothing to do
        };

        // Only populate scope for specific item imports (e.g., use mod: name1, name2)
        if !use_stmt.items.is_empty() {
            // Plan 339 Phase 5: use file_stem as module qualifier (matching
            // collect_module_imports). E.g. `use back.api: create_note`
            // → import_scope["create_note"] = "api.create_note"
            let module_qualifier = use_stmt
                .paths
                .last()
                .map(|p| p.as_str().to_string())
                .unwrap_or_else(|| {
                    module_path
                        .split('.')
                        .last()
                        .unwrap_or(&module_path)
                        .to_string()
                });
            for item in &use_stmt.items {
                let local_name = item.as_str();
                let qualified = format!("{}.{}", module_qualifier, local_name);
                self.import_scope.insert(local_name.to_string(), qualified);
            }
        } else {
            // Module-level import (use types): record module name as known prefix
            // so Dot(Dot(Ident("types"), ...), ...) can be recognized as qualified access
            let module_name = module_path.split('.').last().unwrap_or(&module_path);
            self.known_module_prefixes.insert(module_name.to_string());
            // Plan 545: `use db: *` — explicit flat import. Map every exported
            // fn of the module to its qualified reloc name (`db.add`) so bare
            // calls bind module-qualified in the linker (dep-module exports
            // no longer register bare names). bare `use db` (not wildcard)
            // stops here: namespace-only, bare names must NOT resolve.
            if use_stmt.is_wildcard {
                if let Ok(ts) = self.type_store.read() {
                    if let Some(loaded) = ts.lookup_module(module_name) {
                        for fn_name in &loaded.export_fns {
                            self.import_scope
                                .insert(fn_name.clone(), format!("{}.{}", module_name, fn_name));
                        }
                    }
                }
            }
        }
    }

    /// Plan 212b Task 3: Handle use.rust statement and record function name mappings
    ///
    /// Records each imported function name in `rust_native_map` so that
    /// when the codegen encounters a call to that function, it can emit
    /// CALL_NAT instead of a regular CALL.
    fn handle_rust_import(&mut self, use_stmt: &crate::ast::Use) {
        // Build module path from AST
        let module_path = if let Some(ref mp) = use_stmt.module_path {
            mp.display()
        } else if !use_stmt.paths.is_empty() {
            use_stmt.paths.join("::")
        } else {
            return;
        };

        // Extract crate name (first segment of :: path)
        let crate_name = module_path
            .split("::")
            .next()
            .unwrap_or(&module_path)
            .to_string();

        // Also register the crate name itself as a module prefix
        // so that e.g. `log.set_boxed_logger(...)` resolves instead of
        // erroring with "Undefined variable: log"
        if !self.rust_native_map.contains_key(&crate_name) {
            self.rust_native_map
                .insert(crate_name.clone(), (crate_name.clone(), crate_name.clone()));
        }

        // Collect items to register: explicit items list, or last path segment if items is empty
        let items_to_register: Vec<&str> = if !use_stmt.items.is_empty() {
            use_stmt.items.iter().map(|s| s.as_str()).collect()
        } else if use_stmt.paths.len() > 1 {
            // use.rust regex::Regex → paths=["regex","Regex"] → register "Regex"
            vec![use_stmt.paths.last().unwrap().as_str()]
        } else if use_stmt.paths.len() == 1 {
            // use.rust walkdir → register "walkdir" as module-level import
            vec![use_stmt.paths[0].as_str()]
        } else {
            vec![]
        };

        for local_name in &items_to_register {
            let full_path = format!("{}::{}", module_path, local_name);
            self.rust_native_map
                .insert(local_name.to_string(), (crate_name.clone(), full_path));
            // Phase 2.1: Infer return type from known signature
            // Opaque constructors (parse, new, now, etc.) return heap object handles (Int),
            // not strings. Default to Int for functions without known signatures.
            let ret_type = crate::ffi::resolve_signature(&crate_name, local_name)
                .map(|sig| match sig.returns {
                    crate::ffi::RustType::Void => Type::Void,
                    crate::ffi::RustType::Bool => Type::Bool,
                    crate::ffi::RustType::Int => Type::Int,
                    crate::ffi::RustType::Long => Type::I64,
                    crate::ffi::RustType::Float => Type::Float,
                    crate::ffi::RustType::Double => Type::Double,
                    _ => Type::StrFixed(0),
                })
                .unwrap_or_else(|| {
                    // Opaque constructor detection: methods that return heap handles
                    let name = *local_name;
                    let is_opaque_ctor = matches!(
                        name,
                        "parse"
                            | "new"
                            | "now"
                            | "from_str"
                            | "from_reader"
                            | "from_writer"
                            | "open"
                            | "connect"
                            | "bind"
                            | "listen"
                            | "build"
                            | "from_path"
                            | "from_secs"
                            | "from_millis"
                    );
                    if is_opaque_ctor {
                        Type::Int
                    } else {
                        Type::StrFixed(0)
                    }
                });
            self.fn_return_types
                .insert(local_name.to_string(), ret_type);
        }
    }

    /// Plan 216 Phase 2: Handle `use c <header.h>` statement.
    ///
    /// Loads the manifest for the given C header and registers all its functions
    /// as C-FFI native shims. The native IDs are stored in `c_ffi_functions` so
    /// that subsequent function calls can emit CALL_NAT.
    fn handle_c_import(&mut self, use_stmt: &crate::ast::Use) {
        // The C header path is stored in paths (e.g. ["<string.h>"])
        let header = if !use_stmt.paths.is_empty() {
            use_stmt.paths[0].as_str().to_string()
        } else {
            return;
        };

        // Load the manifest: built-in dataset first, then a JSON file path
        // (Plan 597: non-builtin datasets resolve as-given/CWD-relative).
        let clean = header.trim_start_matches('<').trim_end_matches('>');
        let manifest = match crate::vm::ffi::c_ffi::load_builtin_manifest(&header) {
            Some(m) => m,
            None => {
                if clean.ends_with(".json") {
                    match crate::vm::ffi::c_ffi::load_manifest_file(clean) {
                        Some(m) => m,
                        None => {
                            log::warn!("No C-FFI manifest file readable: {}", clean);
                            return;
                        }
                    }
                } else {
                    log::warn!("No C-FFI manifest found for header: {}", header);
                    return;
                }
            }
        };

        // Register each function in the C-FFI runtime and record native IDs
        // We use a lazy-static pattern: the CFfiRuntime lives in a global registry
        // so it can be accessed both at codegen time (to get IDs) and at VM runtime
        // (to provide the shims).
        let mut cffi = CFFI_GLOBAL.lock().unwrap();
        if let Err(e) = cffi.load_header(&manifest) {
            log::warn!("Failed to load C-FFI header {}: {:?}", header, e);
            return;
        }

        // Record native IDs for each function so codegen can emit CALL_NAT
        for func in &manifest.functions {
            if func.variadic {
                continue;
            }
            if let Some(native_id) = cffi.get_function_id(&func.name) {
                self.c_ffi_functions.insert(func.name.clone(), native_id);
            }
        }
    }

    /// Plan 214/300: Handle `use.py module::{items}` statement.
    ///
    /// Records each imported Python function name in `py_native_map` so that
    /// when the codegen encounters a call to that function, it can emit
    /// CALL_NAT instead of a regular CALL. Also sets return type to Auto.
    ///
    /// Plan 300: For bare `use.py module` (no items), records the module name
    /// in `py_modules` so dot-calls like `module.method()` can be resolved.
    /// Plan 567 T19: 语句根裸消费 nullable 返回的 py 调用 → lint 记录
    ///（仅 Ident 直呼形态；`.?`/`??` 包装后的节点不再是裸 Call）。
    fn check_nullable_unguarded(&self, e: &Expr) {
        if let Expr::Call(c) = e {
            if let Expr::Ident(n) = c.name.as_ref() {
                let name = n.as_str();
                if self.py_native_map.contains_key(name)
                    && matches!(
                        self.py_return_types.get(name),
                        Some(crate::py_ffi_types::PyType::Nullable(_))
                    )
                {
                    crate::py_ffi_types::record_nullable_lint(format!("var = {}(...)", name));
                }
            }
        }
    }

    fn handle_py_import(&mut self, use_stmt: &crate::ast::Use) {
        let module_path = if let Some(ref mp) = use_stmt.module_path {
            mp.display()
        } else if !use_stmt.paths.is_empty() {
            use_stmt.paths.join(".")
        } else {
            return;
        };

        if !use_stmt.items.is_empty() {
            for item in &use_stmt.items {
                let local_name = item.as_str();
                let full_path = format!("{}.{}", module_path, local_name);
                self.py_native_map
                    .insert(local_name.to_string(), (module_path.to_string(), full_path));
                // Plan 300: Auto return type for dynamic marshalling
                self.fn_return_types
                    .insert(local_name.to_string(), Type::StrFixed(0));
                // Plan 567 T17（W3 注解预言机）: 有返回注解知识则灌注
                //（Float/Int → shim 出口 D4 强制；Nullable → T19 lint 消费）；
                // 无知识 = Auto 零变化。
                let anno_type = crate::py_ffi_types::lookup_return_annotation(local_name)
                    .unwrap_or(crate::py_ffi_types::PyType::Auto);
                self.py_return_types
                    .insert(local_name.to_string(), anno_type);
            }
        } else {
            // Plan 300: Bare module import (`use.py math`) — record for dot-call resolution
            self.py_modules.insert(module_path.to_string());
        }

        // Plan 369 Task 12: seed the py-object built-ins (py_call, py_getattr)
        // so calls like `py_call(d, "isoformat")` route through CALL_NAT_COUNTED with the
        // correct arg count. These resolve to fixed native IDs registered by
        // init_py_ffi. Marking them in py_native_map also sets is_py_ffi_call,
        // so the runtime arg-count byte is emitted.
        self.register_py_object_builtins();
    }
}
