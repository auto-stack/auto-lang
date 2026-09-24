//! PLAN-699 T-03..T-06: Axum/Hyper HTTP/1.1 transport for the VM `#[api]`
//! server (Design 33 阶段 B).
//!
//! Topology (frozen in docs/plans/reports/699-bridge-decision.md):
//!
//! ```text
//! net thread "auto-http-net" (current_thread tokio runtime)      VM owner thread
//!   accept loop → per-conn hyper-util auto::Builder(http1_only)   LocalSet owner loop
//!     bridge_handler: build owned ApiRequest ──bounded mpsc──▶  dispatch_api_request
//!     ◀── oneshot ApiReply (Full / Sse frames / WebSocket 101)   (route/middleware/
//!   FrameStream(+shutdown watch) ◀── Sse producer (spawn_local)   binder/handler)
//! ```
//!
//! All network futures are `Send` and never touch the VM; the `!Send`
//! `AutoVM` stays on its owner thread. No `usize` pointer laundering and no
//! `spawn_blocking` VM calls (AC-02). The old hand-written HTTP/1 parser is
//! out of the default call graph (AC-01); framing, keep-alive and chunked
//! decoding are Hyper's job.
//!
//! Resource bounds (AC-03, values frozen in the decision report):
//! header buffer 64 KiB + ≤100 headers (hyper native 431), header idle
//! timeout 10 s (close), body limit 10 MiB (413), body total timeout 10 s
//! (408), in-flight queue `AUTO_HTTP_MAX_INFLIGHT` (default 64, 503 +
//! `Retry-After` when full), reply wait `AUTO_HTTP_REQUEST_TIMEOUT_MS`
//! (default 30 s, 503), graceful drain 10 s.

use std::net::SocketAddr;
use std::time::Duration;

use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto::Builder as ConnBuilder;
use hyper_util::server::graceful::GracefulShutdown;
use hyper_util::service::TowerToHyperService;

use axum::response::Response;

use super::http_server::{ApiBody, ApiReply, ApiRequest};

// ============================================================================
// Configuration
// ============================================================================

/// Transport budgets. Values frozen in the T-01 decision report; env knobs
/// only where operators need them. Invalid or zero env values fall back to
/// the documented default — there is no implicit unbounded mode.
#[derive(Debug, Clone)]
pub(crate) struct TransportConfig {
    /// `max_buf_size` on hyper's http1 connection: bounds the request head.
    pub max_header_buf: usize,
    /// Idle timeout for reading the request head (hyper closes on expiry).
    pub header_read_timeout: Duration,
    /// Maximum accepted request body (Content-Length or chunked alike).
    pub body_limit: usize,
    /// Total deadline for receiving the full request body (408 on expiry).
    pub body_timeout: Duration,
    /// Bounded owner-queue capacity; 503 when full.
    pub queue_capacity: usize,
    /// Max wait for the VM owner's reply (queue + handler); 503 on expiry.
    pub request_timeout: Duration,
    /// Graceful drain budget before force-closing connections.
    pub shutdown_drain: Duration,
}

impl TransportConfig {
    pub(crate) fn from_env() -> Self {
        let env_usize = |key: &str, default: usize| {
            std::env::var(key)
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|v| *v > 0)
                .unwrap_or(default)
        };
        Self {
            max_header_buf: 64 * 1024,
            header_read_timeout: Duration::from_secs(10),
            body_limit: 10 * 1024 * 1024,
            body_timeout: Duration::from_secs(10),
            queue_capacity: env_usize("AUTO_HTTP_MAX_INFLIGHT", 64),
            request_timeout: Duration::from_millis(env_usize(
                "AUTO_HTTP_REQUEST_TIMEOUT_MS",
                30_000,
            ) as u64),
            shutdown_drain: Duration::from_secs(10),
        }
    }
}

// ============================================================================
// Network thread: accept loop + connection driving
// ============================================================================

