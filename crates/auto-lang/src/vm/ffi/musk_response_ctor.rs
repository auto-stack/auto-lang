//! Plan 442 C2: VM-side implementations of the auto-musk backend's extern
//! response-constructor family.
//!
//! The backend `.at` handlers build HTTP responses through `extern_sigs`
//! glue functions (`ok_response(v)`, `err_response(msg, code)`,
//! `json_response(d)`, `error_response(code, d)`, `text_response(msg, code)`,
//! `empty_response(code)`, `err_json_response(d, code)`, `to_response(v, msg,
//! code)`, plus the `resp_*` inspection helpers and the SSE event builders
//! `sse_named_event`/`sse_event`/`sse_plain_event`). In the a2r build these
//! are implemented in `extern_impl.rs`; on the VM they previously executed the
//! empty `extern_sigs` bodies and always returned null, so every handler
//! answered `200 null`.
//!
//! These shims give those names real runtime semantics:
//! - the response constructors register an `HttpResponseData` (via
//!   `stdlib::insert_http_response`) and push its handle, which the server's
//!   response-object path (`lookup_http_response`) serves as status/headers/
//!   body — no server-side change needed;
//! - the `resp_*` helpers inspect a `Value` for the
//!   `{"error":{"code","message"}}` envelope the externs return;
//! - the SSE event builders construct an opaque `axum::response::sse::Event`
//!   heap object (consumed by the SSE streaming path, Plan 442 C2 item ②).
//!
//! Registration: names are resolved by `BIGVM_NATIVES.resolve_qualified`
//! (codegen's `native_id` decision) and bound to these shims in
//! `stdlib::register_stdlib_ffi`.

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;

/// Build + register a structured HTTP response and push its handle as i32.
/// Mirrors `shim_http_response_redirect`'s convention: the server's
/// `is_i32 → lookup_http_response` path serves it directly.
fn push_response(
    task: &mut AutoTask,
    vm: &AutoVM,
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> Result<(), VMError> {
    let handle = crate::vm::ffi::stdlib::insert_http_response(status, headers, body);
    // `_stake` keeps the RC contract balanced if the caller later reads it.
    let _ = vm;
    task.ram.push_i32(handle as i32);
    Ok(())
}

/// JSON body for a successful response: the argument serialized via the
/// handler-return JSON writer, degrading to `null` on failure.
fn nv_to_json_or_null(vm: &AutoVM, nv: auto_val::NanoValue) -> String {
    if is_nullish(nv) {
        return "null".to_string();
    }
    crate::vm::ffi::http_server::nv_to_json(vm, nv, 0).unwrap_or_else(|| "null".to_string())
}

/// A "no value" NanoValue. The `null`/`nil` literal compiles to `CONST_I32 -1`
/// (codegen `Expr::Null` → `CONST_I32 -1`), while the JSON/`Value` producer
/// path yields a tagged nanbox null — so a nullish check must accept both.
fn is_nullish(nv: auto_val::NanoValue) -> bool {
    auto_val::is_null(nv)
        || (auto_val::is_i32(nv) && auto_val::decode_i32(nv) == -1)
}

/// Pop an i32 argument.
fn pop_i32(task: &mut AutoTask) -> i32 {
    crate::vm::native::pop_arg_i32(task)
}

/// Pop a raw NanoValue argument WITH its RC stake — the caller must keep the
/// returned guard alive for the whole read (PLAN-044 fix: the old form scoped
/// the stake inside pop_value, so a last-ref heap object (fresh struct
/// literal) was freed before the caller serialized it — musk health() →
/// `Json(StatusOk(..))` hit the canary at rc.rs:503).
fn pop_value_staked<'v>(
    task: &mut AutoTask,
    vm: &'v AutoVM,
) -> (auto_val::NanoValue, crate::vm::native::StakeGuard<'v>) {
    let nv = crate::vm::native::pop_arg_nv(task);
    let stake = crate::vm::native::StakeGuard::nv(vm, nv);
    (nv, stake)
}

