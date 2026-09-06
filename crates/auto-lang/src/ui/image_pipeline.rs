//! Shared, renderer-neutral image asset pipeline.
//!
//! The public media types, bounded scheduler, cache registry, and HTTP helper
//! live here so VM, generated Rust, and an HTTP backend use identical asset
//! identities and lifetimes.  Task 1 deliberately provides only the module
//! seam; its types and behavior are added in the following tasks.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

/// Opaque identifier used in public media URIs.  It deliberately has no path
/// representation: callers can only address an asset ticket issued by the
/// registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MediaAssetId(pub(crate) u128);

impl fmt::Display for MediaAssetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:032x}", self.0)
    }
}

/// A complete rendition identity.  A source change or display transform must
/// never collide with a previously published media URI.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MediaAssetKey {
    pub source_fingerprint: String,
    pub orientation: MediaOrientation,
    pub rendition: RenditionSpec,
    pub revision: u64,
}

/// EXIF orientation normalized before a rendition is published.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MediaOrientation {
    #[default]
    Normal,
    FlipHorizontal,
    Rotate180,
    FlipVertical,
    Transpose,
    Rotate90,
    Transverse,
    Rotate270,
}

/// Lifecycle of one registry entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MediaAssetState {
    #[default]
    Queued,
    Reading,
    Decoding,
    Transforming,
    Ready,
    Error,
    Stale,
    Expired,
}

impl MediaAssetState {
    /// Returns whether a lifecycle edge is valid.  The registry owns applying
    /// it so transitions remain observable and terminal entries cannot be
    /// resurrected accidentally.
    pub const fn can_transition_to(self, next: Self) -> bool {
        use MediaAssetState::*;

        match (self, next) {
            (Queued, Reading | Error | Stale | Expired)
            | (Reading, Decoding | Error | Stale | Expired)
            | (Decoding, Transforming | Error | Stale | Expired)
            | (Transforming, Ready | Error | Stale | Expired)
            | (Ready, Expired)
            | (Error, Expired)
            | (Stale, Expired) => true,
            _ => false,
        }
    }

    /// Ready, error, and expired states require no worker progress.
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Ready | Self::Error | Self::Stale | Self::Expired)
    }
}

/// Requested output pixels and transforms.  Dimensions are integers so this
/// type is a stable cache key across VM and generated-Rust callers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenditionSpec {
    pub width: u32,
    pub height: u32,
    pub rotation_degrees: i16,
    pub quality: u8,
    pub original_pixels: bool,
}

impl RenditionSpec {
    pub const fn viewport(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            rotation_degrees: 0,
            quality: 90,
            original_pixels: false,
        }
    }

    pub const fn original() -> Self {
        Self {
            width: 0,
            height: 0,
            rotation_degrees: 0,
            quality: 100,
            original_pixels: true,
        }
    }
}

/// Header information safe to return through the JSON control plane.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MediaMetadata {
    pub width: u32,
    pub height: u32,
    pub orientation: MediaOrientation,
    pub mime_type: String,
    pub byte_len: u64,
}

/// Stable machine-readable media failures.  Human-readable details remain
/// local to the registry and are never interpolated into a URI or log route.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MediaAssetError {
    #[error("unsupported image format")]
    UnsupportedFormat,
    #[error("image file exceeds configured size limit")]
    FileTooLarge,
    #[error("image dimensions exceed configured pixel limit")]
    PixelLimitExceeded,
    #[error("image dimensions overflow the pixel buffer")]
    SizeOverflow,
    #[error("image data is corrupt")]
    Corrupt,
    #[error("image asset is stale")]
    Stale,
    #[error("image asset has expired")]
    Expired,
    #[error("image asset is not ready")]
    NotReady,
    #[error("image operation was rejected: {0}")]
    Rejected(String),
}

/// Snapshot counters shared by the control plane and performance harness.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MediaPipelineStats {
    pub queued: usize,
    pub running: usize,
    pub completed: u64,
    pub dropped: u64,
    pub encoded_bytes: usize,
    pub decoded_bytes: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
}

/// A registry-issued ticket.  Only this opaque pair may be used to address a
/// media datum; source paths remain private to the service that queued it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MediaAssetTicket {
    pub id: MediaAssetId,
    pub revision: u64,
}

/// Read result used by both the native URI resolver and the HTTP data plane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaLookup {
    Missing,
    Expired,
    Pending,
    Failed(MediaAssetError),
    Ready(Arc<[u8]>),
}

#[derive(Debug)]
struct MediaAssetEntry {
    key: MediaAssetKey,
    metadata: MediaMetadata,
    state: MediaAssetState,
    encoded: Option<Arc<[u8]>>,
    error: Option<MediaAssetError>,
    references: usize,
    expires_at: Option<Instant>,
    created_at: Instant,
}

#[derive(Default)]
struct RegistryState {
    entries: HashMap<MediaAssetId, MediaAssetEntry>,
    by_key: HashMap<MediaAssetKey, MediaAssetId>,
    shutting_down: bool,
}

struct RegistryInner {
    state: Mutex<RegistryState>,
    changed: Condvar,
    ttl: Duration,
}

/// Process-local media asset registry shared by the HTTP adapter, VM, and
/// generated Rust UI.  It owns byte lifetimes and is intentionally unaware of
/// filesystem paths and decode workers.
#[derive(Clone)]
pub struct MediaAssetRegistry {
    inner: Arc<RegistryInner>,
}

/// Process-wide registry shared by generated servers and native renderers.
/// It exposes opaque tickets only; no source path enters the HTTP-facing API.
pub fn global_media_registry() -> &'static MediaAssetRegistry {
    static REGISTRY: std::sync::LazyLock<MediaAssetRegistry> =
        std::sync::LazyLock::new(|| MediaAssetRegistry::new(Duration::from_secs(30)));
    &REGISTRY
}

