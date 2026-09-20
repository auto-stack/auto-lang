//! PLAN-658: 单进程多后端宿主（back proxy）。
//!
//! 画廊内嵌 fullstack demo 的"后端半身"：每个 app 的 back 链以独立 VM
//! session 在宿主进程内运行，HTTP 请求按子 URL 前缀（`/apps/<id>/api/*`）
//! 路由到对应 session 内 `#[api]` fn 动态分发——merged CALL 语义的 HTTP 化
//! （设计定案见 docs/plans/658-uigallery-multi-backend-proxy.md §5.2）。
//!
//! 线程模型（`autovm_daemon.rs` 两线程先例）：listener 线程收发 HTTP；
//! 每 session 一条专线程独占其 AutoVM（AutoVM 线程亲和约定——代码库一致
//! 按"实际只在一个线程上碰 VM"保证安全），请求经 mpsc 投递。
//!
//! 路由表来源：每 session 装载 back 链时从 Codegen 收集的 `api_routes`
//! （Plan 312）——**不走**进程级全局 `HTTP_ROUTES` 单表（覆盖式语义，
//! 多 session 无法共用）。参数绑定按名（路径参数按占位符名、body/query
//! 按参数名），与 Rust 生成器的 serde 字段映射语义一致（§5.7-3）。
//!
//! 模块位置：crate 根（与 `autovm_daemon` Plan 269 同层）——纯 VM 基建
//! 零 ui 依赖，un-gated 保证 `cargo th`（只开 test-http-e2e）可测。
//!
//! PLAN-037: 承载层自画廊"boot 期一次性全量注册"扩展为**运行期增删**
//! （`RunningProxy::add_session/add_native_media/remove_app/base_url_for`）
//! ——桌面宿主 launch 时按需装载、窗关卸载（生命周期翻转，路由/SSE/原生
//! media 语义零变化；见 docs/specs/auto-lang/vm/back-proxy.md 增补节）。

use std::collections::HashMap;
use std::io::{BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::Arc;

use crate::ast::Stmt;
use crate::vm::engine::AutoVM;
use crate::vm::ffi::http_server::{match_route, HttpRoute};
use crate::vm::loader::Linker;
use crate::vm::task::AutoTask;
use crate::vm::virt_memory::VirtualFlash;

/// Default listening port (`AUTO_GALLERY_PROXY_PORT` can override; binding conflicts fall back to +1..+10,
/// mirroring the MCP 9247 precedent).
const DEFAULT_PROXY_PORT: u16 = 3358;
/// Single source is `ui::handler_codegen::MODULE_INIT_FN` (behind the `ui` gate); this module is
/// un-gated, so a local constant with the same name anchors it (change them together).
const MODULE_INIT_FN: &str = "__module_init";
/// 单请求投递到 session 的超时（含 VM 执行）；超时按 502 返回。
const REQUEST_TIMEOUT_SECS: u64 = 30;
/// 请求体大小上限。
const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

/// 一个 app 后端的装载描述。
#[derive(Debug, Clone, PartialEq)]
pub struct SessionSpec {
    /// app 标识（子 URL 前缀段，如 `020-music-player`）。
    pub app_id: String,
    /// back 链入口文件（如 `examples/ui/020-music-player/src/back/api.at`）。
    pub back_entry: std::path::PathBuf,
}

/// PLAN-658 T-03: 宿主原生 media 路由注册（生成器对全部后端无条件发射的
/// `/api/media/scan` + `/api/media/stream/:id`，§5.7-3——不在 .at 语料，
/// 由宿主 Rust 直答）。020 形态：无 session，仅原生路由。
#[cfg(feature = "ui")]
#[derive(Debug, Clone, PartialEq)]
pub struct NativeMediaApp {
    pub app_id: String,
    /// pac.at `media_root`（None → resolve_root(None) 语义：env → 平台默认）。
    pub media_root: Option<String>,
}

/// proxy 启动配置。
#[derive(Debug, Clone, Default)]
pub struct BackProxyConfig {
    /// 期望端口（0 = 由 `AUTO_GALLERY_PROXY_PORT`/默认值决定；绑定冲突
    /// 自动回退）。
    pub port: u16,
    pub sessions: Vec<SessionSpec>,
    /// 原生 media 路由（cfg ui；无 root 的 app 也注册——诚实空列表语义
    /// 与生成器一致）。
    #[cfg(feature = "ui")]
    pub native_media: Vec<NativeMediaApp>,
}

/// 运行中的 proxy 句柄。drop 不自动停机（宿主进程生命周期即 proxy 生命周期）。
pub struct RunningProxy {
    /// 实际绑定端口（emit 侧绝对 URL 与 AUTO_HTTP_BASE 用它）。
    pub port: u16,
    shared: Arc<ProxyShared>,
}

/// PLAN-037 T-01: 一个已装载 session 的通道 + 线程句柄（remove_app 取出
/// JoinHandle 供测试 join 断言线程退出；生产侧直接 drop = 分离线程）。
struct SessionHandle {
    tx: mpsc::Sender<ProxyRequest>,
    join: Option<std::thread::JoinHandle<()>>,
}

struct ProxyShared {
    /// app_id → session 请求通道。PLAN-037 T-01: listener 连接线程与宿主
    /// 线程（运行期 add/remove）双写，Mutex 化（658 boot 期单写者免锁形态
    /// 随桌面按需装载退役）。
    sessions: std::sync::Mutex<HashMap<String, SessionHandle>>,
    /// PLAN-658 T-03: 宿主原生 media 路由状态（cfg ui）。PLAN-037 T-01:
    /// 同上 Mutex 化（add_native_media/remove_app 运行期写）。
    #[cfg(feature = "ui")]
    native_media: std::sync::Mutex<HashMap<String, NativeMediaState>>,
}

/// PLAN-658 T-03: 一个 app 的原生 media 服务面（惰性索引 + 绝对 URL base）。
#[cfg(feature = "ui")]
struct NativeMediaState {
    app_id: String,
    /// 已解析根（None = 未配置 → 诚实空列表）。
    root: Option<std::path::PathBuf>,
    /// `http://127.0.0.1:<port>`——scan 响应 url 字段发绝对值（画廊内嵌
    /// 前端无 env 展开机制，§5.3）。
    base: String,
    /// 进程内一次性索引（镜像生成器 OnceLock<MEDIA_INDEX> 语义）。
    index: std::sync::Mutex<Option<crate::ui::media_service::MediaIndex>>,
}

struct ProxyRequest {
    method: String,
    /// 已剥去 `/apps/<id>` 前缀的路径（如 `/api/player/status`）。
    path: String,
    query: String,
    /// 原始请求头（小写键名）。T-04 SSE 面消费（Accept/Last-Event-ID）。
    #[allow(dead_code)]
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    reply: mpsc::Sender<ProxyReply>,
}

/// session → listener 的应答。T-01 仅 JSON 面；stream（SSE）在 T-04 扩展。
enum ProxyReply {
    Response {
        status: u16,
        content_type: &'static str,
        body: Vec<u8>,
    },
    /// 大体量字节流（media 文件窗口）——头 + 定长 reader，逐块写出不整读。
    #[cfg(feature = "ui")]
    Stream {
        status: u16,
        content_type: String,
        extra_headers: Vec<(String, String)>,
        content_length: u64,
        reader: Box<dyn std::io::Read + Send>,
    },
    /// PLAN-658 T-04: ~Stream 端点的 SSE 订阅——listener 写响应头后逐帧
    /// 转发（`data: <json>\n\n`），连接断开即退订（recv 失败清订阅）。
    Sse(std::sync::mpsc::Receiver<String>),
}

impl ProxyReply {
    fn json(status: u16, body: String) -> Self {
        ProxyReply::Response {
            status,
            content_type: "application/json",
            body: body.into_bytes(),
        }
    }
}

/// 启动 back proxy：绑定端口、装载全部 session、进入 accept 循环
/// （listener 线程后台运行）。
pub fn start(config: BackProxyConfig) -> std::io::Result<RunningProxy> {
    let want_port = if config.port != 0 {
        config.port
    } else {
        std::env::var("AUTO_GALLERY_PROXY_PORT")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(DEFAULT_PROXY_PORT)
    };
    let (listener, port) = bind_with_fallback(want_port)?;

    let mut sessions = HashMap::new();
    for spec in &config.sessions {
        let (tx, rx) = mpsc::channel::<ProxyRequest>();
        let app_id = spec.app_id.clone();
        let entry = spec.back_entry.clone();
        let spawn_name = format!("back-proxy-session:{app_id}");
        let spawn_app_id = app_id.clone();
        let join = std::thread::Builder::new()
            .name(spawn_name)
            .stack_size(16 * 1024 * 1024)
            .spawn(move || session_main(spawn_app_id, entry, rx))
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("spawn session thread for {app_id} failed: {e}"),
                )
            })?;
        sessions.insert(app_id, SessionHandle { tx, join: Some(join) });
    }

    let shared = Arc::new(ProxyShared {
        sessions: std::sync::Mutex::new(sessions),
        #[cfg(feature = "ui")]
        native_media: std::sync::Mutex::new({
            let base = format!("http://127.0.0.1:{port}");
            let mut map = HashMap::new();
            for app in &config.native_media {
                let root = crate::ui::media_service::resolve_root(app.media_root.as_deref());
                map.insert(
                    app.app_id.clone(),
                    NativeMediaState {
                        app_id: app.app_id.clone(),
                        root,
                        base: base.clone(),
                        index: std::sync::Mutex::new(None),
                    },
                );
            }
            map
        }),
    });
    let listener_shared = shared.clone();
    std::thread::Builder::new()
        .name("back-proxy-listener".to_string())
        .spawn(move || listener_main(listener, listener_shared))?;

    Ok(RunningProxy { port, shared })
}

