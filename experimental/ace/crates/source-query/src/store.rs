//! PLAN-934 T-06: the unified ManagedStore page sink (persistence seam).
//!
//! Facts travel as immutable pages in the SAME managed store every other
//! layer uses (`ace-artifacts`), under their own layer id and carrier kind:
//!
//! * the page payload is a self-describing, versioned, bounded codec — the
//!   envelope identities (producer/schema/profile/algo), the complete query
//!   key and every fact entry are re-validated on read, so a foreign,
//!   truncated, tampered or old-schema page is a controlled rejection, never
//!   a reinterpretation;
//! * the store key is the page's complete query key (canonical hash), so a
//!   lookup by key and a lookup by artifact id both resolve the same page;
//! * read-back can be checked against an EXPECTED key (the caller's current
//!   query), which is what a consumer does before installing.
//!
//! Required edges/roots are left to the host's publication policy (T-06
//! continuation with the 929 required-closure protocol); this adapter owns
//! the payload and the admission, not a second transaction framework.

use crate::facts::{BindingFact, InferBodyFacts, NegativeDep, NodeFact, PortableDiag, PortableTy};
use crate::ident::{SQ_ALGO, SQ_PRODUCER, SQ_PROFILE, SQ_SCHEMA};
use crate::keys::QueryKey;
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey as StoreQueryKey;
use ace_artifacts::store::{ManagedStore, PutRequest};
use std::path::Path;

/// The SQ-A layer id in the unified store (separate from `S796`/`S93x`).
pub const SQ_LAYER: &str = "S934";

/// Root kind under which published pages are pinned in their project.
pub const SQ_ROOT_KIND: &str = "source-sq";

/// Carrier kind of an SQ-A fact page. The artifact kind table is shared and
/// versioned; a fact page is an immutable record-like page, so it rides the
/// snapshot-record carrier instead of adding a new on-disk kind.
pub const SQ_CARRIER: ChunkKind = ChunkKind::SnapshotRecord;

/// Page magic + codec version (a new version must MISS, never reinterpret).
pub const PAGE_MAGIC: &[u8; 6] = b"SQ934P";
pub const PAGE_CODEC_VERSION: u8 = 1;
/// One page's byte ceiling (the plan's 512 KiB page cap).
pub const MAX_PAGE_BYTES: usize = 512 * 1024;
const MAX_COUNT: usize = 8192;
const MAX_STR: usize = 4096;

/// Why a page cannot be read/admitted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PageReadError {
    Missing,
    Store(String),
    Decode(String),
    /// The page belongs to another complete key than the caller expected.
    KeyMismatch {
        expected: String,
        found: String,
    },
    /// The page envelope is not this producer's/schema's.
    Envelope {
        found: String,
    },
}

impl std::fmt::Display for PageReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageReadError::Missing => write!(f, "page not found"),
            PageReadError::Store(m) => write!(f, "store read failed: {m}"),
            PageReadError::Decode(m) => write!(f, "page decode rejected: {m}"),
            PageReadError::KeyMismatch { expected, found } => {
                write!(f, "page key {found} is not the expected {expected}")
            }
            PageReadError::Envelope { found } => write!(f, "page envelope `{found}` is foreign"),
        }
    }
}

// -- codec --------------------------------------------------------------

struct Writer {
    out: Option<Vec<u8>>,
    len: usize,
    overflow: bool,
}

