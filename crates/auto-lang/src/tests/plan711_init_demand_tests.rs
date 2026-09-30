//! PLAN-711 T-03: Init demand 登记簿与代际生命周期单测。
//!
//! 覆盖（对 711 契约 §6 demand/代际族）：
//! - AC-04：同代际重复登记（显示/MCP 双 build）只入队一次，判定≠写身份；
//! - 代际取消：身份变化（A→B→A）对旧代际一次取消（在途段清栈、排队丢弃、
//!   前缀副作用不回滚）；
//! - 依赖序：页 demand 在途（CPU 片 park）时后继 child demand 不派发；
//! - Missing 静默 / Failed 记账不无限重试；
//! - 渲染路径集成：组件调用（fire_child_init_if_any）只登记不派发，泵
//!   驱动完成后 state 副作用落地。

use crate::ui::vm_bridge::{InitDemandDecision, InitDemandPhase};
use crate::vm::engine::CpuSliceBudget;

fn test_budget(max_steps: u64, cumulative_steps: u64) -> CpuSliceBudget {
    CpuSliceBudget {
        max_steps,
        max_duration: std::time::Duration::from_secs(3600),
        clock_check_every: 64,
        cumulative_steps,
    }
}

/// 短 Init widget：Init 即完成（同步段内）。
const FLOW_WIDGET: &str = r#"
widget Flow {
  model {
    inited bool = false
    m int = 0
  }
  on {
    .Init -> { inited = true }
    .SetM(v) -> { m = v }
  }
  view {
    text "flow" {}
  }
}
"#;

/// 长 Init widget：20000 次迭代——小预算下必然 park 成 CpuRunnable。
const SLOW_WIDGET: &str = r#"
widget SlowInit {
  model {
    n int = 0
  }
  on {
    .Init -> {
      var i = 0
      while i < 20000 { n = n + 1; i = i + 1 }
    }
  }
  view {
    text "slow" {}
  }
}
"#;

/// 纯计算 Init child（不触 state 字段——单 VM 统一根态架构下，桥级直登
/// 记的 child demand 携带根 state id；state 写入型 child 的真实 state id
/// 由渲染路径 prepare_child_render_state 提供，集成面归渲染家族回归）。
const TICK_WIDGET: &str = r#"
widget Tick {
  model {
    z int = 0
  }
  on {
    .Init -> {
      var i = 0
      while i < 10 { i = i + 1 }
    }
  }
  view {
    text "t" {}
  }
}
"#;

/// 从多 widget 源中提取指定 name 的 AuraWidget。
#[cfg(feature = "ui")]
fn extract_widget(source: &str, name: &str) -> crate::aura::AuraWidget {
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::Parser::from(source).with_session(session);
    let ast = parser.parse().expect("parse widget source");
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
            if decl.name.to_string() == name {
                return crate::aura::extract_widget_from_decl(decl).expect("extract widget");
            }
        }
    }
    panic!("widget {name} not found in source");
}

#[cfg(feature = "ui")]
fn bridge_from(source: &str, root: &str) -> crate::ui::vm_bridge::VmBridge {
    let widget = extract_widget(source, root);
    crate::ui::vm_bridge::VmBridge::new(&widget).expect("bridge")
}

/// 双 widget 桥（root=SlowInit 长 Init + child=Flow 短 Init，child 的
/// handler 一并合成进导出表）——依赖序与渲染集成测试用。
#[cfg(feature = "ui")]
fn two_widget_bridge(
    root_src: &str,
    root_name: &str,
    child_src: &str,
    child_name: &str,
) -> crate::ui::vm_bridge::VmBridge {
    let root = extract_widget(root_src, root_name);
    let child = extract_widget(child_src, child_name);
    crate::ui::vm_bridge::VmBridge::new_with_children(
        &root,
        &[child],
        Vec::new(),
        &std::collections::HashMap::new(),
        false,
    )
    .expect("bridge with children")
}

