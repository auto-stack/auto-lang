//! PLAN-730: HTTP 服务端上传（T-01 平台探针 + 后续单元/e2e 族）。
//!
//! 本文件随任务推进扩充：T-01 = no-replace 原语/同卷判定/rename 语义探针
//! （决策报告 §3 的平台证据）；T-03/T-04 追加 parser/预算/存储矩阵；
//! `http_e2e_plan730` 真 TCP 族挂在 `test-http-e2e` feature 下。

// ============================================================================
// T-01 平台探针：create-only 原子发布原语（Windows/Linux 双面证据）
// ============================================================================

mod plan730_probe {
    use std::path::{Path, PathBuf};

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "plan730-probe-{tag}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("probe temp dir");
        dir
    }

    /// P1（Windows 核心）：`std::fs::hard_link` 对已存在目标失败
    /// （ERROR_ALREADY_EXISTS / EEXIST）——std 级**原子 create-only** 原语。
    /// 成功路径：hard_link 后 staging 与 target 同内容，删 staging 后
    /// target 完整保留（发布语义 = link + unlink）。
    #[test]
    fn plan730_probe_hard_link_is_create_only() {
        let dir = temp_dir("hardlink");
        let staging = dir.join("staging.part");
        std::fs::write(&staging, b"plan730 probe payload").unwrap();

        // 已存在目标 → AlreadyExists（不覆盖、不改既有内容）。
        let occupied = dir.join("occupied.bin");
        std::fs::write(&occupied, b"original").unwrap();
        let err = std::fs::hard_link(&staging, &occupied).expect_err("must refuse existing target");
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::AlreadyExists,
            "hard_link over existing target must be AlreadyExists (got {err:?})"
        );
        assert_eq!(std::fs::read(&occupied).unwrap(), b"original");

        // 新目标 → 成功；删 staging 后 target 保留（link 计数语义）。
        let target = dir.join("published.bin");
        std::fs::hard_link(&staging, &target).expect("fresh hard_link succeeds");
        assert_eq!(std::fs::read(&target).unwrap(), b"plan730 probe payload");
        std::fs::remove_file(&staging).unwrap();
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"plan730 probe payload",
            "target survives staging removal"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// P2（反证）：`std::fs::rename` 在 Windows = MoveFileExW
    /// (REPLACE_EXISTING)、在 Linux = rename(2)——**两者都覆盖已存在
    /// 目标**，不满足 create-only 合同（决策报告 §3 的拒绝理由）。
    #[test]
    fn plan730_probe_rename_replaces_existing() {
        let dir = temp_dir("rename");
        let staging = dir.join("staging.part");
        std::fs::write(&staging, b"new content").unwrap();
        let target = dir.join("target.bin");
        std::fs::write(&target, b"original").unwrap();
        std::fs::rename(&staging, &target).expect("std rename overwrites by platform semantics");
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"new content",
            "rename replaced the target — NOT create-only"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// P3：同卷前置判定证据——Windows 走 canonicalize 前缀
    /// （`\\?\C:\` / `\\?\Volume{GUID}\`），unix 走 st_dev。同卷 staging
    /// 才能用 hard_link 原子发布（跨卷 → 配置错误，须在读取 body 前拒绝）。
    #[test]
    fn plan730_probe_same_volume_detection() {
        let root = temp_dir("volume");
        let staging = root.join("private-staging");
        std::fs::create_dir_all(&staging).unwrap();
        let same_volume = same_volume_probe(&root, &staging);
        assert!(same_volume, "sibling dirs on the same volume");

        // 嵌套关系（staging 在 root 之内）必须被词法检查拒绝（私有性合同）。
        let nested = root.join("public/nested");
        std::fs::create_dir_all(&nested).unwrap();
        assert!(is_lexically_inside(&root, &nested), "lexical nesting detected");
        let _ = std::fs::remove_dir_all(&root);
    }

    fn volume_key(p: &Path) -> String {
        let canon = std::fs::canonicalize(p).expect("canonicalize probe dir");
        let s = canon.to_string_lossy().to_string();
        #[cfg(windows)]
        {
            // `\\?\C:\x` / `\\?\Volume{GUID}\x` → 卷键 = 第三个反斜杠前的段。
            if let Some(rest) = s.strip_prefix(r"\\?\") {
                if let Some(idx) = rest.find('\\') {
                    return rest[..idx].to_ascii_lowercase();
                }
            }
            s.to_ascii_lowercase()
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let dev = std::fs::metadata(&canon).map(|m| m.dev().to_string());
            dev.unwrap_or(s)
        }
    }

    fn same_volume_probe(a: &Path, b: &Path) -> bool {
        volume_key(a) == volume_key(b)
    }

    fn is_lexically_inside(parent: &Path, child: &Path) -> bool {
        let p = parent.canonicalize().unwrap();
        let c = child.canonicalize().unwrap();
        c.starts_with(&p)
    }
}

