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
// Plan 482: nav-item/nav-group class-token contract (Vue scaffold ↔ VM builder).
pub mod nav_contract;
pub mod ts_adapter;
pub mod vue;
pub mod block;
pub mod rust;
pub mod style;
pub mod jet;
pub mod ark;
pub mod ark_adapter;
pub mod kotlin_adapter;
pub mod widget;
pub mod api;
pub mod validators;
pub mod docs_gen;

// Re-export main types
pub use vue::VueGenerator;
pub use vue::VueMode;
pub use rust::RustGenerator;
pub use style::StyleGenerator;
pub use jet::JetGenerator;
pub use widget::{WidgetCategory, WidgetRegistry, WidgetSpec};
pub use validators::{validate_sfc, ValidationContext, ValidationWarning, Severity};

// Re-export transpiler API (Plan 175 Phase 3 + Plan 361 §3)
pub use api::{
    transpile_file, transpile_aura, transpile_vue_aura,
    generate_component_from_file, ComponentGenOptions, GeneratedComponent,
};

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

/// Normalize a display shortcut (`Ctrl+N`, `alt+f4`) to a keyboard listener's
/// lookup form: `Ctrl+`/`Alt+` prefixes + key. Single alpha chars are
/// lowercased (the OS reports the base character with Ctrl/Alt held); named
/// keys (F4, Enter…) pass through as-is.
///
/// Lives here (ungated) rather than in `ui::action_config` so the vue
/// transpiler — which compiles without the `ui` feature — can use it
/// (plan-451 regression: `ui_gen/vue.rs` referenced the gated path and broke
/// every non-`ui` consumer, e.g. auto-shell).
pub fn normalize_shortcut(s: &str) -> String {
    let mut ctrl = false;
    let mut alt = false;
    let mut key = String::new();
    for part in s.split('+') {
        let raw = part.trim();
        match raw.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" => alt = true,
            "shift" => {} // Shift is expressed by the shifted character itself
            "" => {}
            _ => key = raw.to_string(),
        }
    }
    if key.is_empty() {
        return String::new();
    }
    if key.len() == 1 {
        key = key.to_lowercase();
    }
    let mut out = String::new();
    if ctrl {
        out.push_str("Ctrl+");
    }
    if alt {
        out.push_str("Alt+");
    }
    out.push_str(&key);
    out
}
