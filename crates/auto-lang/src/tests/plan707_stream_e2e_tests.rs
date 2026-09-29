//! PLAN-707 T-06：端到端流转发与取消生命周期（真 TCP，`cargo th` 串行池）。
//!
//! 拓扑（AC-02/AC-03/AC-05 relay 面）：
//! ```text
//! 上游 SSE 服务器(真 TCP,分时 2 帧) → VM #[api] relay(~Iter<str> generator
//!   消费 http.sse_get_stream) → VM HTTP server → 真 TCP 下游客户端
//! ```
//! - 帧分时到达、末尾仅终结一次；
//! - 下游断连 → 上游连接在限定等待内被取消（资源组收口）；
//! - 自然 EOF/close 资源回基线。
//!
//! 命名含 `http_e2e` 与 `plan707`（th 名过滤器与交集命令双覆盖）。

/// 在独立线程起 VM HTTP server（AUTO_HTTP_PORT 进程内一次性；nextest 每
/// 测试独立进程，无 env 竞争），5s 内探活。
fn start_plan707_server(code: &str, port: u16) {
    crate::vm::ffi::stdlib::clear_http_routes();
    std::env::set_var("AUTO_HTTP_PORT", port.to_string());
    let code = code.to_string();
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let _ = crate::run(&code);
        })
        .expect("spawn plan707 server thread");
    for _ in 0..50 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("plan707 server 未在 5s 内就绪 port={port}");
}

/// 上游 SSE 服务器：accept 一连接，回 headers + frames（间隔 gap_ms），
/// 之后**握住连接**不 EOF（由客户端断连/取消收尾）。返回关闭观测通道。
fn serve_upstream_sse(frames: Vec<String>, gap_ms: u64) -> (u16, std::sync::mpsc::Receiver<&'static str>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind upstream");
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = std::sync::mpsc::channel::<&'static str>();
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            let _ = tx.send("accept-failed");
            return;
        };
        use std::io::{Read, Write};
        let _ = stream.set_nodelay(true);
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
        );
        for f in &frames {
            if stream.write_all(f.as_bytes()).is_err() {
                let _ = tx.send("write-failed");
                return;
            }
            let _ = stream.flush();
            std::thread::sleep(std::time::Duration::from_millis(gap_ms));
        }
        // 发完帧后握住：观察读端何时断（下游断连 → 上游被取消的信号）。
        stream.set_read_timeout(Some(std::time::Duration::from_millis(100))).ok();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            let mut probe = [0u8; 64];
            match stream.read(&mut probe) {
                Ok(0) => {
                    let _ = tx.send("eof");
                    return;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                    if std::time::Instant::now() > deadline {
                        let _ = tx.send("timeout");
                        return;
                    }
                }
                Err(_) => {
                    let _ = tx.send("reset");
                    return;
                }
                Ok(_) => {}
            }
        }
    });
    (port, rx)
}

/// SSE 下游客户端：GET 后逐帧读取（data 行），带总期限。返回 (帧, 事件间隔 ms, 终结形态)。
fn sse_client_read(port: u16, path: &str, max_wait_ms: u64) -> (Vec<String>, Vec<u128>, &'static str) {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("connect server");
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(500)))
        .ok();
    use std::io::{BufRead, BufReader, Write};
    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).expect("write req");
    let mut reader = BufReader::new(stream);
    // 跳过响应头。
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).expect("header read");
        if n == 0 || line == "\r\n" {
            break;
        }
    }
    let started = std::time::Instant::now();
    let mut frames = Vec::new();
    let mut at = Vec::new();
    let mut end = "timeout";
    let deadline = started + std::time::Duration::from_millis(max_wait_ms);
    loop {
        if std::time::Instant::now() > deadline {
            break;
        }
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                end = "eof";
                break;
            }
            Ok(_) => {
                let t = line.trim_end();
                if let Some(data) = t.strip_prefix("data: ") {
                    frames.push(data.to_string());
                    at.push(started.elapsed().as_millis());
                } else if t == "data:" {
                    frames.push(String::new());
                    at.push(started.elapsed().as_millis());
                }
                if frames.len() >= 2 && started.elapsed().as_millis() > 800 {
                    // 两帧已收 + 800ms 无第三事件 → 视为稳态终结检查点。
                    end = "steady";
                    break;
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                if frames.len() >= 2 {
                    end = "steady";
                    break;
                }
            }
            Err(_) => {
                end = "reset";
                break;
            }
        }
    }
    (frames, at, end)
}

