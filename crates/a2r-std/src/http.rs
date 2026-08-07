//! HTTP client module for a2r transpiled code.
//!
//! Provides synchronous HTTP POST functions with thread-local status tracking,
//! used by the transpiled agent runtime to call LLM APIs.
//! Also provides streaming HTTP (HTTPStream) for SSE/chunk-by-chunk reading
//! (plan 013 G6: lets the Auto .at client's complete_stream link under a2r).

use std::cell::Cell;
use std::io::Read;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

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

// =============================================================================
// Plan 013 G6: request-builder + streaming HTTP (for transpiled auto-ai-client)
//
// Auto's `http.request(method, url)` returns a `RequestBuilder` whose chained
// `.header(k,v)` / `.body(s)` / `.timeout(ms)` / `.send()` calls must resolve in
// transpiled Rust. Likewise `http.post_stream_with_headers(url, body, headers)`
// returns an `HTTPStream` with `.next()` / `.is_done()` / `.close()`.
//
// These mirror the VM-side stdlib (auto-lang/stdlib/auto/http.at) contract so
// the same .at source runs both in the VM and via a2r→Rust.
// =============================================================================

/// A fluent HTTP request builder (the Rust realization of Auto's
/// `http.request(method, url)` → `.header/.body/.timeout/.send` chain).
pub struct RequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: Option<u64>,
}

/// Build a new request (entry point; mirrors `http.request(method, url)`).
pub fn request(method: &str, url: &str) -> RequestBuilder {
    RequestBuilder {
        method: method.to_string(),
        url: url.to_string(),
        headers: Vec::new(),
        body: None,
        timeout_ms: None,
    }
}

/// Shared blocking send logic (runs ureq on a worker thread + joins).
/// Used by both `RequestBuilder::send` (sync) and `send_async` (via spawn_blocking).
fn send_request_blocking(
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: Option<u64>,
) -> Response {
    let result = std::thread::spawn(move || -> Result<(u32, Vec<u8>), String> {
        let mut req = match method.as_str() {
            "GET" => ureq::get(&url),
            "POST" => ureq::post(&url),
            "PUT" => ureq::put(&url),
            "DELETE" => ureq::delete(&url),
            _ => ureq::request(method.as_str(), &url),
        };
        for (k, v) in &headers {
            req = req.set(k, v);
        }
        if let Some(ms) = timeout_ms {
            req = req.timeout(std::time::Duration::from_millis(ms));
        }
        let send_result = match &body {
            Some(b) => req.send_string(b),
            None => req.call(),
        };
        match send_result {
            Ok(response) => {
                let status = response.status() as u32;
                let mut buf = Vec::new();
                response.into_reader().read_to_end(&mut buf).unwrap_or(0);
                Ok((status, buf))
            }
            Err(ureq::Error::Status(code, response)) => {
                let status = code as u32;
                let mut buf = Vec::new();
                response.into_reader().read_to_end(&mut buf).unwrap_or(0);
                Ok((status, buf))
            }
            Err(ureq::Error::Transport(e)) => Err(e.to_string()),
        }
    })
    .join()
    .unwrap_or_else(|_| Err("thread panicked".to_string()));

    match result {
        Ok((status, body)) => {
            set_last_status(status);
            Response { status, body }
        }
        Err(_e) => {
            set_last_status(0);
            Response { status: 0, body: Vec::new() }
        }
    }
}

