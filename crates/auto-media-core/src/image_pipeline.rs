//! Shared, renderer-neutral image asset pipeline (PLAN-811 media core).
//!
//! The public media types, bounded scheduler, cache registry, and HTTP rules
//! live here so VM, generated Rust, and an HTTP backend use identical asset
//! identities and lifetimes.  Migrated verbatim from auto-lang's
//! `ui/image_pipeline.rs`; the historical in-`ui` path remains as a thin
//! re-export facade with unchanged type identity.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
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
        matches!(
            self,
            Self::Ready | Self::Error | Self::Stale | Self::Expired
        )
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
    /// Per-registry mutation counter (bumped wherever the process-global
    /// media generation bumps). Scoped quiescence oracle: unlike the global
    /// generation it is immune to unrelated registries in the same process.
    mutations: AtomicU64,
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
/// PLAN-811: the instance lives on the default shared [`MediaRuntime`] so the
/// registry, worker pool, and session table share one lifetime owner.
pub fn global_media_registry() -> &'static MediaAssetRegistry {
    global_media_runtime().registry()
}

/// PLAN-095 T-02: process-wide count of non-terminal media assets — the
/// ImageSurface readiness-pump subscription gate.
pub fn pending_media_count() -> usize {
    global_media_registry().pending_count()
}

/// PLAN-095 T-02: monotonic media lifecycle generation — bumped on every
/// queue/transition/publish/fail. The renderer's desktop-level wake poll
/// compares generations: subscription-time gates cannot see millisecond
/// decode windows, but a changed generation reliably wakes one sweep.
static MEDIA_CHANGE_GEN: AtomicU64 = AtomicU64::new(0);

pub fn media_change_generation() -> u64 {
    MEDIA_CHANGE_GEN.load(Ordering::Acquire)
}

fn bump_media_change_gen() {
    MEDIA_CHANGE_GEN.fetch_add(1, Ordering::AcqRel);
}

