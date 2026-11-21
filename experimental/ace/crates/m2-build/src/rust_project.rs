//! Project integration of the 967 provider build and the 1093 AV front door.

use std::fs::{self, File};
use std::path::Path;

use ace_a2x_core::sha256::sha256_hex;
use ace_abi::provider_build_cache::BuildIdentity;
use ace_abi::rust_provider::{
    build_locked_cargo_provider, build_source, describe_rust_provider, open_session_from_build,
    provider_deps, rustc_identity, RustProviderBuild,
};
use ace_av::console_host::ConsoleCapture;
use ace_av::foreign_source::{compile_foreign_program, execute_foreign_program};
use ace_av_core::av::CancelFlag;
use ace_av_core::contract::{ExecOutcome, Value};

use crate::driver::{lane_entry, ProjectInputs, RunObservation, RustProjectInputs};
use crate::M2Reject;

pub(crate) struct PreparedProvider {
    build: RustProviderBuild,
    pub identity: BuildIdentity,
    pub artifact_sha256: String,
    pub rebuilt: bool,
    // Prevent a concurrent repair from replacing the DLL during this build.
    _lease: File,
}

fn prepare_error(error: impl std::fmt::Display) -> M2Reject {
    M2Reject::new("m2build.rust-provider", error.to_string())
}

pub(crate) fn prepare(
    scratch: &Path,
    inputs: &ProjectInputs,
    provider: &RustProjectInputs,
) -> Result<PreparedProvider, M2Reject> {
    if !inputs
        .caps
        .iter()
        .any(|cap| cap == ace_foreign_contract::manual::SOURCE_CAP)
    {
        return Err(M2Reject::new(
            "m2build.foreign-capability",
            "declared Rust provider requires foreign.scalar in the project caps",
        ));
    }
    let lock = provider.cargo_lock.as_deref();
    if lock.is_none() && !provider_deps(&provider.source).is_empty() {
        return Err(M2Reject::new(
            "m2build.rust-lock",
            "the Rust provider declares dependencies and needs a Cargo.lock",
        ));
    }
    fs::create_dir_all(scratch).map_err(prepare_error)?;
    let scratch = scratch.canonicalize().map_err(prepare_error)?;
    let rustc = rustc_identity(&scratch).map_err(prepare_error)?;
    let mut build = describe_rust_provider(
        &provider.source,
        rustc,
        Default::default(),
        Default::default(),
    )
    .map_err(prepare_error)?;
    let mut identity = BuildIdentity::of(&build, &build.shim_source);
    identity.dependency_lock_sha256 = lock.map(|text| sha256_hex(text.as_bytes()));
    let root = scratch
        .join("rust-providers")
        .join(sha256_hex(identity.key().as_bytes()));
    let (product_dir, basename) = if lock.is_some() {
        (root.join("cargo-unit/target/debug"), "provider_unit")
    } else {
        (root.join("out"), "f0_text_provider")
    };
    build.cdylib = product_dir.join(format!("{basename}.dll"));
    build.staticlib = product_dir.join(format!("{basename}.lib"));

    fs::create_dir_all(&root).map_err(prepare_error)?;
    let lease = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join("build.lock"))
        .map_err(prepare_error)?;
    lease.lock().map_err(prepare_error)?;
    let digest_path = root.join("cdylib.sha256");
    if let (Ok(expected), Ok(bytes)) = (fs::read_to_string(&digest_path), fs::read(&build.cdylib)) {
        if sha256_hex(&bytes) == expected {
            return Ok(PreparedProvider {
                build,
                identity,
                artifact_sha256: expected,
                rebuilt: false,
                _lease: lease,
            });
        }
    }

    // Invalidate the upstream local build stamp and Cargo's final output
    // before asking its own builder to repair a missing or corrupt DLL.
    for stale in [&digest_path, &build.cdylib] {
        match fs::remove_file(stale) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(prepare_error(err)),
        }
    }
    let actual = if let Some(lock) = lock {
        build_locked_cargo_provider(&root, &provider.source, lock)
    } else {
        build_source(&root, &provider.source)
    }
    .map_err(prepare_error)?;
    if actual.rustc_identity != build.rustc_identity
        || actual.provider_digest != build.provider_digest
        || actual.shim_source != build.shim_source
    {
        return Err(prepare_error(
            "provider build inputs changed during preparation",
        ));
    }
    let artifact_sha256 = sha256_hex(&fs::read(&actual.cdylib).map_err(prepare_error)?);
    fs::write(digest_path, &artifact_sha256).map_err(prepare_error)?;
    Ok(PreparedProvider {
        build: actual,
        identity,
        artifact_sha256,
        rebuilt: true,
        _lease: lease,
    })
}

pub(crate) fn build_and_run(
    inputs: &ProjectInputs,
    source: &str,
    provider: &PreparedProvider,
) -> Result<RunObservation, M2Reject> {
    let mut session = open_session_from_build(&provider.build, 1).map_err(prepare_error)?;
    let program = compile_foreign_program(
        source,
        &inputs.sources[0].0,
        &lane_entry(&inputs.entry),
        &inputs.caps,
        &session.admitted,
    )
    .map_err(|diag| M2Reject::new("m2build.lane-av", diag.to_string()))?;
    let mut capture = ConsoleCapture::default();
    let (outcome, _) = execute_foreign_program(
        &program,
        &inputs.caps,
        &CancelFlag::new(false),
        &mut capture,
        session.session.table(),
    )
    .map_err(|diag| M2Reject::new("m2build.lane-av", diag.to_string()))?;
    let value = match outcome {
        ExecOutcome::Success {
            value: Value::I32(n),
            ..
        } => Some(n),
        ExecOutcome::Success {
            value: Value::Unit, ..
        } => None,
        ExecOutcome::Exit { code } => Some(code),
        other => return Err(M2Reject::new("m2build.lane-av", format!("{other:?}"))),
    };
    Ok(RunObservation {
        exit_code: value,
        stdout: String::from_utf8(capture.stdout)
            .map_err(|e| M2Reject::new("m2build.lane-av", e.to_string()))?,
        channel: "av-value".into(),
    })
}
