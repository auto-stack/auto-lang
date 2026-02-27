//! Plan 094: Built-in stdlib functions using FFI
//!
//! This module provides high-level FFI functions for common operations
//! like file I/O, environment variables, time, path manipulation, and string operations.
//!
//! These functions use the VMConvertible trait for automatic type conversion.

use super::convert::VMConvertible;
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Native Function IDs (1000-4999 for built-in stdlib)
// ============================================================================

// File functions: 1000-1099
pub const NATIVE_FILE_READ_TEXT: u16 = 1000;
pub const NATIVE_FILE_WRITE_TEXT: u16 = 1001;
pub const NATIVE_FILE_EXISTS: u16 = 1002;
pub const NATIVE_FILE_DELETE: u16 = 1003;
pub const NATIVE_FILE_CREATE_DIR: u16 = 1004;
pub const NATIVE_FILE_READ_BYTES: u16 = 1005;
pub const NATIVE_FILE_WRITE_BYTES: u16 = 1006;
pub const NATIVE_FILE_COPY: u16 = 1007;
pub const NATIVE_FILE_SIZE: u16 = 1008;
pub const NATIVE_FILE_IS_DIR: u16 = 1009;

// Env functions: 1100-1199
pub const NATIVE_ENV_GET: u16 = 1100;
pub const NATIVE_ENV_SET: u16 = 1101;
pub const NATIVE_ENV_REMOVE: u16 = 1102;

// Time functions: 1200-1299
pub const NATIVE_TIME_NOW_MS: u16 = 1200;
pub const NATIVE_TIME_NOW_SEC: u16 = 1201;
pub const NATIVE_TIME_SLEEP_MS: u16 = 1202;

// Process functions: 1300-1399
pub const NATIVE_PROCESS_EXIT: u16 = 1300;
pub const NATIVE_PROCESS_ARGS: u16 = 1301;
pub const NATIVE_PROCESS_CURRENT_DIR: u16 = 1302;
pub const NATIVE_PROCESS_SET_CURRENT_DIR: u16 = 1303;
pub const NATIVE_PROCESS_SPAWN: u16 = 1304;

// Math functions: 1700-1799
pub const NATIVE_MATH_ABS: u16 = 1700;
pub const NATIVE_MATH_MIN: u16 = 1701;
pub const NATIVE_MATH_MAX: u16 = 1702;
pub const NATIVE_MATH_SQRT: u16 = 1703;

// Path functions: 1400-1499
pub const NATIVE_PATH_JOIN: u16 = 1400;
pub const NATIVE_PATH_PARENT: u16 = 1401;
pub const NATIVE_PATH_EXTENSION: u16 = 1402;
pub const NATIVE_PATH_FILENAME: u16 = 1403;
pub const NATIVE_PATH_CANONICALIZE: u16 = 1404;

// String functions: 1500-1599
pub const NATIVE_STR_LEN: u16 = 1500;
pub const NATIVE_STR_IS_EMPTY: u16 = 1501;
pub const NATIVE_STR_CHAR_AT: u16 = 1502;
pub const NATIVE_STR_SUBSTR: u16 = 1503;
pub const NATIVE_STR_CONTAINS: u16 = 1504;
pub const NATIVE_STR_STARTS_WITH: u16 = 1505;
pub const NATIVE_STR_ENDS_WITH: u16 = 1506;
pub const NATIVE_STR_TRIM: u16 = 1507;
pub const NATIVE_STR_SPLIT: u16 = 1508;
pub const NATIVE_STR_REPEAT: u16 = 1509;

// Char functions: 1600-1699
pub const NATIVE_CHAR_IS_ALPHA: u16 = 1600;
pub const NATIVE_CHAR_IS_DIGIT: u16 = 1601;
pub const NATIVE_CHAR_IS_ALPHANUM: u16 = 1602;
pub const NATIVE_CHAR_IS_WHITESPACE: u16 = 1603;
pub const NATIVE_CHAR_IS_IDENT: u16 = 1604;
pub const NATIVE_CHAR_TO_LOWER: u16 = 1605;
pub const NATIVE_CHAR_TO_UPPER: u16 = 1606;

