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
pub mod server_file;
pub mod transfer;

// PLAN-729 T-02：服务端文件响应公共面再导出（描述符 + 纯协议决策；
// 宿主执行在消费端 auto_lang::http_file_service）。
pub use crate::http::server_file::{
    build_file_headers, decide_file_protocol, evaluate_if_range, evaluate_preconditions,
    extract_request_conditions, file_response, format_http_date, parse_http_date,
    parse_range_header, validate_relative_path, Disposition, FileInitError, FileInitErrorKind,
    FileProtocolDecision, FileRangePlan, FileRequestConditions, FileResponse, FileResponseOptions,
    IfRangeOutcome, PreconditionOutcome, RepresentationValidators,
};

// 内核 typed 类型的一等再导出（原生 Rust 消费者面）。
pub use crate::http::client::{
    ClientError, HttpRequest, HttpResponse, StreamItem, StreamMode, StreamSpec,
};

// PLAN-727：文件传输公共面再导出（VM/a2r/原生 Rust 同词汇自由函数）。
#[doc(hidden)]
pub use crate::http::transfer::{active_transfer_len, in_flight_target_len};
pub use crate::http::transfer::{
    cancel_transfer_by_id, transfer_cancel, transfer_download, transfer_error,
    transfer_next_progress, transfer_upload, transfer_wait, transfer_wait_async,
    transfer_wait_typed, DownloadOptions, FileTransfer, OnExists, TransferError, TransferErrorKind,
    TransferHooks, TransferKind, TransferLimits, TransferObserver, TransferOutcome, TransferPhase,
    TransferProgress, TransferReceipt, UploadMode, UploadOptions, Validator,
};

use crate::http::client::HttpClientStream;
use crate::http::client::HttpResponse as KernelResponse;
// （ClientError/HttpRequest/HttpResponse/StreamItem/StreamMode/StreamSpec
//   已由上方 pub use 覆盖命名空间。）

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
            (
                resp.status as u32,
                String::from_utf8_lossy(&resp.body).into_owned(),
            )
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
            (
                resp.status as u32,
                String::from_utf8_lossy(&resp.body).into_owned(),
            )
        }
        Err(e) => {
            set_last_status(0);
            (0, transport_message(&e))
        }
    }
}

// =============================================================================
// PLAN-724 T-06：普通动词族（Auto http.get/post/put/delete 两参形）。
// 同步形态 = 内核同步桥接（仅同步边界）；async 形态 = 内核 async 面。
// 返回 kernel-backed Response（状态/headers/body typed）。
// =============================================================================

fn verb_request(method: &str, url: &str, body: Option<&str>) -> HttpRequest {
    HttpRequest {
        method: method.to_string(),
        url: url.to_string(),
        headers: Vec::new(),
        body: body.map(|b| b.as_bytes().to_vec()),
        timeout_ms: None,
    }
}

fn verb_response(result: Result<KernelResponse, ClientError>) -> Response {
    RequestBuilder::from_kernel(result)
}

/// Perform a GET request. Mirrors Auto's `http.get(url) -> Response`.
/// 同步边界专用；async 上下文用 [`get_async`]。
pub fn get(url: &str) -> Response {
    verb_response(client::execute_blocking(verb_request("GET", url, None)))
}

/// Perform a POST request (two-arg normal form). Mirrors Auto's
/// `http.post(url, body) -> Response`。与三参认证 [`post_sync`] 按元数分派。
pub fn post(url: &str, body: &str) -> Response {
    verb_response(client::execute_blocking(verb_request(
        "POST",
        url,
        Some(body),
    )))
}

/// Perform a PUT request. Mirrors Auto's `http.put(url, body) -> Response`.
pub fn put(url: &str, body: &str) -> Response {
    verb_response(client::execute_blocking(verb_request(
        "PUT",
        url,
        Some(body),
    )))
}

/// Perform a DELETE request. Mirrors Auto's `http.delete(url) -> Response`.
pub fn delete(url: &str) -> Response {
    verb_response(client::execute_blocking(verb_request("DELETE", url, None)))
}

/// Async GET（内核 async 面；async 上下文的正确入口）。
pub async fn get_async(url: &str) -> Response {
    verb_response(client::execute(verb_request("GET", url, None)).await)
}

/// Async POST（两参普通形）。
pub async fn post_async(url: &str, body: &str) -> Response {
    verb_response(client::execute(verb_request("POST", url, Some(body))).await)
}