/// Pop a string argument (pool-indexed scalar string).
fn pop_string(task: &mut AutoTask, vm: &AutoVM, ctx: &str) -> Result<String, VMError> {
    let nv = crate::vm::native::pop_arg_nv(task);
    if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv) as usize;
        vm.strings
            .read()
            .unwrap()
            .get(idx)
            .cloned()
            .map(|b| String::from_utf8_lossy(&b).to_string())
            .ok_or_else(|| VMError::RuntimeError(format!("{ctx}: bad string index")))
    } else {
        Err(VMError::RuntimeError(format!("{ctx}: expected string")))
    }
}

/// `ok_response(v)` → 200 JSON of `v`.
/// Stack: v -> response_handle
pub fn shim_ok_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let body = nv_to_json_or_null(vm, nv).into_bytes();
    push_response(
        task,
        vm,
        200,
        vec![("Content-Type".to_string(), "application/json".to_string())],
        body,
    )
}

/// `json_response(d)` → 200 JSON of `d` (server_serve settings_link form).
/// Stack: d -> response_handle
pub fn shim_json_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let body = nv_to_json_or_null(vm, nv).into_bytes();
    push_response(
        task,
        vm,
        200,
        vec![("Content-Type".to_string(), "application/json".to_string())],
        body,
    )
}

/// `error_response(code, d)` → `code` JSON of `d` (server_serve form).
/// Stack: code, d -> response_handle (d on top)
pub fn shim_error_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let status = pop_i32(task) as u16;
    let body = nv_to_json_or_null(vm, nv).into_bytes();
    push_response(
        task,
        vm,
        status,
        vec![("Content-Type".to_string(), "application/json".to_string())],
        body,
    )
}

/// `err_response(msg, code)` → `code` with `{"error":"<msg>"}`.
/// Stack: msg, code -> response_handle (code on top)
pub fn shim_err_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let status = pop_i32(task) as u16;
    let msg = pop_string(task, vm, "err_response")?;
    let body = format!(r#"{{"error":{}}}"#, serde_json::to_string(&msg).unwrap_or_default())
        .into_bytes();
    push_response(
        task,
        vm,
        status,
        vec![("Content-Type".to_string(), "application/json".to_string())],
        body,
    )
}

/// `err_json_response(d, code)` → `code` JSON of `d` (settings_link error form).
/// Stack: d, code -> response_handle (code on top)
pub fn shim_err_json_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let status = pop_i32(task) as u16;
    let (nv, _stake) = pop_value_staked(task, vm);
    let body = nv_to_json_or_null(vm, nv).into_bytes();
    push_response(
        task,
        vm,
        status,
        vec![("Content-Type".to_string(), "application/json".to_string())],
        body,
    )
}

/// `text_response(msg, code)` → `code` text/plain of `msg`.
/// Stack: msg, code -> response_handle (code on top)
pub fn shim_text_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let status = pop_i32(task) as u16;
    let msg = pop_string(task, vm, "text_response")?;
    push_response(
        task,
        vm,
        status,
        vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
        msg.into_bytes(),
    )
}

/// `empty_response(code)` → `code` empty body.
/// Stack: code -> response_handle
pub fn shim_empty_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let status = pop_i32(task) as u16;
    push_response(task, vm, status, Vec::new(), Vec::new())
}

/// `to_response(v, msg, code)` → null `v` yields `err_response(msg, code)`,
/// otherwise `ok_response(v)`. Mirrors `extern_impl::to_response`.
/// Stack: v, msg, code -> response_handle (code on top)
pub fn shim_to_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let status = pop_i32(task) as u16;
    let msg = pop_string(task, vm, "to_response")?;
    let (nv, _stake) = pop_value_staked(task, vm);
    if is_nullish(nv) {
        let body = format!(r#"{{"error":{}}}"#, serde_json::to_string(&msg).unwrap_or_default())
            .into_bytes();
        push_response(
            task,
            vm,
            status,
            vec![("Content-Type".to_string(), "application/json".to_string())],
            body,
        )
    } else {
        let body = nv_to_json_or_null(vm, nv).into_bytes();
        push_response(
            task,
            vm,
            200,
            vec![("Content-Type".to_string(), "application/json".to_string())],
            body,
        )
    }
}

