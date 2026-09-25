//! PLAN-702 T-01: engine 段执行驱动单测（`call_fn_by_name_segment` /
//! `resume_fn_by_name_segment`）。
//!
//! 覆盖测试设计三件套：
//! (a) handler 内异步 HTTP：首调用返回 `Parked`（忙等零发生——返回先于
//!     服务端响应）、ASYNC_RESULTS 结果槽**未回收**（PLAN-027 缺陷 A 的
//!     drop 只属于真正放弃等待的超时路径）；
//! (b) 结果就绪后续跑：shim 重入臂消费结果、handler 跑到 RET、返回值与
//!     同步对拍一致；
//! (c) 从不 yield 的 handler：段入口与旧同步入口结果一致（兼容性论证）。
//! 另含 resume 段内 VM 错误可被 .at `try/catch` 捕获（T-04 错误语义，
//! E7 同族问题在 parked 路径不得复现）。
//!
//! HTTP worker 用真本地 server（127.0.0.1:0 随机端口 + 受控延迟）——
//! 完全确定性，不依赖外网/环境端口。

use crate::vm::engine::{AutoVM, ParkedWait, SegmentOutcome};
use crate::vm::task::AutoTask;

/// PLAN-702 T-01(a)/(b): 首段 park 零忙等 + 结果槽不回收 + 续跑取值正确。
#[test]
fn engine_segment_parks_on_async_http_then_resumes_with_result() {
    let body = r#"{"ok":true,"seeded":702}"#.to_string();
    let body_for_server = body.clone();
    // 本地受控 server：延迟 400ms 应答——park 返回必须远早于此（证明零忙等）。
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        // 读掉请求头（到空行即止；POST body 按 Content-Length 丢弃）。
        let mut buf = [0u8; 4096];
        let mut read_total = 0usize;
        loop {
            let n = std::io::Read::read(&mut stream, &mut buf).unwrap_or(0);
            read_total += n;
            let head_end = buf[..read_total]
                .windows(4)
                .position(|w| w == b"\r\n\r\n");
            if let Some(pos) = head_end {
                let headers = String::from_utf8_lossy(&buf[..pos]).to_lowercase();
                let content_length: usize = headers
                    .lines()
                    .find_map(|l| l.strip_prefix("content-length:").map(|v| v.trim().parse().unwrap_or(0)))
                    .unwrap_or(0);
                let have = read_total.saturating_sub(pos + 4);
                if have < content_length {
                    let need = content_length - have;
                    let mut rest = vec![0u8; need];
                    let _ = std::io::Read::read(&mut stream, &mut rest);
                }
                break;
            }
            if n == 0 {
                break;
            }
        }
        // 受控延迟：段驱动若仍在忙等，这段墙钟会拖住首调用返回（红）。
        std::thread::sleep(std::time::Duration::from_millis(400));
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body_for_server.len(),
            body_for_server
        );
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });

    let code = r#"
fn pick_data() str {
    var resp = Http.post_json("http://127.0.0.1:7020/seed", "q=1")
    resp
}
"#;
    // 端口占位符替换为随机端口。
    let code = code.replace("7020", &port.to_string());
    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(&code).expect("compile");

    let mut task = AutoTask::new(0, 65536, 0);
    let started = std::time::Instant::now();
    let outcome = vm.call_fn_by_name_segment(&mut task, "pick_data", 0);
    let first_segment_elapsed = started.elapsed();

    // (a) 首段 = Parked{HttpRequest}，且返回发生在 server 应答之前（零忙等）。
    let (req_id, seg) = match &outcome {
        SegmentOutcome::Parked { wait: ParkedWait::HttpRequest(id), seg } => (*id, seg.clone()),
        SegmentOutcome::Completed(Ok(())) => {
            panic!("handler 应 park 而非同步跑完——server 延迟 400ms 下同步跑完=忙等回归")
        }
        other => panic!("预期 Parked(HttpRequest)，得到 {:?}", other),
    };
    assert!(
        first_segment_elapsed < std::time::Duration::from_millis(350),
        "首段返回耗时 {:?} —— 忙等回归（应立即返回）",
        first_segment_elapsed
    );
    // PLAN-027 缺陷 A 边界：parked 路径不回收结果槽（条目仍在，未完成的
    // None 或 worker 已写入的 Some 均合法——绝不因 park 消失）。
    assert!(
        crate::vm::ffi::stdlib::async_http_result_ready(req_id) || {
            let map = crate::vm::ffi::stdlib::ASYNC_RESULTS.lock().unwrap();
            map.contains_key(&req_id)
        },
        "parked 后 ASYNC_RESULTS[{}] 被回收——PLAN-027 缺陷 A 边界破坏",
        req_id
    );

    // 等 server 应答落槽（真 worker 写入）。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
        assert!(std::time::Instant::now() < deadline, "server 应答超时");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    server.join().expect("server thread");

    // (b) 恢复段：shim 重入臂消费结果 → handler 跑到 RET。
    match vm.resume_fn_by_name_segment(&mut task, &seg) {
        SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("恢复段应 Completed(Ok)，得到 {:?}", other),
    }
    // 返回值 = body 字符串（池索引形态）。
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_string(nv), "post_json 返回应为字符串 nv");
    let idx = auto_val::decode_string(nv) as u32;
    let got = vm.get_string(idx).expect("string slot");
    assert_eq!(String::from_utf8_lossy(&got), body);
}