impl MediaAssetRegistry {
    pub const DEFAULT_TTL: Duration = Duration::from_secs(2);
    pub const MAX_ENTRIES: usize = 512;

    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RegistryInner {
                state: Mutex::new(RegistryState::default()),
                changed: Condvar::new(),
                ttl,
            }),
        }
    }

    pub fn queue(&self, key: MediaAssetKey, metadata: MediaMetadata) -> MediaAssetTicket {
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        if let Some(existing_id) = state.by_key.get(&key).copied() {
            if let Some(existing) = state.entries.get_mut(&existing_id) {
                if existing.state != MediaAssetState::Expired {
                    existing.references = existing.references.saturating_add(1);
                    existing.expires_at = None;
                    return MediaAssetTicket {
                        id: existing_id,
                        revision: existing.key.revision,
                    };
                }
            }
        }

        // Keep the process-wide registry bounded even when a caller forgets to
        // release a short-lived ticket. Only unreferenced entries are eligible;
        // active sessions retain their tickets and are never evicted here.
        if state.entries.len() >= Self::MAX_ENTRIES {
            let victim = state
                .entries
                .iter()
                .filter(|(_, entry)| entry.references == 0)
                .min_by_key(|(_, entry)| entry.created_at)
                .map(|(&id, _)| id);
            if let Some(victim) = victim {
                if let Some(entry) = state.entries.remove(&victim) {
                    state.by_key.remove(&entry.key);
                }
            }
        }

        let id = loop {
            let candidate = MediaAssetId(
                ((fastrand::u64(..) as u128) << 64) | fastrand::u64(..) as u128,
            );
            if !state.entries.contains_key(&candidate) {
                break candidate;
            }
        };
        let revision = key.revision;
        state.by_key.insert(key.clone(), id);
        state.entries.insert(
            id,
            MediaAssetEntry {
                key,
                metadata,
                state: MediaAssetState::Queued,
                encoded: None,
                error: None,
                references: 1,
                expires_at: None,
                created_at: Instant::now(),
            },
        );
        self.inner.changed.notify_all();
        MediaAssetTicket { id, revision }
    }

    pub fn transition(
        &self,
        id: MediaAssetId,
        next: MediaAssetState,
    ) -> Result<(), MediaAssetError> {
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        let entry = state
            .entries
            .get_mut(&id)
            .ok_or(MediaAssetError::Expired)?;
        if !entry.state.can_transition_to(next) {
            return Err(MediaAssetError::Rejected("invalid media state transition".into()));
        }
        entry.state = next;
        if next == MediaAssetState::Error && entry.error.is_none() {
            entry.error = Some(MediaAssetError::Corrupt);
        }
        self.inner.changed.notify_all();
        Ok(())
    }

    pub fn publish_ready(
        &self,
        id: MediaAssetId,
        revision: u64,
        bytes: Arc<[u8]>,
    ) -> Result<(), MediaAssetError> {
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        let entry = state
            .entries
            .get_mut(&id)
            .ok_or(MediaAssetError::Expired)?;
        if entry.key.revision != revision {
            return Err(MediaAssetError::Stale);
        }
        if entry.state != MediaAssetState::Transforming {
            return Err(MediaAssetError::Rejected("asset is not ready to publish".into()));
        }
        entry.encoded = Some(bytes);
        entry.state = MediaAssetState::Ready;
        self.inner.changed.notify_all();
        Ok(())
    }

    pub fn fail(&self, id: MediaAssetId, error: MediaAssetError) -> Result<(), MediaAssetError> {
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        let entry = state
            .entries
            .get_mut(&id)
            .ok_or(MediaAssetError::Expired)?;
        if !entry.state.can_transition_to(MediaAssetState::Error) {
            return Err(MediaAssetError::Rejected("asset cannot fail from its current state".into()));
        }
        entry.state = MediaAssetState::Error;
        entry.error = Some(error);
        self.inner.changed.notify_all();
        Ok(())
    }

    pub fn lookup(&self, id: MediaAssetId, revision: u64) -> MediaLookup {
        let state = self.inner.state.lock().expect("media registry lock poisoned");
        let Some(entry) = state.entries.get(&id) else {
            return MediaLookup::Missing;
        };
        if entry.key.revision != revision {
            return MediaLookup::Missing;
        }
        match entry.state {
            MediaAssetState::Ready => entry
                .encoded
                .as_ref()
                .cloned()
                .map(MediaLookup::Ready)
                .unwrap_or(MediaLookup::Pending),
            MediaAssetState::Error => MediaLookup::Failed(
                entry.error.clone().unwrap_or(MediaAssetError::Corrupt),
            ),
            MediaAssetState::Expired | MediaAssetState::Stale => MediaLookup::Expired,
            _ => MediaLookup::Pending,
        }
    }

    pub fn metadata(&self, id: MediaAssetId) -> Option<MediaMetadata> {
        self.inner
            .state
            .lock()
            .expect("media registry lock poisoned")
            .entries
            .get(&id)
            .map(|entry| entry.metadata.clone())
    }

    /// Updates metadata discovered by a background reader. The source path is
    /// never stored in the metadata returned to HTTP/control-plane callers.
    pub fn update_metadata(&self, id: MediaAssetId, metadata: MediaMetadata) -> bool {
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        let Some(entry) = state.entries.get_mut(&id) else { return false; };
        entry.metadata = metadata;
        self.inner.changed.notify_all();
        true
    }

    pub fn stats(&self) -> MediaPipelineStats {
        let state = self.inner.state.lock().expect("media registry lock poisoned");
        let mut stats = MediaPipelineStats::default();
        for entry in state.entries.values() {
            match entry.state {
                MediaAssetState::Queued => stats.queued += 1,
                MediaAssetState::Reading
                | MediaAssetState::Decoding
                | MediaAssetState::Transforming => stats.running += 1,
                MediaAssetState::Ready => stats.completed += 1,
                _ => {}
            }
            if let Some(bytes) = &entry.encoded {
                stats.encoded_bytes += bytes.len();
            }
        }
        stats
    }

    pub fn retain(&self, id: MediaAssetId) -> bool {
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        let Some(entry) = state.entries.get_mut(&id) else {
            return false;
        };
        if entry.state == MediaAssetState::Expired {
            return false;
        }
        entry.references = entry.references.saturating_add(1);
        entry.expires_at = None;
        true
    }

    pub fn release(&self, id: MediaAssetId) {
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        if let Some(entry) = state.entries.get_mut(&id) {
            entry.references = entry.references.saturating_sub(1);
            if entry.references == 0 {
                entry.expires_at = Some(Instant::now() + self.inner.ttl);
            }
        }
    }

    /// Converts unreferenced, elapsed tickets to 410-visible expired entries.
    pub fn collect_expired(&self) {
        let now = Instant::now();
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        for entry in state.entries.values_mut() {
            if entry.references == 0
                && entry.expires_at.is_some_and(|expiry| expiry <= now)
                && entry.state != MediaAssetState::Expired
            {
                entry.encoded = None;
                entry.state = MediaAssetState::Expired;
            }
        }
        self.inner.changed.notify_all();
    }

    pub fn wait_for_terminal(&self, id: MediaAssetId, timeout: Duration) -> Option<MediaAssetState> {
        let deadline = Instant::now() + timeout;
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        loop {
            let entry = state.entries.get(&id)?;
            if entry.state.is_terminal() {
                return Some(entry.state);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }
            let (next_state, _) = self
                .inner
                .changed
                .wait_timeout(state, remaining)
                .expect("media registry lock poisoned");
            state = next_state;
        }
    }

    pub fn shutdown(&self) {
        let now = Instant::now();
        let mut state = self.inner.state.lock().expect("media registry lock poisoned");
        state.shutting_down = true;
        for entry in state.entries.values_mut() {
            entry.references = 0;
            entry.expires_at = Some(now);
            if !entry.state.is_terminal() {
                entry.state = MediaAssetState::Stale;
            }
        }
        self.inner.changed.notify_all();
    }

    pub fn is_shutdown(&self) -> bool {
        self.inner
            .state
            .lock()
            .expect("media registry lock poisoned")
            .shutting_down
    }
}

impl Default for MediaAssetRegistry {
    fn default() -> Self {
        Self::new(Self::DEFAULT_TTL)
    }
}

#[derive(Clone, Debug)]
struct EncodedLruEntry {
    bytes: Arc<[u8]>,
    last_access: u64,
}

/// Byte-accounted encoded rendition cache.  This deliberately owns no global
/// static state: its owner decides lifecycle and drops it with the registry.
#[derive(Clone, Debug)]
pub struct EncodedByteLru {
    budget_bytes: usize,
    used_bytes: usize,
    access_clock: u64,
    entries: HashMap<MediaAssetId, EncodedLruEntry>,
    stats: MediaPipelineStats,
}

impl EncodedByteLru {
    pub const DEFAULT_BUDGET_BYTES: usize = 64 * 1024 * 1024;

    pub fn new(budget_bytes: usize) -> Self {
        Self {
            budget_bytes,
            used_bytes: 0,
            access_clock: 0,
            entries: HashMap::new(),
            stats: MediaPipelineStats::default(),
        }
    }

    pub fn insert(&mut self, id: MediaAssetId, bytes: Arc<[u8]>) {
        self.access_clock = self.access_clock.wrapping_add(1);
        if let Some(previous) = self.entries.remove(&id) {
            self.used_bytes = self.used_bytes.saturating_sub(previous.bytes.len());
        }
        self.used_bytes = self.used_bytes.saturating_add(bytes.len());
        self.entries.insert(
            id,
            EncodedLruEntry {
                bytes,
                last_access: self.access_clock,
            },
        );
        self.evict_to_budget();
    }

    pub fn get(&mut self, id: MediaAssetId) -> Option<Arc<[u8]>> {
        self.access_clock = self.access_clock.wrapping_add(1);
        match self.entries.get_mut(&id) {
            Some(entry) => {
                entry.last_access = self.access_clock;
                self.stats.cache_hits = self.stats.cache_hits.saturating_add(1);
                Some(entry.bytes.clone())
            }
            None => {
                self.stats.cache_misses = self.stats.cache_misses.saturating_add(1);
                None
            }
        }
    }

    pub const fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    pub const fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    pub fn stats(&self) -> &MediaPipelineStats {
        &self.stats
    }

