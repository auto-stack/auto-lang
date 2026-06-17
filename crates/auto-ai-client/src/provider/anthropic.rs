//! Anthropic Claude provider.
//!
//! Uses the Anthropic Messages API (`/v1/messages`) with SSE streaming.
//! Ported from AutoForge's `provider/claude.rs`.

use std::sync::Arc;

use async_trait::async_trait;

use super::AiProvider;
use crate::sse::SseParser;
use crate::types::*;
use crate::ClientError;

pub struct AnthropicProvider {
    name: String,
    base_url: String,
    api_key: String,
    models_list: Vec<String>,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(name: String, base_url: String, api_key: String, models: Vec<String>) -> Self {
        Self {
            name,
            base_url,
            api_key,
            models_list: models,
            client: reqwest::Client::new(),
        }
    }

    fn url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{}/v1/messages", base)
    }

    fn build_body(&self, req: &CompletionRequest) -> serde_json::Value {
        let messages: Vec<serde_json::Value> = req.messages.iter().map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
            })
        }).collect();

        let mut body = serde_json::json!({
            "model": req.model,
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "messages": messages,
        });

        if let Some(sys) = &req.system_prompt {
            body["system"] = serde_json::json!(sys);
        }
        if let Some(t) = req.temperature {
            body["temperature"] = serde_json::json!(t);
        }
        body
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn models(&self) -> Vec<String> {
        self.models_list.clone()
    }

    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        let body = self.build_body(req);

        let resp = self
            .client
            .post(self.url())
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(ClientError::from)?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api(format!("{}: {}", status, text)));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ClientError::Api(format!("parse response: {}", e)))?;

        // Anthropic returns content as an array of blocks.
        let content = json["content"]
            .as_array()
            .map(|blocks| {
                blocks
                    .iter()
                    .filter_map(|b| {
                        if b["type"].as_str() == Some("text") {
                            b["text"].as_str().map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        let usage = json.get("usage").map(|u| Usage {
            input_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
        });

        let model = json["model"]
            .as_str()
            .unwrap_or(&req.model)
            .to_string();

        Ok(CompletionResponse {
            content,
            usage,
            model,
            error: None,
        })
    }

    async fn complete_stream(
        &self,
        req: &CompletionRequest,
        on_delta: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Result<CompletionResponse, ClientError> {
        let mut body = self.build_body(req);
        body["stream"] = serde_json::json!(true);

        let resp = self
            .client
            .post(self.url())
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(ClientError::from)?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api(format!("{}: {}", status, text)));
        }

        use futures::StreamExt;
        let mut stream = resp.bytes_stream();
        let mut parser = SseParser::new();
        let mut content = String::new();

        while let Some(chunk_result) = stream.next().await {
            let bytes = chunk_result.map_err(|e| ClientError::Http(e.to_string()))?;
            let data_events = parser.push(&bytes);

            for data in data_events {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(text) = json["delta"]["text"].as_str().map(|s| s.to_string()) {
                        content.push_str(&text);
                        on_delta(text);
                    }
                }
            }
        }

        // Flush remaining.
        for data in parser.finish() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(text) = json["delta"]["text"].as_str().map(|s| s.to_string()) {
                    content.push_str(&text);
                    on_delta(text);
                }
            }
        }

        Ok(CompletionResponse {
            content,
            usage: None,
            model: req.model.clone(),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_body_anthropic() {
        let p = AnthropicProvider::new(
            "anthropic".into(),
            "https://api.anthropic.com".into(),
            "key".into(),
            vec!["claude-3-5-sonnet-20241022".into()],
        );
        let req = CompletionRequest::single("claude-3-5-sonnet-20241022", "hi");
        let body = p.build_body(&req);
        assert_eq!(body["model"], "claude-3-5-sonnet-20241022");
        assert_eq!(body["max_tokens"], 4096); // default
        assert_eq!(body["messages"][0]["role"], "user");
    }

    #[test]
    fn url_construction() {
        let p = AnthropicProvider::new("a".into(), "https://api.anthropic.com/".into(), "k".into(), vec![]);
        assert_eq!(p.url(), "https://api.anthropic.com/v1/messages");
    }
}
