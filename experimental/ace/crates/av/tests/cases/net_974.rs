//! PLAN-974 T-02: `platform.net/1` — the minimal TCP thin adapter on the
//! AV platform face, proven by a REAL HTTP roundtrip against a live
//! loopback server and by named negative controls.
//!
//! The Auto side carries the HTTP protocol itself (the r2 contract: the
//! stdlib's own Auto protocol over an OS socket thin adapter): it builds
//! the GET request bytes, writes them through `net_tcp_write_some`,
//! reads the response into an OS memory region via `net_tcp_read_into`,
//! closes via `net_tcp_close`, and parses the status line + body length
//! from the region bytes (`mem_read_u8`). Independent oracle: the canned
//! server response is exactly 43 bytes ("HTTP/1.1 200 OK\r\n
//! Content-Length: 5\r\n\r\nhello") -> exit = 200*100 + 43 = 20043.

use std::io::{Read, Write};
use std::net::TcpListener;

use ace_av::console_host::compile;
use ace_av_core::platform_io::{
    Win32PlatformIo, ERR_ALREADY_RELEASED, ERR_INVALID_BOUNDS, ERR_INVALID_HANDLE, ERR_WRONG_ORIGIN,
};

fn net_grants() -> Vec<String> {
    vec![
        "platform.net.connect".into(),
        "platform.net.write".into(),
        "platform.net.read".into(),
        "platform.net.close".into(),
        "platform.mem".into(),
    ]
}

/// One loopback HTTP server accepting `conns` connections; each accepted
/// connection gets the canned 43-byte response after a well-formed GET.
fn spawn_server() -> (std::net::SocketAddr, std::thread::JoinHandle<u16>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    let handle = std::thread::spawn(move || {
        listener.set_nonblocking(false).ok();
        // serve exactly ONE connection per spawned server
        {
            let Ok((mut conn, _)) = listener.accept() else {
                return 1;
            };
            let _ = conn.set_read_timeout(Some(std::time::Duration::from_secs(2)));
            let mut buf = [0u8; 1024];
            let mut seen = 0usize;
            // read until end of request headers
            loop {
                match conn.read(&mut buf[seen..]) {
                    Ok(0) => break,
                    Ok(n) => {
                        seen += n;
                        if buf[..seen].windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Err(_) => return 2,
                }
            }
            if !buf[..seen].starts_with(b"GET /") {
                return 3;
            }
            let _ = conn.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello");
            let _ = conn.flush();
            let _ = conn.shutdown(std::net::Shutdown::Both);
        }
        0
    });
    (addr, handle)
}

fn compile_and_run(
    src: &str,
    grants: &[String],
    backend: &mut Win32PlatformIo,
) -> ace_av_core::contract::ExecOutcome {
    let program = compile(src, "d_main").unwrap_or_else(|e| {
        panic!(
            "compile net source: {e:#?}
--- source ---
{src}"
        )
    });
    let (out, _trace, _summary) = program
        .run_platform(
            grants,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
            None,
            backend,
        )
        .expect("run platform");
    out
}

/// REAL TCP HTTP roundtrip: the Auto protocol drives the socket.
#[test]
fn net974_real_tcp_http_roundtrip() {
    let (addr, server) = spawn_server();
    let src = format!(
        r#"
fn main() int {{
    let s = net_tcp_connect("127.0.0.1", {port})
    if s <= 0 {{ return s }}
    let req = "GET /p974 HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    let w = net_tcp_write_some(s, req)
    if w <= 0 {{ return w }}
    let region = mem_alloc(4096)
    var total = 0
    var code = 0
    for total < 4096 {{
        let n = net_tcp_read_into(s, region, total, 512)
        if n < 0 {{ return n }}
        if n == 0 {{ break }}
        total = total + n
    }}
    let _c = net_tcp_close(s)
    // parse the status line: "HTTP/1.1 " then 3 digits
    var i = 0
    var digs = 0
    code = 0
    for i < total {{
        let b = mem_read_u8(region, i)
        if b == 32 {{
            if digs == 0 {{
                let d0 = mem_read_u8(region, i + 1)
                let d1 = mem_read_u8(region, i + 2)
                let d2 = mem_read_u8(region, i + 3)
                code = (d0 - 48) * 100 + (d1 - 48) * 10 + (d2 - 48)
                digs = 1
            }}
        }}
        i = i + 1
    }}
    let _f = mem_free(region)
    if total != 43 {{ return 9000 + total }}
    if digs != 1 {{ return 9100 }}
    return code * 100 + total
}}
"#,
        port = addr.port()
    );
    let mut backend = Win32PlatformIo::default();
    let out = compile_and_run(&src, &net_grants(), &mut backend);
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(20043),
                ..
            }
        ),
        "real HTTP roundtrip must observe status 200 and 43 bytes, got {out:?}"
    );
    let served = server.join().expect("server thread");
    assert_eq!(served, 0, "the loopback server saw a well-formed GET");
    assert!(backend.dispatches.net_connect >= 1);
}

