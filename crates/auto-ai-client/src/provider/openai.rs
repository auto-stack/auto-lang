//! OpenAI-compatible provider (works with OpenAI, Zhipu GLM, Moonshot, etc.).
//!
//! Uses the standard OpenAI `/v1/chat/completions` API format with SSE streaming.

use std::sync::Arc;

use async_trait::async_trait;

use super::AiProvider;
use crate::sse::SseParser;
use crate::types::*;
use crate::ClientError;

pub struct OpenAiProvider {
    name: String,
    base_url: String,
    api_key: String,
    models_list: Vec<String>,
    client: reqwest::Client,
}

impl OpenAiProvider {
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
        format!("{}/chat/completions", base)
    }

    fn build_body(&self, req: &CompletionRequest) -> serde_json::Value {
        let messages: Vec<serde_json::Value> = req.messages.iter().map(|m| {
            serde_json::json!({ "role": m.role, "content": m.content })
        }).collect();

        let mut body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "stream": false,
        });

        if let Some(sys) = &req.system_prompt {
            // Prepend system message.
            let mut all_msgs = vec![serde_json::json!({ "role": "system", "content": sys })];
            all_msgs.extend(messages);
            body["messages"] = serde_json::Value::Array(all_msgs);
        }
        if let Some(n) = req.max_tokens {
            body["max_tokens"] = serde_json::json!(n);
        }
        if let Some(t) = req.temperature {
            body["temperature"] = serde_json::json!(t);
        }
        body
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
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
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json.get("usage").map(|u| {
            Usage {
                input_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                output_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            }
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
            .header("Authorization", format!("Bearer {}", self.api_key))
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
                    if let Some(delta) = json["choices"][0]["delta"]["content"].as_str().map(|s| s.to_string()) {
                        content.push_str(&delta);
                        on_delta(delta);
                    }
                }
            }
        }

        // Flush remaining.
        for data in parser.finish() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(delta) = json["choices"][0]["delta"]["content"].as_str().map(|s| s.to_string()) {
                    content.push_str(&delta);
                    on_delta(delta);
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
    fn build_body_basic() {
        let p = OpenAiProvider::new("test".into(), "https://api.test.com/v1".into(), "key".into(), vec![]);
        let req = CompletionRequest::single("gpt-4o", "hello");
        let body = p.build_body(&req);
        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "hello");
    }

    #[test]
    fn build_body_with_system() {
        let p = OpenAiProvider::new("test".into(), "https://api.test.com/v1".into(), "key".into(), vec![]);
        let req = CompletionRequest::single("gpt-4o", "hello").with_system("be nice");
        let body = p.build_body(&req);
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "be nice");
        assert_eq!(body["messages"][1]["role"], "user");
    }

    #[test]
    fn url_construction() {
        let p = OpenAiProvider::new("z".into(), "https://api.test.com/v1/".into(), "k".into(), vec![]);
        assert_eq!(p.url(), "https://api.test.com/v1/chat/completions");
    }
}
