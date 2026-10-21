//! PLAN-776 T-02：HTTP 返回资源凭据——handler 成功终值的按值身份载体。
//!
//! 决策报告（`docs/plans/reports/776-return-resource-decision.md` §2/§3，
//! R2/R5 终形修订）冻结方案 A：资源凭据 = `heap_objects` 表 entry
//! （`TypeTag::HttpReturnResource`），NanoValue 侧现成 `TAG_OBJECT`
//! （`encode_object`），Value 侧现成 `Value::VmRef`。普通 i32/i64 是
//! `TAG_I32/TAG_I64`——任何数值运算都不能制造资源身份（AC-01）。
//!
//! 凭据走**专用 id 段** `0x8000_0000 | owner(11bit) << 20 | local(20bit)`
//!（R1-F03/R2-02）：owner 编入 id 且无掩码回绕（第 2049 个 VM 起分配前
//! 受控拒绝；id 层 owner 校验先行，堆内 owner 字段二线）；local 段 2^20
//! 耗尽同样分配前拒绝（无旧引用复活）。凭据随 RC 归零回收（dying 宽限窗）。
//!
//! 各 kind 的数据 backing 不动（SSE iterator 仍在 `vm.iterators`、File 描述符
//! 仍在 `VM_FILE_RESPONSES`、Upload 收据仍在 `VM_UPLOAD_RECEIPTS`——take 单次
//! 移交合同保持）；Response 快照直接 owned 在凭据内（重复返回 = 每次 clone，
//! 不经全局表）。legacy int 面=死兼容（R1-F01/R2-05 查表臂退役后：setter 链
//! 已迁快照+owned-slot，client egress 已凭据化，`http.response()` 构造产
//! 凭据——Plan 326/346 六 wire 用例由真实生产点保持 wire 行为）。