// ============================================================================
// T-02：路由能力分类 + 加载期合同诊断 + VM 纯面 native 探针
// ============================================================================

mod plan730_t02 {
    /// 路由分类门：方法+参数类型双条件——声明 UploadRequest 参数的 fn 才
    /// 是上传路由；同名用户函数/普通参数不误判（AC-01 反例）。
    #[test]
    fn plan730_route_classification_by_param_type() {
        use crate::vm::ffi::http_server::{
            record_api_param_sigs, route_declares_upload, ApiParamSig,
        };
        let sig = |name: &str, ty: &str| ApiParamSig {
            name: name.to_string(),
            ty: ty.to_string(),
        };
        record_api_param_sigs(
            "upload_doc",
            vec![sig("req", "UploadRequest"), sig("meta", "str")],
        );
        record_api_param_sigs("plain_json", vec![sig("data", "str")]);
        record_api_param_sigs("req_named_str", vec![sig("req", "str")]);
        assert!(route_declares_upload("upload_doc"));
        assert!(!route_declares_upload("plain_json"));
        // 名为 req 的普通 str 参数不是注入参数（类型识别优先于命名约定）。
        assert!(!route_declares_upload("req_named_str"));
        assert!(!route_declares_upload("no_such_fn"));
    }

    /// 加载期合同诊断：GET 上传方法、body 参数混用、缺 UploadRequest 的
    /// UploadReceipt 返回——编译错误指名（AC-01/AC-07 的加载面）。
    #[test]
    fn plan730_codegen_upload_contract_diagnostics() {
        let get_upload = r#"
#[api(method = "GET", path = "/up")]
fn up(req UploadRequest) UploadReceipt {
    return http.upload_error(500, "x")
}
"#;
        let err = crate::run_with_capture(get_upload).expect_err("GET upload must be rejected");
        let msg = format!("{err:?}");
        assert!(msg.contains("POST/PUT"), "method diagnostic: {msg}");

        let mixed_body = r#"
#[api(method = "POST", path = "/up")]
fn up(req UploadRequest, note str) UploadReceipt {
    return http.upload_error(500, "x")
}
"#;
        let err = crate::run_with_capture(mixed_body).expect_err("body param must be rejected");
        let msg = format!("{err:?}");
        assert!(msg.contains("upload endpoints allow only path params"), "{msg}");

        let no_param = r#"
#[api(method = "POST", path = "/up")]
fn up() UploadReceipt {
    return http.upload_error(500, "x")
}
"#;
        let err = crate::run_with_capture(no_param).expect_err("receipt without request rejected");
        let msg = format!("{err:?}");
        assert!(msg.contains("no UploadRequest param"), "{msg}");
    }

    /// 合法形态的编译/运行证明经 T-05/T-08 e2e（start_server fixture——
    /// run_with_capture 对含 #[api] 路由的程序会自动起服并阻塞，不适用）。

    /// VM 纯面 native：upload_error 构造登记收据（零 I/O）；upload_metadata
    /// 未知句柄返回可观察冲突形态（不挂死）。
    #[test]
    fn plan730_vm_upload_error_and_metadata_probe() {
        let code = r#"
let receipt = http.upload_error(401, "missing token")
let unknown = http.upload_metadata(99999)
print("done")
"#;
        let (_, out) = crate::run_with_capture(code).expect("run");
        assert!(out.contains("done"), "{out}");
        let (reqs, sessions, receipts) = crate::vm::ffi::http_upload::vm_upload_counts();
        assert_eq!(reqs, 0, "no injected capability in plain script");
        assert!(receipts >= 1, "upload_error receipt registered");
        assert_eq!(sessions, 0);
        // 编组门反例：普通 fn（非 #[api]）不在上传返回表。
        assert!(!crate::vm::ffi::http_server::fn_is_api_upload_return("main"));
    }
}

