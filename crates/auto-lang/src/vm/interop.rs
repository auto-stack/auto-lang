//! Plan 555 T05/T06: 互操作分发层——ForeignObject 协议 + 分发组合子。
//!
//! 设计源 `docs/design/strategy/script-mode-interop.md` §1（组合子分派）
//! 与 §8（跨语言矩阵：ForeignRef + send/get/set/call 组合子 = 可移植层，
//! 差异落各自 ForeignObject 实现）。
//!
//! 分层纪律：
//! - 本模块**不在 engine 热臂**——组合子是 native registry 注册的普通
//!   函数（`interop.obj_get` 家族），内部查运行期 tag 分派（§10 裁决
//!   "架构"行：糖→桥 lowering；组合子分派；s2s 工具先行）；
//! - ForeignObject 协议面向宿主实现者（首实现 = py_ffi 的 PyObjectHandle，
//!   JS/ArkTS 等后续宿主换实现即可接入组合子层）。

use crate::vm::engine::VMError;
use crate::vm::task::AutoTask;
use auto_val::NanoValue;

/// Plan 555 T05: 外对象协议——分发组合子的宿主分派面。
///
/// 方法语义与组合子一一对应；结果统一**推回 task 栈**（与 native shim
/// 约定一致，复用各宿主的 marshal 回程）。W1 实装六操作中协议必需的
/// 四件（get/set/len/type_name）；call/iter 变元形态较多，协议位随
/// W2 糖批接线定签名；send/contains 为跨语言矩阵预留位。
pub trait ForeignObject: Send + Sync + 'static {
    /// 宿主标识（"py" / "js" / ...），诊断与 type_name 组合子消费。
    fn foreign_kind(&self) -> &'static str;

    /// `obj_get(recv, key)` 外对象臂——宿主属性读（py: getattr）。
    /// 结果推栈。
    fn obj_get(
        &self,
        key_nv: NanoValue,
        task: &mut AutoTask,
        vm: &crate::vm::engine::AutoVM,
    ) -> Result<(), VMError>;

    /// `obj_set(recv, key, value)` 外对象臂——宿主属性写（py: setattr）。
    /// 语句形态推 null 保栈平衡（py_setitem 约定）。
    fn obj_set(
        &self,
        key_nv: NanoValue,
        value_nv: NanoValue,
        task: &mut AutoTask,
        vm: &crate::vm::engine::AutoVM,
    ) -> Result<(), VMError>;

    /// `obj_len(recv)` 外对象臂——宿主 len()（py: GIL len）。
    fn obj_len(
        &self,
        task: &mut AutoTask,
        vm: &crate::vm::engine::AutoVM,
    ) -> Result<(), VMError>;

    /// `obj_type_name(recv)` 外对象臂——宿主类型名（py: type(x).__name__）。
    fn obj_type_name(
        &self,
        task: &mut AutoTask,
        vm: &crate::vm::engine::AutoVM,
    ) -> Result<(), VMError>;

    /// `obj_call(recv, method, args...)` 外对象臂——方法调用（py: GIL
    /// call_method）。receiver 已由组合子消费；方法名与实参仍在栈上
    /// （pending_native_arg_count = 方法名 + 实参数），实现方按 shim
    /// 约定弹出消费，结果推栈。
    fn obj_call(
        &self,
        task: &mut AutoTask,
        vm: &crate::vm::engine::AutoVM,
    ) -> Result<(), VMError>;

    /// `obj_iter(recv)` 外对象臂——迭代器物化（py: iter() → 迭代器
    /// 句柄推栈）。
    fn obj_iter(
        &self,
        task: &mut AutoTask,
        vm: &crate::vm::engine::AutoVM,
    ) -> Result<(), VMError>;

    // ---- 协议位预留（跨语言矩阵 §8，W2 挂 B7/C 族时定签名）----
    // fn obj_send(&self, msg, task, vm) -> Result<(), VMError>;
    // fn obj_contains(&self, key, task, vm) -> Result<(), VMError>;
}

// ============================================================================
// Plan 555 T06: 分发组合子——运行期 tag 分派（py 句柄 → ForeignObject
// 协议 → py 桥；Auto 值 → 原生方法表）。native registry 注册的普通函数
// （`interop.obj_*` 家族 + 裸名别名），s2s 改写产物（W2 起）与跨语言
// 宿主共用的承接点。设计源 §1"分发组合子"/§8。
// ============================================================================

