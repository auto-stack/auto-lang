//! PLAN-843 结构拆分：native io 域实现（自 native.rs 原样搬移）。
use super::*;

/// Plan 177: Helper to print output, captures to buffer if present
pub(super) fn vm_print(vm: &AutoVM, s: &str) {
    if let Some(ref buf) = vm.output_buffer {
        let mut guard = buf.write().unwrap();
        guard.push_str(s);
        guard.push('\n');
    } else {
        println!("{}", s);
    }
    // Mirror into the in-app console ring buffer (041 Console panel); the
    // capped buffer makes this cheap enough for non-UI CLI runs too.
    crate::vm::ui_console::ui_console_push(s);
}

/// Helper for write() — same as vm_print but without trailing newline
fn vm_write(vm: &AutoVM, s: &str) {
    if let Some(ref buf) = vm.output_buffer {
        let mut guard = buf.write().unwrap();
        guard.push_str(s);
    } else {
        print!("{}", s);
    }
}

/// `console_log(msg) -> Bool` — append one line to the console buffer.
pub fn shim_console_log(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg = pop_string_arg(task, vm);
    crate::vm::ui_console::ui_console_push(&msg);
    task.ram.push_nv(auto_val::encode_bool(true));
    Ok(())
}

/// `console_lines() -> String` — newest-first snapshot for panel display.
pub fn shim_console_lines(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let text = crate::vm::ui_console::ui_console_lines(crate::vm::ui_console::DEFAULT_LINES);
    let idx = vm.add_string(text.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `console_clear() -> Bool` — drop all console lines.
pub fn shim_console_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    crate::vm::ui_console::ui_console_clear();
    task.ram.push_nv(auto_val::encode_bool(true));
    Ok(())
}

pub fn shim_print_i32(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let str_index = auto_val::decode_string(nv);
            if let Some(bytes) = vm.get_string(str_index) {
                vm_print(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_print(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else if auto_val::is_null(nv) {
            // None value (null)
            vm_print(vm, "None");
        } else if auto_val::is_object(nv) {
            let handle = auto_val::decode_object(nv) as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                } else {
                    // Plan 560 T06 (D7)：py 句柄 print 保真——GIL str()。
                    #[cfg(feature = "python")]
                    {
                        let pyh = guard
                            .as_any()
                            .downcast_ref::<crate::py_ffi::PyObjectHandle>();
                        if let Some(pyh) = pyh {
                            let s = pyo3::Python::attach(|py| {
                                use pyo3::types::PyAnyMethods;
                                pyh.obj
                                    .bind(py)
                                    .str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|e| format!("<py str failed: {}>", e))
                            });
                            vm_print(vm, &s);
                            return Ok(());
                        }
                    }
                    if let Some(dep_obj) = guard
                        .as_any()
                        .downcast_ref::<crate::vm::ffi::dep_methods::DepOpaqueObject>(
                    ) {
                        // PLAN-591 D2(DIV-DEP-8 print 半边):dep 对象路由
                        // shim 包 Display 合成 to_string;无 Display 面维持占位。
                        let short_type = dep_obj.short_type.clone();
                        drop(guard);
                        let text = match crate::vm::ffi::dep_methods::display_string(
                            task,
                            vm,
                            handle,
                            &short_type,
                        )? {
                            Some(t) => t,
                            None => format!("<obj:{}>", handle),
                        };
                        vm_print(vm, &text);
                    } else {
                        vm_print(vm, &format!("<obj:{}>", handle));
                    }
                }
            } else {
                vm_print(vm, &format!("<invalid object: {}>", handle));
            }
        } else {
            let val = auto_val::decode_i32(nv);
            // Boolean sentinel values: i32::MIN = true, i32::MIN+1 = false
            if val == -2147483648 {
                vm_print(vm, "1");
            } else if val == -2147483647 {
                vm_print(vm, "0");
            } else if val > 0 {
                // Check if positive value is a heap object handle (RustStdlibObject etc.)
                let handle = val as u64;
                if let Some(obj) = vm.get_heap_object(handle) {
                    let guard = obj.read().unwrap();
                    if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                        vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                    } else {
                        vm_print(vm, &val.to_string());
                    }
                } else {
                    vm_print(vm, &val.to_string());
                }
            } else {
                vm_print(vm, &val.to_string());
            }
        }
        Ok(())
    }
}

