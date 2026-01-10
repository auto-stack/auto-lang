//! Error types and diagnostics for AutoLang
//!
//! This module provides comprehensive error reporting with source locations,
//! error codes, and helpful suggestions using the `miette` diagnostic library.

use crate::token::Pos;
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

// Re-export commonly used types
pub use miette::{MietteError, Result};

/// Calculate Levenshtein distance between two strings
///
/// Returns the minimum number of single-character edits (insertions, deletions, or substitutions)
/// required to change one string into the other.
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();
    let m = chars1.len();
    let n = chars2.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut matrix = vec![vec![0; n + 1]; m + 1];

    for i in 0..=m {
        matrix[i][0] = i;
    }
    for j in 0..=n {
        matrix[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
            matrix[i][j] = [
                matrix[i - 1][j] + 1,        // deletion
                matrix[i][j - 1] + 1,        // insertion
                matrix[i - 1][j - 1] + cost, // substitution
            ]
            .iter()
            .min()
            .copied()
            .unwrap();
        }
    }

    matrix[m][n]
}

/// Find the best match from a list of candidates using Levenshtein distance
///
/// Returns the candidate with the smallest distance if it's within a reasonable threshold,
/// otherwise returns None.
fn find_best_match(target: &str, candidates: &[String]) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }

    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for candidate in candidates {
        let distance = levenshtein_distance(target, candidate);

        // Threshold: allow up to 3 edits, or 30% of the target length, whichever is larger
        let threshold = std::cmp::max(3, target.len() / 3);

        if distance < best_distance && distance <= threshold {
            best_distance = distance;
            best_match = Some(candidate.clone());
        }
    }

    best_match
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
#[derive(Debug, Clone)]
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

/// Lexer error with attached source code for displaying code snippets
#[derive(Debug, Clone)]
pub struct LexerErrorWithSource {
    pub source: NamedSource<String>,
    pub error: LexerError,
}