/// PLAN-702 T-01(c): 从不 yield 的 handler——段入口与同步入口逐字节同。
#[test]
fn engine_segment_parity_with_sync_driver_on_nonyielding_handler() {
    let code = r#"
fn add_things(a int, b int) int {
    a + b * 2
}
"#;
    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(code).expect("compile");

    let run_sync = |vm: &AutoVM| -> i32 {
        let mut task = AutoTask::new(0, 65536, 0);
        task.ram.push_i32(20);
        task.ram.push_i32(11);
        vm.call_fn_by_name(&mut task, "add_things", 2).expect("sync call");
        task.ram.pop_i32()
    };
    let run_segment = |vm: &AutoVM| -> i32 {
        let mut task = AutoTask::new(0, 65536, 0);
        task.ram.push_i32(20);
        task.ram.push_i32(11);
        match vm.call_fn_by_name_segment(&mut task, "add_things", 2) {
            SegmentOutcome::Completed(Ok(())) => {}
            other => panic!("非 yield handler 应 Completed，得到 {:?}", other),
        }
        task.ram.pop_i32()
    };
    assert_eq!(run_sync(&vm), 42, "同步基线");
    assert_eq!(run_segment(&vm), 42, "段驱动对拍");
    assert_eq!(run_sync(&vm), run_segment(&vm), "两驱动结果一致");
}

/// PLAN-702 T-04: 恢复段内的 VM 错误可被 .at `try/catch` 捕获——E7 中
/// "catch 接不住 VM 级 RuntimeError" 的同族问题在 parked 路径不得复现。
#[test]
fn engine_segment_resume_error_caught_by_at_try() {
    let code = r#"
fn guarded() str {
    try {
        var resp = Http.post_json("http://127.0.0.1:7021/boom", "q=1")
        var d = resp.len() % 2
        var boom = 10 / d
        "unreached"
    } catch (e) {
        "caught"
    }
}
"#;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let code = code.replace("7021", &port.to_string());
    let body_for_server = "aaaa".to_string(); // len 4 → d = 0 → DivisionByZero
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body_for_server.len(),
            body_for_server
        );
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });

    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(&code).expect("compile");
    let mut task = AutoTask::new(0, 65536, 0);
    let outcome = vm.call_fn_by_name_segment(&mut task, "guarded", 0);
    let (req_id, seg) = match outcome {
        SegmentOutcome::Parked { wait: ParkedWait::HttpRequest(id), seg } => (id, seg),
        other => panic!("预期 park，得到 {:?}", other),
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
        assert!(std::time::Instant::now() < deadline, "server 应答超时");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    server.join().expect("server thread");

    // 恢复段：resume 段内 10/0 的 DivisionByZero 被 .at catch 截获，
    // handler 正常 RET "caught"（而非 Err 冒泡）。
    match vm.resume_fn_by_name_segment(&mut task, &seg) {
        SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("catch 应截获 resume 段错误，得到 {:?}", other),
    }
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_string(nv));
    let got = vm
        .get_string(auto_val::decode_string(nv) as u32)
        .expect("string slot");
    assert_eq!(String::from_utf8_lossy(&got), "caught");
}