/// Bumps both the process-global media generation and this registry's
/// scoped mutation counter (same call sites — one semantic event).
fn bump_registry_mutations(registry: &RegistryInner) {
    bump_media_change_gen();
    registry.mutations.fetch_add(1, Ordering::AcqRel);
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
                mutations: AtomicU64::new(0),
            }),
        }
    }

    pub fn queue(&self, key: MediaAssetKey, metadata: MediaMetadata) -> MediaAssetTicket {
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
        // Closed admission (review F811-R1-02): a shut-down registry accepts
        // no new assets. Signature compatibility is kept by returning a DEAD
        // ticket — a fresh opaque id that is never inserted, so every later
        // operation on it reports Missing/Expired instead of resurrecting
        // state. The process-default registry never shuts down.
        if state.shutting_down {
            let id = loop {
                let candidate =
                    MediaAssetId(((fastrand::u64(..) as u128) << 64) | fastrand::u64(..) as u128);
                if !state.entries.contains_key(&candidate) {
                    break candidate;
                }
            };
            return MediaAssetTicket {
                id,
                revision: key.revision,
            };
        }
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
            let candidate =
                MediaAssetId(((fastrand::u64(..) as u128) << 64) | fastrand::u64(..) as u128);
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
        bump_registry_mutations(&self.inner);
        MediaAssetTicket { id, revision }
    }

    pub fn transition(
        &self,
        id: MediaAssetId,
        next: MediaAssetState,
    ) -> Result<(), MediaAssetError> {
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
        if state.shutting_down {
            return Err(MediaAssetError::Rejected(
                "media registry is shut down".into(),
            ));
        }
        let entry = state.entries.get_mut(&id).ok_or(MediaAssetError::Expired)?;
        if !entry.state.can_transition_to(next) {
            return Err(MediaAssetError::Rejected(
                "invalid media state transition".into(),
            ));
        }
        entry.state = next;
        if next == MediaAssetState::Error && entry.error.is_none() {
            entry.error = Some(MediaAssetError::Corrupt);
        }
        self.inner.changed.notify_all();
        bump_registry_mutations(&self.inner);
        Ok(())
    }

    pub fn publish_ready(
        &self,
        id: MediaAssetId,
        revision: u64,
        bytes: Arc<[u8]>,
    ) -> Result<(), MediaAssetError> {
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
        if state.shutting_down {
            return Err(MediaAssetError::Rejected(
                "media registry is shut down".into(),
            ));
        }
        let entry = state.entries.get_mut(&id).ok_or(MediaAssetError::Expired)?;
        if entry.key.revision != revision {
            return Err(MediaAssetError::Stale);
        }
        if entry.state != MediaAssetState::Transforming {
            return Err(MediaAssetError::Rejected(
                "asset is not ready to publish".into(),
            ));
        }
        entry.encoded = Some(bytes);
        entry.state = MediaAssetState::Ready;
        self.inner.changed.notify_all();
        bump_registry_mutations(&self.inner);
        Ok(())
    }

    pub fn fail(&self, id: MediaAssetId, error: MediaAssetError) -> Result<(), MediaAssetError> {
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
        if state.shutting_down {
            return Err(MediaAssetError::Rejected(
                "media registry is shut down".into(),
            ));
        }
        let entry = state.entries.get_mut(&id).ok_or(MediaAssetError::Expired)?;
        if !entry.state.can_transition_to(MediaAssetState::Error) {
            return Err(MediaAssetError::Rejected(
                "asset cannot fail from its current state".into(),
            ));
        }
        entry.state = MediaAssetState::Error;
        entry.error = Some(error);
        self.inner.changed.notify_all();
        bump_registry_mutations(&self.inner);
        Ok(())
    }

    pub fn lookup(&self, id: MediaAssetId, revision: u64) -> MediaLookup {
        let state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
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
            MediaAssetState::Error => {
                MediaLookup::Failed(entry.error.clone().unwrap_or(MediaAssetError::Corrupt))
            }
            MediaAssetState::Expired | MediaAssetState::Stale => MediaLookup::Expired,
            _ => MediaLookup::Pending,
        }
    }

    /// PLAN-095 T-02: number of assets still short of a terminal state
    /// (queued/reading/decoding/transforming). Transient by construction —
    /// worker lanes drive every ticket to Ready/Error/Stale/Expired — so it
    /// is safe as a liveness-pump gate (ticks stop when it reaches zero).
    pub fn pending_count(&self) -> usize {
        let state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
        state
            .entries
            .values()
            .filter(|entry| !entry.state.is_terminal())
            .count()
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
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
        if state.shutting_down {
            return false;
        }
        let Some(entry) = state.entries.get_mut(&id) else {
            return false;
        };
        entry.metadata = metadata;
        self.inner.changed.notify_all();
        true
    }

    /// Scoped mutation generation of THIS registry (quiescence oracle).
    pub fn mutation_generation(&self) -> u64 {
        self.inner.mutations.load(Ordering::Acquire)
    }

    pub fn stats(&self) -> MediaPipelineStats {
        let state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
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
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
        if state.shutting_down {
            return false;
        }
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
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
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
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
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

    pub fn wait_for_terminal(
        &self,
        id: MediaAssetId,
        timeout: Duration,
    ) -> Option<MediaAssetState> {
        let deadline = Instant::now() + timeout;
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
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
        let mut state = self
            .inner
            .state
            .lock()
            .expect("media registry lock poisoned");
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
pub enum MediaPin {
    #[default]
    None,
    Soft,
    Hard,
}
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
    budget_bytes: usize,
    used_bytes: usize,
    clock: u64,
    entries: HashMap<MediaAssetId, DecodedEntry>,
    stats: MediaPipelineStats,
}
impl DecodedPixelCache {
    pub const DEFAULT_BUDGET_BYTES: usize = 256 * 1024 * 1024;
    pub fn new(budget_bytes: usize) -> Self {
        Self {
            budget_bytes,
            used_bytes: 0,
            clock: 0,
            entries: HashMap::new(),
            stats: MediaPipelineStats::default(),
        }
    }
    pub fn insert(&mut self, id: MediaAssetId, bytes: Arc<[u8]>, pin: MediaPin) {
        self.insert_rendered(id, 0, 0, bytes, pin);
    }
    pub fn insert_rendered(
        &mut self,
        id: MediaAssetId,
        width: u32,
        height: u32,
        bytes: Arc<[u8]>,
        pin: MediaPin,
    ) {
        self.clock = self.clock.wrapping_add(1);
        if let Some(old) = self.entries.remove(&id) {
            self.used_bytes = self.used_bytes.saturating_sub(old.bytes.len());
        }
        self.used_bytes = self.used_bytes.saturating_add(bytes.len());
        self.entries.insert(
            id,
            DecodedEntry {
                width,
                height,
                bytes,
                pin,
                last_access: self.clock,
            },
        );
        self.collect();
    }
    pub fn set_pin(&mut self, id: MediaAssetId, pin: MediaPin) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.pin = pin;
        }
    }
    pub fn contains(&self, id: MediaAssetId) -> bool {
        self.entries.contains_key(&id)
    }
    pub fn get_rendered(&mut self, id: MediaAssetId) -> Option<(u32, u32, Arc<[u8]>)> {
        self.clock = self.clock.wrapping_add(1);
        let entry = self.entries.get_mut(&id)?;
        entry.last_access = self.clock;
        self.stats.cache_hits = self.stats.cache_hits.saturating_add(1);
        Some((entry.width, entry.height, entry.bytes.clone()))
    }
    pub const fn used_bytes(&self) -> usize {
        self.used_bytes
    }
    pub fn stats(&self) -> &MediaPipelineStats {
        &self.stats
    }
    pub fn collect(&mut self) {
        while self.used_bytes > self.budget_bytes {
            let candidate = self
                .entries
                .iter()
                .filter(|(_, e)| e.pin != MediaPin::Hard)
                .min_by_key(|(_, e)| (e.pin != MediaPin::None, e.last_access))
                .map(|(&id, _)| id);
            let Some(id) = candidate else {
                break;
            };
            let entry = self
                .entries
                .remove(&id)
                .expect("decoded cache entry exists");
            self.used_bytes = self.used_bytes.saturating_sub(entry.bytes.len());
            self.stats.evictions = self.stats.evictions.saturating_add(1);
        }
        self.stats.decoded_bytes = self.used_bytes;
    }
}
impl Default for DecodedPixelCache {
    fn default() -> Self {
        Self::new(Self::DEFAULT_BUDGET_BYTES)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MediaPriority {
    Thumbnail = 5,
    Neighbor = 10,
    SettledCurrent = 90,
    Current = 100,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaWork {
    pub id: u64,
    pub priority: MediaPriority,
    sequence: u64,
}
impl MediaWork {
    pub const fn new(id: u64, priority: MediaPriority) -> Self {
        Self {
            id,
            priority,
            sequence: 0,
        }
    }
}
/// Scheduler ingress queue; the worker pool consumes `pop` in priority then
/// FIFO order.  Duplicate renditions are coalesced before consuming capacity.
#[derive(Default)]
pub struct MediaPriorityQueue {
    items: Vec<MediaWork>,
    next_sequence: u64,
}
impl MediaPriorityQueue {
    pub const CAPACITY: usize = 8;
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn push(&mut self, mut work: MediaWork) -> bool {
        if self.items.iter().any(|item| item.id == work.id) {
            return false;
        }
        if work.priority == MediaPriority::Neighbor
            && self
                .items
                .iter()
                .filter(|item| item.priority == MediaPriority::Neighbor)
                .count()
                >= 2
        {
            return false;
        }
        if self.items.len() >= Self::CAPACITY {
            let lowest = self
                .items
                .last()
                .map(|item| item.priority)
                .unwrap_or(MediaPriority::Thumbnail);
            if work.priority <= lowest {
                return false;
            }
        }
        let work_id = work.id;
        work.sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.items.push(work);
        self.items
            .sort_by_key(|item| (std::cmp::Reverse(item.priority), item.sequence));
        if self.items.len() > Self::CAPACITY {
            self.items.pop();
        }
        self.items.iter().any(|item| item.id == work_id)
    }
    pub fn pop(&mut self) -> Option<MediaWork> {
        (!self.items.is_empty()).then(|| self.items.remove(0))
    }
    /// Drains all queued work. Shutdown paths use this to drop pending
    /// renditions without executing them.
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// Per-session generation/revision gate shared by each asynchronous phase.
#[derive(Clone, Debug)]
pub struct MediaLatestWins {
    generation: u64,
    view_revision: u64,
    dropped: u64,
}
impl MediaLatestWins {
    pub const fn new(generation: u64, view_revision: u64) -> Self {
        Self {
            generation,
            view_revision,
            dropped: 0,
        }
    }
    pub fn advance(&mut self, generation: u64, view_revision: u64) {
        self.generation = generation;
        self.view_revision = view_revision;
    }
    fn accept(&mut self, generation: u64, view_revision: u64) -> bool {
        let ok = (generation, view_revision) == (self.generation, self.view_revision);
        if !ok {
            self.dropped = self.dropped.saturating_add(1);
        }
        ok
    }
    pub fn accept_enqueue(&mut self, generation: u64, view_revision: u64) -> bool {
        self.accept(generation, view_revision)
    }
    pub fn accept_decode_complete(&mut self, generation: u64, view_revision: u64) -> bool {
        self.accept(generation, view_revision)
    }
    pub fn accept_publish(&mut self, generation: u64, view_revision: u64) -> bool {
        self.accept(generation, view_revision)
    }
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }
}

pub const MAX_IMAGE_FILE_BYTES: u64 = 1024 * 1024 * 1024;
pub const MAX_IMAGE_PIXELS: u64 = 400_000_000;
pub fn validate_image_limits(
    width: u32,
    height: u32,
    byte_len: u64,
) -> Result<(), MediaAssetError> {
    if byte_len > MAX_IMAGE_FILE_BYTES {
        return Err(MediaAssetError::FileTooLarge);
    }
    match (width as u64).checked_mul(height as u64) {
        Some(pixels) if pixels <= MAX_IMAGE_PIXELS => Ok(()),
        Some(_) => Err(MediaAssetError::PixelLimitExceeded),
        None => Err(MediaAssetError::SizeOverflow),
    }
}
pub fn inspect_image_metadata(bytes: &[u8]) -> Result<MediaMetadata, MediaAssetError> {
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| MediaAssetError::UnsupportedFormat)?;
    let format = reader.format().ok_or(MediaAssetError::UnsupportedFormat)?;
    let mime_type = match format {
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::WebP => "image/webp",
        _ => return Err(MediaAssetError::UnsupportedFormat),
    };
    let (width, height) = reader
        .into_dimensions()
        .map_err(|_| MediaAssetError::Corrupt)?;
    validate_image_limits(width, height, bytes.len() as u64)?;
    let orientation = exif::Reader::new()
        .read_from_container(&mut std::io::Cursor::new(bytes))
        .ok()
        .and_then(|exif| {
            exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
                .and_then(|field| field.value.get_uint(0))
        })
        .map(|value| match value {
            2 => MediaOrientation::FlipHorizontal,
            3 => MediaOrientation::Rotate180,
            4 => MediaOrientation::FlipVertical,
            5 => MediaOrientation::Transpose,
            6 => MediaOrientation::Rotate90,
            7 => MediaOrientation::Transverse,
            8 => MediaOrientation::Rotate270,
            _ => MediaOrientation::Normal,
        })
        .unwrap_or_default();
    Ok(MediaMetadata {
        width,
        height,
        orientation,
        mime_type: mime_type.into(),
        byte_len: bytes.len() as u64,
    })
}

/// Public media URI shape: `/api/__auto/media/{id}/{revision}`. Callers never
/// construct paths — tickets are the only addressing currency.
pub fn media_uri(ticket: MediaAssetTicket) -> String {
    format!("/api/__auto/media/{}/{}", ticket.id, ticket.revision)
}

/// The four image extensions the pipeline accepts (jpg/jpeg/png/webp),
/// compared ASCII-lowercased. Distinct from the media file service's video /
/// audio whitelist.
pub fn supported_media_path(path: &std::path::Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("jpg" | "jpeg" | "png" | "webp")
    )
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
            headers.insert(
                "cache-control".into(),
                "public, max-age=31536000, immutable".into(),
            );
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

/// PLAN-095 T-02: renderer-facing terminal status of one media URI, consumed
/// by the ImageSurface loaded/error notify sweep. `NotMedia` URIs (no media
/// route prefix) never participate in notification — they keep today's
/// placeholder-forever display semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaSurfaceStatus {
    /// Decoded rendition is consumable by the current frame.
    Ready,
    /// Ticket still queued/decoding (or not yet terminal).
    Pending,
    /// Terminal failure with a locatable reason (corrupt bytes, missing or
    /// evicted ticket, expired entry…).
    Failed(String),
    /// Not a process-local media URI — outside the notify contract.
    NotMedia,
}

/// PLAN-095 T-02: resolve the notify status of a rendered `src`.
pub fn media_surface_status(uri: &str) -> MediaSurfaceStatus {
    if !uri.starts_with("/api/__auto/media/") {
        return MediaSurfaceStatus::NotMedia;
    }
    let Some((id, revision)) = parse_media_path(uri) else {
        return MediaSurfaceStatus::Failed("media uri is not a well-formed ticket path".into());
    };
    match global_media_registry().lookup(id, revision) {
        MediaLookup::Ready(_) => MediaSurfaceStatus::Ready,
        MediaLookup::Pending => MediaSurfaceStatus::Pending,
        MediaLookup::Failed(e) => MediaSurfaceStatus::Failed(e.to_string()),
        // 缺票（registry 驱逐/进程内从未入队）与过期都是终态失败——
        // 占位图永远不会自行变好，按合同派发 error。
        MediaLookup::Missing => MediaSurfaceStatus::Failed("media ticket missing".into()),
        MediaLookup::Expired => MediaSurfaceStatus::Failed("media ticket expired".into()),
    }
}

/// Parses the public media URI shape. Public so adapter surfaces (VM fast
/// path, back-proxy route, generated backends) share one parser.
pub fn parse_media_path(path: &str) -> Option<(MediaAssetId, u64)> {
    let route = path.strip_prefix("/api/__auto/media/")?;
    let (id, revision) = route.split_once('/')?;
    if revision.contains('/') || id.len() != 32 {
        return None;
    }
    Some((
        MediaAssetId(u128::from_str_radix(id, 16).ok()?),
        revision.parse().ok()?,
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}
pub fn decode_rendition(
    bytes: &[u8],
    orientation: MediaOrientation,
    spec: &RenditionSpec,
) -> Result<RenderedImage, MediaAssetError> {
    let image = image::load_from_memory(bytes).map_err(|_| MediaAssetError::Corrupt)?;
    let oriented = match orientation {
        MediaOrientation::Normal => image,
        MediaOrientation::FlipHorizontal => image.fliph(),
        MediaOrientation::Rotate180 => image.rotate180(),
        MediaOrientation::FlipVertical => image.flipv(),
        MediaOrientation::Transpose => image.rotate90().fliph(),
        MediaOrientation::Rotate90 => image.rotate90(),
        MediaOrientation::Transverse => image.rotate270().fliph(),
        MediaOrientation::Rotate270 => image.rotate270(),
    };
    let rendered = if spec.original_pixels || spec.width == 0 || spec.height == 0 {
        oriented
    } else {
        oriented.resize(
            spec.width,
            spec.height,
            image::imageops::FilterType::Lanczos3,
        )
    };
    let rgba = rendered.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(RenderedImage {
        width,
        height,
        rgba: Arc::from(rgba.into_raw()),
    })
}

/// Computes an RGBA8 allocation length without allowing image dimensions to
/// wrap on 32-bit or 64-bit hosts.
pub const fn checked_rgba_bytes(width: u32, height: u32) -> Option<usize> {
    match (width as usize).checked_mul(height as usize) {
        Some(pixels) => pixels.checked_mul(4),
        None => None,
    }
}

// ---------------------------------------------------------------------------
// Bounded executor: decode workers + resize lane
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum MediaSource {
    Path(PathBuf),
    Bytes(Arc<[u8]>),
}

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

enum ResizeMessage {
    Work(DecodedWork),
    Stop,
}

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
    /// Resize-lane Work gauges (review R5 semantics): `resize_enqueued`
    /// counts Work messages ACCEPTED into the channel (incremented only
    /// after a successful send — a decode thread blocked mid-send is NOT
    /// counted); `resize_handled` counts Work messages the lane has TAKEN
    /// (incremented at receive, before processing; the stopping fast-path
    /// settles without processing). Stop counts on NEITHER side, and the
    /// shutdown order guarantees no Work is accepted behind Stop, so after
    /// shutdown completes enqueued == handled exactly.
    resize_enqueued: AtomicU64,
    resize_handled: AtomicU64,
}

static NEXT_MEDIA_WORK_ID: AtomicU64 = AtomicU64::new(1);

/// Bounded media executor: two decode/read workers feed one resize/publish
/// lane. Workers sleep on condition variables or channel receives while idle;
/// no UI call performs filesystem I/O or image decoding.
pub struct MediaWorkerPool {
    inner: Arc<WorkerInner>,
    // Join handles sit behind a mutex so shutdown/drop can run through &self
    // (the pool is shared as a process static and by MediaRuntime). The
    // resize lane is tracked separately: shutdown joins the DECODE workers
    // first (all senders gone), THEN sends Stop and joins the lane — no Work
    // can be accepted behind the stop marker, so the occupancy gauges settle
    // to the true terminal state (review R5).
    resize_worker: Mutex<Option<JoinHandle<()>>>,
    decode_workers: Mutex<Vec<JoinHandle<()>>>,
}

impl MediaWorkerPool {
    pub fn new() -> Self {
        Self::with_registry(MediaAssetRegistry::default())
    }

    pub fn with_registry(registry: MediaAssetRegistry) -> Self {
        let (resize_tx, resize_rx) = sync_channel(8);
        let inner = Arc::new(WorkerInner {
            state: Mutex::new(WorkerState {
                stopping: false,
                queue: MediaPriorityQueue::new(),
                jobs: HashMap::new(),
            }),
            wake: Condvar::new(),
            registry,
            resize_tx,
            caches: Mutex::new(PipelineCaches {
                encoded: EncodedByteLru::default(),
                decoded: DecodedPixelCache::default(),
            }),
            latest: Mutex::new(HashMap::new()),
            resize_enqueued: AtomicU64::new(0),
            resize_handled: AtomicU64::new(0),
        });
        let resize_inner = inner.clone();
        let resize_thread = thread::Builder::new()
            .name("media-resize".into())
            .spawn(move || {
                run_resize_worker(resize_inner, resize_rx);
            })
            .expect("spawn media resize worker");
        let mut decode_workers = Vec::new();
        for name in ["media-decode-1", "media-decode-2"] {
            let worker_inner = inner.clone();
            decode_workers.push(
                thread::Builder::new()
                    .name(name.into())
                    .spawn(move || {
                        run_decode_worker(worker_inner);
                    })
                    .expect("spawn media decode worker"),
            );
        }
        Self {
            inner,
            resize_worker: Mutex::new(Some(resize_thread)),
            decode_workers: Mutex::new(decode_workers),
        }
    }

    pub fn worker_count(&self) -> usize {
        let decode = self.decode_workers.lock().map(|w| w.len()).unwrap_or(0);
        let resize = self
            .resize_worker
            .lock()
            .map(|w| w.is_some() as usize)
            .unwrap_or(0);
        decode + resize
    }

    pub fn submit_path(
        &self,
        ticket: MediaAssetTicket,
        path: PathBuf,
        spec: RenditionSpec,
        priority: MediaPriority,
        generation: u64,
        view_revision: u64,
    ) -> bool {
        self.submit(MediaWorkItem {
            id: NEXT_MEDIA_WORK_ID.fetch_add(1, Ordering::Relaxed),
            ticket,
            source: MediaSource::Path(path),
            spec,
            priority,
            generation,
            view_revision,
        })
    }

    pub fn submit_bytes(
        &self,
        ticket: MediaAssetTicket,
        bytes: Arc<[u8]>,
        spec: RenditionSpec,
        priority: MediaPriority,
        generation: u64,
        view_revision: u64,
    ) -> bool {
        self.submit(MediaWorkItem {
            id: NEXT_MEDIA_WORK_ID.fetch_add(1, Ordering::Relaxed),
            ticket,
            source: MediaSource::Bytes(bytes),
            spec,
            priority,
            generation,
            view_revision,
        })
    }

    fn submit(&self, item: MediaWorkItem) -> bool {
        if let Ok(mut latest) = self.inner.latest.lock() {
            if let Some(gate) = latest.get_mut(&item.ticket.id) {
                gate.advance(item.generation, item.view_revision);
            } else {
                latest.insert(
                    item.ticket.id,
                    MediaLatestWins::new(item.generation, item.view_revision),
                );
            }
        }
        let mut state = self.inner.state.lock().expect("media worker lock poisoned");
        if state.stopping || !state.queue.push(MediaWork::new(item.id, item.priority)) {
            return false;
        }
        state.jobs.insert(item.id, item);
        self.inner.wake.notify_one();
        true
    }

    pub fn decoded_pixels(&self, id: MediaAssetId) -> Option<(u32, u32, Arc<[u8]>)> {
        self.inner.caches.lock().ok()?.decoded.get_rendered(id)
    }

    /// Resize-lane in-channel Work backlog: messages ACCEPTED into the
    /// channel minus messages the lane has TAKEN. Reads enqueued first,
    /// then handled: a Work taken between the two loads is subtracted
    /// against an enqueue that may postdate the first load, so the value
    /// can transiently UNDER-report (never above the true outstanding
    /// count, never negative — saturating). A decode thread blocked
    /// mid-send is not counted (not yet in the channel). A backlog >=
    /// channel capacity (8) therefore proves all 8 slots hold accepted,
    /// untaken Work — the bounded channel is FULL. 0 == drained/idle;
    /// after shutdown completes the value is exactly 0 (no Work can be
    /// accepted behind Stop).
    pub fn resize_backlog(&self) -> u64 {
        let enqueued = self.inner.resize_enqueued.load(Ordering::Acquire);
        let handled = self.inner.resize_handled.load(Ordering::Acquire);
        enqueued.saturating_sub(handled)
    }

    /// Stops new work, drains the ingress queue, stops the resize lane, and
    /// joins every owned worker thread. Idempotent; safe through `&self`.
    pub fn shutdown(&self) {
        {
            let mut state = self.inner.state.lock().expect("media worker lock poisoned");
            state.stopping = true;
            state.queue.clear();
            state.jobs.clear();
            self.inner.wake.notify_all();
        }
        // Join the DECODE workers (the channel's only senders) first: once
        // they are gone, no Work can be accepted behind the stop marker.
        // In-flight decoders finish their current item (bounded) and exit on
        // `stopping`; a send racing this window either lands ahead of Stop
        // (the lane settles it below) or fails once the receiver is gone
        // (sender-side rollback settles it).
        if let Ok(mut decoders) = self.decode_workers.lock() {
            for worker in decoders.drain(..) {
                let _ = worker.join();
            }
        }
        // Now the lane is the only participant: send Stop and join it. The
        // lane settles every remaining Work (fast-path discard while
        // stopping), so the occupancy gauges return to the true terminal
        // state when shutdown completes.
        let _ = self.inner.resize_tx.send(ResizeMessage::Stop);
        if let Ok(mut lane) = self.resize_worker.lock() {
            if let Some(worker) = lane.take() {
                let _ = worker.join();
            }
        }
    }
}

fn take_media_work(inner: &Arc<WorkerInner>) -> Option<MediaWorkItem> {
    let mut state = inner.state.lock().expect("media worker lock poisoned");
    loop {
        if state.stopping {
            return None;
        }
        if let Some(work) = state.queue.pop() {
            return state.jobs.remove(&work.id);
        }
        state = inner.wake.wait(state).expect("media worker lock poisoned");
    }
}

fn run_decode_worker(inner: Arc<WorkerInner>) {
    while let Some(item) = take_media_work(&inner) {
        let item_id = item.ticket.id;
        if let Ok(mut latest) = inner.latest.lock() {
            if !latest.get_mut(&item.ticket.id).is_some_and(|gate| {
                gate.accept_decode_complete(item.generation, item.view_revision)
            }) {
                let _ = inner
                    .registry
                    .transition(item.ticket.id, MediaAssetState::Stale);
                continue;
            }
        }
        let result = (|| {
            inner
                .registry
                .transition(item.ticket.id, MediaAssetState::Reading)?;
            let encoded: Arc<[u8]> = match &item.source {
                MediaSource::Path(path) => Arc::from(
                    std::fs::read(path)
                        .map_err(|_| MediaAssetError::Rejected("image read failed".into()))?,
                ),
                MediaSource::Bytes(bytes) => bytes.clone(),
            };
            let metadata = inspect_image_metadata(&encoded)?;
            inner
                .registry
                .update_metadata(item.ticket.id, metadata.clone());
            inner
                .registry
                .transition(item.ticket.id, MediaAssetState::Decoding)?;
            let image = image::load_from_memory(&encoded).map_err(|_| MediaAssetError::Corrupt)?;
            Ok::<_, MediaAssetError>(DecodedWork {
                item,
                encoded,
                metadata,
                image,
            })
        })();
        match result {
            Ok(decoded) => {
                if inner.resize_tx.send(ResizeMessage::Work(decoded)).is_ok() {
                    // Count only ACCEPTED Work (a thread blocked mid-send is
                    // not in the channel yet); the receiver takes it after
                    // the send returns, so the take can only be counted later.
                    inner.resize_enqueued.fetch_add(1, Ordering::AcqRel);
                } else {
                    // Receiver gone: nothing was accepted, nothing to settle.
                }
            }
            Err(error) => {
                let _ = inner.registry.fail(item_id, error);
            }
        }
    }
}

fn oriented_image(
    image: image::DynamicImage,
    orientation: MediaOrientation,
) -> image::DynamicImage {
    match orientation {
        MediaOrientation::Normal => image,
        MediaOrientation::FlipHorizontal => image.fliph(),
        MediaOrientation::Rotate180 => image.rotate180(),
        MediaOrientation::FlipVertical => image.flipv(),
        MediaOrientation::Transpose => image.rotate90().fliph(),
        MediaOrientation::Rotate90 => image.rotate90(),
        MediaOrientation::Transverse => image.rotate270().fliph(),
        MediaOrientation::Rotate270 => image.rotate270(),
    }
}

fn render_decoded(
    image: image::DynamicImage,
    orientation: MediaOrientation,
    spec: &RenditionSpec,
) -> RenderedImage {
    let oriented = oriented_image(image, orientation);
    let rendered = if spec.original_pixels || spec.width == 0 || spec.height == 0 {
        oriented
    } else {
        oriented.resize(
            spec.width,
            spec.height,
            image::imageops::FilterType::Lanczos3,
        )
    };
    let rgba = rendered.to_rgba8();
    let (width, height) = rgba.dimensions();
    RenderedImage {
        width,
        height,
        rgba: Arc::from(rgba.into_raw()),
    }
}

fn run_resize_worker(inner: Arc<WorkerInner>, resize_rx: Receiver<ResizeMessage>) {
    while let Ok(message) = resize_rx.recv() {
        let ResizeMessage::Work(work) = message else {
            // Stop is control-plane only — it is not a Work message and
            // counts on neither gauge. The shutdown order (decode workers
            // joined before Stop is sent) means nothing can be accepted
            // behind this marker.
            break;
        };
        inner.resize_handled.fetch_add(1, Ordering::AcqRel);
        {
            // Stopping fast-path: settle the take and discard without the
            // heavy transform — the registry is closing anyway and publish
            // would be rejected; this keeps shutdown bounded by decode,
            // not by resize, work.
            let stopping = inner
                .state
                .lock()
                .map(|state| state.stopping)
                .unwrap_or(true);
            if stopping {
                drop(work);
                continue;
            }
        }
        if let Ok(mut latest) = inner.latest.lock() {
            if !latest.get_mut(&work.item.ticket.id).is_some_and(|gate| {
                gate.accept_publish(work.item.generation, work.item.view_revision)
            }) {
                let _ = inner
                    .registry
                    .transition(work.item.ticket.id, MediaAssetState::Stale);
                continue;
            }
        }
        if inner
            .registry
            .transition(work.item.ticket.id, MediaAssetState::Transforming)
            .is_err()
        {
            continue;
        }
        let rendered = render_decoded(work.image, work.metadata.orientation, &work.item.spec);
        let pin = match work.item.priority {
            MediaPriority::Current | MediaPriority::SettledCurrent => MediaPin::Hard,
            MediaPriority::Neighbor => MediaPin::Soft,
            MediaPriority::Thumbnail => MediaPin::None,
        };
        if let Ok(mut caches) = inner.caches.lock() {
            caches
                .encoded
                .insert(work.item.ticket.id, work.encoded.clone());
            caches.decoded.insert_rendered(
                work.item.ticket.id,
                rendered.width,
                rendered.height,
                rendered.rgba,
                pin,
            );
        }
        let _ = inner.registry.publish_ready(
            work.item.ticket.id,
            work.item.ticket.revision,
            work.encoded,
        );
    }
}

impl Drop for MediaWorkerPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// ---------------------------------------------------------------------------
// Metadata-only viewer sessions + the runtime aggregate
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
    /// Kept for debug/inspection of the session origin; deliberately not
    /// exposed through any public read path (verbatim migration shape).
    #[allow(dead_code)]
    root: PathBuf,
    entries: Vec<MediaSessionEntry>,
    selected: usize,
    generation: u64,
}

// Session ids stay unique for the whole process (monotonic global counter),
// matching the pre-PLAN-811 behavior of the single shared session table.
static MEDIA_SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

struct RuntimeInner {
    registry: MediaAssetRegistry,
    workers: MediaWorkerPool,
    sessions: Mutex<VecDeque<MediaSession>>,
    /// Closing latch (review F811-R1-01/02): set by shutdown/drop before
    /// anything else. Producer threads (directory scan, session populate)
    /// poll it between items and exit; admission paths check it before
    /// spawning. Lock-free so producers never block on teardown.
    closing: AtomicBool,
    /// Tracked producer threads (scan/populate). Handles are drained and
    /// joined in shutdown — a detached producer outliving a closed runtime
    /// would keep mutating its registry. Registration happens under the same
    /// mutex shutdown uses for draining, so a producer is either joined or
    /// never spawned (no registration race).
    producers: Mutex<Vec<JoinHandle<()>>>,
    /// Teardown coordination (review R4): `closing` kills admission the
    /// instant ANY shutdown call starts; `shutdown_started` gives the FIRST
    /// caller exclusive execution (no duplicate Stop); `shutdown_done` plus
    /// the condvar make every later caller (repeat calls, Drop after a
    /// timed-out attempt) WAIT for the same completion — shutdown()
    /// returning always means fully closed, never "closing in progress".
    shutdown_started: AtomicBool,
    shutdown_done: AtomicBool,
    shutdown_finish: Mutex<()>,
    shutdown_finish_cv: Condvar,
}

impl RuntimeInner {
    // Shared queueing core. Callers that only hold `Arc<RuntimeInner>`
    // (detached scan/populate threads) must use these instead of
    // constructing a temporary `MediaRuntime` — dropping an aggregate
    // value would shut the runtime down.
    fn queue_rendition(
        &self,
        path: PathBuf,
        spec: RenditionSpec,
        priority: MediaPriority,
        revision: u64,
    ) -> MediaAssetTicket {
        let key = MediaAssetKey {
            source_fingerprint: path.to_string_lossy().into_owned(),
            orientation: MediaOrientation::Normal,
            rendition: spec.clone(),
            revision,
        };
        let ticket = self.registry.queue(key, MediaMetadata::default());
        if !self
            .workers
            .submit_path(ticket, path, spec, priority, revision, revision)
        {
            let _ = self.registry.fail(
                ticket.id,
                MediaAssetError::Rejected("media queue is full".into()),
            );
        }
        ticket
    }

