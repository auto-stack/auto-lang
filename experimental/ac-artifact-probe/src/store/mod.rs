//! Isolated managed artifact store (PLAN-783 §5.4, T-05).
//!
//! One experimental store = ownership marker + block layer + SQLite
//! metadata + cross-process locks. It is deliberately separate from any
//! production cache; the block layer reuses `auto-cache`'s `BlobStore`
//! through an isolated adapter (the fixed-temp-name write is serialized by
//! a per-chunk cross-process file lock — the audit point for SD-04).
//!
//! Publish protocol (fault points `AC_PROBE_FAULT`):
//!   1. stage framed payload under `staging/<txn>/`          [F1 after-staging]
//!   2. per-chunk lock → move into the blob CAS
//!   3. single SQLite txn: chunk row + artifact + edges + root
//!                                                            [F2 after-chunk-move]
//! Recovery (`recover`): staging leftovers, orphan blobs (no committed
//! row), interrupted tombstone deletes. Tombstone gates:
//!   [F3 after-tombstone] — GC marks, dies, restart completes.
//!
//! Reader/GC interlock: readers hold a shared lock on the chunk's gate
//! file; GC deletes only under the exclusive lock; a new reader seeing a
//! tombstone gets a controlled miss (recompute), never a partial read.

use crate::codec::{frame_chunk, unframe_chunk, ChunkKind, CodecError};
use crate::identity::{ArtifactId, ChunkId, QueryKey, RefKind};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

pub mod gc;

pub use gc::{gc_config_default, run_gc, GcConfig, GcReceipt, GC_HIGH, GC_LOW};

#[derive(Debug)]
pub enum StoreError {
    Io(String),
    Sql(String),
    Codec(CodecError),
    /// Two producers published different content under one (layer, key).
    /// Never last-writer-wins: the loser sees both content ids.
    Conflict { layer: String, key: String, existing: String, attempted: String },
    /// The chunk is tombstoned (logically invalidated); callers recompute.
    TombstonedMiss(String),
    NotManaged(PathBuf),
    ResourceExhausted(String),
    Boundary(String),
    Internal(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "io: {}", e),
            StoreError::Sql(e) => write!(f, "sqlite: {}", e),
            StoreError::Codec(e) => write!(f, "codec: {}", e),
            StoreError::Conflict { layer, key, existing, attempted } => write!(
                f,
                "key conflict on {}/{}: existing {} vs attempted {} (uncertainty reported, not overwritten)",
                layer, key, existing, attempted
            ),
            StoreError::TombstonedMiss(c) => write!(f, "chunk {} tombstoned — recompute", c),
            StoreError::NotManaged(p) => write!(f, "not a managed store: {}", p.display()),
            StoreError::ResourceExhausted(d) => write!(f, "resource exhausted: {}", d),
            StoreError::Boundary(d) => write!(f, "store boundary violation: {}", d),
            StoreError::Internal(d) => write!(f, "internal: {}", d),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e.to_string())
    }
}
impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sql(e.to_string())
    }
}
impl From<CodecError> for StoreError {
    fn from(e: CodecError) -> Self {
        StoreError::Codec(e)
    }
}

