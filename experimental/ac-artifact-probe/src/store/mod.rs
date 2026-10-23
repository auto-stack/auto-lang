//! Isolated managed artifact store (PLAN-783 §5.4, T-05).
//!
//! One experimental store = ownership marker + block layer + SQLite
//! metadata + cross-process locks. It is deliberately separate from any
//! production cache; the block layer reuses `auto-cache`'s `BlobStore`
//! through an isolated adapter (the fixed-temp-name write is serialized by
//! a per-chunk cross-process file lock — the audit point for SD-04).
//!
//! ## Liveness interlock (review R1 F-01/F-03/F-08)
//!
//! All metadata mutations that can change what stays alive (publish,
//! lease acquisition, root registration/replacement, purge) hold the
//! SHARED `liveness` lock while committing; GC and recovery hold it
//! EXCLUSIVELY for their whole decide-and-delete phase. A lease/root
//! therefore can never be granted for a chunk that a concurrent GC is
//! deleting: registration runs either before the GC snapshot (and is
//! counted) or after GC finished (and re-checks target liveness — a dead
//! target is a controlled `TargetMissing`). Same-key producers wait
//! bounded on the per-chunk lock instead of failing instantly, and every
//! failed publish reclaims its staging directory.
//!
//! ## Publish protocol (fault points `AC_PROBE_FAULT`)
//!
//!   1. stage framed payload under `staging/<txn>/`          [F1 after-staging]
//!   2. per-chunk lock (bounded wait) → move into the blob CAS
//!   3. shared liveness → single SQLite txn: chunk state
//!      upsert (deleted → live), artifact + edges + root      [F2 after-chunk-move]
//!
//! Reader/GC interlock: readers hold a shared lock on the chunk's gate
//! file; GC deletes only under the exclusive lock; a new reader seeing a
//! tombstone gets a controlled miss (recompute), never a partial read.
//! CAS admission: every read recomputes the framed-byte content identity
//! and refuses a mismatched address (review R1 F-04).

use crate::codec::{frame_chunk, unframe_chunk, ChunkKind, CodecError};
use crate::identity::{ArtifactId, ChunkId, QueryKey, RefKind};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, UNIX_EPOCH};

pub mod gc;

pub use gc::{gc_config_default, run_gc, GcConfig, GcReceipt, GC_HIGH, GC_LOW};

#[derive(Debug)]
pub enum StoreError {
    Io(String),
    Sql(String),
    Codec(CodecError),
    /// Two producers published different content under one (layer, key).
    /// Never last-writer-wins: the loser sees both content ids.
    Conflict {
        layer: String,
        key: String,
        existing: String,
        attempted: String,
    },
    /// The chunk is tombstoned (logically invalidated); callers recompute.
    TombstonedMiss(String),
    NotManaged(PathBuf),
    ResourceExhausted(String),
    Boundary(String),
    /// A coordination lock was not free within the bounded wait (§5.4:
    /// bounded backoff; the caller may retry).
    Busy(String),
    /// A lease/root/publication names an artifact whose payload is not
    /// live — the final atomic liveness decision refused it.
    TargetMissing(String),
    /// The blob at a CAS address does not hash to that address: content
    /// substitution is rejected at admission.
    ChunkAddressMismatch {
        requested: String,
        actual: String,
    },
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
            StoreError::Busy(d) => write!(f, "busy (bounded wait exceeded): {}", d),
            StoreError::TargetMissing(d) => write!(f, "target not live: {}", d),
            StoreError::ChunkAddressMismatch { requested, actual } => write!(
                f,
                "CAS address mismatch: requested {} but blob hashes to {}",
                requested, actual
            ),
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
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Process-global staging identity sequence (R2-03): millisecond + pid is
/// NOT unique across same-process threads.
static STAGING_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Process-global lease identity sequence (F787-R6-03): millisecond + pid is
/// NOT unique across same-process sequential grants either — released rows
/// keep their `lease_id` in the leases table, so same-millisecond sequential
/// acquire+release cycles collide on the PRIMARY KEY (91/128 observed).
/// Mirrors STAGING_SEQ: grant uniqueness comes from the sequence, liveness
/// stays decided by the OS lock file + TTL/PID as before.
static LEASE_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Process-wide high-water mark of RESERVED transient bytes (staging +
/// CAS tmp copy), recorded on SUCCESSFUL reservations only (R4-02).
pub static PEAK_TRANSIENT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// True for reparse points (junctions/symlinks). Scans and deletes NEVER
/// traverse these — a store-internal junction pointing outside must not
/// turn into an out-of-root delete (PLAN-783 §5.4 目录边界).
pub fn is_reparse(p: &Path) -> bool {
    std::fs::symlink_metadata(p)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
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
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Storage::FileSystem::{
        LockFileEx, UnlockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
    };
    use windows_sys::Win32::System::IO::OVERLAPPED;

    pub struct FileGuard {
        file: File,
    }

    fn try_lock_once(file: &File, exclusive: bool) -> io::Result<bool> {
        let flags: u32 = LOCKFILE_FAIL_IMMEDIATELY
            | if exclusive {
                LOCKFILE_EXCLUSIVE_LOCK
            } else {
                0
            };
        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        let ok = unsafe {
            LockFileEx(
                file.as_raw_handle() as _,
                flags,
                0,
                u32::MAX,
                u32::MAX,
                &mut overlapped,
            )
        };
        if ok != 0 {
            return Ok(true);
        }
        let err = io::Error::last_os_error();
        // 32 ERROR_SHARING_VIOLATION / 33 ERROR_LOCK_VIOLATION: the
        // byte range is held by someone else — a clean "busy".
        if err.raw_os_error() == Some(32) || err.raw_os_error() == Some(33) {
            return Ok(false);
        }
        Err(err)
    }

    impl FileGuard {
        pub fn try_lock(path: &Path, exclusive: bool) -> io::Result<Option<FileGuard>> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path)?;
            if try_lock_once(&file, exclusive)? {
                Ok(Some(FileGuard { file }))
            } else {
                Ok(None)
            }
        }

        /// Bounded blocking acquire (§5.4 bounded backoff — R1 F-08): wait
        /// up to `deadline` for the holder to release, then give up.
        pub fn lock_timeout(
            path: &Path,
            exclusive: bool,
            deadline: Duration,
        ) -> io::Result<Option<FileGuard>> {
            if let Some(g) = Self::try_lock(path, exclusive)? {
                return Ok(Some(g));
            }
            let end = Instant::now() + deadline;
            loop {
                std::thread::sleep(Duration::from_millis(10));
                match Self::try_lock(path, exclusive)? {
                    Some(g) => return Ok(Some(g)),
                    None => {
                        if Instant::now() >= end {
                            return Ok(None);
                        }
                    }
                }
            }
        }
    }

