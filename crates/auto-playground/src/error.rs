use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub enum AppError {
    #[allow(dead_code)]
    VmError(String),
    CompileError(String),
    Internal(String),
    #[allow(dead_code)]
    NotFound(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::VmError(msg) => write!(f, "VM error: {msg}"),
            AppError::CompileError(msg) => write!(f, "Compile error: {msg}"),
            AppError::Internal(msg) => write!(f, "Internal error: {msg}"),
            AppError::NotFound(msg) => write!(f, "Not found: {msg}"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::VmError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::CompileError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
        };
        let body = axum::Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
