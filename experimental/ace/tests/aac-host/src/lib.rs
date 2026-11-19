//! PLAN-1045 AAC hosted client conformance lane.
//!
//! The Auto host client lives outside the frozen `std` package
//! (`experimental/ace/aac/host/*.at`): it is the AAC application-layer
//! hosted-demand client (CS-201/202/203 of `autolang.aac-baseline/1`),
//! consuming the frozen `aac-support` 1.0.0 digest module (PLAN-1056).
//! Sources are embedded with `include_str!`, so identities ARE the
//! repository file contents.
//!
//! Lane: the PLAN-972 assembled `packages-aggregates-ct` lane
//! (`assemble_stdlib_aggregates_inputs` + `compile_assembled_aggregates_inputs`),
//! budget `FrontBudget::WORK_OWNED_64MIB` (the 1044 frozen resource
//! profile). Legs:
//!   * AV — `PackageAvTarget` under the recorded 64M-step harness budget;
//!     the full string observables are compared byte-exact;
//!   * native AC (mn) — `MnDriver::build_collections_checked` lowers the
//!     same checked product through shared MIR to per-function COFF, links
//!     a real PE and runs it; the exe's exit code is the client's FNV-1a
//!     reduction over its case reports, which this crate recomputes over
//!     INDEPENDENTLY derived expected strings (never the client's own
//!     output).
//!
//! All expected values here are independent oracles (literal V-04 vectors,
//! Rust `Ord` byte-lex sort, the `sha2`/`blake3` crates, a mirrored FNV-1a
//! reduction) — never copies of the Auto implementation's output.

use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_aggregates_inputs, compile_assembled_aggregates_inputs, PackageProduct,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

pub const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;

/// The frozen aac-support digest module (PLAN-1056, package 1.0.0).
pub const DIGEST_SOURCE: &str = include_str!("../../../aac/support/digest.at");
/// The PLAN-1045 hosted file/path/process client (T-03, sys set).
pub const HOST_FILES_SOURCE: &str = include_str!("../../../aac/host/host_files.at");
/// The PLAN-1045 failure & publish client (T-04, sys set).
pub const HOST_PUBLISH_SOURCE: &str = include_str!("../../../aac/host/host_publish.at");
/// The PLAN-1045 host client (case reports + shared reduction).
pub const HOST_CLIENT_SOURCE: &str = include_str!("../../../aac/host/host_client.at");
/// The native-entry combine main (exit code reduction).
pub const HOST_MAIN_SOURCE: &str = include_str!("../../../aac/host/host_main.at");

/// The AV driver file set: digest + host_client (the caller-generated
/// `main.at` selects one case function to observe).
pub fn host_av_modules() -> Vec<(String, String)> {
    vec![
        ("digest.at".to_string(), DIGEST_SOURCE.to_string()),
        ("host_client.at".to_string(), HOST_CLIENT_SOURCE.to_string()),
    ]
}

/// The native file set: digest + host_client + the combine main (assembled
/// as module id `main` so the `app::main` entry resolves), entry `app::main`.
pub fn host_native_modules() -> Vec<(String, String)> {
    vec![
        ("digest.at".to_string(), DIGEST_SOURCE.to_string()),
        ("host_client.at".to_string(), HOST_CLIENT_SOURCE.to_string()),
        ("main.at".to_string(), HOST_MAIN_SOURCE.to_string()),
    ]
}

static NEXT: AtomicU64 = AtomicU64::new(0);

