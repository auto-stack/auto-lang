//! Error types and diagnostics for AutoLang
//!
//! This module provides comprehensive error reporting with source locations,
//! error codes, and helpful suggestions using the `miette` diagnostic library.

use crate::token::Pos;
use miette::{Diagnostic, NamedSource, SourceCode, SourceSpan};
use thiserror::Error;

// Re-export commonly used types
pub use miette::{MietteError, Result};

/// Thread-local storage for the current source code being processed
///
/// This allows error reporting to access source code without threading it
/// through every function call.
thread_local! {
    static CURRENT_SOURCE: std::cell::RefCell<Option<NamedSource<String>>> = const { std::cell::RefCell::new(None) };
}

/// Set the current source code for error reporting
///
/// This should be called at the start of parsing/compilation to enable
/// source code snippets in error messages.
pub fn set_source(name: String, code: String) {
    CURRENT_SOURCE.with(|source| {
        *source.borrow_mut() = Some(NamedSource::new(name, code));
    });
}

/// Clear the current source code
pub fn clear_source() {
    CURRENT_SOURCE.with(|source| {
        *source.borrow_mut() = None;
    });
}

/// Get a reference to the current source code, if available
pub fn get_source() -> Option<NamedSource<String>> {
    CURRENT_SOURCE.with(|source| source.borrow().as_ref().cloned())
}

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

/// Syntax error with attached source code for displaying code snippets
#[derive(Debug)]
pub struct SyntaxErrorWithSource {
    pub source: NamedSource<String>,
    pub error: SyntaxError,
}

impl std::fmt::Display for SyntaxErrorWithSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for SyntaxErrorWithSource {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl Diagnostic for SyntaxErrorWithSource {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.error.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.error.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.error.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.error.url()
    }

    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        self.error.labels()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source)
    }
}

/// Alias for Result type with AutoLang errors
pub type AutoResult<T> = std::result::Result<T, AutoError>;

/// Comprehensive error type for AutoLang compiler
///
/// This enum encompasses all possible errors that can occur during
/// compilation, parsing, type checking, and evaluation.
#[derive(Error, Debug)]
pub enum AutoError {
    /// Syntax errors during parsing
    #[error(transparent)]
    Syntax(#[from] SyntaxError),

    /// Syntax errors with source code
    #[error(transparent)]
    SyntaxWithSource(#[from] SyntaxErrorWithSource),

    /// Type errors
    #[error(transparent)]
    Type(#[from] TypeError),

    /// Name/binding errors (undefined variables, duplicate definitions)
    #[error(transparent)]
    Name(#[from] NameError),

    /// Runtime errors (division by zero, index out of bounds, etc.)
    #[error(transparent)]
    Runtime(#[from] RuntimeError),

    /// IO errors (file reading, etc.)
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Generic error message (for converting from other error types)
    #[error("{0}")]
    Msg(String),
}

// Manual implementation of Diagnostic for AutoError to properly delegate
impl Diagnostic for AutoError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            AutoError::Syntax(e) => e.code(),
            AutoError::SyntaxWithSource(e) => e.code(),
            AutoError::Type(e) => e.code(),
            AutoError::Name(e) => e.code(),
            AutoError::Runtime(e) => e.code(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn severity(&self) -> Option<miette::Severity> {
        match self {
            AutoError::Syntax(e) => e.severity(),
            AutoError::SyntaxWithSource(e) => e.severity(),
            AutoError::Type(e) => e.severity(),
            AutoError::Name(e) => e.severity(),
            AutoError::Runtime(e) => e.severity(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            AutoError::Syntax(e) => e.help(),
            AutoError::SyntaxWithSource(e) => e.help(),
            AutoError::Type(e) => e.help(),
            AutoError::Name(e) => e.help(),
            AutoError::Runtime(e) => e.help(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            AutoError::Syntax(e) => e.url(),
            AutoError::SyntaxWithSource(e) => e.url(),
            AutoError::Type(e) => e.url(),
            AutoError::Name(e) => e.url(),
            AutoError::Runtime(e) => e.url(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        match self {
            AutoError::Syntax(e) => e.labels(),
            AutoError::SyntaxWithSource(e) => e.labels(),
            AutoError::Type(e) => e.labels(),
            AutoError::Name(e) => e.labels(),
            AutoError::Runtime(e) => e.labels(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        match self {
            AutoError::Syntax(e) => e.source_code(),
            AutoError::SyntaxWithSource(e) => e.source_code(),
            AutoError::Type(e) => e.source_code(),
            AutoError::Name(e) => e.source_code(),
            AutoError::Runtime(e) => e.source_code(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }
}

impl From<String> for AutoError {
    fn from(msg: String) -> Self {
        AutoError::Msg(msg)
    }
}

impl<'a> From<&'a str> for AutoError {
    fn from(msg: &'a str) -> Self {
        AutoError::Msg(msg.to_string())
    }
}

impl AutoError {
    /// Get the source code associated with this error, if available
    pub fn source_code(&self) -> Option<NamedSource<String>> {
        get_source()
    }

    /// Attach source code to a syntax error
    pub fn with_source(err: SyntaxError, name: String, code: String) -> Self {
        AutoError::SyntaxWithSource(SyntaxErrorWithSource {
            source: NamedSource::new(name, code),
            error: err,
        })
    }
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
