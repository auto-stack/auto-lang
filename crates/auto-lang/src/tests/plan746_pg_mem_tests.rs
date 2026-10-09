// PLAN-746 (PG-MEM-1): cooperative execution deadline + clean thread-panic join.
//
// 背景：2026-10-09 playground 全量示例走查实录——/api/run 无超时，挂起执行
// （死循环/print kwargs 形态）在服务端永不终止并持续吃内存（串行 1343 例
// 16MB→1434MB，8 并发批 20GB+）。本组测试钉死：
//  ① 带 deadline 的死循环确定返回 ExecutionTimeout 错误（而非挂死）；
//  ② 充裕 deadline 下正常程序行为不变；
//  ③ 执行线程内部 panic（parser unwrap 类）返回 Err 而非向上 panic
//    （SD-04：不再以 unwrap 噪音 500 冒泡到 axum）。

#[test]
fn plan746_deadline_terminates_infinite_loop() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    let start = std::time::Instant::now();
    let res = crate::run_with_capture_and_bytecode_with_deadline("for true {}", Some(deadline));
    let elapsed = start.elapsed();
    match res {
        Err(e) => assert!(
            e.to_string().contains("ExecutionTimeout"),
            "expected ExecutionTimeout error, got: {}",
            e
        ),
        Ok((r, out, _, _)) => panic!(
            "expected timeout error, got result={:?} stdout={:?}",
            r, out
        ),
    }
    assert!(
        elapsed < std::time::Duration::from_secs(10),
        "deadline not enforced promptly: {:?}",
        elapsed
    );
}

#[test]
fn plan746_deadline_terminates_allocating_loop() {
    // 直接建模 playground 事故形态：忙循环 + 字符串累加（无截止将无限
    // 分配内存——PG-MEM-1 实测 3MB/s 膨胀的同类）。
    let src = "var s = \"\"\nfor true {\n    s = s + \"x\"\n}\n";
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    let start = std::time::Instant::now();
    let res = crate::run_with_capture_and_bytecode_with_deadline(src, Some(deadline));
    match res {
        Err(e) => assert!(
            e.to_string().contains("ExecutionTimeout"),
            "expected ExecutionTimeout, got: {}",
            e
        ),
        Ok(_) => panic!("expected timeout error, got Ok"),
    }
    assert!(
        start.elapsed() < std::time::Duration::from_secs(10),
        "allocating-loop deadline not enforced promptly"
    );
}

#[test]
fn plan746_generous_deadline_normal_run_unchanged() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let (result, stdout, _, _) =
        crate::run_with_capture_and_bytecode_with_deadline("print(1 + 2)", Some(deadline))
            .expect("normal run with generous deadline must succeed");
    assert!(stdout.contains('3'), "stdout should contain 3, got: {stdout}");
    let _ = result;
}

#[test]
fn plan746_no_deadline_infinite_loop_is_opt_in() {
    // 不传 deadline（None）= 无预算：短程序正常完成（默认面零行为变化
    // 的表征——无限循环 None 面不可测试因真会挂，由上一组 opt-in 面覆盖）。
    let (result, stdout, _, _) = crate::run_with_capture_and_bytecode_with_meta("print(42)")
        .expect("default unlimited path must succeed");
    assert!(stdout.contains("42"), "stdout should contain 42, got: {stdout}");
    let _ = result;
}

#[test]
fn plan746_execution_thread_panic_returns_err() {
    // 走查实录 panic 源：parser 对畸形输入（孤立单引号 → UnterminatedChar
    // @ offset 0）内部 unwrap。修复前 join().unwrap() 把该 panic 冒泡到
    // 调用线程（axum 变裸 500）；修复后返回干净 Err。
    let res = crate::run_with_capture_and_bytecode_with_meta("'\n");
    assert!(res.is_err(), "hostile input must return Err, got Ok");
}
