//! PLAN-711 T-11: CPU 可续跑执行片单测（engine slice 驱动 + bridge 有界泵/写队列）。
//!
//! 覆盖（对 711 契约 §6 单元/族测试的 T-11 首批）：
//! - AC-13（核心缺陷）：预算耗尽 slice 档返回 `Runnable`（栈完整可续跑），
//!   **不再**落入 `Completed(Ok(()))` 静默假成功；legacy 档耗尽返回真错误
//!   （`Result` 契约形状不变）。
//! - AC-10（片等价性前置）：长 handler 逐片续跑终态与同步参考逐字节一致；
//!   跨片累计指令数单调累加（不让出重置护栏）。
//! - 累计护栏：跨片累计预算耗尽 → 真错误（可续跑不等于无限跑）。
//! - D-2 泵纪律（bridge 级）：CpuRunnable 对 tick 泵不可见（16ms tick 不
//!   拾取 CPU continuation）；有界泵 FIFO 快照驱动；排队写事件片间消费；
//!   队满可观察 busy；可覆盖输入合并。

use crate::vm::engine::{CpuSliceBudget, ParkedSegment, SegmentOutcome};
use crate::vm::task::AutoTask;

/// 测试预算：步数封顶决定论（墙钟放开），累计预算按用例给。
fn test_budget(max_steps: u64, cumulative_steps: u64) -> CpuSliceBudget {
    CpuSliceBudget {
        max_steps,
        max_duration: std::time::Duration::from_secs(3600),
        clock_check_every: 64,
        cumulative_steps,
    }
}

/// AC-13 核心（slice 档）+ AC-10 等价性前置：长 handler 逐片续跑到终态，
/// 结果与同步参考一致；每次让出栈保持 mid-frame（bp≠saved_bp），跨片累计
/// 指令数单调累加（不让出重置护栏绕过 runaway 保护）。
#[test]
fn plan711_slice_exhaustion_returns_runnable_and_resumes_to_sync_parity() {
    let code = r#"
fn churn_total() int {
    var acc = 0
    var i = 0
    while i < 20000 { acc = acc + 1; i = i + 1 }
    acc
}
"#;
    let (vm, _stdout, _entry, _ot) = crate::create_vm_from_source(code).expect("compile");

    // 同步参考：legacy 同步驱动的终态结果。
    let mut ref_task = AutoTask::new(0, 65536, 0);
    vm.call_fn_by_name(&mut ref_task, "churn_total", 0)
        .expect("sync reference completes");
    let ref_nv = ref_task.ram.pop_nv();
    assert!(
        auto_val::is_i32(ref_nv),
        "sync reference result should be i32"
    );
    let ref_value = auto_val::decode_i32(ref_nv);
    assert_eq!(ref_value, 20000, "sync reference sanity");

    // slice 驱动：512 步一片，20000 次迭代必然多次让出。
    let budget = test_budget(512, 10_000_000);
    let mut task = AutoTask::new(0, 65536, 0);
    let mut seg_opt: Option<ParkedSegment> = None;
    let mut yields = 0u32;
    let mut prev_cumulative: u64 = 0;
    let final_res = loop {
        let outcome = match seg_opt.take() {
            None => vm.call_fn_by_name_cpu_slice(&mut task, "churn_total", 0, budget),
            Some(seg) => {
                // 让出重入前：栈保持 mid-frame（AC-13 语义——不是 Completed）。
                assert_ne!(
                    task.bp, seg.saved_bp,
                    "mid-slice task must keep the handler frame (bp != saved_bp)"
                );
                vm.resume_fn_by_name_cpu_slice(&mut task, &seg, budget)
            }
        };
        match outcome {
            SegmentOutcome::Runnable { seg } => {
                yields += 1;
                assert!(yields < 10_000, "runaway slice loop — pump not progressing");
                // 跨片累计单调：本片结束后累计值严格大于上一片（护栏不重置）。
                let cum = task.cpu_steps_total;
                assert!(
                    cum > prev_cumulative,
                    "cumulative step budget must accumulate across slices ({prev_cumulative} -> {cum})"
                );
                prev_cumulative = cum;
                seg_opt = Some(seg);
            }
            SegmentOutcome::Parked { .. } => {
                panic!("pure-CPU handler must never park on I/O")
            }
            SegmentOutcome::Completed(res) => {
                assert!(
                    yields >= 2,
                    "20000-iteration handler must yield at least twice, got {yields}"
                );
                break res;
            }
        }
    };
    final_res.expect("slice-driven handler completes Ok");
    let nv = task.ram.pop_nv();
    assert!(
        auto_val::is_i32(nv),
        "slice-driven result should be i32 like the sync driver"
    );
    assert_eq!(
        auto_val::decode_i32(nv),
        ref_value,
        "slice-driven result must equal the sync reference (AC-10 equivalence)"
    );
    assert!(task.cpu_steps_total > 0, "total steps recorded");
}

