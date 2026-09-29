//! PLAN-707 T-01 spike：外部流/取消三个前提红测（全部针对**现有**公共面，
//! 期望失败——修复落地后逐个转绿；目标 API 的金样测试随 T-02..T-05 各自
//! 的测试文件落地，见 docs/plans/reports/707-stream-decision.md §6）。
//!
//! (1) **705 取消缺口定罪**：`cancel_live_op` 只删结果槽——正在运行的
//!     managed job future 未被中止（无 abort handle/取消 select），active
//!     许可继续被占用，gate 打开后 future 继续推进。这是本计划流取消的
//!     基础缺口（计划 §0/待澄清 2：不能只修 Spec 降低要求）。
//! (2) **流建立阻塞 owner**：`shim_http_get_stream` 用
//!     `thread::spawn(...blocking...).join()`——上游不给 headers 时 owner
//!     段被 join 阻塞（reqwest blocking 默认 30s 超时前不返回）。
//! (3) **SSE 逐 chunk lossy 解码破坏跨包 UTF-8**：生产端按 chunk
//!     `from_utf8_lossy` 再找 `\n\n`——码点被网络分段切开时数据被替换符
//!     玷污（T-03 增量 decoder 的立约动机）。

use crate::vm::ffi::async_http as ah;
use crate::vm::ffi::async_http::ClientLimits;
use crate::vm::ffi::stdlib::AsyncResult;

/// (1) 取消缺口：cancel 之后正在 await gate 的 job future 继续推进
/// （marker 被置位）。T-02 落地 managed abort 后转绿（future 在 gate
/// await 点被丢弃，marker 永不置位）。
#[test]
fn plan707_spike_cancel_gap_active_job_future_not_stopped() {
    // 本测试进程内 executor 尚未初始化（nextest 每测试独立进程）——
    // 先覆写限额再首次使用，active=1 保证"已进入执行"与许可一一对应。
    ah::set_client_limits_for_test(ClientLimits {
        workers: 1,
        max_active: 1,
        queue_capacity: 8,
        body_limit: 1024 * 1024,
        total_timeout: std::time::Duration::from_secs(30),
    });

    let req_id = crate::vm::ffi::stdlib::alloc_async_id();
    let marker = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel::<()>();
    let (gate_tx, gate_rx) = tokio::sync::oneshot::channel::<()>();
    let marker_for_job = marker.clone();
    let job = async move {
        // 通知测试：wrapper 已获 active 许可、job 已开始执行（非排队态）。
        let _ = entered_tx.send(());
        // 等 gate：取消语义必须能打断这个 await。
        let _ = gate_rx.await;
        marker_for_job.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(AsyncResult::Body("late-body".to_string()))
    };

    crate::vm::ffi::async_http::register_live_op(req_id);
    let submitted = ah::submit_client_job(req_id, job);
    assert!(submitted, "有界队列未满，提交应成功");

    // 等 job 真正进入 active 执行段（blocked 在 gate await 上）。
    entered_rx
        .blocking_recv()
        .expect("job 应进入执行并发出 entered 信号");

    // 取消：引擎超时臂/scope 取消的幂等终结入口。
    crate::vm::ffi::stdlib::drop_async_result(req_id);

    // 打开 gate：若 future 未被中止，它将继续推进并置位 marker。
    let _ = gate_tx.send(());
    // 宽限期：executor runtime 推进一轮。取消生效时 marker 必须保持 false。
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert!(
        !marker.load(std::sync::atomic::Ordering::SeqCst),
        "RED（705 缺口）：cancel 后 job future 仍在运行并被 gate 唤醒推进——\
         本地网络 future 未实际停止"
    );
    // 迟到完成丢弃（既有 presence 守卫，应保持绿）。
    assert!(
        !ah::complete_live_op(req_id, Ok(AsyncResult::Body("late".to_string()))),
        "取消后迟到完成必须被丢弃"
    );
}

