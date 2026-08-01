//! Hand-written glue: implement the (Auto-transpiled) `Client` trait for the
//! real Rust `AiClient`, plus a `StreamingAiClient` that forwards SSE deltas
//! through a channel for live token display (plan 013 G5).
//!
//! Per plan 013 option A: the HTTP layer stays hand-written Rust (Auto's a2r-std
//! http doesn't match the daemon contract yet — see G6 roadmap). The agent
//! ReAct loop is Auto source. This file is the only non-a2r glue.

use crate::agent::Client;
use crate::auto_ai_client::{AiClient, ClientError, CompletionRequest, CompletionResponse};
use async_trait::async_trait;

/// Plain non-streaming adapter (used when live display isn't needed).
#[async_trait]
impl Client for AiClient {
    async fn complete(
        &self,
        req: CompletionRequest,
    ) -> Result<CompletionResponse, ClientError> {
        AiClient::complete(self, &req).await
    }
}

/// Streaming adapter: wraps `AiClient` and forwards each SSE delta event
/// through a channel. The agent loop still calls `complete()` and gets the full
/// `CompletionResponse` — but tokens flow out in parallel for live display.
///
/// Uses `std::sync::mpsc` (not tokio) to avoid `blocking_send` deadlocks inside
/// the async runtime. `mpsc::Sender` is `Send + 'static`, satisfying
/// `complete_stream`'s `impl Fn(Value) + Send + 'static` closure bound.
pub struct StreamingAiClient {
    inner: AiClient,
    tx: std::sync::mpsc::Sender<serde_json::Value>,
}

impl StreamingAiClient {
    pub fn new(url: &str, tx: std::sync::mpsc::Sender<serde_json::Value>) -> Self {
        Self {
            inner: AiClient::with_url(url),
            tx,
        }
    }
}

#[async_trait]
impl Client for StreamingAiClient {
    async fn complete(
        &self,
        req: CompletionRequest,
    ) -> Result<CompletionResponse, ClientError> {
        let tx = self.tx.clone();
        // Drive complete_stream instead of complete: the daemon sends SSE deltas,
        // each forwarded through the channel. complete_stream returns the fully
        // assembled CompletionResponse (concatenated text + tool_calls/usage).
        AiClient::complete_stream(&self.inner, &req, move |ev| {
            let _ = tx.send(ev);
        })
        .await
    }
}
