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

// ============================================================================
// PLAN-705 T-03 探针：固定 async 客户端执行器的限额与退役面
// ============================================================================

/// 起一个一次性本地 HTTP server（受控响应/延迟），返回 (端口, join 句柄)。
fn one_shot_server(response: String, delay_ms: u64) -> (u16, std::thread::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
        let _ = std::io::Write::write_all(&mut stream, response.as_bytes());
    });
    (port, handle)
}

/// 等待 req_id 完成（真实 worker 写入），返回 take 结果。
fn wait_completed(req_id: u64) -> Option<Result<crate::vm::ffi::stdlib::AsyncResult, String>> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
        assert!(std::time::Instant::now() < deadline, "worker 完成超时");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    crate::vm::ffi::async_http::take_live_op(req_id)
}

/// T-03 (1) 队满=终结性错误：queue_capacity=1 时第二个 job 提交即以
/// `http client queue full` 完成终结（旧形态此处是每 job spawn 兜底线程，
/// 错误面根本不存在）。
/// 本探针要求独占进程的限额覆写——nextest 每测试进程隔离，勿在裸
/// cargo test 同进程混跑多个改限额的探针。
#[test]
fn plan705_client_queue_full_terminal_error_no_thread_fallback() {
    crate::vm::ffi::async_http::set_client_limits_for_test(
        crate::vm::ffi::async_http::ClientLimits {
            queue_capacity: 1,
            ..crate::vm::ffi::async_http::ClientLimits {
                workers: 2,
                max_active: 1,
                queue_capacity: 64,
                body_limit: 1024,
                total_timeout: std::time::Duration::from_secs(30),
            }
        },
    );
    // 慢上游占住唯一队列槽（active=1 也被它持有）。
    let (port, server) = one_shot_server(
        "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_string(),
        300,
    );
    let id1 = crate::vm::ffi::stdlib::alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(id1);
    assert!(
        crate::vm::ffi::stdlib::spawn_async_http(
            "GET".into(),
            format!("http://127.0.0.1:{port}/slow"),
            None,
            id1
        ),
        "第一个 job 应入队"
    );
    let id2 = crate::vm::ffi::stdlib::alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(id2);
    assert!(
        !crate::vm::ffi::stdlib::spawn_async_http(
            "GET".into(),
            format!("http://127.0.0.1:{port}/rejected"),
            None,
            id2
        ),
        "队满提交必须立即拒绝（无线程兜底）"
    );
    // 第二个 job 已终结性完成（既有 JSON 错误形态）。
    match wait_completed(id2) {
        Some(Err(msg)) => assert!(
            msg.contains("queue full"),
            "队满错误应可消费，得到 {msg}"
        ),
        other => panic!("队满应终结为 Err，得到 {:?}", other.is_some()),
    }
    server.join().expect("server thread");
}

/// T-03 (2) 响应体预算：超预算响应 → 终结性错误（增量读不强吞全量）。
#[test]
fn plan705_client_body_budget_enforced() {
    crate::vm::ffi::async_http::set_client_limits_for_test(
        crate::vm::ffi::async_http::ClientLimits {
            body_limit: 16,
            ..crate::vm::ffi::async_http::ClientLimits {
                workers: 2,
                max_active: 8,
                queue_capacity: 64,
                body_limit: 16,
                total_timeout: std::time::Duration::from_secs(30),
            }
        },
    );
    let big = "x".repeat(200);
    let (port, server) = one_shot_server(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            big.len(),
            big
        ),
        0,
    );
    let id = crate::vm::ffi::stdlib::alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(id);
    crate::vm::ffi::stdlib::spawn_async_http(
        "GET".into(),
        format!("http://127.0.0.1:{port}/big"),
        None,
        id,
    );
    match wait_completed(id) {
        Some(Err(msg)) => assert!(
            msg.contains("exceeds budget 16"),
            "超预算错误应可消费，得到 {msg}"
        ),
        other => panic!("超预算应终结为 Err，得到 {:?}", other.is_some()),
    }
    server.join().expect("server thread");
}

