//! Unified types for AI completion requests/responses.
//!
//! Extracted and simplified from AutoForge's `provider/types.rs`.
//! These are provider-agnostic — each provider translates to/from its own format.

use serde::{Deserialize, Serialize};

/// A single message in a conversation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,       // "user" | "assistant" | "system"
    pub content: String,
}

impl Message {
    pub fn user(text: impl Into<String>) -> Self {
        Self { role: "user".into(), content: text.into() }
    }
    pub fn assistant(text: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: text.into() }
    }
    pub fn system(text: impl Into<String>) -> Self {
        Self { role: "system".into(), content: text.into() }
    }
}

/// A completion request (provider-agnostic).
#[derive(Clone, Debug)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
    pub system_prompt: Option<String>,
}

impl CompletionRequest {
    /// Simple single-turn request: one user message.
    pub fn single(model: &str, prompt: &str) -> Self {
        Self {
            model: model.to_string(),
            messages: vec![Message::user(prompt)],
            max_tokens: None,
            temperature: None,
            system_prompt: None,
        }
    }

    /// With a system prompt.
    pub fn with_system(mut self, system: &str) -> Self {
        self.system_prompt = Some(system.to_string());
        self
    }

    /// With max output tokens.
    pub fn with_max_tokens(mut self, n: usize) -> Self {
        self.max_tokens = Some(n);
        self
    }

    /// With temperature.
    pub fn with_temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }
}

/// A completion response.
#[derive(Clone, Debug)]
pub struct CompletionResponse {
    /// The full text response (all chunks joined for streaming).
    pub content: String,
    /// Token usage (if reported by the API).
    pub usage: Option<Usage>,
    /// Model that produced the response.
    pub model: String,
    /// Error message (if any). Content may still be partial.
    pub error: Option<String>,
}

impl CompletionResponse {
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }
}

/// Token usage statistics.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl Usage {
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_constructors() {
        assert_eq!(Message::user("hi").role, "user");
        assert_eq!(Message::assistant("hello").role, "assistant");
        assert_eq!(Message::system("be nice").role, "system");
    }

    #[test]
    fn completion_request_builder() {
        let req = CompletionRequest::single("glm-4.5", "hello")
            .with_system("you are helpful")
            .with_max_tokens(100)
            .with_temperature(0.7);
        assert_eq!(req.model, "glm-4.5");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.system_prompt.as_deref(), Some("you are helpful"));
        assert_eq!(req.max_tokens, Some(100));
        assert_eq!(req.temperature, Some(0.7));
    }

    #[test]
    fn usage_total() {
        let u = Usage { input_tokens: 100, output_tokens: 50 };
        assert_eq!(u.total_tokens(), 150);
    }
}
