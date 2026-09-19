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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

struct ProxyShared {
    /// app_id → session 请求通道。
    sessions: HashMap<String, mpsc::Sender<ProxyRequest>>,
    /// PLAN-658 T-03: 宿主原生 media 路由状态（cfg ui）。
    #[cfg(feature = "ui")]
    native_media: HashMap<String, NativeMediaState>,
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
        content_type: &'static str,
        extra_headers: Vec<(String, String)>,
        content_length: u64,
        reader: Box<dyn std::io::Read + Send>,
    },
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
        std::thread::Builder::new()
            .name(spawn_name)
            .stack_size(16 * 1024 * 1024)
            .spawn(move || session_main(spawn_app_id, entry, rx))
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("spawn session thread for {app_id} failed: {e}"),
                )
            })?;
        sessions.insert(app_id, tx);
    }

    let shared = Arc::new(ProxyShared {
        sessions,
        #[cfg(feature = "ui")]
        native_media: {
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
        },
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
        self.shared.sessions.contains_key(app_id)
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
    let Some(tx) = shared.sessions.get(&app_id) else {
        return ProxyReply::json(404, error_json(&format!("back-proxy: unknown app `{app_id}`")));
    };    let (reply_tx, reply_rx) = mpsc::channel::<ProxyReply>();
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
    let (status, content_type, extra_headers, body_source) = match reply {
        ProxyReply::Response { status, content_type, body } => {
            (status, content_type, Vec::<(String, String)>::new(), BodySource::Bytes(body))
        }
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
    /// （回落 session 分发）。
    fn try_native_media(
        &self,
        app_id: &str,
        sub_path: &str,
        req: &ParsedRequest,
    ) -> Option<ProxyReply> {
        let state = self.native_media.get(app_id)?;
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
            content_type: ct,
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
}

fn session_main(app_id: String, back_entry: std::path::PathBuf, rx: mpsc::Receiver<ProxyRequest>) {
    match load_back_session(&app_id, &back_entry) {
        Ok(mut rt) => {
            log::info!("[back-proxy:{app_id}] session up ({} routes)", rt.routes.len());
            for req in rx {
                let reply = rt.handle_request(&req);
                let _ = req.reply.send(reply);
            }
        }
        Err(e) => {
            // 装载失败：session 进 degraded 态——通道保持打开但一律 503，
            // 让前端拿到可诊断的错误而不是连接拒绝。
            log::error!("[back-proxy:{app_id}] session failed to load: {e}");
            for req in rx {
                let _ = req.reply.send(ProxyReply::json(
                    503,
                    error_json(&format!("back-proxy: session for `{app_id}` failed to load: {e}")),
                ));
            }
        }
    }
    log::info!("[back-proxy:{app_id}] session down");
}

/// 装载一个 app 的 back 链为可服务的 VM session。
///
/// 复用 UI merged 路径的既有机制（Plan 330 un-gated 工具族 + Plan 333
/// `__module_init` 全局初始化约定）：
/// `collect_module_imports` 扁平化 → Codegen 有序编译（Use → 类型 →
/// Store(var) 全局化 + `__module_init` → Fn）→ Linker → VirtualFlash →
/// AutoVM → 显式跑 `__module_init` 激活模块级 `var`（contacts/messages 等）。
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

    // 3. 路由表 + 参数名表（Plan 312 api_routes；参数名取自 AST 声明）。
    let api_routes: Vec<(String, String, String)> = codegen.api_routes.clone();
    let mut fn_params: HashMap<String, Vec<String>> = HashMap::new();
    for stmt in &import_stmts {
        if let Stmt::Fn(f) = stmt {
            if f.api_attrs.is_some() {
                fn_params.insert(
                    f.name.to_string(),
                    f.params.iter().map(|p| p.name.to_string()).collect(),
                );
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
