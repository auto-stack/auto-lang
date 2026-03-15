//! Plan 094: Built-in stdlib functions using FFI
//!
//! This module provides high-level FFI functions for common operations
//! like file I/O, environment variables, time, path manipulation, and string operations.
//!
//! These functions use the VMConvertible trait for automatic type conversion.

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener as StdTcpListener, TcpStream as StdTcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

// Log functions: 1800-1899
pub const NATIVE_LOG_DEBUG: u16 = 1800;
pub const NATIVE_LOG_INFO: u16 = 1801;
pub const NATIVE_LOG_WARN: u16 = 1802;
pub const NATIVE_LOG_ERROR: u16 = 1803;

// JSON functions: 1900-1999
pub const NATIVE_JSON_ENCODE: u16 = 1900;
pub const NATIVE_JSON_DECODE: u16 = 1901;
pub const NATIVE_JSON_PARSE: u16 = 1902;
pub const NATIVE_JSON_PRETTIFY: u16 = 1903;
pub const NATIVE_JSON_MINIFY: u16 = 1904;
pub const NATIVE_JSON_IS_VALID: u16 = 1905;
pub const NATIVE_JSON_GET: u16 = 1906;
pub const NATIVE_JSON_GET_AT: u16 = 1907;
pub const NATIVE_JSON_LEN: u16 = 1908;
pub const NATIVE_JSON_TYPE: u16 = 1909;
pub const NATIVE_JSON_AS_STRING: u16 = 1910;
pub const NATIVE_JSON_AS_NUMBER: u16 = 1911;
pub const NATIVE_JSON_AS_INT: u16 = 1912;
pub const NATIVE_JSON_AS_BOOL: u16 = 1913;
pub const NATIVE_JSON_IS_NULL: u16 = 1914;
pub const NATIVE_JSON_KEYS: u16 = 1915;
pub const NATIVE_JSON_AS_ARRAY: u16 = 1916;
pub const NATIVE_JSON_HAS_KEY: u16 = 1917;

// URL functions: 2000-2099
pub const NATIVE_URL_ENCODE: u16 = 2000;
pub const NATIVE_URL_DECODE: u16 = 2001;
pub const NATIVE_URL_ENCODE_QUERY: u16 = 2002;
pub const NATIVE_URL_DECODE_QUERY: u16 = 2003;
pub const NATIVE_URL_ENCODE_PATH_SEGMENT: u16 = 2004;
pub const NATIVE_URL_DECODE_QUERY_COMPONENT: u16 = 2005;
pub const NATIVE_URL_PARSE: u16 = 2006;
pub const NATIVE_URL_SCHEME: u16 = 2007;
pub const NATIVE_URL_HOST: u16 = 2008;
pub const NATIVE_URL_PORT: u16 = 2009;
pub const NATIVE_URL_PATH: u16 = 2010;
pub const NATIVE_URL_QUERY: u16 = 2011;
pub const NATIVE_URL_FRAGMENT: u16 = 2012;
pub const NATIVE_URL_QUERY_PARAM: u16 = 2013;
pub const NATIVE_URL_QUERY_PARAMS: u16 = 2014;
pub const NATIVE_URL_JOIN_PATH: u16 = 2015;

// Net functions: 2100-2199
pub const NATIVE_NET_TCP_BIND: u16 = 2100;
pub const NATIVE_NET_TCP_LISTENER_ACCEPT: u16 = 2101;
pub const NATIVE_NET_TCP_LISTENER_LOCAL_ADDR: u16 = 2102;
pub const NATIVE_NET_TCP_LISTENER_CLOSE: u16 = 2103;
pub const NATIVE_NET_TCP_CONNECT: u16 = 2104;
pub const NATIVE_NET_TCP_STREAM_READ: u16 = 2105;
pub const NATIVE_NET_TCP_STREAM_WRITE: u16 = 2106;
pub const NATIVE_NET_TCP_STREAM_READ_ALL: u16 = 2107;
pub const NATIVE_NET_TCP_STREAM_READ_LINE: u16 = 2108;
pub const NATIVE_NET_TCP_STREAM_WRITE_STR: u16 = 2109;
pub const NATIVE_NET_TCP_STREAM_CLOSE: u16 = 2110;
pub const NATIVE_NET_TCP_STREAM_PEER_ADDR: u16 = 2111;
pub const NATIVE_NET_TCP_STREAM_SET_READ_TIMEOUT: u16 = 2112;
pub const NATIVE_NET_TCP_STREAM_SET_WRITE_TIMEOUT: u16 = 2113;