/// (2) 流建立阻塞 + 源码路由静默失效（两个红面，T-04 一并转绿）：
///   a. 直达 `shim_http_get_stream`：上游 2s 后断开，shim 在断开前不返回
///      （`thread::spawn(...blocking...).join()` 阻塞 owner）——异步建立后
///      转绿（<500ms 返回句柄）。
///   b. 源码 `Http.get_stream(..)`：codegen 无此路由（裸 Http web 协议族
///      白名单只含 get/get_json/post/put/patch/delete），静默 no-op——
///      段"完成"但 HTTP_STREAMS 为空。`http_stream.*` 命名空间则直接
///      "Undefined variable"。T-04 补路由后转绿（表内有句柄）。
#[test]
fn plan707_spike_stream_open_blocks_owner_until_headers() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        // accept 后握 2s 再断开——阻塞读的解锁点（阻塞时长下界=2s）。
        if let Ok((stream, _)) = listener.accept() {
            let _ = stream.set_nodelay(true);
            std::thread::sleep(std::time::Duration::from_secs(2));
            drop(stream);
        }
    });

    let (vm, _out, _entry, _obj) =
        crate::create_vm_from_source("fn main() {}").expect("vm");

    // a. 直达 shim：测 owner 被阻塞的时长下界。
    let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
    let url_idx = vm.add_string(format!("http://127.0.0.1:{port}/slow").into_bytes());
    vm.rc_push_str_idx(&mut task, url_idx as usize);
    let t0 = std::time::Instant::now();
    let _ = crate::vm::ffi::stdlib::shim_http_get_stream(&mut task, &vm);
    let elapsed = t0.elapsed();
    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "RED：shim_http_get_stream 阻塞 owner {:?}（上游 2s 后才断开）\
         ——流建立必须非阻塞",
        elapsed
    );

    // b. 源码路由：Http.get_stream 段"完成"但必须真的触达 shim。
    let code = format!(
        r#"
fn open_stream() int {{
    var s = Http.get_stream("http://127.0.0.1:{port}/slow")
    return Http.stream_is_done(s)
}}
"#
    );
    let (vm2, _out, _entry, _obj) = crate::create_vm_from_source(&code).expect("vm2");
    let mut task2 = crate::vm::task::AutoTask::new(0, 65536, 0);
    let outcome = vm2.call_fn_by_name_segment(&mut task2, "open_stream", 0);
    assert!(
        matches!(
            outcome,
            crate::vm::engine::SegmentOutcome::Completed(Ok(()))
        ),
        "源码 Http.get_stream 段应 Completed(Ok)"
    );
    let inserted = crate::vm::ffi::stdlib::HTTP_STREAMS.with(|s| s.borrow().len());
    assert!(
        inserted > 0,
        "RED：Http.get_stream 静默 no-op（HTTP_STREAMS 空）——\
         codegen 缺 get_stream/stream_is_done 路由，http.at 声明面不可达"
    );
}

/// (3) SSE 跨 chunk UTF-8：码点被网络分段切开时，逐 chunk
/// `from_utf8_lossy` 把数据替换成 U+FFFD。T-03 落地增量 decoder 后转绿
/// （「中文」按字节切开跨两个 TCP 段交付，轮询结果必须无损）。
#[test]
fn plan707_spike_sse_utf8_split_across_chunks_survives() {
    // SSE 上游：同一事件 data 行的码点「中」(E4 B8 AD) 在 E4 后切开，
    // 第二段补全并带事件终结 \n\n；两段间隔发送确保分段到达。
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            use std::io::Write;
            let _ = stream.set_nodelay(true);
            let mut s = stream;
            let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n");
            let _ = s.write_all(b"data: \xe4");
            let _ = s.flush();
            std::thread::sleep(std::time::Duration::from_millis(200));
            let _ = s.write_all(b"\xb8\xad\xe6\x96\x87\n\n");
            let _ = s.flush();
            // 保持连接至测试结束（EOF 会被当作终结）。
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });

    let (vm, _out, _entry, _obj) =
        crate::create_vm_from_source("fn main() {}").expect("vm");
    let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);

    // sse_open：直接驱动 shim（栈上推 url 字符串）。
    let url_idx = vm.add_string(format!("http://127.0.0.1:{port}/sse").into_bytes());
    vm.rc_push_str_idx(&mut task, url_idx as usize);
    crate::vm::ffi::stdlib::shim_http_stream_sse_open(&mut task, &vm).expect("sse_open");
    let stream_id = auto_val::decode_i32(task.ram.pop_nv()) as i64;

    // sse_poll 轮询到首个非空载荷（宽裕 5s 上限）。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut got = String::new();
    loop {
        assert!(std::time::Instant::now() < deadline, "5s 内未收到 SSE data");
        task.ram.push_i32(stream_id as i32);
        crate::vm::ffi::stdlib::shim_http_stream_sse_poll(&mut task, &vm)
            .expect("sse_poll");
        let nv = task.ram.pop_nv();
        let s = vm
            .get_string(auto_val::decode_string(nv) as u32)
            .map(|b| String::from_utf8_lossy(&b).to_string())
            .unwrap_or_default();
        if !s.is_empty() {
            got = s;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(
        got, "中文",
        "RED：跨 chunk 的 UTF-8 码点被逐 chunk lossy 解码玷污（得到 {got:?}）"
    );
}
