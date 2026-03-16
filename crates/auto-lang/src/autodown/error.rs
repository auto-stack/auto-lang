//! AutoDown Error Types
//!
//! Provides error handling for AutoDown parsing and transpilation.

use thiserror::Error;

/// AutoDown result type
pub type AdocResult<T> = std::result::Result<T, AdocError>;

/// AutoDown error types
#[derive(Debug, Error)]
pub enum AdocError {
    /// Lexer errors
    #[error("lexer error: {message}")]
    Lexer { message: String },

    /// Parser errors
    #[error("parser error: {message}")]
    Parser { message: String },

    /// Unexpected token
    #[error("unexpected token: expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },

    /// Unterminated string/block
    #[error("unterminated {kind}: started at line {line}")]
    Unterminated { kind: String, line: usize },

    /// Invalid expression
    #[error("invalid expression: {message}")]
    InvalidExpression { message: String },

    /// Math parsing errors
    #[error("math error: {message}")]
    Math { message: String },

    /// Transpilation errors
    #[error("transpilation error: {message}")]
    Transpile { message: String },

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error with message
    #[error("{message}")]
    Generic { message: String },
}

impl AdocError {
    /// Create a lexer error
    pub fn lexer(message: impl Into<String>) -> Self {
        AdocError::Lexer {
            message: message.into(),
        }
    }

    /// Create a parser error
    pub fn parser(message: impl Into<String>) -> Self {
        AdocError::Parser {
            message: message.into(),
        }
    }

    /// Create an unexpected token error
    pub fn unexpected_token(expected: impl Into<String>, found: impl Into<String>) -> Self {
        AdocError::UnexpectedToken {
            expected: expected.into(),
            found: found.into(),
        }
    }

    /// Create an unterminated error
    pub fn unterminated(kind: impl Into<String>, line: usize) -> Self {
        AdocError::Unterminated {
            kind: kind.into(),
            line,
        }
    }
    /// Create a generic error
    pub fn generic(message: impl Into<String>) -> Self {
        AdocError::Generic {
            message: message.into(),
        }
    }

    /// Create an invalid expression error
    pub fn invalid_expression(message: impl Into<String>) -> Self {
        AdocError::InvalidExpression {
            message: message.into(),
        }
    }
}