// HTTP functions: 2200-2299
pub const NATIVE_HTTP_SERVER: u16 = 2200;
pub const NATIVE_HTTP_SERVER_GET: u16 = 2201;
pub const NATIVE_HTTP_SERVER_POST: u16 = 2202;
pub const NATIVE_HTTP_SERVER_PUT: u16 = 2203;
pub const NATIVE_HTTP_SERVER_DELETE: u16 = 2204;
pub const NATIVE_HTTP_SERVER_STATIC: u16 = 2205;
pub const NATIVE_HTTP_SERVER_LISTEN: u16 = 2206;
pub const NATIVE_HTTP_RESPONSE: u16 = 2210;
pub const NATIVE_HTTP_RESPONSE_STATUS: u16 = 2211;
pub const NATIVE_HTTP_RESPONSE_HEADER: u16 = 2212;
pub const NATIVE_HTTP_RESPONSE_TEXT: u16 = 2213;
pub const NATIVE_HTTP_RESPONSE_HTML: u16 = 2214;
pub const NATIVE_HTTP_RESPONSE_BYTES: u16 = 2215;
pub const NATIVE_HTTP_OK: u16 = 2220;
pub const NATIVE_HTTP_CREATED: u16 = 2221;
pub const NATIVE_HTTP_BAD_REQUEST: u16 = 2222;
pub const NATIVE_HTTP_NOT_FOUND: u16 = 2223;
pub const NATIVE_HTTP_INTERNAL_ERROR: u16 = 2224;
pub const NATIVE_HTTP_GET: u16 = 2230;
pub const NATIVE_HTTP_POST: u16 = 2231;
pub const NATIVE_HTTP_PUT: u16 = 2232;
pub const NATIVE_HTTP_DELETE: u16 = 2233;

// Task/Msg functions (Plan 121): 2300-2399
pub const NATIVE_TASK_SPAWN: u16 = 2300;
pub const NATIVE_TASK_SEND: u16 = 2301;
pub const NATIVE_TASK_HANDLE_IS_NULL: u16 = 2302;
pub const NATIVE_TASK_HANDLE_TYPE: u16 = 2303;
pub const NATIVE_TASK_HANDLE_ID: u16 = 2304;
pub const NATIVE_TASK_SYSTEM_START: u16 = 2305;

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
    // SAFETY: Environment variable modification is safe in single-threaded context
    // In multi-threaded context, this could cause data races
    #[allow(deprecated)]
    unsafe {
        std::env::set_var(&key, &value);
    }
}

