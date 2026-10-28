//! PLAN-843 结构拆分：native collections 域实现（自 native.rs 原样搬移）。
use super::*;

/// Create a new list with optional initial elements.
/// Stack: [elem1, elem2, ...] -> list_id
/// Returns: list_id (u64 as i32)
///
/// If elements are on the stack (above bp + 1 + num_locals), they are used to initialize the list.
/// Elements are in LIFO order (top of stack is last element).
pub fn shim_list_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // Plan 317: List.new([a, b, c]) receives ONE arg: an array_id (from
    // CREATE_ARRAY). If all elements are Int, create ListData<i32> (backward
    // compat with existing List<int> tests). Otherwise (struct/string/etc),
    // store as Vec<Value> in arrays (supports any element type).
    // List.new() with no args → empty ListData<i32>.
    let target_sp = if task.bp == 0 {
        task.num_locals + 1
    } else {
        task.bp + task.num_locals + 2
    };

    if crate::pool_log_all() {
        eprintln!(
            "[POOLLOG #{:>4}] list.new bp={} nlocals={} sp={} target_sp={} has_arg_guess={}",
            crate::pool_log_seq(),
            task.bp,
            task.num_locals,
            task.ram.sp,
            target_sp,
            task.ram.sp > target_sp
        );
    }

    if task.ram.sp <= target_sp {
        // No arguments: empty ListData<i32>.
        let list: ListData<i32> = ListData::new();
        let list_id = vm.insert_heap_object(list);
        vm.rc_push_id(task, list_id as u64); // Plan 419
        return Ok(());
    }

    let arg = crate::vm::native::pop_arg_i32(task);
    // Plan 419: 先克隆(含 retain)再释放参数 stake(读取必须先于释放)。
    let _stake_arg = crate::vm::native::StakeGuard::new(vm, arg as i64 as u64);

    // Plan 390 §15 H3b: array literals are ListData<Value> in heap_objects.
    if let Some(arr_ref) = vm.get_heap_object(arg as u64) {
        let arr = arr_ref.read().unwrap();
        if let Some(arr) = arr
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<Value>>()
        {
            // Plan 320: empty lists default to ListData<Value> (not ListData<i32>)
            // so struct pushes (List<Note>) work. Only use ListData<i32> when there
            // are actual int elements.
            if !arr.elems.is_empty() && arr.elems.iter().all(|v| matches!(v, Value::Int(_))) {
                let mut list: ListData<i32> = ListData::new();
                for v in arr.elems.iter() {
                    if let Value::Int(i) = v {
                        list.push(*i);
                    }
                }
                let list_id = vm.insert_heap_object(list);
                vm.rc_push_id(task, list_id as u64); // Plan 419
            } else {
                // Plan 320: use ListData<Value> for empty or mixed/struct lists.
                // Plan 419: 克隆进新列表的 VmRef/裸 id 元素随容器持有 retain
                // (arg 已按消费型释放,克隆引用若无 stake 会被级联回收)。
                let mut list: ListData<Value> = ListData::new();
                for v in arr.elems.iter() {
                    match v {
                        Value::VmRef(r) => vm.rc_retain_id(r.id as u64),
                        Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                            vm.rc_retain_id(*i as u64)
                        }
                        _ => {}
                    }
                    list.push(v.clone());
                }
                let list_id = vm.insert_heap_object(list);
                vm.rc_push_id(task, list_id as u64); // Plan 419
            }
            return Ok(());
        }
    }

    // Legacy fallback: treat arg as first element, pop remaining as variadic.
    let target_sp = if task.bp == 0 {
        task.num_locals + 1
    } else {
        task.bp + task.num_locals + 2
    };
    let mut elems = vec![arg];
    while task.ram.sp > target_sp {
        elems.push(task.ram.pop_i32());
    }
    elems.reverse();
    let mut list: ListData<i32> = ListData::new();
    for elem in &elems {
        list.push(*elem);
    }
    let list_id = vm.insert_heap_object(list);
    vm.rc_push_id(task, list_id as u64); // Plan 419
    Ok(())
}

/// Push an element to the end of the list.
/// Stack: list_id, elem -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_push(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // PLAN-062: 列表元素突变（入口计——多成功出口统一覆盖，错误路径
    // 多 bump 一次是保守安全方向）。
    // PLAN-047 T-03: bump 移至 list_id 出栈后定点归因（B 类）。
    let elem_nv = task.ram.pop_nv();
    // PLAN-604 T04: 取走元素槽份额。元素是 copy-on-load/构造回推的暂存
    // 拷贝(rc_push +1 记影子);本 shim 消费它,容器所有权由分支内 retain
    // 承接——stake 在 retain 之后释放(transfer 配平,避免 rc 短暂下探)。
    // CALL_SPEC→resolve 分发路径无 CALL_NAT 式死区结算,此前这份份额被
    // 后续 push 清影静默丢弃 → 元素对象图每调用孤儿一份(KD-VM1 收口
    // 缺口,探针 StructTick +100 obj/拍)。
    let elem_stake = task.ram.take_stake_at(task.ram.sp);
    let elem_val = if auto_val::is_i32(elem_nv) {
        Value::Int(auto_val::decode_i32(elem_nv))
    } else if auto_val::is_object(elem_nv) {
        Value::VmRef(auto_val::VmRef {
            id: auto_val::decode_object(elem_nv) as usize,
        })
    } else if auto_val::is_string(elem_nv) {
        // Plan 403: store the actual string bytes, not the pool index. The old
        // code stored Value::Int(string_index), which corrupted List<str>/<Value>
        // element values on read-back (e.g. "3" rendered as its pool index).
        let idx = auto_val::decode_string(elem_nv) as usize;
        let s = vm
            .strings
            .read()
            .unwrap()
            .get(idx)
            .map(|b| String::from_utf8_lossy(b).to_string())
            .unwrap_or_default();
        Value::Str(s.into())
    } else if auto_val::is_null(elem_nv) {
        Value::Nil
    } else if auto_val::is_bool(elem_nv) {
        Value::Bool(auto_val::decode_bool(elem_nv))
    } else if auto_val::is_f64(elem_nv) {
        // Plan 437: f64 nanbox 此前落进末尾 else 被 decode_i32 重解释成垃圾
        // int（3.0 → 1074266112）。引擎方法分发臂（engine.rs "push"，Plan 403）
        // 与 CREATE_ARRAY 早已修 float，原生 shim 这条 mono-dispatch 路径漏修
        // ——List<float>.push() 全部损坏（chart 几何坐标列表直接受阻）。
        Value::Double(auto_val::decode_f64(elem_nv))
    } else if auto_val::is_f32(elem_nv) {
        Value::Float(auto_val::decode_f32(elem_nv) as f64)
    } else {
        Value::Int(auto_val::decode_i32(elem_nv))
    };
    let list_id = task.ram.pop_i32() as u64;
    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id);
    // PLAN-047 T-03: 定点归因（B 类）。
    vm.bump_path(list_id, None);

    // First try heap_objects (ListData<i32> OR ListData<Value> from List<T>.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            // ListData<i32> is untyped storage: normalize bools to 1/0 so the raw
            // i32::MIN/i32::MIN+1 sentinel payload doesn't leak in (it would break
            // push_tagged_value's negation on read-back and corrupt element values).
            let stored = if auto_val::is_bool(elem_nv) {
                if auto_val::decode_bool(elem_nv) {
                    1
                } else {
                    0
                }
            } else {
                auto_val::decode_i32(elem_nv)
            };
            // Plan 419: 裸 id 元素入容器 —— 容器获得持有一份(CALL_NAT 死区
            // 结算会释放被弹出的参数槽,retain 与之配平)。
            // Plan 432 D26: 字符串负哨兵同入此契约(此前漏 retain → UAF)。
            list_i32_elem_retain(vm, stored);
            list.push(stored);
            // PLAN-604 T04: 元素暂存份额随入容器结算(transfer 配平)。
            if elem_stake != 0 {
                vm.rc_release_id(elem_stake);
            }
            task.ram.push_i32(0);
            return Ok(());
        }
        // Plan 403: ListData<String> (from CREATE_LIST_STR / List<str>) — store
        // the actual string bytes, not the nanbox index. Without this branch a
        // push onto List<str> silently dropped the element (both i32 and Value
        // downcasts failed), corrupting any string-stack algorithm (e.g. the
        // calculator's shunting-yard engine).
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<String>>() {
            let s = if auto_val::is_string(elem_nv) {
                let idx = auto_val::decode_string(elem_nv) as usize;
                vm.strings
                    .read()
                    .unwrap()
                    .get(idx)
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .unwrap_or_default()
            } else {
                auto_val::decode_i32(elem_nv).to_string()
            };
            list.push(s);
            // PLAN-604 T04: 元素暂存份额结算(本分支存字节拷贝,容器不持
            // 池/堆份额;堆 stake 释放即消费配平)。
            if elem_stake != 0 {
                vm.rc_release_id(elem_stake);
            }
            task.ram.push_i32(0);
            return Ok(());
        }
        // Plan 320: ListData<Value> (struct element lists like List<Note>)
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            // Plan 419: VmRef/裸 id 元素入容器,retain 配平死区结算。
            match &elem_val {
                Value::VmRef(r) => vm.rc_retain_id(r.id as u64),
                Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                    vm.rc_retain_id(*i as u64)
                }
                _ => {}
            }
            list.push(elem_val);
            // PLAN-604 T04: 元素暂存份额随入容器结算(transfer 配平)——
            // 容器 retain 承接所有权,暂存拷贝的 stake 释放,rc 净变化 0。
            if elem_stake != 0 {
                vm.rc_release_id(elem_stake);
            }
            task.ram.push_i32(0);
            return Ok(());
        }
    }

    // Plan 390 §15 H3b: array literals are ListData<Value> in heap_objects and
    // are handled by the heap path above (the legacy arrays fallback is gone).

    // PLAN-604 T04: 落空分支(无 downcast 命中,元素被静默丢弃)——暂存
    // 份额同样结算,不随丢弃孤儿化。
    if elem_stake != 0 {
        vm.rc_release_id(elem_stake);
    }
    task.ram.push_i32(0);
    Ok(())
}

/// Pop an element from the end of the list.
/// Stack: list_id -> elem
// Plan 077 Phase 5: Updated to use unified registry
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_pop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // PLAN-062: 列表元素突变（入口计，同 shim_list_push 口径）。
    // PLAN-047 T-03: bump 移至 list_id 出栈后定点归因（B 类）。
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;
    vm.bump_path(list_id, None);

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // First try heap_objects (ListData<i32> from List.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            let elem = list.pop().unwrap_or(0);
            // Plan 419: 元素离开容器 —— 容器 stake 死亡;回栈为转移。
            // Plan 432 D26: ①先 +1 回栈再释放容器份额——原"先 release 后
            // push"在末次引用时会把对象/池条目释放掉再让栈引用之(悬垂);
            // ②字符串负哨兵此前被 push_i32 成裸 int(类型丢失 + 无池份额),
            // 改走 rc_push_str_idx 保 TAG_STRING。
            if (elem as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 {
                vm.rc_push_id(task, elem as u64);
                vm.rc_release_id(elem as u64);
            } else if elem < 0 {
                vm.rc_push_str_idx(task, (-elem - 1) as usize);
                vm.pool_release((-elem - 1) as usize);
            } else {
                task.ram.push_i32(elem);
            }
            return Ok(());
        }
        // Plan 403: ListData<String> (from CREATE_LIST_STR / List<str>) — pop the
        // actual string and push it as a nanbox string value.
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<String>>() {
            if let Some(s) = list.pop() {
                let idx = vm.add_string(s.into_bytes());
                vm.rc_push_str_idx(task, idx as usize);
            } else {
                task.ram.push_i32(0);
            }
            return Ok(());
        }
        // Plan 335: ListData<Value> (struct element lists)
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            if let Some(val) = list.pop() {
                // Plan 419: 容器 stake 死亡(push_value 的 +1 使净效果为转移)。
                match &val {
                    Value::VmRef(r) => vm.rc_release_id(r.id as u64),
                    Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                        vm.rc_release_id(*i as u64)
                    }
                    _ => {}
                }
                push_value(task, vm, &val);
            } else {
                task.ram.push_i32(0);
            }
            return Ok(());
        }
    }

    // Invalid list_id
    task.ram.push_i32(0);
    Ok(())
}

/// Get the length of the list.
/// Stack: list_id -> len
// Plan 077 Phase 5: Updated to use unified registry
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // First try heap_objects (ListData<i32> OR ListData<Value> from List<T>.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            task.ram.push_i32(list.len() as i32);
            return Ok(());
        }
        // Plan 403: ListData<String> (from CREATE_LIST_STR / List<str>)
        if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>() {
            task.ram.push_i32(list.len() as i32);
            return Ok(());
        }
        // Plan 320: ListData<Value> (struct element lists like List<Note>)
        if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
            task.ram.push_i32(list.len() as i32);
            return Ok(());
        }
    }

    // Invalid list_id
    task.ram.push_i32(0);
    Ok(())
}

