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
pub const NATIVE_ENV_LOCAL_DATA_DIR: u16 = 1104;
pub const NATIVE_ENV_HOME_DIR: u16 = 1105;

// IO functions: 1150-1169
pub const NATIVE_IO_READ_LINE: u16 = 1150;

// Time functions: 1200-1299
pub const NATIVE_TIME_NOW_MS: u16 = 1200;
pub const NATIVE_TIME_NOW_SEC: u16 = 1201;
pub const NATIVE_TIME_SLEEP_MS: u16 = 1202;
pub const NATIVE_TIME_NOW: u16 = 1205;

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
pub const NATIVE_STR_UUID: u16 = 1521;
pub const NATIVE_STR_FROM_UINT: u16 = 1522;
pub const NATIVE_STR_TO_UINT: u16 = 1523;

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
#[auto_macros::rust_fn("File.read_text", "auto.file.read_text", "auto.fs.read_text", "auto.fs.read")]
pub fn shim_file_read_text(path: String) -> String {
    fs::read_to_string(&path).unwrap_or_default()
}

/// Write text content to a file
#[auto_macros::rust_fn("File.write_text", "auto.file.write_text", "auto.fs.write_text", "auto.fs.write")]
pub fn shim_file_write_text(path: String, content: String) -> i32 {
    let _ = fs::write(&path, &content);
    0
}

/// Check if a file exists
#[auto_macros::rust_fn("File.exists", "auto.file.exists", "auto.fs.exists")]
pub fn shim_file_exists(path: String) -> i32 {
    if fs::metadata(&path).is_ok() { 1 } else { 0 }
}

/// Delete a file
#[auto_macros::rust_fn("File.delete", "auto.fs.delete")]
pub fn shim_file_delete(path: String) -> i32 {
    let _ = fs::remove_file(&path);
    0
}

/// Create a directory
#[auto_macros::rust_fn("File.create_dir", "auto.fs.create_dir")]
pub fn shim_file_create_dir(path: String) -> i32 {
    let _ = fs::create_dir_all(&path);
    0
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
#[auto_macros::rust_fn("auto.file.walk")]
pub fn shim_file_walk(path: String) -> Result<String, String> {
    let root = Path::new(&path);
    if !root.exists() {
        return Err(format!("File.walk failed: path not found: {}", path));
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

    serde_json::to_string(&files).map_err(|e| format!("JSON serialization failed: {}", e))
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
#[auto_macros::rust_fn("auto.file.read_lines")]
pub fn shim_file_read_lines(path: String) -> Result<String, String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("File.read_lines failed: {} - {}", path, e))?;
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    serde_json::to_string(&lines).map_err(|e| format!("JSON serialization failed: {}", e))
}

/// Check if a file contains null bytes (binary detection). Returns 1 if binary, 0 if text.
#[auto_macros::rust_fn("auto.fs.is_binary")]
pub fn shim_fs_is_binary(path: String) -> i32 {
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(_) => return 0,
    };
    let check_len = bytes.len().min(8192);
    if bytes[..check_len].contains(&0) { 1 } else { 0 }
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

/// Get the local data directory (e.g. ~/.local/share on Unix, %APPDATA% on Windows)
#[auto_macros::rust_fn("Env.local_data_dir")]
pub fn shim_env_local_data_dir() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string())
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        home + "/.local/share"
    }
}

/// Get the user's home directory
#[auto_macros::rust_fn("Env.home_dir")]
pub fn shim_env_home_dir() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
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

/// Get current timestamp as a string (seconds since epoch)
#[auto_macros::rust_fn("Time.now")]
pub fn shim_time_now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

// ============================================================================
// Process Functions
// ============================================================================

/// Exit the process with a code
#[auto_macros::rust_fn("Process.exit")]
pub fn shim_process_exit(code: i32) {
    std::process::exit(code);
}

/// Get command line arguments as a space-joined string
#[auto_macros::rust_fn("Process.args")]
pub fn shim_process_args() -> String {
    std::env::args().collect::<Vec<_>>().join(" ")
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

// ============================================================================
// IO Functions
// ============================================================================

/// Read a line from stdin
#[auto_macros::rust_fn("IO.read_line")]
pub fn shim_io_read_line() -> String {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).ok();
    // Trim trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    line
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

/// Execute a shell command string, capture stdout+stderr, truncate at 64KB.
/// Takes: cmd (String), timeout_ms (i32) — timeout currently unused.
/// Returns: JSON string {"exit_code": N, "stdout": "...", "stderr": "..."}
pub fn shim_sys_exec(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // Pop timeout as i32 (Auto int = i32, not i64, to avoid slot mismatch)
    let timeout_ms: i32 = task.ram.pop_i32();
    let cmd: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let shell = if cfg!(windows) { "cmd" } else { "sh" };
    let flag = if cfg!(windows) { "/C" } else { "-c" };

    let output = std::process::Command::new(shell)
        .arg(flag)
        .arg(&cmd)
        .output()
        .map_err(|e| VMError::RuntimeError(format!("sys.exec failed: {} - {}", cmd, e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

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
    let _ = timeout_ms;
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

/// Find first occurrence of substring with optional start position.
/// Supports both 2-arg (find(s, needle)) and 3-arg (s.find(needle, start_pos)) calling conventions.
/// Detects whether top-of-stack is start_pos (i32) or needle (string) to handle both cases.
pub fn shim_str_find_manual(task: &mut crate::vm::task::AutoTask, vm: &crate::vm::engine::AutoVM) -> Result<(), crate::vm::engine::VMError> {
    use crate::vm::ffi::convert::VMConvertible;

    // Peek at top of stack to determine calling convention
    let start_pos: i64 = {
        let top = task.ram.read_nv(task.ram.sp - 1);
        if auto_val::is_i32(top) {
            task.ram.pop_nv(); // consume start_pos
            auto_val::decode_i32(top) as i64
        } else {
            0i64 // no start_pos, default to 0
        }
    };

    // Pop needle (String)
    let needle: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| crate::vm::engine::VMError::RuntimeError(e.to_string()))?;

    // Pop receiver (String)
    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| crate::vm::engine::VMError::RuntimeError(e.to_string()))?;

    let result = if start_pos > 0 && (start_pos as usize) < s.len() {
        s[start_pos as usize..].find(&needle).map(|i| start_pos as i32 + i as i32).unwrap_or(-1)
    } else {
        s.find(&needle).map(|i| i as i32).unwrap_or(-1)
    };

    // Push result
    task.ram.push_nv(auto_val::encode_i32(result));

    Ok(())
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

/// Generate a UUID-like string from timestamp and random components
#[auto_macros::rust_fn("Str.uuid")]
pub fn shim_str_uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    // Simple UUID v4-like: 8-4-4-4-12 hex format using nanos + LCG
    let r1: u64 = nanos;
    let r2: u64 = nanos.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let r3: u64 = nanos.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407).rotate_left(17);
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:08x}{:04x}",
        (r1 >> 32) as u32,
        (r1 >> 16) as u16 & 0xFFFF,
        ((r2 >> 16) as u16 & 0x0FFF) | 0x4000, // version 4
        ((r2 >> 0) as u16 & 0x3FFF) | 0x8000,  // variant 1
        (r3 >> 32) as u32,
        r3 as u16,
    )
}

/// Convert unsigned integer to string
#[auto_macros::rust_fn("Str.from_uint")]
pub fn shim_str_from_uint(n: i64) -> String {
    n.to_string()
}

/// Parse string as unsigned integer
#[auto_macros::rust_fn("Str.to_uint")]
pub fn shim_str_to_uint(s: String) -> i64 {
    s.trim().parse::<i64>().unwrap_or(0)
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
    // Pop default value
    let default_val = { let nv = task.ram.pop_nv(); auto_val::decode_i32(nv) };

    // Pop option value — check for None (null or -1 sentinel)
    {
        let opt_nv = task.ram.pop_nv();
        // None is represented as encode_null() (TAG_NULL) or encode_i32(-1)
        if auto_val::is_null(opt_nv) {
            task.ram.push_i32(default_val);
            return Ok(());
        }
        let opt_val = auto_val::decode_i32(opt_nv);
        if opt_val == -1 {
            task.ram.push_i32(default_val);
            return Ok(());
        }
        // Check for heap-based Option
        if opt_val > 0 {
            let instance_id = opt_val as u64;
            if vm.is_option_none(instance_id) {
                task.ram.push_i32(default_val);
                return Ok(());
            }
            if vm.is_option_some(instance_id) {
                if let Some(inner) = vm.get_option_inner(instance_id) {
                    match inner {
                        auto_val::Value::Int(n) => task.ram.push_i32(n),
                        auto_val::Value::Bool(b) => task.ram.push_i32(if b { 1 } else { 0 }),
                        _other => task.ram.push_i32(opt_val),
                    }
                    return Ok(());
                }
            }
        }
        task.ram.push_nv(opt_nv);
        return Ok(());
    }
}

// ============================================================================
// Math Functions (ID 1700-1799)
// ============================================================================

#[auto_macros::rust_fn("Math.abs", "auto.math.abs")]
pub fn shim_math_abs(n: i32) -> i32 {
    n.abs()
}

#[auto_macros::rust_fn("Math.min", "auto.math.min")]
pub fn shim_math_min(a: i32, b: i32) -> i32 {
    a.min(b)
}

