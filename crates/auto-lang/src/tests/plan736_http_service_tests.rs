//! PLAN-736: HTTP 服务部署基线——配置/身份/策略 scoped 测试族。
//!
//! T-02 族：配置解析与范围、proxy_service 必填性、纯策略（CORS/Host/代理）、
//! 有界限速器、to_json round-trip、upload 转发壳完备性（qualify 后路径可解析）。
//! 真 TCP wire 族（`http_e2e_plan736`，test-http-e2e）随 T-03..T-06 落地。

use crate::http_service_config::{
    resolve_service_config, CorsOutcome, RateDecision, RateLimiter, ServiceProfile,
};

fn dev() -> crate::http_service_config::HttpServiceConfig {
    resolve_service_config(r#"{"profile":"development"}"#, None).unwrap()
}

fn svc() -> crate::http_service_config::HttpServiceConfig {
    resolve_service_config(
        r#"{"profile":"proxy_service","allowed_hosts":["api.test"],"auth":{"responsibility":"edge"}}"#,
        None,
    )
    .unwrap()
}

#[test]
fn t02_dev_defaults_are_loopback_and_legacy_cors() {
    let c = dev();
    assert_eq!(c.listen_addr.to_string(), "127.0.0.1");
    assert_eq!(c.listen_port, 8080);
    assert_eq!(c.profile, ServiceProfile::Development);
    assert_eq!(c.cors.allowed_origins, vec!["*"]);
    assert!(c.features.media_scan, "development 保留现状（scan 路由开）");
}

#[test]
fn t02_proxy_service_locks_down_by_default() {
    let c = svc();
    assert!(c.cors.allowed_origins.is_empty(), "service 缺省无跨域授权");
    assert!(!c.features.media_scan && !c.features.photo_scan && !c.features.websocket_echo);
    assert_eq!(c.auth_responsibility.as_str(), "edge");
}

#[test]
fn t02_explicit_non_loopback_and_cli_override() {
    let c = resolve_service_config(
        r#"{"profile":"development","listen":{"addr":"0.0.0.0","port":9000}}"#,
        Some(9100),
    )
    .unwrap();
    assert_eq!(c.listen_addr.to_string(), "0.0.0.0");
    assert_eq!(c.listen_port, 9100, "显式 -B 覆盖 config listen.port");
}

#[test]
fn t02_bad_values_exit_named() {
    for json in [
        r#"{"profile":"development","limits":{"max_connections":0}}"#,
        r#"{"profile":"development","nosuch":true}"#,
        r#"{"profile":"proxy_service"}"#,
        r#"{"profile":"development","cors":{"allow_credentials":true}}"#,
    ] {
        let e = resolve_service_config(json, None)
            .err()
            .unwrap_or_else(|| panic!("must reject: {json}"));
        assert!(!e.field.is_empty());
    }
}

#[test]
fn t02_cors_preflight_matrix() {
    let c = svc();
    // service 空 origins：一切跨域 preflight 拒绝（403，无 CORS 头）。
    let (status, acao, _) = c.cors.preflight(Some("http://a.com"), Some("GET"), &[]);
    assert_eq!((status, acao), (403, None));
    assert_eq!(
        c.cors.actual_request(Some("http://a.com")),
        CorsOutcome::Absent
    );
    // development `*`：preflight 放行任意 origin（方法/头仍校验）。
    let d = dev();
    let (status, _, vary) = d.cors.preflight(Some("http://x.io"), Some("GET"), &[]);
    assert_eq!(status, 204);
    assert!(!vary, "`*` 无 Vary");
    assert_eq!(
        d.cors.actual_request(Some("http://x.io")),
        CorsOutcome::Allowed {
            allow_origin: "*".into(),
            vary_origin: false
        }
    );
}

#[test]
fn t02_trusted_proxy_identity_rules() {
    let c = resolve_service_config(
        r#"{"profile":"proxy_service","allowed_hosts":["a"],"auth":{"responsibility":"app"},"trusted_proxy_ips":["127.0.0.1"]}"#,
        None,
    )
    .unwrap();
    let peer: std::net::IpAddr = "127.0.0.1".parse().unwrap();
    let (id, proto) = c.effective_client_identity(peer, Some("9.9.9.9"), Some("https"));
    assert_eq!(id.client.to_string(), "9.9.9.9");
    assert_eq!(proto.as_deref(), Some("https"));
    // 伪造链拒绝。
    let (id, _) = c.effective_client_identity(peer, Some("9.9.9.9, 8.8.8.8"), None);
    assert!(id.xff_rejected && id.client == peer);
}

#[test]
fn t02_rate_limiter_conservative_on_full_table() {
    let rl = RateLimiter::new(crate::http_service_config::RateLimitConfig {
        max_requests: 1,
        window_ms: 60_000,
        bucket_capacity: 1,
        bucket_ttl_ms: 1_000,
    });
    let a: std::net::IpAddr = "1.1.1.1".parse().unwrap();
    let b: std::net::IpAddr = "2.2.2.2".parse().unwrap();
    assert!(matches!(rl.check(a, 0), RateDecision::Allow));
    assert!(
        matches!(rl.check(b, 100), RateDecision::Limited { .. }),
        "满表新身份保守 429"
    );
    assert!(
        matches!(rl.check(a, 1_100), RateDecision::Allow),
        "TTL 回收后可进"
    );
}

#[test]
fn t02_service_json_round_trip_preserves_hash() {
    let c = resolve_service_config(
        r#"{"profile":"proxy_service","listen":{"port":9443},"allowed_hosts":["a.test"],"auth":{"responsibility":"edge"},"cors":{"allowed_origins":["https://app.test"]},"rate_limit":{"max_requests":10,"window_ms":1000}}"#,
        None,
    )
    .unwrap();
    let r = resolve_service_config(&c.to_json(), None).unwrap();
    assert_eq!(c.effective_config_hash(), r.effective_config_hash());
}

#[test]
fn t02_upload_facade_reachable_after_qualify() {
    // api_gen qualify_a2r_std 把上传 glue 的 `a2r_std::` 全量改写为
    // `crate::a2r_std::`——本测试锁定转发壳对 730 面完备（缺一项即
    // 生成上传后端 E0425，T-02 实测回归）。
    use crate::a2r_std::http as facade;
    // 符号存在性锁定（生成 glue 消费的面）；签名不在此钉——a2r-std 单源。
    // 具体签名函数（非泛型）直接锁；impl AsRef<str> 入口（upload_receive/
    // commit/reject/error）由生成 glue 编译面覆盖（真实 e2e T-06 兜底）。
    let _: fn(&facade::UploadSession) -> String = facade::upload_metadata;
    let _: fn(&facade::UploadSession) -> String = facade::upload_metadata_json;
    let _ = std::mem::size_of::<facade::UploadRequest>();
    let _ = std::mem::size_of::<facade::UploadReceipt>();
    let _ = std::mem::size_of::<facade::UploadSession>();
}

// ============================================================================
// http_e2e_plan736（真 TCP；P729-D1 drain 窗 wire）
// ============================================================================

const CRLF: &str = "\r\n";

fn raw_get(port: u16, path: &str) -> Option<String> {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).ok()?;
    let req = format!(
        "GET {path} HTTP/1.1{c}Host: 127.0.0.1{c}Connection: close{c}{c}",
        c = CRLF
    );
    stream.write_all(req.as_bytes()).ok()?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .ok()?;
    let mut buf = String::new();
    let _ = stream.read_to_string(&mut buf);
    Some(buf)
}