impl std::fmt::Display for LexerErrorWithSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for LexerErrorWithSource {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl Diagnostic for LexerErrorWithSource {
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
#[derive(Error, Debug, Clone)]
pub enum AutoError {
    /// Lexer errors during tokenization
    #[error(transparent)]
    Lexer(#[from] LexerError),

    /// Lexer errors with source code
    #[error(transparent)]
    LexerWithSource(#[from] LexerErrorWithSource),

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

    /// Multiple errors collected during parsing with error recovery
    #[error("aborting due to {count} previous error{plural}")]
    MultipleErrors {
        count: usize,
        plural: String,
        errors: Vec<AutoError>,
    },

    /// Compiler warnings
    #[error(transparent)]
    Warning(#[from] Warning),

    /// IO errors (file reading, etc.)
    #[error("{0}")]
    Io(String),

    /// Generic error message (for converting from other error types)
    #[error("{0}")]
    Msg(String),
}

// Manual implementation of Diagnostic for AutoError to properly delegate
impl Diagnostic for AutoError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            AutoError::Lexer(e) => e.code(),
            AutoError::LexerWithSource(e) => e.code(),
            AutoError::Syntax(e) => e.code(),
            AutoError::SyntaxWithSource(e) => e.code(),
            AutoError::Type(e) => e.code(),
            AutoError::Name(e) => e.code(),
            AutoError::Runtime(e) => e.code(),
            AutoError::MultipleErrors { .. } => Some(Box::new("auto_syntax_E0099")),
            AutoError::Warning(e) => e.code(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn severity(&self) -> Option<miette::Severity> {
        match self {
            AutoError::Lexer(e) => e.severity(),
            AutoError::LexerWithSource(e) => e.severity(),
            AutoError::Syntax(e) => e.severity(),
            AutoError::SyntaxWithSource(e) => e.severity(),
            AutoError::Type(e) => e.severity(),
            AutoError::Name(e) => e.severity(),
            AutoError::Runtime(e) => e.severity(),
            AutoError::MultipleErrors { .. } => Some(miette::Severity::Error),
            AutoError::Warning(e) => e.severity(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            AutoError::Lexer(e) => e.help(),
            AutoError::LexerWithSource(e) => e.help(),
            AutoError::Syntax(e) => e.help(),
            AutoError::SyntaxWithSource(e) => e.help(),
            AutoError::Type(e) => e.help(),
            AutoError::Name(e) => e.help(),
            AutoError::Runtime(e) => e.help(),
            AutoError::MultipleErrors { .. } => {
                Some(Box::new("Fix the reported errors and try again"))
            }
            AutoError::Warning(e) => e.help(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            AutoError::Lexer(e) => e.url(),
            AutoError::LexerWithSource(e) => e.url(),
            AutoError::Syntax(e) => e.url(),
            AutoError::SyntaxWithSource(e) => e.url(),
            AutoError::Type(e) => e.url(),
            AutoError::Name(e) => e.url(),
            AutoError::Runtime(e) => e.url(),
            AutoError::MultipleErrors { .. } => None,
            AutoError::Warning(e) => e.url(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        match self {
            AutoError::Lexer(e) => e.labels(),
            AutoError::LexerWithSource(e) => e.labels(),
            AutoError::Syntax(e) => e.labels(),
            AutoError::SyntaxWithSource(e) => e.labels(),
            AutoError::Type(e) => e.labels(),
            AutoError::Name(e) => e.labels(),
            AutoError::Runtime(e) => e.labels(),
            AutoError::MultipleErrors { .. } => None,
            AutoError::Warning(e) => e.labels(),
            AutoError::Io(_) => None,
            AutoError::Msg(_) => None,
        }
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        match self {
            AutoError::MultipleErrors { errors, .. } => Some(Box::new(
                errors.iter().map(|e| e as &dyn miette::Diagnostic),
            )),
            _ => None,
        }
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        match self {
            AutoError::Lexer(e) => e.source_code(),
            AutoError::LexerWithSource(e) => e.source_code(),
            AutoError::Syntax(e) => e.source_code(),
            AutoError::SyntaxWithSource(e) => e.source_code(),
            AutoError::Type(e) => e.source_code(),
            AutoError::Name(e) => e.source_code(),
            AutoError::Runtime(e) => e.source_code(),
            AutoError::MultipleErrors { .. } => None,
            AutoError::Warning(e) => e.source_code(),
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

impl From<std::io::Error> for AutoError {
    fn from(err: std::io::Error) -> Self {
        AutoError::Io(err.to_string())
    }
}

impl<'a> From<&'a str> for AutoError {
    fn from(msg: &'a str) -> Self {
        AutoError::Msg(msg.to_string())
    }
}

impl AutoError {
    /// Attach source code to a syntax error
    pub fn with_source(err: SyntaxError, name: String, code: String) -> Self {
        AutoError::SyntaxWithSource(SyntaxErrorWithSource {
            source: NamedSource::new(name, code),
            error: err,
        })
    }

    /// Attach source code to a lexer error
    pub fn with_source_lexer(err: LexerError, name: String, code: String) -> Self {
        AutoError::LexerWithSource(LexerErrorWithSource {
            source: NamedSource::new(name, code),
            error: err,
        })
    }
}

// ============================================================================
// Lexer Errors (E0001-E0099)
// ============================================================================

/// Lexer errors during tokenization
#[derive(Error, Diagnostic, Debug, Clone)]
pub enum LexerError {
    /// Unknown escape sequence in character literal
    #[error("unknown escape sequence")]
    #[diagnostic(
        code(auto_lexer_E0001),
        help("Valid escape sequences are: \\n, \\t, \\r, \\0")
    )]
    UnknownEscapeSequence {
        sequence: String,
        #[label("unknown escape sequence")]
        span: SourceSpan,
    },

    /// Unterminated character literal
    #[error("unterminated character literal")]
    #[diagnostic(
        code(auto_lexer_E0002),
        help("Character literals must be enclosed in single quotes (')")
    )]
    UnterminatedChar {
        #[label("character literal not closed")]
        span: SourceSpan,
    },

    /// Empty character literal
    #[error("empty character literal")]
    #[diagnostic(
        code(auto_lexer_E0003),
        help("Character literals must contain exactly one character")
    )]
    EmptyChar {
        #[label("empty character literal")]
        span: SourceSpan,
    },

    /// Invalid identifier start
    #[error("invalid identifier")]
    #[diagnostic(
        code(auto_lexer_E0004),
        help("Identifiers must start with a letter or underscore")
    )]
    InvalidIdentifierStart {
        character: String,
        #[label("identifiers must start with a letter or underscore")]
        span: SourceSpan,
    },

    /// Unknown character
    #[error("unknown character")]
    #[diagnostic(
        code(auto_lexer_E0005),
        help("This character is not valid in AutoLang source code")
    )]
    UnknownCharacter {
        character: String,
        #[label("unknown character")]
        span: SourceSpan,
    },
}

// ============================================================================
// Syntax Errors (E0101-E0199)
// ============================================================================

/// Syntax errors during parsing
#[derive(Error, Diagnostic, Debug, Clone)]
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
    #[error("{message}")]
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
#[derive(Error, Diagnostic, Debug, Clone)]
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
#[derive(Error, Debug, Clone)]
pub enum NameError {
    /// Undefined variable
    #[error("undefined variable")]
    UndefinedVariable {
        name: String,
        span: SourceSpan,
        /// Suggested variable name (if a similar one exists)
        suggested: Option<String>,
    },

