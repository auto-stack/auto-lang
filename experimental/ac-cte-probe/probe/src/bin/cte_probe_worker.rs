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
        Some("case-serve") => case_serve(),
        _ => {
            eprintln!("usage: cte_probe_worker case-serve < request.json");
            std::process::exit(2);
        }
    }
}

fn case_serve() -> ! {
    let mut buf = Vec::new();
    if std::io::stdin().read_to_end(&mut buf).is_err() {
        eprintln!("cte worker: stdin read failed");
        std::process::exit(3);
    }
    let req: auto_cte_probe::worker::WorkerRequest = match serde_json::from_slice(&buf) {
        Ok(r) => r,
        Err(e) => {
            let resp = auto_cte_probe::worker::WorkerResponse {
                schema: auto_cte_probe::worker::CTE_WORKER_PROTOCOL.into(),
                nonce: String::new(),
                request_hash: String::new(),
                ok: None,
                error: Some(auto_cte_probe::worker::WorkerError {
                    code: "cte.worker.protocol".into(),
                    message: format!("request does not decode: {e}"),
                }),
            };
            emit(&resp);
            std::process::exit(0);
        }
    };
    let native_root = std::env::temp_dir().join(format!("cte817-worker-{}", std::process::id()));
    let resp = auto_cte_probe::worker::serve_request(&req, &native_root);
    emit(&resp);
    std::process::exit(0);
}

fn emit(resp: &auto_cte_probe::worker::WorkerResponse) {
    let mut line = serde_json::to_string(resp).unwrap_or_default();
    line.push('\n');
    let _ = std::io::stdout().write_all(line.as_bytes());
    let _ = std::io::stdout().flush();
}

fn bench_turnaround(args: &[String]) -> ! {
    let mut samples = auto_cte_probe::bench::DEFAULT_SAMPLES;
    let mut gate_ms = auto_cte_probe::bench::DEFAULT_GATE_MS;
    let mut inject = 0u64;
    let mut out: Option<std::path::PathBuf> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--samples" => {
                i += 1;
                samples = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(samples);
            }
            "--gate-ms" => {
                i += 1;
                gate_ms = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(gate_ms);
            }
            "--inject-delay-ms" => {
                i += 1;
                inject = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(inject);
            }
            "--out" => {
                i += 1;
                out = args.get(i).map(std::path::PathBuf::from);
            }
            other => {
                eprintln!("unknown bench flag {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    let root = std::env::temp_dir().join(format!("cte817-bench-{}", std::process::id()));
    let (code, summary) =
        auto_cte_probe::bench::run_bench(samples, gate_ms, inject, out.as_deref(), &root);
    println!("{summary}");
    std::process::exit(code);
}
