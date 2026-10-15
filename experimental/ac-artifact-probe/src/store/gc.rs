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

    // Batch 2: unreachable chunks, oldest first. Convergence against the
    // §5.4 TOTAL contract (R2-05) is decided by MEASURED totals: after
    // each deletion batch the DB is actually shrunk (WAL truncate +
    // incremental vacuum) and the store re-measured — the GC's own
    // metadata churn therefore counts, and the loop continues until the
    // measured total is under the low watermark or candidates run out.
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

    let mut idx = 0usize;
    // Convergence decisions use TRANSACTIONAL accounting (R2-05 repair):
    // the DB's live-chunk size sum drops the instant a delete commits,
    // while an NTFS directory walk immediately after mass renames can
    // lag (measured undercount during the 512-churn debug). The receipt
    // still reports the real walked totals.
    let live_sum_db = |db: &rusqlite::Connection| -> u64 {
        db.query_row(
            "SELECT COALESCE(SUM(size), 0) FROM chunks WHERE state='live'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|v| v.max(0) as u64)
        .unwrap_or(0)
    };
    loop {
        if cfg.low > 0 {
            super::shrink_db(&store.db());
            let s = store.space();
            let measured = s.total - s.blocks + live_sum_db(&store.db());
            if measured <= cfg.low || idx >= candidates.len() {
                break;
            }
        } else if idx >= candidates.len() {
            break;
        }
        // Delete a bounded batch, then re-measure (avoids per-candidate
        // full walks while keeping the decision measured, not estimated).
        let batch_end = (idx + 32).min(candidates.len());
        while idx < batch_end {
            let (_, cid, size) = &candidates[idx];
            idx += 1;
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
    }
    // Final shrink so the receipt's total_after reflects returned space.
    super::shrink_db(&store.db());

    // Post-condition (R2-05): the transactional measure decides whether we
    // genuinely reached low; the receipt's total_after reports the REAL
    // walked total (file metadata may settle slightly later on NTFS).
    if cfg.low > 0 {
        let s = store.space();
        let measured = s.total - s.blocks + live_sum_db(&store.db());
        if measured > cfg.low && receipt.backoff.is_none() {
            receipt.backoff = Some(format!(
                "accounted total {} still above low watermark {} after exhausting reclaimable candidates",
                measured, cfg.low
            ));
        }
    }

    receipt.total_after = store.space().total;
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
