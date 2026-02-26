mod cmake;
pub use cmake::*;
mod iar;
pub use iar::*;
mod ghs;
pub use ghs::*;
pub mod ninja;
pub use ninja::*;
mod tool;
pub use tool::*;
mod cargo;
pub use cargo::*;

use crate::AutoResult;
use crate::{Pac, Target};
use auto_val::{AutoPath, AutoStr};
use std::collections::HashMap;

pub trait Builder {
    fn build(&mut self, pac: &mut Pac) -> AutoResult<()>;
    fn setup(&mut self, pac: &mut Pac) -> AutoResult<()>;
    fn finish(&mut self, pac: &Pac) -> AutoResult<()>;
    fn target(&mut self, target: &Target, pac: &Pac) -> AutoResult<()>;
    fn clean(&mut self) -> AutoResult<()>;
    fn run(&mut self, pac: &Pac, args: Vec<String>) -> AutoResult<()>;

    /// Enable memory output mode for testing
    ///
    /// When enabled, builder output is captured in memory instead of writing to disk.
    /// This allows tests to verify generated content without file I/O.
    fn enable_memory_output(&mut self) -> AutoResult<()>;

    /// Get captured memory output
    ///
    /// Returns a map of filename -> content bytes.
    /// Only valid after enable_memory_output() has been called and build() has completed.
    fn get_memory_output(&self) -> HashMap<String, Vec<u8>>;
}

pub enum BuilderKind {
    CMake(String),   // Cmake with path to CMakeLists.txt
    IAR(AutoPath),   // IAR with path to project folder
    GHS(AutoPath),   // GHS with path to project folder
    Ninja(AutoPath), // Ninja with path to build folder
    Cargo(AutoPath), // Cargo with path to Cargo.toml directory
}

impl BuilderKind {
    /// Create a BuilderKind from a string identifier
    pub fn from_str(builder_name: &str, path: AutoPath) -> Option<Self> {
        match builder_name {
            "cmake" => {
                let path = path.join("CMakeLists.txt");
                Some(BuilderKind::CMake(path.to_string()))
            }
            "iar" => Some(BuilderKind::IAR(path)),
            "ghs" => Some(BuilderKind::GHS(path)),
            "ninja" => {
                let path = path.join("build.ninja");
                Some(BuilderKind::Ninja(path))
            }
            "cargo" => {
                let path = path.join("Cargo.toml");
                Some(BuilderKind::Cargo(path))
            }
            _ => None,
        }
    }

    /// Create a builder instance from this BuilderKind
    pub fn create_builder(&self) -> Box<dyn Builder> {
        match self {
            BuilderKind::CMake(path) => Box::new(CMakeBuilder::new(path)),
            BuilderKind::IAR(path) => Box::new(IARBuilder::new(path.clone())),
            BuilderKind::GHS(path) => Box::new(GHSBuilder::new(path.clone())),
            BuilderKind::Ninja(path) => Box::new(NinjaBuilder::new(path.clone())),
            BuilderKind::Cargo(path) => Box::new(CargoBuilder::new(path.clone())),
        }
    }
}

pub fn new_builder(kind: BuilderKind) -> Box<dyn Builder> {
    kind.create_builder()
}

/// Create a builder from a string builder type and path
///
/// Returns None if the builder type is unknown
pub fn make_builder(builder: &AutoStr, path: AutoPath) -> Option<Box<dyn Builder>> {
    BuilderKind::from_str(builder.as_str(), path).map(|kind| kind.create_builder())
}
