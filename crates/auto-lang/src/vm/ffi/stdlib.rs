//! Plan 094: Built-in stdlib functions using FFI
//!
//! This module provides high-level FFI functions for common operations
//! like file I/O, environment variables, time, path manipulation, and string operations.
//!
//! These functions use the VMConvertible trait for automatic type conversion.

use std::any::Any;
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;
use crate::vm::ffi::convert::VMConvertible;
use crate::vm::ffi::rust_stdlib::RustStdlibObject;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener as StdTcpListener, TcpStream as StdTcpStream};
use std::path::Path;
use std::path::PathBuf as StdPathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Debug logging macro - only prints when VM debug mode is enabled
macro_rules! vm_debug {
    ($($arg:tt)*) => {
        if crate::is_vm_debug() {
            eprintln!($($arg)*);
        }
    };
}

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
pub const NATIVE_FILE_WALK: u16 = 1010;
pub const NATIVE_FILE_APPEND_TEXT: u16 = 1011;
pub const NATIVE_FILE_READ_LINES: u16 = 1012;

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
pub const NATIVE_PROCESS_SPAWN_WITH_OUTPUT: u16 = 1305;

// Math functions: 1700-1799
pub const NATIVE_MATH_ABS: u16 = 1700;
pub const NATIVE_MATH_MIN: u16 = 1701;
pub const NATIVE_MATH_MAX: u16 = 1702;
pub const NATIVE_MATH_SQRT: u16 = 1703;
pub const NATIVE_MATH_FLOOR: u16 = 1710;
pub const NATIVE_MATH_CEIL: u16 = 1711;
pub const NATIVE_MATH_ROUND: u16 = 1712;
pub const NATIVE_MATH_POW: u16 = 1713;
pub const NATIVE_MATH_MIN_F: u16 = 1714;
pub const NATIVE_MATH_MAX_F: u16 = 1715;
pub const NATIVE_MATH_SIN: u16 = 1716;
pub const NATIVE_MATH_COS: u16 = 1717;
pub const NATIVE_MATH_TAN: u16 = 1718;
pub const NATIVE_MATH_EXP: u16 = 1719;
pub const NATIVE_MATH_LN: u16 = 1720;
pub const NATIVE_MATH_LOG2: u16 = 1721;
pub const NATIVE_MATH_LOG10: u16 = 1722;
pub const NATIVE_MATH_ABS_F: u16 = 1723;
pub const NATIVE_MATH_SIGNUM: u16 = 1724;
pub const NATIVE_MATH_CLAMP: u16 = 1725;

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

// Plan 195: RequestBuilder (2234-2239)
pub const NATIVE_HTTP_REQUEST: u16 = 2234;
pub const NATIVE_HTTP_REQUEST_BUILDER_HEADER: u16 = 2235;
pub const NATIVE_HTTP_REQUEST_BUILDER_BODY: u16 = 2236;
pub const NATIVE_HTTP_REQUEST_BUILDER_TIMEOUT: u16 = 2237;
pub const NATIVE_HTTP_REQUEST_BUILDER_JSON: u16 = 2238;
pub const NATIVE_HTTP_REQUEST_BUILDER_SEND: u16 = 2239;

// Plan 195: Response access (2216-2218)
pub const NATIVE_RESPONSE_STATUS_CODE: u16 = 2216;
pub const NATIVE_RESPONSE_HEADER_GET: u16 = 2217;
pub const NATIVE_RESPONSE_BODY: u16 = 2218;

// Plan 152: 流式 HTTP (2240-2249)
pub const NATIVE_HTTP_GET_STREAM: u16 = 2240;
pub const NATIVE_HTTP_POST_STREAM: u16 = 2241;
pub const NATIVE_HTTP_STREAM_NEXT: u16 = 2242;
pub const NATIVE_HTTP_STREAM_IS_DONE: u16 = 2243;
pub const NATIVE_HTTP_STREAM_CLOSE: u16 = 2244;

// Plan 152: SSE Parser (2245-2249)
pub const NATIVE_SSE_EVENT_NEW: u16 = 2245;
pub const NATIVE_SSE_EVENT_DATA: u16 = 2246;
pub const NATIVE_SSE_EVENT_EVENT: u16 = 2247;
pub const NATIVE_SSE_EVENT_ID: u16 = 2248;
pub const NATIVE_SSE_EVENT_IS_DONE: u16 = 2249;
pub const NATIVE_SSE_PARSE: u16 = 2250;

// Plan 159: HTTP streaming with custom headers (2255-2259)
pub const NATIVE_HTTP_POST_STREAM_WITH_HEADERS: u16 = 2255;

// Regex functions (Plan 159): 2400-2499
pub const NATIVE_REGEX_IS_MATCH: u16 = 2400;
pub const NATIVE_REGEX_FIND_ALL: u16 = 2401;

// Task/Msg functions (Plan 121): 2300-2399
pub const NATIVE_TASK_SPAWN: u16 = 2300;
pub const NATIVE_TASK_SEND: u16 = 2301;
pub const NATIVE_TASK_HANDLE_IS_NULL: u16 = 2302;
pub const NATIVE_TASK_HANDLE_TYPE: u16 = 2303;
pub const NATIVE_TASK_HANDLE_ID: u16 = 2304;
pub const NATIVE_TASK_SYSTEM_START: u16 = 2305;
pub const NATIVE_TASK_SYSTEM_RUN: u16 = 2306; // Plan 124: Sync bridge for async code
pub const NATIVE_TASK_SYSTEM_STOP: u16 = 2307; // Plan 127: Stop the task system scheduler
pub const NATIVE_TASK_SEND_AWAIT: u16 = 2308; // Plan 124 Phase 2.2: send().await backpressure
pub const NATIVE_TASK_ASK: u16 = 2309;        // Plan 124 Phase 2.3: ask/reply RPC
pub const NATIVE_CTX_REPLY: u16 = 2310;       // Plan 127: ctx.reply() for message handlers
pub const NATIVE_TASK_SINGLETON_SEND: u16 = 2311; // Plan 127: Task.send for singleton tasks

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
pub const NATIVE_STR_REPLACE: u16 = 1510;
pub const NATIVE_STR_TO_UPPER: u16 = 1511;
pub const NATIVE_STR_TO_LOWER: u16 = 1512;
pub const NATIVE_STR_REVERSE: u16 = 1513;
pub const NATIVE_STR_FIND: u16 = 1514;
pub const NATIVE_STR_LINES: u16 = 1515;
pub const NATIVE_STR_PARSE_INT: u16 = 1516;
pub const NATIVE_STR_PARSE_FLOAT: u16 = 1517;
pub const NATIVE_STR_SPLIT_ONCE: u16 = 1518;
pub const NATIVE_STR_MATCH_COUNT: u16 = 1519;
pub const NATIVE_STR_REPLACE_FIRST: u16 = 1520;

// Char functions: 1600-1699
pub const NATIVE_CHAR_IS_ALPHA: u16 = 1600;
pub const NATIVE_CHAR_IS_DIGIT: u16 = 1601;
pub const NATIVE_CHAR_IS_ALPHANUM: u16 = 1602;
pub const NATIVE_CHAR_IS_WHITESPACE: u16 = 1603;
pub const NATIVE_CHAR_IS_IDENT: u16 = 1604;
pub const NATIVE_CHAR_TO_LOWER: u16 = 1605;
pub const NATIVE_CHAR_TO_UPPER: u16 = 1606;

// Option functions: 1550-1559 (Plan 200 Task 2.4)
pub const NATIVE_OPTION_OR: u16 = 1550;
pub const NATIVE_OPTION_UNWRAP_OR: u16 = 1551;

// Rust stdlib dynamic dispatch: 3000-3099 (Plan 192)
pub const NATIVE_RUST_STDLIB_DISPATCH: u16 = 3000;

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

/// Remove an empty directory
#[auto_macros::rust_fn("File.remove_dir")]
pub fn shim_file_remove_dir(path: String) -> Result<(), String> {
    fs::remove_dir(&path).map_err(|e| format!("File.remove_dir failed: {} - {}", path, e))
}

/// Remove a directory and all its contents recursively
#[auto_macros::rust_fn("File.remove_dir_all")]
pub fn shim_file_remove_dir_all(path: String) -> Result<(), String> {
    fs::remove_dir_all(&path).map_err(|e| format!("File.remove_dir_all failed: {} - {}", path, e))
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

/// Walk a directory tree, returning all file paths as a JSON array.
///
/// Uses walkdir for recursive directory traversal. Returns a JSON string
/// like `["file1.txt", "dir/file2.rs", ...]`.
pub fn shim_file_walk(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let root = Path::new(&path);
    if !root.exists() {
        return Err(VMError::RuntimeError(format!(
            "File.walk failed: path not found: {}",
            path
        )));
    }

    let mut files: Vec<String> = Vec::new();
    if root.is_file() {
        files.push(path.clone());
    } else {
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(p) = entry.path().to_str() {
                    files.push(p.to_string());
                }
            }
        }
    }

    let json = serde_json::to_string(&files)
        .map_err(|e| VMError::RuntimeError(format!("JSON serialization failed: {}", e)))?;

    json.push_to_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Append text content to a file (creates if doesn't exist)
#[auto_macros::rust_fn("File.append_text")]
pub fn shim_file_append_text(path: String, content: String) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(&path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("File.append_text failed to create dir: {}", e))?;
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("File.append_text failed: {} - {}", path, e))?;
    use std::io::Write;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("File.append_text failed: {} - {}", path, e))?;
    Ok(())
}

