//! Media foundation crate (PLAN-811).
//!
//! Single source for the renderer-neutral media mechanisms that used to live
//! in `auto-lang`'s `ui` directory:
//!
//! * [`image_pipeline`] — public media values (asset id/key/ticket/state),
//!   the process-shared asset registry, byte-accounted caches, the bounded
//!   priority scheduler, the decode/resize worker pool, host-owned viewer
//!   sessions, and the transport-neutral HTTP rules for
//!   `/api/__auto/media/{id}/{revision}`.
//! * [`media_service`] — the local media file service: recursive index,
//!   blake3 tokens, natural ordering, no-symlink traversal, and HTTP
//!   byte-range (`Range` → `StreamPlan`) semantics.
//!
//! The crate is deliberately free of auto-lang/auto-man/auto-ui/auto-vm,
//! interpreter, and renderer dependencies; codecs (`image`/`kamadak-exif`)
//! are gated behind the `image-codec` feature. Consumers that need the
//! historical in-`ui` paths keep them as thin re-export facades with
//! unchanged type identity — `auto_lang::ui::image_pipeline::MediaAssetId`
//! IS `auto_media_core::image_pipeline::MediaAssetId`.

/// Renderer-neutral image asset pipeline (PLAN-811; verbatim migration of
/// auto-lang's `ui/image_pipeline.rs` data plane + runtime).
#[cfg(feature = "image-codec")]
pub mod image_pipeline;

/// Backend-neutral local media file service (recursive index, blake3 tokens,
/// natural ordering, no-symlink traversal, HTTP byte-range semantics) —
/// verbatim migration of auto-lang's `ui/media_service.rs` (Plan 617 origin).
pub mod media_service;