    fn queue_path(&self, path: PathBuf, priority: MediaPriority) -> MediaAssetTicket {
        let key = MediaAssetKey {
            source_fingerprint: path.to_string_lossy().into_owned(),
            orientation: MediaOrientation::Normal,
            rendition: RenditionSpec::original(),
            revision: 1,
        };
        let ticket = self.registry.queue(key, MediaMetadata::default());
        if !self
            .workers
            .submit_path(ticket, path, RenditionSpec::original(), priority, 0, 1)
        {
            let _ = self.registry.fail(
                ticket.id,
                MediaAssetError::Rejected("media queue is full".into()),
            );
        }
        ticket
    }
}

/// Aggregate media runtime: one registry + one bounded worker pool + the
/// host-owned session table, sharing a single lifetime owner (PLAN-811).
///
/// The process default is [`global_media_runtime`] — VM natives, the HTTP
/// data plane, and renderers all hit that single shared instance and ticket
/// namespace, exactly as the pre-split globals did. A locally constructed
/// `MediaRuntime` isolates its own registry/tickets/sessions for controllable
/// lifetimes and tests; its [`MediaRuntime::shutdown`] (or drop) stops new
/// work, joins the owned worker threads, terminal-izes every registry entry
/// (late worker callbacks are rejected by the state machine — a closed
/// session cannot be resurrected), and drops the session table.
pub struct MediaRuntime {
    inner: Arc<RuntimeInner>,
}

impl MediaRuntime {
    /// Builds an owned runtime whose registry uses the given unreferenced
    /// ticket TTL (the process default uses 30s).
    pub fn new(ttl: Duration) -> Self {
        let registry = MediaAssetRegistry::new(ttl);
        let workers = MediaWorkerPool::with_registry(registry.clone());
        Self {
            inner: Arc::new(RuntimeInner {
                registry,
                workers,
                sessions: Mutex::new(VecDeque::new()),
                closing: AtomicBool::new(false),
                producers: Mutex::new(Vec::new()),
                shutdown_started: AtomicBool::new(false),
                shutdown_done: AtomicBool::new(false),
                shutdown_finish: Mutex::new(()),
                shutdown_finish_cv: Condvar::new(),
            }),
        }
    }

    pub fn registry(&self) -> &MediaAssetRegistry {
        &self.inner.registry
    }

    pub fn worker_pool(&self) -> &MediaWorkerPool {
        &self.inner.workers
    }

    /// Whether this runtime has been shut down (producers reaped, workers
    /// joined, registry closed). The process-default runtime never closes.
    pub fn is_closed(&self) -> bool {
        self.inner.closing.load(Ordering::Acquire)
    }

    /// Terminates this runtime's owned work in a fixed order: the closing
    /// latch first (scan/open become no-ops), then every tracked producer
    /// thread (in-flight directory scan / session populate finish at most
    /// their current item and exit — `shutdown` returning means they are
    /// reaped), then the decode/resize lanes, then the registry (every
    /// entry terminal; `pending_count() == 0`; post-close queueing returns
    /// dead tickets), and finally the session table. Late worker callbacks
    /// are rejected by the state machine — closed resources cannot be
    /// resurrected. Idempotent; no lock is held across a join.
    pub fn shutdown(&self) {
        shutdown_runtime_inner(&self.inner);
    }

