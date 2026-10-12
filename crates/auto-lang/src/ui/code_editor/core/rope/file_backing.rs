// PLAN-728 T-01..T-05 — file-backed paged rope backing (供⑮ 面①..⑤).
//
// The immutable BASE of a large document lives in its file; the rope holds
// `Node::Chunk` leaf descriptors ({page, offset, length} + prescan-computed
// summaries {chars, newlines, hash, start_chars, end_chars}) instead of the
// text. Text is materialized per page on demand into a shared LRU-bounded
// page cache (`PageStore`); base pages are NEVER modified in place (frozen
// ③ — offsets stay true, which is what the merge-save copy and the digest
// shortcuts rely on).
//
// Prescan (面①): one sequential pass records per-page summaries + validates
// UTF-8 + cuts page boundaries on char boundaries, so every page decodes
// independently and whole-page digests are available without residency
// (703 Merkle shortcuts keep working over unmaterialized chunks).
//
// Async prefetch (面④): a background thread warms pages around reads; the
// query path itself fault-ins synchronously (one page ≈64KB ≈100μs on
// NVMe — inside frame budget; frozen ④ is delivered by prefetch keeping
// the scroll/edit window warm, not by blocking on large loads).

use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::poly_digest;

/// Default page size (T-00③: 64KB — 1GB = 16384 pages ≈ 0.5MB table; fine
/// fault granularity for the ~10MB-class residency contract).
pub(crate) const DEFAULT_PAGE_BYTES: u32 = 64 * 1024;
/// Default page-cache budget (T-00⑦: 6MB — with the ~3MB@1GB page table +
/// tree descriptors the residency contract is "10MB-class", asserted at
/// ≤12MB structural in the bench谱).
pub(crate) const DEFAULT_CACHE_BUDGET: usize = 6 * 1024 * 1024;
/// Default prefetch radius in pages, both directions (T-00⑥: ±16 pages
/// ≈ ±1MB around the read window).
pub(crate) const DEFAULT_PREFETCH_RADIUS: u32 = 16;
/// Minimum page size — the prescan loop must always make progress past a
/// multi-byte char, and the cache needs at least a couple of pages.
const MIN_PAGE_BYTES: u32 = 256;

/// Tuning knobs for `Rope::open_file_backed` (test seams included: small
/// pages exercise multi-page logic on tiny fixtures).
#[derive(Debug, Clone)]
pub(crate) struct PageConfig {
    /// Page size in bytes.
    pub page_bytes: u32,
    /// Resident page-cache budget in bytes (LRU eviction above it).
    pub cache_budget: usize,
    /// Prefetch radius in pages around a read.
    pub prefetch_radius: u32,
}

impl Default for PageConfig {
    fn default() -> Self {
        PageConfig {
            page_bytes: DEFAULT_PAGE_BYTES,
            cache_budget: DEFAULT_CACHE_BUDGET,
            prefetch_radius: DEFAULT_PREFETCH_RADIUS,
        }
    }
}

/// One immutable base page: file range + prescan summaries. The summaries
/// make whole-page `Node::Chunk`s answerable without residency (line count,
/// digests, first/last-line lengths).
#[derive(Debug, Clone, Copy)]
pub(crate) struct PageDesc {
    /// Absolute file offset of the page start.
    pub offset: u64,
    /// Page length in bytes (cut back to a char boundary at prescan).
    pub len: u32,
    /// Chars in the page (non-continuation bytes).
    pub chars: u32,
    /// `'\n'` occurrences in the page.
    pub newlines: u32,
    /// Chars before the page's first `'\n'` (first line's length).
    pub start_chars: u32,
    /// Chars after the page's last `'\n'` (last line's length).
    pub end_chars: u32,
    /// `poly_digest` of the page bytes (content-determined — equals the
    /// flat digest regardless of rope shape).
    pub hash: u64,
}

/// Page residency state — the observability face for the placeholder
/// semantics (面④: prefetch warmth is probe-observable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageState {
    /// Text materialized in the cache.
    Resident,
    /// Not resident (next read fault-ins from disk).
    Cold,
}

