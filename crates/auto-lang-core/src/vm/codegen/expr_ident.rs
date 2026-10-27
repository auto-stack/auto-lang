//! PLAN-843 结构拆分：compile_expr_inner 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl Codegen {
    /// PLAN-843 尾递归派发段（自 compile_expr_inner 连续臂切片，臂体逐字保留）。
    pub(super) fn expr_ident(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            Expr::Str(s) => {
                // Intern string literal: reuse existing index for identical strings
                // so that == compares identical tags
                let idx = self.add_string(s);
                self.emit(OpCode::LOAD_STR);
                self.code.extend_from_slice(&idx.to_le_bytes());
                // Plan 118 Phase 7: Track expression type for proper result formatting
                self.last_expr_type = ObjectType::String;
            }
            Expr::Ident(name) => {
                let name_str = name.to_string();
                vm_debug!("DEBUG: Compiling Ident: {}", name_str);

                // Plan 118: Check variable type for result formatting
                if let Some(var_type) = self.var_types.get(&name_str) {
                    self.last_expr_type = match var_type {
                        Type::StrFixed(_) | Type::StrSlice | Type::StrOwned => ObjectType::String,
                        Type::Byte => ObjectType::Byte,
                        Type::Uint | Type::U64 => ObjectType::Uint,
                        Type::Float => ObjectType::Float,
                        Type::Double => ObjectType::Double,
                        _ => ObjectType::Int,
                    };
                } else {
                    // No type tracked — default to Int for untyped variables
                    self.last_expr_type = ObjectType::Int;
                }
                // Plan 569 D2: Ident 加载镜像侧表成员资格——`var u = t` 的 RHS
                // 传播（粘性位在 compile_expr 顶已重置，此处按变量名补回）。
                if self.py_typed_vars.contains(&name_str) {
                    self.last_expr_may_py = true;
                }

                // Plan 317: actor state field read (inside task hooks/handlers).
                // Takes precedence over ordinary locals — state fields are the
                // persistent per-actor storage accessed via LOAD_STATE_FIELD.
                if let Some(&field_idx) = self.current_task_state_fields.get(&name_str) {
                    self.emit(OpCode::LOAD_STATE_FIELD);
                    self.code.push(field_idx);
                // Check if this is a captured variable (Plan 071)
                } else if let Some(_capture_index) = self.current_captured_vars().get(&name_str) {
                    // Variable is captured - emit LOAD_CAPTURED
                    vm_debug!("DEBUG: Variable {} is captured", name_str);
                    self.emit_load_captured(&name_str);
                } else if let Some(var_index) = self.lookup_var(&name_str) {
                    // Variable found in local scope - load it.
                    // Plan 377: 全值单槽化 —— u64/i64/Double 变量亦只占 1 槽，
                    // 不再 emit 第二个 LOAD_LOC(var_index+1)。
                    self.emit_load_loc(var_index);
                } else if self.global_vars.contains(&name_str) {
                    // Plan 317: module-level global variable.
                    self.emit_global_load(&name_str);
                } else if let Some(qualified) = self.import_scope.get(&name_str).cloned() {
                    // Plan 348 E1: imported value name (e.g. `use auto.testlib: MAX`
                    // → import_scope["MAX"] = "testlib.MAX"). Emit LOAD_GLOBAL with
                    // the exact qualified name under which it was stored in
                    // vm.globals (no current_module prefix).
                    self.emit_global_load_qualified(&qualified);
                } else {
                    // Plan 127: Check if this is an enum variant (e.g., Red from enum Color)
                    let enum_variant_value = self
                        .type_store
                        .read()
                        .unwrap()
                        .find_enum_variant_by_name(&name_str)
                        .map(|(_, v)| v);
                    if let Some(value) = enum_variant_value {
                        vm_debug!(
                            "DEBUG: Variable {} resolved as enum variant with value {}",
                            name_str,
                            value
                        );
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(value);
                        self.last_expr_type = ObjectType::Int;
                    } else {
                        // Plan 087 Phase 3: Check implicit field access in methods
                        // If we're inside a type method, bare field names should resolve to self.field
                        if let Some(ref members) = self.current_type_members {
                            if members.contains(&name_str) {
                                if let Some(self_index) = self.lookup_var("self") {
                                    vm_debug!(
                                        "DEBUG: Variable {} resolved as implicit self.{} access",
                                        name_str,
                                        name_str
                                    );
                                    self.emit_load_loc(self_index);
                                    // Determine type name from var_types["self"]
                                    let type_name = self.var_types.get("self").and_then(|t| {
                                        if let Type::User(td) = t {
                                            Some(td.name.to_string())
                                        } else {
                                            None
                                        }
                                    });
                                    if let Some(ref tn) = type_name {
                                        if self.generic_registry.has_template(tn) {
                                            if let Ok(class_type) =
                                                self.generic_registry.get_or_create_type(tn, vec![])
                                            {
                                                if let Some(field_idx) =
                                                    class_type.template.field_index(&name_str)
                                                {
                                                    self.emit(OpCode::GET_GENERIC_FIELD);
                                                    self.code.extend_from_slice(
                                                        &(field_idx as u32).to_le_bytes(),
                                                    );
                                                    self.last_expr_type = ObjectType::Int;
                                                    return Ok(());
                                                }
                                            }
                                        }
                                    }
                                    // Fallback: use GET_FIELD (name-based)
                                    let field_bytes = name_str.as_bytes().to_vec();
                                    let field_idx = self.strings.len() as u32;
                                    self.strings.push(field_bytes);
                                    self.emit(OpCode::GET_FIELD);
                                    self.code.extend_from_slice(&field_idx.to_le_bytes());
                                    return Ok(());
                                }
                            }
                        }
                        vm_debug!("DEBUG: Variable {} NOT FOUND!", name_str);
                        // Plan 240: Check if this is a rust_native_map import (module prefix)
                        if self.rust_native_map.contains_key(&name_str) {
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(0); // placeholder for module prefix
                            self.last_expr_type = ObjectType::NestedObject;
                        } else if Self::is_known_rust_crate(&name_str) {
                            // Fallback: treat unknown identifiers matching known crate names
                            // as module prefixes (e.g., log, env_logger, regex, etc.)
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(0);
                            self.last_expr_type = ObjectType::NestedObject;
                        } else if self
                            .exports
                            .keys()
                            .any(|k| k.starts_with(&format!("{}.", name_str)))
                        {
                            // Module prefix from a `mod` block (e.g., `network.connect`)
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(0);
                            self.last_expr_type = ObjectType::NestedObject;
                        } else if self.known_module_prefixes.contains(&name_str)
                            || matches!(
                                name_str.as_ref(),
                                "str"
                                    | "json"
                                    | "fs"
                                    | "time"
                                    | "math"
                                    | "sys"
                                    | "env"
                                    | "http"
                                    | "net"
                                    | "os"
                                    | "log"
                                    | "db"
                                    | "rand"
                                    | "fmt"
                                    | "io"
                                    | "path"
                                    | "process"
                                    | "tcp"
                                    | "udp"
                                    | "thread"
                                    | "channel"
                                    | "regex"
                                    | "hash"
                                    | "crypto"
                                    | "base64"
                                    | "hex"
                                    | "csv"
                                    | "xml"
                                    | "yaml"
                                    | "toml"
                                    | "session"
                                    | "template"
                                    | "openapi"
                                    | "storage"
                                    | "sched"
                                    | "localStorage"
                                    | "dom"
                                    | "location"
                                    | "image"
                                    | "bus"
                                    | "frame"
                                    | "ui"
                            )
                            || matches!(
                                name_str.as_ref(),
                                "str"
                                    | "json"
                                    | "fs"
                                    | "time"
                                    | "math"
                                    | "sys"
                                    | "env"
                                    | "http"
                                    | "net"
                                    | "os"
                                    | "log"
                                    | "db"
                                    | "rand"
                                    | "fmt"
                                    | "io"
                                    | "path"
                                    | "process"
                                    | "tcp"
                                    | "udp"
                                    | "thread"
                                    | "channel"
                                    | "regex"
                                    | "hash"
                                    | "crypto"
                                    | "base64"
                                    | "hex"
                                    | "csv"
                                    | "xml"
                                    | "yaml"
                                    | "toml"
                                    | "session"
                                    | "template"
                                    | "openapi"
                                    | "storage"
                                    | "sched"
                                    | "localStorage"
                                    | "dom"
                                    | "location"
                                    | "image"
                                    | "bus"
                                    | "ui"
                            )
                        {
                            // Module prefix from module-level import or built-in stdlib module
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(0);
                            self.last_expr_type = ObjectType::NestedObject;
                        } else if self.py_native_map.contains_key(&name_str) {
                            // Plan 369 Task 11: bare reference to a py-imported name
                            // that is not a local/global/enum resolves to a zero-arg
                            // py-FFI native call (e.g. math.pi constant, or a no-arg
                            // Python function). Resolve the native_id and emit CALL_NAT_COUNTED
                            // with arg_count=0. py_constants marks genuine constants;
                            // zero-arg callables also flow through here.
                            let qualified = format!("py.{}", name_str);
                            let native_id = {
                                let mut reg = BIGVM_NATIVES.lock().unwrap();
                                reg.resolve_qualified(&qualified)
                                    .unwrap_or_else(|| reg.register(&qualified))
                            };
                            self.emit(OpCode::CALL_NAT_COUNTED);
                            self.emit_u16(native_id);
                            self.code.push(0); // arg_count = 0
                                               // py-FFI auto return marshals to string pool by default
                            self.last_expr_type = ObjectType::String;
                        } else if self.is_defined_function(&name_str) {
                            // Plan 383: 命名函数引用 —— 裸函数名在值位置（非调用
                            // `name(...)`）当作函数引用。对标 Rust/Python 隐式函数引用：
                            // 函数项在值位置自动成为可调用值。复用 CLOSURE 运行时
                            // （零捕获 closure = 函数引用），使 `f()` 走既有 CALL_CLOSURE。
                            // 分支置于所有变量/枚举/导入查找失败之后，故同名变量永远优先。
                            let n_args = self.function_param_count(&name_str);
                            self.emit(OpCode::CLOSURE);
                            let func_addr_offset = self.code.len() as u32;
                            self.code.extend_from_slice(&(0u32).to_le_bytes()); // func_addr 占位，reloc 填充
                            self.code.push(0); // capture_count = 0（函数引用不捕获环境）
                            self.code.push(n_args as u8); // n_args（CALL_CLOSURE 用）
                            self.relocs.push(crate::vm::loader::RelocEntry {
                                offset: func_addr_offset,
                                symbol_name: name_str.clone(),
                                reloc_type: crate::vm::loader::RelocType::FuncCall,
                                source_pos: None,
                            });
                            self.last_expr_type = ObjectType::NestedObject;
                        } else {
                            return Err(AutoError::Msg(format!(
                                "Undefined variable: {}",
                                name_str
                            )));
                        }
                    }
                }
            }
            _ => return self.expr_dot_index(expr),
        }
        Ok(())
    }
}
