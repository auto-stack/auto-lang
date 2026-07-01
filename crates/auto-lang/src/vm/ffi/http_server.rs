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

use std::collections::HashMap;
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
}

/// Match a request (method, path) against a list of routes.
/// Supports `:param` path parameter extraction (e.g. /api/notes/:id).
pub fn match_route(routes: &[HttpRoute], method: &str, path: &str) -> Option<RouteMatch> {
    for route in routes {
        if route.method.to_uppercase() != method.to_uppercase() {
            continue;
        }
        let route_segments: Vec<&str> = route.path.split('/').collect();
        let path_segments: Vec<&str> = path.split('/').collect();
        if route_segments.len() != path_segments.len() {
            continue;
        }
        let mut params = Vec::new();
        let mut matched = true;
        for (rs, ps) in route_segments.iter().zip(path_segments.iter()) {
            if let Some(param_name) = rs.strip_prefix(':') {
                params.push((param_name.to_string(), ps.to_string()));
            } else if rs != ps {
                matched = false;
                break;
            }
        }
        if matched {
            return Some(RouteMatch {
                fn_name: route.fn_name.clone(),
                path_params: params,
            });
        }
    }
    None
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
    // Array ids start at 2_000_000 (engine.rs array_id_gen), heap object ids
    // at 4_000_000 (heap_object_id_gen). Rather than assume a range (which
    // could misclassify large user integers), we probe the VM tables: if the
    // value is a known array/heap id, expand it; otherwise treat as a plain int.
    if auto_val::is_i32(nv) {
        let v = auto_val::decode_i32(nv);
        if depth < MAX_DEPTH {
            let id = v as u64;
            if vm.arrays.contains_key(&id) || vm.heap_objects.contains_key(&id) || vm.objects.contains_key(&id) {
                if let Some(json) = heap_object_to_json(vm, id, depth) {
                    return Some(json);
                }
            }
        }
        return Some(v.to_string());
    }
    Some("null".to_string())
}

