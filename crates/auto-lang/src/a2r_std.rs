//! AutoLang Standard Library for a2r (Auto-to-Rust Transpiler)
//!
//! This module provides Rust implementations of AutoLang's standard types
//! so that transpiled code can compile and run.
//!
//! Types implemented:
//! - `List<T>` - Dynamic array with push, pop, len, etc.
//! - `May<T>` - Optional value (alias for Option<T>)
//!
//! Usage in transpiled code:
//! ```rust,ignore
//! use auto_lang::a2r_std::List;
//!
//! fn main() {
//!     let mut list = List::new();
//!     list.push(1);
//! }
//! ```

use std::cell::RefCell;

/// AutoLang's List<T> - a dynamic array similar to Vec<T>
/// but with AutoLang's method naming conventions.
#[derive(Debug, Clone)]
pub struct List<T> {
    inner: RefCell<Vec<T>>,
}

impl<T> List<T> {
    /// Create a new empty list
    pub fn new() -> Self {
        List {
            inner: RefCell::new(Vec::new()),
        }
    }

    /// Create a list with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        List {
            inner: RefCell::new(Vec::with_capacity(capacity)),
        }
    }

    /// Push a value to the end of the list
    pub fn push(&self, value: T) {
        self.inner.borrow_mut().push(value);
    }

    /// Pop a value from the end of the list
    pub fn pop(&self) -> Option<T> {
        self.inner.borrow_mut().pop()
    }

    /// Get the length of the list
    pub fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }

    /// Get a value by index (returns cloned value)
    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        self.inner.borrow().get(index).cloned()
    }

    /// Set value at index
    pub fn set(&self, index: usize, value: T) {
        self.inner.borrow_mut()[index] = value;
    }

    /// Clear the list
    pub fn clear(&self) {
        self.inner.borrow_mut().clear();
    }

    /// Get first element
    pub fn first(&self) -> Option<T>
    where
        T: Clone,
    {
        self.inner.borrow().first().cloned()
    }

    /// Get last element
    pub fn last(&self) -> Option<T>
    where
        T: Clone,
    {
        self.inner.borrow().last().cloned()
    }

    /// Insert at index
    pub fn insert(&self, index: usize, value: T) {
        self.inner.borrow_mut().insert(index, value);
    }

    /// Remove at index
    pub fn remove(&self, index: usize) -> T {
        self.inner.borrow_mut().remove(index)
    }

    /// Convert to Vec
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.inner.borrow().clone()
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> std::ops::Index<usize> for List<T> {
    type Output = T;
    fn index(&self, i: usize) -> &Self::Output {
        if let Some(val) = self.get(i) {
            Box::leak(Box::new(val))
        } else {
            panic!("index out of bounds: the len is {} but the index is {}", self.len(), i);
        }
    }
}

impl<T: Clone> std::ops::IndexMut<usize> for List<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        self.inner.get_mut().index_mut(i)
    }
}

impl<T: Clone> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_inner().into_iter()
    }
}

impl<T: Clone> From<Vec<T>> for List<T> {
    fn from(vec: Vec<T>) -> Self {
        List {
            inner: RefCell::new(vec),
        }
    }
}

/// May<T> - AutoLang's optional type (alias for Option<T>)
pub type May<T> = Option<T>;

// Type aliases for May<T> with specific types (for a2r transpiler)
pub type MayInt = Option<i32>;
pub type MayUint = Option<u32>;
pub type MayFloat = Option<f64>;
pub type MayDouble = Option<f64>;
pub type MayChar = Option<char>;
pub type MayBool = Option<bool>;
pub type MayStr = Option<String>;

/// Nil - AutoLang's nil value type marker
pub struct Nil;

/// Create a Nil value (None)
pub fn nil<T>() -> Option<T> {
    None
}

/// AutoLang's Json module - thin wrappers around serde_json for transpiled code
#[allow(non_snake_case)]
pub mod json {
    use serde_json::Value;

    pub fn is_valid(s: &str) -> bool {
        serde_json::from_str::<Value>(s).is_ok()
    }

    pub fn parse(s: &str) -> Value {
        serde_json::from_str(s).unwrap_or(Value::Null)
    }

    pub fn get_at(val: &Value, idx: usize) -> &Value {
        static NULL_VALUE: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
        val.get(idx).unwrap_or_else(|| NULL_VALUE.get_or_init(|| Value::Null))
    }

