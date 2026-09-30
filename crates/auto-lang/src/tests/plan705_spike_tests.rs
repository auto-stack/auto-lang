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

/// OS 分配临时端口（fix-test-tiering：固定端口在并行 worktree/多 agent 下
/// bind 互踩或跨 run 连错 mock；Windows 临时端口顺序分配，刚释放的端口
/// 不会被立即重派，TOCTOU 窗口可忽略）。
fn ephemeral_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind :0")
        .local_addr()
        .expect("local_addr")
        .port()
}

/// 启动 AutoVM HTTP server（临时端口 + 等待可连接；plan326 同款约束——
/// 端口经 AUTO_HTTP_PORT env 传入，调用方用 ephemeral_port() 预分配）。
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
    let server_port = ephemeral_port();
    // 受控上游：两个连接各持一段 body，等 release 后才应答。
    let release = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind upstream");
    let upstream_port = listener.local_addr().unwrap().port();
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
        .replace("UPSTREAM", &upstream_port.to_string()),
        server_port,
    );

    // 两个 gate 请求并发发出（客户端线程阻塞在响应读上）。
    let gate1 = std::thread::spawn(move || http_get_raw(server_port, "/api/gate1"));
    std::thread::sleep(std::time::Duration::from_millis(150));
    let gate2 = std::thread::spawn(move || http_get_raw(server_port, "/api/gate2"));
    std::thread::sleep(std::time::Duration::from_millis(150));

    // 上游未解除：health 必须已完成（park 中的请求不占 owner）。
    let health_started = std::time::Instant::now();
    let (hstatus, hbody) = http_get_raw(server_port, "/api/health");
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
    let server_port = ephemeral_port();
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
        server_port,
    );
    let (s1, b1) = http_get_raw(server_port, "/api/async-later");
    assert_eq!(s1, 200, "async 返回应 200");
    assert_eq!(b1, "150", "~T 应解析为最终 T（150ms），非 future 位模式: {b1}");
    let (s2, b2) = http_get_raw(server_port, "/api/plain-int");
    assert_eq!(s2, 200);
    assert_eq!(b2, "240", "普通 int 240 不得被猜成 future: {b2}");
}

/// T-04 E2E (3) 链式 await + 错误恢复：handler 内两次顺序上游调用
/// （两次 park），随后 0 除失败被 .at catch 截获——全程段恢复语义正确。
#[test]
fn plan705_e2e_chained_await_and_error_recovery() {
    let server_port = ephemeral_port();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind upstream");
    let upstream_port = listener.local_addr().unwrap().port();
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
        .replace("UP", &upstream_port.to_string()),
        server_port,
    );
    // 先探纯链式双 park（无 catch），再探 catch 恢复。
    // 上游序列：chain-plain 消费 first+second，chain 消费 first+second。
    let (sp, bp) = http_get_raw(server_port, "/api/chain-plain");
    eprintln!("[probe] chain-plain → {sp} {bp:?}");
    let (status, body) = http_get_raw(server_port, "/api/chain");
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

// ============================================================================
// PLAN-705 T-05 E2E：scope 取消 / deadline / 关闭 / 半关闭 / 副作用边界
// ============================================================================

/// T-05 (1) deadline 取消 parked 等待并回收资源：上游慢响应超回复
/// deadline → 桥 503；owner 侧 parked 等待废弃、live-op 回收（迟到完成
/// 被 presence 守卫丢弃）、scope/许可回基线（AC-04）。
#[test]
fn plan705_e2e_deadline_cancels_parked_and_reclaims() {
    let server_port = ephemeral_port();
    std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "400");
    let op_baseline = crate::vm::ffi::async_http::live_op_count();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind upstream");
    let upstream_port = listener.local_addr().unwrap().port();
    let up = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept upstream");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        std::thread::sleep(std::time::Duration::from_millis(2_000));
        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });
    start_plan705_server(
        &r#"
#[api(method = "GET", path = "/api/slow")]
fn slow() str {
    Http.post_json("http://127.0.0.1:UP/slow", "q=1")
}
"#
        .replace("UP", &upstream_port.to_string()),
        server_port,
    );
    let started = std::time::Instant::now();
    let (status, body) = http_get_raw(server_port, "/api/slow");
    let elapsed = started.elapsed();
    up.join().expect("upstream");
    assert_eq!(status, 503, "超 deadline 应 503");
    assert!(
        elapsed >= std::time::Duration::from_millis(350)
            && elapsed < std::time::Duration::from_millis(2_000),
        "deadline 应在 ~400ms 收口（非等上游 2s），实际 {:?}",
        elapsed
    );
    assert!(body.contains("error"), "503 带 error 形态: {body}");
    // owner 侧 parked 等待已废弃（上游 2s 应答到达时无人等待——迟到完成
    // 被丢弃）：live-op 表回基线、scope 表空、许可全量可用。
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert_eq!(
        crate::vm::ffi::async_http::live_op_count(),
        op_baseline,
        "live-op 未回基线——parked 等待未回收"
    );
    assert_eq!(crate::vm::ffi::http_server::live_scope_count(), 0, "scope 未回基线");
}