/// Read file contents as an array of lines (JSON string array)
pub fn shim_file_read_lines(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let content = fs::read_to_string(&path)
        .map_err(|e| VMError::RuntimeError(format!("File.read_lines failed: {} - {}", path, e)))?;

    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let json = serde_json::to_string(&lines)
        .map_err(|e| VMError::RuntimeError(format!("JSON serialization failed: {}", e)))?;

    json.push_to_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
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

/// Get an environment variable with a default value
#[auto_macros::rust_fn("Env.get_or")]
pub fn shim_env_get_or(key: String, default: String) -> String {
    std::env::var(&key).unwrap_or(default)
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

/// Spawn an external process and capture its stdout/stderr output.
///
/// Returns a JSON object: `{"exit_code": i32, "stdout": String, "stderr": String}`
///
/// This is a manual shim (not `#[rust_fn]`) because it returns a complex JSON value
/// that doesn't map cleanly to a single primitive type.
pub fn shim_process_spawn_with_output(
    task: &mut AutoTask,
    _vm: &AutoVM,
) -> Result<(), VMError> {
    // Pop args as JSON array string: ["cmd", "arg1", "arg2"]
    let args_json: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let args: Vec<String> = serde_json::from_str(&args_json)
        .map_err(|e| VMError::RuntimeError(format!("Invalid args JSON: {}", e)))?;

    if args.is_empty() {
        return Err(VMError::RuntimeError(
            "Process.spawn_with_output failed: empty arguments".into(),
        ));
    }

    let cmd = &args[0];
    let cmd_args: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    let output = std::process::Command::new(cmd)
        .args(&cmd_args)
        .output()
        .map_err(|e| VMError::RuntimeError(format!("Process.spawn_with_output failed: {} - {}", cmd, e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    // Truncate stdout at 64KB to avoid excessive memory use
    let stdout_truncated = if stdout.len() > 65536 {
        format!("{}...(truncated, {} bytes total)", &stdout[..65536], stdout.len())
    } else {
        stdout
    };

    let result_json = serde_json::json!({
        "exit_code": exit_code,
        "stdout": stdout_truncated,
        "stderr": stderr,
    });

    let result_str = serde_json::to_string(&result_json)
        .map_err(|e| VMError::RuntimeError(format!("JSON serialization failed: {}", e)))?;

    result_str.push_to_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

// ============================================================================
// Path Functions (ID 1400-1499)
// ============================================================================

/// Join path components together
#[allow(non_snake_case)]
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

/// Get character at byte index (returns single-char string)
#[auto_macros::rust_fn("Str.char_at")]
pub fn shim_str_char_at(s: String, index: i32) -> i32 {
    if index < 0 || index as usize >= s.len() {
        return 0
    }
    match s[index as usize..].chars().next() {
        Some(c) => c as i32,
        None => 0,
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

/// Replace all occurrences of a pattern in a string
#[auto_macros::rust_fn("Str.replace")]
pub fn shim_str_replace(s: String, from: String, to: String) -> String {
    s.replace(&from, &to)
}

/// Convert string to uppercase
#[auto_macros::rust_fn("Str.to_upper")]
pub fn shim_str_to_upper(s: String) -> String {
    s.to_uppercase()
}

/// Convert string to lowercase
#[auto_macros::rust_fn("Str.to_lower")]
pub fn shim_str_to_lower(s: String) -> String {
    s.to_lowercase()
}

/// Reverse a string (unicode-aware)
#[auto_macros::rust_fn("Str.reverse")]
pub fn shim_str_reverse(s: String) -> String {
    s.chars().rev().collect()
}

/// Find first occurrence of substring, returns byte index or -1
#[auto_macros::rust_fn("Str.find")]
pub fn shim_str_find(s: String, needle: String) -> i32 {
    s.find(&needle).map(|i| i as i32).unwrap_or(-1)
}

/// Split string into lines
#[auto_macros::rust_fn("Str.lines")]
pub fn shim_str_lines(s: String) -> Vec<String> {
    s.lines().map(|l| l.to_string()).collect()
}

/// Parse string as integer
#[auto_macros::rust_fn("Str.parse_int")]
pub fn shim_str_parse_int(s: String) -> Result<i64, String> {
    s.trim().parse::<i64>().map_err(|e| format!("Str.parse_int failed: {}", e))
}

/// Parse string as float
#[auto_macros::rust_fn("Str.parse_float")]
pub fn shim_str_parse_float(s: String) -> Result<f64, String> {
    s.trim().parse::<f64>().map_err(|e| format!("Str.parse_float failed: {}", e))
}

/// Split string at first occurrence of delimiter, returns empty list if not found
#[auto_macros::rust_fn("Str.split_once")]
pub fn shim_str_split_once(s: String, delimiter: String) -> Vec<String> {
    match s.split_once(&delimiter) {
        Some((before, after)) => vec![before.to_string(), after.to_string()],
        None => vec![],
    }
}

/// Count non-overlapping occurrences of pattern in string
#[auto_macros::rust_fn("Str.match_count")]
pub fn shim_str_match_count(s: String, pattern: String) -> i32 {
    s.matches(&pattern).count() as i32
}

/// Replace first occurrence of from with to
#[auto_macros::rust_fn("Str.replace_first")]
pub fn shim_str_replace_first(s: String, from: String, to: String) -> String {
    s.replacen(&from, &to, 1)
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
// Option Functions (ID 1550-1559) — Plan 200 Task 2.4
// ============================================================================

/// Option.or(default) / Option.unwrap_or(default) — returns default if None, unwraps Some
pub fn shim_option_or(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let default_val = task.ram.pop_i32();
    let opt_val = task.ram.pop_i32();

    // Check if it's Option.None (heap object or legacy sentinel)
    if opt_val == -1 {
        task.ram.push_i32(default_val);
        return Ok(());
    }

    if opt_val > 0 {
        let instance_id = opt_val as u64;
        if vm.is_option_none(instance_id) {
            task.ram.push_i32(default_val);
            return Ok(());
        }
        if vm.is_option_some(instance_id) {
            // Unwrap: push the inner value (field _0) as i32
            if let Some(inner) = vm.get_option_inner(instance_id) {
                match inner {
                    auto_val::Value::Int(n) => task.ram.push_i32(n),
                    auto_val::Value::Bool(b) => task.ram.push_i32(if b { 1 } else { 0 }),
                    _other => {
                        // For non-i32 values, push the heap_id of the inner value
                        // (strings, floats etc are heap-stored)
                        task.ram.push_i32(opt_val);
                    }
                }
                return Ok(());
            }
        }
    }

    // Not an Option — return as-is
    task.ram.push_i32(opt_val);
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
#[auto_macros::rust_fn("Math.sqrt")]
pub fn shim_math_sqrt(n: f64) -> f64 {
    n.sqrt()
}

/// Floor of a float
#[auto_macros::rust_fn("Math.floor")]
pub fn shim_math_floor(n: f64) -> f64 {
    n.floor()
}

/// Ceiling of a float
#[auto_macros::rust_fn("Math.ceil")]
pub fn shim_math_ceil(n: f64) -> f64 {
    n.ceil()
}

/// Round a float to nearest integer
#[auto_macros::rust_fn("Math.round")]
pub fn shim_math_round(n: f64) -> f64 {
    n.round()
}

/// Power function: base^exp
#[auto_macros::rust_fn("Math.pow")]
pub fn shim_math_pow(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

/// Minimum of two floats
#[auto_macros::rust_fn("Math.min_f")]
pub fn shim_math_min_f(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// Maximum of two floats
#[auto_macros::rust_fn("Math.max_f")]
pub fn shim_math_max_f(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// Sine function (radians)
#[auto_macros::rust_fn("Math.sin")]
pub fn shim_math_sin(n: f64) -> f64 {
    n.sin()
}

/// Cosine function (radians)
#[auto_macros::rust_fn("Math.cos")]
pub fn shim_math_cos(n: f64) -> f64 {
    n.cos()
}

/// Tangent function (radians)
#[auto_macros::rust_fn("Math.tan")]
pub fn shim_math_tan(n: f64) -> f64 {
    n.tan()
}

/// Exponential function e^x
#[auto_macros::rust_fn("Math.exp")]
pub fn shim_math_exp(n: f64) -> f64 {
    n.exp()
}

/// Natural logarithm ln(x)
#[auto_macros::rust_fn("Math.ln")]
pub fn shim_math_ln(n: f64) -> f64 {
    n.ln()
}

/// Base-2 logarithm log2(x)
#[auto_macros::rust_fn("Math.log2")]
pub fn shim_math_log2(n: f64) -> f64 {
    n.log2()
}

/// Base-10 logarithm log10(x)
#[auto_macros::rust_fn("Math.log10")]
pub fn shim_math_log10(n: f64) -> f64 {
    n.log10()
}

/// Absolute value of a float
#[auto_macros::rust_fn("Math.abs_f")]
pub fn shim_math_abs_f(n: f64) -> f64 {
    n.abs()
}

/// Signum of a float (-1.0, 0.0, or 1.0)
#[auto_macros::rust_fn("Math.signum")]
pub fn shim_math_signum(n: f64) -> f64 {
    n.signum()
}

/// Clamp a value between min and max
#[auto_macros::rust_fn("Math.clamp")]
pub fn shim_math_clamp(n: f64, min: f64, max: f64) -> f64 {
    n.clamp(min, max)
}

/// Arc sine
#[auto_macros::rust_fn("Math.asin")]
pub fn shim_math_asin(n: f64) -> f64 {
    n.asin()
}

/// Arc cosine
#[auto_macros::rust_fn("Math.acos")]
pub fn shim_math_acos(n: f64) -> f64 {
    n.acos()
}

/// Arc tangent
#[auto_macros::rust_fn("Math.atan")]
pub fn shim_math_atan(n: f64) -> f64 {
    n.atan()
}

/// Arc tangent of y/x
#[auto_macros::rust_fn("Math.atan2")]
pub fn shim_math_atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

/// Integer power
#[auto_macros::rust_fn("Math.powi")]
pub fn shim_math_powi(n: f64, exp: i32) -> f64 {
    n.powi(exp)
}

/// Float power
#[auto_macros::rust_fn("Math.powf")]
pub fn shim_math_powf(n: f64, exp: f64) -> f64 {
    n.powf(exp)
}

/// Convert degrees to radians
#[auto_macros::rust_fn("Math.to_radians")]
pub fn shim_math_to_radians(n: f64) -> f64 {
    n.to_radians()
}

/// Convert radians to degrees
#[auto_macros::rust_fn("Math.to_degrees")]
pub fn shim_math_to_degrees(n: f64) -> f64 {
    n.to_degrees()
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

/// Get a value from a JSON object by key
#[auto_macros::rust_fn("Json.get")]
pub fn shim_json_get(json_str: String, key: String) -> String {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    match val.get(&key) {
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

/// Get a value from a JSON array by index
#[auto_macros::rust_fn("Json.get_at")]
pub fn shim_json_get_at(json_str: String, index: i32) -> String {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    match val.as_array() {
        Some(arr) => {
            if index < 0 || index as usize >= arr.len() {
                return String::new();
            }
            arr[index as usize].to_string()
        }
        None => String::new(),
    }
}

/// Get the length of a JSON array or object
#[auto_macros::rust_fn("Json.len")]
pub fn shim_json_len(json_str: String) -> i32 {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return -1,
    };
    match &val {
        serde_json::Value::Array(arr) => arr.len() as i32,
        serde_json::Value::Object(map) => map.len() as i32,
        _ => -1,
    }
}

/// Get the type of a JSON value as a string
#[auto_macros::rust_fn("Json.type_of")]
pub fn shim_json_type_of(json_str: String) -> String {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return "invalid".to_string(),
    };
    match val {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "bool".to_string(),
        serde_json::Value::Number(_) => "number".to_string(),
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(_) => "array".to_string(),
        serde_json::Value::Object(_) => "object".to_string(),
    }
}

/// Get a JSON string value as a plain string
#[auto_macros::rust_fn("Json.as_string")]
pub fn shim_json_as_string(json_str: String) -> String {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    val.as_str().unwrap_or("").to_string()
}

/// Get a JSON number value as f64
#[auto_macros::rust_fn("Json.as_number")]
pub fn shim_json_as_number(json_str: String) -> f64 {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return 0.0,
    };
    val.as_f64().unwrap_or(0.0)
}

/// Get a JSON number value as i64
#[auto_macros::rust_fn("Json.as_int")]
pub fn shim_json_as_int(json_str: String) -> i64 {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    val.as_i64().unwrap_or(0)
}

/// Get a JSON boolean value
#[auto_macros::rust_fn("Json.as_bool")]
pub fn shim_json_as_bool(json_str: String) -> bool {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return false,
    };
    val.as_bool().unwrap_or(false)
}

/// Check if a JSON value is null
#[auto_macros::rust_fn("Json.is_null")]
pub fn shim_json_is_null(json_str: String) -> bool {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return false,
    };
    val.is_null()
}

/// Get the keys of a JSON object as a string list
#[auto_macros::rust_fn("Json.keys")]
pub fn shim_json_keys(json_str: String) -> Vec<String> {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    match val.as_object() {
        Some(map) => map.keys().cloned().collect(),
        None => Vec::new(),
    }
}

/// Check if a JSON object has a given key
#[auto_macros::rust_fn("Json.has_key")]
pub fn shim_json_has_key(json_str: String, key: String) -> bool {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return false,
    };
    val.as_object().map_or(false, |map| map.contains_key(&key))
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

/// Parse a URL and return its components as a JSON string
#[auto_macros::rust_fn("Url.parse")]
pub fn shim_url_parse(url_str: String) -> String {
    // Manual parsing without url crate
    let result = serde_json::json!({
        "url": url_str,
    });
    serde_json::to_string(&result).unwrap_or_default()
}

/// Get the scheme of a URL (e.g., "https")
#[auto_macros::rust_fn("Url.scheme")]
pub fn shim_url_scheme(url_str: String) -> String {
    if let Some(pos) = url_str.find("://") {
        url_str[..pos].to_string()
    } else {
        String::new()
    }
}

/// Get the host of a URL (e.g., "example.com")
#[auto_macros::rust_fn("Url.host")]
pub fn shim_url_host(url_str: String) -> String {
    let without_scheme = if let Some(pos) = url_str.find("://") {
        &url_str[pos + 3..]
    } else {
        &url_str
    };
    // Remove userinfo if present
    let after_at = if let Some(pos) = without_scheme.rfind('@') {
        &without_scheme[pos + 1..]
    } else {
        without_scheme
    };
    // Take up to : or /
    let end = after_at.find(|c: char| c == ':' || c == '/' || c == '?' || c == '#')
        .unwrap_or(after_at.len());
    after_at[..end].to_string()
}

/// Get the port of a URL (returns -1 if no explicit port)
#[auto_macros::rust_fn("Url.port")]
pub fn shim_url_port(url_str: String) -> i32 {
    let without_scheme = if let Some(pos) = url_str.find("://") {
        &url_str[pos + 3..]
    } else {
        &url_str
    };
    let after_at = if let Some(pos) = without_scheme.rfind('@') {
        &without_scheme[pos + 1..]
    } else {
        without_scheme
    };
    // Find the colon after host
    if let Some(colon_pos) = after_at.find(':') {
        let after_colon = &after_at[colon_pos + 1..];
        let end = after_colon.find(|c: char| c == '/' || c == '?' || c == '#')
            .unwrap_or(after_colon.len());
        after_colon[..end].parse::<i32>().unwrap_or(-1)
    } else {
        -1
    }
}

/// Get the path of a URL
#[auto_macros::rust_fn("Url.path")]
pub fn shim_url_path(url_str: String) -> String {
    let without_scheme = if let Some(pos) = url_str.find("://") {
        &url_str[pos + 3..]
    } else if url_str.starts_with('/') {
        // Relative URL
        return extract_path(url_str);
    } else {
        &url_str
    };
    // Skip host:port
    let after_host = if let Some(pos) = without_scheme.find('/') {
        &without_scheme[pos..]
    } else {
        return "/".to_string();
    };
    extract_path(after_host.to_string())
}

/// Get the query string of a URL
#[auto_macros::rust_fn("Url.query")]
pub fn shim_url_query(url_str: String) -> String {
    if let Some(qpos) = url_str.find('?') {
        let after_q = &url_str[qpos + 1..];
        if let Some(hpos) = after_q.find('#') {
            after_q[..hpos].to_string()
        } else {
            after_q.to_string()
        }
    } else {
        String::new()
    }
}

/// Get the fragment of a URL
#[auto_macros::rust_fn("Url.fragment")]
pub fn shim_url_fragment(url_str: String) -> String {
    if let Some(pos) = url_str.rfind('#') {
        url_str[pos + 1..].to_string()
    } else {
        String::new()
    }
}

/// Helper: extract path portion from a string that starts with /
fn extract_path(s: String) -> String {
    let end = s.find(|c: char| c == '?' || c == '#').unwrap_or(s.len());
    if end == 0 {
        "/".to_string()
    } else {
        s[..end].to_string()
    }
}

// ============================================================================
// Net Functions (ID 2100-2199)
// ============================================================================

/// Global handle counter for net resources
static NET_HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);

// Thread-local storage for TCP listeners
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
        Err(_) => {
            task.ram.push_i32(0); // Return 0 on failure
            return Ok(());
        }
    };

    listener.set_nonblocking(false).ok();

    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    TCP_LISTENERS.with(|listeners| {
        listeners.borrow_mut().insert(handle, listener);
    });

    task.ram.push_i32(handle as i32);
    Ok(())
}

/// Accept a new connection from listener
/// Returns stream handle (positive) on success, 0 on failure
pub fn shim_net_tcp_listener_accept(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let listener_handle: i32 = task.ram.pop_i32();

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
            task.ram.push_i32(handle as i32);
        }
        None => {
            task.ram.push_i32(0); // Return 0 on failure
        }
    }
    Ok(())
}

/// Get local address of listener
#[auto_macros::rust_fn("Net.tcp_listener_local_addr")]
pub fn shim_net_tcp_listener_local_addr(listener_handle: i32) -> String {
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
    let listener_handle: i32 = task.ram.pop_i32();
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
        Err(_) => {
            task.ram.push_i32(0);
            return Ok(());
        }
    };

    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    TCP_STREAMS.with(|streams| {
        streams.borrow_mut().insert(handle, stream);
    });

    task.ram.push_i32(handle as i32);
    Ok(())
}

/// Read data from stream
/// Returns number of bytes read, or -1 on error
pub fn shim_net_tcp_stream_read(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let buf_size: i32 = task.ram.pop_i32();
    let stream_handle: i32 = task.ram.pop_i32();

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
    let stream_handle: i32 = task.ram.pop_i32();

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
    let stream_handle: i32 = task.ram.pop_i32();

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
pub fn shim_net_tcp_stream_read_line(stream_handle: i32) -> String {
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
    let stream_handle: i32 = task.ram.pop_i32();

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
    let stream_handle: i32 = task.ram.pop_i32();
    TCP_STREAMS.with(|streams| {
        streams.borrow_mut().remove(&(stream_handle as u64));
    });
    Ok(())
}

/// Get peer address
#[auto_macros::rust_fn("Net.tcp_stream_peer_addr")]
pub fn shim_net_tcp_stream_peer_addr(stream_handle: i32) -> String {
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
    let stream_handle: i32 = task.ram.pop_i32();

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
    let stream_handle: i32 = task.ram.pop_i32();

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

// HTTP Response data stored in thread-local
thread_local! {
    static HTTP_RESPONSES: std::cell::RefCell<std::collections::HashMap<u64, HttpResponseData>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

// Plan 152: HTTP 流数据存储
thread_local! {
    static HTTP_STREAMS: std::cell::RefCell<std::collections::HashMap<u64, HttpStreamData>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

// Plan 195: RequestBuilder 数据存储
#[derive(Debug, Clone)]
struct HttpRequestBuilderData {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: Option<u64>,
}

thread_local! {
    static HTTP_REQUEST_BUILDERS: std::cell::RefCell<std::collections::HashMap<u64, HttpRequestBuilderData>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// HTTP Response data
#[derive(Debug, Clone, Default)]
struct HttpResponseData {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

/// Plan 154: HTTP 流数据（真正的流式实现）
/// 使用 reqwest::blocking::Response 逐 chunk 读取
struct HttpStreamData {
    url: String,
    response: Option<reqwest::blocking::Response>,
    done: bool,
    status_code: u16,
}

impl std::fmt::Debug for HttpStreamData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpStreamData")
            .field("url", &self.url)
            .field("done", &self.done)
            .field("status_code", &self.status_code)
            .field("has_response", &self.response.is_some())
            .finish()
    }
}

impl HttpStreamData {
    fn new(url: String, response: reqwest::blocking::Response) -> Self {
        let status = response.status().as_u16();
        Self {
            url,
            response: Some(response),
            done: false,
            status_code: status,
        }
    }
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

/// Start server listening (blocking, using tokio)
pub fn shim_http_server_listen(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let addr: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let _server: i64 = task.ram.pop_i64();

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| VMError::RuntimeError(format!("Failed to create tokio runtime: {}", e)))?;

    rt.block_on(async {
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[HTTP] Server bind failed: {}", e);
                return;
            }
        };
        eprintln!("[HTTP] Server listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((mut stream, _peer)) => {
                    tokio::spawn(async move {
                        use tokio::io::AsyncReadExt;
                        use tokio::io::AsyncWriteExt;

                        let mut buf = vec![0u8; 4096];
                        match stream.read(&mut buf).await {
                            Ok(n) if n > 0 => {
                                let response = "HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/plain\r\n\
                                    Content-Length: 27\r\n\
                                    Connection: close\r\n\
                                    \r\n\
                                    Hello from Auto HTTP Server";
                                let _ = stream.write_all(response.as_bytes()).await;
                            }
                            _ => {}
                        }
                    });
                }
                Err(e) => eprintln!("[HTTP] Accept error: {}", e),
            }
        }
    });

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

// ============================================================================
// Plan 195: RequestBuilder FFI
// ============================================================================

/// Create a new RequestBuilder handle
/// http_request(method, url) -> RequestBuilder handle
pub fn shim_http_request(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let method: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let data = HttpRequestBuilderData {
        method,
        url,
        headers: vec![],
        body: None,
        timeout_ms: None,
    };
    HTTP_REQUEST_BUILDERS.with(|b| b.borrow_mut().insert(handle, data));
    task.ram.push_i64(handle as i64);
    Ok(())
}

/// Add a header to RequestBuilder
/// request_builder_header(rb, key, value) -> rb
pub fn shim_request_builder_header(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let value: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = task.ram.pop_i64();

    HTTP_REQUEST_BUILDERS.with(|b| {
        if let Some(builder) = b.borrow_mut().get_mut(&(rb_handle as u64)) {
            builder.headers.push((key, value));
        }
    });

    task.ram.push_i64(rb_handle);
    Ok(())
}

/// Set body on RequestBuilder
/// request_builder_body(rb, body) -> rb
pub fn shim_request_builder_body(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = task.ram.pop_i64();

    HTTP_REQUEST_BUILDERS.with(|b| {
        if let Some(builder) = b.borrow_mut().get_mut(&(rb_handle as u64)) {
            builder.body = Some(body);
        }
    });

    task.ram.push_i64(rb_handle);
    Ok(())
}

/// Set timeout on RequestBuilder
/// request_builder_timeout(rb, ms) -> rb
pub fn shim_request_builder_timeout(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let ms: i64 = task.ram.pop_i64();
    let rb_handle: i64 = task.ram.pop_i64();

    HTTP_REQUEST_BUILDERS.with(|b| {
        if let Some(builder) = b.borrow_mut().get_mut(&(rb_handle as u64)) {
            builder.timeout_ms = Some(ms as u64);
        }
    });

    task.ram.push_i64(rb_handle);
    Ok(())
}

/// Set JSON body on RequestBuilder
/// request_builder_json(rb, data) -> rb
pub fn shim_request_builder_json(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let data: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = task.ram.pop_i64();

    HTTP_REQUEST_BUILDERS.with(|b| {
        if let Some(builder) = b.borrow_mut().get_mut(&(rb_handle as u64)) {
            builder.body = Some(data);
            if !builder.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")) {
                builder.headers.push(("Content-Type".to_string(), "application/json".to_string()));
            }
        }
    });

    task.ram.push_i64(rb_handle);
    Ok(())
}

/// Send RequestBuilder and return Response handle
/// request_builder_send(rb) -> Response handle
pub fn shim_request_builder_send(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let rb_handle: i64 = task.ram.pop_i64();

    let builder_data = HTTP_REQUEST_BUILDERS.with(|b| {
        b.borrow_mut().remove(&(rb_handle as u64))
    }).ok_or_else(|| VMError::RuntimeError(format!("Invalid RequestBuilder handle: {}", rb_handle)))?;

    let response_handle = simple_http_request(
        &builder_data.method,
        &builder_data.url,
        builder_data.body.as_deref(),
    );

    task.ram.push_i64(response_handle);
    Ok(())
}

// ============================================================================
// Plan 195: Enhanced Response access methods
// ============================================================================

/// Get status code from Response handle
/// response_status_code(res_handle) -> int
pub fn shim_response_status_code(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let res_handle: i64 = task.ram.pop_i64();

    let status = HTTP_RESPONSES.with(|r| {
        r.borrow().get(&(res_handle as u64)).map(|res| res.status as i64)
    }).unwrap_or(0);

    task.ram.push_i64(status);
    Ok(())
}

/// Get header value from Response handle
/// response_header_get(res_handle, key) -> str
pub fn shim_response_header_get(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let key: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_handle: i64 = task.ram.pop_i64();

    let value = HTTP_RESPONSES.with(|r| {
        let responses = r.borrow();
        responses.get(&(res_handle as u64)).and_then(|res| {
            res.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(&key)).map(|(_, v)| v.clone())
        })
    }).unwrap_or_default();

    value.push_to_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Get raw body bytes from Response handle
/// response_body(res_handle) -> []byte
pub fn shim_response_body(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let res_handle: i64 = task.ram.pop_i64();

    let body_bytes = HTTP_RESPONSES.with(|r| {
        r.borrow().get(&(res_handle as u64)).map(|res| res.body.clone())
    }).unwrap_or_default();

    let byte_vec: Vec<i32> = body_bytes.into_iter().map(|b| b as i32).collect();
    byte_vec.push_to_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

// ============================================================================
// Plan 154: 真正的流式 HTTP 函数（使用 reqwest::blocking）
// ============================================================================

/// 创建流式 HTTP GET 请求
/// 使用 reqwest::blocking 发起请求，将 Response 存入流式句柄
pub fn shim_http_get_stream(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let response = reqwest::blocking::Client::new()
        .get(&url)
        .send()
        .map_err(|e| VMError::RuntimeError(format!("HTTP GET stream failed: {}", e)))?;

    let stream_handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let stream_data = HttpStreamData::new(url, response);
    HTTP_STREAMS.with(|streams| {
        streams.borrow_mut().insert(stream_handle, stream_data);
    });

    task.ram.push_i64(stream_handle as i64);
    Ok(())
}

/// 创建流式 HTTP POST 请求
pub fn shim_http_post_stream(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let response = reqwest::blocking::Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .map_err(|e| VMError::RuntimeError(format!("HTTP POST stream failed: {}", e)))?;

    let stream_handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let stream_data = HttpStreamData::new(url, response);
    HTTP_STREAMS.with(|streams| {
        streams.borrow_mut().insert(stream_handle, stream_data);
    });

    task.ram.push_i64(stream_handle as i64);
    Ok(())
}

/// 从流中读取下一个数据块
/// 使用 reqwest::blocking::Response 的 std::io::Read trait 逐块读取
pub fn shim_http_stream_next(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle: i64 = task.ram.pop_i64();

    let result = HTTP_STREAMS.try_with(|streams| -> Result<(), VMError> {
        let mut streams = streams.borrow_mut();
        let stream = streams.get_mut(&(handle as u64))
            .ok_or_else(|| VMError::RuntimeError(format!("Invalid HTTP stream handle: {}", handle)))?;

        if stream.done {
            "[DONE]".to_string().push_to_stack(task, _vm)
                .map_err(|e| VMError::RuntimeError(e.to_string()))?;
            return Ok(());
        }

        if let Some(ref mut response) = stream.response {
            // Read a chunk (up to 8KB) from the response
            use std::io::Read;
            let mut buf = vec![0u8; 8192];
            match response.read(&mut buf) {
                Ok(0) => {
                    // EOF - stream complete
                    stream.done = true;
                    stream.response = None;
                    "[DONE]".to_string().push_to_stack(task, _vm)
                        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
                }
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]).to_string();
                    text.push_to_stack(task, _vm)
                        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
                }
                Err(e) => {
                    stream.done = true;
                    stream.response = None;
                    return Err(VMError::RuntimeError(format!("Stream read error: {}", e)));
                }
            }
        } else {
            "[DONE]".to_string().push_to_stack(task, _vm)
                .map_err(|e| VMError::RuntimeError(e.to_string()))?;
        }

        Ok(())
    });

    result.map_err(|e| VMError::RuntimeError(e.to_string()))??;
    Ok(())
}

/// 检查流是否完成
pub fn shim_http_stream_is_done(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle: i64 = task.ram.pop_i64();

    let result = HTTP_STREAMS.try_with(|streams| -> Result<(), VMError> {
        let streams = streams.borrow();
        let stream = streams.get(&(handle as u64))
            .ok_or_else(|| VMError::RuntimeError(format!("Invalid HTTP stream handle: {}", handle)))?;

        task.ram.push_i32(if stream.done { 1 } else { 0 });
        Ok(())
    });

    result.map_err(|e| VMError::RuntimeError(e.to_string()))??;
    Ok(())
}

/// 关闭流
pub fn shim_http_stream_close(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle: i64 = task.ram.pop_i64();

    HTTP_STREAMS.with(|streams| {
        streams.borrow_mut().remove(&(handle as u64));
    });

    Ok(())
}

/// POST streaming with custom headers (Plan 159: AutoCode agent support).
///
/// Stack args (bottom to top): url, body, headers_json
/// - url: String
/// - body: String (JSON body)
/// - headers_json: String (JSON object like `{"Authorization": "Bearer xxx", "Content-Type": "application/json"}`)
///
/// Returns an HTTP stream handle (i64) on the stack.
pub fn shim_http_post_stream_with_headers(
    task: &mut AutoTask,
    _vm: &AutoVM,
) -> Result<(), VMError> {
    let headers_json: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let body: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Parse headers JSON
    let headers_map: std::collections::HashMap<String, String> =
        serde_json::from_str(&headers_json).unwrap_or_default();

    let mut request = reqwest::blocking::Client::new()
        .post(&url)
        .body(body);

    for (key, value) in &headers_map {
        request = request.header(key.as_str(), value.as_str());
    }

    let response = request
        .send()
        .map_err(|e| VMError::RuntimeError(format!("HTTP POST stream with headers failed: {}", e)))?;

    let stream_handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let stream_data = HttpStreamData::new(url, response);
    HTTP_STREAMS.with(|streams| {
        streams.borrow_mut().insert(stream_handle, stream_data);
    });

    task.ram.push_i64(stream_handle as i64);
    Ok(())
}

// ============================================================================
// Regex Functions (Plan 159: 2400-2499)
// ============================================================================

/// Check if a string matches a regex pattern.
///
/// Returns 1 if match found, 0 otherwise.
#[auto_macros::rust_fn("Regex.is_match")]
pub fn shim_regex_is_match(pattern: String, text: String) -> Result<i32, String> {
    let re = regex::Regex::new(&pattern)
        .map_err(|e| format!("Regex.is_match failed: invalid pattern '{}': {}", pattern, e))?;
    Ok(if re.is_match(&text) { 1 } else { 0 })
}

/// Find all matches of a regex pattern in text.
///
/// Returns a JSON array of match objects: `[{"match": "...", "start": 0, "end": 5}, ...]`
pub fn shim_regex_find_all(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let text: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let pattern: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let re = regex::Regex::new(&pattern)
        .map_err(|e| VMError::RuntimeError(format!("Regex.find_all failed: invalid pattern '{}': {}", pattern, e)))?;

    let mut matches: Vec<serde_json::Value> = Vec::new();
    for cap in re.find_iter(&text) {
        matches.push(serde_json::json!({
            "match": cap.as_str(),
            "start": cap.start(),
            "end": cap.end(),
        }));
        // Limit to 250 matches to avoid excessive memory
        if matches.len() >= 250 {
            break;
        }
    }

    let json = serde_json::to_string(&matches)
        .map_err(|e| VMError::RuntimeError(format!("JSON serialization failed: {}", e)))?;

    json.push_to_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

// ============================================================================
// Plan 152: SSE Parser Functions
// ============================================================================

/// Parse SSE text chunk
/// Returns: Array of SSE event objects (JSON strings)
#[auto_macros::rust_fn("parse_sse")]
pub fn shim_sse_parse(chunk: String) -> Vec<String> {
    // Simple SSE parser implementation
    let mut events = Vec::new();
    let mut current_data = String::new();
    let mut current_event = String::new();
    let mut current_id = String::new();

    for line in chunk.lines() {
        // Skip comment lines
        if line.starts_with(':') {
            continue;
        }

        if line.is_empty() {
            // Empty line means event end
            if !current_data.is_empty() {
                let mut json = String::from("{\"data\":\"");
                json.push_str(&current_data.replace('"', "\\\""));
                json.push_str("\"");
                if !current_event.is_empty() {
                    json.push_str(",\"event\":\"");
                    json.push_str(&current_event);
                    json.push_str("\"");
                }
                if !current_id.is_empty() {
                    json.push_str(",\"id\":\"");
                    json.push_str(&current_id);
                    json.push_str("\"");
                }
                json.push_str("}");
                events.push(json);
            }
            current_data.clear();
            current_event.clear();
            current_id.clear();
        } else if line.starts_with("data:") {
            let data = &line[5..].trim_start();
            if !current_data.is_empty() {
                current_data.push('\n');
            }
            current_data.push_str(data);
        } else if line.starts_with("event:") {
            current_event = line[6..].trim().to_string();
        } else if line.starts_with("id:") {
            current_id = line[3..].trim().to_string();
        }
    }

    // Don't forget the last event if no trailing newline
    if !current_data.is_empty() {
        let mut json = String::from("{\"data\":\"");
        json.push_str(&current_data.replace('"', "\\\""));
        json.push_str("\"");
        if !current_event.is_empty() {
            json.push_str(",\"event\":\"");
            json.push_str(&current_event);
            json.push_str("\"");
        }
        if !current_id.is_empty() {
            json.push_str(",\"id\":\"");
            json.push_str(&current_id);
            json.push_str("\"");
        }
        json.push_str("}");
        events.push(json);
    }

    events
}

/// HTTP request using reqwest::blocking
fn simple_http_request(method: &str, url: &str, body: Option<&str>) -> i64 {
    let client = reqwest::blocking::Client::new();
    let mut builder = match method {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => client.get(url),
    };

    if let Some(b) = body {
        builder = builder
            .header("Content-Type", "application/json")
            .body(b.to_string());
    }

    match builder.send() {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers: Vec<(String, String)> = response
                .headers()
                .iter()
                .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string())))
                .collect();
            let body_bytes = response.bytes().unwrap_or_default().to_vec();

            let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let resp_data = HttpResponseData {
                status,
                headers,
                body: body_bytes,
            };

            HTTP_RESPONSES.with(|r| {
                r.borrow_mut().insert(handle, resp_data);
            });

            handle as i64
        }
        Err(e) => shim_http_internal_error(format!("HTTP {} failed: {}", method, e)),
    }
}

