// VM implementation of storage strategies (Plan 052)
// This module provides VM-level implementations for Heap<T>, Inline<T>, etc.

use auto_val::{Shared, Value, Instance, Obj};
use crate::universe::Universe;

// ============================================================================
// Heap<T> Implementation
// ============================================================================

/// Create a new empty Heap storage
/// Returns an Instance with empty array and 0 capacity
pub fn heap_new(_uni: Shared<Universe>, _args: Value) -> Value {
    

    // Start with empty array (no allocation yet)
    let initial_array = Value::Array(auto_val::Array::new());

    let mut fields = Obj::new();
    fields.set("ptr", initial_array);  // Store actual array, not pointer
    fields.set("cap", Value::Uint(0));  // zero capacity

    let instance = Instance {
        ty: auto_val::Type::User("Heap".into()),
        fields,
    };

    Value::Instance(instance)
}

/// Get the data pointer from Heap storage
/// Returns the underlying array
pub fn heap_data(_uni: Shared<Universe>, self_instance: &mut Value, _args: Vec<Value>) -> Value {
    // Extract .ptr field from the Heap instance (now stores actual array)
    match self_instance {
        Value::Instance(instance) => {
            instance.fields.get_or("ptr", Value::Array(auto_val::Array::new()))
        }
        _ => Value::Error("heap_data: self is not an Instance".into()),
    }
}

/// Get the capacity from Heap storage
pub fn heap_capacity(_uni: Shared<Universe>, self_instance: &mut Value, _args: Vec<Value>) -> Value {
    // Extract .cap field from the Heap instance
    match self_instance {
        Value::Instance(instance) => {
            instance.fields.get_or("cap", Value::Int(0))
        }
        _ => Value::Error("heap_capacity: self is not an Instance".into()),
    }
}

/// Try to grow the Heap storage to minimum capacity
/// Uses alloc_array/realloc_array from VM memory module
pub fn heap_try_grow(uni: Shared<Universe>, self_instance: &mut Value, args: Vec<Value>) -> Value {
    use crate::vm::memory::{alloc_array, realloc_array};

    // Extract min_cap from args[0]
    if args.is_empty() {
        return Value::Error("heap_try_grow requires min_cap argument".into());
    }

    let min_cap = match args[0] {
        Value::Int(n) => n as u32,
        Value::Uint(n) => n,
        _ => return Value::Error("heap_try_grow: min_cap must be an integer".into()),
    };

    match self_instance {
        Value::Instance(instance) => {
            // Get current capacity
            let current_cap = instance.fields.get_or("cap", Value::Int(0));
            let cap = match current_cap {
                Value::Int(n) => n as u32,
                Value::Uint(n) => n,
                _ => return Value::Error("heap_try_grow: invalid cap field".into()),
            };

            // Calculate new capacity: max(cap * 2, min_cap)
            let new_cap = if cap == 0 {
                std::cmp::max(8, min_cap)
            } else {
                std::cmp::max(cap * 2, min_cap)
            };

            // Get current array
            let current_array = instance.fields.get_or("ptr", Value::Array(auto_val::Array::new()));

            // Allocate or reallocate memory
            let new_array = if cap == 0 {
                // First allocation
                alloc_array(uni.clone(), Value::Uint(new_cap))
            } else {
                // Reallocation: grow existing array
                realloc_array(uni.clone(), current_array, Value::Uint(new_cap))
            };

            // Check if allocation succeeded
            match new_array {
                Value::Array(_) => {
                    // Success: update instance with new array and capacity
                    let mut instance_mut = instance.clone();
                    instance_mut.fields.set("ptr", new_array);
                    instance_mut.fields.set("cap", Value::Uint(new_cap));
                    *self_instance = Value::Instance(instance_mut);
                    Value::Bool(true)
                }
                Value::Error(msg) => {
                    // Allocation failed
                    Value::Error(format!("heap_try_grow: allocation failed: {}", msg).into())
                }
                _ => {
                    Value::Error("heap_try_grow: unexpected result from allocation".into())
                }
            }
        }
        _ => Value::Error("heap_try_grow: self is not an Instance".into()),
    }
}

