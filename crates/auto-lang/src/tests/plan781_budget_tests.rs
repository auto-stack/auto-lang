//! PLAN-781 T-15: HTTP 请求级累计步预算单测（合同 =
//! docs/plans/reports/781-budget-decision.md D-1/D-4）。
//!
//! - V-16 累计：N-1 耗尽 / N 通过 / N+1 余量 的逐指令精确边界；嵌套 fn
//!   同一 drive 循环计数；park/resume 段间不重置（真实本地 server 受控
//!   延迟，plan702 同款确定性 harness）。
//! - V-14 隔离与默认不变：policy 是 per-VM 快照字段——两实例不同 cap 互
//!   不影响；未设 policy 的实例逐字节走 Legacy 10M 段帽。
//! - V-15 配置表：vm/budget_policy.rs 内联（解析层）。

use crate::vm::engine::{AutoVM, DriveBudget, ParkedWait, SegmentOutcome};
use crate::vm::task::AutoTask;

/// 构造带 policy 快照的 VM（绕过 env——单测进程 env 串扰面）。
fn vm_with_policy(code: &str, cap: Option<u64>) -> AutoVM {
    let (mut vm, _stdout, _entry, _t) = crate::create_vm_from_source(code).expect("compile");
    vm.http_step_budget = cap;
    vm
}

const LOOP_FN: &str = r#"
fn spin(n int) int {
    var acc int = 0
    var i int = 0
    while i < n {
        acc = acc + 1
        i = i + 1
    }
    return acc
}
"#;

/// 计跑一个 handler 的实际指令数（cap=HUGE 全量跑完，读 task.cpu_steps_total）。
fn measure_steps(vm: &AutoVM, fn_name: &str, args: &[i64]) -> u64 {
    let mut task = AutoTask::new(0, 65536, 0);
    for a in args {
        task.ram.push_i32(*a as i32);
    }
    let out = vm.call_fn_by_name_segment_http(&mut task, fn_name, args.len());
    match out {
        SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("measure run must complete: {other:?}"),
    }
    task.cpu_steps_total
}

/// V-16(a)：N-1 耗尽 / N 通过 / N+1 余量——逐指令精确边界。
#[test]
fn plan781_v16_budget_boundary_n_minus_one_n_n_plus_one() {
    // 先量出 spin(20) 的精确指令数（大 cap 全量跑）。
    let measurer = vm_with_policy(LOOP_FN, Some(4_000_000_000));
    let n_steps = measure_steps(&measurer, "spin", &[20]);
    assert!(n_steps > 50 && n_steps < 10_000, "sanity: {n_steps}");

    // N = 精确步数：N 通过、N-1 耗尽、N+1 通过且有余量。
    let vm = vm_with_policy(LOOP_FN, Some(n_steps));
    let mut task = AutoTask::new(0, 65536, 0);
    task.ram.push_i32(20);
    let out = vm.call_fn_by_name_segment_http(&mut task, "spin", 1);
    assert!(
        matches!(out, SegmentOutcome::Completed(Ok(()))),
        "budget N must pass exactly: {out:?}"
    );

    let vm = vm_with_policy(LOOP_FN, Some(n_steps - 1));
    let mut task = AutoTask::new(0, 65536, 0);
    task.ram.push_i32(20);
    let out = vm.call_fn_by_name_segment_http(&mut task, "spin", 1);
    match out {
        SegmentOutcome::Completed(Err(e)) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("instruction budget exhausted")
                    && msg.contains("per-request cumulative budget")
                    && msg.contains("AUTO_HTTP_STEP_BUDGET"),
                "exhaustion must be a named resource rejection: {msg}"
            );
        }
        other => panic!("budget N-1 must exhaust: {other:?}"),
    }

    let vm = vm_with_policy(LOOP_FN, Some(n_steps + 1));
    let mut task = AutoTask::new(0, 65536, 0);
    task.ram.push_i32(20);
    let out = vm.call_fn_by_name_segment_http(&mut task, "spin", 1);
    assert!(
        matches!(out, SegmentOutcome::Completed(Ok(()))),
        "budget N+1 passes: {out:?}"
    );
}

