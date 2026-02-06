// Plan 078: AutoMan Error Types
//
// Error types for AutoMan operations

use std::path::PathBuf;

/// AutoMan error type
#[derive(Debug, thiserror::Error)]
pub enum AutoManError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Dependency resolution error
    #[error("Dependency resolution error: {0}")]
    DependencyError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Result type for AutoMan operations
pub type AutoManResult<T> = Result<T, AutoManError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AutoManError::FileNotFound(PathBuf::from("test.at"));
        assert!(err.to_string().contains("test.at"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let auto_err: AutoManError = io_err.into();
        assert!(matches!(auto_err, AutoManError::Io(_)));
    }
}
