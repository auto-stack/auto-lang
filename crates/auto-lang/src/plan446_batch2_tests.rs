//! Plan 446 批二：http natives 可用性回归（§E 报告项）。
//!
//! 现场背景（auto-os-config Plan 007）：
//!   - E1: `res.status()` 恒返回哨兵 -2147483647（活 daemon、死端口皆然）；
//!   - E2: builder 链 `.send()` 之后同作用域内的后续任何 http 调用崩溃
//!     （handler 原子回滚、零诊断）。
//! 验收靶子：真实 TcpListener 断言线上语义。
//!
//! 修复落点（2026-08-28）：
//!   - E1 ✅：engine CALL_SPEC 对"裸句柄命中 HTTP_RESPONSES 表"的
//!     `.status()/.body()/.header(k)` 直接路由到只读 native（NATIVE_RESPONSE_*);
//!     另加 codegen arity 分流（零参 status → Response.status_code）。
//!   - E2 ✅：协议级根因 = CALL_SPEC 路径执行链式 `.send()` 后置位的
//!     waiting_http_request_id 无人消费（CALL_SPEC 无挂起协议），stale 标记
//!     令后续任何 CALL_NAT 无限 rewind+Yield（轨迹实证 82 万次自旋；旧观察
//!     的栈下溢崩溃为同一失序的另一表现）。修复 = engine 对 RequestBuilder
//!     堆 tag 链直调 shim 支撑的 native 族（2235-2239），send 以同步 drain
//!     完成 Yield 协议（轮询就绪后触发 shim 重入清位+推柄）。

#[cfg(test)]
mod plan446_batch2_http {
    fn run(src: &str) -> Result<String, String> {
        match std::panic::catch_unwind(|| crate::run_with_capture(src)) {
            Ok(Ok((_result, stdout))) => Ok(stdout),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("panicked".to_string()),
        }
    }

    /// One-shot local HTTP server; returns its port.
    fn spawn_one_shot_server(
        status_line: &'static str,
        content_type: &'static str,
        body: &'static str,
    ) -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let resp = format!(
                "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = s.write_all(resp.as_bytes());
        });
        port
    }

    // ------------------------------------------------------------------
    // E1: res.status() must report the wire status, not the i32 null
    // sentinel (-2147483647).
    // ------------------------------------------------------------------
    #[test]
    fn e1_res_status_reports_wire_status_on_get_handle() {
        let port = spawn_one_shot_server("HTTP/1.1 201 Created", "application/json", "{}");
        let src = format!(
            "use auto.http\n\
             fn main() {{\n\
             \x20   let res = http.get(\"http://127.0.0.1:{port}/probe\")\n\
             \x20   print(res.status().to_string())\n\
             }}\n"
        );
        let out = run(&src).expect("E1 program must run");
        assert!(
            !out.contains("-2147483647"),
            "E1 regression: sentinel -2147483647 leaked instead of wire status, stdout={out:?}"
        );
        assert!(out.contains("201"), "expected HTTP 201 on stdout, got: {out:?}");
    }

    // ------------------------------------------------------------------
    // E2: after a full request-builder chain (.send()), subsequent http
    // calls in the same scope must still work.
    // ------------------------------------------------------------------
    #[test]
    fn e2_second_http_call_after_builder_chain_survives() {
        let port_body = spawn_one_shot_server(
            "HTTP/1.1 200 OK",
            "application/json",
            r#"{"ok":true}"#,
        );
        let port_post = spawn_one_shot_server(
            "HTTP/1.1 204 No Content",
            "application/json",
            "",
        );
        let src = format!(
            "use auto.http\n\
             fn main() {{\n\
             \x20   let built = http.request(\"POST\", \"http://127.0.0.1:{port_post}/submit\").header(\"Content-Type\", \"application/json\").body(\"{{}}\").timeout(5000).send()\n\
             \x20   print(built.status().to_string())\n\
             \x20   let res = http.get(\"http://127.0.0.1:{port_body}/probe\")\n\
             \x20   print(res.status().to_string())\n\
             }}\n"
        );
        let out = run(&src).expect("E2 program must run");
        assert!(
            out.contains("204"),
            "builder-chain status missing from output: {out:?}"
        );
        assert!(
            out.contains("200"),
            "E2 regression: http.get after builder chain failed, stdout={out:?}"
        );
        assert!(
            !out.contains("-2147483647"),
            "sentinel leaked: {out:?}"
        );
    }
}