/// Async PUT。
pub async fn put_async(url: &str, body: &str) -> Response {
    verb_response(client::execute(verb_request("PUT", url, Some(body))).await)
}

/// Async DELETE。
pub async fn delete_async(url: &str) -> Response {
    verb_response(client::execute(verb_request("DELETE", url, None)).await)
}

/// Legacy `http.get_sync`（历史发射面引用）：返回 (status, body 文本)。
pub fn get_sync(url: &str) -> (u32, String) {
    let resp = get(url);
    (
        resp.status_code(),
        String::from_utf8_lossy(&resp.body_bytes()).into_owned(),
    )
}

/// [`get_sync`] 的 async 面（async 上下文专用）。
pub async fn get_sync_async(url: &str) -> (u32, String) {
    let resp = get_async(url).await;
    (
        resp.status_code(),
        String::from_utf8_lossy(&resp.body_bytes()).into_owned(),
    )
}

/// [`post_sync`] 的 async 面（tuple 形状不变；async 上下文专用）。
pub async fn post_sync_async(url: &str, body: &str, api_key: &str) -> (u32, String) {
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
    match client::execute(req).await {
        Ok(resp) => {
            set_last_status(resp.status as u32);
            (
                resp.status as u32,
                String::from_utf8_lossy(&resp.body).into_owned(),
            )
        }
        Err(e) => {
            set_last_status(0);
            (0, transport_message(&e))
        }
    }
}