/// Remove an environment variable
#[auto_macros::rust_fn("Env.remove")]
pub fn shim_env_remove(key: String) {
    // SAFETY: Environment variable modification is safe in single-threaded context
    // In multi-threaded context, this could cause data races
    #[allow(deprecated)]
    unsafe {
        std::env::remove_var(&key);
    }
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
// Log Functions (ID 1800-1899)
// ============================================================================

/// Log a debug message to stdout
#[auto_macros::rust_fn("Log.debug")]
pub fn shim_log_debug(msg: String) {
    println!("[DEBUG] {}", msg);
}

/// Log an info message to stdout
#[auto_macros::rust_fn("Log.info")]
pub fn shim_log_info(msg: String) {
    println!("[INFO] {}", msg);
}

/// Log a warning message to stdout
#[auto_macros::rust_fn("Log.warn")]
pub fn shim_log_warn(msg: String) {
    println!("[WARN] {}", msg);
}

/// Log an error message to stderr
#[auto_macros::rust_fn("Log.error")]
pub fn shim_log_error(msg: String) {
    eprintln!("[ERROR] {}", msg);
}

// ============================================================================
// JSON Functions (ID 1900-1999)
// ============================================================================

/// Encode a value to JSON string (placeholder - currently just stringifies)
#[auto_macros::rust_fn("Json.encode")]
pub fn shim_json_encode(value: String) -> String {
    serde_json::to_string(&value).unwrap_or_default()
}

/// Parse a JSON string into a value (placeholder)
#[auto_macros::rust_fn("Json.parse")]
pub fn shim_json_parse(s: String) -> String {
    // For now, return the string as-is (placeholder)
    // Full implementation would return a JsonValue handle
    s
}

/// Prettify a JSON string
#[auto_macros::rust_fn("Json.prettify")]
pub fn shim_json_prettify(s: String) -> String {
    serde_json::from_str::<serde_json::Value>(&s)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_default()
}

/// Check if a string is valid JSON
#[auto_macros::rust_fn("Json.is_valid")]
pub fn shim_json_is_valid(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

// ============================================================================
// URL Functions (ID 2000-2099)
// ============================================================================

/// URL encode a string
#[auto_macros::rust_fn("Url.encode")]
pub fn shim_url_encode(s: String) -> String {
    urlencoding::encode(&s).to_string()
}

/// URL decode a string
#[auto_macros::rust_fn("Url.decode")]
pub fn shim_url_decode(s: String) -> String {
    urlencoding::decode(&s)
        .map(|c| c.to_string())
        .unwrap_or_default()
}

/// Encode query parameters
/// TODO: Handle Map type properly
#[auto_macros::rust_fn("Url.encode_query")]
pub fn shim_url_encode_query(_placeholder: String) -> String {
    // Placeholder - would need to handle Map type
    String::new()
}

/// Decode query string
/// TODO: Handle Map type properly
#[auto_macros::rust_fn("Url.decode_query")]
pub fn shim_url_decode_query(_query: String) -> Vec<String> {
    // Placeholder - would need to handle Map type
    Vec::new()
}

/// Join URL path segments
#[auto_macros::rust_fn("Url.join_path")]
pub fn shim_url_join_path(segments: Vec<String>) -> String {
    let path = segments.iter()
        .map(|s| s.trim_start_matches('/').trim_end_matches('/'))
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    format!("/{}", path)
}

// ============================================================================
// Net Functions (ID 2100-2199)
// ============================================================================

/// Global handle counter for net resources
static NET_HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Thread-local storage for TCP listeners
thread_local! {
    static TCP_LISTENERS: std::cell::RefCell<std::collections::HashMap<u64, StdTcpListener>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    static TCP_STREAMS: std::cell::RefCell<std::collections::HashMap<u64, StdTcpStream>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Bind to address and create TCP listener
/// Returns handle (positive) on success, 0 on failure
pub fn shim_net_tcp_bind(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let addr: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let listener = match StdTcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[NET] tcp_bind failed: {} - {}", addr, e);
            task.ram.push_i64(0); // Return 0 on failure
            return Ok(());
        }
    };

    // Set non-blocking mode for accept
    listener.set_nonblocking(false).ok();

    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    TCP_LISTENERS.with(|listeners| {
        listeners.borrow_mut().insert(handle, listener);
    });

    task.ram.push_i64(handle as i64);
    Ok(())
}

/// Accept a new connection from listener
/// Returns stream handle (positive) on success, 0 on failure
pub fn shim_net_tcp_listener_accept(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let listener_handle: i64 = task.ram.pop_i64();

    let stream = TCP_LISTENERS.with(|listeners| {
        let mut listeners = listeners.borrow_mut();
        let listener = listeners.get_mut(&(listener_handle as u64))?;
        listener.accept().ok().map(|(s, _)| s)
    });

    match stream {
        Some(stream) => {
            let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
            TCP_STREAMS.with(|streams| {
                streams.borrow_mut().insert(handle, stream);
            });
            task.ram.push_i64(handle as i64);
        }
        None => {
            task.ram.push_i64(0); // Return 0 on failure
        }
    }
    Ok(())
}

/// Get local address of listener
#[auto_macros::rust_fn("Net.tcp_listener_local_addr")]
pub fn shim_net_tcp_listener_local_addr(listener_handle: i64) -> String {
    TCP_LISTENERS.with(|listeners| {
        let listeners = listeners.borrow();
        listeners
            .get(&(listener_handle as u64))
            .map(|l| l.local_addr().map(|a| a.to_string()).unwrap_or_default())
            .unwrap_or_default()
    })
}

/// Close listener
pub fn shim_net_tcp_listener_close(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let listener_handle: i64 = task.ram.pop_i64();
    TCP_LISTENERS.with(|listeners| {
        listeners.borrow_mut().remove(&(listener_handle as u64));
    });
    Ok(())
}

/// Connect to remote TCP server
/// Returns stream handle (positive) on success, 0 on failure
pub fn shim_net_tcp_connect(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let addr: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let stream = match StdTcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[NET] tcp_connect failed: {} - {}", addr, e);
            task.ram.push_i64(0);
            return Ok(());
        }
    };

    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    TCP_STREAMS.with(|streams| {
        streams.borrow_mut().insert(handle, stream);
    });

    task.ram.push_i64(handle as i64);
    Ok(())
}

/// Read data from stream
/// Returns number of bytes read, or -1 on error
pub fn shim_net_tcp_stream_read(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let buf_size: i32 = task.ram.pop_i32();
    let stream_handle: i64 = task.ram.pop_i64();

    let result = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        let stream = match streams.get_mut(&(stream_handle as u64)) {
            Some(s) => s,
            None => return (-1, Vec::new()),
        };

        let mut buf = vec![0u8; buf_size as usize];
        match stream.read(&mut buf) {
            Ok(n) => {
                buf.truncate(n);
                (n as i32, buf)
            }
            Err(_) => (-1, Vec::new()),
        }
    });

    task.ram.push_i32(result.0);
    // Push bytes as Vec<i32>
    let bytes: Vec<i32> = result.1.into_iter().map(|b| b as i32).collect();
    super::convert::VMConvertible::push_to_stack(&bytes, task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Write data to stream
/// Returns number of bytes written, or -1 on error
pub fn shim_net_tcp_stream_write(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let data: Vec<i32> = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_handle: i64 = task.ram.pop_i64();

    let bytes: Vec<u8> = data.into_iter().map(|b| b as u8).collect();

    let result = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => stream.write(&bytes).map(|n| n as i32).unwrap_or(-1),
            None => -1,
        }
    });

    task.ram.push_i32(result);
    Ok(())
}

/// Read all data until EOF
pub fn shim_net_tcp_stream_read_all(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let stream_handle: i64 = task.ram.pop_i64();

    let bytes = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => {
                let mut buf = Vec::new();
                stream.read_to_end(&mut buf).ok();
                buf
            }
            None => Vec::new(),
        }
    });

    let result: Vec<i32> = bytes.into_iter().map(|b| b as i32).collect();
    super::convert::VMConvertible::push_to_stack(&result, task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Read a line from stream
#[auto_macros::rust_fn("Net.tcp_stream_read_line")]
pub fn shim_net_tcp_stream_read_line(stream_handle: i64) -> String {
    TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).ok();
                line.trim_end_matches('\n').trim_end_matches('\r').to_string()
            }
            None => String::new(),
        }
    })
}

