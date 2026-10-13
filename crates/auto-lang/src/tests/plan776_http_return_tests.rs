//! PLAN-776 T-01 表示探针——HTTP 返回资源凭据的载体假设冻结（决策报告 §6）。
//!
//! 冻结方案 A：资源凭据 = heap_objects 表 entry（TypeTag 专属变体，T-02 落地），
//! NanoValue 侧 TAG_OBJECT `encode_object`，Value 侧 `Value::VmRef`。本探针用
//! 现有 `RustStdlibObject` 作资源对象替身，验证三条地基假设：
//!
//! (1) **传播保身份**：TAG_OBJECT 凭据经 globals / FutureValue.result /
//!     closure captures（Value::VmRef 载体）传播后 downcast 不变；
//! (2) **数字不可伪造**：任意 encode_i32/encode_i64——包括与活跃 heap id
//!     同值、Future 0xF0 位模式、算术结果——is_object 恒 false；
//! (3) **查表制对照**：现状 `insert_http_response`/`lookup_http_response`
//!     是表命中制（本计划要叠加资源臂的兼容面），同值 int 不是对象。
//!
//! 正式资源类型与编组分派由 T-02/T-03 交付；本文件随后扩展为 §6 全矩阵。

use crate::vm::engine::{AutoVM, FutureKind, FutureState, FutureValue};
use crate::vm::ffi::rust_stdlib::RustStdlibObject;
use crate::vm::ffi::stdlib;
use crate::vm::virt_memory::VirtualFlash;

fn fresh_vm() -> AutoVM {
    AutoVM::new(VirtualFlash::new(1024), 1024)
}

/// 探针 (1)：对象凭据的 NanoValue/Value 双载体在共享全局、Future 终值、
/// closure 捕获三传播形态下身份保持（AC-02 的表示地基）。
#[test]
fn plan776_probe_resource_identity_survives_propagation() {
    let vm = fresh_vm();
    let sentinel = 776_776u64;
    let handle = vm.insert_heap_object(RustStdlibObject::new(
        "plan776.probe.Sentinel",
        sentinel,
    ));
    let nv = auto_val::encode_object(handle as u32);
    assert!(auto_val::is_object(nv), "资源凭据必须是 TAG_OBJECT 载体");

    // 共享全局（NanoValue 传播）。
    vm.globals.insert("plan776_probe_resource".to_string(), nv);
    let back = vm
        .globals
        .get("plan776_probe_resource")
        .expect("globals 读回");
    assert!(auto_val::is_object(*back), "globals 传播丢 TAG_OBJECT");
    assert_eq!(auto_val::decode_object(*back) as u64, handle);

    // Future 终值（Value::VmRef 传播——External/Internal 终值同一载体）。
    let value = auto_val::Value::VmRef(auto_val::VmRef { id: handle as usize });
    vm.futures.insert(
        1,
        std::sync::Arc::new(std::sync::RwLock::new(FutureValue {
            body_offset: 0,
            state: FutureState::Ready,
            result: Some(value),
            owner_task_id: 0,
            captures: std::collections::HashMap::new(),
            kind: FutureKind::External,
        })),
    );
    let fut = vm.futures.get(&1).expect("future 读回");
    let result = fut.read().unwrap().result.clone().expect("result 在位");
    match result {
        auto_val::Value::VmRef(r) => assert_eq!(r.id as u64, handle),
        other => panic!("Future 终值载体应为 VmRef，实际 {other:?}"),
    }
    drop(fut);

    // closure 捕获（captures: HashMap<String, Value>）。
    let mut fut2 = vm.futures.get_mut(&1).expect("future 可变读");
    fut2.write().unwrap().captures.insert(
        "res".to_string(),
        auto_val::Value::VmRef(auto_val::VmRef { id: handle as usize }),
    );
    drop(fut2);
    let cap = vm
        .futures
        .get(&1)
        .unwrap()
        .read()
        .unwrap()
        .captures
        .get("res")
        .cloned()
        .expect("capture 在位");
    match cap {
        auto_val::Value::VmRef(r) => assert_eq!(r.id as u64, handle),
        other => panic!("closure 捕获载体应为 VmRef，实际 {other:?}"),
    }

    // 三形态读回后 downcast 均成功（身份未衰减为普通值）。
    let obj = vm.get_heap_object(handle).expect("堆对象存活");
    let guard = obj.read().unwrap();
    let ro = guard
        .as_any()
        .downcast_ref::<RustStdlibObject>()
        .expect("downcast 保持（传播后仍是同一对象类型）");
    let inner = ro
        .downcast_ref::<u64>()
        .expect("载荷类型保持");
    assert_eq!(*inner, sentinel);
}

