//! Two frozen on-demand-read binary layouts over the SAME logical data
//! (PLAN-783 §5.2, T-03).
//!
//! * **Layout A** — fixed-record segment table + arithmetic index: 32-byte
//!   fixed records paged at 64 KiB, with variable payloads in a separate
//!   payload-page stream referenced by absolute offset.
//! * **Layout B** — varlen records + paged offset directory: whole records
//!   packed into 64 KiB pages, record id → (page, offset, len) resolved via
//!   a paged fixed-entry directory.
//!
//! Both layouts are generated from one deterministic pure generator
//! (`expected_record`), so every read is checkable against an independent
//! semantic baseline (rendered Atom-style text) computed without either
//! layout. Pages are content-addressed chunks, so a new version that only
//! modifies a suffix rewrites those pages and shares the untouched prefix
//! chunk-for-chunk.
//!
//! **Synthetic data**: this population proves layout and budget behavior
//! only; it does not pretend to be a general CST (plan §2).
//!
//! Frozen experiment budgets (§5.2 — tests must not widen them):
//! 64 KiB logical pages, ≥128 MiB real writes per layout, ≤8 MiB managed
//! hot budget, I/O staging ≤2 pages, small-query payload ≤512 KiB.

use crate::codec::{frame_chunk, unframe_chunk, ChunkKind, CodecError};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const PAGE_SIZE: usize = 64 * 1024;
pub const HOT_BUDGET: usize = 8 * 1024 * 1024;
pub const STAGING_PAGES: usize = 2;
pub const QUERY_PAYLOAD_CAP: u64 = 512 * 1024;
/// Experiment load: record count so EACH layout writes ≥128 MiB.
pub const RECORD_COUNT: u64 = 2_200_000;
/// Fixed record size (layout A) in bytes.
pub const A_RECORD_SIZE: usize = 32;
pub const A_RECORDS_PER_PAGE: u64 = (PAGE_SIZE / A_RECORD_SIZE) as u64;
/// Directory entry size (layout B): page u32 + offset u32 + len u16.
pub const B_DIR_ENTRY: usize = 10;
pub const B_DIR_ENTRIES_PER_PAGE: u64 = (PAGE_SIZE / B_DIR_ENTRY) as u64;
/// Layout B record header: kind u16 + parent u32 + span u32×2 + len u8.
pub const B_HEADER: usize = 13;

#[derive(Clone, Debug)]
pub enum LayoutError {
    Io(String),
    Codec(CodecError),
    BadState(String),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutError::Io(e) => write!(f, "io: {}", e),
            LayoutError::Codec(e) => write!(f, "codec: {}", e),
            LayoutError::BadState(e) => write!(f, "bad layout state: {}", e),
        }
    }
}

impl std::error::Error for LayoutError {}