/// Drive the accept loop + per-connection hyper serving on the net thread.
/// `ready` reports bind success/failure to the VM owner; `req_tx` is the
/// bounded bridge into the owner loop; `shutdown` flips under graceful
/// shutdown (stop accept → drain with deadline → force close).
pub(crate) async fn serve_network(
    addr: String,
    cfg: TransportConfig,
    req_tx: tokio::sync::mpsc::Sender<(ApiRequest, tokio::sync::oneshot::Sender<ApiReply>)>,
    shutdown: tokio::sync::watch::Receiver<bool>,
    ready: tokio::sync::oneshot::Sender<Result<(), String>>,
) {
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            let _ = ready.send(Err(e.to_string()));
            return;
        }
    };
    let _ = ready.send(Ok(()));

    let graceful = GracefulShutdown::new();
    let mut shutdown = shutdown;
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, peer) = match accepted {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("[HTTP] Accept error: {}", e);
                        continue;
                    }
                };
                let req_tx = req_tx.clone();
                let shutdown = shutdown.clone();
                // Peer is captured per connection — the fallback handler
                // closure is the tower service hyper drives.
                let app = {
                    let cfg = cfg.clone();
                    axum::Router::new().fallback(move |request: axum::extract::Request| {
                        let cfg = cfg.clone();
                        let req_tx = req_tx.clone();
                        let shutdown = shutdown.clone();
                        async move { bridge_handler(request, peer, cfg, req_tx, shutdown).await }
                    })
                };
                let watcher = graceful.watcher();
                tokio::spawn(async move {
                    // http1_only: the transport contract is HTTP/1.1 — the
                    // auto builder would otherwise also serve h2c
                    // prior-knowledge connections, widening scope silently.
                    let mut builder =
                        ConnBuilder::new(TokioExecutor::new()).http1_only();
                    builder
                        .http1()
                        .timer(TokioTimer::new())
                        .max_buf_size(cfg.max_header_buf)
                        .header_read_timeout(cfg.header_read_timeout);
                    let conn = builder.serve_connection_with_upgrades(
                        TokioIo::new(stream),
                        TowerToHyperService::new(app),
                    );
                    let _ = watcher.watch(conn).await;
                });
            }
            _ = shutdown.changed() => {
                // Graceful: stop accepting, let in-flight responses/SSE finish
                // (SSE streams end themselves on the same flag), force close
                // past the drain budget.
                let _ = tokio::time::timeout(cfg.shutdown_drain, graceful.shutdown()).await;
                break;
            }
        }
    }
    drop(listener); // release the port
}

// ============================================================================
// Bridge handler: HTTP protocol objects → owned ApiRequest → ApiReply
// ============================================================================

