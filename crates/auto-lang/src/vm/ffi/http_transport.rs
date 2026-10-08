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
    /// PLAN-736 AC-03: connection admission cap（idle/读头/keep-alive/发送/
    /// upgrade 全覆盖）。满载在开始解析前关闭（conn_rejected 计数），不承诺
    /// 此时能回 503。
    pub max_connections: usize,
    /// PLAN-736 T-04: 服务策略面（Host/CORS/可信代理/限速）。None = legacy 面
    /// （既有语义零变化）。
    pub service: Option<std::sync::Arc<crate::http_service_config::ServiceRuntime>>,
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
            shutdown_drain: Duration::from_millis(env_usize(
                "AUTO_HTTP_SHUTDOWN_DRAIN_MS",
                10_000,
            ) as u64),
            max_connections: env_usize("AUTO_HTTP_MAX_CONNECTIONS", 128),
            service: None,
        }
    }

    /// 服务配置驱动（T-04）：预算 + 策略面单源装配（env 不参与）。
    pub(crate) fn from_service_runtime(
        rt: std::sync::Arc<crate::http_service_config::ServiceRuntime>,
    ) -> Self {
        let mut cfg = Self::from_service_config(&rt.config);
        cfg.service = Some(rt);
        cfg
    }

    /// PLAN-736 AC-03: 服务配置驱动的预算（显式面；env 不再参与——单一来源）。
    pub(crate) fn from_service_config(cfg: &crate::http_service_config::HttpServiceConfig) -> Self {
        let l = &cfg.limits;
        Self {
            max_header_buf: l.header_buf_bytes,
            header_read_timeout: Duration::from_millis(l.header_read_timeout_ms),
            body_limit: l.body_limit_bytes,
            body_timeout: Duration::from_millis(l.body_timeout_ms),
            queue_capacity: l.max_inflight_requests,
            request_timeout: Duration::from_millis(l.request_timeout_ms),
            shutdown_drain: Duration::from_millis(l.drain_timeout_ms),
            max_connections: l.max_connections,
            service: None,
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
    req_tx: tokio::sync::mpsc::Sender<(
        ApiRequest,
        tokio::sync::oneshot::Sender<ApiReply>,
        u64, // scope id (PLAN-705 T-05)
    )>,
    shutdown: tokio::sync::watch::Receiver<bool>,
    ready: tokio::sync::oneshot::Sender<Result<String, String>>,
) {
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            let _ = ready.send(Err(e.to_string()));
            return;
        }
    };
    // PLAN-736 AC-02: ready 报真实 bound 地址（port=0 → 内核分配的临时端口），
    // 启动者与 health 身份都以它为准，不用请求方拼接值。
    let bound = listener
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| addr.clone());
    let _ = ready.send(Ok(bound));

    let graceful = GracefulShutdown::new();
    let mut shutdown = shutdown;
    // PLAN-736 AC-03: 连接准入许可——idle/正在读头/keep-alive/发送/upgrade
    // 全生命期持有（task 结束即释放）；满载在开始解析前关闭并计
    // conn_rejected（此时不承诺能回 503）。
    let conn_permits = std::sync::Arc::new(tokio::sync::Semaphore::new(cfg.max_connections));
    static CONN_ID_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
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
                let conn_id = CONN_ID_GEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let permit = match conn_permits.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => {
                        // 满：解析前直接关闭（零业务工作）。
                        crate::http_service_observability::counter_add(
                            crate::http_service_observability::CounterName::ConnRejected,
                            1,
                        );
                        crate::http_service_observability::emit_event(serde_json::json!({
                            "event": "conn_rejected",
                            "reason": "connection_cap",
                            "cap": cfg.max_connections,
                        }));
                        drop(stream);
                        continue;
                    }
                };
                crate::http_service_observability::counter_add(
                    crate::http_service_observability::CounterName::ConnAccepted,
                    1,
                );
                crate::http_service_observability::counter_add(
                    crate::http_service_observability::CounterName::ConnActive,
                    1,
                );
                let req_tx = req_tx.clone();
                let shutdown = shutdown.clone();
                // Peer is captured per connection — the fallback handler
                // closure is the tower service hyper drives.
                let app = {
                    let cfg = cfg.clone();
                    let req_tx = req_tx.clone();
                    let shutdown = shutdown.clone();
                    axum::Router::new().fallback(move |request: axum::extract::Request| {
                        let cfg = cfg.clone();
                        let req_tx = req_tx.clone();
                        let shutdown = shutdown.clone();
                        async move { bridge_handler(request, peer, cfg, req_tx, shutdown, conn_id).await }
                    })
                };
                let watcher = graceful.watcher();
                tokio::spawn(async move {
                    // permitRAII：连接任务结束（正常/断连/RST）即归还。
                    let _permit = permit;
                    // http1_only: the transport contract is HTTP/1.1 — the
                    // auto builder would otherwise also serve h2c
                    // prior-knowledge connections, widening scope silently.
                    let mut builder =
                        ConnBuilder::new(TokioExecutor::new()).http1_only();
                    builder
                        .http1()
                        .timer(TokioTimer::new())
                        .max_buf_size(cfg.max_header_buf)
                        .header_read_timeout(cfg.header_read_timeout)
                        // PLAN-705 T-05/§5.4: 半关闭不判取消——客户端半关
                        // 写端仍读响应；hyper 默认在请求后 FIN 即弃在途
                        // 派发，显式开启 half_close 语义。
                        .half_close(true);
                    let conn = builder.serve_connection_with_upgrades(
                        TokioIo::new(stream),
                        TowerToHyperService::new(app),
                    );
                    // PLAN-705 T-05: 连接任务确证终结（client 关闭/RST/解析
                    // 错——watch 返回即任务终结证据）→ 取消该连接名下全部
                    // 请求 scope（决策报告 §5-3 断连判据）。
                    let _ = watcher.watch(conn).await;
                    super::http_server::cancel_scopes_for_conn(conn_id);
                    crate::http_service_observability::counter_sub(
                        crate::http_service_observability::CounterName::ConnActive,
                        1,
                    );
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

fn reply_status(reply: &ApiReply) -> u16 {
    match reply {
        ApiReply::Full { status, .. } => *status,
        ApiReply::WebSocket { .. } => 101,
    }
}

fn reply_bytes_hint(reply: &ApiReply) -> u64 {
    match reply {
        ApiReply::Full { body, .. } => match body {
            ApiBody::Text(t) => t.len() as u64,
            _ => 0, // SSE/文件以 scope 终态收口，头部阶段不计
        },
        ApiReply::WebSocket { .. } => 0,
    }
}

async fn bridge_handler(
    request: axum::extract::Request,
    peer: SocketAddr,
    cfg: TransportConfig,
    req_tx: tokio::sync::mpsc::Sender<(
        ApiRequest,
        tokio::sync::oneshot::Sender<ApiReply>,
        u64,
    )>,
    shutdown: tokio::sync::watch::Receiver<bool>,
    conn_id: u64,
) -> Response {
    let (mut parts, body) = request.into_parts();
    let method = parts.method.as_str().to_uppercase();
    let path = parts.uri.to_string();
    let headers: Vec<(String, String)> = parts
        .headers
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // PLAN-736 T-05: 服务控制面（/__auto/*）——不经业务 VM/auth（AC-02/05）；
    // snapshot/shutdown 仅 loopback。早于策略链与 body 读取。
    if path.starts_with("/__auto/") {
        if let Some(resp) =
            super::http_server::handle_service_control_path(&method, &path, peer)
        {
            crate::http_service_observability::counter_add(
                crate::http_service_observability::CounterName::RequestsTotal,
                1,
            );
            return resp;
        }
    }

    // PLAN-736 T-04: 服务策略链（仅显式服务面；legacy 面零变化）。全部在
    // body 读取/上传预检之前短路——坏 Host/CORS 拒绝/限速 429 零业务
    // 零 body 工作（AC-04：保护上传不预读/不落盘）。
    let mut cors_attach: Option<(String, bool)> = None;
    if let Some(svc) = &cfg.service {
        let c = &svc.config;
        // (a) Host 允许表（hostname 精确匹配；X-Forwarded-Host 不放宽）。
        let authority = parts
            .headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .or_else(|| parts.uri.authority().map(|a| a.as_str()));
        if !c.host_allowed(authority) {
            crate::http_service_observability::emit_event(serde_json::json!({
                "event": "request_rejected", "reason": "host_not_allowed",
            }));
            return error_response(400, "host not allowed");
        }
        // (b) 单层可信代理身份（peer 精确命中才消费单段 XFF/XFP）。
        let xff = parts.headers.get("x-forwarded-for").and_then(|v| v.to_str().ok());
        let xfp = parts.headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok());
        let (identity, _proto) = c.effective_client_identity(peer.ip(), xff, xfp);
        let _ = identity.via_trusted_proxy;
        // (c) 服务级限速（有界桶 + TTL；满表新身份保守 429）。
        if let Some(rl) = &svc.limiter {
            if let crate::http_service_config::RateDecision::Limited { retry_after_secs } =
                rl.check(identity.client, crate::http_service_config::ServiceRuntime::now_ms())
            {
                let mut resp = error_response(429, "rate limited");
                if let Ok(v) = axum::http::HeaderValue::from_str(&retry_after_secs.to_string()) {
                    resp.headers_mut().insert("Retry-After", v);
                }
                return resp;
            }
        }
        // (d) CORS：preflight（OPTIONS+Origin）短路；实际请求算好附件。
        let origin = parts.headers.get("origin").and_then(|v| v.to_str().ok());
        if method == "OPTIONS" {
            if let Some(origin) = origin {
                let acrm = parts
                    .headers
                    .get("access-control-request-method")
                    .and_then(|v| v.to_str().ok());
                let acrh: Vec<String> = parts
                    .headers
                    .get("access-control-request-headers")
                    .and_then(|v| v.to_str().ok())
                    .map(|h| h.split(',').map(|x| x.trim().to_string()).collect())
                    .unwrap_or_default();
                let (status, acao, vary) = c.cors.preflight(Some(origin), acrm, &acrh);
                let mut resp = error_response(status, "");
                if let Some(acao) = acao {
                    if let Ok(v) = axum::http::HeaderValue::from_str(&acao) {
                        resp.headers_mut().insert("Access-Control-Allow-Origin", v);
                    }
                    resp.headers_mut().insert(
                        "Access-Control-Max-Age",
                        axum::http::HeaderValue::from(c.cors.max_age_seconds),
                    );
                    if vary {
                        resp.headers_mut().insert("Vary", axum::http::HeaderValue::from_static("Origin"));
                    }
                    let methods = c.cors.allowed_methods.join(", ");
                    if let Ok(v) = axum::http::HeaderValue::from_str(&methods) {
                        resp.headers_mut().insert("Access-Control-Allow-Methods", v);
                    }
                    let hdrs = c.cors.allowed_headers.join(", ");
                    if let Ok(v) = axum::http::HeaderValue::from_str(&hdrs) {
                        resp.headers_mut().insert("Access-Control-Allow-Headers", v);
                    }
                }
                return resp;
            }
        }
        if let Some(origin) = origin {
            if let crate::http_service_config::CorsOutcome::Allowed { allow_origin, vary_origin } =
                c.cors.actual_request(Some(origin))
            {
                cors_attach = Some((allow_origin, vary_origin));
            }
        }
    }

    // Body: bounded incremental collection with a total deadline (AC-03).
    // Content-Length over the limit is rejected before reading (parity with
    // the legacy pre-check); chunked overflow surfaces as LengthLimitError.
    //
    // PLAN-730 T-05：上传路由（方法+参数类型双条件，与 owner 同源分类）
    // 的 body **不在桥侧消费**——CL 对上传 wire 预算（默认 65MiB）预检
    // 后，原始流随 ApiRequest 延迟到授权后的 receive（未授权不解析、
    // 不落盘、不 drain）。非上传路由维持 10MiB 全量上限不变。
    let upload_route = (method == "POST" || method == "PUT")
        && super::http_server::upload_route_lookup(&method, &path);
    let (body_bytes, raw_upload_body) = if upload_route {
        let wire_cap = a2r_std::http::UploadServeLimits::from_env().max_wire_bytes;
        if let Some(cl) = parts.headers.get("content-length") {
            if let Ok(parsed) = cl.to_str().unwrap_or("").parse::<usize>() {
                if parsed as u64 > wire_cap {
                    eprintln!(
                        "[HTTP] {} {} → 413 (upload body {} > {})",
                        method, path, parsed, wire_cap
                    );
                    return error_response(413, "body too large");
                }
            }
        }
        (Vec::new(), Some(body))
    } else {
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
        let bytes = match tokio::time::timeout(
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
        (bytes, None)
    };

    let api_req = ApiRequest {
        method,
        path,
        headers,
        body: body_bytes,
        raw_upload_body,
        peer: Some(peer),
    };

    // PLAN-705 T-05: 生命期许可 + scope ——入队前获取（queued+running+
    // parked 总上限）；许可不可得 = 总上限满 → 503（零分配零排队）。
    let scope = super::http_server::create_scope(
        conn_id,
        std::time::Instant::now() + cfg.request_timeout,
    );
    let Some(scope) = scope else {
        eprintln!("[HTTP] {} {} → 503 (lifetime permit cap)", peer, "");
        let mut resp = error_response(503, "server busy");
        if let Ok(v) = axum::http::HeaderValue::from_str("1") {
            resp.headers_mut().insert("Retry-After", v);
        }
        return resp;
    };
    // PLAN-736 T-05/AC-05：请求事件挂 scope（finalize_scope 单点恰一次发射；
    // route v1 = 去 query 路径，T-08 复核参数化模板化）。
    {
        let route_no_query = api_req.path.split('?').next().unwrap_or("").to_string();
        crate::http_service_observability::counter_add(
            crate::http_service_observability::CounterName::RequestsTotal,
            1,
        );
        let mut ev = crate::http_service_observability::PendingRequestEvent::new(
            &api_req.method,
            &route_no_query,
        );
        ev.status = None;
        if let Ok(mut slot) = scope.event.lock() {
            *slot = Some(ev);
        }
    }

    // Bounded queue: a full queue is a fast, testable 503 (AC-03) — never an
    // unbounded wait or silent allocation growth. 队满 = scope 终结（许可
    // 释放），不留下无效排队（AC-04）。
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel::<ApiReply>();
    if req_tx.try_send((api_req, reply_tx, scope.id)).is_err() {
        eprintln!("[HTTP] {} {} → 503 (in-flight queue full)", peer, "");
        super::http_server::cancel_scope(&scope);
        let mut resp = error_response(503, "server busy");
        if let Ok(v) = axum::http::HeaderValue::from_str("1") {
            resp.headers_mut().insert("Retry-After", v);
        }
        return resp;
    }

    // Reply wait：回复 / scope 取消 / deadline 三臂。PLAN-730 T-05：deadline
    // 臂经 watch 重臂——上传 receive 启动时 scope deadline 从 30s 切换到
    // 上传总期限（extend_scope_deadline 发 watch），桥不再保留旧捕获值；
    // 取消臂（含真到期）= scope 幂等终结：owner 侧 parked 等待随后废弃、
    // live-op 回收，已排队请求不再执行业务函数（AC-04）。
    tokio::pin!(reply_rx);
    let mut deadline_rx = scope.deadline_tx.subscribe();
    let reply = loop {
        let current = *deadline_rx.borrow_and_update();
        tokio::select! {
            r = &mut reply_rx => break Some(r),
            _ = scope.cancel_notify.notified() => break None,
            _ = tokio::time::sleep_until(tokio::time::Instant::from_std(current)) => {
                deadline_rx.borrow_and_update();
                if std::time::Instant::now() >= *deadline_rx.borrow() {
                    break None; // 真到期（延展与到期的竞态由重读分辨）
                }
                // deadline 已延展——循环重臂。
            }
            _ = deadline_rx.changed() => {
                // deadline 延展通知——循环重臂。
            }
        }
    };
    let reply = match reply {
        Some(Ok(reply)) => {
            // 回复已定：状态与字节进事件（终态仍由 scope finalize 发射——
            // 流/文件以 body 收口为终态，不 headers 即记）。
            if let Ok(mut slot) = scope.event.lock() {
                if let Some(ev) = slot.as_mut() {
                    ev.status = Some(reply_status(&reply));
                    ev.bytes_sent = reply_bytes_hint(&reply);
                }
            }
            reply
        }
        Some(Err(_)) => {
            super::http_server::cancel_scope(&scope);
            return error_response(503, "server shutting down");
        }
        None => {
            eprintln!(
                "[HTTP] request wait timed out / cancelled after {:?}",
                cfg.request_timeout
            );
            super::http_server::cancel_scope(&scope);
            return error_response(503, "request wait timed out");
        }
    };

    // SSE：许可随 scope 移交响应体（FrameStream Drop 释放——流结束/断连/
    // 关闭均触发）；普通回复立即终结 scope。
    match &reply {
        // SSE/文件体的许可随 scope 移交响应体代持；其余回复立即终结 scope。
        ApiReply::Full { body: ApiBody::Sse(_), .. } => {}
        ApiReply::Full { body: ApiBody::File(_), .. } => {}
        _ => super::http_server::complete_scope(&scope),
    }
    let response = api_reply_to_response(reply, parts, shutdown, Some(scope)).await;
    // PLAN-736 T-04: 实际请求（携 Origin 且命中）的 CORS 附件（桥侧短路路径
    // 不虚构 CORS 头——只有业务回复面附加）。
    if let Some((acao, vary)) = cors_attach {
        let (mut parts, body) = response.into_parts();
        if let Ok(v) = axum::http::HeaderValue::from_str(&acao) {
            parts.headers.insert("Access-Control-Allow-Origin", v);
        }
        if vary {
            parts
                .headers
                .insert("Vary", axum::http::HeaderValue::from_static("Origin"));
        }
        return axum::response::Response::from_parts(parts, body);
    }
    response
}

/// PLAN-729 T-04：文件 seed 的宿主 serve 结果。`finalize_scope` = 无 body
/// 回复（HEAD/304/412/416/错误）待终结的 scope；流回复的 scope 已随
/// `FileBodyAdapter` 代持。
struct FileReplyOutcome {
    reply: crate::http_file_service::FileReply,
    scope: Option<std::sync::Arc<super::http_server::RequestScope>>,
    finalize_scope: Option<std::sync::Arc<super::http_server::RequestScope>>,
}

/// 文件 seed → 宿主 serve。准备期限 = min(30s, scope 剩余)；finish hook =
/// scope 幂等终结（body 收口恰一次）。
async fn serve_file_seed(
    seed: super::http_server::FileReplySeed,
    request: axum::http::request::Parts,
    scope: Option<std::sync::Arc<super::http_server::RequestScope>>,
) -> FileReplyOutcome {
    use crate::http_file_service::{serve_file_response, ServeFileRequest};
    let method = request.method.as_str().to_string();
    let headers: Vec<(String, String)> = request
        .headers
        .iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|val| (k.as_str().to_string(), val.to_string()))
        })
        .collect();
    let limits = crate::http_file_service::file_serve_limits();
    let now = std::time::Instant::now();
    let cap = now + limits.prepare_timeout;
    let prepare_deadline = match &scope {
        Some(s) if *s.deadline.read().unwrap() < cap => *s.deadline.read().unwrap(),
        _ => cap,
    };
    let hook_scope = scope.clone();
    let finish_hook = hook_scope.map(|s| {
        let hook: crate::http_file_service::FileFinishHook =
            std::sync::Arc::new(move |_finish: &crate::http_file_service::FileFinish| {
                super::http_server::complete_scope(&s);
            });
        hook
    });
    let reply = serve_file_response(
        &seed.descriptor,
        ServeFileRequest {
            method: &method,
            request_headers: &headers,
            prepare_deadline,
            finish_hook,
            idle_timeout: None,
        },
    )
    .await;
    let is_stream = matches!(
        reply.body,
        crate::http_file_service::FileReplyBody::Stream(_)
    );
    FileReplyOutcome {
        reply,
        // 流回复：scope 随适配器代持；无 body 回复：带回终结。
        scope: if is_stream { scope.clone() } else { None },
        finalize_scope: if is_stream { None } else { scope },
    }
}