    fn evict_to_budget(&mut self) {
        while self.used_bytes > self.budget_bytes {
            let Some((&oldest_id, _)) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_access)
            else {
                break;
            };
            let removed = self.entries.remove(&oldest_id).expect("LRU entry exists");
            self.used_bytes = self.used_bytes.saturating_sub(removed.bytes.len());
            self.stats.evictions = self.stats.evictions.saturating_add(1);
        }
        self.stats.encoded_bytes = self.used_bytes;
    }
}

impl Default for EncodedByteLru {
    fn default() -> Self {
        Self::new(Self::DEFAULT_BUDGET_BYTES)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MediaPin { #[default] None, Soft, Hard }
#[derive(Clone, Debug)]
struct DecodedEntry {
    width: u32,
    height: u32,
    bytes: Arc<[u8]>,
    pin: MediaPin,
    last_access: u64,
}
/// Bounded host-pixel cache. Hard-pinned current content may exceed budget;
/// all non-hard entries are reclaimed as soon as a publish changes the set.
#[derive(Clone, Debug)]
pub struct DecodedPixelCache {
    budget_bytes: usize, used_bytes: usize, clock: u64,
    entries: HashMap<MediaAssetId, DecodedEntry>, stats: MediaPipelineStats,
}
impl DecodedPixelCache {
    pub const DEFAULT_BUDGET_BYTES: usize = 256 * 1024 * 1024;
    pub fn new(budget_bytes: usize) -> Self { Self { budget_bytes, used_bytes: 0, clock: 0, entries: HashMap::new(), stats: MediaPipelineStats::default() } }
    pub fn insert(&mut self, id: MediaAssetId, bytes: Arc<[u8]>, pin: MediaPin) {
        self.insert_rendered(id, 0, 0, bytes, pin);
    }
    pub fn insert_rendered(&mut self, id: MediaAssetId, width: u32, height: u32, bytes: Arc<[u8]>, pin: MediaPin) {
        self.clock = self.clock.wrapping_add(1);
        if let Some(old) = self.entries.remove(&id) { self.used_bytes = self.used_bytes.saturating_sub(old.bytes.len()); }
        self.used_bytes = self.used_bytes.saturating_add(bytes.len());
        self.entries.insert(id, DecodedEntry { width, height, bytes, pin, last_access: self.clock }); self.collect();
    }
    pub fn set_pin(&mut self, id: MediaAssetId, pin: MediaPin) { if let Some(entry) = self.entries.get_mut(&id) { entry.pin = pin; } }
    pub fn contains(&self, id: MediaAssetId) -> bool { self.entries.contains_key(&id) }
    pub fn get_rendered(&mut self, id: MediaAssetId) -> Option<(u32, u32, Arc<[u8]>)> {
        self.clock = self.clock.wrapping_add(1);
        let entry = self.entries.get_mut(&id)?;
        entry.last_access = self.clock;
        self.stats.cache_hits = self.stats.cache_hits.saturating_add(1);
        Some((entry.width, entry.height, entry.bytes.clone()))
    }
    pub const fn used_bytes(&self) -> usize { self.used_bytes }
    pub fn stats(&self) -> &MediaPipelineStats { &self.stats }
    pub fn collect(&mut self) {
        while self.used_bytes > self.budget_bytes {
            let candidate = self.entries.iter().filter(|(_, e)| e.pin != MediaPin::Hard).min_by_key(|(_, e)| (e.pin != MediaPin::None, e.last_access)).map(|(&id, _)| id);
            let Some(id) = candidate else { break; };
            let entry = self.entries.remove(&id).expect("decoded cache entry exists"); self.used_bytes = self.used_bytes.saturating_sub(entry.bytes.len()); self.stats.evictions = self.stats.evictions.saturating_add(1);
        }
        self.stats.decoded_bytes = self.used_bytes;
    }
}
impl Default for DecodedPixelCache { fn default() -> Self { Self::new(Self::DEFAULT_BUDGET_BYTES) } }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MediaPriority { Thumbnail = 5, Neighbor = 10, SettledCurrent = 90, Current = 100 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaWork { pub id: u64, pub priority: MediaPriority, sequence: u64 }
impl MediaWork { pub const fn new(id: u64, priority: MediaPriority) -> Self { Self { id, priority, sequence: 0 } } }
/// Scheduler ingress queue; the worker pool consumes `pop` in priority then
/// FIFO order.  Duplicate renditions are coalesced before consuming capacity.
#[derive(Default)]
pub struct MediaPriorityQueue { items: Vec<MediaWork>, next_sequence: u64 }
impl MediaPriorityQueue {
    pub const CAPACITY: usize = 8;
    pub fn new() -> Self { Self::default() }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn push(&mut self, mut work: MediaWork) -> bool {
        if self.items.iter().any(|item| item.id == work.id) { return false; }
        if work.priority == MediaPriority::Neighbor && self.items.iter().filter(|item| item.priority == MediaPriority::Neighbor).count() >= 2 { return false; }
        if self.items.len() >= Self::CAPACITY {
            let lowest = self.items.last().map(|item| item.priority).unwrap_or(MediaPriority::Thumbnail);
            if work.priority <= lowest { return false; }
        }
        let work_id = work.id;
        work.sequence = self.next_sequence; self.next_sequence = self.next_sequence.wrapping_add(1); self.items.push(work);
        self.items.sort_by_key(|item| (std::cmp::Reverse(item.priority), item.sequence));
        if self.items.len() > Self::CAPACITY { self.items.pop(); }
        self.items.iter().any(|item| item.id == work_id)
    }
    pub fn pop(&mut self) -> Option<MediaWork> { (!self.items.is_empty()).then(|| self.items.remove(0)) }
}

/// Per-session generation/revision gate shared by each asynchronous phase.
#[derive(Clone, Debug)]
pub struct MediaLatestWins { generation: u64, view_revision: u64, dropped: u64 }
impl MediaLatestWins {
    pub const fn new(generation: u64, view_revision: u64) -> Self { Self { generation, view_revision, dropped: 0 } }
    pub fn advance(&mut self, generation: u64, view_revision: u64) { self.generation = generation; self.view_revision = view_revision; }
    fn accept(&mut self, generation: u64, view_revision: u64) -> bool { let ok = (generation, view_revision) == (self.generation, self.view_revision); if !ok { self.dropped = self.dropped.saturating_add(1); } ok }
    pub fn accept_enqueue(&mut self, generation: u64, view_revision: u64) -> bool { self.accept(generation, view_revision) }
    pub fn accept_decode_complete(&mut self, generation: u64, view_revision: u64) -> bool { self.accept(generation, view_revision) }
    pub fn accept_publish(&mut self, generation: u64, view_revision: u64) -> bool { self.accept(generation, view_revision) }
    pub const fn dropped(&self) -> u64 { self.dropped }
}

#[derive(Clone, Debug)]
enum MediaSource { Path(PathBuf), Bytes(Arc<[u8]>) }

#[derive(Clone, Debug)]
struct MediaWorkItem {
    id: u64,
    ticket: MediaAssetTicket,
    source: MediaSource,
    spec: RenditionSpec,
    priority: MediaPriority,
    generation: u64,
    view_revision: u64,
}

#[derive(Debug)]
struct DecodedWork {
    item: MediaWorkItem,
    encoded: Arc<[u8]>,
    metadata: MediaMetadata,
    image: image::DynamicImage,
}

enum ResizeMessage { Work(DecodedWork), Stop }

struct PipelineCaches {
    encoded: EncodedByteLru,
    decoded: DecodedPixelCache,
}

struct WorkerState {
    stopping: bool,
    queue: MediaPriorityQueue,
    jobs: HashMap<u64, MediaWorkItem>,
}

struct WorkerInner {
    state: Mutex<WorkerState>,
    wake: Condvar,
    registry: MediaAssetRegistry,
    resize_tx: SyncSender<ResizeMessage>,
    caches: Mutex<PipelineCaches>,
    latest: Mutex<HashMap<MediaAssetId, MediaLatestWins>>,
}

static NEXT_MEDIA_WORK_ID: AtomicU64 = AtomicU64::new(1);

/// Bounded media executor: two decode/read workers feed one resize/publish
/// lane. Workers sleep on condition variables or channel receives while idle;
/// no UI call performs filesystem I/O or image decoding.
pub struct MediaWorkerPool { inner: Arc<WorkerInner>, workers: Vec<JoinHandle<()>> }

impl MediaWorkerPool {
    pub fn new() -> Self { Self::with_registry(MediaAssetRegistry::default()) }

    pub fn with_registry(registry: MediaAssetRegistry) -> Self {
        let (resize_tx, resize_rx) = sync_channel(8);
        let inner = Arc::new(WorkerInner {
            state: Mutex::new(WorkerState { stopping: false, queue: MediaPriorityQueue::new(), jobs: HashMap::new() }),
            wake: Condvar::new(),
            registry,
            resize_tx,
            caches: Mutex::new(PipelineCaches { encoded: EncodedByteLru::default(), decoded: DecodedPixelCache::default() }),
            latest: Mutex::new(HashMap::new()),
        });
        let resize_inner = inner.clone();
        let resize_thread = thread::Builder::new().name("media-resize".into()).spawn(move || {
            run_resize_worker(resize_inner, resize_rx);
        }).expect("spawn media resize worker");
        let mut workers = vec![resize_thread];
        for name in ["media-decode-1", "media-decode-2"] {
            let worker_inner = inner.clone();
            workers.push(thread::Builder::new().name(name.into()).spawn(move || {
                run_decode_worker(worker_inner);
            }).expect("spawn media decode worker"));
        }
        Self { inner, workers }
    }

    pub fn worker_count(&self) -> usize { self.workers.len() }

    pub fn submit_path(&self, ticket: MediaAssetTicket, path: PathBuf, spec: RenditionSpec, priority: MediaPriority, generation: u64, view_revision: u64) -> bool {
        self.submit(MediaWorkItem { id: NEXT_MEDIA_WORK_ID.fetch_add(1, Ordering::Relaxed), ticket, source: MediaSource::Path(path), spec, priority, generation, view_revision })
    }

    pub fn submit_bytes(&self, ticket: MediaAssetTicket, bytes: Arc<[u8]>, spec: RenditionSpec, priority: MediaPriority, generation: u64, view_revision: u64) -> bool {
        self.submit(MediaWorkItem { id: NEXT_MEDIA_WORK_ID.fetch_add(1, Ordering::Relaxed), ticket, source: MediaSource::Bytes(bytes), spec, priority, generation, view_revision })
    }

    fn submit(&self, item: MediaWorkItem) -> bool {
        if let Ok(mut latest) = self.inner.latest.lock() {
            if let Some(gate) = latest.get_mut(&item.ticket.id) {
                gate.advance(item.generation, item.view_revision);
            } else {
                latest.insert(item.ticket.id, MediaLatestWins::new(item.generation, item.view_revision));
            }
        }
        let mut state = self.inner.state.lock().expect("media worker lock poisoned");
        if state.stopping || !state.queue.push(MediaWork::new(item.id, item.priority)) { return false; }
        state.jobs.insert(item.id, item);
        self.inner.wake.notify_one();
        true
    }

    pub fn decoded_pixels(&self, id: MediaAssetId) -> Option<(u32, u32, Arc<[u8]>)> {
        self.inner.caches.lock().ok()?.decoded.get_rendered(id)
    }

    pub fn shutdown(&mut self) {
        {
            let mut state = self.inner.state.lock().expect("media worker lock poisoned");
            state.stopping = true;
            state.queue.items.clear();
            state.jobs.clear();
            self.inner.wake.notify_all();
        }
        let _ = self.inner.resize_tx.send(ResizeMessage::Stop);
        for worker in self.workers.drain(..) { let _ = worker.join(); }
    }
}

fn take_media_work(inner: &Arc<WorkerInner>) -> Option<MediaWorkItem> {
    let mut state = inner.state.lock().expect("media worker lock poisoned");
    loop {
        if state.stopping { return None; }
        if let Some(work) = state.queue.pop() { return state.jobs.remove(&work.id); }
        state = inner.wake.wait(state).expect("media worker lock poisoned");
    }
}

fn run_decode_worker(inner: Arc<WorkerInner>) {
    while let Some(item) = take_media_work(&inner) {
        let item_id = item.ticket.id;
        if let Ok(mut latest) = inner.latest.lock() {
            if !latest.get_mut(&item.ticket.id).is_some_and(|gate| gate.accept_decode_complete(item.generation, item.view_revision)) {
                let _ = inner.registry.transition(item.ticket.id, MediaAssetState::Stale);
                continue;
            }
        }
        let result = (|| {
            inner.registry.transition(item.ticket.id, MediaAssetState::Reading)?;
            let encoded: Arc<[u8]> = match &item.source {
                MediaSource::Path(path) => Arc::from(std::fs::read(path).map_err(|_| MediaAssetError::Rejected("image read failed".into()))?),
                MediaSource::Bytes(bytes) => bytes.clone(),
            };
            let metadata = inspect_image_metadata(&encoded)?;
            inner.registry.update_metadata(item.ticket.id, metadata.clone());
            inner.registry.transition(item.ticket.id, MediaAssetState::Decoding)?;
            let image = image::load_from_memory(&encoded).map_err(|_| MediaAssetError::Corrupt)?;
            Ok::<_, MediaAssetError>(DecodedWork { item, encoded, metadata, image })
        })();
        match result {
            Ok(decoded) => {
                if inner.resize_tx.send(ResizeMessage::Work(decoded)).is_err() { return; }
            }
            Err(error) => { let _ = inner.registry.fail(item_id, error); }
        }
    }
}

fn oriented_image(image: image::DynamicImage, orientation: MediaOrientation) -> image::DynamicImage {
    match orientation { MediaOrientation::Normal => image, MediaOrientation::FlipHorizontal => image.fliph(), MediaOrientation::Rotate180 => image.rotate180(), MediaOrientation::FlipVertical => image.flipv(), MediaOrientation::Transpose => image.rotate90().fliph(), MediaOrientation::Rotate90 => image.rotate90(), MediaOrientation::Transverse => image.rotate270().fliph(), MediaOrientation::Rotate270 => image.rotate270() }
}

fn render_decoded(image: image::DynamicImage, orientation: MediaOrientation, spec: &RenditionSpec) -> RenderedImage {
    let oriented = oriented_image(image, orientation);
    let rendered = if spec.original_pixels || spec.width == 0 || spec.height == 0 { oriented } else { oriented.resize(spec.width, spec.height, image::imageops::FilterType::Lanczos3) };
    let rgba = rendered.to_rgba8();
    let (width, height) = rgba.dimensions();
    RenderedImage { width, height, rgba: Arc::from(rgba.into_raw()) }
}

fn run_resize_worker(inner: Arc<WorkerInner>, resize_rx: Receiver<ResizeMessage>) {
    while let Ok(message) = resize_rx.recv() {
        let ResizeMessage::Work(work) = message else { break; };
        if let Ok(mut latest) = inner.latest.lock() {
            if !latest.get_mut(&work.item.ticket.id).is_some_and(|gate| gate.accept_publish(work.item.generation, work.item.view_revision)) {
                let _ = inner.registry.transition(work.item.ticket.id, MediaAssetState::Stale);
                continue;
            }
        }
        if inner.registry.transition(work.item.ticket.id, MediaAssetState::Transforming).is_err() { continue; }
        let rendered = render_decoded(work.image, work.metadata.orientation, &work.item.spec);
        let pin = match work.item.priority { MediaPriority::Current | MediaPriority::SettledCurrent => MediaPin::Hard, MediaPriority::Neighbor => MediaPin::Soft, MediaPriority::Thumbnail => MediaPin::None };
        if let Ok(mut caches) = inner.caches.lock() {
            caches.encoded.insert(work.item.ticket.id, work.encoded.clone());
            caches.decoded.insert_rendered(work.item.ticket.id, rendered.width, rendered.height, rendered.rgba, pin);
        }
        let _ = inner.registry.publish_ready(work.item.ticket.id, work.item.ticket.revision, work.encoded);
    }
}

pub fn global_media_worker_pool() -> &'static MediaWorkerPool {
    static POOL: std::sync::LazyLock<MediaWorkerPool> = std::sync::LazyLock::new(|| MediaWorkerPool::with_registry(global_media_registry().clone()));
    &POOL
}

pub fn queue_media_path(path: impl Into<PathBuf>, priority: MediaPriority) -> MediaAssetTicket {
    let path = path.into();
    let key = MediaAssetKey { source_fingerprint: path.to_string_lossy().into_owned(), orientation: MediaOrientation::Normal, rendition: RenditionSpec::original(), revision: 1 };
    let ticket = global_media_registry().queue(key, MediaMetadata::default());
    if !global_media_worker_pool().submit_path(ticket, path, RenditionSpec::original(), priority, 0, 1) {
        let _ = global_media_registry().fail(ticket.id, MediaAssetError::Rejected("media queue is full".into()));
    }
    ticket
}

/// Starts a directory scan away from VM/UI handler threads. Each discovered
/// supported file is submitted to the bounded media executor; the call itself
/// returns immediately and never exposes source paths to the control plane.
pub fn scan_media_directory(path: impl Into<PathBuf>) {
    let root = path.into();
    thread::Builder::new().name("media-directory-scan".into()).spawn(move || {
        let Ok(entries) = std::fs::read_dir(root) else { return; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = queue_media_path(path, MediaPriority::Thumbnail);
            }
        }
    }).ok();
}

// ---------------------------------------------------------------------------
// Metadata-only viewer sessions
// ---------------------------------------------------------------------------

/// A viewer session is deliberately host-owned.  The public value is only the
/// stable session id; paths and tickets remain in this bounded table and are
/// never serialized into Auto state or media URLs.
const MAX_MEDIA_SESSIONS: usize = 32;
const MAX_SESSION_ENTRIES: usize = 256;

#[derive(Debug)]
struct MediaSessionEntry {
    path: PathBuf,
    ticket: MediaAssetTicket,
    name: String,
}

#[derive(Debug)]
struct MediaSession {
    id: String,
    root: PathBuf,
    entries: Vec<MediaSessionEntry>,
    selected: usize,
    generation: u64,
}

static MEDIA_SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);
static MEDIA_SESSIONS: std::sync::LazyLock<Mutex<VecDeque<MediaSession>>> =
    std::sync::LazyLock::new(|| Mutex::new(VecDeque::new()));

fn media_uri(ticket: MediaAssetTicket) -> String {
    format!("/api/__auto/media/{}/{}", ticket.id, ticket.revision)
}

fn supported_media_path(path: &std::path::Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("jpg" | "jpeg" | "png" | "webp")
    )
}

