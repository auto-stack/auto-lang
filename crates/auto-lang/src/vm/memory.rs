use crate::eval::Evaler;
/// Memory management functions for VM storage strategies
/// Used by vm/storage.rs for Heap<T> implementation
use auto_val::Value;

/// Allocate a new array of the given size
pub fn alloc_array(_evaler: &mut Evaler, size_val: Value) -> Value {
    let size = match size_val {
        Value::Uint(n) => n as usize,
        Value::Int(n) => n as usize,
        _ => return Value::Error("alloc_array: invalid size".into()),
    };

    // Initialize with Nil
    let vec = vec![Value::Nil; size];
    // Use Value::array_of which creates Array from Vec<Value>
    // Note: Value::array_of expects items that impl Into<Value>. Value impls Into<Value>.
    Value::array_of(vec)
}

/// Reallocate an existing array to a new size
/// Returns a new array with copied data (or modified if owned)
pub fn realloc_array(_evaler: &mut Evaler, array: Value, size_val: Value) -> Value {
    let new_size = match size_val {
        Value::Uint(n) => n as usize,
        Value::Int(n) => n as usize,
        _ => return Value::Error("realloc_array: invalid size".into()),
    };

    match array {
        Value::Array(mut arr) => {
            // Access underlying Vec via .values (assuming Array struct has pub values)
            // or use provided methods if fields are private.
            // Based on vm/list.rs usage 'array.values', it seems public.
            arr.values.resize(new_size, Value::Nil);
            Value::Array(arr)
        }
        _ => Value::Error("realloc_array: not an array".into()),
    }
}

/// Free an array (Hint to VM)
pub fn free_array(_evaler: &mut Evaler, _array: Value) -> Value {
    // No-op in this Rust-based VM as memory is managed by Rc/Drop
    Value::Nil
}

/// Wrapper for realloc_array to be used as a VmFunction
/// Expects args to be an Array where [0] is array, [1] is new_size
pub fn realloc_array_wrapped(evaler: &mut Evaler, args: Value) -> Value {
    match args {
        // Standard calling convention passes arguments as an Array
        Value::Array(arr) => {
            // Check argument count
            if arr.values.len() < 2 {
                return Value::Error("realloc_array requires 2 arguments".into());
            }
            // Clone arguments as we need to pass them to realloc_array
            let array = arr.values[0].clone();
            let size = arr.values[1].clone();
            realloc_array(evaler, array, size)
        }
        _ => Value::Error("realloc_array: invalid arguments format".into()),
    }
}
