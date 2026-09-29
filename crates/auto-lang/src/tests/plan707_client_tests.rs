//! PLAN-707 T-04：流客户端 shim 与 for-in 装配（决策 D-4/D-5）。
//!
//! - 零阻塞建立：上游不给 headers 时 open 立即返回句柄（spike 红测 2 的
//!   绿面已含，此处补 raw 读路径）；
//! - 手动 next/is_done/close：EOF `[DONE]` 哨兵、状态终结、close 真收口；
//! - for-in 三形态：内联调用 / 赋变量再 for / SSE iterator（plan341 已钉）；
//! - SSE poll：Data/pending/终结三态 + sse_close/sse_error 新面。

use crate::vm::ffi::http_stream as hs;
use crate::vm::ffi::http_stream::StreamLimits;
use crate::vm::task::AutoTask;

/// 段驱动到完成：park → 轮询就绪（测试泵；生产侧为 COMPLETION_NOTIFY
/// 事件唤醒泵，语义等价）→ resume，直至 Completed 或超时。
fn drive_to_completion(
    vm: &std::rc::Rc<crate::vm::engine::AutoVM>,
    task: &mut AutoTask,
    fn_name: &str,
) -> crate::vm::engine::SegmentOutcome {
    use crate::vm::engine::{ParkedWait, SegmentOutcome};
    let mut outcome = vm.call_fn_by_name_segment(task, fn_name, 0);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        match outcome {
            SegmentOutcome::Parked { wait, seg } => {
                while !match &wait {
                    ParkedWait::HttpStream(id) => hs::stream_ready(*id),
                    ParkedWait::HttpRequest(id) => {
                        crate::vm::ffi::stdlib::async_http_result_ready(*id)
                    }
                    ParkedWait::Future(_) => true,
                } {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "5s 内流未就绪——生产者未推进或通知丢失"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                outcome = vm.resume_fn_by_name_segment(task, &seg);
            }
            completed => return completed,
        }
    }
}

fn open_vm(code: &str) -> (std::rc::Rc<crate::vm::engine::AutoVM>, AutoTask) {
    let (vm, _out, _entry, _obj) = crate::create_vm_from_source(code).expect("compile");
    (std::rc::Rc::new(vm), AutoTask::new(0, 65536, 0))
}

/// 起 raw 流上游：N 个 chunk 分时发送后关闭。返回 (port, 关闭栅栏)。
fn serve_raw_chunks(chunks: Vec<&'static str>, gap_ms: u64) -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            use std::io::{Read, Write};
            let _ = stream.set_nodelay(true);
            // 先读请求再写响应：不带读取的关闭会触发 RST 而非 FIN，
            // 客户端看到的是连接错误而非 EOF（707 实测）。
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
            );
            for c in chunks {
                let _ = stream.write_all(c.as_bytes());
                let _ = stream.flush();
                std::thread::sleep(std::time::Duration::from_millis(gap_ms));
            }
        }
    });
    port
}

/// 起 SSE 上游：两帧分时 + 结束。返回 port。
fn serve_sse_frames(frames: Vec<String>, gap_ms: u64) -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            use std::io::Write;
            let _ = stream.set_nodelay(true);
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
            );
            for f in frames {
                let _ = stream.write_all(f.as_bytes());
                let _ = stream.flush();
                std::thread::sleep(std::time::Duration::from_millis(gap_ms));
            }
        }
    });
    port
}

/// 内联 for：`for c in Http.get_stream(url)` 拉全 chunk 后自然终结。
#[test]
fn plan707_client_for_in_inline_raw_stream() {
    let port = serve_raw_chunks(vec!["hello ", "stream"], 120);
    let code = format!(
        r#"
fn consume() int {{
    var n = 0
    for c in Http.get_stream("http://127.0.0.1:{port}/raw") {{
        n = n + 1
    }}
    return n
}}
"#
    );
    let (vm, mut task) = open_vm(&code);
    let outcome = drive_to_completion(&vm, &mut task, "consume");
    match outcome {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("段应 Completed(Ok)，得到 {other:?}"),
    }
    let n = auto_val::decode_i32(task.ram.pop_nv());
    assert!(n >= 2, "应拉到 ≥2 个文本块，得到 {n}");
}