/// External-change baseline: what the base file looked like at prescan.
/// A save whose base changed underneath (length or mtime) is rejected —
/// offsets into a mutated file would silently corrupt the merge copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    mtime: Option<SystemTime>,
}

fn stamp_of(path: &Path) -> FileStamp {
    match std::fs::metadata(path) {
        Ok(m) => FileStamp {
            len: m.len(),
            mtime: m.modified().ok(),
        },
        Err(_) => FileStamp {
            len: u64::MAX,
            mtime: None,
        },
    }
}

struct PageCache {
    map: HashMap<u32, Arc<str>>,
    /// LRU recency, front = least recently used, back = most recent.
    order: VecDeque<u32>,
    bytes: usize,
    budget: usize,
}

impl PageCache {
    fn new(budget: usize) -> Self {
        PageCache {
            map: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
            budget,
        }
    }

    fn get(&mut self, page: u32) -> Option<Arc<str>> {
        let text = self.map.get(&page).cloned()?;
        if let Some(pos) = self.order.iter().rposition(|&p| p == page) {
            self.order.remove(pos);
            self.order.push_back(page);
        }
        Some(text)
    }

    fn insert(&mut self, page: u32, text: Arc<str>) {
        let len = text.len();
        if self.map.insert(page, text).is_none() {
            self.order.push_back(page);
            self.bytes += len;
        }
        // Evict LRU pages over budget. A reader may still hold an evicted
        // Arc — dropping our handle only ends caching, not the borrow.
        while self.bytes > self.budget {
            let Some(victim) = self.order.pop_front() else {
                break;
            };
            if let Some(t) = self.map.remove(&victim) {
                self.bytes -= t.len();
            }
        }
    }
}

/// The shared backing store of every `Node::Chunk` from one `open_file_backed`:
/// the kept-open read handle (base immutability + merge-save copy source),
/// the immutable page table, the LRU page cache, and the prefetch worker
/// handle. `Send + Sync` (rope snapshots cross threads with it).
pub struct PageStore {
    path: PathBuf,
    file: File,
    file_len: u64,
    page_bytes: u32,
    prefetch_radius: u32,
    pages: Vec<PageDesc>,
    baseline: Mutex<FileStamp>,
    cache: Mutex<PageCache>,
    /// Prefetch requests; `None` when no worker runs (spawn failure or
    /// store teardown — sync fault-in keeps working without it).
    prefetch_tx: Mutex<Option<Sender<PrefetchReq>>>,
    /// Pages materialized by the prefetch worker (observability/probes).
    prefetched_pages: AtomicU64,
}

type PrefetchReq = (u32, u32); // [start, end) page range

#[cfg(windows)]
fn pread_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    use std::os::windows::fs::FileExt;
    file.seek_read(buf, offset)
}

#[cfg(unix)]
fn pread_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    use std::os::unix::fs::FileExt;
    file.read_at(buf, offset)
}

/// Read `buf.len()` bytes at `offset`, looping over short reads.
fn pread_exact(file: &File, buf: &mut [u8], mut offset: u64) -> io::Result<()> {
    let mut filled = 0usize;
    while filled < buf.len() {
        let n = pread_at(file, &mut buf[filled..], offset)?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("short read at offset {offset} (base file truncated?)"),
            ));
        }
        filled += n;
        offset += n as u64;
    }
    Ok(())
}

fn utf8_io_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "stream did not contain valid UTF-8",
    )
}