    pub fn get<'a>(val: &'a Value, key: &str) -> &'a Value {
        static NULL_VALUE: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
        val.get(key).unwrap_or_else(|| NULL_VALUE.get_or_init(|| Value::Null))
    }

    pub fn get_owned(val: &Value, key: &str) -> Value {
        val.get(key).cloned().unwrap_or(Value::Null)
    }

    pub fn get_str(val: &Value, key: &str) -> String {
        val.get(key).and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default()
    }

    pub fn as_string(val: &Value) -> String {
        val.as_str().map(|s| s.to_string()).unwrap_or_default()
    }

    pub fn as_string_opt(val: Option<&Value>) -> String {
        val.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default()
    }

    pub fn get_u64(val: &Value, key: &str) -> u64 {
        val.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
    }

    pub fn as_int(val: &Value) -> i64 {
        val.as_i64().unwrap_or(0)
    }

    pub fn as_number(val: &Value) -> f64 {
        val.as_f64().unwrap_or(0.0)
    }

    pub fn as_bool(val: &Value) -> bool {
        val.as_bool().unwrap_or(false)
    }

    pub fn is_null(val: &Value) -> bool {
        val.is_null()
    }

    pub fn len(val: &Value) -> usize {
        match val {
            Value::Array(a) => a.len(),
            Value::Object(o) => o.len(),
            _ => 0,
        }
    }

    pub fn len_str(s: &str) -> usize {
        match serde_json::from_str::<Value>(s) {
            Ok(Value::Array(a)) => a.len(),
            Ok(Value::Object(o)) => o.len(),
            _ => 0,
        }
    }

    pub fn has_key(val: &Value, key: &str) -> bool {
        val.get(key).is_some()
    }

    pub fn has_key_str(s: &str, key: &str) -> bool {
        match serde_json::from_str::<Value>(s) {
            Ok(val) => val.get(key).is_some(),
            Err(_) => false,
        }
    }

    pub fn to_string(val: &Value) -> String {
        serde_json::to_string(val).unwrap_or_default()
    }

    pub fn keys(val: &Value) -> Vec<String> {
        match val {
            Value::Object(map) => map.keys().cloned().collect(),
            _ => Vec::new(),
        }
    }
}

// =============================================================================
// String functions for a2r transpiler
// =============================================================================

/// Create a new string with initial capacity
/// In AutoLang: str_new("hello", 10)
pub fn str_new(s: &str, _capacity: usize) -> String {
    s.to_string()
}

/// Find substring in string, starting from position.
/// Returns -1 if not found (matches Auto semantics, not Rust's Option).
/// In AutoLang: s.find(needle, start_pos)
pub fn str_find(s: &str, needle: &str, start: i32) -> i32 {
    if start < 0 || start as usize > s.len() {
        return -1;
    }
    match s[start as usize..].find(needle) {
        Some(idx) => (start as usize + idx) as i32,
        None => -1,
    }
}

/// Get substring from position with length.
/// In AutoLang: s.substr(start, length)
pub fn str_substr(s: &str, start: i32, length: i32) -> String {
    if start < 0 || length <= 0 || start as usize >= s.len() {
        return String::new();
    }
    let start_usize = start as usize;
    let end = std::cmp::min(start_usize + length as usize, s.len());
    s[start_usize..end].to_string()
}

/// String contains check.
/// In AutoLang: s.contains(needle)
pub fn str_contains(s: &str, needle: &str) -> bool {
    s.contains(needle)
}

/// String ends with check. Returns bool for Rust compat.
/// In AutoLang: s.ends_with(suffix)
pub fn str_ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// Get string length
/// In AutoLang: str_len(s)
/// Accepts both String and &str for transpiler convenience
pub fn str_len<S: AsRef<str>>(s: S) -> usize {
    s.as_ref().len()
}

/// Append to string
/// In AutoLang: str_append(s, " world")
pub fn str_append(s: &mut String, other: &str) {
    s.push_str(other);
}

/// Convert a JSON value to i32 (handles both string and number values)
/// Used by to_int() interceptor when the receiver may be Option<Value>
pub fn value_to_int(val: &serde_json::Value) -> i32 {
    if val.is_i64() || val.is_u64() || val.is_f64() {
        val.as_i64().map(|n| n as i32)
    } else if let Some(s) = val.as_str() {
        s.parse::<i32>().ok()
    } else {
        None
    }.unwrap_or(0)
}

/// Get the length of a JSON value (string length for strings, 0 for other types)
pub fn value_len(val: &serde_json::Value) -> i32 {
    if let Some(s) = val.as_str() {
        s.len() as i32
    } else {
        0
    }
}