fn bind_with_fallback(want: u16) -> std::io::Result<(TcpListener, u16)> {
    let mut last_err = None;
    for off in 0..10u32 {
        let port = want as u32 + off;
        if port > u16::MAX as u32 {
            break;
        }
        match TcpListener::bind(("127.0.0.1", port as u16)) {
            Ok(l) => return Ok((l, port as u16)),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                log::warn!("[back-proxy] port {port} busy, trying next");
                last_err = Some(e);
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::AddrInUse, "no free port in fallback range")
    }))
}

impl RunningProxy {
    /// app 是否已注册 session（020 这类仅宿主原生路由的 app 无 session）。
    pub fn has_session(&self, app_id: &str) -> bool {
        self.shared.sessions.lock().unwrap().contains_key(app_id)
    }

    /// PLAN-037 T-01: 运行期装载一个 app 的 back 链为 VM session（桌面
    /// launch 时按需供给；spawn 形态与 start 同款——16MB 栈 + 线程名）。
    /// 同 app_id 重复 add = 幂等拒绝（双窗共享由调用侧引用计数保证，
    /// 装载面单例）。
    pub fn add_session(&self, spec: SessionSpec) -> std::io::Result<()> {
        let mut sessions = self.shared.sessions.lock().unwrap();
        if sessions.contains_key(&spec.app_id) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("back-proxy: session for `{}` already registered", spec.app_id),
            ));
        }
        let (tx, rx) = mpsc::channel::<ProxyRequest>();
        let spawn_app_id = spec.app_id.clone();
        let entry = spec.back_entry.clone();
        let join = std::thread::Builder::new()
            .name(format!("back-proxy-session:{}", spec.app_id))
            .stack_size(16 * 1024 * 1024)
            .spawn(move || session_main(spawn_app_id, entry, rx))
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("spawn session thread for {} failed: {e}", spec.app_id),
                )
            })?;
        sessions.insert(spec.app_id.clone(), SessionHandle { tx, join: Some(join) });
        Ok(())
    }

    /// PLAN-037 T-01: 运行期注册宿主原生 media 路由（resolve_root 语义
    /// 同 start——None → env → 平台默认）。同 id 重复 add 覆盖旧态（索引
    /// 一并丢弃重建）。
    #[cfg(feature = "ui")]
    pub fn add_native_media(&self, app: NativeMediaApp) {
        let base = format!("http://127.0.0.1:{}", self.port);
        let root = crate::ui::media_service::resolve_root(app.media_root.as_deref());
        let app_id = app.app_id.clone();
        self.shared.native_media.lock().unwrap().insert(
            app_id,
            NativeMediaState {
                app_id: app.app_id,
                root,
                base,
                index: std::sync::Mutex::new(None),
            },
        );
    }

    /// PLAN-037 T-01: 运行期卸载一个 app 的全部供给面（session 表项 +
    /// 原生 media 路由；摘表 drop sender → session 线程 `for req in rx`
    /// 自然退出）。返回 session 线程 JoinHandle 供测试 join 断言退出；
    /// 无 session（仅原生 media 的 020 形态）返回 None，生产侧直接 drop。
    pub fn remove_app(&self, app_id: &str) -> Option<std::thread::JoinHandle<()>> {
        let join = self
            .shared
            .sessions
            .lock()
            .unwrap()
            .remove(app_id)
            .and_then(|mut handle| handle.join.take());
        #[cfg(feature = "ui")]
        self.shared.native_media.lock().unwrap().remove(app_id);
        join
    }

    /// PLAN-037 T-01: 前缀化 root 唯一供给源——`http://127.0.0.1:{port}/
    /// apps/{id}`（桌面 launch 期对 spec.code 内存态改写的 base）。
    pub fn base_url_for(&self, app_id: &str) -> String {
        format!("http://127.0.0.1:{}/apps/{app_id}", self.port)
    }
}

