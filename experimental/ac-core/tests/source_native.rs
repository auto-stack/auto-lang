//! PLAN-745 T-05 source native execution tests: every positive corpus case
//! is built from `.at` source through the real pipeline (compile → verify →
//! lower → link) and executed as a Windows PE, asserting the corpus exit
//! code and stderr oracle. Every subprocess — cargo, the CLI, the built PE —
//! runs under the shared deadline executor (concurrent pipe drain, kill +
//! reap on expiry; review QA-05). Windows only (native gate).

#![cfg(windows)]

mod common;

use common::run_deadlined;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const PROFILE: &str = "auto.source.core-i32.draft";
const TRACE: &str = "hir.test.trace.v1";

/// (fixture, exit, stderr, needs-trace-capability, needs-support-lib)
const POSITIVE: &[(&str, i32, &str, bool, bool)] = &[
    ("p01-precedence.at", 14, "", false, false),
    ("p02-signed-positional.at", 1, "", false, false),
    ("p03-bool-param-constant-encoding.at", 7, "", false, false),
    ("p04-while-retest.at", 3, "", false, false),
    ("p05-while-zero.at", 1, "", false, false),
    ("p06-continue-retests-condition.at", 3, "", false, false),
    ("p07-break-nearest-loop.at", 5, "", false, false),
    ("p08-recursion.at", 3, "", false, false),
    ("p09-forward-signature.at", 8, "", false, false),
    ("p10-shadow-rhs-outer.at", 5, "", false, false),
    ("p11-literal-min-max.at", 0, "", false, false),
    ("p12-unexecuted-trap-not-lifted-x1.at", 0, "", false, false),
    ("p13-add-overflow-trap.at", 70, "", false, false),
    ("p14-mul-overflow-trap.at", 70, "", false, false),
    ("p15-named-eval-order-x2.at", 12, "ba", true, true),
    ("p16-observable-before-trap-x9.at", 70, "a", true, true),
    ("p17-heterogeneous-binding-map.at", 6, "", false, false),
    ("p18-bool-return-local-branch.at", 4, "", false, false),
    ("p19-user-function-not-intrinsic.at", 6, "", true, false),
    ("p20-utf8-crlf-negative-min.at", 0, "", false, false),
    ("p21-mutual-recursion.at", 4, "", false, false),
    // review r1 QA-02: native-internal names are legal source names, as
    // called functions AND as entries (C05/C06)
    ("c05-ac-start-name.at", 3, "", false, false),
    ("c06-exitprocess-name.at", 4, "", false, false),
    // review r2 QA-01: multi-line bracketed subexpressions continue on the
    // closing line (D01..D05); prefix-colliding names coexist (D07)
    ("d01-call-multiline-then-add.at", 5, "", false, false),
    ("d02-paren-multiline-then-add.at", 6, "", false, false),
    ("d03-negative-multiline-arg.at", 1, "", false, false),
    ("d04-nested-paren-multiline.at", 6, "", false, false),
    ("d05-call-multiline-no-tail.at", 2, "", false, false),
    ("d07-colliding-prefix-names.at", 7, "", false, false),
    // R3-QA-02: a deep LEGAL operator chain builds and runs natively with
    // the exact i32 sum as the exit code (300; shell conventions would
    // show 300 & 0xFF, the test reads the real code).
    ("f11-deep-add-chain.at", 300, "", false, false),
    // r6 QA-01: binary operators inside call arguments — positional,
    // named-reversed, nested calls, bool args, bracketed controls, trace
    // order — the comma branch reduces the open argument first
    ("m01-comma-binop-args.at", 42, "", false, false),
    ("m02-named-binop-args.at", 42, "", false, false),
    ("m06-nested-call-binop-args.at", 192, "", false, false),
    ("m07-bool-binop-arg.at", 5, "", false, false),
    ("m10-parenthesized-binop-args.at", 42, "", false, false),
    ("m18-trace-named-binop-args.at", 23, "ba", true, true),
    ("m22-named-args-newline.at", 33, "", false, false),
];

fn valid(name: &str) -> String {
    Path::new(REPO)
        .join("experimental/ac-core/fixtures/source/valid")
        .join(name)
        .display()
        .to_string()
}

