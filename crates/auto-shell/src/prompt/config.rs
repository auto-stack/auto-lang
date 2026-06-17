//! Prompt configuration (Plan 318 — Auto/Atom `.at` format).
//!
//! Loads from `~/.config/ash/prompt.at` (preferred) or `~/.config/ash-prompt.toml`
//! (backward compat). The per-module config is stored as a flat string map
//! (`module_name → (key → value)`) rather than toml::Value, eliminating the
//! TOML dependency from the prompt config path.

use std::collections::HashMap;

/// Prompt configuration.
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
    /// Per-module config: module_name → (key → string value).
    pub module_configs: HashMap<String, HashMap<String, String>>,
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
    /// Load config: prefer `prompt.at` (Auto/Atom), fall back to `ash-prompt.toml`.
    pub fn load() -> Self {
        // 1. Try prompt.at (Auto/Atom, Plan 318).
        let auto_cfg = crate::auto_config::load_file("prompt.at");
        if !auto_cfg.is_empty() {
            return Self::from_auto_config(&auto_cfg);
        }
        // 2. Fall back to ash-prompt.toml (TOML, backward compat).
        let path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("ash-prompt.toml");
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            Self::parse_toml(&content)
        } else {
            Self::default()
        }
    }

    /// Build from Auto/Atom parsed config map.
    fn from_auto_config(cfg: &HashMap<String, HashMap<String, String>>) -> Self {
        let mut config = Self::default();
        // Top-level prompt fields.
        if let Some(prompt) = cfg.get("prompt") {
            if let Some(v) = prompt.get("format") {
                config.format = v.clone();
            }
            if let Some(v) = prompt.get("right_format") {
                config.right_format = v.clone();
            }
            if let Some(v) = prompt.get("add_newline") {
                config.add_newline = parse_bool(v).unwrap_or(false);
            }
            if let Some(v) = prompt.get("cmd_duration_threshold") {
                config.cmd_duration_threshold = v.parse().unwrap_or(2000);
            }
        }
        // Per-module: blocks named "prompt.<module>".
        for (block, entries) in cfg {
            if let Some(module) = block.strip_prefix("prompt.") {
                config
                    .module_configs
                    .insert(module.to_string(), entries.clone());
            }
        }
        config
    }

    /// Parse TOML content (backward compat for ash-prompt.toml).
    fn parse_toml(content: &str) -> Self {
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
        if let Some(v) = table.get("cmd_duration_threshold").and_then(|v| v.as_integer()) {
            config.cmd_duration_threshold = v as u64;
        }
        // Per-module tables → flatten to string maps.
        for (key, val) in table {
            if val.is_table() {
                let mut module_map = HashMap::new();
                for (k, v) in val.as_table().unwrap() {
                    let s = match v {
                        toml::Value::String(s) => s.clone(),
                        toml::Value::Integer(i) => i.to_string(),
                        toml::Value::Boolean(b) => b.to_string(),
                        _ => continue,
                    };
                    module_map.insert(k.clone(), s);
                }
                config.module_configs.insert(key.clone(), module_map);
            }
        }
        config
    }

    // ── Module-level accessors (same API as before, new storage) ──────────

    /// Check if a module is disabled.
    pub fn is_module_disabled(&self, name: &str) -> bool {
        self.module_bool(name, "disabled", false)
    }

    /// Get a string config value for a module.
    pub fn module_string(&self, module: &str, key: &str, default: &str) -> String {
        self.module_configs
            .get(module)
            .and_then(|m| m.get(key))
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Get an integer config value for a module.
    pub fn module_int(&self, module: &str, key: &str, default: i64) -> i64 {
        self.module_configs
            .get(module)
            .and_then(|m| m.get(key))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(default)
    }

    /// Get a boolean config value for a module.
    pub fn module_bool(&self, module: &str, key: &str, default: bool) -> bool {
        self.module_configs
            .get(module)
            .and_then(|m| m.get(key))
            .and_then(|v| parse_bool(v))
            .unwrap_or(default)
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = AshConfig::default();
        assert!(config.format.contains("$directory"));
        assert!(!config.add_newline);
    }

    #[test]
    fn from_auto_config_reads_prompt_block() {
        let cfg = crate::auto_config::parse_auto_config(
            r#"
            prompt {
                format : "$directory$character"
                add_newline : true
                cmd_duration_threshold : 5000
                git_branch {
                    symbol : "⎇ "
                    disabled : true
                }
            }
            "#,
        );
        let config = AshConfig::from_auto_config(&cfg);
        assert_eq!(config.format, "$directory$character");
        assert!(config.add_newline);
        assert_eq!(config.cmd_duration_threshold, 5000);
        assert_eq!(config.module_string("git_branch", "symbol", ""), "⎇ ");
        assert!(config.is_module_disabled("git_branch"));
    }

    #[test]
    fn toml_backward_compat() {
        let config = AshConfig::parse_toml(
            r#"
format = "$character"
add_newline = true

[git_branch]
symbol = "❯"
disabled = false
"#,
        );
        assert_eq!(config.format, "$character");
        assert!(config.add_newline);
        assert_eq!(config.module_string("git_branch", "symbol", ""), "❯");
        assert!(!config.is_module_disabled("git_branch"));
    }
}