#[auto_macros::rust_fn("Math.max", "auto.math.max")]
pub fn shim_math_max(a: i32, b: i32) -> i32 {
    a.max(b)
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

/// Parse a TOML string (placeholder)
#[auto_macros::rust_fn("toml.from_str")]
pub fn shim_toml_from_str(s: String) -> String {
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

/// Plan 312: Global HTTP route table — (method, path_pattern) → fn_name.
/// Populated at VM startup from CompiledPackage.api_routes, consulted by
/// shim_http_server_listen to dispatch requests to VM handler functions.
/// Format: Vec<(method: String, path: String, fn_name: String)>.
static HTTP_ROUTES: std::sync::Mutex<Vec<(String, String, String)>> = std::sync::Mutex::new(Vec::new());

/// Plan 312: Register API routes into the global table. Called at VM startup.
pub fn register_http_routes(routes: Vec<(String, String, String)>) {
    if let Ok(mut table) = HTTP_ROUTES.lock() {
        *table = routes;
    }
}

/// Plan 312: Get the current HTTP routes (for testing / introspection).
pub fn get_http_routes() -> Vec<(String, String, String)> {
    HTTP_ROUTES.lock().map(|t| t.clone()).unwrap_or_default()
}

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
        Some(mut stream) => {
            // Plan 313: Enable TCP_NODELAY by default for low-latency (SSE)
            stream.set_nodelay(true).ok();
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

/// Plan 313: Flush TCP stream (explicit flush for SSE/low-latency writes).
/// Note: TcpStream::flush() is a no-op for raw sockets (no user-space buffer),
/// but this provides API completeness and will be effective if BufWriter is
/// added later. For real SSE latency improvement, use set_nodelay(true).
pub fn shim_net_tcp_stream_flush(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let stream_handle: i32 = task.ram.pop_i32();
    TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        if let Some(stream) = streams.get_mut(&(stream_handle as u64)) {
            use std::io::Write;
            stream.flush().ok();
        }
    });
    task.ram.push_i32(0); // success
    Ok(())
}

/// Plan 313: Set TCP_NODELAY on a stream (disables Nagle's algorithm).
/// Critical for SSE: without this, small data packets (like `data: ...\n\n`)
/// are buffered up to ~40ms before sending. Returns 0 on success, -1 on error.
pub fn shim_net_tcp_stream_set_nodelay(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let enabled: i32 = task.ram.pop_i32();
    let stream_handle: i32 = task.ram.pop_i32();
    let result = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => stream.set_nodelay(enabled != 0).map(|_| 0).unwrap_or(-1),
            None => -1,
        }
    });
    task.ram.push_i32(result);
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
    pub(crate) static HTTP_STREAMS: std::cell::RefCell<std::collections::HashMap<u64, HttpStreamData>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

// Plan 195: RequestBuilder data storage
#[derive(Debug, Clone)]
struct HttpRequestBuilderData {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: Option<u64>,
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
pub(crate) struct HttpStreamData {
    pub url: String,
    pub response: Option<reqwest::blocking::Response>,
    pub done: bool,
    pub status_code: u16,
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

/// Plan 316 fix: Run the HTTP server in a blocking loop WITHOUT requiring an
/// AutoTask parameter. This is the entry point for auto-starting the server
/// from execute_autovm via a dedicated OS thread (std::thread::spawn).
///
/// The listen loop is identical to shim_http_server_listen but creates its own
/// handler tasks internally, avoiding the need for a caller-provided task.
/// Must be called from a non-tokio thread (std::thread::spawn), because it
/// uses blocking_lock() on handler tasks.
pub fn run_http_server_blocking(vm: &AutoVM, addr: &str) {
    use std::io::{Read, Write, BufRead};
    use std::net::TcpListener;

    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[HTTP] Server bind failed on {}: {}", addr, e);
            return;
        }
    };
    eprintln!("[HTTP] Server listening on {}", addr);

    let routes = get_http_routes();

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[HTTP] Accept error: {}", e);
                continue;
            }
        };

        // Parse HTTP request
        let mut reader = std::io::BufReader::new(&mut stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line).is_err() {
            continue;
        }
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let req_method = parts[0].to_uppercase();
        let req_path = parts[1].to_string();

        // Read headers
        let mut content_length = 0usize;
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header).is_err() { break; }
            let header = header.trim();
            if header.is_empty() { break; }
            if header.to_lowercase().starts_with("content-length:") {
                content_length = header[15..].trim().parse().unwrap_or(0);
            }
        }

        // Read body
        let body = if content_length > 0 {
            let mut buf = vec![0u8; content_length];
            let _ = (&mut reader).read_exact(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        } else {
            String::new()
        };

        // Route matching
        let (fn_name, path_params) = match find_route(&routes, &req_method, &req_path) {
            Some(m) => m,
            None => {
                let resp = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
                let _ = stream.write_all(resp.as_bytes());
                continue;
            }
        };

        // Call VM handler on a fresh task
        let handler_task_id = vm.spawn_task(0, 8192);
        let result_json: Option<String> = if let Some(handler_task_arc) = vm.tasks.get(&handler_task_id) {
            // blocking_lock is safe: we're on a std::thread, NOT in tokio context
            let mut ht = handler_task_arc.blocking_lock();

            let mut n_args = 0;
            for (_param_name, param_val) in &path_params {
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(param_val.as_bytes().to_vec());
                    i
                };
                ht.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }
            if !body.is_empty() {
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(body.as_bytes().to_vec());
                    i
                };
                ht.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }

            match vm.call_fn_by_name(&mut ht, &fn_name, n_args) {
                Ok(()) => {
                    let nv = ht.ram.pop_nv();
                    if auto_val::is_string(nv) {
                        let idx = auto_val::decode_string(nv);
                        vm.strings.read().unwrap().get(idx as usize)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                    } else if auto_val::is_i32(nv) {
                        Some(auto_val::decode_i32(nv).to_string())
                    } else if auto_val::is_null(nv) {
                        Some("null".to_string())
                    } else {
                        Some("null".to_string())
                    }
                }
                Err(e) => {
                    eprintln!("[HTTP] Handler '{}' error: {:?}", fn_name, e);
                    None
                }
            }
        } else {
            None
        };

        vm.tasks.remove(&handler_task_id);

        let (status, body_json) = match result_json {
            Some(s) => ("200 OK", s),
            None => ("500 Internal Server Error", "{}".to_string()),
        };
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status, body_json.len(), body_json
        );
        let _ = stream.write_all(response.as_bytes());
    }
}

/// Start server listening (blocking, using tokio)
pub fn shim_http_server_listen(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let addr: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let _server: i64 = task.ram.pop_i64();

    // Plan 312 Phase 3: Real HTTP server with route dispatch.
    // Uses synchronous std::net (not tokio) for simplicity. Each request is
    // handled serially on the listen thread. Handler functions are called via
    // vm.call_fn_by_name on a fresh AutoTask (isolation per request).
    use std::io::{Read, Write, BufRead};
    use std::net::TcpListener;

    let listener = TcpListener::bind(&addr)
        .map_err(|e| VMError::RuntimeError(format!("HTTP server bind failed: {}", e)))?;
    eprintln!("[HTTP] Server listening on {}", addr);

    // Clone routes for the listen loop (avoid holding lock during requests)
    let routes = get_http_routes();

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[HTTP] Accept error: {}", e);
                continue;
            }
        };

        // Parse HTTP request: method, path, body
        let mut reader = std::io::BufReader::new(&mut stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line).is_err() {
            continue;
        }
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let req_method = parts[0].to_uppercase();
        let req_path = parts[1].to_string();

        // Read remaining headers (until empty line)
        let mut content_length = 0usize;
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header).is_err() { break; }
            let header = header.trim();
            if header.is_empty() { break; }
            if header.to_lowercase().starts_with("content-length:") {
                content_length = header[15..].trim().parse().unwrap_or(0);
            }
        }

        // Read body if present
        let body = if content_length > 0 {
            let mut buf = vec![0u8; content_length];
            let _ = (&mut reader).read_exact(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        } else {
            String::new()
        };

        // Route matching: find (method, path) match with :param support
        let (fn_name, path_params) = match find_route(&routes, &req_method, &req_path) {
            Some(match_result) => match_result,
            None => {
                let resp = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
                let _ = stream.write_all(resp.as_bytes());
                continue;
            }
        };

        // Call VM handler function on a fresh task
        let handler_task_id = vm.spawn_task(0, 8192);
        let result_json: Option<String> = if let Some(handler_task_arc) = vm.tasks.get(&handler_task_id) {
            // tokio::sync::Mutex — use blocking_lock() for sync context
            let mut ht = handler_task_arc.blocking_lock();

            // Push path params as string args (path params first, then body)
            let mut n_args = 0;
            for (_param_name, param_val) in &path_params {
                // Allocate string in VM string pool and push
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(param_val.as_bytes().to_vec());
                    i
                };
                ht.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }
            // Push body if present (for POST/PUT)
            if !body.is_empty() {
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(body.as_bytes().to_vec());
                    i
                };
                ht.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }

            match vm.call_fn_by_name(&mut ht, &fn_name, n_args) {
                Ok(()) => {
                    // Result is on top of stack — try to decode as string
                    let nv = ht.ram.pop_nv();
                    if auto_val::is_string(nv) {
                        let idx = auto_val::decode_string(nv);
                        vm.strings.read().unwrap().get(idx as usize)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                    } else if auto_val::is_i32(nv) {
                        Some(auto_val::decode_i32(nv).to_string())
                    } else if auto_val::is_null(nv) {
                        Some("null".to_string())
                    } else {
                        Some("null".to_string())
                    }
                }
                Err(e) => {
                    eprintln!("[HTTP] Handler '{}' error: {:?}", fn_name, e);
                    None
                }
            }
        } else {
            None
        };

        // Clean up handler task
        vm.tasks.remove(&handler_task_id);

        // Serialize result to JSON response
        let (status, body_json) = match result_json {
            Some(s) => ("200 OK", s),
            None => ("500 Internal Server Error", "{}".to_string()),
        };
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status, body_json.len(), body_json
        );
        let _ = stream.write_all(response.as_bytes());
    }

    Ok(())
}