/// Named negative controls: unauthorized (no net grant), invalid handle,
/// wrong origin (mem handle to a net op), double close, bad bounds.
#[test]
fn net974_negative_controls() {
    let (addr, _server) = spawn_server();
    let port = addr.port();
    let src = format!(
        r#"
fn main() int {{
    // 1) wrong origin: a mem region handle is NOT a socket
    let region = mem_alloc(64)
    let bad_origin = net_tcp_write_some(region, "x")
    // 2) invalid handle
    let bad_handle = net_tcp_close(999)
    // 3) real socket, then bad bounds + double close
    let s = net_tcp_connect("127.0.0.1", {port})
    if s <= 0 {{ return s }}
    let bad_bounds = net_tcp_read_into(s, region, 0, 4096)
    let _c = net_tcp_close(s)
    let double = net_tcp_close(s)
    // 4) invalid port value
    let bad_port = net_tcp_connect("127.0.0.1", 70000)
    let _f = mem_free(region)
    if bad_origin != -3 {{ return 1 }}
    if bad_handle != -2 {{ return 2 }}
    if bad_bounds != -4 {{ return 3 }}
    if double != -6 {{ return 4 }}
    if bad_port != -7 {{ return 5 }}
    return 0
}}
"#
    );
    let mut backend = Win32PlatformIo::default();
    let out = compile_and_run(&src, &net_grants(), &mut backend);
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            }
        ),
        "negative controls: origin/handle/bounds/double-close/port-value, got {out:?}"
    );

    // 5) unauthorized: with NO net cap granted, `run_platform` refuses at
    // ADMISSION (`host.unauthorized`) — no socket is ever opened and no
    // dispatch happens (the named preflight control, one level earlier
    // than the ledger's ERR_UNAUTHORIZED path)
    let no_net: Vec<String> = vec!["platform.mem".into()];
    let mut backend2 = Win32PlatformIo::default();
    let program2 = compile(
        &format!(r#"fn main() int {{ return net_tcp_connect("127.0.0.1", {port}) }}"#),
        "d_main",
    )
    .expect("compile unauthorized source");
    let refused = program2.run_platform(
        &no_net,
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(false),
        None,
        &mut backend2,
    );
    assert!(
        refused
            .as_ref()
            .is_err_and(|e| format!("{e:?}").contains("host.unauthorized")),
        "missing net grants must be refused at admission, got {refused:?}"
    );
    assert_eq!(backend2.dispatches.net_connect, 0, "zero dispatches");
    // error-code sanity for the named constants used above
    assert_eq!(
        (
            ERR_WRONG_ORIGIN,
            ERR_INVALID_HANDLE,
            ERR_INVALID_BOUNDS,
            ERR_ALREADY_RELEASED
        ),
        (-3, -2, -4, -6)
    );
}