/// Check if the list is empty.
/// Stack: list_id -> is_empty (1 if empty, 0 otherwise)
// Plan 077 Phase 5: Updated to unified registry
pub fn shim_list_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            task.ram.push_nv(auto_val::encode_bool(list.is_empty()));
            return Ok(());
        }
    }

    // Invalid list_id treated as empty
    task.ram.push_nv(auto_val::encode_bool(true));
    Ok(())
}

/// Clear all elements from the list.
/// Stack: list_id -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            // Plan 419: 被清空元素的容器 stake 逐个死亡。
            // Plan 432 D26: 字符串负哨兵同入契约。
            for &e in &list.elems {
                list_i32_elem_release(vm, e);
            }
            list.clear();
        }
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            // Plan 419: 同上(VmRef/裸 id)。
            for v in &list.elems {
                match v {
                    Value::VmRef(r) => vm.rc_release_id(r.id as u64),
                    Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                        vm.rc_release_id(*i as u64)
                    }
                    _ => {}
                }
            }
            list.clear();
        }
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

pub fn shim_list_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    {
        let index_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_index_nv = crate::vm::native::StakeGuard::nv(vm, index_nv);
        let list_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_list_nv = crate::vm::native::StakeGuard::nv(vm, list_nv);
        let index = if auto_val::is_i32(index_nv) {
            auto_val::decode_i32(index_nv) as usize
        } else {
            0usize
        };
        // Plan 437: 列表引用是 TAG_OBJECT nanbox（encode_object(list_id)），
        // 原先只认 is_i32 → object-tag 接收者落 list_id=0，.get() 恒返回 0。
        let list_id = if auto_val::is_object(list_nv) {
            auto_val::decode_object(list_nv) as u64
        } else if auto_val::is_i32(list_nv) {
            auto_val::decode_i32(list_nv) as u64
        } else {
            0u64
        };

        if let Some(obj) = vm.get_heap_object(list_id) {
            let guard = obj.read().unwrap();
            if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                if let Some(&val) = list.get(index) {
                    // Plan 419: 元素仍在容器 —— 拷贝引用入栈 +1。
                    push_tagged_value_rc(vm, task, val);
                } else {
                    task.ram.push_i32(0);
                }
                return Ok(());
            }
            // Plan 403: ListData<String> (from CREATE_LIST_STR / List<str>) —
            // return the element as a nanbox string.
            if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>() {
                if let Some(s) = list.get(index) {
                    let idx = vm.add_string(s.clone().into_bytes());
                    vm.rc_push_str_idx(task, idx as usize);
                } else {
                    task.ram.push_i32(0);
                }
                return Ok(());
            }
            // Plan 335: ListData<Value> (struct element lists like List<Note>)
            if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
                if let Some(val) = list.get(index) {
                    push_value(task, vm, val);
                } else {
                    task.ram.push_i32(0);
                }
                return Ok(());
            }
        }
        task.ram.push_i32(0);
        return Ok(());
    }
}

/// Set element at index.
/// Stack: list_id, index, elem -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // PLAN-062: 列表元素突变（入口计，同 shim_list_push 口径）。
    // PLAN-047 T-03: bump 移至 list_id 出栈后定点归因（B 类）。
    let elem_nv = task.ram.pop_nv();
    let elem_val = nv_to_value(elem_nv);
    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;
    vm.bump_path(list_id, None);
    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id);

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            // Plan 419: 旧元素释放;新值 retain(死区结算配平)。
            // Plan 432 D26: 字符串负哨兵同入契约。
            if let Some(&old) = list.get(index) {
                list_i32_elem_release(vm, old);
            }
            let stored = auto_val::decode_i32(elem_nv);
            list_i32_elem_retain(vm, stored);
            list.set(index, stored);
            task.ram.push_i32(0);
            return Ok(());
        }
        // Plan 335: ListData<Value> (struct element lists)
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            // Plan 419: 同上。
            if let Some(old) = list.get(index) {
                match old {
                    Value::VmRef(r) => vm.rc_release_id(r.id as u64),
                    Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                        vm.rc_release_id(*i as u64)
                    }
                    _ => {}
                }
            }
            match &elem_val {
                Value::VmRef(r) => vm.rc_retain_id(r.id as u64),
                Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                    vm.rc_retain_id(*i as u64)
                }
                _ => {}
            }
            list.set(index, elem_val);
            task.ram.push_i32(0);
            return Ok(());
        }
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Insert element at index.
/// Stack: list_id, index, elem -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_insert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let elem_nv = task.ram.pop_nv();
    let elem_val = nv_to_value(elem_nv);
    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;
    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id);

    // First try heap_objects (ListData<i32> from List.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            // Plan 419: 新值 retain(死区结算配平)。
            // Plan 432 D26: 字符串负哨兵同入契约。
            let stored = auto_val::decode_i32(elem_nv);
            list_i32_elem_retain(vm, stored);
            list.insert(index, stored);
            task.ram.push_i32(0);
            return Ok(());
        }
        // Plan 335: ListData<Value> (struct element lists)
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            // Plan 419: 新值 retain。
            match &elem_val {
                Value::VmRef(r) => vm.rc_retain_id(r.id as u64),
                Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                    vm.rc_retain_id(*i as u64)
                }
                _ => {}
            }
            list.insert(index, elem_val);
            task.ram.push_i32(0);
            return Ok(());
        }
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Remove element at index and return it.
/// Stack: list_id, index -> elem
// Plan 077 Phase 5: Updated to use unified registry
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let index = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_index = crate::vm::native::StakeGuard::new(vm, index as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // First try heap_objects (ListData<i32> from List.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            if let Some(elem) = list.remove(index) {
                // Plan 419: 元素离开容器 —— 容器 stake 死亡;回栈为转移。
                // Plan 432 D26: 先 +1 回栈再释放;字符串哨兵保 TAG_STRING。
                if (elem as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 {
                    vm.rc_push_id(task, elem as u64);
                    vm.rc_release_id(elem as u64);
                } else if elem < 0 {
                    vm.rc_push_str_idx(task, (-elem - 1) as usize);
                    vm.pool_release((-elem - 1) as usize);
                } else {
                    task.ram.push_i32(elem);
                }
                return Ok(());
            }
        }
        // Plan 335: ListData<Value> (struct element lists)
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            if let Some(val) = list.remove(index) {
                // Plan 419: 容器 stake 死亡(push_value +1 → 净转移)。
                match &val {
                    Value::VmRef(r) => vm.rc_release_id(r.id as u64),
                    Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                        vm.rc_release_id(*i as u64)
                    }
                    _ => {}
                }
                push_value(task, vm, &val);
                return Ok(());
            }
        }
    }

    // Return default value if index out of bounds
    task.ram.push_i32(0);
    Ok(())
}

/// List.splice(list_id, start, count) -> removed List
/// PLAN-622 (d): JS Array.splice removal form — removes `count` elements at
/// `start` (negative start counts from the end; both clamped) and returns
/// them as a NEW list. Element stakes transfer from the source container to
/// the returned one (retain-into-new before release-from-old). The insertion
/// variant of JS splice is intentionally not in v1 — callers compose with
/// insert; noted in the ui/store-facade-semantics spec.
/// Stack: list_id, start, count -> removed_list_id
pub fn shim_list_splice(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let count = crate::vm::native::pop_arg_i32(task).max(0) as usize;
    let start_raw = crate::vm::native::pop_arg_i32(task);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id);

    // Removed elements, encoded per source-list representation.
    let mut removed_i32: Option<Vec<i32>> = None;
    let mut removed_val: Option<Vec<auto_val::Value>> = None;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            let len = list.elems.len();
            let start = normalize_splice_start(start_raw, len);
            let end = (start + count).min(len);
            if start < end {
                removed_i32 = Some(list.elems.drain(start..end).collect());
            }
        } else if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            let len = list.elems.len();
            let start = normalize_splice_start(start_raw, len);
            let end = (start + count).min(len);
            if start < end {
                removed_val = Some(list.elems.drain(start..end).collect());
            }
        }
    }

    // Build the removed list with transferred stakes: retain into the new
    // container first (insert semantics), then release the old container's
    // stake (remove semantics). JS-parity: no elements removed still returns
    // a fresh (empty) list.
    let removed_id: u64 = if let Some(items) = removed_i32 {
        let mut out: ListData<i32> = ListData::with_capacity(items.len());
        for v in &items {
            list_i32_elem_retain(vm, *v);
        }
        out.elems = items.clone();
        let id = vm.insert_heap_object(out);
        for v in &items {
            list_i32_elem_release(vm, *v);
        }
        id
    } else if let Some(items) = removed_val {
        let mut out: ListData<Value> = ListData::with_capacity(items.len());
        for v in &items {
            match v {
                Value::VmRef(r) => vm.rc_retain_id(r.id as u64),
                Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                    vm.rc_retain_id(*i as u64);
                }
                _ => {}
            }
        }
        out.elems = items.clone();
        let id = vm.insert_heap_object(out);
        for v in &items {
            match v {
                Value::VmRef(r) => vm.rc_release_id(r.id as u64),
                Value::Int(i) if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 => {
                    vm.rc_release_id(*i as u64);
                }
                _ => {}
            }
        }
        id
    } else {
        vm.insert_heap_object(ListData::<i32>::new())
    };

    vm.rc_push_id(task, removed_id);
    Ok(())
}