use std::any::Any;
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
/// R1-F03：携带 owner（构造方 VM 的 instance_id）——heap id 是 per-VM 局部
/// 编号（两个 VM 都从 4M 起步，同号必然），解析侧必须校验 owner 等值，
/// 不能把"每 VM 一张表"当作来源证明。R1-F08：不 derive Clone（Clone 语义
/// 不携带份额，与 Drop 的 LIVE 计数不配平；kind 数据经 ResourceClaim clone）。
#[derive(Debug)]
pub struct HttpReturnResource {
    pub owner: u64,
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

/// R1-F03/R2-02：凭据专用 id 段——`0x8000_0000 | owner(11bit) << 20 |
/// local(20bit)`（NanoValue TAG_OBJECT 的 u32 载荷内；与通用 heap id 段
/// 4M+ 增长区间不重叠）。**owner 编入 id 且不掩码回绕**：owner 段超界
///（进程内创建第 2049 个 VM 后）→ 该 VM 构造凭据**受控拒绝**
///（`owner space exhausted`——有限表示不静默复用使旧 owner 的活跃凭据
/// 可达；R2-02 反例：7 位掩码下 A 与 A+128 同段误取）。local 段耗尽
/// 同样分配前拒绝（R1-F07——不截断复用使旧引用复活）。
const CREDENTIAL_ID_FLAG: u64 = 0x8000_0000;
const CREDENTIAL_OWNER_BITS: u64 = 11;
const CREDENTIAL_OWNER_MAX: u64 = 1 << CREDENTIAL_OWNER_BITS; // 2048
const CREDENTIAL_LOCAL_MAX: u64 = 1 << 20; // ~1M/VM

fn owner_slot(vm: &AutoVM) -> Result<u64, VMError> {
    let owner = vm.instance_id;
    if owner >= CREDENTIAL_OWNER_MAX {
        return Err(VMError::RuntimeError(
            "http return resource owner space exhausted".into(),
        ));
    }
    Ok(owner)
}

/// 凭据 id 分配（owner+local 编码）；任一段耗尽 → Err（不分配）。
fn credential_id(vm: &AutoVM) -> Result<u64, VMError> {
    let owner = owner_slot(vm)?;
    let local = vm.http_credential_gen.fetch_add(1, Ordering::Relaxed);
    if local >= CREDENTIAL_LOCAL_MAX {
        return Err(VMError::RuntimeError(
            "http return resource id space exhausted".into(),
        ));
    }
    Ok(CREDENTIAL_ID_FLAG | (owner << 20) | local)
}

/// id 的 owner 段是否匹配本 VM（id 层校验——同号跨 VM 在此拒绝；
/// 本 VM owner 超界时恒 false——该 VM 从未成功构造过凭据）。
fn id_owner_matches(vm: &AutoVM, id: u64) -> bool {
    if id & CREDENTIAL_ID_FLAG == 0 {
        return false;
    }
    match owner_slot(vm) {
        Ok(owner) => (id >> 20) & (CREDENTIAL_OWNER_MAX - 1) == owner,
        Err(_) => false,
    }
}

/// 构造 + 入堆 + RC push。凭据不走通用 heap_object_id_gen（id 段编 owner，
/// R1-F03）；入堆簿记（rc_created_total/state_mutation_seq）与
/// insert_heap_object 同款。HTTP 面 push 全部在 owner 线程（LocalSet 串行）。
fn push_kind(task: &mut AutoTask, vm: &AutoVM, kind: HttpReturnKind) -> Result<(), VMError> {
    let id = credential_id(vm)?;
    LIVE_RESOURCES.fetch_add(1, Ordering::Relaxed);
    let obj: HttpReturnResource = HttpReturnResource {
        owner: vm.instance_id,
        kind,
    };
    vm.heap_objects
        .insert(id, std::sync::Arc::new(std::sync::RwLock::new(obj)));
    vm.rc_created_total.fetch_add(1, Ordering::Relaxed);
    vm.state_mutation_seq.fetch_add(1, Ordering::Relaxed);
    LAST_HANDLE.store(id, Ordering::Relaxed);
    vm.rc_push(task, auto_val::encode_object(id as u32));
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
/// 非 TAG_OBJECT / 堆查无（已回收）/ downcast 失败（非资源对象）/**owner
/// 不等值（R1-F03：另一 VM 的同号凭据）** 一律 `None`——凭据不可凭数值
/// 命中，也不可凭 per-VM 表号命中（AC-01/AC-05）。
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

/// R1-F03：owner 等值校验的公开探针（测试用——同号凭据跨 VM 互拒 oracle）。
pub fn credential_owner(vm: &AutoVM, nv: auto_val::NanoValue) -> Option<u64> {
    if !auto_val::is_object(nv) {
        return None;
    }
    let handle = auto_val::decode_object(nv) as u64;
    if !id_owner_matches(vm, handle) {
        return None;
    }
    let obj = vm.get_heap_object(handle)?;
    let guard = obj.read().ok()?;
    let res = guard.as_any().downcast_ref::<HttpReturnResource>()?;
    Some(res.owner)
}

/// R1-F06：对凭据快照的可变访问（`http.response()` setter 链 / `res.status`
/// 等访问器的对象载荷分派）——按本 VM owner 校验后改快照，表与凭据互不
/// 串扰（构造时快照即真身，mutable 面直接改凭据）。
pub fn mutate_response_snapshot<R>(
    vm: &AutoVM,
    nv: auto_val::NanoValue,
    f: impl FnOnce(&mut ResponseSnapshot) -> R,
) -> Option<R> {
    if !auto_val::is_object(nv) {
        return None;
    }
    let handle = auto_val::decode_object(nv) as u64;
    if !id_owner_matches(vm, handle) {
        return None;
    }
    let obj = vm.get_heap_object(handle)?;
    let mut guard = obj.write().ok()?;
    let res = guard
        .as_any_mut()
        .downcast_mut::<HttpReturnResource>()
        .filter(|r| r.owner == vm.instance_id)?;
    match &mut res.kind {
        HttpReturnKind::Response(s) => Some(f(s)),
        _ => None,
    }
}

/// R1-F06：只读快照访问（访问器 `.status()/.body()/.header(k)` 的对象载荷）。
pub fn read_response_snapshot(vm: &AutoVM, nv: auto_val::NanoValue) -> Option<ResponseSnapshot> {
    match resolve_resource(vm, nv)? {
        ResourceClaim::Response(s) => Some(s),
        _ => None,
    }
}

fn read_claim(vm: &AutoVM, handle: u64) -> Option<ResourceClaim> {
    // R1-F03：id 层 owner 校验先行（owner 编入 id 段——跨 VM 同号在此
    // 拒绝，不依赖堆内字段——堆内 owner 是第二道防线）。
    if !id_owner_matches(vm, handle) {
        return None;
    }
    let obj = vm.get_heap_object(handle)?;
    let guard = obj.read().ok()?;
    let res = guard
        .as_any()
        .downcast_ref::<HttpReturnResource>()
        .filter(|r| r.type_tag() == TypeTag::HttpReturnResource)?;
    if res.owner != vm.instance_id {
        return None;
    }
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

/// R1-F04：HTTP 边界任务终结的资源凭据清偿——扫描任务 RAM 有效槽
///（0..sp），释放所有本 VM `HttpReturnResource` 凭据的栈份额。Internal
/// `~{}` body RET 后值进 result（VmRef 无份额），body 栈的构造份额与
/// handler 异常/取消路径的残留引用都在此清偿；共享全局常量的份额不在
/// 任务栈（STORE_GLOBAL 后释放过），不受影响。通用对象的栈清理是
/// P776-D1 既有边界，此处只清本计划资源。
pub fn release_task_credentials(vm: &AutoVM, task: &AutoTask) {
    // R2-04：任务栈上的**全部**本 VM 堆对象槽（不只凭据）——record/list 等
    // 容器的栈份额释放后，父对象进 dying 宽限窗，真回收时经 child_refs
    // 级联释放子引用 stakes（凭据随之归零；逃逸/shared 值在宽限窗内被
    // retain 复活——PLAN-062 语义保护）。此前只匹配顶层凭据，漏掉
    // `Box { response: ... }` 等容器持有形态（R2-04 反例：异常退出 11 残留）。
    for slot in 0..task.ram.sp {
        let nv = task.ram.read_nv(slot);
        if !auto_val::is_object(nv) {
            continue;
        }
        vm.rc_release_id(auto_val::decode_object(nv) as u64);
    }
}