// ============================================================================
// Task/Msg Functions (Plan 121)
// ============================================================================

use crate::vm::task_system::{TaskHandle, TaskInstance, TaskRegistry};

// TaskHandle wrapper for passing through VM
// The handle is stored as a tuple: (task_type: String, instance_id: u64, tx_ptr: u64)
// We use a thread-local storage to keep actual handles alive
thread_local! {
    static TASK_HANDLES: std::cell::RefCell<std::collections::HashMap<u64, TaskHandle>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    // Keep TaskInstance alive so the receiver (rx) doesn't get dropped
    static TASK_INSTANCES: std::cell::RefCell<std::collections::HashMap<u64, TaskInstance>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    // Map task_type name -> handle_id for singleton tasks (#[single] annotation)
    static SINGLETON_TASKS: std::cell::RefCell<std::collections::HashMap<String, u64>> =
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
/// A handle ID (i32) that can be used to reference the task
#[auto_macros::rust_fn("Task.spawn")]
pub fn shim_task_spawn(task_type: String, capacity: i32) -> i32 {
    vm_debug!("DEBUG shim_task_spawn: task_type='{}', capacity={}", task_type, capacity);
    let cap = if capacity <= 0 { 64 } else { capacity as usize };

    // Create a new task instance
    let mut instance = TaskInstance::new(task_type.clone(), cap);
    let handle = instance.handle.clone();

    // Generate a unique handle ID
    let handle_id = TASK_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    vm_debug!("DEBUG shim_task_spawn: generated handle_id={}", handle_id);

    // Plan 128: Get the global registry and store mailbox receiver
    // This allows spawn_initial_tasks to use the actual receiver
    let registry = get_global_task_registry();
    if let Some(receiver) = instance.take_receiver() {
        registry.store_mailbox_receiver(task_type.clone(), instance.instance_id, receiver);
        vm_debug!("DEBUG shim_task_spawn: stored receiver for {}#{}", task_type, instance.instance_id);
    } else {
        eprintln!("WARN shim_task_spawn: failed to take receiver for {}#{}", task_type, instance.instance_id);
    }

    // Register the handle with the global registry
    registry.register_instance(handle.clone());

    // Store the handle in thread-local storage (for legacy send operations)
    TASK_HANDLES.with(|handles| {
        handles.borrow_mut().insert(handle_id, handle);
    });

    // Note: We no longer store the full TaskInstance in TASK_INSTANCES
    // since the receiver has been extracted and stored in TaskRegistry

    vm_debug!("DEBUG shim_task_spawn: returning {}", handle_id as i32);
    // Return as i32 (fits in one stack slot)
    handle_id as i32
}

/// Send a message to a task
///
/// # Arguments
/// * `handle_id` - The handle ID returned by Task.spawn
/// * `msg` - The message value to send (enum variant as i32)
///
/// # Returns
/// 1 on success, 0 on failure
#[auto_macros::rust_fn("TaskHandle.send")]
pub fn shim_task_send(handle_id: i32, msg: i32) -> i32 {
    let id = handle_id as u64;

    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            match handle.try_send(auto_val::Value::Int(msg)) {
                Ok(()) => 1,
                Err(e) => {
                    eprintln!("TaskHandle.send failed: {}", e);
                    0
                }
            }
        } else {
            eprintln!("TaskHandle.send failed: invalid handle ID {}", id);
            0
        }
    })
}

