use axum::Json;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Deserialize)]
pub struct TransRequest {
    pub source: String,
    pub target: String, // "rust" | "c"
}

#[derive(Serialize)]
pub struct TransResponse {
    pub code: String,
    pub target: String,
}

pub async fn trans_handler(
    Json(req): Json<TransRequest>,
) -> Result<Json<TransResponse>, AppError> {
    let target = req.target.clone();
    let source = req.source.clone();

    let code = tokio::task::spawn_blocking(move || match target.as_str() {
        "rust" => transpile_rust(&source),
        "c" => transpile_c(&source),
        _ => Err(AppError::Internal(format!("Unknown target: {target}"))),
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(TransResponse {
        code,
        target: req.target,
    }))
}

fn transpile_rust(source: &str) -> Result<String, AppError> {
    use auto_lang::trans::rust::transpile_rust as auto_transpile_rust;
    use auto_lang::trans::Sink;

    let mut sink: Sink = auto_transpile_rust("playground", source)
        .map_err(|e| AppError::CompileError(e.to_string()))?;

    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(String::from_utf8_lossy(output).to_string())
}

fn transpile_c(source: &str) -> Result<String, AppError> {
    use auto_lang::trans::c::transpile_c as auto_transpile_c;
    use auto_lang::trans::Sink;

    let mut sink: Sink = auto_transpile_c("playground", source)
        .map_err(|e| AppError::CompileError(e.to_string()))?;

    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(String::from_utf8_lossy(output).to_string())
}
