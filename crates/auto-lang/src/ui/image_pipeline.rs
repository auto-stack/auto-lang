//! Shared, renderer-neutral image asset pipeline.
//!
//! The public media types, bounded scheduler, cache registry, and HTTP helper
//! live here so VM, generated Rust, and an HTTP backend use identical asset
//! identities and lifetimes.  Task 1 deliberately provides only the module
//! seam; its types and behavior are added in the following tasks.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
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
}
