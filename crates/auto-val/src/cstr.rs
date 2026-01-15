//! C string (null-terminated) type for FFI
//!
//! This module provides a C-compatible string type for safe FFI boundaries.
//! CStr wraps null-terminated UTF-8 bytes for passing to C functions.

use crate::AutoStr;
use std::fmt;
use std::os::raw::c_char;
use std::slice;

/// C-compatible null-terminated string
///
/// This type wraps null-terminated UTF-8 bytes for safe FFI.
/// It ensures that the data is valid UTF-8 and null-terminated.
#[derive(Clone, PartialEq)]
pub struct CStr {
    /// Null-terminated UTF-8 bytes
    data: Vec<u8>,
}

impl CStr {
    /// Create a CStr from a string slice (adds null terminator)
    ///
    /// # Arguments
    /// * `s` - String slice to convert
    ///
    /// # Returns
    /// New CStr with null terminator
    pub fn from_str(s: &str) -> Self {
        let mut data = s.as_bytes().to_vec();
        data.push(0); // Null terminator
        CStr { data }
    }

    /// Create a CStr from an AutoStr
    pub fn from_astr(astr: &AutoStr) -> Self {
        Self::from_str(astr.as_str())
    }

    /// Create a CStr from raw bytes (must include null terminator)
    ///
    /// # Safety
    ///
    /// The bytes must be null-terminated and valid UTF-8
    pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Self {
        CStr { data: bytes }
    }

    /// Create a CStr from a C string pointer
    ///
    /// # Safety
    ///
    /// The pointer must be valid, null-terminated, and point to valid UTF-8
    pub unsafe fn from_ptr(ptr: *const c_char) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        // Calculate length including null terminator
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        // Copy bytes including null terminator
        let data = slice::from_raw_parts(ptr as *const u8, len + 1).to_vec();

        // Validate UTF-8
        if std::str::from_utf8(&data[..len]).is_err() {
            return None;
        }

        Some(CStr { data })
    }

    /// Get pointer to C string data (for FFI)
    ///
    /// # Returns
    /// Pointer to null-terminated bytes (valid as long as CStr exists)
    pub fn as_ptr(&self) -> *const c_char {
        self.data.as_ptr() as *const c_char
    }

    /// Get byte length (excluding null terminator)
    pub fn len(&self) -> usize {
        self.data.len() - 1 // Exclude null terminator
    }

    /// Check if C string is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get as Rust CStr (for std::ffi compatibility)
    pub fn as_cstr(&self) -> &std::ffi::CStr {
        unsafe {
            // Safety: data is always null-terminated and valid UTF-8
            std::ffi::CStr::from_bytes_with_nul_unchecked(&self.data)
        }
    }

    /// Get as Rust string slice
    pub fn as_str(&self) -> &str {
        unsafe {
            // Safety: data is always valid UTF-8
            std::str::from_utf8_unchecked(&self.data[..self.len()])
        }
    }

    /// Convert to AutoStr
    pub fn to_astr(&self) -> AutoStr {
        self.as_str().into()
    }

    /// Create a CStr from an OwnedStr
    pub fn from_owned_str(s: &crate::owned_str::Str) -> Self {
        let bytes = s.data();
        let mut data = bytes.to_vec();
        data.push(0); // Add null terminator
        CStr { data }
    }
}

impl fmt::Debug for CStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CStr({})", self.as_str())
    }
}

impl fmt::Display for CStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr_from_str() {
        let cstr = CStr::from_str("hello");
        assert_eq!(cstr.len(), 5);
        assert_eq!(cstr.as_str(), "hello");
        assert!(!cstr.is_empty());
    }

    #[test]
    fn test_cstr_empty() {
        let cstr = CStr::from_str("");
        assert_eq!(cstr.len(), 0);
        assert!(cstr.is_empty());
        // Should still have null terminator
        assert_eq!(cstr.data.len(), 1);
    }

    #[test]
    fn test_cstr_null_terminated() {
        let cstr = CStr::from_str("test");
        assert_eq!(cstr.data[4], 0); // Last byte is null
        assert_eq!(cstr.data.len(), 5); // 4 bytes + null
    }

    #[test]
    fn test_cstr_to_astr() {
        let cstr = CStr::from_str("hello");
        let astr = cstr.to_astr();
        assert_eq!(astr.as_str(), "hello");
    }

    #[test]
    fn test_cstr_as_ptr() {
        let cstr = CStr::from_str("test");
        let ptr = cstr.as_ptr();
        assert!(!ptr.is_null());

        unsafe {
            // Verify we can read through the pointer
            assert_eq!(*ptr.add(0) as u8, b't');
            assert_eq!(*ptr.add(1) as u8, b'e');
            assert_eq!(*ptr.add(2) as u8, b's');
            assert_eq!(*ptr.add(3) as u8, b't');
            assert_eq!(*ptr.add(4) as u8, 0); // Null terminator
        }
    }

    #[test]
    fn test_cstr_utf8() {
        let cstr = CStr::from_str("hello 世界");
        assert_eq!(cstr.len(), 12); // "hello " (6) + "世界" (6) in UTF-8
        assert_eq!(cstr.as_str(), "hello 世界");
    }
}
