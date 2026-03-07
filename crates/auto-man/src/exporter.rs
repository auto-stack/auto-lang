mod cmake;
pub use cmake::*;
mod iar;
pub use iar::*;
mod ghs;
pub use ghs::*;

use crate::AutoResult;
use crate::Pac;
use auto_val::AutoPath;
use std::collections::HashMap;

pub trait Exporter {
    /// Export the given pac to the target format
    fn export(&mut self, pac: &mut Pac) -> AutoResult<()>;

    /// Enable memory output mode for testing
    fn enable_memory_output(&mut self) -> AutoResult<()>;

    /// Get captured memory output
    fn get_memory_output(&self) -> HashMap<String, Vec<u8>>;
}

pub enum ExporterKind {
    CMake(String), // Path to CMakeLists.txt
    IAR(AutoPath), // Path to IAR workspace
    GHS(AutoPath), // Path to GHS workspace
}

impl ExporterKind {
    pub fn from_str(exporter_name: &str, path: AutoPath) -> Option<Self> {
        match exporter_name {
            "cmake" => {
                let path = path.join("CMakeLists.txt");
                Some(ExporterKind::CMake(path.to_string()))
            }
            "iar" => Some(ExporterKind::IAR(path)),
            "ghs" => Some(ExporterKind::GHS(path)),
            _ => None,
        }
    }

    pub fn create_exporter(&self) -> Box<dyn Exporter> {
        match self {
            ExporterKind::CMake(path) => Box::new(CMakeExporter::new(path)),
            ExporterKind::IAR(path) => Box::new(IARExporter::new(path.clone())),
            ExporterKind::GHS(path) => Box::new(GHSExporter::new(path.clone())),
        }
    }
}

pub fn make_exporter(exporter: &str, path: AutoPath) -> Option<Box<dyn Exporter>> {
    ExporterKind::from_str(exporter, path).map(|kind| kind.create_exporter())
}
