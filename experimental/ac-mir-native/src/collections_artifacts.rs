//! Collections MIR/runtime/manifest pages in the existing ManagedStore. No
//! verification credential is serialized; restoration must decode and re-admit.
use crate::{cache::CoffMeta, config::*, MnError};
use ac_artifact_probe::{
    codec::ChunkKind,
    identity::{QueryKey, RefKind},
    schema::{LayerCoverage, LayerSpec, Regeneration},
    store::{ManagedStore, PutRequest},
};
use serde::{Deserialize, Serialize};
const PRODUCER: &str = "plan889/native-collections";
const BLOB_LAYER: LayerSpec = LayerSpec {
    id: "A08",
    name: "native collections input pages",
    schema_id: "ac889/a08-native-input/1",
    producer: PRODUCER,
    consumers: &["collections driver/restore"],
    key_inputs: &["kind", "hash", "page"],
    ref_kinds: &[],
    storage: "bounded LayoutData pages",
    residency: "untrusted bytes, decode/re-admit",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};
pub const MANIFEST_LAYER: LayerSpec = LayerSpec {
    id: "A10",
    name: "native collections closure",
    schema_id: "ac889/a10-collections/3",
    producer: PRODUCER,
    consumers: &["collections restore"],
    key_inputs: &["manifest"],
    ref_kinds: &["required-data"],
    storage: "envelope with required paged closure",
    residency: "untrusted bytes, decode/re-admit",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Blob {
    pub kind: String,
    pub hash: String,
    pub len: usize,
    pub pages: Vec<String>,
    /// Only MIR may occupy a slice of shared immutable pages. This is a byte
    /// address, never an admission credential; the sliced ABC is verified anew.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slice_offset: Option<usize>,
}
fn bad(message: impl Into<String>) -> MnError {
    MnError::Admit("collections-artifact", message.into())
}
/// The store's existing by-id convenience read has no caller byte ceiling.
/// Resolve only its chunk address, then use the landed bounded/CAS-gated read
/// before allocating page contents. The query never writes store state.
pub(crate) fn read_page(
    store: &ManagedStore,
    id: &str,
    cap: usize,
) -> Result<ac_artifact_probe::store::LoadedArtifact, MnError> {
    use ac_artifact_probe::codec::{unframe_chunk, CHUNK_HEADER_LEN};
    let (chunk,kind):(String,u8)=store.db().query_row(
        "SELECT a.payload_chunk,c.kind FROM artifacts a JOIN chunks c ON c.chunk_id=a.payload_chunk WHERE a.artifact_id=?1",
        [id],|row|Ok((row.get(0)?,row.get(1)?)))
        .map_err(|e|bad(format!("required artifact address: {e}")))?;
    let bytes = store
        .read_chunk_gated_bounded(&chunk, (cap + CHUNK_HEADER_LEN) as u64)
        .map_err(|e| bad(format!("bounded artifact read: {e}")))?
        .ok_or_else(|| bad("required artifact missing"))?;
    let (actual, payload) =
        unframe_chunk(&bytes).map_err(|e| bad(format!("artifact frame: {e}")))?;
    if actual != ChunkKind::LayoutData || kind != actual as u8 || payload.len() > cap {
        return Err(bad("artifact page carrier/size"));
    }
    Ok(ac_artifact_probe::store::LoadedArtifact {
        artifact_id: id.into(),
        chunk_id: chunk,
        kind: actual,
        payload: payload.to_vec(),
    })
}
fn cap(kind: &str) -> Result<usize, MnError> {
    match kind {
        "mir" => Ok(auto_av_core::contract::MAX_INPUT_BYTES),
        "source" => Ok(crate::config::MAX_INPUT_BYTES),
        "runtime" => Ok(MAX_LINK_INPUT_BYTES),
        "manifest" => Ok(METADATA_CAP),
        _ => Err(bad("unknown blob kind")),
    }
}
pub fn put_blob(store: &ManagedStore, kind: &str, bytes: &[u8]) -> Result<Blob, MnError> {
    crate::collections_memory::scoped(crate::resources::Partition::Io, || {
        put_blob_inner(store, kind, bytes)
    })
}
fn put_blob_inner(store: &ManagedStore, kind: &str, bytes: &[u8]) -> Result<Blob, MnError> {
    if bytes.is_empty() || bytes.len() > cap(kind)? {
        return Err(MnError::Resource(
            "collections-blob",
            format!("{kind} has {} bytes", bytes.len()),
        ));
    }
    let hash = crate::cache::content_hash(bytes);
    let mut pages = Vec::new();
    for (i, page) in bytes.chunks(PAGE_BYTES).enumerate() {
        let key = QueryKey::new(
            &BLOB_LAYER,
            vec![
                ("kind".into(), kind.into()),
                ("hash".into(), hash.clone()),
                ("page".into(), i.to_string()),
            ],
        );
        pages.push(
            store
                .put_artifact(PutRequest {
                    layer: BLOB_LAYER.id,
                    key,
                    kind: ChunkKind::LayoutData,
                    payload: page.to_vec(),
                    edges: vec![],
                    project: None,
                })
                .map_err(|e| MnError::Infra(format!("blob store: {e}")))?,
        );
    }
    Ok(Blob {
        kind: kind.into(),
        hash,
        len: bytes.len(),
        pages,
        slice_offset: None,
    })
}
/// Cold units share carrier pages, avoiding one durable store transaction per
/// small MIR unit. Only one page is buffered; unit hashes still bind exact ABC.
#[derive(Default)]
pub(crate) struct MirPagePacker {
    buffer: Vec<u8>,
    pages: Vec<String>,
    pending: Vec<(String, usize, usize, usize)>,
}
impl MirPagePacker {
    fn flush(&mut self, store: &ManagedStore) -> Result<(), MnError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let hash = crate::cache::content_hash(&self.buffer);
        let key = QueryKey::new(
            &BLOB_LAYER,
            vec![
                ("kind".into(), "mir-page".into()),
                ("hash".into(), hash),
                ("page".into(), "0".into()),
            ],
        );
        let payload = std::mem::take(&mut self.buffer);
        let id = store
            .put_artifact(PutRequest {
                layer: BLOB_LAYER.id,
                key,
                kind: ChunkKind::LayoutData,
                payload,
                edges: vec![],
                project: None,
            })
            .map_err(|e| MnError::Infra(format!("packed MIR store: {e}")))?;
        self.pages.push(id);
        Ok(())
    }
    pub(crate) fn push(&mut self, store: &ManagedStore, bytes: &[u8]) -> Result<usize, MnError> {
        if bytes.is_empty() || bytes.len() > cap("mir")? {
            return Err(bad("packed MIR size"));
        }
        let ticket = self.pending.len();
        self.pending.push((
            crate::cache::content_hash(bytes),
            bytes.len(),
            self.pages.len(),
            self.buffer.len(),
        ));
        let mut remaining = bytes;
        while !remaining.is_empty() {
            let count = remaining.len().min(PAGE_BYTES - self.buffer.len());
            self.buffer.extend_from_slice(&remaining[..count]);
            remaining = &remaining[count..];
            if self.buffer.len() == PAGE_BYTES {
                self.flush(store)?;
            }
        }
        Ok(ticket)
    }
    pub(crate) fn finish(mut self, store: &ManagedStore) -> Result<Vec<Blob>, MnError> {
        self.flush(store)?;
        self.pending
            .into_iter()
            .map(|(hash, len, first, offset)| {
                let count = (offset + len).div_ceil(PAGE_BYTES);
                let pages = self
                    .pages
                    .get(first..first + count)
                    .ok_or_else(|| bad("packed MIR ticket"))?
                    .to_vec();
                Ok(Blob {
                    kind: "mir".into(),
                    hash,
                    len,
                    pages,
                    slice_offset: Some(offset),
                })
            })
            .collect()
    }
}
fn visit_blob(
    store: &ManagedStore,
    blob: &Blob,
    expected: &str,
    mut consume: impl FnMut(&[u8]),
) -> Result<(), MnError> {
    let offset = blob.slice_offset.unwrap_or(0);
    if blob.kind != expected
        || blob.len == 0
        || blob.len > cap(expected)?
        || offset >= PAGE_BYTES
        || (blob.slice_offset.is_some() && expected != "mir")
        || blob.pages.len() != (offset + blob.len).div_ceil(PAGE_BYTES)
    {
        return Err(bad("blob kind/size/page count mismatch"));
    }
    let mut hash = blake3::Hasher::new();
    let mut total = 0;
    for (i, id) in blob.pages.iter().enumerate() {
        let art = read_page(store, id, PAGE_BYTES)?;
        let start = if i == 0 { offset } else { 0 };
        let count = (blob.len - total).min(PAGE_BYTES - start);
        let end = start + count;
        if art.kind != ChunkKind::LayoutData
            || art.payload.len() < end
            || (i + 1 < blob.pages.len() && art.payload.len() != PAGE_BYTES)
            || (blob.slice_offset.is_none() && art.payload.len() != end)
        {
            return Err(bad("page carrier/length mismatch"));
        }
        let slice = &art.payload[start..end];
        hash.update(slice);
        total += slice.len();
        consume(slice);
    }
    if total != blob.len || hash.finalize().to_hex().as_str() != blob.hash {
        return Err(bad("blob content binding mismatch"));
    }
    Ok(())
}
pub fn get_blob(store: &ManagedStore, blob: &Blob, kind: &str) -> Result<Vec<u8>, MnError> {
    crate::collections_memory::scoped(crate::resources::Partition::Io, || {
        get_blob_inner(store, blob, kind)
    })
}
fn get_blob_inner(store: &ManagedStore, blob: &Blob, kind: &str) -> Result<Vec<u8>, MnError> {
    // Validate claimed size before capacity allocation (also checked in visit).
    if blob.kind != kind || blob.len > cap(kind)? {
        return Err(bad("blob allocation cap"));
    }
    let mut out = Vec::with_capacity(blob.len);
    visit_blob(store, blob, kind, |page| out.extend_from_slice(page))?;
    Ok(out)
}
pub fn verify_runtime_blob(store: &ManagedStore, blob: &Blob) -> Result<(), MnError> {
    visit_blob(store, blob, "runtime", |_| {})
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    pub owner_schema: String,
    pub owner_module: String,
    pub unit: String,
    pub symbol: String,
    pub fp: String,
    pub abi: String,
    pub target: String,
    pub flags: String,
    pub backend: String,
    pub runtime: String,
    pub mir_schema: String,
    pub mir_semantics: String,
}
impl From<&CoffMeta> for Meta {
    fn from(m: &CoffMeta) -> Self {
        Self {
            owner_schema: m.owner.schema.clone(),
            owner_module: m.owner.module.clone(),
            unit: m.unit.clone(),
            symbol: m.symbol.clone(),
            fp: m.fp.clone(),
            abi: m.abi.clone(),
            target: m.target.clone(),
            flags: m.flags.clone(),
            backend: m.backend.clone(),
            runtime: m.runtime.clone(),
            mir_schema: m.mir_schema.clone(),
            mir_semantics: m.mir_semantics.clone(),
        }
    }
}
impl Meta {
    pub fn to_meta(&self) -> CoffMeta {
        CoffMeta {
            owner: auto_av_core::mir::OwnerScope {
                schema: self.owner_schema.clone(),
                module: self.owner_module.clone(),
            },
            unit: self.unit.clone(),
            symbol: self.symbol.clone(),
            fp: self.fp.clone(),
            abi: self.abi.clone(),
            target: self.target.clone(),
            flags: self.flags.clone(),
            backend: self.backend.clone(),
            runtime: self.runtime.clone(),
            mir_schema: self.mir_schema.clone(),
            mir_semantics: self.mir_semantics.clone(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub meta: Meta,
    pub mir: Blob,
    pub coff_id: String,
    pub coff_hash: String,
    pub coff_len: usize,
    pub extra_coff_pages: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coordinate {
    pub unit: String,
    pub mint_start: usize,
    pub mint_end: usize,
    pub current_start: usize,
    pub current_end: usize,
    pub mint_snapshot: u64,
    pub text_hash: String,
}
impl Coordinate {
    pub fn identity(unit: &auto_av_core::mir::MirModule, source: &str) -> Self {
        Self {
            unit: unit.funcs[0].name.clone(),
            mint_start: 0,
            mint_end: source.len(),
            current_start: 0,
            current_end: source.len(),
            mint_snapshot: unit.funcs[0].source_snapshot,
            text_hash: crate::cache::content_hash(source.as_bytes()),
        }
    }
    pub fn validate_and_map(
        &self,
        unit: &auto_av_core::mir::MirModule,
        source: &str,
    ) -> Result<Vec<(u32, auto_av_core::contract::SpanInfo)>, MnError> {
        let text = source
            .get(self.current_start..self.current_end)
            .ok_or_else(|| bad("coordinate current range/UTF8"))?;
        if self.unit != unit.funcs[0].name
            || self.mint_start > self.mint_end
            || self.mint_end - self.mint_start != text.len()
            || self.mint_end > crate::config::MAX_INPUT_BYTES
            || self.mint_snapshot != unit.funcs[0].source_snapshot
            || crate::cache::content_hash(text.as_bytes()) != self.text_hash
        {
            return Err(bad("coordinate unit/range/snapshot/text binding"));
        }
        let mut mapped = Vec::with_capacity(unit.funcs[0].spans.len());
        for (instruction, span) in &unit.funcs[0].spans {
            let start = span.start as usize;
            let end = span.end as usize;
            if start < self.mint_start || end < start || end > self.mint_end {
                return Err(bad("MIR span outside its mint range"));
            }
            let start = start - self.mint_start + self.current_start;
            let end = end - self.mint_start + self.current_start;
            if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
                return Err(bad("mapped MIR span splits UTF8"));
            }
            mapped.push((
                *instruction,
                auto_av_core::contract::SpanInfo {
                    func: span.func,
                    start: start as u32,
                    end: end as u32,
                },
            ));
        }
        Ok(mapped)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub profile: String,
    pub abi: String,
    pub target: String,
    pub codegen_build: String,
    pub owner_module: String,
    pub entry: String,
    pub entry_symbol: String,
    pub source_hash: String,
    pub source: Blob,
    pub coordinates: Vec<Coordinate>,
    pub runtime: crate::collections_runtime::Identity,
    pub runtime_blob: Blob,
    pub units: Vec<Unit>,
    pub lld: String,
    pub sdk: String,
    pub kernel32: String,
}
impl Manifest {
    pub fn bytes(&self) -> Result<Vec<u8>, MnError> {
        serde_json::to_vec(self).map_err(|e| bad(e.to_string()))
    }
    pub fn fingerprint(&self) -> Result<String, MnError> {
        Ok(crate::cache::content_hash(&self.bytes()?))
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: String,
    body: Blob,
}
pub fn put_manifest(store: &ManagedStore, manifest: &Manifest) -> Result<String, MnError> {
    let bytes = manifest.bytes()?;
    let body = put_blob(store, "manifest", &bytes)?;
    let fingerprint = crate::cache::content_hash(&bytes);
    let key = QueryKey::new(&MANIFEST_LAYER, vec![("manifest".into(), fingerprint)]);
    let mut required = body.pages.clone();
    required.extend(manifest.source.pages.iter().cloned());
    required.extend(manifest.runtime_blob.pages.iter().cloned());
    for unit in &manifest.units {
        required.extend(unit.mir.pages.iter().cloned());
        required.push(unit.coff_id.clone());
        required.extend(unit.extra_coff_pages.iter().cloned());
    }
    required.sort();
    required.dedup();
    let envelope = Envelope {
        schema: MANIFEST_LAYER.schema_id.into(),
        body,
    };
    store
        .put_artifact(PutRequest {
            layer: MANIFEST_LAYER.id,
            key,
            kind: ChunkKind::LayoutData,
            payload: serde_json::to_vec(&envelope).map_err(|e| bad(e.to_string()))?,
            edges: required
                .into_iter()
                .map(|id| (id, RefKind::RequiredData))
                .collect(),
            project: None,
        })
        .map_err(|e| MnError::Infra(format!("manifest store: {e}")))
}
pub fn get_manifest(store: &ManagedStore, id: &str) -> Result<Manifest, MnError> {
    let art = read_page(store, id, PAGE_BYTES)?;
    if art.kind != ChunkKind::LayoutData || art.payload.len() > PAGE_BYTES {
        return Err(bad("manifest envelope carrier/size"));
    }
    let envelope: Envelope =
        serde_json::from_slice(&art.payload).map_err(|e| bad(format!("manifest envelope: {e}")))?;
    if envelope.schema != MANIFEST_LAYER.schema_id {
        return Err(bad("manifest envelope schema"));
    }
    let bytes = get_blob(store, &envelope.body, "manifest")?;
    serde_json::from_slice(&bytes).map_err(|e| bad(format!("manifest body: {e}")))
}

#[cfg(test)]
mod packed_mir_tests {
    use super::*;
    #[test]
    fn exact_cross_page_slices_bind_content_and_reject_forged_addresses() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/plans/attachments/889")
            .join(format!(
                "packed-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        let store = ManagedStore::open(&root).unwrap();
        let inputs = [
            vec![19; PAGE_BYTES - 3],
            vec![71; 10],
            vec![109; PAGE_BYTES + 11],
        ];
        let mut pack = MirPagePacker::default();
        for (i, input) in inputs.iter().enumerate() {
            assert_eq!(pack.push(&store, input).unwrap(), i);
        }
        let blobs = pack.finish(&store).unwrap();
        assert_eq!(blobs[0].pages[0], blobs[1].pages[0]);
        assert_eq!(blobs[1].pages.len(), 2);
        for (blob, input) in blobs.iter().zip(&inputs) {
            assert_eq!(&get_blob(&store, blob, "mir").unwrap(), input);
        }
        for fault in 0..8 {
            let mut blob = blobs[1].clone();
            match fault {
                0 => blob.slice_offset = Some(PAGE_BYTES),
                1 => blob.slice_offset = Some(0),
                2 => blob.slice_offset = None,
                3 => blob.pages.truncate(1),
                4 => blob.pages[0] = "missing".into(),
                5 => blob.hash = "forged".into(),
                6 => blob.len += 1,
                _ => blob.pages.swap(0, 1),
            }
            assert!(get_blob(&store, &blob, "mir").is_err(), "fault {fault}");
        }
        let mut runtime = blobs[0].clone();
        runtime.kind = "runtime".into();
        assert!(get_blob(&store, &runtime, "runtime").is_err());
    }
}