/// 变量形态：`var s = Http.get_stream(u); for c in s { }`（决策 D-5 形态 2）。
#[test]
fn plan707_client_for_in_variable_stream() {
    let port = serve_raw_chunks(vec!["v1 ", "v2"], 120);
    let code = format!(
        r#"
fn consume() int {{
    var s = Http.get_stream("http://127.0.0.1:{port}/raw")
    var n = 0
    for c in s {{
        n = n + 1
    }}
    return n
}}
"#
    );
    let (vm, mut task) = open_vm(&code);
    let outcome = drive_to_completion(&vm, &mut task, "consume");
    match outcome {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("段应 Completed(Ok)，得到 {other:?}"),
    }
    let n = auto_val::decode_i32(task.ram.pop_nv());
    assert!(n >= 2, "变量形态应拉到 ≥2 个文本块，得到 {n}");
}

/// 手动 next/is_done/close：EOF `[DONE]`；is_done 终态置 1；close 幂等收口。
#[test]
fn plan707_client_manual_next_is_done_close() {
    let port = serve_raw_chunks(vec!["m1"], 80);
    let url = format!("http://127.0.0.1:{port}/raw");
    let code = format!(
        r#"
fn drive() str {{
    var s = Http.get_stream("{url}")
    var first = Http.stream_next(s)
    var tail = ""
    loop {{
        var chunk = Http.stream_next(s)
        if chunk == "[DONE]" {{ break }}
        tail = chunk
    }}
    var done = Http.stream_is_done(s)
    Http.stream_close(s)
    if done == 1 {{ return first + tail }}
    return "not-done"
}}
"#
    );
    let (vm, mut task) = open_vm(&code);
    let outcome = drive_to_completion(&vm, &mut task, "drive");
    match outcome {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("段应 Completed(Ok)，得到 {other:?}"),
    }
    let nv = task.ram.pop_nv();
    let got = vm
        .get_string(auto_val::decode_string(nv) as u32)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    assert_eq!(got, "m1", "first+tail 应为 m1（单 chunk 流），得到 {got:?}");
}

/// close 真收口：打开→立即 close→资源出表（不依赖自然 EOF）。
#[test]
fn plan707_client_close_reclaims_entry() {
    // 上游永不响应——close 必须仍能终结（不靠 EOF）。
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((s, _)) = listener.accept() {
            let _ = s;
            std::thread::sleep(std::time::Duration::from_secs(30));
        }
    });

    let mut task = AutoTask::new(0, 65536, 0);
    let (vm, _out, _e, _o) = crate::create_vm_from_source("fn main() {}").expect("vm");
    let url_idx = vm.add_string(format!("http://127.0.0.1:{port}/held").into_bytes());
    vm.rc_push_str_idx(&mut task, url_idx as usize);
    crate::vm::ffi::stdlib::shim_http_get_stream(&mut task, &vm).expect("open");
    let id = task.ram.pop_i32() as u64;

    // 打开后立即可见（Opening 态）。
    assert_eq!(hs::stream_live_count(), 1);
    task.ram.push_i32(id as i32);
    crate::vm::ffi::stdlib::shim_http_stream_close(&mut task, &vm).expect("close");
    assert_eq!(hs::stream_live_count(), 0, "close 必须出表收口");
    // close 后 pull = Eof（不复活）。
    assert_eq!(hs::stream_pull(id), hs::Pull::Eof);
}

