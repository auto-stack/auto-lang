//! VM `#[api]` HTTP server core: route table, argument binding, middleware,
//! handler dispatch, SSE producers and the owned request/reply bridge.
//!
//! PLAN-699 (Design 33 阶段 B): the HTTP/1.1 wire protocol is Axum/Hyper's
//! job — the transport lives in [`super::http_transport`] on a dedicated net
//! thread and hands owned `ApiRequest`s to [`dispatch_api_request`] on the
//! VM owner thread. AutoVM is `!Send` (`Rc<RefCell>` in its type system);
//! only owned `Send` data crosses the bridge — no `usize` pointer
//! laundering, no `spawn_blocking` VM calls. This module is the VM server
//! only: the generated Rust track (auto-man api_gen) builds its own Axum
//! service, and `shim_http_server_listen`/`back_proxy` are separate server
//! paths (see docs/specs/stdlib/design/backend-assembly.md).

use std::io::{BufRead, Read, Write};
use std::net::TcpListener;

/// An HTTP route registered with the server.
#[derive(Debug, Clone)]
pub struct HttpRoute {
    pub method: String,
    pub path: String,
    pub fn_name: String,
}

/// Result of matching a request against routes.
#[derive(Debug, Clone)]
pub struct RouteMatch {
    pub fn_name: String,
    pub path_params: Vec<(String, String)>,
    pub query_params: Vec<(String, String)>,
}

// ============================================================================
// PLAN-669: #[api] handler 形参签名侧信道
// ============================================================================
// codegen 在 api_routes.push 处同步发布每个 #[api] fn 的形参（名+类型），
// 服务侧据此按名装配 handler 实参（路径段 → body 字段 → query，缺参 400；
// 见 bind_api_args_by_name）。生命周期模型同 stdlib 的 HTTP_ROUTES /
// axum_adapter 的 PARAM_SIGS：每驱动进程一个程序，run pipeline 编译前重置
// （clear_api_param_sigs），e2e 隔离经 clear_http_routes 连带清空。
// 条目缺失（legacy 生产者）→ 服务侧回退旧位序装配，行为不变。

/// One declared `#[api]` handler parameter: name + type display string
/// ("int"/"str"/"bool"/... — `Type`'s Display, same source as
/// axum_adapter::record_param_sig).
#[derive(Debug, Clone)]
pub struct ApiParamSig {
    pub name: String,
    pub ty: String,
}

static API_PARAM_SIGS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, Vec<ApiParamSig>>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// Codegen publishes each `#[api]` fn's declared params at fn-compile time
/// (dep-module Codegen instances publish here too, so by-name binding resolves
/// for cross-module handlers — same lifetime model as record_param_sig).
pub fn record_api_param_sigs(fn_name: &str, sigs: Vec<ApiParamSig>) {
    if let Ok(mut table) = API_PARAM_SIGS.lock() {
        table.insert(fn_name.to_string(), sigs);
    }
}

/// Resolve a `#[api]` handler fn's declared params by name (clone).
pub fn api_param_sigs(fn_name: &str) -> Option<Vec<ApiParamSig>> {
    API_PARAM_SIGS
        .lock()
        .ok()
        .and_then(|t| t.get(fn_name).cloned())
}

/// PLAN-698 T-02/SD-02: #[api] fn 返回类型显示串侧信道（与 API_PARAM_SIGS
/// 同生命周期模型）。SSE publisher 臂据此判 has_sse（任一 ~Stream 端点）
/// 并按 api_gen broadcast_event_name 同款约定拼 New{Type} 事件名。
static API_RETURN_TYPES: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, String>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// Codegen 在 #[api] fn 编译时发布返回类型（dep-module Codegen 同步发布）。
pub fn record_api_return_type(fn_name: &str, ret_display: String) {
    if let Ok(mut table) = API_RETURN_TYPES.lock() {
        table.insert(fn_name.to_string(), ret_display);
    }
}

/// 本工程是否声明了 ~Stream 端点（对齐 api_gen 的 has_sse 广播门控）。
pub fn has_stream_endpoint() -> bool {
    API_RETURN_TYPES
        .lock()
        .map(|t| t.values().any(|r| r.contains("Stream")))
        .unwrap_or(false)
}

/// Reset before compiling a program (the run pipeline calls this next to
/// axum_adapter::reset; the codegen publishers rebuild the table from scratch).
pub fn clear_api_param_sigs() {
    if let Ok(mut table) = API_PARAM_SIGS.lock() {
        table.clear();
    }
    if let Ok(mut table) = API_RETURN_TYPES.lock() {
        table.clear();
    }
    if let Ok(mut table) = API_ASYNC_RETURNS.lock() {
        table.clear();
    }
}

/// PLAN-705 T-04/SD-01: 声明 `~T`（Future<T>）返回的函数名集——HTTP 编组
/// 阶段判定"返回值可能是 future bits"的**元数据门**（禁止把普通 int 的
/// 位模式猜成 future ID 后误消费；AC-01 普通 int 反例由此挡住）。对全部
/// fn 发布（非仅 #[api]——段驱动 handler/`__axum:` fn-ref closure 同样
/// 依此判定；closure 经 func_addr → exports 反查名，同 resolve_params）。
static API_ASYNC_RETURNS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

/// Codegen 在 fn 编译时发布（声明 ret 为 Future<T> 时调用）。
pub fn record_api_async_return(fn_name: &str) {
    if let Ok(mut table) = API_ASYNC_RETURNS.lock() {
        table.insert(fn_name.to_string());
    }
}

/// 编组期判定：该函数（或 fn-ref closure 反查出的函数）声明 `~T` 返回。
pub(crate) fn fn_is_api_async(fn_name: &str) -> bool {
    API_ASYNC_RETURNS
        .lock()
        .ok()
        .map(|t| t.contains(fn_name))
        .unwrap_or(false)
}

/// PLAN-729 T-04：编组期文件返回门——声明返回类型（`FileResponse` 或
/// `Future<FileResponse>`）含 "FileResponse"。与登记命中（
/// http_server_file::take_file_response）共同识别文件结果；普通 int 即使
/// 数值撞上描述符 id 也走 JSON 兜底（AC-01 反例防线）。
pub(crate) fn fn_is_api_file_return(fn_name: &str) -> bool {
    fn_response_kind(fn_name) == crate::api::contract::ResponseKind::File
}

/// PLAN-730 T-02：编组期上传收据返回门——声明返回类型（`UploadReceipt`
/// 或 `Future<UploadReceipt>`）含 "UploadReceipt"。与登记命中
/// （http_upload::take_upload_receipt）共同识别；普通 int 撞号走 JSON
/// 兜底（同 729 三重命中形态）。
pub(crate) fn fn_is_api_upload_return(fn_name: &str) -> bool {
    fn_response_kind(fn_name) == crate::api::contract::ResponseKind::Upload
}

/// PLAN-734 T-03：返回种类单源——API_RETURN_TYPES 条目经契约身份分类
/// （ResponseKind::from_return_string；假同名类型不命中）。
pub(crate) fn fn_response_kind(fn_name: &str) -> crate::api::contract::ResponseKind {
    API_RETURN_TYPES
        .lock()
        .ok()
        .and_then(|t| t.get(fn_name).map(|r| crate::api::contract::ResponseKind::from_return_string(r)))
        .unwrap_or(crate::api::contract::ResponseKind::Json)
}

/// PLAN-730 T-02：路由的上传能力分类——方法+参数类型双条件（不按
/// Content-Type/URL 猜）。bridge deferral 与 owner dispatch 共用同一判定。
pub fn route_declares_upload(fn_name: &str) -> bool {
    api_param_sigs(fn_name)
        .map(|sigs| {
            sigs.iter()
                .any(|p| crate::api::contract::is_upload_param(&p.ty))
        })
        .unwrap_or(false)
}

/// PLAN-730 T-05：bridge 侧上传路由预判（方法已由调用方限定 POST/PUT）。
/// 与 owner 共用 match_route + 类型分类——同一请求两臂判定一致。
pub fn upload_route_lookup(method: &str, path: &str) -> bool {
    let routes = get_routes();
    match_route(&routes, method, path)
        .map(|rm| route_declares_upload(&rm.fn_name))
        .unwrap_or(false)
}

/// PLAN-698 T-02/SD-02: POST 成功后的 SSE 广播臂——生成 Axum 侧
/// void-POST+broadcast 路径（api_gen broadcast_event_name + events::broadcast）
/// 的 VM 运行面对照实现。仅当工程声明 ~Stream 端点时发布（has_sse 门控同
/// api_gen）。事件形态：
/// - fn 名含 "typing"（void 信号端点）→ `{"event":"Typing","name":<首个
///   str 形参的请求值>}`（api_gen 取首个 body 形参的同一约定）；
/// - 其余 POST（实体创建）→ 响应体（序列化实体）注入 `"event":"New{RetType}"`。
pub fn publish_post_broadcast(fn_name: &str, request_body: &str, response_json: &str) {
    if !has_stream_endpoint() {
        return;
    }
    if fn_name.to_lowercase().contains("typing") {
        let name_field = api_param_sigs(fn_name)
            .and_then(|sigs| {
                sigs.into_iter()
                    .find(|p| p.ty.contains("str"))
                    .map(|p| p.name)
            })
            .unwrap_or_else(|| "sender".to_string());
        let name = serde_json::from_str::<serde_json::Value>(request_body)
            .ok()
            .and_then(|v| v.get(&name_field).cloned())
            .unwrap_or(serde_json::Value::Null);
        crate::vm::ffi::stdlib::bus_broadcast(
            serde_json::json!({ "event": "Typing", "name": name }).to_string(),
        );
        return;
    }
    // 实体创建：响应体必须是 JSON 对象才注入事件名（流式/空响应跳过）。
    let ret = API_RETURN_TYPES
        .lock()
        .ok()
        .and_then(|t| t.get(fn_name).cloned());
    let Some(ret) = ret else { return };
    if ret.contains("Stream") || ret == "()" || ret.is_empty() {
        return;
    }
    let Ok(mut evt) = serde_json::from_str::<serde_json::Value>(response_json) else {
        return;
    };
    if !evt.is_object() {
        return;
    }
    evt["event"] = serde_json::Value::String(format!("New{}", ret));
    crate::vm::ffi::stdlib::bus_broadcast(evt.to_string());
}

/// Match a request (method, path) against a list of routes.
/// Supports `:param` path parameter extraction (e.g. /api/notes/:id).
pub fn match_route(routes: &[HttpRoute], method: &str, path: &str) -> Option<RouteMatch> {
    // Plan 346: Split path and query string (e.g. /api/notes?page=1&size=10).
    let (path_only, query_string) = match path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (path, ""),
    };

    // Parse query parameters.
    let query_params: Vec<(String, String)> = if query_string.is_empty() {
        Vec::new()
    } else {
        query_string
            .split('&')
            .filter_map(|pair| {
                let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
                Some((url_decode(k), url_decode(v)))
            })
            .collect()
    };

    // PLAN-729 R1（G-11 修复）：两遍匹配——第一遍精确 method；HEAD 请求在
    // 第二遍回退到声明文件返回的 GET 路由（自动 HEAD）。显式同路径 HEAD
    // 路由因第一遍先匹配而优先，与注册顺序无关（SD-01 §3 承诺对齐）。
    // PLAN-093 (G-8 系)：尾部 catch-all（`*` / `*name`，由 axum `{*rest}`
    // 翻译而来）消费全部剩余段——段数严格相等检查曾使多段剩余路径永远
    // 404（/api/files/{ws}/{*path} 即此）。
    let head_fallback = method.eq_ignore_ascii_case("HEAD");
    let mut pass = 0;
    while pass < 2 {
        let use_fallback = pass == 1;
        for route in routes {
            let method_ok = route.method.to_uppercase() == method.to_uppercase()
                || (use_fallback
                    && head_fallback
                    && route.method.eq_ignore_ascii_case("GET")
                    && fn_is_api_file_return(&route.fn_name));
            if !method_ok {
                continue;
            }
            let route_segments: Vec<&str> = route.path.split('/').collect();
            let path_segments: Vec<&str> = path_only.split('/').collect();
            let catch_all = route_segments
                .last()
                .map_or(false, |s| *s == "*" || s.starts_with('*'));
            if catch_all {
                if path_segments.len() < route_segments.len() - 1 {
                    continue;
                }
            } else if route_segments.len() != path_segments.len() {
                continue;
            }
            let mut params = Vec::new();
            let mut matched = true;
            for (i, rs) in route_segments.iter().enumerate() {
                if let Some(param_name) = rs.strip_prefix(':') {
                    // Plan 022 (auto-down): Path params must arrive
                    // percent-DECODED (axum Path semantics): the front calls
                    // encodeURIComponent on wiki titles ("Hello%20World.ad"),
                    // and the undecoded form misses the file on disk.
                    let Some(ps) = path_segments.get(i) else {
                        matched = false;
                        break;
                    };
                    let decoded = url_decode(ps);
                    if let Some(wild_name) = param_name.strip_prefix('*') {
                        params.push((wild_name.to_string(), decoded));
                    } else {
                        params.push((param_name.to_string(), decoded));
                    }
                } else if *rs == "*" || rs.starts_with('*') {
                    // Plan 346 通配 + axum `{*name}` 语义（G-8）：剩余段以 '/'
                    // 连接（解码后）作为该参数值；无名 `*` 仅消费不产出参数。
                    let wild_name = rs.strip_prefix('*').unwrap_or("");
                    let rest: Vec<&str> = path_segments[i.min(path_segments.len())..].to_vec();
                    let decoded = url_decode(&rest.join("/"));
                    if !wild_name.is_empty() {
                        params.push((wild_name.to_string(), decoded));
                    }
                    break;
                } else if path_segments.get(i) != Some(rs) {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Some(RouteMatch {
                    fn_name: route.fn_name.clone(),
                    path_params: params,
                    query_params,
                });
            }
        }
        pass += 1;
    }
    None
}

/// Plan 349 步骤 7/8 (W5): CORS origin for the AutoVM server. Reads `AUTO_CORS_ORIGIN`
/// (default `*`) so deployments can lock down the allowed origin via env var,
/// mirroring the `AUTO_*` convention used by the a2r HTTP client (Plan 388).
fn cors_origin() -> String {
    std::env::var("AUTO_CORS_ORIGIN").unwrap_or_else(|_| "*".to_string())
}

/// Plan 349 步骤 7/8 (W5): static CORS response-header block (CRLF-terminated, no
/// trailing blank line) appended to every server response so browser clients
/// can read the body. Origin is read once per call to honor env overrides.
pub(crate) fn cors_headers() -> String {
    let origin = cors_origin();
    format!(
        "Access-Control-Allow-Origin: {}\r\n\
         Access-Control-Allow-Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
         Access-Control-Max-Age: 86400\r\n",
        origin
    )
}

/// Plan 346 B6: process-wide rate limiter (fixed window per client IP) and
/// request-id minting for `dispatch_api_request` (PLAN-699: the legacy
/// inline handler this comment named was removed by the Axum transport).

/// Plan 346 5e (B6): active rate-limit config `(max_requests, window_ms)`.
/// `None` (default) = no limiting — backwards compatible until
/// `http.rate_limit(n, ms)` is called.
static RATE_LIMIT_CFG: std::sync::Mutex<Option<(u32, u64)>> = std::sync::Mutex::new(None);
/// Plan 346 5e (B6): per-IP fixed-window buckets `ip -> (window_start_ms, count)`.
static RATE_BUCKETS: std::sync::Mutex<Option<std::collections::HashMap<String, (u64, u32)>>> =
    std::sync::Mutex::new(None);
/// Plan 346 #12 (B6): request-id counter for minted ids (incoming ids pass
/// through unchanged).
static REQ_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

// ── Plan 346 5a (B6 multipart): server-side multipart/form-data parsing ──

/// One parsed multipart part: a text field (`filename == None`) or a file
/// part with its raw bytes.
pub struct MultipartPart {
    pub name: String,
    pub filename: Option<String>,
    pub data: Vec<u8>,
}

/// Plan 346 5a: split a multipart/form-data body on its boundary and parse
/// each part's Content-Disposition (name/filename) + data (RFC 2046
/// simplified: CRLF-delimited parts, closing `--boundary--`).
pub fn parse_multipart(body: &[u8], boundary: &str) -> Vec<MultipartPart> {
    let delim = format!("--{}", boundary);
    let delim_b = delim.as_bytes();
    let mut parts = Vec::new();
    // Find the first delimiter line, then iterate delimiter-separated parts.
    let mut pos = match find_sub(body, delim_b) {
        Some(p) => p + delim_b.len(),
        None => return parts,
    };
    loop {
        // At delimiter end: either `--` (closing) or CRLF (part follows).
        if body[pos..].starts_with(b"--") {
            break;
        }
        if body[pos..].starts_with(b"\r\n") {
            pos += 2;
        } else if body[pos..].starts_with(b"\n") {
            pos += 1;
        }
        // Part headers end at the first blank line.
        let (headers_raw, data_start) = match find_sub(&body[pos..], b"\r\n\r\n") {
            Some(h) => (pos..pos + h, pos + h + 4),
            None => match find_sub(&body[pos..], b"\n\n") {
                Some(h) => (pos..pos + h, pos + h + 2),
                None => break,
            },
        };
        let headers = String::from_utf8_lossy(&body[headers_raw]).to_string();
        // Content-Disposition: form-data; name="x"; filename="y"
        let mut name = String::new();
        let mut filename = None;
        for h_line in headers.lines() {
            if h_line.to_lowercase().starts_with("content-disposition:") {
                for attr in h_line.split(';').skip(1) {
                    let attr = attr.trim();
                    if let Some(v) = attr.strip_prefix("name=") {
                        name = v.trim_matches('"').to_string();
                    } else if let Some(v) = attr.strip_prefix("filename=") {
                        filename = Some(v.trim_matches('"').to_string());
                    }
                }
            }
        }
        // Data runs to the next CRLF + delimiter.
        let mut search = Vec::with_capacity(delim_b.len() + 4);
        search.extend_from_slice(b"\r\n");
        search.extend_from_slice(delim_b);
        let data_end = match find_sub(&body[data_start..], &search) {
            Some(e) => data_start + e,
            None => match find_sub(&body[data_start..], delim_b) {
                Some(e) => data_start + e,
                None => body.len(),
            },
        };
        parts.push(MultipartPart {
            name,
            filename,
            data: body[data_start..data_end].to_vec(),
        });
        // Advance past this part's closing delimiter.
        pos = match find_sub(&body[data_end..], delim_b) {
            Some(d) => data_end + d + delim_b.len(),
            None => break,
        };
    }
    parts
}

