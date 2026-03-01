//! API Code Generation Targets
//!
//! Plan 102 Phase 5.2-5.3: Code generators for different deployment targets

mod typescript;
mod tauri;
mod axum;

pub use typescript::TypeScriptGenerator;
pub use tauri::TauriGenerator;
pub use axum::AxumGenerator;

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

    /// Axum routes (Rust HTTP server)
    Axum,

    /// TypeScript types and API client
    TypeScript,
}

impl Target {
    /// Get generator for this target
    pub fn generator(&self) -> Box<dyn TargetGenerator> {
        match self {
            Target::Tauri => Box::new(TauriGenerator::new()),
            Target::Axum => Box::new(AxumGenerator::new()),
            Target::TypeScript => Box::new(TypeScriptGenerator::new()),
        }
    }
}