fn append_session_entry(session_id: &str, path: PathBuf) {
    if !supported_media_path(&path) {
        return;
    }
    let ticket = queue_media_path(path.clone(), MediaPriority::Thumbnail);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("image")
        .to_owned();
    let mut sessions = match MEDIA_SESSIONS.lock() {
        Ok(sessions) => sessions,
        Err(_) => {
            global_media_registry().release(ticket.id);
            return;
        }
    };
    let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) else {
        global_media_registry().release(ticket.id);
        return;
    };
    if session.entries.len() >= MAX_SESSION_ENTRIES {
        global_media_registry().release(ticket.id);
        return;
    }
    session.entries.push(MediaSessionEntry { path, ticket, name });
}

fn populate_media_session(session_id: String, root: PathBuf) {
    // All filesystem work happens on this detached worker, never on a VM/UI
    // handler thread.  Sorting makes directory navigation deterministic.
    let mut paths = Vec::new();
    if root.is_file() {
        paths.push(root);
    } else if root.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&root) {
            paths.extend(entries.flatten().map(|entry| entry.path()).filter(|path| path.is_file()));
            paths.sort();
        }
    }
    for path in paths {
        append_session_entry(&session_id, path);
    }
}

/// Open a file or directory without blocking the caller on filesystem I/O.
/// The returned id is safe to keep in Auto model state.
pub fn open_media_session(root: impl Into<PathBuf>) -> String {
    let root = root.into();
    let id = format!("image-session-{}", MEDIA_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed));
    if let Ok(mut sessions) = MEDIA_SESSIONS.lock() {
        if sessions.len() >= MAX_MEDIA_SESSIONS {
            if let Some(evicted) = sessions.pop_front() {
                for entry in evicted.entries {
                    global_media_registry().release(entry.ticket.id);
                }
            }
        }
        sessions.push_back(MediaSession { id: id.clone(), root: root.clone(), entries: Vec::new(), selected: 0, generation: 1 });
    }
    let worker_id = id.clone();
    thread::Builder::new()
        .name("media-session-open".into())
        .spawn(move || populate_media_session(worker_id, root))
        .ok();
    id
}

