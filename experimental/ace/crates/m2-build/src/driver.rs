//! PLAN-978 T-02/T-03 (AC-02/AC-03): the engineering build/run driver —
//! one project, five targets (av/native/a2r/a2c/a2py), precise
//! identity-keyed caching in the 783 store, CURRENT publication by CAS,
//! candidate-SDK packaging and relocated precompiled consumption.
//!
//! Incrementality contract (plan §5/V-01..04/V-10): a build whose
//! `ProjectBuildIdentity` already has a live receipt is a cache HIT (no
//! producer work, no tool spawn); an AUTO BODY edit flips the identity of
//! the affected node ONLY — unrelated targets' nodes and the 967 provider
//! build cache stay untouched (an ordinary Auto edit is not a provider
//! build input); real input changes (provider/shim/tool axes) MISS and
//! rebuild. Every observation rides real counters, never self-reported
//! booleans.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::compat::{check_compatibility, ConsumerRequirements, ProviderOffer};
use crate::identity::{
    BuildReceipt, ProjectBuildIdentity, TargetKind, BUILD_RECEIPT_SCHEMA, M978B_LAYER,
};
use crate::probe;
use crate::recipe::{
    BuildToolDependency, RunDependency, SdkArtifact, SdkPayload, SdkRecipe, SDK_RECIPE_SCHEMA,
};
use crate::selection::{admit_selection, ProviderArtifact, ProviderSelection};
use crate::M2Reject;
use ace_a2x_core::target::TargetProvider as _;

/// Default namespace quota for a2x sessions opened by this driver.
pub const DEFAULT_A2X_QUOTA: u64 = 64 * 1024 * 1024;

/// One project's declared inputs.
#[derive(Clone, Debug)]
pub struct ProjectInputs {
    pub name: String,
    pub entry: String,
    pub caps: Vec<String>,
    /// Exactly one source (the public entry source). Suffix-provider
    /// layers are SDK-closure material (crate::load), never silently
    /// merged here.
    pub sources: Vec<(String, String)>,
    /// Declared stdlib closure members (path under `stdlib/auto`, text).
    pub stdlib: Vec<(String, String)>,
}

impl ProjectInputs {
    pub fn new(name: &str, entry: &str, source_path: &str, source_text: &str) -> Self {
        Self {
            name: name.into(),
            entry: entry.into(),
            caps: vec![],
            sources: vec![(source_path.into(), source_text.into())],
            stdlib: vec![],
        }
    }

    fn entry_source(&self) -> Result<&str, M2Reject> {
        if self.sources.len() != 1 {
            return Err(M2Reject::new(
                "m2build.project.multi-source",
                format!(
                    "expected exactly one public source, found {} (suffix-layer \
                     selection is the SDK closure ledger, not a silent merge)",
                    self.sources.len()
                ),
            ));
        }
        Ok(&self.sources[0].1)
    }
}

/// One observed execution (value channel or process exit).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunObservation {
    /// The i32 result of the entry function (AV lane) or the child exit
    /// code (process lanes).
    pub exit_code: Option<i32>,
    pub stdout: String,
    /// How the value was observed: "av-value" | "process-exit" |
    /// "process-stdout" | "precompiled-provider".
    pub channel: String,
}

/// Real counters of one build (never self-reported booleans).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildCounts {
    pub lane: String,
    pub fragments_produced: u64,
    pub store_hit_fragment: u64,
    pub store_hit_semantic: u64,
    /// Child processes THIS driver spawned, by seam name (the independent
    /// machine-level oracle is crate::probe).
    pub tool_spawns: BTreeMap<String, u64>,
}

#[derive(Clone, Debug)]
pub struct BuildOutcome {
    pub target: TargetKind,
    pub identity: ProjectBuildIdentity,
    pub rebuilt: bool,
    pub run: Option<RunObservation>,
    /// (SDK-relative path, bytes) products of the build node.
    pub products: Vec<(String, Vec<u8>)>,
    pub counts: BuildCounts,
}

/// An optional precompiled provider artifact packed into the SDK.
pub struct ProviderArtifactInput {
    pub provider_id: String,
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub kind: String,
}

