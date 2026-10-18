//! Backend-neutral local media service — thin re-export facade.
//!
//! PLAN-811: the service (recursive index, blake3 tokens, natural ordering,
//! no-symlink traversal, HTTP byte-range semantics) moved verbatim into the
//! `auto-media-core` crate. This module preserves the historical in-`ui`
//! ungated path with UNCHANGED type identity — every re-exported name here
//! IS the `auto_media_core::media_service` item. The generated backend
//! reaches it as `auto_lang::ui::media_service` unchanged.

pub use auto_media_core::media_service::{
    content_range, content_type, display_title, entry_path, find, human_size, index_directory,
    parse_artist_and_title, parse_range, resolve_root, MediaEntry, MediaIndex, StreamPlan,
    SUPPORTED_EXTENSIONS,
};