fn find_sub(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// PLAN-730 T-07：legacy 落盘记录（files JSON 项 + provisional 清理路径）。
pub(crate) struct LegacyStoredFile {
    pub field: String,
    pub filename: String,
    pub path: String,
    pub size: usize,
}

/// Plan 346 5a / PLAN-730 T-07：save file parts under the upload dir (env
/// `AUTO_UPLOAD_DIR`, default `./uploads`)。文件名归约同历史（basename +
/// 计数前缀防同名互踩）。**错误必须传播**（目录创建/写失败 → Err——
/// 不返回不存在的假路径）；在宿主 spawn_blocking 执行，不在 owner。
pub(crate) fn legacy_store_files(
    files: &[(String, String, Vec<u8>)],
) -> Result<Vec<LegacyStoredFile>, String> {
    static FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let dir = std::env::var("AUTO_UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    std::fs::create_dir_all(&dir).map_err(|e| format!("upload dir create failed: {e}"))?;
    let mut stored = Vec::with_capacity(files.len());
    for (field, filename, data) in files {
        let base = filename
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("upload")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect::<String>();
        let base = if base.is_empty() {
            "upload".to_string()
        } else {
            base
        };
        let n = FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let path = std::path::Path::new(&dir).join(format!("{}_{}", n, base));
        std::fs::write(&path, data).map_err(|e| {
            format!(
                "upload write failed for {}: {e}",
                path.to_string_lossy()
            )
        })?;
        stored.push(LegacyStoredFile {
            field: field.clone(),
            filename: filename.clone(),
            path: path.to_string_lossy().to_string(),
            size: data.len(),
        });
    }
    Ok(stored)
}

/// Plan 346 5a / PLAN-730 T-07：handler-facing JSON for a multipart request:
/// `{"fields":{"k":"v"},"files":[{"field","filename","path","size"}]}`。
/// 文本字段即时折叠（纯内存，dispatch 段）；file 项由**中间件之后**的
/// 宿主落盘结果合并（multipart_files_json）——解析与落盘分离。
pub fn multipart_fields_json(parts: &[MultipartPart]) -> String {
    let mut fields: Vec<String> = Vec::new();
    for part in parts {
        if part.filename.is_none() {
            let text = String::from_utf8_lossy(&part.data).to_string();
            fields.push(format!(
                "\"{}\":\"{}\"",
                part.name.replace('"', "\\\""),
                text.replace('\\', "\\\\").replace('"', "\\\"")
            ));
        }
    }
    fields.join(",")
}

/// PLAN-730 T-07：宿主落盘结果 → files JSON 项（历史形状逐字节保持）。
pub(crate) fn multipart_files_json(stored: &[LegacyStoredFile]) -> String {
    stored
        .iter()
        .map(|f| {
            format!(
                "{{\"field\":\"{}\",\"filename\":\"{}\",\"path\":\"{}\",\"size\":{}}}",
                f.field.replace('"', "\\\""),
                f.filename.replace('"', "\\\""),
                f.path.replace('\\', "/").replace('"', "\\\""),
                f.size
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// PLAN-730 T-07：fields + files 合并为 handler body（B6 历史成功形状）。
pub(crate) fn multipart_handler_json(fields: &str, files: &str) -> String {
    format!("{{\"fields\":{{{}}},\"files\":[{}]}}", fields, files)
}

/// Plan 346 5e (B6): enable per-IP fixed-window rate limiting.
/// Called by the `http.rate_limit(max_requests, window_ms)` native.
pub fn set_rate_limit(max_requests: u32, window_ms: u64) {
    if let Ok(mut cfg) = RATE_LIMIT_CFG.lock() {
        *cfg = if max_requests == 0 {
            None
        } else {
            Some((max_requests, window_ms.max(1)))
        };
    }
}

/// Plan 346 5e (B6): reset config + buckets. Called from `clear_http_routes`
/// so each e2e test starts unthrottled (same-process tests share 127.0.0.1).
pub fn clear_rate_limit() {
    if let Ok(mut cfg) = RATE_LIMIT_CFG.lock() {
        *cfg = None;
    }
    if let Ok(mut buckets) = RATE_BUCKETS.lock() {
        *buckets = None;
    }
}

/// Plan 346 5e (B6): consume one request slot for `ip`.
/// Returns `Some(retry_after_ms)` when the request exceeds the window quota
/// (and must be rejected with 429), `None` when allowed.
fn rate_limit_take(ip: &str) -> Option<u64> {
    let cfg = RATE_LIMIT_CFG.lock().ok()?.clone()?;
    let (max, window_ms) = cfg;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let mut guard = RATE_BUCKETS.lock().ok()?;
    let buckets = guard.get_or_insert_with(std::collections::HashMap::new);
    let entry = buckets.entry(ip.to_string()).or_insert((now_ms, 0));
    if now_ms.saturating_sub(entry.0) >= window_ms {
        *entry = (now_ms, 0);
    }
    entry.1 += 1;
    if entry.1 > max {
        let retry_after = window_ms.saturating_sub(now_ms.saturating_sub(entry.0));
        Some(retry_after.max(1))
    } else {
        None
    }
}

/// Plan 346 #12 (B6): mint a request id (`req-<unix_ms>-<counter>-<hex>`),
/// used only when the client did not supply `X-Request-Id`.
fn gen_request_id() -> String {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let n = REQ_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!("req-{:x}-{:x}", now_ms, n)
}

/// Plan 349 步骤 7/8 (W5): reason phrase for common status codes (redirect focus).
fn http_status_reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        413 => "Payload Too Large",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "",
    }
}

/// Plan 349 步骤 7/8 (W5): write a CORS preflight (OPTIONS) response and return true if
/// the request is an OPTIONS method; returns false otherwise so the caller can
/// continue normal routing. Used by both blocking and async servers.
pub(crate) fn handle_cors_preflight(method: &str) -> Option<String> {
    if crate::http_service_config::service_policy_active() {
        // 服务面：preflight 由桥层按配置决策（含 Origin 校验），不经此直通 204。
        return None;
    }
    if method.eq_ignore_ascii_case("OPTIONS") {
        Some(format!("HTTP/1.1 204 No Content\r\n{}\r\n", cors_headers()))
    } else {
        None
    }
}

/// Plan 346: Simple URL-decode (percent-encoding).
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

/// Get the global HTTP routes (populated by VM startup from #[api] annotations).
/// This delegates to the existing HTTP_ROUTES global in stdlib.rs.
pub fn get_routes() -> Vec<HttpRoute> {
    crate::vm::ffi::stdlib::get_http_routes()
        .into_iter()
        .map(|(method, path, fn_name)| HttpRoute {
            method,
            path,
            fn_name,
        })
        .collect()
}

/// Plan 326 Phase 3: Serialize a handler return value (NanoValue) to a JSON string.
///
/// Root cause of the "handler returns struct → null" bug: struct/array return
/// values leave a heap object ID (>= HEAP_OBJECT_BASE = 4_000_000) on the stack
/// as an i32. The old serialization only checked `is_string`/`is_i32`/`is_null`,
/// so a struct became the bare number `"4000000"` and a `?T` None became `"null"`.
///
/// This function recognizes heap object IDs and recursively expands them:
/// - `GenericInstanceData` (user structs) → `{"field": value, ...}`
/// - `Vec<Value>` (array literals `[...]`) → `[v1, v2, ...]`
/// - Option `Some(x)` → the inner value's JSON; `None` → HTTP caller maps to 404
///
/// `depth` guards against cyclic references (objects referencing each other).
pub fn nv_to_json(
    vm: &crate::vm::engine::AutoVM,
    nv: auto_val::NanoValue,
    depth: u32,
) -> Option<String> {
    const MAX_DEPTH: u32 = 32;

    // Tagged string (the canonical handler-returns-string path)
    if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv);
        let s = vm
            .strings
            .read()
            .unwrap()
            .get(idx as usize)
            .map(|b| String::from_utf8_lossy(b).to_string())?;
        return Some(json_escape_string(&s));
    }
    // f64 (not nanboxed as i32)
    if auto_val::is_f64(nv) {
        return Some(format_f64_json(auto_val::decode_f64(nv)));
    }
    if auto_val::is_f32(nv) {
        return Some(format_f64_json(auto_val::decode_f32(nv) as f64));
    }
    if auto_val::is_bool(nv) {
        return Some(if auto_val::decode_bool(nv) {
            "true".to_string()
        } else {
            "false".to_string()
        });
    }
    if auto_val::is_null(nv) {
        return Some("null".to_string());
    }
    // Tagged object/list (formal TAG_OBJECT / TAG_LIST)
    if auto_val::is_object(nv) {
        let id = auto_val::decode_object(nv) as u64;
        return heap_object_to_json(vm, id, depth);
    }
    if auto_val::is_list(nv) {
        let id = auto_val::decode_list(nv) as u64;
        return heap_object_to_json(vm, id, depth);
    }
    // PLAN-734 T-03（E3 修复）：i64/u64/BigInt 全域——48 位内联与堆装箱均按
    // 数值序列化（此前无臂落 null——i64 响应体损坏为 "null"）。
    if auto_val::is_i64(nv) {
        return Some(auto_val::decode_i64(nv).to_string());
    }
    if auto_val::is_u64(nv) {
        return Some(auto_val::decode_u64(nv).to_string());
    }
    if auto_val::is_bigint(nv) {
        let id = auto_val::decode_bigint_handle(nv) as u64;
        if let Some(obj) = vm.get_heap_object(id) {
            if let Ok(g) = obj.read() {
                if let Some(b) = g
                    .as_any()
                    .downcast_ref::<crate::vm::heap_object::BigIntData>()
                {
                    return Some(if b.is_unsigned {
                        format!("{}", b.as_u64())
                    } else {
                        format!("{}", b.as_i64())
                    });
                }
            }
        }
        return Some("0".to_string());
    }
    // i32: either a plain integer OR a heap/array object ID stored as i32.
    // Heap object ids start at 4_000_000 (heap_object_id_gen); array literals
    // are ListData<Value> in heap_objects too (Plan 390 §15 H3b). Rather than
    // assume a range (which could misclassify large user integers), we probe
    // the VM tables: if the value is a known heap/object id, expand it;
    // otherwise treat as a plain int.
    if auto_val::is_i32(nv) {
        let v = auto_val::decode_i32(nv);
        if depth < MAX_DEPTH {
            let id = v as u64;
            if vm.heap_objects.contains_key(&id) {
                if let Some(json) = heap_object_to_json(vm, id, depth) {
                    return Some(json);
                }
            }
        }
        return Some(v.to_string());
    }
    Some("null".to_string())
}

/// Expand a heap object ID into JSON. Handles the storage used by the VM:
/// `heap_objects` (GenericInstanceData, ListData<Value>/<i32> collections,
/// Node) and `objects` (ObjectData maps).
///
/// Option handling: a `GenericInstanceData` whose mono_name starts with
/// "Option.Some" is unwrapped to its single inner field; "Option.None"
/// yields `None` (the HTTP layer maps this to 404).
fn heap_object_to_json(vm: &crate::vm::engine::AutoVM, id: u64, depth: u32) -> Option<String> {
    use crate::vm::generic_registry::GenericInstanceData;

    // 1. heap_objects: GenericInstanceData (user-defined struct instances)
    if let Some(obj) = vm.get_heap_object(id) {
        let guard = obj.read().unwrap();
        if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
            // Option unwrapping: Some(x) → inner value JSON; None → JSON null.
            // (Plan 326: we serialize Option.None as `null` rather than 404 to
            //  keep the JSON response well-formed. A 404 mapping can be layered
            //  on later by the HTTP status branch if desired.)
            if inst.mono_name.starts_with("Option.Some") {
                if let Some(inner) = inst.get_field(0) {
                    return value_to_json(vm, &inner, depth + 1);
                }
                return Some("null".to_string());
            }
            if inst.mono_name.starts_with("Option.None") || inst.mono_name == "Option.None" {
                return Some("null".to_string());
            }
            // Regular struct: {"field": value, ...}
            let mut parts: Vec<String> = Vec::new();
            for (i, field_name) in inst.field_names.iter().enumerate() {
                if let Some(field_val) = inst.get_field(i) {
                    let val_json = value_to_json(vm, &field_val, depth + 1)
                        .unwrap_or_else(|| "null".to_string());
                    parts.push(format!("{}: {}", json_escape_string(field_name), val_json));
                }
            }
            return Some(format!("{{{}}}", parts.join(", ")));
        }
        // Plan 346: ListData<Value> (List<T>.new(...) collections).
        if let Some(list) = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
        {
            let mut parts: Vec<String> = Vec::new();
            for elem in &list.elems {
                let json = value_to_json(vm, elem, depth + 1).unwrap_or_else(|| "null".to_string());
                parts.push(json);
            }
            return Some(format!("[{}]", parts.join(", ")));
        }
        // ListData<i32> (int collections, or struct lists where elements are
        // stored as heap object IDs >= 4000000).
        if let Some(list) = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<i32>>()
        {
            let mut parts: Vec<String> = Vec::new();
            for &i in &list.elems {
                if i >= 4_000_000 {
                    // Heap object ID — expand recursively.
                    let json = heap_object_to_json(vm, i as u64, depth + 1)
                        .unwrap_or_else(|| i.to_string());
                    parts.push(json);
                } else {
                    parts.push(i.to_string());
                }
            }
            return Some(format!("[{}]", parts.join(", ")));
        }
        // Plan 390 §15 H3b: ObjectData (obj literals { k: v }) in heap_objects.
        if let Some(od) = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ObjectData>()
        {
            let mut parts: Vec<String> = Vec::new();
            for (key, val) in od.fields.iter() {
                let key_json = json_escape_string(&key.to_string());
                let val_json =
                    value_to_json(vm, val, depth + 1).unwrap_or_else(|| "null".to_string());
                parts.push(format!("{}: {}", key_json, val_json));
            }
            return Some(format!("{{{}}}", parts.join(", ")));
        }
        // Other heap objects (opaque types) — can't serialize generically.
        return None;
    }

    None
}

/// Serialize a `Value` (the enum used inside arrays / struct fields) to JSON.
/// Struct/array `Value`s carry heap object IDs in `Value::Int` (>= 4_000_000),
/// which we re-dispatch through `heap_object_to_json`.
fn value_to_json(
    vm: &crate::vm::engine::AutoVM,
    value: &auto_val::Value,
    depth: u32,
) -> Option<String> {
    use auto_val::Value;
    const MAX_DEPTH: u32 = 32;
    if depth >= MAX_DEPTH {
        return Some("null".to_string());
    }
    match value {
        Value::Int(i) => {
            // Probe the VM tables to decide: heap id → expand, else plain int.
            let id = *i as u64;
            if vm.heap_objects.contains_key(&id) {
                if let Some(json) = heap_object_to_json(vm, id, depth) {
                    return Some(json);
                }
            }
            Some(i.to_string())
        }
        Value::Uint(u) => Some(u.to_string()),
        Value::I8(i) => Some(i.to_string()),
        Value::U8(u) => Some(u.to_string()),
        Value::I64(i) => Some(i.to_string()),
        Value::Byte(b) => Some(b.to_string()),
        Value::USize(u) => Some(u.to_string()),
        Value::Bool(b) => Some(if *b {
            "true".to_string()
        } else {
            "false".to_string()
        }),
        Value::Float(f) | Value::Double(f) => Some(format_f64_json(*f)),
        Value::Char(c) => Some(json_escape_string(&c.to_string())),
        Value::Str(s) => Some(json_escape_string(&s.to_string())),
        Value::String(s) => Some(json_escape_string(&s.to_string())),
        Value::StrSlice(s) => Some(json_escape_string(&s.to_string())),
        Value::CStr(s) => Some(json_escape_string(s.as_str())),
        Value::Nil | Value::Null => Some("null".to_string()),
        Value::VmRef(r) => heap_object_to_json(vm, r.id as u64, depth),
        // Fallback: render as null rather than crashing the HTTP response.
        _ => Some("null".to_string()),
    }
}

/// Escape a string as a JSON string literal (with surrounding quotes).
pub(crate) fn json_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Format an f64 as JSON (integers without trailing .0, per JSON convention the
/// number is still valid; we keep the natural Rust representation).
fn format_f64_json(f: f64) -> String {
    if f.is_nan() || f.is_infinite() {
        "null".to_string()
    } else if f.fract() == 0.0 && f.abs() < 1e16 {
        format!("{}", f as i64)
    } else {
        format!("{}", f)
    }
}

// =============================================================================
// Plan 326 Phase 3: serialization unit tests
// =============================================================================
#[cfg(test)]
mod plan326_tests {
    use super::{format_f64_json, json_escape_string};

    #[test]
    fn json_escape_basic() {
        assert_eq!(json_escape_string("hello"), r#""hello""#);
    }
    #[test]
    fn json_escape_quotes_and_backslash() {
        assert_eq!(json_escape_string(r#"a"b\c"#), r#""a\"b\\c""#);
    }

    #[test]
    fn json_escape_control_chars() {
        assert_eq!(json_escape_string("a\nb\tc"), r#""a\nb\tc""#);
    }

    #[test]
    fn json_escape_unicode_control() {
        // 0x01 is a control char → \u0001
        assert_eq!(json_escape_string("\u{0001}"), r#""\u0001""#);
    }

    #[test]
    fn f64_integer_no_trailing_dot() {
        assert_eq!(format_f64_json(42.0), "42");
        assert_eq!(format_f64_json(-7.0), "-7");
    }

    #[test]
    fn f64_fractional_preserved() {
        assert_eq!(format_f64_json(3.14), "3.14");
    }

    #[test]
    fn f64_nan_and_inf_become_null() {
        assert_eq!(format_f64_json(f64::NAN), "null");
        assert_eq!(format_f64_json(f64::INFINITY), "null");
        assert_eq!(format_f64_json(f64::NEG_INFINITY), "null");
    }

    /// Verify the probe-based id detection: a small plain int (not in any VM
    /// table) must serialize as a plain number, never as an object/array.
    #[test]
    fn plain_int_not_treated_as_id() {
        let vm = fresh_vm();
        // 999999 is below all VM id bases and not inserted anywhere.
        let nv = auto_val::encode_i32(999999);
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some("999999".to_string()));
    }

    // ---------------------------------------------------------------------
    // VM-backed integration tests: construct a real AutoVM, insert objects,
    // and verify nv_to_json expands them correctly.
    // ---------------------------------------------------------------------

    use crate::vm::engine::AutoVM;
    use crate::vm::generic_registry::GenericInstanceData;
    use crate::vm::virt_memory::VirtualFlash;

    fn fresh_vm() -> AutoVM {
        // Empty flash is fine — nv_to_json only touches heap_objects/arrays/
        // objects/string pool, none of which need compiled code.
        let flash = VirtualFlash::new_with_code(vec![]);
        AutoVM::new(flash, 1024)
    }

    /// Plan 317 §11 Phase 11 (Bug fix regression): `get_fn_n_args` must read the
    /// declared parameter count from a function's FN_PROLOG. This is the
    /// primitive `build_handler_args` uses to decide whether to push the
    /// cookies/auth metadata — the root cause of the `e2e_int_path_param` /
    /// `e2e_notes_crud` failures (1-param handlers received the metadata JSON as
    /// their first arg). This test pins the lookup without needing a live HTTP
    /// server, so it runs in the regular (non-`--ignored`) suite.
    #[test]
    fn get_fn_n_args_reads_declared_arity() {
        use crate::vm::opcode::OpCode;
        // Bytecode for two exported "functions":
        //   - "echo_id" at addr 0: FN_PROLOG, n_args=1, n_locals=0, RET
        //   - "create_note" at addr 4: FN_PROLOG, n_args=2, n_locals=0, RET
        let bytecode: Vec<u8> = vec![
            OpCode::FN_PROLOG as u8,
            1,
            0,
            OpCode::RET as u8, // echo_id (1 param)
            OpCode::FN_PROLOG as u8,
            2,
            0,
            OpCode::RET as u8, // create_note (2 params)
        ];
        let mut flash = VirtualFlash::new_with_code(bytecode);
        flash.exports_by_name.insert("echo_id".to_string(), 0);
        flash.exports_by_name.insert("create_note".to_string(), 4);
        let vm = AutoVM::new(flash, 1024);

        assert_eq!(
            vm.get_fn_n_args("echo_id"),
            Some(1),
            "1-param handler arity"
        );
        assert_eq!(
            vm.get_fn_n_args("create_note"),
            Some(2),
            "2-param handler arity"
        );
        assert_eq!(vm.get_fn_n_args("nonexistent"), None, "unknown fn -> None");
    }

    #[test]
    fn nv_to_json_plain_int() {
        let vm = fresh_vm();
        let nv = auto_val::encode_i32(42);
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some("42".to_string()));
    }

    #[test]
    fn nv_to_json_string() {
        let vm = fresh_vm();
        let idx = {
            let mut strings = vm.strings.write().unwrap();
            strings.push(b"hello".to_vec());
            strings.len() - 1
        };
        let nv = auto_val::encode_string(idx as u32);
        assert_eq!(
            super::nv_to_json(&vm, nv, 0),
            Some(r#""hello""#.to_string())
        );
    }

    #[test]
    fn nv_to_json_null() {
        let vm = fresh_vm();
        let nv = auto_val::encode_null();
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some("null".to_string()));
    }

    /// Struct return: the handler leaves a heap object ID (>= 4_000_000) on the
    /// stack as i32. nv_to_json must expand it into {"field": value, ...}.
    #[test]
    fn nv_to_json_struct_expansion() {
        let vm = fresh_vm();
        let inst = GenericInstanceData::new_with_names(
            "Note".to_string(),
            vec![
                auto_val::Value::Int(1),
                auto_val::Value::Str(auto_val::AutoStr::from("hello")),
            ],
            vec!["id".to_string(), "title".to_string()],
        );
        let id = vm.insert_heap_object(inst);
        // The handler return path pushes this id as i32 (see CONSTRUCT_INSTANCE).
        let nv = auto_val::encode_i32(id as i32);
        let json = super::nv_to_json(&vm, nv, 0).unwrap();
        assert_eq!(json, r#"{"id": 1, "title": "hello"}"#);
    }

    /// Array of structs: the handler returns Vec<Value> where each element is
    /// a struct stored as Value::Int(heap_id). Array id is allocated the same
    /// way CREATE_ARRAY does (Plan 390 §15 H3b: ListData<Value> in
    /// heap_objects). nv_to_json must recurse.
    #[test]
    fn nv_to_json_array_of_structs() {
        let vm = fresh_vm();
        let a = GenericInstanceData::new_with_names(
            "Note".to_string(),
            vec![
                auto_val::Value::Int(0),
                auto_val::Value::Str(auto_val::AutoStr::from("a")),
            ],
            vec!["id".to_string(), "title".to_string()],
        );
        let b = GenericInstanceData::new_with_names(
            "Note".to_string(),
            vec![
                auto_val::Value::Int(1),
                auto_val::Value::Str(auto_val::AutoStr::from("b")),
            ],
            vec!["id".to_string(), "title".to_string()],
        );
        let id_a = vm.insert_heap_object(a) as i32;
        let id_b = vm.insert_heap_object(b) as i32;
        // Allocate an array id the same way CREATE_ARRAY does now.
        let arr_id = vm.insert_heap_object(crate::vm::types::ListData {
            elems: vec![auto_val::Value::Int(id_a), auto_val::Value::Int(id_b)],
            storage: None,
        });
        // The handler returns the array id as i32.
        let nv = auto_val::encode_i32(arr_id as i32);
        let json = super::nv_to_json(&vm, nv, 0).unwrap();
        assert_eq!(
            json,
            r#"[{"id": 0, "title": "a"}, {"id": 1, "title": "b"}]"#
        );
    }

    /// Option.Some(x) → unwrap to inner value's JSON.
    #[test]
    fn nv_to_json_option_some() {
        let vm = fresh_vm();
        let inst = GenericInstanceData::new_with_names(
            "Option.Some".to_string(),
            vec![auto_val::Value::Str(auto_val::AutoStr::from("found"))],
            vec!["_0".to_string()],
        );
        let id = vm.insert_heap_object(inst);
        let nv = auto_val::encode_i32(id as i32);
        assert_eq!(
            super::nv_to_json(&vm, nv, 0),
            Some(r#""found""#.to_string())
        );
    }

    /// Option.None → JSON null (Plan 326: we serialize None as `null` to keep
    /// the JSON response well-formed; a 404 mapping can be layered on later).
    #[test]
    fn nv_to_json_option_none() {
        let vm = fresh_vm();
        let inst = GenericInstanceData::new_with_names("Option.None".to_string(), vec![], vec![]);
        let id = vm.insert_heap_object(inst);
        let nv = auto_val::encode_i32(id as i32);
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some("null".to_string()));
    }

    /// Nested struct: a field whose value is itself a struct (VmRef / Int heap-id).
    #[test]
    fn nv_to_json_nested_struct() {
        let vm = fresh_vm();
        let inner = GenericInstanceData::new_with_names(
            "Point".to_string(),
            vec![auto_val::Value::Int(3), auto_val::Value::Int(4)],
            vec!["x".to_string(), "y".to_string()],
        );
        let inner_id = vm.insert_heap_object(inner) as i32;
        let outer = GenericInstanceData::new_with_names(
            "Box".to_string(),
            vec![auto_val::Value::Int(inner_id)],
            vec!["p".to_string()],
        );
        let outer_id = vm.insert_heap_object(outer);
        let nv = auto_val::encode_i32(outer_id as i32);
        let json = super::nv_to_json(&vm, nv, 0).unwrap();
        assert_eq!(json, r#"{"p": {"x": 3, "y": 4}}"#);
    }

    // ---------------------------------------------------------------------
    // PLAN-669: #[api] param-sig side channel (registry round-trip; the
    // codegen publish site is covered end-to-end by the http_e2e battery).
    // ---------------------------------------------------------------------
    #[test]
    fn plan669_api_param_sigs_roundtrip() {
        use super::{api_param_sigs, clear_api_param_sigs, record_api_param_sigs, ApiParamSig};
        clear_api_param_sigs();
        record_api_param_sigs(
            "create_note",
            vec![
                ApiParamSig {
                    name: "title".into(),
                    ty: "str".into(),
                },
                ApiParamSig {
                    name: "id".into(),
                    ty: "int".into(),
                },
            ],
        );
        let sigs = api_param_sigs("create_note").expect("recorded");
        assert_eq!(sigs.len(), 2);
        assert_eq!(sigs[0].name, "title");
        assert_eq!(sigs[1].ty, "int");
        clear_api_param_sigs();
        assert!(api_param_sigs("create_note").is_none());
    }

    // ---------------------------------------------------------------------
    // PLAN-669: by-name binder unit matrix (no live server — the marshalled
    // stack values are inspected directly; e2e shapes run in http_e2e).
    // ---------------------------------------------------------------------
    mod plan669_bind_tests {
        use super::super::{bind_api_args_by_name, ApiArgBindError, ApiParamSig, RouteMatch};
        use crate::vm::engine::AutoVM;
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;

        fn sig(name: &str, ty: &str) -> ApiParamSig {
            ApiParamSig {
                name: name.into(),
                ty: ty.into(),
            }
        }
        fn rm(path_params: Vec<(&str, &str)>, query_params: Vec<(&str, &str)>) -> RouteMatch {
            RouteMatch {
                fn_name: "h".into(),
                path_params: path_params
                    .into_iter()
                    .map(|(n, v)| (n.to_string(), v.to_string()))
                    .collect(),
                query_params: query_params
                    .into_iter()
                    .map(|(n, v)| (n.to_string(), v.to_string()))
                    .collect(),
            }
        }
        fn rig() -> (AutoVM, AutoTask) {
            (
                AutoVM::new(VirtualFlash::new_with_code(vec![]), 1024),
                AutoTask::new(0, 4096, 0),
            )
        }
        fn pop_str(vm: &AutoVM, task: &mut AutoTask) -> String {
            let nv = task.ram.pop_nv();
            assert!(auto_val::is_string(nv), "expected string nv, got {nv:?}");
            let idx = auto_val::decode_string(nv);
            vm.strings
                .read()
                .unwrap()
                .get(idx as usize)
                .map(|b| String::from_utf8_lossy(b).to_string())
                .expect("string pool entry")
        }

        #[test]
        fn body_fields_bind_by_name_in_declaration_order() {
            let (vm, mut task) = rig();
            let body: serde_json::Value =
                serde_json::from_str(r#"{"body":"second","title":"first"}"#).unwrap();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("title", "str"), sig("body", "str")],
                &rm(vec![], vec![]),
                Some(&body),
                "",
                None,
                "POST",
                "/api/notes",
                None,
            )
            .expect("bind");
            assert_eq!(n, 2);
            // LIFO: last-pushed pops first (body was pushed second).
            assert_eq!(pop_str(&vm, &mut task), "second");
            assert_eq!(pop_str(&vm, &mut task), "first");
        }

        #[test]
        fn query_binds_by_name() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("query", "str")],
                &rm(vec![], vec![("q", "ignored"), ("query", "Build")]),
                None,
                "",
                None,
                "GET",
                "/api/search",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            assert_eq!(pop_str(&vm, &mut task), "Build");
        }

        #[test]
        fn typed_int_query_converts_and_rejects() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("page", "int")],
                &rm(vec![], vec![("page", "7")]),
                None,
                "",
                None,
                "GET",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            let nv = task.ram.pop_nv();
            assert!(
                auto_val::is_i32(nv) || auto_val::decode_i32(nv) == 7,
                "int nv {nv:?}"
            );

            let (vm2, mut task2) = rig();
            let err = bind_api_args_by_name(
                &vm2,
                &mut task2,
                &[sig("page", "int")],
                &rm(vec![], vec![("page", "seven")]),
                None,
                "",
                None,
                "GET",
                "/x",
                None,
            )
            .unwrap_err();
            match err {
                ApiArgBindError::BadRequest(m) => {
                    assert!(m.contains("page") && m.contains("int"), "{m}");
                }
                other => panic!("expected BadRequest, got {other:?}"),
            }
        }

        #[test]
        fn bool_query_converts() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("done", "bool")],
                &rm(vec![], vec![("done", "true")]),
                None,
                "",
                None,
                "GET",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            let nv = task.ram.pop_nv();
            assert!(
                auto_val::is_bool(nv) && auto_val::decode_bool(nv),
                "bool nv {nv:?}"
            );
        }

        #[test]
        fn path_wins_over_body_and_query() {
            let (vm, mut task) = rig();
            let body: serde_json::Value = serde_json::from_str(r#"{"id": 999}"#).unwrap();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("id", "int")],
                &rm(vec![("id", "42")], vec![("id", "7")]),
                Some(&body),
                "",
                None,
                "GET",
                "/api/notes/42",
                None,
                        )
            .expect("bind");
            assert_eq!(n, 1);
            let nv = task.ram.pop_nv();
            assert!(
                auto_val::decode_i32(nv) == 42,
                "path source must win, nv {nv:?}"
            );
        }

        #[test]
        fn missing_param_is_400_naming_param() {
            let (vm, mut task) = rig();
            let err = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("title", "str"), sig("body", "str")],
                &rm(vec![], vec![]),
                None,
                "",
                None,
                "POST",
                "/api/notes",
                None,
            )
            .unwrap_err();
            match err {
                ApiArgBindError::BadRequest(m) => {
                    assert!(m.contains("missing param `title`"), "{m}");
                }
                other => panic!("expected BadRequest, got {other:?}"),
            }
        }

        #[test]
        fn trailing_unbound_param_receives_metadata() {
            let (vm, mut task) = rig();
            let body: serde_json::Value = serde_json::from_str(r#"{"title":"t"}"#).unwrap();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("title", "str"), sig("meta", "str")],
                &rm(vec![], vec![]),
                Some(&body),
                "",
                Some(r#"{"cookies":{}}"#),
                "POST",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 2);
            assert_eq!(pop_str(&vm, &mut task), r#"{"cookies":{}}"#);
            assert_eq!(pop_str(&vm, &mut task), "t");
        }

        #[test]
        fn non_trailing_unbound_is_missing_even_with_metadata() {
            let (vm, mut task) = rig();
            let err = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("a", "str"), sig("meta", "str")],
                &rm(vec![], vec![]),
                None,
                "",
                Some("{}"),
                "GET",
                "/x",
                None,
            )
            .unwrap_err();
            assert!(matches!(err, ApiArgBindError::BadRequest(_)));
        }

        #[test]
        fn raw_body_single_param_tolerance() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("data", "str")],
                &rm(vec![], vec![]),
                None,
                "plain text not json",
                None,
                "POST",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            assert_eq!(pop_str(&vm, &mut task), "plain text not json");
        }

        #[test]
        fn whole_body_tolerance_covers_parseable_object_too() {
            // Multipart/B6-`form` shape: body parses as JSON but the lone
            // param's name isn't a field of it — legacy single-arg body
            // contract hands it the whole body source.
            let (vm, mut task) = rig();
            let body: serde_json::Value =
                serde_json::from_str(r#"{"fields":{"title":"hello"}}"#).unwrap();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("form", "str")],
                &rm(vec![], vec![]),
                Some(&body),
                r#"{"fields":{"title":"hello"}}"#,
                None,
                "POST",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 1);
            assert_eq!(pop_str(&vm, &mut task), r#"{"fields":{"title":"hello"}}"#);
        }

        #[test]
        fn metadata_optin_requires_meta_param_name() {
            // A trailing unbound param NOT named meta/metadata/req/request is
            // a missing param (400), never the cookies/auth payload — this is
            // what makes POSTs with a dropped last field fail loudly.
            let (vm, mut task) = rig();
            let body: serde_json::Value = serde_json::from_str(r#"{"title":"t"}"#).unwrap();
            let err = bind_api_args_by_name(
                &vm,
                &mut task,
                &[sig("title", "str"), sig("body", "str")],
                &rm(vec![], vec![]),
                Some(&body),
                r#"{"title":"t"}"#,
                Some("{}"),
                "POST",
                "/x",
                None,
            )
            .unwrap_err();
            match err {
                ApiArgBindError::BadRequest(m) => {
                    assert!(m.contains("missing param `body`"), "{m}");
                }
                other => panic!("expected BadRequest, got {other:?}"),
            }
        }

        #[test]
        fn empty_sigs_bind_zero() {
            let (vm, mut task) = rig();
            let n = bind_api_args_by_name(
                &vm,
                &mut task,
                &[],
                &rm(vec![], vec![]),
                None,
                "",
                None,
                "GET",
                "/x",
                None,
            )
            .expect("bind");
            assert_eq!(n, 0);
        }

        /// F-669-R1 / AC-06: with no API_PARAM_SIGS entry the sync-serve
        /// helper falls back to the pre-669 positional convention verbatim —
        /// path params in order (i32-heuristic), query ignored, raw body as
        /// one trailing arg. e2e always has sigs (codegen publishes them), so
        /// the fallback arm is only reachable here.
        #[test]
        fn legacy_fallback_without_sigs_is_positional() {
            use super::super::{bind_api_args_or_legacy, clear_api_param_sigs};
            clear_api_param_sigs();
            let (vm, mut task) = rig();
            let n = bind_api_args_or_legacy(
                &vm,
                &mut task,
                "h_legacy_never_published",
                &[
                    ("id".to_string(), "42".to_string()),
                    ("slug".to_string(), "hello".to_string()),
                ],
                &[("ignored".to_string(), "q".to_string())],
                "raw-body",
                "GET",
                "/api/notes/42/hello",
            )
            .expect("legacy bind");
            assert_eq!(n, 3, "id + slug + body, query dropped");
            // LIFO: body (last pushed) pops first, then slug, then id as i32.
            assert_eq!(pop_str(&vm, &mut task), "raw-body");
            assert_eq!(pop_str(&vm, &mut task), "hello");
            let nv = task.ram.pop_nv();
            assert!(
                auto_val::decode_i32(nv) == 42,
                "numeric path param as i32, nv {nv:?}"
            );
        }
    }

    // ---------------------------------------------------------------------
    // Plan 326 Phase 3 end-to-end: spawn the real AutoVM HTTP server with a
    // minimal #[api] program that returns a struct, then assert the HTTP
    // response body is well-formed JSON (not the bare heap-id "4000000").
    // ---------------------------------------------------------------------
    //
    // Plan 317 §11 Phase 11: these e2e tests start a real TCP HTTP server and
    // are gated behind the `test-http-e2e` feature (off by default). They must
    // run serially (`--test-threads=1`) because each server thread is detached
    // (runs until process exit) and they share process-global state
    // (AUTO_HTTP_PORT env var, HTTP_ROUTES table). Each test clears the global
    // route table first and binds a dynamic port to avoid cross-test
    // contamination. Run with:
    //   cargo test -p auto-lang --lib --features test-http-e2e -- --test-threads=1
    #[cfg(feature = "test-http-e2e")]
    mod http_e2e {
        use crate::vm::ffi::stdlib::clear_http_routes;
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::path::PathBuf;
        use std::time::Duration;

        /// Send a raw HTTP request to localhost:port and return the full response.
        fn http_get(port: u16, path: &str) -> String {
            // Retry-connect for up to ~5s while the server comes up.
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("could not connect to test HTTP server");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            write!(
                stream,
                "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
                path
            )
            .unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).ok();
            resp
        }

        /// Plan 346 B6: GET with extra request headers (e.g. X-Request-Id
        /// passthrough test).
        fn http_get_with_headers(port: u16, path: &str, headers: &[(&str, &str)]) -> String {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("could not connect to test HTTP server");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let mut req = format!(
                "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n",
                path
            );
            for (k, v) in headers {
                req.push_str(&format!("{}: {}\r\n", k, v));
            }
            req.push_str("\r\n");
            write!(stream, "{}", req).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).ok();
            resp
        }

        /// Audit B1: JSON POST with extra request headers (Authorization).
        fn http_post_json_with_headers(
            port: u16,
            path: &str,
            json_body: &str,
            headers: &[(&str, &str)],
        ) -> String {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("connect to test server");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let mut req = format!(
                "POST {} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
                path,
                json_body.len()
            );
            for (k, v) in headers {
                req.push_str(&format!("{}: {}\r\n", k, v));
            }
            req.push_str("\r\n");
            req.push_str(json_body);
            write!(stream, "{}", req).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).ok();
            resp
        }

        /// PLAN-696 T-02: POST a body after the headers in multiple TCP writes.
        /// The delays make the write boundaries observable to the server instead
        /// of letting the OS coalesce the whole request into one read.
        fn http_post_json_in_chunks(port: u16, path: &str, body: &str) -> String {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("connect to test server");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let head = format!(
                "POST {} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                path,
                body.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            std::thread::sleep(Duration::from_millis(40));

            let bytes = body.as_bytes();
            let cuts = [bytes.len() / 3, (bytes.len() * 2) / 3];
            let mut start = 0;
            for end in [cuts[0], cuts[1], bytes.len()] {
                if stream.write_all(&bytes[start..end]).is_err() {
                    break;
                }
                let _ = stream.flush();
                start = end;
                std::thread::sleep(Duration::from_millis(40));
            }
            let _ = stream.shutdown(std::net::Shutdown::Write);
            let mut resp = String::new();
            stream.read_to_string(&mut resp).ok();
            resp
        }

        /// Extract the body (after the blank line) from a raw HTTP response.
        fn body_of(resp: &str) -> &str {
            resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or(resp)
        }

        /// Spin up the AutoVM HTTP server for `code` on a fixed unique `port`,
        /// then block until the server is accepting connections (up to ~5s).
        ///
        /// Fixed ports (not dynamic `:0`) are used deliberately: each test gets
        /// a distinct port, and the OS reliably hands a connect back to the
        /// bound listener. Dynamic `:0` introduced a TOCTOU race (after dropping
        /// the probe listener, the OS could reassign the port to a prior test's
        /// detached server thread), which made `e2e_concurrent_sse` flaky.
        ///
        /// Clears the global route table first (isolation from prior tests'
        /// detached servers, which may still be reading the global HTTP_ROUTES
        /// snapshot). The server thread is detached (blocks forever; the test
        /// process reaps it on exit — fine because CI runs each `cargo test`
        /// invocation as a separate process).
        fn start_server(code: &str, port: u16) -> u16 {
            clear_http_routes();
            std::env::set_var("AUTO_HTTP_PORT", port.to_string());
            let code = code.to_string();
            let _server = std::thread::Builder::new()
                .stack_size(8 * 1024 * 1024)
                .spawn(move || {
                    let _ = crate::run(&code);
                })
                .expect("spawn server thread");
            // Wait for the server to accept connections before returning, so the
            // first http_get doesn't race against a not-yet-bound listener.
            for _ in 0..50 {
                if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            port
        }

        /// Start the real back/api.at with its sibling db.at resolved from the
        /// example directory, so Plan 696 exercises the runtime entry point
        /// against the same source the app uses.
        fn start_example_api_server(example: &str, port: u16) -> u16 {
            clear_http_routes();
            std::env::set_var("AUTO_HTTP_PORT", port.to_string());
            let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .expect("repo root")
                .to_path_buf();
            let entry = repo_root
                .join("examples/ui")
                .join(example)
                .join("src/back/api.at");
            let source = std::fs::read_to_string(&entry)
                .unwrap_or_else(|error| panic!("read {}: {error}", entry.display()));
            let source_path = entry.to_string_lossy().into_owned();
            let _server = std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn(move || {
                    let _ = crate::run_with_capture_and_path(&source, &source_path);
                })
                .expect("spawn real example API server");
            for _ in 0..50 {
                if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                    return port;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            panic!("{} VM API did not listen on port {port}", example);
        }

        /// P442-4: e2e 端口唯一性守卫。nextest 每测试一进程并行执行，两个
        /// 测试复用同一端口时，后绑定进程的请求会打到先绑定进程的服务器上
        /// （无该路由 → 连接被静默丢弃 → 空 body 假红）。历史三组撞号
        /// （Plan 442 §8.5）由本守卫防复发：扫描本文件全部端口字面量，任何
        /// 五位端口号后随右括号出现两次即红。新测试请选用未占用端口。
        #[test]
        fn e2e_ports_unique() {
            let src = include_str!("http_server.rs");
            let mut seen: std::collections::BTreeMap<&str, usize> = Default::default();
            let mut dup: Vec<&str> = Vec::new();
            for (i, line) in src.lines().enumerate() {
                let b = line.as_bytes();
                let mut j = 0;
                while j < b.len() {
                    if b[j].is_ascii_digit() {
                        let start = j;
                        while j < b.len() && b[j].is_ascii_digit() {
                            j += 1;
                        }
                        let run = &line[start..j];
                        let paren_next = j < b.len() && b[j] == b')';
                        if run.len() == 5 && run.starts_with("18") && paren_next {
                            if seen.insert(run, i + 1).is_some() {
                                dup.push(run);
                            }
                        }
                    } else {
                        j += 1;
                    }
                }
            }
            assert!(
                dup.is_empty(),
                "e2e 端口撞号（nextest 并行下输家连到赢家服务器假红）: {:?}",
                dup
            );
        }

        #[test]
        fn e2e_struct_handler_returns_json() {
            let port = start_server(
                r#"
type Note { id int; title str }

#[api(method = "GET", path = "/api/notes/test")]
fn get_note() Note {
    Note { id: 1, title: "hello" }
}
"#,
                18731,
            );
            let resp = http_get(port, "/api/notes/test");
            let body = body_of(&resp);
            // The fix: body must be JSON object, not the bare heap-id "4000000".
            assert_eq!(
                body, r#"{"id": 1, "title": "hello"}"#,
                "struct handler JSON: full resp = {:?}",
                resp
            );
        }

        #[test]
        fn e2e_int_path_param_handler() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/echo/:id")]
fn echo_id(id int) int {
    id
}
"#,
                18732,
            );
            let resp = http_get(port, "/api/echo/42");
            let body = body_of(&resp);
            // Phase 5: :id injected as int 42, returned as-is.
            assert_eq!(body, "42", "int path param: full resp = {:?}", resp);
        }

        /// Plan 317 Phase 3: SSE handler returning a generator (~Iter<int>).
        /// Each yield becomes an SSE data frame. Lazy evaluation means each
        /// next() runs only to the next yield (not the whole body upfront).
        #[test]
        fn e2e_sse_generator_handler() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/counter")]
fn counter_handler() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
"#,
                18733,
            );
            let resp = http_get(port, "/api/counter");
            // SSE response: the body should contain three "data: N\n\n" frames.
            let body = body_of(&resp);
            assert!(body.contains("data: 1"), "SSE frame 1: body={:?}", body);
            assert!(body.contains("data: 2"), "SSE frame 2: body={:?}", body);
            assert!(body.contains("data: 3"), "SSE frame 3: body={:?}", body);
        }

        /// Plan 317 Phase 3 遗留: SSE handler that INDIRECTLY calls a generator
        /// (handler itself has no yield; it calls a generator fn). The handler
        /// returns the iter_id from the inner generator; SSE detection must still
        /// fire on that iter_id.
        #[test]
        fn e2e_sse_indirect_generator() {
            let port = start_server(
                r#"
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
#[api(method = "GET", path = "/api/stream")]
fn stream_handler() ~Iter<int> {
    return counter()
}
"#,
                18734,
            );
            let resp = http_get(port, "/api/stream");
            let body = body_of(&resp);
            assert!(
                body.contains("data: 1"),
                "indirect SSE frame 1: body={:?}",
                body
            );
            assert!(
                body.contains("data: 2"),
                "indirect SSE frame 2: body={:?}",
                body
            );
            assert!(
                body.contains("data: 3"),
                "indirect SSE frame 3: body={:?}",
                body
            );
        }

        /// Plan 442 C2 item ②: the SSE form — a generator yielding
        /// `sse_named_event(name, payload)` objects must stream proper
        /// `event: <name>\ndata: <payload>` frames (not the raw heap handle).
        /// The bare `sse_named_event` is resolved to a VM shim which builds an
        /// opaque Event object; `sse_frame_from_nv` formats it.
        #[test]
        fn e2e_sse_named_event_frames() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/events")]
fn events_handler() ~Iter<int> {
    yield sse_named_event("e1", "hello")
    yield sse_named_event("e2", "world")
}
"#,
                18738,
            );
            let resp = http_get(port, "/api/events");
            let body = body_of(&resp);
            assert!(
                body.contains("event: e1\ndata: \"hello\""),
                "named SSE frame 1: body={:?}",
                body
            );
            assert!(
                body.contains("event: e2\ndata: \"world\""),
                "named SSE frame 2: body={:?}",
                body
            );
        }

        /// Plan 442 C2 item ②: the full `Sse.new(stream).keep_alive(
        /// KeepAlive.new()).into_response()` chain — `into_response()` returns
        /// the generator's iterator id, so the server's iterator→SSE branch
        /// streams the yielded Event frames through the dispatch-3000 SSE arms.
        #[test]
        fn e2e_sse_chain() {
            let port = start_server(
                r#"
dep axum
use.rs axum::response::sse::{Event, KeepAlive, Sse}

#[api(method = "GET", path = "/api/sse-chain")]
fn chain_handler() int {
    var sse = Sse.new(events_stream())
    return sse.keep_alive(KeepAlive.new()).into_response()
}

fn events_stream() ~Iter<int> {
    yield sse_named_event("tick", "42")
}
"#,
                18746,
            );
            let resp = http_get(port, "/api/sse-chain");
            let body = body_of(&resp);
            assert!(
                body.contains("event: tick\ndata: \"42\""),
                "Sse chain SSE frame: body={:?}",
                body
            );
        }

        /// Plan 442 C2 item ③ path (b): pure-logic value-accessor externs
        /// (`value_get_str`/`value_get_bool`/`value_is_null`) read fields out of
        /// a `Value` built by `json.to_value`. They resolve as bare natives.
        #[test]
        fn e2e_value_accessors() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/acc")]
fn acc() str {
    let obj = json.to_value("{\"msg\":\"hi\",\"ok\":true}")
    let s = value_get_str(obj.view, "msg")
    let b = value_get_bool(obj.view, "ok")
    if b {
        return s
    } else {
        return "no"
    }
}

#[api(method = "GET", path = "/isnull")]
fn isnull() str {
    let obj = json.to_value("null")
    if value_is_null(obj.view) {
        return "yes"
    } else {
        return "no"
    }
}

#[api(method = "GET", path = "/arr")]
fn arrrt() int {
    let obj = json.to_value("{\"items\":[1,2,3]}")
    return ok_response(value_get_array(obj.view, "items"))
}

#[api(method = "GET", path = "/nested")]
fn nested() int {
    let obj = json.to_value("{\"nested\":{\"x\":7}}")
    return ok_response(value_get(obj.view, "nested"))
}

#[api(method = "GET", path = "/hex")]
fn hex() str {
    return random_hex(4)
}

#[api(method = "GET", path = "/newid")]
fn newid() str {
    return new_id(8)
}

#[api(method = "GET", path = "/hash")]
fn hash() str {
    return hash_password("p", "salt")
}

#[api(method = "GET", path = "/path")]
fn path() str {
    return path_inner("seg/ment")
}
"#,
                18747,
            );

            let acc = http_get(port, "/acc");
            assert!(
                acc.starts_with("HTTP/1.1 200"),
                "value accessor status: {}",
                acc.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&acc),
                "\"hi\"",
                "value_get_str/value_get_bool: full = {:?}",
                acc
            );

            let isnull = http_get(port, "/isnull");
            assert_eq!(
                body_of(&isnull),
                "\"yes\"",
                "value_is_null on JSON null: full = {:?}",
                isnull
            );

            let arr = http_get(port, "/arr");
            assert_eq!(
                body_of(&arr),
                "[1, 2, 3]",
                "value_get_array: full = {:?}",
                arr
            );

            let nested = http_get(port, "/nested");
            assert_eq!(
                body_of(&nested),
                r#"{"x": 7}"#,
                "value_get nested object: full = {:?}",
                nested
            );

            // random_hex/new_id return a hex string of the request length.
            let h = body_of(&http_get(port, "/hex"))
                .trim_matches('"')
                .to_string();
            assert_eq!(h.len(), 8, "random_hex(4) should be 8 hex chars: {:?}", h);
            assert!(
                h.chars().all(|c| c.is_ascii_hexdigit()),
                "random_hex not hex: {:?}",
                h
            );

            let n = body_of(&http_get(port, "/newid"))
                .trim_matches('"')
                .to_string();
            assert_eq!(n.len(), 16, "new_id(8) should be 16 hex chars: {:?}", n);

            // hash_password = sha256(salt || p) hex (64 chars, deterministic).
            let h = body_of(&http_get(port, "/hash"))
                .trim_matches('"')
                .to_string();
            assert_eq!(h.len(), 64, "hash_password should be 64 hex chars: {:?}", h);
            assert!(
                h.chars().all(|c| c.is_ascii_hexdigit()),
                "hash_password not hex: {:?}",
                h
            );

            // path_inner on a string is identity (axum Path pushes a string).
            assert_eq!(
                body_of(&http_get(port, "/path")),
                "\"seg/ment\"",
                "path_inner identity: full = {:?}",
                http_get(port, "/path")
            );
        }

        /// Plan 442 C2 item ③ path (a): a data-source extern
        /// (`app_config_effective_daemon_url`) forwards to a registered host call
        /// (`host_bridge`) when one is present, else serves a default constant.
        /// Proves the extern→backend routing mechanism works when a backend
        /// registers the call (auto-musk cdylib). String-returning → no nested
        /// object RC pitfalls.
        #[test]
        fn e2e_host_forward_app_config_daemon() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/daemon")]