/// AC-13（legacy 档）：legacy 同步预算耗尽返回**真错误**——不再落入
/// `Completed(Ok(()))` 静默假成功（旧 :2748 缺陷臂）。`Result` 契约形状
/// 不变（错误也是契约内结果）。
#[test]
fn plan711_legacy_budget_exhaustion_returns_real_error_not_fake_success() {
    let code = r#"
fn spin() int {
    var i = 0
    while i >= 0 { i = i + 1 }
    i
}
"#;
    let (vm, _stdout, _entry, _ot) = crate::create_vm_from_source(code).expect("compile");
    let mut task = AutoTask::new(0, 65536, 0);
    let res = vm.call_fn_by_name(&mut task, "spin", 0);
    let err = res.expect_err("legacy budget exhaustion must be an error (fake-success fix, AC-13)");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("budget exhausted"),
        "error should name the budget exhaustion, got: {msg}"
    );
}

/// 累计护栏（D-2）：跨片累计预算耗尽 → 真错误。可续跑 ≠ 无限跑——逐片
/// 让出不得绕过原 runaway/预算保护。
#[test]
fn plan711_cumulative_guardrail_halts_runaway_across_slices() {
    let code = r#"
fn spin() int {
    var i = 0
    while i >= 0 { i = i + 1 }
    i
}
"#;
    let (vm, _stdout, _entry, _ot) = crate::create_vm_from_source(code).expect("compile");
    // 512 步一片、累计 4096 → 约 8 片后累计护栏必须以错误收口。
    let budget = test_budget(512, 4096);
    let mut task = AutoTask::new(0, 65536, 0);
    let mut seg_opt: Option<ParkedSegment> = None;
    let mut drives = 0u32;
    let final_res = loop {
        let outcome = match seg_opt.take() {
            None => vm.call_fn_by_name_cpu_slice(&mut task, "spin", 0, budget),
            Some(seg) => vm.resume_fn_by_name_cpu_slice(&mut task, &seg, budget),
        };
        drives += 1;
        assert!(drives < 1000, "cumulative guardrail never fired");
        match outcome {
            SegmentOutcome::Runnable { seg } => seg_opt = Some(seg),
            SegmentOutcome::Parked { .. } => panic!("pure-CPU handler must never park"),
            SegmentOutcome::Completed(res) => break res,
        }
    };
    let err = final_res.expect_err("cumulative budget must halt the runaway handler");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("cumulative CPU budget exhausted"),
        "error should name the cumulative guardrail, got: {msg}"
    );
    assert!(
        task.cpu_steps_total >= 4096,
        "guardrail fires only after the cumulative budget: {}",
        task.cpu_steps_total
    );
}

// ==========================================================================
// bridge 级：CpuRunnable 凭据 / 有界泵 / 写队列（D-2 泵纪律）
// =========================================================================

/// 最小长任务 widget：`.Churn` 跑 20000 次迭代写 state（n++），足以在测试
/// 预算下多次让出；`.SetN` 独立 state 字段（m）供排队写事件副作用观察
/// ——与 Churn 的 n 递增计数正交，落点断言与片间交错序无关。
const PUMP_WIDGET: &str = r#"
widget Pump {
  model {
    n int = 0
    m int = 0
  }
  on {
    .Churn -> {
      var i = 0
      while i < 20000 { n = n + 1; i = i + 1 }
    }
    .SetN(v) -> { m = v }
  }
  view {
    text "pump" {}
  }
}
"#;

#[cfg(feature = "ui")]
fn pump_bridge() -> crate::ui::vm_bridge::VmBridge {
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::Parser::from(PUMP_WIDGET).with_session(session);
    let ast = parser.parse().expect("parse widget source");
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
            let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
            return crate::ui::vm_bridge::VmBridge::new(&widget).expect("bridge");
        }
    }
    panic!("no widget decl in source");
}

