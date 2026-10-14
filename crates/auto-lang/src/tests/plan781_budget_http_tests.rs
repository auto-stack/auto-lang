//! PLAN-781 T-16: HTTP 请求级累计步预算真 TCP e2e（合同 =
//! docs/plans/reports/781-budget-decision.md D-2 入口矩阵 / §2 V-14/V-18）。
//!
//! `http_e2e_plan781` 族挂 `test-http-e2e`（cargo th 档；nextest 每测试
//! 一进程——env 置换天然隔离）。服务器 fixture 同 plan730 `start_server`
//! 形态（真 TCP、crate::run 自动起服）；预算经 `AUTO_HTTP_STEP_BUDGET`
//! env 注入（AutoVM 构造期快照）。
//!
//! 覆盖：低预算 = 指名资源拒绝（500 + "per-request cumulative budget"）
//! 且服务继续服务后续请求（V-14/V-18 单失败终态）；高预算同 handler 完成
//! （V-14 正例）；断连中途 = 服务器不楔死（V-18）。closure 入口（V-20）
//! 在 plan781_budget_tests 引擎级覆盖（plan705 合成 fn-ref closure 模式）。

#[cfg(feature = "test-http-e2e")]
mod budget_e2e {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    /// 同 plan730 start_server 形态：编译+运行 .at（#[api] 路由自动起服，
    /// 真 TCP），失败载荷有界上报。
    fn start_server(code: &str, port: u16) -> u16 {
        crate::vm::ffi::stdlib::clear_http_routes();
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = code.to_string();
        let outcome_rx = crate::vm::ffi::http_server::spawn_outcome_reporter(
            8 * 1024 * 1024,
            "crate::run",
            move || crate::run(&code),
        );
        for _ in 0..50 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return port;
            }
            if let Ok(outcome) = outcome_rx.try_recv() {
                panic!("test server failed to start on port {port}: {outcome}");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("test server never accepted connections on port {port}");
    }

    const PROGRAM: &str = r#"
#[api(method = "GET", path = "/api/heavy")]
fn heavy() str {
    var acc int = 0
    var i int = 0
    while i < 200000 {
        acc = acc + 1
        i = i + 1
    }
    return "done"
}

#[api(method = "GET", path = "/api/ping")]
fn ping() str {
    return "pong"
}
"#;

    fn http_get(port: u16, path: &str) -> (u16, String) {
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        s.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
        write!(
            s,
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut buf = String::new();
        s.read_to_string(&mut buf).unwrap_or_default();
        let status = buf
            .split_whitespace()
            .nth(1)
            .and_then(|c| c.parse::<u16>().ok())
            .unwrap_or(0);
        (status, buf)
    }

    fn free_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    /// V-14 负例 + V-18：低预算 = 指名资源拒绝（非解析错/空成功）+ 一次
    /// 失败终态后服务继续服务后续请求。
    #[test]
    fn http_e2e_plan781_budget_low_rejects_named_and_server_continues() {
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "1000");
        start_server(PROGRAM, port);
        let (status, body) = http_get(port, "/api/heavy");
        assert_eq!(status, 500, "{body}");
        assert!(
            body.contains("per-request cumulative budget")
                && body.contains("AUTO_HTTP_STEP_BUDGET"),
            "named resource rejection required: {body}"
        );
        // 单失败终态：服务器继续服务（轻路由不受污染）。
        let (status2, body2) = http_get(port, "/api/ping");
        assert_eq!(status2, 200, "{body2}");
        assert!(body2.contains("pong"), "{body2}");
        // 再次拒绝仍指名（重复失败不变形）。
        let (status3, body3) = http_get(port, "/api/heavy");
        assert_eq!(status3, 500, "{body3}");
        assert!(body3.contains("per-request cumulative budget"), "{body3}");
    }

    /// V-14 正例：宽预算下同一 handler 完成（相同有限计算分别成功/受拒
    /// 的“成功”半边；低预算半边由上一测持有）。
    #[test]
    fn http_e2e_plan781_budget_generous_completes() {
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "50000000");
        start_server(PROGRAM, port);
        let (status, body) = http_get(port, "/api/heavy");
        assert_eq!(status, 200, "{body}");
        assert!(body.contains("done"), "{body}");
    }

    /// V-18：请求中途断连——服务器不楔死（后续请求正常；scope 回收沿
    /// 既有机制，深度面由 plan705/707 族持有）。
    #[test]
    fn http_e2e_plan781_budget_client_disconnect_mid_request() {
        let port = free_port();
        std::env::set_var("AUTO_HTTP_STEP_BUDGET", "50000000");
        start_server(PROGRAM, port);
        let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        write!(
            &mut s,
            "GET /api/heavy HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        drop(s); // 中途断连（handler 尚在跑）
        std::thread::sleep(Duration::from_millis(300));
        let (status, body) = http_get(port, "/api/ping");
        assert_eq!(status, 200, "server must survive client disconnect: {body}");
        assert!(body.contains("pong"), "{body}");
    }
}