/// Write string to stream
/// Returns number of bytes written, or -1 on error
pub fn shim_net_tcp_stream_write_str(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let s: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_handle: i64 = task.ram.pop_i64();

    let result = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => stream.write_all(s.as_bytes()).map(|_| s.len() as i32).unwrap_or(-1),
            None => -1,
        }
    });

    task.ram.push_i32(result);
    Ok(())
}

/// Close stream
pub fn shim_net_tcp_stream_close(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let stream_handle: i64 = task.ram.pop_i64();
    TCP_STREAMS.with(|streams| {
        streams.borrow_mut().remove(&(stream_handle as u64));
    });
    Ok(())
}

/// Get peer address
#[auto_macros::rust_fn("Net.tcp_stream_peer_addr")]
pub fn shim_net_tcp_stream_peer_addr(stream_handle: i64) -> String {
    TCP_STREAMS.with(|streams| {
        let streams = streams.borrow();
        streams
            .get(&(stream_handle as u64))
            .map(|s| s.peer_addr().map(|a| a.to_string()).unwrap_or_default())
            .unwrap_or_default()
    })
}

/// Set read timeout
pub fn shim_net_tcp_stream_set_read_timeout(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let ms: i32 = task.ram.pop_i32();
    let stream_handle: i64 = task.ram.pop_i64();

    TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        if let Some(stream) = streams.get_mut(&(stream_handle as u64)) {
            let timeout = if ms > 0 {
                Some(Duration::from_millis(ms as u64))
            } else {
                None
            };
            stream.set_read_timeout(timeout).ok();
        }
    });
    Ok(())
}

/// Set write timeout
pub fn shim_net_tcp_stream_set_write_timeout(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let ms: i32 = task.ram.pop_i32();
    let stream_handle: i64 = task.ram.pop_i64();

    TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        if let Some(stream) = streams.get_mut(&(stream_handle as u64)) {
            let timeout = if ms > 0 {
                Some(Duration::from_millis(ms as u64))
            } else {
                None
            };
            stream.set_write_timeout(timeout).ok();
        }
    });
    Ok(())
}

// ============================================================================
// HTTP Functions (ID 2200-2299)
// ============================================================================