/// Drop/free the list.
/// Stack: list_id -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
// Plan 419: 语义更新 —— 无条件 remove 改为 rc 释放(RC 归零才真回收,
// 兼容"手动提前释放"直觉;多持有时只减一)。
pub fn shim_list_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);
    vm.rc_release_id(list_id);

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// List.reserve(list_id, additional) -> 0
/// Pre-allocate capacity for additional elements
pub fn shim_list_reserve(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let additional = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_additional = crate::vm::native::StakeGuard::new(vm, additional as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.reserve(additional);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// List.map(closure) -> new List
/// Stack: closure_id, list_id -> result_list_id
pub fn shim_list_map(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_closure_id = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Fast path: i32 elements
    if let Ok(elements) = get_list_i32_elements(vm, list_id) {
        let mut results = Vec::with_capacity(elements.len());
        for elem in elements {
            push_tagged_value_rc(vm, task, elem);
            vm.call_closure(task, closure_id, 1)?;
            results.push(crate::vm::native::pop_arg_i32(task));
        }
        let new_id = create_list_from_i32(vm, results);
        // Plan 432 D26 堆侧修复:新建堆 id 入栈必须 rc_push_id(+1)——
        // 此前裸 push_i32,栈拷贝无份额 → STORE_LOC 转移进局部槽仍 0 份额,
        // 局部槽 LOAD 的 copy-on-load +1 成"首次获取",被 pop_arg 释放归零
        // 即对象在局部槽仍引用时死亡(RETAIN-AFTER-FREE → canary,
        // conformance_bootstrap 4000001 实证)。
        vm.rc_push_id(task, new_id);
        return Ok(());
    }

    // Plan 340: Value path (struct/str elements)
    let elements = get_list_elements_as_value(vm, list_id)?;
    let mut results: Vec<auto_val::Value> = Vec::with_capacity(elements.len());
    for elem in &elements {
        push_value(task, vm, elem);
        vm.call_closure(task, closure_id, 1)?;
        let ret_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_ret_nv = crate::vm::native::StakeGuard::nv(vm, ret_nv);
        results.push(nv_to_value(ret_nv));
    }
    let new_id = create_list_from_value(vm, results);
    // Plan 432 D26 堆侧修复:同上,新建堆 id 入栈 +1。
    vm.rc_push_id(task, new_id);
    Ok(())
}

/// List.filter(closure) -> new List
/// Stack: closure_id, list_id -> result_list_id
pub fn shim_list_filter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_closure_id = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Fast path: i32 elements
    if let Ok(elements) = get_list_i32_elements(vm, list_id) {
        let mut results = Vec::new();
        for elem in elements {
            push_tagged_value_rc(vm, task, elem);
            vm.call_closure(task, closure_id, 1)?;
            let predicate = crate::vm::native::pop_arg_i32(task);

            let _stake_predicate = crate::vm::native::StakeGuard::new(vm, predicate as i64 as u64);
            if vm_is_truthy(predicate) {
                results.push(elem);
            }
        }
        let new_id = create_list_from_i32(vm, results);
        // Plan 432 D26 堆侧修复:新建堆 id 入栈必须 rc_push_id(+1)——
        // 此前裸 push_i32,栈拷贝无份额 → STORE_LOC 转移进局部槽仍 0 份额,
        // 局部槽 LOAD 的 copy-on-load +1 成"首次获取",被 pop_arg 释放归零
        // 即对象在局部槽仍引用时死亡(RETAIN-AFTER-FREE → canary,
        // conformance_bootstrap 4000001 实证)。
        vm.rc_push_id(task, new_id);
        return Ok(());
    }

    // Plan 340: Value path (struct/str elements)
    let elements = get_list_elements_as_value(vm, list_id)?;
    let mut results: Vec<auto_val::Value> = Vec::new();
    for elem in &elements {
        push_value(task, vm, elem);
        vm.call_closure(task, closure_id, 1)?;
        let predicate = crate::vm::native::pop_arg_i32(task);

        let _stake_predicate = crate::vm::native::StakeGuard::new(vm, predicate as i64 as u64);
        if vm_is_truthy(predicate) {
            results.push(elem.clone());
        }
    }
    let new_id = create_list_from_value(vm, results);
    // Plan 432 D26 堆侧修复:同上,新建堆 id 入栈 +1。
    vm.rc_push_id(task, new_id);
    Ok(())
}

/// List.for_each(closure) -> void
/// Stack: closure_id, list_id -> 0
pub fn shim_list_for_each(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_closure_id = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Fast path: i32 elements
    if let Ok(elements) = get_list_i32_elements(vm, list_id) {
        for elem in elements {
            push_tagged_value_rc(vm, task, elem);
            vm.call_closure(task, closure_id, 1)?;
            {
                let d = crate::vm::native::pop_arg_i32(task);
                vm.rc_release_id(d as u64);
            } // Plan 419: 丢弃即释放
        }
        task.ram.push_i32(0);
        return Ok(());
    }

    // Plan 340: Value path (struct/str elements)
    let elements = get_list_elements_as_value(vm, list_id)?;
    for elem in &elements {
        push_value(task, vm, elem);
        vm.call_closure(task, closure_id, 1)?;
        let _disc = crate::vm::native::pop_arg_nv(task);

        let _stake__disc = crate::vm::native::StakeGuard::nv(vm, _disc);
        vm.rc_release(_disc); // Plan 419: 丢弃的结果若为引用,-1
    }
    task.ram.push_i32(0);
    Ok(())
}

/// List.find(closure) -> ?T (found value or -1 for None)
/// Stack: closure_id, list_id -> result
pub fn shim_list_find(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_closure_id = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Fast path: genuine ListData<i32>
    if let Ok(elements) = get_list_i32_elements(vm, list_id) {
        for elem in elements {
            push_tagged_value_rc(vm, task, elem);
            vm.call_closure(task, closure_id, 1)?;
            let found = crate::vm::native::pop_arg_i32(task);

            let _stake_found = crate::vm::native::StakeGuard::new(vm, found as i64 as u64);
            if vm_is_truthy(found) {
                task.ram.push_i32(elem);
                return Ok(());
            }
        }
        // Plan 446 批四 D4: miss 契约对齐语言层 None（TAG_NULL，与 auto.obj.find
        // 的 Plan 454 E 契约一致）——此前推 -1 哨兵，`hit == None` 恒假。
        task.ram.push_nv(auto_val::encode_null());
        return Ok(());
    }

    // Plan 340: Value path (struct/str elements)
    let elements = get_list_elements_as_value(vm, list_id)?;
    for elem in &elements {
        push_value(task, vm, elem);
        vm.call_closure(task, closure_id, 1)?;
        let found = crate::vm::native::pop_arg_i32(task);

        let _stake_found = crate::vm::native::StakeGuard::new(vm, found as i64 as u64);
        if vm_is_truthy(found) {
            push_value(task, vm, elem);
            return Ok(());
        }
    }
    // Plan 446 批四 D4: miss = 语言层 None（同上 fast-path 注记）。
    task.ram.push_nv(auto_val::encode_null());
    Ok(())
}

/// List.find_index(closure) -> int — first matching index, or -1 on miss.
/// PLAN-624 (P4): jade `findIndex(t => t.path == path)` 用法；此前列表原生面
/// 无 find_index，调用静默失效。栈：list_id, closure_id -> index(i32)。
pub fn shim_list_find_index(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_closure_id = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Fast path: genuine ListData<i32>
    if let Ok(elements) = get_list_i32_elements(vm, list_id) {
        for (i, elem) in elements.iter().enumerate() {
            push_tagged_value_rc(vm, task, *elem);
            vm.call_closure(task, closure_id, 1)?;
            let found = crate::vm::native::pop_arg_i32(task);

            let _stake_found = crate::vm::native::StakeGuard::new(vm, found as i64 as u64);
            if vm_is_truthy(found) {
                task.ram.push_i32(i as i32);
                return Ok(());
            }
        }
        task.ram.push_i32(-1);
        return Ok(());
    }

    // Value path (struct/str elements)
    let elements = get_list_elements_as_value(vm, list_id)?;
    for (i, elem) in elements.iter().enumerate() {
        push_value(task, vm, elem);
        vm.call_closure(task, closure_id, 1)?;
        let found = crate::vm::native::pop_arg_i32(task);

        let _stake_found = crate::vm::native::StakeGuard::new(vm, found as i64 as u64);
        if vm_is_truthy(found) {
            task.ram.push_i32(i as i32);
            return Ok(());
        }
    }
    task.ram.push_i32(-1);
    Ok(())
}

/// List.any(closure) -> bool
/// Stack: closure_id, list_id -> bool (1/0)
pub fn shim_list_any(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_closure_id = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Fast path: i32 elements
    if let Ok(elements) = get_list_i32_elements(vm, list_id) {
        for elem in elements {
            push_tagged_value_rc(vm, task, elem);
            vm.call_closure(task, closure_id, 1)?;
            let result = crate::vm::native::pop_arg_i32(task);

            let _stake_result = crate::vm::native::StakeGuard::new(vm, result as i64 as u64);
            if vm_is_truthy(result) {
                task.ram.push_nv(auto_val::encode_bool(true));
                return Ok(());
            }
        }
        task.ram.push_nv(auto_val::encode_bool(false));
        return Ok(());
    }

    // Plan 340: Value path (struct/str elements)
    let elements = get_list_elements_as_value(vm, list_id)?;
    for elem in &elements {
        push_value(task, vm, elem);
        vm.call_closure(task, closure_id, 1)?;
        let result = crate::vm::native::pop_arg_i32(task);

        let _stake_result = crate::vm::native::StakeGuard::new(vm, result as i64 as u64);
        if vm_is_truthy(result) {
            task.ram.push_nv(auto_val::encode_bool(true));
            return Ok(());
        }
    }
    task.ram.push_nv(auto_val::encode_bool(false));
    Ok(())
}

/// List.all(closure) -> bool
/// Stack: closure_id, list_id -> bool (1/0)
pub fn shim_list_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_closure_id = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Fast path: i32 elements
    if let Ok(elements) = get_list_i32_elements(vm, list_id) {
        for elem in elements {
            push_tagged_value_rc(vm, task, elem);
            vm.call_closure(task, closure_id, 1)?;
            let result = crate::vm::native::pop_arg_i32(task);

            let _stake_result = crate::vm::native::StakeGuard::new(vm, result as i64 as u64);
            if !vm_is_truthy(result) {
                task.ram.push_nv(auto_val::encode_bool(false));
                return Ok(());
            }
        }
        task.ram.push_nv(auto_val::encode_bool(true));
        return Ok(());
    }

    // Plan 340: Value path (struct/str elements)
    let elements = get_list_elements_as_value(vm, list_id)?;
    for elem in &elements {
        push_value(task, vm, elem);
        vm.call_closure(task, closure_id, 1)?;
        let result = crate::vm::native::pop_arg_i32(task);

        let _stake_result = crate::vm::native::StakeGuard::new(vm, result as i64 as u64);
        if !vm_is_truthy(result) {
            task.ram.push_nv(auto_val::encode_bool(false));
            return Ok(());
        }
    }
    task.ram.push_nv(auto_val::encode_bool(true));
    Ok(())
}

/// List.reduce(init, closure) -> accumulated value
/// Stack: closure_id, init_val, list_id -> result
pub fn shim_list_reduce(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_closure_id = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let init_val = crate::vm::native::pop_arg_i32(task);

    let _stake_init_val = crate::vm::native::StakeGuard::new(vm, init_val as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Fast path: i32 elements
    if let Ok(elements) = get_list_i32_elements(vm, list_id) {
        let mut acc = init_val;
        for elem in elements {
            task.ram.push_i32(acc);
            task.ram.push_i32(elem);
            vm.call_closure(task, closure_id, 2)?;
            acc = crate::vm::native::pop_arg_i32(task);
        }
        task.ram.push_i32(acc);
        return Ok(());
    }

    // Plan 340: Value path (struct/str elements)
    let elements = get_list_elements_as_value(vm, list_id)?;
    let mut acc = auto_val::Value::Int(init_val);
    for elem in &elements {
        push_value(task, vm, &acc);
        push_value(task, vm, elem);
        vm.call_closure(task, closure_id, 2)?;
        let ret_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_ret_nv = crate::vm::native::StakeGuard::nv(vm, ret_nv);
        acc = nv_to_value(ret_nv);
    }
    push_value(task, vm, &acc);
    Ok(())
}

/// List.join(separator) -> str
/// Stack: separator (str tag), list_id -> joined_str (str tag)
/// Joins string list elements with the given separator.
pub fn shim_list_join(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // Pop separator (string pool tag)
    let sep_idx = decode_str_idx_nv(crate::vm::native::pop_arg_nv(task));
    let separator = vm
        .strings
        .read()
        .unwrap()
        .get(sep_idx)
        .map(|b| String::from_utf8_lossy(b).to_string())
        .unwrap_or_default();

    // Pop list_id
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        // Try ListData<Value> (FFI bridge uses this for Vec<String>)
        if let Some(list) = guard.as_any().downcast_ref::<ListData<auto_val::Value>>() {
            let parts: Vec<String> = list
                .elems
                .iter()
                .filter_map(|v| {
                    if let auto_val::Value::Str(s) = v {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let joined = parts.join(&separator);
            let str_idx = vm.add_string(joined.into_bytes());
            vm.rc_push_str_idx(task, str_idx as usize);
            return Ok(());
        }
        // Try ListData<String>
        if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>() {
            let joined = list.elems.join(&separator);
            let str_idx = vm.add_string(joined.into_bytes());
            vm.rc_push_str_idx(task, str_idx as usize);
            return Ok(());
        }
        // Try ListData<i32> as fallback
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            let parts: Vec<String> = list.elems.iter().map(|e| e.to_string()).collect();
            let joined = parts.join(&separator);
            let str_idx = vm.add_string(joined.into_bytes());
            vm.rc_push_str_idx(task, str_idx as usize);
            return Ok(());
        }
    }

    // Fallback: return empty string
    let str_idx = vm.add_string(Vec::new());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// List.contains(value) -> bool
/// Stack: value (i32), list_id -> bool (1/0)
pub fn shim_list_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let elem_nv = crate::vm::native::pop_arg_nv(task);

    let _stake_elem_nv = crate::vm::native::StakeGuard::nv(vm, elem_nv);
    let elem_val = nv_to_value(elem_nv);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Plan 335: 统一在 ListData<i32> / ListData<Value> / heap ListData（array
    // 字面量已并入，Plan 390 §15 H3b）上查找。
    // struct List 的元素是 VmRef —— 按 id 比较（同一 struct 实例视为相等）。
    let found = if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            list.elems
                .iter()
                .any(|&e| e == auto_val::decode_i32(elem_nv))
        } else if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
            list.elems.iter().any(|v| values_eq(v, &elem_val))
        } else {
            false
        }
    } else {
        false
    };
    task.ram.push_nv(auto_val::encode_bool(found));
    Ok(())
}