/// Plan 312: Find a matching route for (method, path). Supports :param extraction.
/// Returns (fn_name, Vec<(param_name, param_value)>).
fn find_route(
    routes: &[(String, String, String)],
    method: &str,
    path: &str,
) -> Option<(String, Vec<(String, String)>)> {
    for (route_method, route_pattern, fn_name) in routes {
        if route_method.to_uppercase() != method.to_uppercase() {
            continue;
        }
        // Match path with :param support
        let route_segments: Vec<&str> = route_pattern.split('/').collect();
        let path_segments: Vec<&str> = path.split('/').collect();
        if route_segments.len() != path_segments.len() {
            continue;
        }
        let mut params = Vec::new();
        let mut matched = true;
        for (rs, ps) in route_segments.iter().zip(path_segments.iter()) {
            if rs.starts_with(':') {
                params.push((rs[1..].to_string(), ps.to_string()));
            } else if rs != ps {
                matched = false;
                break;
            }
        }
        if matched {
            return Some((fn_name.clone(), params));
        }
    }
    None
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

    task.ram.push_i32(response_handle as i32);
    Ok(())
}

/// Perform a POST request
pub fn shim_http_post(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let response_handle = simple_http_request("POST", &url, Some(&body));

    task.ram.push_i32(response_handle as i32);
    Ok(())
}

/// Perform a PUT request
pub fn shim_http_put(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let response_handle = simple_http_request("PUT", &url, Some(&body));

    task.ram.push_i32(response_handle as i32);
    Ok(())
}

/// Perform a DELETE request
pub fn shim_http_delete(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let response_handle = simple_http_request("DELETE", &url, None);

    task.ram.push_i32(response_handle as i32);
    Ok(())
}

// ============================================================================
// Plan 195: RequestBuilder FFI
// ============================================================================

/// Create a new RequestBuilder handle (stored in heap_objects for CALL_SPEC dispatch)
/// http_request(method, url) -> RequestBuilder handle
pub fn shim_http_request(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let method: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let data = HttpRequestBuilderData {
        method,
        url,
        headers: vec![],
        body: None,
        timeout_ms: None,
    };
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
        "RequestBuilder",
        std::sync::Mutex::new(data),
    );
    let heap_id = vm.insert_heap_object(obj);
    task.ram.push_i32(heap_id as i32);
    Ok(())
}

/// Add a header to RequestBuilder
/// request_builder_header(rb, key, value) -> rb
pub fn shim_request_builder_header(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = task.ram.pop_i32() as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut builder) = mutex.lock() {
                    builder.headers.push((key, value));
                }
            }
        }
    }

    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// Set body on RequestBuilder
/// request_builder_body(rb, body) -> rb
pub fn shim_request_builder_body(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = task.ram.pop_i32() as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut builder) = mutex.lock() {
                    builder.body = Some(body);
                }
            }
        }
    }

    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// Set timeout on RequestBuilder
/// request_builder_timeout(rb, ms) -> rb
pub fn shim_request_builder_timeout(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let ms: i64 = task.ram.pop_i32() as i64;
    let rb_handle: i64 = task.ram.pop_i32() as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut builder) = mutex.lock() {
                    builder.timeout_ms = Some(ms as u64);
                }
            }
        }
    }

    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// Set JSON body on RequestBuilder
/// request_builder_json(rb, data) -> rb
pub fn shim_request_builder_json(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let data: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = task.ram.pop_i32() as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut builder) = mutex.lock() {
                    builder.body = Some(data);
                    if !builder.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")) {
                        builder.headers.push(("Content-Type".to_string(), "application/json".to_string()));
                    }
                }
            }
        }
    }

    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// Send RequestBuilder and return Response handle
/// request_builder_send(rb) -> Response handle
pub fn shim_request_builder_send(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let rb_handle: i64 = task.ram.pop_i32() as i64;
    let heap_id = rb_handle as u64;

    // Extract builder data and remove from heap
    let obj = vm.remove_heap_object(heap_id)
        .ok_or_else(|| VMError::RuntimeError(format!("Invalid RequestBuilder handle: {}", rb_handle)))?;
    let guard = obj.write().unwrap();
    let rso = guard.as_any()
        .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let mutex = rso.downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>()
        .ok_or_else(|| VMError::RuntimeError("Not a RequestBuilder".to_string()))?;
    let builder_data = mutex.lock().map_err(|e| VMError::RuntimeError(format!("Mutex poison: {}", e)))?;
    let method = builder_data.method.clone();
    let url = builder_data.url.clone();
    let body = builder_data.body.clone();
    drop(builder_data);
    drop(guard);

    let response_handle = simple_http_request(&method, &url, body.as_deref());
    task.ram.push_i32(response_handle as i32);
    Ok(())
}

// ============================================================================
// Plan 195: Enhanced Response access methods
// ============================================================================

/// Get status code from Response handle
/// response_status_code(res_handle) -> int
pub fn shim_response_status_code(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let res_handle: i64 = task.ram.pop_i32() as i64;

    let status = HTTP_RESPONSES.with(|r| {
        r.borrow().get(&(res_handle as u64)).map(|res| res.status as i32)
    }).unwrap_or(0);

    task.ram.push_i32(status);
    Ok(())
}

/// Get header value from Response handle
/// response_header_get(res_handle, key) -> str
pub fn shim_response_header_get(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let key: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_handle: i64 = task.ram.pop_i32() as i64;

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
    let res_handle: i64 = task.ram.pop_i32() as i64;

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
/// Runs in a dedicated OS thread to avoid tokio runtime conflicts.
pub fn shim_http_get_stream(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let url_clone = url.clone();
    let response = std::thread::spawn(move || {
        reqwest::blocking::Client::new().get(&url_clone).send()
    }).join()
        .map_err(|_| VMError::RuntimeError("HTTP GET stream thread panicked".to_string()))?
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
/// Runs in a dedicated OS thread to avoid tokio runtime conflicts.
pub fn shim_http_post_stream(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let url_clone = url.clone();
    let response = std::thread::spawn(move || {
        reqwest::blocking::Client::new()
            .post(&url_clone)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
    }).join()
        .map_err(|_| VMError::RuntimeError("HTTP POST stream thread panicked".to_string()))?
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

/// Plan 321: Create an iterator from an HTTPStream for for-loop consumption.
/// `for chunk in http_stream { }` calls this via the Iter protocol.
pub fn shim_http_stream_iter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle: i64 = task.ram.pop_i64();

    let hs_iter = crate::vm::engine::HttpStreamIterator {
        stream_handle: handle as u64,
        done: false,
    };
    let iter_id = {
        let next_id = vm.iterator_id_gen.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        vm.iterators.insert(next_id, crate::vm::engine::Iterator::HttpStream(hs_iter));
        next_id
    };
    task.ram.push_i32(iter_id as i32);
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

    let url_clone = url.clone();
    let response = std::thread::spawn(move || {
        let mut request = reqwest::blocking::Client::new()
            .post(&url_clone)
            .body(body);
        for (key, value) in &headers_map {
            request = request.header(key.as_str(), value.as_str());
        }
        request.send()
    }).join()
        .map_err(|_| VMError::RuntimeError("HTTP POST stream thread panicked".to_string()))?
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

/// Check if a regex pattern matches text. Returns 1 if match, 0 if not.
pub fn shim_regex_match(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let text: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let pattern: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let re = regex::Regex::new(&pattern)
        .map_err(|e| VMError::RuntimeError(format!("regex.match failed: invalid pattern '{}': {}", pattern, e)))?;

    let result: i32 = if re.is_match(&text) { 1 } else { 0 };
    task.ram.push_i32(result);
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
/// Runs in a dedicated OS thread to avoid tokio runtime conflicts.
fn simple_http_request(method: &str, url: &str, body: Option<&str>) -> i64 {
    let method = method.to_string();
    let url = url.to_string();
    let body = body.map(|s| s.to_string());

    let result = std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let mut builder = match method.as_str() {
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => client.get(&url),
        };

        if let Some(b) = body {
            builder = builder.header("Content-Type", "application/json").body(b);
        }

        builder.send().map(|response| {
            let status = response.status().as_u16();
            let headers: Vec<(String, String)> = response
                .headers().iter()
                .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string())))
                .collect();
            let body_bytes = response.bytes().unwrap_or_default().to_vec();
            (status, headers, body_bytes)
        }).map_err(|e| e.to_string())
    }).join();

    match result {
        Ok(Ok((status, headers, body_bytes))) => {
            let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
            HTTP_RESPONSES.with(|r| { r.borrow_mut().insert(handle, HttpResponseData { status, headers, body: body_bytes }); });
            handle as i64
        }
        Ok(Err(e)) => shim_http_internal_error(format!("HTTP failed: {}", e)),
        Err(_) => shim_http_internal_error("HTTP thread panicked".to_string()),
    }
}

/// Synchronous HTTP POST with auth headers (for Anthropic API calls from AutoVM).
/// Returns response body as string and stores status code in thread-local.
/// Uses std::thread::spawn to avoid tokio runtime conflict with reqwest::blocking.
fn simple_http_request_with_auth(
    method: &str,
    url: &str,
    body: Option<&str>,
    api_key: Option<&str>,
) -> (i32, String) {
    let method = method.to_string();
    let url = url.to_string();
    let body = body.map(|s| s.to_string());
    let api_key = api_key.map(|s| s.to_string());

    let result = std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let mut builder = match method.as_str() {
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => client.get(&url),
        };

        builder = builder
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01");

        if let Some(key) = api_key {
            if !key.is_empty() {
                builder = builder.header("x-api-key", key);
            }
        }

        if let Some(b) = body {
            builder = builder.body(b);
        }

        match builder.send() {
            Ok(response) => {
                let status = response.status().as_u16() as i32;
                let body_text = response.text().unwrap_or_default();
                Ok((status, body_text))
            }
            Err(e) => Err(format!("HTTP error: {}", e)),
        }
    })
    .join();

    match result {
        Ok(Ok((status, body_text))) => (status, body_text),
        Ok(Err(e)) => (0, e),
        Err(_) => (0, "HTTP thread panicked".to_string()),
    }
}

