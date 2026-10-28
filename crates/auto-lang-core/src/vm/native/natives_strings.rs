//! PLAN-843 结构拆分：native strings 域实现（自 native.rs 原样搬移）。
use super::*;

/// Create a new StringBuilder
/// Stack: capacity -> sb_id
pub fn shim_stringbuilder_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _capacity = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake__capacity = crate::vm::native::StakeGuard::new(vm, _capacity as i64 as u64);

    let sb = crate::vm::collections::SpecializedStringBuilder::with_capacity(_capacity.max(16));
    let sb_id = vm.insert_heap_object(sb);

    vm.rc_push_id(task, sb_id as u64); // Plan 419
    Ok(())
}

/// Append a string to the StringBuilder
/// Stack: sb_id, str_id -> result (0)
pub fn shim_stringbuilder_append(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx();
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(sb_id) {
        let bytes = vm
            .strings
            .read()
            .unwrap()
            .get(str_idx)
            .cloned()
            .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
        let s = String::from_utf8_lossy(&bytes).to_string();
        drop(bytes);

        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
        {
            sb.buffer.push_str(&s);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Append an integer to the StringBuilder
/// Stack: sb_id, int_val -> result (0)
pub fn shim_stringbuilder_append_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let int_val = crate::vm::native::pop_arg_i32(task);

    let _stake_int_val = crate::vm::native::StakeGuard::new(vm, int_val as i64 as u64);
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
        {
            sb.buffer.push_str(&int_val.to_string());
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Append a character to the StringBuilder
/// Stack: sb_id, char_val -> result (0)
pub fn shim_stringbuilder_append_char(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let char_bits = crate::vm::native::pop_arg_i32(task);

    let _stake_char_bits = crate::vm::native::StakeGuard::new(vm, char_bits as i64 as u64);
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    // Decode character (char is stored as i32 representing a Unicode code point)
    if let Some(ch) = char::from_u32(char_bits as u32) {
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let mut guard = obj.write().unwrap();
            if let Some(sb) = guard
                .as_any_mut()
                .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
            {
                sb.buffer.push(ch);
            }
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get the length of the StringBuilder content
/// Stack: sb_id -> length
pub fn shim_stringbuilder_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    let len = if let Some(obj) = vm.get_heap_object(sb_id) {
        let guard = obj.read().unwrap();
        if let Some(sb) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedStringBuilder>()
        {
            sb.buffer.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(len);
    Ok(())
}

/// Clear the StringBuilder
/// Stack: sb_id -> result (0)
pub fn shim_stringbuilder_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
        {
            sb.buffer.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Drop the StringBuilder (no-op, managed by Arc)
/// Stack: sb_id -> result (0)
pub fn shim_stringbuilder_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);
    vm.rc_release_id(sb_id);
    Ok(())
}

/// Build the final string from StringBuilder
/// Stack: sb_id -> str_id (tagged string index)
pub fn shim_stringbuilder_build(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    let result_str = if let Some(obj) = vm.get_heap_object(sb_id) {
        let guard = obj.read().unwrap();
        if let Some(sb) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedStringBuilder>()
        {
            sb.buffer.clone()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Store the string in the string pool and return tagged index
    let str_idx = vm.add_string(result_str.into_bytes());

    // Return as tagged string index
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Get the length of a string from the constant pool.
/// Stack: str_idx (tagged) -> length (as i32)
pub fn shim_str_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // When compiler can't determine receiver type, it may call str.len on a List heap ID.
    // Detect this and dispatch to list.len instead.
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        // PLAN-057 T6（JSON.parse 数组形态接线）：堆句柄判定扩展——
        // ①编码从裸 i32 扩到 object/list tag（json.parse 数组产物是
        //   encode_object 推栈的 ListData<Value>，此前只认 i32 ≥4000000）；
        // ②变体从 ListData<i32> 扩到全四型（Value/String/bool 同为列表）。
        //   原 i32-tag + 非 i32 列表形态落「字符串路径」按池索引解码 → 恒 0
        //   （case_web_builtins H 现场）。
        let heap_id = if auto_val::is_object(nv) {
            Some(auto_val::decode_object(nv) as u64)
        } else if auto_val::is_list(nv) {
            Some(auto_val::decode_list(nv) as u64)
        } else if auto_val::is_i32(nv) && auto_val::decode_i32(nv) >= 4000000 {
            Some(auto_val::decode_i32(nv) as u64)
        } else {
            None
        };
        if let Some(id) = heap_id {
            use crate::vm::types::ListData;
            if let Some(obj) = vm.get_heap_object(id) {
                let guard = obj.read().unwrap();
                let len = if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                    list.len() as i32
                } else if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>() {
                    list.len() as i32
                } else if let Some(list) = guard.as_any().downcast_ref::<ListData<bool>>() {
                    list.len() as i32
                } else if let Some(list) =
                    guard.as_any().downcast_ref::<ListData<auto_val::Value>>()
                {
                    list.len() as i32
                } else if let Some(fo) = guard.as_foreign_object() {
                    // Plan 569 D2: py 句柄误入 str.len（存量编译产物/侧表漏报
                    // 兜底）——GIL len（与 obj_len 组合子外对象臂同语义）。
                    // 根治归侧表分派，此臂为兜底加固。
                    return fo.obj_len(task, vm);
                } else {
                    // 堆上非列表（对象/实例）：按 0 走（.length 对非数组无 JS 语义）
                    0
                };
                task.ram.push_i32(len);
            } else {
                task.ram.push_i32(0);
            }
            return Ok(());
        }
        // Normal string path
        // PLAN-055: JS .length 语义 = 字符数（web a2ts `text.length` 同值）。
        // 此前 bytes.len() 按 UTF-8 字节计——CJK ×3 膨胀，musk estimateTokens
        // 的 nonCjk 基数失真（思考块 "1 tokens" 根因链一环，P055-M 实测）。
        let str_idx = auto_val::decode_string(nv);
        if let Some(bytes) = vm.get_string(str_idx) {
            let char_count = String::from_utf8_lossy(&bytes).chars().count() as i32;
            task.ram.push_i32(char_count);
        } else {
            task.ram.push_i32(0);
        }
        return Ok(());
    }
}

/// str.contains(substring) — check if string contains substring
/// Stack: substring, string -> bool
pub fn shim_str_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let sub_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_sub_nv = crate::vm::native::StakeGuard::nv(vm, sub_nv);
        let str_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_str_nv = crate::vm::native::StakeGuard::nv(vm, str_nv);
        let str_s = nv_to_string(str_nv, vm);
        let sub_s = nv_to_string(sub_nv, vm);
        task.ram
            .push_nv(auto_val::encode_bool(str_s.contains(sub_s.as_str())));
    }
    Ok(())
}

/// str.starts_with(prefix) — check if string starts with prefix
pub fn shim_str_starts_with(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let prefix_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_prefix_nv = crate::vm::native::StakeGuard::nv(vm, prefix_nv);
        let str_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_str_nv = crate::vm::native::StakeGuard::nv(vm, str_nv);
        let str_s = nv_to_string(str_nv, vm);
        let prefix_s = nv_to_string(prefix_nv, vm);
        task.ram
            .push_nv(auto_val::encode_bool(str_s.starts_with(prefix_s.as_str())));
    }
    Ok(())
}

/// str.ends_with(suffix) — check if string ends with suffix
pub fn shim_str_ends_with(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let suffix_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_suffix_nv = crate::vm::native::StakeGuard::nv(vm, suffix_nv);
        let str_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_str_nv = crate::vm::native::StakeGuard::nv(vm, str_nv);
        let str_s = nv_to_string(str_nv, vm);
        let suffix_s = nv_to_string(suffix_nv, vm);
        task.ram
            .push_nv(auto_val::encode_bool(str_s.ends_with(suffix_s.as_str())));
    }
    Ok(())
}

/// str.to_int() / str.parse_int() — parse string to int, return Result<int>
/// Stack: str -> CREATE_OK(int) or CREATE_ERR(str)
/// Returns the value as CREATE_OK for proper .?() chaining
pub fn shim_str_to_int_nv(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        let s = if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv);
            vm.get_string(idx as u32)
                .map(|b| String::from_utf8_lossy(&b[..]).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let result = s.trim().parse::<i32>().unwrap_or(0);
        task.ram.push_nv(auto_val::encode_i32(result));
    }
    Ok(())
}

/// Nanbox-aware shim for String.to_uint().
///
/// Returns a real 2-slot i64 (low/high), matching the native catalog
/// registration (`str.to_uint` → I64) and the stdlib declaration
/// (`fn to_uint(s str) i64`). Plan 073 Stage A added `push_i64` to the
/// nanbox VM, so the earlier i32-truncation workaround (which corrupted
/// `let x u64 = s.to_uint()` and `s.to_uint() + 1.5`) is obsolete.
///
/// Plan 378: restored the full i64 range and 2-slot stack representation.
pub fn shim_str_to_uint_nv(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        let s = if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv);
            vm.get_string(idx as u32)
                .map(|b| String::from_utf8_lossy(&b[..]).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let result = s.trim().parse::<i64>().unwrap_or(0);
        // Plan 377: heap-aware push —— >2^47 经 BigInt 堆装箱（完整 64 位范围）。
        vm.push_i64_vm(task, result);
    }
    Ok(())
}

