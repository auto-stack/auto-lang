//! PLAN-843 结构拆分：native rand 域实现（自 native.rs 原样搬移）。
use super::*;

/// thread_rng() -> opaque Rng handle (stored as RustStdlibObject on heap)
/// Stack: [] -> [rng_id_i32]
pub fn shim_rand_thread_rng(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let rng = std::sync::Mutex::new(Xorshift64::new(seed));
    let obj = RustStdlibObject::new("rand::ThreadRng", rng);
    let rng_id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, rng_id as u64); // Plan 419
    Ok(())
}

/// rng.gen_range(range) -> i32
/// Stack: [range_or_hi, lo_or_rng_id, maybe_rng_id] -> [result]
/// Supports both range expression (single marker) and explicit (hi, lo) args.
pub fn shim_rng_gen_range(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let top = crate::vm::native::pop_arg_i32(task);

    let _stake_top = crate::vm::native::StakeGuard::new(vm, top as i64 as u64);

    // Check if top is a range marker (-1000000 + idx)
    let (lo, hi, rng_id) = if top <= -1000000 && top > -2000000 {
        let range_idx = (top + 1000000) as usize;
        if let Some(&(start, end, _eq)) = task.ram.ranges.get(range_idx) {
            let rng_id = crate::vm::native::pop_arg_i32(task) as u64;

            let _stake_rng_id = crate::vm::native::StakeGuard::new(vm, rng_id as i64 as u64);
            (start, end, rng_id)
        } else {
            let rng_id = crate::vm::native::pop_arg_i32(task) as u64;

            let _stake_rng_id = crate::vm::native::StakeGuard::new(vm, rng_id as i64 as u64);
            (0, 1, rng_id)
        }
    } else {
        // Legacy: hi, lo, rng_id layout
        let hi = top;
        let lo = crate::vm::native::pop_arg_i32(task);

        let _stake_lo = crate::vm::native::StakeGuard::new(vm, lo as i64 as u64);
        let rng_id = crate::vm::native::pop_arg_i32(task) as u64;

        let _stake_rng_id = crate::vm::native::StakeGuard::new(vm, rng_id as i64 as u64);
        (lo, hi, rng_id)
    };

    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let range = (hi - lo).max(1);
                let val = rng.get_mut().unwrap().next();
                let result = lo + ((val % range as u64) as i32);
                task.ram.push_i32(result);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// rng.gen() -> i32
/// Stack: [rng_id] -> [result]
pub fn shim_rng_gen(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let rng_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_rng_id = crate::vm::native::StakeGuard::new(vm, rng_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let val = rng.get_mut().unwrap().next() as i32;
                task.ram.push_i32(val);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// rng.drop() — no-op, GC will handle
pub fn shim_rng_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_id = crate::vm::native::StakeGuard::new(vm, id as i64 as u64);
    vm.rc_release_id(id);
    Ok(())
}

/// rand::random() → random i32
/// Stack: [] -> [random_i32]
pub fn shim_rand_random(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = Xorshift64::new(seed);
    // Plan 402 §13.10: the native registry marks auto.rand.random as Void
    // return, so push_f64 here would leave a float that codegen doesn't
    // expect (stack imbalance). Return a non-negative i32 instead — the old
    // push_i32(rng.next() as i32) returned arbitrary (often negative) ints,
    // so `(math.random() * n).to_int()` produced negative indices and
    // mine placement never worked. Mask to 31 bits → [0, i32::MAX).
    task.ram.push_i32((rng.next() & 0x7FFF_FFFF) as i32);
    Ok(())
}

/// Thread-local random int in [0, max).
/// Stack: max (i32) -> i32
pub fn shim_rand_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let max = crate::vm::native::pop_arg_i32(task);

    let _stake_max = crate::vm::native::StakeGuard::new(vm, max as i64 as u64);
    let max = if max <= 0 { 1 } else { max as u32 };
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = Xorshift64::new(seed);
    let val = (rng.next() % max as u64) as i32;
    task.ram.push_i32(val);
    Ok(())
}

/// Thread-local random float in [0, 1).
/// Stack: -> f64
pub fn shim_rand_float(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = Xorshift64::new(seed);
    let val = (rng.next() as f64) / (u64::MAX as f64);
    task.ram.push_f64(val);
    Ok(())
}

/// Thread-local random bool.
/// Stack: -> bool
pub fn shim_rand_bool(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = Xorshift64::new(seed);
    let val = rng.next() % 2 == 0;
    task.ram.push_nv(auto_val::encode_bool(val));
    Ok(())
}

/// Shuffle a list in-place using Fisher-Yates.
/// Stack: list_handle -> void
pub fn shim_rand_shuffle(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    let handle = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_handle = crate::vm::native::StakeGuard::new(vm, handle as i64 as u64);
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let obj = vm
        .get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid list handle in shuffle".to_string()))?;
    let mut guard = obj.write().unwrap();
    if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
        let mut rng = Xorshift64::new(seed);
        let mut i = list.elems.len();
        while i > 1 {
            i -= 1;
            let j = (rng.next() % (i as u64 + 1)) as usize;
            list.elems.swap(i, j);
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// Create RNG from system entropy.
/// Stack: [] -> rng_handle (i32)
pub fn shim_random_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let rng = std::sync::Mutex::new(Xorshift64::new(seed));
    let obj = RustStdlibObject::new("random::Rng", rng);
    let id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// Create seeded RNG.
/// Stack: seed (i32) -> rng_handle (i32)
pub fn shim_random_seeded(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let seed = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_seed = crate::vm::native::StakeGuard::new(vm, seed as i64 as u64);
    let rng = std::sync::Mutex::new(Xorshift64::new(seed));
    let obj = RustStdlibObject::new("random::Rng", rng);
    let id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// Instance method: rng.int(max) -> i32
/// Stack: max (i32), rng_handle (i32) -> i32
pub fn shim_random_instance_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let max = crate::vm::native::pop_arg_i32(task);

    let _stake_max = crate::vm::native::StakeGuard::new(vm, max as i64 as u64);
    let rng_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_rng_id = crate::vm::native::StakeGuard::new(vm, rng_id as i64 as u64);
    let max = if max <= 0 { 1 } else { max as u32 };
    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let val = rng.get_mut().unwrap().next();
                let result = (val % max as u64) as i32;
                task.ram.push_i32(result);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// Instance method: rng.float() -> f64
/// Stack: rng_handle (i32) -> f64
pub fn shim_random_instance_float(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let rng_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_rng_id = crate::vm::native::StakeGuard::new(vm, rng_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let val = (rng.get_mut().unwrap().next() as f64) / (u64::MAX as f64);
                task.ram.push_f64(val);
                return Ok(());
            }
        }
    }
    task.ram.push_f64(0.0);
    Ok(())
}

/// Instance method: rng.bool() -> bool (i32)
/// Stack: rng_handle (i32) -> i32 (0 or 1)
pub fn shim_random_instance_bool(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let rng_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_rng_id = crate::vm::native::StakeGuard::new(vm, rng_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let val = rng.get_mut().unwrap().next() % 2 == 0;
                task.ram.push_nv(auto_val::encode_bool(val));
                return Ok(());
            }
        }
    }
    task.ram.push_nv(auto_val::encode_bool(false));
    Ok(())
}