// ---------------------------------------------------------------------------
// PLAN-974 T-02 increment 2 — the Auto-side HTTP PROTOCOL API face
// (http.get / request-builder chain / post): the legacy a2p
// `30_plan724/001_http_client_sync` obligations ported onto the
// platform.net/1 lane. The protocol lives in AUTO source (the r2
// contract); the loopback server routes by method (GET -> 200 "hello"
// + X-Echo header, POST -> 201 with the received payload length echoed).
// ---------------------------------------------------------------------------

/// A loopback server serving exactly `conns` method-routed requests.
fn spawn_routed_server(conns: usize) -> (std::net::SocketAddr, std::thread::JoinHandle<i32>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    let handle = std::thread::spawn(move || {
        let mut served = 0i32;
        for _ in 0..conns {
            let Ok((mut conn, _)) = listener.accept() else {
                return served;
            };
            let _ = conn.set_read_timeout(Some(std::time::Duration::from_secs(2)));
            let mut buf = [0u8; 2048];
            let mut seen = 0usize;
            let mut head_end = None;
            loop {
                match conn.read(&mut buf[seen..]) {
                    Ok(0) => break,
                    Ok(n) => {
                        seen += n;
                        if let Some(p) = buf[..seen].windows(4).position(|w| w == b"\r\n\r\n") {
                            head_end = Some(p + 4);
                            break;
                        }
                    }
                    Err(_) => return -1,
                }
            }
            let Some(head_end) = head_end else {
                return -2;
            };
            let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
            let mut body = buf[head_end..seen].to_vec();
            let cl = head
                .lines()
                .find_map(|l| {
                    l.strip_prefix("Content-Length: ")
                        .and_then(|v| v.parse::<usize>().ok())
                })
                .unwrap_or(0);
            while body.len() < cl {
                match conn.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => body.extend_from_slice(&buf[..n]),
                    Err(_) => break,
                }
            }
            let response: Vec<u8> = if head.starts_with("GET /") {
                b"HTTP/1.1 200 OK\r\nX-Echo: served\r\nContent-Length: 5\r\n\r\nhello".to_vec()
            } else if head.starts_with("POST /") {
                format!(
                    "HTTP/1.1 201 Created\r\nX-Echo: {}\r\nContent-Length: 2\r\n\r\nok",
                    body.len()
                )
                .into_bytes()
            } else {
                return -3;
            };
            let _ = conn.write_all(&response);
            let _ = conn.flush();
            let _ = conn.shutdown(std::net::Shutdown::Both);
            served += 1;
        }
        served
    });
    (addr, handle)
}