/// HTTP Response data stored in thread-local
thread_local! {
    static HTTP_RESPONSES: std::cell::RefCell<std::collections::HashMap<u64, HttpResponseData>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// HTTP Response data
#[derive(Debug, Clone, Default)]
struct HttpResponseData {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

/// Create a new HTTP server (placeholder - returns handle)
pub fn shim_http_server(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // For now, return a placeholder handle
    // Full implementation would store route handlers
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    task.ram.push_i64(handle as i64);
    Ok(())
}

/// Add GET route (placeholder)
pub fn shim_http_server_get(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // Pop handler and path (we'll ignore them for now)
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();

    // Return server handle unchanged
    task.ram.push_i64(server);
    Ok(())
}

/// Add POST route (placeholder)
pub fn shim_http_server_post(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Add PUT route (placeholder)
pub fn shim_http_server_put(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Add DELETE route (placeholder)
pub fn shim_http_server_delete(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Add static file route (placeholder)
pub fn shim_http_server_static(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _dir: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let _prefix: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Start server listening (placeholder)
pub fn shim_http_server_listen(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _addr: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let _server: i64 = task.ram.pop_i64();

    // Placeholder - would start TCP server with HTTP parsing
    eprintln!("[HTTP] Server listen not yet implemented");
    Ok(())
}

/// Create a new HTTP response
pub fn shim_http_response(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let response = HttpResponseData::default();

    HTTP_RESPONSES.with(|responses| {
        responses.borrow_mut().insert(handle, response);
    });

    task.ram.push_i64(handle as i64);
    Ok(())
}

/// Set response status
pub fn shim_http_response_status(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let code: i32 = task.ram.pop_i32();
    let res_handle: i64 = task.ram.pop_i64();

    HTTP_RESPONSES.with(|responses| {
        if let Some(res) = responses.borrow_mut().get_mut(&(res_handle as u64)) {
            res.status = code as u16;
        }
    });

    task.ram.push_i64(res_handle);
    Ok(())
}

/// Set response header
pub fn shim_http_response_header(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let value: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_handle: i64 = task.ram.pop_i64();

    HTTP_RESPONSES.with(|responses| {
        if let Some(res) = responses.borrow_mut().get_mut(&(res_handle as u64)) {
            res.headers.push((key, value));
        }
    });

    task.ram.push_i64(res_handle);
    Ok(())
}

/// Set response text body
pub fn shim_http_response_text(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_handle: i64 = task.ram.pop_i64();

    HTTP_RESPONSES.with(|responses| {
        if let Some(res) = responses.borrow_mut().get_mut(&(res_handle as u64)) {
            res.body = body.into_bytes();
            // Add Content-Type header if not set
            if !res.headers.iter().any(|(k, _)| k.to_lowercase() == "content-type") {
                res.headers.push(("Content-Type".to_string(), "text/plain; charset=utf-8".to_string()));
            }
        }
    });

    task.ram.push_i64(res_handle);
    Ok(())
}

/// Set response HTML body
pub fn shim_http_response_html(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_handle: i64 = task.ram.pop_i64();

    HTTP_RESPONSES.with(|responses| {
        if let Some(res) = responses.borrow_mut().get_mut(&(res_handle as u64)) {
            res.body = body.into_bytes();
            if !res.headers.iter().any(|(k, _)| k.to_lowercase() == "content-type") {
                res.headers.push(("Content-Type".to_string(), "text/html; charset=utf-8".to_string()));
            }
        }
    });

    task.ram.push_i64(res_handle);
    Ok(())
}

/// Set response bytes body
pub fn shim_http_response_bytes(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let data: Vec<i32> = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_handle: i64 = task.ram.pop_i64();

    HTTP_RESPONSES.with(|responses| {
        if let Some(res) = responses.borrow_mut().get_mut(&(res_handle as u64)) {
            res.body = data.into_iter().map(|b| b as u8).collect();
        }
    });

    task.ram.push_i64(res_handle);
    Ok(())
}

/// Create a 200 OK response
#[auto_macros::rust_fn("Http.ok")]
pub fn shim_http_ok(body: String) -> i64 {
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let response = HttpResponseData {
        status: 200,
        headers: vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
        body: body.into_bytes(),
    };

    HTTP_RESPONSES.with(|responses| {
        responses.borrow_mut().insert(handle, response);
    });

    handle as i64
}

/// Create a 201 Created response
#[auto_macros::rust_fn("Http.created")]
pub fn shim_http_created(body: String) -> i64 {
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let response = HttpResponseData {
        status: 201,
        headers: vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
        body: body.into_bytes(),
    };

    HTTP_RESPONSES.with(|responses| {
        responses.borrow_mut().insert(handle, response);
    });

    handle as i64
}

/// Create a 400 Bad Request response
#[auto_macros::rust_fn("Http.bad_request")]
pub fn shim_http_bad_request(msg: String) -> i64 {
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let response = HttpResponseData {
        status: 400,
        headers: vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
        body: msg.into_bytes(),
    };

    HTTP_RESPONSES.with(|responses| {
        responses.borrow_mut().insert(handle, response);
    });

    handle as i64
}

/// Create a 404 Not Found response
#[auto_macros::rust_fn("Http.not_found")]
pub fn shim_http_not_found(msg: String) -> i64 {
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let response = HttpResponseData {
        status: 404,
        headers: vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
        body: msg.into_bytes(),
    };

    HTTP_RESPONSES.with(|responses| {
        responses.borrow_mut().insert(handle, response);
    });

    handle as i64
}

/// Create a 500 Internal Server Error response
#[auto_macros::rust_fn("Http.internal_error")]
pub fn shim_http_internal_error(msg: String) -> i64 {
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let response = HttpResponseData {
        status: 500,
        headers: vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
        body: msg.into_bytes(),
    };

    HTTP_RESPONSES.with(|responses| {
        responses.borrow_mut().insert(handle, response);
    });

    handle as i64
}

/// Perform a GET request (simple HTTP client)
pub fn shim_http_get(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Simple HTTP GET implementation
    let response_handle = simple_http_request("GET", &url, None);

    task.ram.push_i64(response_handle);
    Ok(())
}

/// Perform a POST request
pub fn shim_http_post(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let response_handle = simple_http_request("POST", &url, Some(&body));

    task.ram.push_i64(response_handle);
    Ok(())
}

/// Perform a PUT request
pub fn shim_http_put(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let response_handle = simple_http_request("PUT", &url, Some(&body));

    task.ram.push_i64(response_handle);
    Ok(())
}

/// Perform a DELETE request
pub fn shim_http_delete(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let response_handle = simple_http_request("DELETE", &url, None);

    task.ram.push_i64(response_handle);
    Ok(())
}

/// Simple HTTP request implementation
fn simple_http_request(method: &str, url: &str, body: Option<&str>) -> i64 {
    // Parse URL (simple: expect http://host:port/path)
    let url = url.trim_start_matches("http://");

    let (host_port, path) = match url.find('/') {
        Some(i) => (&url[..i], &url[i..]),
        None => (url, "/"),
    };

    let (host, port) = match host_port.find(':') {
        Some(i) => (&host_port[..i], &host_port[i + 1..]),
        None => (host_port, "80"),
    };

    let addr = format!("{}:{}", host, port);

    // Connect
    let mut stream = match StdTcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[HTTP] Connection failed: {} - {}", addr, e);
            return shim_http_internal_error(format!("Connection failed: {}", e));
        }
    };

    // Set timeout
    stream.set_read_timeout(Some(Duration::from_secs(30))).ok();

    // Build request
    let body_len = body.map(|b| b.len()).unwrap_or(0);
    let mut request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
        method, path, host
    );

    if body.is_some() {
        request.push_str(&format!("Content-Length: {}\r\n", body_len));
        request.push_str("Content-Type: application/json\r\n");
    }

    request.push_str("\r\n");

    if let Some(b) = body {
        request.push_str(b);
    }

    // Send request
    if let Err(e) = stream.write_all(request.as_bytes()) {
        return shim_http_internal_error(format!("Write failed: {}", e));
    }

    // Read response
    let mut response_bytes = Vec::new();
    if let Err(e) = stream.read_to_end(&mut response_bytes) {
        return shim_http_internal_error(format!("Read failed: {}", e));
    }

    // Parse response (simple: just extract status code and body)
    let response_str = String::from_utf8_lossy(&response_bytes);
    let status = extract_status_code(&response_str);
    let body = extract_body(&response_str);

    // Create response handle
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let response = HttpResponseData {
        status,
        headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
        body: body.into_bytes(),
    };

    HTTP_RESPONSES.with(|responses| {
        responses.borrow_mut().insert(handle, response);
    });

    handle as i64
}

/// Extract status code from HTTP response
fn extract_status_code(response: &str) -> u16 {
    // HTTP/1.1 200 OK
    let first_line = response.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse().unwrap_or(500)
    } else {
        500
    }
}

/// Extract body from HTTP response
fn extract_body(response: &str) -> String {
    // Find \r\n\r\n separator
    match response.find("\r\n\r\n") {
        Some(i) => response[i + 4..].to_string(),
        None => String::new(),
    }
}

// ============================================================================
// Task/Msg Functions (Plan 121)
// ============================================================================

use crate::vm::task_system::{TaskHandle, TaskInstance, TaskRegistry};

/// TaskHandle wrapper for passing through VM
/// The handle is stored as a tuple: (task_type: String, instance_id: u64, tx_ptr: u64)
/// We use a thread-local storage to keep actual handles alive
thread_local! {
    static TASK_HANDLES: std::cell::RefCell<std::collections::HashMap<u64, TaskHandle>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    // Keep TaskInstance alive so the receiver (rx) doesn't get dropped
    static TASK_INSTANCES: std::cell::RefCell<std::collections::HashMap<u64, TaskInstance>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

static TASK_HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Spawn a new task instance
///
/// Creates a new TaskInstance with its own mailbox and returns a handle.
/// The handle can be used to send messages to the task.
///
/// # Arguments
/// * `task_type` - The name of the task type (e.g., "CounterTask")
/// * `capacity` - The mailbox capacity (default: 64)
///
/// # Returns
/// A handle ID (u64) that can be used to reference the task
#[auto_macros::rust_fn("Task.spawn")]
pub fn shim_task_spawn(task_type: String, capacity: i64) -> Result<u64, String> {
    let cap = if capacity <= 0 { 64 } else { capacity as usize };

    // Create a new task instance
    let instance = TaskInstance::new(task_type.clone(), cap);
    let handle = instance.handle.clone();

    // Generate a unique handle ID
    let handle_id = TASK_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Store the handle in thread-local storage
    TASK_HANDLES.with(|handles| {
        handles.borrow_mut().insert(handle_id, handle);
    });

    // Store the instance to keep the receiver alive
    // In a full implementation, we would spawn a tokio task to process messages.
    TASK_INSTANCES.with(|instances| {
        instances.borrow_mut().insert(handle_id, instance);
    });

    Ok(handle_id)
}

/// Send a message to a task
///
/// # Arguments
/// * `handle_id` - The handle ID returned by Task.spawn
/// * `msg` - The message value to send
///
/// # Returns
/// 1 on success, 0 on failure
#[auto_macros::rust_fn("TaskHandle.send")]
pub fn shim_task_send(handle_id: i64, msg: String) -> Result<i32, String> {
    let id = handle_id as u64;

    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            let auto_str: auto_val::AutoStr = msg.into();
            match handle.try_send(auto_val::Value::Str(auto_str)) {
                Ok(()) => Ok(1),
                Err(e) => Err(format!("TaskHandle.send failed: {}", e)),
            }
        } else {
            Err(format!("TaskHandle.send failed: invalid handle ID {}", id))
        }
    })
}

