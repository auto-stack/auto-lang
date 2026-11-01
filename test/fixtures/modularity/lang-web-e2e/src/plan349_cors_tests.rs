//! PLAN-349 步骤 8 W5: AutoVM server CORS（PLAN-906 随 Web 符号迁入
//! lang-web-e2e 探针工程——handle_cors_preflight/cors_headers 为
//! auto_web::http_server 所有；测试 ID/断言原样）。
mod plan349_cors_tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    // ------------------------------------------------------------------
    // 步骤 8 W5: AutoVM server CORS
    // ------------------------------------------------------------------

    /// handle_cors_preflight returns a full 204 response for OPTIONS — verify
    /// its shape (status + required CORS headers).
    #[test]
    fn test_cors_preflight_shape() {
        let preflight = auto_web::http_server::handle_cors_preflight("OPTIONS");
        assert!(
            preflight.is_some(),
            "OPTIONS must produce a preflight response"
        );
        let resp = preflight.unwrap();
        assert!(
            resp.contains("204 No Content"),
            "preflight must be 204: [{}]",
            resp
        );
        assert!(
            resp.contains("Access-Control-Allow-Origin"),
            "preflight must carry CORS origin header: [{}]",
            resp
        );
        assert!(
            resp.contains("Access-Control-Allow-Methods"),
            "preflight must list allowed methods: [{}]",
            resp
        );
        assert!(
            resp.contains("OPTIONS"),
            "allowed methods must include OPTIONS: [{}]",
            resp
        );
    }

    /// Non-OPTIONS methods must NOT short-circuit (return None) so normal
    /// routing proceeds and the regular response still carries CORS headers.
    #[test]
    fn test_cors_preflight_skips_non_options() {
        for method in &["GET", "POST", "PUT", "DELETE", "PATCH"] {
            assert!(
                auto_web::http_server::handle_cors_preflight(method).is_none(),
                "{} should not be treated as a preflight request",
                method
            );
        }
    }

    /// `cors_headers()` must be appendable to a normal response and include the
    /// three required headers (origin, methods, headers).
    #[test]
    fn test_cors_headers_content() {
        let h = auto_web::http_server::cors_headers();
        assert!(
            h.contains("Access-Control-Allow-Origin"),
            "missing origin: [{}]",
            h
        );
        assert!(
            h.contains("Access-Control-Allow-Methods"),
            "missing methods: [{}]",
            h
        );
        assert!(
            h.contains("Access-Control-Allow-Headers"),
            "missing headers: [{}]",
            h
        );
        // Default origin is `*` when AUTO_CORS_ORIGIN is unset.
        assert!(
            h.contains("*") || std::env::var("AUTO_CORS_ORIGIN").is_ok(),
            "default origin should be '*': [{}]",
            h
        );
    }

    /// Lightweight integration: a mock that mirrors the AutoVM response shape
    /// (CORS-bearing 404 built with cors_headers()) must expose the header over
    /// a real socket exchange.
    #[test]
    fn test_cors_header_on_real_get() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let cors = auto_web::http_server::cors_headers();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let resp = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n{}\r\n",
                    cors
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("connect");
        stream
            .write_all(b"GET /anything HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        assert!(
            resp.contains("Access-Control-Allow-Origin"),
            "response must carry CORS header: [{}]",
            resp
        );
    }

}
