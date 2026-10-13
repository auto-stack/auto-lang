//! PLAN-776 T-02：HTTP 返回资源凭据——handler 成功终值的按值身份载体。
//!
//! 决策报告（`docs/plans/reports/776-return-resource-decision.md` §2/§3）冻结
//! 方案 A：资源凭据 = `heap_objects` 表 entry（`TypeTag::HttpReturnResource`），
//! NanoValue 侧现成 `TAG_OBJECT`（`encode_object`），Value 侧现成
//! `Value::VmRef`。普通 i32/i64 是 `TAG_I32/TAG_I64`——任何数值运算都不能
//! 制造资源身份（AC-01）；heap 表 per-VM，编组持 `vm` 查表即 owner 校验
//! （AC-05）；heap id 单调递增不复用（无旧引用复活）；凭据随 RC 归零回收。
//!
//! 各 kind 的数据 backing 不动（SSE iterator 仍在 `vm.iterators`、File 描述符
//! 仍在 `VM_FILE_RESPONSES`、Upload 收据仍在 `VM_UPLOAD_RECEIPTS`——take 单次
//! 移交合同保持）；Response 快照直接 owned 在凭据内（重复返回 = 每次 clone，
//! 不经全局表）。legacy int 面（`http.response()`+setter 链、client 句柄、
//! Plan 326/346 六 wire 用例）保持原查表路径。

use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::heap_object::{HeapObject, TypeTag};
use crate::vm::task::AutoTask;

/// 全局存活资源凭据计数（构造 +1 / Drop -1）——AC-06 存活回基线观察钩子
/// （测试读取；生产日志诊断用）。
static LIVE_RESOURCES: AtomicUsize = AtomicUsize::new(0);

/// 最近一次构造的凭据 heap id（碰撞族测试观察钩子——仅 cfg(test) 读取；
/// 生产侧 store 一次 AtomicU64 开销可忽略，不构成可伪造凭据的公共 API：
/// 读到的数值是普通数字，无 TAG_OBJECT 位型）。
static LAST_HANDLE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 测试观察：最近构造的凭据 heap id（真碰撞 oracle 的实际 id 来源）。
#[cfg(test)]
pub fn probe_last_resource_handle() -> u64 {
    LAST_HANDLE.load(Ordering::Relaxed)
}

/// owned HTTP 响应快照（与 stdlib::HttpResponseData 同形——凭据内独立 owned
/// 副本，重复返回 clone 不消费；mutable setter 链继续走 stdlib 表，互不串扰）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResponseSnapshot {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// 凭据种类（决策报告 §3.1）。
#[derive(Debug, Clone)]
pub enum HttpReturnKind {
    /// owned status/headers/body 快照（redirect / musk ctor / 显式 Response 构造）。
    Response(ResponseSnapshot),
    /// SSE 输出凭据：底层 iterator 仍在 `vm.iterators`（注册机制不动；
    /// `Sse.into_response` 装箱点）。
    Sse { iterator_id: u32 },
    /// 文件描述符凭据：数据仍在 `VM_FILE_RESPONSES`（take 单次移交；
    /// scope 组收口合同不动——本凭据 Drop 不触碰表，避免双重释放）。
    File { descriptor_id: u64 },
    /// 上传收据凭据：数据仍在 `VM_UPLOAD_RECEIPTS`（take 单次移交；同上）。
    Upload { receipt_id: u64 },
}

/// HTTP 返回资源凭据对象（heap_objects 表 entry；RC 生命周期）。
#[derive(Debug, Clone)]
pub struct HttpReturnResource {
    pub kind: HttpReturnKind,
}

impl Drop for HttpReturnResource {
    fn drop(&mut self) {
        LIVE_RESOURCES.fetch_sub(1, Ordering::Relaxed);
    }
}

