//! Watermark garbage collection (PLAN-783 §5.4 / Design 40 §7).
//!
//! Reachability: live roots (project soft roots + hard pins) and live
//! leases seed an artifact set; the `required`-edge closure defines which
//! CHUNKS stay. Everything else — unreachable chunks, orphan blobs, stale
//! staging, trash residue — is reclaimable in (oldest-first) batches down
//! to the low watermark. If the LIVE set alone already exceeds the low
//! watermark, GC reports a bounded backoff (stop-write status) and deletes
//! no live data. Deletion is tombstone → gate lock → trash → remove; a
//! blocked gate leaves a pending-reader entry, never a partial read.

use super::{now_secs, FileGuard, ManagedStore, StoreError};
use rusqlite::{params, OptionalExtension};
use std::collections::{HashMap, HashSet};

/// Frozen experiment watermarks (§5.4): high 64 MiB, low 48 MiB, extra
/// temp/staging cap 8 MiB.
pub const GC_HIGH: u64 = 64 * 1024 * 1024;
pub const GC_LOW: u64 = 48 * 1024 * 1024;
pub const TEMP_CAP: u64 = 8 * 1024 * 1024;

pub fn gc_config_default() -> GcConfig {
    GcConfig { high: GC_HIGH, low: GC_LOW }
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
    let db = store.db();
    {
        let mut stmt = db.prepare("SELECT artifact FROM roots")?;
        for a in stmt.query_map([], |r| r.get::<_, String>(0))?.flatten() {
            seeds.insert(a);
        }
    }
    let leases: Vec<(String, u8, Option<String>)> = db
        .prepare("SELECT lease_id, released, artifact FROM leases")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .flatten()
        .collect();
    drop(db);
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
fn required_closure(store: &ManagedStore, seeds: &HashSet<String>) -> Result<HashSet<String>, StoreError> {
    let mut live: HashSet<String> = seeds.clone();
    let mut frontier: Vec<String> = seeds.iter().cloned().collect();
    while let Some(src) = frontier.pop() {
        let db = store.db();
        let dsts: Vec<String> = db
            .prepare("SELECT dst FROM edges WHERE src=?1 AND kind='required'")?
            .query_map(params![src], |r| r.get(0))?
            .flatten()
            .collect();
        drop(db);
        for d in dsts {
            if live.insert(d.clone()) {
                frontier.push(d);
            }
        }
    }
    Ok(live)
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

    // Live set and its chunks.
    let seeds = live_seed_artifacts(store)?;
    let live_artifacts = required_closure(store, &seeds)?;
    let db = store.db();
    let all_artifacts: Vec<(String, String, u64)> = db
        .prepare("SELECT artifact_id, payload_chunk, created_epoch FROM artifacts")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, u64>(2)?)))?
        .flatten()
        .collect();
    let chunk_states: HashMap<String, (String, u64)> = db
        .prepare("SELECT chunk_id, state, size FROM chunks")?
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, (r.get::<_, String>(1)?, r.get::<_, u64>(2)?)))
        })?
        .flatten()
        .collect();
    drop(db);
    receipt.scanned_chunks = chunk_states.len();
    let mut live_chunk_ids: HashSet<String> = HashSet::new();
    for (aid, cid, _) in &all_artifacts {
        if live_artifacts.contains(aid) {
            live_chunk_ids.insert(cid.clone());
        }
    }
    receipt.live_chunks = live_chunk_ids.len();

    let mut live_bytes = space.db + space.locks + space.logs;
    for cid in &live_chunk_ids {
        live_bytes += chunk_states.get(cid).map(|(_, s)| *s).unwrap_or(0);
    }
    receipt.live_bytes = live_bytes;

    // Batch 1: reclaimable junk (never live): stale staging (grace passed),
    // orphan blob files, trash residue.
    // Orphans are handled by `recover`; here we only clear old staging.
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
                            let age = now_secs()
                                .saturating_sub(
                                    modt.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
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
        let sz = ManagedStore::dir_size(&p);
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

    // Batch 2: unreachable chunks, oldest first, down to the low watermark.
    let mut candidates: Vec<(u64, String, u64)> = Vec::new(); // (created, chunk, size)
    for (aid, cid, created) in &all_artifacts {
        if !live_artifacts.contains(aid) && !live_chunk_ids.contains(cid) {
            if let Some((state, size)) = chunk_states.get(cid) {
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

    for (_, cid, size) in &candidates {
        let after = store.space().total;
        if cfg.low > 0 && after <= cfg.low {
            break;
        }
        // Invalidate logically first (tombstone), then take the gate.
        store.db().execute(
            "UPDATE chunks SET state='tombstoned' WHERE chunk_id=?1 AND state='live'",
            params![cid],
        )?;
        super::fault("after-tombstone");
        let gate = FileGuard::try_lock(
            &store.lock_path(&format!("gate-{}", cid)),
            true,
        )?;
        match gate {
            Some(_g) => {
                receipt.invalid_bytes += size;
                let removed = store.blob_remove(cid)?;
                // Converge metadata with physical state: the artifact rows
                // carrying this (now deleted) chunk are logically gone;
                // leaving them would block same-key republication forever.
                {
                    let db = store.db();
                    db.execute(
                        "DELETE FROM edges WHERE src IN (SELECT artifact_id FROM artifacts WHERE payload_chunk=?1)",
                        params![cid],
                    )?;
                    db.execute(
                        "DELETE FROM roots WHERE artifact IN (SELECT artifact_id FROM artifacts WHERE payload_chunk=?1)",
                        params![cid],
                    )?;
                    db.execute("DELETE FROM artifacts WHERE payload_chunk=?1", params![cid])?;
                }
                if removed {
                    receipt.freed_bytes += size;
                    receipt.deleted_chunks += 1;
                } else {
                    // Blocked remove (reader/AV holds a handle): the trash
                    // copy remains and is reported as pending, not freed.
                    // Rows were already removed — the pending-delete trash
                    // entry is reclaimed by `recover`.
                    receipt.pending_reader_bytes += size;
                    receipt.failed_deletes += 1;
                }
                store.db().execute(
                    "UPDATE chunks SET state='deleted' WHERE chunk_id=?1",
                    params![cid],
                )?;
            }
            None => {
                // A reader is active under the shared gate: revert to live
                // and record pending — the reader finishes untouched.
                receipt.pending_reader_bytes += size;
                store.db().execute(
                    "UPDATE chunks SET state='live' WHERE chunk_id=?1",
                    params![cid],
                )?;
            }
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