impl Writer {
    fn bytes(&mut self, bytes: &[u8]) {
        let Some(len) = self
            .len
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_PAGE_BYTES)
        else {
            self.overflow = true;
            return;
        };
        self.len = len;
        if let Some(out) = &mut self.out {
            out.extend_from_slice(bytes);
        }
    }
    fn u8(&mut self, v: u8) {
        self.bytes(&[v]);
    }
    fn u32(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }
    fn s(&mut self, v: &str) -> Result<(), PageReadError> {
        if v.len() > MAX_STR {
            return Err(PageReadError::Decode(format!(
                "string of {} bytes exceeds the page budget",
                v.len()
            )));
        }
        self.u32(v.len() as u32);
        self.bytes(v.as_bytes());
        if self.overflow {
            return Err(PageReadError::Decode(
                "page exceeds the byte cap before allocation".into(),
            ));
        }
        Ok(())
    }
    fn ty(&mut self, t: PortableTy) {
        self.u8(match t {
            PortableTy::Unit => 0,
            PortableTy::I32 => 1,
            PortableTy::Bool => 2,
        });
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    decoded_bytes: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Reader {
            bytes,
            at: 0,
            decoded_bytes: 0,
        }
    }
    fn reserve(&mut self, bytes: usize) -> Result<(), PageReadError> {
        self.decoded_bytes = self
            .decoded_bytes
            .checked_add(bytes)
            .filter(|n| *n <= 8 << 20)
            .ok_or_else(|| {
                PageReadError::Decode("decoded page/index exceeds 8MiB budget".into())
            })?;
        Ok(())
    }
    fn vector<T>(&mut self, count: usize, minimum_wire: usize) -> Result<Vec<T>, PageReadError> {
        if count
            .checked_mul(minimum_wire)
            .is_none_or(|n| n > self.bytes.len() - self.at)
        {
            return Err(PageReadError::Decode(
                "truncated page before vector allocation".into(),
            ));
        }
        let size = count
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(|| PageReadError::Decode("vector size overflow".into()))?;
        self.reserve(size)?;
        let mut out = Vec::new();
        out.try_reserve_exact(count)
            .map_err(|e| PageReadError::Decode(format!("page allocation: {e}")))?;
        Ok(out)
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], PageReadError> {
        let end = self
            .at
            .checked_add(n)
            .ok_or_else(|| PageReadError::Decode("length overflow".into()))?;
        if end > self.bytes.len() {
            return Err(PageReadError::Decode("truncated page".into()));
        }
        let s = &self.bytes[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, PageReadError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, PageReadError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Result<u64, PageReadError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
    fn count(&mut self, what: &str) -> Result<usize, PageReadError> {
        let n = self.u32()? as usize;
        if n > MAX_COUNT {
            return Err(PageReadError::Decode(format!(
                "{what} count {n} exceeds the page budget"
            )));
        }
        Ok(n)
    }
    fn s(&mut self) -> Result<String, PageReadError> {
        let n = self.u32()? as usize;
        if n > MAX_STR {
            return Err(PageReadError::Decode(format!(
                "string of {n} bytes exceeds the page budget"
            )));
        }
        let b = self.take(n)?;
        let text = std::str::from_utf8(b).map_err(|_| PageReadError::Decode("bad UTF-8".into()))?;
        self.reserve(n)?;
        let mut out = String::new();
        out.try_reserve_exact(n)
            .map_err(|e| PageReadError::Decode(format!("page string allocation: {e}")))?;
        out.push_str(text);
        Ok(out)
    }
    fn ty(&mut self) -> Result<PortableTy, PageReadError> {
        Ok(match self.u8()? {
            0 => PortableTy::Unit,
            1 => PortableTy::I32,
            2 => PortableTy::Bool,
            k => return Err(PageReadError::Decode(format!("unknown type tag {k}"))),
        })
    }
}

/// Every node kind the finite surface can carry. A page naming another kind
/// is a foreign/corrupt page.
const KINDS: &[&str] = &[
    "expr.int",
    "expr.bool",
    "expr.var",
    "expr.call",
    "expr.binary",
    "expr.unary",
    // PLAN-936（T-08）：CT 需求原子（类型=inner 的类型）。
    "expr.hash",
];

fn kind_index(kind: &str) -> Result<u8, PageReadError> {
    KINDS
        .iter()
        .position(|k| *k == kind)
        .map(|i| i as u8)
        .ok_or_else(|| PageReadError::Decode(format!("unknown node kind `{kind}`")))
}

fn kind_of(i: u8) -> Result<&'static str, PageReadError> {
    KINDS
        .get(i as usize)
        .copied()
        .ok_or_else(|| PageReadError::Decode(format!("unknown node kind tag {i}")))
}