impl HeapObject for HttpReturnResource {
    fn type_tag(&self) -> TypeTag {
        TypeTag::HttpReturnResource
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// 存活凭据计数（观察钩子）。
pub fn live_resource_count() -> usize {
    LIVE_RESOURCES.load(Ordering::Relaxed)
}

/// 构造 + 入堆 + RC push（`push_rust_obj` 同款模板——RC 簿记齐全）。
/// heap id 单调递增；回绕（fetch_add 溢出回卷）在分配前拒绝——u64 空间
/// 现实不可达，防御性失败而非重用使旧引用复活。
fn push_kind(task: &mut AutoTask, vm: &AutoVM, kind: HttpReturnKind) -> Result<(), VMError> {
    if vm.heap_object_id_gen.load(Ordering::Relaxed) < crate::vm::rc::HEAP_ID_BASE {
        return Err(VMError::RuntimeError(
            "http return resource heap id space exhausted".into(),
        ));
    }
    LIVE_RESOURCES.fetch_add(1, Ordering::Relaxed);
    let handle = vm.insert_heap_object(HttpReturnResource { kind });
    LAST_HANDLE.store(handle, Ordering::Relaxed);
    vm.rc_push(task, auto_val::encode_object(handle as u32));
    Ok(())
}

/// Response 快照凭据（redirect / musk ctor 8 族构造点）。
pub fn push_response_resource(
    task: &mut AutoTask,
    vm: &AutoVM,
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> Result<(), VMError> {
    push_kind(
        task,
        vm,
        HttpReturnKind::Response(ResponseSnapshot {
            status,
            headers,
            body,
        }),
    )
}

/// SSE iterator 凭据（`Sse.into_response` 装箱点）。
pub fn push_sse_resource(
    task: &mut AutoTask,
    vm: &AutoVM,
    iterator_id: u32,
) -> Result<(), VMError> {
    push_kind(task, vm, HttpReturnKind::Sse { iterator_id })
}

/// File 描述符凭据（`http.file_response` 构造点；表 insert 由调用方完成）。
pub fn push_file_resource(
    task: &mut AutoTask,
    vm: &AutoVM,
    descriptor_id: u64,
) -> Result<(), VMError> {
    push_kind(task, vm, HttpReturnKind::File { descriptor_id })
}

/// Upload 收据凭据（upload_commit/reject 终值点；表 insert 由调用方完成）。
pub fn push_upload_resource(
    task: &mut AutoTask,
    vm: &AutoVM,
    receipt_id: u64,
) -> Result<(), VMError> {
    push_kind(task, vm, HttpReturnKind::Upload { receipt_id })
}

/// 编组侧凭据解析视图（owned clone——快照重复返回 / id 供 take）。
#[derive(Debug, Clone)]
pub enum ResourceClaim {
    Response(ResponseSnapshot),
    Sse { iterator_id: u32 },
    File { descriptor_id: u64 },
    Upload { receipt_id: u64 },
}

/// 解析 NanoValue 为本 VM 的资源凭据视图。
///
/// 非 TAG_OBJECT / 堆查无（跨 owner、已回收）/ downcast 失败（非资源对象）
/// 一律 `None`——凭据不可凭数值命中（AC-01/AC-05）。
pub fn resolve_resource(vm: &AutoVM, nv: auto_val::NanoValue) -> Option<ResourceClaim> {
    if !auto_val::is_object(nv) {
        return None;
    }
    let handle = auto_val::decode_object(nv) as u64;
    read_claim(vm, handle)
}

/// 按 Value::VmRef 解析（async 终值路径——`FutureValue.result` 载体）。
pub fn resolve_resource_value(vm: &AutoVM, value: &auto_val::Value) -> Option<ResourceClaim> {
    match value {
        auto_val::Value::VmRef(r) => read_claim(vm, r.id as u64),
        _ => None,
    }
}

fn read_claim(vm: &AutoVM, handle: u64) -> Option<ResourceClaim> {
    let obj = vm.get_heap_object(handle)?;
    let guard = obj.read().ok()?;
    let res = guard
        .as_any()
        .downcast_ref::<HttpReturnResource>()
        .filter(|r| r.type_tag() == TypeTag::HttpReturnResource)?;
    Some(match &res.kind {
        HttpReturnKind::Response(s) => ResourceClaim::Response(s.clone()),
        HttpReturnKind::Sse { iterator_id } => ResourceClaim::Sse {
            iterator_id: *iterator_id,
        },
        HttpReturnKind::File { descriptor_id } => ResourceClaim::File {
            descriptor_id: *descriptor_id,
        },
        HttpReturnKind::Upload { receipt_id } => ResourceClaim::Upload {
            receipt_id: *receipt_id,
        },
    })
}

// ============================================================================
// 表驱动测试（身份/错 owner/过期/单调 id——T-02 验证门）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::virt_memory::VirtualFlash;

    fn fresh_vm() -> AutoVM {
        AutoVM::new(VirtualFlash::new(1024), 1024)
    }

    fn pop(task: &mut AutoTask) -> auto_val::NanoValue {
        task.ram.pop_nv()
    }

    /// producer push → 栈顶 TAG_OBJECT → resolve 得快照（身份链）。
    #[test]
    fn plan776_t02_response_resource_identity() {
        let vm = fresh_vm();
        let mut task = AutoTask::new(0, 1024, 0);
        push_response_resource(
            &mut task,
            &vm,
            302,
            vec![("Location".to_string(), "/moved".to_string())],
            Vec::new(),
        )
        .unwrap();
        let nv = pop(&mut task);
        assert!(auto_val::is_object(nv));
        match resolve_resource(&vm, nv).expect("凭据解析") {
            ResourceClaim::Response(s) => {
                assert_eq!(s.status, 302);
                assert_eq!(s.headers[0].1, "/moved");
            }
            other => panic!("错种类：{other:?}"),
        }
    }

    /// 错 owner：凭据 nv 属 vm2，vm1 查堆 miss → None（不误取）。
    #[test]
    fn plan776_t02_wrong_owner_rejected() {
        let vm1 = fresh_vm();
        let vm2 = fresh_vm();
        let mut task = AutoTask::new(0, 1024, 0);
        push_response_resource(&mut task, &vm2, 200, Vec::new(), b"ok".to_vec()).unwrap();
        let nv = pop(&mut task);
        assert!(
            resolve_resource(&vm1, nv).is_none(),
            "跨 VM 凭据不得命中（per-VM heap 即 owner 边界）"
        );
        assert!(resolve_resource(&vm2, nv).is_some(), "本 owner 命中");
    }

    /// 过期：构造后移除堆 entry（模拟回收）→ resolve None；同值 int 也 None。
    #[test]
    fn plan776_t02_stale_resource_rejected() {
        let vm = fresh_vm();
        let mut task = AutoTask::new(0, 1024, 0);
        push_sse_resource(&mut task, &vm, 77).unwrap();
        let nv = pop(&mut task);
        let handle = auto_val::decode_object(nv) as u64;
        assert!(matches!(
            resolve_resource(&vm, nv),
            Some(ResourceClaim::Sse { iterator_id: 77 })
        ));
        vm.heap_objects.remove(&handle);
        assert!(resolve_resource(&vm, nv).is_none(), "回收后凭据过期");
        assert!(
            resolve_resource(&vm, auto_val::encode_i32(handle as i32)).is_none(),
            "同值普通 int 恒非凭据"
        );
    }

    /// heap id 单调无复用（代次语义：不同凭据不同 id；构造计数正确）。
    #[test]
    fn plan776_t02_monotonic_ids_and_live_count() {
        let vm = fresh_vm();
        let before = live_resource_count();
        let mut task = AutoTask::new(0, 1024, 0);
        let mut handles = Vec::new();
        for i in 0..3u32 {
            push_file_resource(&mut task, &vm, 1000 + i as u64).unwrap();
            let nv = pop(&mut task);
            handles.push(auto_val::decode_object(nv) as u64);
        }
        let uniq: std::collections::HashSet<u64> = handles.iter().copied().collect();
        assert_eq!(uniq.len(), 3, "heap id 不复用");
        assert_eq!(live_resource_count(), before + 3);
        // 表移除 → 对象 Drop → 计数回落（引用归零路径由 RC 归零触发；表
        // remove 直接 drop 最后 Arc——单元级验证 Drop 钩子语义）。
        for h in handles {
            vm.heap_objects.remove(&h);
        }
        assert_eq!(live_resource_count(), before, "回收后计数回基线");
    }

    /// 非资源堆对象（RustStdlibObject）不解析为凭据（downcast 过滤）。
    #[test]
    fn plan776_t02_non_resource_object_rejected() {
        let vm = fresh_vm();
        let handle = vm.insert_heap_object(crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
            "probe.NotResource",
            1u32,
        ));
        assert!(
            resolve_resource(&vm, auto_val::encode_object(handle as u32)).is_none(),
            "普通堆对象不得当资源凭据"
        );
    }

    /// propagation 传播保持（globals + FutureValue.result 双载体）。
    #[test]
    fn plan776_t02_resource_propagates_through_value_carriers() {
        let vm = fresh_vm();
        let mut task = AutoTask::new(0, 1024, 0);
        push_upload_resource(&mut task, &vm, 4242).unwrap();
        let nv = pop(&mut task);
        vm.globals.insert("shared".to_string(), nv);
        let via_global = *vm.globals.get("shared").unwrap();
        assert!(matches!(
            resolve_resource(&vm, via_global),
            Some(ResourceClaim::Upload { receipt_id: 4242 })
        ));
        let value = auto_val::Value::VmRef(auto_val::VmRef {
            id: auto_val::decode_object(nv) as usize,
        });
        assert!(matches!(
            resolve_resource_value(&vm, &value),
            Some(ResourceClaim::Upload { receipt_id: 4242 })
        ));
    }
}