fn daemon() str {
    return app_config_effective_daemon_url(0)
}
"#,
                18748,
            );

            // Phase 1: no host call registered → default constant.
            let d = http_get(port, "/daemon");
            assert_eq!(
                body_of(&d),
                "\"http://127.0.0.1:17654\"",
                "app_config_effective_daemon_url default: full = {:?}",
                d
            );

            // Phase 2: register a host call → the VM extern forwards to it.
            crate::vm::host_bridge::register_host_call(
                "app_config_effective_daemon_url",
                std::sync::Arc::new(move |_args: &str| -> Result<String, String> {
                    Ok(r#""http://10.0.0.5:9999""#.to_string())
                }),
            );
            let f = http_get(port, "/daemon");
            assert_eq!(
                body_of(&f),
                "\"http://10.0.0.5:9999\"",
                "app_config_effective_daemon_url host-forwarded: full = {:?}",
                f
            );
        }

        /// Plan 442 C2 item ③ path (a): a Value-returning data extern
        /// (`relay_runs_list`) forwards to a registered host call, else serves a
        /// default. Exercises the heap-ref dead-zone compensation: the fresh
        /// `{runs: []}` object survives the CALL_NAT dead-zone release.
        #[test]
        fn e2e_host_forward_relay_runs() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/runs")]
fn runs() int {
    return ok_response(relay_runs_list(0, 0))
}
"#,
                18749,
            );

            // Phase 1: no host call → empty-store default shape.
            let d = http_get(port, "/runs");
            assert_eq!(
                body_of(&d),
                r#"{"runs": []}"#,
                "relay_runs_list default (empty store): full = {:?}",
                d
            );

            // Phase 2: register host call → the VM extern forwards to it,
            // passing the (marshalled) request args in args_json.
            crate::vm::host_bridge::register_host_call(
                "relay_runs_list",
                std::sync::Arc::new(move |args: &str| -> Result<String, String> {
                    Ok(format!(r#"{{"runs":[{{"run_id":"r1","arg":{}}}]}}"#, args))
                }),
            );
            let f = http_get(port, "/runs");
            let fbody = body_of(&f);
            // The shim marshalled q (an int 0 in this test) → "0", and forwarded it.
            assert!(
                fbody.contains(r#""run_id": "r1""#) && fbody.contains(r#""arg": 0"#),
                "relay_runs_list host-forwarded with args: full = {:?}",
                fbody
            );
        }

        /// Fetch `path` repeatedly until the body contains all `need_fragments`,
        /// or `max_attempts` is exhausted. Returns the last body.
        ///
        /// The concurrent SSE test fires two simultaneous connections at the
        /// single-worker `serve_async`. The two `spawn_local` handler tasks
        /// interleave via `yield_now`, but on a loaded machine one connection's
        /// SSE stream can be cut short by the client read-timeout (5s) before
        /// all frames flush — a real nondeterminism in the cooperative schedule,
        /// not a server bug. Since the endpoint is an idempotent GET, retrying
        /// the connection is a faithful client behavior and removes the test's
        /// dependence on scheduler timing.
        fn http_get_until(
            port: u16,
            path: &str,
            need_fragments: &[&str],
            max_attempts: usize,
        ) -> String {
            let mut last_body = String::new();
            for _ in 0..max_attempts {
                let resp = http_get(port, path);
                last_body = body_of(&resp).to_string();
                if need_fragments.iter().all(|f| last_body.contains(f)) {
                    return last_body;
                }
            }
            last_body
        }

        /// Plan 317 Phase 4: concurrent SSE — two simultaneous connections to the
        /// same streaming endpoint. Both must receive complete data. Under the old
        /// serial server, the second connection would block until the first's
        /// generator exhausted. With serve_async + spawn_local + yield_now, the
        /// two handlers interleave (Goroutine-style cooperative scheduling).
        #[test]
        fn e2e_concurrent_sse() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/count")]
fn counter_handler() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
"#,
                18735,
            );

            let need = &["data: 1", "data: 2", "data: 3"];
            // Fire two connections concurrently from separate threads; each retries
            // its own connection until it sees all three frames (cooperative
            // scheduling can truncate a stream under load — see http_get_until).
            // Generous attempt count (8) tolerates accumulated detached-server
            // threads from prior tests slowing the tokio runtime within a suite.
            let h1 = std::thread::spawn(move || http_get_until(port, "/api/count", need, 8));
            let h2 = std::thread::spawn(move || http_get_until(port, "/api/count", need, 8));
            let body1 = h1.join().expect("conn1");
            let body2 = h2.join().expect("conn2");
            // Both connections must receive all three frames.
            assert!(
                body1.contains("data: 1") && body1.contains("data: 2") && body1.contains("data: 3"),
                "conn1 incomplete: body={:?}",
                body1
            );
            assert!(
                body2.contains("data: 1") && body2.contains("data: 2") && body2.contains("data: 3"),
                "conn2 incomplete: body={:?}",
                body2
            );
        }

        /// PLAN-696 T-02 red sample: ordinary request bodies must be read in
        /// full even when headers and body, and then body chunks, arrive in
        /// separate TCP writes. The JSON itself is pretty-printed and carries
        /// an escaped newline in a string value.
        #[test]
        fn e2e_plan696_body_split_across_tcp_writes() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/echo")]
fn echo(text str) str { text }
"#,
                18760,
            );
            let body = "{\n  \"text\": \"first\\nsecond\"\n}";
            let resp = http_post_json_in_chunks(port, "/api/echo", body);
            assert!(
                resp.starts_with("HTTP/1.1 200 OK"),
                "split body status: {:?}",
                resp
            );
            assert_eq!(
                body_of(&resp),
                r#""first\nsecond""#,
                "split body response: {:?}",
                resp
            );
        }

        /// PLAN-696 T-02 red sample: EOF before Content-Length is satisfied
        /// must reject the request before dispatching a body-independent route.
        #[test]
        fn e2e_plan696_short_body_rejected() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/ready")]
fn ready() str { "ready" }
"#,
                18761,
            );
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            stream.write_all(
                b"POST /api/ready HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 12\r\nConnection: close\r\n\r\nabc",
            ).unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).unwrap();
            assert!(
                resp.starts_with("HTTP/1.1 400"),
                "short body must be rejected, got: {:?}",
                resp
            );
        }

        /// PLAN-696 T-02: reject Content-Length above the shared request-body
        /// limit before allocating or dispatching the request.
        #[test]
        fn e2e_plan696_over_limit_body_rejected() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/ready")]
fn ready() str { "ready" }
"#,
                18763,
            );
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            stream.write_all(
                b"POST /api/ready HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 10485761\r\nConnection: close\r\n\r\n",
            ).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).unwrap();
            assert!(
                resp.starts_with("HTTP/1.1 413"),
                "oversize body must be rejected, got: {:?}",
                resp
            );
        }

        /// PLAN-696 T-03: reject malformed Content-Length instead of silently
        /// treating it as a bodyless request.
        #[test]
        fn e2e_plan696_invalid_content_length_rejected() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/ready")]
fn ready() str { "ready" }
"#,
                18764,
            );
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            stream.write_all(
                b"POST /api/ready HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: not-a-number\r\nConnection: close\r\n\r\n",
            ).unwrap();
            let mut resp = String::new();
            stream.read_to_string(&mut resp).unwrap();
            assert!(
                resp.starts_with("HTTP/1.1 400"),
                "malformed length must be rejected, got: {:?}",
                resp
            );
        }

        /// PLAN-696 T-02 red sample: a delayed generator must not keep the
        /// single-thread I/O reactor from serving an unrelated health request.
        #[test]
        fn e2e_plan696_slow_sse_does_not_block_health() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/slow")]
fn slow() ~Iter<int> {
    yield 1
    Time.sleep_ms(2500)
    yield 2
}
#[api(method = "GET", path = "/health")]
fn health() str { "ok" }
"#,
                18762,
            );

            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            stream
                .write_all(
                    b"GET /api/slow HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: keep-alive\r\n\r\n",
                )
                .unwrap();
            let mut received = Vec::new();
            let mut chunk = [0u8; 512];
            while !received
                .windows(b"data: 1\n\n".len())
                .any(|w| w == b"data: 1\n\n")
            {
                let n = stream.read(&mut chunk).expect("read first SSE frame");
                assert!(n > 0, "SSE closed before first frame: {:?}", received);
                received.extend_from_slice(&chunk[..n]);
            }
            let first_frame_at = std::time::Instant::now();

            let started = std::time::Instant::now();
            let health = http_get(port, "/health");
            let elapsed = started.elapsed();
            assert!(
                health.starts_with("HTTP/1.1 200"),
                "health response: {:?}",
                health
            );
            assert!(
                elapsed < Duration::from_millis(1500),
                "health request waited {elapsed:?} behind the slow SSE producer"
            );

            while !received
                .windows(b"data: 2\n\n".len())
                .any(|w| w == b"data: 2\n\n")
            {
                let n = stream.read(&mut chunk).expect("read second SSE frame");
                assert!(n > 0, "SSE closed before second frame: {:?}", received);
                received.extend_from_slice(&chunk[..n]);
            }
            let producer_delay = first_frame_at.elapsed();
            assert!(
                producer_delay >= Duration::from_secs(2),
                "Time.sleep_ms delay was not preserved: {producer_delay:?}"
            );
        }

        /// PLAN-696 T-06: closing the SSE client stops a sleeping generator
        /// before its post-sleep side effect runs.
        #[test]
        fn e2e_plan696_sse_disconnect_cancels_generator() {
            let port = start_server(
                r#"
var produced int = 0
#[api(method = "GET", path = "/api/cancel")]
fn cancel_stream() ~Iter<int> {
    yield 1
    Time.sleep_ms(2500)
    produced = 1
    yield 2
}
#[api(method = "GET", path = "/api/produced")]
fn produced_count() int { produced }
"#,
                18765,
            );

            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            stream.write_all(
                b"GET /api/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: keep-alive\r\n\r\n",
            ).unwrap();
            let mut received = Vec::new();
            let mut chunk = [0u8; 512];
            while !received
                .windows(b"data: 1\n\n".len())
                .any(|w| w == b"data: 1\n\n")
            {
                let n = stream.read(&mut chunk).expect("read first SSE frame");
                assert!(n > 0, "SSE closed before first frame: {:?}", received);
                received.extend_from_slice(&chunk[..n]);
            }
            drop(stream);

            std::thread::sleep(Duration::from_millis(3200));
            let state = http_get(port, "/api/produced");
            assert_eq!(
                body_of(&state),
                "0",
                "disconnected SSE generator continued: {state:?}"
            );
        }

        /// PLAN-696 T-07: the real 015-notes api.at/db.at pair preserves CRUD
        /// response shape and session state through the owner-thread server.
        #[test]
        fn e2e_plan696_real_015_notes_crud_parity() {
            let port = start_example_api_server("015-notes", 18766);
            let initial = http_get(port, "/api/notes");
            assert!(
                initial.starts_with("HTTP/1.1 200"),
                "initial list: {initial}"
            );
            let notes: serde_json::Value = serde_json::from_str(body_of(&initial)).unwrap();
            assert!(notes
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["title"] == "Welcome"));

            let created = http_post_json_with_headers(
                port,
                "/api/notes",
                r#"{"title":"Plan 696 parity","body":"body split","folder":"work"}"#,
                &[],
            );
            assert!(
                created.starts_with("HTTP/1.1 200"),
                "create response: {created}"
            );
            let created_note: serde_json::Value =
                serde_json::from_str(body_of(&created)).expect("created note JSON");
            assert_eq!(created_note["title"], "Plan 696 parity");
            assert_eq!(created_note["id"], 6);

            let persisted = http_get(port, "/api/notes");
            let notes: serde_json::Value = serde_json::from_str(body_of(&persisted)).unwrap();
            assert!(notes.as_array().unwrap().iter().any(|n| n["id"] == 6));
        }

        /// PLAN-696 T-07: the real 017-chat API keeps the message response and
        /// subsequent list response consistent across the VM HTTP requests.
        #[test]
        fn e2e_plan696_real_017_chat_crud_parity() {
            let port = start_example_api_server("017-chat", 18768);
            let contacts = http_get(port, "/api/contacts");
            assert!(contacts.starts_with("HTTP/1.1 200"), "contacts: {contacts}");
            assert!(
                body_of(&contacts).contains("Alice"),
                "contact seed: {contacts}"
            );

            let sent = http_post_json_with_headers(
                port,
                "/api/messages",
                r#"{"sender":"You","text":"Plan 696 parity"}"#,
                &[],
            );
            assert!(sent.starts_with("HTTP/1.1 200"), "send response: {sent}");
            let sent_message: serde_json::Value =
                serde_json::from_str(body_of(&sent)).expect("message JSON");
            assert_eq!(sent_message["text"], "Plan 696 parity");

            let listed = http_get(port, "/api/messages");
            assert!(listed.starts_with("HTTP/1.1 200"), "message list: {listed}");
            let messages: serde_json::Value = serde_json::from_str(body_of(&listed)).unwrap();
            assert!(messages
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["text"] == "Plan 696 parity"));
        }

        /// PLAN-696 T-07: real 023-realworld auth and article responses retain
        /// the current empty-record failure conventions and use the bearer
        /// token metadata binding for authenticated routes.
        #[test]
        fn e2e_plan696_real_023_auth_parity() {
            let port = start_example_api_server("023-realworld", 18767);
            let login = http_post_json_with_headers(
                port,
                "/api/users/login",
                r#"{"email":"sarah@vercel.com","password":"sarah-secret"}"#,
                &[],
            );
            assert!(login.starts_with("HTTP/1.1 200"), "login response: {login}");
            let user: serde_json::Value = serde_json::from_str(body_of(&login)).unwrap();
            assert_eq!(user["username"], "Sarah Chen");
            assert!(user["token"]
                .as_str()
                .unwrap_or_default()
                .starts_with("tok-"));
            assert!(user.get("password").is_none(), "password leaked: {user}");

            let token = user["token"].as_str().unwrap();
            let authorization = format!("Bearer {token}");
            let current = http_get_with_headers(
                port,
                "/api/user",
                &[("Authorization", authorization.as_str())],
            );
            assert!(
                current.starts_with("HTTP/1.1 200"),
                "current user: {current}"
            );
            let current: serde_json::Value = serde_json::from_str(body_of(&current)).unwrap();
            assert_eq!(current["username"], "Sarah Chen");

            let article = http_post_json_with_headers(
                port,
                "/api/articles",
                r#"{"slug":"plan-696-parity","title":"Plan 696","description":"runtime check","body":"auth owner","tagList":"verification"}"#,
                &[("Authorization", authorization.as_str())],
            );
            assert!(
                article.starts_with("HTTP/1.1 200"),
                "authenticated article: {article}"
            );
            let article: serde_json::Value = serde_json::from_str(body_of(&article)).unwrap();
            assert_eq!(article["slug"], "plan-696-parity");
            assert_eq!(article["author"], "Sarah Chen");

            let anonymous_article = http_post_json_with_headers(
                port,
                "/api/articles",
                r#"{"slug":"anonymous","title":"Anonymous","description":"","body":"","tagList":""}"#,
                &[],
            );
            assert!(
                anonymous_article.starts_with("HTTP/1.1 200"),
                "anonymous article convention changed: {anonymous_article}"
            );
            let anonymous_article: serde_json::Value =
                serde_json::from_str(body_of(&anonymous_article)).unwrap();
            assert_eq!(anonymous_article["slug"], "");

            let rejected = http_post_json_with_headers(
                port,
                "/api/users/login",
                r#"{"email":"sarah@vercel.com","password":"wrong"}"#,
                &[],
            );
            assert!(
                rejected.starts_with("HTTP/1.1 200"),
                "invalid-login convention changed: {rejected}"
            );
            let rejected: serde_json::Value = serde_json::from_str(body_of(&rejected)).unwrap();
            assert_eq!(rejected["id"], 0);
        }

        /// Plan 346 3c: `http.response_redirect(url, 302)` end-to-end. The
        /// handler returns a redirect Response object; the client must see
        /// 302 + Location, and following it reaches the target endpoint.
        ///
        /// Named `e2e_a_...` so it sorts FIRST in the serial suite: a prior
        /// test's detached server thread can auto-start late and read the
        /// process-global AUTO_HTTP_PORT while THIS test owns it, binding our
        /// port with its stale route table (404). Running first avoids the
        /// pre-existing harness race (see Plan 317 §11 detached-server notes).
        #[test]
        fn e2e_a_redirect_302_with_location() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/old")]
fn old_handler() int {
    return http.response_redirect("/new", 302)
}

#[api(method = "GET", path = "/new")]
fn new_handler() str {
    return "arrived"
}
"#,
                18740,
            );
            let resp = http_get(port, "/old");
            assert!(
                resp.starts_with("HTTP/1.1 302"),
                "expected 302 status line, got full response: {:?}",
                resp
            );
            assert!(
                resp.to_lowercase().contains("location: /new"),
                "expected Location: /new header, got: {:?}",
                resp
            );
            // Following the redirect reaches the target.
            let follow = http_get(port, "/new");
            assert!(
                body_of(&follow).contains("arrived"),
                "follow target should serve body, got: {:?}",
                follow
            );
        }

        /// Plan 442 C2: the musk backend's extern response-constructor shims
        /// (`ok_response`/`err_response`/`text_response` …) must produce real
        /// HttpResponseData objects served by the response-object path —
        /// instead of the empty `extern_sigs` no-op that answered `200 null`.
        /// Bare names resolve via BIGVM_NATIVES → CALL_NAT, so no `use
        /// extern_sigs` is needed here.
        #[test]
        fn e2e_musk_response_constructors() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/ok")]
fn ok_handler() int {
    return ok_response("hello")
}

#[api(method = "GET", path = "/err")]
fn err_handler() int {
    return err_response("boom", 500)
}

#[api(method = "GET", path = "/text")]
fn text_handler() int {
    return text_response("not found", 404)
}

#[api(method = "GET", path = "/to")]
fn to_handler() int {
    return to_response(null, "failed", 500)
}
"#,
                18739,
            );

            let ok = http_get(port, "/ok");
            assert!(
                ok.starts_with("HTTP/1.1 200"),
                "ok_response status: {}",
                ok.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&ok),
                "\"hello\"",
                "ok_response JSON body: full = {:?}",
                ok
            );

            let err = http_get(port, "/err");
            assert!(
                err.starts_with("HTTP/1.1 500"),
                "err_response status: {}",
                err.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&err),
                r#"{"error":"boom"}"#,
                "err_response body: full = {:?}",
                err
            );

            let text = http_get(port, "/text");
            assert!(
                text.starts_with("HTTP/1.1 404"),
                "text_response status: {}",
                text.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&text),
                "not found",
                "text_response body: full = {:?}",
                text
            );

            // to_response with a null value degrades to err_response.
            let to = http_get(port, "/to");
            assert!(
                to.starts_with("HTTP/1.1 500"),
                "to_response(null,…) status: {}",
                to.lines().next().unwrap_or("")
            );
            assert_eq!(
                body_of(&to),
                r#"{"error":"failed"}"#,
                "to_response(null,…) body: full = {:?}",
                to
            );
        }

        /// Audit B1 (023-realworld real token auth): loads the REAL 023
        /// back-end sources (api.at types + db.at auth logic) and drives the
        /// full auth flow over HTTP. The VM server binds a POST body as ONE
        /// string arg (per-field binding is the api_gen rust path), so thin
        /// #[api] shims (h_login etc.) parse fields via the real json_str
        /// helper; the db logic (password check, token minting/validation,
        /// author-from-token) is the verbatim 023 source.
        #[test]
        fn e2e_b1_realworld_token_auth() {
            let back = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/ui/023-realworld/src/back");
            let api_src = match std::fs::read_to_string(back.join("api.at")) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("skip: 023 api.at unreadable: {}", e);
                    return;
                }
            };
            let db_src =
                std::fs::read_to_string(back.join("db.at")).expect("023 db.at must exist in-repo");

            // Types block from api.at (up to `use db`), db.at minus its own
            // `use api:` import, plus bearer_token/json_str extracted verbatim.
            let types_block: String = api_src.split("use db").next().unwrap_or("").to_string();
            let db_body = db_src
                .lines()
                .filter(|l| !l.starts_with("use api:"))
                .collect::<Vec<_>>()
                .join("\n");
            let bearer_fn = api_src
                .split("pub fn bearer_token")
                .nth(1)
                .and_then(|rest| rest.split("// --- Stage 1: auth ---").next())
                .map(|body| format!("pub fn bearer_token{}", body))
                .expect("bearer_token fn present");
            let json_str_fn = api_src
                .split("pub fn json_str")
                .nth(1)
                .and_then(|rest| rest.split("/// Extract the bearer").next())
                .map(|body| format!("pub fn json_str{}", body))
                .expect("json_str fn present");

            let code = format!(
                r#"{}{}
{}
{}
#[api(method = "POST", path = "/api/users/login")]
fn h_login(email str, password str) User {{
    return login(email, password)
}}

#[api(method = "POST", path = "/api/users")]
fn h_register(username str, email str, password str) User {{
    return register(username, email, password)
}}

#[api(method = "GET", path = "/api/user")]
fn h_current_user(meta str) User {{
    return current_user(bearer_token(meta))
}}


#[api(method = "POST", path = "/api/articles")]
fn h_create_article(slug str, title str, description str, body str, tagList str, meta str) Article {{
    let u User = current_user(bearer_token(meta))
    if u.id == 0 {{
        let rejected Article = Article {{ slug: "", title: "", description: "", body: "", tagList: "", author: "", favoritesCount: 0, createdAt: "" }}
        return rejected
    }}
    return create_article(slug, title, description, body, tagList, u.username)
}}
"#,
                types_block, bearer_fn, json_str_fn, db_body
            );
            let port = start_server(&code, 18745);

            // 1. Wrong password → empty user (the old stub matched email alone).
            let bad = body_of(&http_post_json_with_headers(
                port,
                "/api/users/login",
                r#"{"email":"sarah@vercel.com","password":"WRONG"}"#,
                &[],
            ))
            .to_string();
            assert!(
                bad.contains("\"id\": 0"),
                "wrong password must fail: {}",
                bad
            );

            // 2. Correct login → real user + fresh unique token.
            let ok = body_of(&http_post_json_with_headers(
                port,
                "/api/users/login",
                r#"{"email":"sarah@vercel.com","password":"sarah-secret"}"#,
                &[],
            ))
            .to_string();
            assert!(ok.contains("\"id\": 1"), "login id: {}", ok);
            assert!(ok.contains("\"token\": \"tok-"), "minted token: {}", ok);
            let tok = {
                let s = ok.find("\"token\": \"tok-").expect("token idx") + 10;
                let e = ok[s..].find('"').expect("token end") + s;
                ok[s..e].to_string()
            };

            // 3. GET /api/user with Bearer → the logged-in user (meta path).
            let me = body_of(&http_get_with_headers(
                port,
                "/api/user",
                &[("Authorization", &format!("Bearer {}", tok))],
            ))
            .to_string();
            assert!(me.contains("\"username\": \"Sarah Chen\""), "me: {}", me);

            // 4. Without a token → logged out.
            let anon = body_of(&http_get(port, "/api/user")).to_string();
            assert!(anon.contains("\"id\": 0"), "anon: {}", anon);

            // 5. Author comes from the token, not the client.
            let created = body_of(&http_post_json_with_headers(
                port,
                "/api/articles",
                r#"{"slug":"authed-post","title":"Authed","description":"d","body":"b","tagList":"x"}"#,
                &[("Authorization", &format!("Bearer {}", tok))],
            ))
            .to_string();
            assert!(
                created.contains("\"author\": \"Sarah Chen\""),
                "author from token: {}",
                created
            );
            let rejected = body_of(&http_post_json_with_headers(
                port,
                "/api/articles",
                r#"{"slug":"anon-post","title":"A","description":"d","body":"b","tagList":"x"}"#,
                &[],
            ))
            .to_string();
            assert!(
                rejected.contains("\"slug\": \"\""),
                "anonymous create rejected: {}",
                rejected
            );
        }

        /// Plan 346 5a (B6): server-side multipart/form-data — POST a text
        /// field + a 20KB binary file (incl. 0xFF bytes and near-boundary
        /// CRLF--X sequences, and a body > the initial 8KB read to exercise
        /// the byte-level continuation). The handler receives
        /// {"fields":{...},"files":[{field,filename,path,size}]} as its body
        /// param; the test also verifies the persisted file's bytes.
        #[test]
        fn e2e_b6_multipart_upload_field_and_file() {
            let upload_dir = std::env::temp_dir().join("auto_multipart_e2e");
            let _ = std::fs::remove_dir_all(&upload_dir);
            std::env::set_var("AUTO_UPLOAD_DIR", upload_dir.to_str().unwrap());
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/upload")]
fn upload(form str) str {
    return form
}
"#,
                18744,
            );

            let boundary = "AutoBoundary7381";
            let mut file_data: Vec<u8> = Vec::new();
            for i in 0..20_000usize {
                // Pseudo-varied binary: high bytes, plain ascii, and a
                // near-boundary sequence that must NOT split a part.
                file_data.push([0xFFu8, b'A', b'\r', b'\n', b'-', b'-', b'X'][i % 7]);
            }
            let mut body: Vec<u8> = Vec::new();
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(
                b"Content-Disposition: form-data; name=\"title\"\r\n\r\nhello upload\r\n",
            );
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(
                b"Content-Disposition: form-data; name=\"avatar\"; filename=\"pic a.bin\"\r\n\
                  Content-Type: application/octet-stream\r\n\r\n",
            );
            body.extend_from_slice(&file_data);
            body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

            let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let req = format!(
                "POST /api/upload HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: multipart/form-data; boundary={}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                boundary,
                body.len()
            );
            use std::io::Write as _;
            stream.write_all(req.as_bytes()).unwrap();
            stream.write_all(&body).unwrap();
            let mut resp = String::new();
            use std::io::Read as _;
            stream.read_to_string(&mut resp).ok();

            let resp_body = body_of(&resp).to_string();
            assert!(
                resp_body.contains("hello upload"),
                "text field must reach the handler, got: {:?}",
                &resp_body[..resp_body.len().min(400)]
            );
            assert!(
                resp_body.contains("pic a.bin"),
                "file part metadata expected, got: {:?}",
                &resp_body[..resp_body.len().min(400)]
            );
            // The response wraps the handler JSON as a JSON string (inner
            // quotes escaped), so pin size via the unescaped `:<len>` tail.
            assert!(
                resp_body.contains(&format!(":{}", file_data.len())),
                "binary size must survive byte-level reads, got: {:?}",
                &resp_body[..resp_body.len().min(400)]
            );
            // The persisted file is discoverable in the upload dir — verify
            // its bytes match the uploaded binary exactly (escape-free).
            let mut stored_files: Vec<_> = std::fs::read_dir(&upload_dir)
                .expect("upload dir exists")
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file())
                .collect();
            stored_files.sort();
            assert_eq!(
                stored_files.len(),
                1,
                "exactly one stored file: {:?}",
                stored_files
            );
            let stored = std::fs::read(&stored_files[0])
                .unwrap_or_else(|e| panic!("stored file readable: {}", e));
            assert_eq!(
                stored, file_data,
                "persisted bytes must match uploaded bytes exactly"
            );

            let _ = std::fs::remove_dir_all(&upload_dir);
        }

        /// Plan 346 #12 (B6): every response carries X-Request-Id — minted
        /// (`req-<ms>-<n>`) when the client sends none.
        #[test]
        fn e2e_b6_request_id_generated_and_echoed() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/rid")]
fn rid() int {
    return 7
}
"#,
                18741,
            );
            let resp = http_get(port, "/api/rid");
            assert!(
                resp.to_lowercase().contains("x-request-id: req-"),
                "minted request id header expected, got: {:?}",
                resp
            );
            // 404 responses carry it too.
            let not_found = http_get(port, "/api/nope");
            assert!(
                not_found.to_lowercase().contains("x-request-id: req-"),
                "404 should carry request id, got: {:?}",
                not_found
            );
        }

        /// Plan 346 #12 (B6): an incoming X-Request-Id passes through verbatim
        /// (trace propagation).
        #[test]
        fn e2e_b6_request_id_incoming_passthrough() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/rid2")]