// ===========================================================================
// e2e（`test-http-e2e` 串行档）：VM 默认 HTTP 真 TCP 上传矩阵（T-05）。
// 生成 Rust 腿 + 727/729 互通在 T-06/T-08（auto-man api_gen 测试 + 本族扩展）。
// ===========================================================================

#[cfg(feature = "test-http-e2e")]
mod http_e2e {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::path::PathBuf;
    use std::time::Duration;

    /// 固定唯一端口（729 已占 18950-18966；http_server e2e 已占 18731-18775）。
    const PORT_MP: u16 = 18970;
    const PORT_RAW: u16 = 18971;
    const PORT_AUTH: u16 = 18972;
    const PORT_CONF: u16 = 18973;
    const PORT_SLOW: u16 = 18974;
    const PORT_LEGACY: u16 = 18975;

    fn temp_roots(tag: &str) -> (PathBuf, PathBuf) {
        let base =
            std::env::temp_dir().join(format!("plan730-e2e-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("public");
        let staging = base.join("private-staging");
        std::fs::create_dir_all(root.join("mp")).unwrap();
        std::fs::create_dir_all(root.join("raw")).unwrap();
        std::fs::create_dir_all(&staging).unwrap();
        (root, staging)
    }

    /// .at 字符串字面量：反斜杠必须转正斜杠（729 同款约束）。
    fn fwd(p: &std::path::Path) -> String {
        p.to_string_lossy().replace('\\', "/")
    }

    pub(super) fn start_server(code: &str, port: u16) -> u16 {
        crate::vm::ffi::stdlib::clear_http_routes();
        std::env::set_var("AUTO_HTTP_PORT", port.to_string());
        let code = code.to_string();
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let _ = crate::run(&code);
            })
            .expect("spawn server thread");
        for _ in 0..50 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        port
    }

    /// 上传程序：multipart/raw 端点 + 业务拒绝 + 早拒 + 729 下载路由 +
    /// middleware guard（http.server.use 注册）。
    fn upload_program(root: &std::path::Path, staging: &std::path::Path) -> String {
        let root_s = fwd(root);
        let staging_s = fwd(staging);
        let mut src = format!(
            r#"
fn plan730_guard(info str) str {{
    if info.contains("/guarded/") {{
        return "{{\"error\":\"auth required\"}}"
    }}
    return ""
}}

#[api(method = "POST", path = "/api/uploads/mp")]
fn upload_mp(req UploadRequest) ~UploadReceipt {{
    let session = http.upload_receive(req, "{root_s}", "{staging_s}", "{{\"mode\":\"multipart\",\"text_fields\":[\"note\"]}}")
    let meta = http.upload_metadata(session)
    if meta.contains("{{\"state\":\"failed\"") {{
        return http.upload_reject(session, 0, "receive failed")
    }}
    if meta.contains("\"value\":\"deny\"") {{
        return http.upload_reject(session, 422, "business check failed")
    }}
    return http.upload_commit(session, "mp/blob.bin")
}}

#[api(method = "POST", path = "/api/uploads/raw")]
fn upload_raw(req UploadRequest) ~UploadReceipt {{
    let session = http.upload_receive(req, "{root_s}", "{staging_s}", "{{\"mode\":\"raw\"}}")
    let meta = http.upload_metadata(session)
    if meta.contains("{{\"state\":\"failed\"") {{
        return http.upload_reject(session, 0, "receive failed")
    }}
    return http.upload_commit(session, "raw/data.bin")
}}

#[api(method = "POST", path = "/api/early")]
fn upload_early(req UploadRequest) ~UploadReceipt {{
    return http.upload_error(401, "missing token")
}}

#[api(method = "POST", path = "/guarded/mp")]
fn upload_guarded(req UploadRequest) ~UploadReceipt {{
    let session = http.upload_receive(req, "{root_s}", "{staging_s}", "{{\"mode\":\"raw\"}}")
    return http.upload_commit(session, "mp/guarded.bin")
}}

#[api(method = "GET", path = "/api/files/:d/:name")]
fn download(d str, name str) FileResponse {{
    return http.file_response("{root_s}", d + "/" + name, "{{}}")
}}

#[api(method = "GET", path = "/plain")]
fn plain_int() int {{
    return 730730
}}
"#
        );
        let _ = &mut src;
        src
    }