/// Encode one complete fact page (payload only; the store frames it).
pub fn encode_page(key: &QueryKey, facts: &InferBodyFacts) -> Result<Vec<u8>, PageReadError> {
    let size = encoded_page_size(key, facts)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|e| PageReadError::Decode(format!("page allocation: {e}")))?;
    let mut writer = Writer {
        out: Some(bytes),
        len: 0,
        overflow: false,
    };
    write_page(&mut writer, key, facts)?;
    Ok(writer.out.expect("encoding writer"))
}

pub fn encoded_page_size(key: &QueryKey, facts: &InferBodyFacts) -> Result<usize, PageReadError> {
    for n in [
        facts.nodes.len(),
        facts.bindings.len(),
        facts.diags.len(),
        facts.missing.len(),
        facts.sig_reads.len(),
        facts.negatives.len(),
    ] {
        if n > MAX_COUNT {
            return Err(PageReadError::Decode(
                "entry count exceeds the page budget".into(),
            ));
        }
    }
    let mut measure = Writer {
        out: None,
        len: 0,
        overflow: false,
    };
    write_page(&mut measure, key, facts)?;
    Ok(measure.len)
}

fn write_page(w: &mut Writer, key: &QueryKey, facts: &InferBodyFacts) -> Result<(), PageReadError> {
    w.bytes(PAGE_MAGIC);
    w.u8(PAGE_CODEC_VERSION);
    w.s(&key.canonical())?;
    w.s(SQ_PRODUCER)?;
    w.s(SQ_SCHEMA)?;
    w.s(SQ_PROFILE)?;
    w.s(SQ_ALGO)?;
    w.s(&facts.name)?;
    w.s(facts.logical.as_str())?;
    w.s(facts.body_version.as_str())?;
    w.u64(facts.revision);
    w.u64(facts.steps);
    w.u32(facts.nodes.len() as u32);
    for n in &facts.nodes {
        w.u32(n.id.0);
        w.u8(kind_index(n.kind)?);
        w.ty(n.ty);
    }
    w.u32(facts.bindings.len() as u32);
    for b in &facts.bindings {
        w.u32(b.node.0);
        w.s(&b.name)?;
        w.ty(b.ty);
    }
    w.u32(facts.diags.len() as u32);
    for d in &facts.diags {
        w.s(&d.code)?;
        match d.node {
            Some(id) => {
                w.u8(1);
                w.u32(id.0);
            }
            None => w.u8(0),
        }
        w.s(&d.message)?;
    }
    w.u32(facts.missing.len() as u32);
    for id in &facts.missing {
        w.u32(id.0);
    }
    w.u32(facts.sig_reads.len() as u32);
    for s in &facts.sig_reads {
        w.s(s)?;
    }
    w.u32(facts.negatives.len() as u32);
    for n in &facts.negatives {
        w.s(&n.domain)?;
        w.s(&n.name)?;
    }
    if w.overflow {
        return Err(PageReadError::Decode(
            "page exceeds the byte cap before allocation".into(),
        ));
    }
    Ok(())
}