async fn bridge_handler(
    request: axum::extract::Request,
    peer: SocketAddr,
    cfg: TransportConfig,
    req_tx: tokio::sync::mpsc::Sender<(ApiRequest, tokio::sync::oneshot::Sender<ApiReply>)>,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> Response {
    let (mut parts, body) = request.into_parts();
    let method = parts.method.as_str().to_uppercase();
    let path = parts.uri.to_string();
    let headers: Vec<(String, String)> = parts
        .headers
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Body: bounded incremental collection with a total deadline (AC-03).
    // Content-Length over the limit is rejected before reading (parity with
    // the legacy pre-check); chunked overflow surfaces as LengthLimitError.
    if let Some(cl) = parts.headers.get("content-length") {
        if let Ok(parsed) = cl.to_str().unwrap_or("").parse::<usize>() {
            if parsed > cfg.body_limit {
                eprintln!(
                    "[HTTP] {} {} → 413 (body {} > {})",
                    method, path, parsed, cfg.body_limit
                );
                return error_response(413, "body too large");
            }
        }
    }
    let body_bytes = match tokio::time::timeout(
        cfg.body_timeout,
        axum::body::to_bytes(body, cfg.body_limit),
    )
    .await
    {
        Err(_) => {
            return error_response(408, "request body timed out");
        }
        Ok(Err(e)) => {
            let over_limit = std::error::Error::source(&e)
                .map(|s| s.is::<axum::extract::rejection::LengthLimitError>())
                .unwrap_or(false);
            if over_limit {
                return error_response(413, "body too large");
            }
            return error_response(400, "malformed request body");
        }
        Ok(Ok(b)) => b.to_vec(),
    };

    let api_req = ApiRequest {
        method,
        path,
        headers,
        body: body_bytes,
        peer: Some(peer),
    };

    // Bounded queue: a full queue is a fast, testable 503 (AC-03) — never an
    // unbounded wait or silent allocation growth.
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel::<ApiReply>();
    if req_tx.try_send((api_req, reply_tx)).is_err() {
        eprintln!("[HTTP] {} {} → 503 (in-flight queue full)", peer, "");
        let mut resp = error_response(503, "server busy");
        if let Ok(v) = axum::http::HeaderValue::from_str("1") {
            resp.headers_mut().insert("Retry-After", v);
        }
        return resp;
    }

    // Reply wait timeout releases the queued request relationship; if the
    // owner already started the synchronous handler, the late result is
    // dropped when the receiver is gone — it cannot be preempted (decision
    // report §5.1).
    let reply = match tokio::time::timeout(cfg.request_timeout, reply_rx).await {
        Err(_) => {
            eprintln!("[HTTP] request wait timed out after {:?}", cfg.request_timeout);
            return error_response(503, "request wait timed out");
        }
        Ok(Err(_)) => {
            return error_response(503, "server shutting down");
        }
        Ok(Ok(reply)) => reply,
    };

    api_reply_to_response(reply, parts, shutdown).await
}

/// Map the VM owner's structured reply onto the Axum response world.
async fn api_reply_to_response(
    reply: ApiReply,
    request: axum::http::request::Parts,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> axum::response::Response {
    match reply {
        ApiReply::Full {
            status,
            headers,
            body,
        } => {
            let mut builder = Response::builder().status(
                axum::http::StatusCode::from_u16(status)
                    .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
            );
            {
                let map = builder.headers_mut().expect("fresh builder");
                for (k, v) in &headers {
                    match (
                        axum::http::HeaderName::try_from(k.as_str()),
                        axum::http::HeaderValue::from_str(v),
                    ) {
                        (Ok(name), Ok(value)) => {
                            map.insert(name, value);
                        }
                        _ => {
                            eprintln!("[HTTP] dropping unmappable reply header {k:?}");
                        }
                    }
                }
            }
            match body {
                ApiBody::Text(bytes) => builder
                    .body(axum::body::Body::from(bytes))
                    .unwrap_or_else(|_| error_response(500, "internal error")),
                ApiBody::Sse(frames) => builder
                    .body(axum::body::Body::from_stream(FrameStream {
                        rx: frames,
                        shutdown,
                    }))
                    .unwrap_or_else(|_| error_response(500, "internal error")),
            }
        }
        ApiReply::WebSocket { accept } => {
            // Same simplified echo capability as the legacy path: handshake
            // here, raw text-frame echo on the upgraded IO (not a general
            // WebSocket implementation — Design 33 §7.3).
            let resp = Response::builder()
                .status(axum::http::StatusCode::SWITCHING_PROTOCOLS)
                .header("upgrade", "websocket")
                .header("connection", "Upgrade")
                .header("sec-websocket-accept", accept)
                .body(axum::body::Body::empty())
                .expect("static 101 response");
            let mut req = axum::extract::Request::from_parts(request, axum::body::Body::empty());
            tokio::spawn(async move {
                if let Ok(upgraded) = hyper::upgrade::on(&mut req).await {
                    let mut io = TokioIo::new(upgraded);
                    serve_ws_echo_loop(&mut io).await;
                }
            });
            resp
        }
    }
}

/// Determinate transport-level rejection (pre-dispatch: no VM, no request id
/// — same header shape as the legacy `write_request_error_response`).
fn error_response(status: u16, message: &str) -> axum::response::Response {
    let body = format!("{{\"error\":{}}}", super::http_server::json_escape_string(message));
    let mut builder = axum::response::Response::builder()
        .status(
            axum::http::StatusCode::from_u16(status)
                .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        )
        .header("Content-Type", "application/json");
    {
        let map = builder.headers_mut().expect("fresh builder");
        for (k, v) in super::http_server::cors_header_pairs() {
            if let (Ok(name), Ok(value)) = (
                axum::http::HeaderName::try_from(k.as_str()),
                axum::http::HeaderValue::from_str(&v),
            ) {
                map.insert(name, value);
            }
        }
    }
    builder
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|_| {
            axum::response::Response::builder()
                .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::empty())
                .expect("fallback response")
        })
}

// ============================================================================
// SSE body stream + WebSocket echo
// ============================================================================

/// SSE body adapter: bounded frame channel → HTTP body stream, with correct
/// waker wiring (`poll_recv`) and a shutdown arm so graceful shutdown ends
/// SSE bodies deterministically (the channel receiver drop then reclaims the
/// producer's iterator/task/subscription on the VM owner — 696/698 reuse).
struct FrameStream {
    rx: tokio::sync::mpsc::Receiver<String>,
    shutdown: tokio::sync::watch::Receiver<bool>,
}

