//! A2UI Protocol Bridge
//!
//! Provides bidirectional conversion between AutoUI's AURA intermediate
//! representation and Google's A2UI v0.8 JSON protocol.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auto_lang::a2ui::{export_widget, import_message, A2UIError};
//!
//! // Export AURA widget to A2UI JSON
//! let aura_widget = extract_widget_from_ast(...);
//! let a2ui_msg = export_widget(&aura_widget)?;
//! let json = serde_json::to_string(&a2ui_msg)?;
//!
//! // Import A2UI JSON to AURA widget
//! let a2ui_msg: A2UIMessage = serde_json::from_str(json)?;
//! let aura_widget = import_message(&a2ui_msg)?;
//! ```

pub mod export;
pub mod import;
pub mod schema;

// Re-export main types for convenience
pub use export::export_widget;
pub use import::import_message;
pub use schema::*;

use std::fmt;

/// Errors that can occur during A2UI conversion.
#[derive(Debug, Clone, PartialEq)]
pub enum A2UIError {
    /// Component type has no A2UI equivalent.
    UnsupportedComponent(String),
    /// Expression type cannot be represented in A2UI.
    UnsupportedExpression(String),
    /// Value format is invalid.
    InvalidValue(String),
    /// Required field is missing.
    MissingRequiredField(String),
    /// JSON serialization/deserialization error.
    Serde(String),
}

impl fmt::Display for A2UIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            A2UIError::UnsupportedComponent(tag) => {
                write!(f, "A2UI export: unsupported component tag '{}'", tag)
            }
            A2UIError::UnsupportedExpression(expr) => {
                write!(f, "A2UI export: unsupported expression '{}'", expr)
            }
            A2UIError::InvalidValue(val) => {
                write!(f, "A2UI import: invalid value '{}'", val)
            }
            A2UIError::MissingRequiredField(field) => {
                write!(f, "A2UI import: missing required field '{}'", field)
            }
            A2UIError::Serde(msg) => {
                write!(f, "A2UI JSON error: {}", msg)
            }
        }
    }
}

impl std::error::Error for A2UIError {}

impl From<serde_json::Error> for A2UIError {
    fn from(err: serde_json::Error) -> Self {
        A2UIError::Serde(err.to_string())
    }
}
