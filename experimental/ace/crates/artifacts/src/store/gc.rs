//! Watermark garbage collection (PLAN-783 §5.4 / Design 40 §7).
//!
//! Runs under the EXCLUSIVE liveness guard for the whole decide-and-delete
//! phase (review R1 F-01): publishers, lease grants and root registrations
//! hold it shared, so a GC snapshot can never race an acquisition —
//! whatever the snapshot decided is exactly what the deletion loop does,
//! and anything registered after the snapshot sees post-GC state with a
//! fresh liveness check.
//!
//! Reachability: live roots (project soft roots + hard pins) and live
//! leases seed an artifact set; the `required`-edge closure defines which
//! CHUNKS stay. Everything else — unreachable chunks, orphan rows, stale
//! staging, trash residue — is reclaimable in (oldest-first) batches down
//! to the low watermark. If the LIVE set alone already exceeds the low
//! watermark, GC reports a bounded backoff (stop-write status) and deletes
//! no live data. Deletion is tombstone → gate lock → trash → remove with
//! the SAME row cascade as recovery; a blocked gate leaves a
//! pending-reader entry, never a partial read.

use super::{now_secs, FileGuard, ManagedStore, StoreError};
use rusqlite::{params, OptionalExtension};
use std::collections::{HashMap, HashSet};

// On Windows, an exclusive share-mode open proves that no reader,
// publisher or lease holder still has the lock file open. Delete-on-close
// keeps later opens blocked until the old name has been retired; unlike
// unlink-after-unlock this cannot split a lock between two file objects.
#[cfg(windows)]
fn retire_quiescent_lock(path: &std::path::Path) -> Result<bool, StoreError> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        DELETE, FILE_FLAG_DELETE_ON_CLOSE, FILE_FLAG_OPEN_REPARSE_POINT,
    };
    if super::is_reparse(path) {
        return Ok(false);
    }
    match std::fs::OpenOptions::new()
        .write(true)
        .access_mode(DELETE)
        .share_mode(0)
        .custom_flags(FILE_FLAG_DELETE_ON_CLOSE | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
    {
        Ok(file) => {
            drop(file);
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) if matches!(error.raw_os_error(), Some(5 | 32 | 33)) => Ok(false),
        Err(error) => Err(StoreError::Io(error.to_string())),
    }
}

// Other platforms retain old lock files until their own safe quiescence
// primitive is supplied; terminal metadata can still be compacted.
#[cfg(not(windows))]
fn retire_quiescent_lock(_path: &std::path::Path) -> Result<bool, StoreError> {
    Ok(true)
}

/// Frozen experiment watermarks (§5.4): high 64 MiB, low 48 MiB, extra
/// temp/staging cap 8 MiB.
pub const GC_HIGH: u64 = 64 * 1024 * 1024;
pub const GC_LOW: u64 = 48 * 1024 * 1024;
pub const TEMP_CAP: u64 = 8 * 1024 * 1024;

