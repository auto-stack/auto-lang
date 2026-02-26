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
///
/// Stack: path_str_idx -> content_str_idx (or -1 on error)
pub fn shim_file_read_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop path from stack
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Read file
    match fs::read_to_string(&path) {
        Ok(content) => {
            // Push content to stack
            content
                .push_to_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(e.to_string()))?;
        }
        Err(e) => {
            // Push error indicator (-1)
            task.ram.push_i32(-1);
            log::error!("File.read_text failed: {} - {}", path, e);
        }
    }

    Ok(())
}

/// Write text content to a file
///
/// Stack: path_str_idx, content_str_idx -> result (0 success, -1 error)
pub fn shim_file_write_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop content first (LIFO)
    let content: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Pop path
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Write file
    match fs::write(&path, &content) {
        Ok(()) => {
            task.ram.push_i32(0); // Success
        }
        Err(e) => {
            task.ram.push_i32(-1); // Error
            log::error!("File.write_text failed: {} - {}", path, e);
        }
    }

    Ok(())
}

/// Check if a file exists
///
/// Stack: path_str_idx -> bool (1 exists, 0 not exists)
pub fn shim_file_exists(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let exists = fs::metadata(&path).is_ok();
    task.ram.push_i32(if exists { 1 } else { 0 });

    Ok(())
}

/// Delete a file
///
/// Stack: path_str_idx -> result (0 success, -1 error)
pub fn shim_file_delete(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    match fs::remove_file(&path) {
        Ok(()) => {
            task.ram.push_i32(0);
        }
        Err(e) => {
            task.ram.push_i32(-1);
            log::error!("File.delete failed: {} - {}", path, e);
        }
    }

    Ok(())
}

/// Create a directory
///
/// Stack: path_str_idx -> result (0 success, -1 error)
pub fn shim_file_create_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    match fs::create_dir_all(&path) {
        Ok(()) => {
            task.ram.push_i32(0);
        }
        Err(e) => {
            task.ram.push_i32(-1);
            log::error!("File.create_dir failed: {} - {}", path, e);
        }
    }

    Ok(())
}

/// Read file contents as bytes
///
/// Stack: path_str -> bytes_list_id (Vec<i32>)
pub fn shim_file_read_bytes(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    match fs::read(&path) {
        Ok(bytes) => {
            // Convert bytes to Vec<i32>
            let byte_list: Vec<i32> = bytes.into_iter().map(|b| b as i32).collect();
            byte_list
                .push_to_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(e.to_string()))?;
        }
        Err(e) => {
            log::error!("File.read_bytes failed: {} - {}", path, e);
            // Return empty list on error
            let empty: Vec<i32> = Vec::new();
            empty
                .push_to_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(e.to_string()))?;
        }
    }

    Ok(())
}

/// Write bytes to a file
///
/// Stack: path_str, bytes_list_id -> result (0 success, -1 error)
pub fn shim_file_write_bytes(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let byte_list: Vec<i32> = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Convert Vec<i32> to bytes
    let bytes: Vec<u8> = byte_list.into_iter().map(|b| b as u8).collect();

    match fs::write(&path, &bytes) {
        Ok(()) => {
            task.ram.push_i32(0);
        }
        Err(e) => {
            task.ram.push_i32(-1);
            log::error!("File.write_bytes failed: {} - {}", path, e);
        }
    }

    Ok(())
}

/// Copy a file
///
/// Stack: src_str, dst_str -> result (0 success, -1 error)
pub fn shim_file_copy(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let dst: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let src: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    match fs::copy(&src, &dst) {
        Ok(_) => {
            task.ram.push_i32(0);
        }
        Err(e) => {
            task.ram.push_i32(-1);
            log::error!("File.copy failed: {} -> {} - {}", src, dst, e);
        }
    }

    Ok(())
}

/// Get file size in bytes
///
/// Stack: path_str -> size (i64) or -1 on error
pub fn shim_file_size(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    match fs::metadata(&path) {
        Ok(meta) => {
            let size = meta.len() as i64;
            task.ram.push_i64(size);
        }
        Err(e) => {
            log::error!("File.size failed: {} - {}", path, e);
            task.ram.push_i64(-1);
        }
    }

    Ok(())
}

/// Check if path is a directory
///
/// Stack: path_str -> bool (1 is dir, 0 not dir or error)
pub fn shim_file_is_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let is_dir = fs::metadata(&path)
        .map(|m| m.is_dir())
        .unwrap_or(false);

    task.ram.push_i32(if is_dir { 1 } else { 0 });

    Ok(())
}

// ============================================================================
// Environment Functions
// ============================================================================

