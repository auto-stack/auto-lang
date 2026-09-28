//! PLAN-705 T-01 spike：服务端 HTTP handler 异步生命周期的三个前提探针。
//!
//! (1) **notify 驱动的 owner loop**：真 VM 段 park 后，owner 侧用
//!     `tokio::sync::Notify` 的 enable→检查→await 三段式等待完成通知——
//!     零固定间隔轮询、零丢唤醒（T-04 owner loop 的骨架预演）。
//! (2) **取消 + 迟到完成单次终结**：cancel（drop）之后的 worker 迟到
//!     完成写必须被 presence-guard 吸收（T-02 统一登记表协议的种子）。
//!
//! 生产语义中完成端 notify 由 T-02 挂进统一完成写点；本 spike 的
//! 观察者线程只是测试脚手架（等待 shim worker 落槽后替它发通知），
//! owner 侧的 enable→check→await 模式则是将来 owner loop 的最终形态。

use crate::vm::engine::{ParkedWait, SegmentOutcome};
use crate::vm::task::AutoTask;

/// (1) notify 驱动 owner loop：真 VM 段 park → 完成通知唤醒 → 恢复取值。
/// select 只有三臂：完成通知 / 兜底超时（30s，通知丢失即红）/ 永不触发的
/// 关闭臂占位。断言从"结果就绪"到"恢复完成"的延迟远小于兜底超时——
/// 证明唤醒走的是通知臂而非轮询。
#[tokio::test]
async fn plan705_spike_notify_owner_loop_parks_resumes_without_polling() {
    let body = r#"{"ok":true,"spike":705}"#.to_string();
    let body_for_server = body.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        std::thread::sleep(std::time::Duration::from_millis(200));
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body_for_server.len(),
            body_for_server
        );
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });

    let code = r#"
fn pick_data() str {
    var resp = Http.post_json("http://127.0.0.1:7050/seed", "q=1")
    resp
}
"#
    .replace("7050", &port.to_string());
    let (vm, _stdout, _entry, _object_type) =
        crate::create_vm_from_source(&code).expect("compile");

    let mut task = AutoTask::new(0, 65536, 0);
    let outcome = vm.call_fn_by_name_segment(&mut task, "pick_data", 0);
    let (req_id, seg) = match &outcome {
        SegmentOutcome::Parked {
            wait: ParkedWait::HttpRequest(id),
            seg,
        } => (*id, seg.clone()),
        other => panic!("预期 Parked(HttpRequest)，得到 {:?}", other),
    };
    let parked_at = std::time::Instant::now();

    // 完成端替身（T-02 起 notify 挂进统一完成写点）：等 shim worker 把
    // 结果落槽后发 notify_waiters。观察线程自身允许等待——它不是 owner。
    let notify = std::sync::Arc::new(tokio::sync::Notify::new());
    let notify_for_worker = notify.clone();
    let observer = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
            assert!(std::time::Instant::now() < deadline, "server 应答超时");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        notify_for_worker.notify_waiters();
    });

    // owner loop 骨架：enable → 检查 → await，零固定间隔轮询。
    let resumed = tokio::select! {
        _ = async {
            loop {
                // (a) 先注册兴趣：Notified 首次 poll 才登记 waiter，
                // enable() 把登记提前到检查之前——消除丢唤醒窗口。
                let notified = notify.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                // (b) 检查就绪（覆盖"先完成再 park"与"enable 前完成"两态）。
                if crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
                    break;
                }
                // (c) 等通知：enable 之后的任意完成必然唤醒本 waiter。
                notified.await;
            }
        } => true,
        _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => false,
    };
    let woke_after = parked_at.elapsed();
    observer.join().expect("observer thread");
    server.join().expect("server thread");
    assert!(resumed, "完成通知丢失——owner loop 落入 30s 兜底超时（红）");
    assert!(
        woke_after < std::time::Duration::from_secs(5),
        "通知唤醒耗时 {:?}，超出 server 延迟 + 余量——唤醒路径可疑",
        woke_after
    );

    match vm.resume_fn_by_name_segment(&mut task, &seg) {
        SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("恢复段应 Completed(Ok)，得到 {:?}", other),
    }
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_string(nv), "post_json 返回应为字符串 nv");
    let got = vm
        .get_string(auto_val::decode_string(nv) as u32)
        .expect("string slot");
    assert_eq!(String::from_utf8_lossy(&got), body);
}