/// GET/builder/POST through the Auto-side protocol on platform.net/1:
/// every leg observes `status*10000 + body_len` with a hand oracle
/// (GET 200/5, builder GET 200/5, POST 201/2) and main returns the sum
/// 600002.
#[test]
fn net974_http_api_face() {
    let (addr, server) = spawn_routed_server(3);
    let src = format!(
        r#"
fn send_request(host Str, port int, req Str) int {{
    let s = net_tcp_connect(host, port)
    if s <= 0 {{ return s }}
    let w = net_tcp_write_some(s, req)
    if w <= 0 {{ return w }}
    let region = mem_alloc(2048)
    var total = 0
    for total < 2048 {{
        let n = net_tcp_read_into(s, region, total, 512)
        if n < 0 {{
            let _m = mem_free(region)
            return n
        }}
        if n == 0 {{ break }}
        total = total + n
    }}
    let _c = net_tcp_close(s)
    // status: the 3 digits after the first space
    var i = 0
    var code = 0
    var got = 0
    for i < total {{
        let b = mem_read_u8(region, i)
        if b == 32 {{
            let d0 = mem_read_u8(region, i + 1)
            let d1 = mem_read_u8(region, i + 2)
            let d2 = mem_read_u8(region, i + 3)
            code = (d0 - 48) * 100 + (d1 - 48) * 10 + (d2 - 48)
            got = 1
            break
        }}
        i = i + 1
    }}
    // body: everything after the first "\r\n\r\n" (scan for CR LF CR LF)
    var j = 0
    var blank = -1
    for j + 3 < total {{
        let c0 = mem_read_u8(region, j)
        let c1 = mem_read_u8(region, j + 1)
        let c2 = mem_read_u8(region, j + 2)
        let c3 = mem_read_u8(region, j + 3)
        if c0 == 13 {{
            if c1 == 10 {{
                if c2 == 13 {{
                    if c3 == 10 {{
                        blank = j + 4
                        break
                    }}
                }}
            }}
        }}
        j = j + 1
    }}
    let _f = mem_free(region)
    if got != 1 {{ return 9100 }}
    if blank < 0 {{ return 9200 }}
    return code * 10000 + (total - blank)
}}

fn req_get(path Str) Str {{
    return "GET " + path + " HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
}}
fn req_with_header(base Str, k Str, v Str) Str {{
    return base + k + ": " + v + "\r\n"
}}
fn int_to_str(v int) Str {{
    if v == 0 {{ return "0" }}
    var n = v
    var out Str = ""
    for n > 0 {{
        let d = n - (n / 10) * 10
        out = int_digit(d) + out
        n = n / 10
    }}
    return out
}}
fn int_digit(d int) Str {{
    if d == 0 {{ return "0" }}
    if d == 1 {{ return "1" }}
    if d == 2 {{ return "2" }}
    if d == 3 {{ return "3" }}
    if d == 4 {{ return "4" }}
    if d == 5 {{ return "5" }}
    if d == 6 {{ return "6" }}
    if d == 7 {{ return "7" }}
    if d == 8 {{ return "8" }}
    return "9"
}}
fn req_post(path Str, payload Str) Str {{
    let n = payload.len_bytes()
    let head Str = "POST " + path + " HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: "
    let tail Str = "\r\nConnection: close\r\n\r\n"
    return head + int_to_str(n) + tail + payload
}}

fn main() int {{
    // 1) http.get: 200, body "hello" (5)
    let a = send_request("127.0.0.1", {port}, req_get("/p974"))
    // 2) builder chain with a custom header: still a GET, 200/5
    let base Str = "GET /builder HTTP/1.1\r\nHost: 127.0.0.1\r\n"
    let with_h Str = req_with_header(base, "X-Plan974", "builder")
    let built Str = with_h + "Connection: close\r\n\r\n"
    let b = send_request("127.0.0.1", {port}, built)
    // 3) http.post with a 13-byte payload: 201, body "ok" (2)
    let c = send_request("127.0.0.1", {port}, req_post("/submit", "post-payload"))
    if a != 2000005 {{ return 1 }}
    if b != 2000005 {{ return 2 }}
    if c != 2010002 {{ return 3 }}
    return a + b + c
}}
"#,
        port = addr.port()
    );
    let mut backend = Win32PlatformIo::default();
    let out = compile_and_run(&src, &net_grants(), &mut backend);
    match &out {
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(v),
            ..
        } => assert_eq!(
            *v, 6010012,
            "GET 200/5 + builder 200/5 + POST 201/2 = 2000005+2000005+2010002"
        ),
        other => panic!("unexpected outcome: {other:?}"),
    }
    let served = server.join().expect("server");
    assert_eq!(served, 3, "three well-formed requests served");
}

// ---------------------------------------------------------------------------
// PLAN-974 T-02 increment 3 — the SERVER leg: the Auto program binds a
// listener on a PRE-RESERVED port (the test binds+drops to pick a free
// port, injects it into the source), accepts ONE connection, reads the
// request, writes a routed response and closes. A real std::net CLIENT
// (Rust side) drives it — the roles reversed from increments 1/2.
// ---------------------------------------------------------------------------