// ---------------------------------------------------------------------------
// HTTP 前端（listener 线程）
// ---------------------------------------------------------------------------

fn listener_main(listener: TcpListener, shared: Arc<ProxyShared>) {
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let shared = shared.clone();
        std::thread::Builder::new()
            .name("back-proxy-conn".to_string())
            .spawn(move || {
                let _ = serve_connection(stream, &shared);
            })
            .ok();
    }
}

struct ParsedRequest {
    method: String,
    path: String,
    query: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn serve_connection(mut stream: TcpStream, shared: &ProxyShared) -> std::io::Result<()> {
    let req = match read_request(&mut stream)? {
        Some(r) => r,
        None => return Ok(()),
    };

    // 子前缀路由：/apps/<app_id>/<rest>
    let reply = route_request(shared, &req);
    write_response(&mut stream, reply)?;
    Ok(())
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<Option<ParsedRequest>> {
    let mut reader = std::io::BufReader::new(stream);
    let mut request_line = String::new();
    let n = reader.read_line(&mut request_line)?;
    if n == 0 {
        return Ok(None);
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("").to_string();
    if method.is_empty() || target.is_empty() {
        return Ok(None);
    }
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target, String::new()),
    };

    let mut headers = Vec::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim().to_ascii_lowercase();
            let v = v.trim().to_string();
            if k == "content-length" {
                content_length = v.parse().unwrap_or(0);
            }
            headers.push((k, v));
        }
    }
    if content_length > MAX_BODY_BYTES {
        return Ok(None);
    }
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    Ok(Some(ParsedRequest { method, path, query, headers, body }))
}

fn route_request(shared: &ProxyShared, req: &ParsedRequest) -> ProxyReply {
    // PLAN-658 T-06: 宿主观测面——会话日志环调试路由（只读）。
    if req.path == "/__backproxy/log" {
        let app = req
            .query
            .split('&')
            .find_map(|p| p.strip_prefix("app="))
            .map(|v| url_decode(v))
            .unwrap_or_default();
        let body = if app.is_empty() {
            let logs = SESSION_LOGS.lock().unwrap();
            serde_json::json!({
                "apps": logs.keys().cloned().collect::<Vec<_>>(),
            })
            .to_string()
        } else {
            let logs = SESSION_LOGS.lock().unwrap();
            let lines: Vec<String> = logs
                .get(&app)
                .map(|r| r.0.iter().cloned().collect())
                .unwrap_or_default();
            serde_json::json!({ "app": app, "lines": lines }).to_string()
        };
        return ProxyReply::json(200, body);
    }
    let Some(rest) = req.path.strip_prefix("/apps/") else {
        return ProxyReply::json(404, error_json("back-proxy: path must start with /apps/<id>/"));
    };
    let (app_id, sub_path) = match rest.split_once('/') {
        Some((id, rest)) => (id.to_string(), format!("/{rest}")),
        None => {
            return ProxyReply::json(
                404,
                error_json("back-proxy: path must start with /apps/<id>/"),
            )
        }
    };
    // PLAN-658 T-03: 宿主原生 media 路由先于 session 分发（020 无 session）。
    #[cfg(feature = "ui")]
    if let Some(reply) = shared.try_native_media(&app_id, &sub_path, req) {
        return reply;
    }
    // PLAN-658 T-05: 031 族图片字节路由（image_pipeline 共享 registry 的
    // 不透明 `/api/__auto/media/{id}/{rev}` URI，session 内 auto.image 原生
    // 创建的条目同进程可读——字节保真 + ETag/304 语义直答）。
    #[cfg(feature = "ui-iced")]
    if sub_path.starts_with("/api/__auto/media/") {
        let if_none = req
            .headers
            .iter()
            .find(|(k, _)| k == "if-none-match")
            .map(|(_, v)| v.clone());
        if let Some(resp) = crate::vm::ffi::stdlib::media_response_for_vm_request(
            &req.method,
            &sub_path,
            if_none.as_deref(),
        ) {
            let mut content_type = "application/octet-stream".to_string();
            let mut extra_headers = Vec::new();
            for (k, v) in &resp.headers {
                if k.eq_ignore_ascii_case("content-type") {
                    content_type = v.clone();
                } else {
                    extra_headers.push((k.clone(), v.clone()));
                }
            }
            let body = resp.body.unwrap_or_else(|| Arc::from(Vec::new()));
            return ProxyReply::Stream {
                status: resp.status,
                content_type,
                extra_headers,
                content_length: body.len() as u64,
                reader: Box::new(std::io::Cursor::new(body)),
            };
        }
    }
    // PLAN-037 T-01: sessions 已 Mutex 化——锁内克隆 sender 即放（send
    // 非阻塞，reply 等待不持锁）。
    let tx = shared
        .sessions
        .lock()
        .unwrap()
        .get(&app_id)
        .map(|handle| handle.tx.clone());
    let Some(tx) = tx else {
        return ProxyReply::json(404, error_json(&format!("back-proxy: unknown app `{app_id}`")));
    };
    let (reply_tx, reply_rx) = mpsc::channel::<ProxyReply>();
    let sent = tx.send(ProxyRequest {
        method: req.method.clone(),
        path: sub_path,
        query: req.query.clone(),
        headers: req.headers.clone(),
        body: req.body.clone(),
        reply: reply_tx,
    });
    if sent.is_err() {
        return ProxyReply::json(
            503,
            error_json(&format!("back-proxy: session for `{app_id}` is gone")),
        );
    }
    match reply_rx.recv_timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS)) {
        Ok(reply) => reply,
        Err(_) => ProxyReply::json(
            502,
            error_json(&format!("back-proxy: session for `{app_id}` timed out")),
        ),
    }
}

