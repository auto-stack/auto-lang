//! Project integration of the 967 provider build and the 1093 AV front door.

use std::fs::{self, File};
use std::io::Read;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use ace_a2x_core::sha256::sha256_hex;
use ace_abi::provider_build_cache::BuildIdentity;
use ace_abi::rust_provider::{
    build_locked_cargo_provider, build_source, describe_rust_provider, inspect_rust_cdylib,
    open_sdk_session_from_build, open_session_from_build, provider_deps, rustc_identity, F0Session,
    RustProviderBuild,
};
use ace_av::console_host::ConsoleCapture;
use ace_av::foreign_source::{compile_foreign_program, execute_foreign_program, ForeignProgram};
use ace_av_core::av::CancelFlag;
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::mir::OwnerScope;
use serde::{Deserialize, Serialize};

use crate::compat::{
    check_compatibility, ConsumerRequirements, ProviderOffer, SymbolContract, TargetAxis,
    COMPAT_SCHEMA,
};
use crate::driver::{lane_entry, ProjectInputs, RunObservation, RustProjectInputs};
use crate::identity::TargetKind;
use crate::recipe::{SdkPayload, AV_RUST_PROGRAM_PATH};
use crate::selection::{admit_selection, ProviderArtifact, ProviderSelection, SELECTION_SCHEMA};
use crate::M2Reject;
use crate::SdkRecipe;

const DLL_PATH: &str = "providers/rust/provider.dll";
const ABC_PATH: &str = "targets/av/program.abc";
const SCHEMA: &str = "ace-m2-build/av-rust-program/1";
const PROVIDER_KIND: &str = "rust-cdylib";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RustProgramManifest {
    schema: String,
    provider: String,
    source: String,
    owner_schema: String,
    owner_module: String,
    dll_sha256: String,
    abc_sha256: String,
}

fn sdk_error(error: impl std::fmt::Display) -> M2Reject {
    M2Reject::new("m2build.rust-sdk", error.to_string())
}

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
) -> Result<(RunObservation, Vec<(String, Vec<u8>)>), M2Reject> {
    let mut session = open_session_from_build(&provider.build, 1).map_err(prepare_error)?;
    let program = compile_foreign_program(
        source,
        &inputs.sources[0].0,
        &lane_entry(&inputs.entry),
        &inputs.caps,
        &session.admitted,
    )
    .map_err(|diag| M2Reject::new("m2build.lane-av", diag.to_string()))?;
    let observation = run(&program, &inputs.caps, &mut session)?;
    let dll = fs::read(&provider.build.cdylib).map_err(prepare_error)?;
    if sha256_hex(&dll) != provider.artifact_sha256 {
        return Err(prepare_error("provider DLL changed during build"));
    }
    let manifest = RustProgramManifest {
        schema: SCHEMA.into(),
        provider: session.build.manifest.provider.clone(),
        source: inputs
            .rust_provider
            .as_ref()
            .expect("prepared project")
            .source
            .clone(),
        owner_schema: program.owner.schema,
        owner_module: program.owner.module,
        dll_sha256: provider.artifact_sha256.clone(),
        abc_sha256: sha256_hex(&program.abc),
    };
    Ok((
        observation,
        vec![
            (ABC_PATH.into(), program.abc),
            (
                AV_RUST_PROGRAM_PATH.into(),
                serde_json::to_vec(&manifest).map_err(sdk_error)?,
            ),
            (DLL_PATH.into(), dll),
        ],
    ))
}

fn run(
    program: &ForeignProgram,
    caps: &[String],
    session: &mut F0Session,
) -> Result<RunObservation, M2Reject> {
    let mut capture = ConsoleCapture::default();
    let (outcome, _) = execute_foreign_program(
        program,
        caps,
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

fn file<'a>(files: &'a [(String, Vec<u8>)], path: &str) -> Result<&'a [u8], M2Reject> {
    files
        .iter()
        .find_map(|(name, bytes)| (name == path).then_some(bytes.as_slice()))
        .ok_or_else(|| sdk_error(format!("missing Rust SDK member {path}")))
}

fn manifest(files: &[(String, Vec<u8>)]) -> Result<RustProgramManifest, M2Reject> {
    let manifest: RustProgramManifest =
        serde_json::from_slice(file(files, AV_RUST_PROGRAM_PATH)?).map_err(sdk_error)?;
    if manifest.schema != SCHEMA
        || manifest.dll_sha256 != sha256_hex(file(files, DLL_PATH)?)
        || manifest.abc_sha256 != sha256_hex(file(files, ABC_PATH)?)
    {
        return Err(sdk_error(
            "Rust program schema or artifact identity mismatch",
        ));
    }
    Ok(manifest)
}

