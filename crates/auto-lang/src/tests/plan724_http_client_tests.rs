//! PLAN-724：a2r HTTP 客户端收敛的转译探针与 golden。
//!
//! - golden：`test/a2r/30_plan724/00{1,2}_http_client_{sync,async}/` 冻结
//!   发射形态（test-convention：golden 只验证 Rust 文本）。
//! - 编译运行腿：`http_e2e_plan724_*`（`test-http-e2e` feature 守卫，真
//!   TCP + 临时 cargo 工程，见 `http_e2e_plan724_client_matrix`）。
//!
//! 分派矩阵冻结见 `docs/plans/reports/724-client-decision.md`。

use crate::{
    error::AutoResult,
    trans::rust::transpile_rust_with_source_dir,
};
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
/// async 上下文两参 post 必须走 post_async，三参认证必须走 _async 内核面。
#[test]
fn plan724_probe_post_arity_dispatch_shapes() {
    let async_src = r#"
fn fetch(url str) ~str {
    let res = http.post(url, "x")
    return str.from_bytes(res.body_bytes())
}
fn main() {
    let a = http.post("http://x", "y", "key")
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
    // 三参 post → 历史 HttpResponse 认证面（不冲突）
    assert!(
        code.contains("a2r_std::http::post("),
        "3-arg post must keep the auth HttpResponse face:\n{code}"
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
    assert!(!code.contains("send_async"), "sync ctx must not emit send_async:\n{code}");

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
        let proj = std::env::temp_dir()
            .join(format!("plan724-e2e-{name}-{hash}"));
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
        assert!(stdout.contains("builder body: echo:builder-payload"), "stdout: {stdout}");
        assert!(stdout.contains("post body: echo:post-payload"), "stdout: {stdout}");
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
        assert!(stdout.contains("helper body: echo:async-post-payload"), "stdout: {stdout}");
        assert!(stdout.contains("builder body: echo:async-builder-payload"), "stdout: {stdout}");
        assert!(stdout.contains("chunk: plan724-ok"), "stdout: {stdout}");
        assert!(stdout.contains("auth status: 200"), "stdout: {stdout}");
        assert!(stdout.contains("auth body: echo:auth-payload"), "stdout: {stdout}");
    }
}