fn write_response(stream: &mut TcpStream, reply: ProxyReply) -> std::io::Result<()> {
    // PLAN-658 T-04: SSE 长连接——头后逐帧 `data: <json>\n\n`，断写即退订。
    let reply = match reply {
        ProxyReply::Sse(rx) => {
            let head =
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n";
            stream.write_all(head.as_bytes())?;
            stream.flush()?;
            for msg in rx {
                let frame = format!("data: {msg}\n\n");
                if stream.write_all(frame.as_bytes()).is_err() || stream.flush().is_err() {
                    break;
                }
            }
            return Ok(());
        }
        other => other,
    };
    let (status, content_type, extra_headers, body_source) = match reply {
        ProxyReply::Response { status, content_type, body } => {
            (status, content_type.to_string(), Vec::<(String, String)>::new(), BodySource::Bytes(body))
        }
        ProxyReply::Sse(_) => unreachable!("Sse handled in the outer match"),
        #[cfg(feature = "ui")]
        ProxyReply::Stream { status, content_type, extra_headers, content_length, reader } => (
            status,
            content_type,
            extra_headers,
            BodySource::Reader { content_length, reader },
        ),
    };
    let reason = match status {
        200 => "OK",
        206 => "Partial Content",
        400 => "Bad Request",
        404 => "Not Found",
        416 => "Range Not Satisfiable",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    };
    let mut head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nConnection: close\r\n"
    );
    for (k, v) in &extra_headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    match body_source {
        BodySource::Bytes(body) => {
            head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
            stream.write_all(head.as_bytes())?;
            stream.write_all(&body)?;
        }
        BodySource::Reader { content_length, mut reader } => {
            head.push_str(&format!("Content-Length: {content_length}\r\n\r\n"));
            stream.write_all(head.as_bytes())?;
            let mut buf = [0u8; 64 * 1024];
            let mut remaining = content_length;
            while remaining > 0 {
                let want = buf.len().min(remaining as usize);
                let n = reader.read(&mut buf[..want])?;
                if n == 0 {
                    break;
                }
                stream.write_all(&buf[..n])?;
                remaining -= n as u64;
            }
        }
    }
    stream.flush()
}

enum BodySource {
    Bytes(Vec<u8>),
    #[allow(dead_code)]
    Reader {
        content_length: u64,
        reader: Box<dyn std::io::Read + Send>,
    },
}

fn error_json(msg: &str) -> String {
    format!("{{\"error\":{}}}", serde_json::json!(msg))
}

// ---------------------------------------------------------------------------
// 宿主原生 media 路由（PLAN-658 T-03，cfg ui）
// ---------------------------------------------------------------------------

#[cfg(feature = "ui")]
impl ProxyShared {
    /// `/api/media/scan` + `/api/media/stream/:id` 直答。未命中返回 None
    /// （回落 session 分发）。PLAN-037 T-01: 表已 Mutex 化——命中即持锁
    /// 服务（per-app 粒度；media_stream 的文件 IO 同款既有 index 锁语义）。
    fn try_native_media(
        &self,
        app_id: &str,
        sub_path: &str,
        req: &ParsedRequest,
    ) -> Option<ProxyReply> {
        let media = self.native_media.lock().unwrap();
        let state = media.get(app_id)?;
        if sub_path == "/api/media/scan" && req.method == "GET" {
            return Some(Self::media_scan(state));
        }
        if let Some(id) = sub_path
            .strip_prefix("/api/media/stream/")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
        {
            if req.method == "GET" || req.method == "HEAD" {
                let range = req
                    .headers
                    .iter()
                    .find(|(k, _)| k == "range")
                    .map(|(_, v)| v.clone());
                return Some(Self::media_stream(state, &id, range.as_deref(), req.method == "HEAD"));
            }
        }
        None
    }