fn offer(build: &RustProviderBuild, dll: &[u8]) -> Result<ProviderOffer, M2Reject> {
    let (exports, libraries) = inspect_rust_cdylib(dll).map_err(sdk_error)?;
    let mut offered_symbols = Vec::new();
    for (entry, fact) in build
        .manifest
        .functions
        .iter()
        .zip(&build.manual.facts.functions)
    {
        if !exports.contains(&entry.export) {
            return Err(sdk_error(format!(
                "provider DLL lacks export {}",
                entry.export
            )));
        }
        let sig = match entry.completion {
            Some(mode) => serde_json::to_vec(&(fact, mode)),
            None => serde_json::to_vec(fact),
        }
        .map_err(sdk_error)?;
        offered_symbols.push(SymbolContract {
            name: entry.export.clone(),
            signature_hash: sha256_hex(&sig),
        });
    }
    let is_multi = build
        .manifest
        .functions
        .iter()
        .any(|entry| entry.completion.is_some_and(|mode| mode.is_async()));
    Ok(ProviderOffer {
        target: TargetAxis::host(),
        abi_domain: build.manual.callconv.clone(),
        interface_schema: build.manual.schema.clone(),
        offered_symbols,
        semantic_owner: build.manual.facts.logical.clone(),
        threading: if is_multi { "multi" } else { "single" }.into(),
        runtime_dependencies: libraries,
    })
}

pub(crate) fn selection(
    files: &[(String, Vec<u8>)],
    grant: &str,
) -> Result<(ProviderSelection, ConsumerRequirements), M2Reject> {
    let manifest = manifest(files)?;
    let build = describe_rust_provider(
        &manifest.source,
        String::new(),
        Default::default(),
        Default::default(),
    )
    .map_err(sdk_error)?;
    let offer = offer(&build, file(files, DLL_PATH)?)?;
    let selection = ProviderSelection::new(
        &manifest.provider,
        1,
        vec![ProviderArtifact {
            provider_id: manifest.provider.clone(),
            path: DLL_PATH.into(),
            sha256: manifest.dll_sha256.clone(),
            kind: PROVIDER_KIND.into(),
        }],
        &manifest.dll_sha256,
        grant,
    );
    Ok((
        selection,
        ConsumerRequirements {
            schema: COMPAT_SCHEMA.into(),
            target: offer.target,
            abi_domain: offer.abi_domain,
            interface_schema: offer.interface_schema,
            required_symbols: offer.offered_symbols,
            semantic_owner: offer.semantic_owner,
            threading: offer.threading,
            runtime_dependencies: offer.runtime_dependencies,
        },
    ))
}

pub(crate) fn run_sdk(
    root: &Path,
    recipe: &SdkRecipe,
    files: &[(String, Vec<u8>)],
    grant: &str,
) -> Result<RunObservation, M2Reject> {
    if !recipe.targets.iter().any(|target| target == "av") {
        return Err(sdk_error("Rust program requires the declared AV target"));
    }
    let manifest = manifest(files)?;
    let selection = recipe
        .provider_selection
        .as_ref()
        .ok_or_else(|| sdk_error("Rust SDK lacks a provider selection"))?;
    let [artifact] = selection.artifacts.as_slice() else {
        return Err(sdk_error("Rust SDK requires exactly one selected DLL"));
    };
    if selection.schema != SELECTION_SCHEMA
        || selection.generation == 0
        || selection.provider_id != manifest.provider
        || artifact.provider_id != manifest.provider
        || artifact.path != DLL_PATH
        || artifact.kind != PROVIDER_KIND
    {
        return Err(sdk_error("Rust SDK provider selection identity mismatch"));
    }
    admit_selection(
        selection,
        &manifest.dll_sha256,
        true,
        grant == selection.authorization,
    )
    .map_err(|err| M2Reject::new(err.code, err.detail))?;
    let provenance = recipe
        .provenance
        .iter()
        .find(|identity| identity.target == TargetKind::Av)
        .ok_or_else(|| sdk_error("Rust SDK lacks AV build provenance"))?;
    let identity = provenance
        .provider_build
        .as_ref()
        .ok_or_else(|| sdk_error("Rust SDK lacks provider build provenance"))?;
    let dll = root.join(DLL_PATH).canonicalize().map_err(sdk_error)?;
    if !dll.starts_with(root.canonicalize().map_err(sdk_error)?) {
        return Err(sdk_error("provider DLL resolves outside the SDK root"));
    }
    let build = describe_rust_provider(
        &manifest.source,
        identity.rustc_identity.clone(),
        dll,
        Default::default(),
    )
    .map_err(sdk_error)?;
    for cap in &build.manual.facts.capabilities {
        if !recipe.caps.contains(cap) {
            return Err(M2Reject::new(
                "m2build.foreign-capability",
                format!("missing current grant {cap}"),
            ));
        }
    }
    if build.manifest.provider != manifest.provider
        || identity.provider_sha256 != build.provider_digest
        || identity.target != build.manifest.target
        || provenance.provider_artifact_sha256.as_deref() != Some(&manifest.dll_sha256)
    {
        return Err(sdk_error(
            "Rust SDK source, target, or provider artifact identity mismatch",
        ));
    }
    if provenance.project != recipe.project
        || provenance.entry != recipe.entry
        || provenance.sources != recipe.sources
    {
        return Err(sdk_error(
            "Rust SDK project provenance does not match the source closure",
        ));
    }
    // Keep the verified DLL pinned against concurrent replacement through
    // both LoadLibrary and the lifetime of the provider session.
    let mut pinned = File::options()
        .read(true)
        .share_mode(1) // FILE_SHARE_READ
        .open(&build.cdylib)
        .map_err(sdk_error)?;
    let mut dll_bytes = Vec::new();
    pinned.read_to_end(&mut dll_bytes).map_err(sdk_error)?;
    if sha256_hex(&dll_bytes) != manifest.dll_sha256 {
        return Err(sdk_error("provider DLL changed after SDK verification"));
    }
    let offer = offer(&build, &dll_bytes)?;
    check_compatibility(&recipe.consumer_requirements, &offer)
        .map_err(|err| M2Reject::new(err.code, err.detail))?;
    for library in &offer.runtime_dependencies {
        if !recipe
            .run_dependencies
            .iter()
            .any(|dep| dep.kind == "system-lib" && dep.name == *library)
        {
            return Err(M2Reject::new(
                "m2build.rust-dependency",
                format!("Rust provider runtime dependency {library} is undeclared"),
            ));
        }
    }
    let mut session =
        open_sdk_session_from_build(&build, selection.generation).map_err(sdk_error)?;
    run(
        &ForeignProgram {
            owner: OwnerScope {
                schema: manifest.owner_schema,
                module: manifest.owner_module,
            },
            abc: file(files, ABC_PATH)?.to_vec(),
        },
        &recipe.caps,
        &mut session,
    )
}

