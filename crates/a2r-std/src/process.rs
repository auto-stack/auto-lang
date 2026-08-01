//! Process module — detached process spawning.
//!
//! Plan 013 G6: provides the runtime for transpiled `process.spawn(args)`,
//! used by the auto-ai-client daemon bootstrap. `args` is a Vec<String> whose
//! first element is the program path; the rest are arguments. Returns the
//! spawned child's PID (>0 on success, 0 on failure) — matching Auto's
//! `process.spawn` contract.

use std::process::{Command, Stdio};

/// Spawn a detached process. `args[0]` is the program; `args[1..]` are args.
/// Returns the child PID on success, 0 on failure.
pub fn spawn(args: Vec<String>) -> u32 {
    if args.is_empty() {
        return 0;
    }
    let prog = &args[0];
    let rest = &args[1..];
    match Command::new(prog)
        .args(rest)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child.id(),
        Err(_) => 0,
    }
}
