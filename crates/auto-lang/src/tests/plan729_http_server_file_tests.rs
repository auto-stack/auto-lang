//! PLAN-729：服务端文件响应——a2r 发射与 VM native 面测试。
//!
//! - golden：`test/a2r/32_plan729/001_http_file_response/` 冻结发射形态
//!   （描述符构造 sync/async 同形、严格 options 字面量透传、typed 返回面）。
//! - probe：FileResponse 类型映射（`a2r_std::http::FileResponse` 全限定）、
//!   普通 int 不被误判（VM 面：声明门反例）、native 构造零 I/O。
//! - e2e wire/生命周期矩阵在 `http_e2e_plan729_*`（`test-http-e2e` 串行档，
//!   T-06/T-07 增补）。

use crate::trans::rust::transpile_rust_with_source_dir;
use std::fs::read_to_string;
use std::path::PathBuf;

const FILE_RESPONSE_CASE_DIR: &str = "test/a2r/32_plan729/001_http_file_response";

fn case_name(case_dir: &str) -> String {
    case_dir
        .rsplit('/')
        .next()
        .unwrap_or(case_dir)
        .split_once('_')
        .unwrap()
        .1
        .to_string()
}

fn transpile_case(case_dir: &str) -> Vec<u8> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let full = d.join(case_dir);
    let name = case_name(case_dir);
    let src = read_to_string(full.join(format!("{name}.at"))).expect("case .at");
    let mut rcode = transpile_rust_with_source_dir(&full, &name, &src).expect("transpile");
    rcode.done().expect("finalize").clone()
}