fn build_support_lib() -> PathBuf {
    // Standalone no_std crate; separate target dir to avoid cargo lock
    // contention with the outer test run (same policy as trace_execution).
    // Bounded wait like every other subprocess (QA-05).
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--locked"])
        .current_dir(Path::new(REPO).join("experimental/ac-core/test-support"));
    let r = run_deadlined(cmd, Duration::from_secs(300)).expect("spawn cargo for test-support");
    assert!(!r.timed_out, "test-support build exceeded 300s deadline");
    assert!(
        r.code == 0,
        "test-support build failed (exit {}):\n{}",
        r.code,
        r.stderr
    );
    let lib =
        Path::new(REPO).join("experimental/ac-core/test-support/target/debug/ac_trace_support.lib");
    assert!(lib.is_file(), "support lib missing at {}", lib.display());
    lib
}

#[test]
fn positive_corpus_executes_natively() {
    let support = build_support_lib();
    let root = std::env::temp_dir().join("ac745-source-native");
    let _ = std::fs::remove_dir_all(&root);
    for (name, want_exit, want_stderr, trace, need_lib) in POSITIVE {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("app.exe");
        let mut cmd = Command::new(EXE);
        cmd.args([
            "build-source",
            &valid(name),
            "--source-profile",
            PROFILE,
            "--entry",
            "main",
            "--output",
        ])
        .arg(&exe);
        if *trace {
            cmd.args(["--capability", TRACE]);
        }
        if *need_lib {
            cmd.arg("--support-lib").arg(&support);
        }
        let r = run_deadlined(cmd, Duration::from_secs(120)).expect("spawn build");
        assert!(!r.timed_out, "{}: build exceeded 120s deadline", name);
        assert_eq!(
            r.code, 0,
            "{}: build failed\nstdout: {}\nstderr: {}",
            name, r.stdout, r.stderr
        );

        let run = run_deadlined(Command::new(&exe), Duration::from_secs(60)).expect("run exe");
        assert!(!run.timed_out, "{}: exe exceeded 60s deadline", name);
        assert_eq!(
            run.code, *want_exit,
            "{}: exit {} expected {} (stderr: {})",
            name, run.code, want_exit, run.stderr
        );
        // Trace oracle: stderr contains exactly the marks, in order.
        if !want_stderr.is_empty() {
            assert_eq!(run.stderr, *want_stderr, "{}: stderr oracle", name);
        } else {
            assert!(
                run.stderr.is_empty(),
                "{}: unexpected stderr: {}",
                name,
                run.stderr
            );
        }
        // The publish transaction left all three artifacts.
        assert!(dir.join("app.obj").is_file(), "{}: obj missing", name);
        assert!(
            dir.join("app.ac-link.txt").is_file(),
            "{}: receipt missing",
            name
        );
    }
}

/// R5-QA-02 (block item): statement-block nesting is unlimited — a
/// 1001-deep if chain (the reviewer's K04/H08 boundary; the old NEST_LIMIT
/// counted the function body as a level and rejected 1000-deep ifs) builds
/// and runs natively with the correct exit.
#[test]
fn deep_block_nesting_builds_and_runs() {
    let dir = std::env::temp_dir().join("ac745-source-native-deepblocks");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let depth = 1001usize;
    let src = format!(
        "fn main() int {{ {}return 1{}\nreturn 0 }}",
        "if true { ".repeat(depth),
        " }".repeat(depth)
    );
    let src_path = dir.join("deep.at");
    std::fs::write(&src_path, src).unwrap();
    let exe = dir.join("deep.exe");
    let build = run_deadlined(
        {
            let mut c = Command::new(EXE);
            c.args([
                "build-source",
                &src_path.display().to_string(),
                "--source-profile",
                PROFILE,
                "--entry",
                "main",
                "--output",
            ])
            .arg(&exe);
            c
        },
        Duration::from_secs(300),
    )
    .expect("spawn build");
    assert!(!build.timed_out, "deep block build exceeded 300s");
    assert_eq!(build.code, 0, "stderr: {}", build.stderr);
    let run = run_deadlined(Command::new(&exe), Duration::from_secs(60)).expect("run exe");
    assert!(!run.timed_out, "deep block exe exceeded 60s");
    assert_eq!(run.code, 1, "1001 nested ifs must reach the return 1");
    assert!(run.stderr.is_empty());
}

