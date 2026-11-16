// ---------------------------------------------------------------------------
// PLAN-975 T-02 — the redis leg (S3 selection): the RESP protocol IN
// AUTO over the archived-974 platform.net/1 lane. The Rust loopback
// server speaks minimal RESP2 (inline commands in, simple-error/simple-
// string/bulk replies out); the Auto client issues SET/GET/INCR/DEL and
// observes the replies byte-exact. Independent oracle: the server also
// records the commands it received; the assertions require BOTH the
// wire replies AND the server-side record to agree.
//
// Packaging increment: the tracked `stdlib/ace/std/redis.at` module
// (PLAN-1041 inline convention) drives a server that STRICTLY parses
// RESP2 arrays — every "$len" header must match the arg bytes that
// follow, so an encoder off-by-one fails the server-side record.
// ---------------------------------------------------------------------------

use std::io::{Read, Write};
use std::net::TcpListener;

use ace_av_core::platform_io::Win32PlatformIo;

fn redis_grants() -> Vec<String> {
    vec![
        "platform.net.connect".into(),
        "platform.net.write".into(),
        "platform.net.read".into(),
        "platform.net.close".into(),
        "platform.mem".into(),
    ]
}

#[test]
fn redis975_resp_over_platform_net() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let server = std::thread::spawn(move || {
        let mut received: Vec<String> = Vec::new();
        let Ok((mut c, _)) = listener.accept() else {
            return received;
        };
        let _ = c.set_read_timeout(Some(std::time::Duration::from_secs(2)));
        use std::io::Write as _;
        // interactive: read ONE command line, reply, repeat (real RESP
        // request/response — no read-to-EOF deadlock)
        let mut carry = Vec::<u8>::new();
        let mut rbuf = [0u8; 128];
        loop {
            if let Some(pos) = carry.windows(2).position(|w| w == b"\r\n") {
                let line: Vec<u8> = carry.drain(..pos + 2).collect();
                let cmd = String::from_utf8_lossy(&line[..pos]).trim().to_string();
                if cmd.is_empty() {
                    continue;
                }
                received.push(cmd.clone());
                let upper = cmd.to_uppercase();
                let reply: &[u8] = if upper.starts_with("SET K") {
                    b"+OK\r\n"
                } else if upper.starts_with("GET K") {
                    b"$5\r\nhello\r\n"
                } else if upper.starts_with("INCR C") {
                    b":3\r\n"
                } else if upper.starts_with("DEL K") {
                    b":1\r\n"
                } else {
                    b"-ERR unknown command\r\n"
                };
                if c.write_all(reply).is_err() {
                    break;
                }
                let _ = c.flush();
                continue;
            }
            match c.read(&mut rbuf) {
                Ok(0) => break,
                Ok(n) => carry.extend_from_slice(&rbuf[..n]),
                Err(_) => break,
            }
        }
        let _ = c.shutdown(std::net::Shutdown::Both);
        received
    });

    // The RESP client in Auto: one connection, four commands, replies
    // accumulated and classified by first byte (+ $ : -).
    let src = format!(
        r#"fn send_recv(s int, region int, total int, cmd Str) int {{
    let w = net_tcp_write_some(s, cmd + "\r\n")
    if w <= 0 {{ return w }}
    // read one reply line
    var t = total
    for t - total < 256 {{
        let n = net_tcp_read_into(s, region, t, 64)
        if n < 0 {{
            if n != -10 {{ return n }}
        }}
        if n == 0 {{ break }}
        if n > 0 {{ t = t + n }}
        if t >= 2 {{
            let c0 = mem_read_u8(region, t - 2)
            let c1 = mem_read_u8(region, t - 1)
            if c0 == 13 {{
                if c1 == 10 {{ break }}
            }}
        }}
    }}
    return t
}}

