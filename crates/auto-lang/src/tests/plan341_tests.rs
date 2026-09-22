//! Plan 341: 异步 HTTP 客户端（SSE 流式接收）测试。
//!
//! 启动一个微型原始 TCP SSE server，写出几帧 `data: ...\n\n`，然后用
//! `http.sse_get_stream(url)` 在 VM 里消费，验证 for-in 能拉到全部帧。

#[cfg(test)]
mod plan341_tests {
    use crate::run_with_capture;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// 启动一个微型 SSE server：每 accept 一个连接，先读掉 HTTP 请求行，
    /// 然后写 SSE header + 三帧 `data: <msg>\n\n`，最后关闭连接。
    /// 返回绑定的端口号。server 在独立线程跑，accept 一次后退出。
    fn spawn_sse_server(events: Vec<String>) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        std::thread::spawn(move || {
            // 只服务一个连接（测试用）
            if let Ok((mut stream, _)) = listener.accept() {
                // 读掉客户端发来的 HTTP 请求（读到空或 \r\n\r\n）
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                // 写 SSE 响应
                let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.flush();
                for ev in &events {
                    let frame = format!("data: {}\n\n", ev);
                    let _ = stream.write_all(frame.as_bytes());
                    let _ = stream.flush();
                    // 小延时模拟流式（分帧到达）
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                // 关闭连接（Connection: close）→ 客户端 stream 结束
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        });
        // 给 server 一点时间进入 accept
        std::thread::sleep(std::time::Duration::from_millis(50));
        port
    }

    /// for-in 消费 SSE 流：拉到全部 3 帧。
    /// 注：for-in 要求迭代对象是 Call 表达式（codegen 的 for x in <Call> 路径），
    /// 所以直接内联 http.sse_get_stream(url)，不能先存到变量再 for-in。
    #[test]
    fn test_sse_get_stream_for_in() {
        let port = spawn_sse_server(vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ]);
        let url = format!("http://127.0.0.1:{}/api/stream", port);
        let code = format!(
            r#"
for event in http.sse_get_stream("{}") {{
    print(event)
}}
print("done")
"#,
            url
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok(), "sse stream should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        eprintln!("plan341 sse stdout = [{}]", stdout);
        assert!(stdout.contains("first"), "missing first frame: [{}]", stdout);
        assert!(stdout.contains("second"), "missing second frame: [{}]", stdout);
        assert!(stdout.contains("third"), "missing third frame: [{}]", stdout);
        assert!(stdout.contains("done"), "missing done: [{}]", stdout);
    }

    /// 单帧 SSE 流。
    #[test]
    fn test_sse_get_stream_single() {
        let port = spawn_sse_server(vec!["hello-sse".to_string()]);
        let url = format!("http://127.0.0.1:{}/api/s", port);
        let code = format!(
            r#"
for event in http.sse_get_stream("{}") {{
    print(event)
}}
"#,
            url
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok(), "single sse should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        eprintln!("plan341 single sse = [{}]", stdout);
        assert!(stdout.contains("hello-sse"), "missing event: [{}]", stdout);
    }

    /// 空流（server 立即关闭，不写任何 data 帧）。
    #[test]
    fn test_sse_get_stream_empty() {
        let port = spawn_sse_server(vec![]);
        let url = format!("http://127.0.0.1:{}/api/e", port);
        let code = format!(
            r#"
var got = 0
for event in http.sse_get_stream("{}") {{
    got = got + 1
}}
print("got")
print(got)
"#,
            url
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok(), "empty sse should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        eprintln!("plan341 empty sse = [{}]", stdout);
        assert!(stdout.contains("got"), "missing got: [{}]", stdout);
        // got should be 0 (no events)
        assert!(stdout.contains("0"), "expected 0 events: [{}]", stdout);
    }
}
