//! PLAN-843 结构拆分：image 域实现（原样搬移，算法/行为零变化）。
use super::*;

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_queue(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let ticket = auto_media_core::image_pipeline::queue_media_path(
        path,
        auto_media_core::image_pipeline::MediaPriority::Current,
    );
    let handle = IMAGE_HANDLE_COUNTER.fetch_add(1, Ordering::Relaxed);
    insert_image_ticket_handle(handle, ticket);
    task.ram.push_i64(handle as i64);
    Ok(())
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_open(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = task.ram.pop_i64() as u64;
    task.ram
        .push_i64(if image_ticket_from_handle(handle).is_some() {
            handle as i64
        } else {
            0
        });
    Ok(())
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_retain(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = task.ram.pop_i64() as u64;
    if let Some(ticket) = image_ticket_from_handle(handle) {
        let _ = auto_media_core::image_pipeline::global_media_registry().retain(ticket.id);
        task.ram.push_i64(handle as i64);
    } else {
        task.ram.push_i64(0);
    }
    Ok(())
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_release(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    if let Some(ticket) = image_ticket_from_handle(task.ram.pop_i64() as u64) {
        auto_media_core::image_pipeline::global_media_registry().release(ticket.id);
    }
    Ok(())
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_close(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = task.ram.pop_i64() as u64;
    let ticket = {
        let mut handles = IMAGE_TICKET_HANDLES.lock().unwrap();
        handles
            .iter()
            .position(|(id, _)| *id == handle)
            .and_then(|index| handles.remove(index).map(|(_, ticket)| ticket))
    };
    if let Some(ticket) = ticket {
        auto_media_core::image_pipeline::global_media_registry().release(ticket.id);
    }
    Ok(())
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_request(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _height = task.ram.pop_i32();
    let _width = task.ram.pop_i32();
    let handle = task.ram.pop_i64();
    task.ram.push_i64(handle);
    Ok(())
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_scan(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    auto_media_core::image_pipeline::scan_media_directory(path);
    task.ram.push_i64(0);
    Ok(())
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_stats(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    task.ram.push_i64(
        auto_media_core::image_pipeline::global_media_registry()
            .stats()
            .completed as i64,
    );
    Ok(())
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_open_session(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let root: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::open_media_session(root),
    )
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_snapshot(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let session: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::media_session_snapshot(&session),
    )
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_current_uri(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let session: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::media_session_uri(&session),
    )
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_names(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let session: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    auto_media_core::image_pipeline::media_session_names(&session)
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_navigate(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let delta = task.ram.pop_i32();
    let session: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::navigate_media_session(&session, delta),
    )
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_request_view(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let quality = task.ram.pop_i32();
    let viewport_height = task.ram.pop_i32();
    let viewport_width = task.ram.pop_i32();
    let index = task.ram.pop_i32();
    let session: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::request_media_view(
            &session,
            index,
            viewport_width,
            viewport_height,
            quality,
        ),
    )
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_close_session(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let session: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let closed = auto_media_core::image_pipeline::close_media_session(&session);
    closed
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_session_stats(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let session: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::media_session_stats(&session),
    )
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_thumb(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Args pushed (path, size): size pops first.
    let size = task.ram.pop_i32();
    let path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::queue_media_thumbnail(path, size),
    )
}

// ---- PLAN-884 T-04：自 stdlib 壳迁入（原样搬移） ----

// Plan 547: VM handles contain only the opaque ticket id; decoded pixels stay
// in the shared registry and are never pushed onto the VM stack.
// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
const IMAGE_TICKET_HANDLE_CAPACITY: usize = 256;
// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
static IMAGE_TICKET_HANDLES: std::sync::LazyLock<
    std::sync::Mutex<
        std::collections::VecDeque<(u64, auto_media_core::image_pipeline::MediaAssetTicket)>,
    >,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::VecDeque::new()));
// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
static IMAGE_HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);
// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub const IMAGE_NATIVE_NAMES: &[&str] = &[
    "auto.image.queue",
    "auto.image.open",
    "auto.image.scan",
    "auto.image.request",
    "auto.image.retain",
    "auto.image.release",
    "auto.image.close",
    "auto.image.stats",
    "auto.image.open_session",
    "auto.image.snapshot",
    "auto.image.current_uri",
    "auto.image.names",
    "auto.image.navigate",
    "auto.image.request_view",
    "auto.image.close_session",
    "auto.image.session_stats",
    "auto.image.thumb",
];

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
fn image_ticket_from_handle(
    handle: u64,
) -> Option<auto_media_core::image_pipeline::MediaAssetTicket> {
    IMAGE_TICKET_HANDLES
        .lock()
        .ok()?
        .iter()
        .find(|(id, _)| *id == handle)
        .map(|(_, ticket)| *ticket)
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
fn insert_image_ticket_handle(
    handle: u64,
    ticket: auto_media_core::image_pipeline::MediaAssetTicket,
) {
    let mut handles = IMAGE_TICKET_HANDLES
        .lock()
        .expect("image handle lock poisoned");
    if handles.len() >= IMAGE_TICKET_HANDLE_CAPACITY {
        if let Some((_, evicted)) = handles.pop_front() {
            auto_media_core::image_pipeline::global_media_registry().release(evicted.id);
        }
    }
    handles.push_back((handle, ticket));
}

/// Resolve renderer-neutral media content for an HTTP request. Network wire
/// assembly belongs to the serving adapter, not this language primitive.
// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn media_response_for_vm_request(
    method: &str,
    path: &str,
    if_none_match: Option<&str>,
) -> Option<auto_media_core::image_pipeline::MediaHttpResponse> {
    if !path.starts_with("/api/__auto/media/") {
        return None;
    }
    let path = path.split_once('?').map(|(path, _)| path).unwrap_or(path);
    Some(auto_media_core::image_pipeline::media_http_response(
        auto_media_core::image_pipeline::global_media_registry(),
        method,
        path,
        if_none_match,
    ))
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_image_shims(natives: &mut crate::vm::native::NativeInterface) {
    // PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
    {
        // Package unit tests and embedded callers run with the crate directory
        // as cwd, so native_registry's relative stdlib scan cannot be relied on
        // to discover image.vm.at.  Seed these eight names explicitly before
        // resolving the shims; the declaration file remains the source of the
        // public signatures for normal workspace builds.
        {
            let mut registry = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
            for name in IMAGE_NATIVE_NAMES {
                registry.register(name);
            }
        }
        natives.register_shim_by_name("auto.image.queue", shim_image_queue);
        natives.register_shim_by_name("auto.image.open", shim_image_open);
        natives.register_shim_by_name("auto.image.scan", shim_image_scan);
        natives.register_shim_by_name("auto.image.request", shim_image_request);
        natives.register_shim_by_name("auto.image.retain", shim_image_retain);
        natives.register_shim_by_name("auto.image.release", shim_image_release);
        natives.register_shim_by_name("auto.image.close", shim_image_close);
        natives.register_shim_by_name("auto.image.stats", shim_image_stats);
        natives.register_shim_by_name("auto.image.open_session", shim_image_open_session);
        natives.register_shim_by_name("auto.image.snapshot", shim_image_snapshot);
        natives.register_shim_by_name("auto.image.current_uri", shim_image_current_uri);
        natives.register_shim_by_name("auto.image.names", shim_image_names);
        natives.register_shim_by_name("auto.image.navigate", shim_image_navigate);
        natives.register_shim_by_name("auto.image.request_view", shim_image_request_view);
        natives.register_shim_by_name("auto.image.close_session", shim_image_close_session);
        natives.register_shim_by_name("auto.image.session_stats", shim_image_session_stats);
        natives.register_shim_by_name("auto.image.thumb", shim_image_thumb);
    }
}

// Preserve the original media wire assertion until its test owner migrates.
#[cfg(test)]
pub(crate) use crate::vm::ffi::http_server::server_blocking::write_media_response;
