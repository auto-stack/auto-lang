//! ac-case-worker entry: a bounded JSON-lines session on stdin/stdout.
//!
//! begin → request×N → end. One response line per request. The session is
//! bounded by REQUEST_COUNT/LIFETIME caps; a parent that disappears (EOF)
//! ends the session cleanly. stderr carries human diagnostics only.

use ac_case_worker::exec;
use ac_case_worker::protocol::{
    Envelope, WorkerResponse, SESSION_MAX_LIFETIME_MS, SESSION_MAX_LINE_BYTES,
    SESSION_MAX_REQUESTS, WORKER_SCHEMA,
};
use std::io::Write;
use std::time::{Duration, Instant};

fn main() {
    // R2 (F05): self-containment — a parent kill reaps this process's
    // whole tree via the leaked kill-on-close job handle.
    if let Err(e) = auto_ac_prototype::link::enter_self_containment() {
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
    let mut stdin_lock = stdin.lock();
    loop {
        let line = match read_line_bounded(&mut stdin_lock, SESSION_MAX_LINE_BYTES) {
            Ok(Some(l)) => l,
            Ok(None) => break, // EOF ends the session cleanly
            Err(observed) => {
                let _ = writeln!(
                    out,
                    "{}",
                    serde_json::to_string(&WorkerResponse::err(
                        ("", ""),
                        ac_case_worker::WorkerError {
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
                        ac_case_worker::WorkerError {
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
                            ac_case_worker::WorkerError {
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
                            ac_case_worker::WorkerError {
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
                            ac_case_worker::WorkerError {
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

/// R2 (F05): read ONE line with a hard pre-allocation cap. Ok(None) = EOF;
/// Err(observed) = the line exceeded `cap` bytes before its newline.
fn read_line_bounded<R: std::io::BufRead>(r: &mut R, cap: usize) -> Result<Option<String>, usize> {
    let mut line = Vec::new();
    loop {
        let available = match r.fill_buf() {
            Ok(b) => b,
            Err(_) => {
                return if line.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(String::from_utf8_lossy(&line).to_string()))
                };
            }
        };
        if available.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Ok(Some(String::from_utf8_lossy(&line).to_string()))
            };
        }
        match available.iter().position(|&b| b == b'\n') {
            Some(nl) => {
                if line.len() + nl > cap {
                    return Err(line.len() + nl);
                }
                line.extend_from_slice(&available[..nl]);
                r.consume(nl + 1);
                return Ok(Some(String::from_utf8_lossy(&line).to_string()));
            }
            None => {
                if line.len() + available.len() > cap {
                    return Err(line.len() + available.len());
                }
                let take = available.len();
                line.extend_from_slice(available);
                r.consume(take);
            }
        }
    }
}