impl RequestBuilder {
    /// Add a header. Mirrors `RequestBuilder.header(self, key, value)`.
    pub fn header(mut self, key: &str, value: &str) -> RequestBuilder {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    /// Set the body. Mirrors `RequestBuilder.body(self, body)`.
    /// Accepts anything string-like (the transpiled call site may pass an owned
    /// `String` or a `&str`).
    pub fn body(mut self, body: impl AsRef<str>) -> RequestBuilder {
        self.body = Some(body.as_ref().to_string());
        self
    }

    /// Set a timeout in milliseconds. Mirrors `RequestBuilder.timeout(self, ms)`.
    pub fn timeout(mut self, ms: u32) -> RequestBuilder {
        self.timeout_ms = Some(ms as u64);
        self
    }

    /// Send the request. Mirrors `RequestBuilder.send(self) -> Response`.
    ///
    /// Blocks: runs the ureq call on a `std::thread::spawn` worker and `.join()`s.
    /// For use from synchronous (non-async) call sites. Async call sites should
    /// use `send_async` instead to avoid blocking the tokio executor.
    pub fn send(self) -> Response {
        send_request_blocking(self.method, self.url, self.headers, self.body, self.timeout_ms)
    }

    /// Async send: runs the same ureq call via `tokio::task::spawn_blocking`,
    /// yielding the executor while the blocking HTTP I/O runs on the dedicated
    /// blocking thread pool. This is the correct entry point for code running
    /// inside an async fn / tokio runtime (Plan 024 sync-in-async fix).
    pub async fn send_async(self) -> Response {
        let method = self.method;
        let url = self.url;
        let headers = self.headers;
        let body = self.body;
        let timeout_ms = self.timeout_ms;
        tokio::task::spawn_blocking(move || {
            send_request_blocking(method, url, headers, body, timeout_ms)
        })
        .await
        .unwrap_or_else(|_| Response { status: 0, body: Vec::new() })
    }
}

/// An HTTP response. Mirrors Auto's `http.Response`.
pub struct Response {
    status: u32,
    body: Vec<u8>,
}

impl Response {
    /// The HTTP status code. Mirrors `Response.status_code(self) -> int`.
    pub fn status_code(&self) -> u32 {
        self.status
    }

    /// The response body as bytes. Mirrors `Response.body_bytes(self) -> []byte`.
    pub fn body_bytes(&self) -> Vec<u8> {
        self.body.clone()
    }

    /// Look up a response header. Mirrors `Response.header_get(self, key)`.
    /// (Not tracked for the builder path; returns "".)
    pub fn header_get(&self, _key: &str) -> String {
        String::new()
    }
}

/// A streaming HTTP response. Mirrors Auto's `http.HTTPStream`.
///
/// A background thread reads the response body in chunks and feeds them over a
/// channel; `next()` pulls one chunk, `is_done()` reports end-of-stream.
/// (Synthetic markers deliver the status code and end-of-stream signal over the
/// same channel so the call sites compiled from `.at` need no extra plumbing.)
pub struct HTTPStream {
    rx: Arc<Mutex<mpsc::Receiver<String>>>,
    done: Arc<Mutex<bool>>,
}

/// Create a streaming POST request with custom headers.
/// `headers` is a single string of newline-separated `"Key: Value"` lines
/// (mirrors Auto's `post_stream_with_headers(url, body, headers)`).
pub fn post_stream_with_headers(url: &str, body: &str, headers: &str) -> HTTPStream {
    let (tx, rx) = mpsc::channel::<String>();
    let rx = Arc::new(Mutex::new(rx));
    let done = Arc::new(Mutex::new(false));

    let url = url.to_string();
    let body = body.to_string();
    let parsed_headers: Vec<(String, String)> = headers
        .split('\n')
        .filter_map(|line| {
            let line = line.trim();
            let idx = line.find(':')?;
            let (k, v) = line.split_at(idx);
            Some((k.trim().to_string(), v[1..].trim().to_string()))
        })
        .collect();

    let done_clone = Arc::clone(&done);
    std::thread::spawn(move || {
        let mut req = ureq::post(&url);
        for (k, v) in &parsed_headers {
            req = req.set(k, v);
        }
        let result = req.send_string(&body);
        match result {
            Ok(response) => {
                let _ = tx.send(format!("__status__:{}", response.status()));
                drain_body(&tx, response.into_reader());
            }
            Err(ureq::Error::Status(code, response)) => {
                let _ = tx.send(format!("__status__:{code}"));
                drain_body(&tx, response.into_reader());
            }
            Err(ureq::Error::Transport(_e)) => {
                let _ = tx.send("__status__:0".to_string());
            }
        }
        let _ = tx.send("__done__".to_string());
        let mut d = done_clone.lock().unwrap();
        *d = true;
    });

    HTTPStream { rx, done }
}

/// Read a response body reader to EOF, sending 8 KiB text chunks on `tx`.
fn drain_body(tx: &mpsc::Sender<String>, mut reader: impl Read) {
    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]).into_owned();
                if tx.send(chunk).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