/// Expand a heap object ID into JSON. Handles three storage backends used by
/// the VM: `heap_objects` (GenericInstanceData, type instances), `arrays`
/// (`Vec<Value>` from `[...]` literals), and `objects` (ObjectData maps).
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
        // ListData<i32> (int collections).
        if let Some(list) = guard.as_any().downcast_ref::<crate::vm::types::ListData<i32>>() {
            let parts: Vec<String> = list.elems.iter().map(|i| i.to_string()).collect();
            return Some(format!("[{}]", parts.join(", ")));
        }
        // Other heap objects (opaque types) — can't serialize generically.
        return None;
    }

    // 2. arrays: Vec<Value> (array literals like [a, b, c])
    if let Some(arr_ref) = vm.arrays.get(&id) {
        let arr = arr_ref.read().unwrap();
        let mut parts: Vec<String> = Vec::new();
        for elem in arr.iter() {
            let json = value_to_json(vm, elem, depth + 1)
                .unwrap_or_else(|| "null".to_string());
            parts.push(json);
        }
        return Some(format!("[{}]", parts.join(", ")));
    }

    // 3. objects: ObjectData maps ({ key: value, ... })
    if let Some(obj_ref) = vm.objects.get(&id) {
        let obj = obj_ref.read().unwrap();
        let mut parts: Vec<String> = Vec::new();
        for (key, val) in obj.fields.iter() {
            let key_json = json_escape_string(&key.to_string());
            let val_json = value_to_json(vm, val, depth + 1)
                .unwrap_or_else(|| "null".to_string());
            parts.push(format!("{}: {}", key_json, val_json));
        }
        return Some(format!("{{{}}}", parts.join(", ")));
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
            // Probe the VM tables to decide: array/heap id → expand, else plain int.
            let id = *i as u64;
            if vm.arrays.contains_key(&id) || vm.heap_objects.contains_key(&id) || vm.objects.contains_key(&id) {
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
    /// a struct stored as Value::Int(heap_id). Array id is allocated via the
    /// VM's array_id_gen (starts at 2_000_000). nv_to_json must recurse.
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
        // Allocate an array id the same way the engine does (engine.rs:1585).
        let arr_id = vm.array_id_gen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        vm.arrays.insert(arr_id, std::sync::Arc::new(std::sync::RwLock::new(vec![
            auto_val::Value::Int(id_a),
            auto_val::Value::Int(id_b),
        ])));
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

    /// Extract the body (after the blank line) from a raw HTTP response.
    fn body_of(resp: &str) -> &str {
        resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or(resp)
    }

    /// NOTE: these e2e tests start a real HTTP server. They set the
    /// process-global AUTO_HTTP_PORT env var, so they MUST run serially.
    /// Default-ignored to keep the parallel test suite green; run with:
    ///   cargo test -p auto-lang --lib e2e_ -- --ignored --test-threads=1
    #[test]
    #[ignore]
    fn e2e_struct_handler_returns_json() {
        let port = 18731; // unique port per test to avoid clashes
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = format!(r#"
type Note {{ id int; title str }}

#[api(method = "GET", path = "/api/notes/test")]
fn get_note() Note {{
    Note {{ id: 1, title: "hello" }}
}}
"#);
        // Run the program on a detached thread. run() detects #[api] routes
        // and starts the AutoVM HTTP server, blocking this thread forever —
        // which is fine, the test process exits after assertion.
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let _ = crate::run(&code);
            })
            .expect("spawn server thread");

        let resp = http_get(port, "/api/notes/test");
        let body = body_of(&resp);
        // The fix: body must be JSON object, not the bare heap-id "4000000".
        assert_eq!(
            body, r#"{"id": 1, "title": "hello"}"#,
            "struct handler JSON: full resp = {:?}", resp
        );
    }

    #[test]
    #[ignore]
    fn e2e_int_path_param_handler() {
        let port = 18732;
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = format!(r#"
#[api(method = "GET", path = "/api/echo/:id")]
fn echo_id(id int) int {{
    id
}}
"#);
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let _ = crate::run(&code);
            })
            .expect("spawn server thread");

        let resp = http_get(port, "/api/echo/42");
        let body = body_of(&resp);
        // Phase 5: :id injected as int 42, returned as-is.
        assert_eq!(body, "42", "int path param: full resp = {:?}", resp);
    }

    /// Plan 327 Phase 3: SSE handler returning a generator (~Iter<int>).
    /// Each yield becomes an SSE data frame. Lazy evaluation means each
    /// next() runs only to the next yield (not the whole body upfront).
    #[test]
    #[ignore]
    fn e2e_sse_generator_handler() {
        let port = 18733;
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = format!(r#"
#[api(method = "GET", path = "/api/counter")]
fn counter_handler() ~Iter<int> {{
    yield 1
    yield 2
    yield 3
}}
"#);
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let _ = crate::run(&code);
            })
            .expect("spawn server thread");

        let resp = http_get(port, "/api/counter");
        // SSE response: the body should contain three "data: N\n\n" frames.
        let body = body_of(&resp);
        assert!(body.contains("data: 1"), "SSE frame 1: body={:?}", body);
        assert!(body.contains("data: 2"), "SSE frame 2: body={:?}", body);
        assert!(body.contains("data: 3"), "SSE frame 3: body={:?}", body);
    }

    /// Plan 327 Phase 3 遗留: SSE handler that INDIRECTLY calls a generator
    /// (handler itself has no yield; it calls a generator fn). The handler
    /// returns the iter_id from the inner generator; SSE detection must still
    /// fire on that iter_id.
    #[test]
    #[ignore]
    fn e2e_sse_indirect_generator() {
        let port = 18734;
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = format!(r#"
fn counter() ~Iter<int> {{
    yield 1
    yield 2
    yield 3
}}
#[api(method = "GET", path = "/api/stream")]
fn stream_handler() ~Iter<int> {{
    return counter()
}}
"#);
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let _ = crate::run(&code);
            })
            .expect("spawn server thread");

        let resp = http_get(port, "/api/stream");
        let body = body_of(&resp);
        assert!(body.contains("data: 1"), "indirect SSE frame 1: body={:?}", body);
        assert!(body.contains("data: 2"), "indirect SSE frame 2: body={:?}", body);
        assert!(body.contains("data: 3"), "indirect SSE frame 3: body={:?}", body);
    }

    /// Plan 327 Phase 4: concurrent SSE — two simultaneous connections to the
    /// same streaming endpoint. Both must receive complete data. Under the old
    /// serial server, the second connection would block until the first's
    /// generator exhausted. With serve_async + spawn_local + yield_now, the
    /// two handlers interleave (Goroutine-style cooperative scheduling).
    #[test]
    #[ignore]
    fn e2e_concurrent_sse() {
        let port = 18735;
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = format!(r#"
#[api(method = "GET", path = "/api/count")]
fn counter_handler() ~Iter<int> {{
    yield 1
    yield 2
    yield 3
}}
"#);
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let _ = crate::run(&code);
            })
            .expect("spawn server thread");

        // Fire two connections concurrently from separate threads.
        let h1 = std::thread::spawn(move || http_get(port, "/api/count"));
        let h2 = std::thread::spawn(move || http_get(port, "/api/count"));
        let resp1 = h1.join().expect("conn1");
        let resp2 = h2.join().expect("conn2");
        let body1 = body_of(&resp1);
        let body2 = body_of(&resp2);
        // Both connections must receive all three frames.
        assert!(body1.contains("data: 1") && body1.contains("data: 2") && body1.contains("data: 3"),
            "conn1 incomplete: body={:?}", body1);
        assert!(body2.contains("data: 1") && body2.contains("data: 2") && body2.contains("data: 3"),
            "conn2 incomplete: body={:?}", body2);
    }

    /// Plan 327 Phase 4 validation: 015-notes-style CRUD on the async HTTP
    /// server. Exercises the same patterns as examples/ui/015-notes/src/back:
    ///   - list: returns []Note (array of structs → JSON array of objects)
    ///   - get:  :id path param + ?Note (Option → inner value or null)
    ///   - create: POST body (title/body) → Note
    /// Uses a module-level var for in-memory storage (like db.at's `var notes`).
    #[test]
    #[ignore]
    fn e2e_notes_crud() {
        let port = 18736;
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = format!(r#"
type Note {{ id int; title str; body str; time str }}

var notes = [
    Note {{ id: 0, title: "Welcome", body: "first", time: "now" }},
    Note {{ id: 1, title: "Shopping", body: "milk", time: "ago" }},
]
var nextid int = 2

#[api(method = "GET", path = "/api/notes")]
fn list_notes() []Note {{
    return notes
}}

#[api(method = "GET", path = "/api/notes/:id")]
fn get_note(id int) ?Note {{
    for note in notes {{
        if note.id == id {{
            return Some(note)
        }}
    }}
    return None
}}

#[api(method = "POST", path = "/api/notes")]
fn create_note(title str, body str) Note {{
    let note = Note {{ id: nextid, title: title, body: body, time: "now" }}
    nextid = nextid + 1
    return note
}}
"#);
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let _ = crate::run(&code);
            })
            .expect("spawn server thread");

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

    /// Plan 327 final validation: 015-notes backend pattern with List<Note>
    /// generic + module-level var + #[api] handler returning the list.
    /// This mirrors db.at's `var notes List<Note>` + `fn all_notes() []Note`.
    #[test]
    #[ignore]
    fn e2e_notes_list_generic() {
        let port = 18737;
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = format!(r#"
type Note {{ id int; title str; body str; time str }}

var notes = [
    Note {{ id: 0, title: "Welcome", body: "first", time: "now" }},
    Note {{ id: 1, title: "Shopping", body: "milk", time: "ago" }},
]

#[api(method = "GET", path = "/api/notes")]
fn list_notes() []Note {{
    return notes
}}
"#);
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let _ = crate::run(&code);
            })
            .expect("spawn server thread");

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
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let req_method = parts[0].to_uppercase();
        let req_path = parts[1].to_string();

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
                let resp = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
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
                                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n"
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
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status, body_json.len(), body_json
        );
        let _ = stream.write_all(response.as_bytes());
    }
}