// ============================================================================
// File Functions
// ============================================================================

/// Read text content from a file
#[auto_macros::rust_fn("File.read_text")]
pub fn shim_file_read_text(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("File.read_text failed: {} - {}", path, e))
}

/// Write text content to a file
#[auto_macros::rust_fn("File.write_text")]
pub fn shim_file_write_text(path: String, content: String) -> Result<(), String> {
    fs::write(&path, &content).map_err(|e| format!("File.write_text failed: {} - {}", path, e))
}

/// Check if a file exists
#[auto_macros::rust_fn("File.exists")]
pub fn shim_file_exists(path: String) -> bool {
    fs::metadata(&path).is_ok()
}

/// Delete a file
#[auto_macros::rust_fn("File.delete")]
pub fn shim_file_delete(path: String) -> Result<(), String> {
    fs::remove_file(&path).map_err(|e| format!("File.delete failed: {} - {}", path, e))
}

/// Create a directory
#[auto_macros::rust_fn("File.create_dir")]
pub fn shim_file_create_dir(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| format!("File.create_dir failed: {} - {}", path, e))
}

/// Read file contents as bytes
#[auto_macros::rust_fn("File.read_bytes")]
pub fn shim_file_read_bytes(path: String) -> Result<Vec<i32>, String> {
    match fs::read(&path) {
        Ok(bytes) => Ok(bytes.into_iter().map(|b| b as i32).collect()),
        Err(e) => Err(format!("File.read_bytes failed: {} - {}", path, e)),
    }
}

/// Write bytes to a file
#[auto_macros::rust_fn("File.write_bytes")]
pub fn shim_file_write_bytes(path: String, byte_list: Vec<i32>) -> Result<(), String> {
    let bytes: Vec<u8> = byte_list.into_iter().map(|b| b as u8).collect();
    fs::write(&path, &bytes).map_err(|e| format!("File.write_bytes failed: {} - {}", path, e))
}

/// Copy a file
#[auto_macros::rust_fn("File.copy")]
pub fn shim_file_copy(src: String, dst: String) -> Result<(), String> {
    fs::copy(&src, &dst)
        .map(|_| ())
        .map_err(|e| format!("File.copy failed: {} -> {} - {}", src, dst, e))
}

/// Get file size in bytes
#[auto_macros::rust_fn("File.size")]
pub fn shim_file_size(path: String) -> Result<i64, String> {
    fs::metadata(&path)
        .map(|meta| meta.len() as i64)
        .map_err(|e| format!("File.size failed: {} - {}", path, e))
}

/// Check if path is a directory
#[auto_macros::rust_fn("File.is_dir")]
pub fn shim_file_is_dir(path: String) -> bool {
    fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false)
}

// ============================================================================
// Environment Functions
// ============================================================================

/// Get an environment variable
#[auto_macros::rust_fn("Env.get")]
pub fn shim_env_get(key: String) -> String {
    std::env::var(&key).unwrap_or_default()
}

/// Set an environment variable
#[auto_macros::rust_fn("Env.set")]
pub fn shim_env_set(key: String, value: String) {
    std::env::set_var(&key, &value);
}

/// Remove an environment variable
#[auto_macros::rust_fn("Env.remove")]
pub fn shim_env_remove(key: String) {
    std::env::remove_var(&key);
}

// ============================================================================
// Time Functions
// ============================================================================