/// [`post_bearer_sync`] 的 async 面（tuple 形状不变；async 上下文专用）。
pub async fn post_bearer_sync_async(url: &str, body: &str, api_key: &str) -> (u32, String) {
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
    match client::execute(req).await {
        Ok(resp) => {
            set_last_status(resp.status as u32);
            (
                resp.status as u32,
                String::from_utf8_lossy(&resp.body).into_owned(),
            )
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
    /// PLAN-727 T-06：multipart 描述（(field, path)，**路径描述直到发送**，
    /// 不预读为 Vec）。
    multipart_files: Vec<(String, String)>,
    multipart_texts: Vec<(String, String)>,
    /// 预算越界（builder 期记录；send 终结性失败，坏请求不发出）。
    build_error: Option<String>,
}

/// PLAN-727 T-06：multipart 预算（与 VM 桥同值，见 727-transfer-decision §7）。
const BUILDER_MAX_FILES: usize = 16;
const BUILDER_MAX_TEXT_FIELDS: usize = 64;
const BUILDER_MAX_TEXT_VALUE: usize = 64 * 1024;
const BUILDER_MAX_TEXT_TOTAL: usize = 1024 * 1024;

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
        multipart_files: Vec::new(),
        multipart_texts: Vec::new(),
        build_error: None,
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

    /// Attach a file part（PLAN-727：路径描述到发送，不预读）。
    /// 预算：文件 part ≤ 16；越界 = send 终结性失败。
    pub fn multipart_file(mut self, field: &str, path: &str) -> RequestBuilder {
        if self.multipart_files.len() >= BUILDER_MAX_FILES {
            self.build_error = Some(format!(
                "multipart file parts exceed budget {BUILDER_MAX_FILES}"
            ));
            return self;
        }
        self.multipart_files
            .push((field.to_string(), path.to_string()));
        self
    }

    /// Attach a text field。预算：字段 ≤ 64、单值 ≤ 64KiB、总量 ≤ 1MiB。
    pub fn multipart_text(mut self, field: &str, value: &str) -> RequestBuilder {
        if self.multipart_texts.len() >= BUILDER_MAX_TEXT_FIELDS {
            self.build_error = Some(format!(
                "multipart text fields exceed budget {BUILDER_MAX_TEXT_FIELDS}"
            ));
        } else if value.len() > BUILDER_MAX_TEXT_VALUE {
            self.build_error = Some(format!(
                "multipart text value {} exceeds budget {BUILDER_MAX_TEXT_VALUE}",
                value.len()
            ));
        } else {
            let total: usize = self.multipart_texts.iter().map(|(_, v)| v.len()).sum();
            if total + value.len() > BUILDER_MAX_TEXT_TOTAL {
                self.build_error = Some(format!(
                    "multipart text total {} exceeds budget {BUILDER_MAX_TEXT_TOTAL}",
                    total + value.len()
                ));
            } else {
                self.multipart_texts
                    .push((field.to_string(), value.to_string()));
            }
        }
        self
    }

    fn is_multipart(&self) -> bool {
        !self.multipart_files.is_empty() || !self.multipart_texts.is_empty()
    }

    /// Send the request. Mirrors `RequestBuilder.send(self) -> Response`.
    ///
    /// 同步桥接：走内核同步入口。仅供同步（非 async）上下文使用——async
    /// 上下文须用 [`RequestBuilder::send_async`]（内核 async 路径，不阻塞
    /// 执行线程）。multipart 面（PLAN-727）：发送时开文件流式 part，
    /// 缺失/读失败 = 终结性失败（status 0 Response，不跳过 part）。
    pub fn send(self) -> Response {
        if self.is_multipart() {
            return Self::from_kernel(
                client::kernel_handle().block_on(Self::execute_multipart(self)),
            );
        }
        Self::from_kernel(client::execute_blocking(self.into_request()))
    }

    /// Async send：内核 async 执行（排队/建立/读体全程让出执行线程）。
    /// async fn / tokio runtime 内的正确入口（Plan 024 语义、PLAN-724 实现）。
    pub async fn send_async(self) -> Response {
        if self.is_multipart() {
            return Self::from_kernel(Self::execute_multipart(self).await);
        }
        Self::from_kernel(client::execute(self.into_request()).await)
    }

    /// multipart 执行：共享内核 client + 流式文件 part + 有界响应体。
    async fn execute_multipart(self) -> Result<KernelResponse, ClientError> {
        use futures::StreamExt;
        if let Some(err) = &self.build_error {
            return Err(ClientError::Transport(format!("invalid request: {err}")));
        }
        let file_errors: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();
        let method = reqwest::Method::from_bytes(self.method.to_ascii_uppercase().as_bytes())
            .map_err(|e| ClientError::Transport(format!("invalid method: {e}")))?;
        let mut builder = client::shared_http_client()
            .request(method, &self.url)
            .timeout(
                self.timeout_ms
                    .map(std::time::Duration::from_millis)
                    .unwrap_or(std::time::Duration::from_secs(600)),
            );
        for (k, v) in &self.headers {
            builder = builder.header(k.as_str(), v.as_str());
        }
        let mut form = reqwest::multipart::Form::new();
        for (name, value) in &self.multipart_texts {
            form = form.text(name.clone(), value.clone());
        }
        for (name, path) in &self.multipart_files {
            let part = stream_builder_file_part(path, &file_errors);
            form = form.part(name.clone(), part);
        }
        builder = builder.multipart(form);
        let resp = builder.send().await.map_err(|e| {
            // 文件读错误（in-band body error）优先呈现为类型化消息。
            let errs = file_errors.lock().unwrap().join("; ");
            if errs.is_empty() {
                if e.is_timeout() {
                    ClientError::Timeout
                } else {
                    ClientError::Transport(format!("multipart send: {e}"))
                }
            } else {
                ClientError::Transport(format!("multipart send: {e}; file errors: {errs}"))
            }
        })?;
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp
            .headers()
            .iter()
            .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string())))
            .collect();
        let mut body: Vec<u8> = Vec::new();
        let mut upstream = resp.bytes_stream();
        while let Some(chunk) = upstream.next().await {
            let chunk = chunk.map_err(|e| ClientError::Transport(format!("body read: {e}")))?;
            let cap = client::kernel_response_body_limit();
            if body.len() + chunk.len() > cap {
                return Err(ClientError::BodyTooLarge { limit: cap });
            }
            body.extend_from_slice(&chunk);
        }
        Ok(KernelResponse {
            status,
            headers,
            body,
        })
    }
}