// =============================================================================
// IO module for a2r transpiler
// =============================================================================

/// AutoLang's io module — stdin/stdout helpers
#[allow(non_snake_case)]
pub mod io {
    /// Read a line from stdin (blocks until user presses Enter).
    /// Returns the line without trailing newline, or empty string on EOF.
    pub fn read_line() -> String {
        use std::io;
        let mut buf = String::new();
        match io::stdin().read_line(&mut buf) {
            Ok(_) => {
                let trimmed = buf.trim_end_matches('\n').trim_end_matches('\r').to_string();
                trimmed
            }
            Err(_) => String::new(),
        }
    }
}

// =============================================================================
// Environment module for a2r transpiler
// =============================================================================

/// AutoLang's env module — thin wrappers around std::env
#[allow(non_snake_case)]
pub mod env {
    pub fn get(key: &str) -> String {
        std::env::var(key).unwrap_or_default()
    }

    pub fn get_or(key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| default.to_string())
    }

    pub fn set(key: &str, val: &str) {
        std::env::set_var(key, val);
    }

    /// Returns all command-line arguments as a single space-joined string.
    /// Skips the first argument (program name).
    pub fn args() -> String {
        std::env::args().skip(1).collect::<Vec<_>>().join(" ")
    }
}

// =============================================================================
// File system module for a2r transpiler
// =============================================================================

/// Alias: File → fs (AutoLang uses File.xxx() for file operations)
pub mod File {
    pub use super::fs::*;
}

/// AutoLang's fs module — thin wrappers around std::fs
#[allow(non_snake_case)]
pub mod fs {
    pub fn read_to_string(path: &str) -> String {
        std::fs::read_to_string(path).unwrap_or_default()
    }

    pub fn read_text<S: AsRef<str>>(path: S) -> String {
        std::fs::read_to_string(path.as_ref()).unwrap_or_default()
    }

    pub fn write(path: &str, content: &str) -> bool {
        std::fs::write(path, content).is_ok()
    }

    pub fn exists(path: &str) -> i32 {
        if std::path::Path::new(path).exists() { 1 } else { 0 }
    }

    pub fn create_dir(path: &str) -> bool {
        std::fs::create_dir_all(path).is_ok()
    }

    pub fn write_text(path: &str, content: &str) -> bool {
        std::fs::write(path, content).is_ok()
    }

    pub fn read_bytes(path: &str) -> Vec<u8> {
        std::fs::read(path).unwrap_or_default()
    }

    pub fn delete(path: &str) -> bool {
        std::fs::remove_file(path).is_ok()
    }

    pub fn is_dir(path: &str) -> i32 {
        if std::path::Path::new(path).is_dir() { 1 } else { 0 }
    }

    pub fn is_binary(path: &str) -> i32 {
        match std::fs::read(path) {
            Ok(bytes) => {
                if bytes.windows(2).any(|w| w == [0, 0]) { 1 } else { 0 }
            }
            Err(_) => 0,
        }
    }

    pub fn file_size(path: &str) -> i64 {
        match std::fs::metadata(path) {
            Ok(meta) => meta.len() as i64,
            Err(_) => -1,
        }
    }

    pub fn walk(dir: &str) -> String {
        fn do_walk(dir: &str, entries: &mut Vec<String>) {
            if let Ok(rd) = std::fs::read_dir(dir) {
                for entry in rd.flatten() {
                    let path = entry.path();
                    let path_str = path.to_string_lossy().replace("\\", "/");
                    entries.push(format!("\"{}\"", path_str));
                    if path.is_dir() {
                        do_walk(path_str.as_str(), entries);
                    }
                }
            }
        }
        let mut entries: Vec<String> = Vec::new();
        do_walk(dir, &mut entries);
        if entries.is_empty() {
            "[]".to_string()
        } else {
            format!("[{}]", entries.join(","))
        }
    }
}

/// Parse ~/.claude/settings.json into (api_key, base_url, vars HashMap)
/// Returns (None, None, empty_map) if parsing fails
pub fn parse_settings_json(text: &str) -> (Option<String>, Option<String>, std::collections::HashMap<String, String>) {
    let val: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return (None, None, std::collections::HashMap::new()),
    };
    let mut vars = std::collections::HashMap::new();
    if let Some(env) = val.get("env").and_then(|v| v.as_object()) {
        for (k, v) in env {
            if let Some(s) = v.as_str() {
                vars.insert(k.clone(), s.to_string());
            }
        }
    }
    let api_key = vars.get("ANTHROPIC_API_KEY")
        .or_else(|| vars.get("ANTHROPIC_AUTH_TOKEN"))
        .cloned();
    let base_url = vars.get("ANTHROPIC_BASE_URL").cloned();
    (api_key, base_url, vars)
}

