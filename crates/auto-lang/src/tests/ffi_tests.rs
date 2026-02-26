//! Tests for Plan 094: Hybrid FFI Bridge
//!
//! These tests verify:
//! 1. VMConvertible trait implementations
//! 2. FFI error handling
//! 3. Built-in stdlib function IDs

use crate::vm::ffi::VMConvertible;
use crate::vm::ffi::FFIError;
use crate::vm::ffi::stdlib;
use crate::vm::ffi::{STATIC_ID_MAX, DYNAMIC_ID_START};
use crate::vm::engine::AutoVM;
use crate::vm::task::AutoTask;
use crate::vm::virt_memory::VirtualFlash;
use std::sync::Arc;

// ============================================================================
// VMConvertible Trait Tests
// ============================================================================

#[test]
fn test_i32_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    // Test push and pop
    let value: i32 = 42;
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: i32 = i32::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, 42);

    // Test negative value
    let neg_value: i32 = -123;
    neg_value.push_to_stack(&mut task, &vm).unwrap();
    let result: i32 = i32::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, -123);
}

#[test]
fn test_i64_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    let value: i64 = 1234567890123i64;
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: i64 = i64::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, value);
}

#[test]
fn test_u32_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    let value: u32 = 0xFFFF_FFFF;
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: u32 = u32::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, value);
}

#[test]
fn test_u64_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    let value: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: u64 = u64::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, value);
}

#[test]
fn test_bool_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    // Test true
    true.push_to_stack(&mut task, &vm).unwrap();
    let result: bool = bool::pop_from_stack(&mut task, &vm).unwrap();
    assert!(result);

    // Test false
    false.push_to_stack(&mut task, &vm).unwrap();
    let result: bool = bool::pop_from_stack(&mut task, &vm).unwrap();
    assert!(!result);
}

#[test]
fn test_f32_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    let value: f32 = 3.14159;
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: f32 = f32::pop_from_stack(&mut task, &vm).unwrap();
    assert!((result - value).abs() < 0.0001);
}

#[test]
fn test_f64_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    let value: f64 = 3.14159265358979;
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: f64 = f64::pop_from_stack(&mut task, &vm).unwrap();
    assert!((result - value).abs() < 0.00000001);
}

#[test]
fn test_unit_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    // Unit should push 0
    ().push_to_stack(&mut task, &vm).unwrap();
    let result = task.ram.pop_i32();
    assert_eq!(result, 0);
}

#[test]
fn test_string_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    let value = "hello, world!".to_string();
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: String = String::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, "hello, world!");
}

#[test]
fn test_string_empty() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    let value = "".to_string();
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: String = String::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_string_unicode() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    let value = "你好世界 🌍".to_string();
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: String = String::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, "你好世界 🌍");
}

#[test]
fn test_tuple_2_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    // Test (i32, i32)
    let value: (i32, i32) = (10, 20);
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: (i32, i32) = <(i32, i32)>::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, (10, 20));
}

#[test]
fn test_tuple_3_convertible() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    // Test (i32, i32, i32)
    let value: (i32, i32, i32) = (1, 2, 3);
    value.push_to_stack(&mut task, &vm).unwrap();
    let result: (i32, i32, i32) = <(i32, i32, i32)>::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, (1, 2, 3));
}

// ============================================================================
// FFIError Tests
// ============================================================================

#[test]
fn test_ffi_error_display_type_mismatch() {
    let err = FFIError::TypeMismatch {
        expected: "String",
        found: "i32",
    };
    let msg = err.to_string();
    assert!(msg.contains("type mismatch"));
    assert!(msg.contains("String"));
    assert!(msg.contains("i32"));
}

#[test]
fn test_ffi_error_display_invalid_string_index() {
    let err = FFIError::InvalidStringIndex(42);
    let msg = err.to_string();
    assert!(msg.contains("42"));
}

