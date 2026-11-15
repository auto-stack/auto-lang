//! PLAN-972 T-10 / V-10: the core/alloc layering over the assembled std
//! set, proven on the AC/native end where the runtime's own observation
//! counters are the oracle.
//! - CORE layer: a core-only program (integer + double math, char
//!   predicates — no Str/List construction) runs with allocation == 0
//!   (observation counters; no heap, no host dispatch).
//! - ALLOC layer: an allocating program (Str construction through the
//!   std algorithms) runs with balanced frees (live = 0) under the
//!   1039/1040-backed arena; a configured byte budget that fits succeeds.
//! - Failure discipline: fail-the-Nth-allocation injection traps (never
//!   a silent wrong value) with every SUCCESSFUL allocation still freed
//!   (frees_ok == allocs, live = 0 — no leak on the failure path); an
//!   under-sized byte budget traps the same way.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};
use ace_native::collections_lower::WrapperKnobs;
use ace_source_query::modules::packages::{
    assemble_stdlib_aggregates_inputs, compile_assembled_aggregates_inputs,
};

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/asm972/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Build + run on mn with the given wrapper knobs; returns (exit, stderr).
fn mn_run(tag: &str, main_src: &str, knobs: WrapperKnobs) -> (i32, String) {
    let mut ctx =
        PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = assemble_stdlib_aggregates_inputs(
        &[("main.at".to_string(), main_src.to_string())],
        "app::main",
        &mut ctx,
    )
    .unwrap();
    let product = compile_assembled_aggregates_inputs(
        &inputs,
        "app::main",
        FrontBudget::WORK_OWNED_64MIB,
        &ctx,
    )
    .unwrap_or_else(|ds| panic!("{tag} compile: {ds:?}"));
    let dir = scratch(&format!("v10-{tag}"));
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    driver.resource_knobs = knobs;
    let built = driver
        .build_collections_checked(
            &product.compiled.checked,
            "d_main",
            &dir,
            &product.bound.input_hash,
            972,
        )
        .unwrap_or_else(|e| panic!("{tag} mn build: {e:?}"));
    let out = std::process::Command::new(&built.exe_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// The SUMMARY counters line fields: attempts, allocs, frees, live.
fn counter(stderr: &str, key: &str) -> i64 {
    for line in stderr.lines() {
        if line.starts_with("SUMMARY") {
            for field in line.split_whitespace() {
                if let Some(v) = field.strip_prefix(&format!("{key}=")) {
                    return v.parse().unwrap_or(-1);
                }
            }
        }
    }
    -1
}

/// Core-only program: integer math, the pure-Auto double family, char
/// predicates — no Str/List construction anywhere (char and numeric
/// literals are immediate constants, not arena nodes).
const CORE_MAIN: &str = r#"
use std.math.abs
use std.math.min
use std.math.max
use std.math.square
use std.math.cube
use std.math.abs_f
use std.math.sqrt
use std.math.powf
use std.math.sin
use std.math.cos
use std.char.is_digit
use std.char.is_alpha_ascii
use std.char.is_alphanum_ascii
use std.char.is_ident_ascii
use std.char.is_whitespace
use std.char.to_upper_ascii
use std.char.to_lower_ascii
fn main() int {
    var code = 0
    if (abs(-9223372036854775807i64) == 9223372036854775807i64) {
        code = code + 1000000
    }
    if (min(3i64, 5i64) == 3i64) {
        code = code + 100000
    }
    if (max(3i64, 5i64) == 5i64) {
        code = code + 10000
    }
    if (square(7) == 49) {
        code = code + 1000
    }
    if (cube(3) == 27) {
        code = code + 100
    }
    if (abs_f(-2.5d) == 2.5d) {
        code = code + 10
    }
    let r2 = sqrt(2.0d)
    if (r2 > 1.41421356237d) {
        if (r2 < 1.41421356238d) {
            code = code + 1
        }
    }
    let p10 = powf(2.0d, 10.0d)
    if (p10 > 1023.9999d) {
        if (p10 < 1024.0001d) {
            code = code + 1
        }
    }
    let sn1 = sin(1.0d)
    if (sn1 > 0.8414709847d) {
        if (sn1 < 0.8414709849d) {
            code = code + 1
        }
    }
    let cs1 = cos(1.0d)
    if (cs1 > 0.5403023057d) {
        if (cs1 < 0.5403023059d) {
            code = code + 1
        }
    }
    if (is_digit('7')) {
        code = code + 1
    }
    if (!is_alpha_ascii('中')) {
        code = code + 1
    }
    if (is_alphanum_ascii('9')) {
        code = code + 1
    }
    if (is_ident_ascii('_')) {
        code = code + 1
    }
    if (is_whitespace(' ')) {
        code = code + 1
    }
    if (to_upper_ascii(97) == 65) {
        code = code + 1
    }
    if (to_lower_ascii(65) == 97) {
        code = code + 1
    }
    return code
}
"#;

/// Alloc-layer program: Str construction through the std algorithms
/// (upper_ascii builds a fresh Str; to_hex walks + builds; repeat builds).
const ALLOC_MAIN: &str = r#"
use std.text.upper_ascii
use std.text.repeat
use std.bytes.to_hex
use std.text.equal
fn main() int {
    var code = 0
    let u = upper_ascii("aB9")
    if (equal(u, "AB9")) {
        code = code + 100
    }
    let r = repeat("ab", 3)
    if (r.len_bytes() == 6) {
        code = code + 10
    }
    let h = to_hex("AB")
    if (equal(h, "4142")) {
        code = code + 1
    }
    return code
}
"#;

#[test]
fn v10_core_alloc_layers_mn() {
    // ---- CORE: allocation == 0, full score, no heap/host dispatch. ----
    let (exit, stderr) = mn_run(
        "core",
        CORE_MAIN,
        WrapperKnobs {
            observe: true,
            ..Default::default()
        },
    );
    assert_eq!(exit, 1111121, "core exit (all checks): {stderr}");
    assert_eq!(
        counter(&stderr, "allocs"),
        0,
        "core layer allocates nothing"
    );
    assert_eq!(counter(&stderr, "attempts"), 0, "no allocation attempts");
    assert_eq!(counter(&stderr, "live"), 0);

    // ---- ALLOC: balanced frees on the normal path, generous budget OK. ----
    let (exit, stderr) = mn_run(
        "alloc",
        ALLOC_MAIN,
        WrapperKnobs {
            observe: true,
            budget_bytes: Some(1 << 20),
            ..Default::default()
        },
    );
    assert_eq!(exit, 111, "alloc exit: {stderr}");
    assert!(counter(&stderr, "allocs") > 0, "the alloc layer allocates");
    assert!(frees_ok_of(&stderr) > 0, "allocations are freed");
    assert_eq!(invalid_frees_of(&stderr), 0, "no invalid frees");
    // accounting identity: allocs - frees_ok == live (nothing unaccounted)
    assert_eq!(
        counter(&stderr, "allocs") - frees_ok_of(&stderr),
        counter(&stderr, "live"),
        "allocation accounting closes"
    );

    // ---- Failure discipline: fail the 3rd allocation — trap, no leak. ----
    let (exit, stderr) = mn_run(
        "fail3",
        ALLOC_MAIN,
        WrapperKnobs {
            observe: true,
            fail_nth: Some(3),
            ..Default::default()
        },
    );
    assert_eq!(exit, 70, "injected allocation failure traps: {stderr}");
    // the injection's own evidence line (the refusal precedes the arena
    // attempt counter — attempts stays honest), then the trap is a
    // resource trap (code 22 in the observation).
    assert!(
        stderr.contains("MN-COLL-INJECT"),
        "refusal recorded: {stderr}"
    );
    assert!(
        stderr.contains("MN-COLL-TRAP code=22"),
        "resource trap: {stderr}"
    );
    // the failure path swept everything: no leak, no invalid frees, and
    // the accounting closes at live == 0.
    assert_eq!(invalid_frees_of(&stderr), 0, "no invalid frees on failure");
    assert_eq!(
        counter(&stderr, "live"),
        0,
        "failure path leaves nothing live"
    );
    assert_eq!(
        counter(&stderr, "allocs") - frees_ok_of(&stderr),
        counter(&stderr, "live"),
        "failure-path accounting closes"
    );

    // ---- Budget: an under-sized byte budget traps (configured bound). ----
    let (exit, stderr) = mn_run(
        "budget",
        ALLOC_MAIN,
        WrapperKnobs {
            observe: true,
            budget_bytes: Some(8),
            ..Default::default()
        },
    );
    assert_eq!(exit, 70, "under-sized budget traps: {stderr}");
    assert_eq!(invalid_frees_of(&stderr), 0);
}

fn invalid_frees_of(stderr: &str) -> i64 {
    for line in stderr.lines() {
        if line.starts_with("FREESTAT") {
            for field in line.split_whitespace() {
                if let Some(v) = field.strip_prefix("invalid=") {
                    return v.parse().unwrap_or(-1);
                }
            }
        }
    }
    -1
}

fn frees_ok_of(stderr: &str) -> i64 {
    for line in stderr.lines() {
        if line.starts_with("FREESTAT") {
            for field in line.split_whitespace() {
                if let Some(v) = field.strip_prefix("frees_ok=") {
                    return v.parse().unwrap_or(-1);
                }
            }
        }
    }
    -1
}
