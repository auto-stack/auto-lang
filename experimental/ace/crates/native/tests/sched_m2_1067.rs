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

/// The seven positive corpus cases: real native execution, byte-exact
/// stdout, exit 0 (which also proves the zero-residue drain audit), no
/// stderr noise. Expectations are the documented contract values.
#[test]
fn scheduler_corpus_runs_native_byte_exact() {
    let cases = [
        ("message_handler", "got one\n"),
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
    // The shared front door reports the desugar/bind stage for this code.\n    assert_eq!(err.stage, "bind", "unknown-task stage");

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
