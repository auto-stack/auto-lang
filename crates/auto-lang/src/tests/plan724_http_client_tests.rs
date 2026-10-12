//! PLAN-724：a2r HTTP 客户端收敛的转译探针与 golden。
//!
//! - golden：`test/a2r/30_plan724/00{1,2}_http_client_{sync,async}/` 冻结
//!   发射形态（test-convention：golden 只验证 Rust 文本）。
//! - 编译运行腿：`http_e2e_plan724_*`（`test-http-e2e` feature 守卫，真
//!   TCP + 临时 cargo 工程，见 `http_e2e_plan724_client_matrix`）。
//!
//! 分派矩阵冻结见 `docs/plans/reports/724-client-decision.md`。

use crate::{error::AutoResult, trans::rust::transpile_rust_with_source_dir};
use std::fs::read_to_string;
use std::path::PathBuf;

const SYNC_CASE_DIR: &str = "test/a2r/30_plan724/001_http_client_sync";
const ASYNC_CASE_DIR: &str = "test/a2r/30_plan724/002_http_client_async";

fn transpile_case(case_dir: &str, name: &str) -> AutoResult<String> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = read_to_string(d.join(case_dir).join(format!("{name}.at")))?;
    let mut rcode = transpile_rust_with_source_dir(&d.join(case_dir), name, &src)?;
    let code = rcode.done()?;
    Ok(String::from_utf8_lossy(code).into_owned())
}

fn assert_golden(case_dir: &str, name: &str) {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exp_path = d.join(case_dir).join(format!("{name}.expected.rs"));
    let expected = if exp_path.is_file() {
        read_to_string(&exp_path).unwrap()
    } else {
        String::new()
    };
    let actual = transpile_case(case_dir, name).expect("transpile ok");
    if actual != expected {
        let wrong = d.join(case_dir).join(format!("{name}.wrong.rs"));
        std::fs::write(&wrong, &actual).unwrap();
        panic!(
            "golden mismatch for {case_dir}; product written to {}",
            wrong.display()
        );
    }
}

/// T-06 golden：同步上下文全矩阵发射形态冻结。
#[test]
fn plan724_golden_http_client_sync() {
    assert_golden(SYNC_CASE_DIR, "http_client_sync");
}

/// T-06 golden：async 上下文全矩阵发射形态冻结（~helper/.await/stream）。
#[test]
fn plan724_golden_http_client_async() {
    assert_golden(ASYNC_CASE_DIR, "http_client_async");
}