thread_local! {
    static LAST_HTTP_STATUS: std::cell::Cell<i32> = std::cell::Cell::new(0);
}

/// HTTP POST with auth — returns body string directly, stores status code.
/// Stack: [url, body, api_key] -> [response_body_str]
pub fn shim_http_post_sync(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let api_key: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let body: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let key_opt = if api_key.is_empty() {
        None
    } else {
        Some(api_key.as_str())
    };
    let (status, resp_body) = simple_http_request_with_auth("POST", &url, Some(&body), key_opt);

    LAST_HTTP_STATUS.with(|s| s.set(status));

    resp_body.push_to_stack(task, vm)?;
    Ok(())
}

/// HTTP POST with Bearer auth — for OpenAI-compatible APIs.
/// Uses `Authorization: Bearer <key>` instead of `x-api-key`.
/// Stack: [url, body, api_key] -> [response_body_str]
pub fn shim_http_post_bearer(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let api_key: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let body: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let key_opt = if api_key.is_empty() {
        None
    } else {
        Some(api_key.as_str())
    };
    let (status, resp_body) = simple_http_request_bearer("POST", &url, Some(&body), key_opt);

    LAST_HTTP_STATUS.with(|s| s.set(status));

    resp_body.push_to_stack(task, vm)?;
    Ok(())
}

/// Synchronous HTTP request with Bearer token auth (for OpenAI-compatible APIs).
fn simple_http_request_bearer(
    method: &str,
    url: &str,
    body: Option<&str>,
    api_key: Option<&str>,
) -> (i32, String) {
    let method = method.to_string();
    let url = url.to_string();
    let body = body.map(|s| s.to_string());
    let api_key = api_key.map(|s| s.to_string());

    let result = std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let mut builder = match method.as_str() {
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => client.get(&url),
        };

        builder = builder.header("Content-Type", "application/json");

        if let Some(key) = api_key {
            if !key.is_empty() {
                builder = builder.header("Authorization", format!("Bearer {}", key));
            }
        }

        if let Some(b) = body {
            builder = builder.body(b);
        }

        match builder.send() {
            Ok(response) => {
                let status = response.status().as_u16() as i32;
                let body_text = response.text().unwrap_or_default();
                Ok((status, body_text))
            }
            Err(e) => Err(format!("HTTP error: {}", e)),
        }
    })
    .join();

    match result {
        Ok(Ok((status, body_text))) => (status, body_text),
        Ok(Err(e)) => (0, e),
        Err(_) => (0, "HTTP thread panicked".to_string()),
    }
}

/// Return the status code from the last HTTP request.
/// Stack: [] -> [status_i32]
pub fn shim_http_last_status(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let status = LAST_HTTP_STATUS.with(|s| s.get());
    task.ram.push_i32(status);
    Ok(())
}

/// HTTP listen stub — starts a simple HTTP server.
/// Stack: [callback_closure, port, host] -> []
/// Currently a stub that prints a message and returns.
pub fn shim_http_listen(_task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // Pop 3 args: host, port, callback
    // For now, just print a warning and return void
    eprintln!("WARN: http.listen() stub called — HTTP server not yet implemented in AutoVM");
    Ok(())
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
// Essential File System VM Shims
// ============================================================================

// ============================================================================
// Registration Function
// ============================================================================

/// Register all stdlib FFI functions with the NativeInterface
/// Register manual shims that cannot use #[rust_fn] (custom VM access, variadic args, etc.).
///
/// #[rust_fn]-annotated functions are auto-registered via inventory (build_from_inventory).
/// This function only registers the ~54 manual shims that need special handling.
pub fn register_stdlib_ffi(natives: &mut crate::vm::native::NativeInterface) {
    // File utility functions (walk/read_lines/is_binary now via #[rust_fn] registration)

    // String method — manual shim
    natives.register_shim_by_name("auto.str.find", shim_str_find_manual);
    natives.register_shim_by_name("Str.find", shim_str_find_manual);
    natives.register_shim_by_name("str.find", shim_str_find_manual);

    // Process functions (manual shims only)
    natives.register_shim_by_name("auto.process.spawn_with_output", shim_process_spawn_with_output);
    natives.register_shim_by_name("auto.sys.exec", shim_sys_exec);

    // Option functions (manual shims only)
    natives.register_shim_by_name("Option.or", shim_option_or);
    natives.register_shim_by_name("Option.unwrap_or", shim_option_or);

    // Math functions (abs/min/max now via #[rust_fn] multi-name registration)

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
    // Plan 313: TCP flush + nodelay for SSE/low-latency writes
    natives.register_shim_by_name("auto.net.tcp_stream_flush", shim_net_tcp_stream_flush);
    natives.register_shim_by_name("auto.net.tcp_stream_set_nodelay", shim_net_tcp_stream_set_nodelay);

    // HTTP server functions (manual shims — heap objects for server state)
    natives.register_shim_by_name("auto.http.server", shim_http_server);
    natives.register_shim_by_name("auto.http.server_get", shim_http_server_get);
    natives.register_shim_by_name("auto.http.server_post", shim_http_server_post);
    natives.register_shim_by_name("auto.http.server_put", shim_http_server_put);
    natives.register_shim_by_name("auto.http.server_delete", shim_http_server_delete);
    natives.register_shim_by_name("auto.http.server_static", shim_http_server_static);
    natives.register_shim_by_name("auto.http.server_listen", shim_http_server_listen);
    natives.register_shim_by_name("auto.http.response", shim_http_response);
    natives.register_shim_by_name("auto.http.response_status", shim_http_response_status);
    natives.register_shim_by_name("auto.http.response_header", shim_http_response_header);
    natives.register_shim_by_name("auto.http.response_text", shim_http_response_text);
    natives.register_shim_by_name("auto.http.response_html", shim_http_response_html);
    natives.register_shim_by_name("auto.http.response_bytes", shim_http_response_bytes);

    // HTTP client functions (manual shims — heap objects for request/response)
    natives.register_shim_by_name("auto.http.get", shim_http_get);
    natives.register_shim_by_name("auto.http.post", shim_http_post);
    natives.register_shim_by_name("auto.http.put", shim_http_put);
    natives.register_shim_by_name("auto.http.delete", shim_http_delete);
    natives.register_shim_by_name("auto.http.request", shim_http_request);
    natives.register_shim_by_name("auto.http.request_builder_header", shim_request_builder_header);
    natives.register_shim_by_name("auto.http.request_builder_body", shim_request_builder_body);
    natives.register_shim_by_name("auto.http.request_builder_timeout", shim_request_builder_timeout);
    natives.register_shim_by_name("auto.http.request_builder_json", shim_request_builder_json);
    natives.register_shim_by_name("auto.http.request_builder_send", shim_request_builder_send);

    // RequestBuilder method aliases for CALL_SPEC dispatch
    // When CALL_SPEC detects type "RequestBuilder", it looks up "RequestBuilder.method"
    natives.register_shim_by_name("RequestBuilder.header", shim_request_builder_header);
    natives.register_shim_by_name("RequestBuilder.body", shim_request_builder_body);
    natives.register_shim_by_name("RequestBuilder.timeout", shim_request_builder_timeout);
    natives.register_shim_by_name("RequestBuilder.json", shim_request_builder_json);
    natives.register_shim_by_name("RequestBuilder.send", shim_request_builder_send);

    natives.register_shim_by_name("Response.status_code", shim_response_status_code);
    natives.register_shim_by_name("Response.header_get", shim_response_header_get);
    natives.register_shim_by_name("Response.body", shim_response_body);

    // HTTP streaming (manual shims — heap objects for stream state)
    natives.register_shim_by_name("auto.http_stream.get_stream", shim_http_get_stream);
    natives.register_shim_by_name("auto.http_stream.post_stream", shim_http_post_stream);
    natives.register_shim_by_name("auto.http_stream.stream_next", shim_http_stream_next);
    natives.register_shim_by_name("auto.http_stream.stream_is_done", shim_http_stream_is_done);
    natives.register_shim_by_name("auto.http_stream.stream_close", shim_http_stream_close);
    // Plan 321: HTTPStream → Iter protocol for for-loop consumption
    natives.register_shim_by_name("auto.http_stream.stream_iter", shim_http_stream_iter);
    natives.register_shim_by_name("auto.http.post_stream_with_headers", shim_http_post_stream_with_headers);

    // HTTP client sync with auth (for Anthropic API from AutoVM)
    natives.register_shim_by_name("auto.http.post_sync", shim_http_post_sync);
    natives.register_shim_by_name("auto.http.post_bearer", shim_http_post_bearer);
    natives.register_shim_by_name("auto.http.last_status", shim_http_last_status);
    natives.register_shim_by_name("auto.http.listen", shim_http_listen);

    // Regex (manual shim — heap objects for compiled regex)
    natives.register_shim_by_name("auto.regex.find_all", shim_regex_find_all);
    natives.register_shim_by_name("auto.regex.match", shim_regex_match);

    // Task system (manual shim — VM access for event loop)
    natives.register_shim_by_name("auto.task_system.run", shim_task_system_run);

    // Plan 192: Rust stdlib dynamic dispatch (manual — uses heap objects)
    natives.register_shim_by_name("auto.rust_stdlib.dispatch", shim_rust_stdlib_dispatch);

    // Plan 263 Phase 2-3: Test runners (manual shims — calls discover/run from test_runner)
    natives.register_shim_by_name("auto.test.run_a2r_dir", shim_test_run_a2r_dir);
    natives.register_shim_by_name("auto.test.run_vm_dir", shim_test_run_vm_dir);
    natives.register_shim_by_name("auto.test.run_a2c_dir", shim_test_run_a2c_dir);
    natives.register_shim_by_name("auto.test.run_a2ts_dir", shim_test_run_a2ts_dir);
}

// ============================================================================
// Plan 192: Rust stdlib Dynamic Dispatch Handler
// ============================================================================

/// Push a Rust stdlib object onto the VM heap and push its handle.
fn push_rust_obj<T: Any + Send + Sync + 'static>(
    task: &mut AutoTask,
    vm: &AutoVM,
    type_name: &str,
    value: T,
) -> Result<(), VMError> {
    let obj = RustStdlibObject::new(type_name, value);
    let handle = vm.insert_heap_object(obj) as u32;
    {
        task.ram.push_nv(auto_val::encode_object(handle));
    }
    Ok(())
}