/// Scratch root: the single plan-artifacts home when set
/// (`AUTO_PLAN_ARTIFACTS`), else the OS temp dir.
pub fn scratch_root() -> PathBuf {
    std::env::var_os("AUTO_PLAN_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

pub fn scratch(tag: &str) -> PathBuf {
    let safe: String = tag
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let d = scratch_root().join(format!(
        "aac1045-{safe}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Assemble + compile an explicit file closure on the aggregates lane.
pub fn product_from_files(
    tag: &str,
    files: &[(String, String)],
    entry: &str,
) -> Result<PackageProduct, String> {
    let mut ctx =
        PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = assemble_stdlib_aggregates_inputs(files, entry, &mut ctx)
        .map_err(|e| format!("{tag} assemble: {e}"))?;
    compile_assembled_aggregates_inputs(&inputs, entry, BUDGET, &ctx).map_err(|ds| {
        format!(
            "{tag} compile: {} {} {} @{}..{}",
            ds[0].module,
            ds[0].diagnostic.code,
            ds[0].diagnostic.message,
            ds[0].diagnostic.span.start,
            ds[0].diagnostic.span.end
        )
    })
}

/// Compile + run one generated driver main (`fn main() Str { return
/// <case>() }`) against the host AV module set, returning the observed
/// string bytes (byte-exact comparison, not a checksum proxy).
pub fn run_case(tag: &str, case_use: &str, call: &str) -> Result<Vec<u8>, String> {
    let main_src = format!("use {case_use}\nfn main() Str {{ return {call} }}");
    let mut files = host_av_modules();
    files.push(("main.at".to_string(), main_src));
    let product = product_from_files(tag, &files, "app::main")?;
    let av =
        ace_av::modules::packages::PackageAvTarget::cold(&scratch(&format!("av-{tag}")), &product)
            .map_err(|e| format!("av cold {tag}: {e}"))?;
    let limits = ace_av_core::ExecLimits {
        max_steps: AV_MAX_STEPS,
        wall: std::time::Duration::from_secs(AV_WALL_SECS),
        ..ace_av_core::ExecLimits::default()
    };
    let (out, _trace) =
        ace_av_core::execute_program(&av.current, &limits, &ace_av_core::CancelFlag::default());
    match out {
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::Str(h),
            ..
        } => Ok(h.bytes().to_vec()),
        other => Err(format!("av run {tag}: {other:?}")),
    }
}

/// The AV harness budget (mirrors the aac-support lane's recorded budget;
/// a harness budget, not a change to any frozen 1044 resource contract).
pub const AV_MAX_STEPS: u64 = 64_000_000;
pub const AV_WALL_SECS: u64 = 300;

// ---------------------------------------------------------------------------
// T-03: the sys lane (aggregates + sys set) with the REAL Win32 backend
// ---------------------------------------------------------------------------

/// The full sys grant vector (the sys_d01_d06_1064 shape — all families).
pub const SYS_GRANTS: &[&str] = &[
    "platform.io.stdout",
    "platform.io.stderr",
    "platform.io.stdin",
    "platform.io.write",
    "platform.mem",
    "platform.file",
    "platform.file.path",
    "platform.env",
    "platform.clock",
    "platform.entropy",
    "platform.process",
];

/// Compile + run one generated driver main against the hosted sys set on
/// the AV executor driving the REAL Win32PlatformIo backend (real files,
/// real child processes), returning the observed string bytes.
pub fn run_sys_case(tag: &str, main_src: &str) -> Result<Vec<u8>, String> {
    let files = vec![
        ("digest.at".to_string(), DIGEST_SOURCE.to_string()),
        ("host_client.at".to_string(), HOST_CLIENT_SOURCE.to_string()),
        ("host_files.at".to_string(), HOST_FILES_SOURCE.to_string()),
        (
            "host_publish.at".to_string(),
            HOST_PUBLISH_SOURCE.to_string(),
        ),
        ("main.at".to_string(), main_src.to_string()),
    ];
    let product = sys_product(tag, &files)?;
    let grants: Vec<String> = SYS_GRANTS.iter().map(|s| s.to_string()).collect();
    let target =
        ace_av::modules::packages::PackageAvTarget::cold(&scratch(&format!("sys-{tag}")), &product)
            .map_err(|e| format!("av cold {tag}: {e}"))?;
    let mut backend = ace_av_core::platform_io::Win32PlatformIo::default();
    let (out, _stats) = target
        .run_platform_value(&grants, &mut backend)
        .map_err(|e| format!("av exec {tag}: {e}"))?;
    match out {
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::Str(h),
            ..
        } => Ok(h.bytes().to_vec()),
        other => Err(format!("sys run {tag}: {other:?}")),
    }
}

/// Assemble + compile on the sys lane (aggregates + sys set, Windows).
pub fn sys_product(tag: &str, files: &[(String, String)]) -> Result<PackageProduct, String> {
    let mut ctx =
        PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = ace_source_query::modules::packages::assemble_stdlib_sys_inputs(
        files,
        "app::main",
        &mut ctx,
    )
    .map_err(|e| format!("{tag} assemble sys: {e}"))?;
    compile_assembled_aggregates_inputs(&inputs, "app::main", BUDGET, &ctx).map_err(|ds| {
        format!(
            "{tag} compile sys: {} {} {} @{}..{}",
            ds[0].module,
            ds[0].diagnostic.code,
            ds[0].diagnostic.message,
            ds[0].diagnostic.span.start,
            ds[0].diagnostic.span.end
        )
    })
}

/// The native AC (mn) leg: lower the SAME checked product through shared
/// MIR, emit per-function COFF, link a real PE, run it once; the exit code
/// is the client's reduction of its own case reports.
#[cfg(all(windows, target_arch = "x86_64"))]
pub fn run_native_combined(tag: &str) -> Result<(i32, std::path::PathBuf), String> {
    let product = product_from_files(tag, &host_native_modules(), "app::main")?;
    let dir = scratch(&format!("mn-{tag}"));
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_collections_checked(
            &product.compiled.checked,
            "d_main",
            &dir,
            &product.bound.input_hash,
            1045,
        )
        .map_err(|e| format!("mn build {tag}: {e:?}"))?;
    let out = std::process::Command::new(&built.exe_path)
        .current_dir(&dir)
        .output()
        .map_err(|e| format!("run {tag} exe {}: {e}", built.exe_path.display()))?;
    // The exit code IS the observable for this leg (the reduction of the
    // case reports); stdout/stderr are only diagnostic payload.
    let _ = out;
    Ok((out.status.code().unwrap_or(-1), built.exe_path))
}

// ---------------------------------------------------------------------------
// Independent oracles (never the client's output)
// ---------------------------------------------------------------------------

/// Mirror of the client's wrap32 FNV-1a (u64 register, mod 2^32 per step).
pub fn fnv32(s: &[u8]) -> u64 {
    let mut h: u64 = 2166136261;
    for &b in s {
        h = ((h ^ b as u64).wrapping_mul(16777619)) % (1u64 << 32);
    }
    h
}

/// Mirror of `host_main`'s combine: c = wrap32(c*31 + fnv(report)) over the
/// fixed case order, reduced mod 2^31 for the i32 exit code.
pub fn combine(reports: &[&str]) -> i32 {
    let mut c: u64 = 0;
    for r in reports {
        c = (c.wrapping_mul(31).wrapping_add(fnv32(r.as_bytes()))) % (1u64 << 32);
    }
    (c % 2147483648) as i32
}

/// The INDEPENDENTLY derived expected case reports, in the exact
/// `host_main` fold order (bytes_span, map, intern, sort, int, budget,
/// digest).
pub fn expected_reports() -> Vec<String> {
    let mut v = Vec::with_capacity(7);
    // V-04 bytes/spans: "a\0b" = 3 scalars, 3 bytes [0x61,0x00,0x62] (decimal
    // 97,0,98 — the report alphabet is decimal); "α" bytes [206,177], span
    // [0,2); "a中b" = 5 bytes, 3 scalars, [97,228,184,173,98], span of
    // 中 = [1,4).
    v.push("3,3,97,0,98|206,177,0,2|5,3,97,228,184,173,98,1,4".to_string());
    // V-04 map: put flags new,new,replace; 2 survivors, key-sorted.
    v.push("001|2,a=3,b=2".to_string());
    // V-01 intern: per-input index, then distinct count + arena bytes.
    v.push("0,1,2,3,1,0,4,3,5|6,11".to_string());
    // Deterministic byte-lex sort — Rust `Ord` over the same inputs.
    let mut items = vec!["b", "a", "α", "a\0b", "", "中", "ab", "b"];
    items.sort();
    v.push(items.join(","));
    // Precise ints: fnv32("a\0b") mod 1e9, parse round trips, overflow class.
    let h = (fnv32(b"a\0b") % 1_000_000_000) as i32;
    v.push(format!("{h}|2147483647,-2147483648,ref1|ovf1,12"));
    // Budget: full walk ok, 3-step budget refused by name.
    v.push("ok:7,refused".to_string());
    // Digests via the independent crates over the same literals.
    use sha2::{Digest, Sha256};
    let sha = |b: &[u8]| {
        let mut h = Sha256::new();
        h.update(b);
        h.finalize()
    };
    let a = hex_lower(&sha(b"abc"));
    let b = hex_lower(&sha(b"a\0b"));
    let c = hex_lower(blake3::hash("α".as_bytes()).as_bytes());
    v.push(format!("{a},{b},{c}"));
    v
}

fn hex_lower(bs: &[u8]) -> String {
    bs.iter().map(|b| format!("{b:02x}")).collect()
}