fn main() int {{
    let s = net_tcp_connect("127.0.0.1", {port})
    if s <= 0 {{ return s }}
    let region = mem_alloc(2048)
    var total = 0

    total = send_recv(s, region, total, "SET k hello")
    if total <= 0 {{ return 9000 - total }}
    if mem_read_u8(region, 0) != 43 {{ return 1 }}
    // +OK
    if mem_read_u8(region, 1) != 79 {{ return 2 }}
    if mem_read_u8(region, 2) != 75 {{ return 3 }}

    total = send_recv(s, region, total, "GET k")
    // "$5\r\nhello\r\n" starts at byte 5 ("+OK\r\n" is 5 bytes)
    if mem_read_u8(region, 5) != 36 {{ return 9000 + mem_read_u8(region, 5) }}
    if mem_read_u8(region, 6) != 53 {{ return 5 }}
    if mem_read_u8(region, 9) != 104 {{ return 6 }}
    if mem_read_u8(region, 13) != 111 {{ return 7 }}

    total = send_recv(s, region, total, "INCR counter")
    // ":3\r\n" starts at byte 16 (5 + 11)
    if mem_read_u8(region, 16) != 58 {{ return 8 }}
    if mem_read_u8(region, 17) != 51 {{ return 9 }}

    total = send_recv(s, region, total, "DEL k")
    // ":1\r\n" starts at byte 20 (16 + 4)
    if mem_read_u8(region, 20) != 58 {{ return 10 }}
    if mem_read_u8(region, 21) != 49 {{ return 11 }}

    let _c = net_tcp_close(s)
    let _f = mem_free(region)
    return 0
}}
"#,
        port = port
    );
    let mut backend = Win32PlatformIo::default();
    let program = ace_av::console_host::compile(&src, "d_main").expect("compile redis source");
    let mut limits = ace_av_core::av::ExecLimits::default();
    limits.wall = std::time::Duration::from_secs(10);
    let (out, _t, _s) = program
        .run_platform(
            &redis_grants(),
            &limits,
            &ace_av_core::av::CancelFlag::new(false),
            None,
            &mut backend,
        )
        .expect("run platform redis");
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            }
        ),
        "RESP SET/GET/INCR/DEL replies byte-exact, got {out:?}"
    );
    let received = server.join().expect("server");
    assert_eq!(
        received,
        vec!["SET k hello", "GET k", "INCR counter", "DEL k"],
        "the server-side record agrees with the client's commands"
    );
}

// ---- the stdlib packaging face: `stdlib/ace/std/redis.at` verbatim,
// public helpers only, against a STRICT RESP2-array-parsing server. ---

/// Drain one CRLF-terminated line from `buf`, reading more from `c` as
/// needed. None on EOF/error.
fn resp_read_line(buf: &mut Vec<u8>, c: &mut std::net::TcpStream) -> Option<String> {
    loop {
        if let Some(pos) = buf.windows(2).position(|w| w == b"\r\n") {
            let line: Vec<u8> = buf.drain(..pos + 2).collect();
            return Some(String::from_utf8_lossy(&line[..pos]).to_string());
        }
        let mut tmp = [0u8; 128];
        match c.read(&mut tmp) {
            Ok(0) => return None,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
            Err(_) => return None,
        }
    }
}

/// Drain exactly `n` payload bytes + the trailing CRLF from `buf`
/// (reading as needed). None on EOF/error/trailing-CRLF mismatch.
fn resp_read_bulk(buf: &mut Vec<u8>, c: &mut std::net::TcpStream, n: usize) -> Option<Vec<u8>> {
    while buf.len() < n {
        let mut tmp = [0u8; 128];
        match c.read(&mut tmp) {
            Ok(0) => return None,
            Ok(k) => buf.extend_from_slice(&tmp[..k]),
            Err(_) => return None,
        }
    }
    let out: Vec<u8> = buf.drain(..n).collect();
    while buf.len() < 2 {
        let mut tmp = [0u8; 8];
        match c.read(&mut tmp) {
            Ok(0) => return None,
            Ok(k) => buf.extend_from_slice(&tmp[..k]),
            Err(_) => return None,
        }
    }
    if &buf[..2] != b"\r\n" {
        return None;
    }
    buf.drain(..2);
    Some(out)
}