/// D-2 泵纪律：slice 派发让出后凭据 CpuRunnable——tick 泵
/// （`resume_ready_parked`）**不拾取**（16ms tick 不驱动 CPU continuation），
/// 有界泵（`resume_cpu_slices`）消费至终态，state 副作用与同步参考一致。
#[cfg(feature = "ui")]
#[test]
fn plan711_cpu_runnable_invisible_to_tick_pump_progresses_via_cpu_pump() {
    let bridge = pump_bridge();
    let budget = test_budget(512, 10_000_000);
    let state_id = bridge.state_obj_id();

    // slice 派发：长 handler 让出即返回 Ok（parked，UI 立即恢复）。
    bridge
        .call_handler_for_cpu_slice("Pump", "Churn", state_id, &[], budget)
        .expect("slice dispatch");
    assert_eq!(
        bridge.cpu_continuation_count(),
        1,
        "one live CPU continuation after slice dispatch"
    );

    // tick 泵不拾取：无 I/O 就绪 → 零消费，continuation 原样存活。
    let tick_report = bridge.resume_ready_parked();
    assert_eq!(
        tick_report.completed, 0,
        "tick pump must not drive CPU continuations (D-2)"
    );
    assert!(tick_report.failed.is_empty());
    assert_eq!(bridge.cpu_continuation_count(), 1, "still parked");

    // 有界泵消费至终态：每轮至多 512 步的一片，多轮推进到完成。
    let mut rounds = 0;
    let mut total_completed = 0;
    while bridge.has_cpu_continuations() {
        rounds += 1;
        assert!(rounds < 10_000, "cpu pump not progressing");
        let report = bridge.resume_cpu_slices(budget);
        total_completed += report.completed;
    }
    assert_eq!(
        total_completed, 1,
        "the continuation must terminate exactly once"
    );
    // 副作用与同步参考一致：n == 20000（等价性的 bridge 级证据）。
    let n = bridge.read_state("n").expect("read state");
    assert!(
        matches!(n, auto_val::Value::Int(20000)),
        "handler side effect must land in full, got {n:?}"
    );
}

/// D-2 写队列：可覆盖输入同键合并（新值覆盖旧值、不增队深）；满队列
/// 拒绝并给可观察 busy（`WriteQueueFull`，上限 128），不静默丢动作。
#[cfg(feature = "ui")]
#[test]
fn plan711_write_queue_merges_overwritable_and_rejects_when_full() {
    use crate::ui::vm_bridge::{QueuedVmWrite, VmBridgeError};
    let bridge = pump_bridge();
    let state_id = bridge.state_obj_id();

    // 同键可覆盖输入合并到一条（滚动位置/文本草稿形态）。
    for i in 0..50 {
        bridge
            .enqueue_vm_write(QueuedVmWrite {
                widget_name: "Pump".into(),
                event_name: "OnScroll".into(),
                state_obj_id: state_id,
                args: vec![auto_val::Value::Int(i)],
                overwritable: true,
            })
            .expect("mergeable enqueue");
    }
    assert_eq!(
        bridge.vm_write_queue_len(),
        1,
        "same-key overwritable inputs must merge into one queue slot"
    );

    // 非合并事件逐条占位；到 128 上限后第 129 条拒绝。
    for i in 0..127 {
        bridge
            .enqueue_vm_write(QueuedVmWrite {
                widget_name: "Pump".into(),
                event_name: format!("Click{i}"),
                state_obj_id: state_id,
                args: vec![],
                overwritable: false,
            })
            .expect("distinct enqueue within cap");
    }
    assert_eq!(bridge.vm_write_queue_len(), 128, "cap reached");
    let err = bridge
        .enqueue_vm_write(QueuedVmWrite {
            widget_name: "Pump".into(),
            event_name: "Overflow".into(),
            state_obj_id: state_id,
            args: vec![],
            overwritable: false,
        })
        .expect_err("129th distinct event must be rejected");
    assert!(
        matches!(err, VmBridgeError::WriteQueueFull { len: 128 }),
        "full queue error must be observable: {err:?}"
    );
}