impl PageStore {
    /// Prescan (面①): sequential pass building the page table. One read
    /// syscall per page (sequential offsets — OS readahead carries the
    /// throughput); per page: char-boundary cut, UTF-8 validation (mirrors
    /// `read_to_string`'s InvalidData), summary counts, flat digest.
    pub(crate) fn prescan(path: &Path, cfg: &PageConfig) -> io::Result<Arc<PageStore>> {
        let page_bytes = cfg.page_bytes.max(MIN_PAGE_BYTES);
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        let mut pages: Vec<PageDesc> = Vec::with_capacity((len as usize / page_bytes as usize) + 1);
        let mut buf = vec![0u8; page_bytes as usize];
        let mut offset = 0u64;
        while offset < len {
            let want = ((len - offset) as usize).min(buf.len());
            let at_eof = offset + want as u64 >= len;
            pread_exact(&file, &mut buf[..want], offset)?;
            // Page-boundary cut: std's UTF-8 validator pinpoints a char
            // straddling the page tail (error_len() == None = incomplete
            // sequence AT THE END; valid_up_to() = the boundary to cut at).
            // A mid-file page backs up ≤3 bytes; the EOF page never backs
            // up (a file ending mid-char is invalid, like read_to_string).
            let n = match std::str::from_utf8(&buf[..want]) {
                Ok(_) => want,
                Err(e) => {
                    let cut = e.valid_up_to();
                    if !at_eof && e.error_len().is_none() && cut > 0 {
                        cut
                    } else {
                        return Err(utf8_io_error());
                    }
                }
            };
            let text = std::str::from_utf8(&buf[..n]).map_err(|_| utf8_io_error())?;
            pages.push(PageDesc {
                offset,
                len: n as u32,
                chars: text.chars().count() as u32,
                newlines: text.bytes().filter(|&b| b == b'\n').count() as u32,
                start_chars: text.chars().take_while(|&c| c != '\n').count() as u32,
                end_chars: text.chars().rev().take_while(|&c| c != '\n').count() as u32,
                hash: poly_digest(text.as_bytes()),
            });
            offset += n as u64;
        }
        let store = Arc::new(PageStore {
            path: path.to_path_buf(),
            file,
            file_len: len,
            page_bytes,
            prefetch_radius: cfg.prefetch_radius,
            pages,
            baseline: Mutex::new(stamp_of(path)),
            cache: Mutex::new(PageCache::new(
                cfg.cache_budget.max(page_bytes as usize * 2),
            )),
            prefetch_tx: Mutex::new(None),
            prefetched_pages: AtomicU64::new(0),
        });
        Ok(store)
    }

    // ── page table ───────────────────────────────────────────────────────

    pub(crate) fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub(crate) fn page_desc(&self, page: u32) -> &PageDesc {
        &self.pages[page as usize]
    }

