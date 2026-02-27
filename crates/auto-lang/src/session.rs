//! Compiler Session - Scenario-based compilation context
//!
//! This module implements the "Scenario Programming" architecture where
//! the compiler's behavior is driven by the current scenario (Core, UI, Shell).
//!
//! ## Key Concepts
//!
//! - **Scenario**: The compilation context (Core, UI, Shell)
//! - **CompilerSession**: Carries scenario info throughout compilation
//! - **Contextual Keywords**: Keywords like `widget` only active in UI scenario
//!
//! ## Usage
//!
//! ```rust
//! use auto_lang::session::{CompilerSession, Scenario};
//!
//! // Create a UI scenario session
//! let session = CompilerSession::new(Scenario::UI);
//!
//! // Create with backend
//! let session = CompilerSession::new(Scenario::UI).with_backend("react");
//! ```

use std::fmt;

/// Compilation scenario - determines available syntax and features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Scenario {
    /// Standard Auto language - all core features
    #[default]
    Core,

    /// UI scenario - enables widget, view, model, msg, on keywords
    UI,

    /// Shell scenario - for scripting and CLI tools
    Shell,
}

impl Scenario {
    /// Check if this is the UI scenario
    pub fn is_ui(&self) -> bool {
        matches!(self, Scenario::UI)
    }

    /// Check if this is the Core scenario
    pub fn is_core(&self) -> bool {
        matches!(self, Scenario::Core)
    }

    /// Parse scenario from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "core" | "default" => Some(Scenario::Core),
            "ui" | "gui" => Some(Scenario::UI),
            "shell" | "script" => Some(Scenario::Shell),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Scenario::Core => "core",
            Scenario::UI => "ui",
            Scenario::Shell => "shell",
        }
    }
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Compiler session - carries scenario and configuration throughout compilation
///
/// This is the single source of truth for:
/// - Current scenario (Core, UI, Shell)
/// - Target backend (react, compose, gpui, etc.)
/// - Feature flags and options
#[derive(Debug, Clone)]
pub struct CompilerSession {
    /// The current compilation scenario
    pub scenario: Scenario,

    /// Target backend for code generation (e.g., "react", "compose", "gpui")
    pub backend: Option<String>,

    /// Whether to emit debug information
    pub debug: bool,

    /// Whether to enable incremental compilation
    pub incremental: bool,
}

impl CompilerSession {
    /// Create a new session with the given scenario
    pub fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            backend: None,
            debug: false,
            incremental: false,
        }
    }

    /// Create a Core scenario session
    pub fn core() -> Self {
        Self::new(Scenario::Core)
    }

    /// Create a UI scenario session
    pub fn ui() -> Self {
        Self::new(Scenario::UI)
    }

    /// Create a Shell scenario session
    pub fn shell() -> Self {
        Self::new(Scenario::Shell)
    }

    /// Set the target backend
    pub fn with_backend(mut self, backend: impl Into<String>) -> Self {
        self.backend = Some(backend.into());
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Enable incremental compilation
    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }

    /// Check if this is a UI scenario
    pub fn is_ui(&self) -> bool {
        self.scenario.is_ui()
    }

    /// Check if this is a Core scenario
    pub fn is_core(&self) -> bool {
        self.scenario.is_core()
    }

    /// Get the backend, or a default based on scenario
    pub fn backend_or_default(&self) -> &str {
        self.backend.as_deref().unwrap_or_else(|| match self.scenario {
            Scenario::Core => "a2r",
            Scenario::UI => "gpui",
            Scenario::Shell => "vm",
        })
    }
}

impl Default for CompilerSession {
    fn default() -> Self {
        Self::core()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_from_str() {
        assert_eq!(Scenario::from_str("core"), Some(Scenario::Core));
        assert_eq!(Scenario::from_str("UI"), Some(Scenario::UI));
        assert_eq!(Scenario::from_str("shell"), Some(Scenario::Shell));
        assert_eq!(Scenario::from_str("unknown"), None);
    }

    #[test]
    fn test_scenario_is_methods() {
        assert!(Scenario::UI.is_ui());
        assert!(!Scenario::UI.is_core());

        assert!(Scenario::Core.is_core());
        assert!(!Scenario::Core.is_ui());
    }

    #[test]
    fn test_session_builder() {
        let session = CompilerSession::ui()
            .with_backend("react")
            .with_debug(true);

        assert!(session.is_ui());
        assert_eq!(session.backend, Some("react".to_string()));
        assert!(session.debug);
    }

    #[test]
    fn test_backend_or_default() {
        let core_session = CompilerSession::core();
        assert_eq!(core_session.backend_or_default(), "a2r");

        let ui_session = CompilerSession::ui();
        assert_eq!(ui_session.backend_or_default(), "gpui");

        let shell_session = CompilerSession::shell();
        assert_eq!(shell_session.backend_or_default(), "vm");
    }
}