/// D-2 片间消费：continuation 存活期间排队的写事件由有界泵在片间串行
/// 落地（不与在途 continuation 交错；终态后队列清空）；两个副作用都
/// 完整落地（Churn 的 20000 次递增 + 排队 SetN 的 m=7）。
#[cfg(feature = "ui")]
#[test]
fn plan711_queued_write_drains_between_slices() {
    use crate::ui::vm_bridge::QueuedVmWrite;
    let bridge = pump_bridge();
    let budget = test_budget(512, 10_000_000);
    let state_id = bridge.state_obj_id();

    bridge
        .call_handler_for_cpu_slice("Pump", "Churn", state_id, &[], budget)
        .expect("slice dispatch");
    assert!(bridge.has_cpu_continuations());

    // continuation 存活期入队一条独立副作用写（m 字段，与 n 计数正交）。
    bridge
        .enqueue_vm_write(QueuedVmWrite {
            widget_name: "Pump".into(),
            event_name: "SetN".into(),
            state_obj_id: state_id,
            args: vec![auto_val::Value::Int(7)],
            overwritable: false,
        })
        .expect("enqueue write");

    let mut saw_drain = false;
    let mut rounds = 0;
    while bridge.has_cpu_continuations() || bridge.vm_write_queue_len() > 0 {
        rounds += 1;
        assert!(rounds < 10_000, "pump not progressing");
        let report = bridge.resume_cpu_slices(budget);
        if report.queue_drained > 0 {
            saw_drain = true;
        }
    }
    assert!(saw_drain, "queued write must be drained by the cpu pump");
    assert_eq!(bridge.vm_write_queue_len(), 0, "queue empty after drain");
    let n = bridge.read_state("n").expect("read state");
    let m = bridge.read_state("m").expect("read state");
    assert!(
        matches!(n, auto_val::Value::Int(20000)),
        "continuation side effect lands in full, got {n:?}"
    );
    assert!(
        matches!(m, auto_val::Value::Int(7)),
        "queued write side effect lands, got {m:?}"
    );
}

/// D-2 公平性：一个 continuation 与一条排队写事件同轮都被驱动（无饥饿、
/// 不因排队写独占轮次），本轮让出的 continuation 不被同轮重拾（快照纪律
/// ——completed==0 证明单轮 512 步推不完 20000 迭代）。
#[cfg(feature = "ui")]
#[test]
fn plan711_cpu_pump_drives_continuation_and_queued_write_same_round() {
    use crate::ui::vm_bridge::QueuedVmWrite;
    let bridge = pump_bridge();
    let budget = test_budget(512, 10_000_000);
    let state_id = bridge.state_obj_id();

    bridge
        .call_handler_for_cpu_slice("Pump", "Churn", state_id, &[], budget)
        .expect("slice dispatch");
    bridge
        .enqueue_vm_write(QueuedVmWrite {
            widget_name: "Pump".into(),
            event_name: "SetN".into(),
            state_obj_id: state_id,
            args: vec![auto_val::Value::Int(1)],
            overwritable: false,
        })
        .expect("enqueue");

    let report = bridge.resume_cpu_slices(budget);
    assert!(
        report.slices_run >= 2,
        "one round must drive the continuation slice AND the queued write (fairness), got {}",
        report.slices_run
    );
    assert_eq!(
        report.completed, 0,
        "512-step budget cannot finish a 20000-iteration handler in one slice"
    );
    assert_eq!(
        bridge.cpu_continuation_count(),
        1,
        "continuation still alive (yielded, not re-run this round)"
    );
    assert_eq!(
        bridge.vm_write_queue_len(),
        0,
        "queued write consumed this round"
    );
}

/// T-04 (D-2): tick 泵订阅门收窄——CpuRunnable-only 注册表不吊 tick
/// （`has_parked_io_tasks` 不计 CPU continuation）；I/O 凭据在册时门开。
#[cfg(feature = "ui")]
#[test]
fn plan711_io_gate_ignores_cpu_runnable_registry() {
    let bridge = pump_bridge();
    let budget = test_budget(512, 10_000_000);
    let state_id = bridge.state_obj_id();

    assert!(
        !bridge.has_parked_io_tasks(),
        "empty registry: io gate closed"
    );

    // CPU continuation 入册：io 门仍关（tick 不该被吊起）。
    bridge
        .call_handler_for_cpu_slice("Pump", "Churn", state_id, &[], budget)
        .expect("slice dispatch");
    assert!(bridge.has_cpu_continuations());
    assert!(
        !bridge.has_parked_io_tasks(),
        "CpuRunnable-only registry must not open the io tick gate"
    );

    // CPU continuation 推进到终态：注册表清空，门仍关。
    let mut rounds = 0;
    while bridge.has_cpu_continuations() {
        rounds += 1;
        assert!(rounds < 10_000, "pump not progressing");
        bridge.resume_cpu_slices(budget);
    }
    assert!(!bridge.has_parked_io_tasks());
}
