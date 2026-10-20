//! T-02 first-case gate (PLAN-818 §8, V-01/V-04/V-10 first slices): the
//! frozen two-function trace vertical runs through the NEW
//! `CheckedMir → per-function COFF → multi-object PE` path, the AV leg,
//! and the landed old HIR-native leg — all against the INDEPENDENT
//! hand-computed golden (attachments/818/golden.md §1: exit 0, stderr
//! "aab", no trap). Cache: a real store put → a NEW-process worker
//! restore → hit skips codegen while still linking and running.

use ac_mn::driver::MnDriver;

/// The frozen first-case source (golden.md §1.1, byte-frozen).
pub const FIRST_CASE_SRC: &str = "fn gate(v int) int {\n    let a = mark_a()\n    let t = v * 3 + a\n    if t > 40 {\n        return t - mark_b()\n    }\n    return t + a\n}\n\nfn main() int {\n    let x = gate(9)\n    let y = gate(15)\n    if x + y != 73 {\n        return 1\n    }\n    return 0\n}\n";

/// Independent hand-audited expectations (golden.md §1.3).
pub const FIRST_CASE_EXIT: i32 = 0;
pub const FIRST_CASE_STDERR: &[u8] = b"aab";
pub const TRACE_CAP: &str = "hir.test.trace.v1";

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn trace_support_lib() -> std::path::PathBuf {
    let support = repo_root().join("experimental/ac-core/test-support");
    let lib = support.join("target/debug/ac_trace_support.lib");
    if !lib.is_file() {
        let out = std::process::Command::new("cargo")
            .args(["build", "--locked"])
            .current_dir(&support)
            .output()
            .expect("spawn cargo for test-support");
        assert!(
            out.status.success(),
            "test-support build failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert!(lib.is_file(), "support lib missing at {}", lib.display());
    lib
}

fn tools_available() -> bool {
    auto_ac_prototype::link::find_rust_lld().is_ok()
        && auto_ac_prototype::link::find_sdk_um_dir().is_ok()
}

#[test]
#[cfg(windows)]
fn mir_native_first_new_pe_matches_independent_golden() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — first case skipped");
        return;
    }
    let workdir = std::env::temp_dir().join(format!("mn8-first-ephemeral-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let lib = trace_support_lib();
    let mut d = MnDriver::ephemeral();
    let r = d.request(
        FIRST_CASE_SRC,
        "d_main",
        &[TRACE_CAP.to_string()],
        &workdir,
        Some(&lib),
    );
    assert!(r.ok, "first case failed: {:?}", r.error);
    // The golden binds observable behavior only.
    assert_eq!(
        r.exit_code,
        Some(FIRST_CASE_EXIT),
        "PE exit must match the independent golden"
    );
    assert_eq!(
        r.stderr, FIRST_CASE_STDERR,
        "PE stderr bytes must be \"aab\" in source order"
    );
    assert!(r.stdout.is_empty(), "no stdout expected");
    // The ephemeral path still exercised the FULL new codegen: two unit
    // objects + one wrapper emitted, one real link, one real run.
    assert_eq!(
        *r.obs.counters.get("codegen_emit_units").unwrap_or(&0),
        2,
        "two function units compiled fresh"
    );
    assert!(
        *r.obs.counters.get("link_invocations").unwrap_or(&0) >= 1,
        "a real rust-lld invocation"
    );
    assert!(
        *r.obs.counters.get("run_invocations").unwrap_or(&0) >= 1,
        "a real PE run"
    );
    eprintln!("first-case trace: {}", r.obs.render());
    let _ = std::fs::remove_dir_all(&workdir);
}

#[test]
#[cfg(windows)]
fn mir_native_first_three_path_parity() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — parity skipped");
        return;
    }
    let caps = vec![TRACE_CAP.to_string()];
    let lib = trace_support_lib();

    // Leg 1 — AV (798 delivered executor): value + source-order events.
    let mut av = auto_av_probe::driver::Driver::ephemeral();
    let avr = av.request(FIRST_CASE_SRC, "d_main", &caps);
    assert!(avr.diag.is_none(), "AV leg rejected: {:?}", avr.diag);
    assert_eq!(avr.value_i32(), Some(FIRST_CASE_EXIT));
    use auto_av_core::mir::TraceEvent::*;
    assert_eq!(avr.trace, vec![MarkA, MarkA, MarkB]);

    // Leg 2 — old HIR-native reference (landed path, real PE).
    let old_workdir = std::env::temp_dir().join(format!("mn8-first-oldref-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&old_workdir);
    let old = auto_av_probe::native_reference::build_and_run(
        FIRST_CASE_SRC,
        &caps,
        "d_main",
        &old_workdir,
        Some(&lib),
    )
    .expect("old-native build+run");
    assert_eq!(old.exit_code, FIRST_CASE_EXIT, "old native exit");
    assert_eq!(old.stderr.as_bytes(), FIRST_CASE_STDERR, "old native marks");
    let _ = std::fs::remove_dir_all(&old_workdir);

    // Leg 3 — the NEW MIR-native path.
    let workdir = std::env::temp_dir().join(format!("mn8-first-parity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let mut d = MnDriver::ephemeral();
    let r = d.request(FIRST_CASE_SRC, "d_main", &caps, &workdir, Some(&lib));
    assert!(r.ok, "new native leg failed: {:?}", r.error);
    assert_eq!(r.exit_code, Some(FIRST_CASE_EXIT), "new native exit");
    assert_eq!(r.stderr, FIRST_CASE_STDERR, "new native marks");
    let _ = std::fs::remove_dir_all(&workdir);
}

#[test]
#[cfg(windows)]
fn mir_native_first_cache_store_round_new_process_restore_and_hit() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — cache round skipped");
        return;
    }
    let root = std::env::temp_dir().join(format!("mn8-first-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let lib = trace_support_lib();
    let caps = vec![TRACE_CAP.to_string()];

    // Round 1 — miss: everything fresh, objects + manifest persisted.
    let w1 = root.join("w1");
    let mut d = MnDriver::with_store(&root, "mn8-first").expect("store");
    let r1 = d.request(FIRST_CASE_SRC, "d_main", &caps, &w1, Some(&lib));
    assert!(r1.ok, "miss round failed: {:?}", r1.error);
    assert_eq!(r1.exit_code, Some(FIRST_CASE_EXIT));
    assert_eq!(r1.stderr, FIRST_CASE_STDERR);
    assert_eq!(*r1.obs.counters.get("codegen_emit_units").unwrap_or(&0), 2);
    assert_eq!(
        *r1.obs.counters.get("coff_objects_persisted").unwrap_or(&0),
        2
    );
    assert_eq!(
        *r1.obs
            .counters
            .get("link_manifests_persisted")
            .unwrap_or(&0),
        1
    );
    assert_eq!(*r1.obs.counters.get("cache_hit_coff").unwrap_or(&0), 0);
    let manifest_id = r1
        .manifest_artifact_id
        .clone()
        .expect("manifest artifact id echoed");

    // Round 2 — NEW PROCESS restore-run from the store: the worker never
    // lowers (structural codegen_emit_units=0), still really links+runs.
    let w2 = root.join("w2");
    let worker = std::path::PathBuf::from(env!("CARGO_BIN_EXE_mn-worker"));
    let out = std::process::Command::new(&worker)
        .arg("restore-run")
        .arg("--store")
        .arg(&root)
        .arg("--manifest-id")
        .arg(&manifest_id)
        .arg("--workdir")
        .arg(&w2)
        .arg("--tracelib")
        .arg(&lib)
        .output()
        .expect("spawn mn-worker");
    assert!(
        out.status.success(),
        "worker restore-run failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let line = std::str::from_utf8(&out.stdout)
        .expect("worker stdout is UTF-8")
        .lines()
        .find(|l| l.starts_with('{'))
        .expect("worker result line");
    assert!(line.contains("\"ok\":true"), "worker line: {line}");
    assert!(
        line.contains("\"exit_code\":Some(0)"),
        "worker must observe exit 0: {line}"
    );
    assert!(
        line.contains("\"stderr_hex\":\"616162\""),
        "worker must observe stderr bytes aab: {line}"
    );
    assert!(
        line.contains("\"units_admitted\":2"),
        "worker admitted both units: {line}"
    );
    assert!(
        line.contains("\"codegen_emit_units\":0"),
        "worker has no lowering path at all: {line}"
    );

    // Round 3 — in-process hit: codegen skipped (counter-control seam on),
    // link and run still happen, results identical.
    d.forbid_codegen = true;
    let w3 = root.join("w3");
    let r3 = d.request(FIRST_CASE_SRC, "d_main", &caps, &w3, Some(&lib));
    assert!(r3.ok, "hit round failed: {:?}", r3.error);
    assert_eq!(r3.exit_code, Some(FIRST_CASE_EXIT), "hit still runs the PE");
    assert_eq!(r3.stderr, FIRST_CASE_STDERR);
    assert_eq!(
        *r3.obs.counters.get("codegen_emit_units").unwrap_or(&0),
        0,
        "a hit compiles nothing (and forbid_codegen proves it)"
    );
    assert_eq!(*r3.obs.counters.get("cache_hit_coff").unwrap_or(&0), 2);
    assert!(
        *r3.obs.counters.get("link_invocations").unwrap_or(&0) >= 1
            && *r3.obs.counters.get("run_invocations").unwrap_or(&0) >= 1,
        "a hit STILL links and runs (never a cached PASS)"
    );
    let _ = std::fs::remove_dir_all(&root);
}
