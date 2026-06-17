//! Configuration for auto-ai-client.
//!
//! Loaded from `~/.config/autoos/ai-client.at` (Auto/Atom format) or
//! environment variables. Falls back to sensible defaults.

use std::collections::HashMap;

/// Client configuration.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Configured providers (name → settings).
    pub providers: HashMap<String, ProviderConfig>,
    /// Default provider name.
    pub default_provider: String,
    /// Default model for the default provider.
    pub default_model: String,
}

/// Per-provider configuration.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Provider type: "anthropic" | "openai" | "zhipu".
    pub kind: String,
    /// API base URL.
    pub base_url: String,
    /// API key (direct string).
    pub api_key: Option<String>,
    /// Environment variable name for the API key (alternative to api_key).
    pub key_env: Option<String>,
    /// Available models.
    pub models: Vec<String>,
}

impl ProviderConfig {
    /// Resolve the API key: direct string > env var.
    pub fn resolve_key(&self) -> Option<String> {
        if let Some(key) = &self.api_key {
            return Some(key.clone());
        }
        if let Some(env_name) = &self.key_env {
            return std::env::var(env_name).ok();
        }
        None
    }
}

impl ClientConfig {
    /// Load configuration from file + environment.
    pub fn load() -> Self {
        // Try ~/.config/autoos/ai-client.at first.
        if let Some(config) = Self::load_from_file() {
            return config;
        }
        // Fall back to environment variables (Forge-compatible).
        Self::load_from_env()
    }

    fn load_from_file() -> Option<Self> {
        let paths = Self::config_paths();
        for path in &paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    return Self::parse_config(&content);
                }
            }
        }
        None
    }

    fn config_paths() -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".config/autoos/ai-client.at"));
        }
        if let Some(cfg) = dirs::config_dir() {
            paths.push(cfg.join("autoos/ai-client.at"));
        }
        paths
    }

    fn parse_config(content: &str) -> Option<Self> {
        // Parse Auto/Atom format using a simple scanner (same as auto_config.rs).
        // Expected:
        //   default_provider : zhipu
        //   default_model : glm-4.5
        //   zhipu { kind : openai, base_url : "...", key_env : ZHIPU_API_KEY, models : glm-4.5,glm-flash }
        let mut providers = HashMap::new();
        let mut default_provider = String::new();
        let mut default_model = String::new();

        let mut current_block: Option<String> = None;
        let mut block_fields: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            if trimmed == "}" {
                if let Some(name) = current_block.take() {
                    if let Some(kind) = block_fields.remove("kind") {
                        providers.insert(name.clone(), ProviderConfig {
                            kind,
                            base_url: block_fields.remove("base_url").unwrap_or_default(),
                            api_key: block_fields.remove("api_key"),
                            key_env: block_fields.remove("key_env"),
                            models: block_fields.remove("models")
                                .map(|s| s.split(',').map(|m| m.trim().to_string()).collect())
                                .unwrap_or_default(),
                        });
                    }
                    block_fields.clear();
                }
                continue;
            }
            // Check for block entry: "name {"
            if let Some(pos) = trimmed.find('{') {
                let name = trimmed[..pos].trim().to_string();
                if !name.is_empty() {
                    current_block = Some(name);
                    continue;
                }
            }
            // Key-value: "key : value"
            if let Some((k, v)) = trimmed.split_once(':') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').to_string();
                if let Some(ref _block) = current_block {
                    block_fields.insert(key.to_string(), val);
                } else {
                    match key {
                        "default_provider" => default_provider = val,
                        "default_model" => default_model = val,
                        _ => {}
                    }
                }
            }
        }

        if providers.is_empty() {
            return None;
        }
        if default_provider.is_empty() {
            default_provider = providers.keys().next().cloned().unwrap_or_default();
        }

        Some(Self { providers, default_provider, default_model })
    }

    fn load_from_env() -> Self {
        let mut providers = HashMap::new();

        // AutoForge-compatible: ANTHROPIC_API_KEY / ANTHROPIC_BASE_URL.
        if std::env::var("ANTHROPIC_API_KEY").is_ok() || std::env::var("ANTHROPIC_AUTH_TOKEN").is_ok() {
            providers.insert("anthropic".into(), ProviderConfig {
                kind: "anthropic".into(),
                base_url: std::env::var("ANTHROPIC_BASE_URL")
                    .unwrap_or_else(|_| "https://api.anthropic.com".into()),
                api_key: None,
                key_env: Some("ANTHROPIC_API_KEY".into()),
                models: vec!["claude-3-5-sonnet-20241022".into()],
            });
        }

        // OpenAI-compatible: OPENAI_API_KEY.
        if std::env::var("OPENAI_API_KEY").is_ok() {
            providers.insert("openai".into(), ProviderConfig {
                kind: "openai".into(),
                base_url: std::env::var("OPENAI_BASE_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
                api_key: None,
                key_env: Some("OPENAI_API_KEY".into()),
                models: vec!["gpt-4o".into()],
            });
        }

        // Zhipu: ZHIPU_API_KEY.
        if std::env::var("ZHIPU_API_KEY").is_ok() {
            providers.insert("zhipu".into(), ProviderConfig {
                kind: "openai".into(), // Zhipu uses OpenAI-compatible API.
                base_url: "https://open.bigmodel.cn/api/paas/v4".into(),
                api_key: None,
                key_env: Some("ZHIPU_API_KEY".into()),
                models: vec!["glm-4.5".into(), "glm-4-flash".into()],
            });
        }

        let default_provider = providers.keys().next().cloned().unwrap_or_default();
        let default_model = providers.get(&default_provider)
            .map(|p| p.models.first().cloned().unwrap_or_default())
            .unwrap_or_default();

        Self { providers, default_provider, default_model }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: String::new(),
            default_model: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_file() {
        let content = r#"
default_provider : zhipu
default_model : glm-4.5

zhipu {
    kind : openai
    base_url : "https://open.bigmodel.cn/api/paas/v4"
    key_env : ZHIPU_API_KEY
    models : glm-4.5,glm-4-flash
}
"#;
        let config = ClientConfig::parse_config(content).unwrap();
        assert_eq!(config.default_provider, "zhipu");
        assert_eq!(config.default_model, "glm-4.5");
        let zhipu = config.providers.get("zhipu").unwrap();
        assert_eq!(zhipu.kind, "openai");
        assert_eq!(zhipu.models, vec!["glm-4.5", "glm-4-flash"]);
        assert_eq!(zhipu.key_env.as_deref(), Some("ZHIPU_API_KEY"));
    }

    #[test]
    fn resolve_key_from_direct() {
        let pc = ProviderConfig {
            kind: "openai".into(),
            base_url: String::new(),
            api_key: Some("sk-xxx".into()),
            key_env: None,
            models: vec![],
        };
        assert_eq!(pc.resolve_key(), Some("sk-xxx".into()));
    }
}