fn rid2() int {
    return 8
}
"#,
                18742,
            );
            let resp = http_get_with_headers(port, "/api/rid2", &[("X-Request-Id", "my-trace-42")]);
            assert!(
                resp.to_lowercase().contains("x-request-id: my-trace-42"),
                "incoming request id should be echoed verbatim, got: {:?}",
                resp
            );
            assert!(
                !resp.to_lowercase().contains("x-request-id: req-"),
                "minted id should not override the incoming one, got: {:?}",
                resp
            );
        }

        /// Plan 346 5e (B6): http.rate_limit(2, 60000) — the third request
        /// from the same IP gets 429 + Retry-After. start_server resets the
        /// limiter via clear_http_routes, so this test cannot poison others.
        #[test]
        fn e2e_zz_rate_limit_429_after_quota() {
            let port = start_server(
                r#"
http.rate_limit(2, 60000)

#[api(method = "GET", path = "/api/rl")]
fn rl() int {
    return 1
}
"#,
                18743,
            );
            let first = http_get(port, "/api/rl");
            let second = http_get(port, "/api/rl");
            assert!(
                first.starts_with("HTTP/1.1 200"),
                "first should pass, got: {:?}",
                first
            );
            assert!(
                second.starts_with("HTTP/1.1 200"),
                "second should pass, got: {:?}",
                second
            );
            let third = http_get(port, "/api/rl");
            assert!(
                third.starts_with("HTTP/1.1 429"),
                "third request should be rate limited, got: {:?}",
                third
            );
            assert!(
                third.to_lowercase().contains("retry-after:"),
                "429 should carry Retry-After, got: {:?}",
                third
            );
            assert!(
                body_of(&third).contains("rate limit exceeded"),
                "429 body should explain, got: {:?}",
                third
            );
            assert!(
                third.to_lowercase().contains("x-request-id: "),
                "429 should carry the request id, got: {:?}",
                third
            );
        }

        /// Plan 317 Phase 4 validation: 015-notes-style CRUD on the async HTTP
        /// server. Exercises the same patterns as examples/ui/015-notes/src/back:
        ///   - list: returns []Note (array of structs → JSON array of objects)
        ///   - get:  :id path param + ?Note (Option → inner value or null)
        ///   - create: POST body (title/body) → Note
        /// Uses a module-level var for in-memory storage (like db.at's `var notes`).
        #[test]
        fn e2e_notes_crud() {
            let port = start_server(
                r#"
type Note { id int; title str; body str; time str }

var notes = [
    Note { id: 0, title: "Welcome", body: "first", time: "now" },
    Note { id: 1, title: "Shopping", body: "milk", time: "ago" },
]
var nextid int = 2

#[api(method = "GET", path = "/api/notes")]
fn list_notes() []Note {
    return notes
}

#[api(method = "GET", path = "/api/notes/:id")]
fn get_note(id int) ?Note {
    for note in notes {
        if note.id == id {
            return Some(note)
        }
    }
    return None
}

#[api(method = "POST", path = "/api/notes")]
fn create_note(title str, body str) Note {
    let note = Note { id: nextid, title: title, body: body, time: "now" }
    nextid = nextid + 1
    return note
}
"#,
                18736,
            );

            // GET /api/notes → JSON array of Note objects
            let resp_list = http_get(port, "/api/notes");
            let body_list = body_of(&resp_list);
            assert!(
                body_list.contains("\"title\": \"Welcome\""),
                "list: body={:?}",
                body_list
            );
            assert!(
                body_list.contains("\"title\": \"Shopping\""),
                "list: body={:?}",
                body_list
            );
            // Should be a JSON array: starts with [
            assert!(
                body_list.trim_start().starts_with('['),
                "list not array: body={:?}",
                body_list
            );

            // GET /api/notes/1 → single Note (Option.Some unwrapped)
            let resp_get = http_get(port, "/api/notes/1");
            let body_get = body_of(&resp_get);
            assert!(
                body_get.contains("\"id\": 1"),
                "get id=1: body={:?}",
                body_get
            );
            assert!(
                body_get.contains("\"title\": \"Shopping\""),
                "get title: body={:?}",
                body_get
            );

            // PLAN-669 AC-01: POST body fields bind by param name — the
            // handler's `title`/`body` slots receive the JSON fields, not the
            // raw body string (this declared-but-untested surface was the
            // defect's hiding place; F7 in the plan).
            let resp_post = http_post_json_with_headers(
                port,
                "/api/notes",
                r#"{"title": "probe-item", "body": "probe-body"}"#,
                &[],
            );
            let body_post = body_of(&resp_post);
            assert!(
                body_post.contains("\"title\": \"probe-item\""),
                "post title by name: body={:?}",
                body_post
            );
            assert!(
                body_post.contains("\"body\": \"probe-body\""),
                "post body by name: body={:?}",
                body_post
            );
            assert!(
                !body_post.contains("{\\\"title\\\""),
                "raw body literal must not leak into fields: body={:?}",
                body_post
            );
        }

        /// PLAN-669 AC-01: POST JSON body fields bind by param name — the
        /// 013-todo create_todo probe shape from the defect report (raw body
        /// string used to land in `text` as a JSON literal).
        #[test]
        fn http_e2e_api_post_body_by_name() {
            let port = start_server(
                r#"
type Todo { id int; text str; done bool }
var nextid int = 4

#[api(method = "POST", path = "/api/todos")]
fn create_todo(text str) Todo {
    let todo = Todo { id: nextid, text: text, done: false }
    nextid = nextid + 1
    return todo
}
"#,
                18750,
            );

            let resp =
                http_post_json_with_headers(port, "/api/todos", r#"{"text":"probe-item"}"#, &[]);
            let body = body_of(&resp);
            assert!(
                body.contains("\"text\": \"probe-item\""),
                "text bound from body field: body={:?}",
                body
            );
            assert!(
                !body.contains("{\\\"text\\\""),
                "raw body literal leaked into field: body={:?}",
                body
            );
        }

        /// PLAN-669 AC-02: GET query params bind by name (the 015-notes
        /// search_notes shape; both `q` and `query` present — the declared
        /// name wins).
        #[test]
        fn http_e2e_api_query_by_name() {
            let port = start_server(
                r#"
type Hit { text str }

#[api(method = "GET", path = "/api/search")]
fn search(query str) Hit {
    return Hit { text: query }
}
"#,
                18751,
            );

            let resp = http_get(port, "/api/search?q=wrong&query=Build");
            let body = body_of(&resp);
            assert!(
                body.contains("\"text\": \"Build\""),
                "query bound by name: body={:?}",
                body
            );
        }

        /// PLAN-669 AC-03: declared `int` params convert exactly from the
        /// query string; non-numeric input is a named 400.
        #[test]
        fn http_e2e_api_typed_query_int() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/page")]
fn page(p int) int {
    return p
}
"#,
                18752,
            );

            let resp = http_get(port, "/api/page?p=7");
            let body = body_of(&resp);
            assert!(body.trim() == "7", "int query converted: body={:?}", body);

            let resp_bad = http_get(port, "/api/page?p=seven");
            assert!(
                resp_bad.starts_with("HTTP/1.1 400"),
                "non-numeric int → 400: resp={:?}",
                &resp_bad[..resp_bad.len().min(80)]
            );
            assert!(
                body_of(&resp_bad).contains("param `p`"),
                "400 names the param: body={:?}",
                body_of(&resp_bad)
            );
        }

        /// PLAN-669 AC-04: a missing body field is a 400 naming the param
        /// (back_proxy parity — silently wrong args are the defect itself).
        #[test]
        fn http_e2e_api_missing_param_400() {
            let port = start_server(
                r#"
type Note { title str; body str }

#[api(method = "POST", path = "/api/notes")]
fn create_note(title str, body str) Note {
    return Note { title: title, body: body }
}
"#,
                18753,
            );

            let resp =
                http_post_json_with_headers(port, "/api/notes", r#"{"title": "only-title"}"#, &[]);
            assert!(
                resp.starts_with("HTTP/1.1 400"),
                "missing body field → 400: resp={:?}",
                &resp[..resp.len().min(80)]
            );
            let body = body_of(&resp);
            assert!(
                body.contains("missing param `body`"),
                "400 names the missing param: body={:?}",
                body
            );
        }

        /// PLAN-669 AC-07: single str param + unparseable body → the raw
        /// body string passes through (back_proxy tolerance; the positional
        /// convention's one correct case, preserved).
        #[test]
        fn http_e2e_api_raw_body_single_param() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/save")]
fn save(data str) str {
    return data
}
"#,
                18754,
            );

            // Not JSON — parse fails, lone str param receives it verbatim.
            let resp = http_post_json_with_headers(port, "/api/save", "plain text probe", &[]);
            let body = body_of(&resp);
            assert!(
                body.contains("plain text probe"),
                "raw body passthrough: body={:?}",
                body
            );
        }

        /// Plan 317 final validation: 015-notes backend pattern with List<Note>
        /// generic + module-level var + #[api] handler returning the list.
        /// This mirrors db.at's `var notes List<Note>` + `fn all_notes() []Note`.
        #[test]
        fn e2e_notes_list_generic() {
            let port = start_server(
                r#"
type Note { id int; title str; body str; time str }

var notes = [
    Note { id: 0, title: "Welcome", body: "first", time: "now" },
    Note { id: 1, title: "Shopping", body: "milk", time: "ago" },
]

#[api(method = "GET", path = "/api/notes")]
fn list_notes() []Note {
    return notes
}
"#,
                18737,
            );

            let resp = http_get(port, "/api/notes");
            let body = body_of(&resp);
            // List<Note> serialized as JSON array of Note objects.
            assert!(
                body.contains("\"title\": \"Welcome\""),
                "list generic frame 1: body={:?}",
                body
            );
            assert!(
                body.contains("\"title\": \"Shopping\""),
                "list generic frame 2: body={:?}",
                body
            );
            assert!(
                body.trim_start().starts_with('['),
                "list generic should be JSON array: body={:?}",
                body
            );
        }

        // ============ PLAN-699: Axum/Hyper transport protocol probes ========
        // Raw-TCP probes against the real VM server (AC-01 framing/keep-alive,
        // AC-03 header bounds, AC-05 injected-signal shutdown). Ports
        // 18770-18775 (unique per e2e_ports_unique).

        /// Connect to the test server with the standard retry loop.
        fn connect_retry(port: u16) -> std::net::TcpStream {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = std::net::TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("connect to test HTTP server");
            stream.set_read_timeout(Some(Duration::from_secs(25))).ok();
            stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
            stream
        }

        /// AC-01: a chunked request body is decoded by the transport — the
        /// legacy hand-written parser rejected Transfer-Encoding outright.
        #[test]
        fn e2e_plan699_chunked_request_body_accepted() {
            let port = start_server(
                r#"
#[api(method = "POST", path = "/api/echo")]
fn echo(text str) str { text }
"#,
                18770,
            );
            let mut s = connect_retry(port);
            let req = "POST /api/echo HTTP/1.1\r\nHost: 127.0.0.1\r\n\
                       Content-Type: text/plain\r\nTransfer-Encoding: chunked\r\n\
                       Connection: close\r\n\r\n5\r\nhello\r\nE\r\n chunked world\r\n0\r\n\r\n";
            s.write_all(req.as_bytes()).unwrap();
            let mut resp = String::new();
            s.read_to_string(&mut resp).ok();
            assert!(
                resp.starts_with("HTTP/1.1 200"),
                "chunked body accepted, got: {:?}",
                resp
            );
            assert!(
                resp.contains("hello chunked world"),
                "decoded body echoed, got: {:?}",
                resp
            );
        }

        /// AC-01: two sequential requests on one connection (keep-alive);
        /// the legacy server closed after every response.
        #[test]
        fn e2e_plan699_keepalive_two_requests_one_connection() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/ping")]
fn ping() str { "pong" }
"#,
                18771,
            );
            let mut s = connect_retry(port);
            let req = "GET /api/ping HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
            s.write_all(req.as_bytes()).unwrap();
            s.write_all(req.as_bytes()).unwrap();
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                let n = match s.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                buf.extend_from_slice(&chunk[..n]);
                let text = String::from_utf8_lossy(&buf);
                let heads = text.matches("HTTP/1.1").count();
                if heads >= 2 && text.matches("pong").count() >= 2 {
                    break;
                }
                if buf.len() > 64 * 1024 {
                    break;
                }
            }
            let resp = String::from_utf8_lossy(&buf).into_owned();
            assert_eq!(
                resp.matches("HTTP/1.1 200").count(),
                2,
                "two keep-alive responses on one connection, got: {:?}",
                resp
            );
        }

        /// AC-03: oversized request head (beyond the 64 KiB connection
        /// buffer) is rejected with hyper's native 431.
        #[test]
        fn e2e_plan699_oversized_headers_431() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/ping")]
fn ping() str { "pong" }
"#,
                18772,
            );
            let mut s = connect_retry(port);
            let big = format!("X-Big: {}\r\n", "a".repeat(70 * 1024));
            let req = format!("GET /api/ping HTTP/1.1\r\nHost: 127.0.0.1\r\n{}\r\n", big);
            let _ = s.write_all(req.as_bytes());
            let mut resp = String::new();
            let _ = s.read_to_string(&mut resp);
            assert!(
                resp.starts_with("HTTP/1.1 431"),
                "oversized head must be rejected with 431, got: {:?}",
                resp.lines().next()
            );
        }

        /// AC-03: a request head that never completes is closed by the 10s
        /// header read timeout (deterministic close, no unbounded hold).
        #[test]
        fn e2e_plan699_slow_headers_timeout_close() {
            let port = start_server(
                r#"
#[api(method = "GET", path = "/api/ping")]
fn ping() str { "pong" }
"#,
                18773,
            );
            let mut s = connect_retry(port);
            let _ = s.write_all(b"GET /api/ping HTTP/1.1\r\nHost: 127.0.0.1\r\n");
            let _ = s.flush();
            let started = std::time::Instant::now();
            let mut sink = Vec::new();
            let _ = s.read_to_end(&mut sink);
            let elapsed = started.elapsed();
            assert!(
                elapsed >= Duration::from_secs(8) && elapsed <= Duration::from_secs(20),
                "head timeout should close at ~10s, elapsed={:?}",
                elapsed
            );
        }

        /// AC-05: the injected shutdown flag stops the accept loop, the owner
        /// loop drains and exits, and the port is rebindable. A bare VM (no
        /// routes) is enough — every request 404s through the real dispatch.
        #[test]
        fn e2e_plan699_injected_shutdown_releases_port() {
            let port: u16 = 18775;
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let cfg = crate::vm::ffi::http_transport::TransportConfig::from_env();
            let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
            // Rc<AutoVM> is !Send — build the VM inside the owner thread
            // (same pattern as start_server).
            let _server = std::thread::Builder::new()
                .stack_size(8 * 1024 * 1024)
                .spawn(move || {
                    let vm = std::rc::Rc::new(crate::vm::engine::AutoVM::new(
                        crate::vm::virt_memory::VirtualFlash::new_with_code(vec![
                            crate::vm::opcode::OpCode::RET as u8,
                        ]),
                        1024,
                    ));
                    crate::block_on_autovm_local(async move {
                        crate::vm::ffi::http_server::serve_with(
                            vm,
                            &format!("0.0.0.0:{}", port),
                            cfg,
                            shutdown_rx,
                        )
                        .await;
                    });
                    let _ = done_tx.send(());
                })
                .expect("spawn shutdown-test server thread");

            // Wait for accept, then exercise the real dispatch (404 path).
            let mut s = connect_retry(port);
            s.write_all(
                "GET /api/nope HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n".as_bytes(),
            )
            .unwrap();
            let mut resp = String::new();
            let _ = s.read_to_string(&mut resp);
            assert!(
                resp.starts_with("HTTP/1.1 404"),
                "bare VM serves 404 through the bridge, got: {:?}",
                resp
            );

            // Flip the shutdown flag: the server must exit promptly.
            let _ = shutdown_tx.send(true);
            done_rx
                .recv_timeout(Duration::from_secs(15))
                .expect("server exits after injected shutdown");
            // Port is released: a fresh bind on the same port must succeed.
            let rebind = std::net::TcpListener::bind(("127.0.0.1", port));
            assert!(rebind.is_ok(), "port must be rebindable after shutdown");
        }
    }
}

/// Run the HTTP server in blocking mode using std::net (MVP).
///
/// This is the current implementation — synchronous, serial request handling.
/// Each request is dispatched to a VM handler function via call_fn_by_name.
///
/// Future: replace with Axum for concurrency, SSE, TLS support.
/// Plan 510 G1-1: handler 实参字符串入池咽喉。原三处(路径参数/body/
/// request_info)裸 `strings.write().push` + `push_nv(encode_string)`:
/// 不进 dedup、不 ensure_len(rc 数组不覆盖该槽)——引用天生无计数,
/// 消费侧任何 release 即凭空多扣(over-release 注入源;P-053-5 把
/// native.rs/stdlib.rs 收口到 add_string 时漏掉本文件)。
/// 统一走 intern_runtime_str(add_string + rc 入栈 +1)。
pub(crate) fn push_str_arg(
    vm: &crate::vm::engine::AutoVM,
    task: &mut crate::vm::task::AutoTask,
    s: &str,
) {
    vm.intern_runtime_str(task, s.as_bytes().to_vec());
}

pub fn serve_blocking_stdnet(vm: &crate::vm::engine::AutoVM, addr: &str) {
    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[HTTP] Server bind failed on {}: {}", addr, e);
            return;
        }
    };
    eprintln!("[HTTP] Server listening on {}", addr);

    let routes = get_routes();

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[HTTP] Accept error: {}", e);
                continue;
            }
        };

        // Parse HTTP request
        let mut reader = std::io::BufReader::new(&mut stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line).is_err() {
            continue;
        }
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            let resp = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n{}\r\n",
                cors_headers()
            );
            let _ = stream.write_all(resp.as_bytes());
            continue;
        }
        let req_method = parts[0].to_uppercase();
        let req_path = parts[1].to_string();

        // Plan 349 步骤 7/8 (W5): CORS preflight short-circuit — respond before reading
        // the body since preflight requests have no body.
        if let Some(preflight) = handle_cors_preflight(&req_method) {
            let _ = stream.write_all(preflight.as_bytes());
            continue;
        }

        // Read headers
        let mut content_length = 0usize;
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header).is_err() {
                break;
            }
            let header = header.trim();
            if header.is_empty() {
                break;
            }
            if header.to_lowercase().starts_with("content-length:") {
                content_length = header[15..].trim().parse().unwrap_or(0);
            }
        }

        // Read body
        let body = if content_length > 0 {
            let mut buf = vec![0u8; content_length];
            let _ = (&mut reader).read_exact(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        } else {
            String::new()
        };

        // Route matching
        let route_match = match match_route(&routes, &req_method, &req_path) {
            Some(m) => m,
            None => {
                let resp = format!("HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n{}\r\nNot Found", cors_headers());
                let _ = stream.write_all(resp.as_bytes());
                continue;
            }
        };

        // Call VM handler
        let handler_task_id = vm.spawn_task(0, 8192);
        let result_json: Option<String> = if let Some(handler_task_arc) =
            vm.tasks.get(&handler_task_id)
        {
            let mut ht = handler_task_arc.blocking_lock();

            // PLAN-669: by-name binding when the fn's sigs are published;
            // legacy positional convention otherwise (see
            // bind_api_args_or_legacy — shared across all sync-serve sites).
            let bind_result: Result<usize, ApiArgBindError> = bind_api_args_or_legacy(
                vm,
                &mut ht,
                &route_match.fn_name,
                &route_match.path_params,
                &route_match.query_params,
                &body,
                &req_method,
                &req_path,
            );
            let n_args = match bind_result {
                Ok(n) => n,
                Err(e) => {
                    drop(ht);
                    vm.tasks.remove(&handler_task_id);
                    let (status, msg) = match e {
                        ApiArgBindError::BadRequest(m) => ("400 Bad Request", m),
                        ApiArgBindError::Internal(m) => ("500 Internal Server Error", m),
                    };
                    eprintln!("[HTTP] {} {} → {} ({})", req_method, req_path, status, msg);
                    let err_body = format!("{{\"error\":{}}}", json_escape_string(&msg));
                    let resp = format!(
                        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
                        status, err_body.len(), cors_headers(), err_body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    continue;
                }
            };

            // PLAN-705 T-05: legacy 同步驱动保留位——本 fn 是非默认调用图
            // 的串行 stdnet server（决策报告 §2 入口矩阵 legacy 行）；默认
            // Axum 调用图（serve_with → dispatch_api_request_segment）零
            // 同步忙等派发，由 plan705 gate 测试锁定。
            match vm.call_fn_by_name(&mut ht, &route_match.fn_name, n_args) {
                Ok(()) => {
                    let nv = ht.ram.pop_nv();

                    // Plan 321 SSE: Check if the return value is an iterator ID
                    // (generator/~Stream<T>/~Iter<T> handler → SSE streaming mode).
                    if auto_val::is_i32(nv) {
                        let iter_id = auto_val::decode_i32(nv) as u32;
                        if vm.iterators.contains_key(&(iter_id)) {
                            // SSE streaming mode: write SSE headers, then pull
                            // values from the iterator as SSE data frames.
                            drop(ht);
                            let sse_response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n{}\r\n",
                                cors_headers()
                            );
                            let _ = stream.write_all(sse_response.as_bytes());
                            let _ = stream.flush();

                            // Pull values from the iterator and write SSE frames
                            loop {
                                // Create a temp task for the next() call
                                let next_task_id = vm.spawn_task(0, 1024);
                                let next_result = if let Some(nt_arc) = vm.tasks.get(&next_task_id)
                                {
                                    let mut nt = nt_arc.blocking_lock();
                                    // Push iterator_id for auto.iterator.next
                                    nt.ram.push_i32(iter_id as i32);
                                    // Call the native iterator.next
                                    crate::vm::native::shim_iterator_next(&mut nt, vm).ok();
                                    // Result is on stack (i32) or nothing (done)
                                    if nt.ram.sp > 0 {
                                        Some(nt.ram.pop_nv())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                vm.tasks.remove(&next_task_id);

                                match next_result {
                                    Some(val) if auto_val::is_i32(val) => {
                                        let v = auto_val::decode_i32(val);
                                        if v == -1 {
                                            // Iterator exhausted
                                            break;
                                        }
                                        // Write SSE data frame
                                        let frame = format!("data: {}\n\n", v);
                                        let _ = stream.write_all(frame.as_bytes());
                                        let _ = stream.flush();
                                    }
                                    Some(val) if auto_val::is_string(val) => {
                                        let idx = auto_val::decode_string(val);
                                        let s = vm
                                            .strings
                                            .read()
                                            .unwrap()
                                            .get(idx as usize)
                                            .map(|b| String::from_utf8_lossy(b).to_string())
                                            .unwrap_or_default();
                                        let frame = format!("data: {}\n\n", s);
                                        let _ = stream.write_all(frame.as_bytes());
                                        let _ = stream.flush();
                                    }
                                    _ => break,
                                }
                            }
                            // Stream ended — close connection
                            continue; // Skip the normal JSON response below
                        }
                    }

                    // PLAN-729 T-04：legacy stdnet 不承载文件响应——声明
                    // 文件返回的路由在此入口明确 500 诊断（决策报告 §6：
                    // 未接入入口不冒充支持；描述符不序列化为 JSON）。
                    if fn_is_api_file_return(&route_match.fn_name) {
                        drop(ht);
                        let resp = format!(
                            "HTTP/1.1 500 Internal Server Error
Content-Type: application/json
Content-Length: {}
Connection: close
{}
{{\"error\":\"file responses require the default HTTP transport\"}}",
                            br#"{"error":"file responses require the default HTTP transport"}"#
                                .len(),
                            cors_headers()
                        );
                        let _ = stream.write_all(resp.as_bytes());
                        continue;
                    }
                    // Normal JSON response mode (Plan 326 Phase 3)
                    // nv_to_json handles string/i32/f64/bool/null, and recognizes
                    // heap object IDs (>= 4_000_000) to expand struct/array/Option
                    // return values into proper JSON instead of bare "null".
                    nv_to_json(vm, nv, 0)
                }
                Err(e) => {
                    eprintln!("[HTTP] Handler '{}' error: {:?}", route_match.fn_name, e);
                    None
                }
            }
        } else {
            None
        };

        vm.tasks.remove(&handler_task_id);

        let (status, body_json) = match result_json {
            Some(s) => ("200 OK", s),
            None => ("500 Internal Server Error", "{}".to_string()),
        };
        // PLAN-698 SD-02: POST 广播臂（Typing / New{Type}，has_sse 门控）。
        if req_method == "POST" && status == "200 OK" {
            publish_post_broadcast(&route_match.fn_name, &body, &body_json);
        }
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
            status, body_json.len(), cors_headers(), body_json
        );
        let _ = stream.write_all(response.as_bytes());
    }
}

// ============================================================================
// PLAN-699 T-02: owned request/reply bridge + VM-owner dispatch core
// ============================================================================

/// Owned, `Send` request handed from the HTTP network layer to the VM owner
/// thread. Protocol objects (hyper types, sockets) never cross the bridge;
/// the owner re-runs the same routing/binding/middleware/response semantics
/// as the legacy inline path (AC-02: no `Rc<AutoVM>`, no raw pointers).
pub(crate) struct ApiRequest {
    pub method: String,
    /// Raw request target (path + query string), as received.
    pub path: String,
    /// All request headers (names lowercased, values as received).
    pub headers: Vec<(String, String)>,
    /// Fully framed body bytes (bounded by the transport's body limit).
    pub body: Vec<u8>,
    /// PLAN-730 T-05：上传路由的原始 body（bridge 不消费，授权后由
    /// receive 增量读取；非上传路由恒 None——`body` 语义不变）。
    pub raw_upload_body: Option<axum::body::Body>,
    pub peer: Option<std::net::SocketAddr>,
}

impl ApiRequest {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Reply body payload: plain bytes, SSE frames streamed from a producer
/// task on the VM owner thread, or a file response descriptor (PLAN-729).
pub(crate) enum ApiBody {
    Text(Vec<u8>),
    Sse(tokio::sync::mpsc::Receiver<String>),
    /// PLAN-729 T-04：文件响应描述符（编组单次取出；打开/协议决策/发送
    /// 在 transport 侧的共享宿主服务，不在 VM owner）。status/headers 由
    /// 宿主 serve 决定，`Full` 携带的占位被 File 臂忽略。
    File(FileReplySeed),
}

/// [`ApiBody::File`] 载荷：owned 描述符 + 请求 id（日志/CORS 追加用）。
pub(crate) struct FileReplySeed {
    pub descriptor: a2r_std::http::FileResponse,
    pub request_id: String,
}

/// Structured reply produced by the VM owner for one request.
pub(crate) enum ApiReply {
    Full {
        status: u16,
        headers: Vec<(String, String)>,
        body: ApiBody,
    },
    /// `#[api]` route matched an `Upgrade: websocket` request — the network
    /// layer writes the 101 (accept value precomputed here) and serves the
    /// raw echo loop on the upgraded IO. Capability boundary unchanged:
    /// simplified echo, not general WebSocket (Design 33 §7.3).
    WebSocket { accept: String },
}

/// CORS response headers as typed pairs (the string block in `cors_headers`
/// stays for the sync stdnet server's string writer).
pub(crate) fn cors_header_pairs() -> Vec<(String, String)> {
    // PLAN-736 T-04: 服务策略激活时 legacy `*` 头块停用——附件由桥层按
    // 配置决策（精确 origin/Vary），避免双份/泄漏。
    if crate::http_service_config::service_policy_active() {
        return Vec::new();
    }
    let origin = cors_origin();
    vec![
        ("Access-Control-Allow-Origin".to_string(), origin),
        (
            "Access-Control-Allow-Methods".to_string(),
            "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string(),
        ),
        (
            "Access-Control-Allow-Headers".to_string(),
            "Content-Type, Authorization".to_string(),
        ),
        ("Access-Control-Max-Age".to_string(), "86400".to_string()),
    ]
}

/// Sec-WebSocket-Accept = base64(sha1(key + magic GUID)) — same handshake the
/// legacy inline path computed; pure protocol math, no VM involvement.
pub(crate) fn compute_ws_accept(key: &str) -> String {
    use base64::Engine;
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let hash = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(&hash)
}

/// Plan 317 Phase 4: Concurrent HTTP server using tokio async I/O.
///
/// Replaces the serial `serve_blocking_stdnet` for the Goroutine-style
/// concurrency model. The VM owner enters this on a Tokio `LocalSet`, and all
/// connection tasks remain on that same thread because `AutoVM` is `!Send`.
///
/// Each accepted connection becomes a `spawn_local` task:
///   - JSON handlers: call_fn_by_name (synchronous), write response, done.
///   - SSE handlers: a bounded local producer steps the VM in instruction
///     batches while the connection task writes frames.
/// PLAN-736 T-05：进程内服务关停发送端（`POST /__auto/shutdown` 触发同一
/// 关闭入口；headless 测试的确定性注入路径——真实 Ctrl+C 证据范围见 SD-01）。
static SERVICE_SHUTDOWN_TX: std::sync::Mutex<
    Option<tokio::sync::watch::Sender<bool>>,
> = std::sync::Mutex::new(None);

fn register_service_shutdown_tx(tx: tokio::sync::watch::Sender<bool>) {
    if let Ok(mut slot) = SERVICE_SHUTDOWN_TX.lock() {
        *slot = Some(tx);
    }
}

fn peer_is_loopback(peer: std::net::SocketAddr) -> bool {
    peer.ip().is_loopback()
}

/// PLAN-736 T-05/AC-02/05：服务控制面（`/__auto/*`，装配期与业务同名路由
/// 冲突=启动诊断）。live/ready 开放（身份最小体）；snapshot/shutdown 仅
/// loopback peer（不对代理/外部暴露操作面）。
pub(crate) fn handle_service_control_path(
    method: &str,
    path: &str,
    peer: std::net::SocketAddr,
) -> Option<axum::response::Response> {
    use crate::http_service_observability as obs;
    if !path.starts_with("/__auto/") {
        return None;
    }
    let json_resp = |status: u16, body: serde_json::Value| -> axum::response::Response {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::from_u16(status).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR))
            .header("content-type", "application/json")
            .body(axum::body::Body::from(body.to_string()))
            .unwrap()
    };
    match (method, path) {
        ("GET", "/__auto/health/live") => Some(json_resp(
            200,
            serde_json::json!({ "state": obs::service_state().as_str() }),
        )),
        ("GET", "/__auto/health/ready") => {
            if obs::is_ready() {
                Some(json_resp(200, obs::service_identity().unwrap_or_else(|| serde_json::json!({}))))
            } else {
                Some(json_resp(
                    503,
                    serde_json::json!({ "state": obs::service_state().as_str() }),
                ))
            }
        }
        ("GET", "/__auto/health/snapshot") => {
            if !peer_is_loopback(peer) {
                return Some(json_resp(403, serde_json::json!({ "error": "loopback only" })));
            }
            Some(json_resp(
                200,
                serde_json::json!({
                    "state": obs::service_state().as_str(),
                    "identity": obs::service_identity(),
                    "counters": obs::service_counters().snapshot(),
                }),
            ))
        }
        ("POST", "/__auto/shutdown") => {
            if !peer_is_loopback(peer) {
                return Some(json_resp(403, serde_json::json!({ "error": "loopback only" })));
            }
            let sent = SERVICE_SHUTDOWN_TX
                .lock()
                .ok()
                .and_then(|slot| slot.as_ref().map(|tx| tx.send(true).is_ok()))
                .unwrap_or(false);
            obs::emit_event(serde_json::json!({
                "event": "drain", "trigger": "shutdown_endpoint", "delivered": sent,
            }));
            Some(json_resp(202, serde_json::json!({ "draining": sent })))
        }
        _ => Some(json_resp(404, serde_json::json!({ "error": "not found" }))),
    }
}

/// PLAN-736 AC-01/02: 最近一次 serve 的致命错误（bind 失败/net 线程死亡）。
/// serve 入口（`auto serve`/run_file_with_service 消费者）据此以非零退出；
/// legacy `auto run` 面不消费——保持既有"打印后继续"语义不变。
static SERVE_FATAL: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
/// PLAN-736 AC-02: 最近一次 serve 成功 bind 的真实地址（port=0 解析值）。
static SERVE_BOUND_ADDR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

fn record_serve_fatal(msg: impl Into<String>) {
    if let Ok(mut slot) = SERVE_FATAL.lock() {
        *slot = Some(msg.into());
    }
}

/// serve 入口在 serve 返回后取致命错误（Some = 非零退出依据）。
pub fn take_serve_fatal_error() -> Option<String> {
    SERVE_FATAL.lock().ok().and_then(|mut s| s.take())
}

/// 只读快照（不消费）——run_file 内的钩子用它把 fatal 转成 Err 返回；
/// 外层 serve 入口随后 `take` 收割消息。take/snapshot 双面分离避免单次
/// take 被钩子吞掉后外层拿不到诊断。
pub fn serve_fatal_snapshot() -> Option<String> {
    SERVE_FATAL.lock().ok().and_then(|s| s.clone())
}

/// serve 入口在 ready 判定后取真实 bound 地址（port=0 场景必须用它）。
pub fn bound_addr_snapshot() -> Option<String> {
    SERVE_BOUND_ADDR.lock().ok().and_then(|s| s.clone())
}

/// 测试/serve 入口复位（进程内多 serve 序列时防陈旧值串场）。
pub fn reset_serve_state() {
    if let Ok(mut f) = SERVE_FATAL.lock() {
        *f = None;
    }
    if let Ok(mut b) = SERVE_BOUND_ADDR.lock() {
        *b = None;
    }
}

pub async fn serve_async(vm: std::rc::Rc<crate::vm::engine::AutoVM>, addr: &str) {
    // PLAN-699 T-03: Axum/Hyper HTTP/1.1 transport (Design 33 阶段 B). The VM
    // owner keeps this thread's `LocalSet` and consumes a bounded queue of
    // owned `ApiRequest`s; the network layer runs on a dedicated net thread
    // (super::http_transport). Default shutdown signals: Ctrl+C (+ SIGTERM on
    // unix); tests/embedders inject theirs through `serve_with`.
    let cfg = super::http_transport::TransportConfig::from_env();
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    // PLAN-705 T-05（仅测试面）：注册全局关停触发器——E2E 经
    // test_trigger_shutdown 注入优雅关停（默认信号面 Ctrl+C 不变）。
    #[cfg(test)]
    {
        let _ = TEST_SHUTDOWN_TX.set(shutdown_tx.clone());
    }
    #[cfg(not(test))]
    let _ = &shutdown_tx;
    // PLAN-736 T-05：/__auto/shutdown 控制面与信号汇入同一 watch。
    register_service_shutdown_tx(shutdown_tx.clone());
    tokio::task::spawn_local({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            eprintln!("[HTTP] Ctrl+C — shutting down gracefully");
            let _ = shutdown_tx.send(true);
        }
    });
    #[cfg(unix)]
    tokio::task::spawn_local({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            if let Ok(mut sig) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            {
                sig.recv().await;
                eprintln!("[HTTP] SIGTERM — shutting down gracefully");
                let _ = shutdown_tx.send(true);
            }
        }
    });
    serve_with(vm, addr, cfg, shutdown_rx).await;
}

/// PLAN-736 AC-03: 显式预算注入面——`auto service` 的 VM 轨用服务配置驱动
/// TransportConfig（env 不参与）；信号面与 `serve_async` 相同。
pub async fn serve_async_with(
    vm: std::rc::Rc<crate::vm::engine::AutoVM>,
    addr: &str,
    cfg: super::http_transport::TransportConfig,
) {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    #[cfg(test)]
    {
        let _ = TEST_SHUTDOWN_TX.set(shutdown_tx.clone());
    }
    #[cfg(not(test))]
    let _ = &shutdown_tx;
    register_service_shutdown_tx(shutdown_tx.clone());
    tokio::task::spawn_local({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            eprintln!("[HTTP] Ctrl+C — shutting down gracefully");
            let _ = shutdown_tx.send(true);
        }
    });
    #[cfg(unix)]
    tokio::task::spawn_local({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            if let Ok(mut sig) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            {
                sig.recv().await;
                eprintln!("[HTTP] SIGTERM — shutting down gracefully");
                let _ = shutdown_tx.send(true);
            }
        }
    });
    serve_with(vm, addr, cfg, shutdown_rx).await;
}

/// PLAN-705 T-05（仅测试面）：构造最小 GET ApiRequest（失效队列单元用）。
#[cfg(test)]
pub(crate) fn test_api_request_get(path: &str) -> ApiRequest {
    ApiRequest {
        method: "GET".to_string(),
        path: path.to_string(),
        headers: Vec::new(),
        body: Vec::new(),
        raw_upload_body: None,
        peer: None,
    }
}

/// PLAN-705 T-05（仅测试面）：serve_async 注册的最新实例关停发送端。
#[cfg(test)]
static TEST_SHUTDOWN_TX: std::sync::OnceLock<tokio::sync::watch::Sender<bool>> =
    std::sync::OnceLock::new();

/// PLAN-705 T-05（仅测试面）：触发 serve_async 已注册实例的优雅关停。
#[cfg(test)]
pub(crate) fn test_trigger_shutdown() -> bool {
    TEST_SHUTDOWN_TX
        .get()
        .map(|tx| tx.send(true).is_ok())
        .unwrap_or(false)
}