// ── resp_* inspection helpers ────────────────────────────────────────────────
// These read the `{"error":{"code":N,"message":...}}` envelope that the
// business externs return as a `Value` (a `__json_object` GenericInstanceData).

/// Resolve a NanoValue to its heap id, if it references a heap object.
fn nv_heap_id(nv: auto_val::NanoValue) -> Option<u64> {
    if auto_val::is_object(nv) {
        Some(auto_val::decode_object(nv) as u64)
    } else if auto_val::is_i32(nv) {
        let v = auto_val::decode_i32(nv);
        // Negative i32 (e.g. the `null` literal's -1) is not a heap id.
        if v <= 0 {
            None
        } else {
            (v as u64 >= crate::vm::rc::HEAP_ID_BASE).then_some(v as u64)
        }
    } else {
        None
    }
}

/// Fetch a field of a GenericInstanceData by name, returning the inner Value.
fn gid_field(vm: &AutoVM, heap_id: u64, key: &str) -> Option<auto_val::Value> {
    use crate::vm::generic_registry::GenericInstanceData;
    let obj = vm.get_heap_object(heap_id)?;
    let guard = obj.read().unwrap();
    let inst = guard.as_any().downcast_ref::<GenericInstanceData>()?;
    inst.field_names.iter().position(|n| n == key).and_then(|i| inst.get_field(i).cloned())
}

/// Is a field Value an object (a VmRef to another GenericInstanceData)?
fn value_is_object(vm: &AutoVM, val: &auto_val::Value) -> bool {
    use crate::vm::generic_registry::GenericInstanceData;
    if let auto_val::Value::VmRef(r) = val {
        if let Some(obj) = vm.get_heap_object(r.id as u64) {
            return obj.read().unwrap().as_any().downcast_ref::<GenericInstanceData>().is_some();
        }
    }
    false
}

/// Read an i32/int field from a GenericInstanceData field Value.
fn gid_field_i32(vm: &AutoVM, heap_id: u64, key: &str) -> Option<i32> {
    match gid_field(vm, heap_id, key)? {
        auto_val::Value::Int(i) => Some(i),
        auto_val::Value::Uint(u) => Some(u as i32),
        auto_val::Value::I64(i) => Some(i as i32),
        auto_val::Value::U8(u) => Some(u as i32),
        auto_val::Value::I8(i) => Some(i as i32),
        _ => None,
    }
}

/// Read a string field from a GenericInstanceData field Value.
fn gid_field_str(vm: &AutoVM, heap_id: u64, key: &str) -> Option<String> {
    match gid_field(vm, heap_id, key)? {
        auto_val::Value::Str(s) => Some(s.to_string()),
        auto_val::Value::String(s) => Some(s.to_string()),
        auto_val::Value::StrSlice(s) => Some(s.to_string()),
        auto_val::Value::CStr(s) => Some(s.to_string()),
        _ => None,
    }
}

/// `resp_is_err(v)` → bool: `v` is an `{"error":{...}}` envelope.
/// Stack: v -> bool
pub fn shim_resp_is_err(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let err = nv_heap_id(nv)
        .and_then(|id| gid_field(vm, id, "error"))
        .map(|e| value_is_object(vm, &e))
        .unwrap_or(false);
    task.ram.push_nv(auto_val::encode_bool(err));
    Ok(())
}

/// `resp_err_code(v)` → int: `v["error"]["code"]`, default 500.
/// Stack: v -> int
pub fn shim_resp_err_code(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let code = nv_heap_id(nv)
        .and_then(|id| {
            let err = gid_field(vm, id, "error")?;
            if let auto_val::Value::VmRef(r) = err {
                gid_field_i32(vm, r.id as u64, "code")
            } else {
                None
            }
        })
        .unwrap_or(500);
    task.ram.push_i32(code);
    Ok(())
}