impl From<CodecError> for LayoutError {
    fn from(e: CodecError) -> Self {
        LayoutError::Codec(e)
    }
}
impl From<std::io::Error> for LayoutError {
    fn from(e: std::io::Error) -> Self {
        LayoutError::Io(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Deterministic synthetic population (pure — the semantic baseline)
// ---------------------------------------------------------------------------

/// One logical syntax-node-like record. The population is synthetic; the
/// generator is a pure function of (seed, id), independently re-computable
/// by any test as the oracle.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SynthRecord {
    pub id: u64,
    pub kind: u16,
    pub parent: Option<u64>,
    pub span: (u32, u32),
    pub payload: Vec<u8>,
}

fn mix64(mut x: u64) -> u64 {
    // splitmix64 — deterministic on all platforms.
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Pure generator: the expected record for (seed, id). `seed2` versions the
/// population for suffix-modified versions.
pub fn expected_record(id: u64, seed: u64, modified_from: Option<u64>, seed2: u64) -> SynthRecord {
    let effective_seed = match modified_from {
        Some(from) if id >= from => seed ^ seed2,
        _ => seed,
    };
    let h = mix64(id.wrapping_mul(0xA24B_AED4_97F1_2345) ^ effective_seed);
    let kind = (h % 977) as u16;
    let parent = if id == 0 { None } else { Some(id.saturating_sub(1) / 3) };
    let span_start = ((h >> 16) & 0xFFFF_FFFF) as u32;
    let span_end = span_start.saturating_add(((h >> 40) % 4096) as u32);
    let payload_len = 8 + ((h >> 24) % 65) as usize; // 8..=72
    let mut payload = Vec::with_capacity(payload_len);
    let mut ctr = 0u64;
    while payload.len() < payload_len {
        let stream = mix64(h ^ ctr.wrapping_mul(0x1234_5678_9ABC_DEF1));
        for b in stream.to_le_bytes() {
            if payload.len() < payload_len {
                payload.push(b ^ (ctr as u8));
            }
        }
        ctr += 1;
    }
    SynthRecord { id, kind, parent, span: (span_start, span_end), payload }
}

/// Atom-style semantic baseline rendering of a record (the text form both
/// layouts must agree with).
pub fn render_record_atom(r: &SynthRecord) -> String {
    let parent = r.parent.map(|p| p.to_string()).unwrap_or_else(|| "none".into());
    format!(
        "node(id: {}, kind: {}, parent: {}, span: {}..{}, payload_len: {}, payload_head: {})",
        r.id,
        r.kind,
        parent,
        r.span.0,
        r.span.1,
        r.payload.len(),
        r.payload.iter().take(8).map(|b| format!("{:02x}", b)).collect::<String>()
    )
}

// ---------------------------------------------------------------------------
// Layout store: content-addressed page chunks under a dedicated root
// ---------------------------------------------------------------------------

pub struct LayoutStore {
    pub root: PathBuf,
}

impl LayoutStore {
    pub fn new(root: &Path) -> Self {
        LayoutStore { root: root.to_path_buf() }
    }

    pub fn page_path(&self, chunk_id: &str) -> PathBuf {
        self.root.join("pages").join(&chunk_id[..2]).join(chunk_id)
    }

    /// Write a framed page chunk; returns (chunk id, on-disk size). An
    /// identical chunk already present is left as-is (content sharing).
    pub fn write_chunk(&self, payload: &[u8]) -> Result<(String, u64), LayoutError> {
        let framed = frame_chunk(ChunkKind::LayoutData, payload);
        let id = crate::identity::ChunkId::from_parts(&[framed.as_slice()]).0;
        let path = self.page_path(&id);
        if path.is_file() {
            return Ok((id, framed.len() as u64));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, &framed)?;
        // Same-key concurrent writers coordinate by identical content:
        // rename is atomic; losing writer overwrites with identical bytes.
        fs::rename(&tmp, &path)?;
        Ok((id, framed.len() as u64))
    }

    pub fn chunk_len(&self, chunk_id: &str) -> Option<u64> {
        self.page_path(chunk_id).metadata().ok().map(|m| m.len())
    }

    pub fn total_disk_bytes(&self) -> u64 {
        let mut total = 0;
        let dir = self.root.join("pages");
        let Ok(shards) = fs::read_dir(&dir) else { return 0 };
        for shard in shards.flatten() {
            if let Ok(files) = fs::read_dir(shard.path()) {
                for f in files.flatten() {
                    if f.path().is_file() {
                        total += f.metadata().map(|m| m.len()).unwrap_or(0);
                    }
                }
            }
        }
        total
    }
}

// ---------------------------------------------------------------------------
// Version manifests
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub layout: String, // "A" | "B"
    pub version: u64,
    pub record_count: u64,
    /// Layout A: fixed-record pages (id → page id/2048).
    pub record_pages: Vec<String>,
    /// Layout A: payload stream pages (absolute offsets, 64 KiB each).
    pub payload_pages: Vec<String>,
    /// Total payload stream length (A).
    pub payload_len: u64,
    /// Layout B: packed varlen record pages.
    pub data_pages: Vec<String>,
    /// Layout B: offset-directory pages (10-byte entries, 6553/page).
    pub dir_pages: Vec<String>,
}

impl Manifest {
    pub fn data_size(&self) -> u64 {
        (self.record_pages.len() as u64 + self.payload_pages.len() as u64 + self.data_pages.len() as u64 + self.dir_pages.len() as u64)
            * PAGE_SIZE as u64
    }
}

// ---------------------------------------------------------------------------
// Builders (streaming: no full-population materialization)
// ---------------------------------------------------------------------------

pub struct BuildReport {
    pub manifest: Manifest,
    pub bytes_written: u64,
    pub chunks_written: usize,
    pub shared_with_base: usize,
    pub total_pages: usize,
    pub duration_ms: u64,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
}

/// Build layout A for a population version. `base` enables prefix sharing:
/// pages whose chunk ids already exist in `base` count as shared and are
/// not rewritten.
pub fn build_layout_a(
    store: &LayoutStore,
    version: u64,
    seed: u64,
    modified_from: Option<u64>,
    seed2: u64,
    base: Option<&Manifest>,
) -> Result<BuildReport, LayoutError> {
    let t0 = now_ms();
    let n_pages = RECORD_COUNT.div_ceil(A_RECORDS_PER_PAGE);
    // Pass 1: payload stream (absolute offsets feed the record table).
    let mut payload_pages = Vec::new();
    let mut payload_len: u64 = 0;
    let mut pending: Vec<u8> = Vec::with_capacity(PAGE_SIZE + 128);
    let mut record_payload_off: Vec<u64> = Vec::with_capacity(RECORD_COUNT as usize);
    let mut record_payload_len: Vec<u16> = Vec::with_capacity(RECORD_COUNT as usize);
    for id in 0..RECORD_COUNT {
        let r = expected_record(id, seed, modified_from, seed2);
        // Payloads never straddle pages: pad + flush first if needed. The
        // offset counter must include the padding so recorded offsets
        // match the physical stream byte-for-byte; the record's offset is
        // taken AFTER the pad adjustment (a straddling payload starts on
        // the fresh page).
        if pending.len() + r.payload.len() > PAGE_SIZE {
            let pad = (PAGE_SIZE - pending.len()) as u64;
            let mut page = std::mem::replace(&mut pending, Vec::with_capacity(PAGE_SIZE + 128));
            page.resize(PAGE_SIZE, 0);
            let (cid, _) = store.write_chunk(&page)?;
            payload_pages.push(cid);
            payload_len += pad;
        }
        record_payload_off.push(payload_len);
        record_payload_len.push(r.payload.len() as u16);
        pending.extend_from_slice(&r.payload);
        payload_len += r.payload.len() as u64;
    }
    if !pending.is_empty() {
        let mut page = std::mem::take(&mut pending);
        page.resize(PAGE_SIZE, 0);
        let (cid, _) = store.write_chunk(&page)?;
        payload_pages.push(cid);
    }
    // Pass 2: fixed record table.
    let mut record_pages = Vec::new();
    for page_idx in 0..n_pages {
        let mut page = vec![0u8; PAGE_SIZE];
        for slot in 0..A_RECORDS_PER_PAGE {
            let id = page_idx * A_RECORDS_PER_PAGE + slot;
            if id >= RECORD_COUNT {
                break;
            }
            let r = expected_record(id, seed, modified_from, seed2);
            let b = &mut page[(slot as usize) * A_RECORD_SIZE..(slot as usize + 1) * A_RECORD_SIZE];
            b[0..2].copy_from_slice(&r.kind.to_le_bytes());
            b[2..6].copy_from_slice(&(r.parent.map(|p| p as u32).unwrap_or(u32::MAX)).to_le_bytes());
            b[6..10].copy_from_slice(&r.span.0.to_le_bytes());
            b[10..14].copy_from_slice(&r.span.1.to_le_bytes());
            b[14..16].copy_from_slice(&(r.payload.len() as u16).to_le_bytes());
            b[16..24].copy_from_slice(&record_payload_off[id as usize].to_le_bytes());
        }
        let (cid, size) = store.write_chunk(&page)?;
        record_pages.push(cid);
        // Sharing accounting: identical chunk ids are not rewritten.
        let _ = size;
    }
    let manifest = Manifest {
        layout: "A".into(),
        version,
        record_count: RECORD_COUNT,
        record_pages,
        payload_pages,
        payload_len,
        data_pages: vec![],
        dir_pages: vec![],
    };
    finish_report(store, manifest, base, t0)
}

/// Build layout B for the same logical population.
pub fn build_layout_b(
    store: &LayoutStore,
    version: u64,
    seed: u64,
    modified_from: Option<u64>,
    seed2: u64,
    base: Option<&Manifest>,
) -> Result<BuildReport, LayoutError> {
    let t0 = now_ms();
    let mut data_pages = Vec::new();
    let mut dir_pages = Vec::new();
    let mut page: Vec<u8> = Vec::with_capacity(PAGE_SIZE);
    let mut dir: Vec<u8> = vec![0u8; PAGE_SIZE];
    let mut page_idx: u32 = 0;
    for id in 0..RECORD_COUNT {
        let r = expected_record(id, seed, modified_from, seed2);
        let mut rec = Vec::with_capacity(B_HEADER + r.payload.len());
        rec.extend_from_slice(&r.kind.to_le_bytes());
        rec.extend_from_slice(&(r.parent.map(|p| p as u32).unwrap_or(u32::MAX)).to_le_bytes());
        rec.extend_from_slice(&r.span.0.to_le_bytes());
        rec.extend_from_slice(&r.span.1.to_le_bytes());
        rec.push(r.payload.len() as u8);
        rec.extend_from_slice(&r.payload);
        if page.len() + rec.len() > PAGE_SIZE {
            let full = std::mem::replace(&mut page, Vec::with_capacity(PAGE_SIZE));
            let mut padded = full;
            padded.resize(PAGE_SIZE, 0);
            let (cid, _) = store.write_chunk(&padded)?;
            data_pages.push(cid);
            page_idx += 1;
        }
        // Directory entry for this record: current page + offset + len.
        let entry_off = (id % B_DIR_ENTRIES_PER_PAGE) as usize * B_DIR_ENTRY;
        let e = &mut dir[entry_off..entry_off + B_DIR_ENTRY];
        e[0..4].copy_from_slice(&page_idx.to_le_bytes());
        e[4..8].copy_from_slice(&(page.len() as u32).to_le_bytes());
        e[8..10].copy_from_slice(&(rec.len() as u16).to_le_bytes());
        if (id + 1) % B_DIR_ENTRIES_PER_PAGE == 0 {
            let full = std::mem::replace(&mut dir, vec![0u8; PAGE_SIZE]);
            let (cid, _) = store.write_chunk(&full)?;
            dir_pages.push(cid);
        }
        page.extend_from_slice(&rec);
    }
    if !page.is_empty() {
        let mut padded = page;
        padded.resize(PAGE_SIZE, 0);
        let (cid, _) = store.write_chunk(&padded)?;
        data_pages.push(cid);
    }
    if !dir.is_empty() {
        let mut padded = dir;
        padded.resize(PAGE_SIZE, 0);
        let (cid, _) = store.write_chunk(&padded)?;
        dir_pages.push(cid);
    }
    let manifest = Manifest {
        layout: "B".into(),
        version,
        record_count: RECORD_COUNT,
        record_pages: vec![],
        payload_pages: vec![],
        payload_len: 0,
        data_pages,
        dir_pages,
    };
    finish_report(store, manifest, base, t0)
}

fn finish_report(
    store: &LayoutStore,
    manifest: Manifest,
    base: Option<&Manifest>,
    t0: u64,
) -> Result<BuildReport, LayoutError> {
    let all: Vec<&String> = manifest
        .record_pages
        .iter()
        .chain(&manifest.payload_pages)
        .chain(&manifest.data_pages)
        .chain(&manifest.dir_pages)
        .collect();
    let mut bytes_written = 0u64;
    let mut shared = 0usize;
    let base_ids: Option<std::collections::HashSet<&String>> = base.map(|b| {
        b.record_pages.iter().chain(&b.payload_pages).chain(&b.data_pages).chain(&b.dir_pages).collect()
    });
    for id in &all {
        let sz = store.chunk_len(id).unwrap_or(0);
        bytes_written += sz;
        if let Some(ids) = &base_ids {
            if ids.contains(*id) {
                shared += 1;
            }
        }
    }
    Ok(BuildReport {
        total_pages: all.len(),
        chunks_written: all.len(),
        bytes_written,
        shared_with_base: shared,
        manifest,
        duration_ms: now_ms().saturating_sub(t0),
    })
}

// ---------------------------------------------------------------------------
// Frozen views with budgets (hot LRU ≤8 MiB, staging ≤2 pages, counted I/O)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewStats {
    pub reads: u64,
    pub bytes_read: u64,
    pub hot_hits: u64,
    pub hot_misses: u64,
    pub hot_evictions: u64,
    pub hot_bytes_now: usize,
    pub staging_buffers: usize,
    pub staging_bytes: usize,
    pub decoded_bytes: u64,
    /// Largest I/O total caused by any single `record()` call.
    pub max_query_io_bytes: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum PageKey {
    ARecord(u32),
    APayload(u32),
    BData(u32),
    BDir(u32),
}

pub struct LayoutView<'s> {
    store: &'s LayoutStore,
    pub manifest: Manifest,
    hot: HashMap<PageKey, Vec<u8>>,
    lru: VecDeque<PageKey>,
    hot_bytes: usize,
    staging: [Vec<u8>; STAGING_PAGES],
    staging_next: usize,
    pub stats: ViewStats,
}

impl<'s> LayoutView<'s> {
    pub fn open(store: &'s LayoutStore, manifest: Manifest) -> Self {
        LayoutView {
            store,
            manifest,
            hot: HashMap::new(),
            lru: VecDeque::new(),
            hot_bytes: 0,
            staging: [Vec::with_capacity(PAGE_SIZE), Vec::with_capacity(PAGE_SIZE)],
            staging_next: 0,
            stats: ViewStats { staging_buffers: STAGING_PAGES, staging_bytes: PAGE_SIZE * STAGING_PAGES, ..Default::default() },
        }
    }