/// 流式文件 part（发送时开文件；缺失/读失败 = 立即出错的流 + 消息登记，
/// 绝不产生空 part 假成功）。
fn stream_builder_file_part(
    path: &str,
    file_errors: &std::sync::Arc<std::sync::Mutex<Vec<String>>>,
) -> reqwest::multipart::Part {
    struct PartStream {
        rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    }
    impl futures::Stream for PartStream {
        type Item = Result<Vec<u8>, std::io::Error>;
        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            match self.rx.poll_recv(cx) {
                std::task::Poll::Ready(Some(v)) => std::task::Poll::Ready(Some(Ok(v))),
                std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        }
    }
    fn erroring_part(msg: String) -> reqwest::multipart::Part {
        let stream = futures::stream::once(async move {
            Err::<Vec<u8>, std::io::Error>(std::io::Error::other(msg))
        });
        reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(stream))
    }
    let len = match std::fs::metadata(path) {
        Ok(m) if m.is_file() => m.len(),
        Ok(_) => {
            let msg = format!("multipart file is not a regular file: {path}");
            file_errors.lock().unwrap().push(msg.clone());
            return erroring_part(msg);
        }
        Err(e) => {
            let msg = format!("multipart file open failed: {path}: {e}");
            file_errors.lock().unwrap().push(msg.clone());
            return erroring_part(msg);
        }
    };
    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(2);
    let errors = std::sync::Arc::clone(file_errors);
    let path_owned = path.to_string();
    client::kernel_handle().spawn(async move {
        let _permit = client::fs_ops().acquire().await.ok();
        let mut file = match tokio::fs::File::open(&path_owned).await {
            Ok(f) => f,
            Err(e) => {
                errors
                    .lock()
                    .unwrap()
                    .push(format!("multipart file open failed: {path_owned}: {e}"));
                return;
            }
        };
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            match file.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).await.is_err() {
                        return;
                    }
                }
                Err(e) => {
                    errors
                        .lock()
                        .unwrap()
                        .push(format!("multipart file read failed: {path_owned}: {e}"));
                    return;
                }
            }
        }
    });
    let filename = std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());
    reqwest::multipart::Part::stream_with_length(reqwest::Body::wrap_stream(PartStream { rx }), len)
        .file_name(filename)
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
/// 内部可变性（Mutex）：消费方法取 `&self`，与 VM 手柄语义一致（let 绑定
/// 无需 mut）；跨线程消费不在支持面（VM 同为 owner 线程拉取）。
pub struct HTTPStream {
    inner: std::sync::Mutex<HttpClientStream>,
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
            return HTTPStream {
                inner: std::sync::Mutex::new(inner),
            };
        }
    };
    let inner = client::open_stream(StreamSpec {
        method: "POST".into(),
        url: url.to_string(),
        body: Some(body.as_bytes().to_vec()),
        headers,
        mode: StreamMode::Raw,
    });
    HTTPStream {
        inner: std::sync::Mutex::new(inner),
    }
}

/// Create a streaming GET request. Mirrors Auto's `http.get_stream(url)`.
pub fn get_stream(url: &str) -> HTTPStream {
    let inner = client::open_stream(StreamSpec {
        method: "GET".into(),
        url: url.to_string(),
        body: None,
        headers: Vec::new(),
        mode: StreamMode::Raw,
    });
    HTTPStream {
        inner: std::sync::Mutex::new(inner),
    }
}

/// Create a streaming POST request. Mirrors Auto's `http.post_stream(url, body)`.
pub fn post_stream(url: &str, body: &str) -> HTTPStream {
    let inner = client::open_stream(StreamSpec {
        method: "POST".into(),
        url: url.to_string(),
        body: Some(body.as_bytes().to_vec()),
        headers: Vec::new(),
        mode: StreamMode::Raw,
    });
    HTTPStream {
        inner: std::sync::Mutex::new(inner),
    }
}

impl HTTPStream {
    /// Read the next chunk from the stream. Returns "" when the stream is
    /// exhausted. Mirrors `HTTPStream.next(self) -> str`.
    ///
    /// 同步桥接边界：阻塞等待仅在同步上下文合法；async 消费请用
    /// [`AsyncHTTPStream`]。
    pub fn next(&self) -> String {
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        match client::kernel_handle().block_on(guard.next()) {
            Some(StreamItem::Data(s)) => s,
            _ => String::new(),
        }
    }
    /// 1 if the stream is finished, 0 if more chunks may arrive. Mirrors
    /// `HTTPStream.is_done(self) -> int`（不因上游写完丢尾部——队列排空且
    /// 终结才算 done）。
    pub fn is_done(&self) -> u32 {
        let guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if guard.is_finished() {
            1
        } else {
            0
        }
    }

    /// Close/release the stream. Mirrors `HTTPStream.close(self)`.
    /// PLAN-724：真实取消——终结流并 abort 生产者（幂等），不再是占位。
    pub fn close(&self) {
        let guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        guard.close();
    }

