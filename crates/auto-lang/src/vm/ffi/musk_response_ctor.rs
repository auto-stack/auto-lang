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

/// Pop a raw NanoValue argument with an RC stake so the underlying heap
/// object stays alive while the shim reads it.
fn pop_value(task: &mut AutoTask, vm: &AutoVM) -> auto_val::NanoValue {
    let nv = crate::vm::native::pop_arg_nv(task);
    let _stake = crate::vm::native::StakeGuard::nv(vm, nv);
    let _ = _stake;
    nv
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
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
    let nv = pop_value(task, vm);
    let data = nv_to_json_or_null(vm, nv);
    crate::vm::ffi::stdlib::push_rust_obj(
        task,
        vm,
        "axum::response::sse::Event",
        SseEvent { name: None, data },
    )
}