/// QA-02: `ac_start` used directly as the ENTRY (not just as a callee).
#[test]
fn ac_start_as_entry_runs() {
    let dir = std::env::temp_dir().join("ac745-source-native-acstart");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("app.exe");
    let r = run_deadlined(
        {
            let mut c = Command::new(EXE);
            c.args([
                "build-source",
                &valid("c05-ac-start-name.at"),
                "--source-profile",
                PROFILE,
                "--entry",
                "ac_start",
                "--output",
            ])
            .arg(&exe);
            c
        },
        Duration::from_secs(120),
    )
    .expect("spawn build");
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    let run = run_deadlined(Command::new(&exe), Duration::from_secs(60)).expect("run");
    assert!(!run.timed_out);
    assert_eq!(run.code, 3);
}

/// QA-05 control: a child that floods stderr far past the OS pipe buffer
/// must complete under the deadline with the FULL output drained — the
/// poll-before-drain ordering used to wedge at 4096 chars.
#[test]
fn big_stderr_output_is_fully_drained() {
    let support = build_support_lib();
    let dir = std::env::temp_dir().join("ac745-source-native-bigout");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // 20_000 mark_a calls → 20_000 'a' chars on stderr, exit 20_000.
    let src = "fn main() int { var i = 0; var n = 0; while i < 20000 { let a = mark_a(); n = n + a; i = i + 1 }; return n }\n";
    let src_path = dir.join("big.at");
    std::fs::write(&src_path, src).unwrap();
    let exe = dir.join("big.exe");
    let build = run_deadlined(
        {
            let mut c = Command::new(EXE);
            c.args([
                "build-source",
                &src_path.display().to_string(),
                "--source-profile",
                PROFILE,
                "--entry",
                "main",
                "--capability",
                TRACE,
                "--output",
            ])
            .arg(&exe)
            .arg("--support-lib")
            .arg(&support);
            c
        },
        Duration::from_secs(120),
    )
    .expect("spawn build");
    assert_eq!(build.code, 0, "stderr: {}", build.stderr);
    let run = run_deadlined(Command::new(&exe), Duration::from_secs(60)).expect("run");
    assert!(!run.timed_out, "the drain must never wedge the wait");
    assert_eq!(run.code, 20_000, "unexpected exit: {}", run.code);
    assert_eq!(run.stderr.len(), 20_000, "stderr must be fully drained");
    assert!(run.stderr.chars().all(|c| c == 'a'));
}

/// QA-05 control: a non-terminating PE (an infinite `loop { ...; continue }`
/// is representable; verify only requires a conservative return path) must
/// be killed at a short deadline — never hang the suite.
#[test]
fn hanging_exe_is_killed_at_deadline() {
    let dir = std::env::temp_dir().join("ac745-source-native-hang");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // A state-free infinite loop (an incrementing counter would reach the
    // add_i32 trap in ~2^31 iterations and exit 70 BEFORE the deadline —
    // observed; `loop { continue }` has no arithmetic to trap on).
    let src = "fn main() int { loop { continue }; return 0 }\n";
    let src_path = dir.join("hang.at");
    std::fs::write(&src_path, src).unwrap();
    let exe = dir.join("hang.exe");
    let build = run_deadlined(
        {
            let mut c = Command::new(EXE);
            c.args([
                "build-source",
                &src_path.display().to_string(),
                "--source-profile",
                PROFILE,
                "--entry",
                "main",
                "--output",
            ])
            .arg(&exe);
            c
        },
        Duration::from_secs(120),
    )
    .expect("spawn build");
    assert_eq!(build.code, 0, "stderr: {}", build.stderr);
    let started = std::time::Instant::now();
    let run = run_deadlined(Command::new(&exe), Duration::from_secs(3)).expect("run");
    assert!(
        run.timed_out,
        "the infinite loop must hit the deadline (code={} out={} err={})",
        run.code,
        run.stdout.len(),
        run.stderr.len()
    );
    assert!(started.elapsed() < Duration::from_secs(20));
}

#[test]
fn trace_case_without_support_lib_fails_at_link() {
    // Compilation and capability gating succeed; only the unresolved
    // intrinsic import fails the link — exit 1, no artifacts.
    let dir = std::env::temp_dir().join("ac745-source-native-nolib");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("p15.exe");
    let (code, _out, err) = {
        let mut c = Command::new(EXE);
        c.args([
            "build-source",
            &valid("p15-named-eval-order-x2.at"),
            "--source-profile",
            PROFILE,
            "--entry",
            "main",
            "--capability",
            TRACE,
            "--output",
        ])
        .arg(&exe);
        let r = run_deadlined(c, Duration::from_secs(120)).expect("spawn build");
        (r.code, r.stdout, r.stderr)
    };
    assert_eq!(code, 1, "stderr: {}", err);
    assert!(!exe.exists());
}

