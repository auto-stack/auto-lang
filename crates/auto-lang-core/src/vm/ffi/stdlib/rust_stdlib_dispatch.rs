//! PLAN-884 T-04：rust_stdlib 动态派发（push/pop_rust_obj + 巨 match + fallback）
//! 自 stdlib 壳迁入。
use super::*;

// ============================================================================
// Plan 192: Rust stdlib Dynamic Dispatch Handler
// ============================================================================

/// Push a Rust stdlib object onto the VM heap and push its handle.
pub(crate) fn push_rust_obj<T: Any + Send + Sync + 'static>(
    task: &mut AutoTask,
    vm: &AutoVM,
    type_name: &str,
    value: T,
) -> Result<(), VMError> {
    let obj = RustStdlibObject::new(type_name, value);
    let handle = vm.insert_heap_object(obj) as u32;
    {
        vm.rc_push(task, auto_val::encode_object(handle));
    }
    Ok(())
}

/// PLAN-054 T5 (A7): Date.format(epoch_ms, pattern) 宿主格式化——本地时区,
/// 对齐 web 轨 `new Date(ms).toLocaleTimeString()`。token 按连续同字符
/// run 解析:yyyy/MM/dd/HH/hh/mm/ss/SSS 全宽,单字符窄位;未知 run 原样保留
/// （分隔符 `:` 等自然直通）。ms 非法(溢出/回退)返回空串。
/// PLAN-536 T4(题3): 双口径归一——后端时间戳常以 epoch **秒**下发(musk
/// created_at=1788436450 实录),此前恒按毫秒换算 → 1970 垃圾时刻(呈现为
/// "非整时区偏移"假象)。阈值 1e11 沿业界惯例:真实纪元秒(< 2286 年)恒
/// < 1e11,真实纪元毫秒(> 1973-03)恒 ≥ 1e11。归一在 Local 换算之前,
/// 只动单位、不动时区。
pub(crate) fn format_date_ms(ms: i64, pattern: &str) -> String {
    use chrono::{Datelike, Local, TimeZone, Timelike};
    let ms = if ms.abs() < 100_000_000_000 {
        ms.saturating_mul(1000)
    } else {
        ms
    };
    let dt = match Local.timestamp_millis_opt(ms) {
        chrono::LocalResult::Single(dt) => dt,
        _ => return String::new(),
    };
    let mut out = String::with_capacity(pattern.len() + 8);
    let bytes = pattern.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        let run = bytes[i..].iter().take_while(|&&c| c == b).count();
        let token = &pattern[i..i + run];
        let rep: String = match token {
            "yyyy" | "YYYY" => format!("{:04}", dt.year()),
            "MM" => format!("{:02}", dt.month()),
            "dd" | "DD" => format!("{:02}", dt.day()),
            "HH" | "hh" => format!("{:02}", dt.hour()),
            "H" => dt.hour().to_string(),
            "mm" => format!("{:02}", dt.minute()),
            "m" => dt.minute().to_string(),
            "ss" => format!("{:02}", dt.second()),
            "s" => dt.second().to_string(),
            "SSS" => format!("{:03}", dt.timestamp_subsec_millis()),
            other => other.to_string(),
        };
        out.push_str(&rep);
        i += run;
    }
    out
}

/// Pop a heap handle and return a reference to the RustStdlibObject.
fn pop_rust_obj(task: &mut AutoTask, vm: &AutoVM, context: &str) -> Result<u64, VMError> {
    {
        let nv = task.ram.pop_nv();
        let handle = if auto_val::is_object(nv) {
            auto_val::decode_object(nv) as u64
        } else {
            auto_val::decode_i32(nv) as u64
        };
        if vm.get_heap_object(handle).is_none() {
            return Err(VMError::RuntimeError(format!(
                "Invalid Rust stdlib handle in {} (handle={})",
                context, handle
            )));
        }
        Ok(handle)
    }
}

/// Generic dispatch handler for Rust stdlib calls.
///
/// Stack layout (popped in reverse order):
///   ... user args ... | method: String | type_name: String
pub(crate) fn shim_rust_stdlib_dispatch(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // 壳(PLAN-596 T4):弹 method/type_name → 头部剥 mono 后缀走全链(裸名是
    // std/handwritten/native_catalog/dep 裸名条目的唯一键——Red.paint__String
    // 全链 miss 回归实证);裸名全链 miss 且原名带后缀 → 带原名再走全链一次
    // (dep_methods 泛型实例键带后缀注册)。
    let method: String = String::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(format!("rust_stdlib_dispatch: {}", e)))?;
    let type_name: String = String::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(format!("rust_stdlib_dispatch: {}", e)))?;
    let stripped = if method.ends_with("__i64") || method.ends_with("__String") {
        method.split("__").next().unwrap_or(&method).to_string()
    } else {
        method.clone()
    };
    let unknown = "Unknown Rust stdlib call";
    match shim_rust_stdlib_dispatch_inner(task, vm, &type_name, &stripped) {
        Err(VMError::RuntimeError(msg)) if msg.starts_with(unknown) && stripped != method => {
            shim_rust_stdlib_dispatch_inner(task, vm, &type_name, &method)
        }
        other => other,
    }
}