impl HTTPStream {
    /// Read the next chunk from the stream. Returns "" when the stream is
    /// exhausted (or the synthetic "__done__" marker is reached). Mirrors
    /// `HTTPStream.next(self) -> str`.
    pub fn next(&mut self) -> String {
        let rx = self.rx.lock().unwrap();
        match rx.recv() {
            Ok(chunk) => {
                if chunk == "__done__" {
                    return String::new();
                }
                // Swallow the leading status marker; callers consume body chunks.
                if chunk.starts_with("__status__:") {
                    drop(rx);
                    return self.next();
                }
                chunk
            }
            Err(_) => String::new(),
        }
    }

    /// 1 if the stream is finished, 0 if more chunks may arrive. Mirrors
    /// `HTTPStream.is_done(self) -> int`.
    pub fn is_done(&self) -> u32 {
        let d = self.done.lock().unwrap();
        if *d {
            1
        } else {
            0
        }
    }

    /// Close/release the stream. Mirrors `HTTPStream.close(self)`. The
    /// background thread exits when the receiver is dropped; this is a no-op
    /// placeholder that lets transpiled `.close()` calls compile.
    pub fn close(&self) {}
}

// =============================================================================
// Plan 024 sync-in-async fix: async streaming HTTP for transpiled code that
// runs inside a tokio runtime. The synchronous HTTPStream (above) uses
// std::thread + std::sync::mpsc + blocking recv(); the async variant below
// uses tokio::task::spawn_blocking for the ureq reader + tokio::sync::mpsc
// so the consumer loop can `.recv().await` without blocking the executor.
// =============================================================================

/// An async streaming HTTP response. The ureq body is drained on a blocking
/// thread; chunks arrive on a `tokio::sync::mpsc::UnboundedReceiver`. Each
/// chunk is a text `String` (UTF-8 lossy, 8 KiB). The stream ends when the
/// receiver yields `None` (sender dropped = body fully read or error).
///
/// The first message is a synthetic `__status__:CODE` marker so the caller can
/// observe the HTTP status (mirrors the sync HTTPStream contract).
pub struct AsyncHTTPStream {
    rx: tokio::sync::mpsc::UnboundedReceiver<String>,
}

/// Create an async streaming POST request with custom headers (Plan 024).
/// Same `headers` format as `post_stream_with_headers` (newline-separated
/// `"Key: Value"`). The ureq call runs on a `spawn_blocking` thread.
pub async fn post_stream_with_headers_async(
    url: &str,
    body: &str,
    headers: &str,
) -> AsyncHTTPStream {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let url = url.to_string();
    let body = body.to_string();
    let parsed_headers: Vec<(String, String)> = headers
        .split('\n')
        .filter_map(|line| {
            let line = line.trim();
            let idx = line.find(':')?;
            let (k, v) = line.split_at(idx);
            Some((k.trim().to_string(), v[1..].trim().to_string()))
        })
        .collect();

    tokio::task::spawn_blocking(move || {
        let mut req = ureq::post(&url);
        for (k, v) in &parsed_headers {
            req = req.set(k, v);
        }
        let result = req.send_string(&body);
        match result {
            Ok(response) => {
                let _ = tx.send(format!("__status__:{}", response.status()));
                drain_body_async(&tx, response.into_reader());
            }
            Err(ureq::Error::Status(code, response)) => {
                let _ = tx.send(format!("__status__:{code}"));
                drain_body_async(&tx, response.into_reader());
            }
            Err(ureq::Error::Transport(_e)) => {
                let _ = tx.send("__status__:0".to_string());
            }
        }
        // tx drops here → rx.recv().await returns None, signaling end-of-stream.
    });

    AsyncHTTPStream { rx }
}

