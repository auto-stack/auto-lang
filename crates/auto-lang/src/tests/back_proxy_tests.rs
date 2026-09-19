//! PLAN-658 T-01: back proxy 骨架回归（真 TCP，`cargo th` 档）。
//!
//! 覆盖：①子前缀路由 + `#[api]` fn 动态分发（含跨请求 session 态存续——
//! module-level var 经 `__module_init` 激活后在请求间保持）；②按名参数
//! 绑定（POST body 字段 / query）；③404 面（未知 app / 未知路由）；
//! ④真实语料冒烟（020-music-player 的 `#[api]` status 路由经
//! `/apps/020-music-player/api/player/status` 返回 `[]`）。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;

use crate::back_proxy::{start, BackProxyConfig, SessionSpec};

/// 原始 socket HTTP 客户端（避免测试对 reqwest blocking 的额外依赖面）。
fn http_request(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    let (status, _headers, body) = http_request_raw(port, method, path, body, &[]);
    (status, String::from_utf8_lossy(&body).into_owned())
}

/// 带请求头与原始字节应答的形态（media 字节保真/Range 断言用）。
fn http_request_raw(
    port: u16,
    method: &str,
    path: &str,
    body: Option<&str>,
    req_headers: &[(&str, &str)],
) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect proxy");
    let body = body.unwrap_or("");
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (k, v) in req_headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str(&format!("\r\n{body}"));
    stream.write_all(req.as_bytes()).expect("write request");
    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line).expect("read status");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut content_length = 0usize;
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read header");
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                content_length = v.trim().parse().unwrap_or(0);
            }
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    let mut body_buf = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body_buf).expect("read body");
    }
    (status, headers, body_buf)
}

/// 临时 back 链（api.at + db.at，use 链 + module-level var + 增改路径）。
fn write_fixture(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("p658-back-proxy-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create fixture dir");
    std::fs::write(
        dir.join("db.at"),
        r#"
use api: Note

var notes List<Note> = List<Note>.new([
    Note { id: 1, title: "seed" },
])

pub fn all() []Note {
    return notes
}

pub fn add(title str) Note {
    let n = Note { id: notes.len() + 1, title: title }
    notes.push(n)
    return n
}
"#,
    )
    .expect("write db.at");
    std::fs::write(
        dir.join("api.at"),
        r#"
pub type Note = {
    id: int
    title: str
}

use db

#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note {
    return db.all()
}

#[api(method = "POST", path = "/api/notes")]
pub fn create_note(title str) Note {
    return db.add(title)
}
"#,
    )
    .expect("write api.at");
    dir
}

fn fixture_config(tag: &str, port: u16) -> (BackProxyConfig, PathBuf) {
    let dir = write_fixture(tag);
    let config = BackProxyConfig {
        port,
        sessions: vec![SessionSpec {
            app_id: "fixture".to_string(),
            back_entry: dir.join("api.at"),
        }],
        #[cfg(feature = "ui")]
        native_media: Vec::new(),
    };
    (config, dir)
}

/// 子前缀路由 + JSON 分发 + 跨请求 session 态存续。
#[test]
fn http_e2e_back_proxy_json_routes_and_session_state() {
    let (config, _dir) = fixture_config("state", 3958);
    let proxy = start(config).expect("start back proxy");

    // GET 列表：__module_init 激活的种子数据。
    let (status, body) = http_request(proxy.port, "GET", "/apps/fixture/api/notes", None);
    assert_eq!(status, 200, "seeded list status, body: {body}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&body).expect("seeded list json"),
        serde_json::json!([{"id": 1, "title": "seed"}]),
        "seeded list body: {body}"
    );

    // POST 按名绑定 body 字段 → db.add 命中 module-level var。
    let (status, body) = http_request(
        proxy.port,
        "POST",
        "/apps/fixture/api/notes",
        Some(r#"{"title":"second"}"#),
    );
    assert_eq!(status, 200, "create status, body: {body}");
    assert!(
        serde_json::from_str::<serde_json::Value>(&body)
            .expect("create json")
            .get("title")
            .and_then(|t| t.as_str())
            == Some("second"),
        "create body: {body}"
    );

    // 再 GET：session 态跨请求存续（不是每请求重装载）。
    let (status, body) = http_request(proxy.port, "GET", "/apps/fixture/api/notes", None);
    assert_eq!(status, 200);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&body).expect("persisted list json"),
        serde_json::json!([{"id": 1, "title": "seed"}, {"id": 2, "title": "second"}]),
        "state persists, body: {body}"
    );
}