/// PLAN-729 T-04：文件 body 适配器——宿主流 + scope 代持（EOF/Drop/断连/
/// watchdog 收口恰一次，SSE FrameStream 同形）。
struct FileBodyAdapter {
    inner: crate::http_file_service::FileBodyStream,
    scope: Option<std::sync::Arc<super::http_server::RequestScope>>,
}

impl Drop for FileBodyAdapter {
    fn drop(&mut self) {
        if let Some(scope) = self.scope.take() {
            super::http_server::complete_scope(&scope);
        }
    }
}

impl futures::Stream for FileBodyAdapter {
    type Item = Result<Vec<u8>, std::io::Error>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Map the VM owner's structured reply onto the Axum response world.
async fn api_reply_to_response(
    reply: ApiReply,
    request: axum::http::request::Parts,
    shutdown: tokio::sync::watch::Receiver<bool>,
    scope: Option<std::sync::Arc<super::http_server::RequestScope>>,
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
                        scope,
                    }))
                    .unwrap_or_else(|_| error_response(500, "internal error")),
                // PLAN-729 T-04：文件响应——宿主 serve（准入/打开/协议决策/
                // 有界 body）在此 async 完成，不在 VM owner。scope 随 body
                // 移交（pump 收口恰一次）；无 body 回复（HEAD/304/412/416/
                // 错误）由本臂直接终结 scope。
                ApiBody::File(seed) => {
                    let outcome = serve_file_seed(seed, request, scope).await;
                    let mut builder = Response::builder().status(
                        axum::http::StatusCode::from_u16(outcome.reply.status)
                            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
                    );
                    {
                        let map = builder.headers_mut().expect("fresh builder");
                        for (k, v) in &outcome.reply.headers {
                            if let (Ok(name), Ok(value)) = (
                                axum::http::HeaderName::try_from(k.as_str()),
                                axum::http::HeaderValue::from_str(v),
                            ) {
                                map.insert(name, value);
                            }
                        }
                        // CORS 追加（与其他回复形态一致）。
                        for (k, v) in super::http_server::cors_header_pairs() {
                            if let (Ok(name), Ok(value)) = (
                                axum::http::HeaderName::try_from(k.as_str()),
                                axum::http::HeaderValue::from_str(&v),
                            ) {
                                map.insert(name, value);
                            }
                        }
                    }
                    let mut resp = match outcome.reply.body {
                        crate::http_file_service::FileReplyBody::None => builder
                            .body(axum::body::Body::empty())
                            .unwrap_or_else(|_| error_response(500, "internal error")),
                        crate::http_file_service::FileReplyBody::Inline(bytes) => builder
                            .body(axum::body::Body::from(bytes))
                            .unwrap_or_else(|_| error_response(500, "internal error")),
                        crate::http_file_service::FileReplyBody::Stream(stream) => builder
                            .body(axum::body::Body::from_stream(FileBodyAdapter {
                                inner: stream,
                                scope: outcome.scope,
                            }))
                            .unwrap_or_else(|_| error_response(500, "internal error")),
                    };
                    if let Some(scope) = outcome.finalize_scope {
                        super::http_server::complete_scope(&scope);
                    }
                    resp
                }
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
    /// PLAN-705 T-05: SSE 的生命期许可随响应体代持——流结束/断连/关闭
    /// 触发 Drop → scope 幂等终结（许可释放）。
    scope: Option<std::sync::Arc<super::http_server::RequestScope>>,
}

impl Drop for FrameStream {
    fn drop(&mut self) {
        if let Some(scope) = self.scope.take() {
            super::http_server::complete_scope(&scope);
        }
    }
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
                    raw_upload_body: None,
                    peer: None,
                },
                _reply_tx,
                1,
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
            77,
        )
        .await;
        assert_eq!(resp.status(), 503, "full queue must reject fast");
        assert!(
            resp.headers().get("Retry-After").is_some(),
            "503 carries Retry-After"
        );
    }
}
