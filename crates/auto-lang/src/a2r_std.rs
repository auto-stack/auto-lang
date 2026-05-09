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
        // Get the value using get() which returns Option<T>
        // We clone it and leak it to get a static reference
        // This is a workaround for RefCell's borrowing limitations
        // SAFETY: The reference is valid for the lifetime of the program
        // In practice, this should only be used for short-lived indexing operations
        if let Some(val) = self.get(i) {
            // Leak the value to get a static reference
            // This is memory-safe but leaks memory - acceptable for examples
            Box::leak(Box::new(val))
        } else {
            panic!("index out of bounds: the len is {} but the index is {}", self.len(), i);
        }
    }
}

impl<T: Clone> std::ops::IndexMut<usize> for List<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        // With &mut self, we have exclusive access
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
pub mod Json {
    use serde_json::Value;

    pub fn is_valid(s: &str) -> bool {
        serde_json::from_str::<Value>(s).is_ok()
    }

    pub fn parse(s: &str) -> Option<Value> {
        serde_json::from_str(s).ok()
    }

    pub fn get_at(val: &Value, idx: usize) -> Option<Value> {
        val.get(idx).cloned()
    }

    pub fn get<'a>(val: &'a Value, key: &str) -> Option<&'a Value> {
        val.get(key)
    }

    pub fn get_str(val: &Value, key: &str) -> String {
        val.get(key).and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default()
    }

    pub fn as_string(val: &Value) -> Option<String> {
        val.as_str().map(|s| s.to_string())
    }

    pub fn to_string(val: &Value) -> String {
        serde_json::to_string(val).unwrap_or_default()
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

// =============================================================================
// Environment module for a2r transpiler
// =============================================================================

/// AutoLang's env module — thin wrappers around std::env
#[allow(non_snake_case)]
pub mod env {
    pub fn get(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    pub fn set(key: &str, val: &str) {
        std::env::set_var(key, val);
    }
}

// =============================================================================
// File system module for a2r transpiler
// =============================================================================

/// AutoLang's fs module — thin wrappers around std::fs
#[allow(non_snake_case)]
pub mod fs {
    pub fn read_to_string(path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    pub fn write(path: &str, content: &str) -> bool {
        std::fs::write(path, content).is_ok()
    }

    pub fn exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    pub fn create_dir(path: &str) -> bool {
        std::fs::create_dir_all(path).is_ok()
    }
}

// =============================================================================
// Utility functions for a2r transpiler
// =============================================================================

/// Sleep for the specified number of milliseconds
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
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
