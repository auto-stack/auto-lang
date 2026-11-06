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
use ace_foreign_contract::manual::{AdmittedForeign, ForeignPages, HostAuthority};
use ace_source_query::frontend::{
    Candidate as SourceCandidate, ForeignCandidate as SourceForeignCandidate, Frontend,
};
use ace_source_query::resources::{Charge, Reservation, Resources};
use ace_source_query::work::{Ledger, Phase, Work};
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
    pub(crate) fn admit_source_bytes(self, bytes: usize) -> Result<(), String> {
        let limit = match self {
            Self::Native => ace_native::config::MAX_INPUT_BYTES,
            Self::C => ace_source_query::frontend::MAX_SOURCE_BYTES,
        };
        if bytes > limit {
            return Err(format!(
                "foreign.resource.source: {bytes} source bytes exceed {limit} for {self:?}"
            ));
        }
        Ok(())
    }
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
    pub cache_hits: u64,
    pub cache_lookups: u64,
    pub cache_writes: u64,
    pub cache_bytes_read: u64,
    pub cache_bytes_written: u64,
    pub cache_evictions: u64,
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
        semantic: String,
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
    fn semantic(&self) -> &str {
        match &self.decoded {
            TargetWire::Native { semantic, .. } | TargetWire::C { semantic, .. } => semantic,
        }
    }
    fn decoded_bytes(decoded: &TargetWire) -> usize {
        std::mem::size_of::<Self>()
            + 2 * std::mem::size_of::<usize>()
            + 64
            + match decoded {
                TargetWire::Native {
                    name,
                    symbol,
                    owner_schema,
                    owner_module,
                    semantic,
                    mir,
                    coff,
                } => {
                    name.capacity()
                        + symbol.capacity()
                        + owner_schema.capacity()
                        + owner_module.capacity()
                        + semantic.capacity()
                        + mir.capacity()
                        + coff.capacity()
                }
                TargetWire::C {
                    name,
                    semantic,
                    text,
                    ..
                } => name.capacity() + semantic.capacity() + text.capacity(),
            }
    }
    fn new(decoded: TargetWire, resources: &Resources) -> Result<Arc<Self>, String> {
        Self::new_reserved(decoded, resources, None)
    }
    fn new_reserved(
        decoded: TargetWire,
        resources: &Resources,
        previous: Option<Reservation>,
    ) -> Result<Arc<Self>, String> {
        let charge = Charge::resident(Self::decoded_bytes(&decoded));
        let mut decoded_charge = if let Some(mut previous) = previous {
            previous.resize(charge).map_err(|e| e.to_string())?;
            previous
        } else {
            resources.reserve(charge).map_err(|e| e.to_string())?
        };
        let size = ace_foreign_contract::codec::json_size(&decoded, MAX_ARTIFACT)
            .map_err(|e| e.to_string())?;
        let production = resources.reserve_output(size).map_err(|e| e.to_string())?;
        let mut bytes_charge = resources
            .reserve(Charge::resident(
                size + std::mem::size_of::<Vec<u8>>() + 2 * std::mem::size_of::<usize>(),
            ))
            .map_err(|e| e.to_string())?;
        let bytes = ace_foreign_contract::codec::encode_json_bounded(&decoded, MAX_ARTIFACT)
            .map_err(|e| e.to_string())?;
        production
            .complete(bytes.len())
            .map_err(|e| e.to_string())?;
        if bytes.len() > MAX_ARTIFACT {
            return Err("foreign.resource.artifact".into());
        }
        let bytes = Arc::new(bytes);
        bytes_charge.try_retain(&bytes).map_err(|e| e.to_string())?;
        let artifact = Arc::new(Self {
            digest: hash(&bytes),
            bytes,
            decoded,
        });
        decoded_charge
            .try_retain(&artifact)
            .map_err(|e| e.to_string())?;
        Ok(artifact)
    }
}

