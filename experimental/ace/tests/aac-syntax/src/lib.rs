//! PLAN-1046 AAC syntax/source conformance lane.
//!
//! The Auto syntax runtime lives in `experimental/ace/aac/syntax/*.at` (and
//! the source projection / module catalog in `aac/source/*.at`): it is the
//! AAC stage driver implementing the unified grammar's tokenization, parse,
//! CST, incremental edit, positions, AST projection and module/#in catalog
//! **in Auto**, compiled by Gen0 on the assembled packages-aggregates-ct
//! lane and executed on the AV executor. Sources are embedded with
//! `include_str!`, so identities ARE the repository file contents.
//!
//! Oracle discipline (the plan's 同源验收): the Rust `ace-syntax-runtime`
//! executing the SAME versioned grammar program over the SAME frozen inputs
//! is the independent reference — canonical CST texts, decl directories,
//! diagnostics, positions and snapshot identities are compared byte-exact
//! between the two implementations, never copied from the Auto output.
//! Grammar data hexes embedded in `syntax_grammar_data.at` are re-compiled
//! from the repository grammar and verified byte-for-byte on every run.
//!
//! Budget/edit/boundary cases assert the controlled-refusal contracts
//! (resource exhaustion is its own outcome class; illegal sources are
//! refused at the syntax stage with no half-published tree).

use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_aggregates_inputs, compile_assembled_aggregates_inputs, PackageProduct,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

mod native;

pub use native::{
    native_fingerprint, run_native_case, run_native_catalog, run_native_projection, DRIVER_EXE, DRIVER_MANIFEST, NATIVE_DRIVER_SOURCE,
};

pub const SYNTAX_PROFILES: [&str; 7] = [
    "auto",
    "age-package",
    "packages-ct",
    "packages-modules",
    "text-record-instances",
    "scalar",
    "scheduler",
];

pub const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;

// The Auto module set (file name -> embedded source). `syntax_smoke.at` is
// the stage driver; permanent cases consolidate into `syntax_driver.at` as
// their tasks land.
pub const DIGEST_SOURCE: &str = include_str!("../../../aac/support/digest.at");
pub const PROGRAM_SOURCE: &str = include_str!("../../../aac/syntax/syntax_program.at");
pub const LEXER_SOURCE: &str = include_str!("../../../aac/syntax/syntax_lexer.at");
pub const CST_SOURCE: &str = include_str!("../../../aac/syntax/syntax_cst.at");
pub const PARSE_SOURCE: &str = include_str!("../../../aac/syntax/syntax_parse.at");
pub const SNAPSHOT_SOURCE: &str = include_str!("../../../aac/syntax/syntax_snapshot.at");
pub const INCREMENTAL_SOURCE: &str = include_str!("../../../aac/syntax/syntax_incremental.at");
pub const GRAMMAR_DATA_SOURCE: &str = include_str!("../../../aac/syntax/syntax_grammar_data.at");
pub const SYNTAX_FRONTEND: &str = include_str!("../../../aac/syntax/syntax_frontend.at");
const SYNTAX_FSTR: &str = include_str!("../../../aac/syntax/syntax_fstr.at");
pub const SOURCE_CATALOG: &str = include_str!("../../../aac/source/source_catalog.at");
pub const SOURCE_POSITION: &str = include_str!("../../../aac/source/source_position.at");
pub const SOURCE_AST: &str = include_str!("../../../aac/source/source_ast.at");
pub const SMOKE_SOURCE: &str = include_str!("../../../aac/syntax/syntax_smoke.at");

/// The AV file set: all syntax modules + the smoke driver.
pub fn syntax_av_modules() -> Vec<(String, String)> {
    vec![
        ("digest.at".to_string(), DIGEST_SOURCE.to_string()),
        ("syntax_program.at".to_string(), PROGRAM_SOURCE.to_string()),
        ("syntax_lexer.at".to_string(), LEXER_SOURCE.to_string()),
        ("syntax_cst.at".to_string(), CST_SOURCE.to_string()),
        ("syntax_parse.at".to_string(), PARSE_SOURCE.to_string()),
        ("syntax_snapshot.at".to_string(), SNAPSHOT_SOURCE.to_string()),
        ("syntax_incremental.at".to_string(), INCREMENTAL_SOURCE.to_string()),
        (
            "syntax_grammar_data.at".to_string(),
            GRAMMAR_DATA_SOURCE.to_string(),
        ),
        ("syntax_frontend.at".to_string(), SYNTAX_FRONTEND.to_string()),
        ("syntax_fstr.at".to_string(), SYNTAX_FSTR.to_string()),
        ("source_ast.at".to_string(), SOURCE_AST.to_string()),
        ("source_position.at".to_string(), SOURCE_POSITION.to_string()),
        ("source_catalog.at".to_string(), SOURCE_CATALOG.to_string()),
        ("syntax_smoke.at".to_string(), SMOKE_SOURCE.to_string()),
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
        "aac1046-{safe}-{}-{}",
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
/// <case>() }`) against the AV module set, returning the observed string
/// bytes (byte-exact comparison, never a checksum proxy). `case_use` is the
/// full 3-segment item path (`app.<module>.<fn>`).
pub fn run_case(tag: &str, case_use: &str, call: &str) -> Result<Vec<u8>, String> {
    let budget: u64 = std::env::var("AV_BUDGET").ok().and_then(|v| v.parse().ok()).unwrap_or(AV_MAX_STEPS);
    let main_src = format!("use {case_use}\nfn main() Str {{ return {call} }}");
    let mut files = syntax_av_modules();
    files.push(("main.at".to_string(), main_src));
    let product = product_from_files(tag, &files, "app::main")?;
    let av =
        ace_av::modules::packages::PackageAvTarget::cold(&scratch(&format!("av-{tag}")), &product)
            .map_err(|e| format!("av cold {tag}: {e}"))?;
    let limits = ace_av_core::ExecLimits {
        max_steps: budget,
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

/// The AV harness budget. The 1045 lane recorded 64M for its digest-sized
/// drivers; the syntax machine is far heavier per parse on the interpreter
/// (measured: ONE scalar-crop parse+decode run lands in the low hundreds of
/// millions of VM instructions; D-1046-02 turned out to be pure cost, not a
/// loop — a two-parse edit case needs ~2B). This is a HARNESS budget only
/// — the 1044-frozen parse resource profile (depth/work bytes) is asserted
/// inside the driver's own budget parameters, not here.
pub const AV_MAX_STEPS: u64 = 4_000_000_000;
pub const AV_WALL_SECS: u64 = 300;
