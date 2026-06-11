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
    fn pop_from_stack(task: &mut AutoTask, _vm: &AutoVM) -> Result<Self, FFIError> {
        Ok(task.ram.pop_i64())
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        task.ram.push_i64(*self);
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
    fn pop_from_stack(task: &mut AutoTask, _vm: &AutoVM) -> Result<Self, FFIError> {
        Ok(task.ram.pop_u64())
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        task.ram.push_u64(*self);
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
        {
            // f64 occupies 2 slots (value at sp-2, marker at sp-1).
            // f32 occupies 1 slot. Codegen may push f32 when FFI expects f64.
            if task.ram.sp >= 2 {
                let nv = task.ram.raw_nv[task.ram.sp - 2];
                if auto_val::is_f64(nv) {
                    return Ok(task.ram.pop_f64());
                }
            }
            if task.ram.sp >= 1 {
                let nv = task.ram.raw_nv[task.ram.sp - 1];
                if auto_val::is_f32(nv) {
                    task.ram.sp -= 1;
                    return Ok(auto_val::decode_f32(nv) as f64);
                }
            }
        }
        Ok(task.ram.pop_f64())
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        task.ram.push_f64(*self);
        Ok(())
    }
}

impl VMConvertible for bool {
    fn pop_from_stack(task: &mut AutoTask, _vm: &AutoVM) -> Result<Self, FFIError> {
        let val = task.ram.pop_i32();
        Ok(val != 0)
    }

    fn push_to_stack(&self, task: &mut AutoTask, _vm: &AutoVM) -> Result<(), FFIError> {
        task.ram.push_i32(if *self { 1 } else { 0 });
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
                .get_string(str_idx as u16)
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
                    .get_string(str_idx as u16)
                    .ok_or_else(|| FFIError::InvalidStringIndex(str_idx as u16))?;
                let s = String::from_utf8_lossy(&bytes).to_string();
                return Ok(s);
            }
            return Ok(val.to_string());
        }

        Ok(format!("{:?}", nv))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        let strings = vm.strings.read().unwrap();
        let len = strings.len();
        drop(strings);

        {
            let mut strings = vm.strings.write().unwrap();
            strings.push(self.as_bytes().to_vec());
        }

        task.ram.push_str_idx(len as u32);
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
        task.ram.push_i32(list_id as i32);
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

        // Path 2: arrays (ID 2000000+) — Vec<auto_val::Value> from CREATE_ARRAY
        if let Some(array_ref) = vm.arrays.get(&list_id) {
            let guard = array_ref.read().unwrap();
            let mut result = Vec::new();
            for val in guard.iter() {
                match val {
                    auto_val::Value::Str(s) => result.push(s.as_str().to_string()),
                    auto_val::Value::Int(n) if *n < 0 => {
                        let str_idx = (-n - 1) as usize;
                        if let Some(bytes) = strings.get(str_idx) {
                            result.push(String::from_utf8_lossy(bytes).to_string());
                        }
                    }
                    _ => {}
                }
            }
            return Ok(result);
        }

        Err(FFIError::InvalidListId(list_id))
    }

    fn push_to_stack(&self, task: &mut AutoTask, vm: &AutoVM) -> Result<(), FFIError> {
        use crate::vm::types::ListData;

        // Create a ListData<i32> — the same type used by List.new/List.len/List.get.
        // Each string is registered in vm.strings and stored as a negative i32 index.
        let mut list: ListData<i32> = ListData::new();

        for s in self.iter() {
            // Register string in the string table
            let strings = vm.strings.read().unwrap();
            let len = strings.len();
            drop(strings);
            {
                let mut strings = vm.strings.write().unwrap();
                strings.push(s.as_bytes().to_vec());
            }
            // Encode as string index (negative i32), matching push_str_idx encoding
            list.push(-(len as i32) - 1);
        }

        // Register list in heap
        let list_id = vm.insert_heap_object(list);

        // Push list_id to stack
        task.ram.push_i32(list_id as i32);
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
