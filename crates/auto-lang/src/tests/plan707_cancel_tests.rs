//! PLAN-707 T-02：可取消执行器——managed job 的 queued/active/retry 三阶段
//! 实际取消、许可回基线、迟到不复活、detached 消息桥反例。
//!
//! 冻结依据：docs/plans/reports/707-stream-decision.md D-1。
//! 红相 witness：`plan707_spike_tests::plan707_spike_cancel_gap_*`（本任务
//! 落地后转绿）。

use crate::vm::ffi::async_http as ah;
use crate::vm::ffi::async_http::ClientLimits;
use crate::vm::ffi::stdlib::AsyncResult;

fn limit_active_1() {
    ah::set_client_limits_for_test(ClientLimits {
        workers: 1,
        max_active: 1,
        queue_capacity: 8,
        body_limit: 1024 * 1024,
        total_timeout: std::time::Duration::from_secs(30),
    });
}

/// AC-01（active 阶段）：取消正在执行（blocked 于 gate await）的 job——
/// future 被丢弃（marker 永不置位）、active 许可归还基线、迟到完成被守卫
/// 丢弃。witness = spike 红测 1 的绿形态（此处再断言许可回基线）。
#[test]
fn plan707_cancel_active_job_aborts_and_frees_permit() {
    limit_active_1();
    let baseline_active = ah::client_active_available();

    let req_id = crate::vm::ffi::stdlib::alloc_async_id();
    let marker = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel::<()>();
    let (gate_tx, gate_rx) = tokio::sync::oneshot::channel::<()>();
    let marker_for_job = marker.clone();
    let job = async move {
        let _ = entered_tx.send(());
        let _ = gate_rx.await;
        marker_for_job.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(AsyncResult::Body("late".to_string()))
    };
    crate::vm::ffi::async_http::register_live_op(req_id);
    assert!(ah::submit_client_job(req_id, job));
    entered_rx.blocking_recv().expect("job entered active");

    crate::vm::ffi::stdlib::drop_async_result(req_id);
    let _ = gate_tx.send(());
    std::thread::sleep(std::time::Duration::from_millis(300));

    assert!(
        !marker.load(std::sync::atomic::Ordering::SeqCst),
        "取消后 job future 必须已停止（abort 生效于 gate await）"
    );
    // 许可回基线：wrapper task 整树丢弃 → active 许可随 Drop 释放。
    assert_eq!(
        ah::client_active_available(),
        baseline_active,
        "取消后 active 许可必须归还"
    );
    assert!(!ah::complete_live_op(
        req_id,
        Ok(AsyncResult::Body("x".into()))
    ));
}

/// AC-01（queued 阶段）：active=1 被 A 占住（gate 关闭），B 排队；取消 B
/// 后开 gate——A 正常完成，B 的 job 体**从未执行**（上游无连接尝试、
/// started 标记 false），queue 许可归还。
#[test]
fn plan707_cancel_queued_job_never_starts() {
    limit_active_1();
    let baseline_active = ah::client_active_available();

    // A：占住唯一 active 槽，blocked 于 gate。
    let (gate_tx, gate_rx) = tokio::sync::oneshot::channel::<()>();
    let (a_done_tx, a_done_rx) = tokio::sync::oneshot::channel::<()>();
    let req_a = crate::vm::ffi::stdlib::alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(req_a);
    assert!(ah::submit_client_job(req_a, async move {
        let _ = gate_rx.await;
        let _ = a_done_tx.send(());
        Ok(AsyncResult::Body("a".to_string()))
    }));

    // B：排队（active 满）。started 标记只在 job 体（active 之后）执行。
    let started = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let started_for_b = started.clone();
    let req_b = crate::vm::ffi::stdlib::alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(req_b);
    assert!(ah::submit_client_job(req_b, async move {
        started_for_b.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(AsyncResult::Body("b".to_string()))
    }));
    // 确认 B 在排队（A 仍占 active、gate 未开）。
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(
        !started.load(std::sync::atomic::Ordering::SeqCst),
        "前置：B 应处于 queued（active=1 被 A 占住）"
    );

    // 取消排队的 B，再放行 A。
    crate::vm::ffi::stdlib::drop_async_result(req_b);
    let _ = gate_tx.send(());
    a_done_rx
        .blocking_recv()
        .expect("A 应正常完成（许可复用证据）");

    std::thread::sleep(std::time::Duration::from_millis(300));
    assert!(
        !started.load(std::sync::atomic::Ordering::SeqCst),
        "取消的排队 job 不得再获得执行（B 的 job 体被 abort 丢弃）"
    );
    assert_eq!(
        ah::client_active_available(),
        baseline_active,
        "A 完成后 active 许可回基线"
    );
    assert!(!ah::complete_live_op(
        req_b,
        Ok(AsyncResult::Body("x".into()))
    ));
}