pub fn shim_print_f32(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 377: 全值单槽化。f64/f32 各占 1 槽，弹出单个 NanoValue 按 tag 分派。
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_f64(nv) {
            // f64（非 nanboxed，直接位模式）
            let val = f64::from_bits(nv);
            vm_print(vm, &val.to_string());
            return Ok(());
        }
        if auto_val::is_f32(nv) {
            let val = auto_val::decode_f32(nv);
            vm_print(vm, &val.to_string());
            return Ok(());
        }
        // 兜底：i32 的位模式当 f32 解（兼容旧路径）
        let val_bits = auto_val::decode_i32(nv) as u32;
        let val = f32::from_bits(val_bits);
        vm_print(vm, &val.to_string());
    }
    Ok(())
}

/// Print an f64 value. Plan 377: 单槽（pop_f64 现在弹 1 个 NanoValue）。
pub fn shim_print_f64(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_f64();
    vm_print(vm, &val.to_string());
    Ok(())
}

/// Print a u64/i64 value.
/// Plan 377: 单槽化后 pop_i64 弹单个 NanoValue（TAG_I64/U64/BIGINT）。
pub fn shim_print_u64(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i64();
    vm_print(vm, &val.to_string());
    Ok(())
}

/// Plan 377 §3.1: 统一 print —— 弹出单个 NanoValue，按 tag 解码打印任意类型。
///
/// 取代按类型路由的 5 个 print native（I32/F32/F64/U64/STR）的分裂：单槽化后
/// 每个值都是单个 NanoValue，无需 codegen 在编译期推断 ObjectType 选 print 入口。
/// 本函数把 i32/f32/f64/i64/u64/bigint/string/bool/null/object 的解码全部集中。
///
/// 格式约定（与既有 shim_print_i32/f32/f64/u64 保持一致，避免回归）：
/// - bool sentinel（i32::MIN/-2147483648 = true → "1"，i32::MIN+1 = false → "0"）
/// - null → "None"；object → format_rust_stdlib_obj 或 <obj:id>
/// - 数值 → to_string()
pub fn shim_print_unified(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        let tag = auto_val::tag_of(nv);
        // f64（非 nanboxed，直接位模式）
        if !auto_val::is_nanboxed(nv) {
            vm_print(vm, &auto_val::decode_f64(nv).to_string());
            return Ok(());
        }
        match tag {
            // TAG_STRING
            t if t == 2 => {
                let str_index = auto_val::decode_string(nv);
                if let Some(bytes) = vm.get_string(str_index) {
                    vm_print(vm, &String::from_utf8_lossy(&bytes));
                } else {
                    vm_print(vm, &format!("<invalid string index: {}>", str_index));
                }
            }
            // TAG_BOOL（sentinel: i32::MIN = true, i32::MIN+1 = false）
            // Plan 474 待澄清#3: 显示形态对齐 Rust 式语义 true/false（原打
            // "1"/"0"——json bool 字段 print 与真值语义混淆的旁支根源）。
            t if t == 3 => {
                vm_print(
                    vm,
                    if auto_val::decode_bool(nv) {
                        "true"
                    } else {
                        "false"
                    },
                );
            }
            // TAG_NULL
            t if t == 4 => {
                vm_print(vm, "None");
            }
            // TAG_OBJECT
            t if t == 5 => {
                let handle = auto_val::decode_object(nv) as u64;
                if let Some(obj) = vm.get_heap_object(handle) {
                    let guard = obj.read().unwrap();
                    if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                        vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                    } else if let Some(dep_obj) = guard
                        .as_any()
                        .downcast_ref::<crate::vm::ffi::dep_methods::DepOpaqueObject>(
                    ) {
                        // PLAN-591 D2(DIV-DEP-8 print 半边):dep 对象路由 shim 包
                        // Display 合成 to_string;无 Display 面维持占位。
                        let short_type = dep_obj.short_type.clone();
                        drop(guard);
                        let text = match crate::vm::ffi::dep_methods::display_string(
                            task,
                            vm,
                            handle,
                            &short_type,
                        )? {
                            Some(t) => t,
                            None => format!("<obj:{}>", handle),
                        };
                        vm_print(vm, &text);
                    } else {
                        vm_print(vm, &format!("<obj:{}>", handle));
                    }
                } else {
                    vm_print(vm, &format!("<invalid object: {}>", handle));
                }
            }
            // TAG_F32
            t if t == 7 => {
                vm_print(vm, &auto_val::decode_f32(nv).to_string());
            }
            // TAG_I64
            t if t == 8 => {
                vm_print(vm, &auto_val::decode_i64(nv).to_string());
            }
            // TAG_U64
            t if t == 9 => {
                vm_print(vm, &auto_val::decode_u64(nv).to_string());
            }
            // TAG_BIGINT（堆对象，需解引用 —— 与 convert.rs decode 一致）
            t if t == 0xA => {
                let id = auto_val::decode_bigint_handle(nv) as u64;
                let printed = if let Some(obj) = vm.get_heap_object(id) {
                    if let Some(guard) = obj.read().ok() {
                        use crate::vm::heap_object::{downcast, BigIntData};
                        if let Some(big) = downcast::<BigIntData>(&*guard) {
                            if big.is_unsigned {
                                big.as_u64().to_string()
                            } else {
                                big.as_i64().to_string()
                            }
                        } else {
                            format!("<bigint:{}>", id)
                        }
                    } else {
                        format!("<bigint:{}>", id)
                    }
                } else {
                    format!("<invalid bigint: {}>", id)
                };
                vm_print(vm, &printed);
            }
            // TAG_I32（正数 heap handle 探测，兼容 shim_print_i32）。
            // Plan 474 待澄清#3: 原 bool 哨兵特判（i32::MIN→"1"、MIN+1→"0"）
            // 摘除——TAG_BOOL 已由上方 tag==3 臂处理，此处特判只会把真整数
            // i32::MIN/i32::MIN+1 误显为 1/0。
            _ => {
                let val = auto_val::decode_i32(nv);
                if val > 0 {
                    let handle = val as u64;
                    if let Some(obj) = vm.get_heap_object(handle) {
                        let guard = obj.read().unwrap();
                        if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                            vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                        } else {
                            vm_print(vm, &val.to_string());
                        }
                    } else {
                        vm_print(vm, &val.to_string());
                    }
                } else {
                    vm_print(vm, &val.to_string());
                }
            }
        }
        Ok(())
    }
}