/// Plan 327 Phase 4: Concurrent HTTP server using tokio async I/O.
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
        let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n").await;
        return;
    }
    let req_method = parts[0].to_uppercase();
    let req_path = parts[1].to_string();

    // Parse body (after blank line) if Content-Length present.
    let mut content_length = 0usize;
    let mut body = String::new();
    let mut header_done = false;
    for line in lines {
        if !header_done {
            if line.is_empty() {
                header_done = true;
                continue;
            }
            if line.to_lowercase().starts_with("content-length:") {
                content_length = line[15..].trim().parse().unwrap_or(0);
            }
        } else if body.len() < content_length {
            body.push_str(line);
        }
    }

    // Route match
    let route_match = match match_route(routes, &req_method, &req_path) {
        Some(rm) => rm,
        None => {
            let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
            return;
        }
    };

    // Dispatch to handler via call_fn_by_name (synchronous VM execution).
    let handler_task_id = vm.spawn_task(0, 65536);
    let n_args = build_handler_args(vm, handler_task_id, &route_match, &body);

    let result_json = if let Some(_task_arc) = vm.tasks.get(&handler_task_id) {
        let mut ht = match _task_arc.try_lock() {
            Ok(t) => t,
            Err(_) => { vm.tasks.remove(&handler_task_id); return; }
        };
        match vm.call_fn_by_name(&mut ht, &route_match.fn_name, n_args) {
            Ok(()) => {
                let nv = ht.ram.pop_nv();
                // SSE detection: generator/iterator return → stream frames.
                if auto_val::is_i32(nv) {
                    let iter_id = auto_val::decode_i32(nv) as u32;
                    if vm.iterators.contains_key(&iter_id) {
                        drop(ht);
                        let sse_header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";
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
                    } else {
                        nv_to_json(vm, nv, 0)
                    }
                } else {
                    nv_to_json(vm, nv, 0)
                }
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

    // Non-SSE: write JSON response.
    if let Some(result_json) = result_json {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            result_json.len(), result_json
        );
        let _ = stream.write_all(response.as_bytes()).await;
    }
}

/// Build handler arguments on the task's stack (path params + body).
/// Returns the number of args pushed.
fn build_handler_args(
    vm: &crate::vm::engine::AutoVM,
    task_id: u64,
    route_match: &RouteMatch,
    body: &str,
) -> usize {
    let mut n_args = 0;
    if let Some(_task_arc) = vm.tasks.get(&task_id) {
        if let Ok(mut task) = _task_arc.try_lock() {
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
            if !body.is_empty() {
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(body.as_bytes().to_vec());
                    i
                };
                task.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }
        }
    }
    n_args
}
