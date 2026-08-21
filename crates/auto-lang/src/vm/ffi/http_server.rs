//! Plan 321/322: AutoHttpServer — unified HTTP server backend wrapping Axum.
//!
//! This module is shared by both VM mode (via native shim) and a2r mode
//! (via generated Rust code). It encapsulates Axum Router construction,
//! route matching, SSE streaming, and the !Send VM bridging.
//!
//! ## VM mode bridging strategy
//!
//! AutoVM is !Send (Rc<RefCell> in type system). Axum handlers must be
//! Send + 'static futures. To bridge:
//!
//! 1. The HTTP server runs on a dedicated OS thread (not the tokio runtime
//!    that drives the VM's async task system).
//! 2. On that thread, we create a `current_thread` tokio runtime and run
//!    `axum::serve` inside `block_on`.
//! 3. Each Axum handler is a thin async wrapper that uses `spawn_blocking`
//!    to call the VM synchronously (the VM lives on the same thread, so
//!    the blocking call is safe — it just blocks the current_thread runtime,
//!    which is fine since there's only one worker).
//!
//! Alternatively (simpler for MVP): skip Axum entirely for VM mode and
//! keep the existing std::net implementation, but route it through this
//! module's route table for unified route matching logic. Axum can be
//! added later for a2r mode.

use std::io::{Read, Write, BufRead};
use std::net::TcpListener;

/// An HTTP route registered with the server.
#[derive(Debug, Clone)]
pub struct HttpRoute {
    pub method: String,
    pub path: String,
    pub fn_name: String,
}

