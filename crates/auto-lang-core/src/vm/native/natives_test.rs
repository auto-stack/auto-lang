//! PLAN-843 结构拆分：native test 域实现（自 native.rs 原样搬移）。
use super::*;

pub fn shim_assert_eq(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let right_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_right_nv = crate::vm::native::StakeGuard::nv(vm, right_nv);
        let left_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_left_nv = crate::vm::native::StakeGuard::nv(vm, left_nv);

        let left_is_str = auto_val::is_string(left_nv);
        let right_is_str = auto_val::is_string(right_nv);
        let equal = if left_is_str && right_is_str {
            let lidx = auto_val::decode_string(left_nv);
            let ridx = auto_val::decode_string(right_nv);
            let left_str = vm
                .get_string(lidx)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            let right_str = vm
                .get_string(ridx)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            left_str.as_deref() == right_str.as_deref()
        } else if left_nv == right_nv {
            true
        } else if auto_val::is_i32(left_nv) && auto_val::is_i32(right_nv) {
            auto_val::decode_i32(left_nv) == auto_val::decode_i32(right_nv)
        } else if auto_val::is_object(left_nv) && auto_val::is_object(right_nv) {
            vm.struct_eq(
                auto_val::decode_object(left_nv) as i32,
                auto_val::decode_object(right_nv) as i32,
            )
        } else {
            false
        };

        if !equal {
            return Err(VMError::RuntimeError(format!(
                "Assertion failed: {:?} != {:?}",
                left_nv, right_nv
            )));
        }
        Ok(())
    }
}

pub fn shim_assert_ne(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let right_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_right_nv = crate::vm::native::StakeGuard::nv(vm, right_nv);
        let left_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_left_nv = crate::vm::native::StakeGuard::nv(vm, left_nv);

        let left_is_str = auto_val::is_string(left_nv);
        let right_is_str = auto_val::is_string(right_nv);

        let equal = if left_is_str && right_is_str {
            let left_str = vm
                .get_string(auto_val::decode_string(left_nv))
                .map(|b| String::from_utf8_lossy(&b).to_string());
            let right_str = vm
                .get_string(auto_val::decode_string(right_nv))
                .map(|b| String::from_utf8_lossy(&b).to_string());
            left_str.as_deref() == right_str.as_deref()
        } else if left_nv == right_nv {
            true
        } else if auto_val::is_i32(left_nv) && auto_val::is_i32(right_nv) {
            auto_val::decode_i32(left_nv) == auto_val::decode_i32(right_nv)
        } else if auto_val::is_object(left_nv) && auto_val::is_object(right_nv) {
            vm.struct_eq(
                auto_val::decode_object(left_nv) as i32,
                auto_val::decode_object(right_nv) as i32,
            )
        } else {
            false
        };

        if equal {
            return Err(VMError::RuntimeError(format!(
                "Assertion failed: {:?} == {:?}",
                left_nv, right_nv
            )));
        }
        Ok(())
    }
}

/// InlineInt64.new() -> instance_id
pub fn shim_inline_int64_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let mut list: ListData<i32> = ListData::new();
    for _ in 0..64 {
        list.push(0);
    }
    let id = vm.insert_heap_object(list);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// InlineInt64.capacity() -> 64
/// Stack: [instance_id] -> 64
pub fn shim_inline_int64_capacity(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _inst_id = crate::vm::native::pop_arg_i32(task);

    let _stake__inst_id = crate::vm::native::StakeGuard::new(vm, _inst_id as i64 as u64);
    task.ram.push_i32(64);
    Ok(())
}

/// InlineInt64.try_grow(min_cap) -> bool
/// Stack: [min_cap, instance_id] -> bool
pub fn shim_inline_int64_try_grow(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let min_cap = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_min_cap = crate::vm::native::StakeGuard::new(vm, min_cap as i64 as u64);
    let _inst_id = crate::vm::native::pop_arg_i32(task);

    let _stake__inst_id = crate::vm::native::StakeGuard::new(vm, _inst_id as i64 as u64);
    task.ram.push_nv(auto_val::encode_bool(min_cap <= 64));
    Ok(())
}