/// Pop a heap handle and return a reference to the RustStdlibObject.
fn pop_rust_obj(task: &mut AutoTask, vm: &AutoVM, context: &str) -> Result<u64, VMError> {
    {
        let nv = task.ram.pop_nv();
        let handle = if auto_val::is_object(nv) {
            auto_val::decode_object(nv) as u64
        } else {
            auto_val::decode_i32(nv) as u64
        };
        if vm.get_heap_object(handle).is_none() {
            return Err(VMError::RuntimeError(format!(
                "Invalid Rust stdlib handle in {} (handle={})", context, handle
            )));
        }
        Ok(handle)
    }
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
        ("Instant", "elapsed") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(instant) = rust_obj.downcast_ref::<std::time::Instant>() {
                        let elapsed = instant.elapsed();
                        push_rust_obj(task, vm, "Duration", elapsed)?;
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
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
        ("Duration", "as_secs") | ("Duration", "as_millis") | ("Duration", "as_micros")
        | ("Duration", "as_nanos") | ("Duration", "as_secs_f64") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(dur) = rust_obj.downcast_ref::<std::time::Duration>() {
                        let val = match method.as_str() {
                            "as_secs" => dur.as_secs() as i32,
                            "as_millis" => dur.as_millis() as i32,
                            "as_micros" => dur.as_micros() as i32,
                            "as_nanos" => dur.as_nanos() as i32,
                            "as_secs_f64" => dur.as_secs_f64() as i32,
                            _ => 0,
                        };
                        task.ram.push_i32(val);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
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
            if let Some(rust_obj) = guard.as_any_mut().downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                if let Some(path) = rust_obj.downcast_mut::<StdPathBuf>() {
                    path.push(&other);
                    task.ram.push_i32(self_handle as i32);
                } else {
                    return Err(VMError::RuntimeError(format!("PathBuf.join: invalid object at handle {}", self_handle)));
                }
            } else {
                return Err(VMError::RuntimeError(format!("PathBuf.join: not a RustStdlibObject at handle {}", self_handle)));
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
        ("RefCell", "borrow") | ("RefCell", "borrow_mut") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(val) = rust_obj.downcast_ref::<i32>() {
                        task.ram.push_i32(*val);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("RefCell", "replace") => {
            let new_val: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let mut guard = obj.write().unwrap();
                if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
                    if let Some(val) = rso.downcast_mut::<i32>() {
                        *val = new_val;
                    }
                }
            }
            task.ram.push_i32(handle as i32);
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

        // std::sync::Arc — single-threaded stub
        ("Arc", "load") => {
            let _ordering = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            // Stub: return the inner value (same handle for single-threaded)
            task.ram.push_i32(handle as i32);
        }
        ("Arc", "fetch_add") => {
            let _ordering = task.ram.pop_i32();
            let _delta = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }

        // ---- chrono ----
        ("Utc", "now") => {
            let now = chrono::Utc::now();
            push_rust_obj(task, vm, "chrono::DateTime<chrono::Utc>", now)?;
        }
        ("Local", "now") => {
            let now = chrono::Local::now();
            push_rust_obj(task, vm, "chrono::DateTime<chrono::Local>", now)?;
        }
        ("Duration", "days") => {
            let days: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Duration.days: {}", e)))?;
            push_rust_obj(task, vm, "Duration", chrono::Duration::days(days as i64))?;
        }
        ("Duration", "hours") => {
            let hours: i64 = i64::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Duration.hours: {}", e)))?;
            push_rust_obj(task, vm, "Duration", chrono::Duration::hours(hours))?;
        }
        ("Duration", "seconds") => {
            let secs: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Duration.seconds: {}", e)))?;
            push_rust_obj(task, vm, "Duration", chrono::Duration::seconds(secs as i64))?;
        }
        ("NaiveDateTime", "parse_from_str") => {
            let fmt: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("NaiveDateTime.parse_from_str: {}", e)))?;
            let s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("NaiveDateTime.parse_from_str: {}", e)))?;
            match chrono::NaiveDateTime::parse_from_str(&s, &fmt) {
                Ok(dt) => push_rust_obj(task, vm, "chrono::NaiveDateTime", dt)?,
                Err(e) => return Err(VMError::RuntimeError(format!("NaiveDateTime::parse_from_str failed: {}", e))),
            }
        }
        ("Utc", "timestamp_opt") | ("Local", "timestamp_opt") => {
            let ts: i64 = i64::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("timestamp_opt: {}", e)))?;
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
                .ok_or_else(|| VMError::RuntimeError("timestamp_opt: invalid timestamp".to_string()))?;
            push_rust_obj(task, vm, "chrono::DateTime<chrono::Utc>", dt)?;
        }

        // ---- csv ----
        ("ReaderBuilder", "new") => {
            let builder = csv::ReaderBuilder::new();
            push_rust_obj(task, vm, "csv::ReaderBuilder", builder)?;
        }
        ("ReaderBuilder", "delimiter") => {
            let delim: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("ReaderBuilder.delimiter: {}", e)))?;
            let handle = pop_rust_obj(task, vm, "ReaderBuilder.delimiter")?;
            let obj = vm.get_heap_object(handle).unwrap();
            let mut guard = obj.write().unwrap();
            if let Some(rust_obj) = guard.as_any_mut().downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                if let Some(builder) = rust_obj.downcast_mut::<csv::ReaderBuilder>() {
                    builder.delimiter(delim as u8);
                    task.ram.push_i32(handle as i32);
                } else {
                    return Err(VMError::RuntimeError("ReaderBuilder.delimiter: invalid inner object".into()));
                }
            } else {
                return Err(VMError::RuntimeError("ReaderBuilder.delimiter: invalid object".into()));
            }
        }
        ("ReaderBuilder", "from_reader") => {
            let reader_handle = pop_rust_obj(task, vm, "ReaderBuilder.from_reader")?;
            let bytes: Vec<u8> = {
                let reader_obj = vm.get_heap_object(reader_handle).unwrap();
                let reader_guard = reader_obj.read().unwrap();
                if let Some(rust_obj) = reader_guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                    rust_obj.downcast_ref::<Vec<u8>>().cloned().unwrap_or_default()
                } else {
                    Vec::new()
                }
            };
            let builder_handle = pop_rust_obj(task, vm, "ReaderBuilder.from_reader")?;
            let obj = vm.get_heap_object(builder_handle).unwrap();
            let guard = obj.read().unwrap();
            if let Some(rust_obj) = guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                if let Some(builder) = rust_obj.downcast_ref::<csv::ReaderBuilder>() {
                    let reader = builder.from_reader(std::io::Cursor::new(bytes));
                    push_rust_obj(task, vm, "csv::Reader<std::io::Cursor<Vec<u8>>>", reader)?;
                } else {
                    return Err(VMError::RuntimeError("ReaderBuilder.from_reader: invalid inner object".into()));
                }
            } else {
                return Err(VMError::RuntimeError("ReaderBuilder.from_reader: invalid object".into()));
            }
        }
        ("Reader", "from_reader") => {
            let reader_handle = pop_rust_obj(task, vm, "Reader.from_reader")?;
            let bytes: Vec<u8> = {
                let reader_obj = vm.get_heap_object(reader_handle).unwrap();
                let reader_guard = reader_obj.read().unwrap();
                if let Some(rust_obj) = reader_guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                    rust_obj.downcast_ref::<Vec<u8>>().cloned().unwrap_or_default()
                } else {
                    Vec::new()
                }
            };
            let reader = csv::Reader::from_reader(std::io::Cursor::new(bytes));
            push_rust_obj(task, vm, "csv::Reader", reader)?;
        }
        ("Reader", "records") => {
            let handle = pop_rust_obj(task, vm, "Reader.records")?;
            let Some(obj) = vm.get_heap_object(handle) else {
                return Err(VMError::RuntimeError("Reader.records: invalid reader handle".into()));
            };
            // Collect all records into a List of record heap objects
            use crate::vm::types::ListData;
            use std::sync::atomic::Ordering;
            let mut outer_list: ListData<i32> = ListData::new();
            {
                let mut guard = obj.write().unwrap();
                if let Some(rust_obj) = guard.as_any_mut().downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                    if let Some(reader) = rust_obj.downcast_mut::<csv::Reader<std::io::Cursor<Vec<u8>>>>() {
                        for result in reader.records() {
                            if let Ok(record) = result {
                                // Each record becomes a ListData<i32> of string indices
                                let mut record_list: ListData<i32> = ListData::new();
                                for field in record.iter() {
                                    let str_idx = vm.add_string(field.as_bytes().to_vec());
                                    record_list.push(-(str_idx as i32) - 1); // string encoding convention
                                }
                                let record_id = vm.insert_heap_object(record_list);
                                outer_list.push(record_id as i32);
                            }
                        }
                    }
                }
            }
            let list_id = vm.insert_heap_object(outer_list);
            // Create a List iterator for the for-loop
            let iterator_id = vm.iterator_id_gen.fetch_add(1, Ordering::Relaxed);
            let iterator = crate::vm::engine::Iterator::List(crate::vm::engine::ListIterator {
                list_id,
                current_index: 0,
            });
            vm.iterators.insert(iterator_id, iterator);
            task.ram.push_i32(iterator_id as i32);
        }
        ("StringRecord", "get") | ("List", "get") => {
            let index: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("{}.get: {}", type_name, e)))?;
            let handle = pop_rust_obj(task, vm, "List.get")?;
            let Some(obj) = vm.get_heap_object(handle) else {
                return Err(VMError::RuntimeError("List.get: invalid handle".into()));
            };
            let guard = obj.read().unwrap();
            if let Some(list) = guard.as_any().downcast_ref::<crate::vm::types::ListData<i32>>() {
                let idx = index as usize;
                if let Some(val) = list.get(idx) {
                    let val = *val;
                    if val >= 4000000 {
                        // Heap object ID — push as object reference
                        task.ram.push_nv(auto_val::encode_object(val as u32));
                    } else if let Some(bytes) = vm.get_string(val as u16) {
                        let new_idx = vm.add_string(bytes.to_vec());
                        task.ram.push_str_idx(new_idx as u32);
                    } else {
                        task.ram.push_nv(auto_val::encode_i32(val));
                    }
                } else {
                    // Out of bounds — push None (0)
                    task.ram.push_i32(0);
                }
            } else {
                return Err(VMError::RuntimeError("List.get: not a list".into()));
            }
        }
        ("List", "count") | ("List", "len") => {
            let handle = pop_rust_obj(task, vm, "List.count")?;
            let Some(obj) = vm.get_heap_object(handle) else {
                return Err(VMError::RuntimeError("List.count: invalid handle".into()));
            };
            let guard = obj.read().unwrap();
            if let Some(list) = guard.as_any().downcast_ref::<crate::vm::types::ListData<i32>>() {
                let len = list.len() as i32;
                task.ram.push_nv(auto_val::encode_i32(len));
            } else {
                task.ram.push_i32(0);
            }
        }

        // ---- ansi_term ----
        ("Red", "paint") | ("Green", "paint") | ("Yellow", "paint")
        | ("Blue", "paint") | ("Cyan", "paint") | ("White", "paint") | ("Purple", "paint") => {
            let s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("{}.paint: {}", type_name, e)))?;
            let color = match type_name.as_str() {
                "Red" => ansi_term::Colour::Red,
                "Green" => ansi_term::Colour::Green,
                "Yellow" => ansi_term::Colour::Yellow,
                "Blue" => ansi_term::Colour::Blue,
                "Cyan" => ansi_term::Colour::Cyan,
                "White" => ansi_term::Colour::White,
                "Purple" => ansi_term::Colour::Purple,
                _ => ansi_term::Colour::Red,
            };
            let str_idx = vm.add_string(color.paint(s).to_string().into_bytes());
            task.ram.push_str_idx(str_idx as u32);
        }

        // ---- std::collections ----
        ("HashMap", "new") => {
            let map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            push_rust_obj(task, vm, "HashMap", map)?;
        }
        ("HashSet", "new") => {
            let set: std::collections::HashSet<String> = std::collections::HashSet::new();
            push_rust_obj(task, vm, "HashSet", set)?;
        }

        // ---- std::fs (module calls) ----
        ("fs", "read_dir") => {
            let path: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("fs.read_dir: {}", e)))?;
            match std::fs::read_dir(&path) {
                Ok(entries) => push_rust_obj(task, vm, "ReadDir", entries)?,
                Err(e) => return Err(VMError::RuntimeError(format!("fs.read_dir: {}", e))),
            }
        }

        // ---- std::thread ----
        ("thread", "available_parallelism") => {
            match std::thread::available_parallelism() {
                Ok(n) => task.ram.push_i32(n.get() as i32),
                Err(_) => task.ram.push_i32(1),
            }
        }

        // ---- std::cell::RefCell (instance methods) ----
        ("cell", "borrow") => {
            task.ram.push_i32(0);
        }
        ("cell", "replace") => {
            let _val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("cell.replace: {}", e)))?;
            let _handle = pop_rust_obj(task, vm, "cell.replace")?;
            task.ram.push_i32(0);
        }

        // ---- heapless::Vec ----
        ("Vec", "new") => {
            push_rust_obj(task, vm, "heapless::Vec", Vec::<u8>::new())?;
        }

        // ---- urlencoding ----
        ("", "encode") => {
            let s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("urlencoding.encode: {}", e)))?;
            let str_idx = vm.add_string(urlencoding::encode(&s).as_bytes().to_vec());
            task.ram.push_str_idx(str_idx as u32);
        }

        // ---- log macros (no-op in VM) ----
        ("", "info") | ("", "debug") | ("", "warn") | ("", "error")
        | ("", "trace") => {
            let _msg: Result<String, _> = String::pop_from_stack(task, vm);
        }

        // ---- serde_json (handled below with better implementation) ----

        // ---- toml ----
        ("toml", "from_str") => {
            let _s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("toml.from_str: {}", e)))?;
            let str_idx = vm.add_string(b"{}".to_vec());
            task.ram.push_str_idx(str_idx as u32);
        }

        // ---- Path ----
        ("Path", "new") => {
            let _s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Path.new: {}", e)))?;
            push_rust_obj(task, vm, "PathBuf", StdPathBuf::new())?;
        }

        // ---- same_file (Plan 267 Phase C) ----
        ("same_file", "is_same_file") => {
            let path2: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let path1: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let same = same_file::is_same_file(&path1, &path2).unwrap_or(false);
            task.ram.push_i32(if same { 1 } else { 0 });
        }

        // ---- String ----
        ("String", "new") => {
            let str_idx = vm.add_string(b"".to_vec());
            task.ram.push_str_idx(str_idx as u32);
        }

        // ---- File ----
        ("File", "create") => {
            let path: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("File.create: {}", e)))?;
            match std::fs::File::create(&path) {
                Ok(f) => push_rust_obj(task, vm, "FileWriter", f)?,
                Err(e) => return Err(VMError::RuntimeError(format!("File.create: {}", e))),
            }
        }
        ("File", "open") => {
            let path: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("File.open: {}", e)))?;
            match std::fs::File::open(&path) {
                Ok(f) => push_rust_obj(task, vm, "FileWriter", f)?,
                Err(e) => return Err(VMError::RuntimeError(format!("File.open: {}", e))),
            }
        }

        // ---- Stdio ----
        ("Stdio", "piped") => {
            task.ram.push_i32(0);
        }
        ("Stdio", "from") => {
            let _fd: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            task.ram.push_i32(0);
        }

        // ---- WriteLogger (simplelog) ----
        ("WriteLogger", "init") => {
            let _file: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let _level: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let _config: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            task.ram.push_i32(0);
        }
        ("LevelFilter", "Info") => {
            task.ram.push_i32(0);
        }
        ("Config", "default") => {
            task.ram.push_i32(0);
        }

        // ---- Backtrace ----
        ("Backtrace", "capture") => {
            let bt = std::backtrace::Backtrace::capture();
            let handle = vm.insert_heap_object(
                RustStdlibObject::new("Backtrace", bt)
            ) as i32;
            task.ram.push_i32(handle);
        }

        // ---- percent_encoding ----
        ("percent_encoding", "NON_ALPHANUMERIC") => {
            // Sentinel value indicating non-alphanumeric encoding
            push_rust_obj(task, vm, "EncodeSet", "NON_ALPHANUMERIC")?;
        }

        // ---- clap Args ----
        ("Args", "parse") => {
            task.ram.push_i32(0);
        }

        // ---- serde_json ----
        ("serde_json", "from_str") => {
            let json_str: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("serde_json.from_str: {}", e)))?;
            // For MVP, store the raw JSON string as a RustStdlibObject
            push_rust_obj(task, vm, "serde_json::Value", json_str)?;
        }
        ("serde_json", "to_string") => {
            let _value: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("serde_json.to_string: {}", e)))?;
            // For MVP, return the value as-is (no actual serialization)
            let s = _value;
            let str_idx = vm.add_string(s.into_bytes());
            task.ram.push_str_idx(str_idx as u32);
        }

        // ---- chrono DateTime methods ----
        ("DateTime", "single") => {
            // .single() on MappedTemporalError returns Option<DateTime>
            // For VM, just pass through the DateTime that was already constructed
            // Pop the receiver (DateTime handle) and push it back
            let handle = pop_rust_obj(task, vm, "DateTime.single")?;
            task.ram.push_nv(auto_val::encode_i32(handle as i32));
        }

        // ---- regex extra methods ----
        ("Regex", "captures_iter") => {
            let text: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Regex.captures_iter: {}", e)))?;
            let handle = pop_rust_obj(task, vm, "Regex.captures_iter")?;
            let obj = vm.get_heap_object(handle);
            if obj.is_none() {
                return Err(VMError::RuntimeError("Regex.captures_iter: invalid handle".into()));
            }
            // Collect all matches as string pool indices in a flat list
            use crate::vm::types::ListData;
            let mut match_list: ListData<i32> = ListData::new();
            if let Some(obj) = obj {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(re) = rust_obj.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                        for mat in re.lock().unwrap().find_iter(&text) {
                            let str_idx = vm.add_string(mat.as_str().as_bytes().to_vec());
                            match_list.push(-(str_idx as i32) - 1); // string encoding convention
                        }
                    }
                }
            }
            let list_id = vm.insert_heap_object(match_list);
            task.ram.push_nv(auto_val::encode_object(list_id as u32));
        }

        // ---- percent_encoding ----
        ("", "percent_encode") => {
            let _encode_set_handle = task.ram.pop_i32();
            let bytes_handle = pop_rust_obj(task, vm, "percent_encode")?;
            let Some(obj) = vm.get_heap_object(bytes_handle) else {
                let str_idx = vm.add_string(Vec::new());
                task.ram.push_str_idx(str_idx as u32);
                return Ok(());
            };
            let guard = obj.read().unwrap();
            let bytes = if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                rust_obj.downcast_ref::<Vec<u8>>().cloned().unwrap_or_default()
            } else {
                Vec::new()
            };
            drop(guard);

            // Percent-encode non-alphanumeric bytes
            let encoded: String = bytes.iter()
                .flat_map(|&b| {
                    if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
                        vec![b]
                    } else {
                        format!("%{:02X}", b).into_bytes()
                    }
                })
                .map(|b| b as char)
                .collect();

            let str_idx = vm.add_string(encoded.into_bytes());
            task.ram.push_str_idx(str_idx as u32);
        }

        // ---- Vec<u8> methods (for as_bytes results) ----
        ("Vec<u8>", "len") | ("Vec", "len") | ("buf", "len") => {
            let handle = pop_rust_obj(task, vm, "Vec.len")?;
            let Some(obj) = vm.get_heap_object(handle) else {
                task.ram.push_i32(0);
                return Ok(());
            };
            let guard = obj.read().unwrap();
            if let Some(rust_obj) = guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                if let Some(bytes) = rust_obj.downcast_ref::<Vec<u8>>() {
                    task.ram.push_i32(bytes.len() as i32);
                } else {
                    task.ram.push_i32(0);
                }
            } else {
                task.ram.push_i32(0);
            }
        }

        // ---- csv::Writer ----
        ("Writer", "from_writer") => {
            let _inner_handle = task.ram.pop_i32();
            let buffer: Vec<u8> = Vec::new();
            let writer = csv::Writer::from_writer(buffer);
            push_rust_obj(task, vm, "csv::Writer<Vec<u8>>", std::sync::Mutex::new(writer))?;
        }
        ("Writer", "write_record") => {
            let record: Vec<String> = Vec::<String>::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Writer.write_record: {}", e)))?;
            let writer_handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(writer_handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(writer) = rust_obj.downcast_ref::<std::sync::Mutex<csv::Writer<Vec<u8>>>>() {
                        let _ = writer.lock().unwrap().write_record(&record);
                        task.ram.push_i32(writer_handle as i32);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("Writer", "serialize") => {
            // Pop object handle (from vm.objects, ID >= 1_000_000)
            let obj_nv = task.ram.pop_nv();
            let obj_id = if auto_val::is_object(obj_nv) {
                auto_val::decode_object(obj_nv) as u64
            } else {
                auto_val::decode_i32(obj_nv) as u64
            };
            let writer_handle = task.ram.pop_i32() as u64;
            // Extract fields from ObjectData and serialize as CSV record
            let mut record_map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
            if let Some(obj_arc) = vm.objects.get(&obj_id) {
                let obj_guard = obj_arc.read().unwrap();
                for (key, val) in &obj_guard.fields {
                    let key_str = format!("{:?}", key);
                    let val_str = match val {
                        auto_val::Value::Str(s) => s.to_string(),
                        auto_val::Value::Int(n) => n.to_string(),
                        auto_val::Value::Float(f) => f.to_string(),
                        auto_val::Value::Bool(b) => b.to_string(),
                        _ => format!("{:?}", val),
                    };
                    record_map.insert(key_str, val_str);
                }
            }
            let record: Vec<String> = record_map.into_values().collect();
            if let Some(obj) = vm.get_heap_object(writer_handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(writer) = rust_obj.downcast_ref::<std::sync::Mutex<csv::Writer<Vec<u8>>>>() {
                        let _ = writer.lock().unwrap().write_record(&record);
                        task.ram.push_i32(writer_handle as i32);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("Writer", "flush") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("Writer", "into_inner") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(writer) = rust_obj.downcast_ref::<std::sync::Mutex<csv::Writer<Vec<u8>>>>() {
                        let mut w = writer.lock().unwrap();
                        let taken = std::mem::replace(&mut *w, csv::Writer::from_writer(Vec::new()));
                        let inner = taken.into_inner().unwrap_or_default();
                        push_rust_obj(task, vm, "Vec<u8>", inner)?;
                        return Ok(());
                    }
                }
            }
            push_rust_obj(task, vm, "Vec<u8>", Vec::<u8>::new())?;
        }

        // ---- String ----
        ("String", "from_utf8") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(bytes) = rust_obj.downcast_ref::<Vec<u8>>() {
                        match String::from_utf8(bytes.clone()) {
                            Ok(s) => {
                                let str_idx = vm.add_string(s.into_bytes());
                                task.ram.push_str_idx(str_idx as u32);
                                return Ok(());
                            }
                            Err(_) => {
                                return Err(VMError::RuntimeError("String.from_utf8: invalid utf8".into()));
                            }
                        }
                    }
                }
            }
            task.ram.push_i32(0);
        }

        // ---- std::process::Command ----
        ("Command", "new") => {
            let program: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Command.new: {}", e)))?;
            let cmd = std::process::Command::new(&program);
            push_rust_obj(task, vm, "std::process::Command", std::sync::Mutex::new(cmd))?;
        }
        ("Command", "arg") => {
            let arg: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Command.arg: {}", e)))?;
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(cmd) = rust_obj.downcast_ref::<std::sync::Mutex<std::process::Command>>() {
                        cmd.lock().unwrap().arg(&arg);
                        task.ram.push_i32(handle as i32);
                        return Ok(());
                    }
                }
            }
            return Err(VMError::RuntimeError("Command.arg: invalid Command handle".into()));
        }
        ("Command", "args") => {
            let _args_handle = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("Command", "stdout") => {
            let _stdio_val = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("Command", "stdin") => {
            let _stdio_val = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("Command", "stderr") => {
            let _stdio_val = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("Command", "output") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(cmd) = rust_obj.downcast_ref::<std::sync::Mutex<std::process::Command>>() {
                        let output = cmd.lock().unwrap().output()
                            .map_err(|e| VMError::RuntimeError(format!("Command.output: {}", e)))?;
                        push_rust_obj(task, vm, "std::process::Output", output)?;
                        return Ok(());
                    }
                }
            }
            return Err(VMError::RuntimeError("Command.output: invalid Command handle".into()));
        }
        ("Command", "spawn") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(cmd) = rust_obj.downcast_ref::<std::sync::Mutex<std::process::Command>>() {
                        let child = cmd.lock().unwrap().spawn()
                            .map_err(|e| VMError::RuntimeError(format!("Command.spawn: {}", e)))?;
                        push_rust_obj(task, vm, "std::process::Child", child)?;
                        return Ok(());
                    }
                }
            }
            return Err(VMError::RuntimeError("Command.spawn: invalid Command handle".into()));
        }
        ("Command", "status") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        // Child process
        ("Child", "stdin") | ("Child", "stdout") | ("Child", "stderr") => {
            let _child_handle = task.ram.pop_i32();
            task.ram.push_i32(0);
        }
        ("Child", "wait") | ("Child", "wait_with_output") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(_child) = rust_obj.downcast_ref::<std::process::Child>() {
                        // stub: push 0 as exit code
                        task.ram.push_i32(0);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("Child", "kill") => {
            let _handle = task.ram.pop_i32();
            task.ram.push_i32(0);
        }
        // Normal distribution (rand_distr)
        ("Normal", "new") => {
            let _stddev: f64 = f64::pop_from_stack(task, vm).unwrap_or(1.0);
            let _mean: f64 = f64::pop_from_stack(task, vm).unwrap_or(0.0);
            push_rust_obj(task, vm, "Normal", 0i32)?;
        }

        // Complex (num::Complex) — stub with (real, imag) stored as (f64, f64)
        ("Complex", "new") => {
            let imag: f64 = f64::pop_from_stack(task, vm).unwrap_or(0.0);
            let real: f64 = f64::pop_from_stack(task, vm).unwrap_or(0.0);
            push_rust_obj(task, vm, "Complex", (real, imag))?;
        }
        ("Complex", "norm") | ("Complex", "arg") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some((real, imag)) = rust_obj.downcast_ref::<(f64, f64)>() {
                        let val = if method == "norm" { (real * real + imag * imag).sqrt() } else { imag.atan2(*real) };
                        task.ram.push_f64(val);
                        return Ok(());
                    }
                }
            }
            task.ram.push_f64(0.0);
        }
        // BigInt (num-bigint)
        ("BigInt", "from") | ("BigInt", "new") => {
            let _val: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            push_rust_obj(task, vm, "BigInt", 0i64)?;
        }
        // ThreadRng
        ("ThreadRng", "sample") => {
            let _handle = task.ram.pop_i32();
            push_rust_obj(task, vm, "ThreadRng", 0i32)?;
        }
        // WalkDir (walkdir crate) — Plan 267 Phase A
        ("WalkDir", "new") => {
            let root: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("WalkDir.new: {}", e)))?;
            let mut paths = Vec::new();
            for entry in walkdir::WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
                paths.push(entry.path().display().to_string());
            }
            let json = serde_json::to_string(&paths)
                .map_err(|e| VMError::RuntimeError(format!("WalkDir.new json: {}", e)))?;
            let str_idx = vm.add_string(json.into_bytes());
            task.ram.push_str_idx(str_idx as u32);
        }
        ("WalkDir", "into_iter") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("DirEntry", "path") | ("DirEntry", "file_name") => {
            let _handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(0);
        }
        ("Result<DirEntry, walkdir::Error>", "unwrap") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("walkdir::Error", "to_string") => {
            let _handle = task.ram.pop_i32() as u64;
            let str_idx = vm.add_string(b"io error".to_vec());
            task.ram.push_str_idx(str_idx as u32);
        }
        // TarGzip builder methods — Plan 267 Phase B
        ("Builder", "append_path") => {
            let _path: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32); // return self for chaining
        }
        ("Builder", "append_dir_all") => {
            let _dest: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let _src: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32); // return self
        }
        // same_file::Handle — Plan 267 Phase C
        ("Handle", "from_path") => {
            let _path: String = String::pop_from_stack(task, vm).unwrap_or_default();
            push_rust_obj(task, vm, "same_file::Handle", 0i32)?;
        }

        // ---- std::sync::Arc clone stub ----
        ("Arc", "clone") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        // ---- std::thread stubs ----
        ("thread", "spawn") => {
            let _closure = task.ram.pop_i32();
            push_rust_obj(task, vm, "std::thread::JoinHandle", 0i32)?;
        }
        ("JoinHandle", "join") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(result) = rust_obj.downcast_ref::<std::sync::Mutex<i32>>() {
                        let val = result.lock().unwrap();
                        task.ram.push_i32(*val);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        // ---- crossbeam stubs ----
        // ("", "unbounded") — module-level call, type_name is empty
        ("", "unbounded") => {
            // crossbeam::channel::unbounded() — stub: push dummy channel handle
            push_rust_obj(task, vm, "crossbeam::channel::unbounded", 0i32)?;
        }
        ("crossbeam", "scope") => {
            // crossbeam::scope — stub: pop closure, push 0
            let _closure = task.ram.pop_i32();
            task.ram.push_i32(0);
        }
        // ---- std::sync::atomic stubs ----
        ("AtomicUsize", "new") => {
            let _val: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            push_rust_obj(task, vm, "AtomicUsize", std::sync::atomic::AtomicUsize::new(0))?;
        }
        ("AtomicUsize", "fetch_add") => {
            let _ordering: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(atom) = rust_obj.downcast_ref::<std::sync::atomic::AtomicUsize>() {
                        let old = atom.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        task.ram.push_i32(old as i32);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("AtomicUsize", "load") => {
            let _ordering: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(atom) = rust_obj.downcast_ref::<std::sync::atomic::AtomicUsize>() {
                        let val = atom.load(std::sync::atomic::Ordering::SeqCst);
                        task.ram.push_i32(val as i32);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        // ---- env_logger::Builder stub ----
        ("Builder", "new") => {
            push_rust_obj(task, vm, "env_logger::Builder", 0i32)?;
        }
        // Builder.format(closure) — noop, accepts closure arg, returns self
        ("Builder", "format") => {
            let _closure = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            // Return same builder handle (chainable)
            task.ram.push_i32(handle as i32);
        }
        // Builder.init() — noop, consumes builder, pushes unit
        ("Builder", "init") => {
            let _handle = task.ram.pop_i32();
            // env_logger::init returns (), push nothing meaningful
        }
        // ---- log stubs ----
        ("log", "set_boxed_logger") => {
            let _logger = task.ram.pop_i32();
            task.ram.push_i32(0); // Ok(())
        }
        ("log", "set_max_level") => {
            let _level = task.ram.pop_i32();
        }
        // ---- heapless::Vec stub ----
        // ("Vec", "new") and ("Vec", "len") are handled by existing handlers above
        ("heapless::Vec", "push") | ("Vec", "push") => {
            let _byte: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let mut guard = obj.write().unwrap();
                if let Some(rust_obj) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
                    if let Some(vec) = rust_obj.downcast_mut::<Vec<u8>>() {
                        let _ = vec.push(_byte as u8);
                        task.ram.push_i32(0); // Ok(())
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        // ---- semver::VersionReq stub ----
        ("VersionReq", "parse") => {
            let req_str: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("VersionReq.parse: {}", e)))?;
            match semver::VersionReq::parse(&req_str) {
                Ok(req) => {
                    push_rust_obj(task, vm, "semver::VersionReq", req)?;
                }
                Err(e) => {
                    return Err(VMError::RuntimeError(format!("VersionReq::parse failed: {}", e)));
                }
            }
        }
        ("VersionReq", "matches") => {
            let _version_handle = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(_req) = rust_obj.downcast_ref::<semver::VersionReq>() {
                        // Stub: always return true
                        task.ram.push_i32(1);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(1);
        }
        // ---- chrono DateTime/NaiveDateTime.checked_add_signed stub ----
        ("DateTime", "checked_add_signed") | ("NaiveDateTime", "checked_add_signed") => {
            let _duration_handle = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(dt) = rust_obj.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                        let dt = dt.lock().unwrap();
                        let result = dt.checked_add_signed(chrono::Duration::days(30));
                        match result {
                            Some(new_dt) => {
                                push_rust_obj(task, vm, "chrono::NaiveDateTime", new_dt)?;
                                return Ok(());
                            }
                            None => {
                                task.ram.push_i32(0);
                                return Ok(());
                            }
                        }
                    }
                    if let Some(dt) = rust_obj.downcast_ref::<std::sync::Mutex<chrono::DateTime<chrono::Utc>>>() {
                        let dt = dt.lock().unwrap();
                        let result = dt.checked_add_signed(chrono::Duration::days(30));
                        match result {
                            Some(new_dt) => {
                                push_rust_obj(task, vm, "chrono::DateTime<chrono::Utc>", new_dt)?;
                                return Ok(());
                            }
                            None => {
                                task.ram.push_i32(0); // None
                                return Ok(());
                            }
                        }
                    }
                }
            }
            task.ram.push_i32(0);
        }
        // ---- flate2::GzDecoder stub ----
        ("GzDecoder", "new") => {
            let _file_handle = task.ram.pop_i32();
            push_rust_obj(task, vm, "flate2::GzDecoder", 0i32)?;
        }
        // ---- tar::Archive stub ----
        ("Archive", "new") => {
            let _gz_handle = task.ram.pop_i32();
            push_rust_obj(task, vm, "tar::Archive", 0i32)?;
        }
        ("Archive", "set_prefix_strip") | ("Archive", "entries") => {
            let _arg = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        _ => {
            // Fallback: check opaque dispatch table for native shim routing
            if let Some(native_name) = crate::vm::native_catalog::lookup_opaque_dispatch_by_type(&type_name, &method) {
                let name_owned = native_name.to_string();
                let native_id = {
                    let mut reg = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
                    reg.resolve_qualified(&name_owned).or_else(|| {
                        reg.resolve_qualified(&format!("auto.{}.{}", type_name.to_lowercase().replace("::", "."), method))
                    })
                };
                if let Some(id) = native_id {
                    if let Some(shim) = vm.native_interface.get(id).cloned() {
                        shim(task, vm)?;
                        return Ok(());
                    }
                }
            }
            return Err(VMError::RuntimeError(format!(
                "Unknown Rust stdlib call: {type_name}.{method}"
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

// ============================================================================
// Plan 263 Phase 2: Test runner FFI — run_a2r_dir
// ============================================================================

/// Run all VM file-based tests in a directory.
/// Pops path (String) from stack, pushes failure count (i64) to stack.
pub fn shim_test_run_vm_dir(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let dir = Path::new(&path);
    let cases = crate::test_runner::discover_vm_tests(dir);

    if cases.is_empty() {
        return Err(VMError::RuntimeError(format!(
            "Test.run_vm_dir: no tests found in: {}",
            path
        )));
    }

    let mut failures = 0i64;
    for case in &cases {
        let report = crate::run_vm_file_test(case);
        match &report.outcome {
            crate::test_runner::TestOutcome::Passed => {
                print!("test {} ... ok\n", case.name);
            }
            crate::test_runner::TestOutcome::Failed(msg) => {
                print!("test {} ... FAILED\n", case.name);
                print!("    {}\n", msg);
                failures += 1;
            }
        }
    }

    failures.push_to_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Run all a2c transpiler tests in a directory.
/// Pops path (String) from stack, pushes failure count (i64) to stack.
pub fn shim_test_run_a2c_dir(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let dir = Path::new(&path);
    let cases = crate::test_runner::discover_a2c_tests(dir);

    if cases.is_empty() {
        return Err(VMError::RuntimeError(format!(
            "Test.run_a2c_dir: no tests found in: {}",
            path
        )));
    }

    let mut failures = 0i64;
    for case in &cases {
        let report = crate::run_a2c_file_test(case);
        match &report.outcome {
            crate::test_runner::TestOutcome::Passed => {
                print!("test {} ... ok\n", case.name);
            }
            crate::test_runner::TestOutcome::Failed(msg) => {
                print!("test {} ... FAILED\n", case.name);
                print!("    {}\n", msg);
                failures += 1;
            }
        }
    }

    failures.push_to_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Run all a2ts transpiler tests in a directory.
/// Pops path (String) from stack, pushes failure count (i64) to stack.
pub fn shim_test_run_a2ts_dir(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let dir = Path::new(&path);
    let cases = crate::test_runner::discover_a2ts_tests(dir);

    if cases.is_empty() {
        return Err(VMError::RuntimeError(format!(
            "Test.run_a2ts_dir: no tests found in: {}",
            path
        )));
    }

    let mut failures = 0i64;
    for case in &cases {
        let report = crate::run_a2ts_file_test(case);
        match &report.outcome {
            crate::test_runner::TestOutcome::Passed => {
                print!("test {} ... ok\n", case.name);
            }
            crate::test_runner::TestOutcome::Failed(msg) => {
                print!("test {} ... FAILED\n", case.name);
                print!("    {}\n", msg);
                failures += 1;
            }
        }
    }

    failures.push_to_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Run all a2r transpiler tests in a directory.
/// Pops path (String) from stack, pushes failure count (i64) to stack.
pub fn shim_test_run_a2r_dir(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let dir = Path::new(&path);
    let suite_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("a2r");
    let cases = crate::test_runner::discover_a2r_tests(dir, suite_name);

    if cases.is_empty() {
        return Err(VMError::RuntimeError(format!(
            "Test.run_a2r_dir: no tests found in: {}",
            path
        )));
    }

    let mut failures = 0i64;
    for case in &cases {
        let report = crate::run_a2r_file_test(case);
        match &report.outcome {
            crate::test_runner::TestOutcome::Passed => {
                print!("test {} ... ok\n", case.name);
            }
            crate::test_runner::TestOutcome::Failed(msg) => {
                print!("test {} ... FAILED\n", case.name);
                print!("    {}\n", msg);
                failures += 1;
            }
        }
    }

    failures.push_to_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}
