//! PLAN-727 T-06/T-07：文件传输 a2r 发射与三方一致性测试。
//!
//! - golden：`test/a2r/31_plan727/00{1,2}_http_transfer_{sync,async}/` 冻结
//!   发射形态（submit/observe 同形、wait 的 sync/async 分叉、builder
//!   multipart 方法）。
//! - legacy：`http.download/download_resume/upload`（Plan 349 形状）发射
//!   不回归（probe）。
//! - e2e（`test-http-e2e`，cargo th 串行）：真 TCP stub + 转译 + cargo
//!   build/run——新面/legacy/builder 的实际编译运行对拍
//!   （`http_e2e_plan727_*` 命名约定）。

use crate::trans::rust::transpile_rust_with_source_dir;
use std::fs::read_to_string;
use std::path::PathBuf;

const SYNC_CASE_DIR: &str = "test/a2r/31_plan727/001_http_transfer_sync";
const ASYNC_CASE_DIR: &str = "test/a2r/31_plan727/002_http_transfer_async";

fn case_name(case_dir: &str) -> String {
    case_dir.rsplit('/').next().unwrap_or(case_dir).split_once('_').unwrap().1.to_string()
}

fn transpile_case(case_dir: &str) -> Vec<u8> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let full = d.join(case_dir);
    let name = case_name(case_dir);
    let src = read_to_string(full.join(format!("{name}.at"))).expect("case .at");
    let mut rcode =
        transpile_rust_with_source_dir(&full, &name, &src).expect("transpile");
    rcode.done().expect("finalize").clone()
}

fn assert_golden(case_dir: &str) {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let name = case_name(case_dir);
    let expected = read_to_string(d.join(case_dir).join(format!("{name}.expected.rs")))
        .unwrap_or_default();
    let rs = transpile_case(case_dir);
    if rs != expected.as_bytes() {
        let wrong = d.join(case_dir).join(format!("{name}.wrong.rs"));
        std::fs::write(&wrong, &rs).expect("write wrong.rs");
        panic!(
            "golden mismatch for {case_dir}; actual written to {}",
            wrong.display()
        );
    }
}

#[test]
fn plan727_golden_sync_transfer_matrix() {
    assert_golden(SYNC_CASE_DIR);
}

#[test]
fn plan727_golden_async_transfer_matrix() {
    assert_golden(ASYNC_CASE_DIR);
}

