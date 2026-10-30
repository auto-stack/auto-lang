//! The cte_probe_worker binary (`cteworker/1`).
//!
//! `case-serve` — one request JSON on stdin (to EOF), one JSON response
//! line on stdout, exit 0. Protocol failures exit non-zero (the parent
//! treats that as infra, never a language verdict).

use std::io::{Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("bench-turnaround") => bench_turnaround(&args[2..]),
        Some("case-serve-stream") => {
            let mut stdin = std::io::BufReader::new(std::io::stdin());
            let mut stdout = std::io::BufWriter::new(std::io::stdout());
            ace_comptime::worker::serve_stream(&mut stdin, &mut stdout);
            std::process::exit(0);
        }
        Some("case-serve") => case_serve(),
        _ => {
            eprintln!("usage: cte_probe_worker case-serve < request.json");
            std::process::exit(2);
        }
    }
}

fn case_serve() -> ! {
    let mut buf = Vec::new();
    // F-817-R6-02: BOUNDED wire read — a request larger than the wire
    // bound is refused after at most bound+1 bytes (never an unbounded
    // read-to-end allocation).
    let stdin = std::io::stdin();
    let mut limited = stdin
        .lock()
        .take((ace_comptime::worker::MAX_REQUEST_WIRE_BYTES + 1) as u64);
    if limited.read_to_end(&mut buf).is_err() {
        eprintln!("cte worker: stdin read failed");
        std::process::exit(3);
    }
    if buf.len() > ace_comptime::worker::MAX_REQUEST_WIRE_BYTES {
        let resp = ace_comptime::worker::WorkerResponse {
            schema: ace_comptime::worker::CTE_WORKER_PROTOCOL.into(),
            nonce: String::new(),
            request_hash: String::new(),
            ok: None,
            error: Some(ace_comptime::worker::WorkerError {
                code: "cte.worker.protocol".into(),
                message: format!(
                    "request {}B exceeds the {}B wire bound",
                    buf.len(),
                    ace_comptime::worker::MAX_REQUEST_WIRE_BYTES
                ),
            }),
        };
        emit(&resp);
        std::process::exit(0);
    }
    let req: ace_comptime::worker::WorkerRequest = match serde_json::from_slice(&buf) {
        Ok(r) => r,
        Err(e) => {
            let resp = ace_comptime::worker::WorkerResponse {
                schema: ace_comptime::worker::CTE_WORKER_PROTOCOL.into(),
                nonce: String::new(),
                request_hash: String::new(),
                ok: None,
                error: Some(ace_comptime::worker::WorkerError {
                    code: "cte.worker.protocol".into(),
                    message: format!("request does not decode: {e}"),
                }),
            };
            emit(&resp);
            std::process::exit(0);
        }
    };
    let native_root = std::env::temp_dir().join(format!("cte817-worker-{}", std::process::id()));
    let resp = ace_comptime::worker::serve_request(&req, &native_root);
    emit(&resp);
    std::process::exit(0);
}

fn emit(resp: &ace_comptime::worker::WorkerResponse) {
    let mut line = serde_json::to_string(resp).unwrap_or_default();
    line.push('\n');
    let _ = std::io::stdout().write_all(line.as_bytes());
    let _ = std::io::stdout().flush();
}

fn bench_turnaround(args: &[String]) -> ! {
    let mut samples = ace_comptime::bench::DEFAULT_SAMPLES;
    let mut gate_ms = ace_comptime::bench::DEFAULT_GATE_MS;
    let mut inject = 0u64;
    let mut inject_rebuild = false;
    let mut out: Option<std::path::PathBuf> = None;
    let mut only: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--samples" => {
                i += 1;
                // Parse SIGNED so negative counts reach run_bench's
                // validation instead of silently keeping the default
                // (F-817-R1-04: never a green run on malformed input).
                samples = args
                    .get(i)
                    .and_then(|v| v.parse::<i64>().ok())
                    .map(|v| v.max(-1) as usize)
                    .unwrap_or(samples);
            }
            "--gate-ms" => {
                i += 1;
                gate_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(gate_ms);
            }
            "--inject-delay-ms" => {
                i += 1;
                inject = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(inject);
            }
            "--inject-rebuild" => {
                inject_rebuild = true;
            }
            "--out" => {
                i += 1;
                out = args.get(i).map(std::path::PathBuf::from);
            }
            // F-817-R5: restrict the run to named scenarios (comma
            // separated). The formal gate runs ALL scenarios (no flag);
            // the memory/admission counter-controls use this to reach
            // ONE gate cheaply through the very same machinery.
            "--only" => {
                i += 1;
                if let Some(list) = args.get(i) {
                    only = list.split(',').map(|s| s.trim().to_string()).collect();
                }
            }
            other => {
                eprintln!("unknown bench flag {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    let only_refs: Vec<&str> = only.iter().map(|s| s.as_str()).collect();
    let root = std::env::temp_dir().join(format!("cte817-bench-{}", std::process::id()));
    let (code, summary) = ace_comptime::bench::run_bench_ext(
        samples,
        gate_ms,
        inject,
        inject_rebuild,
        &only_refs,
        out.as_deref(),
        &root,
    );
    println!("{summary}");
    std::process::exit(code);
}
