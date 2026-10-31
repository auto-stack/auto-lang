//! Language value conversion used by HTTP and language response constructors.
use super::json_escape_string;

/// Decode the legacy native string-list ABI only under an actual compiled
/// return declaration. Untyped/negative integer lists retain numeric JSON;
/// changing their interpretation by probing the string pool would be ambiguous.
pub fn nv_to_typed_json(
    vm: &crate::vm::engine::AutoVM,
    nv: auto_val::NanoValue,
    return_type: Option<&str>,
    depth: u32,
) -> Option<String> {
    let outer = return_type
        .unwrap_or("")
        .trim()
        .trim_start_matches('~')
        .trim();
    // The compiler desugars ~T to Future<T> in canonical owner metadata.
    let declaration = outer
        .strip_prefix("Future<")
        .and_then(|inner| inner.strip_suffix('>'))
        .unwrap_or(outer)
        .trim();
    if !matches!(declaration, "[]str" | "List<str>") {
        return nv_to_json(vm, nv, depth);
    }
    if depth >= 32 {
        return None;
    }
    let id = if auto_val::is_object(nv) {
        auto_val::decode_object(nv) as u64
    } else if auto_val::is_list(nv) {
        auto_val::decode_list(nv) as u64
    } else if auto_val::is_i32(nv) {
        auto_val::decode_i32(nv) as u64
    } else {
        return None;
    };
    let object = vm.get_heap_object(id)?;
    let guard = object.read().ok()?;
    if let Some(list) = guard
        .as_any()
        .downcast_ref::<crate::vm::types::ListData<i32>>()
    {
        let mut values = Vec::with_capacity(list.elems.len());
        for &encoded in &list.elems {
            values.push(legacy_string_element_json(vm, encoded)?);
        }
        return Some(format!("[{}]", values.join(", ")));
    }
    if let Some(list) = guard
        .as_any()
        .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
    {
        let mut values = Vec::with_capacity(list.elems.len());
        for value in &list.elems {
            values.push(match value {
                auto_val::Value::Str(s) => json_escape_string(s.as_str()),
                auto_val::Value::CStr(s) => json_escape_string(s.as_str()),
                _ => return None,
            });
        }
        return Some(format!("[{}]", values.join(", ")));
    }
    // A nominal user record may share a display name with a container. Keep
    // ordinary object serialization; only physical legacy lists use sentinels.
    drop(guard);
    nv_to_json(vm, nv, depth)
}

fn legacy_string_element_json(vm: &crate::vm::engine::AutoVM, encoded: i32) -> Option<String> {
    let index = encoded.checked_neg()?.checked_sub(1)?;
    let strings = vm.strings.read().ok()?;
    let bytes = strings.get(usize::try_from(index).ok()?)?;
    Some(json_escape_string(&String::from_utf8_lossy(bytes)))
}

#[cfg(test)]
mod typed_native_list_tests {
    use super::*;
    use crate::vm::ffi::convert::VMConvertible;
    use crate::vm::{engine::AutoVM, task::AutoTask, virt_memory::VirtualFlash};

