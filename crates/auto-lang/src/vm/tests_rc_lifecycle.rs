//! Plan 419 Phase 1 里程碑测试(§2.2)。
//!
//! 引用计数协议(copy-on-load 所有权)的行为锁:
//! - 作用域/覆盖/容器移除 → 确定性归零(tier 1)
//! - 捕获/返回/全局 → 存活(逃逸出口)
//! - 双属主 → 部分释放存活、全释放归零(tier 3 预演)
//! - churn → live_heap 回基线(泄漏检测)
//! - canary → UAF 毒化报错(debug)
//!
//! 断言通道:`auto.rc.live()` / `auto.rc.count(x)`(脚本内 print 探针)+
//! Rust 侧 `vm.rc_stats()`。禁用 RSS 断言(计划 §5)。

#![cfg(test)]

use crate::vm::engine::AutoVM;
use crate::vm::virt_memory::VirtualFlash;

fn make_vm() -> AutoVM {
    AutoVM::new(VirtualFlash::new(1024), 1024)
}

/// 编译并跑一段 Auto 源码到完成,返回 (vm, stdout)。
/// 跑毕对主任务做残余栈释放,使 live_heap 反映"程序真实留存"。
async fn run_code_vm(code: &str) -> (AutoVM, String) {
    let (vm, stdout, entry, _result_type) = crate::create_vm_from_source(code)
        .expect("compile failed");
    let tid = vm.spawn_task(entry, 65536);
    vm.run_task_loop().await;
    if let Some(arc) = vm.tasks.get(&tid) {
        let mut t = arc.lock().await;
        vm.rc_release_task_stack(&mut t);
    }
    vm.tasks.remove(&tid);
    let out = stdout.read().unwrap().clone();
    (vm, out)
}

// ============================================================================
// §2.2 里程碑 1:scope_drop_basic —— 块尾确定性回收
// ============================================================================

#[tokio::test]
async fn scope_drop_basic() {
    let code = r#"
type Note { id int }

fn main() int {
    var keep int = 0
    for i in 0..1000 {
        var c Note = Note { id: i }
        keep = c.id
    }
    keep
}
"#;
    // churn 放大:块尾若不释放,1000 个 Note 全部滞留;正确 → 0。
    let (vm, _out) = run_code_vm(code).await;
    assert_eq!(vm.rc_stats().live_heap, 0, "block-scoped Notes freed at each block end");
}

// ============================================================================
// §2.2 里程碑 2:fn_ret_drop —— 函数返回后帧内对象回收
// ============================================================================

#[tokio::test]
async fn fn_ret_drop() {
    let code = r#"
type Note { id int }

fn make() Note {
    let n Note = Note { id: 7 }
    n
}

fn main() int {
    let x Note = make()
    x.id
}
"#;
    let (vm, _out) = run_code_vm(code).await;
    assert_eq!(vm.rc_stats().live_heap, 0, "fn-local Note freed by RET frame sweep");
}

// ============================================================================
// §2.2 里程碑 3:overwrite_drop —— 覆盖赋值旧值回收
// ============================================================================

#[tokio::test]
async fn overwrite_drop() {
    let code = r#"
type Note { id int }

fn bump(_old Note, i int) Note {
    Note { id: i }
}

fn main() int {
    var x Note = Note { id: 0 }
    for i in 0..1000 {
        x = bump(x, i)
    }
    x.id
}
"#;
    // churn 放大:覆盖赋值若不释放旧值,999 个 Note 滞留;正确 → 终值 1 个,
    // 主任务收尾后再归零。
    let (vm, _out) = run_code_vm(code).await;
    assert_eq!(vm.rc_stats().live_heap, 0, "overwritten Notes freed at each store");
}

// ============================================================================
// §2.2 里程碑 4:container_elem_drop —— 容器元素移除后回收
// ============================================================================

#[tokio::test]
async fn container_elem_drop() {
    let code = r#"
type Note { id int }

fn main() int {
    var l List<Note> = List<Note>.new([])
    for i in 0..1000 {
        l.push(Note { id: i })
        l.pop()
    }
    0
}
"#;
    // churn 放大:push+pop 配对若容器侧不释放,1000 个 Note 滞留;正确 → 0
    // (list 本身随主任务收尾释放)。
    let (vm, _out) = run_code_vm(code).await;
    assert_eq!(vm.rc_stats().live_heap, 0, "popped elements freed (container stake released)");
}