/// Decode + admit one page payload. Every identity is re-derived and
/// checked; nothing is trusted from the bytes.
pub fn decode_page(bytes: &[u8]) -> Result<(String, InferBodyFacts), PageReadError> {
    if bytes.len() > MAX_PAGE_BYTES {
        return Err(PageReadError::Decode("page exceeds the byte cap".into()));
    }
    let mut r = Reader::new(bytes);
    if r.take(PAGE_MAGIC.len())? != PAGE_MAGIC {
        return Err(PageReadError::Decode("bad page magic".into()));
    }
    let version = r.u8()?;
    if version != PAGE_CODEC_VERSION {
        return Err(PageReadError::Decode(format!(
            "codec version {version} is not {PAGE_CODEC_VERSION}"
        )));
    }
    let key_canonical = r.s()?;
    let producer = r.s()?;
    if producer != SQ_PRODUCER {
        return Err(PageReadError::Envelope { found: producer });
    }
    let schema = r.s()?;
    if schema != SQ_SCHEMA {
        return Err(PageReadError::Envelope { found: schema });
    }
    let profile = r.s()?;
    if profile != SQ_PROFILE {
        return Err(PageReadError::Envelope { found: profile });
    }
    let algo = r.s()?;
    if algo != SQ_ALGO {
        return Err(PageReadError::Envelope { found: algo });
    }
    let name = r.s()?;
    let logical = r.s()?;
    let body_version = r.s()?;
    let revision = r.u64()?;
    let steps = r.u64()?;

    let n = r.count("node")?;
    let mut nodes = r.vector(n, 6)?;
    for _ in 0..n {
        let id = crate::walk::NodeId(r.u32()?);
        let kind = kind_of(r.u8()?)?;
        let ty = r.ty()?;
        nodes.push(NodeFact { id, kind, ty });
    }
    let n = r.count("binding")?;
    let mut bindings = r.vector(n, 9)?;
    for _ in 0..n {
        bindings.push(BindingFact {
            node: crate::walk::NodeId(r.u32()?),
            name: r.s()?,
            ty: r.ty()?,
        });
    }
    let n = r.count("diag")?;
    let mut diags = r.vector(n, 9)?;
    for _ in 0..n {
        let code = r.s()?;
        let node = match r.u8()? {
            0 => None,
            1 => Some(crate::walk::NodeId(r.u32()?)),
            k => return Err(PageReadError::Decode(format!("bad diag tag {k}"))),
        };
        diags.push(PortableDiag {
            code,
            node,
            message: r.s()?,
        });
    }
    let n = r.count("missing")?;
    let mut missing = r.vector(n, 4)?;
    for _ in 0..n {
        missing.push(crate::walk::NodeId(r.u32()?));
    }
    let n = r.count("sig-read")?;
    let mut sig_reads = r.vector(n, 4)?;
    for _ in 0..n {
        sig_reads.push(r.s()?);
    }
    let n = r.count("negative")?;
    let mut negatives = r.vector(n, 8)?;
    for _ in 0..n {
        negatives.push(NegativeDep {
            domain: r.s()?,
            name: r.s()?,
        });
    }
    if r.at != bytes.len() {
        return Err(PageReadError::Decode(
            "trailing bytes after the page".into(),
        ));
    }
    Ok((
        key_canonical,
        InferBodyFacts {
            name,
            logical: ace_semantics_contract::LogicalId(logical),
            body_version: ace_semantics_contract::ContentVersion(body_version),
            profile: SQ_PROFILE,
            algo: SQ_ALGO,
            schema: SQ_SCHEMA,
            producer: SQ_PRODUCER,
            revision,
            nodes,
            bindings,
            diags,
            missing,
            sig_reads,
            negatives,
            steps,
        },
    ))
}

// -- sink ---------------------------------------------------------------

/// The unified-store page sink.
pub struct ManagedStoreSink {
    store: ManagedStore,
    project: Option<String>,
}

impl ManagedStoreSink {
    pub fn open(
        root: &Path,
        project: Option<&str>,
    ) -> Result<Self, ace_artifacts::store::StoreError> {
        Ok(ManagedStoreSink {
            store: ManagedStore::open(root)?,
            project: project.map(|s| s.to_string()),
        })
    }

    pub fn from_store(store: ManagedStore, project: Option<&str>) -> Self {
        ManagedStoreSink {
            store,
            project: project.map(|s| s.to_string()),
        }
    }

    pub fn store(&self) -> &ManagedStore {
        &self.store
    }

    /// Look up by the complete key (the native store lookup).
    pub fn get_by_key(&self, key: &QueryKey) -> Result<Option<InferBodyFacts>, PageReadError> {
        let sk = store_key(key);
        let loaded = self
            .store
            .get_artifact(SQ_LAYER, &sk)
            .map_err(|e| PageReadError::Store(e.to_string()))?;
        match loaded {
            None => Ok(None),
            Some(a) => {
                let (page_key, facts) = decode_page(&a.payload)?;
                if page_key != key.canonical() {
                    return Err(PageReadError::KeyMismatch {
                        expected: key.canonical(),
                        found: page_key,
                    });
                }
                Ok(Some(facts))
            }
        }
    }