/// Injectable-shutdown entry (tests / embedders). Owns the VM for the server
/// lifetime; returns after the shutdown flag fired and draining finished
/// (port released).
async fn serve_with(
    vm: std::rc::Rc<crate::vm::engine::AutoVM>,
    addr: &str,
    cfg: super::http_transport::TransportConfig,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    // PLAN-730 T-05：安装上传宿主 executor（幂等；a2r facade 的 receive/
    // commit/reject 经此执行——VM 桥与生成 Rust 服务共用单源）。
    crate::http_upload_service::install_upload_executor_service();
    let routes = get_routes();
    let (req_tx, mut req_rx) = tokio::sync::mpsc::channel::<(
        ApiRequest,
        tokio::sync::oneshot::Sender<ApiReply>,
        u64, // scope id (PLAN-705 T-05)
    )>(cfg.queue_capacity);

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<Result<String, String>>();
    let net_addr = addr.to_string();
    let net_cfg = cfg.clone();
    let net_shutdown = shutdown_rx.clone();
    let spawned = std::thread::Builder::new()
        .name("auto-http-net".to_string())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = ready_tx.send(Err(format!("net runtime: {e}")));
                    return;
                }
            };
            rt.block_on(super::http_transport::serve_network(
                net_addr,
                net_cfg,
                req_tx,
                net_shutdown,
                ready_tx,
            ));
        });
    if spawned.is_err() {
        eprintln!("[HTTP] failed to spawn the auto-http-net thread");
        record_serve_fatal("failed to spawn the auto-http-net thread");
        return;
    }
    match ready_rx.await {
        Ok(Ok(bound)) => {
            // PLAN-736: 真实 bound 地址（port=0 时为内核分配值）供 serve 入口/
            // health 身份消费。
            if let Ok(mut slot) = SERVE_BOUND_ADDR.lock() {
                *slot = Some(bound.clone());
            }
            // PLAN-736 T-05/AC-02：bind+初始化完成 → Ready（身份四元组）。
            // 业务路由/VM 初始化在此之前已发生（run_file 编译先于 serve hook）。
            if let Some(svc) = &cfg.service {
                let c = &svc.config;
                crate::http_service_observability::set_service_identity(
                    &bound,
                    c.profile.as_str(),
                    c.effective_config_hash(),
                );
                crate::http_service_observability::set_service_state(
                    crate::http_service_observability::ServiceState::Ready,
                );
            }
        }
        Ok(Err(e)) => {
            eprintln!("[HTTP] Async server bind failed on {}: {}", addr, e);
            record_serve_fatal(format!("bind failed on {addr}: {e}"));
            return;
        }
        Err(_) => {
            eprintln!("[HTTP] net thread died before binding");
            record_serve_fatal("net thread died before binding");
            return;
        }
    }
    eprintln!(
        "[HTTP] Axum/Hyper transport listening on {} ({} route(s), VM owner thread)",
        addr,
        routes.len()
    );

    // Owner loop (PLAN-705 T-04): 事件驱动的段调度——请求到达即段驱动
    // （首 park 前同步完成），park 后挂入 parked 表；完成通知
    // （COMPLETION_NOTIFY，T-02 完成端写点发出）唤醒后按 park 顺序恢复
    // 就绪条目。零固定间隔轮询；enable→复查→await 三段式消除丢唤醒窗口
    // （spike 已证）。等待上游的请求不再占住 owner——期间新请求/健康
    // 检查照常服务（AC-01）。CPU 段仍不可抢占（10M 指令预算内跑完），
    // 这是 Design 33 §C 的既有边界。
    let mut shutdown_rx = shutdown_rx;
    let mut parked: Vec<ParkedRequest> = Vec::new();
    let completion_notify = &crate::vm::ffi::async_http::COMPLETION_NOTIFY;
    let mut draining = false;
    let mut loop_result = ();
    loop {
        // (0) 事件驱动扫描：scope 失效废弃 + 恢复就绪 parked（本轮通知/
        // 事件唤醒后的消费点——取消信号同经 COMPLETION_NOTIFY）。
        drain_ready_parked(&vm, &mut parked);
        // 最早 deadline 定时臂（parked 非空时唤醒失效检查，防无事件悬挂）。
        let next_deadline = parked
            .iter()
            .filter_map(|p| lookup_scope(p.scope_id))
            .map(|s| tokio::time::Instant::from_std(*s.deadline.read().unwrap()))
            .min();
        // (1) 注册完成通知兴趣（Notified 首次 poll 才登记 waiter，enable()
        // 把登记提前到检查之前）。
        let notify_fut = completion_notify.notified();
        tokio::pin!(notify_fut);
        notify_fut.as_mut().enable();
        // (2) enable 后复查（覆盖"通知在登记前发出"的窗口）。
        if parked.iter().any(|p| parked_is_ready(&vm, p)) {
            continue;
        }
        // (3) 事件等待：新请求 / 完成通知 / 关闭。
        if draining {
            // 关闭排水窗：继续应答已排队请求；parked 请求等待其完成或
            // 排水截止（超时统一取消——T-05 的 scope 取消语义；窗宽随
            // cfg，AUTO_HTTP_SHUTDOWN_DRAIN_MS 可调）。
            let drain_deadline = tokio::time::Instant::now() + cfg.shutdown_drain;
            tokio::select! {
                req = req_rx.recv() => match req {
                    Some((api_req, reply_tx, scope_id)) => {
                        dispatch_owner_request(&vm, &routes, api_req, reply_tx, scope_id, &mut parked);
                    }
                    None => {
                        cancel_all_parked(&vm, &mut parked);
                        break loop_result;
                    }
                },
                _ = &mut notify_fut => {}
                _ = tokio::time::sleep_until(drain_deadline) => {
                    cancel_all_parked(&vm, &mut parked);
                    break loop_result;
                }
            }
        } else {
            tokio::select! {
                req = req_rx.recv() => match req {
                    Some((api_req, reply_tx, scope_id)) => {
                        dispatch_owner_request(&vm, &routes, api_req, reply_tx, scope_id, &mut parked);
                    }
                    None => {
                        // net 侧发送端已清（端口释放）——放弃 parked
                        //（连接已不在，回复无处投递）。
                        cancel_all_parked(&vm, &mut parked);
                        break loop_result;
                    }
                },
                _ = &mut notify_fut => {}
                _ = async {
                    match next_deadline {
                        Some(d) => tokio::time::sleep_until(d).await,
                        None => std::future::pending::<()>().await,
                    }
                } => {
                    // 最早 parked deadline 到期：回到循环顶做失效废弃。
                }
                _ = shutdown_rx.changed() => {
                    // 排水窗开启：net 侧同步停收（watch 广播同源），已排队
                    // 请求继续应答；parked 请求等待完成或排水截止。
                    draining = true;
                    if cfg.service.is_some() {
                        crate::http_service_observability::set_service_state(
                            crate::http_service_observability::ServiceState::Draining,
                        );
                    }
                }
            }
        }
    }
    if cfg.service.is_some() {
        crate::http_service_observability::set_service_state(
            crate::http_service_observability::ServiceState::Stopped,
        );
    }
    eprintln!("[HTTP] VM owner loop exited (port released)");
}

/// owner 出队派发：scope 失效（取消/过期）的排队请求**跳过业务函数**
/// 直接 503（AC-04 失效队列不执行）；有效则置 RUNNING 并段驱动。
pub(crate) fn dispatch_owner_request(
    vm: &std::rc::Rc<AutoVM>,
    routes: &[HttpRoute],
    api_req: ApiRequest,
    reply_tx: tokio::sync::oneshot::Sender<ApiReply>,
    scope_id: u64,
    parked: &mut Vec<ParkedRequest>,
) {
    let scope = lookup_scope(scope_id);
    let usable = scope.as_ref().map(|s| scope_usable(s)).unwrap_or(false);
    if !usable {
        // 失效队列：终结 scope（释放许可）+ 503，不调用 handler。
        if let Some(s) = &scope {
            cancel_scope(s);
        }
        let _ = reply_tx.send(ApiReply::Full {
            status: 503,
            headers: json_reply_headers(""),
            body: ApiBody::Text(br#"{"error":"request cancelled"}"#.to_vec()),
        });
        return;
    }
    if let Some(s) = &scope {
        s.state
            .store(SCOPE_RUNNING, std::sync::atomic::Ordering::SeqCst);
    }
    match dispatch_api_request_segment(vm, routes, api_req, reply_tx, scope_id) {
        DispatchOutcome::Replied => {}
        DispatchOutcome::Parked(p) => {
            if let Some(s) = lookup_scope(scope_id) {
                s.state
                    .store(SCOPE_PARKED, std::sync::atomic::Ordering::SeqCst);
            }
            parked.push(*p);
        }
    }
}

/// 关闭/退出时放弃全部 parked 请求：503 终态回复、任务清理、未完成的
/// live-op 回收（迟到的 worker 完成被 presence 守卫丢弃，无泄漏面）。
fn cancel_all_parked(vm: &std::rc::Rc<AutoVM>, parked: &mut Vec<ParkedRequest>) {
    for mut p in parked.drain(..) {
        // PLAN-730 T-07：关闭废弃的 provisional 文件同样删除。
        cleanup_legacy_files(&p.ctx.legacy_stored);
        match &p.wait {
            ParkedWait::HttpRequest(req_id) => {
                crate::vm::ffi::stdlib::drop_async_result(*req_id);
            }
            // PLAN-707 T-05: 放弃 parked 流等待 → 取消上游流（收口）。
            ParkedWait::HttpStream(stream_id) => {
                crate::vm::ffi::http_stream::stream_cancel(*stream_id);
            }
            ParkedWait::Future(_) => {}
            // PLAN-711 T-11: CPU continuation 无 I/O 资源可取消——栈随任务
            // 弃置清账（HTTP 轨不产 CPU continuation，防御臂）。
            ParkedWait::CpuRunnable => {}
        }
        // T-05: scope 幂等终结（取消信号唤醒等它的桥臂；许可释放）。
        if let Some(s) = lookup_scope(p.scope_id) {
            cancel_scope(&s);
        }
        vm.tasks.remove(&p.task_id);
        if let Some(tx) = p.reply_tx.take() {
            let _ = tx.send(ApiReply::Full {
                status: 503,
                headers: json_reply_headers(&p.ctx.request_id),
                body: ApiBody::Text(br#"{"error":"server shutting down"}"#.to_vec()),
            });
        }
    }
}

/// ============================================================================
/// PLAN-705 T-04/SD-01: 段驱动的 owner dispatch——可 park 的请求状态机
/// ============================================================================
///
/// 请求生命周期从"一次同步 fn 调用"升级为可跨 park 的阶段机：
/// 预处理（CORS/限流/路由/WS，纯同步）→ middleware 链（逐个段驱动）→
/// handler 段驱动（named / `__axum:` closure）→ 编组（SSE / Response
/// handle / ~T 最终值 / JSON）。任一段 Yield（异步 HTTP / External
/// Future 等待）即返回 [`DispatchOutcome::Parked`]，owner loop 把它挂入
/// parked 表，由 [`COMPLETION_NOTIFY`] 唤醒后经 [`resume_parked_request`]
/// 续跑。等待期间 owner 可自由服务其他请求（AC-01：等待上游时健康请求
/// 仍可完成）。
///
/// 所有用到的 VM 状态访问都发生在 owner 线程（AutoVM !Send 契约不变）；
/// 跨线程仍只有 owned `ApiRequest`/`ApiReply`。单执行段顺序执行、跨
/// await 非原子事务（SD-01 契约：副作用不因取消/交错回滚）。
use crate::vm::engine::{AutoVM, ParkedSegment, ParkedWait, SegmentOutcome};

/// 编组延续所需的全部请求上下文（park 后恢复重建 reply 的最小集）。
pub(crate) struct DispatchCtx {
    pub req_method: String,
    pub req_path: String,
    pub request_id: String,
    /// 解码后的 body 字符串（multipart 已折叠为 JSON 时即其内容）。
    pub body: String,
    pub content_type: String,
    #[allow(dead_code)]
    pub content_type_raw: String,
    pub cookie_header: String,
    pub auth_header: String,
    pub multipart_json: Option<String>,
    pub route: RouteMatch,
    pub axum_route: Option<crate::vm::ffi::axum_adapter::AxumRoute>,
    pub started: std::time::Instant,
    pub middleware_names: Vec<String>,
    /// middleware request-info JSON（method/path/ct/has_body/request_id）。
    pub request_info: String,
    /// handler 任务 id（Handler / AwaitReturnFuture 阶段有效；终态移除）。
    pub handler_task_id: Option<u64>,
    /// 请求作用域 id（T-05：owner 派发/恢复前的失效检查）。
    pub scope_id: u64,
    /// PLAN-730 T-05：上传路由的延迟 body 能力（原始流 + 请求头快照）；
    /// start_handler 按 UploadRequest 参数注入消费（一次性）。
    pub upload_pending: Option<PendingUpload>,
    /// PLAN-730 T-07：legacy multipart 解析产物（纯内存，dispatch 段）；
    /// 落盘在 middleware 链后经宿主执行（ParkStage::LegacyStore）。
    pub legacy_parts: Option<Vec<MultipartPart>>,
    /// PLAN-730 T-07：已折叠的文本字段 JSON（落盘完成后与 files 合并）。
    pub legacy_fields_json: String,
    /// PLAN-730 T-07：本次请求新建的 provisional 文件（绑定失败/handler
    /// Err/取消 → 删除；成功回复按历史语义保留）。
    pub legacy_stored: Vec<String>,
}

/// PLAN-730 T-05：bridge→handler 的上传 body 能力（未授权不解析/不落盘）。
pub(crate) struct PendingUpload {
    pub body: axum::body::Body,
    pub headers: Vec<(String, String)>,
}

/// park 的延续点。
#[derive(Debug, Clone, Copy)]
pub(crate) enum ParkStage {
    /// `middleware_names[index]` 的段挂起中。
    Middleware { index: usize },
    /// handler 段挂起中。
    Handler,
    /// handler 已 RET；等待 `~T` 返回 future（future id 在 `wait` 里）。
    AwaitReturnFuture,
    /// `~{}` 内部 future 体在 handler RET 后由服务端驱动，中途挂在外层
    /// External await 上（wait=外层 future id；internal_fid=内部 future）。
    AsyncReturnBody { internal_fid: u32 },
    /// PLAN-730 T-07：legacy multipart 落盘在宿主 spawn_blocking 执行中
    ///（middleware 已过；live-op 完成后合并 files JSON 进 body 再进 handler）。
    LegacyStore { op_id: u64 },
}

/// 一次跨 park 的请求延续（owner loop parked 表的条目）。
pub(crate) struct ParkedRequest {
    pub reply_tx: Option<tokio::sync::oneshot::Sender<ApiReply>>,
    /// 挂起中的任务 id（仍在 `vm.tasks` 注册表；单 owner try_lock 访问）。
    pub task_id: u64,
    /// 请求作用域 id（取消/deadline 失效检查——T-05）。
    pub scope_id: u64,
    pub seg: ParkedSegment,
    /// 就绪凭据（HttpRequest 的 live-op / Future 的 vm.futures）。
    pub wait: ParkedWait,
    pub stage: ParkStage,
    pub ctx: DispatchCtx,
}

/// 段入口的产出：回复已发出（reply_tx 已消费），或请求 parked。
pub(crate) enum DispatchOutcome {
    Replied,
    Parked(Box<ParkedRequest>),
}

/// owner dispatch 段形态入口：预处理 + middleware + handler，跑到回复或
/// 首 park。回复经 `reply_tx` 发出（Replied 时 tx 已消费）。
pub(crate) fn dispatch_api_request_segment(
    vm: &std::rc::Rc<AutoVM>,
    routes: &[HttpRoute],
    mut req: ApiRequest,
    reply_tx: tokio::sync::oneshot::Sender<ApiReply>,
    scope_id: u64,
) -> DispatchOutcome {
    let req_method = req.method.to_uppercase();
    let req_path = req.path.clone();

    if let Some(_preflight) = handle_cors_preflight(&req_method) {
        let _ = reply_tx.send(ApiReply::Full {
            status: 204,
            headers: cors_header_pairs(),
            body: ApiBody::Text(Vec::new()),
        });
        return DispatchOutcome::Replied;
    }

    let content_type_raw = req.header("content-type").unwrap_or("").to_string();
    let content_type = content_type_raw.to_ascii_lowercase();
    let cookie_header = req.header("cookie").unwrap_or("").to_string();
    let auth_header = req.header("authorization").unwrap_or("").to_string();
    let incoming_request_id = req.header("x-request-id").unwrap_or("").to_string();
    let is_websocket = req
        .header("upgrade")
        .map(|v| v.to_ascii_lowercase().contains("websocket"))
        .unwrap_or(false);

    // PLAN-736 T-04/AC-05: 入站 request-id 校验——1..64 可见 ASCII 安全子集
    // （[0-9A-Za-z._:@-]）；不合者重生成（原样透传 = 日志注入面）。
    fn request_id_ok(id: &str) -> bool {
        !id.is_empty()
            && id.len() <= 64
            && id
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '@' | '-'))
    }
    let request_id = if request_id_ok(&incoming_request_id) {
        incoming_request_id
    } else {
        gen_request_id()
    };

    let client_ip = req
        .peer
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    if let Some(retry_after_ms) = rate_limit_take(&client_ip) {
        let body = format!(
            "{{\"error\":\"rate limit exceeded\",\"retry_after_ms\":{}}}",
            retry_after_ms
        );
        let mut headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "Retry-After".to_string(),
                ((retry_after_ms + 999) / 1000).to_string(),
            ),
        ];
        headers.extend(cors_header_pairs());
        headers.push(("X-Request-Id".to_string(), request_id.clone()));
        eprintln!(
            "[HTTP] {} {} [{}] → 429 rate limited (ip {})",
            req_method, req_path, request_id, client_ip
        );
        let _ = reply_tx.send(ApiReply::Full {
            status: 429,
            headers,
            body: ApiBody::Text(body.into_bytes()),
        });
        return DispatchOutcome::Replied;
    }

    let body = String::from_utf8_lossy(&req.body).into_owned();

    // PLAN-730 T-05/T-07：路由先行（404/方法不匹配零解析零写盘）；上传
    // 路由按类型分类跳过 legacy multipart（body 为延迟能力，不在此解析）。
    let route_match = match match_route(routes, &req_method, &req_path) {
        Some(rm) => rm,
        None => {
            let mut headers = cors_header_pairs();
            headers.push(("X-Request-Id".to_string(), request_id.clone()));
            let _ = reply_tx.send(ApiReply::Full {
                status: 404,
                headers,
                body: ApiBody::Text(Vec::new()),
            });
            return DispatchOutcome::Replied;
        }
    };
    let is_upload_route = route_declares_upload(&route_match.fn_name);
    let upload_headers = req.headers.clone();
    let upload_pending = if is_upload_route {
        if !matches!(req_method.as_str(), "POST" | "PUT") {
            let mut headers = cors_header_pairs();
            headers.push(("X-Request-Id".to_string(), request_id.clone()));
            headers.push(("Allow".to_string(), "POST, PUT".to_string()));
            let _ = reply_tx.send(ApiReply::Full {
                status: 405,
                headers,
                body: ApiBody::Text(
                    br#"{"error":"upload endpoints support POST/PUT only"}"#.to_vec(),
                ),
            });
            return DispatchOutcome::Replied;
        }
        match req.raw_upload_body.take() {
            Some(b) => Some(PendingUpload {
                body: b,
                headers: upload_headers,
            }),
            // bridge 与 owner 的分类不一致（非 transport 入口的派发）——契约
            // 破坏，fail-closed 500，不执行业务函数。
            None => {
                eprintln!(
                    "[HTTP] {} {} [{}] → 500 (upload route without body capability)",
                    req_method, req_path, request_id
                );
                let mut headers = cors_json_headers(&request_id);
                headers[0].1 = "application/json".to_string();
                let _ = reply_tx.send(ApiReply::Full {
                    status: 500,
                    headers,
                    body: ApiBody::Text(
                        br#"{"error":"upload route missing deferred body"}"#.to_vec(),
                    ),
                });
                return DispatchOutcome::Replied;
            }
        }
    } else {
        None
    };

    // PLAN-730 T-07：解析（纯内存，路由命中后）与落盘（middleware 后宿主
    // 执行）分离——404/method 不匹配/中间件拒绝零解析零写盘。
    let mut legacy_parts: Option<Vec<MultipartPart>> = None;
    let mut legacy_fields_json = String::new();
    let mut multipart_json: Option<String> = None;
    if !is_upload_route && content_type.starts_with("multipart/form-data") {
        let boundary = content_type_raw
            .split(';')
            .find_map(|p| p.trim().strip_prefix("boundary="))
            .map(|b| b.trim_matches('"').to_string());
        if let Some(boundary) = boundary {
            let parts = parse_multipart(&req.body, &boundary);
            legacy_fields_json = multipart_fields_json(&parts);
            let has_files = parts.iter().any(|p| p.filename.is_some());
            if has_files {
                legacy_parts = Some(parts);
            } else {
                multipart_json = Some(multipart_handler_json(&legacy_fields_json, ""));
            }
            eprintln!(
                "[HTTP] {} {} [{}] multipart: {} bytes parsed (store deferred)",
                req_method,
                req_path,
                request_id,
                req.body.len()
            );
        } else {
            eprintln!(
                "[HTTP] {} {} [{}] multipart without boundary — ignored",
                req_method, req_path, request_id
            );
        }
    }

    // PLAN-729 T-04：文件端点仅 GET/HEAD（决策报告 §4）——非 GET/HEAD 的
    // 文件返回注解在路由命中处即 405（不执行 handler）。
    if fn_is_api_file_return(&route_match.fn_name) && !matches!(req_method.as_str(), "GET" | "HEAD")
    {
        let mut headers = cors_header_pairs();
        headers.push(("X-Request-Id".to_string(), request_id.clone()));
        headers.push(("Allow".to_string(), "GET, HEAD".to_string()));
        let _ = reply_tx.send(ApiReply::Full {
            status: 405,
            headers,
            body: ApiBody::Text(br#"{"error":"file endpoints support GET/HEAD only"}"#.to_vec()),
        });
        return DispatchOutcome::Replied;
    }

    if is_websocket {
        if let Some(key) = req.header("sec-websocket-key") {
            let _ = reply_tx.send(ApiReply::WebSocket {
                accept: compute_ws_accept(key),
            });
            return DispatchOutcome::Replied;
        }
    }

    let request_info = format!(
        r#"{{"method":"{}","path":"{}","content_type":"{}","has_body":{},"request_id":"{}"}}"#,
        req_method,
        req_path,
        content_type,
        !body.is_empty() || upload_pending.is_some(),
        request_id
    );
    let middleware_names: Vec<String> = crate::vm::ffi::stdlib::MIDDLEWARE_CHAIN
        .lock()
        .map(|c| c.clone())
        .unwrap_or_default();

    let ctx = DispatchCtx {
        req_method,
        req_path,
        request_id,
        body,
        content_type,
        content_type_raw,
        cookie_header,
        auth_header,
        multipart_json,
        route: route_match,
        axum_route: None,
        started: std::time::Instant::now(),
        middleware_names,
        request_info,
        handler_task_id: None,
        scope_id,
        upload_pending,
        legacy_parts,
        legacy_fields_json,
        legacy_stored: Vec::new(),
    };
    advance_dispatch(vm, ctx, 0, Some(reply_tx))
}

/// 段状态机的推进核心：从 middleware `mw_start` 起跑完剩余链 + handler +
/// 编组；首 park 即返回 Parked（reply_tx 随条目保存）。
fn advance_dispatch(
    vm: &std::rc::Rc<AutoVM>,
    mut ctx: DispatchCtx,
    mw_start: usize,
    mut reply_tx: Option<tokio::sync::oneshot::Sender<ApiReply>>,
) -> DispatchOutcome {
    // ── middleware 链（逐个段驱动；Err → 视为无响应继续，legacy 语义）──
    let mut index = mw_start;
    while index < ctx.middleware_names.len() {
        match run_middleware_at(vm, &ctx, index) {
            MWStep::Continue => index += 1,
            MWStep::Reply(reply) => {
                eprintln!(
                    "[HTTP] {} {} [{}] → MW ({}ms)",
                    ctx.req_method,
                    ctx.req_path,
                    ctx.request_id,
                    ctx.started.elapsed().as_millis()
                );
                if let Some(tx) = reply_tx.take() {
                    let _ = tx.send(reply);
                }
                return DispatchOutcome::Replied;
            }
            MWStep::Parked { task_id, seg, wait } => {
                return DispatchOutcome::Parked(Box::new(ParkedRequest {
                    reply_tx,
                    task_id,
                    scope_id: ctx.scope_id,
                    seg,
                    wait,
                    stage: ParkStage::Middleware { index },
                    ctx,
                }));
            }
        }
    }
    // PLAN-730 T-07：legacy multipart 落盘——middleware 全过后才执行（有界
    // 宿主 spawn_blocking + live-op 唤醒；owner 不做阻塞 std::fs::write）。
    if let Some(parts) = ctx.legacy_parts.take() {
        let files: Vec<(String, String, Vec<u8>)> = parts
            .iter()
            .filter_map(|p| {
                p.filename
                    .as_ref()
                    .map(|f| (p.name.clone(), f.clone(), p.data.clone()))
            })
            .collect();
        let op_id = crate::vm::ffi::stdlib::alloc_async_id();
        crate::vm::ffi::async_http::register_live_op(op_id);
        LEGACY_STORE_RESULTS
            .lock()
            .unwrap()
            .insert(op_id, None);
        tokio::task::spawn_blocking(move || {
            let result = legacy_store_files(&files).map(|stored| {
                (
                    stored
                        .iter()
                        .map(|f| f.path.clone())
                        .collect::<Vec<String>>(),
                    multipart_files_json(&stored),
                )
            });
            if let Ok(mut m) = LEGACY_STORE_RESULTS.lock() {
                m.insert(op_id, Some(result));
            }
            // 唤醒协议同 live-op：迟到（取消后）完成被 presence 守卫丢弃，
            // 结果条目随 abort 清理（consumed=Some 由 resume/abort 取走）。
            if !crate::vm::ffi::async_http::complete_live_op(
                op_id,
                Ok(crate::vm::ffi::stdlib::AsyncResult::Body(String::new())),
            ) {
                if let Ok(mut m) = LEGACY_STORE_RESULTS.lock() {
                    let _ = m.remove(&op_id);
                }
            }
        });
        return DispatchOutcome::Parked(Box::new(ParkedRequest {
            reply_tx,
            task_id: 0, // 无 VM 任务在执行（存储在宿主）
            scope_id: ctx.scope_id,
            seg: ParkedSegment {
                fn_name: ctx.route.fn_name.clone(),
                saved_bp: 0,
                saved_fn_n_args: 0,
            },
            wait: ParkedWait::HttpRequest(op_id),
            stage: ParkStage::LegacyStore { op_id },
            ctx,
        }));
    }
    start_handler(vm, ctx, reply_tx)
}

/// PLAN-730 T-07：落盘完成后的续跑（middleware 已在 park 前全过——从链尾
/// 直达 handler）。
fn advance_after_legacy_store(
    vm: &std::rc::Rc<AutoVM>,
    p: &mut ParkedRequest,
) -> ParkedResume {
    let ctx = std::mem::replace(&mut p.ctx, placeholder_ctx());
    match start_handler(vm, ctx, p.reply_tx.take()) {
        DispatchOutcome::Replied => ParkedResume::Consumed,
        DispatchOutcome::Parked(boxed) => {
            // handler 段 park：延续条目（provisional 路径随 ctx 转移）。
            let mut b = *boxed;
            p.task_id = b.task_id;
            p.scope_id = b.scope_id;
            p.seg = b.seg.clone();
            p.wait = b.wait;
            p.stage = b.stage;
            p.ctx = b.ctx;
            ParkedResume::StillParked
        }
    }
}

/// PLAN-730 T-07：legacy 落盘结果表（op_id → Option<Result>；None=Pending）。
lazy_static::lazy_static! {
    static ref LEGACY_STORE_RESULTS: std::sync::Mutex<
        std::collections::HashMap<u64, Option<Result<(Vec<String>, String), String>>>,
    > = std::sync::Mutex::new(std::collections::HashMap::new());
}

/// PLAN-730 T-07：provisional 文件清理（绑定失败/handler Err/取消路径）。
fn cleanup_legacy_files(paths: &[String]) {
    for path in paths {
        if let Err(e) = std::fs::remove_file(path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("[HTTP] legacy provisional cleanup failed for {path}: {e}");
            }
        }
    }
}

enum MWStep {
    Continue,
    Reply(ApiReply),
    Parked {
        task_id: u64,
        seg: ParkedSegment,
        wait: ParkedWait,
    },
}

/// 运行 middleware_names[index]（spawn 任务 + 段驱动）。上传路由的执行
/// 错误 fail-closed（500 短路、零落盘）——不沿用 legacy Err→Continue。
fn run_middleware_at(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx, index: usize) -> MWStep {
    let mw_fn = ctx.middleware_names[index].clone();
    let mw_task_id = vm.spawn_task(0, 65536);
    if let Some(t_arc) = vm.tasks.get(&mw_task_id) {
        if let Ok(mut t) = t_arc.try_lock() {
            push_str_arg(vm, &mut t, &ctx.request_info);
        }
    }
    enum MWEnd {
        Null,
        Response(String),
        Parked(ParkedWait, ParkedSegment),
    }
    let fail_closed = ctx.upload_pending.is_some();
    let end = {
        let Some(t_arc) = vm.tasks.get(&mw_task_id) else {
            vm.tasks.remove(&mw_task_id);
            return MWStep::Continue; // legacy：任务槽缺失 = 无响应继续
        };
        let mut t = match t_arc.try_lock() {
            Ok(t) => t,
            Err(_) => {
                return MWStep::Continue; // legacy：锁忙 = break 静默跳过
            }
        };
        match vm.call_fn_by_name_segment(&mut t, &mw_fn, 1) {
            SegmentOutcome::Completed(Ok(())) => {
                let nv = t.ram.pop_nv();
                if auto_val::is_null(nv) {
                    MWEnd::Null
                } else {
                    match nv_to_json(vm, nv, 0) {
                        Some(resp) if !resp.is_empty() && resp != "null" => MWEnd::Response(resp),
                        _ => MWEnd::Null,
                    }
                }
            }
            // legacy：middleware Err → None → 继续链（行为保持）。上传路由
            // fail-closed：Err → 500（决策 §4.4——Err 静默放行不可接受）。
            SegmentOutcome::Completed(Err(_)) => {
                if fail_closed {
                    return MWStep::Reply(ApiReply::Full {
                        status: 500,
                        headers: cors_json_headers(&ctx.request_id),
                        body: ApiBody::Text(
                            br#"{"error":"middleware failed"}"#.to_vec(),
                        ),
                    });
                }
                MWEnd::Null
            }
            SegmentOutcome::Parked { wait, seg } => MWEnd::Parked(wait, seg),
            // PLAN-711 T-11: Runnable 仅由 CpuSlice 驱动产生；HTTP server
            // 走 legacy 段契约（非 UI 轨），防御臂按 Err 同族继续链。
            SegmentOutcome::Runnable { .. } => MWEnd::Null,
        }
    };
    match end {
        MWEnd::Parked(wait, seg) => MWStep::Parked {
            task_id: mw_task_id,
            seg,
            wait,
        },
        MWEnd::Null => {
            vm.tasks.remove(&mw_task_id);
            MWStep::Continue
        }
        MWEnd::Response(resp) => {
            vm.tasks.remove(&mw_task_id);
            let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
            headers.extend(cors_header_pairs());
            headers.push(("X-Request-Id".to_string(), ctx.request_id.clone()));
            MWStep::Reply(ApiReply::Full {
                status: 200,
                headers,
                body: ApiBody::Text(resp.into_bytes()),
            })
        }
    }
}

enum HandlerEnd {
    Value(auto_val::NanoValue),
    Err(crate::vm::engine::VMError),
    Parked(ParkedWait, ParkedSegment),
    /// 任务槽缺失（legacy contract：空体 200）。
    MissingTask,
    /// tokio Mutex try_lock 失败（守卫出作用域后再落 500 副作用）。
    LockBusy,
}

