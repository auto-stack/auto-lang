//! PLAN-1041 OUT-01/02 Linux legs: the same common Auto output source
//! (`stdlib/auto/platform/io.at`, inlined) really executed INSIDE the
//! WSL2 Linux environment — (a) by the AV interpreter through the static
//! musl runner (raw-syscall backend, console host writing the real fds),
//! (b) as a linked static ELF via the AC console lane, and (c) the frozen
//! SIGPIPE policy witness: the standalone AC CLI keeps the kernel default
//! disposition, so a broken pipe on a >64KiB output kills the process
//! (signal 13), never a hang and never a silent fallback.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_native::console::{console_authorize, console_compile, console_native_build_elf};
use ace_native::linux_exec::{run_elf_console_lane, run_wsl_argv, wsl_distro, wsl_resolve_path};
use ace_native::platform_io_linux::audit_elf_dependencies;
use std::path::{Path, PathBuf};

const IO_AT: &str = include_str!("../../../../../../stdlib/auto/platform/io.at");

fn grants() -> Vec<String> {
    vec![
        "console.stdout".into(),
        "console.stderr".into(),
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
    ]
}

fn temp_workdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("p1041lnx-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// OUT-01/02 AV/Linux: byte-exact UTF-8 (incl. NUL and an astral char) on
/// BOTH real fds through the raw-syscall backend; kind is 2 (byte sink —
/// the console path never engages on Linux); resources clean at exit.
#[test]
fn out01_av_linux_runner_bytes_nul_both_streams() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let runner = workspace.join("target/x86_64-unknown-linux-musl/debug/mn-linux-av-runner");
    {
        let out = std::process::Command::new(env!("CARGO"))
            .current_dir(&workspace)
            .env("RUSTFLAGS", "-C linker=rust-lld")
            .args([
                "build",
                "-p",
                "ace-av-core",
                "--bin",
                "mn-linux-av-runner",
                "--target",
                "x86_64-unknown-linux-musl",
            ])
            .output()
            .expect("cargo build runner");
        assert!(
            out.status.success(),
            "musl runner build failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let main = r#"
fn main() int {
    let h = out_stdout()
    let k = out_kind(h)
    if k != 2 { return -1 }
    let a = out_text(h, "L你好\0😀\n")
    let e = out_stderr()
    let b = out_text(e, "err中!")
    return a + b
}
"#;
    let src = format!("{IO_AT}\n{main}");
    let program = ace_av::console_host::compile(&src, "d_main").expect("compile");
    let dir = temp_workdir("av");
    let wire_path = dir.join("wire.abc");
    let summary_path = dir.join("summary.json");
    std::fs::write(&wire_path, program.wire()).unwrap();
    std::fs::write(&summary_path, b"").unwrap();
    let wire_linux = wsl_resolve_path(&wire_path, &wsl_distro()).expect("wire path");
    let summary_linux = wsl_resolve_path(&summary_path, &wsl_distro()).expect("summary path");
    let argv = vec![
        wire_linux.linux_path.clone(),
        program.reference.owner.clone(),
        "d_main".to_string(),
        grants().join(","),
        summary_linux.linux_path.clone(),
        "20000".to_string(),
    ];
    let run = run_wsl_argv(
        &runner.to_string_lossy(),
        &argv,
        std::time::Duration::from_secs(60),
        ace_core::link::CaptureCaps {
            stdout_max: 4 << 20,
            stderr_max: 1 << 20,
        },
    )
    .expect("runner exec");
    let summary_text = std::fs::read_to_string(&summary_path).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        run.exit_code,
        Some(0),
        "runner infra: {summary_text} {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let summary: serde_json::Value = serde_json::from_str(&summary_text)
        .unwrap_or_else(|e| panic!("summary not JSON ({e}): {summary_text}"));
    assert_eq!(summary["outcome"], "success", "{summary:?}");
    assert_eq!(summary["entry_value"], 20, "{summary:?}");
    assert_eq!(summary["platform_live_mapped_at_exit"], 0, "{summary:?}");
    assert_eq!(summary["platform_leaked_cleaned"], 0, "{summary:?}");
    assert_eq!(run.stdout, "L你好\0😀\n".as_bytes());
    assert_eq!(run.stderr, "err中!".as_bytes());
}

/// OUT-01/02 AC/Linux: the SAME source linked as a static ELF (no interp,
/// no dynamic, zero undefined) and really run in WSL — byte-exact both
/// streams; kind is 2; the ELF dependency closure stays honest.
#[test]
fn out01_ac_linux_elf_bytes_and_audit() {
    let main = r#"
fn main() int {
    let h = out_stdout()
    let k = out_kind(h)
    if k != 2 { return -1 }
    let a = out_text(h, "E你好\0Z\n")
    let b = out_text(out_stderr(), "E!")
    return a + b
}
"#;
    let src = format!("{IO_AT}\n{main}");
    let mir = console_compile(&src, "d_main").expect("compile");
    console_authorize(&mir, &grants()).expect("authorize");
    let dir = temp_workdir("ac");
    let linked = console_native_build_elf(&mir, "d_main", &dir).expect("elf build");
    let elf_bytes = std::fs::read(&linked.exe_path).expect("read elf");
    let report = audit_elf_dependencies(&elf_bytes).expect("elf audit");
    assert!(report.elf64_x86_64_exec);
    assert!(report.interpreter.is_none());
    assert!(!report.has_dynamic);
    assert!(report.undefined_symbols.is_empty());
    let run = run_elf_console_lane(&linked.exe_path).expect("wsl run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        run.exit_code,
        Some(12),
        "exit {:?} out {:?} err {:?}",
        run.exit_code,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, "E你好\0Z\n".as_bytes());
    assert_eq!(run.stderr, b"E!");
}

/// OUT-04 SIGPIPE (frozen policy, AC standalone CLI): kernel default
/// disposition — a broken pipe on a >64KiB output kills the process with
/// signal 13 (bash reports 141). Never a hang, never a silent fallback.
#[test]
fn out04_ac_linux_sigpipe_default_disposition() {
    // ~70KiB of 3-byte chars: crosses the pipe buffer, blocks, and the
    // closed reader turns the next write into SIGPIPE (default action).
    let mut lit = String::new();
    for _ in 0..24000 {
        lit.push('中');
    }
    let src = format!(
        "{IO_AT}\nfn main() int {{\n    let x = \"{lit}\"\n    return out_text(out_stdout(), x)\n}}\n"
    );
    let mir = console_compile(&src, "d_main").expect("compile");
    console_authorize(&mir, &grants()).expect("authorize");
    let dir = temp_workdir("sigpipe");
    let linked = console_native_build_elf(&mir, "d_main", &dir).expect("elf build");
    let exe_linux = wsl_resolve_path(&linked.exe_path, &wsl_distro()).expect("exe path");
    // `exe | (exit 0)` — the reader exits immediately; the blocked writer
    // must die by SIGPIPE (141 = 128 + 13).
    let sh = format!(
        "{} | (exit 0); echo ${{PIPESTATUS[0]}}",
        exe_linux.linux_path
    );
    let out = std::process::Command::new("wsl.exe")
        .arg("--exec")
        .arg("bash")
        .arg("-c")
        .arg(&sh)
        .output()
        .expect("wsl bash");
    let _ = std::fs::remove_dir_all(&dir);
    let status_line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(
        status_line,
        "141",
        "SIGPIPE default disposition (got {status_line}; stderr {})",
        String::from_utf8_lossy(&out.stderr)
    );
}
