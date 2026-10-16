//! V-04 native trace-parity leg: the SAME source runs through AV (trace
//! events) and the landed native path (real PE + mark_a/mark_b stderr
//! bytes); both must agree with the INDEPENDENT source-order expectation
//! (PLAN-798 §5.3: native never generates its expectation from the new
//! MIR/ABC).

#[cfg(windows)]
#[test]
fn av_semantics_native_trace_parity() {
    use auto_ac_prototype::source::compile_scalar_source;
    let src = "fn side(x int) int {\n    let m = mark_b()\n    return x * m\n}\n\nfn main() int {\n    let a = mark_a()\n    let b = side(3)\n    let c = mark_a()\n    if a + b + c != 10 {\n        return 1\n    }\n    return 0\n}\n";
    let caps = vec!["hir.test.trace.v1".to_string()];

    // AV leg: source-order events + the frozen mark values (a=1, b=2).
    let mut d = auto_av_probe::driver::Driver::ephemeral();
    let r = d.request(src, "d_main", &caps);
    assert!(r.diag.is_none(), "{:?}", r.diag);
    use auto_av_core::mir::TraceEvent::*;
    assert_eq!(r.trace, vec![MarkA, MarkB, MarkA]);
    // a=1, side(3)=3*2=6, c=1 → 8 ≠ 10 → main returns 1.
    assert_eq!(r.value_i32(), Some(1));

    // Native leg: build the trace support object exactly like the landed
    // ac-core suites, link + run the real PE, compare stderr bytes.
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let support = repo.join("experimental/ac-core/test-support");
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
    let compiled = compile_scalar_source(src, "trace.at", &caps).expect("frontend");
    let workdir = std::env::temp_dir().join(format!("av798-trace-parity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let run = auto_av_probe::native_reference::build_and_run(
        &compiled.checked,
        "d_main",
        &workdir,
        Some(&lib),
    )
    .expect("native trace build+run");
    assert_eq!(run.exit_code, 1, "PE mirrors the AV value");
    assert_eq!(run.stderr, "aba", "PE mark bytes in source order");
    let _ = std::fs::remove_dir_all(&workdir);
}