/// 404 面：未知 app / 未知路由 / 前缀缺失。
#[test]
fn http_e2e_back_proxy_not_found_faces() {
    let (config, _dir) = fixture_config("404", 3968);
    let proxy = start(config).expect("start back proxy");

    let (status, body) = http_request(proxy.port, "GET", "/apps/unknown/api/notes", None);
    assert_eq!(status, 404, "unknown app, body: {body}");
    assert!(body.contains("unknown app"), "body: {body}");

    let (status, body) = http_request(proxy.port, "GET", "/apps/fixture/api/missing", None);
    assert_eq!(status, 404, "unknown route, body: {body}");
    assert!(body.contains("no route"), "body: {body}");

    let (status, _) = http_request(proxy.port, "GET", "/api/notes", None);
    assert_eq!(status, 404, "missing /apps prefix");
}

/// 缺参按名绑定的 400 面。
#[test]
fn http_e2e_back_proxy_missing_param_is_400() {
    let (config, _dir) = fixture_config("400", 3978);
    let proxy = start(config).expect("start back proxy");
    let (status, body) = http_request(proxy.port, "POST", "/apps/fixture/api/notes", Some("{}"));
    assert_eq!(status, 400, "missing param, body: {body}");
    assert!(body.contains("missing param `title`"), "body: {body}");
}

/// 真实语料冒烟（T-01 验收）：020-music-player 的 #[api] status 路由经
/// 子前缀返回真实状态（该 fn 返回空 PlayerInfo 列表——真实执行语义）。
#[test]
fn http_e2e_back_proxy_real_020_status_route() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest.ancestors().nth(2).expect("repo root").to_path_buf();
    let entry = repo_root.join("examples/ui/020-music-player/src/back/api.at");
    if !entry.exists() {
        // 仓外发布形态（语料不在）——跳过而非假绿。
        eprintln!("skip: {} not present", entry.display());
        return;
    }
    let config = BackProxyConfig {
        port: 3988,
        sessions: vec![SessionSpec {
            app_id: "020-music-player".to_string(),
            back_entry: entry,
        }],
        #[cfg(feature = "ui")]
        native_media: Vec::new(),
    };
    let proxy = start(config).expect("start back proxy for 020");
    let (status, body) =
        http_request(proxy.port, "GET", "/apps/020-music-player/api/player/status", None);
    assert_eq!(status, 200, "020 status route, body: {body}");
    assert_eq!(body, "[]", "020 player/status returns empty PlayerInfo list");
}