    /// Read by artifact id and admit against the caller's expected key.
    pub fn get_checked(
        &self,
        artifact: &str,
        expected: &QueryKey,
    ) -> Result<InferBodyFacts, PageReadError> {
        let loaded = self
            .store
            .get_artifact_by_id(artifact)
            .map_err(|e| PageReadError::Store(e.to_string()))?
            .ok_or(PageReadError::Missing)?;
        let (page_key, facts) = decode_page(&loaded.payload)?;
        if page_key != expected.canonical() {
            return Err(PageReadError::KeyMismatch {
                expected: expected.canonical(),
                found: page_key,
            });
        }
        Ok(facts)
    }
}

impl crate::host::PageSink for ManagedStoreSink {
    fn put(&mut self, key: &QueryKey, facts: &InferBodyFacts) -> Result<String, String> {
        // PLAN-999 (T-06, D-950-1): republishing the SAME semantics under
        // the SAME complete key is IDEMPOTENT. The per-request `revision`
        // stamped into a candidate is live view data (the session/host own
        // the current revision), never a semantic fact — an existing page
        // whose payload equals the candidate on every other field IS this
        // page: return its artifact id instead of re-putting. The old path
        // re-put with a fresh revision byte stream, hit the store's
        // same-key-different-content Conflict, and rejected identical
        // semantics. Content that genuinely differs still falls through to
        // the real put and keeps the store's hard Conflict.
        let sk = store_key(key);
        if let Ok(Some(existing)) = self.store.get_artifact(SQ_LAYER, &sk) {
            if let Ok((_, page)) = decode_page(&existing.payload) {
                if facts_eq_mod_revision(&page, facts) {
                    if let Some(project) = &self.project {
                        self.store
                            .register_root(project, &existing.artifact_id, SQ_ROOT_KIND)
                            .map_err(|e| e.to_string())?;
                    }
                    return Ok(existing.artifact_id);
                }
            }
        }
        let payload = encode_page(key, facts).map_err(|e| e.to_string())?;
        let req = PutRequest {
            layer: SQ_LAYER,
            key: store_key(key),
            kind: SQ_CARRIER,
            payload,
            edges: Vec::new(),
            project: self.project.clone(),
        };
        let art = self.store.put_artifact(req).map_err(|e| e.to_string())?;
        // A published page is a soft root of its project, so the store's
        // GC/lease machinery owns its lifetime (reclamation belongs to the
        // store, not to a second framework here).
        if let Some(project) = &self.project {
            self.store
                .register_root(project, &art, SQ_ROOT_KIND)
                .map_err(|e| e.to_string())?;
        }
        Ok(art)
    }

    fn get(&self, artifact: &str) -> Option<InferBodyFacts> {
        self.store
            .get_artifact_by_id(artifact)
            .ok()
            .flatten()
            .and_then(|a| decode_page(&a.payload).ok().map(|(_, f)| f))
    }
}

/// The store key of one complete query key: the key's canonical hash rides
/// the single component, so the store index and our identity agree.
pub fn store_key(key: &QueryKey) -> StoreQueryKey {
    StoreQueryKey {
        layer: SQ_LAYER,
        components: vec![("sq-key".to_string(), key.canonical())],
    }
}

/// PLAN-999 (T-06): page-semantic equality — every field of
/// [`InferBodyFacts`] except the per-request `revision` (live view data the
/// session/host own; a stored page's revision is the first publication's
/// stamp, never current authorization). Used by the sink's idempotent
/// republish check.
fn facts_eq_mod_revision(a: &InferBodyFacts, b: &InferBodyFacts) -> bool {
    a.name == b.name
        && a.logical == b.logical
        && a.body_version == b.body_version
        && a.profile == b.profile
        && a.algo == b.algo
        && a.schema == b.schema
        && a.producer == b.producer
        && a.steps == b.steps
        && a.nodes == b.nodes
        && a.bindings == b.bindings
        && a.diags == b.diags
        && a.missing == b.missing
        && a.sig_reads == b.sig_reads
        && a.negatives == b.negatives
}