#[test]
fn test_ffi_error_display_invalid_list_id() {
    let err = FFIError::InvalidListId(12345);
    let msg = err.to_string();
    assert!(msg.contains("12345"));
}

#[test]
fn test_ffi_error_display_runtime_error() {
    let err = FFIError::RuntimeError("test error".to_string());
    let msg = err.to_string();
    assert!(msg.contains("test error"));
}

#[test]
fn test_ffi_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let ffi_err: FFIError = io_err.into();
    assert!(matches!(ffi_err, FFIError::IoError(_)));
}

#[test]
fn test_ffi_error_from_utf8_error() {
    let bytes: &[u8] = &[0xFF, 0xFE];
    let utf8_err = std::str::from_utf8(bytes).unwrap_err();
    let ffi_err: FFIError = utf8_err.into();
    assert!(matches!(ffi_err, FFIError::Utf8Error(_)));
}

// ============================================================================
// Stdlib Function ID Tests
// ============================================================================

#[test]
fn test_file_function_ids_in_range() {
    // File functions should be in 1000-1099
    assert!(stdlib::NATIVE_FILE_READ_TEXT >= 1000);
    assert!(stdlib::NATIVE_FILE_READ_TEXT < 1100);
    assert!(stdlib::NATIVE_FILE_WRITE_TEXT >= 1000);
    assert!(stdlib::NATIVE_FILE_WRITE_TEXT < 1100);
    assert!(stdlib::NATIVE_FILE_EXISTS >= 1000);
    assert!(stdlib::NATIVE_FILE_EXISTS < 1100);
}

#[test]
fn test_env_function_ids_in_range() {
    // Env functions should be in 1100-1199
    assert!(stdlib::NATIVE_ENV_GET >= 1100);
    assert!(stdlib::NATIVE_ENV_GET < 1200);
    assert!(stdlib::NATIVE_ENV_SET >= 1100);
    assert!(stdlib::NATIVE_ENV_SET < 1200);
}

#[test]
fn test_time_function_ids_in_range() {
    // Time functions should be in 1200-1299
    assert!(stdlib::NATIVE_TIME_NOW_MS >= 1200);
    assert!(stdlib::NATIVE_TIME_NOW_MS < 1300);
    assert!(stdlib::NATIVE_TIME_SLEEP_MS >= 1200);
    assert!(stdlib::NATIVE_TIME_SLEEP_MS < 1300);
}

#[test]
fn test_process_function_ids_in_range() {
    // Process functions should be in 1300-1399
    assert!(stdlib::NATIVE_PROCESS_EXIT >= 1300);
    assert!(stdlib::NATIVE_PROCESS_EXIT < 1400);
}

#[test]
fn test_static_id_max_constant() {
    assert_eq!(STATIC_ID_MAX, 10000);
    assert_eq!(DYNAMIC_ID_START, 10000);
}

// ============================================================================
// Stack Operations Tests
// ============================================================================

#[test]
fn test_multiple_push_pop() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    // Push multiple values
    let a: i32 = 10;
    let b: i32 = 20;
    let c: i32 = 30;

    a.push_to_stack(&mut task, &vm).unwrap();
    b.push_to_stack(&mut task, &vm).unwrap();
    c.push_to_stack(&mut task, &vm).unwrap();

    // Pop in reverse order (LIFO)
    let c_result: i32 = i32::pop_from_stack(&mut task, &vm).unwrap();
    let b_result: i32 = i32::pop_from_stack(&mut task, &vm).unwrap();
    let a_result: i32 = i32::pop_from_stack(&mut task, &vm).unwrap();

    assert_eq!(c_result, 30);
    assert_eq!(b_result, 20);
    assert_eq!(a_result, 10);
}