/// `resp_err_message(v)` → str: `v["error"]["message"]`, default "request failed".
/// Stack: v -> str
pub fn shim_resp_err_message(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let msg = nv_heap_id(nv)
        .and_then(|id| {
            let err = gid_field(vm, id, "error")?;
            if let auto_val::Value::VmRef(r) = err {
                gid_field_str(vm, r.id as u64, "message")
            } else {
                None
            }
        })
        .unwrap_or_else(|| "request failed".to_string());
    let idx = vm.add_string(msg.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

// ── SSE event builders (consumed by the Plan 442 C2 item ② streaming path) ──

/// An SSE frame carrying an optional event name + a JSON payload, exposed as an
/// opaque `axum::response::sse::Event` heap object. The SSE streaming path
/// (`.into_response()` on a `Sse` carried generator) formats these as
/// `event: <name>\ndata: <payload>\n\n`.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub name: Option<String>,
    pub data: String,
}

/// `sse_named_event(name, dto)` → Event with a name.
/// Stack: name, dto -> event_handle (dto on top)
pub fn shim_sse_named_event(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let name = pop_string(task, vm, "sse_named_event")?;
    let data = nv_to_json_or_null(vm, nv);
    crate::vm::ffi::stdlib::push_rust_obj(
        task,
        vm,
        "axum::response::sse::Event",
        SseEvent { name: Some(name), data },
    )
}

/// `sse_event(name, dto)` → Event with a name (server_stream alias).
/// Stack: name, dto -> event_handle (dto on top)
pub fn shim_sse_event(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let name = pop_string(task, vm, "sse_event")?;
    let data = nv_to_json_or_null(vm, nv);
    crate::vm::ffi::stdlib::push_rust_obj(
        task,
        vm,
        "axum::response::sse::Event",
        SseEvent { name: Some(name), data },
    )
}

/// `sse_plain_event(dto)` → Event without a name.
/// Stack: dto -> event_handle
pub fn shim_sse_plain_event(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let data = nv_to_json_or_null(vm, nv);
    crate::vm::ffi::stdlib::push_rust_obj(
        task,
        vm,
        "axum::response::sse::Event",
        SseEvent { name: None, data },
    )
}

// ── Sse/KeepAlive/into_response chain (Plan 442 C2 item ②) ───────────────────
// The backend SSE handlers build
//   `Sse.new(stream).keep_alive(KeepAlive.new()).into_response()`
// where `stream` is a generator's iterator id (`run_events_stream(...)`).
// `into_response()` returns that iterator id so the server's existing
// iterator→SSE branch streams it; the yielded `Event` objects are formatted
// by `sse_frame_from_nv`.

/// An `axum::response::sse::Sse` heap object wrapping the generator's iterator.
#[derive(Debug, Clone)]
pub struct Sse {
    pub iter_id: u32,
    pub keep_alive: bool,
}

/// An `axum::response::sse::KeepAlive` heap object (accepted, config stored).
#[derive(Debug, Clone)]
pub struct KeepAlive {
    pub enabled: bool,
}

/// Pop a heap-object handle arg.
fn pop_rust_handle(task: &mut AutoTask, ctx: &str) -> Result<u64, VMError> {
    let nv = crate::vm::native::pop_arg_nv(task);
    if auto_val::is_object(nv) {
        Ok(auto_val::decode_object(nv) as u64)
    } else if auto_val::is_i32(nv) {
        let v = auto_val::decode_i32(nv);
        if v <= 0 {
            Err(VMError::RuntimeError(format!("{ctx}: expected heap object handle")))
        } else {
            Ok(v as u64)
        }
    } else {
        Err(VMError::RuntimeError(format!("{ctx}: expected heap object handle")))
    }
}

/// `Sse.new(stream_iter_id)` → Sse heap object. Stack: iter_id -> sse_handle
pub fn shim_sse_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let iter_id = pop_i32(task) as u32;
    crate::vm::ffi::stdlib::push_rust_obj(
        task,
        vm,
        "axum::response::sse::Sse",
        Sse { iter_id, keep_alive: false },
    )
}

/// `KeepAlive.new()` → KeepAlive heap object. Stack: (none) -> keepalive_handle
pub fn shim_keepalive_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    crate::vm::ffi::stdlib::push_rust_obj(
        task,
        vm,
        "axum::response::sse::KeepAlive",
        KeepAlive { enabled: true },
    )
}