/// PLAN-658 T-03: 原生 media 路由——scan 绝对 URL + 字节保真 + Range 语义。
/// （cfg ui：media_service 在 ui 门后；th 档已含 ui feature。）
#[cfg(feature = "ui")]
#[test]
fn http_e2e_back_proxy_native_media_routes() {
    let dir = std::env::temp_dir().join(format!(
        "p658-back-proxy-media-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("media dir");
    let payload: Vec<u8> = b"ID3-fake-mp3-payload-0123456789abcdef".to_vec();
    std::fs::write(dir.join("song.mp3"), &payload).expect("write mp3");

    let config = BackProxyConfig {
        port: 3998,
        sessions: Vec::new(),
        native_media: vec![crate::back_proxy::NativeMediaApp {
            app_id: "020-t".to_string(),
            media_root: Some(dir.to_string_lossy().into_owned()),
        }],
    };
    let proxy = start(config).expect("start back proxy (media)");

    // scan：条目齐 + 绝对 url（画廊内嵌前端无 env 展开，§5.3）。
    let (status, body) = http_request(proxy.port, "GET", "/apps/020-t/api/media/scan", None);
    assert_eq!(status, 200, "scan status, body: {body}");
    let scan: serde_json::Value = serde_json::from_str(&body).expect("scan json");
    let entries = scan["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 1, "one fixture file");
    let entry = &entries[0];
    assert_eq!(entry["extension"].as_str(), Some("mp3"));
    assert_eq!(entry["bytes"].as_u64(), Some(payload.len() as u64));
    let id = entry["id"].as_str().expect("id").to_string();
    let expected_url = format!("http://127.0.0.1:{}/apps/020-t/api/media/stream/{id}", proxy.port);
    assert_eq!(entry["url"].as_str(), Some(expected_url.as_str()), "absolute url");
    // 原始 socket 请求用 path 段（绝对 URL 是浏览器/reqwest 展开后的形态）。
    let stream_path = format!("/apps/020-t/api/media/stream/{id}");

    // 全量流：200 + 字节保真 + Content-Type。
    let (status, headers, bytes) = http_request_raw(proxy.port, "GET", &stream_path, None, &[]);
    assert_eq!(status, 200, "full stream status");
    assert_eq!(bytes, payload, "byte fidelity");
    let ct = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    assert_eq!(ct, "audio/mpeg", "content type");

    // 区间流：206 + 窗口字节 + Content-Range。
    let (status, headers, bytes) = http_request_raw(
        proxy.port,
        "GET",
        &stream_path,
        None,
        &[("Range", "bytes=4-11")],
    );
    assert_eq!(status, 206, "partial status");
    assert_eq!(bytes, &payload[4..=11], "window bytes");
    let cr = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-range"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    assert_eq!(
        cr,
        format!("bytes 4-11/{}", payload.len()),
        "content range"
    );

    // 未知 id → 404；未配置 root 的 app → 诚实空列表。
    let (status, body) = http_request(
        proxy.port,
        "GET",
        "/apps/020-t/api/media/stream/deadbeef",
        None,
    );
    assert_eq!(status, 404, "unknown id, body: {body}");
}

/// PLAN-658 T-03: 未配置 media root 的 app——诚实空列表（与生成器
/// auto_media_scan 三态语义一致），不是 500。
#[cfg(feature = "ui")]
#[test]
fn http_e2e_back_proxy_native_media_no_root_honest_empty() {
    let config = BackProxyConfig {
        port: 3999,
        sessions: Vec::new(),
        native_media: vec![crate::back_proxy::NativeMediaApp {
            app_id: "no-root-t".to_string(),
            media_root: Some("Z:/definitely/not/a/real/dir".to_string()),
        }],
    };
    let proxy = start(config).expect("start back proxy (no root)");
    let (status, body) = http_request(proxy.port, "GET", "/apps/no-root-t/api/media/scan", None);
    assert_eq!(status, 200, "no-root scan status");
    assert_eq!(body, "{\"entries\":[],\"root_missing\":true}", "honest empty");
}

/// PLAN-658 T-04 探针（临时）：017-chat 真实 back（含 ~Stream 死码体
/// bus.subscribe()）能否装载进 session 并应答 CRUD 路由。
#[test]
fn http_e2e_back_proxy_real_017_crud_probe() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest.ancestors().nth(2).expect("repo root").to_path_buf();
    let entry = repo_root.join("examples/ui/017-chat/src/back/api.at");
    if !entry.exists() {
        eprintln!("skip: {} not present", entry.display());
        return;
    }
    let config = BackProxyConfig {
        port: 3978,
        sessions: vec![SessionSpec {
            app_id: "017-chat".to_string(),
            back_entry: entry,
        }],
        #[cfg(feature = "ui")]
        native_media: Vec::new(),
    };
    let proxy = start(config).expect("start proxy 017");
    let (status, body) = http_request(proxy.port, "GET", "/apps/017-chat/api/contacts", None);
    assert_eq!(status, 200, "017 contacts, body: {body}");
    assert!(body.contains("Alice"), "contacts seeded: {body}");
    let (status, body) = http_request(
        proxy.port,
        "POST",
        "/apps/017-chat/api/messages",
        Some(r#"{"sender":"You","text":"hello from proxy"}"#),
    );
    assert_eq!(status, 200, "017 send, body: {body}");
    assert!(body.contains("hello from proxy"), "created msg: {body}");
    let (status, body) = http_request(proxy.port, "GET", "/apps/017-chat/api/messages", None);
    assert_eq!(status, 200);
    assert!(body.contains("hello from proxy"), "persisted: {body}");
}

/// PLAN-658 T-04: stream 一等公民——真 017 语料经 proxy 的 SSE 转发：
/// 订阅 /api/stream 长连接后，POST typing → Typing 帧、POST messages →
/// NewMessage 帧实时到达（~Stream 端点按签名特路 + 宿主总线广播）。
#[test]
fn http_e2e_back_proxy_real_017_sse_stream() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest.ancestors().nth(2).expect("repo root").to_path_buf();
    let entry = repo_root.join("examples/ui/017-chat/src/back/api.at");
    if !entry.exists() {
        eprintln!("skip: {} not present", entry.display());
        return;
    }
    let config = BackProxyConfig {
        port: 3968,
        sessions: vec![SessionSpec {
            app_id: "017-chat".to_string(),
            back_entry: entry,
        }],
        #[cfg(feature = "ui")]
        native_media: Vec::new(),
    };
    let proxy = start(config).expect("start proxy 017 sse");

    // 打开 SSE 长连接（独立 socket，读超时防挂死）。
    let mut sse = TcpStream::connect(("127.0.0.1", proxy.port)).expect("connect sse");
    sse.set_read_timeout(Some(std::time::Duration::from_secs(8))).unwrap();
    let req = "GET /apps/017-chat/api/stream HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept: text/event-stream\r\nConnection: close\r\n\r\n";
    sse.write_all(req.as_bytes()).unwrap();
    let mut reader = BufReader::new(sse.try_clone().unwrap());
    let mut status_line = String::new();
    reader.read_line(&mut status_line).unwrap();
    assert!(status_line.contains("200"), "sse status: {status_line}");
    let mut content_type = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line == "\r\n" {
            break;
        }
        if line.to_ascii_lowercase().starts_with("content-type") {
            content_type = line.trim().to_string();
        }
    }
    assert!(
        content_type.contains("text/event-stream"),
        "sse content-type: {content_type}"
    );

    let read_frame = |reader: &mut BufReader<TcpStream>| -> String {
        let mut frame = String::new();
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line).expect("read frame line");
            assert!(n > 0, "sse stream closed prematurely");
            frame.push_str(&line);
            if frame.ends_with("\n\n") {
                return frame;
            }
        }
    };

    // 触发两族事件，帧实时到达。
    let (status, body) = http_request(
        proxy.port,
        "POST",
        "/apps/017-chat/api/typing",
        Some(r#"{"sender":"Alice"}"#),
    );
    assert_eq!(status, 200, "typing status, body: {body}");
    let f1 = read_frame(&mut reader);
    assert!(
        f1.contains("\"event\":\"Typing\"") && f1.contains("Alice"),
        "typing frame: {f1}"
    );

    let (status, body) = http_request(
        proxy.port,
        "POST",
        "/apps/017-chat/api/messages",
        Some(r#"{"sender":"You","text":"sse hello"}"#),
    );
    assert_eq!(status, 200, "send status, body: {body}");
    let f2 = read_frame(&mut reader);
    assert!(
        f2.contains("\"event\":\"NewMessage\"") && f2.contains("sse hello"),
        "new-message frame: {f2}"
    );
}

/// PLAN-658 T-05: 031-image-viewer 真实 back（use auto.image）装载进
/// session——auto.image 原生经进程级注册表在 session 内可用，open/snapshot
/// 走真实 image_pipeline（fixtures 目录）。
#[cfg(feature = "ui-iced")]
#[test]
fn http_e2e_back_proxy_real_031_native_ns_session() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest.ancestors().nth(2).expect("repo root").to_path_buf();
    let base = repo_root.join("examples/ui/031-image-viewer");
    if !base.join("src/back/api.at").exists() {
        eprintln!("skip: 031 corpus not present");
        return;
    }
    let config = BackProxyConfig {
        port: 3958,
        sessions: vec![SessionSpec {
            app_id: "031-image-viewer".to_string(),
            back_entry: base.join("src/back/api.at"),
        }],
        native_media: Vec::new(),
    };
    let proxy = start(config).expect("start proxy 031");

    // health：纯字面路由。
    let (status, body) = http_request(proxy.port, "GET", "/apps/031-image-viewer/api/viewer/health", None);
    assert_eq!(status, 200, "health, body: {body}");
    assert!(body.contains("Ready"), "health body: {body}");

    // open-directory：session 内 image.open_session 真实执行（fixtures 目录）。
    let fixtures = repo_root
        .join("examples/ui/031-image-viewer/tests/fixtures")
        .to_string_lossy()
        .replace('\\', "/");
    let (status, body) = http_request(
        proxy.port,
        "POST",
        "/apps/031-image-viewer/api/viewer/open-directory",
        Some(&format!("{{\"path\":\"{fixtures}\"}}")),
    );
    assert_eq!(status, 200, "open-directory, body: {body}");
    let sess: serde_json::Value = serde_json::from_str(&body).expect("session handle json");

    // names：目录条目（GET + query 绑定 session；jpg/png 过滤后非空）。
    let sid = sess.as_str().unwrap_or_default().to_string();
    let (status, body) = http_request(
        proxy.port,
        "GET",
        &format!("/apps/031-image-viewer/api/viewer/names?session={sid}"),
        None,
    );
    assert_eq!(status, 200, "names, body: {body}");
    assert!(body.len() > 10, "names non-empty: {body}");
    println!("031 names sample: {body}");
}

