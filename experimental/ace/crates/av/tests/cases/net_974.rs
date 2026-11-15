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
    let program = compile(src, "d_main").expect("compile net source");
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
