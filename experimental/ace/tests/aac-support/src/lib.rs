//! PLAN-1056 AAC exact-digest / compiler-codec support package.
//!
//! The Auto support sources live outside the frozen `std` package
//! (`experimental/ace/aac/support/*.at`, package identity `aac-support`
//! `1.0.0`): they are the AAC application-layer digest/codec surface, not a
//! public standard-library operation (the public-`std` publication and its
//! five-end coverage contract are a named later decision, see the plan
//! T-02 rationale). They are embedded with `include_str!`, so the package
//! identity IS the repository file content.
//!
//! The lane is the PLAN-972 assembled `packages-aggregates-ct` lane
//! (`assemble_stdlib_aggregates_inputs` + `compile_assembled_aggregates_inputs`):
//! the Auto sources are compiled by the Gen0 (Rust) frontend and the
//! algorithms run on the AV executor / declared consumer ends — never by
//! the legacy VM or an A2X transpile of the driver for the AV observations.

use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_aggregates_inputs, compile_assembled_aggregates_inputs, PackageProduct,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;

/// The frozen support-package sources, by module id (file stem).
pub const DIGEST_SOURCE: &str = include_str!("../../../aac/support/digest.at");
pub const CODEC_SOURCE: &str = include_str!("../../../aac/support/codec.at");
pub const ATOM_DATA_SOURCE: &str = include_str!("../../../aac/support/atom_data.at");
pub const DRIVER_SOURCE: &str = include_str!("../../../aac/support/driver.at");

/// Package identity: name `aac-support`, version 1.0.0, module ids below.
pub const SUPPORT_PACKAGE: &str = "aac-support";
pub const SUPPORT_VERSION: &str = "1.0.0";

/// `(module id, source)` pairs of the frozen support package. The app
/// assembly lane takes its package name from the entry (`app::main`), so the
/// modules ride the app package in-run; the identity axis is the file
/// content, which is what this table pins.
pub fn support_modules() -> Vec<(String, String)> {
    vec![
        ("digest.at".to_string(), DIGEST_SOURCE.to_string()),
        ("codec.at".to_string(), CODEC_SOURCE.to_string()),
        ("atom_data.at".to_string(), ATOM_DATA_SOURCE.to_string()),
        ("driver.at".to_string(), DRIVER_SOURCE.to_string()),
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
    // Case names may carry `:` (the driver's `family:case` spelling), which is
    // not a legal Windows path character — sanitise the scratch directory name
    // only; identity is the embedded source, never this path.
    let safe: String = tag
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let d = scratch_root().join(format!(
        "aac1056-{safe}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Assemble + compile one driver program: `main.at` plus the frozen support
/// modules, entry `app::main`, on the assembled aggregates lane.
pub fn support_product(tag: &str, main_src: &str) -> Result<PackageProduct, String> {
    let mut files = support_modules();
    files.push(("main.at".to_string(), main_src.to_string()));
    product_from_files(tag, &files, "app::main")
}

/// Assemble + compile an explicit file closure on the same lane.
pub fn product_from_files(
    tag: &str,
    files: &[(String, String)],
    entry: &str,
) -> Result<PackageProduct, String> {
    let mut ctx = PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
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

/// The AV driver budget for the support-package evidence. The Default
/// (`MAX_AV_INSTR_STEPS` = 1M) is small for a 1 KiB pure-Auto BLAKE3 (the
/// digest walk is ~1M steps/KiB, see the plan's T-03 note); the driver runs
/// under this explicit, recorded budget instead, so every vector is executed
/// rather than skipped. This is a harness budget, not a change to any frozen
/// 1044 resource contract.
pub const AV_MAX_STEPS: u64 = 64_000_000;
pub const AV_WALL_SECS: u64 = 300;

/// Run a support product on the AV end and return its observed value.
pub fn run_av(tag: &str, product: &PackageProduct) -> Result<ace_av_core::contract::Value, String> {
    let av = ace_av::modules::packages::PackageAvTarget::cold(&scratch(&format!("av-{tag}")), product)
        .map_err(|e| format!("av cold {tag}: {e}"))?;
    let limits = ace_av_core::ExecLimits {
        max_steps: AV_MAX_STEPS,
        wall: std::time::Duration::from_secs(AV_WALL_SECS),
        ..ace_av_core::ExecLimits::default()
    };
    let (out, _trace) = ace_av_core::execute_program(
        &av.current,
        &limits,
        &ace_av_core::CancelFlag::default(),
    );
    match out {
        ace_av_core::contract::ExecOutcome::Success { value, .. } => Ok(value),
        other => Err(format!("av run {tag}: {other:?}")),
    }
}

/// Convenience: compile + run one driver program, returning the raw value.
pub fn run_program(tag: &str, main_src: &str) -> Result<ace_av_core::contract::Value, String> {
    let p = support_product(tag, main_src)?;
    run_av(tag, &p)
}

/// The observed string bytes of a `Value::Str` result, or an error naming the
/// actual variant (the driver programs return owned `Str` hex output so the
/// comparison is byte-exact, not a checksum proxy).
pub fn str_bytes(v: &ace_av_core::contract::Value) -> Result<Vec<u8>, String> {
    match v {
        ace_av_core::contract::Value::Str(h) => Ok(h.bytes().to_vec()),
        other => Err(format!("expected Str, got {other:?}")),
    }
}

/// Directory-existence probe used by evidence lines (never creates).
pub fn is_dir(p: &Path) -> bool {
    p.is_dir()
}