/// AC-04：同代际重复登记只入队一次；派发完成后同身份登记不再重派
/// （Init 挂载语义每代际一次，PLAN-536 保持）。
#[cfg(feature = "ui")]
#[test]
fn plan711_init_demand_registers_once_per_generation() {
    let bridge = bridge_from(FLOW_WIDGET, "Flow");
    let state_id = bridge.state_obj_id();
    let budget = test_budget(4096, 10_000_000);

    let d1 = bridge.register_init_demand("Flow", "Flow#k1", state_id, Vec::new());
    assert_eq!(d1, InitDemandDecision::Queued, "first registration queues");
    // 重复 build（显示/MCP 双路径）：只确认簿记，不二次入队。
    let d2 = bridge.register_init_demand("Flow", "Flow#k1", state_id, Vec::new());
    assert_eq!(d2, InitDemandDecision::InFlightKnown, "queued duplicate");
    assert_eq!(bridge.pending_init_demand_count(), 1, "single queue entry");

    // 派发：短 Init 同步完成；相位 Done；副作用落地。
    let report = bridge.dispatch_pending_inits(budget);
    assert_eq!(report.completed, 1, "short init completes inline");
    assert_eq!(bridge.init_demand_phase("Flow"), Some(InitDemandPhase::Done));
    let inited = bridge.read_state("inited").expect("read state");
    assert!(matches!(inited, auto_val::Value::Bool(true)), "Init ran");

    // 完成后同代际登记：不重派（Done 终态）。
    let d3 = bridge.register_init_demand("Flow", "Flow#k1", state_id, Vec::new());
    assert_eq!(d3, InitDemandDecision::AlreadyKnown, "done is terminal");
    assert_eq!(bridge.pending_init_demand_count(), 0);
}

/// 代际取消：在途（CPU park） demand 遇身份变化 → 旧段一次取消（出册/
/// 清栈），新代际入队；A→B→A 的第二个 A 重跑 Init。
#[cfg(feature = "ui")]
#[test]
fn plan711_init_demand_identity_change_supersedes_and_cancels() {
    let bridge = bridge_from(SLOW_WIDGET, "SlowInit");
    let state_id = bridge.state_obj_id();
    let budget = test_budget(512, 10_000_000);

    // A 代际登记 + 派发 → CPU 片耗尽 park 在途。
    let d = bridge.register_init_demand("SlowInit", "Slow", state_id, Vec::new());
    assert_eq!(d, InitDemandDecision::Queued);
    let report = bridge.dispatch_pending_inits(budget);
    assert_eq!(report.in_flight, 1, "long init parks as CpuRunnable");
    assert_eq!(
        bridge.init_demand_phase("SlowInit"),
        Some(InitDemandPhase::InFlight)
    );

    // B 代际登记：旧在途段一次取消，新 demand 入队。
    let d = bridge.register_init_demand("SlowInit", "Slow#v2", state_id, Vec::new());
    assert_eq!(d, InitDemandDecision::Queued);
    assert_eq!(
        bridge.cpu_continuation_count(),
        0,
        "superseded in-flight segment must be cancelled (stack released)"
    );
    assert_eq!(bridge.pending_init_demand_count(), 1, "only the new demand");

    // A→B→A：第二个 A 是新代际（身份 B≠A）——B 的排队 demand 被丢弃，
    // A 重新入队；派发 + 泵推进到终态后副作用完整（重跑而非复活旧段）。
    let d = bridge.register_init_demand("SlowInit", "Slow", state_id, Vec::new());
    assert_eq!(d, InitDemandDecision::Queued);
    assert_eq!(bridge.pending_init_demand_count(), 1, "B demand dropped");
    let report = bridge.dispatch_pending_inits(budget);
    assert_eq!(report.in_flight, 1, "second A parks again (fresh run)");
    let mut rounds = 0;
    while bridge.has_cpu_continuations() {
        rounds += 1;
        assert!(rounds < 10_000, "pump not progressing");
        bridge.resume_cpu_slices(budget);
    }
    let report = bridge.dispatch_pending_inits(budget);
    assert_eq!(report.dispatched, 0, "queue drained");
    assert_eq!(
        bridge.init_demand_phase("SlowInit"),
        Some(InitDemandPhase::Done)
    );
    // 取消不回滚前缀副作用（M-01 契约）：首个 A 代际的首片前缀写入保留
    //（27 量级），第二个 A 全量重跑 20000 次递增——n = 前缀 + 20000，
    // 界 [20001, 40000) 证明"全量重跑"而非"复活旧段续跑"（后者 <20000）
    // 也非"双倍完整跑"（后者 ≥40000）。
    let n = bridge.read_state("n").expect("read state");
    match n {
        auto_val::Value::Int(v) => {
            assert!(
                (20001..40000).contains(&v),
                "fresh A generation must complete a full 20000 run on top of preserved prefix writes, got {v}"
            );
        }
        other => panic!("n must be Int, got {other:?}"),
    }
}