pub struct Session {
    front: Frontend,
    target: Target,
    artifacts: BTreeMap<String, Arc<Artifact>>,
    runtime: Arc<Vec<u8>>,
    provider: Arc<Vec<u8>>,
    runtime_identity: String,
    runtime_page_id: String,
    provider_page_id: String,
    id: u64,
    generation: u64,
    meter: Meter,
    per_function: BTreeMap<String, Meter>,
    cache: function_cache::FunctionCache,
    root_lease: Option<ace_artifacts::store::Lease>,
    publication_maintenance: PublicationMaintenance,
}
/// Publication has already committed when post-switch root retirement runs.
/// Maintenance failure is explicit and retryable, never an ambiguous rejected
/// candidate after the accepted pointer has changed.
#[derive(Clone, Debug, Default, Serialize)]
pub struct PublicationMaintenance {
    pub roots_retired: usize,
    pub temp_files_removed: usize,
    pub temp_bytes_removed: u64,
    pub cleanup_error: Option<String>,
}
#[derive(Debug)]
pub enum PrepareError {
    Source(ace_source_query::frontend::Error),
    Target(String),
}
impl std::fmt::Display for PrepareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(e) => e.fmt(f),
            Self::Target(e) => f.write_str(e),
        }
    }
}
impl std::error::Error for PrepareError {}
pub struct Candidate {
    source: SourceCandidate,
    artifact: Arc<Artifact>,
    owner: u64,
    base: u64,
    work: Vec<Work>,
}
pub struct ForeignCandidate {
    source: SourceForeignCandidate,
    artifacts: BTreeMap<String, Arc<Artifact>>,
    provider: Arc<Vec<u8>>,
    owner: u64,
    base: u64,
    work: Vec<Work>,
}
impl ForeignCandidate {
    pub fn include_interface_work(mut self, work: Vec<Work>) -> Self {
        self.work.extend(work);
        self
    }
}
pub struct Snapshot {
    pub root: Vec<u8>,
    pub digest: String,
    pub pages: BTreeMap<String, Arc<Vec<u8>>>,
    work: Ledger,
    page_owners: BTreeMap<String, String>,
    resources: Resources,
    _root_charge: Reservation,
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
#[derive(Serialize)]
struct StoreRootRef<'a> {
    schema: &'a str,
    digest: &'a str,
    root: &'a [u8],
    pages: &'a BTreeMap<String, String>,
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
        let (stored, lease) = self.persist_leased(store, previous)?;
        store
            .register_root(project, &stored.artifact_id, "soft")
            .map_err(|e| e.to_string())?;
        drop(lease);
        Ok(stored)
    }
    fn persist_leased(
        &self,
        store: &ManagedStore,
        previous: Option<&Stored>,
    ) -> Result<(Stored, ace_artifacts::store::Lease), String> {
        let mut ids = BTreeMap::new();
        let mut leases = Vec::new();
        if let Some(previous) = previous {
            leases.push(
                store
                    .acquire_lease(
                        "source938-previous-closure",
                        120,
                        Some(&previous.artifact_id),
                    )
                    .map_err(|e| e.to_string())?,
            );
        }
        for (digest, bytes) in &self.pages {
            let owner = self
                .page_owners
                .get(digest)
                .map(String::as_str)
                .unwrap_or("*");
            if let Some(id) = previous.and_then(|p| p.pages.get(digest)) {
                self.work.reuse(owner, "store-leaf", 1);
                ids.insert(digest.clone(), id.clone());
                continue;
            }
            let _copy_charge = self
                .resources
                .reserve(Charge::io(
                    (bytes.len() + ace_artifacts::codec::CHUNK_HEADER_LEN)
                        .checked_mul(3)
                        .ok_or("foreign.resource.copy-size")?,
                ))
                .map_err(|e| e.to_string())?;
            let put = self.work.begin(owner, "store-leaf", Phase::Metadata);
            self.work.io(owner, "store-leaf", 0, bytes.len() as u64);
            let (id, lease) = store
                .put_artifact_leased(
                    PutRequest {
                        layer: LAYER.id,
                        key: content_key(digest),
                        kind: ChunkKind::SnapshotRecord,
                        payload: bytes.as_ref().clone(),
                        edges: vec![],
                        project: None,
                    },
                    "source938-candidate-page",
                    120,
                )
                .map_err(|e| e.to_string())?;
            put.accept();
            leases.push(lease);
            ids.insert(digest.clone(), id);
        }
        let root_work = self.work.begin("*", "store-root", Phase::Metadata);
        let wire = StoreRootRef {
            schema: SCHEMA,
            digest: &self.digest,
            root: &self.root,
            pages: &ids,
        };
        let size =
            ace_foreign_contract::codec::json_size(&wire, MAX_ROOT).map_err(|e| e.to_string())?;
        let production = self
            .resources
            .reserve_output(size)
            .map_err(|e| e.to_string())?;
        let _root_copy_charge = self
            .resources
            .reserve(Charge::io(
                size.checked_mul(3)
                    .ok_or("foreign.resource.root-copy-size")?,
            ))
            .map_err(|e| e.to_string())?;
        let payload = ace_foreign_contract::codec::encode_json_bounded(&wire, MAX_ROOT)
            .map_err(|e| e.to_string())?;
        production
            .complete(payload.len())
            .map_err(|e| e.to_string())?;
        if payload.len() > MAX_ROOT {
            return Err("foreign.resource.store-root".into());
        }
        let digest = hash(&payload);
        self.work.io("*", "store-root", 0, payload.len() as u64);
        let (id, lease) = store
            .put_artifact_leased(
                PutRequest {
                    layer: LAYER.id,
                    key: content_key(&digest),
                    kind: ChunkKind::SnapshotRecord,
                    payload,
                    edges: ids
                        .values()
                        .map(|id| (id.clone(), RefKind::RequiredData))
                        .collect(),
                    project: None,
                },
                "source938-prepared-closure",
                120,
            )
            .map_err(|e| e.to_string())?;
        root_work.accept();
        drop(leases);
        Ok((
            Stored {
                artifact_id: id,
                digest,
                pages: ids,
            },
            lease,
        ))
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
        Self::restore_stored_input(
            store,
            stored,
            target,
            capabilities,
            EnvironmentInput::Admitted(environment),
            Resources::default(),
        )
    }
    pub fn restore_stored_host(
        store: &ManagedStore,
        stored: &Stored,
        target: Target,
        capabilities: &[String],
        host: HostAuthority,
    ) -> Result<Self, String> {
        Self::restore_stored_host_with_resources(
            store,
            stored,
            target,
            capabilities,
            host,
            Resources::default(),
        )
    }
    pub fn restore_stored_host_with_resources(
        store: &ManagedStore,
        stored: &Stored,
        target: Target,
        capabilities: &[String],
        host: HostAuthority,
        resources: Resources,
    ) -> Result<Self, String> {
        Self::restore_stored_input(
            store,
            stored,
            target,
            capabilities,
            EnvironmentInput::Host(host),
            resources,
        )
    }
    fn restore_stored_input(
        store: &ManagedStore,
        stored: &Stored,
        target: Target,
        capabilities: &[String],
        environment: EnvironmentInput,
        resources: Resources,
    ) -> Result<Self, String> {
        // Lease BEFORE the first payload read, so GC cannot retire the root
        // while the decoder is traversing its required closure.
        let lease = store
            .acquire_lease("source938-restoring", 120, Some(&stored.artifact_id))
            .map_err(|e| e.to_string())?;
        let root = resources
            .load_payload(MAX_ROOT, ChunkKind::SnapshotRecord, |bound| {
                store.get_artifact_by_id_bounded(&stored.artifact_id, bound)
            })
            .map_err(|e| e.to_string())?
            .ok_or("foreign.restore.store-root-missing")?;
        if hash(&root) != stored.digest {
            return Err("foreign.restore.store-root-identity".into());
        }
        let (wire, _wire_heap): (StoreRoot, _) =
            resources.decode_json(&root).map_err(|e| e.to_string())?;
        if wire.schema != SCHEMA || wire.pages != stored.pages {
            return Err("foreign.restore.store-index".into());
        }
        let mut read_error = None;
        let restored = Self::restore_with_resources(
            &wire.root,
            &wire.digest,
            target,
            capabilities,
            environment,
            |digest| {
                let Some(id) = wire.pages.get(digest) else {
                    return None;
                };
                match resources.load_payload(MAX_ARTIFACT, ChunkKind::SnapshotRecord, |bound| {
                    store.get_artifact_by_id_bounded(id, bound)
                }) {
                    Ok(Some(page)) => Some(page),
                    Ok(None) => None,
                    Err(e) => {
                        read_error = Some(e.to_string());
                        None
                    }
                }
            },
            resources.clone(),
        )
        .map_err(|e| read_error.unwrap_or(e));
        let mut restored = restored?;
        restored
            .cache
            .attach(&store.root, restored.front.foreign())?;
        restored.root_lease = Some(lease);
        Ok(restored)
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
        target.admit_source_bytes(source.len())?;
        let resources = Resources::default();
        admit_provider(&environment, &provider, &resources)?;
        let (front, checked) = Frontend::cold_observed_with_resources(
            source,
            module,
            package,
            capabilities,
            environment,
            &mut ace_source_query::frontend::ColdObservation::default(),
            resources,
        )
        .map_err(|e| e.to_string())?;
        Self::from_frontend(front, checked, provider, target)
    }
    pub fn cold_stored(
        source: &str,
        module: &str,
        package: &str,
        capabilities: &[String],
        environment: AdmittedForeign,
        provider: Arc<Vec<u8>>,
        target: Target,
        store: &ManagedStore,
    ) -> Result<Self, String> {
        target.admit_source_bytes(source.len())?;
        let resources = Resources::default();
        admit_provider(&environment, &provider, &resources)?;
        let (front, checked) = Frontend::cold_observed_with_resources(
            source,
            module,
            package,
            capabilities,
            environment,
            &mut ace_source_query::frontend::ColdObservation::default(),
            resources,
        )
        .map_err(|e| e.to_string())?;
        Self::from_frontend_stored(front, checked, provider, target, store)
    }
    pub(crate) fn from_frontend_stored(
        front: Frontend,
        checked: ace_core::verify::CheckedModule,
        provider: Arc<Vec<u8>>,
        target: Target,
        store: &ManagedStore,
    ) -> Result<Self, String> {
        let mut cache = function_cache::FunctionCache::new(
            front.package(),
            front.module(),
            front.work(),
            front.resources(),
        )?;
        cache.attach(&store.root, front.foreign())?;
        Self::from_frontend_cached(front, checked, provider, target, cache)
    }
    pub(crate) fn from_frontend(
        front: Frontend,
        checked: ace_core::verify::CheckedModule,
        provider: Arc<Vec<u8>>,
        target: Target,
    ) -> Result<Self, String> {
        let cache = function_cache::FunctionCache::new(
            front.package(),
            front.module(),
            front.work(),
            front.resources(),
        )?;
        Self::from_frontend_cached(front, checked, provider, target, cache)
    }
    fn from_frontend_cached(
        front: Frontend,
        checked: ace_core::verify::CheckedModule,
        provider: Arc<Vec<u8>>,
        target: Target,
        mut cache: function_cache::FunctionCache,
    ) -> Result<Self, String> {
        target.admit_source_bytes(front.source().len())?;
        admit_provider(front.foreign(), &provider, &front.resources())?;
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
        let runtime_sha = hash(&runtime);
        cache
            .resources
            .adopt(&runtime, Charge::resident(runtime.capacity() + 40))
            .map_err(|e| e.to_string())?;
        cache
            .resources
            .adopt(&provider, Charge::resident(provider.capacity() + 40))
            .map_err(|e| e.to_string())?;
        let mut work = Vec::new();
        for f in view.functions() {
            let mut counts = Meter::default();
            let artifact = cached_artifact(
                target,
                checked.bundle(),
                &checked.bundle().modules[0].defs[f.def_index].id_text,
                f.source_name,
                front.capabilities(),
                front.foreign(),
                &view,
                f,
                &mut cache,
                &runtime_sha,
                &mut counts,
                &mut work,
            )?;
            add(&mut meter, counts);
            per_function.insert(f.source_name.into(), counts);
            artifacts.insert(f.source_name.into(), artifact);
        }
        let session = Self {
            front,
            target,
            artifacts,
            runtime_identity: runtime_identity(target, &hash(&runtime)),
            runtime_page_id: hash(&runtime),
            provider_page_id: hash(&provider),
            runtime,
            provider,
            id: IDS.fetch_add(1, Ordering::Relaxed),
            generation: 0,
            meter,
            per_function,
            cache,
            root_lease: None,
            publication_maintenance: PublicationMaintenance::default(),
        };
        session.admit_resident(0)?;
        for output in work {
            output.accept();
        }
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
        self.prepare_checked(name, declaration)
            .map_err(|e| e.to_string())
    }
    pub fn prepare_checked(
        &mut self,
        name: &str,
        declaration: &str,
    ) -> Result<Candidate, PrepareError> {
        let bytes = self
            .front
            .replacement_source_bytes(name, declaration.len())
            .map_err(PrepareError::Source)?;
        self.target
            .admit_source_bytes(bytes)
            .map_err(PrepareError::Target)?;
        let source = self
            .front
            .prepare(name, declaration)
            .map_err(PrepareError::Source)?;
        self.finish_candidate(name, source)
            .map_err(PrepareError::Target)
    }
    fn finish_candidate(
        &mut self,
        name: &str,
        source: SourceCandidate,
    ) -> Result<Candidate, String> {
        let counts = self.per_function.entry(name.into()).or_default();
        let before = *counts;
        let mut work = Vec::new();
        let result = selected_artifact(
            self.target,
            source.parts(),
            source.checked(),
            name,
            self.front.capabilities(),
            self.front.foreign(),
            &mut self.cache,
            &self.runtime_page_id,
            counts,
            &mut work,
        );
        add(&mut self.meter, difference(*counts, before));
        let artifact = result?;
        self.admit_resident(artifact.bytes.len())?;
        Ok(Candidate {
            source,
            artifact,
            owner: self.id,
            base: self.generation,
            work,
        })
    }
    pub fn prepare_source(&mut self, source: &str) -> Result<Candidate, String> {
        self.prepare_source_checked(source)
            .map_err(|e| e.to_string())
    }
    pub fn prepare_source_checked(&mut self, source: &str) -> Result<Candidate, PrepareError> {
        self.target
            .admit_source_bytes(source.len())
            .map_err(PrepareError::Target)?;
        let (name, declaration) = self
            .front
            .owner_replacement(source)
            .map_err(PrepareError::Source)?;
        self.prepare_checked(&name, &declaration)
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
        for output in candidate.work {
            output.accept();
        }
        self.generation += 1;
        Ok(())
    }
    pub fn rebind_provider(
        &mut self,
        environment: AdmittedForeign,
        bytes: Arc<Vec<u8>>,
    ) -> Result<(), String> {
        admit_provider(&environment, &bytes, &self.front.resources())?;
        self.admit_resident(bytes.len())?;
        self.front
            .rebind_provider(environment)
            .map_err(|e| e.to_string())?;
        self.provider_page_id = hash(&bytes);
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
        self.prepare_foreign_source(environment, provider, None)
    }
    pub fn prepare_foreign_source(
        &mut self,
        environment: AdmittedForeign,
        provider: Arc<Vec<u8>>,
        source: Option<&str>,
    ) -> Result<ForeignCandidate, String> {
        self.prepare_foreign_source_checked(environment, provider, source)
            .map_err(|e| e.to_string())
    }
    pub fn prepare_foreign_source_checked(
        &mut self,
        environment: AdmittedForeign,
        provider: Arc<Vec<u8>>,
        source: Option<&str>,
    ) -> Result<ForeignCandidate, PrepareError> {
        self.target
            .admit_source_bytes(source.map_or(self.front.source().len(), str::len))
            .map_err(PrepareError::Target)?;
        admit_provider(&environment, &provider, &self.front.resources())
            .map_err(PrepareError::Target)?;
        self.admit_resident(provider.len())
            .map_err(PrepareError::Target)?;
        let source = self
            .front
            .prepare_foreign_source(environment, source)
            .map_err(PrepareError::Source)?;
        self.finish_foreign_candidate(source, provider)
            .map_err(PrepareError::Target)
    }
    fn finish_foreign_candidate(
        &mut self,
        source: SourceForeignCandidate,
        provider: Arc<Vec<u8>>,
    ) -> Result<ForeignCandidate, String> {
        let mut artifacts = BTreeMap::new();
        let mut extra = provider.len();
        let mut work = Vec::new();
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
                &mut self.cache,
                &self.runtime_page_id,
                counts,
                &mut work,
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
            work,
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
        self.provider_page_id = hash(&candidate.provider);
        self.provider = candidate.provider;
        for output in candidate.work {
            output.accept();
        }
        self.generation += 1;
        Ok(())
    }
    pub fn snapshot(&self) -> Result<Snapshot, String> {
        self.snapshot_view(
            self.front.checkpoint().map_err(|e| e.to_string())?,
            self.front.foreign(),
            &self.provider,
            &self.provider_page_id,
            &BTreeMap::new(),
        )
    }
    fn snapshot_view(
        &self,
        front: ace_source_query::frontend::Checkpoint,
        environment: &AdmittedForeign,
        provider: &Arc<Vec<u8>>,
        provider_id: &str,
        overrides: &BTreeMap<String, Arc<Artifact>>,
    ) -> Result<Snapshot, String> {
        let work = self.front.work();
        let checkpoint = work.begin("*", "target-checkpoint", Phase::Metadata);
        let mut page_owners: BTreeMap<_, _> = front
            .pages
            .iter()
            .map(|(name, page)| (page.digest.clone(), name.clone()))
            .collect();
        let mut pages: BTreeMap<_, _> = front
            .pages
            .values()
            .map(|p| (p.digest.clone(), p.bytes.clone()))
            .collect();
        let front_digest = front.digest;
        let resources = self.front.resources();
        let _arc_header = resources
            .reserve(Charge::io(16))
            .map_err(|e| e.to_string())?;
        let front_root = Arc::new(front.root);
        resources
            .adopt(&front_root, Charge::resident(front_root.capacity() + 40))
            .map_err(|e| e.to_string())?;
        pages.insert(front_digest.clone(), front_root);
        let mut functions = BTreeMap::new();
        for (name, artifact) in &self.artifacts {
            let artifact = overrides.get(name).unwrap_or(artifact);
            page_owners.insert(artifact.digest.clone(), name.clone());
            pages.insert(artifact.digest.clone(), artifact.bytes.clone());
            functions.insert(name.clone(), artifact.digest.clone());
        }
        pages.insert(self.runtime_page_id.clone(), self.runtime.clone());
        pages.insert(provider_id.into(), provider.clone());
        let foreign = ForeignRoot::from_pages(environment.pages());
        for (stage, page) in environment.pages().iter() {
            let digest = hash(page);
            page_owners.insert(digest.clone(), stage.into());
            pages.insert(digest, page.clone());
        }
        for page in pages.values() {
            resources
                .adopt(page, Charge::resident(page.capacity() + 40))
                .map_err(|e| e.to_string())?;
        }
        let root = Root {
            schema: SCHEMA.into(),
            target: self.target,
            target_identity: self.target.identity(),
            frontend: front_digest,
            runtime: self.runtime_page_id.clone(),
            runtime_identity: self.runtime_identity.clone(),
            provider: provider_id.into(),
            provider_digest: environment.provider_digest().into(),
            binding_generation: environment.generation(),
            contract: environment.contract_digest().into(),
            foreign,
            functions,
        };
        let size =
            ace_foreign_contract::codec::json_size(&root, MAX_ROOT).map_err(|e| e.to_string())?;
        let production = resources.reserve_output(size).map_err(|e| e.to_string())?;
        let root_charge = resources
            .reserve(Charge::io(size + std::mem::size_of::<Vec<u8>>()))
            .map_err(|e| e.to_string())?;
        let root = ace_foreign_contract::codec::encode_json_bounded(&root, MAX_ROOT)
            .map_err(|e| e.to_string())?;
        production.complete(root.len()).map_err(|e| e.to_string())?;
        if root.len() > MAX_ROOT {
            return Err("foreign.resource.root".into());
        }
        checkpoint.accept();
        Ok(Snapshot {
            digest: hash(&root),
            root,
            pages,
            work,
            page_owners,
            resources,
            _root_charge: root_charge,
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
        read: impl FnMut(&str) -> Option<Arc<Vec<u8>>>,
    ) -> Result<Self, String> {
        Self::restore_with_resources(
            root,
            expected,
            target,
            capabilities,
            EnvironmentInput::Admitted(environment),
            read,
            Resources::default(),
        )
    }
    pub fn restore_host(
        root: &[u8],
        expected: &str,
        target: Target,
        capabilities: &[String],
        host: HostAuthority,
        read: impl FnMut(&str) -> Option<Arc<Vec<u8>>>,
    ) -> Result<Self, String> {
        Self::restore_host_with_resources(
            root,
            expected,
            target,
            capabilities,
            host,
            read,
            Resources::default(),
        )
    }
    /// Host-owned live account, shared with request decoding and other retained
    /// compiler state. Limits may only tighten the original partitions.
    pub fn restore_host_with_resources(
        root: &[u8],
        expected: &str,
        target: Target,
        capabilities: &[String],
        host: HostAuthority,
        read: impl FnMut(&str) -> Option<Arc<Vec<u8>>>,
        resources: Resources,
    ) -> Result<Self, String> {
        Self::restore_with_resources(
            root,
            expected,
            target,
            capabilities,
            EnvironmentInput::Host(host),
            read,
            resources,
        )
    }
    fn restore_with_resources(
        root: &[u8],
        expected: &str,
        target: Target,
        capabilities: &[String],
        environment: EnvironmentInput,
        mut read: impl FnMut(&str) -> Option<Arc<Vec<u8>>>,
        resources: Resources,
    ) -> Result<Self, String> {
        let work = Ledger::default();
        let recovery = work.begin("*", "restore-target-root", Phase::Recovery);
        work.io("*", "restore-target-root", root.len() as u64, 0);
        if root.len() > MAX_ROOT || hash(root) != expected {
            return Err("foreign.restore.root".into());
        }
        let (root, _root_heap): (Root, _) =
            resources.decode_json(root).map_err(|e| e.to_string())?;
        let (contract, provider_digest, generation) = match &environment {
            EnvironmentInput::Admitted(e) => {
                (e.contract_digest(), e.provider_digest(), e.generation())
            }
            EnvironmentInput::Host(h) => (
                h.contract_digest.as_str(),
                h.provider_digest.as_str(),
                h.binding_generation,
            ),
        };
        if root.schema != SCHEMA
            || root.target != target
            || root.target_identity != target.identity()
            || root.provider_digest != provider_digest
            || root.binding_generation != generation
            || root.contract != contract
            || root.runtime_identity != runtime_identity(target, &root.runtime)
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
            resources
                .adopt(&bytes, Charge::resident(bytes.capacity() + 40))
                .map_err(|e| e.to_string())?;
            Ok(bytes)
        };
        let mut environment_pages = Vec::new();
        for (stage, id) in root.foreign.iter() {
            let ticket = work.begin("*", stage, Phase::Recovery);
            let page = required(id)?;
            work.io("*", stage, page.len() as u64, 0);
            environment_pages.push(page);
            ticket.accept();
        }
        let [logical, physical, declarations, binding]: [Arc<Vec<u8>>; 4] = environment_pages
            .try_into()
            .map_err(|_| "foreign.restore.interface-coverage")?;
        let pages = ForeignPages {
            logical,
            physical,
            declarations,
            binding,
        };
        let environment = match environment {
            EnvironmentInput::Admitted(current) => current.restore_required(pages)?,
            EnvironmentInput::Host(host) => {
                let mut allocations = Vec::new();
                for (_, page) in pages.iter() {
                    allocations.push(
                        resources
                            .reserve_foreign_decode(page)
                            .map_err(|e| e.to_string())?,
                    );
                }
                let admitted = AdmittedForeign::restore_pages(pages, &host)?;
                for allocation in allocations {
                    allocation
                        .retain_foreign_metadata(admitted.allocation_identity())
                        .map_err(|e| e.to_string())?;
                }
                admitted
            }
        };
        let provider = required(&root.provider)?;
        let qualification = work.begin("*", "restore-provider", Phase::Recovery);
        work.io("*", "restore-provider", provider.len() as u64, 0);
        admit_provider(&environment, &provider, &resources)?;
        qualification.accept();
        let runtime = required(&root.runtime)?;
        work.io("*", "restore-runtime", runtime.len() as u64, 0);
        let front = required(&root.frontend)?;
        let mut read_error = None;
        let front = Frontend::restore_with_resources(
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
            work.clone(),
            resources.clone(),
        )
        .map_err(|e| read_error.unwrap_or_else(|| e.to_string()))?;
        target.admit_source_bytes(front.source().len())?;
        let mut artifacts = BTreeMap::new();
        for (name, id) in root.functions {
            let page_work = work.begin(&name, "restore-target", Phase::Recovery);
            if front.page(&name).is_none() {
                return Err("foreign.restore.extra-function".into());
            }
            let bytes = required(&id)?;
            work.io(&name, "restore-target", bytes.len() as u64, 0);
            let (decoded, _decoded_heap): (TargetWire, _) =
                resources.decode_json(&bytes).map_err(|e| e.to_string())?;
            validate_target_artifact(target, &name, &decoded, front.foreign(), &resources)?;
            page_work.accept();
            let mut allocation = resources
                .reserve(Charge::resident(Artifact::decoded_bytes(&decoded)))
                .map_err(|e| e.to_string())?;
            let artifact = Arc::new(Artifact {
                digest: id,
                bytes,
                decoded,
            });
            allocation
                .try_retain(&artifact)
                .map_err(|e| e.to_string())?;
            artifacts.insert(name, artifact);
        }
        if artifacts.len() != front.function_count() {
            return Err("foreign.restore.required-coverage".into());
        }
        let cache = function_cache::FunctionCache::new(
            front.package(),
            front.module(),
            front.work(),
            front.resources(),
        )?;
        let count = artifacts.len() as u64;
        let session = Self {
            front,
            target,
            artifacts,
            runtime_identity: root.runtime_identity,
            runtime_page_id: root.runtime,
            provider_page_id: root.provider,
            runtime,
            provider,
            id: IDS.fetch_add(1, Ordering::Relaxed),
            generation: 0,
            meter: Meter {
                restore_pages: count + 7,
                ..Meter::default()
            },
            per_function: BTreeMap::new(),
            cache,
            root_lease: None,
            publication_maintenance: PublicationMaintenance::default(),
        };
        session.admit_resident(0)?;
        recovery.accept();
        Ok(session)
    }
    // Publication links a borrowed overlay before installing any source,
    // declaration, provider binding or target root.
}