/// Get current time in milliseconds since Unix epoch
#[auto_macros::rust_fn("Time.now_ms")]
pub fn shim_time_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Get current time in seconds since Unix epoch
#[auto_macros::rust_fn("Time.now_sec")]
pub fn shim_time_now_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Sleep for specified milliseconds
#[auto_macros::rust_fn("Time.sleep_ms")]
pub fn shim_time_sleep_ms(ms: i32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

// ============================================================================
// Process Functions
// ============================================================================

/// Exit the process with a code
#[auto_macros::rust_fn("Process.exit")]
pub fn shim_process_exit(code: i32) {
    std::process::exit(code);
}

/// Get command line arguments
#[auto_macros::rust_fn("Process.args")]
pub fn shim_process_args() -> Vec<String> {
    std::env::args().collect()
}

/// Get current working directory
#[auto_macros::rust_fn("Process.current_dir")]
pub fn shim_process_current_dir() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Set current working directory
#[auto_macros::rust_fn("Process.set_current_dir")]
pub fn shim_process_set_current_dir(path: String) -> Result<(), String> {
    std::env::set_current_dir(&path)
        .map_err(|e| format!("Process.set_current_dir failed: {} - {}", path, e))
}

/// Spawn an external process and wait for it to complete
#[auto_macros::rust_fn("Process.spawn")]
pub fn shim_process_spawn(args: Vec<String>) -> Result<i32, String> {
    if args.is_empty() {
        return Err("Process.spawn failed: empty arguments".to_string());
    }

    let cmd = &args[0];
    let cmd_args: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    match std::process::Command::new(cmd).args(&cmd_args).status() {
        Ok(status) => Ok(status.code().unwrap_or(-1)),
        Err(e) => Err(format!("Process.spawn failed: {} - {}", cmd, e)),
    }
}

// ============================================================================
// Path Functions (ID 1400-1499)
// ============================================================================

/// Join path components together
pub fn __shim_Path_join(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), crate::vm::engine::VMError> {
    // 1. Pop argument count
    let n = task.ram.pop_i32() as usize;

    // 2. Pop arguments in reverse order
    let mut parts: Vec<String> = Vec::with_capacity(n);
    for _ in 0..n {
        let part: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
            .map_err(|e| crate::vm::engine::VMError::RuntimeError(e.to_string()))?;
        parts.push(part);
    }
    parts.reverse();

    // 3. Join using PathBuf
    let mut result = std::path::PathBuf::new();
    for part in parts {
        result.push(part);
    }

    let joined = result.to_string_lossy().to_string();

    // 4. Push result
    crate::vm::ffi::convert::VMConvertible::push_to_stack(&joined, task, vm)
        .map_err(|e| crate::vm::engine::VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Get parent directory of a path
#[auto_macros::rust_fn("Path.parent")]
pub fn shim_path_parent(path: String) -> String {
    Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Get file extension of a path
#[auto_macros::rust_fn("Path.extension")]
pub fn shim_path_extension(path: String) -> String {
    Path::new(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Get filename from a path
#[auto_macros::rust_fn("Path.filename")]
pub fn shim_path_filename(path: String) -> String {
    Path::new(&path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Canonicalize a path (resolve symlinks, .., .)
#[auto_macros::rust_fn("Path.canonicalize")]
pub fn shim_path_canonicalize(path: String) -> String {
    std::fs::canonicalize(&path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

// ============================================================================
// String Functions (ID 1500-1599)
// ============================================================================

/// Get string length in bytes
#[auto_macros::rust_fn("Str.len")]
pub fn shim_str_len(s: String) -> i32 {
    s.len() as i32
}

/// Check if string is empty
#[auto_macros::rust_fn("Str.is_empty")]
pub fn shim_str_is_empty(s: String) -> bool {
    s.is_empty()
}

/// Get character at byte index (returns unicode codepoint)
#[auto_macros::rust_fn("Str.char_at")]
pub fn shim_str_char_at(s: String, index: i32) -> i32 {
    if index < 0 || index as usize >= s.len() {
        return -1;
    }
    match s[index as usize..].chars().next() {
        Some(c) => c as i32,
        None => -1,
    }
}

/// Get substring (byte indices)
#[auto_macros::rust_fn("Str.substr")]
pub fn shim_str_substr(s: String, start: i32, end: i32) -> String {
    if start < 0 || end < start || start as usize > s.len() || end as usize > s.len() {
        return String::new();
    }
    s[start as usize..end as usize].to_string()
}

/// Check if string contains substring
#[auto_macros::rust_fn("Str.contains")]
pub fn shim_str_contains(s: String, needle: String) -> bool {
    s.contains(&needle)
}

/// Check if string starts with prefix
#[auto_macros::rust_fn("Str.starts_with")]
pub fn shim_str_starts_with(s: String, prefix: String) -> bool {
    s.starts_with(&prefix)
}

/// Check if string ends with suffix
#[auto_macros::rust_fn("Str.ends_with")]
pub fn shim_str_ends_with(s: String, suffix: String) -> bool {
    s.ends_with(&suffix)
}

/// Trim whitespace from string
#[auto_macros::rust_fn("Str.trim")]
pub fn shim_str_trim(s: String) -> String {
    s.trim().to_string()
}

/// Split string by delimiter
#[auto_macros::rust_fn("Str.split")]
pub fn shim_str_split(s: String, delimiter: String) -> Vec<String> {
    s.split(&delimiter).map(|p| p.to_string()).collect()
}

/// Repeat string n times
#[auto_macros::rust_fn("Str.repeat")]
pub fn shim_str_repeat(s: String, n: i32) -> String {
    if n < 0 {
        String::new()
    } else {
        s.repeat(n as usize)
    }
}

// ============================================================================
// Character Functions (ID 1600-1699)
// ============================================================================

/// Check if character is alphabetic
#[auto_macros::rust_fn("Char.is_alpha")]
pub fn shim_char_is_alpha(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_alphabetic())
}

/// Check if character is a digit
#[auto_macros::rust_fn("Char.is_digit")]
pub fn shim_char_is_digit(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_ascii_digit())
}

/// Check if character is alphanumeric
#[auto_macros::rust_fn("Char.is_alphanum")]
pub fn shim_char_is_alphanum(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_alphanumeric())
}

/// Check if character is whitespace
#[auto_macros::rust_fn("Char.is_whitespace")]
pub fn shim_char_is_whitespace(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_whitespace())
}

/// Check if character is valid for an identifier start or continue
#[auto_macros::rust_fn("Char.is_ident")]
pub fn shim_char_is_ident(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_alphanumeric() || c == '_')
}

/// Convert character to lowercase
#[auto_macros::rust_fn("Char.to_lower")]
pub fn shim_char_to_lower(codepoint: i32) -> i32 {
    if let Some(c) = char::from_u32(codepoint as u32) {
        if let Some(lower) = c.to_lowercase().next() {
            return lower as i32;
        }
    }
    codepoint
}

/// Convert character to uppercase
#[auto_macros::rust_fn("Char.to_upper")]
pub fn shim_char_to_upper(codepoint: i32) -> i32 {
    if let Some(c) = char::from_u32(codepoint as u32) {
        if let Some(upper) = c.to_uppercase().next() {
            return upper as i32;
        }
    }
    codepoint
}

// ============================================================================
// Math Functions (ID 1700-1799)
// ============================================================================

/// Absolute value of a number
///
/// Stack: n (i64) -> abs(n) (i64)
pub fn shim_math_abs(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i64();
    task.ram.push_i64(n.abs());
    Ok(())
}

/// Minimum of two numbers
///
/// Stack: a (i64), b (i64) -> min(a, b) (i64)
pub fn shim_math_min(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let b = task.ram.pop_i64();
    let a = task.ram.pop_i64();
    task.ram.push_i64(a.min(b));
    Ok(())
}

/// Maximum of two numbers
///
/// Stack: a (i64), b (i64) -> max(a, b) (i64)
pub fn shim_math_max(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let b = task.ram.pop_i64();
    let a = task.ram.pop_i64();
    task.ram.push_i64(a.max(b));
    Ok(())
}

/// Square root of a number
///
/// Stack: n (f64) -> sqrt(n) (f64)
pub fn shim_math_sqrt(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_f64();
    task.ram.push_f64(n.sqrt());
    Ok(())
}

// ============================================================================
// Registration Function
// ============================================================================

/// Register all stdlib FFI functions with the NativeInterface
pub fn register_stdlib_ffi(natives: &mut crate::vm::native::NativeInterface) {
    // File functions
    natives.register_static(NATIVE_FILE_READ_TEXT, __shim_File_read_text);
    natives.register_static(NATIVE_FILE_WRITE_TEXT, __shim_File_write_text);
    natives.register_static(NATIVE_FILE_EXISTS, __shim_File_exists);
    natives.register_static(NATIVE_FILE_DELETE, __shim_File_delete);
    natives.register_static(NATIVE_FILE_CREATE_DIR, __shim_File_create_dir);
    natives.register_static(NATIVE_FILE_READ_BYTES, __shim_File_read_bytes);
    natives.register_static(NATIVE_FILE_WRITE_BYTES, __shim_File_write_bytes);
    natives.register_static(NATIVE_FILE_COPY, __shim_File_copy);
    natives.register_static(NATIVE_FILE_SIZE, __shim_File_size);
    natives.register_static(NATIVE_FILE_IS_DIR, __shim_File_is_dir);

    // Env functions
    natives.register_static(NATIVE_ENV_GET, __shim_Env_get);
    natives.register_static(NATIVE_ENV_SET, __shim_Env_set);
    natives.register_static(NATIVE_ENV_REMOVE, __shim_Env_remove);

    // Time functions
    natives.register_static(NATIVE_TIME_NOW_MS, __shim_Time_now_ms);
    natives.register_static(NATIVE_TIME_NOW_SEC, __shim_Time_now_sec);
    natives.register_static(NATIVE_TIME_SLEEP_MS, __shim_Time_sleep_ms);

    // Process functions
    natives.register_static(NATIVE_PROCESS_EXIT, __shim_Process_exit);
    natives.register_static(NATIVE_PROCESS_ARGS, __shim_Process_args);
    natives.register_static(NATIVE_PROCESS_CURRENT_DIR, __shim_Process_current_dir);
    natives.register_static(
        NATIVE_PROCESS_SET_CURRENT_DIR,
        __shim_Process_set_current_dir,
    );
    natives.register_static(NATIVE_PROCESS_SPAWN, __shim_Process_spawn);

    // Path functions
    natives.register_static(NATIVE_PATH_JOIN, __shim_Path_join);
    natives.register_static(NATIVE_PATH_PARENT, __shim_Path_parent);
    natives.register_static(NATIVE_PATH_EXTENSION, __shim_Path_extension);
    natives.register_static(NATIVE_PATH_FILENAME, __shim_Path_filename);
    natives.register_static(NATIVE_PATH_CANONICALIZE, __shim_Path_canonicalize);

    // String functions
    natives.register_static(NATIVE_STR_LEN, __shim_Str_len);
    natives.register_static(NATIVE_STR_IS_EMPTY, __shim_Str_is_empty);
    natives.register_static(NATIVE_STR_CHAR_AT, __shim_Str_char_at);
    natives.register_static(NATIVE_STR_SUBSTR, __shim_Str_substr);
    natives.register_static(NATIVE_STR_CONTAINS, __shim_Str_contains);
    natives.register_static(NATIVE_STR_STARTS_WITH, __shim_Str_starts_with);
    natives.register_static(NATIVE_STR_ENDS_WITH, __shim_Str_ends_with);
    natives.register_static(NATIVE_STR_TRIM, __shim_Str_trim);
    natives.register_static(NATIVE_STR_SPLIT, __shim_Str_split);
    natives.register_static(NATIVE_STR_REPEAT, __shim_Str_repeat);

    // Char functions
    natives.register_static(NATIVE_CHAR_IS_ALPHA, __shim_Char_is_alpha);
    natives.register_static(NATIVE_CHAR_IS_DIGIT, __shim_Char_is_digit);
    natives.register_static(NATIVE_CHAR_IS_ALPHANUM, __shim_Char_is_alphanum);
    natives.register_static(NATIVE_CHAR_IS_WHITESPACE, __shim_Char_is_whitespace);
    natives.register_static(NATIVE_CHAR_IS_IDENT, __shim_Char_is_ident);
    natives.register_static(NATIVE_CHAR_TO_LOWER, __shim_Char_to_lower);
    natives.register_static(NATIVE_CHAR_TO_UPPER, __shim_Char_to_upper);

    // Math functions
    natives.register_static(NATIVE_MATH_ABS, shim_math_abs);
    natives.register_static(NATIVE_MATH_MIN, shim_math_min);
    natives.register_static(NATIVE_MATH_MAX, shim_math_max);
    natives.register_static(NATIVE_MATH_SQRT, shim_math_sqrt);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_ids_are_in_range() {
        // Verify all IDs are in the static range (0-9999)
        assert!(NATIVE_FILE_READ_TEXT < 10000);
        assert!(NATIVE_FILE_WRITE_TEXT < 10000);
        assert!(NATIVE_FILE_EXISTS < 10000);
        assert!(NATIVE_FILE_READ_BYTES < 10000);
        assert!(NATIVE_FILE_WRITE_BYTES < 10000);
        assert!(NATIVE_FILE_COPY < 10000);
        assert!(NATIVE_FILE_SIZE < 10000);
        assert!(NATIVE_FILE_IS_DIR < 10000);
        assert!(NATIVE_ENV_GET < 10000);
        assert!(NATIVE_TIME_NOW_MS < 10000);
        assert!(NATIVE_PROCESS_EXIT < 10000);
        assert!(NATIVE_PROCESS_ARGS < 10000);
        assert!(NATIVE_PROCESS_CURRENT_DIR < 10000);
        assert!(NATIVE_PROCESS_SPAWN < 10000);
        assert!(NATIVE_PATH_JOIN < 10000);
        assert!(NATIVE_STR_LEN < 10000);
        assert!(NATIVE_CHAR_IS_ALPHA < 10000);
        assert!(NATIVE_MATH_ABS < 10000);
        assert!(NATIVE_MATH_MIN < 10000);
        assert!(NATIVE_MATH_MAX < 10000);
        assert!(NATIVE_MATH_SQRT < 10000);
    }

    #[test]
    fn test_id_ranges_are_correct() {
        // File: 1000-1099
        assert!((1000..1100).contains(&NATIVE_FILE_READ_TEXT));
        assert!((1000..1100).contains(&NATIVE_FILE_READ_BYTES));
        assert!((1000..1100).contains(&NATIVE_FILE_WRITE_BYTES));
        assert!((1000..1100).contains(&NATIVE_FILE_COPY));
        assert!((1000..1100).contains(&NATIVE_FILE_SIZE));
        assert!((1000..1100).contains(&NATIVE_FILE_IS_DIR));

        // Env: 1100-1199
        assert!((1100..1200).contains(&NATIVE_ENV_GET));

        // Time: 1200-1299
        assert!((1200..1300).contains(&NATIVE_TIME_NOW_MS));

        // Process: 1300-1399
        assert!((1300..1400).contains(&NATIVE_PROCESS_EXIT));
        assert!((1300..1400).contains(&NATIVE_PROCESS_ARGS));
        assert!((1300..1400).contains(&NATIVE_PROCESS_CURRENT_DIR));
        assert!((1300..1400).contains(&NATIVE_PROCESS_SPAWN));

        // Path: 1400-1499
        assert!((1400..1500).contains(&NATIVE_PATH_JOIN));

        // String: 1500-1599
        assert!((1500..1600).contains(&NATIVE_STR_LEN));

        // Char: 1600-1699
        assert!((1600..1700).contains(&NATIVE_CHAR_IS_ALPHA));

        // Math: 1700-1799
        assert!((1700..1800).contains(&NATIVE_MATH_ABS));
        assert!((1700..1800).contains(&NATIVE_MATH_MIN));
        assert!((1700..1800).contains(&NATIVE_MATH_MAX));
        assert!((1700..1800).contains(&NATIVE_MATH_SQRT));
    }
}
