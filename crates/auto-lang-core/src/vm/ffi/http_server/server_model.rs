//! PLAN-843 结构拆分：HTTP 服务 model 职责（原样搬移）。
use super::*;

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

/// Format an f64 as JSON (integers without trailing .0, per JSON convention the
/// number is still valid; we keep the natural Rust representation).
pub(super) fn format_f64_json(f: f64) -> String {
    if f.is_nan() || f.is_infinite() {
        "null".to_string()
    } else if f.fract() == 0.0 && f.abs() < 1e16 {
        format!("{}", f as i64)
    } else {
        format!("{}", f)
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
pub fn push_str_arg(vm: &crate::vm::engine::AutoVM, task: &mut crate::vm::task::AutoTask, s: &str) {
    vm.intern_runtime_str(task, s.as_bytes().to_vec());
}

/// Owned, `Send` request handed from the HTTP network layer to the VM owner
/// thread. Protocol objects (hyper types, sockets) never cross the bridge;
/// the owner re-runs the same routing/binding/middleware/response semantics
/// as the legacy inline path (AC-02: no `Rc<AutoVM>`, no raw pointers).
pub struct ApiRequest {
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
    pub(super) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Reply body payload: plain bytes, SSE frames streamed from a producer
/// task on the VM owner thread, or a file response descriptor (PLAN-729).
pub enum ApiBody {
    Text(Vec<u8>),
    Sse(tokio::sync::mpsc::Receiver<String>),
    /// PLAN-729 T-04：文件响应描述符（编组单次取出；打开/协议决策/发送
    /// 在 transport 侧的共享宿主服务，不在 VM owner）。status/headers 由
    /// 宿主 serve 决定，`Full` 携带的占位被 File 臂忽略。
    File(FileReplySeed),
}

/// [`ApiBody::File`] 载荷：owned 描述符 + 请求 id（日志/CORS 追加用）。
pub struct FileReplySeed {
    pub descriptor: a2r_std::http::FileResponse,
    pub request_id: String,
}

/// Structured reply produced by the VM owner for one request.
pub enum ApiReply {
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

/// CORS + JSON content-type + 请求 id（task-lock 失败臂用）。
pub(super) fn cors_json_headers(request_id: &str) -> Vec<(String, String)> {
    let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
    headers.extend(cors_header_pairs());
    headers.push(("X-Request-Id".to_string(), request_id.to_string()));
    headers
}

/// PLAN-776 T-04：终值 Value → NanoValue 完整值桥（sync/async 同形）。
/// - I64 全宽（48 位内联 / BigInt 堆装箱——`encode_i64_with_heap`）；
/// - VmRef 身份保持（资源凭据/普通对象——TAG_OBJECT 回编组侧）；
/// - Array/Obj 物化堆容器（id 记入 `materialized`，编组后统一释放——
///   nv_to_json/value_to_json 现成序列化 record/list）；
/// - 不可转换变体 → None（调用方 500 诊断——不再静默 null）。
pub(super) fn value_to_nv_via_task(
    vm: &std::rc::Rc<AutoVM>,
    ctx: &DispatchCtx,
    value: auto_val::Value,
    materialized: &mut Vec<u64>,
) -> Option<auto_val::NanoValue> {
    let task_id = ctx.handler_task_id?;
    let t_arc = vm.tasks.get(&task_id)?.clone();
    let mut t = t_arc.try_lock().ok()?;
    let nv = match value {
        auto_val::Value::Int(i) => auto_val::encode_i32(i),
        auto_val::Value::I64(i) => crate::vm::ffi::encode_i64_with_heap(vm, i),
        auto_val::Value::Float(f) => auto_val::encode_f64(f),
        auto_val::Value::Double(f) => auto_val::encode_f64(f),
        auto_val::Value::Bool(b) => auto_val::encode_bool(b),
        auto_val::Value::Nil | auto_val::Value::Null => auto_val::encode_null(),
        auto_val::Value::Str(s) => {
            let idx = vm.add_string(s.to_string().into_bytes());
            vm.rc_push_str_idx(&mut t, idx);
            t.ram.pop_nv()
        }
        auto_val::Value::VmRef(r) => auto_val::encode_object(r.id as u32),
        // PLAN-776 R8-01：Future 句柄 Value 载体还原 TAG_FUTURE 载荷。
        auto_val::Value::Future(f) => auto_val::encode_future(f.id as u32),
        auto_val::Value::Array(a) => {
            let list = crate::vm::types::ListData {
                elems: a.values,
                storage: None,
            };
            let id = vm.insert_heap_object(list);
            materialized.push(id);
            auto_val::encode_object(id as u32)
        }
        auto_val::Value::Obj(o) => {
            let mut od = crate::vm::types::ObjectData::new();
            for (k, v) in o.iter() {
                od.fields.insert(k.clone(), v.clone());
            }
            let id = vm.insert_heap_object(od);
            materialized.push(id);
            auto_val::encode_object(id as u32)
        }
        _ => return None,
    };
    Some(nv)
}

/// Marshal a returned NanoValue into the final JSON reply arm: PLAN-698 POST
/// broadcast + 200/500 mapping by the legacy `{"error":` prefix convention +
/// the success log line.
pub(super) fn json_value_reply(
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
