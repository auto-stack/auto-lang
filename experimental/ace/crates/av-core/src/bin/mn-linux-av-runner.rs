//! PLAN-1040: the Linux AV runner (`mn-linux-av-runner`).
//!
//! A static musl binary that runs INSIDE the WSL2 Linux environment and
//! executes the SAME checked facts the AC/ELF lane consumes: the
//! console-lane ABC wire (produced on the Windows host by the shared
//! front door), re-verified from bytes by the shared av-core admission
//! (never trusting the producer), then executed by the AV interpreter
//! with the REAL Linux platform backend (raw syscalls via stable inline
//! asm — zero libc `syscall()`, zero std::fs/std::io on the primitive
//! path) and a console host that writes guest console bytes to the
//! process's real stdout/stderr fds, so the parent observes byte-exact
//! OS output.
//!
//! Protocol (exit 0 = infra OK; the ENTRY result travels in the summary
//! file so a guest failure can never be confused with a runner failure):
//! stdin  — unused
//! argv   — <wire-file> <owner-module> <entry-name> <grants-comma> <summary-file> [budget-ms]
//! stdout — the program's console stdout + platform stdout writes, raw
//! stderr — the program's console stderr + platform stderr writes, raw
//! summary — one JSON object (see `write_summary`)

// The runner is a Linux-only product (built for x86_64-unknown-linux-musl
// and executed inside the WSL2 acceptance environment); every item is
// cfg-gated so a Windows workspace build stays warning-clean with a stub.

use ace_av_core::av::{execute_platform, CancelFlag, ExecLimits};
use ace_av_core::contract::ExecOutcome;
use ace_av_core::contract::{Value, MIR_HOST_CONSOLE_SCHEMA};
use ace_av_core::host::{ConsoleHost, HostError, Stream};
use ace_av_core::host::{MAX_CALL_BYTES, MAX_RUN_BYTES};
use ace_av_core::mir::OwnerScope;
use ace_av_core::platform_io::LinuxPlatformIo;
use ace_av_core::verify_abc_bytes;
use std::io::Write as _;

#[cfg(target_os = "linux")]
/// Console host writing straight to the real std fds with the frozen
/// console/1 budgets (per-call 64 KiB, cumulative 1 MiB, dispatch count).
#[derive(Default)]
struct RealConsole {
    run_bytes: usize,
    dispatches: u64,
}

