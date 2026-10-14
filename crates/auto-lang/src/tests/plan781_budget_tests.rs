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