/// SSE poll 三态 + sse_close/sse_error：pending ""、Data 载荷、close 后
/// poll 终结；sse_error 非消费式。
#[test]
fn plan707_client_sse_poll_close_error_surface() {
    let frame = "data: poll-1\n\n".to_string();
    let port = serve_sse_frames(vec![frame], 150);
    let mut task = AutoTask::new(0, 65536, 0);
    let (vm, _out, _e, _o) = crate::create_vm_from_source("fn main() {}").expect("vm");

    let url_idx = vm.add_string(format!("http://127.0.0.1:{port}/sse").into_bytes());
    vm.rc_push_str_idx(&mut task, url_idx as usize);
    crate::vm::ffi::stdlib::shim_http_stream_sse_open(&mut task, &vm).expect("open");
    let id = task.ram.pop_i32();

    // pending：上游静默期 poll = ""。
    task.ram.push_i32(id);
    crate::vm::ffi::stdlib::shim_http_stream_sse_poll(&mut task, &vm).expect("poll");
    let pending = vm
        .get_string(auto_val::decode_string(task.ram.pop_nv()) as u32)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    assert_eq!(pending, "", "上游静默期 poll 应为空串哨兵");

    // Data 帧到达。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let got = loop {
        assert!(std::time::Instant::now() < deadline, "5s 内未收到 data");
        task.ram.push_i32(id);
        crate::vm::ffi::stdlib::shim_http_stream_sse_poll(&mut task, &vm).expect("poll");
        let s = vm
            .get_string(auto_val::decode_string(task.ram.pop_nv()) as u32)
            .map(|b| String::from_utf8_lossy(&b).to_string())
            .unwrap_or_default();
        if !s.is_empty() {
            break s;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert_eq!(got, "poll-1");

    // 未终结时 sse_error = ""（非消费式）。
    task.ram.push_i32(id);
    crate::vm::ffi::stdlib::shim_http_stream_sse_error(&mut task, &vm).expect("error");
    let err = vm
        .get_string(auto_val::decode_string(task.ram.pop_nv()) as u32)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    assert_eq!(err, "");

    // sse_close：显式收口（上游还握着也不等 EOF）。
    task.ram.push_i32(id);
    crate::vm::ffi::stdlib::shim_http_stream_sse_close(&mut task, &vm).expect("close");
    assert_eq!(hs::stream_live_count(), 0, "sse_close 必须出表");
}

/// 队满提交终结性拒绝：queue=1 占满后 open 立即 Failed（不排队不兜底）。
#[test]
fn plan707_client_stream_queue_full_is_terminal() {
    hs::set_stream_limits_for_test(StreamLimits {
        max_active: 1,
        queue_capacity: 1,
        max_queued_items: 4,
        max_item_bytes: 64 * 1024,
        open_timeout: std::time::Duration::from_secs(5),
        idle_timeout: std::time::Duration::from_secs(5),
    });
    // 上游 A：握手不响应——占住唯一队列槽（提交后生产者 await active）。
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((s, _)) = listener.accept() {
            let _ = s;
            std::thread::sleep(std::time::Duration::from_secs(30));
        }
    });
    let mut task = AutoTask::new(0, 65536, 0);
    let (vm, _out, _e, _o) = crate::create_vm_from_source("fn main() {}").expect("vm");
    let url_idx = vm.add_string(format!("http://127.0.0.1:{port}/held").into_bytes());
    vm.rc_push_str_idx(&mut task, url_idx as usize);
    crate::vm::ffi::stdlib::shim_http_get_stream(&mut task, &vm).expect("open");
    let id_a = task.ram.pop_i64() as u64;

    // B：队列仅剩 0 槽（A 占 queue 槽）→ 提交即终结性 Failed。
    let url_idx = vm.add_string(format!("http://127.0.0.1:{port}/b").into_bytes());
    vm.rc_push_str_idx(&mut task, url_idx as usize);
    crate::vm::ffi::stdlib::shim_http_get_stream(&mut task, &vm).expect("open-b");
    let id_b = task.ram.pop_i64() as u64;
    match hs::stream_pull(id_b) {
        hs::Pull::Failed(e) => assert!(e.contains("queue full"), "e={e}"),
        other => panic!("队满应终结性 Failed，得到 {other:?}"),
    }

    hs::stream_cancel(id_a);
}

/// 流许可独立于非流式配额（决策 D-2）：client active=1 被占用时流仍可开。
#[test]
fn plan707_client_stream_permits_independent_of_client() {
    crate::vm::ffi::async_http::set_client_limits_for_test(
        crate::vm::ffi::async_http::ClientLimits {
            workers: 1,
            max_active: 1,
            queue_capacity: 8,
            body_limit: 1024 * 1024,
            total_timeout: std::time::Duration::from_secs(30),
        },
    );
    hs::set_stream_limits_for_test(StreamLimits {
        max_active: 4,
        queue_capacity: 8,
        max_queued_items: 4,
        max_item_bytes: 64 * 1024,
        open_timeout: std::time::Duration::from_secs(5),
        idle_timeout: std::time::Duration::from_secs(5),
    });
    // 占满 client active（永不完成的 managed job）。
    let (gate_tx, gate_rx) = tokio::sync::oneshot::channel::<()>();
    let req_id = crate::vm::ffi::stdlib::alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(req_id);
    assert!(crate::vm::ffi::async_http::submit_client_job(
        req_id,
        async move {
            let _ = gate_rx.await;
            Ok(crate::vm::ffi::stdlib::AsyncResult::Body("x".into()))
        }
    ));
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 流 open + 消费：不受 client active=1 影响。
    let port = serve_raw_chunks(vec!["independent"], 50);
    let code = format!(
        r#"
fn consume() int {{
    var n = 0
    for c in Http.get_stream("http://127.0.0.1:{port}/raw") {{
        n = n + 1
    }}
    return n
}}
"#
    );
    let (vm, mut task) = open_vm(&code);
    let outcome = drive_to_completion(&vm, &mut task, "consume");
    match outcome {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("段应 Completed(Ok)，得到 {other:?}"),
    }
    assert_eq!(auto_val::decode_i32(task.ram.pop_nv()), 1);

    crate::vm::ffi::stdlib::drop_async_result(req_id);
    let _ = gate_tx.send(());
}
