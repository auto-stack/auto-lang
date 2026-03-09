pub mod ninja;
pub use ninja::*;
mod tool;
pub use tool::*;
mod cargo;
pub use cargo::*;
mod vue;
pub use vue::*;

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
    fn enable_memory_output(&mut self) -> AutoResult<()>;

    /// Get captured memory output
    fn get_memory_output(&self) -> HashMap<String, Vec<u8>>;
}

pub enum BuilderKind {
    Ninja(AutoPath), // Ninja with path to build folder
    Cargo(AutoPath), // Cargo with path to Cargo.toml directory
    Vue(AutoPath),   // Vue with path to project directory (containing package.json)
}

impl BuilderKind {
    /// Create a BuilderKind from a string identifier
    pub fn from_str(builder_name: &str, path: AutoPath) -> Option<Self> {
        match builder_name {
            "ninja" => {
                let path = path.join("build.ninja");
                Some(BuilderKind::Ninja(path))
            }
            "cargo" => {
                let path = path.join("Cargo.toml");
                Some(BuilderKind::Cargo(path))
            }
            "vue" => {
                // Vue project: path is the dist directory (where package.json is)
                Some(BuilderKind::Vue(path))
            }
            _ => None,
        }
    }

    /// Create a builder instance from this BuilderKind
    pub fn create_builder(&self) -> Box<dyn Builder> {
        match self {
            BuilderKind::Ninja(path) => Box::new(NinjaBuilder::new(path.clone())),
            BuilderKind::Cargo(path) => Box::new(CargoBuilder::new(path.clone())),
            BuilderKind::Vue(path) => Box::new(VueBuilder::new(path.clone())),
        }
    }
}

pub fn new_builder(kind: BuilderKind) -> Box<dyn Builder> {
    kind.create_builder()
}

/// Create a builder from a string builder type and path
pub fn make_builder(builder: &AutoStr, path: AutoPath) -> Option<Box<dyn Builder>> {
    BuilderKind::from_str(builder.as_str(), path).map(|kind| kind.create_builder())
}