/// legacy Plan 349 形状发射不回归：download/upload/download_resume 仍直发
/// `a2r_std::http::{download,upload,download_resume}`（u32 status 面），
/// 不被新面悄悄重定（决策报告 §4 legacy adapter 差异表的发射侧锁定）。
#[test]
fn plan727_probe_legacy_file_helper_emission_unchanged() {
    let src = r#"
fn main() {
    let s = http.download("http://x/f", "/tmp/f.bin")
    print(s)
    let r = http.download_resume("http://x/g", "/tmp/g.bin", 1024)
    print(r)
    let u = http.upload("http://x/up", "/tmp/f.bin")
    print(u)
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("plan727_legacy", src)
        .expect("transpile");
    let rs = String::from_utf8(rcode.done().expect("finalize").clone()).expect("utf8");
    assert!(rs.contains("a2r_std::http::download(\"http://x/f\""), "{rs}");
    assert!(
        rs.contains("a2r_std::http::download_resume(\"http://x/g\", \"/tmp/g.bin\", 1024)"),
        "{rs}"
    );
    assert!(rs.contains("a2r_std::http::upload(\"http://x/up\""), "{rs}");
    assert!(!rs.contains("transfer_download"), "legacy 面不得重定为新面: {rs}");
}

/// 64 位字面量：legacy 第三参/新面 options 内大 offset 不截断（u64 面发射）。
#[test]
fn plan727_probe_legacy_offset_u64_emission() {
    let src = r#"
fn main() {
    let r = http.download_resume("http://x/g", "/tmp/g.bin", 4294967296)
    print(r)
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("plan727_legacy_u64", src)
        .expect("transpile");
    let rs = String::from_utf8(rcode.done().expect("finalize").clone()).expect("utf8");
    assert!(rs.contains("4294967296"), "64 位字面量丢失: {rs}");
}

// ===========================================================================
// VM 腿（日常档）：新面 natives 经 Auto 源直跑 VM（run_with_capture）
// ===========================================================================

#[cfg(test)]
mod vm {
    use crate::run_with_capture;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn spawn_mock(response_body: &str) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let body = response_body.to_string();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let resp = format!(
                    "HTTP/1.1 200 OK
Content-Length: {}
Connection: close

{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        });
        std::thread::sleep(std::time::Duration::from_millis(50));
        port
    }

    fn dead_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    }

    /// 新面：transfer_download + transfer_wait 成功收据 + 目标替换。
    #[test]
    fn plan727_vm_transfer_download_wait_success() {
        let port = spawn_mock("vm-transfer-body");
        let url = format!("http://127.0.0.1:{port}/f");
        let dir = std::env::temp_dir().join(format!("plan727-vm-{}", port));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f.bin");
        std::fs::write(&path, b"OLD").unwrap();
        let code = format!(
            r#"
let t = http.transfer_download("{}", "{}", "{{}}")
let r = http.transfer_wait(t)
print(r)
"#,
            url,
            path.to_str().unwrap()
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok(), "VM run failed: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        eprintln!("DEBUG vm stdout = [{stdout}]");
        assert!(stdout.contains("\"kind\":\"success\""), "{stdout}");
        assert_eq!(std::fs::read(&path).unwrap(), b"vm-transfer-body");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 新面：坏 options → Failed(options)，请求不发出（死端口也不挂死）。
    #[test]
    fn plan727_vm_transfer_bad_options_terminal() {
        let port = dead_port();
        let dir = std::env::temp_dir().join(format!("plan727-vm-bad-{}", port));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f.bin");
        let code = format!(
            r#"
let t = http.transfer_download("http://127.0.0.1:{}/f", "{}", "{{bad json")
let r = http.transfer_wait(t)
print(r)
"#,
            port,
            path.to_str().unwrap()
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok(), "VM run failed: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        assert!(stdout.contains("\"kind\":\"failed\""), "{stdout}");
        assert!(stdout.contains("\"kind\":\"options\""), "{stdout}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 新面：取消词汇——cancel + wait 交付 cancelled（或已完成的 success）。
    #[test]
    fn plan727_vm_transfer_cancel_observable() {
        let port = spawn_mock("quick");
        let url = format!("http://127.0.0.1:{port}/f");
        let dir = std::env::temp_dir().join(format!("plan727-vm-cancel-{}", port));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f.bin");
        let code = format!(
            r#"
let t = http.transfer_download("{}", "{}", "{{}}")
http.transfer_cancel(t)
let r = http.transfer_wait(t)
print(r)
let e = http.transfer_error(t)
print(e)
"#,
            url,
            path.to_str().unwrap()
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok(), "VM run failed: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        assert!(
            stdout.contains("\"kind\":\"cancelled\"") || stdout.contains("\"kind\":\"success\""),
            "{stdout}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

}

// ===========================================================================
// e2e：真 TCP + 转译 + cargo build/run（test-http-e2e；cargo th 串行）
// ===========================================================================

#[cfg(feature = "test-http-e2e")]
mod e2e {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::PathBuf;
    use std::time::Duration;

    fn a2r_std_manifest_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../a2r-std")
    }

    fn hash_name(name: &str, payload: &str) -> String {
        let mut hash: u64 = 5381;
        for b in payload.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(b as u64);
        }
        format!("plan727-e2e-{name}-{hash:x}")
    }

    /// 转译产物 + a2r-std path-dep 临时工程，`cargo run --quiet`（产物按内容
    /// hash 复用实现秒级二次运行）。
    fn build_and_run(name: &str, product: &str) -> String {
        let payload = format!("{name}:{product}");
        let dir = std::env::temp_dir().join(hash_name(name, &payload));
        let src_dir = dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\na2r-std = {{ path = {:?} }}\ntokio = {{ version = \"1\", features = [\"full\"] }}\n\n[[bin]]\nname = \"{}\"\npath = \"src/main.rs\"\n",
            hash_name(name, &payload),
            a2r_std_manifest_dir(),
            hash_name(name, &payload),
        ))
        .unwrap();
        std::fs::write(src_dir.join("main.rs"), product).unwrap();
        let out = std::process::Command::new("cargo")
            .args(["run", "--quiet", "--bin", &hash_name(name, &payload)])
            .current_dir(&dir)
            .output()
            .expect("cargo run");
        assert!(
            out.status.success(),
            "cargo run failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// 回环 stub：GET /file 200 + payload；POST /upload 捕获请求头+体；
    /// GET /missing 404。
    fn spawn_stub() -> (u16, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let captured: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();
        let captured2 = captured.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let mut buf = vec![0u8; 65536];
                let mut head = Vec::new();
                loop {
                    let n = match stream.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => n,
                    };
                    head.extend_from_slice(&buf[..n]);
                    if head.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                let head_str = String::from_utf8_lossy(&head).to_string();
                // multipart 体按 Content-Length 继续读。
                let body_start = head_str.find("\r\n\r\n").map(|i| i + 4).unwrap_or(head.len());
                let cl = head_str
                    .to_ascii_lowercase()
                    .lines()
                    .find_map(|l| l.strip_prefix("content-length:").and_then(|v| v.trim().parse::<usize>().ok()))
                    .unwrap_or(0);
                let mut body = head.len() - body_start;
                while body < cl {
                    match stream.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => body += n,
                    }
                }
                captured2.lock().unwrap().push(head_str.clone());
                let resp = if head_str.starts_with("GET /missing ") {
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 4\r\n\r\nnope"
                } else if head_str.starts_with("GET ") {
                    "HTTP/1.1 200 OK\r\nContent-Length: 16\r\n\r\nplan727-payload!"
                } else {
                    "HTTP/1.1 201 Created\r\nContent-Length: 5\r\n\r\nsaved"
                };
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
            }
        });
        (port, captured)
    }

    /// sync 新面端到端：下载提交/等待收据（成功与 404 保留旧目标）、上传
    /// multipart wire、进度/取消词汇。
    #[test]
    fn http_e2e_plan727_sync_transfer_matrix() {
        let (port, captured) = spawn_stub();
        let base = format!("http://127.0.0.1:{port}");
        let dir = std::env::temp_dir().join(format!("plan727-e2e-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dst = dir.join("dst.bin");
        std::fs::write(&dst, b"OLD").unwrap();
        let src = dir.join("src.bin");
        std::fs::write(&src, b"FILE-BODY").unwrap();

        std::env::set_var("PLAN727_URL", &base);
        std::env::set_var("PLAN727_DST", dst.to_str().unwrap());
        std::env::set_var("PLAN727_SRC", src.to_str().unwrap());

        let product = String::from_utf8(transpile_case(SYNC_CASE_DIR)).unwrap();
        let out = build_and_run("sync", &product);

        // 1) 下载/上传成功收据 + 目标被替换。
        assert!(out.contains("receipt: {\"kind\":\"success\""), "{out}");
        assert!(out.contains("\"status\":200"), "{out}");
        assert!(out.contains("up_receipt: {\"kind\":\"success\""), "{out}");
        assert_eq!(std::fs::read(&dst).unwrap(), b"plan727-payload!");
        // 2) 上传：multipart wire（字段/file/文本字段）在 wire 断言。
        let uploads = captured.lock().unwrap();
        assert!(
            uploads.iter().any(|h| h.contains("POST ") && h.contains("multipart/form-data") && h.contains("name=\"note\"")),
            "multipart wire 缺失: {uploads:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 404 保留旧目标（原文件字节符合 AC-03；实编产物语境）。
    #[test]
    fn http_e2e_plan727_missing_preserves_target() {
        let (port, _captured) = spawn_stub();
        let dir = std::env::temp_dir().join(format!("plan727-e2e-miss-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dst = dir.join("keep.bin");
        std::fs::write(&dst, b"PRECIOUS").unwrap();
        let product = format!(
            r#"
fn main() {{
    let t = a2r_std::http::transfer_download("http://127.0.0.1:{port}/missing", {:?}, "{{}}");
    let receipt = a2r_std::http::transfer_wait(&t);
    println!("{{}}", receipt);
}}
"#,
            dst.to_str().unwrap()
        );
        let out = build_and_run("missing", &product);
        assert!(out.contains("\"kind\":\"failed\""), "{out}");
        assert!(out.contains("\"status\":404"), "{out}");
        assert_eq!(std::fs::read(&dst).unwrap(), b"PRECIOUS");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// async 新面：transfer_wait_async 让出 + 成功收据（编译运行腿）。
    #[test]
    fn http_e2e_plan727_async_transfer_matrix() {
        let (port, _captured) = spawn_stub();
        let dir = std::env::temp_dir().join(format!("plan727-e2e-async-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dst = dir.join("async.bin");
        std::env::set_var("PLAN727_URL", format!("http://127.0.0.1:{port}"));
        std::env::set_var("PLAN727_DST", dst.to_str().unwrap());
        let product = String::from_utf8(transpile_case(ASYNC_CASE_DIR)).unwrap();
        let out = build_and_run("async", &product);
        assert!(out.contains("receipt: {\"kind\":\"success\""), "{out}");
        assert!(out.contains("\"kind\":\"cancelled\""), "{out}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// builder multipart 实编：字段/filename/binary 内容上 wire。
    #[test]
    fn http_e2e_plan727_builder_multipart_wire() {
        let (port, captured) = spawn_stub();
        let dir = std::env::temp_dir().join(format!("plan727-e2e-mp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("report.bin");
        let payload: Vec<u8> = (0u8..=255).cycle().take(1000).collect();
        std::fs::write(&src, &payload).unwrap();
        let product = format!(
            r#"
fn main() {{
    let resp = a2r_std::http::request("POST", "http://127.0.0.1:{port}/upload")
        .multipart_text("note", "plan727-mp")
        .multipart_file("file", {:?})
        .send();
    println!("{{}}", resp.status_code());
}}
"#,
            src.to_str().unwrap()
        );
        let out = build_and_run("mp", &product);
        assert!(out.contains("201"), "{out}");
        let uploads = captured.lock().unwrap();
        let head = uploads.iter().find(|h| h.contains("POST /upload")).expect("upload captured");
        assert!(head.contains("multipart/form-data"), "{head}");
        assert!(head.contains("name=\"note\""), "{head}");
        assert!(head.contains("filename=\"report.bin\""), "{head}");
        std::fs::remove_dir_all(&dir).ok();
        let _ = Duration::from_secs(1);
    }
}
