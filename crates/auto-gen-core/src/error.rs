use auto_val::AutoStr;
use std::path::PathBuf;

/// Source location for error reporting
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub source_line: String,
}

impl SourceLocation {
    pub fn new(file: PathBuf, line: usize, column: usize) -> Self {
        Self {
            file,
            line,
            column,
            source_line: String::new(),
        }
    }

    pub fn with_source_line(mut self, line: String) -> Self {
        self.source_line = line;
        self
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
    }
}

/// Code generator errors
#[derive(thiserror::Error, Debug)]
pub enum GenError {
    #[error("Failed to load data from {path}: {reason}")]
    DataLoadError { path: PathBuf, reason: String },

    #[error("Failed to load template from {path}: {reason}")]
    TemplateLoadError { path: PathBuf, reason: String },

    #[error("Failed to load config from {path}: {reason}")]
    ConfigLoadError { path: PathBuf, reason: String },

    #[error("{location}: Template syntax error: {message}")]
    TemplateSyntaxError {
        location: SourceLocation,
        message: String,
    },

    #[error("{location}: {message}")]
    ConfigSyntaxError {
        location: SourceLocation,
        message: String,
    },

    #[error("Guard merge conflict in {file} at guard '{guard_id}'")]
    GuardConflict {
        file: PathBuf,
        guard_id: AutoStr,
        existing_content: AutoStr,
        generated_content: AutoStr,
    },

    #[error("Invalid guard syntax at line {line}")]
    InvalidGuardSyntax { line: usize },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("{0}")]
    Other(String),
}

impl GenError {
    /// Format error with IDE-style display
    pub fn display(&self) -> String {
        match self {
            GenError::TemplateSyntaxError { location, message } => {
                let mut result = format!("error: {}\n", message);
                result.push_str(&format!(
                    "  --> {}:{}\n",
                    location.file.display(),
                    location.line
                ));
                if !location.source_line.is_empty() {
                    result.push_str(&format!("   |\n"));
                    result.push_str(&format!("{} | {}\n", location.line, location.source_line));
                    result.push_str(&format!("   | {}^\n", " ".repeat(location.column.min(100))));
                }
                result
            }
            GenError::GuardConflict { file, guard_id, .. } => {
                format!(
                    "error: Guard conflict in '{}' at guard '{}'\n  --> {}",
                    file.display(),
                    guard_id,
                    file.display()
                )
            }
            _ => format!("{}", self),
        }
    }
}

/// Result type for code generator operations
pub type GenResult<T> = Result<T, GenError>;