/// V-16(b)：嵌套 fn 不重置——内层调用与外层同循环计数（单 drive 内累计）。
#[test]
fn plan781_v16_nested_fn_counts_into_same_budget() {
    let code = r#"
fn inner(k int) int {
    var acc int = 0
    var i int = 0
    while i < k {
        acc = acc + 2
        i = i + 1
    }
    return acc
}
fn outer(k int) int {
    let a int = inner(k)
    let b int = inner(k)
    return a + b
}
"#;
    let measurer = vm_with_policy(code, Some(4_000_000_000));
    let total = measure_steps(&measurer, "outer", &[10]);
    // 半预算 = 只够跑第一个 inner（约一半指令）→ 必耗尽于第二段内层循环。
    let half = total / 2;
    let vm = vm_with_policy(code, Some(half));
    let mut task = AutoTask::new(0, 65536, 0);
    task.ram.push_i32(10);
    let out = vm.call_fn_by_name_segment_http(&mut task, "outer", 1);
    match out {
        SegmentOutcome::Completed(Err(e)) => {
            assert!(
                format!("{e:?}").contains("per-request cumulative budget"),
                "nested fn must consume the same request budget: {e:?}"
            );
        }
        other => panic!("half budget across nested fns must exhaust: {other:?}"),
    }
}