    /// 镜像 api_gen auto_media_scan 的 JSON 形态（字段逐一齐平），差异仅
    /// 一处：url/audio_url/video_url 发绝对值（画廊内嵌前端无 env 展开，
    /// §5.3 实施裁定）。
    fn media_scan(state: &NativeMediaState) -> ProxyReply {
        let Some(root) = &state.root else {
            return ProxyReply::json(
                200,
                "{\"entries\":[],\"root_missing\":false}".to_string(),
            );
        };
        if !root.exists() {
            return ProxyReply::json(200, "{\"entries\":[],\"root_missing\":true}".to_string());
        }
        let mut guard = state.index.lock().unwrap();
        let index = guard.get_or_insert_with(|| {
            crate::ui::media_service::index_directory(root).unwrap_or_default()
        });
        let mut out = String::from("{\"entries\":[");
        for (i, e) in index.entries.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let (artist, song_title) =
                crate::ui::media_service::parse_artist_and_title(&e.name);
            let album = if e.rel_dir.is_empty() {
                "本地单曲".to_string()
            } else {
                e.rel_dir.clone()
            };
            let index_str = format!("{:02}", i + 1);
            let abs = format!("{}/apps/{}/api/media/stream/{}", state.base, state.app_id, e.id);
            out.push_str(
                &serde_json::json!({
                    "id": &e.id,
                    "index": i + 1,
                    "index_str": index_str,
                    "title": crate::ui::media_service::display_title(&e.name),
                    "song_title": song_title,
                    "artist": artist,
                    "album": album,
                    "is_liked": false,
                    "name": &e.name,
                    "rel_dir": &e.rel_dir,
                    "relative_path": &e.relative_path,
                    "extension": &e.extension,
                    "bytes": e.bytes,
                    "size_str": crate::ui::media_service::human_size(e.bytes),
                    "url": abs,
                    "audio_url": abs,
                    "video_url": abs,
                })
                .to_string(),
            );
        }
        out.push_str("],\"root_missing\":false}");
        ProxyReply::json(200, out)
    }

    /// 镜像 api_gen auto_media_stream：单区间 Range 语义（Full 200 /
    /// Partial 206 + Content-Range / Unsatisfiable 416），GET 才带 body。
    fn media_stream(
        state: &NativeMediaState,
        id: &str,
        range: Option<&str>,
        head_only: bool,
    ) -> ProxyReply {
        use crate::ui::media_service::{content_range, content_type, find, parse_range, StreamPlan};
        let Some(root) = &state.root else {
            return ProxyReply::json(503, error_json("media root not configured"));
        };
        let mut guard = state.index.lock().unwrap();
        let index = guard.get_or_insert_with(|| {
            crate::ui::media_service::index_directory(root).unwrap_or_default()
        });
        let Some(entry) = find(index, id) else {
            return ProxyReply::json(404, error_json(&format!("unknown media id `{id}`")));
        };
        let path = root.join(&entry.relative_path);
        let Ok(file) = std::fs::File::open(&path) else {
            return ProxyReply::json(404, error_json(&format!("media file missing: {}", entry.name)));
        };
        let len = file.metadata().map(|m| m.len()).unwrap_or(0);
        let plan = parse_range(range, len);
        if matches!(plan, StreamPlan::Unsatisfiable) {
            return ProxyReply::Response {
                status: 416,
                content_type: "application/json",
                body: error_json("range not satisfiable").into_bytes(),
            };
        }
        let (status, start, end) = match &plan {
            StreamPlan::Full => (200u16, 0u64, len.saturating_sub(1)),
            StreamPlan::Partial { start, end } => (206u16, *start, *end),
            StreamPlan::Unsatisfiable => unreachable!(),
        };
        let content_length = plan.content_length(len);
        let ct = content_type(&entry.extension);
        let mut extra_headers = vec![
            ("Accept-Ranges".to_string(), "bytes".to_string()),
            ("Cache-Control".to_string(), "no-store".to_string()),
        ];
        if status == 206 {
            extra_headers.push(("Content-Range".to_string(), content_range(start, end, len)));
        }
        let _ = head_only; // HEAD：Content-Length 已声明，连接即关——省 body
        ProxyReply::Stream {
            status,
            content_type: ct.to_string(),
            extra_headers,
            content_length: if head_only { 0 } else { content_length },
            reader: if head_only {
                Box::new(std::io::empty())
            } else {
                use std::io::{Seek, SeekFrom};
                let mut f = file;
                let _ = f.seek(SeekFrom::Start(start));
                Box::new(f.take(content_length))
            },
        }
    }
}

// ---------------------------------------------------------------------------
// session 线程：装载 back 链 + 请求分发
// ---------------------------------------------------------------------------

struct SessionRuntime {
    app_id: String,
    vm: AutoVM,
    routes: Vec<HttpRoute>,
    /// fn 名 → 声明参数名有序表（按名绑定 body/query 用）。
    fn_params: HashMap<String, Vec<String>>,
    /// PLAN-658 T-04: #[api] fn 名 → (HTTP method, 返回类型 Display 形)。
    /// ~Stream 判定（含 "Stream<"）与 POST 广播判别（void/typing/New<Type>）
    /// 都读它——镜像 api_gen broadcast_event_name 约定。
    fn_meta: HashMap<String, (String, String)>,
    /// PLAN-658 T-04: session 事件总线——~Stream 端点订阅、POST 广播。
    bus: std::sync::Arc<SessionBus>,
}

/// PLAN-658 T-04: 每 session 宿主侧事件总线（镜像 api_gen events.rs 的
/// broadcast channel 语义：POST 处理器广播，SSE 连接订阅）。
struct SessionBus {
    subs: std::sync::Mutex<Vec<mpsc::Sender<String>>>,
}

impl SessionBus {
    fn new() -> Self {
        SessionBus { subs: std::sync::Mutex::new(Vec::new()) }
    }

    fn subscribe(&self) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel();
        self.subs.lock().unwrap().push(tx);
        rx
    }

    fn broadcast(&self, msg: &str) {
        let mut subs = self.subs.lock().unwrap();
        subs.retain(|tx| tx.send(msg.to_string()).is_ok());
    }

    fn has_subscribers(&self) -> bool {
        !self.subs.lock().unwrap().is_empty()
    }
}

/// PLAN-658 T-06: 测试面——`sys.panic_hard(msg)` 触发真 Rust panic（隔离
/// 边界驱动；见 load_back_session 注册点注释）。
fn shim_sys_panic_hard(task: &mut AutoTask, vm: &AutoVM) -> Result<(), crate::vm::engine::VMError> {
    let nv = crate::vm::native::pop_arg_nv(task);
    let msg = if auto_val::is_string(nv) {
        vm.get_string(auto_val::decode_string(nv) as u32)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    } else {
        String::new()
    };
    panic!("{}", if msg.is_empty() { "panic_hard".to_string() } else { msg });
}