/// Check if a task handle is null/empty
///
/// # Arguments
/// * `handle_id` - The handle ID to check
///
/// # Returns
/// 1 if null, 0 otherwise
#[auto_macros::rust_fn("TaskHandle.is_null")]
pub fn shim_task_handle_is_null(handle_id: i64) -> Result<i32, String> {
    let id = handle_id as u64;

    if id == 0 {
        return Ok(1);
    }

    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            Ok(if handle.is_null() { 1 } else { 0 })
        } else {
            // Invalid handle ID is treated as null
            Ok(1)
        }
    })
}

/// Get the task type from a handle
///
/// # Arguments
/// * `handle_id` - The handle ID
///
/// # Returns
/// The task type name
#[auto_macros::rust_fn("TaskHandle.task_type")]
pub fn shim_task_handle_type(handle_id: i64) -> Result<String, String> {
    let id = handle_id as u64;

    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            Ok(handle.task_type.clone())
        } else {
            Err(format!("TaskHandle.task_type failed: invalid handle ID {}", id))
        }
    })
}

/// Get the instance ID from a handle
///
/// # Arguments
/// * `handle_id` - The handle ID
///
/// # Returns
/// The instance ID
#[auto_macros::rust_fn("TaskHandle.instance_id")]
pub fn shim_task_handle_id(handle_id: i64) -> Result<u64, String> {
    let id = handle_id as u64;

    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            Ok(handle.instance_id)
        } else {
            Err(format!("TaskHandle.instance_id failed: invalid handle ID {}", id))
        }
    })
}