/// R3-QA-01 control: the parent exits instantly but a DESCENDANT inherits
/// the pipes and holds them past the deadline — the shared contained
/// executor must return bounded (timeout semantics, never a truncated
/// success) and the Job-terminated tree must leave no survivor behind.
#[test]
fn descendant_holding_pipes_cannot_extend_the_deadline() {
    let before = ping_count();
    let started = std::time::Instant::now();
    let mut cmd = Command::new("cmd.exe");
    cmd.args(["/c", "start /b ping -n 6 127.0.0.1"]);
    cmd.current_dir(std::env::temp_dir());
    let r = run_deadlined(cmd, Duration::from_millis(800)).expect("spawn");
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "helper waited {}ms for a pipe-holding descendant",
        started.elapsed().as_millis()
    );
    assert!(r.timed_out, "collection incomplete within deadline");
    assert_eq!(r.code, -1, "a timed-out run never reports a success code");
    // the contained tree (including the surviving descendant) is dead;
    // tolerate transient pings owned by sibling test controls
    wait_ping_count_reaches(before);
}

/// R3-QA-01 control: a descendant that writes LATER (after its parent
/// exited). With a deadline large enough, EOF-driven collection must
/// deliver the late output as a real success — not a premature quiet
/// window.
#[test]
fn late_writing_descendant_output_is_fully_collected() {
    let mut cmd = Command::new("cmd.exe");
    cmd.args(["/c", "start /b cmd /c ping -n 2 127.0.0.1 >nul & echo LATE"]);
    cmd.current_dir(std::env::temp_dir());
    let r = run_deadlined(cmd, Duration::from_secs(10)).expect("spawn");
    assert!(!r.timed_out, "deadline must cover the late writer");
    assert!(
        r.stdout.contains("LATE"),
        "late descendant output must be collected: {:?}",
        r.stdout
    );
}

/// R3-QA-01 control: parent itself runs past the deadline with a
/// descendant — the whole tree is terminated and reaped, bounded by the
/// deadline.
#[test]
fn runaway_parent_with_descendant_is_tree_killed() {
    let before = ping_count();
    let started = std::time::Instant::now();
    let mut cmd = Command::new("cmd.exe");
    cmd.args(["/c", "start /b ping -n 30 127.0.0.1 & ping -n 30 127.0.0.1"]);
    cmd.current_dir(std::env::temp_dir());
    let r = run_deadlined(cmd, Duration::from_millis(800)).expect("spawn");
    assert!(r.timed_out);
    assert!(started.elapsed() < Duration::from_secs(3));
    wait_ping_count_reaches(before);
}

fn ping_count() -> usize {
    let out = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq PING.EXE", "/FO", "CSV", "/NH"])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| l.to_ascii_uppercase().contains("PING.EXE"))
            .count(),
        Err(_) => 0,
    }
}

/// Under parallel test execution, a sibling control's legitimately-alive
/// ping can be visible; OUR tree is dead the moment the executor returns.
/// Converge to the baseline within a bounded window instead of requiring an
/// instantaneous global zero.
fn wait_ping_count_reaches(baseline: usize) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while ping_count() > baseline {
        if std::time::Instant::now() > deadline {
            panic!(
                "ping survivors did not converge to baseline {} (still {})",
                baseline,
                ping_count()
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// R2-QA-02 control: invalid UTF-8 bytes on the pipe must survive as
/// replacement characters around the valid data - never an empty stream.
#[test]
fn invalid_utf8_output_is_preserved_lossily() {
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-Command",
        "$s=[Console]::OpenStandardOutput(); $b=[Text.Encoding]::ASCII.GetBytes('PREFIX '); \n         $s.Write($b,0,$b.Length); $s.WriteByte(255); \n         $b=[Text.Encoding]::ASCII.GetBytes(' SUFFIX'); $s.Write($b,0,$b.Length); $s.Flush()",
    ]);
    let r = run_deadlined(cmd, Duration::from_secs(30)).expect("spawn");
    assert!(!r.timed_out);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    assert!(
        r.stdout.contains("PREFIX") && r.stdout.contains("SUFFIX"),
        "output swallowed: {:?}",
        r.stdout
    );
    assert!(
        r.stdout.contains('\u{FFFD}'),
        "lossy marker expected: {:?}",
        r.stdout
    );
    assert!(!r.drain_error, "no reader error on a clean pipe");
}
