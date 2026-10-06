//! PLAN-745 T-05 source native execution tests: every positive corpus case
//! is built from `.at` source through the real pipeline (compile → verify →
//! lower → link) and executed as a Windows PE, asserting the corpus exit
//! code and stderr oracle. Every run carries a hard 60s deadline. Windows
//! only (native gate).

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

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
    let out = Command::new("cargo")
        .args(["build", "--locked"])
        .current_dir(Path::new(REPO).join("experimental/ac-core/test-support"))
        .output()
        .expect("spawn cargo for test-support");
    assert!(
        out.status.success(),
        "test-support build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let lib =
        Path::new(REPO).join("experimental/ac-core/test-support/target/debug/ac_trace_support.lib");
    assert!(lib.is_file(), "support lib missing at {}", lib.display());
    lib
}

/// Run a process with a hard deadline; returns (exit, stdout, stderr).
fn run_deadlined(mut cmd: Command, deadline: Duration) -> (i32, String, String) {
    let start = Instant::now();
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    // Poll try_wait so a wedged child is killed at the deadline instead of
    // hanging the suite (pipes are drained by the OS until read below).
    loop {
        match child.try_wait().expect("try_wait") {
            Some(_) => break,
            None if start.elapsed() >= deadline => {
                let _ = child.kill();
                panic!("process exceeded {:?} deadline", deadline);
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    }
    let out = child.wait_with_output().expect("collect output");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
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
        let (code, out, err) = run_deadlined(cmd, Duration::from_secs(120));
        assert_eq!(
            code, 0,
            "{}: build failed\nstdout: {}\nstderr: {}",
            name, out, err
        );

        let run = Command::new(&exe);
        let (code, _out, stderr) = run_deadlined(run, Duration::from_secs(60));
        assert_eq!(
            code, *want_exit,
            "{}: exit {} expected {} (stderr: {})",
            name, code, want_exit, stderr
        );
        // Trace oracle: stderr contains exactly the marks, in order.
        if !want_stderr.is_empty() {
            assert_eq!(stderr, *want_stderr, "{}: stderr oracle", name);
        } else {
            assert!(stderr.is_empty(), "{}: unexpected stderr: {}", name, stderr);
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

#[test]
fn trace_case_without_support_lib_fails_at_link() {
    // Compilation and capability gating succeed; only the unresolved
    // intrinsic import fails the link — exit 1, no artifacts.
    let dir = std::env::temp_dir().join("ac745-source-native-nolib");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("p15.exe");
    let (code, _out, err) = run_deadlined(
        {
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
            c
        },
        Duration::from_secs(120),
    );
    assert_eq!(code, 1, "stderr: {}", err);
    assert!(!exe.exists());
}
