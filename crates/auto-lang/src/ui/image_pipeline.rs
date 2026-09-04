//! Shared, renderer-neutral image asset pipeline.
//!
//! The public media types, bounded scheduler, cache registry, and HTTP helper
//! live here so VM, generated Rust, and an HTTP backend use identical asset
//! identities and lifetimes.  Task 1 deliberately provides only the module
//! seam; its types and behavior are added in the following tasks.

use std::fmt;

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
    use super::{checked_rgba_bytes, MediaAssetState};

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
}
