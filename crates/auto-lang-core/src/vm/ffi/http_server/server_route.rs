//! PLAN-843 结构拆分：HTTP 服务 route 职责（原样搬移）。
use super::*;

pub use crate::vm::host_capabilities::http_service::messages::{ApiParamSig, HttpRoute};

/// Result of matching a request against routes.
#[derive(Debug, Clone)]
pub struct RouteMatch {
    pub fn_name: String,
    pub path_params: Vec<(String, String)>,
    pub query_params: Vec<(String, String)>,
}

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
    if let Some(value) = with_program_metadata(|p| p.params.get(fn_name).cloned()) {
        return value;
    }
    API_PARAM_SIGS
        .lock()
        .ok()
        .and_then(|t| t.get(fn_name).cloned())
}

/// Codegen 在 #[api] fn 编译时发布返回类型（dep-module Codegen 同步发布）。
pub fn record_api_return_type(fn_name: &str, ret_display: String) {
    if let Ok(mut table) = API_RETURN_TYPES.lock() {
        table.insert(fn_name.to_string(), ret_display);
    }
}

pub(super) fn api_return_type(fn_name: &str) -> Option<String> {
    with_program_metadata(|p| p.returns.get(fn_name).cloned()).unwrap_or_else(|| {
        API_RETURN_TYPES
            .lock()
            .ok()
            .and_then(|t| t.get(fn_name).cloned())
    })
}

/// Codegen 发布遮蔽标记（与 record_api_return_type 同点调用）。
pub fn record_api_return_shadowed(fn_name: &str) {
    if let Ok(mut table) = API_RETURN_SHADOWED.lock() {
        table.insert(fn_name.to_string());
    }
}

/// 本工程是否声明了 ~Stream 端点（对齐 api_gen 的 has_sse 广播门控）。
pub fn has_stream_endpoint() -> bool {
    if let Some(value) = with_program_metadata(|p| p.returns.values().any(|r| r.contains("Stream")))
    {
        return value;
    }
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
    if let Ok(mut table) = API_RETURN_SHADOWED.lock() {
        table.clear();
    }
    if let Ok(mut table) = API_ASYNC_RETURNS.lock() {
        table.clear();
    }
}

/// Codegen 在 fn 编译时发布（声明 ret 为 Future<T> 时调用）。
pub fn record_api_async_return(fn_name: &str) {
    if let Ok(mut table) = API_ASYNC_RETURNS.lock() {
        table.insert(fn_name.to_string());
    }
}

/// PLAN-729 T-04：编组期文件返回门——声明返回类型（`FileResponse` 或
/// `Future<FileResponse>`）含 "FileResponse"。与登记命中（
/// http_server_file::take_file_response）共同识别文件结果；普通 int 即使
/// 数值撞上描述符 id 也走 JSON 兜底（AC-01 反例防线）。
pub fn fn_is_api_file_return(fn_name: &str) -> bool {
    fn_response_kind(fn_name) == crate::api::contract::ResponseKind::File
}

/// PLAN-730 T-02：编组期上传收据返回门——声明返回类型（`UploadReceipt`
/// 或 `Future<UploadReceipt>`）含 "UploadReceipt"。与登记命中
/// （http_upload::take_upload_receipt）共同识别；普通 int 撞号走 JSON
/// 兜底（同 729 三重命中形态）。
pub fn fn_is_api_upload_return(fn_name: &str) -> bool {
    fn_response_kind(fn_name) == crate::api::contract::ResponseKind::Upload
}

/// PLAN-734 T-03：返回种类单源——API_RETURN_TYPES 条目经契约身份分类
/// （ResponseKind::from_return_string；假同名类型不命中）。
/// PLAN-776 T-06：编译期冻结的用户同名遮蔽优先（遮蔽 → Json——同裸名
/// 用户类型不冒领 File/Upload/Stream/ExplicitResponse 专属身份）。
pub fn fn_response_kind(fn_name: &str) -> crate::api::contract::ResponseKind {
    if let Some(value) = with_program_metadata(|p| {
        if p.shadowed.contains(fn_name) {
            crate::api::contract::ResponseKind::Json
        } else {
            p.returns
                .get(fn_name)
                .map(|r| crate::api::contract::ResponseKind::from_return_string(r))
                .unwrap_or(crate::api::contract::ResponseKind::Json)
        }
    }) {
        return value;
    }
    if API_RETURN_SHADOWED
        .lock()
        .map(|t| t.contains(fn_name))
        .unwrap_or(false)
    {
        return crate::api::contract::ResponseKind::Json;
    }
    API_RETURN_TYPES
        .lock()
        .ok()
        .and_then(|t| {
            t.get(fn_name)
                .map(|r| crate::api::contract::ResponseKind::from_return_string(r))
        })
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
pub(super) fn cors_origin() -> String {
    std::env::var("AUTO_CORS_ORIGIN").unwrap_or_else(|_| "*".to_string())
}

/// Plan 349 步骤 7/8 (W5): static CORS response-header block (CRLF-terminated, no
/// trailing blank line) appended to every server response so browser clients
/// can read the body. Origin is read once per call to honor env overrides.
pub fn cors_headers() -> String {
    let origin = cors_origin();
    format!(
        "Access-Control-Allow-Origin: {}\r\n\
         Access-Control-Allow-Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
         Access-Control-Max-Age: 86400\r\n",
        origin
    )
}

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
pub(super) fn rate_limit_take(ip: &str) -> Option<u64> {
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
pub(super) fn gen_request_id() -> String {
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
pub fn handle_cors_preflight(method: &str) -> Option<String> {
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
pub(super) fn url_decode(s: &str) -> String {
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
    if let Some(routes) = with_program_metadata(|p| p.routes.clone()) {
        return routes;
    }
    crate::vm::ffi::stdlib::get_http_routes()
        .into_iter()
        .map(|(method, path, fn_name)| HttpRoute {
            method,
            path,
            fn_name,
        })
        .collect()
}
