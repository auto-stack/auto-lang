use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

// PLAN-729: file response glue (shared host service; version-agnostic FileReply)
#[allow(dead_code)]
fn __file_header_pairs(headers: &axum::http::HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|val| (k.as_str().to_string(), val.to_string()))
        })
        .collect()
}
#[allow(dead_code)]
async fn __file_reply(
    descriptor: auto_lang::a2r_std::http::FileResponse,
    method: &axum::http::Method,
    headers: &axum::http::HeaderMap,
) -> axum::response::Response {
    let hdrs = __file_header_pairs(headers);
    let deadline =
        std::time::Instant::now() + auto_lang::http_file_service::file_serve_limits().prepare_timeout;
    let reply = auto_lang::http_file_service::serve_file_response(
        &descriptor,
        auto_lang::http_file_service::ServeFileRequest {
            method: method.as_str(),
            request_headers: &hdrs,
            prepare_deadline: deadline,
            finish_hook: None,
            idle_timeout: None,
        },
    )
    .await;
    let mut builder = axum::response::Response::builder().status(
        axum::http::StatusCode::from_u16(reply.status)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    );
    for (k, v) in &reply.headers {
        if let (Ok(name), Ok(value)) = (
            axum::http::HeaderName::try_from(k.as_str()),
            axum::http::HeaderValue::from_str(v),
        ) {
            builder = builder.header(name, value);
        }
    }
    match reply.body {
        auto_lang::http_file_service::FileReplyBody::None => {
            builder.body(axum::body::Body::empty()).unwrap()
        }
        auto_lang::http_file_service::FileReplyBody::Inline(b) => {
            builder.body(axum::body::Body::from(b)).unwrap()
        }
        auto_lang::http_file_service::FileReplyBody::Stream(s) => {
            builder.body(axum::body::Body::from_stream(s)).unwrap()
        }
    }
}
#[allow(dead_code)]
async fn file_response_reply_missing(message: &str) -> axum::response::Response {
    let body = format!("{{\"error\":{}}}", message);
    axum::response::Response::builder()
        .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap()
}

pub type Db = Arc<Mutex<Vec<Probe>>>;

pub async fn echo(State(db): State<Db>, method: axum::http::Method, headers: axum::http::HeaderMap) -> axum::response::Response {
    let mut path: String = repository.path();
    return __file_reply(auto_lang::a2r_std::http::file_response("D:/files", path.as_str(), "{}"), &method, &headers).await;
}