/// Send a message to a singleton task
///
/// # Arguments
/// * `task_type` - The task type name (e.g., "MonitorTask")
/// * `msg` - The message value to send (enum variant as i32)
///
/// # Returns
/// 1 on success, 0 on failure
#[auto_macros::rust_fn("Task.singleton_send")]
pub fn shim_task_singleton_send(task_type: String, msg: i32) -> i32 {
    // First, check if the singleton task already exists
    let existing_handle_id = SINGLETON_TASKS.with(|singletons| {
        let singletons = singletons.borrow();
        singletons.get(&task_type).copied()
    });

    let handle_id = if let Some(id) = existing_handle_id {
        id
    } else {
        // Auto-spawn the singleton task on first access
        vm_debug!("DEBUG: Auto-spawning singleton task '{}'", task_type);
        let mut instance = TaskInstance::new(task_type.clone(), 64);
        let handle = instance.handle.clone();
        let new_id = TASK_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Plan 128: Extract and store the mailbox receiver in global TaskRegistry
        if let Some(receiver) = instance.take_receiver() {
            let registry = get_global_task_registry();
            registry.store_mailbox_receiver(task_type.clone(), instance.instance_id, receiver);
            registry.register_singleton(task_type.clone(), handle.clone());
            vm_debug!("DEBUG: Stored receiver for singleton {}#{}", task_type, instance.instance_id);
        } else {
            eprintln!("WARN: Failed to take receiver for singleton {}#{}", task_type, instance.instance_id);
            // Still register the singleton even if receiver extraction failed
            let registry = get_global_task_registry();
            registry.register_singleton(task_type.clone(), handle.clone());
        }

        // Store the handle in thread-local storage
        TASK_HANDLES.with(|handles| {
            handles.borrow_mut().insert(new_id, handle);
        });

        // Register as singleton in thread-local tracking
        SINGLETON_TASKS.with(|singletons| {
            singletons.borrow_mut().insert(task_type.clone(), new_id);
        });

        new_id
    };

    // Now send the message
    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&handle_id) {
            match handle.try_send(auto_val::Value::Int(msg)) {
                Ok(()) => 1,
                Err(e) => {
                    eprintln!("Task.send failed: {}", e);
                    0
                }
            }
        } else {
            eprintln!("Task.send failed: handle not found for {}", task_type);
            0
        }
    })
}

