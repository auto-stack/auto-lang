use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::notebook::{Diagnostic, NotebookCellMeta, NotebookState, SessionStatus, VariableInfo};
use crate::routes::trans;

// ============================================================================
// Request / Response types
// ============================================================================

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    #[allow(dead_code)]
    pub title: Option<String>,
}

#[derive(Serialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub cell_id: String,
    pub source: String,
    pub notebook_cells: Option<Vec<NotebookCellMeta>>,
}

#[derive(Serialize)]
pub struct ExecuteResponse {
    pub stdout: String,
    pub stderr: String,
    pub result: String,
    pub time_ms: u64,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize)]
pub struct VariablesResponse {
    pub variables: Vec<VariableInfo>,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: SessionStatus,
}

#[derive(Deserialize)]
pub struct TranspileRequest {
    pub source: String,
    pub target: String,
}

#[derive(Serialize)]
pub struct TranspileResponse {
    pub code: String,
    pub target: String,
    pub source_map: Vec<auto_lang::trans::SourceMapEntry>,
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/notebook/session — Create a new notebook session
pub async fn create_session_handler(
    State(state): State<NotebookState>,
    Json(_req): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, AppError> {
    let id = state.create_session().await;
    Ok(Json(CreateSessionResponse { session_id: id }))
}

/// POST /api/notebook/{sid}/execute — Execute a cell in the session
pub async fn execute_handler(
    State(state): State<NotebookState>,
    Path(sid): Path<String>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, AppError> {
    let output = state.execute(sid, req.cell_id, req.source, req.notebook_cells).await;

    Ok(Json(ExecuteResponse {
        stdout: output.stdout,
        stderr: output.stderr,
        result: output.result,
        time_ms: output.time_ms,
        diagnostics: output.diagnostics,
    }))
}

/// GET /api/notebook/{sid}/status — Get session status
pub async fn status_handler(
    State(state): State<NotebookState>,
    Path(sid): Path<String>,
) -> Result<Json<StatusResponse>, AppError> {
    let status = state.status(sid).await;
    Ok(Json(StatusResponse { status }))
}

/// GET /api/notebook/{sid}/variables — Get current variables
pub async fn variables_handler(
    State(state): State<NotebookState>,
    Path(sid): Path<String>,
) -> Result<Json<VariablesResponse>, AppError> {
    let vars = state.variables(sid).await;
    Ok(Json(VariablesResponse { variables: vars }))
}

/// POST /api/notebook/{sid}/transpile — Transpile code (stateless, reuses existing pipeline)
pub async fn transpile_handler(
    Path(_sid): Path<String>,
    Json(req): Json<TranspileRequest>,
) -> Result<Json<TranspileResponse>, AppError> {
    let target = req.target.clone();
    let (code, source_map) = tokio::task::spawn_blocking(move || match target.as_str() {
        "rust" => trans::transpile_rust(&req.source),
        "c" => trans::transpile_c(&req.source),
        "python" => trans::transpile_python(&req.source),
        "typescript" => trans::transpile_typescript(&req.source),
        "abt" | "bytecode" => trans::transpile_abt(&req.source),
        _ => Err(AppError::Internal(format!("Unknown target: {}", target))),
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(TranspileResponse {
        code,
        target: req.target,
        source_map,
    }))
}

/// DELETE /api/notebook/{sid} — Destroy a session
pub async fn delete_session_handler(
    State(state): State<NotebookState>,
    Path(sid): Path<String>,
) -> StatusCode {
    state.destroy(sid);
    StatusCode::NO_CONTENT
}
