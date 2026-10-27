//! PLAN-843 结构拆分：native web 域实现（自 native.rs 原样搬移）。
use super::*;

/// Url.parse(url_str) → opaque handle
/// Stack: [url_str] -> [handle_i32]
pub fn shim_url_opaque_parse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_str = pop_vm_string(task, vm);
    match url::Url::parse(&url_str) {
        Ok(url) => {
            let obj = RustStdlibObject::new("url::Url", std::sync::Mutex::new(url));
            let id = vm.insert_heap_object(obj);
            vm.rc_push_id(task, id as u64); // Plan 419
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("Url::parse failed: {}", e)));
        }
    }
    Ok(())
}

/// url.scheme() → string
/// Stack: [handle_i32] -> [scheme_str]
pub fn shim_url_opaque_scheme(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                push_vm_string(task, vm, url.lock().unwrap().scheme());
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.host_str() → string or empty
/// Stack: [handle_i32] -> [host_str]
pub fn shim_url_opaque_host_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let locked = url.lock().unwrap();
                let s = locked.host_str().unwrap_or("").to_string();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.path() → string
/// Stack: [handle_i32] -> [path_str]
pub fn shim_url_opaque_path(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                push_vm_string(task, vm, url.lock().unwrap().path());
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.fragment() → string or empty
/// Stack: [handle_i32] -> [fragment_str]
pub fn shim_url_opaque_fragment(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let locked = url.lock().unwrap();
                let s = locked.fragment().unwrap_or("").to_string();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.port() → int (0 if none)
/// Stack: [handle_i32] -> [port_i32]
pub fn shim_url_opaque_port(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let port = url.lock().unwrap().port().unwrap_or(0) as i32;
                task.ram.push_i32(port);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// url.query_pairs() → List of "key=value" strings
/// Stack: [handle_i32] -> [list_id_i32]
pub fn shim_url_opaque_query_pairs(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use crate::vm::types::ListData;

    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);
    let mut pairs: Vec<i32> = Vec::new();

    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                for (k, v) in url.lock().unwrap().query_pairs() {
                    let pair = format!("{}={}", k, v);
                    let idx = vm.add_string(pair.as_bytes().to_vec()) as u32;
                    pairs.push(-(idx as i32) - 1);
                }
            }
        }
    }

    let mut list = ListData::<i32>::new();
    list.elems = pairs;
    let id = vm.insert_heap_object(list);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// url.query() → raw query string (without ?)
/// Stack: [handle_i32] -> [query_str]
pub fn shim_url_opaque_query(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let s = url.lock().unwrap().query().unwrap_or("").to_string();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.to_string() → full URL string
/// Stack: [handle_i32] -> [url_str]
pub fn shim_url_opaque_to_string(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let s = url.lock().unwrap().to_string();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.join(relative) → new opaque handle
/// Stack: [relative_str, handle_i32] -> [new_handle_i32]
pub fn shim_url_opaque_join(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let relative = pop_vm_string(task, vm);
    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                match url.lock().unwrap().join(&relative) {
                    Ok(joined) => {
                        let new_obj =
                            RustStdlibObject::new("url::Url", std::sync::Mutex::new(joined));
                        let id = vm.insert_heap_object(new_obj);
                        vm.rc_push_id(task, id as u64); // Plan 419
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(VMError::RuntimeError(format!("Url::join failed: {}", e)));
                    }
                }
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// url.origin() → string
/// Stack: [handle_i32] -> [origin_str]
pub fn shim_url_opaque_origin(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_url_id = crate::vm::native::StakeGuard::new(vm, url_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let origin = url.lock().unwrap().origin().ascii_serialization();
                push_vm_string(task, vm, &origin);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.drop() — no-op, GC handles cleanup
pub fn shim_url_opaque_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_id = crate::vm::native::StakeGuard::new(vm, id as i64 as u64);
    vm.rc_release_id(id);
    Ok(())
}

/// `url.encode(s) -> percent-encoded string` — also serves the JS-compat
/// bare call `encodeURIComponent(s)` via the codegen intrinsics alias.
/// Stack: s -> encoded.
pub fn shim_url_encode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let s_nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, s_nv);
        let s = if auto_val::is_string(s_nv) {
            let idx = auto_val::decode_string(s_nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(s_nv).to_string()
        };
        let encoded = urlencoding::encode(&s).to_string();
        let idx = vm.add_string(encoded.into_bytes());
        vm.rc_push_str_idx(task, idx); // Plan 510 G1-2: 返回串入栈配平
    }
    Ok(())
}

/// `auto.url.encode_path(s)` — percent-encode each `/`-separated segment,
/// preserving the separators. Client parity for axum `{*path}` wildcards
/// (plan-022 auto-down): the browser keeps `/` raw in fetch URLs so the
/// server-side per-segment percent-decode (5441dda28) reconstructs the
/// path; a full `auto.url.encode` would send `%2F`, which wildcard
/// captures hand back still-encoded.
/// Stack: s -> encoded.
pub fn shim_url_encode_path(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let s_nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, s_nv);
        let s = if auto_val::is_string(s_nv) {
            let idx = auto_val::decode_string(s_nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(s_nv).to_string()
        };
        let encoded = s
            .split('/')
            .map(|seg| urlencoding::encode(seg).to_string())
            .collect::<Vec<_>>()
            .join("/");
        let idx = vm.add_string(encoded.into_bytes());
        vm.rc_push_str_idx(task, idx); // Plan 510 G1-2: 返回串入栈配平
    }
    Ok(())
}

/// `localStorage.getItem(key) -> stored string or None`
/// Stack: key -> value|None (None for a missing key — musk's
/// `saved != None` checks depend on it; session-scoped like storage.*).
pub fn shim_localstorage_get_item(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let key_nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, key_nv);
        let key = if auto_val::is_string(key_nv) {
            let idx = auto_val::decode_string(key_nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(key_nv).to_string()
        };
        match crate::vm::ffi::stdlib::storage_raw_get(&key) {
            Some(v) => {
                let idx = vm.add_string(v.into_bytes());
                vm.rc_push_str_idx(task, idx); // Plan 510 G1-2: 返回串入栈配平
            }
            None => {
                // Same encoding the None literal compiles to (PUSH_NIL).
                task.ram.push_nv(auto_val::encode_null());
            }
        }
    }
    Ok(())
}

/// `localStorage.setItem(key, value)` — Stack: key, value -> (void).
/// Non-string values are stringified (session persistence only).
pub fn shim_localstorage_set_item(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let val_nv = crate::vm::native::pop_arg_nv(task);
        let _stake_val = crate::vm::native::StakeGuard::nv(vm, val_nv);
        let key_nv = crate::vm::native::pop_arg_nv(task);
        let _stake_key = crate::vm::native::StakeGuard::nv(vm, key_nv);
        let key = if auto_val::is_string(key_nv) {
            let idx = auto_val::decode_string(key_nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(key_nv).to_string()
        };
        let value = if auto_val::is_string(val_nv) {
            let idx = auto_val::decode_string(val_nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else if auto_val::is_i32(val_nv) {
            auto_val::decode_i32(val_nv).to_string()
        } else if auto_val::is_f64(val_nv) {
            format!("{}", auto_val::decode_f64(val_nv))
        } else if auto_val::is_bool(val_nv) {
            auto_val::decode_bool(val_nv).to_string()
        } else {
            format!("{:?}", val_nv)
        };
        crate::vm::ffi::stdlib::storage_raw_set(key, value);
    }
    Ok(())
}

/// `localStorage.removeItem(key)` — Stack: key -> (void).
pub fn shim_localstorage_remove_item(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let key_nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, key_nv);
        let key = if auto_val::is_string(key_nv) {
            let idx = auto_val::decode_string(key_nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(key_nv).to_string()
        };
        crate::vm::ffi::stdlib::storage_raw_remove(&key);
    }
    Ok(())
}

/// `dom.set_dark(on)` — Stack: on -> (void). Renderer theme is dark-fixed;
/// the intent is recorded in the session KV for a future light theme.
pub fn shim_dom_set_dark(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let on_nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, on_nv);
        crate::vm::ffi::stdlib::storage_raw_set(
            "auto.dom.dark".to_string(),
            if auto_val::decode_bool(on_nv) {
                "1".to_string()
            } else {
                "0".to_string()
            },
        );
    }
    Ok(())
}

