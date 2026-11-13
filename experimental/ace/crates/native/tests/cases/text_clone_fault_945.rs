//! PLAN-945 V-04 native leg: run the text-runtime owned-clone fault probe
//! as a real native PE (rustc → COFF obj → rust-lld with the landed
//! `ac_text_runtime.lib`). Exit code 0 = the full contract held: the
//! faulted clone returns NULL without touching the source, the retried
//! clone is byte-identical in an independent buffer, both owners release,
//! and the runtime ledger converges (typed failures, never a panic).
#![cfg(all(windows, target_arch = "x86_64"))]

#[path = "../common/mod.rs"]
mod common;

#[test]
fn text_clone_fault_contract_holds_on_a_real_native_pe() {
    assert!(
        common::tools_available(),
        "native tools are required; no skip pass"
    );
    let work = std::env::temp_dir().join(format!(
        "plan945-text-fault-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&work).unwrap();
    let object = work.join("probe.obj");
    let mut compiler = std::process::Command::new("rustc");
    compiler
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/text_clone_fault_945.rs"),
        )
        .args([
            "--crate-type=lib",
            "--emit=obj",
            "--edition=2021",
            "-C",
            "panic=abort",
            "-C",
            "opt-level=1",
            "-o",
        ])
        .arg(&object);
    let output =
        auto_tool_execution::run_command_output(compiler).expect("compile 945 text fault probe");
    std::fs::write(work.join("compiler.log"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = std::fs::read(&object).unwrap();
    let text_lib = ace_av::text_runtime::text_runtime_lib().expect("text runtime lib");
    let run = ace_native::link_driver::link_and_run(
        &[],
        &bytes,
        "ac_start",
        &work.join("pe"),
        &[text_lib],
        "PLAN-945 text-runtime owned-clone fault probe",
    )
    .expect("real native runtime PE");
    std::fs::write(work.join("stderr.log"), &run.stderr).unwrap();
    assert_eq!(
        run.exit_code,
        Some(0),
        "probe failed; stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    // The faulted attempt reports the typed runtime failure (RT-FAIL), not a
    // crash; the summary confirms the ledger converged.
    assert!(stderr.contains("RT-FAIL"), "{stderr}");
    assert!(stderr.contains("SUMMARY attempts="), "{stderr}");
    std::fs::remove_dir_all(&work).ok();
}
