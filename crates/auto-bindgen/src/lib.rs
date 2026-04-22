//! auto-bindgen: C header manifest types and built-in definitions.
//!
//! This crate provides:
//! - `manifest` — Serde types for JSON manifests (`CHeaderManifest`, `CFunction`, etc.)
//! - `extractor` — Hard-coded manifests for 5 standard C headers
//! - `type_map` — Utility conversions for CTypeDesc
//!
//! Plan 216 Phase 1.

pub mod extractor;
pub mod manifest;
pub mod type_map;