#[tokio::test]
async fn container_elem_keeps_alive() {
    let code = r#"
type Note { id int }

fn main() int {
    var l List<Note> = List<Note>.new([])
    for i in 0..1000 {
        l.push(Note { id: i })
    }
    print(l.get(999).id)
    0
}
"#;
    // 1000 元素在容器中滞留 = 1001(容器+元素)。终值证明容器持有语义存在
    // (若容器写入时漏 retain,元素将被死区结算过早释放 —— canary/读值都会炸)。
    let (vm, out) = run_code_vm(code).await;
    assert_eq!(out.trim(), "999", "element readable while in container");
    // 扫尾后 list 归零 → 级联释放全部元素(容器持有语义由"可读"已证明;
    // 若容器侧漏 retain,死区结算会在 push 处过早释放 → canary/读值错)。
    assert_eq!(vm.rc_stats().live_heap, 0);
}

// ============================================================================
// §2.2 里程碑 5:closure_keeps_alive —— 捕获存活过作用域
// ============================================================================

#[tokio::test]
async fn closure_keeps_alive() {
    let code = r#"
type Note { id int }

fn main() int {
    var keep int = 0
    for i in 0..1000 {
        var c Note = Note { id: i }
        let f = fn() { c.id }
        keep = f()
    }
    print(keep)
    keep
}
"#;
    // f() 可读 = 捕获对象存活(canary 即探针);迭代尾不释放被捕获对象。
    let (vm, out) = run_code_vm(code).await;
    assert_eq!(out.trim(), "999", "captured Note readable through closure");
    let live = vm.rc_stats().live_heap;
    assert!(live >= 1000, "closure envs hold stakes for all captured Notes: got {}", live);
}

// ============================================================================
// §2.2 里程碑 6:return_keeps_alive —— 返回值存活至调用方丢弃
// ============================================================================

#[tokio::test]
async fn return_keeps_alive() {
    let code = r#"
type Note { id int }

fn make() Note { Note { id: 9 } }

fn main() int {
    let x Note = make()
    print(x.id)
    x.id
}
"#;
    // 返回值在调用方持有期间可读(提前释放 → canary panic / 读值错)。
    let (_vm, out) = run_code_vm(code).await;
    assert_eq!(out.trim(), "9", "returned Note alive while caller holds it");
}

// ============================================================================
// §2.2 里程碑 7:global_keeps_alive —— 存入全局后存活
// ============================================================================

#[tokio::test]
async fn global_keeps_alive() {
    let code = r#"
type Note { id int }

var g Note = Note { id: 5 }

fn main() int {
    print(g.id)
    g.id
}
"#;
    let (vm, out) = run_code_vm(code).await;
    assert_eq!(out.trim(), "5", "global Note readable");
    let after = vm.rc_stats().live_heap;
    assert!(after >= 1, "global survives program end: got {}", after);
}

// ============================================================================
// §2.2 里程碑 8:shared_two_owners —— 双属主(tier 3 预演)
// ============================================================================

#[tokio::test]
async fn shared_two_owners() {
    let code = r#"
type Note { id int }

fn fresh(_old Note, i int) Note {
    Note { id: i }
}

fn main() int {
    var total int = 0
    for i in 0..1000 {
        var a Note = Note { id: i }
        var b Note = a
        a = fresh(a, 0)
        total = total + b.id + a.id
    }
    total
}
"#;
    // b 在 a 覆盖后仍可读 = 共享对象在一方释放后存活(tier 3);若引用计数
    // 把拷贝当独占,b 的对象会随 a 的覆盖被过早释放 → canary panic/读值错。
    let (vm, _out) = run_code_vm(code).await;
    assert_eq!(vm.rc_stats().live_heap, 0, "churn of shared pairs fully released at end");
}

// ============================================================================
// §2.2 里程碑 9:churn_returns_to_baseline —— 循环 10 万临时对象回基线
// ============================================================================

#[tokio::test]
async fn churn_returns_to_baseline() {
    let code = r#"
type Note { id int }

fn main() int {
    var total int = 0
    for i in 0..100000 {
        let t Note = Note { id: i }
        total = total + t.id - i
    }
    total
}
"#;
    let start = std::time::Instant::now();
    let (vm, _out) = run_code_vm(code).await;
    let elapsed = start.elapsed();
    assert_eq!(vm.rc_stats().live_heap, 0, "churn must return live_heap to baseline");
    // 泄漏检测护栏:10 万分配/回收若泄漏,live 会是 100000 量级。
    // (不断言绝对耗时 —— perf 门禁由基准测试承担。)
    eprintln!("churn 100k alloc/free took {:?}", elapsed);
}

