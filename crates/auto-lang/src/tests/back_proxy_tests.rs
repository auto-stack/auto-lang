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
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect proxy");
    let body = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
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
        }
    }
    let mut body_buf = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body_buf).expect("read body");
    }
    (status, String::from_utf8_lossy(&body_buf).into_owned())
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
    };
    let proxy = start(config).expect("start back proxy for 020");
    let (status, body) =
        http_request(proxy.port, "GET", "/apps/020-music-player/api/player/status", None);
    assert_eq!(status, 200, "020 status route, body: {body}");
    assert_eq!(body, "[]", "020 player/status returns empty PlayerInfo list");
}