pub const NATIVE_INTEROP_OBJ_GET: u16 = 1860;
pub const NATIVE_INTEROP_OBJ_SET: u16 = 1861;
pub const NATIVE_INTEROP_OBJ_CALL: u16 = 1862;
pub const NATIVE_INTEROP_OBJ_LEN: u16 = 1863;
pub const NATIVE_INTEROP_OBJ_ITER: u16 = 1864;
pub const NATIVE_INTEROP_OBJ_TYPE_NAME: u16 = 1865;
// 预留：1866 obj_send / 1867 obj_contains（协议位，W2 挂 B7/C 族）。

/// 外对象分派入口：receiver 为堆对象且实现 ForeignObject 协议时执行闭包。
fn dispatch_foreign(
    vm: &crate::vm::engine::AutoVM,
    recv_nv: NanoValue,
    f: impl FnOnce(&dyn ForeignObject) -> Result<(), VMError>,
) -> Option<Result<(), VMError>> {
    let id = if auto_val::is_object(recv_nv) {
        auto_val::decode_object(recv_nv) as u64
    } else if auto_val::is_list(recv_nv) {
        auto_val::decode_list(recv_nv) as u64
    } else if auto_val::is_i32(recv_nv) {
        auto_val::decode_i32(recv_nv) as u64
    } else {
        return None;
    };
    let obj = vm.get_heap_object(id)?;
    let guard = obj.read().ok()?;
    let fo = guard.as_foreign_object()?;
    Some(f(fo))
}

/// Value → 栈推入（Auto 臂的容器元素/字段回程）。
fn push_value(vm: &crate::vm::engine::AutoVM, task: &mut AutoTask, v: &auto_val::Value) {
    use auto_val::Value;
    match v {
        Value::Int(i) => task.ram.push_i32(*i),
        Value::Uint(u) => task.ram.push_i32(*u as i32),
        Value::Bool(b) => task.ram.push_nv(auto_val::encode_bool(*b)),
        Value::Float(f) => task.ram.push_f64(*f),
        Value::Double(d) => task.ram.push_f64(*d),
        Value::Char(c) => task.ram.push_i32(*c as i32),
        Value::Str(s) => {
            let idx = vm.add_string(s.as_bytes().to_vec());
            vm.rc_push_str_idx(task, idx);
        }
        Value::VmRef(r) => vm.rc_push(task, auto_val::encode_object(r.id as u32)),
        _ => task.ram.push_i32(0),
    }
}

/// 栈 nv → Value（Auto 臂 obj_set 的字段写入转换）。
fn nv_to_value(vm: &crate::vm::engine::AutoVM, nv: NanoValue) -> auto_val::Value {
    use auto_val::Value;
    if auto_val::is_i32(nv) {
        Value::Int(auto_val::decode_i32(nv))
    } else if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv) as usize;
        let s = vm
            .strings
            .read()
            .map(|pool| {
                pool.get(idx)
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        Value::Str(s.into())
    } else if auto_val::is_bool(nv) {
        Value::Bool(auto_val::decode_bool(nv))
    } else if auto_val::is_f64(nv) {
        Value::Double(auto_val::decode_f64(nv))
    } else if auto_val::is_f32(nv) {
        Value::Float(auto_val::decode_f32(nv) as f64)
    } else if auto_val::is_null(nv) {
        Value::Nil
    } else if auto_val::is_object(nv) {
        Value::VmRef(auto_val::VmRef { id: auto_val::decode_object(nv) as usize })
    } else {
        Value::Int(auto_val::decode_i32(nv))
    }
}

/// 读字符串 nv（属性名通道）。
fn nv_string(vm: &crate::vm::engine::AutoVM, nv: NanoValue) -> Option<String> {
    if !auto_val::is_string(nv) {
        return None;
    }
    let idx = auto_val::decode_string(nv) as usize;
    vm.strings
        .read()
        .ok()?
        .get(idx)
        .map(|b| String::from_utf8_lossy(b).to_string())
}