/// Result of matching a request against routes.
pub struct RouteMatch {
    pub fn_name: String,
    pub path_params: Vec<(String, String)>,
    pub query_params: Vec<(String, String)>,
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
        query_string.split('&')
            .filter_map(|pair| {
                let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
                Some((url_decode(k), url_decode(v)))
            })
            .collect()
    };

    for route in routes {
        if route.method.to_uppercase() != method.to_uppercase() {
            continue;
        }
        let route_segments: Vec<&str> = route.path.split('/').collect();
        let path_segments: Vec<&str> = path_only.split('/').collect();
        if route_segments.len() != path_segments.len() {
            continue;
        }
        let mut params = Vec::new();
        let mut matched = true;
        for (rs, ps) in route_segments.iter().zip(path_segments.iter()) {
            if let Some(param_name) = rs.strip_prefix(':') {
                params.push((param_name.to_string(), ps.to_string()));
            } else if *rs == "*" || rs.starts_with('*') {
                // Plan 346: Wildcard route — matches any remaining segments.
                continue;
            } else if rs != ps {
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
/// request-id minting for `handle_connection_async`.

/// Plan 346 5e (B6): active rate-limit config `(max_requests, window_ms)`.
/// `None` (default) = no limiting — backwards compatible until
/// `http.rate_limit(n, ms)` is called.
static RATE_LIMIT_CFG: std::sync::Mutex<Option<(u32, u64)>> =
    std::sync::Mutex::new(None);
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
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
}

/// Plan 346 5a: save a file part under the upload dir (env `AUTO_UPLOAD_DIR`,
/// default `./uploads`) and return the stored path. The original filename is
/// reduced to its basename and prefixed with a counter so concurrent uploads
/// of the same name cannot clobber each other.
fn store_multipart_file(filename: &str, data: &[u8]) -> String {
    static FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let dir = std::env::var("AUTO_UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    let _ = std::fs::create_dir_all(&dir);
    let base = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("upload")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>();
    let base = if base.is_empty() { "upload".to_string() } else { base };
    let n = FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let path = std::path::Path::new(&dir).join(format!("{}_{}", n, base));
    let _ = std::fs::write(&path, data);
    path.to_string_lossy().to_string()
}

/// Plan 346 5a: build the handler-facing JSON for a multipart request:
/// `{"fields":{"k":"v"},"files":[{"field","filename","path","size"}]}`.
/// Text parts land in `fields`; file parts are persisted and described in
/// `files` (handlers read metadata, not megabytes of bytes).
pub fn multipart_to_handler_json(parts: Vec<MultipartPart>) -> String {
    let mut fields: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    for part in parts {
        match part.filename {
            None => {
                let text = String::from_utf8_lossy(&part.data).to_string();
                fields.push(format!(
                    "\"{}\":\"{}\"",
                    part.name.replace('"', "\\\""),
                    text.replace('\\', "\\\\").replace('"', "\\\"")
                ));
            }
            Some(fname) => {
                let path = store_multipart_file(&fname, &part.data);
                files.push(format!(
                    "{{\"field\":\"{}\",\"filename\":\"{}\",\"path\":\"{}\",\"size\":{}}}",
                    part.name.replace('"', "\\\""),
                    fname.replace('"', "\\\""),
                    path.replace('\\', "/").replace('"', "\\\""),
                    part.data.len()
                ));
            }
        }
    }
    format!(
        "{{\"fields\":{{{}}},\"files\":[{}]}}",
        fields.join(","),
        files.join(",")
    )
}

/// Plan 346 5e (B6): enable per-IP fixed-window rate limiting.
/// Called by the `http.rate_limit(max_requests, window_ms)` native.
pub fn set_rate_limit(max_requests: u32, window_ms: u64) {
    if let Ok(mut cfg) = RATE_LIMIT_CFG.lock() {
        *cfg = if max_requests == 0 { None } else { Some((max_requests, window_ms.max(1))) };
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

/// Plan 346 #12 (B6): response-header block carrying the request id
/// (CRLF-terminated), appended next to `cors_headers()` on every response.
fn request_id_header(request_id: &str) -> String {
    format!("X-Request-Id: {}\r\n", request_id)
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
        500 => "Internal Server Error",
        _ => "",
    }
}

/// Plan 346 3c: write a handler-returned Response object (status/headers/body)
/// directly to the connection. The parts come from
/// `stdlib::lookup_http_response`.
async fn write_http_response_object(
    stream: &mut tokio::net::TcpStream,
    res: (u16, Vec<(String, String)>, Vec<u8>),
    req_method: &str,
    req_path: &str,
    elapsed_ms: u128,
    request_id: &str,
) {
    use tokio::io::AsyncWriteExt;
    let (status, headers, body) = res;
    let mut head = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n",
        status,
        http_status_reason(status),
        body.len()
    );
    for (k, v) in &headers {
        head.push_str(k);
        head.push_str(": ");
        head.push_str(v);
        head.push_str("\r\n");
    }
    head.push_str(&cors_headers());
    head.push_str(&request_id_header(request_id));
    head.push_str("\r\n");
    let _ = stream.write_all(head.as_bytes()).await;
    if !body.is_empty() {
        let _ = stream.write_all(&body).await;
    }
    let _ = stream.flush().await;
    eprintln!(
        "[HTTP] {} {} [{}] → {} ({}ms)",
        req_method, req_path, request_id, status, elapsed_ms
    );
}

/// Plan 349 步骤 7/8 (W5): write a CORS preflight (OPTIONS) response and return true if
/// the request is an OPTIONS method; returns false otherwise so the caller can
/// continue normal routing. Used by both blocking and async servers.
pub(crate) fn handle_cors_preflight(method: &str) -> Option<String> {
    if method.eq_ignore_ascii_case("OPTIONS") {
        Some(format!(
            "HTTP/1.1 204 No Content\r\n{}\r\n",
            cors_headers()
        ))
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
        .map(|(method, path, fn_name)| HttpRoute { method, path, fn_name })
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
pub fn nv_to_json(vm: &crate::vm::engine::AutoVM, nv: auto_val::NanoValue, depth: u32) -> Option<String> {
    const MAX_DEPTH: u32 = 32;

    // Tagged string (the canonical handler-returns-string path)
    if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv);
        let s = vm.strings.read().unwrap()
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
        return Some(if auto_val::decode_bool(nv) { "true".to_string() } else { "false".to_string() });
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
fn heap_object_to_json(
    vm: &crate::vm::engine::AutoVM,
    id: u64,
    depth: u32,
) -> Option<String> {
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
        if let Some(list) = guard.as_any().downcast_ref::<crate::vm::types::ListData<auto_val::Value>>() {
            let mut parts: Vec<String> = Vec::new();
            for elem in &list.elems {
                let json = value_to_json(vm, elem, depth + 1)
                    .unwrap_or_else(|| "null".to_string());
                parts.push(json);
            }
            return Some(format!("[{}]", parts.join(", ")));
        }
        // ListData<i32> (int collections, or struct lists where elements are
        // stored as heap object IDs >= 4000000).
        if let Some(list) = guard.as_any().downcast_ref::<crate::vm::types::ListData<i32>>() {
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
        if let Some(od) = guard.as_any().downcast_ref::<crate::vm::types::ObjectData>() {
            let mut parts: Vec<String> = Vec::new();
            for (key, val) in od.fields.iter() {
                let key_json = json_escape_string(&key.to_string());
                let val_json = value_to_json(vm, val, depth + 1)
                    .unwrap_or_else(|| "null".to_string());
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
fn value_to_json(vm: &crate::vm::engine::AutoVM, value: &auto_val::Value, depth: u32) -> Option<String> {
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
        Value::Bool(b) => Some(if *b { "true".to_string() } else { "false".to_string() }),
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
fn json_escape_string(s: &str) -> String {
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
            OpCode::FN_PROLOG as u8, 1, 0, OpCode::RET as u8,            // echo_id (1 param)
            OpCode::FN_PROLOG as u8, 2, 0, OpCode::RET as u8,            // create_note (2 params)
        ];
        let mut flash = VirtualFlash::new_with_code(bytecode);
        flash.exports_by_name.insert("echo_id".to_string(), 0);
        flash.exports_by_name.insert("create_note".to_string(), 4);
        let vm = AutoVM::new(flash, 1024);

        assert_eq!(vm.get_fn_n_args("echo_id"), Some(1), "1-param handler arity");
        assert_eq!(vm.get_fn_n_args("create_note"), Some(2), "2-param handler arity");
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
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some(r#""hello""#.to_string()));
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
            vec![auto_val::Value::Int(0), auto_val::Value::Str(auto_val::AutoStr::from("a"))],
            vec!["id".to_string(), "title".to_string()],
        );
        let b = GenericInstanceData::new_with_names(
            "Note".to_string(),
            vec![auto_val::Value::Int(1), auto_val::Value::Str(auto_val::AutoStr::from("b"))],
            vec!["id".to_string(), "title".to_string()],
        );
        let id_a = vm.insert_heap_object(a) as i32;
        let id_b = vm.insert_heap_object(b) as i32;
        // Allocate an array id the same way CREATE_ARRAY does now.
        let arr_id = vm.insert_heap_object(crate::vm::types::ListData {
            elems: vec![
                auto_val::Value::Int(id_a),
                auto_val::Value::Int(id_b),
            ],
            storage: None,
        });
        // The handler returns the array id as i32.
        let nv = auto_val::encode_i32(arr_id as i32);
        let json = super::nv_to_json(&vm, nv, 0).unwrap();
        assert_eq!(json, r#"[{"id": 0, "title": "a"}, {"id": 1, "title": "b"}]"#);
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
        assert_eq!(super::nv_to_json(&vm, nv, 0), Some(r#""found""#.to_string()));
    }

    /// Option.None → JSON null (Plan 326: we serialize None as `null` to keep
    /// the JSON response well-formed; a 404 mapping can be layered on later).
    #[test]
    fn nv_to_json_option_none() {
        let vm = fresh_vm();
        let inst = GenericInstanceData::new_with_names(
            "Option.None".to_string(),
            vec![],
            vec![],
        );
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
            write!(stream, "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n", path).unwrap();
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
            let mut req = format!("GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n", path);
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

        #[test]
        fn e2e_struct_handler_returns_json() {
            let port = start_server(r#"
type Note { id int; title str }

#[api(method = "GET", path = "/api/notes/test")]
fn get_note() Note {
    Note { id: 1, title: "hello" }
}
"#, 18731);
            let resp = http_get(port, "/api/notes/test");
            let body = body_of(&resp);
            // The fix: body must be JSON object, not the bare heap-id "4000000".
            assert_eq!(
                body, r#"{"id": 1, "title": "hello"}"#,
                "struct handler JSON: full resp = {:?}", resp
            );
        }

        #[test]
        fn e2e_int_path_param_handler() {
            let port = start_server(r#"
#[api(method = "GET", path = "/api/echo/:id")]
fn echo_id(id int) int {
    id
}
"#, 18732);
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
            let port = start_server(r#"
#[api(method = "GET", path = "/api/counter")]
fn counter_handler() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
"#, 18733);
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
            let port = start_server(r#"
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
#[api(method = "GET", path = "/api/stream")]
fn stream_handler() ~Iter<int> {
    return counter()
}
"#, 18734);
            let resp = http_get(port, "/api/stream");
            let body = body_of(&resp);
            assert!(body.contains("data: 1"), "indirect SSE frame 1: body={:?}", body);
            assert!(body.contains("data: 2"), "indirect SSE frame 2: body={:?}", body);
            assert!(body.contains("data: 3"), "indirect SSE frame 3: body={:?}", body);
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
        fn http_get_until(port: u16, path: &str, need_fragments: &[&str], max_attempts: usize) -> String {
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
            let port = start_server(r#"
#[api(method = "GET", path = "/api/count")]
fn counter_handler() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
"#, 18735);

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
            assert!(body1.contains("data: 1") && body1.contains("data: 2") && body1.contains("data: 3"),
                "conn1 incomplete: body={:?}", body1);
            assert!(body2.contains("data: 1") && body2.contains("data: 2") && body2.contains("data: 3"),
                "conn2 incomplete: body={:?}", body2);
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
            let port = start_server(r#"
#[api(method = "GET", path = "/old")]
fn old_handler() int {
    return http.response_redirect("/new", 302)
}

#[api(method = "GET", path = "/new")]
fn new_handler() str {
    return "arrived"
}
"#, 18736);
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
            let db_src = std::fs::read_to_string(back.join("db.at"))
                .expect("023 db.at must exist in-repo");

            // Types block from api.at (up to `use db`), db.at minus its own
            // `use api:` import, plus bearer_token/json_str extracted verbatim.
            let types_block: String = api_src
                .split("use db")
                .next()
                .unwrap_or("")
                .to_string();
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
fn h_login(arg str) User {{
    return login(json_str(arg, "email"), json_str(arg, "password"))
}}

#[api(method = "POST", path = "/api/users")]
fn h_register(arg str) User {{
    return register(json_str(arg, "username"), json_str(arg, "email"), json_str(arg, "password"))
}}

#[api(method = "GET", path = "/api/user")]
fn h_current_user(meta str) User {{
    return current_user(bearer_token(meta))
}}


#[api(method = "POST", path = "/api/articles")]
fn h_create_article(arg str, meta str) Article {{
    let u User = current_user(bearer_token(meta))
    if u.id == 0 {{
        let rejected Article = Article {{ slug: "", title: "", description: "", body: "", tagList: "", author: "", favoritesCount: 0, createdAt: "" }}
        return rejected
    }}
    return create_article(json_str(arg, "slug"), json_str(arg, "title"), json_str(arg, "description"), json_str(arg, "body"), json_str(arg, "tagList"), u.username)
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
            assert!(bad.contains("\"id\": 0"), "wrong password must fail: {}", bad);

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
            let port = start_server(r#"
#[api(method = "POST", path = "/api/upload")]
fn upload(form str) str {
    return form
}
"#, 18744);

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
            assert_eq!(stored_files.len(), 1, "exactly one stored file: {:?}", stored_files);
            let stored = std::fs::read(&stored_files[0])
                .unwrap_or_else(|e| panic!("stored file readable: {}", e));
            assert_eq!(stored, file_data, "persisted bytes must match uploaded bytes exactly");

            let _ = std::fs::remove_dir_all(&upload_dir);
        }

        /// Plan 346 #12 (B6): every response carries X-Request-Id — minted
        /// (`req-<ms>-<n>`) when the client sends none.
        #[test]
        fn e2e_b6_request_id_generated_and_echoed() {
            let port = start_server(r#"
#[api(method = "GET", path = "/api/rid")]
fn rid() int {
    return 7
}
"#, 18741);
            let resp = http_get(port, "/api/rid");
            assert!(
                resp.contains("X-Request-Id: req-"),
                "minted request id header expected, got: {:?}",
                resp
            );
            // 404 responses carry it too.
            let not_found = http_get(port, "/api/nope");
            assert!(
                not_found.contains("X-Request-Id: req-"),
                "404 should carry request id, got: {:?}",
                not_found
            );
        }

        /// Plan 346 #12 (B6): an incoming X-Request-Id passes through verbatim
        /// (trace propagation).
        #[test]
        fn e2e_b6_request_id_incoming_passthrough() {
            let port = start_server(r#"
#[api(method = "GET", path = "/api/rid2")]
fn rid2() int {
    return 8
}
"#, 18742);
            let resp = http_get_with_headers(
                port,
                "/api/rid2",
                &[("X-Request-Id", "my-trace-42")],
            );
            assert!(
                resp.contains("X-Request-Id: my-trace-42"),
                "incoming request id should be echoed verbatim, got: {:?}",
                resp
            );
            assert!(
                !resp.contains("X-Request-Id: req-"),
                "minted id should not override the incoming one, got: {:?}",
                resp
            );
        }

        /// Plan 346 5e (B6): http.rate_limit(2, 60000) — the third request
        /// from the same IP gets 429 + Retry-After. start_server resets the
        /// limiter via clear_http_routes, so this test cannot poison others.
        #[test]
        fn e2e_zz_rate_limit_429_after_quota() {
            let port = start_server(r#"
http.rate_limit(2, 60000)

#[api(method = "GET", path = "/api/rl")]
fn rl() int {
    return 1
}
"#, 18743);
            let first = http_get(port, "/api/rl");
            let second = http_get(port, "/api/rl");
            assert!(first.starts_with("HTTP/1.1 200"), "first should pass, got: {:?}", first);
            assert!(second.starts_with("HTTP/1.1 200"), "second should pass, got: {:?}", second);
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
                third.contains("X-Request-Id: "),
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
            let port = start_server(r#"
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
"#, 18736);

            // GET /api/notes → JSON array of Note objects
            let resp_list = http_get(port, "/api/notes");
            let body_list = body_of(&resp_list);
            assert!(body_list.contains("\"title\": \"Welcome\""),
                "list: body={:?}", body_list);
            assert!(body_list.contains("\"title\": \"Shopping\""),
                "list: body={:?}", body_list);
            // Should be a JSON array: starts with [
            assert!(body_list.trim_start().starts_with('['),
                "list not array: body={:?}", body_list);

            // GET /api/notes/1 → single Note (Option.Some unwrapped)
            let resp_get = http_get(port, "/api/notes/1");
            let body_get = body_of(&resp_get);
            assert!(body_get.contains("\"id\": 1"),
                "get id=1: body={:?}", body_get);
            assert!(body_get.contains("\"title\": \"Shopping\""),
                "get title: body={:?}", body_get);
        }

        /// Plan 317 final validation: 015-notes backend pattern with List<Note>
        /// generic + module-level var + #[api] handler returning the list.
        /// This mirrors db.at's `var notes List<Note>` + `fn all_notes() []Note`.
        #[test]
        fn e2e_notes_list_generic() {
            let port = start_server(r#"
type Note { id int; title str; body str; time str }

var notes = [
    Note { id: 0, title: "Welcome", body: "first", time: "now" },
    Note { id: 1, title: "Shopping", body: "milk", time: "ago" },
]

#[api(method = "GET", path = "/api/notes")]
fn list_notes() []Note {
    return notes
}
"#, 18737);

            let resp = http_get(port, "/api/notes");
            let body = body_of(&resp);
            // List<Note> serialized as JSON array of Note objects.
            assert!(body.contains("\"title\": \"Welcome\""),
                "list generic frame 1: body={:?}", body);
            assert!(body.contains("\"title\": \"Shopping\""),
                "list generic frame 2: body={:?}", body);
            assert!(body.trim_start().starts_with('['),
                "list generic should be JSON array: body={:?}", body);
        }
    }
}

/// Run the HTTP server in blocking mode using std::net (MVP).
///
/// This is the current implementation — synchronous, serial request handling.
/// Each request is dispatched to a VM handler function via call_fn_by_name.
///
/// Future: replace with Axum for concurrency, SSE, TLS support.
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
            let resp = format!("HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n{}\r\n", cors_headers());
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
            if reader.read_line(&mut header).is_err() { break; }
            let header = header.trim();
            if header.is_empty() { break; }
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
        let result_json: Option<String> = if let Some(handler_task_arc) = vm.tasks.get(&handler_task_id) {
            let mut ht = handler_task_arc.blocking_lock();

            let mut n_args = 0;
            for (_param_name, param_val) in &route_match.path_params {
                // Plan 326 Phase 5: path params arrive as strings, but handlers
                // often declare them as `id int`. Try to parse as i32 first; if
                // it's a pure integer literal, inject as i32 so the handler
                // receives the right type. Non-numeric params stay strings.
                // (Long-term: codegen should record per-param types in api_routes
                //  so we can convert exactly. See plan §2 Phase 5.)
                if let Ok(i) = param_val.parse::<i32>() {
                    ht.ram.push_i32(i);
                } else {
                    let idx = {
                        let mut strings = vm.strings.write().unwrap();
                        let i = strings.len();
                        strings.push(param_val.as_bytes().to_vec());
                        i
                    };
                    ht.ram.push_nv(auto_val::encode_string(idx as u32));
                }
                n_args += 1;
            }
            if !body.is_empty() {
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(body.as_bytes().to_vec());
                    i
                };
                ht.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }

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
                                let next_result = if let Some(nt_arc) = vm.tasks.get(&next_task_id) {
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
                                        let s = vm.strings.read().unwrap()
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
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
            status, body_json.len(), cors_headers(), body_json
        );
        let _ = stream.write_all(response.as_bytes());
    }
}

/// Plan 317 Phase 4: Concurrent HTTP server using tokio async I/O.
///
/// Replaces the serial `serve_blocking_stdnet` for the Goroutine-style
/// concurrency model. The tokio runtime is `worker_threads(1)` (lib.rs:14),
/// so all `tokio::spawn` tasks run cooperatively on a single thread — this
/// matches Auto's Task model (single-thread, cooperative yield). &AutoVM is
/// safe to share because there is no cross-thread access.
///
/// Each accepted connection becomes a `tokio::spawn` task:
///   - JSON handlers: call_fn_by_name (synchronous), write response, done.
///   - SSE handlers: pull generator values, write a frame per value, and
///     `yield_now().await` after each frame so other connections' tasks get
///     scheduled. This gives interleaved streaming (connection A's frame,
///     connection B's frame, ...) without any single connection monopolizing
///     the single worker.
pub async fn serve_async(vm: &crate::vm::engine::AutoVM, addr: &str) {
    use tokio::net::TcpListener;

    // AutoVM is !Send and we need the spawned futures to be 'static (tokio
    // requirement). Encode the VM reference as a usize (which is 'static +
    // Send + Sync) and reconstruct &AutoVM inside each task. This is sound
    // because serve_async runs in a LocalSet on the VM-owning thread; all
    // spawned-local tasks run on that same thread.
    let vm_ptr = vm as *const crate::vm::engine::AutoVM as usize;

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[HTTP] Async server bind failed on {}: {}", addr, e);
            return;
        }
    };
    eprintln!("[HTTP] Async server listening on {} (concurrent, single-worker)", addr);
    eprintln!("[HTTP] Press Ctrl+C to shut down gracefully");

    let routes = get_routes();

    loop {
        let (mut stream, _peer) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[HTTP] Accept error: {}", e);
                continue;
            }
        };

        let routes_clone = routes.clone();
        let vp = vm_ptr; // usize is 'static + Copy
        tokio::task::spawn_local(async move {
            // SAFETY: LocalSet ensures single-thread execution. The VM lives
            // for the duration of serve_async (server = process lifetime).
            let vm: &crate::vm::engine::AutoVM = unsafe { &*(vp as *const _) };
            handle_connection_async(vm, &mut stream, &routes_clone).await;
        });
    }
}

/// Handle a single HTTP connection (async). Parses the request, dispatches to
/// the matched #[api] handler via call_fn_by_name, and writes the response.
/// SSE handlers interleave with other connections via yield_now.
async fn handle_connection_async(
    vm: &crate::vm::engine::AutoVM,
    stream: &mut tokio::net::TcpStream,
    routes: &[HttpRoute],
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // Read the request line + headers (raw bytes; minimal parser).
    let mut buf = vec![0u8; 8192];
    let n = match stream.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let raw = String::from_utf8_lossy(&buf[..n]).to_string();

    let mut lines = raw.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        let resp = format!("HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n{}\r\n", cors_headers());
        let _ = stream.write_all(resp.as_bytes()).await;
        return;
    }
    let req_method = parts[0].to_uppercase();
    let req_path = parts[1].to_string();

    // Plan 349 步骤 7/8 (W5): CORS preflight short-circuit — respond before parsing the
    // body since preflight requests have no body.
    if let Some(preflight) = handle_cors_preflight(&req_method) {
        let _ = stream.write_all(preflight.as_bytes()).await;
        return;
    }

    // Parse body (after blank line) if Content-Length present.
    let mut content_length = 0usize;
    let mut body = String::new();
    let mut header_done = false;
    let mut is_websocket = false;
    let mut content_type = String::new();
    let mut content_type_raw = String::new();
    let mut cookie_header = String::new();
    let mut auth_header = String::new();
    let mut incoming_request_id = String::new();
    for line in lines {
        if !header_done {
            if line.is_empty() {
                header_done = true;
                continue;
            }
            let lower = line.to_lowercase();
            if lower.starts_with("content-length:") {
                content_length = lower[15..].trim().parse().unwrap_or(0);
            }
            // Plan 346 stage 5: Request body size limit (default 10MB).
            const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;
            if content_length > MAX_BODY_SIZE {
                let resp = format!("HTTP/1.1 413 Payload Too Large\r\nContent-Type: application/json\r\nContent-Length: 27\r\nConnection: close\r\n{}\r\n{{\"error\":\"body too large\"}}", cors_headers());
                let _ = stream.write_all(resp.as_bytes()).await;
                eprintln!("[HTTP] {} {} → 413 (body {} > {})", req_method, req_path, content_length, MAX_BODY_SIZE);
                return;
            }
            if lower.starts_with("upgrade:") && lower.contains("websocket") {
                is_websocket = true;
            }
            if lower.starts_with("content-type:") {
                // Keep the lowercased copy for `contains` checks, plus the
                // original case — multipart boundaries are case-sensitive
                // (Plan 346 5a: a lowercased boundary never matched the body).
                content_type = lower[13..].trim().to_string();
                content_type_raw = line[13..].trim().to_string();
            }
            // Plan 346 stage 4: Parse Cookie and Authorization headers.
            if lower.starts_with("cookie:") {
                cookie_header = line[7..].trim().to_string();
            }
            if lower.starts_with("authorization:") {
                auth_header = line[14..].trim().to_string();
            }
            // Plan 346 #12 (B6): honor an incoming X-Request-Id (trace
            // propagation); otherwise mint one below.
            if lower.starts_with("x-request-id:") {
                incoming_request_id = line[14..].trim().to_string();
            }
        } else if body.len() < content_length {
            body.push_str(line);
        }
    }

    // Plan 346 B6: resolve this request's id (incoming value wins, else mint).
    // Present on every response as X-Request-Id and in the middleware
    // request-info JSON, so logs/middleware/handler share one trace id.
    let request_id = if incoming_request_id.is_empty() {
        gen_request_id()
    } else {
        incoming_request_id
    };

    // Plan 346 5e (B6): per-IP fixed-window rate limit — checked after the
    // CORS preflight short-circuit (preflights stay free) but before the
    // middleware chain and route matching. 429 carries Retry-After + the
    // request id.
    let client_ip = stream
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    if let Some(retry_after_ms) = rate_limit_take(&client_ip) {
        let body = format!("{{\"error\":\"rate limit exceeded\",\"retry_after_ms\":{}}}", retry_after_ms);
        let resp = format!(
            "HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nRetry-After: {}\r\nContent-Length: {}\r\nConnection: close\r\n{}{}\r\n{}",
            (retry_after_ms + 999) / 1000,
            body.len(),
            cors_headers(),
            request_id_header(&request_id),
            body
        );
        let _ = stream.write_all(resp.as_bytes()).await;
        eprintln!("[HTTP] {} {} [{}] → 429 rate limited (ip {})", req_method, req_path, request_id, client_ip);
        return;
    }

    // Plan 346 5a (B6): multipart/form-data — the body is BINARY (file
    // bytes) and may exceed the initial 8KB read, so acquire it at the byte
    // level (split at \r\n\r\n, then read until Content-Length satisfied).
    // The legacy lossy-UTF-8 `body` path stays untouched for other types.
    let mut multipart_json: Option<String> = None;
    if content_type.starts_with("multipart/form-data") {
        let boundary = content_type_raw
            .split(';')
            .find_map(|p| p.trim().strip_prefix("boundary="))
            .map(|b| b.trim_matches('"').to_string());
        if let Some(boundary) = boundary {
            let header_split = find_sub(&buf[..n], b"\r\n\r\n")
                .map(|p| p + 4)
                .or_else(|| find_sub(&buf[..n], b"\n\n").map(|p| p + 2));
            if let Some(split) = header_split {
                let mut body_bytes = buf[split..n].to_vec();
                while body_bytes.len() < content_length {
                    let mut chunk = vec![0u8; (content_length - body_bytes.len()).min(8192)];
                    match stream.read(&mut chunk).await {
                        Ok(r) if r > 0 => body_bytes.extend_from_slice(&chunk[..r]),
                        _ => break,
                    }
                }
                let parts = parse_multipart(&body_bytes, &boundary);
                multipart_json = Some(multipart_to_handler_json(parts));
                eprintln!(
                    "[HTTP] {} {} [{}] multipart: {} bytes parsed",
                    req_method, req_path, request_id, body_bytes.len()
                );
            }
        } else {
            eprintln!("[HTTP] {} {} [{}] multipart without boundary — ignored", req_method, req_path, request_id);
        }
    }

    // Route match
    let route_match = match match_route(routes, &req_method, &req_path) {
        Some(rm) => rm,
        None => {
            let resp = format!("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n{}{}\r\n", cors_headers(), request_id_header(&request_id));
            let _ = stream.write_all(resp.as_bytes()).await;
            return;
        }
    };

    // Plan 350: WebSocket upgrade handling.
    if is_websocket {
        // We need to do the WebSocket handshake using tungstenite. Since we've
        // already read the HTTP request into `raw`, we need to convert the
        // tokio TcpStream into a tungstenite WebSocket. The simplest approach:
        // write the raw request back into the stream (tungstenite reads it),
        // but actually tungstenite's server::accept expects a stream where the
        // client's upgrade request hasn't been consumed yet. Since we already
        // consumed it, we need to manually do the handshake.
        //
        // Alternative: use tungstenite's handshake manually. We extract the
        // Sec-WebSocket-Key from the raw request, compute the accept value,
        // write the response, then wrap the stream.
        let ws_key = raw.lines()
            .find(|l| l.to_lowercase().starts_with("sec-websocket-key:"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim().to_string());

        if let Some(key) = ws_key {
            // Compute Sec-WebSocket-Accept: base64(sha1(key + magic_guid))
            // Compute Sec-WebSocket-Accept: base64(sha1(key + magic_guid))
            use sha1::Digest;
            let mut hasher = sha1::Sha1::new();
            hasher.update(key.as_bytes());
            hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
            let hash = hasher.finalize();
            use base64::Engine;
            let accept = base64::engine::general_purpose::STANDARD.encode(&hash);

            let response = format!(
                "HTTP/1.1 101 Switching Protocols\r\n\
                 Upgrade: websocket\r\n\
                 Connection: Upgrade\r\n\
                 Sec-WebSocket-Accept: {}\r\n\r\n",
                accept
            );
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.flush().await;

            // Now the TCP stream is a raw WebSocket. We use tokio's AsyncRead
            // to read raw WebSocket frames and echo text messages back.
            // This avoids the ownership issue of converting &mut TcpStream.
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
                    if stream.read_exact(&mut ext).await.is_err() { break; }
                    u16::from_be_bytes(ext) as usize
                } else if payload_len == 127 {
                    let mut ext = [0u8; 8];
                    if stream.read_exact(&mut ext).await.is_err() { break; }
                    u64::from_be_bytes(ext) as usize
                } else {
                    payload_len
                };

                // Masking key (4 bytes if masked).
                let mut mask_key = [0u8; 4];
                if masked {
                    if stream.read_exact(&mut mask_key).await.is_err() { break; }
                }

                // Payload.
                let mut payload = vec![0u8; actual_len];
                if stream.read_exact(&mut payload).await.is_err() { break; }
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
                        let resp = encode_ws_text_frame(&text);
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
        return;
    }

    // Plan 352: Execute middleware chain before handler.
    // Each middleware is a VM fn that receives a request-info JSON string.
    // If it returns a non-empty/non-null value, that becomes the response
    // (short-circuit). If it returns nil/empty, the handler runs normally.
    // Plan 346 #12 (B6): request_id is part of the request-info payload so
    // middleware (logging/auth) can correlate with the X-Request-Id header.
    let request_info = format!(
        r#"{{"method":"{}","path":"{}","content_type":"{}","has_body":{},"request_id":"{}"}}"#,
        req_method, req_path, content_type, !body.is_empty(), request_id
    );
    let middleware_names: Vec<String> = crate::vm::ffi::stdlib::MIDDLEWARE_CHAIN
        .lock().map(|c| c.clone()).unwrap_or_default();
    let mut middleware_response: Option<String> = None;
    for mw_fn in &middleware_names {
        let mw_task_id = vm.spawn_task(0, 65536);
        // Push request info as arg.
        if let Some(t_arc) = vm.tasks.get(&mw_task_id) {
            if let Ok(mut t) = t_arc.try_lock() {
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(request_info.as_bytes().to_vec());
                    i
                };
                t.ram.push_nv(auto_val::encode_string(idx as u32));
            }
        }
        let mw_result = if let Some(t_arc) = vm.tasks.get(&mw_task_id) {
            let mut t = match t_arc.try_lock() { Ok(t) => t, Err(_) => break };
            match vm.call_fn_by_name(&mut t, mw_fn, 1) {
                Ok(()) => {
                    let nv = t.ram.pop_nv();
                    if auto_val::is_null(nv) { None }
                    else { nv_to_json(vm, nv, 0) }
                }
                Err(_) => None,
            }
        } else { None };
        vm.tasks.remove(&mw_task_id);
        if let Some(ref resp) = mw_result {
            if !resp.is_empty() && resp != "null" {
                middleware_response = Some(resp.clone());
                break;
            }
        }
    }
    if let Some(ref mw_resp) = middleware_response {
        // Middleware short-circuited — return its response directly.
        eprintln!("[HTTP] {} {} [{}] → MW ({}ms)", req_method, req_path, request_id, 0);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}{}\r\n{}",
            mw_resp.len(), cors_headers(), request_id_header(&request_id), mw_resp
        );
        let _ = stream.write_all(response.as_bytes()).await;
        return;
    }

    // Dispatch to handler via call_fn_by_name (synchronous VM execution).
    // Plan 346 stage 2: Request logging + error handling.
    let request_start = std::time::Instant::now();
    let handler_task_id = vm.spawn_task(0, 65536);
    let n_args = build_handler_args(vm, handler_task_id, &route_match, &body, &content_type, &cookie_header, &auth_header, multipart_json.as_deref());

    let result_json = if let Some(_task_arc) = vm.tasks.get(&handler_task_id) {
        let mut ht = match _task_arc.try_lock() {
            Ok(t) => t,
            Err(_) => {
                eprintln!("[HTTP] {} {} → 500 (task lock failed, {}ms)",
                    req_method, req_path, request_start.elapsed().as_millis());
                vm.tasks.remove(&handler_task_id);
                let resp = format!("HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: 27\r\nConnection: close\r\n{}\r\n{{\"error\":\"internal error\"}}", cors_headers());
                let _ = stream.write_all(resp.as_bytes()).await;
                return;
            }
        };
        match vm.call_fn_by_name(&mut ht, &route_match.fn_name, n_args) {
            Ok(()) => {
                let nv = ht.ram.pop_nv();
                // SSE detection: generator/iterator return → stream frames.
                if auto_val::is_i32(nv) {
                    let iter_id = auto_val::decode_i32(nv) as u32;
                    if vm.iterators.contains_key(&iter_id) {
                        eprintln!("[HTTP] {} {} → 200 SSE ({}ms)",
                            req_method, req_path, request_start.elapsed().as_millis());
                        drop(ht);
                        let sse_header = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n{}\r\n", cors_headers());
                        let _ = stream.write_all(sse_header.as_bytes()).await;
                        let _ = stream.flush().await;
                        // Pull generator values and write SSE frames. After each
                        // frame, yield_now so other connections get scheduled
                        // (Goroutine-style cooperative concurrency on the single
                        // worker thread).
                        loop {
                            let next_task_id = vm.spawn_task(0, 1024);
                            let next_val = if let Some(nt_arc) = vm.tasks.get(&next_task_id) {
                                let mut nt = nt_arc.try_lock().unwrap();
                                nt.ram.push_i32(iter_id as i32);
                                let _ = crate::vm::native::shim_iterator_next(&mut nt, vm);
                                nt.ram.pop_i32()
                            } else { -1 };
                            vm.tasks.remove(&next_task_id);
                            if next_val == -1 { break; }
                            let frame = format!("data: {}\n\n", next_val);
                            let _ = stream.write_all(frame.as_bytes()).await;
                            let _ = stream.flush().await;
                            // Cooperative yield: let other connections' tasks run.
                            tokio::task::yield_now().await;
                        }
                        None
                    } else if let Some(res) =
                        crate::vm::ffi::stdlib::lookup_http_response(iter_id as u64)
                    {
                        // Plan 346 3c: response-object return — the handler built a
                        // Response (e.g. http.response_redirect(url, 302)) and
                        // returned its handle; serve status/headers/body directly.
                        // NOTE: do NOT vm.tasks.remove() here — we are inside the
                        // `vm.tasks.get()` DashMap read-guard scope; removing the
                        // same shard now self-deadlocks. Return None and let the
                        // shared cleanup below (outside the scope) remove the task.
                        write_http_response_object(
                            stream,
                            res,
                            &req_method,
                            &req_path,
                            request_start.elapsed().as_millis(),
                            &request_id,
                        )
                        .await;
                        drop(ht);
                        None
                    } else {
                        nv_to_json(vm, nv, 0)
                    }
                } else if auto_val::is_i64(nv) {
                    // Plan 346 3c: response handles built via http.response()/
                    // response_status()/... are pushed as i64 (Plan 377 inline
                    // tag 8). Serve them as response objects, not JSON ints.
                    let handle = auto_val::decode_i64(nv) as u64;
                    match crate::vm::ffi::stdlib::lookup_http_response(handle) {
                        Some(res) => {
                            // Same DashMap-guard caveat as the i32 branch above.
                            write_http_response_object(
                                stream,
                                res,
                                &req_method,
                                &req_path,
                                request_start.elapsed().as_millis(),
                                &request_id,
                            )
                            .await;
                            drop(ht);
                            None
                        }
                        None => nv_to_json(vm, nv, 0),
                    }
                } else {
                    nv_to_json(vm, nv, 0)
                }
            }
            Err(e) => {
                eprintln!("[HTTP] {} {} → 500 (handler '{}' error: {:?}, {}ms)",
                    req_method, req_path, route_match.fn_name, e, request_start.elapsed().as_millis());
                // Plan 346 stage 2: Return a proper 500 JSON error response
                // instead of silently dropping the connection.
                let error_json = format!(
                    r#"{{"error":"internal server error","detail":"{}"}}"#,
                    format!("{:?}", e).replace('"', "\\\"").replace('\n', " ")
                );
                Some(error_json)
            }
        }
    } else {
        None
    };

    vm.tasks.remove(&handler_task_id);

    // Non-SSE: write JSON response.
    if let Some(result_json) = result_json {
        // Plan 346 stage 2: Determine status code from response content.
        let is_error = result_json.starts_with("{\"error\":");
        let status = if is_error { "500 Internal Server Error" } else { "200 OK" };
        // Log successful request (non-SSE, non-error already logged above).
        if !is_error {
            eprintln!("[HTTP] {} {} [{}] → 200 ({}ms)",
                req_method, req_path, request_id, request_start.elapsed().as_millis());
        }
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}{}\r\n{}",
            status, result_json.len(), cors_headers(), request_id_header(&request_id), result_json
        );
        let _ = stream.write_all(response.as_bytes()).await;
    }
}

/// Encode a text message as a WebSocket frame (server→client, unmasked).
fn encode_ws_text_frame(text: &str) -> Vec<u8> {
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

/// Build handler arguments on the task's stack (path params + body).
/// Returns the number of args pushed.
fn build_handler_args(
    vm: &crate::vm::engine::AutoVM,
    task_id: u64,
    route_match: &RouteMatch,
    body: &str,
    content_type: &str,
    cookie_header: &str,
    auth_header: &str,
    multipart_json: Option<&str>,
) -> usize {
    let mut n_args = 0;
    if let Some(_task_arc) = vm.tasks.get(&task_id) {
        if let Ok(mut task) = _task_arc.try_lock() {
            // Push path params (existing behavior).
            for (_param_name, param_val) in &route_match.path_params {
                if let Ok(i) = param_val.parse::<i32>() {
                    task.ram.push_i32(i);
                } else {
                    let idx = {
                        let mut strings = vm.strings.write().unwrap();
                        let i = strings.len();
                        strings.push(param_val.as_bytes().to_vec());
                        i
                    };
                    task.ram.push_nv(auto_val::encode_string(idx as u32));
                }
                n_args += 1;
            }

            // Plan 346: Push query params as a JSON object string if no body.
            if !route_match.query_params.is_empty() && body.is_empty() {
                let json_parts: Vec<String> = route_match.query_params.iter()
                    .map(|(k, v)| format!("\"{}\":\"{}\"", k.replace('"', "\\\""), v.replace('"', "\\\"")))
                    .collect();
                let json_str = format!("{{{}}}", json_parts.join(","));
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(json_str.into_bytes());
                    i
                };
                task.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }

            // Plan 346: Push body.
            if let Some(mp) = multipart_json {
                // Plan 346 5a (B6): multipart push — fields + persisted-file
                // metadata as JSON (takes the body arg slot).
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(mp.as_bytes().to_vec());
                    i
                };
                task.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            } else if !body.is_empty() {
                let body_to_push = if content_type.contains("application/x-www-form-urlencoded") {
                    let pairs: Vec<String> = body.split('&')
                        .filter_map(|pair| {
                            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
                            Some(format!("\"{}\":\"{}\"", url_decode(k).replace('"', "\\\""), url_decode(v).replace('"', "\\\"")))
                        })
                        .collect();
                    format!("{{{}}}", pairs.join(","))
                } else {
                    body.to_string()
                };
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(body_to_push.into_bytes());
                    i
                };
                task.ram.push_nv(auto_val::encode_string(idx as u32));
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
                None => true, // unknown — preserve old behavior (defensive)
            };
            if push_meta {
                let cookies_json: String = if cookie_header.is_empty() {
                    "{}".to_string()
                } else {
                    let pairs: Vec<String> = cookie_header.split(';')
                        .filter_map(|pair| {
                            let pair = pair.trim();
                            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
                            Some(format!("\"{}\":\"{}\"", k.trim().replace('"', "\\\""), v.trim().replace('"', "\\\"")))
                        })
                        .collect();
                    format!("{{{}}}", pairs.join(","))
                };
                let auth_val = if auth_header.is_empty() { "".to_string() } else { auth_header.replace('"', "\\\"") };
                let meta_json = format!(r#"{{"cookies":{},"auth":"{}"}}"#, cookies_json, auth_val);
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(meta_json.into_bytes());
                    i
                };
                task.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }
        }
    }
    n_args
}
