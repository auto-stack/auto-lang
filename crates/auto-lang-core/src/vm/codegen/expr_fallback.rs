//! PLAN-843 结构拆分：compile_expr_inner 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl Codegen {
    /// PLAN-843 尾递归派发段（自 compile_expr_inner 连续臂切片，臂体逐字保留）。
    pub(super) fn expr_fallback(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            // Cover expression (enum variant pattern like Color.Red or ContentBlock.Text(expr))
            Expr::Cover(crate::ast::Cover::Tag(tag_cover)) => {
                let key = format!("{}.{}", tag_cover.kind, tag_cover.tag);

                // Check if this is a scalar enum variant (no payload)
                if let Some(&value) = self.enum_values.get(&key) {
                    self.emit(OpCode::CONST_I32);
                    self.emit_i32(value);
                } else if self.generic_registry.has_template(&key) {
                    // Data-carrying variant construction: variant with fields
                    // e.g., ContentBlock.Text("hello") -> NEW_INSTANCE + CONSTRUCT_INSTANCE
                    let has_bindings = tag_cover.bindings.iter().any(|b| b.as_str() != "_");

                    if has_bindings {
                        // Get ClassType for field info
                        if let Ok(class_type) =
                            self.generic_registry.get_or_create_type(&key, Vec::new())
                        {
                            let field_count = class_type.template.fields.len();
                            let mono_name = class_type.mono_name.clone();
                            let name_bytes = mono_name.as_bytes();
                            let name_len = name_bytes.len();

                            // Compile each binding value expression (pushes onto stack)
                            for binding in &tag_cover.bindings {
                                // Bindings are variable names referencing outer scope values
                                self.compile_expr(&crate::ast::Expr::Ident(binding.clone()))?;
                            }

                            // NEW_INSTANCE: push name_len, then opcode + name bytes
                            self.emit(OpCode::CONST_I32);
                            self.emit_u32(name_len as u32);
                            self.emit(OpCode::NEW_INSTANCE);
                            for &byte in name_bytes {
                                self.code.push(byte);
                            }

                            // CONSTRUCT_INSTANCE: field_count + opcode
                            self.emit(OpCode::CONST_I32);
                            self.emit_u32(field_count as u32);
                            self.emit(OpCode::CONSTRUCT_INSTANCE);
                        } else {
                            return Err(AutoError::Msg(format!(
                                "Failed to create type for variant: {}",
                                key
                            )));
                        }
                    } else {
                        // Unit variant with no payload but registered as data type
                        // Just create an instance with 0 fields
                        if let Ok(class_type) =
                            self.generic_registry.get_or_create_type(&key, Vec::new())
                        {
                            let mono_name = class_type.mono_name.clone();
                            let name_bytes = mono_name.as_bytes();
                            let name_len = name_bytes.len();

                            self.emit(OpCode::CONST_I32);
                            self.emit_u32(name_len as u32);
                            self.emit(OpCode::NEW_INSTANCE);
                            for &byte in name_bytes {
                                self.code.push(byte);
                            }

                            self.emit(OpCode::CONST_I32);
                            self.emit_u32(0u32);
                            self.emit(OpCode::CONSTRUCT_INSTANCE);
                        } else {
                            return Err(AutoError::Msg(format!(
                                "Failed to create type for variant: {}",
                                key
                            )));
                        }
                    }
                } else {
                    return Err(AutoError::Msg(format!("Unknown enum variant: {}", key)));
                }
            }
            // Plan 223: is-expression as value
            Expr::Is(is_expr) => {
                // Evaluate target expression once and store in temp variable
                self.compile_expr(&is_expr.target)?;
                let target_var = self.add_var("_is_expr_target");
                self.emit_store_loc(target_var);

                let mut end_jumps = Vec::new();

                // Process each branch — same pattern matching as Stmt::Is
                // but the body's last expression stays on stack as the is-expression's value
                for branch in &is_expr.branches {
                    match branch {
                        crate::ast::IsBranch::EqBranch(patterns, body) => {
                            let pattern = &patterns[0];
                            match pattern {
                                crate::ast::Expr::StructPattern(sc) if sc.variant.is_none() => {
                                    // Plan 525 W2: is-struct 解构臂(表达式位)——
                                    // 同语句位:恒命中+逐绑定解构;体末值留栈
                                    // (is 表达式的值通道)
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
                                    let name_bytes = b"Option.None";
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
                                            let name_bytes = b"Option.Some";
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
                                                        .infer_option_inner_type(&is_expr.target);
                                                    self.var_types
                                                        .insert(binding.to_string(), inner_type);
                                                }
                                            }

                                            // Compile body — last expr value stays on stack
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
                                            let name_bytes = b"Option.None";
                                            self.emit_u16(name_bytes.len() as u16);
                                            for &byte in name_bytes {
                                                self.code.push(byte);
                                            }
                                        }
                                    }
                                }
                                crate::ast::Expr::Some(_inner) => {
                                    self.emit_load_loc(target_var);
                                    self.emit(OpCode::IS_VARIANT);
                                    let name_bytes = b"Option.Some";
                                    self.emit_u16(name_bytes.len() as u16);
                                    for &byte in name_bytes {
                                        self.code.push(byte);
                                    }
                                }
                                crate::ast::Expr::Ok(_inner) => {
                                    self.emit_load_loc(target_var);
                                    self.emit(OpCode::IS_VARIANT);
                                    let name_bytes = b"Result.Ok";
                                    self.emit_u16(name_bytes.len() as u16);
                                    for &byte in name_bytes {
                                        self.code.push(byte);
                                    }
                                }
                                crate::ast::Expr::Err(_msg) => {
                                    self.emit_load_loc(target_var);
                                    self.emit(OpCode::IS_VARIANT);
                                    let name_bytes = b"Result.Err";
                                    self.emit_u16(name_bytes.len() as u16);
                                    for &byte in name_bytes {
                                        self.code.push(byte);
                                    }
                                }
                                // Cover::Tag destructuring for data-carrying enum variants
                                crate::ast::Expr::Cover(crate::ast::Cover::Tag(tag_cover)) => {
                                    let variant_mono =
                                        format!("{}.{}", tag_cover.kind, tag_cover.tag);
                                    // Plan 442 C2: unknown variants (use.rust enums) — same IS_VARIANT
                                    // routing as the Stmt::Is arm above.
                                    // Plan 447 H1 fix: Same multi-pattern handling fix as Stmt::Is.
                                    let known_scalar = self.enum_values.contains_key(&variant_mono);
                                    let has_data_payload =
                                        self.generic_registry.has_template(&variant_mono)
                                            || !known_scalar;

                                    if has_data_payload
                                        && tag_cover.bindings.iter().any(|b| b.as_str() != "_")
                                    {
                                        self.emit_load_loc(target_var);
                                        self.emit(OpCode::IS_VARIANT);
                                        let name_bytes = variant_mono.as_bytes();
                                        self.emit_u16(name_bytes.len() as u16);
                                        for &byte in name_bytes {
                                            self.code.push(byte);
                                        }

                                        self.emit(OpCode::JMP_IF_Z);
                                        let jump_to_next = self.emit_placeholder_i16();

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
                                            (1, vec![])
                                        };

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

                                        // Compile body — last expr value stays on stack
                                        self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                                        self.emit(OpCode::JMP);
                                        let jump_to_end = self.emit_placeholder_i16();
                                        end_jumps.push(jump_to_end);

                                        self.patch_jump(jump_to_next);
                                        continue;
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
                                        // Plan 447 H1: scalar enum multi-pattern arm —
                                        // per-pattern EQ + short-circuit OR via the shared
                                        // emit_is_or_arm helper (keeps Expr::Is in lockstep
                                        // with Stmt::Is; formerly emitted nothing here).
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
                                    // Plan 325: 通配符 `_` —— 总匹配（catch-all）。
                                    let is_wildcard = patterns.len() == 1
                                        && matches!(&patterns[0], crate::ast::Expr::Ident(n) if n.as_str() == "_");
                                    if is_wildcard {
                                        // 总匹配：body 值留栈，跳到 is 末尾（表达式上下文）
                                        self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;
                                        self.emit(OpCode::JMP);
                                        let jump_to_end = self.emit_placeholder_i16();
                                        end_jumps.push(jump_to_end);
                                        continue;
                                    }
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
                                        continue;
                                    }
                                }
                            }

                            // Jump to next branch if not matched
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_next = self.emit_placeholder_i16();

                            // Compile body — last expr value stays on stack
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            end_jumps.push(jump_to_end);

                            self.patch_jump(jump_to_next);
                        }
                        crate::ast::IsBranch::IfBranch(condition, body) => {
                            self.compile_expr(condition)?;

                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_next = self.emit_placeholder_i16();

                            // Compile body — last expr value stays on stack
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            end_jumps.push(jump_to_end);

                            self.patch_jump(jump_to_next);
                        }
                        crate::ast::IsBranch::ElseBranch(body) => {
                            // Compile body — last expr value stays on stack
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

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
            Expr::Block(body) => {
                self.push_scope();
                let n = body.stmts.len();
                for (i, s) in body.stmts.iter().enumerate() {
                    if i < body.source_lines.len() {
                        self.emit_source_line(body.source_lines[i]);
                    }
                    let is_last = i == n - 1;
                    let old_pop = self.should_pop_expr_result;
                    self.should_pop_expr_result = !is_last;
                    self.compile_stmt(s)?;
                    self.should_pop_expr_result = old_pop;
                }
                self.pop_scope();
                // Block value = last stmt result (already on stack if is_last)
            }
            Expr::Lambda(_) => {
                // Lambda (FnKind::Lambda) — treat as closure-like: compile as no-op stub
                // Full lambda support requires dedicated opcodes; for now emit a null placeholder
                self.emit(OpCode::CONST_I32);
                self.emit_i32(0);
                self.last_expr_type = ObjectType::Int;
            }
            Expr::TupleDestruct { names, expr } => {
                // let (a, b) = expr — compile as: evaluate expr, then extract elements
                self.compile_expr(expr)?;
                // The result is on the stack — assume it's a list/array-like object
                // Store it in a temp variable
                let temp_idx = self.add_var("_tuple_destruct");
                self.emit_store_loc(temp_idx);

                // Extract each element and bind to variable
                for (i, name) in names.iter().enumerate() {
                    self.emit_load_loc(temp_idx);
                    self.emit(OpCode::CONST_I32);
                    self.emit_i32(i as i32);
                    self.emit(OpCode::GET_ELEM);
                    let var_idx = self.add_var(name.as_str());
                    self.emit_store_loc(var_idx);
                }
                // Push null as the expression result
                self.emit(OpCode::CONST_I32);
                self.emit_i32(0);
            }
            Expr::GenName(name) => {
                // Plan 317: Generic type name as expression (e.g. List<Note>).
                // This appears as the receiver of List<Note>.new(...). In that
                // case the Dot/Call path handles it (5493 GenName arm). If it
                // reaches compile_expr directly, it's a type reference with no
                // runtime value — push a placeholder 0 (shouldn't happen in
                // correct code; the method-call path intercepts it first).
                let _ = name;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(0);
            }
            _ => {
                unimplemented!("Expression {:?}", expr);
            }
        }
        Ok(())
    }
}