impl From<auto_cache::storage::Error> for StoreError {
    fn from(e: auto_cache::storage::Error) -> Self {
        StoreError::Io(e.to_string())
    }
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// True for reparse points (junctions/symlinks). Scans and deletes NEVER
/// traverse these — a store-internal junction pointing outside must not
/// turn into an out-of-root delete (PLAN-783 §5.4 目录边界).
pub fn is_reparse(p: &Path) -> bool {
    std::fs::symlink_metadata(p)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Cross-process file locks (OS truth for lease liveness / write turns)
// ---------------------------------------------------------------------------

#[cfg(windows)]
mod flock {
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::os::windows::io::AsRawHandle;
    use std::path::Path;
    use windows_sys::Win32::System::IO::OVERLAPPED;
    use windows_sys::Win32::Storage::FileSystem::{
        LockFileEx, UnlockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
    };

    pub struct FileGuard {
        file: File,
    }

    impl FileGuard {
        pub fn try_lock(path: &Path, exclusive: bool) -> io::Result<Option<FileGuard>> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
            let flags: u32 = LOCKFILE_FAIL_IMMEDIATELY
                | if exclusive { LOCKFILE_EXCLUSIVE_LOCK } else { 0 };
            let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
            let ok = unsafe {
                LockFileEx(file.as_raw_handle() as _, flags, 0, u32::MAX, u32::MAX, &mut overlapped)
            };
            if ok != 0 {
                return Ok(Some(FileGuard { file }));
            }
            let err = io::Error::last_os_error();
            // 32 ERROR_SHARING_VIOLATION / 33 ERROR_LOCK_VIOLATION: the
            // byte range is held by someone else — a clean "busy".
            if err.raw_os_error() == Some(32) || err.raw_os_error() == Some(33) {
                return Ok(None);
            }
            Err(err)
        }
    }

    impl Drop for FileGuard {
        fn drop(&mut self) {
            let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
            unsafe {
                UnlockFileEx(self.file.as_raw_handle() as _, 0, u32::MAX, u32::MAX, &mut overlapped)
            };
        }
    }
}

#[cfg(not(windows))]
mod flock {
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::path::Path;

    pub struct FileGuard {
        _file: File,
    }

    impl FileGuard {
        pub fn try_lock(path: &Path, _exclusive: bool) -> io::Result<Option<FileGuard>> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
            Ok(Some(FileGuard { _file: file }))
        }
    }
}

pub use flock::FileGuard;

// ---------------------------------------------------------------------------
// Managed store
// ---------------------------------------------------------------------------

pub const MARKER_NAME: &str = "AC783-STORE.marker";
pub const STORE_VERSION: u32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
struct Marker {
    store_id: String,
    version: u32,
    created_epoch: u64,
}

pub struct ManagedStore {
    pub root: PathBuf,
    canonical_root: PathBuf,
    db: Mutex<Connection>,
    /// Isolated adapter over auto-cache's BlobStore (SD-04 audit point).
    blobs: auto_cache::storage::BlobStore,
}

/// Classified on-disk accounting: blocks, DB/WAL, staging, trash, locks,
/// logs — everything counts (PLAN-783 §5.4).
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct SpaceReport {
    pub blocks: u64,
    pub blocks_count: u64,
    pub db: u64,
    pub staging: u64,
    pub trash: u64,
    pub locks: u64,
    pub logs: u64,
    pub total: u64,
}

pub struct PutRequest {
    pub layer: &'static str,
    pub key: QueryKey,
    pub kind: ChunkKind,
    pub payload: Vec<u8>,
    /// (target artifact id, kind) — required edges keep targets live.
    pub edges: Vec<(String, RefKind)>,
    /// Project soft-root registration (None = store-internal artifact).
    pub project: Option<String>,
}

#[derive(Clone, Debug)]
pub struct LoadedArtifact {
    pub artifact_id: String,
    pub chunk_id: String,
    pub kind: ChunkKind,
    pub payload: Vec<u8>,
}

