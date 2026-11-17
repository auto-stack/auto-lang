//! PLAN-1067 (M2-B): the native scheduler lane's permanent corpus.
//!
//! Independent execution evidence for the scheduler surface PLAN-968 froze
//! for AV/A2R, now on the MIR-native backend: the SAME corpus sources
//! (`crates/auto-lang/test/cases/ace-m2/scheduler/src/`) run as REAL PEs on
//! real OS threads, and their observable behavior is compared with the
//! documented per-case expectations (byte-exact stdout, exit codes, named
//! diagnostics) — never with an AV run at test time.
//!
//! Every case also closes the resource invariant implicitly: the lane
//! wrapper returns `SCHED_LEAK_EXIT_CODE` (72) instead of the entry's code
//! when the orderly drain finds a live task/mailbox/port/thread argument.
//! An exit-0 assertion therefore proves "all tasks joined, all buffers and
//! handles released", not merely "no crash".

use std::path::{Path, PathBuf};

fn corpus(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/ace-m2/scheduler/src")
        .join(format!("{name}.at"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("corpus {path:?}: {e}"))
}

fn workdir(case: &str) -> PathBuf {
    // Each build gets its own output directory: the link lane publishes the
    // exe + receipt transactionally and never reuses a previous product.
    static RUN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = RUN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("sched-m2-1067")
        .join(format!("{case}-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scheduler workdir");
    dir
}

#[derive(Debug)]
struct Observed {
    exit: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(case: &str) -> Observed {
    let src = corpus(case);
    match ace_native::sched::sched_native_run(&src, "d_main", &workdir(case)) {
        Ok(run) => Observed {
            exit: run.exit_code,
            stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
        },
        Err(e) => panic!("{case}: scheduler lane rejected: {e}"),
    }
}

/// The eight positive corpus cases: real native execution, byte-exact
/// stdout, exit 0 (which also proves the zero-residue drain audit), no
/// stderr noise. Expectations are the documented contract values.
#[test]
fn scheduler_corpus_runs_native_byte_exact() {
    let cases = [
        ("message_handler", "got one\n"),
        ("spawn_with_state", "mycounter: 15\n"),
        ("mailbox_fifo", "got two\ngot one\n"),
        ("ask_reply", "reply:\n42\n"),
        ("go_join", "go:\n7\n"),
        ("backpressure", "1\n2\n3\n"),
        ("stop_hook", "one\nstopped\n"),
        ("virtual_timer", "main\nlater\n"),
    ];
    for (case, expected) in cases {
        let observed = run(case);
        assert_eq!(
            observed.exit,
            Some(0),
            "{case}: non-zero exit (72 = drain residue/leak): {observed:?}",
            observed = observed.exit
        );
        assert_eq!(observed.stdout, expected, "{case}: stdout drift");
        assert_eq!(observed.stderr, "", "{case}: unexpected stderr");
    }
}

/// A stop request never discards a buffered delivery: the stop hook runs
/// only after the queued message dispatched (the `one` before `stopped`
/// order above is the same invariant), and a send to a stopped/closed task
/// is the named language error — never a silent success.
#[test]
fn scheduler_send_after_stop_is_the_named_language_error() {
    let observed = run("neg_send_after_stop");
    assert_eq!(
        observed.exit,
        Some(70),
        "send-after-stop must be the native trap exit, got {observed:?}",
        observed = observed.exit
    );
    assert!(
        observed.stderr.contains("send on a closed task"),
        "named error missing: {:?}",
        observed.stderr
    );
}

/// The front-end rejections keep their exact AV diagnostics: an unknown
/// task name is a resolve-stage `source.sched-task`, a malformed arm is a
/// parse-stage `source.unsupported`. The native lane must not turn them
/// into a backend or runtime failure.
#[test]
fn scheduler_frontend_rejections_are_named() {
    let source = corpus("neg_unknown_task");
    let err = ace_native::sched::sched_compile(&source, "d_main")
        .expect_err("unknown task must be rejected");
    assert_eq!(err.code, "source.sched-task", "unknown-task diagnostic");
    // The shared front door reports the desugar/bind stage for this code.
    assert_eq!(err.stage, "bind", "unknown-task stage");

    let source = corpus("neg_malformed_task");
    let err = ace_native::sched::sched_compile(&source, "d_main")
        .expect_err("malformed task must be rejected");
    assert_eq!(err.code, "source.unsupported", "malformed-task diagnostic");
    assert_eq!(err.stage, "text", "malformed-task stage");
}

/// Repeated runs are independent: the runtime registry is per-process and
/// the lane must not leak state across PE runs (each run is a new process,
/// but the compiler side prepares the runtime once — the second run reuses
/// the content-bound library identity instead of rebuilding).
#[test]
fn scheduler_runs_are_repeatable() {
    let first = run("message_handler");
    let second = run("message_handler");
    assert_eq!(first.exit, Some(0));
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

/// Run one inline scheduler program (PLAN-1067 V-02 rows): the same lane,
/// with the source supplied by the test instead of the frozen corpus.
fn run_source(case: &str, src: &str) -> Observed {
    match ace_native::sched::sched_native_run(src, "d_main", &workdir(case)) {
        Ok(run) => Observed {
            exit: run.exit_code,
            stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
        },
        Err(e) => panic!("{case}: scheduler lane rejected: {e}"),
    }
}

/// V-02 (owned value across the spawn boundary): a task state field
/// declared/initialized as owned text crosses `Task.spawn` — the runtime
/// materializes the initializer into a fresh owned buffer, the driver keeps
/// it as its state, prints it, and the receiver's exit sweep frees it.
#[test]
fn scheduler_owned_text_state_crosses_spawn() {
    let src = r#"
task Counter {
    label = "default"
    on { n int -> {
        print(label)
    } }
}
fn main() {
    let h = Task.spawn("Counter", 4, "my")
    h.send(1)
}
"#;
    let observed = run_source("owned-text-state", src);
    assert_eq!(observed.exit, Some(0), "{observed:?}");
    assert_eq!(observed.stdout, "my\n", "owned-state drift");
    assert_eq!(observed.stderr, "", "trace/noise on stderr");
}

/// V-02 (typed result across the suspension point): the reply domain is not
/// hard-wired to i32 — a `bool` reply is inferred, carried through the port,
/// and awaited as a bool by the caller.
#[test]
fn scheduler_typed_reply_crosses_the_await() {
    let src = r#"
task Q {
    on {
        n int -> {
            reply(true)
        }
    }
}
fn main() {
    let h = Task.spawn("Q", 4)
    let v = await h.ask(1)
    print(v)
}
"#;
    let observed = run_source("typed-reply", src);
    assert_eq!(observed.exit, Some(0), "{observed:?}");
    assert_eq!(observed.stdout, "true\n", "bool reply drift");
    assert_eq!(observed.stderr, "", "trace/noise on stderr");
}

/// V-03 (resource leg): an ask whose future is never awaited leaves a
/// published reply behind; the orderly drain releases it, so the
/// zero-residue audit still passes (exit 0, never 72).
#[test]
fn scheduler_unawaited_reply_is_released_at_drain() {
    let src = r#"
task Silent {
    on {
        n int -> {
            reply(n + 1)
        }
    }
}
fn main() {
    let h = Task.spawn("Silent", 4)
    let port = h.ask(7)
    print("main")
}
"#;
    let observed = run_source("unawaited-reply", src);
    assert_eq!(observed.exit, Some(0), "{observed:?}");
    assert_eq!(observed.stdout, "main\n", "main output drift");
    assert_eq!(observed.stderr, "", "trace/noise on stderr");
}

/// PINNED BOUNDARY (PLAN-1067 §10 named gap): an owned-text MESSAGE or
/// REPLY is still rejected by the shared front end, because the scheduler
/// intrinsic signatures demand owned `Str` while the surface only produces
/// borrowed views (`verify.type-mismatch`), and the resolver keeps its
/// documented `source.sched-argument` rejection. This test pins the exact
/// current behaviour so closing the gap must update it deliberately.
#[test]
fn scheduler_owned_text_message_and_reply_stay_named_rejections() {
    let message = r#"
task Greeter {
    on {
        who Str -> {
            print(who)
        }
    }
}
fn main() {
    let h = Task.spawn("Greeter", 4)
    h.send("hello")
}
"#;
    let err = ace_native::sched::sched_compile(message, "d_main")
        .expect_err("owned-text message must stay rejected");
    assert_eq!(err.code, "source.sched-argument", "message rejection code");

    let reply = r#"
task Echo {
    on {
        n int -> {
            reply("v42")
        }
    }
}
fn main() {
    let h = Task.spawn("Echo", 4)
    let v = await h.ask(1)
    print(v)
}
"#;
    let err = ace_native::sched::sched_compile(reply, "d_main")
        .expect_err("owned-text reply must stay rejected");
    assert_eq!(err.code, "source.sched-argument", "reply rejection code");
}
/// Build (untimed) and then run (timed) one source, so a wall-clock oracle
/// measures ONLY the PE run — never compilation, linking or runtime
/// preparation.
fn run_timed(case: &str, src: &str) -> (Observed, std::time::Duration) {
    let module = ace_native::sched::sched_compile(src, "d_main")
        .unwrap_or_else(|d| panic!("{case}: front end rejected: {} {}", d.code, d.message));
    let linked = ace_native::sched::sched_native_build(&module, "d_main", &workdir(case))
        .unwrap_or_else(|e| panic!("{case}: build failed: {e}"));
    let started = std::time::Instant::now();
    let run = ace_native::link_driver::run_pe(
        &linked.exe_path,
        ace_native::config::RUN_BUDGET,
        ace_core::link::CaptureCaps {
            stdout_max: ace_native::config::STREAM_CAP,
            stderr_max: ace_native::config::STREAM_CAP,
        },
    )
    .unwrap_or_else(|e| panic!("{case}: run failed: {e}"));
    let elapsed = started.elapsed();
    (
        Observed {
            exit: run.exit_code,
            stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
        },
        elapsed,
    )
}

/// V-04 (real OS wait, distinguishing oracle): a task that sleeps 40 ms
/// before replying must make the PE actually WAIT. A busy-loop-free no-op
/// sleep (the failure mode a virtual clock would produce on this lane)
/// finishes in single-digit milliseconds; the oracle is the measured process
/// wall time, not just the printed bytes.
#[test]
fn scheduler_sleep_is_a_real_os_wait() {
    let src = r#"
task Slow {
    on {
        n int -> {
            sleep(40)
            reply(n + 1)
        }
    }
}
fn main() {
    let h = Task.spawn("Slow", 4)
    let v = await h.ask(41)
    print(v)
}
"#;
    let (observed, elapsed) = run_timed("real-wait", src);
    assert_eq!(observed.exit, Some(0), "{observed:?}");
    assert_eq!(observed.stdout, "42\n", "sleep/reply drift");
    assert_eq!(observed.stderr, "", "trace/noise on stderr");
    assert!(
        elapsed >= std::time::Duration::from_millis(35),
        "sleep did not actually wait: run took only {elapsed:?}"
    );
}

/// V-03 (cancellation, no hang): an ask whose actor stops WITHOUT replying
/// terminates the waiter with a named error. The failure mode guarded here
/// is a permanent park (the process would then hit the run budget instead of
/// reporting a language-level outcome).
#[test]
fn scheduler_await_on_unreplied_ask_is_a_named_error() {
    let src = r#"
task Mute {
    fn stop() ! {
    }
    on {
        n int -> {
        }
    }
}
fn main() {
    let h = Task.spawn("Mute", 4)
    let p = h.ask(1)
    h.stop()
    let v = await p
    print(v)
}
"#;
    let (observed, elapsed) = run_timed("await-unreplied", src);
    assert_eq!(
        observed.exit,
        Some(70),
        "expected the named language-error trap, got {observed:?}"
    );
    assert!(
        observed.stderr.contains("closed without replying"),
        "named outcome missing: {:?}",
        observed.stderr
    );
    assert_eq!(
        observed.stdout, "",
        "no value must be printed: {observed:?}"
    );
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "the waiter parked instead of terminating: {elapsed:?}"
    );
}

/// V-03/V-04 (exit cleanup, composite): several live tasks, a buffered
/// message nobody consumed, an unawaited reply and a timer all outstanding
/// when `main` returns. The orderly drain must join every thread, release
/// every buffer/port and report a zero residue (exit 0 — never 72 — and no
/// hang).
#[test]
fn scheduler_exit_drain_releases_every_outstanding_owner() {
    let src = r#"
task A {
    on { n int -> {
        print(n)
    } }
}
task B {
    fn stop() ! {
        print("b-stopped")
    }
    on { n int -> {
        reply(n * 2)
    } }
}
fn main() {
    let a = Task.spawn("A", 1)
    let b = Task.spawn("B", 4)
    let p = b.ask(21)
    a.send(1)
    a.send(2)
    go tick()
    print("main")
}
fn tick() {
    sleep(5)
    print("tick")
}
"#;
    // NOTE: this source deliberately prints scalars, not f-strings. An owned
    // text temporary re-created on a SECOND loop iteration is not yet
    // released per iteration by the shared owned-text lowering (recorded in
    // the Plan as a named debt); using one here would make this test measure
    // that unrelated gap instead of the lane's exit cleanup.
    let (observed, elapsed) = run_timed("exit-drain", src);
    assert_eq!(
        observed.exit,
        Some(0),
        "drain residue or failure at exit: {observed:?}"
    );
    assert_eq!(observed.stderr, "", "trace/noise on stderr");
    assert!(
        observed.stdout.contains("main\n"),
        "main output missing: {observed:?}"
    );
    assert!(
        elapsed < std::time::Duration::from_secs(10),
        "exit drain did not converge: {elapsed:?}"
    );
}
/// V-01 (triad, AV leg): the SAME corpus sources run on the AV lane in this
/// process (shared front end → closures lowering → verify → ABC admit → the
/// deterministic cooperative scheduler). Each lane is checked against the
/// DOCUMENTED per-case expectation independently — the lanes are never
/// compared against each other as the oracle (they share a front end, so
/// agreement alone would hide a shared defect).
///
/// The A2R leg lives in its own lane's harness
/// (`ace-a2x-worker` `sched_m2_968::sched_968_a2r_rust_emit` +
/// `sched_1067_a2r_timer_and_closed_task`), which emits Rust, compiles it with
/// rustc and runs it — never reusing this lane's objects.
mod av_leg {
    use ace_av::console_host::ConsoleCapture;
    use ace_av_core::abc::encode_abc;
    use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
    use ace_av_core::contract::ExecOutcome;
    use ace_av_core::verify_mir;

    fn run(src: &str) -> Result<(ExecOutcome, String), String> {
        let compiled =
            ace_core::source::compile_scheduler_source(src, "probe.at", &[]).map_err(|ds| {
                ds.first()
                    .map(|d| format!("{}/{}", format!("{:?}", d.stage).to_lowercase(), d.code))
                    .unwrap_or_default()
            })?;
        let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0)
            .map_err(|e| format!("lower: {e:?}"))?;
        verify_mir(&mir).map_err(|e| format!("verify: {e:?}"))?;
        let abc = encode_abc(&mir).map_err(|e| format!("encode: {e:?}"))?;
        let flat = abc.to_flat();
        let loaded = ace_av_core::abc_verify::verify_abc_bytes(&flat, &mir.owner)
            .map_err(|e| e.to_string())?;
        let mut host = ConsoleCapture::default();
        let (out, _trace) = execute_sched(
            &loaded,
            &ExecLimits::default(),
            &CancelFlag::new(false),
            &mut host,
        );
        Ok((out, String::from_utf8_lossy(&host.stdout).to_string()))
    }

    #[test]
    fn av_lane_matches_the_documented_corpus_expectations() {
        for (case, expected) in [
            ("message_handler", "got one\n"),
            ("spawn_with_state", "mycounter: 15\n"),
            ("mailbox_fifo", "got two\ngot one\n"),
            ("ask_reply", "reply:\n42\n"),
            ("go_join", "go:\n7\n"),
            ("backpressure", "1\n2\n3\n"),
            ("stop_hook", "one\nstopped\n"),
            ("virtual_timer", "main\nlater\n"),
        ] {
            let src = super::corpus(case);
            let (out, stdout) = run(&src).unwrap_or_else(|e| panic!("{case}: AV rejected: {e}"));
            assert_eq!(stdout, expected, "{case}: AV stdout drift");
            match out {
                ExecOutcome::Success { value, .. } => match value {
                    ace_av_core::contract::Value::I32(v) => assert_eq!(v, 0, "{case}: AV exit"),
                    other => panic!("{case}: non-i32 AV exit {other:?}"),
                },
                other => panic!("{case}: expected AV success, got {other:?}"),
            }
        }
    }

    /// The three negative corpus legs, AV classification: two are front-end
    /// rejections (shared by every lane) and one is a runtime language error.
    #[test]
    fn av_lane_rejections_match_the_documented_expectations() {
        let err = run(&super::corpus("neg_unknown_task")).expect_err("must be rejected");
        assert_eq!(err, "bind/source.sched-task", "unknown-task diagnostic");
        let err = run(&super::corpus("neg_malformed_task")).expect_err("must be rejected");
        assert_eq!(err, "text/source.unsupported", "malformed-task diagnostic");

        let (out, stdout) = run(&super::corpus("neg_send_after_stop")).expect("compiles");
        match out {
            ExecOutcome::Error { message, .. } => {
                assert_eq!(message, "send on a closed task");
                assert_eq!(stdout, "", "no value may be produced");
            }
            other => panic!("expected the named language error, got {other:?}"),
        }
    }
}
/// V-04/V-05 (T-07): the REAL candidate path — build the scheduler PE into an
/// EXPLICIT out-of-repo output root, run it from there as an ordinary process
/// (no cargo, no workspace cwd), and audit what it actually depends on:
///
/// * exactly one DLL (kernel32.dll) and only the scheduler lane's declared
///   entry points — no CRT (`msvcrt`/`ucrtbase`/`vcruntime`), no
///   `api-ms-win-crt-*`, no stray Win32 surface, no ordinal imports;
/// * the support staticlib the PE embeds is the CONTENT-BOUND runtime
///   (identity hash equals the library's current bytes), so an unattested
///   runtime cannot be silently linked;
/// * the run itself is byte-exact and exits through the drain audit.
///
/// Linux: this lane lowers COFF for the frozen Windows triple only — there is
/// no Linux branch to test here, and Windows evidence never upgrades the Linux
/// rows (implementation + full cases are handed to PLAN-1063, recorded as
/// NOT_RUN in the plan/spec, never as verified).
#[test]
fn scheduler_candidate_runs_out_of_repo_kernel32_only() {
    let src = corpus("spawn_with_state");
    let module = ace_native::sched::sched_compile(&src, "d_main").expect("front end");
    // The explicit out-of-repo output root is the plan's sanctioned scratch
    // home; `AUTO_LANG_PLAN_ARTIFACTS` may relocate it, and a machine without
    // that root still exercises the SAME audit from the target tmp dir (the
    // audit, not the directory, is what this test is about).
    let candidate = {
        let preferred = std::env::var_os("AUTO_LANG_PLAN_ARTIFACTS")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("D:/autostack/plan-artifacts/plan-1067"))
            .join("candidate-t07");
        match std::fs::create_dir_all(&preferred) {
            Ok(()) => preferred,
            Err(_) => {
                let fallback = Path::new(env!("CARGO_TARGET_TMPDIR")).join("candidate-t07");
                std::fs::create_dir_all(&fallback).expect("candidate dir");
                fallback
            }
        }
    };
    // A candidate root is a scratch area: drop earlier runs' products so the
    // directory never grows without bound across reruns.
    for entry in std::fs::read_dir(&candidate)
        .into_iter()
        .flatten()
        .flatten()
    {
        let path = entry.path();
        if path.is_file() {
            let _ = std::fs::remove_file(path);
        }
    }

    let linked = ace_native::sched::sched_native_build(&module, "d_main", &candidate)
        .expect("build scheduler candidate");
    assert!(
        linked.exe_path.starts_with(&candidate),
        "the candidate must live in the explicit output root, got {:?}",
        linked.exe_path
    );

    // ---- the run: an ordinary process, started in the candidate directory
    let run = std::process::Command::new(&linked.exe_path)
        .current_dir(&candidate)
        .output()
        .expect("run the candidate PE");
    assert_eq!(run.status.code(), Some(0), "candidate exit drift");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "mycounter: 15\n",
        "candidate stdout drift"
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stderr),
        "",
        "candidate stderr noise"
    );

    // ---- the dependency audit on the published bytes
    let bytes = std::fs::read(&linked.exe_path).expect("read candidate PE");
    assert_eq!(&bytes[..2], b"MZ", "not a PE image");
    let report = ace_native::sched::audit_sched_pe_imports(&bytes).expect("import audit");
    // Windows import names are case-insensitive (`KERNEL32.dll` in the image).
    let mut dlls: Vec<String> = report.dlls.iter().map(|d| d.to_lowercase()).collect();
    dlls.sort();
    dlls.dedup();
    assert_eq!(
        dlls,
        vec!["kernel32.dll".to_string()],
        "the scheduler PE must depend on kernel32.dll only"
    );
    let lower = String::from_utf8_lossy(&bytes).to_lowercase();
    for forbidden in [
        "msvcrt.dll",
        "ucrtbase.dll",
        "vcruntime",
        "api-ms-win-crt",
        "kernelbase.dll",
        "bcrypt.dll",
    ] {
        assert!(
            !lower.contains(forbidden),
            "forbidden dependency `{forbidden}` appears in the candidate image"
        );
    }
    // Every observed import is one of the lane's declared entry points.
    for (dll, sym) in &report.symbols {
        assert_eq!(dll.to_lowercase(), "kernel32.dll");
        assert!(
            ace_native::sched::SCHED_KERNEL32_IMPORTS.contains(&sym.as_str()),
            "undeclared import `{sym}` reached the candidate"
        );
    }
    assert!(
        report.symbols.iter().any(|(_, s)| s == "CreateThread"),
        "expected the task-chain thread entry point, got {:?}",
        report.symbols
    );

    // ---- the embedded runtime is the content-bound one
    let prepared = ace_native::sched_runtime::prepare().expect("runtime identity");
    let library_hash = ace_native::sched_runtime::file_hash(&prepared.library).expect("hash");
    assert_eq!(
        prepared.identity.content_hash, library_hash,
        "the linked runtime library does not match its build-bound identity"
    );
    assert_eq!(prepared.identity.abi, "mn.sched.runtime/1");
}
/// V-05 (T-07 negative control): the audit must FAIL on a mutant, otherwise
/// "kernel32-only" is a slogan, not a check. This rewrites the image's import
/// DLL name to a CRT DLL of the same slot and expects the named rejection —
/// the rest of the PE stays structurally valid, so only the audit can catch it.
#[test]
fn scheduler_candidate_audit_rejects_a_mutant_dependency() {
    let src = corpus("message_handler");
    let module = ace_native::sched::sched_compile(&src, "d_main").expect("front end");
    let dir = workdir("audit-mutant");
    let linked =
        ace_native::sched::sched_native_build(&module, "d_main", &dir).expect("build candidate");
    let mut bytes = std::fs::read(&linked.exe_path).expect("read candidate PE");
    let needle = b"KERNEL32.dll";
    let at = bytes
        .windows(needle.len())
        .position(|w| w == needle)
        .expect("import DLL name present");
    // Same slot, different DLL: a CRT dependency the lane must never have.
    bytes[at..at + needle.len()].copy_from_slice(b"MSVCRT.dll\0\0");

    let err = ace_native::sched::audit_sched_pe_imports(&bytes)
        .expect_err("a CRT dependency must be rejected");
    assert!(
        err.code().contains("pe.audit.forbidden-dll"),
        "expected the named forbidden-dll rejection, got {}: {err}",
        err.code()
    );
}
/// R-1067-1 (review blocker): a `go` chain started from INSIDE a task handler
/// races the program's exit drain. The drain snapshots the chain table under
/// the registry lock, so publication and thread creation must happen under
/// that same lock — otherwise the chain is created after the snapshot, never
/// joined, leaks its argument blocks and reports a spurious residue (exit 72)
/// while its output races `ExitProcess`. Twenty consecutive runs must all be
/// byte-exact with exit 0.
#[test]
fn scheduler_go_from_task_handler_survives_the_drain() {
    let src = r#"
task A {
    on { n int -> {
        go helper(n)
    } }
}
fn helper(n int) {
    print(n)
}
fn main() {
    let a = Task.spawn("A", 4)
    a.send(5)
}
"#;
    for run_index in 0..20 {
        let observed = run_source(&format!("go-from-task-{run_index}"), src);
        assert_eq!(
            observed.exit,
            Some(0),
            "run {run_index}: drain residue or failure: {observed:?}"
        );
        assert_eq!(
            observed.stdout, "5\n",
            "run {run_index}: output lost or duplicated"
        );
        assert_eq!(observed.stderr, "", "run {run_index}: stderr noise");
    }
}

