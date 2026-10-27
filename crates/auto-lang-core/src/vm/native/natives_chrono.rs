//! PLAN-843 结构拆分：native chrono 域实现（自 native.rs 原样搬移）。
use super::*;

/// Local.now() → opaque NaiveDateTime handle
/// Stack: [] -> [handle_i32]
pub fn shim_chrono_local_now(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let dt = chrono::Local::now().naive_local();
    let obj = RustStdlibObject::new("chrono::NaiveDateTime", std::sync::Mutex::new(dt));
    let id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// dt.year() → i32
/// Stack: [handle_i32] -> [year_i32]
pub fn shim_chrono_year(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Datelike;

    let dt_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_dt_id = crate::vm::native::StakeGuard::new(vm, dt_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().year());
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.month() → i32
/// Stack: [handle_i32] -> [month_i32]
pub fn shim_chrono_month(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Datelike;

    let dt_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_dt_id = crate::vm::native::StakeGuard::new(vm, dt_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().month() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.day() → i32
/// Stack: [handle_i32] -> [day_i32]
pub fn shim_chrono_day(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Datelike;

    let dt_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_dt_id = crate::vm::native::StakeGuard::new(vm, dt_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().day() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.hour() → i32
/// Stack: [handle_i32] -> [hour_i32]
pub fn shim_chrono_hour(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Timelike;

    let dt_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_dt_id = crate::vm::native::StakeGuard::new(vm, dt_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().hour() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.minute() → i32
/// Stack: [handle_i32] -> [minute_i32]
pub fn shim_chrono_minute(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Timelike;

    let dt_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_dt_id = crate::vm::native::StakeGuard::new(vm, dt_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().minute() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.second() → i32
/// Stack: [handle_i32] -> [second_i32]
pub fn shim_chrono_second(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Timelike;

    let dt_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_dt_id = crate::vm::native::StakeGuard::new(vm, dt_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().second() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.timestamp() → i64 (pushed as two i32)
/// Stack: [handle_i32] -> [timestamp_lo, timestamp_hi]
pub fn shim_chrono_timestamp(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let dt_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_dt_id = crate::vm::native::StakeGuard::new(vm, dt_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                let ts = dt.lock().unwrap().and_utc().timestamp();
                // Push as i32 if fits, otherwise heap-aware i64 (Plan 377: >2^47 经 BigInt)
                if ts >= i32::MIN as i64 && ts <= i32::MAX as i64 {
                    task.ram.push_i32(ts as i32);
                } else {
                    vm.push_i64_vm(task, ts);
                }
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.format(fmt) → string
/// Stack: [fmt_str, handle_i32] -> [result_str]
pub fn shim_chrono_format(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let fmt = pop_vm_string(task, vm);
    let dt_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_dt_id = crate::vm::native::StakeGuard::new(vm, dt_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                let formatted = dt.lock().unwrap().format(&fmt).to_string();
                push_vm_string(task, vm, &formatted);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// dt.drop() — no-op, GC handles cleanup
pub fn shim_chrono_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_id = crate::vm::native::StakeGuard::new(vm, id as i64 as u64);
    vm.rc_release_id(id);
    Ok(())
}

/// Create an Instant::now() opaque handle.
/// Stack: -> handle_id
pub fn shim_instant_now(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let instant = std::time::Instant::now();
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new("std::time::Instant", instant);
    let id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// Get elapsed time from Instant handle.
/// PLAN-701 供④ elapsed 约定勘定：返回自创建点起的**毫秒 int**（单调钟）。
/// 此前形态=「Nms」字符串入池 + null 双推、catalog 返回型 Void——.at 侧
/// 实际得到 None（m1-supply §9 观察件）。现对齐 I64 语义（catalog 同步
/// Void→I64）；跨轨边界注记：a2r/trans 轨的 `Instant.elapsed()` 保持真实
/// Rust `Duration`（opaque 不透明形），VM 轨为毫秒 int——不透明类型跨轨
/// 既有边界，本约定成文于 SD-02。
/// Stack: handle_id -> elapsed_ms (i64)
pub fn shim_instant_elapsed(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_handle = crate::vm::native::StakeGuard::new(vm, handle as i64 as u64);
    let obj = vm
        .get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid Instant handle".to_string()))?;
    let guard = obj.read().unwrap();
    let rust_obj = guard
        .as_any()
        .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let instant = rust_obj
        .downcast_ref::<std::time::Instant>()
        .ok_or_else(|| VMError::RuntimeError("Not an Instant object".to_string()))?;
    let millis = instant.elapsed().as_millis() as i64;
    drop(guard);
    vm.push_i64_vm(task, millis);
    Ok(())
}

/// Create DateTime from Unix timestamp.
/// Stack: timestamp (i32) -> opaque_handle
pub fn shim_chrono_from_timestamp(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let ts = crate::vm::native::pop_arg_i32(task) as i64;

    let _stake_ts = crate::vm::native::StakeGuard::new(vm, ts as i64 as u64);
    let dt = chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.naive_utc())
        .unwrap_or_else(|| {
            chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .unwrap()
        });
    let obj = RustStdlibObject::new("DateTime", std::sync::Mutex::new(dt));
    let handle = vm.insert_heap_object(obj) as i32;
    vm.rc_push_id(task, handle as u64); // Plan 419
    Ok(())
}

/// Create DateTime from year/month/day.
/// Stack: day, month, year -> opaque_handle
pub fn shim_chrono_from_ymd(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let day = crate::vm::native::pop_arg_i32(task);

    let _stake_day = crate::vm::native::StakeGuard::new(vm, day as i64 as u64);
    let month = crate::vm::native::pop_arg_i32(task);

    let _stake_month = crate::vm::native::StakeGuard::new(vm, month as i64 as u64);
    let year = crate::vm::native::pop_arg_i32(task);

    let _stake_year = crate::vm::native::StakeGuard::new(vm, year as i64 as u64);
    let dt = chrono::NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .unwrap_or_else(|| {
            chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .unwrap()
        });
    let obj = RustStdlibObject::new("DateTime", std::sync::Mutex::new(dt));
    let handle = vm.insert_heap_object(obj) as i32;
    vm.rc_push_id(task, handle as u64); // Plan 419
    Ok(())
}

/// Get weekday (0=Monday..6=Sunday).
/// Stack: opaque_handle -> i32
pub fn shim_chrono_weekday(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Datelike;
    let handle = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_handle = crate::vm::native::StakeGuard::new(vm, handle as i64 as u64);
    let obj = vm
        .get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid DateTime handle".to_string()))?;
    let guard = obj.read().unwrap();
    if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
        if let Some(dt) = rust_obj.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
            task.ram
                .push_i32(dt.lock().unwrap().weekday().num_days_from_monday() as i32);
            return Ok(());
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// Compare two datetimes by timestamp.
/// Stack: handle_b (i32), handle_a (i32) -> i32 (-1, 0, 1)
pub fn shim_datetime_cmp(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let b_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_b_id = crate::vm::native::StakeGuard::new(vm, b_id as i64 as u64);
    let a_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_a_id = crate::vm::native::StakeGuard::new(vm, a_id as i64 as u64);

    let ts_a = get_datetime_timestamp(vm, a_id);
    let ts_b = get_datetime_timestamp(vm, b_id);

    use std::cmp::Ordering;
    let result = match ts_a.cmp(&ts_b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    };
    task.ram.push_i32(result);
    Ok(())
}
