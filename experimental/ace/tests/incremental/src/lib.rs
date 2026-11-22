//! PLAN-1052 AAC incremental driver & persistent store conformance lane.
//!
//! The Auto incremental/store/package modules live in
//! `experimental/ace/aac/store/*.at` and `experimental/ace/aac/driver/*.at`:
//! they own the query-host decisions (positive/negative read sets,
//! body-vs-signature separation), the six-axis BUILD identity keys, the
//! controlled store/restore admission and the package-closure/FFI binding.
//! They are compiled by the Gen0 (Rust-frontend) assembled
//! `packages-aggregates-ct` lane and executed on the AV executor; the Rust
//! side of this crate supplies only the lane, the AV run and the
//! INDEPENDENT byte-exact references (computed via `ace_artifacts`/`blake3`
//! from the same inputs) — never copies of the Auto output.
//!
//! Oracle discipline: every report returned by the Auto modules is compared
//! byte-exact against a Rust reference derived from the frozen contract
//! text (six-axis nested domain hash, page key framing, admission codes,
//! decision taxonomy), so any divergence in any case changes the bytes.

use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, PackageDiagnostic, ProviderChoice, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_aggregates_inputs, compile_assembled_aggregates_inputs, PackageProduct,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

pub const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;

/// The frozen aac-support digest module (PLAN-1056, package 1.0.0) — the
/// identity/checksum primitives the new modules consume.
pub const DIGEST_SOURCE: &str = include_str!("../../../aac/support/digest.at");
/// The PLAN-1052 store/restore module (T-03).
pub const STORE_SOURCE: &str = include_str!("../../../aac/store/store.at");
/// The PLAN-1052 query host & dependency graph module (T-02).
pub const QUERY_SOURCE: &str = include_str!("../../../aac/driver/query.at");
/// The PLAN-1052 build/check/publish driver module (T-04).
pub const DRIVER_SOURCE: &str = include_str!("../../../aac/driver/driver.at");
/// The PLAN-1052 package closure & FFI module (T-09).
pub const PACKAGE_SOURCE: &str = include_str!("../../../aac/driver/package.at");

/// The Auto module set (file name -> embedded source). The entry `main.at`
/// is appended per case by the run helpers.
pub fn incremental_modules() -> Vec<(String, String)> {
    vec![
        ("digest.at".to_string(), DIGEST_SOURCE.to_string()),
        ("store.at".to_string(), STORE_SOURCE.to_string()),
        ("query.at".to_string(), QUERY_SOURCE.to_string()),
        ("driver.at".to_string(), DRIVER_SOURCE.to_string()),
        ("package.at".to_string(), PACKAGE_SOURCE.to_string()),
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
        "aac1052-{safe}-{}-{}",
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
    product_from_files_diag(tag, files, entry).map_err(|ds| format!("{tag} compile: {}", describe(&ds)))
}

/// The raw diagnostics of a refused assembly (the canonical code/stage/
/// span the negative controls assert — never a folded error string).
pub fn product_from_files_diag(
    tag: &str,
    files: &[(String, String)],
    entry: &str,
) -> Result<PackageProduct, Vec<PackageDiagnostic>> {
    let mut ctx =
        PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = assemble_stdlib_aggregates_inputs(files, entry, &mut ctx)
        .map_err(|e| vec![PackageDiagnostic {
            module: entry.to_string(),
            diagnostic: ace_core::atom_text::Diagnostic::new(
                ace_core::atom_text::Stage::Bind,
                "assembly.discovery",
                e,
                ace_core::atom_text::Span { start: 0, end: 0 },
            ),
        }])?;
    compile_assembled_aggregates_inputs(&inputs, entry, BUDGET, &ctx)
}

fn describe(ds: &[PackageDiagnostic]) -> String {
    let d = &ds[0];
    format!(
        "{} {} {} @{}..{}",
        d.module,
        d.diagnostic.code,
        d.diagnostic.message,
        d.diagnostic.span.start,
        d.diagnostic.span.end
    )
}

/// Compile + run one generated driver main: `use app.<module>.<fn>` imports
/// the case function, `main` calls it unqualified (the packages lane binds
/// the three-segment path), returning the observed string bytes.
pub fn run_case(tag: &str, module: &str, func: &str, call: &str) -> Result<Vec<u8>, String> {
    let main_src = format!("use app.{module}.{func}\nfn main() Str {{ return {call} }}");
    let mut files = incremental_modules();
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

pub fn run_str(tag: &str, module: &str, func: &str, call: &str) -> Result<String, String> {
    let bytes = run_case(tag, module, func, call)?;
    String::from_utf8(bytes).map_err(|e| format!("{tag}: non-utf8 observable: {e}"))
}

/// Run a case function whose result is an I32 (the decision-code helpers),
/// returning the raw int.
pub fn run_int(tag: &str, module: &str, func: &str, call: &str) -> Result<i32, String> {
    let main_src = format!("use app.{module}.{func}\nfn main() int {{ return {call} }}");
    let mut files = incremental_modules();
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
            value: ace_av_core::contract::Value::I32(v),
            ..
        } => Ok(v),
        other => Err(format!("av run {tag}: {other:?}")),
    }
}

