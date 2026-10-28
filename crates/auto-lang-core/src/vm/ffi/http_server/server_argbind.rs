//! PLAN-843 结构拆分：HTTP 服务 argbind 职责（原样搬移）。
use super::*;

/// R1-F01：ctx 声明流返回（`~Iter<T>`/`~Stream<T>`——`fn_response_kind ==
/// Stream`，含 axum closure 反查）。SSE 的 int 路径唯一合法形态：generator
/// ctx 声明文件返回（axum_route closure 反查同 async 门）。
pub(super) fn handler_declares_file_return(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx) -> bool {
    match ctx.axum_route {
        Some(ref r) => crate::vm::ffi::axum_adapter::export_name_for_closure(vm, r.closure_id)
            .map(|n| fn_is_api_file_return(&n))
            .unwrap_or(false),
        None => fn_is_api_file_return(&ctx.route.fn_name),
    }
}

/// ctx 声明上传收据返回（axum_route closure 反查同门）。
pub(super) fn handler_declares_upload_return(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx) -> bool {
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
    let mut headers = cors_json_headers(&ctx.request_id);
    headers[0].1 = "application/json".to_string();
    ApiReply::Full {
        status: receipt.status,
        headers,
        body: ApiBody::Text(receipt.json.into_bytes()),
    }
}

/// 声明上传收据返回但值不是登记收据 → 500 诊断（不 JSON 200）。
pub(super) fn upload_contract_mismatch_reply(ctx: &DispatchCtx, raw: u64) -> ApiReply {
    eprintln!(
        "[HTTP] {} {} [{}] → 500 (handler '{}' declares UploadReceipt but returned non-receipt value {})",
        ctx.req_method, ctx.req_path, ctx.request_id, ctx.route.fn_name, raw
    );
    let mut headers = cors_json_headers(&ctx.request_id);
    headers[0].1 = "application/json".to_string();
    ApiReply::Full {
        status: 500,
        headers,
        body: ApiBody::Text(br#"{"error":"handler did not return an UploadReceipt"}"#.to_vec()),
    }
}

/// 声明文件返回但值不是登记描述符 → 500 诊断（不 JSON 200；决策报告 §6）。
pub(super) fn file_contract_mismatch_reply(ctx: &DispatchCtx, raw: u64) -> ApiReply {
    eprintln!(
        "[HTTP] {} {} [{}] → 500 (handler '{}' declares FileResponse but returned non-descriptor value {})",
        ctx.req_method, ctx.req_path, ctx.request_id, ctx.route.fn_name, raw
    );
    let mut headers = cors_json_headers(&ctx.request_id);
    let body = br#"{"error":"handler did not return a FileResponse descriptor"}"#.to_vec();
    headers[0].1 = "application/json".to_string();
    ApiReply::Full {
        status: 500,
        headers,
        body: ApiBody::Text(body),
    }
}

/// PLAN-776 T-03：SSE 迭代器 → wire 回复的**单源**（i32 查表命中制臂与
/// 资源凭据臂共用；不新增另一套 SSE 输出循环——producer spawn/
/// `produce_sse_frames`/cleanup 守卫原样）。
fn sse_iterator_reply(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx, iter_id: u32) -> ApiReply {
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
    ApiReply::Full {
        status: 200,
        headers: vec![
            ("Content-Type".to_string(), "text/event-stream".to_string()),
            ("Cache-Control".to_string(), "no-cache".to_string()),
            ("Connection".to_string(), "keep-alive".to_string()),
        ],
        body: ApiBody::Sse(receiver),
    }
}

/// PLAN-776 T-03：资源凭据 → owned 回复（sync 终值与 async 终值共用分派）。
/// Response clone 快照（重复返回不消费）；Sse 校验注册表后走单源 serve；
/// File/Upload take 单次移交（未登记 → mismatch 500，原 729/730 诊断体例）。
pub(super) fn resource_claim_reply(
    vm: &std::rc::Rc<AutoVM>,
    ctx: &DispatchCtx,
    claim: crate::vm::ffi::http_return_resource::ResourceClaim,
) -> ApiReply {
    use crate::vm::ffi::http_return_resource::ResourceClaim;
    match claim {
        ResourceClaim::Response(snapshot) => {
            response_handle_reply(ctx, (snapshot.status, snapshot.headers, snapshot.body))
        }
        ResourceClaim::Sse { iterator_id } => {
            if vm.iterators.contains_key(&iterator_id) {
                sse_iterator_reply(vm, ctx, iterator_id)
            } else {
                eprintln!(
                    "[HTTP] {} {} [{}] → 500 (sse credential references missing iterator {})",
                    ctx.req_method, ctx.req_path, ctx.request_id, iterator_id
                );
                ApiReply::Full {
                    status: 500,
                    headers: cors_json_headers(&ctx.request_id),
                    body: ApiBody::Text(
                        br#"{"error":"sse credential references a missing iterator"}"#.to_vec(),
                    ),
                }
            }
        }
        ResourceClaim::File { descriptor_id } => {
            match super::super::http_server_file::take_file_response(descriptor_id) {
                Some(descriptor) => ApiReply::Full {
                    status: 200, // 占位：File 臂由宿主 serve 决定
                    headers: Vec::new(),
                    body: ApiBody::File(FileReplySeed {
                        descriptor,
                        request_id: ctx.request_id.clone(),
                    }),
                },
                None => file_contract_mismatch_reply(ctx, descriptor_id),
            }
        }
        ResourceClaim::Upload { receipt_id } => {
            match super::super::http_upload::take_upload_receipt(receipt_id) {
                Some(receipt) => upload_receipt_reply(ctx, receipt),
                None => upload_contract_mismatch_reply(ctx, receipt_id),
            }
        }
    }
}

/// PLAN-669 binder failure — site maps to an HTTP error response.
#[derive(Debug)]
pub enum ApiArgBindError {
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
pub(super) fn validate_body_value(declared_ty: &str, v: &serde_json::Value) -> Result<(), String> {
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
                let inner = declared_ty
                    .trim()
                    .trim_start_matches('?')
                    .trim_end_matches('?');
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

pub fn push_typed_string_arg(
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
                task.ram
                    .push_nv(crate::vm::ffi::convert::encode_i64_with_heap(vm, i));
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

fn is_meta_param_name(name: &str) -> bool {
    META_PARAM_NAMES.contains(&name.trim().to_lowercase().as_str())
}

/// Build handler arguments on the task's stack (path params + body).
/// Returns the number of args pushed.
/// PLAN-669: cookies + auth metadata JSON (the Plan 346 stage 4 payload).
/// PLAN-734 T-03：meta 构造单源（standalone 与 back-proxy 共用）。
pub fn build_meta_json(cookie_header: &str, auth_header: &str) -> String {
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

pub(super) fn build_handler_args(
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
