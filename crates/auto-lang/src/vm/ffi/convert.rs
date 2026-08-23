//! VMConvertible trait for automatic type conversion between Rust and AutoVM
//!
//! This trait enables seamless conversion between Rust types and VM values,
//! reducing boilerplate in FFI shims.

use super::error::FFIError;
use crate::vm::engine::AutoVM;
use crate::vm::task::AutoTask;

/// Trait for types that can cross the FFI boundary
///
/// This trait provides automatic conversion between Rust types and AutoVM values.
/// Implementations are provided for common types like String, i32, bool, etc.
///
/// # Example
///
/// ```rust,ignore
/// use auto_lang::vm::ffi::VMConvertible;
///
/// // Convert from VM to Rust
/// let path: String = String::from_vm(&vm_value, &vm)?;
///
/// // Convert from Rust to VM
/// let result = "hello".to_string();
/// let vm_value = result.to_vm(&mut vm)?;
/// ```
pub trait VMConvertible: Sized {
    /// Convert from AutoVM stack to Rust type
    ///
    /// Pops values from the stack and converts them to the Rust type.
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError>;

    /// Convert from Rust type to AutoVM stack
    ///
    /// Pushes the converted values onto the stack.
    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError>;
}

// ============================================================================
// Primitive Type Implementations
// ============================================================================

impl VMConvertible for i32 {
    fn pop_from_stack(task: &mut AutoTask, _vm: &AutoVM) -> Result<Self, FFIError> {
        Ok(task.ram.pop_i32())
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        task.ram.push_i32(*self);
        Ok(())
    }
}

impl VMConvertible for i64 {
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError> {
        // Plan 377: 单槽。TAG_BIGINT 时解引用堆对象读完整 64 位。
        let nv = task.ram.pop_nv();
        Ok(decode_i64_full(vm, nv))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        // Plan 377: 单槽。48 位内联；>2^47 堆装箱（保留完整 64 位范围）。
        task.ram.push_nv(encode_i64_with_heap(vm, *self));
        Ok(())
    }
}

impl VMConvertible for u32 {
    fn pop_from_stack(task: &mut AutoTask, _vm: &AutoVM) -> Result<Self, FFIError> {
        Ok(task.ram.pop_u32())
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        task.ram.push_u32(*self);
        Ok(())
    }
}

impl VMConvertible for u64 {
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError> {
        // Plan 377: 单槽。TAG_BIGINT 时解引用堆对象读完整 64 位。
        let nv = task.ram.pop_nv();
        Ok(decode_u64_full(vm, nv))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        // Plan 377: 单槽。48 位内联；>2^48 堆装箱（保留完整 64 位范围）。
        task.ram.push_nv(encode_u64_with_heap(vm, *self));
        Ok(())
    }
}

impl VMConvertible for f32 {
    fn pop_from_stack(task: &mut AutoTask, _vm: &AutoVM) -> Result<Self, FFIError> {
        Ok(task.ram.pop_f32())
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        task.ram.push_f32(*self);
        Ok(())
    }
}

impl VMConvertible for f64 {
    fn pop_from_stack(task: &mut AutoTask, _vm: &AutoVM) -> Result<Self, FFIError> {
        // Plan 377: 单槽。兼容 codegen 推 f32（FFI 期望 f64）的情况 —— 提升 f32→f64。
        let nv = task.ram.peek_nv(0);
        if auto_val::is_f32(nv) {
            task.ram.sp -= 1;
            return Ok(auto_val::decode_f32(nv) as f64);
        }
        Ok(task.ram.pop_f64())
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        // Plan 377: 全值单槽化后 f64 原生单槽（不再截断为 f32）。
        task.ram.push_f64(*self);
        Ok(())
    }
}

impl VMConvertible for bool {
    fn pop_from_stack(task: &mut AutoTask, _vm: &AutoVM) -> Result<Self, FFIError> {
        let nv = task.ram.pop_nv();
        if auto_val::is_bool(nv) {
            Ok(auto_val::decode_bool(nv))
        } else {
            // Backward compat: treat nonzero i32 as true
            Ok(auto_val::decode_i32(nv) != 0)
        }
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        task.ram.push_nv(auto_val::encode_bool(*self));
        Ok(())
    }
}

impl VMConvertible for () {
    fn pop_from_stack(_task: &mut AutoTask, _vm: &AutoVM) -> Result<Self, FFIError> {
        Ok(())
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        // Push unit (0) as return value
        task.ram.push_i32(0);
        Ok(())
    }
}

// ============================================================================
// String Implementation
// ============================================================================