/// Free the Heap storage memory
pub fn heap_drop(uni: Shared<Universe>, self_instance: &mut Value, _args: Vec<Value>) -> Value {
    use crate::vm::memory::free_array;

    match self_instance {
        Value::Instance(instance) => {
            // Get the array from .ptr field
            let array_value = instance.fields.get_or("ptr", Value::Array(auto_val::Array::new()));

            match array_value {
                Value::Array(_) => {
                    // Free the array (no-op in VM with GC, but good for completeness)
                    free_array(uni, array_value);

                    // Reset fields to empty state
                    let mut instance_mut = instance.clone();
                    instance_mut.fields.set("ptr", Value::Array(auto_val::Array::new()));
                    instance_mut.fields.set("cap", Value::Uint(0));
                    *self_instance = Value::Instance(instance_mut);
                    Value::Nil
                }
                _ => {
                    // Unexpected state (non-Array in ptr field)
                    // Just reset to empty
                    let mut instance_mut = instance.clone();
                    instance_mut.fields.set("ptr", Value::Array(auto_val::Array::new()));
                    instance_mut.fields.set("cap", Value::Uint(0));
                    *self_instance = Value::Instance(instance_mut);
                    Value::Nil
                }
            }
        }
        _ => Value::Error("heap_drop: self is not an Instance".into()),
    }
}

// ============================================================================
// InlineInt64 Implementation
// ============================================================================

/// Create a new InlineInt64 storage (64-element stack-allocated array for integers)
/// Returns an Instance with buffer=[0]*64
pub fn inline_int64_new(_uni: Shared<Universe>, _args: Value) -> Value {
    // Create an InlineInt64 instance with a 64-element buffer initialized to 0
    let mut fields = Obj::new();

    // Create a 64-element array filled with zeros
    let buffer = vec![Value::Int(0); 64];
    fields.set("buffer", Value::Array(buffer.into()));

    let instance = Instance {
        ty: auto_val::Type::User("InlineInt64".into()),
        fields,
    };

    Value::Instance(instance)
}

/// Get the data pointer from InlineInt64 storage
/// Returns the buffer field (which is an Array)
pub fn inline_int64_data(_uni: Shared<Universe>, self_instance: &mut Value, _args: Vec<Value>) -> Value {
    // Extract .buffer field from the InlineInt64 instance
    match self_instance {
        Value::Instance(instance) => {
            // Return the buffer array (acts as a pointer in the VM)
            instance.fields.get_or("buffer", Value::Array(auto_val::Array::new()))
        }
        _ => Value::Error("inline_int64_data: self is not an Instance".into()),
    }
}

/// Get the capacity from InlineInt64 storage (always 64)
pub fn inline_int64_capacity(_uni: Shared<Universe>, _self_instance: &mut Value, _args: Vec<Value>) -> Value {
    // InlineInt64 always has capacity 64
    Value::Int(64)
}

/// Try to grow the InlineInt64 storage (only succeeds if min_cap <= 64)
pub fn inline_int64_try_grow(_uni: Shared<Universe>, _self_instance: &mut Value, args: Vec<Value>) -> Value {
    // Extract min_cap from args[0]
    if args.is_empty() {
        return Value::Error("inline_int64_try_grow requires min_cap argument".into());
    }

    let min_cap = match args[0] {
        Value::Int(n) => n as u32,
        Value::Uint(n) => n,
        _ => return Value::Error("inline_int64_try_grow: min_cap must be an integer".into()),
    };

    // InlineInt64 can only grow up to 64 elements
    Value::Bool(min_cap <= 64)
}

/// Free the InlineInt64 storage (no-op for stack allocation)
pub fn inline_int64_drop(_uni: Shared<Universe>, _self_instance: &mut Value, _args: Vec<Value>) -> Value {
    // No-op for stack-allocated storage
    Value::Nil
}