/// AC-02/AC-05：本地 POST+auth 上游 → Auto public http API → ~Iter<str> →
/// 真 TCP 下游；两帧分时到达、末尾仅终结一次；上游流经请求资源组登记。
#[test]
fn http_e2e_plan707_relay_frames_timed_single_termination() {
    let op_baseline = crate::vm::ffi::async_http::live_op_count();

    // 上游：两帧间隔 250ms。
    let (up_port, _up_rx) = serve_upstream_sse(
        vec!["data: alpha\n\n".to_string(), "data: beta\n\n".to_string()],
        250,
    );

    const SERVER_PORT: u16 = 18701;
    start_plan707_server(
        &format!(
            r#"
#[api(method = "GET", path = "/api/relay")]
fn relay() ~Iter<str> {{
    for e in http.sse_get_stream("http://127.0.0.1:{up_port}/sse") {{
        yield e
    }}
}}
"#
        ),
        SERVER_PORT,
    );

    let (frames, at, end) = sse_client_read(SERVER_PORT, "/api/relay", 8000);
    // SSE 帧载荷为 JSON 值形态（sse_frame_from_nv 对 str 加引号）——线上
    // 契约即如此（017-chat publisher 同形）。
    assert_eq!(
        frames,
        vec!["\"alpha\"".to_string(), "\"beta\"".to_string()],
        "帧序列"
    );
    assert!(
        at.len() == 2 && at[1] - at[0] >= 150,
        "两帧应分时到达（间隔 ≥150ms，实际 {:?}）",
        at
    );
    assert!(
        matches!(end, "eof" | "steady" | "reset"),
        "末尾终结形态异常: {end}"
    );

    // 资源回基线（SSE 输出结束后 generator/stream 收口）。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while crate::vm::ffi::http_stream::stream_live_count() > 0 {
        assert!(std::time::Instant::now() < deadline, "5s 内流未回基线");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        crate::vm::ffi::async_http::live_op_count(),
        op_baseline,
        "live-op 未回基线"
    );
}

/// AC-03：下游断连 → 上游连接在限定等待内被取消（请求资源组收口判据：
/// 上游读端观测到连接关闭，而非 timeout）。
#[test]
fn http_e2e_plan707_relay_downstream_disconnect_cancels_upstream() {
    let op_baseline = crate::vm::ffi::async_http::live_op_count();

    // 上游发 1 帧后握住——下游收帧后立即断连，上游读端应观测到关闭。
    let (up_port, up_rx) =
        serve_upstream_sse(vec!["data: only\n\n".to_string()], 0);

    const SERVER_PORT: u16 = 18702;
    start_plan707_server(
        &format!(
            r#"
#[api(method = "GET", path = "/api/relay")]
fn relay() ~Iter<str> {{
    for e in http.sse_get_stream("http://127.0.0.1:{up_port}/sse") {{
        yield e
    }}
}}
"#
        ),
        SERVER_PORT,
    );

    // 下游：收 1 帧后 RST（SO_LINGER 0 硬断）。
    {
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", SERVER_PORT)).expect("connect");
        use std::io::{BufRead, BufReader, Write};
        let req = "GET /api/relay HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
        stream.write_all(req.as_bytes()).expect("write");
        let mut reader = BufReader::new(stream.try_clone().expect("clone"));
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).expect("hdr");
            if n == 0 || line == "\r\n" {
                break;
            }
        }
        // 读到首帧 data。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            line.clear();
            let n = reader.read_line(&mut line).expect("frame");
            assert!(n > 0, "首帧前不应 EOF");
            if line.trim_start().starts_with("data:") {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
        }
        // RST 断连。
        drop(reader);
        let _ = std::net::TcpStream::shutdown(&mut stream, std::net::Shutdown::Both);
        std::thread::sleep(std::time::Duration::from_millis(50));
        drop(stream);
    }

    // 上游读端应在限定等待内观测到连接关闭（eof/reset），而非 timeout。
    let observed = up_rx.recv_timeout(std::time::Duration::from_secs(8));
    assert!(
        matches!(observed, Ok("eof") | Ok("reset")),
        "下游断连后上游应被取消（观测到 {observed:?}；timeout=上游未断）"
    );

    // 资源回基线。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while crate::vm::ffi::http_stream::stream_live_count() > 0 {
        assert!(std::time::Instant::now() < deadline, "5s 内流未回基线");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        crate::vm::ffi::async_http::live_op_count(),
        op_baseline,
        "live-op 未回基线"
    );
}

/// AC-03：显式 close 路径（无 request scope）——open 后 close，资源出表、
/// 上游连接终结；两轮无增长（风暴收口的直接证据面）。
#[test]
fn http_e2e_plan707_stream_explicit_close_two_rounds_baseline() {
    let op_baseline = crate::vm::ffi::async_http::live_op_count();
    let mut task = crate::vm::task::AutoTask::new(0, 65536, 0);
    let (vm, _out, _e, _o) = crate::create_vm_from_source("fn main() {}").expect("vm");

    // 上游：accept 后握住（close 必须不依赖 EOF 生效）。
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let up_port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((s, _)) = listener.accept() {
            let _ = s;
            std::thread::sleep(std::time::Duration::from_secs(15));
        }
    });

    for round in 0..2 {
        let url_idx = vm
            .add_string(format!("http://127.0.0.1:{up_port}/hold?r={round}").into_bytes());
        vm.rc_push_str_idx(&mut task, url_idx as usize);
        crate::vm::ffi::stdlib::shim_http_get_stream(&mut task, &vm).expect("open");
        let id = task.ram.pop_i32() as u64;
        assert!(crate::vm::ffi::http_stream::stream_is_live(id), "round {round} 应存活");
        task.ram.push_i32(id as i32);
        crate::vm::ffi::stdlib::shim_http_stream_close(&mut task, &vm).expect("close");
        assert_eq!(
            crate::vm::ffi::http_stream::stream_live_count(),
            0,
            "round {round} close 后应回基线"
        );
    }
    assert_eq!(
        crate::vm::ffi::async_http::live_op_count(),
        op_baseline,
        "live-op 未回基线"
    );
}