/// Plan 555 T06: 组合子注册（engine.rs AutoVM::new 调用）。
pub fn register_interop_natives(ni: &mut crate::vm::native::NativeInterface) {
    use crate::vm::engine::AutoVM;

    // ---- obj_get(x, key) ----
    ni.register(NATIVE_INTEROP_OBJ_GET, |task: &mut AutoTask, vm: &AutoVM| {
        let n = task.pending_native_arg_count as usize;
        if n != 2 {
            return Err(VMError::RuntimeError(format!(
                "obj_get needs 2 args (obj, key), got {}",
                n
            )));
        }
        let key_nv = task.ram.pop_nv();
        let recv_nv = task.ram.pop_nv();
        // 外对象臂：协议分派（py_getattr 等宿主语义）。
        if let Some(r) = dispatch_foreign(vm, recv_nv, |fo| fo.obj_get(key_nv, task, vm)) {
            return r;
        }
        // Auto 臂：str[i] 字符码点 / list[i] 元素 / map["k"] 字段。
        if let Some(r) = auto_get(vm, task, recv_nv, key_nv) {
            return r;
        }
        Err(VMError::RuntimeError(
            "obj_get: unsupported receiver".to_string(),
        ))
    });

    // ---- obj_set(x, key, value) ----
    ni.register(NATIVE_INTEROP_OBJ_SET, |task: &mut AutoTask, vm: &AutoVM| {
        let n = task.pending_native_arg_count as usize;
        if n != 3 {
            return Err(VMError::RuntimeError(format!(
                "obj_set needs 3 args (obj, key, value), got {}",
                n
            )));
        }
        let value_nv = task.ram.pop_nv();
        let key_nv = task.ram.pop_nv();
        let recv_nv = task.ram.pop_nv();
        if let Some(r) =
            dispatch_foreign(vm, recv_nv, |fo| fo.obj_set(key_nv, value_nv, task, vm))
        {
            return r;
        }
        if let Some(r) = auto_set(vm, task, recv_nv, key_nv, value_nv) {
            return r;
        }
        Err(VMError::RuntimeError(
            "obj_set: unsupported receiver".to_string(),
        ))
    });

    // ---- obj_call(x, method, args...) ----
    ni.register(NATIVE_INTEROP_OBJ_CALL, |task: &mut AutoTask, vm: &AutoVM| {
        let n = task.pending_native_arg_count as usize;
        if n < 2 {
            return Err(VMError::RuntimeError(format!(
                "obj_call needs at least 2 args (obj, method), got {}",
                n
            )));
        }
        // receiver 在栈底；外对象臂：倒装弹出 receiver 后，协议臂按
        // shim 约定弹方法名+实参。
        let recv_nv = task.ram.peek_nv(n - 1);
        if let Some(id) = heap_id_of(recv_nv) {
            if let Some(obj) = vm.get_heap_object(id) {
                let is_foreign = {
                    let guard = obj.read().unwrap();
                    guard.as_foreign_object().is_some()
                };
                if is_foreign {
                    let mut saved = Vec::with_capacity(n - 1);
                    for _ in 0..(n - 1) {
                        saved.push(task.ram.pop_nv());
                    }
                    let _recv = task.ram.pop_nv(); // 已消费（协议臂以 self 为 receiver）
                    for nv in saved.into_iter().rev() {
                        task.ram.push_nv(nv);
                    }
                    let guard = obj.read().unwrap();
                    let fo = guard.as_foreign_object().unwrap();
                    return fo.obj_call(task, vm);
                }
            }
        }
        // Auto 臂：非外对象不可调用（对标 550 callable 守卫语义；Auto
        // 闭包经组合子的动态调用面归 W2 糖批）。
        Err(VMError::RuntimeError(
            "TypeError: object is not callable".to_string(),
        ))
    });

    // ---- obj_len(x) ----
    ni.register(NATIVE_INTEROP_OBJ_LEN, |task: &mut AutoTask, vm: &AutoVM| {
        let n = task.pending_native_arg_count as usize;
        if n != 1 {
            return Err(VMError::RuntimeError(format!(
                "obj_len needs 1 arg, got {}",
                n
            )));
        }
        let recv_nv = task.ram.pop_nv();
        if let Some(r) = dispatch_foreign(vm, recv_nv, |fo| fo.obj_len(task, vm)) {
            return r;
        }
        // Auto 臂：ARRAY_LEN 语义（str 字符数 / ListData 长度 / ObjectData 字段数）。
        if auto_val::is_string(recv_nv) {
            let idx = auto_val::decode_string(recv_nv) as usize;
            let len = vm
                .strings
                .read()
                .unwrap()
                .get(idx)
                .map(|b| String::from_utf8_lossy(b).chars().count() as i32)
                .unwrap_or(0);
            task.ram.push_i32(len);
            return Ok(());
        }
        match heap_id_of(recv_nv) {
            Some(id) => {
                if let Some(obj) = vm.get_heap_object(id) {
                    let guard = obj.read().unwrap();
                    use crate::vm::types::ListData;
                    let len = if let Some(l) = guard.as_any().downcast_ref::<ListData<i32>>() {
                        l.elems.len() as i32
                    } else if let Some(l) = guard.as_any().downcast_ref::<ListData<String>>() {
                        l.elems.len() as i32
                    } else if let Some(l) = guard.as_any().downcast_ref::<ListData<bool>>() {
                        l.elems.len() as i32
                    } else if let Some(l) = guard
                        .as_any()
                        .downcast_ref::<ListData<auto_val::Value>>()
                    {
                        l.elems.len() as i32
                    } else if let Some(o) = guard
                        .as_any()
                        .downcast_ref::<crate::vm::types::ObjectData>()
                    {
                        o.fields.len() as i32
                    } else {
                        0
                    };
                    drop(guard);
                    task.ram.push_i32(len);
                    return Ok(());
                }
                task.ram.push_i32(0);
                Ok(())
            }
            None => {
                task.ram.push_i32(0);
                Ok(())
            }
        }
    });

    // ---- obj_iter(x) ----
    ni.register(NATIVE_INTEROP_OBJ_ITER, |task: &mut AutoTask, vm: &AutoVM| {
        let n = task.pending_native_arg_count as usize;
        if n != 1 {
            return Err(VMError::RuntimeError(format!(
                "obj_iter needs 1 arg, got {}",
                n
            )));
        }
        let recv_nv = task.ram.pop_nv();
        if let Some(r) = dispatch_foreign(vm, recv_nv, |fo| fo.obj_iter(task, vm)) {
            return r;
        }
        // Auto 臂：list/str 原样回推（array 通道 for-in 直接消费——
        // ARRAY_LEN+GET_ELEM 索引循环，设计 E3 双通道的 Auto 侧）。
        if auto_val::is_string(recv_nv) || auto_val::is_list(recv_nv) || auto_val::is_object(recv_nv)
        {
            task.ram.push_nv(recv_nv);
            return Ok(());
        }
        Err(VMError::RuntimeError(
            "TypeError: object is not iterable".to_string(),
        ))
    });

    // ---- obj_type_name(x) ----
    ni.register(NATIVE_INTEROP_OBJ_TYPE_NAME, |task: &mut AutoTask, vm: &AutoVM| {
        let n = task.pending_native_arg_count as usize;
        if n != 1 {
            return Err(VMError::RuntimeError(format!(
                "obj_type_name needs 1 arg, got {}",
                n
            )));
        }
        let recv_nv = task.ram.pop_nv();
        if let Some(r) = dispatch_foreign(vm, recv_nv, |fo| fo.obj_type_name(task, vm)) {
            return r;
        }
        // Auto 臂：nv_py_type_name 家族（550）+ 容器具体名。
        let name = if auto_val::is_string(recv_nv) {
            "str"
        } else if auto_val::is_list(recv_nv) {
            "list"
        } else if auto_val::is_object(recv_nv) {
            match heap_id_of(recv_nv).and_then(|id| vm.get_heap_object(id)) {
                Some(obj) => {
                    let guard = obj.read().unwrap();
                    if guard
                        .as_any()
                        .downcast_ref::<crate::vm::types::ObjectData>()
                        .is_some()
                    {
                        "map"
                    } else if guard
                        .as_any()
                        .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                        .is_some()
                    {
                        "list"
                    } else {
                        "object"
                    }
                }
                None => "object",
            }
        } else {
            crate::vm::virt_memory::nv_py_type_name(recv_nv)
        };
        let idx = vm.add_string(name.as_bytes().to_vec());
        vm.rc_push_str_idx(task, idx);
        Ok(())
    });
}