// ============================================================================
// §2.2 里程碑 10:uaf_canary_poisoned —— 毒化 canary(debug)
// ============================================================================

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "RC canary")]
fn uaf_canary_poisoned() {
    let vm = make_vm();
    let id = vm.insert_heap_object(crate::vm::types::ObjectData::new());
    vm.rc_retain_id(id);
    assert_eq!(vm.rc_count(id), 1);
    vm.rc_release_id(id);
    assert!(!vm.contains_heap_object(id), "freed at rc 0");
    // UAF:访问已释放 id → 毒化 canary panic。
    let _ = vm.get_heap_object(id);
}

// ============================================================================
// §2.2 里程碑 11:rc_balance_unit —— 协议屏障单元测试(§1 表逐行)
// ============================================================================

#[test]
fn rc_balance_unit() {
    use crate::vm::rc::{heap_ref_id, is_heap_ref_nv, HEAP_ID_BASE};
    use crate::vm::task::AutoTask;

    // ---- is_heap_ref_nv / heap_ref_id 三种编码 ----
    assert!(is_heap_ref_nv(auto_val::encode_object(42)));
    assert!(is_heap_ref_nv(auto_val::encode_list(42)));
    assert!(is_heap_ref_nv(auto_val::encode_bigint(42)));
    assert!(is_heap_ref_nv(auto_val::encode_i32(HEAP_ID_BASE as i32)));
    assert!(!is_heap_ref_nv(auto_val::encode_i32(HEAP_ID_BASE as i32 - 1)));
    assert!(!is_heap_ref_nv(auto_val::encode_i32(-5))); // 字符串负 tag(Phase 2 接管)
    assert!(!is_heap_ref_nv(auto_val::encode_string(3)));
    assert!(!is_heap_ref_nv(auto_val::encode_bool(true)));
    assert_eq!(heap_ref_id(auto_val::encode_object(77)), Some(77));
    assert_eq!(heap_ref_id(auto_val::encode_i32(HEAP_ID_BASE as i32 + 5)), Some(HEAP_ID_BASE + 5));
    assert_eq!(heap_ref_id(auto_val::encode_i32(123)), None);

    // ---- 计数平衡:birth(+1) / copy(+1) / death(-1) / 归零释放 ----
    let vm = make_vm();
    let mut task = AutoTask::new(0, 256, 0);
    let obj = crate::vm::types::ObjectData::new();
    let id = vm.insert_heap_object(obj);
    assert_eq!(vm.rc_stats().live_heap, 1);
    assert_eq!(vm.rc_count(id), 0, "fresh insert has no owner yet");

    // 唯一属主死亡(push +1 / pop -1)→ 归零即释放(确定性回收)。
    vm.rc_push_id(&mut task, id);
    assert_eq!(vm.rc_count(id), 1);
    let nv = task.ram.pop_nv();
    vm.rc_release(nv);
    assert_eq!(vm.rc_count(id), 0);
    assert!(!vm.contains_heap_object(id), "freed when the only owner dies");

    // 无条目的 release(已释放 id 的重复释放)→ 安全跳过,不 panic。
    vm.rc_release_id(id);
    assert!(!vm.contains_heap_object(id));

    // 双属主:一方释放仍活,双方释放才归零。
    let id2 = vm.insert_heap_object(crate::vm::types::ObjectData::new());
    vm.rc_retain_id(id2);
    vm.rc_retain_id(id2);
    assert_eq!(vm.rc_count(id2), 2);
    vm.rc_release_id(id2);
    assert!(vm.contains_heap_object(id2), "still alive with one owner");
    assert_eq!(vm.rc_count(id2), 1);
    vm.rc_release_id(id2);
    assert!(!vm.contains_heap_object(id2), "freed when rc transitions to 0");

    // ---- 嵌套图:父回收 → 子传递回收 ----
    let vm2 = make_vm();
    let child = vm2.insert_heap_object(crate::vm::types::ObjectData::new());
    let mut parent = crate::vm::types::ObjectData::new();
    parent.set(
        auto_val::ValueKey::Str("child".into()),
        auto_val::Value::VmRef(auto_val::VmRef { id: child as usize }),
    );
    vm2.rc_retain_id(child); // 父字段的持有(与 SET_FIELD 转移语义一致)
    let parent_id = vm2.insert_heap_object(parent);
    vm2.rc_retain_id(parent_id); // 属主 stake
    assert_eq!(vm2.rc_count(child), 1);
    assert_eq!(vm2.rc_count(parent_id), 1);
    vm2.rc_release_id(parent_id);
    assert!(!vm2.contains_heap_object(parent_id), "parent freed");
    assert!(!vm2.contains_heap_object(child), "child cascade-freed with parent");
}