pub fn gc_config_default() -> GcConfig {
    GcConfig {
        high: GC_HIGH,
        low: GC_LOW,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GcConfig {
    pub high: u64,
    pub low: u64,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct GcReceipt {
    pub ran: bool,
    pub reason: String,
    pub scanned_chunks: usize,
    pub live_chunks: usize,
    pub deleted_chunks: usize,
    pub failed_deletes: usize,
    pub invalid_bytes: u64,
    pub freed_bytes: u64,
    pub pending_reader_bytes: u64,
    pub orphans_removed: usize,
    pub staging_removed: usize,
    pub stale_leases_swept: usize,
    pub total_before: u64,
    pub total_after: u64,
    pub live_bytes: u64,
    /// Set when the live set alone cannot reach the low watermark:
    /// bounded backoff — new cache writes should stop; no live data is
    /// ever deleted to satisfy the watermark.
    pub backoff: Option<String>,
    pub duration_ms: u64,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Artifact ids seeding the live set: soft roots + hard pins + live leases.
fn live_seed_artifacts(store: &ManagedStore) -> Result<HashSet<String>, StoreError> {
    let mut seeds: HashSet<String> = HashSet::new();
    {
        let db = store.db();
        let mut stmt = db.prepare("SELECT artifact FROM roots")?;
        for a in stmt.query_map([], |r| r.get::<_, String>(0))?.flatten() {
            seeds.insert(a);
        }
    }
    let leases: Vec<(String, u8, Option<String>)> = store
        .db()
        .prepare("SELECT lease_id, released, artifact FROM leases")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .flatten()
        .collect();
    for (lid, released, artifact) in leases {
        if released == 0 && store.lease_live(&lid)? {
            if let Some(a) = artifact {
                seeds.insert(a);
            }
        }
    }
    Ok(seeds)
}

/// Transitive closure over `required` edges from the seed artifacts.
fn required_closure(
    store: &ManagedStore,
    seeds: &HashSet<String>,
) -> Result<HashSet<String>, StoreError> {
    let mut live: HashSet<String> = seeds.clone();
    let mut frontier: Vec<String> = seeds.iter().cloned().collect();
    while let Some(src) = frontier.pop() {
        let dsts: Vec<String> = store
            .db()
            .prepare("SELECT dst FROM edges WHERE src=?1 AND kind='required'")?
            .query_map(params![src], |r| r.get(0))?
            .flatten()
            .collect();
        for d in dsts {
            if live.insert(d.clone()) {
                frontier.push(d);
            }
        }
    }
    Ok(live)
}

/// One full chunk deletion under the gate: physical removal (trash-move
/// first) + the metadata cascade (R1 F-03/F-06). Returns Ok(false) when a
/// reader holds the shared gate (pending) or the blob was already gone.
pub(crate) fn delete_chunk_full(store: &ManagedStore, cid: &str) -> Result<bool, StoreError> {
    let gate = FileGuard::try_lock(&store.lock_path_pub(&format!("gate-{}", cid)), true)?;
    let Some(_g) = gate else { return Ok(false) };
    let removed = store.blob_remove(cid)?;
    store.db().execute(
        "UPDATE chunks SET state='tombstoned' WHERE chunk_id=?1 AND state='live'",
        params![cid],
    )?;
    super::cascade_chunk_rows(&store.db(), cid)?;
    Ok(removed)
}

pub fn run_gc(store: &ManagedStore, cfg: GcConfig) -> Result<GcReceipt, StoreError> {
    let t0 = now_ms();
    let mut receipt = GcReceipt::default();
    receipt.stale_leases_swept = store.sweep_stale_leases()?;
    // Namespace-root gates (R3-03): the SAME boundary rule recover enforces
    // — a reparse point at a managed namespace root is a hard refusal.
    // GC deletes inside staging/trash/blocks; it must not enumerate
    // through a junction any more than recovery may.
    super::check_namespace_roots(store)?;
    let space = store.space();
    receipt.total_before = space.total;
    if space.total <= cfg.high {
        receipt.reason = format!("below high watermark ({} <= {})", space.total, cfg.high);
        receipt.total_after = space.total;
        receipt.duration_ms = now_ms().saturating_sub(t0);
        return Ok(receipt);
    }
    receipt.ran = true;
    receipt.reason = "over high watermark".into();

    // EXCLUSIVE liveness for the whole decide+delete phase (R1 F-01):
    // acquisitions/publishes either preceded the snapshot (counted) or
    // wait and re-check liveness afterwards.
    let _exclusive = store.liveness_exclusive()?;

    // Live set and its chunks.
    let seeds = live_seed_artifacts(store)?;
    let live_artifacts = required_closure(store, &seeds)?;
    let all_artifacts: Vec<(String, String, u64)> = store
        .db()
        .prepare("SELECT artifact_id, payload_chunk, created_epoch FROM artifacts")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, u64>(2)?)))?
        .flatten()
        .collect();
    let chunk_sizes: HashMap<String, (String, u64)> = store
        .db()
        .prepare("SELECT chunk_id, state, size FROM chunks")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                (r.get::<_, String>(1)?, r.get::<_, u64>(2)?),
            ))
        })?
        .flatten()
        .collect();
    receipt.scanned_chunks = chunk_sizes.len();
    let mut live_chunk_ids: HashSet<String> = HashSet::new();
    for (aid, cid, _) in &all_artifacts {
        if live_artifacts.contains(aid) {
            live_chunk_ids.insert(cid.clone());
        }
    }
    receipt.live_chunks = live_chunk_ids.len();

    // Uncompressible floor = main DB file + locks + logs + live chunks.
    // The WAL/SHM are transient (checkpoint shrinks them) and stay in the
    // ACCOUNTED total, but not in the floor — otherwise write bursts
    // spuriously trigger backoff (R1 F-07 repair).
    let mut live_bytes = space.db_main + space.locks + space.logs;
    for cid in &live_chunk_ids {
        live_bytes += chunk_sizes.get(cid).map(|(_, s)| *s).unwrap_or(0);
    }
    receipt.live_bytes = live_bytes;

    // Batch 1: reclaimable junk (never live): stale staging (grace passed).
    let staging_old: Vec<std::path::PathBuf> = {
        let mut v = Vec::new();
        if let Ok(rd) = std::fs::read_dir(store.root.join("staging")) {
            for e in rd.flatten() {
                if super::is_reparse(&e.path()) {
                    continue;
                }
                if e.path().is_dir() {
                    if let Ok(meta) = e.metadata() {
                        if let Ok(modt) = meta.modified() {
                            let age = now_secs().saturating_sub(
                                modt.duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_secs())
                                    .unwrap_or(0),
                            );
                            if age > 600 {
                                v.push(e.path());
                            }
                        }
                    }
                }
            }
        }
        v
    };
    for p in staging_old {
        let sz = ManagedStore::dir_size_pub(&p);
        if std::fs::remove_dir_all(&p).is_ok() {
            receipt.staging_removed += 1;
            receipt.freed_bytes += sz;
        }
    }

    // Backoff check: if the LIVE slab alone already exceeds the low
    // watermark, report and stop — no live deletion to force convergence.
    // `low = 0` disables the floor semantics (tests: reclaim every dead
    // candidate; production configs always run with a real watermark).
    if cfg.low > 0 && live_bytes + receipt.freed_bytes >= cfg.low {
        receipt.backoff = Some(format!(
            "live set {} (+ reclaimable {}) cannot reach low watermark {}; stop-write advised, live data retained",
            live_bytes, receipt.freed_bytes, cfg.low
        ));
        receipt.total_after = store.space().total;
        receipt.duration_ms = now_ms().saturating_sub(t0);
        return Ok(receipt);
    }

    // Batch 2: unreachable chunks, oldest first. Candidates ALSO include
    // unaccounted physical blocks (R3-02): under the exclusive liveness
    // guard there are no in-flight publishers, so any blob file without a
    // live chunk row — a crashed F2 publish, a conflict's committed loser,
    // a leftover CAS `.tmp` — is garbage this pass may reclaim. The
    // convergence decision uses the SETTLED PHYSICAL total (settled_space,
    // R3-02), which always counts orphans/tmp/trash — never just "rows
    // with live metadata".
    let mut candidates: Vec<(u64, String, u64)> = Vec::new(); // (created, chunk, size)
    for (aid, cid, created) in &all_artifacts {
        if !live_artifacts.contains(aid) && !live_chunk_ids.contains(cid) {
            if let Some((state, size)) = chunk_sizes.get(cid) {
                if state == "live" {
                    candidates.push((*created, cid.clone(), *size));
                }
            }
        }
    }
    candidates.sort();
    // Dedup chunk ids shared by several dead artifacts.
    let mut seen = HashSet::new();
    candidates.retain(|(_, c, _)| seen.insert(c.clone()));

    // Orphan physical blocks — the FULL state matrix (R4-01): a blob file
    // is garbage unless it is the live payload of a LIVE artifact. That
    // covers: no row at all (F2 crash), a `deleted`/`tombstoned` row with
    // the file re-appeared (F2 crash after same-content republication),
    // and a `live` row with NO artifact (a conflict's committed loser).
    // Dead-artifact chunks are already in `candidates`; everything else
    // on disk without a live-artifact reference is reclaimed here so the
    // watermark decision is made on the REAL total (R3-02/R4-01).
    let candidate_ids: HashSet<String> = candidates.iter().map(|(_, c, _)| c.clone()).collect();
    let mut orphan_candidates: Vec<(u64, String, u64)> = Vec::new();
    {
        let blocks_dir = store.root.join("blocks").join("blobs");
        if blocks_dir.is_dir() {
            for shard in std::fs::read_dir(&blocks_dir)?.flatten() {
                let shard_path = shard.path();
                if !shard_path.is_dir() || super::is_reparse(&shard_path) {
                    continue;
                }
                for blob in std::fs::read_dir(&shard_path)?.flatten() {
                    if super::is_reparse(&blob.path()) {
                        continue;
                    }
                    let name = blob.file_name().to_string_lossy().to_string();
                    let sz = blob.metadata().map(|m| m.len()).unwrap_or(0);
                    let garbage = name.ends_with(".tmp")
                        || (!live_chunk_ids.contains(&name) && !candidate_ids.contains(&name));
                    if garbage {
                        orphan_candidates.push((0, name, sz));
                    }
                }
            }
        }
    }

    let mut idx = 0usize;
    let mut orphan_idx = 0usize;
    loop {
        if cfg.low > 0 {
            super::shrink_db(&store.db());
            let measured = super::settled_space(store).total;
            if measured <= cfg.low
                || (idx >= candidates.len() && orphan_idx >= orphan_candidates.len())
            {
                break;
            }
        } else if idx >= candidates.len() && orphan_idx >= orphan_candidates.len() {
            break;
        }
        // Delete a bounded batch (dead candidates first, then orphans),
        // then re-measure (avoids per-candidate full walks while keeping
        // the decision measured on real bytes, not estimates).
        let mut batch = 32usize;
        while batch > 0 && idx < candidates.len() {
            let (_, cid, size) = &candidates[idx];
            idx += 1;
            batch -= 1;
            // Invalidate logically first (tombstone), then take the gate.
            store.db().execute(
                "UPDATE chunks SET state='tombstoned' WHERE chunk_id=?1 AND state='live'",
                params![cid],
            )?;
            super::fault_pub("after-tombstone");
            let gate = FileGuard::try_lock(&store.lock_path_pub(&format!("gate-{}", cid)), true)?;
            match gate {
                Some(_g) => {
                    receipt.invalid_bytes += size;
                    if store.blob_remove(cid)? {
                        receipt.freed_bytes += size;
                        receipt.deleted_chunks += 1;
                    } else {
                        // Blocked remove (reader/AV holds a handle): the
                        // trash copy remains and is reported as pending.
                        receipt.pending_reader_bytes += size;
                        receipt.failed_deletes += 1;
                    }
                    // Metadata converges with physical state (R1 F-03/06).
                    super::cascade_chunk_rows(&store.db(), cid)?;
                }
                None => {
                    // A reader is active under the shared gate: revert to
                    // live and record pending — the reader finishes intact.
                    receipt.pending_reader_bytes += size;
                    store.db().execute(
                        "UPDATE chunks SET state='live' WHERE chunk_id=?1",
                        params![cid],
                    )?;
                }
            }
        }
        while batch > 0 && orphan_idx < orphan_candidates.len() {
            let (_, name, size) = &orphan_candidates[orphan_idx];
            orphan_idx += 1;
            batch -= 1;
            // Physical garbage (no live-artifact reference). If a metadata
            // row exists (deleted/tombstoned/conflict-loser), converge it
            // with the same cascade so republication never hits a zombie.
            if let Some(p) = store.blocks_blob_path(name) {
                match store.bounded_remove_file(&p) {
                    Ok(true) => {
                        receipt.freed_bytes += size;
                        receipt.orphans_removed += 1;
                        let db = store.db();
                        let has_row: Option<String> = db
                            .query_row(
                                "SELECT state FROM chunks WHERE chunk_id=?1",
                                params![name],
                                |r| r.get(0),
                            )
                            .optional()
                            .unwrap_or(None);
                        if has_row.is_some() {
                            let _ = super::cascade_chunk_rows(&db, name);
                        }
                    }
                    Ok(false) => {
                        receipt.pending_reader_bytes += size;
                        receipt.failed_deletes += 1;
                    }
                    Err(_) => {
                        receipt.failed_deletes += 1;
                    }
                }
            }
        }
    }
    // No publishers are in flight under exclusive liveness. Retire only
    // terminal chunk/lease lock files whose names and OS handles are safe;
    // keep busy rows so a later GC can retry without losing recovery state.
    let deleted: Vec<String> = {
        let db = store.db();
        let mut stmt = db.prepare("SELECT chunk_id FROM chunks WHERE state='deleted' AND NOT EXISTS (SELECT 1 FROM artifacts WHERE payload_chunk=chunks.chunk_id)")?;
        let rows = stmt
            .query_map([], |r| r.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    for id in deleted {
        if id.len() != 67
            || !id.starts_with("chk")
            || !id[3..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            continue;
        }
        let chunk = store.lock_path_pub(&format!("chunk-{id}"));
        let gate = store.lock_path_pub(&format!("gate-{id}"));
        if retire_quiescent_lock(&chunk)? && retire_quiescent_lock(&gate)? {
            store.db().execute(
                "DELETE FROM chunks WHERE chunk_id=?1 AND state='deleted'",
                params![id],
            )?;
        }
    }
    let released: Vec<String> = {
        let db = store.db();
        let mut stmt = db.prepare("SELECT lease_id FROM leases WHERE released=1")?;
        let rows = stmt
            .query_map([], |r| r.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    for id in released {
        if !id.starts_with("ls-") || !id[3..].bytes().all(|b| b.is_ascii_digit() || b == b'-') {
            continue;
        }
        let lock = store.lock_path_pub(&format!("lease-{id}"));
        if retire_quiescent_lock(&lock)? {
            store.db().execute(
                "DELETE FROM leases WHERE lease_id=?1 AND released=1",
                params![id],
            )?;
        }
    }
    store
        .db()
        .execute("DELETE FROM txns WHERE phase='committed'", [])?;
    // Final shrink so the receipt's total_after reflects returned space.
    super::shrink_db(&store.db());

    // Post-condition (R3-02): the SETTLED PHYSICAL total decides whether we
    // genuinely reached low — orphans/tmp/trash included. Anything still
    // above low after exhausting reclaimable candidates is reported as an
    // explicit bounded health state, never a silent pass.
    if cfg.low > 0 {
        let measured = super::settled_space(store).total;
        if measured > cfg.low && receipt.backoff.is_none() {
            receipt.backoff = Some(format!(
                "physical total {} still above low watermark {} after exhausting reclaimable candidates",
                measured, cfg.low
            ));
        }
    }

    receipt.total_after = super::settled_space(store).total;
    receipt.duration_ms = now_ms().saturating_sub(t0);
    Ok(receipt)
}

/// Count of live artifacts (test helper).
pub fn artifact_count(store: &ManagedStore) -> Result<usize, StoreError> {
    let db = store.db();
    let n: Option<i64> = db
        .query_row("SELECT COUNT(*) FROM artifacts", [], |r| r.get(0))
        .optional()?;
    Ok(n.unwrap_or(0) as usize)
}