// Plan 124 Phase 2.2: send().await with backpressure
//
// Sends a message to a task, blocking (suspending) if the mailbox is full.
// This is the async version that supports backpressure.
//
// In Phase 2.2, this is a simplified implementation that uses try_send
// with immediate return. Full implementation would:
// 1. Check if mailbox has capacity
// 2. If full, suspend the current task
// 3. Resume when space becomes available
//
// Usage in AutoLang:
//   TaskHandle.send_await(msg)  // Returns Future<void>
//   TaskHandle.send(msg).await  // Syntactic sugar for above
#[auto_macros::rust_fn("TaskHandle.send_await")]
pub fn shim_task_send_await(handle_id: i64, msg: String) -> Result<i64, String> {
    let id = handle_id as u64;

    // Phase 2.2: Simplified - just wrap send in a Future
    // Full implementation would actually suspend if mailbox is full
    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            let auto_str: auto_val::AutoStr = msg.into();
            match handle.try_send(auto_val::Value::Str(auto_str)) {
                Ok(()) => {
                    // Return a completed Future (represented as 0 for now)
                    // In full implementation, this would return a Future that
                    // resolves when the message is actually delivered
                    Ok(0)
                }
                Err(e) => Err(format!("TaskHandle.send_await failed: {}", e)),
            }
        } else {
            Err(format!("TaskHandle.send_await failed: invalid handle ID {}", id))
        }
    })
}