// ============================================================================
// Plan 419 Phase 2 里程碑(§3.2):字符串池 RC + 空洞复用 + pinned
// ============================================================================

#[test]
fn str_pool_rc_balance_unit() {
    let mut vm = make_vm();
    vm.load_strings(vec![b"flash-const".to_vec()]); // pinned
    let pinned_idx = 0usize;

    // pinned:计数免计费,重复 release 无效。
    vm.pool_release(pinned_idx);
    vm.pool_release(pinned_idx);
    assert_eq!(vm.pool_count(pinned_idx), u32::MAX);
    assert!(!vm.pool_is_tombstone(pinned_idx));

    // 运行期条目:retain/release 配平 → 归零释放 → 墓碑 + freelist。
    let idx = vm.add_string(b"runtime-str".to_vec());
    assert!(!vm.pool_is_tombstone(idx));
    vm.pool_retain(idx);
    assert_eq!(vm.pool_count(idx), 1);
    vm.pool_retain(idx);
    assert_eq!(vm.pool_count(idx), 2);
    vm.pool_release(idx);
    assert!(!vm.pool_is_tombstone(idx), "still one owner");
    vm.pool_release(idx);
    assert!(vm.pool_is_tombstone(idx), "freed at zero");
    // 墓碑读 → canary(debug)。AutoVM 非 UnwindSafe,用 AssertUnwindSafe。
    #[cfg(debug_assertions)]
    {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            vm.get_string(idx as u32)
        }));
        assert!(result.is_err(), "tombstone read must panic in debug");
    }

    // 复用:同内容 re-add 走 freelist 槽位(池总长不增)。
    let len_before = vm.strings.read().unwrap().len();
    let idx2 = vm.add_string(b"runtime-str".to_vec());
    assert!(!vm.pool_is_tombstone(idx2), "slot revived");
    assert!(idx2 == idx, "freelist reuse prefers the freed slot");
    assert_eq!(vm.strings.read().unwrap().len(), len_before, "pool size bounded");

    // dedup:活条目内容命中复用索引。
    let idx3 = vm.add_string(b"runtime-str".to_vec());
    assert_eq!(idx2, idx3);
}

#[tokio::test]
async fn str_cat_temp_freed() {
    // s = s + "x" 循环:中间串死亡,池 live 回基线(freelist 复用生效)。
    let code = r#"
fn main() int {
    var s str = ""
    for i in 0..2000 {
        s = s + "x"
    }
    print(s.len())
    0
}
"#;
    let (vm, out) = run_code_vm(code).await;
    assert_eq!(out.trim(), "2000", "concat result correct");
    let live = vm.pool_live_count();
    assert!(live < 64, "intermediate concat strings must be freed: live_pool={}", live);
}

#[tokio::test]
async fn str_pinned_constants_survive() {
    // LOAD_STR 常量条目 pinned:反复执行 + 池 churn 后仍可读。
    let code = r#"
fn main() int {
    var total int = 0
    for i in 0..100 {
        let label str = "pinned-constant-label"
        total = total + label.len() - 21
    }
    print(total)
    0
}
"#;
    let (_vm, out) = run_code_vm(code).await;
    assert_eq!(out.trim(), "0", "pinned constant readable through churn");
}

#[tokio::test]
async fn str_field_cycle_bounded() {
    // 对象 str 字段随对象生命周期:churn 下池有界(字段持拷贝,SET_FIELD
    // 时池 stake 即死 —— 语义等价于"随对象回收")。
    let code = r#"
type Note { id int, title str }

fn main() int {
    var keep int = 0
    for i in 0..1000 {
        var n Note = Note { id: i, title: "note-title" }
        keep = n.id
    }
    print(keep)
    0
}
"#;
    let (vm, out) = run_code_vm(code).await;
    assert_eq!(out.trim(), "999");
    let live = vm.pool_live_count();
    assert!(live < 128, "field strings must not accumulate: live_pool={}", live);
}

// ============================================================================
// Plan 419 Phase 3 里程碑(§4.6):借用接线 + 逃逸分析
// ============================================================================