/// Get an environment variable
///
/// Stack: key_str_idx -> value_str_idx (or empty string if not set)
pub fn shim_env_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let value = std::env::var(&key).unwrap_or_default();

    value
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Set an environment variable
///
/// Stack: key_str_idx, value_str_idx -> result (0 success, -1 error)
pub fn shim_env_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    std::env::set_var(&key, &value);
    task.ram.push_i32(0);

    Ok(())
}

/// Remove an environment variable
///
/// Stack: key_str_idx -> result (0 success, -1 error)
pub fn shim_env_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    std::env::remove_var(&key);
    task.ram.push_i32(0);

    Ok(())
}

// ============================================================================
// Time Functions
// ============================================================================

/// Get current time in milliseconds since Unix epoch
///
/// Stack: -> time_ms (i64 as two i32 slots)
pub fn shim_time_now_ms(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    task.ram.push_i64(now);

    Ok(())
}

/// Get current time in seconds since Unix epoch
///
/// Stack: -> time_sec (i64 as two i32 slots)
pub fn shim_time_now_sec(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    task.ram.push_i64(now);

    Ok(())
}

/// Sleep for specified milliseconds
///
/// Stack: ms (i32) -> ()
pub fn shim_time_sleep_ms(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let ms = task.ram.pop_i32();

    std::thread::sleep(std::time::Duration::from_millis(ms as u64));

    task.ram.push_i32(0); // Return unit

    Ok(())
}

// ============================================================================
// Process Functions
// ============================================================================

/// Exit the process with a code
///
/// Stack: code (i32) -> (never returns)
pub fn shim_process_exit(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let code = task.ram.pop_i32();
    std::process::exit(code);
}

/// Get command line arguments
///
/// Stack: -> list_id (Vec<String>)
pub fn shim_process_args(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let args: Vec<String> = std::env::args().collect();
    args.push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Get current working directory
///
/// Stack: -> dir_str
pub fn shim_process_current_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    dir.push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Set current working directory
///
/// Stack: path_str -> result (0 success, -1 error)
pub fn shim_process_set_current_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    match std::env::set_current_dir(&path) {
        Ok(()) => {
            task.ram.push_i32(0);
        }
        Err(e) => {
            task.ram.push_i32(-1);
            log::error!("Process.set_current_dir failed: {} - {}", path, e);
        }
    }

    Ok(())
}

/// Spawn an external process and wait for it to complete
///
/// Stack: args_list_id -> exit_code (i32)
/// The args list should contain: [cmd, arg1, arg2, ...]
pub fn shim_process_spawn(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let args: Vec<String> = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    if args.is_empty() {
        task.ram.push_i32(-1);
        return Ok(());
    }

    let cmd = &args[0];
    let cmd_args: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    match std::process::Command::new(cmd).args(&cmd_args).status() {
        Ok(status) => {
            let code = status.code().unwrap_or(-1);
            task.ram.push_i32(code);
        }
        Err(e) => {
            log::error!("Process.spawn failed: {} - {}", cmd, e);
            task.ram.push_i32(-1);
        }
    }

    Ok(())
}

// ============================================================================
// Path Functions (ID 1400-1499)
// ============================================================================

