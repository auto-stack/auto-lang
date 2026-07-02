//! Plan 349: HTTP 扩展特性 VM 测试。
//!
//! 测试 multipart upload、download、download_with_progress、WebSocket echo
//! 等 VM native 的功能正确性。

#[cfg(test)]
mod plan349_tests {
    use crate::run_with_capture;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// 启动一个简单的 mock HTTP server，返回固定 JSON 响应。
    /// 用于测试 http.download 等基础 HTTP 功能。
    fn spawn_mock_server(response_body: &str) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let body = response_body.to_string();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(), body
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        });
        std::thread::sleep(std::time::Duration::from_millis(50));
        port
    }

    /// 启动一个支持 Range 的 mock server（用于断点续传测试）。
    fn spawn_range_server(full_body: &[u8]) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let full_body = full_body.to_vec();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);

                // Parse Range header
                let (start, data) = if let Some(range_line) = request.lines().find(|l| l.to_lowercase().starts_with("range:")) {
                    let range_val = range_line.split(':').nth(1).unwrap_or("").trim();
                    let start: usize = range_val
                        .strip_prefix("bytes=")
                        .and_then(|s| s.split('-').next())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    (start, &full_body[start..])
                } else {
                    (0, &full_body[..])
                };

                let end = start + data.len() - 1;
                let total = full_body.len();
                let resp = format!(
                    "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\nConnection: close\r\n\r\n",
                    data.len(), start, end, total
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.write_all(data);
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        });
        std::thread::sleep(std::time::Duration::from_millis(50));
        port
    }

    /// Test http.download: download a file and verify content.
    #[test]
    fn test_http_download() {
        let port = spawn_mock_server(r#"{"status":"ok"}"#);
        let url = format!("http://127.0.0.1:{}/api/test", port);
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("auto_test_download_{}.json", port));

        let code = format!(
            r#"
let ok = http.download("{}", "{}")
print(ok)
"#,
            url,
            file_path.to_str().unwrap()
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok(), "download should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        eprintln!("plan349 download stdout = [{}]", stdout);
        assert!(stdout.contains("1"), "expected success (1), got: [{}]", stdout);

        // Verify file content
        let content = std::fs::read_to_string(&file_path).unwrap_or_default();
        assert!(content.contains("ok"), "file should contain response body");
        let _ = std::fs::remove_file(&file_path);
    }

    /// Test http.download_resume with Range header.
    #[test]
    fn test_http_download_resume() {
        let full_data = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let port = spawn_range_server(full_data);
        let url = format!("http://127.0.0.1:{}/file", port);
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("auto_test_resume_{}.bin", port));

        // Pre-create file with first 10 bytes.
        std::fs::write(&file_path, &full_data[..10]).unwrap();

        // Resume from offset 10.
        let code = format!(
            r#"
let ok = http.download_resume("{}", "{}", 10)
print(ok)
"#,
            url,
            file_path.to_str().unwrap()
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok());
        let (_, stdout) = result.unwrap();
        eprintln!("plan349 resume stdout = [{}]", stdout);
        assert!(stdout.contains("1"), "expected success, got: [{}]", stdout);

        // Verify file: first 10 bytes + rest.
        let content = std::fs::read(&file_path).unwrap_or_default();
        assert_eq!(content, full_data, "file should contain full data after resume");
        let _ = std::fs::remove_file(&file_path);
    }

    /// Test TLS skip_verify: verify the native is registered and callable.
    /// We don't test against a real HTTPS server, just verify the function
    /// doesn't crash when called.
    #[test]
    fn test_tls_skip_verify_native_exists() {
        // Just verify _tls_skip_verify helper code exists by checking that
        // http.request().tls_skip_verify(true) doesn't panic on registration.
        // A full test would need a real HTTPS server with self-signed cert.
        let code = r#"
let builder = http.request("GET", "http://127.0.0.1:1/nonexistent")
builder.tls_skip_verify(true)
print("ok")
"#;
        // This may fail at .send() (connection refused), but shouldn't panic
        // on .tls_skip_verify() registration.
        let result = run_with_capture(code);
        assert!(result.is_ok(), "tls_skip_verify should not crash: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        assert!(stdout.contains("ok"), "expected ok, got: [{}]", stdout);
    }

    /// Test multipart: verify RequestBuilder.multipart_text is callable.
    #[test]
    fn test_multipart_text_native_exists() {
        let code = r#"
let builder = http.request("POST", "http://127.0.0.1:1/nonexistent")
builder.multipart_text("field1", "value1")
print("ok")
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok());
        let (_, stdout) = result.unwrap();
        assert!(stdout.contains("ok"), "expected ok, got: [{}]", stdout);
    }
}