/// Get the length of a string (String.len).
/// Supports both constant pool strings (tagged index) and heap-based mutable Strings (SpecializedStringBuilder).
/// Stack: str_idx_or_sb_id -> length (as i32, char count for heap, byte count for const pool)
pub fn shim_string_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);

        // If it's a string, compute byte length directly
        if auto_val::is_string(nv) {
            let str_idx = auto_val::decode_string(nv);
            if let Some(bytes) = vm.get_string(str_idx) {
                task.ram.push_i32(bytes.len() as i32);
            } else {
                task.ram.push_i32(0);
            }
            return Ok(());
        }

        // Otherwise try as heap object ID
        let sb_id = auto_val::decode_i32(nv) as u64;
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let guard = obj.read().unwrap();
            if let Some(sb) = guard
                .as_any()
                .downcast_ref::<crate::vm::collections::SpecializedStringBuilder>()
            {
                task.ram.push_i32(sb.buffer.chars().count() as i32);
                return Ok(());
            }
        }

        task.ram.push_i32(0);
        Ok(())
    }
}

/// Plan 118 Phase 4: Create a new mutable string with initial content and capacity.
/// Stack: capacity (i32), initial_str_idx (tagged) -> mut_str_id (i32)
/// The mutable string is stored in heap_objects as a SpecializedStringBuilder.
pub fn shim_str_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop capacity (not used in this simple implementation)
    let _capacity = crate::vm::native::pop_arg_i32(task);

    let _stake__capacity = crate::vm::native::StakeGuard::new(vm, _capacity as i64 as u64);

    // Pop initial string index
    let str_idx = task.ram.pop_str_idx() as u32;

    // Get initial string content
    let initial_content = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(bytes.as_slice()).to_string()
    } else {
        String::new()
    };

    // Create a SpecializedStringBuilder with initial content
    let mut builder = crate::vm::collections::SpecializedStringBuilder::new();
    builder.buffer = initial_content;

    // Store in heap_objects
    let obj_id = vm.heap_object_id_gen.fetch_add(1, Ordering::SeqCst);
    let obj: Arc<RwLock<dyn crate::vm::heap_object::HeapObject>> = Arc::new(RwLock::new(builder));
    vm.heap_objects.insert(obj_id, obj);
    // Plan 419: 手写 insert 与 insert_heap_object 保持同一计数口径。
    vm.rc_created_total
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Return object ID
    vm.rc_push_id(task, obj_id as u64); // Plan 419
    Ok(())
}