/// 依赖序：页 demand 在途（park）时后继 child demand 不派发；页到终态后
/// 后继继续消费（FIFO = 渲染登记序 = 页先 child 后）。页/child 用同一桥
/// 内两个真 widget（root=SlowInit 长 Init，child=Flow 短 Init）。
#[cfg(feature = "ui")]
#[test]
fn plan711_init_dependency_order_child_waits_for_parent() {
    let bridge = two_widget_bridge(SLOW_WIDGET, "SlowInit", TICK_WIDGET, "Tick");
    let state_id = bridge.state_obj_id();
    let budget = test_budget(512, 10_000_000);

    // 页（长 Init）先登记，child 后登记——渲染路径的登记序天然如此
    //（outlet 页 memo 先于 child 渲染）。
    bridge.register_init_demand("SlowInit", "SlowInit#1", state_id, Vec::new());
    bridge.register_init_demand("Tick", "Tick#1", state_id, Vec::new());

    // 一轮派发：页 park → 本轮停止，child 仍在队列（依赖序）。
    let report = bridge.dispatch_pending_inits(budget);
    assert_eq!(report.in_flight, 1, "page init parks");
    assert_eq!(
        report.dispatched, 1,
        "child must NOT dispatch behind a parked parent"
    );
    assert_eq!(
        bridge.init_demand_phase("SlowInit"),
        Some(InitDemandPhase::InFlight)
    );
    assert_eq!(bridge.pending_init_demand_count(), 1, "child still queued");
    assert_eq!(bridge.init_demand_phase("Tick"), Some(InitDemandPhase::Queued));

    // 页推进到终态 → 下一轮驱动：InFlight 探测收敛 → child 派发完成。
    let mut rounds = 0;
    while bridge.has_cpu_continuations() {
        rounds += 1;
        assert!(rounds < 10_000, "pump not progressing");
        bridge.resume_cpu_slices(budget);
    }
    let report = bridge.dispatch_pending_inits(budget);
    assert_eq!(report.completed, 1, "child dispatches after parent terminal");
    assert_eq!(bridge.init_demand_phase("Tick"), Some(InitDemandPhase::Done));
    assert_eq!(bridge.pending_init_demand_count(), 0);
}

/// Missing 静默：声明了 lifecycle.Init 但导出缺失（登记了不存在的
/// widget）→ 静默记 Done，不报错不重试。
#[cfg(feature = "ui")]
#[test]
fn plan711_init_missing_handler_silently_done() {
    let bridge = bridge_from(FLOW_WIDGET, "Flow");
    let state_id = bridge.state_obj_id();
    let budget = test_budget(4096, 10_000_000);

    bridge.register_init_demand("NoSuchWidget", "NoSuchWidget", state_id, Vec::new());
    let report = bridge.dispatch_pending_inits(budget);
    assert_eq!(report.missing, 1, "missing init recorded");
    assert!(report.failed.is_empty(), "missing is silent");
    assert_eq!(
        bridge.init_demand_phase("NoSuchWidget"),
        Some(InitDemandPhase::Done),
        "missing must not retry"
    );
}

/// Failed 记账：派发即 VM 错误 → Failed 终态，错误上报；同身份登记不再
/// 重派（不无限重试）。
#[cfg(feature = "ui")]
#[test]
fn plan711_init_dispatch_failure_records_without_retry() {
    // Init 体内数组越界 → 运行时错误（确定性 VM error）。
    let code = r#"
widget Boom {
  model {
    hits int = 0
  }
  on {
    .Init -> {
      var arr = [1, 2]
      hits = arr[9]
    }
  }
  view {
    text "boom" {}
  }
}
"#;
    let bridge = bridge_from(code, "Boom");
    let state_id = bridge.state_obj_id();
    let budget = test_budget(4096, 10_000_000);

    bridge.register_init_demand("Boom", "Boom", state_id, Vec::new());
    let report = bridge.dispatch_pending_inits(budget);
    assert_eq!(report.failed.len(), 1, "dispatch error recorded: {:?}", report.failed);
    assert_eq!(
        bridge.init_demand_phase("Boom"),
        Some(InitDemandPhase::Failed),
        "failed is terminal"
    );
    // 同身份登记：不重派（不无限重试）。
    let d = bridge.register_init_demand("Boom", "Boom", state_id, Vec::new());
    assert_eq!(d, InitDemandDecision::AlreadyKnown);
    assert_eq!(bridge.pending_init_demand_count(), 0);
}
