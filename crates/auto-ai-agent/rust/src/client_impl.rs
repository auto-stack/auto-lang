//! Hand-written glue: implement the (Auto-transpiled) `Client` trait for the
//! real Rust `AiClient`. This is the bridge between the Auto ReAct loop (which
//! calls `self.client.complete(req).await`) and the HTTP layer (the real
//! auto-ai-client that talks to the aaid daemon).
//!
//! Per plan 013 option A: the HTTP layer stays hand-written Rust (Auto's a2r-std
//! http doesn't match the daemon contract yet — see roadmap option B). The agent
//! ReAct loop is Auto source. This file is the only non-a2r glue.

use crate::agent::Client;
use crate::auto_ai_client::{AiClient, ClientError, CompletionRequest, CompletionResponse};
use async_trait::async_trait;

#[async_trait]
impl Client for AiClient {
    async fn complete(
        &self,
        req: CompletionRequest,
    ) -> Result<CompletionResponse, ClientError> {
        // Delegate to the real client (by reference — AiClient::complete takes
        // &self in the Rust original).
        AiClient::complete(self, &req).await
    }
}