struct StagedDirectory(PathBuf);

impl Drop for StagedDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn require_owned_sdk(root: &Path) -> Result<(), M2Reject> {
    const REPARSE_POINT: u32 = 0x400;
    let mut paths = vec![root.to_path_buf()];
    while let Some(path) = paths.pop() {
        let metadata = fs::symlink_metadata(&path).map_err(sdk_error)?;
        if metadata.file_attributes() & REPARSE_POINT != 0 {
            return Err(sdk_error(format!(
                "SDK contains a reparse point at {}",
                path.display()
            )));
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(path).map_err(sdk_error)? {
                paths.push(entry.map_err(sdk_error)?.path());
            }
        }
    }
    if fs::read_dir(root).map_err(sdk_error)?.next().is_some() {
        SdkRecipe::load_verify(root)?;
    }
    Ok(())
}

pub(crate) fn publish_sdk(
    out: &Path,
    recipe: &SdkRecipe,
    payload: &SdkPayload,
) -> Result<(), M2Reject> {
    let parent = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(sdk_error)?;
    let parent = parent.canonicalize().map_err(sdk_error)?;
    let name = out
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| sdk_error("SDK output requires a directory name"))?;
    let out = parent.join(name);
    let backup = parent.join(format!(".{name}.ace-backup"));
    let lease = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(parent.join(format!(".{name}.ace-publish.lock")))
        .map_err(sdk_error)?;
    lease.lock().map_err(sdk_error)?;
    if backup.exists() {
        require_owned_sdk(&backup)?;
        if out.exists() {
            require_owned_sdk(&out)?;
            fs::remove_dir_all(&backup).map_err(sdk_error)?;
        } else {
            fs::rename(&backup, &out).map_err(sdk_error)?;
        }
    }

    let staged = parent.join(format!(".{name}.ace-stage-{}", std::process::id()));
    fs::create_dir(&staged).map_err(sdk_error)?;
    let staged = StagedDirectory(staged);
    recipe.write_to_dir(&staged.0, payload)?;
    SdkRecipe::load_verify(&staged.0)?;

    if out.exists() {
        require_owned_sdk(&out)?;
        fs::rename(&out, &backup).map_err(sdk_error)?;
    }
    if let Err(error) = fs::rename(&staged.0, &out) {
        if backup.exists() {
            fs::rename(&backup, &out).map_err(|restore| {
                sdk_error(format!(
                    "{error}; restore old SDK from {}: {restore}",
                    backup.display()
                ))
            })?;
        }
        return Err(sdk_error(error));
    }
    if backup.exists() {
        fs::remove_dir_all(&backup).map_err(sdk_error)?;
    }
    Ok(())
}