fn session_snapshot_locked(session: &MediaSession) -> String {
    let uri = session
        .entries
        .get(session.selected)
        .map(|entry| media_uri(entry.ticket))
        .unwrap_or_default();
    let name = session
        .entries
        .get(session.selected)
        .map(|entry| entry.name.as_str())
        .unwrap_or("");
    // `id`, `uri`, and `name` are generated/filename values.  Escape the two
    // user-visible strings so this metadata response remains valid JSON.
    let escape = |value: &str| value.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "{{\"session\":\"{}\",\"selected\":{},\"count\":{},\"generation\":{},\"name\":\"{}\",\"uri\":\"{}\"}}",
        escape(&session.id), session.selected, session.entries.len(), session.generation, escape(name), escape(&uri)
    )
}

/// Return a metadata-only snapshot for a session.
pub fn media_session_snapshot(session_id: &str) -> String {
    MEDIA_SESSIONS
        .lock()
        .ok()
        .and_then(|sessions| sessions.iter().find(|session| session.id == session_id).map(session_snapshot_locked))
        .unwrap_or_else(|| "{\"error\":\"unknown_session\"}".into())
}

/// Return only the currently selected opaque media URI. This lets a front
/// model bind `ImageSurface.src` without parsing JSON or receiving paths.
pub fn media_session_uri(session_id: &str) -> String {
    MEDIA_SESSIONS
        .lock()
        .ok()
        .and_then(|sessions| sessions.iter().find(|session| session.id == session_id).and_then(|session| session.entries.get(session.selected).map(|entry| media_uri(entry.ticket))))
        .unwrap_or_default()
}

/// Return bounded display names for the current directory session. Names are
/// metadata only; the corresponding media URI is still resolved by index.
pub fn media_session_names(session_id: &str) -> Vec<String> {
    MEDIA_SESSIONS
        .lock()
        .ok()
        .and_then(|sessions| sessions.iter().find(|session| session.id == session_id).map(|session| session.entries.iter().map(|entry| entry.name.clone()).collect()))
        .unwrap_or_default()
}

/// Move the selection while preserving a bounded, host-side session table.
pub fn navigate_media_session(session_id: &str, delta: i32) -> String {
    let Ok(mut sessions) = MEDIA_SESSIONS.lock() else {
        return "{\"error\":\"session_lock\"}".into();
    };
    let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) else {
        return "{\"error\":\"unknown_session\"}".into();
    };
    if !session.entries.is_empty() {
        let len = session.entries.len() as i32;
        session.selected = (session.selected as i32 + delta).rem_euclid(len) as usize;
        session.generation = session.generation.saturating_add(1);
    }
    session_snapshot_locked(session)
}