impl ManagedStore {
    /// Open (creating when absent) an isolated managed store.
    pub fn open(root: &Path) -> Result<Self, StoreError> {
        fs::create_dir_all(root)?;
        for d in ["blocks", "staging", "locks", "trash", "logs"] {
            fs::create_dir_all(root.join(d))?;
        }
        let marker_path = root.join(MARKER_NAME);
        let marker = if marker_path.is_file() {
            serde_json::from_str::<Marker>(&fs::read_to_string(&marker_path)?)
                .map_err(|_| StoreError::NotManaged(root.to_path_buf()))?
        } else {
            let m = Marker {
                store_id: format!("st-{}-{}", now_ms(), crate::sysinfo::current_pid()),
                version: STORE_VERSION,
                created_epoch: now_secs(),
            };
            fs::write(&marker_path, serde_json::to_string(&m).unwrap())?;
            m
        };
        if marker.version != STORE_VERSION {
            return Err(StoreError::NotManaged(root.to_path_buf()));
        }
        let db = Connection::open(root.join("journal.db"))?;
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.pragma_update(None, "synchronous", "NORMAL")?;
        db.pragma_update(None, "busy_timeout", 8000)?;
        init_schema(&db)?;
        let canonical_root = fs::canonicalize(root)?;
        Ok(ManagedStore {
            root: root.to_path_buf(),
            canonical_root,
            db: Mutex::new(db),
            blobs: auto_cache::storage::BlobStore::new(root.join("blocks")),
        })
    }

    /// Open refusing to create: operations that delete require the marker.
    pub fn open_existing(root: &Path) -> Result<Self, StoreError> {
        if !root.join(MARKER_NAME).is_file() {
            return Err(StoreError::NotManaged(root.to_path_buf()));
        }
        Self::open(root)
    }

    /// A deletion candidate must canonically stay inside this store.
    fn check_inside(&self, p: &Path) -> Result<PathBuf, StoreError> {
        let canon = fs::canonicalize(p).map_err(|e| StoreError::Io(e.to_string()))?;
        if canon.starts_with(&self.canonical_root) {
            Ok(canon)
        } else {
            Err(StoreError::Boundary(format!("path escapes store root: {}", canon.display())))
        }
    }

    fn lock_path(&self, name: &str) -> PathBuf {
        self.root.join("locks").join(format!("{}.lock", name))
    }

    // -- space accounting ---------------------------------------------------