/// Sort a list in-place.
/// Stack: list_id -> void
pub fn shim_list_sort(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Plan 390 §15 H3b: array literals are ListData<Value> in heap_objects.
    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            list.elems.sort_by(|a, b| match (a, b) {
                (auto_val::Value::Int(x), auto_val::Value::Int(y)) => x.cmp(y),
                (auto_val::Value::Uint(x), auto_val::Value::Uint(y)) => x.cmp(y),
                (auto_val::Value::Float(x), auto_val::Value::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (auto_val::Value::Double(x), auto_val::Value::Double(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (auto_val::Value::Bool(x), auto_val::Value::Bool(y)) => x.cmp(y),
                (auto_val::Value::Str(x), auto_val::Value::Str(y)) => {
                    x.to_string().cmp(&y.to_string())
                }
                (auto_val::Value::String(x), auto_val::Value::String(y)) => {
                    x.as_str().cmp(y.as_str())
                }
                (auto_val::Value::VmRef(x), auto_val::Value::VmRef(y)) => x.id.cmp(&y.id),
                _ => std::cmp::Ordering::Equal,
            });
        } else if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.elems.sort();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Sort a list with a comparator closure.
/// Stack: list_id, closure_id -> void
/// (Currently delegates to default sort — custom comparators not yet supported in VM)
pub fn shim_list_sort_by(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let _closure_id = crate::vm::native::pop_arg_i32(task);

    let _stake__closure_id = crate::vm::native::StakeGuard::new(vm, _closure_id as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            list.elems.sort_by(|a, b| match (a, b) {
                (auto_val::Value::Int(x), auto_val::Value::Int(y)) => x.cmp(y),
                (auto_val::Value::Uint(x), auto_val::Value::Uint(y)) => x.cmp(y),
                (auto_val::Value::Float(x), auto_val::Value::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (auto_val::Value::Double(x), auto_val::Value::Double(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (auto_val::Value::VmRef(x), auto_val::Value::VmRef(y)) => x.id.cmp(&y.id),
                _ => std::cmp::Ordering::Equal,
            });
        } else if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.elems.sort();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Create an iterator for a list.
/// Stack: list_id -> iterator_id
/// Returns: iterator_id (u32 as i32)
pub fn shim_list_iter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::{Iterator, ListIterator};

    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    let iterator_id = vm.register_iterator(Iterator::List(ListIterator {
        list_id,
        current_index: 0,
    }));

    // Return iterator_id
    task.ram.push_i32(iterator_id as i32);
    Ok(())
}

pub fn shim_iterator_next_cooperative(
    task: &mut AutoTask,
    vm: &AutoVM,
    budget: u32,
) -> Result<bool, VMError> {
    let iterator_id = auto_val::decode_i32(task.ram.peek_nv(0)) as u32;
    let stack_before = task.ram.sp;
    let previous =
        PLAN696_SSE_NEXT_BUDGET.with(|slot| slot.replace(Some((iterator_id, budget.max(1)))));
    let result = shim_iterator_next(task, vm);
    PLAN696_SSE_NEXT_BUDGET.with(|slot| slot.set(previous));
    result.map(|()| task.ram.sp == stack_before)
}

pub fn shim_iterator_next(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::Iterator;
    use crate::vm::types::ListData;

    // Plan 550 T04: null 迭代源守卫——for-in 源为 TAG_NULL 时按 Python
    // 风格报 TypeError（现状：位模式解码成垃圾 iterator id → 查表落空
    // → 静默 -1 → 零迭代，p5 探针实证）。合法迭代器 id 永不与 TAG_NULL
    // 重叠，守卫零误伤。
    if auto_val::is_null(task.ram.peek_nv(0)) {
        return Err(VMError::RuntimeError(
            "TypeError: 'NoneType' object is not iterable".to_string(),
        ));
    }

    // PLAN-776 R5-01：凭据感知——for-in/迭代源为 Sse 凭据对象（TAG_OBJECT）
    // 时解包底层 iterator id。stream 声明函数内 generator 产出已凭据化
    //（生产点包装），迭代消费经此保持 int 源同语义（非 HTTP 消费兼容）。
    let iterator_id = if auto_val::is_object(task.ram.peek_nv(0)) {
        // PLAN-776 R8-02/R9-01：与 pop_iterator_receiver 同款——槽清零 +
        // StakeGuard 即一次消费（Drop 释放；不加显式 release——F776-R9-01）。
        let nv = task.ram.pop_nv();
        task.ram.raw_nv[task.ram.sp] = 0;
        let stake = StakeGuard::nv(vm, nv);
        let resolved = crate::vm::ffi::http_return_resource::resolve_resource(vm, nv);
        drop(stake);
        match resolved {
            Some(crate::vm::ffi::http_return_resource::ResourceClaim::Sse { iterator_id }) => {
                iterator_id
            }
            _ => {
                return Err(VMError::RuntimeError(
                    "iterator.next: object is not an SSE credential".to_string(),
                ));
            }
        }
    } else {
        crate::vm::native::pop_arg_i32(task) as u32
    };
    let cooperative_budget = PLAN696_SSE_NEXT_BUDGET.with(|slot| {
        slot.get()
            .filter(|(target_iterator, _)| *target_iterator == iterator_id)
            .map(|(_, budget)| budget)
    });
    let cooperative = cooperative_budget.is_some();

    let _stake_iterator_id = crate::vm::native::StakeGuard::new(vm, iterator_id as i64 as u64);

    // Plan 076 Phase 3: Always return i32 by extracting Int from Value
    // Get the iterator (need to clone to update)
    // PLAN-707 T-05：Generator 臂在 get_mut **之前**分流——generator 驱动
    // 不得持有 iterators 写锁（内层嵌套 iterator_next 对同 DashMap shard
    // 的 get_mut/get 重入自死锁，707 实测挂死）。clone→驱动→回写。
    let is_generator_iter = vm
        .iterators
        .get(&iterator_id)
        .map(|e| matches!(&*e, Iterator::Generator(_)))
        .unwrap_or(false);
    if is_generator_iter {
        let mut gen_state = {
            let entry = vm.iterators.get(&iterator_id).expect("checked above");
            match &*entry {
                Iterator::Generator(g) => g.clone(),
                _ => unreachable!("checked above"),
            }
        };
        generator_next_drive(task, vm, iterator_id, &mut gen_state);
        match vm.iterators.entry(iterator_id) {
            dashmap::mapref::entry::Entry::Occupied(mut e) => {
                *e.get_mut() = Iterator::Generator(gen_state);
            }
            dashmap::mapref::entry::Entry::Vacant(e) => {
                e.insert(Iterator::Generator(gen_state));
            }
        }
        return Ok(());
    }

    let result = if let Some(mut iter_mut) = vm.iterators.get_mut(&iterator_id) {
        match &mut *iter_mut {
            Iterator::List(list_iter) => {
                // Plan 077 Phase 6: Get list from unified registry
                // Plan 390 §15 H3b: array literals are ListData<Value> here too.
                if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                    let list = obj.read().unwrap();
                    if list.type_tag() != crate::vm::heap_object::TypeTag::ListInt {
                        -1
                    } else if let Some(list_data) = list.as_any().downcast_ref::<ListData<i32>>() {
                        if list_iter.current_index >= list_data.len() as u32 {
                            -1
                        } else {
                            let elem = list_data
                                .get(list_iter.current_index as usize)
                                .copied()
                                .unwrap_or(0);
                            list_iter.current_index += 1;
                            elem
                        }
                    } else {
                        -1
                    }
                } else if let Some(arr_ref) = vm.get_heap_object(list_iter.list_id) {
                    // Plan 390 §15 H3b: ListData<Value> (array literals, struct
                    // lists). Supports any element type (struct, str, etc).
                    let arr = arr_ref.read().unwrap();
                    if let Some(list) = arr.as_any().downcast_ref::<ListData<Value>>() {
                        if list_iter.current_index >= list.elems.len() as u32 {
                            -1
                        } else {
                            let val = list.elems[list_iter.current_index as usize].clone();
                            list_iter.current_index += 1;
                            // Encode Value back to i32 for the result (struct→object id,
                            // str→string idx, int→int). The caller (for-loop) pushes
                            // this i32; struct id is recognized by GET_FIELD later.
                            match val {
                                Value::Int(i) => i,
                                Value::VmRef(r) => r.id as i32,
                                Value::Nil => -1,
                                Value::Bool(b) => {
                                    if b {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                _ => 0,
                            }
                        }
                    } else {
                        -1
                    }
                } else {
                    -1
                }
            }
            Iterator::Map(map_iter) => {
                // Recursively get next element from source iterator
                // For MVP, we don't actually call the function yet
                // We just return the source element as-is

                // Call next() on source iterator
                // We need to manually call the logic here since we can't recursively call shim_iterator_next
                if let Some(mut source_iter) = vm.iterators.get_mut(&map_iter.source_iterator_id) {
                    match &mut *source_iter {
                        Iterator::List(list_iter) => {
                            // Plan 077 Phase 6: Get list from unified registry
                            if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                                let list = obj.read().unwrap();

                                if list.type_tag() != crate::vm::heap_object::TypeTag::ListInt {
                                    -1 // Wrong type
                                } else if let Some(list_data) =
                                    list.as_any().downcast_ref::<ListData<i32>>()
                                {
                                    if list_iter.current_index >= list_data.len() as u32 {
                                        -1 // Source exhausted
                                    } else {
                                        let elem = list_data
                                            .get(list_iter.current_index as usize)
                                            .copied()
                                            .unwrap_or(0);
                                        list_iter.current_index += 1;

                                        // TODO: Call the function at map_iter.func_addr with elem
                                        // For MVP, just return the element without transformation
                                        elem
                                    }
                                } else {
                                    -1 // Downcast failed
                                }
                            } else {
                                -1 // Invalid list
                            }
                        }
                        Iterator::Map(_) => {
                            // Nested Map not supported yet
                            -1
                        }
                        Iterator::Filter(_) => {
                            // Filter source not supported yet
                            -1
                        }
                        Iterator::Generator(_) => {
                            // Generator as Map source not yet supported
                            -1
                        }
                        Iterator::HttpStream(_) => {
                            // HttpStream as Map source not yet supported
                            -1
                        }
                        Iterator::AsyncHttpStream(_) => {
                            // Plan 341: AsyncHttpStream as Map source not yet supported
                            -1
                        }
                        Iterator::Enumerate(enumerate_iter) => {
                            // Get next from source, then push index on top
                            if let Some(mut source_iter) =
                                vm.iterators.get_mut(&enumerate_iter.source_iterator_id)
                            {
                                match &mut *source_iter {
                                    Iterator::List(list_iter) => {
                                        if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                                            let list = obj.read().unwrap();
                                            if list.type_tag()
                                                != crate::vm::heap_object::TypeTag::ListInt
                                            {
                                                -1
                                            } else if let Some(list_data) =
                                                list.as_any().downcast_ref::<ListData<i32>>()
                                            {
                                                if list_iter.current_index >= list_data.len() as u32
                                                {
                                                    -1
                                                } else {
                                                    let elem = list_data
                                                        .get(list_iter.current_index as usize)
                                                        .copied()
                                                        .unwrap_or(0);
                                                    list_iter.current_index += 1;
                                                    let idx = enumerate_iter.current_index as i32;
                                                    enumerate_iter.current_index += 1;
                                                    // Push value then index (caller pops index first)
                                                    task.ram.push_i32(elem);
                                                    task.ram.push_i32(idx);
                                                    return Ok(());
                                                }
                                            } else {
                                                -1
                                            }
                                        } else {
                                            -1
                                        }
                                    }
                                    _ => -1,
                                }
                            } else {
                                -1
                            }
                        }
                    }
                } else {
                    -1 // Source iterator not found
                }
            }
            Iterator::Filter(filter_iter) => {
                // Recursively get next element from source iterator
                // For MVP, we don't actually call the predicate yet
                // We just return the source element as-is (no filtering)

                // Call next() on source iterator
                if let Some(mut source_iter) = vm.iterators.get_mut(&filter_iter.source_iterator_id)
                {
                    match &mut *source_iter {
                        Iterator::List(list_iter) => {
                            // Plan 077 Phase 6: Get list from unified registry
                            if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                                let list = obj.read().unwrap();

                                if list.type_tag() != crate::vm::heap_object::TypeTag::ListInt {
                                    -1 // Wrong type
                                } else if let Some(list_data) =
                                    list.as_any().downcast_ref::<ListData<i32>>()
                                {
                                    if list_iter.current_index >= list_data.len() as u32 {
                                        -1 // Source exhausted
                                    } else {
                                        let elem = list_data
                                            .get(list_iter.current_index as usize)
                                            .copied()
                                            .unwrap_or(0);
                                        list_iter.current_index += 1;

                                        // TODO: Call the predicate at filter_iter.func_addr with elem
                                        // For MVP, just return the element without filtering
                                        elem
                                    }
                                } else {
                                    -1 // Downcast failed
                                }
                            } else {
                                -1 // Invalid list
                            }
                        }
                        _ => {
                            // Nested adapters not yet supported
                            return Err(VMError::RuntimeError(
                                "Nested adapters not yet implemented".to_string(),
                            ));
                        }
                    }
                } else {
                    -1 // Invalid source iterator
                }
            }
            Iterator::Enumerate(enumerate_iter) => {
                // Plan 200 Task 3.2: Get next from source, push (index, value)
                if let Some(mut source_iter) =
                    vm.iterators.get_mut(&enumerate_iter.source_iterator_id)
                {
                    match &mut *source_iter {
                        Iterator::List(list_iter) => {
                            if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                                let list = obj.read().unwrap();
                                if list.type_tag() != crate::vm::heap_object::TypeTag::ListInt {
                                    -1
                                } else if let Some(list_data) =
                                    list.as_any().downcast_ref::<ListData<i32>>()
                                {
                                    if list_iter.current_index >= list_data.len() as u32 {
                                        -1
                                    } else {
                                        let elem = list_data
                                            .get(list_iter.current_index as usize)
                                            .copied()
                                            .unwrap_or(0);
                                        list_iter.current_index += 1;
                                        let idx = enumerate_iter.current_index as i32;
                                        enumerate_iter.current_index += 1;
                                        // Push value then index (two values on stack)
                                        task.ram.push_i32(elem);
                                        task.ram.push_i32(idx);
                                        return Ok(());
                                    }
                                } else {
                                    -1
                                }
                            } else {
                                -1
                            }
                        }
                        _ => -1,
                    }
                } else {
                    -1
                }
            }
            Iterator::Generator(_) => unreachable!(
                "generator handled before iterators.get_mut dispatch (re-entrant guard)",
            ),
            Iterator::HttpStream(hs_iter) => {
                // PLAN-707 T-04：HTTPStream 迭代器走统一 typed pull
                //（http_stream::STREAMS 资源表；旧 thread-local blocking
                // read 退役）。EOF/Failed/Cancelled → -1 终结（修复 G-I：
                // 旧 EOF 臂不推 -1，cooperative 驱动误判 pending）；无数据
                // → 挂 waiting_http_stream_id 等待（同 AsyncHttpStream 的
                // Waiting 语义，wake source 复查 stream_ready）。
                if hs_iter.done {
                    return Ok(());
                }

                match crate::vm::ffi::http_stream::stream_pull(hs_iter.stream_handle) {
                    crate::vm::ffi::http_stream::Pull::Data(chunk) => {
                        let idx = vm.add_string(chunk.into_bytes());
                        vm.rc_push_str_idx(task, idx as usize);
                    }
                    crate::vm::ffi::http_stream::Pull::Eof
                    | crate::vm::ffi::http_stream::Pull::Failed(_) => {
                        hs_iter.done = true;
                        task.ram.push_i32(-1);
                    }
                    crate::vm::ffi::http_stream::Pull::Opening
                    | crate::vm::ffi::http_stream::Pull::Pending => {
                        task.waiting_http_stream_id = Some(hs_iter.stream_handle);
                        task.status = crate::vm::task::TaskStatus::Waiting("sse".into());
                        // 参数回推（同 fallback 注释；cooperative 驱动自推）。
                        if !cooperative {
                            task.ram.push_i32(iterator_id as i32);
                        }
                        return Ok(());
                    }
                }
                return Ok(());
            }
            crate::vm::engine::Iterator::AsyncHttpStream(async_iter) => {
                // Plan 348: Non-blocking SSE event pull. Instead of busy-wait
                // with sleep(2ms) (which freezes the VM), we do a single
                // try_recv. If no event is available, yield the task (set
                // Waiting("sse") + back up IP) so run_task_loop can schedule
                // other tasks and re-check the channel on the next round.
                if async_iter.done {
                    return Ok(());
                }

                // Single non-blocking poll of the channel.
                // Plan 349: HTTP and IO streams share the unified ASYNC_STREAMS
                // table; stream_id is globally unique, so one lookup suffices.
                let event = crate::vm::ffi::stdlib::ASYNC_STREAMS
                    .lock()
                    .ok()
                    .and_then(|map| {
                        map.get(&async_iter.stream_id).and_then(|handle| {
                            handle.rx.lock().ok().and_then(|mut rx| rx.try_recv().ok())
                        })
                    });

                // Check if the producing future is done (for end-of-stream detection).
                let future_done = crate::vm::ffi::stdlib::ASYNC_STREAMS
                    .lock()
                    .ok()
                    .and_then(|map| {
                        map.get(&async_iter.stream_id)
                            .map(|handle| handle.done.load(std::sync::atomic::Ordering::SeqCst))
                    })
                    .unwrap_or(true);

                match event {
                    Some(crate::vm::ffi::stdlib::AsyncStreamEvent::Data(s)) => {
                        let idx = vm.add_string(s.into_bytes());
                        vm.rc_push_str_idx(task, idx as usize);
                    }
                    Some(crate::vm::ffi::stdlib::AsyncStreamEvent::Done)
                    | Some(crate::vm::ffi::stdlib::AsyncStreamEvent::Error(_)) => {
                        async_iter.done = true;
                        task.ram.push_i32(-1);
                    }
                    None => {
                        // No event available right now.
                        if future_done {
                            // Future finished and channel drained — end of stream.
                            async_iter.done = true;
                            task.ram.push_i32(-1);
                        } else {
                            // Still waiting for data — yield without blocking.
                            // run_task_loop will re-check on the next scheduling round
                            // (≤10ms), and other tasks (UI rendering etc.) get to run.
                            task.waiting_sse_stream_id = Some(async_iter.stream_id);
                            task.status = crate::vm::task::TaskStatus::Waiting("sse".into());
                            // 参数回推（与 PLAN-707 fallback/HttpStream 臂同款）：
                            // CALL_NAT rewind 重试时 shim 重新 pop——不回推则
                            // 重试弹到垃圾 id（707 实测嵌套误入）。cooperative
                            // 驱动自行重推，回推破坏其 sp==stack_before 契约。
                            if !cooperative {
                                // 回推**迭代器 id**（被 pop 的栈参）——流 id 与
                                // 迭代器 id 是不同键，回推错值令重试弹到无关
                                // 句柄（707 实测 plan341 回归根因）。
                                task.ram.push_i32(iterator_id as i32);
                            }
                            return Ok(());
                        }
                    }
                }
                // Clear the waiting flag if we got data or finished.
                task.waiting_sse_stream_id = None;
                return Ok(());
            }
        }
    } else {
        // Fallback: if iterator_id is actually a heap list object, auto-create iterator
        // PLAN-707 D-5：流句柄惰性消费——for-in 的迭代器通道把「内联
        // stream-opening 调用」或「持流变量」的值（= 流 id）直接传给本
        // shim；命中存活流即就地 pull（不进 iterators 表，零 id 碰撞；
        // 状态全在流资源表 cell 内）。覆盖内联调用、赋变量再 for 两形态。
        if crate::vm::ffi::http_stream::stream_is_live(iterator_id as u64) {
            match crate::vm::ffi::http_stream::stream_pull(iterator_id as u64) {
                crate::vm::ffi::http_stream::Pull::Data(chunk) => {
                    // 就绪出口必须清等待凭据——否则残留标志令引擎在后续
                    // 任意 CALL_NAT 上误 rewind（热循环根因，707 实测）。
                    task.waiting_http_stream_id = None;
                    let idx = vm.add_string(chunk.into_bytes());
                    vm.rc_push_str_idx(task, idx as usize);
                    return Ok(());
                }
                crate::vm::ffi::http_stream::Pull::Eof
                | crate::vm::ffi::http_stream::Pull::Failed(_) => {
                    task.waiting_http_stream_id = None;
                    task.ram.push_i32(-1);
                    return Ok(());
                }
                crate::vm::ffi::http_stream::Pull::Opening
                | crate::vm::ffi::http_stream::Pull::Pending => {
                    task.waiting_http_stream_id = Some(iterator_id as u64);
                    task.status = crate::vm::task::TaskStatus::Waiting("sse".into());
                    // 参数回推：CALL_NAT rewind 重试时 shim 会重新 pop
                    //（Plan 348/419 重试协议的参数重建臂；cooperative 驱动
                    // 自行重推 id，回推会破坏其 sp==stack_before 契约）。
                    if !cooperative {
                        task.ram.push_i32(iterator_id as i32);
                    }
                    return Ok(());
                }
            }
        }
        if let Some(obj) = vm.get_heap_object(iterator_id as u64) {
            let guard = obj.read().unwrap();
            if guard.type_tag() == crate::vm::heap_object::TypeTag::ListInt {
                if let Some(_list_data) = guard.as_any().downcast_ref::<ListData<i32>>() {
                    // Create an iterator for this list on the fly
                    let list_iter = crate::vm::engine::ListIterator {
                        list_id: iterator_id as u64,
                        current_index: 0,
                    };
                    // Use iterator_id (the list's heap ID) as key so subsequent
                    // iterator_next calls find it with the same ID the loop passes
                    vm.iterators.insert(iterator_id, Iterator::List(list_iter));
                    drop(guard);
                    if let Some(mut iter_mut) = vm.iterators.get_mut(&iterator_id) {
                        if let Iterator::List(ref mut li) = *iter_mut {
                            if let Some(obj2) = vm.get_heap_object(li.list_id) {
                                let list = obj2.read().unwrap();
                                if let Some(list_data) =
                                    list.as_any().downcast_ref::<ListData<i32>>()
                                {
                                    if li.current_index < list_data.len() as u32 {
                                        let elem = list_data
                                            .get(li.current_index as usize)
                                            .copied()
                                            .unwrap_or(0);
                                        li.current_index += 1;
                                        task.ram.push_i32(elem);
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        -1
    };

    // Plan 419: 元素仍在源容器 —— 裸堆 id 结果入栈 +1。
    if (result as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 {
        vm.rc_retain_id(result as u64);
    }
    task.ram.push_i32(result);
    Ok(())
}

/// Create a map adapter iterator.
/// Stack: func_addr, iterator_id -> new_iterator_id
/// Returns: new iterator_id (u32 as i32)
///
/// NOTE: For MVP, this creates the MapIterator but the actual function
/// calling during iteration is not yet implemented. The map iterator
/// will currently return an error when next() is called.
pub fn shim_iterator_map(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::{Iterator, MapIterator};

    // Stack: func_addr, iterator_id
    // Pop in reverse order (stack is LIFO)
    let func_addr = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_func_addr = crate::vm::native::StakeGuard::new(vm, func_addr as i64 as u64);
    // PLAN-776 R7-02：凭据感知源（TAG_OBJECT Sse 解包；裸 id 原语义）。
    let (source_iterator_id, from_credential) = pop_iterator_receiver(task, vm);

    let _stake_source_iterator_id =
        crate::vm::native::StakeGuard::new(vm, source_iterator_id as i64 as u64);

    // Verify source iterator exists
    if !vm.iterators.contains_key(&source_iterator_id) {
        task.ram.push_i32(-1); // Return -1 on error
        return Ok(());
    }

    let new_iterator_id = vm.register_iterator(Iterator::Map(MapIterator {
        source_iterator_id,
        func_addr,
    }));

    // Return new iterator_id（PLAN-776 R7-02：源为凭据时派生 id 同样包装）。
    push_derived_iterator(task, vm, new_iterator_id, from_credential)?;
    Ok(())
}

/// Create a filter adapter iterator.
/// Stack: func_addr, iterator_id -> new_iterator_id
/// Returns: new iterator_id (u32 as i32)
///
/// NOTE: For MVP, this creates the FilterIterator but the actual predicate
/// calling during iteration is not yet implemented. The filter iterator
/// will currently return all elements without filtering.
pub fn shim_iterator_filter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::{FilterIterator, Iterator};

    // Stack: func_addr, iterator_id
    // Pop in reverse order (stack is LIFO)
    let func_addr = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_func_addr = crate::vm::native::StakeGuard::new(vm, func_addr as i64 as u64);
    // PLAN-776 R7-02：凭据感知源（TAG_OBJECT Sse 解包；裸 id 原语义）。
    let (source_iterator_id, from_credential) = pop_iterator_receiver(task, vm);

    let _stake_source_iterator_id =
        crate::vm::native::StakeGuard::new(vm, source_iterator_id as i64 as u64);

    // Verify source iterator exists
    if !vm.iterators.contains_key(&source_iterator_id) {
        task.ram.push_i32(-1); // Return -1 on error
        return Ok(());
    }

    let new_iterator_id = vm.register_iterator(Iterator::Filter(FilterIterator {
        source_iterator_id,
        func_addr,
    }));

    // Return new iterator_id（PLAN-776 R7-02：源为凭据时派生 id 同样包装）。
    push_derived_iterator(task, vm, new_iterator_id, from_credential)?;
    Ok(())
}

/// Create an enumerate adapter iterator.
/// Plan 200 Task 3.2: Wraps a source iterator, tracking index.
/// Stack: iterator_id -> new_iterator_id
pub fn shim_iterator_enumerate(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::{EnumerateIterator, Iterator};

    // PLAN-776 R7-02：凭据感知源。
    let (source_iterator_id, from_credential) = pop_iterator_receiver(task, vm);

    let _stake_source_iterator_id =
        crate::vm::native::StakeGuard::new(vm, source_iterator_id as i64 as u64);

    if !vm.iterators.contains_key(&source_iterator_id) {
        task.ram.push_i32(-1);
        return Ok(());
    }

    let new_iterator_id = vm.register_iterator(Iterator::Enumerate(EnumerateIterator {
        source_iterator_id,
        current_index: 0,
    }));
    // PLAN-776 R7-02：派生 iterator 身份——源为凭据时同样包装。
    push_derived_iterator(task, vm, new_iterator_id, from_credential)?;
    Ok(())
}

/// Collect all elements from an iterator into a new list.
/// Stack: iterator_id -> list_id
/// Returns: new list_id (lower 32 bits of u64 as i32)
// Plan 077 Phase 6: Updated to use unified registry
pub fn shim_iterator_collect(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::Iterator;
    use crate::vm::types::ListData;

    // PLAN-776 R7-02：凭据感知源（terminal 消费同族）。
    let (iterator_id, _from_credential) = pop_iterator_receiver(task, vm);

    let _stake_iterator_id = crate::vm::native::StakeGuard::new(vm, iterator_id as i64 as u64);

    // Collect all elements from the iterator
    let mut elements = Vec::new();

    // Get the iterator and consume all elements
    if let Some(mut iter) = vm.iterators.get_mut(&iterator_id) {
        match &mut *iter {
            Iterator::List(list_iter) => {
                // Plan 077 Phase 6: Get list from unified registry
                if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                    let list_ref = obj.read().unwrap();

                    if list_ref.type_tag() == crate::vm::heap_object::TypeTag::ListInt {
                        if let Some(list_data) = list_ref.as_any().downcast_ref::<ListData<i32>>() {
                            // Collect all remaining elements
                            while list_iter.current_index < list_data.len() as u32 {
                                if let Some(&elem) = list_data.get(list_iter.current_index as usize)
                                {
                                    elements.push(elem);
                                }
                                list_iter.current_index += 1;
                            }
                        }
                    }
                }
            }
            Iterator::Map(_)
            | Iterator::Filter(_)
            | Iterator::Enumerate(_)
            | Iterator::Generator(_)
            | Iterator::HttpStream(_)
            | Iterator::AsyncHttpStream(_) => {
                // For adapters, we'd need to recursively call next()
                // For MVP, only support direct list iteration
                return Err(VMError::RuntimeError(
                    "Collect from adapters not yet implemented".to_string(),
                ));
            }
        }
    }

    // Plan 077 Phase 6: Create a new list in unified registry
    let mut list_data: ListData<i32> = ListData::new();
    for elem in elements {
        list_data.push(elem);
    }
    let list_id = vm.insert_heap_object(list_data);

    // Return list_id as i32 (lower 32 bits only for MVP)
    vm.rc_push_id(task, list_id as u64); // Plan 419

    Ok(())
}

/// Reduce all elements from an iterator using a function.
/// Stack: initial, func_addr, iterator_id -> result
/// Returns: final reduced value
///
/// NOTE: For MVP, this just sums all elements without calling the function.
pub fn shim_iterator_reduce(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::Iterator;
    use crate::vm::types::ListData;

    // PLAN-776 R7-02：凭据感知源（terminal 消费同族）。
    let (iterator_id, _from_credential) = pop_iterator_receiver(task, vm);

    let _stake_iterator_id = crate::vm::native::StakeGuard::new(vm, iterator_id as i64 as u64);
    let _func_addr = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake__func_addr = crate::vm::native::StakeGuard::new(vm, _func_addr as i64 as u64); // Not used in MVP
    let initial = crate::vm::native::pop_arg_i32(task);

    let _stake_initial = crate::vm::native::StakeGuard::new(vm, initial as i64 as u64);

    let mut result = initial;

    // Plan 077 Phase 6: Reduce all elements from the iterator
    if let Some(mut iter) = vm.iterators.get_mut(&iterator_id) {
        match &mut *iter {
            Iterator::List(list_iter) => {
                // Plan 077 Phase 6: Get list from unified registry
                if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                    let list_ref = obj.read().unwrap();

                    if list_ref.type_tag() == crate::vm::heap_object::TypeTag::ListInt {
                        if let Some(list_data) = list_ref.as_any().downcast_ref::<ListData<i32>>() {
                            // Sum all remaining elements
                            while list_iter.current_index < list_data.len() as u32 {
                                if let Some(&elem) = list_data.get(list_iter.current_index as usize)
                                {
                                    result += elem;
                                }
                                list_iter.current_index += 1;
                            }
                        }
                    }
                }
            }
            Iterator::Map(_)
            | Iterator::Filter(_)
            | Iterator::Enumerate(_)
            | Iterator::Generator(_)
            | Iterator::HttpStream(_)
            | Iterator::AsyncHttpStream(_) => {
                return Err(VMError::RuntimeError(
                    "Reduce from adapters not yet implemented".to_string(),
                ));
            }
        }
    }

    task.ram.push_i32(result);
    Ok(())
}

/// Find the first element from an iterator that matches a predicate.
/// Stack: func_addr, iterator_id -> element (or -1 if not found)
/// Returns: first matching element, or -1 if none match
///
/// NOTE: For MVP, this just returns the first element without calling the predicate.
// Plan 077 Phase 6: Updated to use unified registry
pub fn shim_iterator_find(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::Iterator;
    use crate::vm::types::ListData;

    // PLAN-776 R7-02：凭据感知源（terminal 消费同族）。
    let (iterator_id, _from_credential) = pop_iterator_receiver(task, vm);

    let _stake_iterator_id = crate::vm::native::StakeGuard::new(vm, iterator_id as i64 as u64);
    let _func_addr = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake__func_addr = crate::vm::native::StakeGuard::new(vm, _func_addr as i64 as u64); // Not used in MVP

    let mut result = -1; // Default: not found

    // Plan 077 Phase 6: Find first element from the iterator
    if let Some(mut iter) = vm.iterators.get_mut(&iterator_id) {
        match &mut *iter {
            Iterator::List(list_iter) => {
                // Plan 077 Phase 6: Get list from unified registry
                if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                    let list_ref = obj.read().unwrap();

                    if list_ref.type_tag() == crate::vm::heap_object::TypeTag::ListInt {
                        if let Some(list_data) = list_ref.as_any().downcast_ref::<ListData<i32>>() {
                            // Return first element
                            if list_iter.current_index < list_data.len() as u32 {
                                if let Some(&elem) = list_data.get(list_iter.current_index as usize)
                                {
                                    result = elem;
                                }
                                list_iter.current_index += 1;
                            }
                        }
                    }
                }
            }
            Iterator::Map(_)
            | Iterator::Filter(_)
            | Iterator::Enumerate(_)
            | Iterator::Generator(_)
            | Iterator::HttpStream(_)
            | Iterator::AsyncHttpStream(_) => {
                return Err(VMError::RuntimeError(
                    "Find from adapters not yet implemented".to_string(),
                ));
            }
        }
    }

    task.ram.push_i32(result);
    Ok(())
}

/// Create a new HashMap
/// Stack: -> hashmap_id
pub fn shim_hashmap_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map = SpecializedHashMap::new("value"); // Use StringValue variant for generic storage
    let map_id = vm.insert_heap_object(map);

    vm.rc_push_id(task, map_id as u64); // Plan 419
    Ok(())
}

/// Insert a string key with string value
/// Stack: hashmap_id, key_str_id, value_str_id -> result (0)
/// Insert a string key with any value type (str, int, bool, ref)
/// Stack: hashmap_id, key_str_id, value -> result (0)
///
/// The value is auto-detected: negative (but not bool sentinels) = string tag,
/// positive/zero/bool sentinels = int.
pub fn shim_hashmap_insert_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let value_nv = task.ram.pop_nv();
        let key_idx = task.ram.pop_str_idx();
        let map_id = task.ram.pop_i32() as u64;
        let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id);
        if let Some(obj) = vm.get_heap_object(map_id) {
            let strings = vm.strings.read().unwrap();
            let key_bytes = strings
                .get(key_idx)
                .cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            let value = if auto_val::is_string(value_nv) {
                let str_idx = auto_val::decode_string(value_nv) as usize;
                if let Some(bytes) = strings.get(str_idx) {
                    Value::Str(auto_val::AutoStr::from(
                        String::from_utf8_lossy(bytes).as_ref(),
                    ))
                } else {
                    Value::Int(0)
                }
            } else if auto_val::is_bool(value_nv) {
                Value::Bool(auto_val::decode_bool(value_nv))
            } else if auto_val::is_object(value_nv) {
                Value::VmRef(auto_val::VmRef {
                    id: auto_val::decode_object(value_nv) as usize,
                })
            } else if auto_val::is_list(value_nv) {
                Value::VmRef(auto_val::VmRef {
                    id: auto_val::decode_list(value_nv) as usize,
                })
            } else {
                Value::Int(auto_val::decode_i32(value_nv))
            };
            drop(strings);
            // PLAN-047 T-03（普查发现闭合）：本 shim 原无任何状态面 seq
            // bump（PLAN-062 遗漏，memo 全局快路径陈旧命中窗口）——补 B 类
            // 定点归因（按键名 A-able，v1 落 exact+wildcard）。
            vm.bump_path(map_id, Some(&key_str));
            let mut guard = obj.write().unwrap();
            if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
                // Plan 419: 旧值 VmRef 释放;新值 VmRef retain(死区结算配平)。
                if let Some(old) = map.get(&key_str) {
                    if let Value::VmRef(r) = old {
                        vm.rc_release_id(r.id as u64);
                    }
                }
                if let Value::VmRef(r) = &value {
                    vm.rc_retain_id(r.id as u64);
                }
                map.insert(key_str, value)
                    .map_err(|e| VMError::RuntimeError(e))?;
            }
        }
        task.ram.push_i32(0);
        return Ok(());
    }
}

/// Insert a string key with i32 value
/// Stack: hashmap_id, key_str_id, value -> result (0)
pub fn shim_hashmap_insert_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let value_nv = task.ram.pop_nv();
        let key_nv = task.ram.pop_nv();
        let map_nv = task.ram.pop_nv();
        // Key should be a string pool reference. The tag can be
        // TAG_STRING or (due to i32 round-trips in some code paths) TAG_I32.
        // Both use the same payload encoding (negative i32 = string pool index),
        // so decode_string works for both.
        if !auto_val::is_string(key_nv) && !auto_val::is_i32(key_nv) {
            return Err(VMError::RuntimeError(format!(
                "Invalid key for insert_int: key_nv={:#018x} tag={}",
                key_nv,
                auto_val::tag_of(key_nv)
            )));
        }
        let key_str_idx = auto_val::decode_string(key_nv) as usize;
        // Decode value: support i32, object ref, string ref, etc.
        let value = if auto_val::is_i32(value_nv) {
            Value::Int(auto_val::decode_i32(value_nv))
        } else if auto_val::is_object(value_nv) {
            // Store object reference as VmRef so get_str can restore the tag
            Value::VmRef(auto_val::VmRef {
                id: auto_val::decode_object(value_nv) as usize,
            })
        } else if auto_val::is_string(value_nv) {
            // Store the string pool index as an int for later retrieval
            Value::Int(auto_val::decode_string(value_nv) as i32)
        } else if auto_val::is_list(value_nv) {
            Value::VmRef(auto_val::VmRef {
                id: auto_val::decode_list(value_nv) as usize,
            })
        } else {
            Value::Int(0)
        };
        let map_id = if auto_val::is_i32(map_nv) {
            auto_val::decode_i32(map_nv) as u64
        } else if auto_val::is_object(map_nv) {
            auto_val::decode_object(map_nv) as u64
        } else {
            0
        };
        let _stake_map_id = crate::vm::native::StakeGuard::nv(vm, map_nv);
        if let Some(obj) = vm.get_heap_object(map_id) {
            let key_bytes = vm
                .strings
                .read()
                .unwrap()
                .get(key_str_idx)
                .cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            drop(key_bytes);
            let mut guard = obj.write().unwrap();
            if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
                // Plan 419: 旧值 VmRef 释放;新值 VmRef retain。
                if let Some(old) = map.get(&key_str) {
                    if let Value::VmRef(r) = old {
                        vm.rc_release_id(r.id as u64);
                    }
                }
                if let Value::VmRef(r) = &value {
                    vm.rc_retain_id(r.id as u64);
                }
                map.insert(key_str, value)
                    .map_err(|e| VMError::RuntimeError(e))?;
            }
        }
        task.ram.push_i32(0);
        return Ok(());
    }
}