// =============================================================================
// Utility functions for a2r transpiler
// =============================================================================

/// Sleep for the specified number of milliseconds
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

/// AutoLang's http module — async HTTP helpers
#[allow(non_snake_case)]
pub mod http {
    /// Async HTTP POST with Anthropic API headers.
    /// Returns (status, body, error, kind) for constructing a local HttpResponse.
    /// Uses spawn_blocking to run reqwest::blocking on the tokio runtime.
    pub async fn post(url: &str, body: &str, api_key: &str) -> (i32, String, String, String) {
        let url = url.to_string();
        let body = body.to_string();
        let api_key = api_key.to_string();
        tokio::task::spawn_blocking(move || {
            let client = reqwest::blocking::Client::new();
            let result = client
                .post(&url)
                .header("content-type", "application/json")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .body(body)
                .send();
            match result {
                Ok(resp) => {
                    let status = resp.status().as_u16() as i32;
                    let resp_body = resp.text().unwrap_or_default();
                    if status >= 200 && status < 300 {
                        (status, resp_body, String::new(), "ok".to_string())
                    } else {
                        (status, resp_body, format!("HTTP {}", status), "error".to_string())
                    }
                }
                Err(e) => (0, String::new(), e.to_string(), "error".to_string())
            }
        }).await.unwrap_or((0, String::new(), "spawn failed".to_string(), "error".to_string()))
    }

    /// Synchronous HTTP POST — blocking version for use in non-async contexts.
    /// Used by Auto's http.post_sync() when transpiled via a2r.
    pub fn post_sync(url: &str, body: &str, api_key: &str) -> (i32, String) {
        let url = url.to_string();
        let body = body.to_string();
        let api_key = api_key.to_string();
        let result = std::thread::spawn(move || -> Result<reqwest::blocking::Response, String> {
            let client = reqwest::blocking::Client::new();
            client
                .post(&url)
                .header("content-type", "application/json")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .body(body)
                .send()
                .map_err(|e| e.to_string())
        }).join().unwrap_or_else(|_| Err("thread panicked".to_string()));
        match result {
            Ok(resp) => {
                let status = resp.status().as_u16() as i32;
                let resp_body = resp.text().unwrap_or_default();
                (status, resp_body)
            }
            Err(e) => (0, e),
        }
    }

    /// Async HTTP POST with Bearer token auth (for OpenAI-compatible APIs).
    /// Returns (status, body).
    pub async fn post_bearer(url: &str, body: &str, api_key: &str) -> (i32, String) {
        let url = url.to_string();
        let body = body.to_string();
        let api_key = api_key.to_string();
        let result = tokio::task::spawn_blocking(move || {
            let client = reqwest::blocking::Client::new();
            let result = client
                .post(&url)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(body)
                .send();
            match result {
                Ok(resp) => {
                    let status = resp.status().as_u16() as i32;
                    let resp_body = resp.text().unwrap_or_default();
                    (status, resp_body)
                }
                Err(e) => (0, format!("HTTP error: {}", e)),
            }
        }).await;
        result.unwrap_or((0, "spawn failed".to_string()))
    }

    thread_local! {
        static LAST_HTTP_STATUS: std::cell::Cell<i32> = std::cell::Cell::new(0);
    }

    pub fn set_last_status(status: i32) {
        LAST_HTTP_STATUS.with(|s| s.set(status));
    }

    pub fn last_status() -> i32 {
        LAST_HTTP_STATUS.with(|s| s.get())
    }

    /// Synchronous HTTP POST with Bearer token auth (blocking, for non-async contexts).
    pub fn post_bearer_sync(url: &str, body: &str, api_key: &str) -> (i32, String) {
        let url = url.to_string();
        let body = body.to_string();
        let api_key = api_key.to_string();
        let result = std::thread::spawn(move || {
            let client = reqwest::blocking::Client::new();
            client
                .post(&url)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(body)
                .send()
                .map_err(|e| e.to_string())
        }).join().unwrap_or_else(|_| Err("thread panicked".to_string()));
        match result {
            Ok(resp) => {
                let status = resp.status().as_u16() as i32;
                let resp_body = resp.text().unwrap_or_default();
                (status, resp_body)
            }
            Err(e) => (0, format!("HTTP error: {}", e)),
        }
    }
}