/// PLAN-658 T-05: `/api/__auto/media/{id}/{rev}` 字节保真——经 image_pipeline
/// registry 直接发布资产，proxy 转发路由逐字节一致 + Content-Type 保真
/// + ETag/304 语义（AC-03 字节级校验面）。
#[cfg(feature = "ui-iced")]
#[test]
fn http_e2e_back_proxy_media_uri_byte_fidelity() {
    use crate::ui::image_pipeline::{
        global_media_registry, MediaAssetKey, MediaAssetState, MediaMetadata, RenditionSpec,
    };
    let registry = global_media_registry();
    let ticket = registry.queue(
        MediaAssetKey {
            source_fingerprint: "p658-proxy-fixture".into(),
            orientation: Default::default(),
            rendition: RenditionSpec::original(),
            revision: 1,
        },
        MediaMetadata {
            mime_type: "image/png".to_string(),
            ..Default::default()
        },
    );
    registry.transition(ticket.id, MediaAssetState::Reading).unwrap();
    registry.transition(ticket.id, MediaAssetState::Decoding).unwrap();
    registry.transition(ticket.id, MediaAssetState::Transforming).unwrap();
    let payload: std::sync::Arc<[u8]> = std::sync::Arc::from([7u8, 255, 0, 128, 42, 99]);
    registry
        .publish_ready(ticket.id, ticket.revision, payload.clone())
        .unwrap();

    let config = BackProxyConfig {
        port: 3938,
        sessions: Vec::new(),
        native_media: Vec::new(),
    };
    let proxy = start(config).expect("start proxy (media uri)");
    let path = format!("/apps/031-image-viewer/api/__auto/media/{}/{}", ticket.id, ticket.revision);

    // GET：字节一致 + Content-Type。
    let (status, headers, bytes) = http_request_raw(proxy.port, "GET", &path, None, &[]);
    assert_eq!(status, 200, "media uri status");
    assert_eq!(&bytes[..], &payload[..], "byte fidelity");
    let ct = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    assert_eq!(ct, "image/png", "content type preserved");

    // ETag 命中 → 304。
    let etag = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("etag"))
        .map(|(_, v)| v.clone());
    if let Some(etag) = etag {
        let (status, _, _) = http_request_raw(
            proxy.port,
            "GET",
            &path,
            None,
            &[("If-None-Match", etag.as_str())],
        );
        assert_eq!(status, 304, "etag revalidation");
    }
}

