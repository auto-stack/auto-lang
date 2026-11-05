//! The S936 executed-envelope sink (unified ManagedStore seam).
//!
//! Executed envelopes ride the SAME managed store every other layer
//! uses, under this lane's own layer id (`S936` — separate from the
//! `S934` fact pages and from the 817 `ac817/a05-cte-exec/v1` store):
//!
//! * the payload is the strict envelope codec (every identity
//!   re-validated on read — a foreign, truncated, tampered or old-schema
//!   page is a controlled rejection, never a reinterpretation);
//! * the store key is the execution key's content hash, so a lookup by
//!   key and the page's own embedded key agree (checked on every read);
//! * publication registers the page as a soft root of its project, so
//!   the store's GC/lease machinery owns its lifetime.
//!
//! In-memory sink for tests/cold lanes that must not observe residue
//! (the "owned store, proven empty before a real miss" discipline).

use crate::envelope::{decode_envelope, encode_envelope, CtExecutedV1, EnvelopeError};
use crate::identity::ExecKey;
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey as StoreQueryKey;
use ace_artifacts::store::{ManagedStore, PutRequest};
use std::path::Path;

/// The source-CTE layer id in the unified store.
pub const CT_LAYER: &str = "S936";
/// Root kind under which published envelopes are pinned.
pub const CT_ROOT_KIND: &str = "source-cte";
/// Envelope carrier (immutable record-like page, same carrier table as
/// the 934 pages).
pub const CT_CARRIER: ChunkKind = ChunkKind::SnapshotRecord;

pub const PAGE_MAGIC: &[u8; 6] = b"CT936P";
pub const PAGE_CODEC_VERSION: u8 = 1;
/// One envelope's byte ceiling (the plan's 512 KiB page cap axis).
pub const MAX_PAGE_BYTES: usize = 512 * 1024;

/// The sink abstraction: probe by key, publish, read back.
pub trait CtEnvelopeSink {
    fn probe(&self, key: &ExecKey) -> Result<Option<CtExecutedV1>, EnvelopeError>;
    fn publish(&mut self, key: &ExecKey, env: &CtExecutedV1) -> Result<(), EnvelopeError>;
}

/// The unified-store sink.
pub struct ManagedEnvelopeSink {
    store: ManagedStore,
    project: Option<String>,
}

impl ManagedEnvelopeSink {
    pub fn open(
        root: &Path,
        project: Option<&str>,
    ) -> Result<Self, ace_artifacts::store::StoreError> {
        Ok(ManagedEnvelopeSink {
            store: ManagedStore::open(root)?,
            project: project.map(|s| s.to_string()),
        })
    }

    pub fn from_store(store: ManagedStore, project: Option<&str>) -> Self {
        ManagedEnvelopeSink {
            store,
            project: project.map(|s| s.to_string()),
        }
    }

    pub fn store(&self) -> &ManagedStore {
        &self.store
    }
}

impl CtEnvelopeSink for ManagedEnvelopeSink {
    fn probe(&self, key: &ExecKey) -> Result<Option<CtExecutedV1>, EnvelopeError> {
        let loaded = self
            .store
            .get_artifact(CT_LAYER, &store_key(key))
            .map_err(|e| EnvelopeError::Decode(format!("store read failed: {e}")))?;
        match loaded {
            None => Ok(None),
            Some(a) => {
                let (page_key, env) = decode_envelope(&a.payload)?;
                if page_key != key.canonical() {
                    return Err(EnvelopeError::KeyMismatch {
                        expected: key.canonical(),
                        found: page_key,
                    });
                }
                Ok(Some(env))
            }
        }
    }

    fn publish(&mut self, key: &ExecKey, env: &CtExecutedV1) -> Result<(), EnvelopeError> {
        // Write-path admission: the sink never stores a page its own codec
        // would refuse on read (foreign schema/producer/tag = rejected
        // here, not written for the reader to trip over).
        let payload = encode_envelope(env)?;
        let (page_key, _) = decode_envelope(&payload)?;
        if page_key != key.canonical() {
            return Err(EnvelopeError::KeyMismatch {
                expected: key.canonical(),
                found: page_key,
            });
        }
        let req = PutRequest {
            layer: CT_LAYER,
            key: store_key(key),
            kind: CT_CARRIER,
            payload,
            edges: Vec::new(),
            project: self.project.clone(),
        };
        let art = self
            .store
            .put_artifact(req)
            .map_err(|e| EnvelopeError::Decode(format!("store put failed: {e}")))?;
        if let Some(project) = &self.project {
            self.store
                .register_root(project, &art, CT_ROOT_KIND)
                .map_err(|e| EnvelopeError::Decode(format!("root registration failed: {e}")))?;
        }
        Ok(())
    }
}

/// An in-memory sink (cold lanes/tests; a fresh instance is provably
/// empty — the owned-store-before-a-real-miss discipline).
#[derive(Default)]
pub struct MemoryEnvelopeSink {
    pages: std::collections::HashMap<String, CtExecutedV1>,
}

impl MemoryEnvelopeSink {
    pub fn new() -> Self {
        MemoryEnvelopeSink::default()
    }
}

impl CtEnvelopeSink for MemoryEnvelopeSink {
    fn probe(&self, key: &ExecKey) -> Result<Option<CtExecutedV1>, EnvelopeError> {
        match self.pages.get(&key.hash()) {
            None => Ok(None),
            Some(env) => {
                if env.exec_key_canonical != key.canonical() {
                    return Err(EnvelopeError::KeyMismatch {
                        expected: key.canonical(),
                        found: env.exec_key_canonical.clone(),
                    });
                }
                Ok(Some(env.clone()))
            }
        }
    }

    fn publish(&mut self, key: &ExecKey, env: &CtExecutedV1) -> Result<(), EnvelopeError> {
        // Re-validate on the write path too: the sink never stores a page
        // its own codec would refuse.
        let bytes = encode_envelope(env)?;
        let (page_key, _) = decode_envelope(&bytes)?;
        if page_key != key.canonical() {
            return Err(EnvelopeError::KeyMismatch {
                expected: key.canonical(),
                found: page_key,
            });
        }
        self.pages.insert(key.hash(), env.clone());
        Ok(())
    }
}

/// The store key of one execution key: the key's content hash rides the
/// single component.
pub fn store_key(key: &ExecKey) -> StoreQueryKey {
    StoreQueryKey {
        layer: CT_LAYER,
        components: vec![("ct-exec-key".to_string(), key.hash())],
    }
}