fn link_artifacts(
    directory: &Path,
    target: Target,
    source: &str,
    environment: &AdmittedForeign,
    provider: &Arc<Vec<u8>>,
    runtime: &Arc<Vec<u8>>,
    artifacts: &BTreeMap<String, &Artifact>,
    meter: &mut Meter,
    ledger: &Ledger,
    work: &mut Vec<Work>,
    resources: &Resources,
    crt: Option<&crate::source_crt::Toolchain>,
) -> Result<std::path::PathBuf, String> {
    let mut assembly = ledger.begin("*", "shared-assembly", Phase::Production);
    let write = |path: &Path, bytes: &[u8]| -> Result<(), String> {
        ledger.io("*", "assembly-output", 0, bytes.len() as u64);
        std::fs::write(path, bytes).map_err(|e| e.to_string())
    };
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    write(
        &directory.join("source938_events.c"),
        crate::source_provider::EVENT_RUNTIME.as_bytes(),
    )?;
    let provider_path = directory.join("provider.obj");
    write(&provider_path, provider.as_slice())?;
    let mut objects = vec![provider_path];
    let sources;
    match target {
        Target::Native => {
            let mut entry = None;
            for (i, (name, a)) in artifacts.iter().enumerate() {
                let TargetWire::Native { symbol, coff, .. } = &a.decoded else {
                    return Err("foreign.target".into());
                };
                let path = directory.join(format!("caller-{i}.obj"));
                write(&path, coff)?;
                objects.push(path);
                if name == "main" {
                    entry = Some(symbol);
                }
            }
            let mut wrapper_work = ledger.begin("*", "entry-wrapper", Phase::Production);
            let wrapper = ace_native::emit::emit_wrapper_coff(entry.ok_or("foreign.missing-main")?)
                .map_err(|e| e.to_string())?;
            wrapper_work.produced(1);
            work.push(wrapper_work);
            let path = directory.join("entry-wrapper.obj");
            write(&path, &wrapper)?;
            objects.push(path);
            write(
                &directory.join("entry.c"),
                b"extern void ac_start(void);\nint main(void){ac_start();return 99;}\n",
            )?;
            write(&directory.join("foreign_runtime.c"), runtime.as_slice())?;
            sources = vec!["entry.c", "foreign_runtime.c", "source938_events.c"];
        }
        Target::C => {
            let fragments = artifacts
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
            let runtime = std::str::from_utf8(runtime.as_slice()).map_err(|e| e.to_string())?;
            let files = ForeignTargetProvider::new(environment.clone())
                .assemble_fileset(FilesetRequest {
                    entry: "main",
                    input_hash: &hash(source.as_bytes()),
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
                write(&directory.join(name), text.as_bytes())?;
            }
            write(&directory.join("entry.c"), b"#include \"a2x_c_runtime.h\"\nextern a2x_result fn_main(void);\nint main(void){a2x_result v=fn_main();return v.trap?70:v.value;}\n")?;
            sources = vec![
                "entry.c",
                "foreign_runtime.c",
                "generated.c",
                "source938_events.c",
            ];
        }
    }
    assembly.produced(1);
    work.push(assembly);
    meter.links += 1;
    let mut tool = ledger.begin("*", "compile-link-tool", Phase::Production);
    let exe = directory.join("source.exe");
    match crt {
        Some(crt) => crt.compile(directory, &sources, &objects, &exe),
        None => crate::bridge::compile_bridge_crt_with_objects_admitted(
            directory,
            &sources,
            &objects,
            &exe,
            Some(resources),
        ),
    }
    .map_err(|e| e.to_string())?;
    tool.produced(1);
    work.push(tool);
    Ok(exe)
}

fn native_artifact(
    semantic: &str,
    bundle: &ace_core::hir::Bundle,
    id: &str,
    name: &str,
    caps: &[String],
    env: &AdmittedForeign,
    meter: &mut Meter,
    ledger: &Ledger,
    work: &mut Vec<Work>,
    resources: &Resources,
) -> Result<Arc<Artifact>, String> {
    let mut lower = ledger.begin(name, "mir-lower", Phase::Production);
    meter.mir_attempts += 1;
    let mut lowering_allocation = resources
        .reserve(Charge::mir(0))
        .map_err(|e| e.to_string())?;
    let mir = ace_lowering::hir_adapter::lower_function_unit_reserved(
        bundle,
        id,
        caps,
        0,
        &mut |bytes| {
            lowering_allocation
                .resize(Charge::mir(bytes))
                .map_err(|e| e.to_string())
        },
    )
    .map_err(|e| match e {
        ace_lowering::AdapterError::Resource { .. } => format!("foreign.resource: {e:?}"),
        _ => format!("foreign.lower: {e:?}"),
    })?;
    meter.mir_produced += 1;
    lower.produced(1);
    work.push(lower);
    let mut verify = ledger.begin(name, "mir-verify", Phase::Production);
    ace_native::foreign_lower::admit_reserved(&mir, env, resources).map_err(|e| e.to_string())?;
    verify.produced(1);
    work.push(verify);
    let mut emit = ledger.begin(name, "native-emit", Phase::Production);
    meter.native_attempts += 1;
    let (object, output_charge) =
        ace_native::emit::emit_foreign_unit_coff_reserved(&mir, env, resources)
            .map_err(|e| e.to_string())?;
    meter.native_produced += 1;
    emit.produced(1);
    work.push(emit);
    let owner_schema = mir.owner.schema.clone();
    let owner_module = mir.owner.module.clone();
    let mut encode = ledger.begin(name, "mir-encode", Phase::Production);
    let mut encoded_charge = resources
        .reserve(Charge::staging(0))
        .map_err(|e| e.to_string())?;
    let paged = ace_av_core::abc::encode_abc_reserved(&mir, |bytes| {
        encoded_charge.resize(Charge::staging(bytes)).map_err(|e| {
            ace_av_core::abc::AbcDecodeError::Resource {
                kind: ace_av_core::contract::ResourceKind::CodecPage,
                message: e.to_string(),
            }
        })
    })
    .map_err(|e| format!("foreign.mir-encode: {e:?}"))?;
    // The emitted arrays are now represented by the separately reserved
    // encoded pages. Drop the MIR owner before releasing its capacity fee.
    drop(mir);
    drop(lowering_allocation);
    let mir = paged.to_flat();
    drop(paged);
    encoded_charge
        .resize(Charge::staging(mir.capacity()))
        .map_err(|e| e.to_string())?;
    let output_charge = output_charge
        .merge(encoded_charge)
        .map_err(|e| e.to_string())?;
    encode.produced(1);
    work.push(encode);
    meter.page_encodes += 1;
    let mut page = ledger.begin(name, "target-encode", Phase::Production);
    let artifact = Artifact::new_reserved(
        TargetWire::Native {
            name: name.into(),
            symbol: object.symbol,
            owner_schema,
            owner_module,
            semantic: semantic.into(),
            mir,
            coff: object.bytes,
        },
        resources,
        Some(output_charge),
    )?;
    page.produced(1);
    work.push(page);
    Ok(artifact)
}
fn admit_provider(
    env: &AdmittedForeign,
    bytes: &Arc<Vec<u8>>,
    resources: &Resources,
) -> Result<(), String> {
    if bytes.len() > MAX_ARTIFACT || blake3::hash(bytes).to_hex().as_str() != env.provider_digest()
    {
        return Err("foreign.provider-content".into());
    }
    // Caller-owned input is charged as it enters this composition, before
    // symbol parsing or any source/target producer. Aliases, restored pages
    // and externally retained candidate inputs share one immutable owner.
    resources
        .adopt(bytes, Charge::resident(bytes.capacity() + 40))
        .map_err(|e| e.to_string())?;
    use object::{Object, ObjectSymbol};
    let declarations = env.source_declarations_ref();
    let names = declarations
        .iter()
        .try_fold(0usize, |n, d| {
            n.checked_add(1 + usize::from(d.context_symbol.is_some()))
        })
        .ok_or("foreign.resource.provider-symbol-index-overflow")?;
    // The current supported libstd B-tree has B=6: at most 11 keys/values and 12
    // edges per node. 64 bytes covers parent/index/len/alignment. Count
    // at most one node per inserted key in EACH of these two borrowed
    // indexes. Explicit insert avoids FromIterator's temporary sort Vec.
    let node = 64
        + 11 * (std::mem::size_of::<&str>() + std::mem::size_of::<object::SymbolKind>())
        + 12 * std::mem::size_of::<usize>();
    let index_bytes = names
        .checked_mul(2)
        .and_then(|n| n.checked_mul(node))
        .ok_or("foreign.resource.provider-symbol-index-overflow")?;
    let _index = resources
        .reserve(Charge::index(index_bytes))
        .map_err(|e| format!("foreign.resource.provider-symbol-index: {e}"))?;
    let object = object::File::parse(bytes.as_slice())
        .map_err(|e| format!("foreign.provider-object: {e}"))?;
    if object.format() != object::BinaryFormat::Coff
        || object.architecture() != object::Architecture::X86_64
        || object.kind() != object::ObjectKind::Relocatable
    {
        return Err("foreign.provider-profile: expected AMD64 relocatable COFF".into());
    }
    let mut needed = std::collections::BTreeSet::new();
    for declaration in declarations {
        needed.insert(declaration.function.symbol.as_str());
        if let Some(context) = &declaration.context_symbol {
            needed.insert(context.as_str());
        }
    }
    let mut definitions = std::collections::BTreeMap::new();
    for symbol in object
        .symbols()
        .filter(|s| s.is_definition() && s.is_global())
    {
        if let Ok(name) = symbol.name() {
            if needed.contains(name) {
                definitions.insert(name, symbol.kind());
            }
        }
    }
    for declaration in declarations {
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
    runtime_identity: String,
    provider: String,
    provider_digest: String,
    binding_generation: u64,
    contract: String,
    foreign: ForeignRoot,
    functions: BTreeMap<String, String>,
}
enum EnvironmentInput {
    Admitted(AdmittedForeign),
    Host(HostAuthority),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ForeignRoot {
    logical: String,
    physical: String,
    declarations: String,
    binding: String,
}
impl ForeignRoot {
    fn from_pages(p: &ForeignPages) -> Self {
        Self {
            logical: hash(&p.logical),
            physical: hash(&p.physical),
            declarations: hash(&p.declarations),
            binding: hash(&p.binding),
        }
    }
    fn iter(&self) -> impl Iterator<Item = (&'static str, &str)> {
        [
            ("restore-foreign-logical", self.logical.as_str()),
            ("restore-foreign-physical", self.physical.as_str()),
            ("restore-foreign-declarations", self.declarations.as_str()),
            ("restore-foreign-binding", self.binding.as_str()),
        ]
        .into_iter()
    }
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
    a.cache_hits += b.cache_hits;
    a.cache_lookups += b.cache_lookups;
    a.cache_writes += b.cache_writes;
    a.cache_bytes_read += b.cache_bytes_read;
    a.cache_bytes_written += b.cache_bytes_written;
    a.cache_evictions += b.cache_evictions;
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
        cache_hits: a.cache_hits - b.cache_hits,
        cache_lookups: a.cache_lookups - b.cache_lookups,
        cache_writes: a.cache_writes - b.cache_writes,
        cache_bytes_read: a.cache_bytes_read - b.cache_bytes_read,
        cache_bytes_written: a.cache_bytes_written - b.cache_bytes_written,
        cache_evictions: a.cache_evictions - b.cache_evictions,
    }
}

#[allow(clippy::too_many_arguments)]
fn selected_artifact(
    target: Target,
    parts: &ace_core::source::SourceParts,
    checked: &ace_core::verify::CheckedFunction,
    name: &str,
    capabilities: &[String],
    environment: &AdmittedForeign,
    cache: &mut function_cache::FunctionCache,
    runtime: &str,
    counts: &mut Meter,
    work: &mut Vec<Work>,
) -> Result<Arc<Artifact>, String> {
    let view = SemanticView::from_parts(parts, checked).map_err(|e| format!("{e:?}"))?;
    let f = view.functions().first().ok_or("foreign.empty-fragment")?;
    cached_artifact(
        target,
        &parts.bundle,
        checked.def_id_text(),
        name,
        capabilities,
        environment,
        &view,
        f,
        cache,
        runtime,
        counts,
        work,
    )
}
#[allow(clippy::too_many_arguments)]
fn cached_artifact(
    target: Target,
    bundle: &ace_core::hir::Bundle,
    id: &str,
    name: &str,
    capabilities: &[String],
    environment: &AdmittedForeign,
    view: &SemanticView<'_>,
    f: &ace_a2x_core::FnFacts<'_>,
    cache: &mut function_cache::FunctionCache,
    runtime: &str,
    counts: &mut Meter,
    work: &mut Vec<Work>,
) -> Result<Arc<Artifact>, String> {
    ForeignTargetProvider::new(environment.clone())
        .validate_view(view)
        .map_err(|e| format!("{e:?}"))?;
    let semantic =
        ace_a2x_core::prepare::semantic_fingerprint(view, f).map_err(|e| format!("{e:?}"))?;
    let (key, _key_charge) = cache.key(
        name,
        &semantic,
        target,
        if target == Target::C {
            runtime
        } else {
            "foreign-scalar-runtime-abi/1"
        },
        capabilities,
    )?;
    if let Some(artifact) = cache.get(&key, target, name, &semantic, environment, counts)? {
        return Ok(artifact);
    }
    let artifact = match target {
        Target::Native => native_artifact(
            &semantic,
            bundle,
            id,
            name,
            capabilities,
            environment,
            counts,
            &cache.work,
            work,
            &cache.resources,
        )?,
        Target::C => {
            let text =
                cache.c_fragment(view, f, capabilities, runtime, environment, counts, work)?;
            let mut page = cache.work.begin(name, "target-encode", Phase::Production);
            counts.page_encodes += 1;
            let artifact = Artifact::new(
                TargetWire::C {
                    name: name.into(),
                    def_index: f.def_index,
                    semantic,
                    text,
                },
                &cache.resources,
            )?;
            page.produced(1);
            work.push(page);
            artifact
        }
    };
    cache.put(&key, artifact.clone(), counts)?;
    Ok(artifact)
}
#[path = "source_function_cache.rs"]
mod function_cache;

#[path = "source_publication.rs"]
mod publication;
#[path = "source_publication_scratch.rs"]
mod publication_scratch;
pub use publication::{
    publication_root_project, read_pointer_bytes, read_pointer_bytes_reserved,
    read_pointer_identity, CurrentAuthority, Edit, PointerRead, PreparedPublication,
};
pub use publication_scratch::PointerEncoder;

/// SDK composition identity includes the common failure/event runtime assets,
/// not merely the persisted selected-target runtime page.
pub fn runtime_identity(target: Target, runtime_page: &str) -> String {
    hash(
        format!(
            "source-runtime/1|{}|{}|{}|{}",
            target.identity(),
            runtime_page,
            hash(ace_a2x_c::foreign::FAILURE_RUNTIME.as_bytes()),
            hash(crate::source_provider::EVENT_RUNTIME.as_bytes())
        )
        .as_bytes(),
    )
}
/// Independent policy for the explicit no-trace source certification fixture.
/// Pinned SDK asset identity: qualification/restore must not invoke a runtime
/// emitter to reconstruct the expected bytes. The source conformance test
/// compares this pin with the current runtime material.
pub const CERTIFICATION_C_RUNTIME_SHA256: &str =
    "c7cf3971f2058ef57ca4d55386336abceb7af60ceb9fcef71ed4641954de6427";
pub fn certification_runtime_policy(target: Target) -> String {
    let page = match target {
        Target::Native => hash(ace_a2x_c::foreign::FAILURE_RUNTIME.as_bytes()),
        Target::C => CERTIFICATION_C_RUNTIME_SHA256.into(),
    };
    runtime_identity(target, &page)
}

fn validate_target_artifact(
    target: Target,
    name: &str,
    decoded: &TargetWire,
    environment: &AdmittedForeign,
    resources: &Resources,
) -> Result<(), String> {
    match decoded {
        TargetWire::Native {
            name: stored,
            symbol,
            owner_schema,
            owner_module,
            mir,
            coff,
            ..
        } if target == Target::Native && stored.as_str() == name => {
            // The shared source adapter uses the stable HIR module
            // id `m_src`; its logical source path lives separately in
            // the authenticated frontend root.
            if owner_schema != ace_av_core::contract::MIR_SCHEMA || owner_module != "m_src" {
                return Err("foreign.restore.native-owner".into());
            }
            // ABC deliberately omits the project/unit owner. Rebind
            // it from the authenticated required envelope BEFORE MIR
            // admission, as the existing unit-store protocol does.
            let (maximum, mut mir_allocation) = resources
                .reserve_remaining_mir()
                .map_err(|e| e.to_string())?;
            let mut decoded = ace_av_core::abc::decode_abc_limited(mir, maximum)
                .map_err(|e| format!("foreign.restore.mir: {e:?}"))?;
            let reanchor = owner_schema
                .len()
                .checked_add(owner_module.len())
                .and_then(|n| n.checked_mul(decoded.module.funcs.len().saturating_add(1)))
                .and_then(|n| n.checked_add(decoded.module.heap_estimate()))
                .ok_or("foreign.resource.native-owner-size")?;
            mir_allocation
                .resize(Charge::mir(reanchor))
                .map_err(|e| e.to_string())?;
            decoded.module.owner = ace_av_core::mir::OwnerScope {
                schema: owner_schema.clone(),
                module: owner_module.clone(),
            };
            for f in &mut decoded.module.funcs {
                f.owner = decoded.module.owner.clone();
            }
            mir_allocation
                .resize(Charge::mir(decoded.module.heap_estimate()))
                .map_err(|e| e.to_string())?;
            ace_native::foreign_lower::admit_reserved(&decoded.module, environment, resources)
                .map_err(|e| e.to_string())?;
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
        } if target == Target::C && stored.as_str() == name => {
            if !text.starts_with("// origin:") || text.lines().nth(1).is_none() {
                return Err("foreign.restore.c-fragment".into());
            }
        }
        _ => return Err("foreign.restore.target-artifact".into()),
    }
    Ok(())
}
