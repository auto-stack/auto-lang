//! PLAN-712 T-17（候选②）：`Http.*_json` 的失败语义分层回归。
//!
//! 旧契约把**传输层失败**（连接拒绝/超时/DNS）包成 `{"error":..,"status":0}`
//! 字符串当成功值推回——`.at` try/catch 永不触发，坏数据静默入态（018
//! 详情页「0 entries」的助燃层：相对路径 fetch 打到死端口 → 吞错 → 空态）。
//! 新契约对齐浏览器 `fetch()`：
//!   - 网络层失败 = **可捕获异常**（shim 重入臂转 VMError，catch 可接）；
//!   - HTTP 非 2xx = 仍是**值面**（`{"error":"HTTP n","status":n}`，可读
//!     `.status`——fetch().json() 的「HTTP 错误状态不 reject」同律）。
//!
//! 门槛级证据来自段驱动（与 plan702 同套 harness）：park → async 执行器
//! 完成 → resume → 断言 catch 截获 / 值面返回。

use crate::vm::engine::{ParkedWait, SegmentOutcome};
use crate::vm::task::AutoTask;

#[test]
fn transport_error_is_caught_by_at_try_not_swallowed() {
    // 死端口：bind 后立刻 drop listener = 该端口必然连接拒绝（秒拒，
    // 不等超时——与实机 86ms 形态同源）。
    let dead = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        l.local_addr().unwrap().port()
    };
    let code = format!(
        r#"
fn guarded() str {{
    try {{
        var d = Http.get_json("http://127.0.0.1:{dead}/api/books/1")
        "unreached"
    }} catch (e) {{
        "caught"
    }}
}}
"#
    );

    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(&code).expect("compile");
    let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
    let (req_id, seg) = match vm.call_fn_by_name_segment(&mut task, "guarded", 0) {
        crate::vm::engine::SegmentOutcome::Parked {
            wait: crate::vm::engine::ParkedWait::HttpRequest(id),
            seg,
        } => (id, seg),
        other => panic!("预期 park(HttpRequest)，得到 {:?}", other),
    };

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
        assert!(
            std::time::Instant::now() < deadline,
            "连接拒绝应在秒级就绪（死端口秒拒形态）"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    // 恢复段：传输错误必须被 .at catch 截获（handler 正常 RET "caught"），
    // 而不是把 {"error":..,"status":0} 当值面返回（旧吞错形态 = "unreached"）。
    match vm.resume_fn_by_name_segment(&mut task, &seg) {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("catch 应截获传输错误，得到 {:?}", other),
    }
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_string(nv));
    let got = vm
        .get_string(auto_val::decode_string(nv) as u32)
        .expect("string slot");
    assert_eq!(
        String::from_utf8_lossy(&got),
        "caught",
        "传输失败必须走 catch 分支（旧吞错形态会返回 \"unreached\"）"
    );
}

#[test]
fn http_error_status_still_returns_value_not_throw() {
    // 本地 stub 回 500：非 2xx 保持值面（catch 不触发），body 可读 .status。
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        let resp =
            "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    });

    let code = format!(
        r#"
fn guarded() str {{
    try {{
        var d = Http.get_json("http://127.0.0.1:{port}/api/x")
        "value"
    }} catch (e) {{
        "caught"
    }}
}}
"#
    );

    let (vm, _stdout, _entry, _object_type) = crate::create_vm_from_source(&code).expect("compile");
    let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
    let (req_id, seg) = match vm.call_fn_by_name_segment(&mut task, "guarded", 0) {
        crate::vm::engine::SegmentOutcome::Parked {
            wait: crate::vm::engine::ParkedWait::HttpRequest(id),
            seg,
        } => (id, seg),
        other => panic!("预期 park(HttpRequest)，得到 {:?}", other),
    };

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
        assert!(std::time::Instant::now() < deadline, "stub 应答超时");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    server.join().expect("server thread");

    match vm.resume_fn_by_name_segment(&mut task, &seg) {
        crate::vm::engine::SegmentOutcome::Completed(Ok(())) => {}
        other => panic!("非 2xx 不得抛（值面契约），得到 {:?}", other),
    }
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_string(nv));
    let got = vm
        .get_string(auto_val::decode_string(nv) as u32)
        .expect("string slot");
    assert_eq!(
        String::from_utf8_lossy(&got),
        "value",
        "非 2xx 走值面（fetch().json() 同律：HTTP 错误状态不 reject）"
    );
}