/// Plan 118 Phase 4: Append a string to a mutable string.
/// Stack: str_idx (tagged), mut_str_id (i32) -> mut_str_id (i32)
pub fn shim_str_append(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop string to append
    let str_idx = task.ram.pop_str_idx() as u32;

    // Pop mutable string ID
    let obj_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_obj_id = crate::vm::native::StakeGuard::new(vm, obj_id as i64 as u64);

    // Get string to append
    let to_append = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(bytes.as_slice()).to_string()
    } else {
        String::new()
    };

    // Get and modify the mutable string
    if let Some(obj_arc) = vm.heap_objects.get(&obj_id) {
        let mut obj = obj_arc.write().unwrap();
        if let Some(builder) = obj
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
        {
            builder.buffer.push_str(&to_append);
        }
    }

    // Return the same mutable string ID
    task.ram.push_i32(obj_id as i32);
    Ok(())
}

/// Plan 118 Phase 4: Convert string to uppercase.
/// Stack: str_idx (tagged) -> str_idx (tagged)
pub fn shim_str_upper(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop string index
    let str_idx = task.ram.pop_str_idx() as u32;

    // Get string content
    let upper_str = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(bytes.as_slice()).to_uppercase()
    } else {
        String::new()
    };

    // Add to string pool
    let bytes = upper_str.into_bytes();
    let new_idx = vm.add_string(bytes);

    // Return tagged string index
    vm.rc_push_str_idx(task, new_idx as usize);
    Ok(())
}

