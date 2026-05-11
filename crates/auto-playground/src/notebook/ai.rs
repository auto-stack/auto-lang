//! AI Provider integration for AutoLab
//!
//! Calls Claude API (Anthropic) for code generation and explanation.
//! Falls back gracefully if no API key is configured.

use serde::{Deserialize, Serialize};
use std::env;

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-3-5-sonnet-20241022";

/// Request from the frontend
#[derive(Debug, Deserialize)]
pub struct AIRequest {
    pub prompt: String,
    pub context: Option<String>,
}

/// Response returned to the frontend
#[derive(Debug, Serialize)]
pub struct AIResponse {
    pub content: String,
    pub error: Option<String>,
}

/// Trait for AI providers (allows future OpenAI/Gemini support)
pub trait AiProvider: Send + Sync {
    fn chat(&self, request: AIRequest) -> impl std::future::Future<Output = AIResponse> + Send;
}

/// Anthropic Claude provider
pub struct ClaudeProvider {
    client: reqwest::Client,
    api_key: Option<String>,
}

impl ClaudeProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: env::var("ANTHROPIC_API_KEY").ok(),
        }
    }

    pub fn is_available(&self) -> bool {
        self.api_key.is_some()
    }
}

impl AiProvider for ClaudeProvider {
    async fn chat(&self, request: AIRequest) -> AIResponse {
        let Some(api_key) = &self.api_key else {
            return AIResponse {
                content: String::new(),
                error: Some(
                    "ANTHROPIC_API_KEY not set. Please configure your API key.".to_string(),
                ),
            };
        };

        let system_prompt = build_system_prompt();
        let user_prompt = if let Some(ctx) = request.context {
            format!("Notebook context:\n{}\n\nUser request:\n{}", ctx, request.prompt)
        } else {
            request.prompt
        };

        let body = serde_json::json!({
            "model": CLAUDE_MODEL,
            "max_tokens": 4096,
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ]
        });

        let result = self
            .client
            .post(CLAUDE_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status = resp.status();
                let text = match resp.text().await {
                    Ok(t) => t,
                    Err(e) => {
                        return AIResponse {
                            content: String::new(),
                            error: Some(format!("Failed to read response: {}", e)),
                        }
                    }
                };

                if !status.is_success() {
                    return AIResponse {
                        content: String::new(),
                        error: Some(format!("Claude API error ({}): {}", status, text)),
                    };
                }

                match extract_content(&text) {
                    Ok(content) => AIResponse { content, error: None },
                    Err(e) => AIResponse {
                        content: String::new(),
                        error: Some(format!("Failed to parse response: {}", e)),
                    },
                }
            }
            Err(e) => AIResponse {
                content: String::new(),
                error: Some(format!("Request failed: {}", e)),
            },
        }
    }
}

fn build_system_prompt() -> String {
    r#"You are an expert assistant for the Auto programming language.

Auto language syntax rules:
- Functions: `fn name(args) ret_type { body }`
- Variables: `var x = expr` or `let x = expr` (immutable)
- Types: `int`, `float`, `string`, `bool`, `list<T>`, `map<K,V>`
- String interpolation: `f"Hello, ${name}"`
- Pipes: `data |> filter(x -> x > 0) |> map(x -> x * 2)`
- Pattern matching: `match expr { A => ..., B => ... }`
- No semicolons needed; expression blocks return last value

When generating code:
1. Use correct Auto syntax
2. Provide brief explanation before the code block
3. Wrap code in markdown fenced code blocks with `auto` language tag
4. Keep examples concise and runnable
"#
    .to_string()
}

fn extract_content(json_str: &str) -> Result<String, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| e.to_string())?;

    let content = parsed
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| "Missing content in response".to_string())?;

    Ok(content.to_string())
}

/// Shared AI provider handle
pub type AIProviderState = std::sync::Arc<ClaudeProvider>;