/// PLAN-658 T-06 / AC-05: 崩溃隔离——单 app 后端 panic 注入 → 500 可诊断
/// 应答 + session 退避重启（状态归零）+ 宿主与其他 session 存活 + 日志环
/// 记录 PANIC/RESTART。
#[test]
fn http_e2e_back_proxy_panic_isolation_and_restart() {
    let dir = std::env::temp_dir().join(format!("p658-back-proxy-panic-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("panic fixture dir");
    std::fs::write(
        dir.join("api.at"),
        r#"
var hits int = 0

#[api(method = "GET", path = "/api/hits")]
pub fn hits_fn() int {
    return hits
}

#[api(method = "POST", path = "/api/bump")]
pub fn bump() int {
    hits = hits + 1
    return hits
}

#[api(method = "GET", path = "/api/boom")]
pub fn boom() str {
    sys.panic_hard("kaboom-p658")
    return "unreachable"
}
"#,
    )
    .expect("write panic api.at");

    let (healthy_dir, _) = {
        let d = std::env::temp_dir().join(format!("p658-back-proxy-panic2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("dir2");
        std::fs::write(
            d.join("api.at"),
            "#[api(method = \"GET\", path = \"/api/ping\")]\npub fn ping() str {\n    return \"pong\"\n}\n",
        )
        .expect("write healthy api.at");
        (d, ())
    };

    let config = BackProxyConfig {
        port: 3928,
        sessions: vec![
            SessionSpec {
                app_id: "boom-app".to_string(),
                back_entry: dir.join("api.at"),
            },
            SessionSpec {
                app_id: "healthy-app".to_string(),
                back_entry: healthy_dir.join("api.at"),
            },
        ],
        #[cfg(feature = "ui")]
        native_media: Vec::new(),
    };
    let proxy = start(config).expect("start proxy (panic isolation)");

    // 前置：bump ×2 使会话态可观测（hits=2）。
    let (s, _) = http_request(proxy.port, "POST", "/apps/boom-app/api/bump", Some("{}"));
    assert_eq!(s, 200);
    let (s, _) = http_request(proxy.port, "POST", "/apps/boom-app/api/bump", Some("{}"));
    assert_eq!(s, 200);
    let (s, body) = http_request(proxy.port, "GET", "/apps/boom-app/api/hits", None);
    assert_eq!(s, 200);
    assert_eq!(body.trim_matches(|c| c == '"'), "2", "hits state: {body}");

    // panic 注入：500 + 可诊断消息 + （退避 500ms + 重建）随后恢复且状态归零。
    let t0 = std::time::Instant::now();
    let (s, body) = http_request(proxy.port, "GET", "/apps/boom-app/api/boom", None);
    let elapsed = t0.elapsed();
    assert_eq!(s, 500, "panic status, body: {body}");
    assert!(body.contains("kaboom-p658"), "panic message: {body}");
    assert!(body.contains("restart attempted"), "restart hint: {body}");
    assert!(elapsed.as_millis() >= 400, "backoff applied: {elapsed:?}");

    // 宿主与其他 session 存活。
    let (s, body) = http_request(proxy.port, "GET", "/apps/healthy-app/api/ping", None);
    assert_eq!(s, 200, "other session alive");
    assert_eq!(body.trim_matches(|c| c == '"'), "pong");

    // 重启后状态归零（hits 回 0）+ 路由恢复。
    let (s, body) = http_request(proxy.port, "GET", "/apps/boom-app/api/hits", None);
    assert_eq!(s, 200, "boom-app recovered, body: {body}");
    assert_eq!(body.trim_matches(|c| c == '"'), "0", "state reset after restart: {body}");

    // 日志环：PANIC + RESTART 记录。
    let (s, body) = http_request(proxy.port, "GET", "/__backproxy/log?app=boom-app", None);
    assert_eq!(s, 200);
    assert!(body.contains("PANIC kaboom-p658"), "log ring panic: {body}");
    assert!(body.contains("RESTART"), "log ring restart: {body}");
}