/// PLAN-658 T-06: panic 消息提取（&str/String 双态 downcast）。
fn panic_payload_msg(p: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else {
        "(non-string panic)".to_string()
    }
}

/// PLAN-658 T-06: 崩溃隔离 + 退避重启。线程即隔离边界（宿主与其他 session
/// 不受影响由构造保证）；单请求 panic → 500 + 消息 → 指数退避重建 session
/// （back 链重编译装载，状态归零——内存态后端语义诚实）；连续失败封顶
/// 8s。重建失败维持降级应答（503），通道不关闭。
fn session_main(app_id: String, back_entry: std::path::PathBuf, rx: mpsc::Receiver<ProxyRequest>) {
    let mut rt = match load_back_session(&app_id, &back_entry) {
        Ok(rt) => {
            log::info!("[back-proxy:{app_id}] session up ({} routes)", rt.routes.len());
            Some(rt)
        }
        Err(e) => {
            // 装载失败：session 进 degraded 态——通道保持打开但一律 503，
            // 让前端拿到可诊断的错误而不是连接拒绝。
            log::error!("[back-proxy:{app_id}] session failed to load: {e}");
            None
        }
    };
    let mut panic_streak: u32 = 0;
    for req in rx {
        let reply = match &mut rt {
            Some(session) => {
                let app_id_ref = &app_id;
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    session.handle_request(&req)
                })) {
                    Ok(reply) => {
                        panic_streak = 0;
                        reply
                    }
                    Err(p) => {
                        let msg = panic_payload_msg(&p);
                        log::error!("[back-proxy:{app_id_ref}] request panicked: {msg}");
                        session_log(&app_id_ref, &format!("PANIC {msg}"));
                        // 退避后重建（状态归零）。
                        panic_streak += 1;
                        let backoff_ms = backoff_ms(panic_streak);
                        std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                        match load_back_session(&app_id_ref, &back_entry) {
                            Ok(fresh) => {
                                log::warn!(
                                    "[back-proxy:{app_id_ref}] session restarted after panic (streak={panic_streak}, backoff={backoff_ms}ms)"
                                );
                                session_log(&app_id_ref, &format!("RESTART streak={panic_streak}"));
                                *session = fresh;
                                panic_streak = 0;
                            }
                            Err(e) => {
                                log::error!(
                                    "[back-proxy:{app_id_ref}] session rebuild failed after panic: {e}"
                                );
                            }
                        }
                        ProxyReply::json(
                            500,
                            error_json(&format!(
                                "back-proxy:{app_id_ref}: session panicked: {msg} (restart attempted)"
                            )),
                        )
                    }
                }
            }
            None => ProxyReply::json(
                503,
                error_json(&format!(
                    "back-proxy: session for `{app_id}` failed to load (degraded)"
                )),
            ),
        };
        let _ = req.reply.send(reply);
    }
    log::info!("[back-proxy:{app_id}] session down");
}

/// PLAN-658 T-06: 退避曲线——500ms × 2^(n-1)，封顶 8s。
fn backoff_ms(streak: u32) -> u64 {
    (500u64.saturating_mul(1u64 << (streak - 1).min(4))).min(8000)
}

/// PLAN-658 T-06: 每 session 日志环（容量 256；宿主观测面 + 调试路由消费）。
fn session_log(app_id: &str, line: &str) {
    SESSION_LOGS.lock().unwrap().entry(app_id.to_string()).or_default().push(line);
}

#[derive(Default)]
struct SessionLogRing(std::collections::VecDeque<String>);