/// Parse ONE full RESP2 array command ("*N" + N "$len" args). The
/// $len framing IS the wire oracle: each arg is decoded by length, so
/// a miscounted encoder yields the wrong recorded command (or None).
fn resp_read_command(buf: &mut Vec<u8>, c: &mut std::net::TcpStream) -> Option<Vec<String>> {
    let head = resp_read_line(buf, c)?;
    let n: usize = head.strip_prefix('*')?.parse().ok()?;
    let mut args = Vec::with_capacity(n);
    for _ in 0..n {
        let len_line = resp_read_line(buf, c)?;
        let len: usize = len_line.strip_prefix('$')?.parse().ok()?;
        let bytes = resp_read_bulk(buf, c, len)?;
        args.push(String::from_utf8_lossy(&bytes).to_string());
    }
    Some(args)
}

#[test]
fn redis975_std_redis_packaging_inline() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let server = std::thread::spawn(move || {
        let mut received: Vec<Vec<String>> = Vec::new();
        let Ok((mut c, _)) = listener.accept() else {
            return received;
        };
        let _ = c.set_read_timeout(Some(std::time::Duration::from_secs(2)));
        let mut buf = Vec::<u8>::new();
        loop {
            let Some(args) = resp_read_command(&mut buf, &mut c) else {
                break;
            };
            received.push(args.clone());
            let cmd = args[0].to_uppercase();
            let reply: Vec<u8> = match (cmd.as_str(), args.len()) {
                ("SET", 3) => b"+OK\r\n".to_vec(),
                ("GET", 2) if args[1] == "k" => b"$5\r\nhello\r\n".to_vec(),
                ("GET", 2) if args[1] == "missing" => b"$-1\r\n".to_vec(),
                ("GET", 2) if args[1] == "u" => b"$4\r\n\xe4\xb8\xadA\r\n".to_vec(),
                ("GET", 2) if args[1] == "badutf8" => b"$2\r\n\xff\xfe\r\n".to_vec(),
                ("GET", 2) if args[1] == "overlong" => b"$2\r\n\xc0\xaf\r\n".to_vec(),
                ("GET", 2) if args[1] == "surr" => b"$3\r\n\xed\xa0\x80\r\n".to_vec(),
                ("INCR", 2) => b":3\r\n".to_vec(),
                ("EXISTS", 2) => b":1\r\n".to_vec(),
                ("PING", 1) => b"+PONG\r\n".to_vec(),
                ("DEL", 2) => b":1\r\n".to_vec(),
                // the protocol negative: an unknown command is an error
                // reply the client must CLASSIFY, never silent success
                _ => b"-ERR unknown command\r\n".to_vec(),
            };
            if c.write_all(&reply).is_err() {
                break;
            }
            let _ = c.flush();
        }
        let _ = c.shutdown(std::net::Shutdown::Both);
        received
    });

    // The tracked stdlib module, verbatim, then a main that calls ONLY
    // the public helpers — the packaging seam (PLAN-1041 inline form).
    let stdlib_redis = include_str!("../../../../../../stdlib/ace/std/redis.at");
    let src = format!(
        r#"{stdlib_redis}
fn main() int {{
    let s = redis_connect("127.0.0.1", {port})
    if s <= 0 {{ return s }}
    // legacy-shaped ops: 1/0 bool sentinels, raw integer replies
    let a = redis_set(s, "k", "hello")          // +OK      -> 1
    let b = redis_get_len(s, "k")               // $5       -> 5
    let nilv = redis_get_len(s, "missing")      // $-1      -> -1
    let c = redis_incr(s, "counter")            // :3       -> 3
    let e = redis_exists(s, "k")                // :1       -> 1
    let p = redis_ping(s)                       // +PONG    -> 1
    // PLAN-975 T-03: get's Str return (B1 materialization) — ASCII,
    // nil -> "", and a multi-byte UTF-8 payload pass through intact
    let g Str = redis_get(s, "k")               // $5 hello -> "hello"
    let want_g Str = "hello"
    if g != want_g {{ return 40 }}
    let gm Str = redis_get(s, "missing")        // $-1      -> ""
    if gm.len_bytes() != 0 {{ return 41 }}
    let gu Str = redis_get(s, "u")              // $4 中A   -> "中A"
    let want_u Str = "中A"
    if gu != want_u {{ return 42 }}
    // malformed UTF-8 negative controls must reject cleanly to "" without trapping
    let gb Str = redis_get(s, "badutf8")
    if gb.len_bytes() != 0 {{ return 43 }}
    let g_ol Str = redis_get(s, "overlong")
    if g_ol.len_bytes() != 0 {{ return 44 }}
    let gs Str = redis_get(s, "surr")
    if gs.len_bytes() != 0 {{ return 45 }}
    // payload BYTES observable through the reader face
    let region = mem_alloc(4096)
    let end = redis_cmd(s, region, 0, redis_cmd_get("k"))
    let bl = redis_bulk_header_num(region, 0)   // 5
    let pa = redis_bulk_payload_at(region, 0, end) // 4
    let b0 = redis_bulk_byte(region, 0, end, 0)    // 'h' 104
    let b4 = redis_bulk_byte(region, 0, end, 4)    // 'o' 111
    let _f = mem_free(region)
    // the protocol negative: unknown command -> classified error reply
    let r2 = mem_alloc(4096)
    let _e2 = redis_cmd(s, r2, 0, "*1\r\n$3\r\nFOO\r\n")
    let iserr = redis_reply_is_error(r2, 0)     // 1
    let _f2 = mem_free(r2)
    let d = redis_del(s, "k")                   // :1       -> 1
    let _c = redis_close(s)
    if a != 1 {{ return 1 }}
    if b != 5 {{ return 2 }}
    if nilv != -1 {{ return 3 }}
    if c != 3 {{ return 4 }}
    if e != 1 {{ return 5 }}
    if p != 1 {{ return 6 }}
    if end != 11 {{ return 7 }}
    if bl != 5 {{ return 8 }}
    if pa != 4 {{ return 9 }}
    if b0 != 104 {{ return 10 }}
    if b4 != 111 {{ return 11 }}
    if iserr != 1 {{ return 12 }}
    if d != 1 {{ return 13 }}
    return 0
}}
"#,
        port = port
    );
    let mut backend = Win32PlatformIo::default();
    let program =
        ace_av::console_host::compile(&src, "d_main").expect("compile redis stdlib source");
    let mut limits = ace_av_core::av::ExecLimits::default();
    limits.wall = std::time::Duration::from_secs(10);
    let (out, _t, _s) = program
        .run_platform(
            &redis_grants(),
            &limits,
            &ace_av_core::av::CancelFlag::new(false),
            None,
            &mut backend,
        )
        .expect("run platform redis stdlib");
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            }
        ),
        "std.redis packaged face oracles, got {out:?}"
    );
    let received = server.join().expect("server");
    // the strict RESP2-array parse decoded EXACTLY these commands —
    // the wire-level oracle for the Auto-side encoder
    assert_eq!(
        received,
        vec![
            vec!["SET", "k", "hello"],
            vec!["GET", "k"],
            vec!["GET", "missing"],
            vec!["INCR", "counter"],
            vec!["EXISTS", "k"],
            vec!["PING"],
            vec!["GET", "k"],
            vec!["GET", "missing"],
            vec!["GET", "u"],
            vec!["GET", "badutf8"],
            vec!["GET", "overlong"],
            vec!["GET", "surr"],
            vec!["GET", "k"],
            vec!["FOO"],
            vec!["DEL", "k"],
        ],
        "the server-side strict-parse record must match"
    );
}
