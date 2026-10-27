//! PLAN-843 结构拆分：compile_expr_inner 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl Codegen {
    /// PLAN-843 尾递归派发段（自 compile_expr_inner 连续臂切片，臂体逐字保留）。
    pub(super) fn expr_calls(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            Expr::Call(call) => {
                // Plan 390 §14 L3: Task variant constructor in expression position.
                // `Add(3, 5)` where Add is a WithBindings variant of some task →
                // build a Value::Obj {__variant:"Add", <binding0>:3, <binding1>:5}
                // via CREATE_OBJ, mirroring Expr::Object codegen. This lets
                // `h.send(Add(3, 5))` carry a structured multi-field message.
                if let Expr::Ident(variant_name) = call.name.as_ref() {
                    let vname = variant_name.to_string();
                    // Search all tasks for a variant with this name. Clone the
                    // binding names out to release the immutable borrow of self
                    // before we start emitting code (mutable borrows).
                    let matched: Option<Vec<String>> = self
                        .task_variants
                        .values()
                        .find_map(|vs| vs.iter().find(|(vn, _)| *vn == vname))
                        .map(|(_, bn)| bn.clone());
                    if let Some(bind_names) = matched {
                        // Build pairs: [__variant: "Add", <b0>: arg0, <b1>: arg1, ...]
                        // __variant is a string constant.
                        let variant_str_idx = {
                            let idx = self.strings.len();
                            self.strings.push(vname.clone().into_bytes());
                            idx as u32
                        };
                        self.emit(OpCode::LOAD_STR);
                        self.code.extend_from_slice(&variant_str_idx.to_le_bytes());

                        // Push each arg value (compile the Call's positional args).
                        let n_args = call.args.args.len().min(bind_names.len());
                        for i in 0..n_args {
                            if let crate::ast::Arg::Pos(e) = &call.args.args[i] {
                                self.compile_expr(e)?;
                            }
                        }

                        // Build object_keys: ["__variant", bind_names[0], ...]
                        let keys: Vec<auto_val::ValueKey> =
                            std::iter::once(auto_val::ValueKey::Str(auto_val::AutoStr::from(
                                "__variant",
                            )))
                            .chain(bind_names[..n_args].iter().map(|n| {
                                auto_val::ValueKey::Str(auto_val::AutoStr::from(n.as_str()))
                            }))
                            .collect();
                        let key_index = self.object_keys.len() as u32;
                        // Types: __variant is String; args inferred.
                        let mut types = vec![ObjectType::String];
                        for i in 0..n_args {
                            if let crate::ast::Arg::Pos(e) = &call.args.args[i] {
                                types.push(self.infer_object_type(e));
                            }
                        }
                        self.object_types.push(types.clone());
                        self.object_keys.push(keys);

                        let field_count = (1 + n_args) as u8; // __variant + args
                        self.emit(OpCode::CREATE_OBJ);
                        self.code.extend_from_slice(&key_index.to_le_bytes());
                        self.code.push(field_count);
                        self.last_expr_type = ObjectType::NestedObject;
                        return Ok(());
                    }
                }

                // Plan 087 Phase 2: Check if this is a generic constructor call (e.g., Pair.new(1, "a"))
                // IMPORTANT: Skip inline construction if the type has a user-defined new() method
                let is_generic_constructor = if let Expr::Dot(obj, method) = call.name.as_ref() {
                    if method == "new" {
                        // Plan 322: Handle both Ident("Type") and GenName("List<Item>")
                        let type_name_opt: Option<&str> = match obj.as_ref() {
                            Expr::Ident(name) => Some(name.as_ref()),
                            Expr::GenName(full) => {
                                // Extract base type name: "List<Item>" → "List"
                                full.as_str().split('<').next()
                            }
                            _ => None,
                        };
                        if let Some(type_name) = type_name_opt {
                            // Plan 322: List<T>.new is NOT a generic constructor — it's
                            // a builtin that creates ListData, not GenericInstanceData.
                            // Let it fall through to the normal call path (shim_list_new).
                            if type_name == "List" || type_name == "Array" || type_name == "Map" {
                                false
                            } else if self.exports.contains_key(&format!("{}.new", type_name)) {
                                false // User defined their own new() — use regular CALL
                            } else if self.rust_native_map.contains_key(type_name)
                                && !matches!(type_name, "HashMap" | "HashSet")
                            {
                                // Plan 430: use.rust 导入的类型名(Vec...)与 Auto 泛型
                                // 模板撞名时 rust 导入优先——构造器走 dispatch 3000。
                                // HashMap/HashSet 例外:Auto Map natives(auto.hashmap.*)
                                // 与 a2r 双路径已打通,由原生路径拥有,避免表示分裂。
                                false
                            } else {
                                self.generic_registry.has_template(type_name)
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };
                // DEBUG: trace generic constructor decision for List.new
                if let Expr::Dot(_obj, method) = call.name.as_ref() {
                    if method == "new" {}
                }

                if is_generic_constructor {
                    // Plan 087 Phase 2: Generic constructor call
                    // Generate: NEW_INSTANCE + CONSTRUCT_INSTANCE

                    if let Expr::Dot(obj, _method) = call.name.as_ref() {
                        if let Expr::Ident(type_name) = obj.as_ref() {
                            // Get or create ClassType to determine mono_name and field count
                            let type_args = Vec::new(); // For Phase 2, use empty type args (no inference)
                            if let Ok(class_type) = self
                                .generic_registry
                                .get_or_create_type(type_name.as_ref(), type_args)
                            {
                                let field_count = class_type.template.fields.len();

                                // Plan 207 Task 4: Compile arguments, reordering named args by field index
                                let has_named_args = call
                                    .args
                                    .args
                                    .iter()
                                    .any(|a| matches!(a, crate::ast::Arg::Pair(_, _)));
                                let args_to_compile: Vec<&crate::ast::Arg> =
                                    if has_named_args && !class_type.template.fields.is_empty() {
                                        let mut indexed: Vec<(usize, &crate::ast::Arg)> =
                                            call.args.args.iter().enumerate().collect();
                                        indexed.sort_by_key(|(_, arg)| {
                                            if let crate::ast::Arg::Pair(name, _) = arg {
                                                class_type
                                                    .template
                                                    .fields
                                                    .iter()
                                                    .position(|f| f.name == name.as_ref())
                                                    .unwrap_or(0)
                                            } else {
                                                0
                                            }
                                        });
                                        indexed.into_iter().map(|(_, a)| a).collect()
                                    } else {
                                        call.args.args.iter().collect()
                                    };

                                // Compile arguments (push values onto stack)
                                // Stack: ..., arg1, arg2, ..., argN
                                let field_defs = class_type.fields();
                                for (i, arg) in args_to_compile.iter().enumerate() {
                                    match arg {
                                        crate::ast::Arg::Pos(expr) => {
                                            self.compile_expr(expr)?;
                                        }
                                        crate::ast::Arg::Pair(_key, expr) => {
                                            // Named argument: compile value only
                                            self.compile_expr(expr)?;
                                        }
                                        crate::ast::Arg::Name(name) => {
                                            // Named argument without value - treat as string
                                            self.emit(OpCode::LOAD_STR);
                                            let s_bytes = name.to_string().as_bytes().to_vec();
                                            let s_idx = self.strings.len() as u32;
                                            self.strings.push(s_bytes);
                                            self.code.extend_from_slice(&s_idx.to_le_bytes());
                                        }
                                    }
                                    // Plan 230: promote f32 -> f64 when field type is Double
                                    if let Some(field_def) = field_defs.get(i) {
                                        if matches!(field_def.field_type, Type::Double)
                                            && self.last_expr_type == ObjectType::Float
                                        {
                                            self.emit(OpCode::PROMOTE_F64);
                                        }
                                    }
                                }

                                // Emit NEW_INSTANCE instruction
                                // Push mono_name length
                                let mono_name = class_type.mono_name.clone();
                                let name_bytes = mono_name.as_bytes();
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(name_bytes.len() as i32);

                                // Emit NEW_INSTANCE opcode first
                                self.emit(OpCode::NEW_INSTANCE);

                                // Then emit mono_name bytes directly into code (after opcode)
                                for &byte in name_bytes {
                                    self.code.push(byte);
                                }

                                // Emit CONSTRUCT_INSTANCE
                                // Stack layout should be: ..., instance_id, field_count, arg1, ..., argN
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(field_count as i32);
                                self.emit(OpCode::CONSTRUCT_INSTANCE);

                                return Ok(());
                            } else {
                                eprintln!(
                                    "Warning: Failed to get/create generic type '{}'",
                                    type_name
                                );
                                // Fallback to regular call
                            }
                        }
                    }

                    // Fallback to regular call if something went wrong
                }

                // Plan 197 Task 12: Check if this is an enum variant construction (e.g., Atom.Int(42))
                // Pattern: Expr::Dot(Expr::Ident(type_name), variant_name) where
                // "TypeName.VariantName" is a registered template in the generic registry.
                let is_enum_variant = if let Expr::Dot(obj, method) = call.name.as_ref() {
                    if let Expr::Ident(type_name) = obj.as_ref() {
                        let variant_mono = format!("{}.{}", type_name.as_ref(), method.as_ref());
                        self.generic_registry.has_template(&variant_mono)
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_enum_variant {
                    // Generate: NEW_INSTANCE + CONSTRUCT_INSTANCE (same pattern as generic constructor)
                    if let Expr::Dot(obj, method) = call.name.as_ref() {
                        if let Expr::Ident(type_name) = obj.as_ref() {
                            let variant_mono =
                                format!("{}.{}", type_name.as_ref(), method.as_ref());

                            // Get the ClassType to determine field count
                            let type_args = Vec::new();
                            if let Ok(class_type) = self
                                .generic_registry
                                .get_or_create_type(&variant_mono, type_args)
                            {
                                let field_count = class_type.template.fields.len();

                                // Plan 207 Task 4: Compile arguments, reordering named args by field index
                                let has_named_args = call
                                    .args
                                    .args
                                    .iter()
                                    .any(|a| matches!(a, crate::ast::Arg::Pair(_, _)));
                                let args_to_compile: Vec<&crate::ast::Arg> =
                                    if has_named_args && !class_type.template.fields.is_empty() {
                                        let mut indexed: Vec<(usize, &crate::ast::Arg)> =
                                            call.args.args.iter().enumerate().collect();
                                        indexed.sort_by_key(|(_, arg)| {
                                            if let crate::ast::Arg::Pair(name, _) = arg {
                                                class_type
                                                    .template
                                                    .fields
                                                    .iter()
                                                    .position(|f| f.name == name.as_ref())
                                                    .unwrap_or(0)
                                            } else {
                                                0
                                            }
                                        });
                                        indexed.into_iter().map(|(_, a)| a).collect()
                                    } else {
                                        call.args.args.iter().collect()
                                    };

                                // Compile arguments (push values onto stack)
                                let field_defs = class_type.fields();
                                for (i, arg) in args_to_compile.iter().enumerate() {
                                    match arg {
                                        crate::ast::Arg::Pos(expr) => {
                                            self.compile_expr(expr)?;
                                        }
                                        crate::ast::Arg::Pair(_key, expr) => {
                                            self.compile_expr(expr)?;
                                        }
                                        crate::ast::Arg::Name(name) => {
                                            self.emit(OpCode::LOAD_STR);
                                            let s_bytes = name.to_string().as_bytes().to_vec();
                                            let s_idx = self.strings.len() as u32;
                                            self.strings.push(s_bytes);
                                            self.code.extend_from_slice(&s_idx.to_le_bytes());
                                        }
                                    }
                                    // Plan 230: promote f32 -> f64 when field type is Double
                                    if let Some(field_def) = field_defs.get(i) {
                                        if matches!(field_def.field_type, Type::Double)
                                            && self.last_expr_type == ObjectType::Float
                                        {
                                            self.emit(OpCode::PROMOTE_F64);
                                        }
                                    }
                                }

                                // Emit NEW_INSTANCE instruction
                                let mono_name = class_type.mono_name.clone();
                                let name_bytes = mono_name.as_bytes();
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(name_bytes.len() as i32);

                                self.emit(OpCode::NEW_INSTANCE);
                                for &byte in name_bytes {
                                    self.code.push(byte);
                                }

                                // Emit CONSTRUCT_INSTANCE
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(field_count as i32);
                                self.emit(OpCode::CONSTRUCT_INSTANCE);

                                // Plan 197 Task 13: Mark result as a heap object with variant type info
                                // This enables field access (e.g., a._0) on enum variant instances
                                self.last_expr_type = ObjectType::NestedObject;
                                self.last_enum_variant_mono = Some(variant_mono);

                                return Ok(());
                            } else {
                                eprintln!(
                                    "Warning: Failed to get/create enum variant type '{}'",
                                    variant_mono
                                );
                                // Fallback to regular call
                            }
                        }
                    }
                }

                // Plan 118 Phase 2: Check if this is a type constructor call (e.g., Inner(x: 10))
                // If the call name is a registered type, treat it as a type instance creation
                if let Expr::Ident(type_name) = call.name.as_ref() {
                    let type_name_str = type_name.to_string();

                    // Plan 442 C2 (Bug B): rust/py-FFI imports also register
                    // synthetic type decls (register_rust_type), so a bare
                    // routing-fn call like `get(h1)` would hit this branch and
                    // construct a GenericInstanceData instead of dispatching
                    // to the FFI path. FFI-imported names never construct
                    // here — mirror of the var_types guard below.
                    let is_ffi_import = self.rust_native_map.contains_key(&type_name_str)
                        || self.py_native_map.contains_key(&type_name_str);

                    // Check if this type is registered
                    if !is_ffi_import
                        && (self.generic_registry.has_template(&type_name_str)
                            || self.get_type(&type_name_str).is_some())
                    {
                        // This is a type constructor call - compile as type instance
                        vm_debug!(
                            "DEBUG: Compiling type constructor call for '{}'",
                            type_name_str
                        );

                        // Get type info
                        let member_names = if self.generic_registry.has_template(&type_name_str) {
                            let type_args = Vec::new();
                            if let Ok(class_type) = self
                                .generic_registry
                                .get_or_create_type(&type_name_str, type_args)
                            {
                                class_type
                                    .template
                                    .fields
                                    .iter()
                                    .map(|f| f.name.clone())
                                    .collect()
                            } else {
                                Vec::new()
                            }
                        } else if let Some(type_info) = self.get_type(&type_name_str) {
                            type_info.member_names.clone()
                        } else {
                            Vec::new()
                        };

                        // Compile arguments (push values onto stack)
                        let arg_count = call.args.args.len() as u8;
                        // Plan 230: Get field types for f32→f64 promotion
                        let constructor_field_types: Option<Vec<Type>> = if self
                            .generic_registry
                            .has_template(&type_name_str)
                        {
                            self.generic_registry
                                .get_or_create_type(&type_name_str, Vec::new())
                                .ok()
                                .map(|ct| ct.fields().into_iter().map(|f| f.field_type).collect())
                        } else {
                            None
                        };
                        for (i, arg) in call.args.args.iter().enumerate() {
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Pair(_key, expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Name(_) => {
                                    // Name-only arg - placeholder
                                }
                            }
                            // Plan 230: promote f32 -> f64 when field type is Double
                            if let Some(ref field_types) = constructor_field_types {
                                if let Some(field_type) = field_types.get(i) {
                                    if matches!(field_type, Type::Double)
                                        && self.last_expr_type == ObjectType::Float
                                    {
                                        self.emit(OpCode::PROMOTE_F64);
                                    }
                                }
                            }
                        }

                        // Create object keys using type member names
                        let keys: Vec<auto_val::ValueKey> = member_names
                            .iter()
                            .take(arg_count as usize)
                            .map(|name| auto_val::ValueKey::Str(name.clone().into()))
                            .collect();

                        let _key_index = self.object_keys.len() as u32;
                        self.object_keys.push(keys);

                        // Infer field types from args
                        let types: Vec<ObjectType> = call
                            .args
                            .args
                            .iter()
                            .take(arg_count as usize)
                            .map(|arg| match arg {
                                crate::ast::Arg::Pos(expr) => self.infer_object_type(expr),
                                crate::ast::Arg::Pair(_, expr) => self.infer_object_type(expr),
                                crate::ast::Arg::Name(_) => ObjectType::Int,
                            })
                            .collect();
                        self.object_types.push(types);

                        // Plan 118 Phase 7: Use NEW_INSTANCE + CONSTRUCT_INSTANCE for user types
                        // This ensures objects are stored in heap_objects (4000000+) instead of objects (1000000+)
                        let field_count = arg_count.min(member_names.len() as u8);

                        // NEW_INSTANCE expects:
                        // - Stack: mono_name_len (i32)
                        // - Flash: mono_name_bytes
                        // After execution: instance_id pushed to stack

                        let mono_name_bytes = type_name_str.as_bytes().to_vec();

                        // Push mono_name length to stack
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(mono_name_bytes.len() as i32);

                        // Emit NEW_INSTANCE opcode
                        self.emit(OpCode::NEW_INSTANCE);

                        // Emit mono_name bytes directly to flash (code stream)
                        for byte in &mono_name_bytes {
                            self.code.push(*byte);
                        }

                        // Stack now has: [..., field_value1, ..., field_valueN, instance_id]
                        // CONSTRUCT_INSTANCE expects:
                        // - Stack: field_count, instance_id, field_value1, ..., field_valueN
                        // So we need to: push field_count, then CONSTRUCT_INSTANCE
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(field_count as i32);
                        self.emit(OpCode::CONSTRUCT_INSTANCE);

                        // Track variable type for this instance
                        self.last_expr_type = ObjectType::NestedObject;

                        return Ok(());
                    }
                }

                // Regular function/method call (existing code)
                // Extract function name and determine if it's a method call
                // Plan 073: Support both static methods (Type.method) and instance methods (obj.method)
                let mut func_name = match call.name.as_ref() {
                    Expr::Ident(name) => Some(name.to_string()),
                    Expr::Dot(obj, method) => {
                        // Method call: Type.method (static) or obj.method (instance)
                        // Plan 569 D2: py-类型侧表插队——接收者 py 可能（py-ffi
                        // 调用点 / 变量落盘 / 用户 fn 返回传导三源）时，方法调用
                        // 改发 obj_len/obj_call 组合子（运行期 tag 双通道：
                        // PyObjectHandle → GIL 通道，Auto 值 → 原生语义），
                        // 不再进下方 infer_object_type → "str.len" 限定名 peek
                        // 的静态路由（P539-D2 病灶：py 句柄被谎记 String 后
                        // str.len 把堆 id 当字符串池索引读到垃圾）。
                        if self.receiver_may_py(obj.as_ref()) {
                            if method.as_str() == "len" && call.args.args.is_empty() {
                                self.compile_expr(obj)?;
                                self.emit(OpCode::CALL_NAT_COUNTED);
                                self.code.extend_from_slice(
                                    &crate::vm::interop::NATIVE_INTEROP_OBJ_LEN.to_le_bytes(),
                                );
                                self.code.push(1u8);
                                self.last_expr_type = ObjectType::Int;
                                self.last_expr_may_py = false; // len 结果为 Int
                                return Ok(());
                            }
                            // 其余方法 → obj_call(recv, "method", args...)——与
                            // s2s A1 产物同构；结果可能为 py 值（侧表续传）。
                            self.compile_expr(obj)?;
                            let method_idx = self.add_string(method.as_str());
                            self.emit(OpCode::LOAD_STR);
                            self.code.extend_from_slice(&method_idx.to_le_bytes());
                            for arg in &call.args.args {
                                if let crate::ast::Arg::Pos(a) = arg {
                                    self.compile_expr(a)?;
                                }
                            }
                            let n_args = 2 + call.args.args.len() as u8; // recv + method + args
                            self.emit(OpCode::CALL_NAT_COUNTED);
                            self.code.extend_from_slice(
                                &crate::vm::interop::NATIVE_INTEROP_OBJ_CALL.to_le_bytes(),
                            );
                            self.code.push(n_args);
                            // py 结果沿用既有谎言口径（Auto|String → String），
                            // 顶层格式化照常；may_py 续传供链式 Store 落盘。
                            self.last_expr_type = ObjectType::String;
                            self.last_expr_may_py = true;
                            return Ok(());
                        }
                        // Plan 087 Phase 3: Support generic instance method calls
                        match obj.as_ref() {
                            Expr::Ident(obj_name) => {
                                vm_debug!(
                                    "DEBUG: Dot Ident: obj_name={}, method={}, var_type={:?}",
                                    obj_name,
                                    method,
                                    self.var_types.get(obj_name.as_str())
                                );
                                // Check if it's a static method call (Type.method with capital T)
                                // Also treat stdlib singleton module names (env, fs) as static
                                // BUT: if the name is a known local variable (e.g. `json str` parameter)
                                // or a module-level global variable, treat as instance method call
                                // instead of module call. Without the global_vars check, an
                                // uppercase-named global (e.g. `H0`) is misclassified as a static
                                // type reference. See Plan 347 (sha2 global-var bitop bug).
                                // PLAN-066 T-12（F-W1）：i18n 单例排除在局部
                                // 变量判定外——合成作用域无 i18n 绑定（组件级
                                // `let i18n = useI18n()` 不进合成体），实例路径
                                // 编译接收者报 Undefined variable 毒化导出；
                                // 静态模块路由 → P240 ("i18n","t") → auto.i18n.t
                                // （PLAN-050 C7 查表）。
                                let is_local_var = obj_name.as_str() != "i18n"
                                    && (self.var_types.contains_key(obj_name.as_ref())
                                    || self.global_vars.contains(obj_name.as_ref())
                                    || self.lookup_var(obj_name.as_str()).is_some()
                                    // Plan 348 E1: an imported value name (e.g.
                                    // `use testlib: MAX`) is a value global, so a
                                    // call like `MAX.to(str)` must treat MAX as
                                    // an instance (load the global) rather than a
                                    // static type reference.
                                    || self.import_scope.contains_key(obj_name.as_ref()));
                                let is_stdlib_module = !is_local_var
                                    && matches!(
                                        obj_name.as_ref(),
                                        "env"
                                            | "fs"
                                            | "json"
                                            | "http"
                                            | "url"
                                            | "shell"
                                            | "regex"
                                            | "session"
                                            | "template"
                                            | "openapi"
                                            | "storage"
                                            | "host"
                                            | "i18n"
                                    );
                                vm_debug!("DEBUG: Dot Ident static-check: obj={}, is_local_var={}, is_stdlib_module={}", obj_name, is_local_var, is_stdlib_module);
                                if !is_local_var
                                    && (is_stdlib_module
                                        || self.is_type_name_heuristic(obj_name)
                                        || self.is_type(obj_name))
                                {
                                    // Plan 127: Special handling for TaskType.spawn() and TaskType.send()
                                    // These should use the generic Task.spawn/Task.send native functions
                                    if method.as_str() == "spawn"
                                        && self.types.contains_key(obj_name.as_ref())
                                    {
                                        // Check if this is a task type
                                        let type_info = self.types.get(obj_name.as_ref());
                                        if let Some(info) = type_info {
                                            if info._name.contains("#single")
                                                || info._name == obj_name.as_ref()
                                            {
                                                // This is a task type - use Task.spawn
                                                vm_debug!("DEBUG: Task spawn detected: {}.spawn() -> auto.task.spawn", obj_name);
                                                Some("auto.task.spawn".to_string())
                                            } else {
                                                Some(format!("{}.{}", obj_name, method))
                                            }
                                        } else {
                                            Some(format!("{}.{}", obj_name, method))
                                        }
                                    } else if method.as_str() == "send"
                                        && self.types.contains_key(obj_name.as_ref())
                                    {
                                        // Singleton task send: TaskType.send(msg)
                                        let type_info = self.types.get(obj_name.as_ref());
                                        if let Some(info) = type_info {
                                            if info._name.contains("#single") {
                                                // This is a singleton task - use Task.send
                                                vm_debug!("DEBUG: Singleton task send detected: {}.send() -> auto.task.singleton_send", obj_name);
                                                Some("auto.task.singleton_send".to_string())
                                            } else {
                                                Some(format!("{}.{}", obj_name, method))
                                            }
                                        } else {
                                            Some(format!("{}.{}", obj_name, method))
                                        }
                                    } else {
                                        // Static method call: Type.method
                                        // PLAN-066 T-12（F-W1）：i18n 单例 →
                                        // auto.i18n.t 原生查表（i18n_lookup，
                                        // PLAN-050 C7）。此 inner func_name
                                        // 走静态 native 发射通道。
                                        if obj_name.as_str() == "i18n" && method.as_str() == "t" {
                                            Some("auto.i18n.t".to_string())
                                        } else {
                                            Some(format!("{}.{}", obj_name, method))
                                        }
                                    }
                                } else {
                                    // Instance method call: obj.method
                                    // Plan 087 Phase 3: Check if obj is a generic instance
                                    let func_name = if let Some(ty) =
                                        self.var_types.get(obj_name.as_ref())
                                    {
                                        if let Type::GenericInstance(inst) = ty {
                                            // Generate monomorphic method name for generic instance
                                            // Example: p.get_key() where p: Pair<int, string>
                                            //          → "Pair_int_str.get_key"
                                            let resolved = if let Some(tmpl) = self
                                                .generic_registry
                                                .get_template(&inst.base_name.to_string())
                                            {
                                                // User-defined generic type
                                                let mono_name =
                                                    tmpl.mono_name_from_args(&inst.args);
                                                format!("{}.{}", mono_name, method)
                                            } else if let Some(mono) = self.try_mono_dispatch(
                                                &inst.base_name.to_string(),
                                                method.as_ref(),
                                                &inst.args,
                                            ) {
                                                // Plan 194 Task 3: Built-in collection type
                                                // (HashMap, HashSet, etc.) with generic params
                                                // try_mono_dispatch returns a fully qualified name
                                                mono
                                            } else {
                                                // Fallback: use base_name.method for methods
                                                // that don't need typed dispatch (new, size, clear, etc.)
                                                format!("{}.{}", inst.base_name, method)
                                            };
                                            Some(resolved)
                                        } else {
                                            // Not a generic instance, use regular inference
                                            // Plan 194 Task 1: Try monomorphic dispatch first
                                            // Extract base type name and generic params for typed natives
                                            let (base_name, type_args) = match ty {
                                                Type::Map(k, v) => (
                                                    "Map".to_string(),
                                                    vec![*k.clone(), *v.clone()],
                                                ),
                                                Type::List(elem) => {
                                                    ("List".to_string(), vec![*elem.clone()])
                                                }
                                                Type::Array(_) => {
                                                    // Array HOF methods (map, filter, etc.) are dispatched as List.* natives
                                                    let hof_methods = [
                                                        "map", "filter", "for_each", "find", "any",
                                                        "all", "reduce",
                                                    ];
                                                    if hof_methods.contains(&method.as_str()) {
                                                        ("List".to_string(), vec![])
                                                    } else {
                                                        ("Array".to_string(), vec![])
                                                    }
                                                }
                                                Type::User(td) => {
                                                    let name = td.name.to_string();
                                                    // Strip module prefix (e.g., "types.ToolRegistry" → "ToolRegistry")
                                                    let short = name
                                                        .rsplit('.')
                                                        .next()
                                                        .unwrap_or(&name)
                                                        .to_string();
                                                    // Plan 325: enum 变体类型的实例方法分发。enum 变体值
                                                    // （如 MyResult.Ok(42)）的 var_type 是 User("MyResult.Ok")，
                                                    // rsplit 取到 "Ok" → func_name "Ok.is_ok" 找不到。若 td.name
                                                    // 含 "." 且去掉最后段后的 enum 名有已编译方法（在 exports），
                                                    // 用 enum 名作 base_name，使 MyResult.is_ok 被解析到。
                                                    let enum_base =
                                                        name.rsplit_once('.').map(|(en, _)| en);
                                                    let base = if let Some(enum_name) = enum_base {
                                                        let enum_method = format!(
                                                            "{}.{}",
                                                            enum_name,
                                                            method.as_ref()
                                                        );
                                                        let is_enum_variant = self
                                                            .generic_registry
                                                            .has_template(&name)
                                                            || self.enum_values.contains_key(&name);
                                                        // Plan 325: enum 变体类型的方法分发。本模块时
                                                        // exports 含 enum_method；跨模块时方法在被 use 模块
                                                        // 的 exports，本地查不到——但 linker 的 fallback
                                                        // 解析能找到。故 is_enum_variant 为真即用 enum 名，
                                                        // has_method 只作本模块快速确认（不阻塞跨模块）。
                                                        if is_enum_variant {
                                                            enum_name.to_string()
                                                        } else {
                                                            short
                                                        }
                                                    } else {
                                                        short
                                                    };
                                                    (base, vec![])
                                                }
                                                other => {
                                                    // Extract name via infer_type_from_var logic for other types
                                                    let name = match other {
                                                        Type::Int | Type::I64 => "int".to_string(),
                                                        Type::Uint
                                                        | Type::U64
                                                        | Type::Byte
                                                        | Type::USize => "uint".to_string(),
                                                        Type::Float | Type::Double => {
                                                            "float".to_string()
                                                        }
                                                        Type::Bool => "bool".to_string(),
                                                        Type::Char => "char".to_string(),
                                                        Type::StrFixed(_)
                                                        | Type::StrOwned
                                                        | Type::StrSlice
                                                        | Type::CStrLit => "str".to_string(),
                                                        Type::Array(_) => "Array".to_string(),
                                                        _ => String::new(),
                                                    };
                                                    (name, vec![])
                                                }
                                            };

                                            // Try monomorphic dispatch for collections with type params
                                            if !type_args.is_empty() {
                                                if let Some(mono_name) = self.try_mono_dispatch(
                                                    &base_name,
                                                    method.as_ref(),
                                                    &type_args,
                                                ) {
                                                    Some(mono_name)
                                                } else {
                                                    // Mono dispatch didn't resolve -- fall back to regular native name
                                                    vm_debug!("DEBUG: Instance method call: obj={}, method={}, var_types={:?}", obj_name, method, self.var_types);
                                                    if let Some(type_name) =
                                                        self.infer_type_from_var(obj_name.as_ref())
                                                    {
                                                        vm_debug!(
                                                            "DEBUG: Inferred type name: {}",
                                                            type_name
                                                        );
                                                        Some(format!("{}.{}", type_name, method))
                                                    } else {
                                                        if method.as_str() == "send" {
                                                            vm_debug!("DEBUG: Assuming TaskHandle.send for unknown type variable {}", obj_name);
                                                            Some("auto.task.send".to_string())
                                                        } else {
                                                            vm_debug!("DEBUG: Failed to infer type for {}",
                                                                obj_name
                                                            );
                                                            Some(format!("{}.{}", obj_name, method))
                                                        }
                                                    }
                                                }
                                            } else {
                                                // No type args -- regular inference
                                                // Special case: base_name was overridden to "List" for Array HOF methods
                                                if base_name == "List" {
                                                    Some(format!("List.{}", method))
                                                } else if self.exports.contains_key(&format!(
                                                    "{}.{}",
                                                    base_name,
                                                    method.as_ref()
                                                )) || self.is_enum_name(&base_name)
                                                {
                                                    // Plan 325: base_name 已是有效 enum 方法前缀（Type::User
                                                    // 分支从 "MyResult.Ok" 提取出 "MyResult"）。本模块时 exports
                                                    // 含方法；跨模块时方法在被 use 模块的 exports，本地查不到，
                                                    // 但 base_name 是已知 enum 名即应构造 func_name，交 linker
                                                    // fallback 解析。is_enum_name 避免对普通类型误判。
                                                    Some(format!(
                                                        "{}.{}",
                                                        base_name,
                                                        method.as_ref()
                                                    ))
                                                } else {
                                                    if let Some(type_name) =
                                                        self.infer_type_from_var(obj_name.as_ref())
                                                    {
                                                        vm_debug!(
                                                            "DEBUG: Inferred type name: {}",
                                                            type_name
                                                        );
                                                        Some(format!("{}.{}", type_name, method))
                                                    } else {
                                                        if method.as_str() == "send" {
                                                            vm_debug!("DEBUG: Assuming TaskHandle.send for unknown type variable {}", obj_name);
                                                            Some("auto.task.send".to_string())
                                                        } else {
                                                            vm_debug!("DEBUG: Failed to infer type for {}",
                                                                obj_name
                                                            );
                                                            Some(format!("{}.{}", obj_name, method))
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        // No type info, use regular inference
                                        vm_debug!(
                                            "DEBUG: No type info for obj={}, var_types empty",
                                            obj_name
                                        );
                                        if let Some(type_name) =
                                            self.infer_type_from_var(obj_name.as_ref())
                                        {
                                            vm_debug!("DEBUG: Inferred type name: {}", type_name);
                                            Some(format!("{}.{}", type_name, method))
                                        } else {
                                            // Plan 127: Handle TaskHandle.send() when type is unknown
                                            // If the variable name suggests it's a handle (e.g., "handle")
                                            // and the method is "send", use TaskHandle.send
                                            if method.as_str() == "send" {
                                                vm_debug!("DEBUG: Assuming TaskHandle.send for unknown type variable {}", obj_name);
                                                Some("auto.task.send".to_string())
                                            } else {
                                                vm_debug!(
                                                    "DEBUG: Failed to infer type for {}",
                                                    obj_name
                                                );
                                                Some(format!("{}.{}", obj_name, method))
                                            }
                                        }
                                    };

                                    func_name
                                }
                            }
                            Expr::GenName(gen_name) => {
                                // Plan 317: Generic type static method call, e.g.
                                // List<Note>.new([...]). GenName is "List<Note>".
                                // Strip the type args to get the base type name,
                                // then build func_name as "Type.method" so the
                                // existing BIGVM_NATIVES dispatch resolves it
                                // (e.g. "List.new" → auto.list.new, "List.push"
                                // → auto.list.push).
                                let base = gen_name.split('<').next().unwrap_or(gen_name);
                                Some(format!("{}.{}", base, method))
                            }
                            _ => {
                                // Complex expression (e.g., arr[0].push, foo().method, self.field.method)
                                // Or literal expressions (e.g., 1.str(), "hello".upper())
                                // Plan 118 Phase 4: Handle literal method calls
                                // Plan 325 缺陷 1: enum 变体方法调用（Color.Red.name()）。
                                // receiver 是 Dot(Ident(EnumName), variant)，enum 变体的
                                // 运行时值是 i32 discriminant，infer_object_type 推不出 enum
                                // 类型名。特判：若 receiver 形如 EnumName.Variant 且 EnumName
                                // 有已编译的方法，func_name 用 EnumName.method（命中下方 exports
                                // 解析）。否则 fall through 到常规类型推断。
                                let enum_method_name = (|| -> Option<String> {
                                    // 提取 receiver 的 enum 名：Color.Red（Dot）或 Reg.Yes(42)（Call{Dot}）
                                    let enum_name_str: Option<String> = match obj.as_ref() {
                                        Expr::Dot(enum_obj, _) => {
                                            if let Expr::Ident(enum_name) = enum_obj.as_ref() {
                                                Some(enum_name.to_string())
                                            } else {
                                                None
                                            }
                                        }
                                        Expr::Call(inner) => {
                                            if let Expr::Dot(enum_obj, _) = inner.name.as_ref() {
                                                if let Expr::Ident(enum_name) = enum_obj.as_ref() {
                                                    Some(enum_name.to_string())
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    };
                                    if let Some(enum_name_str) = enum_name_str {
                                        // enum 存在性：scalar 变体在 enum_values，enum 类型在 type_store，
                                        // 带 payload 变体在 generic_registry。三者任一命中即可。
                                        let prefix = format!("{}.", enum_name_str);
                                        let has_enum = self
                                            .enum_values
                                            .keys()
                                            .any(|k| k.starts_with(&prefix))
                                            || self.types.contains_key(&enum_name_str)
                                            || self.generic_registry.has_template(&enum_name_str);
                                        let has_method = self.exports.contains_key(&format!(
                                            "{}.{}",
                                            enum_name_str,
                                            method.as_ref()
                                        ));
                                        if has_enum && has_method {
                                            return Some(format!(
                                                "{}.{}",
                                                enum_name_str,
                                                method.as_ref()
                                            ));
                                        }
                                    }
                                    None
                                })();
                                if let Some(em) = enum_method_name {
                                    Some(em)
                                } else {
                                    let inferred_type = self.infer_object_type(obj.as_ref());

                                    // Plan 197 Task 14: Array.len() emits ARRAY_LEN opcode directly
                                    if inferred_type == ObjectType::Array
                                        && method.as_str() == "len"
                                        && call.args.args.is_empty()
                                    {
                                        // Compile receiver (array/list id) then emit ARRAY_LEN
                                        self.compile_expr(obj)?;
                                        self.emit(OpCode::ARRAY_LEN);
                                        self.last_expr_type = ObjectType::Int;
                                        return Ok(());
                                    }

                                    let type_name: String = match inferred_type {
                                        ObjectType::Int | ObjectType::Byte => "int".to_string(),
                                        ObjectType::Uint => "uint".to_string(),
                                        ObjectType::Float | ObjectType::Double => {
                                            "float".to_string()
                                        }
                                        ObjectType::String => "str".to_string(),
                                        ObjectType::Char => "char".to_string(),
                                        ObjectType::Bool => "bool".to_string(),
                                        ObjectType::Array => "List".to_string(),
                                        _ => {
                                            // Plan 202 Step 3: Fallback to infer_expr_type when infer_object_type
                                            // returns a non-specific type (NestedObject or default)
                                            let fallback_type = self.infer_expr_type(obj.as_ref());
                                            let fallback_ot =
                                                self.type_to_object_type(&fallback_type);
                                            match fallback_ot {
                                                ObjectType::String => "str".to_string(),
                                                ObjectType::Array => "List".to_string(),
                                                ObjectType::Int => "int".to_string(),
                                                ObjectType::Uint => "uint".to_string(),
                                                ObjectType::Float | ObjectType::Double => {
                                                    "float".to_string()
                                                }
                                                ObjectType::Bool => "bool".to_string(),
                                                ObjectType::Char => "char".to_string(),
                                                _ => {
                                                    // Plan 197 Task 3: Try to resolve user-defined type name
                                                    self.infer_user_type_name(obj.as_ref())
                                                        .unwrap_or_else(|| "Unknown".to_string())
                                                }
                                            }
                                        }
                                    };
                                    let native_name = format!("{}.{}", type_name, method);

                                    // Check if this native exists(peek:决策点不得注册)
                                    if BIGVM_NATIVES
                                        .lock()
                                        .unwrap()
                                        .peek_qualified(&native_name)
                                        .is_some()
                                    {
                                        Some(native_name)
                                    } else if self.exports.contains_key(&format!(
                                        "{}.{}",
                                        type_name,
                                        method.as_ref()
                                    )) {
                                        // Plan 197 Task 3: User-defined method on chained result
                                        Some(format!("{}.{}", type_name, method.as_ref()))
                                    } else if method.as_str() == "len" {
                                        // Fallback for len() — emit ARRAY_LEN which handles
                                        // List/Array heap objects at runtime without requiring
                                        // a typed String argument (avoids Str.len FFI crash).
                                        self.compile_expr(obj)?;
                                        self.emit(OpCode::ARRAY_LEN);
                                        self.last_expr_type = ObjectType::Int;
                                        return Ok(());
                                    } else {
                                        Some(format!("Unknown_{}", method))
                                    }
                                } // 闭合 Plan 325 enum_method_name 的 else
                            }
                        }
                    }
                    _ => None,
                };

                // PLAN-080 F-2③: 裸 Http web 协议族（Http.get/get_json/post/
                // put/delete/patch，arity 匹配时）——web 轨语义（ts_adapter
                // Plan 028 F8）= await fetch(...).json() 解析体。VM 此前把
                // 裸名经 canonical 规则派到 auto.http.get/post（Response
                // 句柄 int），.at 侧字段读全空、try/catch 静默吞——musk
                // ws_resolve_current/relay_store 等整族"VM 数据空"的根因
                // （080 probe080 实证：Http.get 返 to_str="2"）。改写为
                // *_json native + json.to_value，与 emit_api_http_call
                // 步骤 4-5 同配方；句柄族（Http.request/RequestBuilder 链）
                // 与不匹配 arity 不经此臂，原 CALL 路径不变。
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    if matches!(obj.as_ref(), Expr::Ident(id) if id.as_str() == "Http") {
                        let n_args = call.args.args.len();
                        let web_native = match (method.as_str(), n_args) {
                            ("get", 1) | ("get_json", 1) => Some("auto.http.get_json"),
                            ("post", 2) => Some("auto.http.post_json"),
                            ("put", 2) => Some("auto.http.put_json"),
                            ("patch", 2) => Some("auto.http.patch_json"),
                            ("delete", 1) => Some("auto.http.delete_json"),
                            _ => None,
                        };
                        if let Some(native) = web_native {
                            for arg in &call.args.args {
                                if let crate::ast::Arg::Pos(a) = arg {
                                    self.compile_expr(a)?;
                                }
                            }
                            self.emit_call_nat_by_name(native, n_args as u16)?;
                            // body 字符串 → VM 值（ListData/__json_object）。
                            self.emit_call_nat_by_name("auto.json.to_value", 1)?;
                            self.last_expr_type = ObjectType::NestedObject;
                            self.last_was_native_void = false;
                            return Ok(());
                        }
                    }
                }

                // Plan 340: API-over-HTTP rewriting (VM+VM split mode). When
                // api_over_http is set, rewrite calls to #[api] functions
                // into HTTP requests. Must come BEFORE any native/local/external
                // resolution so the call never reaches the normal CALL path.
                // PLAN-648 T-01: 候选除裸名(Expr::Ident)外,增加模块别名
                // 限定形态 Expr::Dot(Ident, fn)(`api.get_lines()`)——此前
                // 仅裸名可改写,限定调用静默回落 back 链进程内执行(auto-term
                // dev 跑法空屏定因)。db.* 等非 api 限定名不命中 api_funcs,
                // 天然不受影响。
                if self.api_over_http {
                    if let Some(name) = func_name.as_ref() {
                        let rewritable = matches!(call.name.as_ref(), Expr::Ident(_))
                            || matches!(
                                call.name.as_ref(),
                                Expr::Dot(obj, _)
                                    if matches!(obj.as_ref(), Expr::Ident(_))
                            );
                        if rewritable {
                            if let Some(api) = self.api_funcs.get(name).cloned() {
                                return self.emit_api_http_call(&api, call);
                            }
                        }
                    }
                }
                // Plan 060 M3:host 分派 —— merged 模式下宿主桥已注册
                // (ash-runner 进程内起 ash_server worker)时,裸 #[api] 调用
                // 改写为 auto.host.call 直调桥函数(与 HTTP 改写同构:调用点
                // 发射,值经 json.to_value 落栈 —— 规避 .at fn return 无法
                // 携带复合值的 VM 边界限制)。api.at 实现体(调 shell.X 桩)
                // 与 HTTP 模式同样被旁路。
                else if crate::vm::host_bridge::has_host_calls() {
                    if let Some(name) = func_name.as_ref() {
                        if matches!(call.name.as_ref(), Expr::Ident(_)) {
                            if let Some(api) = self.api_funcs.get(name).cloned() {
                                return self.emit_api_host_call(&api, call);
                            }
                        }
                    }
                } else {
                    // PLAN-053 P-053-4: merged 模式下 #[api] no-op 显式告警（一次性）。
                    // PLAN-633: api 实现体已随 back 链扁平编译进本模块（导出存在，
                    // 画廊内嵌/standalone merged 场景）→ 不发 no-op 桩，直落常规
                    // 解析直调编译体（否则 no-op null 与真调用的栈序纠缠，返回值
                    // 被吞——013 内嵌 list_todos 空列表实证）。导出缺席（api 体
                    // 未装载）时保持 no-op 告警原语义。
                    if let Some(name) = func_name.as_ref() {
                        if matches!(call.name.as_ref(), Expr::Ident(_)) {
                            if let Some(api) = self.api_funcs.get(name).cloned() {
                                if !self.exports.contains_key(&api.fn_name) {
                                    self.emit_str_const_push(&api.fn_name);
                                    self.emit_call_nat_by_name("auto.vm.warn_api_noop", 1)?;
                                    self.emit(OpCode::POP);
                                }
                            }
                        }
                    }
                }

                // Plan 197 Task 14: Array.len() emits ARRAY_LEN opcode directly
                if func_name.as_deref() == Some("Array.len") && call.args.args.is_empty() {
                    if let Expr::Dot(obj, _) = call.name.as_ref() {
                        self.compile_expr(obj)?;
                        self.emit(OpCode::ARRAY_LEN);
                        self.last_expr_type = ObjectType::Int;
                        return Ok(());
                    }
                }

                // VM-1: Route float math methods (x.sin(), x.sqrt(), etc.) to auto.math.*
                // Must happen BEFORE native_id lookup so the rewritten name is resolved correctly.
                // Plan 300: Skip routing if the function is a Python FFI import (e.g., use.py math::{sqrt})
                if let Some(ref fname) = func_name {
                    let method = fname.rsplit('.').next().unwrap_or("");
                    let is_py_ffi = self.py_native_map.contains_key(fname)
                        || self.py_native_map.contains_key(method)
                        || self
                            .py_modules
                            .contains(fname.split('.').next().unwrap_or(""));
                    const MATH_METHODS: &[&str] = &[
                        "sin",
                        "cos",
                        "tan",
                        "sqrt",
                        "abs",
                        "floor",
                        "ceil",
                        "round",
                        "pow",
                        "powf",
                        "powi",
                        "exp",
                        "ln",
                        "log2",
                        "log10",
                        "signum",
                        "asin",
                        "acos",
                        "atan",
                        "atan2",
                        "to_radians",
                        "to_degrees",
                    ];
                    if !is_py_ffi
                        && MATH_METHODS.contains(&method)
                        && !fname.starts_with("auto.math.")
                    {
                        let new_name = format!("auto.math.{}", method);
                        let reg = BIGVM_NATIVES.lock().unwrap();
                        if reg.peek_qualified(&new_name).is_some() {
                            func_name = Some(new_name);
                        }
                    }
                }
                // Route bitwise methods (x.and(), x.not(), x.xor(), etc.) to auto.int.*
                // For .or(), only route when receiver is a known integer type,
                // since Option.or has the same method name.
                if let Some(ref fname) = func_name {
                    let method = fname.rsplit('.').next().unwrap_or("");
                    if method == "or" {
                        // Only route .or() to bitwise int.or for known integer types
                        let prefix = fname.split('.').next().unwrap_or("");
                        const INT_TYPE_PREFIXES: &[&str] = &[
                            "int", "uint", "i32", "i64", "u32", "u64", "u8", "byte", "usize",
                            "isize",
                        ];
                        if INT_TYPE_PREFIXES.contains(&prefix) && !fname.starts_with("auto.int.") {
                            let new_name = "auto.int.or".to_string();
                            let reg = BIGVM_NATIVES.lock().unwrap();
                            if reg.peek_qualified(&new_name).is_some() {
                                func_name = Some(new_name);
                            }
                        }
                    } else {
                        const BITWISE_METHODS: &[&str] = &["and", "not", "xor", "shl", "shr"];
                        if BITWISE_METHODS.contains(&method) && !fname.starts_with("auto.int.") {
                            let new_name = format!("auto.int.{}", method);
                            let reg = BIGVM_NATIVES.lock().unwrap();
                            if reg.peek_qualified(&new_name).is_some() {
                                func_name = Some(new_name);
                            }
                        }
                    }
                }

                // Plan 212 Phase 2: Route rand/Rng methods to built-in shims
                if let Some(ref fname) = func_name {
                    let method = fname.rsplit('.').next().unwrap_or("");
                    match method {
                        "thread_rng" => {
                            func_name = Some("auto.rand.thread_rng".to_string());
                        }
                        "gen_range" => {
                            func_name = Some("auto.rng.gen_range".to_string());
                        }
                        "gen" if fname.contains('.') => {
                            func_name = Some("auto.rng.gen".to_string());
                        }
                        "random" => {
                            func_name = Some("auto.rand.random".to_string());
                        }
                        _ => {}
                    }
                }

                // Plan 212 Phase 2: Route log/tracing macros to built-in Log shims
                if let Some(ref fname) = func_name {
                    let method = fname.rsplit('.').next().unwrap_or("").to_string();
                    let bare = fname
                        .rsplit("::")
                        .next()
                        .unwrap_or(fname.as_str())
                        .to_string();
                    match bare.as_str() {
                        "debug" => func_name = Some("Log.debug".to_string()),
                        "info" => func_name = Some("Log.info".to_string()),
                        "warn" => func_name = Some("Log.warn".to_string()),
                        "error" => func_name = Some("Log.error".to_string()),
                        _ => {}
                    }
                    // no-op shims for env_logger/log/tracing configuration
                    if method == "init" || method == "set_max_level" || method == "set_logger" {
                        func_name = Some("auto.log.noop".to_string());
                    }
                }

                // Plan 240: Route std::env/std::fs module methods
                if let Some(ref fname) = func_name {
                    let parts: Vec<String> = fname.splitn(2, '.').map(|s| s.to_string()).collect();
                    if parts.len() == 2 {
                        let module = parts[0].as_str();
                        let method = parts[1].as_str();
                        let routed = match (module, method) {
                            ("env", "get") => Some("auto.env.get".to_string()),
                            ("env", "get_or") => Some("auto.env.get_or".to_string()),
                            ("env", "set") => Some("auto.env.set".to_string()),
                            ("env", "remove") => Some("auto.env.remove".to_string()),
                            ("env", "local_data_dir") => {
                                Some("auto.env.local_data_dir".to_string())
                            }
                            ("env", "home_dir") => Some("auto.env.home_dir".to_string()),
                            // Plan 442 C2: rust-form env::var bridge (Option-
                            // shaped at the producer; caller's .ok() is the
                            // engine's identity passthrough).
                            ("env", "var") => Some("auto.env.var".to_string()),
                            // Plan 401: localStorage-style key/value store for the
                            // VM/iced path (no browser in iced; back it by an
                            // in-process HashMap so settings persist for the
                            // session). vue already implements these in the browser.
                            ("storage", "get") => Some("auto.storage.get".to_string()),
                            ("storage", "set") => Some("auto.storage.set".to_string()),
                            ("storage", "remove") => Some("auto.storage.remove".to_string()),
                            ("time", "now_ms") => Some("auto.time.now_ms".to_string()),
                            ("time", "now_sec") => Some("auto.time.now_sec".to_string()),
                            ("time", "sleep_ms") => Some("auto.time.sleep_ms".to_string()),
                            ("time", "now") => Some("auto.time.now".to_string()),
                            // PLAN-716 组B: 帧时间戳观测（供②——9918/9919 固定
                            // 位；stdlib/auto/frame.vm.at 编译器签名面）。
                            ("frame", "begin_ms") => Some("auto.frame.begin_ms".to_string()),
                            ("frame", "present_ms") => Some("auto.frame.present_ms".to_string()),
                            // Plan 442 A5: one-shot scheduler primitives
                            // (set_timeout/clear_timeout; callback = closure or
                            // event-name string, fires on the render tick).
                            ("sched", "set_timeout") => Some("auto.sched.set_timeout".to_string()),
                            ("sched", "clear_timeout") => {
                                Some("auto.sched.clear_timeout".to_string())
                            }
                            // Plan 442 B-support: JS-shaped localStorage over the
                            // Plan 401 session KV store (getItem -> None when
                            // missing, mirroring the browser semantics musk
                            // sources test against).
                            ("localStorage", "getItem") => {
                                Some("auto.localstorage.get_item".to_string())
                            }
                            ("localStorage", "setItem") => {
                                Some("auto.localstorage.set_item".to_string())
                            }
                            ("localStorage", "removeItem") => {
                                Some("auto.localstorage.remove_item".to_string())
                            }
                            // Plan 442 Phase B: `dom.*` / `location.reload()` web
                            // globals (musk visual_store/settings/app). Desktop
                            // semantics: prefers_dark -> true (iced renderer is
                            // dark-fixed), open_url opens the OS browser,
                            // copy_text reuses the Plan 418 clipboard native,
                            // the rest are recorded no-op stubs.
                            ("dom", "set_dark") => Some("auto.dom.set_dark".to_string()),
                            ("dom", "prefers_dark") => Some("auto.dom.prefers_dark".to_string()),
                            ("dom", "set_css_var") => Some("auto.dom.set_css_var".to_string()),
                            ("dom", "focus_first") => Some("auto.dom.focus_first".to_string()),
                            ("ui", "focus") => Some("auto.ui.focus".to_string()),
                            ("dom", "click_first") => Some("auto.dom.click_first".to_string()),
                            ("dom", "open_url") => Some("auto.dom.open_url".to_string()),
                            ("dom", "copy_text") => Some("auto.clipboard.set_text".to_string()),
                            ("location", "reload") => Some("auto.dom.reload".to_string()),
                            ("io", "read_line") => Some("auto.io.read_line".to_string()),
                            ("io", "say") => Some("auto.print_str".to_string()),
                            ("process", "args") => Some("auto.process.args".to_string()),
                            ("process", "exit") => Some("auto.process.exit".to_string()),
                            ("process", "current_dir") => {
                                Some("auto.process.current_dir".to_string())
                            }
                            ("fs", "read_to_string") | ("fs", "read_text") => {
                                Some("auto.fs.read_text".to_string())
                            }
                            ("fs", "write") | ("fs", "write_all") | ("fs", "write_text") => {
                                Some("auto.fs.write_text".to_string())
                            }
                            ("fs", "create_dir") | ("fs", "create_dir_all") => {
                                Some("auto.fs.create_dir".to_string())
                            }
                            ("fs", "read") | ("fs", "read_bytes") => {
                                Some("auto.fs.read_bytes".to_string())
                            }
                            ("fs", "copy") => Some("auto.fs.copy".to_string()),
                            ("fs", "exists") => Some("auto.fs.exists".to_string()),
                            ("fs", "remove_file") | ("fs", "delete") => {
                                Some("auto.fs.delete".to_string())
                            }
                            ("fs", "remove_dir") => Some("auto.fs.remove_dir".to_string()),
                            ("fs", "remove_dir_all") => Some("auto.fs.remove_dir_all".to_string()),
                            ("fs", "metadata") => Some("auto.fs.size".to_string()),
                            ("fs", "is_dir") => Some("auto.fs.is_dir".to_string()),
                            ("fs", "walk") => Some("auto.file.walk".to_string()),
                            ("fs", "is_binary") => Some("auto.fs.is_binary".to_string()),
                            ("fs", "file_size") => Some("auto.fs.size".to_string()),
                            ("fs", "list_dir") => Some("auto.fs.walk".to_string()),
                            ("fs", "mkdir_all") => Some("auto.fs.create_dir".to_string()),
                            ("fs", "parent_dir") => Some("auto.fs.parent".to_string()),
                            ("fs", "filename") => Some("auto.fs.filename".to_string()),
                            ("fs", "cwd") => Some("auto.process.current_dir".to_string()),
                            ("fs", "append_text") => Some("auto.file.append_text".to_string()),
                            ("fs", "read_lines") => Some("auto.file.read_lines".to_string()),
                            ("json", "parse") => Some("auto.json.parse".to_string()),
                            ("json", "get") => Some("auto.json.get".to_string()),
                            ("json", "get_str") => Some("auto.json.as_string".to_string()),
                            ("json", "as_string") => Some("auto.json.as_string".to_string()),
                            ("json", "encode") | ("json", "to_string") => {
                                Some("auto.json.encode".to_string())
                            }
                            ("json", "decode") => Some("auto.json.decode".to_string()),
                            ("json", "is_valid") => Some("auto.json.is_valid".to_string()),
                            ("json", "has_key") => Some("auto.json.has_key".to_string()),
                            ("json", "as_int") => Some("auto.json.as_int".to_string()),
                            ("json", "as_number") => Some("auto.json.as_number".to_string()),
                            ("json", "as_bool") => Some("auto.json.as_bool".to_string()),
                            ("json", "is_null") => Some("auto.json.is_null".to_string()),
                            ("json", "len") => Some("auto.json.len".to_string()),
                            ("json", "type_of") => Some("auto.json.type_of".to_string()),
                            ("json", "get_at") => Some("auto.json.get_at".to_string()),
                            ("json", "keys") => Some("auto.json.keys".to_string()),
                            // Plan 046 (auto-musk T2): JS-style bare `Object`
                            // qualifier routes to the dynamic-receiver family.
                            ("Object", "keys") => Some("auto.obj.keys".to_string()),
                            ("Object", "values") => Some("auto.obj.values".to_string()),
                            ("shell", "exec") => Some("auto.sys.exec".to_string()),
                            ("regex", "match") => Some("auto.regex.match".to_string()),
                            // PLAN-066 T-12（F-W1）：i18n.t VM 查表路由——
                            // 合成作用域无 i18n 绑定，不路由则接收者编译报
                            // Undefined variable 毒化导出（shim 接
                            // i18n_lookup 查表，PLAN-050 C7）。PLAN-814 T-02
                            // 起 i18n_lookup 注册表随 SDK（hook 供值），shim
                            // 与路由不再随 ui 裁剪——查表 miss 回落 key 本身。
                            ("i18n", "t") => Some("auto.i18n.t".to_string()),
                            // URL module → opaque heap object shims
                            ("url", "parse") => Some("auto.url_opaque.parse".to_string()),
                            ("url", "encode") => Some("auto.url.encode".to_string()),
                            ("url", "decode") => Some("auto.url.decode".to_string()),
                            ("url", "join_path") => Some("auto.url.join_path".to_string()),
                            ("sys", method) => {
                                let cand = format!("auto.sys.{}", method);
                                if BIGVM_NATIVES
                                    .lock()
                                    .unwrap()
                                    .peek_qualified(&cand)
                                    .is_some()
                                {
                                    Some(cand)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };
                        if routed.is_some() {
                            func_name = routed;
                        }
                    }
                }

                // Plan 212 Phase 2.2: Route Regex/Url/Semver opaque struct methods
                // Handles both Type.Method (e.g., "regex::Regex.is_match") and
                // var.method (e.g., "re.is_match") — the latter needs import-based lookup.
                {
                    // Extract owned values to avoid borrow conflicts with func_name assignment
                    let fname_owned = func_name.clone();
                    if let Some(ref fname) = fname_owned {
                        use crate::vm::native_catalog::lookup_opaque_dispatch;
                        // When func_name is "str.matches" but receiver is an opaque crate var,

                        // the type was wrongly inferred as str (from StrFixed(0) return type).
                        // Use opaque_var_crates to route to the correct native.
                        if let Expr::Dot(obj_expr, method_name) = call.name.as_ref() {
                            if let Expr::Ident(receiver_name) = obj_expr.as_ref() {
                                if let Some(crate_name) =
                                    self.opaque_var_crates.get(receiver_name.as_str())
                                {
                                    if let Some(native) =
                                        lookup_opaque_dispatch(crate_name, method_name.as_str())
                                    {
                                        func_name = Some(native.to_string());
                                    }
                                }
                            }
                        }
                        let method = fname.rsplit('.').next().unwrap_or("").to_string();
                        let receiver = fname.rsplit('.').nth(1).unwrap_or("");
                        // Module-level calls (Url.scheme, Regex.new, Version.parse) use native
                        // dispatch via TYPE_CANONICAL_MAP. Only route to opaque when receiver
                        // is a variable name (lowercase start) or an opaque-tracked var.
                        let is_module_call = receiver == "Url"
                            || receiver == "Regex"
                            || receiver == "Version"
                            || receiver == "VersionReq";
                        let has_regex =
                            !is_module_call && (fname.contains("Regex") || fname.contains("regex"));
                        let has_url =
                            !is_module_call && (fname.contains("Url") || fname.contains("url"));
                        let has_version = !is_module_call
                            && fname.contains("Version")
                            && !fname.contains("VersionReq");
                        // Route VersionReq module calls to dedicated opaque natives
                        let mut versionreq_routed = false;
                        if receiver == "VersionReq" {
                            match method.as_str() {
                                "parse" => {
                                    func_name =
                                        Some("auto.semver_opaque_versionreq.parse".to_string());
                                    versionreq_routed = true;
                                }
                                "matches" => {
                                    func_name =
                                        Some("auto.semver_opaque_versionreq.matches".to_string());
                                    versionreq_routed = true;
                                }
                                _ => {}
                            }
                        }
                        // Plan 249 Phase 4: Unified opaque dispatch via native_catalog
                        if has_regex {
                            if let Some(native) = lookup_opaque_dispatch("regex", method.as_str()) {
                                func_name = Some(native.to_string());
                            }
                        }
                        if has_url {
                            if let Some(native) = lookup_opaque_dispatch("url", method.as_str()) {
                                func_name = Some(native.to_string());
                            }
                        }
                        if has_version {
                            if let Some(native) = lookup_opaque_dispatch("semver", method.as_str())
                            {
                                func_name = Some(native.to_string());
                            }
                        }
                        // Fallback: var.method where var was assigned from an opaque constructor
                        // Check rust_native_map and opaque_var_crates for the receiver type
                        if !has_regex && !has_url && !has_version && !versionreq_routed {
                            let obj_part = fname.rsplit('.').nth(1).unwrap_or("");
                            let crate_name = self
                                .opaque_var_crates
                                .get(obj_part)
                                .map(|s| s.as_str())
                                .or_else(|| {
                                    self.rust_native_map
                                        .get(obj_part)
                                        .map(|(cn, _)| cn.as_str())
                                })
                                .unwrap_or("");
                            // Plan 249 Phase 4: Unified opaque dispatch
                            // Only route instance methods (lowercase receiver) for file/stdio ops;
                            // static calls like File.create() should go through CALL_SPEC dispatch.
                            let is_static_receiver = obj_part
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false);
                            if crate_name == "std" && !is_static_receiver {
                                match method.as_str() {
                                    "now" => func_name = Some("auto.time.instant_now".to_string()),
                                    "elapsed" => {
                                        func_name = Some("auto.time.instant_elapsed".to_string())
                                    }
                                    "new" => {
                                        if fname.starts_with("OnceCell") {
                                            func_name = Some("auto.cell.once_new".to_string());
                                        }
                                    }
                                    "get" => func_name = Some("auto.cell.once_get".to_string()),
                                    "set" => func_name = Some("auto.cell.once_set".to_string()),
                                    "create" => {
                                        func_name = Some("auto.file.create_handle".to_string())
                                    }
                                    "open" => func_name = Some("auto.file.open_handle".to_string()),
                                    "write" => {
                                        func_name = Some("auto.file.write_handle".to_string())
                                    }
                                    "try_clone" => {
                                        func_name = Some("auto.file.try_clone".to_string())
                                    }
                                    _ => {}
                                }
                            } else if let Some(native) =
                                lookup_opaque_dispatch(crate_name, method.as_str())
                            {
                                func_name = Some(native.to_string());
                            } else {
                                match method.as_str() {
                                    "elapsed" => {
                                        func_name = Some("auto.time.instant_elapsed".to_string())
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                // Plan 212 Phase 2.3: Route free function calls from built-in shim crates
                // e.g., encode("hello") where encode is from base64 → auto.base64.encode
                if let Some(ref fname) = func_name {
                    if !fname.contains('.') {
                        // Bare function name — check if it belongs to a built-in shim crate
                        if let Some((crate_name, _)) = self.rust_native_map.get(fname.as_str()) {
                            let canonical = match crate_name.as_str() {
                                "base64" => match fname.as_str() {
                                    "encode" => Some("auto.base64.encode".to_string()),
                                    "decode" => Some("auto.base64.decode".to_string()),
                                    _ => None,
                                },
                                "hex" => match fname.as_str() {
                                    "encode" => Some("auto.hex.encode".to_string()),
                                    "decode" => Some("auto.hex.decode".to_string()),
                                    _ => None,
                                },
                                "mime_guess" => match fname.as_str() {
                                    "from_path" => Some("auto.mime.from_path".to_string()),
                                    _ => None,
                                },
                                _ => None,
                            };
                            if let Some(canonical_name) = canonical {
                                func_name = Some(canonical_name);
                            }
                        }
                    }
                }

                // Check if it's a native function (either intrinsic or BIGVM_NATIVE)
                // Plan 369 Task 10: track whether the resolved native is a py-FFI call,
                // so the emit site can use CALL_NAT_COUNTED (carrying runtime arg count) instead
                // of CALL_NAT. Set true in the py_native_map / py_modules branches below.
                let mut is_py_ffi_call = false;
                // Plan 454 E(§M 缺口③·路由半):动态接收者(obj/Object 注解)
                // 的方法族强制走 auto.obj.*——否则被字面量推断成 List.find,
                // 携带 -1 哨兵与 Int 返回型,下游 hit==None/hit.u 全错位。
                if let Expr::Dot(obj_expr, method_name) = call.name.as_ref() {
                    if matches!(method_name.as_str(), "keys" | "values" | "find") {
                        if let Expr::Ident(recv) = obj_expr.as_ref() {
                            let is_dyn = self
                                .var_types
                                .get(recv.as_str())
                                .map(|t| match t {
                                    crate::ast::Type::User(td) => {
                                        let n = td.name.to_string();
                                        let short = n.rsplit('.').next().unwrap_or(&n).to_string();
                                        short == "obj" || short == "Object"
                                    }
                                    _ => false,
                                })
                                .unwrap_or(false);
                            if is_dyn {
                                func_name = Some(format!("auto.obj.{}", method_name));
                            }
                        }
                    }
                }

                // Plan 446 批二 E1/E2(续): http 客户端访问器的 arity 分流。
                // stdlib 表面把 `Response.status` 声明为服务端 setter
                // (self, code int);客户端零参读法 res.status() 会按 setter 签名
                // 弹栈 → 多吃一个栈槽,后续任何语句栈下溢(os-config E2 现场)。
                // 零参读法改路由到 shim 支撑的只读 native;一参 header 读法同理
                // (声明面只有双参 set-form Response.header(key,value))。
                if let Expr::Dot(_, method_name) = call.name.as_ref() {
                    let argc = call.args.args.len();
                    let redirected = match (method_name.as_str(), argc) {
                        ("status", 0) | ("status_code", 0) => {
                            Some("Response.status_code".to_string())
                        }
                        ("header", 1) => Some("Response.header_get".to_string()),
                        _ => None,
                    };
                    if let Some(new_name) = redirected {
                        if BIGVM_NATIVES
                            .lock()
                            .unwrap()
                            .peek_qualified(&new_name)
                            .is_some()
                        {
                            func_name = Some(new_name);
                        }
                    }
                }

                let native_id = if let Some(name) = &func_name {
                    // Check intrinsics first (print, etc.)
                    if let Some(&id) = self.intrinsics.get(name) {
                        Some(id)
                    }
                    // Plan 212b/240: Check rust_native_map FIRST (before BIGVM_NATIVES lock)
                    // Matches both bare names (e.g., "encode") and Type.method (e.g., "Utc.now")
                    // But only route to dispatch 3000 if no existing native is registered.
                    else if self.rust_native_map.contains_key(name)
                        || name.contains('.')
                            && self
                                .rust_native_map
                                .contains_key(name.split('.').next().unwrap_or(""))
                    {
                        // Check if there's already a registered native with an actual shim.
                        //
                        // 跨测试污染修复(ffi_dual_014 发现,2026-08-25):"已有 native
                        // 优先"只允许作用于 **crate/模块级导入**(首字母小写,如
                        // toml.parse、json.parse)。**类型导入**(首字母大写,如
                        // String/Duration/Vec)必须恒走 dispatch 3000——BIGVM_NATIVES
                        // 是惰性注册的进程级全局,同进程里任何先前程序用过原生
                        // API(如 dstr 测试的 String.from)就会把 auto.str.from
                        // 注册进表,此后本进程内所有 use.rust 的 String.from 都被
                        // 劫持到 native 路径,print 出裸堆 ID(4000011 形态)。
                        // 路由决策不得依赖"之前跑过什么"。
                        let matched_key = if self.rust_native_map.contains_key(name) {
                            name.as_str()
                        } else {
                            name.split('.').next().unwrap_or("")
                        };
                        let is_type_import = matched_key
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false);
                        // Plan 442 C2: axum routing fns (get/post/put/delete/
                        // patch from `use.rust axum::routing::{...}`) must
                        // ALWAYS take dispatch 3000 — their lowercase names
                        // collide with lazily-registered stdlib natives
                        // (http.patch etc.), and the "existing native wins"
                        // rule would hijack `patch(h)` to an HTTP-client shim.
                        let is_axum_routing_import = self
                            .rust_native_map
                            .get(matched_key)
                            .map(|(_, full)| full.starts_with("axum::routing::"))
                            .unwrap_or(false);
                        // 注意:此处必须查**注册表**(真实注册过的 native),不能用
                        // peek_qualified 的静态表语义——auto.json.parse 在静态固定
                        // ID 表里但走 CALL_NAT 的返回编组与 dispatch 3000 不同
                        // (cb_encoding_json 实证:peek 劫持后 print 出裸堆 ID)。
                        // dispatch 3000 的运行期兜底链会以正确编组转到同一 native,
                        // 故未注册时恒走 dispatch。toml.parse 这类启动即注册的
                        // native 仍按原语义被采用。
                        // PLAN-596 T4:dep crate 自由函数 + 齐整字面量实参 →
                        // 试 mono 实例名("rust.{fn}__{label}",212 侧生成逐实例 shim)。
                        let mono_label = self
                            .type_is_dep_face(name)
                            .then(|| Self::mono_literal_label(&call.args.args))
                            .flatten();
                        let has_existing = !is_type_import && !is_axum_routing_import && {
                            let reg = BIGVM_NATIVES.lock().unwrap();
                            // PLAN-592: dep 自由函数以 "rust.{func}" 限定名注册
                            // (init_rust_ffi 装载 wrapper cdylib 时)——裸名查不到时
                            // 补查限定名,否则调用退化为 dispatch 3000 的
                            // "Unknown Rust stdlib call"(自由函数不在 METHODS 表)。
                            (reg.resolve_qualified_to_canonical(name).is_some()
                                && reg.get_id(name).is_some())
                                || reg
                                    .resolve_qualified_to_canonical(&format!("rust.{name}"))
                                    .is_some()
                                || mono_label
                                    .map(|l| {
                                        reg.resolve_qualified_to_canonical(&format!(
                                            "rust.{name}__{l}"
                                        ))
                                        .is_some()
                                    })
                                    .unwrap_or(false)
                        };
                        if has_existing {
                            // Use the existing native (e.g., toml.parse, json.parse)
                            let mut reg = BIGVM_NATIVES.lock().unwrap();
                            reg.resolve_qualified(name)
                                .or_else(|| reg.resolve_qualified(&format!("rust.{name}")))
                                .or_else(|| {
                                    mono_label.and_then(|l| {
                                        reg.resolve_qualified(&format!("rust.{name}__{l}"))
                                    })
                                })
                        } else {
                            // Route to dispatch handler for external crate calls
                            Some(NATIVE_RUST_STDLIB_DISPATCH)
                        }
                    }
                    // Plan 214: Check py_native_map for Python FFI functions
                    else if self.py_native_map.contains_key(name) {
                        let qualified = format!("py.{}", name);
                        let mut reg = BIGVM_NATIVES.lock().unwrap();
                        if let Some(id) = reg.resolve_qualified(&qualified) {
                            drop(reg);
                            is_py_ffi_call = true;
                            Some(id)
                        } else {
                            drop(reg);
                            let id = BIGVM_NATIVES.lock().unwrap().register(&qualified);
                            is_py_ffi_call = true;
                            Some(id)
                        }
                    }
                    // Plan 300: Bare py module dot-call (e.g., math.sqrt from `use.py math`)
                    // When func_name is "module.method" and module is a known py module,
                    // dynamically register as "py.module.method" native.
                    //
                    // NOTE: the guard MUST require an actual py-module prefix. A bare
                    // `name.contains('.')` would swallow every dotted method name
                    // (e.g. "List.push", "Array.push", "SomeType.method") and return
                    // None, short-circuiting the BIGVM_NATIVES fallback below — which
                    // is where stdlib list natives like `auto.list.push` get lazily
                    // registered (no list.vm.at exists). Letting non-py dotted names
                    // fall through is what makes `var c = []; c.push(x)` link.
                    else if name.contains('.')
                        && name
                            .splitn(2, '.')
                            .next()
                            .map(|p| self.py_modules.contains(p))
                            .unwrap_or(false)
                    {
                        let qualified = format!("py.{}", name);
                        let mut reg = BIGVM_NATIVES.lock().unwrap();
                        if let Some(id) = reg.resolve_qualified(&qualified) {
                            drop(reg);
                            is_py_ffi_call = true;
                            Some(id)
                        } else {
                            drop(reg);
                            let id = BIGVM_NATIVES.lock().unwrap().register(&qualified);
                            is_py_ffi_call = true;
                            Some(id)
                        }
                    }
                    // Then check BIGVM_NATIVES (List methods, etc.)
                    // Plan 203 Phase 3: resolve_qualified checks qualified registry
                    // first, then falls back to short-name registry internally.
                    // IMPORTANT: Extract lock into separate scope to ensure the MutexGuard
                    // is dropped before the import_scope branch tries to lock again.
                    // A temporary guard in an `else if let` condition lives until the
                    // end of the entire if-else chain, causing deadlock.
                    // PLAN-054 T5 (A7): 宿主对象 Date 直桥 dispatch 3000——
                    // `Date.now`/`Date.format` 是语言级宿主对象调用,不在
                    // use.rust/py/import 链上;此前落 extern stub 返 Nil
                    // (musk 消息时间标签缺失现场,KNOWN-DEBT 051 上游债)。
                    // 尾段统一注入 type/method 串(Plan 192/240 同款),宿主臂
                    // stdlib ("Date","now"/"format") 消费。
                    else if name.contains('.') && name.split('.').next() == Some("Date") {
                        Some(NATIVE_RUST_STDLIB_DISPATCH)
                    } else {
                        let natives_id = {
                            let mut reg = BIGVM_NATIVES.lock().unwrap();
                            reg.resolve_qualified(name)
                        }; // guard dropped here
                        if let Some(id) = natives_id {
                            // Plan 555 T06: 分发组合子走 CALL_NAT_COUNTED 传输形态
                            // （携带调用点实参数字节——组合子 shim 按
                            // pending_native_arg_count 弹参；is_py_ffi_call
                            // 在此仅是"带计数字节的原生调用"发射约定，
                            // 与 py 无耦合）。限定名/裸名皆可命中。
                            if id == crate::vm::interop::NATIVE_INTEROP_OBJ_GET
                                || id == crate::vm::interop::NATIVE_INTEROP_OBJ_SET
                                || id == crate::vm::interop::NATIVE_INTEROP_OBJ_CALL
                                || id == crate::vm::interop::NATIVE_INTEROP_OBJ_LEN
                                || id == crate::vm::interop::NATIVE_INTEROP_OBJ_ITER
                                || id == crate::vm::interop::NATIVE_INTEROP_OBJ_TYPE_NAME
                            {
                                is_py_ffi_call = true;
                            }
                            Some(id)
                        } else if let Some(qualified) = self.import_scope.get(name) {
                            // Plan 347: a user-loaded Auto library must shadow a
                            // same-named native opaque module (e.g. `base64.encode`
                            // from `use auto.base64: encode`). If the qualified
                            // alias's module prefix is a known user Auto module
                            // (registered in `auto_modules`), do NOT route to the
                            // native — let the CALL-with-reloc path below bind to
                            // the user's library via the linker.
                            let is_user_auto_module = qualified
                                .split('.')
                                .next()
                                .map(|mod_name| self.file_modules.contains(mod_name))
                                .unwrap_or(false);
                            if is_user_auto_module {
                                // PLAN-013 T1: a `#[vm]` declaration (stdlib
                                // *.vm.at shim signatures, e.g. term.vm.at via
                                // `use auto.term: …`) has no bytecode body —
                                // shadowing it would bind the call to the
                                // runtime-panic stub instead of the very native
                                // it declares. Body-ful user-library fns keep
                                // the Plan 347 shadow priority unchanged.
                                if self.vm_fn_names.contains(name.as_str()) {
                                    BIGVM_NATIVES.lock().unwrap().resolve_qualified(qualified)
                                } else {
                                    None
                                }
                            } else {
                                BIGVM_NATIVES.lock().unwrap().resolve_qualified(qualified)
                            }
                        } else if let Some(&id) = self.c_ffi_functions.get(name) {
                            Some(id)
                        } else {
                            // PLAN-057 T7（等价性缺陷族③门禁面）：模块式 web 内建
                            // 全局调用（接收者字面 ∈ Math/JSON/Object/Array 且非
                            // 变量/全局）解析失败时升编译期报错——不再落「运行期
                            // 静默 None 桩」（Array.isArray/stringify 掩蔽雷点
                            // 教训）。豁免=调用点同行或上一行
                            // `// vm-safe-allow <原因>`（musk 侧
                            // scripts/vm-safe-lint.mjs 同机制）。
                            // 注意：实例方法的 func_name 会被类型重写带上
                            // "Array." 前缀（xs.frobnicate → Array.frobnicate），
                            // 按原始接收者形态判别不误伤实例方法路径；非四
                            // 命名空间的未解析静态调用维持静默兜底。
                            let is_module_style_web_global = match call.name.as_ref() {
                                Expr::Dot(obj, _) => match obj.as_ref() {
                                    Expr::Ident(id)
                                        if matches!(
                                            id.as_str(),
                                            "Math" | "JSON" | "Object" | "Array"
                                        ) =>
                                    {
                                        self.lookup_var(id.as_str()).is_none()
                                            && !self.global_vars.contains(id.as_str())
                                    }
                                    _ => false,
                                },
                                _ => false,
                            };
                            if is_module_style_web_global && !self.web_builtin_call_exempt() {
                                return Err(AutoError::Msg(format!(
                                    "未解析的 web 内建 `{}`：VM 轨对 natives 表外的 Math.*/JSON.*/Object.*/Array.* 调用不再静默 None。请实现对应 native，或在调用点上一行加 `// vm-safe-allow <原因>` 豁免（musk 侧 scripts/vm-safe-lint.mjs 同机制）",
                                    name
                                )));
                            }
                            None
                        }
                    }
                    // Plan 216 Phase 2: Check c_ffi_functions for C FFI functions (removed — merged above)
                    // else if let Some(&id) = self.c_ffi_functions.get(name) {
                    //     Some(id)
                    // }
                    // else {
                    //     None
                    // }
                    // NOTE: The original code had these as separate else-if branches:
                    //   else if let Some(id) = BIGVM_NATIVES.lock()... { Some(id) }
                    //   else if let Some(qualified) = self.import_scope.get(name) { ... }
                    //   else if let Some(&id) = self.c_ffi_functions.get(name) { ... }
                    //   else { None }
                    // But the MutexGuard temporary in the first else-if extends to the
                    // end of the entire if-else chain, so the import_scope branch would
                    // deadlock trying to re-acquire the lock.
                } else {
                    None
                };
                // DEBUG
                if let Expr::Dot(_, m) = call.name.as_ref() {
                    if m == "push" || m == "len" {}
                }

                // Plan 118 Phase 7: Check if calling a closure variable
                // If call.name is Ident and the variable has Fn type, use CALL_CLOSURE
                let is_closure_call = if let Expr::Ident(name) = call.name.as_ref() {
                    let name_str = name.to_string();
                    if let Some(var_type) = self.var_types.get(&name_str) {
                        matches!(var_type, Type::Fn(_, _))
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_closure_call {
                    // Closure variable call: load closure_id, then use CALL_CLOSURE
                    // Stack layout for CALL_CLOSURE: [..., arg1, arg2, closure_id]
                    // (closure_id should be on top of stack when CALL_CLOSURE executes)
                    if let Expr::Ident(name) = call.name.as_ref() {
                        let name_str = name.to_string();
                        vm_debug!("DEBUG: Closure variable call: {}", name_str);

                        // Compile arguments FIRST (push them to stack)
                        for arg in &call.args.args {
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Pair(_, expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Name(name) => {
                                    self.compile_expr(&Expr::Ident(name.clone()))?;
                                }
                            }
                        }

                        // Load the closure_id from the variable LAST (so it's on top)
                        if let Some(var_index) = self.lookup_var(&name_str) {
                            self.emit_load_loc(var_index);
                        } else {
                            return Err(AutoError::Msg(format!(
                                "Undefined closure variable: {}",
                                name_str
                            )));
                        }

                        // Emit CALL_CLOSURE with arg_count
                        self.emit(OpCode::CALL_CLOSURE);
                        self.code.push(call.args.args.len() as u8);

                        // Skip the rest of the function call logic
                        return Ok(());
                    }
                }

                if let Some(id) = native_id {
                    // Native function call
                    // For instance methods, compile receiver (self) FIRST, then arguments
                    // This ensures stack order: [self, arg1, arg2, ...]
                    vm_debug!(
                        "DEBUG: Native function call: func_name={:?}, native_id={}",
                        func_name,
                        id
                    );
                    if let Expr::Dot(obj, _method) = call.name.as_ref() {
                        // Check if it's a static method call (Type.method with capital T)
                        // Also treat stdlib module names (env, fs, etc.) as static
                        let is_static_method = match obj.as_ref() {
                            Expr::Ident(obj_name) => {
                                // If the identifier resolves to a variable (local OR
                                // module-level global), it's an instance method call on
                                // that value — not a static call. This MUST be checked
                                // before the uppercase-heuristic below: a global named
                                // `H0` or `ABC` starts with an uppercase letter and would
                                // otherwise be misclassified as a static type reference,
                                // causing the receiver to never be compiled (so the
                                // method's `self` argument would pop garbage off the
                                // stack). See Plan 347 (sha2 global-var bitop bug).
                                if self.lookup_var(obj_name.as_str()).is_some()
                                    || self.global_vars.contains(obj_name.as_str())
                                {
                                    false
                                } else {
                                    let lower = obj_name.as_ref();
                                    // Plan 437 §0.6.H③："math" 补入静态模块白名单——
                                    // 此前缺位使 math.cos(a) 走实例路径，receiver 经
                                    // Ident("math") 模块兜底推 const.i32 0 占位，
                                    // math shim 只弹实参不消费占位 → 每次调用
                                    // 泄漏 +1 槽（循环体内累积；还会顶高 sp 干扰
                                    // list.new 族 bp+num_locals+2 有参判定启发式）。
                                    // PLAN-057 T6：JS web 内建全局（Array/JSON/
                                    // Object/Math）补入——Array.isArray 走实例路径
                                    // 时类型名 receiver 被推栈，shim 弹到字符串
                                    // 索引（实测评价值 0..N 递增=类型名池位）。
                                    // PLAN-588（Stage B P-4，583 残留债①）：file
                                    // 补入——file.read_text 等模块式调用走实例
                                    // 路径时 Ident("file") 占位 const.i32 0 无人
                                    // 消费，循环体每调用净漏 +1 槽且垫入表达式
                                    // 栈（`a = a + f(file.read_text(p))` 的 ADD
                                    // 吃到 [占位 0, 结果] 而非 [a, 结果]——首轮
                                    // "正确"纯属 a=0 巧合；m12/m16/kanban
                                    // source_root 同族）。
                                    // PLAN-066 T-12（F-W1）：i18n 补入——静态分支
                                    // 已路由 i18n.t→auto.i18n.t（id2460 惰性注册
                                    // 必解析成功），但本表漏项使 receiver Ident
                                    // ("i18n") 仍走实例编译→Undefined variable→
                                    // computed 导出毒化（WikiNav_dropText/
                                    // MentionInput 标签×2 实证）。i18n 在合成
                                    // 作用域无绑定（composable facade，063），
                                    // 永远不该编译 receiver。
                                    matches!(
                                        lower,
                                        "env"
                                            | "fs"
                                            | "json"
                                            | "http"
                                            | "url"
                                            | "shell"
                                            | "regex"
                                            | "host"
                                            | "math"
                                            | "sys"
                                            | "file"
                                            | "i18n"
                                            | "Array"
                                            | "Object"
                                            | "JSON"
                                            | "Math"
                                            | "Date"
                                    ) || self.is_type_name_heuristic(obj_name)
                                        || self.is_type(obj_name)
                                }
                            }
                            _ => false,
                        };
                        vm_debug!("DEBUG: is_static_method={}", is_static_method);

                        // Compile receiver for instance methods
                        if !is_static_method {
                            vm_debug!("DEBUG: Compiling receiver for instance method");
                            // Check if this method needs 'id' field extraction
                            if let Some(ref method_name) = func_name {
                                vm_debug!(
                                    "DEBUG: method_name={}, needs_id_extraction={}",
                                    method_name,
                                    self.needs_id_extraction(method_name)
                                );
                                if self.needs_id_extraction(method_name) {
                                    // Compile object expression
                                    self.compile_expr(obj)?;

                                    // Extract 'id' field using GET_FIELD
                                    let field_str = "id".to_string();
                                    let field_bytes = field_str.as_bytes().to_vec();
                                    let field_idx = self.strings.len() as u32;
                                    self.strings.push(field_bytes);

                                    self.emit(OpCode::GET_FIELD);
                                    self.code.extend_from_slice(&field_idx.to_le_bytes());
                                } else {
                                    // Compile full instance (for user-defined types)
                                    vm_debug!("DEBUG: Compiling object expr (no id extraction)");
                                    self.compile_expr(obj)?;
                                }
                            } else {
                                self.compile_expr(obj)?;
                            }
                        }
                    }

                    // Plan 127 / Plan 390 §5 Phase A (M1): Task.spawn handling.
                    // `Task.spawn("Name", cap, init1, init2, ...)` — the first arg is the
                    // task type name as a string literal, the second is capacity, and any
                    // further args are positional initializers for the task's state fields
                    // (declaration order). We consume ALL args here (the generic user-arg
                    // loop below is skipped via skip_task_spawn_user_args) and push a
                    // self-describing stack for shim_task_spawn_vm:
                    //   bottom→top: [task_type_str, capacity, n_init, initN, ..., init0]
                    // The shim pops n_init first (top), then the n_init values, then
                    // capacity, then task_type — and writes the init values into the
                    // spawned task's state_vars, locking them so the #start hook's
                    // default initializers don't clobber them.
                    let mut skip_task_spawn_user_args = false;
                    if func_name.as_deref() == Some("Task.spawn") {
                        // task type name = call.args[0] as a string literal.
                        if let Some(crate::ast::Arg::Pos(Expr::Str(task_type))) =
                            call.args.args.first().cloned()
                        {
                            // Push task_type string (bottom of stack).
                            let task_type_str = task_type.to_string();
                            let task_type_bytes = task_type_str.as_bytes().to_vec();
                            let str_idx = self.strings.len() as u32;
                            self.strings.push(task_type_bytes);
                            self.emit(OpCode::LOAD_STR);
                            self.code.extend_from_slice(&str_idx.to_le_bytes());

                            // capacity = call.args[1] (literal int) or default 64.
                            let capacity_val = match call.args.args.get(1) {
                                Some(crate::ast::Arg::Pos(Expr::Int(c))) => *c,
                                _ => 64,
                            };

                            // init args = call.args[2..], pushed right-to-left so the
                            // shim pops field 0 first.
                            // Push capacity (above task_type).
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(capacity_val);
                            // Push init args right-to-left (so field 0 ends on top).
                            let init_args: Vec<&crate::ast::Arg> = call
                                .args
                                .args
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| *i >= 2)
                                .map(|(_, a)| a)
                                .collect();
                            for arg in init_args.iter().rev() {
                                if let crate::ast::Arg::Pos(expr) = arg {
                                    self.compile_expr(expr)?;
                                }
                            }
                            // Push n_init count (top) — self-describing.
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(init_args.len() as i32);

                            skip_task_spawn_user_args = true;
                            vm_debug!(
                                "DEBUG: Task.spawn '{}' capacity={} init_args={}",
                                task_type_str,
                                capacity_val,
                                init_args.len()
                            );
                        }
                    }

                    // Plan 127: Special handling for Task.send - inject task_type for singleton tasks
                    if func_name.as_deref() == Some("auto.task.singleton_send") {
                        // Get the task type name from the original Dot expression
                        if let Expr::Dot(obj, _) = call.name.as_ref() {
                            if let Expr::Ident(task_type) = obj.as_ref() {
                                // Push task_type as string (first arg)
                                let task_type_str = task_type.to_string();
                                let task_type_bytes = task_type_str.as_bytes().to_vec();
                                let str_idx = self.strings.len() as u32;
                                self.strings.push(task_type_bytes);
                                self.emit(OpCode::LOAD_STR);
                                self.code.extend_from_slice(&str_idx.to_le_bytes());
                                vm_debug!(
                                    "DEBUG: Injected task_type='{}' for Task.send",
                                    task_type_str
                                );
                            }
                        }
                    }

                    // Plan 212 Phase 2: Log no-op — skip arg compilation + CALL_NAT entirely.
                    // env_logger.init(), log.set_max_level(), etc. have no VM-side effect.
                    if func_name.as_deref() == Some("auto.log.noop") {
                        self.last_expr_type = ObjectType::Void;
                        self.last_was_native_void = true;
                        return Ok(());
                    }

                    // Plan 539: all py special call forms (py_call kwargs /
                    // py_with inline bracket / item-import kwargs) dispatch
                    // through ONE helper call — this hot recursive arm's
                    // stack frame must stay minimal (the inline versions
                    // overflowed the deeply recursive aavm2 corpus, T09/T19).
                    if is_py_ffi_call && self.try_py_call_special_form(call)? {
                        return Ok(());
                    } else
                    // Compile arguments (left-to-right)
                    // Plan 088 Phase 4: Smart parameter passing for native functions
                    if !call.args.is_empty() && !skip_task_spawn_user_args {
                        let func_name_for_params =
                            func_name.as_ref().map(|s| s.as_str()).unwrap_or("");
                        let is_assert_msg = func_name_for_params == "assert_eq"
                            || func_name_for_params == "assert_ne";
                        let max_args = if is_assert_msg && call.args.args.len() > 2 {
                            2 // Skip the 3rd arg (message string) for assert_eq/ne
                        } else {
                            call.args.args.len()
                        };
                        // PLAN-746 (PG-MEM-2): reject keyword-argument-looking
                        // shapes on the builtin print family at compile time.
                        // The parser turns `end=" "` into a Store (reassignment)
                        // arg, whose codegen emits value+dup+store — the print
                        // shim pops only the top, leaking slots every call;
                        // inside a loop the imbalance corrupts sp-relative
                        // locals (runaway infinite loop) and grows the output
                        // buffer ~3MB/s (walkthrough evidence: think-python
                        // ch07 fences +210/+348MB).
                        let kw_like_arg = call.args.args.iter().find_map(|a| match a {
                            crate::ast::Arg::Pair(k, _) => Some(k.to_string()),
                            // Parser turns `end=" "` into a Bina(Asn) assignment
                            // expression arg (dup+store codegen, stack leak).
                            crate::ast::Arg::Pos(crate::ast::Expr::Bina(lhs, op, _))
                                if *op == Op::Asn =>
                            {
                                Some(format!("{:?}", lhs))
                            }
                            _ => None,
                        });
                        if let Some(kw) = kw_like_arg {
                            let name = func_name_for_params;
                            if name == "print"
                                || name.starts_with("print")
                                || name == "write"
                                || name == "say"
                                || name.starts_with("assert")
                            {
                                return Err(crate::error::AutoError::Msg(format!(
                                    "`{}` does not accept keyword arguments (got `={}`? Auto print has no keyword arguments)",
                                    name, kw
                                )));
                            }
                        }
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i >= max_args {
                                break; // Skip extra args (e.g., assert_eq message)
                            }
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    // Use smart parameter passing if we have function info
                                    if !func_name_for_params.is_empty()
                                        && self.fn_params.contains_key(func_name_for_params)
                                    {
                                        self.compile_call_arg(expr, func_name_for_params, i)?;
                                    } else {
                                        // No parameter info, compile normally
                                        self.compile_expr(expr)?;
                                    }
                                }
                                _ => {
                                    // Arg::Name — compile as positional identifier (e.g., print(x))
                                    // Plan 442 C2: Arg::Pair (named arg, e.g. relay_flows'
                                    // variadic calls) previously hit unreachable here —
                                    // compile the VALUE; the name carries no runtime
                                    // slot in this variadic context.
                                    match arg {
                                        crate::ast::Arg::Name(n) => {
                                            self.compile_expr(&Expr::Ident(n.clone()))?;
                                        }
                                        crate::ast::Arg::Pair(_, expr) => {
                                            if !func_name_for_params.is_empty()
                                                && self.fn_params.contains_key(func_name_for_params)
                                            {
                                                self.compile_call_arg(
                                                    expr,
                                                    func_name_for_params,
                                                    i,
                                                )?;
                                            } else {
                                                self.compile_expr(expr)?;
                                            }
                                        }
                                        crate::ast::Arg::Pos(_) => unreachable!(),
                                    }
                                }
                            }
                        }
                    }

                    // Plan 197 Task 6: str.slice(start) with 1 arg needs end = str.len()
                    // Compile the receiver again and call str.len to push the end arg
                    if func_name.as_deref() == Some("str.slice") && call.args.args.len() == 1 {
                        if let Expr::Dot(obj, _method) = call.name.as_ref() {
                            // Compile receiver again (duplicate of the string on stack)
                            self.compile_expr(obj)?;
                            // Call str.len to get the string length as the end index
                            self.emit(OpCode::CALL_NAT);
                            self.emit_u16(
                                BIGVM_NATIVES
                                    .lock()
                                    .unwrap()
                                    .resolve_qualified("auto.str.len")
                                    .unwrap(),
                            );
                        }
                    }

                    // PLAN-057 T6: JSON.stringify 参数规整——JS 可变参
                    // (v[, replacer][, space])，CALL_NAT shim 固定弹 3 参；
                    // 缺位编译期补齐（replacer=null、space=0），沿 Plan 197
                    // str.slice 补参先例。超参（>3）沿 JS 忽略语义不管——
                    // shim 只弹 3，多参调用点会失衡，现源无此形态。
                    if func_name.as_deref() == Some("JSON.stringify") {
                        match call.args.args.len() {
                            1 => {
                                self.emit(OpCode::PUSH_NIL);
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(0);
                            }
                            2 => {
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(0);
                            }
                            _ => {}
                        }
                    }

                    // PLAN-066 T-05: Regex.match 两参调用编译期补 flags=""——
                    // shim 固定弹 3 参 [flags, pattern, text]（JS match 双形态
                    // 统一：无 'g' = 非 global 组提取语义）。沿 057 T6
                    // JSON.stringify 补参先例。
                    if func_name.as_deref() == Some("Regex.match") && call.args.args.len() == 2 {
                        self.emit_str_const_push("");
                    }

                    // Plan 192/240: Inject implicit type_name and method for Rust stdlib dispatch
                    // Push AFTER user args so type_name/method are on top of stack.
                    // Handler pops method first (top), then type_name (next).
                    if id == NATIVE_RUST_STDLIB_DISPATCH {
                        let (type_str, method_str) =
                            if let Expr::Dot(obj, method_name) = call.name.as_ref() {
                                // Use resolved type name from func_name (e.g., "Command" from "Command.arg")
                                // instead of raw source identifier (e.g., "cmd" from cmd.arg)
                                let resolved_type = func_name
                                    .as_ref()
                                    .and_then(|fn_name| fn_name.split('.').next().map(String::from))
                                    .unwrap_or_default();
                                let t = if resolved_type
                                    .chars()
                                    .next()
                                    .map(|c| c.is_uppercase())
                                    .unwrap_or(false)
                                {
                                    resolved_type
                                } else if let Expr::Ident(type_name_ident) = obj.as_ref() {
                                    type_name_ident.to_string()
                                } else {
                                    String::new()
                                };
                                (t, method_name.to_string())
                            } else if let Expr::Ident(name) = call.name.as_ref() {
                                (String::new(), name.to_string())
                            } else {
                                (String::new(), String::new())
                            };

                        // PLAN-596 T4:dep 侧方法 + 齐整字面量实参 → method 带
                        // mono 实例后缀("__i64"/"__String",方法包按实例条目注册);
                        // dispatch 兜底有剥后缀重试,非泛型方法误后缀无害(多一次 miss)。
                        let method_str = if self.type_is_dep_face(&type_str) {
                            match Self::mono_literal_label(&call.args.args) {
                                Some(label) => format!("{method_str}__{label}"),
                                None => method_str,
                            }
                        } else {
                            method_str
                        };
                        let type_bytes = type_str.as_bytes().to_vec();
                        let type_idx = self.strings.len() as u32;
                        self.strings.push(type_bytes);
                        self.emit(OpCode::LOAD_STR);
                        self.code.extend_from_slice(&type_idx.to_le_bytes());

                        let method_bytes = method_str.as_bytes().to_vec();
                        let method_idx = self.strings.len() as u32;
                        self.strings.push(method_bytes);
                        self.emit(OpCode::LOAD_STR);
                        self.code.extend_from_slice(&method_idx.to_le_bytes());
                    }

                    // Plan 377 §3.1: print() 统一路由到 NATIVE_PRINT_UNIFIED（单槽按 tag 解码）。
                    // 取代 Plan 178/378 的按 ObjectType 分发（I32/F32/F64/U64）—— 单槽化后
                    // 无需编译期推断类型选 print 入口，shim_print_unified 运行时按 tag 解码。
                    // 注：显式的 print_i32()/print_f32() 等调用仍保留各自 id（走 else 分支）。
                    let resolved_id = if id == NATIVE_PRINT_STR {
                        NATIVE_PRINT_UNIFIED
                    } else {
                        id
                    };

                    // Plan 369 Task 10: py-FFI calls use CALL_NAT_COUNTED which carries the
                    // call-site arg count as an extra byte, so the Python shim pops
                    // the ACTUAL number of args (count cannot be introspected for
                    // C builtins like datetime.date, and struct.pack is variadic).
                    // PLAN-656 F-3 续：scroll 族（9900-9905）同样走 COUNTED——
                    // 可选轴实参依赖 pending_native_arg_count 辨识调用形状
                    //（plain CALL_NAT 不携带 arity，多弹/漏弹破坏栈平衡）。
                    if is_py_ffi_call || (9900..=9905).contains(&resolved_id) {
                        self.emit(OpCode::CALL_NAT_COUNTED);
                        self.emit_u16(resolved_id);
                        self.code.push(call.args.args.len().min(255) as u8);
                    } else {
                        self.emit(OpCode::CALL_NAT);
                        self.emit_u16(resolved_id);
                    }

                    // Track return type for type-aware dispatch (e.g., print choosing STR vs I32)
                    if let Some(ref name) = func_name {
                        // Plan 569 D2: 重置纪律——原生调用结果的 may_py 默认 false
                        // （实参编译的粘性位不得泄漏到调用结果），py 臂下方对照置 true。
                        self.last_expr_may_py = false;
                        if name.starts_with("print")
                            || name == "write"
                            || name == "say"
                            || name.starts_with("assert")
                        {
                            self.last_expr_type = ObjectType::Void;
                            self.last_was_native_void = true;
                        } else if name == "system" {
                            // Plan 011: system(cmd) -> String
                            self.last_expr_type = ObjectType::String;
                        } else if name == "shell_query" {
                            // PLAN-082: shell_query(cmd) -> List of records
                            // (heap ObjectData/ListData payloads). Array typing
                            // routes var-stored receivers to ARRAY_LEN/list
                            // natives — str-typed dispatch would stringify the
                            // bare heap id (spike probe: len()==7 on
                            // "4194307"-shaped ids).
                            self.last_expr_type = ObjectType::Array;
                        } else if name == "shell_run" {
                            // PLAN-082: shell_run(cmd) -> void (host renders)
                            self.last_expr_type = ObjectType::Void;
                            self.last_was_native_void = true;
                        } else if name == "code_editor_text" {
                            // Plan 413: code_editor_text(key) -> String
                            self.last_expr_type = ObjectType::String;
                        } else if name == "code_editor_delta" {
                            // Plan 673 T-01: code_editor_delta(key) -> String (JSON)
                            self.last_expr_type = ObjectType::String;
                        } else if name == "code_editor_cursor_line"
                            || name == "code_editor_cursor_col"
                            || name == "code_editor_selection_len"
                        {
                            // Plan 413: cursor/selection accessors -> Int
                            self.last_expr_type = ObjectType::Int;
                        } else if name == "code_editor_find" {
                            // Plan 413: find(key) -> Bool
                            self.last_expr_type = ObjectType::Bool;
                        } else if name == "code_editor_edit" {
                            // Plan 673 T-02: code_editor_edit(key, start, end, replacement) -> Bool
                            self.last_expr_type = ObjectType::Bool;
                        } else if name == "code_editor_load_file" {
                            // PLAN-687: code_editor_load_file(key, path) -> Int (bytes; -1 err)
                            self.last_expr_type = ObjectType::Int;
                        } else if name == "console_lines" {
                            // Console panel: console_lines() -> String
                            self.last_expr_type = ObjectType::String;
                        } else if name == "console_log" || name == "console_clear" {
                            // Console panel: log/clear -> Bool
                            self.last_expr_type = ObjectType::Bool;
                        } else if name == "system_status" {
                            // Plan 011: system_status() -> Int
                            self.last_expr_type = ObjectType::Int;
                        } else if name == "exit" || name == "export" {
                            // Plan 011: void
                            self.last_expr_type = ObjectType::Void;
                            self.last_was_native_void = true;
                        } else if name.ends_with(".to_hex")
                            || name.ends_with(".to_str")
                            || name.ends_with(".str")
                            || name == "int_str"
                            || name.starts_with("auto.url_opaque.scheme")
                            || name.starts_with("auto.url_opaque.host_str")
                            || name.starts_with("auto.url_opaque.path")
                            || name.starts_with("auto.url_opaque.query")
                            || name.starts_with("auto.url_opaque.fragment")
                            || name.starts_with("auto.url_opaque.origin")
                            || name.starts_with("auto.url_opaque.to_string")
                        {
                            self.last_expr_type = ObjectType::String;
                        } else if name.starts_with("auto.obj.") || name.starts_with("obj.") {
                            // Plan 454 E(§M 缺口③·型别半):优先于 fn_return_types
                            // 的通用注册表默认值,否则 values/find 被标 Int 退化。
                            self.last_expr_type = self.infer_native_return_type(name);
                        } else if let Some(ret_ty) = self.fn_return_types.get(name) {
                            self.last_expr_type = match ret_ty {
                                Type::Void => ObjectType::Void,
                                Type::Float => ObjectType::Float,
                                Type::Double => ObjectType::Double,
                                Type::StrFixed(_)
                                | Type::StrOwned
                                | Type::CStrLit
                                | Type::StrSlice => ObjectType::String,
                                Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                                Type::Byte => ObjectType::Byte,
                                Type::Bool => ObjectType::Bool,
                                // Plan 378: I64 occupies 2 slots on the stack
                                // (same layout as U64), so map it to Uint for the
                                // 2-slot print/load/store paths. Int stays 1-slot.
                                Type::I64 => ObjectType::Uint,
                                Type::Int => ObjectType::Int,
                                _ => ObjectType::NestedObject,
                            };
                        } else {
                            // Fallback: infer return type from native name suffix
                            // to avoid stale last_expr_type from argument compilation
                            self.last_expr_type = self.infer_native_return_type(name);
                            self.last_was_native_void =
                                matches!(self.last_expr_type, ObjectType::Void);
                        }
                        // List HOF methods return arrays (tracked for type-aware dispatch)
                        if name == "List.map" || name == "List.filter" {
                            self.last_expr_type = ObjectType::Array;
                        }
                        // Plan 212 Phase 2.1: Rust FFI return type from fn_return_types
                        if self.rust_native_map.contains_key(name) {
                            self.last_expr_type = match self.fn_return_types.get(name) {
                                Some(Type::Void) => ObjectType::Void,
                                Some(Type::Bool) => ObjectType::Bool,
                                Some(Type::Int) => ObjectType::Int,
                                Some(Type::I64) => ObjectType::Int,
                                Some(Type::Float) => ObjectType::Float,
                                Some(Type::Double) => ObjectType::Double,
                                Some(Type::Uint | Type::U64) => ObjectType::Uint,
                                _ => ObjectType::String,
                            };
                            if matches!(self.last_expr_type, ObjectType::Void) {
                                self.last_was_native_void = true;
                            }
                        }
                        // Plan 214/222: Python FFI functions — type-aware return tracking
                        if self.py_native_map.contains_key(name) {
                            use crate::py_ffi_types::PyType;
                            let py_ret = self
                                .py_return_types
                                .get(name)
                                .cloned()
                                .unwrap_or(PyType::Auto);
                            self.last_expr_type = match py_ret {
                                PyType::Int => ObjectType::Int,
                                PyType::Float => ObjectType::Double,
                                PyType::Bool => ObjectType::Bool,
                                PyType::List => ObjectType::Array,
                                PyType::None => ObjectType::Void,
                                // Plan 567 T16: nullable 值可为 null——
                                // 非确定标量（消费面自判空）。
                                PyType::Nullable(_) => ObjectType::Void,
                                // Auto and String both use string pool
                                PyType::Auto | PyType::String => ObjectType::String,
                            };
                            // Plan 569 D2: py-ffi 调用点置位（侧表源①）——py 桥
                            // 结果按运行期 tag 分派，不进 str.* 静态路由。
                            self.last_expr_may_py = true;
                        } else if name.contains('.')
                            && name
                                .split('.')
                                .next()
                                .map(|p| self.py_modules.contains(p))
                                .unwrap_or(false)
                        {
                            // Plan 569 D2: py 模块点调（`use.py math` 后 math.sqrt）
                            // 结果同置 may_py——py_modules 臂不经 py_native_map。
                            self.last_expr_may_py = true;
                        }
                    }

                    return Ok(()).into();
                }

                // Normal Function Call (user-defined)
                // Plan 087 Phase 3 + Plan 088 Phase 4: Instance method receiver as first argument
                let mut is_instance_method_call = false;
                let mut is_spec_dispatch = false;

                if let Expr::Dot(obj, _method) = call.name.as_ref() {
                    // Check if it's a static method call (Type.method with capital T)
                    // Also treat stdlib module names as static
                    let is_static_method = match obj.as_ref() {
                        Expr::Ident(obj_name) => {
                            // Plan 348 Task 20: a module-level global var (or an
                            // imported value name) is an INSTANCE receiver, never a
                            // static type — even when its name starts uppercase (e.g.
                            // `var PAT str = "hello"` then `PAT.len()`). Without this
                            // guard, is_type_name_heuristic misclassifies the global
                            // as a type reference, so the receiver is never loaded and
                            // the method dispatches against garbage (e.g. returning the
                            // length of the variable NAME instead of its value).
                            // Mirrors the Plan 347 guard on the native-call path above.
                            if self.lookup_var(obj_name.as_str()).is_some()
                                || self.global_vars.contains(obj_name.as_str())
                                || self.import_scope.contains_key(obj_name.as_ref())
                            {
                                false
                            } else {
                                let lower = obj_name.as_ref();
                                matches!(
                                    lower,
                                    "env" | "fs" | "process" | "path" | "time" | "math" | "sys" | "log" | "rand" | "json" | "url" | "regex" | "base64" | "hex" | "http" | "shell"
                                    | "env_logger" | "chrono" | "serde_json" | "csv" | "walkdir" | "clap" | "simplelog"
                                    | "crossbeam" | "rayon" | "num" | "percent_encoding" | "urlencoding"
                                    | "sha2" | "hmac" | "flate2" | "tar" | "semver" | "once_cell"
                                    | "rand_distr"
                                    // PLAN-057 T6：JS web 内建全局（防未解析静态
                                    // 兜底推类型名 receiver 占位栈）
                                    | "Array" | "Object" | "JSON" | "Math" | "Date"
                                ) || self.is_type_name_heuristic(obj_name)
                                    || self.is_type(obj_name)
                            }
                        }
                        // Plan 322: GenName("List<Item>") is a static type name, not
                        // an instance. Without this, List<Item>.new([]) compiles the
                        // GenName as a receiver expression (creating an Item struct
                        // instance via CONSTRUCT_INSTANCE) instead of treating it as
                        // a static call to List.new (shim_list_new).
                        Expr::GenName(_) => true,
                        _ => false,
                    };

                    // For instance methods, treat receiver as first argument
                    if !is_static_method {
                        is_instance_method_call = true;

                        // Check if receiver's declared type is a spec (for dynamic dispatch)
                        if let Expr::Ident(var_name) = obj.as_ref() {
                            if let Some(ty) = self.var_types.get(var_name.to_string().as_str()) {
                                if matches!(ty, Type::Spec(_)) {
                                    is_spec_dispatch = true;
                                }
                                // Plan 417-E3 (DIV-TRAIT-VM-1): a receiver whose
                                // static type is one of the enclosing fn's type
                                // parameters (`fn max_of<T has C>(a T, ...)`) has
                                // no static `T.method` symbol — dispatch on the
                                // runtime heap tag via CALL_SPEC instead.
                                if let Type::User(td) = ty {
                                    if self.current_fn_type_params.contains(&td.name.to_string()) {
                                        is_spec_dispatch = true;
                                    }
                                }
                            }
                        }

                        // Plan 088 Phase 4: Compile receiver as first argument (index 0)
                        if let Some(ref method_name) = func_name {
                            vm_debug!(
                                "DEBUG: Compiling instance method call: receiver is arg 0 for '{}'",
                                method_name
                            );

                            // Check if this method needs 'id' field extraction
                            if self.needs_id_extraction(method_name) {
                                // Compile object expression
                                self.compile_expr(obj)?;

                                // Extract 'id' field using GET_FIELD
                                let field_str = "id".to_string();
                                let field_bytes = field_str.as_bytes().to_vec();
                                let field_idx = self.strings.len() as u32;
                                self.strings.push(field_bytes);

                                self.emit(OpCode::GET_FIELD);
                                self.code.extend_from_slice(&field_idx.to_le_bytes());
                            } else {
                                // Plan 088 Phase 4: Smart parameter passing for receiver
                                // Use compile_call_arg to support Copy/View/Mut/Take modes

                                // Receiver (arg 0) is always an object ID (i32 value),
                                // so always use direct compile_expr instead of smart param passing
                                // (LOAD_REF would push var_index instead of the actual object ID)
                                // Exception: crate_module.Type.method calls (e.g., walkdir.WalkDir.new)
                                // need the type name pushed as string receiver for CALL_SPEC dispatch
                                let is_crate_static = if let Expr::Call(inner_call) = obj.as_ref() {
                                    if let Expr::Dot(inner_obj, _inner_method) =
                                        inner_call.name.as_ref()
                                    {
                                        if let Expr::Dot(module_obj, type_field) =
                                            inner_obj.as_ref()
                                        {
                                            if let Expr::Ident(mn) = module_obj.as_ref() {
                                                const CM: &[&str] = &[
                                                    "env",
                                                    "fs",
                                                    "json",
                                                    "http",
                                                    "url",
                                                    "shell",
                                                    "regex",
                                                    "chrono",
                                                    "serde_json",
                                                    "csv",
                                                    "walkdir",
                                                    "clap",
                                                    "simplelog",
                                                    "crossbeam",
                                                    "rayon",
                                                    "num",
                                                    "percent_encoding",
                                                    "urlencoding",
                                                    "sha2",
                                                    "hmac",
                                                    "flate2",
                                                    "tar",
                                                    "semver",
                                                    "once_cell",
                                                    "rand_distr",
                                                    "log",
                                                    "time",
                                                    "math",
                                                    "sys",
                                                    "rand",
                                                    "base64",
                                                    "hex",
                                                    "env_logger",
                                                    "process",
                                                    "path",
                                                ];
                                                CM.contains(&mn.as_ref())
                                                    && type_field
                                                        .as_ref()
                                                        .chars()
                                                        .next()
                                                        .map(|c| c.is_uppercase())
                                                        .unwrap_or(false)
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };

                                if is_crate_static {
                                    if let Expr::Call(inner_call) = obj.as_ref() {
                                        if let Expr::Dot(_, type_field) = inner_call.name.as_ref() {
                                            let type_bytes =
                                                type_field.as_ref().as_bytes().to_vec();
                                            let type_idx = self.strings.len() as u32;
                                            self.strings.push(type_bytes);
                                            self.emit(OpCode::LOAD_STR);
                                            self.code.extend_from_slice(&type_idx.to_le_bytes());
                                            for arg in &inner_call.args.args {
                                                if let crate::ast::Arg::Pos(expr) = arg {
                                                    self.compile_call_arg(expr, "", 0)?;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    self.compile_expr(obj)?;
                                }
                            }
                        }
                    } else {
                        // Static method - no receiver
                        is_instance_method_call = false;
                    }
                }

                // Compile Arguments (pushes them to stack)
                // Plan 088 Phase 4: Smart parameter passing based on type and mode
                // For instance methods, receiver is arg 0, so other args start from index 1
                let arg_offset = if is_instance_method_call { 1 } else { 0 };

                // Check is_unresolved_static early (needed before arg compilation for stack layout)
                const KNOWN_STATIC_TYPES: &[&str] = &["HashMap", "Option", "Result"];
                let is_unresolved_static = !is_instance_method_call
                    && func_name
                        .as_ref()
                        .map(|name| {
                            let type_part = name.split('.').next().unwrap_or("");
                            type_part
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false)
                                && !KNOWN_STATIC_TYPES.contains(&type_part)
                                && !self.exports.contains_key(name)
                        })
                        .unwrap_or(false);

                // For unresolved static calls (e.g., Normal.new(2.0, 3.0)),
                // push type name as receiver BEFORE args so stack layout is [receiver, arg0, ...]
                // Only do this for truly unresolved calls (not for regular functions
                // that happen to start with uppercase like OP_RESERVE_STACK).
                // PLAN-053 P-053-6: 名字经 canonical 解析能命中 native
                // （Regex.replace → auto.regex.replace 一族）时不得压——此类
                // 调用走 CALL_NAT，native shim 只弹实参，残留的接收者槽会
                // 平移帧窗口（musk 现场：函数体内 Regex.replace 后续局部/字段
                // 读取落 "0"，消息 blocks text 全灭）。
                let unresolved_static_native_id = if is_unresolved_static {
                    func_name
                        .as_ref()
                        .and_then(|name| BIGVM_NATIVES.lock().unwrap().peek_qualified(name))
                } else {
                    None
                };
                if is_unresolved_static && unresolved_static_native_id.is_none() {
                    if let Some(name) = func_name.as_ref() {
                        let type_part = name.split('.').next().unwrap_or("");
                        let type_bytes = type_part.as_bytes().to_vec();
                        let type_idx = self.strings.len() as u32;
                        self.strings.push(type_bytes);
                        self.emit(OpCode::LOAD_STR);
                        self.code.extend_from_slice(&type_idx.to_le_bytes());
                    }
                }

                // Plan 417-E3-P4: call-site bound checking. For a call to a
                // bounded generic fn (`max_of(x, y)` where max_of is
                // `fn max_of<T has Comparable>(a T, b T) T`), reject
                // arguments whose static type demonstrably does not implement
                // the bound. Deliberately conservative — see
                // check_generic_call_bounds for what is skipped.
                if let Some(callee) = func_name.as_deref() {
                    self.check_generic_call_bounds(callee, &call.args.args)?;
                }

                if !call.args.is_empty() {
                    let func_name_for_params = func_name.as_ref().map(|s| s.as_str()).unwrap_or("");
                    // Plan 230: Get field types for f32→f64 promotion
                    let constructor_field_types: Option<Vec<Type>> = if let Expr::Ident(name) =
                        call.name.as_ref()
                    {
                        if self.generic_registry.has_template(name.as_ref()) {
                            self.generic_registry
                                .get_or_create_type(name.as_ref(), Vec::new())
                                .ok()
                                .map(|ct| ct.fields().into_iter().map(|f| f.field_type).collect())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    for (i, arg) in call.args.args.iter().enumerate() {
                        match arg {
                            crate::ast::Arg::Pos(expr) => {
                                let param_index = i + arg_offset;
                                if !func_name_for_params.is_empty()
                                    && self.fn_params.contains_key(func_name_for_params)
                                {
                                    vm_debug!(
                                        "DEBUG:   Arg {}: smart param passing for '{}'",
                                        param_index,
                                        func_name_for_params
                                    );
                                    self.compile_call_arg(expr, func_name_for_params, param_index)?;
                                } else {
                                    vm_debug!("DEBUG:   Arg {}: normal compile", param_index);
                                    self.compile_expr(expr)?;
                                }
                                // Plan 230: promote f32 -> f64 for type constructor args
                                if let Some(ref field_types) = constructor_field_types {
                                    if let Some(field_type) = field_types.get(i) {
                                        if matches!(field_type, Type::Double)
                                            && self.last_expr_type == ObjectType::Float
                                        {
                                            self.emit(OpCode::PROMOTE_F64);
                                        }
                                    }
                                }
                            }
                            crate::ast::Arg::Pair(_, expr) => {
                                // Named argument (e.g., add(a: 12, b: 2))
                                // Extract the value expression and compile it like a positional arg
                                let param_index = i + arg_offset;
                                if !func_name_for_params.is_empty()
                                    && self.fn_params.contains_key(func_name_for_params)
                                {
                                    vm_debug!(
                                        "DEBUG:   Named arg {}: smart param passing for '{}'",
                                        param_index,
                                        func_name_for_params
                                    );
                                    self.compile_call_arg(expr, func_name_for_params, param_index)?;
                                } else {
                                    vm_debug!("DEBUG:   Named arg {}: normal compile", param_index);
                                    self.compile_expr(expr)?;
                                }
                                // Plan 230: promote f32 -> f64 for type constructor args
                                if let Some(ref field_types) = constructor_field_types {
                                    if let Some(field_type) = field_types.get(i) {
                                        if matches!(field_type, Type::Double)
                                            && self.last_expr_type == ObjectType::Float
                                        {
                                            self.emit(OpCode::PROMOTE_F64);
                                        }
                                    }
                                }
                            }
                            crate::ast::Arg::Name(name) => {
                                // Name-only argument (e.g., add(a) where a is both name and value)
                                // Convert to expression by wrapping in Ident
                                let param_index = i + arg_offset;
                                if !func_name_for_params.is_empty()
                                    && self.fn_params.contains_key(func_name_for_params)
                                {
                                    vm_debug!(
                                        "DEBUG:   Named arg {}: smart param passing for '{}'",
                                        param_index,
                                        func_name_for_params
                                    );
                                    self.compile_call_arg(
                                        &Expr::Ident(name.clone()),
                                        func_name_for_params,
                                        param_index,
                                    )?;
                                } else {
                                    vm_debug!("DEBUG:   Named arg {}: normal compile", param_index);
                                    self.compile_expr(&Expr::Ident(name.clone()))?;
                                }
                                // Plan 230: promote f32 -> f64 for type constructor args
                                if let Some(ref field_types) = constructor_field_types {
                                    if let Some(field_type) = field_types.get(i) {
                                        if matches!(field_type, Type::Double)
                                            && self.last_expr_type == ObjectType::Float
                                        {
                                            self.emit(OpCode::PROMOTE_F64);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // 2. Emit CALL or CALL_SPEC opcode
                // Use CALL_SPEC when:
                // a) The receiver's declared type is a spec, OR
                // b) The function name (e.g., "tool.fly") doesn't exist in exports — likely dynamic dispatch
                // But NOT when it's a known native (e.g., str.find) — those use CALL_NAT
                let resolved_func = func_name.as_ref().and_then(|name| {
                    // Plan 339 Phase 4: check use alias mapping first.
                    if let Some(qualified) = self.import_scope.get(name) {
                        if let Some(addr) = self.exports.get(qualified).copied() {
                            return Some(addr);
                        }
                        // If alias exists but export not found, fall through to
                        // bare-name and prefix-strip resolution below.
                    }
                    // Plan 317 Phase B: exact match in exports.
                    if let Some(addr) = self.exports.get(name).copied() {
                        return Some(addr);
                    }
                    // Prefix-strip fallback (for calls like "db.create_note" → "db.create_note").
                    if let Some(stripped) = name.split('.').last() {
                        if stripped != name {
                            // Try the stripped bare name
                            if let Some(addr) = self.exports.get(stripped).copied() {
                                return Some(addr);
                            }
                        }
                    }
                    // PLAN-019 T-06 (vm delegate): own-module binding — a bare
                    // call inside a module-qualified fn resolves against its
                    // own module's export first (ambiguous bare names like
                    // mux_resize_branch exist under both api. and db.; the
                    // fn's module is authoritative). Mirrors resolve_call_symbol
                    // step 2.5; see current_fn_module field doc.
                    if !name.contains('.') {
                        if let Some(mod_name) = &self.current_fn_module {
                            if let Some(addr) =
                                self.exports.get(&format!("{}.{}", mod_name, name)).copied()
                            {
                                return Some(addr);
                            }
                        }
                    }
                    // Plan 339 Phase 6: bare-name → module-qualified fallback.
                    // A bare call to a function that wasn't named in any `use`
                    // clause (e.g. `search_notes(...)` from db.at) should still
                    // resolve if exactly ONE module-qualified export ends with
                    // that name. Ambiguous names (two modules define it) stay
                    // unresolved so the explicit `use` alias remains authoritative.
                    if !name.contains('.') {
                        let mut hits = self
                            .exports
                            .keys()
                            .filter(|k| k.split('.').last() == Some(name.as_str()));
                        let first = hits.next();
                        if let Some(k) = first {
                            if hits.next().is_none() {
                                if let Some(addr) = self.exports.get(k).copied() {
                                    return Some(addr);
                                }
                            }
                        }
                    }
                    None
                });
                let is_native = func_name
                    .as_ref()
                    .map(|name| BIGVM_NATIVES.lock().unwrap().peek_qualified(name).is_some())
                    .unwrap_or(false);
                // Check if receiver is a known user-defined type (not spec, not unknown)
                // If so, use direct CALL with relocation — the method export may just not be compiled yet
                let is_user_type_method = if let Expr::Dot(obj, _) = call.name.as_ref() {
                    if let Expr::Ident(receiver_name) = obj.as_ref() {
                        self.var_types
                            .get(receiver_name.as_ref())
                            .map(|ty| {
                                if let Type::User(td) = ty {
                                    // Option.Some/Option.None are synthetic types from enum destructuring,
                                    // not real user-defined types — don't treat them as user type methods.
                                    // Plan 325: 但 enum 变体类型（含 Result.Ok）若其 enum 名是已知 enum，
                                    // 应走 CALL reloc 让 enum 方法被解析（跨模块经 linker fallback）。
                                    // 此前 Result. 前缀被一刀切排除，导致 stdlib result.at 的方法走
                                    // CALL_SPEC 运行时分发失败。
                                    let is_enum_variant = td.name.contains('.')
                                        && self.is_enum_name(
                                            td.name.split('.').next().unwrap_or(&td.name),
                                        );
                                    is_enum_variant
                                        || (!td.name.starts_with("Option.")
                                            && !td.name.starts_with("Result."))
                                } else {
                                    matches!(ty, Type::User(_))
                                }
                            })
                            .unwrap_or(false)
                    } else {
                        false
                    }
                } else {
                    false
                };
                // Plan 317: If this is an Auto module call (db.func()), generate
                // a CALL with reloc so the linker resolves it to the module's
                // exported function. Don't fall through to CALL_SPEC.
                let is_auto_module_call = func_name
                    .as_ref()
                    .map(|name| {
                        name.split('.')
                            .next()
                            .map(|mod_name| self.auto_modules.contains(mod_name))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);

                // Also use CALL_SPEC for unresolved static-like calls (e.g., Normal.new, Complex.new)
                // where the type name starts with uppercase but isn't in exports/natives
                // (is_unresolved_static was computed above before arg compilation)
                if is_auto_module_call && resolved_func.is_none() {
                    // Generate CALL with reloc symbol = func_name (e.g. "db2.hello").
                    // Linker's prefix fallback resolves it to the module's "hello".
                    self.emit(OpCode::CALL);
                    let placeholder_idx = self.code.len();
                    self.code.extend_from_slice(&0u32.to_le_bytes());
                    // Plan 339 Phase 4: apply import_scope alias to the reloc
                    // symbol so bare calls (e.g. "delete_note") resolve to the
                    // module-qualified export (e.g. "api.delete_note").
                    let reloc_symbol = func_name
                        .as_ref()
                        .and_then(|name| {
                            self.import_scope
                                .get(name)
                                .cloned()
                                .or_else(|| Some(name.clone()))
                        })
                        .unwrap_or_default();
                    self.relocs.push(RelocEntry {
                        offset: placeholder_idx as u32,
                        symbol_name: reloc_symbol,
                        reloc_type: RelocType::FuncCall,
                        source_pos: None,
                    });
                    return Ok(());
                } else if is_user_type_method
                    && matches!(call.name.as_ref(), Expr::Dot(_, m) if m.as_str() == "clone")
                    && resolved_func.is_none()
                    && call.args.args.is_empty()
                {
                    // Plan 442 C2: `.clone()` on a user-type/enum value with no
                    // declared clone method — transparent passthrough, mirroring
                    // the .unwrap()/.expect() opaque passthrough below. The
                    // receiver (already compiled onto the stack) IS the result.
                    // Enum values are scalar-like on the VM; heap instances share
                    // by reference (documented C2-probe semantics — musk backend
                    // corpus's User/AgentMode/Profession/SpecsDocument.clone).
                    return Ok(());
                } else if is_spec_dispatch || (func_name.is_some() && resolved_func.is_none() && !is_native && !is_user_type_method && (is_instance_method_call || is_unresolved_static))
                    // PLAN-066 T-06（055-4⑥ 现代真身）：未注册的 str.* 方法
                    // （includes/startsWith/trim…）走 CALL_SPEC 运行时按堆标签
                    // 分发——引擎 str 臂已实现该族。此前解析失败落 extern
                    // no-op 桩（Nil→if 恒假），musk filteredMessages 的
                    // includes 过滤恒假即此；t3_filter 脚本层另一分发路故绿。
                    || func_name.as_deref().map_or(false, |f| f.starts_with("str.") || f.starts_with("auto.str."))
                {
                    // Plan 249: Transparent unwrap for opaque handle values.
                    // If this is .unwrap()/.expect() on an opaque Rust crate value,
                    // skip CALL_SPEC — the receiver (inner call result) is already a valid handle.
                    let callee_method = if let Expr::Dot(_, m) = call.name.as_ref() {
                        m.as_str()
                    } else {
                        ""
                    };
                    let is_opaque_unwrap = (callee_method == "unwrap" || callee_method == "expect")
                        && if let Expr::Dot(obj, _) = call.name.as_ref() {
                            match obj.as_ref() {
                                Expr::Ident(recv) => {
                                    self.opaque_var_crates.contains_key(recv.as_str())
                                }
                                Expr::Call(inner_call) => {
                                    if let Expr::Dot(inner_obj, _) = inner_call.name.as_ref() {
                                        if let Expr::Ident(type_name) = inner_obj.as_ref() {
                                            self.opaque_var_crates.contains_key(type_name.as_str())
                                                || self
                                                    .rust_native_map
                                                    .contains_key(type_name.as_str())
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                }
                                _ => false,
                            }
                        } else {
                            false
                        };

                    if is_opaque_unwrap {
                        // Receiver already on stack from inner call, just leave it
                    } else {
                        // Dynamic dispatch: emit CALL_SPEC with method name string index and arg count
                        let method_str = if let Expr::Dot(_, method) = call.name.as_ref() {
                            method.to_string()
                        } else {
                            func_name.clone().unwrap_or_default()
                        };
                        // Arg count (excluding receiver) so engine knows stack layout
                        // Note: for is_unresolved_static, receiver was already pushed before args
                        let arg_count = call.args.args.len() as u8;
                        self.emit(OpCode::CALL_SPEC);
                        let method_bytes = method_str.as_bytes().to_vec();
                        let method_idx = self.strings.len() as u32;
                        self.strings.push(method_bytes);
                        self.code.extend_from_slice(&method_idx.to_le_bytes());
                        self.code.push(arg_count);

                        // Infer return type for CALL_SPEC based on method name suffix
                        self.last_expr_type = self.infer_call_spec_return_type(&method_str);
                    } // close opaque_unwrap else
                } else {
                    // Plan 321: Check if this is a generator function call.
                    // If so, emit CREATE_GENERATOR instead of CALL.
                    let is_generator = func_name
                        .as_ref()
                        .map(|n| self.generator_fns.contains(n))
                        .unwrap_or(false);

                    if is_generator {
                        // Emit CREATE_GENERATOR with inline operands:
                        // CREATE_GENERATOR, func_addr:u32 (relocated), n_args:u8
                        self.emit(OpCode::CREATE_GENERATOR);
                        let placeholder_idx = self.code.len();
                        self.code.extend_from_slice(&0u32.to_le_bytes()); // func_addr placeholder
                        self.code.push(call.args.args.len() as u8); // n_args

                        // Create relocation for the func_addr placeholder
                        let reloc_name =
                            func_name
                                .clone()
                                .unwrap_or_else(|| match call.name.as_ref() {
                                    Expr::Ident(name) => name.to_string(),
                                    _ => "unknown".to_string(),
                                });
                        self.relocs.push(RelocEntry {
                            offset: placeholder_idx as u32,
                            symbol_name: reloc_name,
                            reloc_type: RelocType::FuncCall,
                            source_pos: call.pos,
                        });

                        self.last_expr_type = ObjectType::Int; // iterator_id

                        // PLAN-776 R5-01/R6-01：generator **生产点**凭据化
                        //（无条件——身份授予不依赖所在 fn 声明：旧 int 声明/
                        // int helper 返回真实 generator 同样保来源，F776-R6-01
                        // 反例 S7/S8；声明只用于编组校验/能力选择）。凭据随
                        // local/helper/容器/闭包/Future 载体天然传播；for-in
                        // 源由 iterator natives 凭据感知解包（非 HTTP 消费
                        // 兼容）；普通数值永不取得身份（`return 0` 仍是数据）。
                        self.emit(OpCode::WRAP_SSE_RESOURCE);
                    } else {
                        self.emit(OpCode::CALL);

                        // Emit Placeholder for Address (u32)
                        let placeholder_idx = self.code.len();
                        self.code.extend_from_slice(&0u32.to_le_bytes());

                        // Create Relocation Entry
                        // Plan 339: resolve the bare call name to its module-qualified
                        // export name so the linker binds the right target. Checks
                        // import_scope alias first, then a unique bare-name →
                        // qualified fallback (see resolved_func above).
                        let raw_name =
                            func_name
                                .clone()
                                .unwrap_or_else(|| match call.name.as_ref() {
                                    Expr::Ident(name) => name.to_string(),
                                    _ => unimplemented!(
                                        "Dynamic call not supported: name expr = {:?}",
                                        call.name
                                    ),
                                });
                        let reloc_name = self.resolve_call_symbol(&raw_name);
                        self.debug_resolve_call_symbol(&raw_name, &reloc_name);

                        vm_debug!(
                            "DEBUG: Creating reloc for function '{}' at offset 0x{:04x}",
                            reloc_name,
                            placeholder_idx
                        );
                        vm_debug!(
                            "DEBUG: Available exports: {:?}",
                            self.exports.keys().collect::<Vec<_>>()
                        );

                        self.relocs.push(RelocEntry {
                            offset: placeholder_idx as u32,
                            symbol_name: reloc_name.clone(),
                            reloc_type: RelocType::FuncCall,
                            source_pos: call.pos,
                        });

                        // Plan 118 Phase 4: Function return type inference.
                        // Plan 368 R-W4: try the reloc name first (intra-module calls,
                        // native names, method relocs). For `use auto.<lib>: fn` imports
                        // the reloc name is module-qualified ("c_process_app.parse_nth")
                        // but the user fn's return type is registered under its BARE name
                        // ("parse_nth") by enrich_fn_return_types_from_type_store (which
                        // reads the shared type_store), so fall back to the last path
                        // segment. Without this, cross-module user-fn calls mis-tag
                        // last_expr_type with the stale value left by arg compilation,
                        // dropping/mis-dispatching the returned value (W4 residual; same
                        // family as the W8 native-method fix below).
                        let ret_ty = self.fn_return_types.get(&reloc_name).or_else(|| {
                            reloc_name
                                .rsplit('.')
                                .next()
                                .filter(|bare| !bare.is_empty() && *bare != reloc_name.as_str())
                                .and_then(|bare| self.fn_return_types.get(bare))
                        });
                        if let Some(ret_ty) = ret_ty {
                            self.last_expr_type = match ret_ty {
                                Type::Void => ObjectType::Void,
                                Type::Float => ObjectType::Float,
                                Type::Double => ObjectType::Double,
                                Type::StrFixed(_)
                                | Type::StrOwned
                                | Type::CStrLit
                                | Type::StrSlice => ObjectType::String,
                                Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                                Type::Byte => ObjectType::Byte,
                                Type::Bool => ObjectType::Bool,
                                Type::Int | Type::I64 => ObjectType::Int,
                                _ => ObjectType::NestedObject,
                            };
                        } else {
                            // Plan 368 W8: native instance methods (e.g. s.upper(),
                            // s.replace(...), s.split(...)) reach here as a CALL with
                            // a reloc name like "str.upper" that is NOT in
                            // fn_return_types (native shims aren't registered there).
                            // Without this fallback, last_expr_type keeps the stale
                            // value left by compiling the receiver+args (often Void),
                            // so `return s.method()` / `f(s.method())` drop the result.
                            // infer_native_return_type knows the str/char method types.
                            self.last_expr_type = self.infer_native_return_type(&reloc_name);
                            self.last_was_native_void =
                                matches!(self.last_expr_type, ObjectType::Void);
                        }
                        // Plan 569 D2: 用户 fn 返回 py 桥值传导——调用点查
                        // fn_may_py_returns（限定名/裸名双查，镜像上方 ret_ty 回退序）。
                        self.last_expr_may_py =
                            self.fn_may_py_returns.contains(reloc_name.as_str())
                                || reloc_name
                                    .rsplit('.')
                                    .next()
                                    .filter(|bare| !bare.is_empty() && *bare != reloc_name.as_str())
                                    .map(|bare| self.fn_may_py_returns.contains(bare))
                                    .unwrap_or(false);
                    } // close else (non-generator CALL)
                }
            }
            _ => return self.expr_control(expr),
        }
        Ok(())
    }
}