/// handler 阶段：spawn 任务 → 绑定 → 段驱动调用 → 编组/park。
fn start_handler(
    vm: &std::rc::Rc<AutoVM>,
    mut ctx: DispatchCtx,
    reply_tx: Option<tokio::sync::oneshot::Sender<ApiReply>>,
) -> DispatchOutcome {
    let handler_task_id = vm.spawn_task(0, 65536);
    ctx.handler_task_id = Some(handler_task_id);

    let axum_route = if ctx.route.fn_name.starts_with("__axum:") {
        match crate::vm::ffi::axum_adapter::route_by_synthetic_name(&ctx.route.fn_name) {
            Some(r) => Some(r),
            None => {
                eprintln!(
                    "[HTTP] {} {} → 500 (axum route lookup failed for {})",
                    ctx.req_method, ctx.req_path, ctx.route.fn_name
                );
                vm.tasks.remove(&handler_task_id);
                let mut headers =
                    vec![("Content-Type".to_string(), "application/json".to_string())];
                headers.extend(cors_header_pairs());
                let reply = ApiReply::Full {
                    status: 500,
                    headers,
                    body: ApiBody::Text(br#"{"error":"route lookup failed"}"#.to_vec()),
                };
                if let Some(tx) = reply_tx {
                    let _ = tx.send(reply);
                }
                return DispatchOutcome::Replied;
            }
        }
    } else {
        None
    };
    ctx.axum_route = axum_route.clone();

    // PLAN-730 T-05：UploadRequest 注入——路由声明该参数类型时，把延迟
    // body 能力建成宿主 UploadRequest（一次性；scope 组绑定）。分类与
    // bridge 同源（方法+参数类型），不一致在 dispatch 段已 fail-closed。
    let upload_handle: Option<i64> = match ctx.upload_pending.take() {
        Some(pending) => {
            let stream = {
                use futures::StreamExt;
                let data = pending.body.into_data_stream();
                let mapped = data.map(|r| {
                    r.map(|b| b.to_vec())
                        .map_err(|e| std::io::Error::other(e.to_string()))
                });
                let boxed: a2r_std::http::UploadBodyStream = Box::pin(mapped);
                boxed
            };
            let req = a2r_std::http::upload_request_from_parts(
                &ctx.req_method,
                &ctx.req_path,
                pending.headers,
                stream,
            );
            Some(super::http_upload::insert_upload_request(req) as i64)
        }
        None => None,
    };

    let n_args = if let Some(ref axum_route) = axum_route {
        let query_json = if ctx.route.query_params.is_empty() {
            "{}".to_string()
        } else {
            let pairs: Vec<String> = ctx
                .route
                .query_params
                .iter()
                .map(|(k, v)| {
                    format!(
                        "\"{}\":\"{}\"",
                        k.replace('"', "\\\""),
                        v.replace('"', "\\\"")
                    )
                })
                .collect();
            format!("{{{}}}", pairs.join(","))
        };
        let mut pushed = 0usize;
        if let Some(t_arc) = vm.tasks.get(&handler_task_id) {
            if let Ok(mut t) = t_arc.try_lock() {
                let headers_json = if ctx.auth_header.is_empty() {
                    "{}".to_string()
                } else {
                    format!(
                        "{{\"authorization\":\"{}\"}}",
                        ctx.auth_header.replace('"', "")
                    )
                };
                pushed = crate::vm::ffi::axum_adapter::push_extractor_args(
                    vm,
                    &mut t,
                    axum_route,
                    &ctx.route.path_params,
                    &query_json,
                    &ctx.body,
                    &headers_json,
                );
            }
        }
        pushed
    } else {
        match build_handler_args(
            vm,
            handler_task_id,
            &ctx.route,
            &ctx.body,
            &ctx.content_type,
            &ctx.cookie_header,
            &ctx.auth_header,
            ctx.multipart_json.as_deref(),
            &ctx.req_method,
            &ctx.req_path,
            upload_handle,
        ) {
            Ok(n) => n,
            Err(ApiArgBindError::BadRequest(msg)) => {
                eprintln!("[HTTP] {} {} → 400 ({})", ctx.req_method, ctx.req_path, msg);
                vm.tasks.remove(&handler_task_id);
                // PLAN-730 T-07：绑定失败——本次新建 provisional 文件删除。
                cleanup_legacy_files(&ctx.legacy_stored);
                let err_body = format!("{{\"error\":{}}}", json_escape_string(&msg));
                let mut headers =
                    vec![("Content-Type".to_string(), "application/json".to_string())];
                headers.extend(cors_header_pairs());
                let reply = ApiReply::Full {
                    status: 400,
                    headers,
                    body: ApiBody::Text(err_body.into_bytes()),
                };
                if let Some(tx) = reply_tx {
                    let _ = tx.send(reply);
                }
                return DispatchOutcome::Replied;
            }
            Err(ApiArgBindError::Internal(msg)) => {
                eprintln!("[HTTP] {} {} → 500 ({})", ctx.req_method, ctx.req_path, msg);
                vm.tasks.remove(&handler_task_id);
                cleanup_legacy_files(&ctx.legacy_stored);
                let err_body = format!("{{\"error\":{}}}", json_escape_string(&msg));
                let mut headers =
                    vec![("Content-Type".to_string(), "application/json".to_string())];
                headers.extend(cors_header_pairs());
                let reply = ApiReply::Full {
                    status: 500,
                    headers,
                    body: ApiBody::Text(err_body.into_bytes()),
                };
                if let Some(tx) = reply_tx {
                    let _ = tx.send(reply);
                }
                return DispatchOutcome::Replied;
            }
        }
    };

    // DashMap 读守卫纪律：守卫作用域内不得对同一 shard remove——失败形态
    // 编码为变体，守卫出作用域后再落副作用。
    let end = if let Some(t_arc) = vm.tasks.get(&handler_task_id) {
        match t_arc.try_lock() {
            Err(_) => HandlerEnd::LockBusy,
            Ok(mut ht) => {
                // Dispatch: axum routes use closure (Plan 383 fn-ref), legacy
                // use fn-name. PLAN-705: 段驱动形态——等待即 park，不再同步
                // 占线程。PLAN-707 T-06: 段执行期间绑定资源组（流 open 登记）。
                let _scope_bind = bind_current_scope(ctx.scope_id);
                match if let Some(ref axum_route) = axum_route {
                    vm.call_closure_segment(&mut ht, axum_route.closure_id, n_args)
                } else {
                    vm.call_fn_by_name_segment(&mut ht, &ctx.route.fn_name, n_args)
                } {
                    SegmentOutcome::Completed(Ok(())) => HandlerEnd::Value(ht.ram.pop_nv()),
                    SegmentOutcome::Completed(Err(e)) => HandlerEnd::Err(e),
                    SegmentOutcome::Parked { wait, seg } => HandlerEnd::Parked(wait, seg),
                    // PLAN-711 T-11: Runnable 仅由 CpuSlice 驱动产生；HTTP
                    // server 走 legacy 段契约（非 UI 轨），防御臂按 Err 同族。
                    SegmentOutcome::Runnable { .. } => {
                        HandlerEnd::Err(crate::vm::engine::VMError::RuntimeError(
                            "internal: cpu-slice outcome escaped legacy http dispatch (unreachable)"
                                .into(),
                        ))
                    }
                }
            }
        }
    } else {
        HandlerEnd::MissingTask
    };

    match end {
        HandlerEnd::MissingTask => {
            // legacy contract：任务槽缺失 → 空体 200。
            let reply = ApiReply::Full {
                status: 200,
                headers: json_reply_headers(&ctx.request_id),
                body: ApiBody::Text(Vec::new()),
            };
            if let Some(tx) = reply_tx {
                let _ = tx.send(reply);
            }
            return DispatchOutcome::Replied;
        }
        HandlerEnd::LockBusy => {
            eprintln!(
                "[HTTP] {} {} → 500 (task lock failed, {}ms)",
                ctx.req_method,
                ctx.req_path,
                ctx.started.elapsed().as_millis()
            );
            vm.tasks.remove(&handler_task_id);
            let reply = ApiReply::Full {
                status: 500,
                headers: cors_json_headers(&ctx.request_id),
                body: ApiBody::Text(br#"{"error":"internal error"}"#.to_vec()),
            };
            if let Some(tx) = reply_tx {
                let _ = tx.send(reply);
            }
            return DispatchOutcome::Replied;
        }
        HandlerEnd::Parked(wait, seg) => DispatchOutcome::Parked(Box::new(ParkedRequest {
            reply_tx,
            task_id: handler_task_id,
            scope_id: ctx.scope_id,
            seg,
            wait,
            stage: ParkStage::Handler,
            ctx,
        })),
        HandlerEnd::Err(e) => {
            vm.tasks.remove(&handler_task_id);
            // PLAN-730 T-07：handler 异常——本次新建 provisional 文件删除。
            cleanup_legacy_files(&ctx.legacy_stored);
            let reply = handler_error_reply(&ctx, &e);
            if let Some(tx) = reply_tx {
                let _ = tx.send(reply);
            }
            DispatchOutcome::Replied
        }
        HandlerEnd::Value(nv) => {
            match marshal_handler_value(vm, &ctx, nv) {
                MarshalOutcome::Reply(reply) => {
                    vm.tasks.remove(&handler_task_id);
                    if let Some(tx) = reply_tx {
                        let _ = tx.send(reply);
                    }
                    DispatchOutcome::Replied
                }
                MarshalOutcome::ParkFuture(fid) => {
                    // ~T 返回 future 未完成：handler 任务保留（终值编组需要
                    // 栈暂存），stage 切 AwaitReturnFuture，wait=Future(fid)
                    // 统一就绪探测。
                    DispatchOutcome::Parked(Box::new(ParkedRequest {
                        reply_tx,
                        task_id: handler_task_id,
                        scope_id: ctx.scope_id,
                        seg: ParkedSegment {
                            fn_name: ctx.route.fn_name.clone(),
                            saved_bp: 0,
                            saved_fn_n_args: 0,
                        },
                        wait: ParkedWait::Future(fid),
                        stage: ParkStage::AwaitReturnFuture,
                        ctx,
                    }))
                }
                MarshalOutcome::ParkBody(internal_fid, outer_fid) => {
                    DispatchOutcome::Parked(Box::new(ParkedRequest {
                        reply_tx,
                        task_id: handler_task_id,
                        scope_id: ctx.scope_id,
                        seg: ParkedSegment {
                            fn_name: ctx.route.fn_name.clone(),
                            saved_bp: 0,
                            saved_fn_n_args: 0,
                        },
                        wait: ParkedWait::Future(outer_fid),
                        stage: ParkStage::AsyncReturnBody { internal_fid },
                        ctx,
                    }))
                }
            }
        }
    }
}

/// 编组 handler 返回值：SSE / Response handle / ~T 最终值 / JSON。
/// ~T 判定为元数据门（fn_is_api_async，`__axum:` 反查导出名）+ future
/// bits 形态 + vm.futures 注册表存在性三重闸——普通 int 位模式永不误判
/// （AC-01 反例）。
enum MarshalOutcome {
    Reply(ApiReply),
    /// External 返回 future 未完成。
    ParkFuture(u32),
    /// Internal 体挂在外层 External await 上（internal/outer future id）。
    ParkBody(u32, u32),
}

/// park 时任务的外层等待 future id（体挂起时 waiting_future_id 即外层）。
fn ctx_await_outer(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx) -> Option<u32> {
    let task_id = ctx.handler_task_id?;
    let t_arc = vm.tasks.get(&task_id)?.clone();
    let t = t_arc.try_lock().ok()?;
    t.waiting_future_id
}

/// ctx 声明文件返回（axum_route closure 反查同 async 门）。
fn handler_declares_file_return(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx) -> bool {
    match ctx.axum_route {
        Some(ref r) => crate::vm::ffi::axum_adapter::export_name_for_closure(vm, r.closure_id)
            .map(|n| fn_is_api_file_return(&n))
            .unwrap_or(false),
        None => fn_is_api_file_return(&ctx.route.fn_name),
    }
}

/// ctx 的声明返回串（named fn 经 API_RETURN_TYPES；axum closure 反查导出名）。
fn declared_return(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx) -> String {
    let lookup = |name: &str| -> Option<String> {
        API_RETURN_TYPES.lock().ok().and_then(|t| t.get(name).cloned())
    };
    match ctx.axum_route {
        Some(ref r) => crate::vm::ffi::axum_adapter::export_name_for_closure(vm, r.closure_id)
            .and_then(|n| lookup(&n))
            .unwrap_or_default(),
        None => lookup(&ctx.route.fn_name).unwrap_or_default(),
    }
}

/// ctx 声明上传收据返回（axum_route closure 反查同门）。
fn handler_declares_upload_return(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx) -> bool {
    match ctx.axum_route {
        Some(ref r) => crate::vm::ffi::axum_adapter::export_name_for_closure(vm, r.closure_id)
            .map(|n| fn_is_api_upload_return(&n))
            .unwrap_or(false),
        None => fn_is_api_upload_return(&ctx.route.fn_name),
    }
}

/// 收据 → 真实 HTTP 回复（receipt.status + 有界 JSON；CORS + 请求 id）。
fn upload_receipt_reply(ctx: &DispatchCtx, receipt: a2r_std::http::UploadReceipt) -> ApiReply {
    eprintln!(
        "[HTTP] {} {} [{}] → {} ({}ms)",
        ctx.req_method,
        ctx.req_path,
        ctx.request_id,
        receipt.status,
        ctx.started.elapsed().as_millis()
    );
    let mut headers = json_reply_headers(&ctx.request_id);
    headers[0].1 = "application/json".to_string();
    ApiReply::Full {
        status: receipt.status,
        headers,
        body: ApiBody::Text(receipt.json.into_bytes()),
    }
}

/// 声明上传收据返回但值不是登记收据 → 500 诊断（不 JSON 200）。
fn upload_contract_mismatch_reply(ctx: &DispatchCtx, raw: u64) -> ApiReply {
    eprintln!(
        "[HTTP] {} {} [{}] → 500 (handler '{}' declares UploadReceipt but returned non-receipt value {})",
        ctx.req_method, ctx.req_path, ctx.request_id, ctx.route.fn_name, raw
    );
    let mut headers = json_reply_headers(&ctx.request_id);
    headers[0].1 = "application/json".to_string();
    ApiReply::Full {
        status: 500,
        headers,
        body: ApiBody::Text(br#"{"error":"handler did not return an UploadReceipt"}"#.to_vec()),
    }
}

/// 声明文件返回但值不是登记描述符 → 500 诊断（不 JSON 200；决策报告 §6）。
fn file_contract_mismatch_reply(ctx: &DispatchCtx, raw: u64) -> ApiReply {
    eprintln!(
        "[HTTP] {} {} [{}] → 500 (handler '{}' declares FileResponse but returned non-descriptor value {})",
        ctx.req_method, ctx.req_path, ctx.request_id, ctx.route.fn_name, raw
    );
    let mut headers = json_reply_headers(&ctx.request_id);
    let body = br#"{"error":"handler did not return a FileResponse descriptor"}"#.to_vec();
    headers[0].1 = "application/json".to_string();
    ApiReply::Full {
        status: 500,
        headers,
        body: ApiBody::Text(body),
    }
}

fn marshal_handler_value(
    vm: &std::rc::Rc<AutoVM>,
    ctx: &DispatchCtx,
    nv: auto_val::NanoValue,
) -> MarshalOutcome {
    if auto_val::is_i32(nv) {
        let bits = auto_val::decode_i32(nv);
        if (bits & 0xFF) == 0xF0 {
            let fid = ((bits as u32) >> 8) as u32;
            let handler_async = match ctx.axum_route {
                Some(ref r) => {
                    crate::vm::ffi::axum_adapter::export_name_for_closure(vm, r.closure_id)
                        .map(|n| fn_is_api_async(&n))
                        .unwrap_or(false)
                }
                None => fn_is_api_async(&ctx.route.fn_name),
            };
            if handler_async {
                if let Some(fut) = vm.futures.get(&fid) {
                    let (pending, is_external, body_offset) = {
                        let f = fut.read().unwrap();
                        (
                            f.state == crate::vm::engine::FutureState::Pending,
                            f.kind == crate::vm::engine::FutureKind::External,
                            f.body_offset,
                        )
                    };
                    if pending {
                        if is_external {
                            return MarshalOutcome::ParkFuture(fid);
                        }
                        // Internal `~{}`：体由 await 上下文驱动——handler 已
                        // RET，服务端就地驱动（handle_await_future）；体内部
                        // 再挂 External await 时转 AsyncReturnBody 阶段，由
                        // 完成通知唤醒后续跑体。
                        let task_id = ctx.handler_task_id;
                        if let Some(task_id) = task_id {
                            if let Some(t_arc) = vm.tasks.get(&task_id) {
                                if let Ok(mut t) = t_arc.try_lock() {
                                    let _ = vm.handle_await_future(&mut t, fid, body_offset);
                                }
                            }
                        }
                        let Some(fut2) = vm.futures.get(&fid) else {
                            return MarshalOutcome::Reply(final_value_reply(vm, ctx, None));
                        };
                        {
                            let f = fut2.read().unwrap();
                            if f.state == crate::vm::engine::FutureState::Ready {
                                let value = f.result.clone();
                                drop(f);
                                return MarshalOutcome::Reply(final_value_reply(vm, ctx, value));
                            }
                            if f.state == crate::vm::engine::FutureState::Failed {
                                drop(f);
                                return MarshalOutcome::Reply(final_value_reply(vm, ctx, None));
                            }
                        }
                        // 体仍挂在外层 await 上 → AsyncReturnBody 阶段。
                        if let Some(outer_fid) = ctx_await_outer(vm, ctx) {
                            return MarshalOutcome::ParkBody(fid, outer_fid);
                        }
                        return MarshalOutcome::Reply(final_value_reply(vm, ctx, None));
                    }
                    let value = fut.read().unwrap().result.clone();
                    return MarshalOutcome::Reply(final_value_reply(vm, ctx, value));
                }
            }
        }
        // PLAN-730 T-05：上传收据门（声明返回类型 + i32 + 登记命中三重闸；
        // 置于 iterator 检查之前——收据 id 与 iterator id 空间独立，普通
        // int 撞号仍走 JSON 兜底）。
        if handler_declares_upload_return(vm, ctx) {
            let rid = bits as u32 as u64;
            return MarshalOutcome::Reply(match super::http_upload::take_upload_receipt(rid) {
                Some(receipt) => upload_receipt_reply(ctx, receipt),
                None => upload_contract_mismatch_reply(ctx, rid),
            });
        }
        let iter_id = bits as u32;
        // PLAN-734 T-07 兼容回退：iterator/SSE 探测保持注册表命中制——
        // Plan 326 wire 契约的 SSE 链 handler 声明 int 返回（axum Sse.into_response
        // 返回持有 iterator 的句柄），声明门会破坏既有 e2e（sse_chain/302 实证）。
        // 普通 int 撞 id 的理论碰撞由 id 空间分离（iterator/Response 各自
        // 计数器）覆盖，AC-03 反例以 e2e 锁定。
        if vm.iterators.contains_key(&iter_id) {
            eprintln!(
                "[HTTP] {} {} → 200 SSE ({}ms)",
                ctx.req_method,
                ctx.req_path,
                ctx.started.elapsed().as_millis()
            );
            let (sender, receiver) = tokio::sync::mpsc::channel(1);
            let producer_vm = vm.clone();
            tokio::task::spawn_local(async move {
                produce_sse_frames(producer_vm, iter_id, sender).await;
            });
            return MarshalOutcome::Reply(ApiReply::Full {
                status: 200,
                headers: vec![
                    ("Content-Type".to_string(), "text/event-stream".to_string()),
                    ("Cache-Control".to_string(), "no-cache".to_string()),
                    ("Connection".to_string(), "keep-alive".to_string()),
                ],
                body: ApiBody::Sse(receiver),
            });
        }
        // PLAN-729 T-04：文件响应描述符（声明返回类型门 + 登记命中）。
        if handler_declares_file_return(vm, ctx) {
            return MarshalOutcome::Reply(
                match super::http_server_file::take_file_response(iter_id as u64) {
                    Some(descriptor) => ApiReply::Full {
                        status: 200, // 占位：File 臂由宿主 serve 决定
                        headers: Vec::new(),
                        body: ApiBody::File(FileReplySeed {
                            descriptor,
                            request_id: ctx.request_id.clone(),
                        }),
                    },
                    None => file_contract_mismatch_reply(ctx, iter_id as u64),
                },
            );
        }
        // PLAN-734 T-07 兼容回退：Response-object 探测保持注册表命中制——
        // Plan 346 wire 契约的 handler 声明 int 返回（无 Response 声明词汇），
        // 声明门会破坏既有 302/SSE-value e2e。普通 int 撞 handle id 的理论
        // 碰撞由句柄计数器基座（4M+ 段）与既有 AC 反例防线共同覆盖。
        if let Some(res) = crate::vm::ffi::stdlib::lookup_http_response(iter_id as u64) {
            return MarshalOutcome::Reply(response_handle_reply(ctx, res));
        }
        return MarshalOutcome::Reply(json_value_reply(
            vm,
            nv,
            &ctx.req_method,
            &ctx.req_path,
            &ctx.route.fn_name,
            &ctx.body,
            &ctx.request_id,
            ctx.started.elapsed().as_millis(),
        ));
    }
    if auto_val::is_i64(nv) {
        let handle = auto_val::decode_i64(nv) as u64;
        if let Some(res) = crate::vm::ffi::stdlib::lookup_http_response(handle) {
            return MarshalOutcome::Reply(response_handle_reply(ctx, res));
        }
        return MarshalOutcome::Reply(json_value_reply(
            vm,
            nv,
            &ctx.req_method,
            &ctx.req_path,
            &ctx.route.fn_name,
            &ctx.body,
            &ctx.request_id,
            ctx.started.elapsed().as_millis(),
        ));
    }
    MarshalOutcome::Reply(json_value_reply(
        vm,
        nv,
        &ctx.req_method,
        &ctx.req_path,
        &ctx.route.fn_name,
        &ctx.body,
        &ctx.request_id,
        ctx.started.elapsed().as_millis(),
    ))
}

/// Response-object 分支的共享编组（status/headers/body + CORS + 请求 id）。
fn response_handle_reply(
    ctx: &DispatchCtx,
    res: (u16, Vec<(String, String)>, Vec<u8>),
) -> ApiReply {
    let (status, headers, body) = res;
    let mut all_headers = headers;
    all_headers.extend(cors_header_pairs());
    all_headers.push(("X-Request-Id".to_string(), ctx.request_id.clone()));
    eprintln!(
        "[HTTP] {} {} [{}] → {} ({}ms)",
        ctx.req_method,
        ctx.req_path,
        ctx.request_id,
        status,
        ctx.started.elapsed().as_millis()
    );
    ApiReply::Full {
        status,
        headers: all_headers,
        body: ApiBody::Text(body),
    }
}

/// handler Err 的 500 形态（legacy 同款）。
fn handler_error_reply(ctx: &DispatchCtx, e: &crate::vm::engine::VMError) -> ApiReply {
    eprintln!(
        "[HTTP] {} {} → 500 (handler '{}' error: {:?}, {}ms)",
        ctx.req_method,
        ctx.req_path,
        ctx.route.fn_name,
        e,
        ctx.started.elapsed().as_millis()
    );
    let error_json = format!(
        r#"{{"error":"internal server error","detail":"{}"}}"#,
        format!("{:?}", e).replace('"', "\\\"").replace('\n', " ")
    );
    ApiReply::Full {
        status: 500,
        headers: json_reply_headers(&ctx.request_id),
        body: ApiBody::Text(error_json.into_bytes()),
    }
}

/// CORS + JSON content-type + 请求 id（task-lock 失败臂用）。
fn cors_json_headers(request_id: &str) -> Vec<(String, String)> {
    let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
    headers.extend(cors_header_pairs());
    headers.push(("X-Request-Id".to_string(), request_id.to_string()));
    headers
}

/// ~T 最终值编组：FutureValue.result → NanoValue → json_value_reply
/// （Failed / 缺 result → null；Phase A 语义同引擎恢复臂）。
fn final_value_reply(
    vm: &std::rc::Rc<AutoVM>,
    ctx: &DispatchCtx,
    value: Option<auto_val::Value>,
) -> ApiReply {
    let nv = value
        .and_then(|v| value_to_nv_via_task(vm, ctx, v))
        .unwrap_or_else(auto_val::encode_null);
    // PLAN-730 T-05：`~UploadReceipt` 终值（Value::Int = 收据句柄）在
    // JSON 兜底前按同一门识别（sync/异步同形）。
    if handler_declares_upload_return(vm, ctx) {
        if auto_val::is_i32(nv) {
            let rid = auto_val::decode_i32(nv) as u32 as u64;
            return match super::http_upload::take_upload_receipt(rid) {
                Some(receipt) => upload_receipt_reply(ctx, receipt),
                None => upload_contract_mismatch_reply(ctx, rid),
            };
        }
        return upload_contract_mismatch_reply(ctx, u64::MAX);
    }
    // PLAN-729 T-04：`~FileResponse` 终值（Value::Int = 描述符句柄）在
    // JSON 兜底前按同一门识别（sync/异步同形）。
    if handler_declares_file_return(vm, ctx) {
        if auto_val::is_i32(nv) {
            let id = auto_val::decode_i32(nv) as u64;
            return match super::http_server_file::take_file_response(id) {
                Some(descriptor) => ApiReply::Full {
                    status: 200,
                    headers: Vec::new(),
                    body: ApiBody::File(FileReplySeed {
                        descriptor,
                        request_id: ctx.request_id.clone(),
                    }),
                },
                None => file_contract_mismatch_reply(ctx, id),
            };
        }
        return file_contract_mismatch_reply(ctx, u64::MAX);
    }
    json_value_reply(
        vm,
        nv,
        &ctx.req_method,
        &ctx.req_path,
        &ctx.route.fn_name,
        &ctx.body,
        &ctx.request_id,
        ctx.started.elapsed().as_millis(),
    )
}

/// Value → NanoValue（经 handler 任务栈的 scratch——字符串走
/// rc_push_str_idx 保 RC 簿记；复杂变体降级 null，~T 典型返回为标量/串）。
fn value_to_nv_via_task(
    vm: &std::rc::Rc<AutoVM>,
    ctx: &DispatchCtx,
    value: auto_val::Value,
) -> Option<auto_val::NanoValue> {
    let task_id = ctx.handler_task_id?;
    let t_arc = vm.tasks.get(&task_id)?.clone();
    let mut t = t_arc.try_lock().ok()?;
    let nv = match value {
        auto_val::Value::Int(i) => auto_val::encode_i32(i),
        auto_val::Value::Float(f) => auto_val::encode_f64(f),
        auto_val::Value::Double(f) => auto_val::encode_f64(f),
        auto_val::Value::Bool(b) => auto_val::encode_bool(b),
        auto_val::Value::Nil => auto_val::encode_null(),
        auto_val::Value::Str(s) => {
            let idx = vm.add_string(s.to_string().into_bytes());
            vm.rc_push_str_idx(&mut t, idx);
            t.ram.pop_nv()
        }
        _ => auto_val::encode_null(),
    };
    Some(nv)
}

// ============================================================================
// PLAN-705 T-05: 请求作用域与生命期许可（queued+running+parked 总上限）
// ============================================================================
//
// 许可语义升级（决策报告 §3，兼容性明示）：AUTO_HTTP_MAX_INFLIGHT 从
// "队列容量"升级为"请求生命期总上限"——许可在桥入队前获取，scope 终结
// （回复送达 / SSE 流结束 / 取消 / 关闭）幂等释放。mpsc 队列容量仍为同值
// （队满 503 先于许可上限发生）。取消/超时的请求不再无效排队与复活结果
// 槽：cancel 是幂等终结入口，parked 等待废弃时同步回收 live-op。

/// 作用域状态机（AtomicU8 载荷）。
pub(crate) const SCOPE_QUEUED: u8 = 0;
pub(crate) const SCOPE_RUNNING: u8 = 1;
pub(crate) const SCOPE_PARKED: u8 = 2;
pub(crate) const SCOPE_DONE: u8 = 3;
pub(crate) const SCOPE_CANCELLED: u8 = 4;

pub(crate) struct RequestScope {
    pub id: u64,
    pub conn_id: u64,
    pub state: std::sync::atomic::AtomicU8,
    /// PLAN-730 T-05：deadline 升级 RwLock——上传期限切换需透过 Arc 写
    ///（extend_scope_deadline 只增不减；读方短暂陈旧方向安全）。
    pub deadline: std::sync::RwLock<std::time::Instant>,
    /// 生命期许可（scope 终结时释放；SSE 流由桥侧 FrameStream Drop 代持）。
    pub permit: std::sync::Mutex<Option<tokio::sync::OwnedSemaphorePermit>>,
    /// 取消信号（桥 reply 等待 select 臂）。
    pub cancel_notify: tokio::sync::Notify,
    /// PLAN-707 T-06（D-7）：请求资源组——handler 段内打开的上游流 id。
    /// finalize_scope（完成/取消/断连/关闭）逐一流收口（abort 生产者 +
    /// 释放队列），保证组内资源不越过请求生命期。
    pub resources: std::sync::Mutex<Vec<u64>>,
    /// PLAN-727 T-05：有类型的传输资源组——与流 id 分开登记，finalize 时
    /// 取消传输并出 VM 注册表（transfer id 不与 HTTPStream id 混淆清理）。
    pub transfer_resources: std::sync::Mutex<Vec<u64>>,
    /// PLAN-729 T-04：文件响应描述符组——finalize 时移除闲置描述符
    /// （handler 未返回的构造防注册表无界增长）。
    pub file_response_resources: std::sync::Mutex<Vec<u64>>,
    /// PLAN-730 T-02：上传资源组（注入能力/会话/收据句柄混用同一 id 空间
    /// 的三张表按 id 幂等收口——take 已消费的 no-op）。finalize 时取消在途
    /// 接收、清理 staged、释放未消费 body 能力。
    pub upload_resources: std::sync::Mutex<Vec<u64>>,
    /// PLAN-730 T-05：请求起点（bridge 入队时刻）——上传总期限
    /// （started_at + total_timeout）切换的锚点。
    pub started_at: std::time::Instant,
    /// PLAN-736 T-05/AC-05：请求观测事件（恰一次终态；finalize_scope 发射）。
    pub event: std::sync::Mutex<Option<crate::http_service_observability::PendingRequestEvent>>,
    /// PLAN-730 T-05：deadline 变更通道（上传 receive 启动时延展 30s→总
    /// 期限；桥 reply 等待循环 watch 重臂——修"select 保留旧捕获值"）。
    /// 只延展（extend）不缩短；与 `deadline` 字段同步写（读方短暂陈旧
    /// 方向安全：只会更早到期判定由 watch 修正）。
    pub deadline_tx: tokio::sync::watch::Sender<std::time::Instant>,
}

impl RequestScope {
    /// 终态判定（DONE/CANCELLED）。
    pub(crate) fn terminal(&self) -> bool {
        matches!(
            self.state.load(std::sync::atomic::Ordering::SeqCst),
            SCOPE_DONE | SCOPE_CANCELLED
        )
    }
}

lazy_static::lazy_static! {
    static ref REQUEST_SCOPES: std::sync::Mutex<std::collections::HashMap<u64, std::sync::Arc<RequestScope>>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
}

static SCOPE_ID_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

static LIFE_PERMITS: std::sync::OnceLock<std::sync::Arc<tokio::sync::Semaphore>> =
    std::sync::OnceLock::new();

static TEST_PERMIT_CAPACITY: std::sync::Mutex<Option<usize>> = std::sync::Mutex::new(None);

/// 仅供测试：覆写生命期许可容量（首次 life_permits() 前调用生效；
/// nextest 每测试进程隔离）。
#[cfg(test)]
pub(crate) fn set_life_permit_capacity_for_test(cap: usize) {
    *TEST_PERMIT_CAPACITY.lock().unwrap() = Some(cap);
}

fn life_permits() -> &'static std::sync::Arc<tokio::sync::Semaphore> {
    LIFE_PERMITS.get_or_init(|| {
        let cap = TEST_PERMIT_CAPACITY
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| super::http_transport::TransportConfig::from_env().queue_capacity);
        std::sync::Arc::new(tokio::sync::Semaphore::new(cap))
    })
}

/// 桥侧（net 线程）：入队前建 scope（许可随 scope 持有，Queued）。
/// 许可不可得（生命期总上限满）→ None（桥回 503，零分配零排队）。
pub(crate) fn create_scope(
    conn_id: u64,
    deadline: std::time::Instant,
) -> Option<std::sync::Arc<RequestScope>> {
    let Ok(permit) = life_permits().clone().try_acquire_owned() else {
        return None;
    };
    let id = SCOPE_ID_GEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let (deadline_tx, _deadline_rx) = tokio::sync::watch::channel(deadline);
    let deadline = std::sync::RwLock::new(deadline);
    let scope = std::sync::Arc::new(RequestScope {
        id,
        conn_id,
        state: std::sync::atomic::AtomicU8::new(SCOPE_QUEUED),
        deadline,
        permit: std::sync::Mutex::new(Some(permit)),
        cancel_notify: tokio::sync::Notify::new(),
        resources: std::sync::Mutex::new(Vec::new()),
        transfer_resources: std::sync::Mutex::new(Vec::new()),
        file_response_resources: std::sync::Mutex::new(Vec::new()),
        upload_resources: std::sync::Mutex::new(Vec::new()),
        started_at: std::time::Instant::now(),
        event: std::sync::Mutex::new(None),
        deadline_tx,
    });
    if let Ok(mut map) = REQUEST_SCOPES.lock() {
        map.insert(id, scope.clone());
    }
    Some(scope)
}

/// scope 状态迁移（幂等；终态不可逆）。
fn scope_transition(scope: &RequestScope, to: u8) {
    let current = scope.state.load(std::sync::atomic::Ordering::SeqCst);
    if current < SCOPE_DONE {
        scope.state.store(to, std::sync::atomic::Ordering::SeqCst);
    }
}

/// scope 幂等终结（完成/取消共用）：释放许可 + 移除登记 + 发取消信号
/// （等它的桥 select 臂与 owner 唤醒臂消费）。
fn finalize_scope(scope: &RequestScope, to: u8) {
    scope_transition(scope, to);
    // PLAN-707 T-06（D-7）：组收口先行——请求内打开的全部上游流终结
    //（abort 生产者 + 出表 + 通知等待者）。「等待可取消 ≠ 关闭登记」：
    // 这里是关闭登记面；生产者实际停止由 stream_cancel 的 abort 承载。
    let group: Vec<u64> = scope
        .resources
        .lock()
        .map(|mut r| std::mem::take(&mut *r))
        .unwrap_or_default();
    for stream_id in group {
        crate::vm::ffi::http_stream::stream_cancel(stream_id);
    }
    // PLAN-727 T-05：传输组收口——取消 + 出 VM 注册表（scope deadline/
    // 断连/shutdown 级联取消的传输臂）。
    let transfers: Vec<u64> = scope
        .transfer_resources
        .lock()
        .map(|mut r| std::mem::take(&mut *r))
        .unwrap_or_default();
    crate::vm::ffi::http_transfer::scope_finalize_transfers(&transfers);
    // PLAN-729 T-04：文件描述符组收口（幂等；编组已取出的 no-op）。
    let descriptors: Vec<u64> = scope
        .file_response_resources
        .lock()
        .map(|mut r| std::mem::take(&mut *r))
        .unwrap_or_default();
    crate::vm::ffi::http_server_file::scope_finalize_file_responses(&descriptors);
    // PLAN-730 T-02：上传资源组收口（幂等；取消在途接收 + 清理 staged +
    /// 释放未消费 body 能力 + 移除闲置收据）。
    let uploads: Vec<u64> = scope
        .upload_resources
        .lock()
        .map(|mut r| std::mem::take(&mut *r))
        .unwrap_or_default();
    crate::vm::ffi::http_upload::scope_finalize_uploads(&uploads);
    scope.permit.lock().unwrap().take(); // 释放生命期许可（幂等）
    if let Ok(mut map) = REQUEST_SCOPES.lock() {
        map.remove(&scope.id);
    }
    scope.cancel_notify.notify_waiters();
    crate::vm::ffi::async_http::COMPLETION_NOTIFY.notify_waiters();
    // PLAN-736 T-05/AC-05：请求终态事件恰一次（完成/取消单点收敛；
    // 重复 cancel/drop 双路径由事件自身的原子闸防双计）。
    if let Ok(mut slot) = scope.event.lock() {
        if let Some(ev) = slot.take() {
            if to == SCOPE_CANCELLED {
                ev.cancel();
            } else {
                ev.complete();
            }
        }
    }
}

/// 正常完成（回复送达非 SSE / SSE 流结束）。
pub(crate) fn complete_scope(scope: &RequestScope) {
    finalize_scope(scope, SCOPE_DONE);
}

/// 取消（桥超时/连接终结/关闭）：幂等；首次迁移返回 true。
pub(crate) fn cancel_scope(scope: &RequestScope) -> bool {
    if scope.terminal() {
        return false;
    }
    finalize_scope(scope, SCOPE_CANCELLED);
    true
}

/// scope 是否仍可执行（owner 派发/恢复前的失效检查）：
/// 已取消 / 已终结 / 已过 deadline → false（失效请求不调用业务函数）。
pub(crate) fn scope_usable(scope: &RequestScope) -> bool {
    if scope.terminal() {
        return false;
    }
    std::time::Instant::now() <= *scope.deadline.read().unwrap()
}

/// 按 id 查 scope（owner 侧）。
pub(crate) fn lookup_scope(id: u64) -> Option<std::sync::Arc<RequestScope>> {
    REQUEST_SCOPES.lock().ok().and_then(|m| m.get(&id).cloned())
}

/// PLAN-730 T-05：延展 scope deadline（只增不减；上传 receive 启动时从
/// 30s 切换到上传总期限）。watch 唤醒桥 reply 等待循环重臂；完成通知唤醒
/// owner parked 定时臂重算。
pub(crate) fn extend_scope_deadline(scope_id: u64, new_deadline: std::time::Instant) {
    if let Some(scope) = lookup_scope(scope_id) {
        let extended = {
            let mut d = scope.deadline.write().unwrap();
            if new_deadline > *d {
                *d = new_deadline;
                true
            } else {
                false
            }
        };
        if extended {
            let _ = scope.deadline_tx.send(new_deadline);
            crate::vm::ffi::async_http::COMPLETION_NOTIFY.notify_waiters();
        }
    }
}

/// PLAN-730 T-05：当前段执行的 scope id（无绑定为 None——非请求上下文）。
pub(crate) fn current_scope_id() -> Option<u64> {
    CURRENT_SCOPE_ID.with(|c| c.get())
}

/// 连接终结（net 侧 watcher）：取消该连接名下全部 scope（已确证销毁判据，
/// 决策报告 §5-3）。
pub(crate) fn cancel_scopes_for_conn(conn_id: u64) {
    let scopes: Vec<std::sync::Arc<RequestScope>> = match REQUEST_SCOPES.lock() {
        Ok(map) => map
            .values()
            .filter(|s| s.conn_id == conn_id)
            .cloned()
            .collect(),
        Err(_) => return,
    };
    for s in scopes {
        cancel_scope(&s);
    }
}

// PLAN-707 T-06（D-7）：handler 段执行的当前 scope——流 open shim 经
// [`register_scope_stream`] 把新建流挂进请求资源组；非 request 上下文
// （UI/CLI 程序）无 scope，资源归显式 close 管理（决策 §5.4）。
thread_local! {
    static CURRENT_SCOPE_ID: std::cell::Cell<Option<u64>> =
        const { std::cell::Cell::new(None) };
}

/// 段执行期间绑定 scope 的 RAII 守卫（嵌套安全：保存/恢复外层值）。
pub(crate) struct ScopeGuard;

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        CURRENT_SCOPE_ID.with(|c| c.set(None));
    }
}