/// T-03 (3) 单 job 总期限：慢上游在期限后以终结性错误收口（不悬挂）。
#[test]
fn plan705_client_total_deadline_terminal_error() {
    crate::vm::ffi::async_http::set_client_limits_for_test(
        crate::vm::ffi::async_http::ClientLimits {
            total_timeout: std::time::Duration::from_millis(200),
            ..crate::vm::ffi::async_http::ClientLimits {
                workers: 2,
                max_active: 8,
                queue_capacity: 64,
                body_limit: 10 * 1024 * 1024,
                total_timeout: std::time::Duration::from_millis(200),
            }
        },
    );
    let (port, server) = one_shot_server(
        "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_string(),
        2_000,
    );
    let id = crate::vm::ffi::stdlib::alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(id);
    crate::vm::ffi::stdlib::spawn_async_http(
        "GET".into(),
        format!("http://127.0.0.1:{port}/slow"),
        None,
        id,
    );
    let started = std::time::Instant::now();
    match wait_completed(id) {
        Some(Err(msg)) => assert!(
            msg.contains("timed out"),
            "总期限错误应可消费，得到 {msg}"
        ),
        other => panic!("总期限应终结为 Err，得到 {:?}", other.is_some()),
    }
    assert!(
        started.elapsed() < std::time::Duration::from_millis(1_500),
        "期限收口不应等待上游完成"
    );
    server.join().expect("server thread");
}

/// T-03 (4) RequestBuilder `.send()` 在段模式下 park 而非同步 drain：
/// 首段立即返回 Parked{HttpRequest}（服务端延迟 400ms 下零忙等），
/// 恢复段消费句柄。旧形态此处是拦截内 30s 同步 drain（占住 owner）。
#[test]
fn plan705_builder_send_parks_in_segment_mode() {
    let body = r#"{"ok":true,"built":705}"#.to_string();
    let body_for_server = body.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        std::thread::sleep(std::time::Duration::from_millis(400));
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body_for_server.len(),
            body_for_server
        );
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });

    let code = r#"
fn grab() str {
    var b = http.request("GET", "http://127.0.0.1:7051/probe")
    var r = b.send()
    r.body()
}
"#
    .replace("7051", &port.to_string());
    let (vm, _stdout, _entry, _object_type) =
        crate::create_vm_from_source(&code).expect("compile");

    let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
    let started = std::time::Instant::now();
    let outcome = vm.call_fn_by_name_segment(&mut task, "grab", 0);
    let first_elapsed = started.elapsed();
    let (req_id, seg) = match &outcome {
        crate::vm::engine::SegmentOutcome::Parked {
            wait: crate::vm::engine::ParkedWait::HttpRequest(id),
            seg,
        } => (*id, seg.clone()),
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => panic!(
            "builder send 应 park 而非同步跑完——server 延迟 400ms 下跑完=同步 drain 回归"
        ),
        other => panic!("预期 Parked(HttpRequest)，得到 {:?}", other),
    };
    assert!(
        first_elapsed < std::time::Duration::from_millis(350),
        "首段返回耗时 {:?}——CALL_SPEC 段模式同步 drain 回归",
        first_elapsed
    );
    assert!(
        crate::vm::ffi::async_http::live_op_exists(req_id),
        "parked 后 live-op 不得回收"
    );

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
        assert!(std::time::Instant::now() < deadline, "server 应答超时");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    server.join().expect("server thread");

    match vm.resume_fn_by_name_segment(&mut task, &seg) {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("恢复段应 Completed(Ok)，得到 {:?}", other),
    }
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_string(nv), "body() 应返回字符串 nv");
    let got = vm
        .get_string(auto_val::decode_string(nv) as u32)
        .expect("string slot");
    assert_eq!(String::from_utf8_lossy(&got), body);
}

// ============================================================================
// PLAN-705 T-04 E2E：段驱动 owner dispatch（真实 server 面）
// ============================================================================

/// 启动 AutoVM HTTP server（固定端口 + 等待可连接；plan326 同款约束）。
fn start_plan705_server(code: &str, port: u16) {
    crate::vm::ffi::stdlib::clear_http_routes();
    std::env::set_var("AUTO_HTTP_PORT", port.to_string());
    let code = code.to_string();
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let _ = crate::run(&code);
        })
        .expect("spawn plan705 server thread");
    for _ in 0..50 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("plan705 server 未在 5s 内就绪 port={port}");
}

/// 原始 HTTP GET（阻塞读完整响应）。
fn http_get_raw(port: u16, path: &str) -> (u16, String) {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("connect");
    use std::io::{Read, Write};
    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).expect("write");
    stream.flush().expect("flush");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).ok();
    let status: u16 = resp
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body = resp
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default();
    (status, body)
}