/// Get value by string key
/// Stack: hashmap_id, key_str_id -> value (0 if not found)
pub fn shim_hashmap_get_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_str_idx = task.ram.pop_str_idx();
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm
                .strings
                .read()
                .unwrap()
                .get(key_str_idx)
                .cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            // Get the value from map
            if let Some(value) = map.get(&key_str) {
                match value {
                    auto_val::Value::Int(i) => {
                        task.ram.push_i32(i);
                        return Ok(());
                    }
                    auto_val::Value::Uint(u) => {
                        task.ram.push_i32(u as i32);
                        return Ok(());
                    }
                    auto_val::Value::Bool(b) => {
                        task.ram.push_i32(if b { 1 } else { 0 });
                        return Ok(());
                    }
                    auto_val::Value::Str(s) => {
                        let str_idx = vm.add_string(s.as_bytes().to_vec());
                        vm.rc_push_str_idx(task, str_idx as usize);
                        return Ok(());
                    }
                    auto_val::Value::VmRef(vm_ref) => {
                        {
                            vm.rc_push(task, auto_val::encode_object(vm_ref.id as u32));
                        }
                        return Ok(());
                    }
                    _ => {
                        // Unsupported value type — push nil
                    }
                }
            }
        }
    }

    // Not found — push empty string (bash associative-array semantics:
    // ${arr[key]} returns empty when key doesn't exist, not null/error).
    // Use contains_key to distinguish "missing" from "empty value".
    {
        let str_idx = vm.add_string(Vec::new()); // empty string
        vm.rc_push_str_idx(task, str_idx as usize);
    }
    Ok(())
}

