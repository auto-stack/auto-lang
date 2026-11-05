//! PLAN-938 source-import host composition. The SAME neutral compiler session
//! feeds either shared MIR/native or HIR/C TargetProvider; no target owns a
//! parser, solver, local foreign stub or persistent Checked credential.
use ace_a2x_c::foreign::ForeignTargetProvider;
use ace_a2x_core::{
    target::{Family, FilesetRequest, FragmentText, TargetProvider},
    SemanticView,
};
use ace_artifacts::{
    codec::ChunkKind,
    identity::{QueryKey, RefKind},
    schema::{LayerCoverage, LayerSpec, Regeneration},
    store::{ManagedStore, PutRequest},
};
use ace_foreign_contract::manual::AdmittedForeign;
use ace_source_query::frontend::{
    Candidate as SourceCandidate, ForeignCandidate as SourceForeignCandidate, Frontend,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

pub const SCHEMA: &str = "abi/source-foreign-session/1";
const MAX_ROOT: usize = 8 << 20;
const MAX_ARTIFACT: usize = 64 << 20;
const MAX_RESIDENT: usize = 48 << 20;
static IDS: AtomicU64 = AtomicU64::new(1);
fn hash(b: &[u8]) -> String {
    ace_a2x_core::sha256::sha256_hex(b)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    Native,
    C,
}
impl Target {
    pub fn identity(self) -> String {
        match self {
            Self::Native => ace_native::foreign_lower::emitter_identity(),
            Self::C => "auto-a2x-c-foreign/1+c11+a2x-c-foreign-print/1".into(),
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Meter {
    pub mir_attempts: u64,
    pub mir_produced: u64,
    pub native_attempts: u64,
    pub native_produced: u64,
    pub c_attempts: u64,
    pub c_produced: u64,
    pub page_encodes: u64,
    pub restore_pages: u64,
    pub links: u64,
}

struct Artifact {
    digest: String,
    bytes: Arc<Vec<u8>>,
    decoded: TargetWire,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum TargetWire {
    Native {
        name: String,
        symbol: String,
        owner_schema: String,
        owner_module: String,
        mir: Vec<u8>,
        coff: Vec<u8>,
    },
    C {
        name: String,
        def_index: usize,
        semantic: String,
        text: String,
    },
}
impl Artifact {
    fn new(decoded: TargetWire) -> Result<Arc<Self>, String> {
        let bytes = serde_json::to_vec(&decoded).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_ARTIFACT {
            return Err("foreign.resource.artifact".into());
        }
        Ok(Arc::new(Self {
            digest: hash(&bytes),
            bytes: Arc::new(bytes),
            decoded,
        }))
    }
}

pub struct Session {
    front: Frontend,
    target: Target,
    artifacts: BTreeMap<String, Arc<Artifact>>,
    runtime: Arc<Vec<u8>>,
    provider: Arc<Vec<u8>>,
    id: u64,
    generation: u64,
    meter: Meter,
    per_function: BTreeMap<String, Meter>,
}
pub struct Candidate {
    source: SourceCandidate,
    artifact: Arc<Artifact>,
    owner: u64,
    base: u64,
}
pub struct ForeignCandidate {
    source: SourceForeignCandidate,
    artifacts: BTreeMap<String, Arc<Artifact>>,
    provider: Arc<Vec<u8>>,
    owner: u64,
    base: u64,
}
pub struct Snapshot {
    pub root: Vec<u8>,
    pub digest: String,
    pub pages: BTreeMap<String, Arc<Vec<u8>>>,
}

/// The original managed store remains the ONLY persistence/GC/lease authority.
/// Content edges include frozen semantic dependencies, so same source body
/// with different foreign facts never overwrites an immutable query key.
pub const LAYER: LayerSpec = LayerSpec {
    id: "S938",
    name: "checked-source-foreign-session",
    schema_id: SCHEMA,
    producer: "auto-abi-source-foreign/1",
    consumers: &["source-import restore"],
    key_inputs: &["content-sha256"],
    ref_kinds: &["required"],
    storage: "original ManagedStore immutable pages",
    residency: "source/selected-target root closure",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stored {
    pub artifact_id: String,
    pub digest: String,
    pub pages: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreRoot {
    schema: String,
    digest: String,
    root: Vec<u8>,
    pages: BTreeMap<String, String>,
}
impl Snapshot {
    /// New content is written once. Unchanged required pages reuse only their
    /// resident artifact ids: no store payload read or extra re-encoding.
    pub fn publish(
        &self,
        store: &ManagedStore,
        project: &str,
        previous: Option<&Stored>,
    ) -> Result<Stored, String> {
        let mut ids = BTreeMap::new();
        let mut leases = Vec::new();
        for (digest, bytes) in &self.pages {
            if let Some(id) = previous.and_then(|p| p.pages.get(digest)) {
                ids.insert(digest.clone(), id.clone());
                continue;
            }
            let id = store
                .put_artifact(PutRequest {
                    layer: LAYER.id,
                    key: content_key(digest),
                    kind: ChunkKind::SnapshotRecord,
                    payload: bytes.as_ref().clone(),
                    edges: vec![],
                    project: None,
                })
                .map_err(|e| e.to_string())?;
            leases.push(
                store
                    .acquire_lease("source938-candidate-page", 120, Some(&id))
                    .map_err(|e| e.to_string())?,
            );
            ids.insert(digest.clone(), id);
        }
        let wire = StoreRoot {
            schema: SCHEMA.into(),
            digest: self.digest.clone(),
            root: self.root.clone(),
            pages: ids.clone(),
        };
        let payload = serde_json::to_vec(&wire).map_err(|e| e.to_string())?;
        if payload.len() > MAX_ROOT {
            return Err("foreign.resource.store-root".into());
        }
        let digest = hash(&payload);
        let id = store
            .put_artifact(PutRequest {
                layer: LAYER.id,
                key: content_key(&digest),
                kind: ChunkKind::SnapshotRecord,
                payload,
                edges: ids
                    .values()
                    .map(|id| (id.clone(), RefKind::RequiredData))
                    .collect(),
                project: Some(project.into()),
            })
            .map_err(|e| e.to_string())?;
        drop(leases);
        Ok(Stored {
            artifact_id: id,
            digest,
            pages: ids,
        })
    }
}
fn content_key(digest: &str) -> QueryKey {
    QueryKey::new(&LAYER, vec![("content-sha256".into(), digest.into())])
}

impl Session {
    pub fn restore_stored(
        store: &ManagedStore,
        stored: &Stored,
        target: Target,
        capabilities: &[String],
        environment: AdmittedForeign,
    ) -> Result<Self, String> {
        let root = store
            .get_artifact_by_id_bounded(
                &stored.artifact_id,
                (MAX_ROOT + ace_artifacts::codec::CHUNK_HEADER_LEN) as u64,
            )
            .map_err(|e| e.to_string())?
            .ok_or("foreign.restore.store-root-missing")?;
        if hash(&root.payload) != stored.digest {
            return Err("foreign.restore.store-root-identity".into());
        }
        let wire: StoreRoot = serde_json::from_slice(&root.payload).map_err(|e| e.to_string())?;
        if wire.schema != SCHEMA || wire.pages != stored.pages {
            return Err("foreign.restore.store-index".into());
        }
        let lease = store
            .acquire_lease("source938-restoring", 120, Some(&stored.artifact_id))
            .map_err(|e| e.to_string())?;
        let mut read_error = None;
        let restored = Self::restore(
            &wire.root,
            &wire.digest,
            target,
            capabilities,
            environment,
            |digest| {
                let Some(id) = wire.pages.get(digest) else {
                    return None;
                };
                match store.get_artifact_by_id_bounded(
                    id,
                    (MAX_ARTIFACT + ace_artifacts::codec::CHUNK_HEADER_LEN) as u64,
                ) {
                    Ok(Some(page)) => Some(Arc::new(page.payload)),
                    Ok(None) => None,
                    Err(e) => {
                        read_error = Some(e.to_string());
                        None
                    }
                }
            },
        )
        .map_err(|e| read_error.unwrap_or(e));
        drop(lease);
        restored
    }
    pub fn cold(
        source: &str,
        module: &str,
        package: &str,
        capabilities: &[String],
        environment: AdmittedForeign,
        provider: Arc<Vec<u8>>,
        target: Target,
    ) -> Result<Self, String> {
        admit_provider(&environment, &provider)?;
        let (front, checked) = Frontend::cold(source, module, package, capabilities, environment)
            .map_err(|e| e.to_string())?;
        Self::from_frontend(front, checked, provider, target)
    }
    pub(crate) fn from_frontend(
        front: Frontend,
        checked: ace_core::verify::CheckedModule,
        provider: Arc<Vec<u8>>,
        target: Target,
    ) -> Result<Self, String> {
        admit_provider(front.foreign(), &provider)?;
        let view = SemanticView::from_checked(&checked).map_err(|e| format!("{e:?}"))?;
        let mut artifacts = BTreeMap::new();
        let mut meter = Meter::default();
        let mut per_function = BTreeMap::new();
        let runtime = if target == Target::C {
            let provider = ForeignTargetProvider::new(front.foreign().clone());
            Arc::new(
                provider
                    .runtime_source(&view, Family::Scalar)
                    .map_err(|e| format!("{e:?}"))?
                    .into_bytes(),
            )
        } else {
            Arc::new(ace_a2x_c::foreign::FAILURE_RUNTIME.as_bytes().to_vec())
        };
        match target {
            Target::Native => {
                // The real one-function unit lowering is shared by cold and
                // warm. It reads only the selected body's checked HIR.
                for f in view.functions() {
                    let mut counts = Meter::default();
                    let artifact = native_artifact(
                        checked.bundle(),
                        &checked.bundle().modules[0].defs[f.def_index].id_text,
                        f.source_name,
                        front.capabilities(),
                        front.foreign(),
                        &mut counts,
                    )?;
                    add(&mut meter, counts);
                    per_function.insert(f.source_name.into(), counts);
                    artifacts.insert(f.source_name.into(), artifact);
                }
            }
            Target::C => {
                let provider = ForeignTargetProvider::new(front.foreign().clone());
                for f in view.functions() {
                    let counts = Meter {
                        c_attempts: 1,
                        c_produced: 1,
                        page_encodes: 1,
                        ..Meter::default()
                    };
                    let text = provider
                        .produce_fragments(&view, &[f], Family::Scalar)
                        .map_err(|e| format!("{e:?}"))?
                        .remove(0);
                    let semantic = ace_a2x_core::prepare::semantic_fingerprint(&view, f)
                        .map_err(|e| format!("{e:?}"))?;
                    artifacts.insert(
                        f.source_name.into(),
                        Artifact::new(TargetWire::C {
                            name: f.source_name.into(),
                            def_index: f.def_index,
                            semantic,
                            text,
                        })?,
                    );
                    add(&mut meter, counts);
                    per_function.insert(f.source_name.into(), counts);
                }
            }
        }
        let session = Self {
            front,
            target,
            artifacts,
            runtime,
            provider,
            id: IDS.fetch_add(1, Ordering::Relaxed),
            generation: 0,
            meter,
            per_function,
        };
        session.admit_resident(0)?;
        Ok(session)
    }
    pub fn frontend(&self) -> &Frontend {
        &self.front
    }
    pub fn meter(&self) -> Meter {
        self.meter
    }
    pub fn function_meter(&self, name: &str) -> Meter {
        self.per_function.get(name).copied().unwrap_or_default()
    }
    pub fn artifact_bytes(&self, name: &str) -> Option<&Arc<Vec<u8>>> {
        self.artifacts.get(name).map(|a| &a.bytes)
    }
    pub fn target(&self) -> Target {
        self.target
    }
    fn admit_resident(&self, extra: usize) -> Result<(), String> {
        let resident = self
            .artifacts
            .values()
            .try_fold(extra + self.runtime.len() + self.provider.len(), |n, a| {
                n.checked_add(a.bytes.len())
            })
            .ok_or("foreign.resource.size-overflow")?;
        if resident > MAX_RESIDENT {
            return Err("foreign.resource.resident".into());
        }
        Ok(())
    }
    pub fn prepare(&mut self, name: &str, declaration: &str) -> Result<Candidate, String> {
        let source = self
            .front
            .prepare(name, declaration)
            .map_err(|e| e.to_string())?;
        let counts = self.per_function.entry(name.into()).or_default();
        let before = *counts;
        let result = selected_artifact(
            self.target,
            source.parts(),
            source.checked(),
            name,
            self.front.capabilities(),
            self.front.foreign(),
            counts,
        );
        add(&mut self.meter, difference(*counts, before));
        let artifact = result?;
        self.admit_resident(artifact.bytes.len())?;
        Ok(Candidate {
            source,
            artifact,
            owner: self.id,
            base: self.generation,
        })
    }
    pub fn prepare_source(&mut self, source: &str) -> Result<Candidate, String> {
        let (name, declaration) = self
            .front
            .owner_replacement(source)
            .map_err(|e| e.to_string())?;
        self.prepare(&name, &declaration)
    }
    pub fn validate(&self, candidate: &Candidate) -> Result<(), String> {
        if candidate.owner != self.id || candidate.base != self.generation {
            return Err("foreign.stale-candidate".into());
        }
        self.front
            .validate(&candidate.source)
            .map_err(|e| e.to_string())
    }
    /// Host calls this only after its selected-target publication is ready.
    pub fn commit(&mut self, candidate: Candidate) -> Result<(), String> {
        self.validate(&candidate)?;
        let name = candidate.source.owner().to_string();
        self.front
            .commit(candidate.source)
            .map_err(|e| e.to_string())?;
        self.artifacts.insert(name, candidate.artifact);
        self.generation += 1;
        Ok(())
    }
    pub fn rebind_provider(
        &mut self,
        environment: AdmittedForeign,
        bytes: Arc<Vec<u8>>,
    ) -> Result<(), String> {
        admit_provider(&environment, &bytes)?;
        self.admit_resident(bytes.len())?;
        self.front
            .rebind_provider(environment)
            .map_err(|e| e.to_string())?;
        self.provider = bytes;
        self.generation += 1;
        Ok(())
    }
    /// All affected target bodies are ready before the shared declaration
    /// transaction can be installed. Cancellation is an ordinary candidate drop.
    pub fn prepare_foreign(
        &mut self,
        environment: AdmittedForeign,
        provider: Arc<Vec<u8>>,
    ) -> Result<ForeignCandidate, String> {
        admit_provider(&environment, &provider)?;
        self.admit_resident(provider.len())?;
        let source = self
            .front
            .prepare_foreign(environment)
            .map_err(|e| e.to_string())?;
        let mut artifacts = BTreeMap::new();
        let mut extra = provider.len();
        for body in source.bodies() {
            let name = body.owner();
            let counts = self.per_function.entry(name.into()).or_default();
            let before = *counts;
            let result = selected_artifact(
                self.target,
                body.parts(),
                body.checked(),
                name,
                self.front.capabilities(),
                source.foreign(),
                counts,
            );
            add(&mut self.meter, difference(*counts, before));
            let artifact = result?;
            extra = extra
                .checked_add(artifact.bytes.len())
                .ok_or("foreign.resource.size-overflow")?;
            self.admit_resident(extra)?;
            artifacts.insert(name.into(), artifact);
        }
        Ok(ForeignCandidate {
            source,
            artifacts,
            provider,
            owner: self.id,
            base: self.generation,
        })
    }
    pub fn validate_foreign(&self, candidate: &ForeignCandidate) -> Result<(), String> {
        if candidate.owner != self.id || candidate.base != self.generation {
            return Err("foreign.stale-candidate".into());
        }
        self.front
            .validate_foreign(&candidate.source)
            .map_err(|e| e.to_string())
    }
    pub fn commit_foreign(&mut self, candidate: ForeignCandidate) -> Result<(), String> {
        self.validate_foreign(&candidate)?;
        self.front
            .commit_foreign(candidate.source)
            .map_err(|e| e.to_string())?;
        for (name, artifact) in candidate.artifacts {
            self.artifacts.insert(name, artifact);
        }
        self.provider = candidate.provider;
        self.generation += 1;
        Ok(())
    }
    pub fn snapshot(&self) -> Result<Snapshot, String> {
        let front = self.front.checkpoint().map_err(|e| e.to_string())?;
        let mut pages: BTreeMap<_, _> = front
            .pages
            .values()
            .map(|p| (p.digest.clone(), p.bytes.clone()))
            .collect();
        let front_digest = front.digest;
        pages.insert(front_digest.clone(), Arc::new(front.root));
        for a in self.artifacts.values() {
            pages.insert(a.digest.clone(), a.bytes.clone());
        }
        let runtime = hash(&self.runtime);
        pages.insert(runtime.clone(), self.runtime.clone());
        let provider = hash(&self.provider);
        pages.insert(provider.clone(), self.provider.clone());
        let root = Root {
            schema: SCHEMA.into(),
            target: self.target,
            target_identity: self.target.identity(),
            frontend: front_digest,
            runtime,
            provider,
            provider_digest: self.front.foreign().provider_digest().into(),
            binding_generation: self.front.foreign().generation(),
            contract: self.front.foreign().contract_digest().into(),
            functions: self
                .artifacts
                .iter()
                .map(|(n, a)| (n.clone(), a.digest.clone()))
                .collect(),
        };
        let root = serde_json::to_vec(&root).map_err(|e| e.to_string())?;
        if root.len() > MAX_ROOT {
            return Err("foreign.resource.root".into());
        }
        Ok(Snapshot {
            digest: hash(&root),
            root,
            pages,
        })
    }
    /// Required-page decode/deep validation only. The caller supplies the
    /// accepted root identity and a NEWLY admitted current host binding.
    pub fn restore(
        root: &[u8],
        expected: &str,
        target: Target,
        capabilities: &[String],
        environment: AdmittedForeign,
        mut read: impl FnMut(&str) -> Option<Arc<Vec<u8>>>,
    ) -> Result<Self, String> {
        if root.len() > MAX_ROOT || hash(root) != expected {
            return Err("foreign.restore.root".into());
        }
        let root: Root = serde_json::from_slice(root).map_err(|e| e.to_string())?;
        if root.schema != SCHEMA
            || root.target != target
            || root.target_identity != target.identity()
            || root.provider_digest != environment.provider_digest()
            || root.binding_generation != environment.generation()
            || root.contract != environment.contract_digest()
        {
            return Err("foreign.restore.identity".into());
        }
        let mut bytes_read = 0usize;
        let mut required = |id: &str| -> Result<Arc<Vec<u8>>, String> {
            let bytes = read(id).ok_or("foreign.restore.required-missing")?;
            bytes_read = bytes_read
                .checked_add(bytes.len())
                .ok_or("foreign.resource.size-overflow")?;
            if bytes.len() > MAX_ARTIFACT || bytes_read > MAX_RESIDENT || hash(&bytes) != id {
                return Err("foreign.restore.content-or-budget".into());
            }
            Ok(bytes)
        };
        let provider = required(&root.provider)?;
        admit_provider(&environment, &provider)?;
        let runtime = required(&root.runtime)?;
        let front = required(&root.frontend)?;
        let mut read_error = None;
        let front =
            Frontend::restore(
                &front,
                &root.frontend,
                capabilities,
                environment,
                |id| match required(id) {
                    Ok(bytes) => Some(bytes),
                    Err(e) => {
                        read_error = Some(e);
                        None
                    }
                },
            )
            .map_err(|e| read_error.unwrap_or_else(|| e.to_string()))?;
        let mut artifacts = BTreeMap::new();
        for (name, id) in root.functions {
            if front.page(&name).is_none() {
                return Err("foreign.restore.extra-function".into());
            }
            let bytes = required(&id)?;
            let decoded: TargetWire = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            match &decoded {
                TargetWire::Native {
                    name: stored,
                    symbol,
                    owner_schema,
                    owner_module,
                    mir,
                    coff,
                } if target == Target::Native && stored == &name => {
                    // The shared source adapter uses the stable HIR module
                    // id `m_src`; its logical source path lives separately in
                    // the authenticated frontend root.
                    if owner_schema != ace_av_core::contract::MIR_SCHEMA || owner_module != "m_src"
                    {
                        return Err("foreign.restore.native-owner".into());
                    }
                    // ABC deliberately omits the project/unit owner. Rebind
                    // it from the authenticated required envelope BEFORE MIR
                    // admission, as the existing unit-store protocol does.
                    let mut decoded = ace_av_core::abc::decode_abc(mir)
                        .map_err(|e| format!("foreign.restore.mir: {e:?}"))?;
                    decoded.module.owner = ace_av_core::mir::OwnerScope {
                        schema: owner_schema.clone(),
                        module: owner_module.clone(),
                    };
                    for f in &mut decoded.module.funcs {
                        f.owner = decoded.module.owner.clone();
                    }
                    ace_native::foreign_lower::admit(&decoded.module, front.foreign())?;
                    if decoded.module.funcs.len() != 1
                        || decoded.module.funcs[0].name != format!("d_{name}")
                        || *symbol
                            != ace_native::symbols::function_symbol(
                                &decoded.module.owner,
                                &decoded.module.funcs[0].name,
                            )
                        || coff.len() < 20
                        || coff[..2] != [0x64, 0x86]
                    {
                        return Err("foreign.restore.native-identity".into());
                    }
                }
                TargetWire::C {
                    name: stored, text, ..
                } if target == Target::C && stored == &name => {
                    if !text.starts_with("// origin:") || text.lines().nth(1).is_none() {
                        return Err("foreign.restore.c-fragment".into());
                    }
                }
                _ => return Err("foreign.restore.target-artifact".into()),
            }
            artifacts.insert(
                name,
                Arc::new(Artifact {
                    digest: id,
                    bytes,
                    decoded,
                }),
            );
        }
        if artifacts.len() != front.function_count() {
            return Err("foreign.restore.required-coverage".into());
        }
        let count = artifacts.len() as u64;
        let session = Self {
            front,
            target,
            artifacts,
            runtime,
            provider,
            id: IDS.fetch_add(1, Ordering::Relaxed),
            generation: 0,
            meter: Meter {
                restore_pages: count + 3,
                ..Meter::default()
            },
            per_function: BTreeMap::new(),
        };
        session.admit_resident(0)?;
        Ok(session)
    }
    /// Shared assembly and link/tool work are explicit and separately counted.
    /// All source foreign calls remain in the emitted business functions.
    pub fn run(&mut self, directory: &Path) -> Result<ace_native::link_driver::PeRun, String> {
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        std::fs::write(
            directory.join("source938_events.c"),
            crate::source_provider::EVENT_RUNTIME,
        )
        .map_err(|e| e.to_string())?;
        let provider = directory.join("provider.obj");
        std::fs::write(&provider, &*self.provider).map_err(|e| e.to_string())?;
        let mut objects = vec![provider];
        let sources;
        match self.target {
            Target::Native => {
                let mut entry = None;
                for (i, (name, a)) in self.artifacts.iter().enumerate() {
                    let TargetWire::Native { symbol, coff, .. } = &a.decoded else {
                        return Err("foreign.target".into());
                    };
                    let path = directory.join(format!("caller-{i}.obj"));
                    std::fs::write(&path, coff).map_err(|e| e.to_string())?;
                    objects.push(path);
                    if name == "main" {
                        entry = Some(symbol);
                    }
                }
                let wrapper =
                    ace_native::emit::emit_wrapper_coff(entry.ok_or("foreign.missing-main")?)
                        .map_err(|e| e.to_string())?;
                let path = directory.join("entry-wrapper.obj");
                std::fs::write(&path, wrapper).map_err(|e| e.to_string())?;
                objects.push(path);
                std::fs::write(
                    directory.join("entry.c"),
                    "extern void ac_start(void);\nint main(void){ac_start();return 99;}\n",
                )
                .map_err(|e| e.to_string())?;
                std::fs::write(directory.join("foreign_runtime.c"), &*self.runtime)
                    .map_err(|e| e.to_string())?;
                sources = vec!["entry.c", "foreign_runtime.c", "source938_events.c"];
            }
            Target::C => {
                let fragments = self
                    .artifacts
                    .values()
                    .map(|a| match &a.decoded {
                        TargetWire::C {
                            name,
                            def_index,
                            semantic,
                            text,
                        } => Ok(FragmentText {
                            source_name: name,
                            def_index: *def_index,
                            sem_fp: semantic,
                            text,
                        }),
                        _ => Err("foreign.target".to_string()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let runtime = std::str::from_utf8(&self.runtime).map_err(|e| e.to_string())?;
                let files = ForeignTargetProvider::new(self.front.foreign().clone())
                    .assemble_fileset(FilesetRequest {
                        entry: "main",
                        input_hash: &hash(self.front.source().as_bytes()),
                        family: Family::Scalar,
                        fragments: &fragments,
                        view: None,
                        runtime,
                        prelude_seed: None,
                        origins: None,
                        source_sha256: None,
                    })
                    .map_err(|e| format!("{e:?}"))?;
                for (name, text) in files.files {
                    std::fs::write(directory.join(name), text).map_err(|e| e.to_string())?;
                }
                std::fs::write(directory.join("entry.c"),"#include \"a2x_c_runtime.h\"\nextern a2x_result fn_main(void);\nint main(void){a2x_result v=fn_main();return v.trap?70:v.value;}\n").map_err(|e|e.to_string())?;
                sources = vec![
                    "entry.c",
                    "foreign_runtime.c",
                    "generated.c",
                    "source938_events.c",
                ];
            }
        }
        self.meter.links += 1;
        let exe = directory.join("source.exe");
        crate::bridge::compile_bridge_crt_with_objects(directory, &sources, &objects, &exe)
            .map_err(|e| e.to_string())?;
        ace_native::link_driver::run_pe(
            &exe,
            crate::bridge::BRIDGE_RUN_DEADLINE,
            ace_core::link::CaptureCaps {
                stdout_max: 1 << 20,
                stderr_max: 1 << 20,
            },
        )
        .map_err(|e| e.to_string())
    }
}
fn native_artifact(
    bundle: &ace_core::hir::Bundle,
    id: &str,
    name: &str,
    caps: &[String],
    env: &AdmittedForeign,
    meter: &mut Meter,
) -> Result<Arc<Artifact>, String> {
    meter.mir_attempts += 1;
    let mir = ace_lowering::hir_adapter::lower_function_unit(bundle, id, caps, 0)
        .map_err(|e| format!("foreign.lower: {e:?}"))?;
    ace_native::foreign_lower::admit(&mir, env)?;
    meter.mir_produced += 1;
    meter.native_attempts += 1;
    let object = ace_native::emit::emit_foreign_unit_coff(&mir, env).map_err(|e| e.to_string())?;
    meter.native_produced += 1;
    let owner_schema = mir.owner.schema.clone();
    let owner_module = mir.owner.module.clone();
    let mir = ace_av_core::abc::encode_abc(&mir)
        .map_err(|e| format!("foreign.mir-encode: {e:?}"))?
        .to_flat();
    meter.page_encodes += 1;
    Artifact::new(TargetWire::Native {
        name: name.into(),
        symbol: object.symbol,
        owner_schema,
        owner_module,
        mir,
        coff: object.bytes,
    })
}
fn admit_provider(env: &AdmittedForeign, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_ARTIFACT || blake3::hash(bytes).to_hex().as_str() != env.provider_digest()
    {
        return Err("foreign.provider-content".into());
    }
    use object::{Object, ObjectSymbol};
    let object = object::File::parse(bytes).map_err(|e| format!("foreign.provider-object: {e}"))?;
    if object.format() != object::BinaryFormat::Coff
        || object.architecture() != object::Architecture::X86_64
        || object.kind() != object::ObjectKind::Relocatable
    {
        return Err("foreign.provider-profile: expected AMD64 relocatable COFF".into());
    }
    let declarations = env.source_declarations();
    let needed = declarations
        .iter()
        .flat_map(|d| {
            std::iter::once(d.function.symbol.as_str())
                .chain(d.context_symbol.iter().map(|s| s.as_str()))
        })
        .collect::<std::collections::BTreeSet<_>>();
    let definitions = object
        .symbols()
        .filter(|s| s.is_definition() && s.is_global())
        .filter_map(|s| {
            s.name()
                .ok()
                .filter(|name| needed.contains(name))
                .map(|name| (name, s.kind()))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    for declaration in &declarations {
        for (symbol, kind) in std::iter::once((
            declaration.function.symbol.as_str(),
            object::SymbolKind::Text,
        ))
        .chain(
            declaration
                .context_symbol
                .iter()
                .map(|name| (name.as_str(), object::SymbolKind::Data)),
        ) {
            let Some(actual) = definitions.get(symbol) else {
                return Err(format!("foreign.provider-symbol: reviewed symbol `{symbol}` is absent from the actual provider object"));
            };
            if *actual != kind {
                return Err(format!(
                    "foreign.provider-symbol-kind: `{symbol}` is {actual:?}, expected {kind:?}"
                ));
            }
        }
    }
    Ok(())
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Root {
    schema: String,
    target: Target,
    target_identity: String,
    frontend: String,
    runtime: String,
    provider: String,
    provider_digest: String,
    binding_generation: u64,
    contract: String,
    functions: BTreeMap<String, String>,
}
fn add(a: &mut Meter, b: Meter) {
    a.mir_attempts += b.mir_attempts;
    a.mir_produced += b.mir_produced;
    a.native_attempts += b.native_attempts;
    a.native_produced += b.native_produced;
    a.c_attempts += b.c_attempts;
    a.c_produced += b.c_produced;
    a.page_encodes += b.page_encodes;
    a.restore_pages += b.restore_pages;
    a.links += b.links;
}
fn difference(a: Meter, b: Meter) -> Meter {
    Meter {
        mir_attempts: a.mir_attempts - b.mir_attempts,
        mir_produced: a.mir_produced - b.mir_produced,
        native_attempts: a.native_attempts - b.native_attempts,
        native_produced: a.native_produced - b.native_produced,
        c_attempts: a.c_attempts - b.c_attempts,
        c_produced: a.c_produced - b.c_produced,
        page_encodes: a.page_encodes - b.page_encodes,
        restore_pages: a.restore_pages - b.restore_pages,
        links: a.links - b.links,
    }
}

fn selected_artifact(
    target: Target,
    parts: &ace_core::source::SourceParts,
    checked: &ace_core::verify::CheckedFunction,
    name: &str,
    capabilities: &[String],
    environment: &AdmittedForeign,
    counts: &mut Meter,
) -> Result<Arc<Artifact>, String> {
    match target {
        Target::Native => native_artifact(
            &parts.bundle,
            checked.def_id_text(),
            name,
            capabilities,
            environment,
            counts,
        ),
        Target::C => {
            counts.c_attempts += 1;
            let view = SemanticView::from_parts(parts, checked).map_err(|e| format!("{e:?}"))?;
            let f = view.functions().first().ok_or("foreign.empty-fragment")?;
            let provider = ForeignTargetProvider::new(environment.clone());
            let text = provider
                .produce_fragments(&view, &[f], Family::Scalar)
                .map_err(|e| format!("{e:?}"))?
                .remove(0);
            counts.c_produced += 1;
            counts.page_encodes += 1;
            let semantic = ace_a2x_core::prepare::semantic_fingerprint(&view, f)
                .map_err(|e| format!("{e:?}"))?;
            Artifact::new(TargetWire::C {
                name: name.into(),
                def_index: f.def_index,
                semantic,
                text,
            })
        }
    }
}