/// V-16(c)：park/resume 段间累计不重置——首段跑掉预算大头后 park（异步
/// HTTP），resume 段在剩余预算内必然耗尽（plan702 同款受控 server）。
#[test]
fn plan781_v16_park_resume_cumulative_not_reset() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    // 多连接 server：同一 VM 的 .at 串常量钉死端口——测量跑 + 两预算跑
    // 共用本端口，逐连接应答（受控 200ms 延迟逐次生效）。
    let server = std::thread::spawn(move || {
        for _ in 0..8 {
            let Ok((mut stream, _)) = listener.accept() else { break };
            let mut buf = [0u8; 4096];
            let mut read_total = 0usize;
            loop {
                let n = std::io::Read::read(&mut stream, &mut buf).unwrap_or(0);
                read_total += n;
                if read_total >= 4
                    && buf[..read_total].windows(4).position(|w| w == b"\r\n\r\n").is_some()
                {
                    break;
                }
                if n == 0 {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
            let body = r#"{"ok":true}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
        }
    });

    let code = r#"
fn mix() str {
    var acc int = 0
    var i int = 0
    while i < 5000 {
        acc = acc + 1
        i = i + 1
    }
    var resp str = Http.post_json("http://127.0.0.1:7810/x", "q=1")
    var j int = 0
    while j < 5000 {
        acc = acc + 1
        j = j + 1
    }
    return resp
}
"#;
    let code = code.replace("7810", &port.to_string());

    // 全量预算量总步数（两段+park）。
    let measurer = vm_with_policy(&code, Some(4_000_000_000));
    let mut task = AutoTask::new(0, 65536, 0);
    let out = vm_call_to_completion(&measurer, &mut task, "mix");
    assert!(matches!(out, SegmentOutcome::Completed(Ok(()))), "{out:?}");
    let total = task.cpu_steps_total;
    assert!(total > 1000, "sanity {total}");

    // 预算 = 总量 + 1：首段（≈一半）park 后，resume 段必须仍以累计执法——
    // 若段间重置，resume 段拥有完整余量并跑完（旧缺陷形态）→ 通过即红。
    let vm = vm_with_policy(&code, Some(total + 1));
    let mut task = AutoTask::new(0, 65536, 0);
    let out = vm_call_to_completion(&vm, &mut task, "mix");
    assert!(
        matches!(out, SegmentOutcome::Completed(Ok(()))),
        "total+1 budget must complete across park/resume: {out:?}"
    );

    // 预算 = 总量 - 1：resume 段耗尽（累计不重置的直接证据）。
    let vm = vm_with_policy(&code, Some(total - 1));
    let mut task = AutoTask::new(0, 65536, 0);
    let out = vm_call_to_completion(&vm, &mut task, "mix");
    match out {
        SegmentOutcome::Completed(Err(e)) => {
            assert!(
                format!("{e:?}").contains("per-request cumulative budget"),
                "resume segment must inherit cumulative accounting: {e:?}"
            );
        }
        other => panic!("total-1 across park/resume must exhaust: {other:?}"),
    }
    drop(server);
}

/// 手动驱动到终态（模拟 http step 循环的 park→resume 泵；resume 用 http 变体）。
fn vm_call_to_completion(vm: &AutoVM, task: &mut AutoTask, fn_name: &str) -> SegmentOutcome {
    let mut outcome = vm.call_fn_by_name_segment_http(task, fn_name, 0);
    let mut guard = 0;
    while let SegmentOutcome::Parked { wait, seg } = outcome {
        guard += 1;
        assert!(guard < 200, "resume loop runaway");
        match wait {
            ParkedWait::HttpRequest(req_id) => {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
                    if std::time::Instant::now() > deadline {
                        panic!("test server response never arrived");
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
            other => panic!("unexpected park wait in test: {other:?}"),
        }
        outcome = vm.resume_fn_by_name_segment_http(task, &seg);
    }
    outcome
}

/// V-14(a)：policy 是 per-VM 快照——同进程两实例不同 cap 互不影响。
#[test]
fn plan781_v14_policy_is_per_instance_snapshot() {
    let measurer = vm_with_policy(LOOP_FN, Some(4_000_000_000));
    let n_steps = measure_steps(&measurer, "spin", &[10]);

    let tight = vm_with_policy(LOOP_FN, Some(n_steps / 2));
    let loose = vm_with_policy(LOOP_FN, Some(4_000_000_000));

    let mut t1 = AutoTask::new(0, 65536, 0);
    t1.ram.push_i32(10);
    let out1 = tight.call_fn_by_name_segment_http(&mut t1, "spin", 1);
    assert!(
        matches!(out1, SegmentOutcome::Completed(Err(_))),
        "tight instance rejects: {out1:?}"
    );

    let mut t2 = AutoTask::new(0, 65536, 0);
    t2.ram.push_i32(10);
    let out2 = loose.call_fn_by_name_segment_http(&mut t2, "spin", 1);
    assert!(
        matches!(out2, SegmentOutcome::Completed(Ok(()))),
        "loose instance unaffected: {out2:?}"
    );
}

/// V-14(b)：未设 policy = Legacy 逐字节（10M 段帽仍在，LegacyHttp 档不进入）。
#[test]
fn plan781_v14_default_is_byte_identical_legacy() {
    let vm = vm_with_policy(LOOP_FN, None);
    let mut task = AutoTask::new(0, 65536, 0);
    task.ram.push_i32(10);
    // 未设 policy → 入口落 DriveBudget::Legacy（间接验证：同入口在两档下的
    // 形态一致——Legacy 档无累计记账）。
    let out = vm.call_fn_by_name_segment_http(&mut task, "spin", 1);
    assert!(matches!(out, SegmentOutcome::Completed(Ok(()))));
    assert_eq!(
        task.cpu_steps_total, 0,
        "Legacy path must not accumulate (byte-identical semantics)"
    );
}

/// DriveBudget 变体形状锁定（防止未来重构静默丢掉 LegacyHttp 档）。
#[test]
fn plan781_drive_budget_variants_shape() {
    assert!(matches!(DriveBudget::Legacy, DriveBudget::Legacy));
    assert!(matches!(DriveBudget::LegacyHttp(10), DriveBudget::LegacyHttp(_)));
    assert!(matches!(
        DriveBudget::CpuSlice(crate::vm::engine::CpuSliceBudget::d2_default()),
        DriveBudget::CpuSlice(_)
    ));
}

/// V-20 closure 入口：合成 fn-ref closure（plan705 T-06(2) 同形态——
/// axum_adapter 路由注册的 closure 载体）在 policy 下同样耗尽——
/// `call_closure_segment` 是 axum closure 臂的唯一驱动（D-2 入口矩阵）。
#[test]
fn plan781_v20_closure_entry_consumes_policy() {
    let code = r#"
fn closure_target() str {
    var acc int = 0
    var i int = 0
    while i < 5000 {
        acc = acc + 1
        i = i + 1
    }
    return "done"
}
"#;
    let (mut vm, _out, _entry, _t) = crate::create_vm_from_source(code).expect("compile");
    // 紧预算：该循环必然超出。
    vm.http_step_budget = Some(300);
    let (addr, _name) = vm
        .flash
        .exports_by_name
        .iter()
        .find(|(n, _)| n.contains("closure_target"))
        .map(|(n, a)| (*a, n.clone()))
        .expect("export closure_target");
    let closure_id = vm
        .closure_id_gen
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    vm.closures.insert(
        closure_id,
        crate::vm::engine::Closure {
            func_addr: addr as u32,
            env: Default::default(),
            n_args: 0,
            capture_slots: Default::default(),
            param_abs: Default::default(),
            creator_frame: None,
        },
    );
    let mut task = AutoTask::new(0, 65536, 0);
    let out = vm.call_closure_segment(&mut task, closure_id, 0);
    match out {
        SegmentOutcome::Completed(Err(e)) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("per-request cumulative budget"),
                "closure entry must consume the request budget policy: {msg}"
            );
        }
        other => panic!("tight budget must exhaust closure drive: {other:?}"),
    }
}

/// F-781-E01 回归（PLAN-781 T-22 根修）：跨帧 raise+catch 后调用方帧
/// 槽完整性。根因 = intercept_error 跨帧弹帧条件 `old_bp >= handler.bp`
/// 把 try 帧自身也弹掉（old_bp 全仓语义=帧自身 bp）→ 同帧 catch 后 RET
/// 弹空、current_fn_n_args 不恢复 → 调用方 LOAD_LOCAL 参数臂
/// （bp-n_args+param_idx-1 依赖该计数）走 null 守卫读 None。四形态：
/// 同帧 native raise / 跨两层 / catch 带参绑定 / opcode raise（除零）。
#[test]
fn plan781_e01_cross_frame_raise_catch_restores_caller_frame() {
    let code = r#"
fn maybe_raise() str {
    var out str = "ok"
    try {
        json.parse("{bad")
        out = "no-raise"
    } catch {
        out = "caught"
    }
    return out
}
fn caller(n int) int {
    let a str = maybe_raise()
    let r str = a + "|n=" + n.to_string()
    if r == "caught|n=42" {
        return 1
    }
    return 0
}
fn deep_raise() str {
    var z int = 0
    var d int = 1 / z
    return "unreachable" + d.to_string()
}
fn mid() str {
    var out str = "ok"
    try {
        out = deep_raise()
    } catch {
        out = "deep-caught"
    }
    return out
}
fn outer(k int) int {
    let a str = mid()
    let r str = a + "|k=" + k.to_string()
    if r == "deep-caught|k=7" {
        return 1
    }
    return 0
}
fn with_param() str {
    var out str = "ok"
    try {
        json.parse("[bad")
    } catch e {
        out = "p:" + e.len().to_string()
    }
    return out
}
fn wp_caller(m int) int {
    let a str = with_param()
    let r str = a + "|m=" + m.to_string()
    if r.len() > 4 {
        return 1
    }
    return 0
}
"#;
    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(code).expect("compile");
    for (fname, arg) in [("caller", 42i32), ("outer", 7), ("wp_caller", 9)] {
        let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
        task.ram.push_i32(arg);
        let outcome = vm.call_fn_by_name_segment(&mut task, fname, 1);
        match outcome {
            crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {
                let nv = task.ram.pop_nv();
                assert_eq!(
                    auto_val::decode_i32(nv),
                    1,
                    "{fname}: caller frame poisoned after callee catch"
                );
            }
            other => panic!("{fname}: unexpected outcome {other:?}"),
        }
    }
}

/// PLAN-781 T-25（R05 r6）：`type_of` 类型谓词——浮点 0.0 与 int 0 的
/// 可靠区分（字符串形态判别对 0.0 失真：位模式恰为 0）。
#[test]
fn plan781_type_of_vm_tags() {
    let code = r#"
fn tag_int() str { return type_of(0) }
fn tag_float_zero() str { return type_of(0.0) }
fn tag_float() str { return type_of(1.5) }
fn tag_str() str { return type_of("19") }
fn tag_bool() str { return type_of(true) }
fn tag_null() str { return type_of(null) }
fn tag_list() str { return type_of([1, 2]) }
fn tag_obj() str { return type_of(json.parse("{\"a\":1}")) }
fn tag_parsed_zero() str { return type_of(json.parse("{\"x\":0.0}").x) }
fn tag_parsed_int_zero() str { return type_of(json.parse("{\"x\":0}").x) }
fn tag_neg_half() str { return type_of(json.parse("{\"x\":-0.5}").x) }
"#;
    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(code).expect("compile");
    let expect = |fname: &str, want: &str| {
        let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
        let outcome = vm.call_fn_by_name_segment(&mut task, fname, 0);
        match outcome {
            crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {
                let nv = task.ram.pop_nv();
                assert!(
                    auto_val::is_string(nv),
                    "{fname}: expected string tag, nv={nv:?}"
                );
                let idx = auto_val::decode_string(nv) as usize;
                let pool = vm.strings.read().unwrap();
                let s = String::from_utf8(pool[idx].clone()).unwrap();
                assert_eq!(s, want, "{fname}");
            }
            other => panic!("{fname}: unexpected outcome {other:?}"),
        }
    };
    expect("tag_int", "int");
    expect("tag_float_zero", "float");
    expect("tag_float", "float");
    expect("tag_str", "str");
    expect("tag_bool", "bool");
    expect("tag_null", "null");
    // 数组字面量为堆对象（tag 5）；is_list 为专用列表引用形态。
    expect("tag_list", "object");
    expect("tag_obj", "object");
    expect("tag_parsed_zero", "float");
    expect("tag_parsed_int_zero", "int");
    expect("tag_neg_half", "float");
}

/// F-781-R07 回归：type_of 消费输入的引用份额必须释放——100 次重复
/// 调用（对象参 / 每轮新动态字符串参）后终态资源平衡（heap/池回到
/// 基线；复审独立宿主实证修复前 heap 0→100、pool +100）。
#[test]
fn plan781_type_of_input_stake_released_repeated_calls() {
    let code = r#"
fn call_obj() str {
    let o = json.parse("{\"a\":1}")
    return type_of(o)
}
fn call_str(s str) str {
    return type_of(s)
}
fn warm_str() str {
    return type_of("x")
}
fn call_str_once(i int) str {
    return type_of("dyn-" + i.to_string())
}
"#;
    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(code).expect("compile");
    let vm = std::rc::Rc::new(vm);
    let run = |fname: &str, arg: Option<i32>| -> String {
        let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
        if let Some(a) = arg {
            task.ram.push_i32(a);
        }
        let outcome = vm.call_fn_by_name_segment(&mut task, fname, usize::from(arg.is_some()));
        match outcome {
            crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {
                let nv = task.ram.pop_nv();
                let idx = auto_val::decode_string(nv) as usize;
                let s = {
                    let pool = vm.strings.read().unwrap();
                    String::from_utf8(pool[idx].clone()).unwrap()
                };
                // 排除延迟回收：显式任务栈释放 + 全量收割（复审探针同款）。
                vm.rc_release_task_stack(&mut task);
                drop(task);
                vm.reap_all();
                s
            }
            other => panic!("{fname}: unexpected {other:?}"),
        }
    };
    // 对象参：100 次调用后 heap 回基线。
    let heap_base = vm.heap_objects.len();
    for _ in 0..100 {
        assert_eq!(run("call_obj", None), "object");
    }
    assert_eq!(
        vm.heap_objects.len(),
        heap_base,
        "type_of object input stake leaked: heap {} -> {}",
        heap_base,
        vm.heap_objects.len()
    );
    // 每轮新动态字符串参（内联生成+消费，逐次独立段调用——与对象腿
    // 同形）：100 次后池回基线；预热驻留结果常量 "str"（非泄漏）。
    let _ = run("warm_str", None);
    vm.reap_all();
    let pool_base = vm.strings.read().unwrap().len();
    for i in 0..100 {
        assert_eq!(run("call_str_once", Some(i)), "str");
    }
    let pool_now = vm.strings.read().unwrap().len();
    // 恒定 +1 为一次性驻留常量（首调用内联 concat 形状的池驻留，非
    // 按调用累积）；修复前为每调用 +1（100 次 → +100，复审实证）。
    // 斜率型二次断言：再 100 次仍 ≤ base+1——按调用零泄漏。
    assert!(
        pool_now <= pool_base + 1,
        "type_of dynamic-string input stake leaked: pool {pool_base} -> {pool_now}"
    );
    for i in 100..200 {
        assert_eq!(run("call_str_once", Some(i)), "str");
    }
    let pool_final = vm.strings.read().unwrap().len();
    assert!(
        pool_final <= pool_base + 1,
        "type_of per-call pool leak: {pool_base} -> {pool_now} -> {pool_final} over 200 calls"
    );
}