/// T-05 (2) 失效队列不执行：scope 已取消的排队请求在 owner 出队时被
/// 跳过——handler 任务零派发（vm.tasks 零增长）、503 终结（确定性单元
/// 形态：真 VM + 真路由表 + 已取消 scope，出队臂 = dispatch_owner_request）。
#[test]
fn plan705_e2e_invalid_scope_skips_handler_dispatch() {
    let code = r#"
#[api(method = "GET", path = "/api/touch")]
fn touch() str {
    "touched"
}
"#;
    let (vm, _out, _entry, _t) = crate::create_vm_from_source(code).expect("compile");
    let routes = crate::vm::ffi::http_server::get_routes();
    assert!(!routes.is_empty(), "路由应已注册");
    let tasks_before = vm.tasks.len();
    let (tx, rx) = tokio::sync::oneshot::channel::<crate::vm::ffi::http_server::ApiReply>();
    let scope = crate::vm::ffi::http_server::create_scope(
        999,
        std::time::Instant::now() + std::time::Duration::from_secs(30),
    )
    .expect("scope");
    // 取消发生在出队前（桥超时/连接终结/关闭皆可到达此态）。
    assert!(crate::vm::ffi::http_server::cancel_scope(&scope));
    assert!(!crate::vm::ffi::http_server::scope_usable(&scope));
    let mut parked = Vec::new();
    let vm = std::rc::Rc::new(vm);
    crate::vm::ffi::http_server::dispatch_owner_request(
        &vm,
        &routes,
        crate::vm::ffi::http_server::test_api_request_get("/api/touch"),
        tx,
        scope.id,
        &mut parked,
    );
    let reply = rx.blocking_recv().expect("503 reply");
    match reply {
        crate::vm::ffi::http_server::ApiReply::Full { status, .. } => {
            assert_eq!(status, 503, "失效 scope 应 503");
        }
        _ => panic!("应 Full 503"),
    }
    assert_eq!(vm.tasks.len(), tasks_before, "失效请求不得派发 handler 任务");
    assert!(parked.is_empty(), "失效请求不得挂表");
    assert_eq!(crate::vm::ffi::http_server::live_scope_count(), 0, "取消后 scope 回基线");
}

/// T-05 (3) 关闭排空：优雅关停取消 parked 等待（503 + 资源回收），许可
/// /scope 回基线；await 前副作用保留（非回滚契约——hits 计数不撤销）。
#[test]
fn plan705_e2e_shutdown_cancels_parked_side_effect_kept() {
    // 排水窗 500ms < 上游 2.5s：parked 请求在排水截止后被取消（503），
    // 而非排水期内自然完成（200）。
    std::env::set_var("AUTO_HTTP_SHUTDOWN_DRAIN_MS", "500");
    std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "30000");
    let server_port = ephemeral_port();
    let op_baseline = crate::vm::ffi::async_http::live_op_count();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind upstream");
    let upstream_port = listener.local_addr().unwrap().port();
    let up = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept upstream");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        std::thread::sleep(std::time::Duration::from_millis(2_500));
        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });
    start_plan705_server(
        &r#"
var hits = 0

#[api(method = "GET", path = "/api/side")]
fn side() str {
    hits = hits + 1
    Http.post_json("http://127.0.0.1:UP/slow", "q=1")
}
#[api(method = "GET", path = "/api/hits")]
fn hits_fn() int {
    hits
}
"#
        .replace("UP", &upstream_port.to_string()),
        server_port,
    );
    // 客户端线程阻塞等待 /api/side（服务端 park 中）。
    let client = std::thread::spawn(move || http_get_raw(server_port, "/api/side"));
    std::thread::sleep(std::time::Duration::from_millis(300));
    // 副作用已在 await 前提交。
    let (_, hits_now) = http_get_raw(server_port, "/api/hits");
    assert_eq!(hits_now, "1", "await 前副作用应可见: {hits_now}");
    // 优雅关停：parked 请求废弃、503、资源回收。
    assert!(crate::vm::ffi::http_server::test_trigger_shutdown(), "关停触发");
    let (status, body) = client.join().expect("client");
    eprintln!("[probe] side after shutdown → {status} {body:?}");
    // 关停的客户端可见形态：503 终态或连接终结（net 侧排水力关与 503
    // 下发竞速，两者同义=请求未完成、等待已废弃）；资源回收由下方
    // 基线断言承载。
    assert!(
        status == 503 || status == 0,
        "关停应使 parked 请求 503 或连接终结，得到 {status} {body:?}"
    );
    up.join().expect("upstream");
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert_eq!(
        crate::vm::ffi::async_http::live_op_count(),
        op_baseline,
        "live-op 未回基线"
    );
    assert_eq!(crate::vm::ffi::http_server::live_scope_count(), 0, "scope 未回基线");
}