impl SessionLogRing {
    fn push(&mut self, line: &str) {
        if self.0.len() >= 256 {
            self.0.pop_front();
        }
        self.0.push_back(format!(
            "[{}] {line}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));
    }
}

lazy_static::lazy_static! {
    static ref SESSION_LOGS: std::sync::Mutex<HashMap<String, SessionLogRing>> =
        std::sync::Mutex::new(HashMap::new());
}

/// 装载一个 app 的 back 链为可服务的 VM session。
///
/// 复用 UI merged 路径的既有机制（Plan 330 un-gated 工具族 + Plan 333
/// `__module_init` 全局初始化约定）：
/// `collect_module_imports` 扁平化 → Codegen 有序编译（Use → 类型 →
/// Store(var) 全局化 + `__module_init` → Fn）→ Linker → VirtualFlash →
/// AutoVM → 显式跑 `__module_init` 激活模块级 `var`（contacts/messages 等）。
/// #[api] fn 返回类型的**主类型名**（POST 广播 "New<Type>" 判别用）。
/// 遍历包装层（List/?/!/引用）取首个命名类型；标量返回 None。
fn primary_type_name(t: &crate::ast::Type) -> Option<String> {
    use crate::ast::Type;
    match t {
        Type::User(d) => Some(d.name.to_string()),
        Type::Enum(e) => Some(e.borrow().name.to_string()),
        Type::Tag(tag) => Some(tag.borrow().name.to_string()),
        Type::GenericInstance(inst) => Some(inst.base_name.to_string()),
        Type::List(inner) | Type::Reference(inner) | Type::Option(inner) | Type::Result(inner) => {
            primary_type_name(inner)
        }
        _ => None,
    }
}

fn load_back_session(app_id: &str, back_entry: &std::path::Path) -> Result<SessionRuntime, String> {
    crate::vm::native_registry::register_builtin_natives();

    // 1. 扁平化 back 链（use 递归、fn 模块限定名、类型/store 去重合并）。
    let mut visited = std::collections::HashSet::new();
    let mut seen_symbols = std::collections::HashSet::new();
    let mut import_session = crate::compile::CompileSession::new();
    let mut import_stmts: Vec<Stmt> = Vec::new();
    crate::collect_module_imports(
        back_entry,
        &mut visited,
        &mut import_stmts,
        &mut seen_symbols,
        &mut import_session,
        None,
    );

    // 2. Codegen 有序编译（顺序语义见 handler_codegen::synthesize_widget_module
    //    Plan 318/333 注释：Use 先注册 auto_modules；类型先于 __module_init；
    //    var 声明提升 globals；fn 最后）。
    let mut codegen = crate::vm::codegen::Codegen::new();
    codegen.api_over_http = false;
    for stmt in &import_stmts {
        if let Stmt::Fn(f) = stmt {
            let qualified = f.name.to_string();
            let bare = qualified.rsplit('.').next().unwrap_or(&qualified).to_string();
            codegen
                .import_scope
                .entry(bare)
                .or_insert_with(|| qualified.clone());
            codegen.fn_return_types.insert(qualified.clone(), f.ret.clone());
        }
    }
    for stmt in &import_stmts {
        if let Stmt::Use(_) = stmt {
            if let Err(e) = codegen.compile_stmt(stmt) {
                return Err(format!("back chain use stmt compile failed: {e}"));
            }
        }
    }
    for stmt in &import_stmts {
        if matches!(stmt, Stmt::TypeDecl(_) | Stmt::EnumDecl(_)) {
            if let Err(e) = codegen.compile_stmt(stmt) {
                return Err(format!("back chain type decl compile failed: {e}"));
            }
        }
    }
    for stmt in &import_stmts {
        if let Stmt::Store(s) = stmt {
            if matches!(s.kind, crate::ast::StoreKind::Var) {
                codegen.global_vars.insert(s.name.to_string());
            }
        }
    }
    let store_inits: Vec<Stmt> = import_stmts
        .iter()
        .filter(|s| matches!(s, Stmt::Store(st) if matches!(st.kind, crate::ast::StoreKind::Var)))
        .cloned()
        .collect();
    if !store_inits.is_empty() {
        codegen.force_global_store = true;
        let init_fn = Stmt::Fn(crate::ast::Fn::new(
            crate::ast::FnKind::Function,
            crate::ast::Name::from(MODULE_INIT_FN),
            None,
            Vec::new(),
            crate::ast::Body {
                stmts: store_inits,
                has_new_line: false,
                source_lines: Vec::new(),
            },
            crate::ast::Type::Void,
        ));
        if let Err(e) = codegen.compile_stmt(&init_fn) {
            return Err(format!("__module_init compile failed: {e}"));
        }
        codegen.force_global_store = false;
    }
    for stmt in &import_stmts {
        if matches!(stmt, Stmt::Fn(_) | Stmt::TypeDecl(_) | Stmt::EnumDecl(_) | Stmt::Ext(_)) {
            if let Err(e) = codegen.compile_stmt(stmt) {
                return Err(format!("back chain stmt compile failed: {e}"));
            }
        }
    }

    // 3. 路由表 + 参数名表 + fn 元数据（Plan 312 api_routes；参数名取自 AST，
    //    返回类型双形态：Display（`~Stream<T>` 呈 `linear<Stream<T>>`，流判定
    //    用）+ 主类型名（POST 广播 "New<Type>" 判别用——Display 对 User 类型
    //    印声明文本，不可直接抽名））。
    let api_routes: Vec<(String, String, String)> = codegen.api_routes.clone();
    let mut fn_params: HashMap<String, Vec<String>> = HashMap::new();
    let mut fn_meta: HashMap<String, (String, String)> = HashMap::new();
    for stmt in &import_stmts {
        if let Stmt::Fn(f) = stmt {
            if f.api_attrs.is_some() {
                fn_params.insert(
                    f.name.to_string(),
                    f.params.iter().map(|p| p.name.to_string()).collect(),
                );
                let method = f
                    .api_attrs
                    .as_ref()
                    .map(|a| a.method.clone())
                    .unwrap_or_default();
                let display = format!("{}", f.ret);
                let primary = primary_type_name(&f.ret)
                    .unwrap_or_else(|| display.clone());
                fn_meta.insert(f.name.to_string(), (method, format!("{primary}|{display}")));
            }
        }
    }

    let registry = std::mem::take(&mut codegen.generic_registry);
    let module = codegen.finish(format!("<back-proxy:{app_id}>"));
    let object_keys = module.object_keys.clone();
    let object_types = module.object_types.clone();
    let strings = module.strings.clone();

    // 4. 链接 + 装载。
    let mut linker = Linker::new();
    linker.add_entry_module(module);
    let (code, exports) = linker.link().map_err(|e| format!("link failed: {e}"))?;
    let flash = VirtualFlash::from_vec_with_metadata(code, exports, object_keys, object_types);
    let mut vm = AutoVM::new(flash, 8192);
    vm.load_generic_registry(registry);
    vm.load_strings(strings);
    {
        let mut ni = crate::vm::native::NativeInterface::new();
        ni.register_std_shims();
        crate::vm::ffi::stdlib::register_stdlib_ffi(&mut ni);
        // PLAN-658 T-06: 崩溃隔离测试面——真 Rust panic 注入（VM 内建
        // `panic` 有意映射 RuntimeError，走的是 500 错误臂而非 unwind 边界；
        // 本原生专供 AC-05 隔离/重启路径的可控触发）。
        ni.register_shim_by_name("auto.sys.panic_hard", shim_sys_panic_hard);
        vm.merge_native_interface(&ni);
    }

    let rt_routes: Vec<HttpRoute> = api_routes
        .into_iter()
        .map(|(method, path, fn_name)| HttpRoute { method, path, fn_name })
        .collect();

    let mut session = SessionRuntime {
        app_id: app_id.to_string(),
        vm,
        routes: rt_routes,
        fn_params,
        fn_meta,
        bus: std::sync::Arc::new(SessionBus::new()),
    };

    // 5. 激活模块级全局（var contacts = ... 等）。
    session.run_module_init()?;

    Ok(session)
}

impl SessionRuntime {
    fn run_module_init(&mut self) -> Result<(), String> {
        if !self.vm.flash.exports_by_name.contains_key(MODULE_INIT_FN) {
            return Ok(());
        }
        let mut task = AutoTask::new(0, 4096, 0);
        self.vm
            .call_fn_by_name(&mut task, MODULE_INIT_FN, 0)
            .map_err(|e| format!("__module_init failed: {e:?}"))
    }

    fn handle_request(&mut self, req: &ProxyRequest) -> ProxyReply {
        let Some(route_match) = match_route(&self.routes, &req.method, &req.path) else {
            return ProxyReply::json(
                404,
                error_json(&format!(
                    "back-proxy:{}: no route {} {}",
                    self.app_id, req.method, req.path
                )),
            );
        };
        let Some(param_names) = self.fn_params.get(&route_match.fn_name).cloned() else {
            return ProxyReply::json(
                500,
                error_json(&format!(
                    "back-proxy:{}: route fn `{}` has no recorded signature",
                    self.app_id, route_match.fn_name
                )),
            );
        };

        // PLAN-658 T-04: ~Stream 端点按签名特路（两个生成器同语义：函数体
        // 不在执行面——bus.subscribe 是宿主 seam）。订阅 session 事件总线，
        // listener 侧逐帧 SSE 转发。
        if let Some((_, ret)) = self.fn_meta.get(&route_match.fn_name) {
            if ret.contains("Stream<") {
                let rx = self.bus.subscribe();
                log::info!(
                    "[back-proxy:{}] SSE subscribe {} {} (subs={})",
                    self.app_id,
                    req.method,
                    req.path,
                    self.bus.subs.lock().unwrap().len()
                );
                return ProxyReply::Sse(rx);
            }
        }

        // 请求体 JSON（POST/PUT/PATCH 按字段名绑定；解析失败按原始字符串
        // 透传给单参形态的兼容臂——与既有 split wire 的宽容度一致）。
        let body_json: Option<serde_json::Value> = if req.body.is_empty() {
            None
        } else {
            serde_json::from_slice(&req.body).ok()
        };
        let query_map = parse_query(&req.query);

        // 按名绑定参数：路径占位符 → body 字段 → query 参数。
        let mut bound: Vec<serde_json::Value> = Vec::with_capacity(param_names.len());
        for name in &param_names {
            if let Some((_, v)) = route_match.path_params.iter().find(|(n, _)| n == name) {
                bound.push(serde_json::Value::String(v.clone()));
                continue;
            }
            if let Some(v) = body_json.as_ref().and_then(|b| b.get(name)) {
                bound.push(v.clone());
                continue;
            }
            if let Some(v) = query_map.get(name) {
                bound.push(serde_json::Value::String(v.clone()));
                continue;
            }
            return ProxyReply::json(
                400,
                error_json(&format!(
                    "back-proxy:{}: missing param `{name}` for {} {}",
                    self.app_id, req.method, req.path
                )),
            );
        }

        // 调用（VmBridge::call_vm_fn 范本：fresh task + 左→右压参 +
        // call_fn_by_name + 结果弹出 + 整栈 rc 清账）。
        let mut task = AutoTask::new(0, 4096, 0);
        let pre_args_sp = task.ram.sp;
        for v in &bound {
            if let Err(e) =
                crate::vm::ffi::stdlib::json_to_vm_value(&mut task, &self.vm, v, 0)
            {
                return ProxyReply::json(
                    500,
                    error_json(&format!(
                        "back-proxy:{}: arg marshal failed: {e:?}",
                        self.app_id
                    )),
                );
            }
        }
        if let Err(e) = self.vm.call_fn_by_name(&mut task, &route_match.fn_name, bound.len()) {
            return ProxyReply::json(
                500,
                error_json(&format!(
                    "back-proxy:{}: `{}` failed: {e:?} (crash ip=0x{:x})",
                    self.app_id, route_match.fn_name, task.ip
                )),
            );
        }
        let body_json_str = if task.ram.sp > pre_args_sp {
            let nv = task.ram.pop_nv();
            crate::vm::ffi::http_server::nv_to_json(&self.vm, nv, 6)
                .unwrap_or_else(|| "null".to_string())
        } else {
            "null".to_string()
        };
        self.vm.rc_release_task_stack(&mut task);

        // PLAN-658 T-04: POST 广播（镜像 api_gen 广播约定——session 存在
        // ~Stream 端点时才广播）：typing 型（fn 名含 "typing"）void POST →
        // {"event":"Typing","name":<首参>}；create 型非 void POST → 返回实体
        // 加 "event":"New<主类型>" 判别后广播。
        let has_sse = self.fn_meta.values().any(|(_, r)| r.contains("Stream<"));
        if has_sse {
            if let Some((method, meta)) = self.fn_meta.get(&route_match.fn_name).cloned() {
                let (primary, display) = meta.split_once('|').unwrap_or((meta.as_str(), ""));
                if method.eq_ignore_ascii_case("POST") {
                    let fn_bare = route_match
                        .fn_name
                        .rsplit('.')
                        .next()
                        .unwrap_or(&route_match.fn_name)
                        .to_lowercase();
                    if fn_bare.contains("typing") {
                        let name = bound
                            .first()
                            .map(|v| v.as_str().unwrap_or_default().to_string())
                            .unwrap_or_default();
                        self.bus.broadcast(&format!(
                            "{{\"event\":\"Typing\",\"name\":{}}}",
                            serde_json::json!(name)
                        ));
                    } else if display != "void" && !display.contains("Stream<") {
                        if let Ok(mut v) =
                            serde_json::from_str::<serde_json::Value>(&body_json_str)
                        {
                            if let Some(obj) = v.as_object_mut() {
                                obj.insert(
                                    "event".to_string(),
                                    serde_json::Value::String(format!("New{primary}")),
                                );
                                self.bus.broadcast(&v.to_string());
                            }
                        }
                    }
                }
            }
        }

        if std::env::var("AUTO_BACK_PROXY_TRACE").is_ok() {
            log::info!(
                "[back-proxy:{}] {} {} -> {}",
                self.app_id,
                req.method,
                req.path,
                body_json_str.len()
            );
        }
        ProxyReply::json(200, body_json_str)
    }
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        out.insert(url_decode(k), url_decode(v));
    }
    out
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                if let Ok(b) = u8::from_str_radix(hex, 16) {
                    out.push(b);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