/// Print a string from the string constant pool, or an integer if not a string.
/// Expects tagged string index on TOS (LOAD_STR pushes -(idx+1)).
/// If the value is non-negative and not a valid string index, prints as integer.
pub fn shim_print_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let str_index = auto_val::decode_string(nv);
            if let Some(bytes) = vm.get_string(str_index) {
                vm_print(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_print(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else if auto_val::is_null(nv) {
            vm_print(vm, "None");
        } else if auto_val::is_object(nv) {
            let handle = auto_val::decode_object(nv) as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                } else if let Some(dep_obj) = guard
                    .as_any()
                    .downcast_ref::<crate::vm::ffi::dep_methods::DepOpaqueObject>(
                ) {
                    // PLAN-591 D2(DIV-DEP-8 print/write 半边):dep 对象路由
                    // shim 包 Display 合成 to_string;无 Display 面维持占位。
                    let short_type = dep_obj.short_type.clone();
                    drop(guard);
                    let text = match crate::vm::ffi::dep_methods::display_string(
                        task,
                        vm,
                        handle,
                        &short_type,
                    )? {
                        Some(t) => t,
                        None => format!("<obj:{}>", handle),
                    };
                    vm_print(vm, &text);
                } else {
                    if let Some(dep_obj) = guard
                        .as_any()
                        .downcast_ref::<crate::vm::ffi::dep_methods::DepOpaqueObject>(
                    ) {
                        // PLAN-591 D2(DIV-DEP-8 print 半边):dep 对象路由
                        // shim 包 Display 合成 to_string;无 Display 面维持占位。
                        let short_type = dep_obj.short_type.clone();
                        drop(guard);
                        let text = match crate::vm::ffi::dep_methods::display_string(
                            task,
                            vm,
                            handle,
                            &short_type,
                        )? {
                            Some(t) => t,
                            None => format!("<obj:{}>", handle),
                        };
                        vm_print(vm, &text);
                    } else {
                        vm_print(vm, &format!("<obj:{}>", handle));
                    }
                }
            } else {
                vm_print(vm, &format!("<invalid object: {}>", handle));
            }
        } else if auto_val::is_bool(nv) {
            // Plan 474 待澄清#3: bool 按 tag 判定打印 true/false。原实现对
            // TAG_BOOL 的哨兵 payload 走 decode_i32 后按值特判打 "1"/"0"——
            // decode 不分 tag，真整数 i32::MIN/i32::MIN+1 会被同路误打，且
            // bool 显示形态与 Rust 式语义不符。
            vm_print(
                vm,
                if auto_val::decode_bool(nv) {
                    "true"
                } else {
                    "false"
                },
            );
        } else {
            let val = auto_val::decode_i32(nv);
            if val > 0 {
                // Check if positive value is a heap object handle
                let handle = val as u64;
                if let Some(obj) = vm.get_heap_object(handle) {
                    let guard = obj.read().unwrap();
                    if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                        vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                    } else {
                        vm_print(vm, &val.to_string());
                    }
                } else {
                    vm_print(vm, &val.to_string());
                }
            } else {
                vm_print(vm, &val.to_string());
            }
        }
    }
    Ok(())
}