/// InlineInt64.drop() -> nil
pub fn shim_inline_int64_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_id = crate::vm::native::StakeGuard::new(vm, id as i64 as u64);
    vm.rc_release_id(id);
    Ok(())
}

/// Assert true.
/// Stack: message_str_idx, condition (i32) -> void
pub fn shim_test_assert_true(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u32;
    let condition = crate::vm::native::pop_arg_i32(task);

    let _stake_condition = crate::vm::native::StakeGuard::new(vm, condition as i64 as u64);
    if condition == 0 {
        let msg = if let Some(bytes) = vm.get_string(msg_idx) {
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            "assertion failed".to_string()
        };
        return Err(VMError::RuntimeError(format!(
            "assert_true failed: {}",
            msg
        )));
    }
    Ok(())
}

/// Assert false.
/// Stack: message_str_idx, condition (i32) -> void
pub fn shim_test_assert_false(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u32;
    let condition = crate::vm::native::pop_arg_i32(task);

    let _stake_condition = crate::vm::native::StakeGuard::new(vm, condition as i64 as u64);
    if condition != 0 {
        let msg = if let Some(bytes) = vm.get_string(msg_idx) {
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            "assertion failed".to_string()
        };
        return Err(VMError::RuntimeError(format!(
            "assert_false failed: {}",
            msg
        )));
    }
    Ok(())
}

/// Assert string contains.
/// Stack: message_str_idx, needle_str_idx, haystack_str_idx -> void
pub fn shim_test_assert_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u32;
    let needle_idx = task.ram.pop_str_idx() as u32;
    let haystack_idx = task.ram.pop_str_idx() as u32;
    let haystack = vm
        .get_string(haystack_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let needle = vm
        .get_string(needle_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let msg = vm
        .get_string(msg_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    if !haystack.contains(&needle) {
        return Err(VMError::RuntimeError(format!(
            "assert_contains failed: '{}' not found in '{}' — {}",
            needle, haystack, msg
        )));
    }
    Ok(())
}

/// Assert list length.
/// Stack: message_str_idx, expected_len (i32), list_handle -> void
pub fn shim_test_assert_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    let msg_idx = task.ram.pop_str_idx() as u32;
    let expected = crate::vm::native::pop_arg_i32(task);

    let _stake_expected = crate::vm::native::StakeGuard::new(vm, expected as i64 as u64);
    let handle = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_handle = crate::vm::native::StakeGuard::new(vm, handle as i64 as u64);
    let msg = vm
        .get_string(msg_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let len = {
        let obj = vm.get_heap_object(handle).ok_or_else(|| {
            VMError::RuntimeError("Invalid list handle in assert_len".to_string())
        })?;
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            list.elems.len() as i32
        } else {
            -1
        }
    };
    if len != expected {
        return Err(VMError::RuntimeError(format!(
            "assert_len failed: expected {} got {} — {}",
            expected, len, msg
        )));
    }
    Ok(())
}

/// Assert result is Ok.
/// Stack: message_str_idx, result_val -> void
pub fn shim_test_assert_ok(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u32;
    let val = crate::vm::native::pop_arg_i32(task);

    let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
    let msg = vm
        .get_string(msg_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    if val < 0 {
        return Err(VMError::RuntimeError(format!(
            "assert_ok failed: result is Err — {}",
            msg
        )));
    }
    Ok(())
}

/// Assert result is Err.
/// Stack: message_str_idx, result_val -> void
pub fn shim_test_assert_err(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u32;
    let val = crate::vm::native::pop_arg_i32(task);

    let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
    let msg = vm
        .get_string(msg_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    if val >= 0 {
        return Err(VMError::RuntimeError(format!(
            "assert_err failed: result is Ok — {}",
            msg
        )));
    }
    Ok(())
}