/// T-04 E2E (1) 上游 gate + health 并存 + 双 park：两个 handler park 在
/// 受控上游上时，health 请求必须照常完成；解除后两个请求各自拿到自己
/// 的上游响应（AC-01 核心）。
#[test]
fn plan705_e2e_upstream_gate_health_two_parked() {
    const SERVER_PORT: u16 = 18501;
    const UPSTREAM_PORT: u16 = 18502;
    // 受控上游：两个连接各持一段 body，等 release 后才应答。
    let release = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let listener = std::net::TcpListener::bind(("127.0.0.1", UPSTREAM_PORT)).expect("bind upstream");
    let up = {
        let release = release.clone();
        std::thread::spawn(move || {
            for i in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept upstream");
                let mut buf = [0u8; 4096];
                let _ = std::io::Read::read(&mut stream, &mut buf);
                while !release.load(std::sync::atomic::Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                let body = format!("upstream-{}", i + 1);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
            }
        })
    };

    start_plan705_server(
        &r#"
#[api(method = "GET", path = "/api/gate1")]
fn gate1() str {
    Http.post_json("http://127.0.0.1:UPSTREAM/one", "q=1")
}
#[api(method = "GET", path = "/api/gate2")]
fn gate2() str {
    Http.post_json("http://127.0.0.1:UPSTREAM/two", "q=1")
}
#[api(method = "GET", path = "/api/health")]
fn health() str {
    "ok"
}
"#
        .replace("UPSTREAM", &UPSTREAM_PORT.to_string()),
        SERVER_PORT,
    );

    // 两个 gate 请求并发发出（客户端线程阻塞在响应读上）。
    let gate1 = std::thread::spawn(move || http_get_raw(SERVER_PORT, "/api/gate1"));
    std::thread::sleep(std::time::Duration::from_millis(150));
    let gate2 = std::thread::spawn(move || http_get_raw(SERVER_PORT, "/api/gate2"));
    std::thread::sleep(std::time::Duration::from_millis(150));

    // 上游未解除：health 必须已完成（park 中的请求不占 owner）。
    let health_started = std::time::Instant::now();
    let (hstatus, hbody) = http_get_raw(SERVER_PORT, "/api/health");
    assert!(
        health_started.elapsed() < std::time::Duration::from_secs(3),
        "health 在 gate park 期间被阻塞——owner 未交还"
    );
    assert_eq!((hstatus, hbody.as_str()), (200, "\"ok\""), "health 响应（str 经 JSON 编组带引号）");

    // 解除上游：两个 gate 请求各自拿到自己的 body。
    release.store(true, std::sync::atomic::Ordering::SeqCst);
    let (s1, b1) = gate1.join().expect("gate1 client");
    let (s2, b2) = gate2.join().expect("gate2 client");
    up.join().expect("upstream thread");
    assert_eq!((s1, b1.as_str()), (200, "\"upstream-1\""), "gate1 最终值（body 串 JSON 编组带引号）");
    assert_eq!((s2, b2.as_str()), (200, "\"upstream-2\""), "gate2 最终值（body 串 JSON 编组带引号）");
}

/// T-04 E2E (2) `~T` 最终值 + 普通 int 反例：声明 ~T 的 handler 返回
/// External future → 服务端等最终 T；未声明 ~T 的 handler 返回 240
/// （低字节恰为 0xF0 的位形态）→ 必须按普通 int 编组，绝不被猜成
/// future（AC-01 元数据门反例）。
#[test]
fn plan705_e2e_async_return_final_value_and_int_counterexample() {
    const SERVER_PORT: u16 = 18503;
    start_plan705_server(
        r#"
#[api(method = "GET", path = "/api/async-later")]
fn async_later() ~int {
    delay_async(150)
}
#[api(method = "GET", path = "/api/plain-int")]
fn plain_int() int {
    return 240
}
"#,
        SERVER_PORT,
    );
    let (s1, b1) = http_get_raw(SERVER_PORT, "/api/async-later");
    assert_eq!(s1, 200, "async 返回应 200");
    assert_eq!(b1, "150", "~T 应解析为最终 T（150ms），非 future 位模式: {b1}");
    let (s2, b2) = http_get_raw(SERVER_PORT, "/api/plain-int");
    assert_eq!(s2, 200);
    assert_eq!(b2, "240", "普通 int 240 不得被猜成 future: {b2}");
}

