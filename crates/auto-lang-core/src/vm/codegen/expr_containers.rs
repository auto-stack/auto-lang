//! PLAN-843 结构拆分：compile_expr_inner 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl Codegen {
    /// PLAN-843 尾递归派发段（自 compile_expr_inner 连续臂切片，臂体逐字保留）。
    pub(super) fn expr_containers(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            // Plan 073: Object literal support {key: val, ...}
            Expr::Object(pairs) => {
                // Evaluate each value expression (pushes values onto stack)
                for pair in pairs {
                    self.compile_expr(&pair.value)?;
                }

                // Store keys in the object_keys pool
                let keys: Vec<auto_val::ValueKey> = pairs
                    .iter()
                    .map(|pair| self.ast_key_to_value_key(&pair.key))
                    .collect();
                let key_index = self.object_keys.len() as u32;

                // Plan 073: Track field types for runtime conversion
                let types: Vec<ObjectType> = pairs
                    .iter()
                    .map(|pair| self.infer_object_type(&pair.value))
                    .collect();
                self.object_types.push(types.clone());

                self.object_keys.push(keys);

                // Emit CREATE_OBJ with key_index and field count
                let field_count = pairs.len() as u8;
                self.emit(OpCode::CREATE_OBJ);
                self.code.extend_from_slice(&key_index.to_le_bytes());
                self.code.push(field_count);
            }
            // Plan 073: Array literal support [elem1, elem2, ...]
            Expr::Array(elems) => {
                // Evaluate each element expression (pushes values onto stack)
                for elem in elems {
                    self.compile_expr(elem)?;
                }

                // Emit CREATE_ARRAY with element count
                let elem_count = elems.len() as u8;
                self.emit(OpCode::CREATE_ARRAY);
                self.code.push(elem_count);
                // Plan 445 M3：数组字面量必须设置 last_expr_type —— 此前残留
                // 前一表达式的类型（如 f-string 的 String），紧随的
                // `var out = []` 经 store 推断（last_expr_type==String →
                // StrFixed(0)）把 out 记成字符串，后续 out.push 按
                // str.push(178) 解析而静默失效（024-charts 实机 .Tick 滑窗
                // 重建列表恒空 → 重算 DivisionByZero 中止）。
                self.last_expr_type = ObjectType::Array;
            }
            // Plan 200: Tuple literal (expr1, expr2, ...)
            Expr::Tuple(elems) => {
                for elem in elems {
                    self.compile_expr(elem)?;
                }
                let elem_count = elems.len() as u8;
                self.emit(OpCode::CREATE_TUPLE);
                self.code.push(elem_count);
            }
            // Plan 073: Range expression support (0..10, 0..=10)
            Expr::Range(range) => {
                // Compile start expression (pushes onto stack)
                self.compile_expr(&range.start)?;

                // Compile end expression (pushes onto stack)
                self.compile_expr(&range.end)?;

                // Emit CREATE_RANGE or CREATE_RANGE_EQ based on range.eq
                if range.eq {
                    self.emit(OpCode::CREATE_RANGE_EQ); // Inclusive range: 0..=10
                } else {
                    self.emit(OpCode::CREATE_RANGE); // Exclusive range: 0..10
                }
            }
            // Plan 073: F-string support (f"hello $name")
            Expr::FStr(fstr) => {
                // Determine type tag for each part: 0=i32, 1=string, 2=f64, 3=f32
                let type_tags: Vec<u8> = fstr
                    .parts
                    .iter()
                    .map(|part| match self.expr_type_hint(part) {
                        FStrPartType::Int => 0,
                        FStrPartType::String => 1,
                        FStrPartType::Float64 => 2,
                        FStrPartType::Float32 => 3,
                        FStrPartType::Uint64 => 4,
                    })
                    .collect();

                // Compile each part expression (pushes values onto stack)
                for part in &fstr.parts {
                    self.compile_expr(part)?;
                }

                // Emit BUILD_FSTR with part count and type tags
                let part_count = fstr.parts.len() as u8;
                self.emit(OpCode::BUILD_FSTR);
                self.code.push(part_count);
                for &tag in &type_tags {
                    self.code.push(tag);
                }
                self.last_expr_type = ObjectType::String;
            }
            // Plan 073: Node support (for type instances like Point(10, 20))
            Expr::Node(node) => {
                // Plan 364 Step 5: Config-mode node compilation. Build the node
                // via a PUSH_ACCUM/.../POP_ACCUM container so that body Pairs
                // become props and sub-nodes become `_exprN` kids (the legacy
                // Script path can only express props — kids are hard-coded -1).
                // POP_ACCUM leaves the assembled Node id on the stack.
                if self.config_mode {
                    return self.compile_config_node(node);
                }
                // Plan 087 Phase 3: Check if this is a user-defined type instance
                let type_name = node.name.to_string();

                // Check if this is a type registered in generic_registry
                let is_registered_type = self.generic_registry.has_template(&type_name);

                if is_registered_type {
                    // Plan 087 Phase 2/3: Generic type or user-defined type instance
                    // Generate: [CONST_I32, length, NEW_INSTANCE, name_bytes..., CONSTRUCT_INSTANCE]
                    // VM: pop length, read name from code (after NEW_INSTANCE), push instance_id
                    // Note: NEW_INSTANCE does NOT push instance_id - VM will push it when executing the instruction
                    vm_debug!("DEBUG: Compiling type instance: {}", type_name);

                    // Get ClassType to determine mono_name and field count
                    let type_args = Vec::new(); // Non-generic types have empty type args
                    if let Ok(class_type) = self
                        .generic_registry
                        .get_or_create_type(&type_name, type_args)
                    {
                        let field_count = class_type.template.fields.len();
                        let mono_name = class_type.mono_name.clone();
                        vm_debug!(
                            "DEBUG: mono_name = '{}' ({} bytes)",
                            mono_name,
                            mono_name.len()
                        );
                        vm_debug!("DEBUG: mono_name bytes = {:?}", mono_name.as_bytes());
                        let name_bytes = mono_name.as_bytes();
                        let name_len = name_bytes.len();

                        // Plan 087 Phase 3: Generate bytecode in correct order
                        // CONSTRUCT_INSTANCE expects: [..., field_count, value1, value2, ..., instance_id]
                        // So we need to push field_count FIRST, then values, then instance_id

                        // Collect all field values from both args and body
                        let mut field_values = Vec::new();

                        // 1. Collect from args (if any, e.g., Counter(count: 42))
                        for arg in &node.args.args {
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    field_values.push(expr.clone());
                                }
                                crate::ast::Arg::Pair(_, expr) => {
                                    field_values.push(expr.clone());
                                }
                                crate::ast::Arg::Name(_) => {
                                    // Name-only arg - treat as nil value
                                    field_values.push(crate::ast::Expr::Nil);
                                }
                            }
                        }

                        // 2. Collect from body stmts (for Counter{count: 42} syntax)
                        // The body contains Stmt::Expr(Expr::Pair(...)) for each field
                        for stmt in &node.body.stmts {
                            if let crate::ast::Stmt::Expr(expr) = stmt {
                                if let crate::ast::Expr::Pair(pair) = expr {
                                    // Extract the value from the pair
                                    field_values.push(pair.value.as_ref().clone());
                                }
                            }
                        }

                        vm_debug!("DEBUG codegen: field_count = {}, collected {} field values from args ({}), body ({})",
                            field_count,
                            field_values.len(),
                            node.args.args.len(),
                            node.body.stmts.len()
                        );

                        // Plan 087 Phase 3: Generate bytecode in correct order
                        // CONSTRUCT_INSTANCE pops: instance_id, then field_count, then field_count values
                        // So the stack should be: [..., value1, value2, ..., valueN, instance_id, field_count]

                        // 1. Compile each field value expression (pushes values onto stack)
                        // Stack: ..., value1, value2, ..., valueN
                        let field_defs = class_type.fields();
                        // EDGE-04-B: `Type{}` 空字面量(无 args/body)时 field_values
                        // 不足 field_count,CONSTRUCT_INSTANCE 会从栈上 pop 垃圾值 →
                        // 字段是无效 heap id(Invalid instance ID: 0)。补类型默认值。
                        while field_values.len() < field_count {
                            let default_expr = field_defs
                                .get(field_values.len())
                                .map(|fd| default_expr_for_type(&fd.field_type))
                                .unwrap_or(crate::ast::Expr::Nil);
                            field_values.push(default_expr);
                        }
                        for (i, value_expr) in field_values.iter().enumerate() {
                            vm_debug!("DEBUG codegen: Compiling field value {}", i);
                            self.compile_expr(value_expr)?;
                            // Plan 230: promote f32 -> f64 when field type is Double
                            if let Some(field_def) = field_defs.get(i) {
                                if matches!(field_def.field_type, Type::Double)
                                    && self.last_expr_type == ObjectType::Float
                                {
                                    self.emit(OpCode::PROMOTE_F64);
                                }
                            }
                            vm_debug!(
                                "DEBUG codegen: code.len() = 0x{:04x} after field value {}",
                                self.code.len(),
                                i
                            );
                        }

                        // 2. Push mono_name length onto stack (for NEW_INSTANCE to pop)
                        self.emit(OpCode::CONST_I32);
                        self.emit_u32(name_len as u32);

                        // 3. Emit NEW_INSTANCE instruction
                        // VM will pop length, read name_bytes from code, push instance_id
                        // Stack after: [..., value1, value2, ..., valueN, instance_id]
                        self.emit(OpCode::NEW_INSTANCE);

                        // 4. Emit mono_name bytes directly into code (AFTER NEW_INSTANCE instruction)
                        for &byte in name_bytes {
                            self.code.push(byte);
                        }

                        // 5. Push field_count (for CONSTRUCT_INSTANCE)
                        // Stack: [..., value1, value2, ..., valueN, instance_id, field_count]
                        self.emit(OpCode::CONST_I32);
                        self.emit_u32(field_count as u32);

                        // 6. Emit CONSTRUCT_INSTANCE
                        // Stack layout: [..., value1, value2, ..., valueN, instance_id, field_count]
                        self.emit(OpCode::CONSTRUCT_INSTANCE);
                        vm_debug!(
                            "DEBUG codegen: code.len() = 0x{:04x} after CONSTRUCT_INSTANCE",
                            self.code.len()
                        );
                    } else {
                        eprintln!(
                            "Warning: Failed to get/create ClassType for '{}'",
                            type_name
                        );
                        // Fallback to CREATE_OBJ (regular object)
                        let member_names = ["count".to_string()]; // Fallback

                        // Compile args
                        let arg_count = node.num_args as u8;
                        for arg in &node.args.args {
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Pair(_, expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Name(_) => {
                                    self.emit(OpCode::CONST_0);
                                }
                            }
                        }

                        // Create object keys
                        let keys: Vec<auto_val::ValueKey> = member_names
                            .iter()
                            .take(arg_count as usize)
                            .map(|name| auto_val::ValueKey::Str(name.clone().into()))
                            .collect();
                        let key_index = self.object_keys.len() as u32;
                        self.object_keys.push(keys);

                        // Emit CREATE_OBJ
                        let field_count = arg_count.min(member_names.len() as u8);
                        self.emit(OpCode::CREATE_OBJ);
                        self.code.extend_from_slice(&key_index.to_le_bytes());
                        self.code.push(field_count);
                    }
                } else if let Some(type_info) = self.get_type(&type_name) {
                    // This is a type instance! Generate object creation instead of node
                    // Example: Point(10, 20) -> object with x: 10, y: 20

                    // Clone type_info to avoid holding immutable borrow
                    let member_names = type_info.member_names.clone();

                    // Compile each argument expression (pushes values onto stack)
                    let arg_count = node.num_args as u8;
                    for arg in &node.args.args {
                        match arg {
                            crate::ast::Arg::Pos(expr) => {
                                self.compile_expr(expr)?;
                            }
                            crate::ast::Arg::Pair(_key, expr) => {
                                // For named args, compile the value
                                self.compile_expr(expr)?;
                            }
                            crate::ast::Arg::Name(_) => {
                                // Name-only arg (placeholder for future)
                            }
                        }
                    }

                    // Create object keys using type member names
                    // Positional args map to type members in order
                    let keys: Vec<auto_val::ValueKey> = member_names
                        .iter()
                        .take(arg_count as usize) // Only take as many as we have args
                        .map(|name| auto_val::ValueKey::Str(name.clone().into()))
                        .collect();

                    // Register keys in object_keys pool
                    let key_index = self.object_keys.len() as u32;
                    self.object_keys.push(keys);

                    // Plan 087: Infer field types from node args
                    // For Node instances like Point{x: 1, y: 2}, infer types from args
                    let types: Vec<ObjectType> = node
                        .args
                        .args
                        .iter()
                        .take(arg_count as usize)
                        .map(|arg| {
                            match arg {
                                crate::ast::Arg::Pos(expr) => self.infer_object_type(expr),
                                crate::ast::Arg::Pair(_, expr) => self.infer_object_type(expr),
                                crate::ast::Arg::Name(_) => {
                                    ObjectType::Int // Default to Int
                                }
                            }
                        })
                        .collect();

                    // Register types in object_types pool
                    self.object_types.push(types);

                    // Emit CREATE_OBJ instead of CREATE_NODE
                    let field_count = arg_count.min(member_names.len() as u8);
                    self.emit(OpCode::CREATE_OBJ);
                    self.code.extend_from_slice(&key_index.to_le_bytes());
                    self.code.push(field_count);
                } else {
                    // Not a type - create generic Node
                    // Compile node name as string
                    let name_bytes = node.name.as_bytes().to_vec();
                    let name_idx = self.strings.len() as u32;
                    self.strings.push(name_bytes);

                    // Compile each argument expression (pushes values onto stack)
                    let arg_count = node.num_args as u8;
                    for arg in &node.args.args {
                        match arg {
                            crate::ast::Arg::Pos(expr) => {
                                self.compile_expr(expr)?;
                            }
                            crate::ast::Arg::Pair(_key, expr) => {
                                // For named args, compile the value
                                self.compile_expr(expr)?;
                            }
                            crate::ast::Arg::Name(_) => {
                                // Name-only arg (placeholder for future)
                            }
                        }
                    }

                    // Compile node body into props object (Plan 073)
                    let mut keys = Vec::new();
                    let mut types = Vec::new();
                    let mut prop_count = 0;

                    for stmt in &node.body.stmts {
                        if let crate::ast::Stmt::Expr(expr) = stmt {
                            if let crate::ast::Expr::Pair(pair) = expr {
                                let key_str = match &pair.key {
                                    crate::ast::Key::NamedKey(name) => name.to_string(),
                                    crate::ast::Key::StrKey(s) => s.to_string(),
                                    _ => format!("_prop{}", prop_count),
                                };
                                keys.push(auto_val::ValueKey::Str(key_str.into()));

                                self.compile_expr(&pair.value)?;
                                types.push(self.infer_object_type(&pair.value));
                                prop_count += 1;
                            }
                        } else if let crate::ast::Stmt::Store(store) = stmt {
                            let key_str = store.name.to_string();
                            keys.push(auto_val::ValueKey::Str(key_str.into()));

                            self.compile_expr(&store.expr)?;
                            types.push(self.infer_object_type(&store.expr));
                            prop_count += 1;
                        }
                    }

                    if prop_count > 0 {
                        let key_index = self.object_keys.len() as u32;
                        self.object_keys.push(keys);
                        self.object_types.push(types);

                        self.emit(OpCode::CREATE_OBJ);
                        self.code.extend_from_slice(&key_index.to_le_bytes());
                        self.code.push(prop_count as u8);
                    } else {
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(-1); // props_id
                    }

                    // For now, use -1 for kids_id
                    self.emit(OpCode::CONST_I32);
                    self.emit_i32(-1); // kids_id

                    // Compile node.id (if present) as string index, or 0xFFFF if absent
                    let id_idx = if !node.id.is_empty() {
                        let id_bytes = node.id.as_bytes().to_vec();
                        let idx = self.strings.len() as u32;
                        self.strings.push(id_bytes);
                        idx
                    } else {
                        0xFFFF // sentinel: no id
                    };

                    // Emit CREATE_NODE with name index, arg count, and id index
                    self.emit(OpCode::CREATE_NODE);
                    self.code.extend_from_slice(&name_idx.to_le_bytes());
                    self.code.push(arg_count);
                    self.code.extend_from_slice(&id_idx.to_le_bytes());
                }
            }
            _ => return self.expr_ident(expr),
        }
        Ok(())
    }
}