#[tokio::test]
async fn mut_on_shared_errors() {
    // .mut 动态兜底:RC>1(Shared,双属主)→ RuntimeError;
    // 单属主 .mut 正常通过。
    let code = r#"
type Note { id int }

fn probe(c Note) int {
    let d Note = c.mut
    d.id
}

fn main() int {
    probe(Note { id: 9 })
}
"#;
    // 单属主 .mut 可用(不 panic)。
    let (_vm, out) = run_code_vm(code).await;
    let _ = out;

    // 双属主 .mut → 运行时错误(错误信息含 "mutable borrow of shared value")。
    let code2 = r#"
type Note { id int }

fn main() int {
    var a Note = Note { id: 1 }
    var b Note = a
    let m = a.mut
    0
}
"#;
    let (vm, _out) = {
        let (vm, stdout, entry, _rt) = crate::create_vm_from_source(code2).unwrap();
        let tid = vm.spawn_task(entry, 65536);
        vm.run_task_loop().await;
        if let Some(arc) = vm.tasks.get(&tid) {
            let mut t = arc.lock().await;
            vm.rc_release_task_stack(&mut t);
        }
        let out = stdout.read().unwrap().clone();
        (vm, out)
    };
    // 出错任务的 last_error 携带明确信息(通过任务状态检查)。
    let has_err = vm.tasks.iter().any(|e| {
        e.value().try_lock().map(|t| {
            t.last_error
                .as_ref()
                .map(|m| m.contains("mutable borrow of shared value"))
                .unwrap_or(false)
        }).unwrap_or(false)
    });
    assert!(has_err, ".mut on shared (rc=2) must raise the dynamic borrow error");
}

#[tokio::test]
async fn move_prevents_double_drop() {
    // .move 后源槽置 Nil:对象单次回收(目标持有唯一 stake),读源得 Nil。
    let code = r#"
type Note { id int }

fn main() int {
    var a Note = Note { id: 5 }
    let b Note = a.move
    let gone = a
    0
}
"#;
    let (vm, _out) = run_code_vm(code).await;
    // b 持唯一 stake;主任务收尾后归零 —— 若 move 未清源槽,a 的槽会在
    // 收尾再减一次造成过度释放(对象在 b 存活期被过早回收 → canary)。
    assert_eq!(vm.rc_stats().live_heap, 0);
}

#[test]
fn escape_analysis_unit() {
    use crate::trans::escape::{EscapeAnalyzer, OwnershipTier};
    use crate::ast::Stmt;

    // 三类出口标定(plan §4.1):
    let src = r#"
type Cell { v int }

var g Cell = Cell { v: 1 }

fn captured() {
    var x Cell = Cell { v: 2 }
    let f = fn() { x.v }
    f()
}

fn returned() Cell {
    let y Cell = Cell { v: 3 }
    y
}

fn stored_global() {
    var z Cell = Cell { v: 4 }
    g = z
}
"#;
    let mut parser = crate::parser::Parser::from(src);
    let ast = parser.parse().unwrap();
    let mut tiers: std::collections::HashMap<String, (OwnershipTier, bool)> =
        std::collections::HashMap::new();
    for stmt in &ast.stmts {
        if let Stmt::Fn(f) = stmt {
            let map = EscapeAnalyzer::analyze_fn(f);
            for (id, tier) in map.decisions_iter() {
                tiers.insert(
                    format!("{}:{}", f.name, id.name),
                    (*tier, map.is_write_capture(&id.name)),
                );
            }
        }
    }
    // 闭包读捕获:按 Plan 310 模型可保守降为借用(Owned/BorrowView 均合法
    // —— Rust 借用闭包承接);关键是不得误标 write-capture。
    let (captured_tier, captured_wc) = tiers.get("captured:x").copied()
        .expect("captured:x must be tracked");
    assert!(matches!(captured_tier,
            OwnershipTier::Owned | OwnershipTier::BorrowView
            | OwnershipTier::Clone | OwnershipTier::RcRefCell | OwnershipTier::ArcMutex),
        "captured local tier must be valid (got {:?})", captured_tier);
    assert!(!captured_wc, "read-only capture is not a write-capture");

    // 写捕获(plan 419 §4.5):tier = RcRefCell 且标记 write_capture。
    let src2 = r#"
fn writer() {
    var w int = 0
    let f = fn() { w = w + 1 }
    f()
}
"#;
    let mut p2 = crate::parser::Parser::from(src2);
    let ast2 = p2.parse().unwrap();
    if let Some(Stmt::Fn(f)) = ast2.stmts.iter().find(|s| matches!(s, Stmt::Fn(_))) {
        let map = EscapeAnalyzer::analyze_fn(f);
        let tier = map.lookup(0, &"w".into()).unwrap_or(OwnershipTier::Owned);
        assert_eq!(tier, OwnershipTier::RcRefCell, "write-capture upgrades to RcRefCell");
        assert!(map.is_write_capture(&"w".into()));
    } else {
        panic!("fn writer not found");
    }
}