/// P729-D1 专项：在途文件 body 在 shutdown drain 窗内自然完成（729 在途 FS
/// 不中断）+ drain 期 ready 不再 200 + 端口释放重绑。真 TCP、注入关停（与
/// Ctrl+C 汇入同一 watch——headless 的确定性注入路径，实机信号面见 SD-01）。
#[test]
fn http_e2e_plan736_file_body_drain_window_and_rebind() {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let dir = std::env::temp_dir().join("plan736-drain-e2e");
    let _ = std::fs::create_dir_all(&dir);
    let blob = dir.join("drain-blob.bin");
    let payload: Vec<u8> = (0..64 * 1024).map(|i| (i % 251) as u8).collect();
    std::fs::write(&blob, &payload).unwrap();
    let dir_s = dir.to_string_lossy().replace('\\', "/");
    let port: u16 = 18610;

    let src = format!(
        r#"
#[api(method = "GET", path = "/api/blob")]
pub fn blob() FileResponse {{
    return http.file_response("{dir_s}", "drain-blob.bin", "{{}}")
}}

pub fn main() {{}}
"#,
        dir_s = dir_s
    );

    // 服务面装配（显式 config seam）：drain 窗 2s，port 固定。
    let cfg = std::sync::Arc::new(
        crate::http_service_config::resolve_service_config(
            r#"{"profile":"development","listen":{"port":18610},"limits":{"drain_timeout_ms":2000}}"#,
            None,
        )
        .unwrap(),
    );
    crate::vm::ffi::http_server::reset_serve_state();
    crate::http_service_config::set_active_service_config(cfg);
    let src_thread = src.clone();
    let server = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            if let Err(e) = crate::run(&src_thread) {
                eprintln!("[p736-e2e] run error: {}", e);
            }
        })
        .unwrap();

    // ready：health/ready 200（非纯 TCP——身份门）。
    let mut ready = false;
    for _ in 0..100 {
        if let Some(resp) = raw_get(port, "/__auto/health/ready") {
            if resp.starts_with("HTTP/1.1 200") {
                ready = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    assert!(ready, "service did not become ready");

    // 在途下载：读到 headers 末尾（\r\n\r\n），body 保持部分未读（在途）。
    let mut dl = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let req = format!(
        "GET /api/blob HTTP/1.1{c}Host: 127.0.0.1{c}Connection: close{c}{c}",
        c = CRLF
    );
    dl.write_all(req.as_bytes()).unwrap();
    dl.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut head_end = 0usize;
    let mut received: Vec<u8> = Vec::new();
    loop {
        let mut chunk = [0u8; 4096];
        let n = dl.read(&mut chunk).expect("read during flight");
        if n == 0 {
            panic!("stream ended before headers");
        }
        received.extend_from_slice(&chunk[..n]);
        if let Some(pos) = received.windows(4).position(|w| w == b"\r\n\r\n") {
            head_end = pos + 4;
            break;
        }
    }
    assert!(
        received.starts_with(b"HTTP/1.1 200"),
        "{}",
        String::from_utf8_lossy(&received[..head_end.min(received.len())])
    );
    let body_so_far = received.split_off(head_end);

    // 注入关停（同一 watch）：draining 期 ready 不得再 200。
    assert!(
        crate::vm::ffi::http_server::test_trigger_shutdown(),
        "shutdown trigger must be registered"
    );
    std::thread::sleep(Duration::from_millis(150));
    if let Some(resp) = raw_get(port, "/__auto/health/ready") {
        assert!(
            !resp.starts_with("HTTP/1.1 200"),
            "ready must not be 200 during drain: {}",
            &resp[..44.min(resp.len())]
        );
    }

    // 在途 body 必须完整收完（729 在途 FS 不中断；drain 窗 2s 覆盖本地读）。
    let mut rest = Vec::new();
    dl.read_to_end(&mut rest).expect("in-flight body completes");
    drop(dl);
    let mut full_body = body_so_far;
    full_body.extend_from_slice(&rest);
    assert_eq!(
        full_body.len(),
        payload.len(),
        "in-flight file body delivered in full across drain"
    );
    assert_eq!(full_body, payload, "in-flight body bytes identical");

    // 端口释放重绑（P729-D1 收尾面，与 699 同证）。
    server.join().unwrap();
    let mut rebound = false;
    for _ in 0..20 {
        match std::net::TcpListener::bind(("127.0.0.1", port)) {
            Ok(l) => {
                rebound = true;
                drop(l);
                break;
            }
            Err(_) => std::thread::sleep(Duration::from_millis(150)),
        }
    }
    assert!(rebound, "port must be releasable after drain completes");
}

#[test]
fn t02_active_config_seam_single_take() {
    let c = std::sync::Arc::new(dev());
    crate::http_service_config::set_active_service_config(c.clone());
    let taken = crate::http_service_config::take_active_service_config().expect("take once");
    assert_eq!(taken.effective_config_hash(), c.effective_config_hash());
    assert!(crate::http_service_config::take_active_service_config().is_none());
}
