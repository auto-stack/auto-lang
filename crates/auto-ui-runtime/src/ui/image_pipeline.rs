//! Shared, renderer-neutral image asset pipeline — thin re-export facade.
//!
//! PLAN-811: the complete pipeline (public media types, registry, caches,
//! scheduler, worker pool, viewer sessions, owned `MediaRuntime`, and the
//! transport-neutral HTTP rules) lives in the `auto-media-core` crate,
//! migrated verbatim. This module preserves the historical in-`ui` path with
//! UNCHANGED type identity — every re-exported name here IS the
//! `auto_media_core::image_pipeline` item, and `global_media_registry()` /
//! `global_media_worker_pool()` still expose the one process-shared runtime
//! behind the legacy free functions. No second registry, worker pool, or
//! session table exists on this path.

pub use auto_media_core::image_pipeline::{
    checked_rgba_bytes, close_media_session, decode_rendition, global_media_registry,
    global_media_runtime, global_media_worker_pool, inspect_image_metadata,
    media_change_generation, media_http_response, media_session_names, media_session_snapshot,
    media_session_stats, media_session_uri, media_surface_status, navigate_media_session,
    open_media_session, pending_media_count, queue_media_path, queue_media_thumbnail,
    request_media_view, resolve_media_pixels, resolve_media_render, resolve_media_uri,
    scan_media_directory, validate_image_limits, DecodedPixelCache, EncodedByteLru,
    MediaAssetError, MediaAssetId, MediaAssetKey, MediaAssetRegistry, MediaAssetState,
    MediaAssetTicket, MediaHttpResponse, MediaLatestWins, MediaLookup, MediaMetadata,
    MediaOrientation, MediaPin, MediaPipelineStats, MediaPriority, MediaPriorityQueue,
    MediaRuntime, MediaSurfaceStatus, MediaWork, MediaWorkerPool, RenderedImage, RenditionSpec,
    MAX_IMAGE_FILE_BYTES, MAX_IMAGE_PIXELS,
};
