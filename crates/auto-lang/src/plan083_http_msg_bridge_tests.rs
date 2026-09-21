//! PLAN-083 T-01：异步 HTTP 消息桥（`Http.get_msg`）单测。
//!
//! 覆盖（U-3 的 native 层证据）：
//! 1. 派生线程完成项入队——真实 TcpListener 服务一帧 JSON 响应，spawn 后
//!    轮询队列取到 `{widget, event, payload}`，payload 为
//!    `{"ok":true,"status":200,"body":...}`。
//! 2. 失败臂——指向已关闭端口，完成项仍入队且 `ok:false` / `status:0`
//!    （回填 handler 可区分失败与空体）。
//! 3. 命名空间切分——`"Store.Handler"` → (Store, Handler)；无点 → 根。
//! 4. 载荷编码往返——渲染层 `Event␟s␟<payload>` 串经
//!    `dynamic::decode_payload` 解出 handler 名 + 单字符串实参（回填派发
//!    契约，对齐 shell SSE 桥先例）。
//!
//! 发起段"不阻塞"为结构性事实（shim 不置 task Waiting、无忙等段），实机
//! 可响应性由 PLAN-083 T-05 V-1 验收（musk 切 workspace 实测）。

use crate::vm::ffi::stdlib::{http_msg_poll_one, http_msg_queue_clear, http_msg_split_target};

/// 进程级队列是共享态——两个 spawn 测试互斥串行（并行跑会互洗队列/互取
/// 对方完成项，首跑实测 2 红复跑全绿的竞态即此）。
fn queue_test_lock() -> &'static std::sync::Mutex<()> {
    static M: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    M.get_or_init(|| std::sync::Mutex::new(()))
}

/// 起一个一次性 TcpListener 服务一帧固定 JSON 响应，返回端口。
fn serve_once(body: &'static str) -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            use std::io::{Read, Write};
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf); // 请求行+头（一帧读完即答）
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });
    port
}