    fn dir_size(p: &Path) -> u64 {
        let Ok(rd) = fs::read_dir(p) else { return 0 };
        let mut t = 0;
        for e in rd.flatten() {
            let path = e.path();
            if is_reparse(&path) {
                continue; // never traverse out of the managed tree
            }
            if path.is_dir() {
                t += Self::dir_size(&path);
            } else if path.is_file() {
                t += e.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
        t
    }

    pub fn space(&self) -> SpaceReport {
        let blocks = self.blobs.total_size().unwrap_or(0);
        let blocks_count = self.blobs.count().unwrap_or(0) as u64;
        let db = ["journal.db", "journal.db-wal", "journal.db-shm"]
            .iter()
            .map(|f| self.root.join(f))
            .filter(|p| p.is_file())
            .map(|p| p.metadata().map(|m| m.len()).unwrap_or(0))
            .sum();
        let staging = Self::dir_size(&self.root.join("staging"));
        let trash = Self::dir_size(&self.root.join("trash"));
        let locks = Self::dir_size(&self.root.join("locks"));
        let logs = Self::dir_size(&self.root.join("logs"));
        SpaceReport {
            blocks,
            blocks_count,
            db,
            staging,
            trash,
            locks,
            logs,
            total: blocks + db + staging + trash + locks + logs,
        }
    }

    // -- publish -------------------------------------------------------------

    /// Deterministic artifact content identity.
    fn artifact_id(req: &PutRequest, chunk_id: &str) -> String {
        let mut parts: Vec<Vec<u8>> = vec![
            req.layer.as_bytes().to_vec(),
            req.key.hash().into_bytes(),
            chunk_id.as_bytes().to_vec(),
            format!("{:?}", req.kind).into_bytes(),
        ];
        let mut edges: Vec<String> = req
            .edges
            .iter()
            .map(|(a, k)| format!("{}:{}", a, k.as_str()))
            .collect();
        edges.sort();
        for e in edges {
            parts.push(e.into_bytes());
        }
        let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
        ArtifactId::from_parts(&refs).0
    }

    /// Staging/temp budget (§5.4): in-flight staging above the cap is a
    /// controlled resource rejection (bounded backoff / stop-write).
    pub const STAGING_BUDGET: u64 = 8 * 1024 * 1024;

    pub fn put_artifact(&self, req: PutRequest) -> Result<String, StoreError> {
        let space = self.space();
        if space.staging + req.payload.len() as u64 > Self::STAGING_BUDGET {
            return Err(StoreError::ResourceExhausted(format!(
                "staging {} + new {} exceeds budget {}",
                space.staging,
                req.payload.len(),
                Self::STAGING_BUDGET
            )));
        }
        let framed = frame_chunk(req.kind, &req.payload);
        let chunk_id = ChunkId::from_parts(&[framed.as_slice()]).0;
        let art_id = Self::artifact_id(&req, &chunk_id);

        // 1. Stage.
        let txn = format!("t{}-{}", now_ms(), crate::sysinfo::current_pid());
        let staging_dir = self.root.join("staging").join(&txn);
        fs::create_dir_all(&staging_dir)?;
        let staged_file = staging_dir.join("payload");
        fs::write(&staged_file, &framed)?;
        fault("after-staging");

        // 2. Per-chunk cross-process lock → CAS insert. This serializes the
        //    reused BlobStore's fixed temp name for identical content and
        //    gives same-chunk producers a defined write turn.
        let _chunk_lock = FileGuard::try_lock(&self.lock_path(&format!("chunk-{}", chunk_id)), true)?
            .ok_or_else(|| StoreError::Internal("chunk lock unavailable".into()))?;
        self.blobs.put(&chunk_id, &staged_file)?;
        fault("after-chunk-move");

        // 3. Atomic metadata publish.
        let db = self.db.lock().unwrap();
        let tx = db.unchecked_transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO chunks(chunk_id, kind, size, state, created_epoch) VALUES (?1, ?2, ?3, 'live', ?4)",
            params![chunk_id, req.kind as u8, framed.len() as u64, now_secs()],
        )?;
        let existing: Option<(String, String)> = tx
            .query_row(
                "SELECT artifact_id, payload_chunk FROM artifacts WHERE layer=?1 AND key_hash=?2",
                params![req.layer, req.key.hash()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        // A zombie row (its chunk was physically deleted) is not a live
        // claim: republication replaces it instead of conflicting forever.
        let existing_is_dead = existing.as_ref().is_some_and(|(_, cid)| {
            let state: Option<String> = tx
                .query_row("SELECT state FROM chunks WHERE chunk_id=?1", params![cid], |r| r.get(0))
                .optional()
                .unwrap_or(None);
            !matches!(state.as_deref(), Some("live"))
        });
        let existing = if existing_is_dead {
            tx.execute(
                "DELETE FROM edges WHERE src IN (SELECT artifact_id FROM artifacts WHERE layer=?1 AND key_hash=?2)",
                params![req.layer, req.key.hash()],
            )?;
            tx.execute(
                "DELETE FROM artifacts WHERE layer=?1 AND key_hash=?2",
                params![req.layer, req.key.hash()],
            )?;
            None
        } else {
            existing
        };
        match &existing {
            None => {
                tx.execute(
                    "INSERT INTO artifacts(artifact_id, layer, key_hash, key_canonical, payload_chunk, created_epoch) VALUES (?1,?2,?3,?4,?5,?6)",
                    params![art_id, req.layer, req.key.hash(), req.key.canonical(), chunk_id, now_secs()],
                )?;
                for (dst, kind) in &req.edges {
                    tx.execute(
                        "INSERT OR IGNORE INTO edges(src, dst, kind) VALUES (?1, ?2, ?3)",
                        params![art_id, dst, kind.as_str()],
                    )?;
                }
                if let Some(p) = &req.project {
                    tx.execute(
                        "INSERT OR IGNORE INTO roots(project, artifact, kind) VALUES (?1, ?2, 'soft')",
                        params![p, art_id],
                    )?;
                }
            }
            Some((id, cid)) if id == &art_id && cid == &chunk_id => {
                // Idempotent republication of identical content.
            }
            Some((id, _)) => {
                // Different content under one key: commit the (shared)
                // chunk row, then report the conflict — never overwrite.
                tx.execute(
                    "INSERT OR IGNORE INTO txns(txn_id, phase, detail, epoch) VALUES (?1, 'conflict', ?2, ?3)",
                    params![txn, format!("{} vs {}", id, art_id), now_secs()],
                )?;
                tx.commit()?;
                drop(_chunk_lock);
                let _ = fs::remove_dir_all(&staging_dir);
                return Err(StoreError::Conflict {
                    layer: req.layer.into(),
                    key: req.key.canonical(),
                    existing: id.clone(),
                    attempted: art_id,
                });
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO txns(txn_id, phase, detail, epoch) VALUES (?1, 'committed', '', ?2)",
            params![txn, now_secs()],
        )?;
        tx.commit()?;
        drop(db);
        drop(_chunk_lock);
        let _ = fs::remove_dir_all(&staging_dir);
        Ok(art_id)
    }

    /// Lookup by (layer, key); a tombstoned chunk reads as a miss (the
    /// caller recomputes) — never a partial read and never an error.
    pub fn get_artifact(&self, layer: &str, key: &QueryKey) -> Result<Option<LoadedArtifact>, StoreError> {
        let row: Option<(String, String, u8)> = self
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT a.artifact_id, a.payload_chunk, c.kind FROM artifacts a JOIN chunks c ON c.chunk_id = a.payload_chunk WHERE a.layer=?1 AND a.key_hash=?2",
                params![layer, key.hash()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let Some((art_id, chunk_id, kind_byte)) = row else { return Ok(None) };
        let kind = ChunkKind::from_code(kind_byte).ok_or_else(|| StoreError::Internal("bad kind".into()))?;
        match self.read_chunk_gated(&chunk_id) {
            Ok(Some(bytes)) => {
                let (k, payload) = unframe_chunk(&bytes)?;
                if k != kind {
                    return Err(StoreError::Internal(format!("kind mismatch for {}", chunk_id)));
                }
                Ok(Some(LoadedArtifact { artifact_id: art_id, chunk_id, kind, payload: payload.to_vec() }))
            }
            Ok(None) => Ok(None),
            Err(StoreError::TombstonedMiss(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Reader-side interlock: shared gate → tombstone check → read.
    pub fn read_chunk_gated(&self, chunk_id: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let gate = FileGuard::try_lock(&self.lock_path(&format!("gate-{}", chunk_id)), false)?
            .ok_or_else(|| StoreError::Internal("gate lock unavailable".into()))?;
        let state: Option<String> = self
            .db
            .lock()
            .unwrap()
            .query_row("SELECT state FROM chunks WHERE chunk_id=?1", params![chunk_id], |r| r.get(0))
            .optional()?;
        match state.as_deref() {
            None => return Ok(None), // unknown chunk = miss
            Some("deleted") => return Ok(None),
            Some("tombstoned") => return Err(StoreError::TombstonedMiss(chunk_id.into())),
            Some("live") => {}
            Some(other) => return Err(StoreError::Internal(format!("bad chunk state {}", other))),
        }
        let Some(path) = self.blobs.get(chunk_id) else { return Ok(None) };
        let bytes = fs::read(&path)?;
        drop(gate);
        Ok(Some(bytes))
    }

    // -- roots / pins / leases ------------------------------------------------

    pub fn register_root(&self, project: &str, artifact: &str, kind: &str) -> Result<(), StoreError> {
        self.db.lock().unwrap().execute(
            "INSERT OR IGNORE INTO roots(project, artifact, kind) VALUES (?1, ?2, ?3)",
            params![project, artifact, kind],
        )?;
        Ok(())
    }

    pub fn remove_project_roots(&self, project: &str) -> Result<usize, StoreError> {
        let n = self.db.lock().unwrap().execute(
            "DELETE FROM roots WHERE project=?1 AND kind='soft'",
            params![project],
        )?;
        Ok(n)
    }

    /// Remove roots of one kind (test/lifecycle seam: unpinning hard pins).
    pub fn remove_roots(&self, project: &str, kind: &str) -> Result<usize, StoreError> {
        let n = self.db.lock().unwrap().execute(
            "DELETE FROM roots WHERE project=?1 AND kind=?2",
            params![project, kind],
        )?;
        Ok(n)
    }

    /// Drop all artifact metadata rows of a layer (test seam for forced
    /// misses: the payloads stay until GC reclaims them as unreachable).
    pub fn purge_layer(&self, layer: &str) -> Result<usize, StoreError> {
        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM edges WHERE src IN (SELECT artifact_id FROM artifacts WHERE layer=?1)", params![layer])?;
        let n = db.execute("DELETE FROM roots WHERE artifact IN (SELECT artifact_id FROM artifacts WHERE layer=?1)", params![layer])?;
        let _ = n;
        let m = db.execute("DELETE FROM artifacts WHERE layer=?1", params![layer])?;
        Ok(m)
    }

    pub fn acquire_lease(&self, purpose: &str, ttl_secs: u64, artifact: Option<&str>) -> Result<Lease, StoreError> {
        let id = format!("ls-{}-{}", now_ms(), crate::sysinfo::current_pid());
        // Hold the lock file for the lease's lifetime (OS releases it when
        // the holder dies — TTL/PID alone never decide liveness).
        let guard = FileGuard::try_lock(&self.lock_path(&format!("lease-{}", id)), true)?
            .ok_or_else(|| StoreError::Internal("lease lock unavailable".into()))?;
        self.db.lock().unwrap().execute(
            "INSERT INTO leases(lease_id, purpose, artifact, lock_path, acquired_epoch, heartbeat_epoch, ttl_secs, released) VALUES (?1,?2,?3,?4,?5,?5,?6,0)",
            params![id, purpose, artifact, self.lock_path(&format!("lease-{}", id)).display().to_string(), now_secs(), ttl_secs],
        )?;
        Ok(Lease { store_root: self.root.clone(), lease_id: id, _guard: guard })
    }

    /// A lease is live iff its row is unreleased AND its lock file is still
    /// held by someone (acquiring it proves the holder is gone).
    pub fn lease_live(&self, lease_id: &str) -> Result<bool, StoreError> {
        let row: Option<(u8, String)> = self
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT released, lock_path FROM leases WHERE lease_id=?1",
                params![lease_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((released, lock_path)) = row else { return Ok(false) };
        if released != 0 {
            return Ok(false);
        }
        match FileGuard::try_lock(Path::new(&lock_path), true) {
            Ok(Some(_won)) => Ok(false), // lock was free → holder dead
            Ok(None) => Ok(true),        // still held → live even past TTL
            Err(e) => Err(StoreError::Io(e.to_string())),
        }
    }

    /// Drop leases whose holders provably died (lock file acquirable).
    pub fn sweep_stale_leases(&self) -> Result<usize, StoreError> {
        let ids: Vec<(String, String)> = self
            .db
            .lock()
            .unwrap()
            .prepare("SELECT lease_id, lock_path FROM leases WHERE released=0")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .flatten()
            .collect();
        let mut n = 0;
        for (id, lock_path) in ids {
            if let Ok(Some(_won)) = FileGuard::try_lock(Path::new(&lock_path), true) {
                self.db.lock().unwrap().execute(
                    "UPDATE leases SET released=1 WHERE lease_id=?1",
                    params![id],
                )?;
                n += 1;
            }
        }
        Ok(n)
    }

    pub fn log_line(&self, line: &str) -> Result<(), StoreError> {
        let p = self.root.join("logs").join("store.log");
        use std::io::Write;
        let mut f = fs::OpenOptions::new().create(true).append(true).open(p)?;
        writeln!(f, "{}", line)?;
        Ok(())
    }

    pub(crate) fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.db.lock().unwrap()
    }

    pub(crate) fn blob_remove(&self, chunk_id: &str) -> Result<bool, StoreError> {
        let path = self.blobs.get(chunk_id);
        let Some(p) = path else { return Ok(false) };
        self.check_inside(&p)?;
        // Trash-move first: a blocked final remove leaves a reclaimable
        // state instead of a half-deleted block.
        let trash = self.root.join("trash").join(chunk_id);
        fs::rename(&p, &trash)?;
        Ok(fs::remove_file(&trash).is_ok())
    }
}

/// A held lease. Drop releases the OS lock (the row stays for receipts).
pub struct Lease {
    pub store_root: PathBuf,
    pub lease_id: String,
    _guard: FileGuard,
}

impl Lease {
    /// Explicit release: mark the row and drop the lock.
    pub fn release(self) {
        if let Ok(store) = ManagedStore::open_existing(&self.store_root) {
            let _ = store
                .db()
                .execute("UPDATE leases SET released=1 WHERE lease_id=?1", params![self.lease_id]);
        }
    }
}

impl ChunkKind {
    pub(crate) fn from_code(v: u8) -> Option<ChunkKind> {
        match v {
            0 => Some(ChunkKind::Hir),
            1 => Some(ChunkKind::DeclSummary),
            2 => Some(ChunkKind::Object),
            3 => Some(ChunkKind::SnapshotRecord),
            4 => Some(ChunkKind::ObjectManifest),
            5 => Some(ChunkKind::LayoutData),
            _ => None,
        }
    }
}

fn init_schema(db: &Connection) -> Result<(), StoreError> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS chunks(
            chunk_id TEXT PRIMARY KEY,
            kind INTEGER NOT NULL,
            size INTEGER NOT NULL,
            state TEXT NOT NULL DEFAULT 'live',
            created_epoch INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS artifacts(
            artifact_id TEXT PRIMARY KEY,
            layer TEXT NOT NULL,
            key_hash TEXT NOT NULL,
            key_canonical TEXT NOT NULL,
            payload_chunk TEXT NOT NULL,
            created_epoch INTEGER NOT NULL,
            UNIQUE(layer, key_hash));
        CREATE TABLE IF NOT EXISTS edges(
            src TEXT NOT NULL,
            dst TEXT NOT NULL,
            kind TEXT NOT NULL,
            PRIMARY KEY(src, dst, kind));
        CREATE TABLE IF NOT EXISTS roots(
            project TEXT NOT NULL,
            artifact TEXT NOT NULL,
            kind TEXT NOT NULL,
            created_epoch INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY(project, artifact, kind));
        CREATE TABLE IF NOT EXISTS leases(
            lease_id TEXT PRIMARY KEY,
            purpose TEXT NOT NULL,
            artifact TEXT,
            lock_path TEXT NOT NULL,
            acquired_epoch INTEGER NOT NULL,
            heartbeat_epoch INTEGER NOT NULL,
            ttl_secs INTEGER NOT NULL,
            released INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS txns(
            txn_id TEXT PRIMARY KEY,
            phase TEXT NOT NULL,
            detail TEXT NOT NULL DEFAULT '',
            epoch INTEGER NOT NULL);
        ",
    )?;
    Ok(())
}

/// Fault injection for the three publish/reclaim crash points.
fn fault(name: &str) {
    if std::env::var("AC_PROBE_FAULT").as_deref() == Ok(name) {
        // Abort without running destructors: the locks stay OS-released,
        // exactly like a hard crash.
        eprintln!("AC_PROBE_FAULT={} — aborting", name);
        std::process::abort();
    }
}

// ---------------------------------------------------------------------------
// Recovery
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct RecoveryReceipt {
    pub staging_dirs_removed: usize,
    pub orphan_chunks_removed: usize,
    pub tombstone_deletes_completed: usize,
    pub aborted_txn_rows: usize,
    pub freed_bytes: u64,
}

/// Reconcile a store after crashes. `assumed_exclusive` skips the staging
/// grace period (operator- or test-invoked right after a crash, with no
/// concurrent producers); the default mode only reclaims staging older
/// than the grace window.
pub fn recover(store: &ManagedStore, assumed_exclusive: bool) -> Result<RecoveryReceipt, StoreError> {
    let mut receipt = RecoveryReceipt::default();
    // 1. Staging leftovers.
    for entry in fs::read_dir(store.root.join("staging"))?.flatten() {
        let p = entry.path();
        if is_reparse(&p) {
            continue; // never follow a reparse jump out of the store
        }
        if p.is_dir() {
            let age = now_secs().saturating_sub(
                entry.metadata().ok().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0),
            );
            if assumed_exclusive || age > 600 {
                let sz = ManagedStore::dir_size(&p);
                fs::remove_dir_all(&p)?;
                receipt.staging_dirs_removed += 1;
                receipt.freed_bytes += sz;
            }
        }
    }
    // 2. Orphan blobs: present on disk, no committed chunk row. Blob file
    //    names ARE the chunk ids (BlobStore sharding is internal), so scan
    //    the CAS via a temporary listing through the blocks dir.
    let committed: std::collections::HashSet<String> = store
        .db()
        .prepare("SELECT chunk_id FROM chunks")?
        .query_map([], |r| r.get::<_, String>(0))?
        .flatten()
        .collect();
    let blocks_dir = store.root.join("blocks").join("blobs");
    if blocks_dir.is_dir() {
        for shard in fs::read_dir(&blocks_dir)?.flatten() {
            let shard_path = shard.path();
            if !shard_path.is_dir() || is_reparse(&shard_path) {
                continue;
            }
            for blob in fs::read_dir(&shard_path)?.flatten() {
                if is_reparse(&blob.path()) {
                    continue; // reparse jump: report, never delete through
                }
                let name = blob.file_name().to_string_lossy().to_string();
                if name.ends_with(".tmp") {
                    // Interrupted CAS write; safe to reclaim.
                    let sz = blob.metadata().map(|m| m.len()).unwrap_or(0);
                    let _ = fs::remove_file(blob.path());
                    receipt.orphan_chunks_removed += 1;
                    receipt.freed_bytes += sz;
                    continue;
                }
                if !committed.contains(&name) {
                    let sz = blob.metadata().map(|m| m.len()).unwrap_or(0);
                    let _ = store.blob_remove(&name)?;
                    receipt.orphan_chunks_removed += 1;
                    receipt.freed_bytes += sz;
                }
            }
        }
    }
    // 3. Complete interrupted tombstone deletes.
    let tombstoned: Vec<(String, u64)> = store
        .db()
        .prepare("SELECT chunk_id, size FROM chunks WHERE state='tombstoned'")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .flatten()
        .collect();
    for (cid, sz) in tombstoned {
        let gate = FileGuard::try_lock(&store.lock_path(&format!("gate-{}", cid)), true)?;
        if let Some(_g) = gate {
            if store.blob_remove(&cid)? {
                receipt.freed_bytes += sz;
            }
            store.db().execute(
                "UPDATE chunks SET state='deleted' WHERE chunk_id=?1",
                params![cid],
            )?;
            receipt.tombstone_deletes_completed += 1;
        }
    }
    // 4. Journal hygiene: mark lingering staging-phase txns aborted.
    receipt.aborted_txn_rows = store.db().execute(
        "UPDATE txns SET phase='aborted' WHERE phase IN ('staging','conflict')",
        [],
    )?;
    Ok(receipt)
}
