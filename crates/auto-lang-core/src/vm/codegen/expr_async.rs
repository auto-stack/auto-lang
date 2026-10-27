//! PLAN-843 结构拆分：compile_expr_inner 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl Codegen {
    /// PLAN-843 尾递归派发段（自 compile_expr_inner 连续臂切片，臂体逐字保留）。
    pub(super) fn expr_async(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            // Plan 124: Async block - ~{ stmts }
            // Plan 348 Task 22: compile the body OUT-OF-LINE (like a closure) and
            // hand CREATE_FUTURE a real bytecode address. This lets SPAWN_GO spawn
            // a genuine background task (rather than running the body inline) and
            // lets .await run the body on demand. Captured free variables are
            // passed through the future's env so the spawned task can read/write
            // them via LOAD_CAPTURED / STORE_CAPTURED (same mechanism as closures).
            Expr::AsyncBlock {
                body,
                return_type: _,
            } => {
                // Step 1: Find free variables referenced by the body (captures).
                // Reuse the closure free-var analysis by wrapping the Body in an
                // Expr::Block; collect_free_vars understands that variant.
                let empty_params: HashSet<String> = HashSet::new();
                let free_vars: Vec<String> = {
                    let mut free: HashSet<String> = HashSet::new();
                    for stmt in &body.stmts {
                        match stmt {
                            Stmt::Expr(e) => self.collect_free_vars(e, &empty_params, &mut free),
                            Stmt::Return(e) => self.collect_free_vars(e, &empty_params, &mut free),
                            _ => {}
                        }
                    }
                    free.into_iter().collect()
                };

                // Step 2: Emit LOAD_LOC for each captured variable so the values
                // are on the stack for CREATE_FUTURE to consume.
                for var_name in &free_vars {
                    if let Some(var_index) = self.lookup_var(var_name) {
                        self.emit_load_loc(var_index);
                    } else {
                        // Not a local — push a zero placeholder so the stack
                        // layout stays consistent. This matches the closure
                        // codegen fallback for missing captures.
                        self.emit(OpCode::CONST_0);
                    }
                }

                // Step 3: Emit CREATE_FUTURE. Its immediates are:
                //   body_offset: u32   (resolved via a reloc, like closures, so
                //                       the FN_PROLOG/RESERVE_STACK insertion
                //                       performed by enclosing fn codegen shifts
                //                       both the patch site and the body address)
                //   capture_count: u8
                //   capture_name_idx: u16  (repeated capture_count times)
                // CREATE_FUTURE pops `capture_count` values and records them in
                // the future's environment, keyed by the capture names.
                self.emit(OpCode::CREATE_FUTURE);
                let body_offset_pos = self.code.len() as u32;
                self.code.extend_from_slice(&(0u32).to_le_bytes()); // placeholder body offset
                self.code.push(free_vars.len() as u8); // capture_count
                for var_name in &free_vars {
                    let var_idx = self.add_string(var_name);
                    self.code.extend_from_slice(&var_idx.to_le_bytes());
                }
                // Unique symbol name for the out-of-line body; resolved at link
                // time via the reloc we register in Step 7.
                let body_symbol = format!("async_block_body_{}", body_offset_pos);

                // Step 4: JMP over the out-of-line body so normal execution flow
                // skips it (the body is only reached via SPAWN_GO or .await).
                self.emit(OpCode::JMP);
                let jmp_offset_pos = self.code.len();
                self.code.extend_from_slice(&(0i16).to_le_bytes()); // patched after body

                // Step 5: Compile the body as a standalone out-of-line function.
                // The body address is what we feed back into CREATE_FUTURE.
                let body_addr = self.code.len() as u32;

                // Make captures visible to the body so LOAD_CAPTURED / STORE_CAPTURED
                // are emitted instead of LOAD_LOC / STORE_LOC.
                let mut new_captured_vars: HashMap<String, usize> = HashMap::new();
                for (idx, var_name) in free_vars.iter().enumerate() {
                    new_captured_vars.insert(var_name.clone(), idx);
                }
                self.push_captured_vars(new_captured_vars);

                // Give the body its own scope so any local declarations inside
                // don't leak into the enclosing function.
                self.push_scope();

                // Treat the body like a function body: keep the last expression's
                // value on the stack so an explicit .await path (which DOES pop a
                // return value via RET when bp != 0) has something to read. For
                // the SPAWN_GO path, the spawned task has bp == 0 so RET short-
                // circuits to Terminated without popping — no underflow risk.
                let saved_should_pop = self.should_pop_expr_result;
                self.should_pop_expr_result = false;
                for stmt in &body.stmts {
                    self.compile_stmt(stmt)?;
                }
                self.should_pop_expr_result = saved_should_pop;

                // If the body's last statement didn't push a value (e.g. an
                // assignment, if/for without a tail expression, etc.), push nil
                // so the .await path's RET has something to return. When the
                // body already produced a value, last_expr_type will be non-Void
                // and we leave the stack alone.
                if matches!(self.last_expr_type, ObjectType::Void) {
                    self.emit(OpCode::PUSH_NIL);
                }

                // The body must always end with RET so a spawned task running it
                // can terminate cleanly. n_args = 0 (async blocks take no params).
                self.emit(OpCode::RET);
                self.code.push(0u8); // n_args = 0

                self.pop_scope();
                self.pop_captured_vars();

                // Step 6: Back-fill the JMP so the inline path skips the body.
                let body_end_addr = self.code.len() as u32;
                let jmp_offset = ((body_end_addr as i32) - (jmp_offset_pos as i32 + 2)) as i16;
                let jmp_bytes = jmp_offset.to_le_bytes();
                for (i, byte) in jmp_bytes.iter().enumerate() {
                    self.code[jmp_offset_pos + i] = *byte;
                }

                // Step 7: Export the body address and register a reloc so the
                // enclosing function's FN_PROLOG/RESERVE_STACK insertion (which
                // happens AFTER this codegen and shifts everything past the
                // function entry_point) keeps the body_offset site and the body
                // address in sync. The loader resolves the reloc by writing the
                // (adjusted) exported symbol address into the patch site.
                self.exports.insert(body_symbol.clone(), body_addr);
                self.relocs.push(crate::vm::loader::RelocEntry {
                    offset: body_offset_pos,
                    symbol_name: body_symbol,
                    reloc_type: crate::vm::loader::RelocType::FuncCall,
                    source_pos: None,
                });
            }
            // Plan 124: Await expression - expr.await
            Expr::Await { expr } => {
                // Check if the inner expression is a synchronous native call
                // that doesn't return a Future (e.g., http.post_sync, http.post_bearer)
                let mut skip_await = false;
                match expr.as_ref() {
                    Expr::Dot(obj, method) => {
                        if let Expr::Ident(obj_name) = obj.as_ref() {
                            if obj_name.as_str() == "http"
                                && (method.as_str() == "post_sync"
                                    || method.as_str() == "post_bearer")
                            {
                                skip_await = true;
                            }
                        }
                    }
                    Expr::Call(call_expr) => {
                        if let Expr::Dot(obj, method) = call_expr.name.as_ref() {
                            if let Expr::Ident(obj_name) = obj.as_ref() {
                                if obj_name.as_str() == "http"
                                    && (method.as_str() == "post_sync"
                                        || method.as_str() == "post_bearer")
                                {
                                    skip_await = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // Compile the inner expression (should evaluate to a Future)
                self.compile_expr(expr)?;

                if skip_await {
                    // Synchronous native — already pushed the result, no await needed
                } else {
                    // Emit AWAIT_FUTURE to wait for the future's completion
                    self.emit(OpCode::AWAIT_FUTURE);
                }
                // Unwrap the inner type from Future<T> for type-aware dispatch
                if let Type::GenericInstance(ref inst) = self.infer_expr_type(expr) {
                    if inst.base_name.as_ref() == "Future" {
                        if let Some(inner) = inst.args.first() {
                            self.last_expr_type = match inner {
                                Type::StrFixed(_)
                                | Type::StrOwned
                                | Type::CStrLit
                                | Type::StrSlice => ObjectType::String,
                                Type::Float => ObjectType::Float,
                                Type::Double => ObjectType::Double,
                                Type::Int | Type::I64 => ObjectType::Int,
                                Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                                Type::Bool => ObjectType::Bool,
                                _ => ObjectType::NestedObject,
                            };
                        }
                    }
                }
            }
            // Plan 126: Go expression - expr.go (spawn background task)
            // Fire-and-forget semantics: spawn the future and discard the result
            Expr::Go { expr } => {
                // Compile the inner expression (should evaluate to a Future).
                // For an async block `~{ ... }`, the codegen compiles the body
                // inline and leaves a single Future value on the stack.
                self.compile_expr(expr)?;
                // Emit SPAWN_GO to spawn the future in background. SPAWN_GO pops
                // the single Future value and spawns it (fire-and-forget); it
                // pushes nothing back, so the net stack effect is zero.
                self.emit(OpCode::SPAWN_GO);
                // Plan 348 G1: SPAWN_GO leaves nothing on the stack (void), so
                // the statement-level POP must be suppressed. Reuse the
                // existing `last_was_native_void` signal so `needs_pop` is
                // false and no stray POP eats a value below the stack,
                // which previously caused a stack underflow after a few spawns.
                self.last_was_native_void = true;
                self.last_expr_type = ObjectType::Void;
            }
            // Plan 321: yield expression — compile inner expr, then emit YIELD_VAL
            Expr::Yield(expr) => {
                self.compile_expr(expr)?; // Push yielded value onto stack
                self.emit(OpCode::YIELD_VAL); // Pop value, suspend generator
                                              // After resume, generator continues from next statement.
                                              // No return value pushed (yield's value is void in MVP).
            }
            Expr::Pair(pair) => {
                // Handle Pair as a single-element object for config syntax like: name: "value"
                // This is equivalent to Object {key: value}
                self.compile_expr(&pair.value)?;

                // Store key in the object_keys pool
                let key = self.ast_key_to_value_key(&pair.key);
                let key_index = self.object_keys.len() as u32;
                self.object_keys.push(vec![key.clone()]);

                // Track field type
                let ty = self.infer_object_type(&pair.value);
                self.object_types.push(vec![ty]);

                // Emit CREATE_OBJ with key_index and field count (1)
                self.emit(OpCode::CREATE_OBJ);
                self.code.extend_from_slice(&key_index.to_le_bytes());
                self.code.push(1); // field_count = 1
            }
            // Plan 095: Compile-time expression #{ expr }
            // For now, compile the inner expression normally
            // TODO: In full implementation, evaluate at compile time and substitute the result
            Expr::Comptime(hash_brace) => {
                // Compile the inner expression
                // The result will be on the stack
                self.compile_expr(&hash_brace.expr)?;
            }
            // Hold expression: bind value to name, evaluate body
            // For MVP: compile path, compile body statements
            Expr::Hold(hold) => {
                // Compile the path expression (the value to hold)
                self.compile_expr(&hold.path)?;
                // Compile the body statements
                for stmt in &hold.body.stmts {
                    self.compile_stmt(stmt)?;
                }
            }
            // Nil expression: push nil marker
            // PLAN-053 P-053-2 —— 原编码 CONST_I32 i32::MIN+1 与 `None`
            // (encode_null)、`null` 三种 nil 家族字面量表示分裂，比较/合并
            // 语义随表示漂移。归一到 PUSH_NIL(tag-null)后三者全链等值。
            Expr::Nil => {
                self.emit(OpCode::PUSH_NIL);
            }
            // Null literal: PLAN-053 P-053-2 —— 原编码 CONST_I32 -1 与 `None`
            // 字面量(PUSH_NIL → encode_null)表示分裂：EQ/NE 快速路径
            // `a_nv == b_nv` 不命中、tag 混合落兜底，`null != None` 恒真
            // (musk `.store.current_gate != None` 守卫拦不住 null 态，
            // GateCard 常显)；NULL_COALESCE 也只认 is_null，`null ?? x` 落
            // -1。归一到 PUSH_NIL 后存储/读回/比较与 JSON null、`None`
            // 全链等值(engine 侧 IS_NIL/IS_SOME/UNWRAP_SOME 的
            // `is_null || i32==-1` 特判对新旧编码都成立)。
            Expr::Null => {
                self.emit(OpCode::PUSH_NIL);
            }
            _ => return self.expr_fallback(expr),
        }
        Ok(())
    }
}