    fn queue_rendition(
        &self,
        path: PathBuf,
        spec: RenditionSpec,
        priority: MediaPriority,
        revision: u64,
    ) -> MediaAssetTicket {
        self.inner.queue_rendition(path, spec, priority, revision)
    }

    pub fn queue_path(
        &self,
        path: impl Into<PathBuf>,
        priority: MediaPriority,
    ) -> MediaAssetTicket {
        self.inner.queue_path(path.into(), priority)
    }

    /// Queues a small square thumbnail rendition for one image file (PLAN-023
    /// `auto.image.thumb` runtime). Returns the opaque media URI for the
    /// rendition, or "" when the path is missing, unreadable, or has an
    /// unsupported extension — callers render their icon fallback off "".
    ///
    /// The registry queue takes a reference on dedup hit, and file-manager-style
    /// callers never hold tickets, so the reference is released immediately: the
    /// entry rides its expiry grace (decode finishes in milliseconds) and
    /// stays evictable afterwards, while the decoded-pixels LRU keeps the URI
    /// rendering until budget pressure evicts it (self-healing on re-navigation).
    pub fn queue_thumbnail(&self, path: impl Into<PathBuf>, size: i32) -> String {
        let path = path.into();
        if !supported_media_path(&path) {
            return String::new();
        }
        if !path.is_file() {
            return String::new();
        }
        let edge = size.clamp(16, 1024) as u32;
        let spec = RenditionSpec {
            width: edge,
            height: edge,
            rotation_degrees: 0,
            quality: 85,
            original_pixels: false,
        };
        let ticket = self.queue_rendition(path, spec, MediaPriority::Thumbnail, 1);
        self.inner.registry.release(ticket.id);
        media_uri(ticket)
    }

    /// Starts a directory scan away from VM/UI handler threads. Each
    /// discovered supported file is submitted to the bounded media executor;
    /// the call itself returns immediately and never exposes source paths to
    /// the control plane. The scan thread is tracked: shutdown joins it (it
    /// checks the closing latch between items), and a closed runtime admits
    /// no new scan.
    pub fn scan_directory(&self, path: impl Into<PathBuf>) {
        if self.is_closed() {
            return;
        }
        let root = path.into();
        spawn_tracked_producer(&self.inner, "media-directory-scan", move |inner| {
            let Ok(entries) = std::fs::read_dir(root) else {
                return;
            };
            for entry in entries.flatten() {
                if inner.closing.load(Ordering::Acquire) {
                    return;
                }
                let path = entry.path();
                if path.is_file() {
                    inner.queue_path(path, MediaPriority::Thumbnail);
                }
            }
        });
    }

    /// Open a file or directory without blocking the caller on filesystem
    /// I/O. The returned id is safe to keep in Auto model state. A closed
    /// runtime admits no new session and returns `""` (callers treat it as
    /// unknown, matching `close_session`/snapshot semantics for dead ids).
    pub fn open_session(&self, root: impl Into<PathBuf>) -> String {
        if self.is_closed() {
            return String::new();
        }
        let root = root.into();
        let id = format!(
            "image-session-{}",
            MEDIA_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        // The session-table insertion shares ONE synchronization domain with
        // shutdown's clear (review F811-R2-01): shutdown sets `closing` before
        // it drains/clears, and this re-check happens under the same mutex it
        // clears under — so an open that raced past the entry check either
        // inserts BEFORE the clear (row removed by it) or observes `closing`
        // here and refuses. No post-shutdown row can survive.
        {
            let mut sessions = match self.inner.sessions.lock() {
                Ok(sessions) => sessions,
                Err(_) => return String::new(),
            };
            if self.inner.closing.load(Ordering::Acquire) {
                return String::new();
            }
            if sessions.len() >= MAX_MEDIA_SESSIONS {
                if let Some(evicted) = sessions.pop_front() {
                    for entry in evicted.entries {
                        self.inner.registry.release(entry.ticket.id);
                    }
                }
            }
            sessions.push_back(MediaSession {
                id: id.clone(),
                root: root.clone(),
                entries: Vec::new(),
                selected: 0,
                generation: 1,
            });
        }
        let worker_id = id.clone();
        if !spawn_tracked_producer(&self.inner, "media-session-open", move |inner| {
            populate_media_session(inner, worker_id, root)
        }) {
            // No populate thread can exist for this row (spawn refused — the
            // runtime closed underneath us, or the OS refused the thread).
            // Remove the row so the id never answers as a live session.
            if let Ok(mut sessions) = self.inner.sessions.lock() {
                sessions.retain(|session| session.id != id);
            }
            return String::new();
        }
        id
    }

    fn snapshot_locked(session: &MediaSession) -> String {
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
        // `id`, `uri`, and `name` are generated/filename values.  Escape the
        // two user-visible strings so this metadata response remains valid JSON.
        let escape = |value: &str| value.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "{{\"session\":\"{}\",\"selected\":{},\"count\":{},\"generation\":{},\"name\":\"{}\",\"uri\":\"{}\"}}",
            escape(&session.id), session.selected, session.entries.len(), session.generation, escape(name), escape(&uri)
        )
    }

    /// Return a metadata-only snapshot for a session.
    pub fn session_snapshot(&self, session_id: &str) -> String {
        self.inner
            .sessions
            .lock()
            .ok()
            .and_then(|sessions| {
                sessions
                    .iter()
                    .find(|session| session.id == session_id)
                    .map(Self::snapshot_locked)
            })
            .unwrap_or_else(|| "{\"error\":\"unknown_session\"}".into())
    }

    /// Return only the currently selected opaque media URI. This lets a front
    /// model bind `ImageSurface.src` without parsing JSON or receiving paths.
    pub fn session_uri(&self, session_id: &str) -> String {
        self.inner
            .sessions
            .lock()
            .ok()
            .and_then(|sessions| {
                sessions
                    .iter()
                    .find(|session| session.id == session_id)
                    .and_then(|session| {
                        session
                            .entries
                            .get(session.selected)
                            .map(|entry| media_uri(entry.ticket))
                    })
            })
            .unwrap_or_default()
    }

    /// Return bounded display names for the current directory session. Names
    /// are metadata only; the corresponding media URI is still resolved by
    /// index.
    pub fn session_names(&self, session_id: &str) -> Vec<String> {
        self.inner
            .sessions
            .lock()
            .ok()
            .and_then(|sessions| {
                sessions
                    .iter()
                    .find(|session| session.id == session_id)
                    .map(|session| {
                        session
                            .entries
                            .iter()
                            .map(|entry| entry.name.clone())
                            .collect()
                    })
            })
            .unwrap_or_default()
    }

    /// Move the selection while preserving a bounded, host-side session table.
    pub fn navigate_session(&self, session_id: &str, delta: i32) -> String {
        let Ok(mut sessions) = self.inner.sessions.lock() else {
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
        Self::snapshot_locked(session)
    }

    /// Queue a viewport rendition for the selected asset.  The original
    /// ticket is retained until the replacement is published, so a stale
    /// request cannot invalidate the currently displayed media.
    pub fn request_view(
        &self,
        session_id: &str,
        index: i32,
        width: i32,
        height: i32,
        quality: i32,
    ) -> String {
        let (path, generation) = {
            let Ok(mut sessions) = self.inner.sessions.lock() else {
                return "{\"error\":\"session_lock\"}".into();
            };
            let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) else {
                return "{\"error\":\"unknown_session\"}".into();
            };
            if session.entries.is_empty() {
                return Self::snapshot_locked(session);
            }
            session.selected = index.clamp(0, session.entries.len() as i32 - 1) as usize;
            session.generation = session.generation.saturating_add(1);
            (
                session.entries[session.selected].path.clone(),
                session.generation,
            )
        };
        let spec = RenditionSpec {
            width: width.max(0) as u32,
            height: height.max(0) as u32,
            rotation_degrees: 0,
            quality: quality.clamp(1, 100) as u8,
            original_pixels: false,
        };
        let ticket = self.queue_rendition(path, spec, MediaPriority::SettledCurrent, generation);
        if let Ok(mut sessions) = self.inner.sessions.lock() {
            if let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) {
                let old_ticket = session.entries[session.selected].ticket;
                session.entries[session.selected].ticket = ticket;
                // The session owns exactly one reference per entry. Release the
                // superseded ticket after swapping it so repeated view requests
                // cannot grow the registry indefinitely while queued work settles.
                if old_ticket.id != ticket.id {
                    self.inner.registry.release(old_ticket.id);
                }
                return Self::snapshot_locked(session);
            }
        }
        self.inner.registry.release(ticket.id);
        "{\"error\":\"unknown_session\"}".into()
    }

    /// Release all tickets held by a session.  Returns false for an already
    /// closed or unknown id, allowing generated APIs to report deterministic
    /// close state.
    pub fn close_session(&self, session_id: &str) -> bool {
        let Some(session) = self.inner.sessions.lock().ok().and_then(|mut sessions| {
            sessions
                .iter()
                .position(|session| session.id == session_id)
                .and_then(|index| sessions.remove(index))
        }) else {
            return false;
        };
        for entry in session.entries {
            self.inner.registry.release(entry.ticket.id);
        }
        true
    }

    /// Session stats answer with the runtime registry's global counters (the
    /// historical `auto.image.session_stats` semantics — the session argument
    /// is deliberately ignored).
    pub fn session_stats(&self, _session_id: &str) -> String {
        let stats = self.inner.registry.stats();
        format!(
            "{{\"queued\":{},\"running\":{},\"completed\":{},\"dropped\":{},\"encoded_bytes\":{},\"decoded_bytes\":{},\"cache_hits\":{},\"cache_misses\":{}}}",
            stats.queued, stats.running, stats.completed, stats.dropped, stats.encoded_bytes, stats.decoded_bytes, stats.cache_hits, stats.cache_misses
        )
    }
}

impl Drop for MediaRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Shutdown body shared by `MediaRuntime::shutdown` and the bounded-test
/// runner (operates on the shared `Arc<RuntimeInner>`).
///
/// Contract (review R4): admission (`closing`) dies the instant ANY caller
/// starts; the FIRST caller exclusively executes the teardown (no duplicate
/// Stop behind a full resize channel); every LATER caller waits for the
/// same completion — so `shutdown()` returning always means the runtime is
/// fully closed (producers reaped, workers joined, registry terminal,
/// sessions dropped), for concurrent duplicates and Drop-after-timeout
/// alike.
fn shutdown_runtime_inner(inner: &RuntimeInner) {
    inner.closing.store(true, Ordering::Release);
    if inner.shutdown_started.swap(true, Ordering::AcqRel) {
        // A concurrent/earlier caller owns the teardown: wait for ITS
        // completion (bounded by the in-flight work it is draining). The
        // waiter holds no lock the executor needs.
        let mut guard = inner
            .shutdown_finish
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !inner.shutdown_done.load(Ordering::Acquire) {
            guard = inner
                .shutdown_finish_cv
                .wait(guard)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        return;
    }
    // Exclusive teardown:
    // 1. Reap tracked producers (join outside any lock).
    let producers = inner
        .producers
        .lock()
        .map(|mut handles| std::mem::take(&mut *handles))
        .unwrap_or_default();
    for producer in producers {
        let _ = producer.join();
    }
    // 2. Stop the decode/resize lanes and join their threads.
    inner.workers.shutdown();
    // 3. Terminal-ize every registry entry and close admission.
    inner.registry.shutdown();
    // 4. Drop the session table.
    if let Ok(mut sessions) = inner.sessions.lock() {
        sessions.clear();
    }
    // 5. Publish completion and release every waiter.
    inner.shutdown_done.store(true, Ordering::Release);
    let _guard = inner.shutdown_finish.lock();
    inner.shutdown_finish_cv.notify_all();
}

/// Spawns a tracked producer thread for the runtime (review F811-R1-01).
/// Registration and the closing check share the producers mutex shutdown
/// drains under, so every producer is either joined at close or never
/// spawned — a registration race cannot leave a live producer unreaped.
/// The body receives the `Arc<RuntimeInner>` and must poll
/// `inner.closing` between units of work.
fn spawn_tracked_producer(
    inner: &Arc<RuntimeInner>,
    name: &str,
    body: impl FnOnce(Arc<RuntimeInner>) + Send + 'static,
) -> bool {
    let mut producers = match inner.producers.lock() {
        Ok(producers) => producers,
        Err(_) => return false,
    };
    if inner.closing.load(Ordering::Acquire) {
        return false;
    }
    let worker = inner.clone();
    match thread::Builder::new()
        .name(name.to_string())
        .spawn(move || body(worker))
    {
        Ok(handle) => {
            producers.push(handle);
            true
        }
        Err(_) => false,
    }
}

fn append_session_entry(inner: &Arc<RuntimeInner>, session_id: &str, path: PathBuf) {
    if !supported_media_path(&path) {
        return;
    }
    let ticket = inner.queue_path(path.clone(), MediaPriority::Thumbnail);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("image")
        .to_owned();
    let mut sessions = match inner.sessions.lock() {
        Ok(sessions) => sessions,
        Err(_) => {
            inner.registry.release(ticket.id);
            return;
        }
    };
    let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) else {
        // Session closed while the populate worker was scanning: release the
        // ticket immediately — a late callback never resurrects a closed
        // session or re-feeds another one.
        inner.registry.release(ticket.id);
        return;
    };
    if session.entries.len() >= MAX_SESSION_ENTRIES {
        inner.registry.release(ticket.id);
        return;
    }
    session
        .entries
        .push(MediaSessionEntry { path, ticket, name });
}

