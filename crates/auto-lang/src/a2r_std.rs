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

/// Extension trait to provide stable `as_str()` for String/&str.
/// In Rust 1.93, String::as_str() is stable but str::as_str() is still unstable (E0658).
/// This trait provides a stable alternative that works for both types.
#[allow(unstable_name_collisions)]
pub trait StringAsStr {
    fn as_str(&self) -> &str;
}

impl StringAsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

impl StringAsStr for str {
    fn as_str(&self) -> &str {
        self
    }
}

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
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                self.len(),
                i
            );
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

// ============================================================================
// StringBuilder — runtime support for the Auto VM `StringBuilder` type.
// ============================================================================
//
// The Auto VM exposes `StringBuilder` as a built-in type with the methods
// `new(capacity)`, `append(str)`, `append_char(code)`, `build()`, `len()`, and
// `clear()`. The a2r transpiler emits references to this type verbatim into
// transpiled Rust (resolved as `a2r_std::StringBuilder` via the glob import the
// transpiler prepends), so this module provides a Rust-native implementation
// with the identical API surface.
//
// Auto's `append_char(code)` accepts a Unicode code point expressed as an `i32`
// (matching the VM's int-based char representation), so `append_char` takes an
// `i32` argument here and converts it to a `char` internally.

/// A growable, owned string builder mirroring the Auto VM `StringBuilder` type.
#[derive(Debug, Clone)]
pub struct StringBuilder {
    buffer: String,
}

impl StringBuilder {
    /// Create a new, empty `StringBuilder` with the given reserved capacity.
    ///
    /// Matches Auto's `StringBuilder.new(capacity)`. The capacity is an `i32`
    /// in Auto; non-positive values are treated as "use default capacity".
    pub fn new(capacity: i32) -> Self {
        let cap = if capacity > 0 { capacity as usize } else { 0 };
        StringBuilder {
            buffer: String::with_capacity(cap),
        }
    }