impl AsyncHTTPStream {
    /// Await the next chunk. Returns `Some(chunk)` for each text piece, or
    /// `None` when the stream is fully read (sender dropped). The synthetic
    /// `__status__:CODE` marker is returned as the first Some, then swallowed
    /// on subsequent calls (use `recv_status` first if you need the code).
    pub async fn next(&mut self) -> Option<String> {
        loop {
            match self.rx.recv().await {
                Some(chunk) if chunk.starts_with("__status__:") => {
                    // Stash the status on thread-local and continue to body chunks.
                    if let Ok(code) = chunk["__status__:".len()..].parse::<u32>() {
                        set_last_status(code);
                    }
                    continue;
                }
                Some(chunk) => return Some(chunk),
                None => return None,
            }
        }
    }
}

/// Read a response body reader to EOF, sending 8 KiB text chunks on `tx`.
fn drain_body_async(tx: &tokio::sync::mpsc::UnboundedSender<String>, mut reader: impl Read) {
    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]).into_owned();
                if tx.send(chunk).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

// ===========================================================================
// Plan 349: File download + multipart upload (parity with VM http module).
// ===========================================================================

/// Download a file from `url` and save it to `file_path`.
///
/// Returns the HTTP status code (200 on success, 0 on transport error).
/// Mirrors Auto's `http.download(url, file_path) -> int`.
pub fn download(url: &str, file_path: &str) -> u32 {
    let resp = ureq::get(url).call();
    match resp {
        Ok(response) => {
            let status = response.status() as u32;
            if let Ok(mut file) = std::fs::File::create(file_path) {
                let _ = std::io::copy(&mut response.into_reader(), &mut file);
            }
            set_last_status(status);
            status
        }
        Err(ureq::Error::Status(code, _)) => {
            set_last_status(code as u32);
            code as u32
        }
        Err(ureq::Error::Transport(_)) => {
            set_last_status(0);
            0
        }
    }
}

/// Upload a single file to `url` via raw POST body.
///
/// Returns the HTTP status code. The file contents are sent as the request
/// body with `Content-Type: application/octet-stream`.
/// Mirrors Auto's `http.upload(url, file_path) -> int`.
pub fn upload(url: &str, file_path: &str) -> u32 {
    let data = match std::fs::read(file_path) {
        Ok(d) => d,
        Err(_) => {
            set_last_status(0);
            return 0;
        }
    };
    let resp = ureq::post(url)
        .set("Content-Type", "application/octet-stream")
        .send_bytes(&data);
    match resp {
        Ok(response) => {
            let status = response.status() as u32;
            set_last_status(status);
            status
        }
        Err(ureq::Error::Status(code, _)) => {
            set_last_status(code as u32);
            code as u32
        }
        Err(ureq::Error::Transport(_)) => {
            set_last_status(0);
            0
        }
    }
}

/// Download with resume support — sends a Range header for `offset` bytes.
///
/// If the server supports range requests (206), appends to the existing file.
/// Otherwise (200), overwrites from the beginning.
/// Mirrors Auto's `http.download_resume(url, file_path, offset) -> int`.
pub fn download_resume(url: &str, file_path: &str, offset: u64) -> u32 {
    let req = ureq::get(url).set("Range", &format!("bytes={offset}-"));
    match req.call() {
        Ok(response) => {
            let status = response.status() as u32;
            let file_result = if status == 206 {
                std::fs::OpenOptions::new().append(true).open(file_path)
            } else {
                std::fs::File::create(file_path)
            };
            if let Ok(mut file) = file_result {
                let _ = std::io::copy(&mut response.into_reader(), &mut file);
            }
            set_last_status(status);
            status
        }
        Err(ureq::Error::Status(code, _)) => {
            set_last_status(code as u32);
            code as u32
        }
        Err(ureq::Error::Transport(_)) => {
            set_last_status(0);
            0
        }
    }
}