fn populate_media_session(inner: Arc<RuntimeInner>, session_id: String, root: PathBuf) {
    // All filesystem work happens on this tracked worker, never on a VM/UI
    // handler thread.  Sorting makes directory navigation deterministic.
    // The closing latch is polled between entries so shutdown can reap an
    // in-flight populate (review F811-R1-01).
    let mut paths = Vec::new();
    if root.is_file() {
        paths.push(root);
    } else if root.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&root) {
            paths.extend(
                entries
                    .flatten()
                    .map(|entry| entry.path())
                    .filter(|path| path.is_file()),
            );
            paths.sort();
        }
    }
    for path in paths {
        if inner.closing.load(Ordering::Acquire) {
            return;
        }
        append_session_entry(&inner, &session_id, path);
    }
}

/// The process-default shared media runtime (30s unreferenced ticket TTL):
/// one registry, one worker pool, one session table behind every legacy free
/// function, VM native, HTTP fast path, and renderer hook. Dropping a VM
/// never shuts this shared instance down.
pub fn global_media_runtime() -> &'static MediaRuntime {
    static RUNTIME: std::sync::LazyLock<MediaRuntime> =
        std::sync::LazyLock::new(|| MediaRuntime::new(Duration::from_secs(30)));
    &RUNTIME
}

pub fn global_media_worker_pool() -> &'static MediaWorkerPool {
    global_media_runtime().worker_pool()
}

pub fn queue_media_path(path: impl Into<PathBuf>, priority: MediaPriority) -> MediaAssetTicket {
    global_media_runtime().queue_path(path, priority)
}

pub fn scan_media_directory(path: impl Into<PathBuf>) {
    global_media_runtime().scan_directory(path);
}

pub fn open_media_session(root: impl Into<PathBuf>) -> String {
    global_media_runtime().open_session(root)
}

pub fn media_session_snapshot(session_id: &str) -> String {
    global_media_runtime().session_snapshot(session_id)
}

pub fn media_session_uri(session_id: &str) -> String {
    global_media_runtime().session_uri(session_id)
}

pub fn media_session_names(session_id: &str) -> Vec<String> {
    global_media_runtime().session_names(session_id)
}

pub fn navigate_media_session(session_id: &str, delta: i32) -> String {
    global_media_runtime().navigate_session(session_id, delta)
}

pub fn request_media_view(
    session_id: &str,
    index: i32,
    width: i32,
    height: i32,
    quality: i32,
) -> String {
    global_media_runtime().request_view(session_id, index, width, height, quality)
}

pub fn close_media_session(session_id: &str) -> bool {
    global_media_runtime().close_session(session_id)
}

pub fn media_session_stats(session_id: &str) -> String {
    global_media_runtime().session_stats(session_id)
}

pub fn queue_media_thumbnail(path: impl Into<PathBuf>, size: i32) -> String {
    global_media_runtime().queue_thumbnail(path, size)
}

/// Resolves already-decoded pixels for the native renderer. The lookup never
/// opens a file or decodes encoded bytes; only the resize lane populates this
/// bounded cache.
pub fn resolve_media_pixels(uri: &str) -> Option<(u32, u32, Arc<[u8]>)> {
    if !uri.starts_with("/api/__auto/media/") {
        return None;
    }
    let (id, _) = parse_media_path(uri)?;
    global_media_worker_pool().decoded_pixels(id)
}