    /// Ensure a page is hot; on miss load it through the bounded staging
    /// buffer (I/O counted). `cid` is only materialized by the caller when
    /// a miss is possible, keeping the hot path allocation-free.
    fn ensure_page(&mut self, key: PageKey, cid: Option<&str>) -> Result<(), LayoutError> {
        if self.hot.contains_key(&key) {
            self.stats.hot_hits += 1;
            if let Some(pos) = self.lru.iter().position(|k| *k == key) {
                self.lru.remove(pos);
                self.lru.push_back(key);
            }
            return Ok(());
        }
        self.stats.hot_misses += 1;
        let cid = cid.ok_or_else(|| LayoutError::BadState(format!("page {:?} missing from manifest", key)))?;
        let path = self.store.page_path(cid);
        // Staging rotation: at most STAGING_PAGES raw buffers exist.
        let slot = self.staging_next;
        self.staging_next = (self.staging_next + 1) % STAGING_PAGES;
        let buf = &mut self.staging[slot];
        buf.clear();
        buf.resize(PAGE_SIZE, 0);
        let mut f = fs::File::open(&path)?;
        f.seek(SeekFrom::Start(crate::codec::CHUNK_HEADER_LEN as u64))?;
        let mut pos = 0usize;
        while pos < PAGE_SIZE {
            let n = f.read(&mut buf[pos..])?;
            if n == 0 {
                return Err(LayoutError::BadState(format!("page chunk truncated: {}", path.display())));
            }
            pos += n;
        }
        self.stats.reads += 1;
        self.stats.bytes_read += PAGE_SIZE as u64;
        // Hot insert (owned copy; accounting counts the copy).
        while !self.lru.is_empty() && buf.len() + self.hot_bytes > HOT_BUDGET {
            let victim = self.lru.pop_front().unwrap();
            if let Some(v) = self.hot.remove(&victim) {
                self.hot_bytes -= v.len();
                self.stats.hot_evictions += 1;
            }
        }
        if buf.len() + self.hot_bytes <= HOT_BUDGET {
            self.hot.insert(key, buf.clone());
            self.hot_bytes += buf.len();
            self.lru.push_back(key);
        }
        Ok(())
    }