    /// 终结错误查询（707 同形）：None=未终结；Some(None)=EOF；
    /// Some(Some(msg))=错误/取消（含 queue full 等提交期终结）。
    pub fn terminal_error(&self) -> Option<Option<String>> {
        let guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        guard.terminal_error()
    }
}

// PLAN-724 T-06：流自由函数面（707 手工接口的 a2r 形态；EOF 哨兵 ""）。
/// Read the next chunk（自由函数形）。Mirrors `stream_next(stream) -> str`。
pub fn stream_next(s: &HTTPStream) -> String {
    s.next()
}

/// 1 if finished（自由函数形）。Mirrors `stream_is_done(stream) -> int`。
pub fn stream_is_done(s: &HTTPStream) -> u32 {
    s.is_done()
}

/// Close（自由函数形）。Mirrors `stream_close(stream)`。
pub fn stream_close(s: &HTTPStream) {
    s.close()
}

// ===========================================================================
// async streaming（Plan 024 形态、PLAN-724 内核化）：consumer loop
// `.next().await` 不阻塞执行线程；状态经 typed 元数据进入 last_status。
// ===========================================================================

/// An async streaming HTTP response backed by the shared kernel stream.
/// Each item is a text `String` (UTF-8 lossy carry；raw 模式单块 ≤16 KiB)。
/// The stream ends when `next()` yields `None`（EOF/错误/取消后）。
pub struct AsyncHTTPStream {
    /// tokio Mutex（跨 await 持有守卫，future 保持 Send——.go spawn 兼容）。
    /// Arc 外壳：close 的 kernel-spawn 回退路径需要克隆句柄。
    inner: std::sync::Arc<tokio::sync::Mutex<HttpClientStream>>,
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
            return AsyncHTTPStream {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(inner)),
            };
        }
    };
    let inner = client::open_stream(StreamSpec {
        method: "POST".into(),
        url: url.to_string(),
        body: Some(body.as_bytes().to_vec()),
        headers,
        mode: StreamMode::Raw,
    });
    AsyncHTTPStream {
        inner: std::sync::Arc::new(tokio::sync::Mutex::new(inner)),
    }
}

/// Create an async streaming GET request. Mirrors Auto's `http.get_stream`
/// in async 上下文。
pub async fn get_stream_async(url: &str) -> AsyncHTTPStream {
    let inner = client::open_stream(StreamSpec {
        method: "GET".into(),
        url: url.to_string(),
        body: None,
        headers: Vec::new(),
        mode: StreamMode::Raw,
    });
    AsyncHTTPStream {
        inner: std::sync::Arc::new(tokio::sync::Mutex::new(inner)),
    }
}

/// Create an async streaming POST request. Mirrors Auto's `http.post_stream`
/// in async 上下文。
pub async fn post_stream_async(url: &str, body: &str) -> AsyncHTTPStream {
    let inner = client::open_stream(StreamSpec {
        method: "POST".into(),
        url: url.to_string(),
        body: Some(body.as_bytes().to_vec()),
        headers: Vec::new(),
        mode: StreamMode::Raw,
    });
    AsyncHTTPStream {
        inner: std::sync::Arc::new(tokio::sync::Mutex::new(inner)),
    }
}

impl AsyncHTTPStream {
    /// Await the next chunk. Returns `Some(chunk)` for each text piece, or
    /// `None` when the stream is fully read（含错误/取消终结）。
    pub async fn next(&self) -> Option<String> {
        let mut guard = self.inner.lock().await;
        if let Some(status) = guard.status() {
            set_last_status(status as u32);
        }
        match guard.next().await {
            Some(StreamItem::Data(s)) => Some(s),
            _ => None,
        }
    }

    /// 1 if the stream is finished (queue drained + terminal), 0 otherwise.
    /// async 安全：try_lock 失败（消费中）视为未完成，不阻塞执行线程。
    pub fn is_done(&self) -> u32 {
        match self.inner.try_lock() {
            Ok(guard) => {
                if guard.is_finished() {
                    1
                } else {
                    0
                }
            }
            Err(_) => 0,
        }
    }

    /// 真取消：终结 + abort 生产者（幂等；Drop 同样回收）。
    /// async 安全：try_lock（常规 close-after-loop 路径无竞争）；若消费方
    /// 正持锁等待，把 close 抛到内核 runtime 执行（不阻塞调用线程）。
    pub fn close(&self) {
        match self.inner.try_lock() {
            Ok(guard) => guard.close(),
            Err(_) => {
                let inner = self.inner.clone();
                client::kernel_handle().spawn(async move {
                    inner.lock().await.close();
                });
            }
        }
    }
}