impl VMConvertible for String {
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError> {
        let nv = task.ram.pop_nv();

        if auto_val::is_string(nv) {
            let str_idx = auto_val::decode_string(nv) as usize;
            let bytes = vm
                .get_string(str_idx as u32)
                .ok_or_else(|| FFIError::InvalidStringIndex(str_idx as u16))?;
            let s = String::from_utf8_lossy(&bytes).to_string();
            return Ok(s);
        }

        // Handle i32 values (legacy tags or integer-to-string conversion)
        if auto_val::is_i32(nv) {
            let val = auto_val::decode_i32(nv);
            if val < 0 {
                let str_idx = (-val - 1) as usize;
                let bytes = vm
                    .get_string(str_idx as u32)
                    .ok_or_else(|| FFIError::InvalidStringIndex(str_idx as u16))?;
                let s = String::from_utf8_lossy(&bytes).to_string();
                return Ok(s);
            }
            return Ok(val.to_string());
        }

        Ok(format!("{:?}", nv))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        let len = vm.add_string(self.as_bytes().to_vec());
        vm.rc_push_str_idx(task, len as usize);
        Ok(())
    }
}

// ============================================================================
// Option Implementation
// ============================================================================

impl<T: VMConvertible> VMConvertible for Option<T> {
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError> {
        // Option is represented as: tag (i32) + value (if Some)
        let tag = task.ram.pop_i32();

        if tag == 0 {
            // None - no value follows
            Ok(None)
        } else {
            // Some - value follows
            let value = T::pop_from_stack(task, vm)?;
            Ok(Some(value))
        }
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        match self {
            None => {
                // Push None tag
                task.ram.push_i32(0);
            }
            Some(value) => {
                // Push value first, then Some tag
                value.push_to_stack(task, vm)?;
                task.ram.push_i32(1);
            }
        }
        Ok(())
    }
}

// ============================================================================
// Result Implementation
// ============================================================================

impl<T: VMConvertible, E: std::fmt::Display> VMConvertible for Result<T, E> {
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError> {
        // Result is represented as: tag (i32) + value (if Ok)
        // For now, we'll just try to pop the Ok value
        // Error handling would need more infrastructure
        let value = T::pop_from_stack(task, vm)?;
        Ok(Ok(value))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        match self {
            Ok(value) => {
                // Only push the inner value (AutoVM expects single return value)
                value.push_to_stack(task, vm)?;
            }
            Err(_) => {
                // Push 0 as error indicator (Auto code checks for 0/false/null)
                task.ram.push_i32(0);
            }
        }
        Ok(())
    }
}

// ============================================================================
// Vec<i32> Implementation (List) - MVP
// ============================================================================

impl VMConvertible for Vec<i32> {
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError> {
        // List is represented as list_id (i32/u64)
        let list_id = task.ram.pop_i32() as u64;

        // Get list from heap
        let obj = vm
            .get_heap_object(list_id)
            .ok_or(FFIError::InvalidListId(list_id))?;

        let guard = obj.read().unwrap();

        // Try to downcast to ListData<i32>
        if let Some(list_data) = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<i32>>()
        {
            let mut result = Vec::new();
            for i in 0..list_data.len() {
                let elem = list_data.get(i).copied().unwrap_or(0);
                result.push(elem);
            }
            return Ok(result);
        }

        Err(FFIError::InvalidListId(list_id))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        use crate::vm::types::ListData;

        // Create a new list
        let mut list: ListData<i32> = ListData::new();

        // Push all elements
        for &elem in self.iter() {
            list.push(elem);
        }

        // Register list in heap
        let list_id = vm.insert_heap_object(list);

        // Push list_id to stack
        vm.rc_push_id(task, list_id as u64); // Plan 419
        Ok(())
    }
}

// ============================================================================
// Vec<String> Implementation (List of Strings)
// ============================================================================

impl VMConvertible for Vec<String> {
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError> {
        let list_id = task.ram.pop_i32() as u64;
        let strings = vm.strings.read().unwrap();

        // Path 1: heap_objects (ID 4000000+) — ListData<i32> or ListData<Value>
        if let Some(obj) = vm.get_heap_object(list_id) {
            let guard = obj.read().unwrap();

            // ListData<auto_val::Value>
            if let Some(list_data) = guard
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
            {
                let mut result = Vec::new();
                for i in 0..list_data.len() {
                    if let Some(auto_val::Value::Str(s)) = list_data.get(i) {
                        result.push(s.as_str().to_string());
                    }
                }
                return Ok(result);
            }

            // ListData<i32> — strings stored as negative indices
            if let Some(list_data) = guard
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<i32>>()
            {
                let mut result = Vec::new();
                for i in 0..list_data.len() {
                    if let Some(&val) = list_data.get(i) {
                        if val < 0 {
                            let str_idx = (-val - 1) as usize;
                            if let Some(bytes) = strings.get(str_idx) {
                                result.push(String::from_utf8_lossy(bytes).to_string());
                            }
                        }
                    }
                }
                return Ok(result);
            }
        }

        // Plan 390 §15 H3b: array literals are ListData<Value> in heap_objects,
        // handled by Path 1 above (the legacy arrays registry is gone).

        Err(FFIError::InvalidListId(list_id))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        use crate::vm::types::ListData;

        // Create a ListData<i32> — the same type used by List.new/List.len/List.get.
        // Each string is registered in vm.strings and stored as a negative i32 index.
        let mut list: ListData<i32> = ListData::new();

        for s in self.iter() {
            // Register string in the string table
            let len = vm.add_string(s.as_bytes().to_vec());
            // Encode as string index (negative i32), matching push_str_idx encoding
            list.push(-(len as i32) - 1);
        }

        // Register list in heap
        let list_id = vm.insert_heap_object(list);

        // Push list_id to stack
        vm.rc_push_id(task, list_id as u64); // Plan 419
        Ok(())
    }
}