fn shim_rust_stdlib_dispatch_inner(
    task: &mut AutoTask,
    vm: &AutoVM,
    type_name: &str,
    method: &str,
) -> Result<(), VMError> {
    // Plan 430 D1: 生成段(shim-metadata)优先;命中即返回,未命中回退手写臂。
    match crate::vm::ffi::generated_std::generated_std_dispatch(type_name, method, task, vm)? {
        Some(()) => return Ok(()),
        None => {}
    }

    match (type_name, method) {
        // Plan 442 C2: axum serve adapter —— Router/MethodRouter 构建。
        // 裸 `get(h)`/`post(h)`... 以空 type_name 到达(Ident 调用);链式
        // `.post(h)` 以堆对象 tag "axum::MethodRouter"(短化后
        // "MethodRouter")到达。Router.route 即时安装路由到 HTTP 注册表,
        // 供 execute_autovm 的 auto-serve 检查接线。
        ("Router", "new") => {
            crate::vm::ffi::axum_adapter::shim_router_new(task, vm)?;
        }
        ("Router", "route") => {
            crate::vm::ffi::axum_adapter::shim_router_route(task, vm)?;
        }
        ("Router", "serve") => {
            crate::vm::ffi::axum_adapter::shim_router_serve(task, vm)?;
        }
        ("MethodRouter", "get")
        | ("MethodRouter", "post")
        | ("MethodRouter", "put")
        | ("MethodRouter", "delete")
        | ("MethodRouter", "patch") => {
            crate::vm::ffi::axum_adapter::shim_method_chain(task, vm, method)?;
        }
        ("", "get") | ("", "post") | ("", "put") | ("", "delete") | ("", "patch") => {
            crate::vm::ffi::axum_adapter::shim_method_bare(task, vm, method)?;
        }

        // PLAN-044 (musk vm_entry): opaque AppState accessor pass-through.
        // musk handlers write `specs_load(s.view, q)` — on the VM side `s` is
        // the opaque app_state_handle; `.view` (and any sibling accessor)
        // yields the same handle (the host closure captures the real state).
        ("AppState", "view")
        | ("AppState", "registry")
        | ("AppState", "auth")
        | ("AppState", "client") => {
            let receiver = task.ram.pop_i32();
            task.ram.push_i32(receiver);
        }

        // PLAN-044: `to_value(x)` (serde_json::to_value) — VM 里 DTO 实例
        // 本身即可序列化(heap_object_to_json),恒等透传;下游 sse_event/
        // Json 构造自行 nv_to_json。
        ("", "to_value") => {
            let v = task.ram.pop_nv();
            task.ram.push_nv(v);
        }

        // PLAN-044 (musk vm_entry): bare `Json(d)` constructor (axum
        // `Json<T>` return form, e.g. musk health() -> ~Json<StatusOk>) —
        // same wire shape as json_response: 200 + application/json of d.
        ("", "Json") => {
            crate::vm::ffi::musk_response_ctor::shim_json_response(task, vm)?;
        }

        // PLAN-044 (musk vm_entry): axum layer/constructor pass-throughs.
        // The VM emulation ignores layers (CORS/body-limit are handled by the
        // stdnet server per-request); `DefaultBodyLimit.max(n)` only feeds
        // `.layer(..)`, so both sides collapse to opaque no-ops.
        ("DefaultBodyLimit", "max") => {
            let _n = task.ram.pop_i32();
            let obj = RustStdlibObject::new("axum::extract::DefaultBodyLimit", ());
            let handle = vm.insert_heap_object(obj);
            // PLAN-667 (F-07): bare push_i32(handle) migrated to rc_push_id —
            // a heap id pushed onto the stack must carry a staked share.
            vm.rc_push_id(task, handle as u64);
        }
        ("Router", "layer")
        | ("Router", "with_state")
        | ("Router", "merge")
        | ("MethodRouter", "layer") => {
            let _layer_arg = task.ram.pop_i32();
            let receiver = task.ram.pop_i32();
            task.ram.push_i32(receiver);
        }

        // Plan 442 C2 item ②: axum SSE form — `Sse.new(stream)` →
        // `.keep_alive(KeepAlive.new())` → `.into_response()`. `stream` is a
        // generator's iterator id; `into_response()` returns it so the server's
        // iterator→SSE branch streams the yielded `Event` frames.
        ("Sse", "new") => {
            crate::vm::ffi::musk_response_ctor::shim_sse_new(task, vm)?;
        }
        ("Sse", "keep_alive") => {
            crate::vm::ffi::musk_response_ctor::shim_sse_keep_alive(task, vm)?;
        }
        ("Sse", "into_response") => {
            crate::vm::ffi::musk_response_ctor::shim_sse_into_response(task, vm)?;
        }
        ("KeepAlive", "new") => {
            crate::vm::ffi::musk_response_ctor::shim_keepalive_new(task, vm)?;
        }

        // std::time::Instant(now/elapsed 已迁 plan-430 生成段)

        // std::time::Duration(构造器与 as_secs/as_secs_f64 已迁 plan-430 生成段;
        // 残留 u128 返回的访问器,超 i64 槽暂留手写)
        ("Duration", "as_millis") | ("Duration", "as_micros") | ("Duration", "as_nanos") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(dur) = rust_obj.downcast_ref::<std::time::Duration>() {
                        let val = match method {
                            "as_millis" => dur.as_millis() as i32,
                            "as_micros" => dur.as_micros() as i32,
                            "as_nanos" => dur.as_nanos() as i32,
                            _ => 0,
                        };
                        task.ram.push_i32(val);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }

        // std::path::PathBuf(from 已迁 plan-430 生成段;join 保留手写——
        // std PathBuf::join 经 deref 返回新对象,此臂语义是 push 原地改)
        ("PathBuf", "join") => {
            let other: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("PathBuf.join: {}", e)))?;
            let self_handle = pop_rust_obj(task, vm, "PathBuf.join")?;
            let obj = vm.get_heap_object(self_handle).unwrap();
            let mut guard = obj.write().unwrap();
            if let Some(rust_obj) = guard
                .as_any_mut()
                .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
            {
                if let Some(path) = rust_obj.downcast_mut::<StdPathBuf>() {
                    path.push(&other);
                    task.ram.push_i32(self_handle as i32);
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "PathBuf.join: invalid object at handle {}",
                        self_handle
                    )));
                }
            } else {
                return Err(VMError::RuntimeError(format!(
                    "PathBuf.join: not a RustStdlibObject at handle {}",
                    self_handle
                )));
            }
        }

        // Plan 442 C1 后半程 (musk backend corpus): PathBuf::parent ->
        // Option<PathBuf> — Some 推新句柄,None 推 null(调用方 is Some(p)
        // 匹配形态)。
        ("PathBuf", "parent") => {
            let self_handle = pop_rust_obj(task, vm, "PathBuf.parent")?;
            let parent = vm.get_heap_object(self_handle).and_then(|obj| {
                let guard = obj.read().unwrap();
                guard
                    .as_any()
                    .downcast_ref::<RustStdlibObject>()
                    .and_then(|ro| ro.downcast_ref::<StdPathBuf>())
                    .and_then(|p| p.parent().map(|pp| pp.to_path_buf()))
            });
            match parent {
                Some(p) => push_rust_obj(task, vm, "PathBuf", StdPathBuf::from(p))?,
                None => task.ram.push_nv(auto_val::encode_null()),
            }
        }

        // PathBuf::file_stem -> Option<str> — Some 推池字符串,None 推 null。
        ("PathBuf", "file_stem") => {
            let self_handle = pop_rust_obj(task, vm, "PathBuf.file_stem")?;
            let stem = vm.get_heap_object(self_handle).and_then(|obj| {
                let guard = obj.read().unwrap();
                guard
                    .as_any()
                    .downcast_ref::<RustStdlibObject>()
                    .and_then(|ro| ro.downcast_ref::<StdPathBuf>())
                    .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
            });
            match stem {
                Some(s) => {
                    let idx = vm.add_string(s.into_bytes());
                    vm.rc_push_str_idx(task, idx); // Plan 510 G1-2: 返回串入栈配平
                }
                None => task.ram.push_nv(auto_val::encode_null()),
            }
        }

        // Plan 442 C1 后半程: std::time::SystemTime —— musk 后端的
        // `SystemTime.now().duration_since(SystemTime.UNIX_EPOCH)` 时间戳
        // 形态。Result 在生产者边界坍缩(约定同 env.var);earlier 参数不是
        // 有效 SystemTime 句柄时按 epoch 兜底(UNIX_EPOCH 静态常量在 VM 侧
        // 无独立表示,直呼点推的是占位值)。
        ("SystemTime", "now") => {
            push_rust_obj(task, vm, "std::time::SystemTime", SystemTime::now())?;
        }
        ("SystemTime", "duration_since") => {
            let earlier_nv = task.ram.pop_nv();
            let self_handle = pop_rust_obj(task, vm, "SystemTime.duration_since")?;
            let read_time = |h: u64| -> Option<SystemTime> {
                vm.get_heap_object(h).and_then(|obj| {
                    let guard = obj.read().unwrap();
                    guard
                        .as_any()
                        .downcast_ref::<RustStdlibObject>()
                        .and_then(|ro| ro.downcast_ref::<SystemTime>())
                        .cloned()
                })
            };
            let now = read_time(self_handle).unwrap_or_else(SystemTime::now);
            let earlier = if auto_val::is_i32(earlier_nv) {
                read_time(auto_val::decode_i32(earlier_nv) as u64).unwrap_or(UNIX_EPOCH)
            } else {
                UNIX_EPOCH
            };
            let dur = now.duration_since(earlier).unwrap_or_default();
            push_rust_obj(task, vm, "std::time::Duration", dur)?;
        }

        // std::boxed::Box
        ("Box", "new") => {
            let val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Box.new: {}", e)))?;
            push_rust_obj(task, vm, "Box", val)?;
        }

        // std::cell::RefCell
        ("RefCell", "new") => {
            let val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("RefCell.new: {}", e)))?;
            push_rust_obj(task, vm, "RefCell", val)?;
        }
        ("RefCell", "borrow") | ("RefCell", "borrow_mut") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(val) = rust_obj.downcast_ref::<i32>() {
                        task.ram.push_i32(*val);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("RefCell", "replace") => {
            let new_val: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let mut guard = obj.write().unwrap();
                if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
                    if let Some(val) = rso.downcast_mut::<i32>() {
                        *val = new_val;
                    }
                }
            }
            task.ram.push_i32(handle as i32);
        }

        // std::sync::Arc
        ("Arc", "new") => {
            let val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Arc.new: {}", e)))?;
            push_rust_obj(task, vm, "Arc", val)?;
        }

        // std::sync::Mutex
        ("Mutex", "new") => {
            let val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Mutex.new: {}", e)))?;
            push_rust_obj(task, vm, "Mutex", val)?;
        }

        // std::sync::Arc — single-threaded stub
        ("Arc", "load") => {
            let _ordering = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            // Stub: return the inner value (same handle for single-threaded)
            task.ram.push_i32(handle as i32);
        }
        ("Arc", "fetch_add") => {
            let _ordering = task.ram.pop_i32();
            let _delta = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }

        // ---- chrono ----
        ("Utc", "now") => {
            let now = chrono::Utc::now();
            push_rust_obj(task, vm, "chrono::DateTime<chrono::Utc>", now)?;
        }
        ("Local", "now") => {
            let now = chrono::Local::now();
            push_rust_obj(task, vm, "chrono::DateTime<chrono::Local>", now)?;
        }
        ("Duration", "days") => {
            let days: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Duration.days: {}", e)))?;
            push_rust_obj(task, vm, "Duration", chrono::Duration::days(days as i64))?;
        }
        ("Duration", "hours") => {
            let hours: i64 = i64::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Duration.hours: {}", e)))?;
            push_rust_obj(task, vm, "Duration", chrono::Duration::hours(hours))?;
        }
        ("Duration", "seconds") => {
            let secs: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Duration.seconds: {}", e)))?;
            push_rust_obj(task, vm, "Duration", chrono::Duration::seconds(secs as i64))?;
        }
        ("NaiveDateTime", "parse_from_str") => {
            let fmt: String = String::pop_from_stack(task, vm).map_err(|e| {
                VMError::RuntimeError(format!("NaiveDateTime.parse_from_str: {}", e))
            })?;
            let s: String = String::pop_from_stack(task, vm).map_err(|e| {
                VMError::RuntimeError(format!("NaiveDateTime.parse_from_str: {}", e))
            })?;
            match chrono::NaiveDateTime::parse_from_str(&s, &fmt) {
                Ok(dt) => push_rust_obj(task, vm, "chrono::NaiveDateTime", dt)?,
                Err(e) => {
                    return Err(VMError::RuntimeError(format!(
                        "NaiveDateTime::parse_from_str failed: {}",
                        e
                    )))
                }
            }
        }
        ("Utc", "timestamp_opt") | ("Local", "timestamp_opt") => {
            let ts: i64 = i64::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("timestamp_opt: {}", e)))?;
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0).ok_or_else(|| {
                VMError::RuntimeError("timestamp_opt: invalid timestamp".to_string())
            })?;
            push_rust_obj(task, vm, "chrono::DateTime<chrono::Utc>", dt)?;
        }

        // ---- csv ----
        ("ReaderBuilder", "new") => {
            let builder = csv::ReaderBuilder::new();
            push_rust_obj(task, vm, "csv::ReaderBuilder", builder)?;
        }
        ("ReaderBuilder", "delimiter") => {
            let delim: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("ReaderBuilder.delimiter: {}", e)))?;
            let handle = pop_rust_obj(task, vm, "ReaderBuilder.delimiter")?;
            let obj = vm.get_heap_object(handle).unwrap();
            let mut guard = obj.write().unwrap();
            if let Some(rust_obj) = guard
                .as_any_mut()
                .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
            {
                if let Some(builder) = rust_obj.downcast_mut::<csv::ReaderBuilder>() {
                    builder.delimiter(delim as u8);
                    task.ram.push_i32(handle as i32);
                } else {
                    return Err(VMError::RuntimeError(
                        "ReaderBuilder.delimiter: invalid inner object".into(),
                    ));
                }
            } else {
                return Err(VMError::RuntimeError(
                    "ReaderBuilder.delimiter: invalid object".into(),
                ));
            }
        }
        ("ReaderBuilder", "from_reader") => {
            let reader_handle = pop_rust_obj(task, vm, "ReaderBuilder.from_reader")?;
            let bytes: Vec<u8> = {
                let reader_obj = vm.get_heap_object(reader_handle).unwrap();
                let reader_guard = reader_obj.read().unwrap();
                if let Some(rust_obj) = reader_guard
                    .as_any()
                    .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>(
                ) {
                    rust_obj
                        .downcast_ref::<Vec<u8>>()
                        .cloned()
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            };
            let builder_handle = pop_rust_obj(task, vm, "ReaderBuilder.from_reader")?;
            let obj = vm.get_heap_object(builder_handle).unwrap();
            let guard = obj.read().unwrap();
            if let Some(rust_obj) = guard
                .as_any()
                .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
            {
                if let Some(builder) = rust_obj.downcast_ref::<csv::ReaderBuilder>() {
                    let reader = builder.from_reader(std::io::Cursor::new(bytes));
                    push_rust_obj(task, vm, "csv::Reader<std::io::Cursor<Vec<u8>>>", reader)?;
                } else {
                    return Err(VMError::RuntimeError(
                        "ReaderBuilder.from_reader: invalid inner object".into(),
                    ));
                }
            } else {
                return Err(VMError::RuntimeError(
                    "ReaderBuilder.from_reader: invalid object".into(),
                ));
            }
        }
        ("Reader", "from_reader") => {
            let reader_handle = pop_rust_obj(task, vm, "Reader.from_reader")?;
            let bytes: Vec<u8> = {
                let reader_obj = vm.get_heap_object(reader_handle).unwrap();
                let reader_guard = reader_obj.read().unwrap();
                if let Some(rust_obj) = reader_guard
                    .as_any()
                    .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>(
                ) {
                    rust_obj
                        .downcast_ref::<Vec<u8>>()
                        .cloned()
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            };
            let reader = csv::Reader::from_reader(std::io::Cursor::new(bytes));
            push_rust_obj(task, vm, "csv::Reader", reader)?;
        }
        ("Reader", "records") => {
            let handle = pop_rust_obj(task, vm, "Reader.records")?;
            let Some(obj) = vm.get_heap_object(handle) else {
                return Err(VMError::RuntimeError(
                    "Reader.records: invalid reader handle".into(),
                ));
            };
            // Collect all records into a List of record heap objects
            use crate::vm::types::ListData;
            let mut outer_list: ListData<i32> = ListData::new();
            {
                let mut guard = obj.write().unwrap();
                if let Some(rust_obj) = guard
                    .as_any_mut()
                    .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>(
                ) {
                    if let Some(reader) =
                        rust_obj.downcast_mut::<csv::Reader<std::io::Cursor<Vec<u8>>>>()
                    {
                        for result in reader.records() {
                            if let Ok(record) = result {
                                // Each record becomes a ListData<i32> of string indices
                                let mut record_list: ListData<i32> = ListData::new();
                                for field in record.iter() {
                                    let str_idx = vm.add_string(field.as_bytes().to_vec());
                                    vm.pool_retain(str_idx); // Plan 583 T4: 容器子份额
                                    record_list.push(-(str_idx as i32) - 1); // string encoding convention
                                }
                                let record_id = vm.insert_heap_object(record_list);
                                outer_list.push(record_id as i32);
                            }
                        }
                    }
                }
            }
            let list_id = vm.insert_heap_object(outer_list);
            // Create a List iterator for the for-loop
            let iterator_id = vm.register_iterator(crate::vm::engine::Iterator::List(
                crate::vm::engine::ListIterator {
                    list_id,
                    current_index: 0,
                },
            ));
            task.ram.push_i32(iterator_id as i32);
        }
        ("StringRecord", "get") | ("List", "get") => {
            let index: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("{}.get: {}", type_name, e)))?;
            let handle = pop_rust_obj(task, vm, "List.get")?;
            let Some(obj) = vm.get_heap_object(handle) else {
                return Err(VMError::RuntimeError("List.get: invalid handle".into()));
            };
            let guard = obj.read().unwrap();
            if let Some(list) = guard
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<i32>>()
            {
                let idx = index as usize;
                if let Some(val) = list.get(idx) {
                    let val = *val;
                    if val >= 4000000 {
                        // Heap object ID — push as object reference
                        vm.rc_push(task, auto_val::encode_object(val as u32));
                    } else if let Some(bytes) = vm.get_string(val as u32) {
                        let new_idx = vm.add_string(bytes.to_vec());
                        vm.rc_push_str_idx(task, new_idx as usize);
                    } else {
                        task.ram.push_nv(auto_val::encode_i32(val));
                    }
                } else {
                    // Out of bounds — push None (0)
                    task.ram.push_i32(0);
                }
            } else {
                return Err(VMError::RuntimeError("List.get: not a list".into()));
            }
        }
        ("List", "count") | ("List", "len") => {
            // Plan 057 (Bug 2): accept plain heap ids (raw i32 / TAG_OBJECT)
            // pointing at ListData — bridge-stored array props (ensure_child_state)
            // use this form. Previously only RustStdlibObject handles worked, so
            // `.history.len()` on a prop array errored at pop_rust_obj and
            // silently aborted the enclosing handler.
            let nv = task.ram.pop_nv();
            if auto_val::is_i32(nv) || auto_val::is_object(nv) {
                let id = if auto_val::is_object(nv) {
                    auto_val::decode_object(nv) as u64
                } else {
                    auto_val::decode_i32(nv) as u64
                };
                if let Some(obj) = vm.get_heap_object(id) {
                    let guard = obj.read().unwrap();
                    let len = if let Some(list) = guard
                        .as_any()
                        .downcast_ref::<crate::vm::types::ListData<i32>>()
                    {
                        list.len() as i32
                    } else if let Some(list) = guard
                        .as_any()
                        .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                    {
                        list.elems.len() as i32
                    } else if let Some(list) = guard
                        .as_any()
                        .downcast_ref::<crate::vm::types::ListData<String>>()
                    {
                        list.elems.len() as i32
                    } else {
                        0
                    };
                    task.ram.push_nv(auto_val::encode_i32(len));
                    return Ok(());
                }
            }
            // Fall back to the original RustStdlibObject-handle path (re-push
            // the popped value for pop_rust_obj).
            task.ram.push_nv(nv);
            let handle = pop_rust_obj(task, vm, "List.count")?;
            let Some(obj) = vm.get_heap_object(handle) else {
                return Err(VMError::RuntimeError("List.count: invalid handle".into()));
            };
            let guard = obj.read().unwrap();
            if let Some(list) = guard
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<i32>>()
            {
                let len = list.len() as i32;
                task.ram.push_nv(auto_val::encode_i32(len));
            } else if let Some(list) = guard
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
            {
                let len = list.elems.len() as i32;
                task.ram.push_nv(auto_val::encode_i32(len));
            } else {
                task.ram.push_i32(0);
            }
        }

        // ---- ansi_term ----
        ("Red", "paint")
        | ("Green", "paint")
        | ("Yellow", "paint")
        | ("Blue", "paint")
        | ("Cyan", "paint")
        | ("White", "paint")
        | ("Purple", "paint") => {
            let s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("{}.paint: {}", type_name, e)))?;
            let color = match type_name {
                "Red" => ansi_term::Colour::Red,
                "Green" => ansi_term::Colour::Green,
                "Yellow" => ansi_term::Colour::Yellow,
                "Blue" => ansi_term::Colour::Blue,
                "Cyan" => ansi_term::Colour::Cyan,
                "White" => ansi_term::Colour::White,
                "Purple" => ansi_term::Colour::Purple,
                _ => ansi_term::Colour::Red,
            };
            let str_idx = vm.add_string(color.paint(s).to_string().into_bytes());
            vm.rc_push_str_idx(task, str_idx as usize);
        }

        // ---- std::collections ----
        ("HashMap", "new") => {
            let map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            push_rust_obj(task, vm, "HashMap", map)?;
        }
        ("HashSet", "new") => {
            let set: std::collections::HashSet<String> = std::collections::HashSet::new();
            push_rust_obj(task, vm, "HashSet", set)?;
        }

        // ---- std::fs (module calls) ----
        ("fs", "read_dir") => {
            let path: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("fs.read_dir: {}", e)))?;
            match std::fs::read_dir(&path) {
                Ok(entries) => push_rust_obj(task, vm, "ReadDir", entries)?,
                Err(e) => return Err(VMError::RuntimeError(format!("fs.read_dir: {}", e))),
            }
        }

        // ---- std::thread ----
        ("thread", "available_parallelism") => match std::thread::available_parallelism() {
            Ok(n) => task.ram.push_i32(n.get() as i32),
            Err(_) => task.ram.push_i32(1),
        },

        // ---- std::cell::RefCell (instance methods) ----
        ("cell", "borrow") => {
            task.ram.push_i32(0);
        }
        ("cell", "replace") => {
            let _val: i32 = i32::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("cell.replace: {}", e)))?;
            let _handle = pop_rust_obj(task, vm, "cell.replace")?;
            task.ram.push_i32(0);
        }

        // ---- heapless::Vec ----
        ("Vec", "new") => {
            push_rust_obj(task, vm, "heapless::Vec", Vec::<u8>::new())?;
        }

        // ---- urlencoding ----
        ("", "encode") => {
            let s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("urlencoding.encode: {}", e)))?;
            let str_idx = vm.add_string(urlencoding::encode(&s).as_bytes().to_vec());
            vm.rc_push_str_idx(task, str_idx as usize);
        }

        // ---- log macros (no-op in VM) ----
        ("", "info") | ("", "debug") | ("", "warn") | ("", "error") | ("", "trace") => {
            let _msg: Result<String, _> = String::pop_from_stack(task, vm);
        }

        // ---- serde_json (handled below with better implementation) ----

        // ---- toml ----
        ("toml", "from_str") => {
            let _s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("toml.from_str: {}", e)))?;
            let str_idx = vm.add_string(b"{}".to_vec());
            vm.rc_push_str_idx(task, str_idx as usize);
        }

        // ---- Path ----
        ("Path", "new") => {
            let _s: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Path.new: {}", e)))?;
            push_rust_obj(task, vm, "PathBuf", StdPathBuf::new())?;
        }

        // ---- same_file (Plan 267 Phase C) ----
        ("same_file", "is_same_file") => {
            let path2: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let path1: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let same = same_file::is_same_file(&path1, &path2).unwrap_or(false);
            task.ram.push_nv(auto_val::encode_bool(same));
        }

        // ---- String ----
        ("String", "new") => {
            let str_idx = vm.add_string(b"".to_vec());
            vm.rc_push_str_idx(task, str_idx as usize);
        }

        // ---- File ----
        ("File", "create") => {
            let path: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("File.create: {}", e)))?;
            match std::fs::File::create(&path) {
                Ok(f) => push_rust_obj(task, vm, "FileWriter", f)?,
                Err(e) => return Err(VMError::RuntimeError(format!("File.create: {}", e))),
            }
        }
        ("File", "open") => {
            let path: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("File.open: {}", e)))?;
            match std::fs::File::open(&path) {
                Ok(f) => push_rust_obj(task, vm, "FileWriter", f)?,
                Err(e) => return Err(VMError::RuntimeError(format!("File.open: {}", e))),
            }
        }

        // ---- Plan 051 T10: Sse 显式降级 + Date.now ----
        // VM 轨无 SSE 形态（KD-047 G1 阶段2 债）：Sse.open/close 以 no-op
        // native 显式降级——返回 None 句柄，handler 不再因缺 native 中止
        // （musk StartStream 的乐观 push 因此得以执行；流式刷新由
        // PollStream timer 轮询承担，见 PLAN-051 T10）。
        ("Sse", "open") => {
            // 形参（url + handler）按调用约定弹出后丢弃；返回 0 作句柄令牌
            // （消费方仅存取不判别）。
            let _url: String = String::pop_from_stack(task, vm).unwrap_or_default();
            drop(i64::pop_from_stack(task, vm).unwrap_or(0));
            task.ram.push_i32(0);
        }
        ("Sse", "close") => {
            drop(i64::pop_from_stack(task, vm).unwrap_or(0));
            task.ram.push_i32(0);
        }
        // Date.now：epoch 毫秒（musk msgTimeLabel/乐观 push 时间戳消费）。
        ("Date", "now") => {
            let ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            task.ram.push_i64(ms);
        }
        // PLAN-054 T5 (A7): Date.format(epoch_ms, "HH:mm:ss") 宿主桥——
        // KNOWN-DEBT 051 登记的"Date.format 富格式上游债"收口。musk
        // msgTimeLabel(forge_helpers.at)经此产出本地时区 HH:MM:SS 时间标签,
        // 对齐 web 轨 toLocaleTimeString 形态。token 支持最小集(未知原样):
        // yyyy/MM/dd/HH/H/mm/m/ss/s/SSS。
        ("Date", "format") => {
            let pattern: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let ms: i64 = i64::pop_from_stack(task, vm).unwrap_or(0);
            let label = format_date_ms(ms, &pattern);
            let idx = vm.add_string(label.into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
        }

        // ---- Stdio ----
        ("Stdio", "piped") => {
            task.ram.push_i32(0);
        }
        ("Stdio", "from") => {
            let _fd: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            task.ram.push_i32(0);
        }

        // ---- WriteLogger (simplelog) ----
        ("WriteLogger", "init") => {
            let _file: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let _level: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let _config: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            task.ram.push_i32(0);
        }
        ("LevelFilter", "Info") => {
            task.ram.push_i32(0);
        }
        ("Config", "default") => {
            task.ram.push_i32(0);
        }

        // ---- Backtrace ----
        ("Backtrace", "capture") => {
            let bt = std::backtrace::Backtrace::capture();
            let handle = vm.insert_heap_object(RustStdlibObject::new("Backtrace", bt)) as i32;
            vm.rc_push_id(task, handle as u64); // Plan 419
        }

        // ---- percent_encoding ----
        ("percent_encoding", "NON_ALPHANUMERIC") => {
            // Sentinel value indicating non-alphanumeric encoding
            push_rust_obj(task, vm, "EncodeSet", "NON_ALPHANUMERIC")?;
        }

        // ---- clap Args ----
        ("Args", "parse") => {
            task.ram.push_i32(0);
        }

        // ---- serde_json ----
        ("serde_json", "from_str") => {
            let json_str: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("serde_json.from_str: {}", e)))?;
            // For MVP, store the raw JSON string as a RustStdlibObject
            push_rust_obj(task, vm, "serde_json::Value", json_str)?;
        }
        ("serde_json", "to_string") => {
            let _value: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("serde_json.to_string: {}", e)))?;
            // For MVP, return the value as-is (no actual serialization)
            let s = _value;
            let str_idx = vm.add_string(s.into_bytes());
            vm.rc_push_str_idx(task, str_idx as usize);
        }

        // ---- chrono DateTime methods ----
        ("DateTime", "single") => {
            // .single() on MappedTemporalError returns Option<DateTime>
            // For VM, just pass through the DateTime that was already constructed
            // Pop the receiver (DateTime handle) and push it back
            let handle = pop_rust_obj(task, vm, "DateTime.single")?;
            task.ram.push_nv(auto_val::encode_i32(handle as i32));
        }

        // ---- regex extra methods ----
        ("Regex", "captures_iter") => {
            let text: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Regex.captures_iter: {}", e)))?;
            let handle = pop_rust_obj(task, vm, "Regex.captures_iter")?;
            let obj = vm.get_heap_object(handle);
            if obj.is_none() {
                return Err(VMError::RuntimeError(
                    "Regex.captures_iter: invalid handle".into(),
                ));
            }
            // Collect all matches as string pool indices in a flat list
            use crate::vm::types::ListData;
            let mut match_list: ListData<i32> = ListData::new();
            if let Some(obj) = obj {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(re) = rust_obj.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                        for mat in re.lock().unwrap().find_iter(&text) {
                            let str_idx = vm.add_string(mat.as_str().as_bytes().to_vec());
                            vm.pool_retain(str_idx); // Plan 583 T4: 容器子份额
                            match_list.push(-(str_idx as i32) - 1); // string encoding convention
                        }
                    }
                }
            }
            let list_id = vm.insert_heap_object(match_list);
            vm.rc_push(task, auto_val::encode_object(list_id as u32));
        }

        // ---- percent_encoding ----
        ("", "percent_encode") => {
            let _encode_set_handle = task.ram.pop_i32();
            let bytes_handle = pop_rust_obj(task, vm, "percent_encode")?;
            let Some(obj) = vm.get_heap_object(bytes_handle) else {
                let str_idx = vm.add_string(Vec::new());
                vm.rc_push_str_idx(task, str_idx as usize);
                return Ok(());
            };
            let guard = obj.read().unwrap();
            let bytes = if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                rust_obj
                    .downcast_ref::<Vec<u8>>()
                    .cloned()
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            drop(guard);

            // Percent-encode non-alphanumeric bytes
            let encoded: String = bytes
                .iter()
                .flat_map(|&b| {
                    if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~'
                    {
                        vec![b]
                    } else {
                        format!("%{:02X}", b).into_bytes()
                    }
                })
                .map(|b| b as char)
                .collect();

            let str_idx = vm.add_string(encoded.into_bytes());
            vm.rc_push_str_idx(task, str_idx as usize);
        }

        // ---- Vec<u8> methods (for as_bytes results) ----
        ("Vec<u8>", "len") | ("Vec", "len") | ("buf", "len") => {
            let handle = pop_rust_obj(task, vm, "Vec.len")?;
            let Some(obj) = vm.get_heap_object(handle) else {
                task.ram.push_i32(0);
                return Ok(());
            };
            let guard = obj.read().unwrap();
            if let Some(rust_obj) = guard
                .as_any()
                .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
            {
                if let Some(bytes) = rust_obj.downcast_ref::<Vec<u8>>() {
                    task.ram.push_i32(bytes.len() as i32);
                } else {
                    task.ram.push_i32(0);
                }
            } else {
                task.ram.push_i32(0);
            }
        }

        // ---- csv::Writer ----
        ("Writer", "from_writer") => {
            let _inner_handle = task.ram.pop_i32();
            let buffer: Vec<u8> = Vec::new();
            let writer = csv::Writer::from_writer(buffer);
            push_rust_obj(
                task,
                vm,
                "csv::Writer<Vec<u8>>",
                std::sync::Mutex::new(writer),
            )?;
        }
        ("Writer", "write_record") => {
            let record: Vec<String> = Vec::<String>::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Writer.write_record: {}", e)))?;
            let writer_handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(writer_handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(writer) =
                        rust_obj.downcast_ref::<std::sync::Mutex<csv::Writer<Vec<u8>>>>()
                    {
                        let _ = writer.lock().unwrap().write_record(&record);
                        task.ram.push_i32(writer_handle as i32);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("Writer", "serialize") => {
            // Pop object handle (ObjectData lives in heap_objects, H3b)
            let obj_nv = task.ram.pop_nv();
            let obj_id = if auto_val::is_object(obj_nv) {
                auto_val::decode_object(obj_nv) as u64
            } else {
                auto_val::decode_i32(obj_nv) as u64
            };
            let writer_handle = task.ram.pop_i32() as u64;
            // Extract fields from ObjectData and serialize as CSV record
            let mut record_map: std::collections::BTreeMap<String, String> =
                std::collections::BTreeMap::new();
            if let Some(obj_arc) = vm.get_heap_object(obj_id) {
                let obj_guard = obj_arc.read().unwrap();
                if let Some(od) = obj_guard
                    .as_any()
                    .downcast_ref::<crate::vm::types::ObjectData>()
                {
                    for (key, val) in &od.fields {
                        let key_str = format!("{:?}", key);
                        let val_str = match val {
                            auto_val::Value::Str(s) => s.to_string(),
                            auto_val::Value::Int(n) => n.to_string(),
                            auto_val::Value::Float(f) => f.to_string(),
                            auto_val::Value::Bool(b) => b.to_string(),
                            _ => format!("{:?}", val),
                        };
                        record_map.insert(key_str, val_str);
                    }
                }
            }
            let record: Vec<String> = record_map.into_values().collect();
            if let Some(obj) = vm.get_heap_object(writer_handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(writer) =
                        rust_obj.downcast_ref::<std::sync::Mutex<csv::Writer<Vec<u8>>>>()
                    {
                        let _ = writer.lock().unwrap().write_record(&record);
                        task.ram.push_i32(writer_handle as i32);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("Writer", "flush") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("Writer", "into_inner") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(writer) =
                        rust_obj.downcast_ref::<std::sync::Mutex<csv::Writer<Vec<u8>>>>()
                    {
                        let mut w = writer.lock().unwrap();
                        let taken =
                            std::mem::replace(&mut *w, csv::Writer::from_writer(Vec::new()));
                        let inner = taken.into_inner().unwrap_or_default();
                        push_rust_obj(task, vm, "Vec<u8>", inner)?;
                        return Ok(());
                    }
                }
            }
            push_rust_obj(task, vm, "Vec<u8>", Vec::<u8>::new())?;
        }

        // ---- String ----
        ("String", "from_utf8") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(bytes) = rust_obj.downcast_ref::<Vec<u8>>() {
                        match String::from_utf8(bytes.clone()) {
                            Ok(s) => {
                                let str_idx = vm.add_string(s.into_bytes());
                                vm.rc_push_str_idx(task, str_idx as usize);
                                return Ok(());
                            }
                            Err(_) => {
                                return Err(VMError::RuntimeError(
                                    "String.from_utf8: invalid utf8".into(),
                                ));
                            }
                        }
                    }
                }
            }
            task.ram.push_i32(0);
        }

        // ---- std::process::Command ----
        ("Command", "new") => {
            let program: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Command.new: {}", e)))?;
            let cmd = std::process::Command::new(&program);
            push_rust_obj(
                task,
                vm,
                "std::process::Command",
                std::sync::Mutex::new(cmd),
            )?;
        }
        ("Command", "arg") => {
            let arg: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Command.arg: {}", e)))?;
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(cmd) =
                        rust_obj.downcast_ref::<std::sync::Mutex<std::process::Command>>()
                    {
                        cmd.lock().unwrap().arg(&arg);
                        task.ram.push_i32(handle as i32);
                        return Ok(());
                    }
                }
            }
            return Err(VMError::RuntimeError(
                "Command.arg: invalid Command handle".into(),
            ));
        }
        ("Command", "args") => {
            // Pop the args array as Vec<String> (same extraction path as
            // shim_process_spawn's Vec<String> parameter via VMConvertible).
            let args: Vec<String> = Vec::<String>::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("Command.args: {}", e)))?;
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(cmd) =
                        rust_obj.downcast_ref::<std::sync::Mutex<std::process::Command>>()
                    {
                        let mut cmd_guard = cmd.lock().unwrap();
                        cmd_guard.args(&args);
                        task.ram.push_i32(handle as i32);
                        return Ok(());
                    }
                }
            }
            return Err(VMError::RuntimeError(
                "Command.args: invalid Command handle".into(),
            ));
        }
        ("Command", "stdout") => {
            let _stdio_val = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("Command", "stdin") => {
            let _stdio_val = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("Command", "stderr") => {
            let _stdio_val = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("Command", "output") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(cmd) =
                        rust_obj.downcast_ref::<std::sync::Mutex<std::process::Command>>()
                    {
                        let output =
                            cmd.lock().unwrap().output().map_err(|e| {
                                VMError::RuntimeError(format!("Command.output: {}", e))
                            })?;
                        push_rust_obj(task, vm, "std::process::Output", output)?;
                        return Ok(());
                    }
                }
            }
            return Err(VMError::RuntimeError(
                "Command.output: invalid Command handle".into(),
            ));
        }
        ("Command", "spawn") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(cmd) =
                        rust_obj.downcast_ref::<std::sync::Mutex<std::process::Command>>()
                    {
                        let child =
                            cmd.lock().unwrap().spawn().map_err(|e| {
                                VMError::RuntimeError(format!("Command.spawn: {}", e))
                            })?;
                        push_rust_obj(task, vm, "std::process::Child", child)?;
                        return Ok(());
                    }
                }
            }
            return Err(VMError::RuntimeError(
                "Command.spawn: invalid Command handle".into(),
            ));
        }
        ("Command", "status") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        // Child process
        ("Child", "stdin") | ("Child", "stdout") | ("Child", "stderr") => {
            let _child_handle = task.ram.pop_i32();
            task.ram.push_i32(0);
        }
        ("Child", "wait") | ("Child", "wait_with_output") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(_child) = rust_obj.downcast_ref::<std::process::Child>() {
                        // stub: push 0 as exit code
                        task.ram.push_i32(0);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("Child", "kill") => {
            let _handle = task.ram.pop_i32();
            task.ram.push_i32(0);
        }
        // Normal distribution (rand_distr)
        ("Normal", "new") => {
            let _stddev: f64 = f64::pop_from_stack(task, vm).unwrap_or(1.0);
            let _mean: f64 = f64::pop_from_stack(task, vm).unwrap_or(0.0);
            push_rust_obj(task, vm, "Normal", 0i32)?;
        }

        // Complex (num::Complex) — stub with (real, imag) stored as (f64, f64)
        ("Complex", "new") => {
            let imag: f64 = f64::pop_from_stack(task, vm).unwrap_or(0.0);
            let real: f64 = f64::pop_from_stack(task, vm).unwrap_or(0.0);
            push_rust_obj(task, vm, "Complex", (real, imag))?;
        }
        ("Complex", "norm") | ("Complex", "arg") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some((real, imag)) = rust_obj.downcast_ref::<(f64, f64)>() {
                        let val = if method == "norm" {
                            (real * real + imag * imag).sqrt()
                        } else {
                            imag.atan2(*real)
                        };
                        task.ram.push_f64(val);
                        return Ok(());
                    }
                }
            }
            task.ram.push_f64(0.0);
        }
        // BigInt (num-bigint)
        ("BigInt", "from") | ("BigInt", "new") => {
            let _val: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            push_rust_obj(task, vm, "BigInt", 0i64)?;
        }
        // ThreadRng
        ("ThreadRng", "sample") => {
            let _handle = task.ram.pop_i32();
            push_rust_obj(task, vm, "ThreadRng", 0i32)?;
        }
        // WalkDir (walkdir crate) — Plan 267 Phase A
        ("WalkDir", "new") => {
            let root: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("WalkDir.new: {}", e)))?;
            let mut paths = Vec::new();
            for entry in walkdir::WalkDir::new(&root)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                paths.push(entry.path().display().to_string());
            }
            let json = serde_json::to_string(&paths)
                .map_err(|e| VMError::RuntimeError(format!("WalkDir.new json: {}", e)))?;
            let str_idx = vm.add_string(json.into_bytes());
            vm.rc_push_str_idx(task, str_idx as usize);
        }
        ("WalkDir", "into_iter") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("DirEntry", "path") | ("DirEntry", "file_name") => {
            let _handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(0);
        }
        ("Result<DirEntry, walkdir::Error>", "unwrap") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        ("walkdir::Error", "to_string") => {
            let _handle = task.ram.pop_i32() as u64;
            let str_idx = vm.add_string(b"io error".to_vec());
            vm.rc_push_str_idx(task, str_idx as usize);
        }
        // TarGzip builder methods — Plan 267 Phase B
        ("Builder", "append_path") => {
            let _path: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32); // return self for chaining
        }
        ("Builder", "append_dir_all") => {
            let _dest: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let _src: String = String::pop_from_stack(task, vm).unwrap_or_default();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32); // return self
        }
        // same_file::Handle — Plan 267 Phase C
        ("Handle", "from_path") => {
            let _path: String = String::pop_from_stack(task, vm).unwrap_or_default();
            push_rust_obj(task, vm, "same_file::Handle", 0i32)?;
        }

        // ---- std::sync::Arc clone stub ----
        ("Arc", "clone") => {
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        // ---- std::thread stubs ----
        ("thread", "spawn") => {
            let _closure = task.ram.pop_i32();
            push_rust_obj(task, vm, "std::thread::JoinHandle", 0i32)?;
        }
        ("JoinHandle", "join") => {
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(result) = rust_obj.downcast_ref::<std::sync::Mutex<i32>>() {
                        let val = result.lock().unwrap();
                        task.ram.push_i32(*val);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        // ---- crossbeam stubs ----
        // ("", "unbounded") — module-level call, type_name is empty
        ("", "unbounded") => {
            // crossbeam::channel::unbounded() — stub: push dummy channel handle
            push_rust_obj(task, vm, "crossbeam::channel::unbounded", 0i32)?;
        }
        ("crossbeam", "scope") => {
            // crossbeam::scope — stub: pop closure, push 0
            let _closure = task.ram.pop_i32();
            task.ram.push_i32(0);
        }
        // ---- std::sync::atomic stubs ----
        ("AtomicUsize", "new") => {
            let _val: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            push_rust_obj(
                task,
                vm,
                "AtomicUsize",
                std::sync::atomic::AtomicUsize::new(0),
            )?;
        }
        ("AtomicUsize", "fetch_add") => {
            let _ordering: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(atom) = rust_obj.downcast_ref::<std::sync::atomic::AtomicUsize>() {
                        let old = atom.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        task.ram.push_i32(old as i32);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        ("AtomicUsize", "load") => {
            let _ordering: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(atom) = rust_obj.downcast_ref::<std::sync::atomic::AtomicUsize>() {
                        let val = atom.load(std::sync::atomic::Ordering::SeqCst);
                        task.ram.push_i32(val as i32);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        // ---- env_logger::Builder stub ----
        ("Builder", "new") => {
            push_rust_obj(task, vm, "env_logger::Builder", 0i32)?;
        }
        // Builder.format(closure) — noop, accepts closure arg, returns self
        ("Builder", "format") => {
            let _closure = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            // Return same builder handle (chainable)
            task.ram.push_i32(handle as i32);
        }
        // Builder.init() — noop, consumes builder, pushes unit
        ("Builder", "init") => {
            let _handle = task.ram.pop_i32();
            // env_logger::init returns (), push nothing meaningful
        }
        // ---- log stubs ----
        ("log", "set_boxed_logger") => {
            let _logger = task.ram.pop_i32();
            task.ram.push_i32(0); // Ok(())
        }
        ("log", "set_max_level") => {
            let _level = task.ram.pop_i32();
        }
        // ---- heapless::Vec stub ----
        // ("Vec", "new") and ("Vec", "len") are handled by existing handlers above
        ("heapless::Vec", "push") | ("Vec", "push") => {
            let _byte: i32 = i32::pop_from_stack(task, vm).unwrap_or(0);
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let mut guard = obj.write().unwrap();
                if let Some(rust_obj) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
                    if let Some(vec) = rust_obj.downcast_mut::<Vec<u8>>() {
                        let _ = vec.push(_byte as u8);
                        task.ram.push_i32(0); // Ok(())
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(0);
        }
        // ---- semver::VersionReq stub ----
        ("VersionReq", "parse") => {
            let req_str: String = String::pop_from_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(format!("VersionReq.parse: {}", e)))?;
            match semver::VersionReq::parse(&req_str) {
                Ok(req) => {
                    push_rust_obj(task, vm, "semver::VersionReq", req)?;
                }
                Err(e) => {
                    return Err(VMError::RuntimeError(format!(
                        "VersionReq::parse failed: {}",
                        e
                    )));
                }
            }
        }
        ("VersionReq", "matches") => {
            let _version_handle = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(_req) = rust_obj.downcast_ref::<semver::VersionReq>() {
                        // Stub: always return true
                        task.ram.push_i32(1);
                        return Ok(());
                    }
                }
            }
            task.ram.push_i32(1);
        }
        // ---- chrono DateTime/NaiveDateTime.checked_add_signed stub ----
        ("DateTime", "checked_add_signed") | ("NaiveDateTime", "checked_add_signed") => {
            let _duration_handle = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    if let Some(dt) =
                        rust_obj.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>()
                    {
                        let dt = dt.lock().unwrap();
                        let result = dt.checked_add_signed(chrono::Duration::days(30));
                        match result {
                            Some(new_dt) => {
                                push_rust_obj(task, vm, "chrono::NaiveDateTime", new_dt)?;
                                return Ok(());
                            }
                            None => {
                                task.ram.push_i32(0);
                                return Ok(());
                            }
                        }
                    }
                    if let Some(dt) =
                        rust_obj.downcast_ref::<std::sync::Mutex<chrono::DateTime<chrono::Utc>>>()
                    {
                        let dt = dt.lock().unwrap();
                        let result = dt.checked_add_signed(chrono::Duration::days(30));
                        match result {
                            Some(new_dt) => {
                                push_rust_obj(task, vm, "chrono::DateTime<chrono::Utc>", new_dt)?;
                                return Ok(());
                            }
                            None => {
                                task.ram.push_i32(0); // None
                                return Ok(());
                            }
                        }
                    }
                }
            }
            task.ram.push_i32(0);
        }
        // ---- flate2::GzDecoder stub ----
        ("GzDecoder", "new") => {
            let _file_handle = task.ram.pop_i32();
            push_rust_obj(task, vm, "flate2::GzDecoder", 0i32)?;
        }
        // ---- tar::Archive stub ----
        ("Archive", "new") => {
            let _gz_handle = task.ram.pop_i32();
            push_rust_obj(task, vm, "tar::Archive", 0i32)?;
        }
        ("Archive", "set_prefix_strip") | ("Archive", "entries") => {
            let _arg = task.ram.pop_i32();
            let handle = task.ram.pop_i32() as u64;
            task.ram.push_i32(handle as i32);
        }
        _ => {
            // Fallback: check opaque dispatch table for native shim routing
            if let Some(native_name) =
                crate::vm::native_catalog::lookup_opaque_dispatch_by_type(&type_name, &method)
            {
                let name_owned = native_name.to_string();
                let native_id = {
                    let mut reg = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
                    reg.resolve_qualified(&name_owned).or_else(|| {
                        reg.resolve_qualified(&format!(
                            "auto.{}.{}",
                            type_name.to_lowercase().replace("::", "."),
                            method
                        ))
                    })
                };
                if let Some(id) = native_id {
                    if let Some(shim) = vm.native_interface.get(id).cloned() {
                        shim(task, vm)?;
                        return Ok(());
                    }
                }
            }
            // Plan 430 C2: 三方 crate 方法 shim 包(dep 声明 + use.rust 导入的类型;
            // 生成段/手写臂/native_catalog 均未命中后的最后一段)。
            // PLAN-596 T4:mono 实例后缀键 miss 时剥 "__<label>" 重试——codegen 对
            // dep 侧齐整字面量调用统一加后缀,非泛型方法靠此回退到裸名条目。
            if crate::vm::ffi::dep_methods::dispatch(&type_name, &method, task, vm)? {
                return Ok(());
            }
            return Err(VMError::RuntimeError(format!(
                "Unknown Rust stdlib call: {type_name}.{method}"
            )));
        }
    }
    Ok(())
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_rust_stdlib_dispatch_shims(
    natives: &mut crate::vm::native::NativeInterface,
) {
    // Plan 192: Rust stdlib dynamic dispatch (manual — uses heap objects)
    natives.register_shim_by_name("auto.rust_stdlib.dispatch", shim_rust_stdlib_dispatch);
}
