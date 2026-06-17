//! Provider trait + registry + concrete provider implementations.
//!
//! Extracted and generalized from AutoForge's `provider/claude.rs`.

pub mod openai;
pub mod anthropic;

pub use anthropic::AnthropicProvider;
pub use openai::OpenAiProvider;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::config::ClientConfig;
use crate::types::*;
use crate::ClientError;

/// Trait that every LLM provider implements.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Provider name (e.g. "zhipu", "anthropic").
    fn name(&self) -> &str;

    /// Available models.
    fn models(&self) -> Vec<String>;

    /// Non-streaming completion.
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ClientError>;

    /// Streaming completion. Calls `on_delta` for each text chunk.
    async fn complete_stream(
        &self,
        req: &CompletionRequest,
        on_delta: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Result<CompletionResponse, ClientError>;
}

/// Registry of configured providers.
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
    default_name: String,
}

impl ProviderRegistry {
    pub fn from_config(config: &ClientConfig) -> Result<Self, ClientError> {
        let mut providers: HashMap<String, Arc<dyn AiProvider>> = HashMap::new();

        for (name, pc) in &config.providers {
            let key = pc.resolve_key().ok_or_else(|| ClientError::NoApiKey(name.clone()))?;
            let provider: Arc<dyn AiProvider> = match pc.kind.as_str() {
                "anthropic" => Arc::new(AnthropicProvider::new(
                    name.clone(),
                    pc.base_url.clone(),
                    key,
                    pc.models.clone(),
                )),
                "openai" | _ => Arc::new(OpenAiProvider::new(
                    name.clone(),
                    pc.base_url.clone(),
                    key,
                    pc.models.clone(),
                )),
            };
            providers.insert(name.clone(), provider);
        }

        if providers.is_empty() {
            return Err(ClientError::NoProvider);
        }

        Ok(Self {
            providers,
            default_name: config.default_provider.clone(),
        })
    }

    pub fn default_provider(&self) -> Result<&Arc<dyn AiProvider>, ClientError> {
        self.providers.get(&self.default_name).ok_or(ClientError::NoProvider)
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn AiProvider>> {
        self.providers.get(name)
    }

    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    pub fn models_for(&self, provider: &str) -> Vec<String> {
        self.providers.get(provider).map(|p| p.models()).unwrap_or_default()
    }
}
