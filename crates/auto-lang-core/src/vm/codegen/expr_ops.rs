//! PLAN-843 结构拆分：compile_expr_inner 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl Codegen {
    /// PLAN-843 尾递归派发段（自 compile_expr_inner 连续臂切片，臂体逐字保留）。
    pub(super) fn expr_ops(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            Expr::Bina(lhs, op, rhs) => {
                // Assignment is special: compile RHS first, then store to LHS
                if matches!(
                    op,
                    Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq | Op::ModEq
                ) {
                    // For a += b, compile: LOAD_LOC(a), LOAD_CONST(b), ADD, STORE_LOC(a)
                    // IMPORTANT: Order matters! Stack must have [a, b] so DIV computes a/b (not b/a)
                    // Since binary ops pop b then a, we need a pushed before b
                    if let Expr::Ident(name) = lhs.as_ref() {
                        let name_str = name.to_string();

                        // Plan 317: compound assignment to actor state field
                        // (e.g., count += 1 inside a handler). LOAD_STATE_FIELD,
                        // compile RHS, op, DUP, STORE_STATE_FIELD.
                        if let Some(&field_idx) = self.current_task_state_fields.get(&name_str) {
                            self.emit(OpCode::LOAD_STATE_FIELD);
                            self.code.push(field_idx);
                            self.compile_expr(rhs)?;
                            // Plan 403-F Bug B: pick float/double/int opcode by
                            // operand types, not a hardcoded i32 ADD/SUB/....
                            let is_d = self.is_double_operation(lhs, rhs);
                            let is_f = !is_d && self.is_float_operation(lhs, rhs);
                            self.emit(match op {
                                Op::AddEq => {
                                    if is_d {
                                        OpCode::ADD_D
                                    } else if is_f {
                                        OpCode::ADD_F
                                    } else {
                                        OpCode::ADD
                                    }
                                }
                                Op::SubEq => {
                                    if is_d {
                                        OpCode::SUB_D
                                    } else if is_f {
                                        OpCode::SUB_F
                                    } else {
                                        OpCode::SUB
                                    }
                                }
                                Op::MulEq => {
                                    if is_d {
                                        OpCode::MUL_D
                                    } else if is_f {
                                        OpCode::MUL_F
                                    } else {
                                        OpCode::MUL
                                    }
                                }
                                Op::DivEq => {
                                    if is_d {
                                        OpCode::DIV_D
                                    } else if is_f {
                                        OpCode::DIV_F
                                    } else {
                                        OpCode::DIV
                                    }
                                }
                                Op::ModEq => {
                                    if is_d {
                                        OpCode::MOD_D
                                    } else if is_f {
                                        OpCode::MOD_F
                                    } else {
                                        OpCode::MOD
                                    }
                                }
                                _ => OpCode::NOP,
                            });
                            self.emit(OpCode::DUP);
                            self.emit(OpCode::STORE_STATE_FIELD);
                            self.code.push(field_idx);
                        // Check if this is a captured variable (Plan 071)
                        } else if self.current_captured_vars().contains_key(&name_str) {
                            // Variable is captured - we need different handling
                            return Err(crate::error::AutoError::Msg(
                                "Compound assignment to captured variables not yet supported in AutoVM".to_string(),
                            ));
                        } else if let Some(var_index) = self.lookup_var(&name_str) {
                            // Variable found in local scope - check mutability
                            if let Some(is_mutable) = self.var_mutability.get(&name_str) {
                                if !is_mutable {
                                    return Err(crate::error::AutoError::Msg(
                                        format!("Cannot reassign to immutable variable '{}' (declared with 'let')", name_str)
                                    ));
                                }
                            }
                            // Load variable FIRST (for correct operand order)
                            self.emit_load_loc(var_index);

                            // Compile RHS (value to add/sub/mul/div/mod)
                            self.compile_expr(rhs)?;

                            // Perform operation
                            // Plan 403-F Bug B: pick float/double/int opcode by
                            // operand types, not a hardcoded i32 ADD/SUB/....
                            let is_d = self.is_double_operation(lhs, rhs);
                            let is_f = !is_d && self.is_float_operation(lhs, rhs);
                            self.emit(match op {
                                Op::AddEq => {
                                    if is_d {
                                        OpCode::ADD_D
                                    } else if is_f {
                                        OpCode::ADD_F
                                    } else {
                                        OpCode::ADD
                                    }
                                }
                                Op::SubEq => {
                                    if is_d {
                                        OpCode::SUB_D
                                    } else if is_f {
                                        OpCode::SUB_F
                                    } else {
                                        OpCode::SUB
                                    }
                                }
                                Op::MulEq => {
                                    if is_d {
                                        OpCode::MUL_D
                                    } else if is_f {
                                        OpCode::MUL_F
                                    } else {
                                        OpCode::MUL
                                    }
                                }
                                Op::DivEq => {
                                    if is_d {
                                        OpCode::DIV_D
                                    } else if is_f {
                                        OpCode::DIV_F
                                    } else {
                                        OpCode::DIV
                                    }
                                }
                                Op::ModEq => {
                                    if is_d {
                                        OpCode::MOD_D
                                    } else if is_f {
                                        OpCode::MOD_F
                                    } else {
                                        OpCode::MOD
                                    }
                                }
                                _ => OpCode::NOP,
                            });

                            // Plan 080: Duplicate value on stack because THIS IS AN EXPRESSION
                            // The parent (Stmt::Expr) or outer expression expects 1 value result!
                            self.emit(OpCode::DUP);
                            // Store result back to variable
                            self.emit_store_loc(var_index);
                        } else {
                            // Variable not found - error
                            return Err(crate::error::AutoError::Msg(format!(
                                "Undefined variable '{}' in compound assignment",
                                name_str
                            )));
                        }
                    } else {
                        // LHS is not an identifier - error for compound assignment
                        return Err(crate::error::AutoError::Msg(
                            "Compound assignment requires a variable on left side".to_string(),
                        ));
                    }
                } else if *op == Op::Asn {
                    // Compile RHS (value to store)
                    self.compile_expr(rhs)?;

                    // Check if LHS is an identifier (variable assignment)
                    if let Expr::Ident(name) = lhs.as_ref() {
                        let name_str = name.to_string();

                        // Coerce RHS to match LHS type if needed
                        let asn_stored_type = self.var_types.get(&name_str).cloned();
                        // PLAN-026 T-03 配套: 同声明位分流(I64 sign-extend)。
                        if matches!(asn_stored_type, Some(Type::I64))
                            && !self.contains_u64(rhs.as_ref())
                        {
                            self.emit(OpCode::TYPE_CAST_I64);
                        } else if matches!(asn_stored_type, Some(Type::U64))
                            && !self.contains_u64(rhs.as_ref())
                        {
                            self.emit(OpCode::TYPE_CAST_U64);
                        } else if matches!(asn_stored_type, Some(Type::Double))
                            && self.last_expr_type != ObjectType::Double
                        {
                            if matches!(self.last_expr_type, ObjectType::Float) {
                                self.emit(OpCode::PROMOTE_F64);
                            }
                        }

                        // Plan 317: actor state field assignment (inside task
                        // hooks/handlers). DUP keeps the value as the assignment
                        // expression result, then STORE_STATE_FIELD persists it.
                        if let Some(&field_idx) = self.current_task_state_fields.get(&name_str) {
                            self.emit(OpCode::DUP);
                            self.emit(OpCode::STORE_STATE_FIELD);
                            self.code.push(field_idx);
                        // Plan 317: module-level global variable (check BEFORE
                        // local lookup — otherwise top-level var would store to
                        // a script-wrapper local that fns can't see).
                        } else if self.global_vars.contains(&name_str) {
                            // Plan 377: globals 与栈一致地单槽化。RHS 现在是单槽 NanoValue，
                            // 无需 Plan 378 的 POP-high hack（globals 已能存完整 64 位值）。
                            self.emit(OpCode::DUP);
                            self.emit_global_store(&name_str);
                        // Check if this is a captured variable (Plan 071)
                        } else if self.current_captured_vars().contains_key(&name_str) {
                            // Variable is captured - emit STORE_CAPTURED
                            self.emit(OpCode::DUP); // Keep value for expression result
                            self.emit_store_captured(&name_str);
                        } else if let Some(var_index) = self.lookup_var(&name_str) {
                            // Variable found in local scope - check mutability
                            if let Some(is_mutable) = self.var_mutability.get(&name_str) {
                                if !is_mutable {
                                    return Err(crate::error::AutoError::Msg(
                                        format!("Cannot reassign to immutable variable '{}' (declared with 'let')", name_str)
                                    ));
                                }
                            }
                            // Plan 377: 全值单槽化 —— 恒为 DUP + 单 STORE_LOC。
                            self.emit(OpCode::DUP); // Keep value for expression result
                            self.emit_store_loc(var_index);
                        } else if self.global_vars.contains(&name_str) {
                            // Plan 317: module-level global variable assignment.
                            self.emit(OpCode::DUP);
                            self.emit_global_store(&name_str);
                        } else {
                            // Variable not found - this is an error
                            // For now, emit STORE_LOC_0 as a fallback
                            // TODO: Proper error handling for undefined variables
                            self.emit(OpCode::DUP); // Keep value for expression result
                            self.emit(OpCode::STORE_LOC_0);
                        }
                    } else if let Expr::Index(array, index) = lhs.as_ref() {
                        // Array element assignment: arr[index] = value
                        // Stack has: value (from RHS compilation above)
                        // Need to compile: array, index, then emit SET_ELEM
                        // Compile array expression
                        self.compile_expr(array)?;
                        // Compile index expression
                        self.compile_expr(index)?;
                        // Now stack has: value, array_id, index (need to reorder)
                        // SET_ELEM expects: array_id, index, value
                        // So we need to swap: value, array_id, index -> array_id, index, value
                        // For now, let's use a simpler approach:
                        // 1. Compile array (push array_id)
                        // 2. Compile index (push index)
                        // 3. Emit SET_ELEM (pops array_id, index, value from stack)
                        // But the value is already on stack from RHS!
                        // So we need: SWAP to get array_id to top, then SWAP again...
                        // Actually, let's reorder: compile array, index, RHS
                        // But we already compiled RHS...

                        // Simpler: emit SWAP instructions to reorder
                        // Current stack: value (top), need: array, index, value (bottom)
                        // After compiling array and index: value, array, index (top)
                        // We want: array, index, value
                        // So: SWAP (value, array -> array, value), then rotate...
                        // This is getting complex. Let's use a simpler approach:
                        // Just swap value to bottom after compiling array and index

                        // Actually, the stack after RHS is: [value]
                        // After compiling array: [value, array_id]
                        // After compiling index: [value, array_id, index]
                        // SET_ELEM wants: [array_id, index, value]
                        // So we need to rotate: SWAP index<->value -> [value, array_id, index]
                        // No wait, SWAP swaps top two: [value, array_id, index] -> [value, array_id, index] (no change if we swap index and value?)
                        // Let me think again... SWAP swaps top 2 elements
                        // [value, array_id, index] -> SWAP -> [value, array_id, index]?? No...
                        // SWAP swaps top two: index and array_id -> [value, index, array_id]
                        // Then SWAP again: [value, index, array_id] -> [index, value, array_id]
                        // This is confusing. Let's just emit the opcodes in the right order.

                        // Better approach: compile RHS last
                        // But that's a big refactor...

                        // Simplest fix: Add more swap opcodes or just handle it
                        // For now, let's use: we have [value, array, index] and want [array, index, value]
                        // Rotate top 3: [value, array, index] -> [array, index, value]
                        // This needs a ROTATE opcode or we can use SWAP twice:
                        // SWAP: [value, array, index] -> [value, index, array] (swaps index<->array)
                        // No wait, SWAP swaps top TWO elements, not the bottom two

                        // Let me re-read SWAP implementation...
                        // Looking at the code, I think SWAP swaps sp[-1] and sp[-2]
                        // So [value, array, index] with index at sp-1, array at sp-2
                        // SWAP -> [value, index, array]

                        // OK so the approach is:
                        // [value, array, index] -> SWAP -> [value, index, array]
                        // Then SWAP value and index? No, SWAP only swaps top 2...

                        // Let me just use a practical approach:
                        // 1. POP value to temp
                        // 2. Compile array, index
                        // 3. PUSH value back
                        // 4. SET_ELEM
                        // But we don't have a POP_TEMP opcode...

                        // Actually, the simplest is:
                        // Just compile in order: array, index, value (RHS)
                        // But the code already compiled RHS first...

                        // For now, let's use the existing stack:
                        // [value, array, index]
                        // We need [array, index, value]
                        // Solution: Change the codegen to compile array, index, RHS in that order
                        // But that's a bigger change...

                        // Quick fix: Accept the current order and change SET_ELEM to expect [value, array, index]
                        self.emit(OpCode::SET_ELEM); // Expects: value, array_id, index
                                                     // SET_ELEM doesn't push a return value - mark as void to prevent
                                                     // Stmt::Expr from emitting a POP that would corrupt the stack
                        self.last_expr_type = ObjectType::Void;
                        self.last_was_native_void = true;
                    } else if let Expr::Dot(obj, field) = lhs.as_ref() {
                        // Plan 075: Field assignment: obj.field = value
                        // Plan 087 Phase 2: Support generic instance field assignment
                        // Stack has: value (from RHS compilation above)

                        // Check if obj is a generic instance variable
                        let is_generic_instance = if let Expr::Ident(var_name) = obj.as_ref() {
                            // Look up variable type
                            if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                                matches!(var_type, Type::GenericInstance(_))
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_generic_instance {
                            // Plan 087 Phase 2: Generic instance field assignment
                            // Compile object expression (pushes instance_id onto stack)
                            self.compile_expr(obj)?;
                            // Now stack has: value, instance_id

                            // Get field index from generic registry
                            if let Expr::Ident(var_name) = obj.as_ref() {
                                if let Some(Type::GenericInstance(ref inst)) =
                                    self.var_types.get(var_name.as_ref())
                                {
                                    // Generate mono_name from base_name and args
                                    let mono_name = self
                                        .generic_registry
                                        .get_template(&inst.base_name.to_string())
                                        .map(|t| t.mono_name_from_args(&inst.args))
                                        .unwrap_or_else(|| format!("{}_unknown", inst.base_name));

                                    // Get ClassType to find field index
                                    if let Some(class_type) =
                                        self.generic_registry.get_type(&mono_name)
                                    {
                                        let field_str = field.to_string();
                                        if let Some(field_index) =
                                            class_type.field_index(&field_str)
                                        {
                                            // Emit SET_GENERIC_FIELD: code layout [opcode, field_index:u32]
                                            self.emit(OpCode::SET_GENERIC_FIELD);
                                            self.emit_u32(field_index as u32);
                                        } else {
                                            eprintln!("Warning: Field '{}' not found in generic type '{}' (assignment)",
                                                field, inst.base_name);
                                            // Fallback: emit placeholder
                                            self.emit(OpCode::SET_GENERIC_FIELD);
                                            self.emit_u32(0);
                                        }
                                    } else {
                                        eprintln!("Warning: Generic type '{}' not found in registry (assignment)", mono_name);
                                        // Fallback to regular field access
                                        let field_str = field.to_string();
                                        let field_bytes = field_str.as_bytes().to_vec();
                                        let field_idx = self.strings.len() as u32;
                                        self.strings.push(field_bytes);
                                        self.emit(OpCode::LOAD_STR);
                                        self.code.extend_from_slice(&field_idx.to_le_bytes());
                                        self.emit(OpCode::SET_FIELD);
                                    }
                                } else {
                                    // Fallback to regular field access
                                    let field_str = field.to_string();
                                    let field_bytes = field_str.as_bytes().to_vec();
                                    let field_idx = self.strings.len() as u32;
                                    self.strings.push(field_bytes);
                                    self.emit(OpCode::LOAD_STR);
                                    self.code.extend_from_slice(&field_idx.to_le_bytes());
                                    self.emit(OpCode::SET_FIELD);
                                }
                            } else {
                                // Fallback to regular field access
                                let field_str = field.to_string();
                                let field_bytes = field_str.as_bytes().to_vec();
                                let field_idx = self.strings.len() as u32;
                                self.strings.push(field_bytes);
                                self.emit(OpCode::LOAD_STR);
                                self.code.extend_from_slice(&field_idx.to_le_bytes());
                                self.emit(OpCode::SET_FIELD);
                            }
                        } else {
                            // Regular field assignment (Plan 075)
                            // Or nested field assignment on user type (Plan 118 Phase 7)
                            // Compile object expression
                            self.compile_expr(obj)?;

                            // Check if the result is a heap object (nested user type)
                            let obj_expr_type = self.infer_expr_type(obj);
                            vm_debug!(
                                "DEBUG ASSIGN: obj={:?}, field={}, obj_expr_type={:?}",
                                obj,
                                field,
                                obj_expr_type
                            );
                            let is_user_type =
                                matches!(obj_expr_type, Type::User(_) | Type::GenericInstance(_));
                            let is_heap_object =
                                is_user_type || self.last_expr_type == ObjectType::NestedObject;
                            vm_debug!("DEBUG ASSIGN: is_user_type={}, is_heap_object={}, last_expr_type={:?}",
                                is_user_type, is_heap_object, self.last_expr_type);

                            if is_heap_object || is_user_type {
                                // Get type name from inferred type
                                let type_name = match &obj_expr_type {
                                    Type::User(type_decl) => type_decl.name.to_string(),
                                    Type::GenericInstance(inst) => self
                                        .generic_registry
                                        .get_template(&inst.base_name.to_string())
                                        .map(|t| t.mono_name_from_args(&inst.args))
                                        .unwrap_or_else(|| format!("{}_unknown", inst.base_name)),
                                    _ => {
                                        // Fallback: try var_types for Ident
                                        if let Expr::Ident(var_name) = obj.as_ref() {
                                            if let Some(var_type) =
                                                self.var_types.get(var_name.as_ref())
                                            {
                                                match var_type {
                                                    Type::User(type_decl) => {
                                                        type_decl.name.to_string()
                                                    }
                                                    Type::GenericInstance(inst) => self
                                                        .generic_registry
                                                        .get_template(&inst.base_name.to_string())
                                                        .map(|t| t.mono_name_from_args(&inst.args))
                                                        .unwrap_or_else(|| {
                                                            format!("{}_unknown", inst.base_name)
                                                        }),
                                                    _ => "Unknown".to_string(),
                                                }
                                            } else {
                                                "Unknown".to_string()
                                            }
                                        } else {
                                            "Unknown".to_string()
                                        }
                                    }
                                };

                                // Check if this is truly a generic instance (needs SET_GENERIC_FIELD)
                                // Non-generic user types registered in generic_registry for field lookup
                                // should use SET_FIELD because they're created with CREATE_OBJ, not NEW_INSTANCE
                                let is_generic_instance = self
                                    .generic_registry
                                    .get_template(&type_name)
                                    .map(|t| !t.generic_params.is_empty())
                                    .unwrap_or(false);
                                if is_generic_instance {
                                    if let Some(class_type) =
                                        self.generic_registry.get_type(&type_name)
                                    {
                                        let field_str = field.to_string();
                                        if let Some(field_index) =
                                            class_type.field_index(&field_str)
                                        {
                                            // Stack: [value, instance_id]
                                            // SET_GENERIC_FIELD code layout: [opcode, field_index:u32]
                                            vm_debug!("DEBUG: Emitting SET_GENERIC_FIELD for field '{}' with index {} at code position {}",
                                                field_str, field_index, self.code.len());
                                            self.emit(OpCode::SET_GENERIC_FIELD);
                                            self.emit_u32(field_index as u32);
                                            vm_debug!(
                                                "DEBUG: After emit, code position = {}",
                                                self.code.len()
                                            );
                                        } else {
                                            eprintln!("Warning: Field '{}' not found in generic type '{}' (nested assignment)",
                                                field, type_name);
                                            self.emit(OpCode::SET_GENERIC_FIELD);
                                            self.emit_u32(0);
                                        }
                                    } else {
                                        eprintln!("Warning: Generic type '{}' not found in registry (assignment)", type_name);
                                        let field_str = field.to_string();
                                        let field_bytes = field_str.as_bytes().to_vec();
                                        let field_idx = self.strings.len() as u32;
                                        self.strings.push(field_bytes);
                                        self.emit(OpCode::LOAD_STR);
                                        self.code.extend_from_slice(&field_idx.to_le_bytes());
                                        self.emit(OpCode::SET_FIELD);
                                    }
                                } else {
                                    // Type not in registry, fall back to SET_FIELD
                                    let field_str = field.to_string();
                                    let field_bytes = field_str.as_bytes().to_vec();
                                    let field_idx = self.strings.len() as u32;
                                    self.strings.push(field_bytes);
                                    self.emit(OpCode::LOAD_STR);
                                    self.code.extend_from_slice(&field_idx.to_le_bytes());
                                    self.emit(OpCode::SET_FIELD);
                                }
                            } else {
                                // Now stack has: value, object_id
                                // Load field name
                                let field_str = field.to_string();
                                let field_bytes = field_str.as_bytes().to_vec();
                                let field_idx = self.strings.len() as u32;
                                self.strings.push(field_bytes);

                                self.emit(OpCode::LOAD_STR);
                                self.code.extend_from_slice(&field_idx.to_le_bytes());
                                // Emit SET_FIELD: expects value, object_id, field_name_idx
                                self.emit(OpCode::SET_FIELD);
                            }
                        }
                        // SET_GENERIC_FIELD and SET_FIELD don't push a return value -
                        // mark as void to prevent Stmt::Expr from emitting a POP
                        // that would corrupt the stack
                        self.last_expr_type = ObjectType::Void;
                        self.last_was_native_void = true;
                    } else {
                        unimplemented!("Assignment to complex LHS not supported yet");
                    }
                } else {
                    // PLAN-615 T-01: && / || 短路求值发射——RHS 仅在 LHS 真值需要时求值，
                    // 与全部转译后端一致（TS/Py/C/Rust/GD 均发射原生短路算符）。旧的急切
                    // `emit(AND/OR)` 让 `ops.len() > 0 && ops[ops.len() - 1]` 类守卫在空栈
                    // 上也求值 ops[-1]（calc 011 Equals 回归根因，Plan 550 IndexError 引爆）。
                    // 发射形态（JMP_IF_Z/NZ 弹掉的是 DUP 副本，LHS 本体留栈；末端保留
                    // AND/OR 真值归一——短路路径 = [falsy LHS, truthy 占位] 归一 false，
                    // 求值路径 = [a, b]，双路径栈平衡）：
                    //   a && b:  [a] DUP JMP_IF_Z Lshort [b] JMP Lend
                    //            Lshort: PUSH_BOOL 1   Lend: AND
                    //   a || b:  [a] DUP JMP_IF_NZ Lshort [b] JMP Lend
                    //            Lshort: PUSH_BOOL 0   Lend: OR
                    if matches!(op, Op::And | Op::Or) {
                        // PLAN-624 (P3, rev 2 裁定②): JS value 语义——短路透传
                        // 操作数值（`a && b` = a 真值 ? b : a；`a || b` = a 真值
                        // ? a : b），与 web 轨 TS/JS 对齐，使 .at 惯用法
                        // `fm && fm.title || fb` 逐字可用。此前形态：短路路径
                        // PUSH_BOOL 占位 + 末端 AND/OR 归一——非布尔操作数被
                        // 静默写成 true（jade facade 切换实机，DEBTS 064 ⑥）。
                        // 真值判定 = JMP_IF_Z/NZ 既有判定面（Plan 406 nv_truthy：
                        // tagged bool 权威，null/0/哨兵假）——不经 shim 归一：
                        // raw-i32 归一会丢 NV 栈签，且 sp 中性 CALL_NAT 不结算
                        // DUP 副本的 stake（泄漏堆引用）。
                        // 发射（双路径栈平衡；WIP 缺陷修正：POP 必须在编译 RHS
                        // 之前弹 LHS 本体——放在 RHS 之后弹掉的是 RHS 本体，
                        // 真值路径恒返回 LHS，`falsy || fb` 得 Nil 即此）：
                        //   a && b:  [a] DUP JMP_IF_Z Lshort  POP [b] JMP Lend
                        //            Lshort: (a 留栈即结果)
                        //   a || b:  [a] DUP JMP_IF_NZ Lshort  POP [b] JMP Lend
                        self.compile_expr(lhs)?;
                        self.emit(OpCode::DUP);
                        self.emit(if matches!(op, Op::And) {
                            OpCode::JMP_IF_Z
                        } else {
                            OpCode::JMP_IF_NZ
                        });
                        let short_jump = self.emit_placeholder_i16();
                        // 真值路径：弃 LHS 本体（POP 的 stake/串释放对齐旧
                        // AND/OR 末端对该槽的结算），再编译 RHS
                        self.emit(OpCode::POP);
                        self.compile_expr(rhs)?;
                        self.emit(OpCode::JMP);
                        let end_jump = self.emit_placeholder_i16();
                        // 短路路径：LHS 值留栈即为结果；真值路径 JMP 汇聚至此
                        let end_pos = self.code.len();
                        self.patch_jump_to(short_jump, end_pos);
                        self.patch_jump_to(end_jump, end_pos);
                        self.last_expr_type = self.infer_object_type(rhs.as_ref());
                        return Ok(());
                    }

                    // Plan 073 Stage A.5: Check if this is a float/double operation
                    let mut is_float = self.is_float_operation(lhs, rhs);
                    let mut is_double = self.is_double_operation(lhs, rhs);
                    let is_string = self.is_string_operation(lhs, rhs);
                    let is_u64 = self.is_u64_operation(lhs, rhs);

                    // Mixed u64 + float arithmetic: promote to double (f64 can hold all u64 values)
                    if is_u64 && is_float && !is_double {
                        is_double = true;
                        is_float = false;
                    }

                    // Normal binary operation: compile both operands, then apply operator
                    // Plan 117: Emit type coercion for mixed int/float arithmetic
                    self.compile_expr(lhs)?;
                    if is_float && !is_double && self.needs_float_coercion(lhs) {
                        self.emit(OpCode::I32_TO_F32);
                    } else if is_double && self.needs_double_coercion(lhs) {
                        if matches!(lhs.as_ref(), Expr::Float(_, _)) {
                            self.emit(OpCode::PROMOTE_F64);
                        } else if self.is_u64_expr(lhs) {
                            self.emit(OpCode::U64_TO_F64);
                        } else {
                            self.emit(OpCode::I64_TO_F64);
                        }
                    } else if is_u64 && !self.contains_u64(lhs) {
                        // Promote i32 operand to u64 for u64 arithmetic
                        self.emit(OpCode::TYPE_CAST_U64);
                    }

                    self.compile_expr(rhs)?;
                    if is_float && !is_double && self.needs_float_coercion(rhs) {
                        self.emit(OpCode::I32_TO_F32);
                    } else if is_double && self.needs_double_coercion(rhs) {
                        if matches!(rhs.as_ref(), Expr::Float(_, _)) {
                            self.emit(OpCode::PROMOTE_F64);
                        } else if self.is_u64_expr(rhs) {
                            self.emit(OpCode::U64_TO_F64);
                        } else {
                            self.emit(OpCode::I64_TO_F64);
                        }
                    } else if is_u64 && !self.contains_u64(rhs) {
                        // Promote i32 operand to u64 for u64 arithmetic
                        self.emit(OpCode::TYPE_CAST_U64);
                    }

                    // For arithmetic operations, use float/double opcodes if operands are floats
                    match op {
                        Op::Add => {
                            if is_string {
                                self.emit(OpCode::STR_CAT);
                            } else if is_double {
                                self.emit(OpCode::ADD_D);
                            } else if is_float {
                                self.emit(OpCode::ADD_F);
                            } else if is_u64 {
                                self.emit(OpCode::ADD_U64);
                            } else {
                                self.emit(OpCode::ADD);
                            }
                        }
                        Op::Sub => {
                            if is_double {
                                self.emit(OpCode::SUB_D);
                            } else if is_float {
                                self.emit(OpCode::SUB_F);
                            } else if is_u64 {
                                self.emit(OpCode::SUB_U64);
                            } else {
                                self.emit(OpCode::SUB);
                            }
                        }
                        Op::Mul => {
                            if is_double {
                                self.emit(OpCode::MUL_D);
                            } else if is_float {
                                self.emit(OpCode::MUL_F);
                            } else if is_u64 {
                                self.emit(OpCode::MUL_U64);
                            } else {
                                self.emit(OpCode::MUL);
                            }
                        }
                        Op::Div => {
                            if is_double {
                                self.emit(OpCode::DIV_D);
                            } else if is_float {
                                self.emit(OpCode::DIV_F);
                            } else if is_u64 {
                                self.emit(OpCode::DIV_U64);
                            } else {
                                self.emit(OpCode::DIV);
                            }
                        }
                        Op::Mod => {
                            if is_double {
                                self.emit(OpCode::MOD_D);
                            } else if is_float {
                                self.emit(OpCode::MOD_F);
                            } else if is_u64 {
                                self.emit(OpCode::MOD_U64);
                            } else {
                                self.emit(OpCode::MOD);
                            }
                        }
                        Op::Eq => {
                            if is_double {
                                self.emit(OpCode::EQ_D);
                            } else if is_float {
                                self.emit(OpCode::EQ_F);
                            } else if is_u64 {
                                self.emit(OpCode::EQ_U64);
                            } else {
                                self.emit(OpCode::EQ);
                            }
                        }
                        Op::Neq => {
                            if is_double {
                                self.emit(OpCode::NE_D);
                            } else if is_float {
                                self.emit(OpCode::NE_F);
                            } else if is_u64 {
                                self.emit(OpCode::NE_U64);
                            } else {
                                self.emit(OpCode::NE);
                            }
                        }
                        Op::Lt => {
                            if is_double {
                                self.emit(OpCode::LT_D);
                            } else if is_float {
                                self.emit(OpCode::LT_F);
                            } else if is_u64 {
                                self.emit(OpCode::LT_U64);
                            } else {
                                self.emit(OpCode::LT);
                            }
                        }
                        Op::Le => {
                            if is_double {
                                self.emit(OpCode::LE_D);
                            } else if is_float {
                                self.emit(OpCode::LE_F);
                            } else if is_u64 {
                                self.emit(OpCode::LE_U64);
                            } else {
                                self.emit(OpCode::LE);
                            }
                        }
                        Op::Gt => {
                            if is_double {
                                self.emit(OpCode::GT_D);
                            } else if is_float {
                                self.emit(OpCode::GT_F);
                            } else if is_u64 {
                                self.emit(OpCode::GT_U64);
                            } else {
                                self.emit(OpCode::GT);
                            }
                        }
                        Op::Ge => {
                            if is_double {
                                self.emit(OpCode::GE_D);
                            } else if is_float {
                                self.emit(OpCode::GE_F);
                            } else if is_u64 {
                                self.emit(OpCode::GE_U64);
                            } else {
                                self.emit(OpCode::GE);
                            }
                        }
                        Op::And => self.emit(OpCode::AND),
                        Op::Or => self.emit(OpCode::OR),
                        Op::Not => self.emit(OpCode::NOT),
                        _ => {
                            // Other ops (Bang, DotView, etc.) shouldn't appear in binary expressions
                        }
                    }

                    // Plan 118 Phase 4: Track result type for binary operations
                    // For arithmetic ops, result type matches operand type
                    // For comparison ops, result is always bool (Int)
                    let is_comparison =
                        matches!(op, Op::Eq | Op::Neq | Op::Lt | Op::Le | Op::Gt | Op::Ge);
                    if !is_comparison {
                        // Check operand types to determine result type
                        if is_string {
                            self.last_expr_type = ObjectType::String;
                        } else if is_double {
                            self.last_expr_type = ObjectType::Double;
                        } else if is_float {
                            self.last_expr_type = ObjectType::Float;
                        } else if is_u64 {
                            self.last_expr_type = ObjectType::Uint;
                        } else {
                            // For integer types, check if operands are Uint/Byte/U8/I8
                            // by looking at the expression types
                            let lhs_type = self.infer_object_type(lhs);
                            let rhs_type = self.infer_object_type(rhs);
                            if lhs_type == ObjectType::Uint || rhs_type == ObjectType::Uint {
                                self.last_expr_type = ObjectType::Uint;
                            } else if lhs_type == ObjectType::Byte || rhs_type == ObjectType::Byte {
                                self.last_expr_type = ObjectType::Byte;
                            } else {
                                self.last_expr_type = ObjectType::Int;
                            }
                        }
                    } else {
                        // Comparison results are always bool (Int for now)
                        self.last_expr_type = ObjectType::Int;
                    }
                }
            }
            Expr::Unary(op, rhs) => {
                // Plan 403-F Bug C: use contains_float/contains_double (which
                // also check var_types) instead of only matching literals, so
                // that `-v` where v is a float/double variable emits NEG_F/NEG_D
                // rather than the i32 NEG opcode.
                let is_float = self.contains_float(rhs);
                let is_double = self.contains_double(rhs);

                // Compile the operand first
                self.compile_expr(rhs)?;

                // Emit the appropriate unary opcode
                match op {
                    Op::Sub => {
                        if is_double {
                            self.emit(OpCode::NEG_D);
                        } else if is_float {
                            self.emit(OpCode::NEG_F);
                        } else {
                            self.emit(OpCode::NEG);
                        }
                    }
                    Op::Not => self.emit(OpCode::NOT),
                    Op::Add => { /* Unary + is a no-op */ }
                    _ => unimplemented!("Unary Op {:?}", op),
                }
            }
            _ => return self.expr_calls(expr),
        }
        Ok(())
    }
}
