/// String slice type - borrowed view into a string (Phase 3)
///
/// A `StrSlice` is a borrowed view into an existing string, similar to Rust's `&str`.
/// It contains a pointer to the data and a length, but does not own the data.
///
/// # Safety
///
/// `StrSlice` is unsafe by design - it must be ensured that:
/// 1. The underlying data outlives the slice
/// 2. The data pointer is valid for the given length
/// 3. The data is valid UTF-8
///
/// The borrow checker (Phase 3) should enforce these guarantees at compile time.
///
/// # Example
///
/// ```ignore
/// let s = str_new("hello", 5)
/// let slice = str_slice(s)  // Creates a borrow
/// let len = str_slice_len(slice)  // 5
/// // s is still valid here (immutable borrow)
/// ```

use crate::AutoStr;
use std::fmt;

/// Borrowed string slice
///
/// # Layout
///
/// ```text
/// StrSlice
/// ├── data: *const u8    // Pointer to borrowed bytes
/// └── len: usize          // Length in bytes (not characters!)
/// ```
///
/// # Important Notes
///
/// - **No lifetime field**: The compiler tracks lifetimes, not runtime
/// - **Byte length**: `len` is bytes, not characters (UTF-8)
/// - **Immutable**: Cannot modify the underlying data
#[derive(Debug, Clone)]
pub struct StrSlice {
    /// Pointer to borrowed UTF-8 data
    pub data: *const u8,
    /// Length in bytes
    pub len: usize,
}

unsafe impl Send for StrSlice {}
unsafe impl Sync for StrSlice {}

impl StrSlice {
    /// Create a new StrSlice from a pointer and length
    ///
    /// # Safety
    ///
    /// Caller must ensure:
    /// - `data` is valid for `len` bytes
    /// - `data` points to valid UTF-8 data
    /// - The underlying data outlives the StrSlice
    pub unsafe fn from_raw_parts(data: *const u8, len: usize) -> Self {
        Self { data, len }
    }

    /// Create a StrSlice from a string slice
    ///
    /// # Safety
    ///
    /// The underlying string must outlive this StrSlice
    pub unsafe fn from_str(s: &str) -> Self {
        Self {
            data: s.as_ptr(),
            len: s.len(),
        }
    }

    /// Create a StrSlice from an AutoStr
    ///
    /// # Safety
    ///
    /// The AutoStr must outlive this StrSlice
    pub unsafe fn from_auto_str(s: &AutoStr) -> Self {
        Self {
            data: s.as_ptr(),
            len: s.len(),
        }
    }

    /// Create an empty StrSlice
    pub fn empty() -> Self {
        Self {
            data: std::ptr::null(),
            len: 0,
        }
    }

    /// Get the length of the slice in bytes
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the slice is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get a reference to the underlying bytes
    ///
    /// # Safety
    ///
    /// Caller must ensure the StrSlice is still valid
    pub unsafe fn as_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts(self.data, self.len)
    }

    /// Convert to a string slice
    ///
    /// # Safety
    ///
    /// Caller must ensure:
    /// - The data is valid UTF-8
    /// - The StrSlice is still valid
    pub unsafe fn as_str(&self) -> &str {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.data, self.len))
    }

    /// Get the byte at the given index
    ///
    /// Returns None if index is out of bounds
    pub fn get_byte(&self, index: usize) -> Option<u8> {
        if index >= self.len {
            return None;
        }
        unsafe { Some(*self.data.add(index)) }
    }
}

impl fmt::Display for StrSlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let s = self.as_str();
            write!(f, "{}", s)
        }
    }
}

impl PartialEq for StrSlice {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        unsafe {
            let self_bytes = self.as_bytes();
            let other_bytes = other.as_bytes();
            self_bytes == other_bytes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_slice_from_str() {
        let s = "hello";
        let slice = unsafe { StrSlice::from_str(s) };
        assert_eq!(slice.len(), 5);
        assert!(!slice.is_empty());
    }

    #[test]
    fn test_str_slice_empty() {
        let slice = StrSlice::empty();
        assert_eq!(slice.len(), 0);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_str_slice_get_byte() {
        let s = "hello";
        let slice = unsafe { StrSlice::from_str(s) };
        assert_eq!(slice.get_byte(0), Some(b'h'));
        assert_eq!(slice.get_byte(4), Some(b'o'));
        assert_eq!(slice.get_byte(5), None);
    }

    #[test]
    fn test_str_slice_display() {
        let s = "hello";
        let slice = unsafe { StrSlice::from_str(s) };
        assert_eq!(format!("{}", slice), "hello");
    }

    #[test]
    fn test_str_slice_equality() {
        let s1 = "hello";
        let s2 = "hello";
        let s3 = "world";

        let slice1 = unsafe { StrSlice::from_str(s1) };
        let slice2 = unsafe { StrSlice::from_str(s2) };
        let slice3 = unsafe { StrSlice::from_str(s3) };

        assert_eq!(slice1, slice2);
        assert_ne!(slice1, slice3);
    }
}
