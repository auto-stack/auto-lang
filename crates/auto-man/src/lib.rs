// Plan 078-079: AutoMan Migration - AutoLang Monorepo Edition
//
// This crate is being migrated from ../auto-man into the auto-lang workspace
// to provide package management and dependency resolution for AutoLang projects.

// Plan 078: ModuleResolver trait implementation
pub mod error;
pub mod resolver;

// Phase 1: Core modules (from ../auto-man)
pub mod asset;
pub mod file_types;
pub mod fs;
pub mod group;
pub mod node_ext;
pub mod port;
pub mod pull;
pub mod up;
pub mod util;
pub mod version;

// Phase 2: Package management (from ../auto-man)
pub mod git;
pub mod index;
pub mod lock;

// Phase 3: Build system (from ../auto-man)
pub mod builder;

// Phase 4: Target & Scanner (from ../auto-man)
pub mod cache;
pub mod dir;
pub mod scanner;
pub mod target;

// Phase 2B: Package management (completed - depends on Phase 3 & 4)
pub mod automan;
pub mod pac;

// NOTE: stubs.rs removed in Phase 6 - all types migrated

// Re-exports (Plan 078 + Phase 1)
pub use asset::*;
pub use error::*;
pub use file_types::*;
pub use group::*;
pub use node_ext::*;
pub use port::*;
pub use resolver::AutoManResolver;

pub use up::*;
pub use util::*;
pub use version::*;

// Re-exports (Phase 2)
pub use git::*;
pub use index::*;
pub use lock::*;
// Re-exports (Phase 3)
pub use builder::*;
// Re-exports (Phase 4)
pub use cache::*;
pub use dir::*;
pub use scanner::*;
pub use target::*;
// Re-exports from Phase 3 (builder/ninja/config)
pub use builder::ninja::config::CompilerConfig;
// Re-exports (Phase 2B)
pub use automan::*;
pub use pac::*;

// NOTE: All types migrated in Phases 1-5
// NOTE: stubs.rs removed in Phase 6 (cleanup)

// AutoVal re-exports
pub use auto_val::AutoError;
pub use auto_val::AutoResult;

/// AutoMan version
pub const AUTOMAN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// AutoMan - Package manager for AutoLang projects
///
/// This is a stub during migration. Full functionality will be added
/// as modules are migrated from ../auto-man.
pub struct AutoMan {
    /// Project root directory
    root: std::path::PathBuf,
}

impl AutoMan {
    /// Create a new AutoMan instance for the given project root
    pub fn new(root: std::path::PathBuf) -> Self {
        Self { root }
    }

    /// Get the project root directory
    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    /// Initialize AutoMan in a new project
    ///
    /// This creates the necessary configuration files and directory structure
    pub fn init(&self) -> Result<(), AutoManError> {
        // TODO: Implement pac.at creation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automan_creation() {
        let automan = AutoMan::new(std::path::PathBuf::from("."));
        assert_eq!(automan.root(), std::path::Path::new("."));
    }

    #[test]
    fn test_version() {
        assert!(!AUTOMAN_VERSION.is_empty());
    }
}
