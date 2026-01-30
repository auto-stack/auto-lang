//! VM Memory Management Functions for Runtime Arrays
//!
//! This module provides low-level memory operations for AutoLang code
//! implementing self-hosted data structures (e.g., List<T>).
//!
//! # Purpose
//!
//! These functions enable AutoLang code to manually manage memory for
//! dynamic data structures, similar to how C uses malloc/realloc/free.
//!
//! # Functions
//!
//! - `alloc_array<T>(size: int) -> Array`: Allocate array with given capacity
//! - `free_array<T>(arr: Array) -> void`: Free array (no-op in VM with GC)
//! - `realloc_array<T>(arr: Array, new_size: int) -> Array`: Reallocate with new capacity
//!
//! # Example Usage in AutoLang
//!
//! ```auto
//! type List<T> {
//!     data [runtime]T
//!     capacity int
//!
//!     fn push(elem T) {
//!         if .len >= .capacity {
//!             .data = realloc_array(.data, .capacity * 2)
//!             .capacity = .capacity * 2
//!         }
//!         .data[.len] = elem
//!         .len = .len + 1
//!     }
//! }
//! ```

use auto_val::{Array, Shared, Value};
use crate::Universe;

/// Allocate a new runtime array with the specified capacity
///
/// # Arguments
///
/// * `uni` - Universe reference for type lookup
/// * `size` - Array capacity (must be positive integer)
///
/// # Returns
///
/// * `Value::Array` - New array with `size` capacity (all elements initialized to Nil)
/// * `Value::Error` - If size is invalid
///
/// # VM Signature
///
/// ```auto
/// #[vm]
/// fn alloc_array<T>(size: int) -> [runtime]T
/// ```
///
/// # Example
///
/// ```auto
/// let arr = alloc_array<int>(10)  // Create array with capacity 10
/// ```
pub fn alloc_array(_uni: Shared<Universe>, size: Value) -> Value {
    match size {
        Value::Int(n) if n > 0 => {
            // Create Vec with specified capacity
            let mut values = Vec::with_capacity(n as usize);
            // Initialize all elements to Nil
            values.resize(n as usize, Value::Nil);

            Value::Array(Array { values })
        }
        Value::Int(0) | Value::Uint(0) => {
            // Empty array is allowed
            Value::Array(Array { values: Vec::new() })
        }
        _ => Value::Error(
            format!("alloc_array: invalid size {}, must be positive integer",
                match &size {
                    Value::Int(n) => n.to_string(),
                    Value::Uint(n) => n.to_string(),
                    _ => "?".to_string(),
                }
            ).into()
        ),
    }
}

/// Free an array (no-op in VM with garbage collection)
///
/// # Arguments
///
/// * `uni` - Universe reference (unused, for API consistency)
/// * `array` - Array to free (ignored)
///
/// # Returns
///
/// * `Value::Nil` - Always succeeds
///
/// # VM Signature
///
/// ```auto
/// #[vm]
/// fn free_array<T>(arr: [runtime]T) -> void
/// ```
///
/// # Note
///
/// This function exists for API compatibility with C transpilation.
/// In the VM, garbage collection automatically handles cleanup.
/// In transpiled C code, this generates a real `free()` call.
pub fn free_array(_uni: Shared<Universe>, _array: Value) -> Value {
    // VM uses garbage collection, no explicit free needed
    // This function exists for API compatibility with C transpilation
    Value::Nil
}

/// Reallocate an array to a new capacity
///
/// # Arguments
///
/// * `uni` - Universe reference (unused, for API consistency)
/// * `array` - Original array to reallocate
/// * `new_size` - New capacity (must be larger than current)
///
/// # Returns
///
/// * `Value::Array` - New array with increased capacity, existing elements copied
/// * `Value::Error` - If parameters are invalid
///
/// # VM Signature
///
/// ```auto
/// #[vm]
/// fn realloc_array<T>(arr: [runtime]T, new_size: int) -> [runtime]T
/// ```
///
/// # Behavior
///
/// - Creates a new array with `new_size` capacity
/// - Copies all elements from the old array to the new one
/// - New elements beyond old length are initialized to Nil
/// - Old array is automatically garbage collected
///
/// # Example
///
/// ```auto
/// let arr = alloc_array<int>(5)
/// arr[0] = 1
/// arr[1] = 2
/// let new_arr = realloc_array<int>(arr, 10)  // Grow to capacity 10
/// // new_arr[0] == 1, new_arr[1] == 2, rest are Nil
/// ```
pub fn realloc_array(_uni: Shared<Universe>, array: Value, new_size: Value) -> Value {
    // Validate array type first
    let arr = match &array {
        Value::Array(a) => a,
        _ => {
            return Value::Error(
                format!("realloc_array: first argument must be Array, got {:?}", array).into()
            );
        }
    };

    // Validate new_size
    let new_cap = match new_size {
        Value::Int(n) if n > 0 => n as usize,
        Value::Int(n) => {
            return Value::Error(
                format!("realloc_array: invalid new_size {}, must be positive", n).into()
            );
        }
        _ => {
            return Value::Error(
                "realloc_array: second argument must be int".into()
            );
        }
    };

    // Create new Vec with new capacity
    let mut new_values = Vec::with_capacity(new_cap);
    new_values.resize(new_cap, Value::Nil);

    // Copy existing elements
    for (i, elem) in arr.values.iter().enumerate() {
        if i < new_cap {
            new_values[i] = elem.clone();
        }
    }

    Value::Array(Array { values: new_values })
}