/// T-05 (4) 半关闭：请求写端关闭（shutdown(Write)）但读端保留——服务端
/// 仍正常响应（半关闭不判取消；AC-04 断连判据边界）。
#[test]
fn plan705_e2e_half_close_still_responds() {
    let server_port = ephemeral_port();
    start_plan705_server(
        r#"
#[api(method = "GET", path = "/api/ping")]
fn ping() int {
    42
}
"#,
        server_port,
    );
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", server_port)).expect("connect");
    let req = "GET /api/ping HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(req.as_bytes()).expect("write");
    stream.flush().expect("flush");
    // 半关闭写端：服务端不得据此判取消。
    stream.shutdown(std::net::Shutdown::Write).expect("half close");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");
    assert!(resp.starts_with("HTTP/1.1 200"), "半关闭后应正常响应: {resp:?}");
    assert!(resp.contains("42"), "半关闭响应体: {resp:?}");
}

// ============================================================================
// PLAN-705 T-06：兼容探针（middleware park / __axum: closure 段）
// ============================================================================

/// T-06 (1) middleware park：middleware 内异步 HTTP 等待 → 请求在
/// middleware 阶段挂起，恢复后链继续到 handler（短路与续跑两臂）。
#[test]
fn plan705_e2e_middleware_park_then_chain() {
    let server_port = ephemeral_port();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind upstream");
    let upstream_port = listener.local_addr().unwrap().port();
    let up = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept upstream");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        std::thread::sleep(std::time::Duration::from_millis(250));
        let body = "flag";
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });
    start_plan705_server(
        &r#"
fn gate_mw(info) {
    var verdict = Http.post_json("http://127.0.0.1:UP/gate", "q=1")
    if verdict == "flag" {
        return null
    }
    "blocked"
}
#[api(method = "GET", path = "/api/after-mw")]
fn after_mw() str {
    "reached"
}
"#
        .replace("UP", &upstream_port.to_string()),
        server_port,
    );
    // 注册 middleware（须在 server 编译后、请求前——MIDDLEWARE_CHAIN 全局）。
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", server_port)).expect("connect");
    // middleware 经 main 面注册不可行（run() 已进 server），改由诊断：
    // 直接驱动 MIDDLEWARE_CHAIN 等价于 http.server.middleware 调用效果。
    drop(stream);
    crate::vm::ffi::stdlib::MIDDLEWARE_CHAIN.lock().unwrap().clear();
    let code_reg = r#"
fn gate_mw(info) {
    var verdict = Http.post_json("http://127.0.0.1:UPREG/gate", "q=1")
    if verdict == "flag" {
        return null
    }
    "blocked"
}
"#
    .replace("UPREG", &upstream_port.to_string());
    let (vm_reg, _o, _e, _t) = crate::create_vm_from_source(&code_reg).expect("compile mw");
    let _ = vm_reg;
    crate::vm::ffi::stdlib::MIDDLEWARE_CHAIN
        .lock()
        .unwrap()
        .push("gate_mw".to_string());
    // middleware park 期间 handler 不会被调用；上游解除后链继续。
    let started = std::time::Instant::now();
    let (status, body) = http_get_raw(server_port, "/api/after-mw");
    up.join().expect("upstream");
    assert_eq!(status, 200, "middleware 续跑后应 200: {body:?}");
    assert_eq!(body, "\"reached\"", "handler 应在 middleware 恢复后到达: {body:?}");
    assert!(
        started.elapsed() >= std::time::Duration::from_millis(200),
        "middleware 等待未发生（提前返回）——park 面未覆盖"
    );
}