/// str.bytes() -> iterator of byte values
/// Creates a list of i32 byte values from a string and returns a ListIterator.
/// Stack: str_idx (tagged) -> iterator_id
pub fn shim_str_bytes(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::{Iterator, ListIterator};
    use crate::vm::types::ListData;

    let str_idx = task.ram.pop_str_idx() as u32;

    // Get string bytes
    let bytes: Vec<i32> = if let Some(str_bytes) = vm.get_string(str_idx) {
        str_bytes.iter().map(|&b| b as i32).collect()
    } else {
        Vec::new()
    };

    // Create a ListData<i32> with byte values
    let mut list_data: ListData<i32> = ListData::new();
    for b in bytes {
        list_data.push(b);
    }
    let list_id = vm.insert_heap_object(list_data);

    let iterator_id = vm.register_iterator(Iterator::List(ListIterator {
        list_id,
        current_index: 0,
    }));

    task.ram.push_i32(iterator_id as i32);
    Ok(())
}

/// Plan 155: String.from(str) -> String
/// Creates an owned mutable String (heap object) from a string literal.
/// Stack: str_idx (tagged) -> sb_id (heap object ID)
pub fn shim_string_from(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u32;

    // Get string content from constant pool
    let content = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(bytes.as_slice()).to_string()
    } else {
        String::new()
    };

    // Create a SpecializedStringBuilder with the content
    let mut sb = crate::vm::collections::SpecializedStringBuilder::new();
    sb.buffer = content;

    // Register in heap
    let sb_id = vm.insert_heap_object(sb);
    vm.rc_push_id(task, sb_id as u64); // Plan 419
    Ok(())
}

/// String.new() -> sb_id
/// Create a new mutable String (backed by SpecializedStringBuilder).
/// Stack: [] -> sb_id
pub fn shim_string_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb = crate::vm::collections::SpecializedStringBuilder::new();
    let sb_id = vm.insert_heap_object(sb);
    vm.rc_push_id(task, sb_id as u64); // Plan 419
    Ok(())
}