impl futures::Stream for FrameStream {
    type Item = Result<String, std::convert::Infallible>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if *self.shutdown.borrow_and_update() {
            return std::task::Poll::Ready(None);
        }
        std::pin::Pin::new(&mut self.rx)
            .poll_recv(cx)
            .map(|opt| opt.map(Ok))
    }
}

/// Raw WebSocket echo over an upgraded IO — same simplified frame loop the
/// legacy inline path ran on the raw TCP socket (text echo / ping-pong /
/// close; client-masked payloads unmasked).
async fn serve_ws_echo_loop(stream: &mut (impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin)) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    loop {
        // Read a WebSocket frame (simplified: text frames only).
        let mut header = [0u8; 2];
        if stream.read_exact(&mut header).await.is_err() {
            break;
        }
        let opcode = header[0] & 0x0F;
        let masked = (header[1] & 0x80) != 0;
        let payload_len = (header[1] & 0x7F) as usize;

        // Extended payload length (16/64 bit).
        let actual_len = if payload_len == 126 {
            let mut ext = [0u8; 2];
            if stream.read_exact(&mut ext).await.is_err() {
                break;
            }
            u16::from_be_bytes(ext) as usize
        } else if payload_len == 127 {
            let mut ext = [0u8; 8];
            if stream.read_exact(&mut ext).await.is_err() {
                break;
            }
            u64::from_be_bytes(ext) as usize
        } else {
            payload_len
        };

        // Masking key (4 bytes if masked).
        let mut mask_key = [0u8; 4];
        if masked {
            if stream.read_exact(&mut mask_key).await.is_err() {
                break;
            }
        }

        // Payload.
        let mut payload = vec![0u8; actual_len];
        if stream.read_exact(&mut payload).await.is_err() {
            break;
        }
        if masked {
            for (i, b) in payload.iter_mut().enumerate() {
                *b ^= mask_key[i % 4];
            }
        }

        // Handle by opcode.
        match opcode {
            0x1 => {
                // Text frame — echo back (unmasked, server→client).
                let text = String::from_utf8_lossy(&payload).to_string();
                let resp = super::http_server::encode_ws_text_frame(&text);
                let _ = stream.write_all(&resp).await;
                let _ = stream.flush().await;
            }
            0x8 => {
                // Close frame.
                break;
            }
            0x9 => {
                // Ping → Pong.
                let pong = [0x8Au8, payload.len() as u8];
                let _ = stream.write_all(&pong).await;
                let _ = stream.write_all(&payload).await;
            }
            _ => {}
        }
        // Cooperative yield for other connections.
        tokio::task::yield_now().await;
    }
}

#[cfg(test)]
mod bridge_tests {
    use super::*;

    /// AC-03: a full in-flight queue rejects fast with 503 + Retry-After —
    /// no unbounded wait, no silent allocation growth. Deterministic unit
    /// probe at the bridge boundary (no VM, no timing race).
    #[tokio::test]
    async fn full_queue_rejects_with_503() {
        let cfg = TransportConfig {
            queue_capacity: 1,
            ..TransportConfig::from_env()
        };
        let (req_tx, _hold) = tokio::sync::mpsc::channel(1);
        // Fill the single queue slot; the receiver is held, not consumed.
        let (_reply_tx, _reply_rx) = tokio::sync::oneshot::channel();
        req_tx
            .send((
                ApiRequest {
                    method: "GET".into(),
                    path: "/api/held".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                    peer: None,
                },
                _reply_tx,
            ))
            .await
            .expect("one slot fits");
        let request = axum::extract::Request::builder()
            .method("GET")
            .uri("/api/next")
            .body(axum::body::Body::empty())
            .unwrap();
        let (_flag_tx, shutdown) = tokio::sync::watch::channel(false);
        let resp = bridge_handler(
            request,
            "127.0.0.1:1234".parse().unwrap(),
            cfg,
            req_tx,
            shutdown,
        )
        .await;
        assert_eq!(resp.status(), 503, "full queue must reject fast");
        assert!(
            resp.headers().get("Retry-After").is_some(),
            "503 carries Retry-After"
        );
    }
}
