//! PLAN-608 T05：CALL_SPEC 内联分发区不可达臂金样守护。
//!
//! T-01 定罪结论（见 docs/plans/608 §9 矩阵）：engine.rs 内联 str/List
//! 分发区的一批方法臂**静态不可达**——registry（NativeInterface 名表，
//! canonical "List.X"→"auto.list.X"）恒先命中同名 shim。其中内联
//! "push" 臂 `list.push(elem_val)` 裸存不 retain：**若 registry 覆盖
//! 回退（删名/改名）使该臂复活，即 UAF 面**，而非单纯泄漏。
//!
//! 本守护按用户指令（2026-09-11「路径若未触及，以测试检测」）钉死路由：
//! registry 覆盖变化触雷时本测试先红，而不是现场堆腐化。行为金样另钉
//! 未知方法兜底臂（str/List `_` fallback）的现行为：弹接收者+实参、
//! 压 null、栈平衡、池配平。

use crate::vm::engine::AutoVM;
use crate::vm::virt_memory::VirtualFlash;

fn vm_idle() -> AutoVM {
    AutoVM::new(VirtualFlash::new_with_code(vec![]), 1024)
}

/// 路由守护：不可达内联臂的方法名必须由 registry resolve 命中且 shim
/// 已绑定。任何一行变红 = 对应内联死臂复活（"push" 行 = UAF 风险面）。
#[test]
fn callspec_route_guard_registry_covers_dead_arms() {
    let vm = vm_idle();
    // List 内联死臂（engine.rs type_name=="List" match，PLAN-608 §9 矩阵）。
    let list_methods = [
        "push", // 裸存不 retain —— UAF 面（KD-VM5 移除候选）
        "get", "pop", "insert", "remove", "sort", "sort_by", "reverse", "len", "map", "filter",
        "contains", "join",
    ];
    for m in list_methods {
        let name = format!("List.{}", m);
        let id = vm.native_interface.resolve(&name).unwrap_or_else(|| {
            panic!(
                "registry 覆盖回退：{} 不再命中 shim——engine.rs 内联 \"{}\" \
                 死臂复活（push 臂无 retain = UAF 面；PLAN-608 路由守护）",
                name, m
            )
        });
        assert!(
            vm.native_interface.get(id).is_some(),
            "{} 命中 id={} 但 shim 未绑定（分发将 MissingNative）",
            name,
            id
        );
    }
    // str 内联死臂（resolve 命中 shim 的注册名集合，PLAN-608 §4 差集）。
    let str_methods = [
        "upper",
        "to_upper",
        "to_lower",
        "split",
        "is_empty",
        "starts_with",
        "ends_with",
        "contains",
        "len",
        "trim",
        "replace",
        "parse_int",
        "parse_float",
    ];
    for m in str_methods {
        let name = format!("str.{}", m);
        let id = vm.native_interface.resolve(&name).unwrap_or_else(|| {
            panic!(
                "registry 覆盖回退：{} 不再命中 shim（PLAN-608 路由守护）",
                name
            )
        });
        assert!(
            vm.native_interface.get(id).is_some(),
            "{} 命中 id={} 但 shim 未绑定",
            name,
            id
        );
    }
}

/// 行为金样：未知 str/List 方法走 CALL_SPEC 内联 `_` 兜底臂——弹
/// 接收者+实参、压 null、后续调用栈平衡、池份额配平（PLAN-608 T02
/// 已给兜底臂补结算；本测试钉"返回 null + 无泄漏"现行为）。
#[tokio::test]
async fn callspec_unknown_method_fallback_pins_null_and_balance() {
    let code = r#"
fn main() int {
    var x str = "abc"
    var y = x.frobnicate()
    print(y)
    var after = x.trimEnd()
    print(after)
    var l List<str> = List<str>.new(["m"])
    var z = l.frobnicate()
    print(z)
    print(l.len())
    0
}
"#;
    let (vm, stdout, entry, _rt) = crate::create_vm_from_source(code).expect("compile failed");
    let tid = vm.spawn_task(entry, 65536);
    vm.run_task_loop().await;
    if let Some(arc) = vm.tasks.get(&tid) {
        let mut t = arc.lock().await;
        vm.rc_release_task_stack(&mut t);
    }
    vm.reap_all();
    vm.tasks.remove(&tid);
    let out = stdout.read().unwrap().clone();
    // 现行为：未知方法 → None（PLAN-057 兜底语义）；后续调用栈不偏移。
    assert!(out.contains("None"), "未知 str 方法应压 None：[{}]", out);
    assert!(
        out.contains("abc"),
        "兜底后 trimEnd 仍正确（栈平衡）：[{}]",
        out
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 4, "四行输出：[{}]", out);
    assert_eq!(lines[2], "None", "未知 List 方法应压 None：[{}]", out);
    assert_eq!(
        lines[3], "1",
        "兜底后 l.len()=1（栈平衡+列表完好）：[{}]",
        out
    );
    let h = vm.pool_health();
    assert_eq!(h.underflow_events, 0, "不得引入多扣款：{:?}", h);
    assert_eq!(h.live_shares, 0, "兜底臂结算后池配平：{:?}", h);
}
