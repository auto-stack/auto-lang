//! Error types and diagnostics for AutoLang
//!
//! This module provides comprehensive error reporting with source locations,
//! error codes, and helpful suggestions using the `miette` diagnostic library.

use crate::token::Pos;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

// Re-export commonly used types
pub use miette::{MietteError, Result};

/// Convert a `Pos` to a `SourceSpan` for use with miette diagnostics
///
/// # Example
///
/// ```rust
/// use auto_lang::error::pos_to_span;
/// use auto_lang::token::Pos;
///
/// let pos = Pos {
///     line: 5,
///     at: 10,
///     pos: 100,
///     len: 5,
/// };
/// let span = pos_to_span(pos);
/// assert_eq!(span.offset(), 100);
/// assert_eq!(span.len(), 5);
/// ```
pub fn pos_to_span(pos: Pos) -> SourceSpan {
    SourceSpan::new(pos.pos.into(), pos.len.into())
}

/// Create a span from absolute position and length
pub fn span_from(offset: usize, len: usize) -> SourceSpan {
    SourceSpan::new(offset.into(), len.into())
}

/// Alias for Result type with AutoLang errors
pub type AutoResult<T> = std::result::Result<T, AutoError>;

/// Comprehensive error type for AutoLang compiler
///
/// This enum encompasses all possible errors that can occur during
/// compilation, parsing, type checking, and evaluation.
#[derive(Error, Diagnostic, Debug)]
pub enum AutoError {
    /// Syntax errors during parsing
    #[error(transparent)]
    #[diagnostic(code(auto_syntax_E0001))]
    Syntax(#[from] SyntaxError),

    /// Type errors
    #[error(transparent)]
    #[diagnostic(code(auto_type_E0101))]
    Type(#[from] TypeError),

    /// Name/binding errors (undefined variables, duplicate definitions)
    #[error(transparent)]
    #[diagnostic(code(auto_name_E0201))]
    Name(#[from] NameError),

    /// Runtime errors (division by zero, index out of bounds, etc.)
    #[error(transparent)]
    #[diagnostic(code(auto_runtime_E0301))]
    Runtime(#[from] RuntimeError),

    /// IO errors (file reading, etc.)
    #[error(transparent)]
    #[diagnostic(code(auto_io_E0401))]
    Io(#[from] std::io::Error),
}

// ============================================================================
// Syntax Errors (E0001-E0099)
// ============================================================================

/// Syntax errors during parsing
#[derive(Error, Diagnostic, Debug)]
pub enum SyntaxError {
    /// Unexpected token encountered
    #[error("unexpected token")]
    #[diagnostic(
        code(auto_syntax_E0001),
        help("Expected {expected}, but found {found}")
    )]
    UnexpectedToken {
        expected: String,
        found: String,
        #[label("unexpected token")]
        span: SourceSpan,
    },

    /// Invalid expression syntax
    #[error("invalid expression syntax")]
    #[diagnostic(
        code(auto_syntax_E0002),
        help("This expression is not valid in the current context")
    )]
    InvalidExpression {
        #[label("here")]
        span: SourceSpan,
    },

    /// Unterminated string literal
    #[error("unterminated string literal")]
    #[diagnostic(
        code(auto_syntax_E0003),
        help("Add a closing quote (\") to end the string")
    )]
    UnterminatedString {
        #[label("string starts here")]
        span: SourceSpan,
    },

    /// Unterminated comment
    #[error("unterminated comment")]
    #[diagnostic(code(auto_syntax_E0004), help("Add '*/' to close the comment"))]
    UnterminatedComment {
        #[label("comment starts here")]
        span: SourceSpan,
    },

    /// Invalid escape sequence
    #[error("invalid escape sequence")]
    #[diagnostic(
        code(auto_syntax_E0005),
        help("Use standard escape sequences like \\n, \\t, \\\\, etc.")
    )]
    InvalidEscapeSequence {
        sequence: String,
        #[label("invalid escape sequence")]
        span: SourceSpan,
    },

    /// Missing closing delimiter
    #[error("missing closing delimiter")]
    #[diagnostic(
        code(auto_syntax_E0006),
        help("Add '{delimiter}' to close this {context}")
    )]
    MissingDelimiter {
        delimiter: String,
        context: String,
        #[label("opened here")]
        span: SourceSpan,
    },

    /// Generic syntax error
    #[error("syntax error")]
    #[diagnostic(code(auto_syntax_E0007))]
    Generic {
        message: String,
        #[label("{}", message)]
        span: SourceSpan,
    },
}

// ============================================================================
// Type Errors (E0101-E0199)
// ============================================================================