/// Resolve a media URI for rendering, also returning the asset id (the
/// renderer keys its texture-Handle cache on it).
pub fn resolve_media_render(uri: &str) -> Option<(MediaAssetId, u32, u32, Arc<[u8]>)> {
    if !uri.starts_with("/api/__auto/media/") {
        return None;
    }
    let (id, _) = parse_media_path(uri)?;
    global_media_worker_pool()
        .decoded_pixels(id)
        .map(|(w, h, px)| (id, w, h, px))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::{
        checked_rgba_bytes, MediaAssetKey, MediaAssetRegistry, MediaAssetState, MediaLookup,
        MediaMetadata, MediaOrientation, MediaPriority, MediaPriorityQueue, RenditionSpec,
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
        assert_eq!(
            registry.lookup(ticket.id, ticket.revision),
            MediaLookup::Pending
        );

        registry
            .transition(ticket.id, MediaAssetState::Reading)
            .unwrap();
        registry
            .transition(ticket.id, MediaAssetState::Decoding)
            .unwrap();
        registry
            .transition(ticket.id, MediaAssetState::Transforming)
            .unwrap();
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
        assert_eq!(
            registry.lookup(ticket.id, ticket.revision),
            MediaLookup::Expired
        );
        assert_eq!(
            registry.lookup(ticket.id, ticket.revision + 1),
            MediaLookup::Missing
        );
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

        assert!(
            cache.get(first).is_some(),
            "recently accessed entry is retained"
        );
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
        cache.insert(
            neighbor,
            Arc::<[u8]>::from([1, 2, 3, 4]),
            super::MediaPin::Soft,
        );
        cache.insert(
            current,
            Arc::<[u8]>::from([5, 6, 7, 8, 9, 10]),
            super::MediaPin::Hard,
        );
        assert!(cache.contains(current));
        assert_eq!(
            cache.used_bytes(),
            6,
            "hard-pinned current can temporarily exceed budget"
        );
        assert!(
            !cache.contains(neighbor),
            "publish reclaims non-current content"
        );

        cache.insert(
            replacement,
            Arc::<[u8]>::from([11, 12]),
            super::MediaPin::None,
        );
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
        let workers = super::MediaWorkerPool::new();
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
        let workers = super::MediaWorkerPool::with_registry(registry.clone());
        assert!(workers.submit_bytes(
            ticket,
            Arc::from(bytes.into_inner()),
            RenditionSpec::viewport(1, 1),
            MediaPriority::Current,
            1,
            1
        ));
        assert_eq!(
            registry.wait_for_terminal(ticket.id, Duration::from_secs(2)),
            Some(MediaAssetState::Ready)
        );
        assert!(matches!(
            registry.lookup(ticket.id, ticket.revision),
            MediaLookup::Ready(_)
        ));
        workers.shutdown();
    }

    #[test]
    fn viewer_session_opens_directory_and_exposes_only_metadata() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures");
        let session = super::open_media_session(root);
        let deadline = Instant::now() + Duration::from_secs(2);
        while super::media_session_names(&session).is_empty() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        let names = super::media_session_names(&session);
        assert!(names.iter().any(|name| name == "rgb-1x1.png"));
        let snapshot = super::media_session_snapshot(&session);
        assert!(snapshot.contains("\"count\":"));
        assert!(
            !snapshot.contains("fixtures"),
            "snapshot must not leak source paths"
        );
        let _ = super::request_media_view(&session, 0, 32, 32, 90);
        let uri = super::media_session_uri(&session);
        assert!(uri.is_empty() || uri.starts_with("/api/__auto/media/"));
        assert!(super::close_media_session(&session));
        assert!(!super::close_media_session(&session));
    }

    #[test]
    fn media_change_generation_advances_on_queue_and_terminal() {
        use super::{media_change_generation, queue_media_path, MediaPriority};
        let before = media_change_generation();
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures/rgb-1x1.png");
        let ticket = queue_media_path(&fixture, MediaPriority::Current);
        assert_ne!(
            media_change_generation(),
            before,
            "queue must bump generation"
        );
        let mid = media_change_generation();
        let _ = super::global_media_registry().fail(ticket.id, super::MediaAssetError::Corrupt);
        assert_ne!(
            media_change_generation(),
            mid,
            "terminal failure must bump generation"
        );
    }

    #[test]
    fn thumbnail_queue_rejects_unsupported_and_queues_supported_paths() {
        // Missing path (supported ext) and unsupported ext both degrade to ""
        // so callers can fall back to their icon branch.
        assert_eq!(super::queue_media_thumbnail("not-a-file.png", 64), "");
        assert_eq!(super::queue_media_thumbnail("fixture.txt", 64), "");

        // Supported fixture queues a Thumbnail-priority rendition and returns
        // an opaque media URI that never leaks the source path.
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures/rgb-1x1.png");
        let uri = super::queue_media_thumbnail(&fixture, 64);
        assert!(uri.starts_with("/api/__auto/media/"));
        assert!(!uri.contains("fixtures"));
        let route = uri.strip_prefix("/api/__auto/media/").unwrap();
        let (id, revision) = route.split_once('/').unwrap();
        assert_eq!(id.len(), 32);
        assert_eq!(revision, "1");
    }

    // PLAN-811 T-03: owned-runtime lifecycle invariants (shutdown closure,
    // close-vs-scan race, bounded shutdown under a full ingress queue, and
    // runtime isolation).

    #[test]
    fn owned_runtime_shutdown_is_observable_and_rejects_late_publishes() {
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        let registry = runtime.registry().clone();
        let ticket = registry.queue(
            MediaAssetKey {
                source_fingerprint: "shutdown-fixture".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::original(),
                revision: 1,
            },
            MediaMetadata::default(),
        );
        assert!(registry.pending_count() > 0, "queued asset is pending");
        runtime.shutdown();
        // Terminal-ized: the pump gate observes zero pending work.
        assert_eq!(
            registry.pending_count(),
            0,
            "shutdown must drive every entry to a terminal state"
        );
        assert!(registry.is_shutdown());
        // A late publish cannot resurrect the closed asset.
        assert!(matches!(
            registry.publish_ready(ticket.id, ticket.revision, Arc::<[u8]>::from([9])),
            Err(super::MediaAssetError::Rejected(_))
        ));
        // New submissions are refused after shutdown.
        let again = registry.queue(
            MediaAssetKey {
                source_fingerprint: "shutdown-fixture-2".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::original(),
                revision: 1,
            },
            MediaMetadata::default(),
        );
        assert!(!runtime.worker_pool().submit_bytes(
            again,
            Arc::from(Vec::new()),
            RenditionSpec::original(),
            MediaPriority::Current,
            0,
            1
        ));
        // Session table is dropped: subsequent ops report unknown_session.
        assert_eq!(
            runtime.session_snapshot("image-session-999999"),
            "{\"error\":\"unknown_session\"}"
        );
    }

    #[test]
    fn close_during_populate_releases_tickets_and_never_resurrects() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures");
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        let session = runtime.open_session(&root);
        // Close immediately: the populate worker is still scanning. Late
        // appends must release their tickets instead of resurrecting the
        // session.
        assert!(runtime.close_session(&session));
        assert!(!runtime.close_session(&session), "double close is false");
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            assert_eq!(
                runtime.session_snapshot(&session),
                "{\"error\":\"unknown_session\"}",
                "closed session never comes back"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        runtime.shutdown();
    }

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::new(width, height))
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    /// Bounded runner for shutdown-style teardown checks (review R2-02):
    /// the work runs on a DETACHED thread; on timeout the harness fails
    /// immediately WITHOUT joining or otherwise waiting for the stuck work
    /// (the thread is leaked into the test process — a failing test may
    /// leave it running, it must never keep the harness hanging).
    fn bounded_teardown<F: FnOnce() + Send + 'static>(work: F, limit: Duration) {
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        let worker = std::thread::spawn(move || {
            work();
            let _ = done_tx.send(());
        });
        if done_rx.recv_timeout(limit).is_err() {
            panic!(
                "teardown did not converge within the external deadline ({limit:?}) —                  this failure intentionally does NOT join the stuck worker"
            );
        }
        let _ = worker.join();
    }

    /// shutdown_under_external_deadline — on the bounded runner. The
    /// runtime shares its state through `Arc<RuntimeInner>`, so the
    /// detached teardown thread operates on a clone while the test keeps
    /// its single owning `MediaRuntime` (shutdown is idempotent and &self).
    fn shutdown_under_external_deadline(runtime: &super::MediaRuntime, limit: Duration) {
        let inner = runtime.inner.clone();
        bounded_teardown(move || super::shutdown_runtime_inner(&inner), limit);
    }

    #[test]
    fn bounded_teardown_negative_control_fails_within_the_deadline() {
        // Review R2-02/R3: a task exceeding the deadline must turn the
        // harness red WITHIN the deadline window (plus scheduling slack) —
        // not when the stuck task eventually finishes.
        let started = Instant::now();
        let result = std::panic::catch_unwind(|| {
            bounded_teardown(
                || std::thread::sleep(Duration::from_millis(1500)),
                Duration::from_millis(100),
            );
        });
        let elapsed = started.elapsed();
        assert!(result.is_err(), "over-deadline task must fail the harness");
        assert!(
            elapsed < Duration::from_millis(800),
            "failure must arrive within the deadline window, got {elapsed:?}"
        );
    }

    /// Result-returning variant for negative controls (review R3): the
    /// harness learns about the timeout WITHOUT panicking, so the control
    /// can then PROVE it reclaimed its own worker and bytes.
    fn bounded_teardown_result<F: FnOnce() + Send + 'static>(
        work: F,
        limit: Duration,
    ) -> Result<(), Duration> {
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        let worker = std::thread::spawn(move || {
            work();
            let _ = done_tx.send(());
        });
        let started = Instant::now();
        if done_rx.recv_timeout(limit).is_err() {
            return Err(started.elapsed());
        }
        let _ = worker.join();
        Ok(())
    }

    #[test]
    fn bounded_teardown_cooperative_failure_reclaims_worker_and_bytes() {
        // Review R3: the timeout exit must RECLAIM the harness's own worker
        // thread and bytes. A cancellable task times out, reports Err
        // within budget, is cancelled, and joins with the bytes observed
        // dead — failure AND cleanup both bounded.
        use std::sync::atomic::{AtomicBool, Ordering};
        let cancel = std::sync::Arc::new(AtomicBool::new(false));
        let bytes = std::sync::Arc::new(vec![7u8; 4096]);
        let weak = Arc::downgrade(&bytes);
        let (worker_done_tx, worker_done_rx) = std::sync::mpsc::channel::<()>();
        let cancel_for_thread = cancel.clone();
        let bytes_for_thread = bytes.clone();
        let verdict = bounded_teardown_result(
            move || {
                while !cancel_for_thread.load(Ordering::Acquire) {
                    std::thread::sleep(Duration::from_millis(25));
                }
                drop(bytes_for_thread);
                let _ = worker_done_tx.send(());
            },
            Duration::from_millis(100),
        );
        assert!(verdict.is_err(), "over-deadline work reports Err");
        assert!(
            verdict.clone().unwrap_err() < Duration::from_secs(1),
            "failure reported within budget"
        );
        // Reclaim: cancel, then bounded-join with the worker's own signal.
        cancel.store(true, Ordering::Release);
        assert!(
            worker_done_rx.recv_timeout(Duration::from_secs(2)).is_ok(),
            "cancelled worker exits within the reclaim window"
        );
        drop(bytes);
        assert!(
            weak.upgrade().is_none(),
            "owned bytes reclaimed after failure cleanup"
        );
    }

    #[test]
    fn bounded_teardown_real_owner_failure_still_reclaims() {
        // Review R3: the failure exit for a REAL MediaRuntime owner. A
        // large resize backlog makes shutdown take seconds; the bounded
        // helper reports Err within a short budget WITHOUT waiting. The
        // owner is then reclaimed deterministically: a second shutdown()
        // returns promptly (latch set, producers drained, join tables
        // empty — idempotent) and the runtime reaches a terminal state
        // within a measured bound even though the first attempt timed out.
        let runtime = super::MediaRuntime::new(Duration::from_secs(600));
        let registry = runtime.registry().clone();
        let payload: Arc<[u8]> = Arc::from(png_bytes(3200, 2400).into_boxed_slice());
        let target = RenditionSpec::viewport(1920, 1440);
        let mut accepted = 0usize;
        for i in 0..12u64 {
            let ticket = registry.queue(
                MediaAssetKey {
                    source_fingerprint: format!("slow-teardown-{i}"),
                    orientation: Default::default(),
                    rendition: target.clone(),
                    revision: i + 1,
                },
                MediaMetadata::default(),
            );
            if runtime.worker_pool().submit_bytes(
                ticket,
                payload.clone(),
                target.clone(),
                MediaPriority::Current,
                0,
                i + 1,
            ) {
                accepted += 1;
            }
        }
        assert!(accepted >= 6, "lane-busy workload accepted: {accepted}");
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && runtime.worker_pool().resize_backlog() < 2 {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(
            runtime.worker_pool().resize_backlog() >= 2,
            "slow teardown precondition: resize lane busy"
        );
        let inner = runtime.inner.clone();
        let verdict = bounded_teardown_result(
            move || super::shutdown_runtime_inner(&inner),
            Duration::from_millis(120),
        );
        assert!(verdict.is_err(), "slow shutdown exceeds the budget");
        // Review R4: a repeat shutdown must WAIT for the first caller's
        // completion — "shutdown returned" means fully closed. The reclaim
        // is bounded by the in-flight backlog the detached attempt drains.
        let reclaim_started = Instant::now();
        runtime.shutdown();
        let reclaim_elapsed = reclaim_started.elapsed();
        assert!(
            reclaim_elapsed < Duration::from_secs(60),
            "repeat shutdown waits for completion within the backlog bound: {reclaim_elapsed:?}"
        );
        assert!(runtime.is_closed());
        assert!(
            registry.is_shutdown(),
            "shutdown returned => registry closed (no admission window)"
        );
        let fresh = registry.queue(
            MediaAssetKey {
                source_fingerprint: "after-repeat-shutdown".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::original(),
                revision: 1,
            },
            MediaMetadata::default(),
        );
        assert_eq!(
            registry.lookup(fresh.id, fresh.revision),
            MediaLookup::Missing,
            "no live ticket after every shutdown caller returned"
        );
        let quiesce_deadline = Instant::now() + Duration::from_secs(60);
        while Instant::now() < quiesce_deadline && registry.pending_count() > 0 {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(
            registry.pending_count(),
            0,
            "real owner reaches terminal state after timed-out teardown"
        );
    }

    // PLAN-811 R4 permanent controls: concurrent duplicate shutdown
    // admission + occupancy gauge lifecycle.

    #[test]
    fn concurrent_duplicate_shutdown_returns_only_when_fully_closed() {
        // Review F811-R4-02 repro as a permanent oracle: real decode work
        // in flight, caller 1 starts shutdown, caller 2 enters AFTER the
        // closing latch is visible. Caller 2's shutdown() must NOT return
        // until the registry is closed — and after BOTH returned, a fresh
        // ticket must be dead (no admission window).
        let runtime = super::MediaRuntime::new(Duration::from_secs(600));
        let registry = runtime.registry().clone();
        let payload: Arc<[u8]> = Arc::from(png_bytes(3200, 2400).into_boxed_slice());
        let target = RenditionSpec::viewport(1920, 1440);
        let mut accepted = 0usize;
        for i in 0..12u64 {
            let ticket = registry.queue(
                MediaAssetKey {
                    source_fingerprint: format!("dup-shutdown-{i}"),
                    orientation: Default::default(),
                    rendition: target.clone(),
                    revision: i + 1,
                },
                MediaMetadata::default(),
            );
            if runtime.worker_pool().submit_bytes(
                ticket,
                payload.clone(),
                target.clone(),
                MediaPriority::Current,
                0,
                i + 1,
            ) {
                accepted += 1;
            }
        }
        assert!(accepted >= 6, "work in flight: {accepted}");
        let inner = runtime.inner.clone();
        let first = std::thread::spawn(move || {
            super::shutdown_runtime_inner(&inner);
        });
        // Wait until the closing latch is live, then run the DUPLICATE.
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && !runtime.is_closed() {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(runtime.is_closed(), "closing latch live before duplicate");
        let duplicate_started = Instant::now();
        runtime.shutdown();
        let duplicate_elapsed = duplicate_started.elapsed();
        // The duplicate waited for completion: registry closed NOW.
        assert!(
            registry.is_shutdown(),
            "duplicate shutdown returned only after full closure"
        );
        assert!(
            duplicate_elapsed < Duration::from_secs(60),
            "duplicate bounded by in-flight drain: {duplicate_elapsed:?}"
        );
        let fresh = registry.queue(
            MediaAssetKey {
                source_fingerprint: "post-duplicate".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::original(),
                revision: 1,
            },
            MediaMetadata::default(),
        );
        assert_eq!(
            registry.lookup(fresh.id, fresh.revision),
            MediaLookup::Missing,
            "no live ticket after both shutdown callers returned"
        );
        let _ = first.join();
    }

    #[test]
    fn resize_backlog_gauge_lifecycle_controls() {
        // Review F811-R4-01: the occupancy gauge must read 0 on an idle
        // runtime, transiently >0 while real Work flows, return to 0 after
        // draining, and stay 0 (no underflow panic) after shutdown — the
        // reviewer's public-API counterexample shape.
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        assert_eq!(
            runtime.worker_pool().resize_backlog(),
            0,
            "idle gauge reads zero"
        );

        // Real small renditions flow through the lane; observe a transient
        // positive reading or completion (both prove the gauge moves with
        // real Work), then drain back to exactly zero.
        let registry = runtime.registry().clone();
        let payload: Arc<[u8]> = Arc::from(png_bytes(64, 48).into_boxed_slice());
        let target = RenditionSpec::viewport(48, 36);
        let mut tickets = Vec::new();
        for i in 0..6u64 {
            let ticket = registry.queue(
                MediaAssetKey {
                    source_fingerprint: format!("gauge-{i}"),
                    orientation: Default::default(),
                    rendition: target.clone(),
                    revision: i + 1,
                },
                MediaMetadata::default(),
            );
            assert!(runtime.worker_pool().submit_bytes(
                ticket,
                payload.clone(),
                target.clone(),
                MediaPriority::Current,
                0,
                i + 1
            ));
            tickets.push(ticket);
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut saw_positive = false;
        let mut all_ready = false;
        while Instant::now() < deadline {
            let backlog = runtime.worker_pool().resize_backlog();
            saw_positive |= backlog > 0;
            if tickets
                .iter()
                .all(|t| matches!(registry.lookup(t.id, t.revision), MediaLookup::Ready(_)))
            {
                all_ready = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(all_ready, "small renditions completed");
        assert!(
            saw_positive,
            "gauge observed real Work occupancy (>0 transiently)"
        );
        assert_eq!(
            runtime.worker_pool().resize_backlog(),
            0,
            "gauge drains to exactly zero after completion"
        );

        // The reviewer's counterexample: shutdown then read — must be 0
        // with NO underflow panic (Stop counts on neither gauge).
        runtime.shutdown();
        assert_eq!(
            runtime.worker_pool().resize_backlog(),
            0,
            "gauge stays zero after shutdown (no Stop accounting, no underflow)"
        );
        // Repeated reads after close remain stable.
        for _ in 0..10 {
            assert_eq!(runtime.worker_pool().resize_backlog(), 0);
        }
    }

    #[test]
    fn resize_backlog_is_exactly_zero_after_inflight_shutdown() {
        // Review F811-R5-01 repro as a permanent oracle: real decode work
        // in flight (first item transforming, second decoding) when
        // shutdown runs. After shutdown completes — threads joined,
        // registry terminal — the gauge must read EXACTLY 0, matching the
        // true terminal state (was: stuck at 1 from a Work accepted behind
        // Stop that the lane never settled).
        for trial in 0..3 {
            let runtime = super::MediaRuntime::new(Duration::from_secs(600));
            let registry = runtime.registry().clone();
            let payload: Arc<[u8]> = Arc::from(png_bytes(3200, 2400).into_boxed_slice());
            let target = RenditionSpec::viewport(1920, 1440);
            let mut tickets = Vec::new();
            for i in 0..2u64 {
                let ticket = registry.queue(
                    MediaAssetKey {
                        source_fingerprint: format!("inflight-{trial}-{i}"),
                        orientation: Default::default(),
                        rendition: target.clone(),
                        revision: i + 1,
                    },
                    MediaMetadata::default(),
                );
                assert!(runtime.worker_pool().submit_bytes(
                    ticket,
                    payload.clone(),
                    target.clone(),
                    MediaPriority::Current,
                    0,
                    i + 1
                ));
                tickets.push(ticket);
            }
            // Wait until work is genuinely in flight (first item past
            // decode, second readable/decoding), then shut down mid-flight.
            let deadline = Instant::now() + Duration::from_secs(10);
            while Instant::now() < deadline {
                let stats = registry.stats();
                let first_busy = matches!(
                    registry.lookup(tickets[0].id, tickets[0].revision),
                    MediaLookup::Pending
                ) && stats.running >= 1;
                if first_busy && stats.queued + stats.running >= 2 {
                    break;
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            shutdown_under_external_deadline(&runtime, Duration::from_secs(60));
            assert!(runtime.is_closed());
            assert_eq!(registry.pending_count(), 0, "terminal registry");
            assert_eq!(
                runtime.worker_pool().worker_count(),
                0,
                "trial {trial}: all threads joined"
            );
            assert_eq!(
                runtime.worker_pool().resize_backlog(),
                0,
                "trial {trial}: gauge reads the TRUE terminal state (0), not a phantom Work"
            );
        }
    }

    #[test]
    fn bounded_teardown_positive_control_completes() {
        bounded_teardown(
            || std::thread::sleep(Duration::from_millis(10)),
            Duration::from_secs(5),
        );
    }

    #[test]
    fn shutdown_with_real_decode_backlog_completes_within_external_deadline() {
        // Review R2-02: PROVE the resize channel actually fills, not just
        // that the ingress queue accepted work. Large renditions (decode
        // 3200x2400, resize to 1920x1440 Lanczos3) make the single resize
        // lane the bottleneck: items wait in the sync_channel(8) while in
        // the Decoding state, so the sampled `running` count (Reading+
        // Decoding+Transforming) exceeding the 2 decode workers + 1 resize
        // lane by more than the channel capacity proves channel saturation.
        //
        // Review R6 (F811-R2-02 fixture): a fixed submit burst cannot
        // establish that state — the ingress queue rejects same-priority
        // work beyond CAPACITY (8) until decode workers drain it, so a
        // burst legally gets only 8 items accepted, and 8 accepted items
        // can never observe fullness (max composition: 2 decoding + 1
        // transforming + 5 channel waiters). Replenish instead: keep
        // offering distinct items — bounded by item budget and wall clock,
        // one offer per poll, rejected offers retried — until the
        // full-channel state is OBSERVED, so the precondition no longer
        // depends on how the submitting thread races the workers' pops.
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        let registry = runtime.registry().clone();
        let payload: Arc<[u8]> = Arc::from(png_bytes(3200, 2400).into_boxed_slice());
        let target = RenditionSpec::viewport(1920, 1440);
        const ITEM_BUDGET: usize = 24;
        let full_channel = MediaPriorityQueue::CAPACITY as u64;
        let establish_deadline = Instant::now() + Duration::from_secs(20);
        let mut next_index = 0u64;
        let mut accepted = 0usize;
        let mut rejected_offers = 0usize;
        let mut peak_running = 0usize;
        let mut peak_backlog = 0u64;
        let mut last_running = 0usize;
        let mut last_backlog = 0u64;
        while Instant::now() < establish_deadline {
            last_running = registry.stats().running;
            last_backlog = runtime.worker_pool().resize_backlog();
            peak_running = peak_running.max(last_running);
            peak_backlog = peak_backlog.max(last_backlog);
            if peak_running > full_channel as usize && peak_backlog >= full_channel {
                break;
            }
            if (next_index as usize) < ITEM_BUDGET {
                let ticket = registry.queue(
                    MediaAssetKey {
                        source_fingerprint: format!("saturate-{next_index}"),
                        orientation: Default::default(),
                        rendition: target.clone(),
                        revision: next_index + 1,
                    },
                    MediaMetadata::default(),
                );
                if runtime.worker_pool().submit_bytes(
                    ticket,
                    payload.clone(),
                    target.clone(),
                    MediaPriority::Current,
                    0,
                    next_index + 1,
                ) {
                    accepted += 1;
                    next_index += 1;
                } else {
                    // Ingress full: decode workers are busy upstream, so a
                    // slot will open; retry the same index after the poll.
                    rejected_offers += 1;
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(
            accepted > full_channel as usize,
            "precondition: only {accepted} items accepted ({rejected_offers} offers \
             rejected, budget {ITEM_BUDGET}) — observing {full_channel} channel slots \
             plus the resizing item requires more accepted items than the ingress \
             capacity alone can hold"
        );
        assert!(
            peak_running > full_channel as usize,
            "in-flight states observed: sampled running peak {peak_running}, accepted \
             {accepted}, rejected offers {rejected_offers}, backlog peak \
             {peak_backlog}, last sample running {last_running} backlog {last_backlog} \
             (needs 2 active decoders + 1 resizing + a FULL channel of waiters)"
        );
        assert!(
            peak_backlog >= full_channel,
            "resize channel observed FULL via occupancy gauges: sampled backlog peak \
             {peak_backlog} of capacity {full_channel}, accepted {accepted}, running \
             peak {peak_running}, last sample running {last_running} backlog \
             {last_backlog}"
        );
        // Wait-for-completion semantics: the bounded shutdown drains the
        // in-flight backlog before returning — deadline covers full drain
        // under machine contention.
        shutdown_under_external_deadline(&runtime, Duration::from_secs(120));
        // Resource reclamation FROM THE FULL STATE: every owned thread
        // joined, the lane's occupancy gauge back at the true terminal 0,
        // and the registry terminal — including items still sitting in the
        // ingress queue when fullness was observed.
        assert!(runtime.is_closed());
        assert_eq!(
            runtime.worker_pool().worker_count(),
            0,
            "all owned threads joined after shutdown from the full channel"
        );
        assert_eq!(
            runtime.worker_pool().resize_backlog(),
            0,
            "resize gauge reads the true terminal state after the drain"
        );
        assert_eq!(registry.pending_count(), 0);
        assert!(registry.is_shutdown());
    }

    #[test]
    fn full_channel_oracle_negative_control_below_saturation_stays_not_full() {
        // Review R6 (F811-R2-02): the full-channel oracle must DISCRIMINATE.
        // Same payload, same gauges, same polling loop as the positive
        // test, but the item budget is capped BELOW saturation. Review R7
        // (F811-R7-01): the cap must make not-full an ARITHMETIC property,
        // valid under every schedule — including the receiver
        // (media-resize) thread not having taken a single item when the
        // capped load finishes entering. A budget of CAPACITY−1 accepted
        // items means at most CAPACITY−1 items exist to sit in the
        // channel (backlog ≤ budget) and to compose in-flight states
        // (running ≤ budget); no consumer-scheduling premise is involved.
        // The R6 shape used a CAPACITY-sized budget and leaned on "the
        // resizing item always occupies one slot" — a premise that only
        // exists once the receiver has taken an item, and a controlled
        // 8s receiver stall reproduced its failure. The gauges must read
        // NOT-full for the whole run while real work flows — proving the
        // positive test's thresholds are genuine observations rather than
        // an always-green banner — and the load must still drain to
        // completion through the normal consumption path (no shutdown
        // shortcut).
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        let registry = runtime.registry().clone();
        let payload: Arc<[u8]> = Arc::from(png_bytes(3200, 2400).into_boxed_slice());
        let target = RenditionSpec::viewport(1920, 1440);
        let full_channel = MediaPriorityQueue::CAPACITY as u64;
        let item_budget = MediaPriorityQueue::CAPACITY as u64 - 1;
        // The not-full premise, stated where it is enforced: strictly
        // fewer items than channel slots, so even a fully-stalled
        // receiver cannot fill the channel.
        assert!(
            item_budget < full_channel,
            "not-full control premise: budget {item_budget} must be strictly below the \
             channel capacity {full_channel}"
        );
        let establish_deadline = Instant::now() + Duration::from_secs(30);
        let mut next_index = 0u64;
        let mut accepted = 0usize;
        let mut tickets = Vec::new();
        let mut peak_running = 0usize;
        let mut peak_backlog = 0u64;
        while next_index < item_budget && Instant::now() < establish_deadline {
            let ticket = registry.queue(
                MediaAssetKey {
                    source_fingerprint: format!("not-full-{next_index}"),
                    orientation: Default::default(),
                    rendition: target.clone(),
                    revision: next_index + 1,
                },
                MediaMetadata::default(),
            );
            if runtime.worker_pool().submit_bytes(
                ticket,
                payload.clone(),
                target.clone(),
                MediaPriority::Current,
                0,
                next_index + 1,
            ) {
                accepted += 1;
                next_index += 1;
                tickets.push(ticket);
            } else {
                std::thread::sleep(Duration::from_millis(2));
                continue;
            }
            peak_running = peak_running.max(registry.stats().running);
            peak_backlog = peak_backlog.max(runtime.worker_pool().resize_backlog());
        }
        assert_eq!(
            accepted, item_budget as usize,
            "capped load fully accepted within the deadline"
        );
        // Normal consumption path: all capped items must reach Ready.
        let settle_deadline = Instant::now() + Duration::from_secs(60);
        loop {
            let all_ready = tickets.iter().all(|ticket| {
                matches!(
                    registry.lookup(ticket.id, ticket.revision),
                    MediaLookup::Ready(_)
                )
            });
            peak_running = peak_running.max(registry.stats().running);
            peak_backlog = peak_backlog.max(runtime.worker_pool().resize_backlog());
            if all_ready || Instant::now() > settle_deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(
            tickets.iter().all(|ticket| {
                matches!(
                    registry.lookup(ticket.id, ticket.revision),
                    MediaLookup::Ready(_)
                )
            }),
            "capped load drains to completion: running peak {peak_running}, backlog \
             peak {peak_backlog}"
        );
        assert!(
            peak_running <= item_budget as usize,
            "not-full control: running peak {peak_running} must stay within the item \
             budget ({item_budget} accepted items cannot compose more in-flight states, \
             regardless of receiver scheduling)"
        );
        assert!(
            peak_backlog <= item_budget,
            "not-full control: backlog peak {peak_backlog} must stay within the item \
             budget {item_budget} — at most every accepted item sits in the channel, so a \
             stalled receiver still cannot reach the capacity {full_channel}"
        );
        assert!(
            peak_backlog < full_channel,
            "not-full control: backlog peak {peak_backlog} stays below the channel \
             capacity {full_channel} by arithmetic (budget {item_budget} < capacity), not \
             by a receiver-scheduling premise"
        );
        assert_eq!(
            runtime.worker_pool().resize_backlog(),
            0,
            "gauge drains to exactly zero after normal completion"
        );
        runtime.shutdown();
        assert!(runtime.is_closed());
    }

    // PLAN-811 R1 F811-R1-01/02 closure oracles: producer reaping on close,
    // admission refusal after close, and byte release on drop.

    #[test]
    fn scan_shutdown_reaps_producer_and_registry_stays_quiescent() {
        // Reproduction shape of the review probe: a directory scan in flight
        // when shutdown runs. shutdown() returning now MEANS the scan thread
        // exited (tracked + joined), so the registry must be immutable from
        // that point on - observed as byte-identical stats snapshots across
        // an idle window - and the temp tree must be deletable immediately
        // (no lingering file handles on Windows).
        let dir = std::env::temp_dir().join(format!(
            "p811_scan_reap_{}_{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for i in 0..600 {
            std::fs::write(dir.join(format!("f{i:03}.png")), b"").unwrap();
        }
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        let registry = runtime.registry().clone();
        runtime.scan_directory(&dir);
        // Review R2/R3: gate on OBSERVED producer activity (queued > 0),
        // not a guessed sleep - the close provably races a scan that has
        // started. 600 entries cannot finish inside this window.
        let started = Instant::now();
        loop {
            assert!(
                Instant::now() - started < Duration::from_secs(5),
                "scan producer never became observable"
            );
            if registry.stats().queued > 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        shutdown_under_external_deadline(&runtime, Duration::from_secs(30));
        assert!(runtime.is_closed());

        assert_eq!(
            registry.pending_count(),
            0,
            "closed runtime has no pending work"
        );
        // Quiescence against the media CHANGE GENERATION: it bumps on every
        // registry mutation (queue/transition/publish/fail/expire), so a
        // stable generation across an idle window proves no post-shutdown
        // mutation of any kind (stronger than stats, which lacks counters
        // for failed/corrupt entries).
        let gen_at_close = registry.mutation_generation();
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(
            registry.mutation_generation(),
            gen_at_close,
            "registry stays quiescent after shutdown returned (producer reaped)"
        );

        // File-handle release evidence: an open reader inside the reaped
        // producer would make remove_dir_all fail (Windows sharing mode).
        std::fs::remove_dir_all(&dir).expect("temp tree deletable right after shutdown");
    }

    #[test]
    fn admission_after_close_is_refused_everywhere() {
        // F811-R1-02: a closed runtime refuses new tickets (dead ticket),
        // new sessions (""), and new scans (no-op); repeat shutdown is
        // idempotent; an independent runtime and the process default stay
        // fully operational (positive controls).
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        let registry = runtime.registry().clone();
        runtime.shutdown();

        // queue after close -> dead ticket: never inserted, every operation
        // on it reports missing/expired instead of creating state.
        let dead = registry.queue(
            MediaAssetKey {
                source_fingerprint: "after-close".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::original(),
                revision: 1,
            },
            MediaMetadata::default(),
        );
        assert_eq!(
            registry.lookup(dead.id, dead.revision),
            MediaLookup::Missing
        );
        assert!(registry
            .transition(dead.id, MediaAssetState::Reading)
            .is_err());
        assert!(registry
            .publish_ready(dead.id, dead.revision, Arc::<[u8]>::from([1]))
            .is_err());
        assert!(registry
            .wait_for_terminal(dead.id, Duration::from_millis(10))
            .is_none());
        assert!(!registry.retain(dead.id));
        assert!(!registry.update_metadata(dead.id, MediaMetadata::default()));
        assert_eq!(
            registry.stats().queued,
            0,
            "dead tickets never enter the map"
        );

        // session/scan admission after close.
        let dir = std::env::temp_dir().join(format!(
            "p811_closed_admission_{}_{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.png"), b"").unwrap();
        let session = runtime.open_session(&dir);
        assert_eq!(session, "", "closed runtime admits no session");
        assert_eq!(
            runtime.session_snapshot("image-session-1"),
            "{\"error\":\"unknown_session\"}"
        );
        runtime.scan_directory(&dir);
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(registry.stats().queued, 0, "closed runtime admits no scan");

        // repeat shutdown idempotent.
        runtime.shutdown();
        assert!(runtime.is_closed());

        // positive control: a fresh runtime decodes end to end.
        let other = super::MediaRuntime::new(Duration::from_secs(60));
        let ticket = other.queue_path(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/ui/031-image-viewer/tests/fixtures/rgb-1x1.png"),
            MediaPriority::Current,
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match other.registry().lookup(ticket.id, ticket.revision) {
                MediaLookup::Ready(_) => break,
                MediaLookup::Failed(e) => panic!("decode failed: {e}"),
                _ if Instant::now() > deadline => panic!("decode timeout"),
                _ => std::thread::sleep(Duration::from_millis(5)),
            }
        }

        // and the process-default registry never closes.
        let global = super::global_media_registry();
        let g = global.queue(
            MediaAssetKey {
                source_fingerprint: "default-never-closes".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::original(),
                revision: 1,
            },
            MediaMetadata::default(),
        );
        assert_eq!(global.lookup(g.id, g.revision), MediaLookup::Pending);
        global.release(g.id);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn shutdown_interleaved_with_concurrent_submit_and_open() {
        // Representative close-vs-producer interleaving: submitters and
        // session openers race shutdown; afterwards the runtime must be
        // quiescent and immutable (whatever interleaving won, no half-open
        // producer survives close).
        let dir = std::env::temp_dir().join(format!(
            "p811_interleave_{}_{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for i in 0..40 {
            std::fs::write(dir.join(format!("f{i:02}.png")), b"").unwrap();
        }
        use std::sync::atomic::Ordering;
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        let registry = runtime.registry().clone();
        let stop = std::sync::atomic::AtomicBool::new(false);
        let runtime_ref = &runtime;
        std::thread::scope(|scope| {
            for t in 0..4u32 {
                let stop_ref = &stop;
                let dir = dir.clone();
                scope.spawn(move || {
                    let mut n = 0u64;
                    while !stop_ref.load(Ordering::Relaxed) {
                        if t % 2 == 0 {
                            runtime_ref.queue_path(
                                dir.join(format!("f{:02}.png", n % 40)),
                                MediaPriority::Thumbnail,
                            );
                        } else {
                            let _ = runtime_ref.open_session(&dir);
                        }
                        n += 1;
                    }
                });
            }
            std::thread::sleep(Duration::from_millis(30));
            shutdown_under_external_deadline(&runtime, Duration::from_secs(30));
            stop.store(true, Ordering::Relaxed);
        });
        assert_eq!(registry.pending_count(), 0);
        let first = registry.stats();
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(format!("{first:?}"), format!("{:?}", registry.stats()));
        std::fs::remove_dir_all(&dir).ok();
    }

    // PLAN-811 R2 F811-R2-01: session admission vs shutdown interleaving.

    use std::path::PathBuf;

    /// Deterministic interleaving gate: the Into<PathBuf> conversion blocks
    /// inside open_session between the entry check and the table lock —
    /// exactly the scheduler window the review reproduced. Legal public API
    /// only (a custom Into impl is a normal caller capability).
    struct GatedPath {
        path: PathBuf,
        entered: std::sync::mpsc::Sender<()>,
        release: std::sync::mpsc::Receiver<()>,
    }
    impl From<GatedPath> for PathBuf {
        fn from(gated: GatedPath) -> PathBuf {
            let _ = gated.entered.send(());
            // Park until the main thread finished a full shutdown().
            let _ = gated.release.recv_timeout(Duration::from_secs(30));
            gated.path
        }
    }

    #[test]
    fn open_session_racing_shutdown_never_inserts_a_live_session() {
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        let (entered_tx, entered_rx) = std::sync::mpsc::channel::<()>();
        let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();
        let returned = std::thread::scope(|scope| {
            let runtime_ref = &runtime;
            let opener = scope.spawn(move || {
                let gated = GatedPath {
                    path: PathBuf::from("."),
                    entered: entered_tx,
                    release: release_rx,
                };
                runtime_ref.open_session(gated)
            });
            // Wait until the racing open is parked INSIDE the conversion
            // window.
            entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            // Complete a full shutdown while the open is parked.
            shutdown_under_external_deadline(runtime_ref, Duration::from_secs(30));
            assert!(runtime.is_closed());
            // Release the racing open: it must refuse admission — no live
            // session may reappear in the cleared table.
            release_tx.send(()).unwrap();
            opener.join().unwrap()
        });
        assert_eq!(returned, "", "racing open after full shutdown refuses");
        // The table stays empty: any id answers unknown_session.
        assert_eq!(
            runtime.session_snapshot("image-session-1"),
            "{\"error\":\"unknown_session\"}"
        );
        assert!(!runtime.close_session("image-session-1"));
    }

    #[test]
    fn open_session_sequential_controls() {
        // Post-close (sequential): refuses outright.
        let closed = super::MediaRuntime::new(Duration::from_secs(60));
        closed.shutdown();
        assert_eq!(closed.open_session(fixtures_path()), "");

        // Live runtime: a real open produces a queryable session that
        // populate fills and close retires (positive control).
        let live = super::MediaRuntime::new(Duration::from_secs(60));
        let session = live.open_session(fixtures_path());
        assert!(session.starts_with("image-session-"), "{session}");
        let deadline = Instant::now() + Duration::from_secs(2);
        while live.session_names(&session).is_empty() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            live.session_names(&session)
                .iter()
                .any(|n| n == "rgb-1x1.png"),
            "populate filled the live session"
        );
        assert!(live.close_session(&session));
        assert!(!live.close_session(&session));
    }

    fn fixtures_path() -> PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures")
    }

    #[test]
    fn drop_releases_decoded_pixel_bytes() {
        // Weak-reference release evidence: after a Ready rendition, the only
        // strong owner of the decoded RGBA bytes is the runtime pixel cache.
        // Dropping the runtime (worker lanes joined by Drop) must invalidate
        // the Weak - bytes are actually freed, not just hidden.
        let runtime = super::MediaRuntime::new(Duration::from_secs(60));
        let registry = runtime.registry().clone();
        let ticket = runtime.queue_path(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/ui/031-image-viewer/tests/fixtures/rgb-1x1.png"),
            MediaPriority::Current,
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match registry.lookup(ticket.id, ticket.revision) {
                MediaLookup::Ready(_) => break,
                MediaLookup::Failed(e) => panic!("decode failed: {e}"),
                _ if Instant::now() > deadline => panic!("decode timeout"),
                _ => std::thread::sleep(Duration::from_millis(5)),
            }
        }
        let (_, _, bytes) = runtime
            .worker_pool()
            .decoded_pixels(ticket.id)
            .expect("decoded pixels present");
        let weak = Arc::downgrade(&bytes);
        drop(bytes);
        assert!(weak.upgrade().is_some(), "runtime still owns the bytes");
        drop(runtime);
        assert!(
            weak.upgrade().is_none(),
            "decoded bytes released after runtime drop"
        );
    }

    #[test]
    fn owned_runtimes_isolate_tickets_and_sessions_from_the_shared_default() {
        let local = super::MediaRuntime::new(Duration::from_secs(60));
        let ticket = local.queue_path(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/ui/031-image-viewer/tests/fixtures/rgb-1x1.png"),
            MediaPriority::Current,
        );
        // The local ticket is invisible through the process-default
        // registry — and vice versa, default-global assets never leak into
        // the local runtime.
        assert_eq!(
            super::global_media_registry().lookup(ticket.id, ticket.revision),
            MediaLookup::Missing
        );
        let session = local.open_session(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/ui/031-image-viewer/tests/fixtures"),
        );
        assert_eq!(
            super::media_session_snapshot(&session),
            "{\"error\":\"unknown_session\"}",
            "local session ids are not visible on the default runtime"
        );
        local.shutdown();
    }

    // PLAN-095 T-02: renderer-facing notify contract (status projection +
    // lifecycle generation wake source).

    #[test]
    fn media_surface_status_projects_notify_contract() {
        use super::{media_surface_status, MediaSurfaceStatus};
        // Non-media URIs stay outside the notify contract (placeholder
        // display semantics unchanged for legacy components).
        assert_eq!(
            media_surface_status("https://example.com/x.png"),
            MediaSurfaceStatus::NotMedia
        );
        assert_eq!(media_surface_status(""), MediaSurfaceStatus::NotMedia);
        // Media-prefixed but malformed ticket path = terminal failure with a
        // locatable reason (缺票族——占位图不会自行变好).
        assert!(matches!(
            media_surface_status("/api/__auto/media/notahexticket/1"),
            MediaSurfaceStatus::Failed(_)
        ));
        // Well-formed yet never-queued ticket = missing → failed.
        assert!(matches!(
            media_surface_status("/api/__auto/media/00000000000000000000000000000000/1"),
            MediaSurfaceStatus::Failed(reason) if reason.contains("missing")
        ));
    }

    #[test]
    fn metadata_and_limits_identify_png_and_reject_invalid_or_excessive_inputs() {
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::new(2, 3))
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let metadata = super::inspect_image_metadata(bytes.get_ref()).unwrap();
        assert_eq!(
            (metadata.width, metadata.height, metadata.mime_type.as_str()),
            (2, 3, "image/png")
        );
        assert_eq!(
            super::inspect_image_metadata(b"not an image"),
            Err(super::MediaAssetError::UnsupportedFormat)
        );
        assert_eq!(
            super::validate_image_limits(2, 3, super::MAX_IMAGE_FILE_BYTES + 1),
            Err(super::MediaAssetError::FileTooLarge)
        );
    }

    #[test]
    fn decode_and_rendition_applies_orientation_and_preserves_rgba() {
        let mut bytes = std::io::Cursor::new(Vec::new());
        let mut source = image::RgbaImage::new(2, 3);
        source.put_pixel(1, 2, image::Rgba([4, 5, 6, 7]));
        image::DynamicImage::ImageRgba8(source)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let rendered = super::decode_rendition(
            bytes.get_ref(),
            super::MediaOrientation::Rotate90,
            &RenditionSpec::original(),
        )
        .unwrap();
        assert_eq!((rendered.width, rendered.height), (3, 2));
        assert_eq!(rendered.rgba.len(), 3 * 2 * 4);
        assert!(rendered
            .rgba
            .chunks_exact(4)
            .any(|pixel| pixel == [4, 5, 6, 7]));

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
        assert_eq!(
            super::media_http_response(&registry, "GET", &path, None).status,
            503
        );

        registry
            .transition(ticket.id, MediaAssetState::Reading)
            .unwrap();
        registry
            .transition(ticket.id, MediaAssetState::Decoding)
            .unwrap();
        registry
            .transition(ticket.id, MediaAssetState::Transforming)
            .unwrap();
        registry
            .publish_ready(ticket.id, ticket.revision, Arc::<[u8]>::from([1, 2, 3]))
            .unwrap();
        let response = super::media_http_response(&registry, "GET", &path, None);
        assert_eq!(response.status, 200);
        assert_eq!(response.body.as_deref(), Some(&[1, 2, 3][..]));
        assert_eq!(
            response.headers.get("content-type"),
            Some(&"image/png".into())
        );
        assert_eq!(
            super::media_http_response(&registry, "HEAD", &path, None).body,
            None
        );
        assert_eq!(
            super::media_http_response(
                &registry,
                "GET",
                &path,
                Some(response.headers["etag"].as_str())
            )
            .status,
            304
        );
        assert_eq!(
            super::media_http_response(&registry, "GET", "/api/__auto/media/nope/9", None).status,
            404
        );
        assert!(!format!("{:?}", response).contains("private/source"));

        let failed = registry.queue(
            MediaAssetKey {
                source_fingerprint: "fixture-b".into(),
                orientation: Default::default(),
                rendition: RenditionSpec::original(),
                revision: 10,
            },
            MediaMetadata::default(),
        );
        registry
            .fail(failed.id, super::MediaAssetError::UnsupportedFormat)
            .unwrap();
        let failed_path = format!("/api/__auto/media/{}/{}", failed.id, failed.revision);
        assert_eq!(
            super::media_http_response(&registry, "GET", &failed_path, None).status,
            422
        );
        registry.release(ticket.id);
        registry.collect_expired();
        assert_eq!(
            super::media_http_response(&registry, "GET", &path, None).status,
            410
        );
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
        registry
            .transition(ticket.id, MediaAssetState::Reading)
            .unwrap();
        registry
            .transition(ticket.id, MediaAssetState::Decoding)
            .unwrap();
        registry
            .transition(ticket.id, MediaAssetState::Transforming)
            .unwrap();
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
        // Plan 547 渲染修复批次:+scenic-320x240.png(可辨识风景图,
        // Open File 默认;肉眼可辨"真图上屏",纯函数生成无版权负担)。
        assert_eq!(fixtures.len(), 7);
        for fixture in fixtures {
            let name = fixture["file"].as_str().unwrap();
            let bytes = std::fs::read(root.join(name)).unwrap();
            assert!(!bytes.is_empty(), "fixture {name} is empty");
            match fixture["kind"].as_str().unwrap() {
                "jpeg" => assert!(bytes.starts_with(&[0xff, 0xd8, 0xff]), "{name} is not JPEG"),
                "png" => assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"), "{name} is not PNG"),
                "webp" => assert!(
                    bytes.starts_with(b"RIFF") && bytes[8..].starts_with(b"WEBP"),
                    "{name} is not WebP"
                ),
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
