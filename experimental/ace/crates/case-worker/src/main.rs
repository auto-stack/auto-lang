//! ac-case-worker entry: a bounded JSON-lines session on stdin/stdout.
//!
//! begin → request×N → end. One response line per request. The session is
//! bounded by REQUEST_COUNT/LIFETIME caps; a parent that disappears (EOF)
//! ends the session cleanly. stderr carries human diagnostics only.

use ace_case_worker::exec;
use ace_case_worker::protocol::{
    Envelope, WorkerResponse, SESSION_MAX_LIFETIME_MS, SESSION_MAX_LINE_BYTES,
    SESSION_MAX_REQUESTS, WORKER_SCHEMA,
};
use std::io::Write;
use std::time::{Duration, Instant};

fn main() {
    // R2 (F05): self-containment — a parent kill reaps this process's
    // whole tree via the leaked kill-on-close job handle.
    if let Err(e) = ace_core::link::enter_self_containment() {
        eprintln!("ac-case-worker: self-containment unavailable: {}", e);
        std::process::exit(2);
    }
    let started = Instant::now();
    // R1 (F03): the session lifetime is a HARD wall clock enforced even
    // while idle — a watchdog exits the process when the budget is spent,
    // so neither an idle session nor a wedged request outlives it. Normal
    // `end`/EOF exits race harmlessly against it.
    let lifetime = Duration::from_millis(SESSION_MAX_LIFETIME_MS);
    std::thread::spawn(move || {
        std::thread::sleep(lifetime);
        eprintln!("ac-case-worker: session lifetime cap exceeded");
        std::process::exit(2);
    });
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut requests: u32 = 0;
    let mut began = false;

    // R2 (F05): lines are read with a PRE-ALLOCATION bound — fill_buf
    // scans for the newline without ever buffering an unbounded line.
    // PLAN-1034 IPC-02: a mid-stream IO error is a NAMED infra termination
    // of the current incomplete request (never a clean EOF, never an
    // accepted prefix); previously completed responses are not rewritten.
    let mut stdin_lock = stdin.lock();
    loop {
        let line = match read_line_bounded(&mut stdin_lock, SESSION_MAX_LINE_BYTES) {
            LineOutcome::Line(l) => l,
            LineOutcome::Eof => break, // EOF ends the session cleanly
            LineOutcome::TooLong(observed) => {
                let _ = writeln!(
                    out,
                    "{}",
                    serde_json::to_string(&WorkerResponse::err(
                        ("", ""),
                        ace_case_worker::WorkerError {
                            code: "session.line-cap".into(),
                            message: format!(
                                "request line exceeds the {} byte bound ({}+ observed)",
                                SESSION_MAX_LINE_BYTES, observed
                            ),
                        }
                    ))
                    .unwrap()
                );
                let _ = out.flush();
                std::process::exit(2);
            }
            LineOutcome::Io(e) => {
                let _ = writeln!(
                    out,
                    "{}",
                    serde_json::to_string(&WorkerResponse::err(
                        ("", ""),
                        ace_case_worker::WorkerError {
                            code: "session.line-io".into(),
                            message: format!(
                                "stdin read ended with an I/O error; the incomplete \
                                 request is discarded (not EOF, not accepted): {e}"
                            ),
                        }
                    ))
                    .unwrap()
                );
                let _ = out.flush();
                std::process::exit(2);
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let envelope: Envelope = match serde_json::from_str(trimmed) {
            Ok(e) => e,
            Err(e) => {
                let _ = writeln!(
                    out,
                    "{}",
                    serde_json::to_string(&WorkerResponse::err(
                        ("", ""),
                        ace_case_worker::WorkerError {
                            code: "request.envelope".into(),
                            message: format!("undecodable request line: {}", e),
                        }
                    ))
                    .unwrap()
                );
                let _ = out.flush();
                continue;
            }
        };
        match envelope {
            Envelope::Begin { schema } => {
                if schema != WORKER_SCHEMA {
                    eprintln!(
                        "ac-case-worker: begin schema {:?} != {}",
                        schema, WORKER_SCHEMA
                    );
                    std::process::exit(2);
                }
                began = true;
            }
            Envelope::End => break,
            Envelope::Request { request } => {
                if !began {
                    let _ = writeln!(
                        out,
                        "{}",
                        serde_json::to_string(&WorkerResponse::err(
                            (&request.nonce, &request.request_hash),
                            ace_case_worker::WorkerError {
                                code: "request.session".into(),
                                message: "request before begin".into(),
                            }
                        ))
                        .unwrap()
                    );
                    let _ = out.flush();
                    continue;
                }
                requests += 1;
                if requests > SESSION_MAX_REQUESTS {
                    let _ = writeln!(
                        out,
                        "{}",
                        serde_json::to_string(&WorkerResponse::err(
                            (&request.nonce, &request.request_hash),
                            ace_case_worker::WorkerError {
                                code: "session.limit".into(),
                                message: format!(
                                    "session request cap {} exceeded",
                                    SESSION_MAX_REQUESTS
                                ),
                            }
                        ))
                        .unwrap()
                    );
                    let _ = out.flush();
                    std::process::exit(2);
                }
                if started.elapsed().as_millis() as u64 > SESSION_MAX_LIFETIME_MS {
                    let _ = writeln!(
                        out,
                        "{}",
                        serde_json::to_string(&WorkerResponse::err(
                            (&request.nonce, &request.request_hash),
                            ace_case_worker::WorkerError {
                                code: "session.limit".into(),
                                message: format!(
                                    "session lifetime cap {}ms exceeded",
                                    SESSION_MAX_LIFETIME_MS
                                ),
                            }
                        ))
                        .unwrap()
                    );
                    let _ = out.flush();
                    std::process::exit(2);
                }
                let response = match exec::execute(&request) {
                    Ok(ok) => WorkerResponse::ok(&request, ok),
                    Err(e) => WorkerResponse::err((&request.nonce, &request.request_hash), e),
                };
                if let Err(e) = writeln!(out, "{}", serde_json::to_string(&response).unwrap()) {
                    eprintln!("ac-case-worker: stdout closed: {}", e);
                    std::process::exit(2);
                }
                let _ = out.flush();
            }
        }
    }
}

/// R2 (F05) + PLAN-1034 IPC-02: read ONE line with a hard pre-allocation
/// cap. The cap counts RAW frame bytes (lossy UTF-8 conversion happens
/// AFTER admission; a converted String is never re-measured against the
/// raw cap) and EXCLUDES the delimiter LF. A final non-empty frame without
/// LF stays parseable (EOF right after its bytes); an I/O error is a named
/// infra end — the partial frame is discarded, never returned as a request
/// or mistaken for EOF.
#[derive(Debug)]
enum LineOutcome {
    Line(String),
    Eof,
    TooLong(usize),
    Io(std::io::Error),
}

fn read_line_bounded<R: std::io::BufRead>(r: &mut R, cap: usize) -> LineOutcome {
    let mut line = Vec::new();
    loop {
        let available = match r.fill_buf() {
            Ok(b) => b,
            Err(e) => return LineOutcome::Io(e),
        };
        if available.is_empty() {
            return if line.is_empty() {
                LineOutcome::Eof
            } else {
                LineOutcome::Line(String::from_utf8_lossy(&line).to_string())
            };
        }
        match available.iter().position(|&b| b == b'\n') {
            Some(nl) => {
                if line.len() + nl > cap {
                    return LineOutcome::TooLong(line.len() + nl);
                }
                line.extend_from_slice(&available[..nl]);
                r.consume(nl + 1);
                return LineOutcome::Line(String::from_utf8_lossy(&line).to_string());
            }
            None => {
                if line.len() + available.len() > cap {
                    return LineOutcome::TooLong(line.len() + available.len());
                }
                let take = available.len();
                line.extend_from_slice(available);
                r.consume(take);
            }
        }
    }
}

// ---- PLAN-1034 T-03（IPC-02）：line reader 的 EOF/IO/TooLong/编码分界 ----
#[cfg(test)]
mod audit1034_ipc_02 {
    use super::*;
    use std::io::ErrorKind;

    /// 帧计数不含分隔 LF；恰 cap 帧接受、cap+1 拒绝（机制等界用小 cap）。
    #[test]
    fn line_cap_counts_raw_frame_bytes_excluding_lf() {
        let cap = 8usize;
        // "abcd" + LF：帧 4 字节 ≤ cap。
        let mut r: &[u8] = b"abcd\n";
        match read_line_bounded(&mut r, cap) {
            LineOutcome::Line(l) => assert_eq!(l, "abcd"),
            other => panic!("expected Line, got {other:?}"),
        }
        // 帧 8 字节（不含 LF）恰等于 cap：接受。
        let mut r: &[u8] = b"12345678\n";
        match read_line_bounded(&mut r, cap) {
            LineOutcome::Line(l) => assert_eq!(l, "12345678"),
            other => panic!("cap-equal frame must pass, got {other:?}"),
        }
        // 帧 9 字节（不含 LF）= cap+1：TooLong。
        let mut r: &[u8] = b"123456789\n";
        match read_line_bounded(&mut r, cap) {
            LineOutcome::TooLong(n) => assert_eq!(n, 9),
            other => panic!("cap+1 frame must be TooLong, got {other:?}"),
        }
    }

    /// 纯 EOF 不产生请求；无 LF 的最终非空帧仍可解析（本通道宽松合同）。
    #[test]
    fn pure_eof_no_request_final_frame_without_lf_parseable() {
        let mut r: &[u8] = b"";
        match read_line_bounded(&mut r, 8) {
            LineOutcome::Eof => {}
            other => panic!("empty input is Eof, got {other:?}"),
        }
        let mut r: &[u8] = b"{\"op\":\"end\"}";
        match read_line_bounded(&mut r, 64) {
            LineOutcome::Line(l) => assert_eq!(l, "{\"op\":\"end\"}"),
            other => panic!("final frame without LF stays parseable, got {other:?}"),
        }
    }

    /// 非法 UTF-8（含 0xff）在 raw 计量后 lossy 转换——不因转换后长度
    /// 重新套 raw cap 而误拒（lossy 结果最多 3× 原字节）。
    #[test]
    fn invalid_utf8_goes_lossy_after_raw_admission() {
        let frame: &[u8] = &[b'{', 0xff, b'}'];
        let mut r = frame;
        match read_line_bounded(&mut r, 8) {
            LineOutcome::Line(l) => assert_eq!(l, "{\u{fffd}}"),
            other => panic!("lossy conversion expected, got {other:?}"),
        }
    }

    /// IO 错误（半行后失败）：具名 Io——前缀不被当请求、不折 EOF。
    struct PrefixThenIo;
    impl std::io::Read for PrefixThenIo {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(ErrorKind::Other, "kernel read gone"))
        }
    }
    impl std::io::BufRead for PrefixThenIo {
        fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
            Err(std::io::Error::new(ErrorKind::Other, "kernel read gone"))
        }
        fn consume(&mut self, _: usize) {}
    }
    // 直接注入：先给半行数据再持续 IO 错。
    struct HalfThenIo {
        first: bool,
    }
    impl std::io::Read for HalfThenIo {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(ErrorKind::Other, "kernel read gone"))
        }
    }
    impl std::io::BufRead for HalfThenIo {
        fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
            if self.first {
                self.first = false;
                Ok(&b"{\"op\":"[..])
            } else {
                Err(std::io::Error::new(ErrorKind::Other, "kernel read gone"))
            }
        }
        fn consume(&mut self, _: usize) {}
    }
    #[test]
    fn io_error_mid_frame_is_named_not_eof_not_prefix() {
        let mut r = HalfThenIo { first: true };
        match read_line_bounded(&mut r, 64) {
            LineOutcome::Io(e) => assert!(e.to_string().contains("kernel read gone")),
            other => panic!("IO mid-frame must be named Io, got {other:?}"),
        }
        let mut r = PrefixThenIo;
        match read_line_bounded(&mut r, 64) {
            LineOutcome::Io(_) => {}
            other => panic!("immediate IO must be Io, got {other:?}"),
        }
    }
}