// Plan 124 Phase 2.3: ask/reply bidirectional RPC
//
// Sends a message to a task and returns a Future that will receive the reply.
// The receiver uses `reply expr` to send back a response.
//
// Usage in AutoLang:
//   let result = TaskHandle.ask(msg).await.?
//
// Implementation:
// 1. Creates a oneshot channel for the reply
// 2. Sends the message with the reply sender attached
// 3. Returns a Future that awaits the oneshot receiver
//
// Phase 2.3: Simplified implementation that just wraps send
#[auto_macros::rust_fn("TaskHandle.ask")]
pub fn shim_task_ask(handle_id: i64, msg: String) -> Result<i64, String> {
    let id = handle_id as u64;

    // Phase 2.3: Simplified - just wrap send in a Future
    // Full implementation would:
    // 1. Create a oneshot channel (reply_tx, reply_rx)
    // 2. Send message with reply_tx attached
    // 3. Return Future that awaits reply_rx
    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            let auto_str: auto_val::AutoStr = msg.into();
            match handle.try_send(auto_val::Value::Str(auto_str)) {
                Ok(()) => {
                    // Return a completed Future (represented as 0 for now)
                    // In full implementation, this would return a FutureReceiver
                    Ok(0)
                }
                Err(e) => Err(format!("TaskHandle.ask failed: {}", e)),
            }
        } else {
            Err(format!("TaskHandle.ask failed: invalid handle ID {}", id))
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
    use crate::vm::scheduler::GlobalMeta;
    use crate::vm::virt_memory::VirtualFlash;
    use crate::vm::native::NativeInterface;
    use std::collections::HashMap;

    let registry = get_global_task_registry();

    // Build GlobalMeta if not already set
    // For MVP, we create a minimal GlobalMeta with empty bytecode
    // Full integration would populate this from the compiled VM state
    if registry.get_global_meta().is_none() {
        let meta = Arc::new(GlobalMeta::from_components(
            VirtualFlash::new(0),
            Vec::new(),
            NativeInterface::new(),
            HashMap::new(),
        ));
        registry.set_global_meta(meta);
    }

    registry.start_scheduler();
    Ok(())
}

/// Stop the task system scheduler
///
/// This method signals the scheduler to stop and executes all stop hooks in LIFO order.
/// Unlike start_scheduler, this does NOT block - it just signals shutdown.
///
/// # Note
/// This is useful for testing and graceful shutdown without Ctrl+C.
#[auto_macros::rust_fn("TaskSystem.stop")]
pub fn shim_task_system_stop() -> Result<(), String> {
    let registry = get_global_task_registry();

    // Signal the scheduler to stop (Plan 127)
    registry.signal_shutdown();

    Ok(())
}