/// receiver nv 的堆 id（object/list tag；历史裸 i32 id 亦认）。
fn heap_id_of(recv_nv: NanoValue) -> Option<u64> {
    if auto_val::is_object(recv_nv) {
        Some(auto_val::decode_object(recv_nv) as u64)
    } else if auto_val::is_list(recv_nv) {
        Some(auto_val::decode_list(recv_nv) as u64)
    } else if auto_val::is_i32(recv_nv) {
        Some(auto_val::decode_i32(recv_nv) as u64)
    } else {
        None
    }
}

/// Auto 值的 obj_get 臂：str[i] / list[i] / map["k"]。
fn auto_get(
    vm: &crate::vm::engine::AutoVM,
    task: &mut AutoTask,
    recv_nv: NanoValue,
    key_nv: NanoValue,
) -> Option<Result<(), VMError>> {
    // str[i] → 字符码点
    if auto_val::is_string(recv_nv) && auto_val::is_i32(key_nv) {
        let idx_nv = auto_val::decode_i32(key_nv);
        let sidx = auto_val::decode_string(recv_nv) as usize;
        let s = vm
            .strings
            .read()
            .ok()?
            .get(sidx)
            .map(|b| String::from_utf8_lossy(b).to_string())?;
        let chars: Vec<char> = s.chars().collect();
        let i = if idx_nv >= 0 {
            idx_nv as usize
        } else {
            chars.len().checked_sub((-idx_nv) as usize)?
        };
        let ch = *chars.get(i)?;
        task.ram.push_i32(ch as i32);
        return Some(Ok(()));
    }
    let id = heap_id_of(recv_nv)?;
    let obj = vm.get_heap_object(id)?;
    let guard = obj.read().ok()?;
    use crate::vm::types::{ListData, ObjectData};
    // list[i]
    if auto_val::is_i32(key_nv) {
        let i = auto_val::decode_i32(key_nv);
        if let Some(l) = guard.as_any().downcast_ref::<ListData<i32>>() {
            let n = l.elems.len() as i32;
            let idx = if i >= 0 { i } else { n.checked_add(i)? };
            if idx < 0 || idx >= n {
                return Some(Err(VMError::RuntimeError(format!(
                    "IndexError: index {} out of range",
                    i
                ))));
            }
            let elem = l.elems[idx as usize];
            if elem < 0 {
                // 负元素=字符串池索引约定（GET_ELEM 同款保 tag）
                let str_idx = (-elem - 1) as u32;
                vm.rc_push_str_idx(task, str_idx as usize);
            } else {
                task.ram.push_i32(elem);
            }
            return Some(Ok(()));
        }
        if let Some(l) = guard.as_any().downcast_ref::<ListData<auto_val::Value>>() {
            let n = l.elems.len() as i32;
            let idx = if i >= 0 { i } else { n.checked_add(i)? };
            if idx < 0 || idx >= n {
                return Some(Err(VMError::RuntimeError(format!(
                    "IndexError: index {} out of range",
                    i
                ))));
            }
            let val = l.elems[idx as usize].clone();
            drop(guard);
            push_value(vm, task, &val);
            return Some(Ok(()));
        }
        return None;
    }
    // map["k"]（ObjectData 按名）
    if auto_val::is_string(key_nv) {
        let key = nv_string(vm, key_nv)?;
        if let Some(o) = guard.as_any().downcast_ref::<ObjectData>() {
            let v = o.get(&auto_val::ValueKey::Str(key.clone().into())).cloned();
            drop(guard);
            match v {
                Some(val) => push_value(vm, task, &val),
                None => task.ram.push_i32(0), // 缺字段 0 哨兵（GET_FIELD 同款）
            }
            return Some(Ok(()));
        }
    }
    None
}

