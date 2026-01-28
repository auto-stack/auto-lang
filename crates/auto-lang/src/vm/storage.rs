// VM implementation of storage strategies (Plan 052)
// This module provides VM-level implementations for Heap<T>, Inline<T>, etc.

use auto_val::{Shared, Value, Instance, Obj};
use crate::universe::Universe;

// ============================================================================
// Heap<T> Implementation
// ============================================================================

/// Create a new empty Heap storage
/// Returns an Instance with ptr=0 and cap=0
pub fn heap_new(_uni: Shared<Universe>, _args: Value) -> Value {
    // Create a Heap<T> instance with nil pointer and 0 capacity
    // TODO: Need to track the generic type parameter T
    // For now, create a simple Instance with ptr and cap fields
    let mut fields = Obj::new();
    fields.set("ptr", Value::Int(0));  // null pointer
    fields.set("cap", Value::Int(0));  // zero capacity

    // TODO: Set proper type (should be Heap<T> but we don't have T yet)
    let instance = Instance {
        ty: auto_val::Type::User("Heap".into()),  // Placeholder type
        fields,
    };

    Value::Instance(instance)
}

/// Get the data pointer from Heap storage
/// VmMethod signature: (uni, &mut self, args) -> Value
pub fn heap_data(_uni: Shared<Universe>, self_instance: &mut Value, _args: Vec<Value>) -> Value {
    // Extract .ptr field from the Heap instance
    match self_instance {
        Value::Instance(instance) => {
            instance.fields.get_or("ptr", Value::Int(0))
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
pub fn heap_try_grow(_uni: Shared<Universe>, self_instance: &mut Value, args: Vec<Value>) -> Value {
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

            // TODO: Actually allocate/reallocate memory using memory::realloc_array()
            // For now, just update the capacity field
            let mut instance_mut = instance.clone();
            instance_mut.fields.set("cap", Value::Uint(new_cap));

            // Update self_instance (this is a simplified approach)
            *self_instance = Value::Instance(instance_mut);

            Value::Bool(true)
        }
        _ => Value::Error("heap_try_grow: self is not an Instance".into()),
    }
}

/// Free the Heap storage memory
pub fn heap_drop(_uni: Shared<Universe>, self_instance: &mut Value, _args: Vec<Value>) -> Value {
    match self_instance {
        Value::Instance(instance) => {
            // Get the pointer value
            let ptr_value = instance.fields.get_or("ptr", Value::Int(0));
            match ptr_value {
                Value::Int(0) => {
                    // Already null, nothing to free
                    Value::Nil
                }
                _ => {
                    // TODO: Call memory::free_array() on the pointer
                    // For now, just set ptr to null
                    let mut instance_mut = instance.clone();
                    instance_mut.fields.set("ptr", Value::Int(0));
                    instance_mut.fields.set("cap", Value::Int(0));
                    *self_instance = Value::Instance(instance_mut);
                    Value::Nil
                }
            }
        }
        _ => Value::Error("heap_drop: self is not an Instance".into()),
    }
}