/// s.push(char) -> 0
/// Push a character (as codepoint) to the end of the mutable string.
/// Stack: [char_codepoint, sb_id] -> 0
pub fn shim_string_push(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let char_codepoint = crate::vm::native::pop_arg_i32(task);

    let _stake_char_codepoint =
        crate::vm::native::StakeGuard::new(vm, char_codepoint as i64 as u64);
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    if let Some(ch) = char::from_u32(char_codepoint as u32) {
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let mut guard = obj.write().unwrap();
            if let Some(sb) = guard
                .as_any_mut()
                .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
            {
                sb.buffer.push(ch);
            }
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// s.pop() -> char_codepoint (0 if empty)
/// Pop the last character from the mutable string.
/// Stack: [sb_id] -> char_codepoint
pub fn shim_string_pop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
        {
            match sb.buffer.pop() {
                Some(ch) => ch as i32,
                None => 0,
            }
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// s.get(index) -> char_codepoint
/// Get the character at the given char index.
/// Stack: [index, sb_id] -> char_codepoint
pub fn shim_string_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let index = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_index = crate::vm::native::StakeGuard::new(vm, index as i64 as u64);
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(sb_id) {
        let guard = obj.read().unwrap();
        if let Some(sb) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedStringBuilder>()
        {
            sb.buffer
                .chars()
                .nth(index)
                .map(|ch| ch as i32)
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// s.set(index, char) -> 0
/// Replace the character at the given char index.
/// Stack: [char_codepoint, index, sb_id] -> 0
pub fn shim_string_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let char_codepoint = crate::vm::native::pop_arg_i32(task);

    let _stake_char_codepoint =
        crate::vm::native::StakeGuard::new(vm, char_codepoint as i64 as u64);
    let index = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_index = crate::vm::native::StakeGuard::new(vm, index as i64 as u64);
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    if let Some(new_ch) = char::from_u32(char_codepoint as u32) {
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let mut guard = obj.write().unwrap();
            if let Some(sb) = guard
                .as_any_mut()
                .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
            {
                // Find byte offset and old char len at char position `index`
                if let Some((byte_offset, old_ch)) = sb.buffer.char_indices().nth(index) {
                    let old_len = old_ch.len_utf8();
                    sb.buffer
                        .replace_range(byte_offset..byte_offset + old_len, &new_ch.to_string());
                }
            }
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// s.insert(index, char) -> 0
/// Insert a character at the given char index.
/// Stack: [char_codepoint, index, sb_id] -> 0
pub fn shim_string_insert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let char_codepoint = crate::vm::native::pop_arg_i32(task);

    let _stake_char_codepoint =
        crate::vm::native::StakeGuard::new(vm, char_codepoint as i64 as u64);
    let index = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_index = crate::vm::native::StakeGuard::new(vm, index as i64 as u64);
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    if let Some(ch) = char::from_u32(char_codepoint as u32) {
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let mut guard = obj.write().unwrap();
            if let Some(sb) = guard
                .as_any_mut()
                .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
            {
                // Find byte offset at char position `index`
                let byte_offset = sb
                    .buffer
                    .char_indices()
                    .nth(index)
                    .map(|(offset, _)| offset)
                    .unwrap_or(sb.buffer.len());
                sb.buffer.insert(byte_offset, ch);
            }
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// s.remove(index) -> char_codepoint
/// Remove the character at the given char index and return it.
/// Stack: [index, sb_id] -> char_codepoint
pub fn shim_string_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let index = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_index = crate::vm::native::StakeGuard::new(vm, index as i64 as u64);
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
        {
            // Find byte offset and char len at char position `index`
            if let Some((byte_offset, old_ch)) = sb.buffer.char_indices().nth(index) {
                let old_len = old_ch.len_utf8();
                let removed_codepoint = old_ch as i32;
                sb.buffer.drain(byte_offset..byte_offset + old_len);
                removed_codepoint
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// s.clear() -> 0
/// Clear all characters from the mutable string.
/// Stack: [sb_id] -> 0
pub fn shim_string_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
        {
            sb.buffer.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// s.is_empty() -> bool (1/0)
/// Check if the mutable string is empty.
/// Stack: [sb_id] -> 1/0
pub fn shim_string_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(sb_id) {
        let guard = obj.read().unwrap();
        if let Some(sb) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedStringBuilder>()
        {
            sb.buffer.is_empty()
        } else {
            true
        }
    } else {
        true
    };

    task.ram.push_nv(auto_val::encode_bool(result));
    Ok(())
}

/// s.reserve(n) -> 0
/// Reserve capacity for at least n additional bytes.
/// Stack: [n, sb_id] -> 0
pub fn shim_string_reserve(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_n = crate::vm::native::StakeGuard::new(vm, n as i64 as u64);
    let sb_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_sb_id = crate::vm::native::StakeGuard::new(vm, sb_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
        {
            sb.buffer.reserve(n);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Regex.new(pattern) → opaque handle
/// Stack: [pattern_str] -> [handle_i32]
pub fn shim_re_opaque_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let pattern = pop_vm_string(task, vm);
    match regex::Regex::new(&pattern) {
        Ok(re) => {
            let obj = RustStdlibObject::new("regex::Regex", std::sync::Mutex::new(re));
            let id = vm.insert_heap_object(obj);
            vm.rc_push_id(task, id as u64); // Plan 419
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("Regex::new failed: {}", e)));
        }
    }
    Ok(())
}

/// re.is_match(text) → bool
/// Stack: [text_str, handle_i32] -> [bool_i32]
pub fn shim_re_opaque_is_match(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let text = pop_vm_string(task, vm);
    let re_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_re_id = crate::vm::native::StakeGuard::new(vm, re_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(re_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                let result = re.lock().unwrap().is_match(&text);
                task.ram.push_nv(auto_val::encode_bool(result));
                return Ok(());
            }
        }
    }
    task.ram.push_nv(auto_val::encode_bool(false));
    Ok(())
}

/// re.find(text) → string (first match) or empty string if none
/// Stack: [text_str, handle_i32] -> [result_str]
pub fn shim_re_opaque_find(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let text = pop_vm_string(task, vm);
    let re_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_re_id = crate::vm::native::StakeGuard::new(vm, re_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(re_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                let result = re.lock().unwrap().find(&text);
                let s = result.map(|m| m.as_str().to_string()).unwrap_or_default();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// re.find_all(text) → List of matched strings
/// Stack: [text_str, handle_i32] -> [list_id_i32]
pub fn shim_re_opaque_find_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use crate::vm::types::ListData;

    let text = pop_vm_string(task, vm);
    let re_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_re_id = crate::vm::native::StakeGuard::new(vm, re_id as i64 as u64);

    let mut matches: Vec<i32> = Vec::new();
    if let Some(obj) = vm.get_heap_object(re_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                for m in re.lock().unwrap().find_iter(&text) {
                    let match_str = m.as_str();
                    let idx = vm.add_string(match_str.as_bytes().to_vec()) as u32;
                    matches.push(-(idx as i32) - 1);
                }
            }
        }
    }

    let mut list = ListData::<i32>::new();
    list.elems = matches;
    let id = vm.insert_heap_object(list);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// re.replace_all(text, replacement) → string
/// Stack: [replacement_str, text_str, handle_i32] -> [result_str]
pub fn shim_re_opaque_replace_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let replacement = pop_vm_string(task, vm);
    let text = pop_vm_string(task, vm);
    let re_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_re_id = crate::vm::native::StakeGuard::new(vm, re_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(re_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                let result = re.lock().unwrap().replace_all(&text, replacement.as_str());
                let result_str = result.to_string();
                push_vm_string(task, vm, &result_str);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, &text);
    Ok(())
}

/// re.captures(text) → opaque captures handle
/// Stores captures as Vec<String> (owned copy of each capture group)
/// Stack: [text_str, handle_i32] -> [captures_handle_i32]
pub fn shim_re_opaque_captures(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let text = pop_vm_string(task, vm);
    let re_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_re_id = crate::vm::native::StakeGuard::new(vm, re_id as i64 as u64);

    let captures: Option<Vec<String>> = {
        if let Some(obj) = vm.get_heap_object(re_id) {
            let guard = obj.read().unwrap();
            if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                    re.lock().unwrap().captures(&text).map(|caps| {
                        caps.iter()
                            .map(|m| m.map(|s| s.as_str().to_string()).unwrap_or_default())
                            .collect()
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(groups) = captures {
        let caps_obj = RustStdlibObject::new("regex::Captures", std::sync::Mutex::new(groups));
        let id = vm.insert_heap_object(caps_obj);
        vm.rc_push_id(task, id as u64); // Plan 419
    } else {
        task.ram.push_i32(0);
    }
    Ok(())
}

/// re.drop() — no-op, GC handles cleanup
pub fn shim_re_opaque_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_id = crate::vm::native::StakeGuard::new(vm, id as i64 as u64);
    vm.rc_release_id(id);
    Ok(())
}

/// base64::encode(input) → string
/// Stack: [input_str] -> [encoded_str]
pub fn shim_base64_encode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let input = pop_vm_string(task, vm);
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &input);
    push_vm_string(task, vm, &encoded);
    Ok(())
}

/// base64::decode(input) → string (decoded bytes as UTF-8 string)
/// Stack: [input_str] -> [decoded_str]
pub fn shim_base64_decode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let input = pop_vm_string(task, vm);
    match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &input) {
        Ok(bytes) => {
            let decoded = String::from_utf8_lossy(&bytes).into_owned();
            push_vm_string(task, vm, &decoded);
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!(
                "base64::decode failed: {}",
                e
            )));
        }
    }
    Ok(())
}

/// hex::encode(input) → string
/// Stack: [input_str] -> [hex_str]
pub fn shim_hex_encode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let input = pop_vm_string(task, vm);
    let encoded = hex::encode(input.as_bytes());
    push_vm_string(task, vm, &encoded);
    Ok(())
}

/// hex::decode(input) → string (decoded bytes as UTF-8 string)
/// Stack: [input_str] -> [decoded_str]
pub fn shim_hex_decode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let input = pop_vm_string(task, vm);
    match hex::decode(&input) {
        Ok(bytes) => {
            let decoded = String::from_utf8_lossy(&bytes).into_owned();
            push_vm_string(task, vm, &decoded);
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("hex::decode failed: {}", e)));
        }
    }
    Ok(())
}

/// mime_guess::from_path(path) → string (MIME type or empty)
/// Stack: [path_str] -> [mime_str]
pub fn shim_mime_from_path(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = pop_vm_string(task, vm);
    let mime = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();
    push_vm_string(task, vm, &mime);
    Ok(())
}

/// String lexicographic comparison, returns Ordering (-1, 0, 1).
/// Stack: str_idx(b), str_idx(a) -> i32 (-1, 0, 1)
pub fn shim_str_cmp(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let b_idx = task.ram.pop_str_idx() as u32;
    let a_idx = task.ram.pop_str_idx() as u32;
    let a = vm
        .get_string(a_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let b = vm
        .get_string(b_idx)
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_default();
    use std::cmp::Ordering;
    let result = match a.cmp(&b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    };
    task.ram.push_i32(result);
    Ok(())
}