/// Backward-compat: delegates to http::post
pub async fn http_post(url: &str, body: &str, api_key: &str) -> (i32, String, String, String) {
    http::post(url, body, api_key).await
}

/// Simple DJB2 hash for string → directory-safe name
pub fn simple_hash(s: &str) -> String {
    let mut hash: u64 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    format!("{:x}", hash)
}

/// Current timestamp as seconds since epoch (for session file naming)
pub fn time_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}", duration.as_secs())
}

// =============================================================================
// Shell module for a2r transpiler
// =============================================================================

#[allow(non_snake_case)]
pub mod shell {
    fn json_escape(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "")
    }

    pub fn exec(cmd: &str, timeout_ms: i32) -> String {
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};

        fn make_cmd(c: &str) -> Command {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                let mut cmd = Command::new("cmd");
                cmd.args(["/C", c]).creation_flags(0x08000000);
                cmd
            }
            #[cfg(not(target_os = "windows"))]
            {
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg(c);
                cmd
            }
        }

        let result = if timeout_ms > 0 {
            let mut child = make_cmd(cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();
            match child {
                Ok(ref mut child) => {
                    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
                    loop {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                let mut stdout_buf = Vec::new();
                                let mut stderr_buf = Vec::new();
                                if let Some(mut out) = child.stdout.take() {
                                    let _ = std::io::Read::read_to_end(&mut out, &mut stdout_buf);
                                }
                                if let Some(mut err) = child.stderr.take() {
                                    let _ = std::io::Read::read_to_end(&mut err, &mut stderr_buf);
                                }
                                let stdout = String::from_utf8_lossy(&stdout_buf);
                                let stderr = String::from_utf8_lossy(&stderr_buf);
                                let code = status.code().unwrap_or(-1);
                                return format!(
                                    r#"{{"exit_code":{},"stdout":"{}","stderr":"{}"}}"#,
                                    code, json_escape(&stdout), json_escape(&stderr)
                                );
                            }
                            Ok(None) => {
                                if Instant::now() >= deadline {
                                    let _ = child.kill();
                                    return r#"{"exit_code":-1,"stdout":"","stderr":"timeout"}"#.to_string();
                                }
                                std::thread::sleep(Duration::from_millis(50));
                            }
                            Err(e) => {
                                return format!(
                                    r#"{{"exit_code":-1,"stdout":"","stderr":"{}"}}"#,
                                    json_escape(&e.to_string())
                                );
                            }
                        }
                    }
                }
                Err(e) => format!(
                    r#"{{"exit_code":-1,"stdout":"","stderr":"{}"}}"#,
                    json_escape(&e.to_string())
                ),
            }
        } else {
            let output = make_cmd(cmd).output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    let code = o.status.code().unwrap_or(-1);
                    format!(
                        r#"{{"exit_code":{},"stdout":"{}","stderr":"{}"}}"#,
                        code, json_escape(&stdout), json_escape(&stderr)
                    )
                }
                Err(e) => format!(
                    r#"{{"exit_code":-1,"stdout":"","stderr":"{}"}}"#,
                    json_escape(&e.to_string())
                ),
            }
        };
        result
    }
}

// =============================================================================
// Regex module for a2r transpiler
// =============================================================================

#[allow(non_snake_case)]
pub mod re {
    pub fn r#match(pattern: &str, text: &str) -> i32 {
        match ::regex::Regex::new(pattern) {
            Ok(re) => if re.is_match(text) { 1 } else { 0 },
            Err(_) => 0,
        }
    }

    pub fn find_all(pattern: &str, text: &str) -> String {
        match ::regex::Regex::new(pattern) {
            Ok(re) => {
                let matches: Vec<String> = re.find_iter(text).map(|m| format!("\"{}\"", m.as_str())).collect();
                if matches.is_empty() { "[]".to_string() } else { format!("[{}]", matches.join(",")) }
            }
            Err(_) => "[]".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_basic() {
        let list = List::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(1));
        assert_eq!(list.get(1), Some(2));
        assert_eq!(list.get(2), Some(3));
    }

    #[test]
    fn test_list_pop() {
        let list: List<i32> = List::new();
        list.push(1);
        list.push(2);

        assert_eq!(list.pop(), Some(2));
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn test_may() {
        let some: May<i32> = Some(42);
        let none: May<i32> = None;

        assert_eq!(some.unwrap_or(0), 42);
        assert_eq!(none.unwrap_or(0), 0);
    }
}