/// Global TaskRegistry for TaskSystem operations
static GLOBAL_TASK_REGISTRY: std::sync::OnceLock<TaskRegistry> = std::sync::OnceLock::new();

/// Get or initialize the global TaskRegistry
fn get_global_task_registry() -> &'static TaskRegistry {
    GLOBAL_TASK_REGISTRY.get_or_init(TaskRegistry::new)
}

/// Start the task system scheduler
///
/// This method blocks the main thread and waits for Ctrl+C signal.
/// When Ctrl+C is received, all registered stop hooks are executed in LIFO order.
///
/// # Note
/// This is a blocking call that will not return until Ctrl+C is received.
#[auto_macros::rust_fn("TaskSystem.start")]
pub fn shim_task_system_start() -> Result<(), String> {
    let registry = get_global_task_registry();
    registry.start_scheduler();
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

    // Log functions
    natives.register_static(NATIVE_LOG_DEBUG, __shim_Log_debug);
    natives.register_static(NATIVE_LOG_INFO, __shim_Log_info);
    natives.register_static(NATIVE_LOG_WARN, __shim_Log_warn);
    natives.register_static(NATIVE_LOG_ERROR, __shim_Log_error);

    // JSON functions
    natives.register_static(NATIVE_JSON_ENCODE, __shim_Json_encode);
    natives.register_static(NATIVE_JSON_PARSE, __shim_Json_parse);
    natives.register_static(NATIVE_JSON_PRETTIFY, __shim_Json_prettify);
    natives.register_static(NATIVE_JSON_IS_VALID, __shim_Json_is_valid);

    // URL functions
    natives.register_static(NATIVE_URL_ENCODE, __shim_Url_encode);
    natives.register_static(NATIVE_URL_DECODE, __shim_Url_decode);
    natives.register_static(NATIVE_URL_ENCODE_QUERY, __shim_Url_encode_query);
    natives.register_static(NATIVE_URL_DECODE_QUERY, __shim_Url_decode_query);
    natives.register_static(NATIVE_URL_JOIN_PATH, __shim_Url_join_path);

    // Net functions
    natives.register_static(NATIVE_NET_TCP_BIND, shim_net_tcp_bind);
    natives.register_static(NATIVE_NET_TCP_LISTENER_ACCEPT, shim_net_tcp_listener_accept);
    natives.register_static(NATIVE_NET_TCP_LISTENER_LOCAL_ADDR, __shim_Net_tcp_listener_local_addr);
    natives.register_static(NATIVE_NET_TCP_LISTENER_CLOSE, shim_net_tcp_listener_close);
    natives.register_static(NATIVE_NET_TCP_CONNECT, shim_net_tcp_connect);
    natives.register_static(NATIVE_NET_TCP_STREAM_READ, shim_net_tcp_stream_read);
    natives.register_static(NATIVE_NET_TCP_STREAM_WRITE, shim_net_tcp_stream_write);
    natives.register_static(NATIVE_NET_TCP_STREAM_READ_ALL, shim_net_tcp_stream_read_all);
    natives.register_static(NATIVE_NET_TCP_STREAM_READ_LINE, __shim_Net_tcp_stream_read_line);
    natives.register_static(NATIVE_NET_TCP_STREAM_WRITE_STR, shim_net_tcp_stream_write_str);
    natives.register_static(NATIVE_NET_TCP_STREAM_CLOSE, shim_net_tcp_stream_close);
    natives.register_static(NATIVE_NET_TCP_STREAM_PEER_ADDR, __shim_Net_tcp_stream_peer_addr);
    natives.register_static(NATIVE_NET_TCP_STREAM_SET_READ_TIMEOUT, shim_net_tcp_stream_set_read_timeout);
    natives.register_static(NATIVE_NET_TCP_STREAM_SET_WRITE_TIMEOUT, shim_net_tcp_stream_set_write_timeout);

    // HTTP functions
    natives.register_static(NATIVE_HTTP_SERVER, shim_http_server);
    natives.register_static(NATIVE_HTTP_SERVER_GET, shim_http_server_get);
    natives.register_static(NATIVE_HTTP_SERVER_POST, shim_http_server_post);
    natives.register_static(NATIVE_HTTP_SERVER_PUT, shim_http_server_put);
    natives.register_static(NATIVE_HTTP_SERVER_DELETE, shim_http_server_delete);
    natives.register_static(NATIVE_HTTP_SERVER_STATIC, shim_http_server_static);
    natives.register_static(NATIVE_HTTP_SERVER_LISTEN, shim_http_server_listen);
    natives.register_static(NATIVE_HTTP_RESPONSE, shim_http_response);
    natives.register_static(NATIVE_HTTP_RESPONSE_STATUS, shim_http_response_status);
    natives.register_static(NATIVE_HTTP_RESPONSE_HEADER, shim_http_response_header);
    natives.register_static(NATIVE_HTTP_RESPONSE_TEXT, shim_http_response_text);
    natives.register_static(NATIVE_HTTP_RESPONSE_HTML, shim_http_response_html);
    natives.register_static(NATIVE_HTTP_RESPONSE_BYTES, shim_http_response_bytes);
    natives.register_static(NATIVE_HTTP_OK, __shim_Http_ok);
    natives.register_static(NATIVE_HTTP_CREATED, __shim_Http_created);
    natives.register_static(NATIVE_HTTP_BAD_REQUEST, __shim_Http_bad_request);
    natives.register_static(NATIVE_HTTP_NOT_FOUND, __shim_Http_not_found);
    natives.register_static(NATIVE_HTTP_INTERNAL_ERROR, __shim_Http_internal_error);
    natives.register_static(NATIVE_HTTP_GET, shim_http_get);
    natives.register_static(NATIVE_HTTP_POST, shim_http_post);
    natives.register_static(NATIVE_HTTP_PUT, shim_http_put);
    natives.register_static(NATIVE_HTTP_DELETE, shim_http_delete);

    // Task/Msg functions (Plan 121)
    natives.register_static(NATIVE_TASK_SPAWN, __shim_Task_spawn);
    natives.register_static(NATIVE_TASK_SEND, __shim_TaskHandle_send);
    natives.register_static(NATIVE_TASK_HANDLE_IS_NULL, __shim_TaskHandle_is_null);
    natives.register_static(NATIVE_TASK_HANDLE_TYPE, __shim_TaskHandle_task_type);
    natives.register_static(NATIVE_TASK_HANDLE_ID, __shim_TaskHandle_instance_id);
    natives.register_static(NATIVE_TASK_SYSTEM_START, __shim_TaskSystem_start);
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

        // Task/Msg: 2300-2399
        assert!((2300..2400).contains(&NATIVE_TASK_SPAWN));
        assert!((2300..2400).contains(&NATIVE_TASK_SEND));
        assert!((2300..2400).contains(&NATIVE_TASK_HANDLE_IS_NULL));
        assert!((2300..2400).contains(&NATIVE_TASK_HANDLE_TYPE));
        assert!((2300..2400).contains(&NATIVE_TASK_HANDLE_ID));
        assert!((2300..2400).contains(&NATIVE_TASK_SYSTEM_START));
    }

    // Plan 121: Task spawn tests
    #[test]
    fn test_task_spawn() {
        let result = shim_task_spawn("TestTask".to_string(), 64);
        assert!(result.is_ok());
        let handle_id = result.unwrap();
        assert!(handle_id > 0);
    }

    #[test]
    fn test_task_spawn_default_capacity() {
        // Test with negative capacity (should use default 64)
        let result = shim_task_spawn("TestTask".to_string(), -1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_task_handle_is_null() {
        // Handle ID 0 should be null
        let result = shim_task_handle_is_null(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Non-existent handle should also be treated as null
        let result = shim_task_handle_is_null(999999);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // A spawned task should not be null
        let spawn_result = shim_task_spawn("TestTask".to_string(), 64);
        assert!(spawn_result.is_ok());
        let handle_id = spawn_result.unwrap() as i64;

        let result = shim_task_handle_is_null(handle_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_task_handle_type() {
        let spawn_result = shim_task_spawn("MyCounterTask".to_string(), 64);
        assert!(spawn_result.is_ok());
        let handle_id = spawn_result.unwrap() as i64;

        let result = shim_task_handle_type(handle_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "MyCounterTask");
    }

    #[test]
    fn test_task_handle_id() {
        let spawn_result = shim_task_spawn("TestTask".to_string(), 64);
        assert!(spawn_result.is_ok());
        let handle_id = spawn_result.unwrap();

        let result = shim_task_handle_id(handle_id as i64);
        assert!(result.is_ok());
        // Instance ID should be > 0 (global counter)
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_task_send() {
        let spawn_result = shim_task_spawn("TestTask".to_string(), 64);
        assert!(spawn_result.is_ok());
        let handle_id = spawn_result.unwrap() as i64;

        let result = shim_task_send(handle_id, "hello".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_task_send_invalid_handle() {
        let result = shim_task_send(999999, "hello".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_spawn_unique_ids() {
        let result1 = shim_task_spawn("Task1".to_string(), 64);
        let result2 = shim_task_spawn("Task2".to_string(), 64);

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let id1 = result1.unwrap();
        let id2 = result2.unwrap();

        // Each spawn should return a unique handle ID
        assert_ne!(id1, id2);
    }
}