/// `sse.keep_alive(keep_alive)` → sse (mutated). Stack: sse_handle, ka_handle.
pub fn shim_sse_keep_alive(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _ka_handle = pop_rust_handle(task, "Sse.keep_alive")?;
    let sse_handle = pop_rust_handle(task, "Sse.keep_alive")?;
    if let Some(obj) = vm.get_heap_object(sse_handle) {
        let mut guard = obj.write().unwrap();
        if let Some(ro) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(sse) = ro.downcast_mut::<Sse>() {
                sse.keep_alive = true;
            }
        }
    }
    task.ram.push_i32(sse_handle as i32);
    Ok(())
}

/// `sse.into_response()` → the generator iterator id the server streams.
/// Stack: sse_handle -> iter_id
pub fn shim_sse_into_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sse_handle = pop_rust_handle(task, "Sse.into_response")?;
    let iter_id = if let Some(obj) = vm.get_heap_object(sse_handle) {
        let guard = obj.read().unwrap();
        guard
            .as_any()
            .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
            .and_then(|ro| ro.downcast_ref::<Sse>())
            .map(|s| s.iter_id)
            .unwrap_or(0)
    } else {
        0
    };
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// Read an `SseEvent` from a heap object handle if it is one.
fn get_sse_event(vm: &AutoVM, handle: u64) -> Option<SseEvent> {
    let obj = vm.get_heap_object(handle)?;
    let guard = obj.read().unwrap();
    guard
        .as_any()
        .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .and_then(|ro| ro.downcast_ref::<SseEvent>())
        .cloned()
}

/// Unwrap a `Result.Ok(...)` value to its single inner heap handle, if the
/// value is a Result.Some/Ok GenericInstanceData with one field.
fn result_ok_inner(vm: &AutoVM, handle: u64) -> Option<u64> {
    use crate::vm::generic_registry::GenericInstanceData;
    let obj = vm.get_heap_object(handle)?;
    let guard = obj.read().unwrap();
    let inst = guard.as_any().downcast_ref::<GenericInstanceData>()?;
    if inst.mono_name.contains("Result.Ok") || inst.mono_name.contains("Option.Some") {
        if let Some(auto_val::Value::VmRef(r)) = inst.get_field(0) {
            return Some(r.id as u64);
        }
    }
    None
}

/// Format one SSE frame from a yielded NanoValue. `None` = done/empty frame.
/// - `SseEvent` → `event: <name>\ndata: <payload>\n\n` (name optional).
/// - `Result.Ok(SseEvent)` → the same, unwrapped.
/// - raw scalar → `data: <value>\n\n` (preserves the legacy numeric/string path).
pub fn sse_frame_from_nv(vm: &AutoVM, nv: auto_val::NanoValue) -> Option<String> {
    // Try to render a heap value as a named/named-less SSE event frame.
    if let Some(handle) = nv_heap_id(nv) {
        let mut seen_handles = Vec::new();
        let mut cur = handle;
        loop {
            if let Some(ev) = get_sse_event(vm, cur) {
                let mut frame = String::new();
                if let Some(name) = &ev.name {
                    frame.push_str(&format!("event: {}\n", name));
                }
                frame.push_str(&format!("data: {}\n\n", ev.data));
                return Some(frame);
            }
            if seen_handles.len() > 4 {
                break; // do not chase chains indefinitely
            }
            match result_ok_inner(vm, cur) {
                Some(inner) if !seen_handles.contains(&inner) => {
                    seen_handles.push(cur);
                    cur = inner;
                }
                _ => break,
            }
        }
    }
    // Raw value: `data: <json>`. Non-heap scalars (int/str/bool) and heap
    // objects that aren't SSE events all land here, preserving the legacy path.
    Some(format!(
        "data: {}\n\n",
        crate::vm::ffi::http_server::nv_to_json(vm, nv, 0).unwrap_or_else(|| "null".to_string())
    ))
}