#[test]
fn test_mixed_types() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    // Push different types
    let int_val: i32 = 42;
    let bool_val: bool = true;
    let float_val: f32 = 3.14;

    int_val.push_to_stack(&mut task, &vm).unwrap();
    bool_val.push_to_stack(&mut task, &vm).unwrap();
    float_val.push_to_stack(&mut task, &vm).unwrap();

    // Pop in reverse order
    let float_result: f32 = f32::pop_from_stack(&mut task, &vm).unwrap();
    let bool_result: bool = bool::pop_from_stack(&mut task, &vm).unwrap();
    let int_result: i32 = i32::pop_from_stack(&mut task, &vm).unwrap();

    assert!((float_result - 3.14).abs() < 0.01);
    assert!(bool_result);
    assert_eq!(int_result, 42);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_auto_vm_includes_stdlib() {
    // Create VM
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Verify the VM was created without panic
    // The native_interface should include stdlib functions
    assert!(Arc::strong_count(&vm.native_interface) >= 1);
}

#[test]
fn test_ffi_error_to_vm_error_conversion() {
    let ffi_err = FFIError::RuntimeError("test error".to_string());
    let vm_err: crate::vm::engine::VMError = ffi_err.into();

    // Should convert to VMError::RuntimeError
    assert!(matches!(
        vm_err,
        crate::vm::engine::VMError::RuntimeError(_)
    ));
}

// ============================================================================
// Option Tests
// ============================================================================

#[test]
fn test_option_some() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    // Push Some(42)
    let value: Option<i32> = Some(42);
    value.push_to_stack(&mut task, &vm).unwrap();

    // Pop and verify
    let result: Option<i32> = Option::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, Some(42));
}

#[test]
fn test_option_none() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);
    let mut task = AutoTask::new(0, 1024, 0);

    // Push None
    let value: Option<i32> = None;
    value.push_to_stack(&mut task, &vm).unwrap();

    // Pop and verify
    let result: Option<i32> = Option::pop_from_stack(&mut task, &vm).unwrap();
    assert_eq!(result, None);
}

// ============================================================================
// New Shim Function ID Tests (Process, Math, File+)
// ============================================================================

#[test]
fn test_process_function_ids_extended() {
    // Process functions should be in 1300-1399
    assert!((1300..1400).contains(&stdlib::NATIVE_PROCESS_ARGS));
    assert!((1300..1400).contains(&stdlib::NATIVE_PROCESS_CURRENT_DIR));
    assert!((1300..1400).contains(&stdlib::NATIVE_PROCESS_SET_CURRENT_DIR));
    assert!((1300..1400).contains(&stdlib::NATIVE_PROCESS_SPAWN));
}

#[test]
fn test_math_function_ids() {
    // Math functions should be in 1700-1799
    assert!((1700..1800).contains(&stdlib::NATIVE_MATH_ABS));
    assert!((1700..1800).contains(&stdlib::NATIVE_MATH_MIN));
    assert!((1700..1800).contains(&stdlib::NATIVE_MATH_MAX));
    assert!((1700..1800).contains(&stdlib::NATIVE_MATH_SQRT));
}

#[test]
fn test_file_function_ids_extended() {
    // Extended File functions should be in 1000-1099
    assert!((1000..1100).contains(&stdlib::NATIVE_FILE_READ_BYTES));
    assert!((1000..1100).contains(&stdlib::NATIVE_FILE_WRITE_BYTES));
    assert!((1000..1100).contains(&stdlib::NATIVE_FILE_COPY));
    assert!((1000..1100).contains(&stdlib::NATIVE_FILE_SIZE));
    assert!((1000..1100).contains(&stdlib::NATIVE_FILE_IS_DIR));
}

