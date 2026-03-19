//! UI Artifact for incremental code generation (Plan 134)
//!
//! Tracks generated UI files (.vue, .kt) for incremental compilation.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// A generated UI artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIArtifact {
    /// Source .at file path (relative to project root)
    pub source_path: PathBuf,
    /// Widget name extracted from source
    pub widget_name: String,
    /// Generated output file path (relative to output directory)
    pub output_path: PathBuf,
    /// Hash of source file content (BLAKE3 truncated to u64)
    pub source_hash: u64,
    /// Hash of generated content
    pub content_hash: u64,
    /// Target backend
    pub backend: UIBackend,
}

/// UI backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UIBackend {
    Vue,
    Jet,
    Tauri,
}

impl std::fmt::Display for UIBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UIBackend::Vue => write!(f, "vue"),
            UIBackend::Jet => write!(f, "jet"),
            UIBackend::Tauri => write!(f, "tauri"),
        }
    }
}

impl std::str::FromStr for UIBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vue" => Ok(UIBackend::Vue),
            "jet" => Ok(UIBackend::Jet),
            "tauri" => Ok(UIBackend::Tauri),
            _ => Err(format!("Unknown UI backend: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_backend_display() {
        assert_eq!(UIBackend::Vue.to_string(), "vue");
        assert_eq!(UIBackend::Jet.to_string(), "jet");
        assert_eq!(UIBackend::Tauri.to_string(), "tauri");
    }

    #[test]
    fn test_ui_backend_from_str() {
        assert_eq!("vue".parse::<UIBackend>().unwrap(), UIBackend::Vue);
        assert_eq!("JET".parse::<UIBackend>().unwrap(), UIBackend::Jet);
    }
}
