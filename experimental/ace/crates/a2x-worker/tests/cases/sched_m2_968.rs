//! PLAN-968 (M2-A): the scheduler surface on the a2r lane — task actor
//! sources through the Rust emit (`std` threads + bounded channels, the
//! same observable semantics as the AV lane) with byte-exact stdout
//! oracles. The generated wrapper appends `value:<exit>` after the drain.

use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;

fn run_a2x_sched_rust(name: &str, src: &str) -> String {
    // per-case store: the corpus declares the same synthesized driver names
    // (`__task_Echo`) with different bodies across cases — the fragment
    // store keys by decl identity, so each case owns its namespace.
    let store =
        std::env::temp_dir().join(format!("a2x-968-r-store-{name}-{}", std::process::id()));
    let dir = std::env::temp_dir().join(format!("a2x-968-r-out-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = A2xSession::open(
        &store,
        "a2r-968",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .expect("open Rust session");
    let asm = s
        .cold_assemble_profile(src, SourceProfile::Scheduler, &[], "main", false)
        .unwrap_or_else(|e| panic!("Rust assembly failed for {name}: {e:?}"));
    for (path, content) in &asm.files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
    let exe = dir.join("out.exe");
    let out =
        std::process::Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
            .args(["--edition", "2021", "-O", "-o"])
            .arg(&exe)
            .arg("main.rs")
            .current_dir(&dir)
            .output()
            .expect("spawn rustc");
    assert!(
        out.status.success(),
        "rustc failed for {name}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = std::process::Command::new(&exe).current_dir(&dir).output().expect("run exe");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&store);
    String::from_utf8_lossy(&run.stdout).to_string()
}

const CASES: &[(&str, &str, &str)] = &[
    (
        "message_handler",
        include_str!(
            "../../../../../../crates/auto-lang/test/cases/ace-m2/scheduler/src/message_handler.at"
        ),
        "got one\nvalue:0\n",
    ),
    // spawn_with_state (a Str state field + the legacy bare-brace fstr)
    // stays on the AV lane for M2-A: the a2r emit owned-text field-init
    // seam (a borrowed spawn initializer binding an owned field local)
    // is a NAMED DEFERRAL recorded in PLAN-968 receipts; the canonical
    // stays fully covered on the AV lane and in the case-runner corpus.
    (
        "mailbox_fifo",
        include_str!(
            "../../../../../../crates/auto-lang/test/cases/ace-m2/scheduler/src/mailbox_fifo.at"
        ),
        "got two\ngot one\nvalue:0\n",
    ),
    (
        "ask_reply",
        include_str!(
            "../../../../../../crates/auto-lang/test/cases/ace-m2/scheduler/src/ask_reply.at"
        ),
        "reply:\n42\nvalue:0\n",
    ),
    (
        "go_join",
        include_str!(
            "../../../../../../crates/auto-lang/test/cases/ace-m2/scheduler/src/go_join.at"
        ),
        "go:\n7\nvalue:0\n",
    ),
    (
        "backpressure",
        include_str!(
            "../../../../../../crates/auto-lang/test/cases/ace-m2/scheduler/src/backpressure.at"
        ),
        "1\n2\n3\nvalue:0\n",
    ),
    (
        "stop_hook",
        include_str!(
            "../../../../../../crates/auto-lang/test/cases/ace-m2/scheduler/src/stop_hook.at"
        ),
        "one\nstopped\nvalue:0\n",
    ),
];

#[test]
fn sched_968_a2r_rust_emit() {
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    for &(name, src, want) in CASES {
        let got = run_a2x_sched_rust(name, src);
        assert_eq!(got, want, "a2r scheduler case {name} stdout mismatch");
    }
}

fn tools_available() -> bool {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    std::process::Command::new(rustc)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}