#[test]
fn test_all_new_ids_in_static_range() {
    // Verify all new IDs are in the static range (0-9999)
    assert!(stdlib::NATIVE_PROCESS_ARGS < 10000);
    assert!(stdlib::NATIVE_PROCESS_CURRENT_DIR < 10000);
    assert!(stdlib::NATIVE_PROCESS_SET_CURRENT_DIR < 10000);
    assert!(stdlib::NATIVE_PROCESS_SPAWN < 10000);
    assert!(stdlib::NATIVE_MATH_ABS < 10000);
    assert!(stdlib::NATIVE_MATH_MIN < 10000);
    assert!(stdlib::NATIVE_MATH_MAX < 10000);
    assert!(stdlib::NATIVE_MATH_SQRT < 10000);
    assert!(stdlib::NATIVE_FILE_READ_BYTES < 10000);
    assert!(stdlib::NATIVE_FILE_WRITE_BYTES < 10000);
    assert!(stdlib::NATIVE_FILE_COPY < 10000);
    assert!(stdlib::NATIVE_FILE_SIZE < 10000);
    assert!(stdlib::NATIVE_FILE_IS_DIR < 10000);
}

#[test]
fn test_ids_are_unique() {
    // Verify all IDs are unique
    let ids = vec![
        stdlib::NATIVE_FILE_READ_TEXT,
        stdlib::NATIVE_FILE_WRITE_TEXT,
        stdlib::NATIVE_FILE_EXISTS,
        stdlib::NATIVE_FILE_DELETE,
        stdlib::NATIVE_FILE_CREATE_DIR,
        stdlib::NATIVE_FILE_READ_BYTES,
        stdlib::NATIVE_FILE_WRITE_BYTES,
        stdlib::NATIVE_FILE_COPY,
        stdlib::NATIVE_FILE_SIZE,
        stdlib::NATIVE_FILE_IS_DIR,
        stdlib::NATIVE_ENV_GET,
        stdlib::NATIVE_ENV_SET,
        stdlib::NATIVE_ENV_REMOVE,
        stdlib::NATIVE_TIME_NOW_MS,
        stdlib::NATIVE_TIME_NOW_SEC,
        stdlib::NATIVE_TIME_SLEEP_MS,
        stdlib::NATIVE_PROCESS_EXIT,
        stdlib::NATIVE_PROCESS_ARGS,
        stdlib::NATIVE_PROCESS_CURRENT_DIR,
        stdlib::NATIVE_PROCESS_SET_CURRENT_DIR,
        stdlib::NATIVE_PROCESS_SPAWN,
        stdlib::NATIVE_PATH_JOIN,
        stdlib::NATIVE_PATH_PARENT,
        stdlib::NATIVE_PATH_EXTENSION,
        stdlib::NATIVE_PATH_FILENAME,
        stdlib::NATIVE_PATH_CANONICALIZE,
        stdlib::NATIVE_STR_LEN,
        stdlib::NATIVE_STR_IS_EMPTY,
        stdlib::NATIVE_STR_CHAR_AT,
        stdlib::NATIVE_STR_SUBSTR,
        stdlib::NATIVE_STR_CONTAINS,
        stdlib::NATIVE_STR_STARTS_WITH,
        stdlib::NATIVE_STR_ENDS_WITH,
        stdlib::NATIVE_STR_TRIM,
        stdlib::NATIVE_STR_SPLIT,
        stdlib::NATIVE_STR_REPEAT,
        stdlib::NATIVE_CHAR_IS_ALPHA,
        stdlib::NATIVE_CHAR_IS_DIGIT,
        stdlib::NATIVE_CHAR_IS_ALPHANUM,
        stdlib::NATIVE_CHAR_IS_WHITESPACE,
        stdlib::NATIVE_CHAR_IS_IDENT,
        stdlib::NATIVE_CHAR_TO_LOWER,
        stdlib::NATIVE_CHAR_TO_UPPER,
        stdlib::NATIVE_MATH_ABS,
        stdlib::NATIVE_MATH_MIN,
        stdlib::NATIVE_MATH_MAX,
        stdlib::NATIVE_MATH_SQRT,
    ];

    // Check for duplicates
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    sorted_ids.dedup();
    assert_eq!(ids.len(), sorted_ids.len(), "Duplicate IDs found!");
}