/// 限时轮询队列（派生线程异步完成）。
fn poll_with_deadline(deadline_ms: u64) -> Option<crate::vm::ffi::stdlib::HttpMsgDone> {
    let t0 = std::time::Instant::now();
    loop {
        if let Some(d) = http_msg_poll_one() {
            return Some(d);
        }
        if t0.elapsed() > std::time::Duration::from_millis(deadline_ms) {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn spawn_completes_into_queue_with_json_payload() {
    let _guard = queue_test_lock().lock().unwrap();
    http_msg_queue_clear();
    let port = serve_once(r#"{"sessions":[]}"#);
    // 直接驱动派生线程体（shim 薄封装：pop 两参 + 切分 + spawn + push 0）。
    crate::vm::ffi::stdlib::shim_http_get_msg_spawn_for_test(
        &format!("http://127.0.0.1:{}/api/chats/sessions", port),
        "ForgeStore.SessionsLoaded",
    );
    let done = poll_with_deadline(5000).expect("completion should arrive in queue");
    assert_eq!(done.widget, "ForgeStore");
    assert_eq!(done.event, "SessionsLoaded");
    let v: serde_json::Value = serde_json::from_str(&done.payload).expect("payload is JSON");
    assert_eq!(v["ok"], serde_json::json!(true));
    assert_eq!(v["status"], serde_json::json!(200));
    assert_eq!(v["body"], serde_json::json!(r#"{"sessions":[]}"#));
}

#[test]
fn failure_yields_ok_false_and_still_dispatches() {
    let _guard = queue_test_lock().lock().unwrap();
    http_msg_queue_clear();
    // 已关闭端口：bind 后立即 drop，连接必拒。
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let p = l.local_addr().unwrap().port();
        drop(l);
        p
    };
    crate::vm::ffi::stdlib::shim_http_get_msg_spawn_for_test(
        &format!("http://127.0.0.1:{}/x", port),
        "Root",
    );
    let done = poll_with_deadline(15000).expect("failure completion should arrive too");
    assert_eq!(done.widget, "");
    assert_eq!(done.event, "Root");
    let v: serde_json::Value = serde_json::from_str(&done.payload).expect("payload is JSON");
    assert_eq!(v["ok"], serde_json::json!(false));
    assert_eq!(v["status"], serde_json::json!(0));
    assert!(v["body"].as_str().is_some(), "error text carried in body");
}

#[test]
fn namespace_split_rules() {
    assert_eq!(
        http_msg_split_target("ForgeStore.SessionsLoaded"),
        ("ForgeStore".to_string(), "SessionsLoaded".to_string())
    );
    // 无点 = 根 handler。
    assert_eq!(
        http_msg_split_target("Root"),
        (String::new(), "Root".to_string())
    );
    // 空段防御。
    assert_eq!(
        http_msg_split_target(".Leading"),
        (String::new(), ".Leading".to_string())
    );
    assert_eq!(
        http_msg_split_target("Trailing."),
        (String::new(), "Trailing.".to_string())
    );
}

#[test]
fn payload_encoding_roundtrips_through_decode_payload() {
    // 渲染层泵拼的 IcedMessage.event 形态（poll_http_msgs 同式）。
    let payload = r#"{"ok":true,"status":200,"body":"{\"sessions\":[]}"}"#;
    let event = format!("SessionsLoaded\u{1F}s\u{1F}{}", payload);
    let (name, args) = crate::ui::dynamic::decode_payload(&event);
    assert_eq!(name, "SessionsLoaded");
    assert_eq!(args.len(), 1);
    match args.first() {
        Some(auto_val::Value::Str(s)) => assert_eq!(s.as_str(), payload),
        other => panic!("expected single Str arg, got {:?}", other),
    }
}

// ─── U-3 模拟器探针：两段式全链（.at 语料 → 发起段非阻塞 → 泵回填） ───────

#[cfg(all(test, feature = "ui-interpreter"))]
mod two_phase_probe {
    use super::{poll_with_deadline, queue_test_lock, serve_once};

    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan083_http_msg/src/front/app.at";
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(format!("../../{}", rel))),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    /// 全链：App.Kick → SpkStore.Fetch（Http.get_msg 入队即返，phase=
    /// loading 可探测=发起段非阻塞证据）→ 派生线程完成 → 模拟渲染层泵
    /// （poll_one + "Loaded␟s␟payload"）→ Loaded 落态 done。
    #[test]
    fn two_phase_load_roundtrip_through_real_dispatch() {
        let _guard = queue_test_lock().lock().unwrap();
        crate::vm::ffi::stdlib::http_msg_queue_clear();

        let marker = r#"{"marker":"plan083-spike"}"#;
        let port = serve_once(marker);
        let Some(mut dc) = crate::plan370_test_support::build_component_from_app(
            &locate_corpus().expect("plan083 corpus present"),
        ) else {
            eprintln!("plan083 T-01: SKIPPED — corpus not found");
            return;
        };

        // 注入动态端口 URL（语料源静态串；store 字段落根态，write_state 可达）。
        dc.write_state(
            "probe_url",
            auto_val::Value::str(&format!("http://127.0.0.1:{}/probe.json", port)),
        )
        .expect("seed probe_url");

        // 发起段：handler 当场结束——read_state 立即可见 loading（若 Http.get_msg
        // 走 get_json 族 re-entry yield，此处 call_handler 会忙等到响应返回才能
        // 继续——loading 探测即非阻塞判据）。
        dc.on_with_input_for("", "Kick", None);
        assert_eq!(
            dc.read_state("phase").map(|v| v.as_str().to_string()),
            Ok("loading".to_string()),
            "发起段立即返回：loading 态可探测而请求仍在途"
        );

        // 回填段：模拟渲染层泵（poll_http_msgs 同式拼载荷 → on_with_input_for）。
        let done = poll_with_deadline(5000).expect("completion lands in queue");
        assert_eq!(done.widget, "SpkStore");
        assert_eq!(done.event, "Loaded");
        let event = format!("{}\u{1F}s\u{1F}{}", done.event, done.payload);
        dc.on_with_input_for(&done.widget, &event, None);

        assert_eq!(
            dc.read_state("phase").map(|v| v.as_str().to_string()),
            Ok("done".to_string()),
            "回填 handler 到达并落态"
        );
        let body = dc
            .read_state("last_body")
            .map(|v| v.as_str().to_string())
            .expect("last_body readable");
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("payload is JSON");
        assert_eq!(parsed["ok"], serde_json::json!(true));
        assert_eq!(parsed["body"], serde_json::json!(marker));
    }
}
