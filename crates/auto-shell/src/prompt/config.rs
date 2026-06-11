//! TOML-based prompt configuration
//!
//! Loads from `~/.config/ash-prompt.toml`. Falls back to defaults if file is missing.

use std::collections::HashMap;
use std::path::PathBuf;

/// Prompt configuration loaded from TOML
#[derive(Debug, Clone)]
pub struct AshConfig {
    /// Left prompt format string ($module_name placeholders)
    pub format: String,
    /// Right prompt format string
    pub right_format: String,
    /// Add newline before prompt
    pub add_newline: bool,
    /// Minimum command duration (ms) to show cmd_duration module
    pub cmd_duration_threshold: u64,
    /// Per-module config (key = module name, value = TOML sub-table)
    pub module_configs: HashMap<String, toml::Value>,
}

impl Default for AshConfig {
    fn default() -> Self {
        Self {
            format: "$directory$git_branch$git_status$cmd_duration$character".to_string(),
            right_format: "$time".to_string(),
            add_newline: false,
            cmd_duration_threshold: 2000,
            module_configs: HashMap::new(),
        }
    }
}

impl AshConfig {
    /// Load config from `~/.config/ash-prompt.toml`, falling back to default
    pub fn load() -> Self {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ash-prompt.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            Self::parse(&content)
        } else {
            Self::default()
        }
    }

    /// Parse TOML content into config
    pub fn parse(content: &str) -> Self {
        let value: toml::Value = match toml::from_str(content) {
            Ok(v) => v,
            Err(_) => return Self::default(),
        };

        let table = match value.as_table() {
            Some(t) => t,
            None => return Self::default(),
        };

        let mut config = Self::default();

        if let Some(v) = table.get("format").and_then(|v| v.as_str()) {
            config.format = v.to_string();
        }
        if let Some(v) = table.get("right_format").and_then(|v| v.as_str()) {
            config.right_format = v.to_string();
        }
        if let Some(v) = table.get("add_newline").and_then(|v| v.as_bool()) {
            config.add_newline = v;
        }
        if let Some(v) = table
            .get("cmd_duration_threshold")
            .and_then(|v| v.as_integer())
        {
            config.cmd_duration_threshold = v as u64;
        }

        // Collect per-module config tables
        for (key, val) in table {
            if val.is_table() {
                config
                    .module_configs
                    .insert(key.clone(), val.clone());
            }
        }

        config
    }

    /// Get config for a specific module
    pub fn module_config(&self, name: &str) -> Option<&toml::Value> {
        self.module_configs.get(name)
    }

    /// Check if a module is disabled
    pub fn is_module_disabled(&self, name: &str) -> bool {
        self.module_configs
            .get(name)
            .and_then(|v| v.get("disabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Get a string config value for a module
    pub fn module_string(&self, module: &str, key: &str, default: &str) -> String {
        self.module_configs
            .get(module)
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    /// Get an integer config value for a module
    pub fn module_int(&self, module: &str, key: &str, default: i64) -> i64 {
        self.module_configs
            .get(module)
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_integer())
            .unwrap_or(default)
    }

    /// Get a boolean config value for a module
    pub fn module_bool(&self, module: &str, key: &str, default: bool) -> bool {
        self.module_configs
            .get(module)
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AshConfig::default();
        assert!(!config.add_newline);
        assert_eq!(config.cmd_duration_threshold, 2000);
        assert!(config.format.contains("$directory"));
        assert!(config.format.contains("$character"));
    }

    #[test]
    fn test_parse_empty() {
        let config = AshConfig::parse("");
        assert_eq!(config.format, AshConfig::default().format);
    }

    #[test]
    fn test_parse_full() {
        let toml = r#"
format = "$directory$character"
right_format = "$time"
add_newline = true
cmd_duration_threshold = 5000

[directory]
style = "cyan bold"
truncation_length = 3

[git_branch]
disabled = true

[cmd_duration]
min_time = 3000
"#;
        let config = AshConfig::parse(toml);
        assert_eq!(config.format, "$directory$character");
        assert!(config.add_newline);
        assert_eq!(config.cmd_duration_threshold, 5000);
        assert!(config.is_module_disabled("git_branch"));
        assert!(!config.is_module_disabled("directory"));
        assert_eq!(config.module_string("directory", "style", ""), "cyan bold");
        assert_eq!(config.module_int("directory", "truncation_length", 0), 3);
        assert_eq!(config.module_int("cmd_duration", "min_time", 0), 3000);
    }

    #[test]
    fn test_invalid_toml_falls_back() {
        let config = AshConfig::parse("not valid toml {{{");
        assert_eq!(config.format, AshConfig::default().format);
    }
}