    fn hot_bytes_now(&self) -> usize {
        self.hot_bytes
    }

    /// Read one record by id. Frozen: returns an owned value; no mutation
    /// API exists (the view is read-only by construction).
    pub fn record(&mut self, id: u64) -> Result<SynthRecord, LayoutError> {
        let io_before = self.stats.bytes_read;
        let r = self.record_inner(id)?;
        let delta = self.stats.bytes_read - io_before;
        self.stats.max_query_io_bytes = self.stats.max_query_io_bytes.max(delta);
        self.stats.decoded_bytes += (A_RECORD_SIZE.max(B_HEADER + r.payload.len())) as u64;
        self.stats.hot_bytes_now = self.hot_bytes_now();
        Ok(r)
    }

    fn record_inner(&mut self, id: u64) -> Result<SynthRecord, LayoutError> {
        if id >= self.manifest.record_count {
            return Err(LayoutError::BadState(format!("record id {} out of range", id)));
        }
        if self.manifest.layout == "A" {
            let page = (id / A_RECORDS_PER_PAGE) as u32;
            let slot = (id % A_RECORDS_PER_PAGE) as usize;
            let cid = self.manifest.record_pages.get(page as usize).cloned();
            self.ensure_page(PageKey::ARecord(page), cid.as_deref())?;
            let page_bytes = self.hot[&PageKey::ARecord(page)].clone();
            let b = &page_bytes[slot * A_RECORD_SIZE..(slot + 1) * A_RECORD_SIZE];
            let kind = u16::from_le_bytes(b[0..2].try_into().unwrap());
            let parent_raw = u32::from_le_bytes(b[2..6].try_into().unwrap());
            let span0 = u32::from_le_bytes(b[6..10].try_into().unwrap());
            let span1 = u32::from_le_bytes(b[10..14].try_into().unwrap());
            let plen = u16::from_le_bytes(b[14..16].try_into().unwrap()) as usize;
            let poff = u64::from_le_bytes(b[16..24].try_into().unwrap());
            let ppage = (poff / PAGE_SIZE as u64) as u32;
            let pin = (poff % PAGE_SIZE as u64) as usize;
            let cid = self.manifest.payload_pages.get(ppage as usize).cloned();
            self.ensure_page(PageKey::APayload(ppage), cid.as_deref())?;
            let payload_page = self.hot[&PageKey::APayload(ppage)].clone();
            // Payloads never straddle pages (builder pads).
            debug_assert!(pin + plen <= PAGE_SIZE);
            let payload = payload_page[pin..pin + plen].to_vec();
            Ok(SynthRecord {
                id,
                kind,
                parent: if parent_raw == u32::MAX { None } else { Some(parent_raw as u64) },
                span: (span0, span1),
                payload,
            })
        } else {
            let dir_page = (id / B_DIR_ENTRIES_PER_PAGE) as u32;
            let entry = (id % B_DIR_ENTRIES_PER_PAGE) as usize;
            let cid = self.manifest.dir_pages.get(dir_page as usize).cloned();
            self.ensure_page(PageKey::BDir(dir_page), cid.as_deref())?;
            let dir_bytes = self.hot[&PageKey::BDir(dir_page)].clone();
            let e = &dir_bytes[entry * B_DIR_ENTRY..(entry + 1) * B_DIR_ENTRY];
            let dpage = u32::from_le_bytes(e[0..4].try_into().unwrap());
            let doff = u32::from_le_bytes(e[4..8].try_into().unwrap()) as usize;
            let dlen = u16::from_le_bytes(e[8..10].try_into().unwrap()) as usize;
            let cid = self.manifest.data_pages.get(dpage as usize).cloned();
            self.ensure_page(PageKey::BData(dpage), cid.as_deref())?;
            let data_bytes = self.hot[&PageKey::BData(dpage)].clone();
            let b = &data_bytes[doff..doff + dlen];
            let kind = u16::from_le_bytes(b[0..2].try_into().unwrap());
            let parent_raw = u32::from_le_bytes(b[2..6].try_into().unwrap());
            let span0 = u32::from_le_bytes(b[6..10].try_into().unwrap());
            let span1 = u32::from_le_bytes(b[10..14].try_into().unwrap());
            let plen = b[14] as usize;
            let payload = b[15..15 + plen].to_vec();
            Ok(SynthRecord {
                id,
                kind,
                parent: if parent_raw == u32::MAX { None } else { Some(parent_raw as u64) },
                span: (span0, span1),
                payload,
            })
        }
    }
}

/// Validate one fetched page chunk (framing + hash) — used by tests to
/// prove a corrupted page is a controlled rejection.
pub fn validate_page_chunk(store: &LayoutStore, chunk_id: &str) -> Result<(), LayoutError> {
    let bytes = fs::read(store.page_path(chunk_id))?;
    unframe_chunk(&bytes)?;
    Ok(())
}