// Plan 224: TaskSystem.run() - VM-aware async body execution
//
// Executes an async block by looking up the future in the VM registry
// and running its body bytecode via execute_single_frame.
//
// Usage in AutoLang:
//   TaskSystem.run(~{
//     // async code here
//   })
//
// The future_id is encoded on the VM stack as (future_id << 8) | 0xF0.
#[allow(non_snake_case)]
pub fn shim_task_system_run(
    task: &mut crate::vm::task::AutoTask,
    vm: &crate::vm::engine::AutoVM,
) -> Result<(), crate::vm::engine::VMError> {
    use crate::vm::engine::{FrameResult, FutureState};

    // Pop the future encoding from stack
    let future_bits = task.ram.pop_i32();

    // Decode future ID
    if (future_bits & 0xFF) != 0xF0 {
        // Not a valid future — push nil and return
        task.ram.push_i32(0);
        return Ok(());
    }
    let future_id = (future_bits >> 8) as u32;

    // Look up the future
    let future_arc = match vm.futures.get(&future_id) {
        Some(f) => f,
        None => {
            task.ram.push_i32(0);
            return Ok(());
        }
    };

    let body_offset = {
        let fv = future_arc.read().unwrap();
        fv.body_offset
    };

    // Execute the async body using VM engine
    let saved_ip = task.ip;
    task.ip = body_offset as usize;

    let mut result_value = auto_val::Value::Int(0);
    let success;

    loop {
        match vm.execute_single_frame(task, 10_000) {
            FrameResult::Return => {
                if task.ram.sp > task.bp + 1 {
                    let raw = task.ram.pop_i32();
                    result_value = auto_val::Value::Int(raw);
                }
                success = true;
                break;
            }
            FrameResult::AwaitFuture { future_id: inner_id, body_offset: inner_offset } => {
                vm.handle_await_future(task, inner_id, inner_offset)?;
            }
            FrameResult::BudgetExhausted | FrameResult::Yielded => {
                // Continue executing
                continue;
            }
            FrameResult::Error(e) => {
                if let Some(fv) = vm.futures.get(&future_id) {
                    fv.write().unwrap().state = FutureState::Failed;
                }
                task.ip = saved_ip;
                return Err(e);
            }
            FrameResult::Continue => unreachable!(),
        }
    }

    task.ip = saved_ip;

    // Update future state
    if let Some(fv) = vm.futures.get(&future_id) {
        let mut future = fv.write().unwrap();
        future.state = if success { FutureState::Ready } else { FutureState::Failed };
        future.result = Some(result_value.clone());
    }

    // Push result onto stack
    match &result_value {
        auto_val::Value::Int(n) => task.ram.push_i32(*n),
        auto_val::Value::Nil => task.ram.push_i32(0),
        _ => task.ram.push_i32(0),
    }

    Ok(())
}

/// Plan 224: Helper for executing async operations in FFI shims.
/// Creates an independent tokio runtime and runs the future with block_on.
/// Note: This blocks the current thread. Do not call from within an existing tokio runtime.
pub fn ffi_async_block_on<F, T>(f: F) -> Result<T, crate::vm::engine::VMError>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| crate::vm::engine::VMError::RuntimeError(format!("Failed to create tokio runtime: {}", e)))?;
    rt.block_on(f)
        .map_err(|e| crate::vm::engine::VMError::RuntimeError(e))
}

// Plan 127 Phase 3: ctx.reply() - Send reply from message handler
//
// This function is called from task message handlers that have a context parameter:
//
//   on(ctx) {
//       "ping" => { ctx.reply("pong") }
//   }
//
// The reply is sent via the current task's MessageContext, which was set up
// when the handler was invoked by HANDLE_MSG.
//
// Note: This FFI shim is for fallback compatibility. The primary execution
// path uses the REPLY OpCode directly, which reads the MessageContext from
// task.current_msg_context set by HANDLE_MSG.
#[auto_macros::rust_fn("ctx.reply")]
pub fn shim_ctx_reply(value: i64) -> Result<(), String> {
    // The actual reply is handled by REPLY OpCode in engine.rs
    // This shim exists for FFI registration and potential future use
    // For now, we just return Ok - the real work is done by OpCode::REPLY
    if crate::is_vm_debug() {
        eprintln!("[ctx.reply] FFI called with value: {}", value);
    }
    Ok(())
}

// ============================================================================
// Registration Function
// ============================================================================

/// Register all stdlib FFI functions with the NativeInterface
/// Register manual shims that cannot use #[rust_fn] (custom VM access, variadic args, etc.).
///
/// #[rust_fn]-annotated functions are auto-registered via inventory (build_from_inventory).
/// This function only registers the ~54 manual shims that need special handling.
pub fn register_stdlib_ffi(natives: &mut crate::vm::native::NativeInterface) {
    // File functions (manual shims only)
    natives.register_shim_by_name("auto.file.walk", shim_file_walk);
    natives.register_shim_by_name("auto.file.read_lines", shim_file_read_lines);

    // Process functions (manual shims only)
    natives.register_shim_by_name("auto.process.spawn_with_output", shim_process_spawn_with_output);

    // Option functions (manual shims only)
    natives.register_shim_by_name("Option.or", shim_option_or);
    natives.register_shim_by_name("Option.unwrap_or", shim_option_or);

    // Math functions (manual shims only — polymorphic over i64/f64)
    natives.register_shim_by_name("auto.math.abs", shim_math_abs);
    natives.register_shim_by_name("auto.math.min", shim_math_min);
    natives.register_shim_by_name("auto.math.max", shim_math_max);
    // auto.math.sqrt now registered in native.rs register_std_shims() (Plan 240 VM-1)

    // Net/TCP functions (manual shims — use heap objects for TCP state)
    // Note: registry uses underscores in method portion (auto.net.tcp_bind, not auto.net.tcp.bind)
    natives.register_shim_by_name("auto.net.tcp_bind", shim_net_tcp_bind);
    natives.register_shim_by_name("auto.net.tcp_listener_accept", shim_net_tcp_listener_accept);
    natives.register_shim_by_name("auto.net.tcp_listener_close", shim_net_tcp_listener_close);
    natives.register_shim_by_name("auto.net.tcp_connect", shim_net_tcp_connect);
    natives.register_shim_by_name("auto.net.tcp_stream_read", shim_net_tcp_stream_read);
    natives.register_shim_by_name("auto.net.tcp_stream_write", shim_net_tcp_stream_write);
    natives.register_shim_by_name("auto.net.tcp_stream_read_all", shim_net_tcp_stream_read_all);
    natives.register_shim_by_name("auto.net.tcp_stream_write_str", shim_net_tcp_stream_write_str);
    natives.register_shim_by_name("auto.net.tcp_stream_close", shim_net_tcp_stream_close);
    natives.register_shim_by_name("auto.net.tcp_stream_set_read_timeout", shim_net_tcp_stream_set_read_timeout);
    natives.register_shim_by_name("auto.net.tcp_stream_set_write_timeout", shim_net_tcp_stream_set_write_timeout);

    // HTTP server functions (manual shims — heap objects for server state)
    natives.register_shim_by_name("Http.server", shim_http_server);
    natives.register_shim_by_name("Http.server_get", shim_http_server_get);
    natives.register_shim_by_name("Http.server_post", shim_http_server_post);
    natives.register_shim_by_name("Http.server_put", shim_http_server_put);
    natives.register_shim_by_name("Http.server_delete", shim_http_server_delete);
    natives.register_shim_by_name("Http.server_static", shim_http_server_static);
    natives.register_shim_by_name("Http.server_listen", shim_http_server_listen);
    natives.register_shim_by_name("Http.response", shim_http_response);
    natives.register_shim_by_name("Http.response_status", shim_http_response_status);
    natives.register_shim_by_name("Http.response_header", shim_http_response_header);
    natives.register_shim_by_name("Http.response_text", shim_http_response_text);
    natives.register_shim_by_name("Http.response_html", shim_http_response_html);
    natives.register_shim_by_name("Http.response_bytes", shim_http_response_bytes);

    // HTTP client functions (manual shims — heap objects for request/response)
    natives.register_shim_by_name("Http.get", shim_http_get);
    natives.register_shim_by_name("Http.post", shim_http_post);
    natives.register_shim_by_name("Http.put", shim_http_put);
    natives.register_shim_by_name("Http.delete", shim_http_delete);
    natives.register_shim_by_name("Http.request", shim_http_request);
    natives.register_shim_by_name("Http.request_builder_header", shim_request_builder_header);
    natives.register_shim_by_name("Http.request_builder_body", shim_request_builder_body);
    natives.register_shim_by_name("Http.request_builder_timeout", shim_request_builder_timeout);
    natives.register_shim_by_name("Http.request_builder_json", shim_request_builder_json);
    natives.register_shim_by_name("Http.request_builder_send", shim_request_builder_send);
    natives.register_shim_by_name("Response.status_code", shim_response_status_code);
    natives.register_shim_by_name("Response.header_get", shim_response_header_get);
    natives.register_shim_by_name("Response.body", shim_response_body);

    // HTTP streaming (manual shims — heap objects for stream state)
    natives.register_shim_by_name("auto.http_stream.get_stream", shim_http_get_stream);
    natives.register_shim_by_name("auto.http_stream.post_stream", shim_http_post_stream);
    natives.register_shim_by_name("auto.http_stream.stream_next", shim_http_stream_next);
    natives.register_shim_by_name("auto.http_stream.stream_is_done", shim_http_stream_is_done);
    natives.register_shim_by_name("auto.http_stream.stream_close", shim_http_stream_close);
    natives.register_shim_by_name("Http.post_stream_with_headers", shim_http_post_stream_with_headers);

    // Regex (manual shim — heap objects for compiled regex)
    natives.register_shim_by_name("auto.regex.find_all", shim_regex_find_all);

    // Task system (manual shim — VM access for event loop)
    natives.register_shim_by_name("auto.task_system.run", shim_task_system_run);

    // Plan 192: Rust stdlib dynamic dispatch (manual — uses heap objects)
    natives.register_shim_by_name("auto.rust_stdlib.dispatch", shim_rust_stdlib_dispatch);
}