/// Type checking errors
#[derive(Error, Diagnostic, Debug)]
pub enum TypeError {
    /// Type mismatch
    #[error("type mismatch")]
    #[diagnostic(
        code(auto_type_E0101),
        help("Expected type '{expected}', but found '{found}'")
    )]
    Mismatch {
        expected: String,
        found: String,
        #[label("this expression has type '{found}'")]
        span: SourceSpan,
    },

    /// Invalid operation for type
    #[error("invalid operation for type")]
    #[diagnostic(
        code(auto_type_E0102),
        help("The operation '{op}' is not supported for values of type '{ty}'")
    )]
    InvalidOperation {
        op: String,
        ty: String,
        #[label("cannot perform '{op}' on type '{ty}'")]
        span: SourceSpan,
    },

    /// Not a callable type
    #[error("not a callable type")]
    #[diagnostic(
        code(auto_type_E0103),
        help("Only functions can be called, but this expression has type '{ty}'")
    )]
    NotCallable {
        ty: String,
        #[label("not a function")]
        span: SourceSpan,
    },

    /// Invalid array index
    #[error("invalid array index")]
    #[diagnostic(
        code(auto_type_E0104),
        help("Array indices must be integers, but this has type '{ty}'")
    )]
    InvalidIndexType {
        ty: String,
        #[label("this expression has type '{ty}', not an integer")]
        span: SourceSpan,
    },

    /// Invalid array size
    #[error("invalid array size")]
    #[diagnostic(
        code(auto_type_E0105),
        help("Array size must be a constant integer expression")
    )]
    InvalidArraySize {
        #[label("not a constant integer")]
        span: SourceSpan,
    },
}

// ============================================================================
// Name Errors (E0201-E0299)
// ============================================================================

/// Name resolution and binding errors
#[derive(Error, Diagnostic, Debug)]
pub enum NameError {
    /// Undefined variable
    #[error("undefined variable")]
    #[diagnostic(
        code(auto_name_E0201),
        help("Variable '{name}' is not defined in this scope")
    )]
    UndefinedVariable {
        name: String,
        #[label("variable '{name}' not found")]
        span: SourceSpan,
    },

    /// Duplicate definition
    #[error("duplicate definition")]
    #[diagnostic(
        code(auto_name_E0202),
        help("The name '{name}' is already defined in this scope")
    )]
    DuplicateDefinition {
        name: String,
        #[label("'{name}' is already defined")]
        span: SourceSpan,
        original_span: Option<SourceSpan>,
    },

    /// Cannot assign to immutable variable
    #[error("cannot assign to immutable variable")]
    #[diagnostic(
        code(auto_name_E0203),
        help("Use 'mut' instead of 'let' to make this variable mutable")
    )]
    ImmutableAssignment {
        name: String,
        #[label("'{name}' is immutable")]
        span: SourceSpan,
    },

    /// Undefined function
    #[error("undefined function")]
    #[diagnostic(code(auto_name_E0204), help("Function '{name}' is not defined"))]
    UndefinedFunction {
        name: String,
        #[label("function '{name}' not found")]
        span: SourceSpan,
    },
}

// ============================================================================
// Runtime Errors (E0301-E0399)
// ============================================================================

/// Runtime evaluation errors
#[derive(Error, Diagnostic, Debug)]
pub enum RuntimeError {
    /// Division by zero
    #[error("division by zero")]
    #[diagnostic(code(auto_runtime_E0301), help("Division by zero is undefined"))]
    DivisionByZero {
        #[label("attempting to divide by zero")]
        span: SourceSpan,
    },

    /// Modulo by zero
    #[error("modulo by zero")]
    #[diagnostic(code(auto_runtime_E0302), help("Modulo by zero is undefined"))]
    ModuloByZero {
        #[label("attempting modulo by zero")]
        span: SourceSpan,
    },

    /// Index out of bounds
    #[error("index out of bounds")]
    #[diagnostic(
        code(auto_runtime_E0303),
        help("Index {index} is out of bounds for array of length {len}")
    )]
    IndexOutOfBounds {
        index: i64,
        len: i64,
        #[label("index {index} is out of bounds")]
        span: SourceSpan,
    },

    /// Invalid assignment target
    #[error("invalid assignment target")]
    #[diagnostic(code(auto_runtime_E0304), help("Cannot assign to this expression"))]
    InvalidAssignmentTarget {
        #[label("not a valid assignment target")]
        span: SourceSpan,
    },

    /// Break outside loop
    #[error("break outside loop")]
    #[diagnostic(
        code(auto_runtime_E0305),
        help("'break' can only be used inside loops")
    )]
    BreakOutsideLoop {
        #[label("'break' statement not inside a loop")]
        span: SourceSpan,
    },
}