/// Auto 值的 obj_set 臂：list[i]=v / map["k"]=v。
fn auto_set(
    vm: &crate::vm::engine::AutoVM,
    task: &mut AutoTask,
    recv_nv: NanoValue,
    key_nv: NanoValue,
    value_nv: NanoValue,
) -> Option<Result<(), VMError>> {
    let id = heap_id_of(recv_nv)?;
    let obj = vm.get_heap_object(id)?;
    let mut guard = obj.write().ok()?;
    use crate::vm::types::{ListData, ObjectData};
    if auto_val::is_i32(key_nv) {
        let i = auto_val::decode_i32(key_nv);
        if let Some(l) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            let n = l.elems.len() as i32;
            let idx = if i >= 0 { i } else { n.checked_add(i)? };
            if idx < 0 || idx >= n {
                return Some(Err(VMError::RuntimeError(format!(
                    "IndexError: index {} out of range",
                    i
                ))));
            }
            if auto_val::is_i32(value_nv) {
                l.elems[idx as usize] = auto_val::decode_i32(value_nv);
            }
            drop(guard);
            task.ram.push_nv(auto_val::encode_null()); // 语句形态保栈平衡
            return Some(Ok(()));
        }
        if let Some(l) = guard
            .as_any_mut()
            .downcast_mut::<ListData<auto_val::Value>>()
        {
            let n = l.elems.len() as i32;
            let idx = if i >= 0 { i } else { n.checked_add(i)? };
            if idx < 0 || idx >= n {
                return Some(Err(VMError::RuntimeError(format!(
                    "IndexError: index {} out of range",
                    i
                ))));
            }
            l.elems[idx as usize] = nv_to_value(vm, value_nv);
            drop(guard);
            task.ram.push_nv(auto_val::encode_null());
            return Some(Ok(()));
        }
        return None;
    }
    if auto_val::is_string(key_nv) {
        let key = nv_string(vm, key_nv)?;
        if let Some(o) = guard.as_any_mut().downcast_mut::<ObjectData>() {
            o.set(
                auto_val::ValueKey::Str(key.into()),
                nv_to_value(vm, value_nv),
            );
            drop(guard);
            task.ram.push_nv(auto_val::encode_null());
            return Some(Ok(()));
        }
    }
    None
}
