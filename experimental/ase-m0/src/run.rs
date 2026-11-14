//! One-shot headless app execution over the public ACE packages chain
//! (PLAN-1021 AC-03/04/05): strict resolve -> contained source read ->
//! `compile_package_product` -> `PackageAvTarget::cold` -> `run_value`.
//! Each call is a fresh cold execution; there is no warm body-edit path,
//! no persistent session and no cross-run cache claim.

use crate::error::{AseError, Kind, Phase};
use crate::registry::{Registry, RegistryApp};
use crate::source::read_sources;
use crate::uri::parse_app_uri;
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::Value;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_query::modules::packages::compile_package_product;
use std::path::PathBuf;

pub struct RunOptions {
    /// Requested version; `None` selects the registered version. A
    /// mismatching version is a named refusal before any dispatch.
    pub version: Option<String>,
    /// Explicit front budget; `None` = `FrontBudget::WORK_OWNED_64MIB`
    /// (the public work-owned tier). Smaller budgets surface as resource
    /// refusals, never as syntax errors or success.
    pub budget: Option<FrontBudget>,
    /// AV managed-store directory for this run (created if absent).
    pub store_dir: PathBuf,
}

impl RunOptions {
    pub fn new(store_dir: PathBuf) -> Self {
        RunOptions {
            version: None,
            budget: None,
            store_dir,
        }
    }
}

/// The observed result of one headless run, including the identity of the
/// sources actually consumed (module/file/sha256) and the package input
/// hash from the public chain — a stub returning a constant cannot fake
/// these.
#[derive(Clone, Debug)]
pub struct RunReport {
    pub identity: String,
    pub version: String,
    pub package: String,
    pub entry: String,
    /// (module, file, sha256_16) per consumed source.
    pub sources: Vec<(String, String, String)>,
    /// `BoundPackages::input_hash` of the consumed closure.
    pub input_hash: String,
    /// Number of units lowered into the AV store.
    pub lowered: u64,
    /// The i32 value returned by the entry.
    pub value: i64,
    /// ACE-boundary dispatches for this run (a successful run is exactly 1).
    pub dispatches: u64,
}

fn authorize(app: &RegistryApp, requested: Option<&str>) -> Result<(), AseError> {
    if !app.capabilities.is_empty() {
        return Err(AseError::new(
            Phase::Resolve,
            Kind::Authz,
            format!(
                "identity {} declares unsupported capabilities {:?}",
                app.identity, app.capabilities
            ),
        ));
    }
    if let Some(requested) = requested {
        if requested != app.version {
            return Err(AseError::new(
                Phase::Resolve,
                Kind::Authz,
                format!(
                    "version {requested:?} is not registered for {} (registered: {})",
                    app.identity, app.version
                ),
            ));
        }
    }
    Ok(())
}

/// Summarize package diagnostics; `source.resource` codes keep their
/// resource identity instead of being flattened into compile errors.
fn diagnostics_error(diags: &[ace_core::source::packages::PackageDiagnostic]) -> AseError {
    let summary: Vec<String> = diags
        .iter()
        .map(|d| {
            format!(
                "{}: {} {}",
                d.module, d.diagnostic.code, d.diagnostic.message
            )
        })
        .collect();
    let kind = if diags.iter().any(|d| d.diagnostic.code == "source.resource") {
        Kind::Resource
    } else {
        Kind::Compile
    };
    AseError::new(Phase::Compile, kind, summary.join("; ")).with_dispatches(1)
}

pub fn run_app(registry: &Registry, uri: &str, opts: &RunOptions) -> Result<RunReport, AseError> {
    let parsed = parse_app_uri(uri)?;
    let app = registry.lookup(&parsed.identity).ok_or_else(|| {
        AseError::new(
            Phase::Resolve,
            Kind::Authz,
            format!(
                "identity {:?} is not registered (supported: the two registered demo identities)",
                parsed.identity
            ),
        )
    })?;
    authorize(app, opts.version.as_deref())?;
    let sources = read_sources(registry, app)?;
    let inputs: Vec<PackageModuleInput> = sources
        .iter()
        .map(|s| PackageModuleInput {
            package: app.package.clone(),
            version: app.version.clone(),
            id: s.module.clone(),
            source: s.source.clone(),
            provider: None,
        })
        .collect();
    let entry = format!("{}::{}", app.package, app.entry);
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let budget = opts.budget.unwrap_or(FrontBudget::WORK_OWNED_64MIB);
    // The one ACE-boundary dispatch of this run.
    let product = compile_package_product(&inputs, &entry, &caps, budget)
        .map_err(|ds| diagnostics_error(&ds))?;
    if let Err(e) = std::fs::create_dir_all(&opts.store_dir) {
        return Err(AseError::new(
            Phase::Link,
            Kind::Resource,
            format!("create store {}: {e}", opts.store_dir.display()),
        )
        .with_dispatches(1));
    }
    let av = PackageAvTarget::cold(&opts.store_dir, &product).map_err(|e| {
        AseError::new(
            Phase::Link,
            Kind::Link,
            format!("cold link over the checked product: {e}"),
        )
        .with_dispatches(1)
    })?;
    let value = av.run_value().map_err(|e| {
        AseError::new(
            Phase::Execute,
            Kind::Execute,
            format!("entry execution failed: {e}"),
        )
        .with_dispatches(1)
    })?;
    let value = match value {
        Value::I32(n) => n as i64,
        other => {
            return Err(AseError::new(
                Phase::Execute,
                Kind::Execute,
                format!("entry must return i32, got {other:?}"),
            )
            .with_dispatches(1))
        }
    };
    let lowered = av.lowered;
    let input_hash = av.source.clone();
    Ok(RunReport {
        identity: app.identity.clone(),
        version: app.version.clone(),
        package: app.package.clone(),
        entry,
        sources: sources
            .iter()
            .map(|s| (s.module.clone(), s.file.clone(), s.sha256_16.clone()))
            .collect(),
        input_hash,
        lowered,
        value,
        dispatches: 1,
    })
}