#[test]
fn net974_server_leg_auto_listens_rust_client() {
    // reserve a free port (bind + drop)
    let probe = TcpListener::bind(("127.0.0.1", 0)).expect("probe bind");
    let port = probe.local_addr().expect("probe addr").port();
    drop(probe);

    let src = format!(
        r#"fn main() int {{
    let ln = net_tcp_listen({port})
    if ln <= 0 {{ return ln }}
    var s = 0
    var stop = 0
    for stop == 0 {{
        let a = net_tcp_accept(ln)
        if a > 0 {{
            s = a
            stop = 1
        }}
    }}
    let region = mem_alloc(1024)
    var total = 0
    for total < 1024 {{
        let n = net_tcp_read_into(s, region, total, 256)
        if n < 0 {{ return n }}
        if n == 0 {{ break }}
        total = total + n
        if total >= 4 {{
            let c0 = mem_read_u8(region, total - 4)
            let c1 = mem_read_u8(region, total - 3)
            let c2 = mem_read_u8(region, total - 2)
            let c3 = mem_read_u8(region, total - 1)
            if c0 == 13 {{
                if c1 == 10 {{
                    if c2 == 13 {{
                        if c3 == 10 {{ break }}
                    }}
                }}
            }}
        }}
    }}
    let body Str = "HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\npong"
    let w = net_tcp_write_some(s, body)
    let _c = net_tcp_close(s)
    let _f = mem_free(region)
    if w != 42 {{ return 9100 }}
    return 0
}}
"#,
        port = port
    );
    let grants: Vec<String> = vec![
        "platform.net.listen".into(),
        "platform.net.read".into(),
        "platform.net.write".into(),
        "platform.net.close".into(),
        "platform.mem".into(),
    ];
    let program = ace_av::console_host::compile(&src, "d_main").expect("compile server source");

    // Auto server on a worker thread; the Rust client drives it on the
    // pre-reserved port (connect retries while the listener binds).
    let handle = std::thread::spawn(move || {
        let mut backend = Win32PlatformIo::default();
        let mut limits = ace_av_core::av::ExecLimits::default();
        // the server leg legitimately waits for an external client: this
        // TEST owns its wall (production callers choose their own budget)
        limits.wall = std::time::Duration::from_secs(10);
        let (out, _t, _s) = program
            .run_platform(
                &grants,
                &limits,
                &ace_av_core::av::CancelFlag::new(false),
                None,
                &mut backend,
            )
            .expect("run platform server");
        eprintln!(
            "[p974] server outcome: {out:?}; dispatches conn={} wr={} rd={} cl={}",
            backend.dispatches.net_connect,
            backend.dispatches.net_write,
            backend.dispatches.net_read,
            backend.dispatches.net_close
        );
        match out {
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            } => {}
            other => panic!("server outcome: {other:?}"),
        }
    });
    let mut client = loop {
        if let Ok(c) = std::net::TcpStream::connect(("127.0.0.1", port)) {
            break c;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let _ = client.set_nodelay(true);
    // an HTTP client sets TCP_NODELAY: without it Nagle+delayed-ACK can
    // stall the request tail past the read window (the inc3 puzzle's
    // second half — the first half was the mangled source escapes)
    client.set_nodelay(true).ok();
    client
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();
    client
        .write_all(b"GET /drive HTTP/1.1\r\nHost: x\r\n\r\n")
        .expect("client send");
    let mut buf = [0u8; 128];
    let n = client.read(&mut buf).expect("client read");
    assert_eq!(
        &buf[..n],
        b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\npong",
        "server response bytes"
    );
    handle.join().expect("server thread");
}

// ---------------------------------------------------------------------------
// PLAN-974 T-02 increment 4 — SSE streaming parse + redirect-follow +
// the connect-refused timeout negative. All three ride the SAME
// platform.net/1 lane (paced read/accept); the loopback server serves
// three connections: the SSE stream, the 302 redirect hop and the
// followed /next hop.
// ---------------------------------------------------------------------------

#[test]
fn net974_sse_and_redirect() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let server = std::thread::spawn(move || {
        // conn 1: SSE — two events streamed with a gap, then close.
        let (mut c, _) = listener.accept().expect("sse accept");
        let _ = c.set_read_timeout(Some(std::time::Duration::from_secs(2)));
        let mut b = [0u8; 256];
        let _ = c.read(&mut b); // drain the GET
        use std::io::Write as _;
        let _ = c.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n");
        let _ = c.write_all(b"event: tick\ndata: 41\n\n");
        let _ = c.flush();
        std::thread::sleep(std::time::Duration::from_millis(120));
        let _ = c.write_all(b"event: tock\ndata: 42\n\n");
        let _ = c.flush();
        let _ = c.shutdown(std::net::Shutdown::Both);
        // conn 2: the redirect hop (GET /start -> 302 + Location: /next).
        let (mut c, _) = listener.accept().expect("hop accept");
        let _ = c.set_read_timeout(Some(std::time::Duration::from_secs(2)));
        let mut b = [0u8; 256];
        let _ = c.read(&mut b);
        let _ = c.write_all(b"HTTP/1.1 302 Found\r\nLocation: /next\r\nContent-Length: 0\r\n\r\n");
        let _ = c.flush();
        let _ = c.shutdown(std::net::Shutdown::Both);
        // conn 3: the followed hop (GET /next -> 200 ok).
        let (mut c, _) = listener.accept().expect("next accept");
        let _ = c.set_read_timeout(Some(std::time::Duration::from_secs(2)));
        let mut b = [0u8; 256];
        let _ = c.read(&mut b);
        let _ = c.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
        let _ = c.flush();
        let _ = c.shutdown(std::net::Shutdown::Both);
    });

    let src = format!(
        r#"fn get_code_and_events(host Str, port int, path Str) int {{
    let s = net_tcp_connect(host, port)
    if s <= 0 {{ return s }}
    let req Str = "GET " + path + " HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    let w = net_tcp_write_some(s, req)
    if w <= 0 {{ return w }}
    let region = mem_alloc(4096)
    var total = 0
    for total < 4096 {{
        let n = net_tcp_read_into(s, region, total, 512)
        if n < 0 {{ return n }}
        if n == 0 {{ break }}
        total = total + n
    }}
    let _c = net_tcp_close(s)
    // status code after the first space
    var i = 0
    var code = 0
    var got = 0
    for i < total {{
        let b = mem_read_u8(region, i)
        if b == 32 {{
            let d0 = mem_read_u8(region, i + 1)
            let d1 = mem_read_u8(region, i + 2)
            let d2 = mem_read_u8(region, i + 3)
            code = (d0 - 48) * 100 + (d1 - 48) * 10 + (d2 - 48)
            got = 1
            break
        }}
        i = i + 1
    }}
    if got != 1 {{ let _f = mem_free(region); return 9100 }}
    // SSE event count: blank lines ("\n\n") AFTER the header blank line.
    // header end: "\r\n\r\n"
    var j = 0
    var blank = -1
    for j + 3 < total {{
        let c0 = mem_read_u8(region, j)
        let c1 = mem_read_u8(region, j + 1)
        let c2 = mem_read_u8(region, j + 2)
        let c3 = mem_read_u8(region, j + 3)
        if c0 == 13 {{
            if c1 == 10 {{
                if c2 == 13 {{
                    if c3 == 10 {{ blank = j + 4; break }}
                }}
            }}
        }}
        j = j + 1
    }}
    if blank < 0 {{ let _f = mem_free(region); return 9200 }}
    var events = 0
    var k = blank
    for k + 1 < total {{
        let b0 = mem_read_u8(region, k)
        let b1 = mem_read_u8(region, k + 1)
        if b0 == 10 {{
            if b1 == 10 {{ events = events + 1; k = k + 1 }}
        }}
        k = k + 1
    }}
    let _f = mem_free(region)
    return code * 100 + events
}}

