//! HTTP client module for a2r transpiled code.
//!
//! Provides synchronous HTTP POST functions with thread-local status tracking,
//! used by the transpiled agent runtime to call LLM APIs.

use std::cell::Cell;

thread_local! {
    static LAST_STATUS: Cell<u32> = Cell::new(0);
}

/// Store the last HTTP response status code (thread-local).
pub fn set_last_status(status: u32) {
    LAST_STATUS.with(|s| s.set(status));
}

/// Retrieve the last HTTP response status code (thread-local).
pub fn last_status() -> u32 {
    LAST_STATUS.with(|s| s.get())
}

/// Synchronous HTTP POST with `x-api-key` header (Anthropic-style auth).
///
/// Sends JSON body with `Content-Type: application/json` and `x-api-key: <api_key>`.
/// Returns `(status_code, response_body)`.
/// On connection or request failure, returns `(0, error_message)`.
pub fn post_sync(url: &str, body: &str, api_key: &str) -> (u32, String) {
    let result = ureq::post(url)
        .set("Content-Type", "application/json")
        .set("x-api-key", api_key)
        .set("anthropic-version", "2023-06-01")
        .send_string(body);

    match result {
        Ok(response) => {
            let status = response.status();
            let body_text = response.into_string().unwrap_or_default();
            set_last_status(status as u32);
            (status as u32, body_text)
        }
        Err(ureq::Error::Status(code, response)) => {
            let body_text = response.into_string().unwrap_or_default();
            set_last_status(code as u32);
            (code as u32, body_text)
        }
        Err(ureq::Error::Transport(e)) => {
            let msg = format!("transport error: {}", e);
            set_last_status(0);
            (0, msg)
        }
    }
}

/// Synchronous HTTP POST with `Authorization: Bearer <api_key>` header (OpenAI-style auth).
///
/// Sends JSON body with `Content-Type: application/json` and `Authorization: Bearer <api_key>`.
/// Returns `(status_code, response_body)`.
/// On connection or request failure, returns `(0, error_message)`.
pub fn post_bearer_sync(url: &str, body: &str, api_key: &str) -> (u32, String) {
    let auth_header = format!("Bearer {}", api_key);
    let result = ureq::post(url)
        .set("Content-Type", "application/json")
        .set("Authorization", &auth_header)
        .send_string(body);

    match result {
        Ok(response) => {
            let status = response.status();
            let body_text = response.into_string().unwrap_or_default();
            set_last_status(status as u32);
            (status as u32, body_text)
        }
        Err(ureq::Error::Status(code, response)) => {
            let body_text = response.into_string().unwrap_or_default();
            set_last_status(code as u32);
            (code as u32, body_text)
        }
        Err(ureq::Error::Transport(e)) => {
            let msg = format!("transport error: {}", e);
            set_last_status(0);
            (0, msg)
        }
    }
}