/// The AV harness budget (mirrors the aac-support lane's recorded budget;
/// a harness budget, not a change to any frozen 1044 resource contract).
pub const AV_MAX_STEPS: u64 = 64_000_000;
pub const AV_WALL_SECS: u64 = 300;

// ---------------------------------------------------------------------------
// Independent Rust references (written from the contract text, never from
// the Auto output)
// ---------------------------------------------------------------------------

/// Mirror of `digest.identity_hex2`: blake3 over
/// "blake3-1.5\0<family>\0" + two length-prefixed parts.
pub fn axis2(domain: &str, a: &str, b: &str) -> String {
    let mut pre = Vec::new();
    pre.extend_from_slice(b"blake3-1.5");
    pre.push(0);
    pre.extend_from_slice(domain.as_bytes());
    pre.push(0);
    for part in [a.as_bytes(), b.as_bytes()] {
        pre.extend_from_slice(&(part.len() as u64).to_le_bytes());
        pre.extend_from_slice(part);
    }
    blake3::hash(&pre).to_hex().to_string()
}

/// Mirror of `store.six_axis_key`: schema/target/stdlib/grammar/pass/bridge
/// as nested pair hashes (the same framing the Auto module composes).
pub fn six_axis_key(schema: &str, target: &str, stdlib: &str, grammar: &str, pass: &str, bridge: &str) -> String {
    let st = axis2("ac1052/axis2/v1", schema, target);
    let sg = axis2("ac1052/axis2/v1", stdlib, grammar);
    let pb = axis2("ac1052/axis2/v1", pass, bridge);
    axis2(
        "ac1052/build/v1",
        &axis2("ac1052/pair/v1", &st, &sg),
        &pb,
    )
}

/// Mirror of `store.page_key`: owner/sig/body/caps page identity.
pub fn page_key(owner: &str, sig: &str, body: &str, caps: &str) -> String {
    axis2(
        "ac1052/page/v1",
        &axis2("ac1052/owner/v1", owner, sig),
        &axis2("ac1052/body-caps/v1", body, caps),
    )
}

/// Mirror of `store.ref_key`.
pub fn ref_key(domain: &str, page: &str, artifact: &str) -> String {
    axis2(domain, page, artifact)
}

/// Mirror of `query.fact_key`.
pub fn fact_key(query_kind: &str, logical: &str, sig: &str, body: &str, caps: &str) -> String {
    axis2(
        "ac1052/query/v1",
        &axis2("ac1052/q-kind/v1", query_kind, logical),
        &axis2("ac1052/axes/v1", sig, body),
    ) + caps
}

/// Mirror of `driver.stage_key`.
pub fn stage_key(stage: &str, pass: &str, src: &str, gram: &str) -> String {
    axis2(
        "ac1052/stage/v1",
        &axis2("ac1052/pass/v1", stage, pass),
        &axis2("ac1052/inputs/v1", src, gram),
    )
}
