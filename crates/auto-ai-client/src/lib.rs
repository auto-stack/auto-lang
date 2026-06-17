//! AutoOS shared AI client (Plan 325 Phase 1).
//!
//! Extracted from AutoForge's provider layer. Provides a unified `AiClient` API
//! for all AutoOS apps to call LLM services. Currently supports:
//! - Anthropic Claude (ported from Forge)
//! - OpenAI-compatible APIs (Zhipu GLM, OpenAI, etc.)
//!
//! Each app links this crate and calls `AiClient::complete()` — the crate handles
//! HTTP requests, SSE parsing, error handling, and (in daemon mode) concurrency
//! routing through the `aaid` system daemon.

pub mod config;
pub mod provider;
pub mod sse;
pub mod types;

pub use provider::{AiProvider, ProviderRegistry};
pub use types::*;

use std::sync::Arc;

/// The main client. Apps create one of these and call `complete()`.
///
/// In direct mode: calls the LLM API directly.
/// In daemon mode (future): routes through `aaid` for shared concurrency.
pub struct AiClient {
    registry: ProviderRegistry,
}

impl AiClient {
    /// Create a new client with configuration loaded from
    /// `~/.config/autoos/ai-client.at` (or environment variables).
    pub fn new() -> Result<Self, ClientError> {
        let config = config::ClientConfig::load();
        let registry = ProviderRegistry::from_config(&config)?;
        Ok(Self { registry })
    }

    /// Create a client with an explicit config (for testing).
    pub fn with_config(config: config::ClientConfig) -> Result<Self, ClientError> {
        let registry = ProviderRegistry::from_config(&config)?;
        Ok(Self { registry })
    }

    /// Send a completion request (non-streaming). Returns the full response.
    pub async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        let provider = self.registry.default_provider()?;
        provider.complete(req).await
    }

    /// Send a streaming completion. Returns text chunks via the callback.
    pub async fn complete_stream(
        &self,
        req: &CompletionRequest,
        on_delta: impl Fn(String) + Send + Sync + 'static,
    ) -> Result<CompletionResponse, ClientError> {
        let provider = self.registry.default_provider()?;
        let cb = std::sync::Arc::new(on_delta);
        provider.complete_stream(req, cb).await
    }

    /// List available providers.
    pub fn providers(&self) -> Vec<&str> {
        self.registry.provider_names()
    }

    /// List available models for a provider.
    pub fn models(&self, provider: &str) -> Vec<String> {
        self.registry.models_for(provider)
    }
}

/// Unified error type.
#[derive(Debug)]
pub enum ClientError {
    /// No API key configured.
    NoApiKey(String),
    /// No provider configured.
    NoProvider,
    /// HTTP request failed.
    Http(String),
    /// SSE parse error.
    Sse(String),
    /// API returned an error response.
    Api(String),
    /// Config file error.
    Config(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoApiKey(p) => write!(f, "no API key for provider '{}'", p),
            Self::NoProvider => write!(f, "no provider configured"),
            Self::Http(e) => write!(f, "HTTP error: {}", e),
            Self::Sse(e) => write!(f, "SSE parse error: {}", e),
            Self::Api(e) => write!(f, "API error: {}", e),
            Self::Config(e) => write!(f, "config error: {}", e),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_error_display() {
        assert!(format!("{}", ClientError::NoProvider).contains("no provider"));
        assert!(format!("{}", ClientError::NoApiKey("zhipu".into())).contains("zhipu"));
    }
}
