//! Daemon configuration.
//!
//! Loaded from `~/.config/autoos/ai-daemon.at` (Auto/Atom format).

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub listen_addr: String,
    pub idle_timeout_min: u64,
    pub providers: HashMap<String, ProviderEntry>,
    pub default_provider: String,
    pub default_model: String,
    pub log_level: String,
}

#[derive(Debug, Clone)]
pub struct ProviderEntry {
    pub kind: String,
    pub base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub max_concurrency: usize,
}

impl DaemonConfig {
    /// Load from `~/.config/autoos/ai-daemon.at` or env fallback.
    pub fn load() -> Self {
        if let Some(cfg) = Self::load_from_file() {
            return cfg;
        }
        Self::load_from_env()
    }

    fn load_from_file() -> Option<Self> {
        let path = dirs::home_dir()?.join(".config/autoos/ai-daemon.at");
        let content = std::fs::read_to_string(&path).ok()?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Option<Self> {
        let mut listen_addr = "127.0.0.1:17654".to_string();
        let mut idle_timeout_min = 10;
        let mut default_provider = String::new();
        let mut default_model = String::new();
        let mut log_level = "info".to_string();
        let mut providers = HashMap::new();
        let mut current: Option<String> = None;
        let mut fields: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with("//") { continue; }
            if t == "}" {
                if let Some(name) = current.take() {
                    let kind = fields.remove("kind").unwrap_or_default();
                    let base_url = fields.remove("base_url").unwrap_or_default();
                    let api_key = fields.remove("api_key")
                        .or_else(|| fields.remove("key_env").and_then(|e| std::env::var(&e).ok()))
                        .unwrap_or_default();
                    let models = fields.remove("models")
                        .map(|s| s.split(',').map(|m| m.trim().to_string()).collect())
                        .unwrap_or_default();
                    let max_concurrency = fields.remove("max_concurrency")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(4);
                    if !kind.is_empty() {
                        providers.insert(name.clone(), ProviderEntry {
                            kind, base_url, api_key, models, max_concurrency,
                        });
                        if default_provider.is_empty() {
                            default_provider = name;
                        }
                    }
                    fields.clear();
                }
                continue;
            }
            if t.contains('{') {
                let name = t.split('{').next().unwrap().trim().to_string();
                if !name.is_empty() { current = Some(name); continue; }
            }
            if let Some((k, v)) = t.split_once(':') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').to_string();
                if current.is_some() {
                    fields.insert(key.to_string(), val);
                } else {
                    match key {
                        "listen_addr" => listen_addr = val,
                        "idle_timeout_min" => idle_timeout_min = val.parse().unwrap_or(10),
                        "default_provider" => default_provider = val,
                        "default_model" => default_model = val,
                        "log_level" => log_level = val,
                        _ => {}
                    }
                }
            }
        }

        if providers.is_empty() { return None; }
        if default_model.is_empty() {
            default_model = providers.get(&default_provider)
                .and_then(|p| p.models.first().cloned())
                .unwrap_or_default();
        }
        Some(Self { listen_addr, idle_timeout_min, providers, default_provider, default_model, log_level })
    }

    fn load_from_env() -> Self {
        let mut providers = HashMap::new();

        // Zhipu (OpenAI-compatible).
        if let Ok(key) = std::env::var("ZHIPU_API_KEY") {
            providers.insert("zhipu".into(), ProviderEntry {
                kind: "openai".into(),
                base_url: "https://open.bigmodel.cn/api/paas/v4".into(),
                api_key: key,
                models: vec!["glm-4.5".into(), "glm-4-flash".into()],
                max_concurrency: 4,
            });
        }
        // Anthropic.
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY").or_else(|_| std::env::var("ANTHROPIC_AUTH_TOKEN")) {
            providers.insert("anthropic".into(), ProviderEntry {
                kind: "anthropic".into(),
                base_url: std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| "https://api.anthropic.com".into()),
                api_key: key,
                models: vec!["claude-3-5-sonnet-20241022".into()],
                max_concurrency: 4,
            });
        }
        // OpenAI.
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            providers.insert("openai".into(), ProviderEntry {
                kind: "openai".into(),
                base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into()),
                api_key: key,
                models: vec!["gpt-4o".into()],
                max_concurrency: 4,
            });
        }

        let default_provider = providers.keys().next().cloned().unwrap_or_default();
        let default_model = providers.get(&default_provider)
            .and_then(|p| p.models.first().cloned()).unwrap_or_default();

        Self {
            listen_addr: "127.0.0.1:17654".into(),
            idle_timeout_min: 10,
            providers,
            default_provider,
            default_model,
            log_level: "info".into(),
        }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self::load_from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_file() {
        let content = r#"
listen_addr : "127.0.0.1:9999"
default_provider : zhipu
default_model : glm-4.5

zhipu {
    kind : openai
    base_url : "https://open.bigmodel.cn/api/paas/v4"
    api_key : "test-key"
    models : glm-4.5,glm-4-flash
    max_concurrency : 4
}
"#;
        let cfg = DaemonConfig::parse(content).unwrap();
        assert_eq!(cfg.listen_addr, "127.0.0.1:9999");
        assert_eq!(cfg.default_provider, "zhipu");
        assert_eq!(cfg.default_model, "glm-4.5");
        let zhipu = cfg.providers.get("zhipu").unwrap();
        assert_eq!(zhipu.api_key, "test-key");
        assert_eq!(zhipu.max_concurrency, 4);
        assert_eq!(zhipu.models, vec!["glm-4.5", "glm-4-flash"]);
    }

    #[test]
    fn empty_returns_none() {
        assert!(DaemonConfig::parse("").is_none());
    }
}