// PLAN-724 T-06：async 上下文的流自由函数面。EOF 哨兵 ""（VM 契约）。
/// Await the next chunk（自由函数形；"" = 结束）。
pub async fn stream_next_async(s: &AsyncHTTPStream) -> String {
    s.next().await.unwrap_or_default()
}

/// 1 if finished（自由函数形）。
pub fn stream_is_done_async(s: &AsyncHTTPStream) -> u32 {
    s.is_done()
}

/// Close（自由函数形）。
pub fn stream_close_async(s: &AsyncHTTPStream) {
    s.close()
}

// ===========================================================================
// Plan 349: File download + multipart upload (parity with VM http module).
// PLAN-727 T-03/T-04：旧三 helper 迁移至共享传输核心 [`transfer`]——
// 增量落盘、staging 原子提交、失败保留原目标、写盘失败不再假成功。
// 返回形状逐字节保留（`-> u32` status/0），差异登记为 legacy adapter：
//   - download：成功 = 收据 kind==success（status 可信）；写盘失败 → 0
//     （旧代码 create 成功后 copy 失败仍返回状态 = 假成功，已纠正）。
//   - download_resume：严格字节对齐续传（offset 必须等于本地长度）；206 需
//     合法 Content-Range；200 = 完整重启（旧代码盲目 append 已纠正）。
//     无 validator：不能证明远端版本一致（legacy 限制，明示不承诺）。
//   - upload：raw POST（旧形状保留；新可移植 multipart 面用 transfer_upload）。
// ============================================================================

/// legacy 三 helper 的共享收据 → (status|0) 映射：
/// 成功 / HTTP 状态失败 → status；传输/文件/options/取消 → 0（可观察失败）。
fn legacy_status_from(receipt: &transfer::TransferReceipt) -> u32 {
    match receipt.kind {
        transfer::TransferOutcome::Success => receipt.status.unwrap_or(0) as u32,
        transfer::TransferOutcome::Failed => receipt.status.unwrap_or(0) as u32,
        transfer::TransferOutcome::Cancelled => 0,
    }
}

fn legacy_set_last_status(receipt: &transfer::TransferReceipt) {
    set_last_status(legacy_status_from(receipt));
}

/// Download a file from `url` and save it to `file_path`.
///
/// Returns the HTTP status code (200 on success, 0 on transport/file error).
/// Mirrors Auto's `http.download(url, file_path) -> int`.
/// PLAN-727：共享传输核心执行——staging 原子提交，任何失败保留原目标。
pub fn download(url: &str, file_path: &str) -> u32 {
    let t = transfer::transfer_download(url, file_path, "{}");
    let receipt = transfer::transfer_wait_typed(&t);
    legacy_set_last_status(&receipt);
    legacy_status_from(&receipt)
}

/// Upload a single file to `url` via raw POST body.
///
/// Returns the HTTP status code. The file contents are sent as the request
/// body with `Content-Type: application/octet-stream`（流式读取；缺失/读
/// 失败 = 0，不再吞错）。
/// Mirrors Auto's `http.upload(url, file_path) -> int`.
pub fn upload(url: &str, file_path: &str) -> u32 {
    let t = transfer::transfer_upload(url, file_path, r#"{"mode":"raw"}"#);
    let receipt = transfer::transfer_wait_typed(&t);
    legacy_set_last_status(&receipt);
    legacy_status_from(&receipt)
}

/// Download with resume support — sends a Range header for `offset` bytes.
///
/// 严格续传：offset 必须等于本地文件长度；206 需合法 Content-Range 且起点
/// 一致；200（上游忽略 Range）= 从零完整重启，绝不盲目 append。文件/范围
/// 错误保留原目标并返回 0。
/// Mirrors Auto's `http.download_resume(url, file_path, offset) -> int`.
pub fn download_resume(url: &str, file_path: &str, offset: u64) -> u32 {
    let options = format!("{{\"offset\":{offset}}}");
    let t = transfer::transfer_download(url, file_path, &options);
    let receipt = transfer::transfer_wait_typed(&t);
    legacy_set_last_status(&receipt);
    legacy_status_from(&receipt)
}
