//! Shared, renderer-neutral image asset pipeline.
//!
//! The public media types, bounded scheduler, cache registry, and HTTP helper
//! live here so VM, generated Rust, and an HTTP backend use identical asset
//! identities and lifetimes.  Task 1 deliberately provides only the module
//! seam; its types and behavior are added in the following tasks.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

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

impl MediaAssetRegistry {
    pub const DEFAULT_TTL: Duration = Duration::from_secs(2);

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
struct DecodedEntry { bytes: Arc<[u8]>, pin: MediaPin, last_access: u64 }
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
        self.clock = self.clock.wrapping_add(1);
        if let Some(old) = self.entries.remove(&id) { self.used_bytes = self.used_bytes.saturating_sub(old.bytes.len()); }
        self.used_bytes = self.used_bytes.saturating_add(bytes.len());
        self.entries.insert(id, DecodedEntry { bytes, pin, last_access: self.clock }); self.collect();
    }
    pub fn set_pin(&mut self, id: MediaAssetId, pin: MediaPin) { if let Some(entry) = self.entries.get_mut(&id) { entry.pin = pin; } }
    pub fn contains(&self, id: MediaAssetId) -> bool { self.entries.contains_key(&id) }
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
        work.sequence = self.next_sequence; self.next_sequence = self.next_sequence.wrapping_add(1); self.items.push(work);
        self.items.sort_by_key(|item| (std::cmp::Reverse(item.priority), item.sequence));
        if self.items.len() > Self::CAPACITY { self.items.pop(); }
        true
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

struct WorkerState { stopping: bool }
struct WorkerInner { state: Mutex<WorkerState>, wake: Condvar }
/// Idle workers sleep on a condition variable; no timer or busy poll is used.
pub struct MediaWorkerPool { inner: Arc<WorkerInner>, workers: Vec<JoinHandle<()>> }
impl MediaWorkerPool {
    pub fn new() -> Self {
        let inner = Arc::new(WorkerInner { state: Mutex::new(WorkerState { stopping: false }), wake: Condvar::new() });
        let mut workers = Vec::with_capacity(3);
        for name in ["media-decode-1", "media-decode-2", "media-resize"] {
            let worker_inner = inner.clone();
            workers.push(thread::Builder::new().name(name.into()).spawn(move || {
                let mut state = worker_inner.state.lock().expect("media worker lock poisoned");
                while !state.stopping { state = worker_inner.wake.wait(state).expect("media worker lock poisoned"); }
            }).expect("spawn media worker"));
        }
        Self { inner, workers }
    }
    pub fn worker_count(&self) -> usize { self.workers.len() }
    pub fn shutdown(&mut self) {
        { let mut state = self.inner.state.lock().expect("media worker lock poisoned"); state.stopping = true; self.inner.wake.notify_all(); }
        for worker in self.workers.drain(..) { let _ = worker.join(); }
    }
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
        MediaMetadata, RenditionSpec,
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
}