// ── Pure-logic value accessors (Plan 442 C2 item ③ path (b) start) ───────────
// These are the innermost extern helpers the backend corpus uses to read fields
// out of a `Value` (a `__json_object` GenericInstanceData produced by the json
// bridge / axum extractors). They need no Rust-registry access, so they are
// staged ahead of the data-source externs (relay_runs_list etc.) that depend on
// auto-musk's Rust side.

/// `value_get_str(v, k)` → str. Stack: v, k -> str (k on top).
pub fn shim_value_get_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let k = pop_string(task, vm, "value_get_str")?;
    let (nv, _stake) = pop_value_staked(task, vm);
    let out = nv_heap_id(nv)
        .and_then(|id| gid_field_str(vm, id, &k))
        .unwrap_or_default();
    let idx = vm.add_string(out.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `value_get_bool(v, k)` → bool. Stack: v, k -> bool (k on top).
pub fn shim_value_get_bool(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let k = pop_string(task, vm, "value_get_bool")?;
    let (nv, _stake) = pop_value_staked(task, vm);
    let out = nv_heap_id(nv).and_then(|id| match gid_field(vm, id, &k)? {
        auto_val::Value::Bool(b) => Some(b),
        _ => None,
    }).unwrap_or(false);
    task.ram.push_nv(auto_val::encode_bool(out));
    Ok(())
}

/// `value_is_null(v)` → bool. Stack: v -> bool.
pub fn shim_value_is_null(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (nv, _stake) = pop_value_staked(task, vm);
    let out = is_nullish(nv);
    task.ram.push_nv(auto_val::encode_bool(out));
    Ok(())
}

/// Push an `auto_val::Value` (a GenericInstanceData field payload) onto the
/// stack as a NanoValue, mirroring the container layouts the json bridge uses:
/// VmRef → heap-object handle, scalars → encoded tags, strings → pool index,
/// Nil/Null → nano null.
fn push_vm_value(task: &mut AutoTask, vm: &AutoVM, val: &auto_val::Value) -> Result<(), VMError> {
    use auto_val::Value;
    match val {
        Value::VmRef(r) => {
            vm.rc_push(task, auto_val::encode_object(r.id as u32));
        }
        Value::Int(i) => task.ram.push_nv(auto_val::encode_i32(*i)),
        Value::Uint(u) => task.ram.push_nv(auto_val::encode_i32(*u as i32)),
        Value::I8(i) => task.ram.push_nv(auto_val::encode_i32(*i as i32)),
        Value::I64(i) => task.ram.push_nv(auto_val::encode_i32(*i as i32)),
        Value::U8(u) => task.ram.push_nv(auto_val::encode_i32(*u as i32)),
        Value::USize(u) => task.ram.push_nv(auto_val::encode_i32(*u as i32)),
        Value::Bool(b) => task.ram.push_nv(auto_val::encode_bool(*b)),
        Value::Float(f) => task.ram.push_f64(*f as f64),
        Value::Double(d) => task.ram.push_f64(*d),
        Value::Str(s) => {
            let idx = vm.add_string(s.to_string().into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
        }
        Value::String(s) => {
            let idx = vm.add_string(s.to_string().into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
        }
        Value::StrSlice(s) => {
            let idx = vm.add_string(s.to_string().into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
        }
        Value::CStr(s) => {
            let idx = vm.add_string(s.to_string().into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
        }
        Value::Char(c) => {
            let idx = vm.add_string(c.to_string().into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
        }
        Value::Nil | Value::Null => task.ram.push_nv(auto_val::encode_null()),
        _ => task.ram.push_nv(auto_val::encode_null()),
    }
    Ok(())
}

/// Is an `auto_val::Value` a heap array (ListData<Value>)?
fn value_is_array(vm: &AutoVM, val: &auto_val::Value) -> bool {
    if let auto_val::Value::VmRef(r) = val {
        if let Some(obj) = vm.get_heap_object(r.id as u64) {
            return obj
                .read()
                .unwrap()
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                .is_some();
        }
    }
    false
}

/// `value_get(v, k)` → Value (field value). Stack: v, k -> value (k on top).
pub fn shim_value_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let k = pop_string(task, vm, "value_get")?;
    let (nv, _stake) = pop_value_staked(task, vm);
    let field = nv_heap_id(nv).and_then(|id| gid_field(vm, id, &k));
    match field {
        Some(val) => push_vm_value(task, vm, &val)?,
        None => task.ram.push_nv(auto_val::encode_null()),
    }
    Ok(())
}

/// `value_get_array(v, k)` → Value (field array, default []). Stack: v, k -> value.
pub fn shim_value_get_array(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let k = pop_string(task, vm, "value_get_array")?;
    let (nv, _stake) = pop_value_staked(task, vm);
    let field = nv_heap_id(nv).and_then(|id| gid_field(vm, id, &k));
    match field {
        Some(val) if value_is_array(vm, &val) => push_vm_value(task, vm, &val)?,
        _ => {
            // Empty array default.
            use crate::vm::types::ListData;
            let id = vm.insert_heap_object(ListData::<auto_val::Value>::new());
            vm.rc_push(task, auto_val::encode_object(id as u32));
        }
    }
    Ok(())
}

// ── Utility externs (pure; auto-lang has fastrand + hex) ─────────────────────

fn push_hex_string(task: &mut AutoTask, vm: &AutoVM, bytes: &[u8]) {
    let hex = hex::encode(bytes);
    let idx = vm.add_string(hex.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
}

/// `random_hex(n)` → str: `n` random bytes hex-encoded. Stack: n -> str.
pub fn shim_random_hex(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = pop_i32(task);
    if n <= 0 {
        push_hex_string(task, vm, &[]);
        return Ok(());
    }
    let bytes: Vec<u8> = (0..n).map(|_| fastrand::u8(..)).collect();
    push_hex_string(task, vm, &bytes);
    Ok(())
}

/// `new_id(n)` → str (alias of random_hex). Stack: n -> str.
pub fn shim_new_id(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    shim_random_hex(task, vm)
}

/// `hash_password(p, s)` → str: sha256(s || p) hex (extern_impl parity).
/// Stack: p, s -> str (s on top).
pub fn shim_hash_password(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s = pop_string(task, vm, "hash_password")?;
    let p = pop_string(task, vm, "hash_password")?;
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(s.as_bytes());
    h.update(p.as_bytes());
    let hex = hex::encode(h.finalize());
    let idx = vm.add_string(hex.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `path_inner(p)` → str: the Path extractor's inner segment string. On the VM
/// the axum adapter pushes Path extractors directly as a string, so this is the
/// identity for string args (empty for non-strings). Stack: p -> str.
pub fn shim_path_inner(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let nv = crate::vm::native::pop_arg_nv(task);
    if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv) as usize;
        if let Some(b) = vm.strings.read().unwrap().get(idx).cloned() {
            let s = String::from_utf8_lossy(&b).to_string();
            let nidx = vm.add_string(s.into_bytes());
            vm.rc_push_str_idx(task, nidx as usize);
            return Ok(());
        }
    }
    let nidx = vm.add_string(Vec::new());
    vm.rc_push_str_idx(task, nidx as usize);
    Ok(())
}

// ── Data-extern forwarding (Plan 442 C2 item ③ path (a)) ─────────────────────
// The data-source externs (relay_*/specs_*/app_config_*) cannot be implemented
// faithfully in auto-lang — they depend on auto-musk's Rust store/registry.
// Path (a) is to load the backend (auto-musk) as a cdylib and have it register
// name → HostCallFn (`host_bridge`/`backend_abi`); the VM extern then forwards
// to that registered host call. This forwarding helper proves the routing: if a
// host call is registered for `name`, forward `args_json` and push the JSON
// result; otherwise return Ok(false) so the caller serves a default.

fn try_host_forward(
    name: &str,
    task: &mut AutoTask,
    vm: &AutoVM,
    args_json: &str,
) -> Result<bool, VMError> {
    if !crate::vm::host_bridge::has_host_call(name) {
        return Ok(false);
    }
    let out = crate::vm::host_bridge::call_host(name, args_json)
        .map_err(|e| VMError::RuntimeError(format!("host '{name}': {e}")))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&out).unwrap_or(serde_json::Value::Null);
    push_value_from_json(task, vm, &parsed)?;
    Ok(true)
}

/// Push a serde_json Value onto the stack via `json_to_vm_value`, then give a
/// *fresh* heap-ref result one extra retain. The CALL_NAT dead-zone release
/// frees the slots the shim consumed (pop count > push count leaves the pushed
/// result inside the released range); an extra retain keeps a heap-ref result
/// alive until the caller pops it via its own StakeGuard — balancing so the
/// object is freed only after the caller is done reading it.
fn push_value_from_json(
    task: &mut AutoTask,
    vm: &AutoVM,
    value: &serde_json::Value,
) -> Result<(), VMError> {
    crate::vm::ffi::stdlib::json_to_vm_value(task, vm, value, 0)?;
    let sp = task.ram.sp;
    if sp > 0 {
        let top = task.ram.raw_nv[sp - 1];
        if let Some(id) = crate::vm::rc::heap_ref_id(top) {
            vm.rc_retain_id(id);
        }
    }
    Ok(())
}

/// `musk_extern_dispatch(name, args)` — PLAN-044 generic extern gate.
/// Stack: name(str), args(list of VM values) -> forwarded value (or null).
/// The musk `extern_sigs.at` stub bodies call this instead of hand-written
/// per-name shims: args are marshalled to a JSON array (State params are
/// simply not included by the caller side), forwarded to the host call
/// registered under `name` (path (a)); without a host call the fallback is
/// null — handlers' error envelopes treat it as empty data.
pub fn shim_musk_extern_dispatch(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (args_nv, _args_stake) = pop_value_staked(task, vm);
    let name = pop_string(task, vm, "musk_extern_dispatch")?;
    let args_json = crate::vm::ffi::http_server::nv_to_json(vm, args_nv, 0)
        .unwrap_or_else(|| "[]".to_string());
    if std::env::var("MUSK_VM_DEBUG").is_ok() {
        eprintln!("[VMDISP] {name} args={args_json}");
    }
    if try_host_forward(&name, task, vm, &args_json)? {
        return Ok(());
    }
    if std::env::var("MUSK_VM_DEBUG").is_ok() {
        eprintln!("[VMDISP] {name} -> no host, fallback null");
    }
    task.ram.push_nv(auto_val::encode_null());
    Ok(())
}

/// `relay_runs_list(s, q)` → `{runs: [...]}`. Forwards to a registered host call
/// (path (a)) with the Query extractor marshalled into `args_json` (the State
/// is opaque and carried by the backend via the workspace registry); without a
/// host call, serves the empty-store shape `{runs: []}`. Stack: s, q -> value.
pub fn shim_relay_runs_list(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let (q, _q_stake) = pop_value_staked(task, vm); // Query<WorkspaceQuery> (GenericInstanceData, top)
    let (_s, _s_stake) = pop_value_staked(task, vm); // State<AppState> (opaque)
    let args_json =
        crate::vm::ffi::http_server::nv_to_json(vm, q, 0).unwrap_or_else(|| "{}".to_string());
    if try_host_forward("relay_runs_list", task, vm, &args_json)? {
        return Ok(());
    }
    push_value_from_json(task, vm, &serde_json::json!({ "runs": [] }))?;
    Ok(())
}

/// `app_config_effective_daemon_url(cfg)` → str (default constant). Forwards to
/// a registered host call (path (a)); without one, serves the default daemon URL
/// (parity with extern_impl). String-returning, so it exercises the forwarding
/// path without the nested-object RC pitfalls of a Value extern. Stack: cfg -> str.
pub fn shim_app_config_effective_daemon_url(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    let (_cfg, _stake__cfg) = pop_value_staked(task, vm); // config Value (unused for the default)
    if try_host_forward("app_config_effective_daemon_url", task, vm, "{}")? {
        return Ok(());
    }
    let s = "http://127.0.0.1:17654";
    let idx = vm.add_string(s.as_bytes().to_vec());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}