// ============================================================================
// Plan 192: Rust stdlib Dynamic Dispatch Handler
// ============================================================================

/// Push a Rust stdlib object onto the VM heap and push its handle as i64.
fn push_rust_obj<T: Any + Send + Sync + 'static>(
    task: &mut AutoTask,
    vm: &AutoVM,
    type_name: &str,
    value: T,
) -> Result<(), VMError> {
    let obj = RustStdlibObject::new(type_name, value);
    let handle = vm.insert_heap_object(obj) as i32;
    task.ram.push_i32(handle);
    Ok(())
}

/// Pop a heap handle and return a reference to the RustStdlibObject.
fn pop_rust_obj(task: &mut AutoTask, vm: &AutoVM, context: &str) -> Result<u64, VMError> {
    let handle = task.ram.pop_i32() as u64;
    if vm.get_heap_object(handle).is_none() {
        return Err(VMError::RuntimeError(format!(
            "Invalid Rust stdlib handle in {} (handle={})", context, handle
        )));
    }
    Ok(handle)
}

/// Generic dispatch handler for Rust stdlib calls.
///
/// Stack layout (popped in reverse order):
///   ... user args ... | method: String | type_name: String
fn shim_rust_stdlib_dispatch(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let method: String = String::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(format!("rust_stdlib_dispatch: {}", e)))?;
    let type_name: String = String::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(format!("rust_stdlib_dispatch: {}", e)))?;

    match (type_name.as_str(), method.as_str()) {
        // std::time::Instant
        ("Instant", "now") => {
            let instant = std::time::Instant::now();
            push_rust_obj(task, vm, "Instant", instant)?;
        }

        // std::time::Duration
        ("Duration", "from_secs") => {
            let secs: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Duration.from_secs: {}", e)))?;
            push_rust_obj(task, vm, "Duration", std::time::Duration::from_secs(secs as u64))?;
        }
        ("Duration", "from_millis") => {
            let ms: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Duration.from_millis: {}", e)))?;
            push_rust_obj(task, vm, "Duration", std::time::Duration::from_millis(ms as u64))?;
        }
        ("Duration", "from_secs_f64") => {
            let secs: f32 = f32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Duration.from_secs_f64: {}", e)))?;
            push_rust_obj(task, vm, "Duration", std::time::Duration::from_secs_f64(secs as f64))?;
        }

        // std::path::PathBuf
        ("PathBuf", "from") => {
            let s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("PathBuf.from: {}", e)))?;
            push_rust_obj(task, vm, "PathBuf", StdPathBuf::from(s))?;
        }
        ("PathBuf", "join") => {
            let other: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("PathBuf.join: {}", e)))?;
            let self_handle = pop_rust_obj(task, vm, "PathBuf.join")?;
            let obj = vm.get_heap_object(self_handle).unwrap();
            let mut guard = obj.write().unwrap();
            if let Some(path) = guard.as_any_mut().downcast_mut::<StdPathBuf>() {
                path.push(&other);
                // Return same handle
                task.ram.push_i32(self_handle as i32);
            } else {
                return Err(VMError::RuntimeError(format!("PathBuf.join: invalid object at handle {}", self_handle)));
            }
        }

        // std::boxed::Box
        ("Box", "new") => {
            let val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Box.new: {}", e)))?;
            push_rust_obj(task, vm, "Box", val)?;
        }

        // std::cell::RefCell
        ("RefCell", "new") => {
            let val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("RefCell.new: {}", e)))?;
            push_rust_obj(task, vm, "RefCell", val)?;
        }

        // std::sync::Arc
        ("Arc", "new") => {
            let val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Arc.new: {}", e)))?;
            push_rust_obj(task, vm, "Arc", val)?;
        }

        // std::sync::Mutex
        ("Mutex", "new") => {
            let val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Mutex.new: {}", e)))?;
            push_rust_obj(task, vm, "Mutex", val)?;
        }

        _ => {
            return Err(VMError::RuntimeError(format!(
                "Unknown Rust stdlib call: {}.{}", type_name, method
            )));
        }
    }
    Ok(())
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
        assert!((1000..1100).contains(&NATIVE_FILE_WALK));
        assert!((1000..1100).contains(&NATIVE_FILE_APPEND_TEXT));
        assert!((1000..1100).contains(&NATIVE_FILE_READ_LINES));

        // Env: 1100-1199
        assert!((1100..1200).contains(&NATIVE_ENV_GET));

        // Time: 1200-1299
        assert!((1200..1300).contains(&NATIVE_TIME_NOW_MS));

        // Process: 1300-1399
        assert!((1300..1400).contains(&NATIVE_PROCESS_EXIT));
        assert!((1300..1400).contains(&NATIVE_PROCESS_ARGS));
        assert!((1300..1400).contains(&NATIVE_PROCESS_CURRENT_DIR));
        assert!((1300..1400).contains(&NATIVE_PROCESS_SPAWN));
        assert!((1300..1400).contains(&NATIVE_PROCESS_SPAWN_WITH_OUTPUT));

        // HTTP streaming with headers: 2255-2259
        assert!((2250..2260).contains(&NATIVE_HTTP_POST_STREAM_WITH_HEADERS));

        // Regex: 2400-2499
        assert!((2400..2500).contains(&NATIVE_REGEX_IS_MATCH));
        assert!((2400..2500).contains(&NATIVE_REGEX_FIND_ALL));

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
        let handle_id = shim_task_spawn("TestTask".to_string(), 64);
        assert!(handle_id > 0);
    }

    #[test]
    fn test_task_spawn_default_capacity() {
        // Test with negative capacity (should use default 64)
        let handle_id = shim_task_spawn("TestTask".to_string(), -1);
        assert!(handle_id > 0);
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
        let handle_id = shim_task_spawn("TestTask".to_string(), 64) as i64;

        let result = shim_task_handle_is_null(handle_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_task_handle_type() {
        let handle_id = shim_task_spawn("MyCounterTask".to_string(), 64) as i64;

        let result = shim_task_handle_type(handle_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "MyCounterTask");
    }

    #[test]
    fn test_task_handle_id() {
        let handle_id = shim_task_spawn("TestTask".to_string(), 64);

        let result = shim_task_handle_id(handle_id as i64);
        assert!(result.is_ok());
        // Instance ID should be > 0 (global counter)
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_task_send() {
        let handle_id = shim_task_spawn("TestTask".to_string(), 64);

        // shim_task_send takes (i32, i32) - handle_id and message enum tag
        let result = shim_task_send(handle_id, 1); // 1 is some message tag
        assert_eq!(result, 1); // 1 = success
    }

    #[test]
    fn test_task_send_invalid_handle() {
        // Invalid handle should return 0 (failure)
        let result = shim_task_send(999999, 1);
        assert_eq!(result, 0); // 0 = failure
    }

    #[test]
    fn test_multiple_spawn_unique_ids() {
        let id1 = shim_task_spawn("Task1".to_string(), 64);
        let id2 = shim_task_spawn("Task2".to_string(), 64);

        // Each spawn should return a unique handle ID
        assert_ne!(id1, id2);
    }

    // Plan 159: New FFI function tests

    #[test]
    fn test_file_append_text() {
        let dir = std::env::temp_dir().join("ac-ffi-test-append");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("append_test.txt");

        // First append creates the file
        shim_file_append_text(
            path.to_str().unwrap().to_string(),
            "line one\n".to_string(),
        )
        .unwrap();

        // Second append adds to existing file
        shim_file_append_text(
            path.to_str().unwrap().to_string(),
            "line two\n".to_string(),
        )
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "line one\nline two\n");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_append_creates_parent_dirs() {
        let dir = std::env::temp_dir().join("ac-ffi-test-append-nested");
        let _ = fs::remove_dir_all(&dir);

        let path = dir.join("a/b/c/test.txt");
        shim_file_append_text(
            path.to_str().unwrap().to_string(),
            "hello".to_string(),
        )
        .unwrap();

        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_regex_is_match() {
        // Match found
        let result = shim_regex_is_match("hello\\s+world".to_string(), "hello world".to_string());
        assert_eq!(result.unwrap(), 1);

        // No match
        let result = shim_regex_is_match("xyz\\d+".to_string(), "hello world".to_string());
        assert_eq!(result.unwrap(), 0);

        // Invalid regex
        let result = shim_regex_is_match("[invalid(".to_string(), "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_regex_find_all_simple() {
        let text = "hello world, hello rust";
        let re = regex::Regex::new("hello").unwrap();
        let matches: Vec<_> = re.find_iter(text).collect();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].as_str(), "hello");
        assert_eq!(matches[1].as_str(), "hello");
    }

    #[test]
    fn test_regex_find_all_with_groups() {
        let text = "foo123bar456baz";
        let re = regex::Regex::new("\\d+").unwrap();
        let matches: Vec<_> = re.find_iter(text).collect();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].as_str(), "123");
        assert_eq!(matches[1].as_str(), "456");
    }
}
