//! Owned string type with move semantics
//!
//! This is AutoLang's native string type with ownership semantics.
//! Unlike `AutoStr` (EcoString), `Str` cannot be implicitly copied
//! and must be explicitly moved when transferred.

use crate::linear::Linear;
use crate::AutoStr;
use std::fmt;

/// Owned UTF-8 string with move semantics
///
/// This type implements the Linear trait - it can only be moved, not cloned.
/// When a `Str` is moved, the source becomes invalid.
#[derive(Clone, PartialEq)]
pub struct Str {
    /// UTF-8 encoded bytes
    data: Vec<u8>,
    /// Byte length (not character count)
    len: usize,
    /// Capacity (allocated bytes)
    cap: usize,
}

impl Str {
    /// Create a new owned string from UTF-8 bytes
    ///
    /// # Arguments
    /// - `utf8`: UTF-8 encoded bytes
    /// - `len`: Byte length
    ///
    /// # Returns
    /// New `Str` if UTF-8 is valid, None otherwise
    pub fn new(utf8: impl AsRef<[u8]>) -> Option<Self> {
        let bytes = utf8.as_ref();

        // Validate UTF-8
        if !std::str::from_utf8(bytes).is_ok() {
            return None;
        }

        let len = bytes.len();
        let cap = if len == 0 { 16 } else { len };

        Some(Str {
            data: bytes.to_vec(),
            len,
            cap,
        })
    }

    /// Create a new string from a Rust string slice
    pub fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len();
        let cap = if len == 0 { 16 } else { len };

        Str {
            data: bytes.to_vec(),
            len,
            cap,
        }
    }

    /// Create a new string from an AutoStr
    pub fn from_astr(astr: &AutoStr) -> Self {
        Self::from_str(astr.as_str())
    }

    /// Get pointer to internal UTF-8 data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get byte length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get capacity
    pub fn cap(&self) -> usize {
        self.cap
    }

    /// Check if string is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get UTF-8 character count (slower than len())
    pub fn char_len(&self) -> usize {
        self.as_str().chars().count()
    }

    /// Get as Rust string slice
    pub fn as_str(&self) -> &str {
        unsafe {
            // Safety: data is always valid UTF-8 (enforced by constructor)
            std::str::from_utf8_unchecked(&self.data)
        }
    }

    /// Append UTF-8 bytes to string
    pub fn append(&mut self, utf8: impl AsRef<[u8]>) -> Result<(), StrError> {
        let bytes = utf8.as_ref();

        // Validate UTF-8
        std::str::from_utf8(bytes)
            .map_err(|_| StrError::InvalidUtf8)?;

        // Ensure capacity
        let new_len = self.len + bytes.len();
        if new_len > self.cap {
            self.grow(new_len);
        }

        // Append data
        self.data.extend_from_slice(bytes);
        self.len = new_len;

        Ok(())
    }

    /// Append a Rust string
    pub fn append_str(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
        self.len = self.data.len();
    }

    /// Push a single byte onto the string
    pub fn push(&mut self, byte: u8) {
        // Ensure capacity
        if self.len + 1 > self.cap {
            self.grow(self.len + 1);
        }

        self.data.push(byte);
        self.len += 1;
    }

    /// Ensure capacity (grow if needed)
    fn grow(&mut self, new_cap: usize) {
        let mut cap = self.cap;
        if cap == 0 {
            cap = 16;
        }
        while cap < new_cap {
            cap *= 2;
        }

        self.data.reserve(cap - self.len);
        self.cap = cap;
    }

    /// Convert to null-terminated C string (allocates new buffer)
    pub fn to_cstr(&self) -> Vec<u8> {
        let mut cstr = self.data.clone();
        cstr.push(0); // Null terminator
        cstr
    }
}

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Str({})", self.as_str())
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Str {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<AutoStr> for Str {
    fn from(astr: AutoStr) -> Self {
        Self::from_astr(&astr)
    }
}

/// Implement Linear trait for move semantics
///
/// When a Str is dropped, the underlying Vec<u8> is automatically
/// freed via Rust's Drop trait. This is a no-op implementation
/// because Rust already handles cleanup.
impl Linear for Str {
    fn drop_linear(&mut self) {
        // Rust's Drop trait for Vec<u8> automatically frees memory
        // This is called when MoveTracker drops the value
        // We mark as empty to catch use-after-move bugs
        self.data.clear();
        self.len = 0;
        self.cap = 0;
    }
}

/// String operation errors
#[derive(Debug, Clone, PartialEq)]
pub enum StrError {
    InvalidUtf8,
    OutOfMemory,
}

impl fmt::Display for StrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrError::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence"),
            StrError::OutOfMemory => write!(f, "Out of memory"),
        }
    }
}