/// 进入 handler 段（dispatch/resume）时调用；Drop 恢复无 scope 态。
pub(crate) fn bind_current_scope(scope_id: u64) -> ScopeGuard {
    CURRENT_SCOPE_ID.with(|c| c.set(Some(scope_id)));
    ScopeGuard
}

/// 流 open shim 调用：把新流登记进当前请求资源组（无 scope → no-op）。
pub(crate) fn register_scope_stream(stream_id: u64) {
    let Some(scope_id) = CURRENT_SCOPE_ID.with(|c| c.get()) else {
        return;
    };
    if let Some(scope) = lookup_scope(scope_id) {
        if let Ok(mut r) = scope.resources.lock() {
            r.push(stream_id);
        }
    }
}

/// PLAN-727 T-05：transfer 提交 shim 调用——传输登记进当前请求的**有类型**
/// 传输资源组（无 scope → no-op；CLI/UI 程序资源归显式 cancel/wait 管理）。
pub(crate) fn register_scope_transfer(transfer_id: u64) {
    let Some(scope_id) = CURRENT_SCOPE_ID.with(|c| c.get()) else {
        return;
    };
    if let Some(scope) = lookup_scope(scope_id) {
        if let Ok(mut r) = scope.transfer_resources.lock() {
            r.push(transfer_id);
        }
    }
}

/// PLAN-729 T-04：file_response 构造 shim 调用——描述符登记进当前请求的
/// 文件描述符资源组（无 scope → no-op；编组取出或 scope 终结时移除）。
pub(crate) fn register_scope_file_response(descriptor_id: u64) {
    let Some(scope_id) = CURRENT_SCOPE_ID.with(|c| c.get()) else {
        return;
    };
    if let Some(scope) = lookup_scope(scope_id) {
        if let Ok(mut r) = scope.file_response_resources.lock() {
            r.push(descriptor_id);
        }
    }
}

/// PLAN-730 T-02：上传句柄（注入能力/会话/收据）登记进当前请求的上传
/// 资源组（无 scope → no-op；非请求上下文构造不能进入 HTTP 编组面）。
pub(crate) fn register_scope_upload(upload_id: u64) {
    let Some(scope_id) = CURRENT_SCOPE_ID.with(|c| c.get()) else {
        return;
    };
    if let Some(scope) = lookup_scope(scope_id) {
        if let Ok(mut r) = scope.upload_resources.lock() {
            r.push(upload_id);
        }
    }
}

/// 资源基线探针（测试）：存活 scope 数。
#[cfg(test)]
pub(crate) fn live_scope_count() -> usize {
    REQUEST_SCOPES.lock().map(|m| m.len()).unwrap_or(0)
}

/// 资源基线探针（测试）：可用生命期许可数。
#[cfg(test)]
pub(crate) fn life_permit_available() -> usize {
    life_permits().available_permits()
}

/// 就绪探测（parked 表扫描用；不消费任何状态）。
pub(crate) fn parked_is_ready(vm: &std::rc::Rc<AutoVM>, p: &ParkedRequest) -> bool {
    match &p.wait {
        ParkedWait::HttpRequest(req_id) => crate::vm::ffi::stdlib::async_http_result_ready(*req_id),
        ParkedWait::Future(fid) => vm
            .futures
            .get(fid)
            .map(|f| f.read().unwrap().state != crate::vm::engine::FutureState::Pending)
            .unwrap_or(true), // future 消失 → 唤醒（引擎恢复臂同款 nil fallback）
        // PLAN-707 T-05: 外部流——数据入队或终态即就绪（stream_ready 含
        // 条目消失的终结臂）。
        ParkedWait::HttpStream(stream_id) => crate::vm::ffi::http_stream::stream_ready(*stream_id),
        // PLAN-711 T-11: CPU continuation 归 vm_bridge 的 CPU 泵，HTTP 泵
        // 不拾取（防御臂；HTTP 轨不产该凭据）。
        ParkedWait::CpuRunnable => false,
    }
}

/// 恢复 parked 请求到下一停点或终态。
pub(crate) enum ParkedResume {
    /// 终态回复（任务已清理；caller 经条目内 reply_tx 发送并移除条目）。
    Reply(ApiReply),
    /// 仍未完成（wait/seg 凭据已刷新）。
    StillParked,
    /// 本条目作废，改挂交接条目（middleware 续链重新 park 的形态）。
    Handoff(Box<ParkedRequest>),
    /// PLAN-730 T-07：回复已由下游自送（start_handler 内部消费 reply_tx；
    /// 条目直接移除，caller 不得再送）。
    Consumed,
}

pub(crate) fn resume_parked_request(
    vm: &std::rc::Rc<AutoVM>,
    p: &mut ParkedRequest,
) -> ParkedResume {
    // PLAN-707 T-06: 恢复段与首段同属请求资源组（流 open 同样登记）。
    let _scope_bind = bind_current_scope(p.scope_id);
    match p.stage {
        ParkStage::Middleware { index } => {
            enum MWResume {
                Parked(ParkedWait, ParkedSegment),
                ContinueChain,
                ShortCircuit(ApiReply),
                Missing,
            }
            let task_id = p.task_id;
            let seg = p.seg.clone();
            let step = if let Some(t_arc) = vm.tasks.get(&task_id) {
                let Ok(mut t) = t_arc.try_lock() else {
                    return ParkedResume::StillParked; // 锁忙 → 下一轮重试
                };
                match vm.resume_fn_by_name_segment(&mut t, &seg) {
                    SegmentOutcome::Parked { wait, seg } => MWResume::Parked(wait, seg),
                    SegmentOutcome::Completed(Ok(())) => {
                        let nv = t.ram.pop_nv();
                        if auto_val::is_null(nv) {
                            MWResume::ContinueChain
                        } else {
                            match nv_to_json(vm, nv, 0) {
                                Some(resp) if !resp.is_empty() && resp != "null" => {
                                    let mut headers = vec![(
                                        "Content-Type".to_string(),
                                        "application/json".to_string(),
                                    )];
                                    headers.extend(cors_header_pairs());
                                    headers.push((
                                        "X-Request-Id".to_string(),
                                        p.ctx.request_id.clone(),
                                    ));
                                    MWResume::ShortCircuit(ApiReply::Full {
                                        status: 200,
                                        headers,
                                        body: ApiBody::Text(resp.into_bytes()),
                                    })
                                }
                                _ => MWResume::ContinueChain,
                            }
                        }
                    }
                    // legacy：middleware Err → 无响应继续链。
                    SegmentOutcome::Completed(Err(_)) => MWResume::ContinueChain,
                    // PLAN-711 T-11: HTTP 轨不产 CPU continuation（防御臂：
                    // 未完成=仍 park，凭据 CpuRunnable 对 HTTP 泵恒不就绪）。
                    SegmentOutcome::Runnable { seg } => {
                        MWResume::Parked(ParkedWait::CpuRunnable, seg)
                    }
                }
            } else {
                MWResume::Missing
            };
            match step {
                MWResume::Missing => ParkedResume::Reply(ApiReply::Full {
                    status: 200,
                    headers: json_reply_headers(&p.ctx.request_id),
                    body: ApiBody::Text(Vec::new()),
                }),
                MWResume::Parked(wait, seg) => {
                    p.wait = wait;
                    p.seg = seg;
                    ParkedResume::StillParked
                }
                MWResume::ShortCircuit(reply) => {
                    vm.tasks.remove(&task_id);
                    ParkedResume::Reply(reply)
                }
                MWResume::ContinueChain => {
                    vm.tasks.remove(&task_id);
                    // 续链：ctx 移交进 advance_dispatch；其再次 park 时产出
                    // 新条目（Handoff）——本条目由 caller 移除。
                    let ctx = std::mem::replace(&mut p.ctx, placeholder_ctx());
                    match advance_dispatch(vm, ctx, index + 1, p.reply_tx.take()) {
                        DispatchOutcome::Replied => ParkedResume::Reply(placeholder_reply()),
                        DispatchOutcome::Parked(np) => ParkedResume::Handoff(np),
                    }
                }
            }
        }
        // PLAN-730 T-07：legacy 落盘完成——合并 files JSON 进 body 后进
        // handler；写失败 = 真实 500（不返回不存在的假路径）。
        ParkStage::LegacyStore { op_id } => {
            let result = LEGACY_STORE_RESULTS
                .lock()
                .ok()
                .and_then(|mut m| m.remove(&op_id))
                .flatten();
            // live-op marker 出表（complete 侧落 Body("")；take 走防泄漏）。
            let _ = crate::vm::ffi::stdlib::check_async_http_result(op_id);
            match result {
                Some(Ok((paths, files_json))) => {
                    p.ctx.legacy_stored = paths;
                    p.ctx.body = multipart_handler_json(&p.ctx.legacy_fields_json, &files_json);
                    p.ctx.multipart_json = Some(p.ctx.body.clone());
                    advance_after_legacy_store(vm, p)
                }
                Some(Err(message)) => {
                    eprintln!(
                        "[HTTP] {} {} [{}] → 500 (legacy multipart store: {})",
                        p.ctx.req_method, p.ctx.req_path, p.ctx.request_id, message
                    );
                    let mut headers = cors_json_headers(&p.ctx.request_id);
                    headers[0].1 = "application/json".to_string();
                    ParkedResume::Reply(ApiReply::Full {
                        status: 500,
                        headers,
                        body: ApiBody::Text(
                            format!("{{\"error\":{}}}", json_escape_string(&message))
                                .into_bytes(),
                        ),
                    })
                }
                None => ParkedResume::StillParked,
            }
        }
        ParkStage::Handler => {
            enum HResume {
                Parked(ParkedWait, ParkedSegment),
                Value(auto_val::NanoValue),
                Err(crate::vm::engine::VMError),
                Missing,
            }
            let task_id = p.task_id;
            let seg = p.seg.clone();
            let step = if let Some(t_arc) = vm.tasks.get(&task_id) {
                let Ok(mut t) = t_arc.try_lock() else {
                    return ParkedResume::StillParked;
                };
                match vm.resume_fn_by_name_segment(&mut t, &seg) {
                    SegmentOutcome::Parked { wait, seg } => HResume::Parked(wait, seg),
                    SegmentOutcome::Completed(Ok(())) => HResume::Value(t.ram.pop_nv()),
                    SegmentOutcome::Completed(Err(e)) => HResume::Err(e),
                    // PLAN-711 T-11: HTTP 轨不产 CPU continuation（防御臂）。
                    SegmentOutcome::Runnable { .. } => {
                        HResume::Err(crate::vm::engine::VMError::RuntimeError(
                            "internal: cpu-slice outcome escaped legacy http resume (unreachable)"
                                .into(),
                        ))
                    }
                }
            } else {
                HResume::Missing
            };
            match step {
                HResume::Missing => ParkedResume::Reply(ApiReply::Full {
                    status: 200,
                    headers: json_reply_headers(&p.ctx.request_id),
                    body: ApiBody::Text(Vec::new()),
                }),
                HResume::Parked(wait, seg) => {
                    p.wait = wait;
                    p.seg = seg;
                    ParkedResume::StillParked
                }
                HResume::Err(e) => {
                    vm.tasks.remove(&task_id);
                    ParkedResume::Reply(handler_error_reply(&p.ctx, &e))
                }
                HResume::Value(nv) => match marshal_handler_value(vm, &p.ctx, nv) {
                    MarshalOutcome::Reply(reply) => {
                        vm.tasks.remove(&task_id);
                        ParkedResume::Reply(reply)
                    }
                    MarshalOutcome::ParkFuture(fid) => {
                        p.wait = ParkedWait::Future(fid);
                        p.stage = ParkStage::AwaitReturnFuture;
                        ParkedResume::StillParked
                    }
                    MarshalOutcome::ParkBody(internal_fid, outer_fid) => {
                        p.wait = ParkedWait::Future(outer_fid);
                        p.stage = ParkStage::AsyncReturnBody { internal_fid };
                        ParkedResume::StillParked
                    }
                },
            }
        }
        ParkStage::AsyncReturnBody { internal_fid } => {
            // 外层 await 已完成：恢复臂推送其结果并续跑挂起的 `~{}` 体
            //（resume_suspended_body 完成后 internal future 置 Ready），
            // 然后按内部 future 终态编组。
            let seg = p.seg.clone();
            let task_id = p.task_id;
            let step = if let Some(t_arc) = vm.tasks.get(&task_id) {
                match t_arc.try_lock() {
                    Ok(mut t) => Some(vm.resume_fn_by_name_segment(&mut t, &seg)),
                    Err(_) => return ParkedResume::StillParked,
                }
            } else {
                None
            };
            match step {
                None => {
                    let reply = final_value_reply(vm, &p.ctx, None);
                    vm.tasks.remove(&task_id);
                    return ParkedResume::Reply(reply);
                }
                Some(SegmentOutcome::Completed(Err(_))) => {
                    let reply = final_value_reply(vm, &p.ctx, None);
                    vm.tasks.remove(&task_id);
                    return ParkedResume::Reply(reply);
                }
                Some(SegmentOutcome::Parked { wait, seg }) => {
                    p.wait = wait;
                    p.seg = seg;
                    return ParkedResume::StillParked;
                }
                // PLAN-711 T-11: HTTP 轨不产 CPU continuation（防御臂：视为
                // 未完成继续 park，凭据 CpuRunnable 对 HTTP 泵恒不就绪）。
                Some(SegmentOutcome::Runnable { .. }) => {
                    p.wait = ParkedWait::CpuRunnable;
                    return ParkedResume::StillParked;
                }
                Some(SegmentOutcome::Completed(Ok(()))) => {}
            }
            return probe_return_future(vm, p, internal_fid);
        }
        ParkStage::AwaitReturnFuture => {
            let fid = match &p.wait {
                ParkedWait::Future(fid) => *fid,
                _ => return ParkedResume::StillParked,
            };
            let Some(fut) = vm.futures.get(&fid) else {
                // future 消失 → nil fallback（引擎恢复臂同款）。终值编组需要
                // handler 任务栈做 scratch——先编组后清理。
                let reply = final_value_reply(vm, &p.ctx, None);
                vm.tasks.remove(&p.task_id);
                return ParkedResume::Reply(reply);
            };
            if fut.read().unwrap().state == crate::vm::engine::FutureState::Pending {
                return ParkedResume::StillParked;
            }
            let value = fut.read().unwrap().result.clone();
            let reply = final_value_reply(vm, &p.ctx, value);
            vm.tasks.remove(&p.task_id);
            ParkedResume::Reply(reply)
        }
    }
}

fn placeholder_ctx() -> DispatchCtx {
    DispatchCtx {
        req_method: String::new(),
        req_path: String::new(),
        request_id: String::new(),
        body: String::new(),
        content_type: String::new(),
        content_type_raw: String::new(),
        cookie_header: String::new(),
        auth_header: String::new(),
        multipart_json: None,
        route: RouteMatch {
            fn_name: String::new(),
            path_params: Vec::new(),
            query_params: Vec::new(),
        },
        axum_route: None,
        started: std::time::Instant::now(),
        middleware_names: Vec::new(),
        request_info: String::new(),
        handler_task_id: None,
        scope_id: 0,
        upload_pending: None,
        legacy_parts: None,
        legacy_fields_json: String::new(),
        legacy_stored: Vec::new(),
    }
}

fn placeholder_reply() -> ApiReply {
    ApiReply::Full {
        status: 200,
        headers: Vec::new(),
        body: ApiBody::Text(Vec::new()),
    }
}

/// 按返回 future 终态编组（Ready → 终值；Failed/缺失 → null；Pending →
/// 继续等）。
fn probe_return_future(vm: &std::rc::Rc<AutoVM>, p: &mut ParkedRequest, fid: u32) -> ParkedResume {
    let Some(fut) = vm.futures.get(&fid) else {
        let reply = final_value_reply(vm, &p.ctx, None);
        vm.tasks.remove(&p.task_id);
        return ParkedResume::Reply(reply);
    };
    if fut.read().unwrap().state == crate::vm::engine::FutureState::Pending {
        p.wait = ParkedWait::Future(fid);
        p.stage = ParkStage::AwaitReturnFuture;
        return ParkedResume::StillParked;
    }
    let value = fut.read().unwrap().result.clone();
    let reply = final_value_reply(vm, &p.ctx, value);
    vm.tasks.remove(&p.task_id);
    ParkedResume::Reply(reply)
}

/// 事件驱动的就绪扫描：按 park 顺序恢复全部就绪条目（公平、零轮询——
/// 扫描只发生在完成通知/事件唤醒后）。
pub(crate) fn drain_ready_parked(vm: &std::rc::Rc<AutoVM>, parked: &mut Vec<ParkedRequest>) {
    let mut i = 0;
    while i < parked.len() {
        // T-05: scope 失效（取消 / deadline 过期）优先于就绪检查——失效
        // 请求废弃：live-op 回收（迟到完成被 presence 守卫丢弃）、任务
        // 清理、503 终态。业务函数不再被无效驱动（AC-04）。
        let scope_dead = lookup_scope(parked[i].scope_id)
            .map(|s| !scope_usable(&s))
            .unwrap_or(true);
        if scope_dead {
            let mut p = parked.remove(i);
            abort_parked_request(vm, &mut p);
            continue;
        }
        if parked_is_ready(vm, &parked[i]) {
            let mut p = parked.remove(i);
            match resume_parked_request(vm, &mut p) {
                ParkedResume::Reply(reply) => {
                    if let Some(tx) = p.reply_tx.take() {
                        let _ = tx.send(reply);
                    }
                }
                ParkedResume::StillParked => {
                    parked.insert(i, p);
                    i += 1;
                }
                ParkedResume::Handoff(np) => {
                    parked.insert(i, *np);
                    i += 1;
                }
                ParkedResume::Consumed => {} // 回复已自送——条目移除即可
            }
        } else {
            i += 1;
        }
    }
}

/// 废弃一条失效 parked 请求：live-op 回收（HttpRequest 等待）、任务清理、
/// 503 终态（reply_rx 已在桥侧超时关闭时发送静默失败——幂等）。
fn abort_parked_request(vm: &std::rc::Rc<AutoVM>, p: &mut ParkedRequest) {
    if let ParkedWait::HttpRequest(req_id) = &p.wait {
        crate::vm::ffi::stdlib::drop_async_result(*req_id);
    }
    // PLAN-730 T-07：废弃请求的 legacy provisional 文件删除（取消/超时）；
    // LegacyStore 期的迟到落盘结果由 presence 守卫丢弃 + 结果表条目清除。
    cleanup_legacy_files(&p.ctx.legacy_stored);
    // PLAN-707 T-06（D-7）：废弃请求的任务持有的流一并取消（首段 open
    // 后 park 的形态）。
    if let Some(t) = vm.tasks.get(&p.task_id) {
        let owned = t
            .try_lock()
            .ok()
            .map(|mut t| std::mem::take(&mut t.owned_stream_ids));
        if let Some(ids) = owned {
            for stream_id in ids {
                crate::vm::ffi::http_stream::stream_cancel(stream_id);
            }
        }
    }
    vm.tasks.remove(&p.task_id);
    if let Some(tx) = p.reply_tx.take() {
        let _ = tx.send(ApiReply::Full {
            status: 503,
            headers: json_reply_headers(&p.ctx.request_id),
            body: ApiBody::Text(br#"{"error":"request cancelled"}"#.to_vec()),
        });
    }
}

/// Marshal a returned NanoValue into the final JSON reply arm: PLAN-698 POST
/// broadcast + 200/500 mapping by the legacy `{"error":` prefix convention +
/// the success log line.
fn json_value_reply(
    vm: &crate::vm::engine::AutoVM,
    nv: auto_val::NanoValue,
    req_method: &str,
    req_path: &str,
    fn_name: &str,
    request_body: &str,
    request_id: &str,
    elapsed_ms: u128,
) -> ApiReply {
    match nv_to_json(vm, nv, 0) {
        Some(result_json) => {
            // PLAN-734 T-03（D3）：`{"error":` 前缀猜测退役——业务 record
            // 含 error 字段是数据（200）；错误状态由执行/序列化失败决定。
            let status = 200;
            // PLAN-698 SD-02: POST 广播臂（Typing / New{Type}，has_sse 门控）。
            if req_method == "POST" {
                publish_post_broadcast(fn_name, request_body, &result_json);
            }
            eprintln!(
                "[HTTP] {} {} [{}] → 200 ({}ms)",
                req_method, req_path, request_id, elapsed_ms
            );
            ApiReply::Full {
                status,
                headers: json_reply_headers(request_id),
                body: ApiBody::Text(result_json.into_bytes()),
            }
        }
        // PLAN-734 T-03（D3）：序列化失败 → 500 诊断（不再 200 空 body
        // 冒充成功；AC-03）。
        None => ApiReply::Full {
            status: 500,
            headers: json_reply_headers(request_id),
            body: ApiBody::Text(br#"{"error":"response serialization failed"}"#.to_vec()),
        },
    }
}

/// Standard headers of a JSON `#[api]` reply (content-type + CORS + request id).
fn json_reply_headers(request_id: &str) -> Vec<(String, String)> {
    let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
    headers.extend(cors_header_pairs());
    headers.push(("X-Request-Id".to_string(), request_id.to_string()));
    headers
}

struct SseIteratorCleanup {
    vm: std::rc::Rc<crate::vm::engine::AutoVM>,
    iterator_id: u32,
}

impl Drop for SseIteratorCleanup {
    fn drop(&mut self) {
        cleanup_sse_iterator(&self.vm, self.iterator_id);
    }
}

fn cleanup_sse_iterator(vm: &crate::vm::engine::AutoVM, iterator_id: u32) {
    // PLAN-698 F-1（review #1）: AsyncHttpStream 句柄一并回收——rx drop
    // 使转发线程的 `blocking_send` 失败退出（bus.subscribe 的转发线程
    // 否则在订阅端断连后永生：broadcast 静态永活，无自然终点）。既有
    // http/io 流线程本就自然终止，此处回收同时缩小其句柄累积面。
    // Generator 仍回收其 VM task（原语义零变化）。
    match vm.iterators.remove(&iterator_id) {
        Some((_, crate::vm::engine::Iterator::Generator(state))) => {
            if let Some(task_id) = state.task_id {
                // PLAN-707 T-06（D-7）：generator 体内建立的上游流随任务
                // 回收（首次 pull 发生在 SSE serve 循环，scope 守卫已退出，
                // 资源组登记靠 task.owned_stream_ids 第二线承载）。
                if let Some(t) = vm.tasks.get(&task_id) {
                    let owned = t
                        .try_lock()
                        .ok()
                        .map(|mut t| std::mem::take(&mut t.owned_stream_ids));
                    if let Some(ids) = owned {
                        for stream_id in ids {
                            crate::vm::ffi::http_stream::stream_cancel(stream_id);
                        }
                    }
                }
                vm.tasks.remove(&task_id);
            }
        }
        Some((_, crate::vm::engine::Iterator::AsyncHttpStream(a))) => {
            if let Ok(mut map) = crate::vm::ffi::stdlib::ASYNC_STREAMS.lock() {
                map.remove(&a.stream_id);
            }
            // PLAN-707 T-06：流句柄出表 → 泵 tx.send 失败 → stream_cancel
            //（generator 内建立的上游连接随 SSE 输出终结一并收口）。
        }
        Some((_, crate::vm::engine::Iterator::HttpStream(h))) => {
            // PLAN-707 T-06：raw 流迭代器清理=真取消（连接关闭）。
            crate::vm::ffi::http_stream::stream_cancel(h.stream_handle);
        }
        Some((_, _)) | None => {}
    }
}

fn sse_generator_wake_deadline(
    vm: &crate::vm::engine::AutoVM,
    iterator_id: u32,
) -> Option<std::time::Instant> {
    let task_id = match vm.iterators.get(&iterator_id) {
        Some(state) => match &*state {
            crate::vm::engine::Iterator::Generator(generator) => generator.task_id?,
            _ => return None,
        },
        None => return None,
    };
    vm.tasks
        .get(&task_id)
        .and_then(|task| task.try_lock().ok().and_then(|task| task.wake_time))
}

fn wake_sse_generator(vm: &crate::vm::engine::AutoVM, iterator_id: u32) {
    let task_id = match vm.iterators.get(&iterator_id) {
        Some(state) => match &*state {
            crate::vm::engine::Iterator::Generator(generator) => generator.task_id,
            _ => None,
        },
        None => None,
    };
    if let Some(task_id) = task_id {
        if let Some(task) = vm.tasks.get(&task_id) {
            if let Ok(mut task) = task.try_lock() {
                if task
                    .wake_time
                    .is_some_and(|deadline| deadline <= std::time::Instant::now())
                {
                    task.wake_time = None;
                    task.status = crate::vm::task::TaskStatus::Ready;
                }
            }
        }
    }
}

/// PLAN-707 T-05（D-6）：读取 generator 任务的等待凭据（cooperative sleep
/// 之外的凭据化等待源）——外部流 id 或外部 future id。
fn generator_wait_credential(vm: &crate::vm::engine::AutoVM, iterator_id: u32) -> GeneratorWait {
    use crate::vm::engine::Iterator;
    let task_id = match vm.iterators.get(&iterator_id) {
        Some(state) => match &*state {
            Iterator::Generator(generator) => generator.task_id,
            _ => None,
        },
        None => None,
    };
    let Some(task_id) = task_id else {
        return GeneratorWait::None;
    };
    let Some(task) = vm.tasks.get(&task_id) else {
        return GeneratorWait::None;
    };
    let Ok(task) = task.try_lock() else {
        return GeneratorWait::None;
    };
    if let Some(stream_id) = task.waiting_http_stream_id {
        return GeneratorWait::Stream(stream_id);
    }
    if let Some(stream_id) = task.waiting_sse_stream_id {
        return GeneratorWait::Stream(stream_id);
    }
    if let Some(fid) = task.waiting_future_id {
        return GeneratorWait::Future(fid);
    }
    GeneratorWait::None
}

enum GeneratorWait {
    None,
    Stream(u64),
    Future(u32),
}

/// 测试访问器：next_sse_generator_value（私有 async）驱动。
#[cfg(test)]
pub(crate) async fn next_sse_generator_value_for_test(
    vm: &std::rc::Rc<crate::vm::engine::AutoVM>,
    iterator_id: u32,
) -> Option<u64> {
    next_sse_generator_value(vm, iterator_id).await
}

/// 测试访问器：generator 产出值 == 期望字符串（str 堆索引解码）。
#[cfg(test)]
pub(crate) fn sse_value_is(
    vm: &crate::vm::engine::AutoVM,
    nv: auto_val::NanoValue,
    expect: &str,
) -> bool {
    if auto_val::is_string(nv) {
        vm.get_string(auto_val::decode_string(nv) as u32)
            .map(|b| String::from_utf8_lossy(&b) == expect)
            .unwrap_or(false)
    } else {
        false
    }
}

async fn next_sse_generator_value(
    vm: &std::rc::Rc<crate::vm::engine::AutoVM>,
    iterator_id: u32,
) -> Option<u64> {
    loop {
        if let Some(deadline) = sse_generator_wake_deadline(vm, iterator_id) {
            if deadline > std::time::Instant::now() {
                tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
            }
            wake_sse_generator(vm, iterator_id);
        }
        let next_task_id = vm.spawn_task(0, 1024);
        let step_result = {
            if let Some(next_task) = vm.tasks.get(&next_task_id) {
                match next_task.try_lock() {
                    Ok(mut next_task) => {
                        next_task.ram.push_i32(iterator_id as i32);
                        match crate::vm::native::shim_iterator_next_cooperative(
                            &mut next_task,
                            vm,
                            4096,
                        ) {
                            Ok(true) => Ok(Some(next_task.ram.pop_nv())),
                            Ok(false) => Ok(None),
                            Err(error) => Err(error),
                        }
                    }
                    Err(_) => Err(crate::vm::engine::VMError::RuntimeError(
                        "SSE iterator task is busy".to_string(),
                    )),
                }
            } else {
                Err(crate::vm::engine::VMError::RuntimeError(
                    "SSE iterator task is missing".to_string(),
                ))
            }
        };
        vm.tasks.remove(&next_task_id);

        match step_result {
            Ok(Some(value)) => {
                if auto_val::is_i32(value) && auto_val::decode_i32(value) == -1 {
                    return None;
                }
                return Some(value);
            }
            Ok(None) => {
                // PLAN-707 T-05（D-6）：凭据化等待——外部流（ready Notify +
                // COMPLETION_NOTIFY 三段式）或外部 future（完成通知），
                // 零固定间隔轮询/自旋；cooperative sleep 走既有 wake_time
                // 臂；未知 pending 兜底 yield_now（保守不忙等死锁）。
                match generator_wait_credential(vm, iterator_id) {
                    GeneratorWait::Stream(stream_id) => {
                        crate::vm::ffi::http_stream::wait_stream_ready(stream_id).await;
                    }
                    GeneratorWait::Future(_) => {
                        let notified = crate::vm::ffi::async_http::COMPLETION_NOTIFY.notified();
                        tokio::pin!(notified);
                        notified.as_mut().enable();
                        // enable 后复查：凭据已清除（先完成再 park）则直接
                        // 回拉，否则等完成通知（complete_external_future 发出）。
                        if matches!(
                            generator_wait_credential(vm, iterator_id),
                            GeneratorWait::Future(_)
                        ) {
                            notified.await;
                        }
                    }
                    GeneratorWait::None => {
                        if let Some(deadline) = sse_generator_wake_deadline(vm, iterator_id) {
                            if deadline > std::time::Instant::now() {
                                tokio::time::sleep_until(tokio::time::Instant::from_std(deadline))
                                    .await;
                            }
                            wake_sse_generator(vm, iterator_id);
                        } else {
                            tokio::task::yield_now().await;
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("[HTTP] SSE generator step failed: {:?}", error);
                return None;
            }
        }
    }
}

async fn produce_sse_frames(
    vm: std::rc::Rc<crate::vm::engine::AutoVM>,
    iterator_id: u32,
    sender: tokio::sync::mpsc::Sender<String>,
) {
    let _cleanup = SseIteratorCleanup {
        vm: vm.clone(),
        iterator_id,
    };
    let mut heartbeat = tokio::time::interval_at(
        tokio::time::Instant::now() + std::time::Duration::from_secs(1),
        std::time::Duration::from_secs(1),
    );

    loop {
        tokio::select! {
            _ = sender.closed() => break,
            _ = heartbeat.tick() => {
                if sender.send(": keep-alive\n\n".to_string()).await.is_err() {
                    break;
                }
            }
            value = next_sse_generator_value(&vm, iterator_id) => {
                let Some(value) = value else { break; };
                if let Some(frame) = crate::vm::ffi::musk_response_ctor::sse_frame_from_nv(&vm, value) {
                    if sender.send(frame).await.is_err() {
                        break;
                    }
                }
            }
        }
        tokio::task::yield_now().await;
    }
}

pub(crate) fn encode_ws_text_frame(text: &str) -> Vec<u8> {
    let payload = text.as_bytes();
    let len = payload.len();
    let mut frame = Vec::new();
    // FIN + text opcode (0x81).
    frame.push(0x81);
    // Payload length (server→client frames are NOT masked).
    if len <= 125 {
        frame.push(len as u8);
    } else if len <= 65535 {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }
    frame.extend_from_slice(payload);
    frame
}

// ============================================================================
// PLAN-669: #[api] by-name argument binding
// ============================================================================
// Shared binder for the legacy positional assembly sites (async
// build_handler_args, shim_http_server_listen, serve_blocking_stdnet,
// run_http_server_blocking). Per declared param, in declaration order,
// binding precedence is: path segment (by name) → body JSON field → query
// param (back_proxy Plan 658 parity). Missing params → HTTP 400 naming the
// param; a lone trailing unbound param whose name follows the meta-param
// convention (META_PARAM_NAMES) receives the cookies/auth metadata JSON
// (Plan 317 Phase 11 opt-in — async site passes Some). Sites whose fn
// has no API_PARAM_SIGS entry keep the legacy positional behavior unchanged.

/// PLAN-669 binder failure — site maps to an HTTP error response.
#[derive(Debug)]
pub(crate) enum ApiArgBindError {
    /// 400 — missing param / unconvertible value; message is response detail.
    BadRequest(String),
    /// 500 — body value marshal failure (server fault, not client's).
    Internal(String),
}

/// Declared-param kind derived from the `Type` Display string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApiTyKind {
    Int,
    Float,
    Bool,
    Str,
}

impl ApiTyKind {
    fn from_ty(ty: &str) -> Self {
        match ty.trim() {
            "int" | "i64" | "byte" | "uint" | "usize" | "u64" => ApiTyKind::Int,
            "float" | "double" => ApiTyKind::Float,
            "bool" => ApiTyKind::Bool,
            _ => ApiTyKind::Str,
        }
    }
}

/// Push a string-sourced (path/query) value converted per declared type.
/// Mirrors json_to_vm_value's marshalling exactly (int → encode_i32(i64 as
/// i32), float → push_f64, bool → encode_bool) so body- and string-sourced
/// params land in identical VM slots; strings go through the Plan 510 G1-1
/// push_str_arg throat. PLAN-675 T-02: back_proxy session dispatch consumes
/// this too (crate-internal) so gallery proxy path params coerce per `#[api]`
/// signature — the standalone-server semantics, one implementation.
/// PLAN-734 T-03：body JSON 值按声明类型校验（AC-02）。
/// str/bool/int/float 形态精确匹配；optional 接受 null；数组/record 走
/// 结构存在性（深度展开由 marshal 承担）。未知声明类型不在此拒绝
/// （Unsupported 类按 transport 矩阵在生成期诊断；运行期按原行为 marshal）。
fn validate_body_value(declared_ty: &str, v: &serde_json::Value) -> Result<(), String> {
    use crate::api::contract::ParamKind;
    let kind = ParamKind::from_param_string(declared_ty, &[]);
    let mismatch = |expect: &str| format!("expected {expect}, got {}", json_value_kind_name(v));
    match kind {
        ParamKind::Str => {
            if !v.is_string() {
                return Err(mismatch("str"));
            }
        }
        ParamKind::Bool => {
            if !v.is_boolean() {
                return Err(mismatch("bool"));
            }
        }
        ParamKind::Int => {
            if !v.is_i64() && !v.is_u64() {
                return Err(mismatch("int"));
            }
        }
        ParamKind::Float => {
            if !v.is_number() {
                return Err(mismatch("float"));
            }
            if let Some(f) = v.as_f64() {
                if !f.is_finite() {
                    return Err("float must be finite (NaN/Infinity rejected)".into());
                }
            }
        }
        ParamKind::Optional(_) => {
            if !v.is_null() {
                let inner = declared_ty.trim().trim_start_matches('?').trim_end_matches('?');
                let inner_kind = ParamKind::from_param_string(inner, &[]);
                let bad_scalar = match (&inner_kind, v) {
                    (ParamKind::Str, serde_json::Value::String(_)) => false,
                    (ParamKind::Bool, serde_json::Value::Bool(_)) => false,
                    (ParamKind::Int, v) => !(v.is_i64() || v.is_u64()),
                    (ParamKind::Float, v) => !v.is_number(),
                    _ => false,
                };
                if bad_scalar {
                    return Err(mismatch(inner));
                }
            }
        }
        ParamKind::Array(_) => {
            if !v.is_array() {
                return Err(mismatch("array"));
            }
        }
        ParamKind::Record(_) => {
            if !v.is_object() {
                return Err(mismatch("record"));
            }
        }
        ParamKind::Unsupported(_) => {}
    }
    Ok(())
}

fn json_value_kind_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                "int"
            } else {
                "float"
            }
        }
        serde_json::Value::String(_) => "str",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "record",
    }
}