    /// 带体的二进制安全原始请求。
    fn raw_request_with_body(
        port: u16,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> (u16, Vec<(String, String)>, Vec<u8>) {
        let mut stream = None;
        for _ in 0..60 {
            if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                stream = Some(s);
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let mut stream = stream.expect("connect");
        stream.set_read_timeout(Some(Duration::from_secs(60))).ok();
        let mut req =
            format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n");
        let has_cl = headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-length"));
        for (k, v) in headers {
            req.push_str(&format!("{k}: {v}\r\n"));
        }
        if !has_cl {
            req.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        req.push_str("\r\n");
        stream.write_all(req.as_bytes()).unwrap();
        if !body.is_empty() {
            stream.write_all(body).unwrap();
        }
        let _ = stream.flush();
        read_response(&mut stream)
    }

    fn read_response(stream: &mut TcpStream) -> (u16, Vec<(String, String)>, Vec<u8>) {
        let mut raw = Vec::new();
        let mut buf = [0u8; 65536];
        let header_end = loop {
            let n = stream.read(&mut buf).unwrap_or(0);
            if n == 0 {
                break None;
            }
            raw.extend_from_slice(&buf[..n]);
            if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                break Some(pos + 4);
            }
        };
        let header_end = header_end.expect("response headers");
        let head = String::from_utf8_lossy(&raw[..header_end]).into_owned();
        let mut lines = head.split("\r\n");
        let status_line = lines.next().unwrap_or_default().to_string();
        let status: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let mut hdrs = Vec::new();
        for line in lines {
            if let Some((k, v)) = line.split_once(':') {
                hdrs.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
            }
        }
        let cl: usize = hdrs
            .iter()
            .find(|(k, _)| k == "content-length")
            .and_then(|(_, v)| v.parse().ok())
            .unwrap_or(0);
        let mut body = raw[header_end..].to_vec();
        while body.len() < cl {
            let n = stream.read(&mut buf).unwrap_or(0);
            if n == 0 {
                break;
            }
            body.extend_from_slice(&buf[..n]);
        }
        body.truncate(cl);
        (status, hdrs, body)
    }

    fn multipart_body(boundary: &str, note: &str, file: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        b.extend_from_slice(b"Content-Disposition: form-data; name=\"note\"\r\n\r\n");
        b.extend_from_slice(note.as_bytes());
        b.extend_from_slice(b"\r\n");
        b.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        b.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"blob.bin\"\r\n\r\n",
        );
        b.extend_from_slice(file);
        b.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
        b
    }

    fn staging_is_empty(staging: &std::path::Path) -> bool {
        std::fs::read_dir(staging).unwrap().next().is_none()
    }

    fn connect(port: u16) -> TcpStream {
        for _ in 0..60 {
            if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                return s;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("connect to {port}");
    }

    /// multipart 上传→commit 201→729 路由下载同字节（12MiB>普通 10MiB）。
    #[test]
    fn http_e2e_plan730_vm_multipart_upload_commit_download() {
        let (root, staging) = temp_roots("mp");
        let port = start_server(&upload_program(&root, &staging), PORT_MP);
        let payload: Vec<u8> = (0..12usize * 1024 * 1024).map(|i| (i % 253) as u8).collect();
        let body = multipart_body("P730BOUND", "hello", &payload);
        let (status, _hdrs, resp) = raw_request_with_body(
            port,
            "POST",
            "/api/uploads/mp",
            &[("Content-Type", "multipart/form-data; boundary=P730BOUND")],
            &body,
        );
        assert_eq!(status, 201, "{}", String::from_utf8_lossy(&resp));
        let text = String::from_utf8_lossy(&resp).to_string();
        assert!(text.contains("\"ok\":true"), "{text}");
        assert!(text.contains("\"size\":\"12582912\""), "{text}");
        assert!(text.contains("\"path\":\"mp/blob.bin\""), "{text}");
        assert!(text.contains("\"name\":\"note\",\"value\":\"hello\""), "{text}");
        assert!(staging_is_empty(&staging), "staging cleaned after commit");
        let on_disk = std::fs::read(root.join("mp/blob.bin")).unwrap();
        assert_eq!(on_disk.len(), payload.len());
        // 729 下载路由读同文件（字节相同——上传后下载互通，T-08 前置）。
        let (s2, _h2, downloaded) = raw_request_with_body(port, "GET", "/api/files/mp/blob.bin", &[], b"");
        assert_eq!(s2, 200);
        assert_eq!(downloaded, on_disk, "download bytes identical");
        // 同名再传 → 409（原文件不变）。
        let body2 = multipart_body("P730BOUND", "again", b"second");
        let (s3, _h3, r3) = raw_request_with_body(
            port,
            "POST",
            "/api/uploads/mp",
            &[("Content-Type", "multipart/form-data; boundary=P730BOUND")],
            &body2,
        );
        assert_eq!(s3, 409, "{}", String::from_utf8_lossy(&r3));
        assert_eq!(std::fs::read(root.join("mp/blob.bin")).unwrap(), on_disk);
    }

    /// raw 上传 + 业务拒绝（字段 deny → 422，无公开文件，staging 清零）。
    #[test]
    fn http_e2e_plan730_vm_raw_upload_and_business_reject() {
        let (root, staging) = temp_roots("raw");
        let port = start_server(&upload_program(&root, &staging), PORT_RAW);
        let (s1, _h1, b1) = raw_request_with_body(
            port,
            "POST",
            "/api/uploads/raw",
            &[("Content-Type", "application/octet-stream")],
            b"RAWPAYLOAD-bytes",
        );
        assert_eq!(s1, 201, "{}", String::from_utf8_lossy(&b1));
        assert_eq!(
            std::fs::read(root.join("raw/data.bin")).unwrap(),
            b"RAWPAYLOAD-bytes"
        );
        // 业务拒绝：multipart note=deny → 422，无公开文件、staging 清零。
        let body = multipart_body("B2", "deny", b"SECRET");
        let (s2, _h2, b2) = raw_request_with_body(
            port,
            "POST",
            "/api/uploads/mp",
            &[("Content-Type", "multipart/form-data; boundary=B2")],
            &body,
        );
        assert_eq!(s2, 422, "{}", String::from_utf8_lossy(&b2));
        assert!(
            !root.join("mp/blob.bin").exists(),
            "rejected upload not published"
        );
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !staging_is_empty(&staging) && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(staging_is_empty(&staging), "staging cleaned after reject");
    }

    /// 鉴权先于存储：404 不解析不落盘；早拒（Expect: 100-continue 只发
    /// headers）不接收文件、staging 零创建；middleware 拒绝零落盘。
    #[test]
    fn http_e2e_plan730_vm_auth_before_storage() {
        let (root, staging) = temp_roots("auth");
        let port = start_server(&upload_program(&root, &staging), PORT_AUTH);
        // 404：multipart 打未知路径——零文件零 staging（T-07 顺序修复证据）。
        let body = multipart_body("B3", "x", b"SHOULD-NOT-LAND");
        let (s1, _h1, _b1) = raw_request_with_body(
            port,
            "POST",
            "/api/nope",
            &[("Content-Type", "multipart/form-data; boundary=B3")],
            &body,
        );
        assert_eq!(s1, 404);
        assert!(staging_is_empty(&staging), "404 must not stage anything");
        // 早拒：Expect 100-continue 只发 headers——401 立即返回，无需发体。
        let mut stream = connect(port);
        stream.set_read_timeout(Some(Duration::from_secs(20))).ok();
        let req = "POST /api/early HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\
                   Content-Type: multipart/form-data; boundary=B\r\n\
                   Expect: 100-continue\r\nContent-Length: 1000000\r\n\r\n";
        stream.write_all(req.as_bytes()).unwrap();
        let (s2, _h2, b2) = read_response(&mut stream);
        assert_eq!(s2, 401, "{}", String::from_utf8_lossy(&b2));
        assert!(String::from_utf8_lossy(&b2).contains("missing token"));
        assert!(staging_is_empty(&staging), "early reject stages nothing");
        // middleware 拒绝（短路体 legacy 200 形状）：零落盘（授权先于存储）。
        {
            if let Ok(mut chain) = crate::vm::ffi::stdlib::MIDDLEWARE_CHAIN.lock() {
                chain.push("plan730_guard".to_string());
            }
        }
        let (s3, _h3, b3) = raw_request_with_body(
            port,
            "POST",
            "/guarded/mp",
            &[("Content-Type", "application/octet-stream")],
            b"GUARDED",
        );
        assert_eq!(
            s3, 200,
            "middleware short-circuit keeps legacy reply shape: {}",
            String::from_utf8_lossy(&b3)
        );
        assert!(staging_is_empty(&staging), "middleware reject stages nothing");
        assert!(
            std::fs::read_dir(root.join("mp")).unwrap().next().is_none(),
            "no published file"
        );
        {
            if let Ok(mut chain) = crate::vm::ffi::stdlib::MIDDLEWARE_CHAIN.lock() {
                chain.clear();
            }
        }
    }

    /// 超限与畸形：CL 超 wire 预算 413（读体前）；坏 multipart 400 无发布。
    #[test]
    fn http_e2e_plan730_vm_wire_caps_and_malformed() {
        let (root, staging) = temp_roots("cap");
        let port = start_server(&upload_program(&root, &staging), PORT_CONF);
        let mut stream = connect(port);
        stream.set_read_timeout(Some(Duration::from_secs(20))).ok();
        let req = "POST /api/uploads/mp HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\
                   Content-Type: multipart/form-data; boundary=B\r\n\
                   Content-Length: 104857600\r\n\r\n";
        stream.write_all(req.as_bytes()).unwrap();
        let (s1, _h1, _b1) = read_response(&mut stream);
        assert_eq!(
            s1, 413,
            "declared CL over upload wire cap rejected pre-read"
        );
        assert!(staging_is_empty(&staging));
        // 坏 multipart（截断缺 closing boundary）→ 400，无公开文件。
        let mut truncated = multipart_body("B4", "n", b"DATA");
        truncated.truncate(truncated.len() - 12);
        let (s2, _h2, b2) = raw_request_with_body(
            port,
            "POST",
            "/api/uploads/mp",
            &[("Content-Type", "multipart/form-data; boundary=B4")],
            &truncated,
        );
        assert_eq!(s2, 400, "{}", String::from_utf8_lossy(&b2));
        assert!(!root.join("mp/blob.bin").exists());
    }

    /// >30s 受控慢上传仍合法成功（期限切换：30s handler deadline 不杀上传）。
    #[test]
    fn http_e2e_plan730_vm_slow_upload_survives_handler_deadline() {
        let (root, staging) = temp_roots("slow");
        let port = start_server(&upload_program(&root, &staging), PORT_SLOW);
        let payload: Vec<u8> = vec![0x5A; 12 * 1024 * 1024];
        let body = multipart_body("SLOWB", "slow", &payload);
        let mut stream = connect(port);
        stream.set_read_timeout(Some(Duration::from_secs(120))).ok();
        let head = format!(
            "POST /api/uploads/mp HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\
             Content-Type: multipart/form-data; boundary=SLOWB\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        stream.write_all(head.as_bytes()).unwrap();
        // 受控慢发：~36s 总时长（>30s 普通请求期限）。
        let started = std::time::Instant::now();
        for chunk in body.chunks(512 * 1024) {
            stream.write_all(chunk).unwrap();
            let _ = stream.flush();
            std::thread::sleep(Duration::from_millis(1500));
        }
        let (status, _hdrs, resp) = read_response(&mut stream);
        let elapsed = started.elapsed();
        assert!(elapsed.as_secs() > 30, "upload spanned >30s ({elapsed:?})");
        assert_eq!(status, 201, "{}", String::from_utf8_lossy(&resp));
        assert_eq!(
            std::fs::read(root.join("mp/blob.bin")).unwrap().len(),
            payload.len()
        );
    }

    /// 上传路由存在时普通端点不受影响（普通 int 反例不被上传收据门误判）。
    #[test]
    fn http_e2e_plan730_vm_plain_endpoint_untouched() {
        let (root, staging) = temp_roots("plain");
        let port = start_server(&upload_program(&root, &staging), PORT_LEGACY);
        let (s, _h, b) = raw_request_with_body(port, "GET", "/plain", &[], b"");
        assert_eq!(s, 200);
        assert_eq!(
            b,
            b"730730",
            "plain int endpoint not hijacked by upload gates"
        );
    }
}