/// T-06 (2) `__axum:` fn-ref closure 的段入口：合成 fn-ref closure（与
/// axum_adapter 路由同形态）→ call_closure_segment park → resume 消费。
#[test]
fn plan705_engine_closure_segment_parks_and_resumes() {
    let body = r#"{"ok":true,"closure":705}"#.to_string();
    let body_for_server = body.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        std::thread::sleep(std::time::Duration::from_millis(300));
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body_for_server.len(),
            body_for_server
        );
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });
    let code = r#"
fn closure_target() str {
    var resp = Http.post_json("http://127.0.0.1:7059/seed", "q=1")
    resp
}
"#
    .replace("7059", &port.to_string());
    let (vm, _out, _entry, _t) = crate::create_vm_from_source(&code).expect("compile");
    // 合成 fn-ref closure：func_addr 指向目标导出（axum_adapter 注册的
    // 同一形态——Plan 383 fn-ref closure）。
    let (addr, _name) = vm
        .flash
        .exports_by_name
        .iter()
        .find(|(n, _)| n.contains("closure_target"))
        .map(|(n, a)| (*a, n.clone()))
        .expect("export closure_target");
    let closure_id = vm.closure_id_gen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
    let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
    let started = std::time::Instant::now();
    let outcome = vm.call_closure_segment(&mut task, closure_id, 0);
    let first = started.elapsed();
    let (req_id, seg) = match &outcome {
        crate::vm::engine::SegmentOutcome::Parked {
            wait: crate::vm::engine::ParkedWait::HttpRequest(id),
            seg,
        } => (*id, seg.clone()),
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => panic!(
            "closure 段应 park（server 延迟 300ms 下跑完=同步自旋回归），{:?}",
            first
        ),
        other => panic!("预期 Parked，得到 {:?}", other),
    };
    assert!(
        first < std::time::Duration::from_millis(250),
        "closure 段首返回 {:?}——同步自旋回归",
        first
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
        assert!(std::time::Instant::now() < deadline, "server 应答超时");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    server.join().expect("server");
    match vm.resume_fn_by_name_segment(&mut task, &seg) {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("closure 恢复应 Completed(Ok)，得到 {:?}", other),
    }
    let nv = task.ram.pop_nv();
    let got = vm
        .get_string(auto_val::decode_string(nv) as u32)
        .expect("string slot");
    assert_eq!(String::from_utf8_lossy(&got), body, "closure 段最终值");
}

// ============================================================================
// PLAN-705 T-07：资源生命周期探针（压测/取消风暴/晚完成回收/线程稳定）
// ============================================================================

