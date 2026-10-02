//! HTTP client module for a2r transpiled code.
//!
//! PLAN-724 T-03/T-05：本 facade 是**适配层**——全部网络执行改走共享内核
//! [`client`]（reqwest async + 固定 runtime + 有界准入 + typed 结果 +
//! RAII 取消）；本模块不再持有 ureq 网络执行、spawn_blocking 兜底或
//! detached 读取线程。历史签名（认证 tuple、Response/HTTPStream 形状、
//! last_status 线程局部）逐字节保留，`__status__`/`__done__` 控制串只在
//! 兼容层内部消化，不进内核状态。
//!
//! 文件 helper（download/upload/download_resume）按 PLAN-724 §3 仍走
//! ureq，不为删除依赖扩展范围。

pub mod client;

use crate::http::client::{
    ClientError, HttpRequest, HttpResponse as KernelResponse, HttpClientStream, StreamMode,
    StreamSpec, StreamItem,
};

thread_local! {
    static LAST_STATUS: std::cell::Cell<u32> = std::cell::Cell::new(0);
}

/// Store the last HTTP response status code (thread-local).
pub fn set_last_status(status: u32) {
    LAST_STATUS.with(|s| s.set(status));
}

/// Retrieve the last HTTP response status code (thread-local).
pub fn last_status() -> u32 {
    LAST_STATUS.with(|s| s.get())
}

fn transport_message(err: &ClientError) -> String {
    match err {
        ClientError::Transport(_) => format!("{err}"),
        other => format!("transport error: {other}"),
    }
}

/// Synchronous HTTP POST with `x-api-key` header (Anthropic-style auth).
///
/// Sends JSON body with `Content-Type: application/json` and `x-api-key: <api_key>`.
/// Returns `(status_code, response_body)`.
/// On connection or request failure, returns `(0, error_message)`.
pub fn post_sync(url: &str, body: &str, api_key: &str) -> (u32, String) {
    let req = HttpRequest {
        method: "POST".into(),
        url: url.to_string(),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
            ("x-api-key".into(), api_key.to_string()),
            ("anthropic-version".into(), "2023-06-01".into()),
        ],
        body: Some(body.as_bytes().to_vec()),
        timeout_ms: None,
    };
    match client::execute_blocking(req) {
        Ok(resp) => {
            set_last_status(resp.status as u32);
            (resp.status as u32, String::from_utf8_lossy(&resp.body).into_owned())
        }
        Err(e) => {
            set_last_status(0);
            (0, transport_message(&e))
        }
    }
}