/// T-06 发射断言（golden 之外的关键形状钉，防 expected 漂移掩盖回归）：
/// async 上下文两参 post 必须走 post_async。三参认证面（arity-dispatch
/// 解构壳）在 PLAN-738 T-10 strict 门下按运行形态分家：producer（3 参
/// async post）只存在于内嵌镜像（crates/auto-lang/src/a2r_std.rs），故
/// Standalone 转译不再发射该壳——a2r-std crate 本就无此符号，旧文本
/// 断言冻结的是一个从未可编译的形状。合法认证用法在 Standalone 下走
/// post_sync/post_sync_async 侧信道面（golden + e2e 在案）；Embedded
/// 元数分派的完整证明见 plan738_host::
/// arity_dispatch_post_proves_only_under_embedded。
#[test]
fn plan724_probe_post_arity_dispatch_shapes() {
    let async_src = r#"
fn fetch(url str) ~str {
    let res = http.post(url, "x")
    return str.from_bytes(res.body_bytes())
}
fn main() {
    let b = http.post("http://x", "y")
    let c = fetch("http://x").await
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("probe", async_src).unwrap();
    let code = String::from_utf8_lossy(rcode.done().unwrap()).into_owned();
    // 两参 post（async main）→ post_async
    assert!(
        code.contains("a2r_std::http::post_async("),
        "2-arg post in async ctx must lower to post_async:\n{code}"
    );
    // 三参认证面（Standalone）→ strict 门诚实拒绝（a2r-std 无 3 参 post）。
    let auth_src = r#"
fn main() {
    let a = http.post("http://x", "y", "key")
}
"#;
    let rejected = match crate::trans::rust::transpile_rust("probe_auth", auth_src) {
        Err(e) => e,
        Ok(_) => panic!("3-arg post under standalone must be rejected, got a product"),
    };
    assert!(
        rejected
            .to_string()
            .contains("STDASSEMBLY.SIGNATURE_UNVERIFIED"),
        "3-arg post under standalone must be rejected honestly: {rejected}"
    );
    assert!(
        rejected.to_string().contains("arity-dispatch"),
        "{rejected}"
    );
}

/// T-06 发射断言：同步上下文 builder `.send()` 保持同步形态；async 上下文
/// 变量绑定 builder 的 `.send()` 升级为 `send_async().await`。
#[test]
fn plan724_probe_builder_send_context_dispatch() {
    let sync_src = r#"
fn main() {
    let b = http.request("GET", "http://x")
    let r = b.send()
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("probe_sync", sync_src).unwrap();
    let code = String::from_utf8_lossy(rcode.done().unwrap()).into_owned();
    assert!(code.contains(".send()"), "sync builder send:\n{code}");
    assert!(
        !code.contains("send_async"),
        "sync ctx must not emit send_async:\n{code}"
    );

    let async_src = r#"
fn tick() ~str {
    return "t"
}
fn main() {
    let w = tick().await
    let b = http.request("GET", "http://x")
    let r = b.send()
    let c = http.get("http://x")
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("probe_async", async_src).unwrap();
    let code = String::from_utf8_lossy(rcode.done().unwrap()).into_owned();
    assert!(
        code.contains(".send_async().await"),
        "async builder send must await:\n{code}"
    );
    assert!(
        code.contains("a2r_std::http::get_async("),
        "async ctx get must use async face:\n{code}"
    );
}

/// T-06 发射断言：流自由函数按变量分型选择同步/异步 facade。
#[test]
fn plan724_probe_stream_free_fn_typed_dispatch() {
    let sync_src = r#"
fn main() {
    let s = http.get_stream("http://x")
    let c = stream_next(s)
    let d = stream_is_done(s)
    stream_close(s)
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("probe_stream_sync", sync_src).unwrap();
    let code = String::from_utf8_lossy(rcode.done().unwrap()).into_owned();
    assert!(
        code.contains("a2r_std::http::stream_next(&"),
        "sync stream_next free fn:\n{code}"
    );

    let async_src = r#"
fn tick() ~str {
    return "t"
}
fn main() {
    let w = tick().await
    let s = http.get_stream("http://x")
    let c = stream_next(s)
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("probe_stream_async", async_src).unwrap();
    let code = String::from_utf8_lossy(rcode.done().unwrap()).into_owned();
    assert!(
        code.contains("a2r_std::http::stream_next_async(&"),
        "async-typed stream var must use async free fn:\n{code}"
    );
}

// ============================================================================
// 编译运行腿（真 TCP + cargo 工程）：test-http-e2e 门（cargo th 过滤器收集）。
// ============================================================================

#[cfg(feature = "test-http-e2e")]
mod e2e {
    use super::*;

    /// 回环 stub：GET /text（200 + 自定义头 + body）、POST /echo（回显 body
    /// 与请求头）、GET /stream（分帧 drip）。有限 accept，os 临时端口。
    fn spawn_stub() -> (u16, std::sync::mpsc::Receiver<()>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for _ in 0..12 {
                let (mut s, _) = match listener.accept() {
                    Ok(x) => x,
                    Err(_) => break,
                };
                let mut buf = [0u8; 8192];
                let n = match s.read(&mut buf) {
                    Ok(n) => n,
                    Err(_) => break,
                };
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                // 按方法路由：POST → 回显 body（builder/post/auth 共用）；
                // GET → text + 自定义头（普通 get 与流共用——流分帧由内核
                // 单测覆盖，此处验证端到端读取）。
                let is_post = req.starts_with("POST ");
                if is_post {
                    let payload = req
                        .split("\r\n\r\n")
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let body = format!("echo:{payload}");
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = s.write_all(resp.as_bytes());
                } else {
                    let body = b"plan724-ok";
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nX-Plan724: alpha\r\nContent-Length: {}\r\n\r\n",
                        body.len()
                    );
                    let _ = s.write_all(resp.as_bytes());
                    let _ = s.write_all(body);
                }
                let _ = s.flush();
            }
            let _ = done_tx.send(());
        });
        (port, done_rx)
    }

    /// 编译运行基建：临时 cargo 工程 path 依赖 worktree a2r-std，内容哈希
    /// 复用（同源产物二次运行秒级）。
    fn build_and_run(name: &str, product: &str) -> String {
        let hash = format!("{:x}", md5_compat(product));
        let proj = std::env::temp_dir().join(format!("plan724-e2e-{name}-{hash}"));
        std::fs::create_dir_all(proj.join("src")).unwrap();
        let manifest = format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
             [dependencies]\na2r-std = {{ path = {:?} }}\ntokio = {{ version = \"1\", features = [\"full\"] }}\n\n\
             [[bin]]\nname = \"{name}\"\npath = \"src/main.rs\"\n\n\
             [workspace]\n",
            a2r_std_manifest_dir()
        );
        let manifest_path = proj.join("Cargo.toml");
        let existing = std::fs::read_to_string(&manifest_path).unwrap_or_default();
        if existing != manifest {
            std::fs::write(&manifest_path, manifest).unwrap();
        }
        let main_path = proj.join("src/main.rs");
        let existing_main = std::fs::read_to_string(&main_path).unwrap_or_default();
        if existing_main != product {
            std::fs::write(&main_path, product).unwrap();
        }
        let out = std::process::Command::new("cargo")
            .args(["run", "--quiet", "--bin", name])
            .current_dir(&proj)
            .env("CARGO_TARGET_DIR", proj.join("target"))
            .output()
            .expect("cargo run (is cargo on PATH?)");
        assert!(
            out.status.success(),
            "generated product failed to build/run ({name}):\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    fn md5_compat(s: &str) -> u64 {
        // 稳定内容哈希（非安全用途；djb2 变体足够做缓存键）。
        let mut hash: u64 = 5381;
        for b in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(b as u64);
        }
        hash
    }

    fn a2r_std_manifest_dir() -> String {
        // CARGO_MANIFEST_DIR = crates/auto-lang → ../a2r-std
        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.join("../a2r-std").to_string_lossy().into_owned()
    }

    /// 必选矩阵：同源 Auto 样本 → 转译 → 实编实跑 → wire 断言（同步 main）。
    #[test]
    fn http_e2e_plan724_sync_client_matrix() {
        let (port, _done) = spawn_stub();
        std::env::set_var("PLAN724_URL", format!("http://127.0.0.1:{port}/text"));
        let product = transpile_case(SYNC_CASE_DIR, "http_client_sync").unwrap();
        let stdout = build_and_run("plan724_sync", &product);
        assert!(stdout.contains("get status: 200"), "stdout: {stdout}");
        assert!(stdout.contains("get header: alpha"), "stdout: {stdout}");
        assert!(stdout.contains("get body: plan724-ok"), "stdout: {stdout}");
        assert!(
            stdout.contains("builder body: echo:builder-payload"),
            "stdout: {stdout}"
        );
        assert!(
            stdout.contains("post body: echo:post-payload"),
            "stdout: {stdout}"
        );
        assert!(stdout.contains("chunk: plan724-ok"), "stdout: {stdout}");
    }

    /// 必选矩阵：async main（~helper + .await + 流 for-in + 认证 async 面）。
    #[test]
    fn http_e2e_plan724_async_client_matrix() {
        let (port, _done) = spawn_stub();
        std::env::set_var("PLAN724_URL", format!("http://127.0.0.1:{port}/text"));
        let product = transpile_case(ASYNC_CASE_DIR, "http_client_async").unwrap();
        let stdout = build_and_run("plan724_async", &product);
        assert!(stdout.contains("get status: 200"), "stdout: {stdout}");
        assert!(
            stdout.contains("helper body: echo:async-post-payload"),
            "stdout: {stdout}"
        );
        assert!(
            stdout.contains("builder body: echo:async-builder-payload"),
            "stdout: {stdout}"
        );
        assert!(stdout.contains("chunk: plan724-ok"), "stdout: {stdout}");
        assert!(stdout.contains("auth status: 200"), "stdout: {stdout}");
        assert!(
            stdout.contains("auth body: echo:auth-payload"),
            "stdout: {stdout}"
        );
    }

    /// 第二 facade 腿（AC-01/AC-05）：`auto_lang::a2r_std::http`（生成服务
    /// 限定名消费面，api_gen 发射 `auto_lang::a2r_std` 路径）——同内核
    /// 执行、认证 tuple 形状、500 值语义、传输失败可观察。真 TCP。
    #[test]
    fn http_e2e_plan724_qualified_facade_kernel() {
        use crate::a2r_std::http;

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            use std::io::{Read, Write};
            // 恰好 4 个落线请求（死端口臂不连接）——循环结束线程退出，join 有界。
            for _ in 0..4 {
                let Ok((mut s, _)) = listener.accept() else {
                    break;
                };
                let mut buf = [0u8; 8192];
                let Ok(n) = s.read(&mut buf) else { break };
                if n == 0 {
                    continue;
                }
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                // 认证头恒在（历史行为：空 key 也发 x-api-key: ""）——
                // 500 臂按 key 值为空判定。
                let key_empty = req.lines().any(|l| {
                    let ll = l.to_ascii_lowercase();
                    ll.starts_with("x-api-key:") && l["x-api-key:".len()..].trim().is_empty()
                });
                let (status, body) = if key_empty {
                    ("500 Internal Server Error", "no-auth".to_string())
                } else {
                    ("200 OK", "qualified-ok".to_string())
                };
                let resp = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                );
                let _ = s.write_all(resp.as_bytes());
            }
        });
        let base = format!("http://127.0.0.1:{port}/api");

        // async post（4-tuple）：x-api-key/anthropic-version 落线 → 200 "ok"。
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let b = base.clone();
        let (status, body, error, kind) = rt.block_on(async { http::post(&b, "{}", "sk-q").await });
        assert_eq!(status, 200, "qualified post status");
        assert_eq!(kind, "ok");
        assert!(error.is_empty());
        assert_eq!(body, "qualified-ok");

        // 500 值语义：无认证头 → 500 是错误形状值（kind=error，不 throw，712）。
        let b2 = base.clone();
        let (status, _body, error, kind) = rt.block_on(async { http::post(&b2, "{}", "").await });
        assert_eq!(status, 500);
        assert_eq!(kind, "error");
        assert_eq!(error, "HTTP 500");

        // 同步面（同步桥接，测试线程无 runtime 上下文）。
        let (scode, sbody) = http::post_sync(&base, "sync-p", "sk-s");
        assert_eq!(scode, 200, "post_sync status");
        assert_eq!(sbody, "qualified-ok");
        let (bcode, _) = http::post_bearer_sync(&base, "sync-b", "tok");
        assert_eq!(bcode, 200, "post_bearer_sync 走 authorization（200 臂）");

        // 传输失败可观察（死端口）。
        let (code, msg) = http::post_sync("http://127.0.0.1:1/dead", "x", "k");
        assert_eq!(code, 0, "传输失败 status 0");
        assert!(!msg.is_empty(), "传输失败带错误信息");

        server.join().unwrap();
    }
}