pub(crate) fn push_typed_string_arg(
    vm: &crate::vm::engine::AutoVM,
    task: &mut crate::vm::task::AutoTask,
    sig: &ApiParamSig,
    val: &str,
    method: &str,
    req_path: &str,
) -> Result<(), ApiArgBindError> {
    let bad = |expected: &str| {
        ApiArgBindError::BadRequest(format!(
            "invalid value for param `{}` (expected {}): {:?} — {} {}",
            sig.name, expected, val, method, req_path
        ))
    };
    match ApiTyKind::from_ty(&sig.ty) {
        ApiTyKind::Int => match val.parse::<i64>() {
            Ok(i) => {
                // PLAN-734 T-03（E4 修复）：int 语义 = i64 全域（不截断）；
                // 定点变体按各自范围校验（超界 400，不静默回绕）。
                let range_ok = match sig.ty.trim() {
                    "u32" => (0..=u32::MAX as i64).contains(&i),
                    "u64" | "usize" => i >= 0,
                    "uint" => (0..=4_294_967_295i64).contains(&i),
                    "byte" => (0..=255).contains(&i),
                    _ => true, // int / i64 / i32：i64 全域交付
                };
                if !range_ok {
                    return Err(bad(sig.ty.trim()));
                }
                task.ram.push_nv(crate::vm::ffi::convert::encode_i64_with_heap(
                    vm, i,
                ));
            }
            Err(_) => return Err(bad(sig.ty.trim())),
        },
        ApiTyKind::Float => match val.parse::<f64>() {
            Ok(f) => task.ram.push_f64(f),
            Err(_) => return Err(bad(sig.ty.trim())),
        },
        ApiTyKind::Bool => match val {
            "true" => task.ram.push_nv(auto_val::encode_bool(true)),
            "false" => task.ram.push_nv(auto_val::encode_bool(false)),
            _ => return Err(bad("bool")),
        },
        ApiTyKind::Str => push_str_arg(vm, task, val),
    }
    Ok(())
}

/// Bind `#[api]` handler args by name onto the task stack. Returns the
/// number of args pushed. `body_json` is the parsed request body (when
/// parseable); `whole_body` the body-source string (multipart fields JSON,
/// converted form object, or the raw body — a lone str param that doesn't
/// bind by name receives it whole, the legacy single-arg body contract);
/// `metadata_json` the cookies/auth payload a trailing opt-in param receives
/// (recognized by param name — see `META_PARAM_NAMES`).
pub(crate) fn bind_api_args_by_name(
    vm: &crate::vm::engine::AutoVM,
    task: &mut crate::vm::task::AutoTask,
    sigs: &[ApiParamSig],
    route_match: &RouteMatch,
    body_json: Option<&serde_json::Value>,
    whole_body: &str,
    metadata_json: Option<&str>,
    method: &str,
    req_path: &str,
    upload_handle: Option<i64>,
) -> Result<usize, ApiArgBindError> {
    let mut n_args = 0usize;
    let mut unbound: Vec<&ApiParamSig> = Vec::new();
    for sig in sigs {
        // PLAN-730 T-05：UploadRequest 注入参数——类型身份识别（734 契约
        // 单源）置于 path/body/query/meta 全部规则**之前**（防 "req" meta
        // 名约定与 whole-body 单参容忍吞掉注入参数；AC-01 反例防线）。
        if crate::api::contract::is_upload_param(&sig.ty) {
            match upload_handle {
                Some(h) => {
                    task.ram.push_i32(h as i32);
                    n_args += 1;
                    continue;
                }
                None => {
                    return Err(ApiArgBindError::Internal(format!(
                        "upload param `{}` has no injected request capability",
                        sig.name
                    )));
                }
            }
        }
        if let Some((_, v)) = route_match.path_params.iter().find(|(n, _)| n == &sig.name) {
            push_typed_string_arg(vm, task, sig, v, method, req_path)?;
            n_args += 1;
            continue;
        }
        if let Some(v) = body_json.and_then(|b| b.get(&sig.name)) {
            // PLAN-734 T-03：body 值按声明类型校验（错形态 400，不静默
            // 按值自身形态 marshal；AC-02）。int 语义 i64 全域。
            if let Err(msg) = validate_body_value(&sig.ty, v) {
                return Err(ApiArgBindError::BadRequest(format!(
                    "invalid value for param `{}`: {}",
                    sig.name, msg
                )));
            }
            crate::vm::ffi::stdlib::json_to_vm_value(task, vm, v, 0).map_err(|e| {
                ApiArgBindError::Internal(format!(
                    "body param `{}` marshal failed: {e:?}",
                    sig.name
                ))
            })?;
            n_args += 1;
            continue;
        }
        if let Some((_, v)) = route_match
            .query_params
            .iter()
            .find(|(n, _)| n == &sig.name)
        {
            push_typed_string_arg(vm, task, sig, v, method, req_path)?;
            n_args += 1;
            continue;
        }
        unbound.push(sig);
    }

    // Whole-body single-param tolerance (legacy single-arg body contract —
    // Plan 346 "Push body"; checked BEFORE the metadata rule so `fn save(data
    // str)` + any non-matching body, `fn upload(form str)` + multipart fields
    // JSON, and unparseable text bodies all keep receiving the whole body).
    if unbound.len() == 1 && sigs.len() == 1 && !whole_body.is_empty() {
        push_str_arg(vm, task, whole_body);
        return Ok(n_args + 1);
    }

    // Trailing metadata opt-in (Plan 317 Phase 11 semantics, by-name form):
    // exactly one trailing declared param unbound AND its name follows the
    // metadata-param convention (the 023-realworld / e2e corpus names it
    // `meta`) → cookies/auth JSON. Anything else unbound is a missing param.
    if unbound.len() == 1
        && std::ptr::eq(
            unbound[0],
            sigs.last().expect("unbound implies sigs nonempty"),
        )
        && crate::api::contract::is_meta_alias(&unbound[0].name)
    {
        if let Some(meta) = metadata_json {
            push_str_arg(vm, task, meta);
            return Ok(n_args + 1);
        }
    }

    if let Some(sig) = unbound.first() {
        return Err(ApiArgBindError::BadRequest(format!(
            "missing param `{}` for {} {}",
            sig.name, method, req_path
        )));
    }
    Ok(n_args)
}

/// Param names that opt into the cookies/auth metadata payload (Plan 317
/// Phase 11 / Plan 346 stage 4 convention — the declared trailing param that
/// receives `{"cookies":...,"auth":...}`). Case-insensitive.
const META_PARAM_NAMES: [&str; 4] = ["meta", "metadata", "req", "request"];

fn is_meta_param_name(name: &str) -> bool {
    META_PARAM_NAMES.contains(&name.trim().to_lowercase().as_str())
}

/// PLAN-669 sync-serve sites (serve_blocking_stdnet, run_http_server_blocking,
/// shim_http_server_listen): by-name bind when the fn's sigs are published,
/// else the legacy positional convention. Err → caller writes a 400/500
/// response and skips the handler call. No metadata source on these loops —
/// a meta-convention param 400s with a named error instead of garbage.
pub(crate) fn bind_api_args_or_legacy(
    vm: &crate::vm::engine::AutoVM,
    ht: &mut crate::vm::task::AutoTask,
    fn_name: &str,
    path_params: &[(String, String)],
    query_params: &[(String, String)],
    body: &str,
    method: &str,
    req_path: &str,
) -> Result<usize, ApiArgBindError> {
    if let Some(sigs) = api_param_sigs(fn_name) {
        let route_match = RouteMatch {
            fn_name: fn_name.to_string(),
            path_params: path_params.to_vec(),
            query_params: query_params.to_vec(),
        };
        let body_json: Option<serde_json::Value> = serde_json::from_str(body).ok();
        bind_api_args_by_name(
            vm,
            ht,
            &sigs,
            &route_match,
            body_json.as_ref(),
            body,
            None,
            method,
            req_path,
            None,
        )
    } else {
        let mut n_args = 0;
        for (_param_name, param_val) in path_params {
            // Plan 326 Phase 5 legacy heuristic (no sigs published).
            if let Ok(i) = param_val.parse::<i32>() {
                ht.ram.push_i32(i);
            } else {
                push_str_arg(vm, ht, param_val);
            }
            n_args += 1;
        }
        if !body.is_empty() {
            push_str_arg(vm, ht, body);
            n_args += 1;
        }
        Ok(n_args)
    }
}

/// Build handler arguments on the task's stack (path params + body).
/// Returns the number of args pushed.
/// PLAN-669: cookies + auth metadata JSON (the Plan 346 stage 4 payload).
/// PLAN-734 T-03：meta 构造单源（standalone 与 back-proxy 共用）。
pub(crate) fn build_meta_json(cookie_header: &str, auth_header: &str) -> String {
    cookies_auth_metadata(cookie_header, auth_header)
}

fn cookies_auth_metadata(cookie_header: &str, auth_header: &str) -> String {
    let cookies_json: String = if cookie_header.is_empty() {
        "{}".to_string()
    } else {
        let pairs: Vec<String> = cookie_header
            .split(';')
            .filter_map(|pair| {
                let pair = pair.trim();
                let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
                Some(format!(
                    "\"{}\":\"{}\"",
                    k.trim().replace('"', "\\\""),
                    v.trim().replace('"', "\\\"")
                ))
            })
            .collect();
        format!("{{{}}}", pairs.join(","))
    };
    let auth_val = if auth_header.is_empty() {
        "".to_string()
    } else {
        auth_header.replace('"', "\\\"")
    };
    format!(r#"{{"cookies":{},"auth":"{}"}}"#, cookies_json, auth_val)
}

/// PLAN-669: form-urlencoded body → JSON object string (fields addressable
/// by name; k/v percent-decoded), same conversion the legacy branch pushes.
fn form_urlencoded_to_json_object(body: &str) -> String {
    let pairs: Vec<String> = body
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            Some(format!(
                "\"{}\":\"{}\"",
                url_decode(k).replace('"', "\\\""),
                url_decode(v).replace('"', "\\\"")
            ))
        })
        .collect();
    format!("{{{}}}", pairs.join(","))
}

fn build_handler_args(
    vm: &crate::vm::engine::AutoVM,
    task_id: u64,
    route_match: &RouteMatch,
    body: &str,
    content_type: &str,
    cookie_header: &str,
    auth_header: &str,
    multipart_json: Option<&str>,
    method: &str,
    req_path: &str,
    upload_handle: Option<i64>,
) -> Result<usize, ApiArgBindError> {
    let mut n_args = 0;
    if let Some(_task_arc) = vm.tasks.get(&task_id) {
        if let Ok(mut task) = _task_arc.try_lock() {
            // PLAN-669: by-name binding when the fn's declared params are
            // published (codegen records every #[api] fn; spec §4.1's
            // injection rule). Producers without an entry (legacy) keep the
            // positional convention below unchanged.
            if let Some(sigs) = api_param_sigs(&route_match.fn_name) {
                // Body source: multipart fields JSON (Plan 346 5a B6) takes
                // the body slot; form-urlencoded converts to a JSON object;
                // otherwise the raw body string.
                let body_source: String = if let Some(mp) = multipart_json {
                    mp.to_string()
                } else if !body.is_empty()
                    && content_type.contains("application/x-www-form-urlencoded")
                {
                    form_urlencoded_to_json_object(body)
                } else {
                    body.to_string()
                };
                let body_json: Option<serde_json::Value> = serde_json::from_str(&body_source).ok();
                let metadata = cookies_auth_metadata(cookie_header, auth_header);
                return bind_api_args_by_name(
                    vm,
                    &mut task,
                    &sigs,
                    route_match,
                    body_json.as_ref(),
                    &body_source,
                    Some(&metadata),
                    method,
                    req_path,
                    upload_handle,
                );
            }

            // Push path params (existing behavior).
            for (_param_name, param_val) in &route_match.path_params {
                if let Ok(i) = param_val.parse::<i32>() {
                    task.ram.push_i32(i);
                } else {
                    // Plan 510 G1-1: 统一咽喉(裸写池无 dedup,rc 数组未覆盖时 retain 为静默 no-op)
                    push_str_arg(vm, &mut task, &param_val);
                }
                n_args += 1;
            }

            // Plan 346: Push query params as a JSON object string if no body.
            if !route_match.query_params.is_empty() && body.is_empty() {
                let json_parts: Vec<String> = route_match
                    .query_params
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "\"{}\":\"{}\"",
                            k.replace('"', "\\\""),
                            v.replace('"', "\\\"")
                        )
                    })
                    .collect();
                let json_str = format!("{{{}}}", json_parts.join(","));
                // Plan 510 G1-1: 统一咽喉(裸写池无 dedup,rc 数组未覆盖时 retain 为静默 no-op)
                push_str_arg(vm, &mut task, &json_str);
                n_args += 1;
            }

            // Plan 346: Push body.
            if let Some(mp) = multipart_json {
                // Plan 346 5a (B6): multipart push — fields + persisted-file
                // metadata as JSON (takes the body arg slot).
                // Plan 510 G1-1: 统一咽喉(裸写池无 dedup,rc 数组未覆盖时 retain 为静默 no-op)
                push_str_arg(vm, &mut task, &mp);
                n_args += 1;
            } else if !body.is_empty() {
                let body_to_push = if content_type.contains("application/x-www-form-urlencoded") {
                    form_urlencoded_to_json_object(body)
                } else {
                    body.to_string()
                };
                // Plan 510 G1-1: 统一咽喉(裸写池无 dedup,rc 数组未覆盖时 retain 为静默 no-op)
                push_str_arg(vm, &mut task, &body_to_push);
                n_args += 1;
            }

            // Plan 346 stage 4: Push cookies + auth as a JSON metadata string.
            // Format: {"cookies":{"key":"val"}, "auth":"Bearer xxx"}
            //
            // Plan 317 §11 Phase 11 (Bug fix): ONLY push the metadata when the
            // handler declared an extra parameter to receive it. Previously this
            // was unconditional, so a 1-param handler like `fn echo_id(id int)`
            // received [id=42, meta_json] with n_args=2 — the handler's `id`
            // slot bound to the meta JSON instead of 42 (returned
            // {"cookies":{},"auth":""}). Now we read the handler's declared
            // n_args from its FN_PROLOG and push metadata iff the handler
            // declares more params than the data args we've already pushed
            // (path/query/body). This makes the metadata opt-in via signature.
            let declared_n_args = vm.get_fn_n_args(&route_match.fn_name);
            let push_meta = match declared_n_args {
                Some(declared) => declared > n_args, // handler wants an extra param
                None => true,                        // unknown — preserve old behavior (defensive)
            };
            if push_meta {
                let meta_json = cookies_auth_metadata(cookie_header, auth_header);
                // Plan 510 G1-1: 统一咽喉(裸写池无 dedup,rc 数组未覆盖时 retain 为静默 no-op)
                push_str_arg(vm, &mut task, &meta_json);
                n_args += 1;
            }
        }
    }
    Ok(n_args)
}

/// PLAN-698 T-02/SD-02: POST 广播臂单测——事件 payload 形态与生成 Axum
/// 侧（api_gen broadcast_event_name + events::broadcast）逐字段对拍。
#[cfg(test)]
mod plan698_publisher_tests {
    use super::cleanup_sse_iterator;
    use super::{
        publish_post_broadcast, record_api_param_sigs, record_api_return_type, ApiParamSig,
    };
    use crate::vm::ffi::stdlib::bus_subscribe_for_test;

    fn sig(name: &str, ty: &str) -> ApiParamSig {
        ApiParamSig {
            name: name.to_string(),
            ty: ty.to_string(),
        }
    }

    #[test]
    fn typing_post_broadcasts_typing_event_with_first_str_param() {
        record_api_param_sigs("set_typing", vec![sig("sender", "str")]);
        record_api_return_type("set_typing", "()".into());
        record_api_return_type("stream", "~Stream<ChatEvent>".into());
        let mut rx = bus_subscribe_for_test();

        publish_post_broadcast("set_typing", r#"{"sender":"Carol"}"#, "null");

        let json = rx.blocking_recv().unwrap();
        let evt: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(evt["event"], "Typing");
        assert_eq!(evt["name"], "Carol");
    }

    #[test]
    fn entity_post_broadcasts_newtype_event_with_injected_discriminator() {
        record_api_return_type("send_message", "Message".into());
        record_api_return_type("stream", "~Stream<ChatEvent>".into());
        let mut rx = bus_subscribe_for_test();

        publish_post_broadcast(
            "send_message",
            r#"{"sender":"Bob","text":"hi"}"#,
            r#"{"id":5,"sender":"Bob","text":"hi","time":"Just now","mine":true}"#,
        );

        let json = rx.blocking_recv().unwrap();
        let evt: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            evt,
            serde_json::json!({
                "id": 5, "sender": "Bob", "text": "hi",
                "time": "Just now", "mine": true,
                "event": "NewMessage",
            })
        );
    }

    #[test]
    fn non_object_or_non_entity_responses_are_not_broadcast() {
        record_api_return_type("stream", "~Stream<ChatEvent>".into());
        record_api_return_type("stream_only_fn", "~Stream<ChatEvent>".into());
        let mut rx = bus_subscribe_for_test();

        // ~Stream 返回类型的 fn 自身不是实体创建——不广播。
        publish_post_broadcast("stream_only_fn", "{}", r#"{"any":1}"#);
        // 响应体不是 JSON 对象（标量/数组/坏 JSON）——不广播。
        record_api_return_type("scalar_fn", "int".into());
        publish_post_broadcast("scalar_fn", "{}", "42");
        publish_post_broadcast("scalar_fn", "{}", "not-json");

        // 无事件应到达（有界等待确认空）。
        let mut arrived = false;
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(300);
        while std::time::Instant::now() < deadline {
            if rx.try_recv().is_ok() {
                arrived = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(!arrived, "no event expected for stream/scalar responses");
    }
}

/// PLAN-698 F-1（review #1）: cleanup_sse_iterator 回收 AsyncHttpStream
/// 的 ASYNC_STREAMS 句柄——rx drop 即 bus 转发线程 blocking_send 失败
/// 退出（否则订阅端断连后线程永生）。
#[cfg(test)]
mod plan698_f1_cleanup_tests {
    use super::cleanup_sse_iterator;

    #[test]
    fn cleanup_reclaims_async_stream_handle() {
        let vm = crate::vm::engine::AutoVM::new(
            crate::vm::virt_memory::VirtualFlash::new_with_code(vec![
                crate::vm::opcode::OpCode::RET as u8,
            ]),
            1024,
        );
        let (task_id, iter_id, stream_id) = {
            let tid = vm.spawn_task(0, 8192);
            let arc = vm.tasks.get(&tid).expect("task spawned");
            let mut task = arc.blocking_lock();
            crate::vm::ffi::stdlib::shim_bus_subscribe(&mut task, &vm)
                .expect("subscribe shim runs");
            let iter_id = auto_val::decode_i32(task.ram.peek_nv(0)) as u32;
            let stream_id = match vm.iterators.get(&iter_id).map(|it| it.clone()) {
                Some(crate::vm::engine::Iterator::AsyncHttpStream(a)) => a.stream_id,
                other => panic!("expected AsyncHttpStream, got {other:?}"),
            };
            (tid, iter_id, stream_id)
        };
        assert!(
            crate::vm::ffi::stdlib::ASYNC_STREAMS
                .lock()
                .unwrap()
                .contains_key(&stream_id),
            "handle registered by subscribe"
        );

        // DashMap 读守卫全 drop 后再 cleanup（同 shard 自死锁陷阱纪律）。
        vm.tasks.remove(&task_id);
        cleanup_sse_iterator(&vm, iter_id);

        assert!(
            !crate::vm::ffi::stdlib::ASYNC_STREAMS
                .lock()
                .unwrap()
                .contains_key(&stream_id),
            "F-1: handle reclaimed on cleanup — forwarder thread unblocks and exits"
        );
        assert!(
            !vm.iterators.contains_key(&iter_id),
            "iterator entry removed"
        );
    }
}

/// PLAN-699 T-01 spike: prove the Axum/Hyper transport topology compiles and
/// runs before fixing the bridge types. Decision evidence lands in
/// docs/plans/reports/699-bridge-decision.md. Kept as the seed of the new
/// protocol E2E suite (raw-TCP framing probes); will be folded into the
/// real implementation's test module.
#[cfg(test)]
mod spike699_axum_transport {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    use axum::extract::Request;
    use axum::response::Response;
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto::Builder as ConnBuilder;
    use hyper_util::server::graceful::GracefulShutdown;
    use hyper_util::service::TowerToHyperService;

    const HEADER_BUF: usize = 64 * 1024;
    const HEADER_READ_TIMEOUT: Duration = Duration::from_secs(2);

    struct SpikeServer {
        port: u16,
        shutdown_tx: tokio::sync::watch::Sender<bool>,
        done_rx: std::sync::mpsc::Receiver<()>,
    }

    impl SpikeServer {
        fn start(router: axum::Router) -> Self {
            let (ready_tx, ready_rx) = std::sync::mpsc::channel::<u16>();
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let (done_tx, done_rx) = std::sync::mpsc::channel();
            let _handle = std::thread::Builder::new()
                .name("spike699-net".into())
                .stack_size(4 * 1024 * 1024)
                .spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("spike net runtime");
                    rt.block_on(async move {
                        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                            .await
                            .expect("spike bind");
                        let port = listener.local_addr().unwrap().port();
                        // Ready signal BEFORE the accept loop — start() waits
                        // on it; sending after the loop would deadlock.
                        let _ = ready_tx.send(port);
                        let graceful = GracefulShutdown::new();
                        let mut shutdown_rx = shutdown_rx;
                        loop {
                            tokio::select! {
                                accepted = listener.accept() => {
                                    let (stream, _peer) = match accepted {
                                        Ok(x) => x,
                                        Err(_) => break,
                                    };
                                    let svc = TowerToHyperService::new(router.clone());
                                    let watcher = graceful.watcher();
                                    tokio::spawn(async move {
                                        // http1_only: the transport contract is
                                        // HTTP/1.1 — the auto builder would
                                        // otherwise also serve h2c prior-knowledge
                                        // connections, silently widening scope.
                                        let mut builder = ConnBuilder::new(TokioExecutor::new())
                                            .http1_only();
                                        builder
                                            .http1()
                                            .timer(hyper_util::rt::TokioTimer::new())
                                            .max_buf_size(HEADER_BUF)
                                            .header_read_timeout(HEADER_READ_TIMEOUT);
                                        let conn = builder.serve_connection_with_upgrades(
                                            TokioIo::new(stream),
                                            svc,
                                        );
                                        let _ = watcher.watch(conn).await;
                                    });
                                }
                                _ = shutdown_rx.changed() => {
                                    // Graceful: stop accepting, drain in-flight
                                    // connections with a deadline, then force out.
                                    let _ = tokio::time::timeout(
                                        Duration::from_secs(2),
                                        graceful.shutdown(),
                                    )
                                    .await;
                                    break;
                                }
                            }
                        }
                        drop(listener);
                        let _ = done_tx.send(());
                    });
                })
                .expect("spawn spike net thread");
            let port = ready_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("spike server ready");
            SpikeServer {
                port,
                shutdown_tx,
                done_rx,
            }
        }
    }

    fn connect(port: u16) -> TcpStream {
        let stream = TcpStream::connect(("127.0.0.1", port)).expect("spike connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
    }

    fn read_response(stream: &mut TcpStream) -> String {
        read_response_until(stream, &|resp: &str, head_done: bool, _bytes: usize| {
            // One complete response head (+ its content-length body when present).
            if !head_done {
                return false;
            }
            let lower = resp.to_ascii_lowercase();
            let cl = lower
                .split("\r\n")
                .find_map(|l| {
                    l.strip_prefix("content-length:")
                        .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                })
                .unwrap_or(0);
            // No content-length = streaming body: the caller supplies its own
            // `until` predicate; default waits for EOF via head_done=false… so
            // keep the head-only contract here and let custom predicates decide.
            let pos = find_head_end(resp.as_bytes()).unwrap_or(resp.len());
            resp.len() >= pos + cl
        })
    }

    /// Read from the socket until `until` returns true (checked after every
    /// read), EOF, or the 5s read timeout fires.
    fn read_response_until(
        stream: &mut TcpStream,
        until: &dyn Fn(&str, bool, usize) -> bool,
    ) -> String {
        let mut resp = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = match stream.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            resp.extend_from_slice(&buf[..n]);
            let s = String::from_utf8_lossy(&resp).to_string();
            let head_done = find_head_end(&resp).is_some();
            if until(&s, head_done, resp.len()) {
                break;
            }
            if resp.len() > 256 * 1024 {
                break;
            }
        }
        String::from_utf8_lossy(&resp).into_owned()
    }

    fn find_head_end(b: &[u8]) -> Option<usize> {
        b.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4)
    }

    /// SSE body adapter: tokio mpsc Receiver → Stream with correct waker
    /// wiring (poll_recv registers the waker; a naive try_recv+Pending
    /// poll_fn never wakes and the body starves — spike-verified pitfall).
    struct FrameStream {
        rx: tokio::sync::mpsc::Receiver<String>,
    }

    impl futures::Stream for FrameStream {
        type Item = Result<String, std::convert::Infallible>;
        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            std::pin::Pin::new(&mut self.rx)
                .poll_recv(cx)
                .map(|opt| opt.map(Ok))
        }
    }

    async fn spike_router() -> axum::Router {
        axum::Router::new()
            .route(
                "/api/echo-len",
                axum::routing::post(|body: String| async move { body.len().to_string() }),
            )
            .route("/api/hello", axum::routing::get(|| async { "hi" }))
            .route(
                "/api/sse",
                axum::routing::get(|| async {
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(1);
                    tokio::spawn(async move {
                        for i in 0..3 {
                            if tx.send(format!("data: {i}\n\n")).await.is_err() {
                                break;
                            }
                        }
                    });
                    let stream = FrameStream { rx };
                    axum::response::Response::builder()
                        .status(200)
                        .header("content-type", "text/event-stream")
                        .header("cache-control", "no-cache")
                        .body(axum::body::Body::from_stream(stream))
                        .unwrap()
                }),
            )
    }

    /// AC-01 probe: keep-alive — two requests on one connection.
    #[test]
    fn spike_keepalive_two_requests_one_connection() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        let req = "GET /api/hello HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        s.write_all(req.as_bytes()).unwrap();
        s.write_all(req.as_bytes()).unwrap();
        let resp = read_response_until(&mut s, &|resp, _head, _bytes| {
            resp.matches("HTTP/1.1").count() >= 2 && {
                // Second head must also have its (content-length) body.
                let second = resp.split("HTTP/1.1").nth(2).unwrap_or("");
                second.contains("\r\n\r\n")
            }
        });
        let count = resp.matches("HTTP/1.1 200 OK").count();
        assert_eq!(
            count, 2,
            "two keep-alive responses on one connection, got: {resp:?}"
        );
    }

    /// AC-01 probe: chunked request body is decoded by the transport.
    #[test]
    fn spike_chunked_request_body_decoded() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        let body = b"hello chunked world";
        let req = format!(
            "POST /api/echo-len HTTP/1.1\r\nHost: 127.0.0.1\r\nTransfer-Encoding: chunked\r\n\r\n\
             {:x}\r\n{}\r\n0\r\n\r\n",
            body.len(),
            String::from_utf8_lossy(body)
        );
        s.write_all(req.as_bytes()).unwrap();
        let resp = read_response(&mut s);
        assert!(
            resp.starts_with("HTTP/1.1 200"),
            "chunked body accepted, got: {resp:?}"
        );
        assert!(
            resp.contains(&body.len().to_string()),
            "decoded length echoed, got: {resp:?}"
        );
    }

    /// AC-03 probe: oversized headers must produce a determinate outcome.
    #[test]
    fn spike_oversized_headers_rejected() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        let big = "X-Big: ".to_string() + &"a".repeat(HEADER_BUF + 4096);
        let req = format!("GET /api/hello HTTP/1.1\r\nHost: 127.0.0.1\r\n{big}\r\n\r\n");
        let _ = s.write_all(req.as_bytes());
        let resp = read_response(&mut s);
        let status = resp.lines().next().unwrap_or("").to_string();
        assert!(
            status.contains("431") || status.contains("400") || resp.is_empty(),
            "oversized headers → 431/400/close, got: {status:?}"
        );
        eprintln!(
            "[spike699] oversized headers outcome: {status:?} (bytes={})",
            resp.len()
        );
    }

    /// AC-03 probe: slow headers must hit the header read timeout and close.
    #[test]
    fn spike_slow_headers_timeout() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        let _ = s.write_all(b"GET /api/hello HTTP/1.1\r\nHost: 127.0.0.1\r\n");
        let _ = s.flush();
        let started = std::time::Instant::now();
        let mut resp = Vec::new();
        let _ = s.read_to_end(&mut resp);
        let elapsed = started.elapsed();
        assert!(
            elapsed >= Duration::from_millis(1500) && elapsed < Duration::from_secs(6),
            "slow headers closed after header_read_timeout, elapsed={elapsed:?}"
        );
        eprintln!(
            "[spike699] slow headers outcome: {:?} after {elapsed:?}",
            String::from_utf8_lossy(&resp)
                .lines()
                .next()
                .unwrap_or("<eof>")
        );
    }

    /// AC-04 probe: SSE streams over the transport.
    #[test]
    fn spike_sse_frames_stream() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        s.write_all(b"GET /api/sse HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        let resp = read_response_until(&mut s, &|resp, _head, _bytes| resp.contains("data: 2"));
        assert!(resp.contains("200 OK"), "SSE status, got {resp:?}");
        assert!(resp.contains("text/event-stream"), "SSE content-type");
        assert!(
            resp.contains("data: 0") && resp.contains("data: 2"),
            "frames streamed"
        );
    }

    /// AC-05 probe: graceful shutdown stops accept, drains, releases the port.
    #[test]
    fn spike_graceful_shutdown_releases_port() {
        let server = SpikeServer::start(futures::executor::block_on(spike_router()));
        let mut s = connect(server.port);
        s.write_all(b"GET /api/hello HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        let _ = read_response(&mut s);
        server.shutdown_tx.send(true).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        let done = server.done_rx.recv_timeout(Duration::from_secs(4));
        assert!(done.is_ok(), "net loop exited after graceful shutdown");
        // Port must be free again.
        let rebind = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { tokio::net::TcpListener::bind(("127.0.0.1", server.port)).await });
        assert!(rebind.is_ok(), "port rebindable after shutdown");
    }
}

// =============================================================================
// PLAN-093 (G-8 系) catch-all 路由匹配回归：axum `{*rest}` 翻译为 `*rest` 段
// 后须消费全部剩余段——此前段数严格相等检查使多段剩余路径永远 404。
// =============================================================================
#[cfg(test)]
mod plan093_catch_all_tests {
    use super::{match_route, HttpRoute};

    fn routes() -> Vec<HttpRoute> {
        vec![
            HttpRoute {
                method: "GET".into(),
                path: "/api/files/:workspace_id/*path".into(),
                fn_name: "workspace_file".into(),
            },
            HttpRoute {
                method: "GET".into(),
                path: "/api/chats/session/:id".into(),
                fn_name: "chat_get".into(),
            },
        ]
    }

    #[test]
    fn catch_all_consumes_remaining_segments() {
        let m = match_route(&routes(), "GET", "/api/files/default/targets/probe-a/pac.at")
            .expect("multi-segment catch-all must match");
        assert_eq!(m.fn_name, "workspace_file");
        let ws = m.path_params.iter().find(|(k, _)| k == "workspace_id").unwrap();
        assert_eq!(ws.1, "default");
        let rest = m.path_params.iter().find(|(k, _)| k == "path").unwrap();
        assert_eq!(rest.1, "targets/probe-a/pac.at");
    }

    #[test]
    fn catch_all_still_requires_prefix() {
        // 少于路由前缀段数 → 不匹配（workspace_id 缺失）。
        assert!(match_route(&routes(), "GET", "/api/files").is_none());
    }

    #[test]
    fn catch_all_single_segment_remainder() {
        let m = match_route(&routes(), "GET", "/api/files/default/notes.md")
            .expect("single-segment remainder must match");
        let rest = m.path_params.iter().find(|(k, _)| k == "path").unwrap();
        assert_eq!(rest.1, "notes.md");
    }

    #[test]
    fn non_catch_all_still_strict_length() {
        // 非 catch-all 路由保持严格段数：多余段不匹配。
        assert!(match_route(&routes(), "GET", "/api/chats/session/a/b").is_none());
        let m = match_route(&routes(), "GET", "/api/chats/session/abc")
            .expect("exact-length param route must match");
        assert_eq!(m.fn_name, "chat_get");
    }

    #[test]
    fn catch_all_decodes_percent_encoded_remainder() {
        let m = match_route(&routes(), "GET", "/api/files/default/wiki/Hello%20World.ad")
            .expect("encoded remainder must match");
        let rest = m.path_params.iter().find(|(k, _)| k == "path").unwrap();
        assert_eq!(rest.1, "wiki/Hello World.ad");
    }
}
