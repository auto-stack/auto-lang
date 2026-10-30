//! API Code Generation Targets
//!
//! Plan 102 Phase 5.2-5.3: Code generators for different deployment targets

mod tauri;

pub use tauri::TauriGenerator;

use crate::api::ApiModule;

/// Code generation target
pub trait TargetGenerator {
    /// Generate code for the target
    fn generate(&self, module: &ApiModule) -> String;

    /// Get file extension for generated code
    fn extension(&self) -> &str;

    /// Get subdirectory name for this target
    fn subdirectory(&self) -> &str;
}

/// Generation target types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Tauri commands (Rust)
    Tauri,
}

impl Target {
    /// Get generator for this target
    pub fn generator(&self) -> Box<dyn TargetGenerator> {
        match self {
            Target::Tauri => Box::new(TauriGenerator::new()),
        }
    }
}