/// Queue a viewport rendition for the selected asset.  The original ticket is
/// retained until the replacement is published, so a stale request cannot
/// invalidate the currently displayed media.
pub fn request_media_view(session_id: &str, index: i32, width: i32, height: i32, quality: i32) -> String {
    let (path, generation) = {
        let Ok(mut sessions) = MEDIA_SESSIONS.lock() else {
            return "{\"error\":\"session_lock\"}".into();
        };
        let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) else {
            return "{\"error\":\"unknown_session\"}".into();
        };
        if session.entries.is_empty() {
            return session_snapshot_locked(session);
        }
        session.selected = index.clamp(0, session.entries.len() as i32 - 1) as usize;
        session.generation = session.generation.saturating_add(1);
        (session.entries[session.selected].path.clone(), session.generation)
    };
    let spec = RenditionSpec { width: width.max(0) as u32, height: height.max(0) as u32, rotation_degrees: 0, quality: quality.clamp(1, 100) as u8, original_pixels: false };
    let ticket = queue_media_rendition(path, spec, MediaPriority::SettledCurrent, generation);
    if let Ok(mut sessions) = MEDIA_SESSIONS.lock() {
        if let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) {
            let old_ticket = session.entries[session.selected].ticket;
            session.entries[session.selected].ticket = ticket;
            // The session owns exactly one reference per entry. Release the
            // superseded ticket after swapping it so repeated view requests
            // cannot grow the registry indefinitely while queued work settles.
            if old_ticket.id != ticket.id {
                global_media_registry().release(old_ticket.id);
            }
            return session_snapshot_locked(session);
        }
    }
    global_media_registry().release(ticket.id);
    "{\"error\":\"unknown_session\"}".into()
}

/// Release all tickets held by a session.  Returns false for an already closed
/// or unknown id, allowing generated APIs to report deterministic close state.
pub fn close_media_session(session_id: &str) -> bool {
    let Some(session) = MEDIA_SESSIONS.lock().ok().and_then(|mut sessions| sessions.iter().position(|session| session.id == session_id).and_then(|index| sessions.remove(index))) else {
        return false;
    };
    for entry in session.entries {
        global_media_registry().release(entry.ticket.id);
    }
    true
}

pub fn media_session_stats(_session_id: &str) -> String {
    let stats = global_media_registry().stats();
    format!(
        "{{\"queued\":{},\"running\":{},\"completed\":{},\"dropped\":{},\"encoded_bytes\":{},\"decoded_bytes\":{},\"cache_hits\":{},\"cache_misses\":{}}}",
        stats.queued, stats.running, stats.completed, stats.dropped, stats.encoded_bytes, stats.decoded_bytes, stats.cache_hits, stats.cache_misses
    )
}

fn queue_media_rendition(path: PathBuf, spec: RenditionSpec, priority: MediaPriority, revision: u64) -> MediaAssetTicket {
    let key = MediaAssetKey { source_fingerprint: path.to_string_lossy().into_owned(), orientation: MediaOrientation::Normal, rendition: spec.clone(), revision };
    let ticket = global_media_registry().queue(key, MediaMetadata::default());
    if !global_media_worker_pool().submit_path(ticket, path, spec, priority, revision, revision) {
        let _ = global_media_registry().fail(ticket.id, MediaAssetError::Rejected("media queue is full".into()));
    }
    ticket
}

impl Drop for MediaWorkerPool { fn drop(&mut self) { self.shutdown(); } }

pub const MAX_IMAGE_FILE_BYTES: u64 = 1024 * 1024 * 1024;
pub const MAX_IMAGE_PIXELS: u64 = 400_000_000;
pub fn validate_image_limits(width: u32, height: u32, byte_len: u64) -> Result<(), MediaAssetError> {
    if byte_len > MAX_IMAGE_FILE_BYTES { return Err(MediaAssetError::FileTooLarge); }
    match (width as u64).checked_mul(height as u64) { Some(pixels) if pixels <= MAX_IMAGE_PIXELS => Ok(()), Some(_) => Err(MediaAssetError::PixelLimitExceeded), None => Err(MediaAssetError::SizeOverflow) }
}
pub fn inspect_image_metadata(bytes: &[u8]) -> Result<MediaMetadata, MediaAssetError> {
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().map_err(|_| MediaAssetError::UnsupportedFormat)?;
    let format = reader.format().ok_or(MediaAssetError::UnsupportedFormat)?;
    let mime_type = match format { image::ImageFormat::Jpeg => "image/jpeg", image::ImageFormat::Png => "image/png", image::ImageFormat::WebP => "image/webp", _ => return Err(MediaAssetError::UnsupportedFormat) };
    let (width, height) = reader.into_dimensions().map_err(|_| MediaAssetError::Corrupt)?;
    validate_image_limits(width, height, bytes.len() as u64)?;
    let orientation = exif::Reader::new().read_from_container(&mut std::io::Cursor::new(bytes)).ok().and_then(|exif| exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY).and_then(|field| field.value.get_uint(0))).map(|value| match value { 2 => MediaOrientation::FlipHorizontal, 3 => MediaOrientation::Rotate180, 4 => MediaOrientation::FlipVertical, 5 => MediaOrientation::Transpose, 6 => MediaOrientation::Rotate90, 7 => MediaOrientation::Transverse, 8 => MediaOrientation::Rotate270, _ => MediaOrientation::Normal }).unwrap_or_default();
    Ok(MediaMetadata { width, height, orientation, mime_type: mime_type.into(), byte_len: bytes.len() as u64 })
}

/// Transport-neutral result for the media endpoint.  The generated Axum and
/// VM HTTP adapters translate this shape to their native response types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaHttpResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Option<Arc<[u8]>>,
}

impl MediaHttpResponse {
    fn status(status: u16) -> Self {
        Self {
            status,
            headers: BTreeMap::new(),
            body: None,
        }
    }
}

/// Resolves the only public media URI shape without accepting filesystem paths.
pub fn media_http_response(
    registry: &MediaAssetRegistry,
    method: &str,
    path: &str,
    if_none_match: Option<&str>,
) -> MediaHttpResponse {
    let Some((id, revision)) = parse_media_path(path) else {
        return MediaHttpResponse::status(404);
    };

    match registry.lookup(id, revision) {
        MediaLookup::Missing => MediaHttpResponse::status(404),
        MediaLookup::Expired => MediaHttpResponse::status(410),
        MediaLookup::Pending => MediaHttpResponse::status(503),
        MediaLookup::Failed(_) => MediaHttpResponse::status(422),
        MediaLookup::Ready(bytes) => {
            let Some(metadata) = registry.metadata(id) else {
                return MediaHttpResponse::status(404);
            };
            let etag = format!("\"{id}-{revision}\"");
            if if_none_match == Some(etag.as_str()) {
                return MediaHttpResponse::status(304);
            }
            let mut headers = BTreeMap::new();
            headers.insert("content-type".into(), metadata.mime_type);
            headers.insert("content-length".into(), bytes.len().to_string());
            headers.insert("cache-control".into(), "public, max-age=31536000, immutable".into());
            headers.insert("etag".into(), etag);
            MediaHttpResponse {
                status: 200,
                headers,
                body: (method == "GET").then_some(bytes),
            }
        }
    }
}

/// Resolves a process-local media URI for native renderers.  Non-media URLs
/// and pending/expired tickets deliberately return `None`, allowing callers
/// to retain their normal file or network fallback behavior.
pub fn resolve_media_uri(uri: &str) -> Option<Vec<u8>> {
    if !uri.starts_with("/api/__auto/media/") {
        return None;
    }
    media_http_response(global_media_registry(), "GET", uri, None)
        .body
        .map(|bytes| bytes.to_vec())
}

/// Resolves already-decoded pixels for the native renderer. The lookup never
/// opens a file or decodes encoded bytes; only the resize lane populates this
/// bounded cache.
pub fn resolve_media_pixels(uri: &str) -> Option<(u32, u32, Arc<[u8]>)> {
    if !uri.starts_with("/api/__auto/media/") { return None; }
    let (id, _) = parse_media_path(uri)?;
    global_media_worker_pool().decoded_pixels(id)
}

/// Resolve a media URI for rendering, also returning the asset id (the
/// renderer keys its texture-Handle cache on it — see renderer.rs).
pub fn resolve_media_render(uri: &str) -> Option<(MediaAssetId, u32, u32, Arc<[u8]>)> {
    if !uri.starts_with("/api/__auto/media/") { return None; }
    let (id, _) = parse_media_path(uri)?;
    global_media_worker_pool().decoded_pixels(id).map(|(w, h, px)| (id, w, h, px))
}