/// (2) 取消 + 迟到完成单次终结协议（T-02 统一 live-op 登记表实装验证）：
/// register → cancel(drop) → 迟到 complete 被 presence 守卫丢弃 → 表回
/// 基线、复活零发生；complete-before-take 立即可消费；重复 cancel 幂等；
/// Pending 态 take 不删除 live 令牌（探测/重入不丢条目）。
#[test]
fn plan705_spike_cancel_and_late_completion_single_finalization() {
    use crate::vm::ffi::async_http as reg;
    use crate::vm::ffi::stdlib::AsyncResult;

    let count_before = reg::live_op_count();

    // -- register → Pending。
    let req_id = crate::vm::ffi::stdlib::alloc_async_id();
    reg::register_live_op(req_id);
    assert!(!crate::vm::ffi::stdlib::async_http_result_ready(req_id));
    assert!(reg::live_op_exists(req_id));

    // -- Pending 态 take：不得删除 live 令牌（探测/重入不丢条目）。
    assert!(reg::take_live_op(req_id).is_none());
    assert!(reg::live_op_exists(req_id), "Pending take 不得消费令牌");

    // -- cancel-first：等待方放弃（引擎超时臂/scope 取消的终结入口）。
    crate::vm::ffi::stdlib::drop_async_result(req_id);
    assert!(!reg::live_op_exists(req_id));

    // -- 迟到完成（worker 慢于取消）：presence 守卫丢弃，禁止复活。
    assert!(
        !reg::complete_live_op(req_id, Ok(AsyncResult::Body("late".to_string()))),
        "迟到完成必须被丢弃（返回 false）"
    );
    assert!(
        !reg::live_op_exists(req_id),
        "迟到完成复活了已取消条目——单次终结被破坏"
    );

    // -- complete-before-take：完成先于消费 → 就绪立即可见，一次消费终结。
    let req_id2 = crate::vm::ffi::stdlib::alloc_async_id();
    reg::register_live_op(req_id2);
    assert!(reg::complete_live_op(
        req_id2,
        Ok(AsyncResult::Body("early".to_string()))
    ));
    assert!(crate::vm::ffi::stdlib::async_http_result_ready(req_id2));
    assert!(reg::take_live_op(req_id2).is_some(), "Completed 必须可消费");
    assert!(!reg::live_op_exists(req_id2), "take 后条目终结移除");
    assert!(
        !reg::complete_live_op(req_id2, Ok(AsyncResult::Body("again".to_string()))),
        "消费后的重复完成必须被丢弃"
    );

    // -- 重复完成（未取消）：单次终结——首胜，次弃。
    let req_id3 = crate::vm::ffi::stdlib::alloc_async_id();
    reg::register_live_op(req_id3);
    assert!(reg::complete_live_op(req_id3, Ok(AsyncResult::Body("first".to_string()))));
    assert!(!reg::complete_live_op(req_id3, Ok(AsyncResult::Body("second".to_string()))));
    match reg::take_live_op(req_id3) {
        Some(Ok(AsyncResult::Body(s))) => assert_eq!(s, "first", "首完成胜出"),
        Some(Ok(_)) => panic!("首完成应胜出，得到非 Body 变体"),
        Some(Err(e)) => panic!("首完成应胜出，得到 Err: {e}"),
        None => panic!("首完成应可消费，得到 None"),
    }
    crate::vm::ffi::stdlib::drop_async_result(req_id3); // 幂等（已移除）

    assert_eq!(reg::live_op_count(), count_before, "全部终结后登记表必须回基线");
}
