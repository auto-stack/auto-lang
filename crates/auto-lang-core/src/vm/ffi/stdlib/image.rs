//! PLAN-843 结构拆分：image 域实现（原样搬移，算法/行为零变化）。
use super::*;

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_queue(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
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
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
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
    let root: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::open_media_session(root),
    )
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_snapshot(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let session: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::media_session_snapshot(&session),
    )
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_current_uri(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let session: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::media_session_uri(&session),
    )
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_names(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let session: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    auto_media_core::image_pipeline::media_session_names(&session)
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_navigate(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let delta = task.ram.pop_i32();
    let session: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
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
    let session: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
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
    let session: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let closed = auto_media_core::image_pipeline::close_media_session(&session);
    closed
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_image_session_stats(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let session: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
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
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_string_result(
        task,
        vm,
        auto_media_core::image_pipeline::queue_media_thumbnail(path, size),
    )
}