/// R-1067-2 (review major): a DUPLICATE reply must not resurrect a consumed
/// future. The first reply wins; the second is dropped; the second `await`
/// reports the named already-consumed outcome (the AV lane asserted the same
/// outcome after its publish-only-from-Pending repair).
#[test]
fn scheduler_duplicate_reply_never_resurrects_the_future() {
    let src = r#"
task L {
    on {
        n int -> {
            reply(1)
            reply(2)
        }
    }
}
fn main() {
    let h = Task.spawn("L", 4)
    let p = h.ask(9)
    let a = await p
    print(a)
    sleep(60)
    let b = await p
    print(b)
}
"#;
    let observed = run_source("dup-reply", src);
    assert_eq!(
        observed.exit,
        Some(70),
        "expected the named trap: {observed:?}"
    );
    assert_eq!(
        observed.stdout, "1\n",
        "the second reply must not be delivered"
    );
    assert!(
        observed
            .stderr
            .contains("await on an already-consumed future"),
        "named outcome missing: {:?}",
        observed.stderr
    );
}

/// R-1067-3 (review major): mailbox capacity 0 is refused by name on every
/// lane (it used to park every sender forever: a native hang, an AV deadlock
/// resource, an A2R silent clamp to 1).
#[test]
fn scheduler_zero_capacity_is_a_named_error() {
    let src = r#"
task Z {
    on { n int -> {
        print(n)
    } }
}
fn main() {
    let h = Task.spawn("Z", 0)
    h.send(1)
    print(99)
}
"#;
    let (observed, elapsed) = run_timed("zero-capacity", src);
    assert_eq!(
        observed.exit,
        Some(70),
        "expected the named trap: {observed:?}"
    );
    assert_eq!(observed.stdout, "", "no value may be produced");
    assert!(
        observed
            .stderr
            .contains("task mailbox capacity must be at least 1"),
        "named outcome missing: {:?}",
        observed.stderr
    );
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "capacity 0 must fail fast, not hang: {elapsed:?}"
    );
}

/// R-1067-11 (review minor): spawning a task from inside a task driver is a
/// named FRONT-END boundary, not a backend callee-table error.
#[test]
fn scheduler_nested_spawn_is_a_named_frontend_rejection() {
    let src = r#"
task A {
    on { n int -> {
        let b = Task.spawn("B", 4)
    } }
}
task B {
    on { n int -> {
    } }
}
fn main() {
    let a = Task.spawn("A", 4)
    a.send(1)
}
"#;
    let err =
        ace_native::sched::sched_compile(src, "d_main").expect_err("nested spawn must be rejected");
    assert_eq!(err.code, "source.sched-task", "nested-spawn diagnostic");
    assert!(
        err.message.contains("inside a task driver"),
        "message must name the boundary: {}",
        err.message
    );
}