#[cfg(target_os = "linux")]
impl ConsoleHost for RealConsole {
    fn retained_bytes(&self) -> usize {
        0 // nothing retained: every byte goes to the OS immediately
    }
    fn write(
        &mut self,
        stream: Stream,
        bytes: &[u8],
        _cancel: &CancelFlag,
    ) -> Result<(), HostError> {
        self.dispatches += 1;
        if bytes.len() > MAX_CALL_BYTES {
            return Err(HostError::Resource);
        }
        self.run_bytes = self
            .run_bytes
            .checked_add(bytes.len())
            .ok_or(HostError::Resource)?;
        if self.run_bytes > MAX_RUN_BYTES {
            return Err(HostError::Resource);
        }
        let fd = match stream {
            Stream::Stdout => 1,
            Stream::Stderr => 2,
        };
        // Raw write(2) via the Linux backend's own syscall surface — the
        // runner is a musl binary but the console primitive path must not
        // delegate to libc stdio; use the same raw-syscall helper module.
        let mut off = 0usize;
        while off < bytes.len() {
            let n = unsafe {
                ace_av_core::platform_io::linux_raw_write(
                    fd,
                    bytes[off..].as_ptr() as usize,
                    bytes.len() - off,
                )
            };
            if n < 0 {
                let errno = -n;
                if errno == 4 {
                    continue; // EINTR with zero progress
                }
                return Err(HostError::Io(format!("console write errno {errno}")));
            }
            off += n as usize;
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn usage() -> ! {
    eprintln!(
        "usage: mn-linux-av-runner <wire-file> <owner-module> <entry> <grants-comma> <summary-file> [budget-ms]"
    );
    std::process::exit(2);
}

#[cfg(target_os = "linux")]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 5 {
        usage();
    }
    let wire_path = args[0].clone();
    let owner = args[1].clone();
    let entry = args[2].clone();
    let grants: Vec<String> = args[3]
        .split(',')
        .filter(|g| !g.is_empty())
        .map(|g| g.to_string())
        .collect();
    let summary_path = args[4].clone();
    let budget_ms: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(10_000);

    let start = std::time::Instant::now();

    // 1. Load + re-verify the ABC wire (the runner never trusts the
    //    producer; the same admission the restore path uses).
    let wire = match std::fs::read(&wire_path) {
        Ok(w) => w,
        Err(e) => fail_infra(&summary_path, format!("read wire: {e}")),
    };
    let loaded = match verify_abc_bytes(
        &wire,
        &OwnerScope {
            schema: MIR_HOST_CONSOLE_SCHEMA.into(),
            module: owner.clone(),
        },
    ) {
        Ok(l) => l,
        Err(e) => fail_infra(&summary_path, format!("verify abc: {e:?}")),
    };
    let module = loaded.module();
    if !module.is_host_console_profile() {
        fail_infra(&summary_path, "not the host-console profile".into());
    }
    let entry_matches = module
        .funcs
        .get(module.entry as usize)
        .map(|f| f.name.as_str())
        == Some(entry.as_str());
    if !entry_matches {
        fail_infra(&summary_path, "entry mismatch".into());
    }

    // 2. Execute with the REAL Linux backend (fail closed if this process
    //    cannot serve it — e.g. auxv page size unavailable).
    if !LinuxPlatformIo::available() {
        fail_infra(&summary_path, "linux platform backend unavailable".into());
    }
    let cancel = std::sync::Arc::new(CancelFlag::new(false));
    let limits = ExecLimits {
        wall: std::time::Duration::from_millis(budget_ms),
        ..ExecLimits::default()
    };
    // Deadline watchdog: flips the shared cancel flag at the budget; the
    // executor's own wall limit is the hard backstop.
    let watchdog = {
        let flag = std::sync::Arc::clone(&cancel);
        let dur = std::time::Duration::from_millis(budget_ms);
        std::thread::spawn(move || {
            std::thread::sleep(dur);
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
        })
    };
    let mut console = RealConsole::default();
    let mut backend = LinuxPlatformIo::default();
    let (outcome, _trace, summary_session) = match execute_platform(
        &loaded,
        &limits,
        &cancel,
        &grants,
        Some(&mut console),
        &mut backend,
    ) {
        Ok(triple) => triple,
        Err(e) => fail_infra(&summary_path, format!("admission: {e:?}")),
    };
    let _ = watchdog.join();

    let exit_value = match &outcome {
        ExecOutcome::Success { value, .. } => match value {
            Value::I32(n) => *n,
            _ => i32::MAX,
        },
        _ => -1,
    };
    let outcome_kind = match &outcome {
        ExecOutcome::Success { .. } => "success",
        ExecOutcome::Trap { .. } => "trap",
        ExecOutcome::Resource { .. } => "resource",
        ExecOutcome::Cancelled => "cancelled",
        ExecOutcome::Infra(_) => "infra",
    };
    write_summary(
        &summary_path,
        &format!(
            "{{\"outcome\":\"{outcome_kind}\",\"entry_value\":{exit_value},\
             \"console_dispatches\":{},\"console_run_bytes\":{},\
             \"platform_last_os_error\":{},\"platform_written\":{},\
             \"platform_leaked_cleaned\":{},\"platform_live_mapped_at_exit\":{},\
             \"elapsed_ms\":{}}}",
            console.dispatches,
            console.run_bytes,
            summary_session.last_os_error,
            summary_session.written_total_bytes,
            summary_session.leaked_regions_cleaned,
            summary_session.live_mapped_at_exit,
            start.elapsed().as_millis(),
        ),
    );
    std::process::exit(0);
}

#[cfg(target_os = "linux")]
fn fail_infra(summary_path: &str, message: String) -> ! {
    let json = format!(
        "{{\"outcome\":\"runner-infra\",\"message\":{}}}",
        json_escape(&message)
    );
    write_summary(summary_path, &json);
    eprintln!("mn-linux-av-runner: {message}");
    std::process::exit(3);
}

#[cfg(target_os = "linux")]
fn json_escape(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(target_os = "linux")]
fn write_summary(path: &str, json: &str) {
    if let Ok(mut f) = std::fs::File::create(path) {
        let _ = f.write_all(json.as_bytes());
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("mn-linux-av-runner is a Linux-only product (build for x86_64-unknown-linux-musl)");
    std::process::exit(2);
}
