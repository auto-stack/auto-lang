//! ASH shell configuration (`~/.config/ash.toml`)
//!
//! This is the unified configuration file for shell behavior settings.
//! Prompt configuration remains in `ash-prompt.toml` (loaded by `prompt/config.rs`).
//!
//! # Example `~/.config/ash.toml`
//!
//! ```toml
//! [shell]
//! history_size = 10000
//! autosuggestion = true
//! autosuggestion_min_chars = 1
//! edit_mode = "emacs"     # "emacs" or "vi"
//! syntax_highlighting = true
//!
//! [aliases]
//! ll = "ls -la"
//! la = "ls -a"
//! gs = "git status"
//!
//! [completion]
//! case_sensitive = false
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

/// Shell behavior configuration loaded from `~/.config/ash.toml`
#[derive(Debug, Clone)]
pub struct AshShellConfig {
    /// Maximum number of history entries to keep
    pub history_size: usize,
    /// Enable Fish-style autosuggestion hints
    pub autosuggestion: bool,
    /// Minimum characters before autosuggestion triggers
    pub autosuggestion_min_chars: usize,
    /// Edit mode: "emacs" or "vi"
    pub edit_mode: String,
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    /// Pre-configured aliases
    pub aliases: HashMap<String, String>,
    /// Case-sensitive tab completion
    pub completion_case_sensitive: bool,
}

impl Default for AshShellConfig {
    fn default() -> Self {
        Self {
            history_size: 10000,
            autosuggestion: true,
            autosuggestion_min_chars: 1,
            edit_mode: "emacs".to_string(),
            syntax_highlighting: true,
            aliases: HashMap::new(),
            completion_case_sensitive: false,
        }
    }
}

impl AshShellConfig {
    /// Load config from `~/.config/ash.toml`, falling back to default.
    pub fn load() -> Self {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ash.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            Self::parse(&content)
        } else {
            Self::default()
        }
    }

    /// Parse TOML content into shell config.
    pub fn parse(content: &str) -> Self {
        let value: toml::Value = match toml::from_str(content) {
            Ok(v) => v,
            Err(_) => return Self::default(),
        };

        let mut config = Self::default();

        // [shell] section
        if let Some(shell) = value.get("shell").and_then(|v| v.as_table()) {
            if let Some(v) = shell.get("history_size").and_then(|v| v.as_integer()) {
                config.history_size = v as usize;
            }
            if let Some(v) = shell.get("autosuggestion").and_then(|v| v.as_bool()) {
                config.autosuggestion = v;
            }
            if let Some(v) = shell.get("autosuggestion_min_chars").and_then(|v| v.as_integer()) {
                config.autosuggestion_min_chars = v as usize;
            }
            if let Some(v) = shell.get("edit_mode").and_then(|v| v.as_str()) {
                config.edit_mode = v.to_string();
            }
            if let Some(v) = shell.get("syntax_highlighting").and_then(|v| v.as_bool()) {
                config.syntax_highlighting = v;
            }
        }

        // [aliases] section
        if let Some(aliases) = value.get("aliases").and_then(|v| v.as_table()) {
            for (key, val) in aliases {
                if let Some(s) = val.as_str() {
                    config.aliases.insert(key.clone(), s.to_string());
                }
            }
        }

        // [completion] section
        if let Some(completion) = value.get("completion").and_then(|v| v.as_table()) {
            if let Some(v) = completion.get("case_sensitive").and_then(|v| v.as_bool()) {
                config.completion_case_sensitive = v;
            }
        }

        config
    }

    /// Should we use vi mode?
    pub fn is_vi_mode(&self) -> bool {
        self.edit_mode == "vi"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AshShellConfig::default();
        assert_eq!(config.history_size, 10000);
        assert!(config.autosuggestion);
        assert_eq!(config.autosuggestion_min_chars, 1);
        assert_eq!(config.edit_mode, "emacs");
        assert!(config.syntax_highlighting);
        assert!(!config.is_vi_mode());
    }

    #[test]
    fn test_parse_empty() {
        let config = AshShellConfig::parse("");
        assert_eq!(config.history_size, AshShellConfig::default().history_size);
    }

    #[test]
    fn test_parse_full() {
        let toml = r#"
[shell]
history_size = 5000
autosuggestion = false
autosuggestion_min_chars = 2
edit_mode = "vi"
syntax_highlighting = false

[aliases]
ll = "ls -la"
gs = "git status"

[completion]
case_sensitive = true
"#;
        let config = AshShellConfig::parse(toml);
        assert_eq!(config.history_size, 5000);
        assert!(!config.autosuggestion);
        assert_eq!(config.autosuggestion_min_chars, 2);
        assert!(config.is_vi_mode());
        assert!(!config.syntax_highlighting);
        assert_eq!(config.aliases.get("ll"), Some(&"ls -la".to_string()));
        assert_eq!(config.aliases.get("gs"), Some(&"git status".to_string()));
        assert!(config.completion_case_sensitive);
    }

    #[test]
    fn test_parse_partial() {
        let toml = r#"
[shell]
edit_mode = "vi"

[aliases]
ll = "ls -la"
"#;
        let config = AshShellConfig::parse(toml);
        assert!(config.is_vi_mode());
        assert!(config.autosuggestion); // default preserved
        assert_eq!(config.aliases.len(), 1);
    }

    #[test]
    fn test_invalid_toml_falls_back() {
        let config = AshShellConfig::parse("not valid toml {{{");
        assert_eq!(config.history_size, AshShellConfig::default().history_size);
    }
}