/// Synchronous HTTP POST with `Authorization: Bearer <api_key>` header (OpenAI-style auth).
///
/// Sends JSON body with `Content-Type: application/json` and `Authorization: Bearer <api_key>`.
/// Returns `(status_code, response_body)`.
/// On connection or request failure, returns `(0, error_message)`.
pub fn post_bearer_sync(url: &str, body: &str, api_key: &str) -> (u32, String) {
    let req = HttpRequest {
        method: "POST".into(),
        url: url.to_string(),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
            ("Authorization".into(), format!("Bearer {}", api_key)),
        ],
        body: Some(body.as_bytes().to_vec()),
        timeout_ms: None,
    };
    match client::execute_blocking(req) {
        Ok(resp) => {
            set_last_status(resp.status as u32);
            (resp.status as u32, String::from_utf8_lossy(&resp.body).into_owned())
        }
        Err(e) => {
            set_last_status(0);
            (0, transport_message(&e))
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
// 执行面（PLAN-724）：同步 send 走内核同步桥接（仅同步上下文），async 上
// 下文用 send_async（内核 async，无 spawn_blocking）。
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

impl RequestBuilder {
    fn into_request(self) -> HttpRequest {
        HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: self.body.map(|b| b.into_bytes()),
            timeout_ms: self.timeout_ms,
        }
    }

    fn from_kernel(resp: Result<KernelResponse, ClientError>) -> Response {
        match resp {
            Ok(k) => {
                set_last_status(k.status as u32);
                Response {
                    status: k.status as u32,
                    headers: k.headers,
                    body: k.body,
                }
            }
            Err(_e) => {
                set_last_status(0);
                Response {
                    status: 0,
                    headers: Vec::new(),
                    body: Vec::new(),
                }
            }
        }
    }
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
    /// 同步桥接：走内核同步入口。仅供同步（非 async）上下文使用——async
    /// 上下文须用 [`RequestBuilder::send_async`]（内核 async 路径，不阻塞
    /// 执行线程）。
    pub fn send(self) -> Response {
        Self::from_kernel(client::execute_blocking(self.into_request()))
    }

    /// Async send：内核 async 执行（排队/建立/读体全程让出执行线程）。
    /// async fn / tokio runtime 内的正确入口（Plan 024 语义、PLAN-724 实现）。
    pub async fn send_async(self) -> Response {
        Self::from_kernel(client::execute(self.into_request()).await)
    }
}

/// An HTTP response. Mirrors Auto's `http.Response`.
pub struct Response {
    status: u32,
    headers: Vec<(String, String)>,
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
    /// 大小写无关；缺失返回 ""（PLAN-724：headers 由内核捕获，不再恒空）。
    pub fn header_get(&self, key: &str) -> String {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }
}

/// A streaming HTTP response. Mirrors Auto's `http.HTTPStream`.
///
/// PLAN-724：内核 typed 流（有界队列/背压/真取消）的**同步兼容壳**——
/// `next()` 经内核同步桥接等待（仅同步上下文）；close 真正回收上游。
pub struct HTTPStream {
    inner: HttpClientStream,
}

/// Create a streaming POST request with custom headers.
/// `headers` is a single string of newline-separated `"Key: Value"` lines
/// (mirrors Auto's `post_stream_with_headers(url, body, headers)`).
/// 格式错误 = 终结性失败（流以错误态呈现，不按空 headers 发射）。
pub fn post_stream_with_headers(url: &str, body: &str, headers: &str) -> HTTPStream {
    let headers = match client::parse_line_headers(headers) {
        Ok(h) => h,
        Err(e) => {
            // 无法建立合法请求：流直接终结为错误（next/is_done 可观察）。
            let inner = client::open_stream(StreamSpec {
                method: "POST".into(),
                url: url.to_string(),
                body: None,
                headers: Vec::new(),
                mode: StreamMode::Raw,
            });
            inner.close();
            let _ = e;
            return HTTPStream { inner };
        }
    };
    let inner = client::open_stream(StreamSpec {
        method: "POST".into(),
        url: url.to_string(),
        body: Some(body.as_bytes().to_vec()),
        headers,
        mode: StreamMode::Raw,
    });
    HTTPStream { inner }
}

impl HTTPStream {
    /// Read the next chunk from the stream. Returns "" when the stream is
    /// exhausted. Mirrors `HTTPStream.next(self) -> str`.
    ///
    /// 同步桥接边界：阻塞等待仅在同步上下文合法；async 消费请用
    /// [`AsyncHTTPStream`]。
    pub fn next(&mut self) -> String {
        match client::kernel_handle().block_on(self.inner.next()) {
            Some(StreamItem::Data(s)) => s,
            _ => String::new(),
        }
    }
    /// 1 if the stream is finished, 0 if more chunks may arrive. Mirrors
    /// `HTTPStream.is_done(self) -> int`（不因上游写完丢尾部——队列排空且
    /// 终结才算 done）。
    pub fn is_done(&self) -> u32 {
        if self.inner.is_finished() {
            1
        } else {
            0
        }
    }

    /// Close/release the stream. Mirrors `HTTPStream.close(self)`.
    /// PLAN-724：真实取消——终结流并 abort 生产者（幂等），不再是占位。
    pub fn close(&self) {
        self.inner.close();
    }
}

// ===========================================================================
// async streaming（Plan 024 形态、PLAN-724 内核化）：consumer loop
// `.next().await` 不阻塞执行线程；状态经 typed 元数据进入 last_status。
// ===========================================================================

/// An async streaming HTTP response backed by the shared kernel stream.
/// Each item is a text `String` (UTF-8 lossy carry；raw 模式单块 ≤16 KiB)。
/// The stream ends when `next()` yields `None`（EOF/错误/取消后）。
pub struct AsyncHTTPStream {
    inner: HttpClientStream,
}

/// Create an async streaming POST request with custom headers.
/// Same `headers` format as `post_stream_with_headers` (newline-separated
/// `"Key: Value"`). 格式错误 = 流以错误态终结（不按空 headers 发射）。
pub async fn post_stream_with_headers_async(
    url: &str,
    body: &str,
    headers: &str,
) -> AsyncHTTPStream {
    let parsed = client::parse_line_headers(headers);
    let headers = match parsed {
        Ok(h) => h,
        Err(_e) => {
            let inner = client::open_stream(StreamSpec {
                method: "POST".into(),
                url: url.to_string(),
                body: None,
                headers: Vec::new(),
                mode: StreamMode::Raw,
            });
            inner.close();
            return AsyncHTTPStream { inner };
        }
    };
    let inner = client::open_stream(StreamSpec {
        method: "POST".into(),
        url: url.to_string(),
        body: Some(body.as_bytes().to_vec()),
        headers,
        mode: StreamMode::Raw,
    });
    AsyncHTTPStream { inner }
}

impl AsyncHTTPStream {
    /// Await the next chunk. Returns `Some(chunk)` for each text piece, or
    /// `None` when the stream is fully read（含错误/取消终结）。
    pub async fn next(&mut self) -> Option<String> {
        if let Some(status) = self.inner.status() {
            set_last_status(status as u32);
        }
        match self.inner.next().await {
            Some(StreamItem::Data(s)) => Some(s),
            _ => None,
        }
    }

    /// 1 if the stream is finished (queue drained + terminal), 0 otherwise.
    pub fn is_done(&self) -> u32 {
        if self.inner.is_finished() {
            1
        } else {
            0
        }
    }

    /// 真取消：终结 + abort 生产者（幂等；Drop 同样回收）。
    pub fn close(&self) {
        self.inner.close();
    }
}

// ===========================================================================
// Plan 349: File download + multipart upload (parity with VM http module).
// 文件 helper 仍走 ureq（PLAN-724 §3 边界，不改返回形状）。
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
