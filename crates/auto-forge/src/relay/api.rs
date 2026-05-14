//! Relay HTTP API
//!
//! Axum handlers for the Agents Relay module.
//! Uses a global in-memory store for simplicity.

use crate::relay::flow::FlowSpec;
use crate::relay::handoff::HandoffDocument;
use crate::relay::pipeline::GateDecision;
use crate::relay::profession::ProfessionRegistry;
use crate::relay::store::{
    advance_run, get_run, list_runs, new_run_store, resolve_gate, start_run, submit_handoff,
    RunState, RunStore, RunSummary,
};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::convert::Infallible;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

// Global in-memory run store
static RUN_STORE: LazyLock<RunStore> = LazyLock::new(new_run_store);

// Global event broadcast for SSE
static EVENT_TX: LazyLock<broadcast::Sender<RunEventBroadcast>> = LazyLock::new(|| {
    let (tx, _rx) = broadcast::channel(256);
    tx
});

#[derive(Clone, Debug)]
pub struct RunEventBroadcast {
    pub run_id: String,
    pub event_type: String,
}

// -------------------------------------------------------------------------
// DTOs
// -------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct StartRunRequest {
    pub run_id: Option<String>,
    pub flow_id: String,
    pub steps: Vec<FlowStepDto>,
}

#[derive(serde::Deserialize)]
pub struct FlowStepDto {
    pub id: String,
    pub profession_id: String,
    #[serde(default)]
    pub gate: String,
}

#[derive(serde::Serialize)]
pub struct StartRunResponse {
    pub run_id: String,
    pub state: RunState,
}

