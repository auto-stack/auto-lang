//! AI Provider integration for AutoLab
//!
//! Calls Claude API (Anthropic) for code generation and explanation.
//! Reads API key and base URL from ~/.claude/settings.json by default,
//! falling back to environment variables.

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-3-5-sonnet-20241022";

/// ~/.claude/settings.json structure (partial)
#[derive(Debug, Deserialize)]
struct ClaudeSettings {
    #[serde(default)]
    env: std::collections::HashMap<String, String>,
}

fn read_claude_settings() -> Option<ClaudeSettings> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .ok()?;
    let path = PathBuf::from(home).join(".claude").join("settings.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn load_api_config() -> (Option<String>, String) {
    // 1. Try ~/.claude/settings.json
    if let Some(settings) = read_claude_settings() {
        let key = settings
            .env
            .get("ANTHROPIC_AUTH_TOKEN")
            .cloned()
            .or_else(|| settings.env.get("ANTHROPIC_API_KEY").cloned());
        let base = settings
            .env
            .get("ANTHROPIC_BASE_URL")
            .cloned()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());
        if key.is_some() {
            return (key, base);
        }
    }

    // 2. Fall back to environment variables
    let key = env::var("ANTHROPIC_API_KEY")
        .or_else(|_| env::var("ANTHROPIC_AUTH_TOKEN"))
        .ok();
    let base = env::var("ANTHROPIC_BASE_URL")
        .unwrap_or_else(|_| "https://api.anthropic.com".to_string());

    (key, base)
}

/// Request from the frontend
#[derive(Debug, Deserialize)]
pub struct AIRequest {
    pub prompt: String,
    pub context: Option<String>,
}

/// Response returned to the frontend (blocking mode)
#[derive(Debug, Serialize)]
pub struct AIResponse {
    pub content: String,
    pub error: Option<String>,
}

/// A single text delta for streaming
#[derive(Debug, Serialize)]
pub struct AIStreamDelta {
    pub text: String,
}

/// Trait for AI providers (allows future OpenAI/Gemini support)
pub trait AiProvider: Send + Sync {
    fn chat(&self, request: AIRequest) -> impl std::future::Future<Output = AIResponse> + Send;
}

/// Anthropic Claude provider
pub struct ClaudeProvider {
    pub(crate) client: reqwest::Client,
    pub(crate) api_key: Option<String>,
    pub(crate) base_url: String,
}

impl ClaudeProvider {
    pub fn new() -> Self {
        let (api_key, base_url) = load_api_config();
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
        }
    }

    pub fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn api_url(&self) -> String {
        format!("{}/v1/messages", self.base_url.trim_end_matches('/'))
    }

    /// Stream chat response as text deltas.
    /// Sends each chunk via `tx` and returns final error (if any).
    pub async fn chat_stream(
        &self,
        request: AIRequest,
        tx: tokio::sync::mpsc::UnboundedSender<AIStreamDelta>,
    ) -> Option<String> {
        let Some(api_key) = &self.api_key else {
            return Some(
                "ANTHROPIC_API_KEY not set. Please configure your API key in ~/.claude/settings.json or environment variables.".to_string()
            );
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
            ],
            "stream": true
        });

        let resp = match self
            .client
            .post(self.api_url())
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return Some(format!("Request failed: {}", e)),
        };

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Some(format!("Claude API error ({}): {}", status, text));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let bytes = match chunk_result {
                Ok(b) => b,
                Err(e) => return Some(format!("Stream error: {}", e)),
            };
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            // Parse SSE events from buffer
            while let Some(pos) = buffer.find("\n\n") {
                let event_text = buffer[..pos].to_string();
                buffer = buffer[pos + 2..].to_string();

                let mut event_type = String::new();
                let mut data_line = String::new();
                for line in event_text.lines() {
                    if line.starts_with("event: ") {
                        event_type = line["event: ".len()..].to_string();
                    } else if line.starts_with("data: ") {
                        data_line = line["data: ".len()..].to_string();
                    }
                }

                if event_type == "content_block_delta" {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data_line) {
                        if let Some(text) = json
                            .get("delta")
                            .and_then(|d| d.get("text"))
                            .and_then(|t| t.as_str())
                        {
                            let _ = tx.send(AIStreamDelta { text: text.to_string() });
                        }
                    }
                }
            }
        }

        None
    }
}

impl AiProvider for ClaudeProvider {
    async fn chat(&self, request: AIRequest) -> AIResponse {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<AIStreamDelta>();
        let error = self.chat_stream(request, tx).await;

        let mut content = String::new();
        while let Some(delta) = rx.recv().await {
            content.push_str(&delta.text);
        }

        if let Some(err) = error {
            AIResponse {
                content,
                error: Some(err),
            }
        } else {
            AIResponse { content, error: None }
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
