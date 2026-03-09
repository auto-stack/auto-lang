//! Form Component Generators
//!
//! Generates Jetpack Compose form components from AURA elements.
//!
//! ## Supported Components
//! - `input` → `OutlinedTextField`
//! - `textarea` → `OutlinedTextField` (multi-line)
//! - `checkbox` → `Checkbox`
//! - `switch`/`toggle` → `Switch`
//! - `slider` → `Slider`

// TODO: Will use these imports in Phase 2 implementation
// use crate::aura::{AuraPropValue, AuraExpr};
// use std::collections::HashMap;

/// Form component generator
pub struct FormGenerator {
    /// Track imports needed for form components
    imports: Vec<String>,
}

impl FormGenerator {
    /// Create a new form generator
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
        }
    }

    /// Get required imports for generated form components
    pub fn get_imports(&self) -> &[String] {
        &self.imports
    }

    /// Clear imports for fresh generation
    pub fn clear_imports(&mut self) {
        self.imports.clear();
    }

    /// Add import if not already present
    pub fn add_import(&mut self, import: &str) {
        if !self.imports.iter().any(|i| i == import) {
            self.imports.push(import.to_string());
        }
    }
}

impl Default for FormGenerator {
    fn default() -> Self {
        Self::new()
    }
}
