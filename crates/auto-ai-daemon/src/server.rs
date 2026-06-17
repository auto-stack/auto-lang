//! HTTP server (axum) — the daemon's public API.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use serde_json::json;

use crate::config::DaemonConfig;
use crate::pool::ConcurrencyManager;
use crate::tracker::UsageTracker;

pub struct AppState {
    pub config: DaemonConfig,
    pub pool: ConcurrencyManager,
    pub tracker: UsageTracker,
    pub current_model: std::sync::Mutex<String>,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(config: DaemonConfig) -> Self {
        let pool = ConcurrencyManager::from_config(&config);
        let current_model = config.default_model.clone();
        Self {
            config,
            pool,
            tracker: UsageTracker::new(),
            current_model: std::sync::Mutex::new(current_model),
            http_client: reqwest::Client::new(),
        }
    }
}

pub fn router(state: Arc<AppState>) -> axum::Router {
    axum::Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/status", get(status))
        .route("/v1/models", get(models))
        .route("/v1/usage", get(usage))
        .with_state(state)
}

async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let app_name = headers
        .get("x-app-name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let provider_name = &state.config.default_provider;
    let is_stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);

    // Acquire concurrency permit.
    let _permit = match state.pool.acquire(provider_name).await {
        Some(p) => p,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": {"message": "concurrency pool unavailable"}})),
            )
                .into_response();
        }
    };

    let provider = match state.config.providers.get(provider_name) {
        Some(p) => p,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": {"message": "provider not configured"}})),
            )
                .into_response();
        }
    };

    let url = match provider.kind.as_str() {
        "anthropic" => format!("{}/v1/messages", provider.base_url.trim_end_matches('/')),
        _ => format!("{}/chat/completions", provider.base_url.trim_end_matches('/')),
    };

    let mut req = state
        .http_client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body);

    match provider.kind.as_str() {
        "anthropic" => {
            req = req
                .header("x-api-key", &provider.api_key)
                .header("anthropic-version", "2023-06-01");
        }
        _ => {
            req = req.header("Authorization", format!("Bearer {}", provider.api_key));
        }
    }

    let upstream = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("upstream request failed: {}", e);
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": {"message": format!("upstream error: {e}")}})),
            )
                .into_response();
        }
    };

    let status = upstream.status();

    // Non-streaming: pass through the response body.
    if !is_stream {
        let resp_body = upstream.bytes().await.unwrap_or_default();
        // Extract usage for tracking (best-effort).
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&resp_body) {
            let input = json
                .pointer("/usage/prompt_tokens")
                .or_else(|| json.pointer("/usage/input_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let output = json
                .pointer("/usage/completion_tokens")
                .or_else(|| json.pointer("/usage/output_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if input > 0 || output > 0 {
                state.tracker.record(&app_name, input, output);
            }
        }
        return (status, resp_body).into_response();
    }

    // Streaming: read full response and forward as text/event-stream.
    // (Simple proxy; future: true streaming passthrough.)
    let resp_body = upstream.bytes().await.unwrap_or_default();
    (
        StatusCode::OK,
        [("Content-Type", "text/event-stream")],
        resp_body,
    )
        .into_response()
}

async fn status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pools: Vec<serde_json::Value> = state
        .pool
        .status()
        .iter()
        .map(|(name, available, max)| {
            json!({
                "provider": name,
                "available_permits": available,
                "max_concurrency": max,
                "in_use": max - available,
            })
        })
        .collect();

    let current_model = state.current_model.lock().unwrap().clone();

    Json(json!({
        "status": "running",
        "current_model": current_model,
        "pools": pools,
    }))
}

async fn models(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let models: Vec<serde_json::Value> = state
        .config
        .providers
        .iter()
        .flat_map(|(name, p)| {
            p.models
                .iter()
                .map(move |m| json!({"provider": name, "model": m}))
        })
        .collect();
    Json(json!({"models": models}))
}

async fn usage(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let apps: Vec<serde_json::Value> = state
        .tracker
        .all()
        .iter()
        .map(|(name, u)| {
            json!({
                "app": name,
                "input_tokens": u.total_input_tokens,
                "output_tokens": u.total_output_tokens,
                "total_tokens": u.total_tokens(),
                "requests": u.request_count,
            })
        })
        .collect();
    Json(json!({"usage": apps}))
}