/// `dom.prefers_dark()` — Stack: -> bool. The iced VM renderer is dark-only
/// (renderer.rs theme_fn returns Theme::Dark), so report true.
pub fn shim_dom_prefers_dark(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    task.ram.push_nv(auto_val::encode_bool(true));
    Ok(())
}

/// `dom.set_css_var(name, value)` — Stack: name, value -> (void). CSS custom
/// properties are a web rendering concept; record and drop on desktop.
pub fn shim_dom_set_css_var(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let _val_nv = crate::vm::native::pop_arg_nv(task);
        let name_nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, name_nv);
    }
    Ok(())
}

/// `dom.focus_first(selector)` — Stack: sel -> (void). Keyboard focus routing
/// differs on desktop; no-op stub (web 走 ts_adapter querySelector 真聚焦；
/// 桌面聚焦合同归 `ui.focus`，见下）。
pub fn shim_dom_focus_first(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, nv);
    }
    Ok(())
}

/// `dom.click_first(selector)` — Stack: sel -> (void). No-op stub.
pub fn shim_dom_click_first(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, nv);
    }
    Ok(())
}

/// `dom.open_url(url)` — Stack: url -> (void). Opens the OS browser
/// (detached spawn; failure is non-fatal).
pub fn shim_dom_open_url(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let url_nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, url_nv);
        let url = if auto_val::is_string(url_nv) {
            let idx = auto_val::decode_string(url_nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(url_nv).to_string()
        };
        if !url.is_empty() {
            let r = if cfg!(target_os = "windows") {
                std::process::Command::new("cmd")
                    .args(["/C", "start", "", &url])
                    .spawn()
            } else if cfg!(target_os = "macos") {
                std::process::Command::new("open").arg(&url).spawn()
            } else {
                std::process::Command::new("xdg-open").arg(&url).spawn()
            };
            if let Err(e) = r {
                eprintln!("[VM] dom.open_url('{}') failed: {}", url, e);
            }
        }
    }
    Ok(())
}

/// `location.reload()` — Stack: -> (void). Browser page reload has no direct
/// desktop equivalent (would be an app restart); no-op stub.
pub fn shim_dom_reload(_task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    Ok(())
}