    /// Append a string slice to the buffer.
    ///
    /// Matches Auto's `sb.append(s)`.
    pub fn append(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Append a single character given as a Unicode code point (`i32`).
    ///
    /// Matches Auto's `sb.append_char(code)`. Invalid code points or
    /// out-of-range values are replaced with U+FFFD REPLACEMENT CHARACTER,
    /// mirroring the VM's defensive behaviour.
    pub fn append_char(&mut self, code: i32) {
        let c = char::from_u32(code as u32).unwrap_or('\u{FFFD}');
        self.buffer.push(c);
    }

    /// Return the accumulated `String` without consuming the builder.
    ///
    /// Matches Auto's `sb.build()`: the VM StringBuilder is NOT consumed by
    /// build() — the same builder can be built again later. Plan 368 (consumer-
    /// mode parity): take `&self` + clone so the a2r backend mirrors the VM's
    /// non-consuming semantics (taking `self` moves the builder and breaks any
    /// `.at` source that builds more than once or build()s after a conditional
    /// append path). Kept in sync with crates/a2r-std/src/string_builder.rs.
    pub fn build(&self) -> String {
        self.buffer.clone()
    }

    /// Return the current length of the buffer in bytes.
    ///
    /// Matches Auto's `sb.len()`.
    pub fn len(&self) -> i32 {
        self.buffer.len() as i32
    }

    /// Return whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clear the buffer, leaving it empty with its capacity retained.
    ///
    /// Matches Auto's `sb.clear()`.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for StringBuilder {
    fn default() -> Self {
        StringBuilder {
            buffer: String::new(),
        }
    }
}

/// Nil - AutoLang's nil value type marker
pub struct Nil;

/// Create a Nil value (None)
pub fn nil<T>() -> Option<T> {
    None
}

/// PLAN-710 G-B: catch (e) 绑定形的 panic 载荷消息串——VM shim catch 帧
/// 推入错误消息字符串（vm/codegen.rs Stmt::Try 注记），a2r catch_unwind
/// 臂以同源字符串绑定。未知载荷形退化为 "panic"（字符串化边界）。
pub fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "panic".to_string()
    }
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
        val.get(idx)
            .unwrap_or_else(|| NULL_VALUE.get_or_init(|| Value::Null))
    }

    pub fn get<'a>(val: &'a Value, key: &str) -> &'a Value {
        static NULL_VALUE: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
        val.get(key)
            .unwrap_or_else(|| NULL_VALUE.get_or_init(|| Value::Null))
    }

    pub fn get_owned(val: &Value, key: &str) -> Value {
        val.get(key).cloned().unwrap_or(Value::Null)
    }

    // PLAN-710 G-A: envelope 成员访问投影族（`v.field ?? default` 的
    // a2r 发射形）。语义与 VM 轨同源：路径缺段/字段缺席/类型不符 →
    // ?? 右值缺省（VM 侧缺字段 → Null → coalesce 右值的防御性收敛）。
    // 路径为成员名切片（嵌套链 `v.a.b ?? d` → keys=&["a","b"]）；
    // 索引混合形由生成器以 get_owned/at_owned 逐段组合后落最终投影。

    /// 带缺省的成员路径 str 投影（字面量缺省形）。
    pub fn get_str_or(val: &Value, keys: &[&str], default: &str) -> String {
        get_str_or_with(val, keys, || default.to_string())
    }

    /// 带缺省的成员路径 str 投影（计算缺省形——`t.title ?? file_basename(p)`）。
    pub fn get_str_or_with<F: FnOnce() -> String>(
        val: &Value,
        keys: &[&str],
        default: F,
    ) -> String {
        let mut cur = val;
        for k in keys {
            match cur.get(k) {
                Some(v) => cur = v,
                None => return default(),
            }
        }
        cur.as_str().map(|s| s.to_string()).unwrap_or_else(default)
    }

    /// 带缺省的成员路径 int 投影（.at int = i32 发射位再 `as i32`）。
    pub fn get_int_or(val: &Value, keys: &[&str], default: i64) -> i64 {
        let mut cur = val;
        for k in keys {
            match cur.get(k) {
                Some(v) => cur = v,
                None => return default,
            }
        }
        cur.as_i64().unwrap_or(default)
    }

    /// 带缺省的成员路径 bool 投影。
    pub fn get_bool_or(val: &Value, keys: &[&str], default: bool) -> bool {
        let mut cur = val;
        for k in keys {
            match cur.get(k) {
                Some(v) => cur = v,
                None => return default,
            }
        }
        cur.as_bool().unwrap_or(default)
    }

    /// 成员路径 list 投影（`?? []` 缺省 = 空 Vec<Value>）。
    pub fn get_array_or(val: &Value, keys: &[&str]) -> Vec<Value> {
        let mut cur = val;
        for k in keys {
            match cur.get(k) {
                Some(v) => cur = v,
                None => return Vec::new(),
            }
        }
        cur.as_array().cloned().unwrap_or_default()
    }

    /// 数组位取值（`rows[i]` 混合链段——越界 → Null，与缺字段同收敛）。
    pub fn at_owned(val: &Value, idx: usize) -> Value {
        val.get(idx).cloned().unwrap_or(Value::Null)
    }

    /// PLAN-710 D-6：条件位 Value 裸读的真值投影——VM 轨 `Value::is_true`
    /// 同源语义（auto_val value.rs:682：Bool 直读 / 数值 >0 / 串非空 /
    /// 其余 false）。view/handler 条件位（`if r.ix_del` 族）的编译轨
    /// 等价形。
    pub fn truthy(val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().map_or(false, |f| f > 0.0),
            Value::String(s) => !s.is_empty(),
            _ => false,
        }
    }

    pub fn get_str(val: &Value, key: &str) -> String {
        val.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn as_string(val: &Value) -> String {
        val.as_str().map(|s| s.to_string()).unwrap_or_default()
    }

    pub fn as_string_opt(val: Option<&Value>) -> String {
        val.and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    }

    /// PLAN-714 r3 R3-T1: json.from_value（VM 同语义——struct/map 值序列
    /// 化为 JSON 文本；发射臂经 a2r_std::json! 宏构造 Value 后入此）。
    pub fn from_value(val: Value) -> String {
        val.to_string()
    }
    /// PLAN-714 r3 R3-T2: json.to_value 的 list 语义面（VM json.to_value
    /// 返回 list——corpus 消费形=str 元素清单[路径族]，a2r 轨=Vec<String>
    /// 供 for 直迭代；非 str 元素 as_str 兜空串。int 清单形=242 边界注记）。
    pub fn parse_str_list(text: &str) -> Vec<String> {
        serde_json::from_str::<Vec<Value>>(text)
            .unwrap_or_default()
            .iter()
            .map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_default())
            .collect()
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
pub fn str_find<S: AsRef<str>>(s: S, needle: &str, start: i32) -> i32 {
    let s = s.as_ref();
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
pub fn str_substr<S: AsRef<str>>(s: S, start: i32, length: i32) -> String {
    let s = s.as_ref();
    if start < 0 || length <= 0 || start as usize >= s.len() {
        return String::new();
    }
    let start_usize = start as usize;
    let end = std::cmp::min(start_usize + length as usize, s.len());
    s[start_usize..end].to_string()
}

/// String contains check.
/// In AutoLang: s.contains(needle)
pub fn str_contains<S: AsRef<str>>(s: S, needle: &str) -> bool {
    s.as_ref().contains(needle)
}

/// String ends with check. Returns bool for Rust compat.
/// In AutoLang: s.ends_with(suffix)
pub fn str_ends_with<S: AsRef<str>>(s: S, suffix: &str) -> bool {
    s.as_ref().ends_with(suffix)
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
    }
    .unwrap_or(0)
}

/// Get the length of a JSON value (string length for strings, 0 for other types)
pub fn value_len(val: &serde_json::Value) -> i32 {
    if let Some(s) = val.as_str() {
        s.len() as i32
    } else {
        0
    }
}

// PLAN-714 r3 R3-T1: json! 宏再导出——发射臂发 a2r_std::json!({...})
// 构造 struct-literal 实参（宏与 json 模块分属不同命名空间，无碰撞）。
pub use serde_json::json;

// PLAN-714 r3 R3-T1: `.at` 裸 `list` 动态型的 a2r 形（Vec<Value>——
// glob 导入下裸名直接解析；VM list 动态语义的 serde 对应面）。
#[allow(non_camel_case_types)]
pub type list = Vec<serde_json::Value>;

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
                let trimmed = buf
                    .trim_end_matches('\n')
                    .trim_end_matches('\r')
                    .to_string();
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
    pub fn get(key: impl AsRef<str>) -> String {
        std::env::var(key.as_ref()).unwrap_or_default()
    }

    pub fn get_or(key: impl AsRef<str>, default: impl AsRef<str>) -> String {
        std::env::var(key.as_ref()).unwrap_or_else(|_| default.as_ref().to_string())
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
#[allow(non_snake_case)] // AutoLang API uses PascalCase `File`
pub mod File {
    pub use super::fs::*;
}

/// AutoLang's fs module — thin wrappers around std::fs
#[allow(non_snake_case)]
pub mod fs {
    pub fn read_to_string(path: &str) -> String {
        std::fs::read_to_string(path).unwrap_or_default()
    }

    // PLAN-681: tree/basename host in the standalone a2r-std crate (promoted
    // to a real dependency). Re-exported here so both generated spellings —
    // `a2r_std::fs::tree` and the qualify_a2r_std-post-processed
    // `auto_lang::a2r_std::fs::tree` — resolve to the ONE implementation
    // (tests/fs_tree_parity.rs pins the VM byte parity).
    pub use a2r_std::fs::{basename, tree};
    // PLAN-687: chunked-read envelope re-export (trans File 表映射发射
    // auto_lang::a2r_std::fs::read_text_range —— facade 面)。
    pub use a2r_std::fs::read_text_range;

    pub fn read_text<S: AsRef<str>>(path: S) -> String {
        std::fs::read_to_string(path.as_ref()).unwrap_or_default()
    }

    pub fn write(path: impl AsRef<str>, content: impl AsRef<str>) -> bool {
        std::fs::write(path.as_ref(), content.as_ref()).is_ok()
    }

    pub fn exists(path: impl AsRef<str>) -> bool {
        std::path::Path::new(path.as_ref()).exists()
    }

    pub fn create_dir(path: impl AsRef<str>) -> bool {
        std::fs::create_dir_all(path.as_ref()).is_ok()
    }

    pub fn write_text(path: impl AsRef<str>, content: impl AsRef<str>) -> bool {
        std::fs::write(path.as_ref(), content.as_ref()).is_ok()
    }

    pub fn read_bytes(path: impl AsRef<str>) -> Vec<u8> {
        std::fs::read(path.as_ref()).unwrap_or_default()
    }

    pub fn delete(path: impl AsRef<str>) -> bool {
        std::fs::remove_file(path.as_ref()).is_ok()
    }

    pub fn is_dir(path: impl AsRef<str>) -> bool {
        std::path::Path::new(path.as_ref()).is_dir()
    }

    pub fn is_binary(path: impl AsRef<str>) -> i32 {
        match std::fs::read(path.as_ref()) {
            Ok(bytes) => {
                if bytes.windows(2).any(|w| w == [0, 0]) {
                    1
                } else {
                    0
                }
            }
            Err(_) => 0,
        }
    }

    pub fn file_size(path: impl AsRef<str>) -> i64 {
        match std::fs::metadata(path.as_ref()) {
            Ok(meta) => meta.len() as i64,
            Err(_) => -1,
        }
    }

    /// PLAN-714 r3 R3-T1: fs.copy_recursive（VM 内建 2862 同语义——静默
    /// bool：目录递归复制、文件单发 std::fs::copy；任一步失败即 false）。
    pub fn copy_recursive(src: impl AsRef<str>, dst: impl AsRef<str>) -> bool {
        fn rec(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
            if src.is_dir() {
                std::fs::create_dir_all(dst)?;
                for e in std::fs::read_dir(src)? {
                    let e = e?;
                    rec(&e.path(), &dst.join(e.file_name()))?;
                }
                Ok(())
            } else {
                std::fs::copy(src, dst).map(|_| ())
            }
        }
        rec(
            std::path::Path::new(src.as_ref()),
            std::path::Path::new(dst.as_ref()),
        )
        .is_ok()
    }

    /// PLAN-714 r3 R3-T1: File.read_bytes 的 list 形（VM read_bytes→int
    /// list 同源——a2r 轨裸 `list` 动态型=Vec<Value>，元素=Int 值）。
    pub fn read_bytes_list(path: impl AsRef<str>) -> Vec<serde_json::Value> {
        std::fs::read(path.as_ref())
            .unwrap_or_default()
            .into_iter()
            .map(|b| serde_json::Value::from(b as i64))
            .collect()
    }

    /// PLAN-714 r3 R3-T1: fs.remove_dir（VM File.remove_dir 同语义——
    /// 单层 remove_dir，非 _all）。
    pub fn remove_dir(path: impl AsRef<str>) -> bool {
        std::fs::remove_dir(path.as_ref()).is_ok()
    }

    /// PLAN-714 r3 R3-T1: File.write_bytes 的 list 形（read_bytes_list
    /// 对偶——Value 元素按 as_i64 取低字节；缺失补 0 保长度）。
    pub fn write_bytes_list(path: impl AsRef<str>, data: Vec<serde_json::Value>) -> bool {
        let bytes: Vec<u8> = data.iter().map(|v| v.as_i64().unwrap_or(0) as u8).collect();
        std::fs::write(path.as_ref(), bytes).is_ok()
    }

    pub fn walk(dir: impl AsRef<str>) -> String {
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
        do_walk(dir.as_ref(), &mut entries);
        if entries.is_empty() {
            "[]".to_string()
        } else {
            format!("[{}]", entries.join(","))
        }
    }

    pub fn append_text(path: impl AsRef<str>, content: impl AsRef<str>) {
        use std::fs::OpenOptions;
        use std::io::Write;
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())
        {
            let _ = file.write_all(content.as_ref().as_bytes());
        }
    }
}

/// Parse ~/.claude/settings.json into (api_key, base_url, vars HashMap)
/// Returns (None, None, empty_map) if parsing fails
pub fn parse_settings_json(
    text: &str,
) -> (
    Option<String>,
    Option<String>,
    std::collections::HashMap<String, String>,
) {
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
    let api_key = vars
        .get("ANTHROPIC_API_KEY")
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
///
/// PLAN-724 T-05：网络执行单源化——本模块全部请求改走独立 a2r-std crate
/// 的共享 async 内核（`::a2r_std::http::client`，reqwest async + 固定
/// runtime + 有界准入 + owned typed 结果）。reqwest::blocking、
/// spawn_blocking 与每请求线程全部退役。历史签名（认证 tuple 形状、
/// last_status 线程局部）逐字节保留；线程局部状态只作同步兼容信息，
/// 异步消费使用调用方 own 的返回值（不依赖 TLS 关联并发请求）。
#[allow(non_snake_case)]
pub mod http {
    use ::a2r_std::http::client::{self, ClientError, HttpRequest};

    // PLAN-727 T-06：文件传输公共面——本 facade 是**转发壳**，执行全部走
    // 独立 crate 的共享传输核心（a2r_std::http::transfer）；独立转译产物
    // 直发 `a2r_std::http::transfer_*`（同词汇同形）。
    pub use ::a2r_std::http::{
        cancel_transfer_by_id, transfer_cancel, transfer_download, transfer_error,
        transfer_next_progress, transfer_upload, transfer_wait, transfer_wait_async,
        transfer_wait_typed, FileTransfer, TransferObserver,
    };

    // PLAN-729 T-02：服务端文件响应公共面——同形转发（描述符 + 纯协议
    // 决策在 a2r_std::http::server_file；宿主执行在
    // crate::http_file_service，供 VM 与生成 Rust 两腿共用）。
    pub use ::a2r_std::http::{
        file_response, FileInitError, FileInitErrorKind, FileProtocolDecision, FileRangePlan,
        FileRequestConditions, FileResponse, FileResponseOptions, RepresentationValidators,
    };

    // PLAN-736 T-02：服务端上传公共面——同形转发（730 上传 glue 生成后经
    // api_gen qualify_a2r_std 落到 auto_lang::a2r_std::http；此前缺此转发
    // 使生成上传后端 E0425——727 transfer/729 file 壳均有、upload 漏网）。
    pub use ::a2r_std::http::{
        cancel_upload_session, failed_session, install_upload_executor,
        parse_upload_receive_options, upload_commit, upload_error, upload_metadata,
        upload_metadata_json, upload_receive, upload_reject, upload_request_from_parts,
        UploadBodyStream, UploadErrorKind, UploadExecutor, UploadPhase, UploadPhaseHook,
        UploadReceipt, UploadReceiveMode, UploadReceiveOptions, UploadReceivedMeta, UploadRequest,
        UploadServeLimits, UploadSession, UploadSessionState,
    };

    fn auth_request_json(url: &str, body: &str, api_key: &str, bearer: bool) -> HttpRequest {
        let mut req = HttpRequest::new("POST", url);
        req.headers
            .push(("content-type".into(), "application/json".into()));
        if bearer {
            req.headers
                .push(("Authorization".into(), format!("Bearer {}", api_key)));
        } else {
            req.headers.push(("x-api-key".into(), api_key.to_string()));
            req.headers
                .push(("anthropic-version".into(), "2023-06-01".into()));
        }
        req.body = Some(body.as_bytes().to_vec());
        req
    }

    fn error_message(e: &ClientError) -> String {
        match e {
            ClientError::Transport(m) => m.clone(),
            other => format!("{other}"),
        }
    }

    /// Async HTTP POST with Anthropic API headers.
    /// Returns (status, body, error, kind) for constructing a local HttpResponse.
    /// 内核 async 执行：排队/建立/读体全程让出执行线程；丢弃 future 即取消。
    pub async fn post(url: &str, body: &str, api_key: &str) -> (i32, String, String, String) {
        let req = auth_request_json(url, body, api_key, false);
        match client::execute(req).await {
            Ok(resp) => {
                let status = resp.status as i32;
                let resp_body = String::from_utf8_lossy(&resp.body).into_owned();
                if (200..300).contains(&status) {
                    (status, resp_body, String::new(), "ok".to_string())
                } else {
                    (
                        status,
                        resp_body,
                        format!("HTTP {}", status),
                        "error".to_string(),
                    )
                }
            }
            Err(e) => (0, String::new(), error_message(&e), "error".to_string()),
        }
    }

    /// Synchronous HTTP POST — blocking version for use in non-async contexts.
    /// Used by Auto's http.post_sync() when transpiled via a2r.
    /// 同步桥接边界：仅在允许阻塞的边界调用；async 上下文请用 [`post`]。
    pub fn post_sync(
        url: impl AsRef<str>,
        body: impl AsRef<str>,
        api_key: impl AsRef<str>,
    ) -> (i32, String) {
        let req = auth_request_json(url.as_ref(), body.as_ref(), api_key.as_ref(), false);
        match client::execute_blocking(req) {
            Ok(resp) => (
                resp.status as i32,
                String::from_utf8_lossy(&resp.body).into_owned(),
            ),
            Err(e) => (0, error_message(&e)),
        }
    }

    /// Async HTTP POST with Bearer token auth (for OpenAI-compatible APIs).
    /// Returns (status, body).
    pub async fn post_bearer(
        url: impl AsRef<str>,
        body: impl AsRef<str>,
        api_key: impl AsRef<str>,
    ) -> (i32, String) {
        let req = auth_request_json(url.as_ref(), body.as_ref(), api_key.as_ref(), true);
        match client::execute(req).await {
            Ok(resp) => (
                resp.status as i32,
                String::from_utf8_lossy(&resp.body).into_owned(),
            ),
            Err(e) => (0, format!("HTTP error: {}", error_message(&e))),
        }
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
    pub fn post_bearer_sync(
        url: impl AsRef<str>,
        body: impl AsRef<str>,
        api_key: impl AsRef<str>,
    ) -> (i32, String) {
        let req = auth_request_json(url.as_ref(), body.as_ref(), api_key.as_ref(), true);
        match client::execute_blocking(req) {
            Ok(resp) => (
                resp.status as i32,
                String::from_utf8_lossy(&resp.body).into_owned(),
            ),
            Err(e) => (0, format!("HTTP error: {}", error_message(&e))),
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
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

// =============================================================================
// Shell module for a2r transpiler
// =============================================================================

#[allow(non_snake_case)]
pub mod shell {
    fn json_escape(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "")
    }

    pub fn exec(cmd: impl AsRef<str>, timeout_ms: i32) -> String {
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
            let mut child = make_cmd(cmd.as_ref())
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
                                    code,
                                    json_escape(&stdout),
                                    json_escape(&stderr)
                                );
                            }
                            Ok(None) => {
                                if Instant::now() >= deadline {
                                    let _ = child.kill();
                                    return r#"{"exit_code":-1,"stdout":"","stderr":"timeout"}"#
                                        .to_string();
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
            let output = make_cmd(cmd.as_ref()).output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    let code = o.status.code().unwrap_or(-1);
                    format!(
                        r#"{{"exit_code":{},"stdout":"{}","stderr":"{}"}}"#,
                        code,
                        json_escape(&stdout),
                        json_escape(&stderr)
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
            Ok(re) => {
                if re.is_match(text) {
                    1
                } else {
                    0
                }
            }
            Err(_) => 0,
        }
    }

    /// PLAN-714 r3 R3-T1: Regex.test→bool（VM 实参序 (text, pattern)
    /// 同源——stdlib.rs P-053-6 Regex.test(text, pat)）。
    pub fn test(text: &str, pattern: &str) -> bool {
        match ::regex::Regex::new(pattern) {
            Ok(re) => re.is_match(text),
            Err(_) => false,
        }
    }

    /// PLAN-714 r3 R3-T1: Regex.replace（VM shim_regex_replace 同语义——
    /// flags 含 g=replace_all 否则首替；replacement 直传 as_str 同 $ 语义）。
    pub fn replace(text: &str, pattern: &str, replacement: &str, flags: &str) -> String {
        match ::regex::Regex::new(pattern) {
            Ok(re) => {
                if flags.contains('g') {
                    re.replace_all(text, replacement).into_owned()
                } else {
                    re.replace(text, replacement).into_owned()
                }
            }
            Err(_) => text.to_string(),
        }
    }

    pub fn find_all(pattern: &str, text: &str) -> String {
        match ::regex::Regex::new(pattern) {
            Ok(re) => {
                let matches: Vec<String> = re
                    .find_iter(text)
                    .map(|m| format!("\"{}\"", m.as_str()))
                    .collect();
                if matches.is_empty() {
                    "[]".to_string()
                } else {
                    format!("[{}]", matches.join(","))
                }
            }
            Err(_) => "[]".to_string(),
        }
    }
}

// =============================================================================
// Frame module for a2r transpiler（PLAN-716 组B——供②帧时间戳观测通道，
// VM native 9918/9919 同源；AUTO_FRAME_BENCH 门控在通道层，双轨同源读）
// =============================================================================

/// AutoLang's frame module — 帧时间戳只读观测（单调毫秒；未捕获=0）。
pub mod frame {
    /// PLAN-725 T-06 供⑬：a2r 臂值域收窄 i64→i32（saturating——SD-B §3b
    /// 口径：毫秒值域 <2^31≈24.8 天，进程起点起算单调时源在值域内）。
    /// 此前 i64 直赋 .at `int`（i32）字段 → E0308 mismatched types（下游
    /// a2r regen 实录）；VM 轨 9918/9919 lane 不受影响（i64 内部态）。
    fn narrow_i32(ms: i64) -> i32 {
        ms.clamp(i32::MIN as i64, i32::MAX as i64) as i32
    }

    /// 帧开始时间戳（桌面 update 入口到达时刻）。
    pub fn begin_ms() -> i32 {
        narrow_i32(crate::ui::frame_bench::frame_begin_ms())
    }

    /// 呈现完成时间戳（711 帧泵消费时刻，≥ present 返回）。
    pub fn present_ms() -> i32 {
        narrow_i32(crate::ui::frame_bench::frame_present_ms())
    }
}

// =============================================================================
// Diff module for a2r transpiler（PLAN-714 r3 R3-T1——703 imara 引擎包络，
// VM native 9915/9916/9917 同源；code-editor 门双轨——feature 关=panic
// 同 shim fallback 形[native.rs 非门控段同文案]）
// =============================================================================

/// AutoLang's diff module — envelope JSON wrappers over the diff engine.
#[allow(non_snake_case)]
pub mod diff {
    #[cfg(feature = "code-editor")]
    pub fn diff_files(path_a: &str, path_b: &str, ctx: i64) -> String {
        crate::ui::code_editor::diff::envelope::diff_files_envelope_from_paths(
            path_a,
            path_b,
            ctx.max(0) as usize,
        )
    }

    #[cfg(not(feature = "code-editor"))]
    pub fn diff_files(_path_a: &str, _path_b: &str, _ctx: i64) -> String {
        panic!("diff_files: the `code-editor` feature is disabled")
    }

    /// 窗口投影形（PLAN-716 组C——rows_total+truncated 激活；offset/limit
    /// 钳非负）。
    #[cfg(feature = "code-editor")]
    pub fn diff_files_window(
        path_a: &str,
        path_b: &str,
        ctx: i64,
        rows_offset: i64,
        rows_limit: i64,
    ) -> String {
        crate::ui::code_editor::diff::envelope::diff_files_envelope_from_paths_window(
            path_a,
            path_b,
            ctx.max(0) as usize,
            rows_offset.max(0) as usize,
            rows_limit.max(0) as usize,
        )
    }

    #[cfg(feature = "code-editor")]
    pub fn diff_dirs(path_a: &str, path_b: &str) -> String {
        crate::ui::code_editor::diff::envelope::diff_dirs_envelope(path_a, path_b)
    }

    #[cfg(not(feature = "code-editor"))]
    pub fn diff_dirs(_path_a: &str, _path_b: &str) -> String {
        panic!("diff_dirs: the `code-editor` feature is disabled")
    }

    #[cfg(feature = "code-editor")]
    pub fn diff_snapshots(key_a: &str, key_b: &str) -> String {
        crate::ui::code_editor::diff::envelope::diff_snapshots_envelope(key_a, key_b)
    }

    #[cfg(not(feature = "code-editor"))]
    pub fn diff_snapshots(_key_a: &str, _key_b: &str) -> String {
        panic!("diff_snapshots: the `code-editor` feature is disabled")
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