    impl Drop for FileGuard {
        fn drop(&mut self) {
            let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
            unsafe {
                UnlockFileEx(
                    self.file.as_raw_handle() as _,
                    0,
                    u32::MAX,
                    u32::MAX,
                    &mut overlapped,
                )
            };
        }
    }
}

#[cfg(not(windows))]
mod flock {
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::path::Path;
    use std::time::Duration;

    pub struct FileGuard {
        _file: File,
    }

    impl FileGuard {
        pub fn try_lock(path: &Path, _exclusive: bool) -> io::Result<Option<FileGuard>> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path)?;
            Ok(Some(FileGuard { _file: file }))
        }

        pub fn lock_timeout(
            path: &Path,
            exclusive: bool,
            deadline: Duration,
        ) -> io::Result<Option<FileGuard>> {
            Self::try_lock(path, exclusive)
        }
    }
}

pub use flock::FileGuard;

/// Bounded waits for the coordination locks (probe-scale; production would
/// make these configurable).
pub const LIVENESS_SHARED_WAIT: Duration = Duration::from_secs(10);
pub const LIVENESS_EXCLUSIVE_WAIT: Duration = Duration::from_secs(30);
pub const CHUNK_LOCK_WAIT: Duration = Duration::from_secs(10);

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
/// logs — everything counts (PLAN-783 §5.4). `db` = main file + WAL + SHM
/// (full accounted usage); `db_main` = the main journal file only — the
/// GC's uncompressible floor, because WAL/SHM are transient and shrink
/// back at checkpoint (counting them would spuriously back off during
/// write bursts — review R1 F-07 repair).
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct SpaceReport {
    pub blocks: u64,
    pub blocks_count: u64,
    pub db: u64,
    #[serde(default)]
    pub db_main: u64,
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
    /// (target artifact id, kind) — required edges keep targets alive AND
    /// are liveness-checked at publication time (atomic final decision).
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
        if is_reparse(root) {
            return Err(StoreError::Boundary(format!(
                "store root is a reparse point: {}",
                root.display()
            )));
        }
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
        // Incremental auto-vacuum (R2-05): the main DB file must be able
        // to actually SHRINK after reclamation — the total contract (§5.4)
        // counts DB/WAL bytes, so "SQLite never shrinks" cannot stand as
        // an implementation answer. Must be set before the schema exists.
        db.pragma_update(None, "auto_vacuum", "INCREMENTAL")?;
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
            Err(StoreError::Boundary(format!(
                "path escapes store root: {}",
                canon.display()
            )))
        }
    }

    fn lock_path(&self, name: &str) -> PathBuf {
        self.root.join("locks").join(format!("{}.lock", name))
    }

    pub(crate) fn lock_path_pub(&self, name: &str) -> PathBuf {
        self.lock_path(name)
    }

    pub fn dir_size_pub(p: &Path) -> u64 {
        Self::dir_size(p)
    }

    /// SHARED liveness guard — held by publishers / registrars while
    /// committing metadata that can change what stays alive.
    pub(crate) fn liveness_shared(&self) -> Result<FileGuard, StoreError> {
        FileGuard::lock_timeout(&self.lock_path("liveness"), false, LIVENESS_SHARED_WAIT)?
            .ok_or_else(|| StoreError::Busy("liveness shared lock".into()))
    }

    /// EXCLUSIVE liveness guard — held by GC/recovery for their whole
    /// decide-and-delete phase.
    pub(crate) fn liveness_exclusive(&self) -> Result<FileGuard, StoreError> {
        FileGuard::lock_timeout(&self.lock_path("liveness"), true, LIVENESS_EXCLUSIVE_WAIT)?
            .ok_or_else(|| StoreError::Busy("liveness exclusive lock".into()))
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
        let db_main = self
            .root
            .join("journal.db")
            .metadata()
            .map(|m| m.len())
            .unwrap_or(0);
        let db = ["journal.db-wal", "journal.db-shm"]
            .iter()
            .map(|f| self.root.join(f))
            .filter(|p| p.is_file())
            .map(|p| p.metadata().map(|m| m.len()).unwrap_or(0))
            .sum::<u64>()
            + db_main;
        let staging = Self::dir_size(&self.root.join("staging"));
        let trash = Self::dir_size(&self.root.join("trash"));
        let locks = Self::dir_size(&self.root.join("locks"));
        let logs = Self::dir_size(&self.root.join("logs"));
        SpaceReport {
            blocks,
            blocks_count,
            db,
            db_main,
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
        // Staging identity is unique even across same-millisecond threads
        // in one process (R2-03): millisecond + pid + process-global seq.
        let seq = STAGING_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let txn = format!("t{}-{}-{}", now_ms(), crate::sysinfo::current_pid(), seq);
        let staging_dir = self.root.join("staging").join(&txn);
        let result = self.put_inner(&req, &staging_dir);
        if result.is_err() {
            // A failed publish never leaves staging behind (R1 F-08) —
            // EVERY error path, conflicts included.
            let _ = fs::remove_dir_all(&staging_dir);
        }
        result
    }

    fn put_inner(&self, req: &PutRequest, staging_dir: &Path) -> Result<String, StoreError> {
        let framed = frame_chunk(req.kind, &req.payload);
        let chunk_id = ChunkId::from_parts(&[framed.as_slice()]).0;
        let art_id = Self::artifact_id(req, &chunk_id);

        // ONE critical section (R2-01/03): staging allocation, budget
        // reservation, CAS placement and the metadata commit all run under
        // the SHARED liveness guard, so GC/recovery (exclusive) can never
        // observe or delete a half-published payload.
        let _live = self.liveness_shared()?;
        // ATOMIC TEMP-SPACE BUDGET (R3-01 + R4-02 + R5-01): the §5.4
        // contract caps ALL agreed transient copies at 8 MiB — the staged
        // file AND the CAS `.tmp` copy — INCLUDING residue from earlier
        // failed publishes or crashes (leftover `.tmp` files inside the
        // blocks namespace). The budget lock is an EXCLUSIVE cross-process
        // lock held from measure through the CAS placement, reserving
        // 2× framed bytes (both copies, 48-byte header included) plus the
        // measured leftovers, so a controlled failure earlier can never
        // push a later ordinary write past the cap. PEAK records the
        // observed total on SUCCESSFUL reservations only (R4-02).
        // Lock order: liveness(shared) → budget(exclusive) → chunk →
        // sqlite; GC/recover hold liveness(exclusive) and never take the
        // budget lock — no cycle.
        let _budget =
            FileGuard::lock_timeout(&self.lock_path("staging-budget"), true, CHUNK_LOCK_WAIT)?
                .ok_or_else(|| StoreError::Busy("staging budget lock".into()))?;
        let staging_now = Self::dir_size(&self.root.join("staging"));
        let tmp_residue = self.cas_tmp_bytes();
        let reserved_total = staging_now + tmp_residue + 2 * framed.len() as u64;
        if reserved_total > Self::STAGING_BUDGET {
            return Err(StoreError::ResourceExhausted(format!(
                "transient (staging {} + leftover CAS tmp {} + staged {} + CAS tmp {}) exceeds budget {}",
                staging_now,
                tmp_residue,
                framed.len(),
                framed.len(),
                Self::STAGING_BUDGET
            )));
        }
        PEAK_TRANSIENT.fetch_max(reserved_total, std::sync::atomic::Ordering::Relaxed);

        // 1. Stage (transient sharing violations retried — AV scanners can
        //    hold a freshly created file; R2 repair-round observation). The
        //    budget lock is still held, so concurrent reservations see these
        //    bytes on disk.
        fs::create_dir_all(staging_dir)?;
        let staged_file = staging_dir.join("payload");
        write_with_retry(&staged_file, &framed)?;
        fault("after-staging");

        // 2. Per-chunk cross-process lock, BOUNDED wait (R1 F-08): same-key
        //    producers serialize; the fixed tmp name in the reused
        //    BlobStore is never raced. A busy chunk is a controlled error.
        //    Lock order liveness(shared) → chunk → sqlite; GC takes
        //    liveness(exclusive) → gate → sqlite — no cycle (GC never
        //    takes chunk locks, put never takes gate locks).
        let _chunk_lock = FileGuard::lock_timeout(
            &self.lock_path(&format!("chunk-{}", chunk_id)),
            true,
            CHUNK_LOCK_WAIT,
        )?
        .ok_or_else(|| StoreError::Busy(format!("chunk lock for {} (holder active)", chunk_id)))?;
        // CAS placement with bounded retry on transient sharing violations
        // (os 32 — the reused BlobStore's copy+rename under AV/churn);
        // placement is idempotent (identical content, per-chunk locked).
        let mut placed = false;
        let mut last_err: Option<StoreError> = None;
        for attempt in 0..8 {
            match self.blobs.put(&chunk_id, &staged_file) {
                Ok(_) => {
                    placed = true;
                    break;
                }
                Err(e) => {
                    let transient = matches!(
                        &e,
                        auto_cache::storage::Error::Io(io)
                            if io.raw_os_error() == Some(32) || io.raw_os_error() == Some(5)
                    );
                    last_err = Some(StoreError::from(e));
                    if !transient {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(40 * (attempt + 1)));
                }
            }
        }
        if !placed {
            // Clean up OUR tmp copy under the still-held budget lock
            // (R5-01): the dest may be held, the tmp is ours; anything
            // that still remains is counted by the next reservation.
            self.discard_own_cas_tmp(&chunk_id);
            return Err(
                last_err.unwrap_or_else(|| StoreError::Internal("CAS placement failed".into()))
            );
        }
        fault("after-chunk-move");
        // The transient copies are gone (staging still exists until the
        // final cleanup; the CAS tmp was consumed by the rename) — release
        // the temp-space budget for the next publisher (R4-02: the lock
        // spans staging write THROUGH CAS placement so both copies are
        // inside the quota window).
        drop(_budget);

        // 3. Atomic metadata publish — the SHARED liveness guard taken at
        //    the top of this critical section is still held (R2-01): GC
        //    and recovery cannot decide+delete between placement and
        //    commit, nor observe this payload as an uncommitted orphan.
        let db = self.db.lock().unwrap();
        let tx = db.unchecked_transaction()?;
        // Chunk state upsert (R1 F-03): republication of previously
        // deleted/tombstoned content restores 'live'.
        tx.execute(
            "INSERT INTO chunks(chunk_id, kind, size, state, created_epoch) VALUES (?1, ?2, ?3, 'live', ?4)
             ON CONFLICT(chunk_id) DO UPDATE SET state='live' WHERE state != 'live'",
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
                .query_row(
                    "SELECT state FROM chunks WHERE chunk_id=?1",
                    params![cid],
                    |r| r.get(0),
                )
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
                // Atomic final liveness decision (R1 F-01): every REQUIRED
                // edge target must be live at commit time — a target that a
                // concurrent GC already deleted refuses this publication.
                for (dst, kind) in &req.edges {
                    if *kind == RefKind::RequiredData && !row_chunk_live(&tx, dst)? {
                        return Err(StoreError::TargetMissing(format!(
                            "required edge target {} is not live",
                            dst
                        )));
                    }
                }
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
                    params![staging_dir.file_name().unwrap().to_string_lossy(), format!("{} vs {}", id, art_id), now_secs()],
                )?;
                tx.commit()?;
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
            params![staging_dir.file_name().unwrap().to_string_lossy(), now_secs()],
        )?;
        tx.commit()?;
        drop(db);
        drop(_live);
        drop(_chunk_lock);
        let _ = fs::remove_dir_all(&staging_dir);
        Ok(art_id)
    }

    /// Lookup by (layer, key); a tombstoned chunk reads as a miss (the
    /// caller recomputes) — never a partial read and never an error.
    pub fn get_artifact(
        &self,
        layer: &str,
        key: &QueryKey,
    ) -> Result<Option<LoadedArtifact>, StoreError> {
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
        let Some((art_id, chunk_id, kind_byte)) = row else {
            return Ok(None);
        };
        let kind = ChunkKind::from_code(kind_byte)
            .ok_or_else(|| StoreError::Internal("bad kind".into()))?;
        match self.read_chunk_gated(&chunk_id) {
            Ok(Some(bytes)) => {
                let (k, payload) = unframe_chunk(&bytes)?;
                if k != kind {
                    return Err(StoreError::Internal(format!(
                        "kind mismatch for {}",
                        chunk_id
                    )));
                }
                Ok(Some(LoadedArtifact {
                    artifact_id: art_id,
                    chunk_id,
                    kind,
                    payload: payload.to_vec(),
                }))
            }
            Ok(None) => Ok(None),
            Err(StoreError::TombstonedMiss(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Reader-side interlock: shared gate → tombstone check → read →
    /// CAS ADMISSION (R1 F-04): the framed bytes must hash to the very
    /// address being read; a substituted but self-consistent chunk is a
    /// controlled rejection, never wrong bytes.
    pub fn read_chunk_gated(&self, chunk_id: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let gate = FileGuard::try_lock(&self.lock_path(&format!("gate-{}", chunk_id)), false)?
            .ok_or_else(|| StoreError::Internal("gate lock unavailable".into()))?;
        let state: Option<String> = self
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT state FROM chunks WHERE chunk_id=?1",
                params![chunk_id],
                |r| r.get(0),
            )
            .optional()?;
        match state.as_deref() {
            None => return Ok(None), // unknown chunk = miss
            Some("deleted") => return Ok(None),
            Some("tombstoned") => return Err(StoreError::TombstonedMiss(chunk_id.into())),
            Some("live") => {}
            Some(other) => return Err(StoreError::Internal(format!("bad chunk state {}", other))),
        }
        let Some(path) = self.blobs.get(chunk_id) else {
            return Ok(None);
        };
        let bytes = fs::read(&path)?;
        drop(gate);
        // Admission: recompute the framed-byte content identity.
        let actual = ChunkId::from_parts(&[bytes.as_slice()]).0;
        if actual != chunk_id {
            return Err(StoreError::ChunkAddressMismatch {
                requested: chunk_id.to_string(),
                actual,
            });
        }
        Ok(Some(bytes))
    }

    /// Hard-bounded chunk read (PLAN-817 F-817-R8-01, append-only
    /// dependency adaptation): identical admission to
    /// [`read_chunk_gated`] except the CONTENT read is constrained by
    /// the caller's granted budget: the opened file's REAL length is
    /// checked via handle metadata FIRST (an oversized/replaced file is
    /// a `ResourceExhausted` rejection with ZERO content allocation),
    /// and the actual read runs through `Read::take(max_bytes)` so the
    /// returned Vec can never exceed the bound even if the file grows
    /// or is swapped between the check and the read. Database size
    /// metadata is a hint; the filesystem is the truth at read time.
    pub fn read_chunk_gated_bounded(
        &self,
        chunk_id: &str,
        max_bytes: u64,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let gate = FileGuard::try_lock(&self.lock_path(&format!("gate-{}", chunk_id)), false)?
            .ok_or_else(|| StoreError::Internal("gate lock unavailable".into()))?;
        let state: Option<String> = self
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT state FROM chunks WHERE chunk_id=?1",
                params![chunk_id],
                |r| r.get(0),
            )
            .optional()?;
        match state.as_deref() {
            None => return Ok(None), // unknown chunk = miss
            Some("deleted") => return Ok(None),
            Some("tombstoned") => return Err(StoreError::TombstonedMiss(chunk_id.into())),
            Some("live") => {}
            Some(other) => return Err(StoreError::Internal(format!("bad chunk state {}", other))),
        }
        let Some(path) = self.blobs.get(chunk_id) else {
            return Ok(None);
        };
        drop(gate);
        let mut file = fs::File::open(&path).map_err(|e| StoreError::Io(e.to_string()))?;
        let real_len = file
            .metadata()
            .map_err(|e| StoreError::Io(e.to_string()))?
            .len();
        if real_len > max_bytes {
            return Err(StoreError::ResourceExhausted(format!(
                "chunk {} is {real_len}B on disk, over the {max_bytes}B granted read budget (metadata/file drift — corrupt or foreign-written blob)",
                chunk_id
            )));
        }
        let mut bytes = Vec::new();
        use std::io::Read as _;
        (&mut file)
            .take(max_bytes)
            .read_to_end(&mut bytes)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        // Admission: recompute the framed-byte content identity.
        let actual = ChunkId::from_parts(&[bytes.as_slice()]).0;
        if actual != chunk_id {
            return Err(StoreError::ChunkAddressMismatch {
                requested: chunk_id.to_string(),
                actual,
            });
        }
        Ok(Some(bytes))
    }

    /// Hard-bounded artifact read (PLAN-817 F-817-R8-01): same lookup
    /// and admission as [`get_artifact`], but the chunk content read is
    /// bounded by `max_framed_bytes` (see
    /// [`read_chunk_gated_bounded`]) — a corrupt/replaced blob larger
    /// than the granted budget is rejected BEFORE its bytes are read.
    pub fn get_artifact_bounded(
        &self,
        layer: &str,
        key: &QueryKey,
        max_framed_bytes: u64,
    ) -> Result<Option<LoadedArtifact>, StoreError> {
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
        let Some((art_id, chunk_id, kind_byte)) = row else {
            return Ok(None);
        };
        let kind = ChunkKind::from_code(kind_byte)
            .ok_or_else(|| StoreError::Internal("bad kind".into()))?;
        match self.read_chunk_gated_bounded(&chunk_id, max_framed_bytes) {
            Ok(Some(bytes)) => {
                let (k, payload) = unframe_chunk(&bytes)?;
                if k != kind {
                    return Err(StoreError::Internal(format!(
                        "kind mismatch for {}",
                        chunk_id
                    )));
                }
                Ok(Some(LoadedArtifact {
                    artifact_id: art_id,
                    chunk_id,
                    kind,
                    payload: payload.to_vec(),
                }))
            }
            Ok(None) => Ok(None),
            Err(StoreError::TombstonedMiss(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // -- roots / pins / leases ------------------------------------------------

    pub fn register_root(
        &self,
        project: &str,
        artifact: &str,
        kind: &str,
    ) -> Result<(), StoreError> {
        // Atomic liveness decision (R1 F-01): a root on a dead payload is
        // refused — it would silently pin nothing while claiming protection.
        let _live = self.liveness_shared()?;
        {
            let db = self.db.lock().unwrap();
            if !row_chunk_live(&db, artifact)? {
                return Err(StoreError::TargetMissing(format!(
                    "cannot root {} under {}: payload not live",
                    artifact, project
                )));
            }
            db.execute(
                "INSERT OR IGNORE INTO roots(project, artifact, kind) VALUES (?1, ?2, ?3)",
                params![project, artifact, kind],
            )?;
        }
        Ok(())
    }

    /// Roots of one project+kind (adapter read seam: version arbitration for
    /// current-publication decisions — PLAN-787 R2).
    pub fn root_artifacts(&self, project: &str, kind: &str) -> Result<Vec<String>, StoreError> {
        let _live = self.liveness_shared()?;
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT artifact FROM roots WHERE project=?1 AND kind=?2")?;
        let out = stmt
            .query_map(params![project, kind], |r| r.get(0))?
            .flatten()
            .collect();
        Ok(out)
    }

    /// Artifact payload by artifact id (adapter read seam; gated chunk read —
    /// same admission as key lookups, tombstones read as miss).
    pub fn get_artifact_by_id(
        &self,
        artifact_id: &str,
    ) -> Result<Option<LoadedArtifact>, StoreError> {
        let row: Option<(String, u8)> = self
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT payload_chunk, kind FROM artifacts a JOIN chunks c ON c.chunk_id = a.payload_chunk WHERE a.artifact_id=?1",
                params![artifact_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((chunk_id, kind_byte)) = row else {
            return Ok(None);
        };
        let kind = ChunkKind::from_code(kind_byte)
            .ok_or_else(|| StoreError::Internal("bad kind".into()))?;
        match self.read_chunk_gated(&chunk_id) {
            Ok(Some(bytes)) => {
                let (k, payload) = unframe_chunk(&bytes)?;
                if k != kind {
                    return Err(StoreError::Internal(format!(
                        "kind mismatch for {artifact_id}"
                    )));
                }
                Ok(Some(LoadedArtifact {
                    artifact_id: artifact_id.to_string(),
                    chunk_id,
                    kind,
                    payload: payload.to_vec(),
                }))
            }
            Ok(None) => Ok(None),
            Err(StoreError::TombstonedMiss(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn remove_project_roots(&self, project: &str) -> Result<usize, StoreError> {
        let _live = self.liveness_shared()?;
        let n = self.db.lock().unwrap().execute(
            "DELETE FROM roots WHERE project=?1 AND kind='soft'",
            params![project],
        )?;
        Ok(n)
    }

    /// Remove ONE root row (project, kind, artifact) — adapter seam for
    /// ordered publication (register new first, then drop the superseded row;
    /// PLAN-787 R3-03: a failed registration must never lose the old root).
    pub fn remove_root(
        &self,
        project: &str,
        kind: &str,
        artifact: &str,
    ) -> Result<usize, StoreError> {
        let _live = self.liveness_shared()?;
        let n = self.db.lock().unwrap().execute(
            "DELETE FROM roots WHERE project=?1 AND kind=?2 AND artifact=?3",
            params![project, kind, artifact],
        )?;
        Ok(n)
    }

    /// Remove roots of one kind (test/lifecycle seam: unpinning hard pins).
    pub fn remove_roots(&self, project: &str, kind: &str) -> Result<usize, StoreError> {
        let _live = self.liveness_shared()?;
        let n = self.db.lock().unwrap().execute(
            "DELETE FROM roots WHERE project=?1 AND kind=?2",
            params![project, kind],
        )?;
        Ok(n)
    }

    /// Drop all artifact metadata rows of a layer (test seam for forced
    /// misses: the payloads stay until GC reclaims them as unreachable).
    pub fn purge_layer(&self, layer: &str) -> Result<usize, StoreError> {
        let _live = self.liveness_shared()?;
        let db = self.db.lock().unwrap();
        db.execute(
            "DELETE FROM edges WHERE src IN (SELECT artifact_id FROM artifacts WHERE layer=?1)",
            params![layer],
        )?;
        db.execute("DELETE FROM roots WHERE artifact IN (SELECT artifact_id FROM artifacts WHERE layer=?1)", params![layer])?;
        let m = db.execute("DELETE FROM artifacts WHERE layer=?1", params![layer])?;
        Ok(m)
    }

    pub fn acquire_lease(
        &self,
        purpose: &str,
        ttl_secs: u64,
        artifact: Option<&str>,
    ) -> Result<Lease, StoreError> {
        // Hold the lock file for the lease's lifetime (OS releases it when
        // the holder dies — TTL/PID alone never decide liveness). The id
        // carries a process-global sequence: same-millisecond sequential
        // grants (released rows keep their lease_id in the table) must not
        // collide on the leases PRIMARY KEY (F787-R6-03 — the same
        // uniqueness class STAGING_SEQ already solves for staging txn ids;
        // liveness semantics unchanged: OS lock + TTL/PID).
        let seq = LEASE_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let id = format!("ls-{}-{}-{}", now_ms(), crate::sysinfo::current_pid(), seq);
        let lock_path = self.lock_path(&format!("lease-{}", id));
        let guard = FileGuard::try_lock(&lock_path, true)?
            .ok_or_else(|| StoreError::Internal("lease lock unavailable".into()))?;
        // Interlock (R1 F-01): the grant runs either before a concurrent
        // GC's snapshot or after it finished — and in the latter case the
        // target must still be live, else the grant is refused.
        {
            let _live = self.liveness_shared()?;
            if let Some(a) = artifact {
                let db = self.db.lock().unwrap();
                if !row_chunk_live(&db, a)? {
                    return Err(StoreError::TargetMissing(format!(
                        "cannot lease {}: payload not live",
                        a
                    )));
                }
                db.execute(
                    "INSERT INTO leases(lease_id, purpose, artifact, lock_path, acquired_epoch, heartbeat_epoch, ttl_secs, released) VALUES (?1,?2,?3,?4,?5,?5,?6,0)",
                    params![id, purpose, artifact, lock_path.display().to_string(), now_secs(), ttl_secs],
                )?;
            } else {
                self.db.lock().unwrap().execute(
                    "INSERT INTO leases(lease_id, purpose, artifact, lock_path, acquired_epoch, heartbeat_epoch, ttl_secs, released) VALUES (?1,?2,NULL,?3,?4,?4,?5,0)",
                    params![id, purpose, lock_path.display().to_string(), now_secs(), ttl_secs],
                )?;
            }
        }
        Ok(Lease {
            store_root: self.root.clone(),
            lease_id: id,
            _guard: guard,
        })
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
        let Some((released, lock_path)) = row else {
            return Ok(false);
        };
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

    pub fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.db.lock().unwrap()
    }

    /// Physical removal with a trash-move first: a blocked final remove
    /// leaves a reclaimable state instead of a half-deleted block.
    /// Transient sharing violations (os 32 — AV scanners and racing opens
    /// are a Windows reality under churn) are retried a bounded number of
    /// times before surfacing. Returns Ok(true) when the bytes actually
    /// left the disk.
    pub(crate) fn blob_remove(&self, chunk_id: &str) -> Result<bool, StoreError> {
        let path = self.blobs.get(chunk_id);
        let Some(p) = path else { return Ok(false) };
        self.check_inside(&p)?;
        let trash = self.root.join("trash").join(chunk_id);
        rename_with_retry(&p, &trash)?;
        Ok(remove_with_retry(&trash))
    }

    /// CAS blob path by chunk id (namespace gates already checked by the
    /// caller). Used by GC's orphan reclamation (R3-02).
    pub(crate) fn blocks_blob_path(&self, chunk_id: &str) -> Option<PathBuf> {
        self.blobs.get(chunk_id)
    }

    /// Boundary-checked physical file removal for orphan garbage (no
    /// metadata row): canonical containment + bounded retry (R3-02).
    pub(crate) fn bounded_remove_file(&self, p: &Path) -> Result<bool, StoreError> {
        self.check_inside(p)?;
        Ok(remove_with_retry(p))
    }
}

/// Bounded retry for transient Windows sharing violations (os 32) on
/// rename/remove — AV scanners and racing opens under churn; a persistent
/// violation still surfaces as an error.
fn rename_with_retry(src: &Path, dst: &Path) -> Result<(), StoreError> {
    let mut last = None;
    for attempt in 0..5 {
        match fs::rename(src, dst) {
            Ok(()) => return Ok(()),
            Err(e) if e.raw_os_error() == Some(32) || e.raw_os_error() == Some(5) => {
                last = Some(e);
                std::thread::sleep(Duration::from_millis(20 * (attempt + 1)));
            }
            Err(e) => return Err(StoreError::Io(e.to_string())),
        }
    }
    Err(StoreError::Io(
        last.map(|e| e.to_string()).unwrap_or_default(),
    ))
}

fn remove_with_retry(p: &Path) -> bool {
    for attempt in 0..8 {
        if fs::remove_file(p).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20 * (attempt + 1)));
    }
    false
}

/// Staged-write with bounded retry on transient sharing violations.
fn write_with_retry(p: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let mut last = None;
    for attempt in 0..8 {
        match fs::write(p, bytes) {
            Ok(()) => return Ok(()),
            Err(e) if e.raw_os_error() == Some(32) || e.raw_os_error() == Some(5) => {
                last = Some(e);
                std::thread::sleep(Duration::from_millis(40 * (attempt + 1)));
            }
            Err(e) => return Err(StoreError::Io(e.to_string())),
        }
    }
    Err(StoreError::Io(
        last.map(|e| e.to_string()).unwrap_or_default(),
    ))
}

/// `true` iff the artifact row exists and its payload chunk is live.
fn row_chunk_live(db: &Connection, artifact: &str) -> Result<bool, StoreError> {
    let state: Option<String> = db
        .query_row(
            "SELECT c.state FROM artifacts a JOIN chunks c ON c.chunk_id = a.payload_chunk WHERE a.artifact_id=?1",
            params![artifact],
            |r| r.get(0),
        )
        .optional()?;
    Ok(matches!(state.as_deref(), Some("live")))
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
            let _ = store.db().execute(
                "UPDATE leases SET released=1 WHERE lease_id=?1",
                params![self.lease_id],
            );
        }
    }
}

impl ChunkKind {
    /// DB metadata kind → enum. Delegates to the codec's SINGLE kind table
    /// (F787-R7-01): this locally duplicated match drifted and dropped
    /// SourcePage=10, so `get_artifact`/`get_artifact_by_id` failed with
    /// `bad kind` on valid artifacts before payload admission even ran.
    /// Unknown bytes still map to None — the rejection boundary is unchanged.
    pub(crate) fn from_code(v: u8) -> Option<ChunkKind> {
        ChunkKind::from_u8(v).ok()
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

pub(crate) fn fault_pub(name: &str) {
    fault(name)
}

/// Metadata cascade when a chunk's payload leaves the store: edges, roots
/// and artifact rows referencing it converge with physical state (R1
/// F-03/F-06) so same-key republication can never hit a zombie row.
pub(crate) fn cascade_chunk_rows(db: &Connection, cid: &str) -> Result<(), StoreError> {
    db.execute(
        "DELETE FROM edges WHERE src IN (SELECT artifact_id FROM artifacts WHERE payload_chunk=?1)",
        params![cid],
    )?;
    db.execute(
        "DELETE FROM roots WHERE artifact IN (SELECT artifact_id FROM artifacts WHERE payload_chunk=?1)",
        params![cid],
    )?;
    db.execute("DELETE FROM artifacts WHERE payload_chunk=?1", params![cid])?;
    db.execute(
        "UPDATE chunks SET state='deleted' WHERE chunk_id=?1",
        params![cid],
    )?;
    Ok(())
}

/// Actually return DB/WAL space to the OS (R2-05): truncate the WAL and
/// run incremental vacuum on the auto_vacuum=INCREMENTAL main file, so
/// the §5.4 TOTAL contract is measurable after reclamation.
pub(crate) fn shrink_db(db: &Connection) {
    let _ = db.execute_batch(
        "PRAGMA wal_checkpoint(TRUNCATE); PRAGMA incremental_vacuum; PRAGMA optimize;",
    );
}

/// Namespace-root boundary gate shared by GC and recovery (R2-04/R3-03):
/// staging/trash/locks/logs (and the blocks CAS tree) must not themselves
/// be reparse points. Fail closed with a Boundary error; external content
/// on the far side of any junction is never touched.
pub(crate) fn check_namespace_roots(store: &ManagedStore) -> Result<(), StoreError> {
    for ns in ["staging", "trash", "locks", "logs"] {
        let dir = store.root.join(ns);
        if dir.is_dir() && is_reparse(&dir) {
            return Err(StoreError::Boundary(format!(
                "managed namespace `{}` is a reparse point — refusing to operate through it",
                ns
            )));
        }
    }
    let blocks = store.root.join("blocks").join("blobs");
    if blocks.is_dir() && (is_reparse(&blocks) || is_reparse(blocks.parent().unwrap_or(&blocks))) {
        return Err(StoreError::Boundary(
            "blocks namespace is a reparse point — refusing to operate through it".into(),
        ));
    }
    Ok(())
}

/// Physical total measurement settled for NTFS visibility lag (R3-02):
/// under the exclusive liveness guard there are no concurrent writers, so
/// any growth between two walks is directory-entry propagation catching
/// up. Walk up to three times (short settle sleeps) and keep the LAST
/// (largest-visibility) observation — the measured total always includes
/// crash orphans, CAS tmp files and trash, never just "rows with live
/// metadata".
pub(crate) fn settled_space(store: &ManagedStore) -> crate::store::SpaceReport {
    let mut last = store.space();
    for _ in 0..2 {
        std::thread::sleep(Duration::from_millis(120));
        let next = store.space();
        if next.total <= last.total {
            break;
        }
        last = next;
    }
    last
}

// ---------------------------------------------------------------------------
// Recovery — full state reconciliation (R1 F-06)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct RecoveryReceipt {
    pub staging_dirs_removed: usize,
    pub orphan_chunks_removed: usize,
    /// Committed chunk rows no artifact references (e.g. a conflict's
    /// shared chunk whose artifact was refused).
    pub orphan_rows_removed: usize,
    /// Chunk rows claiming 'live' whose blob is gone (stale metadata).
    pub stale_rows_removed: usize,
    pub tombstone_deletes_completed: usize,
    pub trash_files_removed: usize,
    pub aborted_txn_rows: usize,
    pub freed_bytes: u64,
}

/// Reconcile a store after crashes. `assumed_exclusive` skips the staging
/// grace period (operator- or test-invoked right after a crash, with no
/// concurrent producers); the default mode only reclaims staging older
/// than the grace window. Runs under the EXCLUSIVE liveness guard, so it
/// cannot interleave with publishers, registrars, or GC.
pub fn recover(
    store: &ManagedStore,
    assumed_exclusive: bool,
) -> Result<RecoveryReceipt, StoreError> {
    let _exclusive = store.liveness_exclusive()?;
    let mut receipt = RecoveryReceipt::default();

    // 0. Namespace-root gates (R2-04/R3-03): shared with GC — every managed
    // namespace directory we are about to enumerate/delete inside must
    // itself not be a reparse point. A junction at a namespace root is a
    // hard boundary violation — refuse the whole pass (fail closed,
    // external content preserved) rather than enumerating through it.
    check_namespace_roots(store)?;

    // 1. Staging leftovers.
    for entry in fs::read_dir(store.root.join("staging"))?.flatten() {
        let p = entry.path();
        if is_reparse(&p) {
            continue; // never follow a reparse jump out of the store
        }
        if p.is_dir() {
            let age = now_secs().saturating_sub(
                entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            );
            if assumed_exclusive || age > 600 {
                let sz = ManagedStore::dir_size(&p);
                fs::remove_dir_all(&p)?;
                receipt.staging_dirs_removed += 1;
                receipt.freed_bytes += sz;
            }
        }
    }

    // 2. Tombstoned chunks: complete the interrupted delete with the SAME
    //    cascade as live GC (rows converge with physical state).
    let tombstoned: Vec<String> = store
        .db()
        .prepare("SELECT chunk_id FROM chunks WHERE state='tombstoned'")?
        .query_map([], |r| r.get(0))?
        .flatten()
        .collect();
    for cid in tombstoned {
        if gc::delete_chunk_full(store, &cid)? {
            receipt.tombstone_deletes_completed += 1;
        }
    }

    // 3. Committed orphan chunk ROWS: no artifact references them (e.g.
    //    a conflict's committed loser chunk) — the blob is unreachable
    //    junk that plain GC (artifact-driven) can never see.
    let referenced: std::collections::HashSet<String> = store
        .db()
        .prepare("SELECT DISTINCT payload_chunk FROM artifacts")?
        .query_map([], |r| r.get(0))?
        .flatten()
        .collect();
    let live_rows: Vec<String> = store
        .db()
        .prepare("SELECT chunk_id FROM chunks WHERE state='live'")?
        .query_map([], |r| r.get(0))?
        .flatten()
        .collect();
    for cid in live_rows {
        if !referenced.contains(&cid) {
            let sz: u64 = store
                .db()
                .query_row(
                    "SELECT size FROM chunks WHERE chunk_id=?1",
                    params![cid],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if gc::delete_chunk_full(store, &cid)? {
                receipt.orphan_rows_removed += 1;
                receipt.freed_bytes += sz;
            }
        }
    }

    // 4. Stale rows: chunk claims 'live' but the blob is gone (external
    //    deletion / lost file). The metadata converges to 'deleted' with
    //    the same row cascade as any other delete.
    for cid in live_rows_retained(store)? {
        if store.blobs.get(&cid).is_none() {
            if gc::delete_chunk_full(store, &cid)? {
                receipt.stale_rows_removed += 1;
            }
        }
    }

    // 5. Orphan blob FILES + interrupted CAS tmp writes (namespace-root
    //    gate included — R2-04). Keep-set is the FULL state matrix
    //    (R4-01): a blob survives only while some ARTIFACT row references
    //    it — not merely "any chunk row exists". This reclaims: no row
    //    (F2 crash), deleted/tombstoned row with the file re-appeared
    //    (F2 crash after same-content republication), and live row with
    //    no artifact (conflict loser).
    let committed: std::collections::HashSet<String> = referenced;
    let blocks_dir = store.root.join("blocks").join("blobs");
    if blocks_dir.is_dir() {
        if is_reparse(&blocks_dir) || is_reparse(blocks_dir.parent().unwrap_or(&blocks_dir)) {
            return Err(StoreError::Boundary(
                "blocks namespace is a reparse point — refusing to operate through it".into(),
            ));
        }
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

    // 6. Trash residue (R1 F-06): pending deletes from blocked removes are
    //    retried here; whatever still cannot be removed stays and is
    //    reported by `space().trash`, never silently counted as freed.
    let trash_dir = store.root.join("trash");
    if trash_dir.is_dir() {
        for entry in fs::read_dir(&trash_dir)?.flatten() {
            if is_reparse(&entry.path()) {
                continue;
            }
            if entry.path().is_file() {
                let sz = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if fs::remove_file(entry.path()).is_ok() {
                    receipt.trash_files_removed += 1;
                    receipt.freed_bytes += sz;
                }
            }
        }
    }

    // 7. Journal hygiene: mark lingering staging-phase txns aborted, and
    //    actually return DB/WAL space (R2-05) so freed bytes leave TOTAL.
    receipt.aborted_txn_rows = store.db().execute(
        "UPDATE txns SET phase='aborted' WHERE phase IN ('staging','conflict')",
        [],
    )?;
    shrink_db(&store.db());
    Ok(receipt)
}

fn live_rows_retained(store: &ManagedStore) -> Result<Vec<String>, StoreError> {
    // Rows still live after the orphan-row pass above.
    let v: Vec<String> = store
        .db()
        .prepare("SELECT chunk_id FROM chunks WHERE state='live'")?
        .query_map([], |r| r.get(0))?
        .flatten()
        .collect();
    Ok(v)
}

impl ManagedStore {
    /// Sum of leftover CAS `.tmp` files inside the blocks namespace —
    /// residue from failed placements (bounded retries exhausted) or
    /// crashes mid-window. Counted into every temp-space reservation
    /// (R5-01): a controlled failure earlier must consume quota, not
    /// silently push the next write past the cap.
    fn cas_tmp_bytes(&self) -> u64 {
        let blocks = self.root.join("blocks").join("blobs");
        let mut total = 0u64;
        let Ok(shards) = fs::read_dir(&blocks) else {
            return 0;
        };
        for shard in shards.flatten() {
            let p = shard.path();
            if !p.is_dir() || is_reparse(&p) {
                continue;
            }
            let Ok(files) = fs::read_dir(&p) else {
                continue;
            };
            for f in files.flatten() {
                if is_reparse(&f.path()) {
                    continue;
                }
                if f.file_name().to_string_lossy().ends_with(".tmp") {
                    total += f.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
        total
    }

    /// Best-effort removal of THIS publish's CAS `.tmp` copy after a
    /// failed placement (R5-01): the dest may be held (os5), but the tmp
    /// file itself is ours and unheld. Runs under the still-held budget
    /// lock; a blocked removal simply leaves residue counted by the next
    /// reservation.
    fn discard_own_cas_tmp(&self, chunk_id: &str) {
        if let Some(dest) = self.blobs.get(chunk_id) {
            let tmp = dest.with_extension("tmp");
            let _ = remove_with_retry(&tmp);
        }
    }
}