    /// Absolute file offset of byte `off` inside `page`.
    pub(crate) fn range_offset(&self, page: u32, off: u32) -> u64 {
        self.pages[page as usize].offset + off as u64
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn file_len(&self) -> u64 {
        self.file_len
    }

    // ── page cache (fault-in + LRU) ──────────────────────────────────────

    /// Materialize page text (sync fault-in on miss; LRU insert). Errors on
    /// IO/decode failure (the handle is held open, so this means storage
    /// vanished mid-session — surfaced, never faked).
    pub(crate) fn page_text(&self, page: u32) -> io::Result<Arc<str>> {
        if let Some(text) = self.cache.lock().unwrap().get(page) {
            return Ok(text);
        }
        let desc = &self.pages[page as usize];
        let mut buf = vec![0u8; desc.len as usize];
        pread_exact(&self.file, &mut buf, desc.offset)?;
        let text: Arc<str> = Arc::from(String::from_utf8(buf).map_err(|_| utf8_io_error())?);
        self.cache.lock().unwrap().insert(page, text.clone());
        Ok(text)
    }

    /// Residency probe (面④ observability).
    pub fn page_state(&self, page: u32) -> PageState {
        match self.cache.lock().unwrap().map.contains_key(&page) {
            true => PageState::Resident,
            false => PageState::Cold,
        }
    }

    /// Resident page-cache bytes (memory-contract accounting face).
    pub fn cache_bytes(&self) -> usize {
        self.cache.lock().unwrap().bytes
    }

    /// Structural residency estimate: page cache + page table. The rope
    /// tree's node overhead is counted by the rope face
    /// (`Rope::structural_resident_estimate`); the sum is the ≤12MB@1GB
    /// contract assertion in the bench谱 (AC-03).
    pub fn structural_resident_bytes(&self) -> usize {
        let cache = self.cache_bytes();
        let table = self.pages.capacity().max(self.pages.len()) * std::mem::size_of::<PageDesc>();
        cache + table + std::mem::size_of::<PageStore>()
    }

    /// Total pages materialized by the prefetch worker so far (probe face).
    pub fn prefetched_pages(&self) -> u64 {
        self.prefetched_pages.load(Ordering::Relaxed)
    }

    // ── async prefetch (面④) ─────────────────────────────────────────────

    /// Post a prefetch around `page` (configured radius, clamped to the
    /// table). Never blocks the caller — the worker fills the cache
    /// off-thread; `page_state` flips to Resident as pages land.
    pub fn prefetch_around(&self, page: u32) {
        let r = self.prefetch_radius;
        self.prefetch_range(
            page.saturating_sub(r),
            page.saturating_add(r).saturating_add(1),
        );
    }

    /// Prefetch the page range covering byte range `[start, end)` plus the
    /// configured radius on both sides.
    pub(crate) fn prefetch_byte_range(&self, start: u64, end: u64) {
        if self.pages.is_empty() || end <= start {
            return;
        }
        let pb = self.page_bytes as u64;
        let first = (start / pb) as u32;
        let last = (((end - 1) / pb) as u32).saturating_add(1);
        let r = self.prefetch_radius;
        self.prefetch_range(first.saturating_sub(r), last.saturating_add(r));
    }

    fn prefetch_range(&self, start: u32, end: u32) {
        let start = start.min(self.pages.len() as u32);
        let end = end.min(self.pages.len() as u32);
        if start >= end {
            return;
        }
        if let Some(tx) = self.prefetch_tx.lock().unwrap().as_ref() {
            let _ = tx.send((start, end));
        }
    }

    // ── merge-save copy source (面⑤) ─────────────────────────────────────

    /// Read `[offset, offset+out.len())` from the BASE handle — the
    /// merge-save copy path. Deliberately bypasses the LRU (unchanged
    /// regions stream disk→disk through a copy buffer; no cache churn).
    pub(crate) fn read_base_range(&self, offset: u64, out: &mut [u8]) -> io::Result<()> {
        pread_exact(&self.file, out, offset)
    }

    // ── external-change detection (面⑤ 随形) ─────────────────────────────

    /// Whether the base file still matches the prescan baseline (length +
    /// mtime). A save over a mutated base is rejected (报错不静默).
    pub(crate) fn baseline_matches(&self) -> bool {
        *self.baseline.lock().unwrap() == stamp_of(&self.path)
    }

    /// Re-stamp the baseline after this store's own successful save.
    pub(crate) fn refresh_baseline(&self) {
        *self.baseline.lock().unwrap() = stamp_of(&self.path);
    }
}

/// Spawn the prefetch worker on a fully-constructed store Arc (the worker
/// holds a clone; channel teardown ends it when the last external Arc
/// drops). Called by `Rope::open_file_backed` right after `prescan`.
pub(crate) fn spawn_prefetcher(store: &Arc<PageStore>) {
    let (tx, rx): (Sender<PrefetchReq>, Receiver<PrefetchReq>) = std::sync::mpsc::channel();
    let worker = Arc::clone(store);
    let spawned = std::thread::Builder::new()
        .name("p728-prefetch".into())
        .spawn(move || {
            while let Ok((start, end)) = rx.recv() {
                for p in start..end {
                    if worker.page_state(p) == PageState::Resident {
                        continue;
                    }
                    if worker.page_text(p).is_ok() {
                        worker.prefetched_pages.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
    match spawned {
        Ok(_) => {
            *store.prefetch_tx.lock().unwrap() = Some(tx);
        }
        Err(e) => {
            eprintln!("paged-rope: prefetch worker unavailable (sync fault-in only): {e}");
        }
    }
}

impl std::fmt::Debug for PageStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageStore")
            .field("path", &self.path)
            .field("file_len", &self.file_len)
            .field("pages", &self.pages.len())
            .field("cache_bytes", &self.cache_bytes())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests (PLAN-728 T-01..T-05 单测族；E2E/native 面见 tests/plan728_supply_probes.rs)
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::ui::code_editor::core::rope::Rope;

    /// Small pages + tight budget so tiny fixtures exercise multi-page
    /// tables, sub-page splits, and LRU eviction.
    fn test_cfg() -> PageConfig {
        PageConfig {
            page_bytes: 512,
            cache_budget: 4 * 1024,
            prefetch_radius: 2,
        }
    }

    pub(crate) struct TempFile {
        path: PathBuf,
    }

    impl TempFile {
        fn new(tag: &str, bytes: &[u8]) -> Self {
            let mut p = std::env::temp_dir();
            p.push(format!("p728-{}-{}.txt", tag, std::process::id()));
            std::fs::write(&p, bytes).unwrap();
            TempFile { path: p }
        }
        pub(crate) fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    /// Deterministic multi-line text mixing ASCII + CRLF + multi-byte chars.
    pub(crate) fn fixture(lines: usize) -> Vec<u8> {
        let mut out = String::new();
        for i in 0..lines {
            match i % 5 {
                0 => out.push_str(&format!("line {i} plain ascii\n")),
                1 => out.push_str(&format!("line {i} 中文多字节内容🦀\n")),
                2 => out.push_str(&format!("line {i} crlf\r\n")),
                3 => out.push_str(&format!("line {i} émoji 🎉 mixed\n")),
                _ => out.push_str(&format!("line {i} tab\tsep padding text here\n")),
            }
        }
        out.into_bytes()
    }

    pub(crate) fn open(bytes: &[u8], tag: &str) -> (Rope, Arc<PageStore>, TempFile) {
        let f = TempFile::new(tag, bytes);
        let (rope, store) = Rope::open_file_backed(f.path(), test_cfg()).expect("open");
        (rope, store, f)
    }

    // ── T-01 面①: page table + prescan + jump-without-materialization ──

    #[test]
    fn prescan_summaries_match_full_read() {
        let bytes = fixture(600); // ~30KB → ~60 pages
        let want_text = std::str::from_utf8(&bytes).unwrap();
        let (rope, store, _f) = open(&bytes, "prescan");
        assert_eq!(store.page_count(), bytes.len() as usize / 512 + 1);
        // Root summaries answer without ANY page resident.
        assert_eq!(store.cache_bytes(), 0, "prescan must not warm the cache");
        assert_eq!(rope.len_bytes(), bytes.len());
        assert_eq!(
            rope.line_count(),
            want_text.bytes().filter(|&b| b == b'\n').count() + 1
        );
        assert_eq!(rope.len_chars(), want_text.chars().count());
        // Whole-document digest matches the flat digest of the file — the
        // Merkle chain folds prescan page digests (zero residency).
        assert_eq!(
            rope.content_hash(),
            Rope::from_str(want_text).content_hash()
        );
    }

    #[test]
    fn page_boundaries_never_split_chars() {
        // Multi-byte soup sized to straddle page boundaries at every phase.
        let mut bytes = Vec::new();
        for i in 0..4000 {
            bytes.extend_from_slice(match i % 4 {
                0 => "中".as_bytes(),
                1 => "a".as_bytes(),
                2 => "🦀".as_bytes(),
                _ => "é".as_bytes(),
            });
            if i % 37 == 0 {
                bytes.push(b'\n');
            }
        }
        let (rope, _store, _f) = open(&bytes, "boundary");
        let text = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(
            rope.line_count(),
            text.bytes().filter(|&b| b == b'\n').count() + 1
        );
        assert_eq!(rope.to_string(), text);
    }

    #[test]
    fn prescan_rejects_invalid_utf8() {
        let mut bytes = b"abc\n".to_vec();
        bytes.extend_from_slice(&[0xFF, 0xFE, b'\n']);
        let f = TempFile::new("badutf8", &bytes);
        let err = Rope::open_file_backed(f.path(), test_cfg()).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn line_jump_far_away_bounded_fault() {
        let bytes = fixture(4000); // ~200KB → ~400 pages, budget 4KB = ~8 pages
        let want_text = std::str::from_utf8(&bytes).unwrap();
        let (rope, store, _f) = open(&bytes, "jump");
        let lines: Vec<&str> = want_text.lines().collect();
        // Jump near the end without walking anything.
        let far = lines.len() - 2;
        let start = rope.line_start_byte(far);
        // The jump answered through a BOUNDED fault set (assert before any
        // whole-document materialization below).
        assert!(
            store.cache_bytes() <= 4 * 1024 + 512,
            "cache {} over budget",
            store.cache_bytes()
        );
        assert_eq!(
            &rope.to_string()[start..start + lines[far].len()],
            lines[far]
        );
    }

    // ── T-02 面②: overlay edits — dual-track equivalence ────────────────

    /// Same op sequence on an in-memory rope and a paged rope must yield
    /// identical content/summaries/points/hashes (the AC-02 main gate).
    #[test]
    fn dual_track_edit_equivalence() {
        let mut rng = 0x728u64;
        let mut next = move || {
            rng ^= rng >> 12;
            rng ^= rng << 25;
            rng ^= rng >> 27;
            rng.wrapping_mul(0x2545F4914F6CDD1D)
        };
        let bytes = fixture(900);
        let text = std::str::from_utf8(&bytes).unwrap().to_string();
        let (mut paged, _store, _f) = open(&bytes, "dual");
        let mut mem = Rope::from_str(&text);
        let mut model = text.clone();

        let bounds = |s: &str| -> Vec<usize> {
            std::iter::once(0)
                .chain(s.char_indices().map(|(i, c)| i + c.len_utf8()))
                .collect()
        };
        for op in 0..220 {
            let b = bounds(&model);
            let at = b[(next() % b.len() as u64) as usize];
            let at_pos = b.iter().position(|&x| x == at).unwrap();
            let to = b[(at_pos + (next() as usize % (b.len() - at_pos))) % b.len()];
            let ins = match (next() % 4) % 4 {
                0 => "插\n入".to_string(),
                1 => String::new(),
                _ => "plain\n".to_string(),
            };
            let (lo, hi) = (at.min(to), at.max(to));
            model.replace_range(lo..hi, &ins);
            paged.replace_bytes(lo, hi, &ins);
            mem.replace_bytes(lo, hi, &ins);

            let ctx = format!("op {op}: replace[{lo},{hi})");
            assert_eq!(paged.to_string(), model, "paged content ({ctx})");
            assert_eq!(mem.to_string(), model, "mem content ({ctx})");
            assert_eq!(paged.len_bytes(), mem.len_bytes(), "{ctx}");
            assert_eq!(paged.len_chars(), mem.len_chars(), "{ctx}");
            assert_eq!(paged.line_count(), mem.line_count(), "{ctx}");
            assert_eq!(paged.content_hash(), mem.content_hash(), "{ctx}");
            // A random point roundtrip per op.
            let b2 = bounds(&model);
            let byte = b2[(next() % b2.len() as u64) as usize];
            assert_eq!(paged.byte_to_point(byte), mem.byte_to_point(byte), "{ctx}");
            let (l, c) = paged.byte_to_point(byte);
            assert_eq!(paged.point_to_byte(l, c), Some(byte), "{ctx}");
        }
    }

    #[test]
    fn base_immutable_across_edits() {
        // Frozen ③: editing must not touch the base — snapshots taken
        // before the edit stay frozen and the file bytes stay put.
        let bytes = fixture(300);
        let text = std::str::from_utf8(&bytes).unwrap().to_string();
        let (mut paged, store, f) = open(&bytes, "immutable");
        let snap = paged.snapshot();
        let mid = paged.len_bytes() / 2;
        let boundary = {
            let mut b = mid;
            while !paged.is_char_boundary(b) {
                b -= 1;
            }
            b
        };
        paged.insert_bytes(boundary, "// edit\n");
        paged.delete_bytes(0, 9);
        assert_eq!(snap.to_string(), text, "pre-edit snapshot frozen");
        assert_eq!(
            std::fs::read(f.path()).unwrap(),
            bytes,
            "base file untouched"
        );
        assert!(
            store.baseline_matches(),
            "baseline still matches (we never saved)"
        );
    }

    // ── T-03 面③: LRU budget ─────────────────────────────────────────────

    #[test]
    fn lru_eviction_keeps_cache_within_budget() {
        let bytes = fixture(3000); // ~150KB, 4KB budget
        let (rope, store, _f) = open(&bytes, "lru");
        // Walk the WHOLE document — every page faults once, eviction must
        // hold the resident set at the budget the whole way.
        for i in 0..rope.line_count() {
            let text = rope.line(i);
            if i % 50 == 0 {
                assert!(
                    store.cache_bytes() <= 4 * 1024 + 512,
                    "cache {} exceeds budget after line {i} ({}B)",
                    store.cache_bytes(),
                    text.len()
                );
            }
        }
        // Full materialization still byte-faithful after mass eviction.
        assert_eq!(rope.to_string(), std::str::from_utf8(&bytes).unwrap());
    }

    #[test]
    fn residency_contract_structural_estimate() {
        // AC-03 face: structural estimate = tree nodes + page table +
        // cache — all bounded by construction (real 1GB numbers land in the
        // bench谱; here the mechanics assert).
        let bytes = fixture(2000);
        let (rope, store, _f) = open(&bytes, "budget");
        let _ = rope.to_string(); // warm everything once
        let total = rope.structural_resident_estimate() + store.structural_resident_bytes();
        // Per-page cost sanity: ~100KB doc must stay far under 1MB.
        assert!(
            total < 1024 * 1024,
            "structural estimate {total} unexpectedly large for ~100KB doc"
        );
    }

    // ── T-04 面④: async prefetch + page-state observability ─────────────

    #[test]
    fn prefetch_flips_pages_resident_off_thread() {
        let bytes = fixture(1500);
        let (rope, store, _f) = open(&bytes, "prefetch");
        assert_eq!(store.page_state(3), PageState::Cold);
        store.prefetch_around(3);
        // The worker fills asynchronously — poll for residency (bounded
        // wait; a cold single-page sync fault stays the fallback).
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while store.page_state(3) == PageState::Cold && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            store.page_state(3),
            PageState::Resident,
            "prefetch must warm page 3"
        );
        assert!(store.prefetched_pages() >= 1);
        // Reads through the rope still answer (warm or cold alike).
        assert!(!rope.line(1).is_empty());
    }

    // ── T-05 面⑤: merge save round-trips byte-for-byte ──────────────────

    fn save_roundtrip(tag: &str, bytes: Vec<u8>, edits: &[(usize, usize, &str)]) {
        let text = String::from_utf8(bytes.clone()).expect("valid utf8");
        let (mut rope, _store, f) = open(&bytes, tag);
        let mut model = text.clone();
        for &(start, end, rep) in edits {
            let mut s = start;
            let mut e = end.min(model.len());
            while !model.is_char_boundary(s) {
                s -= 1;
            }
            while e < model.len() && !model.is_char_boundary(e) {
                e += 1;
            }
            model.replace_range(s..e, rep);
            rope.replace_bytes(s, e, rep);
        }
        let out = f.path().with_extension("out");
        rope.write_backed(&out).expect("write_backed");
        let written = std::fs::read(&out).expect("read back");
        assert_eq!(
            written,
            model.into_bytes(),
            "round-trip must be byte-for-byte ({tag})"
        );
        // No temp litter around the destination.
        let stem = out.file_name().unwrap().to_string_lossy().into_owned();
        let dir_litter = std::fs::read_dir(f.path().parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name().to_string_lossy().into_owned();
                n.starts_with(&stem) && n.contains("p728tmp")
            })
            .count();
        assert_eq!(
            dir_litter, 0,
            "temp files must not survive a successful save ({tag})"
        );
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn save_roundtrip_families() {
        // Pure ASCII, no edits (pure base copy).
        save_roundtrip("rt-ascii", fixture(700), &[]);
        // CRLF preserved byte-for-byte (EOL interpretation stays downstream).
        save_roundtrip("rt-crlf", b"a\r\nb\r\nc\r\n".to_vec(), &[]);
        // BOM rides as raw bytes (interpretation stays downstream).
        save_roundtrip(
            "rt-bom",
            format!("\u{FEFF}bom head\nbody\n").into_bytes(),
            &[],
        );
        // Multi-span edits: head insert + middle replace + tail delete.
        save_roundtrip(
            "rt-edits",
            fixture(800),
            &[
                (0, 0, "// header\n"),
                (20_000, 20_050, "替换\n"),
                (61_000, 400_000, ""),
            ],
        );
        // Edits that delete across page boundaries entirely.
        save_roundtrip("rt-bigdel", fixture(500), &[(600, 60_000, "x")]);
    }

    #[test]
    fn save_rejects_external_mutation_and_leaves_original_intact() {
        let bytes = fixture(400);
        let (rope, _store, f) = open(&bytes, "ext");
        // External write behind the rope's back (len changes).
        std::fs::write(f.path(), b"externally replaced\n").unwrap();
        let out = f.path().with_extension("out");
        let err = rope.write_backed(&out).unwrap_err();
        assert!(err.to_string().contains("external modification"), "{err}");
        // The refused save must not produce output; the "original" is now
        // the externally-written bytes (the save refused BEFORE any write).
        assert!(!out.exists(), "refused save must not produce output");
        assert_eq!(std::fs::read(f.path()).unwrap(), b"externally replaced\n");
    }

    #[test]
    fn save_refreshes_baseline_for_next_save() {
        let bytes = fixture(400);
        let (mut rope, store, f) = open(&bytes, "rebase");
        rope.insert_bytes(0, "// v2\n");
        rope.write_backed(f.path()).expect("first save (in place)");
        assert_eq!(
            std::fs::read(f.path()).unwrap(),
            rope.to_string().as_bytes()
        );
        assert!(
            store.baseline_matches(),
            "baseline re-stamped after own save"
        );
        // A second in-place save right after must succeed (fresh baseline).
        rope.insert_bytes(0, "// v3\n");
        rope.write_backed(f.path())
            .expect("second save after refresh");
        assert_eq!(
            std::fs::read(f.path()).unwrap(),
            rope.to_string().as_bytes()
        );
    }

    #[test]
    fn save_new_destination_needs_no_baseline() {
        // Saving to a DIFFERENT path never consults the base baseline.
        let bytes = fixture(200);
        let (rope, _store, f) = open(&bytes, "copydest");
        let out = f.path().with_extension("elsewhere");
        rope.write_backed(&out).expect("save to fresh destination");
        assert_eq!(std::fs::read(&out).unwrap(), bytes);
        let _ = std::fs::remove_file(&out);
    }

    // ── T-06 面⑥: snapshot/digest consumers over paged ropes ────────────

    #[test]
    fn paged_snapshots_frozen_and_merkle_compatible() {
        let bytes = fixture(700);
        let text = std::str::from_utf8(&bytes).unwrap().to_string();
        let (mut paged, _store, _f) = open(&bytes, "snap");
        let mem = Rope::from_str(&text);

        let snap0 = paged.snapshot();
        assert!(
            snap0.subtree_equal(&mem.snapshot()),
            "paged vs mem Merkle-equal pre-edit"
        );
        assert_eq!(snap0.content_hash(), mem.content_hash());

        let edit_at = paged.line_start_byte(200);
        paged.insert_bytes(edit_at, "// diverged\n");
        assert!(!paged.snapshot().subtree_equal(&snap0), "fork diverges");
        assert_eq!(snap0.to_string(), text, "frozen pre-edit view");

        // Prune spans classify the edit region diverged (diff preprocessing
        // works over paged trees with page-digest shortcuts).
        let spans = paged.snapshot().prune_spans(&snap0);
        assert!(
            spans.diverged.iter().any(|&(a1, a2, _, _)| a1 <= edit_at
                && edit_at
                    + "// diverged
"
                    .len()
                    <= a2),
            "edit region must register diverged: {:?}",
            spans.diverged
        );

        // Revert: Merkle-equal again (content-determined digests).
        paged.delete_bytes(
            edit_at,
            edit_at
                + "// diverged
"
                .len(),
        );
        assert!(
            paged.snapshot().subtree_equal(&mem.snapshot()),
            "revert restores equality"
        );
        let (lo, hi) = (mem.line_start_byte(5), mem.line_start_byte(30));
        assert_eq!(paged.range_hash(lo, hi), mem.range_hash(lo, hi));
    }
}
