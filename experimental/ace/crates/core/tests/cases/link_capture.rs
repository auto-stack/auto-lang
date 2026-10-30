//! PLAN-788 T-03 raw-bytes capture entry (`run_deadlined_captured`) and its
//! budget boundaries (V-05/V-06 anchors at the executor level):
//!
//! - `process_budget_*` — deadline kill and immediate output-cap
//!   termination, both with full-tree reaping, never a truncated success;
//! - `native_bytes_*` — exact byte round-trip of invalid-UTF-8/NUL/CRLF
//!   streams where the legacy lossy wrapper provably differs;
//! - containment-failure replay through the captured path (same controlled
//!   refusal, no degraded run).
//!
//! Windows-only (the contained executor is Windows Job-Object based).

#![cfg(windows)]

use ace_core::link::{
    run_deadlined_captured, run_deadlined_captured_with, run_deadlined_split, CaptureCaps,
    SubprocError,
};
use std::io::Write;
use std::process::Command;
use std::time::{Duration, Instant};

fn scratch(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("ac788-capture-{}", name));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// Deterministic byte fountain: PowerShell writes the file's exact bytes to
/// the RAW standard output stream (`cmd /c type` re-encodes non-ASCII through
/// the console codepage and is NOT byte-faithful).
fn type_cmd(path: &std::path::Path) -> Command {
    let script = format!(
        "$c=[IO.File]::ReadAllBytes('{}');[Console]::OpenStandardOutput().Write($c,0,$c.Length)",
        path.display()
    );
    let mut c = Command::new("powershell");
    c.arg("-NoProfile").arg("-Command").arg(script);
    c
}

#[test]
fn process_budget_deadline_kills_the_tree() {
    let start = Instant::now();
    let mut cmd = Command::new("cmd");
    cmd.args(["/c", "ping", "-n", "30", "127.0.0.1"]);
    let err = run_deadlined_captured(&mut cmd, Duration::from_secs(1), CaptureCaps::none())
        .expect_err("ping -n 30 must hit the 1s deadline");
    assert!(matches!(err, SubprocError::Deadline), "{:?}", err);
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "killed promptly, not after the full ping duration ({:?})",
        start.elapsed()
    );
}

#[test]
fn process_budget_output_cap_terminates_immediately() {
    let dir = scratch("cap");
    let big = dir.join("big.bin");
    let mut buf = Vec::with_capacity(2 << 20);
    buf.resize(2 << 20, b'x');
    std::fs::write(&big, &buf).unwrap();
    let start = Instant::now();
    let err = run_deadlined_captured(
        &mut type_cmd(&big),
        Duration::from_secs(30),
        CaptureCaps {
            stdout_max: 4096,
            stderr_max: usize::MAX,
        },
    )
    .expect_err("2MiB against a 4096-byte cap must be a controlled failure");
    match err {
        SubprocError::OutputCapExceeded {
            stream,
            observed,
            cap,
        } => {
            assert_eq!(stream, "stdout");
            assert_eq!(cap, 4096);
            assert!(observed > cap, "observed {} passed the cap", observed);
        }
        other => panic!("expected OutputCapExceeded, got {:?}", other),
    }
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "terminated on the cap, not on the 30s deadline ({:?})",
        start.elapsed()
    );
}

#[test]
fn process_budget_output_cap_boundary_exact_size_passes() {
    let dir = scratch("cap-exact");
    let exact = dir.join("exact.bin");
    std::fs::write(&exact, vec![b'a'; 1024]).unwrap();
    let run = run_deadlined_captured(
        &mut type_cmd(&exact),
        Duration::from_secs(30),
        CaptureCaps {
            stdout_max: 1024,
            stderr_max: usize::MAX,
        },
    )
    .expect("exactly-at-cap output is a success");
    assert_eq!(run.stdout, vec![b'a'; 1024]);
    assert_eq!(run.exit_code, Some(0));
}

#[test]
fn native_bytes_invalid_utf8_round_trips_exactly() {
    let dir = scratch("bytes");
    // 0xFF 0xFE are never valid UTF-8 starters; GBK tool output shape.
    let payload: Vec<u8> = vec![0xFF, 0xFE, b'a', 0xC3, 0x28, b'b'];
    let f = dir.join("gbk.bin");
    std::fs::write(&f, &payload).unwrap();
    let run = run_deadlined_captured(
        &mut type_cmd(&f),
        Duration::from_secs(30),
        CaptureCaps::none(),
    )
    .unwrap();
    assert_eq!(run.stdout, payload, "exact bytes, no lossy replacement");
    // The legacy wrapper is lossy ON THE SAME BYTES — prove the captured
    // entry is the byte oracle, the wrapper only a display projection.
    let (_, legacy_out, _) =
        run_deadlined_split(&mut type_cmd(&f), Duration::from_secs(30)).unwrap();
    assert_ne!(legacy_out.as_bytes(), payload.as_slice());
    assert_eq!(
        legacy_out,
        String::from_utf8_lossy(&payload),
        "wrapper stays the documented lossy projection"
    );
}

#[test]
fn native_bytes_nul_and_crlf_preserved() {
    let dir = scratch("nul-crlf");
    let payload: &[u8] = b"a\r\nb\x00c\r\n\x00";
    let f = dir.join("nul.bin");
    let mut w = std::fs::File::create(&f).unwrap();
    w.write_all(payload).unwrap();
    drop(w);
    let run = run_deadlined_captured(
        &mut type_cmd(&f),
        Duration::from_secs(30),
        CaptureCaps::none(),
    )
    .unwrap();
    assert_eq!(run.stdout, payload);
    assert_eq!(
        run.stderr,
        Vec::<u8>::new(),
        "undeclared stream is empty bytes"
    );
}

#[test]
fn native_bytes_empty_stream_is_exact_empty() {
    let mut cmd = Command::new("cmd");
    cmd.args(["/c", "exit", "7"]);
    let run =
        run_deadlined_captured(&mut cmd, Duration::from_secs(30), CaptureCaps::none()).unwrap();
    assert_eq!(run.exit_code, Some(7));
    assert!(run.stdout.is_empty());
    assert!(run.stderr.is_empty());
}

#[test]
fn containment_failure_replays_through_captured_path() {
    // Same controlled refusal as the legacy executor (R3-QA-01): a job the
    // executor cannot bound is never run — no degraded bare spawn.
    let mut cmd = Command::new("cmd");
    cmd.args(["/c", "exit", "0"]);
    let err = run_deadlined_captured_with(
        &mut cmd,
        Duration::from_secs(10),
        CaptureCaps::none(),
        |_| None,
    )
    .expect_err("factory refusal must surface");
    match err {
        SubprocError::ContainmentUnavailable(reason) => {
            assert!(
                reason.contains("before its first instruction"),
                "{}",
                reason
            )
        }
        other => panic!("expected ContainmentUnavailable, got {:?}", other),
    }
}
