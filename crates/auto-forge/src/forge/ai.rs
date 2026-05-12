//! AutoSmith AI Provider with Tool Support
//!
//! Extends the base ClaudeProvider with native tool-use support
//! for the Forge ReAct loop.

use crate::ai::{AIProviderState, AIStreamDelta, ClaudeProvider};
use crate::forge::tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const CLAUDE_MODEL: &str = "claude-3-5-sonnet-20241022";

/// A request to the AI that may include tool definitions
#[derive(Debug, Clone)]
pub struct ToolChatRequest {
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolDefinition>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

impl ChatMessage {
    pub fn user(text: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![ContentBlock::text(text)],
        }
    }

    pub fn assistant_text(text: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: vec![ContentBlock::text(text)],
        }
    }

    pub fn tool_result(tool_use_id: &str, result: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![ContentBlock::tool_result(tool_use_id, result)],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

impl ContentBlock {
    pub fn text(s: &str) -> Self {
        ContentBlock::Text {
            text: s.to_string(),
        }
    }

    pub fn tool_result(tool_use_id: &str, result: &str) -> Self {
        ContentBlock::ToolResult {
            tool_use_id: tool_use_id.to_string(),
            content: result.to_string(),
        }
    }
}

/// Events emitted during a tool-enabled chat stream
#[derive(Debug, Clone)]
pub enum ToolChatEvent {
    /// A text delta from the AI
    TextDelta { text: String },
    /// The AI wants to use a tool
    ToolUse { id: String, name: String, input: Value },
    /// The stream completed (no more events)
    Done,
    /// An error occurred
    Error { message: String },
}

/// Extended Claude provider with tool support
pub struct ToolClaudeProvider {
    inner: AIProviderState,
}

impl ToolClaudeProvider {
    pub fn new(inner: AIProviderState) -> Self {
        Self { inner }
    }

    /// Run a single turn of tool-enabled chat.
    /// Returns events (text deltas, tool_use requests) via the channel.
    /// If a tool_use is emitted, the caller must execute the tool and call again with the result.
    pub async fn chat_turn(
        &self,
        request: ToolChatRequest,
        tx: tokio::sync::mpsc::UnboundedSender<ToolChatEvent>,
    ) -> Option<String> {
        let provider = self.inner.as_ref();
        let Some(api_key) = &provider.api_key else {
            return Some(
                "ANTHROPIC_API_KEY not set. Please configure your API key in ~/.claude/settings.json or environment variables.".to_string()
            );
        };

        let system = request
            .system_prompt
            .unwrap_or_else(|| build_forge_system_prompt());

        let body = serde_json::json!({
            "model": CLAUDE_MODEL,
            "max_tokens": 4096,
            "system": system,
            "messages": request.messages,
            "tools": request.tools,
            "stream": true
        });

        let api_url = format!("{}/v1/messages", provider.base_url.trim_end_matches('/'));
        let resp = match provider
            .client
            .post(api_url)
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

        use futures::StreamExt;
        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut current_tool_use: Option<(String, String)> = None; // (id, name)
        let mut partial_json_acc = String::new();

        while let Some(chunk_result) = stream.next().await {
            let bytes = match chunk_result {
                Ok(b) => b,
                Err(e) => return Some(format!("Stream error: {}", e)),
            };
            buffer.push_str(&String::from_utf8_lossy(&bytes));

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

                if event_type == "content_block_start" {
                    if let Ok(json) = serde_json::from_str::<Value>(&data_line) {
                        if let Some(block) = json.get("content_block") {
                            if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                                let id = block
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let name = block
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                // Reset accumulator for new tool_use block
                                partial_json_acc.clear();
                                current_tool_use = Some((id, name));
                            }
                        }
                    }
                } else if event_type == "content_block_delta" {
                    if let Ok(json) = serde_json::from_str::<Value>(&data_line) {
                        // Text delta
                        if let Some(text) = json
                            .get("delta")
                            .and_then(|d| d.get("text"))
                            .and_then(|t| t.as_str())
                        {
                            let _ = tx.send(ToolChatEvent::TextDelta {
                                text: text.to_string(),
                            });
                        }
                        // Tool input JSON delta (accumulated)
                        else if let Some(partial_json) = json
                            .get("delta")
                            .and_then(|d| d.get("partial_json"))
                            .and_then(|p| p.as_str())
                        {
                            partial_json_acc.push_str(partial_json);
                        }
                    }
                } else if event_type == "content_block_stop" {
                    if let Some((id, name)) = current_tool_use.take() {
                        // Parse accumulated JSON, fallback to empty object
                        let input = if partial_json_acc.is_empty() {
                            Value::Object(Default::default())
                        } else {
                            serde_json::from_str(&partial_json_acc)
                                .unwrap_or_else(|_| Value::Object(Default::default()))
                        };
                        partial_json_acc.clear();
                        let _ = tx.send(ToolChatEvent::ToolUse { id, name, input });
                    }
                }
            }
        }

        let _ = tx.send(ToolChatEvent::Done);
        None
    }
}

fn build_forge_system_prompt() -> String {
    r#"You are AutoForge, an expert AI coding assistant.

Your workflow:
1. Understand the user's request
2. Use tools to explore the codebase when needed
3. Propose specs or generate code
4. Explain your reasoning clearly

When you need to examine files, search for patterns, or run commands, use the available tools.
When you want to modify code, use the edit_file or write_file tools.

Language policy:
- If the user explicitly asks for a specific language (e.g., Python, JavaScript, Rust), generate code in that language.
- If the user asks about or for the Auto language, use Auto syntax.
- If no language is specified and the context is this Auto-lang project, default to Auto syntax.
- Always respect the user's explicitly requested language.

Auto language syntax rules (for when Auto is requested):
- Functions: `fn name(args) ret_type { body }`
- Variables: `var x = expr` or `let x = expr` (immutable)
- Types: `int`, `float`, `string`, `bool`, `list<T>`, `map<K,V>`
- String interpolation: `f"Hello, ${name}"`
- Pipes: `data |> filter(x -> x > 0) |> map(x -> x * 2)`
- Pattern matching: `match expr { A => ..., B => ... }`
- No semicolons needed; expression blocks return last value

When generating code:
1. Use the correct syntax for the requested language
2. Provide brief explanation before the code block
3. Wrap code in markdown fenced code blocks with the correct language tag (e.g., `python`, `javascript`, `auto`)
4. Keep examples concise and runnable
"#
    .to_string()
}