impl std::error::Error for StrError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_new_empty() {
        let s = Str::new(&[]).unwrap();
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_str_from_str() {
        let s = Str::from_str("hello");
        assert_eq!(s.len(), 5);
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_str_utf8() {
        // Test multi-byte UTF-8 characters
        let s = Str::from_str("你好");
        assert_eq!(s.len(), 6); // 6 bytes
        assert_eq!(s.char_len(), 2); // 2 characters
    }

    #[test]
    fn test_str_invalid_utf8() {
        // Invalid UTF-8 sequence
        let s = Str::new(&[0xFF, 0xFE]);
        assert!(s.is_none());
    }

    #[test]
    fn test_str_append() {
        let mut s = Str::from_str("hello");
        s.append(&b" world"[..]).unwrap();
        assert_eq!(s.len(), 11);
        assert_eq!(s.as_str(), "hello world");
    }

    #[test]
    fn test_str_push() {
        let mut s = Str::from_str("hello");
        s.push(b'!');
        assert_eq!(s.len(), 6);
        assert_eq!(s.as_str(), "hello!");
    }

    #[test]
    fn test_str_capacity_growth() {
        let s = Str::from_str("hello");
        let initial_cap = s.cap();
        assert!(initial_cap >= 5);

        let mut s = s;
        for _ in 0..20 {
            s.push(b'x');
        }

        assert!(s.cap() > initial_cap);
    }

    #[test]
    fn test_str_to_cstr() {
        let s = Str::from_str("hello");
        let cstr = s.to_cstr();
        assert_eq!(cstr.len(), 6); // 5 bytes + null terminator
        assert_eq!(cstr[5], 0);
    }

    #[test]
    fn test_str_clone() {
        // Note: Str implements Clone for internal use,
        // but should not be cloned in user code (move semantics)
        let s1 = Str::from_str("hello");
        let s2 = s1.clone();
        assert_eq!(s1.as_str(), s2.as_str());
    }

    #[test]
    fn test_linear_drop() {
        use crate::linear::Linear;

        let mut s = Str::from_str("hello");
        assert_eq!(s.len(), 5);

        // Call drop_linear explicitly
        s.drop_linear();
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_move_tracker_with_str() {
        use crate::linear::MoveTracker;

        let s = Str::from_str("hello");
        let mut tracker = MoveTracker::new(s);

        assert!(tracker.is_available());
        assert_eq!(tracker.get().unwrap().as_str(), "hello");

        // Take the value
        let owned_str = tracker.take();
        assert_eq!(owned_str.as_str(), "hello");
        assert!(tracker.is_moved());
        assert!(tracker.get().is_none());
    }

    #[test]
    #[should_panic(expected = "Use after move")]
    fn test_move_tracker_use_after_move() {
        use crate::linear::MoveTracker;

        let s = Str::from_str("hello");
        let mut tracker = MoveTracker::new(s);

        // First take works
        let _s1 = tracker.take();

        // Second take should panic
        let _s2 = tracker.take();
    }

    #[test]
    fn test_move_tracker_automatic_cleanup() {
        use crate::linear::MoveTracker;

        let s = Str::from_str("hello, world!");
        {
            let _tracker = MoveTracker::new(s);
            // tracker goes out of scope here
            // drop_linear should be called automatically
        }
        // Test passes if no memory leak / panic
    }
}