/// T-04 E2E (3) 链式 await + 错误恢复：handler 内两次顺序上游调用
/// （两次 park），随后 0 除失败被 .at catch 截获——全程段恢复语义正确。
#[test]
fn plan705_e2e_chained_await_and_error_recovery() {
    const SERVER_PORT: u16 = 18504;
    const UPSTREAM_PORT: u16 = 18505;
    let listener = std::net::TcpListener::bind(("127.0.0.1", UPSTREAM_PORT)).expect("bind upstream");
    let up = std::thread::spawn(move || {
        // 四个请求：plain 链 two + catch 链 two；catch 链 first 的
        // len 为偶数 → d=0 → 除零进 catch。
        for body in ["aaaa", "second-body", "aaaa", "second-body"] {
            let (mut stream, _) = listener.accept().expect("accept upstream");
            let mut buf = [0u8; 4096];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            std::thread::sleep(std::time::Duration::from_millis(80));
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
        }
    });
    start_plan705_server(
        &r#"
fn fetch_len_factor() int {
    var r1 = Http.post_json("http://127.0.0.1:UP/first", "q=1")
    var r2 = Http.post_json("http://127.0.0.1:UP/second", "q=1")
    var d = r1.len() % 2
    var boom = 10 / d
    r2
}
fn guarded() str {
    try {
        fetch_len_factor()
    } catch (e) {
        "caught"
    }
}
#[api(method = "GET", path = "/api/chain")]
fn chain() str {
    guarded()
}
#[api(method = "GET", path = "/api/chain-plain")]
fn chain_plain() str {
    var r1 = Http.post_json("http://127.0.0.1:UP/first", "q=1")
    var r2 = Http.post_json("http://127.0.0.1:UP/second", "q=1")
    r2
}
"#
        .replace("UP", &UPSTREAM_PORT.to_string()),
        SERVER_PORT,
    );
    // 先探纯链式双 park（无 catch），再探 catch 恢复。
    // 上游序列：chain-plain 消费 first+second，chain 消费 first+second。
    let (sp, bp) = http_get_raw(SERVER_PORT, "/api/chain-plain");
    eprintln!("[probe] chain-plain → {sp} {bp:?}");
    let (status, body) = http_get_raw(SERVER_PORT, "/api/chain");
    up.join().expect("upstream");
    assert_eq!(sp, 200, "纯链式双 park");
    assert!(bp.contains("second-body"), "纯链式返回第二个 body: {bp}");
    assert_eq!(status, 200);
    assert_eq!(body, "\"caught\"", "链式 await + catch 恢复: {body}");
}

/// PLAN-705 T-04 回归：深帧 try 恢复——try 在外层 fn、park 在被调 fn、
/// 错误（10/0）在 park 恢复后的被调帧内。拦截必须跨帧展开调用栈账
/// （intercept_error 的 PLAN-705 修复点），catch 生效返回 "caught"。
#[test]
fn plan705_engine_deep_frame_try_recovery() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let body_for_server = "aaaa".to_string();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body_for_server.len(),
            body_for_server
        );
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });
    let code = r#"
fn fetch_len_factor() int {
    var r1 = Http.post_json("http://127.0.0.1:P1/first", "q=1")
    var d = r1.len() % 2
    var boom = 10 / d
    r1
}
fn guarded() str {
    try {
        fetch_len_factor()
    } catch (e) {
        "caught"
    }
}
"#
    .replace("P1", &port.to_string());
    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(&code).expect("compile");
    let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
    let outcome = vm.call_fn_by_name_segment(&mut task, "guarded", 0);
    let (req_id, seg) = match &outcome {
        crate::vm::engine::SegmentOutcome::Parked {
            wait: crate::vm::engine::ParkedWait::HttpRequest(id),
            seg,
        } => (*id, seg.clone()),
        other => panic!("预期 park，得到 {:?}", other),
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
        assert!(std::time::Instant::now() < deadline, "server 应答超时");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    server.join().expect("server thread");
    let resumed = vm.resume_fn_by_name_segment(&mut task, &seg);
    match resumed {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {
            let nv = task.ram.pop_nv();
            let got = vm.get_string(auto_val::decode_string(nv) as u32).expect("str");
            eprintln!("[probe] engine two-frame resume → {:?}", String::from_utf8_lossy(&got));
            assert_eq!(String::from_utf8_lossy(&got), "caught", "深帧 try 恢复");
        }
        other => panic!("resume 应 Completed(Ok)，得到 {:?}", other),
    }
}