/// Get value by string key (returns i32)
/// Stack: hashmap_id, key_str_id -> value (or nil if not found)
pub fn shim_hashmap_get_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_str_idx = task.ram.pop_str_idx();
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm
                .strings
                .read()
                .unwrap()
                .get(key_str_idx)
                .cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            drop(key_bytes);

            if let Some(value) = map.get(&key_str) {
                if let auto_val::Value::Int(i) = value {
                    task.ram.push_i32(i);
                    return Ok(());
                }
            }
        }
    }

    // Not found — push nil marker
    {
        task.ram.push_nv(auto_val::encode_null());
    }
    Ok(())
}

/// Check if key exists
/// Stack: hashmap_id, key_str_id -> result (1 if exists, 0 otherwise)
pub fn shim_hashmap_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_str_idx = task.ram.pop_str_idx();
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm
                .strings
                .read()
                .unwrap()
                .get(key_str_idx)
                .cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            if map.contains_key(&key_str) {
                1
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Remove a key-value pair
/// Stack: hashmap_id, key_str_id -> result (0)
pub fn shim_hashmap_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_str_idx = task.ram.pop_str_idx();
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
            let key_bytes = vm
                .strings
                .read()
                .unwrap()
                .get(key_str_idx)
                .cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            if let Some(value) = map.remove(&key_str) {
                match value {
                    auto_val::Value::Int(i) => {
                        task.ram.push_i32(i);
                        return Ok(());
                    }
                    auto_val::Value::Str(s) => {
                        let str_idx = vm.add_string(s.as_bytes().to_vec());
                        vm.rc_push_str_idx(task, str_idx as usize);
                        return Ok(());
                    }
                    // Plan 419: VmRef 值离开容器且不回栈 —— stake 死亡。
                    auto_val::Value::VmRef(r) => {
                        vm.rc_release_id(r.id as u64);
                        task.ram.push_i32(0);
                        return Ok(());
                    }
                    _ => {
                        task.ram.push_i32(0);
                        return Ok(());
                    }
                }
            }
        }
    }

    // Not found — push nil marker
    {
        task.ram.push_nv(auto_val::encode_null());
    }
    Ok(())
}