/// Join path components together
///
/// Stack: n, part_n, ..., part_0 -> result_str
pub fn shim_path_join(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop number of parts
    let n = task.ram.pop_i32() as usize;

    // Pop parts in reverse order
    let mut parts: Vec<String> = Vec::with_capacity(n);
    for _ in 0..n {
        let part: String = VMConvertible::pop_from_stack(task, vm)
            .map_err(|e| VMError::RuntimeError(e.to_string()))?;
        parts.push(part);
    }
    parts.reverse();

    // Join using PathBuf
    let mut result = std::path::PathBuf::new();
    for part in parts {
        result.push(part);
    }

    let joined = result.to_string_lossy().to_string();
    joined
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Get parent directory of a path
///
/// Stack: path_str -> parent_str (or empty string if no parent)
pub fn shim_path_parent(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let parent = Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    parent
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Get file extension of a path
///
/// Stack: path_str -> extension_str (or empty string if no extension)
pub fn shim_path_extension(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let extension = Path::new(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    extension
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Get filename from a path
///
/// Stack: path_str -> filename_str (or empty string if no filename)
pub fn shim_path_filename(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let filename = Path::new(&path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    filename
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Canonicalize a path (resolve symlinks, .., .)
///
/// Stack: path_str -> canonical_str (or empty string on error)
pub fn shim_path_canonicalize(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let canonical = std::fs::canonicalize(&path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    canonical
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

// ============================================================================
// String Functions (ID 1500-1599)
// ============================================================================

/// Get string length in bytes
///
/// Stack: str -> len (i32)
pub fn shim_str_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    task.ram.push_i32(s.len() as i32);

    Ok(())
}

/// Check if string is empty
///
/// Stack: str -> bool
pub fn shim_str_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    task.ram.push_i32(if s.is_empty() { 1 } else { 0 });

    Ok(())
}

/// Get character at byte index (returns unicode codepoint)
///
/// Stack: str, index -> codepoint (i32) or -1 on error
pub fn shim_str_char_at(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let index: i32 = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Get character at byte index
    if index < 0 || index as usize >= s.len() {
        task.ram.push_i32(-1);
        return Ok(());
    }

    // Find the character at the byte position
    match s[index as usize..].chars().next() {
        Some(c) => task.ram.push_i32(c as i32),
        None => task.ram.push_i32(-1),
    }

    Ok(())
}

/// Get substring (byte indices)
///
/// Stack: str, start, end -> substr_str (or empty on error)
pub fn shim_str_substr(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let end: i32 = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let start: i32 = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Validate indices
    if start < 0 || end < start || start as usize > s.len() || end as usize > s.len() {
        String::new().push_to_stack(task, vm)
            .map_err(|e| VMError::RuntimeError(e.to_string()))?;
        return Ok(());
    }

    let substr = s[start as usize..end as usize].to_string();
    substr
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Check if string contains substring
///
/// Stack: str, needle -> bool
pub fn shim_str_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let needle: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    task.ram.push_i32(if s.contains(&needle) { 1 } else { 0 });

    Ok(())
}

/// Check if string starts with prefix
///
/// Stack: str, prefix -> bool
pub fn shim_str_starts_with(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let prefix: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    task.ram.push_i32(if s.starts_with(&prefix) { 1 } else { 0 });

    Ok(())
}

/// Check if string ends with suffix
///
/// Stack: str, suffix -> bool
pub fn shim_str_ends_with(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let suffix: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    task.ram.push_i32(if s.ends_with(&suffix) { 1 } else { 0 });

    Ok(())
}

/// Trim whitespace from string
///
/// Stack: str -> trimmed_str
pub fn shim_str_trim(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let trimmed = s.trim().to_string();
    trimmed
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Split string by delimiter
///
/// Stack: str, delimiter -> list_id
pub fn shim_str_split(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let delimiter: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Split and collect into Vec<String>
    let parts: Vec<String> = s.split(&delimiter).map(|p| p.to_string()).collect();

    // Push as list
    parts
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

/// Repeat string n times
///
/// Stack: str, n -> repeated_str
pub fn shim_str_repeat(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n: i32 = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    if n < 0 {
        String::new().push_to_stack(task, vm)
            .map_err(|e| VMError::RuntimeError(e.to_string()))?;
        return Ok(());
    }

    let repeated = s.repeat(n as usize);
    repeated
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    Ok(())
}

// ============================================================================
// Character Functions (ID 1600-1699)
// ============================================================================

/// Check if character is alphabetic
///
/// Stack: codepoint (i32) -> bool
pub fn shim_char_is_alpha(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let codepoint = task.ram.pop_i32();

    let is_alpha = if let Some(c) = char::from_u32(codepoint as u32) {
        c.is_alphabetic()
    } else {
        false
    };

    task.ram.push_i32(if is_alpha { 1 } else { 0 });

    Ok(())
}

/// Check if character is a digit
///
/// Stack: codepoint (i32) -> bool
pub fn shim_char_is_digit(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let codepoint = task.ram.pop_i32();

    let is_digit = if let Some(c) = char::from_u32(codepoint as u32) {
        c.is_ascii_digit()
    } else {
        false
    };

    task.ram.push_i32(if is_digit { 1 } else { 0 });

    Ok(())
}

/// Check if character is alphanumeric
///
/// Stack: codepoint (i32) -> bool
pub fn shim_char_is_alphanum(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let codepoint = task.ram.pop_i32();

    let is_alphanum = if let Some(c) = char::from_u32(codepoint as u32) {
        c.is_alphanumeric()
    } else {
        false
    };

    task.ram.push_i32(if is_alphanum { 1 } else { 0 });

    Ok(())
}

/// Check if character is whitespace
///
/// Stack: codepoint (i32) -> bool
pub fn shim_char_is_whitespace(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let codepoint = task.ram.pop_i32();

    let is_whitespace = if let Some(c) = char::from_u32(codepoint as u32) {
        c.is_whitespace()
    } else {
        false
    };

    task.ram.push_i32(if is_whitespace { 1 } else { 0 });

    Ok(())
}

/// Check if character is a valid AutoLang identifier character
///
/// Stack: codepoint (i32) -> bool
pub fn shim_char_is_ident(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let codepoint = task.ram.pop_i32();

    let is_ident = if let Some(c) = char::from_u32(codepoint as u32) {
        // AutoLang identifiers: start with letter or underscore,
        // continue with letters, digits, or underscores
        c.is_alphanumeric() || c == '_'
    } else {
        false
    };

    task.ram.push_i32(if is_ident { 1 } else { 0 });

    Ok(())
}

/// Convert character to lowercase
///
/// Stack: codepoint (i32) -> lowercase_codepoint (i32)
pub fn shim_char_to_lower(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let codepoint = task.ram.pop_i32();

    let lower = if let Some(c) = char::from_u32(codepoint as u32) {
        c.to_lowercase().next().unwrap_or(c) as i32
    } else {
        codepoint
    };

    task.ram.push_i32(lower);

    Ok(())
}

/// Convert character to uppercase
///
/// Stack: codepoint (i32) -> uppercase_codepoint (i32)
pub fn shim_char_to_upper(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let codepoint = task.ram.pop_i32();

    let upper = if let Some(c) = char::from_u32(codepoint as u32) {
        c.to_uppercase().next().unwrap_or(c) as i32
    } else {
        codepoint
    };

    task.ram.push_i32(upper);

    Ok(())
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
    natives.register_static(NATIVE_FILE_READ_TEXT, shim_file_read_text);
    natives.register_static(NATIVE_FILE_WRITE_TEXT, shim_file_write_text);
    natives.register_static(NATIVE_FILE_EXISTS, shim_file_exists);
    natives.register_static(NATIVE_FILE_DELETE, shim_file_delete);
    natives.register_static(NATIVE_FILE_CREATE_DIR, shim_file_create_dir);
    natives.register_static(NATIVE_FILE_READ_BYTES, shim_file_read_bytes);
    natives.register_static(NATIVE_FILE_WRITE_BYTES, shim_file_write_bytes);
    natives.register_static(NATIVE_FILE_COPY, shim_file_copy);
    natives.register_static(NATIVE_FILE_SIZE, shim_file_size);
    natives.register_static(NATIVE_FILE_IS_DIR, shim_file_is_dir);

    // Env functions
    natives.register_static(NATIVE_ENV_GET, shim_env_get);
    natives.register_static(NATIVE_ENV_SET, shim_env_set);
    natives.register_static(NATIVE_ENV_REMOVE, shim_env_remove);

    // Time functions
    natives.register_static(NATIVE_TIME_NOW_MS, shim_time_now_ms);
    natives.register_static(NATIVE_TIME_NOW_SEC, shim_time_now_sec);
    natives.register_static(NATIVE_TIME_SLEEP_MS, shim_time_sleep_ms);

    // Process functions
    natives.register_static(NATIVE_PROCESS_EXIT, shim_process_exit);
    natives.register_static(NATIVE_PROCESS_ARGS, shim_process_args);
    natives.register_static(NATIVE_PROCESS_CURRENT_DIR, shim_process_current_dir);
    natives.register_static(NATIVE_PROCESS_SET_CURRENT_DIR, shim_process_set_current_dir);
    natives.register_static(NATIVE_PROCESS_SPAWN, shim_process_spawn);

    // Path functions
    natives.register_static(NATIVE_PATH_JOIN, shim_path_join);
    natives.register_static(NATIVE_PATH_PARENT, shim_path_parent);
    natives.register_static(NATIVE_PATH_EXTENSION, shim_path_extension);
    natives.register_static(NATIVE_PATH_FILENAME, shim_path_filename);
    natives.register_static(NATIVE_PATH_CANONICALIZE, shim_path_canonicalize);

    // String functions
    natives.register_static(NATIVE_STR_LEN, shim_str_len);
    natives.register_static(NATIVE_STR_IS_EMPTY, shim_str_is_empty);
    natives.register_static(NATIVE_STR_CHAR_AT, shim_str_char_at);
    natives.register_static(NATIVE_STR_SUBSTR, shim_str_substr);
    natives.register_static(NATIVE_STR_CONTAINS, shim_str_contains);
    natives.register_static(NATIVE_STR_STARTS_WITH, shim_str_starts_with);
    natives.register_static(NATIVE_STR_ENDS_WITH, shim_str_ends_with);
    natives.register_static(NATIVE_STR_TRIM, shim_str_trim);
    natives.register_static(NATIVE_STR_SPLIT, shim_str_split);
    natives.register_static(NATIVE_STR_REPEAT, shim_str_repeat);

    // Char functions
    natives.register_static(NATIVE_CHAR_IS_ALPHA, shim_char_is_alpha);
    natives.register_static(NATIVE_CHAR_IS_DIGIT, shim_char_is_digit);
    natives.register_static(NATIVE_CHAR_IS_ALPHANUM, shim_char_is_alphanum);
    natives.register_static(NATIVE_CHAR_IS_WHITESPACE, shim_char_is_whitespace);
    natives.register_static(NATIVE_CHAR_IS_IDENT, shim_char_is_ident);
    natives.register_static(NATIVE_CHAR_TO_LOWER, shim_char_to_lower);
    natives.register_static(NATIVE_CHAR_TO_UPPER, shim_char_to_upper);

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
