//! Error types for Atom operations
//!
//! This module defines the error types used throughout the auto-atom crate.
//! All fallible operations return [`AtomResult<T>`], which is a type alias for
//! `Result<T, AtomError>`.

use thiserror::Error;

/// Error type for Atom operations
///
/// This enum represents all possible errors that can occur when working with
/// Atoms, including type mismatches, access errors, serialization failures, and
/// validation errors.
///
/// # Examples
///
/// ```rust
/// use auto_atom::AtomError;
///
/// let err = AtomError::InvalidType {
///     expected: "Node".to_string(),
///     found: "Int".to_string(),
/// };
///
/// assert_eq!(err.to_string(), "invalid type: expected Node, found Int");
/// ```
#[derive(Error, Debug, PartialEq, Clone)]
pub enum AtomError {
    /// Invalid type conversion or access
    ///
    /// This error occurs when trying to convert or access a Value as an
    /// incompatible type.
    #[error("invalid type: expected {expected}, found {found}")]
    InvalidType {
        /// The expected type name
        expected: String,
        /// The actual type found
        found: String,
    },

    /// General conversion failure
    ///
    /// This error occurs when a conversion between formats fails for reasons
    /// other than type mismatch.
    #[error("conversion failed: {0}")]
    ConversionFailed(String),

    /// Access error for a specific path
    ///
    /// This error occurs when trying to access a value that doesn't exist or
    /// cannot be accessed at the given path.
    #[error("access error: {path} - {reason}")]
    AccessError {
        /// The path that was being accessed
        path: String,
        /// Why the access failed
        reason: String,
    },

    /// Serialization or deserialization error
    ///
    /// This error occurs when converting to/from external formats like JSON.
    #[error("serialization error: {format} - {message}")]
    SerializationError {
        /// The format being serialized/deserialized (e.g., "JSON", "XML")
        format: String,
        /// Error message describing the failure
        message: String,
    },

    /// Validation error
    ///
    /// This error occurs when an Atom fails schema validation or other constraints.
    #[error("validation error: {0}")]
    ValidationError(String),

    /// Missing required field
    ///
    /// This error occurs when accessing a required field that doesn't exist.
    #[error("missing required field: {0}")]
    MissingField(String),
}

/// Result type for Atom operations
///
/// This is a type alias for `Result<T, AtomError>` used throughout the
/// auto-atom crate for fallible operations.
///
/// # Examples
///
/// ```rust
/// use auto_atom::{Atom, AtomResult};
/// use auto_val::Value;
///
/// fn create_atom() -> AtomResult<Atom> {
///     let atom = Atom::assemble(vec![
///         Value::pair("name", "test"),
///     ])?;
///     Ok(atom)
/// }
/// ```
///
/// # See Also
///
/// - [`AtomError`] - The error type
/// - [`Atom`] - The main Atom structure
pub type AtomResult<T> = Result<T, AtomError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_type_error() {
        let err = AtomError::InvalidType {
            expected: "Node".to_string(),
            found: "Int".to_string(),
        };
        assert_eq!(err.to_string(), "invalid type: expected Node, found Int");
    }

    #[test]
    fn test_conversion_failed_error() {
        let err = AtomError::ConversionFailed("test failed".to_string());
        assert_eq!(err.to_string(), "conversion failed: test failed");
    }

    #[test]
    fn test_access_error() {
        let err = AtomError::AccessError {
            path: "users.0.name".to_string(),
            reason: "index out of bounds".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "access error: users.0.name - index out of bounds"
        );
    }

    #[test]
    fn test_serialization_error() {
        let err = AtomError::SerializationError {
            format: "JSON".to_string(),
            message: "unexpected token".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "serialization error: JSON - unexpected token"
        );
    }

    #[test]
    fn test_validation_error() {
        let err = AtomError::ValidationError("invalid schema".to_string());
        assert_eq!(err.to_string(), "validation error: invalid schema");
    }

    #[test]
    fn test_missing_field_error() {
        let err = AtomError::MissingField("name".to_string());
        assert_eq!(err.to_string(), "missing required field: name");
    }

    #[test]
    fn test_error_equality() {
        let err1 = AtomError::InvalidType {
            expected: "Node".to_string(),
            found: "Int".to_string(),
        };
        let err2 = AtomError::InvalidType {
            expected: "Node".to_string(),
            found: "Int".to_string(),
        };
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_atomresult_ok() {
        let result: AtomResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_atomresult_err() {
        let result: AtomResult<i32> = Err(AtomError::ConversionFailed("test".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "conversion failed: test");
    }
}
