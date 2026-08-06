//! Plan 317: Task actor state fields — regression tests.
//!
//! State fields (`task T { count = 0 }`) persist across handler invocations
//! on the actor's AutoVM task, accessed via LOAD_STATE_FIELD/STORE_STATE_FIELD.

use crate::run_with_capture;

/// State field is initialized (via start hook) and a handler can write it.
#[test]
fn actor_state_field_write() {
    let code = r#"
task Counter {
    count = 0
    fn start() ! { }
    on {
        1 -> {
            count = 42
        }
    }
}
fn main() {
    let h = Task.spawn("Counter", 16)
    h.send(1)
    print("ok")
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    assert!(s.contains("ok"), "state write: stdout={:?} result={:?}", s, r);
}

/// State field increment persists across multiple handler invocations.
/// count starts at 0; each message does count = count + 1.
/// After 3 messages, count == 3 (verified via conditional branch).
#[test]
fn actor_state_field_increment_persists() {
    let code = r#"
task Counter {
    count = 0
    fn start() ! { }
    on {
        1 -> {
            count = count + 1
            if count == 2 {
                print("reached two")
            }
            if count == 3 {
                print("reached three")
            }
        }
    }
}
fn main() {
    let h = Task.spawn("Counter", 16)
    h.send(1)
    h.send(1)
    h.send(1)
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // count increments 1, 2, 3 across the three messages. The conditionals
    // fire on the 2nd (count==2) and 3rd (count==3) invocations.
    assert!(s.contains("reached two"), "count==2: stdout={:?} result={:?}", s, r);
    assert!(s.contains("reached three"), "count==3: stdout={:?} result={:?}", s, r);
}

/// Plan 390 §5 Phase A (M1): Task.spawn passes init args that override the
/// state field's declared default. `Task.spawn("Counter", 16, 41)` sets
/// `count` to 41 at spawn; a handler then does `count = count + 1` → 42.
/// Uses a literal-pattern handler (`1 ->`) to avoid the unrelated bound-variable
/// VM defect (Plan 043). Verified end-to-end: start reads 41, handler prints 42.
#[test]
fn actor_spawn_init_arg_overrides_default() {
    let code = r#"
task Counter {
    count = 0
    on {
        1 -> {
            count = count + 1
            print(count)
        }
    }
}
fn main() {
    let h = Task.spawn("Counter", 16, 41)
    h.send(1)
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // The injected count=41 survives to the handler; +1 → 42.
    assert!(s.contains("42"), "spawn init arg (count=41 → +1 = 42): stdout={:?} result={:?}", s, r);
}

/// VM bug fix: a task with state fields but NO `fn start()` now applies the
/// declared default initializers. Previously the whole #start block (initializers
/// + #start export) was gated on `start_hook.is_some()`, so `count = 5` was
/// silently dropped (state started at 0) and shim_task_spawn_vm fell back to
/// ip=0. Now `count = 5` runs, handler does +1 → 6.
#[test]
fn actor_state_default_init_without_fn_start() {
    let code = r#"
task Counter {
    count = 5
    on {
        1 -> {
            count = count + 1
            print(count)
        }
    }
}
fn main() {
    let h = Task.spawn("Counter", 16)
    h.send(1)
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // Default count=5 applied (no fn start), handler +1 → 6.
    assert!(s.contains("6"), "state default-init (count=5 → +1 = 6): stdout={:?} result={:?}", s, r);
}

/// VM bug fix interaction: spawn init args override the (now-working) declared
/// defaults. `Task.spawn("Counter", 16, 100)` injects count=100; the #start
/// default initializer `count = 5` is skipped for the locked field, so the
/// handler sees 100 (+1 → 101), not 5.
#[test]
fn actor_spawn_init_arg_overrides_now_working_default() {
    let code = r#"
task Counter {
    count = 5
    on {
        1 -> {
            count = count + 1
            print(count)
        }
    }
}
fn main() {
    let h = Task.spawn("Counter", 16, 100)
    h.send(1)
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // Injected count=100 wins over default 5; handler +1 → 101.
    assert!(s.contains("101"), "spawn init arg overrides default (100 → +1 = 101): stdout={:?} result={:?}", s, r);
}

/// Plan 390 §G2: VM bound-variable message handler. `on { n int -> }` binds the
/// message payload to `n` so the body can read it. Verified across MULTIPLE sends:
/// each invocation must see its own message (not a stale value from the previous
/// handler call). Previously `n` was unbound ("Undefined variable: n"); an initial
/// codegen-only fix worked for a single send but went stale on the 2nd because the
/// handler reuses the task's bp=0 frame with no per-invocation reset. The runtime
/// fix (§15.8 方案 B：真 bp 帧 + park_ip 重 park) makes each invocation clean.
#[test]
fn actor_bound_var_handler_multi_send() {
    let code = r#"
task Adder {
    total = 0
    on { n int -> {
        total = total + n
        print(total)
    } }
}
fn main() {
    let h = Task.spawn("Adder", 16)
    h.send(5)
    h.send(7)
    h.send(3)
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // total: 0+5=5, 5+7=12, 12+3=15. Each n is the current message (5,7,3).
    assert!(s.contains("5"), "bound-var msg1 (total=5): stdout={:?} result={:?}", s, r);
    assert!(s.contains("12"), "bound-var msg2 (total=12): stdout={:?} result={:?}", s, r);
    assert!(s.contains("15"), "bound-var msg3 (total=15): stdout={:?} result={:?}", s, r);
}

/// Plan 390 §14 L3: WithBindings multi-field variant message. `on { Add(a int, b int) -> }`
/// matches a structured message sent as `h.send(Add(3, 5))`. The codegen constructs a
/// Value::Obj {__variant:"Add", a:3, b:5}, send delivers the VmRef, the wake path pushes
/// it back, and the handler DUP+GET_FIELDs each binding into a named local. This was
/// completely unsupported before (send only carried i32; WithBindings handler was never
/// invoked because the matcher expects Value::Obj/Str for variant patterns).
#[test]
fn actor_withbindings_multi_field() {
    let code = r#"
task Calculator {
    total = 0
    on { Add(a int, b int) -> {
        total = total + a + b
        print(total)
    } }
}
fn main() {
    let h = Task.spawn("Calculator", 16)
    h.send(Add(3, 5))
    h.send(Add(10, 20))
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // 0+3+5=8, 8+10+20=38
    assert!(s.contains("8"), "multi-field msg1 (total=8): stdout={:?} result={:?}", s, r);
    assert!(s.contains("38"), "multi-field msg2 (total=38): stdout={:?} result={:?}", s, r);
}

/// Plan 390 §15 G2-refactor (方案 B) 边界测试：3 字段 WithBindings + 深嵌套表达式。
/// 方案 A 的 HANDLER_LOCALS_BAND=16 预留带在「多字段 + 深表达式临时值」下可能被穿透
/// （handler locals 与 DUP/GET_FIELD/算术临时值重叠）。方案 B 的真 bp 帧使 locals
/// （bp+1..bp+16）永远在 message/临时值之上，根治此问题。
/// `Add3(a,b,c)` 三字段 + `((a+b)*(a+c))-(b*c)` 深表达式 → 验证字段不串、表达式不串。
#[test]
fn actor_withbindings_deep_expr_three_fields() {
    let code = r#"
task Calc3 {
    total = 0
    on { Add3(a int, b int, c int) -> {
        total = total + ((a + b) * (a + c)) - (b * c)
        print(total)
    } }
}
fn main() {
    let h = Task.spawn("Calc3", 16)
    h.send(Add3(2, 3, 5))
    h.send(Add3(1, 1, 1))
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // msg1: ((2+3)*(2+5))-(3*5) = (5*7)-15 = 35-15 = 20; total = 0+20 = 20
    // msg2: ((1+1)*(1+1))-(1*1) = (2*2)-1 = 3; total = 20+3 = 23
    assert!(s.contains("20"), "deep-expr msg1 (total=20): stdout={:?} result={:?}", s, r);
    assert!(s.contains("23"), "deep-expr msg2 (total=23): stdout={:?} result={:?}", s, r);
}

/// Plan 317 §11 Phase 5 (P2 回归): state field 名作为 `print(count)` 的直接参数。
///
/// §9 已知遗留 #1 曾记录"state field 名作为 print(count) 参数时报 undefined
/// variable"——某些 intrinsic/let 的 codegen 路径绕过 state field 检查。实测已
/// 自然解决(`Expr::Ident` 分支 codegen.rs:5113 正确检查 `current_task_state_fields`)。
/// 本测试固化此修复,防回归。
#[test]
fn actor_state_field_print_direct() {
    let code = r#"
task Counter {
    count = 0
    fn start() ! {
        count = count + 1
        print(count)
    }
}
fn main() {
    let h = Task.spawn("Counter", 0)
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // count 初始 0,start hook 里 +1 → 1,print(count) 应输出 "1"。
    // 修复前:print(count) 报 "Undefined variable: count"(intrinsic 参数路径
    // 未识别 state field)。
    assert!(s.contains("1"), "print(count) state field: stdout={:?} result={:?}", s, r);
    assert!(!r.contains("Undefined variable"), "count should resolve as state field: {:?}", r);
}

/// Plan 317 §11 Phase 5 (P2 回归): state field 名作为 `let c = count` 的 RHS。
///
/// §9 已知遗留 #1 的第二个具体场景:let 绑定 RHS 读取 state field。实测已解决。
#[test]
fn actor_state_field_let_rhs() {
    let code = r#"
task Counter {
    count = 0
    fn start() ! {
        count = count + 1
        let c = count
        print(c.to(str))
    }
}
fn main() {
    let h = Task.spawn("Counter", 0)
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // count → 1,let c = count 把 state field 读进 local c,print 输出 "1"。
    assert!(s.contains("1"), "let c = count (state field RHS): stdout={:?} result={:?}", s, r);
    assert!(!r.contains("Undefined variable"), "count should resolve as state field on let RHS: {:?}", r);
}