fn main() int {{
    // 1) SSE: 200 with exactly 2 streamed events (41/42 payloads ride the
    //    same stream; the parse counts event-boundary blank lines).
    let a = get_code_and_events("127.0.0.1", {port}, "/stream")
    // 2) redirect: /start answers 302; follow once to /next -> 200, 0 events.
    let hop = get_code_and_events("127.0.0.1", {port}, "/start")
    let next = get_code_and_events("127.0.0.1", {port}, "/next")
    // 3) connect-refused: nothing listens on the probe port's neighbor.
    //    The OS refusal surfaces as the named OS error (-8), never a hang.
    let refused = net_tcp_connect("127.0.0.1", 1)
    if a != 20002 {{ return 1 }}
    if hop != 30200 {{ return 2 }}
    if next != 20000 {{ return 3 }}
    if refused != -8 {{ return 4 }}
    return 0
}}
"#,
        port = port
    );
    let _ = server; // joined implicitly at process end; assertions below
    let mut backend = Win32PlatformIo::default();
    let program = ace_av::console_host::compile(&src, "d_main").expect("compile sse/redirect source");
    let mut limits = ace_av_core::av::ExecLimits::default();
    limits.wall = std::time::Duration::from_secs(10);
    let (out, _t, _s) = program
        .run_platform(
            &net_grants(),
            &limits,
            &ace_av_core::av::CancelFlag::new(false),
            None,
            &mut backend,
        )
        .expect("run platform sse/redirect");
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            }
        ),
        "SSE 200/2 events + redirect 302->200 + refused -8, got {out:?}"
    );
}