/// Windows 线程计数（TH32CS_SNAPTHREAD 枚举，desktop_protocol 同款裸
/// extern 形态）——AC-03"线程稳定为固定 runtime"的数值探针。
#[cfg(windows)]
fn current_process_thread_count() -> usize {
    #[repr(C)]
    struct THREADENTRY32 {
        dw_size: u32,
        cnt_usage: u32,
        th32_thread_id: u32,
        th32_owner_process_id: u32,
        tp_base_pri: i32,
        tp_delta_pri: i32,
        dw_flags: u32,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
        fn Thread32First(handle: isize, entry: *mut THREADENTRY32) -> i32;
        fn Thread32Next(handle: isize, entry: *mut THREADENTRY32) -> i32;
        fn CloseHandle(handle: isize) -> i32;
        fn GetCurrentProcessId() -> u32;
    }
    const TH32CS_SNAPTHREAD: u32 = 0x4;
    const INVALID_HANDLE_VALUE: isize = -1;
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snap == INVALID_HANDLE_VALUE {
            return 0;
        }
        let mut entry = THREADENTRY32 {
            dw_size: std::mem::size_of::<THREADENTRY32>() as u32,
            cnt_usage: 0,
            th32_thread_id: 0,
            th32_owner_process_id: 0,
            tp_base_pri: 0,
            tp_delta_pri: 0,
            dw_flags: 0,
        };
        let pid = GetCurrentProcessId();
        let mut count = 0usize;
        if Thread32First(snap, &mut entry) != 0 {
            loop {
                if entry.th32_owner_process_id == pid {
                    count += 1;
                }
                if Thread32Next(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        count
    }
}

/// T-07 (1) 取消风暴 + 晚完成回收 + 基线重复可复现：短 deadline × 慢上游
/// 的 N 连发全部终结（503），迟到完成零复活，两轮迭代后 live-op/scope/
/// 许可逐字节回基线（AC-02/04；"同样配置重复可复现"）。
#[test]
fn plan705_e2e_cancel_storm_reclaims_deterministically() {
    let server_port = ephemeral_port();
    std::env::set_var("AUTO_HTTP_REQUEST_TIMEOUT_MS", "250");
    let op_baseline = crate::vm::ffi::async_http::live_op_count();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind upstream");
    let upstream_port = listener.local_addr().unwrap().port();
    let up = std::thread::spawn(move || {
        // 接受 storm+loop×1 个连接（deadline 先收口，上游应答全部迟到）。
        for _ in 0..6 {
            let Ok((mut stream, _)) = listener.accept() else { break };
            let mut buf = [0u8; 4096];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            std::thread::sleep(std::time::Duration::from_millis(400));
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
            let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
        }
    });
    start_plan705_server(
        &r#"
#[api(method = "GET", path = "/api/slow")]
fn slow() str {
    Http.post_json("http://127.0.0.1:UP/slow", "q=1")
}
"#
        .replace("UP", &upstream_port.to_string()),
        server_port,
    );
    for round in 0..2 {
        let mut clients = Vec::new();
        for i in 0..3 {
            clients.push(std::thread::spawn(move || {
                http_get_raw(server_port, &format!("/api/slow?r={round}-{i}"))
            }));
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        for c in clients {
            let (status, body) = c.join().expect("client");
            assert_eq!(status, 503, "风暴应全部 deadline 503，得到 {status} {body:?}");
        }
        // 上游迟到应答落地后：无复活、无泄漏（逐轮基线）。
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(
            crate::vm::ffi::async_http::live_op_count(),
            op_baseline,
            "round {round}: live-op 未回基线"
        );
        assert_eq!(crate::vm::ffi::http_server::live_scope_count(), 0, "round {round}: scope 未回基线");
    }
    up.join().expect("upstream");
}

/// T-07 (2) 线程数量稳定：M 个 client job 前后进程线程数不变（固定
/// async runtime，零每请求线程；AC-03 数值探针，Windows 面）。
///
/// PLAN-712 T-17：mock 上游补**读全请求头**——accepted 流在 Windows 继承
/// listener 的 nonblocking 标志，裸 read 立即 WouldBlock（旧代码丢弃该错、
/// 盲等 40ms 就回写关连接）= 请求未收完即关的竞态。传输失败被旧吞错契约
/// 掩盖（200 + 错误 body 假绿）；候选②让传输失败冒泡 500 后该竞态现形
/// （worktree 3/3 稳定复现）。修 mock：阻塞读至头终结符，保留 40ms 慢上游
/// 语义（线程重叠测量前提）。
#[cfg(windows)]
fn read_request_head_blocking(stream: &mut std::net::TcpStream) {
    use std::io::Read;
    let _ = stream.set_nonblocking(false);
    let mut buf = [0u8; 4096];
    let mut got = 0;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while got < buf.len() && std::time::Instant::now() < deadline {
        match stream.read(&mut buf[got..]) {
            Ok(0) => return,
            Ok(n) => {
                got += n;
                if buf[..got].windows(4).any(|w| w == b"\r\n\r\n") {
                    return;
                }
            }
            Err(_) => return,
        }
    }
}

#[cfg(windows)]
#[test]
fn plan705_client_thread_count_stable_under_load() {
    let server_port = ephemeral_port();
    // PLAN-712 T-17：测试复活——旧形态 handler 烤的是预热上游（listener1）
    // 的端口，而 listener1 随预热线程 join 即 drop；负载相 8 个上游 POST
    // 全部打到死端口，旧吞错契约（传输失败 → 错误 body → 200 假绿）把
    // 这一切掩盖，up2 无指向空转——测试从未真正测过「并发慢上游下的线程
    // 稳定」。候选②让传输失败冒泡 500 后该空转现形。修复：预热与负载
    // 同源指向 up2（活上游 + 40ms 慢应答语义保留），up1 退役。
    let listener2 = std::net::TcpListener::bind("127.0.0.1:0").expect("bind upstream2");
    let upstream_port = listener2.local_addr().unwrap().port();
    // 连接数 ≤ 请求数（reqwest 池对并发 job 复用连接）——accept 循环以
    // 停机标志收口，不假设固定次数。
    let up2_done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let up2_flag = up2_done.clone();
    let up2 = std::thread::spawn(move || {
        let _ = listener2.set_nonblocking(true);
        while !up2_flag.load(std::sync::atomic::Ordering::SeqCst) {
            match listener2.accept() {
                Ok((mut stream, _)) => {
                    read_request_head_blocking(&mut stream);
                    std::thread::sleep(std::time::Duration::from_millis(40));
                    let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
                    let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
                }
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        }
    });
    start_plan705_server(
        &r#"
#[api(method = "GET", path = "/api/fast")]
fn fast() str {
    Http.post_json("http://127.0.0.1:UP/fast", "q=1")
}
"#
        .replace("UP", &upstream_port.to_string()),
        server_port,
    );
    // 预热（runtime/client 池建立后的稳态为基线）——与负载同上游。
    let _ = http_get_raw(server_port, "/api/fast");
    std::thread::sleep(std::time::Duration::from_millis(200));
    let before = current_process_thread_count();
    assert!(before > 0, "线程枚举失败");
    let mut clients = Vec::new();
    eprintln!("[probe] load phase start");
    for i in 0..8 {
        clients.push(std::thread::spawn(move || {
            eprintln!("[probe] client {i} connecting");
            let r = http_get_raw(server_port, &format!("/api/fast?i={i}"));
            eprintln!("[probe] client {i} done {:?}", r.0);
            r
        }));
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    for c in clients {
        let (status, body) = c.join().expect("client");
        assert_eq!(status, 200, "上游活路径下必须 200，得到 {status} {body:?}");
    }
    eprintln!("[probe] all clients done");
    std::thread::sleep(std::time::Duration::from_millis(500));
    up2_done.store(true, std::sync::atomic::Ordering::SeqCst);
    up2.join().expect("upstream2");
    eprintln!("[probe] up2 joined");
    std::thread::sleep(std::time::Duration::from_millis(300));
    let after = current_process_thread_count();
    // 允许 ±2 抖动（tokio 偶发辅助线程），禁止随负载线性增长。
    assert!(
        after <= before + 2,
        "线程数随负载增长：before={before} after={after}——每请求线程回归"
    );
}

/// T-07 (3) 默认 HTTP 调用图零同步忙等门禁：http_server.rs 中
/// `.call_fn_by_name(`（同步忙等派发）只允许落在带 `legacy 同步驱动保留位`
/// 标记的串行 stdnet server（决策报告 §2 legacy 行）；`.call_closure(`
/// 同步入口零命中（`__axum:` 走 call_closure_segment）。
#[test]
fn plan705_gate_default_callgraph_no_busy_wait() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let src = std::fs::read_to_string(std::path::Path::new(manifest)
        .join("src/vm/ffi/http_server.rs"))
        .expect("read http_server.rs");
    let mut unmarked: Vec<String> = Vec::new();
    let mut legacy = 0usize;
    let lines: Vec<&str> = src.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if !line.contains(".call_fn_by_name(") && !line.contains(".call_closure(") {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        let back_start = i.saturating_sub(8);
        let is_legacy = lines[back_start..i]
            .iter()
            .any(|l| l.contains("legacy 同步驱动保留位"));
        if line.contains(".call_closure(") {
            unmarked.push(format!("line {}: 同步 call_closure 出现（应走段入口）", i + 1));
        } else if is_legacy {
            legacy += 1;
        } else {
            unmarked.push(format!("line {}: 未标记的同步忙等派发", i + 1));
        }
    }
    assert!(
        unmarked.is_empty(),
        "默认 HTTP 调用图发现同步忙等派发点: {unmarked:?}"
    );
    assert_eq!(legacy, 1, "legacy 保留位应恰 1 处（serve_blocking_stdnet），现 {legacy}");
}