    /// Duplicate definition
    #[error("duplicate definition")]
    DuplicateDefinition {
        name: String,
        span: SourceSpan,
        original_span: Option<SourceSpan>,
    },

    /// Cannot assign to immutable variable
    #[error("cannot assign to immutable variable")]
    ImmutableAssignment { name: String, span: SourceSpan },

    /// Undefined function
    #[error("undefined function")]
    UndefinedFunction {
        name: String,
        span: SourceSpan,
        /// Suggested function name (if a similar one exists)
        suggested: Option<String>,
    },
}

// Manual Diagnostic implementation for NameError to support dynamic suggestions
impl Diagnostic for NameError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            NameError::UndefinedVariable { .. } => Some(Box::new("auto_name_E0201")),
            NameError::DuplicateDefinition { .. } => Some(Box::new("auto_name_E0202")),
            NameError::ImmutableAssignment { .. } => Some(Box::new("auto_name_E0203")),
            NameError::UndefinedFunction { .. } => Some(Box::new("auto_name_E0204")),
        }
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            NameError::UndefinedVariable { name, .. } => Some(Box::new(format!(
                "Variable '{}' is not defined in this scope",
                name
            ))),
            NameError::DuplicateDefinition { name, .. } => Some(Box::new(format!(
                "The name '{}' is already defined in this scope",
                name
            ))),
            NameError::ImmutableAssignment { name, .. } => Some(Box::new(format!(
                "Use 'mut' instead of 'let' to make '{}' mutable",
                name
            ))),
            NameError::UndefinedFunction { name, .. } => {
                Some(Box::new(format!("Function '{}' is not defined", name)))
            }
        }
    }

    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        match self {
            NameError::UndefinedVariable { name, span, .. } => {
                let offset = span.offset();
                let len = span.len();
                Some(Box::new(std::iter::once(miette::LabeledSpan::new(
                    Some(format!("variable '{}' not found", name)),
                    offset,
                    len,
                ))))
            }
            NameError::DuplicateDefinition { name, span, .. } => {
                let offset = span.offset();
                let len = span.len();
                Some(Box::new(std::iter::once(miette::LabeledSpan::new(
                    Some(format!("'{}' is already defined", name)),
                    offset,
                    len,
                ))))
            }
            NameError::ImmutableAssignment { name, span } => {
                let offset = span.offset();
                let len = span.len();
                Some(Box::new(std::iter::once(miette::LabeledSpan::new(
                    Some(format!("'{}' is immutable", name)),
                    offset,
                    len,
                ))))
            }
            NameError::UndefinedFunction { name, span, .. } => {
                let offset = span.offset();
                let len = span.len();
                Some(Box::new(std::iter::once(miette::LabeledSpan::new(
                    Some(format!("function '{}' not found", name)),
                    offset,
                    len,
                ))))
            }
        }
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        None
    }
}

