//! T-01 minimal vertical: real Checked HIR → MIR → ABC → Rust AV against
//! an INDEPENDENT expected value, plus the landed native reference and the
//! new-process store restore (PLAN-798 §5.1).

use auto_av_probe::driver::Driver;

/// The frozen two-function arithmetic/branch vertical source.
pub const VERTICAL_SRC: &str = "fn score(a int, b int) int {\n    if a > b {\n        return a - b\n    }\n    return b - a\n}\n\nfn main() int {\n    let x = score(17, 29)\n    let y = score(40, 13)\n    if x + y != 39 {\n        return 1\n    }\n    return 0\n}\n";
pub const VERTICAL_EXPECTED: i32 = 0;

#[test]
fn t01_vertical_ephemeral_pipeline_matches_independent_expected() {
    let mut d = Driver::ephemeral();
    let r = d.request(VERTICAL_SRC, "d_main", &[]);
    assert!(
        r.diag.is_none(),
        "pipeline rejected the vertical: {:?}",
        r.diag
    );
    let v = r.value_i32().expect("success value");
    assert_eq!(
        v, VERTICAL_EXPECTED,
        "AV result must match the independent expected"
    );
    // The ephemeral driver still exercised the full lower/emit/verify path:
    // ONE real whole-module lowering call, two ABC units emitted, nothing
    // persisted (no store attached).
    assert_eq!(
        *r.counters.get("mir_lower_modules").unwrap_or(&0),
        1,
        "one real lowering call"
    );
    assert_eq!(
        *r.counters.get("mir_units_persisted").unwrap_or(&0),
        0,
        "no store: nothing persisted"
    );
    assert_eq!(
        *r.counters.get("abc_units_emitted").unwrap_or(&0),
        2,
        "two ABC units emitted"
    );
    assert!(
        *r.counters.get("av_execute").unwrap_or(&0) >= 1,
        "AV executed"
    );
}

#[test]
fn t01_vertical_store_round_and_hit() {
    let root = std::env::temp_dir().join(format!("av798-t01-round-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = Driver::with_store(&root, "t01-round").expect("store");
    let r1 = d.request(VERTICAL_SRC, "d_main", &[]);
    assert!(r1.ok(), "first (miss) request: {:?}", r1.diag);
    assert_eq!(r1.value_i32(), Some(VERTICAL_EXPECTED));
    assert_eq!(
        *r1.counters.get("cache_hit_abc").unwrap_or(&0),
        0,
        "fresh = all misses"
    );
    let r2 = d.request(VERTICAL_SRC, "d_main", &[]);
    assert!(r2.ok(), "second (hit) request: {:?}", r2.diag);
    assert_eq!(r2.value_i32(), Some(VERTICAL_EXPECTED));
    // Hit contract: no mir_build / abc_emit, AV still executed. (The
    // counters are absent when zero units missed — absence means zero.)
    assert_eq!(
        *r2.counters.get("mir_build").unwrap_or(&0),
        0,
        "hit skips mir lowering"
    );
    assert_eq!(
        *r2.counters.get("abc_emit").unwrap_or(&0),
        0,
        "hit skips abc emission"
    );
    assert!(
        *r2.counters.get("av_execute").unwrap_or(&0) > 0,
        "a hit STILL executes AV (never a cached PASS)"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
#[cfg(windows)]
fn t01_vertical_native_cross_check() {
    use auto_av_probe::native_reference as nr;
    let compiled = auto_ac_prototype::source::compile_scalar_source(VERTICAL_SRC, "t01.at", &[])
        .expect("frontend");
    let root = auto_ac_prototype::link::find_rust_lld().ok();
    if root.is_none() {
        eprintln!("infra-blocked: rust-lld unavailable — native cross-check skipped");
        return;
    }
    let workdir = std::env::temp_dir().join(format!("av798-t01-native-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let run =
        nr::build_and_run(&compiled.checked, "d_main", &workdir, None).expect("native build+run");
    assert_eq!(
        run.exit_code, VERTICAL_EXPECTED,
        "native exit must match the independent expected"
    );
    let _ = std::fs::remove_dir_all(&workdir);
}