    #[test]
    fn compiled_string_list_declaration_preserves_legacy_abi_and_numeric_controls() {
        let vm = AutoVM::new(VirtualFlash::new_with_code(vec![]), 1024);
        let values = vec![
            "".to_string(),
            "重复.png".to_string(),
            "重复.png".to_string(),
            "quote\"\\\n.jpg".to_string(),
        ];
        let mut task = AutoTask::new(0, 2048, 0);
        values.push_to_stack(&mut task, &vm).unwrap();
        let nv = task.ram.peek_nv(0);
        let id = auto_val::decode_i32(nv) as u64;
        let pool_size = vm.strings.read().unwrap().len();
        let counts: Vec<_> = (0..pool_size).map(|i| vm.pool_count(i)).collect();
        assert_eq!(vm.rc_count(id), 1);
        let actual = nv_to_typed_json(&vm, nv, Some("[]str"), 0).unwrap();
        assert_eq!(
            serde_json::from_str::<Vec<String>>(&actual).unwrap(),
            values
        );
        for declaration in ["~[]str", "Future<[]str>", "List<str>"] {
            assert_eq!(
                nv_to_typed_json(&vm, nv, Some(declaration), 0).unwrap(),
                actual
            );
        }
        assert_eq!(vm.strings.read().unwrap().len(), pool_size);
        assert_eq!(
            (0..pool_size).map(|i| vm.pool_count(i)).collect::<Vec<_>>(),
            counts
        );
        assert_eq!(vm.rc_count(id), 1);
        assert!(vm
            .get_heap_object(id)
            .unwrap()
            .read()
            .unwrap()
            .as_any()
            .is::<crate::vm::types::ListData<i32>>());
        let mut borrowed = AutoTask::new(0, 1024, 0);
        borrowed.ram.push_i32(id as i32);
        assert_eq!(
            Vec::<String>::pop_from_stack(&mut borrowed, &vm).unwrap(),
            values
        );

        let mut ints = AutoTask::new(0, 1024, 0);
        let mut int_values = crate::vm::types::ListData::<auto_val::Value>::new();
        for n in [-1_i32, -4, -5] {
            int_values.push(auto_val::Value::Int(n));
        }
        vm.rc_push_id(&mut ints, vm.insert_heap_object(int_values));
        let negative = ints.ram.peek_nv(0);
        assert!(nv_to_typed_json(&vm, negative, Some("[]str"), 0).is_none());
        for declaration in [None, Some("[]int"), Some("UnrelatedRecord")] {
            assert_eq!(
                nv_to_typed_json(&vm, negative, declaration, 0).unwrap(),
                "[-1, -4, -5]"
            );
        }
        vm.rc_release_task_stack(&mut ints);
        let mut typed = crate::vm::types::ListData::<auto_val::Value>::new();
        for s in &values {
            typed.push(auto_val::Value::Str(s.clone().into()));
        }
        let mut typed_task = AutoTask::new(0, 1024, 0);
        vm.rc_push_id(&mut typed_task, vm.insert_heap_object(typed));
        assert_eq!(
            nv_to_typed_json(&vm, typed_task.ram.peek_nv(0), Some("[]str"), 0).unwrap(),
            actual
        );
        vm.rc_release_task_stack(&mut typed_task);
        let mut record = crate::vm::types::ObjectData::new();
        record
            .fields
            .insert("value".into(), auto_val::Value::Int(42));
        let mut record_task = AutoTask::new(0, 1024, 0);
        vm.rc_push_id(&mut record_task, vm.insert_heap_object(record));
        let record_json =
            nv_to_typed_json(&vm, record_task.ram.peek_nv(0), Some("List<str>"), 0).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&record_json).unwrap(),
            serde_json::json!({"value":42})
        );
        vm.rc_release_task_stack(&mut record_task);
        let mut broken = AutoTask::new(0, 1024, 0);
        vec![7_i32].push_to_stack(&mut broken, &vm).unwrap();
        assert!(nv_to_typed_json(&vm, broken.ram.peek_nv(0), Some("[]str"), 0).is_none());
        vm.rc_release_task_stack(&mut broken);
        assert_eq!(vm.strings.read().unwrap().len(), pool_size);
        assert_eq!(
            (0..pool_size).map(|i| vm.pool_count(i)).collect::<Vec<_>>(),
            counts
        );
        vm.rc_release_task_stack(&mut task);
        assert_eq!(vm.rc_count(id), 0);
        // RC uses a bounded dying grace queue; quiescent teardown drains it.
        vm.reap_all();
        assert!(!vm.heap_objects.contains_key(&id));

        let mut empty = AutoTask::new(0, 1024, 0);
        Vec::<String>::new().push_to_stack(&mut empty, &vm).unwrap();
        assert_eq!(
            nv_to_typed_json(&vm, empty.ram.peek_nv(0), Some("[]str"), 0).unwrap(),
            "[]"
        );
        vm.rc_release_task_stack(&mut empty);
    }
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

/// Format an f64 as JSON (integers without trailing .0, per JSON convention the
/// number is still valid; we keep the natural Rust representation).
pub fn format_f64_json(f: f64) -> String {
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