// ============================================================================
// Tuple Implementations
// ============================================================================

impl<T1: VMConvertible, T2: VMConvertible> VMConvertible for (T1, T2) {
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError> {
        // Tuples are pushed in order, so we pop in reverse order
        let t2 = T2::pop_from_stack(task, vm)?;
        let t1 = T1::pop_from_stack(task, vm)?;
        Ok((t1, t2))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        self.0.push_to_stack(task, vm)?;
        self.1.push_to_stack(task, vm)?;
        Ok(())
    }
}

impl<T1: VMConvertible, T2: VMConvertible, T3: VMConvertible> VMConvertible for (T1, T2, T3) {
    fn pop_from_stack(task: &mut AutoTask, vm: &AutoVM) -> Result<Self, FFIError> {
        let t3 = T3::pop_from_stack(task, vm)?;
        let t2 = T2::pop_from_stack(task, vm)?;
        let t1 = T1::pop_from_stack(task, vm)?;
        Ok((t1, t2, t3))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        self.0.push_to_stack(task, vm)?;
        self.1.push_to_stack(task, vm)?;
        self.2.push_to_stack(task, vm)?;
        Ok(())
    }
}

// ============================================================================
// Plan 377: BigInt 堆装箱辅助函数（>2^48 的 i64/u64 完整范围兜底）
// ============================================================================

use auto_val::{NanoValue, try_encode_i64, try_encode_u64, decode_i64, decode_u64,
    decode_i32, encode_bigint, is_bigint, decode_bigint_handle, tag_of};
use crate::vm::heap_object::{BigIntData, downcast};

/// 将 i64 编码为单槽 NanoValue：48 位内联，否则堆装箱（TAG_BIGINT handle）。
pub fn encode_i64_with_heap(vm: &AutoVM, val: i64) -> NanoValue {
    if let Some(nv) = try_encode_i64(val) {
        nv
    } else {
        let id = vm.insert_heap_object(BigIntData::from_i64(val));
        // Plan 419: 装箱产物入栈即堆引用 —— 调用方均为直接 push,在此统一 +1。
        vm.rc_retain_id(id);
        encode_bigint(id as u32)
    }
}

/// 将 u64 编码为单槽 NanoValue：48 位内联，否则堆装箱（TAG_BIGINT handle）。
pub fn encode_u64_with_heap(vm: &AutoVM, val: u64) -> NanoValue {
    if let Some(nv) = try_encode_u64(val) {
        nv
    } else {
        let id = vm.insert_heap_object(BigIntData::from_u64(val));
        // Plan 419: 同上。
        vm.rc_retain_id(id);
        encode_bigint(id as u32)
    }
}

/// 解码 NanoValue 为 i64：TAG_I64/U64/BIGINT/i32 全支持。
/// BIGINT 时解引用堆对象读完整 64 位值。
pub fn decode_i64_full(vm: &AutoVM, nv: NanoValue) -> i64 {
    match tag_of(nv) {
        t if t == 8 => decode_i64(nv),       // TAG_I64
        t if t == 9 => decode_u64(nv) as i64, // TAG_U64
        t if t == 0xA => {                    // TAG_BIGINT
            let id = decode_bigint_handle(nv) as u64;
            if let Some(obj) = vm.get_heap_object(id) {
                if let Some(guard) = obj.read().ok() {
                    if let Some(big) = downcast::<BigIntData>(&*guard) {
                        return big.as_i64();
                    }
                }
            }
            0
        }
        _ => decode_i32(nv) as i64,           // 兼容 i32
    }
}

/// 解码 NanoValue 为 u64：TAG_U64/I64/BIGINT/i32 全支持。
/// BIGINT 时解引用堆对象读完整 64 位值。
pub fn decode_u64_full(vm: &AutoVM, nv: NanoValue) -> u64 {
    match tag_of(nv) {
        t if t == 9 => decode_u64(nv),        // TAG_U64
        t if t == 8 => decode_i64(nv) as u64, // TAG_I64
        t if t == 0xA => {                    // TAG_BIGINT
            let id = decode_bigint_handle(nv) as u64;
            if let Some(obj) = vm.get_heap_object(id) {
                if let Some(guard) = obj.read().ok() {
                    if let Some(big) = downcast::<BigIntData>(&*guard) {
                        return big.as_u64();
                    }
                }
            }
            0
        }
        _ => decode_i32(nv) as u32 as u64,    // 兼容 i32
    }
}

/// 判定 NanoValue 是否为 BIGINT（供算术 opcode 慢路径检测用）。
#[allow(dead_code)]
pub fn nv_is_bigint(nv: NanoValue) -> bool { is_bigint(nv) }
