//! Function semantic keys are computed from the checked HIR BEFORE MIR/print.
//! The bounded resident history and the original ManagedStore carry bytes only,
//! never PASS or a persisted current-host authorization.
use super::*;
use std::collections::VecDeque;

pub(super) const FUNCTION_LAYER: LayerSpec = LayerSpec {
    id: "S938F",
    name: "checked-source-target-function",
    schema_id: "abi/source-target-function/1",
    producer: "auto-abi-source-foreign/1",
    consumers: &["source foreign target"],
    key_inputs: &["owner", "decl", "semantic", "target", "runtime", "caps"],
    ref_kinds: &[],
    storage: "original ManagedStore",
    residency: "bounded optional history",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};
const HISTORY_BYTES: usize = 8 << 20;
const HISTORY_ENTRIES: usize = 512;
pub(super) struct FunctionCache {
    pub(super) work: Ledger,
    owner: String,
    memory: BTreeMap<String, Arc<Artifact>>,
    order: VecDeque<String>,
    bytes: usize,
    store: Option<ManagedStore>,
    c: Option<ace_a2x_store::A2xSession>,
}
impl FunctionCache {
    pub fn new(package: &str, module: &str, work: Ledger) -> Self {
        Self {
            work,
            owner: serde_json::to_string(&(package, module)).expect("strings"),
            memory: BTreeMap::new(),
            order: VecDeque::new(),
            bytes: 0,
            store: None,
            c: None,
        }
    }
    pub fn attach(&mut self, root: &Path, environment: &AdmittedForeign) -> Result<(), String> {
        self.store = Some(ManagedStore::open(root).map_err(|e| e.to_string())?);
        self.c = Some(ace_a2x_store::A2xSession::open(
            root,
            "source938-function-fragments",
            &self.owner,
            128 << 20,
            ForeignTargetProvider::new(environment.clone()),
        )?);
        Ok(())
    }
    pub fn key(
        &self,
        name: &str,
        semantic: &str,
        target: Target,
        runtime: &str,
        caps: &[String],
    ) -> QueryKey {
        let mut caps = caps.to_vec();
        caps.sort();
        caps.dedup();
        QueryKey::new(
            &FUNCTION_LAYER,
            vec![
                ("owner".into(), self.owner.clone()),
                ("decl".into(), name.into()),
                ("semantic".into(), semantic.into()),
                ("target".into(), target.identity()),
                ("runtime".into(), runtime.into()),
                ("caps".into(), serde_json::to_string(&caps).expect("caps")),
            ],
        )
    }
    pub fn get(
        &mut self,
        key: &QueryKey,
        target: Target,
        name: &str,
        semantic: &str,
        environment: &AdmittedForeign,
        counts: &mut Meter,
    ) -> Result<Option<Arc<Artifact>>, String> {
        let canonical = key.canonical();
        if let Some(artifact) = self.memory.get(&canonical).cloned() {
            let verify = self
                .work
                .begin(name, "target-cache-validate", Phase::Recovery);
            validate_target_artifact(target, name, &artifact.decoded, environment)?;
            if artifact.semantic() != semantic {
                return Err("foreign.cache.semantic".into());
            }
            verify.accept();
            self.work.reuse(name, "target", 1);
            counts.cache_hits += 1;
            self.order.retain(|k| k != &canonical);
            self.order.push_back(canonical);
            return Ok(Some(artifact));
        }
        let Some(store) = &self.store else {
            return Ok(None);
        };
        let lookup = self
            .work
            .begin(name, "target-cache-lookup", Phase::Metadata);
        counts.cache_lookups += 1;
        let loaded = store
            .get_artifact_bounded(
                FUNCTION_LAYER.id,
                key,
                (MAX_ARTIFACT + ace_artifacts::codec::CHUNK_HEADER_LEN) as u64,
            )
            .map_err(|e| e.to_string())?;
        lookup.accept();
        let Some(loaded) = loaded else {
            return Ok(None);
        };
        self.work
            .io(name, "target-cache", loaded.payload.len() as u64, 0);
        let verify = self
            .work
            .begin(name, "target-cache-validate", Phase::Recovery);
        counts.cache_bytes_read += loaded.payload.len() as u64;
        let decoded: TargetWire = serde_json::from_slice(&loaded.payload)
            .map_err(|e| format!("foreign.cache.decode: {e}"))?;
        validate_target_artifact(target, name, &decoded, environment)?;
        let artifact = Arc::new(Artifact {
            digest: hash(&loaded.payload),
            bytes: Arc::new(loaded.payload),
            decoded,
        });
        if artifact.semantic() != semantic {
            return Err("foreign.cache.semantic".into());
        }
        verify.accept();
        self.work.reuse(name, "target", 1);
        counts.cache_hits += 1;
        self.remember(canonical, artifact.clone(), counts);
        Ok(Some(artifact))
    }
    pub fn put(
        &mut self,
        key: &QueryKey,
        artifact: Arc<Artifact>,
        counts: &mut Meter,
    ) -> Result<(), String> {
        if let Some(store) = &self.store {
            let name = match &artifact.decoded {
                TargetWire::Native { name, .. } | TargetWire::C { name, .. } => name,
            };
            if store.space().total
                + 2 * (artifact.bytes.len() + ace_artifacts::codec::CHUNK_HEADER_LEN) as u64
                + 4096
                > 128 << 20
            {
                return Err("foreign.resource.store-quota".into());
            }
            let put = self.work.begin(name, "target-cache-put", Phase::Metadata);
            self.work
                .io(name, "target-cache", 0, artifact.bytes.len() as u64);
            store
                .put_artifact(PutRequest {
                    layer: FUNCTION_LAYER.id,
                    key: key.clone(),
                    kind: ChunkKind::SnapshotRecord,
                    payload: artifact.bytes.as_ref().clone(),
                    edges: vec![],
                    project: None,
                })
                .map_err(|e| e.to_string())?;
            put.accept();
            counts.cache_writes += 1;
            counts.cache_bytes_written += artifact.bytes.len() as u64;
        }
        self.remember(key.canonical(), artifact, counts);
        Ok(())
    }
    fn remember(&mut self, key: String, artifact: Arc<Artifact>, counts: &mut Meter) {
        if artifact.bytes.len() > HISTORY_BYTES {
            return;
        }
        if let Some(old) = self.memory.remove(&key) {
            self.bytes -= old.bytes.len();
        }
        self.order.retain(|k| k != &key);
        self.bytes += artifact.bytes.len();
        self.memory.insert(key.clone(), artifact);
        self.order.push_back(key);
        while self.bytes > HISTORY_BYTES || self.memory.len() > HISTORY_ENTRIES {
            let key = self.order.pop_front().expect("nonempty bounded cache");
            if let Some(old) = self.memory.remove(&key) {
                self.bytes -= old.bytes.len();
                counts.cache_evictions += 1;
            }
        }
    }
    pub fn c_fragment(
        &mut self,
        view: &SemanticView<'_>,
        f: &ace_a2x_core::FnFacts<'_>,
        caps: &[String],
        runtime: &str,
        environment: &AdmittedForeign,
        counts: &mut Meter,
        work: &mut Vec<Work>,
    ) -> Result<String, String> {
        let Some(session) = &mut self.c else {
            let mut print = self
                .work
                .begin(f.source_name, "c-fragment", Phase::Production);
            counts.c_attempts += 1;
            let text = ForeignTargetProvider::new(environment.clone())
                .produce_fragments(view, &[f], Family::Scalar)
                .map_err(|e| format!("{e:?}"))?
                .remove(0);
            counts.c_produced += 1;
            print.produced(1);
            work.push(print);
            return Ok(text);
        };
        session.set_target(ForeignTargetProvider::new(environment.clone()));
        let before = session.stats().clone();
        let result = session
            .checked_scalar_fragment_observed(view, f, caps, runtime, false, &self.work, work);
        let after = session.stats();
        counts.c_attempts += after.lower - before.lower;
        counts.c_produced += after.fragments_produced - before.fragments_produced;
        counts.cache_writes += after.persist - before.persist;
        counts.cache_bytes_written += after.bytes_persisted - before.bytes_persisted;
        counts.cache_hits += after.store_hit_fragment - before.store_hit_fragment;
        let fragment = result.map_err(|e| format!("{e:?}"))?;
        Ok(fragment.text)
    }
}