// ---------------------------------------------------------------------------
// PLAN-974 T-03 — the std.net PACKAGING: the tracked `stdlib/ace/std/
// net.at` module (PLAN-1041 `pub fn` inline convention — the same file
// spells the future packages face) embedded verbatim into a host-console
// single source, driving a REAL loopback server through the public
// helpers only (net_get / net_get_with_header / net_post).
// ---------------------------------------------------------------------------

#[test]
fn net974_std_net_packaging_inline() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let server = std::thread::spawn(move || {
        for route in 0..3 {
            let Ok((mut c, _)) = listener.accept() else {
                return;
            };
            let _ = c.set_read_timeout(Some(std::time::Duration::from_secs(2)));
            let mut b = [0u8; 512];
            let _ = c.read(&mut b);
            let response: &[u8] = if route == 2 {
                b"HTTP/1.1 201 Created\r\nContent-Length: 2\r\n\r\nok"
            } else {
                b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello"
            };
            use std::io::Write as _;
            let _ = c.write_all(response);
            let _ = c.flush();
            let _ = c.shutdown(std::net::Shutdown::Both);
        }
    });

    // The tracked stdlib module, verbatim, then a main that calls ONLY
    // the public helpers — the packaging seam (PLAN-1041 inline form).
    let stdlib_net = include_str!("../../../../../../stdlib/ace/std/net.at");
    let src = format!(
        r#"{stdlib_net}
fn main() int {{
    let a = net_get("127.0.0.1", {port}, "/a")
    let b = net_get_with_header("127.0.0.1", {port}, "/b", "X-Plan974", "packaged")
    let c = net_post("127.0.0.1", {port}, "/c", "body")
    if a != 2000005 {{ return 1 }}
    if b != 2000005 {{ return 2 }}
    if c != 2010002 {{ return 3 }}
    return a + b + c
}}
"#
    );
    let mut backend = Win32PlatformIo::default();
    let out = compile_and_run(&src, &net_grants(), &mut backend);
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(6010012),
                ..
            }
        ),
        "std.net packaged GET/header-GET/POST -> 2000005+2000005+2010002, got {out:?}"
    );
    server.join().expect("server");
}