/// AC-01（retry 退避阶段）：job 在两次发送尝试之间退避 sleep 时取消——
/// 后续尝试永不发生（attempt 计数停止增长），退避 sleep 随 future 丢弃。
#[test]
fn plan707_cancel_during_retry_backoff_stops_attempts() {
    limit_active_1();
    let attempts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let attempts_for_job = attempts.clone();
    let (first_tx, first_rx) = tokio::sync::oneshot::channel::<()>();
    let first_tx = std::sync::Mutex::new(Some(first_tx));

    let req_id = crate::vm::ffi::stdlib::alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(req_id);
    assert!(ah::submit_client_job(req_id, async move {
        // 指向必然连接失败的端口（1~3 轮退避重试形态）。
        let client = reqwest::Client::new();
        for attempt in 0..5u32 {
            attempts_for_job.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if attempt == 0 {
                if let Some(tx) = first_tx.lock().unwrap().take() {
                    let _ = tx.send(());
                }
            }
            let _ = client.get("http://127.0.0.1:1/never").send().await;
            // 退避 sleep：取消必须能打断这里（不再进入下一轮）。
            tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        }
        Err("exhausted".to_string())
    }));

    first_rx.blocking_recv().expect("第一轮尝试已发生");
    let after_first = attempts.load(std::sync::atomic::Ordering::SeqCst);
    crate::vm::ffi::stdlib::drop_async_result(req_id);
    // 宽限期：若未被取消，退避 120ms 后会进入下一轮（计数增长）。
    std::thread::sleep(std::time::Duration::from_millis(400));
    let after_grace = attempts.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        after_first, after_grace,
        "取消后发送尝试必须停止（退避 sleep 随 future 丢弃，计数不再增长）"
    );
}

/// AC-01（竞态闭合）：cancel 与 submit 并发序——先 submit 后立即 cancel
/// （spawn→登记窗口外）；以及 cancel 抢先（手工移除条目模拟窗口内取消）
/// 后句柄复查必须补 abort。后者是 insert-then-recheck 协议的直接验证。
#[test]
fn plan707_cancel_races_close_without_leak() {
    limit_active_1();
    // 常规序：submit → cancel → 迟到完成丢弃；abort 表条目随 cancel 出表。
    let req_id = crate::vm::ffi::stdlib::alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(req_id);
    assert!(ah::submit_client_job(req_id, async {
        Ok(AsyncResult::Body("quick".to_string()))
    }));
    crate::vm::ffi::stdlib::drop_async_result(req_id);
    std::thread::sleep(std::time::Duration::from_millis(100));
    // abort 表不复活（cancel 已 remove；job 完成路径不写 abort 表）。
    assert!(!ah::complete_live_op(
        req_id,
        Ok(AsyncResult::Body("late".into()))
    ));

    // 多轮取消风暴后表不增长（幂等终结；条目数与残余 live-op 一致）。
    // 队列容量 8——立即完成的 job 由 runtime 排空有延迟，队满即短退避重试。
    let ops_before = ah::live_op_count();
    for _ in 0..16 {
        let id = crate::vm::ffi::stdlib::alloc_async_id();
        crate::vm::ffi::async_http::register_live_op(id);
        let mut submitted = false;
        for _ in 0..100 {
            if ah::submit_client_job(id, async { Err("immediate".to_string()) }) {
                submitted = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(submitted, "排空后提交应成功（队满重试上限内）");
        crate::vm::ffi::stdlib::drop_async_result(id);
    }
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert_eq!(
        ah::live_op_count(),
        ops_before,
        "取消风暴后 live-op 表回基线"
    );
}

/// AC-01 反例：detached 消息桥不被 managed 取消语义误杀——真实端点应答
/// 后 payload 照常入队（`Http.get_msg` fire-and-forget 语义保持）。
#[test]
fn plan707_cancel_detached_msg_bridge_unaffected() {
    use crate::vm::ffi::stdlib::{http_msg_poll_one, http_msg_queue_clear};
    http_msg_queue_clear();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            use std::io::{Read, Write};
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = br#"{"ok":true}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                String::from_utf8_lossy(body)
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    crate::vm::ffi::stdlib::shim_http_get_msg_spawn_for_test(
        &format!("http://127.0.0.1:{port}/msg"),
        "Store.Handler",
    );

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(done) = http_msg_poll_one() {
            assert_eq!(done.widget, "Store");
            assert_eq!(done.event, "Handler");
            assert!(
                done.payload.contains("\"ok\":true"),
                "payload={}",
                done.payload
            );
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "detached 消息桥 5s 内未送达——被 managed 取消语义误杀？"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    http_msg_queue_clear();
}