impl NameError {
    /// Create an UndefinedVariable error with automatic suggestion
    pub fn undefined_variable(name: String, span: SourceSpan, candidates: &[String]) -> Self {
        let suggested = find_best_match(&name, candidates);
        NameError::UndefinedVariable {
            name,
            span,
            suggested,
        }
    }

    /// Create an UndefinedFunction error with automatic suggestion
    pub fn undefined_function(name: String, span: SourceSpan, candidates: &[String]) -> Self {
        let suggested = find_best_match(&name, candidates);
        NameError::UndefinedFunction {
            name,
            span,
            suggested,
        }
    }

    /// Get the suggestion text for display
    pub fn get_suggestion_text(&self) -> Option<String> {
        match self {
            NameError::UndefinedVariable { suggested, .. } => {
                suggested.as_ref().map(|s| format!("Did you mean '{}'?", s))
            }
            NameError::UndefinedFunction { suggested, .. } => {
                suggested.as_ref().map(|s| format!("Did you mean '{}'?", s))
            }
            _ => None,
        }
    }
}

// ============================================================================
// Runtime Errors (E0301-E0399)
// ============================================================================

/// Runtime evaluation errors
#[derive(Error, Diagnostic, Debug, Clone)]
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

// ============================================================================
// Warning Errors (W0001-W0099)
// ============================================================================

/// Compiler warnings
///
/// These don't prevent compilation but indicate potential issues
#[derive(Error, Diagnostic, Debug, Clone)]
pub enum Warning {
    /// Unused variable warning
    #[error("unused variable")]
    #[diagnostic(
        code(auto_warning_W0001),
        severity(warning),
        help("Variable '{name}' is defined but never used")
    )]
    UnusedVariable {
        name: String,
        #[label("unused variable '{name}'")]
        span: SourceSpan,
    },

    /// Unused import warning
    #[error("unused import")]
    #[diagnostic(
        code(auto_warning_W0002),
        severity(warning),
        help("Import '{path}' is not used in this module")
    )]
    UnusedImport {
        path: String,
        #[label("unused import '{path}'")]
        span: SourceSpan,
    },

    /// Dead code warning
    #[error("dead code")]
    #[diagnostic(
        code(auto_warning_W0003),
        severity(warning),
        help("This code will never be executed")
    )]
    DeadCode {
        #[label("unreachable code")]
        span: SourceSpan,
    },

    /// Implicit type conversion warning
    #[error("implicit type conversion")]
    #[diagnostic(
        code(auto_warning_W0004),
        severity(warning),
        help("Implicit conversion from '{from}' to '{to}'")
    )]
    ImplicitTypeConversion {
        from: String,
        to: String,
        #[label("implicit conversion")]
        span: SourceSpan,
    },

    /// Deprecated feature warning
    #[error("deprecated feature")]
    #[diagnostic(
        code(auto_warning_W0005),
        severity(warning),
        help("'{name}' is deprecated: {message}")
    )]
    DeprecatedFeature {
        name: String,
        message: String,
        #[label("deprecated feature '{name}'")]
        span: SourceSpan,
    },
}

/// Attach source code to any error for displaying code snippets
pub fn attach_source(err: AutoError, name: String, code: String) -> AutoError {
    match err {
        AutoError::Lexer(e) => AutoError::LexerWithSource(LexerErrorWithSource {
            source: NamedSource::new(name, code),
            error: e,
        }),
        AutoError::Syntax(e) => AutoError::SyntaxWithSource(SyntaxErrorWithSource {
            source: NamedSource::new(name, code),
            error: e,
        }),
        AutoError::MultipleErrors { count, plural, mut errors } => {
            // Attach source to each error in the list
            for error in errors.iter_mut() {
                *error = attach_source(error.clone(), name.clone(), code.clone());
            }
            AutoError::MultipleErrors { count, plural, errors }
        }
        _ => {
            // For other error types (Name, Type, Runtime), we can't attach source
            // Return the error as-is
            err
        }
    }
}