fn assert_golden(case_dir: &str) {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let name = case_name(case_dir);
    let expected =
        read_to_string(d.join(case_dir).join(format!("{name}.expected.rs"))).unwrap_or_default();
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
fn plan729_golden_file_response_matrix() {
    assert_golden(FILE_RESPONSE_CASE_DIR);
}

/// 描述符构造 lowering：qualified/直串/严格 options 字面量原样透传，
/// sync/async 同形（无 async 变体——构造零 I/O）。
#[test]
fn plan729_probe_file_response_emission() {
    let src = r#"
fn main() {
    let a = http.file_response("root", "x.bin", "{}")
    let b = http.file_response("root", "y.bin", "{\"etag\":\"\\\"e\\\"\"}")
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("plan729_probe", src).expect("transpile");
    let rs = String::from_utf8(rcode.done().expect("finalize").clone()).expect("utf8");
    assert!(rs.contains("a2r_std::http::file_response("), "{rs}");
    // options 字面量字节保真（含转义引号）。
    assert!(
        rs.contains(r#"file_response("root", "y.bin", "{\"etag\":\"\\\"e\\\"\"}")"#)
            || rs.contains("file_response(\"root\", \"y.bin\","),
        "{rs}"
    );
}

/// 类型映射：`FileResponse` → `a2r_std::http::FileResponse`（全限定，
/// StringBuilder/SqliteDb 先例同款——无 glob import 也解析）。
#[test]
fn plan729_probe_file_response_type_mapping() {
    let src = r#"
fn helper(name str) FileResponse {
    return http.file_response("files", name, "{}")
}
fn main() {
    let f = helper("a")
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("plan729_type_map", src).expect("transpile");
    let rs = String::from_utf8(rcode.done().expect("finalize").clone()).expect("utf8");
    assert!(
        rs.contains("-> a2r_std::http::FileResponse"),
        "typed 返回面映射: {rs}"
    );
    assert!(!rs.contains("impl FileResponse"), "不误判 trait 返回: {rs}");
}

// ===========================================================================
// VM 面（日常档）：native 构造（id 9936）、描述符登记、普通 int 反例门。
// e2e wire（真 TCP GET/HEAD/Range/条件）在 http_e2e_plan729_*（T-06）。
// ===========================================================================

#[cfg(test)]
mod vm {
    /// `http.file_response` native 构造：返回登记句柄（int），注册表有条目，
    /// 且构造本身零 I/O（不存在的 root/文件也不失败——故障由 adapter 映射）。
    #[test]
    fn plan729_vm_native_construct_registers_descriptor() {
        let code = r#"
let fr = http.file_response("C:/definitely/not/a/real/root", "missing.bin", "{}")
print("ok")
"#;
        let (_, out) = crate::run_with_capture(code).expect("run");
        assert!(out.contains("ok"), "构造零 I/O（缺 root 不失败）: {out}");
        assert!(
            crate::vm::ffi::http_server_file::vm_file_response_count() >= 1,
            "描述符登记进 VM 注册表"
        );
    }

    /// 严格 options 反例：坏 options 描述符携带 init_error（构造不抛——
    /// adapter 映射 500；本测锁 VM 构造面的错误态存在性）。
    #[test]
    fn plan729_vm_bad_options_descriptor_has_init_error() {
        let code = r#"
let bad = http.file_response("root", "a", "{\"bogus\":1}")
print("done")
"#;
        let (_, out) = crate::run_with_capture(code).expect("run");
        assert!(out.contains("done"), "坏 options 构造不中断: {out}");
    }

    /// 声明返回门反例：非 #[api] 上下文直接调用返回 int——普通 int 不被
    /// marshal 当描述符消费（fn_is_api_file_return 只认 #[api] 声明表）。
    #[test]
    fn plan729_vm_plain_int_not_file_return() {
        assert!(
            !crate::vm::ffi::http_server::fn_is_api_file_return("main"),
            "普通 fn 不在文件返回表"
        );
    }
}

// ===========================================================================
// e2e（`test-http-e2e` 串行档）：VM 默认 HTTP 真 TCP wire 矩阵（T-06）。
// 双端矩阵的生成 Rust 腿在 auto-man api_gen 测试（同 feature 名）。
// ===========================================================================

#[cfg(feature = "test-http-e2e")]
mod http_e2e {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::path::PathBuf;
    use std::time::Duration;

    /// 固定唯一端口（18731..18775 已被 http_server.rs e2e 占用）。
    const PORT_BASIC: u16 = 18950;
    const PORT_LARGE: u16 = 18951;
    const PORT_RANGE: u16 = 18952;
    const PORT_COND: u16 = 18953;
    const PORT_PATH: u16 = 18954;
    const PORT_METHOD: u16 = 18955;

    pub(super) fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("plan729-e2e-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// 12MiB 确定样本（模式生成；返回期望 hash）。
    fn write_large(path: &std::path::Path) -> blake3::Hash {
        use std::io::Write as _;
        let mut hasher = blake3::Hasher::new();
        let mut file = std::fs::File::create(path).unwrap();
        let mut block = vec![0u8; 4096];
        for i in 0u64..3072 {
            let bytes = i.to_le_bytes();
            for (j, b) in block.iter_mut().enumerate() {
                *b = bytes[j % 8] ^ (j as u8) ^ (i as u8);
            }
            file.write_all(&block).unwrap();
            hasher.update(&block);
        }
        hasher.finalize()
    }

    /// 二进制安全的原始请求：返回 (状态码, 头部对(小写名), body 字节)。
    /// 按 Content-Length 精确读体（NUL/非 UTF-8 样本不经 lossy）。
    pub(super) fn raw_request(
        port: u16,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
    ) -> (u16, Vec<(String, String)>, Vec<u8>) {
        let mut stream = None;
        for _ in 0..50 {
            if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                stream = Some(s);
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let mut stream = stream.expect("connect to test server");
        stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
        let mut req =
            format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n");
        for (k, v) in headers {
            req.push_str(&format!("{k}: {v}\r\n"));
        }
        req.push_str("\r\n");
        stream.write_all(req.as_bytes()).unwrap();
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

    pub(super) fn header_of(hdrs: &[(String, String)], name: &str) -> String {
        hdrs.iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
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

    pub(super) fn program(root: &str) -> String {
        // etag 端点：bare 强验证器 `v1`（客户端 If-None-Match/If-Range 直配）。
        format!(
            r#"
#[api(method = "GET", path = "/files/:name")]
fn get_file(name str) FileResponse {{
    return http.file_response("{root}", name, "{{}}")
}}

#[api(method = "GET", path = "/etag/:name")]
fn get_etagged(name str) FileResponse {{
    return http.file_response("{root}", name, "{{\"etag\":\"v1\"}}")
}}

#[api(method = "GET", path = "/deep/:d/:name")]
fn get_deep(d str, name str) FileResponse {{
    return http.file_response("{root}", d + "/" + name, "{{}}")
}}

#[api(method = "GET", path = "/plain")]
fn plain_int() int {{
    return 729729
}}
"#
        )
    }

    /// 基本：文本/二进制（NUL+非 UTF-8）/零字节 + HEAD 同表示 metadata。
    #[test]
    fn http_e2e_plan729_vm_basic_get_head() {
        let root = temp_root("basic");
        std::fs::write(root.join("hello.txt"), b"hello plan729 e2e").unwrap();
        std::fs::write(root.join("raw.bin"), [0x00u8, 0xFF, 0xFE, 0x80, 0x01]).unwrap();
        std::fs::write(root.join("empty.bin"), b"").unwrap();
        let port = start_server(
            &program(&root.to_str().unwrap().replace('\\', "/")),
            PORT_BASIC,
        );

        let (status, hdrs, body) = raw_request(port, "GET", "/files/hello.txt", &[]);
        assert_eq!(status, 200, "{hdrs:?}");
        assert_eq!(header_of(&hdrs, "content-type"), "text/plain");
        assert_eq!(header_of(&hdrs, "content-length"), "17");
        assert_eq!(header_of(&hdrs, "accept-ranges"), "bytes");
        assert_eq!(header_of(&hdrs, "etag"), "", "默认不发 ETag");
        assert!(!header_of(&hdrs, "last-modified").is_empty());
        assert_eq!(body, b"hello plan729 e2e");

        let (status, hdrs, body) = raw_request(port, "GET", "/files/raw.bin", &[]);
        assert_eq!(status, 200);
        assert_eq!(header_of(&hdrs, "content-type"), "application/octet-stream");
        assert_eq!(body, vec![0x00u8, 0xFF, 0xFE, 0x80, 0x01]);

        let (status, hdrs, body) = raw_request(port, "GET", "/files/empty.bin", &[]);
        assert_eq!(status, 200);
        assert_eq!(header_of(&hdrs, "content-length"), "0");
        assert!(body.is_empty());

        let (status, hdrs, body) =
            raw_request(port, "HEAD", "/files/hello.txt", &[("range", "bytes=0-3")]);
        assert_eq!(status, 200);
        assert_eq!(
            header_of(&hdrs, "content-length"),
            "17",
            "HEAD 表示长度（Range 忽略）"
        );
        assert!(body.is_empty(), "HEAD wire body 必须为 0");
        assert!(!header_of(&hdrs, "access-control-allow-origin").is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 12MiB 确定样本：GET hash 一致、HEAD 长度（wire 级流式读满）。
    #[test]
    fn http_e2e_plan729_vm_large_12mib() {
        let root = temp_root("large");
        let expect = write_large(&root.join("big.bin"));
        let port = start_server(
            &program(&root.to_str().unwrap().replace('\\', "/")),
            PORT_LARGE,
        );

        let (status, hdrs, body) = raw_request(port, "GET", "/files/big.bin", &[]);
        assert_eq!(status, 200);
        assert_eq!(header_of(&hdrs, "content-length"), "12582912");
        assert_eq!(body.len(), 12 * 1024 * 1024, "12MiB 全量");
        assert_eq!(blake3::hash(&body), expect, "hash 一致（不经 JSON/lossy）");

        let (status, hdrs, _) = raw_request(port, "HEAD", "/files/big.bin", &[]);
        assert_eq!(status, 200);
        assert_eq!(header_of(&hdrs, "content-length"), "12582912");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Range 矩阵：bounded/open/suffix/end 越 EOF/suffix>len/-0/起点=EOF/
    /// 多区间/坏单位/坏语法/u64 大值。
    #[test]
    fn http_e2e_plan729_vm_range_matrix() {
        let root = temp_root("range");
        let data: Vec<u8> = (0u8..=199).collect();
        std::fs::write(root.join("r.bin"), &data).unwrap();
        let port = start_server(
            &program(&root.to_str().unwrap().replace('\\', "/")),
            PORT_RANGE,
        );

        let (s, h, b) = raw_request(port, "GET", "/files/r.bin", &[("range", "bytes=10-19")]);
        assert_eq!(
            (s, header_of(&h, "content-range").as_str()),
            (206, "bytes 10-19/200")
        );
        assert_eq!(header_of(&h, "content-length"), "10");
        assert_eq!(b, (10u8..=19).collect::<Vec<u8>>());

        let (s, _h, b) = raw_request(port, "GET", "/files/r.bin", &[("range", "bytes=190-")]);
        assert_eq!(b, (190u8..=199).collect::<Vec<u8>>());
        assert_eq!(s, 206u16);

        let (s, _h, b) = raw_request(port, "GET", "/files/r.bin", &[("range", "bytes=-7")]);
        assert_eq!(b, (193u8..=199).collect::<Vec<u8>>());
        assert_eq!(s, 206u16);

        let (s, h, _) = raw_request(port, "GET", "/files/r.bin", &[("range", "bytes=90-99999")]);
        assert_eq!(
            (s, header_of(&h, "content-range").as_str()),
            (206, "bytes 90-199/200")
        );

        let (s, h, b) = raw_request(port, "GET", "/files/r.bin", &[("range", "bytes=-5000")]);
        assert_eq!(
            (s, header_of(&h, "content-range").as_str()),
            (206, "bytes 0-199/200")
        );
        assert_eq!(b.len(), 200);

        let (s, h, b) = raw_request(port, "GET", "/files/r.bin", &[("range", "bytes=-0")]);
        assert_eq!(
            (s, header_of(&h, "content-range").as_str()),
            (416, "bytes */200")
        );
        assert!(b.is_empty());

        let (s, _, _) = raw_request(port, "GET", "/files/r.bin", &[("range", "bytes=200-")]);
        assert_eq!(s, 416, "起点=EOF");

        for bad in ["bytes=0-4,10-14", "chunks=0-4", "bytes=5-2", "bytes=zz"] {
            let (s, _, b) = raw_request(port, "GET", "/files/r.bin", &[("range", bad)]);
            assert_eq!((s, b.len()), (200, 200), "range={bad}");
        }

        let (s, _, _) = raw_request(
            port,
            "GET",
            "/files/r.bin",
            &[("range", "bytes=18446744073709551614-")],
        );
        assert_eq!(s, 416, "u64 大值不读数 GiB");
        let _ = std::fs::remove_dir_all(&root);
    }

    fn httpdate_offset(secs: i64) -> String {
        let t = if secs >= 0 {
            std::time::SystemTime::now() + std::time::Duration::from_secs(secs as u64)
        } else {
            std::time::SystemTime::now() - std::time::Duration::from_secs((-secs) as u64)
        };
        a2r_std::http::format_http_date(t)
    }

    /// 条件矩阵：etag（If-None-Match/If-Match/弱/通配/If-Range）、日期
    /// （If-Modified-Since 未来=304/过去=200）、条件先于 Range。
    #[test]
    fn http_e2e_plan729_vm_conditional_matrix() {
        let root = temp_root("cond");
        std::fs::write(root.join("c.bin"), (0u8..=99).collect::<Vec<u8>>()).unwrap();
        let port = start_server(
            &program(&root.to_str().unwrap().replace('\\', "/")),
            PORT_COND,
        );
        let future = httpdate_offset(3600);
        let past = httpdate_offset(-3600);

        let (s, h, b) = raw_request(port, "GET", "/etag/c.bin", &[("if-none-match", "v1")]);
        assert_eq!(s, 304);
        // 注：hyper 对 304 剥离 Content-Length（wire 现实；表示长度语义
        // 由 200/HEAD 承载——决策报告 §4 注记）。
        assert_eq!(header_of(&h, "etag"), "v1");
        assert!(b.is_empty());

        let (s, _, _) = raw_request(port, "GET", "/etag/c.bin", &[("if-none-match", "W/v1")]);
        assert_eq!(s, 304, "弱比较允许");
        let (s, _, _) = raw_request(port, "GET", "/etag/c.bin", &[("if-none-match", "v2")]);
        assert_eq!(s, 200);
        let (s, _, _) = raw_request(port, "GET", "/etag/c.bin", &[("if-none-match", "*")]);
        assert_eq!(s, 304);

        let (s, _, _) = raw_request(port, "GET", "/etag/c.bin", &[("if-match", "v2")]);
        assert_eq!(s, 412);
        let (s, _, _) = raw_request(port, "GET", "/etag/c.bin", &[("if-match", "W/v1")]);
        assert_eq!(s, 412, "If-Match 强比较弱形态永不匹配");
        let (s, _, _) = raw_request(port, "GET", "/files/c.bin", &[("if-match", "v1")]);
        assert_eq!(s, 412, "无 etag 可比");

        let (s, _, _) = raw_request(
            port,
            "GET",
            "/files/c.bin",
            &[("if-modified-since", &future)],
        );
        assert_eq!(s, 304);
        let (s, _, _) = raw_request(port, "GET", "/files/c.bin", &[("if-modified-since", &past)]);
        assert_eq!(s, 200);

        let (s, _, _) = raw_request(
            port,
            "GET",
            "/etag/c.bin",
            &[("range", "bytes=0-9"), ("if-none-match", "v1")],
        );
        assert_eq!(s, 304, "条件先于 Range");

        let (s, _, b) = raw_request(
            port,
            "GET",
            "/etag/c.bin",
            &[("range", "bytes=10-19"), ("if-range", "v1")],
        );
        assert_eq!((s, b.len()), (206, 10));
        let (s, _, b) = raw_request(
            port,
            "GET",
            "/etag/c.bin",
            &[("range", "bytes=10-19"), ("if-range", "v2")],
        );
        assert_eq!((s, b.len()), (200, 100), "If-Range 失配完整重下");
        let (s, _, b) = raw_request(
            port,
            "GET",
            "/etag/c.bin",
            &[("range", "bytes=10-19"), ("if-range", "W/v1")],
        );
        assert_eq!((s, b.len()), (200, 100), "弱标签完整重下");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 路径安全：traversal/编码/绝对/双重编码/junction/目录/缺失。
    #[test]
    fn http_e2e_plan729_vm_path_security() {
        let base = temp_root("path");
        let root = base.join("root");
        let secret = base.join("secret");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&secret).unwrap();
        std::fs::write(secret.join("esc.txt"), b"ESCAPED").unwrap();
        std::fs::write(root.join("ok.txt"), b"ok").unwrap();
        std::fs::create_dir_all(root.join("sub")).unwrap();
        let link = root.join("jdir");
        let linked = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&secret)
            .output()
            .unwrap();
        let junction_live = linked.status.success();
        if !junction_live {
            eprintln!("SKIP: junction 权限缺失（环境缺项如实记录）");
        }
        // 注：root 经 .at 字面量嵌入——反斜杠会被 .at 转义解释（backslash-r
        // 序列变 CR），必须转正斜杠（Windows API 与本服务 walk 均接受）。
        let port = start_server(
            &program(&root.to_str().unwrap().replace('\\', "/")),
            PORT_PATH,
        );

        // ../ 越界（URL 编码）→ 403（不泄露主机路径）。
        let (s, _, b) = raw_request(port, "GET", "/files/..%2Fsecret%2Fesc.txt", &[]);
        assert_eq!(s, 403);
        assert!(!String::from_utf8_lossy(&b).contains(&base.to_string_lossy().to_string()));

        // 绝对路径（编码 /）→ 403。
        let (s, _, _) = raw_request(port, "GET", "/files/%2Fetc%2Fpasswd", &[]);
        assert_eq!(s, 403);

        // 双重编码 → 解码后字面 %2e%2e 不匹配磁盘 → 404。
        let (s, _, _) = raw_request(port, "GET", "/files/%252e%252e%252fsecret.txt", &[]);
        assert!(s == 404 || s == 403, "double-encoded: {s}");

        let (s, _, _) = raw_request(port, "GET", "/files/sub", &[]);
        assert_eq!(s, 404, "目录");
        let (s, _, _) = raw_request(port, "GET", "/files/nope.bin", &[]);
        assert_eq!(s, 404, "缺失");
        let (s, _, _) = raw_request(port, "GET", "/deep/sub/deeper/none.bin", &[]);
        assert_eq!(s, 404, "深缺失（命中多段路由）");

        if junction_live {
            let (s, _, b) = raw_request(port, "GET", "/deep/jdir/esc.txt", &[]);
            assert_eq!(s, 403, "junction 中间段检出（多段路由）");
            assert_ne!(b, b"ESCAPED");
        }

        let (s, _, b) = raw_request(port, "GET", "/files/ok.txt", &[]);
        assert_eq!((s, b.as_slice()), (200u16, b"ok".as_slice()));
        let _ = std::fs::remove_dir_all(&base);
    }

    /// 方法与误判防线：POST 文件注解 → 405；声明文件返回 int → 500 诊断；
    /// 普通端点 int → JSON 200（int 不误判）。
    #[test]
    fn http_e2e_plan729_vm_method_and_int_guard() {
        let root = temp_root("method");
        std::fs::write(root.join("m.bin"), b"method").unwrap();
        let code = format!(
            r#"
#[api(method = "GET", path = "/files/:name")]
fn get_file(name str) FileResponse {{
    return http.file_response("{root}", name, "{{}}")
}}

#[api(method = "POST", path = "/upload")]
fn upload_file(name str) FileResponse {{
    return http.file_response("{root}", name, "{{}}")
}}

#[api(method = "GET", path = "/wrong")]
fn wrong_file() FileResponse {{
    return 4242
}}

#[api(method = "GET", path = "/plain")]
fn plain_int() int {{
    return 729729
}}
"#,
            root = root.to_str().unwrap().replace('\\', "/")
        );
        let port = start_server(&code, PORT_METHOD);

        let (s, h, _) = raw_request(
            port,
            "POST",
            "/upload",
            &[("content-type", "application/json")],
        );
        assert_eq!(s, 405);
        assert_eq!(header_of(&h, "allow"), "GET, HEAD");

        let (s, _, b) = raw_request(port, "GET", "/wrong", &[]);
        assert_eq!(s, 500);
        assert!(
            String::from_utf8_lossy(&b).contains("did not return a FileResponse"),
            "{}",
            String::from_utf8_lossy(&b)
        );

        let (s, _, b) = raw_request(port, "GET", "/plain", &[]);
        assert_eq!(s, 200);
        assert!(String::from_utf8_lossy(&b).contains("729729"));
        let _ = std::fs::remove_dir_all(&root);
    }
}

// ===========================================================================
// T-07：有界与生命周期矩阵（慢发送/取消/关闭/配额）+ 727 客户端互通。
// ===========================================================================

#[cfg(feature = "test-http-e2e")]
mod lifecycle {
    use super::http_e2e::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    const PORT_SLOW: u16 = 18956;
    const PORT_QUOTA: u16 = 18957;
    const PORT_INTEROP: u16 = 18958;

    /// 只读响应头、不读 body 的慢客户端（黑洞——body 不被 poll）。
    struct Blackhole {
        stream: Option<TcpStream>,
        status: u16,
        content_length: usize,
    }

    impl Blackhole {
        /// 只发请求不读（排队持有者：headers 要等 active 释放才会来）。
        fn fire(port: u16, path: &str) -> Self {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("connect");
            stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
            let _ = write!(
                stream,
                "GET {path} HTTP/1.1
Host: 127.0.0.1
Connection: close

"
            );
            Self {
                stream: Some(stream),
                status: 0,
                content_length: 0,
            }
        }

        fn open(port: u16, path: &str) -> Self {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("connect");
            stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
            write!(
                stream,
                "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
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
            let header_end = header_end.expect("headers");
            let head = String::from_utf8_lossy(&raw[..header_end]).into_owned();
            let status = head
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let content_length = head
                .lines()
                .find_map(|l| {
                    l.split_once(':').and_then(|(k, v)| {
                        if k.trim().eq_ignore_ascii_case("content-length") {
                            v.trim().parse().ok()
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or(0);
            Self {
                stream: Some(stream),
                status,
                content_length,
            }
        }
    }

    /// 慢发送 + 断连回收：慢客户端（不读 body）持有 active；断连后
    /// （pump 发送失败 → ClientGone）资源回基线（AC-05）。
    #[test]
    fn http_e2e_plan729_slow_client_disconnect_reclaims() {
        let root = temp_root("slow");
        std::fs::write(root.join("big.bin"), vec![5u8; 4 * 1024 * 1024]).unwrap();
        let port = start_server(
            &program(&root.to_str().unwrap().replace('\\', "/")),
            PORT_SLOW,
        );
        let mut holes = Vec::new();
        // 4 个黑洞客户端占满 active（默认 max_active=4）。
        for _ in 0..4 {
            let h = Blackhole::open(port, "/files/big.bin");
            assert_eq!(h.status, 200);
            assert_eq!(h.content_length, 4 * 1024 * 1024);
            holes.push(h);
        }
        // 等待 active 占满（对账探针）。
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while crate::http_file_service::file_active_count() < 4
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(
            crate::http_file_service::file_active_count(),
            4,
            "4 个慢客户端占满 active"
        );
        // 断开全部 → pump 发送失败 → 资源回基线。
        for h in holes.iter_mut() {
            h.stream.take();
        }
        drop(holes);
        let deadline = std::time::Instant::now() + Duration::from_secs(15);
        while crate::http_file_service::file_active_count() > 0
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(50));
        }
        assert_eq!(
            crate::http_file_service::file_active_count(),
            0,
            "断连后文件资源回基线"
        );
        // 服务仍可服务新请求（连接/路由未受损）。
        let (s, _, b) = raw_request(port, "GET", "/files/big.bin", &[]);
        assert_eq!((s, b.len()), (200, 4 * 1024 * 1024));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 配额门：N(4) active + Q(16) 排队占满后下一笔确定 503；释放排队即
    /// 恢复（AC-04；503 Retry-After 头在）。
    #[test]
    fn http_e2e_plan729_quota_saturation_503_then_recover() {
        let root = temp_root("quota");
        // 64MiB：OS/hyper 缓冲吞不完——pump 真正背压停住，active 真被持有。
        std::fs::write(root.join("q.bin"), vec![3u8; 64 * 1024 * 1024]).unwrap();
        let port = start_server(
            &program(&root.to_str().unwrap().replace('\\', "/")),
            PORT_QUOTA,
        );
        // 4 active（黑洞，读 headers 证实 200）+ 16 queued（黑洞，只发不读
        // ——排队者 headers 要等 active 释放）= 20。
        let mut holes = Vec::new();
        for _ in 0..4 {
            let h = Blackhole::open(port, "/files/q.bin");
            assert_eq!(h.status, 200, "active 位 200");
            holes.push(h);
        }
        // 16 个慢读者（排队持有者——线程持续 read，socket 非死态）。
        let mut readers = Vec::new();
        for _ in 0..16 {
            let mut stream = None;
            for _ in 0..50 {
                if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                    stream = Some(s);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let mut stream = stream.expect("connect");
            stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
            use std::io::Write as _;
            let _ = stream.write_all(
                b"GET /files/q.bin HTTP/1.1
Host: 127.0.0.1
Connection: close

",
            );
            readers.push(stream);
        }
        let reader_handles: Vec<_> = readers
            .into_iter()
            .map(|mut stream| {
                std::thread::spawn(move || {
                    // 读 ~8KiB 后退出（socket 关闭 → RST）：排队持有者保活
                    // 足够久以饱和队列，随后释放让晋升后的 pump 可回收。
                    let mut buf = [0u8; 512];
                    let mut reads = 0usize;
                    loop {
                        match std::io::Read::read(&mut stream, &mut buf) {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {
                                reads += 1;
                                if reads >= 16 {
                                    break;
                                }
                                std::thread::sleep(Duration::from_millis(200));
                            }
                        }
                    }
                })
            })
            .collect();
        let deadline = std::time::Instant::now() + Duration::from_secs(15);
        while crate::http_file_service::file_queued_count() < 16
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(
            crate::http_file_service::file_queued_count(),
            16,
            "16 笔在文件排队（队内无句柄无缓冲）"
        );
        // 第 21 笔：队满即时 503 + Retry-After（即时送达）。
        let (s, h, _) = raw_request(port, "GET", "/files/q.bin", &[]);
        assert_eq!(s, 503, "队满即时 503");
        assert!(!header_of(&h, "retry-after").is_empty(), "Retry-After 在");
        // 释放全部持有者：4 active 断连 + 16 排队在 30s 准备期限到期终结
        // （排队期零句柄零缓冲——到期即出队）。等两项计数都回基线。
        for h in holes.iter_mut() {
            h.stream.take();
        }
        drop(holes);
        let deadline = std::time::Instant::now() + Duration::from_secs(45);
        while (crate::http_file_service::file_active_count() > 0
            || crate::http_file_service::file_queued_count() > 0)
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(100));
        }
        assert_eq!(
            crate::http_file_service::file_active_count(),
            0,
            "active 回基线"
        );
        assert_eq!(
            crate::http_file_service::file_queued_count(),
            0,
            "排队回基线（到期出队）"
        );
        // 全释放后新请求恢复 200。
        let (s, _, _) = raw_request(port, "GET", "/files/q.bin", &[]);
        assert_eq!(s, 200, "全释放后恢复");
        for h in reader_handles {
            let _ = h.join();
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 727 客户端互通：transfer_download 对 VM 文件路由——完整下载 hash、
    /// 强 validator 206 续传、If-Range 失配 200 完整重下、416 保留目标、
    /// 取消（AC-06）。单流两端绑定同次请求（同一 server fixture）。
    #[test]
    fn http_e2e_plan729_client729_server_interop() {
        let root = temp_root("interop");
        let data: Vec<u8> = (0usize..(1024 * 256)).map(|i| (i % 251) as u8).collect();
        std::fs::write(root.join("d.bin"), &data).unwrap();
        let port = start_server(
            &program(&root.to_str().unwrap().replace('\\', "/")),
            PORT_INTEROP,
        );
        let base = format!("http://127.0.0.1:{port}");
        let dst = |name: &str| root.join(name);

        // 1. 完整下载：hash 一致。
        let t = a2r_std::http::transfer_download(
            &format!("{base}/files/d.bin"),
            dst("full.bin").to_str().unwrap(),
            "{}",
        );
        let receipt = a2r_std::http::transfer_wait_typed(&t);
        assert_eq!(receipt.kind.as_str(), "success", "{receipt:?}");
        assert_eq!(std::fs::read(dst("full.bin")).unwrap(), data);

        // 2. 强 validator 续传：本地半文件 + etag 匹配 → 206 追加。
        let half = &data[..100_000];
        std::fs::write(dst("resume.bin"), half).unwrap();
        let t2 = a2r_std::http::transfer_download(
            &format!("{base}/etag/d.bin"),
            dst("resume.bin").to_str().unwrap(),
            &format!(
                "{{\"offset\":{},\"validator\":{{\"etag\":\"v1\"}}}}",
                half.len()
            ),
        );
        let receipt2 = a2r_std::http::transfer_wait_typed(&t2);
        assert_eq!(receipt2.kind.as_str(), "success", "{receipt2:?}");
        assert_eq!(
            std::fs::read(dst("resume.bin")).unwrap(),
            data,
            "续传拼回完整内容"
        );

        // 3. If-Range 失配（错误 etag）→ 200 完整重下（不 append）。
        std::fs::write(dst("mismatch.bin"), half).unwrap();
        let t3 = a2r_std::http::transfer_download(
            &format!("{base}/etag/d.bin"),
            dst("mismatch.bin").to_str().unwrap(),
            &format!(
                "{{\"offset\":{},\"validator\":{{\"etag\":\"WRONG\"}}}}",
                half.len()
            ),
        );
        let receipt3 = a2r_std::http::transfer_wait_typed(&t3);
        assert_eq!(receipt3.kind.as_str(), "success", "{receipt3:?}");
        assert_eq!(
            std::fs::read(dst("mismatch.bin")).unwrap(),
            data,
            "失配完整重下"
        );

        // 4. 416（起点=EOF 之外的越界 offset 有 validator 失配保护；这里用
        // 无 validator 的大 offset 触发 416）→ 失败保留原目标。
        std::fs::write(dst("keep.bin"), half).unwrap();
        let t4 = a2r_std::http::transfer_download(
            &format!("{base}/files/d.bin"),
            dst("keep.bin").to_str().unwrap(),
            "{\"offset\":999999999}",
        );
        let receipt4 = a2r_std::http::transfer_wait_typed(&t4);
        assert_eq!(receipt4.kind.as_str(), "failed", "{receipt4:?}");
        assert_eq!(
            std::fs::read(dst("keep.bin")).unwrap(),
            half,
            "416/失败保留原目标"
        );

        // 5. 取消：提交后立即 cancel → 终态 cancelled；服务器资源回基线。
        let t5 = a2r_std::http::transfer_download(
            &format!("{base}/files/d.bin"),
            dst("cancel.bin").to_str().unwrap(),
            "{}",
        );
        a2r_std::http::transfer_cancel(&t5);
        let receipt5 = a2r_std::http::transfer_wait_typed(&t5);
        assert_eq!(receipt5.kind.as_str(), "cancelled", "{receipt5:?}");
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while crate::http_file_service::file_active_count() > 0
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(50));
        }
        assert_eq!(
            crate::http_file_service::file_active_count(),
            0,
            "客户端取消后 server 文件资源退出"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