pub struct M2Build {
    store_root: PathBuf,
    scratch: PathBuf,
    spawns: BTreeMap<String, u64>,
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, M2Reject> {
    if s.len() % 2 != 0 {
        return Err(M2Reject::new("m2build.hex", "odd hex length"));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for pair in s.as_bytes().chunks(2) {
        let hi = (pair[0] as char).to_digit(16);
        let lo = (pair[1] as char).to_digit(16);
        match (hi, lo) {
            (Some(h), Some(l)) => out.push(((h << 4) | l) as u8),
            _ => return Err(M2Reject::new("m2build.hex", "non-hex digit")),
        }
    }
    Ok(out)
}

fn grammar_identity(target: TargetKind) -> String {
    match target {
        TargetKind::Av => format!("av.core-i32/1 abc/{}", ace_av_core::abc::ABC_VERSION),
        TargetKind::Native => format!("mn/{}", ace_native::backend::Backend::Mn.tag()),
        TargetKind::A2r => {
            let d = ace_a2x_rust::RustTargetProvider::v1().descriptor();
            format!("a2x/{}/{}", d.backend, d.backend_version)
        }
        TargetKind::A2c => {
            let d = ace_a2x_c::CTargetProvider::v1().descriptor();
            format!("a2x/{}/{}", d.backend, d.backend_version)
        }
        TargetKind::A2py => {
            let d = ace_a2x_python::PythonTargetProvider.descriptor();
            format!("a2x/{}/{}", d.backend, d.backend_version)
        }
    }
}

fn tool_axes(target: TargetKind) -> Vec<(String, String)> {
    match target {
        // Provenance axes RECORDED from real reads (never runtime
        // admission): the generator identity of this crate and the lane
        // backend tag.
        TargetKind::Av => vec![("generator".into(), generator_identity())],
        TargetKind::Native => vec![
            ("generator".into(), generator_identity()),
            ("linker".into(), ace_core_linker_identity()),
        ],
        _ => vec![("generator".into(), generator_identity())],
    }
}

fn generator_identity() -> String {
    format!("ac-build/{}", env!("CARGO_PKG_VERSION"))
}

/// The declared entry NAME (`main`) maps to the lane def id (`d_main`) —
/// the same mapping `ace_native::entry::resolve` freezes (source mode
/// takes the declared name; the id is never the lookup key for
/// hand-authored HIR, so both spellings pass through).
fn lane_entry(entry: &str) -> String {
    if entry.starts_with("d_") || entry.starts_with("sf_") {
        entry.to_string()
    } else {
        format!("d_{entry}")
    }
}

fn ace_core_linker_identity() -> String {
    match ace_core::link::find_rust_lld() {
        Ok(p) => format!("rust-lld@{}", p.display()),
        Err(_) => "rust-lld:unresolved".into(),
    }
}

impl M2Build {
    /// `root` receives `store/` (the 783 namespaces) and `scratch/`
    /// (native lane workdirs).
    pub fn new(root: &Path) -> Self {
        M2Build {
            store_root: root.join("store"),
            scratch: root.join("scratch"),
            spawns: BTreeMap::new(),
        }
    }

    pub fn spawn_counts(&self) -> &BTreeMap<String, u64> {
        &self.spawns
    }

    fn track_spawn(&mut self, name: &str) {
        *self.spawns.entry(name.to_string()).or_insert(0) += 1;
    }

    fn current_kind(target: TargetKind) -> String {
        format!("m2build/{}", target.name())
    }

    /// Build one (project, target) node with the precise identity cache.
    pub fn build(
        &mut self,
        target: TargetKind,
        inputs: &ProjectInputs,
    ) -> Result<BuildOutcome, M2Reject> {
        let src = inputs.entry_source()?;
        let identity =
            ProjectBuildIdentity::of(inputs, target, &grammar_identity(target), tool_axes(target));
        let key = ace_artifacts::identity::QueryKey {
            layer: M978B_LAYER,
            components: vec![("identity".into(), identity.key())],
        };
        let store = ace_artifacts::store::ManagedStore::open(&self.store_root.join("m2b"))
            .map_err(|e| M2Reject::new("m2build.store", format!("open: {e:?}")))?;
        // Precise-cache probe: a HIT needs a live receipt row.
        if let Ok(Some(loaded)) = store.get_artifact(M978B_LAYER, &key) {
            if let Ok(receipt) = serde_json::from_slice::<BuildReceipt>(&loaded.payload) {
                if receipt.schema == BUILD_RECEIPT_SCHEMA
                    && receipt.identity.key() == identity.key()
                {
                    let products = receipt
                        .products
                        .iter()
                        .map(|(p, h)| hex_decode(h).map(|b| (p.clone(), b)))
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(BuildOutcome {
                        target,
                        identity,
                        rebuilt: false,
                        run: None,
                        products,
                        counts: BuildCounts {
                            lane: "cache-hit".into(),
                            ..Default::default()
                        },
                    });
                }
            }
        }
        // MISS: real producer work per lane.
        let (products, run, mut counts) = match target {
            TargetKind::Av => self.lane_av(inputs, src)?,
            TargetKind::Native => self.lane_native(inputs, src)?,
            TargetKind::A2r => self.lane_a2x(inputs, src, "a2r")?,
            TargetKind::A2c => self.lane_a2x(inputs, src, "a2c")?,
            TargetKind::A2py => self.lane_a2x(inputs, src, "a2py")?,
        };
        counts.tool_spawns = self.spawns.clone();
        // Persist the receipt + CURRENT publication (CAS on the previous
        // root — a concurrent writer loses loudly, never silently).
        let receipt = BuildReceipt {
            schema: BUILD_RECEIPT_SCHEMA.into(),
            identity: identity.clone(),
            products: products
                .iter()
                .map(|(p, b)| (p.clone(), hex_encode(b)))
                .collect(),
        };
        let payload = serde_json::to_vec(&receipt)
            .map_err(|e| M2Reject::new("m2build.io", format!("encode receipt: {e}")))?;
        let put = store.put_artifact(ace_artifacts::store::PutRequest {
            layer: M978B_LAYER,
            key: key.clone(),
            kind: ace_artifacts::codec::ChunkKind::LayoutData,
            payload,
            edges: vec![],
            project: Some(inputs.name.clone()),
        });
        if let Err(e) = &put {
            if matches!(e, ace_artifacts::store::StoreError::Conflict { .. }) {
                // The store never overwrites: a row already lives under
                // this exact identity. A VALID receipt for this identity
                // keeps the cache authoritative (idempotent rebuild); a
                // foreign/corrupted row is a NAMED refusal — the unblock
                // is `restore`, which purges this cache-only layer.
                let existing = store
                    .get_artifact(M978B_LAYER, &key)
                    .map_err(|e| M2Reject::new("m2build.store", format!("re-read: {e:?}")))?;
                let valid = existing
                    .as_ref()
                    .and_then(|l| serde_json::from_slice::<BuildReceipt>(&l.payload).ok())
                    .is_some_and(|r| {
                        r.schema == BUILD_RECEIPT_SCHEMA && r.identity.key() == identity.key()
                    });
                if !valid {
                    return Err(M2Reject::new(
                        "m2build.store.poisoned-row",
                        format!(
                            "the receipt row under identity `{}` is foreign or corrupted; \
                             run `ac-build restore` (purges the regenerable M978B cache layer)",
                            identity.key()
                        ),
                    ));
                }
            } else {
                return Err(M2Reject::new("m2build.store", format!("put: {e:?}")));
            }
        }
        let artifact_id = {
            // re-read our row for its artifact id (put returns ())
            let re = store
                .get_artifact(
                    M978B_LAYER,
                    &ace_artifacts::identity::QueryKey {
                        layer: M978B_LAYER,
                        components: vec![("identity".into(), identity.key())],
                    },
                )
                .map_err(|e| M2Reject::new("m2build.store", format!("re-read: {e:?}")))?;
            match re {
                Some(loaded) => loaded.artifact_id,
                None => {
                    return Err(M2Reject::new(
                        "m2build.store",
                        "receipt row missing right after put",
                    ))
                }
            }
        };
        let kind = Self::current_kind(target);
        let prev = store
            .root_artifacts(&inputs.name, &kind)
            .map_err(|e| M2Reject::new("m2build.store", format!("roots: {e:?}")))?;
        store
            .replace_current_root(
                &inputs.name,
                &kind,
                prev.first().map(|s| s.as_str()),
                &artifact_id,
            )
            .map_err(|e| {
                M2Reject::new(
                    "m2build.current",
                    format!("CURRENT publication lost the race: {e:?}"),
                )
            })?;
        Ok(BuildOutcome {
            target,
            identity,
            rebuilt: true,
            run,
            products,
            counts,
        })
    }

    fn lane_av(
        &mut self,
        inputs: &ProjectInputs,
        src: &str,
    ) -> Result<(Vec<(String, Vec<u8>)>, Option<RunObservation>, BuildCounts), M2Reject> {
        let mut driver =
            ace_av::driver::Driver::with_store(&self.store_root.join("av"), &inputs.name)
                .map_err(|e| M2Reject::new("m2build.lane-av", e))?;
        let outcome = driver.request(src, &lane_entry(&inputs.entry), &inputs.caps);
        if let Some(d) = &outcome.diag {
            return Err(M2Reject::new(
                "m2build.lane-av",
                format!("[{}] {}: {}", d.stage, d.code, d.message),
            ));
        }
        let value = outcome.value_i32();
        let run = RunObservation {
            exit_code: value,
            stdout: String::new(),
            channel: "av-value".into(),
        };
        let receipt = serde_json::json!({
            "schema": BUILD_RECEIPT_SCHEMA,
            "target": "av",
            "observed": value,
            "channel": outcome.channel(),
            "wall_ms": outcome.wall_ms,
        });
        Ok((
            vec![(
                "targets/av/build-receipt.json".into(),
                serde_json::to_vec_pretty(&receipt).unwrap(),
            )],
            Some(run),
            BuildCounts {
                lane: "av".into(),
                ..Default::default()
            },
        ))
    }

    fn lane_native(
        &mut self,
        inputs: &ProjectInputs,
        src: &str,
    ) -> Result<(Vec<(String, Vec<u8>)>, Option<RunObservation>, BuildCounts), M2Reject> {
        let workdir = self
            .scratch
            .join(format!("native-{}-{}", inputs.name, std::process::id()));
        let _ = std::fs::remove_dir_all(&workdir);
        std::fs::create_dir_all(&workdir).map_err(|e| {
            M2Reject::new("m2build.io", format!("create {}: {e}", workdir.display()))
        })?;
        let mut driver =
            ace_native::driver::MnDriver::with_store(&self.store_root.join("mn"), &inputs.name)
                .map_err(|e| M2Reject::new("m2build.lane-native", format!("{e:?}")))?;
        let req = driver.request(
            src,
            &lane_entry(&inputs.entry),
            &inputs.caps,
            &workdir,
            None,
        );
        if let Some(err) = &req.error {
            return Err(M2Reject::new(
                "m2build.lane-native",
                format!("[{}] {}", err.code(), err.message()),
            ));
        }
        let stdout = String::from_utf8_lossy(&req.stdout).to_string();
        // Collect the staged exe (the precompiled artifact of this node).
        let mut exe: Option<(String, Vec<u8>)> = None;
        if let Ok(rd) = std::fs::read_dir(&workdir) {
            for entry in rd.flatten() {
                if entry.path().extension().and_then(|e| e.to_str()) == Some("exe") {
                    let bytes = std::fs::read(entry.path()).map_err(|e| {
                        M2Reject::new(
                            "m2build.io",
                            format!("read {}: {e}", entry.path().display()),
                        )
                    })?;
                    exe = Some((format!("targets/native/{}.exe", inputs.name), bytes));
                    break;
                }
            }
        }
        let mut products = vec![
            // Receipt with the lane's real identity needles.
            (
                "targets/native/build-receipt.json".to_string(),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "schema": BUILD_RECEIPT_SCHEMA,
                    "target": "native",
                    "manifest_fp": req.manifest_fp,
                    "exit_code": req.exit_code,
                }))
                .unwrap(),
            ),
        ];
        if let Some((p, b)) = exe {
            products.insert(0, (p, b));
        }
        let run = RunObservation {
            exit_code: req.exit_code,
            stdout,
            channel: "process-exit".into(),
        };
        let _ = std::fs::remove_dir_all(&workdir);
        Ok((
            products,
            Some(run),
            BuildCounts {
                lane: "native".into(),
                ..Default::default()
            },
        ))
    }

    fn lane_a2x(
        &mut self,
        inputs: &ProjectInputs,
        src: &str,
        lane: &str,
    ) -> Result<(Vec<(String, Vec<u8>)>, Option<RunObservation>, BuildCounts), M2Reject> {
        let target = TargetKind::parse(lane).unwrap();
        let mut session = match target {
            TargetKind::A2r => ace_a2x_store::A2xSession::open(
                &self.store_root.join("a2x"),
                &inputs.name,
                "m2build",
                DEFAULT_A2X_QUOTA,
                ace_a2x_rust::RustTargetProvider::v1(),
            ),
            TargetKind::A2c => ace_a2x_store::A2xSession::open(
                &self.store_root.join("a2x"),
                &inputs.name,
                "m2build",
                DEFAULT_A2X_QUOTA,
                ace_a2x_c::CTargetProvider::v1(),
            ),
            _ => ace_a2x_store::A2xSession::open(
                &self.store_root.join("a2x"),
                &inputs.name,
                "m2build",
                DEFAULT_A2X_QUOTA,
                ace_a2x_python::PythonTargetProvider,
            ),
        }
        .map_err(|e| M2Reject::new("m2build.lane-a2x", e))?;
        // PLAN-1107 T-05: a `use.py` project on the A2PY target compiles through
        // the AGGREGATES surface with the neutral py declaration contract (the
        // shared front door); every other lane keeps its frozen cold path.
        let is_py_lane = matches!(target, TargetKind::A2py) && src.contains("use.py");
        let assembly = if is_py_lane {
            session.cold_assemble_py(src, &inputs.caps, &inputs.entry, false)
        } else {
            session.cold_assemble(src, true, &inputs.caps, &inputs.entry, false)
        }
        .map_err(|e| {
                M2Reject::new(
                    "m2build.lane-a2x",
                    match &e {
                        ace_a2x_store::A2xRejectOrResource::Reject(r) => r.to_string(),
                        ace_a2x_store::A2xRejectOrResource::Resource(r) => r.to_string(),
                    },
                )
            })?;
        let stats = session.stats();
        let counts = BuildCounts {
            lane: lane.into(),
            fragments_produced: stats.fragments_produced,
            store_hit_fragment: stats.store_hit_fragment,
            store_hit_semantic: stats.store_hit_semantic,
            tool_spawns: BTreeMap::new(),
        };
        let mut products = Vec::new();
        for (name, text) in &assembly.files {
            products.push((format!("targets/{lane}/{name}"), text.clone().into_bytes()));
        }
        products.push((
            format!("targets/{lane}/assembly-manifest.json"),
            assembly.manifest.clone().into_bytes(),
        ));
        Ok((products, None, counts))
    }

    /// Build every target and pack one relocatable candidate SDK.
    pub fn build_sdk(
        &mut self,
        inputs: &ProjectInputs,
        targets: &[TargetKind],
        out_dir: &Path,
        providers: Vec<ProviderArtifactInput>,
        grant: &str,
    ) -> Result<SdkRecipe, M2Reject> {
        let mut provenance = Vec::new();
        let mut artifacts = Vec::new();
        let mut payload_files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut run_deps: Vec<RunDependency> = Vec::new();
        let mut build_inputs: Vec<BuildToolDependency> = Vec::new();
        let mut last_identity = None;
        for target in targets {
            let outcome = self.build(*target, inputs)?;
            provenance.push(outcome.identity.clone());
            if outcome.identity.target == TargetKind::Av || last_identity.is_none() {
                last_identity = Some(outcome.identity.clone());
            }
            for (path, bytes) in outcome.products {
                let sha = ace_a2x_core::sha256::sha256_hex(&bytes);
                artifacts.push(SdkArtifact {
                    path: path.clone(),
                    role: if path.ends_with(".exe") {
                        "exe".into()
                    } else if path.ends_with("build-receipt.json") {
                        "receipt".into()
                    } else if path.ends_with("assembly-manifest.json") {
                        "manifest".into()
                    } else {
                        "fileset".into()
                    },
                    sha256: sha.clone(),
                    size: bytes.len() as u64,
                });
                payload_files.push((path, bytes));
            }
            match target {
                TargetKind::Av => run_deps.push(RunDependency {
                    name: "ace-av-core".into(),
                    kind: "av-host".into(),
                    detail: format!(
                        "in-process AV executor abc/{}",
                        ace_av_core::abc::ABC_VERSION
                    ),
                }),
                TargetKind::Native => {
                    run_deps.push(RunDependency {
                        name: "msvc-crt".into(),
                        kind: "crt".into(),
                        detail: "VCRUNTIME140+ucrt (dynamic CRT family the lane linked)".into(),
                    });
                    build_inputs.push(BuildToolDependency {
                        name: "linker".into(),
                        identity: ace_core_linker_identity(),
                        role: "native link".into(),
                    });
                }
                TargetKind::A2py => {
                    run_deps.push(RunDependency {
                        name: "cpython".into(),
                        kind: "interpreter".into(),
                        detail: "python3 (declared A2PY runtime interpreter)".into(),
                    });
                }
                _ => {}
            }
        }
        // Provider artifacts (precompiled consumption lanes).
        let mut selection: Option<ProviderSelection> = None;
        for p in &providers {
            let sha = ace_a2x_core::sha256::sha256_hex(&p.bytes);
            let path = format!("providers/{}", p.file_name);
            artifacts.push(SdkArtifact {
                path: path.clone(),
                role: "provider".into(),
                sha256: sha.clone(),
                size: p.bytes.len() as u64,
            });
            payload_files.push((path.clone(), p.bytes.clone()));
            selection = Some(ProviderSelection::new(
                &p.provider_id,
                1,
                vec![ProviderArtifact {
                    provider_id: p.provider_id.clone(),
                    path,
                    sha256: sha,
                    kind: p.kind.clone(),
                }],
                // pin the CURRENT bytes (an explicit update re-pins)
                &ace_a2x_core::sha256::sha256_hex(&p.bytes),
                grant,
            ));
        }
        // Source closure + stdlib closure.
        let mut sources = Vec::new();
        for (path, text) in &inputs.sources {
            sources.push(crate::identity::SourceFile {
                path: path.clone(),
                role: "public".into(),
                sha256: ace_a2x_core::sha256::sha256_hex(text.as_bytes()),
            });
            payload_files.push((
                format!("sources/{}/{}", inputs.name, path),
                text.clone().into_bytes(),
            ));
        }
        for (path, text) in &inputs.stdlib {
            payload_files.push((path.clone(), text.clone().into_bytes()));
        }
        let entry_symbol = format!("sf_{}", inputs.entry);
        let consumer = ConsumerRequirements::native_program(
            &entry_symbol,
            &ace_a2x_core::sha256::sha256_hex(format!("{entry_symbol}()->i32").as_bytes()),
        );
        let recipe = SdkRecipe {
            schema: SDK_RECIPE_SCHEMA.into(),
            project: inputs.name.clone(),
            entry: inputs.entry.clone(),
            caps: inputs.caps.clone(),
            targets: targets.iter().map(|t| t.name().into()).collect(),
            sources,
            artifacts,
            build_inputs,
            run_dependencies: run_deps,
            consumer_requirements: consumer,
            provider_selection: selection,
            provenance,
        };
        recipe.write_to_dir(
            out_dir,
            &SdkPayload {
                files: payload_files,
            },
        )?;
        Ok(recipe)
    }

    /// Run from a RELOCATED candidate SDK: verify the closure, admit the
    /// selection with the CURRENT grant, check compatibility, then
    /// dispatch — in that order. Never reads the development tree and
    /// never requires a Rust/C toolchain.
    pub fn run_sdk(
        &mut self,
        sdk_root: &Path,
        target: TargetKind,
        grant: &str,
    ) -> Result<RunObservation, M2Reject> {
        let (recipe, files) = SdkRecipe::load_verify(sdk_root)?;
        // Admission 1: the selection (per-run grant; precise pin).
        if let Some(selection) = &recipe.provider_selection {
            let primary = &selection.artifacts[0];
            let on_disk = files
                .iter()
                .find(|(p, _)| *p == primary.path)
                .map(|(_, b)| ace_a2x_core::sha256::sha256_hex(b))
                .unwrap_or_default();
            admit_selection(selection, &on_disk, true, selection.authorization == grant)
                .map_err(|r| M2Reject::new(r.code, r.detail))?;
        }
        // Admission 2: runtime compatibility on THIS host.
        let host_offer = ProviderOffer {
            target: crate::compat::TargetAxis::host(),
            abi_domain: recipe.consumer_requirements.abi_domain.clone(),
            interface_schema: recipe.consumer_requirements.interface_schema.clone(),
            offered_symbols: recipe.consumer_requirements.required_symbols.clone(),
            semantic_owner: recipe.consumer_requirements.semantic_owner.clone(),
            threading: recipe.consumer_requirements.threading.clone(),
            runtime_dependencies: recipe.consumer_requirements.runtime_dependencies.clone(),
        };
        check_compatibility(&recipe.consumer_requirements, &host_offer)
            .map_err(|r| M2Reject::new(r.code, r.detail))?;
        match target {
            TargetKind::Av => {
                let src = SdkRecipe::entry_source(&recipe, &files)?;
                let mut driver = ace_av::driver::Driver::ephemeral();
                let outcome = driver.request(&src, &lane_entry(&recipe.entry), &recipe.caps);
                if let Some(d) = &outcome.diag {
                    return Err(M2Reject::new(
                        "m2build.run",
                        format!("[{}] {}: {}", d.stage, d.code, d.message),
                    ));
                }
                Ok(RunObservation {
                    exit_code: outcome.value_i32(),
                    stdout: String::new(),
                    channel: "av-value".into(),
                })
            }
            TargetKind::Native | TargetKind::A2r | TargetKind::A2c => {
                let exe = files
                    .iter()
                    .find(|(p, _)| p.starts_with("targets/") && p.ends_with(".exe"))
                    .ok_or_else(|| {
                        M2Reject::new(
                            "m2build.run.requires-build-lane",
                            format!(
                                "no precompiled exe in this SDK for target `{}` — the \
                                 source-build lane (rustc/clang) is a BUILD-time lane, not a \
                                 run requirement; build the artifact first and pack it",
                                target.name()
                            ),
                        )
                    })?;
                let full = sdk_root.join(exe.0.replace('/', "\\"));
                let run = self.spawn_capture("sdk-exe", &full);
                Ok(RunObservation {
                    exit_code: run.0,
                    stdout: run.1,
                    channel: "process-exit".into(),
                })
            }
            TargetKind::A2py => {
                // The generated fileset ships its own runner
                // (`a2x_main.py`: sys.exit(a2x_run_main(generated.a2x_fn_*)))
                // — run it through the DECLARED interpreter with the
                // fileset directory as cwd.
                let has_runner = files.iter().any(|(p, _)| p == "targets/a2py/a2x_main.py");
                if !has_runner {
                    return Err(M2Reject::new(
                        "m2build.run.requires-build-lane",
                        "no a2py runner (a2x_main.py) in this SDK",
                    ));
                }
                let dir = sdk_root.join("targets").join("a2py");
                let run = self.spawn_capture_args(
                    "python-interpreter",
                    "python",
                    &["-B", "a2x_main.py"],
                    Some(&dir),
                );
                if run.0.is_none() {
                    return Err(M2Reject::new(
                        "m2build.run.interpreter-failed",
                        format!("python interpreter unavailable: {}", run.1),
                    ));
                }
                // The generated runner prints `value:<n>` and exits 0 —
                // normalize the observation to the VALUE (the lane's
                // observable), keeping the raw stdout.
                if let Some(v) = run.1.trim().strip_prefix("value:") {
                    if let Ok(n) = v.trim().parse::<i32>() {
                        return Ok(RunObservation {
                            exit_code: Some(n),
                            stdout: run.1,
                            channel: "process-stdout".into(),
                        });
                    }
                }
                Ok(RunObservation {
                    exit_code: run.0,
                    stdout: run.1,
                    channel: "process-exit".into(),
                })
            }
        }
    }

    /// Verify a candidate SDK directory WITHOUT running it (the negative
    /// controls' entry point: integrity, schema, extras, escapes).
    pub fn verify_sdk(sdk_root: &Path) -> Result<Vec<String>, M2Reject> {
        let (recipe, files) = SdkRecipe::load_verify(sdk_root)?;
        let mut checks = vec![
            format!("schema={}", recipe.schema),
            format!("project={}", recipe.project),
            format!("targets={}", recipe.targets.join("+")),
            format!("artifacts={}", recipe.artifacts.len()),
            format!("verified-files={}", files.len()),
        ];
        for dep in &recipe.run_dependencies {
            checks.push(format!("run-dep:{}:{}", dep.kind, dep.name));
        }
        for tool in &recipe.build_inputs {
            checks.push(format!("build-input:{}={}", tool.name, tool.identity));
        }
        Ok(checks)
    }

    /// Cold recovery: run the 783 recovery pass over the m2b namespace
    /// and re-verify every CURRENT root is readable (plan §5 ID-04: a
    /// restore re-checks selection/declaration/content/authorization —
    /// historical PASS never skips a step).
    pub fn restore(&mut self, project: &str) -> Result<Vec<String>, M2Reject> {
        let root = self.store_root.join("m2b");
        if !root.is_dir() {
            return Err(M2Reject::new(
                "m2build.restore.missing-store",
                format!("store root {} does not exist", root.display()),
            ));
        }
        let store = ace_artifacts::store::ManagedStore::open_existing(&root)
            .map_err(|e| M2Reject::new("m2build.restore", format!("open_existing: {e:?}")))?;
        let receipt = ace_artifacts::store::recover(&store, false)
            .map_err(|e| M2Reject::new("m2build.restore", format!("recover: {e:?}")))?;
        let mut out = vec![format!(
            "recovered: staging={} orphan-chunks={} orphan-rows={} stale={} txns={}",
            receipt.staging_dirs_removed,
            receipt.orphan_chunks_removed,
            receipt.orphan_rows_removed,
            receipt.stale_rows_removed,
            receipt.aborted_txn_rows
        )];
        // Re-verify every published CURRENT root BEFORE any purge (the
        // purge below retires the roots of the cache layer).
        for target in TargetKind::ALL {
            let kind = Self::current_kind(target);
            if let Ok(roots) = store.root_artifacts(project, &kind) {
                for id in roots {
                    let ok = store
                        .get_artifact_by_id(&id)
                        .map(|a| a.is_some())
                        .unwrap_or(false);
                    out.push(format!(
                        "current/{}/{id}: {}",
                        target.name(),
                        if ok { "live" } else { "DEAD" }
                    ));
                }
            }
        }
        // The M978B receipt layer is a REGENERABLE cache: purge it so a
        // poisoned row (foreign payload under a valid identity) cannot
        // outlive recovery — the next build re-produces and re-pins.
        let purged = store
            .purge_layer(M978B_LAYER)
            .map_err(|e| M2Reject::new("m2build.restore", format!("purge M978B: {e:?}")))?;
        out.push(format!("m978b-rows-purged={purged}"));
        Ok(out)
    }

    fn spawn_capture(&mut self, name: &str, exe: &Path) -> (Option<i32>, String) {
        let program = exe.to_string_lossy().to_string();
        let program = program.as_str();
        self.spawn_capture_args(name, program, &[], None)
    }

    fn spawn_capture_args(
        &mut self,
        name: &str,
        program: &str,
        args: &[&str],
        cwd: Option<&Path>,
    ) -> (Option<i32>, String) {
        self.track_spawn(name);
        let mut cmd = Command::new(program);
        cmd.args(args);
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }
        match cmd.output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                (out.status.code(), stdout)
            }
            Err(e) => (None, format!("spawn {program}: {e}")),
        }
    }
}

/// One machine-level tool-process observation the tests drive directly.
pub fn tool_process_delta<F, T>(f: F) -> (T, Vec<String>)
where
    F: FnOnce() -> T,
{
    let before = probe::snapshot();
    let out = f();
    let after = probe::snapshot();
    (out, after.increases_since(&before))
}