#[derive(serde::Deserialize)]
pub struct GateRequest {
    pub decision: String,
    #[serde(default)]
    pub feedback: Option<String>,
    #[serde(default)]
    pub changes: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct HandoffRequest {
    pub handoff: HandoffDocument,
}

#[derive(serde::Serialize)]
pub struct ProfessionsResponse {
    pub professions: Vec<ProfessionDto>,
}

#[derive(serde::Serialize)]
pub struct ProfessionDto {
    pub id: String,
    pub name: String,
    pub phase: String,
    pub owned_sections: Vec<String>,
    pub allowed_tools: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct SoulsResponse {
    pub souls: Vec<SoulDto>,
}

#[derive(serde::Serialize)]
pub struct SoulDto {
    pub id: String,
    pub name: String,
}

// -------------------------------------------------------------------------
// Handlers
// -------------------------------------------------------------------------

pub async fn list_professions() -> Json<ProfessionsResponse> {
    let professions = ProfessionRegistry::new().list().into_iter().map(|p| ProfessionDto {
        id: p.id.clone(),
        name: p.name.clone(),
        phase: p.phase.as_str().to_string(),
        owned_sections: p.owned_sections.iter().map(|s| s.as_str().to_string()).collect(),
        allowed_tools: p.allowed_tools.clone(),
    }).collect();

    Json(ProfessionsResponse { professions })
}

pub async fn list_souls() -> Json<SoulsResponse> {
    let souls = vec![
        SoulDto { id: "assistant".into(), name: "Assistant".into() },
        SoulDto { id: "advisor".into(), name: "Advisor".into() },
        SoulDto { id: "planner".into(), name: "Planner".into() },
        SoulDto { id: "architect".into(), name: "Architect".into() },
        SoulDto { id: "coder".into(), name: "Coder".into() },
        SoulDto { id: "tester".into(), name: "Tester".into() },
        SoulDto { id: "reviewer".into(), name: "Reviewer".into() },
        SoulDto { id: "documenter".into(), name: "Documenter".into() },
    ];
    Json(SoulsResponse { souls })
}

pub async fn list_runs_handler() -> Json<Vec<RunSummary>> {
    Json(list_runs(&RUN_STORE))
}

pub async fn get_run_handler(Path(run_id): Path<String>) -> Result<Json<RunState>, StatusCode> {
    get_run(&RUN_STORE, &run_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn start_run_handler(
    Json(req): Json<StartRunRequest>,
) -> Result<Json<StartRunResponse>, StatusCode> {
    let mut flow = FlowSpec::new(&req.flow_id);
    for step in req.steps {
        let gate = match step.gate.as_str() {
            "human" => crate::relay::flow::GateType::Human,
            _ => crate::relay::flow::GateType::Auto,
        };
        flow.add_step(
            crate::relay::flow::FlowStep::new(step.id, step.profession_id)
                .with_gate(gate),
        );
    }

    let run_id = req.run_id.unwrap_or_else(|| format!("run-{}", uuid::Uuid::new_v4()));
    match start_run(&RUN_STORE, flow, &run_id) {
        Ok(run_state) => {
            let _ = EVENT_TX.send(RunEventBroadcast {
                run_id: run_id.clone(),
                event_type: "run_started".into(),
            });
            Ok(Json(StartRunResponse { run_id, state: run_state }))
        }
        Err(_) => Err(StatusCode::CONFLICT),
    }
}

pub async fn advance_run_handler(
    Path(run_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = advance_run(&RUN_STORE, &run_id).ok_or(StatusCode::NOT_FOUND)?;
    let _ = EVENT_TX.send(RunEventBroadcast {
        run_id: run_id.clone(),
        event_type: "step_advanced".into(),
    });
    Ok(Json(serde_json::json!({ "result": format!("{:?}", result) })))
}

pub async fn submit_handoff_handler(
    Path(run_id): Path<String>,
    Json(req): Json<HandoffRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = submit_handoff(&RUN_STORE, &run_id, req.handoff).ok_or(StatusCode::NOT_FOUND)?;
    let _ = EVENT_TX.send(RunEventBroadcast {
        run_id: run_id.clone(),
        event_type: "handoff_submitted".into(),
    });
    Ok(Json(serde_json::json!({ "result": format!("{:?}", result) })))
}

pub async fn resolve_gate_handler(
    Path(run_id): Path<String>,
    Json(req): Json<GateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let decision = match req.decision.as_str() {
        "approve" => GateDecision::Approve,
        "reject" => GateDecision::Reject {
            feedback: req.feedback.unwrap_or_default(),
        },
        "edit" => GateDecision::Edit {
            changes: req.changes.unwrap_or_default(),
        },
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let result = resolve_gate(&RUN_STORE, &run_id, decision).ok_or(StatusCode::NOT_FOUND)?;
    let _ = EVENT_TX.send(RunEventBroadcast {
        run_id: run_id.clone(),
        event_type: "gate_resolved".into(),
    });
    Ok(Json(serde_json::json!({ "result": format!("{:?}", result) })))
}

/// SSE stream for run events.
pub async fn run_events_handler(
    Path(run_id): Path<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = EVENT_TX.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(move |msg| {
            let Ok(msg) = msg else { return None };
            if msg.run_id != run_id {
                return None;
            }
            let event = Event::default()
                .event("run_event")
                .data(serde_json::to_string(&serde_json::json!({
                    "run_id": msg.run_id,
                    "event_type": msg.event_type,
                })).unwrap_or_default());
            Some(Ok(event))
        });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

// -------------------------------------------------------------------------
// Router
// -------------------------------------------------------------------------

pub fn relay_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/forge/relay/professions", get(list_professions))
        .route("/api/forge/relay/souls", get(list_souls))
        .route("/api/forge/relay/runs", get(list_runs_handler).post(start_run_handler))
        .route("/api/forge/relay/runs/{run_id}", get(get_run_handler))
        .route("/api/forge/relay/runs/{run_id}/advance", post(advance_run_handler))
        .route("/api/forge/relay/runs/{run_id}/handoff", post(submit_handoff_handler))
        .route("/api/forge/relay/runs/{run_id}/gate", post(resolve_gate_handler))
        .route("/api/forge/relay/runs/{run_id}/events", get(run_events_handler))
}