/// Write a string without trailing newline (write() in Auto).
/// Same logic as shim_print_str but uses vm_write instead of vm_print.
pub fn shim_write_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let str_index = auto_val::decode_string(nv);
            if let Some(bytes) = vm.get_string(str_index) {
                vm_write(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_write(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else if auto_val::is_object(nv) {
            let handle = auto_val::decode_object(nv) as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_write(vm, &format_rust_stdlib_obj(rust_obj));
                } else if let Some(dep_obj) = guard
                    .as_any()
                    .downcast_ref::<crate::vm::ffi::dep_methods::DepOpaqueObject>(
                ) {
                    // PLAN-591 D2(DIV-DEP-8 print/write 半边):dep 对象路由
                    // shim 包 Display 合成 to_string;无 Display 面维持占位。
                    let short_type = dep_obj.short_type.clone();
                    drop(guard);
                    let text = match crate::vm::ffi::dep_methods::display_string(
                        task,
                        vm,
                        handle,
                        &short_type,
                    )? {
                        Some(t) => t,
                        None => format!("<obj:{}>", handle),
                    };
                    vm_write(vm, &text);
                } else {
                    vm_write(vm, &format!("<obj:{}>", handle));
                }
            } else {
                vm_write(vm, &format!("<invalid object: {}>", handle));
            }
        } else {
            let val = auto_val::decode_i32(nv);
            vm_write(vm, &val.to_string());
        }
    }
    Ok(())
}

/// log no-op — swallows all arguments, returns nothing.
/// Used for env_logger.init(), log::set_max_level(), tracing::init(), etc.
pub fn shim_log_noop(_task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Arguments are already consumed by the caller's stack management.
    // Nothing to do — these functions are pure side-effects we ignore.
    Ok(())
}
