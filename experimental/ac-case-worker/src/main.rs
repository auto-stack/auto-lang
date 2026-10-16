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
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

fn main() {
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

    for line in stdin.lock().lines() {
        // R1 (F03): a request line beyond the frozen bound is rejected,
        // never buffered without limit.
        if let Ok(l) = &line {
            if l.len() > SESSION_MAX_LINE_BYTES {
                let _ = writeln!(
                    out,
                    "{}",
                    serde_json::to_string(&WorkerResponse::err(
                        ("", ""),
                        ac_case_worker::WorkerError {
                            code: "session.line-cap".into(),
                            message: format!(
                                "request line {} bytes exceeds the {} byte bound",
                                l.len(),
                                SESSION_MAX_LINE_BYTES
                            ),
                        }
                    ))
                    .unwrap()
                );
                let _ = out.flush();
                std::process::exit(2);
            }
        }
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("ac-case-worker: stdin error: {}", e);
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
