//! FFI Error types for the Hybrid FFI Bridge

use std::fmt;

/// Errors that can occur during FFI operations
#[derive(Debug)]
pub enum FFIError {
    /// Type mismatch between expected and actual VM value
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
    },

    /// Invalid string index in VM string pool
    InvalidStringIndex(u16),

    /// Invalid list ID in VM heap
    InvalidListId(u64),

    /// Invalid HashMap ID in VM heap
    InvalidHashMapId(u64),

    /// Invalid heap object ID
    InvalidHeapObjectId(u64),

    /// Stack underflow when popping arguments
    StackUnderflow {
        expected: usize,
        found: usize,
    },

    /// Runtime error from the FFI function
    RuntimeError(String),

    /// IO error during file operations
    IoError(std::io::Error),

    /// UTF-8 conversion error
    Utf8Error(std::str::Utf8Error),
}

impl fmt::Display for FFIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FFIError::TypeMismatch { expected, found } => {
                write!(f, "FFI type mismatch: expected {}, found {}", expected, found)
            }
            FFIError::InvalidStringIndex(idx) => {
                write!(f, "Invalid string index: {}", idx)
            }
            FFIError::InvalidListId(id) => {
                write!(f, "Invalid list ID: {}", id)
            }
            FFIError::InvalidHashMapId(id) => {
                write!(f, "Invalid HashMap ID: {}", id)
            }
            FFIError::InvalidHeapObjectId(id) => {
                write!(f, "Invalid heap object ID: {}", id)
            }
            FFIError::StackUnderflow { expected, found } => {
                write!(
                    f,
                    "Stack underflow: expected {} arguments, found {}",
                    expected, found
                )
            }
            FFIError::RuntimeError(msg) => {
                write!(f, "FFI runtime error: {}", msg)
            }
            FFIError::IoError(e) => {
                write!(f, "FFI I/O error: {}", e)
            }
            FFIError::Utf8Error(e) => {
                write!(f, "FFI UTF-8 error: {}", e)
            }
        }
    }
}

impl std::error::Error for FFIError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FFIError::IoError(e) => Some(e),
            FFIError::Utf8Error(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FFIError {
    fn from(e: std::io::Error) -> Self {
        FFIError::IoError(e)
    }
}

impl From<std::str::Utf8Error> for FFIError {
    fn from(e: std::str::Utf8Error) -> Self {
        FFIError::Utf8Error(e)
    }
}

impl From<FFIError> for crate::vm::engine::VMError {
    fn from(e: FFIError) -> Self {
        crate::vm::engine::VMError::RuntimeError(e.to_string())
    }
}