fn parse_media_path(path: &str) -> Option<(MediaAssetId, u64)> {
    let route = path.strip_prefix("/api/__auto/media/")?;
    let (id, revision) = route.split_once('/')?;
    if revision.contains('/') || id.len() != 32 {
        return None;
    }
    Some((MediaAssetId(u128::from_str_radix(id, 16).ok()?), revision.parse().ok()?))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedImage { pub width: u32, pub height: u32, pub rgba: Arc<[u8]> }
pub fn decode_rendition(bytes: &[u8], orientation: MediaOrientation, spec: &RenditionSpec) -> Result<RenderedImage, MediaAssetError> {
    let image = image::load_from_memory(bytes).map_err(|_| MediaAssetError::Corrupt)?;
    let oriented = match orientation { MediaOrientation::Normal => image, MediaOrientation::FlipHorizontal => image.fliph(), MediaOrientation::Rotate180 => image.rotate180(), MediaOrientation::FlipVertical => image.flipv(), MediaOrientation::Transpose => image.rotate90().fliph(), MediaOrientation::Rotate90 => image.rotate90(), MediaOrientation::Transverse => image.rotate270().fliph(), MediaOrientation::Rotate270 => image.rotate270() };
    let rendered = if spec.original_pixels || spec.width == 0 || spec.height == 0 { oriented } else { oriented.resize(spec.width, spec.height, image::imageops::FilterType::Lanczos3) };
    let rgba = rendered.to_rgba8(); let (width, height) = rgba.dimensions();
    Ok(RenderedImage { width, height, rgba: Arc::from(rgba.into_raw()) })
}

/// Computes an RGBA8 allocation length without allowing image dimensions to
/// wrap on 32-bit or 64-bit hosts.
pub const fn checked_rgba_bytes(width: u32, height: u32) -> Option<usize> {
    match (width as usize).checked_mul(height as usize) {
        Some(pixels) => pixels.checked_mul(4),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use super::{
        checked_rgba_bytes, MediaAssetKey, MediaAssetRegistry, MediaAssetState, MediaLookup,
        MediaMetadata, MediaOrientation, MediaPriority, RenditionSpec,
    };

    #[test]
    fn asset_state_rejects_invalid_terminal_transitions() {
        assert!(MediaAssetState::Queued.can_transition_to(MediaAssetState::Reading));
        assert!(MediaAssetState::Decoding.can_transition_to(MediaAssetState::Error));
        assert!(MediaAssetState::Ready.can_transition_to(MediaAssetState::Expired));
        assert!(!MediaAssetState::Ready.can_transition_to(MediaAssetState::Decoding));
        assert!(!MediaAssetState::Expired.can_transition_to(MediaAssetState::Ready));
    }

    #[test]
    fn asset_state_checked_size_rejects_overflow() {
        assert_eq!(checked_rgba_bytes(2, 3), Some(24));
        assert_eq!(checked_rgba_bytes(u32::MAX, u32::MAX), None);
    }

    #[test]
    fn registry_tracks_ready_expired_and_missing_assets() {
        let registry = MediaAssetRegistry::new(Duration::ZERO);
        let ticket = registry.queue(
            MediaAssetKey {
                source_fingerprint: "fixture-a".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::viewport(16, 16),
                revision: 7,
            },
            MediaMetadata::default(),
        );
        assert_eq!(registry.lookup(ticket.id, ticket.revision), MediaLookup::Pending);

        registry.transition(ticket.id, MediaAssetState::Reading).unwrap();
        registry.transition(ticket.id, MediaAssetState::Decoding).unwrap();
        registry.transition(ticket.id, MediaAssetState::Transforming).unwrap();
        registry
            .publish_ready(ticket.id, ticket.revision, Arc::<[u8]>::from([1, 2, 3]))
            .unwrap();
        assert!(matches!(
            registry.lookup(ticket.id, ticket.revision),
            MediaLookup::Ready(_)
        ));
        assert_eq!(
            registry.wait_for_terminal(ticket.id, Duration::from_millis(1)),
            Some(MediaAssetState::Ready)
        );

        registry.release(ticket.id);
        registry.collect_expired();
        assert_eq!(registry.lookup(ticket.id, ticket.revision), MediaLookup::Expired);
        assert_eq!(registry.lookup(ticket.id, ticket.revision + 1), MediaLookup::Missing);
    }

    #[test]
    fn encoded_lru_deduplicates_refreshes_and_evicts_by_byte_budget() {
        let mut cache = super::EncodedByteLru::new(4);
        let first = super::MediaAssetId(1);
        let second = super::MediaAssetId(2);
        let third = super::MediaAssetId(3);
        cache.insert(first, Arc::<[u8]>::from([1, 2]));
        cache.insert(second, Arc::<[u8]>::from([3, 4]));
        assert_eq!(cache.used_bytes(), 4);
        assert_eq!(cache.get(first).unwrap().as_ref(), &[1, 2]);
        cache.insert(third, Arc::<[u8]>::from([5, 6]));

        assert!(cache.get(first).is_some(), "recently accessed entry is retained");
        assert!(cache.get(second).is_none(), "least-recent entry is evicted");
        assert!(cache.get(third).is_some());
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn decoded_budget_keeps_current_allows_its_temporary_overflow_and_evicts_neighbors() {
        let mut cache = super::DecodedPixelCache::new(4);
        let current = super::MediaAssetId(1);
        let neighbor = super::MediaAssetId(2);
        let replacement = super::MediaAssetId(3);
        cache.insert(neighbor, Arc::<[u8]>::from([1, 2, 3, 4]), super::MediaPin::Soft);
        cache.insert(current, Arc::<[u8]>::from([5, 6, 7, 8, 9, 10]), super::MediaPin::Hard);
        assert!(cache.contains(current));
        assert_eq!(cache.used_bytes(), 6, "hard-pinned current can temporarily exceed budget");
        assert!(!cache.contains(neighbor), "publish reclaims non-current content");

        cache.insert(replacement, Arc::<[u8]>::from([11, 12]), super::MediaPin::None);
        assert!(cache.contains(current));
        cache.set_pin(current, super::MediaPin::None);
        cache.collect();
        assert!(cache.used_bytes() <= 4);
    }

    #[test]
    fn priority_queue_is_bounded_stable_deduplicated_and_limits_neighbors() {
        let mut queue = super::MediaPriorityQueue::new();
        for id in 1..=2 {
            assert!(queue.push(super::MediaWork::new(id, super::MediaPriority::Neighbor)));
        }
        assert_eq!(queue.len(), 2);
        assert!(!queue.push(super::MediaWork::new(3, super::MediaPriority::Neighbor)));
        assert!(queue.push(super::MediaWork::new(10, super::MediaPriority::Thumbnail)));
        assert!(queue.push(super::MediaWork::new(11, super::MediaPriority::Current)));
        assert!(!queue.push(super::MediaWork::new(11, super::MediaPriority::Current)));
        assert_eq!(queue.pop().unwrap().id, 11);
        assert_eq!(queue.pop().unwrap().id, 1);
    }

    #[test]
    fn latest_wins_rejects_stale_work_at_every_pipeline_gate() {
        let mut gate = super::MediaLatestWins::new(4, 9);
        assert!(gate.accept_enqueue(4, 9));
        gate.advance(5, 10);
        assert!(!gate.accept_decode_complete(4, 9));
        assert!(!gate.accept_publish(5, 9));
        assert!(gate.accept_publish(5, 10));
        assert_eq!(gate.dropped(), 2);
    }

    #[test]
    fn worker_shutdown_joins_two_decode_workers_and_resize_lane() {
        let mut workers = super::MediaWorkerPool::new();
        assert_eq!(workers.worker_count(), 3);
        workers.shutdown();
        assert_eq!(workers.worker_count(), 0);
    }

    #[test]
    fn worker_pipeline_reads_decodes_resizes_and_publishes() {
        let registry = MediaAssetRegistry::new(Duration::from_secs(1));
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::new(2, 3))
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let key = MediaAssetKey {
            source_fingerprint: "worker-fixture".into(),
            orientation: MediaOrientation::Normal,
            rendition: RenditionSpec::viewport(1, 1),
            revision: 1,
        };
        let ticket = registry.queue(key, MediaMetadata::default());
        let mut workers = super::MediaWorkerPool::with_registry(registry.clone());
        assert!(workers.submit_bytes(ticket, Arc::from(bytes.into_inner()), RenditionSpec::viewport(1, 1), MediaPriority::Current, 1, 1));
        assert_eq!(registry.wait_for_terminal(ticket.id, Duration::from_secs(2)), Some(MediaAssetState::Ready));
        assert!(matches!(registry.lookup(ticket.id, ticket.revision), MediaLookup::Ready(_)));
        workers.shutdown();
    }

    #[test]
    fn viewer_session_opens_directory_and_exposes_only_metadata() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures");
        let session = super::open_media_session(root);
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while super::media_session_names(&session).is_empty() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        let names = super::media_session_names(&session);
        assert!(names.iter().any(|name| name == "rgb-1x1.png"));
        let snapshot = super::media_session_snapshot(&session);
        assert!(snapshot.contains("\"count\":"));
        assert!(!snapshot.contains("fixtures"), "snapshot must not leak source paths");
        let _ = super::request_media_view(&session, 0, 32, 32, 90);
        let uri = super::media_session_uri(&session);
        assert!(uri.is_empty() || uri.starts_with("/api/__auto/media/"));
        assert!(super::close_media_session(&session));
        assert!(!super::close_media_session(&session));
    }

    #[test]
    fn metadata_and_limits_identify_png_and_reject_invalid_or_excessive_inputs() {
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::new(2, 3))
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let metadata = super::inspect_image_metadata(bytes.get_ref()).unwrap();
        assert_eq!((metadata.width, metadata.height, metadata.mime_type.as_str()), (2, 3, "image/png"));
        assert_eq!(super::inspect_image_metadata(b"not an image"), Err(super::MediaAssetError::UnsupportedFormat));
        assert_eq!(super::validate_image_limits(2, 3, super::MAX_IMAGE_FILE_BYTES + 1), Err(super::MediaAssetError::FileTooLarge));
    }

    #[test]
    fn decode_and_rendition_applies_orientation_and_preserves_rgba() {
        let mut bytes = std::io::Cursor::new(Vec::new());
        let mut source = image::RgbaImage::new(2, 3);
        source.put_pixel(1, 2, image::Rgba([4, 5, 6, 7]));
        image::DynamicImage::ImageRgba8(source)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let rendered = super::decode_rendition(bytes.get_ref(), super::MediaOrientation::Rotate90, &RenditionSpec::original()).unwrap();
        assert_eq!((rendered.width, rendered.height), (3, 2));
        assert_eq!(rendered.rgba.len(), 3 * 2 * 4);
        assert!(rendered.rgba.chunks_exact(4).any(|pixel| pixel == [4, 5, 6, 7]));

        let viewport = super::decode_rendition(
            bytes.get_ref(),
            super::MediaOrientation::Normal,
            &RenditionSpec::viewport(1, 1),
        )
        .unwrap();
        assert_eq!((viewport.width, viewport.height), (1, 1));
    }

    #[test]
    fn http_response_maps_media_states_without_source_paths() {
        let registry = MediaAssetRegistry::new(Duration::ZERO);
        let ticket = registry.queue(
            MediaAssetKey {
                source_fingerprint: "C:/private/source.png".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::original(),
                revision: 9,
            },
            MediaMetadata {
                mime_type: "image/png".into(),
                ..Default::default()
            },
        );
        let path = format!("/api/__auto/media/{}/{}", ticket.id, ticket.revision);
        assert_eq!(super::media_http_response(&registry, "GET", &path, None).status, 503);

        registry.transition(ticket.id, MediaAssetState::Reading).unwrap();
        registry.transition(ticket.id, MediaAssetState::Decoding).unwrap();
        registry.transition(ticket.id, MediaAssetState::Transforming).unwrap();
        registry
            .publish_ready(ticket.id, ticket.revision, Arc::<[u8]>::from([1, 2, 3]))
            .unwrap();
        let response = super::media_http_response(&registry, "GET", &path, None);
        assert_eq!(response.status, 200);
        assert_eq!(response.body.as_deref(), Some(&[1, 2, 3][..]));
        assert_eq!(response.headers.get("content-type"), Some(&"image/png".into()));
        assert_eq!(super::media_http_response(&registry, "HEAD", &path, None).body, None);
        assert_eq!(super::media_http_response(&registry, "GET", &path, Some(response.headers["etag"].as_str())).status, 304);
        assert_eq!(super::media_http_response(&registry, "GET", "/api/__auto/media/nope/9", None).status, 404);
        assert!(!format!("{:?}", response).contains("private/source"));

        let failed = registry.queue(
            MediaAssetKey { source_fingerprint: "fixture-b".into(), orientation: Default::default(), rendition: RenditionSpec::original(), revision: 10 },
            MediaMetadata::default(),
        );
        registry.fail(failed.id, super::MediaAssetError::UnsupportedFormat).unwrap();
        let failed_path = format!("/api/__auto/media/{}/{}", failed.id, failed.revision);
        assert_eq!(super::media_http_response(&registry, "GET", &failed_path, None).status, 422);
        registry.release(ticket.id);
        registry.collect_expired();
        assert_eq!(super::media_http_response(&registry, "GET", &path, None).status, 410);
    }

    #[test]
    fn native_uri_resolves_registry_before_http_fallback() {
        let registry = super::global_media_registry();
        let ticket = registry.queue(
            MediaAssetKey {
                source_fingerprint: "native-uri-fixture".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::original(),
                revision: 1,
            },
            MediaMetadata::default(),
        );
        let uri = format!("/api/__auto/media/{}/{}", ticket.id, ticket.revision);
        assert_eq!(super::resolve_media_uri(&uri), None);
        registry.transition(ticket.id, MediaAssetState::Reading).unwrap();
        registry.transition(ticket.id, MediaAssetState::Decoding).unwrap();
        registry.transition(ticket.id, MediaAssetState::Transforming).unwrap();
        registry
            .publish_ready(ticket.id, ticket.revision, Arc::<[u8]>::from([0, 255, 2]))
            .unwrap();
        assert_eq!(super::resolve_media_uri(&uri), Some(vec![0, 255, 2]));
        assert_eq!(super::resolve_media_uri("relative/path.png"), None);
    }

    #[test]
    fn fixture_manifest() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures");
        let manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
                .unwrap();
        let fixtures = manifest["fixtures"].as_array().unwrap();
        assert_eq!(fixtures.len(), 6);
        for fixture in fixtures {
            let name = fixture["file"].as_str().unwrap();
            let bytes = std::fs::read(root.join(name)).unwrap();
            assert!(!bytes.is_empty(), "fixture {name} is empty");
            match fixture["kind"].as_str().unwrap() {
                "jpeg" => assert!(bytes.starts_with(&[0xff, 0xd8, 0xff]), "{name} is not JPEG"),
                "png" => assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"), "{name} is not PNG"),
                "webp" => assert!(bytes.starts_with(b"RIFF") && bytes[8..].starts_with(b"WEBP"), "{name} is not WebP"),
                "corrupt" => assert!(super::inspect_image_metadata(&bytes).is_err()),
                other => panic!("unknown fixture kind {other}"),
            }
        }
        let alpha = image::load_from_memory(&std::fs::read(root.join("alpha-2x1.png")).unwrap())
            .unwrap()
            .to_rgba8();
        assert_eq!(alpha.dimensions(), (2, 1));
        assert_eq!(alpha.get_pixel(1, 0).0[3], 64);
        let orientation = manifest["fixtures"]
            .as_array()
            .unwrap()
            .iter()
            .find(|fixture| fixture["file"] == "orientation-6.jpg")
            .unwrap();
        assert_eq!(orientation["orientation"], 6);
    }
}
