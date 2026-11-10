//! PLAN-1000 R02:avm — legacy VM 分配故障注入 seam 的源级回归锁。
//!
//! R02(scn) 行的必需观察在 VM 侧的三条引擎级事实（check 面只覆盖逐点
//! 注入与正控，见 resource-915 套件的 autovm-fault-* 检查）：
//! - **旧值/源保持**：故障点之前已提交的对象内容完好（按 id 读回验证）；
//! - **已构造部分回收**：故障中止后任务栈释放 + reap，失败运行的堆对象
//!   全部回收（无泄漏存活）；
//! - **错误分类不伪装语言错误**：AllocFault 经 classify_staged 判为
//!   run 阶段 typed 资源错误（code=AllocFault），绝不是 parse/type。
//!
//! seam 本身（`set_next_run_alloc_fault_at`，默认 0=关，one-shot 由下一
//! 个构造的引擎消费）在生产形态下逐字节不变——见 engine.rs 的字段注释。

#![cfg(test)]

use crate::test_cases::{classify_staged, StageOutcome};
use crate::vm::engine::AutoVM;

/// The one-shot knob is process-global — serialize every test that arms it
/// (cargo test runs tests on parallel threads).
static SEAM_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// L04 形态（已提交列表 + 后续分配）——故障点 2 = 第二个堆对象出生。
const COMMIT_THEN_GROW: &str = "fn main() int {\n    var xs = List<int>.new()\n    xs.push(20)\n    var ys = List<int>.new()\n    ys.push(22)\n    return xs.get(0) + ys.get(0)\n}\n";

/// 编译并运行到结束，任务**保留**（栈未释放）——调用者先观察错误后的
/// 存活堆，再显式 teardown 观察回收。注入由 one-shot 旋钮在引擎构造时
/// 消费。
async fn run_vm_keep_task(code: &str, fault_at: u64) -> (AutoVM, u64, Option<String>) {
    if fault_at != 0 {
        crate::vm::engine::set_next_run_alloc_fault_at(fault_at);
    }
    let (vm, _stdout, entry, _rt) = crate::create_vm_from_source(code).expect("compile failed");
    let tid = vm.spawn_task(entry, 65536);
    vm.run_task_loop().await;
    let mut err_code = None;
    if let Some(arc) = vm.tasks.get(&tid) {
        let t = arc.lock().await;
        err_code = t.last_error_code.map(|c| c.to_string());
    }
    (vm, tid, err_code)
}

/// 任务栈释放 + 任务移除 + reap（run_code_vm 收尾三件套）。
async fn teardown(vm: &AutoVM, tid: u64) {
    if let Some(arc) = vm.tasks.get(&tid) {
        let mut t = arc.lock().await;
        vm.rc_release_task_stack(&mut t);
    }
    vm.remove_task(tid);
    vm.reap_all();
}

#[tokio::test]
async fn old_value_preserved_and_partial_reclaim() {
    let _seam = SEAM_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // 无注入正控：完整运行，收尾后堆清空。
    let (vm, tid, err) = run_vm_keep_task(COMMIT_THEN_GROW, 0).await;
    assert!(err.is_none(), "positive control must complete (got {err:?})");
    teardown(&vm, tid).await;
    assert!(
        vm.heap_objects.is_empty(),
        "normal run reclaims everything after teardown"
    );
    assert_eq!(vm.alloc_ordinal.load(std::sync::atomic::Ordering::Relaxed), 2);
    drop(vm);

    // 故障点 2：第二个列表出生即中止——typed AllocFault。
    let (vm, tid, err) = run_vm_keep_task(COMMIT_THEN_GROW, 2).await;
    assert_eq!(err.as_deref(), Some("AllocFault"));
    // 旧值保持：故障中止后（teardown 之前），故障前已提交的第一个列表
    // 仍在堆上且内容完好（20）——中止对已提交状态只读。故障点的第二个
    // 列表已出生（步界检查：分配完成后的下一条指令前中止）但其 push
    // 从未执行——未提交值 22 不存在于任何对象。
    let survivors: Vec<(u64, std::sync::Arc<std::sync::RwLock<dyn crate::vm::heap_object::HeapObject>>)> = vm
        .heap_objects
        .iter()
        .map(|e| (e.key().clone(), e.value().clone()))
        .collect();
    assert_eq!(survivors.len(), 2, "committed list + aborted growth's empty list");
    let mut committed_seen = 0;
    for (_, obj) in &survivors {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<crate::vm::types::ListData<i32>>() {
            if list.elems == vec![20] {
                committed_seen += 1;
            }
            assert!(
                !list.elems.contains(&22),
                "the failed push never committed its value"
            );
        }
    }
    assert_eq!(committed_seen, 1, "exactly the pre-fault committed list [20] survives");
    // 已构造部分回收：失败运行的栈释放 + reap 后堆清零——无泄漏存活。
    teardown(&vm, tid).await;
    assert!(
        vm.heap_objects.is_empty(),
        "failed run's partially-constructed objects are all reclaimed"
    );
}

#[test]
fn alloc_fault_classifies_as_typed_run_resource_error() {
    let _seam = SEAM_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // 错误分类不伪装语言错误：typed code = AllocFault，stage = run。
    crate::vm::engine::set_next_run_alloc_fault_at(1);
    let staged = crate::run_with_capture_and_path_staged(COMMIT_THEN_GROW, "main.at")
        .expect_err("fault at the first allocation must abort the run");
    crate::vm::engine::clear_next_run_alloc_fault_at();
    assert_eq!(staged.run_trap_code, Some("AllocFault"));
    match classify_staged(&staged.error, staged.run_trap_code) {
        StageOutcome::Language(f) => {
            assert_eq!(f.stage.as_str(), "run");
            assert_eq!(f.code.as_deref(), Some("AllocFault"));
        }
        other => panic!("injected resource fault must be a language-run negative, got {other:?}"),
    }
}