/// 探针 (2)：普通数字永无资源身份（AC-01 的表示地基）。
/// 与活跃 heap id 同值的 i32/i64、Future 0xF0 位模式、算术结果——
/// 任何可由数值运算制造的形态都不是 TAG_OBJECT。
#[test]
fn plan776_probe_plain_numbers_never_gain_resource_identity() {
    let vm = fresh_vm();
    let handle = vm.insert_heap_object(RustStdlibObject::new(
        "plan776.probe.Collision",
        1u32,
    ));
    // 同值普通数（真碰撞形态：id 已知且资源活跃）。
    let i32_same = auto_val::encode_i32(handle as i32);
    let i64_same = auto_val::try_encode_i64(handle as i64)
        .expect("小值 i64 应可 48 位内联编码");
    assert!(!auto_val::is_object(i32_same), "同值 i32 不得命中对象 tag");
    assert!(!auto_val::is_object(i64_same), "同值 i64 不得命中对象 tag");
    assert!(auto_val::is_i32(i32_same) && auto_val::is_i64(i64_same));

    // Future 探测位模式（marshal 0xF0 探测的输入形态）仍是普通数。
    let fid = 42u32;
    let future_pattern = auto_val::encode_i32(((fid << 8) | 0xF0) as i32);
    assert!(!auto_val::is_object(future_pattern));

    // 算术结果不可能跨 tag：对象 tag 位型无法由 int 运算制造。
    let computed = auto_val::encode_i32((auto_val::decode_i32(i32_same) + 1) - 1);
    assert!(!auto_val::is_object(computed));
    assert!(auto_val::is_i32(computed));

    // TAG_OBJECT 只能经 encode_object（受控点）产生。
    let real = auto_val::encode_object(handle as u32);
    assert!(auto_val::is_object(real));
    assert_ne!(real, i32_same);
    assert_ne!(real, i64_same);
}

/// 探针 (3)：现状查表制对照——insert/lookup_http_response 是 id 表命中，
/// 同值 int 的 NanoValue 仍是数据形态。资源臂（T-03）叠加后，真实资源走
/// 对象分派，legacy int 继续查表（6 个 wire 用例兼容面）。
#[test]
fn plan776_probe_legacy_registry_lookup_vs_object_identity() {
    let handle = stdlib::insert_http_response(
        302,
        vec![("Location".to_string(), "/moved".to_string())],
        b"moved".to_vec(),
    );
    // 表命中制：lookup 命中并给出 owned 快照（clone 不消费——重复返回语义）。
    let first = stdlib::lookup_http_response(handle).expect("首次查表命中");
    assert_eq!(first.0, 302);
    let second = stdlib::lookup_http_response(handle).expect("重复查表仍命中");
    assert_eq!(second.0, 302, "lookup 不消费（Response 可重复返回的现行为）");

    // 同值 int 是数据：查表命中不等于持有对象身份。
    let as_int = auto_val::encode_i32(handle as i32);
    assert!(!auto_val::is_object(as_int));
    // 未登记 id 查表不命中（表命中制无凭据校验的对照面）。
    assert!(stdlib::lookup_http_response(handle + 1).is_none());
}

// ============================================================================
// T-02：producer 接线（shim 级——构造点返回 TAG_OBJECT 凭据）
// ============================================================================

use crate::vm::task::AutoTask;
use crate::vm::ffi::http_return_resource::{self, ResourceClaim};

fn fresh_task() -> AutoTask {
    AutoTask::new(0, 1024, 0)
}

/// redirect 构造点：`http.response.redirect(url, 302)` → Response 凭据对象
/// （不进 HTTP_RESPONSES 表；快照 302 + Location）。
#[test]
fn plan776_t02_redirect_shim_pushes_resource() {
    let vm = fresh_vm();
    let mut task = fresh_task();
    use crate::vm::ffi::VMConvertible;
    "/next".to_string().push_to_stack(&mut task, &vm).unwrap();
    task.ram.push_i32(302);
    stdlib::shim_http_response_redirect(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_object(nv), "redirect 返回凭据对象");
    match http_return_resource::resolve_resource(&vm, nv).unwrap() {
        ResourceClaim::Response(s) => {
            assert_eq!(s.status, 302);
            assert_eq!(s.headers[0].1, "/next");
        }
        other => panic!("错种类：{other:?}"),
    }
}

/// musk ctor 接线：`ok_response(v)` → 200 JSON Response 凭据。
#[test]
fn plan776_t02_musk_ok_response_pushes_resource() {
    let vm = fresh_vm();
    let mut task = fresh_task();
    task.ram.push_i32(7);
    crate::vm::ffi::musk_response_ctor::shim_ok_response(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_object(nv));
    match http_return_resource::resolve_resource(&vm, nv).unwrap() {
        ResourceClaim::Response(s) => {
            assert_eq!(s.status, 200);
            assert_eq!(s.body, b"7".to_vec());
            assert_eq!(s.headers[0].1, "application/json");
        }
        other => panic!("错种类：{other:?}"),
    }
}

/// file 构造点：`http.file_response(root, path, "{}")` → File 凭据对象 +
/// 描述符表项在位（take 单次移交合同保留）。
#[test]
fn plan776_t02_file_response_shim_pushes_resource() {
    let vm = fresh_vm();
    let mut task = fresh_task();
    use crate::vm::ffi::VMConvertible;
    "root".to_string().push_to_stack(&mut task, &vm).unwrap();
    "x.bin".to_string().push_to_stack(&mut task, &vm).unwrap();
    "{}".to_string().push_to_stack(&mut task, &vm).unwrap();
    crate::vm::ffi::http_server_file::shim_http_file_response(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_object(nv));
    match http_return_resource::resolve_resource(&vm, nv).unwrap() {
        ResourceClaim::File { descriptor_id } => {
            // 表项在位（take 可消费——单次移交）。
            assert!(crate::vm::ffi::http_server_file::take_file_response(descriptor_id).is_some());
            assert!(crate::vm::ffi::http_server_file::take_file_response(descriptor_id).is_none());
        }
        other => panic!("错种类：{other:?}"),
    }
}
