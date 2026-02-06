use std::env;

use auto_val::AutoStr;

// Phase 3: Use real CompilerConfig from builder/ninja/config
use crate::builder::ninja::config::CompilerConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    pub name: AutoStr,
    pub builder: AutoStr,
    pub platform: AutoStr,
    pub at: AutoStr,
    /// 编译器配置 (可选)
    pub compiler: Option<CompilerConfig>,
}

impl Default for Port {
    fn default() -> Self {
        // Phase 3: Use real CompilerConfig helper methods
        match env::consts::OS {
            "windows" => {
                let compiler = CompilerConfig::msvc_default();
                Self {
                    name: "windows_ninja".into(),
                    builder: "ninja".into(),
                    platform: "windows".into(),
                    at: "build".into(),
                    compiler: Some(compiler),
                }
            }
            _ => {
                let compiler = CompilerConfig::gcc_default();
                Self {
                    name: "linux_ninja".into(),
                    builder: "ninja".into(),
                    platform: "linux".into(),
                    at: "build".into(),
                    compiler: Some(compiler),
                }
            }
        }
    }
}
