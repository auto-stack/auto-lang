//! PLAN-843 结构拆分：compile_expr_inner 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl Codegen {
    /// PLAN-843 尾递归派发段（自 compile_expr_inner 连续臂切片，臂体逐字保留）。
    pub(super) fn expr_dot_index(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            // Plan 073: Dot expression field access (obj.field)
            // Plan 087 Phase 2: Support generic instance field access
            Expr::Dot(obj, field) => {
                // Plan 123: Check if this is enum variant access (e.g., Color.Red)
                if let Expr::Ident(type_name) = obj.as_ref() {
                    // Check codegen's own enum_values first (populated during EnumDecl compilation)
                    let key = format!("{}.{}", type_name, field);
                    if let Some(&value) = self.enum_values.get(&key) {
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(value);
                        return Ok(());
                    }
                    // Also check type_store (populated during type resolution)
                    let variant_value = self
                        .type_store
                        .read()
                        .unwrap()
                        .get_enum_variant_value(type_name.as_ref(), field.as_ref());
                    if let Some(value) = variant_value {
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(value);
                        return Ok(());
                    }
                }

                // Qualified enum variant access (e.g., types.SectionType.Goals)
                // This is Dot(Dot(Ident("types"), "SectionType"), "Goals")
                if let Expr::Dot(inner, type_field) = obj.as_ref() {
                    if let Expr::Ident(_mod_name) = inner.as_ref() {
                        let key = format!("{}.{}", type_field, field);
                        if let Some(&value) = self.enum_values.get(&key) {
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(value);
                            return Ok(());
                        }
                        let variant_value = self
                            .type_store
                            .read()
                            .unwrap()
                            .get_enum_variant_value(type_field.as_ref(), field.as_ref());
                        if let Some(value) = variant_value {
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(value);
                            return Ok(());
                        }
                    }
                }

                // Plan 200: Check if this is tuple field access (t.0, t.1, ...)
                // Only treat as tuple access if the object is known to be a tuple type,
                // otherwise numeric fields like a.3 should be object field access
                let is_tuple_field = field.as_str().parse::<usize>().is_ok()
                    && if let Expr::Ident(var_name) = obj.as_ref() {
                        self.var_types
                            .get(var_name.as_ref())
                            .map(|ty| matches!(ty, Type::Tuple(_)))
                            .unwrap_or(false)
                    } else {
                        false
                    };
                if is_tuple_field {
                    self.compile_expr(obj)?;
                    let field_index: u8 = field.as_str().parse().unwrap();
                    self.emit(OpCode::GET_TUPLE_FIELD);
                    self.code.push(field_index);
                    return Ok(());
                }

                // Check if this is the .type property - returns type name as string
                // PLAN-066 T-04（KD-057① 根修，auto-musk PLAN-066 上游消费）：
                // typeof 语义仅保留原始类型接收者。对象接收者（User/
                // GenericInstance/obj 字面量/JSON 产物等动态值）一律落普通
                // 字段访问通道——与 web 轨 a2ts 纯属性访问同语义。此前对
                // field=="type" 无条件抢占为编译期类型名 LOAD_STR：任何名为
                // type 的字段读取在 VM 轨恒返回接收者推断型名（JSON.parse
                // 推断型 str → j.type == "str"），musk questionnaireFor 的
                // json.type 判定、forge_store 的 ev.type 分派等约 90 处字段
                // 比较恒假。M3 语料门（corpus_m3 t01/t03/t05/t06）的 .type
                // 接收者均为原始类型字面量/原始型 fn 返回/数组元素，本守卫
                // 下语义不变。
                if field.as_str() == "type" {
                    let ty = self.infer_expr_type(obj);
                    // 诊断口（沿 AUTO_DEBUG_GETFIELD 先例，env 门控零行为变更）。
                    if std::env::var_os("AUTO_DEBUG_TYPEPROP").is_some() {
                        let prim = match &ty {
                            Type::StrFixed(n) => *n > 0,
                            Type::Byte
                            | Type::Int
                            | Type::Uint
                            | Type::USize
                            | Type::I64
                            | Type::U64
                            | Type::Float
                            | Type::Double
                            | Type::Bool
                            | Type::Char
                            | Type::CStrLit
                            | Type::StrSlice
                            | Type::StrOwned => true,
                            _ => false,
                        };
                        eprintln!("[TYPEPROP] obj={:?} ty={:?} prim={}", obj, ty, prim);
                    }
                    // StrFixed(0) 是动态值推断哨兵（真实 str 字面量推断带长度
                    // 的 StrFixed(n)，n=len；Plan 212 注记在案 StrFixed(0) 曾
                    // 为错误推断源）——排除之，落字段访问通道。
                    let is_primitive_receiver = match &ty {
                        Type::StrFixed(n) => *n > 0,
                        Type::Byte
                        | Type::Int
                        | Type::Uint
                        | Type::USize
                        | Type::I64
                        | Type::U64
                        | Type::Float
                        | Type::Double
                        | Type::Bool
                        | Type::Char
                        | Type::CStrLit
                        | Type::StrSlice
                        | Type::StrOwned => true,
                        _ => false,
                    };
                    if is_primitive_receiver {
                        // Get type name as string
                        let type_name = ty.unique_name();
                        // Add to string pool
                        let type_bytes = type_name.to_string().into_bytes();
                        let str_idx = self.strings.len() as u32;
                        self.strings.push(type_bytes);
                        // Emit LOAD_STR instruction
                        self.emit(OpCode::LOAD_STR);
                        self.code.extend_from_slice(&str_idx.to_le_bytes());
                        self.last_expr_type = ObjectType::String;
                        vm_debug!(
                            "DEBUG: .type property: obj={:?}, type_name={}",
                            obj,
                            type_name
                        );
                        return Ok(());
                    }
                    // 对象/未知接收者：不抢占，落下方字段访问通道（GET_FIELD）。
                }

                // Plan 087 Phase 3: Check if this is field access on a user-defined type instance
                let is_user_type_instance = if let Expr::Ident(var_name) = obj.as_ref() {
                    // Look up variable type
                    if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                        matches!(var_type, Type::User(_))
                    } else {
                        false
                    }
                } else {
                    false
                };

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

                if is_user_type_instance || is_generic_instance {
                    vm_debug!(
                        "DEBUG: Compiling field access: obj={:?}, field={}",
                        obj,
                        field
                    );
                    // Plan 087 Phase 2/3: Generic instance or user-defined type field access
                    // Compile object expression (pushes instance_id onto stack)
                    self.compile_expr(obj)?;

                    // Get field index from type registry
                    if let Expr::Ident(var_name) = obj.as_ref() {
                        if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                            let type_name = match var_type {
                                Type::User(type_decl) => type_decl.name.to_string(),
                                Type::GenericInstance(inst) => {
                                    // Generate mono_name from base_name and args
                                    self.generic_registry
                                        .get_template(&inst.base_name.to_string())
                                        .map(|t| t.mono_name_from_args(&inst.args))
                                        .unwrap_or_else(|| format!("{}_unknown", inst.base_name))
                                }
                                _ => var_name.to_string(),
                            };

                            vm_debug!(
                                "DEBUG: Looking up type '{}' for field '{}'",
                                type_name,
                                field
                            );
                            // Get ClassType to find field index
                            // For non-generic user types (only registered for field lookup),
                            // use GET_FIELD because instances are created with CREATE_OBJ
                            let is_generic = self
                                .generic_registry
                                .get_template(&type_name)
                                .map(|t| !t.generic_params.is_empty())
                                .unwrap_or(false);
                            if is_generic {
                                if let Some(class_type) = self.generic_registry.get_type(&type_name)
                                {
                                    let field_str = field.to_string();
                                    if let Some(field_index) = class_type.field_index(&field_str) {
                                        vm_debug!(
                                            "DEBUG: Field '{}' index = {}",
                                            field,
                                            field_index
                                        );
                                        // Emit GET_GENERIC_FIELD with field index
                                        self.emit(OpCode::GET_GENERIC_FIELD);
                                        self.emit_u32(field_index as u32);

                                        // Plan 118 Phase 7: Set last_expr_type based on field type
                                        if let Some(field_type) = class_type.field_type(&field_str)
                                        {
                                            self.last_expr_type =
                                                self.type_to_object_type(&field_type);
                                            vm_debug!("DEBUG: Field '{}' type = {:?}, last_expr_type = {:?}",
                                                field, field_type, self.last_expr_type);
                                        }
                                    } else {
                                        eprintln!(
                                            "Warning: Field '{}' not found in type '{}'",
                                            field, type_name
                                        );
                                        self.emit(OpCode::GET_GENERIC_FIELD);
                                        self.emit_u32(0);
                                    }
                                } else {
                                    eprintln!(
                                        "Warning: Type '{}' not found in registry",
                                        type_name
                                    );
                                    self.emit(OpCode::GET_FIELD);
                                    let field_str = field.to_string();
                                    let field_bytes = field_str.as_bytes().to_vec();
                                    let field_idx = self.strings.len() as u32;
                                    self.strings.push(field_bytes);
                                    self.emit_u32(field_idx);
                                }
                            } else {
                                // Non-generic user type (SseParser, etc.): use GET_FIELD (name-based)
                                self.emit(OpCode::GET_FIELD);
                                let field_str = field.to_string();
                                let field_bytes = field_str.as_bytes().to_vec();
                                let field_idx = self.strings.len() as u32;
                                self.strings.push(field_bytes);
                                self.emit_u32(field_idx);

                                // Plan 118 Phase 7: Set last_expr_type from the field's
                                // declared type, so bool/float/char/etc. fields format
                                // correctly at the REPL/last-expression boundary. Non-generic
                                // types are also registered in generic_registry (see
                                // register_type), so we can reuse the same lookup as the
                                // generic branch above.
                                if let Some(class_type) = self.generic_registry.get_type(&type_name)
                                {
                                    if let Some(field_type) = class_type.field_type(&field_str) {
                                        self.last_expr_type = self.type_to_object_type(&field_type);
                                        vm_debug!(
                                            "DEBUG: Field '{}' type = {:?}, last_expr_type = {:?}",
                                            field,
                                            field_type,
                                            self.last_expr_type
                                        );
                                    }
                                }
                            }
                        } else {
                            // Fallback: type not found in var_types, use GET_FIELD
                            self.emit(OpCode::GET_FIELD);
                            let field_str = field.to_string();
                            let field_bytes = field_str.as_bytes().to_vec();
                            let field_idx = self.strings.len() as u32;
                            self.strings.push(field_bytes);
                            self.emit_u32(field_idx);
                        }
                    }
                } else {
                    // Check if this is a built-in type method access (e.g., text.len, arr.len)
                    // Compiled as zero-arg native calls instead of GET_FIELD
                    let builtin_native_id = if let Expr::Ident(var_name) = obj.as_ref() {
                        if let Some(type_name) = self.infer_type_from_var(var_name.as_ref()) {
                            let native_name = format!("{}.{}", type_name, field);
                            BIGVM_NATIVES
                                .lock()
                                .unwrap()
                                .resolve_qualified(&native_name)
                        } else {
                            None
                        }
                    } else {
                        let inferred_type = self.infer_object_type(obj.as_ref());
                        let type_name = match inferred_type {
                            ObjectType::String => "str",
                            ObjectType::Array => "List",
                            ObjectType::Int => "int",
                            ObjectType::Float | ObjectType::Double => "float",
                            ObjectType::Bool => "bool",
                            ObjectType::Char => "char",
                            _ => "",
                        };
                        if type_name.is_empty() {
                            None
                        } else {
                            let native_name = format!("{}.{}", type_name, field);
                            BIGVM_NATIVES
                                .lock()
                                .unwrap()
                                .resolve_qualified(&native_name)
                        }
                    };

                    if let Some(native_id) = builtin_native_id {
                        // Compile object expression, then call the native method with zero args
                        self.compile_expr(obj)?;
                        self.emit(OpCode::CALL_NAT);
                        self.emit_u16(native_id);
                        self.last_expr_type = ObjectType::Int;
                        return Ok(());
                    }

                    // plan-022 (auto-down): `.length` on an untyped receiver —
                    // call-result chains (`bl.links.length`) where static
                    // inference has no type and the alias lookup above misses.
                    // Emit ARRAY_LEN directly: its runtime handles heap lists
                    // and strings; on non-list objects it yields 0 (matching
                    // the prior fallthrough's behavior — no regression), and
                    // user-typed receivers never reach this branch.
                    if field.as_str() == "length" {
                        self.compile_expr(obj)?;
                        self.emit(OpCode::ARRAY_LEN);
                        self.last_expr_type = ObjectType::Int;
                        return Ok(());
                    }

                    // Regular field access (Plan 073)
                    // Or nested field access on user type (Plan 118 Phase 7)
                    // Compile object expression (should push object_id onto stack)
                    self.compile_expr(obj)?;

                    // Plan 118 Phase 7: Check if the result is a heap object (VmRef)
                    // If last_expr_type is NestedObject, we should use GET_GENERIC_FIELD
                    let is_heap_object = self.last_expr_type == ObjectType::NestedObject;

                    if is_heap_object {
                        // For heap objects (user type instances), use GET_GENERIC_FIELD
                        // Need to get type info from infer_expr_type
                        let obj_expr_type = self.infer_expr_type(obj);
                        let type_name = match &obj_expr_type {
                            Type::User(type_decl) => type_decl.name.to_string(),
                            Type::GenericInstance(inst) => self
                                .generic_registry
                                .get_template(&inst.base_name.to_string())
                                .map(|t| t.mono_name_from_args(&inst.args))
                                .unwrap_or_else(|| format!("{}_unknown", inst.base_name)),
                            _ => "Unknown".to_string(),
                        };

                        if let Some(class_type) = self.generic_registry.get_type(&type_name) {
                            let field_str = field.to_string();
                            if let Some(field_index) = class_type.field_index(&field_str) {
                                self.emit(OpCode::GET_GENERIC_FIELD);
                                self.emit_u32(field_index as u32);

                                // Set last_expr_type based on field type
                                if let Some(field_type) = class_type.field_type(&field_str) {
                                    self.last_expr_type = self.type_to_object_type(&field_type);
                                }
                            } else {
                                eprintln!(
                                    "Warning: Field '{}' not found in type '{}' (nested access)",
                                    field, type_name
                                );
                                self.emit(OpCode::GET_GENERIC_FIELD);
                                self.emit_u32(0);
                            }
                        } else {
                            // Type not in registry, fall back to GET_FIELD
                            let field_str = field.to_string();
                            let field_bytes = field_str.as_bytes().to_vec();
                            let field_idx = self.strings.len() as u32;
                            self.strings.push(field_bytes);
                            self.emit(OpCode::GET_FIELD);
                            self.code.extend_from_slice(&field_idx.to_le_bytes());
                        }
                    } else {
                        // Add field name to string pool and emit GET_FIELD <field_idx>
                        let field_str = field.to_string();
                        let field_bytes = field_str.as_bytes().to_vec();
                        let field_idx = self.strings.len() as u32;
                        self.strings.push(field_bytes);

                        self.emit(OpCode::GET_FIELD);
                        self.code.extend_from_slice(&field_idx.to_le_bytes());
                    }
                }
            }
            // Plan 073: Array indexing (arr[index])
            // Plan 118 Phase 4: Also supports string indexing (str[index] -> char)
            Expr::Index(arr, idx) => {
                // Check for range slice: arr[start..end], arr[..end], arr[start..]
                if let Expr::Range(range) = idx.as_ref() {
                    // Compile container
                    self.compile_expr(arr)?;
                    // Compile start (push -1 if Nil = from beginning)
                    if matches!(range.start.as_ref(), Expr::Nil) {
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(-1);
                    } else {
                        self.compile_expr(&range.start)?;
                    }
                    // Compile end (push -1 if Nil = to end)
                    if matches!(range.end.as_ref(), Expr::Nil) {
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(-1);
                    } else {
                        self.compile_expr(&range.end)?;
                    }
                    self.emit(OpCode::SLICE);
                    self.last_expr_type = ObjectType::String; // default: string slice
                    return Ok(());
                }
                // Normal index: compile array/string expression
                self.compile_expr(arr)?;
                // Compile index expression (should push index onto stack)
                self.compile_expr(idx)?;
                // Emit GET_ELEM (pops array_id/str_idx and index, pushes element value)
                self.emit(OpCode::GET_ELEM);

                // Plan 118 Phase 4: Set last_expr_type based on array element type or string char
                // Check if indexing a string literal
                if let Expr::Str(_) = arr.as_ref() {
                    // String indexing returns a character
                    self.last_expr_type = ObjectType::Char;
                } else {
                    // For arrays, try to infer element type
                    let arr_type = self.infer_object_type(arr);
                    match arr_type {
                        ObjectType::Array => {
                            // Use infer_expr_type to get the actual element type
                            let elem_type = self.infer_expr_type(expr);
                            match &elem_type {
                                Type::User(type_decl) => {
                                    self.last_expr_type = ObjectType::NestedObject;
                                    let mono_name = type_decl.name.to_string();
                                    self.last_enum_variant_mono = Some(mono_name);
                                }
                                Type::GenericInstance(_) => {
                                    self.last_expr_type = ObjectType::NestedObject;
                                }
                                Type::StrFixed(_)
                                | Type::StrOwned
                                | Type::CStrLit
                                | Type::StrSlice => {
                                    self.last_expr_type = ObjectType::String;
                                }
                                Type::Char => {
                                    self.last_expr_type = ObjectType::Char;
                                }
                                Type::Float => {
                                    self.last_expr_type = ObjectType::Float;
                                }
                                Type::Double => {
                                    self.last_expr_type = ObjectType::Double;
                                }
                                Type::Bool => {
                                    self.last_expr_type = ObjectType::Bool;
                                }
                                Type::Uint | Type::U64 | Type::USize => {
                                    self.last_expr_type = ObjectType::Uint;
                                }
                                _ => {
                                    self.last_expr_type = ObjectType::Int;
                                }
                            }
                        }
                        ObjectType::String => {
                            // String indexing returns char
                            self.last_expr_type = ObjectType::Char;
                        }
                        _ => {
                            self.last_expr_type = ObjectType::Int;
                        }
                    }
                }
            }
            _ => return self.expr_ops(expr),
        }
        Ok(())
    }
}
