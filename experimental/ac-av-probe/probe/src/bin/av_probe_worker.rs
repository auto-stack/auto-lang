//! PLAN-798 bounded worker: cross-process restore + re-verify + execute.
//!
//! `restore-run` opens the managed store, loads the module manifest and its
//! ABC units BY ARTIFACT ID (the store's address admission guards every
//! read), re-validates every envelope against the expected producer/schema/
//! owner/stage, re-decodes and re-verifies each unit, re-binds diagnostics
//! sidecars, assembles + verifies the closure and executes the entry. It
//! never receives or reads the source, never consults expected outputs, and
//! prints one JSON result line.
//!
//! `bench-turnaround` drives the six-scenario sample loop for the
//! turnaround tier (PLAN-798 §5.5) against a fixed 256 KiB / 100-function
//! load; raw samples print as JSONL for the report script.

use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    match args[0].as_str() {
        "restore-run" => restore_run(&args[1..]),
        "bench-turnaround" => bench_turnaround(&args[1..]),
        other => {
            eprintln!("unknown subcommand `{}`", other);
            usage();
        }
    }
}

fn usage() -> ! {
    eprintln!("usage: av-probe-worker restore-run --store DIR --owner NAME --entry FN --manifest-artifact ID [--snapshot HEX]");
    eprintln!("       av-probe-worker bench-turnaround --samples N --out FILE");
    exit(2)
}

fn flag(args: &[String], name: &str) -> Option<String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == name {
            return it.next().cloned();
        }
    }
    None
}

fn restore_run(args: &[String]) -> ! {
    let store_dir = flag(args, "--store").unwrap_or_else(|| usage());
    let owner = flag(args, "--owner").unwrap_or_else(|| usage());
    let entry = flag(args, "--entry").unwrap_or_else(|| usage());
    let manifest_artifact = flag(args, "--manifest-artifact").unwrap_or_else(|| usage());
    let snapshot = flag(args, "--snapshot")
        .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("snapshot hex"))
        .unwrap_or(0);

    let outcome = auto_av_probe::worker_restore::restore_and_run(
        &PathBuf::from(store_dir),
        &owner,
        &entry,
        &manifest_artifact,
        snapshot,
    );
    let mut line = serde_json::to_string(&outcome).unwrap_or_else(|e| {
        eprintln!("serialize: {}", e);
        exit(3)
    });
    line.push('\n');
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(if outcome.executed_ok { 0 } else { 4 })
}

fn bench_turnaround(args: &[String]) -> ! {
    let samples: usize = flag(args, "--samples")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let gate_ms: f64 = flag(args, "--gate-ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000.0);
    let out_path = flag(args, "--out").unwrap_or_else(|| "-".into());
    let load = auto_av_probe::load256::generate();
    let edited = auto_av_probe::load256::generate_with(true);
    let conflicted = auto_av_probe::load256::edit_signature_conflict(&load.source);
    let mut out: Box<dyn Write> = if out_path == "-" {
        Box::new(std::io::stdout())
    } else {
        Box::new(std::fs::File::create(&out_path).expect("out file"))
    };
    let root = std::env::temp_dir().join(format!("av798-bench-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let result = auto_av_probe::bench::run_six_scenarios(&root, &load, &edited, &conflicted, samples);
    for line in result {
        let mut l = serde_json::to_string(&line).unwrap_or_default();
        l.push('\n');
        out.write_all(l.as_bytes()).ok();
    }
    let _ = std::fs::remove_dir_all(&root);
    exit(0)
}