/// Get the number of entries
/// Stack: hashmap_id -> size
pub fn shim_hashmap_size(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    let size = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            map.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(size);
    Ok(())
}

/// Clear all entries
/// Stack: hashmap_id -> result (0)
pub fn shim_hashmap_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
            // Plan 419: 被清空条目的 VmRef stake 逐个死亡。
            for id in map.value_refs() {
                vm.rc_release_id(id);
            }
            map.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Drop the HashMap (no-op for now, heap objects are managed by Arc)
/// Stack: hashmap_id -> result (0)
/// Plan 419: 语义更新 —— rc 释放(RC 归零才真回收,"手动提前释放"直觉)。
pub fn shim_hashmap_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);
    vm.rc_release_id(map_id);
    Ok(())
}

/// Check if HashMap is empty
/// Stack: hashmap_id -> bool (1 if empty, 0 if not)
pub fn shim_hashmap_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    let is_empty = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            map.is_empty()
        } else {
            true
        }
    } else {
        true
    };

    task.ram.push_nv(auto_val::encode_bool(is_empty));
    Ok(())
}

/// Get value from HashMap with default fallback
/// Stack: hashmap_id, key_str_id, default_value -> value
pub fn shim_hashmap_get_or(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let default_val = crate::vm::native::pop_arg_i32(task);

    let _stake_default_val = crate::vm::native::StakeGuard::new(vm, default_val as i64 as u64);
    let key_str_idx = task.ram.pop_str_idx();
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm
                .strings
                .read()
                .unwrap()
                .get(key_str_idx)
                .cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            if let Some(value) = map.get(&key_str) {
                match value {
                    auto_val::Value::Int(i) => {
                        task.ram.push_i32(i);
                        return Ok(());
                    }
                    auto_val::Value::Uint(u) => {
                        task.ram.push_i32(u as i32);
                        return Ok(());
                    }
                    auto_val::Value::Bool(b) => {
                        task.ram.push_i32(if b { 1 } else { 0 });
                        return Ok(());
                    }
                    auto_val::Value::Str(s) => {
                        let str_idx = vm.add_string(s.as_bytes().to_vec());
                        vm.rc_push_str_idx(task, str_idx as usize);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    task.ram.push_i32(default_val);
    Ok(())
}

/// Get all keys from a HashMap as a List of strings
/// Stack: hashmap_id -> list_id (List of string keys)
pub fn shim_hashmap_keys(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            // Collect keys from the specialized map
            let keys: Vec<String> = match map {
                SpecializedHashMap::StringInt(m) => m.keys().cloned().collect(),
                SpecializedHashMap::StringBool(m) => m.keys().cloned().collect(),
                SpecializedHashMap::StringString(m) => m.keys().cloned().collect(),
                SpecializedHashMap::StringDouble(m) => m.keys().cloned().collect(),
                SpecializedHashMap::StringValue(m) => m.keys().cloned().collect(),
            };
            drop(guard);

            // Create a List<Value> with string entries
            use crate::vm::types::ListData;
            let mut list: ListData<auto_val::Value> = ListData::new();
            for key in keys {
                list.push(auto_val::Value::Str(auto_val::AutoStr::from(key.as_str())));
            }
            let list_id = vm.insert_heap_object(list);
            vm.rc_push_id(task, list_id as u64); // Plan 419
            return Ok(());
        }
    }

    // Empty list if map not found
    use crate::vm::types::ListData;
    let list: ListData<auto_val::Value> = ListData::new();
    let list_id = vm.insert_heap_object(list);
    vm.rc_push_id(task, list_id as u64); // Plan 419
    Ok(())
}

/// Object.keys(dynObj) -> List[str]
/// Stack: obj_id -> list_id
pub fn shim_obj_keys(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let obj_id = crate::vm::native::pop_arg_i32(task) as u64;
    let _stake = crate::vm::native::StakeGuard::new(vm, obj_id as i64 as u64);
    let entries = dyn_object_entries(vm, obj_id)?;
    push_value_list(
        task,
        vm,
        entries
            .into_iter()
            .map(|(k, _)| auto_val::Value::Str(auto_val::AutoStr::from(k.as_str())))
            .collect(),
    );
    Ok(())
}

/// Object.values(dynObj) -> List of field values
/// Stack: obj_id -> list_id
pub fn shim_obj_values(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let obj_id = crate::vm::native::pop_arg_i32(task) as u64;
    let _stake = crate::vm::native::StakeGuard::new(vm, obj_id as i64 as u64);
    let entries = dyn_object_entries(vm, obj_id)?;
    push_value_list(task, vm, entries.into_iter().map(|(_, v)| v).collect());
    Ok(())
}

/// dynObj.find(closure) -> element or None — delegates to the generic list-find
/// shim; receivers are runtime lists, identical element machinery applies.
/// dynObj.find(closure) -> element or None — delegates to the generic list-find
/// shim; receivers are runtime lists, identical element machinery applies.
///
/// PLAN-046 T2 现状注:registry/routing 层已通(命名、注册、链接三关全过,
/// V2 场景 shim 已被调用);残余缺口在下游运行时语义——动态接收者场景的
/// Option 返回表示、谓词闭包与 GET_FIELD 的协作——使端到端结果仍不正确。
/// 完整清偿规格见 auto-musk PLAN-046 T2 注 KNOWN-DEBT 046-A;建议并入
/// auto-lang 后续批次(如 plan454 队列)以本分支为基础续作。
pub fn shim_obj_find(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 454 E(缺口①收敛):不再委托通用 list-find——其 miss 约定是
    // push_i32(-1) 哨兵,而语言层 None 是 TAG_NULL(PUSH_NIL);动态接收者
    // 场景下 hit == None 永假、hit.u 退化为 0(g08 前身实测)。此处改为
    // 语言契约:命中推元素(带引用语义),穷尽推 encode_null。
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;
    let _stake_closure = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;
    let _stake_list = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);

    // Plan 454 E(§M 缺口②):只走 Value 元素路径——动态接收者场景的元素
    // 是堆对象引用(GenericInstance/ObjectData),旧 i32 快路径把存储槽的
    // tag 位误当元素值解码成 0,谓词闭包接到的接收者恒为 int 0(实测
    // pred=i32::MIN=false)。obj 字面量数组不存在纯 i32 元素的合法形态。
    let elements = get_list_elements_as_value(vm, list_id)?;
    for elem in &elements {
        push_value(task, vm, elem);
        vm.call_closure(task, closure_id, 1)?;
        let found = crate::vm::native::pop_arg_i32(task);
        if vm_is_truthy(found) {
            push_value(task, vm, elem);
            return Ok(());
        }
    }
    task.ram.push_nv(auto_val::encode_null());
    Ok(())
}

/// Create a new HashSet
/// Stack: -> hashset_id
pub fn shim_hashset_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let set = SpecializedHashSet::new();
    let set_id = vm.insert_heap_object(set);

    vm.rc_push_id(task, set_id as u64); // Plan 419
    Ok(())
}