/// Wrapper for realloc_array to accept two parameters as Array
///
/// VM functions (`VmFunction`) only accept single Value parameter, so we wrap the
/// two-parameter `realloc_array` in a function that accepts an Array.
///
/// # Arguments
///
/// * `uni` - Universe reference
/// * `args` - Array containing [array, new_size]
///
/// # Returns
///
/// * Result of realloc_array (new array or error)
///
/// # Note
///
/// This is a VM wrapper that unpacks arguments from an Array.
/// In AutoLang code, call it as:
/// ```auto
/// let new_arr = realloc_array_wrapped([old_arr, 10])
/// ```
///
/// The actual `realloc_array` function has signature:
/// `fn(uni, array: Value, new_size: Value) -> Value`
///
/// This wrapper adapts it to VM function signature:
/// `fn(uni, args: Value) -> Value`
pub fn realloc_array_wrapped(uni: Shared<Universe>, args: Value) -> Value {
    match args {
        Value::Array(arr) if arr.values.len() >= 2 => {
            realloc_array(uni, arr.values[0].clone(), arr.values[1].clone())
        }
        Value::Array(arr) => {
            Value::Error(
                format!("realloc_array: expected Array with 2 elements [array, new_size], got {} elements",
                    arr.values.len()).into()
            )
        }
        _ => Value::Error(
            "realloc_array: expected Array with 2 elements [array, new_size]".into()
        ),
    }
}

/// Get the capacity of an array (internal helper)
///
/// # Arguments
///
/// * `array` - Array to query
///
/// # Returns
///
/// * `Value::Int` - Array capacity
/// * `Value::Error` - If argument is not an Array
///
/// # Note
///
/// This uses `Vec::capacity()` internally, which may be larger than `len()`.
pub fn array_capacity(array: Value) -> Value {
    match array {
        Value::Array(arr) => {
            Value::Int(arr.values.capacity() as i32)
        }
        _ => Value::Error(
            format!("array_capacity: expected Array, got {:?}", array).into()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Universe;

    fn test_universe() -> Shared<Universe> {
        std::rc::Rc::new(std::cell::RefCell::new(Universe::new()))
    }

    #[test]
    fn test_alloc_array_basic() {
        let uni = test_universe();
        let result = alloc_array(uni.clone(), Value::Int(10));

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.values.len(), 10);
                // All elements should be Nil
                for elem in &arr.values {
                    assert_eq!(elem, &Value::Nil);
                }
                // Capacity should be at least 10
                assert!(arr.values.capacity() >= 10);
            }
            _ => panic!("Expected Array, got {:?}", result),
        }
    }

    #[test]
    fn test_alloc_array_empty() {
        let uni = test_universe();
        let result = alloc_array(uni, Value::Int(0));

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.values.len(), 0);
            }
            _ => panic!("Expected Array, got {:?}", result),
        }
    }

    #[test]
    fn test_alloc_array_invalid() {
        let uni = test_universe();
        let result = alloc_array(uni, Value::Int(-5));

        match result {
            Value::Error(_) => {
                // Expected error
            }
            _ => panic!("Expected Error for negative size, got {:?}", result),
        }
    }

    #[test]
    fn test_realloc_array_growth() {
        let uni = test_universe();

        // Create initial array with some data
        let arr1 = alloc_array(uni.clone(), Value::Int(5));
        let mut arr1_data = match arr1 {
            Value::Array(ref arr) => arr.clone(),
            _ => panic!("Expected Array"),
        };

        // Set some values
        arr1_data.values[0] = Value::Int(1);
        arr1_data.values[1] = Value::Int(2);
        arr1_data.values[2] = Value::Int(3);

        // Reallocate to larger size
        let arr2 = realloc_array(uni.clone(), Value::Array(arr1_data), Value::Int(10));

        match arr2 {
            Value::Array(arr) => {
                assert_eq!(arr.values.len(), 10);
                assert_eq!(arr.values[0], Value::Int(1));
                assert_eq!(arr.values[1], Value::Int(2));
                assert_eq!(arr.values[2], Value::Int(3));
                // Elements 3-9 should be Nil
                for i in 3..10 {
                    assert_eq!(arr.values[i], Value::Nil);
                }
            }
            _ => panic!("Expected Array after realloc, got {:?}", arr2),
        }
    }

    #[test]
    fn test_realloc_array_preserves_data() {
        let uni = test_universe();

        // Create array
        let arr1 = alloc_array(uni.clone(), Value::Int(3));
        let mut arr1_data = match arr1 {
            Value::Array(ref arr) => arr.clone(),
            _ => panic!("Expected Array"),
        };

        // Fill with data
        arr1_data.values[0] = Value::Str("hello".into());
        arr1_data.values[1] = Value::Int(42);
        arr1_data.values[2] = Value::Bool(true);

        // Realloc
        let arr2 = realloc_array(uni, Value::Array(arr1_data), Value::Int(5));

        match arr2 {
            Value::Array(arr) => {
                assert_eq!(arr.values[0], Value::Str("hello".into()));
                assert_eq!(arr.values[1], Value::Int(42));
                assert_eq!(arr.values[2], Value::Bool(true));
                assert_eq!(arr.values[3], Value::Nil);
                assert_eq!(arr.values[4], Value::Nil);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_free_array_no_op() {
        let uni = test_universe();
        let arr = alloc_array(uni.clone(), Value::Int(5));
        let result = free_array(uni, arr);

        // Should always return Nil (no-op in VM)
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_array_capacity_helper() {
        let uni = test_universe();
        let arr = alloc_array(uni, Value::Int(10));
        let cap = array_capacity(arr);

        match cap {
            Value::Int(n) => {
                assert!(n >= 10); // Capacity should be at least 10
            }
            _ => panic!("Expected Int, got {:?}", cap),
        }
    }
}
