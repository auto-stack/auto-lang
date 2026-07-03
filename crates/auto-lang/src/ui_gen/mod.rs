//! UI Backend Generators
//!
//! This module provides code generators for various UI backends:
//! - **Vue3/JavaScript**: Vue3 SFC (Single File Component) generator
//! - **Rust/AutoUI**: Rust Component trait generator using auto-ui abstraction
//! - **Jetpack Compose**: Kotlin/Compose for Android
//!
//! The Rust generator produces code using the abstract AutoUI components from
//! the `auto-ui` crate. The auto-ui crate then handles backend-specific
//! implementations (Iced, GPUI, etc.).
//!
//! All generators take `AuraWidget` as input and produce target-specific code.

pub mod shared;
pub mod ts_adapter;
pub mod vue;
pub mod block;
pub mod rust;
pub mod style;
pub mod jet;
pub mod ark;
pub mod ark_adapter;
pub mod widget;
pub mod api;

// Re-export main types
pub use vue::VueGenerator;
pub use vue::VueMode;
pub use rust::RustGenerator;
pub use style::StyleGenerator;
pub use jet::JetGenerator;
pub use widget::{WidgetCategory, WidgetRegistry, WidgetSpec};

// Re-export transpiler API (Plan 175 Phase 3)
pub use api::{transpile_file, transpile_aura, transpile_vue_aura};

use crate::aura::AuraWidget;

/// Generation error
#[derive(Debug, Clone)]
pub enum GenError {
    /// Unsupported expression type
    UnsupportedExpr(String),

    /// Unsupported statement type
    UnsupportedStmt(String),

    /// Invalid state reference
    InvalidStateRef(String),

    /// IO error
    Io(String),

    /// Unknown widget requested from the library template table (Plan 331)
    UnknownWidget(String),
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenError::UnsupportedExpr(msg) => write!(f, "Unsupported expression: {}", msg),
            GenError::UnsupportedStmt(msg) => write!(f, "Unsupported statement: {}", msg),
            GenError::InvalidStateRef(msg) => write!(f, "Invalid state reference: {}", msg),
            GenError::Io(msg) => write!(f, "IO error: {}", msg),
            GenError::UnknownWidget(msg) => write!(f, "Unknown widget: {}", msg),
        }
    }
}

impl std::error::Error for GenError {}

pub type GenResult<T> = Result<T, GenError>;

/// Backend generator trait
pub trait BackendGenerator {
    /// Generate code from an AuraWidget
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String>;

    /// Get the file extension for generated code
    fn extension(&self) -> &'static str;
}