/// Insert a string element into the set
/// Stack: hashset_id, elem_str_id -> result (0)
pub fn shim_hashset_insert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key = {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(nv).to_string()
        }
    };

    let set_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_set_id = crate::vm::native::StakeGuard::new(vm, set_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(set_id) {
        let mut guard = obj.write().unwrap();
        if let Some(set) = guard.as_any_mut().downcast_mut::<SpecializedHashSet>() {
            set.data.insert(key, ());
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Check if element exists in the set
/// Stack: hashset_id, elem_str_id -> result (1 if exists, 0 otherwise)
pub fn shim_hashset_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key = {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(nv).to_string()
        }
    };

    let set_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_set_id = crate::vm::native::StakeGuard::new(vm, set_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(set_id) {
        let guard = obj.read().unwrap();
        if let Some(set) = guard.as_any().downcast_ref::<SpecializedHashSet>() {
            if set.data.contains_key(&key) {
                1
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_nv(auto_val::encode_bool(result != 0));
    Ok(())
}

/// Remove an element from the set
/// Stack: hashset_id, elem_str_id -> result (0)
pub fn shim_hashset_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let elem_str_idx = task.ram.pop_str_idx();
    let set_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_set_id = crate::vm::native::StakeGuard::new(vm, set_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(set_id) {
        let elem_bytes = vm
            .strings
            .read()
            .unwrap()
            .get(elem_str_idx)
            .cloned()
            .ok_or(VMError::RuntimeError("Invalid element string ID".into()))?;
        let elem_str = String::from_utf8_lossy(&elem_bytes).to_string();
        drop(elem_bytes);

        let mut guard = obj.write().unwrap();
        if let Some(set) = guard.as_any_mut().downcast_mut::<SpecializedHashSet>() {
            set.data.remove(&elem_str);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get the number of elements
/// Stack: hashset_id -> size
pub fn shim_hashset_size(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let set_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_set_id = crate::vm::native::StakeGuard::new(vm, set_id as i64 as u64);

    let size = if let Some(obj) = vm.get_heap_object(set_id) {
        let guard = obj.read().unwrap();
        if let Some(set) = guard.as_any().downcast_ref::<SpecializedHashSet>() {
            set.data.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(size);
    Ok(())
}

/// Clear all elements
/// Stack: hashset_id -> result (0)
pub fn shim_hashset_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let set_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_set_id = crate::vm::native::StakeGuard::new(vm, set_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(set_id) {
        let mut guard = obj.write().unwrap();
        if let Some(set) = guard.as_any_mut().downcast_mut::<SpecializedHashSet>() {
            set.data.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Drop the HashSet (no-op for now, heap objects are managed by Arc)
/// Stack: hashset_id -> result (0)
pub fn shim_hashset_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let set_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_set_id = crate::vm::native::StakeGuard::new(vm, set_id as i64 as u64);
    vm.rc_release_id(set_id);
    Ok(())
}

/// Create a new VecDeque
/// Stack: -> deque_id
pub fn shim_vecdeque_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque = crate::vm::collections::SpecializedVecDeque::new();
    let deque_id = vm.insert_heap_object(deque);

    vm.rc_push_id(task, deque_id as u64); // Plan 419
    Ok(())
}

/// Push an element to the back
/// Stack: deque_id, elem -> result (0)
pub fn shim_vecdeque_push_back(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let elem = task.ram.pop_i32();
    let deque_id = task.ram.pop_i32() as u64;
    let _stake_deque = crate::vm::native::StakeGuard::new(vm, deque_id);

    if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedVecDeque>()
        {
            deque.data.push_back(elem);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Push an element to the front
/// Stack: deque_id, elem -> result (0)
pub fn shim_vecdeque_push_front(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let elem = task.ram.pop_i32();
    let deque_id = task.ram.pop_i32() as u64;
    let _stake_deque = crate::vm::native::StakeGuard::new(vm, deque_id);

    if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedVecDeque>()
        {
            deque.data.push_front(elem);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Pop an element from the back
/// Stack: deque_id -> elem (or 0 if empty)
pub fn shim_vecdeque_pop_back(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_deque_id = crate::vm::native::StakeGuard::new(vm, deque_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedVecDeque>()
        {
            deque.data.pop_back().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Pop an element from the front
/// Stack: deque_id -> elem (or 0 if empty)
pub fn shim_vecdeque_pop_front(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_deque_id = crate::vm::native::StakeGuard::new(vm, deque_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedVecDeque>()
        {
            deque.data.pop_front().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Get the front element
/// Stack: deque_id -> elem (or 0 if empty)
pub fn shim_vecdeque_front(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_deque_id = crate::vm::native::StakeGuard::new(vm, deque_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(deque_id) {
        let guard = obj.read().unwrap();
        if let Some(deque) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedVecDeque>()
        {
            *deque.data.front().unwrap_or(&0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Get the back element
/// Stack: deque_id -> elem (or 0 if empty)
pub fn shim_vecdeque_back(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_deque_id = crate::vm::native::StakeGuard::new(vm, deque_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(deque_id) {
        let guard = obj.read().unwrap();
        if let Some(deque) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedVecDeque>()
        {
            *deque.data.back().unwrap_or(&0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Get the size
/// Stack: deque_id -> size
pub fn shim_vecdeque_size(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_deque_id = crate::vm::native::StakeGuard::new(vm, deque_id as i64 as u64);

    let size = if let Some(obj) = vm.get_heap_object(deque_id) {
        let guard = obj.read().unwrap();
        if let Some(deque) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedVecDeque>()
        {
            deque.data.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(size);
    Ok(())
}

/// Check if empty
/// Stack: deque_id -> is_empty (1 or 0)
pub fn shim_vecdeque_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_deque_id = crate::vm::native::StakeGuard::new(vm, deque_id as i64 as u64);

    let is_empty = if let Some(obj) = vm.get_heap_object(deque_id) {
        let guard = obj.read().unwrap();
        if let Some(deque) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedVecDeque>()
        {
            deque.data.is_empty()
        } else {
            true
        }
    } else {
        true
    };

    task.ram.push_nv(auto_val::encode_bool(is_empty));
    Ok(())
}

/// Clear the deque
/// Stack: deque_id -> result (0)
pub fn shim_vecdeque_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_deque_id = crate::vm::native::StakeGuard::new(vm, deque_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedVecDeque>()
        {
            deque.data.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Drop the VecDeque (no-op, managed by Arc)
/// Stack: deque_id -> result (0)
pub fn shim_vecdeque_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let deque_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_deque_id = crate::vm::native::StakeGuard::new(vm, deque_id as i64 as u64);
    vm.rc_release_id(deque_id);
    Ok(())
}

/// Create a new BTreeMap
/// Stack: -> btreemap_id
pub fn shim_btreemap_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map = crate::vm::collections::SpecializedBTreeMap::new();
    let map_id = vm.insert_heap_object(map);

    vm.rc_push_id(task, map_id as u64); // Plan 419
    Ok(())
}

/// Insert a key-value pair
/// Stack: btreemap_id, key_str_id, value -> result (0)
pub fn shim_btreemap_insert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value = task.ram.pop_i32();
    let key_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;
    let _stake_map = crate::vm::native::StakeGuard::new(vm, map_id);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let key_bytes = vm
            .strings
            .read()
            .unwrap()
            .get(key_idx)
            .cloned()
            .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
        let key_str = String::from_utf8_lossy(&key_bytes).to_string();
        drop(key_bytes);

        let mut guard = obj.write().unwrap();
        if let Some(map) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedBTreeMap>()
        {
            map.data.insert(key_str, value);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get a value by key
/// Stack: btreemap_id, key_str_id -> value (0 if not found)
pub fn shim_btreemap_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_idx = task.ram.pop_str_idx();
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedBTreeMap>()
        {
            let key_bytes = vm
                .strings
                .read()
                .unwrap()
                .get(key_idx)
                .cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            map.data.get(&key_str).copied().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Check if key exists
/// Stack: btreemap_id, key_str_id -> result (1 if exists, 0 otherwise)
pub fn shim_btreemap_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_idx = task.ram.pop_str_idx();
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedBTreeMap>()
        {
            let key_bytes = vm
                .strings
                .read()
                .unwrap()
                .get(key_idx)
                .cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            if map.data.contains_key(&key_str) {
                1
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Remove a key-value pair
/// Stack: btreemap_id, key_str_id -> result (0)
pub fn shim_btreemap_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_idx = task.ram.pop_str_idx();
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let key_bytes = vm
            .strings
            .read()
            .unwrap()
            .get(key_idx)
            .cloned()
            .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
        let key_str = String::from_utf8_lossy(&key_bytes).to_string();
        drop(key_bytes);

        let mut guard = obj.write().unwrap();
        if let Some(map) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedBTreeMap>()
        {
            map.data.remove(&key_str);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get the size
/// Stack: btreemap_id -> size
pub fn shim_btreemap_size(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    let size = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedBTreeMap>()
        {
            map.data.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(size);
    Ok(())
}

/// Check if empty
/// Stack: btreemap_id -> is_empty (1 or 0)
pub fn shim_btreemap_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    let is_empty = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedBTreeMap>()
        {
            map.data.is_empty()
        } else {
            true
        }
    } else {
        true
    };

    task.ram.push_nv(auto_val::encode_bool(is_empty));
    Ok(())
}

/// Clear the map
/// Stack: btreemap_id -> result (0)
pub fn shim_btreemap_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedBTreeMap>()
        {
            map.data.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get the first (smallest) key
/// Stack: btreemap_id -> key_str_id (or -1 if empty)
pub fn shim_btreemap_first_key(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedBTreeMap>()
        {
            if let Some(first_key) = map.data.keys().next() {
                // Add string to pool and return tagged index(dedup)
                let str_idx = vm.add_string(first_key.as_bytes().to_vec());
                // Return as tagged string index
                -((str_idx as i32) + 1)
            } else {
                -1 // Empty map
            }
        } else {
            -1
        }
    } else {
        -1
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Get the last (largest) key
/// Stack: btreemap_id -> key_str_id (or -1 if empty)
pub fn shim_btreemap_last_key(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard
            .as_any()
            .downcast_ref::<crate::vm::collections::SpecializedBTreeMap>()
        {
            if let Some(last_key) = map.data.keys().next_back() {
                // Add string to pool and return tagged index(dedup)
                let str_idx = vm.add_string(last_key.as_bytes().to_vec());
                // Return as tagged string index
                -((str_idx as i32) + 1)
            } else {
                -1 // Empty map
            }
        } else {
            -1
        }
    } else {
        -1
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Drop the BTreeMap (no-op, managed by Arc)
/// Stack: btreemap_id -> result (0)
pub fn shim_btreemap_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let map_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_map_id = crate::vm::native::StakeGuard::new(vm, map_id as i64 as u64);
    vm.rc_release_id(map_id);
    Ok(())
}

/// alloc_array(size) -> list_id
/// Allocate a new array of the given size initialized to 0.
/// Stack: [size] -> list_id
/// Plan 390 §15 H3b: stored as ListData<Value> in heap_objects (same as
/// CREATE_ARRAY) so it works with SET_ELEM/GET_ELEM.
pub fn shim_alloc_array(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let size_raw = crate::vm::native::pop_arg_i32(task);

    let _stake_size_raw = crate::vm::native::StakeGuard::new(vm, size_raw as i64 as u64);
    if size_raw < 0 {
        return Err(VMError::RuntimeError(format!(
            "alloc_array: invalid size {} (must be >= 0)",
            size_raw
        )));
    }
    let size = size_raw as usize;
    let elems: Vec<auto_val::Value> = vec![auto_val::Value::Int(0); size];

    let array_id = vm.insert_heap_object(crate::vm::types::ListData {
        elems,
        storage: None,
    });
    vm.rc_push(task, auto_val::encode_object(array_id as u32));
    Ok(())
}

/// realloc_array([array_id, new_size]) -> array_id
/// Reallocate list to new size, preserving data.
/// Stack: [new_size, array_id] -> array_id
pub fn shim_realloc_array(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let new_size = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_new_size = crate::vm::native::StakeGuard::new(vm, new_size as i64 as u64);
    let arr_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_arr_id = crate::vm::native::StakeGuard::new(vm, arr_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(arr_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            // Plan 432 D26: 缩容时被丢弃元素的容器 stake 死亡(此前静默泄漏)。
            if new_size < list.elems.len() {
                for &e in &list.elems[new_size..] {
                    list_i32_elem_release(vm, e);
                }
            }
            list.elems.resize(new_size, 0);
            drop(guard);
            task.ram.push_i32(arr_id as i32);
            return Ok(());
        }
        // Plan 390 §15 H3b: alloc_array/CREATE_ARRAY produce ListData<Value>.
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<Value>>() {
            list.elems.resize(new_size, auto_val::Value::Int(0));
            drop(guard);
            task.ram.push_i32(arr_id as i32);
            return Ok(());
        }
    }
    // Fallback: create new list
    let mut list: ListData<i32> = ListData::new();
    for _ in 0..new_size {
        list.push(0);
    }
    let id = vm.insert_heap_object(list);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// free_array(array_id) -> nil
/// Free an array (no-op in GC-managed VM).
/// Stack: [array_id] -> nil (-2147483647)
pub fn shim_free_array(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _arr_id = crate::vm::native::pop_arg_i32(task);

    let _stake__arr_id = crate::vm::native::StakeGuard::new(vm, _arr_id as i64 as u64);
    task.ram.push_i32(-2147483647); // nil marker
    Ok(())
}

/// list.capacity() -> capacity
/// Get list capacity.
/// Stack: [list_id] -> capacity
pub fn shim_list_capacity(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            task.ram.push_i32(list.elems.capacity() as i32);
            return Ok(());
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// Create a new OnceCell (empty).
/// Stack: -> handle_id
pub fn shim_once_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cell: Option<String> = None;
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new("std::cell::OnceCell", cell);
    let id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// Set a value in the OnceCell.
/// Stack: string_value, handle_id -> void (pushes 1 for success, 0 for already set)
pub fn shim_once_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop string value
    let value = {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(nv).to_string()
        }
    };

    let handle = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_handle = crate::vm::native::StakeGuard::new(vm, handle as i64 as u64);
    let obj = vm
        .get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid OnceCell handle".to_string()))?;
    let mut guard = obj.write().unwrap();
    let rust_obj = guard
        .as_any_mut()
        .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let cell = rust_obj
        .downcast_mut::<Option<String>>()
        .ok_or_else(|| VMError::RuntimeError("Not an Option<String>".to_string()))?;
    if cell.is_none() {
        *cell = Some(value);
        task.ram.push_i32(1); // success
    } else {
        task.ram.push_i32(0); // already set
    }
    Ok(())
}

/// Get the value from the OnceCell.
/// Returns opaque handle (>=0) if set, or -1 if None.
/// Stack: handle_id -> i32 (opaque handle or -1)
pub fn shim_once_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_handle = crate::vm::native::StakeGuard::new(vm, handle as i64 as u64);
    let obj = vm
        .get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid OnceCell handle".to_string()))?;
    let guard = obj.read().unwrap();
    let rust_obj = guard
        .as_any()
        .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let cell = rust_obj
        .downcast_ref::<Option<String>>()
        .ok_or_else(|| VMError::RuntimeError("Not an Option<String>".to_string()))?;

    match cell {
        None => {
            task.ram.push_i32(-1);
        }
        Some(value) => {
            let string_obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
                "std::cell::OnceCell::Value",
                value.clone(),
            );
            let id = vm.insert_heap_object(string_obj);
            vm.rc_push_id(task, id as u64); // Plan 419
        }
    }
    Ok(())
}

/// Reverse a list in-place.
/// Stack: list_handle -> void
pub fn shim_list_reverse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    let handle = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_handle = crate::vm::native::StakeGuard::new(vm, handle as i64 as u64);
    let obj = vm
        .get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid list handle in reverse".to_string()))?;
    let mut guard = obj.write().unwrap();
    if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
        list.elems.reverse();
    }
    task.ram.push_i32(0);
    Ok(())
}

/// auto.rc.live() -> int:当前存活堆对象数(rc_stats().live_heap)。
pub fn shim_rc_live(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let live = vm.rc_stats().live_heap as i32;
    task.ram.push_i32(live);
    Ok(())
}

/// auto.rc.count(x) -> int:x 的引用计数(堆 id;非引用返回 0)。
pub fn shim_rc_count(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_id = crate::vm::native::StakeGuard::new(vm, id as i64 as u64);
    let count = vm.rc_count(id) as i32;
    task.ram.push_i32(count);
    Ok(())
}

/// auto.rc.assert_unique(x) -> x:.mut 借用的动态兜底 —— RC>1(Shared,
/// 多属主)时抛 "mutable borrow of shared value"。RefCell 式极简借用检查:
/// 以 RC>1 为冲突判据,不维护活跃借用表(plan 419 §4.4)。非引用值直通。
pub fn shim_rc_assert_unique(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let nv = task.ram.pop_nv();
    if let Some(id) = crate::vm::rc::heap_ref_id(nv) {
        let count = vm.rc_count(id);
        if count > 1 {
            return Err(VMError::RuntimeError(format!(
                "mutable borrow of shared value: heap object {} has {} owners",
                id, count
            )));
        }
    }
    task.ram.push_nv(nv);
    Ok(())
}
