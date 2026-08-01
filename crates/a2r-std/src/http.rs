//! HTTP client module for a2r transpiled code.
//!
//! Provides synchronous HTTP POST functions with thread-local status tracking,
//! used by the transpiled agent runtime to call LLM APIs.
//! Also provides streaming HTTP (HTTPStream) for SSE/chunk-by-chunk reading
//! (plan 013 G6: lets the Auto .at client's complete_stream link under a2r).

use std::cell::Cell;
use std::io::Read;

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

// ═══════════════════════════════════════════════════════════════════════════
// Streaming HTTP (plan 013 G6)
// ═══════════════════════════════════════════════════════════════════════════

/// A streaming HTTP response reader. Mirrors the AutoVM stdlib's HTTPStream
/// type (stdlib/auto/http.at:225-244). Used by the transpiled auto-ai-client
/// `.at` source's `complete_stream` to read SSE chunks.
///
/// NOTE: this is **synchronous blocking I/O** (std::io::Read via ureq). In an
/// async (tokio) context, callers must wrap the read loop in
/// `tokio::task::spawn_blocking` to avoid stalling the runtime.
pub struct HTTPStream {
    reader: Box<dyn Read + Send>,
    done: bool,
}

/// Send a streaming POST request. Returns an `HTTPStream` that can be read
/// chunk by chunk via `next()`.
///
/// `headers` is a newline-separated string of "Key: Value" pairs (matching the
/// AutoVM stdlib's convention, e.g. "Content-Type: application/json\nX-App-Name: foo").
pub fn post_stream_with_headers(url: &str, body: &str, headers: &str) -> HTTPStream {
    let mut req = ureq::post(url);
    req = req.set("Content-Type", "application/json");
    for line in headers.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once(": ") {
            req = req.set(k, v.trim());
        }
    }
    match req.send_string(body) {
        Ok(resp) => {
            let status = resp.status();
            set_last_status(status as u32);
            if status >= 200 && status < 300 {
                HTTPStream { reader: resp.into_reader(), done: false }
            } else {
                HTTPStream { reader: Box::new(std::io::empty()), done: true }
            }
        }
        Err(ureq::Error::Status(code, _)) => {
            set_last_status(code as u32);
            HTTPStream { reader: Box::new(std::io::empty()), done: true }
        }
        Err(ureq::Error::Transport(_)) => {
            set_last_status(0);
            HTTPStream { reader: Box::new(std::io::empty()), done: true }
        }
    }
}

impl HTTPStream {
    /// Read the next chunk. Returns empty string when done or on error.
    pub fn next(&mut self) -> String {
        if self.done { return String::new(); }
        let mut buf = [0u8; 4096];
        match self.reader.read(&mut buf) {
            Ok(0) | Err(_) => { self.done = true; String::new() }
            Ok(n) => String::from_utf8_lossy(&buf[..n]).to_string(),
        }
    }

    /// Returns 1 if done, 0 otherwise (mirrors AutoVM int convention).
    pub fn is_done(&self) -> i32 { if self.done { 1 } else { 0 } }

    /// Close the stream.
    pub fn close(&mut self) { self.done = true; }
}
